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
    ] {
        let out = p.hook(
            &["write-hook"],
            &write_json("Write", &f.to_string_lossy(), p.path()),
        );
        assert!(out.allowed(), "{}: {out:?}", f.display());
    }
    // The harness driver is the file the skill has the model edit.
    for f in [
        r.join("harness")
            .join("H1")
            .join("driver")
            .join("DlaHarness.java"),
        r.join("harness")
            .join("H12")
            .join("driver")
            .join("dla_harness.py"),
        r.join("harness")
            .join("H2")
            .join("driver")
            .join("sub")
            .join("x.go"),
    ] {
        for tool in ["Write", "Edit"] {
            let out = p.hook(
                &["write-hook"],
                &write_json(tool, &f.to_string_lossy(), p.path()),
            );
            assert!(out.allowed(), "{tool} {}: {out:?}", f.display());
        }
    }
    // A tool the hook does not know is not its business.
    let out = p.hook(
        &["write-hook"],
        &pre("WebFetch", json!({"url": "https://x"}), p.path()),
    );
    assert!(out.allowed(), "{out:?}");
}

/// Files `dla` generates and the gates read. No seal covers them, so the hook does.
fn generated_files(p: &P) -> Vec<PathBuf> {
    let r = p.run_dir();
    vec![
        r.join("series").join("S1.json"),
        r.join("trend").join("T1.json"),
        r.join("gate").join("final.json"),
        r.join("integrity").join("state.json.seal"),
        r.join("bundle").join("F1").join("index.json"),
        r.join("intervene").join("I1.json"),
        r.join("alloc").join("A1.json"),
        r.join("raw").join("growth.json"),
        r.join("harness").join("H1").join("harness.json"),
        r.join("harness").join("H1").join("out").join("samples.csv"),
        r.join("harness")
            .join("H1")
            .join("work")
            .join("src")
            .join("a.py"),
        r.join("harness").join("H1").join("agent").join("agent.jar"),
        r.join("harness").join("H1").join("drivers").join("x.py"),
        r.join("harness").join("H1").join("driver"),
        r.join("harness").join("Hx").join("driver").join("x.py"),
        r.join("harness")
            .join("H1")
            .join("driver")
            .join("..")
            .join("harness.json"),
        r.join("probe").join("probe.json"),
        r.join("ledger.lock"),
        p.path().join(".dla").join("current"),
    ]
}

#[test]
fn write_hook_denies_the_generated_files_the_gates_read_but_not_the_driver() {
    let p = project("symptom");
    for tool in ["Write", "Edit", "NotebookEdit"] {
        for f in generated_files(&p) {
            let out = p.hook(
                &["write-hook"],
                &write_json(tool, &f.to_string_lossy(), p.path()),
            );
            assert!(out.denied(), "{tool} {}: {out:?}", f.display());
        }
    }
    // The driver, by a relative path and through `..`, stays editable.
    let run = p.run_id.clone();
    for rel in [
        format!(".dla/{run}/harness/H1/driver/dla_harness.py"),
        format!("src/../.dla/{run}/harness/H3/driver/DlaHarness.java"),
    ] {
        let out = p.hook(&["write-hook"], &write_json("Write", &rel, p.path()));
        assert!(out.allowed(), "{rel}: {out:?}");
    }
}

#[test]
fn bash_hook_denies_writes_to_the_generated_files_but_not_to_the_driver() {
    let p = project("symptom");
    let run = p.run_id.clone();
    let d = format!(".dla/{run}");
    for cmd in [
        format!("echo '{{}}' > {d}/trend/T1.json"),
        format!("cp /tmp/x {d}/series/S1.json"),
        format!("sed -i 's/1/2/' {d}/gate/final.json"),
        format!("python3 -c \"open('{d}/intervene/I1.json','w').write('x')\""),
        format!("tee {d}/bundle/F1/index.json < /tmp/x"),
        format!("echo x > {d}/harness/H1/work/a.py"),
        format!("echo x > {d}/harness/H1/driver/../harness.json"),
        format!("cp /tmp/x {d}/harness/H1/driver/a.py {d}/trend/T1.json"),
    ] {
        let out = p.hook(&["bash-hook"], &bash_json(&cmd, p.path()));
        assert!(out.denied(), "{cmd}: {out:?}");
    }
    for cmd in [
        format!("cat > {d}/harness/H1/driver/dla_harness.py <<'EOF'\nx = 1\nEOF"),
        format!("cp /tmp/x {d}/harness/H2/driver/dla_harness.py"),
        format!("sed -i 's/a/b/' {d}/harness/H1/driver/DlaHarness.java"),
        // Reading a generated file is not writing it.
        format!("cat {d}/trend/T1.json"),
    ] {
        let out = p.hook(&["bash-hook"], &bash_json(&cmd, p.path()));
        assert!(out.allowed(), "{cmd}: {out:?}");
    }
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
    // The block says how much of the budget is left, so a stop with time to spare is not
    // mistaken for a natural end.
    assert!(
        reason.contains("budget minutes remain: measure what is open instead of stopping"),
        "{reason}"
    );
    assert_eq!(p.state()["stop_blocks"], 1);
    // Fixing it lets the session stop, and the counter does not stamp the run.
    answer_intake(&p);
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert!(out.allowed(), "{out:?}");
    assert_eq!(p.state()["status"], "active");
}

#[test]
fn stop_hook_block_says_to_close_not_to_measure_under_the_budget_floor() {
    let p = project("symptom");
    let path = p.run_dir().join("state.json");
    let mut st = p.state();
    let soon = (chrono::Utc::now() + chrono::Duration::minutes(4))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    st["budget"]["deadline"] = json!(soon);
    fs::write(&path, st.to_string()).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let v = out.json();
    assert_eq!(v["decision"], "block", "{out:?}");
    let reason = v["reason"].as_str().unwrap();
    assert!(
        reason.contains("budget minutes remain, too few to start another measurement"),
        "{reason}"
    );
    assert!(
        !reason.contains("measure what is open instead of stopping"),
        "{reason}"
    );
    // The default run here is unattended (consent from the runner): it never extends.
    assert!(
        reason.contains("too few to start another measurement: close what is open"),
        "{reason}"
    );
    assert!(!reason.contains("extend"), "{reason}");
}

#[test]
fn stop_hook_under_the_floor_offers_an_interactive_core_the_extension_first() {
    let p = project("symptom");
    let path = p.run_dir().join("state.json");
    let mut st = p.state();
    let soon = (chrono::Utc::now() + chrono::Duration::minutes(4))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    st["budget"]["deadline"] = json!(soon);
    // Interactive: the consent object exists (stage 1) but the user gave it.
    st["consent"]["given_by"] = json!("user");
    fs::write(&path, st.to_string()).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(
        reason.contains("too few to start another measurement: ask the user once whether to extend the budget (dla run extend <dur>)"),
        "{reason}"
    );
    // Closing comes only after a no, or no answer.
    assert!(
        reason.contains("if the user declines or does not answer, close what is open"),
        "{reason}"
    );
}

#[test]
fn stop_hook_after_a_refusal_with_nothing_smaller_that_fits_counts_as_the_floor() {
    let p = project("symptom");
    let path = p.run_dir().join("state.json");
    let mut st = p.state();
    let far = (chrono::Utc::now() + chrono::Duration::minutes(50))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    st["budget"]["deadline"] = json!(far);
    st["consent"]["given_by"] = json!("user");
    fs::write(&path, st.to_string()).unwrap();
    let refusals = p.run_dir().join("time_refusals.jsonl");
    let line = |none_fits: bool| {
        format!(
            "{}\n",
            json!({"ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                   "worker": null, "none_fits": none_fits})
        )
    };
    // A refusal that suggested a smaller run leaves the keep-measuring instruction.
    fs::write(&refusals, line(false)).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(
        reason.contains("measure what is open instead of stopping"),
        "{reason}"
    );
    // One that said even the smallest run does not fit puts the core at the floor.
    fs::write(&refusals, line(true)).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(
        reason.contains("ask the user once whether to extend the budget (dla run extend <dur>)"),
        "{reason}"
    );
    assert!(
        !reason.contains("measure what is open instead of stopping"),
        "{reason}"
    );
}

#[test]
fn a_workers_refusal_never_puts_the_core_at_the_floor() {
    let p = project("symptom");
    let path = p.run_dir().join("state.json");
    let mut st = p.state();
    let far = (chrono::Utc::now() + chrono::Duration::minutes(50))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    st["budget"]["deadline"] = json!(far);
    st["consent"]["given_by"] = json!("user");
    fs::write(&path, st.to_string()).unwrap();
    let line = format!(
        "{}\n",
        json!({"ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
               "worker": "go", "none_fits": true})
    );
    fs::write(p.run_dir().join("time_refusals.jsonl"), line).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(
        reason.contains("measure what is open instead of stopping"),
        "{reason}"
    );
    assert!(!reason.contains("ask the user once"), "{reason}");
}

#[test]
fn the_reason_after_a_refusal_with_most_of_the_budget_left_says_what_happened() {
    let p = project("symptom");
    let path = p.run_dir().join("state.json");
    let mut st = p.state();
    let far = (chrono::Utc::now() + chrono::Duration::minutes(50))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    st["budget"]["deadline"] = json!(far);
    st["consent"]["given_by"] = json!("user");
    fs::write(&path, st.to_string()).unwrap();
    let line = format!(
        "{}\n",
        json!({"ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
               "worker": null, "none_fits": true})
    );
    fs::write(p.run_dir().join("time_refusals.jsonl"), line).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(
        reason.contains("a command was refused with nothing smaller that fits: ask the user once"),
        "{reason}"
    );
    assert!(!reason.contains("too few"), "{reason}");
}

#[test]
fn an_unattended_core_after_a_refusal_with_nothing_smaller_that_fits_is_told_to_close() {
    let p = project("symptom");
    let path = p.run_dir().join("state.json");
    let mut st = p.state();
    let far = (chrono::Utc::now() + chrono::Duration::minutes(50))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    st["budget"]["deadline"] = json!(far);
    // The consent stays the runner's: the run is unattended and never extends.
    assert_eq!(st["consent"]["given_by"], json!("runner"));
    fs::write(&path, st.to_string()).unwrap();
    let line = format!(
        "{}\n",
        json!({"ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
               "worker": null, "none_fits": true})
    );
    fs::write(p.run_dir().join("time_refusals.jsonl"), line).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(
        reason.contains(
            "a command was refused with nothing smaller that fits: close what is open with a reason and a remedy"
        ),
        "{reason}"
    );
    assert!(!reason.contains("ask the user"), "{reason}");
    assert!(!reason.contains("too few"), "{reason}");
}

#[test]
fn stop_hook_at_a_spent_budget_asks_an_interactive_core_to_extend_first() {
    let p = project("symptom");
    let path = p.run_dir().join("state.json");
    let mut st = p.state();
    let past = (chrono::Utc::now() - chrono::Duration::minutes(4))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    st["budget"]["deadline"] = json!(past);
    let consent = st["consent"].clone();
    // Interactive: the consent object exists (stage 1) but the user gave it.
    st["consent"]["given_by"] = json!("user");
    fs::write(&path, st.to_string()).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(
        reason.contains("ask the user once whether to extend the budget"),
        "{reason}"
    );
    assert!(reason.contains("dla run extend"), "{reason}");
    // An unattended run (consent from the runner) never extends: it only closes.
    st["consent"] = consent;
    fs::write(&path, st.to_string()).unwrap();
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    let reason = out.json()["reason"].as_str().unwrap().to_owned();
    assert!(
        reason.contains("budget minutes are spent: close what is open"),
        "{reason}"
    );
    assert!(!reason.contains("extend"), "{reason}");
}

#[test]
fn three_stop_attempts_with_no_ledger_change_stamp_the_run_incomplete() {
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
        why.starts_with(
            "stop hook let the stop through after 3 stop attempts with no ledger change: "
        ),
        "{why}"
    );
    assert!(why.contains("intake.target"), "{why}");
    // The report prints INCOMPLETE with that reason and its own fix text.
    let out = p.dla().arg("report").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.contains("Status: INCOMPLETE"), "{md}");
    assert!(md.contains("3 stop attempts with no ledger change"), "{md}");
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
fn a_siblings_slice_writes_do_not_reset_a_stuck_workers_counter() {
    let p = project("repository");
    p.ok(&["run", "dispatch", "--workers", "jvm,go"]);
    let t = tempfile::tempdir().unwrap();
    let tr = transcript(t.path(), &["dla --worker jvm ledger add --origin census"]);
    let stop = |p: &P| p.hook(&["stop-hook", "--subagent"], &subagent_json(p.path(), &tr));
    assert_eq!(stop(&p).json()["decision"], "block");
    // The go worker keeps writing to its own slice between the jvm worker's attempts.
    let wrote = p
        .dla()
        .args(["--worker", "go", "ledger", "done"])
        .output()
        .unwrap();
    assert_eq!(wrote.status.code(), Some(0), "{wrote:?}");
    assert_eq!(stop(&p).json()["decision"], "block");
    assert_eq!(p.state()["worker_stop"]["jvm"]["blocks"], 2);
    let third = stop(&p);
    assert!(third.allowed(), "{third:?}");
    assert!(third.stderr.contains("stalled"), "{third:?}");
}

#[test]
fn a_worker_stop_obeys_its_own_refusal_and_not_the_cores_or_a_siblings() {
    let p = project("repository");
    p.ok(&["run", "dispatch", "--workers", "jvm,go"]);
    let path = p.run_dir().join("state.json");
    let mut st = p.state();
    let far = (chrono::Utc::now() + chrono::Duration::minutes(50))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    st["budget"]["deadline"] = json!(far);
    for w in st["workers"].as_array_mut().unwrap() {
        w["deadline"] = json!(far);
    }
    fs::write(&path, st.to_string()).unwrap();
    let refusals = p.run_dir().join("time_refusals.jsonl");
    let line = |worker: Value| {
        format!(
            "{}\n",
            json!({"ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                   "worker": worker, "none_fits": true})
        )
    };
    let t = tempfile::tempdir().unwrap();
    let tr = transcript(t.path(), &["dla --worker jvm ledger add --origin census"]);
    let reason = |p: &P| {
        let out = p.hook(&["stop-hook", "--subagent"], &subagent_json(p.path(), &tr));
        out.json()["reason"].as_str().unwrap().to_owned()
    };
    // The core's and a sibling's refusals do not move worker jvm to the floor.
    fs::write(&refusals, line(Value::Null) + &line(json!("go"))).unwrap();
    let r = reason(&p);
    assert!(
        r.contains("measure what is open instead of stopping"),
        "{r}"
    );
    assert!(!r.contains("nothing smaller that fits"), "{r}");
    // Its own does, and it is told to close what is open (a worker never asks the user).
    fs::write(&refusals, line(json!("jvm"))).unwrap();
    let r = reason(&p);
    assert!(
        r.contains("nothing smaller that fits: close what is open"),
        "{r}"
    );
    assert!(!r.contains("ask the user"), "{r}");
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

// ------------------------------------------------- a dla command is still running

/// Registers `command` of the run as running with the process `pid`, started `minutes_ago`
/// minutes ago, the way a long `dla` command registers itself.
fn register_command(p: &P, command: &str, pid: u32, minutes_ago: i64) {
    use dla_core::state::{ProcessKind, ProcessRecord, ProcessRole};
    dla_core::RunState::update(&p.run_dir(), |s| {
        s.processes.push(ProcessRecord {
            pid,
            start_time: dla_sample::os::process_start_time(pid),
            kind: ProcessKind::Command,
            command: command.to_owned(),
            started_at: chrono::Utc::now() - chrono::Duration::minutes(minutes_ago),
            stopped_at: None,
            role: ProcessRole::Attached,
            worker: None,
            cleanup: Vec::new(),
        });
        Ok(())
    })
    .unwrap();
}

fn stop_attempts(p: &P, n: usize) -> Vec<Out> {
    (0..n)
        .map(|_| p.hook(&["stop-hook"], &stop_json(p.path())))
        .collect()
}

#[test]
fn a_running_dla_command_blocks_the_stop_without_counting_or_stamping() {
    let p = project("symptom");
    let mut child = std::process::Command::new("sleep")
        .arg("120")
        .spawn()
        .unwrap();
    register_command(&p, "dla census", child.id(), 4);
    for out in stop_attempts(&p, 5) {
        let v = out.json();
        assert_eq!(v["decision"], "block", "{out:?}");
        let reason = v["reason"].as_str().unwrap();
        assert!(
            reason.contains("`dla census` of this run is still running"),
            "{reason}"
        );
        assert!(reason.contains("Wait for it"), "{reason}");
        assert!(reason.contains("not counted"), "{reason}");
    }
    let st = p.state();
    assert_eq!(st["status"], "active", "{st}");
    assert!(
        st["stop_blocks"].is_null() || st["stop_blocks"] == 0,
        "{st}"
    );
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn a_registered_command_whose_process_is_gone_counts_as_not_running() {
    let p = project("symptom");
    let mut child = std::process::Command::new("sleep")
        .arg("120")
        .spawn()
        .unwrap();
    let pid = child.id();
    register_command(&p, "dla census", pid, 4);
    child.kill().unwrap();
    child.wait().unwrap();
    let outs = stop_attempts(&p, 3);
    assert_eq!(outs[0].json()["decision"], "block", "{:?}", outs[0]);
    assert!(
        !outs[0].json()["reason"]
            .as_str()
            .unwrap()
            .contains("still running"),
        "{:?}",
        outs[0]
    );
    // The third attempt stamps, as it always did.
    assert!(outs[2].allowed(), "{:?}", outs[2]);
    assert_eq!(p.state()["status"], "incomplete");
}

#[test]
fn a_command_running_past_the_limit_is_taken_for_hung() {
    let p = project("symptom");
    let mut child = std::process::Command::new("sleep")
        .arg("120")
        .spawn()
        .unwrap();
    register_command(&p, "dla census", child.id(), 45);
    let outs = stop_attempts(&p, 3);
    assert!(
        !outs[0].json()["reason"]
            .as_str()
            .unwrap()
            .contains("still running"),
        "{:?}",
        outs[0]
    );
    assert!(outs[2].allowed(), "{:?}", outs[2]);
    assert_eq!(p.state()["status"], "incomplete");
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn a_stamped_run_has_its_stamp_lifted_by_the_next_working_command_and_keeps_it_in_its_history() {
    let p = project("symptom");
    stop_attempts(&p, 3);
    let st = p.state();
    assert_eq!(st["status"], "incomplete", "{st}");
    assert_eq!(st["stop_stamps"].as_array().unwrap().len(), 1, "{st}");
    // Looking at the run does not lift it: status, gate check, report, ledger list, bundle.
    p.ok(&["run", "status"]);
    let _ = p.dla().args(["gate", "check", "--final"]).output().unwrap();
    let _ = p.dla().arg("report").output().unwrap();
    let _ = p.dla().args(["ledger", "list"]).output().unwrap();
    assert_eq!(p.state()["status"], "incomplete", "{}", p.state());
    // A command that works on the run lifts it before it does its work, whether or not it
    // then writes to the ledger (this one fails: there is no such series).
    let o = p.dla().args(["trend", "S9"]).output().unwrap();
    assert_ne!(o.status.code(), Some(0));
    let st = p.state();
    assert_eq!(st["status"], "active", "{st}");
    assert!(st["incomplete_reason"].is_null(), "{st}");
    let stamp = &st["stop_stamps"][0];
    assert!(stamp["lifted_at"].is_string(), "{st}");
    assert!(
        stamp["reason"].as_str().unwrap().contains("stop attempts"),
        "{st}"
    );
}

#[test]
fn an_aborted_run_stays_aborted_whatever_runs_after() {
    let p = project("symptom");
    p.ok(&["run", "abort", "--reason", "x"]);
    assert_eq!(p.state()["status"], "aborted");
    let _ = p.dla().args(["trend", "S9"]).output().unwrap();
    answer_intake(&p);
    assert_eq!(p.state()["status"], "aborted", "{}", p.state());
    assert!(p.state()["stop_stamps"].is_null(), "{}", p.state());
}

#[test]
fn a_long_command_registers_itself_while_it_runs_and_marks_itself_stopped() {
    let p = project("symptom");
    let out = p.dla().arg("report").output().unwrap();
    assert!(out.status.success() || out.status.code() == Some(2));
    let st = p.state();
    let rec = st["processes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "command")
        .unwrap_or_else(|| panic!("no registered command: {st}"));
    assert_eq!(rec["command"], "dla report");
    assert_eq!(rec["role"], "attached");
    assert!(rec["stopped_at"].is_string(), "{rec}");
    // A command that is not on the list leaves nothing behind.
    p.ok(&["run", "status"]);
    assert_eq!(
        p.state()["processes"].as_array().unwrap().len(),
        1,
        "{}",
        p.state()
    );
}

// ----------------------------------------------------- budget left and a queue open

/// A project with a small Python library (a cached function and a class-level dict with a
/// writer and no remover), swept, counted and given its workloads: its ranked candidates
/// make the `python` queue open. Intake is answered; the budget is the run's default, far
/// over its floor.
fn queue_project() -> P {
    let p = project("repository");
    let put = |rel: &str, text: &str| {
        let f = p.path().join(rel);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, text).unwrap();
    };
    put("shop/__init__.py", "");
    put(
        "shop/cache.py",
        "import functools\n\n\n@functools.lru_cache(None)\ndef price_for(handler, key):\n    return (handler, key)\n",
    );
    put(
        "shop/reg.py",
        "class Registry:\n    _by_key = {}\n\n    def put(self, key, value):\n        Registry._by_key[key] = value\n",
    );
    for args in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
    ] {
        let out = p
            .dla()
            .env("DLA_CONSENT_TIERS", "1,2,3")
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "{args:?}: {out:?}");
    }
    p
}

/// The missing items of the final gate that are `kind`.
fn missing_kinds(p: &P) -> Vec<String> {
    let out = p.dla().args(["gate", "check", "--final"]).output().unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    v["missing"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|m| m["kind"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Moves the run's deadline to `minutes` from now.
fn set_deadline(p: &P, minutes: i64) {
    let mut st = p.state();
    st["budget"]["deadline"] = json!(
        (chrono::Utc::now() + chrono::Duration::minutes(minutes))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    );
    fs::write(p.run_dir().join("state.json"), st.to_string()).unwrap();
}

#[test]
fn the_fixture_has_an_open_queue_in_the_final_gate() {
    let p = queue_project();
    let kinds = missing_kinds(&p);
    assert!(kinds.iter().any(|k| k == "queue"), "{kinds:?}");
}

#[test]
fn an_open_queue_with_budget_over_the_floor_is_not_given_up_by_four_stop_attempts() {
    let p = queue_project();
    for out in stop_attempts(&p, 4) {
        let v = out.json();
        assert_eq!(v["decision"], "block", "{out:?}");
        let reason = v["reason"].as_str().unwrap();
        assert!(reason.len() <= 1000, "{} bytes", reason.len());
        assert!(reason.contains("This stop is not counted: "), "{reason}");
        assert!(reason.contains(" budget minutes remain and the python queue is open. Run its next batch (dla harness gen --lang python --driver --next)"), "{reason}");
        // The next measurement first; the abort only for a blocker, never a plain one.
        assert!(
            reason.contains("dla run abort --blocker <kind> --reason \"<what blocks the run>\""),
            "{reason}"
        );
        assert!(!reason.contains("dla run abort --reason"), "{reason}");
        assert!(
            reason.find("Run its next batch").unwrap() < reason.find("dla run abort").unwrap(),
            "{reason}"
        );
        // After the budget line, before the items.
        let budget = reason
            .find("measure what is open instead of stopping")
            .unwrap();
        assert!(
            budget < reason.find("This stop is not counted").unwrap(),
            "{reason}"
        );
    }
    let st = p.state();
    assert_eq!(st["status"], "active", "{st}");
    assert!(
        st["stop_blocks"].is_null() || st["stop_blocks"] == 0,
        "{st}"
    );
    assert!(st["incomplete_reason"].is_null(), "{st}");
}

#[test]
fn an_open_queue_under_the_floor_lets_the_third_stop_attempt_through() {
    let p = queue_project();
    set_deadline(&p, 4);
    let outs = stop_attempts(&p, 3);
    for out in &outs[..2] {
        let reason = out.json()["reason"].as_str().unwrap().to_owned();
        assert!(!reason.contains("not counted"), "{reason}");
    }
    assert!(outs[2].allowed(), "{:?}", outs[2]);
    assert!(outs[2].stderr.contains("INCOMPLETE"), "{:?}", outs[2]);
    assert_eq!(p.state()["status"], "incomplete");
}

#[test]
fn budget_left_and_no_open_queue_lets_the_third_stop_attempt_through() {
    // Findings and open items alone (the symptom run has no queue) never hold a stop.
    let p = project("symptom");
    assert!(!missing_kinds(&p).iter().any(|k| k == "queue"));
    let outs = stop_attempts(&p, 3);
    for out in &outs[..2] {
        let reason = out.json()["reason"].as_str().unwrap().to_owned();
        assert!(!reason.contains("not counted"), "{reason}");
    }
    assert!(outs[2].allowed(), "{:?}", outs[2]);
    assert_eq!(p.state()["status"], "incomplete");
}

#[test]
fn a_queue_closed_for_a_blocker_does_not_hold_the_stop() {
    let p = queue_project();
    p.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "env_unavailable",
        "--remedy",
        "no interpreter for the driver",
        "--runtime",
        "python",
    ]);
    assert!(!missing_kinds(&p).iter().any(|k| k == "queue"));
    let outs = stop_attempts(&p, 3);
    assert!(outs[2].allowed(), "{:?}", outs[2]);
    assert_eq!(p.state()["status"], "incomplete");
}

#[test]
fn an_open_queue_after_a_refusal_with_nothing_smaller_that_fits_is_counted_as_today() {
    let p = queue_project();
    let line = format!(
        "{}\n",
        json!({"ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
               "worker": null, "none_fits": true})
    );
    fs::write(p.run_dir().join("time_refusals.jsonl"), line).unwrap();
    let outs = stop_attempts(&p, 3);
    for out in &outs[..2] {
        let reason = out.json()["reason"].as_str().unwrap().to_owned();
        assert!(!reason.contains("not counted"), "{reason}");
        assert!(reason.contains("nothing smaller that fits"), "{reason}");
    }
    assert!(outs[2].allowed(), "{:?}", outs[2]);
    assert_eq!(p.state()["status"], "incomplete");
}

#[test]
fn a_worker_slice_with_an_open_queue_is_held_and_not_counted() {
    let p = queue_project();
    p.ok(&["run", "dispatch", "--workers", "python"]);
    let t = tempfile::tempdir().unwrap();
    let tr = transcript(
        t.path(),
        &["dla --worker python ledger add --origin census"],
    );
    for _ in 0..4 {
        let out = p.hook(&["stop-hook", "--subagent"], &subagent_json(p.path(), &tr));
        let v = out.json();
        assert_eq!(v["decision"], "block", "{out:?}");
        let reason = v["reason"].as_str().unwrap();
        assert!(reason.contains("This stop is not counted: "), "{reason}");
        assert!(reason.contains("the python queue is open"), "{reason}");
        // A worker is not offered the end of the whole run.
        assert!(!reason.contains("dla run abort"), "{reason}");
    }
    let st = p.state();
    let stalled = st["workers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|w| w["stalled"] == true);
    assert!(!stalled, "{st}");
    // Under the worker's own floor the third attempt stalls it, as today.
    let mut st = p.state();
    let soon = (chrono::Utc::now() + chrono::Duration::minutes(4))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    for w in st["workers"].as_array_mut().unwrap() {
        w["deadline"] = json!(soon);
    }
    fs::write(p.run_dir().join("state.json"), st.to_string()).unwrap();
    let outs: Vec<Out> = (0..3)
        .map(|_| p.hook(&["stop-hook", "--subagent"], &subagent_json(p.path(), &tr)))
        .collect();
    assert!(outs[2].allowed(), "{:?}", outs[2]);
    assert!(outs[2].stderr.contains("stalled"), "{:?}", outs[2]);
}

#[test]
fn run_abort_for_the_user_ends_a_held_run_and_the_stop_is_allowed() {
    let p = queue_project();
    let held = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert_eq!(held.json()["decision"], "block", "{held:?}");
    p.ok(&[
        "run",
        "abort",
        "--blocker",
        "user",
        "--reason",
        "the owner asked to end it",
    ]);
    let out = p.hook(&["stop-hook"], &stop_json(p.path()));
    assert!(out.allowed(), "{out:?}");
    assert_eq!(p.state()["status"], "aborted");
    let report = p.dla().arg("report").output().unwrap();
    let md = String::from_utf8(report.stdout).unwrap();
    assert!(
        md.contains("run is aborted: aborted at the user's request: the owner asked to end it"),
        "{md}"
    );
    assert!(md.contains("the owner asked to end it"), "{md}");
}

// ------------------------------------------- budget left and growth measured, not decided

/// The id of the leak-side finding at `shop/reg.py` in the run of `p`.
fn registry_finding(p: &P) -> dla_core::FindingId {
    let l = dla_core::Ledger::load(&p.run_dir()).unwrap();
    l.findings()
        .find(|f| {
            f.site
                .as_deref()
                .is_some_and(|s| s.starts_with("shop/reg.py"))
        })
        .map(|f| f.id.clone())
        .unwrap_or_else(|| panic!("no finding at shop/reg.py"))
}

fn closed_unresolved(p: &P, f: &dla_core::FindingId) {
    let mut l = dla_core::Ledger::load(&p.run_dir()).unwrap();
    l.transition(
        f,
        dla_core::FindingState::Unresolved,
        Some(dla_core::finding::UnresolvedReason::NotExercised),
        Some("x".into()),
        vec![],
        "t",
    )
    .unwrap();
}

/// Makes the growth of `f` a measured lead the project's own test loop (harness H1) did not
/// decide: H1, its series S1 and trend T1 with a passed column tied to `f`, and an alloc-test of
/// `f` on H1 that favored leak (written directly, as a real one needs a built harness).
fn growth_on_a_loop(p: &P, f: &dla_core::FindingId) {
    let run_dir = p.run_dir();
    let m: dla_core::HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "tools": [dla_core::harness::PYTEST_TOOL],
        "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&run_dir).unwrap();
    fs::create_dir_all(run_dir.join("series")).unwrap();
    fs::write(
        run_dir.join("series/S1.json"),
        json!({"id": "S1", "source": {"kind": "harness", "harness": "H1"},
            "axis": "n", "columns": [], "points": []})
        .to_string(),
    )
    .unwrap();
    fs::write(
        run_dir.join("harness/H1/run-S1.json"),
        json!({"series": ["S1"]}).to_string(),
    )
    .unwrap();
    let col: dla_core::trend::ColumnTrend = serde_json::from_value(json!({
        "name": "probe:Registry._by_key", "unit": "count", "kind": "counter",
        "region": "heap", "n": 8, "noise": 0.0, "rise": 8.0, "shape": "steady",
        "gate": "pass", "gate_failed": [], "finding": f.to_string(),
    }))
    .unwrap();
    dla_core::trend::TrendReport {
        schema_version: 1,
        id: "T1".into(),
        series: "S1".into(),
        thresholds: dla_stats::GateThresholds::default(),
        columns: vec![col],
        targets: vec![],
    }
    .save(&run_dir)
    .unwrap();
    let mut l = dla_core::Ledger::load(&run_dir).unwrap();
    for (kind, result) in [
        (
            dla_core::EvidenceKind::Discriminator,
            json!({"test": "plateau+purge", "favors": "leak", "supports": [],
                "finding": f.to_string(), "harness": "H1"}),
        ),
        (
            dla_core::EvidenceKind::GrowthSeries,
            json!({"trend": "T1", "series": "S1", "n": 8, "passed": [], "failed": 0,
                "gate": "pass"}),
        ),
    ] {
        l.add_evidence(dla_core::NewEvidence {
            source: dla_core::Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: dla_core::Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: dla_core::Authority::Test,
            kind,
            result,
            raw: None,
            coverage: dla_core::evidence::EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
    }
}

/// A run with no open queue and a closed finding whose growth the test loop measured and its
/// alloc-test (favoring leak) did not decide: the final gate lists one `growth` item.
fn growth_project() -> (P, dla_core::FindingId) {
    let p = queue_project();
    p.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "env_unavailable",
        "--remedy",
        "no interpreter for the driver",
        "--runtime",
        "python",
    ]);
    let f = registry_finding(&p);
    closed_unresolved(&p, &f);
    growth_on_a_loop(&p, &f);
    (p, f)
}

#[test]
fn growth_measured_and_not_decided_with_budget_over_the_floor_is_not_given_up_by_four_stops() {
    let (p, f) = growth_project();
    let kinds = missing_kinds(&p);
    assert!(kinds.iter().any(|k| k == "growth"), "{kinds:?}");
    assert!(!kinds.iter().any(|k| k == "queue"), "{kinds:?}");
    for out in stop_attempts(&p, 4) {
        let v = out.json();
        assert_eq!(v["decision"], "block", "{out:?}");
        let reason = v["reason"].as_str().unwrap();
        assert!(reason.len() <= 1000, "{} bytes", reason.len());
        assert!(reason.contains("This stop is not counted: "), "{reason}");
        assert!(
            reason.contains(&format!(
                " budget minutes remain and growth that was measured is not decided ({f}). Run dla ledger "
            )),
            "{reason}"
        );
        assert!(
            reason.contains("dla run abort --blocker <kind> --reason \"<what blocks the run>\"")
                && !reason.contains("dla run abort --reason"),
            "{reason}"
        );
    }
    let st = p.state();
    assert_eq!(st["status"], "active", "{st}");
    assert!(
        st["stop_blocks"].is_null() || st["stop_blocks"] == 0,
        "{st}"
    );
    assert!(st["incomplete_reason"].is_null(), "{st}");
}

#[test]
fn growth_measured_and_not_decided_under_the_floor_lets_the_third_stop_through() {
    let (p, _) = growth_project();
    set_deadline(&p, 4);
    let outs = stop_attempts(&p, 3);
    for out in &outs[..2] {
        let reason = out.json()["reason"].as_str().unwrap().to_owned();
        assert!(!reason.contains("not counted"), "{reason}");
    }
    assert!(outs[2].allowed(), "{:?}", outs[2]);
    assert_eq!(p.state()["status"], "incomplete");
}

#[test]
fn an_open_queue_wins_over_a_growth_item_in_the_hold_note() {
    let p = queue_project();
    let f = registry_finding(&p);
    // Closed directly, past the refusal a queue-pending target would get from `ledger resolve`:
    // the queue stays open for the other candidates.
    closed_unresolved(&p, &f);
    growth_on_a_loop(&p, &f);
    let kinds = missing_kinds(&p);
    assert!(kinds.iter().any(|k| k == "queue"), "{kinds:?}");
    assert!(kinds.iter().any(|k| k == "growth"), "{kinds:?}");
    for out in stop_attempts(&p, 4) {
        let v = out.json();
        assert_eq!(v["decision"], "block", "{out:?}");
        let reason = v["reason"].as_str().unwrap();
        assert!(reason.contains("the python queue is open"), "{reason}");
        assert!(
            !reason.contains("growth that was measured is not decided"),
            "{reason}"
        );
    }
    assert!(
        p.state()["stop_blocks"].is_null() || p.state()["stop_blocks"] == 0,
        "{}",
        p.state()
    );
}

/// The same growth with its finding still open (a candidate, and then under test): the gate lists
/// it as a `finding` item, not a `growth` one, and the stop is held all the same.
#[test]
fn growth_of_a_finding_still_open_holds_the_stop_like_a_closed_one() {
    let p = queue_project();
    p.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "env_unavailable",
        "--remedy",
        "no interpreter for the driver",
        "--runtime",
        "python",
    ]);
    let f = registry_finding(&p);
    growth_on_a_loop(&p, &f);
    let check = |p: &P| {
        let kinds = missing_kinds(p);
        assert!(!kinds.iter().any(|k| k == "queue"), "{kinds:?}");
        assert!(!kinds.iter().any(|k| k == "growth"), "{kinds:?}");
        for out in stop_attempts(p, 4) {
            let v = out.json();
            assert_eq!(v["decision"], "block", "{out:?}");
            let reason = v["reason"].as_str().unwrap();
            assert!(reason.len() <= 1000, "{} bytes", reason.len());
            assert!(
                reason.contains(&format!(
                    " budget minutes remain and growth that was measured is not decided ({f}). Run dla "
                )),
                "{reason}"
            );
        }
        let st = p.state();
        assert_eq!(st["status"], "active", "{st}");
        assert!(
            st["stop_blocks"].is_null() || st["stop_blocks"] == 0,
            "{st}"
        );
    };
    check(&p);
    // Under test with the workload of the loop, as `dla ledger test` puts it.
    let mut l = dla_core::Ledger::load(&p.run_dir()).unwrap();
    l.add_evidence(dla_core::NewEvidence {
        source: dla_core::Source::Repo {
            commit: "c".into(),
            path: None,
        },
        tool: dla_core::Tool {
            name: "dla".into(),
            version: "1".into(),
        },
        authority: dla_core::Authority::Test,
        kind: dla_core::EvidenceKind::Workload,
        result: json!({"harness": "H1", "mode": "driver", "series": ["S1"]}),
        raw: None,
        coverage: dla_core::evidence::EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
    let newest = |kind: dla_core::EvidenceKind| {
        l.evidence()
            .filter(|e| e.kind == kind)
            .last()
            .map(|e| e.evidence_id.to_string())
            .unwrap()
    };
    let (w, g) = (
        newest(dla_core::EvidenceKind::Workload),
        newest(dla_core::EvidenceKind::GrowthSeries),
    );
    let fs = f.to_string();
    p.ok(&[
        "ledger",
        "add",
        "--sibling-of",
        &fs,
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM",
        "--refute-if",
        "growth continues over the extra cycles",
        "--evidence",
        &g,
    ]);
    p.ok(&["ledger", "test", &fs, "--evidence", &w]);
    check(&p);
    // Under the floor the third stop goes through, as for a closed one.
    set_deadline(&p, 4);
    let outs = stop_attempts(&p, 3);
    assert!(outs[2].allowed(), "{:?}", outs[2]);
}

// ------------------------------------------------ abort is no free exit while budget is left

/// What `dla run abort <args>` printed: the exit code and the JSON.
fn abort_out(p: &P, args: &[&str]) -> (i32, Value) {
    let out = p.dla().args(["run", "abort"]).args(args).output().unwrap();
    (
        out.status.code().unwrap(),
        serde_json::from_slice(&out.stdout).unwrap_or(Value::Null),
    )
}

fn refused_with(p: &P, args: &[&str], code: &str) -> Value {
    let (exit, v) = abort_out(p, args);
    assert_eq!(exit, 2, "{args:?}: {v}");
    assert_eq!(v["error"]["code"], code, "{args:?}: {v}");
    assert_eq!(
        p.state()["status"],
        "active",
        "a refused abort ends nothing"
    );
    v["error"].clone()
}

#[test]
fn a_plain_abort_with_an_open_queue_and_budget_left_is_refused_and_names_the_next_batch() {
    let p = queue_project();
    let e = refused_with(&p, &[], "budget-left");
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains(" budget minutes remain and work is open (queue: "),
        "{message}"
    );
    assert!(
        message.contains("candidate(s) of python are ranked for measurement"),
        "{message}"
    );
    let fix = e["fix"].as_str().unwrap();
    assert!(
        fix.starts_with("dla harness gen --lang python --driver --next   or   dla run abort --blocker <user|tool|build|environment|privilege|refusal> --reason \"<what blocks the run>\""),
        "{fix}"
    );
    // A reason alone does not make it pass, whatever it says.
    refused_with(
        &p,
        &["--reason", "139 ranked candidates not targeted"],
        "budget-left",
    );
}

#[test]
fn an_abort_for_the_user_passes_with_an_open_queue() {
    let p = queue_project();
    let (exit, v) = abort_out(&p, &["--blocker", "user"]);
    assert_eq!(exit, 0, "{v}");
    assert_eq!(v["status"], "aborted");
    assert_eq!(v["reason"], "aborted at the user's request");
    assert_eq!(
        p.state()["incomplete_reason"],
        "aborted at the user's request"
    );
    let md = String::from_utf8(p.dla().arg("report").output().unwrap().stdout).unwrap();
    assert!(
        md.contains("run is aborted: aborted at the user's request"),
        "{md}"
    );
}

#[test]
fn a_blocker_of_the_environment_needs_a_reason_that_names_it() {
    let p = queue_project();
    // Empty reason, no reason, too short, a count of the work left: refused, nothing ended.
    for args in [
        &["--blocker", "tool"][..],
        &["--blocker", "tool", "--reason", ""][..],
        &["--blocker", "build", "--reason", "stuck"][..],
        &[
            "--blocker",
            "tool",
            "--reason",
            "unattended run: 4 leak hypotheses confirmed plus 2 SDK caches measured; 139 ranked candidates not targeted (each harness batch settles 2-3)",
        ][..],
    ] {
        let e = refused_with(&p, args, "missing-argument");
        assert!(e["message"].as_str().unwrap().contains("--reason"), "{e}");
    }
    // An unknown kind, and a code with a kind that takes none.
    refused_with(
        &p,
        &["--blocker", "boredom", "--reason", "the run is tired of it"],
        "invalid-value",
    );
    refused_with(
        &p,
        &[
            "--blocker",
            "tool",
            "--code",
            "time-left",
            "--reason",
            "go is not installed",
        ],
        "invalid-value",
    );
    // A blocker that names what is in the way passes.
    let (exit, v) = abort_out(
        &p,
        &[
            "--blocker",
            "tool",
            "--reason",
            "the go toolchain is not installed",
        ],
    );
    assert_eq!(exit, 0, "{v}");
    assert_eq!(
        v["reason"],
        "aborted with blocker tool: the go toolchain is not installed"
    );
}

#[test]
fn a_refusal_blocker_needs_the_code_of_the_refusal() {
    let p = queue_project();
    for args in [
        &[
            "--blocker",
            "refusal",
            "--reason",
            "the run cannot satisfy the refusal",
        ][..],
        &[
            "--blocker",
            "refusal",
            "--code",
            "Not A Code",
            "--reason",
            "the run cannot satisfy it",
        ][..],
    ] {
        let e = refused_with(&p, args, "missing-argument");
        assert!(e["message"].as_str().unwrap().contains("--code"), "{e}");
    }
    let (exit, v) = abort_out(
        &p,
        &[
            "--blocker",
            "refusal",
            "--code",
            "capture-not-summarized",
            "--reason",
            "the capture needs a privilege the run does not have",
        ],
    );
    assert_eq!(exit, 0, "{v}");
    assert_eq!(
        v["reason"],
        "aborted with blocker refusal capture-not-summarized: the capture needs a privilege the run does not have"
    );
}

#[test]
fn a_plain_abort_passes_under_the_floor_and_with_nothing_open() {
    let p = queue_project();
    set_deadline(&p, 4);
    let (exit, v) = abort_out(&p, &["--reason", "out of time"]);
    assert_eq!(exit, 0, "{v}");
    assert_eq!(v["reason"], "out of time");
    // Nothing open: no queue, no growth, no measurement to run.
    let q = project("symptom");
    let (exit, v) = abort_out(&q, &["--reason", "nothing to measure here"]);
    assert_eq!(exit, 0, "{v}");
    assert_eq!(v["status"], "aborted");
}

#[test]
fn a_plain_abort_with_an_undecided_growth_is_refused_and_names_its_first_command() {
    let (p, f) = growth_project();
    let e = refused_with(&p, &[], "budget-left");
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains(&format!("1 undecided growth(s): {f}")),
        "{message}"
    );
    let fix = e["fix"].as_str().unwrap();
    assert!(fix.starts_with("dla "), "{fix}");
    assert!(
        fix.contains(
            "   or   dla run abort --blocker <user|tool|build|environment|privilege|refusal>"
        ),
        "{fix}"
    );
    let (exit, v) = abort_out(&p, &["--blocker", "user"]);
    assert_eq!(exit, 0, "{v}");
}
