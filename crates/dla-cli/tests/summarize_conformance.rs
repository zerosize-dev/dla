//! Conformance of every `dla summarize` adapter, and the ties and attribution the
//! framework adds.
//!
//! The conformance test iterates the registry (`dla summarize --list`). For each adapter
//! it reads `tests/fixtures/summarize/<backend>/fixture.json` and checks: stdout of at most
//! 2048 bytes; `raw` present, existing, and its sha256 right; every evidence result within
//! the bound and carrying a `signal`; no planted secret in stdout, the ledger or any file
//! of the run without raw-values consent (and the secret present with it, so the check can
//! fail); a coverage row after the run, `missing` with an install hint when the tool is
//! absent. A meta-test fails when an adapter has no fixture, so an adapter added later
//! must bring one.

#[path = "../../dla-parsers/tests/common/hprof_builder.rs"]
mod hprof_builder;

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use hprof_builder::model;

const T0: u64 = 1_800_000_000_000;

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

fn run_with(project: &Path, args: &[&str], env: &[(&str, Option<&str>)]) -> (i32, String, Value) {
    let mut c = dla(project);
    c.args(args);
    for (k, v) in env {
        match v {
            Some(v) => c.env(k, v),
            None => c.env_remove(k),
        };
    }
    let out = c.output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), text, v)
}

fn run(project: &Path, args: &[&str]) -> (i32, Value) {
    let (code, _, v) = run_with(project, args, &[]);
    (code, v)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!(code, 2, "{args:?}: {v}");
    let e = &v["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "no {k}: {v}");
    }
    e.clone()
}

/// True when a passing column of the growth evidence `e` is tied to the finding `id` the
/// way the ledger's confirm rule looks for (finding, site, candidates, or owner).
fn ledger_rule_ties(e: &Value, id: &str, site: Option<&str>, owner: Option<&str>) -> bool {
    let summary = dla_core::trend::GrowthSummary::from_result(&e["result"]).unwrap();
    assert_eq!(e["result"]["gate"], "pass");
    let fid: dla_core::FindingId = id.parse().unwrap();
    summary
        .passed
        .iter()
        .any(|c| c.tied_to_owner(&fid, site, owner))
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

struct Project {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl Project {
    fn new() -> Self {
        Self::with_init(&[])
    }

    fn with_init(extra: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        // An artifact run needs its artifacts to exist.
        for d in ["dumps", "hist"] {
            if extra.contains(&d) {
                fs::create_dir_all(dir.path().join(d)).unwrap();
                fs::write(dir.path().join(d).join(".keep"), "").unwrap();
            }
        }
        let mut args = vec!["run", "init", "--session-id", "t"];
        args.extend_from_slice(extra);
        let init = ok(dir.path(), &args);
        let run_dir = dir
            .path()
            .join(".dla")
            .join(init["run_id"].as_str().unwrap());
        Self { dir, run_dir }
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

    fn ledger_lines(&self) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn evidence(&self, kind: &str) -> Vec<Value> {
        self.ledger_lines()
            .into_iter()
            .filter(|e| e["type"] == "evidence" && e["kind"] == kind)
            .collect()
    }

    fn findings(&self) -> Vec<Value> {
        self.ledger_lines()
            .into_iter()
            .filter(|e| e["type"] == "finding_added")
            .map(|e| e["finding"].clone())
            .collect()
    }

    fn coverage(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir.join("coverage.json")).unwrap())
            .unwrap()
    }

    /// Every file under the run directory, as text.
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

// ---------------------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------------------

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/summarize")
}

struct Fixture {
    backend: String,
    dir: PathBuf,
    meta: Value,
}

impl Fixture {
    fn load(backend: &str) -> Option<Self> {
        let dir = fixtures_dir().join(backend);
        let text = fs::read_to_string(dir.join("fixture.json")).ok()?;
        Some(Self {
            backend: backend.to_owned(),
            dir,
            meta: serde_json::from_str(&text).ok()?,
        })
    }

    fn op(&self) -> String {
        self.meta["op"].as_str().unwrap().to_owned()
    }

    fn secrets(&self) -> Vec<String> {
        self.meta["secrets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect()
    }

    /// Puts the inputs into the project; returns their project-relative paths.
    fn materialize(&self, p: &Project) -> Vec<String> {
        if self.meta["generate"] == "hprof" {
            let secret = self.secrets()[0].clone();
            assert_eq!(secret.len(), "interned".len(), "same-length replacement");
            return (0..3)
                .map(|i| {
                    let m = model(8, T0 + 30_000 * i as u64, [200, 400, 600][i], 5);
                    let mut dump = m.dump;
                    let at = dump
                        .windows(8)
                        .position(|w| w == b"interned")
                        .expect("the model holds an interned string");
                    dump[at..at + 8].copy_from_slice(secret.as_bytes());
                    p.put(&format!("in/{}-{i}.hprof", self.backend), &dump)
                })
                .collect();
        }
        self.meta["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| {
                let n = n.as_str().unwrap();
                let bytes = fs::read(self.dir.join(n)).unwrap();
                assert!(bytes.len() < 64 * 1024, "{n} is over 64 KB");
                p.put(&format!("in/{n}"), &bytes)
            })
            .collect()
    }
}

fn adapters() -> Vec<Value> {
    let d = tempfile::tempdir().unwrap();
    let v = ok(d.path(), &["summarize", "--list"]);
    v["adapters"].as_array().unwrap().clone()
}

fn summarize_args<'a>(op: &'a str, backend: &'a str, files: &'a [String]) -> Vec<&'a str> {
    let mut a = vec!["summarize", op, "--backend", backend];
    a.extend(files.iter().map(String::as_str));
    a
}

#[test]
fn every_adapter_has_a_conformance_fixture() {
    let list = adapters();
    assert!(list.len() >= 6, "{list:?}");
    for a in &list {
        let name = a["backend"].as_str().unwrap();
        let fx = Fixture::load(name).unwrap_or_else(|| {
            panic!(
                "adapter {name} has no conformance fixture: add tests/fixtures/summarize/{name}/fixture.json (op, inputs or generate, secrets) with its input files"
            )
        });
        assert_eq!(fx.op(), a["op"].as_str().unwrap(), "{name}: fixture op");
        assert!(
            fx.meta["inputs"].is_array() || fx.meta["generate"].is_string(),
            "{name}: inputs or generate"
        );
        assert!(
            !fx.secrets().is_empty(),
            "{name}: a planted secret is required"
        );
    }
    // And no fixture without an adapter.
    for e in fs::read_dir(fixtures_dir()).unwrap().flatten() {
        let n = e.file_name().to_string_lossy().into_owned();
        assert!(
            list.iter().any(|a| a["backend"] == n.as_str()),
            "fixture {n} names no adapter"
        );
    }
}

fn check_bounded_outputs(p: &Project, text: &str, v: &Value) {
    assert!(text.len() <= 2048, "{}", text.len());
    // `raw` is a run-relative pointer and the sha256 of the file it names.
    let raw = &v["raw"];
    let rel = raw["path"]
        .as_str()
        .unwrap_or_else(|| panic!("no raw: {v}"));
    assert!(!rel.starts_with('/') && !rel.contains(".."), "{rel}");
    let bytes = fs::read(p.run_dir.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
    assert_eq!(raw["sha256"], sha(&bytes), "sha256 of {rel}");
    // Every evidence it wrote: result within the bound, a signal, an existing raw.
    let ids: Vec<&str> = v["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_str().unwrap())
        .collect();
    assert!(!ids.is_empty(), "{v}");
    let lines = p.ledger_lines();
    for id in ids {
        let e = lines
            .iter()
            .find(|l| l["type"] == "evidence" && l["evidence_id"] == id)
            .unwrap_or_else(|| panic!("{id} is not in the ledger"));
        let size = serde_json::to_string(&e["result"]).unwrap().len();
        assert!(size <= 2048, "{id}: result is {size} bytes");
        let s = &e["result"]["signal"];
        for k in ["tool", "class", "source_sha256"] {
            assert!(
                s[k].as_str().is_some_and(|x| !x.is_empty()),
                "{id}: signal {k}"
            );
        }
        assert_eq!(s["source_sha256"].as_str().unwrap().len(), 64, "{id}");
        let raw = e["raw"].as_str().unwrap_or_else(|| panic!("{id}: no raw"));
        assert!(p.run_dir.join(raw).is_file(), "{id}: raw {raw}");
    }
}

fn conformance(a: &Value) {
    let name = a["backend"].as_str().unwrap();
    let op = a["op"].as_str().unwrap();
    let fx = Fixture::load(name).unwrap_or_else(|| panic!("no fixture for {name}"));
    let secrets = fx.secrets();

    // ---- shapes only: bounded, pointed, two-signalled, and no secret anywhere
    let p = Project::new();
    let files = fx.materialize(&p);
    let (code, text, v) = run_with(p.path(), &summarize_args(op, name, &files), &[]);
    assert_eq!(code, 0, "{name}: {v}");
    assert_eq!(v["ok"], true);
    assert_eq!(v["backend"], name);
    check_bounded_outputs(&p, &text, &v);
    for s in &secrets {
        assert!(!text.contains(s.as_str()), "{name}: secret in stdout");
        for (path, t) in p.run_texts() {
            assert!(
                !t.contains(s.as_str()),
                "{name}: secret in {}",
                path.display()
            );
        }
    }
    // A coverage row: the adapter ran.
    let cov = p.coverage();
    let row = cov["workers"]
        .as_object()
        .unwrap()
        .values()
        .find_map(|w| w["backends"].get(name))
        .unwrap_or_else(|| panic!("{name}: no coverage row"));
    assert_eq!(row["status"], "ran", "{name}");

    // ---- with raw-values consent the secret shows (so the check above can fail), except
    // for readers that never emit values
    if fx.meta["emits_values"] != false && name != "dla-hprof" && name != "dla-hprof-histogram" {
        let q = Project::new();
        ok(q.path(), &["consent", "--tiers", "1", "--raw-values"]);
        let files = fx.materialize(&q);
        let (code, _text, v) = run_with(q.path(), &summarize_args(op, name, &files), &[]);
        assert_eq!(code, 0, "{name} with consent: {v}");
        assert_eq!(v["redaction"], "raw", "{name}");
        assert!(
            q.run_texts()
                .iter()
                .any(|(_, t)| secrets.iter().any(|s| t.contains(s.as_str()))),
            "{name}: raw consent should keep the planted value in the raw file"
        );
    }

    // ---- the tool is absent: PATH holds nothing, JAVA_HOME is unset
    if a["tool"].is_string() {
        let empty = tempfile::tempdir().unwrap();
        let env = [
            ("PATH", Some(empty.path().to_str().unwrap())),
            ("JAVA_HOME", None),
            ("DLA_DATA_DIR", None),
        ];
        let r = Project::new();
        let files = fx.materialize(&r);
        let (code, _t, v) = run_with(r.path(), &summarize_args(op, name, &files), &env);
        assert_eq!(code, 2, "{name}: explicit backend without its tool: {v}");
        assert_eq!(v["error"]["code"], "tool-missing", "{name}: {v}");
        assert!(
            v["error"]["fix"].as_str().is_some_and(|f| !f.is_empty()),
            "{name}: install hint in fix"
        );
        let cov = r.coverage();
        let row = cov["workers"]
            .as_object()
            .unwrap()
            .values()
            .find_map(|w| w["backends"].get(name))
            .unwrap_or_else(|| panic!("{name}: no coverage row for the missing tool"));
        assert_eq!(row["status"], "missing", "{name}");
        assert!(row["install_hint"].is_string(), "{name}");
        // `auto` completes: another adapter runs, or a coverage gap is named.
        let s = Project::new();
        let files = fx.materialize(&s);
        let mut args = vec!["summarize", op, "--backend", "auto"];
        args.extend(files.iter().map(String::as_str));
        let (code, _t, v) = run_with(s.path(), &args, &env);
        assert_eq!(code, 0, "{name}: auto without the tool: {v}");
        assert!(v["backend"] != name, "{name}: ran without its tool");
    }
}

#[test]
fn every_adapter_is_conformant() {
    for a in adapters() {
        conformance(&a);
    }
}

#[test]
fn list_names_op_runtime_backend_tool_and_availability() {
    let list = adapters();
    for a in &list {
        for k in ["op", "runtime", "backend"] {
            assert!(a[k].as_str().is_some_and(|s| !s.is_empty()), "{a}");
        }
        assert!(a["available"].is_boolean() && (a["tool"].is_null() || a["tool"].is_string()));
    }
    let ops: std::collections::BTreeSet<&str> =
        list.iter().map(|a| a["op"].as_str().unwrap()).collect();
    for op in [
        "heap-graph",
        "type-histogram-diff",
        "alloc-stack-diff",
        "non-heap",
    ] {
        assert!(ops.contains(op), "{op} has no adapter");
    }
    // DLA's own parsers carry no tool and are always available.
    let hprof = list.iter().find(|a| a["backend"] == "dla-hprof").unwrap();
    assert_eq!(hprof["tool"], Value::Null);
    assert_eq!(hprof["available"], true);
}

// ---------------------------------------------------------------------------------------
// Selection and refusals
// ---------------------------------------------------------------------------------------

#[test]
fn selection_refusals_name_what_is_missing_and_the_fix() {
    let p = Project::new();
    p.put("notes.txt", b"plain text");
    let e = refused(
        p.path(),
        &["summarize", "type-histogram-diff", "notes.txt", "notes.txt"],
    );
    // Duplicates are caught before the format.
    assert_eq!(e["code"], "duplicate-artifact");
    p.put("more.txt", b"also plain");
    let e = refused(
        p.path(),
        &["summarize", "type-histogram-diff", "notes.txt", "more.txt"],
    );
    assert_eq!(e["code"], "unsupported-format");
    assert!(e["message"].as_str().unwrap().contains("supported:"), "{e}");
    let e = refused(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "nope",
            "notes.txt",
        ],
    );
    assert_eq!(e["code"], "unknown-backend");
    assert!(
        e["message"].as_str().unwrap().contains("dla-alloc-json"),
        "{e}"
    );
    // An explicit backend that does not read the file's format.
    let e = refused(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "dla-alloc-json",
            "notes.txt",
        ],
    );
    assert_eq!(e["code"], "unsupported-format");
    let e = refused(p.path(), &["summarize", "non-heap", "gone.json"]);
    assert_eq!(e["code"], "artifact-not-found");
    // A backend of another operation is not found for this one.
    let e = refused(
        p.path(),
        &[
            "summarize",
            "non-heap",
            "--backend",
            "dla-hprof",
            "notes.txt",
        ],
    );
    assert_eq!(e["code"], "unknown-backend");
    // Nothing was written.
    assert!(p.evidence("histogram-diff").is_empty());
    assert!(!p.run_dir.join("histograms").exists());
}

#[test]
fn auto_picks_by_format_and_a_missing_operation_input_is_a_usage_error() {
    let fx = Fixture::load("dla-alloc-json").unwrap();
    let p = Project::new();
    let files = fx.materialize(&p);
    let mut args = vec!["summarize", "alloc-stack-diff"];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["backend"], "dla-alloc-json");
    let (code, v) = run(p.path(), &["summarize"]);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["ok"], false);
}

// ---------------------------------------------------------------------------------------
// Ties and attribution
// ---------------------------------------------------------------------------------------

const HOLDER_JAVA: &str = "package app;\n\nimport java.util.ArrayList;\nimport java.util.List;\n\npublic final class Holder {\n    // the leak\n    static final List<Item> ITEMS = new ArrayList<>();\n\n    static void add(Item i) {\n        ITEMS.add(i);\n    }\n}\n";

fn three_dumps(p: &Project) -> Vec<String> {
    (0..3)
        .map(|i| {
            let m = model(8, T0 + 30_000 * i as u64, [200, 400, 600][i], 5);
            p.put(&format!("dumps/d{i}.hprof"), &m.dump)
        })
        .collect()
}

fn heap_graph(p: &Project, files: &[String], extra: &[&str]) -> Value {
    let mut args = vec!["summarize", "heap-graph"];
    args.extend(files.iter().map(String::as_str));
    args.extend_from_slice(extra);
    ok(p.path(), &args)
}

/// A census file and its candidate finding, as `dla census` leaves them.
fn census_finding(p: &Project, owner: &str, site: &str, writers: &[&str]) -> String {
    fs::create_dir_all(p.run_dir.join("census")).unwrap();
    fs::write(
        p.run_dir.join("census/java.json"),
        json!({"lang": "java", "entries": [{
            "kind": "static-field", "holder": "static/global", "region": "heap",
            "site": site, "owner": owner, "writers": writers, "removers": []
        }]})
        .to_string(),
    )
    .unwrap();
    let v = ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            site,
            "--owner",
            owner,
            "--holder",
            "static/global",
            "--region",
            "heap",
        ],
    );
    v["id"].as_str().unwrap().to_owned()
}

#[test]
fn a_dump_series_on_a_repository_run_ties_to_the_census_finding() {
    let p = Project::new();
    p.put("src/app/Holder.java", HOLDER_JAVA.as_bytes());
    let f = census_finding(
        &p,
        "app.Holder.ITEMS",
        "src/app/Holder.java:8",
        &["src/app/Holder.java:11"],
    );
    let files = three_dumps(&p);
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass");
    // The census finding is the tie: no duplicate candidate.
    assert_eq!(v["candidates"].as_array().unwrap().len(), 0, "{v}");
    assert_eq!(p.findings().len(), 1);
    let g = &p.evidence("growth-series")[0];
    let col = &g["result"]["passed"][0];
    assert_eq!(col["column"], "holder:static:app.Holder.ITEMS");
    assert_eq!(col["finding"], f.as_str());
    assert_eq!(col["candidates"][0], f.as_str());
    assert_eq!(col["site"], "src/app/Holder.java:8");
    assert!(ledger_rule_ties(
        g,
        &f,
        Some("src/app/Holder.java:8"),
        Some("app.Holder.ITEMS")
    ));
    // The series file carries the tie too, so `dla trend` sees it.
    let s: Value = serde_json::from_str(
        &fs::read_to_string(
            p.run_dir
                .join(format!("series/{}.json", v["series"].as_str().unwrap())),
        )
        .unwrap(),
    )
    .unwrap();
    let c = s["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "holder:static:app.Holder.ITEMS")
        .unwrap();
    assert_eq!(c["finding"], f.as_str());
    assert_eq!(c["site"], "src/app/Holder.java:8");
}

#[test]
fn a_new_candidate_gets_the_declaration_site_by_source_scan() {
    let p = Project::new();
    p.put("src/app/Holder.java", HOLDER_JAVA.as_bytes());
    let files = three_dumps(&p);
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass");
    let c = &v["candidates"][0];
    assert_eq!(c["owner"], "app.Holder.ITEMS");
    assert_eq!(c["site"], "src/app/Holder.java:8", "{v}");
    assert_eq!(c["via"], "source-scan");
    let f = &p.findings()[0];
    assert_eq!(f["site"], "src/app/Holder.java:8");
    assert_eq!(f["holder"], "static/global");
    // A second run ties to the finding it made, by owner.
    let again = heap_graph(&p, &files, &[]);
    assert_eq!(again["candidates"].as_array().unwrap().len(), 0);
    assert_eq!(p.findings().len(), 1);
}

#[test]
fn a_site_outside_the_source_or_on_an_artifact_run_is_never_made_up() {
    // No source for the class: no site.
    let p = Project::new();
    p.put(
        "src/other/Other.java",
        b"package other; class Other { int x; }\n",
    );
    let files = three_dumps(&p);
    let v = heap_graph(&p, &files, &[]);
    assert!(v["candidates"][0]["site"].is_null(), "{v}");
    assert!(p.findings()[0]["site"].is_null());
    // An artifact run has no source: the resolver is not asked, though the file is there.
    let a = Project::with_init(&["--entry", "artifact", "--artifact", "dumps"]);
    a.put("src/app/Holder.java", HOLDER_JAVA.as_bytes());
    let files = three_dumps(&a);
    let v = heap_graph(&a, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert!(v["candidates"][0]["site"].is_null(), "{v}");
    assert!(a.findings()[0]["site"].is_null());
    assert_eq!(a.findings()[0]["owner"], "app.Holder.ITEMS");
}

fn histogram_text(rows: &[(&str, u64, u64)]) -> String {
    let mut s = String::from(
        " num     #instances         #bytes  class name (module)\n-------------------------------------------------------\n",
    );
    for (i, (name, n, b)) in rows.iter().enumerate() {
        s.push_str(&format!("{:>4}: {:>13} {:>14}  {name}\n", i + 1, n, b));
    }
    s
}

fn histograms(p: &Project, growing: &str) -> Vec<String> {
    (0..3u64)
        .map(|i| {
            let t = histogram_text(&[
                ("[B", 5000 + 100 * i, 400_000 + 9_000 * i),
                (growing, 200 * (i + 1), 4_800 * (i + 1)),
                ("java.lang.String", 800, 19_200),
            ]);
            p.put(&format!("hist/h{i}.txt"), t.as_bytes())
        })
        .collect()
}

#[test]
fn a_growing_type_ties_to_the_finding_whose_accumulation_class_it_is() {
    let p = Project::new();
    p.put("src/app/Holder.java", HOLDER_JAVA.as_bytes());
    let f = census_finding(&p, "app.Holder.ITEMS", "src/app/Holder.java:8", &[]);
    // The dump summaries say what piles up under app.Holder.ITEMS.
    let dumps = three_dumps(&p);
    heap_graph(&p, &dumps, &[]);
    let summary = fs::read_dir(p.run_dir.join("summaries"))
        .unwrap()
        .flatten()
        .next()
        .unwrap()
        .path();
    let s: Value = serde_json::from_str(&fs::read_to_string(summary).unwrap()).unwrap();
    let acc = s["holders"][0]["accumulation"]["class"]
        .as_str()
        .unwrap()
        .to_owned();
    let files = histograms(&p, &acc);
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["backend"], "jcmd-class-histogram");
    assert_eq!(v["tied"][0], f.as_str(), "{v}");
    let g = p
        .evidence("growth-series")
        .into_iter()
        .rev()
        .find(|e| e["tool"]["name"] == "jcmd-class-histogram")
        .unwrap();
    let tied: Vec<&Value> = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["finding"] == f.as_str())
        .collect();
    assert_eq!(tied.len(), 1, "{g}");
    assert!(ledger_rule_ties(
        &g,
        &f,
        Some("src/app/Holder.java:8"),
        Some("app.Holder.ITEMS")
    ));
    assert!(tied[0]["column"].as_str().unwrap().contains(&acc), "{g}");
    // The histogram-diff evidence names the top growing types and the monotone ones.
    let d = p.evidence("histogram-diff").pop().unwrap();
    assert_eq!(d["authority"], "live");
    assert!(d["result"]["top"][0]["d_bytes"].as_i64().unwrap() > 0);
    assert!(d["result"]["monotone"].as_array().unwrap().len() >= 2);
}

#[test]
fn a_growing_type_the_owner_declares_ties_to_its_finding() {
    let p = Project::new();
    p.put("src/app/Holder.java", HOLDER_JAVA.as_bytes());
    let f = census_finding(&p, "app.Holder.ITEMS", "src/app/Holder.java:8", &[]);
    // `List<Item> ITEMS`: the growing app.Item belongs to it; the growing byte[] does not.
    let files = histograms(&p, "app.Item");
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["tied"][0], f.as_str(), "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    let cols = g["result"]["passed"].as_array().unwrap();
    let item = cols
        .iter()
        .find(|c| c["column"] == "type:app.Item")
        .unwrap();
    assert_eq!(item["finding"], f.as_str());
    assert!(ledger_rule_ties(&g, &f, None, None));
    let other = cols.iter().find(|c| c["column"] == "type:byte[]").unwrap();
    assert!(
        other["finding"].is_null() && other["candidates"].is_null(),
        "{other}"
    );
}

#[test]
fn a_growing_type_that_names_no_finding_stays_untied_unless_the_caller_says_so() {
    let p = Project::new();
    let files = histograms(&p, "shop.Entry");
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["gate"], "pass");
    assert_eq!(v["tied"].as_array().unwrap().len(), 0);
    assert!(v["needed"].as_str().unwrap().contains("--finding"), "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    for c in g["result"]["passed"].as_array().unwrap() {
        assert!(c["finding"].is_null() && c["candidates"].is_null(), "{c}");
    }
    // `--finding` ties the best passing column to the finding.
    ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--owner",
            "shop.Cache.entries",
        ],
    );
    let mut args = vec!["summarize", "type-histogram-diff", "--finding", "F1"];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["tied"][0], "F1");
    assert_eq!(v["tie"], "tied by --finding");
    let e = refused(
        p.path(),
        &[
            "summarize",
            "type-histogram-diff",
            "--finding",
            "F9",
            &files[0],
            &files[1],
            &files[2],
        ],
    );
    assert_eq!(e["code"], "unknown-finding");
}

#[test]
fn two_captures_make_a_diff_but_no_growth_column() {
    let p = Project::new();
    let files = histograms(&p, "shop.Entry");
    let v = ok(
        p.path(),
        &["summarize", "type-histogram-diff", &files[0], &files[1]],
    );
    assert_eq!(v["gate"], "fail");
    assert_eq!(v["reasons"][0], "captures < 3");
    assert!(p.evidence("growth-series").is_empty());
    assert_eq!(p.evidence("histogram-diff").len(), 1);
    assert!(!p.run_dir.join("series").exists());
    let e = refused(p.path(), &["summarize", "type-histogram-diff", &files[0]]);
    assert_eq!(e["code"], "not-enough-captures");
}

#[test]
fn harness_dump_types_and_hprof_class_stats_feed_the_same_diff() {
    let p = Project::new();
    let fx = Fixture::load("harness-types").unwrap();
    let files = fx.materialize(&p);
    let v = ok(p.path(), &["summarize", "type-histogram-diff", &files[0]]);
    assert_eq!(v["backend"], "harness-types");
    assert_eq!(v["captures"], 4);
    assert_eq!(v["gate"], "pass");
    assert_eq!(p.evidence("histogram-diff")[0]["authority"], "test");
    // Heap dumps: the largest classes by shallow bytes, ordered by header time whatever
    // the names say.
    let q = Project::new();
    let dumps = three_dumps(&q);
    let v = ok(
        q.path(),
        &[
            "summarize",
            "type-histogram-diff",
            &dumps[2],
            &dumps[0],
            &dumps[1],
        ],
    );
    assert_eq!(v["backend"], "dla-hprof-histogram");
    assert_eq!(v["ordered_by"], "timestamp");
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(q.evidence("histogram-diff")[0]["authority"], "dump");
    // The dump parser's histogram and its heap graph are one signal on one dump.
    heap_graph(&q, &dumps, &[]);
    let sigs: Vec<Value> = q
        .evidence("retention-path")
        .iter()
        .chain(q.evidence("histogram-diff").iter())
        .map(|e| e["result"]["signal"].clone())
        .collect();
    assert!(sigs.iter().all(|s| s["class"] == "dump-parser"), "{sigs:?}");
}

const POOL_JAVA: &str = "package shop;\n\npublic final class Pool {\n    static void spawn() {\n        new Thread(() -> {}).start();\n    }\n}\n";

#[test]
fn an_alloc_stack_ties_by_the_findings_writers_or_its_root_owner() {
    let fx = Fixture::load("dla-alloc-json").unwrap();
    // By writers: the finding lists the stack's project frame.
    let p = Project::new();
    p.put("src/trail.rs", &[b'\n'; 60]);
    p.put("src/cache.rs", &[b'\n'; 20]);
    let files = fx.materialize(&p);
    ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--site",
            "src/trail.rs:12",
        ],
    );
    ok(
        p.path(),
        &["ledger", "annotate", "F1", "--writer", "src/trail.rs:40"],
    );
    let v = ok(
        p.path(),
        &summarize_args("alloc-stack-diff", "dla-alloc-json", &files),
    );
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"][0], "F1", "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    let col = &g["result"]["passed"][0];
    assert_eq!(col["finding"], "F1");
    assert_eq!(col["site"], "src/trail.rs:40");
    assert!(ledger_rule_ties(&g, "F1", Some("src/trail.rs:12"), None));
    assert_eq!(g["result"]["passed"].as_array().unwrap().len(), 1, "{g}");
    // The alloc-stack evidence records live bytes by stack.
    let a = p.evidence("alloc-stack").pop().unwrap();
    assert_eq!(a["result"]["live"], true);
    assert_eq!(a["result"]["top"][0]["d_bytes"], 4900);

    // By the root path's owner: a finding of that owner, no writers needed.
    let q = Project::new();
    q.put("src/trail.rs", &[b'\n'; 60]);
    let files = fx.materialize(&q);
    ok(
        q.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "src/audit.rs:3",
            "--owner",
            "shop.codec.Audit.trail",
        ],
    );
    let v = ok(
        q.path(),
        &summarize_args("alloc-stack-diff", "dla-alloc-json", &files),
    );
    assert_eq!(v["tied"][0], "F1", "{v}");
    // The allocation site is recorded as a writer of the finding.
    let ann = q
        .ledger_lines()
        .into_iter()
        .find(|l| l["type"] == "finding_annotated")
        .unwrap_or_else(|| panic!("no annotation"));
    assert_eq!(ann["writers"][0], "src/trail.rs:40");

    // No finding: the stack names an owner the resolver cannot find, so nothing is made up.
    let r = Project::new();
    r.put("src/trail.rs", &[b'\n'; 60]);
    let files = fx.materialize(&r);
    let v = ok(
        r.path(),
        &summarize_args("alloc-stack-diff", "dla-alloc-json", &files),
    );
    assert_eq!(v["tied"].as_array().unwrap().len(), 0);
    assert!(r.findings().is_empty());
}

#[test]
fn allocation_only_profiles_never_pass_the_growth_gate() {
    let p = Project::new();
    let files: Vec<String> = (1..=3u64)
        .map(|i| {
            let prof = json!({"source": "jfr", "timestamp_ms": T0 + i * 1000, "sampled": true,
                "groups": [{"frames": [{"function": "shop.Svc.handle", "file": null, "line": null, "module": null}],
                             "live_bytes": 0, "live_blocks": 0, "total_bytes": i * 100_000,
                             "root_path": [], "kind": null}]});
            p.put(&format!("allocs/p{i}.json"), prof.to_string().as_bytes())
        })
        .collect();
    let v = ok(
        p.path(),
        &summarize_args("alloc-stack-diff", "dla-alloc-json", &files),
    );
    assert_eq!(v["gate"], "fail");
    assert_eq!(v["live"], false);
    assert!(p.evidence("growth-series").is_empty());
    let a = p.evidence("alloc-stack").pop().unwrap();
    assert_eq!(a["result"]["live"], false);
    assert_eq!(a["result"]["top"][0]["alloc_bytes"], 200_000);
    // One profile is the leak-at-exit view.
    let v = ok(
        p.path(),
        &summarize_args("alloc-stack-diff", "dla-alloc-json", &files[..1]),
    );
    assert_eq!(v["mode"], "single");
}

#[test]
fn thread_growth_ties_to_the_creation_site() {
    let fx = Fixture::load("dla-nonheap-json").unwrap();
    let p = Project::new();
    p.put("shop/Pool.java", POOL_JAVA.as_bytes());
    ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "shop/Pool.java:7",
            "--owner",
            "shop.Pool.workers",
            "--holder",
            "native-handle/global-ref",
            "--region",
            "non-heap",
        ],
    );
    let files = fx.materialize(&p);
    let v = ok(
        p.path(),
        &summarize_args("non-heap", "dla-nonheap-json", &files),
    );
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"][0], "F1", "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    let tied = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == "group:thread/WAITING")
        .unwrap();
    assert_eq!(tied["finding"], "F1");
    assert_eq!(tied["region"], "non-heap");
    assert!(ledger_rule_ties(&g, "F1", Some("shop/Pool.java:7"), None));
    // Metric growth with no site stays untied.
    let untied = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == "metric:nmt/Thread")
        .unwrap();
    assert!(untied["finding"].is_null());
    assert_eq!(p.evidence("non-heap")[0]["authority"], "test");
}

#[test]
fn files_under_the_run_directory_are_inputs_too() {
    let p = Project::new();
    let fx = Fixture::load("jcmd-class-histogram").unwrap();
    let files = fx.materialize(&p);
    // Move them under attach/P1/ of the run, and pass run-relative paths.
    fs::create_dir_all(p.run_dir.join("attach/P1")).unwrap();
    let mut moved = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let to = format!("attach/P1/h{i}.txt");
        fs::copy(p.path().join(f), p.run_dir.join(&to)).unwrap();
        moved.push(to);
    }
    let v = ok(
        p.path(),
        &summarize_args("type-histogram-diff", "auto", &moved),
    );
    assert_eq!(v["captures"], 3);
    // Absolute and `.dla/<run>/...` project-relative paths work as well.
    let rel = format!(
        ".dla/{}/attach/P1/h0.txt",
        p.run_dir.file_name().unwrap().to_str().unwrap()
    );
    let abs = p.run_dir.join("attach/P1/h1.txt");
    let v = ok(
        p.path(),
        &[
            "summarize",
            "type-histogram-diff",
            &rel,
            abs.to_str().unwrap(),
        ],
    );
    assert_eq!(v["captures"], 2);
}

#[test]
fn artifact_runs_treat_files_as_dump_authority() {
    let p = Project::with_init(&["--entry", "artifact", "--artifact", "hist"]);
    let files = histograms(&p, "shop.Entry");
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(files.iter().map(String::as_str));
    ok(p.path(), &args);
    assert_eq!(p.evidence("histogram-diff")[0]["authority"], "dump");
    assert_eq!(p.evidence("growth-series")[0]["authority"], "dump");
}

// ---------------------------------------------------------------------------------------
// Attach writes coverage rows
// ---------------------------------------------------------------------------------------

struct Child(std::process::Child, #[allow(dead_code)] tempfile::TempDir);

impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A `sleep` process whose executable is called `java`, so a fake `jcmd` can answer for it.
fn java_sleeper() -> Child {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("java");
    let sleep = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|d| d.join("sleep"))
        .find(|p| p.is_file())
        .expect("sleep on PATH");
    fs::copy(sleep, &exe).unwrap();
    fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
    let child = std::process::Command::new(&exe)
        .arg("300")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    Child(child, dir)
}

const FAKE_JCMD: &str = r#"#!/bin/sh
pid="$1"; cmd="$2"
case "$cmd" in
  VM.version) echo "$pid:"; echo "JDK 21.0.1";;
  GC.heap_info) echo "$pid:"; echo " garbage-first heap   total 10240K, used 8192K [0x0, 0x1)";;
  GC.class_histogram) echo "$pid:"; echo " num     #instances         #bytes  class name (module)"; echo "-------------------------------------------------------"; echo "   1:            10           8000  [B (java.base@21)"; echo "Total            10           8000";;
  *) echo "unexpected $cmd" >&2; exit 1;;
esac
"#;

#[test]
fn attach_writes_a_coverage_row_for_a_run_and_for_a_missing_tool() {
    use std::os::unix::fs::PermissionsExt;
    let p = Project::new();
    ok(p.path(), &["consent", "--tiers", "1-4"]);
    let data = tempfile::tempdir().unwrap();
    let tools = data.path().join("tools");
    fs::create_dir_all(&tools).unwrap();
    let jcmd = tools.join("jcmd");
    fs::write(&jcmd, FAKE_JCMD).unwrap();
    fs::set_permissions(&jcmd, fs::Permissions::from_mode(0o755)).unwrap();
    let target = java_sleeper();
    let pid = target.0.id().to_string();
    let env = [("DLA_DATA_DIR", Some(data.path().to_str().unwrap()))];
    let (code, _t, v) = run_with(
        p.path(),
        &["attach", "jcmd", "--pid", &pid, "--op", "histogram"],
        &env,
    );
    assert_eq!(code, 0, "{v}");
    let row = &p.coverage()["workers"]["jvm"]["backends"]["jcmd"];
    assert_eq!(row["status"], "ran", "{row}");
    assert!(row["enumerated"].as_u64().unwrap() >= 1);

    // A tool that is not there: a `missing` row with the install command, and the
    // refusal as before. (Skipped when `grf` happens to be installed.)
    let has_grf = std::process::Command::new("sh")
        .args(["-c", "command -v grf"])
        .stdout(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !has_grf {
        let q = Project::new();
        ok(q.path(), &["consent", "--tiers", "1-4"]);
        let (code, _t, v) = run_with(
            q.path(),
            &["attach", "goref", "--pid", &pid, "--op", "heap-profile"],
            &env,
        );
        assert_eq!(code, 2, "{v}");
        assert_eq!(v["error"]["code"], "tool-missing");
        let row = &q.coverage()["workers"]["go"]["backends"]["goref"];
        assert_eq!(row["status"], "missing", "{row}");
        assert!(
            row["install_hint"].as_str().unwrap().contains("go install"),
            "{row}"
        );
    }
}

#[test]
fn a_long_series_with_long_names_stays_bounded_and_keeps_its_pointers() {
    let p = Project::new();
    let long = "x".repeat(70);
    let files: Vec<String> = (0..8)
        .map(|i| {
            let m = model(8, T0 + 30_000 * i as u64, 100 * (i + 1), 5);
            p.put(&format!("dumps/{long}-{i}.hprof"), &m.dump)
        })
        .collect();
    let mut args = vec!["summarize", "heap-graph"];
    args.extend(files.iter().map(String::as_str));
    let (code, text, v) = run_with(p.path(), &args, &[]);
    assert_eq!(code, 0, "{v}");
    assert!(text.len() <= 2048, "{}", text.len());
    // Eight retention-path evidences, the growth evidence and the discriminators: every
    // id is still named, and `raw` is whole.
    assert!(v["evidence"].as_array().unwrap().len() >= 9, "{v}");
    check_bounded_outputs(&p, &text, &v);
}
