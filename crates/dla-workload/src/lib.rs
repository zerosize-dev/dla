//! `dla workload discover`: what a loop can run.
//!
//! # Contract
//!
//! One entry point, [`discover`]. It does its own writes through the [`ToolCtx`]
//! (design and shared rules: `dla_core::toolctx`) and returns a bounded
//! [`StageOutcome`]; the CLI (`workload_cmd.rs`) only parses arguments and prints.
//!
//! Writes: `.dla/<run>/workload/workloads.json` (a `dla_core::workload::WorkloadReport`),
//! ONE `workload` evidence record (`raw` = `workload/workloads.json`), and in
//! `coverage.json` for each runtime `entry_points.discovered` and
//! `entry_points.not_exercisable` (`exercised` stays empty until the M3 harness runs).
//! It adds no candidates and runs nothing: it reads files only. A runtime whose
//! `dla probe --build` failed gets, for every entry point, the reason `build_fails`,
//! and each of its workloads carries `not_exercisable = "build_fails"`; the
//! entry-point lists never hold workload ids. Entry points no workload references get
//! `no_harness`.
//!
//! # How it works
//!
//! * Files are scanned once (skipping `.git`, `.dla`, `target`, `build`, `node_modules`,
//!   `venv`, `.venv`, `out`, `dist`, `__pycache__`) and parsed with the tree-sitter layer
//!   of `dla-census`.
//! * Per runtime a finder produces [`model::Raw`] workloads. Each `command` is a POSIX
//!   shell line to run with `sh -c` in `cwd` (project-relative).
//! * An *entry point* is a non-test function or method that is public in the source
//!   language's sense (Java and Kotlin: not `private`; Go: exported; Rust: `pub` or a
//!   trait impl; Python and JavaScript: not `_`-prefixed; C: not `static`), named
//!   `file::Owner.name`. `main` functions are workloads, not entry points.
//! * A workload *exercises* an entry point when an identifier of the workload's source
//!   files (definition names excluded) equals the entry point's bare name, or when such
//!   an identifier names an entry point of another project file whose own identifiers
//!   (followed [`REF_HOPS`] hops through the bodies of reached entry points) name it. This is an over-approximation
//!   by name, not a call graph.
//! * Rank = number of entry points exercised; workloads are sorted by rank, then kind
//!   (test, main, dev-server, script), then id.

mod go;
pub mod harness;
pub mod iteration;
mod jvm;
pub mod model;
mod native;
mod node;
mod python;
mod rust;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use dla_core::{
    Refusal,
    coverage::{CoverageManifest, EntryPoints, NotExercisable},
    error::Error,
    evidence::EvidenceKind,
    probe::{BuildStatus, ProbeReport},
    runtime::Runtime,
    toolctx::{EvidenceSpec, StageOutcome, ToolCtx},
    workload::{Workload, WorkloadKind, WorkloadReport},
};
use serde_json::json;

use model::{Raw, Scan};

/// Reason for an entry point that no workload references.
pub const NO_HARNESS: &str = "no_harness";
/// How many file-to-file hops of identifier references are followed from a workload's
/// own sources when computing the entry points it exercises.
pub const REF_HOPS: usize = 2;
static NO_REFS: BTreeSet<String> = BTreeSet::new();
/// Reason when the runtime's probe build failed.
pub const BUILD_FAILS: &str = "build_fails";

fn kind_order(k: WorkloadKind) -> u8 {
    match k {
        WorkloadKind::Test => 0,
        WorkloadKind::Main => 1,
        WorkloadKind::DevServer => 2,
        WorkloadKind::Script => 3,
    }
}

/// Runtimes whose build failed in `probe/probe.json`, if that file exists.
fn failed_builds(ctx: &ToolCtx) -> BTreeSet<Runtime> {
    let p = ctx.run_dir.join("probe/probe.json");
    let Ok(text) = fs::read_to_string(p) else {
        return BTreeSet::new();
    };
    let Ok(report) = serde_json::from_str::<ProbeReport>(&text) else {
        return BTreeSet::new();
    };
    report
        .builds
        .iter()
        .filter(|b| matches!(b.status, BuildStatus::Failed | BuildStatus::Timeout))
        .map(|b| b.runtime)
        .collect()
}

/// Pure core of discovery over an already scanned project.
pub fn build_report(
    scan: &Scan,
    lang: Option<Runtime>,
    build_failed: &BTreeSet<Runtime>,
) -> (WorkloadReport, Vec<Runtime>) {
    let wanted = |rt: Runtime| lang.is_none_or(|l| l == rt);
    let mut raws: Vec<Raw> = Vec::new();
    for (rt, found) in [
        (Runtime::Jvm, jvm::discover as fn(&Scan) -> Vec<Raw>),
        (Runtime::Go, go::discover),
        (Runtime::Rust, rust::discover),
        (Runtime::Node, node::discover),
        (Runtime::Python, python::discover),
        (Runtime::Native, native::discover),
    ] {
        if wanted(rt) {
            raws.extend(found(scan));
        }
    }
    // Entry points of the wanted runtimes, by bare name.
    let mut entries: Vec<(Runtime, String, String)> = Vec::new();
    let mut by_name: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for f in scan.infos.values() {
        let rt = f.lang.runtime();
        if !wanted(rt) {
            continue;
        }
        for e in &f.entries {
            entries.push((rt, e.id.clone(), e.name.clone()));
        }
    }
    entries.sort();
    entries.dedup();
    let refs: Vec<&BTreeSet<String>> = entries
        .iter()
        .map(|(_, id, _)| {
            let file = id.split("::").next().unwrap_or("");
            scan.infos
                .get(file)
                .and_then(|f| f.entries.iter().find(|e| &e.id == id))
                .map_or(&NO_REFS, |e| &e.refs)
        })
        .collect();
    for (i, (_, _, name)) in entries.iter().enumerate() {
        by_name.entry(name.as_str()).or_default().push(i);
    }
    let mut exercised: BTreeSet<usize> = BTreeSet::new();
    let mut workloads: Vec<Workload> = raws
        .into_iter()
        .map(|r| {
            let mut idents: BTreeSet<&str> = r.extra.iter().map(String::as_str).collect();
            for s in &r.srcs {
                if let Some(f) = scan.infos.get(s) {
                    idents.extend(f.idents.iter().map(String::as_str));
                }
            }
            let mut hit: BTreeSet<usize> = BTreeSet::new();
            let mut frontier: Vec<usize> = Vec::new();
            for id in &idents {
                if let Some(v) = by_name.get(id) {
                    frontier.extend(v.iter().copied().filter(|&i| entries[i].0 == r.runtime));
                }
            }
            // Follow identifier references out of what was reached, a bounded number
            // of hops, so entry points used through other files count as exercised.
            for _ in 0..=REF_HOPS {
                frontier.retain(|i| hit.insert(*i));
                let mut next: Vec<usize> = Vec::new();
                for &i in &frontier {
                    for id in refs[i] {
                        if let Some(v) = by_name.get(id.as_str()) {
                            next.extend(v.iter().copied().filter(|&j| entries[j].0 == r.runtime));
                        }
                    }
                }
                if next.is_empty() {
                    break;
                }
                frontier = next;
            }
            exercised.extend(&hit);
            let entry_points: Vec<String> = hit.iter().map(|&i| entries[i].1.clone()).collect();
            let iteration = matches!(r.kind, WorkloadKind::Main | WorkloadKind::Script)
                .then(|| iteration::detect(scan, r.runtime, &r.command, &r.cwd, &r.file, &r.srcs))
                .flatten();
            Workload {
                id: r.id,
                kind: r.kind,
                runtime: r.runtime,
                command: r.command,
                cwd: r.cwd,
                file: r.file,
                not_exercisable: build_failed
                    .contains(&r.runtime)
                    .then(|| BUILD_FAILS.to_owned()),
                rank: u32::try_from(entry_points.len()).unwrap_or(u32::MAX),
                entry_points,
                iteration_arg: iteration.as_ref().map(|i| i.template.clone()),
                iteration_default: iteration.and_then(|i| i.default),
            }
        })
        .collect();
    workloads.sort_by(|a, b| {
        b.rank
            .cmp(&a.rank)
            .then(kind_order(a.kind).cmp(&kind_order(b.kind)))
            .then(a.id.cmp(&b.id))
    });
    workloads.dedup_by(|a, b| a.id == b.id);

    let mut not_exercisable = Vec::new();
    for (i, (rt, id, _)) in entries.iter().enumerate() {
        if build_failed.contains(rt) {
            not_exercisable.push(NotExercisable {
                entry: id.clone(),
                reason: BUILD_FAILS.into(),
            });
        } else if !exercised.contains(&i) {
            not_exercisable.push(NotExercisable {
                entry: id.clone(),
                reason: NO_HARNESS.into(),
            });
        }
    }
    let mut runtimes: BTreeSet<Runtime> = scan
        .infos
        .values()
        .map(|f| f.lang.runtime())
        .filter(|rt| wanted(*rt))
        .collect();
    runtimes.extend(workloads.iter().map(|w| w.runtime));
    if let Some(l) = lang {
        runtimes.insert(l);
    }
    let report = WorkloadReport {
        schema_version: dla_core::SCHEMA_VERSION,
        workloads,
        entry_points: entries.iter().map(|e| e.1.clone()).collect(),
        not_exercisable,
        build_failed: build_failed
            .iter()
            .copied()
            .filter(|rt| wanted(*rt))
            .collect(),
    };
    (report, runtimes.into_iter().collect())
}

/// Finds tests, mains and dev servers of the project (all runtimes when `lang` is
/// `None`), ranks them by the entry points they exercise and records the result.
///
/// Reads files only; a failed `dla probe --build` is recorded as a coverage gap, not
/// an error.
pub fn discover(ctx: &ToolCtx, lang: Option<Runtime>) -> Result<StageOutcome, Refusal> {
    let scan = Scan::new(&ctx.project);
    let failed = failed_builds(ctx);
    let (report, runtimes) = build_report(&scan, lang, &failed);

    let dir = ctx.stage_dir("workload")?;
    let file = dir.join("workloads.json");
    let mut text = serde_json::to_string_pretty(&report).map_err(|e| {
        Error::io(
            &file,
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
        )
        .to_refusal()
    })?;
    text.push('\n');
    fs::write(&file, text).map_err(|e| Error::io(&file, e).to_refusal())?;

    let mut by_kind: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut by_runtime: BTreeMap<&'static str, u64> = BTreeMap::new();
    for w in &report.workloads {
        *by_kind.entry(w.kind.as_str()).or_default() += 1;
        *by_runtime.entry(w.runtime.as_str()).or_default() += 1;
    }
    let exercised = report
        .workloads
        .iter()
        .flat_map(|w| w.entry_points.iter())
        .collect::<BTreeSet<_>>()
        .len();
    let result = json!({
        "workloads": report.workloads.len(),
        "by_kind": by_kind,
        "by_runtime": by_runtime,
        "entry_points": report.entry_points.len(),
        "referenced_entry_points": exercised,
        "not_exercisable": report.not_exercisable.len(),
        "build_failed": report.build_failed.iter().map(|r| r.as_str()).collect::<Vec<_>>(),
        "top": report.workloads.iter().take(5).map(|w| json!({
            "id": w.id, "kind": w.kind.as_str(), "rank": w.rank
        })).collect::<Vec<_>>(),
    });
    let mut ledger = ctx.ledger()?;
    let mut spec = EvidenceSpec::new(
        EvidenceKind::Workload,
        "dla",
        env!("CARGO_PKG_VERSION"),
        result.clone(),
    );
    spec.raw = Some("workload/workloads.json".into());
    spec.limitations = vec![
        "entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph".into(),
        "workloads were discovered by reading files; none was run".into(),
    ];
    let ev = ctx.add_evidence(&mut ledger, spec)?;

    CoverageManifest::update(&ctx.run_dir, |m| {
        for rt in &runtimes {
            let mine = |id: &str| runtime_of_entry(&scan, id) == Some(*rt);
            let prev = m.worker_mut(rt.as_str()).entry_points.exercised.clone();
            m.set_entry_points(
                rt.as_str(),
                EntryPoints {
                    discovered: report
                        .entry_points
                        .iter()
                        .filter(|e| runtime_of_entry(&scan, e) == Some(*rt))
                        .cloned()
                        .collect(),
                    exercised: prev,
                    not_exercisable: report
                        .not_exercisable
                        .iter()
                        .filter(|n| mine(&n.entry))
                        .cloned()
                        .collect(),
                },
            );
        }
    })
    .map_err(|e| e.to_refusal())?;

    Ok(StageOutcome {
        evidence: vec![ev.to_string()],
        candidates: 0,
        skipped_duplicates: 0,
        artifacts: vec![
            ctx.pointer("workload/workloads.json"),
            ctx.pointer("coverage.json"),
        ],
        summary: result,
    })
}

fn runtime_of_entry(scan: &Scan, id: &str) -> Option<Runtime> {
    let file = id.split("::").next()?;
    scan.infos.get(file).map(|f| f.lang.runtime())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::{Ledger, RunState, state::now_utc};
    use std::path::Path;

    fn put(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    fn report(root: &Path, lang: Option<Runtime>, failed: &[Runtime]) -> WorkloadReport {
        let scan = Scan::new(root);
        build_report(&scan, lang, &failed.iter().copied().collect()).0
    }

    fn find<'a>(r: &'a WorkloadReport, id: &str) -> &'a Workload {
        r.workloads
            .iter()
            .find(|w| w.id == id)
            .unwrap_or_else(|| panic!("no workload {id} in {:?}", r.workloads))
    }

    fn go_project(root: &Path) {
        put(root, "go.mod", "module example.com/m\n\ngo 1.22\n");
        put(
            root,
            "pkg/a.go",
            "package pkg\n\nfunc Used() int { return 1 }\nfunc Unused() int { return 2 }\nfunc hidden() {}\n\ntype T struct{}\n\nfunc (t *T) Method() {}\n",
        );
        put(
            root,
            "pkg/a_test.go",
            "package pkg\n\nimport \"testing\"\n\nfunc TestUsed(t *testing.T) { Used() }\n",
        );
        put(
            root,
            "cmd/app/main.go",
            "package main\n\nimport \"example.com/m/pkg\"\n\nfunc main() { var t pkg.T; t.Method() }\n",
        );
    }

    #[test]
    fn go_tests_mains_entry_points_and_rank() {
        let d = tempfile::tempdir().unwrap();
        go_project(d.path());
        let r = report(d.path(), None, &[]);
        let t = find(&r, "go:test:.:./pkg");
        assert_eq!(t.command, "go test -count=1 ./pkg");
        assert_eq!(t.cwd, ".");
        assert_eq!(t.entry_points, ["pkg/a.go::Used"]);
        let m = find(&r, "go:main:.:./cmd/app");
        assert_eq!(m.command, "go run ./cmd/app");
        assert_eq!(m.entry_points, ["pkg/a.go::T.Method"]);
        assert_eq!(
            r.entry_points,
            ["pkg/a.go::T.Method", "pkg/a.go::Unused", "pkg/a.go::Used"]
        );
        let nx: Vec<_> = r
            .not_exercisable
            .iter()
            .map(|n| (&n.entry[..], &n.reason[..]))
            .collect();
        assert_eq!(nx, [("pkg/a.go::Unused", "no_harness")]);
    }

    #[test]
    fn failed_build_marks_everything_of_that_runtime() {
        let d = tempfile::tempdir().unwrap();
        go_project(d.path());
        put(
            d.path(),
            "x.py",
            "def public():\n    pass\n\nif __name__ == '__main__':\n    public()\n",
        );
        let r = report(d.path(), None, &[Runtime::Go]);
        let go: Vec<_> = r
            .not_exercisable
            .iter()
            .filter(|n| n.entry.starts_with("pkg/"))
            .collect();
        assert_eq!(go.len(), 3);
        assert!(go.iter().all(|n| n.reason == BUILD_FAILS));
        // Workloads are never listed as entry points; they carry their own reason.
        assert!(
            r.not_exercisable
                .iter()
                .all(|n| !n.entry.starts_with("workload:"))
        );
        let go_w: Vec<_> = r
            .workloads
            .iter()
            .filter(|w| w.runtime == Runtime::Go)
            .collect();
        assert_eq!(go_w.len(), 2);
        assert!(
            go_w.iter()
                .all(|w| w.not_exercisable.as_deref() == Some(BUILD_FAILS))
        );
        assert!(
            r.workloads
                .iter()
                .filter(|w| w.runtime == Runtime::Python)
                .all(|w| w.not_exercisable.is_none())
        );
        assert!(r.not_exercisable.iter().all(|n| !n.entry.contains("x.py")));
        assert_eq!(r.build_failed, [Runtime::Go]);
    }

    #[test]
    fn entry_points_reached_through_other_files_are_exercised() {
        let d = tempfile::tempdir().unwrap();
        put(d.path(), "pom.xml", "<project/>");
        put(
            d.path(),
            "src/main/java/p/Cache.java",
            "package p;\npublic class Cache { public void store(int k) {} public int find(int k) { return k; } public void orphan() {} }\n",
        );
        put(
            d.path(),
            "src/main/java/p/Service.java",
            "package p;\npublic class Service { Cache c = new Cache(); public void place(int k) { c.store(k); c.find(k); } }\n",
        );
        put(
            d.path(),
            "src/test/java/p/ServiceTest.java",
            "package p;\nimport org.junit.jupiter.api.Test;\nclass ServiceTest { @Test void t() { new Service().place(1); } }\n",
        );
        let r = report(d.path(), None, &[]);
        let t = find(&r, "jvm:test:src/test/java/p/ServiceTest.java");
        assert!(
            t.entry_points.iter().any(|e| e.ends_with("Cache.store")),
            "{t:?}"
        );
        assert!(
            t.entry_points.iter().any(|e| e.ends_with("Cache.find")),
            "{t:?}"
        );
        let nx: Vec<_> = r.not_exercisable.iter().map(|n| &n.entry[..]).collect();
        assert_eq!(nx, ["src/main/java/p/Cache.java::Cache.orphan"]);
    }

    #[test]
    fn lang_filter_limits_runtimes() {
        let d = tempfile::tempdir().unwrap();
        go_project(d.path());
        put(d.path(), "x.py", "def public():\n    pass\n");
        let r = report(d.path(), Some(Runtime::Python), &[]);
        assert!(r.workloads.is_empty());
        assert_eq!(r.entry_points, ["x.py::public"]);
    }

    #[test]
    fn python_unittest_pytest_and_main() {
        let d = tempfile::tempdir().unwrap();
        put(d.path(), "app/__init__.py", "");
        put(
            d.path(),
            "app/core.py",
            "def handle(x):\n    return x\n\ndef _private():\n    pass\n\nclass Svc:\n    def run(self):\n        pass\n    def __init__(self):\n        pass\n",
        );
        put(
            d.path(),
            "app/load.py",
            "from .core import Svc\n\nif __name__ == '__main__':\n    Svc().run()\n",
        );
        put(
            d.path(),
            "tests/test_core.py",
            "import unittest\nfrom app.core import handle\n\nclass T(unittest.TestCase):\n    def test_a(self):\n        handle(1)\n",
        );
        put(
            d.path(),
            "tests/test_other.py",
            "import pytest\nfrom app import core\n\ndef test_b():\n    core.handle(2)\n",
        );
        let r = report(d.path(), None, &[]);
        assert_eq!(
            find(&r, "python:test:tests/test_core.py").command,
            "python3 -m unittest tests.test_core"
        );
        assert_eq!(
            find(&r, "python:test:tests/test_other.py").command,
            "python3 -m pytest -q tests/test_other.py"
        );
        let m = find(&r, "python:main:app/load.py");
        assert_eq!(m.command, "python3 -m app.load");
        assert_eq!(m.entry_points, ["app/core.py::Svc.run"]);
        assert_eq!(
            r.entry_points,
            ["app/core.py::Svc.run", "app/core.py::handle"]
        );
        assert!(r.not_exercisable.is_empty());
    }

    #[test]
    fn node_scripts_tests_and_runner() {
        let d = tempfile::tempdir().unwrap();
        put(
            d.path(),
            "package.json",
            r#"{"scripts":{"test":"node --test","load":"node load.js","dev":"node server.js","build":"tsc"},"devDependencies":{"vitest":"1"}}"#,
        );
        put(
            d.path(),
            "src/a.js",
            "function send() {}\nfunction _skip() {}\nclass K { go() {} }\nmodule.exports = { send, K };\n",
        );
        put(
            d.path(),
            "load.js",
            "const { send } = require('./src/a');\nsend();\n",
        );
        put(
            d.path(),
            "server.js",
            "require('http').createServer().listen(3000);\n",
        );
        put(
            d.path(),
            "test/a.test.js",
            "const { K } = require('../src/a');\nnew K().go();\n",
        );
        let r = report(d.path(), None, &[]);
        assert_eq!(
            find(&r, "node:test:test/a.test.js").command,
            "npx --no-install vitest run test/a.test.js"
        );
        assert_eq!(
            find(&r, "node:script:package.json:load").command,
            "npm run load"
        );
        let dev = find(&r, "node:script:package.json:dev");
        assert_eq!(dev.kind, WorkloadKind::DevServer);
        assert_eq!(
            find(&r, "node:script:package.json:load").entry_points,
            ["src/a.js::send"]
        );
        assert_eq!(
            find(&r, "node:test:test/a.test.js").entry_points,
            ["src/a.js::K.go"]
        );
    }

    #[test]
    fn jvm_script_maven_and_bare() {
        let d = tempfile::tempdir().unwrap();
        put(
            d.path(),
            "s/build.sh",
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then :; fi\n",
        );
        put(
            d.path(),
            "s/src/p/Svc.java",
            "package p;\npublic class Svc { public int calc() { return 1; } private void x() {} }\n",
        );
        put(
            d.path(),
            "s/src/p/Main.java",
            "package p;\nclass Main { public static void main(String[] a) { new Svc().calc(); } }\n",
        );
        put(
            d.path(),
            "s/test/p/SvcTest.java",
            "package p;\nclass SvcTest { public static void main(String[] a) { new Svc().calc(); } }\n",
        );
        put(d.path(), "m/pom.xml", "<project/>");
        put(
            d.path(),
            "m/src/main/java/q/Q.java",
            "package q;\npublic class Q { public void go() {} }\n",
        );
        put(
            d.path(),
            "m/src/test/java/q/QTest.java",
            "package q;\nimport org.junit.jupiter.api.Test;\nclass QTest { @Test void t() { new Q().go(); } }\n",
        );
        let r = report(d.path(), None, &[]);
        assert_eq!(find(&r, "jvm:test:s/build.sh").command, "sh ./build.sh");
        assert_eq!(find(&r, "jvm:test:s/build.sh").cwd, "s");
        assert_eq!(
            find(&r, "jvm:main:s/build.sh:run").command,
            "sh ./build.sh run"
        );
        let q = find(&r, "jvm:test:m/src/test/java/q/QTest.java");
        assert_eq!(q.command, "mvn -q -o test -Dtest=QTest");
        assert_eq!(q.cwd, "m");
        assert_eq!(q.entry_points, ["m/src/main/java/q/Q.java::Q.go"]);
    }

    #[test]
    fn rust_inline_tests_bins_and_integration() {
        let d = tempfile::tempdir().unwrap();
        put(
            d.path(),
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        );
        put(
            d.path(),
            "src/lib.rs",
            "pub fn alpha() {}\npub fn beta() {}\nfn gamma() {}\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn t() { alpha(); }\n}\n",
        );
        put(d.path(), "src/main.rs", "fn main() { demo::beta(); }\n");
        put(d.path(), "tests/it.rs", "fn t() { demo::alpha(); }\n");
        let r = report(d.path(), None, &[]);
        let t = find(&r, "rust:test:Cargo.toml");
        assert_eq!(t.command, "cargo test");
        assert_eq!(t.entry_points, ["src/lib.rs::alpha"]);
        assert_eq!(
            find(&r, "rust:test:tests/it.rs").command,
            "cargo test --test it"
        );
        assert_eq!(
            find(&r, "rust:main:src/main.rs").command,
            "cargo run -q --bin demo"
        );
        assert_eq!(
            find(&r, "rust:main:src/main.rs").entry_points,
            ["src/lib.rs::beta"]
        );
    }

    #[test]
    fn native_make_test_and_binary() {
        let d = tempfile::tempdir().unwrap();
        put(
            d.path(),
            "Makefile",
            "BIN := prog\nall: $(BIN)\n$(BIN): src/main.o\n\tcc -o $@ $^\ntest: t\n\t./t\n",
        );
        put(
            d.path(),
            "src/lib.c",
            "int api(void) { return 1; }\nstatic int local(void) { return 2; }\n",
        );
        put(
            d.path(),
            "src/main.c",
            "int api(void);\nint main(void) { return api(); }\n",
        );
        put(
            d.path(),
            "tests/test_lib.c",
            "int api(void);\nint main(void) { return api() - 1; }\n",
        );
        let r = report(d.path(), None, &[]);
        let t = find(&r, "native:test:Makefile");
        assert_eq!(t.command, "make test");
        assert_eq!(t.file, "tests/test_lib.c");
        assert_eq!(find(&r, "native:main:Makefile").command, "make && ./prog");
        assert_eq!(r.entry_points, ["src/lib.c::api"]);
        assert!(r.not_exercisable.is_empty());
    }

    #[test]
    fn discover_writes_report_evidence_and_coverage() {
        let d = tempfile::tempdir().unwrap();
        go_project(d.path());
        let run_dir = d.path().join(".dla/r1");
        fs::create_dir_all(&run_dir).unwrap();
        let mut st =
            RunState::init(d.path().to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
        st.run_id = "r1".into();
        st.save(&run_dir).unwrap();
        Ledger::create(&run_dir).unwrap();
        let ctx = ToolCtx {
            project: d.path().to_path_buf(),
            run_dir: run_dir.clone(),
            run_id: "r1".into(),
            source_commit: "abc".into(),
            data_flag: None,
        };
        let out = discover(&ctx, Some(Runtime::Go)).unwrap();
        assert_eq!(out.evidence.len(), 1);
        assert_eq!(out.candidates, 0);
        let rep: WorkloadReport = serde_json::from_str(
            &fs::read_to_string(run_dir.join("workload/workloads.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(rep.workloads.len(), 2);
        let cov = CoverageManifest::load(&run_dir).unwrap().unwrap();
        let ep = &cov.workers["go"].entry_points;
        assert_eq!(ep.discovered.len(), 3);
        assert!(ep.exercised.is_empty());
        assert_eq!(ep.not_exercisable.len(), 1);
        let ledger = ctx.ledger().unwrap();
        assert_eq!(ledger.findings().count(), 0);
    }
}
