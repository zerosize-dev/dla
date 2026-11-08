#![allow(dead_code)]

use dla_core::{
    Authority, EvidenceId, EvidenceKind, FindingId, FindingState, Ledger, NewEvidence, NewFinding,
    Origin, Record, RunState, Source, Tool, evidence::EvidenceCoverage, finding::Holder,
    state::now_utc,
};
use serde_json::{Value, json};

pub fn evidence(kind: EvidenceKind, authority: Authority) -> NewEvidence {
    NewEvidence {
        source: Source::Repo {
            commit: "abc123".into(),
            path: None,
        },
        tool: Tool {
            name: "test".into(),
            version: "0".into(),
        },
        authority,
        kind,
        result: json!({"n": 1}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    }
}

pub fn eid(n: u32) -> EvidenceId {
    EvidenceId::new(n)
}

pub fn fid(n: u32) -> FindingId {
    FindingId::new(n)
}

pub fn origin(s: &str) -> Origin {
    Origin::parse(s).unwrap()
}

/// Evidence with an explicit `result`.
pub fn evidence_with(kind: EvidenceKind, authority: Authority, result: Value) -> NewEvidence {
    NewEvidence {
        result,
        ..evidence(kind, authority)
    }
}

/// A memory-only ledger holding a static hit (E1), a discriminator that favors
/// non-leak (E2) and a live growth series (E3).
pub fn ledger() -> Ledger {
    let mut l = Ledger::from_text("").unwrap();
    l.add_evidence(evidence(EvidenceKind::StaticHit, Authority::Static))
        .unwrap();
    l.add_evidence(evidence_with(
        EvidenceKind::Discriminator,
        Authority::Test,
        json!({"test": "plateau", "favors": "non-leak", "supports": ["N-WARM"], "finding": null}),
    ))
    .unwrap();
    l.add_evidence(evidence(EvidenceKind::GrowthSeries, Authority::Live))
        .unwrap();
    l
}

/// Evidence numbers of [`m3_ledger`].
pub const E_STATIC: u32 = 1;
/// A discriminator that favors non-leak and names no finding.
pub const E_DISC_NONLEAK: u32 = 2;
/// A live growth series with no gate in its result.
pub const E_GROWTH_NO_GATE: u32 = 3;
/// Passing growth (test), column `probe:CACHE` at `a.go:1`, bound to F1, steady, heap.
pub const E_GROWTH_PASS: u32 = 4;
/// A harness record from `dla harness gen`.
pub const E_HARNESS: u32 = 5;
/// A workload record from `dla workload discover` (no harness).
pub const E_DISCOVER: u32 = 6;
/// Growth (test) whose gate failed.
pub const E_GROWTH_FAIL: u32 = 7;
/// A discriminator that favors leak and names no finding.
pub const E_DISC_LEAK: u32 = 8;
/// A discriminator that is inconclusive.
pub const E_DISC_INCONCLUSIVE: u32 = 9;
/// A discriminator that favors non-leak but is bound to F9.
pub const E_DISC_NONLEAK_F9: u32 = 10;
/// Passing unit counts (dump), goroutines at `a.go:1`, stair, non-heap.
pub const E_UNIT_PASS: u32 = 11;
/// Growth with a passing gate but authority static.
pub const E_GROWTH_STATIC_AUTH: u32 = 12;
/// Passing growth (test) for another holder: column bound to F9 at `b.go:2`, plateau, off-heap.
pub const E_GROWTH_OTHER: u32 = 13;
/// A discriminator that favors leak but is bound to F9.
pub const E_DISC_LEAK_F9: u32 = 14;
/// A discriminator that favors non-leak and is bound to F1.
pub const E_DISC_NONLEAK_F1: u32 = 15;
/// A discriminator that favors leak and is bound to F1.
pub const E_DISC_LEAK_F1: u32 = 16;

fn disc(favors: &str, finding: Option<&str>) -> NewEvidence {
    let supports = if favors == "non-leak" {
        json!(["N-WARM"])
    } else {
        json!(["leak"])
    };
    evidence_with(
        EvidenceKind::Discriminator,
        Authority::Test,
        json!({"test": "plateau", "favors": favors, "supports": supports, "finding": finding}),
    )
}

fn growth(kind: EvidenceKind, authority: Authority, gate: &str, passed: Value) -> NewEvidence {
    evidence_with(
        kind,
        authority,
        json!({"trend": "T1", "series": "S1", "n": 8, "passed": passed, "failed": 0, "gate": gate}),
    )
}

/// [`ledger`] plus the evidence the M3 rules look at (E4 to E16, see the constants).
pub fn m3_ledger() -> Ledger {
    use Authority::*;
    use EvidenceKind::*;
    let mut l = ledger();
    let col_f1 = json!([{"column": "probe:CACHE", "region": "heap", "site": "a.go:1",
        "finding": "F1", "shape": "steady", "slope": 12.0, "lo": 10.0, "p": 0.001, "ratio": 1.0}]);
    let col_goroutines = json!([{"column": "go.goroutines@a.go:1", "region": "non-heap",
        "site": "a.go:1", "shape": "stair"}]);
    let col_other = json!([{"column": "probe:OTHER", "region": "off-heap", "site": "b.go:2",
        "finding": "F9", "shape": "plateau"}]);
    let adds = [
        growth(GrowthSeries, Test, "pass", col_f1.clone()),
        evidence_with(
            Workload,
            Test,
            json!({"harness": "H1", "mode": "command", "runtime": "go"}),
        ),
        evidence_with(Workload, Static, json!({"workloads": 3})),
        growth(GrowthSeries, Test, "fail", json!([])),
        disc("leak", None),
        disc("inconclusive", None),
        disc("non-leak", Some("F9")),
        growth(UnitCountSeries, Dump, "pass", col_goroutines),
        growth(GrowthSeries, Static, "pass", col_f1),
        growth(GrowthSeries, Test, "pass", col_other),
        disc("leak", Some("F9")),
        disc("non-leak", Some("F1")),
        disc("leak", Some("F1")),
    ];
    for e in adds {
        l.add_evidence(e).unwrap();
    }
    l
}

/// Adds a top-level leak-side finding with a site and a holder, supported by E1.
pub fn add_leak(l: &mut Ledger) -> FindingId {
    let mut nf = NewFinding::new(origin("census"), vec![eid(E_STATIC)]);
    nf.site = Some("a.go:1".into());
    nf.holder = Some(Holder::StaticGlobal);
    let line = l.add_finding(nf).unwrap();
    match line.record {
        Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    }
}

/// Adds a sibling of `root` whose hypothesis is `hypothesis`.
pub fn add_sibling(l: &mut Ledger, root: &FindingId, hypothesis: &str) -> FindingId {
    let mut nf = NewFinding::new(origin("hypothesis"), vec![eid(E_STATIC)]);
    nf.sibling_of = Some(root.clone());
    nf.hypothesis = Some(hypothesis.into());
    nf.refute_if = Some("growth continues".into());
    let line = l.add_finding(nf).unwrap();
    match line.record {
        Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    }
}

/// Moves `id` with the given evidence; panics when refused.
pub fn go(l: &mut Ledger, id: &FindingId, to: FindingState, ev: &[u32]) {
    l.transition(
        id,
        to,
        None,
        None,
        ev.iter().map(|n| eid(*n)).collect(),
        "test",
    )
    .unwrap_or_else(|e| panic!("{id} -> {to}: {e}"));
}

/// Adds a leak-side finding with an N-WARM sibling, refutes the sibling, tests the
/// finding and confirms it. Returns `(finding, sibling)`.
pub fn confirmed_leak(l: &mut Ledger) -> (FindingId, FindingId) {
    let f = add_leak(l);
    let n = add_sibling(l, &f, "N-WARM");
    go(l, &n, FindingState::Refuted, &[E_DISC_LEAK]);
    go(l, &f, FindingState::UnderTest, &[E_HARNESS]);
    go(l, &f, FindingState::Confirmed, &[E_GROWTH_PASS]);
    (f, n)
}

/// Adds F1 (supported by E1) and returns its ID.
pub fn add_f1(l: &mut Ledger) -> FindingId {
    l.add_finding(NewFinding::new(origin("census"), vec![eid(1)]))
        .unwrap();
    fid(1)
}

pub fn run_state() -> RunState {
    RunState::init("/tmp/project", Some("s1"), 3600, None, now_utc()).unwrap()
}

/// Hand-written ledger text with the given extra lines after one evidence line.
pub fn line(seq: u64, body: &str) -> String {
    format!("{{\"schema_version\":1,\"seq\":{seq},\"ts\":\"2026-01-01T00:00:00Z\",{body}}}\n")
}

pub const EVIDENCE_BODY: &str = r#""type":"evidence","evidence_id":"E1","source":{"repo":{"commit":"abc"}},"tool":{"name":"t","version":"1"},"authority":"static","kind":"static-hit","result":{}"#;

// ---------------------------------------------------------------------------------------
// M4: runs on disk with worker slices
// ---------------------------------------------------------------------------------------

use dla_core::state::{WorkerEntry, WorkerStatus};
use std::path::Path;

/// A run directory with `state.json` (the given workers registered, runtime = name,
/// deadline in an hour) and a main ledger holding the evidence of [`m3_ledger`] (E1 to E16).
pub fn new_run(workers: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let mut state = run_state();
    let now = now_utc();
    for w in workers {
        state.workers.push(WorkerEntry {
            name: (*w).to_owned(),
            runtime: (*w).to_owned(),
            share_secs: 600,
            deadline: now + chrono::TimeDelta::hours(1),
            started_at: None,
            status: WorkerStatus::Registered,
            slice: format!("ledger.{w}.jsonl"),
        });
    }
    state.save(dir.path()).unwrap();
    write_main(dir.path(), &m3_ledger());
    dir
}

/// Writes the lines of an in-memory ledger as `ledger.jsonl` in `dir`.
pub fn write_main(dir: &Path, l: &Ledger) {
    let text: String = l
        .lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect();
    std::fs::write(dir.join("ledger.jsonl"), text).unwrap();
}

/// Opens `worker`'s slice of the run in `dir`.
pub fn slice(dir: &Path, worker: &str) -> Ledger {
    Ledger::load_for(dir, Some(worker)).unwrap()
}

/// In worker mode: adds a leak-side finding with a site and a holder (supported by E1),
/// a N-WARM sibling, refutes the sibling, puts the finding under test and confirms it.
/// Returns the finding's own ID (`<worker>/F<n>`).
pub fn confirm_in_slice(l: &mut Ledger, site: &str, holder: Holder) -> FindingId {
    let mut nf = NewFinding::new(origin("census"), vec![eid(E_STATIC)]);
    nf.site = Some(site.into());
    nf.holder = Some(holder);
    let line = l.add_finding(nf).unwrap();
    let Record::FindingAdded { finding } = line.record else {
        unreachable!()
    };
    let f = finding.id;
    let n = add_sibling(l, &f, "N-WARM");
    go(l, &n, FindingState::Refuted, &[E_DISC_LEAK]);
    go(l, &f, FindingState::UnderTest, &[E_HARNESS]);
    go(l, &f, FindingState::Confirmed, &[E_GROWTH_PASS]);
    f
}

/// In worker mode: adds a leak-side finding with a site and a holder in `candidate`.
pub fn candidate_in_slice(l: &mut Ledger, site: &str, holder: Holder) -> FindingId {
    let mut nf = NewFinding::new(origin("census"), vec![eid(E_STATIC)]);
    nf.site = Some(site.into());
    nf.holder = Some(holder);
    let line = l.add_finding(nf).unwrap();
    let Record::FindingAdded { finding } = line.record else {
        unreachable!()
    };
    finding.id
}
