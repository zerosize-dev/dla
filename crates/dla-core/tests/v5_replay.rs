//! A sample of the ledgers the M1 to M5 benchmark runs wrote (schema 1 to 5, core ledgers
//! and worker slices), kept under `tests/data/old-ledgers` after the raw benchmark runs were
//! deleted.
//! The schema 6 build must replay all of them without error and to the same state: the
//! digest of each replay (findings, evidence count, open items, merges) is compared with
//! `tests/data/v5_replay.txt`, computed once with the build before schema 6.
//!
//! `DLA_WRITE_V5_BASELINE=1 cargo test -p dla-core --test v5_replay` rewrites the file.

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use dla_core::Ledger;
use sha2::{Digest, Sha256};

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, out);
        } else if let Some(n) = p.file_name().and_then(|n| n.to_str())
            && n.starts_with("ledger")
            && n.ends_with(".jsonl")
        {
            out.push(p);
        }
    }
}

fn digest(l: &Ledger) -> String {
    let mut h = Sha256::new();
    for f in l.findings() {
        h.update(serde_json::to_string(f).unwrap());
        h.update(b"\n");
    }
    for e in l.evidence() {
        h.update(e.evidence_id.to_string());
        h.update(b"\n");
    }
    for o in l.open_items() {
        h.update(format!("{} {}\n", o.item.id, o.resolution.is_some()));
    }
    for f in l.findings() {
        if let Some(c) = l.merged_into(&f.id) {
            h.update(format!("{} -> {c}\n", f.id));
        }
        h.update(format!("{:?}\n", l.annotation(&f.id)));
    }
    hex::encode(h.finalize())[..16].to_owned()
}

fn replay(p: &Path) -> Result<Ledger, String> {
    let dir = p.parent().unwrap();
    let name = p.file_name().unwrap().to_str().unwrap();
    if name == "ledger.jsonl" {
        return Ledger::load(dir).map_err(|e| e.to_string());
    }
    let worker = name
        .strip_prefix("ledger.")
        .and_then(|n| n.strip_suffix(".jsonl"))
        .unwrap();
    let text = fs::read_to_string(p).unwrap();
    Ledger::load_slice_from(dir, worker, &text).map_err(|e| e.to_string())
}

#[test]
fn committed_benchmark_ledgers_replay_to_their_recorded_state() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/old-ledgers");
    let mut files = Vec::new();
    for set in fs::read_dir(&root).unwrap().flatten() {
        collect(&set.path().join("runs"), &mut files);
    }
    files.sort();
    assert!(!files.is_empty(), "no ledgers under tests/data/old-ledgers");
    let mut got = String::new();
    for f in &files {
        let rel = f
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let l = replay(f).unwrap_or_else(|e| panic!("{rel} no longer replays: {e}"));
        writeln!(
            got,
            "{rel} lines={} findings={} {}",
            l.lines().len(),
            l.findings().count(),
            digest(&l)
        )
        .unwrap();
    }
    let want_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/v5_replay.txt");
    if std::env::var_os("DLA_WRITE_V5_BASELINE").is_some() {
        fs::write(&want_path, &got).unwrap();
        return;
    }
    let want = fs::read_to_string(&want_path).unwrap();
    for (g, w) in got.lines().zip(want.lines()) {
        assert_eq!(g, w, "replay of a committed ledger changed");
    }
    assert_eq!(got.lines().count(), want.lines().count());
}
