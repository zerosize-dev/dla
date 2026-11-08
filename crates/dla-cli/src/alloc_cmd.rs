//! `dla alloc-test <H> [--finding <F>] [--column <name>]... [--purge <kind>] [--extra <k>]`.
//!
//! Tier 1. Re-runs harness H with `iterations + extra` points and a purge applied at the
//! last point, then tells a leak from allocator or collector headroom, a warming cache and
//! a pool high-water mark: a plateau test over the extra window, a purge test against the
//! purge event, and a gc test over `.pre_gc` twins. Writes the new series (source kind
//! `alloc-test`), its trend, `alloc/A<n>.json` with every number, and one `discriminator`
//! evidence.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    EvidenceId, Finding, FindingId, FindingState, Ledger, MAX_RESULT_BYTES,
    discriminator::{DiscriminatorResult, Favors},
    error::codes,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind},
    finding::Polarity,
    fsutil::{to_pretty_json, write_atomic},
    harness::{HarnessManifest, HarnessMode},
    runtime::Runtime,
    series::{ColumnKind, ColumnScope, KeyMode, SeriesFile, SeriesSourceKind, Unit, next_number},
    toolctx::{EvidenceSpec, ToolCtx},
    trend::{ColumnTrend, GrowthSummary, TREND_SCHEMA_VERSION, TrendReport},
};
use dla_stats::{GateThresholds, GateVerdict, analyze};
use dla_workload::harness::run::{RunOptions, run_points};
use serde_json::{Map, Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
    stage,
};

#[path = "alloc/judge.rs"]
mod judge;
#[path = "alloc/pick.rs"]
mod pick;

use judge::{Gc, Plateau, PlateauVerdict, Purge, PurgeVerdict};

/// Directory (inside the run directory) holding the full alloc-test results.
const ALLOC_DIR: &str = "alloc";
/// Purge kinds the flag accepts.
const PURGES: &[&str] = &[
    "auto",
    "none",
    "trim",
    "gc",
    "free-os-memory",
    "trim-native-heap",
    "jemalloc",
    "empty-cache",
];
/// Fewest extra points the plateau test can use.
const MIN_EXTRA: u32 = 3;
/// Most `next` commands printed.
const MAX_NEXT: usize = 8;

/// Flags of `dla alloc-test`.
#[derive(Args, Debug)]
pub struct AllocTestArgs {
    /// Harness ID from `dla harness gen` (`H1`).
    #[arg(value_name = "H")]
    harness: String,
    /// The finding the test is bound to: its probe, site group or region column decides.
    #[arg(long, value_name = "F", value_parser = parse_enum::<FindingId>)]
    finding: Option<FindingId>,
    /// Test this column (repeatable); default: the finding's column, else every column that
    /// passed the gate in the latest trend of the harness, plus os.rss.
    #[arg(long = "column", value_name = "NAME")]
    columns: Vec<String>,
    /// Purge applied at the last point: auto | none | trim | gc | free-os-memory |
    /// trim-native-heap | jemalloc | empty-cache.
    #[arg(long, value_name = "KIND", default_value = "auto",
        value_parser = clap::builder::PossibleValuesParser::new(PURGES))]
    purge: String,
    /// Extra points after the measured ones (default: the harness's iteration count).
    #[arg(long, value_name = "K")]
    extra: Option<u32>,
}

/// The purge `auto` stands for in each runtime.
fn auto_purge(runtime: Runtime) -> &'static str {
    match runtime {
        Runtime::Jvm => "trim-native-heap",
        Runtime::Go => "free-os-memory",
        Runtime::Python | Runtime::Rust | Runtime::Native => "trim",
        Runtime::Node => "gc",
    }
}

/// The purge event of a series: its kind and the readings before and after.
struct PurgeEvent {
    kind: String,
    before: Map<String, Value>,
    after: Map<String, Value>,
}

fn purge_event(series: &SeriesFile) -> Option<PurgeEvent> {
    let ev = series
        .events
        .iter()
        .rev()
        .find(|e| e.kind.starts_with("purge:"))?;
    let detail: Value = serde_json::from_str(ev.detail.as_deref()?).ok()?;
    let map = |k: &str| {
        detail
            .get(k)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    };
    Some(PurgeEvent {
        kind: detail
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or_else(|| ev.kind.trim_start_matches("purge:"))
            .to_owned(),
        before: map("before"),
        after: map("after"),
    })
}

/// Everything the tests found for one column.
struct ColumnResult {
    name: String,
    voting: bool,
    plateau: Plateau,
    purge: Option<Purge>,
    gc: Option<Gc>,
    favors: Favors,
    supports: Vec<String>,
}

impl ColumnResult {
    /// The evidence form (nulls dropped).
    fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("name".into(), json!(self.name));
        m.insert("plateau".into(), compact(self.plateau.to_json()));
        let purge = match &self.purge {
            Some(p) if p.verdict != PurgeVerdict::NotRun => compact(p.to_json()),
            Some(p) => json!({"kind": p.kind, "verdict": "not-run"}),
            None => json!({"verdict": "not-run"}),
        };
        m.insert("purge".into(), purge);
        if let Some(g) = self.gc {
            m.insert("gc".into(), g.to_json());
        }
        m.insert("favors".into(), json!(self.favors));
        Value::Object(m)
    }

    /// A one-line form for the command output.
    fn brief(&self) -> Value {
        json!({
            "name": self.name,
            "plateau": self.plateau.verdict.as_str(),
            "purge": self.purge.as_ref().map_or("not-run", |p| p.verdict.as_str()),
            "favors": self.favors,
        })
    }
}

/// Drops the null members of an object.
fn compact(v: Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(m.into_iter().filter(|(_, x)| !x.is_null()).collect()),
        other => other,
    }
}

/// The value of column `idx` at the last warm-up point that has one.
fn warm_end(series: &SeriesFile, idx: usize) -> Option<f64> {
    series
        .points
        .iter()
        .take(series.warmup as usize)
        .rev()
        .find_map(|p| p.values.get(idx).copied().flatten())
}

/// The level a process column's rise is judged against besides its own: the process's
/// resident memory. Zero for every column that is not an `os.*` byte column (a probe or
/// heap column is measured precisely and is judged against its own level only), or when
/// `os.rss` was not sampled.
fn reference_level(series: &SeriesFile, idx: usize) -> f64 {
    let col = &series.columns[idx];
    if col.unit != Unit::Bytes || !col.name.starts_with("os.") {
        return 0.0;
    }
    let Some(rss) = series.column_index("os.rss") else {
        return 0.0;
    };
    let pts = series.post_warmup(rss);
    if pts.is_empty() {
        return 0.0;
    }
    pts.iter().map(|p| p.1.abs()).sum::<f64>() / pts.len() as f64
}

/// The x of the last main-window point: the `iterations`-th point after the warm-up, or the
/// last point when the series is shorter.
fn main_end_x(series: &SeriesFile, iterations: usize) -> f64 {
    let end = series.warmup as usize + iterations;
    series
        .points
        .get(end.saturating_sub(1))
        .or_else(|| series.points.last())
        .map_or(f64::INFINITY, |p| p.x)
}

fn test_column(
    series: &SeriesFile,
    name: &str,
    voting: bool,
    iterations: usize,
    event: Option<&PurgeEvent>,
    limitations: &mut Vec<String>,
) -> Option<ColumnResult> {
    let idx = series.column_index(name)?;
    let counter = series.columns[idx].kind == ColumnKind::Counter;
    let points = series.post_warmup(idx);
    let (main, extra) = judge::split(&points, main_end_x(series, iterations));
    if main.len() < iterations {
        limitations.push(format!(
            "column {name} has {} of {iterations} main-window readings (missing values); its plateau test used what exists",
            main.len()
        ));
    }
    if extra.is_empty() {
        limitations.push(format!(
            "column {name} has no extra-window readings: its plateau test was not run"
        ));
    }
    let reference = reference_level(series, idx);
    let plateau = judge::plateau(main, extra, counter, reference);
    let purge = event.map(|e| {
        if counter {
            return judge::purge(&e.kind, None, None, None);
        }
        let read = |m: &Map<String, Value>| m.get(name).and_then(Value::as_f64);
        judge::purge(
            &e.kind,
            read(&e.before),
            read(&e.after),
            warm_end(series, idx),
        )
    });
    let gc = (!counter)
        .then(|| series.column_index(&format!("{name}.pre_gc")))
        .flatten()
        .and_then(|pre| judge::gc(&series.post_warmup(pre), &points, reference));
    let (favors, supports) = judge::combine(plateau.verdict, purge.as_ref(), gc);
    Some(ColumnResult {
        name: name.to_owned(),
        voting,
        plateau,
        purge,
        gc,
        favors,
        supports,
    })
}

/// The trend of the new series: every column over its post-warm-up points.
fn trend_of(
    tc: &ToolCtx,
    ledger: &Ledger,
    series: &SeriesFile,
    previous: Option<&TrendReport>,
) -> Res<TrendReport> {
    let th = GateThresholds::default();
    let columns = series
        .columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let counter = col.kind == ColumnKind::Counter;
            let tail = counter
                .then(|| {
                    series
                        .points
                        .iter()
                        .take(series.warmup as usize)
                        .rev()
                        .find_map(|p| Some((p.x, p.values.get(i).copied().flatten()?)))
                })
                .flatten();
            let a = analyze(&series.post_warmup(i), tail, counter, &th);
            let (mut gate, mut gate_failed) = (a.gate, a.gate_failed);
            if col.scope == ColumnScope::Container {
                gate = GateVerdict::Fail;
                gate_failed.push("container scope".to_owned());
            }
            let mut candidates: Vec<FindingId> = Vec::new();
            let mut push = |f: &FindingId| {
                if !candidates.contains(f) {
                    candidates.push(f.clone());
                }
            };
            if let Some(f) = col.finding.as_ref().filter(|f| ledger.finding(f).is_some()) {
                push(f);
            }
            if let Some(site) = &col.site {
                for f in ledger.findings() {
                    if f.polarity() == Polarity::Leak && f.site.as_ref() == Some(site) {
                        push(&f.id);
                    }
                }
            }
            if let Some(old) = previous.and_then(|t| t.columns.iter().find(|c| c.name == col.name))
            {
                for f in &old.candidates {
                    push(f);
                }
            }
            ColumnTrend {
                name: col.name.clone(),
                unit: col.unit,
                kind: col.kind,
                region: col.region,
                site: col.site.clone(),
                finding: col.finding.clone(),
                candidates,
                n: a.n,
                mk: a.mk,
                sen: a.sen,
                halves: a.halves,
                every_delta: a.every_delta,
                noise: a.noise,
                rise: a.rise,
                shape: a.shape,
                gate,
                gate_failed,
            }
        })
        .collect();
    let report = TrendReport {
        schema_version: TREND_SCHEMA_VERSION,
        id: tc.next_trend_id(),
        series: series.id.clone(),
        thresholds: th,
        columns,
    };
    report.save(&tc.run_dir)?;
    Ok(report)
}

/// The key mode the test measures: the manifest's first, or fresh for a driver.
/// The newest harness series of `m` in `mode` that holds every warm-up and main point
/// (command mode), so an alloc-test can run only its extra points.
fn complete_series(run_dir: &Path, m: &HarnessManifest, mode: KeyMode) -> Option<SeriesFile> {
    let want = (m.warmup + m.iterations) as usize;
    let dir = run_dir.join(dla_core::series::SERIES_DIR);
    let mut ids: Vec<(u32, String)> = fs::read_dir(&dir)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(".json")?.to_owned();
            Some((dla_core::series::numbered('S', &id)?, id))
        })
        .collect();
    ids.sort();
    ids.into_iter().rev().find_map(|(_, id)| {
        let s = SeriesFile::load(run_dir, &id).ok()?;
        (s.source.kind == SeriesSourceKind::Harness
            && s.source.harness.as_deref() == Some(m.id.as_str())
            && s.source.mode == Some(mode)
            && s.warmup == m.warmup
            && s.points.len() == want
            && s.points
                .iter()
                .enumerate()
                .all(|(i, p)| (p.x - m.step as f64 * (i as f64 + 1.0)).abs() < 0.5))
        .then_some(s)
    })
}

/// `prev` (warm-up and main points of a harness run) followed by `extra` (the extra
/// points of this alloc-test): columns joined by name, missing values null, the
/// extra series' events and source, both sets of limitations.
fn join_series(prev: &SeriesFile, extra: &SeriesFile) -> SeriesFile {
    let mut columns = prev.columns.clone();
    for c in &extra.columns {
        if !columns.iter().any(|p| p.name == c.name) {
            columns.push(c.clone());
        }
    }
    let row = |s: &SeriesFile, p: &dla_core::series::Point| dla_core::series::Point {
        x: p.x,
        t_ms: p.t_ms,
        values: columns
            .iter()
            .map(|c| {
                s.column_index(&c.name)
                    .and_then(|i| p.values.get(i).copied().flatten())
            })
            .collect(),
    };
    let mut points: Vec<_> = prev.points.iter().map(|p| row(prev, p)).collect();
    points.extend(extra.points.iter().map(|p| row(extra, p)));
    let offset = prev.points.len() as u32;
    let events = extra
        .events
        .iter()
        .map(|e| dla_core::series::SeriesEvent {
            after_point: e.after_point + offset,
            ..e.clone()
        })
        .collect();
    let mut limitations = prev.limitations.clone();
    limitations.extend(extra.limitations.iter().cloned());
    limitations.push(format!(
        "points 1 to {offset} come from {}; the extra points from {}",
        prev.id, extra.id
    ));
    limitations.dedup();
    if columns.len() > dla_core::series::MAX_COLUMNS {
        limitations.push(format!(
            "{} columns beyond the {} the series allows were dropped",
            columns.len() - dla_core::series::MAX_COLUMNS,
            dla_core::series::MAX_COLUMNS
        ));
    }
    columns.truncate(dla_core::series::MAX_COLUMNS);
    SeriesFile {
        schema_version: extra.schema_version,
        id: extra.id.clone(),
        source: extra.source.clone(),
        axis: prev.axis,
        warmup: prev.warmup,
        points: points
            .into_iter()
            .map(|mut p| {
                p.values.truncate(columns.len());
                p
            })
            .collect(),
        columns,
        events,
        limitations,
    }
}

fn key_mode(m: &HarnessManifest) -> KeyMode {
    match m.mode {
        HarnessMode::Driver => KeyMode::Fresh,
        HarnessMode::Command => m.key_modes.first().copied().unwrap_or(KeyMode::Native),
    }
}

fn bad_harness_id(id: &str) -> Failure {
    Failure::usage(
        format!("{id:?} is not a harness ID"),
        "a harness ID such as H1",
        "dla alloc-test H1",
    )
}

/// Runs `dla alloc-test`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: AllocTestArgs) -> Res<Value> {
    if !a.harness.starts_with('H') {
        return Err(bad_harness_id(&a.harness));
    }
    let tc = stage::tool_ctx(project, ctx, data);
    let manifest = tc.harness(&a.harness).map_err(|mut r| {
        r.fix = format!(
            "dla harness gen --lang <lang> then dla alloc-test {}   (was: {})",
            a.harness, r.fix
        );
        r
    })?;
    let mut ledger = tc.ledger()?;
    let bound: Option<Finding> = match &a.finding {
        Some(f) => Some(ledger.finding(f).cloned().ok_or_else(|| {
            Failure::refuse(
                codes::UNKNOWN_FINDING,
                format!("{f} is not in the ledger"),
                "an existing finding for --finding",
                "dla run status   (lists findings)",
            )
        })?),
        None => None,
    };
    let extra = a.extra.unwrap_or(manifest.iterations);
    if extra < MIN_EXTRA {
        return Err(Failure::usage(
            format!("--extra {extra} leaves no plateau window"),
            format!("--extra of at least {MIN_EXTRA}"),
            format!(
                "dla alloc-test {} --extra {}",
                a.harness,
                manifest.iterations.max(MIN_EXTRA)
            ),
        ));
    }
    let purge_kind = match a.purge.as_str() {
        "auto" => Some(auto_purge(manifest.runtime).to_owned()),
        "none" => None,
        other => Some(other.to_owned()),
    };

    // Command mode: a complete harness series of the same mode already holds the main
    // window, so only the extra points are run and the two are joined.
    let reuse = (manifest.mode == HarnessMode::Command)
        .then(|| complete_series(&tc.run_dir, &manifest, key_mode(&manifest)))
        .flatten();
    let opts = RunOptions {
        modes: vec![key_mode(&manifest)],
        iterations: Some(manifest.iterations),
        extra,
        purge: purge_kind.clone(),
        source_kind: SeriesSourceKind::AllocTest,
        first_point: reuse
            .as_ref()
            .map_or(1, |_| manifest.warmup + manifest.iterations + 1),
        ..RunOptions::default()
    };
    let out = run_points(&tc, &manifest, &opts)?;
    let series_id = out.series.first().cloned().ok_or_else(|| {
        Failure::refuse(
            "harness-failed",
            format!("harness {} produced no series", a.harness),
            "at least one measured point",
            format!("dla harness run {}", a.harness),
        )
    })?;
    let mut series = SeriesFile::load(&tc.run_dir, &series_id)?;
    if let Some(prev) = &reuse {
        series = join_series(prev, &series);
        series.id = tc.next_series_id();
        series.save(&tc.run_dir)?;
    }

    // Columns.
    let previous = pick::latest_trend(&tc.run_dir, &a.harness);
    let picked = pick::pick(
        &series,
        &ledger,
        bound.as_ref(),
        &a.columns,
        previous.as_ref(),
        &a.harness,
    )?;
    let event = purge_event(&series);
    let mut limitations: Vec<String> = series.limitations.clone();
    limitations.extend(out.gaps.iter().cloned());
    match (&purge_kind, &event) {
        (None, _) => {
            limitations.push("purge disabled (--purge none): the purge test was not run".into())
        }
        (Some(k), None) => limitations.push(format!(
            "purge {k} produced no before/after reading: the purge test was not run"
        )),
        (Some(_), Some(e)) if e.before.is_empty() || e.after.is_empty() => limitations.push(
            "the purge event carries no before/after readings: the purge test was not run".into(),
        ),
        _ => {}
    }
    let iterations = manifest.iterations as usize;
    let mut results: Vec<ColumnResult> = picked
        .tested
        .iter()
        .filter_map(|n| {
            test_column(
                &series,
                n,
                picked.voting.contains(n),
                iterations,
                event.as_ref(),
                &mut limitations,
            )
        })
        .collect();
    // Voting columns first, in the order they were picked.
    results.sort_by_key(|r| !r.voting);
    if results
        .iter()
        .all(|r| r.plateau.verdict == PlateauVerdict::NotRun)
    {
        limitations.push(format!(
            "the extra window has fewer than {MIN_EXTRA} points: the plateau test was not run (see the series limitations)"
        ));
    }

    // The verdict: the bound column's, or the voting columns together.
    let deciding: Vec<(Favors, Vec<String>)> = results
        .iter()
        .filter(|r| match &picked.primary {
            Some(p) => &r.name == p,
            None => r.voting,
        })
        .map(|r| (r.favors, r.supports.clone()))
        .collect();
    let (favors, supports) = judge::overall(&deciding);

    // Files: the new trend and the full result.
    let trend = trend_of(&tc, &ledger, &series, previous.as_ref())?;
    let n = next_number(&tc.run_dir.join(ALLOC_DIR), 'A', ".json", false);
    let alloc_id = format!("A{n}");
    let raw_rel = format!("{ALLOC_DIR}/{alloc_id}.json");
    let full = json!({
        "id": alloc_id,
        "harness": a.harness,
        "series": series.id,
        "trend": trend.id,
        "finding": a.finding,
        "basis": picked.basis,
        "purge": purge_kind,
        "iterations": manifest.iterations,
        "extra": extra,
        "favors": favors,
        "supports": supports,
        "primary": picked.primary,
        "limitations": limitations,
        "columns": results.iter().map(|r| {
            let mut v = r.to_json();
            v["voting"] = json!(r.voting);
            v["supports"] = json!(r.supports);
            v["plateau"] = r.plateau.to_json();
            if let Some(p) = &r.purge { v["purge"] = p.to_json(); }
            v
        }).collect::<Vec<_>>(),
    });
    let dir = tc.run_dir.join(ALLOC_DIR);
    fs::create_dir_all(&dir).map_err(|e| dla_core::Error::io(&dir, e))?;
    write_atomic(&tc.run_dir.join(&raw_rel), to_pretty_json(&full).as_bytes())?;

    // The discriminator.
    let mut result = DiscriminatorResult::new("plateau+purge", favors);
    result.supports.clone_from(&supports);
    result.finding.clone_from(&a.finding);
    if let Some(c) = picked
        .primary
        .as_deref()
        .or(picked.voting.first().map(String::as_str))
    {
        result.extra.insert("column".into(), json!(c));
    }
    result.extra.insert("series".into(), json!(series.id));
    result.extra.insert("trend".into(), json!(trend.id));
    result.extra.insert("harness".into(), json!(a.harness));
    result.extra.insert("purge_kind".into(), json!(purge_kind));
    result
        .extra
        .insert("iterations".into(), json!(manifest.iterations));
    result.extra.insert("extra".into(), json!(extra));
    let mut keep = results.len();
    let value = loop {
        let cols: Vec<Value> = results
            .iter()
            .take(keep)
            .map(ColumnResult::to_json)
            .collect();
        result.extra.insert("columns".into(), Value::Array(cols));
        if keep < results.len() {
            result
                .extra
                .insert("truncated".into(), json!(results.len() - keep));
        }
        let v = result.to_value();
        if serde_json::to_string(&v).map_or(usize::MAX, |s| s.len()) <= MAX_RESULT_BYTES
            || keep == 0
        {
            break v;
        }
        keep -= 1;
    };
    let mut spec = EvidenceSpec::new(
        EvidenceKind::Discriminator,
        "dla",
        env!("CARGO_PKG_VERSION"),
        value,
    );
    spec.authority = Authority::Test;
    spec.raw = Some(raw_rel);
    spec.source = Some(tc.artifact_source(&format!("series/{}.json", series.id))?);
    spec.coverage = EvidenceCoverage {
        status: if keep == results.len() {
            CoverageStatus::Complete
        } else {
            CoverageStatus::Partial
        },
        examined: Some(results.len() as u64),
        total: Some(series.columns.len() as u64),
    };
    spec.limitations = limitations.clone();
    spec.limitations
        .push("gate and plateau thresholds are provisional (not calibrated)".to_owned());
    let evidence = tc.add_evidence(&mut ledger, spec)?;

    let next = next_commands(
        &ledger,
        bound.as_ref(),
        &a.harness,
        favors,
        &supports,
        &evidence,
        extra,
    );
    let mut out = Map::new();
    out.insert("ok".into(), json!(true));
    out.insert("command".into(), json!("alloc-test"));
    out.insert("series".into(), json!(series.id));
    out.insert("trend".into(), json!(trend.id));
    out.insert("evidence".into(), json!(evidence));
    out.insert("favors".into(), json!(favors));
    out.insert("supports".into(), json!(supports));
    out.insert("finding".into(), json!(a.finding));
    out.insert("basis".into(), json!(picked.basis));
    out.insert("purge".into(), json!(purge_kind));
    let briefs: Vec<Value> = results.iter().map(ColumnResult::brief).collect();
    let mut shown = briefs.len().min(8);
    loop {
        out.insert("columns".into(), Value::Array(briefs[..shown].to_vec()));
        if shown < briefs.len() {
            out.insert("columns_more".into(), json!(briefs.len() - shown));
        }
        out.insert("next".into(), json!(next));
        if serde_json::to_string(&out).map_or(0, |s| s.len()) < MAX_RESULT_BYTES - 128 || shown == 0
        {
            break;
        }
        shown -= 1;
    }
    Ok(Value::Object(out))
}

/// The growth evidence a `confirm` of `root` should cite: the newest passing one tied to it.
fn growth_evidence(ledger: &Ledger, root: &Finding) -> Option<EvidenceId> {
    ledger
        .evidence()
        .filter(|e| {
            matches!(
                e.kind,
                EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries
            )
        })
        .filter_map(|e| {
            let g = GrowthSummary::from_result(&e.result)?;
            (g.passed_gate() && g.best_column(&root.id, root.site.as_deref()).is_some())
                .then(|| e.evidence_id.clone())
        })
        .last()
}

/// The newest `workload` evidence of harness `harness`.
fn workload_evidence(ledger: &Ledger, harness: &str) -> Option<EvidenceId> {
    ledger
        .evidence()
        .filter(|e| e.kind == EvidenceKind::Workload)
        .filter(|e| e.result.get("harness").and_then(Value::as_str) == Some(harness))
        .last()
        .map(|e| e.evidence_id.clone())
}

fn next_commands(
    ledger: &Ledger,
    bound: Option<&Finding>,
    harness: &str,
    favors: Favors,
    supports: &[String],
    evidence: &EvidenceId,
    extra: u32,
) -> Vec<String> {
    let mut next: Vec<String> = Vec::new();
    let Some(f) = bound else {
        match favors {
            Favors::Inconclusive => next.push(format!(
                "dla alloc-test {harness} --finding <F> --extra {}   (bind it to a finding for a verdict on that column)",
                extra * 2
            )),
            _ => next.push(format!(
                "dla alloc-test {harness} --finding <F>   (bind the result to a finding before refuting or confirming)"
            )),
        }
        return next;
    };
    let root = f.root_id();
    let root_f = ledger.finding(&root).unwrap_or(f);
    let siblings: Vec<&Finding> = ledger
        .findings()
        .filter(|s| s.parent.as_ref() == Some(&root) && s.polarity() == Polarity::NonLeak)
        .collect();
    match favors {
        Favors::NonLeak => {
            let matching: Vec<&Finding> = siblings
                .iter()
                .copied()
                .filter(|s| {
                    s.hypothesis
                        .as_deref()
                        .is_some_and(|h| supports.iter().any(|x| h.starts_with(x.as_str())))
                })
                .collect();
            if siblings.is_empty() {
                let h = supports.first().map_or("N-WARM", String::as_str);
                next.push(format!(
                    "dla ledger add --sibling-of {root} --origin hypothesis --hypothesis {h} --refute-if \"growth continues over the extra cycles\" --evidence {evidence}"
                ));
            }
            if !root_f.state.is_dispositioned() {
                next.push(format!("dla ledger refute {root} --evidence {evidence}"));
            }
            for s in matching.iter().filter(|s| !s.state.is_dispositioned()) {
                if s.state == FindingState::Candidate
                    && let Some(w) = workload_evidence(ledger, harness)
                {
                    next.push(format!("dla ledger test {} --evidence {w}", s.id));
                }
                next.push(format!("dla ledger confirm {} --evidence {evidence}", s.id));
            }
        }
        Favors::Leak => {
            for s in siblings.iter().filter(|s| !s.state.is_dispositioned()) {
                next.push(format!("dla ledger refute {} --evidence {evidence}", s.id));
            }
            if siblings.is_empty() {
                next.push(format!(
                    "dla ledger add --sibling-of {root} --origin hypothesis --hypothesis N-WARM --refute-if \"growth continues over the extra cycles\" --evidence {evidence}   (a leak needs a refuted non-leak sibling)"
                ));
            }
            if !root_f.state.is_dispositioned() {
                let growth = growth_evidence(ledger, root_f).map_or_else(
                    || "<growth evidence from dla trend>".to_owned(),
                    |e| e.to_string(),
                );
                if root_f.state == FindingState::Candidate
                    && let Some(w) = workload_evidence(ledger, harness)
                {
                    next.push(format!("dla ledger test {root} --evidence {w}"));
                }
                if root_f.holder.is_none() {
                    next.push(format!("dla ledger annotate {root} --holder <holder>"));
                }
                if root_f.site.is_none() {
                    next.push(format!("dla ledger annotate {root} --site <file:line>"));
                }
                next.push(format!("dla ledger confirm {root} --evidence {growth}"));
            }
        }
        Favors::Inconclusive => next.push(format!(
            "dla alloc-test {harness} --finding {} --extra {}",
            f.id,
            extra * 2
        )),
    }
    next.truncate(MAX_NEXT);
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::series::{
        Axis, Column, ColumnScope, Point, SeriesRegion, SeriesSource, SeriesSourceKind,
    };

    fn col(name: &str) -> Column {
        Column {
            name: name.to_owned(),
            unit: Unit::Bytes,
            kind: ColumnKind::Gauge,
            region: SeriesRegion::Process,
            baseline: false,
            after_gc: false,
            scope: ColumnScope::Process,
            site: None,
            finding: None,
            source: "test".to_owned(),
        }
    }

    /// Warm-up 2, 6 main and 6 extra points at x = 1000, 2000, ...; `rss` grows slowly on
    /// a 500 MB level, `probe` grows 30 KB per step with `probe_gaps` readings missing.
    fn series(probe_gaps: &[usize]) -> SeriesFile {
        let points = (0..14)
            .map(|i| Point {
                x: 1000.0 * (i as f64 + 1.0),
                t_ms: i as u64,
                values: vec![
                    Some(500e6 + 1000.0 * i as f64),
                    (!probe_gaps.contains(&i)).then_some(1e6 + 30_000.0 * i as f64),
                ],
            })
            .collect();
        SeriesFile {
            schema_version: 1,
            id: "S1".to_owned(),
            source: SeriesSource::new(SeriesSourceKind::AllocTest),
            axis: Axis::N,
            warmup: 2,
            columns: vec![col("os.rss"), col("probe:Leak")],
            points,
            events: vec![],
            limitations: vec![],
        }
    }

    #[test]
    fn only_process_columns_use_the_rss_reference() {
        let s = series(&[]);
        assert!(reference_level(&s, 0) > 4e8);
        assert_eq!(reference_level(&s, 1), 0.0);
    }

    #[test]
    fn a_precise_probe_leak_far_below_one_percent_of_rss_favors_leak() {
        let s = series(&[]);
        let mut lim = Vec::new();
        let r = test_column(&s, "probe:Leak", true, 6, None, &mut lim).unwrap();
        assert_eq!(
            r.plateau.verdict,
            PlateauVerdict::Continues,
            "{:?}",
            r.plateau
        );
        assert_eq!(r.favors, Favors::Leak);
        assert!(lim.is_empty(), "{lim:?}");
    }

    #[test]
    fn a_missing_main_reading_does_not_shift_the_windows() {
        // Point index 3 is in the main window (indexes 2..8): one main reading is missing.
        let s = series(&[3]);
        let mut lim = Vec::new();
        let r = test_column(&s, "probe:Leak", true, 6, None, &mut lim).unwrap();
        assert_eq!(r.plateau.verdict, PlateauVerdict::Continues);
        assert!(
            lim.iter().any(|l| l.contains("5 of 6 main-window")),
            "{lim:?}"
        );
    }

    #[test]
    fn a_column_that_exists_only_in_the_extra_part_is_not_called_main() {
        let mut s = series(&[]);
        for p in s.points.iter_mut().take(8) {
            p.values[1] = None;
        }
        let mut lim = Vec::new();
        let r = test_column(&s, "probe:Leak", true, 6, None, &mut lim).unwrap();
        // Six extra points exist and are judged as the extra window (no main slope).
        assert_ne!(r.plateau.verdict, PlateauVerdict::NotRun);
        assert!(
            lim.iter().any(|l| l.contains("0 of 6 main-window")),
            "{lim:?}"
        );
        // No extra points at all.
        let mut s = series(&[]);
        for p in s.points.iter_mut().skip(8) {
            p.values[1] = None;
        }
        let mut lim = Vec::new();
        let r = test_column(&s, "probe:Leak", true, 6, None, &mut lim).unwrap();
        assert_eq!(r.plateau.verdict, PlateauVerdict::NotRun);
        assert!(lim.iter().any(|l| l.contains("no extra-window")), "{lim:?}");
    }

    #[test]
    fn join_names_the_columns_it_drops() {
        let prev = series(&[]);
        let mut extra = series(&[]);
        extra.id = "S2".to_owned();
        extra
            .columns
            .extend((0..70).map(|i| col(&format!("extra{i}"))));
        for p in &mut extra.points {
            p.values.resize(extra.columns.len(), Some(1.0));
        }
        let j = join_series(&prev, &extra);
        assert_eq!(j.columns.len(), dla_core::series::MAX_COLUMNS);
        assert!(
            j.limitations
                .iter()
                .any(|l| l.starts_with("8 columns beyond")),
            "{:?}",
            j.limitations
        );
    }
}
