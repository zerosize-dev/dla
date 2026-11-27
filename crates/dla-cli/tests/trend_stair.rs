//! A column that grew in doubling steps through `dla trend`: a container that doubles its
//! capacity ends any window in a flat stretch about as long as all the growth before it, so a
//! slope gate alone fails it. The gate reads its steps, and the finding the column is tied to
//! then reads `grew` (and is not closed as "its code ran and showed no growth").

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::HarnessManifest;
use serde_json::{Value, json};

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
    c.env("DLA_CONSENT_TIERS", "1,2,3");
    c
}

/// The level of the column of a run of a library whose writer fills a `Vec`: it doubles at each
/// step, the steps come further apart and it ends flat (the first point is the warm-up).
const DOUBLING: [f64; 9] = [
    128_537_961.0,
    254_367_081.0,
    506_025_321.0,
    506_025_321.0,
    1_009_341_801.0,
    1_009_341_801.0,
    1_009_341_801.0,
    1_009_341_801.0,
    1_009_341_977.0,
];

/// The trend of a series whose one column (tied to F1 unless it is a `native.` total), has `ys` at 440 units apart. Returns the
/// output of `dla trend` and the trend file it wrote.
fn trend_of(ys: &[f64], column: &str) -> (Value, Value) {
    let tied = !column.starts_with("native.");
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("shop/reg.py");
    fs::create_dir_all(src.parent().unwrap()).unwrap();
    fs::write(
        &src,
        "class Registry:\n    _by_key = {}\n\n    def put(self, key, value):\n        Registry._by_key[key] = value\n\n\nclass Other:\n    _seen = {}\n\n    def put(self, key, value):\n        Other._seen[key] = value\n",
    )
    .unwrap();
    let o = command(dir.path())
        .args(["run", "init", "--session-id", "t"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
    ] {
        let o = command(dir.path()).args(a).output().unwrap();
        assert!(o.status.success(), "{a:?}");
    }
    let m: HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": ["F1"], "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&run_dir).unwrap();
    let points: Vec<Value> = ys
        .iter()
        .enumerate()
        .map(|(i, y)| json!({"x": 440.0 * (i + 1) as f64, "t_ms": 700 * (i + 1), "values": [y]}))
        .collect();
    fs::create_dir_all(run_dir.join("series")).unwrap();
    fs::write(
        run_dir.join("series/S1.json"),
        json!({
            "schema_version": 1, "id": "S1", "axis": "iteration", "warmup": 1,
            "source": {"kind": "harness", "harness": "H1"},
            "columns": [{"name": column, "unit": "bytes", "kind": "gauge", "region": "heap",
                "scope": "process", "source": "driver", "finding": tied.then_some("F1")}],
            "points": points,
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        run_dir.join("harness/H1/run-S1.json"),
        json!({"series": ["S1"]}).to_string(),
    )
    .unwrap();
    let o = command(dir.path()).args(["trend", "S1"]).output().unwrap();
    let out: Value = serde_json::from_slice(&o.stdout).unwrap();
    let file: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("trend/T1.json")).unwrap()).unwrap();
    (out, file)
}

#[test]
fn a_heap_column_that_doubled_in_steps_passes_as_a_stair_and_its_finding_grew() {
    let (out, file) = trend_of(&DOUBLING, "probe:Registry._by_key");
    let col = &file["columns"][0];
    assert_eq!(col["shape"], "stair", "{col}");
    assert_eq!(col["gate"], "pass", "{col}");
    assert!(
        col["gate_note"]
            .as_str()
            .unwrap()
            .contains("flat end is shorter than the next step"),
        "{col}"
    );
    assert_eq!(out["passed"][0]["shape"], "stair", "{out}");
    let target = &file["targets"][0];
    assert_eq!(target["finding"], "F1", "{file}");
    assert_eq!(target["verdict"], "grew", "{target}");
    assert_eq!(target["column"], "probe:Registry._by_key");
}

#[test]
fn the_same_column_flat_for_longer_than_the_next_step_is_expected_is_not_a_pass() {
    let mut ys = DOUBLING.to_vec();
    ys.extend([1_009_341_977.0; 8]);
    let (_, file) = trend_of(&ys, "probe:Registry._by_key");
    let col = &file["columns"][0];
    assert_eq!(col["gate"], "fail", "{col}");
    assert!(col.get("gate_note").is_none(), "{col}");
    assert_ne!(file["targets"][0]["verdict"], "grew");
}

#[test]
fn a_whole_process_total_passes_as_a_stair_but_ties_to_no_finding() {
    // `native.live_bytes` is a total of the process: the tie rule never ties it to a finding, so
    // the target stays what the coverage says, and the column is a lead for the residual rows and
    // the allocation stacks (the run's `next` lines), not for one finding.
    let (out, file) = trend_of(&DOUBLING, "native.live_bytes");
    assert_eq!(file["columns"][0]["shape"], "stair", "{file}");
    assert_eq!(file["columns"][0]["gate"], "pass", "{file}");
    assert_eq!(out["passed"][0]["column"], "native.live_bytes", "{out}");
    assert!(
        file["targets"]
            .as_array()
            .is_none_or(|t| t.iter().all(|v| v["verdict"] != "grew")),
        "{file}"
    );
}
