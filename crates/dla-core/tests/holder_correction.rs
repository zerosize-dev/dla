//! Schema 5: an annotation may change a holder already set when it cites heap-graph
//! growth evidence whose passing column tied to the finding observed that holder.

mod common;

use common::*;
use dla_core::{
    Authority as A, Error, EvidenceKind as K, FindingId, Ledger, NewEvidence, Refusal, Tool,
    error::codes, finding::Holder, record::FindingAnnotation, slice::codes as m4,
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

fn graph(tool: &str, kind: K, authority: A, passed: Value) -> NewEvidence {
    let mut e = evidence_with(
        kind,
        authority,
        json!({"trend": "T1", "series": "S1", "n": 5, "passed": passed, "failed": 0, "gate": "pass"}),
    );
    e.tool = Tool {
        name: tool.into(),
        version: "0".into(),
    };
    e
}

fn col(finding: Option<&str>, candidates: &[&str], kind: &str) -> Value {
    let mut c = json!({"column": "holder:static:A.b", "region": "heap", "shape": "steady",
        "holder_kind": kind});
    if let Some(f) = finding {
        c["finding"] = json!(f);
    }
    if !candidates.is_empty() {
        c["candidates"] = json!(candidates);
    }
    c
}

/// A ledger with F1 (holder static/global) and the next free evidence ID.
fn setup() -> (Ledger, FindingId) {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    (l, f)
}

fn add(l: &mut Ledger, e: NewEvidence) -> dla_core::EvidenceId {
    match l.add_evidence(e).unwrap().record {
        dla_core::Record::Evidence(r) => r.evidence_id,
        _ => unreachable!(),
    }
}

fn change(
    l: &mut Ledger,
    f: &FindingId,
    to: Holder,
    ev: &[dla_core::EvidenceId],
) -> Result<(), Error> {
    let mut a = FindingAnnotation::new(f.clone());
    a.holder = Some(to);
    a.evidence = ev.to_vec();
    l.annotate_record(a).map(|_| ())
}

#[test]
fn matching_evidence_changes_the_holder_and_records_the_old_one() {
    let (mut l, f) = setup();
    for tool in ["dla-hprof", "dla-v8-snapshot"] {
        for authority in [A::Dump, A::Live] {
            let (mut l2, f2) = (l.clone(), f.clone());
            let e = add(
                &mut l2,
                graph(
                    tool,
                    K::GrowthSeries,
                    authority,
                    json!([col(Some("F1"), &[], "registration-table")]),
                ),
            );
            change(&mut l2, &f2, Holder::RegistrationTable, &[e]).unwrap();
            assert_eq!(
                l2.finding(&f2).unwrap().holder,
                Some(Holder::RegistrationTable),
                "{tool} {authority}"
            );
        }
    }
    let e = add(
        &mut l,
        graph(
            "dla-hprof",
            K::GrowthSeries,
            A::Dump,
            json!([col(None, &["F1"], "registration-table")]),
        ),
    );
    change(
        &mut l,
        &f,
        Holder::RegistrationTable,
        std::slice::from_ref(&e),
    )
    .unwrap();
    let fin = l.finding(&f).unwrap();
    assert_eq!(fin.holder, Some(Holder::RegistrationTable));
    let last = l.lines().last().unwrap();
    assert_eq!(last.schema_version, 6);
    let dla_core::Record::FindingAnnotated(rec) = &last.record else {
        panic!()
    };
    assert_eq!(rec.replaced_holder, Some(Holder::StaticGlobal));
    assert_eq!(rec.evidence, vec![e.clone()]);
    let ch = &l.annotation(&f).unwrap().holder_changes;
    assert_eq!(ch.len(), 1);
    assert_eq!(
        (ch[0].from, ch[0].to),
        (Holder::StaticGlobal, Holder::RegistrationTable)
    );
    assert_eq!(ch[0].evidence, vec![e]);
    // The same holder again needs nothing.
    change(&mut l, &f, Holder::RegistrationTable, &[]).unwrap();
}

#[test]
fn replay_of_a_v5_ledger_reproduces_the_state() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        graph(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            json!([col(Some("F1"), &[], "closure/continuation")]),
        ),
    );
    change(&mut l, &f, Holder::ClosureContinuation, &[e]).unwrap();
    let again = Ledger::from_text(&text(&l)).unwrap();
    assert_eq!(
        again.finding(&f).unwrap().holder,
        Some(Holder::ClosureContinuation)
    );
    assert_eq!(again.annotation(&f).unwrap().holder_changes.len(), 1);
    assert_eq!(text(&again), text(&l));
}

#[test]
fn the_annotation_is_refused_when_the_evidence_does_not_back_it() {
    let (mut l, f) = setup();
    let good = col(Some("F1"), &[], "registration-table");
    let cases: Vec<(&str, NewEvidence, &str)> = vec![
        (
            "another kind",
            graph(
                "dla-hprof",
                K::UnitCountSeries,
                A::Dump,
                json!([good.clone()]),
            ),
            "not growth-series",
        ),
        (
            "another authority",
            graph("dla-hprof", K::GrowthSeries, A::Test, json!([good.clone()])),
            "not dump or live",
        ),
        (
            "another tool",
            graph(
                "dla-sample",
                K::GrowthSeries,
                A::Live,
                json!([good.clone()]),
            ),
            "not a heap-graph reader",
        ),
        (
            "not tied to F1",
            graph(
                "dla-hprof",
                K::GrowthSeries,
                A::Dump,
                json!([col(Some("F9"), &["F2"], "registration-table")]),
            ),
            "no passing column tied to F1",
        ),
        (
            "tied with another holder_kind",
            graph(
                "dla-hprof",
                K::GrowthSeries,
                A::Dump,
                json!([col(Some("F1"), &[], "thread-local")]),
            ),
            "observed thread-local",
        ),
        (
            "no holder_kind",
            graph(
                "dla-hprof",
                K::GrowthSeries,
                A::Dump,
                json!([{"column": "c", "finding": "F1"}]),
            ),
            "no holder_kind",
        ),
    ];
    for (name, ev, want) in cases {
        let e = add(&mut l, ev);
        let r = refusal(change(&mut l, &f, Holder::RegistrationTable, &[e]));
        assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{name}");
        assert!(r.message.contains(want), "{name}: {}", r.message);
        assert!(
            r.fix
                .contains("dla ledger annotate F1 --holder registration-table --evidence"),
            "{name}: {}",
            r.fix
        );
        assert_eq!(l.finding(&f).unwrap().holder, Some(Holder::StaticGlobal));
    }
    // Unknown evidence.
    let r = refusal(change(&mut l, &f, Holder::RegistrationTable, &[eid(99)]));
    assert_eq!(r.code, codes::UNKNOWN_EVIDENCE);
    // No evidence: the old message, with the way out named.
    let r = refusal(change(&mut l, &f, Holder::RegistrationTable, &[]));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT);
    assert!(
        r.fix.contains("--evidence <growth evidence id>"),
        "{}",
        r.fix
    );
    // A good one among bad ones passes.
    let ok = add(
        &mut l,
        graph("dla-hprof", K::GrowthSeries, A::Dump, json!([good])),
    );
    change(&mut l, &f, Holder::RegistrationTable, &[eid(4), ok]).unwrap();
}

#[test]
fn a_finding_without_a_holder_annotates_without_evidence() {
    let mut l = m3_ledger();
    let f = match l
        .add_finding(dla_core::NewFinding::new(
            origin("census"),
            vec![eid(E_STATIC)],
        ))
        .unwrap()
        .record
    {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    change(&mut l, &f, Holder::ContainerField, &[]).unwrap();
    assert_eq!(l.finding(&f).unwrap().holder, Some(Holder::ContainerField));
    let dla_core::Record::FindingAnnotated(rec) = &l.lines().last().unwrap().record else {
        panic!()
    };
    assert_eq!(rec.replaced_holder, None);
    assert!(l.annotation(&f).unwrap().holder_changes.is_empty());
}

#[test]
fn other_conflicts_stay_refused_even_with_evidence() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        graph(
            "dla-hprof",
            K::GrowthSeries,
            A::Dump,
            json!([col(Some("F1"), &[], "registration-table")]),
        ),
    );
    let mut a = FindingAnnotation::new(f.clone());
    a.owner = Some("A.b".into());
    l.annotate_record(a).unwrap();
    let mut a = FindingAnnotation::new(f.clone());
    a.owner = Some("A.c".into());
    a.evidence = vec![e];
    let r = refusal(l.annotate_record(a).map(|_| ()));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT);
}

fn set_version(text: &str, v: u32, only_last: bool) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let range = if only_last { lines.len() - 1.. } else { 0.. };
    for l in &mut lines[range] {
        *l = l.replacen(
            "\"schema_version\":6",
            &format!("\"schema_version\":{v}"),
            1,
        );
    }
    lines.join("\n") + "\n"
}

#[test]
fn version_4_lines_keep_the_old_rule() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        graph(
            "dla-hprof",
            K::GrowthSeries,
            A::Dump,
            json!([col(Some("F1"), &[], "registration-table")]),
        ),
    );
    change(&mut l, &f, Holder::RegistrationTable, &[e]).unwrap();
    // The change line written as version 4 is a conflict on replay.
    let t = set_version(&text(&l), 4, true);
    let Err(Error::Replay(r)) = Ledger::from_text(&t) else {
        panic!("a v4 line cannot change a holder")
    };
    assert_eq!(r.refusal.code, codes::ANNOTATION_CONFLICT);
    // A whole v4 ledger without changes replays as before.
    let (l4, _) = setup();
    let t = set_version(&text(&l4), 4, false);
    let again = Ledger::from_text(&t).unwrap();
    assert_eq!(again.lines()[0].schema_version, 4);
    // A v7 line is refused as written by a newer dla.
    let t =
        set_version(&text(&l4), 7, false).replace("\"schema_version\":4", "\"schema_version\":7");
    assert!(matches!(Ledger::from_text(&t), Err(Error::Schema(_))));
}

#[test]
fn replay_rejects_a_forged_replaced_holder() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        graph(
            "dla-hprof",
            K::GrowthSeries,
            A::Dump,
            json!([col(Some("F1"), &[], "registration-table")]),
        ),
    );
    change(&mut l, &f, Holder::RegistrationTable, &[e]).unwrap();
    let t = text(&l).replace(
        "\"replaced_holder\":\"static/global\"",
        "\"replaced_holder\":\"thread-local\"",
    );
    assert!(Ledger::from_text(&t).is_err());
}

#[test]
fn a_worker_cannot_cite_another_workers_evidence() {
    let dir = new_run(&["jvm", "go"]);
    let mut s = slice(dir.path(), "jvm");
    let f = candidate_in_slice(&mut s, "a.go:1", Holder::StaticGlobal);
    let foreign: dla_core::EvidenceId = "go/E1".parse().unwrap();
    let r = refusal(change(&mut s, &f, Holder::RegistrationTable, &[foreign]));
    assert_eq!(r.code, m4::FOREIGN_ITEM, "{r}");
    // Its own evidence works.
    let e = add(
        &mut s,
        graph(
            "dla-hprof",
            K::GrowthSeries,
            A::Dump,
            json!([col(Some(&f.to_string()), &[], "registration-table")]),
        ),
    );
    assert_eq!(e.prefix(), Some("jvm"));
    change(&mut s, &f, Holder::RegistrationTable, &[e]).unwrap();
    assert_eq!(
        s.finding(&f).unwrap().holder,
        Some(Holder::RegistrationTable)
    );
}
