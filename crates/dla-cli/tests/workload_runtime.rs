//! A workload blocker names one runtime and closes the queue of that runtime only. A project
//! with two runtimes that both have ranked candidates: a Python package and a C source file
//! (the case of a Python library with C extension modules).

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    AnswerStatus, Authority, ChecklistAnswer, EvidenceKind, HarnessManifest, Ledger, NewEvidence,
    Source, Tool, UnresolvedReason,
    evidence::{EvidenceCoverage, Signal},
};
use serde_json::{Value, json};

const BLOCKER: [&str; 8] = [
    "ledger",
    "resolve",
    "intake.workload",
    "--status",
    "unresolved",
    "--reason",
    "build_fails",
    "--remedy",
];

fn command(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1,2,3");
    c
}

struct R {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

impl R {
    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        (
            o.status.code().unwrap(),
            serde_json::from_str(&text).unwrap_or(Value::Null),
        )
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }
    /// `ledger resolve intake.workload --status unresolved --reason build_fails --remedy x`
    /// and `extra`.
    fn blocker(&self, extra: &[&str]) -> (i32, Value) {
        let mut a: Vec<&str> = BLOCKER.to_vec();
        a.push("native driver needs an embedding harness");
        a.extend_from_slice(extra);
        self.run(&a)
    }
}

/// Two runtimes with ranked candidates: python (a cached function, a dict) and native (a C
/// table with a writer and no remover).
fn two_runtimes() -> R {
    let dir = tempfile::tempdir().unwrap();
    for (rel, text) in [
        ("shop/__init__.py", ""),
        (
            "shop/cache.py",
            "import functools\n\n\n@functools.lru_cache(None)\ndef price_for(handler, key):\n    return (handler, key)\n",
        ),
        (
            "shop/reg.py",
            "class Registry:\n    _by_key = {}\n\n    def put(self, key, value):\n        Registry._by_key[key] = value\n",
        ),
        (
            "src/store.c",
            "#include <stdlib.h>\n#include <string.h>\n\nstruct entry { char *key; void *val; struct entry *next; };\nstatic struct entry *table[64];\n\nvoid store_put(const char *key, void *val) {\n    struct entry *e = malloc(sizeof *e);\n    e->key = strdup(key);\n    e->val = val;\n    e->next = table[0];\n    table[0] = e;\n}\n",
        ),
    ] {
        let p = dir.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let o = command(dir.path())
        .args(["run", "init", "--session-id", "t"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let r = R { dir, run_dir };
    for lang in ["python", "native"] {
        r.ok(&["sweep", "--lang", lang]);
        r.ok(&["census", "--lang", lang]);
    }
    let st = r.ok(&["run", "status"]);
    for rt in ["python", "native"] {
        assert_eq!(st["queue"][rt]["pending"], 2, "{st}");
        assert_eq!(st["queue"][rt]["open"], true, "{st}");
    }
    r
}

/// A python harness H1 that ran once and wrote series S1.
fn python_harness_ran(r: &R) {
    let m: HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": ["F1"], "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&r.run_dir).unwrap();
    let series = r.run_dir.join("series");
    fs::create_dir_all(&series).unwrap();
    fs::write(
        series.join("S1.json"),
        json!({"id": "S1", "source": {"kind": "harness", "harness": "H1"},
            "axis": "n", "columns": [], "points": []})
        .to_string(),
    )
    .unwrap();
    fs::write(
        r.run_dir.join("harness/H1/run-S1.json"),
        json!({"series": ["S1"]}).to_string(),
    )
    .unwrap();
}

/// A blocker with no runtime, as an older ledger holds it.
fn old_style_blocker(r: &R) {
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.answer_checklist(ChecklistAnswer {
        item: "intake.workload".into(),
        status: AnswerStatus::Unresolved,
        evidence: vec![],
        note: None,
        reason: Some(UnresolvedReason::BuildFails),
        remedy: Some("no driver".into()),
        runtime: None,
    })
    .unwrap();
}

#[test]
fn a_blocker_without_a_runtime_is_refused_and_lists_both_runtimes() {
    let r = two_runtimes();
    for reason in [
        "env_unavailable",
        "tool_missing",
        "build_fails",
        "privilege_declined",
    ] {
        let (code, v) = r.run(&[
            "ledger",
            "resolve",
            "intake.workload",
            "--status",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
        ]);
        assert_ne!(code, 0, "{reason}: {v}");
        assert_eq!(v["error"]["code"], "missing-argument", "{v}");
        let msg = v["error"]["message"].as_str().unwrap();
        assert!(msg.contains("--runtime is required"), "{msg}");
        assert!(
            msg.contains("native (2 pending)") && msg.contains("python (2 pending)"),
            "{msg}"
        );
    }
    // Nothing was recorded: both queues stay open.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["open"], true, "{st}");
    assert_eq!(st["queue"]["native"]["open"], true, "{st}");
}

#[test]
fn a_blocker_for_a_runtime_with_a_series_is_refused_and_one_for_the_other_is_accepted() {
    let r = two_runtimes();
    python_harness_ran(&r);
    let (code, v) = r.blocker(&["--runtime", "python"]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "workload-ran", "{v}");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("for python: harness H1 ran and wrote series S1"),
        "{msg}"
    );
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["native"]["open"], true, "{st}");
    // The other runtime has no series: its blocker is accepted.
    let v = r.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "build_fails",
        "--remedy",
        "native candidates are in extension modules",
        "--runtime",
        "native",
    ]);
    assert_eq!(v["ok"], true);
    let st = r.ok(&["run", "status"]);
    // The refusal to close a pending python candidate says the blocker is per runtime.
    let f = st["queue"]["python"]["next"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "target-pending", "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains("--remedy \"<text>\" --runtime python (that closes the queue of python only; the queues of other runtimes stay open)"),
        "{fix}"
    );
    assert_eq!(st["queue"]["native"]["closed"], "blocked", "{st}");
    assert_eq!(st["queue"]["native"]["open"], false, "{st}");
    assert_eq!(st["queue"]["python"]["open"], true, "{st}");
    assert!(st["queue"]["python"].get("closed").is_none(), "{st}");
    assert_eq!(st["queue"]["python"]["pending"], 2, "{st}");
}

#[test]
fn close_rest_after_a_native_blocker_closes_the_native_candidates_and_leaves_the_python_ones() {
    let r = two_runtimes();
    python_harness_ran(&r);
    r.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "build_fails",
        "--remedy",
        "x",
        "--runtime",
        "native",
    ]);
    let v = r.ok(&["ledger", "close-rest"]);
    assert_eq!(v["left_open"], 2, "{v}");
    let left: Vec<String> = v["left"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["why"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        left.iter()
            .all(|w| w.starts_with("pending in the python queue")),
        "{left:?}"
    );
    let cmd = "dla harness gen --lang python --driver --next";
    assert_eq!(v["batch_command"], cmd, "{v}");
    assert!(v["next"].as_str().unwrap().contains(cmd), "{v}");
    assert!(
        v["next"].as_str().unwrap().contains("2 candidate(s)"),
        "{v}"
    );
    // The native candidates are closed, the python ones are still candidates.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["pending"], 2, "{st}");
    assert_eq!(st["queue"]["native"]["pending"], 0, "{st}");
    assert_eq!(st["queue"]["python"]["open"], true, "{st}");
}

#[test]
fn an_answer_without_a_runtime_still_closes_every_queue() {
    let r = two_runtimes();
    old_style_blocker(&r);
    let st = r.ok(&["run", "status"]);
    for rt in ["python", "native"] {
        assert_eq!(st["queue"][rt]["closed"], "blocked", "{st}");
        assert_eq!(st["queue"][rt]["open"], false, "{st}");
    }
}

#[test]
fn a_blocker_with_one_runtime_with_candidates_records_that_runtime() {
    let dir = tempfile::tempdir().unwrap();
    for (rel, text) in [
        ("shop/__init__.py", ""),
        (
            "shop/reg.py",
            "class Registry:\n    _by_key = {}\n\n    def put(self, key, value):\n        Registry._by_key[key] = value\n",
        ),
    ] {
        let p = dir.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let o = command(dir.path())
        .args(["run", "init", "--session-id", "t"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let r = R { dir, run_dir };
    r.ok(&["sweep", "--lang", "python"]);
    r.ok(&["census", "--lang", "python"]);
    let (code, v) = r.blocker(&[]);
    assert_eq!(code, 0, "{v}");
    let l = Ledger::load(&r.run_dir).unwrap();
    let a = l.answers().get("intake.workload").unwrap();
    assert_eq!(a.runtime.as_deref(), Some("python"));
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["closed"], "blocked", "{st}");
}

#[test]
fn runtime_is_refused_on_other_items_on_findings_and_with_an_unknown_name() {
    let r = two_runtimes();
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "intake.scope",
        "--status",
        "answered",
        "--evidence",
        "E1",
        "--runtime",
        "python",
    ]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "unsupported-flag", "{v}");
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "intake.scope",
        "--status",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
        "--runtime",
        "python",
    ]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "unsupported-flag", "{v}");
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "env_unavailable",
        "--remedy",
        "x",
        "--runtime",
        "python",
    ]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "unsupported-flag", "{v}");
    for reason in ["build_fails", "not_exercised"] {
        let (code, v) = r.run(&[
            "ledger",
            "resolve",
            "intake.workload",
            "--status",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
            "--runtime",
            "go",
        ]);
        assert_ne!(code, 0, "{reason}: {v}");
        assert_eq!(v["error"]["code"], "invalid-value", "{v}");
        let missing = v["error"]["missing"].as_str().unwrap();
        assert!(
            missing.contains("native") && missing.contains("python"),
            "{v}"
        );
    }
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["open"], true, "{st}");
}

#[test]
fn a_reason_that_is_not_a_blocker_with_a_runtime_is_stored_for_that_runtime() {
    let r = two_runtimes();
    let v = r.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
        "--runtime",
        "native",
    ]);
    assert_eq!(v["ok"], true);
    let l = Ledger::load(&r.run_dir).unwrap();
    let a = l.answers().get("intake.workload").unwrap();
    assert_eq!(a.runtime.as_deref(), Some("native"));
    // Not a blocker: no queue is closed by it.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["native"]["open"], true, "{st}");
}

/// A python test loop H1 that `dla harness gen` wrote (its manifest and the `workload` evidence)
/// and nothing ran.
fn python_test_loop_generated(r: &R) {
    let m: HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": [], "agent": "driver-python", "timeout_secs": 60,
        "tools": ["pytest-loop"],
    }))
    .unwrap();
    m.save(&r.run_dir).unwrap();
    let mut result = json!({"harness": "H1", "mode": "driver"});
    result["signal"] = Signal::new("t", "x", "src").to_value();
    Ledger::load(&r.run_dir)
        .unwrap()
        .add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "abc".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "0".into(),
            },
            authority: Authority::Test,
            kind: EvidenceKind::Workload,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
}

#[test]
fn a_blocker_for_python_is_refused_while_its_test_loop_has_no_end_and_one_for_native_is_accepted() {
    let r = two_runtimes();
    python_test_loop_generated(&r);
    let (code, v) = r.blocker(&["--runtime", "python"]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "loop-not-finished", "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(fix.contains("dla harness run H1"), "{fix}");
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["open"], true, "{st}");
    // The loop of python says nothing about the other runtime.
    let (code, v) = r.blocker(&["--runtime", "native"]);
    assert_eq!(code, 0, "{v}");
    // A failed attempt the tool recorded ends the loop: the python blocker is accepted.
    fs::write(
        r.run_dir.join("harness/H1/attempt.json"),
        json!({"harness": "H1", "started_at": "2026-01-01T00:00:00Z",
            "finished_at": "2026-01-01T00:00:01Z", "outcome": "failed",
            "reason": "build_fails", "message": "x"})
        .to_string(),
    )
    .unwrap();
    let (code, v) = r.blocker(&["--runtime", "python"]);
    assert_eq!(code, 0, "{v}");
}
