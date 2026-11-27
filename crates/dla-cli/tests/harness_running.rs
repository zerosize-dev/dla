//! `dla harness run <H>` is refused (`harness-running`) while another `dla harness run <H>` of the
//! run is alive: two runs of one harness write the same `out/` files and series. A run of
//! another harness, and a run after the first one ended or died, are not refused.

use std::{
    path::Path,
    process::{Child, Command as Proc, Stdio},
};

use assert_cmd::Command;
use dla_core::{
    RunState,
    state::{ProcessKind, ProcessRecord, ProcessRole, now_utc},
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
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1,2");
    c
}

struct R {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

impl R {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let o = dla(dir.path())
            .args(["run", "init", "--session-id", "t"])
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
        Self { dir, run_dir }
    }

    fn harness_run(&self, h: &str) -> (i32, Value) {
        let o = dla(self.dir.path())
            .args(["harness", "run", h])
            .output()
            .unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        (
            o.status.code().unwrap(),
            serde_json::from_str(&text).unwrap_or(Value::Null),
        )
    }

    /// Registers `child` as a `dla harness run <h>` command of the run (marked stopped when
    /// `stopped`).
    fn register(&self, child: &Child, command: &str, stopped: bool) {
        let record = ProcessRecord {
            pid: child.id(),
            start_time: dla_sample::os::process_start_time(child.id()),
            kind: ProcessKind::Command,
            command: command.to_owned(),
            started_at: now_utc(),
            stopped_at: stopped.then(now_utc),
            role: ProcessRole::Attached,
            worker: None,
            cleanup: Vec::new(),
        };
        RunState::update(&self.run_dir, |s| {
            s.processes.push(record);
            Ok(())
        })
        .unwrap();
    }
}

fn sleeper() -> Child {
    Proc::new("sleep")
        .arg("120")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

#[test]
fn a_second_run_of_a_harness_is_refused_while_the_first_is_alive_and_not_after_it() {
    let r = R::new();
    let mut first = sleeper();
    r.register(&first, "dla harness run H1", false);

    let (code, v) = r.harness_run("H1");
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "harness-running", "{v}");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains(&format!("pid {}", first.id())) && msg.contains("started "),
        "the pid and the start time: {msg}"
    );
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains("wait for it to end") && fix.contains("dla run stop"),
        "{fix}"
    );

    // Another harness is not held by it: it gets past this check (and fails for what it is).
    let (_, v) = r.harness_run("H2");
    assert_ne!(v["error"]["code"], "harness-running", "{v}");

    // A command that marked itself stopped holds nothing, whatever its process does.
    let mut done = sleeper();
    r.register(&done, "dla harness run H3", true);
    let (_, v) = r.harness_run("H3");
    assert_ne!(v["error"]["code"], "harness-running", "{v}");
    done.kill().unwrap();
    done.wait().unwrap();

    // The command is dead: its record does not hold the harness any more.
    first.kill().unwrap();
    first.wait().unwrap();
    let (_, v) = r.harness_run("H1");
    assert_ne!(v["error"]["code"], "harness-running", "{v}");
}
