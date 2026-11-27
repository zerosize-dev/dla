//! A heap residual above its threshold is not left unresolved before the harness that showed the
//! growth has a summarized capture: the gate lists one item per harness with the capture command,
//! `dla ledger resolve <O> --as unresolved` is refused (`capture-not-summarized`), and both go once
//! a retention or allocation-stack summary of that harness exists: a record whose source lies
//! under `harness/<H>/`, or an allocation-stack record whose raw file lists capture files under
//! it. A row whose unattributed growth over its series is under 64 KiB demands no capture. A Rust
//! harness is held to a capture of allocation stacks (`--capture alloc`); a capture run that
//! recorded it could not write stacks releases it.

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

/// A run with a harness series S1 of `runtime`, a heap residual row over its threshold (9 bytes
/// per unit over 10,000 units, 90,000 bytes) and the open item O1 of the row.
fn scene(runtime: &str) -> Run {
    scene_with(runtime, 9.0, 10_000.0)
}

/// [`scene`] with the residual per unit and the units between the first and the last point of
/// the series; the threshold is a quarter of the residual.
fn scene_with(runtime: &str, residual: f64, units: f64) -> Run {
    scene_points(runtime, residual, units, 10)
}

/// [`scene_with`] with `n` points in the series.
fn scene_points(runtime: &str, residual: f64, units: f64, n: u32) -> Run {
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
               "axis": "n", "warmup": 0, "columns": [],
               // Ten points: a series shorter than the gate needs demands no capture.
               "points": (0..n)
                   .map(|i| json!({"x": units * f64::from(i) / f64::from(n - 1), "t_ms": 100 * i, "values": []}))
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
            total_slope: residual,
            attributed_slope: 0.0,
            residual,
            unit: "bytes/n".into(),
            threshold: residual / 4.0,
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
    // A native harness has no `--capture` (its agent writes no files; the allocation stacks of
    // C and C++ come from `--tool valgrind|heaptrack`, an external tool a run may not have, whose
    // refusal records nothing that could release the item): not held to it.
    let r = scene("native");
    assert!(r.capture_items(&["--final"]).is_empty());
    assert!(r.capture_items(&["--stage", "classify"]).is_empty());
    let (code, v) = r.resolve_unresolved();
    assert_eq!(code, 0, "{v}");
}

#[test]
fn a_rust_harness_is_held_to_a_capture_of_allocation_stacks_only() {
    let r = scene("rust");
    for target in [&["--stage", "classify"][..], &["--final"][..]] {
        let items = r.capture_items(target);
        assert_eq!(items.len(), 1, "{target:?}: {:?}", r.items(target));
        // A Rust run takes `alloc` only: `alloc,heap` makes it print a limitation.
        assert_eq!(
            items[0]["fix"],
            "dla harness run H1 --mode fresh --capture alloc, then the `dla summarize` commands it prints"
        );
    }
    let (code, v) = r.resolve_unresolved();
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "capture-not-summarized", "{v}");
    assert_eq!(
        v["error"]["fix"],
        "dla harness run H1 --mode fresh --capture alloc, then the `dla summarize` commands it prints",
        "{v}"
    );
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("could not write stacks"),
        "the refusal says what releases the item: {v}"
    );
}

/// An allocation-stack summary of the shim dumps of a Rust driver (`dla summarize
/// alloc-stack-diff` on `out/p<k>-alloc/shim-dump.jsonl`): its result file is
/// `histograms/<H>.json`, which lists the dump files it read.
fn summarize_rust_stacks(r: &Run, h: &str) {
    let captures: Vec<Value> = (1..=4)
        .map(|k| {
            json!({"label": format!(".dla/{}/harness/{h}/out/p{k}-alloc/shim-dump.jsonl", r.run_id),
                   "x": 440.0, "pid": 600 + k, "stacks": 3, "sha256": "0"})
        })
        .collect();
    fs::create_dir_all(r.run_dir().join("histograms")).unwrap();
    fs::write(
        r.run_dir().join(format!("histograms/{h}.json")),
        json!({"id": h, "tool": "shim-stack-groups", "captures": captures}).to_string(),
    )
    .unwrap();
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Artifact {
            path: format!("histograms/{h}.json"),
            sha256: "0".into(),
        },
        tool: Tool {
            name: "shim-stack-groups".into(),
            version: "1".into(),
        },
        authority: Authority::Live,
        kind: EvidenceKind::AllocStack,
        result: json!({"captures": 4}),
        raw: Some(format!("histograms/{h}.json")),
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
}

/// The `captures.json` the latest capturing run of `h` wrote, with these limitations.
fn capture_run(r: &Run, h: &str, limitations: &[&str]) {
    let dir = r.run_dir().join("harness").join(h);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("captures.json"),
        json!({"tool": null, "cost": {"files": 0, "bytes": 0, "ms": 0, "kinds": {}},
               "files": [], "summarize": [], "limitations": limitations})
        .to_string(),
    )
    .unwrap();
}

#[test]
fn an_allocation_stack_summary_of_a_rust_drivers_dump_files_clears_it() {
    let r = scene("rust");
    // The dump files of another harness do not count.
    summarize_rust_stacks(&r, "H2");
    assert_eq!(r.capture_items(&["--final"]).len(), 1);
    assert_eq!(
        r.resolve_unresolved().1["error"]["code"],
        "capture-not-summarized"
    );
    summarize_rust_stacks(&r, "H1");
    assert!(r.capture_items(&["--final"]).is_empty());
    assert!(r.capture_items(&["--stage", "classify"]).is_empty());
    let (code, v) = r.resolve_unresolved();
    assert_eq!(code, 0, "{v}");
}

#[test]
fn a_rust_capture_run_that_could_not_write_stacks_releases_it() {
    // Each line `rust_alloc_points` and the command-mode copy of the shim dumps record: no shim
    // in the agent directory, fewer than three allocation-stack runs that left stacks, no
    // measured process that wrote a dump.
    for line in [
        "--capture alloc needs the allocator shim in the harness's agent directory (cc builds it at dla harness gen; this harness has none: generate it again with cc on PATH)",
        "--capture alloc: 0 of 4 allocation-stack runs left shim stacks (dla summarize alloc-stack-diff needs 3 or more; does the program use the system allocator?)",
        "--capture alloc: no measured process wrote allocator shim stacks (is the shim preloaded, and does the program use the system allocator?)",
    ] {
        let r = scene("rust");
        assert_eq!(r.capture_items(&["--final"]).len(), 1);
        capture_run(
            &r,
            "H1",
            &[
                "the allocation-stack run p1-alloc did not complete (log x)",
                line,
            ],
        );
        assert!(r.capture_items(&["--final"]).is_empty(), "{line}");
        assert!(
            r.capture_items(&["--stage", "classify"]).is_empty(),
            "{line}"
        );
        let (code, v) = r.resolve_unresolved();
        assert_eq!(code, 0, "{line}: {v}");
    }
    // Other lines of a capture run that wrote stacks do not release it: a crate with its own
    // global allocator still gets the stacks of the system allocator, and a Rust run that was
    // asked for `heap` as well took the stacks.
    let r = scene("rust");
    capture_run(
        &r,
        "H1",
        &[
            "the crate sets a #[global_allocator]: the shim sees the system allocator only, so the stacks may miss what that allocator serves",
            "--capture: rust harnesses write allocation stacks (alloc) only; the other kinds were not taken",
        ],
    );
    assert_eq!(r.capture_items(&["--final"]).len(), 1);
    assert_eq!(
        r.resolve_unresolved().1["error"]["code"],
        "capture-not-summarized"
    );
}

#[test]
fn an_agent_capture_whose_alloc_kind_wrote_no_file_does_not_release_it() {
    // An agent's `--capture alloc,heap` that wrote the heap dump and no allocation file records
    // `--capture alloc: no file was written`: the dump is still to be summarized.
    let r = scene("node");
    capture_run(&r, "H1", &["--capture alloc: no file was written"]);
    assert_eq!(r.capture_items(&["--final"]).len(), 1);
    assert_eq!(r.capture_items(&["--stage", "classify"]).len(), 1);
    assert_eq!(
        r.resolve_unresolved().1["error"]["code"],
        "capture-not-summarized"
    );
}

/// An allocation-stack record whose result file is `allocs/A2.json`, computed from capture files
/// under the harnesses `from`: its raw file lists them.
fn summarize_allocs(r: &Run, from: &[&str]) {
    let captures: Vec<Value> = from
        .iter()
        .enumerate()
        .map(|(i, h)| {
            json!({"number": i + 1,
                   "label": format!(".dla/{}/harness/{h}/out/p{}-fresh/capture/1/go-1.heap.pb.gz", r.run_id, i + 1),
                   "source": "go-pprof-heap", "groups": 3, "sha256": "0"})
        })
        .collect();
    fs::create_dir_all(r.run_dir().join("allocs")).unwrap();
    fs::write(
        r.run_dir().join("allocs/A2.json"),
        json!({"id": "A2", "tool": "go-pprof-heap", "captures": captures}).to_string(),
    )
    .unwrap();
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Artifact {
            path: "allocs/A2.json".into(),
            sha256: "0".into(),
        },
        tool: Tool {
            name: "go-pprof-heap".into(),
            version: "1".into(),
        },
        authority: Authority::Live,
        kind: EvidenceKind::AllocStack,
        result: json!({"captures": from.len()}),
        raw: Some("allocs/A2.json".into()),
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
}

#[test]
fn an_allocation_stack_summary_of_the_harness_captures_clears_it_wherever_its_file_lies() {
    let r = scene("go");
    // Computed from captures of H2: H1 is still asked for one, and refused.
    summarize_allocs(&r, &["H2"]);
    assert_eq!(r.capture_items(&["--final"]).len(), 1);
    assert_eq!(
        r.resolve_unresolved().1["error"]["code"],
        "capture-not-summarized"
    );
    // Computed from captures of H1 (its result file is allocs/A2.json): cleared.
    summarize_allocs(&r, &["H1"]);
    assert!(r.capture_items(&["--final"]).is_empty());
    assert!(r.capture_items(&["--stage", "classify"]).is_empty());
    let (code, v) = r.resolve_unresolved();
    assert_eq!(code, 0, "{v}");
}

#[test]
fn a_series_too_short_for_the_gate_demands_no_capture_whatever_its_growth() {
    // 250 bytes per unit over 3,520 units, as below, but four points: nothing of the series can
    // pass, so the longer run comes first, not a capture.
    let r = scene_points("node", 250.0, 3520.0, 4);
    assert!(r.capture_items(&["--final"]).is_empty());
    assert!(r.capture_items(&["--stage", "classify"]).is_empty());
    let (code, v) = r.resolve_unresolved();
    assert_eq!(code, 0, "{v}");
    // Eight points are enough.
    let r = scene_points("node", 250.0, 3520.0, 8);
    assert_eq!(r.capture_items(&["--final"]).len(), 1);
}

#[test]
fn a_row_under_the_growth_floor_demands_no_capture() {
    // 0.004 bytes per unit over 3,520 units: 14 bytes.
    let r = scene_with("go", 0.004, 3520.0);
    assert!(r.capture_items(&["--final"]).is_empty());
    assert!(r.capture_items(&["--stage", "classify"]).is_empty());
    let (code, v) = r.resolve_unresolved();
    assert_eq!(code, 0, "{v}");
    // 250 bytes per unit over 3,520 units: 880,000 bytes.
    let r = scene_with("go", 250.0, 3520.0);
    assert_eq!(r.capture_items(&["--final"]).len(), 1);
    assert_eq!(
        r.resolve_unresolved().1["error"]["code"],
        "capture-not-summarized"
    );
}
