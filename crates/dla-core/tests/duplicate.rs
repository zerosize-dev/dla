//! `dla ledger duplicate`: the rule (same scope, then the same owner or the same site), the
//! refusals, the replay of the `finding_merged` record and the gate's label item.

mod common;

use common::*;
use dla_core::{
    Error, FindingId, FindingState as S, Ledger, NewFinding, Record, Refusal,
    archetypes::Table,
    duplicate::{DuplicateBy, labelled_twins},
    finding::Holder,
    labels::Archetypes,
    m6::codes as m6,
};

fn refusal<T: std::fmt::Debug>(r: Result<T, Error>) -> Refusal {
    r.expect_err("expected a refusal").to_refusal()
}

fn add(l: &mut Ledger, site: Option<&str>, owner: Option<&str>, origin_name: &str) -> FindingId {
    let mut nf = NewFinding::new(origin(origin_name), vec![eid(E_STATIC)]);
    nf.site = site.map(Into::into);
    nf.owner = owner.map(Into::into);
    nf.holder = Some(Holder::StaticGlobal);
    let line = l.add_finding(nf).unwrap();
    let Record::FindingAdded { finding } = line.record else {
        unreachable!()
    };
    finding.id
}

fn text_of(l: &Ledger) -> String {
    l.lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_same_site_is_a_duplicate_and_the_merge_carries_the_supports() {
    let mut l = m3_ledger();
    let (f1, _) = confirmed_leak(&mut l);
    let mut nf = NewFinding::new(origin("census"), vec![eid(E_DISCOVER)]);
    nf.site = Some("a.go:1".into());
    let Record::FindingAdded { finding } = l.add_finding(nf).unwrap().record else {
        unreachable!()
    };
    let f3 = finding.id;
    assert!(
        !l.finding(&f1).unwrap().supports.contains(&eid(E_DISCOVER)),
        "the canonical finding does not cite it yet"
    );
    let (line, by) = l.duplicate_finding(&f3, &f1).unwrap();
    assert!(matches!(by, DuplicateBy::Site(..)), "{by:?}");
    let Record::FindingMerged(m) = &line.record else {
        panic!("{:?}", line.record)
    };
    assert_eq!((&m.canonical, &m.duplicate), (&f1, &f3));
    // The merged state: the duplicate points to the canonical finding, which stays
    // confirmed and now carries the duplicate's supports.
    assert_eq!(l.merged_into(&f3), Some(&f1));
    assert_eq!(l.merged_into(&f1), None);
    assert_eq!(l.finding(&f1).unwrap().state, S::Confirmed);
    let supports = &l.finding(&f1).unwrap().supports;
    assert!(supports.contains(&eid(E_DISCOVER)), "{supports:?}");
    assert!(supports.contains(&eid(E_STATIC)), "{supports:?}");
}

#[test]
fn a_site_that_is_not_the_same_line_is_not_a_duplicate() {
    let mut l = m3_ledger();
    let (f1, _) = confirmed_leak(&mut l);
    // Near, far and in another file: exact identity only.
    let near = add(&mut l, Some("a.go:3"), None, "census");
    let far = add(&mut l, Some("a.go:40"), None, "census");
    let other = add(&mut l, Some("b.go:1"), None, "census");
    for dup in [&near, &far, &other] {
        let r = refusal(l.duplicate_finding(dup, &f1));
        assert_eq!(r.code, m6::DUPLICATE_NOT_SAME, "{}", r.message);
        assert!(r.fix.contains("annotate"), "{}", r.fix);
    }
    assert_eq!(l.merged_into(&near), None);
}

#[test]
fn only_the_same_canonical_owner_is_a_duplicate() {
    let mut l = m3_ledger();
    let a = add(
        &mut l,
        None,
        Some("reports.builder.ReportBuilder.totals"),
        "growth-series",
    );
    let same = add(
        &mut l,
        None,
        Some("reports.builder.ReportBuilder.totals"),
        "growth-series",
    );
    let suffix = add(
        &mut l,
        None,
        Some("builder.ReportBuilder.totals"),
        "growth-series",
    );
    let bare = add(&mut l, None, Some("totals"), "growth-series");
    let other = add(
        &mut l,
        None,
        Some("reports.builder.Other.totals"),
        "growth-series",
    );
    let (_, by) = l.duplicate_finding(&same, &a).unwrap();
    assert!(matches!(by, DuplicateBy::Owner(..)), "{by:?}");
    assert_eq!(l.merged_into(&same), Some(&a));
    for dup in [&suffix, &bare, &other] {
        let r = refusal(l.duplicate_finding(dup, &a));
        assert_eq!(r.code, m6::DUPLICATE_NOT_SAME, "{}", r.message);
    }
}

#[test]
fn findings_of_another_runtime_by_site_language_are_not_duplicates_even_with_one_owner() {
    let mut l = m3_ledger();
    let a = add(
        &mut l,
        Some("svc/app.py:3"),
        Some("svc.app.cache"),
        "census",
    );
    let b = add(
        &mut l,
        Some("web/app.js:9"),
        Some("svc.app.cache"),
        "census",
    );
    let r = refusal(l.duplicate_finding(&b, &a));
    assert_eq!(r.code, m6::DUPLICATE_NOT_SAME, "{}", r.message);
    // Replay refuses the record too, whoever wrote it.
    let r = refusal(l.merge_findings(&a, &b));
    assert_eq!(r.code, "merge-state", "{}", r.message);
}

#[test]
fn a_bare_owner_equal_on_both_sides_is_not_a_duplicate() {
    // Only qualified owners (a class or module part) identify a field; `cache` alone is a
    // name that any module may use.
    let mut l = m3_ledger();
    let a = add(&mut l, None, Some("cache"), "growth-series");
    let b = add(&mut l, None, Some("cache"), "growth-series");
    let r = refusal(l.duplicate_finding(&b, &a));
    assert_eq!(r.code, m6::DUPLICATE_NOT_SAME, "{}", r.message);
    assert_eq!(l.merged_into(&b), None);
}

#[test]
fn the_worker_is_part_of_the_scope_in_core_mode() {
    // The same owner and the same site in the core ledger, in `jvm/F1` and in `rust/F1`:
    // a core finding is not a duplicate of a worker's, nor is one worker's of another's.
    let dir = new_run(&["jvm", "rust"]);
    let mut core = Ledger::load(dir.path()).unwrap();
    let c = add(
        &mut core,
        Some("svc/Cache.java:9"),
        Some("svc.Cache.entries"),
        "census",
    );
    write_main(dir.path(), &core);
    // Another file for the second worker, so the merge of the slices keeps them apart;
    // both are Java files, so only the worker tells the scopes apart.
    for (w, site) in [("jvm", "svc/Cache.java:9"), ("rust", "svc/Other.java:2")] {
        let mut s = slice(dir.path(), w);
        let id = candidate_in_slice(&mut s, site, Holder::StaticGlobal);
        let mut a = dla_core::record::FindingAnnotation::new(id);
        a.owner = Some("svc.Cache.entries".into());
        s.annotate_record(a).unwrap();
    }
    dla_core::merge::merge(dir.path()).unwrap();
    let mut l = Ledger::load(dir.path()).unwrap();
    let jvm: FindingId = "jvm/F1".parse().unwrap();
    let rust: FindingId = "rust/F1".parse().unwrap();
    assert!(l.finding(&jvm).is_some() && l.finding(&rust).is_some());
    for (dup, canon) in [(&jvm, &c), (&c, &jvm), (&jvm, &rust), (&rust, &jvm)] {
        let r = refusal(l.duplicate_finding(dup, canon));
        assert_eq!(
            r.code,
            m6::DUPLICATE_NOT_SAME,
            "{dup} {canon}: {}",
            r.message
        );
    }
    // The replay check of the record refuses a core finding and a worker's finding too,
    // whoever wrote the line.
    for (a, b) in [(&c, &jvm), (&jvm, &c)] {
        let r = refusal(l.merge_findings(a, b));
        assert_eq!(r.code, "merge-state", "{}", r.message);
    }
}

#[test]
fn the_python_report_memo_shape_still_merges() {
    // python-report-memo/3: three findings at the site of F4, which owns the field; four more
    // name the same owner (some at the next line).
    let mut l = m3_ledger();
    let owner = "reports.builder.ReportBuilder.category_totals";
    let f4 = add(&mut l, Some("reports/builder.py:14"), Some(owner), "census");
    let sites = add(&mut l, Some("reports/builder.py:14"), None, "census");
    let by_owner = add(
        &mut l,
        Some("reports/builder.py:15"),
        Some(owner),
        "growth-series",
    );
    let (_, by) = l.duplicate_finding(&sites, &f4).unwrap();
    assert!(matches!(by, DuplicateBy::Site(..)), "{by:?}");
    let (_, by) = l.duplicate_finding(&by_owner, &f4).unwrap();
    assert!(matches!(by, DuplicateBy::Owner(..)), "{by:?}");
    let again = Ledger::from_text(&text_of(&l)).unwrap();
    assert_eq!(again.merged_into(&sites), Some(&f4));
    assert_eq!(again.merged_into(&by_owner), Some(&f4));
}

#[test]
fn hypotheses_itself_merged_and_merged_canonical_are_refused() {
    let mut l = m3_ledger();
    let (f1, n) = confirmed_leak(&mut l);
    let f3 = add(&mut l, Some("a.go:1"), None, "census");
    let f4 = add(&mut l, Some("a.go:1"), None, "census");
    let hyp = add(&mut l, Some("a.go:1"), None, "hypothesis");
    assert_eq!(
        refusal(l.duplicate_finding(&f1, &f1)).code,
        m6::DUPLICATE_STATE
    );
    assert_eq!(
        refusal(l.duplicate_finding(&n, &f1)).code,
        m6::DUPLICATE_STATE
    );
    assert_eq!(
        refusal(l.duplicate_finding(&f3, &n)).code,
        m6::DUPLICATE_STATE
    );
    assert_eq!(
        refusal(l.duplicate_finding(&hyp, &f1)).code,
        m6::DUPLICATE_STATE
    );
    l.duplicate_finding(&f3, &f1).unwrap();
    let again = refusal(l.duplicate_finding(&f3, &f1));
    assert_eq!(again.code, m6::DUPLICATE_STATE);
    assert!(
        again.message.contains("already merged"),
        "{}",
        again.message
    );
    // A duplicate cannot be the canonical side; the fix names its canonical finding.
    let r = refusal(l.duplicate_finding(&f4, &f3));
    assert_eq!(r.code, m6::DUPLICATE_STATE);
    assert!(r.fix.contains(&format!("--of {f1}")), "{}", r.fix);
    // The canonical finding as the duplicate of its own duplicate: the fix never says
    // `--of` the finding itself.
    let r = refusal(l.duplicate_finding(&f1, &f3));
    assert_eq!(r.code, m6::DUPLICATE_STATE);
    assert!(!r.fix.contains(&format!("{f1} --of {f1}")), "{}", r.fix);
    assert!(r.fix.contains(&f1.to_string()), "{}", r.fix);
}

#[test]
fn a_refuted_finding_is_a_verdict_not_a_duplicate() {
    let mut l = m3_ledger();
    let (f1, _) = confirmed_leak(&mut l);
    let dead = add(&mut l, Some("a.go:1"), None, "census");
    let n = add_sibling(&mut l, &dead, "N-WARM");
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &dead, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &dead, S::Refuted, &[E_DISC_NONLEAK]);
    assert_eq!(l.finding(&dead).unwrap().state, S::Refuted);
    // Either side.
    let r = refusal(l.duplicate_finding(&dead, &f1));
    assert_eq!(r.code, m6::DUPLICATE_STATE);
    assert!(r.message.contains("refuted"), "{}", r.message);
    let r = refusal(l.duplicate_finding(&f1, &dead));
    assert_eq!(r.code, m6::DUPLICATE_STATE);
    assert_eq!(l.merged_into(&dead), None);
}

#[test]
fn the_record_replays() {
    let mut l = m3_ledger();
    let (f1, _) = confirmed_leak(&mut l);
    let f3 = add(&mut l, Some("a.go:1"), None, "census");
    l.duplicate_finding(&f3, &f1).unwrap();
    let again = Ledger::from_text(&text_of(&l)).unwrap();
    assert_eq!(again.merged_into(&f3), Some(&f1));
}

#[test]
fn a_worker_merges_its_own_duplicates_in_its_slice() {
    let dir = new_run(&["py"]);
    let mut w = slice(dir.path(), "py");
    let a = candidate_in_slice(&mut w, "r/b.py:14", Holder::ContainerField);
    let b = candidate_in_slice(&mut w, "r/b.py:14", Holder::ContainerField);
    let (_, by) = w.duplicate_finding(&b, &a).unwrap();
    assert!(matches!(by, DuplicateBy::Site(..)));
    assert_eq!(w.merged_into(&b), Some(&a));
    // A finding of the main ledger is not the worker's own.
    let r = refusal(w.duplicate_finding(&b, &"F1".parse().unwrap()));
    assert!(
        matches!(r.code.as_str(), "unknown-finding" | "duplicate-state"),
        "{}",
        r.code
    );
    // The slice replays with the record.
    let reloaded = slice(dir.path(), "py");
    assert_eq!(reloaded.merged_into(&b), Some(&a));
    // `dla ledger merge` carries the record into the main ledger.
    dla_core::merge::merge(dir.path()).unwrap();
    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(main.merged_into(&b), Some(&a));
}

const TABLE: &str = r#"
[L-COLLECT]
holders = ["static/global", "container-field"]
regions = ["heap"]
requires = [["growth-series"]]
disconfirm = []
repair = ["bound/evict"]
summary = "A collection gains entries and nothing removes them."
"#;

#[test]
fn labelled_twins_are_the_findings_of_the_same_leak_with_a_label_decision() {
    let mut l = m3_ledger();
    let (f1, _) = confirmed_leak(&mut l);
    let t = Table::parse(TABLE, "t").unwrap();
    let f3 = add(&mut l, Some("a.go:1"), None, "census");
    // Nothing is labelled yet.
    assert!(labelled_twins(&l, l.finding(&f3).unwrap()).is_empty());
    l.label(
        &f1,
        "L-COLLECT",
        &[eid(E_GROWTH_PASS)],
        None,
        Archetypes::On(&t),
    )
    .unwrap();
    let f3f = l.finding(&f3).unwrap();
    let twins = labelled_twins(&l, f3f);
    assert_eq!(twins.iter().map(|x| &x.id).collect::<Vec<_>>(), [&f1]);
    // Once merged, the duplicate has no twin to look for and the canonical is not offered.
    l.duplicate_finding(&f3, &f1).unwrap();
    assert_eq!(l.merged_into(&f3), Some(&f1));
}
