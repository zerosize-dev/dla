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
    series::{ColumnKind, ColumnScope, KeyMode, SeriesFile, SeriesSourceKind, Unit},
    toolctx::{EvidenceSpec, ToolCtx},
    trend::{ColumnTrend, TREND_SCHEMA_VERSION, TrendReport},
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
const MIN_EXTRA: u32 = dla_workload::harness::pace::MIN_EXTRA;
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
#[cfg_attr(test, derive(Debug))]
struct ColumnResult {
    name: String,
    voting: bool,
    plateau: Plateau,
    purge: Option<Purge>,
    gc: Option<Gc>,
    favors: Favors,
    supports: Vec<String>,
    /// What a purge that kept the growth left of a plateau, in words (the rises of the main
    /// window against the extra window), when it had to be weighed.
    note: Option<String>,
    /// Points of the extra window that would let a held-back plateau stand.
    points_needed: Option<u32>,
    /// A deciding column that did not grow in the main window of this series (see
    /// [`judge::grew`]), whether or not that is held against the verdict.
    flat: bool,
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
        if let Some(n) = &self.note {
            m.insert("note".into(), json!(n));
        }
        if let Some(n) = self.points_needed {
            m.insert("points_needed".into(), json!(n));
        }
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

/// The plateau verdict of another column of the series over the same windows, or `None` when
/// the series has no such column.
fn column_plateau(series: &SeriesFile, name: &str, iterations: usize) -> Option<PlateauVerdict> {
    let idx = series.column_index(name)?;
    let counter = series.columns[idx].kind == ColumnKind::Counter;
    let points = series.post_warmup(idx);
    let (main, extra) = judge::split(&points, main_end_x(series, iterations));
    let min_rise = if series.columns[idx].unit == Unit::Bytes {
        judge::MIN_BYTE_RISE
    } else {
        0.0
    };
    let reference = reference_level(series, idx);
    Some(judge::plateau(main, extra, counter, reference, min_rise).verdict)
}

/// Why a leak reading of a column is not a leak, when the column is file-backed resident
/// memory or resident memory that file-backed memory explains.
///
/// A leak is memory the program allocated and keeps, which is anonymous memory. Resident
/// file-backed pages (`os.rss_file`: mapped libraries, modules and data files touched for the
/// first time or kept warm) are the page cache of files, not an allocation, so:
/// - `os.rss_file` never decides `leak` by itself. Its own reading is at most inconclusive; a
///   leak another column shows still stands, with the file-backed column agreeing in the
///   result.
/// - `os.rss` growth that the file-backed part explains is not a leak either: when the series
///   samples both `os.rss_anon` and `os.rss_file`, the anonymous part plateaued over the extra
///   window and the file-backed part did not, the growth of `os.rss` is the file-backed growth
///   and it cannot favor leak. Where `os.rss_anon` is not sampled (it comes from the Linux
///   process status; the other platforms sample neither part) there is nothing to tell the
///   parts apart with, and `os.rss` keeps its own reading: growth that cannot be shown to be
///   file-backed is not excused.
fn file_backed_note(series: &SeriesFile, name: &str, iterations: usize) -> Option<String> {
    match name {
        "os.rss_file" => Some(
            "resident file-backed pages are the page cache of mapped files, not memory the program allocated and keeps: this column never decides a leak by itself"
                .to_owned(),
        ),
        "os.rss" => {
            let anon = column_plateau(series, "os.rss_anon", iterations)?;
            let file = column_plateau(series, "os.rss_file", iterations)?;
            let flat = matches!(anon, PlateauVerdict::Plateau | PlateauVerdict::NeverGrew);
            let grew = matches!(file, PlateauVerdict::Continues | PlateauVerdict::Inconclusive);
            (flat && grew).then(|| {
                "os.rss grew over the extra window while os.rss_anon plateaued and os.rss_file did not: the growth is file-backed pages (mapped files), not memory the program holds"
                    .to_owned()
            })
        }
        _ => None,
    }
}

fn test_column(
    series: &SeriesFile,
    name: &str,
    voting: bool,
    bound: bool,
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
    let min_rise = if series.columns[idx].unit == Unit::Bytes {
        judge::MIN_BYTE_RISE
    } else {
        0.0
    };
    let mut plateau = judge::plateau(main, extra, counter, reference, min_rise);
    // A deciding column that did not grow in the main window of this series did not stop
    // growing: for a finding the series is judged on (`bound`, see `Exemption` for the two
    // cases it is not), it does not reproduce the growth, whatever its extra window says.
    let flat = voting
        && matches!(
            plateau.verdict,
            PlateauVerdict::Plateau | PlateauVerdict::Inconclusive
        )
        && !judge::grew(main, counter, reference, min_rise);
    if bound && flat {
        plateau.verdict = PlateauVerdict::NeverGrew;
    }
    let purge = event.map(|e| {
        if counter {
            return judge::purge(&e.kind, None, None, None, min_rise);
        }
        let read = |m: &Map<String, Value>| m.get(name).and_then(Value::as_f64);
        judge::purge(
            &e.kind,
            read(&e.before),
            read(&e.after),
            warm_end(series, idx),
            min_rise,
        )
    });
    // A column of a region no purge gives back is judged on its plateau alone, whatever the
    // purge kind and its reading (kept in the result, marked as not applying).
    let purge = purge.map(|p| {
        if pick::purge_does_not_apply(name) {
            p.not_applicable()
        } else {
            p
        }
    });
    let gc = (!counter)
        .then(|| series.column_index(&format!("{name}.pre_gc")))
        .flatten()
        .and_then(|pre| judge::gc(&series.post_warmup(pre), &points, reference));
    // A purge that kept the growth says the growth is not headroom, not that it goes on: the
    // plateau stands when the main window's rises, scaled to the extra window, say another
    // would have shown (gauges only; a counter keeps its own rule).
    let kept = plateau.verdict == PlateauVerdict::Plateau
        && purge
            .as_ref()
            .is_some_and(|p| p.applies && p.verdict == PurgeVerdict::Retained);
    let rises = (kept && !counter)
        .then(|| judge::rises(main, extra, warm_end(series, idx), plateau.noise?))
        .flatten();
    let (mut favors, mut supports) = judge::combine(
        plateau.verdict,
        purge.as_ref(),
        gc,
        rises.map(|r| r.expected),
    );
    // File-backed resident memory is never what decides a leak (see `file_backed_note`).
    let mut file_note = None;
    if favors == Favors::Leak {
        file_note = file_backed_note(series, name, iterations);
        if file_note.is_some() {
            favors = Favors::Inconclusive;
            supports.clear();
        }
    }
    let stair_need = (plateau.verdict == PlateauVerdict::Inconclusive)
        .then_some(plateau.points_needed)
        .flatten();
    let note = file_note.or(stair_need.map(|n| {
        format!(
            "the column grows in steps and no step has come in the extra window for as long as the next gap is expected to be (steps of a growing container come ever further apart), so a plateau cannot be told from a step that has not come yet: an extra window of {n} points would run past it"
        )
    }));
    let note = note.or(rises.map(|r| {
        let main_part = format!(
            "the purge kept the growth (held memory, not allocator or collector headroom); the main window rose {} time(s), so {:.1} rise(s) were expected in the extra window",
            r.count, r.expected
        );
        if favors == Favors::NonLeak {
            format!(
                "{main_part}, none showed (at least {} are needed): the plateau stands",
                judge::MIN_EXPECTED_RISES
            )
        } else {
            let need = r.points_needed.map_or_else(
                || "the main window shows no rise to scale from".to_owned(),
                |n| format!("an extra window of {n} points would expect {}", judge::MIN_EXPECTED_RISES),
            );
            format!("{main_part}; too few to call the plateau ({need})")
        }
    }));
    Some(ColumnResult {
        name: name.to_owned(),
        voting,
        plateau,
        purge,
        gc,
        favors,
        supports,
        note,
        points_needed: rises.and_then(|r| r.points_needed).or(stair_need),
        flat,
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
            if let Some(reason) = dla_core::series::not_a_growth_signal(&col.name) {
                gate = GateVerdict::Fail;
                gate_failed.push(reason.to_owned());
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
            for f in ledger.findings() {
                if f.polarity() == Polarity::Leak
                    && f.owner
                        .as_deref()
                        .is_some_and(|o| dla_core::trend::probe_column_matches_owner(&col.name, o))
                {
                    push(&f.id);
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
        targets: Vec::new(),
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
        // An empty file is an id reserved by a writer that has not finished.
        .filter(|e| dla_core::fsutil::is_written(&e.path()))
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

/// The verdict over the tested columns: the bound column's, or the voting columns together;
/// a non-leak verdict stands only when every required column favours non-leak too (a required
/// thread count that rises counts as not favouring it). Returns the verdict, its supports and the
/// required columns that held a non-leak verdict back.
fn decide(results: &[ColumnResult], picked: &pick::Picked) -> (Favors, Vec<String>, Vec<String>) {
    let deciding: Vec<(Favors, Vec<String>)> = results
        .iter()
        .filter(|r| match &picked.primary {
            Some(p) => &r.name == p,
            None => r.voting,
        })
        .map(|r| (r.favors, r.supports.clone()))
        .collect();
    let required: Vec<(&str, Favors)> = results
        .iter()
        .filter(|r| picked.required.contains(&r.name))
        .map(|r| {
            // A thread count that rises over the extra window (its net rise over the jitter)
            // is no plateau, whatever the plateau test reads: it holds a non-leak verdict back.
            let rises = is_thread_count(&r.name) && column_rises(results, picked, &r.name);
            let favors = if rises && r.favors == Favors::NonLeak {
                Favors::Inconclusive
            } else {
                r.favors
            };
            (r.name.as_str(), favors)
        })
        .collect();
    let overall = judge::overall(&deciding);
    let was_non_leak = overall.0 == Favors::NonLeak;
    let favours: Vec<Favors> = required.iter().map(|r| r.1).collect();
    let (favors, supports) = judge::require_non_leak(overall, &favours);
    let held_back = if was_non_leak && favors != Favors::NonLeak {
        required
            .iter()
            .filter(|r| r.1 != Favors::NonLeak)
            .map(|r| r.0.to_owned())
            .collect()
    } else {
        Vec::new()
    };
    (favors, supports, held_back)
}

/// The least extra rise under which a `jvm.nonheap_used` leak verdict on a flat class count may
/// still be the agent's own checkpoint work.
const AGENT_DRIFT_RISE: f64 = 1024.0 * 1024.0;

/// The most the agent's own checkpoint work was measured to add per checkpoint on an idle driver
/// (measured 10 to 20 KB; the highest is used).
const AGENT_DRIFT_PER_CHECKPOINT: f64 = 20.0 * 1024.0;

/// The most a class count that reads inconclusive may have risen over the extra window and still
/// be called flat (jitter of a few classes, not a staircase). The JVM loads a few classes at
/// moments tied to elapsed time, not to the workload. Measured on idle drivers (drivers that
/// define no class), also under load: 24 fast runs rose 0 or 2 classes; of 15 runs whose unit only
/// sleeps (4 to 25 ms per unit, so the run is slow) 13 rose 4 to 6, and the largest rise that
/// read inconclusive was 5 and the largest that read plateau 6. The constant is the largest rise
/// seen (6) plus a margin of 2. A class leak that adds this many classes or fewer over the extra
/// window cannot be told from the jitter by the count alone: while the count does not read
/// continues it reads as flat, and the result is then inconclusive, never a leak. A count that
/// reads continues is a rising column whatever its net rise.
const AGENT_DRIFT_CLASS_JITTER: f64 = 8.0;

/// The most `jvm.threads` may have risen over the extra window with no leak (the JVM starts and
/// stops compiler threads by itself). Measured on drivers that start no thread (idle, hot loops,
/// a growing heap, slow idle; 59 runs, many under load, the JVM told 4 to 64 processors): the net
/// rise of `jvm.threads` was 0 every time; the constant is that rise plus 1, and at least 2. The
/// slow thread leaks measured rose 6 and 10 threads. `os.threads` is not judged by its net
/// rise: HotSpot's own GC worker threads came and went there (a driver that starts no thread,
/// 32 processors told to the JVM, rose 4 on `os.threads` and 0 on `jvm.threads`), so it counts as
/// rising only when its plateau reads continues. `jvm.threads` has no checkpoint jitter, so a net
/// rise over this is growth whatever its plateau reads (a count that rises in bursts near the
/// end of the window reads plateau).
const AGENT_DRIFT_THREAD_JITTER: f64 = 2.0;

/// The margin on the measured per-checkpoint drift.
const AGENT_DRIFT_MARGIN: f64 = 2.0;

/// The extra rise under which a leak reading may be the agent's drift: the larger of 1 MiB and
/// the measured drift of every extra checkpoint with the margin (the drift scales with the window).
fn agent_drift_line(extra: u32) -> f64 {
    AGENT_DRIFT_RISE.max(AGENT_DRIFT_PER_CHECKPOINT * f64::from(extra) * AGENT_DRIFT_MARGIN)
}

/// What the alloc-test knows of the harness that decides how its non-heap reading is read.
#[derive(Clone, Copy)]
struct DriftCtx {
    /// The extra checkpoints of the window.
    extra: u32,
    /// A command-mode harness: a fresh JVM per point, so no checkpoint drift builds up.
    command: bool,
    /// A JUnit repeat harness (its `next` keeps `--junit`).
    junit: bool,
    /// The run was cut short by the deadline: its verdict is capped at inconclusive.
    partial: bool,
}

/// How a small non-heap leak reading on a flat class count is read.
enum Drift {
    /// The agent's own checkpoint work: inconclusive, with this caution.
    Agent(String),
    /// The class count rises over the extra window (a net rise over the jitter): not the agent's
    /// drift, so the verdict stands; this names the column.
    Classes(String),
    /// A required column rises. The reading is not the agent's drift: `limitation` names the
    /// column, `next` the capture that decides it, and `inconclusive` says whether the verdict
    /// is dropped (a thread count is no agent drift, so a thread leak keeps its verdict).
    Growing {
        limitation: String,
        next: String,
        inconclusive: bool,
    },
}

/// The capture that decides a rising column: the thread capture when a thread count rises (the
/// thread category of NMT rises with it), the NMT category for the other `jvm.nmt.*` columns.
fn deciding_capture(columns: &[&str], harness: &str) -> String {
    let threads = columns
        .iter()
        .any(|c| matches!(*c, "jvm.threads" | "os.threads"));
    let nmt: Vec<&str> = columns
        .iter()
        .copied()
        .filter(|c| c.starts_with("jvm.nmt.") && !(threads && *c == "jvm.nmt.thread"))
        .collect();
    let nmt_text = format!(
        "read nmt-diff.txt for {} with dla summarize non-heap: the category that rises says what grows",
        nmt.join(", ")
    );
    match (threads, nmt.is_empty()) {
        (true, true) => format!(
            "dla harness run {harness} --capture nonheap   (the thread count keeps rising: read threads.json with dla summarize non-heap for the group that grows; a longer --extra cannot decide a thread leak)"
        ),
        (true, false) => format!(
            "dla harness run {harness} --capture nonheap   (read threads.json with dla summarize non-heap for the thread group that grows; {nmt_text}; a longer --extra does not decide either)"
        ),
        (false, _) => format!(
            "dla harness run {harness} --capture nonheap   ({nmt_text}; a longer --extra does not decide it)"
        ),
    }
}

/// Whether a required column rises, by its kind. `jvm.threads`: its plateau reads continues or its
/// net rise over the extra window is over the jitter, whatever its plateau reads (a counter has
/// no checkpoint jitter, so a few more threads are growth even when the plateau test cannot judge
/// them or reads a plateau because the threads came in bursts). `os.threads`: only when its
/// plateau reads continues (the JVM's own GC worker threads move it by a few). A thread count that
/// is not a required column is no growth. A `jvm.nmt.*` byte category: only when its plateau reads
/// continues (bytes jitter); the thread category of NMT also rises when a thread count that is
/// itself rising does, as it rises with the threads.
fn column_rises(results: &[ColumnResult], picked: &pick::Picked, name: &str) -> bool {
    let row = |n: &str| results.iter().find(|r| r.name == n);
    let continues =
        |n: &str| row(n).is_some_and(|r| r.plateau.verdict == PlateauVerdict::Continues);
    let thread_rises = |n: &str| {
        picked.required.iter().any(|c| c == n)
            && row(n).is_some_and(|r| {
                r.plateau.verdict == PlateauVerdict::Continues
                    || (n == "jvm.threads"
                        && r.plateau
                            .extra_rise
                            .is_some_and(|x| x > AGENT_DRIFT_THREAD_JITTER))
            })
    };
    match name {
        "jvm.threads" | "os.threads" => thread_rises(name),
        "jvm.nmt.thread" => {
            continues(name) || thread_rises("jvm.threads") || thread_rises("os.threads")
        }
        _ => continues(name),
    }
}

/// Splits column names into those that keep rising and those that only could not be judged yet.
fn split_rising(
    results: &[ColumnResult],
    picked: &pick::Picked,
    names: &[String],
) -> (Vec<String>, Vec<String>) {
    names
        .iter()
        .cloned()
        .partition(|n| column_rises(results, picked, n))
}

fn is_thread_count(name: &str) -> bool {
    matches!(name, "jvm.threads" | "os.threads")
}

/// The required columns that may be rising: NMT categories that do not favour a non-leak
/// reading, and the thread counts whatever they favour (their rise is judged by `column_rises`).
fn rising_required<'a>(results: &'a [ColumnResult], picked: &pick::Picked) -> Vec<&'a str> {
    results
        .iter()
        .filter(|r| {
            picked.required.contains(&r.name)
                && (matches!(r.name.as_str(), "jvm.threads" | "os.threads")
                    || r.favors != Favors::NonLeak)
        })
        .map(|r| r.name.as_str())
        .collect()
}

/// The required columns of an inconclusive result that are not at a plateau: those that do not
/// favour non-leak, and a thread count only when it rises (a thread count at a plateau is no open
/// column, whatever `rising_required` lists for the drift reading).
fn open_required(results: &[ColumnResult], picked: &pick::Picked) -> Vec<String> {
    rising_required(results, picked)
        .into_iter()
        .filter(|n| {
            let favours_non_leak = results
                .iter()
                .any(|r| r.name == *n && r.favors == Favors::NonLeak);
            !(is_thread_count(n) && favours_non_leak && !column_rises(results, picked, n))
        })
        .map(str::to_owned)
        .collect()
}

/// What the results say of the class count.
enum ClassCount {
    /// At a plateau, or too jittery to judge with a net rise of the jitter at most.
    Flat,
    /// The results carry no `jvm.classes_loaded` column, or its plateau test did not run.
    Unknown,
    /// Not flat: its plateau reads continues, or reads inconclusive with a net rise over the
    /// jitter. A class count with a steady net rise (a staircase) is a class leak, however slow.
    /// The text says what it did, with true numbers.
    Rising(String),
}

fn class_count(results: &[ColumnResult]) -> ClassCount {
    let Some(r) = results.iter().find(|r| r.name == "jvm.classes_loaded") else {
        return ClassCount::Unknown;
    };
    match (r.plateau.verdict, r.plateau.extra_rise) {
        (PlateauVerdict::Plateau, _) => ClassCount::Flat,
        (PlateauVerdict::Continues, Some(n)) => ClassCount::Rising(format!(
            "keeps rising ({n:.0} classes over the extra window)"
        )),
        (PlateauVerdict::Continues, None) => ClassCount::Rising("keeps rising".to_owned()),
        (PlateauVerdict::Inconclusive, Some(n)) if n > AGENT_DRIFT_CLASS_JITTER => {
            ClassCount::Rising(format!(
                "rose {n:.0} classes over the extra window, over the {AGENT_DRIFT_CLASS_JITTER:.0} of an idle driver's jitter"
            ))
        }
        (PlateauVerdict::Inconclusive, Some(_)) => ClassCount::Flat,
        (PlateauVerdict::Inconclusive, None)
        | (PlateauVerdict::NotRun | PlateauVerdict::NeverGrew, _) => ClassCount::Unknown,
    }
}

/// The reading of a leak verdict taken from the JVM's non-heap bytes alone. With a rise under the
/// drift line over the extra window, the checkpoints' own work (compiled code and metadata of the
/// agent) can make the figure rise, but only when nothing else rises. First every rising required
/// column (a `jvm.nmt.*` category or a thread count, by its kind) is collected, and whether the
/// class count is flat. A rising required column: `Growing`, naming all of them and the class
/// count too when it is not flat. Otherwise a class count that is not flat: `Classes`. Otherwise
/// the agent's drift. A command-mode harness (a fresh JVM per point) has no checkpoint drift.
fn agent_drift(
    results: &[ColumnResult],
    picked: &pick::Picked,
    favors: Favors,
    harness: &str,
    ctx: DriftCtx,
) -> Option<Drift> {
    if ctx.command
        || favors != Favors::Leak
        || picked.primary.as_deref() != Some("jvm.nonheap_used")
    {
        return None;
    }
    let rise = results
        .iter()
        .find(|r| r.name == "jvm.nonheap_used")?
        .plateau
        .extra_rise?;
    let line = agent_drift_line(ctx.extra);
    if rise >= line {
        return None;
    }
    let classes = class_count(results);
    let rising: Vec<&str> = rising_required(results, picked)
        .into_iter()
        .filter(|n| column_rises(results, picked, n))
        .collect();
    let stands = if ctx.partial {
        "a partial run is inconclusive whatever the columns show"
    } else {
        "the verdict stands"
    };
    if !rising.is_empty() {
        let threads = rising
            .iter()
            .any(|c| matches!(*c, "jvm.threads" | "os.threads"));
        // The thread category of NMT rises with the thread count; any other category is not
        // thread work, so it leaves the verdict open.
        let others = rising.iter().any(|c| {
            !matches!(*c, "jvm.threads" | "os.threads") && !(threads && *c == "jvm.nmt.thread")
        });
        // A class count that is not flat is a class leak whatever the categories read (a class
        // leak always raises Metaspace): the verdict stands and the capture reads the category.
        let class_leak = matches!(classes, ClassCount::Rising(_));
        // `os.threads` that rises alone (no `jvm.threads`, no NMT thread category of its own) may
        // be threads the JVM does not count as its own or its GC workers: the capture decides
        // it, the result stays inconclusive.
        let os_only = rising.contains(&"os.threads")
            && !rising.contains(&"jvm.threads")
            && !results.iter().any(|r| {
                r.name == "jvm.nmt.thread" && r.plateau.verdict == PlateauVerdict::Continues
            });
        let named: Vec<&str> = rising.clone();
        let class_part = if let ClassCount::Rising(text) = &classes {
            format!(
                "; jvm.classes_loaded is not flat either: it {text}, so the classes that load say what grows there"
            )
        } else {
            String::new()
        };
        return Some(Drift::Growing {
            limitation: format!(
                "{} {} rising over the extra window (jvm.nonheap_used itself rose {:.0} KB): that is not the agent's own drift, so it is not blamed on it; {}{class_part}",
                named.join(", "),
                if named.len() == 1 { "keeps" } else { "keep" },
                rise / 1024.0,
                if os_only && !class_leak {
                    "os.threads rises without jvm.threads, which the JVM's own GC worker threads can do: the result is inconclusive until the thread capture is read".to_owned()
                } else if !others {
                    format!(
                        "a thread count is no checkpoint work; {stands}, and the thread capture decides what holds the threads"
                    )
                } else if class_leak {
                    format!(
                        "the class count is not flat; {stands}, and the capture reads the NMT category that rises"
                    )
                } else {
                    "the result is inconclusive until the NMT category that rises is read"
                        .to_owned()
                }
            ),
            next: deciding_capture(&rising, harness),
            inconclusive: (others || os_only) && !class_leak,
        });
    }
    if let ClassCount::Rising(text) = &classes {
        return Some(Drift::Classes(format!(
            "jvm.classes_loaded {text} while jvm.nonheap_used rose {:.0} KB: that is no flat class count, so the rise is not blamed on the agent's own drift ({stands}); the classes that load say what grows",
            rise / 1024.0
        )));
    }
    Some(Drift::Agent(format!(
        "jvm.nonheap_used rose {:.0} KB over the extra window{}: that is under {:.0} KB, the line for what the agent's own checkpoints add by themselves ({:.0} KB per checkpoint measured on an idle driver, with a {:.0}x margin, over {} checkpoints; at least 1 MiB), and no jvm.nmt.* category or thread count rises, so a small JVM cannot decide this and the result is inconclusive, not a leak. Only a rise over that line, a growing jvm.nmt.* category (Symbol or Metaspace) or a harness that does more work per checkpoint ({}) can decide it.",
        rise / 1024.0,
        match classes {
            ClassCount::Unknown =>
                " (caution: jvm.classes_loaded was not recorded, or its plateau test could not judge it, in this result, so a flat class count could not be checked and a class leak is not excluded)",
            _ => " with a flat class count",
        },
        line / 1024.0,
        AGENT_DRIFT_PER_CHECKPOINT / 1024.0,
        AGENT_DRIFT_MARGIN,
        ctx.extra,
        if ctx.junit {
            "a larger --step at harness gen --junit"
        } else {
            "a larger --step at harness gen"
        }
    )))
}

/// Whether the growing-column reading also prints the ledger commands and the longer `--extra`
/// of an ordinary result: only for a leak verdict that stays (no NMT category open) and not on a
/// partial run, whose verdict is capped at inconclusive (a thread capture decides it, a longer
/// `--extra` cannot, and no refute or confirm follows from it).
fn keeps_ledger_commands(inconclusive: bool, partial: bool) -> bool {
    !inconclusive && !partial
}

/// The `next` of a result read as the agent's drift: the harness is rebuilt with more work per
/// checkpoint, in the mode it ran in.
fn drift_next(ctx: DriftCtx) -> String {
    let gen_cmd = if ctx.junit {
        "dla harness gen --lang jvm --junit --step <larger>"
    } else {
        "dla harness gen --lang jvm --driver --step <larger>"
    };
    format!(
        "{gen_cmd}   (a rise under the drift line on a flat class count is the agent's own drift: do not refute or confirm from this result; read the jvm.nmt.* columns, or give each checkpoint more work)"
    )
}

/// A run the deadline stopped short is inconclusive: its verdict and supports are dropped.
fn cap_for_partial(
    partial: &[String],
    favors: Favors,
    supports: Vec<String>,
) -> (Favors, Vec<String>) {
    if partial.is_empty() {
        (favors, supports)
    } else {
        (Favors::Inconclusive, Vec::new())
    }
}

/// A bound finding whose size probe was never read: a non-leak verdict (from columns that
/// cannot see a step-growing container) is dropped; a leak verdict stands.
fn cap_for_unread_probe(favors: Favors, supports: Vec<String>) -> (Favors, Vec<String>) {
    if favors == Favors::NonLeak {
        (Favors::Inconclusive, Vec::new())
    } else {
        (favors, supports)
    }
}

pub(crate) fn key_mode(m: &HarnessManifest) -> KeyMode {
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

/// The column whose reading decided the verdict: the bound finding's column, else the first
/// voting column. A size probe (`probe:<path>`) decides by its own rise over the extra window,
/// never by the whole-process byte columns, which a container that grows in steps leaves flat
/// between two steps.
fn decided_by(picked: &pick::Picked) -> Option<String> {
    picked
        .primary
        .clone()
        .or_else(|| picked.voting.first().cloned())
}

/// When `finding` has a Go field probe in the harness and the series has no column for it, the
/// limitation that says so (and why), else `None`.
fn unread_field_probe(
    manifest: &HarnessManifest,
    series: &SeriesFile,
    finding: &Finding,
) -> Option<String> {
    use dla_core::harness::ProbeTarget;
    let root = finding.root_id();
    manifest.probes.iter().find_map(|p| {
        let ProbeTarget::GoField {
            package_dir,
            type_name,
            field,
        } = &p.target
        else {
            return None;
        };
        if p.finding.as_ref() != Some(&root) && p.finding.as_ref() != Some(&finding.id) {
            return None;
        }
        let path = format!("{package_dir}.{type_name}.{field}");
        if series.column_index(&format!("probe:{path}")).is_some() {
            return None;
        }
        let why = series
            .limitations
            .iter()
            .find_map(|l| l.strip_prefix(&format!("probe {path}: ")))
            .unwrap_or("it was never read");
        Some(format!(
            "size probe {} ({path}) of {root} has no readings ({why}), so the whole-process columns decide in its place, and a container that grows in steps reads flat between two steps there: the result cannot favour non-leak; call dlaTrack(<your {type_name}>) in setup of the driver and run again",
            p.id
        ))
    })
}

/// A test-loop harness asked about a finding whose holder is a `jvm-static` probe: the holder to
/// attribute to the test classes (`DLA_ATTRIBUTE`). `None` for any other harness or finding.
fn attribution_target(
    manifest: &HarnessManifest,
    finding: Option<&Finding>,
) -> Option<dla_workload::harness::run::Attribution> {
    use dla_core::harness::ProbeTarget;
    let finding = finding?;
    if !manifest.is_test_loop() {
        return None;
    }
    let root = finding.root_id();
    manifest.probes.iter().find_map(|p| {
        let ProbeTarget::JvmStatic { class, field } = &p.target else {
            return None;
        };
        (p.finding.as_ref() == Some(&root) || p.finding.as_ref() == Some(&finding.id)).then(|| {
            dla_workload::harness::run::Attribution {
                finding: root.to_string(),
                class: class.clone(),
                field: field.clone(),
            }
        })
    })
}

/// Test classes named in the printed result: the full names are in the stored file.
const MAX_FED_NAMED: usize = 4;
const MAX_FLAT_NAMED: usize = 8;

/// `fed_by` (the classes whose later-runs sum is positive, with the entries each added per run,
/// largest first) and `ran_flat` (the classes that ran and added nothing in the later runs), by
/// simple name, so that the result stays inside its size bound. The stored file has the full names.
fn attribution_summary(a: &dla_core::testloop::Attributed) -> Map<String, Value> {
    let simple = |c: &str| c.rsplit(['.', '$']).next().unwrap_or(c).to_owned();
    let fed = a.fed_by();
    let flat = a.ran_flat();
    let mut m = Map::new();
    m.insert("holder".into(), json!(a.probe));
    m.insert("later_runs".into(), json!(a.later_runs));
    m.insert(
        "fed_by".into(),
        Value::Array(
            fed.iter()
                .take(MAX_FED_NAMED)
                .map(|f| {
                    json!({"class": simple(&f.class), "per_run": (f.per_run * 10.0).round() / 10.0})
                })
                .collect(),
        ),
    );
    if fed.len() > MAX_FED_NAMED {
        m.insert("fed_by_more".into(), json!(fed.len() - MAX_FED_NAMED));
    }
    m.insert(
        "ran_flat".into(),
        json!(
            flat.iter()
                .take(MAX_FLAT_NAMED)
                .map(|c| simple(&c.class))
                .collect::<Vec<_>>()
        ),
    );
    if flat.len() > MAX_FLAT_NAMED {
        m.insert("ran_flat_more".into(), json!(flat.len() - MAX_FLAT_NAMED));
    }
    let stepped = a.stepped();
    if !stepped.is_empty() {
        // A late one-time fill, not a feed.
        m.insert(
            "added_once".into(),
            json!(
                stepped
                    .iter()
                    .take(MAX_FED_NAMED)
                    .map(|c| simple(&c.class))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if a.parallel {
        m.insert(
            "parallel".into(),
            json!("the figures of the classes overlap"),
        );
    }
    m
}

/// The refusal for a harness that is not about the finding `root` (see
/// [`dla_core::grown::harness_about`]): the fix names the harnesses that are about it, a hook
/// driver first, and always the driver of its own.
pub(crate) fn harness_not_about(
    view: &dla_core::RunView,
    harness: &str,
    runtime: Runtime,
    finding: &Finding,
    what: &str,
) -> Failure {
    let root = finding.root_id();
    let root_f = view.ledger.finding(&root).unwrap_or(finding);
    let rt = dla_core::gate::finding_runtime(view, root_f)
        .unwrap_or_else(|| runtime.as_str().to_owned());
    let root = &root;
    let mut about = dla_core::grown::harnesses_about(view, root);
    about.retain(|(m, _)| m.id != harness);
    about.sort_by_key(|(m, _)| !m.is_hook_driver());
    let own = format!(
        "dla harness gen --lang {rt} --driver --probe {root}, fill the unit so that it calls the function of the line, dla harness run <H'>, dla trend <S>, then dla alloc-test <H'> --finding {root}"
    );
    let named: Vec<String> = about
        .iter()
        .map(|(m, _)| {
            if m.is_test_loop() {
                format!(
                    "{} (a test loop: each run re-runs the project's tests, minutes): dla alloc-test {} --finding {root}",
                    m.id, m.id
                )
            } else {
                format!("dla alloc-test {} --finding {root}", m.id)
            }
        })
        .collect();
    let fix = if named.is_empty() {
        own
    } else {
        format!(
            "a harness about {root} exists: {}; or one of its own: {own}",
            named.join("; ")
        )
    };
    Failure::refuse(
        codes::HARNESS_NOT_ABOUT,
        format!(
            "{harness} does not target {root} and nothing it measured is tied to it, so {what} says nothing about {root}"
        ),
        format!("a harness that targets {root} or measured something tied to it"),
        fix,
    )
}

/// How the run reads the line whose allocation-site count column is missing from the series.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LineRan {
    /// The line's own execution count rose over the measured points.
    Line,
    /// The function of the line ran and the count of the line did not rise: the unit takes
    /// another branch.
    FunctionOnly,
    /// The series has no count of the line, and the function of it ran.
    NoCountFunctionRan,
    /// The series has no count of the line, and nothing says that the function ran.
    NoCount,
    /// The line did not run and neither did its function (or coverage cannot tell).
    NotReached,
    /// The line ran and kept nothing, but a retention record of the run shows a holder of the
    /// project keeping it: this unit does not answer that record. Carries the harness of the
    /// capture when its path says.
    HolderRecord(Option<String>),
}

/// What the run says of a finding whose allocation-site count column (`gone`) grew in a trend of
/// the run and is not in the new series: the verdict, the basis, the limitation that carries the
/// reading and how far the unit got. The agent writes only the sites that grew, so the absence
/// means that none of the blocks of the line stayed, or that the line did not run. The agent's
/// count of the executions of the line itself (`harness/<H>/lines-<S>.json`) tells which: a count
/// that rose makes it a measurement, non-leak. The function of the line having run is not enough:
/// a line on a branch the unit does not take allocates nothing, so nothing stays. A function that
/// ran with a line count that did not rise, and a series with no line count at all (an old agent,
/// another runtime, a point that lost its sections), leave it inconclusive.
#[allow(clippy::too_many_arguments)]
fn absent_site_reading(
    run_dir: &Path,
    project: &Path,
    ledger: &Ledger,
    manifest: &HarnessManifest,
    series: &SeriesFile,
    trend: &TrendReport,
    root: &FindingId,
    gone: &pick::AbsentSite,
) -> (Favors, String, String, LineRan) {
    use dla_core::trend::Verdict;
    let executed = HarnessManifest::dir_in(run_dir, &manifest.id)
        .join(dla_workload::harness::cover::EXECUTED_FILE);
    let inp = crate::trend_cmd::verdicts::Inputs {
        project,
        run_dir,
        ledger,
        series,
        columns: &trend.columns,
        too_short: None,
        executed: Some(&executed),
    };
    let mut verdict = crate::trend_cmd::verdicts::verdict_of(&inp, manifest, root);
    // A probe of the module of the line that has no column (`not_loaded`) says the probe read
    // nothing, not that the function never ran: the coverage of the run is asked directly.
    if verdict.verdict == Verdict::NotLoaded {
        let by_coverage = crate::trend_cmd::verdicts::coverage_of(&inp, root);
        if by_coverage.verdict == Verdict::Executed {
            verdict = by_coverage;
        }
    }
    let function_ran = matches!(
        verdict.verdict,
        Verdict::Executed | Verdict::Constant | Verdict::Grew | Verdict::Changed
    );
    let (col, site, h) = (&gone.column, &gone.site, &manifest.id);
    let span = dla_core::harness::LineRuns::load(run_dir, h, &series.id)
        .and_then(|r| r.span(site, series.warmup as usize));
    // A retention record of the run that shows a holder of the project keeping the line, or a
    // holder the finding's owner names, outweighs a unit in which the blocks do not stay.
    let scan = dla_core::measured_holders::scan_in(ledger, Some(run_dir));
    let finding = ledger.finding(root);
    let holder = scan.project.iter().find(|p| {
        p.lines.iter().any(|l| l == site)
            || finding.is_some_and(|f| dla_core::measured_holders::finding_covers(f, &p.owner))
    });
    match span {
        Some((first, last)) if last > first && holder.is_some() => {
            let h = holder.expect("checked");
            let n = (last - first) as u64;
            (
                Favors::Inconclusive,
                format!(
                    "the line ran {n} times in this series and none of its blocks stayed, but {} keeps {site} according to the retention record {} (a project holder): a unit in which the blocks do not stay does not answer that record",
                    h.key, h.evidence
                ),
                format!(
                    "{col} is not in this series and the line {site} ran {n} times without keeping its blocks; the retention record {} shows the holder {} keeping that line, so the series measured another unit than the one that grows, and it is inconclusive",
                    h.evidence, h.key
                ),
                LineRan::HolderRecord(h.harness.clone()),
            )
        }
        Some((first, last)) if last > first => {
            let n = (last - first) as u64;
            (
                Favors::NonLeak,
                format!(
                    "the line ran {n} times in this series and none of its blocks stayed ({site} of {root}; {col}, which counted growth at that line in another series, is not in this one)"
                ),
                format!(
                    "{col} is not in this series, and the agent writes only the sites that grew: the agent counted {n} executions of the line {site} over the measured points, so the line ran and none of its blocks stayed"
                ),
                LineRan::Line,
            )
        }
        Some(_) if function_ran => (
            Favors::Inconclusive,
            format!(
                "the function of {site} ran but the line itself did not (a branch the unit does not take)"
            ),
            format!(
                "{col} is not in this series and the agent's count of the executions of {site} did not rise over the measured points: the function ran ({}) but the line did not, so nothing was measured at the line",
                verdict.detail
            ),
            LineRan::FunctionOnly,
        ),
        None if function_ran => (
            Favors::Inconclusive,
            format!(
                "this series has no count of how often the line {site} ran, so the absence of {col} proves nothing about it"
            ),
            format!(
                "{col} is not in this series and the series has no count of the executions of {site} (an old agent, another runtime, or a point that lost its sections): the function ran ({}), which does not say that the line did, so the result is inconclusive",
                verdict.detail
            ),
            LineRan::NoCountFunctionRan,
        ),
        _ => (
            Favors::Inconclusive,
            format!("the unit of {h} does not reach {site}"),
            format!(
                "{col} is not in this series and the run does not show that the function of {site} ran ({}): the unit of {h} does not reach {site}, so nothing was measured at the line",
                verdict.detail
            ),
            if span.is_some() {
                LineRan::NotReached
            } else {
                LineRan::NoCount
            },
        ),
    }
}

/// Why a flat series is a fair reading for a finding (see [`exemption`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Exemption {
    /// An earlier series of this harness grew the finding: a leak grows with every unit, so growth
    /// that does not come back there was not a per-unit leak.
    Growth,
    /// The finding was raised on this harness (it cites evidence a run of it produced) and no
    /// growth showed there.
    Raised,
    /// An entry-path harness declared for the finding: it asks whether a caller can feed the
    /// holder, so a flat reading there keeps its meaning.
    EntryPath,
}

/// True when the evidence `e` was produced by a run of harness `manifest`: a `workload` record
/// naming it (`result.harness`) when the harness runs the project's own workload (a command-mode
/// harness with a workload, or a test loop: the test `HarnessAbout::OwnWorkload` uses; the record
/// of a run of a hand-written hook driver does not count), a growth or unit-count record of one of
/// its series (`result.series` whose `source.harness` is it), or a capture summary (retention or
/// allocation stacks) read from a file under `harness/<H>/` (the artifact path of its source).
fn produced_by(
    run_dir: &Path,
    e: &dla_core::evidence::EvidenceRecord,
    manifest: &HarnessManifest,
) -> bool {
    use dla_core::evidence::{EvidenceKind, Source};
    let harness = manifest.id.as_str();
    match e.kind {
        EvidenceKind::Workload => {
            (manifest.is_test_loop()
                || (manifest.mode == dla_core::harness::HarnessMode::Command
                    && manifest.workload.is_some()))
                && dla_core::harness::harness_id_of(&e.result) == Some(harness)
        }
        EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries => e
            .result
            .get("series")
            .and_then(Value::as_str)
            .and_then(|s| SeriesFile::load(run_dir, s).ok())
            .and_then(|s| s.source.harness)
            .is_some_and(|h| h == harness),
        EvidenceKind::RetentionPath | EvidenceKind::AllocStack => match &e.source {
            Source::Artifact { path, .. } => {
                dla_core::measured_holders::harness_of_path(path) == Some(harness)
            }
            _ => false,
        },
        _ => false,
    }
}

/// The two cases in which an alloc-test series that did not show the growth of the finding is
/// still a fair reading; anywhere else the series belongs to "another driver" and says nothing
/// against a growth another series measured.
///
/// * An entry-path harness declared for the finding: the purpose `entry-path` and the root (or
///   the finding itself) among its targets.
/// * The harness the finding was raised on: some evidence that the finding or a finding of its
///   family cites (`supports` or `contradicts`) was produced by a run of this harness
///   ([`produced_by`]; a discriminator is not counted: it is the result of a test, not what the
///   finding was raised on), or an earlier trend of the run, other than the series under test,
///   belongs to a series of this harness and has a column tied to the root
///   ([`dla_core::grown::column_tied_to`]) that counts as growth
///   ([`dla_core::grown::counts_as_growth`]). That the harness declares the finding a target or
///   has a probe for it makes it none of these.
///
/// [`Exemption::Growth`] is the case that an earlier series of the harness grew the finding;
/// [`Exemption::Raised`] the case that it only produced evidence the finding cites.
fn exemption(
    run_dir: &Path,
    ledger: &Ledger,
    manifest: &HarnessManifest,
    finding: &Finding,
    trends: &[&TrendReport],
    this_series: &str,
) -> Option<Exemption> {
    let root = finding.root_id();
    if manifest.is_entry_path()
        && manifest
            .targets
            .iter()
            .any(|t| t == &root || t == &finding.id)
    {
        return Some(Exemption::EntryPath);
    }
    let grew = trends
        .iter()
        .filter(|t| t.series != this_series)
        .filter(|t| {
            t.columns.iter().any(|c| {
                dla_core::grown::counts_as_growth(c)
                    && !dla_core::gate::is_whole_process(&c.name)
                    && dla_core::grown::column_tied_to(ledger, c, &root)
            })
        })
        .any(|t| {
            SeriesFile::load(run_dir, &t.series)
                .ok()
                .and_then(|s| s.source.harness)
                .is_some_and(|h| h == manifest.id)
        });
    if grew {
        return Some(Exemption::Growth);
    }
    let cited = ledger
        .findings()
        .filter(|f| f.id == root || f.parent.as_ref() == Some(&root))
        .flat_map(|f| f.supports.iter().chain(&f.contradicts))
        .filter_map(|id| ledger.evidence().find(|e| &e.evidence_id == id))
        .any(|e| produced_by(run_dir, e, manifest));
    cited.then_some(Exemption::Raised)
}

/// The harness and series of the newest trend of the run, other than the one of `this`, that
/// shows growth tied to the finding `root` (a column that passed the gate or a non-flat
/// allocation-site column: [`dla_core::grown::counts_as_growth`]). The series' harness is the one
/// to run the alloc-test on.
fn growth_elsewhere(
    run_dir: &Path,
    ledger: &Ledger,
    root: &FindingId,
    trends: &[&TrendReport],
    this: &str,
) -> Option<(String, String)> {
    trends.iter().rev().find_map(|t| {
        if t.series == this {
            return None;
        }
        t.columns
            .iter()
            .any(|c| {
                dla_core::grown::counts_as_growth(c)
                    && !dla_core::gate::is_whole_process(&c.name)
                    && dla_core::grown::column_tied_to(ledger, c, root)
            })
            .then(|| {
                let h = SeriesFile::load(run_dir, &t.series).ok()?.source.harness?;
                Some((h, t.series.clone()))
            })
            .flatten()
    })
}

/// The lines whose executions the agent counts in an alloc-test of `finding`: its site, and the
/// line of every allocation-site count column that grew in a trend of the run and is tied to it.
fn lines_of_finding(ledger: &Ledger, finding: &Finding, trends: &[&TrendReport]) -> Vec<String> {
    let root = finding.root_id();
    let mut out: Vec<String> = Vec::new();
    let mut add = |site: &str| {
        let site = site.trim();
        if !site.is_empty() && !out.iter().any(|s| s == site) {
            out.push(site.to_owned());
        }
    };
    let root_f = ledger.finding(&root).unwrap_or(finding);
    for f in [root_f, finding] {
        if let Some(site) = &f.site {
            add(site);
        }
    }
    for c in trends.iter().flat_map(|t| t.columns.iter()) {
        if dla_core::grown::is_alloc_site_column(&c.name)
            && dla_core::grown::counts_as_growth(c)
            && dla_core::grown::column_tied_to(ledger, c, &root)
            && let Some(site) = dla_core::grown::column_site(c)
        {
            add(site);
        }
    }
    out
}

/// Most times the extra window is lengthened to reach the expected rises (a longer one is left
/// to a person: `--extra`).
const MAX_WIDEN: u32 = 8;
/// Most times a run is repeated with a longer extra window.
const MAX_WIDENINGS: usize = 2;

/// The extra window that would decide a measurement whose only open question is the length of
/// that window: its verdict is inconclusive, no point was lost to the deadline, and a deciding
/// plateau was held back against a purge that kept the growth for want of expected rises
/// (see [`judge::combine`]). `None` when it is not so, or the window would be over `limit`
/// points ([`MAX_WIDEN`] times the first one).
fn wider_window(
    results: &[ColumnResult],
    picked: &pick::Picked,
    partial: &[String],
    extra: u32,
    limit: u32,
) -> Option<u32> {
    if !partial.is_empty() || decide(results, picked).0 != Favors::Inconclusive {
        return None;
    }
    let n = deciding_need(results, picked)?;
    (n > extra && n <= limit).then_some(n)
}

/// The extra points the deciding columns (the primary one, else the voting ones) that read
/// inconclusive name as the window that would decide them (`points_needed`, from a staircase or
/// from the rises of the main window): the largest, when any names one.
fn deciding_need(results: &[ColumnResult], picked: &pick::Picked) -> Option<u32> {
    results
        .iter()
        .filter(|r| match &picked.primary {
            Some(p) => &r.name == p,
            None => r.voting,
        })
        .filter(|r| r.favors == Favors::Inconclusive)
        .filter_map(|r| r.points_needed)
        .max()
}

/// Whether `n` extra points fit the time left, going by the `secs` the run of `extra` points
/// took (an overestimate where the run also held the main window) and the reserve every run
/// keeps for what follows its points. No deadline known: it fits.
fn window_fits(tc: &ToolCtx, secs: f64, extra: u32, n: u32) -> bool {
    let Some((left, _)) = tc.time_left() else {
        return true;
    };
    let left = left.as_secs_f64();
    let need = secs * f64::from(n) / f64::from(extra.max(1));
    need + dla_workload::harness::pace::reserve_secs(left) <= left
}

/// What one measurement of an alloc-test holds: the series, its trend, and every column tested.
struct Measured {
    attribution: Option<Map<String, Value>>,
    env_reused: Vec<String>,
    exemption: Option<Exemption>,
    limitations: Vec<String>,
    partial: Vec<String>,
    picked: pick::Picked,
    results: Vec<ColumnResult>,
    series: SeriesFile,
    trend: TrendReport,
    all_trends: Vec<TrendReport>,
    /// Seconds the points took.
    secs: f64,
}

/// Runs the points of an alloc-test with an extra window of `extra` points, joins them to the
/// main window of an earlier series where one is reused, and tests the columns.
fn measure(
    tc: &ToolCtx,
    manifest: &HarnessManifest,
    a: &AllocTestArgs,
    ledger: &Ledger,
    bound: Option<&Finding>,
    purge_kind: &Option<String>,
    extra: u32,
) -> Res<Measured> {
    let began = std::time::Instant::now();
    // Command mode: a complete harness series of the same mode already holds the main
    // window, so only the extra points are run and the two are joined.
    let reuse = (manifest.mode == HarnessMode::Command)
        .then(|| complete_series(&tc.run_dir, manifest, key_mode(manifest)))
        .flatten();
    let opts = RunOptions {
        modes: vec![key_mode(manifest)],
        iterations: Some(manifest.iterations),
        extra,
        purge: purge_kind.clone(),
        source_kind: SeriesSourceKind::AllocTest,
        first_point: reuse
            .as_ref()
            .map_or(1, |_| manifest.warmup + manifest.iterations + 1),
        attribute: attribution_target(manifest, bound),
        // Python: the agent counts how often the lines of the finding run, so that a line on a
        // branch the unit does not take is told from a line that ran and kept nothing.
        lines: match (&bound, manifest.runtime) {
            (Some(f), Runtime::Python) => {
                let earlier = TrendReport::load_all(&tc.run_dir);
                let refs: Vec<&TrendReport> = earlier.iter().collect();
                lines_of_finding(ledger, f, &refs)
            }
            _ => Vec::new(),
        },
        ..RunOptions::default()
    };
    let out = run_points(tc, manifest, &opts)?;
    // The loop wrote its per-class figures to `harness/<H>/attribution-<F>.json`, the place the
    // trigger and the bundle read.
    let attributed = opts.attribute.as_ref().and_then(|t| {
        dla_core::testloop::Attributed::parse(
            &fs::read_to_string(tc.run_dir.join(format!(
                "{}/{}/{}",
                dla_core::harness::HARNESS_DIR,
                a.harness,
                dla_core::testloop::attribution_file(&t.finding)
            )))
            .ok()?,
        )
    });
    let attribution = attributed.as_ref().map(attribution_summary);
    let partial: Vec<String> = out.partial.clone();
    let env_reused: Vec<String> = out.env_reused.clone();
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
    let all_trends = TrendReport::load_all(&tc.run_dir);
    let all_refs: Vec<&TrendReport> = all_trends.iter().collect();
    let picked = pick::pick_in(
        &series,
        ledger,
        bound,
        &a.columns,
        previous.as_ref(),
        &all_refs,
        &a.harness,
    )?;
    let trend = trend_of(tc, ledger, &series, previous.as_ref())?;
    let event = purge_event(&series);
    let mut limitations: Vec<String> = series.limitations.clone();
    limitations.extend(out.gaps.iter().cloned());
    limitations.extend(picked.note.clone());
    if opts.attribute.is_some() && attribution.is_none() {
        limitations.push(
            "which test class feeds the holder was not measured: the loop wrote no attribution file (see the run log)"
                .to_owned(),
        );
    }
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
    // A series that did not show the growth counts against it only on a harness that is neither the
    // one the finding was raised on nor an entry-path harness of the finding.
    let exemption = bound
        .as_ref()
        .and_then(|f| exemption(&tc.run_dir, ledger, manifest, f, &all_refs, &series.id));
    let mut results: Vec<ColumnResult> = picked
        .tested
        .iter()
        .filter_map(|n| {
            test_column(
                &series,
                n,
                picked.voting.contains(n),
                bound.is_some() && exemption.is_none(),
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

    Ok(Measured {
        attribution,
        env_reused,
        exemption,
        limitations,
        partial,
        picked,
        results,
        series,
        trend,
        all_trends,
        secs: began.elapsed().as_secs_f64(),
    })
}

/// Runs `dla alloc-test`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: AllocTestArgs) -> Res<Value> {
    if !a.harness.starts_with('H') {
        return Err(bad_harness_id(&a.harness));
    }
    let tc = stage::tool_ctx(project, ctx, data);
    tc.require_time("alloc-test")?;
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
    // A test says something about a finding only on a harness that is about it; refused before
    // anything runs.
    if let Some(f) = &bound {
        let view = dla_core::RunView::load_for(&tc.run_dir, tc.worker.as_deref())?;
        if dla_core::grown::harness_about(&view, &manifest, &f.root_id()).is_none() {
            return Err(harness_not_about(
                &view,
                &a.harness,
                manifest.runtime,
                f,
                "an alloc-test there",
            ));
        }
    }
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

    let mut extra = extra;
    let mut m = measure(
        &tc,
        &manifest,
        &a,
        &ledger,
        bound.as_ref(),
        &purge_kind,
        extra,
    )?;
    // Only the length of the extra window held a plateau back: run it again with the window
    // the expectation needs, as far as the time left allows. The longer window moves the noise
    // the rises are counted over, so the number of points needed is read again from the longer
    // run, once more at most.
    let first_extra = extra;
    let mut widening: Vec<String> = Vec::new();
    for _ in 0..MAX_WIDENINGS {
        let Some(n) = wider_window(
            &m.results,
            &m.picked,
            &m.partial,
            extra,
            first_extra.saturating_mul(MAX_WIDEN),
        ) else {
            break;
        };
        if !window_fits(&tc, m.secs, extra, n) {
            m.limitations.push(format!(
                "the extra window of {extra} points is too short for a plateau to stand against the purge that kept the growth ({n} points would be); the time left does not allow the longer run"
            ));
            break;
        }
        match measure(&tc, &manifest, &a, &ledger, bound.as_ref(), &purge_kind, n) {
            Ok(w) => {
                widening.push(format!(
                    "the extra window was lengthened from {extra} to {n} points and the command run again, because its length alone kept a plateau from deciding against a purge that kept the growth (series {} is the shorter run)",
                    m.series.id
                ));
                m = w;
                extra = n;
            }
            Err(f) if f.refusal.code == codes::TIME_BUDGET => {
                m.limitations.push(format!(
                    "the extra window of {extra} points is too short for a plateau to stand against the purge that kept the growth ({n} points would be); the longer run was refused for the time left"
                ));
                break;
            }
            Err(f) => return Err(f),
        }
    }
    m.limitations.extend(widening);
    let Measured {
        attribution,
        env_reused,
        exemption,
        mut limitations,
        partial,
        mut picked,
        results,
        series,
        trend,
        all_trends,
        ..
    } = m;
    let all_refs: Vec<&TrendReport> = all_trends.iter().collect();

    // The verdict: the bound column's, or the voting columns together, with the required
    // columns able to hold a non-leak verdict back.
    let (favors, supports, held_back) = decide(&results, &picked);
    // The count column of the finding's line is missing from this series: the agent's count of
    // the executions of the line says whether the line ran, and that decides.
    let mut line_ran: Option<LineRan> = None;
    let (favors, supports) = match (&picked.absent_site.clone(), bound.as_ref()) {
        (Some(gone), Some(f)) => {
            let (favors, basis, note, ran) = absent_site_reading(
                &tc.run_dir,
                project,
                &ledger,
                &manifest,
                &series,
                &trend,
                &f.root_id(),
                gone,
            );
            picked.basis = basis;
            match picked
                .note
                .as_ref()
                .and_then(|old| limitations.iter_mut().find(|l| *l == old))
            {
                Some(slot) => slot.clone_from(&note),
                None => limitations.push(note.clone()),
            }
            picked.note = Some(note);
            line_ran = Some(ran);
            (favors, Vec::new())
        }
        _ => (favors, supports),
    };
    // Every deciding column of the bound finding is flat from its first measured point: the unit does
    // not feed what grew elsewhere, which says nothing against that growth.
    let mut never_grew: Option<Option<(String, String)>> = None;
    let (favors, supports) = match bound.as_ref() {
        Some(f) if picked.absent_site.is_none() => {
            let deciding: Vec<&ColumnResult> = results
                .iter()
                .filter(|r| match &picked.primary {
                    Some(p) => &r.name == p,
                    None => r.voting,
                })
                .collect();
            if !deciding.is_empty()
                && deciding
                    .iter()
                    .all(|r| r.plateau.verdict == PlateauVerdict::NeverGrew)
            {
                let text = format!(
                    "this series does not show the growth of {}: {} is flat from its first measured point, so the unit of {} does not feed it",
                    f.root_id(),
                    deciding[0].name,
                    a.harness
                );
                picked.basis = text.clone();
                limitations.push(text);
                never_grew = Some(growth_elsewhere(
                    &tc.run_dir,
                    &ledger,
                    &f.root_id(),
                    &all_refs,
                    &series.id,
                ));
                (Favors::Inconclusive, Vec::new())
            } else {
                (favors, supports)
            }
        }
        _ => (favors, supports),
    };
    // On the harness that measured the growth a flat series keeps its old reading, and says that the
    // growth did not come back there.
    if matches!(exemption, Some(Exemption::Growth | Exemption::Raised))
        && let Some(f) = bound.as_ref()
        && let deciding = results
            .iter()
            .filter(|r| match &picked.primary {
                Some(p) => &r.name == p,
                None => r.voting,
            })
            .collect::<Vec<_>>()
        && !deciding.is_empty()
        && deciding.iter().all(|r| r.flat)
    {
        let text = if exemption == Some(Exemption::Growth) {
            format!(
                "the growth of {} did not come back on the harness that measured it: {} is flat from its first measured point in this series of {}, so it was not a growth of every unit",
                f.root_id(),
                deciding[0].name,
                a.harness
            )
        } else {
            format!(
                "no growth of {} showed on the harness it was raised on: {} is flat from its first measured point in this series of {}, so it was not a growth of every unit",
                f.root_id(),
                deciding[0].name,
                a.harness
            )
        };
        picked.basis = text.clone();
        limitations.push(text);
    }
    // A required column that keeps rising is decided by its capture, a longer run does not; one
    // that only could not be judged (still levelling off) may show its plateau in a longer window.
    let split_open = |names: &[String]| split_rising(&results, &picked, names);
    let primary_name = picked.primary.as_deref().unwrap_or("the deciding column");
    let (held_rising, held_levelling) = split_open(&held_back);
    if !held_rising.is_empty() {
        limitations.push(format!(
            "{} did not plateau (it keeps rising, or its net rise over the extra window is over the jitter): the plateau of {primary_name} is held back, so the result is inconclusive; a longer --extra does not decide a column that keeps rising: capture it (the thread capture for a thread count, the NMT category for jvm.nmt.*) and read that column's series",
            held_rising.join(", ")
        ));
    }
    if !held_levelling.is_empty() {
        limitations.push(format!(
            "{} could not be judged yet (still levelling off): the plateau of {primary_name} is held back, so the result is inconclusive; a longer --extra may show whether it plateaus",
            held_levelling.join(", ")
        ));
    }
    // An inconclusive result with a required column that is not at a plateau.
    let open_columns: Vec<String> = if held_back.is_empty() && favors == Favors::Inconclusive {
        open_required(&results, &picked)
    } else {
        Vec::new()
    };
    let (open_rising, open_levelling) = split_open(&open_columns);
    if !open_rising.is_empty() {
        limitations.push(format!(
            "{} did not plateau and the result is inconclusive; a longer --extra does not decide a column that keeps rising: capture it (the thread capture for a thread count, the NMT category for jvm.nmt.*)",
            open_rising.join(", ")
        ));
    }
    if !open_levelling.is_empty() {
        limitations.push(format!(
            "{} could not be judged yet (still levelling off) and the result is inconclusive; a longer --extra may show whether it plateaus",
            open_levelling.join(", ")
        ));
    }
    let rising_open: Vec<String> = held_rising.iter().chain(&open_rising).cloned().collect();
    // A small non-heap rise on a flat class count, with no required column rising, is no evidence
    // of a leak: the verdict becomes inconclusive, so no refute or confirm is printed for it, and
    // the caution is shown. When a required column rises, it is named instead, never the drift.
    let ctx = DriftCtx {
        extra,
        command: manifest.mode == HarnessMode::Command,
        // The drift text names jvm.* columns and `harness gen --junit`: the JVM loop only.
        junit: manifest
            .tools
            .iter()
            .any(|t| t == dla_core::harness::JUNIT_TOOL),
        partial: !partial.is_empty(),
    };
    let drift = agent_drift(&results, &picked, favors, &a.harness, ctx);
    let (favors, supports) = match &drift {
        Some(Drift::Agent(_))
        | Some(Drift::Growing {
            inconclusive: true, ..
        }) => (Favors::Inconclusive, Vec::new()),
        _ => (favors, supports),
    };
    match &drift {
        Some(Drift::Agent(note)) => limitations.push(note.clone()),
        Some(Drift::Growing { limitation, .. }) => limitations.push(limitation.clone()),
        Some(Drift::Classes(note)) => limitations.push(note.clone()),
        None => {}
    }
    // A Go field probe of the bound finding that was never read (nothing tracked, or a field
    // with no length): the whole-process columns that decided in its place cannot show a
    // step-growing container growing, so they never favour non-leak.
    let unread = bound
        .as_ref()
        .and_then(|f| unread_field_probe(&manifest, &series, f));
    let (favors, supports) = match &unread {
        Some(note) => {
            limitations.push(note.clone());
            cap_for_unread_probe(favors, supports)
        }
        None => (favors, supports),
    };
    // A run the deadline cut short measured less than it planned: whatever its columns
    // say, it never favours non-leak (and never confirms) on its own.
    let (favors, supports) = cap_for_partial(&partial, favors, supports);
    if !partial.is_empty() {
        limitations.push(
            "partial run: the run deadline stopped the points before the plan, so this result is inconclusive whatever the columns show"
                .to_owned(),
        );
    }

    // Files: the full result.
    let alloc_id = tc.next_alloc_id();
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
        "decided_by": decided_by(&picked),
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
    // An inconclusive result that names the longer window which would decide it: the ledger keeps
    // the finding open while the budget can pay for that window (`grown::decided_by_alloc_test`).
    if favors == Favors::Inconclusive
        && partial.is_empty()
        && let Some(n) = deciding_need(&results, &picked)
    {
        result.extra.insert("points_needed".into(), json!(n));
    }
    if let Some(att) = &attribution {
        for (k, v) in att {
            result.extra.insert(k.clone(), v.clone());
        }
    }
    if !partial.is_empty() {
        result.extra.insert(
            "partial".into(),
            json!({"reason": "the run deadline stopped the points before the plan", "first": partial.iter().take(2).collect::<Vec<_>>()}),
        );
    }
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

    let direct = bound.as_ref().and_then(|f| {
        crate::ledger_cmd::direct_trigger(&tc.run_dir, tc.worker.as_deref(), &f.root_id())
    });
    let decided = decided_by(&picked);
    let stacks_ctx = StacksCtx {
        decided: decided.as_deref(),
        runtime: manifest.runtime,
        mode: key_mode(&manifest).as_str(),
    };
    let next = match &drift {
        Some(Drift::Agent(_)) => vec![drift_next(ctx)],
        Some(Drift::Growing {
            next, inconclusive, ..
        }) => {
            let mut n = vec![next.clone()];
            if keeps_ledger_commands(*inconclusive, !partial.is_empty()) {
                n.extend(next_commands(
                    &ledger,
                    bound.as_ref(),
                    &a.harness,
                    favors,
                    &supports,
                    &evidence,
                    extra,
                    direct.as_ref(),
                    Some(&stacks_ctx),
                ));
            }
            n
        }
        None | Some(Drift::Classes(_))
            if favors == Favors::Inconclusive && !rising_open.is_empty() =>
        {
            let cols: Vec<&str> = rising_open.iter().map(String::as_str).collect();
            vec![deciding_capture(&cols, &a.harness)]
        }
        None | Some(Drift::Classes(_)) => next_commands(
            &ledger,
            bound.as_ref(),
            &a.harness,
            favors,
            &supports,
            &evidence,
            extra,
            direct.as_ref(),
            Some(&stacks_ctx),
        ),
    };
    // The unit does not reach the line: a longer run decides nothing, the unit has to call it.
    let next = match (&picked.absent_site, favors, bound.as_ref()) {
        (Some(gone), Favors::Inconclusive, Some(f)) => {
            let root = f.root_id();
            let how = match line_ran {
                Some(LineRan::HolderRecord(_)) => {
                    "run the unit of {h} so that it keeps the objects of the line {site} as the capture shows"
                }
                Some(LineRan::FunctionOnly | LineRan::NoCountFunctionRan) => {
                    "make the unit of {h} execute the line {site} itself: call the function with what takes the branch of the line (edit the hooks of its driver)"
                }
                _ => {
                    "make the unit of {h} call the function of the line {site} (edit the hooks of its driver)"
                }
            };
            let mut v: Vec<String> = Vec::new();
            if let Some(LineRan::HolderRecord(Some(captured))) = &line_ran
                && captured != &a.harness
            {
                v.push(format!(
                    "dla alloc-test {captured} --finding {root}   (the capture that shows the holder was made there)"
                ));
            }
            // The harness of the series that showed the growth has the unit that reaches the line.
            if let Some(measured) = SeriesFile::load(&tc.run_dir, &gone.series)
                .ok()
                .and_then(|s| s.source.harness)
                .filter(|m| m != &a.harness)
            {
                v.push(format!(
                    "dla alloc-test {measured} --finding {root}   (the growth was measured there)"
                ));
            }
            v.push(format!(
                "{}, then dla harness run {h} --mode {mode}, dla trend <S>, dla alloc-test {h} --finding {root}",
                how.replace("{h}", &a.harness).replace("{site}", &gone.site),
                h = a.harness,
                mode = key_mode(&manifest).as_str(),
            ));
            v
        }
        _ => next,
    };
    let next = match (&never_grew, bound.as_ref()) {
        (Some(elsewhere), Some(f)) => {
            let root = f.root_id();
            let mut v: Vec<String> = Vec::new();
            if let Some((h, s)) = elsewhere {
                v.push(format!(
                    "dla alloc-test {h} --finding {root}   (the growth was measured there, in {s})"
                ));
            }
            v.push(format!(
                "make the unit of {h} feed what grew (edit the hooks of its driver), then dla harness run {h} --mode {mode}, dla trend <S>, dla alloc-test {h} --finding {root}",
                h = a.harness,
                mode = key_mode(&manifest).as_str()
            ));
            v
        }
        _ => next,
    };
    let next = say_how_long_a_held_plateau_needs(next, favors, &results, &picked);
    // The hint rides with the capture command: a key the output bound never cuts.
    let stacks_note = stacks_offer(
        &ledger,
        bound.as_ref(),
        &a.harness,
        favors,
        Some(&stacks_ctx),
    )
    .filter(|(lead, _)| next.contains(&lead[0]))
    .map(|(_, hint)| hint);
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
    out.insert("decided_by".into(), json!(decided_by(&picked)));
    out.insert("purge".into(), json!(purge_kind));
    if !env_reused.is_empty() {
        out.insert("env_reused".into(), json!(env_reused));
    }
    if let Some(att) = &attribution {
        for (k, v) in att {
            out.insert(k.clone(), v.clone());
        }
    }
    if let Some(Drift::Agent(note)) = &drift {
        out.insert("caution".into(), json!(note));
    }
    if let Some(hint) = &stacks_note {
        out.insert("stacks_hint".into(), json!(hint));
    }
    if let Some(hint) = shared_hint(&tc, &manifest, bound.as_ref(), favors) {
        out.insert("shared_hint".into(), json!(hint));
    }
    if !partial.is_empty() {
        out.insert(
            "partial".into(),
            json!(format!(
                "stopped at the run deadline before the plan ({}); inconclusive, never a refutation",
                partial.first().map_or("", String::as_str)
            )),
        );
    }
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

/// When a voting plateau was held back only because its window was too short to have shown
/// another rise (see [`judge::combine`]), the doubled `--extra` of the printed command says how
/// many extra points would reach the expectation.
fn say_how_long_a_held_plateau_needs(
    mut next: Vec<String>,
    favors: Favors,
    results: &[ColumnResult],
    picked: &pick::Picked,
) -> Vec<String> {
    if favors != Favors::Inconclusive {
        return next;
    }
    let needed = results
        .iter()
        .filter(|r| match &picked.primary {
            Some(p) => &r.name == p,
            None => r.voting,
        })
        .filter(|r| r.favors == Favors::Inconclusive)
        .filter_map(|r| {
            r.points_needed
                .map(|n| (n, r.plateau.points_needed.is_some()))
        })
        .max();
    if let (Some((n, stair)), Some(cmd)) = (needed, next.iter_mut().find(|c| c.contains("--extra")))
    {
        raise_extra(cmd, n);
        cmd.push_str(&if stair {
            format!(
                "   (the column grows in steps and no step has come for as long as the next gap is expected to be: {n} extra points would run past it)"
            )
        } else {
            format!(
                "   (the main window rose too few times for a plateau of this length to stand against the purge that kept the growth: {n} extra points would)"
            )
        });
    }
    next
}

/// Raises the `--extra <k>` of `cmd` to `n` points when `k` is fewer, so the printed command runs
/// the window that was named as deciding (the doubled window of `next_commands` can be shorter).
fn raise_extra(cmd: &mut String, n: u32) {
    const FLAG: &str = "--extra ";
    let Some(at) = cmd.find(FLAG).map(|i| i + FLAG.len()) else {
        return;
    };
    let digits = cmd[at..].chars().take_while(char::is_ascii_digit).count();
    if cmd[at..at + digits].parse::<u32>().is_ok_and(|k| k < n) {
        cmd.replace_range(at..at + digits, &n.to_string());
    }
}

/// What `next_commands` needs to offer the stack capture: the column that decided, the runtime
/// of the harness and the key mode the alloc-test ran.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StacksCtx<'a> {
    /// The column whose reading decided the verdict.
    pub decided: Option<&'a str>,
    /// The runtime of the harness.
    pub runtime: Runtime,
    /// The key mode of the run (`fresh`, `native`).
    pub mode: &'a str,
}

/// The two commands that capture the stacks of a growing allocation line and tie them to the
/// finding: the capture run, then a line that says to run the summarize command it prints (a `#`
/// line: nothing tries to execute it).
pub(crate) fn stacks_commands(
    runtime: Runtime,
    harness: &str,
    mode: &str,
    finding: &str,
    site: &str,
) -> [String; 2] {
    let capture = dla_workload::harness::capture::stacks_capture(runtime);
    let line = if dla_workload::harness::capture::heap_is_holder_chains(runtime) {
        format!(
            "# run the two dla summarize commands that run prints: alloc-stack-diff (it ties the stacks at {site} to {finding}) and heap-graph (who holds the objects of the line)"
        )
    } else {
        format!(
            "# run the dla summarize alloc-stack-diff command that run prints; it ties the stacks at {site} to {finding}"
        )
    };
    [
        format!("dla harness run {harness} --mode {mode} --capture {capture}"),
        line,
    ]
}

/// The sentence of the `shared_hint` key: the result favors leak on a harness that is about the
/// bound finding but not made for it (a test loop, the project's own workload, or about it only
/// through a probe, a column, cited evidence or a capture). Growth there may be kept by another
/// holder, since many objects of the program are alive together; a driver made for the finding
/// decides it alone. `None` for any other result.
fn shared_hint(
    tc: &ToolCtx,
    manifest: &HarnessManifest,
    bound: Option<&Finding>,
    favors: Favors,
) -> Option<String> {
    let root = bound?.root_id();
    if favors != Favors::Leak {
        return None;
    }
    let view = dla_core::RunView::load_for(&tc.run_dir, tc.worker.as_deref()).ok()?;
    let about = dla_core::grown::harness_about(&view, manifest, &root)?;
    shared_hint_text(manifest, &about, &root)
}

/// The text of [`shared_hint`] for a harness that is about `root` as `about` says; `None` when
/// the harness is a hook driver made for it.
fn shared_hint_text(
    manifest: &HarnessManifest,
    about: &dla_core::grown::HarnessAbout,
    root: &FindingId,
) -> Option<String> {
    use dla_core::grown::HarnessAbout;
    if manifest.is_hook_driver() && matches!(about, HarnessAbout::Target | HarnessAbout::EntryPath)
    {
        return None;
    }
    Some(format!(
        "a confirm from {h} may count growth that another holder keeps ({why}: many objects of the program are alive together there); a driver made for {root} decides it alone: `dla harness gen --lang {rt} --driver --probe {root}`, then dla harness run <H'>, dla trend <S>, dla alloc-test <H'> --finding {root}",
        h = manifest.id,
        why = dla_core::grown::not_made_for(manifest, about),
        rt = manifest.runtime.as_str()
    ))
}

/// The sentence of the `stacks_hint` key (never cut by the output bound).
pub(crate) fn stacks_hint(runtime: Runtime, harness: &str, mode: &str, finding: &str) -> String {
    let capture = dla_workload::harness::capture::stacks_capture(runtime);
    if dla_workload::harness::capture::heap_is_holder_chains(runtime) {
        return format!(
            "the growth is a count of blocks allocated at one line, and a verifier and an explanation need the stack that leads to it and the object that holds it: run `dla harness run {harness} --mode {mode} --capture {capture}`, then the two summarize commands that run prints: `dla summarize alloc-stack-diff` (the stacks, which tie to {finding}) and `dla summarize heap-graph` (who holds the objects of the line)"
        );
    }
    format!(
        "the growth is a count of blocks allocated at one line, and a verifier and an explanation need the stack that leads to it: run `dla harness run {harness} --mode {mode} --capture {capture}`, then the `dla summarize alloc-stack-diff` command that run prints, which ties the stacks at the line to {finding}"
    )
}

/// The `next` entries that capture the stacks, and the hint sentence, when they apply: the result
/// favors leak, the deciding column is an allocation-site column, the runtime of the harness
/// writes allocation captures, and no allocation-stack record names the finding yet (the test
/// `dla ledger label` uses).
fn stacks_offer(
    ledger: &Ledger,
    bound: Option<&Finding>,
    harness: &str,
    favors: Favors,
    stacks: Option<&StacksCtx>,
) -> Option<([String; 2], String)> {
    let (f, c) = (bound?, stacks?);
    let column = c.decided?;
    if favors != Favors::Leak
        || !dla_core::grown::is_alloc_site_column(column)
        || !dla_workload::harness::capture::supported(c.runtime).alloc
    {
        return None;
    }
    let root = f.root_id();
    let root_f = ledger.finding(&root).unwrap_or(f);
    if ledger.evidence().any(|e| {
        e.kind == EvidenceKind::AllocStack && dla_core::labels::about_finding(e, &[root_f])
    }) {
        return None;
    }
    let site = column.split_once('@').map_or("the line", |(_, s)| s);
    Some((
        stacks_commands(c.runtime, harness, c.mode, &root.to_string(), site),
        stacks_hint(c.runtime, harness, c.mode, &root.to_string()),
    ))
}

#[allow(clippy::too_many_arguments)]
fn next_commands(
    ledger: &Ledger,
    bound: Option<&Finding>,
    harness: &str,
    favors: Favors,
    supports: &[String],
    evidence: &EvidenceId,
    extra: u32,
    direct: Option<&(String, String, String)>,
    stacks: Option<&StacksCtx>,
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
                    && let Some(w) = dla_core::grown::workload_evidence(ledger, harness)
                {
                    next.push(format!("dla ledger test {} --evidence {w}", s.id));
                }
                next.push(format!("dla ledger confirm {} --evidence {evidence}", s.id));
            }
        }
        // The growth is the registration itself: `ledger confirm` is refused, and an entry-path
        // harness whose unit does not call the writer decides both readings.
        Favors::Leak if let Some((h, w, rt)) = direct => {
            next.push(format!(
                "dla harness gen --lang {rt} --driver --entry {root}   ({root} grew only because the unit of {h} calls {w} itself with new keys, so dla ledger confirm {root} is refused with registration-only; the unit of this harness uses the public API without calling {w}; fill the hooks and VARIES.txt, dla harness run <H> --mode both, dla trend <S>)"
            ));
            next.push(format!(
                "dla alloc-test <H> --finding {root}   (on that harness; if the table grows there: dla ledger confirm {root}; if it stays flat the alloc-test favors non-leak: dla ledger refute {root} --evidence <that alloc-test>)"
            ));
        }
        Favors::Leak => next.extend(dla_core::grown::confirm_commands(
            ledger, root_f, harness, evidence,
        )),
        Favors::Inconclusive => next.push(format!(
            "dla alloc-test {harness} --finding {} --extra {}",
            f.id,
            extra * 2
        )),
    }
    // The stack capture leads; when the list is too long the two entries and the confirm line
    // stay and the commands between them give way.
    let Some((lead, _)) = stacks_offer(ledger, bound, harness, favors, stacks) else {
        next.truncate(MAX_NEXT);
        return next;
    };
    let room = MAX_NEXT - lead.len();
    if next.len() > room {
        let confirm = next
            .last()
            .filter(|c| c.starts_with("dla ledger confirm"))
            .cloned();
        next.truncate(room - usize::from(confirm.is_some()));
        next.extend(confirm);
    }
    next.splice(0..0, lead);
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lines_an_alloc_test_counts_are_the_finding_and_the_columns_tied_to_it() {
        use dla_core::{finding::Region, ids::FindingId, trend::TrendReport};
        let mut f = Finding::candidate(
            FindingId::new(3),
            dla_core::Origin::parse("growth-series").unwrap(),
            vec![],
        );
        f.region = Some(Region::Heap);
        f.site = Some("shop/providers.py:8".to_owned());
        let col = |name: &str, candidates: Vec<&str>, rise: f64| -> dla_core::trend::ColumnTrend {
            serde_json::from_value(serde_json::json!({
                "name": name, "unit": "count", "kind": "counter", "region": "heap",
                "n": 8, "noise": 0.0, "rise": rise, "shape": "steady", "gate": "pass",
                "gate_failed": [], "candidates": candidates,
            }))
            .unwrap()
        };
        let trend = TrendReport {
            schema_version: 1,
            id: "T1".into(),
            series: "S1".into(),
            thresholds: dla_stats::GateThresholds::default(),
            columns: vec![
                // Tied by the candidate, at another line than the finding's.
                col("py.blocks@shop/providers.py:9", vec!["F3"], 10.0),
                // Another finding's line, a whole-process column and the finding's own line again.
                col("py.blocks@shop/other.py:2", vec!["F9"], 10.0),
                col("py.traced", vec!["F3"], 10.0),
                col("py.blocks@shop/providers.py:8", vec![], 10.0),
            ],
            targets: vec![],
        };
        let ledger = Ledger::from_text("").unwrap();
        assert_eq!(
            lines_of_finding(&ledger, &f, &[&trend]),
            ["shop/providers.py:8", "shop/providers.py:9"]
        );
        assert_eq!(lines_of_finding(&ledger, &f, &[]), ["shop/providers.py:8"]);
    }

    fn growth_evidence(ledger: &Ledger, root: &Finding) -> Option<EvidenceId> {
        let all: Vec<&dla_core::evidence::EvidenceRecord> = ledger.evidence().collect();
        dla_core::grown::growth_evidence(&all, root, &[])
    }
    use dla_core::series::{
        Axis, Column, ColumnScope, Point, SeriesRegion, SeriesSource, SeriesSourceKind,
    };

    #[test]
    fn the_growth_evidence_to_confirm_with_is_the_newest_tied_one_not_a_newer_whole_process_one() {
        use dla_core::{
            NewEvidence, NewFinding, Origin,
            evidence::{Authority, EvidenceCoverage, Source, Tool},
            state::{RunState, now_utc},
        };
        let dir = tempfile::tempdir().unwrap();
        RunState::init("/p", None, 3600, None, now_utc())
            .unwrap()
            .save(dir.path())
            .unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        let ev = |result: Value| NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            // What `dla trend` writes: the confirm rule accepts no growth record with static
            // authority, so the chooser does not name one.
            authority: Authority::Test,
            kind: EvidenceKind::GrowthSeries,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        };
        l.add_evidence(ev(serde_json::json!({"n": 1}))).unwrap();
        let last = |l: &Ledger| l.evidence().last().unwrap().evidence_id.clone();
        let base = last(&l);
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![base]);
        nf.site = Some("src/lib.rs:3".into());
        l.add_finding(nf).unwrap();
        // The tied stack evidence, then a newer whole-process trend that ties to nothing.
        l.add_evidence(ev(serde_json::json!({"gate": "pass", "passed": [
            {"column": "stack:f (src/lib.rs:6)", "finding": "F1", "site": "src/lib.rs:3"}]})))
            .unwrap();
        let tied = last(&l);
        l.add_evidence(ev(serde_json::json!({"gate": "pass", "passed": [
            {"column": "native.live_blocks"}]})))
            .unwrap();
        let root = l.finding(&"F1".parse().unwrap()).unwrap().clone();
        assert_eq!(growth_evidence(&l, &root), Some(tied));
        // Without a tied one the newest passing evidence is still named, as before.
        let other = Finding {
            id: "F9".parse().unwrap(),
            site: None,
            owner: None,
            ..root.clone()
        };
        assert_eq!(growth_evidence(&l, &other), Some(last(&l)));
    }

    #[test]
    fn a_root_closed_as_static_only_gets_refute_test_and_confirm_in_that_order() {
        use dla_core::{
            NewEvidence, NewFinding, Origin,
            evidence::{Authority, EvidenceCoverage, Source, Tool},
            finding::UnresolvedReason,
            state::{RunState, now_utc},
        };
        let dir = tempfile::tempdir().unwrap();
        RunState::init("/p", None, 3600, None, now_utc())
            .unwrap()
            .save(dir.path())
            .unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        let ev = |kind: EvidenceKind, result: Value| NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            // A growth record is what `dla trend` writes (authority test); the census and the
            // workload records are static.
            authority: if kind == EvidenceKind::GrowthSeries {
                Authority::Test
            } else {
                Authority::Static
            },
            kind,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        };
        let last = |l: &Ledger| l.evidence().last().unwrap().evidence_id.clone();
        l.add_evidence(ev(
            EvidenceKind::CensusHolder,
            serde_json::json!({"lang": "rust"}),
        ))
        .unwrap();
        let census = last(&l);
        l.add_evidence(ev(
            EvidenceKind::Workload,
            serde_json::json!({"harness": "H1", "mode": "driver"}),
        ))
        .unwrap();
        let workload = last(&l);
        l.add_evidence(ev(
            EvidenceKind::GrowthSeries,
            serde_json::json!({"gate": "pass", "passed": [
                {"column": "stack:f (src/lib.rs:6)", "finding": "F1", "site": "src/lib.rs:3"}]}),
        ))
        .unwrap();
        let growth = last(&l);
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![census.clone()]);
        nf.site = Some("src/lib.rs:3".into());
        l.add_finding(nf).unwrap();
        let root: FindingId = "F1".parse().unwrap();
        l.transition(
            &root,
            FindingState::SuspectedStaticOnly,
            Some(UnresolvedReason::NotExercised),
            Some("x".into()),
            vec![census.clone()],
            "t",
        )
        .unwrap();
        // With no sibling yet: add it, refute it with this result, then test and confirm (a
        // confirm without the refuted sibling is refused).
        {
            let f = l.finding(&root).unwrap().clone();
            let next = next_commands(
                &l,
                Some(&f),
                "H1",
                Favors::Leak,
                &[],
                &last(&l),
                4,
                None,
                None,
            );
            let at = |needle: &str| {
                next.iter()
                    .position(|n| n.starts_with(needle))
                    .unwrap_or(usize::MAX)
            };
            let add = at("dla ledger add --sibling-of F1 ");
            let refute = at("dla ledger refute F1.n1 --evidence ");
            let confirm = at(&format!("dla ledger confirm F1 --evidence {growth}"));
            assert!(
                add < refute && refute < confirm && confirm < next.len(),
                "{next:?}"
            );
        }
        let mut sib = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![census]);
        sib.sibling_of = Some(root.clone());
        sib.hypothesis = Some("N-WARM".into());
        sib.refute_if = Some("keeps growing".into());
        l.add_finding(sib).unwrap();
        let f = l.finding(&root).unwrap().clone();
        let next = next_commands(
            &l,
            Some(&f),
            "H1",
            Favors::Leak,
            &[],
            &last(&l),
            4,
            None,
            None,
        );
        let at = |needle: &str| next.iter().position(|n| n.starts_with(needle));
        let refute = at("dla ledger refute F1.n1").unwrap_or(usize::MAX);
        let test = at(&format!("dla ledger test F1 --evidence {workload}")).unwrap_or(usize::MAX);
        let confirm =
            at(&format!("dla ledger confirm F1 --evidence {growth}")).unwrap_or(usize::MAX);
        assert!(
            refute < test && test < confirm && confirm < next.len(),
            "{next:?}"
        );
    }

    /// A ledger with the leak-side F1 at `shop/store.py:5` and the evidence a stack record
    /// of a given result is added to.
    fn stacks_ledger() -> (tempfile::TempDir, Ledger) {
        use dla_core::{
            NewFinding, Origin,
            state::{RunState, now_utc},
        };
        let dir = tempfile::tempdir().unwrap();
        RunState::init("/p", None, 3600, None, now_utc())
            .unwrap()
            .save(dir.path())
            .unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        l.add_evidence(stack_evidence(
            EvidenceKind::GrowthSeries,
            Authority::Test,
            serde_json::json!({"n": 1}),
        ))
        .unwrap();
        let mut nf = NewFinding::new(
            Origin::parse("census").unwrap(),
            vec!["E1".parse().unwrap()],
        );
        nf.site = Some("shop/store.py:5".into());
        l.add_finding(nf).unwrap();
        (dir, l)
    }

    fn stack_evidence(
        kind: EvidenceKind,
        authority: Authority,
        result: serde_json::Value,
    ) -> dla_core::NewEvidence {
        dla_core::NewEvidence {
            source: dla_core::evidence::Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: dla_core::evidence::Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority,
            kind,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        }
    }

    const SITE_COLUMN: &str = "py.blocks@shop/store.py:5";

    fn offered(l: &Ledger, favors: Favors, decided: &str, runtime: Runtime) -> Vec<String> {
        let f = l.findings().next().unwrap().clone();
        let ctx = StacksCtx {
            decided: Some(decided),
            runtime,
            mode: "fresh",
        };
        let ev = l.evidence().next().unwrap().evidence_id.clone();
        next_commands(l, Some(&f), "H1", favors, &[], &ev, 4, None, Some(&ctx))
    }

    #[test]
    fn a_leak_decided_by_a_site_column_leads_with_the_capture_and_the_summarize_line() {
        let (_d, l) = stacks_ledger();
        let next = offered(&l, Favors::Leak, SITE_COLUMN, Runtime::Python);
        // Python: the heap capture is the holder chains, so the capture is `alloc,heap` and the
        // run prints two summarize commands.
        assert_eq!(
            next[0],
            "dla harness run H1 --mode fresh --capture alloc,heap"
        );
        assert!(
            next[1].starts_with("# run the two dla summarize commands that run prints")
                && next[1].contains("alloc-stack-diff")
                && next[1].contains("heap-graph")
                && next[1].contains("F1")
                && next[1].contains("shop/store.py:5"),
            "{next:?}"
        );
        assert!(next[2].starts_with("dla ledger"), "{next:?}");
        assert!(next.len() <= MAX_NEXT, "{next:?}");
        // A runtime whose heap capture is a whole-heap dump keeps the stacks capture alone.
        let node = offered(&l, Favors::Leak, SITE_COLUMN, Runtime::Node);
        assert_eq!(node[0], "dla harness run H1 --mode fresh --capture alloc");
        assert!(
            node[1].starts_with("# run the dla summarize alloc-stack-diff command that run prints"),
            "{node:?}"
        );
        assert!(
            stacks_hint(Runtime::Python, "H1", "fresh", "F1")
                .contains("`dla summarize heap-graph` (who holds the objects of the line)")
                && !stacks_hint(Runtime::Node, "H1", "fresh", "F1").contains("heap-graph"),
        );
        // A column that is not an allocation-site column, or a runtime without the capture,
        // leaves the list as it was.
        for (col, rt) in [("os.rss", Runtime::Python), (SITE_COLUMN, Runtime::Jvm)] {
            let next = offered(&l, Favors::Leak, col, rt);
            assert!(next.iter().all(|n| !n.contains("--capture")), "{next:?}");
        }
    }

    #[test]
    fn a_result_that_favors_non_leak_offers_no_capture() {
        let (_d, l) = stacks_ledger();
        for favors in [Favors::NonLeak, Favors::Inconclusive] {
            let next = offered(&l, favors, SITE_COLUMN, Runtime::Python);
            assert!(
                next.iter()
                    .all(|n| !n.contains("--capture") && !n.starts_with('#')),
                "{next:?}"
            );
        }
    }

    #[test]
    fn a_stack_record_that_names_the_finding_ends_the_offer() {
        let (_d, mut l) = stacks_ledger();
        // A stack of another finding does not.
        l.add_evidence(stack_evidence(
            EvidenceKind::AllocStack,
            Authority::Test,
            serde_json::json!({"finding": "F9"}),
        ))
        .unwrap();
        assert!(offered(&l, Favors::Leak, SITE_COLUMN, Runtime::Python)[0].contains("--capture"));
        l.add_evidence(stack_evidence(
            EvidenceKind::AllocStack,
            Authority::Test,
            serde_json::json!({"finding": "F1", "candidates": ["F1"]}),
        ))
        .unwrap();
        let next = offered(&l, Favors::Leak, SITE_COLUMN, Runtime::Python);
        assert!(
            next.iter().all(|n| !n.contains("--capture")) && next[0].starts_with("dla ledger"),
            "{next:?}"
        );
    }

    #[test]
    fn a_long_list_keeps_the_two_entries_and_the_confirm_line() {
        let (_d, mut l) = stacks_ledger();
        // Several open siblings make the ledger part longer than the room.
        for _ in 0..9 {
            let mut sib = dla_core::NewFinding::new(
                dla_core::Origin::parse("hypothesis").unwrap(),
                vec!["E1".parse().unwrap()],
            );
            sib.sibling_of = Some("F1".parse().unwrap());
            sib.hypothesis = Some("N-WARM".into());
            sib.refute_if = Some("keeps growing".into());
            l.add_finding(sib).unwrap();
        }
        let next = offered(&l, Favors::Leak, SITE_COLUMN, Runtime::Python);
        assert_eq!(next.len(), MAX_NEXT, "{next:?}");
        assert!(next[0].contains("--capture") && next[1].starts_with('#'));
        assert!(
            next.last().unwrap().starts_with("dla ledger confirm F1"),
            "{next:?}"
        );
    }

    #[test]
    fn a_growing_required_column_holds_back_the_plateau_of_the_primary() {
        let result = |name: &str, voting: bool, favors: Favors| ColumnResult {
            name: name.to_owned(),
            voting,
            plateau: Plateau {
                main_slope: None,
                extra_slope: None,
                extra_rise: None,
                noise: None,
                verdict: PlateauVerdict::Plateau,
                points_needed: None,
            },
            purge: None,
            gc: None,
            favors,
            supports: vec![format!("{name} supports")],
            note: None,
            points_needed: None,
            flat: false,
        };
        let picked = pick::Picked {
            tested: vec!["jvm.nonheap_used".into(), "jvm.nmt.symbol".into()],
            voting: vec!["jvm.nonheap_used".into()],
            primary: Some("jvm.nonheap_used".into()),
            required: vec!["jvm.nmt.symbol".into()],
            basis: String::new(),
            note: None,
            absent_site: None,
        };
        // The pools plateau and Symbol keeps rising: inconclusive, and the column is named.
        let results = [
            result("jvm.nonheap_used", true, Favors::NonLeak),
            result("jvm.nmt.symbol", false, Favors::Leak),
        ];
        let (favors, supports, held) = decide(&results, &picked);
        assert_eq!(favors, Favors::Inconclusive);
        assert!(supports.is_empty());
        assert_eq!(held, ["jvm.nmt.symbol"]);
        // Both plateau: the primary's verdict stands.
        let results = [
            result("jvm.nonheap_used", true, Favors::NonLeak),
            result("jvm.nmt.symbol", false, Favors::NonLeak),
        ];
        let (favors, supports, held) = decide(&results, &picked);
        assert_eq!(favors, Favors::NonLeak);
        assert_eq!(supports, ["jvm.nonheap_used supports"]);
        assert!(held.is_empty());
        // A leak of the primary is not held back by anything.
        let results = [
            result("jvm.nonheap_used", true, Favors::Leak),
            result("jvm.nmt.symbol", false, Favors::NonLeak),
        ];
        assert_eq!(decide(&results, &picked).0, Favors::Leak);
    }

    fn drift_result(
        name: &str,
        rise: f64,
        verdict: PlateauVerdict,
        favors: Favors,
    ) -> ColumnResult {
        ColumnResult {
            name: name.to_owned(),
            voting: true,
            plateau: Plateau {
                main_slope: None,
                extra_slope: None,
                extra_rise: Some(rise),
                noise: None,
                verdict,
                points_needed: None,
            },
            purge: None,
            gc: None,
            favors,
            supports: vec![],
            note: None,
            points_needed: None,
            flat: false,
        }
    }

    const CTX: DriftCtx = DriftCtx {
        extra: 20,
        command: false,
        junit: false,
        partial: false,
    };

    fn nonheap_picked(required: &[&str]) -> pick::Picked {
        pick::Picked {
            primary: Some("jvm.nonheap_used".into()),
            required: required.iter().map(|c| (*c).to_owned()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn a_small_non_heap_leak_verdict_on_a_flat_class_count_carries_the_agent_caution() {
        let result = |name: &str, rise: f64, verdict: PlateauVerdict| {
            drift_result(name, rise, verdict, Favors::Leak)
        };
        let picked = nonheap_picked(&[]);
        let flat = result("jvm.classes_loaded", 0.0, PlateauVerdict::Plateau);
        let small = [
            result("jvm.nonheap_used", 90_000.0, PlateauVerdict::Continues),
            flat,
        ];
        let Some(Drift::Agent(note)) = agent_drift(&small, &picked, Favors::Leak, "H1", CTX) else {
            panic!("an agent caution")
        };
        assert!(
            note.contains("88 KB")
                && note.contains("inconclusive")
                && note.contains("under 1024 KB")
                && !note.contains("well over")
                && note.trim_end().ends_with("decide it."),
            "{note}"
        );
        // A big rise, a rising class count or another verdict gets none.
        let big = [
            result("jvm.nonheap_used", 9e6, PlateauVerdict::Continues),
            result("jvm.classes_loaded", 0.0, PlateauVerdict::Plateau),
        ];
        assert!(agent_drift(&big, &picked, Favors::Leak, "H1", CTX).is_none());
        let classes = [
            result("jvm.nonheap_used", 90_000.0, PlateauVerdict::Continues),
            result("jvm.classes_loaded", 500.0, PlateauVerdict::Continues),
        ];
        assert!(matches!(
            agent_drift(&classes, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Classes(_))
        ));
        assert!(agent_drift(&small, &picked, Favors::Inconclusive, "H1", CTX).is_none());
        // Command mode has no checkpoint drift: a fresh JVM per point.
        let command = DriftCtx {
            command: true,
            ..CTX
        };
        assert!(agent_drift(&small, &picked, Favors::Leak, "H1", command).is_none());
    }

    #[test]
    fn the_drift_line_scales_with_the_extra_window() {
        assert_eq!(agent_drift_line(20), AGENT_DRIFT_RISE);
        assert_eq!(agent_drift_line(25), AGENT_DRIFT_RISE);
        // 20 KB per checkpoint with a 2x margin over 100 checkpoints.
        assert_eq!(agent_drift_line(100), 2.0 * 20.0 * 1024.0 * 100.0);
        // A rise of 1.5 MB is over the line at the default window and drift at extra 100.
        let result = |name: &str, rise: f64, verdict: PlateauVerdict| {
            drift_result(name, rise, verdict, Favors::Leak)
        };
        let rows = [
            result("jvm.nonheap_used", 1.5e6, PlateauVerdict::Continues),
            result("jvm.classes_loaded", 0.0, PlateauVerdict::Plateau),
        ];
        let picked = nonheap_picked(&[]);
        assert!(agent_drift(&rows, &picked, Favors::Leak, "H1", CTX).is_none());
        let wide = DriftCtx { extra: 100, ..CTX };
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", wide),
            Some(Drift::Agent(_))
        ));
    }

    #[test]
    fn a_rising_required_column_is_named_and_never_blamed_on_the_agent() {
        let rows = |col: &str| {
            [
                drift_result(
                    "jvm.nonheap_used",
                    350_000.0,
                    PlateauVerdict::Continues,
                    Favors::Leak,
                ),
                drift_result(
                    "jvm.classes_loaded",
                    0.0,
                    PlateauVerdict::Plateau,
                    Favors::NonLeak,
                ),
                drift_result(col, 40.0, PlateauVerdict::Continues, Favors::Leak),
            ]
        };
        // A thread leak: the verdict stands, and the thread capture decides.
        let picked = nonheap_picked(&["jvm.threads"]);
        let Some(Drift::Growing {
            limitation,
            next,
            inconclusive,
        }) = agent_drift(&rows("jvm.threads"), &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("the growing column is named")
        };
        assert!(!inconclusive);
        assert!(
            limitation.contains("jvm.threads") && !limitation.contains("agent's own checkpoints")
        );
        assert!(
            next.starts_with("dla harness run H1 --capture nonheap"),
            "{next}"
        );
        // An NMT category: inconclusive, and the category is named.
        let picked = nonheap_picked(&["jvm.nmt.symbol"]);
        let Some(Drift::Growing {
            limitation,
            next,
            inconclusive,
        }) = agent_drift(&rows("jvm.nmt.symbol"), &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("the growing column is named")
        };
        assert!(inconclusive && limitation.contains("jvm.nmt.symbol"));
        assert!(
            next.contains("nmt-diff.txt") && next.contains("jvm.nmt.symbol"),
            "{next}"
        );
        // The same required column at a plateau leaves the agent caution.
        let flat = [
            drift_result(
                "jvm.nonheap_used",
                350_000.0,
                PlateauVerdict::Continues,
                Favors::Leak,
            ),
            drift_result(
                "jvm.classes_loaded",
                0.0,
                PlateauVerdict::Plateau,
                Favors::NonLeak,
            ),
            drift_result(
                "jvm.nmt.symbol",
                0.0,
                PlateauVerdict::Plateau,
                Favors::NonLeak,
            ),
        ];
        assert!(matches!(
            agent_drift(&flat, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Agent(_))
        ));
    }

    #[test]
    fn a_class_count_with_a_net_rise_is_not_flat() {
        let picked = nonheap_picked(&[]);
        let rows = |class_rise: f64, class_verdict| {
            [
                drift_result(
                    "jvm.nonheap_used",
                    360_000.0,
                    PlateauVerdict::Continues,
                    Favors::Leak,
                ),
                drift_result(
                    "jvm.classes_loaded",
                    class_rise,
                    class_verdict,
                    Favors::Leak,
                ),
            ]
        };
        // A staircase (one hidden class every few checkpoints) reads inconclusive with a net
        // rise of 19 classes: a class leak, not the agent's drift.
        for n in [19.0, 12.0, 9.0] {
            let stair = rows(n, PlateauVerdict::Inconclusive);
            let Some(Drift::Classes(note)) = agent_drift(&stair, &picked, Favors::Leak, "H1", CTX)
            else {
                panic!("a net rise of {n} classes is no flat class count")
            };
            assert!(note.contains("jvm.classes_loaded"), "{note}");
        }
        // The limit itself (8 classes) is still jitter.
        let edge = rows(8.0, PlateauVerdict::Inconclusive);
        assert!(matches!(
            agent_drift(&edge, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Agent(_))
        ));
        // Jitter of a class or two on an idle driver is still flat.
        let jitter = rows(1.0, PlateauVerdict::Inconclusive);
        assert!(matches!(
            agent_drift(&jitter, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Agent(_))
        ));
    }

    #[test]
    fn a_required_column_that_only_could_not_be_judged_is_no_growth() {
        let rows = [
            drift_result(
                "jvm.nonheap_used",
                350_000.0,
                PlateauVerdict::Continues,
                Favors::Leak,
            ),
            drift_result(
                "jvm.classes_loaded",
                0.0,
                PlateauVerdict::Plateau,
                Favors::NonLeak,
            ),
            drift_result(
                "jvm.nmt.code",
                35_000.0,
                PlateauVerdict::Inconclusive,
                Favors::Inconclusive,
            ),
        ];
        let picked = nonheap_picked(&["jvm.nmt.code"]);
        // Not shown to grow: the agent caution, not a rising column that drops the verdict.
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Agent(_))
        ));
    }

    #[test]
    fn the_nmt_thread_category_rises_with_the_thread_count() {
        let rows = [
            drift_result(
                "jvm.nmt.thread",
                1e7,
                PlateauVerdict::Inconclusive,
                Favors::Leak,
            ),
            drift_result("jvm.threads", 40.0, PlateauVerdict::Continues, Favors::Leak),
            drift_result(
                "jvm.nmt.class",
                1e6,
                PlateauVerdict::Inconclusive,
                Favors::Leak,
            ),
        ];
        let names: Vec<String> = ["jvm.nmt.thread", "jvm.nmt.class", "jvm.threads"]
            .map(str::to_owned)
            .to_vec();
        let picked = nonheap_picked(&["jvm.nmt.thread", "jvm.nmt.class", "jvm.threads"]);
        let (rising, levelling) = split_rising(&rows, &picked, &names);
        assert_eq!(rising, ["jvm.nmt.thread", "jvm.threads"]);
        assert_eq!(levelling, ["jvm.nmt.class"]);
        // Without a rising thread count the category is judged on its own.
        let (rising, _) = split_rising(&rows[..1], &picked, &names[..1]);
        assert!(rising.is_empty());
    }

    #[test]
    fn a_thread_leak_with_its_nmt_category_points_at_the_thread_capture() {
        let rows = [
            drift_result(
                "jvm.nonheap_used",
                430_000.0,
                PlateauVerdict::Continues,
                Favors::Leak,
            ),
            drift_result(
                "jvm.classes_loaded",
                0.0,
                PlateauVerdict::Plateau,
                Favors::NonLeak,
            ),
            drift_result(
                "jvm.nmt.thread",
                4e5,
                PlateauVerdict::Continues,
                Favors::Leak,
            ),
            drift_result("jvm.threads", 40.0, PlateauVerdict::Continues, Favors::Leak),
            drift_result("os.threads", 40.0, PlateauVerdict::Continues, Favors::Leak),
        ];
        let picked = nonheap_picked(&["jvm.nmt.thread", "jvm.threads", "os.threads"]);
        let Some(Drift::Growing {
            limitation,
            next,
            inconclusive,
        }) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("the growing columns are named")
        };
        assert!(!inconclusive);
        assert!(
            next.contains("threads.json") && !next.contains("nmt-diff"),
            "{next}"
        );
        assert!(limitation.contains("keep rising"), "{limitation}");
        // Another NMT category beside the threads leaves the verdict open and is named too.
        let mut mixed = rows;
        mixed[2] = drift_result(
            "jvm.nmt.symbol",
            4e5,
            PlateauVerdict::Continues,
            Favors::Leak,
        );
        let picked = nonheap_picked(&["jvm.nmt.symbol", "jvm.threads", "os.threads"]);
        let Some(Drift::Growing {
            next, inconclusive, ..
        }) = agent_drift(&mixed, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("named")
        };
        assert!(inconclusive);
        assert!(
            next.contains("threads.json") && next.contains("nmt-diff.txt"),
            "{next}"
        );
    }

    #[test]
    fn a_column_that_only_could_not_be_judged_still_gets_the_longer_window() {
        let rows = [
            drift_result("jvm.threads", 40.0, PlateauVerdict::Continues, Favors::Leak),
            drift_result(
                "jvm.nmt.code",
                1.0,
                PlateauVerdict::Inconclusive,
                Favors::Inconclusive,
            ),
        ];
        let names = vec!["jvm.threads".to_owned(), "jvm.nmt.code".to_owned()];
        let picked = nonheap_picked(&["jvm.threads", "jvm.nmt.code"]);
        let (rising, levelling) = split_rising(&rows, &picked, &names);
        assert_eq!(rising, ["jvm.threads"]);
        assert_eq!(levelling, ["jvm.nmt.code"]);
    }

    #[test]
    fn a_held_back_thread_count_with_a_net_rise_is_rising_not_levelling() {
        // The plateau test cannot judge a count that grows at some checkpoints only; its net rise
        // of 10 threads says it keeps rising, so the capture decides it, not a longer window.
        let rows = [
            drift_result(
                "jvm.threads",
                10.0,
                PlateauVerdict::Inconclusive,
                Favors::Inconclusive,
            ),
            drift_result(
                "os.threads",
                2.0,
                PlateauVerdict::Inconclusive,
                Favors::Inconclusive,
            ),
        ];
        let names = vec!["jvm.threads".to_owned(), "os.threads".to_owned()];
        let picked = nonheap_picked(&["jvm.threads", "os.threads"]);
        let (rising, levelling) = split_rising(&rows, &picked, &names);
        assert_eq!(rising, ["jvm.threads"]);
        assert_eq!(levelling, ["os.threads"]);
    }

    #[test]
    fn the_nmt_thread_category_joins_only_a_thread_count_that_itself_rises() {
        // The thread counts plateau with a net rise within the jitter (2): they are no growth, so
        // the category that merely could not be judged does not rise with them.
        let rows = rows_with(
            &[
                ("jvm.threads", 2.0, PlateauVerdict::Plateau),
                ("os.threads", 2.0, PlateauVerdict::Plateau),
                ("jvm.nmt.thread", 2e5, PlateauVerdict::Inconclusive),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let picked = nonheap_picked(&["jvm.threads", "os.threads", "jvm.nmt.thread"]);
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Agent(_))
        ));
        let names = vec!["jvm.nmt.thread".to_owned()];
        let (rising, levelling) = split_rising(&rows, &picked, &names);
        assert!(rising.is_empty() && levelling.len() == 1);
        // A thread count that is not a required column (it failed the trend gate) is not
        // collected either, so it does not carry the category.
        let rows = rows_with(
            &[
                ("jvm.threads", 10.0, PlateauVerdict::Inconclusive),
                ("jvm.nmt.thread", 2e5, PlateauVerdict::Inconclusive),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let picked = nonheap_picked(&["jvm.nmt.thread"]);
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Agent(_))
        ));
    }

    #[test]
    fn a_thread_count_that_reads_plateau_with_a_net_rise_over_the_jitter_is_growth() {
        // Threads added in bursts near the end of each window read plateau (the fitted rise is
        // 0), but their net rise over the extra window is 6 threads: a thread leak, and the
        // category that tracks them rises with them.
        let picked = nonheap_picked(&["jvm.threads", "os.threads", "jvm.nmt.thread"]);
        let rows = rows_with(
            &[
                ("jvm.threads", 6.0, PlateauVerdict::Plateau),
                ("os.threads", 6.0, PlateauVerdict::Plateau),
                ("jvm.nmt.thread", 2e5, PlateauVerdict::Inconclusive),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let Some(Drift::Growing {
            limitation,
            inconclusive,
            ..
        }) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("a net rise of 6 threads is growth, whatever the plateau reads")
        };
        assert!(!inconclusive);
        assert!(
            limitation.contains("jvm.threads") && limitation.contains("jvm.nmt.thread"),
            "{limitation}"
        );
    }

    #[test]
    fn a_class_leak_beside_a_rising_nmt_category_keeps_its_verdict_and_the_ledger_commands() {
        let picked = nonheap_picked(&["jvm.nmt.metaspace"]);
        let rows = rows_with(
            &[("jvm.nmt.metaspace", 1.7e5, PlateauVerdict::Continues)],
            Some((20.0, PlateauVerdict::Continues)),
        );
        let Some(Drift::Growing {
            limitation,
            next,
            inconclusive,
        }) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("the category is named")
        };
        assert!(!inconclusive, "a rising class count keeps the verdict");
        assert!(
            limitation.contains("jvm.nmt.metaspace")
                && limitation.contains("jvm.classes_loaded is not flat either: it keeps rising")
                && limitation.contains("the verdict stands"),
            "{limitation}"
        );
        assert!(next.contains("nmt-diff.txt"), "{next}");
        assert!(keeps_ledger_commands(inconclusive, false));
        // A flat class count leaves the verdict open, as before.
        let flat = rows_with(
            &[("jvm.nmt.metaspace", 1.7e5, PlateauVerdict::Continues)],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        assert!(matches!(
            agent_drift(&flat, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Growing {
                inconclusive: true,
                ..
            })
        ));
    }

    #[test]
    fn a_class_column_that_could_not_be_judged_is_unknown_not_flat() {
        let picked = nonheap_picked(&[]);
        let rows = rows_with(&[], Some((0.0, PlateauVerdict::NotRun)));
        let Some(Drift::Agent(note)) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX) else {
            panic!("the agent-drift reading")
        };
        assert!(note.contains("could not judge it"), "{note}");
        assert!(!note.contains("with a flat class count"), "{note}");
        // A plateau reads flat, with no caution.
        let rows = rows_with(&[], Some((0.0, PlateauVerdict::Plateau)));
        let Some(Drift::Agent(note)) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX) else {
            panic!("the agent-drift reading")
        };
        assert!(note.contains("with a flat class count"), "{note}");
    }

    #[test]
    fn a_partial_run_keeps_no_ledger_command_after_the_thread_capture() {
        assert!(keeps_ledger_commands(false, false));
        assert!(!keeps_ledger_commands(true, false));
        assert!(!keeps_ledger_commands(false, true));
    }

    #[test]
    fn the_drift_next_keeps_the_junit_mode() {
        assert!(drift_next(CTX).contains("--driver --step <larger>"));
        let junit = DriftCtx { junit: true, ..CTX };
        let n = drift_next(junit);
        assert!(
            n.contains("--junit") && n.contains("--step <larger>") && !n.contains("--driver"),
            "{n}"
        );
    }

    #[test]
    fn a_partial_run_never_favours_either_side() {
        let said = vec!["x".to_owned()];
        for f in [Favors::Leak, Favors::NonLeak] {
            let (got, supports) = cap_for_partial(&said, f, vec!["plateau".to_owned()]);
            assert_eq!(got, Favors::Inconclusive);
            assert!(supports.is_empty());
        }
        let (got, supports) = cap_for_partial(&[], Favors::NonLeak, vec!["plateau".to_owned()]);
        assert_eq!(got, Favors::NonLeak);
        assert_eq!(supports, ["plateau"]);
    }

    fn rows_with(
        extra: &[(&str, f64, PlateauVerdict)],
        class: Option<(f64, PlateauVerdict)>,
    ) -> Vec<ColumnResult> {
        let mut rows = vec![drift_result(
            "jvm.nonheap_used",
            380_000.0,
            PlateauVerdict::Continues,
            Favors::Leak,
        )];
        if let Some((rise, verdict)) = class {
            rows.push(drift_result(
                "jvm.classes_loaded",
                rise,
                verdict,
                Favors::Leak,
            ));
        }
        for (name, rise, verdict) in extra {
            let favors = if *verdict == PlateauVerdict::Plateau {
                Favors::NonLeak
            } else {
                Favors::Inconclusive
            };
            rows.push(drift_result(name, *rise, *verdict, favors));
        }
        rows
    }

    #[test]
    fn the_thread_limitation_reads_as_one_sentence_whatever_the_run() {
        let picked = nonheap_picked(&["jvm.threads", "os.threads"]);
        let rows = rows_with(
            &[
                ("jvm.threads", 10.0, PlateauVerdict::Inconclusive),
                ("os.threads", 10.0, PlateauVerdict::Inconclusive),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        for (partial, wants) in [
            (
                false,
                "no checkpoint work; the verdict stands, and the thread capture decides",
            ),
            (
                true,
                "no checkpoint work; a partial run is inconclusive whatever the columns show, and the thread capture decides",
            ),
        ] {
            let ctx = DriftCtx { partial, ..CTX };
            let Some(Drift::Growing { limitation, .. }) =
                agent_drift(&rows, &picked, Favors::Leak, "H1", ctx)
            else {
                panic!("growth")
            };
            assert!(limitation.contains(wants), "{limitation}");
        }
        // os.threads that rises beside an NMT thread category that only follows it: the
        // limitation does not say "alone" about a list that names two columns.
        let picked = nonheap_picked(&["jvm.threads", "os.threads", "jvm.nmt.thread"]);
        let rows = rows_with(
            &[
                ("jvm.threads", 0.0, PlateauVerdict::Plateau),
                ("os.threads", 0.0, PlateauVerdict::Continues),
                ("jvm.nmt.thread", 0.0, PlateauVerdict::Inconclusive),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let Some(Drift::Growing { limitation, .. }) =
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("growth")
        };
        assert!(!limitation.contains("alone"), "{limitation}");
    }

    #[test]
    fn a_thread_count_that_reads_inconclusive_with_a_net_rise_is_growth_not_the_agents_drift() {
        let picked = nonheap_picked(&["jvm.threads", "os.threads"]);
        // A thread started every 10th unit: the plateau test cannot judge it (not every delta is
        // a rise), but the net rise over the extra window is 10 threads.
        let rows = rows_with(
            &[
                ("jvm.threads", 10.0, PlateauVerdict::Inconclusive),
                ("os.threads", 10.0, PlateauVerdict::Inconclusive),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let Some(Drift::Growing {
            limitation,
            next,
            inconclusive,
        }) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("a net rise of 10 threads is growth")
        };
        assert!(!inconclusive, "a thread leak keeps its verdict");
        assert!(
            limitation.contains("jvm.threads") && !limitation.contains("os.threads"),
            "os.threads is not judged by its net rise: {limitation}"
        );
        assert!(next.contains("--capture nonheap"), "{next}");
        // The slowest leak measured (6 threads) is growth too.
        let slow = rows_with(
            &[("jvm.threads", 6.0, PlateauVerdict::Inconclusive)],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let picked = nonheap_picked(&["jvm.threads"]);
        assert!(matches!(
            agent_drift(&slow, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Growing { .. })
        ));
    }

    #[test]
    fn a_thread_rise_within_the_jitter_of_an_idle_driver_is_the_agents_drift() {
        let picked = nonheap_picked(&["jvm.threads", "os.threads"]);
        for n in [0.0, 1.0, 2.0] {
            let rows = rows_with(
                &[
                    ("jvm.threads", 0.0, PlateauVerdict::Plateau),
                    ("os.threads", n, PlateauVerdict::Inconclusive),
                ],
                Some((0.0, PlateauVerdict::Plateau)),
            );
            assert!(
                matches!(
                    agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
                    Some(Drift::Agent(_))
                ),
                "a net rise of {n} threads is the compiler threads' jitter"
            );
        }
        // A net rise of 3 threads of `jvm.threads`, one more than the highest rise that is still
        // jitter, is growth.
        let rows = rows_with(
            &[("jvm.threads", 3.0, PlateauVerdict::Inconclusive)],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Growing { .. })
        ));
        // `os.threads` is not judged by its net rise: the JVM's own GC worker threads moved it by
        // 4 (series 38, 43, 42) on a driver that starts no thread, beside a flat `jvm.threads`.
        let rows = rows_with(
            &[
                ("jvm.threads", 0.0, PlateauVerdict::Plateau),
                ("os.threads", 4.0, PlateauVerdict::Plateau),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Agent(_))
        ));
    }

    #[test]
    fn os_threads_that_rises_alone_is_named_with_the_capture_and_stays_inconclusive() {
        let picked = nonheap_picked(&["jvm.threads", "os.threads"]);
        let rows = rows_with(
            &[
                ("jvm.threads", 0.0, PlateauVerdict::Plateau),
                ("os.threads", 40.0, PlateauVerdict::Continues),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let Some(Drift::Growing {
            limitation,
            next,
            inconclusive,
        }) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("os.threads that keeps rising is named")
        };
        assert!(
            inconclusive,
            "os.threads alone does not keep a leak verdict"
        );
        assert!(limitation.contains("os.threads"), "{limitation}");
        assert!(next.contains("--capture nonheap"), "{next}");
        // With `jvm.threads` rising too the verdict stands.
        let rows = rows_with(
            &[
                ("jvm.threads", 40.0, PlateauVerdict::Continues),
                ("os.threads", 40.0, PlateauVerdict::Continues),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Growing {
                inconclusive: false,
                ..
            })
        ));
    }

    #[test]
    fn a_thread_count_that_plateaus_is_no_open_column_and_one_that_rises_is() {
        let picked = nonheap_picked(&["jvm.threads", "os.threads"]);
        // Both thread counts at a plateau (rise 0): the pools' own reading is what is open, no
        // thread count is "not judged yet".
        let rows = rows_with(
            &[
                ("jvm.threads", 0.0, PlateauVerdict::Plateau),
                ("os.threads", 0.0, PlateauVerdict::Plateau),
            ],
            None,
        );
        assert!(open_required(&rows, &picked).is_empty());
        // A plateau that hides a net rise of 6 threads is open, and rising, not levelling.
        let rows = rows_with(
            &[
                ("jvm.threads", 6.0, PlateauVerdict::Plateau),
                ("os.threads", 0.0, PlateauVerdict::Plateau),
            ],
            None,
        );
        let open = open_required(&rows, &picked);
        assert_eq!(open, ["jvm.threads"]);
        let (rising, levelling) = split_rising(&rows, &picked, &open);
        assert_eq!((rising.len(), levelling.len()), (1, 0));
        // One that could not be judged and does not rise is open and levelling.
        let rows = rows_with(&[("jvm.threads", 1.0, PlateauVerdict::Inconclusive)], None);
        let open = open_required(&rows, &picked);
        let (rising, levelling) = split_rising(&rows, &picked, &open);
        assert_eq!((rising.len(), levelling.len()), (0, 1));
    }

    #[test]
    fn a_thread_count_that_rises_while_the_pools_plateau_holds_the_plateau_back() {
        // Six daemon threads started late: both pools and both counts read plateau, `jvm.threads`
        // rose 6 and `os.threads` 4. The plateau of the pools is held back, the result
        // inconclusive, and the count that rose is named (not refuted).
        let picked = nonheap_picked(&["jvm.threads", "os.threads"]);
        let mut picked = picked;
        picked.voting = vec!["jvm.nonheap_used".into()];
        let rows = [
            drift_result(
                "jvm.nonheap_used",
                1e5,
                PlateauVerdict::Plateau,
                Favors::NonLeak,
            ),
            drift_result("jvm.threads", 6.0, PlateauVerdict::Plateau, Favors::NonLeak),
            drift_result("os.threads", 4.0, PlateauVerdict::Plateau, Favors::NonLeak),
        ];
        let (favors, supports, held) = decide(&rows, &picked);
        assert_eq!(favors, Favors::Inconclusive);
        assert!(supports.is_empty());
        assert_eq!(held, ["jvm.threads"]);
        // GC worker threads alone (`os.threads` rose 4, `jvm.threads` 0) hold nothing back.
        let rows = [
            drift_result(
                "jvm.nonheap_used",
                1e5,
                PlateauVerdict::Plateau,
                Favors::NonLeak,
            ),
            drift_result("jvm.threads", 0.0, PlateauVerdict::Plateau, Favors::NonLeak),
            drift_result("os.threads", 4.0, PlateauVerdict::Plateau, Favors::NonLeak),
        ];
        let (favors, _, held) = decide(&rows, &picked);
        assert_eq!(favors, Favors::NonLeak);
        assert!(held.is_empty());
    }

    #[test]
    fn an_nmt_byte_category_rises_only_when_its_plateau_reads_continues() {
        let picked = nonheap_picked(&["jvm.nmt.code"]);
        // Bytes jitter: a large net rise that reads inconclusive is no growth shown.
        let rows = rows_with(
            &[("jvm.nmt.code", 5e6, PlateauVerdict::Inconclusive)],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Agent(_))
        ));
        let rows = rows_with(
            &[("jvm.nmt.code", 5e5, PlateauVerdict::Continues)],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        assert!(matches!(
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX),
            Some(Drift::Growing {
                inconclusive: true,
                ..
            })
        ));
        // The thread category rises with a thread count that rises by its net rise.
        let picked = nonheap_picked(&["jvm.nmt.thread", "jvm.threads"]);
        let rows = rows_with(
            &[
                ("jvm.nmt.thread", 1e6, PlateauVerdict::Inconclusive),
                ("jvm.threads", 10.0, PlateauVerdict::Inconclusive),
            ],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let Some(Drift::Growing { limitation, .. }) =
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("named")
        };
        assert!(limitation.contains("jvm.nmt.thread"), "{limitation}");
    }

    #[test]
    fn classes_and_threads_both_rising_name_both_and_keep_the_thread_capture() {
        let picked = nonheap_picked(&["jvm.nmt.thread", "jvm.threads", "os.threads"]);
        let rows = rows_with(
            &[
                ("jvm.nmt.thread", 1e7, PlateauVerdict::Continues),
                ("jvm.threads", 100.0, PlateauVerdict::Continues),
                ("os.threads", 100.0, PlateauVerdict::Continues),
            ],
            Some((10.0, PlateauVerdict::Inconclusive)),
        );
        let Some(Drift::Growing {
            limitation,
            next,
            inconclusive,
        }) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("both are named")
        };
        assert!(!inconclusive);
        assert!(
            limitation.contains("jvm.threads")
                && limitation.contains("os.threads")
                && limitation.contains("jvm.classes_loaded is not flat either: it rose 10 classes"),
            "{limitation}"
        );
        assert!(next.contains("threads.json"), "{next}");
        // A class count that is flat is not named.
        let flat = rows_with(
            &[("jvm.threads", 100.0, PlateauVerdict::Continues)],
            Some((0.0, PlateauVerdict::Plateau)),
        );
        let picked = nonheap_picked(&["jvm.threads"]);
        let Some(Drift::Growing { limitation, .. }) =
            agent_drift(&flat, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("named")
        };
        assert!(!limitation.contains("jvm.classes_loaded"), "{limitation}");
    }

    #[test]
    fn the_class_note_states_true_numbers() {
        let picked = nonheap_picked(&[]);
        let note = |class: Option<(f64, PlateauVerdict)>, ctx: DriftCtx| match agent_drift(
            &rows_with(&[], class),
            &picked,
            Favors::Leak,
            "H1",
            ctx,
        ) {
            Some(Drift::Classes(n)) => n,
            _ => panic!("a class note"),
        };
        // Reads continues: it keeps rising, by how much, with no jitter comparison.
        let n = note(Some((3.0, PlateauVerdict::Continues)), CTX);
        assert!(
            n.contains("keeps rising (3 classes over the extra window)") && !n.contains("jitter"),
            "{n}"
        );
        // Inconclusive over the jitter: the rise and the jitter it is over.
        let n = note(Some((9.0, PlateauVerdict::Inconclusive)), CTX);
        assert!(
            n.contains(
                "rose 9 classes over the extra window, over the 8 of an idle driver's jitter"
            ) && n.contains("the verdict stands"),
            "{n}"
        );
        // A partial run never says the verdict stands.
        let partial = DriftCtx {
            partial: true,
            ..CTX
        };
        let n = note(Some((9.0, PlateauVerdict::Inconclusive)), partial);
        assert!(
            !n.contains("verdict stands") && n.contains("partial run is inconclusive"),
            "{n}"
        );
    }

    #[test]
    fn a_result_without_the_class_column_is_read_with_a_caution_not_a_made_up_class_note() {
        let picked = nonheap_picked(&["jvm.threads"]);
        // Nothing rises: the agent-drift reading, saying the class count was not recorded.
        let rows = rows_with(&[("jvm.threads", 0.0, PlateauVerdict::Plateau)], None);
        let Some(Drift::Agent(note)) = agent_drift(&rows, &picked, Favors::Leak, "H1", CTX) else {
            panic!("the agent-drift reading")
        };
        assert!(
            note.contains("jvm.classes_loaded was not recorded")
                && !note.contains("rose 0 classes"),
            "{note}"
        );
        // The rising check still runs without it.
        let rows = rows_with(&[("jvm.threads", 10.0, PlateauVerdict::Inconclusive)], None);
        let Some(Drift::Growing { limitation, .. }) =
            agent_drift(&rows, &picked, Favors::Leak, "H1", CTX)
        else {
            panic!("the thread rise is named")
        };
        assert!(!limitation.contains("jvm.classes_loaded"), "{limitation}");
    }

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
    fn with_nothing_grown_the_runtime_heap_decides_an_unbound_test() {
        let mut s = series(&[]);
        s.columns[1] = col("go.heap_live");
        let ledger = Ledger::from_text("").unwrap();
        let trend = TrendReport {
            targets: Vec::new(),
            schema_version: TREND_SCHEMA_VERSION,
            id: "T1".to_owned(),
            series: "S1".to_owned(),
            thresholds: GateThresholds::default(),
            columns: vec![],
        };
        let p = pick::pick(&s, &ledger, None, &[], Some(&trend), "H1").unwrap();
        assert_eq!(p.voting, ["go.heap_live"], "{}", p.basis);
        assert!(
            p.tested.iter().any(|c| c == "os.rss"),
            "os.rss is still reported"
        );
        // Without a trend there is nothing to say no column grew: os.rss votes.
        let p = pick::pick(&s, &ledger, None, &[], None, "H1").unwrap();
        assert!(p.voting.iter().any(|c| c == "os.rss"));
    }

    #[test]
    fn a_peak_that_passed_the_gate_is_reported_but_does_not_vote() {
        let mut s = series(&[]);
        s.columns[1] = col("go.heap_live");
        s.columns.push(col("os.vm_hwm"));
        for p in &mut s.points {
            p.values.push(Some(4e7 + p.x * 10.0));
        }
        let ledger = Ledger::from_text("").unwrap();
        let passed = |name: &str| ColumnTrend {
            name: name.to_owned(),
            unit: Unit::Bytes,
            kind: ColumnKind::Gauge,
            region: SeriesRegion::Process,
            site: None,
            finding: None,
            candidates: vec![],
            n: 12,
            mk: None,
            sen: None,
            halves: None,
            every_delta: None,
            noise: 1.0,
            rise: 1e5,
            shape: dla_stats::TrendShape::Steady,
            gate: GateVerdict::Pass,
            gate_failed: vec![],
        };
        let trend = TrendReport {
            targets: Vec::new(),
            schema_version: TREND_SCHEMA_VERSION,
            id: "T1".to_owned(),
            series: "S1".to_owned(),
            thresholds: GateThresholds::default(),
            columns: vec![passed("os.vm_hwm")],
        };
        // Only the peak passed: as if nothing grew, the runtime heap decides.
        let p = pick::pick(&s, &ledger, None, &[], Some(&trend), "H1").unwrap();
        assert_eq!(p.voting, ["go.heap_live"], "{}", p.basis);
        assert!(p.tested.iter().any(|c| c == "os.vm_hwm"), "{:?}", p.tested);
        // Virtual size is the same: reservations are not given back by a purge.
        s.columns.push(col("os.vm_data"));
        for p in &mut s.points {
            p.values.push(Some(9e7 + p.x * 10.0));
        }
        let trend = TrendReport {
            targets: Vec::new(),
            columns: vec![passed("os.vm_hwm"), passed("os.vm_data")],
            ..trend
        };
        let p = pick::pick(&s, &ledger, None, &[], Some(&trend), "H1").unwrap();
        assert_eq!(p.voting, ["go.heap_live"], "{}", p.basis);
        assert!(p.tested.iter().any(|c| c == "os.vm_data"), "{:?}", p.tested);
    }

    /// A series with one purge event whose detail is `detail` (a recorded node purge record).
    fn purged(detail: &str) -> SeriesFile {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("node.heap_used"), col("node.timers")];
        for (i, p) in s.points.iter_mut().enumerate() {
            // node.heap_used grows 1 MB a step above a 60 MB warm-up level
            p.values = vec![
                Some(150e6),
                Some(60e6 + 1e6 * i as f64),
                Some(10.0 * i as f64),
            ];
        }
        s.events.push(dla_core::series::SeriesEvent {
            after_point: 13,
            kind: "purge:gc".to_owned(),
            detail: Some(detail.to_owned()),
        });
        s
    }

    fn purge_verdict(s: &SeriesFile, name: &str) -> PurgeVerdict {
        let ev = purge_event(s).unwrap();
        let mut lim = Vec::new();
        let r = test_column(s, name, true, false, 6, Some(&ev), &mut lim).unwrap();
        r.purge.unwrap().verdict
    }

    /// The Go shape of a flat run that happened to pass the first trend: `go.heap_objects`
    /// 164520 B at the end of the warm-up, 17 KB more at the last point (live objects), a
    /// purge that gives nothing back (`after` is `after_rise` above the last point).
    fn go_flat(after_rise: f64) -> SeriesFile {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("go.heap_objects")];
        for (i, p) in s.points.iter_mut().enumerate() {
            // 164520 at the end of the warm-up (point 1), a slow climb of 17312, then flat
            let v = 164_520.0 + (17_312.0 * i.saturating_sub(1) as f64 / 12.0).min(17_312.0);
            p.values = vec![Some(500e6), Some(v)];
        }
        let last = s.points.last().unwrap().values[1].unwrap();
        s.events.push(dla_core::series::SeriesEvent {
            after_point: 13,
            kind: "purge:free-os-memory".to_owned(),
            detail: Some(format!(
                r#"{{"kind":"free-os-memory","columns":true,"before":{{"go.heap_objects":{last}}},"after":{{"go.heap_objects":{}}}}}"#,
                last + after_rise
            )),
        });
        s
    }

    #[test]
    fn a_flat_go_column_with_a_few_kb_of_live_growth_reads_non_leak_not_retained() {
        let s = go_flat(200.0);
        let ev = purge_event(&s).unwrap();
        let mut lim = Vec::new();
        let r = test_column(&s, "go.heap_objects", true, false, 6, Some(&ev), &mut lim).unwrap();
        assert_eq!(
            r.plateau.verdict,
            PlateauVerdict::Plateau,
            "{:?}",
            r.plateau
        );
        let purge = r.purge.unwrap();
        assert!(purge.growth.unwrap() > 10_000.0 && purge.growth.unwrap() < 32.0 * 1024.0);
        assert_eq!(purge.verdict, PurgeVerdict::NoGrowth, "{purge:?}");
        assert_eq!(r.favors, Favors::NonLeak, "{:?}", r.supports);
    }

    /// A Go series whose exit-time heap is flat and whose in-run heap column (no purge entry, no
    /// `.pre_gc` twin) rises by `step` per point.
    fn go_in_run(step: f64) -> SeriesFile {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("go.heap_live.in_run")];
        s.columns[1].region = SeriesRegion::Heap;
        s.columns[1].after_gc = true;
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![Some(500e6), Some(8e6 + step * i as f64)];
        }
        s
    }

    #[test]
    fn an_in_run_column_has_no_purge_and_no_gc_reading_and_is_judged_by_its_own_shape() {
        let mut lim = Vec::new();
        let r = test_column(
            &go_in_run(120_000.0),
            "go.heap_live.in_run",
            true,
            false,
            6,
            None,
            &mut lim,
        )
        .unwrap();
        assert!(r.purge.is_none() && r.gc.is_none(), "{r:?}");
        assert_eq!(r.favors, Favors::Leak, "{:?} {:?}", r.plateau, r.supports);
        let r = test_column(
            &go_in_run(0.0),
            "go.heap_live.in_run",
            true,
            false,
            6,
            None,
            &mut lim,
        )
        .unwrap();
        assert!(r.purge.is_none() && r.gc.is_none(), "{r:?}");
        assert_eq!(
            r.favors,
            Favors::NonLeak,
            "{:?} {:?}",
            r.plateau,
            r.supports
        );
    }

    fn gate_passed(name: &str) -> ColumnTrend {
        ColumnTrend {
            name: name.to_owned(),
            unit: Unit::Bytes,
            kind: ColumnKind::Gauge,
            region: SeriesRegion::Process,
            site: None,
            finding: None,
            candidates: vec![],
            n: 12,
            mk: None,
            sen: None,
            halves: None,
            every_delta: None,
            noise: 1.0,
            rise: 1e5,
            shape: dla_stats::TrendShape::Steady,
            gate: GateVerdict::Pass,
            gate_failed: vec![],
        }
    }

    /// An unbound alloc-test over `s` with a trim purge whose `after` is `before` (nothing
    /// given back) and a first trend that passed `passed`.
    fn unbound_run(
        mut s: SeriesFile,
        passed: &[&str],
    ) -> (Favors, Vec<ColumnResult>, pick::Picked) {
        let last = s.points.last().unwrap().values.clone();
        let named: Vec<String> = s
            .columns
            .iter()
            .zip(&last)
            .map(|(c, v)| format!("\"{}\":{}", c.name, v.unwrap()))
            .collect();
        let both = format!("{{{}}}", named.join(","));
        s.events.push(dla_core::series::SeriesEvent {
            after_point: 13,
            kind: "purge:trim".to_owned(),
            detail: Some(format!(
                r#"{{"kind":"trim","columns":true,"before":{both},"after":{both}}}"#
            )),
        });
        let trend = TrendReport {
            targets: Vec::new(),
            schema_version: TREND_SCHEMA_VERSION,
            id: "T1".to_owned(),
            series: "S1".to_owned(),
            thresholds: GateThresholds::default(),
            columns: passed.iter().map(|n| gate_passed(n)).collect(),
        };
        let ledger = Ledger::from_text("").unwrap();
        let picked = pick::pick(&s, &ledger, None, &[], Some(&trend), "H1").unwrap();
        let ev = purge_event(&s).unwrap();
        let mut lim = Vec::new();
        let results: Vec<ColumnResult> = picked
            .tested
            .iter()
            .filter_map(|n| {
                test_column(
                    &s,
                    n,
                    picked.voting.contains(n),
                    false,
                    6,
                    Some(&ev),
                    &mut lim,
                )
            })
            .collect();
        let (favors, _, _) = decide(&results, &picked);
        (favors, results, picked)
    }

    #[test]
    fn growing_file_backed_memory_beside_growing_rss_reads_leak_and_names_both() {
        // Both columns vote: a leak that maps files grows os.rss_file and os.rss, and the
        // result lists both. The file-backed column grows by more than 1% of the process over
        // the extra window, so its plateau test calls the growth.
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("os.rss_file")];
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![Some(500e6 + 2e6 * i as f64), Some(10e6 + 2e6 * i as f64)];
        }
        let (favors, results, picked) = unbound_run(s, &["os.rss", "os.rss_file"]);
        assert_eq!(picked.voting, ["os.rss", "os.rss_file"], "{}", picked.basis);
        assert_eq!(favors, Favors::Leak);
        // `os.rss_anon` is not sampled here, so `os.rss` keeps its own reading and carries the
        // leak. The file-backed column grows (its plateau says so) but never decides a leak:
        // its own reading is inconclusive and says why.
        let rss = results.iter().find(|r| r.name == "os.rss").unwrap();
        assert_eq!(rss.favors, Favors::Leak, "{:?}", rss.plateau);
        let file = results.iter().find(|r| r.name == "os.rss_file").unwrap();
        assert_eq!(file.plateau.verdict, PlateauVerdict::Continues, "{file:?}");
        assert_eq!(file.favors, Favors::Inconclusive, "{file:?}");
        assert!(file.note.is_some(), "{file:?}");
    }

    type PartsRun = (Favors, Vec<ColumnResult>);

    /// `os.rss`, `os.rss_anon` and `os.rss_file` over the series: each `(start, step per point)`.
    fn rss_parts_run(rss: (f64, f64), anon: (f64, f64), file: (f64, f64)) -> PartsRun {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("os.rss_anon"), col("os.rss_file")];
        for (i, p) in s.points.iter_mut().enumerate() {
            let at = |(a, b): (f64, f64)| Some(a + b * i as f64);
            p.values = vec![at(rss), at(anon), at(file)];
        }
        // Only a column that rises passes the trend's gate.
        let passed: Vec<&str> = [
            ("os.rss", rss),
            ("os.rss_anon", anon),
            ("os.rss_file", file),
        ]
        .into_iter()
        .filter(|(_, (_, step))| *step != 0.0)
        .map(|(n, _)| n)
        .collect();
        let (favors, results, _) = unbound_run(s, &passed);
        (favors, results)
    }

    fn favors_of(results: &[ColumnResult], name: &str) -> Favors {
        results.iter().find(|r| r.name == name).unwrap().favors
    }

    #[test]
    fn a_flat_anon_with_rising_file_pages_and_the_rss_they_explain_is_not_a_leak() {
        // The CI case: the anonymous part is flat, the file-backed part rises and `os.rss`
        // rises by the same amount. Neither file column decides a leak.
        let (favors, results) = rss_parts_run((500e6, 2e6), (490e6, 0.0), (10e6, 2e6));
        assert_ne!(favors, Favors::Leak, "{results:?}");
        assert_ne!(favors_of(&results, "os.rss"), Favors::Leak);
        assert_ne!(favors_of(&results, "os.rss_file"), Favors::Leak);
        let rss = results.iter().find(|r| r.name == "os.rss").unwrap();
        assert!(
            rss.note
                .as_deref()
                .is_some_and(|n| n.contains("file-backed"))
        );
    }

    #[test]
    fn a_rising_anon_with_flat_file_pages_is_a_leak() {
        let (favors, results) = rss_parts_run((500e6, 2e6), (490e6, 2e6), (10e6, 0.0));
        assert_eq!(favors, Favors::Leak, "{results:?}");
        assert_eq!(favors_of(&results, "os.rss_anon"), Favors::Leak);
        assert_eq!(favors_of(&results, "os.rss"), Favors::Leak);
    }

    #[test]
    fn a_rising_anon_beside_rising_file_pages_is_a_leak_decided_by_anon_and_rss() {
        let (favors, results) = rss_parts_run((500e6, 4e6), (490e6, 2e6), (10e6, 2e6));
        assert_eq!(favors, Favors::Leak, "{results:?}");
        assert_eq!(favors_of(&results, "os.rss_anon"), Favors::Leak);
        assert_eq!(favors_of(&results, "os.rss"), Favors::Leak);
        assert_ne!(favors_of(&results, "os.rss_file"), Favors::Leak);
    }

    #[test]
    fn a_slow_file_backed_rise_the_plateau_cannot_call_is_inconclusive_beside_a_leaking_rss() {
        // 1.5 MB over the extra window is under 1% of the 500 MB process: the plateau test
        // calls it inconclusive, and the purge (it gives file pages back to nobody) no longer
        // makes a leak of it (it did, before the purge was left out of such a column). Both
        // columns vote and are named; `os.rss` carries the leak verdict.
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("os.rss_file")];
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![
                Some(500e6 + 2e6 * i as f64),
                Some(10e6 + 300_000.0 * i as f64),
            ];
        }
        let (favors, results, picked) = unbound_run(s, &["os.rss", "os.rss_file"]);
        assert_eq!(picked.voting, ["os.rss", "os.rss_file"], "{}", picked.basis);
        let file = results.iter().find(|r| r.name == "os.rss_file").unwrap();
        assert_eq!(
            file.plateau.verdict,
            PlateauVerdict::Inconclusive,
            "{file:?}"
        );
        assert_eq!(file.favors, Favors::Inconclusive, "{file:?}");
        let rss = results.iter().find(|r| r.name == "os.rss").unwrap();
        assert_eq!(rss.favors, Favors::Leak, "{rss:?}");
        assert_eq!(favors, Favors::Leak);
    }

    /// A stair of 200 KB early in the window, flat after it, on `column`, with a purge that
    /// gives nothing back.
    fn stair_run(column: &str) -> (Favors, ColumnResult) {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col(column)];
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![
                Some(500e6),
                Some(10e6 + if i >= 4 { 200_000.0 } else { 0.0 }),
            ];
        }
        let (favors, mut results, _) = unbound_run(s, &[column]);
        let at = results.iter().position(|r| r.name == column).unwrap();
        (favors, results.swap_remove(at))
    }

    #[test]
    fn a_flat_file_backed_column_with_a_retained_purge_reads_non_leak_and_keeps_the_reading() {
        // No purge gives file pages back, so the retained reading of the purge (the step may
        // have mapped files itself) does not contradict the plateau; the reading stays.
        let (favors, file) = stair_run("os.rss_file");
        assert_eq!(file.plateau.verdict, PlateauVerdict::Plateau, "{file:?}");
        assert_eq!(
            file.purge.as_ref().unwrap().verdict,
            PurgeVerdict::Retained,
            "{file:?}"
        );
        assert_eq!(file.favors, Favors::NonLeak, "{file:?}");
        assert_eq!(favors, Favors::NonLeak);
    }

    /// A flat column that scatters by about 70 KB from reading to reading; the point at the
    /// end of the warm-up is one of the low ones, the last reading stands `growth` above it and
    /// the purge gives nothing back.
    fn scattered_run(scatter: f64, growth: f64) -> ColumnResult {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("os.rss_anon")];
        let pattern = [
            0.0, 0.0, 0.3, 0.8, 0.2, 0.9, 0.4, 1.0, 0.1, 0.7, 0.5, 0.9, 0.3, 0.8,
        ];
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![Some(500e6), Some(23e6 + scatter * pattern[i])];
        }
        s.points.last_mut().unwrap().values[1] = Some(23e6 + growth);
        let (_, mut results, _) = unbound_run(s, &["os.rss_anon"]);
        let at = results
            .iter()
            .position(|r| r.name == "os.rss_anon")
            .unwrap();
        results.swap_remove(at)
    }

    #[test]
    fn a_growth_inside_the_columns_own_scatter_but_above_its_minimum_is_still_growth_for_the_purge()
    {
        // A slow real leak under the scatter of a noisy column (about 80 KB): the last reading
        // is 40 KB above the warm-up point, just above the 32 KiB minimum and well inside the
        // scatter the plateau test reads, so a limit that scales with the scatter would read it
        // as no growth. The purge gave nothing back, so the reading is retained and, against
        // the plateau, inconclusive. It must never read non-leak (a bound non-leak refutes).
        let r = scattered_run(80_000.0, 40_000.0);
        assert_eq!(r.plateau.verdict, PlateauVerdict::Plateau, "{r:?}");
        let purge = r.purge.as_ref().unwrap();
        assert!(purge.growth.unwrap() > judge::MIN_BYTE_RISE, "{purge:?}");
        assert_eq!(purge.verdict, PurgeVerdict::Retained, "{r:?}");
        assert_eq!(r.favors, Favors::Inconclusive, "{r:?}");
        // A step far above the scatter reads the same.
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("os.rss_anon")];
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![
                Some(500e6),
                Some(23e6 + if i >= 4 { 2e6 } else { 0.0 } + 1000.0 * (i % 3) as f64),
            ];
        }
        let (_, results, _) = unbound_run(s, &["os.rss_anon"]);
        let r = results.iter().find(|r| r.name == "os.rss_anon").unwrap();
        assert_eq!(
            r.purge.as_ref().unwrap().verdict,
            PurgeVerdict::Retained,
            "{r:?}"
        );
    }

    #[test]
    fn a_retained_purge_does_not_contradict_the_plateau_of_a_column_no_purge_can_release() {
        // `go.total` of a Go run that passed the first trend on the step of the heap growing
        // into new arenas (a stair of 200 KB early in the window, flat after it): the runtime
        // maps memory and never unmaps it, and the purge returns nothing of it. Its plateau
        // stands; the reading stays. The same series under an ordinary column name is
        // inconclusive (the test below).
        for name in ["go.total", "os.vm_data", "os.vm_hwm"] {
            let (favors, r) = stair_run(name);
            assert_eq!(r.plateau.verdict, PlateauVerdict::Plateau, "{name}: {r:?}");
            assert_eq!(
                r.purge.as_ref().unwrap().verdict,
                PurgeVerdict::Retained,
                "{name}: {r:?}"
            );
            assert_eq!(r.favors, Favors::NonLeak, "{name}: {r:?}");
            if name == "go.total" {
                assert_eq!(favors, Favors::NonLeak, "{name}");
            }
        }
    }

    #[test]
    fn growth_that_continues_in_a_column_no_purge_can_release_still_reads_leak() {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("go.total")];
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![Some(500e6), Some(10e6 + 400_000.0 * i as f64)];
        }
        let (_, results, _) = unbound_run(s, &["go.total"]);
        let r = results.iter().find(|r| r.name == "go.total").unwrap();
        assert_eq!(r.plateau.verdict, PlateauVerdict::Continues, "{r:?}");
        assert_eq!(r.favors, Favors::Leak, "{r:?}");
    }

    /// `os.rss_anon` on a flat 500 MB process: 2 warm-up points at 10 MB, 6 main points
    /// `main` and the extra points `extra`, a purge that gives nothing back.
    fn held_run(main: [f64; 6], extra: &[f64]) -> ColumnResult {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("os.rss_anon")];
        let ys: Vec<f64> = [10e6, 10e6]
            .into_iter()
            .chain(main)
            .chain(extra.iter().copied())
            .collect();
        s.points = ys
            .iter()
            .enumerate()
            .map(|(i, &y)| Point {
                x: 1000.0 * (i as f64 + 1.0),
                t_ms: i as u64,
                values: vec![Some(500e6), Some(y)],
            })
            .collect();
        let (_, mut results, _) = unbound_run(s, &["os.rss_anon"]);
        let at = results
            .iter()
            .position(|r| r.name == "os.rss_anon")
            .unwrap();
        results.swap_remove(at)
    }

    #[test]
    fn a_ramp_then_flat_with_a_retained_purge_reads_non_leak_and_says_the_growth_is_held() {
        let r = held_run(
            [12e6, 14e6, 16e6, 18e6, 20e6, 22e6],
            &[22e6, 22.001e6, 22e6, 22.001e6, 22e6, 22.001e6],
        );
        assert_eq!(r.plateau.verdict, PlateauVerdict::Plateau, "{r:?}");
        assert_eq!(
            r.purge.as_ref().unwrap().verdict,
            PurgeVerdict::Retained,
            "{r:?}"
        );
        assert_eq!(r.favors, Favors::NonLeak, "{r:?}");
        assert_eq!(r.supports, ["N-WARM", "N-HWM"]);
        let note = r.note.as_deref().unwrap();
        assert!(
            note.contains("held memory, not allocator or collector headroom"),
            "{note}"
        );
        assert!(note.contains("rose 6 time(s)"), "{note}");
    }

    #[test]
    fn a_stair_that_rose_twice_reads_inconclusive_until_the_extra_window_doubles() {
        let main = [10e6, 10e6, 14e6, 14e6, 18e6, 18e6];
        let flat = |n: usize| -> Vec<f64> { (0..n).map(|i| 18e6 + 1e3 * (i % 2) as f64).collect() };
        // Two rises over a main window as long as the extra window: two are expected, three
        // are needed.
        let r = held_run(main, &flat(6));
        assert_eq!(r.plateau.verdict, PlateauVerdict::Plateau, "{r:?}");
        assert_eq!(r.favors, Favors::Inconclusive, "{r:?}");
        let n = r.points_needed.unwrap();
        assert_eq!(n, 9, "{r:?}");
        let note = r.note.as_deref().unwrap();
        assert!(note.contains("9 points"), "{note}");
        // The doubled window expects 4.4 and the same flat series reads non-leak.
        let r = held_run(main, &flat(12));
        assert_eq!(r.favors, Favors::NonLeak, "{r:?}");
        // A single step needs far more (16 points): the number says so.
        let r = held_run([10e6, 10e6, 10e6, 18e6, 18e6, 18e6], &flat(6));
        assert_eq!(r.favors, Favors::Inconclusive, "{r:?}");
        assert_eq!(r.points_needed, Some(16), "{r:?}");
    }

    /// `native.live_bytes` of a `Vec` that doubles its capacity beside a resident size that
    /// keeps growing: 2 warm-up points, 6 main points in two steps and the extra points `extra`.
    fn doubling_run(extra: &[f64]) -> (Favors, Vec<ColumnResult>) {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("native.live_bytes")];
        s.columns[1].region = SeriesRegion::Heap;
        let live: Vec<f64> = [10., 10., 10., 10., 70., 70., 130., 130.]
            .into_iter()
            .chain(extra.iter().copied())
            .map(|v| v * 1e6)
            .collect();
        s.points = live
            .iter()
            .enumerate()
            .map(|(i, &y)| Point {
                x: 440.0 * (i as f64 + 1.0),
                t_ms: i as u64,
                values: vec![Some(500e6 + 9e6 * i as f64), Some(y)],
            })
            .collect();
        let (favors, results, _) = unbound_run(s, &["native.live_bytes", "os.rss"]);
        (favors, results)
    }

    fn named<'a>(results: &'a [ColumnResult], name: &str) -> &'a ColumnResult {
        results.iter().find(|r| r.name == name).unwrap()
    }

    #[test]
    fn a_column_that_grows_in_steps_with_the_resident_size_growing_no_longer_favors_non_leak() {
        // The measured case: the allocator shim's live bytes (a `Vec` doubling) moved in steps,
        // its extra window held a new step but rose under three of the noise its steps make, and
        // the column read `plateau` and decided non-leak while `os.rss` kept growing.
        let mut extra = vec![130.0; 6];
        extra.extend([184.0; 6]);
        let (favors, results) = doubling_run(&extra);
        let (live, rss) = (
            named(&results, "native.live_bytes"),
            named(&results, "os.rss"),
        );
        assert_eq!(rss.plateau.verdict, PlateauVerdict::Continues, "{rss:?}");
        assert_eq!(live.plateau.verdict, PlateauVerdict::Continues, "{live:?}");
        assert_ne!(live.favors, Favors::NonLeak, "{live:?}");
        assert_ne!(favors, Favors::NonLeak, "{favors:?}");
        // An extra window no longer than the gap between two steps cannot tell a plateau from a
        // step that has not come yet: inconclusive, and the window that covers the gap is named.
        let (favors, results) = doubling_run(&[100.0, 115.0, 130.0]);
        let live = named(&results, "native.live_bytes");
        assert_eq!(
            live.plateau.verdict,
            PlateauVerdict::Inconclusive,
            "{live:?}"
        );
        // (The purge kept the growth here, which on an undecided plateau reads leak.)
        assert_ne!(live.favors, Favors::NonLeak, "{live:?}");
        assert!(live.points_needed.is_some_and(|n| n > 3), "{live:?}");
        let note = live.note.as_deref().unwrap();
        assert!(note.contains("extra window of"), "{note}");
        assert_ne!(favors, Favors::NonLeak, "{favors:?}");
        // The same column that stays flat for longer than a step gap is a plateau, as before.
        let (_, results) = doubling_run(&[130.0; 12]);
        let live = named(&results, "native.live_bytes");
        assert_eq!(live.plateau.verdict, PlateauVerdict::Plateau, "{live:?}");
    }

    /// The measured `Vec` column as a series: 2 warm-up points, the main window and the `extra`
    /// points after it (440 apart), beside a flat resident size.
    fn measured_vec_run(extra: &[f64]) -> (Favors, Vec<ColumnResult>, pick::Picked) {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("native.live_bytes")];
        s.columns[1].region = SeriesRegion::Heap;
        let live: Vec<f64> = [
            2_663_109.,
            3_646_149.,
            5_612_229.,
            5_612_229.,
            9_544_389.,
            9_544_389.,
            9_544_389.,
            17_408_709.,
        ]
        .into_iter()
        .chain(extra.iter().copied())
        .collect();
        s.points = live
            .iter()
            .enumerate()
            .map(|(i, &y)| Point {
                x: 440.0 * (i as f64 + 1.0),
                t_ms: i as u64,
                values: vec![Some(500e6), Some(y)],
            })
            .collect();
        // No purge ran (as in the measured run): the column is judged on its plateau alone.
        let trend = TrendReport {
            targets: Vec::new(),
            schema_version: TREND_SCHEMA_VERSION,
            id: "T1".to_owned(),
            series: "S1".to_owned(),
            thresholds: GateThresholds::default(),
            columns: vec![gate_passed("native.live_bytes")],
        };
        let ledger = Ledger::from_text("").unwrap();
        let picked = pick::pick(&s, &ledger, None, &[], Some(&trend), "H1").unwrap();
        let mut lim = Vec::new();
        let results: Vec<ColumnResult> = picked
            .tested
            .iter()
            .filter_map(|n| test_column(&s, n, picked.voting.contains(n), false, 6, None, &mut lim))
            .collect();
        let (favors, _, _) = decide(&results, &picked);
        (favors, results, picked)
    }

    #[test]
    fn a_staircase_cut_short_is_lengthened_to_the_window_that_covers_its_expected_gap() {
        // Main points end at x = 3520 (the step of 17.4 MB), the extra window starts at 3960:
        // flat until 5720, 2200 after the last step, where the gaps call for 3520.
        let flat = [17_408_709.0; 5];
        let (favors, results, picked) = measured_vec_run(&flat);
        let live = named(&results, "native.live_bytes");
        assert_eq!(
            live.plateau.verdict,
            PlateauVerdict::Inconclusive,
            "{live:?}"
        );
        assert_eq!(favors, Favors::Inconclusive);
        // 3960 + 440 * (n - 1) is past 3520 + 3520 = 7040.
        let n = live.points_needed.unwrap();
        assert_eq!(n, 9, "{live:?}");
        assert!(3960.0 + 440.0 * f64::from(n - 1) > 7040.0);
        // The alloc-test lengthens its window to that many points when the time fits, and the
        // printed `--extra` says it when it does not.
        assert_eq!(wider_window(&results, &picked, &[], 5, 40), Some(n));
        // The evidence records it, per column and for the result (what the ledger reads to keep the
        // finding open while the budget can pay for the window).
        assert_eq!(live.to_json()["points_needed"], json!(n));
        assert_eq!(deciding_need(&results, &picked), Some(n));
        let next = say_how_long_a_held_plateau_needs(
            vec!["dla alloc-test H1 --finding F1 --extra 6".to_owned()],
            favors,
            &results,
            &picked,
        );
        assert!(
            next[0].starts_with(&format!("dla alloc-test H1 --finding F1 --extra {n}   (")),
            "{next:?}"
        );
        assert!(next[0].contains("expected"), "{next:?}");
        // One that already covers it is left alone.
        let next = say_how_long_a_held_plateau_needs(
            vec!["dla alloc-test H1 --finding F1 --extra 20".to_owned()],
            favors,
            &results,
            &picked,
        );
        assert!(next[0].contains("--extra 20   ("), "{next:?}");
        // Run to that many points the step is in the window, and the column grows.
        let mut full = flat.to_vec();
        full.extend([17_408_709.0, 17_408_709.0, 33_137_349.0, 33_137_525.0]);
        let (_, results, _) = measured_vec_run(&full);
        let live = named(&results, "native.live_bytes");
        assert_eq!(live.plateau.verdict, PlateauVerdict::Continues, "{live:?}");
    }

    #[test]
    fn continuing_growth_with_a_retained_purge_still_reads_leak() {
        let r = held_run(
            [12e6, 14e6, 16e6, 18e6, 20e6, 22e6],
            &[24e6, 26e6, 28e6, 30e6, 32e6, 34e6],
        );
        assert_eq!(r.plateau.verdict, PlateauVerdict::Continues, "{r:?}");
        assert_eq!(r.favors, Favors::Leak, "{r:?}");
    }

    #[test]
    fn a_retained_purge_still_contradicts_the_plateau_of_an_ordinary_column() {
        let (favors, anon) = stair_run("os.rss_anon");
        assert_eq!(anon.plateau.verdict, PlateauVerdict::Plateau, "{anon:?}");
        assert_eq!(anon.favors, Favors::Inconclusive, "{anon:?}");
        assert_eq!(favors, Favors::Inconclusive);
    }

    /// `column` on a 500 MB process: the end of the warm-up at 10 MB, then a main window
    /// rising 100 KB a point and an extra window rising 150 KB a point (a rise far under 1% of
    /// the process, kept from the main window with confidence: the plateau test calls it
    /// inconclusive), with a purge that gives nothing back.
    fn slowly_rising_run(column: &str) -> (Favors, ColumnResult) {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col(column)];
        for (i, p) in s.points.iter_mut().enumerate() {
            let y = if i < 2 {
                10e6
            } else if i < 8 {
                10e6 + 100e3 * (i - 1) as f64
            } else {
                10.6e6 + 150e3 * (i - 7) as f64
            };
            p.values = vec![Some(500e6), Some(y)];
        }
        let (favors, mut results, _) = unbound_run(s, &[column]);
        let at = results.iter().position(|r| r.name == column).unwrap();
        (favors, results.swap_remove(at))
    }

    #[test]
    fn a_retained_purge_does_not_make_a_leak_of_an_undecided_plateau_of_file_backed_memory() {
        // The purge reading is kept (retained, marked as not applying), and the column is
        // judged on its plateau alone: inconclusive, not a leak.
        for name in ["os.rss_file", "os.vm_hwm", "os.vm_data"] {
            let (favors, r) = slowly_rising_run(name);
            assert_eq!(
                r.plateau.verdict,
                PlateauVerdict::Inconclusive,
                "{name}: {r:?}"
            );
            let purge = r.purge.as_ref().unwrap();
            assert_eq!(purge.verdict, PurgeVerdict::Retained, "{name}: {r:?}");
            assert!(!purge.applies, "{name}: {r:?}");
            assert_eq!(r.favors, Favors::Inconclusive, "{name}: {r:?}");
            assert_ne!(favors, Favors::Leak, "{name}");
        }
        // The same series of an ordinary column, whose purge does apply, is a leak as before.
        let (favors, r) = slowly_rising_run("os.rss_anon");
        assert_eq!(r.plateau.verdict, PlateauVerdict::Inconclusive, "{r:?}");
        assert!(r.purge.as_ref().unwrap().applies, "{r:?}");
        assert_eq!(r.favors, Favors::Leak, "{r:?}");
        assert_ne!(favors, Favors::NonLeak);
    }

    #[test]
    fn a_window_that_is_too_short_alone_is_lengthened_to_the_points_that_would_decide() {
        let main = [10e6, 10e6, 14e6, 14e6, 18e6, 18e6];
        let flat = |n: usize| -> Vec<f64> { (0..n).map(|i| 18e6 + 1e3 * (i % 2) as f64).collect() };
        let run = |extra: usize| {
            let mut s = series(&[]);
            s.columns = vec![col("os.rss"), col("os.rss_anon")];
            let ys: Vec<f64> = [10e6, 10e6]
                .into_iter()
                .chain(main)
                .chain(flat(extra))
                .collect();
            s.points = ys
                .iter()
                .enumerate()
                .map(|(i, &y)| Point {
                    x: 1000.0 * (i as f64 + 1.0),
                    t_ms: i as u64,
                    values: vec![Some(500e6), Some(y)],
                })
                .collect();
            unbound_run(s, &["os.rss_anon"])
        };
        // Two rises over a window as long as the extra one: nine points would be needed.
        let (favors, results, picked) = run(6);
        assert_eq!(favors, Favors::Inconclusive);
        assert_eq!(wider_window(&results, &picked, &[], 6, 48), Some(9));
        // Not for a run the deadline cut short, nor when the window is long enough already.
        assert_eq!(
            wider_window(&results, &picked, &["p7".to_owned()], 6, 48),
            None
        );
        let (favors, results, picked) = run(12);
        assert_eq!(favors, Favors::NonLeak);
        assert_eq!(wider_window(&results, &picked, &[], 12, 96), None);
        // A window that would have to be over MAX_WIDEN times as long is left to a person.
        let (_, results, picked) = run(6);
        assert_eq!(wider_window(&results, &picked, &[], 6, 8), None);
        // A leak that goes on is no question of length.
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("os.rss_anon")];
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![Some(500e6 + 2e6 * i as f64), Some(10e6 + 2e6 * i as f64)];
        }
        let (favors, results, picked) = unbound_run(s, &["os.rss_anon"]);
        assert_eq!(favors, Favors::Leak);
        assert_eq!(wider_window(&results, &picked, &[], 6, 48), None);
    }

    #[test]
    fn a_node_gc_purge_record_with_the_series_own_reading_as_before_is_retained() {
        // The last point's heap_used is 73 MB. Recorded after the fix: `before` is that
        // post-GC reading, `after` follows one more collection.
        let kept = purged(
            r#"{"kind":"gc","before":{"node.heap_used":73000000.0,"os.rss":150000000.0},
                "after":{"node.heap_used":72900000.0,"os.rss":150000000.0}}"#,
        );
        assert_eq!(
            purge_verdict(&kept, "node.heap_used"),
            PurgeVerdict::Retained
        );
        // The pre-fix record took `before` before the point's forced collections, so the
        // garbage that collection frees read as released growth. That record is released.
        let old = purged(
            r#"{"kind":"gc","before":{"node.heap_used":88700000.0,"os.rss":153000000.0},
                "after":{"node.heap_used":72600000.0,"os.rss":153000000.0}}"#,
        );
        assert_eq!(
            purge_verdict(&old, "node.heap_used"),
            PurgeVerdict::Released
        );
    }

    /// `node.heap_used` over 2 warm-up, 6 main and 6 extra points (`ys`, 14 values), judged as
    /// the deciding column of a finding (`bound`) or of an unbound alloc-test, with the purge
    /// record `purge` (the readings before and after the purge) when given.
    fn heap_column(ys: [f64; 14], bound: bool, purge: Option<(f64, f64)>) -> ColumnResult {
        let mut s = series(&[]);
        s.columns = vec![col("os.rss"), col("node.heap_used")];
        for (i, p) in s.points.iter_mut().enumerate() {
            p.values = vec![Some(150e6), Some(ys[i])];
        }
        let event = purge.map(|(before, after)| {
            s.events.push(dla_core::series::SeriesEvent {
                after_point: 13,
                kind: "purge:gc".to_owned(),
                detail: Some(format!(
                    r#"{{"kind":"gc","before":{{"node.heap_used":{before}}},"after":{{"node.heap_used":{after}}}}}"#
                )),
            });
            purge_event(&s).unwrap()
        });
        test_column(
            &s,
            "node.heap_used",
            true,
            bound,
            6,
            event.as_ref(),
            &mut Vec::new(),
        )
        .unwrap()
    }

    fn ramp(from: f64, step: f64, n: usize, start: f64) -> Vec<f64> {
        (0..n).map(|i| from + step * (start + i as f64)).collect()
    }

    fn heap_ys(main_step: f64, extra_step: f64) -> [f64; 14] {
        let mut v = vec![60e6, 60e6];
        v.extend(ramp(60e6, main_step, 6, 1.0));
        let top = 60e6 + main_step * 6.0;
        v.extend(ramp(top, extra_step, 6, 1.0));
        v.try_into().unwrap()
    }

    #[test]
    fn a_column_that_never_grew_in_the_main_window_cannot_speak_for_a_bound_finding() {
        let flat = heap_ys(0.0, 0.0);
        let r = heap_column(flat, true, None);
        assert_eq!(r.plateau.verdict, PlateauVerdict::NeverGrew, "{r:?}");
        assert_eq!(r.plateau.verdict.as_str(), "never-grew");
        assert_eq!(
            (r.favors, r.supports.len()),
            (Favors::Inconclusive, 0),
            "{r:?}"
        );
        // An alloc-test with no finding is unchanged: the flat column is a plateau.
        let r = heap_column(flat, false, None);
        assert_eq!(r.plateau.verdict, PlateauVerdict::Plateau, "{r:?}");
        assert_eq!(r.favors, Favors::NonLeak, "{r:?}");
        // A purge that "released" 28 MB of a column that did not grow is no way to non-leak.
        let r = heap_column(flat, true, Some((88e6, 60e6)));
        assert_eq!(r.favors, Favors::Inconclusive, "{r:?}");
        assert!(r.supports.is_empty(), "{r:?}");
    }

    #[test]
    fn growth_in_the_main_window_then_a_flat_extra_window_is_still_a_plateau_and_non_leak() {
        let r = heap_column(heap_ys(1e6, 0.0), true, None);
        assert_eq!(r.plateau.verdict, PlateauVerdict::Plateau, "{r:?}");
        assert_eq!(r.favors, Favors::NonLeak, "{r:?}");
        assert!(r.supports.iter().any(|x| x == "N-WARM"), "{r:?}");
        let r = heap_column(heap_ys(1e6, 1e6), true, None);
        assert_eq!(r.plateau.verdict, PlateauVerdict::Continues, "{r:?}");
        assert_eq!(r.favors, Favors::Leak, "{r:?}");
    }

    #[test]
    fn a_purge_record_without_the_columns_key_abstains() {
        let rec = purged(
            r#"{"kind":"gc","before":{"node.heap_used":73000000.0},
                "after":{"node.heap_used":72900000.0}}"#,
        );
        assert_eq!(purge_verdict(&rec, "os.rss"), PurgeVerdict::NotRun);
        let ev = purge_event(&rec).unwrap();
        let mut lim = Vec::new();
        let r = test_column(&rec, "os.rss", true, false, 6, Some(&ev), &mut lim).unwrap();
        // a missing reading is not "released": no purge support is claimed
        assert!(
            !r.supports
                .iter()
                .any(|x| x == "N-HEADROOM" || x == "N-ALLOC"),
            "{:?}",
            r.supports
        );
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
        let r = test_column(&s, "probe:Leak", true, false, 6, None, &mut lim).unwrap();
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
        let r = test_column(&s, "probe:Leak", true, false, 6, None, &mut lim).unwrap();
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
        let r = test_column(&s, "probe:Leak", true, false, 6, None, &mut lim).unwrap();
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
        let r = test_column(&s, "probe:Leak", true, false, 6, None, &mut lim).unwrap();
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
    fn go_field_manifest(finding: &str) -> HarnessManifest {
        serde_json::from_value(json!({
            "id": "H1", "runtime": "go", "mode": "driver", "scratch": "harness/H1/work",
            "run": {"argv": ["go", "test"], "cwd": "."}, "warmup": 2, "iterations": 6,
            "step": 100, "key_modes": ["fresh"], "agent": "driver-go", "timeout_secs": 300,
            "probes": [{"id": "P1", "finding": finding,
                "target": {"kind": "go-field", "package_dir": "health",
                           "type_name": "Server", "field": "statusMap"}}]
        }))
        .unwrap()
    }

    #[test]
    fn an_unread_size_probe_turns_non_leak_inconclusive_and_leaves_a_leak() {
        let (f, s) = cap_for_unread_probe(Favors::NonLeak, vec!["N-WARM".into()]);
        assert_eq!((f, s.len()), (Favors::Inconclusive, 0));
        let (f, s) = cap_for_unread_probe(Favors::Leak, vec!["leak".into()]);
        assert_eq!((f, s.len()), (Favors::Leak, 1));
        assert_eq!(
            cap_for_unread_probe(Favors::Inconclusive, vec![]).0,
            Favors::Inconclusive
        );
    }

    #[test]
    fn a_go_field_probe_without_readings_bars_a_non_leak_verdict_and_says_why() {
        let f = Finding::candidate(
            FindingId::new(3),
            dla_core::Origin::parse("residual").unwrap(),
            vec![],
        );
        let m = go_field_manifest("F3");
        // The series has no column for the probe: the note names it and the way out.
        let mut s = series(&[]);
        s.limitations.push(
            "probe health.Server.statusMap: no tracked object of type Server: call dlaTrack(<your Server>) in setup"
                .to_owned(),
        );
        let note = unread_field_probe(&m, &s, &f).unwrap();
        assert!(note.contains("P1 (health.Server.statusMap)"), "{note}");
        assert!(note.contains("no tracked object of type Server"), "{note}");
        assert!(note.contains("cannot favour non-leak"), "{note}");
        // With the column in the series, or for another finding, there is nothing to say.
        let mut read = series(&[]);
        read.columns[1] = col("probe:health.Server.statusMap");
        assert!(unread_field_probe(&m, &read, &f).is_none());
        assert!(unread_field_probe(&go_field_manifest("F9"), &s, &f).is_none());
    }

    fn test_loop_manifest(finding: &str, tools: serde_json::Value) -> HarnessManifest {
        serde_json::from_value(json!({
            "id": "H1", "runtime": "jvm", "mode": "driver", "scratch": "harness/H1/work",
            "run": {"argv": ["java", "x"], "cwd": "."}, "warmup": 2, "iterations": 6,
            "step": 2, "key_modes": ["fresh"], "agent": "driver-jvm", "timeout_secs": 300,
            "tools": tools,
            "probes": [{"id": "P1", "finding": finding,
                "target": {"kind": "jvm-static", "class": "org.acme.Cache", "field": "resolved"}}]
        }))
        .unwrap()
    }

    #[test]
    fn a_leak_on_a_harness_not_made_for_the_finding_says_so_and_a_driver_made_for_it_does_not() {
        use dla_core::grown::HarnessAbout;
        let f = FindingId::new(3);
        let loop_m = test_loop_manifest("F3", json!(["junit-platform-launcher"]));
        let hint = shared_hint_text(&loop_m, &HarnessAbout::OwnWorkload, &f).unwrap();
        assert!(
            hint.contains(
                "a confirm from H1 may count growth that another holder keeps (a test loop"
            ),
            "{hint}"
        );
        assert!(
            hint.contains("a driver made for F3 decides it alone: `dla harness gen --lang jvm --driver --probe F3`"),
            "{hint}"
        );
        // About it only through a probe: a hook driver, but not made for it.
        let hooks = test_loop_manifest("F3", json!([]));
        let probe = shared_hint_text(&hooks, &HarnessAbout::Probe, &f).unwrap();
        assert!(
            probe.contains("about it only through a holder probe"),
            "{probe}"
        );
        // Made for it: no hint.
        assert_eq!(shared_hint_text(&hooks, &HarnessAbout::Target, &f), None);
        assert_eq!(shared_hint_text(&hooks, &HarnessAbout::EntryPath, &f), None);
    }

    fn finding(n: u32) -> Finding {
        Finding::candidate(
            FindingId::new(n),
            dla_core::Origin::parse("residual").unwrap(),
            vec![],
        )
    }

    #[test]
    fn only_a_test_loop_with_a_jvm_static_probe_of_the_finding_is_attributed() {
        let loop_tools = json!(["junit-platform-launcher"]);
        let m = test_loop_manifest("F3", loop_tools.clone());
        let t = attribution_target(&m, Some(&finding(3))).unwrap();
        assert_eq!(
            (t.finding.as_str(), t.class.as_str(), t.field.as_str()),
            ("F3", "org.acme.Cache", "resolved")
        );
        // Another finding, no finding, and a harness that is not a test loop: nothing changes.
        assert!(attribution_target(&m, Some(&finding(4))).is_none());
        assert!(attribution_target(&m, None).is_none());
        let hooks = test_loop_manifest("F3", json!([]));
        assert!(attribution_target(&hooks, Some(&finding(3))).is_none());
        // A probe of another kind is not a holder the loop can read.
        let mut go = go_field_manifest("F3");
        go.tools = loop_tools
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_owned())
            .collect();
        assert!(attribution_target(&go, Some(&finding(3))).is_none());
    }

    #[test]
    fn the_printed_attribution_names_feeders_by_simple_name_largest_first_and_bounds_them() {
        let classes: Vec<serde_json::Value> = [json!({"class": "org.acme.Gone", "first_run": 1, "later_runs": 0, "dropped": true})]
            .into_iter()
            .chain((0..7).map(|i| json!({"class": format!("org.acme.p{i}.Feeds{i}Tests"), "first_run": 0, "later_runs": 20 * (i + 1)})))
            .chain((0..11).map(|i| json!({"class": format!("org.acme.Flat{i}Tests"), "first_run": 5, "later_runs": 0})))
            .collect();
        let doc = json!({"probe": "org.acme.Cache#resolved", "later_runs": 20, "classes": classes});
        let a = dla_core::testloop::Attributed::parse(&doc.to_string()).unwrap();
        let m = attribution_summary(&a);
        let fed = m["fed_by"].as_array().unwrap();
        assert_eq!(fed.len(), MAX_FED_NAMED);
        assert_eq!(fed[0], json!({"class": "Feeds6Tests", "per_run": 7.0}));
        assert_eq!(fed[1]["class"], "Feeds5Tests");
        assert_eq!(m["fed_by_more"], 3);
        let flat = m["ran_flat"].as_array().unwrap();
        // The class the loop dropped had no later run: it did not run flat.
        assert_eq!(flat.len(), MAX_FLAT_NAMED);
        assert_eq!(flat[0], "Flat0Tests");
        assert_eq!(m["ran_flat_more"], 3);
        assert!(serde_json::to_string(&m).unwrap().len() < MAX_RESULT_BYTES / 2);
        // Parallel tests: said so.
        let mut p = doc.clone();
        p["parallel"] = json!(true);
        let a = dla_core::testloop::Attributed::parse(&p.to_string()).unwrap();
        assert!(attribution_summary(&a).contains_key("parallel"));
    }
}
