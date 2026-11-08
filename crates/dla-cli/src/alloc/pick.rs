//! Which columns `dla alloc-test` tests, and which of them decide the verdict.

use std::{fs, path::Path};

use dla_core::{
    Finding, FindingId, Ledger,
    error::codes,
    series::{SeriesFile, SeriesRegion, SeriesSourceKind, numbered},
    trend::{TREND_DIR, TrendReport},
};
use dla_stats::GateVerdict;

use crate::fail::{Failure, Res};

/// Columns whose purge reading is always wanted (and that decide nothing unless nothing
/// else can).
const SUPPORT: &[&str] = &["os.rss", "os.vm_hwm", "native.peak_bytes"];
/// Columns that count runtime activity, not memory: they rise with every run by design.
const ACTIVITY: &[&str] = &["py.gc_collections"];
/// Most tied columns tested for a bound finding.
const MAX_TIED: usize = 6;
/// Most gate-passing columns tested without a finding.
const MAX_PASSED: usize = 10;

/// The columns to test.
#[derive(Debug, Clone, Default)]
pub struct Picked {
    /// Every column tested, in the order they are reported.
    pub tested: Vec<String>,
    /// The columns whose verdicts decide the overall verdict.
    pub voting: Vec<String>,
    /// The single deciding column of a bound finding, when a tied column was found.
    pub primary: Option<String>,
    /// How the columns were chosen, in a few words.
    pub basis: String,
}

/// The newest trend report of a series of harness `harness` (a harness series first, then
/// any series of it).
pub fn latest_trend(run_dir: &Path, harness: &str) -> Option<TrendReport> {
    let mut ids: Vec<(u32, String)> = fs::read_dir(run_dir.join(TREND_DIR))
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(".json")?.to_owned();
            Some((numbered('T', &id)?, id))
        })
        .collect();
    ids.sort();
    let mut any = None;
    for (_, id) in ids.iter().rev() {
        let Ok(report) = TrendReport::load(run_dir, id) else {
            continue;
        };
        let Ok(series) = SeriesFile::load(run_dir, &report.series) else {
            continue;
        };
        if series.source.harness.as_deref() != Some(harness) {
            continue;
        }
        if series.source.kind == SeriesSourceKind::Harness {
            return Some(report);
        }
        any.get_or_insert(report);
    }
    any
}

/// The column a region is summarized by in `series`.
fn region_primary(series: &SeriesFile, region: Option<SeriesRegion>) -> &'static str {
    let lists: &[&str] = match region {
        Some(SeriesRegion::Heap) => &[
            "jvm.heap_used",
            "go.heap_live",
            "node.heap_used",
            "py.traced",
            "native.live_bytes",
        ],
        Some(SeriesRegion::OffHeap) => &["native.live_bytes", "gap.unexplained"],
        Some(SeriesRegion::NonHeap) => &["go.goroutines", "jvm.threads", "os.threads"],
        _ => &[],
    };
    lists
        .iter()
        .copied()
        .find(|n| series.column_index(n).is_some())
        .unwrap_or("os.rss")
}

fn add(v: &mut Vec<String>, name: &str) {
    if !v.iter().any(|x| x == name) {
        v.push(name.to_owned());
    }
}

/// Columns of the new series tied to `root` (its `finding`, then its site, then the
/// candidates of the latest trend), strongest tier first; passing columns of the
/// latest trend lead inside a tier.
fn tied(
    series: &SeriesFile,
    root: &FindingId,
    site: Option<&str>,
    trend: Option<&TrendReport>,
) -> (Vec<String>, &'static str) {
    let passes = |name: &str| {
        trend
            .and_then(|t| t.columns.iter().find(|c| c.name == name))
            .is_some_and(|c| c.gate == GateVerdict::Pass)
    };
    let order = |mut v: Vec<String>| {
        v.sort_by_key(|n| !passes(n));
        v.truncate(MAX_TIED);
        v
    };
    let by_finding: Vec<String> = series
        .columns
        .iter()
        .filter(|c| c.finding.as_ref() == Some(root))
        .map(|c| c.name.clone())
        .collect();
    if !by_finding.is_empty() {
        return (order(by_finding), "its probe column");
    }
    let by_site: Vec<String> = series
        .columns
        .iter()
        .filter(|c| site.is_some() && c.site.as_deref() == site)
        .map(|c| c.name.clone())
        .collect();
    if !by_site.is_empty() {
        return (order(by_site), "the columns at its site");
    }
    let by_candidate: Vec<String> = trend
        .into_iter()
        .flat_map(|t| t.columns.iter())
        .filter(|c| c.candidates.contains(root) || c.finding.as_ref() == Some(root))
        .filter(|c| series.column_index(&c.name).is_some())
        .map(|c| c.name.clone())
        .collect();
    if !by_candidate.is_empty() {
        return (order(by_candidate), "the columns the trend tied to it");
    }
    (Vec::new(), "")
}

/// Chooses the columns: `--column` names, else the columns tied to the finding (its
/// probe, its site group, its region's primary column), else every column that passed the
/// gate in the latest trend of the harness plus `os.rss`. `os.rss`, `os.vm_hwm` and
/// `native.peak_bytes` are always tested for the purge reading.
pub fn pick(
    series: &SeriesFile,
    ledger: &Ledger,
    finding: Option<&Finding>,
    explicit: &[String],
    trend: Option<&TrendReport>,
    harness: &str,
) -> Res<Picked> {
    let mut p = Picked::default();
    if !explicit.is_empty() {
        for name in explicit {
            if series.column_index(name).is_none() {
                let have: Vec<&str> = series
                    .columns
                    .iter()
                    .take(8)
                    .map(|c| c.name.as_str())
                    .collect();
                return Err(Failure::refuse(
                    codes::INVALID_VALUE,
                    format!(
                        "series {} of harness {harness} has no column {name:?}",
                        series.id
                    ),
                    "an existing --column name",
                    format!(
                        "dla alloc-test {harness} --column {}   (columns include: {})",
                        have.first().copied().unwrap_or("os.rss"),
                        have.join(", ")
                    ),
                ));
            }
            add(&mut p.voting, name);
        }
        p.basis = "named with --column".to_owned();
    } else if let Some(f) = finding {
        let root = f.root_id();
        let root_site = ledger
            .finding(&root)
            .and_then(|r| r.site.clone())
            .or_else(|| f.site.clone());
        let (cols, how) = tied(series, &root, root_site.as_deref(), trend);
        if let Some(first) = cols.first() {
            p.primary = Some(first.clone());
            p.voting.push(first.clone());
            for c in &cols {
                add(&mut p.tested, c);
            }
            p.basis = format!("{how} of {root}");
        } else {
            let region = f
                .region
                .or_else(|| ledger.finding(&root).and_then(|r| r.region))
                .map(|r| match r {
                    dla_core::finding::Region::Heap => SeriesRegion::Heap,
                    dla_core::finding::Region::OffHeap => SeriesRegion::OffHeap,
                    dla_core::finding::Region::NonHeap => SeriesRegion::NonHeap,
                });
            let name = region_primary(series, region);
            p.primary = Some(name.to_owned());
            p.voting.push(name.to_owned());
            p.basis = format!("the primary column of the region of {root}");
        }
    } else {
        let mut any_passed = false;
        if let Some(t) = trend {
            let mut passed: Vec<&dla_core::trend::ColumnTrend> = t
                .columns
                .iter()
                .filter(|c| c.gate == GateVerdict::Pass)
                .filter(|c| series.column_index(&c.name).is_some())
                .filter(|c| !ACTIVITY.contains(&c.name.as_str()))
                .collect();
            passed.sort_by(|a, b| {
                let key = |c: &dla_core::trend::ColumnTrend| {
                    (
                        c.site.is_none() && c.candidates.is_empty() && c.finding.is_none(),
                        c.mk.map_or(f64::INFINITY, |m| m.p_increasing),
                    )
                };
                let (ka, kb) = (key(a), key(b));
                ka.0.cmp(&kb.0).then(ka.1.total_cmp(&kb.1))
            });
            any_passed = !passed.is_empty();
            for c in passed.into_iter().take(MAX_PASSED) {
                add(&mut p.voting, &c.name);
            }
        }
        // When the trend found no growth anywhere, the runtime's after-GC heap is the
        // column to watch over the extra cycles: whole-process RSS steps once at a
        // machine-dependent n (allocator and scavenger growth) and would decide a flat
        // workload by that step alone. RSS still votes when it passed the gate itself, or
        // when the runtime reports no heap column.
        let heap = region_primary(series, Some(SeriesRegion::Heap));
        let heap_present = heap != "os.rss" && series.column_index(heap).is_some();
        if trend.is_some() && !any_passed && heap_present {
            add(&mut p.voting, heap);
            p.basis = format!(
                "no column passed the gate in the latest trend of the harness: the runtime heap ({heap}) decides, os.rss is reported"
            );
        } else {
            if series.column_index("os.rss").is_some() {
                add(&mut p.voting, "os.rss");
            }
            p.basis = if trend.is_some() {
                "every column that passed the gate in the latest trend of the harness, and os.rss"
            } else {
                "os.rss (no trend of this harness yet: dla trend <series> first to test its growing columns)"
            }
            .to_owned();
        }
    }
    for c in &p.voting.clone() {
        add(&mut p.tested, c);
    }
    for name in SUPPORT {
        if series.column_index(name).is_some() {
            add(&mut p.tested, name);
        }
    }
    if p.voting.is_empty() {
        // Nothing else to judge by: the support columns decide.
        for name in SUPPORT {
            if series.column_index(name).is_some() {
                add(&mut p.voting, name);
            }
        }
    }
    Ok(p)
}
