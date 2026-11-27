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
const ACTIVITY: &[&str] = &[
    "py.gc_collections",
    "jvm.jit_seconds",
    "jvm.classes_unloaded",
];
/// The JVM columns a non-heap test reports next to its vote and never counts: a flat class
/// count and flat unloads say there is no class growth and no churn, and compile time
/// levelling off says the JIT has stopped. They tell which cause a verdict is, not the verdict.
const JVM_SIGNS: &[&str] = &[
    "jvm.classes_loaded",
    "jvm.classes_unloaded",
    "jvm.jit_seconds",
];
/// The agent's native-memory categories: compiler scratch arenas and symbol tables swing by
/// tens of megabytes on a flat workload, so on a test with no bound finding they are reported
/// and never vote (and a pass of one does not stop the runtime heap from deciding).
const NMT_PREFIX: &str = "jvm.nmt.";
/// Process peaks and virtual sizes. A peak never falls, and a purge does not give reserved
/// address space back (`os.vm_data` counts reservations, not memory in use), so the purge
/// test cannot release them and their verdict is at best inconclusive; one such vote would
/// block every non-leak verdict. They are reported and vote only when nothing else can,
/// even after passing the gate.
const NO_RELEASE: &[&str] = &["os.vm_hwm", "native.peak_bytes", "os.vm_data"];
/// Columns whose purge reading says nothing about them: it cannot tell growth from retention,
/// so the column is judged on its plateau alone (it still votes, and the reading is still
/// reported, marked as not applying). `os.rss_file` (resident file-backed pages): no purge kind gives file-backed
/// pages back, so a rise of the column is always "retained", and the step that purges can map
/// files itself (the Python `trim` step did, inside the measurement, until the harness
/// loaded the library at start-up; the JVM purge steps run code the program has not run
/// before, judged by reading). The Go and Node purge records do not carry the column at
/// all. A real growth of file-backed memory still reads as one through its plateau test; a
/// plateau that cannot decide leaves the column inconclusive.
pub const PURGE_CANNOT_CONTRADICT: &[&str] = &["os.rss_file"];
/// Whether no purge can give this column's memory back, whatever the program holds: the
/// process peaks and virtual sizes of [`NO_RELEASE`], and `go.total` (`/memory/classes/total`,
/// everything the runtime has mapped, released pages included: a collection or a return of
/// memory to the system moves bytes inside it, and the figure itself only steps up). A
/// retained purge reading of such a column says nothing about its extra window, so it is never
/// weighed against its plateau (the column still votes where it did, the reading stays in the
/// result, and a continuing plateau still reads leak).
pub fn purge_cannot_release(name: &str) -> bool {
    NO_RELEASE.contains(&name) || name == "go.total"
}
/// Whether the purge reading of this column says anything about it, whatever the purge kind:
/// not for a region no kind returns (file-backed pages: `trim` and `trim-native-heap` free
/// malloc heap, `gc`, `free-os-memory` and `jemalloc` free managed or allocator heap and
/// `empty-cache` a device cache, none of them mapped files) nor for the peaks, virtual sizes
/// and runtime totals of [`purge_cannot_release`]. The column is then judged on its plateau
/// alone: a plateau that cannot decide stays inconclusive, it is not read as a leak because the
/// purge gave nothing back.
pub fn purge_does_not_apply(name: &str) -> bool {
    PURGE_CANNOT_CONTRADICT.contains(&name) || purge_cannot_release(name)
}
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
    /// Columns that do not decide but must plateau too before the primary may favour
    /// non-leak (the JVM's NMT categories next to `jvm.nonheap_used`): one that keeps
    /// growing turns a plateau into inconclusive, while a leak verdict stays the primary's.
    pub required: Vec<String>,
    /// How the columns were chosen, in a few words.
    pub basis: String,
    /// A limitation of the choice the result must carry.
    pub note: Option<String>,
    /// The allocation-site column of the finding that grew in a trend of some harness of the run
    /// and is not in this series, and the line it counts. Nothing votes then: the verdict is the
    /// one the coverage of the run gives for the line (see `alloc_cmd`).
    pub absent_site: Option<AbsentSite>,
}

/// An allocation-site count column of the finding that is missing from the series under test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsentSite {
    /// The column (`py.blocks@<file:line>`).
    pub column: String,
    /// The line it counts (`file:line`).
    pub site: String,
    /// The series whose trend showed the column grow (`S3`): the growth was measured there.
    pub series: String,
}

/// The newest trend report of a series of harness `harness` (a harness series first, then
/// any series of it).
pub fn latest_trend(run_dir: &Path, harness: &str) -> Option<TrendReport> {
    let mut ids: Vec<(u32, String)> = fs::read_dir(run_dir.join(TREND_DIR))
        .ok()?
        .filter_map(Result::ok)
        // An empty file is an id reserved by a writer that has not finished.
        .filter(|e| dla_core::fsutil::is_written(&e.path()))
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
        if series.source.run_harness() != Some(harness) {
            continue;
        }
        if series.source.kind == SeriesSourceKind::Harness {
            return Some(report);
        }
        any.get_or_insert(report);
    }
    any
}

/// The Go heap column read while the workload ran (command mode). The exit-time `go.heap_live`
/// is read after `main` returned, so state owned by locals of the workload is gone from it.
const GO_IN_RUN: &str = "go.heap_live.in_run";

/// The least share of the in-run column's rise the exit column must show to lead beside it.
const EXIT_SHARE: f64 = 0.1;

/// Whether `go.heap_live.in_run` decides for a heap finding: `trend` passed it through the gate,
/// and the exit column `go.heap_live` either did not pass or rose by less than [`EXIT_SHARE`] of
/// what the in-run column rose by. An exit column can pass the gate on a few hundred bytes of
/// its own while the state the finding holds is gone at exit; a rise of that size says nothing
/// about the finding.
fn in_run_leads(trend: &TrendReport) -> bool {
    let passed = |n: &str| {
        trend
            .columns
            .iter()
            .find(|c| c.name == n && c.gate == GateVerdict::Pass)
    };
    let Some(in_run) = passed(GO_IN_RUN) else {
        return false;
    };
    passed("go.heap_live").is_none_or(|exit| exit.rise < in_run.rise * EXIT_SHARE)
}

/// The column a region is summarized by in `series`. Of the heap columns, `go.heap_live.in_run`
/// leads when [`in_run_leads`] says so: where the exit column `go.heap_live` passed too and
/// carries a real share of the growth, the state survives the exit and the exact exit reading
/// decides (the in-run one is a floor over a few samples, the noisier of the two); where only
/// the in-run column grows, the exit column is flat for a reason that has nothing to do with
/// the finding.
fn region_primary(
    series: &SeriesFile,
    region: Option<SeriesRegion>,
    trend: Option<&TrendReport>,
) -> &'static str {
    if region == Some(SeriesRegion::Heap)
        && series.column_index(GO_IN_RUN).is_some()
        && trend.is_some_and(in_run_leads)
    {
        return GO_IN_RUN;
    }
    let lists: &[&str] = match region {
        Some(SeriesRegion::Heap) => &[
            "jvm.heap_used",
            "go.heap_live",
            "node.heap_used",
            "py.traced",
            "native.live_bytes",
        ],
        Some(SeriesRegion::OffHeap) => &["native.live_bytes", "gap.unexplained"],
        // A JVM series has `jvm.nonheap_used` (bytes: pools and code cache) and a Go series
        // does not, so Go keeps `go.goroutines` here.
        Some(SeriesRegion::NonHeap) => &[
            "jvm.nonheap_used",
            "go.goroutines",
            "jvm.threads",
            "os.threads",
        ],
        _ => &[],
    };
    lists
        .iter()
        .copied()
        .find(|n| series.column_index(n).is_some())
        .unwrap_or("os.rss")
}

/// The `jvm.nmt.<category>` columns of `series` that passed the gate in `trend`.
fn nmt_categories_that_passed(series: &SeriesFile, trend: Option<&TrendReport>) -> Vec<String> {
    trend
        .into_iter()
        .flat_map(|t| t.columns.iter())
        .filter(|c| c.gate == GateVerdict::Pass && c.name.starts_with("jvm.nmt."))
        .filter(|c| series.column_index(&c.name).is_some())
        .map(|c| c.name.clone())
        .collect()
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
    owner: Option<&str>,
    trend: Option<&TrendReport>,
) -> (Vec<String>, &'static str) {
    let passes = |name: &str| {
        trend
            .and_then(|t| t.columns.iter().find(|c| c.name == name))
            .is_some_and(|c| c.gate == GateVerdict::Pass)
    };
    // A size probe reads the holder of the finding itself, so it leads: a step-growing
    // container reads flat between two steps in a whole-process column, never in its own
    // size. Inside each group the columns that passed the gate in the trend come first.
    let order = |mut v: Vec<String>| {
        v.sort_by_key(|n| (!n.starts_with("probe:"), !passes(n)));
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
    let by_owner: Vec<String> = series
        .columns
        .iter()
        .filter(|c| owner.is_some_and(|o| dla_core::trend::probe_column_matches_owner(&c.name, o)))
        .map(|c| c.name.clone())
        .collect();
    if !by_owner.is_empty() {
        return (order(by_owner), "the probe columns named after its owner");
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

/// The allocation-site count column (`py.blocks@<file:line>`) that counts as growth
/// ([`dla_core::grown::counts_as_growth`]) in a trend of ANY harness of the run, is tied to `root`
/// ([`dla_core::grown::column_tied_to`]: its finding, its candidates, or the finding's own site)
/// and is not in `series`. The agent writes only the sites that grew, so the absence says the count
/// at the line did not grow in this series, or the line did not run; the whole process
/// (`py.traced`: a unit that runs a test runner grows on its own) cannot decide for one line either
/// way. The newest trend is read first.
fn vanished_site_column(
    series: &SeriesFile,
    ledger: &Ledger,
    root: &FindingId,
    trends: &[&TrendReport],
) -> Option<AbsentSite> {
    trends.iter().rev().find_map(|t| {
        t.columns
            .iter()
            .filter(|c| {
                dla_core::grown::is_alloc_site_column(&c.name)
                    && dla_core::grown::counts_as_growth(c)
            })
            .filter(|c| dla_core::grown::column_tied_to(ledger, c, root))
            .find(|c| series.column_index(&c.name).is_none())
            .and_then(|c| {
                Some(AbsentSite {
                    column: c.name.clone(),
                    site: dla_core::grown::column_site(c)?.to_owned(),
                    series: t.series.clone(),
                })
            })
    })
}

/// Chooses the columns: `--column` names, else the columns tied to the finding (its
/// probe, its site group, its region's primary column), else every column that passed the
/// gate in the latest trend of the harness plus `os.rss`. `os.rss`, `os.vm_hwm` and
/// `native.peak_bytes` are always tested for the purge reading.
#[cfg(test)]
pub fn pick(
    series: &SeriesFile,
    ledger: &Ledger,
    finding: Option<&Finding>,
    explicit: &[String],
    trend: Option<&TrendReport>,
    harness: &str,
) -> Res<Picked> {
    pick_in(
        series,
        ledger,
        finding,
        explicit,
        trend,
        &trend.into_iter().collect::<Vec<_>>(),
        harness,
    )
}

/// [`pick`], with every trend of the run (`all_trends`, oldest first) for the allocation-site
/// column of the finding that grew in some harness and is missing from this series.
pub fn pick_in(
    series: &SeriesFile,
    ledger: &Ledger,
    finding: Option<&Finding>,
    explicit: &[String],
    trend: Option<&TrendReport>,
    all_trends: &[&TrendReport],
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
        let root_owner = ledger
            .finding(&root)
            .and_then(|r| r.owner.clone())
            .or_else(|| f.owner.clone());
        let (cols, how) = tied(
            series,
            &root,
            root_site.as_deref(),
            root_owner.as_deref(),
            trend,
        );
        let vanished = if cols.is_empty() {
            vanished_site_column(series, ledger, &root, all_trends)
        } else {
            None
        };
        if let Some(gone) = vanished {
            let col = &gone.column;
            p.basis = format!(
                "no column decides: {col}, the count at the line of {root}, is not in this series"
            );
            p.note = Some(format!(
                "{col} grew in a trend of this run and is not in this series: the agent writes only the sites that grew, so the count at the line did not grow here (or the line did not run). A whole-process column such as py.traced cannot decide for one line, so nothing votes"
            ));
            p.absent_site = Some(gone);
        } else if let Some(first) = cols.first() {
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
            let name = region_primary(series, region, trend);
            p.primary = Some(name.to_owned());
            p.voting.push(name.to_owned());
            p.basis = if name == GO_IN_RUN {
                format!("the primary column of the region of {root}, the in-run reading")
            } else {
                format!("the primary column of the region of {root}")
            };
            if name == "jvm.nonheap_used" {
                // JIT data, class metadata and symbols all land in non-heap: the pools'
                // total plateaus only if every NMT category that grew plateaus too.
                let nmt = nmt_categories_that_passed(series, trend);
                if !nmt.is_empty() {
                    for c in &nmt {
                        add(&mut p.tested, c);
                        add(&mut p.required, c);
                    }
                    p.basis = format!(
                        "jvm.nonheap_used, which must plateau together with every jvm.nmt.* category that passed the gate in the latest trend, for the region of {root}"
                    );
                }
                // A thread leak lands in non-heap too, in stacks the pools do not hold: a thread
                // count that is still rising (or that no trend has judged yet) keeps the pools'
                // plateau from refuting anything.
                for t in ["jvm.threads", "os.threads"] {
                    let passed = trend.is_none_or(|tr| {
                        tr.columns
                            .iter()
                            .any(|c| c.name == t && c.gate == GateVerdict::Pass)
                    });
                    if passed && series.column_index(t).is_some() {
                        add(&mut p.tested, t);
                        add(&mut p.required, t);
                    }
                }
                for sign in JVM_SIGNS {
                    if series.column_index(sign).is_some() {
                        add(&mut p.tested, sign);
                    }
                }
            }
        }
    } else {
        let mut any_passed = false;
        if let Some(t) = trend {
            // A peak or virtual size that passed the gate is still reported.
            for c in &t.columns {
                if c.gate == GateVerdict::Pass
                    && NO_RELEASE.contains(&c.name.as_str())
                    && series.column_index(&c.name).is_some()
                {
                    add(&mut p.tested, &c.name);
                }
            }
            for c in &t.columns {
                if c.gate == GateVerdict::Pass
                    && c.name.starts_with(NMT_PREFIX)
                    && series.column_index(&c.name).is_some()
                {
                    add(&mut p.tested, &c.name);
                }
            }
            let mut passed: Vec<&dla_core::trend::ColumnTrend> = t
                .columns
                .iter()
                .filter(|c| c.gate == GateVerdict::Pass)
                .filter(|c| series.column_index(&c.name).is_some())
                .filter(|c| !c.name.starts_with(NMT_PREFIX))
                .filter(|c| !ACTIVITY.contains(&c.name.as_str()))
                .filter(|c| !NO_RELEASE.contains(&c.name.as_str()))
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
        let heap = region_primary(series, Some(SeriesRegion::Heap), None);
        let heap_present = heap != "os.rss" && series.column_index(heap).is_some();
        if trend.is_some() && !any_passed && heap_present {
            add(&mut p.voting, heap);
            p.basis = format!(
                "no column of memory in use passed the gate in the latest trend of the harness (peaks and sizes that cannot be given back do not count): the runtime heap ({heap}) decides, os.rss is reported"
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
    if p.voting.is_empty() && p.note.is_none() {
        // Nothing else to judge by: the support columns decide. (Not for a finding at a line
        // whose count column is missing: no whole-process column decides for one line.)
        for name in SUPPORT {
            if series.column_index(name).is_some() {
                add(&mut p.voting, name);
            }
        }
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::{
        Origin,
        finding::{Holder, Region},
        series::{Axis, Column, ColumnKind, ColumnScope, Point, SeriesSource, Unit, column_meta},
        trend::{ColumnTrend, TREND_SCHEMA_VERSION},
    };
    use dla_stats::GateThresholds;

    fn series_of(names: &[&str]) -> SeriesFile {
        let columns: Vec<Column> = names
            .iter()
            .map(|n| {
                let m = column_meta(n);
                Column {
                    name: (*n).to_owned(),
                    unit: m.unit,
                    kind: m.kind,
                    region: m.region,
                    baseline: m.baseline,
                    after_gc: m.after_gc,
                    scope: ColumnScope::Process,
                    site: None,
                    finding: None,
                    source: "test".to_owned(),
                }
            })
            .collect();
        let points = (0..8)
            .map(|i| Point {
                x: 100.0 * (i as f64 + 1.0),
                t_ms: i as u64,
                values: names.iter().map(|_| Some(1e6 + 1e3 * i as f64)).collect(),
            })
            .collect();
        SeriesFile {
            schema_version: 1,
            id: "S1".to_owned(),
            source: SeriesSource::new(SeriesSourceKind::AllocTest),
            axis: Axis::N,
            warmup: 2,
            columns,
            points,
            events: vec![],
            limitations: vec![],
        }
    }

    fn passed(name: &str) -> ColumnTrend {
        ColumnTrend {
            name: name.to_owned(),
            unit: Unit::Bytes,
            kind: ColumnKind::Gauge,
            region: SeriesRegion::NonHeap,
            site: None,
            finding: None,
            candidates: vec![],
            n: 6,
            mk: None,
            sen: None,
            halves: None,
            every_delta: None,
            noise: 1.0,
            rise: 1e5,
            shape: dla_stats::TrendShape::Steady,
            gate: GateVerdict::Pass,
            gate_failed: vec![],
            gate_note: None,
            resized: false,
        }
    }

    fn trend(passing: &[&str]) -> TrendReport {
        TrendReport {
            targets: Vec::new(),
            schema_version: TREND_SCHEMA_VERSION,
            id: "T1".to_owned(),
            series: "S1".to_owned(),
            thresholds: GateThresholds::default(),
            columns: passing.iter().map(|n| passed(n)).collect(),
        }
    }

    fn non_heap_finding() -> Finding {
        let mut f = Finding::candidate(
            FindingId::new(24),
            Origin::parse("residual").unwrap(),
            vec![],
        );
        f.region = Some(Region::NonHeap);
        f
    }

    fn heap_finding() -> Finding {
        let mut f = non_heap_finding();
        f.region = Some(Region::Heap);
        f
    }

    /// A finding at `shop/store.py:5`, the series without or with the allocation-site column
    /// of that line, and a trend that passed it.
    fn site_case(in_series: bool) -> (SeriesFile, Ledger, Finding, TrendReport) {
        let site_col = "py.blocks@shop/store.py:5";
        let mut names = vec!["py.traced", "os.rss"];
        if in_series {
            names.push(site_col);
        }
        let mut s = series_of(&names);
        if in_series {
            s.columns[2].site = Some("shop/store.py:5".to_owned());
        }
        let mut f = heap_finding();
        f.site = Some("shop/store.py:5".to_owned());
        let mut t = trend(&["py.traced", site_col]);
        t.columns[1].site = Some("shop/store.py:5".to_owned());
        t.columns[1].candidates = vec![f.id.clone()];
        (s, Ledger::from_text("").unwrap(), f, t)
    }

    #[test]
    fn a_finding_at_a_line_is_decided_by_the_site_column_not_by_the_whole_process() {
        let (s, ledger, f, t) = site_case(true);
        let p = pick(&s, &ledger, Some(&f), &[], Some(&t), "H1").unwrap();
        assert_eq!(
            p.primary.as_deref(),
            Some("py.blocks@shop/store.py:5"),
            "{}",
            p.basis
        );
        assert_eq!(p.voting, ["py.blocks@shop/store.py:5"]);
        assert!(p.note.is_none());
    }

    #[test]
    fn a_site_column_that_is_not_in_the_new_series_leaves_nothing_to_vote_for_the_line() {
        // The agent writes only the sites that grew: the column is missing, `py.traced` stays
        // the whole process and does not decide for one line.
        let (s, ledger, f, t) = site_case(false);
        let p = pick(&s, &ledger, Some(&f), &[], Some(&t), "H1").unwrap();
        assert!(p.primary.is_none() && p.voting.is_empty(), "{p:?}");
        assert!(p.basis.contains("py.blocks@shop/store.py:5"), "{}", p.basis);
        assert!(
            p.note.as_deref().is_some_and(|n| n.contains("py.traced")),
            "{p:?}"
        );
        // A finding of another line is not held to it.
        let mut other = f.clone();
        other.site = Some("shop/other.py:9".to_owned());
        other.id = FindingId::new(25);
        let p = pick(&s, &ledger, Some(&other), &[], Some(&t), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("py.traced"), "{}", p.basis);
    }

    #[test]
    fn the_site_column_that_grew_in_a_trend_of_another_harness_is_the_one_looked_for() {
        // The trend of this harness (a driver made for other code) has no site column at all;
        // the one of another harness has it passed, tied to the finding. Its absence in the new
        // series is what is read, not the whole-process fallback.
        let (s, ledger, f, t_other) = site_case(false);
        let t_here = trend(&["py.traced"]);
        let all = [&t_other, &t_here];
        let p = pick_in(&s, &ledger, Some(&f), &[], Some(&t_here), &all, "H2").unwrap();
        assert!(p.primary.is_none() && p.voting.is_empty(), "{p:?}");
        let gone = p.absent_site.expect("the missing column is named");
        assert_eq!(gone.column, "py.blocks@shop/store.py:5");
        assert_eq!(gone.site, "shop/store.py:5");
        // The series whose trend showed it grow is the one the next step names.
        assert_eq!(gone.series, "S1");
        // With no trend of any harness that counted it, the whole-process column is the fallback
        // as before.
        let p = pick_in(&s, &ledger, Some(&f), &[], Some(&t_here), &[&t_here], "H2").unwrap();
        assert_eq!(p.primary.as_deref(), Some("py.traced"), "{}", p.basis);
        assert!(p.absent_site.is_none());
        // A non-flat site column whose gate failed counts as growth too.
        let mut failed = t_other.clone();
        failed.columns[1].gate = GateVerdict::Fail;
        let p = pick_in(&s, &ledger, Some(&f), &[], Some(&t_here), &[&failed], "H2").unwrap();
        assert!(p.absent_site.is_some(), "{p:?}");
        // A flat one does not.
        failed.columns[1].shape = dla_stats::TrendShape::Flat;
        let p = pick_in(&s, &ledger, Some(&f), &[], Some(&t_here), &[&failed], "H2").unwrap();
        assert!(p.absent_site.is_none(), "{p:?}");
    }

    #[test]
    fn a_size_probe_of_the_finding_decides_before_the_columns_that_passed_the_trend() {
        // The finding has a size probe and a process column tied to it; the trend passed the
        // process column only. The probe reads the holder itself and decides.
        let mut s = series_of(&["go.heap_live", "probe:health.Server.statusMap"]);
        let f = heap_finding();
        for c in &mut s.columns {
            c.finding = Some(f.id.clone());
        }
        let ledger = Ledger::from_text("").unwrap();
        let t = trend(&["go.heap_live"]);
        let p = pick(&s, &ledger, Some(&f), &[], Some(&t), "H1").unwrap();
        assert_eq!(
            p.primary.as_deref(),
            Some("probe:health.Server.statusMap"),
            "{}",
            p.basis
        );
        assert_eq!(p.voting, ["probe:health.Server.statusMap"]);
        assert!(p.basis.contains("its probe column"), "{}", p.basis);
    }

    #[test]
    fn the_in_run_heap_column_is_the_heap_primary_only_when_the_trend_passed_it() {
        let s = series_of(&[
            "go.heap_live",
            "go.heap_live.in_run",
            "go.goroutines",
            "os.rss",
        ]);
        let ledger = Ledger::from_text("").unwrap();
        // The trend passed the in-run column: it decides, and the basis says so.
        let t = trend(&["go.heap_live.in_run"]);
        let p = pick(&s, &ledger, Some(&heap_finding()), &[], Some(&t), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live.in_run"));
        assert_eq!(p.voting, ["go.heap_live.in_run"]);
        assert!(p.basis.contains("in-run"), "{}", p.basis);
        // The trend ran and the in-run column did not pass: the exit column decides as before.
        let mut failed = trend(&["go.heap_live.in_run"]);
        failed.columns[0].gate = GateVerdict::Fail;
        let p = pick(&s, &ledger, Some(&heap_finding()), &[], Some(&failed), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live"), "{}", p.basis);
        assert!(!p.basis.contains("in-run"), "{}", p.basis);
        // A trend that passed only the exit column, and no trend at all.
        let t = trend(&["go.heap_live"]);
        let p = pick(&s, &ledger, Some(&heap_finding()), &[], Some(&t), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live"));
        let p = pick(&s, &ledger, Some(&heap_finding()), &[], None, "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live"));
        // A series without the column (driver mode, or no sample ended) keeps the exit column
        // even when the trend names the in-run one.
        let plain = series_of(&["go.heap_live", "go.goroutines", "os.rss"]);
        let t = trend(&["go.heap_live.in_run"]);
        let p = pick(&plain, &ledger, Some(&heap_finding()), &[], Some(&t), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live"));
        // Other regions are not touched by it.
        let t = trend(&["go.heap_live.in_run"]);
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], Some(&t), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.goroutines"));
    }

    #[test]
    fn the_exit_heap_column_leads_when_it_passed_the_trend_beside_the_in_run_one() {
        let s = series_of(&[
            "go.heap_live",
            "go.heap_live.in_run",
            "go.goroutines",
            "os.rss",
        ]);
        let ledger = Ledger::from_text("").unwrap();
        let both = trend(&["go.heap_live", "go.heap_live.in_run"]);
        let p = pick(&s, &ledger, Some(&heap_finding()), &[], Some(&both), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live"), "{}", p.basis);
        assert!(!p.basis.contains("in-run"), "{}", p.basis);
        let only = trend(&["go.heap_live.in_run"]);
        let p = pick(&s, &ledger, Some(&heap_finding()), &[], Some(&only), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live.in_run"));
        // The exit column passed on a rise of a kilobyte beside 100 MB in the run: the state
        // the finding holds is gone at exit, and the in-run column decides.
        let mut token = trend(&["go.heap_live", "go.heap_live.in_run"]);
        token.columns[0].rise = 1526.0;
        token.columns[1].rise = 113_058_442.0;
        let p = pick(&s, &ledger, Some(&heap_finding()), &[], Some(&token), "H1").unwrap();
        assert_eq!(
            p.primary.as_deref(),
            Some("go.heap_live.in_run"),
            "{}",
            p.basis
        );
        // An exit column that rose by most of what the in-run one did leads.
        token.columns[0].rise = 71_000_000.0;
        let p = pick(&s, &ledger, Some(&heap_finding()), &[], Some(&token), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live"), "{}", p.basis);
        let neither = trend(&["os.rss"]);
        let p = pick(
            &s,
            &ledger,
            Some(&heap_finding()),
            &[],
            Some(&neither),
            "H1",
        )
        .unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live"));
    }

    #[test]
    fn a_passing_in_run_column_votes_on_an_unbound_test_and_a_flat_one_leaves_the_exit_heap() {
        let s = series_of(&["go.heap_live", "go.heap_live.in_run", "os.rss", "os.vm_hwm"]);
        let ledger = Ledger::from_text("").unwrap();
        let t = trend(&["go.heap_live.in_run"]);
        let p = pick(&s, &ledger, None, &[], Some(&t), "H1").unwrap();
        assert!(
            p.voting.iter().any(|c| c == "go.heap_live.in_run"),
            "{:?} {}",
            p.voting,
            p.basis
        );
        // Nothing passed: the exit-time heap decides as it always did.
        let t = trend(&[]);
        let p = pick(&s, &ledger, None, &[], Some(&t), "H1").unwrap();
        assert_eq!(p.voting, ["go.heap_live"], "{}", p.basis);
    }

    const JVM: &[&str] = &[
        "jvm.heap_used",
        "jvm.nonheap_used",
        "jvm.threads",
        "jvm.classes_loaded",
        "jvm.classes_unloaded",
        "jvm.jit_seconds",
        "jvm.nmt.metaspace",
        "jvm.nmt.symbol",
        "jvm.nmt.code",
        "os.rss",
        "os.threads",
    ];

    #[test]
    fn the_non_heap_primary_of_a_jvm_is_its_byte_column_not_the_thread_count() {
        let s = series_of(JVM);
        assert_eq!(
            region_primary(&s, Some(SeriesRegion::NonHeap), None),
            "jvm.nonheap_used"
        );
        let ledger = Ledger::from_text("").unwrap();
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], None, "H1").unwrap();
        assert_eq!(
            p.primary.as_deref(),
            Some("jvm.nonheap_used"),
            "{}",
            p.basis
        );
        assert_eq!(p.voting, ["jvm.nonheap_used"]);
    }

    #[test]
    fn go_keeps_its_goroutines_as_the_non_heap_primary() {
        let s = series_of(&[
            "go.heap_live",
            "go.stacks",
            "go.goroutines",
            "os.rss",
            "os.threads",
        ]);
        assert_eq!(
            region_primary(&s, Some(SeriesRegion::NonHeap), None),
            "go.goroutines"
        );
        let ledger = Ledger::from_text("").unwrap();
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], None, "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.goroutines"));
        assert_eq!(p.voting, ["go.goroutines"]);
        // A series with only the process counters keeps the thread count.
        let s = series_of(&["os.rss", "os.threads"]);
        assert_eq!(
            region_primary(&s, Some(SeriesRegion::NonHeap), None),
            "os.threads"
        );
    }

    #[test]
    fn a_timer_finding_reads_the_heap_column_not_the_goroutines_of_the_census_region() {
        // An unstopped ticker is a timer the runtime keeps registered: the rule declares
        // registration-table and heap, which the join keeps over the census blocked-stack and
        // non-heap. The column follows the finding's region.
        let s = series_of(&["go.heap_live", "go.goroutines", "os.rss"]);
        let ledger = Ledger::from_text("").unwrap();
        let mut ticker = heap_finding();
        ticker.holder = Some(Holder::RegistrationTable);
        let p = pick(&s, &ledger, Some(&ticker), &[], None, "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.heap_live"), "{}", p.basis);
        let mut census_read = non_heap_finding();
        census_read.holder = Some(Holder::BlockedStack);
        let p = pick(&s, &ledger, Some(&census_read), &[], None, "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.goroutines"));
    }

    #[test]
    fn every_nmt_category_that_passed_must_plateau_with_the_pools_and_the_signs_only_report() {
        let s = series_of(JVM);
        let ledger = Ledger::from_text("").unwrap();
        // Metaspace and Symbol grew; Code did not pass; the class count and the compile time did.
        let t = trend(&[
            "jvm.nonheap_used",
            "jvm.nmt.metaspace",
            "jvm.nmt.symbol",
            "jvm.classes_loaded",
            "jvm.classes_unloaded",
            "jvm.jit_seconds",
        ]);
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], Some(&t), "H1").unwrap();
        assert_eq!(p.voting, ["jvm.nonheap_used"], "{}", p.basis);
        assert_eq!(p.primary.as_deref(), Some("jvm.nonheap_used"));
        assert_eq!(
            p.required,
            ["jvm.nmt.metaspace", "jvm.nmt.symbol"],
            "a category that passed must plateau too: {}",
            p.basis
        );
        for c in &p.required {
            assert!(p.tested.contains(c), "{c} is tested");
        }
        for sign in [
            "jvm.classes_loaded",
            "jvm.classes_unloaded",
            "jvm.jit_seconds",
        ] {
            assert!(p.tested.iter().any(|c| c == sign), "{sign} is reported");
            assert!(!p.voting.iter().any(|c| c == sign), "{sign} does not vote");
        }
        assert!(
            !p.tested.iter().any(|c| c == "jvm.nmt.code"),
            "a category that did not pass is not tested: {:?}",
            p.tested
        );
        // With no category passing, the pools decide alone, the signs still reported.
        let t = trend(&["jvm.nonheap_used"]);
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], Some(&t), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("jvm.nonheap_used"));
        assert_eq!(p.voting, ["jvm.nonheap_used"]);
        assert!(p.required.is_empty());
        assert!(p.tested.iter().any(|c| c == "jvm.jit_seconds"));
    }

    #[test]
    fn compile_time_and_unloads_never_vote_on_an_unbound_test() {
        let s = series_of(JVM);
        let ledger = Ledger::from_text("").unwrap();
        let t = trend(&["jvm.jit_seconds", "jvm.classes_unloaded", "jvm.nmt.symbol"]);
        let p = pick(&s, &ledger, None, &[], Some(&t), "H1").unwrap();
        assert!(
            !p.voting.iter().any(|c| c.starts_with("jvm.nmt.")),
            "{:?}",
            p.voting
        );
        assert!(
            !p.voting.iter().any(|c| c == "jvm.jit_seconds"),
            "{:?}",
            p.voting
        );
        assert!(
            !p.voting.iter().any(|c| c == "jvm.classes_unloaded"),
            "{:?}",
            p.voting
        );
    }

    #[test]
    fn a_passing_nmt_category_neither_votes_nor_stops_the_heap_rule_on_an_unbound_test() {
        // A compiler arena that swings on a flat workload passes the gate by chance: the
        // runtime heap still decides, the category is reported.
        let mut names = JVM.to_vec();
        names.push("jvm.nmt.arena_chunk");
        let s = series_of(&names);
        let ledger = Ledger::from_text("").unwrap();
        let t = trend(&["jvm.nmt.arena_chunk", "jvm.nmt.code"]);
        let p = pick(&s, &ledger, None, &[], Some(&t), "H1").unwrap();
        assert_eq!(p.voting, ["jvm.heap_used"], "{}", p.basis);
        assert!(p.tested.iter().any(|c| c == "jvm.nmt.arena_chunk"));
        assert!(p.tested.iter().any(|c| c == "jvm.nmt.code"));
    }

    #[test]
    fn a_file_backed_column_that_passed_the_gate_votes_beside_os_rss() {
        let s = series_of(&["os.rss", "os.rss_file", "go.heap_live"]);
        let ledger = Ledger::from_text("").unwrap();
        let t = trend(&["os.rss_file"]);
        let p = pick(&s, &ledger, None, &[], Some(&t), "H1").unwrap();
        assert!(
            p.voting.iter().any(|c| c == "os.rss_file"),
            "{:?}",
            p.voting
        );
        assert!(p.voting.iter().any(|c| c == "os.rss"), "{:?}", p.voting);
        assert_eq!(PURGE_CANNOT_CONTRADICT, ["os.rss_file"]);
    }

    #[test]
    fn a_thread_count_that_grows_must_plateau_before_the_pools_may_refute_a_non_heap_finding() {
        let s = series_of(JVM);
        let ledger = Ledger::from_text("").unwrap();
        // Thread dumps map to non-heap: stacks are not in the pools, so a rising thread
        // count is required next to jvm.nonheap_used.
        let t = trend(&["jvm.threads"]);
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], Some(&t), "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("jvm.nonheap_used"));
        assert_eq!(p.voting, ["jvm.nonheap_used"]);
        assert_eq!(p.required, ["jvm.threads"], "{}", p.basis);
        assert!(p.tested.iter().any(|c| c == "jvm.threads"));
        // No trend of the harness yet: nothing says the threads are flat, so both are required.
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], None, "H1").unwrap();
        assert_eq!(p.required, ["jvm.threads", "os.threads"]);
        // A trend in which the thread counts did not pass: they are not required.
        let t = trend(&["jvm.nonheap_used"]);
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], Some(&t), "H1").unwrap();
        assert!(p.required.is_empty(), "{:?}", p.required);
        // Go has no jvm.nonheap_used: its goroutines decide and nothing is required.
        let g = series_of(&["go.heap_live", "go.goroutines", "os.rss", "os.threads"]);
        let p = pick(&g, &ledger, Some(&non_heap_finding()), &[], None, "H1").unwrap();
        assert!(p.required.is_empty());
    }
}
