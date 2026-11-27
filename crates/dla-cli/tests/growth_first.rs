//! A heap growth that no finding explains leads the run. `dla trend` writes the residual rows of
//! its series (a model that was never told to run `dla gap --residual` still gets them), the gate
//! then lists the capture of the harness first (`dla gate check --stage classify`, `--final`,
//! `dla run status` and the stop hook's text), `dla trend` prints the capture as its first `next`
//! entry, and `dla harness gen --next` is refused with `capture-first` until the harness has a
//! summarized capture. A harness made for a named finding (`--probe`) is not refused.
//!
//! The series are written by hand (one heap column that rises, nothing that explains it) for a
//! Node and a Python harness, so no runtime is needed.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::{Value, json};

const CAPTURE: &str = "dla harness run H1 --mode fresh --capture alloc,heap";

struct Run {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

fn command(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    c
}

/// A project with two modules that each keep a table: the census lists them, so the queue has
/// candidates (the queue item must come after the capture).
fn project(dir: &Path) {
    for (rel, text) in [
        ("shop/__init__.py", ""),
        (
            "shop/a.py",
            "REG_A = {}\n\n\ndef put_a(k):\n    REG_A[k] = k\n",
        ),
        (
            "shop/b.py",
            "REG_B = {}\n\n\ndef put_b(k):\n    REG_B[k] = k\n",
        ),
    ] {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
}

/// Run init, census, a harness H1 of `runtime` and its series S1: the heap column `heap` rises by
/// `step` bytes per point (1,000 units apart, nine points) and nothing else grows.
fn scene(runtime: &str, heap: &str, step: f64) -> Run {
    let dir = tempfile::tempdir().unwrap();
    project(dir.path());
    let o = command(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2,3")
        .args(["run", "init", "--session-id", "t", "--budget", "3h"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let r = Run { dir, run_dir };
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
    ] {
        r.ok(a);
    }
    let m: dla_core::HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": runtime, "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1000, "key_modes": ["fresh"],
        "agent": if runtime == "node" { "driver-node" } else { "driver-python" },
        "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&r.run_dir).unwrap();
    let column = |name: &str, region: &str| {
        json!({"name": name, "unit": "bytes", "kind": "gauge", "region": region,
               "baseline": true, "after_gc": true, "scope": "process", "source": "driver"})
    };
    let points: Vec<Value> = (0..9)
        .map(|k| {
            json!({"x": 1000.0 * (f64::from(k) + 1.0), "t_ms": 100 * k,
                   "values": [5.0e7 + 3.0e5 * f64::from(k), 1.0e7 + step * f64::from(k)]})
        })
        .collect();
    fs::create_dir_all(r.run_dir.join("series")).unwrap();
    fs::write(
        r.run_dir.join("series/S1.json"),
        json!({"schema_version": 1, "id": "S1",
               "source": {"kind": "harness", "harness": "H1", "mode": "fresh", "runtime": runtime},
               "axis": "iteration", "warmup": 1,
               "columns": [column("os.rss", "process"), column(heap, "heap")],
               "points": points, "events": [], "limitations": []})
        .to_string(),
    )
    .unwrap();
    fs::create_dir_all(r.run_dir.join("harness/H1")).unwrap();
    fs::write(
        r.run_dir.join("harness/H1/run-S1.json"),
        json!({"series": ["S1"]}).to_string(),
    )
    .unwrap();
    r
}

impl Run {
    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let v = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
        (o.status.code().unwrap(), v)
    }

    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }

    /// The items the gate lists, from the full file, for `--stage classify` or `--final`.
    fn items(&self, target: &[&str]) -> Vec<Value> {
        let mut args = vec!["gate", "check"];
        args.extend_from_slice(target);
        let (_, v) = self.run(&args);
        let name = v["stage"].as_str().unwrap().to_owned();
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate").join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }

    /// The heap residual rows of the run's coverage manifest.
    fn heap_rows(&self) -> Vec<Value> {
        let cov: Value =
            serde_json::from_str(&fs::read_to_string(self.run_dir.join("coverage.json")).unwrap())
                .unwrap();
        cov["residual"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter(|r| r["region"] == "heap")
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The stop hook's reason, called as a hook process from another directory.
    fn stop_reason(&self) -> String {
        let away = tempfile::tempdir().unwrap();
        let input = json!({
            "session_id": "0f7c2c1e-0000-4000-8000-000000000001",
            "transcript_path": "/tmp/t.jsonl",
            "cwd": self.dir.path(),
            "permission_mode": "default",
            "hook_event_name": "Stop",
            "stop_hook_active": false,
            "last_assistant_message": "done",
            "background_tasks": [],
            "session_crons": [],
        })
        .to_string();
        let mut c = Command::cargo_bin("dla").unwrap();
        for var in [
            "DLA_RUN",
            "DLA_WORKER",
            "CLAUDE_SESSION_ID",
            "CLAUDE_CODE_SESSION_ID",
        ] {
            c.env_remove(var);
        }
        let o = c
            .current_dir(away.path())
            .args(["gate", "stop-hook"])
            .write_stdin(input)
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
        v["reason"].as_str().unwrap_or_default().to_owned()
    }

    /// Gives harness `h` a summarized capture: a retention record whose file lies under the
    /// harness, as the capture run and `dla summarize heap-graph` leave it.
    fn summarized_capture(&self, h: &str) {
        let dir = self
            .run_dir
            .join("harness")
            .join(h)
            .join("out/p1-fresh/capture/1");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("py-1.retain.json");
        fs::write(
            &file,
            r#"{"dla_pyretain":1,"frames":1,"sites":[{"site":"zz/elsewhere.py:1","blocks_first":0,"blocks_last":50,"found_tracked":0,"found_untracked":0,"kept":0,"note":"no object","chains":[]}],"limitations":[]}"#,
        )
        .unwrap();
        self.ok(&["summarize", "heap-graph", file.to_str().unwrap()]);
    }
}

/// 400 bytes per unit over 8,000 units: far over the threshold and over the 64 KiB floor.
const STEEP: f64 = 4.0e5;

#[test]
fn the_trend_writes_the_heap_residual_row_and_the_capture_leads_everything_node_and_python() {
    for (runtime, heap) in [("node", "node.heap_used"), ("python", "py.traced")] {
        let r = scene(runtime, heap, STEEP);
        assert!(
            r.heap_rows().is_empty(),
            "{runtime}: no row before the trend"
        );
        let t = r.ok(&["trend", "S1"]);
        // The row exists, over its threshold, with its open item; nobody ran `dla gap`.
        let rows = r.heap_rows();
        assert_eq!(rows.len(), 1, "{runtime}: {rows:?}");
        assert_eq!(rows[0]["series"], "S1", "{runtime}: {rows:?}");
        assert!(
            rows[0]["residual"].as_f64() > rows[0]["threshold"].as_f64()
                && rows[0]["open_item"].is_string(),
            "{runtime}: {rows:?}"
        );
        // The trend leads with the capture and says so in `_attention`.
        let next0 = t["next"][0].as_str().unwrap();
        assert!(next0.starts_with(CAPTURE), "{runtime}: {next0}");
        assert!(
            t["_attention"][0].as_str().unwrap().contains("H1"),
            "{runtime}: {t}"
        );
        // The capture item is first in the classify gate and in the final gate, before the queue.
        for target in [&["--stage", "classify"][..], &["--final"][..]] {
            let items = r.items(target);
            assert_eq!(
                items[0]["kind"], "residual_capture",
                "{runtime} {target:?}: {items:?}"
            );
            assert!(
                items[0]["fix"].as_str().unwrap().starts_with(CAPTURE),
                "{runtime}: {}",
                items[0]
            );
            if let Some(q) = items.iter().position(|m| m["kind"] == "queue") {
                assert!(q > 0, "{runtime} {target:?}: {items:?}");
            }
        }
        // `dla run status` and the stop hook's text name it first as well.
        let st = r.ok(&["run", "status"]);
        assert_eq!(
            st["blocking"][0]["kind"], "residual_capture",
            "{runtime}: {st}"
        );
        assert!(
            st["next_command"].as_str().unwrap().starts_with(CAPTURE),
            "{runtime}: {st}"
        );
        let reason = r.stop_reason();
        let first_item = reason.find("\n- ").unwrap_or_else(|| panic!("{reason}"));
        assert!(
            reason[first_item..].starts_with("\n- H1:residual-capture:"),
            "{runtime}: {reason}"
        );
    }
}

/// The `captures.json` of H1 as a capture run leaves it: these `dla summarize` commands.
fn captures_json(r: &Run, summarize: &[&str]) {
    fs::write(
        r.run_dir.join("harness/H1/captures.json"),
        json!({"tool": null, "cost": {"files": 0, "bytes": 0, "ms": 0, "kinds": {}},
               "files": [], "summarize": summarize, "limitations": []})
        .to_string(),
    )
    .unwrap();
}

#[test]
fn a_taken_capture_leads_with_a_summary_that_clears_the_item_not_one_that_does_not() {
    const STACKS: &str = "dla summarize alloc-stack-diff h/p1-alloc/a.json h/p2-alloc/a.json";
    const HEAP: &str = "dla summarize heap-graph h/p1/heap-1.json h/p1/heap-2.json";
    const HIST: &str = "dla summarize type-histogram-diff h/p1/a.json h/p2/a.json";
    const NON_HEAP: &str = "dla summarize non-heap h/p1/threads.json";
    // Only commands whose records never clear the item (a histogram, the non-heap regions): the
    // capture fix as before, not a summary that leaves the item open and drops the capture.
    let r = scene("python", "py.traced", STEEP);
    captures_json(&r, &[HIST, NON_HEAP]);
    let t = r.ok(&["trend", "S1"]);
    assert!(
        t["next"][0].as_str().unwrap().starts_with(CAPTURE),
        "{}",
        t["next"]
    );
    assert!(
        t["_attention"][0]
            .as_str()
            .unwrap()
            .contains("capture H1 first")
    );
    // The allocation-stack summary clears it, wherever it is listed; the heap summary does too.
    for (listed, lead) in [
        (&[HIST, NON_HEAP, STACKS][..], STACKS),
        (&[HIST, HEAP, STACKS][..], STACKS),
        (&[HIST, HEAP][..], HEAP),
    ] {
        let r = scene("python", "py.traced", STEEP);
        captures_json(&r, listed);
        let t = r.ok(&["trend", "S1"]);
        assert_eq!(t["next"][0], lead, "{listed:?}: {}", t["next"]);
        assert!(
            t["_attention"][0]
                .as_str()
                .unwrap()
                .contains("summarize the capture of H1")
        );
    }
}

#[test]
fn a_series_under_the_threshold_gets_a_row_and_no_item() {
    for (runtime, heap) in [("node", "node.heap_used"), ("python", "py.traced")] {
        // A flat heap, and a heap that rises by one byte per unit (8,000 bytes over the series:
        // under the 64 KiB floor below which a residual demands no capture).
        for step in [0.0, 1000.0] {
            let r = scene(runtime, heap, step);
            let t = r.ok(&["trend", "S1"]);
            assert_eq!(r.heap_rows().len(), 1, "{runtime} {step}: row written");
            let next0 = t["next"]
                .as_array()
                .and_then(|n| n.first())
                .and_then(Value::as_str);
            assert!(
                !next0.unwrap_or_default().starts_with(CAPTURE),
                "{runtime} {step}: {t}"
            );
            assert!(t.get("_attention").is_none(), "{runtime} {step}: {t}");
            for target in [&["--stage", "classify"][..], &["--final"][..]] {
                let items = r.items(target);
                assert!(
                    items.iter().all(|m| m["kind"] != "residual_capture"),
                    "{runtime} {step} {target:?}: {items:?}"
                );
            }
        }
    }
}

#[test]
fn the_next_batch_is_refused_until_the_capture_is_summarized_and_a_probe_is_not() {
    let r = scene("python", "py.traced", STEEP);
    r.ok(&["trend", "S1"]);
    let next = ["harness", "gen", "--lang", "python", "--driver", "--next"];
    let (code, v) = r.run(&next);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "capture-first", "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(
        fix.starts_with(CAPTURE) && fix.contains("`dla summarize` commands"),
        "{fix}"
    );
    // A harness for a named finding is not held back.
    let id = {
        let (_, l) = (
            0,
            command(r.dir.path())
                .args(["ledger", "list", "--all"])
                .output()
                .unwrap(),
        );
        String::from_utf8(l.stdout)
            .unwrap()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .find_map(|v| {
                v["id"]
                    .as_str()
                    .filter(|i| i.starts_with('F'))
                    .map(str::to_owned)
            })
            .expect("the census lists a finding")
    };
    let probe = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--probe", &id,
    ]);
    assert_eq!(probe["ok"], true, "{probe}");
    // The same with `--next` beside it: the named finding goes first, it is no batch.
    let both = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--probe", &id, "--next", "1",
    ]);
    assert_eq!(both["ok"], true, "{both}");
    // Still refused: the probe did not take the capture of H1.
    assert_eq!(r.run(&next).1["error"]["code"], "capture-first");
    // With the capture summarized the batch is written.
    r.summarized_capture("H1");
    let (code, v) = r.run(&next);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["ok"], true, "{v}");
}
