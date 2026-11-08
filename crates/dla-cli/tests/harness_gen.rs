//! `dla harness gen` through the binary on copies of the benchmark fixtures: the manifest
//! and the files it leaves in the run directory, per runtime.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::Value;

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/fixtures")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

fn on_path(tool: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(tool).is_file()))
}

struct Out {
    code: i32,
    json: Value,
    text: String,
}

fn dla(project: &Path, args: &[&str]) -> Out {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    assert!(
        text.len() <= dla_core::MAX_OUTPUT_BYTES,
        "{} bytes",
        text.len()
    );
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    o.json
}

/// A copy of the fixture with discovery done for `lang`; returns the run directory too.
fn prepared(name: &str, lang: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    copy_dir(&fixtures_root().join(name).join("project"), &project);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["census", "--lang", lang]);
    ok(&project, &["workload", "discover", "--lang", lang]);
    (dir, project, run_dir)
}

fn manifest(run_dir: &Path, id: &str) -> Value {
    let p = run_dir.join("harness").join(id).join("harness.json");
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

fn argv(v: &Value) -> Vec<&str> {
    v["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect()
}

fn env_of<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v["env"]
        .as_array()?
        .iter()
        .find(|p| p[0] == key)
        .and_then(|p| p[1].as_str())
}

#[test]
fn python_gen_prints_the_summary_and_writes_the_files() {
    let (_d, project, run_dir) = prepared("python-report-memo", "python");
    let out = ok(&project, &["harness", "gen", "--lang", "python"]);
    assert_eq!(out["command"], "harness gen");
    let s = &out["summary"];
    assert_eq!(s["harness"], "H1");
    assert_eq!(s["mode"], "command");
    assert_eq!(s["runtime"], "python");
    assert_eq!(s["iter_arg"], "--iterations {n}");
    assert_eq!(s["points"], 9);
    assert_eq!(s["next"], "dla harness run H1");
    assert!(s["probes"].as_u64().unwrap() >= 1);
    let m = manifest(&run_dir, "H1");
    assert_eq!(
        argv(&m["run"]),
        ["python3", "-m", "reports.load", "--iterations", "{n}"]
    );
    assert!(
        env_of(&m["run"], "PYTHONPATH")
            .unwrap()
            .ends_with("harness/H1/agent")
    );
    assert_eq!(m["agent"], "py-sitecustomize");
    let h = run_dir.join("harness/H1");
    for f in [
        "harness.json",
        "probes.json",
        "agent/sitecustomize.py",
        "work/reports/load.py",
    ] {
        assert!(h.join(f).is_file(), "{f}");
    }
    for d in ["out", "logs", "cov"] {
        assert!(h.join(d).is_dir(), "{d}");
    }
    // The evidence the ledger's under_test rule looks for.
    let ledger = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    assert!(
        ledger
            .lines()
            .any(|l| l.contains("\"workload\"") && l.contains("\"harness\":\"H1\"")),
        "no workload evidence names H1"
    );
}

#[test]
fn node_gen_resolves_the_script_and_the_preload() {
    let (_d, project, run_dir) = prepared("node-socket-heartbeat", "node");
    ok(&project, &["harness", "gen", "--lang", "node"]);
    let m = manifest(&run_dir, "H1");
    assert_eq!(argv(&m["run"]), ["node", "load.js", "--iterations", "{n}"]);
    assert!(
        env_of(&m["run"], "NODE_OPTIONS")
            .unwrap()
            .contains("dla-preload.js")
    );
    let probes: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("harness/H1/probes.json")).unwrap())
            .unwrap();
    assert!(
        probes["probes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["target"]["kind"] == "node-module" && p["target"]["names"].is_array())
    );
}

#[test]
fn go_gen_writes_the_wrapper_into_the_main_package() {
    let (_d, project, run_dir) = prepared("go-buffer-pool", "go");
    ok(
        &project,
        &[
            "harness",
            "gen",
            "--lang",
            "go",
            "--step",
            "100",
            "--iterations",
            "4",
        ],
    );
    let m = manifest(&run_dir, "H1");
    assert_eq!(
        (m["step"].as_u64(), m["iterations"].as_u64()),
        (Some(100), Some(4))
    );
    assert!(
        run_dir
            .join("harness/H1/work/cmd/invoiced/zz_dla_harness_test.go")
            .is_file()
    );
    assert_eq!(env_of(&m["run"], "DLA_ARGS"), Some(r#"["-n","{n}"]"#));
}

#[test]
fn jvm_gen_compiles_the_agent() {
    if !(on_path("javac") && on_path("jar")) {
        eprintln!("skipped: no JDK");
        return;
    }
    let (_d, project, run_dir) = prepared("jvm-audit-chain", "jvm");
    ok(&project, &["harness", "gen", "--lang", "jvm"]);
    let m = manifest(&run_dir, "H1");
    assert_eq!(argv(&m["run"]), ["sh", "./build.sh", "run", "{n}"]);
    assert!(
        env_of(&m["run"], "JAVA_TOOL_OPTIONS")
            .unwrap()
            .contains("-javaagent:")
    );
    assert!(run_dir.join("harness/H1/agent/dla-agent.jar").is_file());
}

#[test]
fn rust_and_native_gen_build_the_shim() {
    if !on_path("cc") || !cfg!(unix) {
        eprintln!("skipped: no cc");
        return;
    }
    let (_d, project, run_dir) = prepared("rust-tree-parent-links", "rust");
    ok(&project, &["harness", "gen", "--lang", "rust"]);
    let m = manifest(&run_dir, "H1");
    assert_eq!(m["build"]["argv"][1], "build");
    assert!(argv(&m["run"])[0].ends_with(&format!(
        "target/release/outline-service{}",
        std::env::consts::EXE_SUFFIX
    )));
    assert_eq!(m["agent"], "alloc-shim");
    assert!(run_dir.join("harness/H1/agent/libdlashim.so").is_file() || cfg!(target_os = "macos"));

    let (_d, project, run_dir) = prepared("native-vendor-codec", "native");
    ok(&project, &["harness", "gen", "--lang", "native"]);
    let m = manifest(&run_dir, "H1");
    assert_eq!(argv(&m["build"]), ["make"]);
    assert_eq!(argv(&m["run"]), ["./capture", "--iterations", "{n}"]);
}

#[test]
fn driver_gen_lists_the_file_and_the_hooks() {
    let (_d, project, run_dir) = prepared("python-ingest-two-leaks", "python");
    let out = ok(
        &project,
        &["harness", "gen", "--lang", "python", "--driver"],
    );
    let s = &out["summary"];
    assert_eq!(s["mode"], "driver");
    assert_eq!(s["hooks"], serde_json::json!(["setup", "unit"]));
    assert!(
        s["edit"]
            .as_str()
            .unwrap()
            .ends_with("harness/H1/driver/dla_harness.py")
    );
    assert!(run_dir.join("harness/H1/driver/dla_harness.py").is_file());
    assert_eq!(
        manifest(&run_dir, "H1")["key_modes"],
        serde_json::json!(["fresh", "repeated"])
    );
}

#[test]
fn refusals_name_the_item_and_the_fix() {
    let (_d, project, _run_dir) = prepared("python-report-memo", "python");
    let o = dla(&project, &["harness", "gen", "--lang", "go"]);
    assert_eq!(o.code, 2, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "workload-required");
    assert!(
        o.json["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("--driver")
    );
    let o = dla(
        &project,
        &[
            "harness",
            "gen",
            "--lang",
            "python",
            "--workload",
            "python:test:tests/test_reports.py",
        ],
    );
    assert_eq!(o.code, 2, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "iteration-arg-unknown");
    assert!(
        o.json["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("--iter-arg")
    );
    // A usable explicit argument turns that refusal into a harness.
    ok(
        &project,
        &[
            "harness",
            "gen",
            "--lang",
            "python",
            "--workload",
            "python:test:tests/test_reports.py",
            "--iter-arg",
            "--n {n}",
        ],
    );
}
