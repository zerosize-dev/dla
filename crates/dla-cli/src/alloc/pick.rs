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

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::{
        Origin,
        finding::Region,
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
        }
    }

    fn trend(passing: &[&str]) -> TrendReport {
        TrendReport {
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
            region_primary(&s, Some(SeriesRegion::NonHeap)),
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
            region_primary(&s, Some(SeriesRegion::NonHeap)),
            "go.goroutines"
        );
        let ledger = Ledger::from_text("").unwrap();
        let p = pick(&s, &ledger, Some(&non_heap_finding()), &[], None, "H1").unwrap();
        assert_eq!(p.primary.as_deref(), Some("go.goroutines"));
        assert_eq!(p.voting, ["go.goroutines"]);
        // A series with only the process counters keeps the thread count.
        let s = series_of(&["os.rss", "os.threads"]);
        assert_eq!(
            region_primary(&s, Some(SeriesRegion::NonHeap)),
            "os.threads"
        );
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
