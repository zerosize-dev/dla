//! Every evidence kind an archetype may require has a producer.
//!
//! `PRODUCED_KINDS` (dla-core) lists the kinds some command or adapter writes. This test
//! greps the non-test sources of the crates that write evidence for `EvidenceKind::<Variant>`
//! and checks the list against what it finds, then checks that `data/archetypes.toml`
//! uses only listed kinds.
//!
//! The crates searched are `dla-cli`, `dla-backends` and `dla-workload`, plus `dla-probe`
//! (the `probe` kind) and `dla-census` (the `census-holder` kind), which write their own
//! evidence.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use dla_core::{
    EvidenceKind,
    archetypes::{self, PRODUCED_KINDS},
};

const CRATES: [&str; 5] = [
    "dla-cli",
    "dla-backends",
    "dla-workload",
    "dla-probe",
    "dla-census",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs")
            && p.file_name().is_some_and(|n| n != "tests.rs")
        {
            out.push(p);
        }
    }
}

/// Text of the non-test sources: files under `src/`, each cut at its first `#[cfg(test)]`.
fn non_test_sources() -> String {
    let mut out = String::new();
    for c in CRATES {
        let mut files = Vec::new();
        rust_files(&root().join("crates").join(c).join("src"), &mut files);
        for f in files {
            let text = fs::read_to_string(&f).unwrap();
            let end = text.find("#[cfg(test)]").unwrap_or(text.len());
            out.push_str(&text[..end]);
        }
    }
    out
}

/// `growth-series` -> `GrowthSeries`.
fn variant(kind: EvidenceKind) -> String {
    kind.as_str()
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn produced_in_sources(src: &str, kind: EvidenceKind) -> bool {
    src.contains(&format!("EvidenceKind::{}", variant(kind)))
}

#[test]
fn listed_kinds_have_a_producer_and_unlisted_kinds_have_none() {
    let src = non_test_sources();
    let listed: BTreeSet<EvidenceKind> = PRODUCED_KINDS.iter().copied().collect();
    for &k in EvidenceKind::ALL {
        let found = produced_in_sources(&src, k);
        if listed.contains(&k) {
            assert!(
                found,
                "{k} is in PRODUCED_KINDS but no non-test source writes it"
            );
        } else {
            assert!(
                !found,
                "{k} is written by a source but missing from PRODUCED_KINDS"
            );
        }
    }
}

#[test]
fn archetype_requires_use_only_listed_kinds() {
    let table = archetypes::load(&root().join("data"))
        .unwrap()
        .expect("data/archetypes.toml");
    let listed: BTreeSet<EvidenceKind> = PRODUCED_KINDS.iter().copied().collect();
    for e in table.entries() {
        for k in e.requires.iter().flatten() {
            assert!(listed.contains(k), "{} requires unproduced kind {k}", e.id);
        }
    }
}
