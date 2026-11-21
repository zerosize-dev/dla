//! The `trigger` item of the gate: a confirmed or strong leak-side finding that grew when a
//! harness fed it keys and whose trigger is unchecked, while the queue is on and time is left.
//! `dla run status`, the final gate and the stop hook list it from one function, so these tests
//! read `status` and `gate::missing`.

mod common;

use std::path::Path;

use common::*;
use dla_core::{
    FindingId, FindingState as S, Ledger, RunState, RunView, gate, status,
    state::now_utc,
    trend::{TargetVerdict, TrendReport, Verdict},
    view::BlockingKind,
};
use serde_json::json;

/// A run directory whose ledger has F1 confirmed (leak-side, `a.go:1`, holder static/global).
fn run() -> tempfile::TempDir {
    let dir = new_run(&[]);
    let mut l: Ledger = m3_ledger();
    let (f, _) = confirmed_leak(&mut l);
    assert_eq!(f, fid(1));
    write_main(dir.path(), &l);
    dir
}

fn harness(dir: &Path, id: &str, entry: bool) {
    let mut m = json!({
        "id": id, "runtime": "go", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": ["F1"], "agent": "driver-go", "timeout_secs": 60,
    });
    if entry {
        m["purpose"] = json!("entry-path");
    }
    let m: dla_core::HarnessManifest = serde_json::from_value(m).unwrap();
    m.save(dir).unwrap();
}

/// A series of `h`, its run record and its trend report with F1's verdict.
fn ran(dir: &Path, series: &str, h: &str, digest: &str, verdict: Verdict) {
    let text = json!({
        "id": series, "source": {"kind": "harness", "harness": h},
        "axis": "n", "columns": [], "points": []
    });
    std::fs::create_dir_all(dir.join("series")).unwrap();
    std::fs::write(dir.join(format!("series/{series}.json")), text.to_string()).unwrap();
    std::fs::write(
        dir.join("harness").join(h).join(format!("run-{series}.json")),
        json!({"series": [series], "driver_digest": digest, "varies": format!("{h} varies")})
            .to_string(),
    )
    .unwrap();
    TrendReport {
        schema_version: 1,
        id: format!("T{}", &series[1..]),
        series: series.into(),
        thresholds: dla_stats::GateThresholds::default(),
        columns: vec![],
        targets: vec![TargetVerdict {
            finding: FindingId::new(1),
            verdict,
            column: None,
            detail: String::new(),
            declared: true,
        }],
    }
    .save(dir)
    .unwrap();
}

fn items(dir: &Path) -> Vec<dla_core::view::BlockingItem> {
    let view = RunView::load(dir).unwrap();
    status(&view)
        .blocking
        .into_iter()
        .filter(|b| b.kind == BlockingKind::Trigger)
        .collect()
}

fn with_state(dir: &Path, f: impl FnOnce(&mut RunState)) {
    let mut st = RunState::load(dir).unwrap();
    f(&mut st);
    st.save(dir).unwrap();
}

#[test]
fn a_confirmed_leak_that_grew_under_a_hook_driver_is_asked_for_its_trigger() {
    let dir = run();
    // Nothing grew in a harness yet: there is no trigger to ask for, with no harness or with
    // one whose run showed no growth.
    assert!(items(dir.path()).is_empty());
    harness(dir.path(), "H1", false);
    ran(dir.path(), "S1", "H1", "aa", Verdict::Constant);
    assert!(items(dir.path()).is_empty());
    ran(dir.path(), "S1", "H1", "aa", Verdict::Grew);
    let it = items(dir.path());
    assert_eq!(it.len(), 1, "{it:?}");
    assert_eq!(it[0].id, "F1");
    assert_eq!(
        it[0].description,
        "F1 grew when the harness fed it new keys; whether a caller of the public API can do that is not shown"
    );
    assert!(
        it[0]
            .fix
            .starts_with("dla harness gen --lang go --driver --entry F1, fill the hooks"),
        "{}",
        it[0].fix
    );
    // The final gate lists it first among what is not the queue's, and with the same fix.
    let view = RunView::load(dir.path()).unwrap();
    let missing = gate::missing(&view, gate::GateTarget::Final, None);
    assert_eq!(missing[0].kind, "trigger", "{missing:?}");
    assert_eq!(missing[0].fix, it[0].fix);
    // A stage gate does not list it.
    let stage = gate::missing(&view, gate::GateTarget::Stage(dla_core::Stage::Probe), None);
    assert!(stage.iter().all(|m| m.kind != "trigger"));
}

#[test]
fn after_one_entry_path_harness_without_growth_the_fix_asks_for_a_second_one() {
    let dir = run();
    harness(dir.path(), "H1", false);
    ran(dir.path(), "S1", "H1", "aa", Verdict::Grew);
    harness(dir.path(), "H2", true);
    // Generated, not run: fill and run it.
    let it = items(dir.path());
    assert!(it[0].fix.starts_with("fill the hooks and VARIES.txt of H2"), "{}", it[0].fix);
    ran(dir.path(), "S2", "H2", "bb", Verdict::Constant);
    let it = items(dir.path());
    assert_eq!(it.len(), 1);
    assert!(
        it[0].description.contains("did not grow in the entry-path harness H2 (varies: H2 varies)"),
        "{}",
        it[0].description
    );
    assert!(
        it[0].fix.starts_with("write a second, different entry-path harness: dla harness gen --lang go --driver --entry F1"),
        "{}",
        it[0].fix
    );
    assert!(it[0].fix.contains("the rejected-input paths"), "{}", it[0].fix);
}

#[test]
fn after_two_different_entry_path_harnesses_without_growth_the_item_is_satisfied() {
    let dir = run();
    harness(dir.path(), "H1", false);
    ran(dir.path(), "S1", "H1", "aa", Verdict::Grew);
    harness(dir.path(), "H2", true);
    ran(dir.path(), "S2", "H2", "bb", Verdict::Constant);
    harness(dir.path(), "H3", true);
    ran(dir.path(), "S3", "H3", "cc", Verdict::Executed);
    assert!(items(dir.path()).is_empty());
}

#[test]
fn an_entry_path_harness_that_grew_it_satisfies_the_item() {
    let dir = run();
    harness(dir.path(), "H1", false);
    ran(dir.path(), "S1", "H1", "aa", Verdict::Grew);
    harness(dir.path(), "H2", true);
    ran(dir.path(), "S2", "H2", "bb", Verdict::Grew);
    assert!(items(dir.path()).is_empty());
}

#[test]
fn with_the_queue_off_nothing_is_asked() {
    let dir = run();
    harness(dir.path(), "H1", false);
    ran(dir.path(), "S1", "H1", "aa", Verdict::Grew);
    assert_eq!(items(dir.path()).len(), 1);
    with_state(dir.path(), |st| st.queue = false);
    assert!(items(dir.path()).is_empty());
    let view = RunView::load(dir.path()).unwrap();
    let missing = gate::missing(&view, gate::GateTarget::Final, None);
    assert!(missing.iter().all(|m| m.kind != "trigger"));
}

#[test]
fn at_the_floor_nothing_is_asked() {
    let dir = run();
    harness(dir.path(), "H1", false);
    ran(dir.path(), "S1", "H1", "aa", Verdict::Grew);
    assert_eq!(items(dir.path()).len(), 1);
    // The floor is the larger of 10 minutes and 20% of the budget: 100 s are left.
    with_state(dir.path(), |st| {
        st.budget.deadline = now_utc() + chrono::TimeDelta::seconds(100);
    });
    assert!(items(dir.path()).is_empty());
    let view = RunView::load(dir.path()).unwrap();
    assert_eq!(
        dla_core::trigger::unchecked_because(&view),
        Some("not enough time was left: the budget was at the floor")
    );
}

#[test]
fn a_workload_blocker_asks_nothing() {
    let dir = run();
    harness(dir.path(), "H1", false);
    ran(dir.path(), "S1", "H1", "aa", Verdict::Grew);
    assert_eq!(items(dir.path()).len(), 1);
    // The runner records that no driver can be built or run.
    let mut l = Ledger::load(dir.path()).unwrap();
    l.answer_checklist(dla_core::ChecklistAnswer {
        item: "intake.workload".into(),
        status: dla_core::AnswerStatus::Unresolved,
        evidence: vec![],
        note: None,
        reason: Some(dla_core::UnresolvedReason::EnvUnavailable),
        remedy: Some("no toolchain".into()),
    })
    .unwrap();
    write_main(dir.path(), &l);
    assert!(items(dir.path()).is_empty());
}

#[test]
fn a_refuted_or_unconfirmed_finding_is_not_asked() {
    // F1 is only a candidate that grew: confirmation is what asks for the trigger.
    let dir = new_run(&[]);
    let mut l: Ledger = m3_ledger();
    let f = add_leak(&mut l);
    assert_eq!(f, fid(1));
    assert_eq!(l.finding(&f).unwrap().state, S::Candidate);
    write_main(dir.path(), &l);
    harness(dir.path(), "H1", false);
    ran(dir.path(), "S1", "H1", "aa", Verdict::Grew);
    assert!(items(dir.path()).is_empty());
}
