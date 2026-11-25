//! A measured leak must not stay silent: a short series says so and settles nothing, the Node
//! driver ends by itself and fits its points into the time limit, a runtime detector hit carries
//! its registration site, and a registration on an object handed in from outside is eligible.
//!
//! The project is a long-lived emitter (a reused connection) and a function that, per call, adds
//! one listener to the object a callback hands it and never removes it. Each test skips,
//! printing the reason, when `node` is not installed.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn have_node() -> bool {
    let ok = Std::new("node")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipping: node is not on PATH");
    }
    ok
}

struct Out {
    text: String,
    json: Value,
}

fn dla(project: &Path, args: &[&str]) -> Out {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
        "DLA_CAPTURE",
        "DLA_CAPTURE_DIR",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1-4")
        .env("DLA_CONSENT_RAW_VALUES", "1");
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    assert!(
        o.status.success() && (json["ok"] == true || json.is_null()),
        "{args:?}: {text}\n{}",
        String::from_utf8_lossy(&o.stderr)
    );
    Out { text, json }
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// `src/pool.js:13` is the registration: `socket` is the parameter of a callback.
fn project(dir: &Path) {
    write(
        dir,
        "package.json",
        "{\"name\":\"pool\",\"version\":\"1.0.0\",\"scripts\":{\"load\":\"node load.js --iterations 400\"}}\n",
    );
    write(
        dir,
        "src/pool.js",
        "'use strict';\n\
         const { EventEmitter } = require('node:events');\n\n\
         // the reused connection of the process\n\
         const shared = new EventEmitter();\n\n\
         function connect(id) {\n\
         \x20 const req = new EventEmitter();\n\
         \x20 req.on('socket', function onSocket(socket) {\n\
         \x20   function onError(err) {\n\
         \x20     socket.removeListener('error', onError);\n\
         \x20   }\n\
         \x20   socket.on('error', onError);\n\
         \x20 });\n\
         \x20 req.emit('socket', shared);\n\
         \x20 return id;\n}\n\n\
         module.exports = { connect, shared };\n",
    );
    write(
        dir,
        "load.js",
        "const { connect } = require('./src/pool');\n\
         const n = Number.parseInt(process.argv[3] || '100', 10);\n\
         for (let i = 0; i < n; i += 1) connect(i);\n",
    );
}

const SITE: &str = "src/pool.js:13";

struct Run {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
    edit: PathBuf,
}

/// Run init, census, discover and a driver batch whose hooks are filled with `setup` and `unit`.
fn prepared(setup: &str, unit: &str) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir.path().join("project");
    project(&project_dir);
    let init = dla(&project_dir, &["run", "init", "--session-id", "t"]);
    let run_dir = project_dir
        .join(".dla")
        .join(init.json["run_id"].as_str().unwrap());
    dla(&project_dir, &["census", "--lang", "node"]);
    dla(&project_dir, &["workload", "discover", "--lang", "node"]);
    let g = dla(
        &project_dir,
        &["harness", "gen", "--lang", "node", "--driver", "--next"],
    );
    let edit = project_dir.join(g.json["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "  throw new Error('DLA-HOOK setup has not been filled in');",
            setup,
        )
        .replace(
            "  throw new Error('DLA-HOOK unit has not been filled in');",
            unit,
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    Run {
        _dir: dir,
        project: project_dir,
        run_dir,
        edit,
    }
}

const SETUP: &str = "  state.pool = projectRequire('src/pool.js');";
/// One unit in eight registers a listener: the leak is real and steady, and the heap growth it
/// leaves stays under the floor below which a heap residual demands no capture (64 KiB over the
/// series), so the trend of the short run leads with its own sentence.
const UNIT: &str = "  if (i % 8 === 0) state.pool.connect(key);";

fn site_finding(run: &Run) -> Value {
    let o = dla(&run.project, &["ledger", "list", "--all"]);
    o.text
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|r| r["site"] == SITE)
        .unwrap_or_else(|| panic!("no finding at {SITE}: {}", o.text))
}

/// S5: the registration on a callback parameter is eligible and ranks at the top.
#[test]
fn a_registration_on_an_object_from_outside_is_eligible() {
    if !have_node() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("project");
    project(&p);
    dla(&p, &["run", "init", "--session-id", "t"]);
    dla(&p, &["census", "--lang", "node"]);
    let o = dla(&p, &["ledger", "list", "--all"]);
    let row: Value = o
        .text
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|r| r["site"] == SITE)
        .unwrap_or_else(|| panic!("no finding at {SITE}: {}", o.text));
    assert_ne!(row["queue"], "not-eligible", "{row}");
    assert!(row["rank"].is_number(), "{row}");
    assert!(row["tier"].as_u64().unwrap() <= 4, "{row}");
}

/// S1, S3, S4: a 4-point run speaks first, names the detector hit with its site, and a target
/// is not settled; the default run passes the site column and ties it to the finding.
#[test]
fn a_short_run_speaks_and_the_full_run_ties_the_site_column_to_its_finding() {
    if !have_node() {
        return;
    }
    let run = prepared(SETUP, UNIT);
    let block = fs::read_to_string(&run.edit).unwrap();
    assert!(
        block.contains("count grows only when the same object is used again"),
        "the to-do line of a registration on an outside object: {block}"
    );
    let finding = site_finding(&run);
    let id = finding["id"].as_str().unwrap().to_owned();

    // A 4-point run.
    let short = dla(
        &run.project,
        &[
            "harness",
            "run",
            "H1",
            "--mode",
            "fresh",
            "--iterations",
            "4",
        ],
    );
    assert!(
        short
            .text
            .starts_with("{\"_attention\":[\"READ THE DETECTOR HIT"),
        "{}",
        &short.text[..short.text.len().min(400)]
    );
    assert!(short.text.contains("SERIES TOO SHORT"), "{}", short.text);
    let t = dla(&run.project, &["trend", "S1"]);
    let next0 = t.json["next"][0].as_str().unwrap();
    assert!(
        next0.starts_with(
            "SERIES TOO SHORT (4 of 8 points, nothing can pass): dla harness run H1 --mode fresh"
        ),
        "{next0}"
    );
    assert!(next0.contains("dla harness run H1 --mode fresh"), "{next0}");
    assert!(
        t.json["short"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["column"] == format!("node.listeners@{SITE}")),
        "{}",
        t.json["short"]
    );
    let next1 = t.json["next"][1].as_str().unwrap();
    assert!(
        next1.starts_with("DETECTOR HIT (listeners added at ") && next1.contains(SITE),
        "{next1}"
    );
    for v in t.json["targets"].as_array().unwrap() {
        assert_ne!(v["verdict"], "executed", "{v}");
    }
    assert_eq!(t.json["passed"].as_array().unwrap().len(), 0);

    // The default run.
    dla(&run.project, &["harness", "run", "H1", "--mode", "fresh"]);
    let t = dla(&run.project, &["trend", "S2"]);
    let col = format!("node.listeners@{SITE}");
    let passed = t.json["passed"].as_array().unwrap();
    let c = passed
        .iter()
        .find(|c| c["column"] == col)
        .unwrap_or_else(|| panic!("{col} did not pass: {}", t.text));
    assert!(
        c["candidates"].as_array().unwrap().iter().any(|f| *f == id),
        "{c}"
    );
    assert!(t.json.get("short").is_none(), "{}", t.text);
}

/// S2: servers a setup leaves open do not keep the process alive, and coverage data is written.
#[test]
fn a_driver_with_an_open_server_ends_by_itself_and_writes_coverage() {
    if !have_node() {
        return;
    }
    let run = prepared(
        "  state.pool = projectRequire('src/pool.js');\n  const http = require('http');\n  state.server = http.createServer((q, r) => r.end('ok'));\n  await new Promise((resolve) => state.server.listen(0, '127.0.0.1', resolve));\n  setInterval(() => {}, 1000);",
        UNIT,
    );
    let started = std::time::Instant::now();
    let out = dla(&run.project, &["harness", "run", "H1", "--mode", "fresh"]);
    assert!(
        started.elapsed().as_secs() < 60,
        "the process had to be killed"
    );
    assert!(!out.text.contains("NO COVERAGE DATA"), "{}", out.text);
    let cov = fs::read_dir(run.run_dir.join("harness/H1/cov"))
        .unwrap()
        .count();
    assert!(cov > 0, "no coverage file");
}

/// S2: with a slow unit the driver lowers the step and every point arrives; the run names it.
#[test]
fn a_slow_unit_lowers_the_step_and_all_points_arrive() {
    if !have_node() {
        return;
    }
    let run = prepared(
        SETUP,
        "  await new Promise((resolve) => setTimeout(resolve, 20));\n  state.pool.connect(key);",
    );
    // 9 points of 90 units at 20 ms would take 16 s: a limit of 12 s cannot fit them.
    let path = run.run_dir.join("harness/H1/harness.json");
    let mut m: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    m["timeout_secs"] = Value::from(12);
    fs::write(&path, serde_json::to_string_pretty(&m).unwrap()).unwrap();
    let out = dla(&run.project, &["harness", "run", "H1", "--mode", "fresh"]);
    assert_eq!(out.json["summary"]["points"], 9, "{}", out.text);
    assert!(
        out.text
            .contains("DRIVER STEP: the driver lowered its step"),
        "{}",
        out.text
    );
}

/// S2: one full round alone (90 units of 100 ms = 9 s plus the start) would use more than the
/// time left under a 20 s limit once the other rounds count; the driver looks inside round 1.
#[test]
fn a_unit_so_slow_that_one_round_exceeds_the_limit_is_fitted_inside_the_first_round() {
    if !have_node() {
        return;
    }
    let run = prepared(
        SETUP,
        "  await new Promise((resolve) => setTimeout(resolve, 100));\n  state.pool.connect(key);",
    );
    let path = run.run_dir.join("harness/H1/harness.json");
    let mut m: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    m["timeout_secs"] = Value::from(20);
    // a round of 300 units would take 30 s alone
    m["step"] = Value::from(300);
    fs::write(&path, serde_json::to_string_pretty(&m).unwrap()).unwrap();
    let out = dla(&run.project, &["harness", "run", "H1", "--mode", "fresh"]);
    assert_eq!(out.json["summary"]["points"], 9, "{}", out.text);
    assert!(
        out.text.contains("in round 1 (after 10 units of it)"),
        "{}",
        out.text
    );
}
