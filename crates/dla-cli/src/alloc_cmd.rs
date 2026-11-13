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
    let min_rise = if series.columns[idx].unit == Unit::Bytes {
        judge::MIN_BYTE_RISE
    } else {
        0.0
    };
    let plateau = judge::plateau(main, extra, counter, reference, min_rise);
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
/// a non-leak verdict stands only when every required column favours non-leak too. Returns the
/// verdict, its supports and the required columns that held a non-leak verdict back.
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
        .map(|r| (r.name.as_str(), r.favors))
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
/// be called flat (jitter of a few classes, not a staircase).
const AGENT_DRIFT_CLASS_JITTER: f64 = 3.0;

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

/// Splits column names into those that keep rising (growth continues) and those that only could
/// not be judged yet. The thread category of NMT counts as rising when a thread count does: it
/// rises with the threads and is decided by the same capture.
fn split_rising(results: &[ColumnResult], names: &[String]) -> (Vec<String>, Vec<String>) {
    let continues = |n: &str| {
        results
            .iter()
            .any(|r| r.name == n && r.plateau.verdict == PlateauVerdict::Continues)
    };
    let threads_rise = continues("jvm.threads") || continues("os.threads");
    names
        .iter()
        .cloned()
        .partition(|n| continues(n) || (threads_rise && n == "jvm.nmt.thread"))
}

/// The required columns (NMT categories and thread counts) that are not shown to plateau.
fn rising_required<'a>(results: &'a [ColumnResult], picked: &pick::Picked) -> Vec<&'a str> {
    results
        .iter()
        .filter(|r| picked.required.contains(&r.name) && r.favors != Favors::NonLeak)
        .map(|r| r.name.as_str())
        .collect()
}

/// The reading of a leak verdict taken from the JVM's non-heap bytes alone. With the class count
/// flat and a rise under the drift line over the extra window, the checkpoints' own work
/// (compiled code and metadata of the agent) can make the figure rise, but only when no required
/// column (a `jvm.nmt.*` category or a thread count) rises itself: then it is that column that is
/// named. A command-mode harness (a fresh JVM per point) has no checkpoint drift.
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
    // Flat: at a plateau, or too jittery to judge with a net rise of a few classes at most. A
    // class count with a steady net rise (a staircase) is a class leak, however slow.
    let classes_flat = results
        .iter()
        .find(|r| r.name == "jvm.classes_loaded")
        .is_some_and(|r| match r.plateau.verdict {
            PlateauVerdict::Plateau => true,
            PlateauVerdict::Inconclusive => r
                .plateau
                .extra_rise
                .is_some_and(|n| n <= AGENT_DRIFT_CLASS_JITTER),
            _ => false,
        });
    let line = agent_drift_line(ctx.extra);
    if rise >= line {
        return None;
    }
    if !classes_flat {
        let class_rise = results
            .iter()
            .find(|r| r.name == "jvm.classes_loaded")
            .and_then(|r| r.plateau.extra_rise)
            .unwrap_or(0.0);
        return Some(Drift::Classes(format!(
            "jvm.classes_loaded rose {class_rise:.0} classes over the extra window (more than the {AGENT_DRIFT_CLASS_JITTER:.0} of an idle driver's jitter) while jvm.nonheap_used rose {:.0} KB: that is no flat class count, so the rise is not blamed on the agent's own drift and the verdict stands; the classes that load say what grows",
            rise / 1024.0
        )));
    }
    // Only the columns that keep rising: one that could not be judged yet is no growth shown.
    let names: Vec<String> = rising_required(results, picked)
        .into_iter()
        .map(str::to_owned)
        .collect();
    let (rising, _) = split_rising(results, &names);
    let rising: Vec<&str> = rising.iter().map(String::as_str).collect();
    if !rising.is_empty() {
        let threads = rising
            .iter()
            .any(|c| matches!(*c, "jvm.threads" | "os.threads"));
        // The thread category of NMT rises with the thread count; any other category is not
        // thread work, so it leaves the verdict open.
        let others = rising.iter().any(|c| {
            !matches!(*c, "jvm.threads" | "os.threads") && !(threads && *c == "jvm.nmt.thread")
        });
        return Some(Drift::Growing {
            limitation: format!(
                "{} {} rising over the extra window (jvm.nonheap_used itself rose {:.0} KB on a flat class count): that is not the agent's own drift, so it is not blamed on it; {}",
                rising.join(", "),
                if rising.len() == 1 { "keeps" } else { "keep" },
                rise / 1024.0,
                if !others {
                    "a thread count is no checkpoint work, so the verdict stands and the thread capture decides what holds the threads"
                } else {
                    "the result is inconclusive until the NMT category that rises is read"
                }
            ),
            next: deciding_capture(&rising, harness),
            inconclusive: others,
        });
    }
    Some(Drift::Agent(format!(
        "jvm.nonheap_used rose {:.0} KB over the extra window with a flat class count: that is under {:.0} KB, the line for what the agent's own checkpoints add by themselves ({:.0} KB per checkpoint measured on an idle driver, with a {:.0}x margin, over {} checkpoints; at least 1 MiB), and no jvm.nmt.* category or thread count rises, so a small JVM cannot decide this and the result is inconclusive, not a leak. Only a rise over that line, a growing jvm.nmt.* category (Symbol or Metaspace) or a harness that does more work per checkpoint ({}) can decide it.",
        rise / 1024.0,
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
    let partial: Vec<String> = out.partial.clone();
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

    // The verdict: the bound column's, or the voting columns together, with the required
    // columns able to hold a non-leak verdict back.
    let (favors, supports, held_back) = decide(&results, &picked);
    // A required column that keeps rising is decided by its capture, a longer run does not; one
    // that only could not be judged (still levelling off) may show its plateau in a longer window.
    let split_open = |names: &[String]| split_rising(&results, names);
    let primary_name = picked.primary.as_deref().unwrap_or("the deciding column");
    let (held_rising, held_levelling) = split_open(&held_back);
    if !held_rising.is_empty() {
        limitations.push(format!(
            "{} did not plateau: the plateau of {primary_name} is held back, so the result is inconclusive; a longer --extra does not decide a column that keeps rising: capture it (the thread capture for a thread count, the NMT category for jvm.nmt.*) and read that column's series",
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
        rising_required(&results, &picked)
            .into_iter()
            .map(str::to_owned)
            .collect()
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
        junit: manifest
            .tools
            .iter()
            .any(|t| t == "junit-platform-launcher"),
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
    // A run the deadline cut short measured less than it planned: whatever its columns
    // say, it never favours non-leak (and never confirms) on its own.
    let (favors, supports) = cap_for_partial(&partial, favors, supports);
    if !partial.is_empty() {
        limitations.push(
            "partial run: the run deadline stopped the points before the plan, so this result is inconclusive whatever the columns show"
                .to_owned(),
        );
    }

    // Files: the new trend and the full result.
    let trend = trend_of(&tc, &ledger, &series, previous.as_ref())?;
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

    let next = match &drift {
        Some(Drift::Agent(_)) => vec![drift_next(ctx)],
        Some(Drift::Growing {
            next, inconclusive, ..
        }) => {
            let mut n = vec![next.clone()];
            if !inconclusive {
                n.extend(next_commands(
                    &ledger,
                    bound.as_ref(),
                    &a.harness,
                    favors,
                    &supports,
                    &evidence,
                    extra,
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
        ),
    };
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
    if let Some(Drift::Agent(note)) = &drift {
        out.insert("caution".into(), json!(note));
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
            (g.passed_gate()
                && g.best_column_for(&root.id, root.site.as_deref(), root.owner.as_deref())
                    .is_some())
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
            },
            purge: None,
            gc: None,
            favors,
            supports: vec![format!("{name} supports")],
        };
        let picked = pick::Picked {
            tested: vec!["jvm.nonheap_used".into(), "jvm.nmt.symbol".into()],
            voting: vec!["jvm.nonheap_used".into()],
            primary: Some("jvm.nonheap_used".into()),
            required: vec!["jvm.nmt.symbol".into()],
            basis: String::new(),
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
            },
            purge: None,
            gc: None,
            favors,
            supports: vec![],
        }
    }

    const CTX: DriftCtx = DriftCtx {
        extra: 20,
        command: false,
        junit: false,
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
        for n in [19.0, 6.0, 4.0] {
            let stair = rows(n, PlateauVerdict::Inconclusive);
            let Some(Drift::Classes(note)) = agent_drift(&stair, &picked, Favors::Leak, "H1", CTX)
            else {
                panic!("a net rise of {n} classes is no flat class count")
            };
            assert!(note.contains("jvm.classes_loaded"), "{note}");
        }
        // The limit itself (3 classes) is still jitter.
        let edge = rows(3.0, PlateauVerdict::Inconclusive);
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
        let (rising, levelling) = split_rising(&rows, &names);
        assert_eq!(rising, ["jvm.nmt.thread", "jvm.threads"]);
        assert_eq!(levelling, ["jvm.nmt.class"]);
        // Without a rising thread count the category is judged on its own.
        let (rising, _) = split_rising(&rows[..1], &names[..1]);
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
        let (rising, levelling) = split_rising(&rows, &names);
        assert_eq!(rising, ["jvm.threads"]);
        assert_eq!(levelling, ["jvm.nmt.code"]);
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
        let r = test_column(s, name, true, 6, Some(&ev), &mut lim).unwrap();
        r.purge.unwrap().verdict
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

    #[test]
    fn a_purge_record_without_the_columns_key_abstains() {
        let rec = purged(
            r#"{"kind":"gc","before":{"node.heap_used":73000000.0},
                "after":{"node.heap_used":72900000.0}}"#,
        );
        assert_eq!(purge_verdict(&rec, "os.rss"), PurgeVerdict::NotRun);
        let ev = purge_event(&rec).unwrap();
        let mut lim = Vec::new();
        let r = test_column(&rec, "os.rss", true, 6, Some(&ev), &mut lim).unwrap();
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
