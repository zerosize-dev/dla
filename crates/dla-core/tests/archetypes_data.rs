//! Consistency of the shipped archetype data: `data/archetypes.toml`, the known-leak
//! table and its generated reference, the six `<lang>-archetypes.md` tables, and the
//! fixtures' answer keys.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use dla_core::{
    EvidenceKind,
    archetypes::{self, PRODUCED_KINDS, Table},
    detectors::ARCHETYPE_IDS,
    known_leaks::{KnownLeaksFile, REFERENCE_OPENING, render_reference},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn table() -> Table {
    archetypes::load(&root().join("data"))
        .expect("archetypes.toml loads")
        .expect("data/archetypes.toml exists")
}

fn is_leak_id(id: &str) -> bool {
    id.starts_with("L-")
}

#[test]
fn keys_equal_the_archetype_id_list() {
    let t = table();
    let mut want: Vec<&str> = ARCHETYPE_IDS.to_vec();
    want.sort_unstable();
    let got: Vec<&str> = t.ids().collect();
    assert_eq!(got, want);
}

#[test]
fn leak_entries_are_complete_and_look_alikes_have_no_holder() {
    let t = table();
    for e in t.entries() {
        assert!(!e.summary.trim().is_empty(), "{} has no summary", e.id);
        assert!(
            !e.summary.contains('\n'),
            "{} summary is not one line",
            e.id
        );
        assert!(!e.requires.is_empty(), "{} has no requires", e.id);
        assert!(
            e.requires.iter().all(|alt| !alt.is_empty()),
            "{} has an empty alternative",
            e.id
        );
        assert!(!e.regions.is_empty(), "{} has no regions", e.id);
        if is_leak_id(&e.id) {
            assert!(!e.holders.is_empty(), "{} has no holders", e.id);
            assert!(!e.repair.is_empty(), "{} has no repair", e.id);
            assert!(!e.disconfirm.is_empty(), "{} disconfirms nothing", e.id);
        } else {
            assert!(e.holders.is_empty(), "{} must have no holder", e.id);
            assert!(e.repair.is_empty(), "{} must have no repair", e.id);
            assert!(e.disconfirm.is_empty(), "{} must not disconfirm", e.id);
        }
    }
}

#[test]
fn disconfirm_ids_are_look_alikes() {
    let t = table();
    for e in t.entries() {
        for d in &e.disconfirm {
            assert!(d.starts_with("N-"), "{}: {d} is not an N-* id", e.id);
            assert!(t.get(d).is_some(), "{}: {d} is not in the table", e.id);
        }
    }
}

#[test]
fn requires_use_only_produced_kinds() {
    let produced: BTreeSet<EvidenceKind> = PRODUCED_KINDS.iter().copied().collect();
    assert_eq!(produced.len(), PRODUCED_KINDS.len(), "duplicate kind");
    for k in [
        EvidenceKind::BlockedStackProfile,
        EvidenceKind::UnreachableAtExit,
    ] {
        assert!(!produced.contains(&k), "{k} has no producer");
    }
    for e in table().entries() {
        for k in e.requires.iter().flatten() {
            assert!(
                produced.contains(k),
                "{} requires {k}, which nothing produces",
                e.id
            );
        }
    }
}

/// Text of the places series names are defined: the column catalog, the samplers, the
/// harness agents and the harness generators.
fn series_corpus() -> String {
    let mut out = String::new();
    let mut dirs = vec![
        root().join("crates/dla-core/src"),
        root().join("crates/dla-sample/src"),
        root().join("crates/dla-workload/src"),
        root().join("harness"),
    ];
    while let Some(d) = dirs.pop() {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                dirs.push(p);
            } else if let Ok(text) = fs::read_to_string(&p) {
                out.push_str(&text);
            }
        }
    }
    out
}

#[test]
fn signals_are_real_series_names() {
    let corpus = series_corpus();
    for e in table().entries() {
        for s in &e.signals {
            assert!(
                corpus.contains(&format!("\"{s}\""))
                    || (s.ends_with('.') && corpus.contains(&format!("\"{s}"))),
                "{}: signal {s:?} is not a series name in the samplers or harness agents",
                e.id
            );
        }
    }
}

fn fixtures() -> Vec<(String, toml::Table)> {
    let dir = root().join("bench/fixtures");
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).expect("bench/fixtures").flatten() {
        let p = entry.path().join("expected.toml");
        if let Ok(text) = fs::read_to_string(&p) {
            let name = entry.file_name().to_string_lossy().into_owned();
            out.push((name, text.parse::<toml::Table>().expect("expected.toml")));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn fixture_answer_keys_agree_with_the_table() {
    let t = table();
    let mut leaks = 0;
    let mut non_leaks = 0;
    for (name, key) in fixtures() {
        for l in key
            .get("leak")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let cause = l["cause"].as_str().unwrap_or("");
            if cause == "UNCLASSIFIED" {
                continue;
            }
            let e = t
                .get(cause)
                .unwrap_or_else(|| panic!("{name}: leak cause {cause} is not in the table"));
            assert!(is_leak_id(cause), "{name}: leak cause {cause} is not L-*");
            if let Some(h) = l.get("holder").and_then(|v| v.as_str()) {
                assert!(
                    e.holders.iter().any(|x| x.as_str() == h),
                    "{name}: holder {h} is not in archetypes[{cause}].holders"
                );
            }
            if let Some(r) = l.get("region").and_then(|v| v.as_str()) {
                assert!(
                    e.regions.iter().any(|x| x.as_str() == r),
                    "{name}: region {r} is not in archetypes[{cause}].regions"
                );
            }
            leaks += 1;
        }
        for n in key
            .get("non_leak")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let cause = n["cause"].as_str().unwrap_or("");
            if cause == "UNCLASSIFIED" {
                continue;
            }
            assert!(
                cause.starts_with("N-") && t.get(cause).is_some(),
                "{name}: non-leak cause {cause} is not in the table"
            );
            non_leaks += 1;
        }
    }
    assert!(
        leaks >= 10 && non_leaks >= 10,
        "{leaks} leaks, {non_leaks} non-leaks"
    );
}

// ---- known leaks ------------------------------------------------------------------------

fn known_leaks() -> KnownLeaksFile {
    let text = fs::read_to_string(root().join("data/known-leaks.toml")).unwrap();
    KnownLeaksFile::parse(&text, "data/known-leaks.toml").unwrap()
}

#[test]
fn known_leak_archetypes_are_table_ids() {
    let t = table();
    for e in &known_leaks().leak {
        assert!(
            t.get(&e.archetype).is_some_and(|a| is_leak_id(&a.id)),
            "{}: archetype {} is not a leak archetype in the table",
            e.id,
            e.archetype
        );
    }
}

#[test]
fn known_leaks_reference_is_generated_from_the_table() {
    let path = root().join("skills/detect-leak-anywhere/references/known-leaks.md");
    let want = render_reference(&known_leaks());
    assert!(want.contains(REFERENCE_OPENING));
    if std::env::var("DLA_REGENERATE").is_ok_and(|v| v == "1") {
        fs::write(&path, &want).unwrap();
    }
    let have = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        have == want,
        "{} is out of date with data/known-leaks.toml. Regenerate it with: \
         DLA_REGENERATE=1 cargo test -p dla-core --test archetypes_data known_leaks_reference",
        path.display()
    );
}

// ---- the six prose tables ----------------------------------------------------------------

const OPENING: &str = "A row match opens a hypothesis and carries no verdict.";
const HEADER: [&str; 5] = [
    "Row",
    "Find candidates",
    "Confirm",
    "Fingerprint",
    "Rule out",
];

/// The rows shipped per runtime: exactly the rows with a decoy/control pair (decision D1).
const ROWS: [(&str, &[&str]); 6] = [
    ("jvm", &["L-COLLECT", "L-TLS", "L-UNLOAD"]),
    ("go", &["L-WORKER"]),
    ("node", &["L-REGISTER", "L-CAPTURE"]),
    ("python", &["L-COLLECT"]),
    ("rust", &["L-CYCLE", "L-OWNERSHIP"]),
    ("native", &["L-HANDLE"]),
];

fn cells(line: &str) -> Vec<String> {
    let t = line.trim().trim_start_matches('|').trim_end_matches('|');
    t.split('|').map(|c| c.trim().to_owned()).collect()
}

/// Source tags such as `[J1]` in `text`.
fn tags(text: &str) -> BTreeSet<String> {
    let b = text.as_bytes();
    let mut out = BTreeSet::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'[' {
            let mut j = i + 1;
            if j < b.len() && b[j].is_ascii_uppercase() {
                j += 1;
                let d = j;
                while j < b.len() && b[j].is_ascii_digit() {
                    j += 1;
                }
                if j > d && j < b.len() && b[j] == b']' {
                    out.insert(text[i + 1..j].to_owned());
                }
            }
        }
        i += 1;
    }
    out
}

#[test]
fn archetype_reference_files_follow_the_rules() {
    let t = table();
    let fixture_names: Vec<String> = fixtures().into_iter().map(|(n, _)| n).collect();
    for (lang, want_rows) in ROWS {
        let path = root().join(format!(
            "skills/detect-leak-anywhere/references/{lang}-archetypes.md"
        ));
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{lang}: {e}"));
        let mut lines = text.lines();
        assert!(
            lines.next().is_some_and(|l| l.starts_with("# ")),
            "{lang}: title"
        );
        let first = lines.by_ref().find(|l| !l.trim().is_empty());
        assert_eq!(first, Some(OPENING), "{lang}: opening sentence");

        let (body, sources) = text
            .split_once("\n## Sources")
            .unwrap_or_else(|| panic!("{lang}: no Sources section"));
        let table_lines: Vec<&str> = body.lines().filter(|l| l.starts_with('|')).collect();
        assert!(table_lines.len() >= 3, "{lang}: table");
        assert_eq!(cells(table_lines[0]), HEADER, "{lang}: header");
        assert!(
            cells(table_lines[1])
                .iter()
                .all(|c| c.chars().all(|ch| ch == '-' || ch == ':')),
            "{lang}: separator"
        );
        let rows: Vec<Vec<String>> = table_lines[2..].iter().map(|l| cells(l)).collect();
        assert!(rows.len() <= 12, "{lang}: more than 12 rows");
        let ids: Vec<String> = rows
            .iter()
            .map(|r| r[0].trim_matches('`').to_owned())
            .collect();
        let mut expect: Vec<String> = want_rows.iter().map(|s| (*s).to_owned()).collect();
        expect.push("UNCLASSIFIED".to_owned());
        assert_eq!(ids, expect, "{lang}: rows");
        for r in &rows {
            assert_eq!(r.len(), 5, "{lang}: {} has {} cells", r[0], r.len());
            assert!(
                r.iter().all(|c| !c.is_empty()),
                "{lang}: empty cell in {}",
                r[0]
            );
        }
        for id in want_rows {
            assert!(t.get(id).is_some(), "{lang}: {id} not in the table");
        }
        // Each rule-out cell names an N-* id.
        for r in &rows {
            assert!(
                r[4].contains("N-"),
                "{lang}: {} rule-out names no N-* id",
                r[0]
            );
        }

        // Sources: every tag used in the table is resolved, every source is used.
        let used = tags(body);
        let listed: BTreeSet<String> = sources
            .lines()
            .filter_map(|l| l.trim_start_matches("- ").strip_prefix('['))
            .filter_map(|l| l.split_once(']'))
            .map(|(t, _)| t.to_owned())
            .collect();
        assert_eq!(used, listed, "{lang}: tags used and Sources differ");
        for l in sources
            .lines()
            .filter(|l| l.trim_start().starts_with("- ["))
        {
            assert!(l.contains("https://"), "{lang}: source without URL: {l}");
        }

        // The benchmark model reads these files: never name a fixture.
        for n in &fixture_names {
            assert!(!text.contains(n.as_str()), "{lang}: names fixture {n}");
        }
    }
    // Every runtime file is covered by the list above.
    let count = fs::read_dir(root().join("skills/detect-leak-anywhere/references"))
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with("-archetypes.md"))
        .count();
    assert_eq!(count, ROWS.len());
}

#[test]
fn runtime_references_point_to_their_archetype_file() {
    for (lang, _) in ROWS {
        let text = fs::read_to_string(
            root().join(format!("skills/detect-leak-anywhere/references/{lang}.md")),
        )
        .unwrap();
        assert!(
            text.contains(&format!("references/{lang}-archetypes.md"))
                && text.contains("dla gate check --stage classify"),
            "{lang}.md does not point to {lang}-archetypes.md"
        );
    }
}

fn marked_block(text: &str, name: &str) -> Vec<String> {
    let begin = format!("<!-- {name}:begin -->");
    let end = format!("<!-- {name}:end -->");
    let mut inside = false;
    let mut lines = Vec::new();
    let (mut begins, mut ends) = (0, 0);
    for l in text.lines() {
        if l.trim() == end {
            inside = false;
            ends += 1;
        } else if inside {
            lines.push(l.to_owned());
        } else if l.trim() == begin {
            inside = true;
            begins += 1;
        }
    }
    assert_eq!((begins, ends), (1, 1), "one {name} block in the file");
    lines
}

#[test]
fn the_verifiers_id_list_equals_the_table() {
    let text = fs::read_to_string(root().join("agents/verifier.md")).unwrap();
    let got = marked_block(&text, "archetypes");
    // The block opens with its own one-line introduction and a blank line, so that a variant
    // without archetypes loses the introduction with the list.
    assert!(got[0].starts_with("IDs ("), "{got:?}");
    assert!(got[1].is_empty());
    let got = got[2..].to_vec();
    let want: Vec<String> = table()
        .entries()
        .map(|e| format!("- `{}`: {}", e.id, e.summary))
        .collect();
    assert_eq!(want.len(), 17);
    assert_eq!(
        got, want,
        "regenerate the block between the archetypes markers"
    );
}

#[test]
fn the_verifier_stays_blind() {
    let text = fs::read_to_string(root().join("agents/verifier.md")).unwrap();
    // Outside the marked block, the file forbids the files that would give the answer away.
    for must in [
        "ledger*.jsonl",
        "state.json",
        "report.md",
        "known-leaks.md",
        "*-archetypes.md",
        "archetypes.toml",
        "--label",
    ] {
        assert!(text.contains(must), "verifier.md lacks {must}");
    }
    let before = text.find("<!-- archetypes:begin -->").unwrap();
    let procedure = text.find("Procedure:").unwrap();
    let id_step = text.find("Only then choose an ID").unwrap();
    let words_first = text.find("before any ID").unwrap();
    assert!(procedure < words_first && words_first < id_step && id_step < before);
    assert!(
        !text.contains("site:"),
        "the verifier is never given a site"
    );
}
