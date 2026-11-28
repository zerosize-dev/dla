//! An allocation capture is checked against the process's own readings. The allocator shim writes
//! its record at process exit, after `exit()` freed what a thread-local or a static destructor
//! held, so a capture can show a flat total for a process whose own counters measured growth.
//! `dla summarize` compares the GROWTH of the captures' totals with the growth of the driver's
//! last checkpoint of the same process (kept with the record as `driver`): a series of captures
//! whose total rose by less than half of what the process measured, above a floor, is blind. A
//! blind capture says so with both series of numbers, names the snapshot instead of "more work",
//! and is no capture of its harness for the gate and the ledger text.
//!
//! The records are written by hand (no frames: the stacks need no symbols for the comparison).

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, CoverageManifest, EvidenceKind, Ledger, NewEvidence, Source, Tool,
    coverage::ResidualRow, evidence::EvidenceCoverage, finding::Region,
};
use serde_json::{Value, json};

fn base(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_PROJECT_DIR",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
    ] {
        c.env_remove(var);
    }
    c.arg("--project").arg(project);
    c
}

struct Run {
    dir: tempfile::TempDir,
    run_id: String,
}

impl Run {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let o = base(dir.path())
            .env("DLA_CONSENT_TIERS", "1,2")
            .args(["run", "init", "--session-id", "t", "--entry", "repository"])
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(v["ok"], true, "{v}");
        Run {
            run_id: v["run_id"].as_str().unwrap().to_owned(),
            dir,
        }
    }

    fn run_dir(&self) -> std::path::PathBuf {
        self.dir.path().join(".dla").join(&self.run_id)
    }

    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = base(self.dir.path()).args(args).output().unwrap();
        let v = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
        (o.status.code().unwrap(), v)
    }

    /// Writes the records of a series of captures under `harness/H1/out/p<k>-alloc/` and returns
    /// their paths.
    fn captures(&self, records: &[String]) -> Vec<String> {
        records
            .iter()
            .enumerate()
            .map(|(k, text)| {
                let rel = format!(
                    ".dla/{}/harness/H1/out/p{}-alloc/shim-dump.jsonl",
                    self.run_id,
                    k + 1
                );
                let p = self.dir.path().join(&rel);
                fs::create_dir_all(p.parent().unwrap()).unwrap();
                fs::write(p, text).unwrap();
                rel
            })
            .collect()
    }

    fn summarize(&self, files: &[String]) -> Value {
        let mut args = vec!["summarize", "alloc-stack-diff"];
        args.extend(files.iter().map(String::as_str));
        let (code, v) = self.run(&args);
        assert_eq!(code, 0, "{v}");
        v
    }

    /// The ledger lines of evidence of `kind`.
    fn evidence(&self, kind: &str) -> Vec<Value> {
        fs::read_to_string(self.run_dir().join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .filter(|l| l["type"] == "evidence" && l["kind"] == kind)
            .collect()
    }
}

/// One shim record: the exit total `live`, its peak, and the driver's own reading at the same x.
fn record(x: u64, live: u64, peak: u64, driver: Option<u64>, at: &str) -> String {
    let drv = driver.map_or_else(String::new, |d| {
        format!(
            ",\"driver\":{{\"x\":{x},\"live_bytes\":{d},\"live_blocks\":{}}}",
            d / 200
        )
    });
    format!(
        "{{\"dla_harness\":1,\"phase\":\"exit\",\"at\":\"{at}\",\"pid\":{pid},\"exe\":\"/p/prog\",\"runtime\":\"native\",\"x\":{x},\"t_ms\":10,\"values\":{{\"native.live_bytes\":{live},\"native.live_blocks\":12810,\"native.peak_bytes\":{peak}}},\"raw_sites\":{{\"0xaa\":{{\"live_bytes\":{live},\"live_blocks\":12810,\"allocs\":{x},\"frames\":[]}}}}{drv}}}\n",
        pid = 70 + x
    )
}

/// The four captures of the real case: x doubles, the process's own live bytes rise from 2.68
/// to 6.49 MB, and the exit record holds 2.51 MB at every one while its peak follows the process.
fn blind_series(at: &str) -> Vec<String> {
    [
        (440, 2_678_905u64, 2_681_625u64),
        (880, 3_539_065, 3_541_785),
        (1760, 4_522_105, 4_524_825),
        (3520, 6_488_185, 6_491_081),
    ]
    .iter()
    .map(|(x, driver, peak)| record(*x, 2_510_682, *peak, Some(*driver), at))
    .collect()
}

#[test]
fn a_flat_capture_of_a_process_whose_own_live_bytes_rose_is_blind_and_says_so() {
    let r = Run::new();
    let files = r.captures(&blind_series("exit"));
    let v = r.summarize(&files);
    assert_eq!(v["gate"], "fail", "{v}");
    assert_eq!(v["blind"], true, "{v}");
    let nums = &v["blind_numbers"];
    assert_eq!(nums["driver_live_bytes"][0], 2_678_905, "{v}");
    assert_eq!(nums["driver_live_bytes"][3], 6_488_185, "{v}");
    assert_eq!(nums["shim_live_bytes"], json!(vec![2_510_682u64; 4]), "{v}");
    let needed = v["needed"].as_str().unwrap();
    assert!(!needed.contains("more work"), "{needed}");
    assert!(needed.contains("snapshot"), "{needed}");
    assert!(
        v["reasons"].as_array().unwrap().iter().any(|x| x
            .as_str()
            .is_some_and(|s| s.contains("cannot clear or explain growth the process measured"))),
        "{v}"
    );
    // The stored record says it too: the ledger text and the gate read it there.
    let ev = r.evidence("alloc-stack");
    assert!(ev.iter().any(|e| e["result"]["blind"] == true), "{ev:?}");
}

#[test]
fn a_snapshot_that_is_still_blind_names_what_to_try_instead_of_the_snapshot() {
    let r = Run::new();
    let files = r.captures(&blind_series("snapshot"));
    let v = r.summarize(&files);
    assert_eq!(v["blind"], true, "{v}");
    let needed = v["needed"].as_str().unwrap();
    assert!(needed.contains("outside the shim's view"), "{needed}");
    assert!(needed.contains("alloc-test"), "{needed}");
}

#[test]
fn captures_that_rose_with_the_process_are_not_blind() {
    let r = Run::new();
    let recs: Vec<String> = [
        (440u64, 2_678_905u64),
        (880, 3_539_065),
        (1760, 4_522_105),
        (3520, 6_488_185),
    ]
    .iter()
    // The exit view is a little below the process's own counter, and it rises with it.
    .map(|(x, d)| record(*x, d - 170_000, *d, Some(*d), "exit"))
    .collect();
    let files = r.captures(&recs);
    let v = r.summarize(&files);
    assert!(v.get("blind").is_none(), "{v}");
    assert!(!v["needed"].as_str().unwrap().contains("blind"), "{v}");
    let ev = r.evidence("alloc-stack");
    assert!(
        ev.iter().all(|e| e["result"].get("blind").is_none()),
        "{ev:?}"
    );
}

#[test]
fn captures_with_no_checkpoint_of_the_process_say_the_exit_view_may_miss_state() {
    let r = Run::new();
    let recs: Vec<String> = [(440u64, 100_000u64), (880, 100_000), (1760, 100_000)]
        .iter()
        .enumerate()
        .map(|(i, (x, live))| record(*x, *live, 1_000_000 + 4_000_000 * i as u64, None, "exit"))
        .collect();
    let files = r.captures(&recs);
    let v = r.summarize(&files);
    assert!(v.get("blind").is_none(), "{v}");
    let limits: Vec<String> = r
        .evidence("alloc-stack")
        .iter()
        .flat_map(|e| e["limitations"].as_array().cloned().unwrap_or_default())
        .filter_map(|l| l.as_str().map(str::to_owned))
        .collect();
    assert!(
        limits.iter().any(
            |l| l.contains("not compared with what the process measured")
                && l.contains("freed at exit")
        ),
        "{limits:?}"
    );
    // A peak that rises while the live total does not is a hint, never a verdict.
    assert!(
        limits
            .iter()
            .any(|l| l.starts_with("hint: the shim's peak")),
        "{limits:?}"
    );
}

#[test]
fn a_transient_peak_with_live_flat_in_both_readings_is_not_blind() {
    let r = Run::new();
    let recs: Vec<String> = [(440u64, 1_000_000u64), (880, 1_000_000), (1760, 1_000_000)]
        .iter()
        .enumerate()
        .map(|(i, (x, live))| {
            record(
                *x,
                *live,
                1_000_000 + 4_000_000 * i as u64,
                Some(*live),
                "exit",
            )
        })
        .collect();
    let files = r.captures(&recs);
    let v = r.summarize(&files);
    assert!(v.get("blind").is_none(), "{v}");
}

// ---------------------------------------------------------------------------------------
// downstream: a blind capture is not a capture of its harness
// ---------------------------------------------------------------------------------------

/// A run with a Rust harness series S1 (10 points) and a heap residual row over its threshold
/// with the open item O1; no capture yet.
fn scene() -> Run {
    let r = Run::new();
    fs::create_dir_all(r.run_dir().join("series")).unwrap();
    fs::write(
        r.run_dir().join("series/S1.json"),
        json!({"schema_version": 1, "id": "S1",
               "source": {"kind": "harness", "harness": "H1", "mode": "fresh", "runtime": "rust"},
               "axis": "n", "warmup": 0, "columns": [],
               "points": (0..10)
                   .map(|i| json!({"x": 10_000.0 * f64::from(i) / 9.0, "t_ms": 100 * i, "values": []}))
                   .collect::<Vec<_>>()})
        .to_string(),
    )
    .unwrap();
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    l.open_item_add(
        "residual growth in heap (9.000 bytes/iteration, threshold 1.000) is not explained by findings",
        None,
        "dla gap",
    )
    .unwrap();
    CoverageManifest::update(&r.run_dir(), |m| {
        m.set_residual(vec![ResidualRow {
            series: Some("S1".into()),
            region: Region::Heap,
            total_slope: 9.0,
            attributed_slope: 0.0,
            residual: 9.0,
            unit: "bytes/n".into(),
            threshold: 2.25,
            open_item: Some("O1".parse().unwrap()),
            listed: None,
        }]);
    })
    .unwrap();
    r
}

/// An allocation-stack summary whose source lies under harness `H1`.
fn summary(r: &Run, result: Value) {
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Artifact {
            path: "harness/H1/out/p1-alloc/shim-dump.jsonl".into(),
            sha256: "0".into(),
        },
        tool: Tool {
            name: "shim-stack-groups".into(),
            version: "1".into(),
        },
        authority: Authority::Live,
        kind: EvidenceKind::AllocStack,
        result,
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
}

fn capture_items(r: &Run) -> Vec<Value> {
    let (_, v) = r.run(&["gate", "check", "--final"]);
    let name = v["stage"].as_str().unwrap().to_owned();
    let full: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir().join("gate").join(format!("{name}.json"))).unwrap(),
    )
    .unwrap();
    full["missing"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["kind"] == "residual_capture")
        .cloned()
        .collect()
}

fn resolve(r: &Run) -> (i32, Value) {
    r.run(&[
        "ledger",
        "resolve",
        "O1",
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "not measured further",
    ])
}

#[test]
fn a_blind_summary_is_not_a_capture_of_the_harness_and_the_gate_says_why() {
    let r = scene();
    summary(
        &r,
        json!({"mode": "diff", "blind": true,
               "blind_text": "the capture is blind: the process's own live bytes rose 2.68 MB -> 6.49 MB while the shim's total rose 2.51 MB -> 2.51 MB"}),
    );
    let items = capture_items(&r);
    assert_eq!(items.len(), 1, "a blind capture clears nothing: {items:?}");
    let text = items[0]["description"].as_str().unwrap();
    assert!(
        text.contains("is blind") && text.contains("2.68 MB"),
        "{text}"
    );
    let (code, v) = resolve(&r);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "capture-not-summarized", "{v}");
    assert!(
        v["error"]["message"].as_str().unwrap().contains("blind"),
        "{v}"
    );
    // The same summary without the mark is a capture, as before.
    summary(&r, json!({"mode": "diff"}));
    assert!(capture_items(&r).is_empty());
}
