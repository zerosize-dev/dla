//! `dla harness run --capture heap,alloc,nonheap` on a Node harness through the binary:
//! heap snapshots, sampling heap profiles and resource creation sites, the manifest listing,
//! the printed `dla summarize` commands, the coverage rows and the size preflight.
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
    code: i32,
    json: Value,
    text: String,
}

fn dla_with(project: &Path, args: &[&str], env: &[(&str, &str)]) -> Out {
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
        "DLA_CAPTURE",
        "DLA_CAPTURE_DIR",
    ] {
        c.env_remove(var);
    }
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    assert!(
        text.len() <= dla_core::MAX_OUTPUT_BYTES,
        "output over the bound: {} bytes: {text}",
        text.len()
    );
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla_with(project, args, &[]);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    assert_eq!(o.json["ok"], true, "{}", o.text);
    o.json
}

/// A project copy of `fixture` with a run, consent for tiers 1 and 2, and discovery for node.
fn prepared(fixture: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    copy_dir(&fixtures_root().join(fixture).join("project"), &project);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "node"]);
    ok(&project, &["workload", "discover", "--lang", "node"]);
    (dir, project, run_dir)
}

fn manifest(run_dir: &Path, id: &str) -> Value {
    let p = run_dir.join("harness").join(id).join("harness.json");
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

fn listed(m: &Value, run_dir: &Path, kind: &str) -> Vec<PathBuf> {
    let mut files: Vec<(String, u64, PathBuf)> = m["captures"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|c| c["kind"] == kind)
        .map(|c| {
            let abs = run_dir.join(c["path"].as_str().unwrap());
            assert!(abs.is_file(), "{} is listed but missing", abs.display());
            assert_eq!(
                fs::metadata(&abs).unwrap().len(),
                c["bytes"].as_u64().unwrap()
            );
            assert!(c["captured_at"].as_str().unwrap().ends_with('Z'));
            (
                c["label"].as_str().unwrap().to_owned(),
                c["seq"].as_u64().unwrap(),
                abs,
            )
        })
        .collect();
    files.sort();
    files.into_iter().map(|(_, _, p)| p).collect()
}

fn ops(run_dir: &Path, id: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("harness").join(id).join("captures.json")).unwrap(),
    )
    .unwrap();
    v["summarize"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn command_mode_captures_one_set_per_selected_point_and_prints_histogram_commands() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared("node-socket-heartbeat");
    ok(&project, &["harness", "gen", "--lang", "node"]);
    let run = ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "heap,alloc,nonheap",
            "--dumps",
            "3",
        ],
    );
    let s = &run["summary"];
    let kinds = &s["capture"]["kinds"];
    assert_eq!(kinds["heap"][0], 3, "{s}");
    assert_eq!(kinds["alloc"][0], 3, "{s}");
    // Three resources.json and three scripts.json sidecars.
    assert_eq!(kinds["nonheap"][0], 6, "{s}");
    let m = manifest(&run_dir, "H1");
    let heaps = listed(&m, &run_dir, "heap");
    assert_eq!(heaps.len(), 3);
    assert!(
        heaps
            .iter()
            .all(|p| p.extension().is_some_and(|e| e == "heapsnapshot"))
    );
    assert_eq!(listed(&m, &run_dir, "alloc").len(), 3);
    // Separate processes at increasing n: a per-point heap-graph series (3 snapshots), then
    // one command per operation and format.
    let cmds = ops(&run_dir, "H1");
    let op = |c: &String| c.split_whitespace().nth(2).unwrap().to_owned();
    let found: Vec<String> = cmds.iter().map(op).collect();
    assert_eq!(
        found,
        [
            "heap-graph",
            "type-histogram-diff",
            "alloc-stack-diff",
            "non-heap"
        ],
        "{cmds:?}"
    );
    assert!(cmds[0].contains(".heapsnapshot") && !cmds[0].contains(".hprof"));
    assert!(cmds[1].contains(".heapsnapshot") && !cmds[1].contains(".hprof"));
    assert!(cmds[2].contains(".heapprofile"));
    assert!(cmds[3].contains("resources.json") && !cmds[3].contains("scripts.json"));
    let lim = fs::read_to_string(run_dir.join("harness/H1/captures.json")).unwrap();
    assert!(lim.contains("separate processes"), "{lim}");
    // Coverage rows say what ran.
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("coverage.json")).unwrap()).unwrap();
    for name in ["heap", "alloc", "nonheap"] {
        assert_eq!(
            cov["workers"]["node"]["backends"][format!("harness-{name}")]["status"],
            "ran",
            "{name}"
        );
    }
}

#[test]
fn driver_mode_snapshots_are_a_series_of_one_process() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared("node-socket-heartbeat");
    let g = ok(&project, &["harness", "gen", "--lang", "node", "--driver"]);
    let edit = project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "  throw new Error('DLA-HOOK setup has not been filled in');",
            "  const { Gateway } = projectRequire('src/gateway.js');\n  state.Socket = projectRequire('src/loopback.js').LoopbackSocket;\n  state.gateway = new Gateway({});",
        )
        .replace(
            "  throw new Error('DLA-HOOK unit has not been filled in');",
            "  const socket = new state.Socket();\n  state.gateway.accept(socket);\n  socket.clientSend({ type: 'hello', user: `device-${key}` });\n  socket.clientSend({ type: 'subscribe', topic: `room-${key % 40}` });\n  socket.clientHangUp();",
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    let run = ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--mode",
            "fresh",
            "--capture",
            "heap,alloc,nonheap",
        ],
    );
    let s = &run["summary"];
    assert!(
        s["capture"]["kinds"]["heap"][0].as_u64().unwrap() >= 3,
        "{s}"
    );
    assert!(
        s["summarize"][0]
            .as_str()
            .unwrap()
            .starts_with("dla summarize heap-graph ")
    );
    let cmds = ops(&run_dir, "H1");
    let found: Vec<String> = cmds
        .iter()
        .map(|c| c.split_whitespace().nth(2).unwrap().to_owned())
        .collect();
    for op in [
        "heap-graph",
        "type-histogram-diff",
        "alloc-stack-diff",
        "non-heap",
    ] {
        assert!(found.iter().any(|f| f == op), "{op} is missing: {found:?}");
    }
    // All snapshots are of one process: one capture/<pid> directory.
    let m = manifest(&run_dir, "H1");
    let heaps = listed(&m, &run_dir, "heap");
    let pids: std::collections::BTreeSet<&Path> = heaps.iter().filter_map(|p| p.parent()).collect();
    assert_eq!(pids.len(), 1, "{heaps:?}");
    let sizes: Vec<u64> = heaps
        .iter()
        .map(|p| fs::metadata(p).unwrap().len())
        .collect();
    assert!(sizes.windows(2).all(|w| w[0] < w[1]), "growing: {sizes:?}");
}

#[test]
fn unsupported_kinds_are_dropped_and_named() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared("node-socket-heartbeat-cleared");
    ok(&project, &["harness", "gen", "--lang", "node"]);
    let run = ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "nonheap,jfr,histogram",
            "--iterations",
            "2",
        ],
    );
    assert_eq!(run["summary"]["capture"]["kinds"]["nonheap"][0], 3);
    let lim = fs::read_to_string(run_dir.join("harness/H1/captures.json")).unwrap();
    assert!(
        lim.contains("--capture histogram,jfr is not written by node harnesses"),
        "{lim}"
    );
}

#[test]
fn a_refused_snapshot_is_a_blocked_coverage_row_not_a_failure() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared("node-socket-heartbeat-cleared");
    ok(&project, &["harness", "gen", "--lang", "node"]);
    let o = dla_with(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "heap",
            "--iterations",
            "2",
        ],
        &[("DLA_CAPTURE_MEM_BYTES", "1000")],
    );
    assert_eq!(o.code, 0, "{}", o.text);
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("coverage.json")).unwrap()).unwrap();
    assert_eq!(
        cov["workers"]["node"]["backends"]["harness-heap"]["status"],
        "blocked"
    );
    let lim = fs::read_to_string(run_dir.join("harness/H1/captures.json")).unwrap();
    assert!(
        lim.contains("preflight") && lim.contains("12x heapUsed"),
        "{lim}"
    );
}

/// A small module whose `register()` keeps every object in a module-level array and attaches a
/// listener to each: the agent reads a `WeakRef` to each emitter when it counts the listeners,
/// just before the snapshot.
fn registry_project(dir: &Path) -> PathBuf {
    let project = dir.join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("package.json"),
        r#"{"name":"registry-demo","version":"1.0.0","private":true,"main":"src/registry.js","scripts":{"test":"node --test"}}"#,
    )
    .unwrap();
    fs::write(
        project.join("src/registry.js"),
        "'use strict';\nconst { EventEmitter } = require('node:events');\nconst entries = [];\n\nfunction register(id) {\n  const entry = new EventEmitter();\n  entry.id = id;\n  entry.payload = new Array(64).fill(id);\n  entry.on('tick', () => entry.id);\n  entries.push(entry);\n  return entry;\n}\n\nmodule.exports = { register };\n",
    )
    .unwrap();
    project
}

#[test]
fn a_heap_summary_names_the_module_array_and_no_path_goes_through_the_weak_ref_job_list() {
    if !have_node() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = registry_project(dir.path());
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "node"]);
    ok(&project, &["workload", "discover", "--lang", "node"]);
    let g = ok(&project, &["harness", "gen", "--lang", "node", "--driver"]);
    let edit = project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "  throw new Error('DLA-HOOK setup has not been filled in');",
            "  state.registry = projectRequire('src/registry.js');",
        )
        .replace(
            "  throw new Error('DLA-HOOK unit has not been filled in');",
            "  state.registry.register(key);",
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    let run = ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--mode",
            "fresh",
            "--capture",
            "heap",
        ],
    );
    let cmd = run["summary"]["summarize"][0].as_str().unwrap().to_owned();
    assert!(cmd.starts_with("dla summarize heap-graph "), "{cmd}");
    let args: Vec<&str> = cmd.split_whitespace().skip(1).collect();
    ok(&project, &args);
    // The newest summary has the holders of the last snapshot.
    let mut files: Vec<PathBuf> = fs::read_dir(run_dir.join("summaries"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no summary in {}", run_dir.display());
    // The array keeps the objects: its holder counts them (when the list of the objects a
    // `WeakRef` was read from also kept them, the array dominated none of them).
    for f in &files {
        let text = fs::read_to_string(f).unwrap();
        let v: Value = serde_json::from_str(&text).unwrap();
        let held = v["holders"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["name"] == "entries" && h["kind"] == "static")
            .unwrap_or_else(|| panic!("no holder named `entries` in {}", f.display()));
        assert!(
            held["count"].as_u64().unwrap() > 1000,
            "the array keeps few objects: {held}"
        );
        for h in v["holders"].as_array().unwrap() {
            let path = h["path"].as_array().unwrap();
            assert!(
                path.iter().all(|hop| !hop["edge"]
                    .as_str()
                    .unwrap()
                    .contains("weak_refs_keep_during_job")),
                "{h}"
            );
        }
    }
}

/// Every heap snapshot a run of `registry_project` wrote, read with the snapshot reader. The
/// unit adds an emitter with a listener to a module-level array once per unit, so many live
/// emitters are tracked by the agent when each snapshot is taken.
fn registry_snapshots() -> Vec<(PathBuf, dla_parsers::node::Snapshot)> {
    let dir = tempfile::tempdir().unwrap();
    let project = registry_project(dir.path());
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "node"]);
    ok(&project, &["workload", "discover", "--lang", "node"]);
    let g = ok(&project, &["harness", "gen", "--lang", "node", "--driver"]);
    let edit = project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "  throw new Error('DLA-HOOK setup has not been filled in');",
            "  state.registry = projectRequire('src/registry.js');",
        )
        .replace(
            "  throw new Error('DLA-HOOK unit has not been filled in');",
            "  state.registry.register(key);",
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--mode",
            "fresh",
            "--capture",
            "heap",
        ],
    );
    let m = manifest(&run_dir, "H1");
    let files = listed(&m, &run_dir, "heap");
    assert!(files.len() >= 2, "{files:?}");
    files
        .into_iter()
        .map(|f| {
            let snap =
                dla_parsers::node::Snapshot::read_file(&f, &dla_parsers::node::Options::default())
                    .unwrap();
            (f, snap)
        })
        .collect()
}

#[test]
fn the_agent_takes_its_heap_snapshot_before_it_reads_any_weak_ref_of_the_job() {
    if !have_node() {
        return;
    }
    for (file, snap) in registry_snapshots() {
        // The agent reads a WeakRef to every tracked emitter (the sweep, the listener count) in
        // the same job; a snapshot taken after those reads has an edge from the strong roots,
        // named `weak_refs_keep_during_job`, to V8's list of the objects read, whose elements
        // are every one of them that is alive.
        let kept: Vec<usize> = snap
            .job_keep_targets()
            .into_iter()
            .flat_map(|list| {
                std::iter::once(list).chain(snap.edges_of(list).map(|e| snap.edge_target(e)))
            })
            .filter(|&n| snap.class_name(n) == "EventEmitter")
            .collect();
        assert!(
            kept.is_empty(),
            "{}: {} EventEmitter objects are kept by the job's WeakRef list: the snapshot was taken after a deref()",
            file.display(),
            kept.len()
        );
        // The snapshot is of the live emitters: they are there, and the module array holds them.
        let live = (0..snap.node_count())
            .filter(|&n| snap.class_name(n) == "EventEmitter")
            .count();
        assert!(
            live > 100,
            "{}: only {live} emitters in the snapshot",
            file.display()
        );
    }
}
