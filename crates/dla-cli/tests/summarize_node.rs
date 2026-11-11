//! `dla summarize` over Node data: V8 heap snapshots (heap-graph, type-histogram-diff),
//! sampling heap profiles (alloc-stack-diff) and the harness's `resources.json`
//! (non-heap). The inputs are real tool output, trimmed (tests/fixtures/summarize/node-*).
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

    /// A project with the sources of the node fixtures and their census.
    fn with_sources() -> Self {
        let p = Self::new();
        let src = fixture("node-heapsnapshot/project/src");
        for e in fs::read_dir(src).unwrap().flatten() {
            let bytes = fs::read(e.path()).unwrap();
            p.put(&format!("src/{}", e.file_name().to_string_lossy()), &bytes);
        }
        ok(p.path(), &["census", "--lang", "node"]);
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

    fn findings(&self) -> Vec<Value> {
        self.lines()
            .into_iter()
            .filter(|e| e["type"] == "finding_added")
            .map(|e| e["finding"].clone())
            .collect()
    }

    fn finding_with_owner(&self, owner: &str) -> Value {
        self.findings()
            .into_iter()
            .find(|f| f["owner"] == owner)
            .unwrap_or_else(|| panic!("no finding with owner {owner}: {:?}", self.findings()))
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

fn heap_graph(p: &Project, files: &[String], extra: &[&str]) -> Value {
    let mut args = vec!["summarize", "heap-graph", "--backend", "node-heapsnapshot"];
    args.extend_from_slice(extra);
    args.extend(files.iter().map(String::as_str));
    ok(p.path(), &args)
}

fn retry(p: &Project) -> Vec<String> {
    p.put_fixture(
        "node-heapsnapshot",
        &[
            "heap-1.heapsnapshot",
            "heap-2.heapsnapshot",
            "heap-3.heapsnapshot",
        ],
        "retry",
    )
}

fn heartbeat(p: &Project) -> Vec<String> {
    p.put_fixture(
        "node-heapsnapshot",
        &[
            "hb-1.heapsnapshot",
            "hb-2.heapsnapshot",
            "hb-3.heapsnapshot",
        ],
        "hb",
    )
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

const SECRET: &str = "AKIAIOSFODNN7EXAMPLE1";

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
fn the_node_adapters_conform() {
    conforms("node-heapsnapshot", "heap-graph");
    conforms("node-heapsnapshot-types", "type-histogram-diff");
    conforms("node-heapprofile", "alloc-stack-diff");
    conforms("node-resources", "non-heap");
}

#[test]
fn auto_picks_the_node_adapters_by_format() {
    let p = Project::new();
    let files = retry(&p);
    let mut args = vec!["summarize", "heap-graph"];
    args.extend(files.iter().map(String::as_str));
    assert_eq!(ok(p.path(), &args)["backend"], "node-heapsnapshot");
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(files.iter().map(String::as_str));
    assert_eq!(ok(p.path(), &args)["backend"], "node-heapsnapshot-types");
    let prof = p.put_fixture(
        "node-heapprofile",
        &["profile-1.heapprofile", "profile-2.heapprofile"],
        "p",
    );
    let mut args = vec!["summarize", "alloc-stack-diff"];
    args.extend(prof.iter().map(String::as_str));
    assert_eq!(ok(p.path(), &args)["backend"], "node-heapprofile");
    let res = p.put_fixture("node-resources", &["resources-1.json"], "r");
    assert_eq!(
        ok(p.path(), &["summarize", "non-heap", &res[0]])["backend"],
        "node-resources"
    );
}

// ---------------------------------------------------------------------------------------
// heap-graph: holders and ties
// ---------------------------------------------------------------------------------------

#[test]
fn a_module_variable_holder_is_static_global_and_ties_to_the_census_finding() {
    let p = Project::with_sources();
    let shared = p.finding_with_owner("src.scheduler.shared");
    assert_eq!(shared["holder"], "static/global");
    let f = shared["id"].as_str().unwrap().to_owned();
    let files = retry(&p);
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["passed"][0], "static:src.scheduler.shared");
    assert!(
        v["tied"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| *t == f.as_str()),
        "{v}"
    );
    assert_eq!(v["observed_holder"][&f], "static/global");
    let g = p.evidence("growth-series").pop().unwrap();
    let col = &g["result"]["passed"][0];
    assert_eq!(col["column"], "holder:static:src.scheduler.shared");
    assert_eq!(col["finding"], f.as_str());
    assert_eq!(col["tie"]["rule"], "holder");
    assert_eq!(col["tie"]["key"], "src.scheduler.shared");
    assert!(ledger_rule_ties(
        &g,
        &f,
        shared["site"].as_str(),
        Some("src.scheduler.shared")
    ));
    // The instance field below it ties to its own census finding, not to the variable's.
    // Its root is the module variable, so the holder observed is static/global.
    let latest = p.finding_with_owner("src.scheduler.RetryScheduler.latest");
    assert_eq!(latest["holder"], "container-field");
    let lid = latest["id"].as_str().unwrap();
    assert_eq!(v["observed_holder"][lid], "static/global", "{v}");
    let lcol = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["finding"] == lid)
        .unwrap_or_else(|| panic!("{g}"));
    assert_eq!(lcol["holder_kind"], "static/global");
    assert_eq!(col["holder_kind"], "static/global");
    assert!(
        v["tied"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| *t == latest["id"]),
        "{v}"
    );
    // The series file carries the tie, so `dla trend` sees it.
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
        .find(|c| c["name"] == "holder:static:src.scheduler.shared")
        .unwrap();
    assert_eq!(c["finding"], f.as_str());
    // No duplicate finding was made for a holder that has an owner.
    assert!(
        p.findings()
            .iter()
            .filter(|x| x["owner"] == "src.scheduler.shared")
            .count()
            == 1
    );
}

#[test]
fn a_timer_holder_is_a_registration_table_and_ties_to_the_timer_finding() {
    let p = Project::with_sources();
    let timer = p.finding_with_owner("src.heartbeat.startHeartbeat");
    // The census calls a setInterval a blocked stack; the snapshots show a registration.
    assert_eq!(timer["holder"], "blocked-stack");
    let f = timer["id"].as_str().unwrap().to_owned();
    let files = heartbeat(&p);
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(
        v["passed"][0],
        "registry:timers:src.heartbeat.startHeartbeat.beat"
    );
    assert!(
        v["tied"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| *t == f.as_str()),
        "{v}"
    );
    assert_eq!(v["observed_holder"][&f], "registration-table");
    let needed = v["needed"].as_str().unwrap();
    // The finding has another holder: the annotation that corrects it cites the evidence.
    let g = p.evidence("growth-series").pop().unwrap();
    let gid = v["growth_evidence"].as_str().unwrap();
    assert!(
        needed.contains(&format!(
            "dla ledger annotate {f} --holder registration-table --evidence {gid}"
        )),
        "{needed}"
    );
    // The column tied to the finding carries the holder kind the ledger reads.
    let tied: Vec<&Value> = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["finding"] == f.as_str())
        .collect();
    assert!(!tied.is_empty());
    assert!(
        tied.iter()
            .all(|c| c["holder_kind"] == "registration-table")
    );
    assert!(ledger_rule_ties(
        &g,
        &f,
        timer["site"].as_str(),
        Some("src.heartbeat.startHeartbeat")
    ));
    // The closure variable below the timer is its own candidate: a closure/continuation.
    let c = v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["owner"] == "src.heartbeat.startHeartbeat.session")
        .unwrap_or_else(|| panic!("{v}"));
    // Its root is the timer, so the kind observed is the registration, not the closure.
    assert_eq!(c["holder"], "registration-table");
    let col = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"].as_str().is_some_and(|n| n.contains("closure:")))
        .unwrap_or_else(|| panic!("{g}"));
    assert_eq!(col["holder_kind"], "registration-table");
}

#[test]
fn finding_narrows_the_tie_and_forces_nothing() {
    let p = Project::with_sources();
    let shared = p.finding_with_owner("src.scheduler.shared")["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let timer = p.finding_with_owner("src.heartbeat.startHeartbeat")["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let files = retry(&p);
    // The timer finding does not own any holder of the scheduler's snapshots.
    let v = heap_graph(&p, &files, &["--finding", &timer]);
    assert_eq!(v["gate"], "pass");
    assert_eq!(v["tied"].as_array().unwrap().len(), 0, "{v}");
    assert!(v["needed"].as_str().unwrap().contains(&timer), "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    assert!(g["result"]["passed"][0].get("finding").is_none());
    // With the finding that owns the holder, only it is tied.
    let v = heap_graph(&p, &files, &["--finding", &shared]);
    assert_eq!(v["tied"], json!([shared]));
}

#[test]
fn an_owner_with_no_finding_becomes_a_candidate_without_a_site() {
    let p = Project::new();
    p.put(
        "src/scheduler.js",
        &fs::read(fixture("node-heapsnapshot/project/src/scheduler.js")).unwrap(),
    );
    // No census ran: nothing owns src.scheduler.shared.
    let files = retry(&p);
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    let c = v["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["owner"] == "src.scheduler.shared")
        .unwrap_or_else(|| panic!("{v}"));
    assert_eq!(c["holder"], "static/global");
    let f = p.finding_with_owner("src.scheduler.shared");
    assert!(f["site"].is_null());
    assert_eq!(f["region"], "heap");
    // A second run ties to the finding it made.
    let again = heap_graph(&p, &files, &[]);
    assert!(
        again["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["owner"] != "src.scheduler.shared")
    );
}

// ---------------------------------------------------------------------------------------
// heap-graph: refusals
// ---------------------------------------------------------------------------------------

#[test]
fn snapshots_of_separate_processes_are_not_a_series() {
    let p = Project::new();
    let files = retry(&p);
    // The harness writes one process into one pid directory.
    for (i, f) in files.iter().enumerate() {
        let bytes = fs::read(p.path().join(f)).unwrap();
        p.put(
            &format!("cap/{}/heap-{}.heapsnapshot", 1000 + i, i + 1),
            &bytes,
        );
    }
    let separate: Vec<String> = (0..3)
        .map(|i| format!("cap/{}/heap-{}.heapsnapshot", 1000 + i, i + 1))
        .collect();
    let e = refused(
        p.path(),
        &[
            "summarize",
            "heap-graph",
            &separate[0],
            &separate[1],
            &separate[2],
        ],
    );
    assert_eq!(e["code"], "dumps-not-comparable");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("different processes"),
        "{e}"
    );
    assert!(e["fix"].as_str().unwrap().contains("--capture heap"), "{e}");
    // The same pid directory is accepted.
    for (i, f) in files.iter().enumerate() {
        let bytes = fs::read(p.path().join(f)).unwrap();
        p.put(&format!("cap/2000/heap-{}.heapsnapshot", i + 1), &bytes);
    }
    let same: Vec<String> = (1..=3)
        .map(|i| format!("cap/2000/heap-{i}.heapsnapshot"))
        .collect();
    assert_eq!(heap_graph(&p, &same, &[])["gate"], "pass");
    // Histograms of separate processes are fine: they compare by class only.
    let mut args = vec![
        "summarize",
        "type-histogram-diff",
        "--backend",
        "node-heapsnapshot-types",
    ];
    args.extend(separate.iter().map(String::as_str));
    ok(p.path(), &args);
}

#[test]
fn the_same_snapshot_twice_and_unrelated_snapshots_are_refused() {
    let p = Project::new();
    let r = retry(&p);
    let hb = heartbeat(&p);
    // Another program's snapshot after the first: its ids do not continue the first's.
    let e = refused(p.path(), &["summarize", "heap-graph", &r[0], &hb[2], &r[2]]);
    assert_eq!(e["code"], "dumps-not-comparable");
    // Copies of one snapshot.
    let copy = p.put(
        "in/copy.heapsnapshot",
        &fs::read(p.path().join(&r[0])).unwrap(),
    );
    let e = refused(p.path(), &["summarize", "heap-graph", &r[0], &copy, &r[1]]);
    assert_eq!(e["code"], "dumps-not-comparable");
    assert!(
        e["message"].as_str().unwrap().contains("same object id"),
        "{e}"
    );
}

#[test]
fn a_damaged_snapshot_is_refused_with_the_fix() {
    let p = Project::new();
    let files = retry(&p);
    let bytes = fs::read(p.path().join(&files[2])).unwrap();
    let cut = p.put("in/cut.heapsnapshot", &bytes[..bytes.len() / 2]);
    let e = refused(
        p.path(),
        &["summarize", "heap-graph", &files[0], &files[1], &cut],
    );
    assert_eq!(e["code"], "malformed-artifact");
    assert!(e["fix"].as_str().unwrap().contains("snapshot again"), "{e}");
    // Not a snapshot at all, named like one.
    let junk = p.put("in/junk.heapsnapshot", b"{\"hello\": [1, 2, 3]}");
    let e = refused(
        p.path(),
        &[
            "summarize",
            "heap-graph",
            "--backend",
            "node-heapsnapshot",
            &junk,
        ],
    );
    assert_eq!(e["code"], "unsupported-format");
    // A file whose edges point outside the node array.
    let text = String::from_utf8(bytes).unwrap();
    let bad = text.replacen("\"edges\":[", "\"edges\":[0,0,99999999,", 1);
    let bad = p.put("in/bad.heapsnapshot", bad.as_bytes());
    let e = refused(
        p.path(),
        &[
            "summarize",
            "heap-graph",
            "--backend",
            "node-heapsnapshot",
            &bad,
        ],
    );
    assert_eq!(e["code"], "malformed-artifact");
}

#[test]
fn fewer_than_three_snapshots_give_the_evidence_and_say_what_is_missing() {
    let p = Project::new();
    let files = retry(&p);
    let v = heap_graph(&p, &files[..2], &[]);
    assert_eq!(v["gate"], "fail");
    assert_eq!(v["reasons"][0], "snapshots < 3");
    assert!(
        v["needed"].as_str().unwrap().contains("1 more snapshot"),
        "{v}"
    );
    let one = heap_graph(&p, &files[..1], &[]);
    assert!(one["needed"].as_str().unwrap().contains("2 more snapshots"));
    assert!(one["series"].is_null());
    assert_eq!(p.evidence("retention-path").len(), 3);
}

#[test]
fn retention_path_holders_state_the_kind_of_their_retention_root() {
    let p = Project::new();
    let files = retry(&p);
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    let growth = &p.evidence("growth-series")[0];
    for r in p.evidence("retention-path") {
        let holders = r["result"]["holders"].as_array().unwrap();
        assert!(!holders.is_empty());
        for h in holders {
            assert!(h["holder_kind"].is_string(), "{h}");
            // The same value the growth column of that holder carries.
            let col = format!("holder:{}", h["key"].as_str().unwrap());
            if let Some(c) = growth["result"]["passed"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["column"] == col.as_str())
            {
                assert_eq!(c["holder_kind"], h["holder_kind"], "{c} {h}");
            }
        }
    }
}

#[test]
fn holders_are_redacted_in_the_summaries_and_the_series() {
    let p = Project::new();
    let files = retry(&p);
    // The fixture names the property `rows` as a key-like string.
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["redaction"], "shapes");
    for (path, t) in p.run_texts() {
        assert!(!t.contains(SECRET), "{}", path.display());
        // The path of the machine that took the snapshot is never stored.
        assert!(!t.contains("/srv/webhook"), "{}", path.display());
    }
}

// ---------------------------------------------------------------------------------------
// type-histogram-diff, alloc-stack-diff, non-heap
// ---------------------------------------------------------------------------------------

#[test]
fn the_class_histogram_diffs_snapshots_and_ties_to_nothing() {
    let p = Project::with_sources();
    let files = retry(&p);
    let mut args = vec![
        "summarize",
        "type-histogram-diff",
        "--backend",
        "node-heapsnapshot-types",
    ];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["captures"], 3);
    assert_eq!(v["tied"].as_array().unwrap().len(), 0, "{v}");
    let top = v["top"].as_array().unwrap();
    assert!(top.iter().any(|t| t["type"] == "Object"), "{v}");
    // String text is data: a string is only ever the class `(string)`.
    assert!(!p.run_texts().iter().any(|(_, t)| t.contains(SECRET)));
}

#[test]
fn a_bundle_profile_is_mapped_through_its_source_map() {
    let p = Project::with_sources();
    let root = p.path().to_string_lossy().replace('\\', "/");
    p.put("out/app.js", b"function o(e){}o(1);\n");
    p.put(
        "out/app.js.map",
        br#"{"version":3,"sources":["../src/scheduler.js"],"mappings":"AAAA,uBACe"}"#,
    );
    let frame = |name: &str, url: &str, line: i64, col: i64| {
        json!({"callFrame": {"functionName": name, "scriptId": "9", "url": url, "lineNumber": line, "columnNumber": col},
               "selfSize": 0, "id": 0, "children": []})
    };
    let profile = |bytes: u64| {
        let mut leaf = frame("push", "", -1, -1);
        leaf["selfSize"] = json!(bytes);
        leaf["id"] = json!(3);
        let url = if root.starts_with('/') {
            format!("file://{root}/out/app.js")
        } else {
            format!("file:///{root}/out/app.js")
        };
        let mut o = frame("o", &url, 0, 25);
        o["id"] = json!(2);
        o["children"] = json!([leaf]);
        let mut head = frame("(root)", "", -1, -1);
        head["id"] = json!(1);
        head["children"] = json!([o]);
        json!({"head": head, "samples": [{"size": bytes, "nodeId": 3, "ordinal": 1}]}).to_string()
    };
    let files: Vec<String> = [1000u64, 2000, 3000]
        .iter()
        .enumerate()
        .map(|(i, b)| p.put(&format!("in/p{i}.heapprofile"), profile(*b).as_bytes()))
        .collect();
    let mut args = vec![
        "summarize",
        "alloc-stack-diff",
        "--backend",
        "node-heapprofile",
    ];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["source_maps"]["frames_mapped"], 3, "{v}");
    assert_eq!(v["top"][0]["site"], "src/scheduler.js:2", "{v}");
    assert_eq!(v["sampled"], true);
    // Windows are not cumulative: rising per-window bytes are not retention growth, so
    // no growth column exists and nothing is tied.
    assert_eq!(v["gate"], "fail", "{v}");
    assert_eq!(v["tied"].as_array().map_or(0, Vec::len), 0, "{v}");
    assert!(v["series"].is_null(), "{v}");
    assert!(v["needed"].as_str().unwrap().contains("heap-graph"), "{v}");
    assert!(p.evidence("growth-series").is_empty());
}

#[test]
fn a_timer_group_ties_to_the_finding_at_its_creation_site() {
    let p = Project::with_sources();
    let timer = p.finding_with_owner("src.heartbeat.startHeartbeat");
    let f = timer["id"].as_str().unwrap().to_owned();
    assert_eq!(timer["site"], "src/heartbeat.js:15");
    let files: Vec<String> = (1..=3)
        .map(|i| {
            let text = json!({
                "dla_resources": 1, "pid": 100 + i, "runtime": "node", "x": null, "t_ms": 10 * i,
                "active": {"TCPSocketWrap": 2},
                "types": {"Timeout": {"active": 0, "tracked": 40 * i, "created": null, "source": "timers",
                                      "sites": {"src/heartbeat.js:15": 40 * i}, "unattributed": 0}},
                "limitations": []
            });
            p.put(&format!("in/res/resources-{i}.json"), text.to_string().as_bytes())
        })
        .collect();
    let mut args = vec!["summarize", "non-heap", "--backend", "node-resources"];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([f]), "{v}");
    let g = p.evidence("growth-series").pop().unwrap();
    let col = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"].as_str().unwrap().starts_with("group:"))
        .unwrap_or_else(|| panic!("{g}"));
    assert_eq!(col["tie"]["rule"], "site");
    assert_eq!(col["site"], "src/heartbeat.js:15");
    // The whole-process active counts tie to nothing.
    assert!(
        g["result"]["passed"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["column"].as_str().unwrap().starts_with("metric:"))
            .all(|c| c.get("finding").is_none())
    );
}

#[test]
fn a_resources_file_without_its_version_is_refused() {
    let p = Project::new();
    let bad = p.put(
        "in/resources.json",
        b"{\"dla_resources\":null,\"types\":{}}",
    );
    let e = refused(
        p.path(),
        &["summarize", "non-heap", "--backend", "node-resources", &bad],
    );
    assert_eq!(e["code"], "malformed-artifact");
    assert!(e["fix"].as_str().unwrap().contains("--capture nonheap"));
}

// ---------------------------------------------------------------------------------------
// per-point series: harness captures of command mode, one process per point
// ---------------------------------------------------------------------------------------

impl Project {
    /// Copies a fixture snapshot in as the heap snapshot of point `label` of harness H1
    /// (its own pid directory) and lists it in `captures.json` with `x`, the way
    /// `dla harness run H1 --capture heap` does. Returns the project-relative path.
    fn capture(&self, label: &str, pid: u32, x: Option<f64>, src: &str) -> String {
        let run_rel = format!("harness/H1/out/{label}/capture/{pid}/heap-1.heapsnapshot");
        let abs = self.run_dir.join(&run_rel);
        fs::create_dir_all(abs.parent().unwrap()).unwrap();
        fs::copy(self.path().join(src), &abs).unwrap();
        let list = self.run_dir.join("harness/H1/captures.json");
        let mut v: Value = fs::read_to_string(&list)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_else(|| serde_json::json!({"files": []}));
        let mut entry = serde_json::json!({
            "kind": "heap", "path": run_rel, "label": label, "seq": 1,
            "bytes": fs::metadata(&abs).unwrap().len(),
            "captured_at": "2026-01-01T00:00:00Z",
        });
        if let Some(x) = x {
            entry["x"] = serde_json::json!(x);
        }
        v["files"].as_array_mut().unwrap().push(entry);
        fs::write(&list, serde_json::to_string_pretty(&v).unwrap()).unwrap();
        format!(
            ".dla/{}/{run_rel}",
            self.run_dir.file_name().unwrap().to_string_lossy()
        )
    }
}

#[test]
fn per_point_harness_snapshots_form_a_series_over_n_and_tie_by_holder() {
    let p = Project::with_sources();
    let shared = p.finding_with_owner("src.scheduler.shared");
    let f = shared["id"].as_str().unwrap().to_owned();
    let src = retry(&p);
    // Separate processes: the snapshots are listed out of order of n.
    let files = vec![
        p.capture("p9-native", 30, Some(900.0), &src[2]),
        p.capture("p1-native", 10, Some(100.0), &src[0]),
        p.capture("p5-native", 20, Some(500.0), &src[1]),
    ];
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["series_kind"], "per-point");
    assert_eq!(v["n_values"], serde_json::json!([100.0, 500.0, 900.0]));
    assert_eq!(v["passed"][0], "static:src.scheduler.shared");
    assert!(
        v["tied"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| *t == f.as_str()),
        "{v}"
    );
    assert_eq!(v["observed_holder"][&f], "static/global");
    let s: Value = serde_json::from_str(
        &fs::read_to_string(
            p.run_dir
                .join(format!("series/{}.json", v["series"].as_str().unwrap())),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(s["axis"], "n");
    let xs: Vec<f64> = s["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| q["x"].as_f64().unwrap())
        .collect();
    assert_eq!(xs, [100.0, 500.0, 900.0]);
    assert!(
        s["limitations"][0]
            .as_str()
            .unwrap()
            .contains("series: per-point")
    );
    let c = s["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "holder:static:src.scheduler.shared")
        .unwrap();
    assert_eq!(c["finding"], f.as_str());
    let g = p.evidence("growth-series").pop().unwrap();
    assert_eq!(g["result"]["axis"], "n");
    assert_eq!(g["result"]["series_kind"], "per-point");
    assert_eq!(g["result"]["passed"][0]["finding"], f.as_str());
    assert_eq!(g["result"]["passed"][0]["tie"]["rule"], "holder");
}

#[test]
fn a_per_point_snapshot_series_that_mixes_modes_or_repeats_x_is_refused() {
    let msg = |e: &Value| e["message"].as_str().unwrap().to_owned();
    let case = |points: [(&str, Option<f64>); 3]| {
        let p = Project::new();
        let src = retry(&p);
        let files: Vec<String> = points
            .iter()
            .enumerate()
            .map(|(i, (label, x))| p.capture(label, 10 + i as u32, *x, &src[i]))
            .collect();
        let mut args = vec!["summarize", "heap-graph", "--backend", "node-heapsnapshot"];
        args.extend(files.iter().map(String::as_str));
        let e = refused(p.path(), &args);
        assert_eq!(e["code"], "dumps-not-comparable", "{e}");
        assert!(!p.run_dir.join("series").exists());
        e
    };
    let e = case([
        ("p1-native", Some(100.0)),
        ("p5-native", Some(500.0)),
        ("p1-fresh", Some(900.0)),
    ]);
    assert!(msg(&e).contains("driver mode"), "{e}");
    let e = case([
        ("p1-native", Some(100.0)),
        ("p5-native", Some(100.0)),
        ("p9-native", Some(900.0)),
    ]);
    assert!(msg(&e).contains("same n"), "{e}");
    let e = case([
        ("p1-native", Some(100.0)),
        ("p1-native", Some(500.0)),
        ("p9-native", Some(900.0)),
    ]);
    assert!(msg(&e).contains("same point"), "{e}");
    let e = case([
        ("p1-native", Some(100.0)),
        ("p5-native", None),
        ("p9-native", Some(900.0)),
    ]);
    assert!(msg(&e).contains("no recorded n"), "{e}");
}

#[test]
fn snapshots_not_listed_as_harness_captures_keep_the_pid_directory_rule() {
    // The same layout as a harness run, but captures.json does not list the files (an
    // artifact run or an attach): separate pid directories are separate processes.
    let p = Project::new();
    let src = retry(&p);
    let files: Vec<String> = (0..3)
        .map(|i| {
            let rel = format!(
                "harness/H1/out/p{}-native/capture/{}/heap-1.heapsnapshot",
                i + 1,
                10 + i
            );
            let abs = p.run_dir.join(&rel);
            fs::create_dir_all(abs.parent().unwrap()).unwrap();
            fs::copy(p.path().join(&src[i]), &abs).unwrap();
            format!(
                ".dla/{}/{rel}",
                p.run_dir.file_name().unwrap().to_string_lossy()
            )
        })
        .collect();
    let mut args = vec!["summarize", "heap-graph", "--backend", "node-heapsnapshot"];
    args.extend(files.iter().map(String::as_str));
    let e = refused(p.path(), &args);
    assert_eq!(e["code"], "dumps-not-comparable");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("different processes"),
        "{e}"
    );
}

// ---------------------------------------------------------------------------------------
// a closure variable dominated by larger holders, in the scratch copy of a harness

const BOARD_JS: &str = r#"'use strict';
class Board {
  constructor() { this.latest = null; this.sequence = 0; }
  publish(symbol, ticks) {
    const previous = this.latest;
    const changeNote = () => (previous ? `${symbol}: ${previous.count}` : `${symbol}: first`);
    const snapshot = {
      id: ++this.sequence, symbol, count: ticks.length, ticks,
      describe() { return `${symbol}#${snapshot.id} ${changeNote.name}`; },
    };
    this.latest = snapshot;
    return snapshot;
  }
}
module.exports = (() => {
  const board = new Board();
  return { publish: (...args) => board.publish(...args) };
})();
"#;

const LOAD_JS: &str = r#"'use strict';
const board = require('./src/board');
const i = process.argv.indexOf('--iterations');
const n = i > 0 ? Number(process.argv[i + 1]) : 10;
for (let k = 0; k < n; k += 1) board.publish(`S${k % 8}`, Array.from({ length: 40 }, (_, j) => j + 0.5));
"#;

const GEN_JS: &str = r#"'use strict';
const v8 = require('node:v8');
const board = require('./src/board');
const n = Number(process.argv[2]);
for (let i = 0; i < n; i += 1) {
  board.publish(`S${i % 8}`, Array.from({ length: 40 }, (_, k) => k + i + 0.5));
}
v8.writeHeapSnapshot(process.argv[3]);
"#;

/// Four per-point snapshots (real V8 output) of a program whose newest snapshot object is
/// held by `Board.latest`, and whose chain of older ones hangs from the context variable
/// `previous`. They are taken from the scratch copy of a harness, as `dla harness run
/// --capture heap` takes them. `None` when node is not installed.
fn board_snapshots(p: &Project) -> Option<Vec<String>> {
    std::process::Command::new("node")
        .arg("--version")
        .output()
        .ok()?;
    // The project, and the harness `dla harness gen` writes for it (id H1).
    p.put(
        "package.json",
        br#"{"name":"board","scripts":{"load":"node load.js"}}"#,
    );
    p.put("src/board.js", BOARD_JS.as_bytes());
    p.put("load.js", LOAD_JS.as_bytes());
    ok(p.path(), &["workload", "discover", "--lang", "node"]);
    ok(p.path(), &["harness", "gen", "--lang", "node"]);
    let work = p.run_dir.join("harness/H1/work");
    fs::create_dir_all(work.join("src")).unwrap();
    fs::write(work.join("src/board.js"), BOARD_JS).unwrap();
    fs::write(work.join("gen.js"), GEN_JS).unwrap();
    let mut files = Vec::new();
    for (k, n) in [200u32, 400, 600, 800].into_iter().enumerate() {
        let out = p.path().join(format!("in/gen/heap-{n}.heapsnapshot"));
        fs::create_dir_all(out.parent().unwrap()).unwrap();
        let st = std::process::Command::new("node")
            .arg("gen.js")
            .arg(n.to_string())
            .arg(&out)
            .current_dir(&work)
            .status()
            .unwrap();
        assert!(st.success());
        let rel = format!("in/gen/heap-{n}.heapsnapshot");
        files.push(p.capture(
            &format!("p{n}-native"),
            100 + k as u32,
            Some(f64::from(n)),
            &rel,
        ));
    }
    Some(files)
}

fn retention_holders(p: &Project, v: &Value) -> Vec<Vec<Value>> {
    v["dumps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            let id = d["evidence"].as_str().unwrap();
            p.lines()
                .into_iter()
                .find(|e| e["type"] == "evidence" && e["evidence_id"] == id)
                .unwrap()["result"]["holders"]
                .as_array()
                .unwrap()
                .clone()
        })
        .collect()
}

#[test]
fn the_holder_of_the_finding_is_in_every_retention_record_and_the_label_is_accepted() {
    let p = Project::new();
    let Some(files) = board_snapshots(&p) else {
        eprintln!("node is not installed: skipped");
        return;
    };
    // The owners are named relative to the harness work copy, as the census names them.
    let first = heap_graph(&p, &files, &[]);
    assert_eq!(first["gate"], "pass", "{first}");
    let f = p.finding_with_owner("src.board.publish.previous");
    let f = f["id"].as_str().unwrap().to_owned();
    assert!(
        first["passed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|k| k == "closure:src.board.publish.previous"),
        "{first}"
    );
    // Without --finding the record lists the three largest holders: `previous` is
    // dominated by larger ones and is not among them.
    for holders in retention_holders(&p, &first) {
        assert_eq!(holders.len(), 3, "{holders:?}");
        assert!(
            holders
                .iter()
                .all(|h| h["key"] != "closure:src.board.publish.previous"),
            "{holders:?}"
        );
    }
    // With --finding the record also lists the finding's holder, with its holder_kind.
    let v = heap_graph(&p, &files, &["--finding", &f]);
    let records = retention_holders(&p, &v);
    assert_eq!(records.len(), 4);
    let mut cited = Vec::new();
    for (holders, d) in records.iter().zip(v["dumps"].as_array().unwrap()) {
        assert_eq!(holders.len(), 4, "{holders:?}");
        let h = holders
            .iter()
            .find(|h| h["key"] == "closure:src.board.publish.previous")
            .unwrap_or_else(|| panic!("the bound holder is missing: {holders:?}"));
        assert_eq!(h["holder_kind"], "closure/continuation");
        cited.push(d["evidence"].as_str().unwrap().to_owned());
    }
    assert!(
        v["tied"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| *t == f.as_str())
    );
    let growth = v["growth_evidence"].as_str().unwrap().to_owned();
    // The model's route to a label: a refuted warm-up sibling, the holder and the site,
    // the confirmation, then the label citing the retention record.
    let disc = p
        .evidence("discriminator")
        .into_iter()
        .rev()
        .find(|e| e["result"]["finding"] == f.as_str())
        .unwrap()["evidence_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let support = format!("{growth},{disc}");
    ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--sibling-of",
            &f,
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "retained bytes plateau",
            "--evidence",
            &disc,
        ],
    );
    ok(
        p.path(),
        &["ledger", "refute", &format!("{f}.n1"), "--evidence", &disc],
    );
    ok(
        p.path(),
        &["ledger", "annotate", &f, "--site", "src/board.js:15"],
    );
    let harness = p.evidence("workload").pop().unwrap()["evidence_id"]
        .as_str()
        .unwrap()
        .to_owned();
    ok(
        p.path(),
        &[
            "ledger",
            "test",
            &f,
            "--evidence",
            &format!("{support},{harness}"),
        ],
    );
    ok(p.path(), &["ledger", "confirm", &f, "--evidence", &support]);
    let retention = &cited[0];
    let labelled = ok(
        p.path(),
        &[
            "ledger",
            "label",
            &f,
            "--as",
            "L-CAPTURE",
            "--evidence",
            &format!("{growth},{retention}"),
        ],
    );
    assert_eq!(labelled["label"], "L-CAPTURE");
    // `strong` is not refused for the holder: the record names it, with the kind the
    // finding has. It is refused for independence (one tool, one tool class).
    let (code, _, s) = run(p.path(), &["ledger", "strong", &f, "--evidence", retention]);
    assert_eq!(code, 2);
    assert_eq!(s["error"]["code"], "strong-not-independent", "{s}");
}
