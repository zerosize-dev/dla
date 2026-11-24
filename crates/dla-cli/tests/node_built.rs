//! A Node library written in TypeScript that is built before it runs: the harness copy carries the
//! ignored build output and a link to `node_modules`, the census reads the sources, and the sites
//! and coverage of the run are at the lines of the sources.
//!
//! The project is written by hand: `src/reg.ts` (source text only), a bundle `lib/index.js` that
//! `.gitignore` lists, and a source map that maps the lines of the bundle that matter to the lines
//! of the source. Each test skips, printing the reason, when `node` or `git` is not installed.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn have(tool: &str) -> bool {
    let ok = Std::new(tool)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipping: {tool} is not on PATH");
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

fn git(p: &Path, args: &[&str]) {
    let ok = Std::new("git")
        .arg("-C")
        .arg(p)
        .args(args)
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git {args:?} failed");
}

/// The source: the registration of a listener on an object handed in from outside is line 7.
const SOURCE: &str = "import { EventEmitter } from 'node:events';\n\
export const shared = new EventEmitter();\n\
export function connect(id: number): number {\n\
\x20 const req = new EventEmitter();\n\
\x20 req.on('socket', function onSocket(socket: EventEmitter) {\n\
\x20   function onError(): void { socket.removeListener('error', onError); }\n\
\x20   socket.on('error', onError);\n\
\x20 });\n\
\x20 req.emit('socket', shared);\n\
\x20 return id;\n\
}\n";

/// The bundle built from it: the same registration is line 9, and it uses a dependency.
const BUNDLE: &str = "\"use strict\";\n\
// built by the project's build step\n\
const { EventEmitter } = require('node:events');\n\
const dep = require('dep');\n\
function connect(id) {\n\
\x20 const req = new EventEmitter();\n\
\x20 req.on('socket', function onSocket(socket) {\n\
\x20   function onError() { socket.removeListener('error', onError); }\n\
\x20   socket.on('error', onError);\n\
\x20 });\n\
\x20 req.emit('socket', shared);\n\
\x20 return dep.pick(id);\n\
}\n\
const shared = new EventEmitter();\n\
module.exports = { connect, shared };\n";

/// A base64 VLQ number.
fn vlq(n: i64) -> String {
    const DIGITS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut v = if n < 0 { ((-n) << 1) | 1 } else { n << 1 };
    let mut out = String::new();
    loop {
        let mut d = (v & 31) as usize;
        v >>= 5;
        if v > 0 {
            d |= 32;
        }
        out.push(DIGITS[d] as char);
        if v == 0 {
            return out;
        }
    }
}

/// A source map of `lib/index.js` that maps the start of each listed generated line (1-based) to
/// the start of a line (1-based) of `../src/reg.ts`.
fn source_map(pairs: &[(usize, usize)]) -> String {
    let last = pairs.iter().map(|p| p.0).max().unwrap();
    let mut prev_src = 0i64;
    let mut lines = Vec::new();
    for g in 1..=last {
        match pairs.iter().find(|p| p.0 == g) {
            Some((_, s)) => {
                let s = *s as i64 - 1;
                lines.push(format!(
                    "{}{}{}{}",
                    vlq(0),
                    vlq(0),
                    vlq(s - prev_src),
                    vlq(0)
                ));
                prev_src = s;
            }
            None => lines.push(String::new()),
        }
    }
    serde_json::json!({
        "version": 3,
        "file": "index.js",
        "sources": ["../src/reg.ts"],
        "sourcesContent": [SOURCE],
        "names": [],
        "mappings": lines.join(";"),
    })
    .to_string()
}

/// The project in a git work tree whose `.gitignore` lists the build output and `node_modules`.
fn project(dir: &Path, with_map: bool) {
    write(dir, ".gitignore", "lib\nnode_modules\n");
    write(
        dir,
        "package.json",
        "{\"name\":\"pool\",\"version\":\"1.0.0\",\"main\":\"./lib/index.js\",\"files\":[\"lib\"]}\n",
    );
    write(dir, "src/reg.ts", SOURCE);
    let mut bundle = BUNDLE.to_owned();
    if with_map {
        bundle.push_str("//# sourceMappingURL=index.js.map\n");
        write(
            dir,
            "lib/index.js.map",
            &source_map(&[(5, 3), (7, 5), (8, 6), (9, 7), (11, 9), (12, 10)]),
        );
    }
    write(dir, "lib/index.js", &bundle);
    write(
        dir,
        "node_modules/dep/package.json",
        "{\"name\":\"dep\",\"version\":\"1.0.0\",\"main\":\"index.js\"}\n",
    );
    write(
        dir,
        "node_modules/dep/index.js",
        "exports.pick = (x) => x;\n",
    );
    git(dir, &["init", "-q"]);
    git(dir, &["add", "-A"]);
}

struct Run {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
    generated: Out,
}

/// Run init, census, discover and a driver batch whose hooks are filled in.
fn prepared(with_map: bool) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("project");
    project(&p, with_map);
    let init = dla(&p, &["run", "init", "--session-id", "t"]);
    let run_dir = p.join(".dla").join(init.json["run_id"].as_str().unwrap());
    dla(&p, &["census", "--lang", "node"]);
    dla(&p, &["workload", "discover", "--lang", "node"]);
    let generated = dla(
        &p,
        &["harness", "gen", "--lang", "node", "--driver", "--next"],
    );
    let edit = p.join(generated.json["summary"]["edit"].as_str().unwrap());
    // The driver loads the built package from the work copy with no workaround.
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "  throw new Error('DLA-HOOK setup has not been filled in');",
            "  state.pool = projectRequire('lib/index.js');",
        )
        .replace(
            "  throw new Error('DLA-HOOK unit has not been filled in');",
            "  state.pool.connect(key);",
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    Run {
        _dir: dir,
        project: p,
        run_dir,
        generated,
    }
}

/// The text of every file below `dir`.
fn records_of(dir: &Path) -> String {
    let mut out = String::new();
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.push_str(&records_of(&p));
        } else if let Ok(t) = fs::read_to_string(&p) {
            out.push_str(&t);
        }
    }
    out
}

fn findings(run: &Run) -> Vec<Value> {
    dla(&run.project, &["ledger", "list", "--all"])
        .text
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

#[test]
fn a_typescript_library_is_loaded_from_its_build_output_and_measured_at_its_sources() {
    if !have("node") || !have("git") {
        return;
    }
    let run = prepared(true);
    // The copy carries the ignored build output and a link to node_modules, and says so.
    let work = run.run_dir.join("harness/H1/work");
    assert!(work.join("lib/index.js").is_file());
    assert!(work.join("lib/index.js.map").is_file());
    assert!(
        fs::symlink_metadata(work.join("node_modules"))
            .unwrap()
            .is_symlink()
    );
    let summary = &run.generated.json["summary"];
    assert_eq!(summary["build_output_kept"], 2, "{summary}");
    assert_eq!(summary["node_modules_linked"], 1, "{summary}");
    let manifest = fs::read_to_string(run.run_dir.join("harness/H1/harness.json")).unwrap();
    assert!(manifest.contains("file(s) of build output that git ignores kept"));
    assert!(manifest.contains("node_modules director"));
    // The census reads the source, not the bundle.
    let all = findings(&run);
    assert!(
        all.iter()
            .all(|r| r["site"].as_str().is_none_or(|s| !s.starts_with("lib/"))),
        "{all:?}"
    );
    let finding = all
        .iter()
        .find(|r| r["site"] == "src/reg.ts:7")
        .unwrap_or_else(|| panic!("no finding at src/reg.ts:7: {all:?}"));
    let id = finding["id"].as_str().unwrap().to_owned();

    dla(&run.project, &["harness", "run", "H1", "--mode", "fresh"]);
    let t = dla(&run.project, &["trend", "S1"]);
    let col = "node.listeners@src/reg.ts:7";
    let c = t.json["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == col)
        .unwrap_or_else(|| panic!("{col} did not pass: {}", t.text));
    assert!(
        c["candidates"].as_array().unwrap().iter().any(|f| *f == id),
        "{c}"
    );
    let verdict = t.json["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["finding"] == id)
        .unwrap_or_else(|| panic!("no verdict for {id}: {}", t.text));
    assert_eq!(verdict["verdict"], "grew", "{verdict}");
    // The registration on the local emitter (line 5) was reached: its function ran in the bundle.
    let other = all
        .iter()
        .find(|r| r["site"] == "src/reg.ts:5")
        .and_then(|r| r["id"].as_str())
        .unwrap();
    let reached = t.json["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["finding"] == other)
        .unwrap_or_else(|| panic!("no verdict for {other}: {}", t.text));
    assert_eq!(reached["verdict"], "executed", "{reached}");
    // The built position stays in the record.
    let records = records_of(&run.run_dir.join("harness/H1/out"));
    assert!(
        records.contains("\"built_sites\":{")
            && records.contains("\"src/reg.ts:7\":\"lib/index.js:9\""),
        "{}",
        &records[records.len().saturating_sub(600)..]
    );
}

#[test]
fn without_a_source_map_the_sites_are_the_built_positions() {
    if !have("node") || !have("git") {
        return;
    }
    let run = prepared(false);
    dla(&run.project, &["harness", "run", "H1", "--mode", "fresh"]);
    let t = dla(&run.project, &["trend", "S1"]);
    let columns: Vec<&str> = t.json["passed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["column"].as_str())
        .collect();
    assert!(
        columns.contains(&"node.listeners@lib/index.js:9"),
        "{columns:?}"
    );
    assert!(
        !columns.iter().any(|c| c.contains("src/reg.ts")),
        "{columns:?}"
    );
    let records = records_of(&run.run_dir.join("harness/H1/out"));
    assert!(!records.contains("built_sites"));
}
