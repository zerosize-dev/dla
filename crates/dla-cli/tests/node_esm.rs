//! The Node harness in an ES-module package (`"type": "module"` in package.json), through the
//! binary: command mode on an ES-module entry file, driver mode with `projectImport` hooks, the
//! `.cjs` agent and driver files, the target block's note, and a CommonJS project beside them.
//!
//! Each test skips, printing the reason, when `node` is not installed.
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

fn dla(project: &Path, args: &[&str]) -> Value {
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
        o.status.success() && json["ok"] == true,
        "{args:?}: {text}\n{}",
        String::from_utf8_lossy(&o.stderr)
    );
    json
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

const STORE: &str = "export const ordersById = new Map();\n\
export const recentLimit = new Map();\n\
export class Store {\n  static audit = new Map();\n}\n\n\
export function remember(id) {\n\
\x20 ordersById.set(id, { id, lines: [id, id + 1] });\n\
\x20 recentLimit.set(id % 8, id);\n\
\x20 Store.audit.set(`a${id % 4}`, id);\n}\n";

const BUS: &str = "import { EventEmitter } from 'node:events';\n\n\
export const bus = new EventEmitter();\n\
bus.setMaxListeners(0);\n\n\
export class Watcher {\n  watch(id) {\n    bus.on('tick', () => id);\n  }\n}\n";

const INDEX: &str = "import { remember } from './store.js';\n\
import { Watcher } from './bus.js';\n\n\
const watcher = new Watcher();\n\n\
export function handle(id) {\n  remember(id);\n  watcher.watch(id);\n}\n";

const LOAD: &str = "import { handle } from './src/index.js';\n\n\
const n = Number.parseInt(process.argv[3] || process.argv[2] || '100', 10);\n\
for (let i = 0; i < n; i += 1) handle(i);\n";

/// An ES-module project (`package.json` with `"type": "module"`): a module-level `Map` per key
/// (`ordersById`), a bounded control (`recentLimit`), a static `Map` and a class that registers a
/// listener per call on a long-lived emitter (`src/bus.js:8`).
fn esm_project(dir: &Path) {
    write(
        dir,
        "package.json",
        "{\"name\":\"shop\",\"version\":\"1.0.0\",\"type\":\"module\",\"scripts\":{\"load\":\"node load.js --iterations 400\"}}\n",
    );
    write(dir, "src/store.js", STORE);
    write(dir, "src/bus.js", BUS);
    write(dir, "src/index.js", INDEX);
    write(dir, "load.js", LOAD);
}

/// The same project as CommonJS.
fn cjs_project(dir: &Path) {
    write(
        dir,
        "package.json",
        "{\"name\":\"shop\",\"version\":\"1.0.0\",\"scripts\":{\"load\":\"node load.js --iterations 400\"}}\n",
    );
    write(
        dir,
        "src/store.js",
        "const ordersById = new Map();\nconst recentLimit = new Map();\n\
         class Store {\n  static audit = new Map();\n}\n\n\
         function remember(id) {\n  ordersById.set(id, { id, lines: [id, id + 1] });\n\
         \x20 recentLimit.set(id % 8, id);\n  Store.audit.set(`a${id % 4}`, id);\n}\n\
         module.exports = { remember };\n",
    );
    write(
        dir,
        "src/bus.js",
        "const { EventEmitter } = require('node:events');\n\n\
         const bus = new EventEmitter();\nbus.setMaxListeners(0);\n\n\
         class Watcher {\n  watch(id) {\n    bus.on('tick', () => id);\n  }\n}\n\
         module.exports = { Watcher, bus };\n",
    );
    write(
        dir,
        "src/index.js",
        "const { remember } = require('./store.js');\nconst { Watcher } = require('./bus.js');\n\n\
         const watcher = new Watcher();\n\n\
         function handle(id) {\n  remember(id);\n  watcher.watch(id);\n}\n\
         module.exports = { handle };\n",
    );
    write(
        dir,
        "load.js",
        "const { handle } = require('./src/index.js');\n\n\
         const n = Number.parseInt(process.argv[3] || process.argv[2] || '100', 10);\n\
         for (let i = 0; i < n; i += 1) handle(i);\n",
    );
}

struct Prepared {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
}

fn prepared(make: fn(&Path)) -> Prepared {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    make(&project);
    let init = dla(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    dla(&project, &["census", "--lang", "node"]);
    dla(&project, &["workload", "discover", "--lang", "node"]);
    Prepared {
        _dir: dir,
        project,
        run_dir,
    }
}

/// The names of the columns the trend's gate passed.
fn passed(trend: &Value) -> Vec<String> {
    trend["passed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["column"].as_str().unwrap().to_owned())
        .collect()
}

fn fill(edit: &Path, setup: &str, unit: &str) {
    let text = fs::read_to_string(edit)
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
    fs::write(edit, text).unwrap();
}

fn grew(columns: &[String], what: &str) {
    assert!(
        columns.iter().any(|c| c == what),
        "{what} not in {columns:?}"
    );
}

/// Before: `dla-harness.js` in the package died with `require is not defined in ES module scope`
/// and the run measured nothing. The driver and agent are `.cjs`, the hooks import the ES modules,
/// the probes read the module-level `Map`s, and the listener column names the registration site.
#[test]
fn driver_mode_in_an_es_module_package_measures_the_probes_and_the_listener_site() {
    if !have_node() {
        return;
    }
    let p = prepared(esm_project);
    let g = dla(
        &p.project,
        &["harness", "gen", "--lang", "node", "--driver", "--next"],
    );
    let edit = p.project.join(g["summary"]["edit"].as_str().unwrap());
    assert!(edit.to_string_lossy().ends_with("dla-harness.cjs"));
    let hdir = p.run_dir.join("harness/H1");
    assert!(hdir.join("agent/dla-preload.cjs").is_file());
    assert!(!hdir.join("agent/dla-preload.js").exists());
    let text = fs::read_to_string(&edit).unwrap();
    assert!(
        text.contains("its .js files are ES modules") && text.contains("projectImport('<file>')"),
        "the target block says to import: {text}"
    );
    fill(
        &edit,
        "  state.app = await projectImport('src/index.js');",
        "  state.app.handle(key);",
    );
    dla(&p.project, &["harness", "run", "H1", "--mode", "fresh"]);
    let trend = dla(&p.project, &["trend", "S1"]);
    let cols = passed(&trend);
    grew(&cols, "probe:src/store.js:ordersById");
    grew(&cols, "node.listeners@src/bus.js:8");
    assert!(
        !cols.iter().any(|c| c.contains("recentLimit.size")),
        "the bounded control is constant: {cols:?}"
    );
}

/// Command mode (`--require` on the project's own command) on an ES-module entry file.
#[test]
fn command_mode_on_an_es_module_entry_file_measures_the_probes_and_the_listener_site() {
    if !have_node() {
        return;
    }
    let p = prepared(esm_project);
    dla(&p.project, &["harness", "gen", "--lang", "node"]);
    dla(&p.project, &["harness", "run", "H1"]);
    let trend = dla(&p.project, &["trend", "S1"]);
    let cols = passed(&trend);
    grew(&cols, "probe:src/store.js:ordersById");
    grew(&cols, "node.listeners@src/bus.js:8");
    assert!(
        !cols.iter().any(|c| c.contains("recentLimit.size")),
        "{cols:?}"
    );
}

/// A CommonJS package: the same files with `.cjs` names, `projectRequire` hooks, and no note
/// about ES modules.
#[test]
fn driver_mode_in_a_commonjs_package_is_unchanged() {
    if !have_node() {
        return;
    }
    let p = prepared(cjs_project);
    let g = dla(
        &p.project,
        &["harness", "gen", "--lang", "node", "--driver", "--next"],
    );
    let edit = p.project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit).unwrap();
    assert!(!text.contains("its .js files are ES modules"), "{text}");
    fill(
        &edit,
        "  state.app = projectRequire('src/index.js');",
        "  state.app.handle(key);",
    );
    dla(&p.project, &["harness", "run", "H1", "--mode", "fresh"]);
    let trend = dla(&p.project, &["trend", "S1"]);
    let cols = passed(&trend);
    grew(&cols, "probe:src/store.js:ordersById");
    grew(&cols, "node.listeners@src/bus.js:8");
}
