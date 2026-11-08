//! The artifact-path rules of the ledger (schema 3 lines on a run whose entry is
//! `artifact`): a dump-confirmed finding needs an owner instead of a site, a dump's
//! retention path can stand in for static evidence, and nothing on such a run names a site.

mod common;

use std::collections::BTreeMap;

use common::*;
use dla_core::{
    Authority, EvidenceId, EvidenceKind, EvidenceRecord, Finding, FindingId, FindingState as S,
    Ledger, NewFinding, UnresolvedReason,
    error::{Error, codes},
    finding::Holder,
    machine::{
        EvidenceLookup, FindingLookup, M3Rules, M4Rules, TransitionContext, TransitionRules,
        check_transition,
    },
    slice::codes as m4,
    state::EntryPath,
};
use serde_json::json;

fn refusal<T: std::fmt::Debug>(r: Result<T, Error>) -> dla_core::Refusal {
    r.expect_err("expected a refusal").to_refusal()
}

// Evidence of the artifact ledger below.
const E_RETENTION_DUMP: u32 = 1;
const E_GROWTH_DUMP: u32 = 2;
const E_GROWTH_TEST: u32 = 3;
const E_DISC_LEAK_DUMP: u32 = 4;
const E_RETENTION_LIVE: u32 = 5;
const E_STATIC: u32 = 6;
const E_GROWTH_DUMP_FAIL: u32 = 7;
const E_HARNESS: u32 = 8;
const E_GROWTH_LIVE: u32 = 9;

fn passed() -> serde_json::Value {
    json!([{"column": "holder:static:Cache.entries", "region": "heap", "shape": "steady"}])
}

fn growth(authority: Authority, gate: &str) -> dla_core::NewEvidence {
    evidence_with(
        EvidenceKind::GrowthSeries,
        authority,
        json!({"trend": "T1", "series": "S1", "n": 3, "passed": passed(), "failed": 0, "gate": gate}),
    )
}

/// A memory ledger of an `entry` run holding E1 to E9, see the constants.
fn run(entry: EntryPath) -> Ledger {
    let mut l = Ledger::from_text_for("", entry).unwrap();
    let adds = [
        evidence_with(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            json!({"holders": 3}),
        ),
        growth(Authority::Dump, "pass"),
        growth(Authority::Test, "pass"),
        evidence_with(
            EvidenceKind::Discriminator,
            Authority::Dump,
            json!({"test": "dump-plateau", "favors": "leak", "supports": ["leak"], "finding": null}),
        ),
        evidence_with(
            EvidenceKind::RetentionPath,
            Authority::Live,
            json!({"holders": 3}),
        ),
        evidence(EvidenceKind::StaticHit, Authority::Static),
        growth(Authority::Dump, "fail"),
        evidence_with(
            EvidenceKind::Workload,
            Authority::Test,
            json!({"harness": "H1", "mode": "command", "runtime": "jvm"}),
        ),
        growth(Authority::Live, "pass"),
    ];
    for e in adds {
        l.add_evidence(e).unwrap();
    }
    l
}

/// A leak-side finding with an owner (and a holder unless `holder` is `None`), plus a
/// refuted N-WARM sibling when `refuted_sibling`.
fn finding(
    l: &mut Ledger,
    owner: Option<&str>,
    holder: Option<Holder>,
    refuted_sibling: bool,
) -> FindingId {
    let mut nf = NewFinding::new(origin("growth-series"), vec![eid(E_GROWTH_DUMP)]);
    nf.owner = owner.map(str::to_owned);
    nf.holder = holder;
    let line = l.add_finding(nf).unwrap();
    let dla_core::Record::FindingAdded { finding } = line.record else {
        unreachable!()
    };
    let f = finding.id;
    let n = add_sibling(l, &f, "N-WARM");
    if refuted_sibling {
        go(l, &n, S::Refuted, &[E_DISC_LEAK_DUMP]);
    }
    f
}

fn under_test(l: &mut Ledger, f: &FindingId, ev: u32) -> Result<(), Error> {
    l.transition(f, S::UnderTest, None, None, vec![eid(ev)], "test")
        .map(|_| ())
}

fn confirm(l: &mut Ledger, f: &FindingId, ev: &[u32]) -> Result<(), Error> {
    l.transition(
        f,
        S::Confirmed,
        None,
        None,
        ev.iter().map(|n| eid(*n)).collect(),
        "test",
    )
    .map(|_| ())
}

// ---------------------------------------------------------------- confirmed

struct Case {
    name: &'static str,
    entry: EntryPath,
    owner: Option<&'static str>,
    holder: Option<Holder>,
    refuted_sibling: bool,
    /// Growth evidence the finding cites (E_GROWTH_DUMP is cited when it is created).
    extra_cited: &'static [u32],
    /// `None` when the confirmation is accepted, else the refusal code.
    want: Option<&'static str>,
}

const OWNER: Option<&str> = Some("com.acme.Cache.entries");
const HOLDER: Option<Holder> = Some(Holder::StaticGlobal);

#[test]
fn confirmed_on_an_artifact_run_takes_an_owner_in_place_of_a_site() {
    use EntryPath::*;
    let cases = [
        Case {
            name: "owner, holder, dump growth, refuted sibling",
            entry: Artifact,
            owner: OWNER,
            holder: HOLDER,
            refuted_sibling: true,
            extra_cited: &[],
            want: None,
        },
        Case {
            name: "no owner and no site",
            entry: Artifact,
            owner: None,
            holder: HOLDER,
            refuted_sibling: true,
            extra_cited: &[],
            want: Some(codes::SITE_REQUIRED),
        },
        Case {
            name: "a blank owner is no owner",
            entry: Artifact,
            owner: Some("  "),
            holder: HOLDER,
            refuted_sibling: true,
            extra_cited: &[],
            want: Some(codes::SITE_REQUIRED),
        },
        Case {
            name: "a cited passing growth record read from a test",
            entry: Artifact,
            owner: OWNER,
            holder: HOLDER,
            refuted_sibling: true,
            extra_cited: &[E_GROWTH_TEST],
            want: Some(codes::SITE_REQUIRED),
        },
        Case {
            name: "a cited passing growth record read from a live process",
            entry: Artifact,
            owner: OWNER,
            holder: HOLDER,
            refuted_sibling: true,
            extra_cited: &[E_GROWTH_LIVE],
            want: Some(codes::SITE_REQUIRED),
        },
        Case {
            name: "a cited failing dump record does not count against it",
            entry: Artifact,
            owner: OWNER,
            holder: HOLDER,
            refuted_sibling: true,
            extra_cited: &[E_GROWTH_DUMP_FAIL],
            want: None,
        },
        Case {
            name: "no holder",
            entry: Artifact,
            owner: OWNER,
            holder: None,
            refuted_sibling: true,
            extra_cited: &[],
            want: Some(codes::HOLDER_REQUIRED),
        },
        Case {
            name: "no refuted non-leak sibling",
            entry: Artifact,
            owner: OWNER,
            holder: HOLDER,
            refuted_sibling: false,
            extra_cited: &[],
            want: Some(codes::NONLEAK_REFUTATION_REQUIRED),
        },
        Case {
            name: "a repository run still needs a site",
            entry: Repository,
            owner: OWNER,
            holder: HOLDER,
            refuted_sibling: true,
            extra_cited: &[],
            want: Some(codes::SITE_REQUIRED),
        },
        Case {
            name: "a process run still needs a site",
            entry: Process,
            owner: OWNER,
            holder: HOLDER,
            refuted_sibling: true,
            extra_cited: &[],
            want: Some(codes::SITE_REQUIRED),
        },
    ];
    for c in cases {
        let mut l = run(c.entry);
        let f = finding(&mut l, c.owner, c.holder, c.refuted_sibling);
        under_test(&mut l, &f, E_GROWTH_DUMP)
            .or_else(|_| under_test(&mut l, &f, E_HARNESS))
            .unwrap_or_else(|e| panic!("{}: under_test: {e}", c.name));
        let mut cited = vec![E_GROWTH_DUMP];
        cited.extend(c.extra_cited);
        let got = confirm(&mut l, &f, &cited);
        match (got, c.want) {
            (Ok(()), None) => {
                let f = l.finding(&f).unwrap();
                assert_eq!(f.state, S::Confirmed, "{}", c.name);
                assert!(f.site.is_none(), "{}: no source-mapping claim", c.name);
            }
            (Err(e), Some(code)) => {
                let r = e.to_refusal();
                assert_eq!(r.code, code, "{}: {r}", c.name);
                if code == codes::SITE_REQUIRED && c.entry == EntryPath::Artifact {
                    assert!(r.missing.contains("owner"), "{}: {r}", c.name);
                    assert!(r.fix.contains("--owner"), "{}: {r}", c.name);
                }
                assert_eq!(l.finding(&f).unwrap().state, S::UnderTest, "{}", c.name);
            }
            (got, want) => panic!("{}: got {got:?}, wanted {want:?}", c.name),
        }
    }
}

/// A lookup over fixed records, for judging a move without a ledger.
struct Fixed {
    evidence: BTreeMap<EvidenceId, EvidenceRecord>,
    findings: BTreeMap<FindingId, Finding>,
}

impl EvidenceLookup for Fixed {
    fn evidence(&self, id: &EvidenceId) -> Option<&EvidenceRecord> {
        self.evidence.get(id)
    }
    fn evidence_ids(&self) -> Vec<EvidenceId> {
        self.evidence.keys().cloned().collect()
    }
}

impl FindingLookup for Fixed {
    fn finding(&self, id: &FindingId) -> Option<&Finding> {
        self.findings.get(id)
    }
    fn family(&self, id: &FindingId) -> Vec<&Finding> {
        let Some(f) = self.findings.get(id) else {
            return Vec::new();
        };
        let root = f.root_id();
        self.findings
            .values()
            .filter(|g| g.id == root || g.parent.as_ref() == Some(&root))
            .collect()
    }
}

#[test]
fn a_confirmed_artifact_finding_must_have_no_site() {
    // Build the family and the evidence from a real ledger, then judge the move by hand
    // with a site on the finding (the ledger itself refuses to hold one).
    let mut l = run(EntryPath::Artifact);
    let f = finding(&mut l, OWNER, HOLDER, true);
    under_test(&mut l, &f, E_GROWTH_DUMP).unwrap();
    let mut with_site = l.finding(&f).unwrap().clone();
    with_site.site = Some("src/Cache.java:12".into());
    let lookup = Fixed {
        evidence: l
            .evidence()
            .map(|e| (e.evidence_id.clone(), e.clone()))
            .collect(),
        findings: l
            .findings()
            .cloned()
            .map(|g| if g.id == f { with_site.clone() } else { g })
            .map(|g| (g.id.clone(), g))
            .collect(),
    };
    let ctx = |entry| TransitionContext {
        finding: &with_site,
        to: S::Confirmed,
        reason: None,
        remedy: None,
        evidence: &[],
        lookup: &lookup,
        findings: &lookup,
        entry,
    };
    let evidence = [eid(E_GROWTH_DUMP)];
    let mk = |entry| TransitionContext {
        evidence: &evidence,
        ..ctx(entry)
    };
    let r = check_transition(&M4Rules, &mk(EntryPath::Artifact)).unwrap_err();
    assert_eq!(r.code, m4::SITE_FORBIDDEN, "{r}");
    assert!(r.message.contains("src/Cache.java:12"));
    // The same finding on a repository run is fine, and so is it under the M3 rules.
    check_transition(&M4Rules, &mk(EntryPath::Repository)).unwrap();
    check_transition(&M3Rules, &mk(EntryPath::Artifact)).unwrap();
}

#[test]
fn version_two_lines_do_not_get_the_artifact_rules() {
    // A v2 transition on an artifact run is judged by the M3 rules: it needs a site.
    let l = run(EntryPath::Artifact);
    let mut text = String::new();
    for line in l.lines() {
        text.push_str(&serde_json::to_string(line).unwrap().replacen(
            "\"schema_version\":3",
            "\"schema_version\":2",
            1,
        ));
        text.push('\n');
    }
    let seq = l.lines().len();
    let mut add = |n: usize, body: &str| {
        text.push_str(&format!(
            "{{\"schema_version\":2,\"seq\":{},\"ts\":\"2026-01-01T00:00:00Z\",{body}}}\n",
            seq + n
        ));
    };
    add(
        1,
        r#""type":"finding_added","finding":{"id":"F1","state":"candidate","origin":"growth-series","owner":"a.B.c","holder":"static/global","supports":["E2"]}"#,
    );
    add(
        2,
        r#""type":"finding_added","finding":{"id":"F1.n1","state":"candidate","origin":"hypothesis","hypothesis":"N-WARM","refute_if":"x","parent":"F1","supports":["E2"]}"#,
    );
    add(
        3,
        r#""type":"transition","finding":"F1.n1","from":"candidate","to":"refuted","evidence":["E4"],"by":"t""#,
    );
    add(
        4,
        r#""type":"transition","finding":"F1","from":"candidate","to":"under_test","evidence":["E8"],"by":"t""#,
    );
    add(
        5,
        r#""type":"transition","finding":"F1","from":"under_test","to":"confirmed","evidence":["E2"],"by":"t""#,
    );
    let e = Ledger::from_text_for(&text, EntryPath::Artifact).unwrap_err();
    assert_eq!(e.to_refusal().code, codes::SITE_REQUIRED);
}

// ---------------------------------------------------------------- under_test

#[test]
fn an_artifact_finding_goes_under_test_on_its_dump_series() {
    // The finding is supported by a static hit only, so its own supports say nothing.
    fn bare(l: &mut Ledger) -> FindingId {
        let mut nf = NewFinding::new(origin("census"), vec![eid(E_STATIC)]);
        nf.owner = OWNER.map(str::to_owned);
        let line = l.add_finding(nf).unwrap();
        let dla_core::Record::FindingAdded { finding } = line.record else {
            unreachable!()
        };
        add_sibling(l, &finding.id, "N-WARM");
        finding.id
    }
    let mut l = run(EntryPath::Artifact);
    let f = bare(&mut l);
    // A test or live growth record is not enough without a harness; a dump series is.
    let r = refusal(under_test(&mut l, &f, E_GROWTH_TEST));
    assert_eq!(r.code, codes::HARNESS_REQUIRED);
    assert!(r.fix.contains("dla summarize heap-graph"), "{r}");
    under_test(&mut l, &f, E_GROWTH_DUMP).unwrap();

    // On a repository run the dump series does not replace the harness.
    let mut l = run(EntryPath::Repository);
    let f = bare(&mut l);
    let r = refusal(under_test(&mut l, &f, E_GROWTH_DUMP));
    assert_eq!(r.code, codes::HARNESS_REQUIRED);
    assert!(r.fix.contains("dla harness gen"), "{r}");
}

// ---------------------------------------------------------------- suspected_static_only

fn suspect(
    l: &mut Ledger,
    f: &FindingId,
    reason: Option<UnresolvedReason>,
    ev: u32,
) -> Result<(), Error> {
    l.transition(
        f,
        S::SuspectedStaticOnly,
        reason,
        Some("capture at least three heap dumps of the same process".into()),
        vec![eid(ev)],
        "test",
    )
    .map(|_| ())
}

#[test]
fn suspected_static_only_accepts_a_dump_retention_path_on_an_artifact_run() {
    // (entry, evidence, reason, wanted refusal)
    let cases: [(EntryPath, u32, Option<UnresolvedReason>, Option<&str>); 8] = [
        (
            EntryPath::Artifact,
            E_RETENTION_DUMP,
            Some(UnresolvedReason::NotExercised),
            None,
        ),
        (
            EntryPath::Artifact,
            E_STATIC,
            Some(UnresolvedReason::NotExercised),
            None,
        ),
        (
            EntryPath::Artifact,
            E_RETENTION_DUMP,
            Some(UnresolvedReason::NoHarness),
            Some(codes::INVALID_VALUE),
        ),
        (
            EntryPath::Artifact,
            E_RETENTION_LIVE,
            Some(UnresolvedReason::NotExercised),
            Some(codes::STATIC_EVIDENCE_REQUIRED),
        ),
        (
            EntryPath::Artifact,
            E_GROWTH_DUMP,
            Some(UnresolvedReason::NotExercised),
            Some(codes::STATIC_EVIDENCE_REQUIRED),
        ),
        (
            EntryPath::Repository,
            E_RETENTION_DUMP,
            Some(UnresolvedReason::NotExercised),
            Some(codes::STATIC_EVIDENCE_REQUIRED),
        ),
        (
            EntryPath::Process,
            E_RETENTION_DUMP,
            Some(UnresolvedReason::NotExercised),
            Some(codes::STATIC_EVIDENCE_REQUIRED),
        ),
        (
            EntryPath::Artifact,
            E_RETENTION_DUMP,
            None,
            Some(codes::REASON_REQUIRED),
        ),
    ];
    for (entry, ev, reason, want) in cases {
        let mut l = run(entry);
        let f = finding(&mut l, OWNER, HOLDER, false);
        let got = suspect(&mut l, &f, reason, ev);
        match (got, want) {
            (Ok(()), None) => {
                let f = l.finding(&f).unwrap();
                assert_eq!(f.state, S::SuspectedStaticOnly, "{entry} E{ev}");
                assert_eq!(f.reason, Some(UnresolvedReason::NotExercised));
                assert!(f.remedy.as_deref().unwrap().contains("heap dumps"));
            }
            (Err(e), Some(code)) => {
                assert_eq!(e.to_refusal().code, code, "{entry} E{ev} {reason:?}");
            }
            (got, want) => panic!("{entry} E{ev} {reason:?}: got {got:?}, wanted {want:?}"),
        }
    }
}

// ---------------------------------------------------------------- no site on an artifact run

#[test]
fn an_artifact_run_refuses_a_site_on_add_and_on_annotate() {
    let mut l = run(EntryPath::Artifact);
    let mut nf = NewFinding::new(origin("growth-series"), vec![eid(E_GROWTH_DUMP)]);
    nf.site = Some("src/Cache.java:12".into());
    let r = refusal(l.add_finding(nf));
    assert_eq!(r.code, m4::SITE_FORBIDDEN, "{r}");
    assert!(r.fix.contains("--owner"));

    let f = finding(&mut l, None, HOLDER, false);
    let before = l.lines().len();
    let r = refusal(l.annotate_record(dla_core::FindingAnnotation {
        site: Some("src/Cache.java:12".into()),
        ..dla_core::FindingAnnotation::new(f.clone())
    }));
    assert_eq!(r.code, m4::SITE_FORBIDDEN, "{r}");
    assert!(r.message.contains("must have no site"), "{r}");
    assert_eq!(l.lines().len(), before, "nothing was written");
    // The owner is what an artifact finding is identified by.
    l.annotate_record(dla_core::FindingAnnotation {
        owner: OWNER.map(str::to_owned),
        ..dla_core::FindingAnnotation::new(f.clone())
    })
    .unwrap();
    assert_eq!(l.finding(&f).unwrap().owner.as_deref(), OWNER);

    // Other entry paths take a site as before.
    let mut l = run(EntryPath::Repository);
    let f = finding(&mut l, None, HOLDER, false);
    l.annotate_record(dla_core::FindingAnnotation {
        site: Some("src/Cache.java:12".into()),
        ..dla_core::FindingAnnotation::new(f)
    })
    .unwrap();
}

#[test]
fn the_entry_path_is_read_from_state_json_when_a_ledger_is_loaded() {
    let dir = new_run(&[]);
    dla_core::RunState::update(dir.path(), |s| {
        s.entry = EntryPath::Artifact;
        Ok(())
    })
    .unwrap();
    let mut l = Ledger::load(dir.path()).unwrap();
    assert_eq!(l.entry(), EntryPath::Artifact);
    let mut nf = NewFinding::new(origin("census"), vec![eid(1)]);
    nf.site = Some("a.go:1".into());
    assert_eq!(refusal(l.add_finding(nf)).code, m4::SITE_FORBIDDEN);
    // A worker slice sees the same entry path.
    let mut w = Ledger::load_for(dir.path(), Some("jvm")).unwrap();
    assert_eq!(w.entry(), EntryPath::Artifact);
    let mut nf = NewFinding::new(origin("census"), vec![eid(1)]);
    nf.site = Some("a.go:1".into());
    assert_eq!(refusal(w.add_finding(nf)).code, m4::SITE_FORBIDDEN);
}

#[allow(dead_code)]
fn _assert_rules_are_object_safe(_: &dyn TransitionRules) {}
