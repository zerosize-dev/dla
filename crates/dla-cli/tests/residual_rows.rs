//! One open item of a region covers the rows it was written for. A heap row of another series, or a
//! row larger than the item recorded, is listed on the item (or re-opens it once it is resolved);
//! a resolution covers a row only when it was made after the row was listed, and cites a capture
//! taken after the largest row's series. The report says how a closed item was resolved, and which
//! rows that resolution does not cover. A row under its threshold needs no item.

use std::{fs, path::Path, thread::sleep, time::Duration};

use assert_cmd::Command;
use dla_core::{
    Authority, CoverageManifest, EvidenceKind, Ledger, NewEvidence, Source, Tool,
    evidence::EvidenceCoverage,
};
use serde_json::{Value, json};

struct Run {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

fn dla(project: &Path) -> Command {
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

impl Run {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let o = dla(dir.path())
            .env("DLA_CONSENT_TIERS", "1,2")
            .args(["run", "init", "--session-id", "t", "--entry", "repository"])
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
        Self { dir, run_dir }
    }
    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = dla(self.dir.path()).args(args).output().unwrap();
        let v = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
        (o.status.code().unwrap(), v)
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }
    /// A harness series of `runtime` with a heap column rising by `per_iteration` bytes per
    /// iteration (9 points, the first is warm-up).
    fn series(&self, id: &str, per_iteration: f64) {
        let columns = json!([{"name": "native.live_bytes", "unit": "bytes", "kind": "gauge",
            "region": "heap", "baseline": false, "after_gc": true, "scope": "process",
            "site": null, "source": "test"}]);
        let points: Vec<Value> = (0..9)
            .map(|k| {
                json!({"x": 1000.0 * (f64::from(k) + 1.0), "t_ms": 50 * k,
                    "values": [1_000_000.0 + per_iteration * 1000.0 * f64::from(k)]})
            })
            .collect();
        let doc = json!({"schema_version": 1, "id": id,
            "source": {"kind": "harness", "harness": "H1", "workload": "W1", "mode": "fresh", "runtime": "rust"},
            "axis": "iteration", "warmup": 1, "columns": columns, "points": points,
            "events": [], "limitations": []});
        fs::create_dir_all(self.run_dir.join("series")).unwrap();
        fs::write(
            self.run_dir.join(format!("series/{id}.json")),
            doc.to_string(),
        )
        .unwrap();
        // The ledger and the files are stamped to the millisecond: keep the order readable.
        sleep(Duration::from_millis(30));
    }
    /// The run record of the run of H1 that wrote `series`, with the digest of its driver.
    fn run_record(&self, series: &str, digest: &str) {
        let dir = self.run_dir.join("harness/H1");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("run-{series}.json")),
            json!({"series": [series], "driver_digest": digest}).to_string(),
        )
        .unwrap();
    }
    /// A summarized allocation-stack capture of harness H1, as `dla summarize` records it.
    fn summarize(&self) {
        let mut l = Ledger::load(&self.run_dir).unwrap();
        l.add_evidence(NewEvidence {
            source: Source::Artifact {
                path: "harness/H1/out/p1-alloc/shim-dump.jsonl".into(),
                sha256: "0".into(),
            },
            tool: Tool {
                name: "dla-alloc-stack".into(),
                version: "1".into(),
            },
            authority: Authority::Live,
            kind: EvidenceKind::AllocStack,
            result: json!({}),
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        sleep(Duration::from_millis(30));
    }
    fn items(&self) -> Vec<Value> {
        let (_, v) = self.run(&["gate", "check", "--final"]);
        let name = v["stage"].as_str().unwrap().to_owned();
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate").join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }
    fn residual_items(&self) -> Vec<String> {
        self.items()
            .iter()
            .filter(|m| m["kind"] == "residual")
            .map(|m| m["description"].as_str().unwrap().to_owned())
            .collect()
    }
    fn report(&self) -> String {
        String::from_utf8(dla(self.dir.path()).arg("report").output().unwrap().stdout).unwrap()
    }
    fn ledger_item_texts(&self) -> Vec<String> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .filter(|l| l["type"] == "open_item")
            .map(|l| l["text"].as_str().unwrap().to_owned())
            .collect()
    }
    fn resolve(&self, id: &str, remedy: &str) -> (i32, Value) {
        self.run(&[
            "ledger",
            "resolve",
            id,
            "--as",
            "unresolved",
            "--reason",
            "inconclusive",
            "--remedy",
            remedy,
        ])
    }
}

#[test]
fn an_item_that_lists_a_larger_row_of_another_series_is_not_resolved_by_the_small_one() {
    let r = Run::new();
    // S1: 0.94 bytes per iteration (under the 64 KiB a capture can attribute); the same harness
    // later writes S2 and S3 of 2 MB per iteration.
    r.series("S1", 0.94);
    r.run_record("S1", "d0");
    let v = r.ok(&["gap", "S1", "--residual"]);
    assert_eq!(v["open_items"]["opened"], json!(["O1"]), "{v}");
    // The capture of the first runs is summarized before the big series exists.
    r.summarize();
    r.series("S2", 2_000_000.0);
    // The driver was edited between the runs: the capture of the first one is of another driver.
    r.run_record("S2", "d1");
    let v = r.ok(&["gap", "S2", "--residual"]);
    assert_eq!(v["open_items"]["opened"], json!([]), "{v}");
    assert_eq!(v["open_items"]["added"], json!(["O1"]), "{v}");
    let next = v["next"].to_string();
    assert!(
        next.contains("largest row is S2") && next.contains("threshold"),
        "the item names the largest row: {next}"
    );
    assert_eq!(r.ledger_item_texts().len(), 1, "one item per region");

    // The summary of S1's capture does not measure S2: resolving O1 is refused, naming S2.
    let (code, v) = r.resolve("O1", "allocator noise");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "capture-not-summarized", "{v}");
    assert!(
        v["error"]["message"].as_str().unwrap().contains("S2"),
        "{v}"
    );

    // A capture taken after S2 is written releases it, and the report says how O1 was closed.
    r.summarize();
    let (code, v) = r.resolve("O1", "the capture of H1 shows the growth is the cache");
    assert_eq!(code, 0, "{v}");
    assert!(r.residual_items().is_empty(), "{:?}", r.items());
    let report = r.report();
    assert!(
        report
            .contains("resolved as inconclusive: the capture of H1 shows the growth is the cache"),
        "{report}"
    );
    assert!(!report.contains("see O1"), "{report}");

    // S3 is a row added after the resolution: O1 does not cover it. The item is re-opened as a new
    // one that names S3, and the final gate lists it.
    r.series("S3", 2_500_000.0);
    r.run_record("S3", "d1");
    let v = r.ok(&["gap", "S3", "--residual"]);
    assert_eq!(v["open_items"]["opened"], json!(["O2"]), "{v}");
    let texts = r.ledger_item_texts();
    assert_eq!(texts.len(), 2);
    assert!(
        texts[1].contains("largest row S3") && texts[1].contains("threshold"),
        "{}",
        texts[1]
    );
    let items = r.residual_items();
    assert_eq!(items.len(), 1, "{items:?}");
    assert!(items[0].contains("heap"), "{items:?}");
    let report = r.report();
    assert!(
        report.contains("S3 heap") && report.contains("see O2"),
        "{report}"
    );
    assert!(report.contains("O1 resolved as inconclusive"), "{report}");

    // S3 was written by the same driver as S2 (the same digest in the run records): the capture
    // taken after S2 is the capture of that driver and releases O2.
    let (code, v) = r.resolve("O2", "the same driver as S2, captured after it");
    assert_eq!(code, 0, "{v}");
    assert!(r.residual_items().is_empty(), "{:?}", r.items());

    // A series with no run record (an alloc-test writes none) is of the newest driver: S5 is
    // covered by the capture of S2 and S3, written long before it.
    r.series("S5", 2_200_000.0);
    let v = r.ok(&["gap", "S5", "--residual"]);
    assert_eq!(v["open_items"]["opened"], json!(["O3"]), "{v}");
    let (code, v) = r.resolve("O3", "the newest driver, captured");
    assert_eq!(code, 0, "{v}");

    // S4 is the series of another driver (its hooks changed): no capture of the old one says
    // anything of it, and O3 cannot be left until the harness is captured again.
    r.series("S4", 3_000_000.0);
    r.run_record("S4", "d2");
    let v = r.ok(&["gap", "S4", "--residual"]);
    assert_eq!(v["open_items"]["opened"], json!(["O4"]), "{v}");
    let (code, v) = r.resolve("O4", "no new capture");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "capture-not-summarized", "{v}");
    r.summarize();
    let (code, v) = r.resolve("O4", "captured after S4");
    assert_eq!(code, 0, "{v}");
}

#[test]
fn a_row_listed_after_the_resolution_is_not_covered_by_it() {
    let r = Run::new();
    r.series("S1", 2_000_000.0);
    r.ok(&["gap", "S1", "--residual"]);
    r.summarize();
    let (code, v) = r.resolve("O1", "the capture shows a cache");
    assert_eq!(code, 0, "{v}");
    assert!(r.residual_items().is_empty());
    // The row is now listed on O1 by a later residual evidence (as a row added to the item after
    // its resolution would be): the resolution was made before it, so it covers nothing.
    r.summarize();
    let last = {
        let l = Ledger::load(&r.run_dir).unwrap();
        l.evidence_in_order().last().unwrap().evidence_id.clone()
    };
    CoverageManifest::update(&r.run_dir, |m| {
        for row in &mut m.residual {
            row.open_item = Some("O1".parse().unwrap());
            row.listed = Some(last.clone());
        }
    })
    .unwrap();
    let items = r.residual_items();
    assert_eq!(items.len(), 1, "{:?}", r.items());
    let report = r.report();
    assert!(report.contains("NOT COVERED"), "{report}");
}

#[test]
fn a_row_under_its_threshold_needs_no_item() {
    let r = Run::new();
    // A flat heap column: nothing unattributed.
    r.series("S1", 0.0);
    let v = r.ok(&["gap", "S1", "--residual"]);
    assert_eq!(v["open_items"]["opened"], json!([]), "{v}");
    assert_eq!(v["open_items"]["added"], json!([]), "{v}");
    assert!(r.ledger_item_texts().is_empty());
    assert!(r.residual_items().is_empty(), "{:?}", r.items());
}

/// A worker raises its items under its own prefix (`rust/O1`) while the rows keep the plain id:
/// the rows are matched to the item all the same, so a second reading of the same series is the
/// same row (kept, not added), and a row of another series that falls under its threshold leaves
/// the item open while the first row is still above.
#[test]
fn a_workers_rows_are_matched_to_its_prefixed_items() {
    let r = Run::new();
    r.ok(&["run", "dispatch", "--workers", "rust,python"]);
    r.series("S1", 2_000_000.0);
    let v = r.ok(&["--worker", "rust", "gap", "S1", "--residual"]);
    assert_eq!(
        v["open_items"]["opened"].as_array().unwrap().len(),
        1,
        "{v}"
    );
    let v = r.ok(&["--worker", "rust", "gap", "S1", "--residual"]);
    assert_eq!(v["open_items"]["added"], json!([]), "the same row: {v}");
    assert_eq!(v["open_items"]["kept"].as_array().unwrap().len(), 1, "{v}");
    r.series("S2", 3_000_000.0);
    let v = r.ok(&["--worker", "rust", "gap", "S2", "--residual"]);
    assert_eq!(v["open_items"]["added"].as_array().unwrap().len(), 1, "{v}");
    r.series("S2", 0.0);
    let v = r.ok(&["--worker", "rust", "gap", "S2", "--residual"]);
    assert!(
        v["limitations"].to_string().contains(
            "stays open: a row of another series listed on it is still above its threshold"
        ),
        "{v}"
    );
}
