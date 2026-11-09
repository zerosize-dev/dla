//! `dla summarize` over Python data: tracemalloc stacks and memray reports (alloc-stack-diff),
//! object counts by type (type-histogram-diff) and threads, tasks and ResourceWarnings by
//! creation site (non-heap). The inputs are real output of the Python harness agent, trimmed
//! (tests/fixtures/summarize/py-*).
//!
//! The adapters are also covered by `summarize_conformance.rs`; the checks here run each
//! adapter by name too, so they hold when `summarize --list` outgrows its output bound.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn run(project: &Path, args: &[&str]) -> (i32, String, Value) {
    let out = dla(project).args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), text, v)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, _, v) = run(project, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let (code, _, v) = run(project, args);
    assert_eq!(code, 2, "{args:?}: {v}");
    let e = &v["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "no {k}: {v}");
    }
    e.clone()
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/summarize")
        .join(name)
}

fn sources() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/py-sources")
}

struct Project {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let init = ok(dir.path(), &["run", "init", "--session-id", "t"]);
        let run_dir = dir
            .path()
            .join(".dla")
            .join(init["run_id"].as_str().unwrap());
        Self { dir, run_dir }
    }

    /// A project with the python sources of the fixtures and their census.
    fn with_sources() -> Self {
        let p = Self::new();
        fn copy(from: &Path, to: &Path) {
            fs::create_dir_all(to).unwrap();
            for e in fs::read_dir(from).unwrap().flatten() {
                let (src, dst) = (e.path(), to.join(e.file_name()));
                if src.is_dir() {
                    copy(&src, &dst);
                } else {
                    fs::copy(&src, &dst).unwrap();
                }
            }
        }
        copy(&sources(), p.path());
        ok(p.path(), &["census", "--lang", "python"]);
        p
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn put(&self, rel: &str, bytes: &[u8]) -> String {
        let p = self.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
        rel.to_owned()
    }

    /// Copies fixture files of `dir` into `in/<sub>/`.
    fn put_fixture(&self, dir: &str, names: &[&str], sub: &str) -> Vec<String> {
        names
            .iter()
            .map(|n| {
                let bytes = fs::read(fixture(dir).join(n)).unwrap();
                assert!(bytes.len() < 64 * 1024, "{n} is over 64 KB");
                self.put(&format!("in/{sub}/{n}"), &bytes)
            })
            .collect()
    }

    fn lines(&self) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn evidence(&self, kind: &str) -> Vec<Value> {
        self.lines()
            .into_iter()
            .filter(|e| e["type"] == "evidence" && e["kind"] == kind)
            .collect()
    }

    fn finding_with_owner(&self, owner: &str, site: &str) -> Value {
        self.lines()
            .into_iter()
            .filter(|e| e["type"] == "finding_added")
            .map(|e| e["finding"].clone())
            .find(|f| f["owner"] == owner && f["site"] == site)
            .unwrap_or_else(|| panic!("no finding {owner} at {site}"))
    }

    fn coverage(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir.join("coverage.json")).unwrap())
            .unwrap()
    }

    fn run_texts(&self) -> Vec<(PathBuf, String)> {
        fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
            for e in fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if let Ok(t) = fs::read_to_string(&p) {
                    out.push((p, t));
                }
            }
        }
        let mut v = Vec::new();
        walk(&self.run_dir, &mut v);
        v
    }
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

const SECRET: &str = "AKIAIOSFODNN7EXAMPLE1";

const ALLOC: [&str; 4] = [
    "alloc-1.json",
    "alloc-2.json",
    "alloc-3.json",
    "alloc-4.json",
];
const MEMRAY: [&str; 4] = [
    "memray-1.json",
    "memray-2.json",
    "memray-3.json",
    "memray-4.json",
];
const TYPES: [&str; 4] = [
    "types-1.json",
    "types-2.json",
    "types-3.json",
    "types-4.json",
];
const RES: [&str; 3] = ["resources-1.json", "resources-2.json", "resources-3.json"];

fn summarize(p: &Project, op: &str, backend: &str, files: &[String], extra: &[&str]) -> Value {
    let mut args = vec!["summarize", op, "--backend", backend];
    args.extend_from_slice(extra);
    args.extend(files.iter().map(String::as_str));
    ok(p.path(), &args)
}

/// True when a passing column of the growth evidence is tied to `id` the way the ledger's
/// confirm rule looks for it.
fn ledger_rule_ties(e: &Value, id: &str, site: Option<&str>, owner: Option<&str>) -> bool {
    let summary = dla_core::trend::GrowthSummary::from_result(&e["result"]).unwrap();
    assert_eq!(e["result"]["gate"], "pass");
    let fid: dla_core::FindingId = id.parse().unwrap();
    summary
        .passed
        .iter()
        .any(|c| c.tied_to_owner(&fid, site, owner))
}

// ---------------------------------------------------------------------------------------
// conformance of the four adapters, by name
// ---------------------------------------------------------------------------------------

fn conforms(backend: &str, op: &str) {
    let fx: Value =
        serde_json::from_str(&fs::read_to_string(fixture(backend).join("fixture.json")).unwrap())
            .unwrap();
    assert_eq!(fx["op"], op);
    let names: Vec<String> = fx["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().to_owned())
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let p = Project::new();
    let files = p.put_fixture(backend, &names, "x");
    let mut args = vec!["summarize", op, "--backend", backend];
    args.extend(files.iter().map(String::as_str));
    let (code, text, v) = run(p.path(), &args);
    assert_eq!(code, 0, "{backend}: {v}");
    assert_eq!(v["backend"], backend);
    // The raw file is where `raw` says, with the hash it says.
    let rel = v["raw"]["path"].as_str().unwrap();
    assert!(!rel.starts_with('/') && !rel.contains(".."), "{rel}");
    let bytes = fs::read(p.run_dir.join(rel)).unwrap();
    assert_eq!(v["raw"]["sha256"], sha(&bytes));
    // Every evidence: within the bound, with a signal and a raw file.
    let lines = p.lines();
    let ids = v["evidence"].as_array().unwrap();
    assert!(!ids.is_empty());
    for id in ids {
        let e = lines
            .iter()
            .find(|l| l["type"] == "evidence" && l["evidence_id"] == *id)
            .unwrap();
        assert!(serde_json::to_string(&e["result"]).unwrap().len() <= 2048);
        for k in ["tool", "class", "source_sha256"] {
            assert!(
                e["result"]["signal"][k]
                    .as_str()
                    .is_some_and(|s| !s.is_empty())
            );
        }
        assert!(p.run_dir.join(e["raw"].as_str().unwrap()).is_file());
    }
    // The planted secret is nowhere without raw-values consent.
    assert!(!text.contains(SECRET), "{backend}: secret in stdout");
    for (path, t) in p.run_texts() {
        assert!(
            !t.contains(SECRET),
            "{backend}: secret in {}",
            path.display()
        );
    }
    // A coverage row: it ran.
    let cov = p.coverage();
    let row = cov["workers"]
        .as_object()
        .unwrap()
        .values()
        .find_map(|w| w["backends"].get(backend))
        .unwrap_or_else(|| panic!("{backend}: no coverage row"));
    assert_eq!(row["status"], "ran");
    // With consent the planted name shows in the raw files (so the check above can fail).
    let q = Project::new();
    ok(q.path(), &["consent", "--tiers", "1", "--raw-values"]);
    let files = q.put_fixture(backend, &names, "x");
    let mut args = vec!["summarize", op, "--backend", backend];
    args.extend(files.iter().map(String::as_str));
    let v = ok(q.path(), &args);
    assert_eq!(v["redaction"].as_str().unwrap_or("raw"), "raw");
    assert!(
        q.run_texts().iter().any(|(_, t)| t.contains(SECRET)),
        "{backend}: with consent the planted value should be in the raw file"
    );
}

#[test]
fn the_python_adapters_conform() {
    conforms("py-tracemalloc", "alloc-stack-diff");
    conforms("py-memray", "alloc-stack-diff");
    conforms("py-types", "type-histogram-diff");
    conforms("py-resources", "non-heap");
}

#[test]
fn auto_picks_the_python_adapters_by_format() {
    let p = Project::new();
    let a = p.put_fixture("py-tracemalloc", &ALLOC, "a");
    let m = p.put_fixture("py-memray", &MEMRAY, "m");
    let t = p.put_fixture("py-types", &TYPES, "t");
    let r = p.put_fixture("py-resources", &RES, "r");
    for (op, files, backend) in [
        ("alloc-stack-diff", &a, "py-tracemalloc"),
        ("alloc-stack-diff", &m, "py-memray"),
        ("type-histogram-diff", &t, "py-types"),
        ("non-heap", &r, "py-resources"),
    ] {
        let mut args = vec!["summarize", op];
        args.extend(files.iter().map(String::as_str));
        assert_eq!(ok(p.path(), &args)["backend"], backend, "{op}");
    }
}

// ---------------------------------------------------------------------------------------
// alloc-stack-diff
// ---------------------------------------------------------------------------------------

#[test]
fn tracemalloc_stacks_grow_and_show_their_first_project_frame() {
    let p = Project::with_sources();
    let files = p.put_fixture("py-tracemalloc", &ALLOC, "a");
    let v = summarize(&p, "alloc-stack-diff", "py-tracemalloc", &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["captures"], 4);
    assert_eq!(v["live"], true);
    assert_eq!(v["top"][0]["site"], "ingest/watchers.py:8");
    assert_eq!(v["top"][0]["at"], "ingest.watchers");
    assert!(
        v["passed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "stack:ingest.watchers"),
        "{v}"
    );
    // No census finding sits at that line, so nothing is tied and the output says why.
    assert_eq!(v["tied"], json!([]), "{v}");
    assert!(
        v["needed"]
            .as_str()
            .unwrap()
            .contains("first project frame")
    );
}

/// The fixture's files with the first stack's frames replaced by `frames`.
fn with_first_stack(p: &Project, frames: Value) -> Vec<String> {
    ALLOC
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let mut doc: Value = serde_json::from_str(
                &fs::read_to_string(fixture("py-tracemalloc").join(n)).unwrap(),
            )
            .unwrap();
            doc["stacks"][0]["frames"] = frames.clone();
            p.put(
                &format!("in/tie/alloc-{}.json", i + 1),
                doc.to_string().as_bytes(),
            )
        })
        .collect()
}

#[test]
fn a_stack_ties_to_the_finding_whose_writer_is_its_first_project_frame() {
    let p = Project::with_sources();
    let registry = p.finding_with_owner("ingest.registry._LISTENERS", "ingest/registry.py:3");
    let f = registry["id"].as_str().unwrap().to_owned();
    // The census records registry.py:7 (`.append`) as a writer of the listener table.
    let files = with_first_stack(
        &p,
        json!([
            ["ingest/registry.py", 7],
            ["ingest/pipeline.py", 28],
            ["<frozen runpy>", 88]
        ]),
    );
    let v = summarize(&p, "alloc-stack-diff", "py-tracemalloc", &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([f]), "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    let col = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == "stack:ingest.registry")
        .unwrap_or_else(|| panic!("{g}"));
    assert_eq!(col["tie"]["rule"], "site");
    assert!(ledger_rule_ties(
        &g,
        &f,
        registry["site"].as_str(),
        Some("ingest.registry._LISTENERS")
    ));
    // A stack ties by site only: no writer is learned from it.
    let notes: Vec<Value> = p
        .lines()
        .into_iter()
        .filter(|l| l["type"] == "finding_annotated" && l["finding"] == f.as_str())
        .collect();
    assert_eq!(notes.len(), 1, "{notes:?}");
}

#[test]
fn finding_narrows_a_tie_and_forces_nothing() {
    let p = Project::with_sources();
    let other = p.finding_with_owner("ingest.parser._newest", "ingest/parser.py:3");
    let o = other["id"].as_str().unwrap().to_owned();
    let files = with_first_stack(&p, json!([["ingest/registry.py", 7]]));
    let v = summarize(
        &p,
        "alloc-stack-diff",
        "py-tracemalloc",
        &files,
        &["--finding", &o],
    );
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([]), "{v}");
    assert!(v["needed"].as_str().unwrap().contains(&o), "{v}");
}

#[test]
fn a_root_level_file_is_a_project_site() {
    let p = Project::with_sources();
    let f = p.finding_with_owner("gateway.open_session", "gateway.py:24");
    let id = f["id"].as_str().unwrap().to_owned();
    let files = with_first_stack(&p, json!([["gateway.py", 24], ["gateway.py", 40]]));
    let v = summarize(&p, "alloc-stack-diff", "py-tracemalloc", &files, &[]);
    assert_eq!(v["top"][0]["site"], "gateway.py:24", "{v}");
    assert_eq!(v["tied"], json!([id]), "{v}");
}

#[test]
fn memray_reports_name_functions_and_the_same_sites() {
    let p = Project::with_sources();
    let files = p.put_fixture("py-memray", &MEMRAY, "m");
    let v = summarize(&p, "alloc-stack-diff", "py-memray", &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["top"][0]["site"], "ingest/watchers.py:8", "{v}");
    assert_eq!(v["top"][0]["at"], "ingest.watchers.__init__");
    // One report is the leak-at-exit view: the evidence, and what a series needs.
    let one = summarize(&p, "alloc-stack-diff", "py-memray", &files[..1], &[]);
    assert_eq!(one["mode"], "single");
    assert!(
        one["needed"].as_str().unwrap().contains("profiles"),
        "{one}"
    );
}

#[test]
fn a_tracemalloc_file_and_a_memray_report_are_not_mixed() {
    let p = Project::new();
    let a = p.put_fixture("py-tracemalloc", &ALLOC[..1], "a");
    let m = p.put_fixture("py-memray", &MEMRAY[..1], "m");
    let both = [a[0].clone(), m[0].clone()];
    let mut args = vec!["summarize", "alloc-stack-diff"];
    args.extend(both.iter().map(String::as_str));
    let (code, _, v) = run(p.path(), &args);
    assert_eq!(code, 2, "{v}");
    // Forcing one adapter on the other's file names the fix.
    let e = refused(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "py-memray",
            &a[0],
        ],
    );
    assert_eq!(e["code"], "unsupported-format");
    assert!(e["message"].as_str().unwrap().contains("py-memray"), "{e}");
}

// ---------------------------------------------------------------------------------------
// type-histogram-diff
// ---------------------------------------------------------------------------------------

#[test]
fn growing_types_pass_and_tie_to_nothing_without_a_recorded_accumulation_class() {
    let p = Project::with_sources();
    let files = p.put_fixture("py-types", &TYPES, "t");
    let v = summarize(&p, "type-histogram-diff", "py-types", &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    let passed = v["passed"].as_array().unwrap();
    assert!(
        passed.iter().any(|c| c == "type:ingest.models.Record"),
        "{v}"
    );
    assert_eq!(v["tied"], json!([]), "{v}");
    assert!(v["needed"].as_str().unwrap().contains("heap-graph"), "{v}");
    assert_eq!(v["top"][0]["type"], "ingest.models.Record");
}

// ---------------------------------------------------------------------------------------
// non-heap
// ---------------------------------------------------------------------------------------

#[test]
fn threads_tasks_and_unclosed_files_tie_to_the_finding_at_their_creation_site() {
    let p = Project::with_sources();
    let t = p.finding_with_owner("gateway.open_session", "gateway.py:24");
    let k = p.finding_with_owner("gateway.open_session", "gateway.py:27");
    let w = p.finding_with_owner("gateway.open_session", "gateway.py:28");
    let ids: Vec<String> = [&t, &k, &w]
        .iter()
        .map(|f| f["id"].as_str().unwrap().to_owned())
        .collect();
    let files = p.put_fixture("py-resources", &RES, "r");
    let v = summarize(&p, "non-heap", "py-resources", &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    let tied: Vec<&str> = v["tied"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    for id in &ids {
        assert!(tied.contains(&id.as_str()), "{id} not in {v}");
    }
    let g = p.evidence("growth-series").pop().unwrap();
    let cols = g["result"]["passed"].as_array().unwrap();
    for (prefix, site) in [
        ("group:thread/name:heartbeat-*", "gateway.py:24"),
        ("group:task/coro:_watch", "gateway.py:27"),
        ("group:handle/warn:unclosed file", "gateway.py:28"),
    ] {
        let c = cols
            .iter()
            .find(|c| c["column"].as_str().unwrap().starts_with(prefix))
            .unwrap_or_else(|| panic!("no {prefix} in {g}"));
        assert_eq!(c["tie"]["rule"], "site", "{prefix}");
        assert_eq!(c["site"], site, "{prefix}");
    }
    // The whole-process counts tie to nothing.
    assert!(
        cols.iter()
            .filter(|c| c["column"].as_str().unwrap().starts_with("metric:"))
            .all(|c| c.get("finding").is_none())
    );
}

#[test]
fn a_thread_name_that_looks_like_a_value_is_a_shape() {
    let p = Project::new();
    let files: Vec<String> = (1..=3)
        .map(|i| {
            let text = json!({
                "dla_py_resources": 1, "pid": 100 + i,
                "threads": {"total": i, "groups": [
                    {"pattern": SECRET, "site": null, "count": i},
                    {"pattern": "pool-*-worker", "site": null, "count": i}]},
                "tasks": {"total": 0, "debug": false, "groups": []},
                "resource_warnings": {"total": 0, "groups": []}
            });
            p.put(&format!("in/r/r{i}.json"), text.to_string().as_bytes())
        })
        .collect();
    let (code, text, v) = run(
        p.path(),
        &[
            "summarize",
            "non-heap",
            "--backend",
            "py-resources",
            &files[0],
            &files[1],
            &files[2],
        ],
    );
    assert_eq!(code, 0, "{v}");
    assert!(!text.contains(SECRET), "{text}");
    for (path, t) in p.run_texts() {
        assert!(!t.contains(SECRET), "{}", path.display());
    }
    let raw = fs::read_to_string(p.run_dir.join(v["raw"]["path"].as_str().unwrap())).unwrap();
    assert!(raw.contains("name:pool-*-worker"), "{raw}");
}

#[test]
fn a_resources_file_without_its_version_is_refused_with_the_fix() {
    let p = Project::new();
    let bad = p.put("in/r.json", b"{\"dla_py_resources\":null,\"threads\":{}}");
    let e = refused(
        p.path(),
        &["summarize", "non-heap", "--backend", "py-resources", &bad],
    );
    assert_eq!(e["code"], "malformed-artifact");
    assert!(e["fix"].as_str().unwrap().contains("--capture nonheap"));
}

#[test]
fn damaged_files_are_refused_for_every_python_adapter() {
    let p = Project::new();
    let torn = p.put("in/torn.json", b"{\"dla_pyalloc\":1,\"stacks\":[{\"size\":");
    let e = refused(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "py-tracemalloc",
            &torn,
        ],
    );
    assert_eq!(e["code"], "malformed-artifact");
    let torn = p.put("in/torn2.json", b"{\"dla_pytypes\":1,\"types\":[");
    let e = refused(
        p.path(),
        &[
            "summarize",
            "type-histogram-diff",
            "--backend",
            "py-types",
            &torn,
        ],
    );
    assert!(e["fix"].as_str().unwrap().contains("--capture histogram"));
}

#[test]
fn two_files_give_the_evidence_and_say_what_is_missing() {
    let p = Project::new();
    let files = p.put_fixture("py-types", &TYPES[..2], "t");
    let v = summarize(&p, "type-histogram-diff", "py-types", &files, &[]);
    assert_eq!(v["gate"], "fail", "{v}");
    assert!(v["needed"].as_str().unwrap().contains("3"), "{v}");
}
