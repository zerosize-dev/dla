//! Integration tests for the `dla` binary.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;
use sha2::{Digest, Sha256};

const FIXTURE_RUN: &str = "20260102-030405-abcd";

fn fixture_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/handwritten-run")
}

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

struct Outcome {
    code: i32,
    stdout: String,
    json: Value,
}

fn run(project: &Path, args: &[&str]) -> Outcome {
    let out = dla(project).args(args).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.len() <= dla_core::MAX_OUTPUT_BYTES,
        "stdout is {} bytes: {stdout}",
        stdout.len()
    );
    let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    Outcome {
        code: out.status.code().unwrap(),
        stdout,
        json,
    }
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = run(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.stdout);
    assert_eq!(o.json["ok"], true);
    o.json
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let o = run(project, args);
    assert_eq!(o.code, 2, "{args:?}: {}", o.stdout);
    assert_eq!(o.json["ok"], false);
    let e = &o.json["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(
            e[k].as_str().is_some_and(|s| !s.is_empty()),
            "refusal lacks {k}: {}",
            o.stdout
        );
    }
    e.clone()
}

/// A temp project holding a copy of the hand-written run.
fn fixture_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let run_dir = dir.path().join(".dla").join(FIXTURE_RUN);
    fs::create_dir_all(&run_dir).unwrap();
    for f in ["state.json", "ledger.jsonl"] {
        fs::copy(fixture_dir().join(f), run_dir.join(f)).unwrap();
    }
    dir
}

fn fixture_ledger(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path()
        .join(".dla")
        .join(FIXTURE_RUN)
        .join("ledger.jsonl")
}

fn append_line(path: &Path, line: &str) {
    let mut text = fs::read_to_string(path).unwrap();
    text.push_str(line);
    text.push('\n');
    fs::write(path, text).unwrap();
}

fn fresh_run() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let init = ok(dir.path(), &["run", "init", "--session-id", "t"]);
    let id = init["run_id"].as_str().unwrap().to_owned();
    (dir, id)
}

#[test]
fn version_flag_prints_version() {
    let out = Command::cargo_bin("dla")
        .unwrap()
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("dla 0.1.0"));
}

#[test]
fn init_creates_run_files_and_gitignore_once() {
    let dir = tempfile::tempdir().unwrap();
    let a = ok(
        dir.path(),
        &["run", "init", "--budget", "90s", "--session-id", "s1"],
    );
    let id = a["run_id"].as_str().unwrap();
    assert_eq!(a["gitignore"], "created");
    assert_eq!(a["budget_seconds"], 90);
    let run_dir = dir.path().join(".dla").join(id);
    assert!(run_dir.join("state.json").is_file());
    let ledger = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    assert_eq!(ledger.lines().count(), 1);
    assert!(ledger.contains("\"kind\":\"run-config\""));
    assert!(ledger.contains("\"authority\":\"static\""));
    assert_eq!(
        fs::read_to_string(dir.path().join(".dla/current"))
            .unwrap()
            .trim(),
        id
    );
    let state: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["session_id"], "s1");
    assert_eq!(state["schema_version"], 1);
    assert_eq!(state["budget"]["seconds"], 90);

    // a second run does not duplicate the ignore entry
    std::thread::sleep(std::time::Duration::from_millis(5));
    let b = ok(dir.path(), &["run", "init"]);
    assert_eq!(b["gitignore"], "already-ignored");
    let gi = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert_eq!(gi.matches(".dla").count(), 1, "{gi:?}");
    assert!(gi.ends_with('\n'));
}

#[test]
fn init_recognises_existing_ignore_variants_and_appends_otherwise() {
    for existing in [".dla", ".dla/", "/.dla", "/.dla/"] {
        let dir = tempfile::tempdir().unwrap();
        let text = format!("target/\n{existing}\n");
        fs::write(dir.path().join(".gitignore"), &text).unwrap();
        let o = ok(dir.path(), &["run", "init"]);
        assert_eq!(o["gitignore"], "already-ignored", "{existing}");
        assert_eq!(
            fs::read_to_string(dir.path().join(".gitignore")).unwrap(),
            text
        );
    }
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".gitignore"), "target/").unwrap();
    let o = ok(dir.path(), &["run", "init"]);
    assert_eq!(o["gitignore"], "appended");
    assert_eq!(
        fs::read_to_string(dir.path().join(".gitignore")).unwrap(),
        "target/\n.dla/\n"
    );
}

#[test]
fn init_reads_session_and_budget_from_environment() {
    let dir = tempfile::tempdir().unwrap();
    let out = dla(dir.path())
        .args(["run", "init"])
        .env("CLAUDE_CODE_SESSION_ID", "from-env")
        .env("DLA_DEFAULT_BUDGET", "2h")
        .output()
        .unwrap();
    let j: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(j["session_id"], "from-env");
    assert_eq!(j["budget_seconds"], 7200);
    let bad = refused(dir.path(), &["run", "init", "--budget", "soon"]);
    assert_eq!(bad["code"], "invalid-value");
}

#[test]
fn init_uses_git_head_commit_when_readable() {
    let dir = tempfile::tempdir().unwrap();
    let git = dir.path().join(".git");
    fs::create_dir_all(git.join("refs/heads")).unwrap();
    fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    let hash = "0123456789abcdef0123456789abcdef01234567";
    fs::write(git.join("refs/heads/main"), format!("{hash}\n")).unwrap();
    let init = ok(dir.path(), &["run", "init"]);
    let id = init["run_id"].as_str().unwrap();
    let ledger = fs::read_to_string(dir.path().join(".dla").join(id).join("ledger.jsonl")).unwrap();
    assert!(ledger.contains(hash), "{ledger}");
}

#[test]
fn handwritten_ledger_renders_a_complete_report_and_stores_its_hash() {
    let dir = fixture_project();
    let out = dla(dir.path())
        .args(["--run", FIXTURE_RUN, "report"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.starts_with("# DLA report\n"), "{md}");
    assert!(md.contains(&format!("Run: `{FIXTURE_RUN}`")));
    assert!(md.contains("Status: COMPLETE"), "{md}");
    assert!(md.contains("### F1: **UNCLASSIFIED**"));
    assert!(md.contains("suspected_static_only(no_harness)"));
    assert!(md.contains("src/cache.py:42"));
    assert!(md.contains("## Evidence index"));

    let run_dir = dir.path().join(".dla").join(FIXTURE_RUN);
    let written = fs::read(run_dir.join("report.md")).unwrap();
    assert_eq!(written, md.as_bytes());
    let hash = hex::encode(Sha256::digest(&written));
    let state: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["report"]["sha256"], hash);
    assert_eq!(
        state["report"]["path"],
        format!(".dla/{FIXTURE_RUN}/report.md")
    );

    // rendering again is byte-identical
    let again = dla(dir.path())
        .args(["--run", FIXTURE_RUN, "report"])
        .output()
        .unwrap();
    assert_eq!(again.stdout, md.as_bytes());
}

#[test]
fn open_item_makes_the_report_incomplete() {
    let dir = fixture_project();
    append_line(
        &fixture_ledger(&dir),
        r#"{"schema_version":1,"seq":13,"ts":"2026-01-02T03:09:00Z","type":"open_item","id":"O1","text":"who owns the cache config?","opened_by":"manual"}"#,
    );
    let out = dla(dir.path())
        .args(["--run", FIXTURE_RUN, "report"])
        .output()
        .unwrap();
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.contains("Status: INCOMPLETE (1 open)"), "{md}");
    assert!(md.contains("dla ledger resolve O1"));

    let st = ok(dir.path(), &["--run", FIXTURE_RUN, "run", "status"]);
    assert_eq!(st["complete"], false);
    assert_eq!(st["counts"]["open_items"], 1);

    // resolving it through the CLI clears the header
    ok(
        dir.path(),
        &[
            "--run",
            FIXTURE_RUN,
            "ledger",
            "resolve",
            "O1",
            "--as",
            "unresolved",
            "--reason",
            "time_budget",
            "--remedy",
            "ask the owner",
        ],
    );
    let out = dla(dir.path())
        .args(["--run", FIXTURE_RUN, "report"])
        .output()
        .unwrap();
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("Status: COMPLETE")
    );
}

#[test]
fn open_item_resolution_forms() {
    let dir = fixture_project();
    let ledger = fixture_ledger(&dir);
    append_line(
        &ledger,
        r#"{"schema_version":1,"seq":13,"ts":"2026-01-02T03:09:00Z","type":"open_item","id":"O1","text":"q1","opened_by":"manual"}"#,
    );
    append_line(
        &ledger,
        r#"{"schema_version":1,"seq":14,"ts":"2026-01-02T03:09:00Z","type":"open_item","id":"O2","text":"q2","opened_by":"manual"}"#,
    );
    fn r<'a>(extra: &[&'a str]) -> Vec<&'a str> {
        let mut a = vec!["--run", FIXTURE_RUN, "ledger", "resolve"];
        a.extend_from_slice(extra);
        a
    }
    let e = refused(
        dir.path(),
        &r(&[
            "O1",
            "--as",
            "suspected_static_only",
            "--reason",
            "no_harness",
            "--remedy",
            "x",
        ]),
    );
    assert_eq!(e["code"], "unsupported-flag");
    let e = refused(dir.path(), &r(&["O1"]));
    assert_eq!(e["code"], "missing-argument");
    let e = refused(dir.path(), &r(&["O1", "--as", "unresolved"]));
    assert_eq!(e["missing"], "--reason");
    let e = refused(dir.path(), &r(&["O1", "--by", "F9"]));
    assert_eq!(e["code"], "unknown-finding");
    ok(dir.path(), &r(&["O1", "--by", "F1"]));
    let e = refused(dir.path(), &r(&["O1", "--by", "F1"]));
    assert_eq!(e["code"], "already-resolved");
    let e = refused(dir.path(), &r(&["O7", "--by", "F1"]));
    assert_eq!(e["code"], "unknown-open-item");
}

#[test]
fn finding_without_evidence_is_refused_by_add() {
    let (dir, _) = fresh_run();
    let e = refused(dir.path(), &["ledger", "add", "--origin", "census"]);
    assert_eq!(e["code"], "missing-evidence");
    assert!(e["fix"].as_str().unwrap().starts_with("dla ledger add"));
    let e = refused(
        dir.path(),
        &["ledger", "add", "--origin", "census", "--evidence", "E42"],
    );
    assert_eq!(e["code"], "unknown-evidence");
    let e = refused(
        dir.path(),
        &["ledger", "add", "--origin", "nonsense", "--evidence", "E1"],
    );
    assert_eq!(e["code"], "invalid-origin");
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--archetype",
            "A1",
        ],
    );
    assert_eq!(e["code"], "archetype-locked");
    // nothing was appended by any refusal
    let st = ok(dir.path(), &["run", "status"]);
    assert_eq!(st["counts"]["enumerated"], 0);
}

#[test]
fn finding_without_evidence_is_refused_on_replay_of_a_handwritten_ledger() {
    let dir = fixture_project();
    append_line(
        &fixture_ledger(&dir),
        r#"{"schema_version":1,"seq":13,"ts":"2026-01-02T03:09:00Z","type":"finding_added","finding":{"id":"F2","state":"candidate","origin":"census","supports":[]}}"#,
    );
    for args in [
        vec!["--run", FIXTURE_RUN, "run", "status"],
        vec!["--run", FIXTURE_RUN, "report"],
        vec![
            "--run",
            FIXTURE_RUN,
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
        ],
    ] {
        let e = refused(dir.path(), &args);
        assert_eq!(e["code"], "missing-evidence", "{args:?}");
        assert!(e["message"].as_str().unwrap().contains("line 13"), "{e}");
    }
}

#[test]
fn malformed_ledger_line_is_an_error_not_a_refusal() {
    let dir = fixture_project();
    append_line(&fixture_ledger(&dir), "{not json");
    let o = run(dir.path(), &["--run", FIXTURE_RUN, "run", "status"]);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert_eq!(o.json["ok"], false);
}

#[test]
fn checklist_answer_without_evidence_is_refused() {
    let (dir, _) = fresh_run();
    let e = refused(
        dir.path(),
        &["ledger", "resolve", "intake.scope", "--status", "answered"],
    );
    assert_eq!(e["code"], "missing-evidence");
    assert!(e["fix"].as_str().unwrap().contains("--evidence E1"));
    let e = refused(
        dir.path(),
        &["ledger", "resolve", "intake.scope", "--status", "partial"],
    );
    assert_eq!(e["code"], "missing-evidence");
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "intake.scope",
            "--status",
            "unresolved",
        ],
    );
    assert_eq!(e["code"], "reason-required");
    let e = refused(dir.path(), &["ledger", "resolve", "intake.scope"]);
    assert_eq!(e["code"], "missing-argument");
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "intake.nope",
            "--status",
            "answered",
            "--evidence",
            "E1",
        ],
    );
    assert_eq!(e["code"], "unknown-target");
    ok(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "intake.scope",
            "--status",
            "answered",
            "--evidence",
            "E1",
        ],
    );
    ok(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "intake.build",
            "--status",
            "unresolved",
            "--reason",
            "env_unavailable",
            "--remedy",
            "install a JDK",
        ],
    );
    let st = ok(dir.path(), &["run", "status"]);
    assert_eq!(st["counts"]["checklist_missing"], 6);
}

#[test]
fn resolve_findings_through_the_cli() {
    let (dir, _) = fresh_run();
    let a = ok(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "a.py:1",
            "--holder",
            "static/global",
            "--region",
            "heap",
        ],
    );
    assert_eq!(a["id"], "F1");
    assert_eq!(a["state"], "candidate");
    let id = a["record"].as_str().unwrap().to_owned();
    assert!(id.ends_with("/ledger.jsonl#L2"), "{id}");
    let sib = refused(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--sibling-of",
            "F1",
        ],
    );
    assert_eq!(sib["code"], "sibling-fields-required");
    let s = ok(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--sibling-of",
            "F1",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "flat after warmup",
        ],
    );
    assert_eq!(s["id"], "F1.n1");
    assert_eq!(s["parent"], "F1");

    let e = refused(dir.path(), &["ledger", "resolve", "F1"]);
    assert_eq!(e["code"], "missing-argument");
    let e = refused(
        dir.path(),
        &["ledger", "resolve", "F1", "--as", "unresolved"],
    );
    assert_eq!(e["code"], "reason-required");
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "F1",
            "--as",
            "unresolved",
            "--reason",
            "time_budget",
        ],
    );
    assert_eq!(e["code"], "remedy-required");
    let e = refused(
        dir.path(),
        &["ledger", "resolve", "F1", "--status", "answered"],
    );
    assert_eq!(e["code"], "unsupported-flag");
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "F9",
            "--as",
            "unresolved",
            "--reason",
            "time_budget",
            "--remedy",
            "x",
        ],
    );
    assert_eq!(e["code"], "unknown-finding");
    let r = ok(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "F1",
            "--as",
            "unresolved",
            "--reason",
            "time_budget",
            "--remedy",
            "rerun with 2h",
        ],
    );
    assert_eq!(r["state"], "unresolved(time_budget)");
    assert_eq!(r["from"], "candidate");
    // terminal-ish: unresolved -> suspected_static_only is not an edge
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "F1",
            "--as",
            "suspected_static_only",
            "--reason",
            "no_harness",
            "--remedy",
            "x",
        ],
    );
    assert_eq!(e["code"], "illegal-transition");
}

#[test]
fn newer_schema_exits_3() {
    let dir = fixture_project();
    let state_path = dir.path().join(".dla").join(FIXTURE_RUN).join("state.json");
    let text = fs::read_to_string(&state_path).unwrap();
    fs::write(
        &state_path,
        text.replace("\"schema_version\": 1", "\"schema_version\": 2"),
    )
    .unwrap();
    for args in [
        vec!["--run", FIXTURE_RUN, "run", "status"],
        vec!["--run", FIXTURE_RUN, "report"],
        vec![
            "--run",
            FIXTURE_RUN,
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
        ],
    ] {
        let o = run(dir.path(), &args);
        assert_eq!(o.code, 3, "{args:?}: {}", o.stdout);
        assert_eq!(o.json["ok"], false);
        assert!(
            o.json["error"]["message"]
                .as_str()
                .unwrap()
                .contains("newer"),
            "{}",
            o.stdout
        );
        assert!(!o.json["error"]["fix"].as_str().unwrap().is_empty());
    }
}

#[test]
fn newer_schema_in_a_ledger_line_exits_3() {
    let dir = fixture_project();
    append_line(
        &fixture_ledger(&dir),
        r#"{"schema_version":2,"seq":13,"ts":"2026-01-02T03:09:00Z","type":"open_item","id":"O1","text":"x","opened_by":"manual"}"#,
    );
    let o = run(dir.path(), &["--run", FIXTURE_RUN, "run", "status"]);
    assert_eq!(o.code, 3, "{}", o.stdout);
}

#[test]
fn schema_zero_cannot_be_migrated() {
    let dir = fixture_project();
    let state_path = dir.path().join(".dla").join(FIXTURE_RUN).join("state.json");
    let text = fs::read_to_string(&state_path).unwrap();
    fs::write(
        &state_path,
        text.replace("\"schema_version\": 1", "\"schema_version\": 0"),
    )
    .unwrap();
    let o = run(dir.path(), &["--run", FIXTURE_RUN, "run", "status"]);
    assert_eq!(o.code, 3, "{}", o.stdout);
    assert!(
        o.json["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("dla run init")
    );
}

#[test]
fn json_output_stays_bounded() {
    let (dir, _) = fresh_run();
    let long = "x".repeat(6000);
    // a huge value in a refusal message
    let o = run(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            &format!("bogus {long}"),
            "--evidence",
            "E1",
        ],
    );
    assert_eq!(o.json["ok"], false);
    assert!(o.stdout.len() <= 2048);
    // a huge value that is accepted and echoed in a later listing
    for i in 0..30 {
        let site = format!("{long}:{i}");
        let o = run(
            dir.path(),
            &[
                "ledger",
                "add",
                "--origin",
                "census",
                "--evidence",
                "E1",
                "--site",
                &site,
            ],
        );
        assert_eq!(o.code, 0, "{}", o.stdout);
    }
    let st = run(dir.path(), &["run", "status"]);
    assert_eq!(st.code, 0);
    assert_eq!(st.json["counts"]["enumerated"], 30);
    assert!(st.stdout.len() <= 2048, "{}", st.stdout.len());
    assert_eq!(st.json["truncated"], true);
}

#[test]
fn usage_errors_are_json_with_exit_1() {
    let (dir, _) = fresh_run();
    let o = run(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--holder",
            "nowhere",
        ],
    );
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert_eq!(o.json["ok"], false);
    assert_eq!(o.json["error"]["code"], "usage");
    let o = run(dir.path(), &["frobnicate"]);
    assert_eq!(o.code, 1);
}

#[test]
fn commands_without_a_run_say_how_to_start_one() {
    let dir = tempfile::tempdir().unwrap();
    let e = refused(dir.path(), &["run", "status"]);
    assert_eq!(e["code"], "no-run");
    assert_eq!(e["fix"], "dla run init");
    let e = refused(dir.path(), &["--run", "nope", "run", "status"]);
    assert_eq!(e["code"], "unknown-run");
    let e = refused(dir.path(), &["--run", "../x", "run", "status"]);
    assert_eq!(e["code"], "invalid-value");
}

#[test]
fn dla_run_environment_selects_the_run() {
    let dir = fixture_project();
    let out = dla(dir.path())
        .args(["run", "status"])
        .env("DLA_RUN", FIXTURE_RUN)
        .output()
        .unwrap();
    assert!(out.status.success());
    let j: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(j["run_id"], FIXTURE_RUN);
}

#[test]
fn end_to_end_run_reaches_complete() {
    let (dir, id) = fresh_run();
    let p = dir.path();
    let st = ok(p, &["run", "status"]);
    assert_eq!(st["counts"]["checklist_missing"], 8);
    ok(
        p,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            "E1",
            "--explanation",
            "never evicted",
            "--explanation-tag",
            "inferred",
        ],
    );
    for item in [
        "target", "scope", "symptom", "runtimes", "build", "workload", "consent", "budget",
    ] {
        ok(
            p,
            &[
                "ledger",
                "resolve",
                &format!("intake.{item}"),
                "--status",
                "answered",
                "--evidence",
                "E1",
            ],
        );
    }
    let st = ok(p, &["run", "status"]);
    assert_eq!(st["complete"], false);
    ok(
        p,
        &[
            "ledger",
            "resolve",
            "F1",
            "--as",
            "unresolved",
            "--reason",
            "no_harness",
            "--remedy",
            "write a harness",
        ],
    );
    let st = ok(p, &["run", "status"]);
    assert_eq!(st["complete"], true);
    let out = dla(p).arg("report").output().unwrap();
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.contains("Status: COMPLETE"), "{md}");
    assert!(md.contains(&id));
    let report = ok(p, &["run", "status"]);
    assert_eq!(report["report"], format!(".dla/{id}/report.md"));
}

#[test]
fn schema_writes_and_checks() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("schemas");
    let out_s = out.to_str().unwrap();
    let c = Command::cargo_bin("dla")
        .unwrap()
        .args(["schema", "--check", "--out", out_s])
        .output()
        .unwrap();
    assert_eq!(c.status.code(), Some(1));
    let j: Value = serde_json::from_slice(&c.stdout).unwrap();
    assert_eq!(j["error"]["code"], "schema-drift");

    let w = Command::cargo_bin("dla")
        .unwrap()
        .args(["schema", "--out", out_s])
        .output()
        .unwrap();
    assert!(w.status.success());
    let names = [
        "evidence-record",
        "finding",
        "ledger-line",
        "run-state",
        "coverage-manifest",
        "checklist",
    ];
    for n in names {
        let text = fs::read_to_string(out.join(format!("{n}.schema.json"))).unwrap();
        assert!(text.ends_with("}\n"));
        serde_json::from_str::<Value>(&text).unwrap();
    }
    let c = Command::cargo_bin("dla")
        .unwrap()
        .args(["schema", "--check", "--out", out_s])
        .output()
        .unwrap();
    assert!(c.status.success());

    fs::write(out.join("finding.schema.json"), "{}\n").unwrap();
    let c = Command::cargo_bin("dla")
        .unwrap()
        .args(["schema", "--check", "--out", out_s])
        .output()
        .unwrap();
    assert_eq!(c.status.code(), Some(1));
}

#[test]
fn committed_schemas_are_up_to_date() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let schemas = repo.join("schemas");
    let c = Command::cargo_bin("dla")
        .unwrap()
        .arg("schema")
        .arg("--check")
        .arg("--out")
        .arg(&schemas)
        .output()
        .unwrap();
    assert!(
        c.status.success(),
        "run `cargo run -p dla-cli -- schema --out schemas`: {}",
        String::from_utf8_lossy(&c.stdout)
    );
}

fn add_census(dir: &Path) -> Value {
    ok(
        dir,
        &["ledger", "add", "--origin", "census", "--evidence", "E1"],
    )
}

#[test]
fn add_reports_the_new_finding_for_a_sibling_of_a_non_last_finding() {
    let (dir, _) = fresh_run();
    assert_eq!(add_census(dir.path())["id"], "F1");
    assert_eq!(add_census(dir.path())["id"], "F2");
    let s = ok(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--sibling-of",
            "F1",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "flat after warmup",
        ],
    );
    assert_eq!(s["id"], "F1.n1");
    assert_eq!(s["parent"], "F1");
    assert_eq!(s["state"], "candidate");
    // The pointer is the line that actually holds F1.n1.
    let ledger = fs::read_to_string(
        dir.path()
            .join(".dla")
            .join(fresh_run_id(dir.path()))
            .join("ledger.jsonl"),
    )
    .unwrap();
    let n = s["seq"].as_u64().unwrap() as usize;
    assert!(
        ledger
            .lines()
            .nth(n - 1)
            .unwrap()
            .contains("\"id\":\"F1.n1\"")
    );
    // A second sibling of F1 continues the numbering.
    let s2 = ok(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--sibling-of",
            "F1",
            "--hypothesis",
            "N-POOL",
            "--refute-if",
            "x",
        ],
    );
    assert_eq!(s2["id"], "F1.n2");
}

fn fresh_run_id(dir: &Path) -> String {
    fs::read_to_string(dir.join(".dla/current"))
        .unwrap()
        .trim()
        .to_owned()
}

fn resolve_unresolved(dir: &Path, id: &str) -> Outcome {
    run(
        dir,
        &[
            "ledger",
            "resolve",
            id,
            "--as",
            "unresolved",
            "--reason",
            "time_budget",
            "--remedy",
            "rerun with 2h",
        ],
    )
}

#[test]
fn illegal_transition_refusals_never_repeat_the_refused_command() {
    let (dir, _) = fresh_run();
    add_census(dir.path());
    assert_eq!(resolve_unresolved(dir.path(), "F1").code, 0);
    let o = resolve_unresolved(dir.path(), "F1");
    assert_eq!(o.code, 2, "{}", o.stdout);
    let e = &o.json["error"];
    assert_eq!(e["code"], "illegal-transition");
    let fix = e["fix"].as_str().unwrap();
    assert!(!fix.contains("--as unresolved"), "{fix}");
    assert!(fix.contains("available from M3"), "{fix}");
}

#[test]
fn unknown_evidence_on_resolve_lists_existing_evidence() {
    let (dir, _) = fresh_run();
    add_census(dir.path());
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "F1",
            "--as",
            "unresolved",
            "--reason",
            "time_budget",
            "--remedy",
            "x",
            "--evidence",
            "E9",
        ],
    );
    assert_eq!(e["code"], "unknown-evidence");
    let fix = e["fix"].as_str().unwrap();
    assert!(fix.contains("E1"), "{fix}");
    assert!(!fix.contains("run status"), "{fix}");
}

/// A project whose ledger also holds E2 (static-hit), appended by hand.
fn project_with_static_hit() -> tempfile::TempDir {
    let (dir, id) = fresh_run();
    let ledger = dir.path().join(".dla").join(&id).join("ledger.jsonl");
    append_line(
        &ledger,
        r#"{"schema_version":1,"seq":2,"ts":"2026-01-02T03:05:00Z","type":"evidence","evidence_id":"E2","source":{"repo":{"commit":"abc"}},"tool":{"name":"scan","version":"1"},"authority":"static","kind":"static-hit","result":{},"coverage":{"status":"complete"}}"#,
    );
    dir
}

#[test]
fn suspected_static_only_needs_static_class_evidence_not_run_config() {
    let dir = project_with_static_hit();
    // F1 cites only E1, the run-config record (authority static, class context).
    add_census(dir.path());
    let args = |extra: &'static [&'static str]| {
        let mut v = vec![
            "ledger",
            "resolve",
            "F1",
            "--as",
            "suspected_static_only",
            "--reason",
            "not_exercised",
            "--remedy",
            "write a harness",
        ];
        v.extend_from_slice(extra);
        v
    };
    let e = refused(dir.path(), &args(&[]));
    assert_eq!(e["code"], "static-evidence-required");
    assert!(e["missing"].as_str().unwrap().contains("static-hit"));
    assert!(e["missing"].as_str().unwrap().contains("census-holder"));
    // Citing the static-hit E2 is accepted.
    let r = ok(dir.path(), &args(&["--evidence", "E2"]));
    assert_eq!(r["state"], "suspected_static_only(not_exercised)");
}

#[test]
fn replay_refuses_unknown_fields_naming_line_and_field() {
    let (dir, id) = fresh_run();
    add_census(dir.path());
    let ledger = dir.path().join(".dla").join(&id).join("ledger.jsonl");
    let text = fs::read_to_string(&ledger).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    lines[1] = lines[1].replacen("\"supports\"", "\"bogus\":1,\"supports\"", 1);
    fs::write(&ledger, lines.join("\n") + "\n").unwrap();
    let o = run(dir.path(), &["run", "status"]);
    assert_eq!(o.code, 1, "{}", o.stdout);
    let msg = o.json["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("line 2") && msg.contains("finding.bogus"),
        "{msg}"
    );
    // A duplicated key is refused as well.
    lines[1] = lines[1].replacen("\"bogus\":1,", "", 1);
    lines[1] = lines[1].replacen("\"seq\":2", "\"seq\":2,\"seq\":2", 1);
    fs::write(&ledger, lines.join("\n") + "\n").unwrap();
    let o = run(dir.path(), &["run", "status"]);
    assert_eq!(o.code, 1, "{}", o.stdout);
    assert!(
        o.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("duplicate field `seq`")
    );
}

fn dir_names(p: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(p)
        .map(|rd| {
            rd.map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[test]
fn failed_init_leaves_nothing_behind() {
    // Fresh project: nothing at all is created.
    let dir = tempfile::tempdir().unwrap();
    let long = "é".repeat(5000);
    let e = refused(dir.path(), &["run", "init", "--session-id", &long]);
    assert_eq!(e["code"], "invalid-value");
    assert!(e["message"].as_str().unwrap().contains("256"));
    assert!(
        dir_names(dir.path()).is_empty(),
        "{:?}",
        dir_names(dir.path())
    );

    // Project with an earlier run: no new directory, `current` unchanged, .gitignore kept.
    let first = ok(dir.path(), &["run", "init", "--session-id", "s"]);
    let before_dirs = dir_names(&dir.path().join(".dla"));
    let before_gi = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    let e = refused(dir.path(), &["run", "init", "--session-id", &long]);
    assert_eq!(e["code"], "invalid-value");
    // A scope path that makes the run-config record too large is refused up front too.
    let big = "p".repeat(3000);
    let e = refused(
        dir.path(),
        &["run", "init", "--scope", "module", "--scope-path", &big],
    );
    assert_eq!(e["code"], "result-too-large");
    assert!(e["fix"].as_str().unwrap().contains("--scope-path"));
    assert_eq!(dir_names(&dir.path().join(".dla")), before_dirs);
    assert_eq!(fresh_run_id(dir.path()), first["run_id"].as_str().unwrap());
    assert_eq!(
        fs::read_to_string(dir.path().join(".gitignore")).unwrap(),
        before_gi
    );
}

#[test]
fn init_accepts_a_session_id_of_exactly_256_characters() {
    let dir = tempfile::tempdir().unwrap();
    let sid = "s".repeat(256);
    ok(dir.path(), &["run", "init", "--session-id", &sid]);
    let sid = "s".repeat(257);
    refused(dir.path(), &["run", "init", "--session-id", &sid]);
}

#[test]
fn init_appends_when_a_negation_cancels_the_ignore() {
    for (initial, expected) in [
        (".dla/\n!.dla/\n", ".dla/\n!.dla/\n.dla/\n"),
        ("  .dla/\n", "  .dla/\n.dla/\n"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), initial).unwrap();
        let o = ok(dir.path(), &["run", "init"]);
        assert_eq!(o["gitignore"], "appended", "{initial:?}");
        assert_eq!(
            fs::read_to_string(dir.path().join(".gitignore")).unwrap(),
            expected
        );
    }
}

#[test]
fn usage_errors_name_the_argument_and_the_subcommand_help() {
    let (dir, _) = fresh_run();
    let o = run(dir.path(), &["ledger", "add"]);
    assert_eq!(o.code, 1);
    let e = &o.json["error"];
    assert_eq!(e["code"], "usage");
    assert!(e["message"].as_str().unwrap().contains("--origin"), "{e}");
    assert!(e["missing"].as_str().unwrap().contains("--origin"), "{e}");
    assert_eq!(e["fix"], "dla ledger add --help");

    let o = run(dir.path(), &["ledger", "resolve"]);
    let e = &o.json["error"];
    assert!(e["missing"].as_str().unwrap().contains("TARGET"), "{e}");
    assert_eq!(e["fix"], "dla ledger resolve --help");

    let o = run(dir.path(), &["run", "init", "--scope", "module"]);
    let e = &o.json["error"];
    assert!(
        e["missing"].as_str().unwrap().contains("--scope-path"),
        "{e}"
    );
    assert_eq!(e["fix"], "dla run init --help");

    let o = run(
        dir.path(),
        &["ledger", "add", "--origin", "census", "--evidence", "zz"],
    );
    let e = &o.json["error"];
    assert!(e["message"].as_str().unwrap().contains("--evidence"), "{e}");
    assert_eq!(e["fix"], "dla ledger add --help");

    for (args, fix, subs) in [
        (
            vec![],
            "dla --help",
            vec!["run", "ledger", "report", "schema"],
        ),
        (vec!["run"], "dla run --help", vec!["init", "status"]),
        (vec!["ledger"], "dla ledger --help", vec!["add", "resolve"]),
    ] {
        let o = run(dir.path(), &args);
        assert_eq!(o.code, 1);
        let e = &o.json["error"];
        assert_eq!(e["fix"], fix);
        for s in subs {
            assert!(e["message"].as_str().unwrap().contains(s), "{args:?}: {e}");
            assert!(e["missing"].as_str().unwrap().contains(s), "{args:?}: {e}");
        }
    }
}

#[test]
fn status_lists_findings_to_revisit_in_ledger_order() {
    let (dir, _) = fresh_run();
    let st = ok(dir.path(), &["run", "status"]);
    assert_eq!(st["revisit"], serde_json::json!([]));
    assert_eq!(st["revisit_total"], 0);

    add_census(dir.path());
    add_census(dir.path());
    ok(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--sibling-of",
            "F1",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "x",
        ],
    );
    // Resolve the sibling first, then F2: ledger order follows the finding_added lines,
    // so F2 comes before F1.n1 even though the sibling was resolved earlier.
    assert_eq!(resolve_unresolved(dir.path(), "F1.n1").code, 0);
    assert_eq!(resolve_unresolved(dir.path(), "F2").code, 0);
    let st = ok(dir.path(), &["run", "status"]);
    assert_eq!(st["revisit_total"], 2);
    let r = st["revisit"].as_array().unwrap();
    assert_eq!(r[0]["id"], "F2");
    assert_eq!(r[1]["id"], "F1.n1");
    for e in r {
        assert_eq!(e["state"], "unresolved");
        assert_eq!(e["reason"], "time_budget");
        assert_eq!(e["remedy"], "rerun with 2h");
    }
}

#[test]
fn status_revisit_is_bounded() {
    let (dir, _) = fresh_run();
    let remedy = "r".repeat(400);
    for i in 1..=12 {
        add_census(dir.path());
        ok(
            dir.path(),
            &[
                "ledger",
                "resolve",
                &format!("F{i}"),
                "--as",
                "unresolved",
                "--reason",
                "time_budget",
                "--remedy",
                &remedy,
            ],
        );
    }
    // `run` asserts stdout stays within the 2048-byte bound.
    let st = ok(dir.path(), &["run", "status"]);
    assert_eq!(st["revisit_total"], 12);
    let n = st["revisit"].as_array().unwrap().len();
    assert!((1..=10).contains(&n), "{n}");
    assert_eq!(st["truncated"], true);
    assert_eq!(st["revisit"][0]["id"], "F1");
}

#[test]
fn schema_check_treats_crlf_and_lf_as_equal() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("schemas");
    let out_s = out.to_str().unwrap();
    let w = Command::cargo_bin("dla")
        .unwrap()
        .args(["schema", "--out", out_s])
        .output()
        .unwrap();
    assert!(w.status.success());
    for entry in fs::read_dir(&out).unwrap() {
        let p = entry.unwrap().path();
        let text = fs::read_to_string(&p).unwrap();
        assert!(!text.contains('\r'), "writing stays LF");
        fs::write(&p, text.replace('\n', "\r\n")).unwrap();
    }
    let c = Command::cargo_bin("dla")
        .unwrap()
        .args(["schema", "--check", "--out", out_s])
        .output()
        .unwrap();
    assert!(c.status.success(), "{}", String::from_utf8_lossy(&c.stdout));
    // Real drift is still caught.
    fs::write(out.join("finding.schema.json"), "{}\r\n").unwrap();
    let c = Command::cargo_bin("dla")
        .unwrap()
        .args(["schema", "--check", "--out", out_s])
        .output()
        .unwrap();
    assert_eq!(c.status.code(), Some(1));
}
