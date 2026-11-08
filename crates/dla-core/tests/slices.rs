//! Worker slices: replay on top of the main ledger, IDs, torn tails, `slice_done`, the
//! merge and its cross-worker gate. Everything runs against run directories on disk.

mod common;

use std::fs;

use common::*;
use dla_core::{
    CoverageManifest, Error, FindingState, Ledger, NewFinding, OpenItemId, Record, RunState,
    error::codes,
    finding::Holder,
    merge::merge,
    slice::{codes as m4, slice_path},
    state::{RunStatus, WorkerStatus, now_utc},
    view::{RunView, status},
};
use serde_json::json;

fn refusal<T: std::fmt::Debug>(r: Result<T, Error>) -> dla_core::Refusal {
    r.expect_err("expected a refusal").to_refusal()
}

fn slice_text(dir: &std::path::Path, w: &str) -> String {
    fs::read_to_string(slice_path(dir, w)).unwrap()
}

// ---------------------------------------------------------------- first write, IDs, replay

#[test]
fn the_first_write_starts_the_slice_and_marks_the_worker_running() {
    let dir = new_run(&["jvm", "rust"]);
    let main_before = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    let mut l = slice(dir.path(), "jvm");
    assert_eq!(l.worker(), Some("jvm"));
    assert_eq!(l.base_len(), 16);
    assert!(
        !slice_path(dir.path(), "jvm").exists(),
        "nothing written by loading"
    );

    let line = l
        .add_evidence(evidence(
            dla_core::EvidenceKind::Probe,
            dla_core::Authority::Live,
        ))
        .unwrap();
    // The slice has its own seq: slice_started is line 1, the evidence line 2.
    assert_eq!(line.seq, 2);
    let Record::Evidence(e) = &line.record else {
        panic!()
    };
    assert_eq!(e.evidence_id.to_string(), "jvm/E1");

    let text = slice_text(dir.path(), "jvm");
    let first: serde_json::Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    assert_eq!(first["type"], "slice_started");
    assert_eq!(first["worker"], "jvm");
    assert_eq!(first["runtime"], "jvm");
    assert_eq!(first["base_seq"], 16);
    assert_eq!(first["schema_version"], 3);
    assert!(first["deadline"].as_str().unwrap().ends_with('Z'));
    assert_eq!(text.lines().count(), 2);
    // The main ledger is untouched.
    assert_eq!(
        fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap(),
        main_before
    );
    // Registry: running, with a start time; the other worker is still registered.
    let st = RunState::load(dir.path()).unwrap();
    assert_eq!(st.worker("jvm").unwrap().status, WorkerStatus::Running);
    assert!(st.worker("jvm").unwrap().started_at.is_some());
    assert_eq!(st.worker("rust").unwrap().status, WorkerStatus::Registered);
}

#[test]
fn an_unregistered_worker_gets_its_name_as_runtime_and_the_run_deadline() {
    let dir = new_run(&[]);
    let mut l = slice(dir.path(), "go");
    candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    let first: serde_json::Value =
        serde_json::from_str(slice_text(dir.path(), "go").lines().next().unwrap()).unwrap();
    assert_eq!(first["runtime"], "go");
    let st = RunState::load(dir.path()).unwrap();
    assert_eq!(first["deadline"], json!(st.budget.deadline));
    assert!(st.workers.is_empty());
}

#[test]
fn slice_ids_carry_the_prefix_and_count_only_their_own() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    let f1 = candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    let f2 = candidate_in_slice(&mut j, "b.go:2", Holder::ThreadLocal);
    assert_eq!(
        (f1.to_string(), f2.to_string()),
        ("jvm/F1".into(), "jvm/F2".into())
    );
    let n = add_sibling(&mut j, &f1, "N-WARM");
    assert_eq!(n.to_string(), "jvm/F1.n1");
    j.open_item_add("look at the pool", Some(fid(1)), "test")
        .unwrap();
    assert!(j.open_item(&"jvm/O1".parse().unwrap()).is_some());
    // The other worker starts at 1 again, and the main ledger has no F1 of its own.
    let mut r = slice(dir.path(), "rust");
    assert_eq!(
        candidate_in_slice(&mut r, "a.go:1", Holder::StaticGlobal).to_string(),
        "rust/F1"
    );
    assert_eq!(r.next_evidence_id().to_string(), "rust/E1");
    assert_eq!(
        Ledger::load(dir.path())
            .unwrap()
            .next_finding_id()
            .to_string(),
        "F1"
    );
    // Reloading the slice continues the numbering.
    let j2 = slice(dir.path(), "jvm");
    assert_eq!(j2.next_finding_id().to_string(), "jvm/F3");
    assert_eq!(j2.lines().len(), j.lines().len());
    assert_eq!(j2.lines()[0].seq, 1);
}

#[test]
fn a_bare_id_means_the_workers_own_and_a_main_id_still_works() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    // `E1` exists only in the main ledger: it stays the main one.
    let f = candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    assert_eq!(f.to_string(), "jvm/F1");
    assert_eq!(
        l.finding(&f).unwrap().supports[0].to_string(),
        "E1",
        "E1 is the main ledger's static hit"
    );
    // Once the worker has its own E1, the bare one is the worker's.
    let e = l
        .add_evidence(evidence(
            dla_core::EvidenceKind::Probe,
            dla_core::Authority::Live,
        ))
        .unwrap();
    let Record::Evidence(e) = e.record else {
        panic!()
    };
    assert_eq!(e.evidence_id.to_string(), "jvm/E1");
    assert_eq!(l.own_evidence_id(&eid(1)).to_string(), "jvm/E1");
    assert_eq!(l.own_evidence_id(&"E2".parse().unwrap()).to_string(), "E2");
    assert_eq!(
        l.own_finding_id(&"jvm/F1".parse().unwrap()).to_string(),
        "jvm/F1"
    );
    // A bare F1 moves jvm/F1.
    l.transition(
        &fid(1),
        FindingState::Unresolved,
        Some(dla_core::UnresolvedReason::TimeBudget),
        Some("later".into()),
        vec![],
        "test",
    )
    .unwrap();
    assert_eq!(
        l.finding(&"jvm/F1".parse().unwrap()).unwrap().state,
        FindingState::Unresolved
    );
}

#[test]
fn a_worker_changes_only_its_own_items() {
    let dir = new_run(&["jvm"]);
    // The main ledger has a finding and an open item of its own.
    let mut main = Ledger::load(dir.path()).unwrap();
    add_leak(&mut main);
    main.open_item_add("core item", None, "test").unwrap();
    let mut l = slice(dir.path(), "jvm");
    // A bare F1 is the main one here (the worker has none): changing it is refused.
    let r = refusal(l.transition(
        &fid(1),
        FindingState::Unresolved,
        Some(dla_core::UnresolvedReason::TimeBudget),
        Some("later".into()),
        vec![],
        "test",
    ));
    assert_eq!(r.code, m4::FOREIGN_ITEM);
    assert!(r.message.contains("jvm") && r.message.contains("F1"), "{r}");
    let r = refusal(l.annotate(&fid(1), vec!["a.go:2".into()], vec![], None));
    assert_eq!(r.code, m4::FOREIGN_ITEM);
    let r = refusal(l.resolve_open_item(
        &OpenItemId::new(1),
        dla_core::Resolution::Unresolved {
            reason: dla_core::UnresolvedReason::TimeBudget,
            remedy: "later".into(),
        },
    ));
    assert_eq!(r.code, m4::FOREIGN_ITEM);
    // A sibling of a main finding is refused too.
    let mut nf = NewFinding::new(origin("hypothesis"), vec![eid(1)]);
    nf.sibling_of = Some(fid(1));
    nf.hypothesis = Some("N-WARM".into());
    nf.refute_if = Some("growth continues".into());
    let r = refusal(l.add_finding(nf));
    assert_eq!(r.code, m4::FOREIGN_ITEM);
    // Nothing was written by the refusals.
    assert!(!slice_path(dir.path(), "jvm").exists());
}

#[test]
fn a_slice_cannot_cite_a_main_id_created_after_it_started() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal); // base_seq = 16
    // The core appends E17 to the main ledger afterwards.
    let mut main = Ledger::load(dir.path()).unwrap();
    main.add_evidence(evidence(
        dla_core::EvidenceKind::Probe,
        dla_core::Authority::Live,
    ))
    .unwrap();
    assert!(
        Ledger::load(dir.path())
            .unwrap()
            .evidence_by_id(&eid(17))
            .is_some()
    );

    let mut l = slice(dir.path(), "jvm");
    let mut nf = NewFinding::new(origin("census"), vec![eid(17)]);
    nf.site = Some("b.go:1".into());
    let r = refusal(l.add_finding(nf));
    assert_eq!(r.code, codes::UNKNOWN_EVIDENCE, "{r}");
    // Citing an id up to base_seq still works.
    let mut nf = NewFinding::new(origin("census"), vec![eid(16)]);
    nf.site = Some("b.go:1".into());
    l.add_finding(nf).unwrap();

    // A hand-edited slice line that cites it fails the replay with a message saying why.
    let mut text = slice_text(dir.path(), "jvm");
    let seq = text.lines().count() + 1;
    text.push_str(&format!(
        "{{\"schema_version\":3,\"seq\":{seq},\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"finding_added\",\"finding\":{{\"id\":\"jvm/F9\",\"state\":\"candidate\",\"origin\":\"census\",\"supports\":[\"E17\"]}}}}\n"
    ));
    fs::write(slice_path(dir.path(), "jvm"), text).unwrap();
    let e = Ledger::load_for(dir.path(), Some("jvm")).unwrap_err();
    let r = e.to_refusal();
    assert_eq!(r.code, codes::UNKNOWN_EVIDENCE);
    assert!(r.message.contains("up to line 16"), "{r}");
}

#[test]
fn unprefixed_ids_in_a_slice_are_refused_on_replay() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    let mut text = slice_text(dir.path(), "jvm");
    let seq = text.lines().count() + 1;
    text.push_str(&format!(
        "{{\"schema_version\":3,\"seq\":{seq},\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"open_item\",\"id\":\"O1\",\"text\":\"x\",\"opened_by\":\"t\"}}\n"
    ));
    fs::write(slice_path(dir.path(), "jvm"), text).unwrap();
    let r = Ledger::load_for(dir.path(), Some("jvm"))
        .unwrap_err()
        .to_refusal();
    assert_eq!(r.code, m4::BAD_PREFIX, "{r}");
}

#[test]
fn bad_worker_names_are_refused() {
    let dir = new_run(&[]);
    for w in ["JVM", "1jvm", "a_b", "", "a/b", &"x".repeat(40)] {
        let r = Ledger::load_for(dir.path(), Some(w))
            .unwrap_err()
            .to_refusal();
        assert_eq!(r.code, m4::BAD_WORKER, "{w:?}");
    }
}

// ---------------------------------------------------------------- torn tail

#[test]
fn a_torn_final_slice_line_is_dropped_noted_and_cut_on_the_next_write() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    let path = slice_path(dir.path(), "jvm");
    let good = fs::read_to_string(&path).unwrap();
    let good_lines = good.lines().count();
    // A kill in the middle of the next line.
    let mut torn = good.clone();
    torn.push_str("{\"schema_version\":3,\"seq\":4,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"ev");
    fs::write(&path, &torn).unwrap();

    let mut l = slice(dir.path(), "jvm");
    assert_eq!(l.lines().len(), good_lines);
    let t = l.torn_tail().expect("the torn fragment is reported");
    assert_eq!(t.line, good_lines as u64 + 1);
    assert!(t.bytes > 0);
    assert!(l.finding(&"jvm/F1".parse().unwrap()).is_some());

    // The next write cuts the fragment off and the file is a clean slice again.
    candidate_in_slice(&mut l, "b.go:2", Holder::ThreadLocal);
    let after = fs::read_to_string(&path).unwrap();
    assert!(
        after.starts_with(&good),
        "complete lines are kept as they were"
    );
    assert!(!after.contains("\"type\":\"ev\n") && !after.contains("\"type\":\"ev"));
    let l = slice(dir.path(), "jvm");
    assert!(l.torn_tail().is_none());
    assert_eq!(l.lines().len(), good_lines + 1);
}

#[test]
fn a_final_slice_line_without_newline_that_is_complete_json_is_kept() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    let path = slice_path(dir.path(), "jvm");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.trim_end()).unwrap();
    let mut l = slice(dir.path(), "jvm");
    assert!(l.torn_tail().is_none());
    let n = l.lines().len();
    candidate_in_slice(&mut l, "b.go:2", Holder::ThreadLocal);
    assert_eq!(slice(dir.path(), "jvm").lines().len(), n + 1);
}

#[test]
fn the_main_ledger_stays_strict_about_a_torn_line() {
    let dir = new_run(&[]);
    let path = dir.path().join("ledger.jsonl");
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("{\"schema_version\":3,\"seq\":17,\"ts\":\"2026-01-01T00:0");
    fs::write(&path, text).unwrap();
    let e = Ledger::load(dir.path()).unwrap_err();
    assert_eq!(e.to_refusal().code, codes::MALFORMED);
    // A worker opening its slice also reads the main ledger strictly.
    assert!(Ledger::load_for(dir.path(), Some("jvm")).is_err());
}

#[test]
fn a_corrupt_middle_line_of_a_slice_is_an_error() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    candidate_in_slice(&mut l, "b.go:2", Holder::ThreadLocal);
    let path = slice_path(dir.path(), "jvm");
    let text = fs::read_to_string(&path).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[1] = "{not json";
    fs::write(&path, lines.join("\n") + "\n").unwrap();
    let e = Ledger::load_for(dir.path(), Some("jvm")).unwrap_err();
    assert_eq!(e.to_refusal().code, codes::MALFORMED);
}

// ---------------------------------------------------------------- slice_done

#[test]
fn done_lists_what_is_open_and_closes_the_slice() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    let a = candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    let b = candidate_in_slice(&mut l, "b.go:2", Holder::ThreadLocal);
    l.open_item_add("check the pool", Some(a.clone()), "test")
        .unwrap();
    l.open_item_add("check the cache", None, "test").unwrap();
    l.transition(
        &b,
        FindingState::Unresolved,
        Some(dla_core::UnresolvedReason::TimeBudget),
        Some("later".into()),
        vec![],
        "test",
    )
    .unwrap();
    l.resolve_open_item(
        &"jvm/O2".parse().unwrap(),
        dla_core::Resolution::Unresolved {
            reason: dla_core::UnresolvedReason::TimeBudget,
            remedy: "later".into(),
        },
    )
    .unwrap();
    let line = l.finish_slice().unwrap();
    let Record::SliceDone(d) = &line.record else {
        panic!()
    };
    assert_eq!(d.worker, "jvm");
    // jvm/F1 is a candidate and jvm/O1 is unresolved; jvm/F2 (unresolved with a reason)
    // and jvm/O2 (resolved) have dispositions.
    assert_eq!(d.open, ["jvm/F1", "jvm/O1"]);
    let st = RunState::load(dir.path()).unwrap();
    assert_eq!(st.worker("jvm").unwrap().status, WorkerStatus::Done);

    // The slice is closed.
    let r = refusal(l.add_evidence(evidence(
        dla_core::EvidenceKind::Probe,
        dla_core::Authority::Live,
    )));
    assert_eq!(r.code, m4::SLICE_CLOSED);
    let r = refusal(slice(dir.path(), "jvm").finish_slice());
    assert_eq!(r.code, m4::SLICE_CLOSED);
}

#[test]
fn done_is_refused_in_core_mode() {
    let dir = new_run(&["jvm"]);
    let mut core = Ledger::load(dir.path()).unwrap();
    let r = refusal(core.finish_slice());
    assert_eq!(r.code, m4::WORKER_ONLY);
    assert!(r.fix.contains("--worker"), "{r}");
}

#[test]
fn done_without_any_other_write_still_starts_the_slice() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    l.finish_slice().unwrap();
    let l = slice(dir.path(), "jvm");
    assert_eq!(l.lines().len(), 2);
    assert!(l.slice_info("jvm").unwrap().done.is_some());
}

#[test]
fn a_worker_cannot_write_core_records() {
    let dir = new_run(&["jvm"]);
    let mut l = slice(dir.path(), "jvm");
    let a = candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
    let r = refusal(l.verify(
        &a,
        None,
        Holder::StaticGlobal,
        "m",
        "UNCLASSIFIED",
        &"a".repeat(64),
    ));
    assert_eq!(r.code, m4::CORE_ONLY);
    let r = refusal(l.merge_findings(&a, &a));
    assert_eq!(r.code, m4::CORE_ONLY);
}

// ---------------------------------------------------------------- merge

#[test]
fn merge_copies_slice_lines_verbatim_and_replays() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    confirm_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    j.finish_slice().unwrap();
    let mut r = slice(dir.path(), "rust");
    candidate_in_slice(&mut r, "z.rs:9", Holder::NativeHandleGlobalRef);
    r.finish_slice().unwrap();
    let jvm_lines = slice(dir.path(), "jvm").lines().to_vec();

    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.merged_lines["jvm"], jvm_lines.len() as u64);
    assert_eq!(rep.merged_lines["rust"], 3);
    assert!(rep.unfinished.is_empty() && rep.duplicates.is_empty());

    let main = Ledger::load(dir.path()).unwrap();
    // 16 core lines + 2 slices + 2 slice_merged records.
    assert_eq!(main.lines().len(), 16 + jvm_lines.len() + 3 + 2);
    let copied: Vec<_> = main
        .lines()
        .iter()
        .filter(|l| {
            l.worker.as_deref() == Some("jvm") && !matches!(l.record, Record::SliceMerged(_))
        })
        .collect();
    assert_eq!(copied.len(), jvm_lines.len());
    for (c, o) in copied.iter().zip(&jvm_lines) {
        assert_eq!(c.record, o.record, "records are copied verbatim");
        assert_eq!(c.ts, o.ts, "the original timestamp is kept");
        assert_eq!(c.schema_version, o.schema_version);
        assert_eq!(c.seq, 16 + o.seq, "seq is re-stamped");
    }
    let f = main.finding(&"jvm/F1".parse().unwrap()).unwrap();
    assert_eq!(f.state, FindingState::Confirmed);
    assert_eq!(main.merged_lines("jvm"), jvm_lines.len() as u64);
    let m = main.slice_info("jvm").unwrap().merged.as_ref().unwrap();
    assert!(m.done && !m.torn_tail);
    assert_eq!(m.sha256.len(), 64);
    // Worker status is done.
    let st = RunState::load(dir.path()).unwrap();
    assert!(st.workers.iter().all(|w| w.status == WorkerStatus::Done));
    assert_eq!(st.status, RunStatus::Active);
}

#[test]
fn merge_is_idempotent_and_picks_up_new_lines() {
    let dir = new_run(&["jvm"]);
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    let first = merge(dir.path()).unwrap();
    assert_eq!(first.merged_lines["jvm"], 2);
    let size = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();

    // Nothing new: nothing is appended, byte for byte.
    let again = merge(dir.path()).unwrap();
    assert!(again.merged_lines.is_empty());
    assert_eq!(
        fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap(),
        size
    );

    // The worker writes more: only the new lines follow.
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "b.go:2", Holder::ThreadLocal);
    candidate_in_slice(&mut j, "c.go:3", Holder::ThreadLocal);
    j.finish_slice().unwrap();
    let third = merge(dir.path()).unwrap();
    assert_eq!(third.merged_lines["jvm"], 3);
    assert_eq!(third.total_lines["jvm"], 5);
    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(main.merged_lines("jvm"), 5);
    assert_eq!(
        main.lines()
            .iter()
            .filter(|l| l.worker.is_some() && !matches!(l.record, Record::SliceMerged(_)))
            .count(),
        5,
        "each slice line is in the main ledger once"
    );
    let n_merged = main
        .lines()
        .iter()
        .filter(|l| matches!(l.record, Record::SliceMerged(_)))
        .count();
    assert_eq!(n_merged, 2);
    // A third run changes nothing again.
    let text = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    merge(dir.path()).unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap(),
        text
    );
}

#[test]
fn merge_finds_slices_that_were_never_registered() {
    let dir = new_run(&[]);
    let mut g = slice(dir.path(), "go");
    candidate_in_slice(&mut g, "a.go:1", Holder::StaticGlobal);
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.merged_lines["go"], 2);
    assert_eq!(rep.unfinished.len(), 1, "no slice_done");
    assert_eq!(rep.unfinished[0].worker, "go");
}

#[test]
fn an_invalid_slice_is_noted_and_the_other_slices_still_merge() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    candidate_in_slice(&mut j, "b.go:2", Holder::StaticGlobal);
    let path = slice_path(dir.path(), "jvm");
    let text = fs::read_to_string(&path).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[1] = "{not json";
    fs::write(&path, lines.join("\n") + "\n").unwrap();
    // rust sorts after jvm: it used to be skipped with the error.
    let mut r = slice(dir.path(), "rust");
    candidate_in_slice(&mut r, "z.rs:1", Holder::StaticGlobal);
    r.finish_slice().unwrap();

    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.invalid_slices, ["jvm"]);
    assert_eq!(rep.merged_lines.get("rust"), Some(&3));
    assert_eq!(rep.unfinished.len(), 1);
    assert_eq!(rep.unfinished[0].worker, "jvm");
    let main = Ledger::load(dir.path()).unwrap();
    let item = main
        .open_items()
        .find(|o| o.item.text.starts_with("slice of worker jvm is invalid"))
        .expect("an open item names the bad slice");
    assert!(
        item.item.text.contains("ledger.jvm.jsonl"),
        "{}",
        item.item.text
    );
    let st = RunState::load(dir.path()).unwrap();
    assert_eq!(st.worker("jvm").unwrap().status, WorkerStatus::Killed);
    assert_eq!(st.worker("rust").unwrap().status, WorkerStatus::Done);
    assert_eq!(st.status, RunStatus::Incomplete);

    // Repeating does not raise the item again; fixing the line lets the slice in.
    merge(dir.path()).unwrap();
    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(
        main.open_items()
            .filter(|o| o.item.text.starts_with("slice of worker jvm is invalid"))
            .count(),
        1
    );
    fs::write(&path, text).unwrap();
    let rep = merge(dir.path()).unwrap();
    assert!(rep.invalid_slices.is_empty());
    assert_eq!(rep.merged_lines.get("jvm"), Some(&3));
}

#[test]
fn a_tear_inside_a_multibyte_character_is_dropped_like_any_other_torn_tail() {
    use std::io::Write;
    let dir = new_run(&["go", "jvm"]);
    let mut g = slice(dir.path(), "go");
    candidate_in_slice(&mut g, "a.go:1", Holder::StaticGlobal);
    let mut j = slice(dir.path(), "jvm");
    j.finish_slice().unwrap();
    let path = slice_path(dir.path(), "go");
    let mut f = fs::OpenOptions::new().append(true).open(&path).unwrap();
    f.write_all(b"{\"schema_version\":3,\"seq\":4,\"type\":\"note\",\"text\":\"caf\xC3")
        .unwrap();
    drop(f);

    // The merge is not blocked and says the tail was torn.
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.torn_tail, ["go"]);
    assert!(rep.invalid_slices.is_empty());
    assert_eq!(rep.merged_lines.get("go"), Some(&2));
    // The worker restarts: the fragment is reported, then cut by the next write.
    drop(g);
    let mut g = slice(dir.path(), "go");
    assert!(g.torn_tail().is_some());
    candidate_in_slice(&mut g, "b.go:2", Holder::StaticGlobal);
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(text.lines().count(), 3);
    assert!(text.ends_with('\n'));
}

// ---------------------------------------------------------------- cross-worker gate

#[test]
fn two_workers_with_the_same_site_and_holder_merge_into_one_finding() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    let jf = confirm_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    j.finish_slice().unwrap();
    let mut r = slice(dir.path(), "rust");
    let rf = candidate_in_slice(&mut r, "a.go:1", Holder::StaticGlobal);
    // A finding of the same worker at another site stays separate.
    candidate_in_slice(&mut r, "b.go:2", Holder::StaticGlobal);
    r.finish_slice().unwrap();

    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.duplicates.len(), 1);
    assert_eq!(
        rep.duplicates[0].canonical,
        jf.to_string(),
        "confirmed beats candidate"
    );
    assert_eq!(rep.duplicates[0].duplicate, rf.to_string());

    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(main.merged_into(&rf), Some(&jf));
    assert_eq!(main.duplicates_of(&jf), vec![&rf]);
    // Supports are unioned into the canonical finding.
    let canonical = main.finding(&jf).unwrap();
    assert!(canonical.supports.iter().any(|e| e.to_string() == "E1"));
    // The duplicate is not counted, nor does it block.
    let view = RunView::new(run_state(), main.clone(), None);
    let st = status(&view);
    assert_eq!(
        st.counts.enumerated, 3,
        "jvm/F1, jvm/F1.n1 and rust/F2, not rust/F1"
    );
    assert!(
        !st.blocking.iter().any(|b| b.id == rf.to_string()),
        "{:?}",
        st.blocking
    );
    assert!(st.blocking.iter().any(|b| b.id == "rust/F2"));

    // Merging again does not repeat it.
    let again = merge(dir.path()).unwrap();
    assert!(again.duplicates.is_empty());
}

#[test]
fn equally_advanced_duplicates_go_to_the_lower_worker_name() {
    let dir = new_run(&["jvm", "rust", "go"]);
    for w in ["rust", "jvm", "go"] {
        let mut l = slice(dir.path(), w);
        candidate_in_slice(&mut l, "a.go:1", Holder::StaticGlobal);
        l.finish_slice().unwrap();
    }
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.duplicates.len(), 2);
    assert!(rep.duplicates.iter().all(|d| d.canonical == "go/F1"));
    let mut dups: Vec<&str> = rep
        .duplicates
        .iter()
        .map(|d| d.duplicate.as_str())
        .collect();
    dups.sort();
    assert_eq!(dups, ["jvm/F1", "rust/F1"]);
}

#[test]
fn findings_without_a_site_or_holder_never_merge() {
    let dir = new_run(&["jvm", "rust"]);
    for w in ["jvm", "rust"] {
        let mut l = slice(dir.path(), w);
        l.add_finding(NewFinding::new(origin("census"), vec![eid(1)]))
            .unwrap();
        let mut nf = NewFinding::new(origin("census"), vec![eid(1)]);
        nf.site = Some("a.go:1".into());
        l.add_finding(nf).unwrap();
        l.finish_slice().unwrap();
    }
    assert!(merge(dir.path()).unwrap().duplicates.is_empty());
}

#[test]
fn confirmed_in_one_worker_and_refuted_in_another_is_a_contradiction() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    let jf = confirm_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    j.finish_slice().unwrap();
    let mut r = slice(dir.path(), "rust");
    let rf = candidate_in_slice(&mut r, "a.go:1", Holder::StaticGlobal);
    // E2 is a discriminator that favors non-leak: it refutes the leak-side finding.
    go(&mut r, &rf, FindingState::Refuted, &[E_DISC_NONLEAK]);
    r.finish_slice().unwrap();

    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.contradictions.len(), 1);
    assert!(
        rep.duplicates.is_empty(),
        "a refuted finding is a verdict, not a duplicate"
    );
    let text = &rep.contradictions[0];
    assert!(
        text.contains("a.go:1")
            && text.contains("static/global")
            && text.contains(&jf.to_string())
            && text.contains(&rf.to_string()),
        "{text}"
    );
    let main = Ledger::load(dir.path()).unwrap();
    let item = main
        .open_items()
        .find(|o| o.item.text == *text)
        .expect("an open item");
    assert!(item.resolution.is_none());
    assert_eq!(item.item.opened_by, "dla ledger merge");
    assert_eq!(
        item.item.id.to_string(),
        "O1",
        "the core allocates unprefixed IDs"
    );
    // It blocks, and a second merge does not raise it again.
    let view = RunView::new(run_state(), main, None);
    assert!(status(&view).blocking.iter().any(|b| b.id == "O1"));
    assert!(merge(dir.path()).unwrap().contradictions.is_empty());
}

#[test]
fn a_worker_without_slice_done_leaves_an_open_item_and_the_run_incomplete() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    j.finish_slice().unwrap();
    // rust writes three lines and is killed.
    let mut r = slice(dir.path(), "rust");
    candidate_in_slice(&mut r, "z.rs:1", Holder::StaticGlobal);
    candidate_in_slice(&mut r, "z.rs:2", Holder::StaticGlobal);

    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.unfinished.len(), 1);
    assert_eq!(rep.unfinished[0].worker, "rust");
    assert_eq!(rep.unfinished[0].slice_lines, 3);
    assert_eq!(rep.unfinished[0].status, "killed");
    assert_eq!(rep.status, RunStatus::Incomplete);

    let main = Ledger::load(dir.path()).unwrap();
    assert!(main.open_items().any(|o| o.item.text
        == "worker rust did not finish (slice ends at line 3)"
        && o.resolution.is_none()));
    let st = RunState::load(dir.path()).unwrap();
    assert_eq!(st.status, RunStatus::Incomplete);
    assert!(st.incomplete_reason.as_deref().unwrap().contains("rust"));
    assert_eq!(st.worker("rust").unwrap().status, WorkerStatus::Killed);
    assert_eq!(st.worker("jvm").unwrap().status, WorkerStatus::Done);
    let view = RunView::new(st, main, None);
    let s = status(&view);
    assert!(!s.complete);
    assert!(
        s.blocking
            .iter()
            .any(|b| matches!(b.kind, dla_core::view::BlockingKind::Run))
    );

    // A second merge does not raise a second item.
    merge(dir.path()).unwrap();
    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(
        main.open_items()
            .filter(|o| o.item.text.starts_with("worker rust did not finish"))
            .count(),
        1
    );
}

#[test]
fn a_registered_worker_with_no_slice_and_a_passed_deadline_is_over_budget() {
    let dir = new_run(&["jvm"]);
    RunState::update(dir.path(), |s| {
        s.workers[0].deadline = now_utc() - chrono::TimeDelta::minutes(5);
        Ok(())
    })
    .unwrap();
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.unfinished[0].slice_lines, 0);
    assert_eq!(rep.unfinished[0].status, "over-budget");
    let main = Ledger::load(dir.path()).unwrap();
    assert!(
        main.open_items()
            .any(|o| o.item.text == "worker jvm did not finish (slice ends at line 0)")
    );
    let st = RunState::load(dir.path()).unwrap();
    assert_eq!(st.worker("jvm").unwrap().status, WorkerStatus::OverBudget);
    assert_eq!(st.status, RunStatus::Incomplete);
}

#[test]
fn a_late_worker_that_finishes_clears_the_incomplete_status_the_merge_set() {
    let dir = new_run(&["jvm"]);
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    merge(dir.path()).unwrap();
    assert_eq!(
        RunState::load(dir.path()).unwrap().status,
        RunStatus::Incomplete
    );
    let mut j = slice(dir.path(), "jvm");
    j.finish_slice().unwrap();
    let rep = merge(dir.path()).unwrap();
    assert!(rep.unfinished.is_empty());
    let st = RunState::load(dir.path()).unwrap();
    assert_eq!(st.status, RunStatus::Active);
    assert!(st.incomplete_reason.is_none());
    assert_eq!(st.worker("jvm").unwrap().status, WorkerStatus::Done);
}

#[test]
fn a_worker_that_finishes_after_a_merge_closes_its_did_not_finish_item() {
    let dir = new_run(&["jvm"]);
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    merge(dir.path()).unwrap();
    let open = |dir: &std::path::Path| {
        Ledger::load(dir)
            .unwrap()
            .open_items()
            .filter(|o| o.item.text.starts_with("worker jvm did not finish"))
            .map(|o| o.resolution.is_none())
            .collect::<Vec<_>>()
    };
    assert_eq!(open(dir.path()), [true]);
    j.finish_slice().unwrap();
    merge(dir.path()).unwrap();
    assert_eq!(
        open(dir.path()),
        [false],
        "closed by the merge, not raised twice"
    );
    let main = Ledger::load(dir.path()).unwrap();
    let view = RunView::new(RunState::load(dir.path()).unwrap(), main, None);
    assert!(
        !status(&view)
            .blocking
            .iter()
            .any(|b| b.description.contains("did not finish")),
    );
}

#[test]
fn an_item_the_core_resolved_is_not_raised_again_by_the_next_merge() {
    let dir = new_run(&["jvm"]);
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    merge(dir.path()).unwrap();
    let mut main = Ledger::load(dir.path()).unwrap();
    let id = main.open_items().next().unwrap().item.id.clone();
    main.resolve_open_item(
        &id,
        dla_core::record::Resolution::Unresolved {
            reason: dla_core::finding::UnresolvedReason::TimeBudget,
            remedy: "rerun the jvm worker".into(),
        },
    )
    .unwrap();
    merge(dir.path()).unwrap();
    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(main.open_items().count(), 1);
}

#[test]
fn a_run_with_unfinished_workers_points_at_finishing_them_not_at_run_init() {
    let dir = new_run(&["jvm"]);
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    merge(dir.path()).unwrap();
    let main = Ledger::load(dir.path()).unwrap();
    let view = RunView::new(RunState::load(dir.path()).unwrap(), main, None);
    let run = status(&view)
        .blocking
        .into_iter()
        .find(|b| matches!(b.kind, dla_core::view::BlockingKind::Run))
        .unwrap();
    assert!(
        run.fix.contains("ledger merge") && !run.fix.contains("run init"),
        "{}",
        run.fix
    );
}

fn confirm_existing(l: &mut Ledger, f: &dla_core::FindingId) {
    let n = add_sibling(l, f, "N-WARM");
    go(l, &n, FindingState::Refuted, &[E_DISC_LEAK]);
    go(l, f, FindingState::UnderTest, &[E_HARNESS]);
    go(l, f, FindingState::Confirmed, &[E_GROWTH_PASS]);
}

#[test]
fn a_duplicate_that_is_confirmed_after_the_merge_becomes_the_canonical_finding() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    let jf = candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    let mut r = slice(dir.path(), "rust");
    let rf = candidate_in_slice(&mut r, "a.go:1", Holder::StaticGlobal);
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.duplicates[0].canonical, jf.to_string());

    confirm_existing(&mut r, &rf);
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.duplicates.len(), 1);
    assert_eq!(rep.duplicates[0].canonical, rf.to_string());
    assert_eq!(rep.duplicates[0].duplicate, jf.to_string());

    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(main.merged_into(&jf), Some(&rf));
    assert_eq!(main.merged_into(&rf), None);
    let view = RunView::new(RunState::load(dir.path()).unwrap(), main, None);
    let st = status(&view);
    assert_eq!(st.counts.by_state.confirmed, 1, "{:?}", st.counts);

    // Stable: nothing more to do.
    assert!(merge(dir.path()).unwrap().duplicates.is_empty());
}

#[test]
fn a_better_third_finding_takes_over_without_a_chain() {
    let dir = new_run(&["jvm", "rust", "zig"]);
    let mut j = slice(dir.path(), "jvm");
    let jf = candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    let mut r = slice(dir.path(), "rust");
    let rf = candidate_in_slice(&mut r, "a.go:1", Holder::StaticGlobal);
    merge(dir.path()).unwrap();
    let mut z = slice(dir.path(), "zig");
    let zf = confirm_in_slice(&mut z, "a.go:1", Holder::StaticGlobal);
    merge(dir.path()).unwrap();
    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(main.merged_into(&jf), Some(&zf));
    assert_eq!(
        main.merged_into(&rf),
        Some(&zf),
        "no jvm -> zig <- rust chain"
    );
    let mut dups: Vec<String> = main
        .duplicates_of(&zf)
        .iter()
        .map(|d| d.to_string())
        .collect();
    dups.sort();
    assert_eq!(dups, [jf.to_string(), rf.to_string()]);
}

#[test]
fn supports_added_to_a_duplicate_after_the_merge_reach_the_canonical_finding() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    let jf = candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    let mut r = slice(dir.path(), "rust");
    let rf = candidate_in_slice(&mut r, "a.go:1", Holder::StaticGlobal);
    merge(dir.path()).unwrap();
    // Both move on to the same state; the duplicate cites one more piece of evidence.
    for (l, f, ev) in [
        (&mut j, &jf, vec![E_HARNESS]),
        (&mut r, &rf, vec![E_HARNESS, E_DISCOVER]),
    ] {
        let n = add_sibling(l, f, "N-WARM");
        go(l, &n, FindingState::Refuted, &[E_DISC_LEAK]);
        go(l, f, FindingState::UnderTest, &ev);
    }
    let extra = eid(E_DISCOVER);
    let main = Ledger::load(dir.path()).unwrap();
    assert!(!main.finding(&jf).unwrap().supports.contains(&extra));
    merge(dir.path()).unwrap();
    let main = Ledger::load(dir.path()).unwrap();
    assert_eq!(main.merged_into(&rf), Some(&jf));
    assert!(main.finding(&jf).unwrap().supports.contains(&extra));
    // And it settles.
    let n = main.lines().len();
    merge(dir.path()).unwrap();
    assert_eq!(Ledger::load(dir.path()).unwrap().lines().len(), n);
}

#[test]
fn an_aborted_run_stays_aborted() {
    let dir = new_run(&["jvm"]);
    RunState::update(dir.path(), |s| {
        s.status = RunStatus::Aborted;
        s.incomplete_reason = Some("user".into());
        Ok(())
    })
    .unwrap();
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.status, RunStatus::Aborted);
    assert_eq!(
        RunState::load(dir.path())
            .unwrap()
            .incomplete_reason
            .as_deref(),
        Some("user")
    );
}

#[test]
fn a_probed_runtime_without_a_worker_becomes_a_blind_spot() {
    let dir = new_run(&["jvm"]);
    let probe = json!({
        "schema_version": 1, "project": "/p",
        "runtimes": [
            {"runtime": "jvm", "detected_by": ["pom.xml"], "source_files": 3},
            {"runtime": "rust", "detected_by": ["Cargo.toml"], "source_files": 2}
        ],
        "privilege": {"os": "linux", "root": false}
    });
    fs::create_dir_all(dir.path().join("probe")).unwrap();
    fs::write(dir.path().join("probe/probe.json"), probe.to_string()).unwrap();
    let mut j = slice(dir.path(), "jvm");
    j.finish_slice().unwrap();
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.uncovered_runtimes, ["rust"]);
    let cov = CoverageManifest::load(dir.path()).unwrap().unwrap();
    assert_eq!(cov.blind_spots.len(), 1);
    assert_eq!(cov.blind_spots[0].runtime, "rust");
    assert!(cov.blind_spots[0].text.contains("no worker"));
    // Once per runtime however often the merge runs.
    merge(dir.path()).unwrap();
    assert_eq!(
        CoverageManifest::load(dir.path())
            .unwrap()
            .unwrap()
            .blind_spots
            .len(),
        1
    );
}

#[test]
fn a_run_without_workers_merges_nothing_and_changes_nothing() {
    let dir = new_run(&[]);
    let before = fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    let rep = merge(dir.path()).unwrap();
    assert!(rep.merged_lines.is_empty() && rep.unfinished.is_empty());
    assert_eq!(rep.status, RunStatus::Active);
    assert_eq!(
        fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap(),
        before
    );
}

// ---------------------------------------------------------------- merged ledger

#[test]
fn a_merged_slice_with_a_torn_tail_records_it() {
    let dir = new_run(&["jvm"]);
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    let path = slice_path(dir.path(), "jvm");
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str("{\"schema_version\":3,\"seq\":4,\"ts\":");
    fs::write(&path, text).unwrap();
    let rep = merge(dir.path()).unwrap();
    assert_eq!(rep.torn_tail, ["jvm"]);
    assert_eq!(rep.merged_lines["jvm"], 2);
    let main = Ledger::load(dir.path()).unwrap();
    assert!(
        main.slice_info("jvm")
            .unwrap()
            .merged
            .as_ref()
            .unwrap()
            .torn_tail
    );
}

#[test]
fn slice_records_are_refused_where_they_do_not_belong() {
    let bad = new_run(&["jvm"]);
    // A hand-written slice that does not begin with slice_started.
    fs::write(
        slice_path(bad.path(), "jvm"),
        "{\"schema_version\":3,\"seq\":1,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"open_item\",\"id\":\"jvm/O1\",\"text\":\"x\",\"opened_by\":\"t\"}\n",
    )
    .unwrap();
    let r = Ledger::load_for(bad.path(), Some("jvm"))
        .unwrap_err()
        .to_refusal();
    assert_eq!(r.code, m4::SLICE_STATE);
    // A second slice_started in the main ledger for the same worker.
    let dir = new_run(&[]);
    let mut j = slice(dir.path(), "go");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    merge(dir.path()).unwrap();
    let mut main = Ledger::load(dir.path()).unwrap();
    let started = main
        .lines()
        .iter()
        .find_map(|l| match &l.record {
            Record::SliceStarted(s) if s.worker == "go" => Some(s.clone()),
            _ => None,
        })
        .unwrap();
    let r = refusal(main.append_merged(
        "go",
        &[dla_core::LedgerLine {
            schema_version: 3,
            seq: 1,
            ts: now_utc(),
            worker: None,
            record: Record::SliceStarted(started),
        }],
        dla_core::SliceMerged {
            worker: "go".into(),
            lines: 9,
            sha256: "a".repeat(64),
            torn_tail: false,
            done: false,
        },
    ));
    assert_eq!(r.code, m4::SLICE_STATE);
    // finding_merged needs two findings of different workers.
    let r = refusal(main.merge_findings(&"go/F1".parse().unwrap(), &"go/F1".parse().unwrap()));
    assert_eq!(r.code, m4::MERGE_STATE);
}

// ---------------------------------------------------------------- verification

fn confirmed_in_main(dir: &std::path::Path) {
    let mut l = Ledger::load(dir).unwrap();
    let f = add_leak(&mut l);
    let n = add_sibling(&mut l, &f, "N-WARM");
    go(&mut l, &n, FindingState::Refuted, &[E_DISC_LEAK]);
    go(&mut l, &f, FindingState::UnderTest, &[E_HARNESS]);
    go(&mut l, &f, FindingState::Confirmed, &[E_GROWTH_PASS]);
}

#[test]
fn a_verification_that_agrees_opens_nothing() {
    let dir = new_run(&[]);
    confirmed_in_main(dir.path());
    let mut l = Ledger::load(dir.path()).unwrap();
    let (line, item) = l
        .verify(
            &fid(1),
            Some("a.go:3".into()),
            Holder::StaticGlobal,
            "a static map keeps every request",
            "UNCLASSIFIED",
            &"b".repeat(64),
        )
        .unwrap();
    assert!(item.is_none());
    let Record::Verification(v) = &line.record else {
        panic!()
    };
    assert!(v.agrees_site && v.agrees_holder);
    let again = Ledger::load(dir.path()).unwrap();
    assert_eq!(again.verifications(&fid(1)).len(), 1);
    assert_eq!(
        again.verifications(&fid(1))[0].mechanism,
        "a static map keeps every request"
    );
    assert_eq!(again.open_items().count(), 0);
}

#[test]
fn a_verification_that_disagrees_opens_an_item_naming_both_values() {
    let dir = new_run(&[]);
    confirmed_in_main(dir.path());
    let mut l = Ledger::load(dir.path()).unwrap();
    let (line, item) = l
        .verify(
            &fid(1),
            Some("a.go:40".into()),
            Holder::ThreadLocal,
            "a thread local",
            "UNCLASSIFIED",
            &"b".repeat(64),
        )
        .unwrap();
    let Record::Verification(v) = &line.record else {
        panic!()
    };
    assert!(!v.agrees_site && !v.agrees_holder);
    let Some(Record::OpenItem(o)) = item.map(|i| i.record) else {
        panic!("an open item")
    };
    for needle in ["a.go:1", "a.go:40", "static/global", "thread-local"] {
        assert!(o.text.contains(needle), "{needle}: {}", o.text);
    }
    assert_eq!(o.finding, Some(fid(1)));
    // none vs a site disagrees as well.
    let (_, item) = l
        .verify(
            &fid(1),
            None,
            Holder::StaticGlobal,
            "m",
            "UNCLASSIFIED",
            &"b".repeat(64),
        )
        .unwrap();
    assert!(item.is_some());
}

#[test]
fn verification_refusals() {
    let dir = new_run(&[]);
    confirmed_in_main(dir.path());
    let mut l = Ledger::load(dir.path()).unwrap();
    let ok = "b".repeat(64);
    let r = refusal(l.verify(
        &fid(9),
        None,
        Holder::StaticGlobal,
        "m",
        "UNCLASSIFIED",
        &ok,
    ));
    assert_eq!(r.code, codes::UNKNOWN_FINDING);
    let r = refusal(l.verify(
        &fid(1),
        None,
        Holder::StaticGlobal,
        "  ",
        "UNCLASSIFIED",
        &ok,
    ));
    assert_eq!(r.code, m4::BAD_VERIFICATION);
    let r = refusal(l.verify(&fid(1), None, Holder::StaticGlobal, "m", "two words", &ok));
    assert_eq!(r.code, m4::BAD_VERIFICATION);
    let r = refusal(l.verify(
        &fid(1),
        None,
        Holder::StaticGlobal,
        "m",
        "UNCLASSIFIED",
        "abc",
    ));
    assert_eq!(r.code, m4::BAD_VERIFICATION);
    assert!(r.fix.contains("dla ledger verify F1"), "{r}");
    let r = refusal(l.verify(
        &fid(1),
        Some("nofile".into()),
        Holder::StaticGlobal,
        "m",
        "UNCLASSIFIED",
        &ok,
    ));
    assert_eq!(r.code, m4::BAD_VERIFICATION);
    assert_eq!(
        Ledger::load(dir.path())
            .unwrap()
            .verifications(&fid(1))
            .len(),
        0
    );
}

// ---------------------------------------------------------------- replay of old ledgers

#[test]
fn v1_v2_and_v3_lines_replay_in_one_ledger() {
    let v = |n: u32, seq: u64, body: &str| {
        format!("{{\"schema_version\":{n},\"seq\":{seq},\"ts\":\"2026-01-01T00:00:00Z\",{body}}}\n")
    };
    let text = [
        v(1, 1, EVIDENCE_BODY),
        v(
            1,
            2,
            r#""type":"finding_added","finding":{"id":"F1","state":"candidate","origin":"census","supports":["E1"]}"#,
        ),
        v(2, 3, r#""type":"open_item","id":"O1","text":"look","opened_by":"t""#),
        v(3, 4, r#""type":"open_item","id":"jvm/O1","text":"look too","opened_by":"t""#)
            .replace("\"seq\":4", "\"seq\":4,\"worker\":\"jvm\""),
    ]
    .concat();
    let l = Ledger::from_text(&text).unwrap();
    assert_eq!(l.lines().len(), 4);
    assert_eq!(l.lines()[3].worker.as_deref(), Some("jvm"));
    assert_eq!(l.open_items().count(), 2);
    // The worker field is optional and absent from lines that have none.
    let back = serde_json::to_string(&l.lines()[0]).unwrap();
    assert!(!back.contains("worker"));
}

#[test]
fn unknown_fields_are_still_refused_on_v3_lines() {
    let text = "{\"schema_version\":3,\"seq\":1,\"ts\":\"2026-01-01T00:00:00Z\",\"wrker\":\"jvm\",\"type\":\"open_item\",\"id\":\"O1\",\"text\":\"x\",\"opened_by\":\"t\"}\n";
    let r = Ledger::from_text(text).unwrap_err().to_refusal();
    assert_eq!(r.code, codes::MALFORMED);
    assert!(r.message.contains("wrker"));
}
