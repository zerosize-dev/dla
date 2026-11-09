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
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
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
        r#"{"schema_version":6,"seq":13,"ts":"2026-01-02T03:09:00Z","type":"open_item","id":"O1","text":"x","opened_by":"manual"}"#,
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
        "series",
        "trend",
        "harness-manifest",
        "harness-dump",
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
            "N-HWM",
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
    assert!(fix.contains("dla ledger test F1 --evidence"), "{fix}");
    assert!(!fix.contains("available from M3"), "{fix}");
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
        (
            vec!["ledger"],
            "dla ledger --help",
            vec!["add", "resolve", "annotate", "test", "confirm", "refute"],
        ),
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

// ---- consent ----

fn state_of(project: &Path, run_id: &str) -> Value {
    let p = project.join(".dla").join(run_id).join("state.json");
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

fn ledger_text(project: &Path, run_id: &str) -> String {
    fs::read_to_string(project.join(".dla").join(run_id).join("ledger.jsonl")).unwrap()
}

#[test]
fn consent_records_state_and_evidence() {
    let (dir, run_id) = fresh_run();
    let o = ok(
        dir.path(),
        &[
            "consent",
            "--tiers",
            "1-3",
            "--raw-values",
            "--ceiling",
            "3",
        ],
    );
    assert_eq!(o["tiers"], serde_json::json!([1, 2, 3]));
    assert_eq!(o["raw_values"], true);
    assert_eq!(o["given_by"], "user");
    assert_eq!(o["evidence"], "E2");
    let st = state_of(dir.path(), &run_id);
    assert_eq!(st["consent"]["tiers"], serde_json::json!([1, 2, 3]));
    assert_eq!(st["consent"]["given_by"], "user");
    assert!(st["consent"]["recorded_at"].is_string());
    let ledger = ledger_text(dir.path(), &run_id);
    assert!(ledger.contains("\"kind\":\"run-config\"") && ledger.contains("\"given_by\":\"user\""));
    // The record is usable as checklist evidence.
    ok(
        dir.path(),
        &[
            "ledger",
            "resolve",
            "intake.consent",
            "--status",
            "answered",
            "--evidence",
            "E2",
        ],
    );
    let s = ok(dir.path(), &["run", "status"]);
    assert_eq!(s["ok"], true);
}

#[test]
fn consent_given_by_runner_and_default_off_raw() {
    let (dir, run_id) = fresh_run();
    let o = ok(
        dir.path(),
        &["consent", "--tiers", "1,2", "--given-by", "runner"],
    );
    assert_eq!(o["given_by"], "runner");
    assert_eq!(o["raw_values"], false);
    assert_eq!(
        state_of(dir.path(), &run_id)["consent"]["given_by"],
        "runner"
    );
}

#[test]
fn consent_refuses_bad_tiers_ceiling_and_second_consent() {
    let (dir, run_id) = fresh_run();
    for bad in ["0", "5", "1-5", "x", "", "4-1"] {
        let e = refused_or_usage(dir.path(), &["consent", "--tiers", bad]);
        assert_eq!(e["code"], "invalid-value", "{bad:?}");
    }
    let e = refused(dir.path(), &["consent", "--tiers", "1-4", "--ceiling", "2"]);
    assert_eq!(e["code"], "invalid-value");
    assert!(e["message"].as_str().unwrap().contains("ceiling"));
    let e = refused(dir.path(), &["consent", "--tiers", "1", "--ceiling", "9"]);
    assert_eq!(e["code"], "invalid-value");
    // Nothing was recorded by the refusals.
    assert!(state_of(dir.path(), &run_id).get("consent").is_none());
    let before = ledger_text(dir.path(), &run_id);
    ok(dir.path(), &["consent", "--tiers", "1-2"]);
    let e = refused(dir.path(), &["consent", "--tiers", "1-4"]);
    assert_eq!(e["code"], "already-resolved");
    assert!(e["fix"].as_str().unwrap().contains("dla run init"));
    assert_eq!(
        state_of(dir.path(), &run_id)["consent"]["tiers"],
        serde_json::json!([1, 2])
    );
    assert_eq!(
        ledger_text(dir.path(), &run_id).lines().count(),
        before.lines().count() + 1
    );
}

fn refused_or_usage(project: &Path, args: &[&str]) -> Value {
    let o = run(project, args);
    assert!(o.code == 2, "{args:?}: {}", o.stdout);
    o.json["error"].clone()
}

#[test]
fn consent_without_a_run_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let e = refused(dir.path(), &["consent", "--tiers", "1"]);
    assert_eq!(e["code"], "no-run");
}

#[test]
fn consent_is_refused_on_a_finished_run() {
    let (dir, run_id) = fresh_run();
    let p = dir.path().join(".dla").join(&run_id).join("state.json");
    let text = fs::read_to_string(&p)
        .unwrap()
        .replace("\"active\"", "\"aborted\"");
    fs::write(&p, text).unwrap();
    let e = refused(dir.path(), &["consent", "--tiers", "1"]);
    assert_eq!(e["code"], "run-not-active");
}

#[test]
fn init_records_headless_consent_from_environment() {
    let dir = tempfile::tempdir().unwrap();
    let out = dla(dir.path())
        .args(["run", "init", "--session-id", "t"])
        .env("DLA_CONSENT_TIERS", "1-4")
        .env("DLA_CONSENT_RAW_VALUES", "1")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let j: Value = serde_json::from_slice(&out.stdout).unwrap();
    let run_id = j["run_id"].as_str().unwrap().to_owned();
    assert_eq!(j["consent"]["given_by"], "runner");
    assert_eq!(j["consent"]["raw_values"], true);
    let st = state_of(dir.path(), &run_id);
    assert_eq!(st["consent"]["tiers"], serde_json::json!([1, 2, 3, 4]));
    assert_eq!(st["consent"]["given_by"], "runner");
    assert_eq!(st["consent"]["raw_values"], true);
    let ledger = ledger_text(dir.path(), &run_id);
    assert_eq!(ledger.lines().count(), 2);
    assert!(ledger.contains("\"given_by\":\"runner\""));
    // A model-side consent afterwards is refused: one question per run.
    let e = refused(dir.path(), &["consent", "--tiers", "1"]);
    assert_eq!(e["code"], "already-resolved");
}

#[test]
fn init_headless_consent_defaults_raw_off_and_validates() {
    let dir = tempfile::tempdir().unwrap();
    let out = dla(dir.path())
        .args(["run", "init"])
        .env("DLA_CONSENT_TIERS", "1,2")
        .env("DLA_CONSENT_RAW_VALUES", "0")
        .output()
        .unwrap();
    let j: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(j["consent"]["raw_values"], false);
    let bad = tempfile::tempdir().unwrap();
    let out = dla(bad.path())
        .args(["run", "init"])
        .env("DLA_CONSENT_TIERS", "7")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let j: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        j["error"]["message"]
            .as_str()
            .unwrap()
            .contains("DLA_CONSENT_TIERS")
    );
    assert!(
        !bad.path().join(".dla").exists(),
        "a refused init writes nothing"
    );
}

#[test]
fn init_without_consent_variables_records_none() {
    let (dir, run_id) = fresh_run();
    assert!(state_of(dir.path(), &run_id).get("consent").is_none());
    assert_eq!(ledger_text(dir.path(), &run_id).lines().count(), 1);
}

// ---- ledger annotate

#[test]
fn annotate_records_sites_and_the_report_lists_them() {
    let (dir, run_id) = fresh_run();
    add_census(dir.path());
    let a = ok(
        dir.path(),
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "src/A.java:10",
            "--writer",
            "src/B.java:4",
            "--remover",
            "src/A.java:90",
            "--note",
            "cleared only on shutdown",
        ],
    );
    assert_eq!(
        (
            a["id"].as_str(),
            a["writers"].as_u64(),
            a["removers"].as_u64()
        ),
        (Some("F1"), Some(2), Some(1))
    );
    let ledger =
        fs::read_to_string(dir.path().join(".dla").join(&run_id).join("ledger.jsonl")).unwrap();
    assert!(
        ledger
            .lines()
            .last()
            .unwrap()
            .contains("\"type\":\"finding_annotated\"")
    );
    // The finding is still a candidate.
    let status = ok(dir.path(), &["run", "status"]);
    assert!(status.to_string().contains("candidate"));
    let md = Command::cargo_bin("dla")
        .unwrap()
        .arg("--project")
        .arg(dir.path())
        .arg("report")
        .output()
        .unwrap();
    let md = String::from_utf8(md.stdout).unwrap();
    assert!(
        md.contains("- Writers: `src/A.java:10`, `src/B.java:4`"),
        "{md}"
    );
    assert!(md.contains("- Removers: `src/A.java:90`"));
    assert!(md.contains("- Annotation: cleared only on shutdown"));
}

#[test]
fn annotate_refuses_unknown_findings_bad_sites_and_empty_annotations() {
    let (dir, _) = fresh_run();
    add_census(dir.path());
    let e = refused(
        dir.path(),
        &["ledger", "annotate", "F9", "--writer", "a.go:1"],
    );
    assert_eq!(e["code"], "unknown-finding");
    let e = refused(
        dir.path(),
        &["ledger", "annotate", "F1", "--writer", "a.go"],
    );
    assert_eq!(e["code"], "invalid-site");
    let e = refused(
        dir.path(),
        &["ledger", "annotate", "F1", "--remover", "a.go:0"],
    );
    assert_eq!(e["code"], "invalid-site");
    let e = refused(dir.path(), &["ledger", "annotate", "F1"]);
    assert_eq!(e["code"], "empty-annotation");
    let o = run(
        dir.path(),
        &["ledger", "annotate", "nonsense", "--writer", "a.go:1"],
    );
    assert_eq!(o.code, 1);
}

#[test]
fn annotate_changes_a_holder_only_with_matching_growth_evidence() {
    let (dir, run_id) = fresh_run();
    ok(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--holder",
            "container-field",
        ],
    );
    let path = dir.path().join(".dla").join(&run_id).join("ledger.jsonl");
    let seq = fs::read_to_string(&path).unwrap().lines().count() + 1;
    let column = r#"{"column":"holder:static:A.b","region":"heap","finding":"F1","shape":"steady","holder_kind":"static/global"}"#;
    append_line(
        &path,
        &format!(
            r#"{{"schema_version":5,"seq":{seq},"ts":"2026-01-02T03:09:00Z","type":"evidence","evidence_id":"E9","source":{{"repo":{{"commit":"abc"}}}},"tool":{{"name":"dla-v8-snapshot","version":"1"}},"authority":"dump","kind":"growth-series","result":{{"gate":"pass","passed":[{column}]}}}}"#
        ),
    );
    // Without evidence the change is a conflict whose fix names the evidence flag.
    let e = refused(
        dir.path(),
        &["ledger", "annotate", "F1", "--holder", "static/global"],
    );
    assert_eq!(e["code"], "annotation-conflict");
    assert!(e["fix"].as_str().unwrap().contains("--evidence"), "{e}");
    // Another holder than the evidence observed is refused.
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "annotate",
            "F1",
            "--holder",
            "thread-local",
            "--evidence",
            "E9",
        ],
    );
    assert_eq!(e["code"], "annotation-conflict");
    let a = ok(
        dir.path(),
        &[
            "ledger",
            "annotate",
            "F1",
            "--holder",
            "static/global",
            "--evidence",
            "E9",
        ],
    );
    assert_eq!(a["holder"], "static/global");
    assert_eq!(a["replaced_holder"], "container-field");
    assert_eq!(a["evidence"][0], "E9");
    // The line carries the old holder, and the ledger replays.
    let last = fs::read_to_string(&path).unwrap();
    let last = last.lines().last().unwrap();
    assert!(
        last.contains(r#""replaced_holder":"container-field""#)
            && last.contains(r#""schema_version":5"#),
        "{last}"
    );
    ok(dir.path(), &["run", "status"]);
}

// ---- stage commands

#[test]
fn stage_commands_run_on_an_empty_project() {
    let (dir, _) = fresh_run();
    for args in [
        vec!["probe"],
        vec!["sweep", "--lang", "go"],
        vec!["census", "--lang", "jvm"],
        vec!["workload", "discover"],
        vec!["workload", "discover", "--lang", "python"],
    ] {
        let o = ok(dir.path(), &args);
        assert_eq!(o["candidates"], 0, "{args:?}");
        assert!(
            o["evidence"].as_array().is_some_and(|e| e.len() == 1),
            "{args:?}"
        );
    }
    // Building and installing need tier-1 consent first.
    for args in [vec!["probe", "--build"], vec!["install", "ruff"]] {
        let e = refused(dir.path(), &args);
        assert_eq!(e["code"], "consent-required", "{args:?}");
        assert_eq!(e["fix"], "dla consent --tiers 1-4");
    }
    // --data is a global flag and must name an existing directory.
    let e = refused(
        dir.path(),
        &["--data", "/nonexistent", "sweep", "--lang", "go"],
    );
    assert_eq!(e["code"], "no-data-dir");
}

#[test]
fn stage_commands_validate_arguments() {
    let (dir, _) = fresh_run();
    for lang in ["jvm", "go", "rust", "node", "python", "native"] {
        let o = ok(dir.path(), &["sweep", "--lang", lang]);
        assert_eq!(o["lang"], lang);
    }
    for args in [
        vec!["sweep"],
        vec!["sweep", "--lang", "cobol"],
        vec!["census"],
        vec!["census", "--lang", "java"],
        vec!["install"],
        vec!["workload"],
        vec!["workload", "discover", "--lang", "x"],
    ] {
        let o = run(dir.path(), &args);
        assert_eq!(o.code, 1, "{args:?}: {}", o.stdout);
        assert_eq!(o.json["error"]["code"], "usage");
    }
    let e = refused(dir.path(), &["install", "nosuch"]);
    assert_eq!(e["code"], "unknown-backend");
    assert!(e["missing"].as_str().unwrap().contains("staticcheck"));
}

#[test]
fn stage_commands_need_a_run() {
    let dir = tempfile::tempdir().unwrap();
    let e = refused(dir.path(), &["sweep", "--lang", "go"]);
    assert_eq!(e["code"], "no-run");
}

// ---- ledger test, confirm and refute

/// Appends a version 2 evidence line (hand-written: there is no CLI that adds evidence)
/// and returns its number.
fn append_evidence(dir: &Path, run_id: &str, kind: &str, authority: &str, result: &str) -> String {
    let ledger = dir.join(".dla").join(run_id).join("ledger.jsonl");
    let text = fs::read_to_string(&ledger).unwrap();
    let seq = text.lines().count() as u64 + 1;
    let n = text.matches("\"type\":\"evidence\"").count() + 1;
    let line = format!(
        r#"{{"schema_version":2,"seq":{seq},"ts":"2026-01-02T03:05:00Z","type":"evidence","evidence_id":"E{n}","source":{{"repo":{{"commit":"abc"}}}},"tool":{{"name":"dla","version":"1"}},"authority":"{authority}","kind":"{kind}","result":{result},"coverage":{{"status":"complete"}}}}"#
    );
    append_line(&ledger, &line);
    format!("E{n}")
}

struct Dynamic {
    dir: tempfile::TempDir,
    run_id: String,
    harness: String,
    growth: String,
    leak: String,
    nonleak: String,
}

/// E1 run-config, then a harness record, a passing growth record and two discriminators.
fn dynamic_project() -> Dynamic {
    let (dir, run_id) = fresh_run();
    let p = dir.path();
    let harness = append_evidence(
        p,
        &run_id,
        "workload",
        "test",
        r#"{"harness":"H1","mode":"command"}"#,
    );
    let growth = append_evidence(
        p,
        &run_id,
        "growth-series",
        "test",
        r#"{"trend":"T1","series":"S1","axis":"n","gate":"pass","passed":[{"column":"probe:Cache.entries","region":"heap","site":"src/cache.go:9","finding":"F1","slope":12.5,"unit":"bytes","shape":"stair"}]}"#,
    );
    let leak = append_evidence(
        p,
        &run_id,
        "discriminator",
        "test",
        r#"{"test":"plateau","favors":"leak","supports":["leak"],"finding":null}"#,
    );
    let nonleak = append_evidence(
        p,
        &run_id,
        "discriminator",
        "test",
        r#"{"test":"purge","favors":"non-leak","supports":["N-ALLOC"],"finding":null}"#,
    );
    Dynamic {
        dir,
        run_id,
        harness,
        growth,
        leak,
        nonleak,
    }
}

fn add_leak_finding(p: &Path, site: Option<&str>) {
    let mut args = vec!["ledger", "add", "--origin", "census", "--evidence", "E1"];
    if let Some(s) = site {
        args.extend(["--site", s]);
    }
    ok(p, &args);
}

fn add_nonleak_sibling(p: &Path, root: &str, hypothesis: &str) -> Value {
    ok(
        p,
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--sibling-of",
            root,
            "--hypothesis",
            hypothesis,
            "--refute-if",
            "growth continues over the extra cycles",
        ],
    )
}

#[test]
fn ledger_test_refuses_each_missing_rule_and_names_the_fix() {
    let d = dynamic_project();
    let p = d.dir.path();
    add_leak_finding(p, Some("src/cache.go:9"));
    // No harness cited.
    let e = refused(p, &["ledger", "test", "F1", "--evidence", &d.growth]);
    assert_eq!(e["code"], "harness-required");
    assert!(
        e["fix"].as_str().unwrap().contains("dla harness gen"),
        "{e}"
    );
    let e = refused(p, &["ledger", "test", "F1"]);
    assert_eq!(e["code"], "harness-required");
    // A harness, but no non-leak sibling.
    let e = refused(p, &["ledger", "test", "F1", "--evidence", &d.harness]);
    assert_eq!(e["code"], "sibling-required");
    assert!(
        e["fix"]
            .as_str()
            .unwrap()
            .starts_with("dla ledger add --sibling-of F1 --origin hypothesis --hypothesis N-WARM"),
        "{e}"
    );
    // A malformed N-* hypothesis is refused on add.
    let e = refused(
        p,
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
            "N-NOPE",
            "--refute-if",
            "x",
        ],
    );
    assert_eq!(e["code"], "invalid-value");
    add_nonleak_sibling(p, "F1", "N-WARM");
    let t = ok(p, &["ledger", "test", "F1", "--evidence", &d.harness]);
    assert_eq!(t["finding"], "F1");
    assert_eq!(
        (t["from"].as_str(), t["to"].as_str()),
        (Some("candidate"), Some("under_test"))
    );
    assert_eq!(t["evidence"], serde_json::json!([d.harness]));
    assert!(t.get("warning").is_none());
}

#[test]
fn ledger_confirm_walks_through_every_leak_refusal_to_confirmed() {
    let d = dynamic_project();
    let p = d.dir.path();
    // F1 has a site but no holder.
    add_leak_finding(p, Some("src/cache.go:9"));
    add_nonleak_sibling(p, "F1", "N-WARM: the cache fills during warm-up");
    ok(p, &["ledger", "test", "F1", "--evidence", &d.harness]);

    let e = refused(p, &["ledger", "confirm", "F1"]);
    assert_eq!(e["code"], "growth-gate-failed");
    assert!(e["fix"].as_str().unwrap().starts_with("dla trend "), "{e}");
    let e = refused(p, &["ledger", "confirm", "F1", "--evidence", &d.growth]);
    assert_eq!(e["code"], "holder-required");
    assert_eq!(
        e["fix"],
        "dla ledger annotate F1 --holder <holder> [--owner <text>]"
    );
    let a = ok(
        p,
        &[
            "ledger",
            "annotate",
            "F1",
            "--holder",
            "container-field",
            "--owner",
            "Cache.entries",
        ],
    );
    assert_eq!(a["holder"], "container-field");
    let e = refused(p, &["ledger", "confirm", "F1", "--evidence", &d.growth]);
    assert_eq!(e["code"], "nonleak-refutation-required");
    assert!(
        e["fix"]
            .as_str()
            .unwrap()
            .contains("dla ledger refute F1.n1"),
        "{e}"
    );
    // Refuting the sibling needs a discriminator that favors leak, not non-leak.
    let e = refused(p, &["ledger", "refute", "F1.n1", "--evidence", &d.nonleak]);
    assert_eq!(e["code"], "discriminator-direction");
    let e = refused(p, &["ledger", "refute", "F1.n1"]);
    assert_eq!(e["code"], "discriminator-required");
    let r = ok(p, &["ledger", "refute", "F1.n1", "--evidence", &d.leak]);
    assert_eq!(r["to"], "refuted");
    let c = ok(p, &["ledger", "confirm", "F1", "--evidence", &d.growth]);
    assert_eq!(
        (c["from"].as_str(), c["to"].as_str()),
        (Some("under_test"), Some("confirmed"))
    );
    assert!(c.get("warning").is_none(), "the column is bound to F1: {c}");

    // The shape and region were derived on confirm; the report shows the growth line.
    let md = Command::cargo_bin("dla")
        .unwrap()
        .arg("--project")
        .arg(p)
        .arg("report")
        .output()
        .unwrap();
    let md = String::from_utf8(md.stdout).unwrap();
    assert!(
        md.contains(&format!(
            "- Growth: `probe:Cache.entries` 12.5 bytes/n (stair, gate pass, {})",
            d.growth
        )),
        "{md}"
    );
    assert!(
        md.contains("- Shape: stair") && md.contains("- Region: heap"),
        "{md}"
    );
    assert!(
        md.contains(&format!("Ruled out by: plateau (leak) {}", d.leak)),
        "{md}"
    );
    let _ = &d.run_id;
}

#[test]
fn ledger_confirm_refuses_growth_that_is_not_tied_to_the_finding() {
    let d = dynamic_project();
    let p = d.dir.path();
    add_leak_finding(p, Some("src/other.go:1"));
    ok(
        p,
        &["ledger", "annotate", "F1", "--holder", "static/global"],
    );
    add_nonleak_sibling(p, "F1", "N-HWM");
    ok(p, &["ledger", "test", "F1", "--evidence", &d.harness]);
    ok(p, &["ledger", "refute", "F1.n1", "--evidence", &d.leak]);
    // Passing growth, but for another holder (F7 at another site).
    let other = append_evidence(
        p,
        &d.run_id,
        "growth-series",
        "test",
        r#"{"trend":"T2","series":"S2","gate":"pass","passed":[{"column":"probe:Elsewhere","region":"heap","site":"src/else.go:3","finding":"F7","shape":"steady"}]}"#,
    );
    // Since ledger schema 4 that is a refusal, not a warning: growth of another holder
    // does not show that this one grows.
    let e = refused(p, &["ledger", "confirm", "F1", "--evidence", &other]);
    assert_eq!(e["code"], "growth-not-tied");
    let msg = e["message"].as_str().unwrap();
    assert!(msg.contains("F1") && msg.contains("probe:Elsewhere"), "{e}");
    assert!(
        e["fix"].as_str().unwrap().contains("dla ledger confirm F1"),
        "{e}"
    );
}

#[test]
fn ledger_confirm_needs_a_site() {
    let d = dynamic_project();
    let p = d.dir.path();
    add_leak_finding(p, None);
    add_nonleak_sibling(p, "F1", "N-WARM");
    ok(p, &["ledger", "test", "F1", "--evidence", &d.harness]);
    ok(p, &["ledger", "refute", "F1.n1", "--evidence", &d.leak]);
    let e = refused(p, &["ledger", "confirm", "F1", "--evidence", &d.growth]);
    assert_eq!(e["code"], "site-required");
    assert!(
        e["fix"]
            .as_str()
            .unwrap()
            .contains("dla ledger annotate F1 --site"),
        "{e}"
    );
    // The annotation sets the site once; a different site conflicts.
    let a = ok(p, &["ledger", "annotate", "F1", "--site", "src/cache.go:9"]);
    assert_eq!(a["site"], "src/cache.go:9");
    let e = refused(p, &["ledger", "annotate", "F1", "--site", "src/other.go:2"]);
    assert_eq!(e["code"], "annotation-conflict");
}

#[test]
fn a_non_leak_hypothesis_is_confirmed_after_the_leak_side_is_refuted() {
    let d = dynamic_project();
    let p = d.dir.path();
    add_leak_finding(p, Some("src/cache.go:9"));
    add_nonleak_sibling(p, "F1", "N-ALLOC");
    ok(p, &["ledger", "test", "F1.n1", "--evidence", &d.harness]);
    let e = refused(p, &["ledger", "confirm", "F1.n1", "--evidence", &d.nonleak]);
    assert_eq!(e["code"], "leak-refutation-required");
    assert!(
        e["fix"].as_str().unwrap().contains("dla ledger refute F1"),
        "{e}"
    );
    let e = refused(p, &["ledger", "refute", "F1", "--evidence", &d.leak]);
    assert_eq!(e["code"], "discriminator-direction");
    ok(p, &["ledger", "refute", "F1", "--evidence", &d.nonleak]);
    let c = ok(p, &["ledger", "confirm", "F1.n1", "--evidence", &d.nonleak]);
    assert_eq!(c["to"], "confirmed");
    let status = ok(p, &["run", "status"]);
    assert!(status.to_string().contains("confirmed"));
}

#[test]
fn annotate_conflicts_are_refused_and_the_same_value_is_accepted() {
    let (dir, _) = fresh_run();
    add_census(dir.path());
    ok(
        dir.path(),
        &[
            "ledger",
            "annotate",
            "F1",
            "--holder",
            "thread-local",
            "--region",
            "heap",
        ],
    );
    ok(
        dir.path(),
        &["ledger", "annotate", "F1", "--holder", "thread-local"],
    );
    for (flag, value) in [("--holder", "cycle-only"), ("--region", "off-heap")] {
        let e = refused(dir.path(), &["ledger", "annotate", "F1", flag, value]);
        assert_eq!(e["code"], "annotation-conflict", "{flag}");
    }
    ok(
        dir.path(),
        &["ledger", "annotate", "F1", "--owner", "Pool.slots"],
    );
    let e = refused(
        dir.path(),
        &["ledger", "annotate", "F1", "--owner", "Other.slots"],
    );
    assert_eq!(e["code"], "annotation-conflict");
    let o = run(
        dir.path(),
        &["ledger", "annotate", "F1", "--holder", "nonsense"],
    );
    assert_eq!(o.code, 1, "{}", o.stdout);
}

#[test]
fn new_ledger_lines_are_version_five_and_old_ones_still_replay() {
    let dir = fixture_project();
    // The hand-written run is entirely version 1.
    let o = ok(dir.path(), &["--run", FIXTURE_RUN, "run", "status"]);
    assert_eq!(o["run_id"], FIXTURE_RUN);
    ok(
        dir.path(),
        &[
            "--run",
            FIXTURE_RUN,
            "ledger",
            "annotate",
            "F1",
            "--note",
            "n",
        ],
    );
    let text = fs::read_to_string(fixture_ledger(&dir)).unwrap();
    let last: Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
    assert_eq!(last["schema_version"], 5);
    assert!(
        text.lines()
            .next()
            .unwrap()
            .contains("\"schema_version\":1")
    );
    ok(dir.path(), &["--run", FIXTURE_RUN, "run", "status"]);
}
