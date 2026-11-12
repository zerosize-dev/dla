mod common;

use common::*;
use dla_core::{
    Authority as A, EvidenceKind as K, Ledger, RunView, coverage::CoverageManifest, measured,
    status,
};

fn one(kind: K, authority: A) -> RunView {
    let mut l = Ledger::from_text("").unwrap();
    l.add_evidence(evidence(kind, authority)).unwrap();
    RunView::new(run_state(), l, None)
}

#[test]
fn an_empty_run_measured_nothing() {
    let v = RunView::new(run_state(), Ledger::from_text("").unwrap(), None);
    assert!(!measured(&v).any());
    assert!(!status(&v).counts.measured);
}

#[test]
fn static_records_measure_nothing() {
    for kind in [
        K::StaticHit,
        K::CensusHolder,
        K::Probe,
        K::RunConfig,
        K::Workload,
    ] {
        let v = one(kind, A::Static);
        assert!(!measured(&v).any(), "{kind} with static authority");
    }
    // The probe and the run config never count, whatever authority they carry.
    for kind in [K::Probe, K::RunConfig] {
        for a in [A::Test, A::Live, A::Dump] {
            assert!(!measured(&one(kind, a)).any(), "{kind} {a}");
        }
    }
}

#[test]
fn each_measuring_source_flips_it() {
    let harness = one(K::Workload, A::Test);
    assert_eq!(measured(&harness).harness_runs, 1);
    let sample = one(K::GrowthSeries, A::Live);
    assert_eq!(measured(&sample).series, 1);
    let dump = one(K::HistogramDiff, A::Dump);
    assert_eq!(measured(&dump).captures, 1);
    for v in [&harness, &sample, &dump] {
        assert!(measured(v).any());
        assert!(status(v).counts.measured);
    }
}

#[test]
fn coverage_iterations_or_exercised_entry_points_flip_it() {
    let base = || RunView::new(run_state(), Ledger::from_text("").unwrap(), None);
    let mut m = CoverageManifest::new(None);
    m.worker_mut("jvm").iterations = Some(0);
    let mut v = base();
    v.coverage = Some(m.clone());
    assert!(!measured(&v).any(), "zero iterations");
    m.worker_mut("jvm").iterations = Some(4);
    v.coverage = Some(m);
    assert!(measured(&v).any());
    assert_eq!(
        measured(&v).harness_runs,
        1,
        "a worker with iterations, no workload record"
    );
    let mut m = CoverageManifest::new(None);
    m.add_exercised("jvm", ["a.Main".to_owned()]);
    let mut v = base();
    v.coverage = Some(m);
    assert!(measured(&v).any());
    assert_eq!(
        measured(&v).harness_runs,
        1,
        "a worker with exercised points, no workload record"
    );
}

#[test]
fn a_derived_gap_series_is_not_counted() {
    let mut v = RunView::new(run_state(), Ledger::from_text("").unwrap(), None);
    v.series_ids = vec!["S1".into(), "S2".into()];
    v.gap_series.insert("S2".into());
    assert_eq!(measured(&v).series, 1);
}

#[test]
fn series_count_files_not_the_several_records_each_series_writes() {
    // `dla trend` writes a gauge and a counter record per series, summarizers add more.
    let mut l = Ledger::from_text("").unwrap();
    for _ in 0..4 {
        l.add_evidence(evidence(K::GrowthSeries, A::Live)).unwrap();
    }
    let mut v = RunView::new(run_state(), l, None);
    v.series_ids = vec!["S1".into(), "S2".into()];
    assert_eq!(measured(&v).series, 2);
    assert!(measured(&v).any());
    // Only gap series files: the records do not make it a series.
    v.series_ids = vec!["S1".into()];
    v.gap_series.insert("S1".into());
    assert_eq!(measured(&v).series, 0);
}

#[test]
fn residual_discriminator_and_intervention_rows_are_not_captures() {
    for kind in [K::Residual, K::Discriminator, K::Intervention] {
        let m = measured(&one(kind, A::Live));
        assert_eq!(m.captures, 0, "{kind}");
        assert!(m.any(), "{kind} is still measured evidence");
    }
    assert_eq!(measured(&one(K::RetentionPath, A::Dump)).captures, 1);
    assert_eq!(measured(&one(K::NonHeap, A::Live)).captures, 1);
}

#[test]
fn the_split_dla_gap_writes_for_a_series_is_not_a_capture() {
    // `dla gap S` records the split as a non-heap record naming the derived series.
    let mut l = Ledger::from_text("").unwrap();
    let mut split = evidence(K::NonHeap, A::Test);
    split.result = serde_json::json!({"series": "S1", "derived": "S2", "route": null});
    l.add_evidence(split).unwrap();
    let mut v = RunView::new(run_state(), l, None);
    v.series_ids = vec!["S1".into(), "S2".into()];
    v.gap_series.insert("S2".into());
    let m = measured(&v);
    assert_eq!((m.series, m.captures), (1, 0));
    // A real non-heap reading (thread dump, NMT) still counts, with or without a result.
    let mut reading = evidence(K::NonHeap, A::Live);
    reading.result = serde_json::json!({"threads": 12});
    let mut l = Ledger::from_text("").unwrap();
    l.add_evidence(reading).unwrap();
    let v = RunView::new(run_state(), l, None);
    assert_eq!(measured(&v).captures, 1);
}
