//! End-to-end tests of the census over small projects.

use std::fs;

use dla_core::{Ledger, RunState, state::now_utc, toolctx::ToolCtx};

use super::*;

fn project(files: &[(&str, &str)]) -> (tempfile::TempDir, ToolCtx) {
    let dir = tempfile::tempdir().unwrap();
    for (p, c) in files {
        let path = dir.path().join(p);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, c).unwrap();
    }
    let run_dir = dir.path().join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st = RunState::init(
        dir.path().to_str().unwrap(),
        Some("s"),
        3600,
        None,
        now_utc(),
    )
    .unwrap();
    st.run_id = "r1".into();
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project: dir.path().to_path_buf(),
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    };
    (dir, ctx)
}

#[test]
fn python_census_writes_evidence_candidates_annotations_and_coverage() {
    let (_d, ctx) = project(&[(
        "app/reg.py",
        "_L = {}\n\ndef sub(t, l):\n    _L.setdefault(t, []).append(l)\n\ndef reset():\n    _L.clear()\n",
    )]);
    let out = census(&ctx, Runtime::Python).unwrap();
    assert_eq!(out.candidates, 1);
    assert_eq!(out.evidence.len(), 1);
    let ledger = ctx.ledger().unwrap();
    let f = ledger.findings().next().unwrap();
    assert_eq!(f.origin.to_string(), "census");
    assert_eq!(f.site.as_deref(), Some("app/reg.py:1"));
    assert_eq!(f.holder.map(|h| h.as_str()), Some("static/global"));
    assert_eq!(f.suggested_archetype, None);
    assert_eq!(f.supports.len(), 1);
    let a = ledger.annotation(&f.id).unwrap();
    assert_eq!(a.writers, ["app/reg.py:4"]);
    assert_eq!(a.removers, ["app/reg.py:7"]);
    let raw: serde_json::Value =
        serde_json::from_slice(&fs::read(ctx.run_dir.join("census/python.json")).unwrap()).unwrap();
    assert_eq!(raw["entries"][0]["owner"], "app.reg._L");
    let cov = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
    assert_eq!(cov.workers["python"].backends["census"].enumerated, 1);
}

#[test]
fn rerunning_does_not_duplicate_candidates() {
    let (_d, ctx) = project(&[("a.py", "X = []\n")]);
    census(&ctx, Runtime::Python).unwrap();
    let again = census(&ctx, Runtime::Python).unwrap();
    assert_eq!((again.candidates, again.skipped_duplicates), (0, 1));
}

#[test]
fn test_files_and_build_output_are_not_enumerated() {
    let (_d, ctx) = project(&[
        ("src/a.py", "X = []\n"),
        ("tests/test_a.py", "Y = []\n"),
        ("build/b.py", "Z = []\n"),
        ("node_modules/m/c.py", "W = []\n"),
    ]);
    let out = census(&ctx, Runtime::Python).unwrap();
    assert_eq!(out.candidates, 1);
}

#[test]
fn class_files_take_precedence_and_cover_their_source() {
    let (d, ctx) = project(&[(
        "src/a/b/Foo.java",
        "package a.b;\nclass Foo {\n  static final java.util.Map<String,String> CACHE = new java.util.HashMap<>();\n}\n",
    )]);
    let dir = d.path().join("target/classes/a/b");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("Foo.class"),
        classfile::testutil::sample("a/b/Foo", "CACHE", "Ljava/util/Map;"),
    )
    .unwrap();
    let out = census(&ctx, Runtime::Jvm).unwrap();
    assert_eq!(out.summary["source"], "class-files+tree-sitter");
    let ledger = ctx.ledger().unwrap();
    let sites: Vec<_> = ledger.findings().filter_map(|f| f.site.clone()).collect();
    // From the class file: the field at its <clinit> line and the thread spawn.
    assert!(
        sites.contains(&"src/a/b/Foo.java:7".to_owned()),
        "{sites:?}"
    );
    assert!(
        !sites.contains(&"src/a/b/Foo.java:3".to_owned()),
        "{sites:?}"
    );
}

#[test]
fn jvm_without_class_files_uses_tree_sitter() {
    let (_d, ctx) = project(&[(
        "src/Foo.java",
        "class Foo {\n  static final java.util.Map<String,String> CACHE = new java.util.HashMap<>();\n}\n",
    )]);
    let out = census(&ctx, Runtime::Jvm).unwrap();
    assert_eq!(out.summary["source"], "tree-sitter");
    assert_eq!(out.candidates, 1);
}

/// The census must not read archetype or known-leak data: the crate's sources name
/// neither directory nor the tag field.
#[test]
fn the_crate_never_touches_archetype_or_known_leak_data() {
    let forbidden = [
        ["data/", "archetypes"].concat(),
        ["known", "-leaks"].concat(),
        ["known", "_leaks"].concat(),
        ["suggested", "_archetype: Some"].concat(),
        ["suggested", "_archetype = Some"].concat(),
        ["archetype", "s/"].concat(),
        ["ARCHETYPE", "_IDS"].concat(),
    ];
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stack = vec![dir];
    let mut checked = 0;
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && p.file_name().is_some_and(|n| n != "tests.rs")
            {
                let text = fs::read_to_string(&p).unwrap();
                for f in &forbidden {
                    assert!(!text.contains(f.as_str()), "{} mentions {f}", p.display());
                }
                checked += 1;
            }
        }
    }
    assert!(checked >= 10);
}

/// Candidates never carry a suggested archetype, so blanking tags cannot change
/// the enumerated set.
#[test]
fn candidates_carry_no_archetype_tag() {
    let (_d, ctx) = project(&[(
        "a.js",
        "const seen = new Map();\nlet timer = null;\nsetInterval(f, 5);\n",
    )]);
    let out = census(&ctx, Runtime::Node).unwrap();
    assert_eq!(out.candidates, 3);
    assert!(
        ctx.ledger()
            .unwrap()
            .findings()
            .all(|f| f.suggested_archetype.is_none())
    );
}

/// Sets the run's scope in `state.json`.
fn set_scope(ctx: &ToolCtx, kind: dla_core::state::ScopeKind, path: &str, excluded: &[&str]) {
    RunState::update(&ctx.run_dir, |s| {
        s.scope = Some(dla_core::state::Scope {
            kind,
            path: path.into(),
            excluded: excluded.iter().map(|e| (*e).to_owned()).collect(),
        });
        Ok(())
    })
    .unwrap();
}

#[test]
fn module_scope_skips_other_modules_and_excluded_dirs_and_counts_them() {
    let reg = "_L = {}\n\ndef sub(t, l):\n    _L.setdefault(t, []).append(l)\n";
    let (_d, ctx) = project(&[
        ("svc/a/reg.py", reg),
        ("svc/a/gen/reg.py", reg),
        ("other/reg.py", reg),
        ("other/two.py", reg),
    ]);
    set_scope(
        &ctx,
        dla_core::state::ScopeKind::Module,
        "svc/a",
        &["svc/a/gen"],
    );
    let out = census(&ctx, Runtime::Python).unwrap();
    assert_eq!(out.candidates, 1);
    assert_eq!(out.summary["out_of_scope"], 3);
    let ledger = ctx.ledger().unwrap();
    let sites: Vec<_> = ledger.findings().filter_map(|f| f.site.clone()).collect();
    assert_eq!(sites, ["svc/a/reg.py:1"]);
    let cov = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
    assert_eq!(cov.workers["python"].out_of_scope, 3);
}

#[test]
fn no_scope_reports_no_out_of_scope_field() {
    let (_d, ctx) = project(&[("a.py", "_L = {}\n")]);
    let out = census(&ctx, Runtime::Python).unwrap();
    assert!(out.summary.get("out_of_scope").is_none());
}
