//! The Go adapters of `dla summarize`: heap profiles (pprof), goroutine dumps and the
//! goroutineleak profile.
//!
//! The inputs are real go1.27.0 output (see `dla-parsers/tests/fixtures/go/README.md`). The
//! checks here do not depend on `dla summarize --list`: bounded output, a `raw` pointer
//! with its hash, no planted secret without consent, a coverage row, and what is specific
//! to Go: goroutines tie to a finding by their creation site, the leak profile is a second
//! signal that never confirms and never refutes, and a goroutine profile never ties.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SECRET: &str = "AKIAIOSFODNN7EXAMPLE1";

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
    v["error"].clone()
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/summarize")
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

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn put(&self, rel: &str, bytes: &[u8]) -> String {
        let p = self.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
        rel.to_owned()
    }

    /// The fixture files of one adapter, put into the project under `in/`.
    fn inputs(&self, backend: &str, names: &[&str]) -> Vec<String> {
        names
            .iter()
            .map(|n| {
                let bytes = fs::read(fixtures().join(backend).join(n)).unwrap();
                assert!(bytes.len() < 64 * 1024, "{n} is over 64 KB");
                self.put(&format!("in/{n}"), &bytes)
            })
            .collect()
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

    /// The Go sources the profiles name, so their creation and allocation sites resolve.
    fn sources(&self) {
        let w = fixtures().parent().unwrap().join("go-sources");
        for f in ["internal/notify/subscription.go", "cmd/scratch/main.go"] {
            self.put(f, &fs::read(w.join(f)).unwrap());
        }
    }

    fn census_finding(&self, owner: &str, site: &str, holder: &str, region: &str) -> String {
        let v = ok(
            self.path(),
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
                holder,
                "--region",
                region,
            ],
        );
        v["id"].as_str().unwrap().to_owned()
    }
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

const GOROUTINES: [&str; 3] = ["goroutine1.txt", "goroutine2.txt", "goroutine3.txt"];
const LEAKS: [&str; 3] = [
    "goroutineleak1.txt",
    "goroutineleak2.txt",
    "goroutineleak3.txt",
];
const HEAPS: [&str; 3] = ["heap1.pb.gz", "heap2.pb.gz", "heap3.pb.gz"];

fn summarize(p: &Project, op: &str, backend: &str, files: &[String]) -> (String, Value) {
    let mut args = vec!["summarize", op, "--backend", backend];
    args.extend(files.iter().map(String::as_str));
    let (code, text, v) = run(p.path(), &args);
    assert_eq!(code, 0, "{backend}: {v}");
    (text, v)
}

fn check_bounded(p: &Project, text: &str, v: &Value) {
    assert!(text.len() <= 2048);
    let rel = v["raw"]["path"].as_str().unwrap_or_else(|| panic!("{v}"));
    assert!(!rel.starts_with('/') && !rel.contains(".."), "{rel}");
    let bytes = fs::read(p.run_dir.join(rel)).unwrap();
    assert_eq!(v["raw"]["sha256"], sha(&bytes));
    let lines = p.ledger_lines();
    let ids = v["evidence"].as_array().unwrap();
    assert!(!ids.is_empty());
    for id in ids {
        let e = lines
            .iter()
            .find(|l| l["type"] == "evidence" && l["evidence_id"] == *id)
            .unwrap_or_else(|| panic!("{id} is not in the ledger"));
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
}

/// (backend, op, fixture files)
fn cases() -> Vec<(&'static str, &'static str, [&'static str; 3])> {
    vec![
        ("go-pprof-heap", "alloc-stack-diff", HEAPS),
        ("go-pprof-heap-functions", "type-histogram-diff", HEAPS),
        ("go-goroutine", "non-heap", GOROUTINES),
        ("go-goroutineleak", "non-heap", LEAKS),
    ]
}

#[test]
fn every_go_adapter_is_bounded_pointed_covered_and_keeps_secrets_out() {
    for (backend, op, names) in cases() {
        let p = Project::new();
        let files = p.inputs(backend, &names);
        let (text, v) = summarize(&p, op, backend, &files);
        assert_eq!(v["backend"], backend);
        check_bounded(&p, &text, &v);
        assert!(!text.contains(SECRET), "{backend}: secret in stdout");
        for (path, t) in p.run_texts() {
            assert!(
                !t.contains(SECRET),
                "{backend}: secret in {}",
                path.display()
            );
        }
        let cov = p.coverage();
        let row = cov["workers"]
            .as_object()
            .unwrap()
            .values()
            .find_map(|w| w["backends"].get(backend))
            .unwrap_or_else(|| panic!("{backend}: no coverage row"));
        assert_eq!(row["status"], "ran", "{backend}");
    }
}

#[test]
fn with_raw_values_consent_the_function_name_shows_in_the_raw_file() {
    for (backend, op, names) in [
        ("go-pprof-heap", "alloc-stack-diff", HEAPS),
        ("go-goroutine", "non-heap", GOROUTINES),
    ] {
        let p = Project::new();
        ok(p.path(), &["consent", "--tiers", "1", "--raw-values"]);
        let files = p.inputs(backend, &names);
        let (_, v) = summarize(&p, op, backend, &files);
        assert_eq!(v["redaction"], "raw", "{backend}");
        assert!(
            p.run_texts().iter().any(|(_, t)| t.contains(SECRET)),
            "{backend}: raw consent should keep the name in the raw file"
        );
    }
}

fn ties_to(e: &Value, id: &str, site: Option<&str>) -> bool {
    let summary = dla_core::trend::GrowthSummary::from_result(&e["result"]).unwrap();
    assert_eq!(e["result"]["gate"], "pass", "{e}");
    let fid: dla_core::FindingId = id.parse().unwrap();
    summary
        .passed
        .iter()
        .any(|c| c.tied_to_owner(&fid, site, None))
}

#[test]
fn goroutine_growth_ties_to_the_creation_site_and_nothing_else() {
    let p = Project::new();
    p.sources();
    let f = p.census_finding(
        "notify.Subscription",
        "internal/notify/subscription.go:33",
        "blocked-stack",
        "non-heap",
    );
    let files = p.inputs("go-goroutine", &GOROUTINES);
    let (_, v) = summarize(&p, "non-heap", "go-goroutine", &files);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([f]), "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    assert!(ties_to(&g, &f, Some("internal/notify/subscription.go:33")));
    let passed = g["result"]["passed"].as_array().unwrap();
    let tied: Vec<&str> = passed
        .iter()
        .filter(|c| c["finding"] == f.as_str())
        .map(|c| c["column"].as_str().unwrap())
        .collect();
    assert_eq!(tied.len(), 1, "{passed:?}");
    assert!(tied[0].starts_with("group:goroutine/created:"), "{tied:?}");
    assert_eq!(
        passed.iter().find(|c| c["column"] == tied[0]).unwrap()["tie"]["rule"],
        "site"
    );
    // Totals, the wait reason and the leaked mark are whole-process figures: no tie.
    for c in passed {
        let col = c["column"].as_str().unwrap();
        if col.starts_with("metric:") || col.contains("wait:") {
            assert!(c["finding"].is_null(), "{col} tied: {c}");
        }
    }
}

#[test]
fn a_goroutine_without_a_finding_at_its_site_ties_to_nothing() {
    let p = Project::new();
    p.sources();
    // A finding on another line of the file: not the go statement.
    p.census_finding(
        "notify.Subscription",
        "internal/notify/subscription.go:40",
        "blocked-stack",
        "non-heap",
    );
    let files = p.inputs("go-goroutine", &GOROUTINES);
    let (_, v) = summarize(&p, "non-heap", "go-goroutine", &files);
    assert_eq!(v["tied"], json!([]), "{v}");
    assert!(
        v["needed"].as_str().unwrap().contains("no finding")
            || v["needed"].as_str().unwrap().contains("tie"),
        "{v}"
    );
}

#[test]
fn the_leak_profile_is_a_second_signal_listed_under_the_finding_and_confirms_nothing() {
    let p = Project::new();
    p.sources();
    let f = p.census_finding(
        "notify.Subscription",
        "internal/notify/subscription.go:33",
        "blocked-stack",
        "non-heap",
    );
    let files = p.inputs("go-goroutineleak", &LEAKS);
    let (text, v) = summarize(&p, "non-heap", "go-goroutineleak", &files);
    check_bounded(&p, &text, &v);
    assert_eq!(v["total"], 30, "{v}");
    assert_eq!(v["tied"], json!([f]), "{v}");
    let g = &v["groups"][0];
    assert_eq!(g["site"], "internal/notify/subscription.go:33");
    assert_eq!(g["blocked_at"], "internal/notify/subscription.go:40");
    assert_eq!(g["wait_reason"], "select");
    assert_eq!(g["count"], 30);
    // Only the goroutines created at the go statement are listed: the ones parked on a
    // global channel are not leaked to the runtime.
    assert_eq!(v["groups"].as_array().unwrap().len(), 1);
    let ev = p.evidence("should-be-dead");
    assert_eq!(ev.len(), 1);
    let sig = &ev[0]["result"]["signal"];
    assert_eq!(sig["tool"], "go-goroutineleak");
    assert_eq!(sig["class"], "gc-reachability");
    assert_eq!(ev[0]["result"]["groups"][0]["findings"], json!([f]));
    // It never writes a growth column, an annotation or a candidate that could confirm.
    assert!(p.evidence("growth-series").is_empty());
    assert!(
        !p.ledger_lines()
            .iter()
            .any(|l| l["type"] == "finding_annotated"
                || l["type"] == "finding_added" && l["finding"]["id"] != f.as_str()),
        "the leak profile must not annotate or add findings"
    );
    assert!(
        v["needed"]
            .as_str()
            .unwrap()
            .contains("never confirms alone"),
        "{v}"
    );
}

#[test]
fn an_empty_leak_profile_proves_nothing() {
    let p = Project::new();
    p.sources();
    let f = p.census_finding(
        "notify.Subscription",
        "internal/notify/subscription.go:33",
        "blocked-stack",
        "non-heap",
    );
    // The profile of a process with nothing leaked: the running goroutine only.
    let text = "goroutine 1 [running]:\nruntime/pprof.writeGoroutineStacks({0x5f5ae8, 0xc0000a0000})\n\t/usr/local/go/src/runtime/pprof/pprof.go:816 +0x69\nmain.main()\n\t/p/main.go:27 +0xc8\n";
    let file = p.put("in/goroutineleak-empty.txt", text.as_bytes());
    let (_, v) = summarize(&p, "non-heap", "go-goroutineleak", &[file]);
    assert_eq!(v["total"], 0, "{v}");
    assert_eq!(v["tied"], json!([]));
    assert!(
        v["needed"].as_str().unwrap().contains("proves nothing"),
        "{v}"
    );
    assert_eq!(p.evidence("should-be-dead").len(), 1);
    // Nothing about the finding changed: no refutation, no annotation.
    assert!(
        !p.ledger_lines()
            .iter()
            .any(|l| l["type"] == "finding_annotated" || l["type"] == "finding_refuted"),
        "{f}"
    );
}

#[test]
fn a_heap_stack_ties_by_its_allocation_site_and_a_function_row_never_ties() {
    let p = Project::new();
    p.sources();
    // The held buffers are allocated at main.go:20 by the function that holds them.
    let f = p.census_finding(
        "main.held",
        "cmd/scratch/main.go:20",
        "static/global",
        "heap",
    );
    let files = p.inputs("go-pprof-heap", &HEAPS);
    let (_, v) = summarize(&p, "alloc-stack-diff", "go-pprof-heap", &files);
    assert_eq!(v["sampled"], true);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([f]), "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    assert!(ties_to(&g, &f, Some("cmd/scratch/main.go:20")));
    // The same profiles as a histogram by function: rows are functions, not types.
    let q = Project::new();
    q.sources();
    q.census_finding(
        "main.held",
        "cmd/scratch/main.go:20",
        "static/global",
        "heap",
    );
    let files = q.inputs("go-pprof-heap-functions", &HEAPS);
    let (_, v) = summarize(&q, "type-histogram-diff", "go-pprof-heap-functions", &files);
    assert_eq!(v["backend"], "go-pprof-heap-functions");
    assert!(v["tied"].as_array().is_none_or(Vec::is_empty), "{v}");
    for g in q.evidence("growth-series") {
        for c in g["result"]["passed"].as_array().unwrap() {
            assert!(c["finding"].is_null(), "{c}");
        }
    }
}

#[test]
fn a_goroutine_profile_groups_by_blocked_frame_and_never_ties() {
    let p = Project::new();
    p.sources();
    p.census_finding(
        "notify.Subscription",
        "internal/notify/subscription.go:33",
        "blocked-stack",
        "non-heap",
    );
    let bytes = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../dla-parsers/tests/fixtures/go/goroutine-3.pb.gz"),
    )
    .unwrap();
    let a = p.put("in/goroutine-3.pb.gz", &bytes);
    let b = p.put("in/goroutine-3-again.pb.gz", &bytes);
    let c = p.put("in/goroutine-3-third.pb.gz", &bytes);
    let (_, v) = summarize(&p, "non-heap", "go-goroutine", &[a, b, c]);
    assert_eq!(v["tied"], json!([]), "{v}");
    // Auto picks the profile reader for a profile.
    let d = p.put("in/x.pb.gz", &bytes);
    let e = p.put("in/y.pb.gz", &bytes);
    let (_, v) = run_ok_auto(&p, "non-heap", &[d, e]);
    assert_eq!(v["backend"], "go-goroutine");
}

fn run_ok_auto(p: &Project, op: &str, files: &[String]) -> (String, Value) {
    let mut args = vec!["summarize", op, "--backend", "auto"];
    args.extend(files.iter().map(String::as_str));
    let (code, text, v) = run(p.path(), &args);
    assert_eq!(code, 0, "{v}");
    (text, v)
}

#[test]
fn text_and_profiles_are_not_mixed_and_other_text_is_refused() {
    let p = Project::new();
    let prof = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../dla-parsers/tests/fixtures/go/goroutine-3.pb.gz"),
    )
    .unwrap();
    let a = p.put("in/g.pb.gz", &prof);
    let t = p.inputs("go-goroutine", &["goroutine1.txt"]);
    let e = refused(
        p.path(),
        &[
            "summarize",
            "non-heap",
            "--backend",
            "go-goroutine",
            &a,
            &t[0],
        ],
    );
    assert_eq!(e["code"], "mixed-formats", "{e}");
    let n = p.put("in/notes.txt", b"just some notes\n");
    let e = refused(
        p.path(),
        &["summarize", "non-heap", "--backend", "go-goroutine", &n],
    );
    assert_eq!(e["code"], "unsupported-format", "{e}");
    // A heap profile is not a goroutine profile, and the other way round.
    let h = p.inputs("go-pprof-heap", &["heap1.pb.gz"]);
    let e = refused(
        p.path(),
        &["summarize", "non-heap", "--backend", "go-goroutine", &h[0]],
    );
    assert_eq!(e["code"], "unsupported-format", "{e}");
    let e = refused(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "go-pprof-heap",
            &a,
        ],
    );
    assert_eq!(e["code"], "unsupported-format", "{e}");
}

#[test]
fn a_damaged_profile_is_refused_with_the_fix() {
    let p = Project::new();
    let mut bytes = fs::read(fixtures().join("go-pprof-heap/heap1.pb.gz")).unwrap();
    bytes.truncate(bytes.len() / 2);
    let a = p.put("in/cut.pb.gz", &bytes);
    // A truncated gzip stream still starts like a profile but does not decode: not sniffed.
    let e = refused(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "go-pprof-heap",
            &a,
        ],
    );
    assert!(
        e["code"] == "unsupported-format" || e["code"] == "malformed-artifact",
        "{e}"
    );
    assert!(e["fix"].as_str().is_some_and(|f| !f.is_empty()));
}
