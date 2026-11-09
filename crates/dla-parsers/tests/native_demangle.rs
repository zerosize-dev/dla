//! Demangling against symbols from a real Rust binary (built with both mangling schemes). The
//! expected text is `c++filt` / binutils output with crate hashes, the legacy `h<hash>` suffix,
//! const type suffixes and closure indexes 0 removed.

use std::{fs, path::PathBuf};

use dla_parsers::native::demangle::{demangle, demangle_or_raw, is_rust_symbol};

fn corpus(name: &str) -> Vec<(String, String)> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native")
        .join(name);
    fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{}: {e}", p.display()))
        .lines()
        .map(|l| {
            let (s, e) = l.split_once('\t').expect("tab separated");
            (s.to_owned(), e.to_owned())
        })
        .collect()
}

#[test]
fn legacy_symbols_of_a_real_binary() {
    let rows = corpus("demangle_legacy.tsv");
    assert!(rows.len() > 100);
    let bad: Vec<_> = rows
        .iter()
        .filter(|(s, e)| demangle(s).as_deref() != Some(e.as_str()))
        .collect();
    assert!(
        bad.is_empty(),
        "{} of {} differ, first: {:?}",
        bad.len(),
        rows.len(),
        bad.first()
    );
}

#[test]
fn v0_symbols_of_a_real_binary() {
    let rows = corpus("demangle_v0.tsv");
    assert!(rows.len() > 100);
    let bad: Vec<_> = rows
        .iter()
        .filter(|(s, e)| demangle(s).as_deref() != Some(e.as_str()))
        .collect();
    assert!(
        bad.is_empty(),
        "{} of {} differ, first: {:?}",
        bad.len(),
        rows.len(),
        bad.first()
    );
}

#[test]
fn symbols_from_heaptrack_and_lsan_output() {
    assert_eq!(
        demangle("_RNvCsinJrbhJGnG5_5rleak12open_session").as_deref(),
        Some("rleak::open_session")
    );
    assert_eq!(
        demangle("_RNvCsinJrbhJGnG5_5rleak4main").as_deref(),
        Some("rleak::main")
    );
}

#[test]
fn other_names_are_left_alone() {
    // C, C++ (Itanium), and truncated or broken Rust names.
    for raw in [
        "malloc",
        "main",
        "_ZN3foo3barEv",
        "_ZNSt6vectorIiSaIiEE9push_backERKi",
        "_ZN2dm4main17h3e04449de143555e",
        "_RNvCs",
        "_RINvC1a",
        "0x7ff22deb738e",
    ] {
        assert_eq!(demangle(raw), None, "{raw}");
        assert_eq!(demangle_or_raw(raw), raw);
        assert!(!is_rust_symbol(raw));
    }
}

#[test]
fn hostile_input_is_bounded() {
    // A long run of back references and deep nesting must fail, not hang or overflow.
    let deep = format!("_R{}C1a", "N".repeat(5000));
    assert_eq!(demangle(&deep), None);
    let refs = format!("_RNvC1a1b{}", "B_".repeat(2000));
    let _ = demangle(&refs);
    let wide = format!("_ZN{}E", "1a".repeat(100_000));
    let _ = demangle(&wide);
}
