//! Concurrent writers of the main ledger never assign the same `seq` or ID, and the
//! seal stays in step. Each writer holds its own handle, loaded before the others wrote.

mod common;

use std::sync::{Arc, Barrier};

use common::*;
use dla_core::{
    Ledger,
    evidence::{Authority, EvidenceKind},
    integrity,
};

const WRITERS: usize = 8;
const EACH: usize = 6;

fn hammer(dir: &std::path::Path) {
    let barrier = Arc::new(Barrier::new(WRITERS));
    let handles: Vec<_> = (0..WRITERS)
        .map(|_| {
            let dir = dir.to_path_buf();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                // Every handle is loaded before any writer appends: the stale view the
                // race came from.
                let mut l = Ledger::load(&dir).unwrap();
                barrier.wait();
                for _ in 0..EACH {
                    l.add_evidence(evidence(EvidenceKind::RunConfig, Authority::Static))
                        .unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

fn assert_gap_free(dir: &std::path::Path, before: usize) {
    let l = Ledger::load(dir).expect("the ledger replays");
    let seqs: Vec<u64> = l.lines().iter().map(|x| x.seq).collect();
    let want: Vec<u64> = (1..=(before + WRITERS * EACH) as u64).collect();
    assert_eq!(seqs, want, "seq runs 1, 2, 3 without gaps or repeats");
    let ids: std::collections::BTreeSet<String> = l
        .lines()
        .iter()
        .filter_map(|x| match &x.record {
            dla_core::Record::Evidence(e) => Some(e.evidence_id.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(ids.len(), before_evidence(&l), "evidence IDs are unique");
}

fn before_evidence(l: &Ledger) -> usize {
    l.lines()
        .iter()
        .filter(|x| matches!(x.record, dla_core::Record::Evidence(_)))
        .count()
}

#[test]
fn concurrent_evidence_writers_get_distinct_seq_and_ids() {
    let dir = new_run(&[]);
    let before = Ledger::load(dir.path()).unwrap().lines().len();
    hammer(dir.path());
    assert_gap_free(dir.path(), before);
}

#[test]
fn concurrent_writers_keep_a_sealed_ledger_sealed() {
    let dir = new_run(&[]);
    integrity::init_dir(dir.path()).unwrap();
    integrity::seal_all(dir.path()).unwrap();
    let before = Ledger::load(dir.path()).unwrap().lines().len();
    hammer(dir.path());
    assert_gap_free(dir.path(), before);
    assert!(
        integrity::verify(dir.path()).is_empty(),
        "{:?}",
        integrity::verify(dir.path())
    );
}

#[test]
fn mixed_writers_keep_findings_and_transitions_in_order() {
    let dir = new_run(&[]);
    let n = 6;
    let handles: Vec<_> = (0..n)
        .map(|_| {
            let d = dir.path().to_path_buf();
            std::thread::spawn(move || {
                let mut l = Ledger::load(&d).unwrap();
                l.open_item_add("check", None, "test").unwrap();
                l.add_evidence(evidence(EvidenceKind::RunConfig, Authority::Static))
                    .unwrap();
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    let l = Ledger::load(dir.path()).unwrap();
    let seqs: Vec<u64> = l.lines().iter().map(|x| x.seq).collect();
    assert_eq!(seqs, (1..=seqs.len() as u64).collect::<Vec<_>>());
}

#[test]
fn a_ledger_with_a_repeated_seq_names_the_repair() {
    let dir = new_run(&[]);
    let text = std::fs::read_to_string(dir.path().join("ledger.jsonl")).unwrap();
    let last = text.lines().last().unwrap().to_owned();
    std::fs::write(dir.path().join("ledger.jsonl"), format!("{text}{last}\n")).unwrap();
    let err = Ledger::load(dir.path()).unwrap_err().to_string();
    assert!(err.contains("two writers appended"), "{err}");
    assert!(err.contains("dla run reseal --reason"), "{err}");
}
