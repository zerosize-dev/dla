//! The allocation-site columns of a Python driver run (`py.blocks@<file:line>`) through the
//! binary: `dla trend` prints the command that adds the finding at the line, the column ties to
//! that finding by its own site, the final gate lists the column until it does, a finding with a
//! passed column cannot be closed without an alloc-test, the alloc-test decides by the site column
//! and `dla ledger confirm` accepts the trend's evidence.
//!
//! The project leaks a Python object per call without any container: a function takes a reference
//! to a new list and never gives it back (`Py_IncRef` through ctypes, standing in for the C
//! function of an extension module that does the same), so nothing in the program holds the
//! objects and the census lists no finding at the line. Each test skips, printing the reason, when
//! `python3` is missing.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn have_python() -> bool {
    let ok = Std::new("python3")
        .args([
            "-c",
            "import sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipping: python3 is not on PATH");
    }
    ok
}

struct Out {
    code: i32,
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
        "DLA_TRACEMALLOC_FRAMES",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1-4")
        .env("DLA_CONSENT_RAW_VALUES", "1");
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
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

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// `shop/store.py:5` allocates the list whose reference is never released.
const SITE: &str = "shop/store.py:5";
const COLUMN: &str = "py.blocks@shop/store.py:5";

struct Run {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
}

/// Run init, census, discover, a driver harness whose hooks call `keep`, one run of it (series
/// S1) and its trend (T1).
fn prepared() -> Run {
    prepare(
        "import ctypes\n\n\ndef pin(i):\n    obj = [i]\n    ctypes.pythonapi.Py_IncRef(ctypes.py_object(obj))\n",
        false,
    )
}

/// The store of [`SITE`] with a switch: `pin(i)` keeps the object (the leak), `pin(i, False)`
/// runs the line and keeps nothing.
const STORE_SWITCH: &str = "import ctypes\n\n\ndef pin(i, keep=True):\n    obj = [i]\n    if keep:\n        ctypes.pythonapi.Py_IncRef(ctypes.py_object(obj))\n";

/// [`prepared`] with `store` as `shop/store.py`, and, with `table`, a module with a table of its
/// own (`shop/cache.py`) that the census lists as the finding F1 and the first harness targets.
fn prepare(store: &str, table: bool) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write(&project, "shop/__init__.py", "");
    write(&project, "shop/store.py", store);
    if table {
        write(
            &project,
            "shop/cache.py",
            "REG = {}\n\n\ndef put(k):\n    REG[k] = k\n",
        );
    }
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["census", "--lang", "python"]);
    ok(&project, &["workload", "discover", "--lang", "python"]);
    let g = ok(
        &project,
        &["harness", "gen", "--lang", "python", "--driver", "--next"],
    );
    let edit = project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            "    from shop import store\n    _state[\"store\"] = store",
        )
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            "    _state[\"store\"].pin(i)",
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    ok(&project, &["harness", "run", "H1", "--mode", "fresh"]);
    Run {
        _dir: dir,
        project,
        run_dir,
    }
}

impl Run {
    fn trend(&self) -> Value {
        ok(&self.project, &["trend", "S1"])
    }

    /// The `next` entry that starts with `prefix`.
    fn next_of(trend: &Value, prefix: &str) -> Option<String> {
        trend["next"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .find(|n| n.starts_with(prefix))
            .map(str::to_owned)
    }

    fn final_gate(&self) -> Vec<Value> {
        let _ = dla(&self.project, &["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }

    fn ledger(&self) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }
}

const ADD_PREFIX: &str = "dla ledger add --origin growth-series --evidence ";

#[test]
fn an_untied_site_column_prints_the_command_that_adds_the_finding_at_its_line() {
    if !have_python() {
        return;
    }
    let r = prepared();
    let t = r.trend();
    let passed = t["passed"].as_array().unwrap();
    let col = passed
        .iter()
        .find(|c| c["column"] == COLUMN)
        .unwrap_or_else(|| panic!("{COLUMN} did not pass: {t}"));
    assert!(
        col["candidates"].as_array().is_none_or(Vec::is_empty),
        "{col}"
    );
    let add = Run::next_of(&t, ADD_PREFIX).unwrap_or_else(|| panic!("{}", t["next"]));
    assert!(
        add.contains(&format!(" --site {SITE} --region heap --holder <kind>   # no finding there; {COLUMN} then ties to it. Holder: read the line: ")),
        "{add}"
    );
    assert!(add.contains("native-handle/global-ref"), "{add}");
    // Site columns come before the whole-process hints.
    let next: Vec<&str> = t["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let at = next.iter().position(|n| n.starts_with(ADD_PREFIX)).unwrap();
    if let Some(gap) = next.iter().position(|n| n.starts_with("dla gap ")) {
        assert!(at < gap, "{next:?}");
    }
    assert!(next.len() <= 8, "{next:?}");
}

/// The evidence id a printed `dla ledger add ... --evidence E ...` names.
fn evidence_of(add: &str) -> String {
    add.strip_prefix(ADD_PREFIX)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

fn column_items(missing: &[Value]) -> Vec<&Value> {
    missing
        .iter()
        .filter(|m| {
            m["kind"] == "trend_column" && m["id"].as_str().is_some_and(|i| i.ends_with(COLUMN))
        })
        .collect()
}

#[test]
fn the_final_gate_lists_the_untied_column_with_the_add_command_and_a_finding_at_another_line_does_not_tie_it()
 {
    if !have_python() {
        return;
    }
    let r = prepared();
    let t = r.trend();
    let add = Run::next_of(&t, ADD_PREFIX).unwrap();
    let e = evidence_of(&add);
    let missing = r.final_gate();
    let items = column_items(&missing);
    assert_eq!(items.len(), 1, "{missing:?}");
    let fix = items[0]["fix"].as_str().unwrap();
    assert!(
        fix.starts_with(&format!(
            "{ADD_PREFIX}{e} --site {SITE} --region heap --holder <kind>   # no finding there; "
        )),
        "{fix}"
    );
    // A finding at another line that cites the same evidence is not at this line.
    ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &e,
            "--site",
            "shop/store.py:1",
            "--region",
            "heap",
            "--holder",
            "static/global",
        ],
    );
    let after = r.final_gate();
    let items = column_items(&after);
    assert_eq!(items.len(), 1, "{after:?}");
    assert!(
        items[0]["description"]
            .as_str()
            .unwrap()
            .contains("is tied to no finding")
            && items[0]["fix"].as_str().unwrap().starts_with(ADD_PREFIX)
            && items[0]["fix"]
                .as_str()
                .unwrap()
                .contains(&format!(" --site {SITE} ")),
        "tied by the evidence, not by the line: {}",
        items[0]
    );
}

#[test]
fn the_finding_added_at_the_line_ties_the_column_and_cannot_be_closed_unmeasured() {
    if !have_python() {
        return;
    }
    let r = prepared();
    let t = r.trend();
    let e = evidence_of(&Run::next_of(&t, ADD_PREFIX).unwrap());
    let added = ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &e,
            "--site",
            SITE,
            "--region",
            "heap",
            "--holder",
            "native-handle/global-ref",
        ],
    );
    let f = added["id"].as_str().unwrap().to_owned();
    // The column is tied now, to a finding without a disposition; the gate lists that finding
    // with the alloc-test.
    let missing = r.final_gate();
    let items = column_items(&missing);
    assert_eq!(items.len(), 1, "{missing:?}");
    assert!(
        items[0]["description"]
            .as_str()
            .unwrap()
            .contains("tied only to findings that have no disposition"),
        "{}",
        items[0]
    );
    let item = missing
        .iter()
        .find(|m| m["id"] == f.as_str())
        .unwrap_or_else(|| panic!("{missing:?}"));
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .contains(&format!("dla alloc-test H1 --finding {f}")),
        "{item}"
    );
    // Closing it is refused, whatever the reason; close-rest leaves it.
    for reason in ["inconclusive", "not_exercised"] {
        let o = dla(
            &r.project,
            &[
                "ledger",
                "resolve",
                &f,
                "--as",
                "unresolved",
                "--reason",
                reason,
                "--remedy",
                "x",
            ],
        );
        assert_ne!(o.code, 0, "{reason}: {}", o.text);
        assert_eq!(o.json["error"]["code"], "growth-not-measured", "{}", o.text);
        assert!(
            o.json["error"]["message"]
                .as_str()
                .unwrap()
                .contains(COLUMN),
            "{}",
            o.text
        );
    }
    let out = ok(&r.project, &["ledger", "close-rest"]);
    assert!(
        out["left"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["id"] == f.as_str()),
        "{out}"
    );
    // A new trend lists the finding as the column's candidate and prints the route.
    let t2 = r.trend();
    let col = t2["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == COLUMN)
        .unwrap()
        .clone();
    assert_eq!(col["candidates"], serde_json::json!([f]), "{col}");
    let site_adds = t2["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .filter(|n| n.starts_with(ADD_PREFIX) && n.contains(" --site "))
        .count();
    assert_eq!(site_adds, 0, "{}", t2["next"]);
    assert!(
        Run::next_of(&t2, &format!("dla alloc-test H1 --finding {f}")).is_some(),
        "{}",
        t2["next"]
    );
}

#[test]
fn the_alloc_test_decides_by_the_site_column_and_the_trend_evidence_confirms() {
    if !have_python() {
        return;
    }
    let r = prepared();
    let t = r.trend();
    let e = evidence_of(&Run::next_of(&t, ADD_PREFIX).unwrap());
    let added = ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &e,
            "--site",
            SITE,
            "--region",
            "heap",
            "--holder",
            "native-handle/global-ref",
        ],
    );
    let f = added["id"].as_str().unwrap().to_owned();
    ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            &e,
        ],
    );
    let workload = r
        .ledger()
        .iter()
        .filter(|v| v["kind"] == "workload")
        .filter_map(|v| v["evidence_id"].as_str().map(str::to_owned))
        .next_back()
        .unwrap();
    ok(&r.project, &["ledger", "test", &f, "--evidence", &workload]);
    let a = ok(&r.project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["decided_by"], COLUMN, "{a}");
    assert_eq!(a["favors"], "leak", "{a}");
    ok(
        &r.project,
        &[
            "ledger",
            "refute",
            &format!("{f}.n1"),
            "--evidence",
            a["evidence"].as_str().unwrap(),
        ],
    );
    // The trend's own evidence confirms: the column ties to the finding by its site.
    let c = ok(&r.project, &["ledger", "confirm", &f, "--evidence", &e]);
    assert_eq!(c["to"], "confirmed", "{c}");
    // A confirmed finding is a disposition: the gate lists neither the column nor an open finding
    // (its explanation and label are other items).
    let missing = r.final_gate();
    assert!(
        column_items(&missing).is_empty(),
        "{:?}",
        column_items(&missing)
    );
    let open: Vec<&Value> = missing
        .iter()
        .filter(|m| {
            m["id"] == f.as_str() && matches!(m["kind"].as_str(), Some("finding" | "growth"))
        })
        .collect();
    assert!(open.is_empty(), "{open:?}");
}

/// Adds the finding of the table that `shop/store.py` writes into: its own site is the line that
/// defines the table (line 1), and the census annotation lists the line of the column (line 5) as
/// a writer of the holder. Returns its id.
fn add_table_finding(r: &Run, evidence: &str) -> String {
    let added = ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            evidence,
            "--site",
            "shop/store.py:1",
            "--region",
            "heap",
            "--holder",
            "static/global",
        ],
    );
    let f = added["id"].as_str().unwrap().to_owned();
    ok(&r.project, &["ledger", "annotate", &f, "--writer", SITE]);
    f
}

fn site_adds(t: &Value) -> usize {
    t["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .filter(|n| n.starts_with(ADD_PREFIX) && n.contains(&format!(" --site {SITE} ")))
        .count()
}

#[test]
fn a_column_at_a_writer_line_of_a_finding_is_tied_to_it_and_the_gate_asks_for_no_second_finding() {
    if !have_python() {
        return;
    }
    let r = prepared();
    let t = r.trend();
    let e = evidence_of(&Run::next_of(&t, ADD_PREFIX).unwrap());
    let f = add_table_finding(&r, &e);
    // The gate (on the first trend, made before the finding existed) ties the column to the
    // finding of the table and asks for nothing at the line; the finding has no disposition.
    let missing = r.final_gate();
    let items = column_items(&missing);
    assert_eq!(items.len(), 1, "{missing:?}");
    let desc = items[0]["description"].as_str().unwrap();
    assert!(
        desc.contains("tied only to findings that have no disposition"),
        "{}",
        items[0]
    );
    let fix = items[0]["fix"].as_str().unwrap();
    assert!(!fix.contains(&format!(" --site {SITE} ")), "{fix}");
    // The open finding is the item that names the route.
    let item = missing
        .iter()
        .find(|m| m["id"] == f.as_str())
        .unwrap_or_else(|| panic!("{missing:?}"));
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .contains(&format!("dla alloc-test H1 --finding {f}")),
        "{item}"
    );
    // The finding counts as grown by the column: it cannot be closed unmeasured.
    for reason in ["inconclusive", "not_exercised"] {
        let o = dla(
            &r.project,
            &[
                "ledger",
                "resolve",
                &f,
                "--as",
                "unresolved",
                "--reason",
                reason,
                "--remedy",
                "x",
            ],
        );
        assert_ne!(o.code, 0, "{reason}: {}", o.text);
        assert_eq!(o.json["error"]["code"], "growth-not-measured", "{}", o.text);
        assert!(
            o.json["error"]["message"]
                .as_str()
                .unwrap()
                .contains(COLUMN),
            "{}",
            o.text
        );
    }
    // A new trend lists the finding as the candidate of the column and prints no command that
    // adds a finding at the line; it leads with the route for the finding.
    let t2 = r.trend();
    let col = t2["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == COLUMN)
        .unwrap()
        .clone();
    assert_eq!(col["candidates"], serde_json::json!([f]), "{col}");
    assert_eq!(site_adds(&t2), 0, "{}", t2["next"]);
    assert!(
        Run::next_of(&t2, &format!("dla alloc-test H1 --finding {f}")).is_some(),
        "{}",
        t2["next"]
    );
    // Once the finding has a disposition the gate lists no item for the column.
    let workload = r
        .ledger()
        .iter()
        .filter(|v| v["kind"] == "workload")
        .filter_map(|v| v["evidence_id"].as_str().map(str::to_owned))
        .next_back()
        .unwrap();
    ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            &e,
        ],
    );
    ok(&r.project, &["ledger", "test", &f, "--evidence", &workload]);
    let a = ok(&r.project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "leak", "{a}");
    ok(
        &r.project,
        &[
            "ledger",
            "refute",
            &format!("{f}.n1"),
            "--evidence",
            a["evidence"].as_str().unwrap(),
        ],
    );
    // The evidence of the second trend, which lists the finding as the candidate of the column,
    // confirms (the first trend was made before the finding had the writer).
    let counter = r
        .ledger()
        .iter()
        .filter(|v| v["kind"] == "unit-count-series")
        .filter_map(|v| v["evidence_id"].as_str().map(str::to_owned))
        .find(|id| {
            t2["evidence"]
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x == id.as_str())
        })
        .unwrap_or_else(|| panic!("{}", t2["evidence"]));
    let c = ok(
        &r.project,
        &["ledger", "confirm", &f, "--evidence", &counter],
    );
    assert_eq!(c["to"], "confirmed", "{c}");
    let after = r.final_gate();
    assert!(
        column_items(&after).is_empty(),
        "{:?}",
        column_items(&after)
    );
}

#[test]
fn a_remover_line_or_a_finding_without_that_writer_does_not_tie_the_column() {
    if !have_python() {
        return;
    }
    let r = prepared();
    let t = r.trend();
    let e = evidence_of(&Run::next_of(&t, ADD_PREFIX).unwrap());
    let added = ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &e,
            "--site",
            "shop/store.py:1",
            "--region",
            "heap",
            "--holder",
            "static/global",
        ],
    );
    let f = added["id"].as_str().unwrap();
    ok(&r.project, &["ledger", "annotate", f, "--remover", SITE]);
    let items = column_items(&r.final_gate())
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 1, "{items:?}");
    assert!(
        items[0]["fix"].as_str().unwrap().starts_with(ADD_PREFIX),
        "{}",
        items[0]
    );
    assert_eq!(site_adds(&r.trend()), 1);
}

/// A run with a hand-written series S1 of harness H1 whose columns are `py.traced` (a gauge, the
/// whole process) first and the allocation-site column second, both rising.
fn synthetic() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write(&project, "shop/__init__.py", "");
    write(
        &project,
        "shop/store.py",
        "import ctypes\n\n\ndef pin(i):\n    obj = [i]\n",
    );
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    let m: dla_core::HarnessManifest = serde_json::from_value(serde_json::json!({
        "id": "H1", "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 10, "key_modes": ["fresh"],
        "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&run_dir).unwrap();
    let points: Vec<Value> = (0..12)
        .map(|i| {
            serde_json::json!({"x": f64::from(i + 1) * 10.0, "t_ms": i * 100,
                "values": [f64::from(i) * 4000.0 + f64::from(i % 3), f64::from(i) * 20.0 + f64::from(i % 2)]})
        })
        .collect();
    fs::create_dir_all(run_dir.join("series")).unwrap();
    fs::write(
        run_dir.join("series/S1.json"),
        serde_json::json!({"schema_version": 1, "id": "S1",
            "source": {"kind": "harness", "harness": "H1", "mode": "fresh", "runtime": "python"},
            "axis": "iteration", "warmup": 1,
            "columns": [
                {"name": "py.traced", "unit": "bytes", "kind": "gauge", "region": "heap",
                 "baseline": true, "after_gc": true, "scope": "process", "source": "driver"},
                {"name": COLUMN, "unit": "count", "kind": "counter", "region": "heap",
                 "baseline": false, "after_gc": true, "scope": "process", "site": SITE,
                 "source": "driver"}],
            "points": points, "events": [], "limitations": []})
        .to_string(),
    )
    .unwrap();
    fs::create_dir_all(run_dir.join("harness/H1")).unwrap();
    fs::write(
        run_dir.join("harness/H1/run-S1.json"),
        serde_json::json!({"series": ["S1"]}).to_string(),
    )
    .unwrap();
    (dir, project)
}

#[test]
fn the_site_column_comes_before_the_whole_process_hint_in_next() {
    let (_dir, project) = synthetic();
    let t = ok(&project, &["trend", "S1"]);
    let passed: Vec<&str> = t["passed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["column"].as_str())
        .collect();
    assert!(
        passed.contains(&"py.traced") && passed.contains(&COLUMN),
        "{passed:?}"
    );
    let next: Vec<&str> = t["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        next[0].starts_with(ADD_PREFIX) && next[0].contains(&format!(" --site {SITE} ")),
        "{next:?}"
    );
    assert!(
        next.iter().any(|n| n.starts_with("dla gap S1 --residual")),
        "{next:?}"
    );
    assert!(next.len() <= 8, "{next:?}");
}

/// Five allocation-site columns `py.blocks@s<k>.py:1` whose name order is the reverse of their
/// slope order: `s1` grows by 10 blocks per run, `s5` by 50.
fn five_sites() -> (tempfile::TempDir, PathBuf) {
    let (dir, project) = synthetic();
    let run_dir = fs::read_dir(project.join(".dla"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.join("series/S1.json").exists())
        .unwrap();
    let columns: Vec<Value> = (1..=5)
        .map(|k| {
            serde_json::json!({"name": format!("py.blocks@s{k}.py:1"), "unit": "count",
                "kind": "counter", "region": "heap", "baseline": false, "after_gc": true,
                "scope": "process", "site": format!("s{k}.py:1"), "source": "driver"})
        })
        .collect();
    let points: Vec<Value> = (0..12)
        .map(|i| {
            let values: Vec<f64> = (1..=5)
                .map(|k| f64::from(i) * f64::from(k) * 10.0)
                .collect();
            serde_json::json!({"x": f64::from(i + 1) * 10.0, "t_ms": i * 100, "values": values})
        })
        .collect();
    fs::write(
        run_dir.join("series/S1.json"),
        serde_json::json!({"schema_version": 1, "id": "S1",
            "source": {"kind": "harness", "harness": "H1", "mode": "fresh", "runtime": "python"},
            "axis": "iteration", "warmup": 1,
            "columns": columns, "points": points, "events": [], "limitations": []})
        .to_string(),
    )
    .unwrap();
    (dir, project)
}

/// The sites of the `ledger add` commands in `next`, in order.
fn add_sites(t: &Value) -> Vec<String> {
    t["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .filter(|n| n.starts_with(ADD_PREFIX))
        .filter_map(|n| n.split(" --site ").nth(1))
        .filter_map(|n| n.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_trend_names_the_lines_that_grow_most_first_and_a_finding_at_a_line_leads_next() {
    let (_dir, project) = five_sites();
    let t = ok(&project, &["trend", "S1"]);
    let passed: Vec<&str> = t["passed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["column"].as_str())
        .collect();
    assert_eq!(
        passed,
        [
            "py.blocks@s5.py:1",
            "py.blocks@s4.py:1",
            "py.blocks@s3.py:1",
            "py.blocks@s2.py:1",
            "py.blocks@s1.py:1"
        ],
        "{t}"
    );
    assert_eq!(
        add_sites(&t),
        ["s5.py:1", "s4.py:1", "s3.py:1"],
        "{}",
        t["next"]
    );
    // A finding added at the second largest line: the commands for it come before the add
    // commands for the lines that still have none.
    let add = Run::next_of(&t, ADD_PREFIX).unwrap();
    let e = evidence_of(
        t["next"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .find(|n| n.contains(" --site s4.py:1 "))
            .unwrap(),
    );
    assert!(add.contains(" --site s5.py:1 "), "{add}");
    let added = ok(
        &project,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &e,
            "--site",
            "s4.py:1",
            "--region",
            "heap",
            "--holder",
            "static/global",
        ],
    );
    let f = added["id"].as_str().unwrap().to_owned();
    let t2 = ok(&project, &["trend", "S1"]);
    let first = t2["next"][0].as_str().unwrap();
    assert!(
        first.contains(&f) && !first.starts_with(ADD_PREFIX),
        "{}",
        t2["next"]
    );
    assert_eq!(
        add_sites(&t2),
        ["s5.py:1", "s3.py:1", "s2.py:1"],
        "{}",
        t2["next"]
    );
}

#[test]
fn the_final_gate_lists_the_untied_site_columns_largest_slope_first() {
    let (_dir, project) = five_sites();
    ok(&project, &["trend", "S1"]);
    let _ = dla(&project, &["gate", "check", "--final"]);
    let run_dir = fs::read_dir(project.join(".dla"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.join("gate/final.json").exists())
        .unwrap();
    let full: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("gate/final.json")).unwrap())
            .unwrap();
    let ids: Vec<&str> = full["missing"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["kind"] == "trend_column")
        .filter_map(|m| m["id"].as_str())
        .filter_map(|i| i.split_once(':').map(|(_, c)| c))
        .collect();
    assert_eq!(
        ids,
        [
            "py.blocks@s5.py:1",
            "py.blocks@s4.py:1",
            "py.blocks@s3.py:1",
            "py.blocks@s2.py:1",
            "py.blocks@s1.py:1"
        ],
        "{full}"
    );
}

// ------------------------------------------------------------------------- the stacks at the line

/// The finding at the growing line, its refuted N-WARM sibling and an alloc-test that favors
/// leak, as the route of the real run did it: returns the finding, the trend evidence and the
/// alloc-test result. The finding is not confirmed yet.
fn leak_at_the_line(r: &Run) -> (String, String, Value) {
    let t = r.trend();
    let e = evidence_of(&Run::next_of(&t, ADD_PREFIX).unwrap());
    let added = ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            &e,
            "--site",
            SITE,
            "--region",
            "heap",
            "--holder",
            "native-handle/global-ref",
        ],
    );
    let f = added["id"].as_str().unwrap().to_owned();
    ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            &e,
        ],
    );
    let workload = r
        .ledger()
        .iter()
        .filter(|v| v["kind"] == "workload")
        .filter_map(|v| v["evidence_id"].as_str().map(str::to_owned))
        .next_back()
        .unwrap();
    ok(&r.project, &["ledger", "test", &f, "--evidence", &workload]);
    let a = ok(&r.project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "leak", "{a}");
    ok(
        &r.project,
        &[
            "ledger",
            "refute",
            &format!("{f}.n1"),
            "--evidence",
            a["evidence"].as_str().unwrap(),
        ],
    );
    (f, e, a)
}

const CAPTURE: &str = "dla harness run H1 --mode fresh --capture alloc";

fn next_lines(v: &Value) -> Vec<&str> {
    v["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect()
}

#[test]
fn a_finding_at_a_growing_line_reaches_the_verifier_with_its_stack() {
    if !have_python() {
        return;
    }
    let r = prepared();
    let (f, e, a) = leak_at_the_line(&r);
    // The alloc-test leads with the capture and the line that says what to do with its output,
    // and carries the sentence in a key the output bound does not cut.
    let next = next_lines(&a);
    assert_eq!(next[0], CAPTURE, "{next:?}");
    assert!(
        next[1].starts_with("# run the dla summarize alloc-stack-diff command that run prints")
            && next[1].contains(&f)
            && next[1].contains(SITE),
        "{next:?}"
    );
    assert!(
        next[2].starts_with("dla ledger"),
        "the ledger commands follow: {next:?}"
    );
    let hint = a["stacks_hint"].as_str().unwrap();
    assert!(
        hint.contains("a count of blocks allocated at one line") && hint.contains(CAPTURE),
        "{hint}"
    );
    // Without the capture the bundle of the confirmed finding is refused.
    let c = ok(&r.project, &["ledger", "confirm", &f, "--evidence", &e]);
    assert_eq!(c["to"], "confirmed", "{c}");
    let refused = dla(&r.project, &["bundle", &f]);
    assert_ne!(refused.code, 0, "{}", refused.text);
    assert_eq!(
        refused.json["error"]["code"], "stacks-not-captured",
        "{}",
        refused.text
    );
    let fix = refused.json["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains(CAPTURE) && fix.contains("alloc-stack-diff") && fix.contains("dla bundle"),
        "{fix}"
    );
    // Run the capture and the summarize command it prints, as an agent would.
    let cap = ok(
        &r.project,
        &[
            "harness",
            "run",
            "H1",
            "--mode",
            "fresh",
            "--capture",
            "alloc",
        ],
    );
    // The capture alone is not enough: no stack is tied yet, and the refusal says so and names
    // the summarize command of the capture.
    let again = dla(&r.project, &["bundle", &f]);
    assert_eq!(
        again.json["error"]["code"], "stacks-not-captured",
        "{}",
        again.text
    );
    assert!(
        again.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("exists and was not summarized"),
        "{}",
        again.text
    );
    let fix = again.json["error"]["fix"].as_str().unwrap();
    let from_fix = fix.split(" ; ").next().unwrap();
    assert!(
        from_fix.starts_with("dla summarize alloc-stack-diff "),
        "{fix}"
    );
    let cmd = cap["summary"]["summarize"]
        .as_array()
        .unwrap_or_else(|| panic!("{cap}"))
        .iter()
        .filter_map(Value::as_str)
        .find(|c| c.starts_with("dla summarize alloc-stack-diff "))
        .unwrap_or_else(|| panic!("{cap}"));
    assert_eq!(
        cmd, from_fix,
        "the fix is the command the capture run prints"
    );
    let args: Vec<&str> = from_fix.split_whitespace().skip(1).collect();
    let s = ok(&r.project, &args);
    // The stack ends at the line of the finding: the evidence names it at once.
    assert_eq!(s["tied"], serde_json::json!([f]), "{s}");
    let record = r
        .ledger()
        .into_iter()
        .find(|v| v["kind"] == "alloc-stack")
        .unwrap_or_else(|| panic!("{s}"));
    let stack = record["evidence_id"].as_str().unwrap().to_owned();
    assert_eq!(record["result"]["finding"], f.as_str(), "{record}");
    // The bundle now holds the stack record and an excerpt of the growing line.
    let b = ok(&r.project, &["bundle", &f]);
    assert_eq!(b["excerpt"], true, "{b}");
    assert!(
        b["excerpt_frames"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == SITE),
        "{b}"
    );
    assert!(
        b["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| *x == stack),
        "{b}"
    );
    let dir = r.project.join(b["dir"].as_str().unwrap());
    let text = fs::read_to_string(dir.join("excerpt.txt")).unwrap();
    assert!(text.contains("obj = [i]"), "{text}");
    // A second alloc-test no longer offers the capture.
    let a2 = ok(&r.project, &["alloc-test", "H1", "--finding", &f]);
    assert!(next_lines(&a2).iter().all(|n| *n != CAPTURE), "{a2}");
    assert!(a2.get("stacks_hint").is_none(), "{a2}");
}

#[test]
fn a_capture_that_no_stack_of_the_line_came_out_of_is_said_in_the_bundle() {
    if !have_python() {
        return;
    }
    let r = prepared();
    // A real capture of H1 and a real summary of it, made before any finding exists at the
    // line: the stack record names no finding.
    let cap = ok(
        &r.project,
        &[
            "harness",
            "run",
            "H1",
            "--mode",
            "fresh",
            "--capture",
            "alloc",
        ],
    );
    let cmd = cap["summary"]["summarize"]
        .as_array()
        .unwrap_or_else(|| panic!("{cap}"))
        .iter()
        .filter_map(Value::as_str)
        .find(|c| c.starts_with("dla summarize alloc-stack-diff "))
        .unwrap();
    let args: Vec<&str> = cmd.split_whitespace().skip(1).collect();
    ok(&r.project, &args);
    let (f, e, _) = leak_at_the_line(&r);
    ok(&r.project, &["ledger", "confirm", &f, "--evidence", &e]);
    let b = ok(&r.project, &["bundle", &f]);
    assert_eq!(b["excerpt"], false, "{b}");
    let note = b["stacks_note"].as_str().unwrap();
    assert!(
        note.contains("an allocation capture of H1 was summarized and no growing stack ends at"),
        "{note}"
    );
    let dir = r.project.join(b["dir"].as_str().unwrap());
    let index: Value = serde_json::from_slice(&fs::read(dir.join("index.json")).unwrap()).unwrap();
    assert_eq!(index["stacks_note"], note, "{index}");
}

/// A prepared run whose series S1 has a flat step in the site column: from the middle point on,
/// every point holds the value of the point before it, as a series does when one run of the unit
/// was cut, so the counter gate (a rise of at least one at every step) fails the column while its
/// shape stays `steady`.
fn prepared_with_flat_step() -> Run {
    let r = prepared();
    let path = r.run_dir.join("series/S1.json");
    let mut s: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let col = s["columns"]
        .as_array()
        .unwrap()
        .iter()
        .position(|c| c["name"] == COLUMN)
        .unwrap();
    let points = s["points"].as_array_mut().unwrap();
    let old: Vec<Value> = points.iter().map(|p| p["values"][col].clone()).collect();
    let mid = old.len() / 2;
    for (i, p) in points.iter_mut().enumerate().skip(mid) {
        p["values"][col] = old[i - 1].clone();
    }
    fs::write(&path, s.to_string()).unwrap();
    r
}

/// Adds the finding at the line of the column from the trend's own growth evidence; returns its id.
fn add_at_the_line(r: &Run, t: &Value) -> String {
    let e = t["evidence"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .as_str()
        .unwrap();
    let added = ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            e,
            "--site",
            SITE,
            "--region",
            "heap",
            "--holder",
            "native-handle/global-ref",
        ],
    );
    added["id"].as_str().unwrap().to_owned()
}

fn resolve_unmeasured(r: &Run, f: &str, reason: &str) -> Out {
    dla(
        &r.project,
        &[
            "ledger",
            "resolve",
            f,
            "--as",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
        ],
    )
}

/// A line that grew in a run with one flat step (a cut run counted as a unit) has a column that
/// the counter gate fails and whose shape is `steady`. A finding at that line is grown all the
/// same: it cannot be closed unmeasured, `close-rest` leaves it, and the gate's fix is the route
/// that measures it; after the alloc-test it is closed as before.
#[test]
fn a_finding_at_a_steady_line_whose_gate_failed_is_grown_and_cannot_be_closed_as_not_exercised() {
    if !have_python() {
        return;
    }
    let r = prepared_with_flat_step();
    let t = r.trend();
    let f = add_at_the_line(&r, &t);
    for reason in ["not_exercised", "inconclusive"] {
        let o = resolve_unmeasured(&r, &f, reason);
        assert_ne!(o.code, 0, "{reason}: {}", o.text);
        assert_eq!(o.json["error"]["code"], "growth-not-measured", "{}", o.text);
        assert!(
            o.json["error"]["message"]
                .as_str()
                .unwrap()
                .contains(COLUMN),
            "{}",
            o.text
        );
    }
    let out = ok(&r.project, &["ledger", "close-rest"]);
    let left = out["left"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["id"] == f.as_str())
        .unwrap_or_else(|| panic!("{out}"));
    assert!(
        left["why"].as_str().unwrap().contains("its column grew")
            && left["why"]
                .as_str()
                .unwrap()
                .contains(&format!("dla alloc-test H1 --finding {f}")),
        "{left}"
    );
    assert_eq!(out["closed"], 0, "{out}");
    // The final gate's fix for the finding is the route that measures it, not close-rest.
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| m["id"] == f.as_str() && m["kind"] == "finding")
        .unwrap_or_else(|| panic!("{missing:?}"));
    let fix = item["fix"].as_str().unwrap();
    assert!(
        !fix.contains("close-rest") && fix.contains(&format!("dla alloc-test H1 --finding {f}")),
        "{fix}"
    );
    // `dla run status` prints the same fix.
    let status = ok(&r.project, &["run", "status"]);
    let listed = status["blocking"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == f.as_str() && b["kind"] == "finding")
        .unwrap_or_else(|| panic!("{status}"));
    assert_eq!(listed["fix"].as_str().unwrap(), fix, "{listed}");
    // The alloc-test route: the sibling, the test, the alloc-test. The finding is decided by
    // measurement afterwards and closes as before.
    let e = t["evidence"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .as_str()
        .unwrap();
    ok(
        &r.project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            e,
        ],
    );
    let workload = r
        .ledger()
        .iter()
        .filter(|v| v["kind"] == "workload")
        .filter_map(|v| v["evidence_id"].as_str().map(str::to_owned))
        .next_back()
        .unwrap();
    ok(&r.project, &["ledger", "test", &f, "--evidence", &workload]);
    let a = ok(&r.project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["decided_by"], COLUMN, "{a}");
    let closed = resolve_unmeasured(&r, &f, "inconclusive");
    assert_eq!(closed.code, 0, "{}", closed.text);
}

/// A column that is flat at the finding's line changes nothing: the finding is not grown, the
/// gate's fix is a plain `resolve` and `close-rest` closes it, as it did.
#[test]
fn a_flat_site_column_at_a_finding_changes_nothing() {
    if !have_python() {
        return;
    }
    let r = prepared();
    let path = r.run_dir.join("series/S1.json");
    let mut s: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let col = s["columns"]
        .as_array()
        .unwrap()
        .iter()
        .position(|c| c["name"] == COLUMN)
        .unwrap();
    for p in s["points"].as_array_mut().unwrap() {
        p["values"][col] = serde_json::json!(7.0);
    }
    fs::write(&path, s.to_string()).unwrap();
    let t = r.trend();
    let f = add_at_the_line(&r, &t);
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| m["id"] == f.as_str() && m["kind"] == "finding")
        .unwrap_or_else(|| panic!("{missing:?}"));
    // Not the route of a grown finding: the plain resolve of an open finding, as before.
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .starts_with("dla ledger resolve ")
            && !item["fix"].as_str().unwrap().contains("alloc-test"),
        "{item}"
    );
    let out = ok(&r.project, &["ledger", "close-rest"]);
    assert_eq!(out["closed"], 1, "{out}");
    assert!(
        out["left"]
            .as_array()
            .is_none_or(|l| l.iter().all(|x| x["id"] != f.as_str())),
        "{out}"
    );
}

/// Generates a driver for `--probe <finding>`, fills its hooks (`setup` and `unit` are the Python
/// bodies) and returns its harness id.
fn second_driver(r: &Run, finding: &str, setup: &str, unit: &str) -> String {
    let g = ok(
        &r.project,
        &[
            "harness", "gen", "--lang", "python", "--driver", "--probe", finding,
        ],
    );
    let edit = r.project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            setup,
        )
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            unit,
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    g["summary"]["harness"].as_str().unwrap().to_owned()
}

const CACHE_HOOKS: (&str, &str) = (
    "    from shop import cache\n    _state[\"cache\"] = cache",
    "    _state[\"cache\"].put(i)",
);
const STORE_SETUP: &str = "    from shop import store\n    _state[\"store\"] = store";

fn files_in(dir: &Path) -> usize {
    fs::read_dir(dir).map_or(0, |d| d.count())
}

#[test]
fn an_alloc_test_on_a_harness_that_is_not_about_the_finding_is_refused_before_any_run() {
    if !have_python() {
        return;
    }
    let r = prepare(STORE_SWITCH, true);
    // The first harness grew the line: the finding at the line is tied to its series, and its
    // alloc-test works as before.
    let (f, _e, a) = leak_at_the_line(&r);
    assert_eq!(a["favors"], "leak", "{a}");
    assert_eq!(a["decided_by"], COLUMN, "{a}");
    // A second driver for another finding (the table of `shop/cache.py`): its unit never touches
    // the line.
    let h2 = second_driver(&r, "F1", CACHE_HOOKS.0, CACHE_HOOKS.1);
    ok(&r.project, &["harness", "run", &h2, "--mode", "fresh"]);
    let (series, allocs) = (
        files_in(&r.run_dir.join("series")),
        files_in(&r.run_dir.join("alloc")),
    );
    let o = dla(&r.project, &["alloc-test", &h2, "--finding", &f]);
    assert_ne!(o.code, 0, "{}", o.text);
    let e = &o.json["error"];
    assert_eq!(e["code"], "harness-not-about", "{}", o.text);
    assert_eq!(
        e["message"],
        format!(
            "{h2} does not target {f} and nothing it measured is tied to it, so an alloc-test there says nothing about {f}"
        ),
        "{}",
        o.text
    );
    // The fix names the harness that is about the finding (a hook driver, an alloc-test on it)
    // and the driver of its own.
    let fix = e["fix"].as_str().unwrap();
    assert!(
        fix.contains(&format!("dla alloc-test H1 --finding {f}")),
        "{fix}"
    );
    assert!(
        fix.contains(&format!(
            "dla harness gen --lang python --driver --probe {f}"
        )) && fix.contains("dla harness run <H'>")
            && fix.contains("dla trend <S>")
            && fix.contains(&format!("dla alloc-test <H'> --finding {f}")),
        "{fix}"
    );
    // Refused before anything ran: no series, no alloc result, no evidence written.
    assert_eq!(files_in(&r.run_dir.join("series")), series);
    assert_eq!(files_in(&r.run_dir.join("alloc")), allocs);
    // The same command on the harness that grew the line works as it did.
    let again = ok(&r.project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(again["decided_by"], COLUMN, "{again}");
    // Without --finding the command is not refused on any harness.
    let free = ok(&r.project, &["alloc-test", &h2]);
    assert_eq!(free["ok"], true, "{free}");
    // Its evidence names no finding, so the ledger's family check would let it through; the
    // discriminator of a harness that is not about the finding cannot be cited for it, to refute
    // the finding or to confirm its sibling.
    let ev = free["evidence"].as_str().unwrap();
    let sibling = format!("{f}.n1");
    for (verb, target) in [("refute", f.as_str()), ("confirm", sibling.as_str())] {
        let o = dla(&r.project, &["ledger", verb, target, "--evidence", ev]);
        assert_ne!(o.code, 0, "{verb}: {}", o.text);
        assert_eq!(o.json["error"]["code"], "harness-not-about", "{}", o.text);
        assert!(
            o.json["error"]["message"]
                .as_str()
                .unwrap()
                .contains(&format!("{h2} does not target {f}")),
            "{}",
            o.text
        );
    }
}

/// A probe driver of the finding at the line whose unit is `unit`, and the alloc-test on it.
fn probe_driver_alloc_test(unit: &str) -> (Run, String, Value) {
    let r = prepare(STORE_SWITCH, true);
    let (f, _e, _a) = leak_at_the_line(&r);
    let h = second_driver(&r, &f, STORE_SETUP, unit);
    let a = ok(&r.project, &["alloc-test", &h, "--finding", &f]);
    (r, h, a)
}

#[test]
fn a_probe_driver_whose_unit_does_not_reach_the_line_is_inconclusive_and_says_so() {
    if !have_python() {
        return;
    }
    // The unit calls another module: the function of the line never runs.
    let (_r, h, a) = probe_driver_alloc_test("    pass");
    assert_eq!(a["favors"], "inconclusive", "{a}");
    assert_eq!(
        a["basis"],
        format!("the unit of {h} does not reach {SITE}"),
        "{a}"
    );
    let next = a["next"].as_array().unwrap();
    assert!(
        next.iter()
            .filter_map(Value::as_str)
            .any(|n| n.contains("call the function of the line") && n.contains(SITE)),
        "{a}"
    );
}

#[test]
fn a_probe_driver_whose_unit_runs_the_line_and_keeps_nothing_is_non_leak() {
    if !have_python() {
        return;
    }
    let (_r, _h, a) = probe_driver_alloc_test("    _state[\"store\"].pin(i, False)");
    assert_eq!(a["favors"], "non-leak", "{a}");
    let basis = a["basis"].as_str().unwrap();
    assert!(
        basis.contains("ran")
            && basis.contains("none of its blocks stayed")
            && basis.contains(SITE),
        "{a}"
    );
}

#[test]
fn a_probe_driver_whose_unit_keeps_the_object_is_a_leak_by_the_site_column() {
    if !have_python() {
        return;
    }
    let (_r, _h, a) = probe_driver_alloc_test("    _state[\"store\"].pin(i)");
    assert_eq!(a["favors"], "leak", "{a}");
    assert_eq!(a["decided_by"], COLUMN, "{a}");
}
