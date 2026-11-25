//! "The function ran" is not "the line that adds to the holder ran": a Rust crate, a Go module and
//! a Node package each hold a collection that a function fills only behind a state check (a
//! drain that has started, a delegate that is set). A driver whose set-up makes the check true
//! runs the function and never the line that adds: `dla trend` reads `writer_not_executed` with
//! the counts, the queue keeps the target and offers a focus harness whose block shows the
//! writer line and the lines that decide whether it runs, and `dla ledger close-rest` leaves it.
//! A driver that does not make that set-up runs the line and reads as before.
//!
//! The tests build and run real drivers: each skips, printing the reason, when its tools are not
//! installed (cargo with the llvm-tools component, go, node and git).
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn have(tool: &str, flag: &str) -> bool {
    let ok = Std::new(tool)
        .arg(flag)
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

/// True when the toolchain that builds a Rust driver has `llvm-profdata` and `llvm-cov` of its
/// own LLVM (the `llvm-tools` component).
fn have_llvm_tools() -> bool {
    let out = |args: &[&str]| {
        Std::new("rustc")
            .args(args)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default()
    };
    let sysroot = out(&["--print", "sysroot"]).trim().to_owned();
    let host = out(&["-vV"])
        .lines()
        .find_map(|l| l.strip_prefix("host: ").map(str::to_owned))
        .unwrap_or_default();
    let bin = Path::new(&sysroot)
        .join("lib/rustlib")
        .join(host)
        .join("bin");
    let ok = bin.join("llvm-profdata").is_file() && bin.join("llvm-cov").is_file();
    if !ok {
        eprintln!("skipping: the toolchain has no llvm-tools component");
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
        "DLA_WORKER",
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

struct Proj {
    _dir: tempfile::TempDir,
    p: PathBuf,
    run_dir: PathBuf,
}

impl Proj {
    /// A project written by `make`, a run, the census of `lang` and the workload discovered.
    fn new(make: fn(&Path), lang: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("project");
        make(&p);
        let init = dla(&p, &["run", "init", "--session-id", "t", "--budget", "3h"]);
        let run_dir = p.join(".dla").join(init.json["run_id"].as_str().unwrap());
        dla(&p, &["census", "--lang", lang]);
        dla(&p, &["workload", "discover", "--lang", lang]);
        Self {
            _dir: dir,
            p,
            run_dir,
        }
    }

    fn dla(&self, args: &[&str]) -> Out {
        dla(&self.p, args)
    }

    /// `dla harness gen --lang <lang> --driver --next`, the hooks filled by `fill` (the driver
    /// text in, the filled text out), and the harness run in both key modes.
    fn driver(&self, lang: &str, file: &str, fill: &dyn Fn(String) -> String) -> Out {
        let gen_out = self.dla(&["harness", "gen", "--lang", lang, "--driver", "--next"]);
        let h = gen_out.json["summary"]["harness"].as_str().unwrap();
        let edit = self
            .p
            .join(gen_out.json["summary"]["edit"].as_str().unwrap());
        assert!(edit.ends_with(file), "{edit:?}");
        let text = fill(fs::read_to_string(&edit).unwrap());
        assert!(!text.contains("has not been filled in"), "{text}");
        fs::write(&edit, text).unwrap();
        self.dla(&["harness", "run", h, "--mode", "both"]);
        gen_out
    }

    /// The verdict of finding `F1` in `dla trend <series>`.
    fn verdict(&self, series: &str) -> Value {
        let t = self.dla(&["trend", series]);
        t.json["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["finding"] == "F1")
            .unwrap_or_else(|| panic!("no verdict for F1: {}", t.text))
            .clone()
    }

    /// The row of `F1` in `dla ledger list --all`.
    fn row(&self) -> Value {
        self.dla(&["ledger", "list", "--all"])
            .text
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .find(|v| v["id"] == "F1")
            .expect("F1 is listed")
    }

    fn executed(&self, harness: &str, series: &str) -> Value {
        let path = self
            .run_dir
            .join(format!("harness/{harness}/executed-{series}.json"));
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    /// What every one of the three languages shows once the first driver read the target
    /// `writer_not_executed`: the queue keeps it and offers a focus harness, `close-rest` leaves
    /// it, and the next driver's block shows the writer line and the lines that decide.
    /// Takes the capture of `harness` and runs every summary it prints when `dla run status`
    /// lists a `residual_capture` item for it; does nothing otherwise.
    fn capture_if_the_gate_asks(&self, harness: &str) {
        let status = self.dla(&["run", "status"]);
        let asked = status.json["blocking"].as_array().is_some_and(|b| {
            b.iter().any(|i| {
                i["kind"] == "residual_capture"
                    && i["id"].as_str().is_some_and(|id| id.starts_with(harness))
            })
        });
        if !asked {
            return;
        }
        let run = self.dla(&[
            "harness",
            "run",
            harness,
            "--mode",
            "fresh",
            "--capture",
            "alloc,heap",
        ]);
        let cmds: Vec<String> = run.json["summary"]["summarize"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            !cmds.is_empty(),
            "the capture printed no summary: {}",
            run.text
        );
        for c in cmds {
            let words: Vec<&str> = c.split_whitespace().collect();
            assert_eq!(words.first(), Some(&"dla"), "{c}");
            self.dla(&words[1..]);
        }
    }

    fn queue_keeps_the_target_and_the_next_block_shows_the_lines(
        &self,
        lang: &str,
        writer_line_text: &str,
        guards: &[&str],
    ) {
        let row = self.row();
        assert_eq!(row["harnesses"]["H1"], "writer_not_executed", "{row}");
        assert!(row.get("settled_by").is_none(), "{row}");
        assert!(
            row["focus"]
                .as_str()
                .unwrap()
                .contains("gives it a harness of its own"),
            "{row}"
        );
        let status = self.dla(&["run", "status"]);
        assert_eq!(status.json["queue"][lang]["open"], true, "{}", status.text);
        assert_eq!(status.json["queue"][lang]["focus"], true, "{}", status.text);
        let rest = self.dla(&["ledger", "close-rest"]);
        assert_eq!(rest.json["closed"], 0, "{}", rest.text);
        assert_eq!(rest.json["left"][0]["id"], "F1", "{}", rest.text);
        // The heap of the scene may grow by itself past the floor on a slow machine (the drain
        // runs on a timer); the gate then asks for a capture before the next batch, and the scene
        // takes it as a model is told to.
        self.capture_if_the_gate_asks("H1");
        // The next harness is a focus harness; its block shows what did not run.
        let next = self.dla(&["harness", "gen", "--lang", lang, "--driver", "--next"]);
        assert_eq!(next.json["summary"]["purpose"], "focus", "{}", next.text);
        let attention = next.json["_attention"][0].as_str().unwrap();
        assert!(
            attention.contains("F1 read writer_not_executed in H1") && attention.len() < 160,
            "{attention}"
        );
        let edit = self.p.join(next.json["summary"]["edit"].as_str().unwrap());
        let block = fs::read_to_string(edit).unwrap();
        assert!(
            block.contains(&format!("ran 0 times in the last run: {writer_line_text}")),
            "{block}"
        );
        assert!(
            block.contains(
                "the unit must put the object in the state in which these lines let the writer run"
            ),
            "{block}"
        );
        for g in guards {
            assert!(
                block.contains(&format!("above it, line {g}")),
                "{g}: {block}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------
// Rust

fn rust_project(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"gated\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        dir,
        "src/lib.rs",
        "use std::sync::{\n    Mutex,\n    atomic::{AtomicBool, Ordering},\n};\n\nstatic PENDING: Mutex<Vec<u32>> = Mutex::new(Vec::new());\nstatic DRAINING: AtomicBool = AtomicBool::new(false);\n\n/// From now on `submit` hands values over at once and keeps nothing.\npub fn start_drain() {\n    DRAINING.store(true, Ordering::SeqCst);\n}\n\n/// Keeps the value until a drain starts.\npub fn submit(value: u32) {\n    if DRAINING.load(Ordering::SeqCst) {\n        return;\n    }\n    PENDING.lock().unwrap().push(value);\n}\n",
    );
}

/// The Rust driver with the set-up `setup` and the unit `unit`.
fn rust_fill(setup: &'static str, unit: &'static str) -> impl Fn(String) -> String {
    move |t| {
        t.replace(
            "    todo!(\"DLA-HOOK setup has not been filled in\")",
            setup,
        )
        .replace(
            "    let _ = (state, i, key);\n    todo!(\"DLA-HOOK unit has not been filled in\")",
            &format!("    let _ = (state, i);\n    {unit}"),
        )
    }
}

const START_DRAIN: &str = "    gated::start_drain();\n    State {}";
const NO_SETUP: &str = "    State {}";

fn rust_ready() -> bool {
    have("cargo", "--version") && have_llvm_tools()
}

#[test]
fn a_rust_driver_whose_setup_closes_the_guard_reads_writer_not_executed() {
    if !rust_ready() {
        return;
    }
    let r = Proj::new(rust_project, "rust");
    r.driver(
        "rust",
        "dla_harness.rs",
        &rust_fill(START_DRAIN, "gated::submit(key as u32);"),
    );
    let v = r.verdict("S1");
    assert_eq!(v["verdict"], "writer_not_executed", "{v}");
    assert_eq!(
        v["detail"],
        "function submit at src/lib.rs:15 ran; the lines that add to the holder did not: :19 0"
    );
    // The remedy: the sentence, the writer line with its text and count, the guard above it.
    let remedy: Vec<&str> = v["remedy"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        remedy[0].contains(
            "the unit must put the object in the state in which these lines let the writer run"
        ),
        "{remedy:?}"
    );
    assert_eq!(
        remedy[1],
        "src/lib.rs:19 ran 0 times in the last run: PENDING.lock().unwrap().push(value);"
    );
    assert_eq!(
        remedy[2],
        "  above it, line 16: if DRAINING.load(Ordering::SeqCst) {"
    );
    // The count is read from the coverage record, next to the functions.
    let rec = r.executed("H1", "S1");
    assert_eq!(
        rec["lines"],
        serde_json::json!([{"file": "src/lib.rs", "line": 19, "count": 0, "fresh": 0}]),
        "{rec}"
    );
    r.queue_keeps_the_target_and_the_next_block_shows_the_lines(
        "rust",
        "PENDING.lock().unwrap().push(value);",
        &["16: if DRAINING.load(Ordering::SeqCst) {"],
    );
}

#[test]
fn a_rust_driver_that_leaves_the_guard_open_runs_the_writer_and_reads_as_before() {
    if !rust_ready() {
        return;
    }
    let r = Proj::new(rust_project, "rust");
    r.driver(
        "rust",
        "dla_harness.rs",
        &rust_fill(NO_SETUP, "gated::submit(key as u32);"),
    );
    let v = r.verdict("S1");
    assert_eq!(v["verdict"], "executed", "{v}");
    assert_eq!(v["detail"], "function submit at src/lib.rs:15 was executed");
    assert!(v.get("remedy").is_none(), "{v}");
    // The line ran once per unit of the one round the coverage point makes.
    let rec = r.executed("H1", "S1");
    let line = &rec["lines"][0];
    assert_eq!(line["line"], 19, "{rec}");
    assert!(line["count"].as_u64().unwrap() >= 100, "{rec}");
    let row = r.row();
    assert_eq!(row["harnesses"]["H1"], "executed", "{row}");
    assert!(row.get("focus").is_none(), "{row}");
}

#[test]
fn two_rust_drivers_that_never_run_the_writer_settle_it_and_the_remedy_says_so() {
    if !rust_ready() {
        return;
    }
    let r = Proj::new(rust_project, "rust");
    r.driver(
        "rust",
        "dla_harness.rs",
        &rust_fill(START_DRAIN, "gated::submit(key as u32);"),
    );
    assert_eq!(r.verdict("S1")["verdict"], "writer_not_executed");
    // The focus harness with another driver that still closes the guard.
    r.driver(
        "rust",
        "dla_harness.rs",
        &rust_fill(START_DRAIN, "gated::submit(key as u32 + 1);"),
    );
    let v = r.verdict("S3");
    assert_eq!(v["verdict"], "writer_not_executed", "{v}");
    let row = r.row();
    assert_eq!(row["settled_by"], "not_reached", "{row}");
    let rest = r.dla(&["ledger", "close-rest"]);
    assert_eq!(rest.json["closed"], 1, "{}", rest.text);
    // The disposition `close-rest` wrote says the lines that add to the holder never ran.
    let ledger = fs::read_to_string(r.run_dir.join("ledger.jsonl")).unwrap();
    let transition = ledger
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|v| v["type"] == "transition" && v["finding"] == "F1")
        .expect("F1 was dispositioned");
    let remedy = transition["remedy"].as_str().unwrap();
    assert!(
        remedy.contains("its function ran but the lines that add to the holder never ran")
            && remedy.contains("puts the object in the state in which those lines run"),
        "{remedy}"
    );
    assert!(remedy.starts_with("targeted in H1 and H2"), "{remedy}");
}

// ---------------------------------------------------------------------------------------
// Go

fn go_project(dir: &Path) {
    write(dir, "go.mod", "module example.test/gated\n\ngo 1.21\n");
    write(
        dir,
        "registry/registry.go",
        "package registry\n\nimport (\n\t\"errors\"\n\t\"sync\"\n\t\"sync/atomic\"\n)\n\n// Counter is one entry of a registry.\ntype Counter struct{ name string }\n\n// Registry hands out counters; until a delegate is set it keeps each one it made.\ntype Registry struct {\n\tdelegate    atomic.Value\n\tmtx         sync.Mutex\n\tentries []*Counter\n}\n\n// SetDelegate makes later calls go to the delegate and keep nothing.\nfunc (m *Registry) SetDelegate(d func(string) (*Counter, error)) {\n\tm.delegate.Store(d)\n}\n\n// NewCounter returns a counter by name.\nfunc (m *Registry) NewCounter(name string) (*Counter, error) {\n\tif del, ok := m.delegate.Load().(func(string) (*Counter, error)); ok {\n\t\treturn del(name)\n\t}\n\tif name == \"\" {\n\t\treturn nil, errors.New(\"empty name\")\n\t}\n\tm.mtx.Lock()\n\tdefer m.mtx.Unlock()\n\tc := &Counter{name: name}\n\tm.entries = append(m.entries, c)\n\treturn c, nil\n}\n\n// NewRegistry makes a registry.\nfunc NewRegistry() *Registry { return &Registry{} }\n",
    );
}

fn go_fill(delegate: bool) -> impl Fn(String) -> String {
    move |t| {
        let set = if delegate {
            "\tdlaReg.SetDelegate(func(string) (*Counter, error) { return nil, nil })\n"
        } else {
            ""
        };
        let mut t = t
            .replace(
                "\tpanic(\"DLA-HOOK setup has not been filled in\")",
                &format!("\tdlaReg = NewRegistry()\n{set}\tdlaTrack(dlaReg)"),
            )
            .replace(
                "\tpanic(\"DLA-HOOK unit has not been filled in\")",
                "\t_, _ = dlaReg.NewCounter(\"c\" + dlastrconv.Itoa(key))",
            );
        t.push_str("\nvar dlaReg *Registry\n");
        t
    }
}

#[test]
fn a_go_driver_that_sets_the_delegate_reads_writer_not_executed() {
    if !have("go", "version") {
        return;
    }
    let r = Proj::new(go_project, "go");
    r.driver("go", "zz_dla_driver_test.go", &go_fill(true));
    let v = r.verdict("S1");
    assert_eq!(v["verdict"], "writer_not_executed", "{v}");
    assert!(
        v["detail"].as_str().unwrap().starts_with(
            "function NewCounter at registry/registry.go:25 ran; the lines that add to the holder did not: :35 0"
        ),
        "{v}"
    );
    let rec = r.executed("H1", "S1");
    assert_eq!(
        rec["lines"],
        serde_json::json!([{"file": "registry/registry.go", "line": 35, "count": 0}]),
        "{rec}"
    );
    r.queue_keeps_the_target_and_the_next_block_shows_the_lines(
        "go",
        "m.entries = append(m.entries, c)",
        &[
            "29: if name == \"\" {",
            "26: if del, ok := m.delegate.Load().(func(string) (*Counter, error)); ok {",
        ],
    );
}

#[test]
fn a_go_driver_that_leaves_the_delegate_unset_grows_the_field() {
    if !have("go", "version") {
        return;
    }
    let r = Proj::new(go_project, "go");
    r.driver("go", "zz_dla_driver_test.go", &go_fill(false));
    let v = r.verdict("S1");
    assert_eq!(v["verdict"], "grew", "{v}");
    let rec = r.executed("H1", "S1");
    assert_eq!(rec["lines"][0]["line"], 35, "{rec}");
    assert!(rec["lines"][0]["count"].as_u64().unwrap() > 0, "{rec}");
}

// ---------------------------------------------------------------------------------------
// Node

fn node_project(dir: &Path) {
    write(
        dir,
        "package.json",
        "{\"name\":\"gated\",\"version\":\"1.0.0\",\"main\":\"src/queue.js\"}\n",
    );
    write(
        dir,
        "src/queue.js",
        "'use strict';\n\nconst pending = [];\nlet draining = false;\n\nfunction startDrain() {\n  draining = true;\n}\n\nfunction submit(value) {\n  if (draining) {\n    return;\n  }\n  pending.push(value);\n}\n\nmodule.exports = { submit, startDrain };\n",
    );
    let ok = Std::new("git")
        .arg("-C")
        .arg(dir)
        .args(["init", "-q"])
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git init failed");
    let ok = Std::new("git")
        .arg("-C")
        .arg(dir)
        .args(["add", "-A"])
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git add failed");
}

fn node_fill(drain: bool) -> impl Fn(String) -> String {
    move |t| {
        let setup = if drain {
            "  state.q = projectRequire('src/queue.js');\n  state.q.startDrain();"
        } else {
            "  state.q = projectRequire('src/queue.js');"
        };
        t.replace(
            "  throw new Error('DLA-HOOK setup has not been filled in');",
            setup,
        )
        .replace(
            "  throw new Error('DLA-HOOK unit has not been filled in');",
            "  state.q.submit(key);",
        )
    }
}

#[test]
fn a_node_function_whose_writer_sits_in_a_branch_that_never_ran_reads_writer_not_executed() {
    if !have("node", "--version") || !have("git", "--version") {
        return;
    }
    let r = Proj::new(node_project, "node");
    r.driver("node", "dla-harness.cjs", &node_fill(true));
    let v = r.verdict("S1");
    assert_eq!(v["verdict"], "writer_not_executed", "{v}");
    assert!(
        v["detail"].as_str().unwrap().starts_with(
            "function submit at src/queue.js:10 ran; the lines that add to the holder did not: :14 0"
        ),
        "{v}"
    );
    let rec = r.executed("H1", "S1");
    assert_eq!(
        rec["lines"],
        serde_json::json!([{"file": "src/queue.js", "line": 14, "count": 0}]),
        "{rec}"
    );
    r.queue_keeps_the_target_and_the_next_block_shows_the_lines(
        "node",
        "pending.push(value);",
        &["11: if (draining) {"],
    );
}

#[test]
fn a_node_driver_that_never_drains_runs_the_writer() {
    if !have("node", "--version") || !have("git", "--version") {
        return;
    }
    let r = Proj::new(node_project, "node");
    r.driver("node", "dla-harness.cjs", &node_fill(false));
    let v = r.verdict("S1");
    assert_ne!(v["verdict"], "writer_not_executed", "{v}");
    let rec = r.executed("H1", "S1");
    assert!(rec["lines"][0]["count"].as_u64().unwrap() > 0, "{rec}");
}
