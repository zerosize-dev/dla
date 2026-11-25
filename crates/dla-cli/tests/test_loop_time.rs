//! The pytest loop ends by itself inside `DLA_COMMAND_SECS`, capture included: the loop is given
//! what is left of the command after the start-up and the capture, cuts its rounds (a capture needs
//! 3 of them for its profiles and the retention record) and says in `_attention` what it shortened.
//!
//! The tests skip, printing the reason, when pytest does not import in the `python3` of PATH.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
    time::Instant,
};

use assert_cmd::Command;
use serde_json::Value;

fn have_pytest() -> bool {
    let ok = Std::new("python3")
        .args([
            "-c",
            "import pytest, sys; sys.exit(0 if sys.version_info >= (3, 12) else 1)",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipping: pytest does not import in the python3 (3.12 or newer) on PATH");
    }
    ok
}

fn dla(project: &Path, env: &[(&str, &str)], args: &[&str]) -> (i32, Value) {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
        "DLA_CAPTURE",
        "DLA_CAPTURE_DIR",
        "DLA_TRACEMALLOC_FRAMES",
        "DLA_COMMAND_SECS",
        "DLA_HEAP_SECS",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1-4")
        .env("DLA_CONSENT_RAW_VALUES", "1");
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    (
        o.status.code().unwrap(),
        serde_json::from_str(&text).unwrap_or(Value::Null),
    )
}

fn ok(project: &Path, env: &[(&str, &str)], args: &[&str]) -> Value {
    let (code, v) = dla(project, env, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    v
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// A project whose test file of 3 tests takes 2 s in all, and that keeps one object per run.
fn prepared() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write(&project, "shop/__init__.py", "");
    write(
        &project,
        "shop/store.py",
        "KEPT = []\n\n\ndef keep(i):\n    KEPT.append(object())\n",
    );
    write(
        &project,
        "tests/test_sleepy.py",
        "import time\n\nfrom shop import store\n\n\ndef test_a():\n    store.keep(1)\n    time.sleep(0.7)\n\n\ndef test_b():\n    time.sleep(0.7)\n\n\ndef test_c():\n    time.sleep(0.6)\n",
    );
    ok(&project, &[], &["run", "init", "--session-id", "t"]);
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
        &["harness", "gen", "--lang", "python", "--pytest"][..],
    ] {
        ok(&project, &[], a);
    }
    (dir, project)
}

fn attention(v: &Value) -> Vec<String> {
    v["_attention"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_capture_run_ends_by_itself_inside_the_command_time_with_its_profiles_and_the_retention_record()
{
    if !have_pytest() {
        return;
    }
    let (_dir, project) = prepared();
    // The series first: nine rounds of the tests, no capture, a roomy command time.
    let full = ok(
        &project,
        &[("DLA_COMMAND_SECS", "120")],
        &["harness", "run", "H1", "--mode", "fresh"],
    );
    assert_eq!(full["summary"]["points"], 9, "{full}");
    assert!(
        full["summary"]["ends_by_itself"]
            .as_str()
            .unwrap()
            .contains("within 2 minutes")
            && full["summary"]["ends_by_itself"]
                .as_str()
                .unwrap()
                .contains("do not wrap it in timeout"),
        "{full}"
    );
    assert!(
        attention(&full).iter().all(|a| !a.starts_with("SHORTENED")),
        "a run that fits shortens nothing: {full}"
    );

    // The capture run: 40 s for the whole command.
    let env = [
        ("DLA_COMMAND_SECS", "40"),
        ("DLA_HEAP_SECS", "2"),
        ("DLA_TRACEMALLOC_FRAMES", "1"),
    ];
    let started = Instant::now();
    let cap = ok(
        &project,
        &env,
        &[
            "harness",
            "run",
            "H1",
            "--mode",
            "fresh",
            "--capture",
            "alloc,heap",
        ],
    );
    let secs = started.elapsed().as_secs_f64();
    assert!(secs < 40.0, "the command took {secs:.1} s: {cap}");
    let kinds = &cap["summary"]["capture"]["kinds"];
    assert!(kinds["alloc"][0].as_u64().unwrap() >= 3, "profiles: {cap}");
    assert!(
        kinds["heap"][0].as_u64().unwrap() >= 1,
        "retention record: {cap}"
    );
    let lines = attention(&cap);
    let shortened = lines
        .iter()
        .find(|a| a.starts_with("SHORTENED TO FIT 40 s"))
        .unwrap_or_else(|| panic!("{cap}"));
    assert!(
        shortened.contains("rounds 9 to 3") && shortened.len() <= 160,
        "{shortened}"
    );
    assert!(
        lines
            .iter()
            .any(|a| a.starts_with("THIS RUN WAS FOR THE CAPTURE")
                && a.contains("dla summarize heap-graph")
                && a.contains("dla summarize alloc-stack-diff")),
        "{lines:?}"
    );
    assert!(
        lines.iter().all(|a| !a.starts_with("SERIES TOO SHORT")),
        "{lines:?}"
    );
    assert!(
        cap["summary"]["ends_by_itself"]
            .as_str()
            .unwrap()
            .contains("do not wrap it in timeout"),
        "{cap}"
    );
    let captures = fs::read_to_string(
        fs::read_dir(project.join(".dla"))
            .unwrap()
            .flatten()
            .map(|e| e.path().join("harness/H1/captures.json"))
            .find(|p| p.is_file())
            .unwrap(),
    )
    .unwrap();
    assert!(captures.contains(".retain.json"), "{captures}");
    let capture_points = cap["summary"]["points"].as_u64().unwrap();

    // The command bound set explicitly holds for a plain run too, and it keeps more rounds.
    let plain = ok(
        &project,
        &[("DLA_COMMAND_SECS", "60")],
        &["harness", "run", "H1", "--mode", "fresh"],
    );
    let plain_points = plain["summary"]["points"].as_u64().unwrap();
    assert!(
        plain_points > capture_points,
        "{plain_points} rounds without a capture, {capture_points} with one: {plain}"
    );
}
