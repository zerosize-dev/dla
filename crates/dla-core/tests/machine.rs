mod common;

use common::*;
use dla_core::{
    Error, FindingState as S, UnresolvedReason,
    error::codes,
    machine::{allowed_targets, is_allowed},
};

/// Every state, so the allowed/illegal tables can be checked exhaustively.
const ALL: [S; 7] = [
    S::Candidate,
    S::UnderTest,
    S::Confirmed,
    S::Strong,
    S::Refuted,
    S::SuspectedStaticOnly,
    S::Unresolved,
];

#[test]
fn graph_matches_the_spec_exactly() {
    let allowed: &[(S, S)] = &[
        (S::Candidate, S::UnderTest),
        (S::Candidate, S::Refuted),
        (S::Candidate, S::SuspectedStaticOnly),
        (S::Candidate, S::Unresolved),
        (S::UnderTest, S::Confirmed),
        (S::UnderTest, S::Refuted),
        (S::UnderTest, S::SuspectedStaticOnly),
        (S::UnderTest, S::Unresolved),
        (S::Confirmed, S::Strong),
        (S::SuspectedStaticOnly, S::UnderTest),
        (S::SuspectedStaticOnly, S::Unresolved),
        (S::Unresolved, S::Candidate),
        (S::Unresolved, S::UnderTest),
    ];
    for from in ALL {
        for to in ALL {
            assert_eq!(
                is_allowed(from, to),
                allowed.contains(&(from, to)),
                "{from} -> {to}"
            );
        }
    }
    assert!(allowed_targets(S::Refuted).is_empty());
    assert!(allowed_targets(S::Strong).is_empty());
}

/// Drives a finding into `state` through legal moves on a fresh ledger.
fn finding_in(state: S) -> (dla_core::Ledger, dla_core::FindingId) {
    let mut l = ledger();
    let f = add_f1(&mut l);
    let mv = |l: &mut dla_core::Ledger, to, reason, remedy: Option<&str>, ev| {
        l.transition(&f, to, reason, remedy.map(String::from), ev, "test")
            .unwrap();
    };
    match state {
        S::Candidate => {}
        S::UnderTest => mv(&mut l, S::UnderTest, None, None, vec![]),
        S::Confirmed => {
            mv(&mut l, S::UnderTest, None, None, vec![]);
            mv(&mut l, S::Confirmed, None, None, vec![]);
        }
        S::Strong => {
            mv(&mut l, S::UnderTest, None, None, vec![]);
            mv(&mut l, S::Confirmed, None, None, vec![]);
            mv(&mut l, S::Strong, None, None, vec![]);
        }
        S::Refuted => mv(&mut l, S::Refuted, None, None, vec![eid(2)]),
        S::SuspectedStaticOnly => mv(
            &mut l,
            S::SuspectedStaticOnly,
            Some(UnresolvedReason::NotExercised),
            Some("run it"),
            vec![],
        ),
        S::Unresolved => mv(
            &mut l,
            S::Unresolved,
            Some(UnresolvedReason::TimeBudget),
            Some("more time"),
            vec![],
        ),
    }
    (l, f)
}

fn plain_move(state: S, to: S) -> Result<(), Error> {
    let (mut l, f) = finding_in(state);
    // Supply what the target needs so only the graph can refuse.
    let (reason, remedy, ev) = match to {
        S::Unresolved | S::SuspectedStaticOnly => (
            Some(UnresolvedReason::ToolMissing),
            Some("install it".to_owned()),
            vec![eid(1)],
        ),
        S::Refuted => (None, None, vec![eid(2)]),
        _ => (None, None, vec![]),
    };
    l.transition(&f, to, reason, remedy, ev, "test").map(|_| ())
}

#[test]
fn every_allowed_move_is_accepted() {
    for from in ALL {
        for &to in allowed_targets(from) {
            plain_move(from, to).unwrap_or_else(|e| panic!("{from} -> {to}: {e}"));
        }
    }
}

#[test]
fn every_illegal_move_is_refused_with_a_fix() {
    for from in ALL {
        for to in ALL {
            if is_allowed(from, to) {
                continue;
            }
            match plain_move(from, to) {
                Err(Error::Refused(r)) => {
                    assert_eq!(r.code, codes::ILLEGAL_TRANSITION, "{from} -> {to}");
                    assert!(r.fix.starts_with("dla "), "{from} -> {to}: {}", r.fix);
                    assert!(!r.missing.is_empty());
                }
                other => panic!("{from} -> {to}: expected refusal, got {other:?}"),
            }
        }
    }
}

fn refusal(r: Result<impl std::fmt::Debug, Error>) -> dla_core::Refusal {
    match r {
        Err(Error::Refused(r)) => r,
        other => panic!("expected refusal, got {other:?}"),
    }
}

#[test]
fn unresolved_needs_reason_and_remedy() {
    for to in [S::Unresolved, S::SuspectedStaticOnly] {
        let (mut l, f) = finding_in(S::Candidate);
        let r = refusal(l.transition(&f, to, None, Some("x".into()), vec![eid(1)], "t"));
        assert_eq!(r.code, codes::REASON_REQUIRED);
        assert!(r.fix.contains("--reason"));
        let r = refusal(l.transition(
            &f,
            to,
            Some(UnresolvedReason::NoHarness),
            None,
            vec![eid(1)],
            "t",
        ));
        assert_eq!(r.code, codes::REMEDY_REQUIRED);
        let r = refusal(l.transition(
            &f,
            to,
            Some(UnresolvedReason::NoHarness),
            Some("  ".into()),
            vec![eid(1)],
            "t",
        ));
        assert_eq!(r.code, codes::REMEDY_REQUIRED);
    }
}

#[test]
fn reason_on_other_states_is_refused() {
    let (mut l, f) = finding_in(S::Candidate);
    let r = refusal(l.transition(
        &f,
        S::UnderTest,
        Some(UnresolvedReason::TimeBudget),
        None,
        vec![],
        "t",
    ));
    assert_eq!(r.code, codes::UNEXPECTED_REASON);
}

#[test]
fn suspected_static_only_requires_static_evidence() {
    let mut l = ledger();
    // F1 is supported only by E3 (live growth series): no static evidence.
    l.add_finding(dla_core::NewFinding::new(origin("census"), vec![eid(3)]))
        .unwrap();
    let f = fid(1);
    let r = refusal(l.transition(
        &f,
        S::SuspectedStaticOnly,
        Some(UnresolvedReason::NotExercised),
        Some("exercise it".into()),
        vec![],
        "t",
    ));
    assert_eq!(r.code, codes::STATIC_EVIDENCE_REQUIRED);
    // Citing the static record E1 in the move satisfies the rule.
    l.transition(
        &f,
        S::SuspectedStaticOnly,
        Some(UnresolvedReason::NotExercised),
        Some("exercise it".into()),
        vec![eid(1)],
        "t",
    )
    .unwrap();
    assert!(l.finding(&f).unwrap().supports.contains(&eid(1)));
    assert_eq!(
        l.finding(&f).unwrap().state_label(),
        "suspected_static_only(not_exercised)"
    );
}

#[test]
fn refuted_requires_discriminator_in_contradicts() {
    let (mut l, f) = finding_in(S::Candidate);
    let r = refusal(l.transition(&f, S::Refuted, None, None, vec![], "t"));
    assert_eq!(r.code, codes::DISCRIMINATOR_REQUIRED);
    // E1 is a static-hit, not a discriminator.
    let r = refusal(l.transition(&f, S::Refuted, None, None, vec![eid(1)], "t"));
    assert_eq!(r.code, codes::DISCRIMINATOR_REQUIRED);
    l.transition(&f, S::Refuted, None, None, vec![eid(2)], "t")
        .unwrap();
    assert_eq!(l.finding(&f).unwrap().contradicts, vec![eid(2)]);
}

#[test]
fn unknown_evidence_and_findings_are_refused() {
    let (mut l, f) = finding_in(S::Candidate);
    let r = refusal(l.transition(&f, S::Refuted, None, None, vec![eid(99)], "t"));
    assert_eq!(r.code, codes::UNKNOWN_EVIDENCE);
    let r = refusal(l.transition(&fid(9), S::UnderTest, None, None, vec![], "t"));
    assert_eq!(r.code, codes::UNKNOWN_FINDING);
}

#[test]
fn leaving_unresolved_clears_reason_and_remedy() {
    let (mut l, f) = finding_in(S::Unresolved);
    assert!(l.finding(&f).unwrap().reason.is_some());
    l.transition(&f, S::UnderTest, None, None, vec![], "t")
        .unwrap();
    let fin = l.finding(&f).unwrap();
    assert!(fin.reason.is_none() && fin.remedy.is_none());
}

#[test]
fn rejected_moves_leave_the_ledger_unchanged() {
    let (mut l, f) = finding_in(S::Candidate);
    let before = l.lines().len();
    let _ = l.transition(&f, S::Confirmed, None, None, vec![], "t");
    assert_eq!(l.lines().len(), before);
    assert_eq!(l.finding(&f).unwrap().state, S::Candidate);
}

#[test]
fn run_config_alone_is_not_static_class_evidence() {
    use dla_core::{Authority, EvidenceKind};
    let mut l = ledger();
    // E4: authority static, but kind run-config (class Context).
    l.add_evidence(evidence(EvidenceKind::RunConfig, Authority::Static))
        .unwrap();
    l.add_finding(dla_core::NewFinding::new(origin("census"), vec![eid(4)]))
        .unwrap();
    let f = fid(1);
    let r = refusal(l.transition(
        &f,
        S::SuspectedStaticOnly,
        Some(UnresolvedReason::NotExercised),
        Some("exercise it".into()),
        vec![eid(4)],
        "t",
    ));
    assert_eq!(r.code, codes::STATIC_EVIDENCE_REQUIRED);
    assert!(r.missing.contains("static-hit") && r.missing.contains("census-holder"));
    assert!(r.message.contains("run-config"));
    assert!(r.fix.contains("static-hit or census-holder"));
    // A census-holder record qualifies.
    l.add_evidence(evidence(EvidenceKind::CensusHolder, Authority::Static))
        .unwrap();
    l.transition(
        &f,
        S::SuspectedStaticOnly,
        Some(UnresolvedReason::NotExercised),
        Some("exercise it".into()),
        vec![eid(5)],
        "t",
    )
    .unwrap();
}

#[test]
fn illegal_transition_fix_comes_from_the_source_state() {
    // unresolved -> unresolved: never suggests the refused command again.
    let (mut l, f) = finding_in(S::Unresolved);
    let r = refusal(l.transition(
        &f,
        S::Unresolved,
        Some(UnresolvedReason::TimeBudget),
        Some("x".into()),
        vec![],
        "t",
    ));
    assert_eq!(r.code, codes::ILLEGAL_TRANSITION);
    assert!(r.message.contains("already unresolved"), "{}", r.message);
    assert!(!r.fix.contains("--as unresolved"), "{}", r.fix);
    assert!(r.fix.contains("available from M3"), "{}", r.fix);

    // confirmed -> unresolved: only `strong` is allowed, and it arrives later.
    let (mut l, f) = finding_in(S::Confirmed);
    let r = refusal(l.transition(
        &f,
        S::Unresolved,
        Some(UnresolvedReason::TimeBudget),
        Some("x".into()),
        vec![],
        "t",
    ));
    assert!(!r.fix.contains("--as unresolved"), "{}", r.fix);
    assert!(
        r.fix.contains("strong") && r.fix.contains("M6"),
        "{}",
        r.fix
    );

    // suspected_static_only -> suspected_static_only: unresolved exists now.
    let (mut l, f) = finding_in(S::SuspectedStaticOnly);
    let r = refusal(l.transition(
        &f,
        S::SuspectedStaticOnly,
        Some(UnresolvedReason::NotExercised),
        Some("x".into()),
        vec![eid(1)],
        "t",
    ));
    assert!(r.fix.contains("--as unresolved"), "{}", r.fix);
    assert!(!r.fix.contains("--as suspected_static_only"), "{}", r.fix);
    assert!(r.fix.contains("available from M3"), "{}", r.fix);
}

#[test]
fn terminal_states_are_final_and_point_at_a_new_hypothesis() {
    for terminal in [S::Refuted, S::Strong] {
        let (mut l, f) = finding_in(terminal);
        let r = refusal(l.transition(&f, S::UnderTest, None, None, vec![], "t"));
        assert_eq!(r.code, codes::ILLEGAL_TRANSITION);
        assert!(r.message.contains("final"), "{}", r.message);
        assert!(r.fix.starts_with("dla ledger add --origin hypothesis"));
        assert!(r.fix.contains("--sibling-of F1"), "{}", r.fix);
    }
}

#[test]
fn refute_fix_names_the_milestone_it_arrives_in() {
    let (mut l, f) = finding_in(S::Candidate);
    let r = refusal(l.transition(&f, S::Refuted, None, None, vec![], "t"));
    assert!(r.fix.contains("available from M3"), "{}", r.fix);
}

#[test]
fn unknown_evidence_on_a_transition_lists_the_existing_ids() {
    let (mut l, f) = finding_in(S::Candidate);
    let r = refusal(l.transition(&f, S::Refuted, None, None, vec![eid(99)], "t"));
    assert_eq!(r.code, codes::UNKNOWN_EVIDENCE);
    assert!(r.fix.contains("E1, E2, E3"), "{}", r.fix);
    assert!(!r.fix.contains("run status"), "{}", r.fix);
}
