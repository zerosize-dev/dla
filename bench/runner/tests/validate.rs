//! `dla-bench validate` against synthetic fixtures.

use std::{fs, path::Path};

use dla_bench::validate::validate_dir;

fn write(p: &Path, text: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn filler(prefix: &str, lines: usize) -> String {
    (0..lines)
        .map(|i| format!("{prefix}_{i} = {i}\n"))
        .collect()
}

/// A valid fixture: 3 python files of 60 lines.
fn make(root: &Path, id: &str, expected: &str) {
    let d = root.join(id);
    write(&d.join("README.md"), "readme\n");
    for f in ["a", "b", "c"] {
        write(&d.join(format!("project/src/{f}.py")), &filler(f, 60));
    }
    write(&d.join("expected.toml"), expected);
}

fn trap(id: &str, extra: &str) -> String {
    format!(
        r#"schema_version = 1
id = "{id}"
runtime = "python"
category = "archetype-leak"
dynamic_confirmation = true
[task]
prompt = "p"
[[leak]]
id = "L1"
site = "src/a.py:10"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
{extra}
"#
    )
}

fn problems(root: &Path) -> Vec<String> {
    validate_dir(root, &root.join("no-schemas")).unwrap().1
}

fn has(p: &[String], needle: &str) -> bool {
    p.iter().any(|l| l.contains(needle))
}

#[test]
fn empty_and_missing_directories_are_ok() {
    let t = tempfile::tempdir().unwrap();
    assert_eq!(validate_dir(t.path(), t.path()).unwrap(), (0, vec![]));
    let missing = t.path().join("nope");
    assert_eq!(validate_dir(&missing, &missing).unwrap(), (0, vec![]));
}

#[test]
fn a_valid_fixture_passes() {
    let t = tempfile::tempdir().unwrap();
    make(
        t.path(),
        "ok",
        &trap("ok", "lines = [9, 11]\nalt_sites = [\"src/b.py:3-5\"]"),
    );
    assert_eq!(problems(t.path()), Vec::<String>::new());
}

#[test]
fn schema_and_typed_parse_errors() {
    let t = tempfile::tempdir().unwrap();
    make(t.path(), "bad", &trap("bad", "bogus = 1"));
    let p = problems(t.path());
    assert!(has(&p, "bad: expected.toml schema"), "{p:?}");
    make(
        t.path(),
        "enum",
        &trap("enum", "").replace("static/global", "somewhere"),
    );
    assert!(has(&problems(t.path()), "enum: expected.toml schema"));
    make(
        t.path(),
        "cause",
        &trap("cause", "").replace("L-COLLECT", "lower"),
    );
    assert!(has(&problems(t.path()), "cause: expected.toml schema"));
    make(t.path(), "syntax", "this is = = not toml");
    assert!(has(&problems(t.path()), "syntax: expected.toml"));
}

#[test]
fn id_must_equal_the_directory_name() {
    let t = tempfile::tempdir().unwrap();
    make(t.path(), "dir", &trap("other", ""));
    assert!(has(&problems(t.path()), "dir: id \"other\" must equal"));
}

#[test]
fn site_rules() {
    let t = tempfile::tempdir().unwrap();
    make(
        t.path(),
        "nofile",
        &trap("nofile", "").replace("src/a.py:10", "src/zz.py:10"),
    );
    assert!(has(&problems(t.path()), "no such file in project/"));
    make(
        t.path(),
        "past",
        &trap("past", "").replace("src/a.py:10", "src/a.py:61"),
    );
    assert!(has(&problems(t.path()), "past: leak L1: site src/a.py:61"));
    make(t.path(), "range", &trap("range", "lines = [11, 12]"));
    assert!(has(&problems(t.path()), "must contain the site line 10"));
    make(
        t.path(),
        "alt",
        &trap("alt", "alt_sites = [\"src/b.py:99\", \"nonsense\"]"),
    );
    let p = problems(t.path());
    assert!(has(&p, "alt: leak L1: alt_site src/b.py:99"), "{p:?}");
    assert!(has(&p, "alt_site \"nonsense\" is not path:line"), "{p:?}");
    make(
        t.path(),
        "abs",
        &trap("abs", "").replace("src/a.py:10", "../x.py:10"),
    );
    assert!(has(&problems(t.path()), "project-relative"));
}

#[test]
fn shares_must_sum_to_one() {
    let t = tempfile::tempdir().unwrap();
    let two = |s1: &str, s2: &str| {
        format!(
            "{}\n[[leak]]\nid = \"L2\"\nsite = \"src/b.py:5\"\nholder = \"static/global\"\nregion = \"heap\"\ncause = \"UNCLASSIFIED\"\nshare = {s2}\n",
            trap("two", &format!("share = {s1}")).replace("archetype-leak", "multi-leak")
        )
    };
    make(t.path(), "two", &two("0.7", "0.3"));
    assert_eq!(problems(t.path()), Vec::<String>::new());
    make(t.path(), "two", &two("0.7", "0.4"));
    assert!(has(&problems(t.path()), "shares sum to"));
}

#[test]
fn pairs_are_reciprocal_with_identical_prompts() {
    let t = tempfile::tempdir().unwrap();
    let decoy = |id: &str, pair: &str, prompt: &str| {
        format!(
            "schema_version = 1\nid = \"{id}\"\nruntime = \"python\"\ncategory = \"decoy-control\"\npair = \"{pair}\"\ndynamic_confirmation = true\n[task]\nprompt = \"{prompt}\"\n[[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n"
        )
    };
    make(
        t.path(),
        "a",
        &trap("a", "").replace("archetype-leak\"\n", "archetype-leak\"\npair = \"b\"\n"),
    );
    make(t.path(), "b", &decoy("b", "a", "p"));
    assert_eq!(problems(t.path()), Vec::<String>::new());
    make(t.path(), "b", &decoy("b", "a", "different"));
    assert!(has(
        &problems(t.path()),
        "a: pair b has a different task.prompt"
    ));
    make(t.path(), "b", &decoy("b", "c", "p"));
    assert!(has(&problems(t.path()), "a: pair b does not name a back"));
    fs::remove_dir_all(t.path().join("b")).unwrap();
    assert!(has(&problems(t.path()), "a: pair b:"));
}

#[test]
fn category_rules() {
    let t = tempfile::tempdir().unwrap();
    let base = |cat: &str, dynamic: &str, body: &str| {
        format!(
            "schema_version = 1\nid = \"c\"\nruntime = \"go\"\ncategory = \"{cat}\"\ndynamic_confirmation = {dynamic}\n[task]\nprompt = \"p\"\n{body}"
        )
    };
    let leak = |conf: &str| {
        format!(
            "[[leak]]\nid = \"L1\"\nsite = \"src/a.py:10\"\nholder = \"static/global\"\nregion = \"heap\"\ncause = \"L-COLLECT\"\nconfirmable = {conf}\n"
        )
    };
    let nl = "[[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n";
    let check = |cat: &str, dynamic: &str, body: &str, needle: Option<&str>| {
        make(t.path(), "c", &base(cat, dynamic, body));
        let p = problems(t.path());
        match needle {
            Some(n) => assert!(has(&p, n), "{cat}: wanted {n:?} in {p:?}"),
            None => assert!(p.is_empty(), "{cat}: {p:?}"),
        }
    };
    check(
        "archetype-leak",
        "true",
        "",
        Some("needs at least one [[leak]]"),
    );
    check(
        "archetype-free",
        "true",
        "",
        Some("needs at least one [[leak]]"),
    );
    check(
        "multi-leak",
        "true",
        &leak("true"),
        Some("needs at least two"),
    );
    check(
        "decoy-control",
        "true",
        &leak("true"),
        Some("must have no [[leak]]"),
    );
    check(
        "decoy-control",
        "true",
        "",
        Some("needs at least one [[non_leak]]"),
    );
    check("non-leak", "true", nl, None);
    check(
        "impossible",
        "true",
        &leak("false"),
        Some("dynamic_confirmation = false"),
    );
    check(
        "impossible",
        "false",
        &leak("true"),
        Some("must have confirmable = false"),
    );
    check("impossible", "false", &leak("false"), None);
}

#[test]
fn hint_words_with_word_boundaries() {
    let t = tempfile::tempdir().unwrap();
    make(t.path(), "h", &trap("h", ""));
    let words = [
        "leak",
        "Leaks",
        "leaking",
        "TODO",
        "fixme",
        "fix",
        "bug",
        "unbounded",
        "bounded",
        "never freed",
        "Never Removed",
        "never cleared",
        "never closed",
        "never stopped",
        "never released",
        "my_leak_count",
    ];
    for w in words {
        write(
            &t.path().join("h/project/src/a.py"),
            &format!("{}x = 1  # {w}\n", filler("a", 59)),
        );
        let p = problems(t.path());
        assert!(has(&p, "h: project/src/a.py:60: hint word"), "{w}: {p:?}");
    }
    // near misses that must pass
    for ok in [
        "prefix",
        "fixture",
        "debug",
        "debugger",
        "boundary",
        "bugle",
        "never fixed",
        "todos_done",
    ] {
        write(
            &t.path().join("h/project/src/a.py"),
            &format!("{}x = 1  # {ok}\n", filler("a", 59)),
        );
        assert!(!has(&problems(t.path()), "hint word"), "{ok}");
    }
    // file names count
    write(
        &t.path().join("h/project/src/leak_test.py"),
        &filler("d", 10),
    );
    assert!(has(&problems(t.path()), "file name contains hint word"));
}

#[test]
fn a_project_file_that_carries_the_fixture_id_is_refused() {
    let t = tempfile::tempdir().unwrap();
    make(t.path(), "py-quota-cache", &trap("py-quota-cache", ""));
    assert_eq!(problems(t.path()), Vec::<String>::new());
    write(
        &t.path().join("py-quota-cache/project/src/a.py"),
        &format!("{}# part of py-quota-cache\n", filler("a", 60)),
    );
    let p = problems(t.path());
    assert!(
        has(&p, "src/a.py:61: the line contains the fixture id"),
        "{p:?}"
    );
    write(
        &t.path().join("py-quota-cache/project/src/a.py"),
        &filler("a", 60),
    );
    write(
        &t.path().join("py-quota-cache/project/py-quota-cache.txt"),
        "x\n",
    );
    let p = problems(t.path());
    assert!(has(&p, "the path contains the fixture id"), "{p:?}");
}

#[test]
fn project_shape_and_build_outputs() {
    let t = tempfile::tempdir().unwrap();
    make(t.path(), "s", &trap("s", ""));
    write(&t.path().join("s/project/target/debug/x"), "bin");
    let p = problems(t.path());
    assert!(has(&p, "build output must not be committed"), "{p:?}");
    fs::remove_dir_all(t.path().join("s/project/target")).unwrap();
    write(&t.path().join("s/project/Main.class"), "bin");
    assert!(has(&problems(t.path()), "Main.class: build output"));
    fs::remove_file(t.path().join("s/project/Main.class")).unwrap();
    fs::remove_file(t.path().join("s/project/src/c.py")).unwrap();
    let p = problems(t.path());
    assert!(has(&p, "2 source files"), "{p:?}");
    assert!(has(&p, "120 source lines"), "{p:?}");
    fs::remove_file(t.path().join("s/README.md")).unwrap();
    assert!(has(&problems(t.path()), "README.md is missing"));
}

fn non_leak(id: &str, cause: &str) -> String {
    format!(
        r#"schema_version = 1
id = "{id}"
runtime = "python"
category = "non-leak"
dynamic_confirmation = true
[task]
prompt = "p"
[[non_leak]]
id = "N1"
cause = "{cause}"
discriminator = "d"
"#
    )
}

#[test]
fn causes_come_from_the_archetype_vocabulary() {
    for ok in [
        "L-HANDLE",
        "L-COLLECT",
        "L-REGISTER",
        "L-WORKER",
        "L-TLS",
        "L-STALE",
        "L-CAPTURE",
        "L-CYCLE",
        "L-OWNERSHIP",
        "L-UNLOAD",
        "UNCLASSIFIED",
    ] {
        let t = tempfile::tempdir().unwrap();
        make(t.path(), "x", &trap("x", "").replace("L-COLLECT", ok));
        assert_eq!(problems(t.path()), Vec::<String>::new(), "{ok}");
    }
    for ok in [
        "N-ALLOC",
        "N-HEADROOM",
        "N-WARM",
        "N-HWM",
        "N-SCALE",
        "N-INTENT",
        "N-OBSERVER",
        "UNCLASSIFIED",
    ] {
        let t = tempfile::tempdir().unwrap();
        make(t.path(), "x", &non_leak("x", ok));
        assert_eq!(problems(t.path()), Vec::<String>::new(), "{ok}");
    }
    // typos, invented ids and the wrong family are all schema errors
    for bad in ["L-COLLCT", "L-FOO", "N-WARM", "N-FOO", "l-collect", ""] {
        let t = tempfile::tempdir().unwrap();
        make(t.path(), "x", &trap("x", "").replace("L-COLLECT", bad));
        let p = problems(t.path());
        assert!(
            has(&p, "x: expected.toml schema") && has(&p, "cause"),
            "leak {bad:?}: {p:?}"
        );
    }
    for bad in ["N-WARMUP", "L-COLLECT", "N-", "n-warm"] {
        let t = tempfile::tempdir().unwrap();
        make(t.path(), "x", &non_leak("x", bad));
        let p = problems(t.path());
        assert!(
            has(&p, "x: expected.toml schema") && has(&p, "cause"),
            "non_leak {bad:?}: {p:?}"
        );
    }
}

#[test]
fn schema_enums_match_the_cause_constants() {
    use dla_bench::model::{LEAK_CAUSES, NON_LEAK_CAUSES};
    let schema_text = dla_bench::schema_cmd::expected_schema();
    let schema: serde_json::Value = serde_json::from_str(&schema_text).unwrap();
    let leak_enum = &schema["$defs"]["Leak"]["properties"]["cause"]["enum"];
    let non_enum = &schema["$defs"]["NonLeak"]["properties"]["cause"]["enum"];
    assert_eq!(leak_enum, &serde_json::json!(LEAK_CAUSES));
    assert_eq!(non_enum, &serde_json::json!(NON_LEAK_CAUSES));
}

// ---- entry paths, owners, multi-runtime

/// A gzip file whose single stored block holds `data` (the checksum is not checked).
fn gz(data: &[u8]) -> Vec<u8> {
    let mut g = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3, 1];
    let len = data.len() as u16;
    g.extend_from_slice(&len.to_le_bytes());
    g.extend_from_slice(&(!len).to_le_bytes());
    g.extend_from_slice(data);
    g.extend_from_slice(&[0; 8]);
    g
}

/// The start of a heap dump file: the header text, the id size and the timestamp.
fn hprof(timestamp_ms: u64) -> Vec<u8> {
    let mut d = b"JAVA PROFILE 1.0.2\0".to_vec();
    d.extend_from_slice(&8u32.to_be_bytes());
    d.extend_from_slice(&timestamp_ms.to_be_bytes());
    d.extend_from_slice(&[0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    d
}

fn write_bytes(p: &Path, data: &[u8]) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, data).unwrap();
}

const ARTIFACT_TOML: &str = r#"schema_version = 1
id = "ID"
runtime = "jvm"
entry = "artifact"
category = "archetype-leak"
dynamic_confirmation = true
[task]
prompt = "p"
[[leak]]
id = "L1"
owner = "quotes.RfqDesk.OPEN"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
"#;

/// An artifact fixture with the given dump timestamps and no source.
fn make_artifact(root: &Path, id: &str, stamps: &[u64]) {
    let d = root.join(id);
    write(&d.join("README.md"), "readme\n");
    write(&d.join("expected.toml"), &ARTIFACT_TOML.replace("ID", id));
    for (i, ts) in stamps.iter().enumerate() {
        write_bytes(
            &d.join(format!("project/dumps/heap-{i}.hprof.gz")),
            &gz(&hprof(*ts)),
        );
    }
}

#[test]
fn an_artifact_fixture_skips_the_source_rules_and_checks_its_dumps() {
    let t = tempfile::tempdir().unwrap();
    make_artifact(t.path(), "art", &[1_000, 2_000, 3_000]);
    assert_eq!(problems(t.path()), Vec::<String>::new());
}

#[test]
fn artifact_dump_checks() {
    let t = tempfile::tempdir().unwrap();
    make_artifact(t.path(), "same", &[5, 5]);
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "same: project/dumps/heap-1.hprof.gz: header timestamp 5 equals"
        ),
        "{p:?}"
    );

    make_artifact(t.path(), "none", &[]);
    write(
        &t.path().join("none/project/dumps/notes.txt"),
        "capture notes\n",
    );
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "none: artifact fixture: project/dumps/ needs at least one"
        ),
        "{p:?}"
    );
    assert!(has(&p, "dumps are stored as *.hprof.gz"), "{p:?}");

    make_artifact(t.path(), "plain", &[1]);
    write_bytes(
        &t.path().join("plain/project/dumps/heap-9.hprof.gz"),
        &hprof(9),
    );
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "plain: project/dumps/heap-9.hprof.gz: does not start with the gzip magic"
        ),
        "{p:?}"
    );

    make_artifact(t.path(), "text", &[1]);
    write_bytes(
        &t.path().join("text/project/dumps/heap-9.hprof.gz"),
        &gz(b"not a heap dump at all, just text padding"),
    );
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "text: project/dumps/heap-9.hprof.gz: does not inflate to a file starting with"
        ),
        "{p:?}"
    );

    make_artifact(t.path(), "short", &[1]);
    write_bytes(
        &t.path().join("short/project/dumps/heap-9.hprof.gz"),
        &gz(b"JAVA PROFILE 1.0.2\0\0\0"),
    );
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "short: project/dumps/heap-9.hprof.gz: the HPROF header ends before its timestamp"
        ),
        "{p:?}"
    );

    make_artifact(t.path(), "broken", &[1]);
    let mut bad = gz(&hprof(7));
    bad.truncate(14);
    write_bytes(&t.path().join("broken/project/dumps/heap-9.hprof.gz"), &bad);
    let p = problems(t.path());
    assert!(has(&p, "broken: project/dumps/heap-9.hprof.gz:"), "{p:?}");

    make_artifact(t.path(), "src", &[1]);
    write(&t.path().join("src/project/Main.java"), "class Main {}\n");
    assert!(has(
        &problems(t.path()),
        "src: artifact fixture: project/ holds 1 source file"
    ));
}

#[test]
fn artifact_leaks_need_an_owner_and_need_no_site() {
    let t = tempfile::tempdir().unwrap();
    make_artifact(t.path(), "noowner", &[1, 2]);
    write(
        &t.path().join("noowner/expected.toml"),
        &ARTIFACT_TOML
            .replace("noowner", "x")
            .replace("ID", "noowner")
            .replace("owner = \"quotes.RfqDesk.OPEN\"\n", ""),
    );
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "noowner: leak L1: needs a site, or an owner when there is no site"
        ),
        "{p:?}"
    );
    for (id, owner) in [("sp", "a b.C.d"), ("sl", "a/b.C.d"), ("pad", " a.B.c")] {
        make_artifact(t.path(), id, &[1, 2]);
        write(
            &t.path().join(format!("{id}/expected.toml")),
            &ARTIFACT_TOML
                .replace("ID", id)
                .replace("quotes.RfqDesk.OPEN", owner),
        );
        let p = problems(t.path());
        assert!(has(&p, &format!("{id}: leak L1: owner")), "{id}: {p:?}");
    }
}

#[test]
fn a_symptom_fixture_holds_only_notes() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("sym");
    write(&d.join("README.md"), "r\n");
    write(&d.join("project/NOTES.txt"), "operator notes\n");
    write(
        &d.join("expected.toml"),
        r#"schema_version = 1
id = "sym"
runtime = "jvm"
entry = "symptom"
category = "impossible"
dynamic_confirmation = false
[task]
prompt = "p"
"#,
    );
    assert_eq!(problems(t.path()), Vec::<String>::new());
    write(&d.join("project/Main.java"), "class Main {}\n");
    assert!(has(
        &problems(t.path()),
        "sym: symptom fixture: project/ holds 1 source file"
    ));
}

fn multi_toml(id: &str, runtimes: &str, extra: &str) -> String {
    format!(
        r#"schema_version = 1
id = "{id}"
runtime = "jvm"
{runtimes}
category = "multi-leak"
dynamic_confirmation = true
[task]
prompt = "p"
[[leak]]
id = "L1"
site = "src/a.py:10"
{extra}
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
share = 0.5
[[leak]]
id = "L2"
site = "native/lib.rs:2"
holder = "native-handle/global-ref"
region = "off-heap"
cause = "L-HANDLE"
share = 0.5
"#
    )
}

#[test]
fn multi_runtime_fixtures_need_sources_for_every_runtime() {
    let t = tempfile::tempdir().unwrap();
    let mk = |id: &str, toml: &str, rust: bool| {
        let d = t.path().join(id);
        write(&d.join("README.md"), "r\n");
        for f in ["a", "b"] {
            write(&d.join(format!("project/src/{f}.py")), &filler(f, 60));
        }
        write(&d.join("project/Main.java"), &filler("m", 40));
        if rust {
            write(&d.join("project/native/lib.rs"), &filler("n", 40));
        }
        write(&d.join("expected.toml"), toml);
    };
    mk(
        "ok",
        &multi_toml("ok", "runtimes = [\"jvm\", \"rust\"]", "runtime = \"jvm\""),
        true,
    );
    assert_eq!(problems(t.path()), Vec::<String>::new());

    mk(
        "norust",
        &multi_toml("norust", "runtimes = [\"jvm\", \"rust\"]", ""),
        false,
    );
    // The native/lib.rs site also fails to exist; the runtime rule is the one under test.
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "norust: runtimes lists \"rust\" but project/ has no rs source files"
        ),
        "{p:?}"
    );

    mk(
        "prim",
        &multi_toml("prim", "runtimes = [\"go\", \"rust\"]", ""),
        true,
    );
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "prim: runtime \"jvm\" (the primary) must be one of runtimes"
        ),
        "{p:?}"
    );
    assert!(
        has(
            &p,
            "prim: runtimes lists \"go\" but project/ has no go source files"
        ),
        "{p:?}"
    );

    mk("one", &multi_toml("one", "runtimes = [\"jvm\"]", ""), true);
    assert!(has(
        &problems(t.path()),
        "one: runtimes lists fewer than two runtimes"
    ));
    mk(
        "dup",
        &multi_toml("dup", "runtimes = [\"jvm\", \"rust\", \"jvm\"]", ""),
        true,
    );
    assert!(has(
        &problems(t.path()),
        "dup: runtimes lists \"jvm\" twice"
    ));
    mk(
        "leakrt",
        &multi_toml(
            "leakrt",
            "runtimes = [\"jvm\", \"rust\"]",
            "runtime = \"go\"",
        ),
        true,
    );
    assert!(has(
        &problems(t.path()),
        "leakrt: leak L1: runtime \"go\" is not one of the fixture's runtimes"
    ));
}

#[test]
fn kotlin_counts_as_source_and_needs_a_build_script_with_kotlinc() {
    let t = tempfile::tempdir().unwrap();
    let d = t.path().join("kt");
    write(&d.join("README.md"), "r\n");
    for f in ["A", "B", "C"] {
        write(&d.join(format!("project/src/{f}.kt")), &filler(f, 60));
    }
    write(
        &d.join("expected.toml"),
        &trap("kt", "")
            .replace("python", "jvm")
            .replace("src/a.py:10", "src/A.kt:10"),
    );
    let p = problems(t.path());
    assert!(
        has(&p, "kt: project/ has .kt sources, so it needs a build.sh"),
        "{p:?}"
    );
    assert!(
        !has(&p, "source files"),
        "three .kt files satisfy the size rule: {p:?}"
    );
    write(
        &d.join("project/build.sh"),
        "#!/bin/sh\nif command -v kotlinc >/dev/null; then kotlinc src/*.kt; else java -cp \"$KCP\" K2JVMCompiler src/*.kt; fi\n",
    );
    assert_eq!(problems(t.path()), Vec::<String>::new());
}

fn decoy(id: &str, pair: &str, toolchain: &str, version: u32) -> String {
    format!(
        r#"schema_version = {version}
id = "{id}"
runtime = "python"
category = "decoy-control"
pair = "{pair}"
dynamic_confirmation = true
{toolchain}
[task]
prompt = "p"
[[non_leak]]
id = "N1"
cause = "N-WARM"
discriminator = "d"
"#
    )
}

fn free_trap(id: &str, pair: &str, toolchain: &str, version: u32) -> String {
    format!(
        r#"schema_version = {version}
id = "{id}"
runtime = "python"
category = "archetype-free"
pair = "{pair}"
dynamic_confirmation = true
{toolchain}
[task]
prompt = "p"
[[leak]]
id = "L1"
site = "src/a.py:10"
holder = "static/global"
region = "heap"
cause = "UNCLASSIFIED"
"#
    )
}

#[test]
fn an_unclassified_version_pair_is_accepted() {
    let t = tempfile::tempdir().unwrap();
    make(
        t.path(),
        "zs-3144",
        &free_trap("zs-3144", "zs-3145", "[toolchain]\npython = \"3.14.4\"", 2),
    );
    make(
        t.path(),
        "zs-3145",
        &decoy("zs-3145", "zs-3144", "[toolchain]\npython = \"3.14.5\"", 2),
    );
    assert_eq!(problems(t.path()), Vec::<String>::new());
}

#[test]
fn a_pair_needs_one_trap_and_one_decoy() {
    let t = tempfile::tempdir().unwrap();
    make(t.path(), "a", &free_trap("a", "b", "", 1));
    make(t.path(), "b", &free_trap("b", "a", "", 1));
    let p = problems(t.path());
    assert!(
        p.iter().any(
            |l| l.contains("one trap (archetype-leak or archetype-free) and one decoy-control")
        ),
        "{p:?}"
    );
}

#[test]
fn a_version_pair_may_not_swap_the_tool_it_names() {
    let t = tempfile::tempdir().unwrap();
    make(
        t.path(),
        "a",
        &free_trap("a", "b", "[toolchain]\npython = \"3.14.4\"", 2),
    );
    make(t.path(), "b", &decoy("b", "a", "", 1));
    let p = problems(t.path());
    assert!(
        p.iter()
            .any(|l| l.contains("names other tools in [toolchain]")),
        "{p:?}"
    );
}

#[test]
fn a_version_pair_may_not_swap_one_tool_for_another() {
    let t = tempfile::tempdir().unwrap();
    make(
        t.path(),
        "a",
        &free_trap("a", "b", "[toolchain]\npython = \"3.14.4\"", 2),
    );
    make(
        t.path(),
        "b",
        &decoy("b", "a", "[toolchain]\nnode = \"22.12.0\"", 2),
    );
    let p = problems(t.path());
    assert!(
        p.iter()
            .any(|l| l.contains("names other tools in [toolchain]")
                && l.contains("python")
                && l.contains("node")),
        "{p:?}"
    );
}
