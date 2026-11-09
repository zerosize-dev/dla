//! Owner-to-site resolution against a census file and a source tree.

use std::{fs, path::Path};

use dla_core::attribution::{self, Resolver, Via};
use serde_json::json;

fn put(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn census(run: &Path, lang: &str, entries: serde_json::Value) {
    put(
        run,
        &format!("census/{lang}.json"),
        &json!({"lang": lang, "entries": entries}).to_string(),
    );
}

fn entry(owner: &str, site: &str, writers: &[&str]) -> serde_json::Value {
    json!({"kind": "static-field", "holder": "static/global", "region": "heap",
           "site": site, "owner": owner, "writers": writers, "removers": ["x/R.java:3"]})
}

const HOLDER: &str = "package app;\n\npublic final class Holder {\n    // all items\n    static final java.util.List<Item> ITEMS = new java.util.ArrayList<>();\n    static void add(Item i) { ITEMS.add(i); }\n}\n";

#[test]
fn a_census_entry_gives_the_site_and_its_writers() {
    let t = tempfile::tempdir().unwrap();
    let (project, run) = (t.path().join("p"), t.path().join("run"));
    fs::create_dir_all(&project).unwrap();
    census(
        &run,
        "java",
        json!([entry(
            "app.Holder.ITEMS",
            "src/app/Holder.java:5",
            &["src/app/Holder.java:6"]
        )]),
    );
    for key in [
        "static:app.Holder.ITEMS",
        "app.Holder.ITEMS",
        "Holder.ITEMS",
        "static:app.Holder$$SpringCGLIB$$0.ITEMS",
    ] {
        let r = attribution::resolve(&run, &project, key).unwrap_or_else(|| panic!("{key}"));
        assert_eq!(r.via, Via::Census, "{key}");
        assert_eq!(r.site, "src/app/Holder.java:5");
        assert_eq!(r.owner, "app.Holder.ITEMS");
        assert_eq!(r.writers, ["src/app/Holder.java:6"]);
        assert_eq!(r.removers, ["x/R.java:3"]);
        assert_eq!(r.kind.as_deref(), Some("static-field"));
    }
    // Not a field: a thread, a jni global, a bare name.
    for key in [
        "thread:main",
        "jni-global:app.Blob",
        "ITEMS",
        "instance:root>a.B",
    ] {
        assert!(attribution::resolve(&run, &project, key).is_none(), "{key}");
    }
}

#[test]
fn kotlin_object_fields_resolve_through_the_census() {
    let t = tempfile::tempdir().unwrap();
    let (project, run) = (t.path().join("p"), t.path().join("run"));
    fs::create_dir_all(&project).unwrap();
    census(
        &run,
        "kotlin",
        json!([entry(
            "shop.codec.Audit.trail",
            "src/Audit.kt:2",
            &["src/Audit.kt:3"]
        )]),
    );
    let r = attribution::resolve(&run, &project, "static:shop.codec.Audit.trail").unwrap();
    assert_eq!(r.site, "src/Audit.kt:2");
    assert_eq!(r.via, Via::Census);
}

#[test]
fn an_ambiguous_owner_resolves_to_nothing() {
    let t = tempfile::tempdir().unwrap();
    let (project, run) = (t.path().join("p"), t.path().join("run"));
    fs::create_dir_all(&project).unwrap();
    census(
        &run,
        "java",
        json!([
            entry("a.Cache.items", "a/Cache.java:3", &[]),
            entry("b.Cache.items", "b/Cache.java:3", &[]),
        ]),
    );
    // `Cache.items` fits both; the full names fit one each.
    assert!(attribution::resolve(&run, &project, "Cache.items").is_none());
    assert_eq!(
        attribution::resolve(&run, &project, "static:b.Cache.items")
            .unwrap()
            .site,
        "b/Cache.java:3"
    );
}

#[test]
fn without_a_census_the_field_declaration_is_found_in_the_source() {
    let t = tempfile::tempdir().unwrap();
    let (project, run) = (t.path().join("p"), t.path().join("run"));
    fs::create_dir_all(&run).unwrap();
    put(&project, "src/main/java/app/Holder.java", HOLDER);
    put(
        &project,
        "build/gen/app/Holder.java",
        "class Holder { int ITEMS; }",
    );
    let r = attribution::resolve(&run, &project, "static:app.Holder.ITEMS").unwrap();
    assert_eq!(r.via, Via::SourceScan);
    assert_eq!(r.site, "src/main/java/app/Holder.java:5");
    assert!(r.writers.is_empty() && r.kind.is_none());
    // A field the file does not declare, and a class without a file.
    assert!(attribution::resolve(&run, &project, "app.Holder.OTHER").is_none());
    assert!(attribution::resolve(&run, &project, "app.Missing.ITEMS").is_none());
}

#[test]
fn nested_classes_kotlin_files_and_equal_names_in_two_packages() {
    let t = tempfile::tempdir().unwrap();
    let (project, run) = (t.path().join("p"), t.path().join("run"));
    fs::create_dir_all(&run).unwrap();
    put(
        &project,
        "src/shop/Outer.java",
        "package shop;\nclass Outer {\n  static class Inner {\n    static java.util.Map<String,String> cache = new java.util.HashMap<>();\n  }\n}\n",
    );
    put(
        &project,
        "src/shop/Audit.kt",
        "package shop\n\nobject Audit {\n    val trail = mutableListOf<String>()\n}\n",
    );
    put(
        &project,
        "src/a/Dup.java",
        "package a;\nclass Dup {\n  static int[] big = new int[9];\n}\n",
    );
    put(
        &project,
        "src/b/Dup.java",
        "package b;\nclass Dup {\n  static int[] big = new int[9];\n}\n",
    );
    let r = attribution::resolve(&run, &project, "static:shop.Outer$Inner.cache").unwrap();
    assert_eq!(r.site, "src/shop/Outer.java:4");
    let r = attribution::resolve(&run, &project, "static:shop.Audit.trail").unwrap();
    assert_eq!(r.site, "src/shop/Audit.kt:4");
    // The package path tells the two Dup files apart; without it there is no answer.
    assert_eq!(
        attribution::resolve(&run, &project, "static:b.Dup.big")
            .unwrap()
            .site,
        "src/b/Dup.java:3"
    );
    assert!(attribution::resolve(&run, &project, "Dup.big").is_none());
}

#[test]
fn frames_in_the_project_have_a_site_and_others_do_not() {
    let t = tempfile::tempdir().unwrap();
    let (project, run) = (t.path().join("p"), t.path().join("run"));
    fs::create_dir_all(&run).unwrap();
    put(&project, "src/trail.rs", "fn record() {}\n");
    put(
        &project,
        "src/shop/Pool.java",
        "package shop;\nclass Pool { void spawn() {} }\n",
    );
    let r = Resolver::new(&run, &project);
    assert_eq!(
        r.frame_site("shop::trail::record", Some("src/trail.rs"), Some(40))
            .as_deref(),
        Some("src/trail.rs:40")
    );
    let abs = project.join("src/trail.rs");
    assert_eq!(
        r.frame_site("f", Some(abs.to_str().unwrap()), Some(3))
            .as_deref(),
        Some("src/trail.rs:3")
    );
    assert_eq!(r.frame_site("f", Some("/rustc/abc/lib.rs"), Some(3)), None);
    assert_eq!(r.frame_site("f", Some("src/gone.rs"), Some(3)), None);
    assert_eq!(
        r.frame_site("shop.Pool.spawn", Some("Pool.java"), Some(2))
            .as_deref(),
        Some("src/shop/Pool.java:2")
    );
    assert_eq!(r.frame_site("java.util.ArrayList.add", None, Some(9)), None);
    assert_eq!(
        r.frame_site("shop.Pool.spawn", None, None).as_deref(),
        Some("src/shop/Pool.java:1")
    );
}

#[test]
fn a_broken_census_file_is_skipped() {
    let t = tempfile::tempdir().unwrap();
    let (project, run) = (t.path().join("p"), t.path().join("run"));
    fs::create_dir_all(&project).unwrap();
    put(&run, "census/java.json", "{not json");
    put(&run, "census/go.json", "{\"entries\": 3}");
    assert!(attribution::census_entries(&run).is_empty());
    assert!(attribution::resolve(&run, &project, "a.B.c").is_none());
}
