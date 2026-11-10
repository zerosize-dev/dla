//! Schema 6 rules: `finding_labeled`, `strong`, `finding_explained`, the verification
//! label and the refusals that go with them.

mod common;

use common::*;
use dla_core::{
    Authority, Error, EvidenceKind, FindingId, FindingState as S, Ledger, Record, Refusal,
    archetypes::Table,
    error::codes,
    finding::{ExplanationTag, Holder},
    labels::Archetypes,
    m6::codes as m6,
    record::RemovalClaim,
};
use serde_json::json;

const TABLE: &str = r#"
[L-COLLECT]
holders = ["static/global", "container-field"]
regions = ["heap"]
requires = [["growth-series", "retention-path"], ["unit-count-series"]]
disconfirm = ["N-WARM"]
repair = ["bound/evict"]
summary = "A collection gains entries and nothing removes them."

[L-HANDLE]
holders = ["native-handle/global-ref"]
regions = ["off-heap"]
requires = [["growth-series", "alloc-stack"]]
disconfirm = []
repair = ["release-regular"]
summary = "A native handle is acquired and never released."

[L-CYCLE]
holders = ["static/global"]
regions = ["heap"]
requires = [["growth-series", "discriminator"]]
disconfirm = []
repair = ["clear-reference"]
summary = "A reference cycle keeps the objects alive."

[N-WARM]
requires = [["growth-series", "discriminator"]]
disconfirm = []
summary = "Growth that levels off after warm-up."
"#;

fn table() -> Table {
    Table::parse(TABLE, "test table").unwrap()
}

fn refusal<T: std::fmt::Debug>(r: Result<T, Error>) -> Refusal {
    r.expect_err("expected a refusal").to_refusal()
}

/// F1 (site a.go:1, holder static/global) confirmed on E4 with a refuted N-WARM sibling F1.n1.
fn leak() -> (Ledger, FindingId, FindingId) {
    let mut l = m3_ledger();
    let (f, n) = confirmed_leak(&mut l);
    (l, f, n)
}

fn label(
    l: &mut Ledger,
    f: &FindingId,
    as_: &str,
    ev: &[u32],
) -> Result<dla_core::LedgerLine, Error> {
    let t = table();
    let ev: Vec<_> = ev.iter().map(|n| eid(*n)).collect();
    l.label(f, as_, &ev, None, Archetypes::On(&t))
}

fn record_of(line: &dla_core::LedgerLine) -> &dla_core::FindingLabeled {
    match &line.record {
        Record::FindingLabeled(r) => r,
        other => panic!("not a label: {other:?}"),
    }
}

// ------------------------------------------------------------------------------- labels

#[test]
fn a_label_records_the_met_alternative_and_sets_the_archetype() {
    let (mut l, f, _) = leak();
    let r = push(&mut l, vgdb_retention("bb"));
    let line = label(&mut l, &f, "L-COLLECT", &[E_GROWTH_PASS, r]).unwrap();
    let rec = record_of(&line);
    assert_eq!(rec.label, "L-COLLECT");
    assert_eq!(rec.previous, "UNCLASSIFIED");
    assert_eq!(rec.satisfied.kinds, ["growth-series", "retention-path"]);
    assert_eq!(rec.satisfied.evidence, [eid(E_GROWTH_PASS), eid(r)]);
    assert_eq!(rec.entry_sha256.len(), 64);
    assert!(!rec.region_mismatch);
    assert_eq!(l.finding(&f).unwrap().archetype, "L-COLLECT");
    assert_eq!(l.labels(&f).len(), 1);
}

#[test]
fn the_default_evidence_is_what_the_finding_cites() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    // E11 is a passing unit-count column at the finding's site: alternative two.
    go(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS, E_UNIT_PASS]);
    let line = label(&mut l, &f, "L-COLLECT", &[]).unwrap();
    assert_eq!(record_of(&line).satisfied.kinds, ["unit-count-series"]);
}

#[test]
fn unmet_kinds_are_named_with_the_command_that_produces_them() {
    let (mut l, f, _) = leak();
    let r = refusal(label(&mut l, &f, "L-HANDLE", &[]));
    assert_eq!(r.code, m6::LABEL_EVIDENCE_MISSING);
    assert!(r.message.contains("alloc-stack"), "{}", r.message);
    assert!(
        r.fix.contains("dla summarize alloc-stack-diff"),
        "{}",
        r.fix
    );
    assert_eq!(l.finding(&f).unwrap().archetype, "UNCLASSIFIED");
}

#[test]
fn records_the_finding_does_not_cite_are_named_in_the_fix() {
    let (mut l, f, _) = leak();
    // E11 ties to F1 by site and meets L-COLLECT's second alternative, but F1 cites E4 only.
    let r = refusal(label(&mut l, &f, "L-COLLECT", &[]));
    assert_eq!(r.code, m6::LABEL_EVIDENCE_MISSING);
    assert_eq!(
        r.fix,
        format!("dla ledger label F1 --as L-COLLECT --evidence E{E_UNIT_PASS}")
    );
}

#[test]
fn a_growth_record_tied_to_another_finding_does_not_count() {
    let (mut l, f, _) = leak();
    // E13 is passing growth for F9 at b.go:2.
    let r = push(&mut l, vgdb_retention("bb"));
    let e = refusal(label(&mut l, &f, "L-COLLECT", &[E_GROWTH_OTHER, r]));
    assert_eq!(e.code, m6::LABEL_EVIDENCE_MISSING);
    assert!(
        e.message.contains("no passing column tied"),
        "{}",
        e.message
    );
}

#[test]
fn a_retention_record_for_another_holder_does_not_count() {
    let (mut l, f, _) = leak();
    let far = signed(
        EvidenceKind::RetentionPath,
        Authority::Live,
        "valgrind-vgdb",
        "leak-checker",
        "cc",
        json!({"root": "static other", "path": [], "alloc": "grow (z.go:90)"}),
    );
    let r = push(&mut l, far);
    let e = refusal(label(&mut l, &f, "L-COLLECT", &[E_GROWTH_PASS, r]));
    assert_eq!(e.code, m6::LABEL_EVIDENCE_MISSING);
    assert!(e.message.contains("names no holder"), "{}", e.message);
}

#[test]
fn polarity_and_hypothesis_are_checked() {
    let (mut l, f, n) = leak();
    let e = refusal(label(&mut l, &f, "N-WARM", &[]));
    assert_eq!(e.code, m6::LABEL_POLARITY);
    // The sibling is refuted, so state comes first for it.
    let e = refusal(label(&mut l, &n, "L-COLLECT", &[]));
    assert_eq!(e.code, m6::LABEL_STATE);
}

#[test]
fn a_non_leak_sibling_takes_its_own_n_label_only() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &f, S::Refuted, &[E_DISC_NONLEAK_F1]);
    go(&mut l, &n, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &n, S::Confirmed, &[E_DISC_NONLEAK_F1]);
    let e = refusal(label(&mut l, &n, "L-COLLECT", &[]));
    assert_eq!(e.code, m6::LABEL_POLARITY);
    let t = Table::parse(
        &format!("{TABLE}\n[N-HWM]\nrequires=[[\"discriminator\"]]\nsummary=\"x\"\n"),
        "t",
    )
    .unwrap();
    let e = refusal(l.label(&n, "N-HWM", &[], None, Archetypes::On(&t)));
    assert_eq!(e.code, m6::LABEL_HYPOTHESIS_MISMATCH);
    // N-WARM needs growth tied to the family and a discriminator favoring non-leak.
    let e = refusal(label(&mut l, &n, "N-WARM", &[E_GROWTH_PASS]));
    assert_eq!(e.code, m6::LABEL_EVIDENCE_MISSING);
    let line = label(&mut l, &n, "N-WARM", &[E_GROWTH_PASS, E_DISC_NONLEAK_F1]).unwrap();
    assert_eq!(record_of(&line).label, "N-WARM");
    assert_eq!(l.finding(&n).unwrap().archetype, "N-WARM");
    // A discriminator that favors leak does not qualify.
    let (mut l2, f2, _) = leak();
    let e = refusal(label(
        &mut l2,
        &f2,
        "L-CYCLE",
        &[E_GROWTH_PASS, E_DISC_LEAK_F9],
    ));
    assert_eq!(e.code, m6::LABEL_EVIDENCE_MISSING);
}

#[test]
fn a_leak_archetype_needs_a_discriminator_that_favors_leak() {
    let (mut l, f, _) = leak();
    // E15 favors non-leak and is bound to F1; E16 favors leak and is bound to F1.
    let e = refusal(label(
        &mut l,
        &f,
        "L-CYCLE",
        &[E_GROWTH_PASS, E_DISC_NONLEAK_F1],
    ));
    assert_eq!(e.code, m6::LABEL_EVIDENCE_MISSING);
    assert!(e.message.contains("does not favor leak"), "{}", e.message);
    let line = label(&mut l, &f, "L-CYCLE", &[E_GROWTH_PASS, E_DISC_LEAK_F1]).unwrap();
    assert_eq!(record_of(&line).label, "L-CYCLE");
}

#[test]
fn only_confirmed_or_strong_findings_are_labelled() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let e = refusal(label(&mut l, &f, "L-COLLECT", &[]));
    assert_eq!(e.code, m6::LABEL_STATE);
    let e = refusal(l.label(
        &f,
        "UNCLASSIFIED",
        &[],
        Some("why"),
        Archetypes::On(&table()),
    ));
    assert_eq!(e.code, m6::LABEL_STATE);
}

#[test]
fn the_holder_must_fit_the_archetype() {
    let (mut l, f, _) = leak();
    let r = push(&mut l, vgdb_retention("bb"));
    // F1's holder is static/global; L-HANDLE fits native-handle/global-ref only. Its evidence
    // is met first, so the holder is what is refused.
    let alloc = push(
        &mut l,
        signed(
            EvidenceKind::AllocStack,
            Authority::Live,
            "jfr",
            "alloc-profiler",
            "dd",
            json!({"mode": "diff", "finding": "F1", "top": []}),
        ),
    );
    let e = refusal(label(&mut l, &f, "L-HANDLE", &[E_GROWTH_PASS, alloc]));
    assert_eq!(e.code, m6::LABEL_HOLDER);
    assert!(
        e.message.contains("native-handle/global-ref"),
        "{}",
        e.message
    );
    assert!(e.fix.contains("ledger annotate F1 --holder"), "{}", e.fix);
    // And the archetype that fits goes through.
    label(&mut l, &f, "L-COLLECT", &[E_GROWTH_PASS, r]).unwrap();
}

/// An allocation-stack record as `dla summarize alloc-stack-diff` writes it for a profile
/// with no root path (heaptrack, valgrind): the stacks name no holder, and the record
/// states the findings its growing stacks were tied to.
fn alloc_stack_tied_to(finding: Option<&str>, src: &str) -> dla_core::evidence::NewEvidence {
    let mut r = json!({"mode": "diff", "captures": 3, "live": true,
        "top": [{"at": "grow", "site": "a.go:1", "d_bytes": 4096}]});
    if let Some(f) = finding {
        r["finding"] = json!(f);
        r["candidates"] = json!([f]);
        r["ties"] = json!([{"stack": "grow", "finding": f, "candidates": [f],
            "tie": {"rule": "site", "key": "a.go:1"}}]);
    }
    signed(
        EvidenceKind::AllocStack,
        Authority::Live,
        "heaptrack",
        "alloc-profiler",
        src,
        r,
    )
}

#[test]
fn an_alloc_stack_record_counts_for_the_findings_its_stated_ties_name_only() {
    let (mut l, f, _) = leak();
    // The record names F1 as the finding of its growing stack: the evidence is met, and
    // only the holder (static/global is not a handle) is refused.
    let tied = push(&mut l, alloc_stack_tied_to(Some("F1"), "ee"));
    let e = refusal(label(&mut l, &f, "L-HANDLE", &[E_GROWTH_PASS, tied]));
    assert_eq!(e.code, m6::LABEL_HOLDER, "{}", e.message);
    // A record that states no tie (no root path, no finding) or another finding's tie
    // does not count, whatever its stack sites are.
    for (finding, src) in [(None, "ff"), (Some("F9"), "gg")] {
        let other = push(&mut l, alloc_stack_tied_to(finding, src));
        let e = refusal(label(&mut l, &f, "L-HANDLE", &[E_GROWTH_PASS, other]));
        assert_eq!(e.code, m6::LABEL_EVIDENCE_MISSING, "{}", e.message);
        assert!(e.message.contains("names no holder"), "{}", e.message);
    }
}

/// A table whose only leak archetype rests on a record whose result names no finding.
const CENSUS_TABLE: &str = r#"
[L-COLLECT]
holders = ["static/global"]
regions = ["heap"]
requires = [["growth-series", "census-holder"]]
disconfirm = ["N-WARM"]
repair = ["clear-reference"]
summary = "A collection gains entries and nothing removes them."

[N-WARM]
requires = [["growth-series", "discriminator"]]
disconfirm = []
summary = "Growth that levels off after warm-up."
"#;

fn census_record(src: &str) -> dla_core::evidence::NewEvidence {
    signed(
        EvidenceKind::CensusHolder,
        Authority::Static,
        "dla-census",
        "static-scan",
        src,
        json!({"holders": 7, "by_holder": {"static/global": 3}}),
    )
}

#[test]
fn a_record_that_names_no_finding_counts_through_the_findings_own_citation_only() {
    let t = Table::parse(CENSUS_TABLE, "census table").unwrap();
    let lab = |l: &mut Ledger, f: &FindingId, ev: &[u32]| {
        let ev: Vec<_> = ev.iter().map(|n| eid(*n)).collect();
        l.label(f, "L-COLLECT", &ev, None, Archetypes::On(&t))
    };
    // Cited by the refuted sibling F1.n1 (its family), not by F1.
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    let by_sibling = push(&mut l, census_record("c1"));
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK, by_sibling]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS]);
    for ev in [vec![E_GROWTH_PASS, by_sibling], vec![]] {
        let e = refusal(lab(&mut l, &f, &ev));
        assert_eq!(e.code, m6::LABEL_EVIDENCE_MISSING, "{}", e.message);
        assert!(
            e.message.contains("census-holder") || e.message.contains("does not cite"),
            "{}",
            e.message
        );
    }
    // Cited by F1 itself on its confirmation: counts, by default too.
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    let own = push(&mut l, census_record("c2"));
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS, own]);
    let line = lab(&mut l, &f, &[E_GROWTH_PASS, own]).unwrap();
    assert_eq!(
        record_of(&line).satisfied.kinds,
        ["growth-series", "census-holder"]
    );
}

#[test]
fn a_disconfirm_sibling_that_is_not_refuted_blocks_the_label() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    let hwm = add_sibling(&mut l, &f, "N-HWM");
    go(&mut l, &n, S::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, S::Confirmed, &[E_GROWTH_PASS, E_UNIT_PASS]);
    // L-COLLECT lists N-WARM only: the open N-HWM does not matter.
    label(&mut l, &f, "L-COLLECT", &[]).unwrap();
    // A table that also lists N-HWM refuses until it is refuted.
    let t = Table::parse(
        &TABLE.replace(
            r#"disconfirm = ["N-WARM"]"#,
            r#"disconfirm = ["N-WARM", "N-HWM"]"#,
        ),
        "t",
    )
    .unwrap();
    let e = refusal(l.label(&f, "L-COLLECT", &[], None, Archetypes::On(&t)));
    assert_eq!(e.code, m6::LABEL_DISCONFIRM_OPEN);
    assert!(e.message.contains(&hwm.to_string()), "{}", e.message);
    go(&mut l, &hwm, S::Refuted, &[E_DISC_LEAK]);
    l.label(&f, "L-COLLECT", &[], None, Archetypes::On(&t))
        .unwrap();
}

#[test]
fn a_relabel_keeps_the_previous_label() {
    let (mut l, f, _) = leak();
    let r = push(&mut l, vgdb_retention("bb"));
    label(&mut l, &f, "L-COLLECT", &[E_GROWTH_PASS, r]).unwrap();
    let line = l
        .label(
            &f,
            "UNCLASSIFIED",
            &[],
            Some("verifier disputes the label"),
            Archetypes::On(&table()),
        )
        .unwrap();
    let rec = record_of(&line);
    assert_eq!(rec.previous, "L-COLLECT");
    assert!(rec.satisfied.kinds.is_empty() && rec.entry_sha256.is_empty());
    assert_eq!(l.finding(&f).unwrap().archetype, "UNCLASSIFIED");
    assert_eq!(l.labels(&f).len(), 2);
}

#[test]
fn unclassified_needs_a_note_and_unknown_ids_list_the_table() {
    let (mut l, f, _) = leak();
    let e = refusal(l.label(&f, "UNCLASSIFIED", &[], None, Archetypes::On(&table())));
    assert_eq!(e.code, codes::EMPTY_FIELD);
    let e = refusal(l.label(
        &f,
        "UNCLASSIFIED",
        &[],
        Some("  "),
        Archetypes::On(&table()),
    ));
    assert_eq!(e.code, codes::EMPTY_FIELD);
    let e = refusal(label(&mut l, &f, "L-NOPE", &[]));
    assert_eq!(e.code, m6::UNKNOWN_ARCHETYPE);
    assert!(
        e.fix.contains("L-COLLECT") && e.fix.contains("UNCLASSIFIED"),
        "{}",
        e.fix
    );
}

#[test]
fn with_archetypes_off_only_unclassified_is_accepted() {
    let (mut l, f, _) = leak();
    let e = refusal(l.label(&f, "L-COLLECT", &[], None, Archetypes::Off));
    assert_eq!(e.code, m6::ARCHETYPES_OFF);
    let e = refusal(l.label(&f, "N-WARM", &[], None, Archetypes::Off));
    assert_eq!(e.code, m6::ARCHETYPES_OFF);
    l.label(
        &f,
        "UNCLASSIFIED",
        &[],
        Some("holder and mechanism in words"),
        Archetypes::Off,
    )
    .unwrap();
    assert_eq!(l.finding(&f).unwrap().archetype, "UNCLASSIFIED");
}

#[test]
fn a_region_that_does_not_fit_is_recorded_not_refused() {
    let (mut l, f, _) = leak();
    let r = push(&mut l, vgdb_retention("bb"));
    // F1's region is heap (derived on confirm); a table that wants off-heap only warns.
    let t = Table::parse(
        &TABLE.replace(
            r#"regions = ["heap"]
requires = [["growth-series", "retention-path"]"#,
            r#"regions = ["off-heap"]
requires = [["growth-series", "retention-path"]"#,
        ),
        "t",
    )
    .unwrap();
    let line = l
        .label(
            &f,
            "L-COLLECT",
            &[eid(E_GROWTH_PASS), eid(r)],
            None,
            Archetypes::On(&t),
        )
        .unwrap();
    assert!(record_of(&line).region_mismatch);
}

#[test]
fn replay_re_checks_the_proof_in_the_record() {
    let (mut l, f, _) = leak();
    let r = push(&mut l, vgdb_retention("bb"));
    label(&mut l, &f, "L-COLLECT", &[E_GROWTH_PASS, r]).unwrap();
    let text: String = l
        .lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect();
    assert_eq!(
        Ledger::from_text(&text)
            .unwrap()
            .finding(&f)
            .unwrap()
            .archetype,
        "L-COLLECT"
    );
    let replay_err = |t: &str| match Ledger::from_text(t) {
        Err(Error::Replay(r)) => r.refusal,
        other => panic!("expected a replay error, got {other:?}"),
    };
    // Cited evidence that is not the growth it claims.
    let forged = text.replace(
        &format!("\"evidence\":[\"E{E_GROWTH_PASS}\",\"E{r}\"]"),
        &format!("\"evidence\":[\"E{E_GROWTH_OTHER}\",\"E{r}\"]"),
    );
    assert_ne!(forged, text);
    assert_eq!(replay_err(&forged).code, m6::LABEL_EVIDENCE_MISSING);
    // A kind the cited records do not carry.
    let forged = text.replace("\"retention-path\"]", "\"alloc-stack\"]");
    assert_eq!(replay_err(&forged).code, m6::LABEL_EVIDENCE_MISSING);
    // A label that is not an archetype ID.
    let forged = text.replace("\"label\":\"L-COLLECT\"", "\"label\":\"L-MADE-UP\"");
    assert_eq!(replay_err(&forged).code, m6::UNKNOWN_ARCHETYPE);
    // A wrong `previous`.
    let forged = text.replace("\"previous\":\"UNCLASSIFIED\"", "\"previous\":\"L-CYCLE\"");
    assert_eq!(replay_err(&forged).code, m6::LABEL_STATE);
    // A schema 5 line cannot carry the record.
    let n = text.lines().count();
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    lines[n - 1] = lines[n - 1].replace("\"schema_version\":6", "\"schema_version\":5");
    assert!(Ledger::from_text(&(lines.join("\n") + "\n")).is_err());
}

#[test]
fn a_worker_labels_its_own_findings_only() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    let f = confirm_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    let r = l.add_evidence(vgdb_retention("bb")).unwrap();
    let Record::Evidence(r) = r.record else {
        panic!()
    };
    let t = table();
    let line = l
        .label(
            &f,
            "L-COLLECT",
            &[eid(E_GROWTH_PASS), r.evidence_id],
            None,
            Archetypes::On(&t),
        )
        .unwrap();
    assert_eq!(record_of(&line).finding, f);
    assert_eq!(record_of(&line).satisfied.evidence[1].to_string(), "jvm/E1");
    // A main-ledger finding is not the worker's.
    let mut main = m3_ledger();
    confirmed_leak(&mut main);
    write_main(dir.path(), &main);
    let mut other = slice(dir.path(), "rust");
    let e = refusal(other.label(
        &fid(1),
        "UNCLASSIFIED",
        &[],
        Some("why"),
        Archetypes::On(&t),
    ));
    assert_eq!(
        e.code,
        dla_core::slice::codes::FOREIGN_ITEM,
        "{}",
        e.message
    );
    // The slice replays, and the merge carries the label.
    let again = slice(dir.path(), "jvm");
    assert_eq!(again.finding(&f).unwrap().archetype, "L-COLLECT");
}

#[test]
fn suggest_lists_met_archetypes_first_and_what_the_others_lack() {
    let (l, f, _) = leak();
    let rows = l.suggest_labels(&f, &table()).unwrap();
    assert!(rows.iter().all(|x| x.id.starts_with("L-")));
    // E11 (unit counts at the finding's site) meets L-COLLECT; F1 does not cite it, so the
    // row names it for --evidence.
    let collect = rows.iter().find(|x| x.id == "L-COLLECT").unwrap();
    assert_eq!(collect.status, "met");
    assert_eq!(collect.evidence, [format!("E{E_UNIT_PASS}")]);
    assert_eq!(rows[0].id, "L-COLLECT", "met rows come first");
    let handle = rows.iter().find(|x| x.id == "L-HANDLE").unwrap();
    assert_eq!(handle.status, "missing");
    assert_eq!(handle.missing, ["alloc-stack"]);
}

// ------------------------------------------------------------------------------- strong

fn confirmed() -> (Ledger, FindingId) {
    let (l, f, _) = leak();
    (l, f)
}

fn strong(l: &mut Ledger, f: &FindingId, ev: &[u32]) -> Result<(), Error> {
    let ev: Vec<_> = ev.iter().map(|n| eid(*n)).collect();
    l.transition(f, S::Strong, None, None, ev, "test")
        .map(|_| ())
}

#[test]
fn strong_takes_a_harness_trend_and_an_independent_retention_record() {
    let (mut l, f) = confirmed();
    let g = push(&mut l, trend_growth("aa"));
    let r = push(&mut l, vgdb_retention("bb"));
    strong(&mut l, &f, &[g, r]).unwrap();
    assert_eq!(l.finding(&f).unwrap().state, S::Strong);
}

#[test]
fn strong_refuses_two_readings_of_one_dump() {
    let (mut l, f) = confirmed();
    let growth = signed(
        EvidenceKind::GrowthSeries,
        Authority::Dump,
        "dla-hprof",
        "dump-parser",
        "same-dump",
        json!({"trend": "T2", "series": "S2", "gate": "pass", "n": 5,
            "passed": [{"column": "holder:a.A.cache", "finding": "F1", "shape": "steady"}]}),
    );
    let retention = signed(
        EvidenceKind::RetentionPath,
        Authority::Dump,
        "dla-hprof",
        "dump-parser",
        "same-dump",
        json!({"holders": [{"key": "static:a.A.cache"}], "alloc": "x (a.go:1)"}),
    );
    let g = push(&mut l, growth);
    let r = push(&mut l, retention);
    let e = refusal(strong(&mut l, &f, &[g, r]));
    assert_eq!(e.code, m6::STRONG_NOT_INDEPENDENT);
    assert!(
        e.message.contains("tool, tool class, source"),
        "{}",
        e.message
    );
    assert!(e.message.contains("dla-hprof"), "{}", e.message);
    assert_eq!(l.finding(&f).unwrap().state, S::Confirmed);
}

#[test]
fn strong_refuses_a_histogram() {
    let (mut l, f) = confirmed();
    let g = push(&mut l, trend_growth("aa"));
    let h = push(
        &mut l,
        signed(
            EvidenceKind::HistogramDiff,
            Authority::Dump,
            "dla-hprof-histogram",
            "histogram",
            "hh",
            json!({"finding": "F1", "types": []}),
        ),
    );
    let e = refusal(strong(&mut l, &f, &[g, h]));
    assert_eq!(e.code, m6::STRONG_SECOND_CLASS);
    assert!(e.message.contains("histogram"), "{}", e.message);
    assert!(
        e.fix.contains("dla summarize heap-graph") && e.fix.contains("dla intervene"),
        "{}",
        e.fix
    );
}

#[test]
fn strong_refuses_a_retention_record_for_another_holder() {
    let (mut l, f) = confirmed();
    let g = push(&mut l, trend_growth("aa"));
    let r = push(
        &mut l,
        signed(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-hprof",
            "dump-parser",
            "cc",
            json!({"holders": [{"key": "static:z.Z.other"}]}),
        ),
    );
    let e = refusal(strong(&mut l, &f, &[g, r]));
    assert_eq!(e.code, m6::STRONG_SECOND_CLASS);
}

#[test]
fn strong_needs_growth_with_a_signal() {
    let (mut l, f) = confirmed();
    let r = push(&mut l, vgdb_retention("bb"));
    // F1 confirmed on E4, a trend record written before schema 6: no signal.
    let e = refusal(strong(&mut l, &f, &[r]));
    assert_eq!(e.code, m6::STRONG_GROWTH_UNSIGNED);
    assert!(e.fix.contains("dla trend"), "{}", e.fix);
    // Citing a fresh signed trend in the same move works.
    let g = push(&mut l, trend_growth("aa"));
    strong(&mut l, &f, &[g, r]).unwrap();
}

#[test]
fn an_intervention_counts_when_it_removed_the_growth_under_its_guards() {
    let ok_guards = json!({"build_ok": true, "run_ok": true, "same_points": true,
        "edit_scoped": true, "exercised": null, "positive_control": null});
    let (mut l, f) = confirmed();
    let g = push(&mut l, trend_growth("aa"));
    let removed = push(
        &mut l,
        intervention("growth-removed", ok_guards.clone(), "ii"),
    );
    strong(&mut l, &f, &[g, removed]).unwrap();

    for (effect, guards, text) in [
        ("growth-remains", ok_guards.clone(), "did not remove"),
        ("invalid", ok_guards.clone(), "invalid"),
        ("inconclusive", ok_guards.clone(), "inconclusive"),
        (
            "growth-removed",
            json!({"build_ok": true, "run_ok": false}),
            "run_ok",
        ),
    ] {
        let (mut l, f) = confirmed();
        let g = push(&mut l, trend_growth("aa"));
        let i = push(&mut l, intervention(effect, guards, "ii"));
        let e = refusal(strong(&mut l, &f, &[g, i]));
        assert_eq!(e.code, m6::STRONG_SECOND_CLASS, "{effect}");
        assert!(e.message.contains(text), "{effect}: {}", e.message);
    }
}

#[test]
fn an_intervention_must_name_a_control_the_finding_cites() {
    let guards = json!({"build_ok": true, "run_ok": true});
    let (mut l, f) = confirmed();
    let g = push(&mut l, trend_growth("aa"));
    let mut other = intervention("growth-removed", guards, "ii");
    other.result["control"]["trend"] = json!("T9");
    let i = push(&mut l, other);
    let e = refusal(strong(&mut l, &f, &[g, i]));
    assert_eq!(e.code, m6::STRONG_SECOND_CLASS);
    assert!(e.message.contains("control T9"), "{}", e.message);
    // An intervention that tested another finding does not count either.
    let (mut l, f) = confirmed();
    let g = push(&mut l, trend_growth("aa"));
    let mut other = intervention("growth-removed", json!({"build_ok": true}), "ii");
    other.result["finding"] = json!("F2");
    let i = push(&mut l, other);
    let e = refusal(strong(&mut l, &f, &[g, i]));
    assert!(e.message.contains("tested F2"), "{}", e.message);
}

#[test]
fn strong_is_defined_for_leak_side_findings_only() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &f, S::Refuted, &[E_DISC_NONLEAK_F1]);
    go(&mut l, &n, S::UnderTest, &[E_HARNESS]);
    go(&mut l, &n, S::Confirmed, &[E_DISC_NONLEAK_F1]);
    let e = refusal(strong(&mut l, &n, &[]));
    assert_eq!(e.code, m6::STRONG_LEAK_ONLY);
}

#[test]
fn a_stated_holder_kind_must_agree_with_the_finding() {
    let (mut l, f) = confirmed();
    let mut a = dla_core::FindingAnnotation::new(f.clone());
    a.owner = Some("x.Y.table".into());
    l.annotate_record(a).unwrap();
    let g = push(&mut l, trend_growth("aa"));
    let retention = |kind: &str, src: &str| {
        signed(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-hprof",
            "dump-parser",
            src,
            json!({"holders": [{"key": "static:x.Y.table", "holder_kind": kind}]}),
        )
    };
    let wrong = push(&mut l, retention("registration-table", "r1"));
    let e = refusal(strong(&mut l, &f, &[g, wrong]));
    assert_eq!(e.code, m6::STRONG_HOLDER_MISMATCH);
    assert!(
        e.fix
            .contains("ledger annotate F1 --holder registration-table --evidence"),
        "{}",
        e.fix
    );
    let right = push(&mut l, retention("static/global", "r2"));
    strong(&mut l, &f, &[g, right]).unwrap();
}

// ----------------------------------------------------------------------------- explain

fn explain_input(
    l: &mut Ledger,
    f: &FindingId,
    adds: &[&str],
    removal: RemovalClaim,
    remover: Option<&str>,
    text: &str,
) -> Result<dla_core::LedgerLine, Error> {
    let adds: Vec<String> = adds.iter().map(|s| (*s).to_owned()).collect();
    l.explain(
        f,
        ExplanationTag::Inferred,
        &adds,
        removal,
        remover,
        None,
        text,
        &[],
        &[],
    )
}

const GOOD: &str = "Requests add entries to the static/global cache at a.go:1 through put at a.go:9 and no code removes them, so the map only grows.";

#[test]
fn an_explanation_needs_a_recorded_writer() {
    let (mut l, f) = confirmed();
    let e = refusal(explain_input(
        &mut l,
        &f,
        &["a.go:9"],
        RemovalClaim::Never,
        None,
        GOOD,
    ));
    assert_eq!(e.code, m6::EXPLAIN_NO_WRITER);
    assert!(e.fix.contains("ledger annotate F1 --writer"), "{}", e.fix);
}

#[test]
fn a_good_explanation_is_stored_and_the_latest_wins() {
    let (mut l, f) = confirmed();
    l.annotate(&f, vec!["a.go:9".into()], vec![], None).unwrap();
    explain_input(&mut l, &f, &["a.go:9"], RemovalClaim::Never, None, GOOD).unwrap();
    let x = l.finding(&f).unwrap().explanation.clone().unwrap();
    assert_eq!(x.tag, ExplanationTag::Inferred);
    assert_eq!(x.text, GOOD);
    let again = GOOD.replace("only grows", "only ever grows");
    explain_input(&mut l, &f, &[], RemovalClaim::Never, None, &again).unwrap();
    assert_eq!(
        l.finding(&f).unwrap().explanation.as_ref().unwrap().text,
        again
    );
    assert_eq!(
        l.explanation(&f).unwrap().adds,
        ["a.go:9"],
        "adds default to the recorded writers"
    );
    // Replays from text.
    let text: String = l
        .lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect();
    let back = Ledger::from_text(&text).unwrap();
    assert_eq!(
        back.finding(&f).unwrap().explanation.as_ref().unwrap().text,
        again
    );
}

#[test]
fn explain_refusals_name_the_missing_part() {
    let (mut l, f) = confirmed();
    l.annotate(&f, vec!["a.go:9".into()], vec!["a.go:40".into()], None)
        .unwrap();
    let go = |l: &mut Ledger, adds: &[&str], removal, remover: Option<&str>, text: &str| {
        refusal(explain_input(l, &f, adds, removal, remover, text))
    };
    // Site not named.
    let e = go(
        &mut l,
        &["a.go:9"],
        RemovalClaim::Conditional,
        Some("a.go:40"),
        "Requests add entries to the static/global cache through put at a.go:9, and a.go:40 removes them only when the flag is set.",
    );
    assert_eq!(e.code, m6::EXPLAIN_MISSING_SITE);
    // Holder not named.
    let e = go(
        &mut l,
        &["a.go:9"],
        RemovalClaim::Conditional,
        Some("a.go:40"),
        "Requests add entries at a.go:1 through put at a.go:9, and a.go:40 removes them only when the flag is set.",
    );
    assert_eq!(e.code, m6::EXPLAIN_MISSING_HOLDER);
    // Adds not recorded.
    let e = go(
        &mut l,
        &["a.go:77"],
        RemovalClaim::Never,
        None,
        &GOOD.replace("a.go:9", "a.go:77"),
    );
    assert_eq!(e.code, m6::EXPLAIN_ADDS);
    // The text names none of the adds.
    let e = go(
        &mut l,
        &["a.go:9"],
        RemovalClaim::Conditional,
        Some("a.go:40"),
        "Requests add entries to the static/global cache at a.go:1 and a.go:40 removes them only when the flag is set, otherwise never.",
    );
    assert_eq!(e.code, m6::EXPLAIN_ADDS);
    // "never" with a recorded remover.
    let e = go(&mut l, &["a.go:9"], RemovalClaim::Never, None, GOOD);
    assert_eq!(e.code, m6::EXPLAIN_REMOVER_RECORDED);
    assert!(e.message.contains("a.go:40"), "{}", e.message);
    // Conditional without a remover, and with one that is not recorded.
    let e = go(&mut l, &["a.go:9"], RemovalClaim::Conditional, None, GOOD);
    assert_eq!(e.code, m6::EXPLAIN_REMOVER);
    let e = go(
        &mut l,
        &["a.go:9"],
        RemovalClaim::Conditional,
        Some("a.go:41"),
        "Requests add entries to the static/global cache at a.go:1 through put at a.go:9, and a.go:41 removes them only when set.",
    );
    assert_eq!(e.code, m6::EXPLAIN_REMOVER);
    // The removal is not stated.
    let e = go(
        &mut l,
        &["a.go:9"],
        RemovalClaim::Conditional,
        Some("a.go:40"),
        "Requests add entries to the static/global cache at a.go:1 through put at a.go:9, and a.go:40 clears the cache sometimes.",
    );
    assert_eq!(e.code, m6::EXPLAIN_REMOVAL_TEXT);
    // Text bounds.
    let e = go(&mut l, &["a.go:9"], RemovalClaim::Never, None, "too short");
    assert_eq!(e.code, m6::EXPLAIN_TEXT);
    let e = go(
        &mut l,
        &["a.go:9"],
        RemovalClaim::Never,
        None,
        &format!("{GOOD}\n{GOOD}"),
    );
    assert_eq!(e.code, m6::EXPLAIN_TEXT);
    // And the good conditional one goes through.
    explain_input(&mut l, &f, &["a.go:9"], RemovalClaim::Conditional, Some("a.go:40"),
        "Requests add entries to the static/global cache at a.go:1 through put at a.go:9, and a.go:40 removes them only when the flag is set.")
        .unwrap();
}

#[test]
fn observed_needs_evidence_that_shows_the_pairing() {
    let (mut l, f) = confirmed();
    l.annotate(&f, vec!["a.go:9".into()], vec![], None).unwrap();
    let adds = ["a.go:9".to_owned()];
    let ex = |l: &mut Ledger, ev: Option<u32>| {
        let ev = ev.map(eid);
        l.explain(
            &f,
            ExplanationTag::Observed,
            &adds,
            RemovalClaim::Never,
            None,
            ev.as_ref(),
            GOOD,
            &[],
            &[],
        )
    };
    let e = refusal(ex(&mut l, None));
    assert_eq!(e.code, m6::EXPLAIN_TAG);
    let e = refusal(ex(&mut l, Some(E_STATIC)));
    assert_eq!(e.code, m6::EXPLAIN_TAG);
    let r = push(&mut l, vgdb_retention("bb"));
    ex(&mut l, Some(r)).unwrap();
    assert_eq!(l.explanation(&f).unwrap().tag, ExplanationTag::Observed);
}

#[test]
fn only_a_confirmed_leak_side_finding_is_explained() {
    let mut l = m3_ledger();
    let f = add_leak(&mut l);
    let e = refusal(explain_input(
        &mut l,
        &f,
        &["a.go:9"],
        RemovalClaim::Never,
        None,
        GOOD,
    ));
    assert_eq!(e.code, m6::EXPLAIN_STATE);
}

#[test]
fn a_worker_explains_its_own_finding() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    let f = confirm_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    l.annotate(&f, vec!["a.go:9".into()], vec![], None).unwrap();
    explain_input(&mut l, &f, &[], RemovalClaim::Never, None, GOOD).unwrap();
    assert!(
        slice(dir.path(), "jvm")
            .finding(&f)
            .unwrap()
            .explanation
            .is_some()
    );
}

#[test]
fn ledger_add_refuses_an_explanation_from_schema_six() {
    let mut l = m3_ledger();
    let mut nf = dla_core::NewFinding::new(origin("census"), vec![eid(E_STATIC)]);
    nf.explanation = Some(dla_core::finding::Explanation {
        text: "x".into(),
        tag: ExplanationTag::Inferred,
    });
    let e = refusal(l.add_finding(nf));
    assert_eq!(e.code, m6::EXPLANATION_LOCKED);
    assert!(e.fix.contains("dla ledger explain"), "{}", e.fix);
}

#[test]
fn a_schema_five_line_with_an_explanation_still_replays() {
    let line = "{\"schema_version\":5,\"seq\":2,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"state\":\"candidate\",\"origin\":\"census\",\"supports\":[\"E1\"],\"explanation\":{\"text\":\"old text\",\"tag\":\"inferred\"}}}\n";
    let text = format!("{}{line}", dla_core_line_one());
    let l = Ledger::from_text(&text).unwrap();
    assert_eq!(
        l.finding(&fid(1))
            .unwrap()
            .explanation
            .as_ref()
            .unwrap()
            .text,
        "old text"
    );
}

fn dla_core_line_one() -> String {
    format!(
        "{{\"schema_version\":5,\"seq\":1,\"ts\":\"2026-01-01T00:00:00Z\",{}}}\n",
        EVIDENCE_BODY
    )
}

// ------------------------------------------------------------------------ verification

fn verify(
    l: &mut Ledger,
    f: &FindingId,
    label: &str,
    a: Archetypes<'_>,
) -> Result<(dla_core::LedgerLine, Option<dla_core::LedgerLine>), Error> {
    l.verify(
        f,
        Some("a.go:1".into()),
        Holder::StaticGlobal,
        "the cache keeps every entry",
        label,
        &"a".repeat(64),
        a,
    )
}

#[test]
fn a_verifier_label_is_validated_and_compared() {
    let (mut l, f, _) = leak();
    let r = push(&mut l, vgdb_retention("bb"));
    label(&mut l, &f, "L-COLLECT", &[E_GROWTH_PASS, r]).unwrap();
    let t = table();
    // Agreement.
    let (line, item) = verify(&mut l, &f, "L-COLLECT", Archetypes::On(&t)).unwrap();
    let Record::Verification(v) = &line.record else {
        panic!()
    };
    assert_eq!(v.agrees_label, Some(true));
    assert!(item.is_none());
    // Disagreement opens an item with both labels and the mechanism.
    let (line, item) = verify(&mut l, &f, "UNCLASSIFIED", Archetypes::On(&t)).unwrap();
    let Record::Verification(v) = &line.record else {
        panic!()
    };
    assert_eq!(v.agrees_label, Some(false));
    let Record::OpenItem(o) = item.unwrap().record else {
        panic!()
    };
    assert!(
        o.text
            .contains("label: finding L-COLLECT vs verifier UNCLASSIFIED"),
        "{}",
        o.text
    );
    assert!(o.text.contains("the cache keeps every entry"), "{}", o.text);
    // An unknown id.
    let e = refusal(verify(&mut l, &f, "L-NOPE", Archetypes::On(&t)));
    assert_eq!(e.code, m6::UNKNOWN_ARCHETYPE);
    // Off: only UNCLASSIFIED, compared with nothing.
    let e = refusal(verify(&mut l, &f, "L-COLLECT", Archetypes::Off));
    assert_eq!(e.code, m6::ARCHETYPES_OFF);
    let (line, item) = verify(&mut l, &f, "UNCLASSIFIED", Archetypes::Off).unwrap();
    let Record::Verification(v) = &line.record else {
        panic!()
    };
    assert_eq!(v.agrees_label, None);
    assert!(item.is_none());
    // The file replays, and a forged agreement does not.
    let text: String = l
        .lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect();
    Ledger::from_text(&text).unwrap();
    let forged = text.replacen("\"agrees_label\":true", "\"agrees_label\":false", 1);
    assert!(Ledger::from_text(&forged).is_err());
}

#[test]
fn integrity_resealed_lines_replay_at_schema_six() {
    let (mut l, _, _) = leak();
    l.integrity_resealed(vec!["ledger.jsonl".into()], "edited by hand")
        .unwrap();
    assert_eq!(l.reseals().len(), 1);
    let e = refusal(l.integrity_resealed(vec![], "x"));
    assert_eq!(e.code, codes::EMPTY_FIELD);
    let text: String = l
        .lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect();
    assert_eq!(Ledger::from_text(&text).unwrap().reseals().len(), 1);
}

// ------------------------------------------------------------------------------- merge

#[test]
fn a_label_on_a_duplicate_is_not_copied_and_the_merge_says_so() {
    let dir = new_run(&["jvm", "rust"]);
    let t = table();
    let mut a = slice(dir.path(), "jvm");
    let fa = confirm_in_slice(&mut a, "a.go:1", Holder::StaticGlobal);
    a.finish_slice().unwrap();
    // The rust worker finds the same site and holder and labels it. The canonical finding
    // of the pair is the one of the worker that sorts first (jvm), which has no label.
    let mut b = slice(dir.path(), "rust");
    let fb = confirm_in_slice(&mut b, "a.go:1", Holder::StaticGlobal);
    let r = b.add_evidence(vgdb_retention("bb")).unwrap();
    let Record::Evidence(r) = r.record else {
        panic!()
    };
    b.label(
        &fb,
        "L-COLLECT",
        &[eid(E_GROWTH_PASS), r.evidence_id],
        None,
        Archetypes::On(&t),
    )
    .unwrap();
    b.finish_slice().unwrap();
    drop((a, b));
    let report = dla_core::merge::merge(dir.path()).unwrap();
    assert_eq!(report.duplicates.len(), 1);
    assert_eq!(report.duplicates[0].canonical, fa.to_string());
    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(main.finding(&fa).unwrap().archetype, "UNCLASSIFIED");
    assert_eq!(main.finding(&fb).unwrap().archetype, "L-COLLECT");
    let item = main
        .open_items()
        .find(|o| o.item.text.contains("not carried"))
        .expect("an open item");
    assert!(
        item.item
            .text
            .contains(&format!("dla ledger label {fa} --as L-COLLECT"))
    );
    // Merging again does not repeat the item.
    dla_core::merge::merge(dir.path()).unwrap();
    let again = Ledger::load(dir.path()).unwrap();
    assert_eq!(
        again
            .open_items()
            .filter(|o| o.item.text.contains("not carried"))
            .count(),
        1
    );
}
