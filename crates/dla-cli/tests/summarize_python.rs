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
    conforms("py-retain", "heap-graph");
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
    // A Python type never ties, and Python has no heap-graph backend: the text names a holder
    // probe.
    let needed = v["needed"].as_str().unwrap();
    assert!(
        needed.contains("holder probe") && !needed.contains("heap-graph"),
        "{v}"
    );
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

// ---------------------------------------------------------------------------------------
// heap-graph: the holder chains of `--capture heap`
// ---------------------------------------------------------------------------------------

/// A hand-written `py-1.retain.json` with one site of each kind: a module global that holds two
/// lines (one through a stored exception's traceback), a class attribute, a frame, an object
/// nothing refers to, a line no object belongs to, and a chain that reached no root.
const RETAIN: &str = r#"{"dla_pyretain":1,"frames":1,"sites":[
 {"site":"pkg/cms.py:40","blocks_first":100,"blocks_last":600,"found_tracked":600,"found_untracked":0,"kept":40,
  "chains":[{"count":30,"root":{"kind":"module-global","module":"pkg.cms","key":"_native"},
             "steps":[{"type":"pkg.cms.Payload","via":"f_locals.local"},{"type":"frame","via":"tb_frame","at":"pkg/cms.py:44"},
                      {"type":"traceback","via":"__traceback__","at":"pkg/cms.py:44","repeat":598},{"type":"ImportError","via":".ex"},
                      {"type":"pkg._util.Deferred","via":"._native"}]},
            {"count":10,"root":{"kind":"module-global","module":"pkg.cms","key":"_native"},
             "steps":[{"type":"traceback","via":"__traceback__","at":"pkg/cms.py:44","repeat":590},{"type":"ImportError","via":".ex"},
                      {"type":"pkg._util.Deferred","via":"._native"}]}]},
 {"site":"pkg/cms.py:52","blocks_first":50,"blocks_last":300,"found_tracked":300,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"module-global","module":"pkg.cms","key":"_native"},
             "steps":[{"type":"frame","via":"tb_frame","at":"pkg/cms.py:55"},
                      {"type":"traceback","via":"__traceback__","at":"pkg/cms.py:44","repeat":280},{"type":"ImportError","via":".ex"},
                      {"type":"pkg._util.Deferred","via":"._native"}]}]},
 {"site":"pkg/store.py:9","blocks_first":20,"blocks_last":220,"found_tracked":200,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"class-attribute","class":"pkg.store.Store","attribute":"items"},
             "steps":[{"type":"pkg.store.Item","via":"[7]"},{"type":"list","via":".items"}]}]},
 {"site":"pkg/work.py:21","blocks_first":0,"blocks_last":120,"found_tracked":120,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"frame","function":"_worker","at":"pkg/work.py:30"},
             "steps":[{"type":"pkg.work.Job","via":"[3]"},{"type":"list","via":"f_locals.acc"}]}]},
 {"site":"pkg/cb.py:5","blocks_first":10,"blocks_last":110,"found_tracked":100,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"no-referrer"},"steps":[{"type":"pkg.cb.Callback"}]}]},
 {"site":"pkg/font.py:580","blocks_first":200,"blocks_last":1200,"found_tracked":0,"found_untracked":0,"kept":0,
  "note":"no object was found for this site: the blocks do not belong to an object any Python container refers to, so native code holds them or a reference was lost there",
  "chains":[]},
 {"site":"pkg/deep.py:3","blocks_first":10,"blocks_last":60,"found_tracked":50,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"depth-limit"},"steps":[{"type":"int","via":".n"}]}]}
],"limitations":["values are not written: a dict key is shown as its type (DLA_RAW_VALUES=1 allows raw values)"]}"#;

/// The retention-path records of the ledger, newest last.
fn retention(p: &Project) -> Vec<Value> {
    p.evidence("retention-path")
}

fn holder<'a>(rec: &'a Value, key: &str) -> &'a Value {
    rec["result"]["holders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["key"] == key)
        .unwrap_or_else(|| panic!("no holder {key}: {}", rec["result"]))
}

#[test]
fn every_root_kind_becomes_a_holder_with_its_kind_sites_objects_and_path() {
    let p = Project::new();
    let f = p.put("in/py-1.retain.json", RETAIN.as_bytes());
    let v = summarize(&p, "heap-graph", "py-retain", std::slice::from_ref(&f), &[]);
    assert_eq!(v["backend"], "py-retain", "{v}");
    let recs = retention(&p);
    assert_eq!(recs.len(), 1);
    let rec = &recs[0];
    assert_eq!(rec["authority"], "live");
    let hs = rec["result"]["holders"].as_array().unwrap();
    // One holder per distinct root: the module global holds two lines, the others one each; the
    // chain that reached no root names no holder.
    let keys: Vec<&str> = hs.iter().map(|h| h["key"].as_str().unwrap()).collect();
    assert_eq!(
        keys,
        [
            "static:pkg.cms._native",
            "pkg/cb.py:5",
            "stack:_worker@pkg/work.py:30",
            "static:pkg.store.Store.items",
            "pkg/font.py:580"
        ],
        "{}",
        rec["result"]
    );
    let kinds: Vec<&str> = hs
        .iter()
        .map(|h| h["holder_kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        [
            "static/global",
            "native-handle/global-ref",
            "blocked-stack",
            "static/global",
            "native-handle/global-ref"
        ]
    );
    let g = holder(rec, "static:pkg.cms._native");
    assert_eq!(g["kept"], 80);
    let sites: Vec<(&str, u64, u64)> = g["sites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["site"].as_str().unwrap(),
                s["first"].as_u64().unwrap(),
                s["last"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        sites,
        [("pkg/cms.py:40", 100, 600), ("pkg/cms.py:52", 50, 300)]
    );
    // The chain with the most objects, as a path with its links, places and repeats.
    let path: Vec<&str> = g["path"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["type"].as_str().unwrap())
        .collect();
    // The chain with the most objects (40) is the one of the second line.
    assert_eq!(path.first().copied(), Some("frame"));
    assert_eq!(path.last().copied(), Some("pkg._util.Deferred"));
    let tb = g["path"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["type"] == "traceback")
        .unwrap();
    assert_eq!(
        (tb["via"].as_str(), tb["at"].as_str(), tb["repeat"].as_u64()),
        (Some("__traceback__"), Some("pkg/cms.py:44"), Some(280))
    );
    assert!(holder(rec, "pkg/cb.py:5")["unreferenced"].is_string());
    let note = holder(rec, "pkg/font.py:580");
    assert!(
        note["unreferenced"]
            .as_str()
            .unwrap()
            .contains("native code holds them")
    );
    assert_eq!(note["kept"], 0);
    assert_eq!(note["sites"][0]["last"], 1200);
    // The raw file keeps every holder, every chain and the digest of the input.
    let raw: Value = serde_json::from_str(
        &fs::read_to_string(p.run_dir.join(v["raw"]["path"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    assert_eq!(
        raw["captures"][0]["sha256"],
        sha(&fs::read(p.path().join(&f)).unwrap())
    );
    let gp = raw["holders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["key"] == "static:pkg.cms._native")
        .unwrap();
    assert_eq!(gp["paths"].as_array().unwrap().len(), 3);
    assert_eq!(gp["owner"], "pkg.cms._native");
    let font = raw["sites"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["site"] == "pkg/font.py:580")
        .unwrap();
    assert!(
        font["unreferenced"]
            .as_str()
            .unwrap()
            .contains("native code holds them"),
        "{font}"
    );
    assert!(
        rec["limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l.as_str().unwrap().contains("depth limit")),
        "{rec}"
    );
}

#[test]
fn a_holder_that_no_finding_names_gets_the_command_that_adds_one() {
    let p = Project::new();
    let f = p.put("in/py-1.retain.json", RETAIN.as_bytes());
    let v = summarize(&p, "heap-graph", "py-retain", &[f], &[]);
    let id = v["evidence"][0].as_str().unwrap();
    let next: Vec<&str> = v["next"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    // The site is the place of the step nearest the root that has one (the traceback), the owner
    // is the module global, and the holder kind is the one of the root.
    assert_eq!(
        next[0],
        format!(
            "dla ledger add --origin growth-series --evidence {id} --site pkg/cms.py:44 --owner pkg.cms._native --region heap --holder static/global"
        )
    );
    // A holder in native code or a frame has no owner and no step with a place: its finding is
    // at the site that grew, and the kind is the one the holder has.
    assert_eq!(
        next[1],
        format!(
            "dla ledger add --origin growth-series --evidence {id} --site pkg/cb.py:5 --region heap --holder native-handle/global-ref"
        )
    );
    assert_eq!(
        next[2],
        format!(
            "dla ledger add --origin growth-series --evidence {id} --site pkg/work.py:21 --region heap --holder blocked-stack"
        )
    );
    assert_eq!(next.len(), 3, "{next:?}");
}

#[test]
fn a_finding_whose_owner_or_site_names_the_holder_is_tied_and_gets_its_route() {
    let p = Project::new();
    let f = p.put("in/py-1.retain.json", RETAIN.as_bytes());
    let first = summarize(&p, "heap-graph", "py-retain", std::slice::from_ref(&f), &[]);
    let e1 = first["evidence"][0].as_str().unwrap().to_owned();
    // Rule 1: the owner of the finding is the holder's.
    let by_owner = ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &e1,
            "--site",
            "pkg/cms.py:44",
            "--owner",
            "pkg.cms._native",
            "--region",
            "heap",
            "--holder",
            "static/global",
        ],
    );
    let f_owner = by_owner["id"].as_str().unwrap().to_owned();
    // A finding at a site the frame holder holds, with no owner.
    let by_site = ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &e1,
            "--site",
            "pkg/work.py:21",
            "--region",
            "heap",
            "--holder",
            "blocked-stack",
        ],
    );
    let f_site = by_site["id"].as_str().unwrap().to_owned();
    // A non-leak sibling that carries the owner of another holder and a site it keeps ties nothing.
    ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f_owner,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            &e1,
            "--owner",
            "pkg.store.Store.items",
            "--site",
            "pkg/store.py:9",
        ],
    );
    let g = p.put("in/second/py-1.retain.json", RETAIN.as_bytes());
    let v = summarize(&p, "heap-graph", "py-retain", &[g], &[]);
    let rec = retention(&p).pop().unwrap();
    let ties = rec["result"]["ties"].as_array().unwrap();
    let tie_of = |key: &str| {
        ties.iter()
            .find(|t| t["holder"] == key)
            .unwrap_or_else(|| panic!("no tie for {key}: {ties:?}"))
    };
    let t = tie_of("static:pkg.cms._native");
    assert_eq!(t["finding"], f_owner.as_str());
    assert_eq!(t["tie"]["rule"], "holder");
    let t = tie_of("stack:_worker@pkg/work.py:30");
    assert_eq!(t["finding"], f_site.as_str());
    assert_eq!(t["tie"]["rule"], "site");
    assert!(
        ties.iter()
            .all(|t| t["holder"] != "static:pkg.store.Store.items"),
        "{ties:?}"
    );
    let named: Vec<&str> = rec["result"]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert_eq!(
        named,
        [f_owner.as_str(), f_site.as_str()]
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    );
    // The holders the findings name come first, and no add command is printed for them.
    let next: Vec<&str> = v["next"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert!(!next.is_empty());
    for c in &next {
        assert!(
            !c.contains("--site pkg/cms.py:44") && !c.contains("--site pkg/work.py"),
            "{next:?}"
        );
    }
    assert!(
        next.iter()
            .any(|c| c.contains(&f_owner) || c.contains(&f_site)),
        "{next:?}"
    );
    // The same label rule the other retention records get: the record names the finding.
    let lines = p.lines();
    let finding = lines
        .iter()
        .find(|l| l["type"] == "finding_added" && l["finding"]["id"] == f_owner.as_str())
        .unwrap();
    assert_eq!(finding["finding"]["owner"], "pkg.cms._native");
}

#[test]
fn a_file_that_is_not_a_holder_chains_file_is_refused_with_the_fix() {
    let p = Project::new();
    let bad = p.put("in/r.json", b"{\"dla_pyretain\":null,\"sites\":[]}");
    let e = refused(
        p.path(),
        &["summarize", "heap-graph", "--backend", "py-retain", &bad],
    );
    assert_eq!(e["code"], "malformed-artifact");
    assert!(e["fix"].as_str().unwrap().contains("--capture heap"));
    let torn = p.put("in/torn.json", b"{\"dla_pyretain\":1,\"sites\":[{\"site\":");
    let e = refused(
        p.path(),
        &["summarize", "heap-graph", "--backend", "py-retain", &torn],
    );
    assert_eq!(e["code"], "malformed-artifact");
}

#[test]
fn auto_reads_a_holder_chains_file_and_a_values_name_is_a_shape() {
    let p = Project::new();
    let tricky = RETAIN.replace(
        "\"via\":\"f_locals.local\"",
        "\"via\":\"[\\\"AKIAIOSFODNN7EXAMPLE1\\\"]\"",
    );
    let f = p.put("in/py-1.retain.json", tricky.as_bytes());
    let v = ok(p.path(), &["summarize", "heap-graph", &f]);
    assert_eq!(v["backend"], "py-retain");
    for (path, t) in p.run_texts() {
        assert!(
            !t.contains(SECRET) || path.ends_with("py-1.retain.json"),
            "secret in {}",
            path.display()
        );
    }
}

#[test]
fn a_file_of_one_dump_point_is_read_and_the_record_says_growth_was_not_computed() {
    let p = Project::new();
    let one = r#"{"dla_pyretain":1,"frames":1,"sites":[
 {"site":"pkg/a.py:3","blocks_first":null,"blocks_last":50,"found_tracked":40,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"module-global","module":"pkg.a","key":"_reg"},"steps":[{"type":"pkg.a.E","via":"[0]"},{"type":"list","via":"._reg"}]}]}],
 "limitations":[]}"#;
    let f = p.put("in/py-1.retain.json", one.as_bytes());
    let v = summarize(&p, "heap-graph", "py-retain", &[f], &[]);
    let rec = retention(&p).pop().unwrap();
    let h = holder(&rec, "static:pkg.a._reg");
    assert!(h["sites"][0]["first"].is_null(), "{h}");
    assert_eq!(h["sites"][0]["last"], 50);
    assert!(
        rec["limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l.as_str().unwrap().contains("growth was not computed")),
        "{rec}"
    );
    // The holder still gets its command and its ties work as with two dump points.
    assert!(
        v["next"][0]
            .as_str()
            .unwrap()
            .contains("--owner pkg.a._reg"),
        "{v}"
    );
}

#[test]
fn sites_with_kept_objects_and_no_chain_are_counted_with_the_reason_of_the_file() {
    let p = Project::new();
    let cut = r#"{"dla_pyretain":1,"frames":1,"sites":[
 {"site":"pkg/a.py:3","blocks_first":0,"blocks_last":50,"found_tracked":40,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"module-global","module":"pkg.a","key":"_reg"},"steps":[{"type":"int"}]}]},
 {"site":"pkg/a.py:4","blocks_first":0,"blocks_last":50,"found_tracked":40,"found_untracked":0,"kept":40,"chains":[]},
 {"site":"pkg/a.py:5","blocks_first":0,"blocks_last":50,"found_tracked":40,"found_untracked":0,"kept":40,"chains":[]}],
 "limitations":["the time bound (60 s) stopped the walk: 80 kept objects have no chain"]}"#;
    let f = p.put("in/py-1.retain.json", cut.as_bytes());
    let v = summarize(&p, "heap-graph", "py-retain", &[f], &[]);
    assert_eq!(v["no_chain_sites"], 2, "{v}");
    assert!(
        v["no_chain_why"].as_str().unwrap().contains("time bound"),
        "{v}"
    );
    let rec = retention(&p).pop().unwrap();
    assert_eq!(rec["result"]["no_chain"]["sites"], 2);
    assert!(
        rec["result"]["no_chain"]["why"]
            .as_str()
            .unwrap()
            .contains("time bound"),
        "{rec}"
    );
    assert!(
        rec["limitations"].as_array().unwrap().iter().any(|l| l
            .as_str()
            .unwrap()
            .contains("2 site(s) have kept objects and no chain")),
        "{rec}"
    );
}

#[test]
fn the_record_lists_the_types_a_holder_keeps_and_its_minor_sites() {
    let p = Project::new();
    let f = p.put("in/py-1.retain.json", RETAIN.as_bytes());
    summarize(&p, "heap-graph", "py-retain", &[f], &[]);
    let rec = retention(&p).pop().unwrap();
    let g = holder(&rec, "static:pkg.cms._native");
    let types: Vec<&str> = g["types"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t.as_str())
        .collect();
    // The kept object of each chain and the step that repeats; not the links in between.
    assert!(
        types.contains(&"traceback") && types.contains(&"frame"),
        "{types:?}"
    );
    assert!(
        !types.contains(&"ImportError") && !types.contains(&"pkg._util.Deferred"),
        "{types:?}"
    );
    // A site where a holder keeps a few of the objects sampled is a minor one.
    let minor = r#"{"dla_pyretain":1,"frames":1,"sites":[
 {"site":"pkg/m.py:1","blocks_first":0,"blocks_last":50,"found_tracked":40,"found_untracked":0,"kept":40,
  "chains":[{"count":36,"root":{"kind":"module-global","module":"pkg.m","key":"big"},"steps":[{"type":"int"}]},
            {"count":4,"root":{"kind":"module-global","module":"pkg.m","key":"small"},"steps":[{"type":"int"}]}]}],"limitations":[]}"#;
    let f = p.put("in/second.retain.json", minor.as_bytes());
    summarize(&p, "heap-graph", "py-retain", &[f], &[]);
    let rec = retention(&p).pop().unwrap();
    let small = holder(&rec, "static:pkg.m.small");
    assert_eq!(small["sites"].as_array().unwrap().len(), 0, "{small}");
    assert_eq!(small["minor_sites"][0], "pkg/m.py:1");
    assert_eq!(
        holder(&rec, "static:pkg.m.big")["sites"][0]["site"],
        "pkg/m.py:1"
    );
}

#[test]
fn an_owner_the_record_names_with_a_site_it_does_not_keep_is_warned_about_and_not_refused() {
    let p = Project::new();
    let f = p.put("in/py-1.retain.json", RETAIN.as_bytes());
    let v = summarize(&p, "heap-graph", "py-retain", &[f], &[]);
    let e = v["evidence"][0].as_str().unwrap().to_owned();
    let add = |site: &str| {
        ok(
            p.path(),
            &[
                "ledger",
                "add",
                "--origin",
                "growth-series",
                "--evidence",
                &e,
                "--site",
                site,
                "--owner",
                "pkg.cms._native",
                "--region",
                "heap",
                "--holder",
                "static/global",
            ],
        )
    };
    let outside = add("pkg/driver.py:97");
    assert_eq!(outside["state"], "candidate", "not refused: {outside}");
    let w = outside["warnings"]
        .as_array()
        .unwrap_or_else(|| panic!("{outside}"))
        .iter()
        .filter_map(|x| x.as_str())
        .find(|x| x.contains("measured column"))
        .unwrap_or_else(|| panic!("{outside}"));
    assert!(
        w.contains("pkg/cms.py:40") && w.contains("pkg/cms.py:52") && w.contains("measured column"),
        "{w}"
    );
    // A line the holder keeps gets no such warning.
    let held = add("pkg/cms.py:40");
    assert!(
        held["warnings"].as_array().is_none_or(|a| a
            .iter()
            .all(|x| !x.as_str().unwrap().contains("measured column"))),
        "{held}"
    );
}
