//! `dla attach bpftrace`: the tier-3 consent gate and the named prerequisite gaps. The tool
//! itself is not run here (bpftrace needs root, tracefs and BTF); the backend's own tests
//! drive a fake program.
#![cfg(target_os = "linux")]

use std::{
    path::Path,
    process::{Child, Command as Proc, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "DLA_DATA_DIR",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn call(project: &Path, path_env: Option<&Path>, args: &[&str]) -> (i32, Value) {
    let mut c = dla(project);
    if let Some(p) = path_env {
        c.env("PATH", p);
    }
    let out = c.args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), v)
}

struct Sleeper(Child);

impl Drop for Sleeper {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn sleeper() -> Sleeper {
    Sleeper(
        Proc::new("sleep")
            .arg("300")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    )
}

fn init(project: &Path, tiers: &str) {
    let (code, v) = call(project, None, &["run", "init", "--session-id", "t"]);
    assert_eq!(code, 0, "{v}");
    let (code, v) = call(project, None, &["consent", "--tiers", tiers]);
    assert_eq!(code, 0, "{v}");
}

#[test]
fn tier_two_consent_does_not_cover_bpftrace() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path(), "1-2");
    let target = sleeper();
    let pid = target.0.id().to_string();
    let (code, v) = call(
        dir.path(),
        None,
        &["attach", "bpftrace", "--pid", &pid, "--op", "record"],
    );
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "consent-required");
    assert!(v["error"]["message"].as_str().unwrap().contains("tier 3"));
}

#[test]
fn missing_prerequisites_are_named_and_become_a_coverage_row() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path(), "1-3");
    let target = sleeper();
    let pid = target.0.id().to_string();
    // A PATH with no programs: bpftrace is certainly not found.
    let empty = tempfile::tempdir().unwrap();
    let (code, v) = call(
        dir.path(),
        Some(empty.path()),
        &["attach", "bpftrace", "--pid", &pid, "--op", "record"],
    );
    assert_eq!(code, 2, "{v}");
    let e = &v["error"];
    assert_eq!(e["code"], "tool-missing", "{v}");
    for k in ["message", "missing", "fix"] {
        assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "no {k}: {v}");
    }
    assert!(e["message"].as_str().unwrap().contains("bpftrace"));
    assert!(
        e["fix"]
            .as_str()
            .unwrap()
            .contains("apt-get install bpftrace")
    );

    let run = std::fs::read_dir(dir.path().join(".dla"))
        .unwrap()
        .flatten()
        .find(|e| e.path().join("coverage.json").is_file())
        .expect("a run with a coverage manifest");
    let cov: Value =
        serde_json::from_str(&std::fs::read_to_string(run.path().join("coverage.json")).unwrap())
            .unwrap();
    let row = &cov["workers"]["native"]["backends"]["bpftrace"];
    assert_eq!(row["status"], "missing", "{cov}");
    assert!(row["reason"].as_str().unwrap().contains("bpftrace"));
}

#[test]
fn an_unknown_operation_is_refused_with_the_choices() {
    let dir = tempfile::tempdir().unwrap();
    init(dir.path(), "1-3");
    let target = sleeper();
    let pid = target.0.id().to_string();
    let (code, v) = call(
        dir.path(),
        None,
        &["attach", "bpftrace", "--pid", &pid, "--op", "histogram"],
    );
    assert_eq!(code, 2, "{v}");
    assert!(v["error"]["message"].as_str().unwrap().contains("record"));
}
