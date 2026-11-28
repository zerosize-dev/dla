//! `--evidence` of `dla run abort`: only a failure the run recorded counts.

mod common;

use common::*;
use dla_core::{
    AnswerStatus, ChecklistAnswer, RunView, UnresolvedReason,
    abort_rule::{self, OpenWork},
    harness::{AttemptOutcome, HarnessAttempt},
};
use serde_json::json;
use std::{fs, path::Path};

fn open() -> OpenWork {
    OpenWork {
        left: 160,
        total: 180,
        queues: vec!["25 of 78 candidates targeted".into()],
        growths: Vec::new(),
        first: "dla harness gen --lang python --driver --next".into(),
    }
}

fn attempt(dir: &Path, h: &str, outcome: AttemptOutcome, reason: Option<&str>) {
    HarnessAttempt {
        harness: h.into(),
        started_at: HarnessAttempt::now(),
        finished_at: HarnessAttempt::now(),
        outcome,
        reason: reason.map(Into::into),
        message: None,
    }
    .save(dir)
    .unwrap();
}

fn run_record(dir: &Path, h: &str, first: &str, doc: serde_json::Value) {
    fs::write(
        dir.join("harness")
            .join(h)
            .join(format!("run-{first}.json")),
        doc.to_string(),
    )
    .unwrap();
}

fn answer(
    l: &mut dla_core::Ledger,
    item: &str,
    status: AnswerStatus,
    reason: Option<UnresolvedReason>,
    runtime: Option<&str>,
) {
    let answered = status != AnswerStatus::Unresolved;
    l.answer_checklist(ChecklistAnswer {
        item: item.into(),
        status,
        evidence: if answered { vec![eid(1)] } else { vec![] },
        note: None,
        reason,
        remedy: (!answered).then(|| "install it".to_owned()),
        runtime: runtime.map(Into::into),
    })
    .unwrap();
}

#[test]
fn a_harness_whose_attempt_ran_is_no_failure() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("harness/H1")).unwrap();
    attempt(dir.path(), "H1", AttemptOutcome::Ran, None);
    assert_eq!(abort_rule::harness_failure(dir.path(), "H1"), None);
    attempt(dir.path(), "H1", AttemptOutcome::Partial, None);
    assert_eq!(abort_rule::harness_failure(dir.path(), "H1"), None);
}

#[test]
fn a_harness_whose_latest_attempt_failed_is_a_failure() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("harness/H1")).unwrap();
    attempt(
        dir.path(),
        "H1",
        AttemptOutcome::Failed,
        Some("build_fails"),
    );
    let why = abort_rule::harness_failure(dir.path(), "H1").unwrap();
    assert!(why.contains("H1") && why.contains("build_fails"), "{why}");
    // A later attempt that ran replaces the record: the failure is no longer the latest.
    attempt(dir.path(), "H1", AttemptOutcome::Ran, None);
    assert_eq!(abort_rule::harness_failure(dir.path(), "H1"), None);
}

#[test]
fn a_run_record_with_a_non_zero_exit_or_a_failed_build_is_a_failure() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("harness/H1")).unwrap();
    attempt(dir.path(), "H1", AttemptOutcome::Ran, None);
    run_record(
        dir.path(),
        "H1",
        "S1",
        json!({"exits": [{"label": "p1", "code": 0}], "limitations": ["a note"]}),
    );
    assert_eq!(abort_rule::harness_failure(dir.path(), "H1"), None);
    run_record(
        dir.path(),
        "H1",
        "S1",
        json!({"exits": [{"label": "p1", "code": 2}, {"label": "p2", "code": 3}], "limitations": []}),
    );
    let why = abort_rule::harness_failure(dir.path(), "H1").unwrap();
    assert!(why.contains("exit with code 2"), "{why}");
    run_record(
        dir.path(),
        "H1",
        "S1",
        json!({"exits": [{"label": "p1", "code": 0}], "limitations": ["the instrumented build for p1 failed (log x)"]}),
    );
    let why = abort_rule::harness_failure(dir.path(), "H1").unwrap();
    assert!(why.contains("build that failed"), "{why}");
}

#[test]
fn an_item_is_a_failure_when_its_answer_says_something_is_missing_or_refused() {
    let mut l = ledger();
    assert_eq!(
        abort_rule::item_failure(&l, "intake.build"),
        None,
        "no answer"
    );
    answer(&mut l, "intake.build", AnswerStatus::Answered, None, None);
    assert_eq!(
        abort_rule::item_failure(&l, "intake.build"),
        None,
        "answered"
    );
    for reason in abort_rule::EVIDENCE_ITEM_REASONS {
        answer(
            &mut l,
            "intake.workload",
            AnswerStatus::Unresolved,
            Some(reason),
            None,
        );
        let why = abort_rule::item_failure(&l, "intake.workload").unwrap();
        assert!(why.contains(reason.as_str()), "{why}");
    }
    // Reasons that name nothing missing or refused are no failure.
    for reason in [
        UnresolvedReason::TimeBudget,
        UnresolvedReason::NotExercised,
        UnresolvedReason::NoHarness,
    ] {
        answer(
            &mut l,
            "intake.workload",
            AnswerStatus::Unresolved,
            Some(reason),
            None,
        );
        assert_eq!(
            abort_rule::item_failure(&l, "intake.workload"),
            None,
            "{reason:?}"
        );
    }
    // The newest answer of a runtime wins: answered after blocked is no failure now.
    answer(
        &mut l,
        "intake.scope",
        AnswerStatus::Unresolved,
        Some(UnresolvedReason::ToolMissing),
        Some("go"),
    );
    assert!(abort_rule::item_failure(&l, "intake.scope").is_some());
    answer(
        &mut l,
        "intake.scope",
        AnswerStatus::Answered,
        None,
        Some("go"),
    );
    assert_eq!(abort_rule::item_failure(&l, "intake.scope"), None);
}

#[test]
fn the_evidence_check_refuses_what_names_no_failure_and_says_why() {
    let dir = tempfile::tempdir().unwrap();
    for h in ["H1", "H2"] {
        fs::create_dir_all(dir.path().join("harness").join(h)).unwrap();
        attempt(dir.path(), h, AttemptOutcome::Ran, None);
    }
    let mut l = ledger();
    answer(&mut l, "intake.build", AnswerStatus::Answered, None, None);
    let view = RunView::new(run_state(), l, None);
    let go = |e: Option<&str>| abort_rule::check_evidence(&view, dir.path(), "tool", e, &open());
    for e in [
        None,
        Some(""),
        Some("H9"),
        Some("nonsense"),
        Some("H1"),
        Some("intake.build"),
    ] {
        let (message, fix) = go(e).expect_err(&format!("{e:?}"));
        assert!(
            message.contains("No command of this run failed for want of a tool"),
            "{message}"
        );
        assert!(
            message.contains("the 2 harness attempts of this run all ran"),
            "{message}"
        );
        assert!(message.contains("not a blocker"), "{message}");
        assert!(
            message.contains("dla harness gen --lang python --driver --next"),
            "{message}"
        );
        assert!(
            message.contains("run the abort with --evidence"),
            "{message}"
        );
        assert_eq!(fix, abort_rule::evidence_refusal_fix(&open().first));
    }
    // A failure of another harness does not make H1 one, and the summary names it.
    attempt(
        dir.path(),
        "H2",
        AttemptOutcome::Failed,
        Some("build_fails"),
    );
    let (message, _) = go(Some("H1")).unwrap_err();
    assert!(
        message.contains("1 of the 2 harness attempts of this run failed (H2)"),
        "{message}"
    );
    assert!(go(Some("H2")).is_ok());
}

#[test]
fn the_evidence_check_takes_a_failed_harness_or_a_blocked_item() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("harness/H1")).unwrap();
    attempt(
        dir.path(),
        "H1",
        AttemptOutcome::Failed,
        Some("build_fails"),
    );
    let mut l = ledger();
    answer(
        &mut l,
        "intake.build",
        AnswerStatus::Unresolved,
        Some(UnresolvedReason::ToolMissing),
        None,
    );
    let view = RunView::new(run_state(), l, None);
    for kind in abort_rule::EVIDENCE_KINDS {
        for e in ["H1", " intake.build "] {
            let found =
                abort_rule::check_evidence(&view, dir.path(), kind, Some(e), &open()).unwrap();
            assert!(found.contains(e.trim()), "{found}");
        }
    }
}

#[test]
fn the_real_reason_of_an_early_stop_names_no_blocker() {
    let reason = "Each queue batch needs a hand-written driver; 12 batches ran (25 of 78 candidates targeted, 3 caches measured), 61 candidates remain and missed targets keep re-queueing. Stopped without finishing the queue.";
    for kind in abort_rule::EVIDENCE_KINDS {
        assert!(
            abort_rule::check_blocker(kind, Some(reason), None, Some("H1")).is_err(),
            "{kind}"
        );
    }
}

#[test]
fn evidence_goes_with_the_four_kinds_only_and_is_kept_in_the_reason() {
    for kind in ["user", "refusal"] {
        let (code, _, fix) = abort_rule::check_blocker(
            kind,
            Some("the capture needs a privilege the run does not have"),
            (kind == "refusal").then_some("time-left"),
            Some("H1"),
        )
        .unwrap_err();
        assert_eq!(code, "invalid-value", "{kind}");
        assert!(fix.starts_with("dla run abort --blocker "), "{fix}");
    }
    let b = abort_rule::check_blocker(
        "tool",
        Some("the go toolchain is not installed"),
        None,
        Some(" H3 "),
    )
    .unwrap();
    assert_eq!(b.evidence.as_deref(), Some("H3"));
    assert_eq!(
        b.incomplete_reason(),
        "aborted with blocker tool (evidence H3): the go toolchain is not installed"
    );
}

#[test]
fn an_attempt_that_failed_for_time_hooks_or_interrupt_is_no_failure() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("harness/H1")).unwrap();
    for reason in ["time_budget", "hooks_unfilled", "interrupted", "no_harness"] {
        attempt(dir.path(), "H1", AttemptOutcome::Failed, Some(reason));
        assert_eq!(
            abort_rule::harness_failure(dir.path(), "H1"),
            None,
            "{reason}"
        );
    }
}

#[test]
fn one_bad_exit_among_good_ones_is_no_failure() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("harness/H1")).unwrap();
    run_record(
        dir.path(),
        "H1",
        "S1",
        json!({"exits": [{"label": "p1", "code": 0}, {"label": "p2", "code": 137}], "limitations": []}),
    );
    assert_eq!(abort_rule::harness_failure(dir.path(), "H1"), None);
    run_record(
        dir.path(),
        "H1",
        "S1",
        json!({"exits": [{"label": "p1", "code": 2}, {"label": "p2", "code": 137}], "limitations": []}),
    );
    assert!(abort_rule::harness_failure(dir.path(), "H1").is_some());
}

#[test]
fn a_later_answer_with_no_runtime_covers_every_runtime() {
    let mut l = ledger();
    answer(
        &mut l,
        "intake.workload",
        AnswerStatus::Unresolved,
        Some(UnresolvedReason::EnvUnavailable),
        Some("jvm"),
    );
    assert!(abort_rule::item_failure(&l, "intake.workload").is_some());
    answer(
        &mut l,
        "intake.workload",
        AnswerStatus::Answered,
        None,
        None,
    );
    assert_eq!(abort_rule::item_failure(&l, "intake.workload"), None);
}

#[test]
fn the_refusal_text_matches_what_was_found() {
    let dir = tempfile::tempdir().unwrap();
    for h in ["H1", "H2", "H3"] {
        fs::create_dir_all(dir.path().join("harness").join(h)).unwrap();
    }
    attempt(
        dir.path(),
        "H1",
        AttemptOutcome::Failed,
        Some("build_fails"),
    );
    attempt(dir.path(), "H2", AttemptOutcome::Ran, None);
    let view = RunView::new(run_state(), ledger(), None);
    let go = |e: Option<&str>, kind: &str| {
        abort_rule::check_evidence(&view, dir.path(), kind, e, &open())
            .unwrap_err()
            .0
    };
    // A failed attempt exists: no "No command ... failed", and the harness is named as evidence.
    let m = go(None, "tool");
    assert!(!m.contains("No command of this run failed"), "{m}");
    assert!(
        m.contains("1 of the 2 harness attempts of this run failed (H1)"),
        "{m}"
    );
    assert!(m.contains("--evidence H1"), "{m}");
    // A harness never run says so and names the command that runs it.
    let m = go(Some("H3"), "tool");
    assert!(
        m.contains("H3 was generated and never run") && m.contains("dla harness run H3"),
        "{m}"
    );
    assert!(!m.contains("ran with exit 0"), "{m}");
    // The article follows the kind.
    fs::remove_file(dir.path().join("harness/H1/attempt.json")).unwrap();
    let m = go(None, "environment");
    assert!(m.contains("for want of an environment"), "{m}");
}
