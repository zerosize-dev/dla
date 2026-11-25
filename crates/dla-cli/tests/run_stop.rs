//! `dla run stop` stops what a cut command left and leaves the run open; `dla run abort` ends the
//! run for good, says so, and every refusal after it names `dla run init`.
#![cfg(unix)]

use std::{
    fs,
    os::unix::process::CommandExt,
    path::Path,
    process::{Child, Command as Std},
};

use assert_cmd::Command;
use dla_core::{
    RunState,
    state::{ProcessKind, ProcessRecord, ProcessRole, RunStatus, now_utc, run_dir as run_dir_of},
};
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
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn run(project: &Path, args: &[&str]) -> (i32, Value, String) {
    let out = dla(project).args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    (
        out.status.code().unwrap(),
        serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    )
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, v, text) = run(project, args);
    assert_eq!(
        (code, &v["ok"]),
        (0, &Value::Bool(true)),
        "{args:?}: {text}"
    );
    v
}

fn sleeper() -> Child {
    let mut c = Std::new("sleep");
    c.arg("60").process_group(0);
    c.spawn().unwrap()
}

fn record(pid: u32, worker: Option<&str>) -> ProcessRecord {
    ProcessRecord {
        pid,
        start_time: dla_sample::os::process_start_time(pid),
        kind: ProcessKind::Harness,
        command: "sleep 60".into(),
        started_at: now_utc(),
        stopped_at: None,
        role: ProcessRole::Started,
        worker: worker.map(str::to_owned),
        cleanup: Vec::new(),
    }
}

fn init() -> (tempfile::TempDir, String) {
    let p = tempfile::tempdir().unwrap();
    fs::write(p.path().join("a.py"), "x = 1\n").unwrap();
    let v = ok(p.path(), &["run", "init", "--session-id", "t"]);
    let id = v["run_id"].as_str().unwrap().to_owned();
    (p, id)
}

#[test]
fn stop_kills_the_registered_process_and_the_run_goes_on() {
    let (p, id) = init();
    let dir = run_dir_of(p.path(), &id);
    let mut child = sleeper();
    RunState::update(&dir, |s| {
        s.processes.push(record(child.id(), None));
        Ok(())
    })
    .unwrap();

    let v = ok(p.path(), &["--run", &id, "run", "stop"]);
    assert_eq!(v["killed"], 1, "{v}");
    assert_eq!(v["status"], "active", "{v}");
    assert!(
        v["message"].as_str().unwrap().contains("the run goes on"),
        "{v}"
    );
    // The process is gone (reaped here), the record is stopped and the run is still active.
    assert!(child.wait().is_ok_and(|s| !s.success()));
    let st = RunState::load(&dir).unwrap();
    assert_eq!(st.status, RunStatus::Active);
    assert!(
        st.processes
            .iter()
            .any(|r| r.pid == child.id() && r.stopped_at.is_some())
    );
    // A later write works: the run is not over.
    let v = ok(
        p.path(),
        &[
            "--run",
            &id,
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--site",
            "a.py:1",
        ],
    );
    assert_eq!(v["state"], "candidate", "{v}");
    assert_eq!(
        RunState::load(&dir).unwrap().status,
        RunStatus::Active,
        "a write after a stop leaves the run active"
    );
    // A second stop has nothing to do and says the same.
    let v = ok(p.path(), &["--run", &id, "run", "stop"]);
    assert_eq!(
        (v["killed"].as_u64(), v["skipped"].as_u64()),
        (Some(0), Some(1))
    );
}

#[test]
fn abort_ends_the_run_says_so_and_the_refusals_after_it_name_run_init() {
    let (p, id) = init();
    let dir = run_dir_of(p.path(), &id);
    let mut child = sleeper();
    RunState::update(&dir, |s| {
        s.processes.push(record(child.id(), None));
        Ok(())
    })
    .unwrap();

    let v = ok(
        p.path(),
        &["--run", &id, "run", "abort", "--reason", "cut command"],
    );
    assert_eq!(v["status"], "aborted", "{v}");
    let message = v["message"].as_str().unwrap();
    assert!(
        message.starts_with("this run ends here: it is marked aborted for good and nothing more can be recorded in it that counts")
            && message.contains("dla run stop"),
        "{message}"
    );
    assert!(child.wait().is_ok_and(|s| !s.success()));
    assert_eq!(RunState::load(&dir).unwrap().status, RunStatus::Aborted);
    // What a command gives after the abort: `dla run init` is the only way on.
    for args in [
        &["--run", id.as_str(), "run", "extend", "30m"][..],
        &["--run", id.as_str(), "run", "dispatch", "--workers", "go"][..],
    ] {
        let (code, v, text) = run(p.path(), args);
        assert_eq!(code, 2, "{args:?}: {text}");
        assert_eq!(v["error"]["code"], "run-not-active", "{text}");
        assert_eq!(v["error"]["fix"], "dla run init", "{text}");
    }
    // The help of both commands says what each does in its first sentence.
    let help = |name: &str| {
        let out = dla(p.path())
            .args(["run", name, "--help"])
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    };
    let abort_help = help("abort");
    let first = abort_help.lines().next().unwrap();
    assert!(
        first.starts_with("Ends the run for good")
            && first.contains("nothing more can be recorded in it")
            && first.contains("`dla run stop`"),
        "{first}"
    );
    let stop_help = help("stop");
    assert!(
        stop_help
            .lines()
            .next()
            .unwrap()
            .contains("the run stays open"),
        "{stop_help}"
    );
}

#[test]
fn a_worker_stops_its_own_processes_only() {
    let (p, id) = init();
    let dir = run_dir_of(p.path(), &id);
    ok(
        p.path(),
        &["--run", &id, "run", "dispatch", "--workers", "go,jvm"],
    );
    let mut mine = sleeper();
    let mut theirs = sleeper();
    RunState::update(&dir, |s| {
        s.processes.push(record(mine.id(), Some("go")));
        s.processes.push(record(theirs.id(), Some("jvm")));
        Ok(())
    })
    .unwrap();
    let v = ok(p.path(), &["--run", &id, "--worker", "go", "run", "stop"]);
    assert_eq!(v["killed"], 1, "{v}");
    assert!(mine.wait().is_ok_and(|s| !s.success()));
    assert!(
        theirs.try_wait().unwrap().is_none(),
        "the other worker's process was left alone"
    );
    // The core stops the rest.
    let v = ok(p.path(), &["--run", &id, "run", "stop"]);
    assert_eq!(v["killed"], 1, "{v}");
    assert!(theirs.wait().is_ok_and(|s| !s.success()));
}
