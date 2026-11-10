//! A `dla` write killed between the write and its seal is not an edit: the next `dla`
//! write of the file, or `dla ledger merge`, settles it. An edit made by hand is not.

mod common;

use std::{fs, io::Write, path::Path};

use common::*;
use dla_core::{
    RunState,
    finding::Holder,
    integrity::{self, Guard},
    merge::merge,
    slice::slice_path,
};

/// A sealed run with a registered worker `jvm` and one candidate in its slice.
fn sealed_run() -> tempfile::TempDir {
    let dir = new_run(&["jvm"]);
    integrity::init_dir(dir.path()).unwrap();
    integrity::seal_all(dir.path()).unwrap();
    let mut j = slice(dir.path(), "jvm");
    candidate_in_slice(&mut j, "a.go:1", Holder::StaticGlobal);
    assert!(integrity::verify(dir.path()).is_empty());
    dir
}

/// What a kill leaves: the write is on disk, its seal is not.
fn killed_append(dir: &Path, file: &str, bytes: &str) {
    let guard = Guard::new(dir, file);
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(dir.join(file))
        .unwrap();
    f.write_all(bytes.as_bytes()).unwrap();
    drop(f);
    drop(guard);
}

#[test]
fn a_torn_slice_settles_when_the_worker_writes_again() {
    let dir = sealed_run();
    killed_append(
        dir.path(),
        "ledger.jvm.jsonl",
        "{\"schema_version\":3,\"seq\":9,",
    );
    assert_eq!(integrity::verify(dir.path()).len(), 1);
    // The worker's next write cuts the fragment and seals what it wrote.
    let mut j = slice(dir.path(), "jvm");
    j.open_item_add("check", None, "test").unwrap();
    assert!(
        integrity::verify(dir.path()).is_empty(),
        "{:?}",
        integrity::verify(dir.path())
    );
}

#[test]
fn a_torn_slice_settles_when_the_core_merges() {
    let dir = sealed_run();
    killed_append(
        dir.path(),
        "ledger.jvm.jsonl",
        "{\"schema_version\":3,\"seq\":9,",
    );
    let tampered = integrity::verify(dir.path());
    assert_eq!(tampered.len(), 1);
    // The gate says what clears it, and it is not a restore from a copy.
    let view = dla_core::RunView::load(dir.path()).unwrap();
    let st = dla_core::status(&view);
    let item = st
        .blocking
        .iter()
        .find(|b| b.id == "ledger.jvm.jsonl")
        .expect("the mismatch is listed");
    assert!(item.fix.contains("dla ledger merge"), "{}", item.fix);
    merge(dir.path()).unwrap();
    assert!(
        integrity::verify(dir.path()).is_empty(),
        "{:?}",
        integrity::verify(dir.path())
    );
    // The torn line was seen and dropped by the merge: the gate does not list it again.
    let view = dla_core::RunView::load(dir.path()).unwrap();
    let missing = dla_core::gate::missing(&view, dla_core::gate::GateTarget::Final, None);
    assert!(
        !missing.iter().any(|m| m.description.contains("torn line")),
        "{missing:?}"
    );
    assert!(view.tamper.is_empty());
}

#[test]
fn a_killed_whole_file_write_settles_on_the_next_write() {
    let dir = sealed_run();
    // state.json was replaced and the process died before the seal.
    let mut state = RunState::load(dir.path()).unwrap();
    state.incomplete_reason = Some("x".into());
    let text = dla_core::fsutil::to_pretty_json(&state);
    let guard = Guard::new(dir.path(), "state.json");
    guard.expect(text.as_bytes());
    dla_core::fsutil::write_atomic(&dir.path().join("state.json"), text.as_bytes()).unwrap();
    drop(guard);
    assert_eq!(integrity::verify(dir.path()).len(), 1);
    RunState::update(dir.path(), |_| Ok(())).unwrap();
    assert!(integrity::verify(dir.path()).is_empty());
}

#[test]
fn an_edit_by_hand_stays_a_mismatch_through_later_writes_and_merges() {
    let dir = sealed_run();
    // Appended with no write of dla's in flight: no marker, so nothing settles it.
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(slice_path(dir.path(), "jvm"))
        .unwrap();
    f.write_all(b"{\"torn\":").unwrap();
    drop(f);
    assert_eq!(integrity::verify(dir.path()).len(), 1);
    merge(dir.path()).unwrap();
    assert_eq!(integrity::verify(dir.path()).len(), 1);
    let mut j = slice(dir.path(), "jvm");
    j.open_item_add("check", None, "test").unwrap();
    assert_eq!(integrity::verify(dir.path()).len(), 1);
    // An edit of a file a killed write also touched is not forgiven either.
    killed_append(dir.path(), "ledger.jsonl", "");
    fs::write(dir.path().join("coverage.json"), "{}").unwrap();
    assert!(
        integrity::verify(dir.path())
            .iter()
            .any(|t| t.file == "coverage.json")
    );
}
