//! The chronological order of records: the key `Ledger::evidence_order` and the iterator
//! `Ledger::evidence_in_order` are time in a plain ledger, in a worker's view of the run and in
//! the merged ledger, where the evidence ID is not (`W/E1` sorts after every plain `E<n>`).

mod common;

use common::*;
use dla_core::{Authority, EvidenceKind, Ledger, merge::merge};

fn probe() -> dla_core::NewEvidence {
    evidence(EvidenceKind::Probe, Authority::Live)
}

fn ids(l: &Ledger) -> Vec<String> {
    l.evidence().map(|e| e.evidence_id.to_string()).collect()
}

fn in_order(l: &Ledger) -> Vec<String> {
    l.evidence_in_order()
        .map(|e| e.evidence_id.to_string())
        .collect()
}

#[test]
fn in_a_plain_ledger_the_key_is_the_line_and_the_orders_agree() {
    let dir = new_run(&[]);
    let mut l = Ledger::load(dir.path()).unwrap();
    let first = l.add_evidence(probe()).unwrap();
    let second = l.add_evidence(probe()).unwrap();
    let key = |n: u32| l.evidence_order(&eid(n)).unwrap();
    // E1 to E16 are the lines 1 to 16 of the fixture; the new ones follow.
    assert_eq!((key(1).at, key(16).at), (1, 16));
    assert_eq!((first.seq, second.seq), (17, 18));
    assert_eq!(key(17).at, 17);
    assert_eq!(key(18).at, 18);
    assert!(key(16) < key(17) && key(17) < key(18));
    assert_eq!(ids(&l), in_order(&l));
    // A replay of the file gives the same keys.
    let again = Ledger::load(dir.path()).unwrap();
    assert_eq!(again.evidence_order(&eid(18)), Some(key(18)));
    // The finding keys count lines too: a finding's line comes after the evidence it cites.
    let f = candidate_in_slice(&mut l, "a.go:1", dla_core::finding::Holder::StaticGlobal);
    assert_eq!(l.finding_order(&f).map(|k| k.at), Some(19));
    assert!(l.finding_order(&fid(99)).is_none());
}

#[test]
fn in_a_workers_view_the_slice_comes_after_the_main_lines_it_was_started_on() {
    let dir = new_run(&["jvm"]);
    let mut w = slice(dir.path(), "jvm");
    let base = w.base_len();
    assert_eq!(base, 16);
    let mut main = Ledger::load(dir.path()).unwrap();
    w.add_evidence(probe()).unwrap();
    w.add_evidence(probe()).unwrap();
    // The core writes a line after the slice started: it is not in the worker's view.
    main.add_evidence(probe()).unwrap();
    let w = slice(dir.path(), "jvm");
    // slice_started is the slice's line 1: its evidence is lines 2 and 3.
    let a: dla_core::EvidenceId = "jvm/E1".parse().unwrap();
    let b: dla_core::EvidenceId = "jvm/E2".parse().unwrap();
    assert_eq!(w.evidence_order(&a).map(|k| k.at), Some(base + 2));
    assert_eq!(w.evidence_order(&b).map(|k| k.at), Some(base + 3));
    // The main line written after the slice started is not in the view at all.
    assert!(w.evidence_order(&eid(17)).is_none());
    let order = in_order(&w);
    assert_eq!(&order[order.len() - 2..], ["jvm/E1", "jvm/E2"]);
    assert_eq!(order[..16], ids(&w)[..16]);
}

#[test]
fn after_a_merge_a_main_record_written_later_is_newer_than_every_merged_one() {
    let dir = new_run(&["jvm", "rust"]);
    let mut j = slice(dir.path(), "jvm");
    j.add_evidence(probe()).unwrap();
    j.finish_slice().unwrap();
    let mut r = slice(dir.path(), "rust");
    r.add_evidence(probe()).unwrap();
    r.finish_slice().unwrap();
    merge(dir.path()).unwrap();
    let mut main = Ledger::load(dir.path()).unwrap();
    let newest = main.next_evidence_id();
    main.add_evidence(probe()).unwrap();
    // By ID the merged records sort after the plain ones, so the last of `evidence()` is the
    // older `rust/E1`; by time the last is the record the core wrote after the merge.
    assert_eq!(ids(&main).last().map(String::as_str), Some("rust/E1"));
    let order = in_order(&main);
    assert_eq!(order.last(), Some(&newest.to_string()));
    let at = |id: &str| order.iter().position(|x| x == id).unwrap();
    assert!(at("jvm/E1") < at("rust/E1") && at("rust/E1") < at(&newest.to_string()));
    assert!(
        main.evidence_order(&"jvm/E1".parse().unwrap()).unwrap()
            < main.evidence_order(&newest).unwrap()
    );
    // What the worker saw before the merge is the order the merge keeps: its own lines after
    // the main lines it started on.
    let seen = slice(dir.path(), "jvm");
    let rel = |l: &Ledger, id: &str| l.evidence_order(&id.parse().unwrap()).unwrap();
    assert!(rel(&seen, "jvm/E1") > rel(&seen, "E16"));
    assert!(rel(&main, "jvm/E1") > rel(&main, "E16"));
}

fn tick() {
    std::thread::sleep(std::time::Duration::from_millis(5));
}

#[test]
fn a_main_record_written_while_a_slice_is_open_is_newer_than_the_older_merged_line_after_it() {
    let dir = new_run(&["jvm"]);
    let mut main = Ledger::load(dir.path()).unwrap();
    let mut w = slice(dir.path(), "jvm");
    w.add_evidence(probe()).unwrap();
    tick();
    // The core writes while the slice is open: newer by time than `jvm/E1`.
    main.add_evidence(probe()).unwrap();
    tick();
    w.add_evidence(probe()).unwrap();
    w.finish_slice().unwrap();
    merge(dir.path()).unwrap();
    let merged = Ledger::load(dir.path()).unwrap();
    // The merge appends the slice after the main line, so the position of `jvm/E1` is later ...
    let key = |id: &str| merged.evidence_order(&id.parse().unwrap()).unwrap();
    assert!(key("jvm/E1").at > key("E17").at);
    // ... and the key still says it is older, and the worker's second record newer than both.
    assert!(key("jvm/E1") < key("E17"));
    assert!(key("E17") < key("jvm/E2"));
    let order = in_order(&merged);
    let at = |id: &str| order.iter().position(|x| x == id).unwrap();
    assert!(at("jvm/E1") < at("E17") && at("E17") < at("jvm/E2"));
}

#[test]
fn two_slices_merged_in_the_opposite_order_of_their_records_times_stay_in_time_order() {
    // Slices merge by worker name: `a` first. `b` recorded first, `a` after it.
    let dir = new_run(&["a", "b"]);
    let mut a = slice(dir.path(), "a");
    let mut b = slice(dir.path(), "b");
    b.add_evidence(probe()).unwrap();
    tick();
    a.add_evidence(probe()).unwrap();
    a.finish_slice().unwrap();
    b.finish_slice().unwrap();
    merge(dir.path()).unwrap();
    let merged = Ledger::load(dir.path()).unwrap();
    let key = |id: &str| merged.evidence_order(&id.parse().unwrap()).unwrap();
    assert!(
        key("b/E1").at > key("a/E1").at,
        "b's line is later in the file"
    );
    assert!(key("b/E1") < key("a/E1"));
    let order = in_order(&merged);
    assert_eq!(&order[order.len() - 2..], ["b/E1", "a/E1"]);
}
