mod common;

use common::*;
use dla_core::{
    AnswerStatus, ChecklistAnswer, Error, FindingState, Ledger, NewFinding, Resolution,
    UnresolvedReason, error::codes, ids::OpenItemId,
};
use serde_json::json;

fn replay_err(text: &str) -> dla_core::ReplayError {
    match Ledger::from_text(text) {
        Err(Error::Replay(e)) => *e,
        other => panic!("expected replay error, got {other:?}"),
    }
}

fn ev_line(seq: u64) -> String {
    line(seq, &EVIDENCE_BODY.replace("E1", &format!("E{seq}")))
}

fn finding_body(id: &str, extra: &str) -> String {
    format!(
        r#""type":"finding_added","finding":{{"id":"{id}","state":"candidate","origin":"census","supports":["E1"]{extra}}}"#
    )
}

#[test]
fn id_allocation_is_max_plus_one() {
    let mut l = ledger();
    assert_eq!(l.next_evidence_id().to_string(), "E4");
    add_f1(&mut l);
    assert_eq!(l.next_finding_id().to_string(), "F2");
    let mut s = NewFinding::new(origin("hypothesis"), vec![eid(1)]);
    s.sibling_of = Some(fid(1));
    s.hypothesis = Some("N-WARM".into());
    s.refute_if = Some("flat after warmup".into());
    let line = l.add_finding(s.clone()).unwrap();
    assert_eq!(line.seq, 5);
    assert!(l.finding(&"F1.n1".parse().unwrap()).is_some());
    assert_eq!(l.next_sibling_id(&fid(1)).to_string(), "F1.n2");
    l.add_finding(s).unwrap();
    assert_eq!(
        l.finding(&fid(1))
            .unwrap()
            .siblings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["F1.n1", "F1.n2"]
    );
    assert_eq!(l.next_finding_id().to_string(), "F2");
}

#[test]
fn add_refuses_missing_and_unknown_evidence() {
    let mut l = ledger();
    let r = l
        .add_finding(NewFinding::new(origin("census"), vec![]))
        .unwrap_err();
    let Error::Refused(r) = r else { panic!() };
    assert_eq!(r.code, codes::MISSING_EVIDENCE);
    assert_eq!(r.fix, "dla ledger add --origin census --evidence E3");
    let Error::Refused(r) = l
        .add_finding(NewFinding::new(origin("census"), vec![eid(7)]))
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::UNKNOWN_EVIDENCE);
    assert_eq!(l.findings().count(), 0);
}

#[test]
fn add_refuses_archetype_and_bad_siblings() {
    let mut l = ledger();
    add_f1(&mut l);
    let mut n = NewFinding::new(origin("census"), vec![eid(1)]);
    n.archetype = Some("LISTENER-LEAK".into());
    let Error::Refused(r) = l.add_finding(n).unwrap_err() else {
        panic!()
    };
    assert_eq!(r.code, codes::ARCHETYPE_LOCKED);

    let mut n = NewFinding::new(origin("hypothesis"), vec![eid(1)]);
    n.archetype = Some("UNCLASSIFIED".into());
    l.add_finding(n).unwrap();

    let mut n = NewFinding::new(origin("hypothesis"), vec![eid(1)]);
    n.sibling_of = Some(fid(1));
    let Error::Refused(r) = l.add_finding(n.clone()).unwrap_err() else {
        panic!()
    };
    assert_eq!(r.code, codes::SIBLING_FIELDS);
    assert!(r.fix.contains("--sibling-of F1 --hypothesis"));
    n.sibling_of = Some(fid(9));
    n.hypothesis = Some("h".into());
    n.refute_if = Some("r".into());
    let Error::Refused(r) = l.add_finding(n).unwrap_err() else {
        panic!()
    };
    assert_eq!(r.code, codes::UNKNOWN_PARENT);
}

#[test]
fn add_refuses_invalid_origin() {
    for bad in [
        "",
        "detector:jvm",
        "detector:/rule",
        "known-leak:",
        "Census",
        "other",
    ] {
        assert!(dla_core::Origin::parse(bad).is_err(), "{bad:?}");
    }
    for ok in [
        "census",
        "detector:jvm/static-collection",
        "known-leak:KL-12",
        "residual",
        "verifier",
        "hypothesis",
        "growth-series",
        "archetype-row",
    ] {
        assert!(dla_core::Origin::parse(ok).is_ok(), "{ok:?}");
    }
}

#[test]
fn evidence_result_is_bounded() {
    let mut l = Ledger::from_text("").unwrap();
    let mut e = evidence(dla_core::EvidenceKind::Probe, dla_core::Authority::Live);
    e.result = json!({"blob": "x".repeat(dla_core::MAX_RESULT_BYTES)});
    let Error::Refused(r) = l.add_evidence(e).unwrap_err() else {
        panic!()
    };
    assert_eq!(r.code, codes::RESULT_TOO_LARGE);
    // Exactly at the bound is fine: {"b":"..."} has 8 bytes of overhead.
    let mut e = evidence(dla_core::EvidenceKind::Probe, dla_core::Authority::Live);
    e.result = json!({"b": "x".repeat(dla_core::MAX_RESULT_BYTES - 8)});
    assert_eq!(
        serde_json::to_vec(&e.result).unwrap().len(),
        dla_core::MAX_RESULT_BYTES
    );
    l.add_evidence(e).unwrap();
}

#[test]
fn replay_refuses_oversized_result() {
    let big = "x".repeat(3000);
    let text = line(
        1,
        &EVIDENCE_BODY.replace(r#""result":{}"#, &format!(r#""result":{{"b":"{big}"}}"#)),
    );
    let e = replay_err(&text);
    assert_eq!(
        (e.line, e.refusal.code.as_str()),
        (1, codes::RESULT_TOO_LARGE)
    );
}

#[test]
fn replay_seq_gaps_and_reordering() {
    let gap = ev_line(1) + &ev_line(3);
    let e = replay_err(&gap);
    assert_eq!((e.line, e.refusal.code.as_str()), (2, codes::BAD_SEQ));
    assert!(e.refusal.message.contains("3") && e.refusal.fix.contains("line 2"));
    let reordered = ev_line(2) + &ev_line(1);
    assert_eq!(replay_err(&reordered).line, 1);
    let dup = ev_line(1) + &line(2, EVIDENCE_BODY);
    assert_eq!(replay_err(&dup).refusal.code, codes::DUPLICATE_ID);
}

#[test]
fn replay_refuses_empty_supports_and_unknown_refs() {
    let ok = ev_line(1);
    let empty = ok.clone()
        + &line(
            2,
            &finding_body("F1", "").replace(r#""supports":["E1"]"#, r#""supports":[]"#),
        );
    let e = replay_err(&empty);
    assert_eq!(
        (e.line, e.refusal.code.as_str()),
        (2, codes::MISSING_EVIDENCE)
    );
    // supports omitted entirely behaves the same.
    let omitted = ok.clone()
        + &line(
            2,
            r#""type":"finding_added","finding":{"id":"F1","state":"candidate","origin":"census"}"#,
        );
    assert_eq!(replay_err(&omitted).refusal.code, codes::MISSING_EVIDENCE);
    let unknown = ok.clone() + &line(2, &finding_body("F1", "").replace("E1", "E9"));
    assert_eq!(replay_err(&unknown).refusal.code, codes::UNKNOWN_EVIDENCE);
    let before = line(1, &finding_body("F1", "")) + &ev_line(2);
    assert_eq!(replay_err(&before).refusal.code, codes::UNKNOWN_EVIDENCE);
    let bad_trans = ok
        + &line(
            2,
            r#""type":"transition","finding":"F5","from":"candidate","to":"under_test","evidence":[],"by":"x""#,
        );
    assert_eq!(replay_err(&bad_trans).refusal.code, codes::UNKNOWN_FINDING);
}

#[test]
fn replay_refuses_bad_findings() {
    let ok = ev_line(1);
    let cases = [
        (
            finding_body("F1", r#","archetype":"X""#),
            codes::ARCHETYPE_LOCKED,
        ),
        (
            finding_body("F1", "").replace("candidate", "confirmed"),
            codes::NOT_CANDIDATE,
        ),
        (
            finding_body(
                "F1.n1",
                r#","parent":"F1","hypothesis":"h","refute_if":"r""#,
            ),
            codes::UNKNOWN_PARENT,
        ),
        (
            finding_body("F1", r#","parent":"F2""#),
            codes::PARENT_MISMATCH,
        ),
        (
            finding_body("F1", r#","siblings":["F1.n1"]"#),
            codes::SIBLINGS_MANAGED,
        ),
    ];
    for (body, code) in cases {
        let e = replay_err(&(ok.clone() + &line(2, &body)));
        assert_eq!(e.refusal.code, code, "{body}");
        assert_eq!(e.line, 2);
    }
    let sib = ok.clone()
        + &line(2, &finding_body("F1", ""))
        + &line(3, &finding_body("F1.n1", r#","parent":"F1""#));
    assert_eq!(replay_err(&sib).refusal.code, codes::SIBLING_FIELDS);
    let dup = ok + &line(2, &finding_body("F1", "")) + &line(3, &finding_body("F1", ""));
    assert_eq!(replay_err(&dup).refusal.code, codes::DUPLICATE_ID);
}

#[test]
fn replay_refuses_bad_transitions() {
    let base = ev_line(1) + &line(2, &finding_body("F1", ""));
    let t = |from: &str, to: &str, extra: &str| {
        base.clone()
            + &line(
                3,
                &format!(
                    r#""type":"transition","finding":"F1","from":"{from}","to":"{to}","evidence":[],"by":"x"{extra}"#
                ),
            )
    };
    assert_eq!(
        replay_err(&t("under_test", "confirmed", "")).refusal.code,
        codes::FROM_MISMATCH
    );
    assert_eq!(
        replay_err(&t("candidate", "strong", "")).refusal.code,
        codes::ILLEGAL_TRANSITION
    );
    assert_eq!(
        replay_err(&t("candidate", "unresolved", "")).refusal.code,
        codes::REASON_REQUIRED
    );
    assert!(
        Ledger::from_text(&t(
            "candidate",
            "unresolved",
            r#","reason":"time_budget","remedy":"later""#
        ))
        .is_ok()
    );
}

#[test]
fn replay_malformed_and_blank_lines() {
    let e = replay_err("not json\n");
    assert!(e.malformed && e.line == 1);
    let e = replay_err(&(ev_line(1) + "\n" + &ev_line(2)));
    assert_eq!((e.line, e.refusal.code.as_str()), (2, codes::BLANK_LINE));
    let e = replay_err(&line(1, r#""type":"mystery""#));
    assert!(e.malformed);
    let e = replay_err("{\"seq\":1}\n");
    assert!(e.malformed && e.refusal.message.contains("schema_version"));
    assert_eq!(Error::from(e).exit_code(), 1);
}

#[test]
fn ledger_line_schema_versions() {
    let v = |n: u32| ev_line(1).replace("\"schema_version\":1", &format!("\"schema_version\":{n}"));
    let Err(e @ Error::Schema(_)) = Ledger::from_text(&v(6)) else {
        panic!("v6 should be refused")
    };
    assert_eq!(e.exit_code(), 3);
    let msg = e.to_string();
    assert!(
        msg.contains("newer dla")
            && msg.contains("schema 6")
            && msg.contains("reads 5")
            && msg.contains("upgrade the plugin"),
        "{msg}"
    );
    let Err(e @ Error::Schema(_)) = Ledger::from_text(&v(0)) else {
        panic!("v0 should be refused")
    };
    assert_eq!(e.exit_code(), 3);
    let msg = e.to_string();
    assert!(
        msg.contains("cannot migrate schema 0") && msg.contains("dla run init"),
        "{msg}"
    );
    assert!(msg.contains("ledger line 1"));
    // Versions 1 to 5 are all current and keep their number.
    assert_eq!(
        Ledger::from_text(&v(4)).unwrap().lines()[0].schema_version,
        4
    );
    assert_eq!(
        Ledger::from_text(&v(5)).unwrap().lines()[0].schema_version,
        5
    );
    assert!(Ledger::from_text(&v(1)).is_ok());
    let l = Ledger::from_text(&v(2)).unwrap();
    assert_eq!(l.lines()[0].schema_version, 2);
    let l = Ledger::from_text(&v(3)).unwrap();
    assert_eq!(l.lines()[0].schema_version, 3);
}

#[test]
fn open_items_and_resolution() {
    let mut l = ledger();
    add_f1(&mut l);
    l.open_item_add("check the pool", Some(fid(1)), "test")
        .unwrap();
    let o1 = OpenItemId::new(1);
    assert!(l.open_item(&o1).unwrap().resolution.is_none());
    let Error::Refused(r) = l
        .resolve_open_item(
            &o1,
            Resolution::Unresolved {
                reason: UnresolvedReason::TimeBudget,
                remedy: " ".into(),
            },
        )
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::REMEDY_REQUIRED);
    let Error::Refused(r) = l
        .resolve_open_item(&o1, Resolution::Hypothesis(fid(9)))
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::UNKNOWN_FINDING);
    l.resolve_open_item(&o1, Resolution::Hypothesis(fid(1)))
        .unwrap();
    let Error::Refused(r) = l
        .resolve_open_item(&o1, Resolution::Hypothesis(fid(1)))
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::ALREADY_RESOLVED);
    let Error::Refused(r) = l
        .resolve_open_item(&OpenItemId::new(5), Resolution::Hypothesis(fid(1)))
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::UNKNOWN_OPEN_ITEM);
}

#[test]
fn checklist_answers() {
    let mut l = ledger();
    let ans = |item: &str, status, ev: Vec<_>, reason, remedy: Option<&str>| ChecklistAnswer {
        item: item.into(),
        status,
        evidence: ev,
        note: None,
        reason,
        remedy: remedy.map(String::from),
    };
    for status in [AnswerStatus::Answered, AnswerStatus::Partial] {
        let Error::Refused(r) = l
            .answer_checklist(ans("intake.scope", status, vec![], None, None))
            .unwrap_err()
        else {
            panic!()
        };
        assert_eq!(r.code, codes::MISSING_EVIDENCE);
        assert!(
            r.fix
                .starts_with("dla ledger resolve intake.scope --status")
        );
    }
    let Error::Refused(r) = l
        .answer_checklist(ans(
            "intake.nope",
            AnswerStatus::Answered,
            vec![eid(1)],
            None,
            None,
        ))
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::UNKNOWN_CHECKLIST_ITEM);
    let Error::Refused(r) = l
        .answer_checklist(ans(
            "intake.scope",
            AnswerStatus::Unresolved,
            vec![],
            None,
            Some("r"),
        ))
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::REASON_REQUIRED);
    let Error::Refused(r) = l
        .answer_checklist(ans(
            "intake.scope",
            AnswerStatus::Unresolved,
            vec![],
            Some(UnresolvedReason::NoHarness),
            None,
        ))
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::REMEDY_REQUIRED);
    let Error::Refused(r) = l
        .answer_checklist(ans(
            "intake.scope",
            AnswerStatus::Answered,
            vec![eid(8)],
            None,
            None,
        ))
        .unwrap_err()
    else {
        panic!()
    };
    assert_eq!(r.code, codes::UNKNOWN_EVIDENCE);
    l.answer_checklist(ans(
        "intake.scope",
        AnswerStatus::Answered,
        vec![eid(1)],
        None,
        None,
    ))
    .unwrap();
    l.answer_checklist(ans(
        "intake.build",
        AnswerStatus::Unresolved,
        vec![],
        Some(UnresolvedReason::BuildFails),
        Some("fix the build"),
    ))
    .unwrap();
    let entries = l.checklist();
    let scope = entries
        .iter()
        .find(|e| e.item.id == "intake.scope")
        .unwrap();
    assert_eq!(scope.status_label(), "answered");
    assert_eq!(entries.iter().filter(|e| e.answer.is_none()).count(), 6);
}

#[test]
fn appends_to_file_only_and_reloads() {
    let dir = tempfile::tempdir().unwrap();
    let mut l = Ledger::create(dir.path()).unwrap();
    assert!(
        Ledger::create(dir.path()).is_err(),
        "create must not truncate"
    );
    l.add_evidence(evidence(
        dla_core::EvidenceKind::RunConfig,
        dla_core::Authority::Static,
    ))
    .unwrap();
    let path = dir.path().join("ledger.jsonl");
    let first = std::fs::read_to_string(&path).unwrap();
    l.add_finding(NewFinding::new(origin("census"), vec![eid(1)]))
        .unwrap();
    let second = std::fs::read_to_string(&path).unwrap();
    assert!(
        second.starts_with(&first),
        "earlier bytes must be untouched"
    );
    assert_eq!(second.lines().count(), 2);
    assert!(second.ends_with('\n'));
    // A refused write must not touch the file.
    assert!(
        l.add_finding(NewFinding::new(origin("census"), vec![]))
            .is_err()
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), second);

    let mut again = Ledger::load(dir.path()).unwrap();
    assert_eq!(again.lines().len(), 2);
    assert_eq!(again.lines()[0].seq, 1);
    assert_eq!(
        again.finding(&fid(1)).unwrap().state,
        FindingState::Candidate
    );
    again
        .transition(
            &fid(1),
            FindingState::Unresolved,
            Some(UnresolvedReason::TimeBudget),
            Some("more time".into()),
            vec![],
            "test",
        )
        .unwrap();
    assert_eq!(
        Ledger::load(dir.path())
            .unwrap()
            .finding(&fid(1))
            .unwrap()
            .state,
        FindingState::Unresolved
    );
}

#[test]
fn append_after_missing_trailing_newline() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ledger.jsonl");
    std::fs::write(&path, ev_line(1).trim_end()).unwrap();
    let mut l = Ledger::load(dir.path()).unwrap();
    l.add_evidence(evidence(
        dla_core::EvidenceKind::Probe,
        dla_core::Authority::Live,
    ))
    .unwrap();
    assert_eq!(Ledger::load(dir.path()).unwrap().lines().len(), 2);
}

#[test]
fn written_lines_have_the_documented_shape() {
    let mut l = Ledger::from_text("").unwrap();
    let line = l
        .add_evidence(evidence(
            dla_core::EvidenceKind::Probe,
            dla_core::Authority::Live,
        ))
        .unwrap();
    let v: serde_json::Value = serde_json::to_value(&line).unwrap();
    assert_eq!(v["schema_version"], 5);
    assert_eq!(v["seq"], 1);
    assert!(v.get("worker").is_none());
    assert_eq!(v["type"], "evidence");
    assert_eq!(v["evidence_id"], "E1");
    assert_eq!(v["source"], json!({"repo": {"commit": "abc123"}}));
    assert!(v["ts"].as_str().unwrap().ends_with('Z'));
}

#[test]
fn replay_refuses_unknown_and_misspelled_fields() {
    // Top level of a line.
    let e = replay_err(&line(1, &format!(r#""extra":"x",{}"#, EVIDENCE_BODY)));
    assert!(e.malformed && e.line == 1);
    assert!(
        e.refusal.message.contains("`extra`"),
        "{}",
        e.refusal.message
    );
    assert!(e.refusal.fix.contains("line 1"));

    // Nested finding object: a misspelled optional field is not silently dropped.
    let text = ev_line(1) + &line(2, &finding_body("F1", r#","holdr":"static/global""#));
    let e = replay_err(&text);
    assert_eq!(e.line, 2);
    assert!(
        e.refusal.message.contains("`finding.holdr`"),
        "{}",
        e.refusal.message
    );

    // Nested evidence object (source.repo).
    let body = EVIDENCE_BODY.replace(r#"{"commit":"abc"}"#, r#"{"commit":"abc","comit":"x"}"#);
    let e = replay_err(&line(1, &body));
    assert!(
        e.refusal.message.contains("`source.repo.comit`"),
        "{}",
        e.refusal.message
    );

    // Misspelled transition field.
    let text = ev_line(1)
        + &line(2, &finding_body("F1", ""))
        + &line(
            3,
            r#""type":"transition","finding":"F1","from":"candidate","to":"unresolved","reason":"time_budget","remedy":"r","remdy":"typo","evidence":[],"by":"t""#,
        );
    let e = replay_err(&text);
    assert_eq!(e.line, 3);
    assert!(e.refusal.message.contains("`remdy`"));

    // Explicit nulls and empty arrays for optional fields are fine.
    let text = ev_line(1) + &line(2, &finding_body("F1", r#","holder":null,"amplifiers":[]"#));
    Ledger::from_text(&text).unwrap();
}

#[test]
fn replay_refuses_duplicate_keys() {
    let e = replay_err(&line(1, &format!(r#""seq":1,{EVIDENCE_BODY}"#)));
    assert!(e.malformed && e.line == 1);
    assert!(
        e.refusal.message.contains("duplicate field `seq`"),
        "{}",
        e.refusal.message
    );
    let text = ev_line(1) + &line(2, &finding_body("F1", r#","site":"a:1","site":"b:2""#));
    let e = replay_err(&text);
    assert_eq!(e.line, 2);
    assert!(e.refusal.message.contains("duplicate field `site`"));
}

#[test]
fn add_finding_returns_the_finding_it_appended() {
    let mut l = ledger();
    add_f1(&mut l);
    l.add_finding(NewFinding::new(origin("census"), vec![eid(1)]))
        .unwrap();
    let mut sib = NewFinding::new(origin("hypothesis"), vec![eid(1)]);
    sib.sibling_of = Some(fid(1));
    sib.hypothesis = Some("N-WARM".into());
    sib.refute_if = Some("flat".into());
    let line = l.add_finding(sib).unwrap();
    match line.record {
        dla_core::Record::FindingAdded { finding } => {
            assert_eq!(finding.id.to_string(), "F1.n1");
            assert_eq!(finding.parent, Some(fid(1)));
        }
        other => panic!("{other:?}"),
    }
}

// ---- finding_annotated

#[test]
fn annotate_records_sites_without_changing_state() {
    let mut l = ledger();
    let f = add_f1(&mut l);
    let before = l.finding(&f).unwrap().clone();
    l.annotate(
        &f,
        vec!["src/A.java:10".into(), "src/B.java:3".into()],
        vec!["src/A.java:99".into()],
        Some("registry is cleared on shutdown only".into()),
    )
    .unwrap();
    l.annotate(
        &f,
        vec!["src/A.java:10".into(), "src/C.java:7".into()],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(
        l.finding(&f).unwrap(),
        &before,
        "state and fields unchanged"
    );
    let a = l.annotation(&f).unwrap();
    assert_eq!(a.writers, ["src/A.java:10", "src/B.java:3", "src/C.java:7"]);
    assert_eq!(a.removers, ["src/A.java:99"]);
    assert_eq!(a.notes, ["registry is cleared on shutdown only"]);
}

#[test]
fn annotate_refuses_unknown_findings_bad_sites_and_empty_annotations() {
    let mut l = ledger();
    let f = add_f1(&mut l);
    let e = l
        .annotate(&fid(9), vec!["a.go:1".into()], vec![], None)
        .unwrap_err();
    assert_eq!(e.to_refusal().code, codes::UNKNOWN_FINDING);
    for bad in [
        "a.go",
        "a.go:0",
        "a.go:x",
        ":3",
        "a.go:-1",
        "a.go:007",
        "a.go:99999999999",
    ] {
        let e = l.annotate(&f, vec![bad.into()], vec![], None).unwrap_err();
        assert_eq!(e.to_refusal().code, codes::INVALID_SITE, "{bad}");
    }
    let e = l
        .annotate(&f, vec![], vec!["nocolon".into()], None)
        .unwrap_err();
    assert_eq!(e.to_refusal().code, codes::INVALID_SITE);
    let e = l.annotate(&f, vec![], vec![], None).unwrap_err();
    assert_eq!(e.to_refusal().code, codes::EMPTY_ANNOTATION);
    let e = l
        .annotate(&f, vec![], vec![], Some("  ".into()))
        .unwrap_err();
    assert_eq!(e.to_refusal().code, codes::EMPTY_FIELD);
    assert!(l.annotation(&f).is_none());
}

#[test]
fn annotation_lines_replay_and_graders_can_skip_them() {
    let dir = tempfile::tempdir().unwrap();
    let mut l = Ledger::create(dir.path()).unwrap();
    l.add_evidence(evidence(
        dla_core::EvidenceKind::StaticHit,
        dla_core::Authority::Static,
    ))
    .unwrap();
    let f = add_f1(&mut l);
    l.annotate(&f, vec!["x/y.rs:5".into()], vec![], None)
        .unwrap();
    let text = std::fs::read_to_string(Ledger::path_in(dir.path())).unwrap();
    assert!(
        text.lines()
            .last()
            .unwrap()
            .contains(r#""type":"finding_annotated""#)
    );
    let again = Ledger::load(dir.path()).unwrap();
    assert_eq!(again.annotation(&f).unwrap().writers, ["x/y.rs:5"]);
    // An annotation of a finding that was never added is a replay error.
    let bad = format!(
        "{}{}",
        ev_line(1),
        line(
            2,
            r#""type":"finding_annotated","finding":"F1","writers":["a.go:1"]"#
        )
    );
    assert_eq!(replay_err(&bad).refusal.code, codes::UNKNOWN_FINDING);
}
