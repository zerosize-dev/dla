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
    make(t.path(), "a", &decoy("a", "b", "p"));
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
