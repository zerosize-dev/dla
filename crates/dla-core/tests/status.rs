mod common;

use common::*;
use dla_core::{
    AnswerStatus, ChecklistAnswer, FindingState as S, Ledger, NewFinding, Resolution, RunStatus,
    RunView, UnresolvedReason, status, view::BlockingKind,
};

fn answer_all(l: &mut Ledger) {
    for e in l.checklist() {
        l.answer_checklist(ChecklistAnswer {
            item: e.item.id,
            status: AnswerStatus::Answered,
            evidence: vec![eid(1)],
            note: None,
            reason: None,
            remedy: None,
        })
        .unwrap();
    }
}

fn view(l: Ledger) -> RunView {
    RunView::new(run_state(), l, None)
}

#[test]
fn empty_ledger_is_incomplete_only_through_the_checklist() {
    let v = view(ledger());
    let s = status(&v);
    assert!(!s.complete);
    assert_eq!(s.counts.checklist_missing, 8);
    assert_eq!(s.blocking.len(), 8);
    assert!(
        s.blocking
            .iter()
            .all(|b| b.kind == BlockingKind::ChecklistItem
                && b.fix.starts_with("dla ledger resolve intake."))
    );
}

#[test]
fn answered_checklist_and_no_findings_is_complete() {
    let mut l = ledger();
    answer_all(&mut l);
    let s = status(&view(l));
    assert!(s.complete, "{:?}", s.blocking);
    assert_eq!(s.counts.enumerated, 0);
}

#[test]
fn candidate_and_under_test_block_with_a_fix() {
    let mut l = m3_ledger();
    answer_all(&mut l);
    add_f1(&mut l);
    // F1.n1 is a non-leak sibling, which may go under test without a sibling of its own.
    let n1 = add_sibling(&mut l, &fid(1), "N-WARM");
    go(&mut l, &n1, S::UnderTest, &[E_HARNESS]);
    let s = status(&view(l.clone()));
    assert!(!s.complete);
    assert_eq!(s.blocking.len(), 2);
    assert_eq!(s.blocking[0].id, "F1");
    assert_eq!(
        s.blocking[0].fix,
        // The fixture ledger holds a discriminator for F1: it was measured, so the reason the
        // evidence supports is inconclusive. The printed fix never pre-fills time_budget.
        "dla ledger resolve F1 --as unresolved --reason inconclusive --remedy \"<text>\""
    );
    assert_eq!((s.counts.enumerated, s.counts.dispositioned), (2, 0));
    assert_eq!(
        (s.counts.by_state.candidate, s.counts.by_state.under_test),
        (1, 1)
    );

    for f in [fid(1), n1] {
        l.transition(
            &f,
            S::Unresolved,
            Some(UnresolvedReason::TimeBudget),
            Some("more time".into()),
            vec![],
            "t",
        )
        .unwrap();
    }
    let s = status(&view(l));
    assert!(s.complete, "unresolved with a reason must not block");
    assert_eq!(
        (
            s.counts.enumerated,
            s.counts.dispositioned,
            s.counts.by_state.unresolved
        ),
        (2, 2, 2)
    );
}

#[test]
fn every_disposition_state_counts_and_does_not_block() {
    let mut l = m3_ledger();
    answer_all(&mut l);
    // F1 confirmed and F2 strong, each with a refuted N-WARM sibling.
    confirmed_leak(&mut l);
    let (strong, _) = confirmed_leak(&mut l);
    let (g, r) = (
        push(&mut l, trend_growth("aa")),
        push(&mut l, vgdb_retention("bb")),
    );
    go(&mut l, &strong, S::Strong, &[g, r]);
    for _ in 0..2 {
        l.add_finding(NewFinding::new(origin("census"), vec![eid(1)]))
            .unwrap();
    }
    let tr = |l: &mut Ledger, n, to, reason: Option<UnresolvedReason>, ev| {
        let rem = reason.map(|_| "r".to_owned());
        l.transition(&fid(n), to, reason, rem, ev, "t").unwrap();
    };
    tr(
        &mut l,
        3,
        S::SuspectedStaticOnly,
        Some(UnresolvedReason::NotExercised),
        vec![],
    );
    tr(
        &mut l,
        4,
        S::Unresolved,
        Some(UnresolvedReason::NoHarness),
        vec![],
    );
    let s = status(&view(l));
    assert!(s.complete, "{:?}", s.blocking);
    let c = s.counts.by_state;
    assert_eq!(
        (
            c.confirmed,
            c.strong,
            c.refuted,
            c.suspected_static_only,
            c.unresolved
        ),
        (1, 1, 2, 1, 1)
    );
    assert_eq!(s.counts.dispositioned, 6);
}

#[test]
fn open_items_block_until_resolved() {
    let mut l = ledger();
    answer_all(&mut l);
    add_f1(&mut l);
    l.transition(
        &fid(1),
        S::Unresolved,
        Some(UnresolvedReason::NoHarness),
        Some("write one".into()),
        vec![],
        "t",
    )
    .unwrap();
    l.open_item_add("residual in off-heap", None, "test")
        .unwrap();
    let s = status(&view(l.clone()));
    assert!(!s.complete);
    assert_eq!(s.counts.open_items, 1);
    assert_eq!(s.blocking[0].kind, BlockingKind::OpenItem);
    assert_eq!(s.blocking[0].id, "O1");
    assert!(s.blocking[0].fix.starts_with("dla ledger resolve O1"));
    l.resolve_open_item(
        &dla_core::OpenItemId::new(1),
        Resolution::Unresolved {
            reason: UnresolvedReason::TimeBudget,
            remedy: "measure".into(),
        },
    )
    .unwrap();
    assert!(status(&view(l)).complete);
}

#[test]
fn aborted_or_incomplete_run_blocks() {
    for st in [RunStatus::Aborted, RunStatus::Incomplete] {
        let mut l = ledger();
        answer_all(&mut l);
        let mut state = run_state();
        state.status = st;
        state.incomplete_reason = Some("budget exhausted".into());
        let s = status(&RunView::new(state, l, None));
        assert!(!s.complete);
        assert_eq!(s.blocking.len(), 1);
        assert_eq!(s.blocking[0].kind, BlockingKind::Run);
        assert!(s.blocking[0].description.contains("budget exhausted"));
    }
    let mut l = ledger();
    answer_all(&mut l);
    let mut state = run_state();
    state.status = RunStatus::Complete;
    assert!(status(&RunView::new(state, l, None)).complete);
}

#[test]
fn run_view_loads_from_a_directory() {
    let dir = tempfile::tempdir().unwrap();
    let state = run_state();
    state.save(dir.path()).unwrap();
    let mut l = Ledger::create(dir.path()).unwrap();
    l.add_evidence(evidence(
        dla_core::EvidenceKind::RunConfig,
        dla_core::Authority::Static,
    ))
    .unwrap();
    let v = RunView::load(dir.path()).unwrap();
    assert_eq!(v.state, state);
    assert!(v.coverage.is_none());
    assert_eq!(v.ledger.lines().len(), 1);
    assert_eq!(v.checklist.len(), 8);
    std::fs::write(
        dir.path().join("coverage.json"),
        r#"{"schema_version":1,"workers":{"jvm":{"runtime":"jvm","backends":{"mat":{"status":"missing","install_hint":"install a JDK","enumerated":0,"dispositioned":0}}}},"residual":[],"blind_spots":[{"runtime":"jvm","text":"native memory"}]}"#,
    )
    .unwrap();
    let v = RunView::load(dir.path()).unwrap();
    let cov = v.coverage.unwrap();
    assert_eq!(cov.blind_spots[0].text, "native memory");
    assert_eq!(
        cov.workers["jvm"].backends["mat"].install_hint.as_deref(),
        Some("install a JDK")
    );
}
