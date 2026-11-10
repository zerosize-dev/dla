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
