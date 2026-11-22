//! The pytest loop through the binary: `dla harness gen --lang python --pytest` as the first step of
//! a Python queue, its refusals, and what the trend and the ledger say about a line that grows in it.
//!
//! The project keeps one list per call of `shop/store.py:pin` without any container (a reference
//! taken with `Py_IncRef` through ctypes, standing in for a C extension that never releases it) and
//! has a registry the tests never call. The tests that run the loop skip, printing the reason, when
//! pytest does not import in the `python3` of PATH.
#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
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

struct Out {
    code: i32,
    text: String,
    json: Value,
}

/// `dla` in `project`; `path` replaces PATH when given.
fn dla_with(project: &Path, path: Option<&str>, args: &[&str]) -> Out {
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
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1-4")
        .env("DLA_CONSENT_RAW_VALUES", "1");
    if let Some(p) = path {
        c.env("PATH", p);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

fn dla(project: &Path, args: &[&str]) -> Out {
    dla_with(project, None, args)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    o.json
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

const PIN: &str = "import ctypes\n\n\ndef pin(i):\n    obj = [i]\n    ctypes.pythonapi.Py_IncRef(ctypes.py_object(obj))\n";
const REG: &str = "class Registry:\n    _by_key = {}\n\n    def put(self, key, value):\n        Registry._by_key[key] = value\n";
const SITE: &str = "shop/store.py:5";
const COLUMN: &str = "py.blocks@shop/store.py:5";

const LOOP_FIRST: &str = "dla harness gen --lang python --pytest";
const DRIVER: &str = "dla harness gen --lang python --driver --next";

fn project(dir: &Path, with_tests: bool) {
    write(dir, "shop/__init__.py", "");
    write(dir, "shop/store.py", PIN);
    write(dir, "shop/reg.py", REG);
    if with_tests {
        write(
            dir,
            "tests/test_pin.py",
            "from shop import store\n\n\ndef test_pin():\n    store.pin(1)\n",
        );
        write(dir, "tests/test_ok.py", "def test_ok():\n    assert True\n");
    }
}

/// `run init`, sweep, census and workload discovery.
fn prepared(with_tests: bool) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir.path().join("project");
    project(&project_dir, with_tests);
    let init = ok(&project_dir, &["run", "init", "--session-id", "t"]);
    assert!(init["run_id"].is_string(), "{init}");
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
    ] {
        ok(&project_dir, a);
    }
    (dir, project_dir)
}

/// The command of the python queue in `dla run status`.
fn queue_command(project: &Path) -> String {
    let s = ok(project, &["run", "status"]);
    s["queue"]["python"]["command"]
        .as_str()
        .unwrap_or_else(|| panic!("no python queue: {s}"))
        .to_owned()
}

#[test]
fn the_loop_is_the_first_command_of_a_python_queue_with_test_files_and_not_otherwise() {
    let (_d, p) = prepared(true);
    assert_eq!(queue_command(&p), LOOP_FIRST);
    // The gate's queue item names it too, with the driver batch after it.
    let gate = dla(&p, &["gate", "check", "--final"]);
    let items = gate.json["missing"].as_array().cloned().unwrap_or_default();
    let queue = items
        .iter()
        .find(|i| i["kind"] == "queue")
        .unwrap_or_else(|| panic!("no queue item: {}", gate.text));
    let fix = queue["fix"].as_str().unwrap();
    assert!(
        fix.starts_with("dla harness gen --lang python --pytest, dla harness run <H>"),
        "{fix}"
    );
    assert!(fix.contains(DRIVER), "{fix}");
    // `--driver --next` is not refused before it.
    let g = dla(
        &p,
        &["harness", "gen", "--lang", "python", "--driver", "--next"],
    );
    assert_eq!(g.code, 0, "{}", g.text);
    // A project without test files never shows it.
    let (_d2, p2) = prepared(false);
    assert_eq!(queue_command(&p2), DRIVER);
    let g = dla(&p2, &["harness", "gen", "--lang", "python", "--pytest"]);
    assert_ne!(g.code, 0);
    assert_eq!(g.json["error"]["code"], "workload-required", "{}", g.text);
    assert!(
        g.json["error"]["fix"].as_str().unwrap().contains(DRIVER),
        "{}",
        g.text
    );
}

#[test]
fn a_closed_queue_never_shows_the_loop() {
    let (_d, p) = prepared(true);
    assert_eq!(queue_command(&p), LOOP_FIRST);
    let r = dla(
        &p,
        &[
            "ledger",
            "resolve",
            "intake.workload",
            "--status",
            "unresolved",
            "--reason",
            "env_unavailable",
            "--remedy",
            "x",
            "--runtime",
            "python",
        ],
    );
    assert_eq!(r.code, 0, "{}", r.text);
    let s = ok(&p, &["run", "status"]);
    let q = &s["queue"]["python"];
    assert_eq!(q["open"], false, "{s}");
    assert!(!q["command"].as_str().unwrap().contains("--pytest"), "{q}");
    // The final gate's text and the status of the closed queue name no loop either.
    let gate = dla(&p, &["gate", "check", "--final"]);
    assert!(
        !gate.text.contains("--lang python --pytest"),
        "{}",
        gate.text
    );
}

#[test]
fn another_language_and_a_node_id_are_refused_through_the_binary() {
    let (_d, p) = prepared(true);
    let o = dla(&p, &["harness", "gen", "--lang", "jvm", "--pytest"]);
    assert_ne!(o.code, 0);
    assert!(
        o.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("--pytest is for Python projects, not jvm"),
        "{}",
        o.text
    );
    let o = dla(
        &p,
        &[
            "harness",
            "gen",
            "--lang",
            "python",
            "--pytest",
            "--tests",
            "tests/test_pin.py::test_pin",
        ],
    );
    assert_ne!(o.code, 0);
    assert!(
        o.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("node ids"),
        "{}",
        o.text
    );
}

/// A `python3` first on PATH that fails `import pytest` and runs the real one otherwise.
fn path_without_pytest(dir: &Path) -> String {
    let real = String::from_utf8(
        Std::new("sh")
            .args(["-c", "command -v python3"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    let shim = dir.join("python3");
    fs::write(
        &shim,
        format!(
            "#!/bin/sh\ncase \"$*\" in *\"import pytest\"*) exit 1;; esac\nexec {real} \"$@\"\n"
        ),
    )
    .unwrap();
    fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).unwrap();
    format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

#[test]
fn without_pytest_the_loop_is_refused_with_the_install_text_and_the_queue_goes_on() {
    let (_d, p) = prepared(true);
    let shim = tempfile::tempdir().unwrap();
    let path = path_without_pytest(shim.path());
    let o = dla_with(
        &p,
        Some(&path),
        &["harness", "gen", "--lang", "python", "--pytest"],
    );
    assert_ne!(o.code, 0);
    let e = &o.json["error"];
    assert_eq!(e["code"], "tool-missing", "{}", o.text);
    let message = e["message"].as_str().unwrap();
    assert!(
        message.contains("pytest does not import in the harness interpreter (python3 at "),
        "{message}"
    );
    let fix = e["fix"].as_str().unwrap();
    for want in [
        "python3 -m pip install pytest",
        "virtual environment",
        "bin directory first on PATH",
        DRIVER,
    ] {
        assert!(fix.contains(want), "{want} missing from {fix}");
    }
    // A failed attempt of the loop: the queue goes on with the driver batches, with PATH as it is.
    assert_eq!(queue_command(&p), DRIVER);
    let gate = dla(&p, &["gate", "check", "--final"]);
    assert!(
        !gate.text.contains("--lang python --pytest"),
        "{}",
        gate.text
    );
}

#[test]
fn the_loop_runs_and_the_site_column_gets_its_finding_and_a_small_driver_of_its_own() {
    if !have_pytest() {
        return;
    }
    let (_d, p) = prepared(true);
    assert_eq!(queue_command(&p), LOOP_FIRST);
    let g = ok(&p, &["harness", "gen", "--lang", "python", "--pytest"]);
    let h = g["summary"]["harness"].as_str().unwrap().to_owned();
    assert_eq!(g["summary"]["next"], format!("dla harness run {h}"));
    assert_eq!(g["summary"]["hooks"], serde_json::json!([]));
    // Generated, not run: the queue says to run it.
    assert_eq!(queue_command(&p), format!("dla harness run {h}"));
    let r = ok(&p, &["harness", "run", &h]);
    let series = r["summary"]["series"][0].as_str().unwrap().to_owned();
    // It wrote a series: the queue goes on with the driver batch, as without the loop.
    assert_eq!(queue_command(&p), DRIVER);
    let t = ok(&p, &["trend", &series]);
    let passed: Vec<&str> = t["passed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["column"].as_str())
        .collect();
    assert!(passed.contains(&COLUMN), "{passed:?}");
    let next: Vec<&str> = t["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let add = next
        .iter()
        .find(|n| n.starts_with("dla ledger add --origin growth-series"))
        .unwrap_or_else(|| panic!("no add command: {next:?}"));
    assert!(add.contains(&format!("--site {SITE}")), "{add}");
    // The finding at the line, as the command says; then the trend names the way to decide it.
    let evidence = add
        .split("--evidence ")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .to_owned();
    let added = ok(
        &p,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &evidence,
            "--site",
            SITE,
            "--region",
            "heap",
            "--holder",
            "native-handle/global-ref",
        ],
    );
    let f = added["id"].as_str().unwrap().to_owned();
    let t2 = ok(&p, &["trend", &series]);
    let next2: Vec<String> = t2["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let small = next2
        .iter()
        .find(|n| {
            n.starts_with(&format!(
                "dla harness gen --lang python --driver --probe {f}"
            ))
        })
        .unwrap_or_else(|| panic!("no small driver: {next2:?}"));
    assert!(
        small.contains("a unit that calls the function of that line")
            && small.contains(&format!(
                "an alloc-test on {h} re-runs the whole test suite per run"
            )),
        "{small}"
    );
    assert!(
        next2
            .iter()
            .any(|n| n.starts_with("dla harness run <H>, dla trend <S>, then dla alloc-test <H>")),
        "{next2:?}"
    );
    // No plain alloc-test of the loop harness is the first thing offered.
    assert!(
        !next2
            .iter()
            .any(|n| n == &format!("dla alloc-test {h} --finding {f}")),
        "{next2:?}"
    );
    // The growth-series record the hint of `ledger label` points at: the plain trend writes it
    // and its passing column names the finding.
    let ledger = fs::read_to_string(
        fs::read_dir(p.join(".dla"))
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .find(|d| d.join("ledger.jsonl").is_file())
            .unwrap()
            .join("ledger.jsonl"),
    )
    .unwrap();
    assert!(
        ledger.lines().any(|l| {
            let v: Value = serde_json::from_str(l).unwrap();
            v["type"] == "evidence"
                && v["kind"] == "unit-count-series"
                && v["result"]["passed"].as_array().is_some_and(|c| {
                    c.iter().any(|c| {
                        c["column"] == COLUMN
                            && c["candidates"]
                                .as_array()
                                .is_some_and(|a| a.iter().any(|x| x == &f))
                    })
                })
        }),
        "the trend's record does not name {f} at its column"
    );
    // The ledger refuses to close it unmeasured and says the same.
    let c = dla(
        &p,
        &[
            "ledger",
            "resolve",
            &f,
            "--as",
            "unresolved",
            "--reason",
            "inconclusive",
            "--remedy",
            "x",
        ],
    );
    assert_eq!(c.json["error"]["code"], "growth-not-measured", "{}", c.text);
    let fix = c.json["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains(&format!(
            "dla harness gen --lang python --driver --probe {f}   (a small driver of its own"
        )) && fix.contains("re-runs the whole test suite per run"),
        "{fix}"
    );
    assert!(
        c.json["error"]["missing"]
            .as_str()
            .unwrap()
            .contains("on a harness of its own, not on"),
        "{}",
        c.text
    );
    // `--probe F` accepts a finding that has a site and no container, and the hooks comment
    // names the line.
    let d = ok(
        &p,
        &[
            "harness", "gen", "--lang", "python", "--driver", "--probe", &f,
        ],
    );
    assert_eq!(d["summary"]["targets"][0]["site"], SITE, "{d}");
    let edit = p.join(d["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(edit).unwrap();
    assert!(
        text.contains(&format!("{f} at {SITE}: holder native-handle/global-ref")),
        "{text}"
    );
    assert!(
        text.contains(&format!("make every unit run the code at {SITE}")),
        "{text}"
    );
    // The final gate lists the growth with the same commands.
    let gate = dla(&p, &["gate", "check", "--final"]);
    assert!(
        gate.text.contains(&format!(
            "dla harness gen --lang python --driver --probe {f}"
        )),
        "{}",
        gate.text
    );
}

#[test]
fn a_failed_run_of_the_loop_lets_the_queue_go_on_with_the_driver() {
    if !have_pytest() {
        return;
    }
    // Every test fails.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("project");
    write(&p, "shop/__init__.py", "");
    write(&p, "shop/reg.py", REG);
    write(
        &p,
        "tests/test_bad.py",
        "def test_bad():\n    assert False, 'always'\n",
    );
    ok(&p, &["run", "init", "--session-id", "t"]);
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
    ] {
        ok(&p, a);
    }
    assert_eq!(queue_command(&p), LOOP_FIRST);
    let g = ok(&p, &["harness", "gen", "--lang", "python", "--pytest"]);
    let h = g["summary"]["harness"].as_str().unwrap().to_owned();
    let r = dla(&p, &["harness", "run", &h]);
    assert_ne!(r.code, 0, "{}", r.text);
    assert_eq!(r.json["error"]["code"], "harness-failed", "{}", r.text);
    assert!(
        r.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("DLA-PYTEST no test passed in any test file"),
        "{}",
        r.text
    );
    // The attempt failed: the loop is not offered again, the driver batch is.
    assert_eq!(queue_command(&p), DRIVER);
}

/// The project of the loop tests with a module the census reads candidates from that the sweep
/// does not report: a handler list and a thread that appends to a list of its own.
fn project_with_census_candidates(dir: &Path, with_tests: bool) {
    project(dir, with_tests);
    write(
        dir,
        "shop/hooks.py",
        "import atexit\nimport threading\n\n_handlers = []\n\n\ndef hook(f):\n    atexit.register(f)\n    _handlers.append(f)\n\n\nclass Pool:\n    def __init__(self):\n        self.items = []\n        self.t = threading.Thread(target=self.run)\n\n    def run(self):\n        self.items.append(1)\n",
    );
}

/// `run init`, probe, sweep and census, with no `workload discover`.
fn census_only(
    init_flags: &[&str],
    setup: impl FnOnce(&Path),
) -> (tempfile::TempDir, PathBuf, Value) {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("project");
    setup(&p);
    let mut init = vec!["run", "init", "--session-id", "t"];
    init.extend_from_slice(init_flags);
    ok(&p, &init);
    ok(&p, &["probe"]);
    ok(&p, &["sweep", "--lang", "python"]);
    let census = ok(&p, &["census", "--lang", "python"]);
    (dir, p, census)
}

/// The workload discovery file of the only run of `project`.
fn discovery_file(project: &Path) -> PathBuf {
    let runs: Vec<_> = fs::read_dir(project.join(".dla"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("state.json").is_file())
        .collect();
    assert_eq!(runs.len(), 1, "{runs:?}");
    runs[0].join("workload/workloads.json")
}

#[test]
fn the_python_census_writes_the_discovery_when_the_project_has_test_files() {
    let (_d, p, census) = census_only(&[], |p| project_with_census_candidates(p, true));
    assert!(census["candidates"].as_u64().unwrap() > 0, "{census}");
    assert!(discovery_file(&p).is_file(), "{census}");
    let note = census["workloads"]
        .as_str()
        .unwrap_or_else(|| panic!("{census}"));
    assert!(
        note.starts_with("discovered for the measurement queue (")
            && note.contains("the queue of a Python project that has test files"),
        "{note}"
    );
    // No `dla workload discover` ran, and the queue starts with the loop.
    assert_eq!(queue_command(&p), LOOP_FIRST);
    // Running the discovery by hand changes nothing about the queue.
    ok(&p, &["workload", "discover", "--lang", "python"]);
    assert_eq!(queue_command(&p), LOOP_FIRST);
}

#[test]
fn the_python_census_of_a_project_without_test_files_writes_no_discovery() {
    let (_d, p, census) = census_only(&[], |p| project_with_census_candidates(p, false));
    assert!(census["candidates"].as_u64().unwrap() > 0, "{census}");
    assert!(!discovery_file(&p).exists());
    assert!(census.get("workloads").is_none(), "{census}");
    assert_eq!(queue_command(&p), DRIVER);
}

#[test]
fn a_finding_of_a_python_project_without_test_files_is_still_closed_without_an_exercise() {
    // As in the gate flow test: the queue is off, so only the ledger rule is in play.
    let (_d, p, census) = census_only(&["--queue", "off"], |p| {
        project_with_census_candidates(p, false)
    });
    assert!(census["candidates"].as_u64().unwrap() > 0, "{census}");
    assert!(!discovery_file(&p).exists());
    let id = "F1";
    let r = dla(
        &p,
        &[
            "ledger",
            "resolve",
            id,
            "--as",
            "unresolved",
            "--reason",
            "not_exercised",
            "--remedy",
            "write a workload that calls pin",
        ],
    );
    assert_eq!(r.code, 0, "{}", r.text);
    assert!(!r.text.contains("harness-not-generated"), "{}", r.text);
}

#[test]
fn a_discovery_that_is_already_there_is_not_rewritten_by_the_python_census() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("project");
    project_with_census_candidates(&p, true);
    ok(&p, &["run", "init", "--session-id", "t"]);
    ok(&p, &["probe"]);
    ok(&p, &["sweep", "--lang", "python"]);
    ok(&p, &["workload", "discover", "--lang", "python"]);
    let file = discovery_file(&p);
    // Change the project, so a rewrite would differ.
    write(
        &p,
        "tests/test_more.py",
        "def test_more():\n    assert True\n",
    );
    let before = fs::read(&file).unwrap();
    let census = ok(&p, &["census", "--lang", "python"]);
    assert!(census["candidates"].as_u64().unwrap() > 0, "{census}");
    assert!(census.get("workloads").is_none(), "{census}");
    assert_eq!(fs::read(&file).unwrap(), before);
}

#[test]
fn the_python_census_writes_the_discovery_even_when_it_found_no_candidate() {
    let (_d, p, census) = census_only(&[], |p| {
        write(p, "app/__init__.py", "");
        write(p, "app/pure.py", "def add(a, b):\n    return a + b\n");
        write(
            p,
            "tests/test_pure.py",
            "def test_add():\n    assert True\n",
        );
    });
    assert_eq!(census["candidates"], 0, "{census}");
    assert_eq!(census["skipped_duplicates"], 0, "{census}");
    assert!(discovery_file(&p).is_file(), "{census}");
    assert!(census["workloads"].is_string(), "{census}");
}

#[test]
fn the_usual_order_probe_sweep_census_offers_the_loop_when_the_census_only_joins_the_sweep() {
    // On the plain project the sweep raises the candidate and the census adds none.
    let (_d, p, census) = census_only(&[], |p| project(p, true));
    assert_eq!(census["candidates"], 0, "{census}");
    assert_eq!(census["skipped_duplicates"], 0, "{census}");
    assert!(discovery_file(&p).is_file(), "{census}");
    assert_eq!(queue_command(&p), LOOP_FIRST);
}

/// The summary of `dla harness gen --lang python --driver --next`.
fn driver_summary(project: &Path) -> Value {
    let g = dla(
        project,
        &["harness", "gen", "--lang", "python", "--driver", "--next"],
    );
    assert_eq!(g.code, 0, "{}", g.text);
    g.json["summary"].clone()
}

#[test]
fn the_census_note_names_the_loop_command() {
    let (_d, _p, census) = census_only(&[], |p| project(p, true));
    let note = census["workloads"]
        .as_str()
        .unwrap_or_else(|| panic!("{census}"));
    assert!(
        note.contains("dla harness gen --lang python --pytest (nothing to write)"),
        "{note}"
    );
}

#[test]
fn a_hook_driver_before_the_loop_carries_the_pytest_hint_and_not_otherwise() {
    let (_d, p, _census) = census_only(&[], |p| project(p, true));
    let summary = driver_summary(&p);
    let hint = summary["pytest_hint"]
        .as_str()
        .unwrap_or_else(|| panic!("no pytest_hint: {summary}"));
    assert!(
        hint.contains("dla harness gen --lang python --pytest"),
        "{hint}"
    );
    assert!(hint.contains("before you fill these hooks"), "{hint}");
    // No test files: no hint.
    let (_d2, p2, _c2) = census_only(&[], |p| project(p, false));
    let summary = driver_summary(&p2);
    assert!(summary.get("pytest_hint").is_none(), "{summary}");
    // After a refused generation of the loop the queue goes on with the drivers: no hint.
    let (_d3, p3, _c3) = census_only(&[], |p| project(p, true));
    let shim = tempfile::tempdir().unwrap();
    let path = path_without_pytest(shim.path());
    let o = dla_with(
        &p3,
        Some(&path),
        &["harness", "gen", "--lang", "python", "--pytest"],
    );
    assert_ne!(o.code, 0);
    let summary = driver_summary(&p3);
    assert!(summary.get("pytest_hint").is_none(), "{summary}");
}

/// `totals.queue.python.command` of `dla ledger list`.
fn ledger_queue_command(project: &Path) -> String {
    let o = dla(project, &["ledger", "list"]);
    assert_eq!(o.code, 0, "{}", o.text);
    let first = o.text.lines().next().unwrap();
    let v: Value = serde_json::from_str(first).unwrap();
    v["totals"]["queue"]["python"]["command"]
        .as_str()
        .unwrap_or_else(|| panic!("no command: {first}"))
        .to_owned()
}

#[test]
fn ledger_list_prints_the_queue_command_of_run_status() {
    let (_d, p) = prepared(true);
    assert_eq!(ledger_queue_command(&p), LOOP_FIRST);
    assert_eq!(ledger_queue_command(&p), queue_command(&p));
    let shim = tempfile::tempdir().unwrap();
    let path = path_without_pytest(shim.path());
    let o = dla_with(
        &p,
        Some(&path),
        &["harness", "gen", "--lang", "python", "--pytest"],
    );
    assert_ne!(o.code, 0);
    assert_eq!(ledger_queue_command(&p), DRIVER);
    assert_eq!(ledger_queue_command(&p), queue_command(&p));
}

/// A project with one worker, python, after its sweep, census and workload discovery.
fn prepared_worker(with_tests: bool) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir.path().join("project");
    project(&project_dir, with_tests);
    ok(&project_dir, &["run", "init", "--session-id", "t"]);
    ok(&project_dir, &["run", "dispatch", "--workers", "python"]);
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
    ] {
        let mut args = vec!["--worker", "python"];
        args.extend_from_slice(a);
        ok(&project_dir, &args);
    }
    (dir, project_dir)
}

/// The items of the worker's `dla gate check --stage sweep`, with the optional `PATH`.
fn worker_gate_items(project: &Path, path: Option<&str>) -> Vec<Value> {
    let gate = dla_with(
        project,
        path,
        &["--worker", "python", "gate", "check", "--stage", "sweep"],
    );
    gate.json["missing"].as_array().cloned().unwrap_or_default()
}

/// The fix of every item that is not the queue's own and generates a harness, with the
/// description of the item.
fn harness_fixes(items: &[Value]) -> Vec<(String, String)> {
    items
        .iter()
        .filter(|i| i["kind"] != "queue")
        .filter_map(|i| {
            let fix = i["fix"].as_str()?;
            fix.contains("dla harness gen").then(|| {
                (
                    i["description"].as_str().unwrap_or("").to_owned(),
                    fix.to_owned(),
                )
            })
        })
        .collect()
}

#[test]
fn gate_items_of_python_findings_name_the_loop_first_while_it_is_pending() {
    let (_d, p) = prepared_worker(true);
    let items = worker_gate_items(&p, None);
    let fixes = harness_fixes(&items);
    assert!(
        fixes.iter().any(|(d, _)| d.contains("has no disposition")),
        "{items:?}"
    );
    for (what, fix) in &fixes {
        assert!(fix.starts_with(LOOP_FIRST), "{what}: {fix}");
    }
    // The core's final gate lists the same items with the same text.
    let (_dc, pc) = prepared(true);
    let gate = dla(&pc, &["gate", "check", "--final"]);
    // The output lists the first items only; the stored file has them all.
    let file = fs::read_to_string(pc.join(gate.json["pointer"].as_str().unwrap())).unwrap();
    let all: Value = serde_json::from_str(&file).unwrap();
    let items = all["missing"].as_array().cloned().unwrap_or_default();
    let fixes = harness_fixes(&items);
    assert!(!fixes.is_empty(), "{items:?}");
    for (what, fix) in &fixes {
        assert!(fix.starts_with(LOOP_FIRST), "{what}: {fix}");
    }
    // After a refused generation of the loop the same items name the driver batch.
    let shim = tempfile::tempdir().unwrap();
    let path = path_without_pytest(shim.path());
    let o = dla_with(
        &p,
        Some(&path),
        &[
            "--worker", "python", "harness", "gen", "--lang", "python", "--pytest",
        ],
    );
    assert_ne!(o.code, 0, "{}", o.text);
    let fixes = harness_fixes(&worker_gate_items(&p, None));
    assert!(!fixes.is_empty());
    for (what, fix) in &fixes {
        assert!(!fix.contains("--pytest"), "{what}: {fix}");
        assert!(fix.contains(DRIVER), "{what}: {fix}");
    }

    // A project without test files names the driver batch from the start.
    let (_d2, p2) = prepared_worker(false);
    let fixes = harness_fixes(&worker_gate_items(&p2, None));
    assert!(!fixes.is_empty());
    for (what, fix) in &fixes {
        assert!(!fix.contains("--pytest"), "{what}: {fix}");
        assert!(fix.contains(DRIVER), "{what}: {fix}");
    }
}
