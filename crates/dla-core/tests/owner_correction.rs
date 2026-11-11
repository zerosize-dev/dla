//! An annotation may replace an owner already set when it cites growth-series or
//! retention-path evidence (heap-graph readers, dump or live) that names a holder with the
//! new owner. Any other change of an owner is still refused.

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

/// A ledger with a finding whose owner is the wrong `src.board.Board.publish.previous`. It
/// is F2: the shared ledger ties F1 to the growth of `probe:CACHE`, which would constrain
/// every owner change of F1.
fn setup() -> (Ledger, FindingId) {
    let mut l = m3_ledger();
    let _tied_to_cache = add_leak(&mut l);
    let f = add_leak(&mut l);
    set_owner(&mut l, &f, "src.board.Board.publish.previous", &[]).unwrap();
    (l, f)
}

#[test]
fn growth_evidence_naming_the_holder_replaces_the_owner_and_records_the_old_one() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth(&[
                "holder:field:src.board.Board.latest",
                "holder:closure:src.board.previous",
            ]),
        ),
    );
    set_owner(&mut l, &f, "src.board.previous", std::slice::from_ref(&e)).unwrap();
    assert_eq!(
        l.finding(&f).unwrap().owner.as_deref(),
        Some("src.board.previous")
    );
    let ch = &l.annotation(&f).unwrap().owner_changes;
    assert_eq!(ch.len(), 1);
    assert_eq!(ch[0].from, "src.board.Board.publish.previous");
    assert_eq!(ch[0].to, "src.board.previous");
    assert_eq!(ch[0].evidence, vec![e]);
    let dla_core::Record::FindingAnnotated(rec) = &l.lines().last().unwrap().record else {
        panic!()
    };
    assert_eq!(
        rec.replaced_owner.as_deref(),
        Some("src.board.Board.publish.previous")
    );
    // Replayed from text, the owner is the corrected one.
    let again = Ledger::from_text(&text(&l)).unwrap();
    assert_eq!(
        again.finding(&f).unwrap().owner.as_deref(),
        Some("src.board.previous")
    );
    // The same value again changes nothing.
    set_owner(&mut l, &f, "src.board.previous", &[]).unwrap();
    assert_eq!(l.annotation(&f).unwrap().owner_changes.len(), 1);
}

#[test]
fn a_retention_record_names_the_holder_too() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::RetentionPath,
            A::Dump,
            json!({"file": "a.heapsnapshot", "holders": [
                {"key": "closure:src.board.previous", "holder_kind": "closure/continuation"}]}),
        ),
    );
    set_owner(&mut l, &f, "src.board.previous", &[e]).unwrap();
    assert_eq!(
        l.finding(&f).unwrap().owner.as_deref(),
        Some("src.board.previous")
    );
}

#[test]
fn evidence_that_does_not_name_the_new_owner_is_refused_and_lists_what_it_names() {
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
    let r = refusal(set_owner(
        &mut l,
        &f,
        "src.board.other",
        std::slice::from_ref(&e),
    ));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    assert!(r.message.contains("src.board.previous"), "{r}");
    // Without evidence the old rule holds.
    let r = refusal(set_owner(&mut l, &f, "src.board.previous", &[]));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    assert!(r.fix.contains("--evidence"), "{r}");
    // Evidence of another kind or another tool does not back an owner.
    let d = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::HistogramDiff,
            A::Dump,
            json!({"holders": [{"key": "closure:src.board.previous"}]}),
        ),
    );
    refusal(set_owner(&mut l, &f, "src.board.previous", &[d]));
    let t = add(
        &mut l,
        reading(
            "dla-trend",
            K::GrowthSeries,
            A::Test,
            growth(&["holder:closure:src.board.previous"]),
        ),
    );
    refusal(set_owner(&mut l, &f, "src.board.previous", &[t]));
    assert_eq!(
        l.finding(&f).unwrap().owner.as_deref(),
        Some("src.board.Board.publish.previous")
    );
}

#[test]
fn replay_rejects_a_forged_replaced_owner() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-hprof",
            K::GrowthSeries,
            A::Dump,
            growth(&["holder:static:app.Cache.map"]),
        ),
    );
    set_owner(&mut l, &f, "app.Cache.map", &[e]).unwrap();
    let t = text(&l).replace(
        "\"replaced_owner\":\"src.board.Board.publish.previous\"",
        "\"replaced_owner\":\"x.y.z\"",
    );
    assert!(Ledger::from_text(&t).is_err());
}

#[test]
fn a_v5_line_still_refuses_an_owner_change() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-hprof",
            K::GrowthSeries,
            A::Dump,
            growth(&["holder:static:app.Cache.map"]),
        ),
    );
    set_owner(&mut l, &f, "app.Cache.map", &[e]).unwrap();
    let t = text(&l).replace("\"schema_version\":6", "\"schema_version\":5");
    assert!(Ledger::from_text(&t).is_err());
}

fn two_closures() -> Value {
    json!({"file": "a.heapsnapshot", "holders": [
        {"key": "closure:src.board.previous", "holder_kind": "closure/continuation"},
        {"key": "closure:lib.board.previous", "holder_kind": "closure/continuation"}]})
}

#[test]
fn an_invented_longer_owner_and_an_ambiguous_short_form_are_refused() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading("dla-v8-snapshot", K::RetentionPath, A::Dump, two_closures()),
    );
    // Invented longer form of a named owner.
    let r = refusal(set_owner(
        &mut l,
        &f,
        "zzz.yyy.src.board.previous",
        std::slice::from_ref(&e),
    ));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    // A short form of two holders ties none.
    let r = refusal(set_owner(
        &mut l,
        &f,
        "board.previous",
        std::slice::from_ref(&e),
    ));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    assert!(r.message.contains("several holders"), "{r}");
    // The exact owner is accepted.
    set_owner(&mut l, &f, "lib.board.previous", &[e]).unwrap();
}

#[test]
fn a_short_form_of_exactly_one_holder_is_accepted() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::RetentionPath,
            A::Dump,
            json!({"holders": [{"key": "closure:src.board.previous"},
                               {"key": "field:src.board.Board.latest"}]}),
        ),
    );
    set_owner(&mut l, &f, "board.previous", &[e]).unwrap();
}

#[test]
fn an_owner_is_replaced_only_before_confirmation() {
    let mut l = m3_ledger();
    let (f, _) = confirmed_leak(&mut l);
    set_owner(&mut l, &f, "src.board.Board.publish.previous", &[]).unwrap();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::RetentionPath,
            A::Dump,
            json!({"holders": [{"key": "closure:src.board.previous"}]}),
        ),
    );
    let r = refusal(set_owner(&mut l, &f, "src.board.previous", &[e]));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    assert!(r.message.contains("before confirmation"), "{r}");
    assert!(r.fix.contains("new finding"), "{r}");
    assert_eq!(
        l.finding(&f).unwrap().owner.as_deref(),
        Some("src.board.Board.publish.previous")
    );
}

#[test]
fn an_under_test_finding_still_takes_a_corrected_owner() {
    let (mut l, f) = setup();
    add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &f, dla_core::FindingState::UnderTest, &[E_HARNESS]);
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::RetentionPath,
            A::Dump,
            json!({"holders": [{"key": "closure:src.board.previous"}]}),
        ),
    );
    set_owner(&mut l, &f, "src.board.previous", &[e]).unwrap();
}

#[test]
fn replay_rejects_a_replaced_owner_that_replaces_nothing() {
    // Owner unchanged: the same value with a replaced_owner.
    let (mut l, f) = setup();
    set_owner(&mut l, &f, "src.board.Board.publish.previous", &[]).unwrap();
    let t = text(&l);
    let last = t.lines().last().unwrap().to_owned();
    let forged = last.replacen(
        "\"owner\":\"src.board.Board.publish.previous\"",
        "\"owner\":\"src.board.Board.publish.previous\",\"replaced_owner\":\"src.board.Board.publish.previous\"",
        1,
    );
    assert_ne!(last, forged);
    assert!(Ledger::from_text(&t.replace(&last, &forged)).is_err());
    // Owner unset before: a replaced_owner on the first owner.
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    set_owner(&mut l, &f, "app.Cache.map", &[]).unwrap();
    let t = text(&l);
    let last = t.lines().last().unwrap().to_owned();
    let forged = last.replacen(
        "\"owner\":\"app.Cache.map\"",
        "\"owner\":\"app.Cache.map\",\"replaced_owner\":\"x.y.z\"",
        1,
    );
    assert_ne!(last, forged);
    assert!(Ledger::from_text(&t.replace(&last, &forged)).is_err());
}

fn growth_tied(columns: &[(&str, &FindingId)]) -> Value {
    let passed: Vec<Value> = columns
        .iter()
        .map(|(c, f)| json!({"column": c, "region": "heap", "shape": "steady", "candidates": [f.to_string()]}))
        .collect();
    json!({"trend": "T1", "series": "S1", "n": 5, "passed": passed, "failed": 0, "gate": "pass"})
}

#[test]
fn a_new_owner_must_agree_with_the_growth_already_tied_to_the_finding() {
    // node-price-board: F is tied (by candidates) to the growth of the closure variable
    // `previous`; the same dump also shows Board.latest. Moving the owner to Board.latest
    // would let the growth of `previous` confirm a finding about another variable.
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            json!({"trend": "T1", "series": "S1", "n": 5, "failed": 0, "gate": "pass", "passed": [
                {"column": "holder:closure:src.board.previous", "region": "heap", "shape": "steady",
                 "candidates": [f.to_string()]},
                {"column": "holder:field:src.board.Board.latest", "region": "heap", "shape": "steady"}]}),
        ),
    );
    let r = refusal(set_owner(
        &mut l,
        &f,
        "src.board.Board.latest",
        std::slice::from_ref(&e),
    ));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    assert!(r.message.contains("src.board.previous"), "{r}");
    assert_eq!(
        l.finding(&f).unwrap().owner.as_deref(),
        Some("src.board.Board.publish.previous")
    );
    // The owner of the tied column is accepted.
    set_owner(&mut l, &f, "src.board.previous", std::slice::from_ref(&e)).unwrap();
}

#[test]
fn a_shorter_form_of_the_tied_owner_agrees_and_an_untied_column_does_not_count() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth_tied(&[("holder:closure:src.board.previous", &f)]),
        ),
    );
    set_owner(&mut l, &f, "board.previous", std::slice::from_ref(&e)).unwrap();
    // A column tied to another finding does not constrain this one.
    let (mut l, f) = setup();
    let other = add_leak(&mut l);
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            json!({"trend": "T1", "series": "S1", "n": 5, "failed": 0, "gate": "pass", "passed": [
                {"column": "holder:closure:src.board.previous", "region": "heap", "candidates": [other.to_string()]},
                {"column": "holder:field:src.board.Board.latest", "region": "heap"}]}),
        ),
    );
    set_owner(
        &mut l,
        &f,
        "src.board.Board.latest",
        std::slice::from_ref(&e),
    )
    .unwrap();
}

#[test]
fn a_short_form_is_unique_over_all_cited_records() {
    let (mut l, f) = setup();
    let a = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth(&["holder:field:app.a.Cache.map"]),
        ),
    );
    let b = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth(&[
                "holder:field:app.b.Cache.map",
                "holder:field:app.c.Cache.map",
            ]),
        ),
    );
    // Unique in the first record alone, but the second names two more holders with the form.
    let r = refusal(set_owner(&mut l, &f, "Cache.map", &[a.clone(), b]));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    assert!(r.message.contains("short form of several"), "{r}");
    set_owner(&mut l, &f, "Cache.map", &[a]).unwrap();
}

#[test]
fn the_holders_a_refusal_names_are_listed_once_each() {
    let (mut l, f) = setup();
    let a = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth(&["holder:field:app.Zed.one", "holder:field:app.Alpha.two"]),
        ),
    );
    let b = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth(&["holder:field:app.Alpha.two", "holder:field:app.Zed.one"]),
        ),
    );
    let r = refusal(set_owner(&mut l, &f, "app.Other.three", &[a, b]));
    assert_eq!(r.message.matches("app.Zed.one").count(), 1, "{r}");
    assert_eq!(r.message.matches("app.Alpha.two").count(), 1, "{r}");
}

#[test]
fn the_conflict_fix_offers_evidence_only_before_confirmation() {
    let (mut l, f) = setup();
    // A candidate: the fix names the evidence route.
    let r = refusal(set_owner(&mut l, &f, "src.board.Other.thing", &[]));
    assert!(r.fix.contains("--evidence"), "{r}");
    // A confirmed finding with an owner: the route is a dead end and is not offered.
    let mut l = m3_ledger();
    let (f, _) = confirmed_leak(&mut l);
    set_owner(&mut l, &f, "src.board.Board.publish.previous", &[]).unwrap();
    let r = refusal(set_owner(&mut l, &f, "src.board.Other.thing", &[]));
    assert!(!r.fix.contains("annotate"), "{r}");
    assert!(r.fix.contains("--sibling-of"), "{r}");
}

#[test]
fn a_tied_column_whose_holder_has_no_owner_form_refuses_a_new_owner() {
    // F is tied to the growth of a timer registration (no owner form); the same record
    // names a field. Nothing shows the field is what the tied growth measured.
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            json!({"trend": "T1", "series": "S1", "n": 5, "failed": 0, "gate": "pass", "passed": [
                {"column": "holder:registry:timers:src.sched.Scheduler.start.tick", "region": "heap",
                 "shape": "steady", "finding": f.to_string()},
                {"column": "holder:field:src.other.Cache.map", "region": "heap", "shape": "steady"}]}),
        ),
    );
    let r = refusal(set_owner(
        &mut l,
        &f,
        "src.other.Cache.map",
        std::slice::from_ref(&e),
    ));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    assert!(r.message.contains("no owner form"), "{r}");
    assert_eq!(
        l.finding(&f).unwrap().owner.as_deref(),
        Some("src.board.Board.publish.previous")
    );
}

#[test]
fn the_count_and_bytes_suffixes_of_tied_columns_name_the_same_holder() {
    // A dump series ties both `holder:K` and `holder:K.count`; the correction to K's owner
    // agrees with both.
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-v8-snapshot",
            K::GrowthSeries,
            A::Dump,
            growth_tied(&[
                ("holder:closure:src.board.previous", &f),
                ("holder:closure:src.board.previous.count", &f),
            ]),
        ),
    );
    set_owner(&mut l, &f, "src.board.previous", std::slice::from_ref(&e)).unwrap();
    assert_eq!(
        dla_core::holders::record_holder_owners(l.evidence_by_id(&e).unwrap()),
        vec!["src.board.previous".to_owned()]
    );
    // A probe named without its package, with a `.bytes` suffix, agrees with the qualified
    // owner a heap-graph record names.
    let (mut l, f) = setup();
    let tied = add(
        &mut l,
        reading(
            "dla",
            K::GrowthSeries,
            A::Test,
            growth_tied(&[("probe:Audit.trail.bytes", &f)]),
        ),
    );
    let _ = tied;
    let r = add(
        &mut l,
        reading(
            "dla-hprof",
            K::RetentionPath,
            A::Dump,
            json!({"holders": [{"key": "static:shop.codec.Audit.trail", "retained": 10}]}),
        ),
    );
    set_owner(
        &mut l,
        &f,
        "shop.codec.Audit.trail",
        std::slice::from_ref(&r),
    )
    .unwrap();
}

#[test]
fn a_short_owner_that_names_two_tied_holders_is_refused() {
    let (mut l, f) = setup();
    let e = add(
        &mut l,
        reading(
            "dla-hprof",
            K::GrowthSeries,
            A::Dump,
            growth_tied(&[("holder:static:a.Foo.x", &f), ("holder:static:b.Foo.x", &f)]),
        ),
    );
    let only_a = add(
        &mut l,
        reading(
            "dla-hprof",
            K::RetentionPath,
            A::Dump,
            json!({"holders": [{"key": "static:a.Foo.x", "retained": 10}]}),
        ),
    );
    let _ = e;
    let r = refusal(set_owner(
        &mut l,
        &f,
        "Foo.x",
        std::slice::from_ref(&only_a),
    ));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{r}");
    assert!(r.message.contains("a.Foo.x and b.Foo.x"), "{r}");
}
