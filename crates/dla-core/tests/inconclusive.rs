//! The reason `inconclusive`: measured, and the measurement cannot decide.
//!
//! It is accepted for `unresolved` only, and only when a measurement is tied to the
//! finding's family; a harness run alone, a whole-process growth column and evidence cited
//! only on the resolve line do not count. Ledger version 7.

mod common;

use common::*;
use dla_core::{
    AnswerStatus, Authority as A, ChecklistAnswer, Error, EvidenceKind as K, FindingState as S,
    Ledger, NewFinding, Refusal, Resolution, UnresolvedReason as R, error::codes, gate,
    record::RESIDUAL_OPENED_BY,
};
use serde_json::json;

fn refusal<T>(r: Result<T, Error>) -> Refusal {
    match r {
        Err(Error::Refused(r)) => r,
        Err(e) => panic!("expected a refusal, got {e}"),
        Ok(_) => panic!("expected a refusal, got Ok"),
    }
}

fn close(l: &mut Ledger, id: &dla_core::FindingId, reason: R) -> Result<(), Error> {
    l.transition(
        id,
        S::Unresolved,
        Some(reason),
        Some("x".into()),
        vec![],
        "t",
    )
    .map(|_| ())
}

/// A ledger with E1 static, a workload record (E2) and a plain finding F1 on E1 and E2.
fn base() -> (Ledger, dla_core::FindingId) {
    let mut l = Ledger::from_text("").unwrap();
    l.add_evidence(evidence(K::StaticHit, A::Static)).unwrap();
    l.add_evidence(evidence_with(
        K::Workload,
        A::Test,
        json!({"harness": "H1", "mode": "command", "runtime": "go"}),
    ))
    .unwrap();
    l.add_finding(NewFinding::new(origin("census"), vec![eid(1), eid(2)]))
        .unwrap();
    (l, fid(1))
}

fn growth(passed: serde_json::Value) -> dla_core::NewEvidence {
    evidence_with(
        K::GrowthSeries,
        A::Test,
        json!({"trend": "T1", "series": "S1", "n": 8, "passed": passed, "failed": 0, "gate": "pass"}),
    )
}

#[test]
fn a_harness_run_alone_is_not_a_measurement_of_the_finding() {
    let (mut l, f) = base();
    let r = refusal(close(&mut l, &f, R::Inconclusive));
    assert_eq!(r.code, codes::MEASUREMENT_REQUIRED);
    // The fix names the measured routes, or the reason that is true when nothing reached it.
    assert!(r.fix.contains("dla alloc-test"), "{}", r.fix);
    assert!(r.fix.contains("--reason not_exercised"), "{}", r.fix);
    assert_eq!(l.supported_reason(&f), R::NotExercised);
    // not_exercised is what the evidence supports, and it is accepted.
    close(&mut l, &f, R::NotExercised).unwrap();
}

#[test]
fn whole_process_growth_and_cited_only_evidence_do_not_count() {
    let (mut l, f) = base();
    // A passing growth column of the whole process, tied to no finding.
    let e = push(
        &mut l,
        growth(json!([{"column": "os.rss", "region": "process", "shape": "steady"}])),
    );
    // Cited only on the resolve line.
    let r = refusal(l.transition(
        &f,
        S::Unresolved,
        Some(R::Inconclusive),
        Some("x".into()),
        vec![eid(e)],
        "t",
    ));
    assert_eq!(r.code, codes::MEASUREMENT_REQUIRED, "{r}");
}

#[test]
fn a_discriminator_for_the_family_makes_it_measured() {
    let (mut l, f) = base();
    // A discriminator bound to another finding does not.
    push(
        &mut l,
        evidence_with(
            K::Discriminator,
            A::Test,
            json!({"test": "plateau", "favors": "inconclusive", "supports": [], "finding": "F9"}),
        ),
    );
    assert_eq!(
        refusal(close(&mut l, &f, R::Inconclusive)).code,
        codes::MEASUREMENT_REQUIRED
    );
    // One bound to F1 does, and so does one bound to a sibling of F1.
    push(
        &mut l,
        evidence_with(
            K::Discriminator,
            A::Test,
            json!({"test": "plateau", "favors": "inconclusive", "supports": [], "finding": "F1.n1"}),
        ),
    );
    // F1.n1 must exist for the family to hold it.
    let n = add_sibling(&mut l, &f, "N-WARM");
    assert_eq!(n.to_string(), "F1.n1");
    assert_eq!(l.supported_reason(&f), R::Inconclusive);
    close(&mut l, &f, R::Inconclusive).unwrap();
    assert_eq!(
        l.finding(&f).unwrap().state_label(),
        "unresolved(inconclusive)"
    );
}

#[test]
fn a_growth_column_tied_to_the_finding_makes_it_measured() {
    let (mut l, f) = base();
    push(
        &mut l,
        growth(
            json!([{"column": "probe:CACHE", "region": "heap", "site": "a.go:1",
            "finding": "F1", "shape": "steady"}]),
        ),
    );
    close(&mut l, &f, R::Inconclusive).unwrap();
}

#[test]
fn a_residual_record_the_family_already_cites_makes_it_measured() {
    let (mut l, _) = base();
    let e = push(
        &mut l,
        evidence_with(
            K::Residual,
            A::Test,
            json!({"region": "non-heap", "residual": 28}),
        ),
    );
    let mut nf = NewFinding::new(origin("residual"), vec![eid(e)]);
    nf.region = Some(dla_core::finding::Region::NonHeap);
    l.add_finding(nf).unwrap();
    let f2 = fid(2);
    close(&mut l, &f2, R::Inconclusive).unwrap();
    // A static record in the supports is not a measurement.
    let (mut l, f) = base();
    assert_eq!(
        refusal(close(&mut l, &f, R::Inconclusive)).code,
        codes::MEASUREMENT_REQUIRED
    );
}

#[test]
fn inconclusive_is_refused_for_suspected_static_only() {
    let (mut l, f) = base();
    push(
        &mut l,
        evidence_with(K::NonHeap, A::Test, json!({"region": "non-heap"})),
    );
    let mut nf = NewFinding::new(origin("residual"), vec![eid(1), eid(3)]);
    nf.region = Some(dla_core::finding::Region::NonHeap);
    l.add_finding(nf).unwrap();
    let g = fid(2);
    let r = refusal(l.transition(
        &g,
        S::SuspectedStaticOnly,
        Some(R::Inconclusive),
        Some("x".into()),
        vec![],
        "t",
    ));
    assert_eq!(r.code, codes::INVALID_VALUE);
    assert!(r.message.contains("never exercised"), "{}", r.message);
    let _ = f;
}

#[test]
fn a_version_6_line_cannot_carry_inconclusive() {
    let (mut l, f) = base();
    push(
        &mut l,
        growth(
            json!([{"column": "probe:CACHE", "region": "heap", "site": "a.go:1",
            "finding": "F1", "shape": "steady"}]),
        ),
    );
    close(&mut l, &f, R::Inconclusive).unwrap();
    let text: String = l
        .lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect();
    // Written as version 7, it replays.
    assert!(Ledger::from_text(&text).is_ok());
    // The same last line written as version 6 is refused on replay: an older build
    // would not have checked the rule.
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let n = lines.len();
    lines[n - 1] = lines[n - 1].replace("\"schema_version\":7", "\"schema_version\":6");
    let Err(Error::Replay(r)) = Ledger::from_text(&(lines.join("\n") + "\n")) else {
        panic!("a v6 line cannot carry inconclusive")
    };
    assert_eq!(r.refusal.code, codes::INVALID_VALUE);
    assert!(
        r.refusal.message.contains("schema 7"),
        "{}",
        r.refusal.message
    );
}

#[test]
fn open_items_take_inconclusive_when_a_measurement_opened_them() {
    let (mut l, f) = base();
    l.open_item_add("residual", None, RESIDUAL_OPENED_BY)
        .unwrap();
    l.open_item_add("by hand", None, "manual").unwrap();
    l.open_item_add("about F1", Some(f.clone()), "dla ledger merge")
        .unwrap();
    let unresolved = |reason| Resolution::Unresolved {
        reason,
        remedy: "x".into(),
    };
    let o = |n: &str| n.parse::<dla_core::OpenItemId>().unwrap();
    // Opened by `dla gap --residual`: a measurement.
    l.resolve_open_item(&o("O1"), unresolved(R::Inconclusive))
        .unwrap();
    // Opened by hand: refused.
    let r = refusal(l.resolve_open_item(&o("O2"), unresolved(R::Inconclusive)));
    assert_eq!(r.code, codes::MEASUREMENT_REQUIRED);
    // About a finding nothing measured: refused, then accepted once it is measured.
    let r = refusal(l.resolve_open_item(&o("O3"), unresolved(R::Inconclusive)));
    assert_eq!(r.code, codes::MEASUREMENT_REQUIRED);
    assert_eq!(l.open_item_supported_reason(&o("O3")), R::NotExercised);
    push(
        &mut l,
        growth(
            json!([{"column": "probe:CACHE", "region": "heap", "site": "a.go:1",
            "finding": "F1", "shape": "steady"}]),
        ),
    );
    assert_eq!(l.open_item_supported_reason(&o("O3")), R::Inconclusive);
    l.resolve_open_item(&o("O3"), unresolved(R::Inconclusive))
        .unwrap();
}

#[test]
fn a_checklist_answer_refuses_inconclusive() {
    let (mut l, _) = base();
    let r = refusal(l.answer_checklist(ChecklistAnswer {
        item: "intake.workload".into(),
        status: AnswerStatus::Unresolved,
        evidence: vec![],
        note: None,
        reason: Some(R::Inconclusive),
        remedy: Some("x".into()),
    }));
    assert_eq!(r.code, codes::INVALID_VALUE);
    assert!(r.message.contains("not measured"), "{}", r.message);
    // Without a remedy the refusal is the same, and its fix does not lead to a second one.
    let r = refusal(l.answer_checklist(ChecklistAnswer {
        item: "intake.workload".into(),
        status: AnswerStatus::Unresolved,
        evidence: vec![],
        note: None,
        reason: Some(R::Inconclusive),
        remedy: None,
    }));
    assert_eq!(r.code, codes::INVALID_VALUE);
    assert!(!r.fix.contains("--reason inconclusive"), "{}", r.fix);
    // Another reason is fine.
    l.answer_checklist(ChecklistAnswer {
        item: "intake.workload".into(),
        status: AnswerStatus::Unresolved,
        evidence: vec![],
        note: None,
        reason: Some(R::EnvUnavailable),
        remedy: Some("x".into()),
    })
    .unwrap();
}

#[test]
fn the_gate_decides_every_reason_explicitly() {
    // `inconclusive` proves something ran, so it is not a close without exercise.
    for (reason, closes) in [
        (R::NotExercised, true),
        (R::NoHarness, true),
        (R::TimeBudget, true),
        (R::EnvUnavailable, false),
        (R::BuildFails, false),
        (R::PrivilegeDeclined, false),
        (R::ToolMissing, false),
        (R::Inconclusive, false),
    ] {
        assert_eq!(gate::closes_without_exercise(reason), closes, "{reason}");
    }
    // The enum has no variant this table misses.
    assert_eq!(R::ALL.len(), 8);
    assert_eq!(R::ALL.last(), Some(&R::Inconclusive));
}

#[test]
fn a_nonleak_split_in_the_counts_keeps_by_state_whole() {
    let mut l = m3_ledger();
    let f = add_f1(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    let v = dla_core::RunView::new(run_state(), l, None);
    let c = dla_core::status(&v).counts;
    assert_eq!(c.enumerated, 2);
    assert_eq!(c.by_state.candidate, 1);
    assert_eq!(c.by_state.refuted, 1);
    assert_eq!(c.non_leak.refuted, 1);
    assert_eq!(c.non_leak.candidate, 0);
    let leak = c.by_state.minus(&c.non_leak);
    assert_eq!((leak.candidate, leak.refuted), (1, 0));
}

/// A ledger with a static hit E1, a growth record E2 and a finding F1 made from both (the
/// shape of a finding added with `--origin growth-series --evidence E2`).
fn finding_from_growth(passed: serde_json::Value) -> (Ledger, dla_core::FindingId) {
    let mut l = Ledger::from_text("").unwrap();
    l.add_evidence(evidence(K::StaticHit, A::Static)).unwrap();
    l.add_evidence(growth(passed)).unwrap();
    l.add_finding(NewFinding::new(
        origin("growth-series"),
        vec![eid(1), eid(2)],
    ))
    .unwrap();
    (l, fid(1))
}

#[test]
fn a_finding_made_from_a_growth_record_is_measured_by_the_gates_tie_rule() {
    // The gate ties a finding that cites the growth evidence to every non-whole-process
    // column of it, passed or not, whatever the column's own finding says.
    for column in [
        "py.type:dict",
        "probe:Cache",
        "jvm.class:a.B.count",
        "metric:heap/a.B",
    ] {
        let (mut l, f) =
            finding_from_growth(json!([{"column": column, "region": "heap", "shape": "steady"}]));
        assert_eq!(l.supported_reason(&f), R::Inconclusive, "{column}");
        close(&mut l, &f, R::Inconclusive).unwrap();
    }
    // A trend that failed its gate has no passed column. A record that does not say what
    // its failed columns were (written before the count existed) is given the benefit of
    // the doubt.
    let (mut l, f) = finding_from_growth(json!([]));
    assert_eq!(l.supported_reason(&f), R::Inconclusive);
    close(&mut l, &f, R::Inconclusive).unwrap();
    // A record whose passed columns all measure the whole process ties to nothing.
    let (mut l, f) =
        finding_from_growth(json!([{"column": "os.rss", "region": "process", "shape": "steady"}]));
    assert_eq!(l.supported_reason(&f), R::NotExercised);
    assert_eq!(
        refusal(close(&mut l, &f, R::Inconclusive)).code,
        codes::MEASUREMENT_REQUIRED
    );
}

#[test]
fn a_failed_trend_of_whole_process_columns_ties_to_nothing() {
    // The shape `dla trend` writes for a sampled process whose os., fd. and cgroup.
    // columns all stayed flat: the gate failed, no column passed, and every failed column
    // measures the whole process. `dla ledger add --origin growth-series --evidence E2`
    // makes a finding from it; the gate ties nothing to it, so no measurement of the
    // finding exists and `inconclusive` is refused.
    let failed_trend = |beyond: Option<u32>| {
        let mut result = json!({
            "trend": "T1", "series": "S1", "n": 8, "passed": [], "failed": 23,
            "shapes": {"flat": 23}, "gate": "fail"
        });
        if let Some(n) = beyond {
            result["failed_beyond_process"] = json!(n);
        }
        let mut l = Ledger::from_text("").unwrap();
        l.add_evidence(evidence(K::StaticHit, A::Static)).unwrap();
        l.add_evidence(evidence_with(K::GrowthSeries, A::Live, result))
            .unwrap();
        l.add_finding(NewFinding::new(
            origin("growth-series"),
            vec![eid(1), eid(2)],
        ))
        .unwrap();
        (l, fid(1))
    };
    let (mut l, f) = failed_trend(Some(0));
    assert_eq!(l.supported_reason(&f), R::NotExercised);
    assert_eq!(
        refusal(close(&mut l, &f, R::Inconclusive)).code,
        codes::MEASUREMENT_REQUIRED
    );
    close(&mut l, &f, R::NotExercised).unwrap();
    // A failed column that is not a whole-process one is what the finding was made from.
    let (mut l, f) = failed_trend(Some(2));
    assert_eq!(l.supported_reason(&f), R::Inconclusive);
    close(&mut l, &f, R::Inconclusive).unwrap();
    // A record that does not count them keeps the benefit of the doubt.
    let (mut l, f) = failed_trend(None);
    assert_eq!(l.supported_reason(&f), R::Inconclusive);
    close(&mut l, &f, R::Inconclusive).unwrap();
}

#[test]
fn a_raw_sample_record_is_judged_by_the_columns_it_lists() {
    // The growth-series record of `dla sample` has no trend and no gate, only the column
    // names; the gate's tie rule never reaches it, so a process-wide list ties to nothing.
    let from_sample = |columns: serde_json::Value| {
        let mut l = Ledger::from_text("").unwrap();
        l.add_evidence(evidence(K::StaticHit, A::Static)).unwrap();
        l.add_evidence(evidence_with(
            K::GrowthSeries,
            A::Live,
            json!({"series": "S1", "ticks": 8, "columns": columns, "columns_total": 2}),
        ))
        .unwrap();
        l.add_finding(NewFinding::new(
            origin("growth-series"),
            vec![eid(1), eid(2)],
        ))
        .unwrap();
        (l, fid(1))
    };
    let (mut l, f) = from_sample(json!(["os.rss", "os.threads"]));
    assert_eq!(l.supported_reason(&f), R::NotExercised);
    assert_eq!(
        refusal(close(&mut l, &f, R::Inconclusive)).code,
        codes::MEASUREMENT_REQUIRED
    );
    // An empty column list says nothing either, and neither does a record with no list.
    let (l, f) = from_sample(json!([]));
    assert_eq!(l.supported_reason(&f), R::NotExercised);
    let mut l = Ledger::from_text("").unwrap();
    l.add_evidence(evidence(K::StaticHit, A::Static)).unwrap();
    l.add_evidence(evidence_with(
        K::GrowthSeries,
        A::Live,
        json!({"series": "S1", "ticks": 8}),
    ))
    .unwrap();
    l.add_finding(NewFinding::new(
        origin("growth-series"),
        vec![eid(1), eid(2)],
    ))
    .unwrap();
    assert_eq!(l.supported_reason(&fid(1)), R::NotExercised);
    // A listed column that is not whole-process is the measurement the finding rests on.
    let (mut l, f) = from_sample(json!(["os.rss", "probe:Cache"]));
    assert_eq!(l.supported_reason(&f), R::Inconclusive);
    close(&mut l, &f, R::Inconclusive).unwrap();
}

#[test]
fn a_discriminator_the_finding_cites_is_measured_without_a_finding_field() {
    // The bench `test_growth` shape: an alloc-test discriminator that names the trend and
    // its columns, cited by the finding, with no `finding` field of its own.
    let mut l = Ledger::from_text("").unwrap();
    l.add_evidence(evidence(K::StaticHit, A::Static)).unwrap();
    l.add_evidence(evidence_with(
        K::Discriminator,
        A::Test,
        json!({"test": "plateau", "favors": "inconclusive", "supports": [],
            "trend": "T1", "columns": [{"name": "probe:Cache"}]}),
    ))
    .unwrap();
    l.add_finding(NewFinding::new(origin("hypothesis"), vec![eid(1), eid(2)]))
        .unwrap();
    let f = fid(1);
    assert_eq!(l.supported_reason(&f), R::Inconclusive);
    close(&mut l, &f, R::Inconclusive).unwrap();
}

#[test]
fn a_residual_or_capture_cited_only_on_the_resolve_line_does_not_count() {
    for (kind, result) in [
        (K::Residual, json!({"region": "non-heap", "residual": 28})),
        (K::NonHeap, json!({"region": "non-heap"})),
    ] {
        let (mut l, f) = base();
        let e = push(&mut l, evidence_with(kind, A::Test, result));
        let r = refusal(l.transition(
            &f,
            S::Unresolved,
            Some(R::Inconclusive),
            Some("x".into()),
            vec![eid(e)],
            "t",
        ));
        assert_eq!(r.code, codes::MEASUREMENT_REQUIRED, "{kind:?}: {r}");
        assert_eq!(l.supported_reason(&f), R::NotExercised, "{kind:?}");
    }
}
