//! A heap residual above its threshold is not left unresolved before the harness that showed the
//! growth has a summarized capture: the gate lists one item per harness with the capture command,
//! `dla ledger resolve <O> --as unresolved` is refused (`capture-not-summarized`), and both go once
//! a retention or allocation-stack record lies under that harness.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, CoverageManifest, EvidenceKind, Ledger, NewEvidence, Source, Tool,
    coverage::ResidualRow, evidence::EvidenceCoverage, finding::Region,
};
use serde_json::{Value, json};

struct Run {
    dir: tempfile::TempDir,
    run_id: String,
}

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

impl Run {
    fn run_dir(&self) -> std::path::PathBuf {
        self.dir.path().join(".dla").join(&self.run_id)
    }
    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = base(self.dir.path()).args(args).output().unwrap();
        let v = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
        (o.status.code().unwrap(), v)
    }
    /// The items the gate lists, from the full file.
    fn items(&self, target: &[&str]) -> Vec<Value> {
        let mut args = vec!["gate", "check"];
        args.extend_from_slice(target);
        let (_, v) = self.run(&args);
        let name = v["stage"].as_str().unwrap().to_owned();
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir().join("gate").join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }
    fn capture_items(&self, target: &[&str]) -> Vec<Value> {
        self.items(target)
            .into_iter()
            .filter(|m| m["kind"] == "residual_capture")
            .collect()
    }
    fn resolve_unresolved(&self) -> (i32, Value) {
        self.run(&[
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
}

/// A run with a harness series S1 of `runtime`, a heap residual row over its threshold and the
/// open item O1 of the row.
fn scene(runtime: &str) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let o = base(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2")
        .args(["run", "init", "--session-id", "t", "--entry", "repository"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let r = Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    };
    fs::create_dir_all(r.run_dir().join("series")).unwrap();
    fs::write(
        r.run_dir().join("series/S1.json"),
        json!({"schema_version": 1, "id": "S1",
               "source": {"kind": "harness", "harness": "H1", "mode": "fresh", "runtime": runtime},
               "axis": "n", "warmup": 0, "columns": [], "points": []})
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
            total_slope: 10.0,
            attributed_slope: 1.0,
            residual: 9.0,
            unit: "bytes/iteration".into(),
            threshold: 1.0,
            open_item: Some("O1".parse().unwrap()),
        }]);
    })
    .unwrap();
    r
}

/// A summarized capture of `kind` whose source lies under harness `h`.
fn summarize(r: &Run, kind: EvidenceKind, h: &str) {
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Artifact {
            path: format!("harness/{h}/out/p1-fresh/capture/1/alloc-1.heapprofile"),
            sha256: "0".into(),
        },
        tool: Tool {
            name: "node-heapprofile".into(),
            version: "1".into(),
        },
        authority: Authority::Live,
        kind,
        result: json!({}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
}

#[test]
fn a_heap_residual_over_its_threshold_needs_a_summarized_capture_before_it_is_left() {
    let r = scene("node");
    for target in [&["--stage", "classify"][..], &["--final"][..]] {
        let items = r.capture_items(target);
        assert_eq!(items.len(), 1, "{target:?}: {:?}", r.items(target));
        let fix = items[0]["fix"].as_str().unwrap();
        assert!(
            fix.starts_with("dla harness run H1 --mode fresh --capture alloc,heap")
                && fix.contains("`dla summarize` commands"),
            "{fix}"
        );
    }
    let (code, v) = r.resolve_unresolved();
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "capture-not-summarized", "{v}");
    assert!(
        v["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("--capture alloc,heap"),
        "{v}"
    );
    // A summary of another harness does not count; one under H1 does, of either kind.
    summarize(&r, EvidenceKind::AllocStack, "H2");
    assert_eq!(r.capture_items(&["--final"]).len(), 1);
    assert_eq!(
        r.resolve_unresolved().1["error"]["code"],
        "capture-not-summarized"
    );
    summarize(&r, EvidenceKind::RetentionPath, "H1");
    assert!(r.capture_items(&["--final"]).is_empty());
    assert!(r.capture_items(&["--stage", "classify"]).is_empty());
    let (code, v) = r.resolve_unresolved();
    assert_eq!(code, 0, "{v}");
}

#[test]
fn an_allocation_stack_summary_under_the_harness_also_clears_it() {
    let r = scene("node");
    summarize(&r, EvidenceKind::AllocStack, "H1");
    assert!(r.capture_items(&["--final"]).is_empty());
    assert_eq!(r.resolve_unresolved().0, 0);
}

#[test]
fn a_runtime_without_capture_is_not_held_to_it() {
    let r = scene("rust");
    assert!(r.capture_items(&["--final"]).is_empty());
    let (code, v) = r.resolve_unresolved();
    assert_eq!(code, 0, "{v}");
}
