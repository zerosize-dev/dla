//! An owner is set once: an annotation fills an owner that is not set and refuses to change
//! one that is, with or without evidence. The fix is a new finding and `dla ledger duplicate`.

mod common;

use common::*;
use dla_core::{
    Authority as A, Error, EvidenceKind as K, FindingId, Ledger, NewEvidence, Refusal, Tool,
    error::codes, record::FindingAnnotation,
};
use serde_json::{Value, json};

fn refusal<T: std::fmt::Debug>(r: Result<T, Error>) -> Refusal {
    match r {
        Err(Error::Refused(r)) => r,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn text(l: &Ledger) -> String {
    l.lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect()
}

fn reading(tool: &str, kind: K, authority: A, result: Value) -> NewEvidence {
    let mut e = evidence_with(kind, authority, result);
    e.tool = Tool {
        name: tool.into(),
        version: "0".into(),
    };
    e
}

fn growth(columns: &[&str]) -> Value {
    let passed: Vec<Value> = columns
        .iter()
        .map(|c| json!({"column": c, "region": "heap", "shape": "steady"}))
        .collect();
    json!({"trend": "T1", "series": "S1", "n": 5, "passed": passed, "failed": 0, "gate": "pass"})
}

fn add(l: &mut Ledger, e: NewEvidence) -> dla_core::EvidenceId {
    match l.add_evidence(e).unwrap().record {
        dla_core::Record::Evidence(r) => r.evidence_id,
        _ => unreachable!(),
    }
}

fn set_owner(
    l: &mut Ledger,
    f: &FindingId,
    owner: &str,
    ev: &[dla_core::EvidenceId],
) -> Result<(), Error> {
    let mut a = FindingAnnotation::new(f.clone());
    a.owner = Some(owner.to_owned());
    a.evidence = ev.to_vec();
    l.annotate_record(a).map(|_| ())
}

/// A ledger with a finding (F2) whose owner is the wrong `src.board.Board.publish.previous`.
fn setup() -> (Ledger, FindingId) {
    let mut l = m3_ledger();
    let _tied_to_cache = add_leak(&mut l);
    let f = add_leak(&mut l);
    set_owner(&mut l, &f, "src.board.Board.publish.previous", &[]).unwrap();
    (l, f)
}

#[test]
fn an_owner_already_set_is_never_replaced_and_the_refusal_names_the_fix() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth(&["holder:closure:src.board.previous"]),
        ),
    );
    for ev in [vec![], vec![e]] {
        let r = refusal(set_owner(&mut l, &f, "src.board.previous", &ev));
        assert_eq!(r.code, codes::ANNOTATION_CONFLICT);
        assert!(
            r.fix
                .contains("dla ledger add --origin growth-series --evidence <E> --site a.go:1 --owner src.board.previous"),
            "{}",
            r.fix
        );
        assert!(r.fix.contains("dla ledger duplicate"), "{}", r.fix);
        assert!(r.fix.contains("or resolve the old one"), "{}", r.fix);
        assert_eq!(
            l.finding(&f).unwrap().owner.as_deref(),
            Some("src.board.Board.publish.previous")
        );
    }
    // The same value again, and an owner on a finding without one, still go through.
    set_owner(&mut l, &f, "src.board.Board.publish.previous", &[]).unwrap();
    let g = add_leak(&mut l);
    set_owner(&mut l, &g, "src.board.previous", &[]).unwrap();
}

#[test]
fn replay_refuses_a_line_that_carries_an_owner_change() {
    let (mut l, f) = setup();
    // Heap-graph growth naming the new owner's holder: the evidence the removed replacement
    // accepted an owner change with.
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth(&["holder:closure:src.board.previous"]),
        ),
    );
    let mut a = FindingAnnotation::new(f.clone());
    a.writers = vec!["a.rs:1".to_owned()];
    l.annotate_record(a).unwrap();
    let t = text(&l);
    assert!(Ledger::from_text(&t).is_ok());
    // An owner change without the old audit field, and one with it (a field of schema 6 as
    // first written, now unknown).
    let w = "\"writers\":[\"a.rs:1\"]";
    let changed = t.replace(w, &format!("{w},\"owner\":\"src.board.previous\""));
    assert_ne!(changed, t);
    let err = Ledger::from_text(&changed).unwrap_err().to_string();
    assert!(err.contains("already has owner"), "{err}");
    let with_evidence = t.replace(
        w,
        &format!("{w},\"owner\":\"src.board.previous\",\"evidence\":[\"{e}\"]"),
    );
    assert_ne!(with_evidence, t);
    let err = Ledger::from_text(&with_evidence).unwrap_err().to_string();
    assert!(err.contains("already has owner"), "{err}");
    let audited = t.replace(
        w,
        &format!(
            "{w},\"owner\":\"src.board.previous\",\"replaced_owner\":\"src.board.Board.publish.previous\""
        ),
    );
    assert!(Ledger::from_text(&audited).is_err());
}
