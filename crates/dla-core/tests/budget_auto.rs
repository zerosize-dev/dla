//! The `auto` budget, `dla run extend` and the numbers behind `dla run status` and the
//! report's `Time:` line, on the state and the ledger directly.

mod common;

use chrono::{DateTime, TimeDelta, Utc};
use common::{line, run_state};
use dla_core::{
    Ledger, RunState, RunView,
    state::{AUTO_START_SECS, BudgetEstimate, WorkerEntry, WorkerStatus, auto, now_utc},
    timebudget,
};

fn mins(n: i64) -> TimeDelta {
    TimeDelta::try_minutes(n).unwrap()
}

fn auto_state() -> RunState {
    let mut s = RunState::init("/p", Some("s"), AUTO_START_SECS, None, now_utc()).unwrap();
    s.budget.auto = true;
    s
}

fn worker(name: &str, deadline: DateTime<Utc>, status: WorkerStatus) -> WorkerEntry {
    WorkerEntry {
        name: name.into(),
        runtime: name.into(),
        share_secs: 1,
        deadline,
        started_at: None,
        status,
        slice: format!("ledger.{name}.jsonl"),
        stalled: false,
    }
}

/// A ledger of static hits (E1 python, E2 go) and `py` + `go` candidates, each with
/// its own site. `extra` lines follow (seq starts at the next number).
fn ledger(py: usize, go: usize, extra: &[&str]) -> Ledger {
    ledger_with(py, go, false, extra)
}

/// [`ledger`], with F3 at the site of F2 when `same_site` (so they can be duplicates).
fn ledger_with(py: usize, go: usize, same_site: bool, extra: &[&str]) -> Ledger {
    let mut text = String::new();
    let mut seq = 1;
    for (id, lang) in [("E1", "python"), ("E2", "go")] {
        text += &line(
            seq,
            &format!(
                "\"type\":\"evidence\",\"evidence_id\":\"{id}\",\"source\":{{\"repo\":{{\"commit\":\"abc\"}}}},\"tool\":{{\"name\":\"t\",\"version\":\"0\"}},\"authority\":\"static\",\"kind\":\"static-hit\",\"result\":{{\"lang\":\"{lang}\"}}"
            ),
        );
        seq += 1;
    }
    let mut n = 0;
    for (count, ev, ext) in [(py, "E1", "py"), (go, "E2", "go")] {
        for _ in 0..count {
            n += 1;
            let site = if same_site && n == 3 {
                "src/f2.py:2".to_owned()
            } else {
                format!("src/f{n}.{ext}:{n}")
            };
            text += &line(
                seq,
                &format!(
                    "\"type\":\"finding_added\",\"finding\":{{\"id\":\"F{n}\",\"state\":\"candidate\",\"origin\":\"census\",\"supports\":[\"{ev}\"],\"site\":\"{site}\"}}"
                ),
            );
            seq += 1;
        }
    }
    for e in extra {
        text += &line(seq, e);
        seq += 1;
    }
    Ledger::from_text(&text).unwrap()
}

fn view(state: RunState, l: Ledger) -> RunView {
    RunView::new(state, l, None)
}

// ------------------------------------------------------------------ the formula

#[test]
fn the_estimate_follows_the_formula_with_a_floor_and_a_cap() {
    // 3 python + 2 go candidates: R = 2, C = 5: 20 + 30 + 15 = 65.
    let e = timebudget::estimate(&view(auto_state(), ledger(3, 2, &[])), None);
    assert_eq!(
        (e.runtimes, e.candidates, e.minutes, e.total_minutes),
        (2, 5, 65, 65)
    );
    // One candidate: 20 + 15 + 3 = 38, raised to the 60-minute floor.
    let e = timebudget::estimate(&view(auto_state(), ledger(1, 0, &[])), None);
    assert_eq!(
        (e.runtimes, e.candidates, e.minutes, e.total_minutes),
        (1, 1, 38, 60)
    );
    // Nothing found: the floor.
    let e = timebudget::estimate(&view(auto_state(), ledger(0, 0, &[])), None);
    assert_eq!(
        (e.runtimes, e.candidates, e.minutes, e.total_minutes),
        (0, 0, 20, 60)
    );
    // 362 candidates of one runtime: 20 + 15 + 1086 = 1121, capped at 180.
    let e = timebudget::estimate(&view(auto_state(), ledger(362, 0, &[])), None);
    assert_eq!(
        (e.runtimes, e.candidates, e.minutes, e.total_minutes),
        (1, 362, 1121, 180)
    );
    // The constants are the documented ones.
    assert_eq!(
        (
            auto::BASE_MINUTES,
            auto::PER_RUNTIME_MINUTES,
            auto::PER_CANDIDATE_MINUTES,
            auto::FLOOR_MINUTES,
            auto::CAP_MINUTES
        ),
        (20, 15, 3, 60, 180)
    );
}

#[test]
fn duplicates_and_siblings_do_not_count_and_closed_findings_do() {
    // F1..F3 python at sites f1, f2, f2; F3 is merged into F2 (the same site), F1 is
    // closed, and F1 has a sibling hypothesis that is still open.
    let l = ledger_with(
        3,
        0,
        true,
        &[
            "\"type\":\"finding_merged\",\"canonical\":\"F2\",\"duplicate\":\"F3\"",
            "\"type\":\"finding_added\",\"finding\":{\"id\":\"F1.n1\",\"state\":\"candidate\",\"origin\":\"hypothesis\",\"supports\":[\"E1\"],\"parent\":\"F1\",\"hypothesis\":\"N-WARM\",\"refute_if\":\"growth continues\"}",
            "\"type\":\"transition\",\"finding\":\"F1\",\"from\":\"candidate\",\"to\":\"suspected_static_only\",\"evidence\":[],\"by\":\"t\",\"reason\":\"no_harness\",\"remedy\":\"x\"",
        ],
    );
    let e = timebudget::estimate(&view(auto_state(), l), None);
    // Top-level and not a duplicate, whatever the state: F1 (closed) and F2.
    assert_eq!((e.runtimes, e.candidates), (1, 2), "{e:?}");
}

#[test]
fn candidates_at_one_site_count_once_before_they_are_merged() {
    // At the sweep gate the census and a sweep rule each open a candidate at the same
    // holder; duplicates are merged only later. F2 and F3 share a site: C = 2, not 3.
    let l = ledger_with(3, 0, true, &[]);
    let e = timebudget::estimate(&view(auto_state(), l), None);
    assert_eq!((e.runtimes, e.candidates), (1, 2), "{e:?}");
}

#[test]
fn the_settled_line_names_the_cap_and_the_floor() {
    let capped = timebudget::settled_line(&BudgetEstimate::new(1, 362));
    assert_eq!(
        capped,
        "budget (auto): 3h for 1 runtime and 362 candidates (20 + 15x1 + 3x362 = 1121 min, capped at 3h)"
    );
    let plain = timebudget::settled_line(&BudgetEstimate::new(2, 5));
    assert_eq!(
        plain,
        "budget (auto): 1h5m for 2 runtimes and 5 candidates (20 + 15x2 + 3x5 = 65 min)"
    );
    let floor = timebudget::settled_line(&BudgetEstimate::new(1, 1));
    assert!(floor.contains("raised to 1h"), "{floor}");
}

// ------------------------------------------------------------------ settle

#[test]
fn settle_moves_the_deadline_to_the_sized_total_once() {
    let mut s = auto_state();
    let start = s.budget.started_at;
    let delta = s.settle_budget(BudgetEstimate::new(2, 30), None).unwrap();
    // 20 + 30 + 90 = 140 minutes: 80 more than the 60 it started with.
    assert_eq!(delta, 80 * 60);
    assert_eq!(s.budget.deadline, start + mins(140));
    assert_eq!(s.budget.seconds, 140 * 60);
    assert!(s.budget.settled);
    assert_eq!(s.budget.estimate.as_ref().unwrap().total_minutes, 140);
    // Settled once: a second call changes nothing, whatever it is told.
    let before = s.clone();
    assert_eq!(s.settle_budget(BudgetEstimate::new(1, 400), None), None);
    assert_eq!(s, before);
}

#[test]
fn settle_never_moves_the_deadline_earlier_and_a_fixed_budget_never_settles() {
    // Already past the sized total (an extension came first).
    let mut s = auto_state();
    s.extend_budget(now_utc(), 3 * 3600).unwrap();
    let deadline = s.budget.deadline;
    let delta = s.settle_budget(BudgetEstimate::new(1, 1), None).unwrap();
    assert_eq!(delta, 0);
    assert_eq!(s.budget.deadline, deadline);
    assert!(s.budget.settled);
    // A fixed budget: settle is a no-op, settled stays false.
    let mut f = run_state();
    let before = f.clone();
    assert_eq!(f.settle_budget(BudgetEstimate::new(3, 300), None), None);
    assert_eq!(f, before);
    assert!(!f.budget.settled && !f.budget.auto);
}

#[test]
fn settle_moves_every_unfinished_worker_deadline_by_the_same_delta() {
    let mut s = auto_state();
    let at = s.budget.started_at + mins(30);
    for (n, st) in [
        ("jvm", WorkerStatus::Registered),
        ("go", WorkerStatus::Running),
        ("python", WorkerStatus::Done),
        ("node", WorkerStatus::Killed),
    ] {
        s.workers.push(worker(n, at, st));
    }
    let delta = s.settle_budget(BudgetEstimate::new(1, 100), None).unwrap();
    // 20 + 15 + 300 = 335, capped at 180: 120 more minutes.
    assert_eq!(delta, 120 * 60);
    let by = |n: &str| s.workers.iter().find(|w| w.name == n).unwrap().deadline;
    assert_eq!(by("jvm"), at + mins(120));
    assert_eq!(by("go"), at + mins(120));
    assert_eq!(by("python"), at, "a finished worker keeps its deadline");
    assert_eq!(by("node"), at);
}

#[test]
fn workers_size_the_budget_in_turn_and_it_only_rises_until_every_worker_has() {
    let mut s = auto_state();
    let start = s.budget.started_at;
    for n in ["python", "go"] {
        s.workers
            .push(worker(n, start + mins(30), WorkerStatus::Running));
    }
    // The first worker sees 5 candidates: 20 + 30 + 15 = 65 minutes, not final yet.
    let d = s
        .settle_budget(BudgetEstimate::new(2, 5), Some("go"))
        .unwrap();
    assert_eq!(d, 5 * 60);
    assert!(!s.budget.settled);
    assert_eq!(s.budget.settled_workers, ["go"]);
    // The same worker again changes nothing.
    let before = s.clone();
    assert_eq!(
        s.settle_budget(BudgetEstimate::new(2, 50), Some("go")),
        None
    );
    assert_eq!(s, before);
    // The second sees 40 more: it raises the budget, and now every worker has sized it.
    let d = s
        .settle_budget(BudgetEstimate::new(2, 45), Some("python"))
        .unwrap();
    assert_eq!(d, (180 - 65) * 60, "20 + 30 + 135 minutes, capped at 3h");
    assert!(s.budget.settled);
    assert_eq!(s.budget.deadline, start + mins(180));
    assert_eq!(s.budget.estimate.as_ref().unwrap().candidates, 45);
    // Final: nothing more is accepted.
    assert_eq!(
        s.settle_budget(BudgetEstimate::new(2, 1), Some("python")),
        None
    );
}

#[test]
fn a_smaller_estimate_from_a_later_worker_never_lowers_the_budget() {
    let mut s = auto_state();
    let start = s.budget.started_at;
    for n in ["python", "go"] {
        s.workers
            .push(worker(n, start + mins(30), WorkerStatus::Running));
    }
    s.settle_budget(BudgetEstimate::new(2, 30), Some("python"))
        .unwrap();
    let deadline = s.budget.deadline;
    assert_eq!(deadline, start + mins(140));
    let d = s
        .settle_budget(BudgetEstimate::new(1, 2), Some("go"))
        .unwrap();
    assert_eq!(d, 0);
    assert_eq!(s.budget.deadline, deadline);
    assert_eq!(s.budget.estimate.as_ref().unwrap().total_minutes, 140);
    assert!(s.budget.settled, "both workers have sized it");
}

// ------------------------------------------------------------------ extend

#[test]
fn extend_from_a_deadline_in_the_future_adds_to_it() {
    let mut s = run_state();
    let now = now_utc();
    s.budget.deadline = now + mins(20);
    let d = s.extend_budget(now, 30 * 60).unwrap();
    assert_eq!(d, now + mins(50));
    assert_eq!(s.budget.deadline, d);
    assert_eq!(
        s.budget.seconds,
        (d - s.budget.started_at).num_seconds() as u64
    );
    assert_eq!(s.budget.extensions.len(), 1);
    assert_eq!(s.budget.extensions[0].seconds, 1800);
}

#[test]
fn extend_from_a_deadline_in_the_past_counts_from_now() {
    let mut s = run_state();
    let now = now_utc();
    s.budget.deadline = now - mins(45);
    let d = s.extend_budget(now, 30 * 60).unwrap();
    assert_eq!(d, now + mins(30), "the spent time is not given back");
}

#[test]
fn extend_moves_unfinished_workers_and_leaves_finished_ones() {
    let mut s = run_state();
    let now = now_utc();
    s.budget.deadline = now + mins(10);
    s.workers
        .push(worker("jvm", now + mins(5), WorkerStatus::Running));
    s.workers
        .push(worker("go", now - mins(5), WorkerStatus::Registered));
    s.workers
        .push(worker("node", now + mins(5), WorkerStatus::Done));
    let d = s.extend_budget(now, 60 * 60).unwrap();
    let by = |n: &str| s.workers.iter().find(|w| w.name == n).unwrap().deadline;
    assert_eq!(by("jvm"), now + mins(65));
    assert_eq!(
        by("go"),
        now + mins(60),
        "a worker past its deadline counts from now"
    );
    assert_eq!(by("node"), now + mins(5));
    assert!(by("jvm") <= d && by("go") <= d);
}

// ------------------------------------------------------------------ to finish

#[test]
fn to_finish_minutes_counts_open_and_time_budget_findings_and_runtimes_without_a_run() {
    // F1, F2 python open; F3, F4 go: F3 closed time_budget, F4 closed no_harness.
    let l = ledger(
        2,
        2,
        &[
            "\"type\":\"transition\",\"finding\":\"F3\",\"from\":\"candidate\",\"to\":\"unresolved\",\"evidence\":[],\"by\":\"t\",\"reason\":\"time_budget\",\"remedy\":\"x\"",
            "\"type\":\"transition\",\"finding\":\"F4\",\"from\":\"candidate\",\"to\":\"suspected_static_only\",\"evidence\":[],\"by\":\"t\",\"reason\":\"no_harness\",\"remedy\":\"x\"",
        ],
    );
    let r = timebudget::remaining(&view(auto_state(), l));
    // 3 findings (F1, F2, F3) x 3 + 2 runtimes without a harness run x 15.
    assert_eq!(
        (
            r.findings,
            r.runtimes_without_run,
            r.minutes,
            r.closed_for_time
        ),
        (3, 2, 39, 1)
    );
}

#[test]
fn a_finding_closed_static_only_for_time_still_needs_time() {
    // F1 open; F2 closed unresolved(time_budget); F3 closed suspected_static_only(time_budget);
    // F4 closed suspected_static_only(no_harness), which is not a lack of time.
    let l = ledger(
        4,
        0,
        &[
            "\"type\":\"transition\",\"finding\":\"F2\",\"from\":\"candidate\",\"to\":\"unresolved\",\"evidence\":[],\"by\":\"t\",\"reason\":\"time_budget\",\"remedy\":\"x\"",
            "\"type\":\"transition\",\"finding\":\"F3\",\"from\":\"candidate\",\"to\":\"suspected_static_only\",\"evidence\":[],\"by\":\"t\",\"reason\":\"time_budget\",\"remedy\":\"x\"",
            "\"type\":\"transition\",\"finding\":\"F4\",\"from\":\"candidate\",\"to\":\"suspected_static_only\",\"evidence\":[],\"by\":\"t\",\"reason\":\"no_harness\",\"remedy\":\"x\"",
        ],
    );
    let r = timebudget::remaining(&view(auto_state(), l));
    // F1, F2, F3 x 3 + one runtime x 15; of those, the two closed ones cost 2 x 3 + 15.
    assert_eq!(
        (r.findings, r.closed_for_time, r.minutes, r.closed_minutes),
        (3, 2, 24, 21)
    );
}

#[test]
fn an_extension_made_before_sizing_stays_on_top_of_the_sized_total() {
    let mut s = auto_state();
    let now = now_utc();
    let started = s.budget.started_at;
    s.extend_budget(now, 30 * 60).unwrap();
    let before = s.budget.deadline;
    // 20 + 15 + 3*30 = 125 minutes, over the 60 + 30 already held.
    let e = BudgetEstimate::new(1, 30);
    assert_eq!(e.total_minutes, 125);
    s.settle_budget(e, None).unwrap();
    assert_eq!(s.budget.deadline, started + mins(125 + 30));
    assert!(s.budget.deadline > before);
    assert_eq!(s.budget.seconds, (125 + 30) * 60);
}

#[test]
fn a_runtime_with_a_successful_harness_run_costs_no_build_time() {
    let l = ledger(2, 2, &[]);
    let cov: dla_core::CoverageManifest = serde_json::from_value(serde_json::json!({
        "schema_version": 1,
        "workers": {"python": {"runtime": "python", "iterations": 500}}
    }))
    .unwrap();
    let r = timebudget::remaining(&RunView::new(auto_state(), l, Some(cov)));
    assert_eq!((r.findings, r.runtimes_without_run, r.minutes), (4, 1, 27));
}

#[test]
fn nothing_to_finish_is_zero() {
    let r = timebudget::remaining(&view(auto_state(), ledger(0, 0, &[])));
    assert_eq!((r.findings, r.minutes), (0, 0));
}

// ------------------------------------------------------------------ old states

#[test]
fn a_state_written_before_the_auto_budget_still_loads() {
    let s = run_state();
    let mut v = serde_json::to_value(&s).unwrap();
    for k in ["auto", "settled", "estimate", "extensions"] {
        v["budget"].as_object_mut().unwrap().remove(k);
    }
    let back: RunState = serde_json::from_value(v).unwrap();
    assert!(!back.budget.auto && !back.budget.settled);
    assert!(back.budget.estimate.is_none() && back.budget.extensions.is_empty());
}
