//! The M3 ledger rules: polarity, `under_test`, `confirmed`, `refuted`, annotations,
//! derivation on confirm and the version split between M0 and M3 lines.

mod common;

use common::*;
use dla_core::{
    Error, FindingId, FindingState as S, Ledger, NewFinding, Refusal,
    error::codes,
    finding::{Holder, Polarity, Region, Shape},
    record::FindingAnnotation,
};
use serde_json::json;

fn refusal<T: std::fmt::Debug>(r: Result<T, Error>) -> Refusal {
    match r {
        Err(Error::Refused(r)) => r,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn mv(l: &mut Ledger, id: &FindingId, to: S, ev: &[u32]) -> Result<(), Error> {
    l.transition(
        id,
        to,
        None,
        None,
        ev.iter().map(|n| eid(*n)).collect(),
        "test",
    )
    .map(|_| ())
}

fn to_text(l: &Ledger) -> String {
    l.lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect()
}

/// F1 (site, holder) with an N-WARM sibling F1.n1, both candidates.
fn pair() -> (Ledger, FindingId, FindingId) {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    (l, f, n)
}

// ----- polarity and N-* validation -------------------------------------------------

#[test]
fn polarity_follows_the_hypothesis_of_a_sibling() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let table = [
        ("N-ALLOC", Polarity::NonLeak),
        ("N-HEADROOM", Polarity::NonLeak),
        ("N-WARM", Polarity::NonLeak),
        ("N-HWM", Polarity::NonLeak),
        ("N-SCALE", Polarity::NonLeak),
        ("N-INTENT", Polarity::NonLeak),
        ("N-OBSERVER", Polarity::NonLeak),
        ("N-WARM: the cache fills up", Polarity::NonLeak),
        ("N-HWM pool high-water mark", Polarity::NonLeak),
        ("the listener is never removed", Polarity::Leak),
        ("n-warm lower case is not an id", Polarity::Leak),
        ("see N-WARM later", Polarity::Leak),
    ];
    for (hyp, want) in table {
        let s = add_sibling(&mut l, &f, hyp);
        assert_eq!(l.finding(&s).unwrap().polarity(), want, "{hyp}");
    }
    assert_eq!(l.finding(&f).unwrap().polarity(), Polarity::Leak);
}

#[test]
fn a_hypothesis_starting_with_n_dash_must_name_one_of_the_seven() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    for bad in [
        "N-FOO",
        "N-WARMUP",
        "N-",
        "N-WARM,N-HWM",
        "N-ALLOCATION",
        "N-warm",
    ] {
        let mut nf = NewFinding::new(origin("hypothesis"), vec![eid(E_STATIC)]);
        nf.sibling_of = Some(f.clone());
        nf.hypothesis = Some(bad.into());
        nf.refute_if = Some("x".into());
        let r = refusal(l.add_finding(nf));
        assert_eq!(r.code, codes::INVALID_VALUE, "{bad}");
        assert!(r.message.contains("N-ALLOC"), "{}", r.message);
        assert!(r.fix.contains("--sibling-of F1"), "{}", r.fix);
    }
    // The rule does not apply to version 1 lines.
    let v1 = [
        line(1, EVIDENCE_BODY),
        line(
            2,
            r#""type":"finding_added","finding":{"id":"F1","state":"candidate","origin":"census","supports":["E1"]}"#,
        ),
        line(
            3,
            r#""type":"finding_added","finding":{"id":"F1.n1","state":"candidate","origin":"hypothesis","supports":["E1"],"parent":"F1","hypothesis":"N-FOO","refute_if":"x"}"#,
        ),
    ]
    .concat();
    Ledger::from_text(&v1).unwrap();
    Ledger::from_text(&v1.replace("\"schema_version\":1", "\"schema_version\":2")).unwrap_err();
}

#[test]
fn families_are_the_root_and_its_siblings() {
    let (mut l, f, n) = pair();
    let other = add_sibling(&mut l, &f, "the registry leaks");
    let mut s = l.finding(&n).unwrap().clone();
    s.parent = Some(f.clone());
    assert_eq!(s.root_id(), f);
    assert_eq!(l.finding(&f).unwrap().root_id(), f);
    assert_eq!(l.finding(&f).unwrap().siblings, [n.clone(), other.clone()]);
}

// ----- -> under_test ---------------------------------------------------------------

#[test]
fn under_test_table() {
    // (name, evidence cited, expected refusal code or None for success)
    let cases: [(&str, &[u32], Option<&str>); 6] = [
        ("nothing cited", &[], Some(codes::HARNESS_REQUIRED)),
        ("a static hit", &[E_STATIC], Some(codes::HARNESS_REQUIRED)),
        (
            "workload discover is not a harness",
            &[E_DISCOVER],
            Some(codes::HARNESS_REQUIRED),
        ),
        ("a harness record", &[E_HARNESS], None),
        ("harness with extras", &[E_HARNESS, E_STATIC], None),
        (
            "growth is not a harness",
            &[E_GROWTH_PASS],
            Some(codes::HARNESS_REQUIRED),
        ),
    ];
    for (name, ev, want) in cases {
        let (mut l, f, _) = pair();
        let got = mv(&mut l, &f, S::UnderTest, ev);
        match want {
            Some(code) => {
                let r = refusal(got);
                assert_eq!(r.code, code, "{name}");
                assert!(r.fix.contains("dla harness gen"), "{name}: {}", r.fix);
                assert!(r.fix.contains("dla ledger test F1"), "{name}: {}", r.fix);
            }
            None => got.unwrap_or_else(|e| panic!("{name}: {e}")),
        }
    }
}

#[test]
fn the_harness_may_already_be_a_support_of_the_finding() {
    let mut l = m3_ledger();
    let mut nf = NewFinding::new(origin("census"), vec![eid(E_HARNESS)]);
    nf.site = Some("a.go:1".into());
    let f = match l.add_finding(nf).unwrap().record {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    add_sibling(&mut l, &f, "N-HWM");
    mv(&mut l, &f, S::UnderTest, &[]).unwrap();
}

#[test]
fn a_leak_side_finding_needs_a_non_leak_sibling_to_go_under_test() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let r = refusal(mv(&mut l, &f, S::UnderTest, &[E_HARNESS]));
    assert_eq!(r.code, codes::SIBLING_REQUIRED);
    assert!(
        r.fix
            .starts_with("dla ledger add --sibling-of F1 --origin hypothesis --hypothesis N-WARM"),
        "{}",
        r.fix
    );
    // A sibling that is leak-side does not count.
    let other = add_sibling(&mut l, &f, "the registry leaks");
    let r = refusal(mv(&mut l, &f, S::UnderTest, &[E_HARNESS]));
    assert_eq!(r.code, codes::SIBLING_REQUIRED);
    // The leak-side sibling is refused the same way, and names the root in its fix.
    let r = refusal(mv(&mut l, &other, S::UnderTest, &[E_HARNESS]));
    assert_eq!(r.code, codes::SIBLING_REQUIRED);
    assert!(r.fix.contains("--sibling-of F1"), "{}", r.fix);
    // Once an N-* sibling exists (in any state), both go under test.
    let n = add_sibling(&mut l, &f, "N-SCALE");
    mv(&mut l, &f, S::UnderTest, &[E_HARNESS]).unwrap();
    mv(&mut l, &other, S::UnderTest, &[E_HARNESS]).unwrap();
    // A non-leak finding needs the harness only.
    mv(&mut l, &n, S::UnderTest, &[E_HARNESS]).unwrap();
}

#[test]
fn a_non_leak_finding_needs_only_the_harness() {
    let (mut l, _, n) = pair();
    let r = refusal(mv(&mut l, &n, S::UnderTest, &[]));
    assert_eq!(r.code, codes::HARNESS_REQUIRED);
    mv(&mut l, &n, S::UnderTest, &[E_HARNESS]).unwrap();
}

#[test]
fn leaving_suspected_static_only_or_unresolved_is_judged_the_same_way() {
    for via in [S::SuspectedStaticOnly, S::Unresolved] {
        let (mut l, f, _) = pair();
        l.transition(
            &f,
            via,
            Some(dla_core::UnresolvedReason::NotExercised),
            Some("run it".into()),
            vec![eid(E_STATIC)],
            "t",
        )
        .unwrap();
        let r = refusal(mv(&mut l, &f, S::UnderTest, &[]));
        assert_eq!(r.code, codes::HARNESS_REQUIRED, "{via}");
        mv(&mut l, &f, S::UnderTest, &[E_HARNESS]).unwrap();
    }
}

// ----- -> confirmed, leak-side -----------------------------------------------------

/// F1 under test with a refuted N-WARM sibling; the ledger is ready to confirm F1.
fn ready() -> (Ledger, FindingId, FindingId) {
    let (mut l, f, n) = pair();
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    (l, f, n)
}

#[test]
fn confirm_leak_growth_gate_table() {
    let cases: [(&str, &[u32], Option<&str>); 9] = [
        ("nothing cited", &[], Some(codes::GROWTH_GATE_FAILED)),
        ("static hit", &[E_STATIC], Some(codes::GROWTH_GATE_FAILED)),
        (
            "gate fail",
            &[E_GROWTH_FAIL],
            Some(codes::GROWTH_GATE_FAILED),
        ),
        (
            "a series with no gate field",
            &[E_GROWTH_NO_GATE],
            Some(codes::GROWTH_GATE_FAILED),
        ),
        (
            "gate pass with static authority",
            &[E_GROWTH_STATIC_AUTH],
            Some(codes::GROWTH_GATE_FAILED),
        ),
        (
            "a discriminator is not growth",
            &[E_DISC_NONLEAK],
            Some(codes::GROWTH_GATE_FAILED),
        ),
        ("growth-series pass (test)", &[E_GROWTH_PASS], None),
        ("unit-count-series pass (dump)", &[E_UNIT_PASS], None),
        (
            "one failing, one passing",
            &[E_GROWTH_FAIL, E_GROWTH_PASS],
            None,
        ),
    ];
    for (name, ev, want) in cases {
        let (mut l, f, _) = ready();
        let got = mv(&mut l, &f, S::Confirmed, ev);
        match want {
            Some(code) => {
                let r = refusal(got);
                assert_eq!(r.code, code, "{name}");
                assert!(r.fix.starts_with("dla trend "), "{name}: {}", r.fix);
                if ev.contains(&E_GROWTH_FAIL) {
                    assert!(r.fix.starts_with("dla trend S1 "), "{name}: {}", r.fix);
                }
                assert!(r.missing.contains("gate"), "{name}: {}", r.missing);
            }
            None => got.unwrap_or_else(|e| panic!("{name}: {e}")),
        }
        assert_eq!(
            l.finding(&f).unwrap().state,
            if want.is_some() {
                S::UnderTest
            } else {
                S::Confirmed
            },
            "{name}"
        );
    }
}

#[test]
fn growth_evidence_already_supporting_the_finding_counts() {
    let mut l = m3_ledger();
    let mut nf = NewFinding::new(origin("growth-series"), vec![eid(E_GROWTH_PASS)]);
    nf.site = Some("a.go:1".into());
    nf.holder = Some(Holder::ContainerField);
    let f = match l.add_finding(nf).unwrap().record {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, S::Confirmed, &[]);
}

#[test]
fn confirm_needs_a_site_then_a_holder_then_a_refuted_alternative() {
    // site
    let mut l = m3_ledger();
    let mut nf = NewFinding::new(origin("census"), vec![eid(E_STATIC)]);
    nf.holder = Some(Holder::StaticGlobal);
    let f = match l.add_finding(nf).unwrap().record {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    let r = refusal(mv(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]));
    assert_eq!(r.code, codes::SITE_REQUIRED);
    assert!(r.fix.contains("--site"), "{}", r.fix);

    // holder
    let mut l = m3_ledger();
    let mut nf = NewFinding::new(origin("census"), vec![eid(E_STATIC)]);
    nf.site = Some("a.go:1".into());
    let f = match l.add_finding(nf).unwrap().record {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    let r = refusal(mv(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]));
    assert_eq!(r.code, codes::HOLDER_REQUIRED);
    assert_eq!(
        r.fix,
        "dla ledger annotate F1 --holder <holder> [--owner <text>]"
    );
    l.annotate_record(FindingAnnotation {
        holder: Some(Holder::RegistrationTable),
        ..FindingAnnotation::new(f.clone())
    })
    .unwrap();
    go(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]);

    // refuted alternative
    let (mut l, f, n) = pair();
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    let r = refusal(mv(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]));
    assert_eq!(r.code, codes::NONLEAK_REFUTATION_REQUIRED);
    assert!(
        r.fix.contains("dla alloc-test") && r.fix.contains("--finding F1"),
        "{}",
        r.fix
    );
    assert!(
        r.fix.contains(&format!("dla ledger refute {n}")),
        "{}",
        r.fix
    );
    // A non-leak sibling that is only under test does not satisfy it.
    go(&mut l, &n, S::UnderTest, &[E_HARNESS]);
    let r = refusal(mv(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]));
    assert_eq!(r.code, codes::NONLEAK_REFUTATION_REQUIRED);
    // A refuted leak-side sibling does not either.
    let other = add_sibling(&mut l, &f, "the registry leaks");
    go(&mut l, &other, S::Refuted, &[E_DISC_NONLEAK]);
    let r = refusal(mv(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]));
    assert_eq!(r.code, codes::NONLEAK_REFUTATION_REQUIRED);
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]);
}

#[test]
fn checks_run_in_the_documented_order() {
    // Everything missing: the growth gate is reported first.
    let mut l = m3_ledger();
    let f = match l
        .add_finding(NewFinding::new(origin("census"), vec![eid(E_STATIC)]))
        .unwrap()
        .record
    {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    let codes_seen: Vec<String> = {
        let mut seen = Vec::new();
        let r = refusal(mv(&mut l, &f, S::Confirmed, &[]));
        seen.push(r.code);
        let r = refusal(mv(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]));
        seen.push(r.code);
        seen
    };
    assert_eq!(
        codes_seen,
        [codes::GROWTH_GATE_FAILED, codes::SITE_REQUIRED]
    );
}

// ----- derivation and warning on confirm -------------------------------------------

#[test]
fn confirm_derives_shape_and_region_from_the_tied_column() {
    let cases: [(&str, &[u32], _, _); 3] = [
        (
            "tied heap column",
            &[E_GROWTH_PASS],
            Some(Shape::Steady),
            Some(Region::Heap),
        ),
        (
            "unit counts at the site",
            &[E_UNIT_PASS],
            Some(Shape::Stair),
            Some(Region::NonHeap),
        ),
        (
            "a tied column wins over an earlier untied one",
            &[E_GROWTH_OTHER, E_GROWTH_PASS],
            Some(Shape::Steady),
            Some(Region::Heap),
        ),
    ];
    for (name, ev, shape, region) in cases {
        let (mut l, f, _) = ready();
        go(&mut l, &f, S::Confirmed, ev);
        let fin = l.finding(&f).unwrap();
        assert_eq!((fin.shape, fin.region), (shape, region), "{name}");
        assert_eq!(l.confirm_warning(&f), None, "{name}");
        // Replaying the written lines derives the same values: no extra record field.
        let again = Ledger::from_text(&to_text(&l)).unwrap();
        let fin = again.finding(&f).unwrap();
        assert_eq!((fin.shape, fin.region), (shape, region), "{name} (replay)");
    }
}

#[test]
fn an_untied_column_does_not_confirm_and_derives_nothing() {
    let (mut l, f, _) = ready();
    // The only passing column belongs to F9 at b.go:2 (plateau, off-heap): it says
    // nothing about F1. Since ledger schema 4 the confirmation is refused.
    let e = l
        .transition(
            &f,
            S::Confirmed,
            None,
            None,
            vec![eid(E_GROWTH_OTHER)],
            "test",
        )
        .unwrap_err()
        .to_refusal();
    assert_eq!(e.code, "growth-not-tied", "{e}");
    assert!(e.message.contains("F1"), "{e}");
    let fin = l.finding(&f).unwrap();
    assert_eq!(fin.state, S::UnderTest);
    assert_eq!((fin.shape, fin.region), (None, None));
}

#[test]
fn values_the_finding_already_has_are_kept_and_process_derives_no_region() {
    // Shape and region set when the finding was added are not overwritten.
    let mut l = m3_ledger();
    let mut nf = NewFinding::new(origin("census"), vec![eid(E_STATIC)]);
    nf.site = Some("a.go:1".into());
    nf.holder = Some(Holder::StaticGlobal);
    nf.shape = Some(Shape::Spike);
    nf.region = Some(Region::NonHeap);
    let f = match l.add_finding(nf).unwrap().record {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]);
    let fin = l.finding(&f).unwrap();
    assert_eq!(
        (fin.shape, fin.region),
        (Some(Shape::Spike), Some(Region::NonHeap))
    );

    // A process-region column derives a shape but leaves the region unset.
    let mut l = m3_ledger();
    let e = l
        .add_evidence(evidence_with(
            dla_core::EvidenceKind::GrowthSeries,
            dla_core::Authority::Live,
            json!({"gate": "pass", "series": "S2", "passed": [
                {"column": "os.rss", "region": "process", "shape": "steady", "site": "a.go:1"}]}),
        ))
        .unwrap();
    let dla_core::Record::Evidence(e) = e.record else {
        unreachable!()
    };
    let (f, n) = {
        let f = add_leak(&mut l);
        let n = add_sibling(&mut l, &f, "N-HWM");
        (f, n)
    };
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    l.transition(
        &f,
        S::Confirmed,
        None,
        None,
        vec![e.evidence_id.clone()],
        "t",
    )
    .unwrap();
    let fin = l.finding(&f).unwrap();
    assert_eq!((fin.shape, fin.region), (Some(Shape::Steady), None));
}

// ----- -> confirmed, non-leak ------------------------------------------------------

#[test]
fn confirm_non_leak_table() {
    // (name, discriminator cited, root refuted first, expected code)
    let cases: [(&str, &[u32], bool, Option<&str>); 8] = [
        (
            "no discriminator",
            &[E_STATIC],
            true,
            Some(codes::DISCRIMINATOR_REQUIRED),
        ),
        (
            "favors leak",
            &[E_DISC_LEAK],
            true,
            Some(codes::DISCRIMINATOR_DIRECTION),
        ),
        (
            "inconclusive",
            &[E_DISC_INCONCLUSIVE],
            true,
            Some(codes::DISCRIMINATOR_DIRECTION),
        ),
        (
            "bound to another family",
            &[E_DISC_NONLEAK_F9],
            true,
            Some(codes::DISCRIMINATOR_DIRECTION),
        ),
        (
            "good discriminator but the root is not refuted",
            &[E_DISC_NONLEAK],
            false,
            Some(codes::LEAK_REFUTATION_REQUIRED),
        ),
        (
            "favors non-leak, root refuted",
            &[E_DISC_NONLEAK],
            true,
            None,
        ),
        ("bound to the family", &[E_DISC_NONLEAK_F1], true, None),
        (
            "a leak verdict next to a non-leak one",
            &[E_DISC_LEAK, E_DISC_NONLEAK],
            true,
            None,
        ),
    ];
    for (name, ev, refute_root, want) in cases {
        let (mut l, f, n) = pair();
        go(&mut l, &n, S::UnderTest, &[E_HARNESS]);
        if refute_root {
            go(&mut l, &f, S::Refuted, &[E_DISC_NONLEAK]);
        }
        let got = mv(&mut l, &n, S::Confirmed, ev);
        match want {
            Some(code) => {
                let r = refusal(got);
                assert_eq!(r.code, code, "{name}: {}", r.message);
                assert!(r.fix.starts_with("dla "), "{name}: {}", r.fix);
            }
            None => got.unwrap_or_else(|e| panic!("{name}: {e}")),
        }
    }
}

#[test]
fn a_confirmed_non_leak_needs_no_site_holder_or_growth_and_derives_nothing() {
    let (mut l, f, n) = pair();
    go(&mut l, &n, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, S::Refuted, &[E_DISC_NONLEAK]);
    // Even with passing growth cited, a non-leak finding derives no shape or region.
    go(&mut l, &n, S::Confirmed, &[E_DISC_NONLEAK, E_GROWTH_PASS]);
    let fin = l.finding(&n).unwrap();
    assert!(fin.site.is_none() && fin.holder.is_none());
    assert_eq!((fin.shape, fin.region), (None, None));
    assert_eq!(l.confirm_warning(&n), None);
    Ledger::from_text(&to_text(&l)).unwrap();
}

// ----- -> refuted ------------------------------------------------------------------

#[test]
fn refute_direction_table() {
    // (name, polarity of the target, discriminator cited, expected)
    let leak = Polarity::Leak;
    let non = Polarity::NonLeak;
    let cases: [(&str, Polarity, &[u32], Option<&str>); 13] = [
        (
            "leak-side, nothing",
            leak,
            &[],
            Some(codes::DISCRIMINATOR_REQUIRED),
        ),
        (
            "leak-side, static only",
            leak,
            &[E_STATIC],
            Some(codes::DISCRIMINATOR_REQUIRED),
        ),
        (
            "leak-side, favors leak",
            leak,
            &[E_DISC_LEAK],
            Some(codes::DISCRIMINATOR_DIRECTION),
        ),
        (
            "leak-side, inconclusive",
            leak,
            &[E_DISC_INCONCLUSIVE],
            Some(codes::DISCRIMINATOR_DIRECTION),
        ),
        (
            "leak-side, non-leak but bound to F9",
            leak,
            &[E_DISC_NONLEAK_F9],
            Some(codes::DISCRIMINATOR_DIRECTION),
        ),
        ("leak-side, favors non-leak", leak, &[E_DISC_NONLEAK], None),
        (
            "leak-side, non-leak bound to F1",
            leak,
            &[E_DISC_NONLEAK_F1],
            None,
        ),
        (
            "non-leak, nothing",
            non,
            &[],
            Some(codes::DISCRIMINATOR_REQUIRED),
        ),
        (
            "non-leak, favors non-leak",
            non,
            &[E_DISC_NONLEAK],
            Some(codes::DISCRIMINATOR_DIRECTION),
        ),
        (
            "non-leak, favors leak but bound to F9",
            non,
            &[E_DISC_LEAK_F9],
            Some(codes::DISCRIMINATOR_DIRECTION),
        ),
        ("non-leak, favors leak", non, &[E_DISC_LEAK], None),
        ("non-leak, leak bound to F1", non, &[E_DISC_LEAK_F1], None),
        (
            "non-leak, mixed verdicts",
            non,
            &[E_DISC_NONLEAK, E_DISC_LEAK],
            None,
        ),
    ];
    for from in [S::Candidate, S::UnderTest] {
        for (name, pol, ev, want) in cases {
            let (mut l, f, n) = pair();
            let target = if pol == Polarity::Leak { f } else { n };
            if from == S::UnderTest {
                go(&mut l, &target, S::UnderTest, &[E_HARNESS]);
            }
            let got = mv(&mut l, &target, S::Refuted, ev);
            match want {
                Some(code) => {
                    let r = refusal(got);
                    assert_eq!(r.code, code, "{name} from {from}: {}", r.message);
                    assert!(r.fix.contains("dla "), "{name}: {}", r.fix);
                }
                None => {
                    got.unwrap_or_else(|e| panic!("{name} from {from}: {e}"));
                    assert_eq!(l.finding(&target).unwrap().state, S::Refuted);
                    assert!(
                        l.finding(&target)
                            .unwrap()
                            .contradicts
                            .contains(&eid(ev[0]))
                    );
                }
            }
        }
    }
}

#[test]
fn a_discriminator_already_in_contradicts_counts_for_a_later_move() {
    // Refuting needs the right direction among contradicts plus cited, not only cited.
    let (mut l, f, _) = pair();
    let r = refusal(mv(&mut l, &f, S::Refuted, &[E_DISC_LEAK]));
    assert_eq!(r.code, codes::DISCRIMINATOR_DIRECTION);
    assert!(r.message.contains("E8 favors: leak"), "{}", r.message);
    assert!(l.finding(&f).unwrap().contradicts.is_empty());
}

#[test]
fn a_discriminator_may_name_a_sibling_of_the_same_family() {
    let (mut l, f, n) = pair();
    let e = l
        .add_evidence(evidence_with(
            dla_core::EvidenceKind::Discriminator,
            dla_core::Authority::Test,
            json!({"test": "plateau", "favors": "non-leak", "supports": ["N-WARM"],
                   "finding": n.to_string()}),
        ))
        .unwrap();
    let dla_core::Record::Evidence(e) = e.record else {
        unreachable!()
    };
    l.transition(&f, S::Refuted, None, None, vec![e.evidence_id], "t")
        .unwrap();
}

// ----- the whole path --------------------------------------------------------------

#[test]
fn leak_path_ends_confirmed_and_non_leak_path_ends_refuted() {
    // Leak: sibling refuted by a leak verdict, finding confirmed on the gate.
    let mut l = m3_ledger();
    let (f, n) = confirmed_leak(&mut l);
    assert_eq!(l.finding(&f).unwrap().state, S::Confirmed);
    assert_eq!(l.finding(&n).unwrap().state, S::Refuted);
    // Non-leak: the finding is refuted by a non-leak verdict, the N-* sibling confirmed.
    let (mut l, f, n) = pair();
    go(&mut l, &n, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, S::Refuted, &[E_DISC_NONLEAK]);
    go(&mut l, &n, S::Confirmed, &[E_DISC_NONLEAK]);
    assert_eq!(l.finding(&f).unwrap().state, S::Refuted);
    assert_eq!(l.finding(&n).unwrap().state, S::Confirmed);
}

// ----- annotations -----------------------------------------------------------------

fn annotate(
    l: &mut Ledger,
    f: &FindingId,
    a: impl FnOnce(&mut FindingAnnotation),
) -> Result<(), Error> {
    let mut rec = FindingAnnotation::new(f.clone());
    a(&mut rec);
    l.annotate_record(rec).map(|_| ())
}

#[test]
fn annotate_fills_holder_owner_and_region_once() {
    let mut l = m3_ledger();
    let f = match l
        .add_finding(NewFinding::new(origin("census"), vec![eid(E_STATIC)]))
        .unwrap()
        .record
    {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    annotate(&mut l, &f, |a| {
        a.holder = Some(Holder::RegistrationTable);
        a.owner = Some("Bus.listeners".into());
        a.region = Some(Region::Heap);
    })
    .unwrap();
    let fin = l.finding(&f).unwrap();
    assert_eq!(fin.holder, Some(Holder::RegistrationTable));
    assert_eq!(fin.owner.as_deref(), Some("Bus.listeners"));
    assert_eq!(fin.region, Some(Region::Heap));
    // The same values again are accepted; a different value is a conflict.
    annotate(&mut l, &f, |a| a.holder = Some(Holder::RegistrationTable)).unwrap();
    for field in ["holder", "owner", "region"] {
        let r = refusal(annotate(&mut l, &f, |a| match field {
            "holder" => a.holder = Some(Holder::ThreadLocal),
            "owner" => a.owner = Some("Other.field".into()),
            _ => a.region = Some(Region::OffHeap),
        }));
        assert_eq!(r.code, codes::ANNOTATION_CONFLICT, "{field}");
        assert!(r.message.contains(field), "{}", r.message);
    }
    // Nothing changed.
    let fin = l.finding(&f).unwrap();
    assert_eq!(fin.holder, Some(Holder::RegistrationTable));
    // The annotation is part of the replayed state.
    let again = Ledger::from_text(&to_text(&l)).unwrap();
    assert_eq!(
        again.finding(&f).unwrap().owner.as_deref(),
        Some("Bus.listeners")
    );
}

#[test]
fn annotate_sets_a_missing_site_and_refuses_a_different_one() {
    let mut l = m3_ledger();
    let f = match l
        .add_finding(NewFinding::new(origin("census"), vec![eid(E_STATIC)]))
        .unwrap()
        .record
    {
        dla_core::Record::FindingAdded { finding } => finding.id,
        _ => unreachable!(),
    };
    assert!(l.finding(&f).unwrap().site.is_none());
    // A malformed site is refused.
    let r = refusal(annotate(&mut l, &f, |a| a.site = Some("nocolon".into())));
    assert_eq!(r.code, codes::INVALID_SITE);
    annotate(&mut l, &f, |a| a.site = Some("src/a.go:12".into())).unwrap();
    assert_eq!(l.finding(&f).unwrap().site.as_deref(), Some("src/a.go:12"));
    // The same site is accepted; a different one is a conflict and changes nothing.
    annotate(&mut l, &f, |a| a.site = Some("src/a.go:12".into())).unwrap();
    let r = refusal(annotate(&mut l, &f, |a| a.site = Some("src/b.go:3".into())));
    assert_eq!(r.code, codes::ANNOTATION_CONFLICT);
    assert!(r.message.contains("site"), "{}", r.message);
    assert_eq!(l.finding(&f).unwrap().site.as_deref(), Some("src/a.go:12"));
    // Replay derives the site from the annotation line.
    let again = Ledger::from_text(&to_text(&l)).unwrap();
    assert_eq!(
        again.finding(&f).unwrap().site.as_deref(),
        Some("src/a.go:12")
    );
}

#[test]
fn annotate_still_needs_something_in_it_and_a_finding() {
    let (mut l, f, _) = pair();
    let r = refusal(annotate(&mut l, &f, |_| {}));
    assert_eq!(r.code, codes::EMPTY_ANNOTATION);
    let r = refusal(annotate(&mut l, &f, |a| a.owner = Some("  ".into())));
    assert_eq!(r.code, codes::EMPTY_FIELD);
    let r = refusal(annotate(&mut l, &fid(9), |a| {
        a.holder = Some(Holder::CycleOnly)
    }));
    assert_eq!(r.code, codes::UNKNOWN_FINDING);
    // Writers keep working next to the new fields; a holder-only annotation is not empty.
    annotate(&mut l, &f, |a| a.writers.push("a.go:3".into())).unwrap();
    assert_eq!(l.annotation(&f).unwrap().writers, ["a.go:3"]);
}

// ----- versions --------------------------------------------------------------------

#[test]
fn a_ledger_written_by_the_api_replays_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let mut l = Ledger::create(dir.path()).unwrap();
    for e in m3_ledger().evidence() {
        l.add_evidence(dla_core::NewEvidence {
            source: e.source.clone(),
            tool: e.tool.clone(),
            authority: e.authority,
            kind: e.kind,
            result: e.result.clone(),
            raw: None,
            coverage: e.coverage.clone(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
    }
    let (f, _) = confirmed_leak(&mut l);
    let text = std::fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    assert!(text.lines().all(|t| t.contains("\"schema_version\":5")));
    let again = Ledger::load(dir.path()).unwrap();
    assert_eq!(again.finding(&f).unwrap(), l.finding(&f).unwrap());
    assert_eq!(again.finding(&f).unwrap().shape, Some(Shape::Steady));
}

#[test]
fn a_version_one_ledger_stays_valid_and_new_lines_are_the_current_version() {
    let dir = tempfile::tempdir().unwrap();
    let v1 = [
        line(1, EVIDENCE_BODY),
        line(
            2,
            r#""type":"finding_added","finding":{"id":"F1","state":"candidate","origin":"census","supports":["E1"]}"#,
        ),
        line(
            3,
            r#""type":"transition","finding":"F1","from":"candidate","to":"under_test","evidence":[],"by":"x""#,
        ),
    ]
    .concat();
    std::fs::write(dir.path().join("ledger.jsonl"), &v1).unwrap();
    let mut l = Ledger::load(dir.path()).unwrap();
    assert_eq!(l.finding(&fid(1)).unwrap().state, S::UnderTest);
    // A v3 line appended now is judged by the M3 rules (plus the artifact rules): no harness, no confirm.
    let r = refusal(mv(&mut l, &fid(1), S::Confirmed, &[]));
    assert_eq!(r.code, codes::GROWTH_GATE_FAILED);
    l.open_item_add("still open", None, "test").unwrap();
    let text = std::fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    let versions: Vec<_> = text
        .lines()
        .map(|t| serde_json::from_str::<serde_json::Value>(t).unwrap()["schema_version"].clone())
        .collect();
    assert_eq!(versions, [1, 1, 1, 5]);
    Ledger::load(dir.path()).unwrap();
}
