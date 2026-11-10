//! The benchmark's cause lists, the archetype IDs and `data/archetypes.toml` must agree:
//! the leak causes are the `L-*` keys plus `UNCLASSIFIED`, the non-leak causes the `N-*`
//! keys plus `UNCLASSIFIED`.

use std::{collections::BTreeSet, path::Path};

use dla_bench::model::{LEAK_CAUSES, NON_LEAK_CAUSES};

fn keys() -> BTreeSet<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/archetypes.toml");
    let text = std::fs::read_to_string(&path).expect("data/archetypes.toml");
    let table: toml::Table = text.parse().expect("archetypes.toml parses");
    table.keys().cloned().collect()
}

fn with_unclassified(prefix: &str, all: &BTreeSet<String>) -> BTreeSet<String> {
    all.iter()
        .filter(|k| k.starts_with(prefix))
        .cloned()
        .chain(["UNCLASSIFIED".to_owned()])
        .collect()
}

#[test]
fn leak_causes_are_the_l_keys_plus_unclassified() {
    let want = with_unclassified("L-", &keys());
    let got: BTreeSet<String> = LEAK_CAUSES.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(got, want);
}

#[test]
fn non_leak_causes_are_the_n_keys_plus_unclassified() {
    let want = with_unclassified("N-", &keys());
    let got: BTreeSet<String> = NON_LEAK_CAUSES.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(got, want);
}

#[test]
fn every_key_is_a_leak_or_a_non_leak_id() {
    for k in keys() {
        assert!(k.starts_with("L-") || k.starts_with("N-"), "{k}");
    }
}

/// The first-column IDs of the rows of every `references/<lang>-archetypes.md`.
fn shipped_rows() -> BTreeSet<String> {
    let refs =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills/detect-leak-anywhere/references");
    let mut rows = BTreeSet::new();
    let mut files = 0;
    for e in std::fs::read_dir(&refs).expect("references/").flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.ends_with("-archetypes.md") {
            continue;
        }
        files += 1;
        for line in std::fs::read_to_string(e.path()).unwrap().lines() {
            let first = line
                .strip_prefix('|')
                .and_then(|l| l.split('|').next())
                .map(str::trim)
                .unwrap_or("");
            if first.starts_with("L-") || first.starts_with("N-") {
                rows.insert(first.to_owned());
            }
        }
    }
    assert_eq!(files, 6, "one archetypes file per runtime family");
    rows
}

#[test]
fn the_policy_labels_exactly_the_rows_of_the_shipped_prose_files() {
    use dla_bench::dynamic_cmd::policy::SHIPPED_ROWS;
    let want = shipped_rows();
    let got: BTreeSet<String> = SHIPPED_ROWS.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(
        got, want,
        "SHIPPED_ROWS must equal the rows of the *-archetypes.md files"
    );
    let keys = keys();
    for r in &got {
        assert!(keys.contains(r), "{r} is not in data/archetypes.toml");
    }
}
