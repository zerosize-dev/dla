//! The hook entry points of `dla gate`, driven with the JSON Claude Code sends on stdin
//! (the shapes captured from a real session: common fields, `tool_name`, `tool_input`,
//! `hook_event_name`, and for SubagentStop `agent_id`, `agent_type`, `agent_transcript_path`).
//!
//! Every blocked action is tried: writes to the protected run files by each tool, by a
//! Windows-style path, a relative path, a `..` path and a symlink; reads of the archetype
//! files before and after the classify gate; the stop hook's block, its three-block
//! INCOMPLETE stamp and the worker form; malformed input and a panic.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::{Value, json};

struct P {
    dir: tempfile::TempDir,
    /// A directory that is not the project, used as the hook process's own cwd.
    away: tempfile::TempDir,
    run_id: String,
}

#[derive(Debug)]
struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Out {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout).unwrap_or(Value::Null)
    }
    fn denied(&self) -> bool {
        self.code == 0 && self.json()["hookSpecificOutput"]["permissionDecision"] == "deny"
    }
    fn allowed(&self) -> bool {
        self.code == 0 && self.stdout.trim().is_empty()
    }
}

fn clean(c: &mut Command) {
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_ARCHETYPES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_PROJECT_DIR",
        "DLA_HOOK_TEST_PANIC",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
    ] {
        c.env_remove(var);
    }
}

impl P {
    fn path(&self) -> &Path {
        self.dir.path()
    }
    fn run_dir(&self) -> PathBuf {
        self.path().join(".dla").join(&self.run_id)
    }
    fn dla(&self) -> Command {
        let mut c = Command::cargo_bin("dla").unwrap();
        clean(&mut c);
        c.arg("--project").arg(self.path());
        c
    }
    fn ok(&self, args: &[&str]) -> Value {
        let out = self.dla().args(args).output().unwrap();
        let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        assert_eq!(out.status.code(), Some(0), "{args:?}: {v}");
        v
    }
    fn state(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir().join("state.json")).unwrap())
            .unwrap()
    }
    /// Runs `dla gate <sub...>` with `stdin`, as a hook process: no `--project`, the
    /// process cwd is elsewhere, so the run is found from the JSON's `cwd`.
    fn hook(&self, sub: &[&str], stdin: &str) -> Out {
        self.hook_env(sub, stdin, &[])
    }
    fn hook_env(&self, sub: &[&str], stdin: &str, env: &[(&str, &str)]) -> Out {
        let mut c = Command::cargo_bin("dla").unwrap();
        clean(&mut c);
        c.current_dir(self.away.path());
        for (k, v) in env {
            c.env(k, v);
        }
        let o = c.arg("gate").args(sub).write_stdin(stdin).output().unwrap();
        Out {
            code: o.status.code().unwrap(),
            stdout: String::from_utf8(o.stdout).unwrap(),
            stderr: String::from_utf8(o.stderr).unwrap(),
        }
    }
}

/// A project with an active run. `entry` is the run's entry path; consent is given by the
/// runner so the intake gate can pass.
fn project(entry: &str) -> P {
    let dir = tempfile::tempdir().unwrap();
    let away = tempfile::tempdir().unwrap();
    let mut c = Command::cargo_bin("dla").unwrap();
    clean(&mut c);
    let out = c
        .env("DLA_CONSENT_TIERS", "1,2")
        .arg("--project")
        .arg(dir.path())
        .args(["run", "init", "--session-id", "t", "--entry", entry])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let run_id = v["run_id"].as_str().unwrap().to_owned();
    P { dir, away, run_id }
}

const INTAKE: [&str; 8] = [
    "intake.target",
    "intake.scope",
    "intake.symptom",
    "intake.runtimes",
    "intake.build",
    "intake.workload",
    "intake.consent",
    "intake.budget",
];

fn answer_intake(p: &P) {
    for item in INTAKE {
        p.ok(&[
            "ledger",
            "resolve",
            item,
            "--status",
            "answered",
            "--evidence",
            "E1",
        ]);
    }
}

fn pre(tool: &str, input: Value, cwd: &Path) -> String {
    json!({
        "session_id": "0f7c2c1e-0000-4000-8000-000000000001",
        "transcript_path": "/tmp/t.jsonl",
        "cwd": cwd,
        "permission_mode": "default",
        "effort": {"level": "high"},
        "hook_event_name": "PreToolUse",
        "tool_name": tool,
        "tool_input": input,
        "tool_use_id": "toolu_01",
    })
    .to_string()
}

fn write_json(tool: &str, path: &str, cwd: &Path) -> String {
    let key = if tool == "NotebookEdit" {
        "notebook_path"
    } else {
        "file_path"
    };
    pre(tool, json!({ key: path, "content": "x" }), cwd)
}

fn bash_json(cmd: &str, cwd: &Path) -> String {
    pre("Bash", json!({"command": cmd, "description": "d"}), cwd)
}

fn stop_json(cwd: &Path) -> String {
    json!({
        "session_id": "0f7c2c1e-0000-4000-8000-000000000001",
        "transcript_path": "/tmp/t.jsonl",
        "cwd": cwd,
        "permission_mode": "default",
        "hook_event_name": "Stop",
        "stop_hook_active": false,
        "last_assistant_message": "done",
        "background_tasks": [],
        "session_crons": [],
    })
    .to_string()
}

fn protected_files(p: &P) -> Vec<PathBuf> {
    let r = p.run_dir();
    vec![
        r.join("report.md"),
        r.join("ledger.jsonl"),
        r.join("ledger.jvm.jsonl"),
        r.join("state.json"),
        r.join("coverage.json"),
        r.join("integrity").join("ledger.jsonl.seal"),
    ]
}

// ---------------------------------------------------------------- write hook

#[test]
fn write_hook_denies_every_protected_file_for_every_file_tool() {
    let p = project("symptom");
    for tool in ["Write", "Edit", "NotebookEdit"] {
        for f in protected_files(&p) {
            let out = p.hook(
                &["write-hook"],
                &write_json(tool, &f.to_string_lossy(), p.path()),
            );
            assert!(out.denied(), "{tool} {}: {out:?}", f.display());
            let reason = out.json()["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .to_owned();
            assert!(reason.contains("dla"), "{reason}");
            assert_eq!(
                out.json()["hookSpecificOutput"]["hookEventName"],
                "PreToolUse"
            );
        }
    }
    // The files were never touched: the state still loads.
    assert_eq!(p.state()["status"], "active");
}

#[test]
fn write_hook_allows_other_paths() {
    let p = project("symptom");
    let r = p.run_dir();
    for f in [
        p.path().join("src").join("a.rs"),
        p.path().join("report.md"),
        p.path().join("notes").join("ledger.md"),
        r.join("series").join("S1.json"),
        r.join("gate").join("final.json"),
    ] {
        let out = p.hook(
            &["write-hook"],
            &write_json("Write", &f.to_string_lossy(), p.path()),
        );
        assert!(out.allowed(), "{}: {out:?}", f.display());
    }
    // A tool the hook does not know is not its business.
    let out = p.hook(
        &["write-hook"],
        &pre("WebFetch", json!({"url": "https://x"}), p.path()),
    );
    assert!(out.allowed(), "{out:?}");
}

#[test]
fn write_hook_normalizes_windows_relative_and_dotdot_paths() {
    let p = project("symptom");
    let run = p.run_id.clone();
    let state = p.run_dir().join("state.json");
    // Backslash spelling of the same absolute path.
    let win = state.to_string_lossy().replace('/', "\\");
    let out = p.hook(&["write-hook"], &write_json("Write", &win, p.path()));
    assert!(out.denied(), "{win}: {out:?}");
    // Relative to the cwd.
    let rel = format!(".dla/{run}/report.md");
    let out = p.hook(&["write-hook"], &write_json("Edit", &rel, p.path()));
    assert!(out.denied(), "{rel}: {out:?}");
    // `..` segments, forward and backward, through a directory that does not exist.
    let dd = format!("src/../.dla/{run}/state.json");
    let out = p.hook(&["write-hook"], &write_json("Write", &dd, p.path()));
    assert!(out.denied(), "{dd}: {out:?}");
    let dd = format!("src\\..\\.dla\\{run}\\ledger.jsonl");
    let out = p.hook(&["write-hook"], &write_json("Write", &dd, p.path()));
    assert!(out.denied(), "{dd}: {out:?}");
    // A relative path that stays in the source tree.
    let out = p.hook(
        &["write-hook"],
        &write_json("Write", "src/../lib/a.rs", p.path()),
    );
    assert!(out.allowed(), "{out:?}");
    // A drive path of some other Windows project is not this project's.
    let out = p.hook(
        &["write-hook"],
        &write_json("Write", "C:\\elsewhere\\.dla\\r\\state.json", p.path()),
    );
    assert!(out.allowed(), "{out:?}");
}

#[cfg(unix)]
#[test]
fn write_hook_follows_symlinks() {
    let p = project("symptom");
    // A link to the directory, and a link to the file.
    let dir_link = p.path().join("shortcut");
    std::os::unix::fs::symlink(p.run_dir(), &dir_link).unwrap();
    let out = p.hook(
        &["write-hook"],
        &write_json(
            "Write",
            &dir_link.join("report.md").to_string_lossy(),
            p.path(),
        ),
    );
    assert!(out.denied(), "{out:?}");
    let file_link = p.path().join("innocent.txt");
    std::os::unix::fs::symlink(p.run_dir().join("state.json"), &file_link).unwrap();
    let out = p.hook(
        &["write-hook"],
        &write_json("Edit", &file_link.to_string_lossy(), p.path()),
    );
    assert!(out.denied(), "{out:?}");
}

#[test]
fn write_hook_leaves_another_projects_dla_alone() {
    let p = project("symptom");
    let other = tempfile::tempdir().unwrap();
    let f = other.path().join(".dla").join("r1").join("report.md");
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    let out = p.hook(
        &["write-hook"],
        &write_json("Write", &f.to_string_lossy(), p.path()),
    );
    assert!(out.allowed(), "{out:?}");
    // And with that project as the cwd there is no run there at all.
    let out = p.hook(
        &["write-hook"],
        &write_json("Write", &f.to_string_lossy(), other.path()),
    );
    assert!(out.allowed(), "{out:?}");
}

#[test]
fn write_hook_allows_everything_without_an_active_run() {
    let p = project("symptom");
    let report = p.run_dir().join("report.md");
    let input = write_json("Write", &report.to_string_lossy(), p.path());
    assert!(p.hook(&["write-hook"], &input).denied());
    // A finished run is not governed.
    p.ok(&["run", "abort"]);
    assert_eq!(p.state()["status"], "aborted");
    assert!(p.hook(&["write-hook"], &input).allowed());
    // Neither is a directory with no run.
    let empty = tempfile::tempdir().unwrap();
    let out = p.hook(
        &["write-hook"],
        &write_json("Write", "/x/.dla/r/report.md", empty.path()),
    );
    assert!(out.allowed(), "{out:?}");
    fs::remove_file(p.path().join(".dla").join("current")).unwrap();
    assert!(p.hook(&["write-hook"], &input).allowed());
}

#[test]
fn bash_write_forms_are_denied_and_dla_commands_are_not() {
    let p = project("symptom");
    let id = &p.run_id;
    let denied = [
        format!("echo x > .dla/{id}/ledger.jsonl"),
        format!("echo x >> .dla/{id}/report.md"),
        format!("cat a | tee .dla/{id}/state.json"),
        format!("sed -i 's/a/b/' .dla/{id}/coverage.json"),
        format!("cp /tmp/x .dla/{id}/ledger.jsonl"),
        format!("mv /tmp/x .dla/{id}/state.json"),
        format!("rm .dla/{id}/report.md"),
        format!("truncate -s 0 .dla/{id}/ledger.jsonl"),
        format!("dd if=/dev/zero of=.dla/{id}/state.json"),
        format!("python3 -c \"open('.dla/{id}/state.json','w').write('x')\""),
        format!("node -e \"require('fs').writeFileSync('.dla/{id}/ledger.jsonl','x')\""),
        format!("perl -e 'unlink q(.dla/{id}/report.md)'"),
        format!("ls && echo x > .dla/{id}/state.json"),
        format!("dla run status > .dla/{id}/state.json"),
    ];
    for c in &denied {
        for hook in ["write-hook", "bash-hook"] {
            let out = p.hook(&[hook], &bash_json(c, p.path()));
            assert!(out.denied(), "{hook}: {c}: {out:?}");
        }
    }
    let allowed = [
        "dla run status".to_owned(),
        "dla ledger add --origin census --evidence E1 2>&1".to_owned(),
        "\"/p/bin/dla\" report".to_owned(),
        format!("cat .dla/{id}/report.md"),
        format!("grep -n F1 .dla/{id}/ledger.jsonl"),
        format!("ls -la .dla/{id}"),
        "git status && cargo build 2>&1 | tail -5".to_owned(),
        "echo '.dla/' >> .gitignore".to_owned(),
    ];
    for c in &allowed {
        for hook in ["write-hook", "bash-hook"] {
            let out = p.hook(&[hook], &bash_json(c, p.path()));
            assert!(out.allowed(), "{hook}: {c}: {out:?}");
        }
    }
}

// ----------------------------------------------------------------- read hook

fn read_json(path: &str, cwd: &Path) -> String {
    pre("Read", json!({ "file_path": path }), cwd)
}

#[test]
fn read_hook_locks_the_archetype_files_until_the_classify_gate_passes() {
    let p = project("symptom");
    let refs = p.path().join("skills/detect-leak-anywhere/references");
    let locked = [
        refs.join("go-archetypes.md"),
        refs.join("known-leaks.md"),
        p.path().join("data/archetypes.toml"),
        p.path().join("data/known-leaks.toml"),
    ];
    for f in &locked {
        let out = p.hook(&["read-hook"], &read_json(&f.to_string_lossy(), p.path()));
        assert!(out.denied(), "{}: {out:?}", f.display());
        let reason = out.json()["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(
            reason.contains("dla gate check --stage classify"),
            "{reason}"
        );
    }
    // Windows spelling, Grep, Glob and Bash reach the same files.
    let win = locked[0].to_string_lossy().replace('/', "\\");
    assert!(p.hook(&["read-hook"], &read_json(&win, p.path())).denied());
    for (tool, input) in [
        (
            "Grep",
            json!({"pattern": "L-", "path": refs.join("go-archetypes.md")}),
        ),
        ("Grep", json!({"pattern": "L-", "glob": "*-archetypes.md"})),
        ("Glob", json!({"pattern": "**/*-archetypes.md"})),
    ] {
        let out = p.hook(&["read-hook"], &pre(tool, input.clone(), p.path()));
        assert!(out.denied(), "{tool} {input}: {out:?}");
    }
    for c in [
        "cat skills/detect-leak-anywhere/references/go-archetypes.md",
        "sed -n 1,20p data/known-leaks.toml",
        "rg L-WORKER data/archetypes.toml",
    ] {
        for hook in ["read-hook", "bash-hook"] {
            let out = p.hook(&[hook], &bash_json(c, p.path()));
            assert!(out.denied(), "{hook}: {c}: {out:?}");
        }
    }
    // Other files are free.
    for f in [
        refs.join("stages.md"),
        p.path().join("README.md"),
        p.path().join("src/archetypes.rs"),
    ] {
        let out = p.hook(&["read-hook"], &read_json(&f.to_string_lossy(), p.path()));
        assert!(out.allowed(), "{}: {out:?}", f.display());
    }
    assert!(
        p.hook(&["read-hook"], &bash_json("cat README.md", p.path()))
            .allowed()
    );

    // The gate: classify passes once intake is answered (a symptom run has no probe or
    // sweep stage), and the files open.
    let failing = p
        .dla()
        .args(["gate", "check", "--stage", "classify"])
        .output()
        .unwrap();
    assert_eq!(failing.status.code(), Some(2));
    answer_intake(&p);
    let v = p.ok(&["gate", "check", "--stage", "classify"]);
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["recorded"], true);
    assert!(
        p.state()["stages_passed"]
            .as_array()
            .unwrap()
            .contains(&json!("classify"))
    );
    for f in &locked {
        let out = p.hook(&["read-hook"], &read_json(&f.to_string_lossy(), p.path()));
        assert!(out.allowed(), "{}: {out:?}", f.display());
    }
    assert!(
        p.hook(
            &["bash-hook"],
            &bash_json("cat skills/x/references/go-archetypes.md", p.path())
        )
        .allowed()
    );
}

#[test]
fn read_hook_allows_everything_without_a_run() {
    let empty = tempfile::tempdir().unwrap();
    let p = project("symptom");
    let out = p.hook(
        &["read-hook"],
        &read_json("/x/references/go-archetypes.md", empty.path()),
    );
    assert!(out.allowed(), "{out:?}");
    p.ok(&["run", "abort"]);
    let out = p.hook(
        &["read-hook"],
        &read_json("/x/references/go-archetypes.md", p.path()),
    );
    assert!(out.allowed(), "{out:?}");
}

/// A PreToolUse call made inside a subagent: `agent_id` and the main session's transcript.
fn pre_in_subagent(path: &str, cwd: &Path, main_transcript: &Path, agent_id: &str) -> String {
    let mut v: Value = serde_json::from_str(&read_json(path, cwd)).unwrap();
    v["transcript_path"] = json!(main_transcript);
    v["agent_id"] = json!(agent_id);
    v["agent_type"] = json!("dla:worker");
    v.to_string()
}

/// Writes `<main dir>/<stem>/subagents/agent-<id>.jsonl` with one Bash call per command.
fn subagent_transcript_file(main: &Path, id: &str, commands: &[&str]) {
    let dir = main.with_extension("").join("subagents");
    fs::create_dir_all(&dir).unwrap();
    let t = transcript(&dir, commands);
    fs::rename(t, dir.join(format!("agent-{id}.jsonl"))).unwrap();
}

#[test]
fn read_hook_opens_the_tables_to_a_subagent_whose_worker_passed_classify() {
    let p = project("repository");
    p.ok(&["run", "dispatch", "--workers", "jvm,go"]);
    let t = tempfile::tempdir().unwrap();
    let main = t.path().join("session.jsonl");
    let table = p
        .path()
        .join("skills/detect-leak-anywhere/references/jvm-archetypes.md");
    let table = table.to_string_lossy().into_owned();
    subagent_transcript_file(&main, "aaa", &["dla --worker jvm run status", "ls"]);
    subagent_transcript_file(&main, "bbb", &["dla --worker go run status"]);
    subagent_transcript_file(&main, "ccc", &["ls", "echo --worker jvm"]);
    let call = |id: &str| {
        p.hook(
            &["read-hook"],
            &pre_in_subagent(&table, p.path(), &main, id),
        )
    };
    // Nothing has passed classify yet.
    assert!(call("aaa").denied());

    // The jvm worker's own check passes: it is recorded for the worker, not for the run.
    let v = p.ok(&["--worker", "jvm", "gate", "check", "--stage", "classify"]);
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["recorded"], true);
    let st = p.state();
    assert_eq!(st["worker_stages_passed"]["jvm"], json!(["classify"]));
    assert!(st.get("stages_passed").is_none(), "{st}");

    // Its subagent may read; another worker's subagent, a subagent whose worker cannot be
    // named, the core session, and an unknown agent file still may not.
    assert!(call("aaa").allowed());
    assert!(call("bbb").denied());
    assert!(call("ccc").denied());
    assert!(call("zzz").denied());
    assert!(
        p.hook(&["read-hook"], &read_json(&table, p.path()))
            .denied()
    );
    // SubagentStop-style field: the transcript named directly.
    let direct = main.with_extension("").join("subagents/agent-aaa.jsonl");
    let mut v: Value = serde_json::from_str(&read_json(&table, p.path())).unwrap();
    v["agent_id"] = json!("aaa");
    v["agent_transcript_path"] = json!(direct);
    assert!(p.hook(&["read-hook"], &v.to_string()).allowed());
    // The bash hook applies the same rule.
    let mut b: Value = serde_json::from_str(&bash_json(&format!("cat {table}"), p.path())).unwrap();
    b["transcript_path"] = json!(main);
    b["agent_id"] = json!("aaa");
    assert!(p.hook(&["bash-hook"], &b.to_string()).allowed());
    b["agent_id"] = json!("bbb");
    assert!(p.hook(&["bash-hook"], &b.to_string()).denied());
}

// ------------------------------------------------------------------ stop hook

#[test]
fn stop_hook_allows_without_a_run() {
    let p = project("symptom");
    let empty = tempfile::tempdir().unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(empty.path()));
    assert!(out.allowed(), "{out:?}");
    assert!(out.stderr.is_empty(), "{out:?}");
}

#[test]
fn stop_hook_blocks_while_the_final_gate_fails_then_allows_when_it_passes() {
    let p = project("symptom");
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert_eq!(out.code, 0, "{out:?}");
    let v = out.json();
    assert_eq!(v["decision"], "block", "{out:?}");
    let reason = v["reason"].as_str().unwrap();
    assert!(reason.len() <= 1000, "{} bytes", reason.len());
    assert!(reason.contains("intake.target"), "{reason}");
    assert!(reason.contains("dla ledger resolve"), "{reason}");
    assert!(reason.contains("dla gate check --final"), "{reason}");
    assert_eq!(p.state()["stop_blocks"], 1);
    // Fixing it lets the session stop, and the counter does not stamp the run.
    answer_intake(&p);
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert!(out.allowed(), "{out:?}");
    assert_eq!(p.state()["status"], "active");
}

#[test]
fn three_blocks_with_no_ledger_change_stamp_the_run_incomplete() {
    let p = project("symptom");
    let first = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert_eq!(first.json()["decision"], "block");
    let second = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert_eq!(second.json()["decision"], "block");
    assert_eq!(p.state()["stop_blocks"], 2);
    // The third attempt with nothing changed lets the model stop and stamps the run.
    let third = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert!(third.allowed(), "{third:?}");
    assert!(third.stderr.contains("INCOMPLETE"), "{third:?}");
    let st = p.state();
    assert_eq!(st["status"], "incomplete");
    let why = st["incomplete_reason"].as_str().unwrap();
    assert!(
        why.starts_with("stop hook blocked 3 times with no ledger change: "),
        "{why}"
    );
    assert!(why.contains("intake.target"), "{why}");
    // The report prints INCOMPLETE with that reason and its own fix text.
    let out = p.dla().arg("report").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.contains("Status: INCOMPLETE"), "{md}");
    assert!(md.contains("stop hook blocked 3 times"), "{md}");
    assert!(!md.contains("Fix: `dla run init`"), "{md}");
    assert!(md.contains("the stop hook stamped this run"), "{md}");
    // A stamped run is not governed any more.
    assert!(p.hook(&["stop-hook"], &stop_json(p.path())).allowed());
}

#[test]
fn a_ledger_change_resets_the_block_counter() {
    let p = project("symptom");
    for _ in 0..2 {
        assert_eq!(
            p.hook(&["stop-hook"], &stop_json(p.path())).json()["decision"],
            "block"
        );
    }
    assert_eq!(p.state()["stop_blocks"], 2);
    // Progress: one item answered.
    p.ok(&[
        "ledger",
        "resolve",
        "intake.target",
        "--status",
        "answered",
        "--evidence",
        "E1",
    ]);
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert_eq!(out.json()["decision"], "block", "{out:?}");
    assert_eq!(p.state()["stop_blocks"], 1);
    assert_eq!(p.state()["status"], "active");
}

#[test]
fn stop_hook_active_does_not_change_the_decision() {
    let p = project("symptom");
    let mut v: Value = serde_json::from_str(&stop_json(p.path())).unwrap();
    v["stop_hook_active"] = json!(true);
    let out = p.hook(&["stop-hook"], &v.to_string());
    assert_eq!(out.json()["decision"], "block", "{out:?}");
}

// --------------------------------------------------------------- subagent stop

fn subagent_json(cwd: &Path, transcript: &Path) -> String {
    json!({
        "session_id": "0f7c2c1e-0000-4000-8000-000000000001",
        "transcript_path": "/tmp/t.jsonl",
        "cwd": cwd,
        "permission_mode": "default",
        "hook_event_name": "SubagentStop",
        "stop_hook_active": false,
        "agent_id": "agent-def456",
        "agent_type": "dla:worker",
        "agent_transcript_path": transcript,
        "last_assistant_message": "done",
    })
    .to_string()
}

fn transcript(dir: &Path, commands: &[&str]) -> PathBuf {
    let path = dir.join("agent-def456.jsonl");
    let mut text = String::new();
    for (i, c) in commands.iter().enumerate() {
        text.push_str(
            &json!({
                "type": "assistant",
                "message": {"role": "assistant", "content": [
                    {"type": "text", "text": "running"},
                    {"type": "tool_use", "id": format!("toolu_{i}"), "name": "Bash",
                     "input": {"command": c}}
                ]}
            })
            .to_string(),
        );
        text.push('\n');
    }
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn subagent_stop_gates_the_worker_named_in_its_transcript() {
    let p = project("repository");
    p.ok(&["run", "dispatch", "--workers", "jvm,go"]);
    let t = tempfile::tempdir().unwrap();
    // No dla command with --worker in the transcript: not a worker we can name, allow.
    let none = transcript(t.path(), &["ls -la", "echo --worker jvm"]);
    assert!(
        p.hook(
            &["stop-hook", "--subagent"],
            &subagent_json(p.path(), &none)
        )
        .allowed()
    );
    // The last worker named wins.
    let tr = transcript(
        t.path(),
        &[
            "dla --worker go run status",
            "\"${CLAUDE_PLUGIN_ROOT}/bin/dla\" --worker jvm ledger add --origin census --evidence E1",
        ],
    );
    let out = p.hook(&["stop-hook", "--subagent"], &subagent_json(p.path(), &tr));
    assert_eq!(out.json()["decision"], "block", "{out:?}");
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(reason.contains("dla --worker jvm ledger done"), "{reason}");
    assert!(reason.len() <= 1000);
    assert_eq!(p.state()["stop_blocks"].as_u64().unwrap_or(0), 0);
    assert_eq!(p.state()["worker_stop"]["jvm"]["blocks"], 1);
    // Unchanged twice more: the worker is marked stalled and may stop.
    let again = p.hook(&["stop-hook", "--subagent"], &subagent_json(p.path(), &tr));
    assert_eq!(again.json()["decision"], "block");
    let third = p.hook(&["stop-hook", "--subagent"], &subagent_json(p.path(), &tr));
    assert!(third.allowed(), "{third:?}");
    assert!(third.stderr.contains("stalled"), "{third:?}");
    let st = p.state();
    let workers = st["workers"].as_array().unwrap();
    let jvm = workers.iter().find(|w| w["name"] == "jvm").unwrap();
    let go = workers.iter().find(|w| w["name"] == "go").unwrap();
    assert_eq!(jvm["stalled"], true);
    assert!(go.get("stalled").is_none_or(|s| s == false));
    // The run itself is untouched.
    assert_eq!(st["status"], "active");
    assert_eq!(st["stop_blocks"].as_u64().unwrap_or(0), 0);
}

#[test]
fn subagent_stop_allows_a_worker_that_finished_its_slice() {
    let p = project("repository");
    p.ok(&["run", "dispatch", "--workers", "jvm"]);
    let done = p
        .dla()
        .args(["--worker", "jvm", "ledger", "done"])
        .output()
        .unwrap();
    assert_eq!(done.status.code(), Some(0), "{done:?}");
    let t = tempfile::tempdir().unwrap();
    let tr = transcript(t.path(), &["dla --worker jvm ledger done"]);
    let out = p.hook(&["stop-hook", "--subagent"], &subagent_json(p.path(), &tr));
    assert!(out.allowed(), "{out:?}");
}

#[test]
fn subagent_stop_with_a_missing_transcript_allows() {
    let p = project("repository");
    p.ok(&["run", "dispatch", "--workers", "jvm"]);
    let out = p.hook(
        &["stop-hook", "--subagent"],
        &subagent_json(p.path(), Path::new("/nonexistent/agent.jsonl")),
    );
    assert!(out.allowed(), "{out:?}");
}

// -------------------------------------------------------------- failure modes

#[test]
fn pre_tool_use_gates_fail_closed_on_malformed_input() {
    let p = project("symptom");
    for sub in ["write-hook", "read-hook", "bash-hook"] {
        for bad in ["", "not json", "{\"tool_name\":", "[1,2]"] {
            let out = p.hook(&[sub], bad);
            assert_eq!(out.code, 2, "{sub} {bad:?}: {out:?}");
            assert!(out.stdout.is_empty());
            assert!(out.stderr.contains("dla gate"), "{out:?}");
        }
    }
}

#[test]
fn stop_hooks_fail_open_on_malformed_input() {
    let p = project("symptom");
    for args in [&["stop-hook"][..], &["stop-hook", "--subagent"][..]] {
        for bad in ["", "not json", "{\"cwd\":"] {
            let out = p.hook(args, bad);
            assert_eq!(out.code, 0, "{args:?} {bad:?}: {out:?}");
            assert!(out.stdout.is_empty());
            assert!(out.stderr.contains("dla gate"), "{out:?}");
        }
    }
}

#[test]
fn a_panic_blocks_a_gate_and_lets_a_stop_through() {
    let p = project("symptom");
    let env = [("DLA_HOOK_TEST_PANIC", "1")];
    for sub in ["write-hook", "read-hook", "bash-hook"] {
        let out = p.hook_env(&[sub], &write_json("Write", "/x/a.rs", p.path()), &env);
        assert_eq!(out.code, 2, "{sub}: {out:?}");
        assert!(out.stderr.contains("internal error"), "{out:?}");
    }
    for args in [&["stop-hook"][..], &["stop-hook", "--subagent"][..]] {
        let out = p.hook_env(args, &stop_json(p.path()), &env);
        assert_eq!(out.code, 0, "{args:?}: {out:?}");
        assert!(out.stderr.contains("internal error"), "{out:?}");
    }
}

#[test]
fn unknown_fields_and_a_missing_path_are_not_errors() {
    let p = project("symptom");
    let mut v: Value = serde_json::from_str(&write_json("Write", "/x/a.rs", p.path())).unwrap();
    v["something_new"] = json!({"a": [1, 2, 3]});
    assert!(p.hook(&["write-hook"], &v.to_string()).allowed());
    let out = p.hook(&["write-hook"], &pre("Write", json!({}), p.path()));
    assert!(out.allowed(), "{out:?}");
}

// ------------------------------------------------------------ drift: the wiring

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

const GATE_SUBCOMMANDS: [&str; 5] = ["check", "stop-hook", "read-hook", "write-hook", "bash-hook"];

/// The subcommand a hook command line names: `... gate <sub> [flags]`.
fn gate_sub(command: &str) -> Option<String> {
    let rest = command.split_once(" gate ")?.1;
    rest.split_whitespace().next().map(str::to_owned)
}

/// One handler of the SKILL.md frontmatter.
#[derive(Debug)]
struct SkillHook {
    event: String,
    matcher: String,
    command: String,
    timeout: String,
    on_failure: Option<String>,
}

/// The handlers under `hooks:` in the SKILL.md frontmatter (a small reader for the block
/// the file uses: there is no YAML crate in the tree).
fn skill_hooks() -> Vec<SkillHook> {
    let text =
        fs::read_to_string(repo_root().join("skills/detect-leak-anywhere/SKILL.md")).unwrap();
    let front = text
        .strip_prefix("---\n")
        .unwrap()
        .split("\n---")
        .next()
        .unwrap();
    let value = |l: &str| -> String {
        l.split_once(':')
            .unwrap()
            .1
            .trim()
            .trim_matches('"')
            .replace("\\\"", "\"")
    };
    let (mut event, mut matcher) = (String::new(), String::new());
    let mut out: Vec<SkillHook> = Vec::new();
    let mut in_hooks = false;
    for line in front.lines() {
        if line.starts_with("hooks:") {
            in_hooks = true;
            continue;
        }
        if !in_hooks {
            continue;
        }
        let t = line.trim_start();
        let indent = line.len() - t.len();
        if indent == 2 && t.ends_with(':') {
            event = t.trim_end_matches(':').to_owned();
            matcher.clear();
        } else if t.starts_with("- matcher:") {
            matcher = value(&t["- ".len()..]);
        } else if t.starts_with("command:") {
            out.push(SkillHook {
                event: event.clone(),
                matcher: matcher.clone(),
                command: value(t),
                timeout: String::new(),
                on_failure: None,
            });
        } else if t.starts_with("timeout:") {
            out.last_mut().unwrap().timeout = value(t);
        } else if t.starts_with("onFailure:") {
            out.last_mut().unwrap().on_failure = Some(value(t));
        }
    }
    out
}

#[test]
fn skill_frontmatter_hooks_name_real_gate_subcommands() {
    let hooks = skill_hooks();
    assert!(hooks.len() >= 4, "{hooks:?}");
    let mut seen = Vec::new();
    for h in &hooks {
        assert!(
            h.command
                .starts_with("\"${CLAUDE_PLUGIN_ROOT}/bin/dla\" gate "),
            "{}",
            h.command
        );
        let sub = gate_sub(&h.command).unwrap();
        assert!(GATE_SUBCOMMANDS.contains(&sub.as_str()), "{}", h.command);
        assert_eq!(h.timeout, "10", "{}", h.command);
        if h.event == "PreToolUse" {
            assert_eq!(h.on_failure.as_deref(), Some("block"), "{}", h.command);
        }
        seen.push((h.event.clone(), h.matcher.clone(), sub));
    }
    let has = |event: &str, tool: &str, sub: &str| {
        seen.iter().any(|(e, m, s)| {
            e == event && s == sub && (tool.is_empty() || m.split('|').any(|t| t == tool))
        })
    };
    for tool in ["Write", "Edit", "NotebookEdit"] {
        assert!(has("PreToolUse", tool, "write-hook"), "{tool}");
    }
    for tool in ["Read", "Grep", "Glob"] {
        assert!(has("PreToolUse", tool, "read-hook"), "{tool}");
    }
    assert!(has("PreToolUse", "Bash", "bash-hook"));
    assert!(has("Stop", "", "stop-hook"));
}

#[test]
fn hooks_json_names_real_gate_subcommands_and_covers_the_worker() {
    let text = fs::read_to_string(repo_root().join("hooks/hooks.json")).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    let hooks = &v["hooks"];
    // SessionStart stays.
    assert!(hooks["SessionStart"].is_array());
    // Every command that mentions `gate` names a real subcommand; PreToolUse handlers
    // block on failure and carry an `if` filter, so unrelated calls spawn nothing.
    let mut tools = Vec::new();
    let mut ifs = Vec::new();
    for (event, groups) in hooks.as_object().unwrap() {
        for g in groups.as_array().unwrap() {
            for h in g["hooks"].as_array().unwrap() {
                let command = h["command"].as_str().unwrap();
                if !command.contains(" gate ") {
                    continue;
                }
                let sub = gate_sub(command).unwrap();
                assert!(GATE_SUBCOMMANDS.contains(&sub.as_str()), "{command}");
                assert!(command.starts_with("\"${CLAUDE_PLUGIN_ROOT}/bin/dla\" gate "));
                assert_eq!(h["timeout"], 10);
                if event == "PreToolUse" {
                    assert_eq!(h["onFailure"], "block", "{h}");
                    let f = h["if"].as_str().expect("an if filter");
                    ifs.push(f.to_owned());
                    tools.push(g["matcher"].as_str().unwrap().to_owned());
                    assert_ne!(sub, "bash-hook", "no Bash hook in hooks.json");
                }
            }
        }
    }
    for t in ["Write", "Edit", "NotebookEdit", "Read"] {
        assert!(tools.iter().any(|x| x == t), "{t} in {tools:?}");
    }
    for f in [
        "Write(**/.dla/**)",
        "Edit(**/.dla/**)",
        "NotebookEdit(**/.dla/**)",
        "Read(**/*-archetypes.md)",
        "Read(**/known-leaks.md)",
        "Read(**/archetypes.toml)",
        "Read(**/known-leaks.toml)",
    ] {
        assert!(ifs.iter().any(|x| x == f), "{f} in {ifs:?}");
    }
    let stop = hooks["SubagentStop"].as_array().unwrap();
    assert_eq!(stop.len(), 1);
    assert_eq!(stop[0]["matcher"], "^dla:worker$");
    let cmd = stop[0]["hooks"][0]["command"].as_str().unwrap();
    assert_eq!(gate_sub(cmd).as_deref(), Some("stop-hook"));
    assert!(cmd.ends_with("--subagent"), "{cmd}");
    // The subcommands exist in the binary.
    let help = Command::cargo_bin("dla")
        .unwrap()
        .args(["gate", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8(help.stdout).unwrap();
    for sub in GATE_SUBCOMMANDS {
        assert!(help.contains(sub), "{sub} in {help}");
    }
}
