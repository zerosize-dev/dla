//! Long commands and the run's time budget: `harness run` and `alloc-test` estimate
//! their duration and refuse up front, or stop at the deadline with a partial result.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

// ---- the run's time budget: long commands stop or refuse, they do not overrun ----

use dla_core::{
    harness::{AgentKind, CommandSpec, HarnessManifest, HarnessMode},
    runtime::Runtime,
    series::KeyMode,
};

/// A project whose run has `budget` and a hand-written native command-mode harness H1
/// that runs `script` (`{n}` is the point's size).
#[cfg(unix)]
fn slow_project(budget: &str, script: &str, iterations: u32) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    let init = dla(p)
        .args(["run", "init", "--session-id", "t", "--budget", budget])
        .output()
        .unwrap();
    let init: Value = serde_json::from_slice(&init.stdout).unwrap();
    let run_dir = p.join(".dla").join(init["run_id"].as_str().unwrap());
    let c = dla(p).args(["consent", "--tiers", "1"]).output().unwrap();
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stdout));
    let work = run_dir.join("harness/H1/work");
    fs::create_dir_all(&work).unwrap();
    fs::create_dir_all(run_dir.join("harness/H1/agent")).unwrap();
    let m = HarnessManifest {
        schema_version: 1,
        id: "H1".into(),
        runtime: Runtime::Native,
        mode: HarnessMode::Command,
        workload: Some("test".into()),
        scratch: "harness/H1/work".into(),
        build: None,
        run: CommandSpec {
            argv: vec!["sh".into(), "-c".into(), script.into()],
            cwd: ".".into(),
            env: vec![],
        },
        iter_arg: Some("{n}".into()),
        warmup: 1,
        iterations,
        step: 100,
        key_modes: vec![KeyMode::Native],
        probes: vec![],
        agent: AgentKind::None,
        detectors: vec![],
        tools: vec![],
        gaps: vec![],
        timeout_secs: 120,
        ffi: None,
        captures: vec![],
    };
    m.save(&run_dir).unwrap();
    fs::write(run_dir.join("harness/H1/probes.json"), "{\"probes\":[]}").unwrap();
    (dir, run_dir)
}

#[cfg(unix)]
#[test]
fn harness_run_refuses_a_plan_the_budget_cannot_hold_and_names_a_smaller_one() {
    let (dir, run_dir) = slow_project("4s", "sleep $(awk 'BEGIN{print {n}/1000}')", 8);
    let out = dla(dir.path())
        .args(["harness", "run", "H1"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(out.status.code(), Some(2), "{v}");
    assert_eq!(v["error"]["code"], "time-budget", "{v}");
    assert!(
        v["error"]["fix"].as_str().unwrap().contains("--iterations"),
        "{v}"
    );
    assert!(
        !run_dir
            .join("series")
            .read_dir()
            .is_ok_and(|mut d| d.next().is_some())
    );
}

#[cfg(unix)]
#[test]
fn alloc_test_stops_at_the_deadline_and_says_the_result_is_partial() {
    // Fast until n = 1000 (point 10), then a point that would outlast the run.
    let (dir, _run) = slow_project(
        "7s",
        "if [ {n} -ge 1000 ]; then sleep 60; else sleep 0.05; fi",
        8,
    );
    let started = std::time::Instant::now();
    let out = dla(dir.path())
        .args(["alloc-test", "H1", "--extra", "3", "--purge", "none"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(started.elapsed().as_secs() < 20, "{:?}", started.elapsed());
    assert_eq!(out.status.code(), Some(0), "{v}");
    assert_eq!(v["favors"], "inconclusive", "{v}");
    assert!(
        v["partial"]
            .as_str()
            .is_some_and(|s| s.contains("run deadline")),
        "{v}"
    );
}
