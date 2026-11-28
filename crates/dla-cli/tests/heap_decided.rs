//! A growth of the live heap that a harness measured is decided by a measurement while the budget
//! allows one, never by text.
//!
//! * A declared target with no probe whose writer lines ran, in a harness whose live heap column
//!   passed, reads `grew_untied` (a lead): the queue does not settle it, `dla trend` prints the
//!   alloc-test of the harness first, `close-rest` and `resolve` refuse it while the budget is
//!   over its floor and allow it at the floor, and an alloc-test decides it.
//! * The open item of a heap residual row whose series passed a live heap column is not left
//!   unresolved while nothing decided it (no capture that shows the growing stack, no
//!   alloc-test): the refusal names the command that can. An alloc-test on the harness, or the
//!   floor, releases it.
//! * An item with rows of several series above their thresholds is resolved by a remedy that
//!   names each of them.
//! * The gate text of the item says what the capture of its harness already showed, and does not
//!   send the reader to take it again.
//! * `dla trend` names what a bounded print cuts, and prints the live heap column first.
//!
//! The reports are written by hand over a small Python project the census reads.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, CoverageManifest, EvidenceKind, FindingId, HarnessManifest, Ledger, NewEvidence,
    Source, Tool,
    coverage::ResidualRow,
    evidence::EvidenceCoverage,
    finding::Region,
    series::{ColumnKind, SeriesRegion, Unit},
    trend::{ColumnTrend, TargetVerdict, TrendReport, Verdict},
};
use dla_stats::{GateThresholds, GateVerdict, TrendShape};
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

struct R {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
    f1: String,
}

impl R {
    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        (
            o.status.code().unwrap(),
            serde_json::from_str(&text).unwrap_or(Value::Null),
        )
    }

    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }

    /// The state the ledger holds for `id`.
    fn state_of(&self, id: &str) -> String {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter(|v| v["type"] == "transition" && v["finding"] == id)
            .filter_map(|v| v["to"].as_str().map(str::to_owned))
            .next_back()
            .unwrap_or_else(|| "candidate".to_owned())
    }

    fn resolve_o1(&self, reason: &str, remedy: &str) -> (i32, Value) {
        self.run(&[
            "ledger",
            "resolve",
            "O1",
            "--as",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            remedy,
        ])
    }

    fn alloc_test(&self, favors: &str) {
        let mut l = Ledger::load(&self.run_dir).unwrap();
        l.add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "dla-alloc-test".into(),
                version: "1".into(),
            },
            authority: Authority::Test,
            kind: EvidenceKind::Discriminator,
            result: json!({"test": "plateau+purge", "harness": "H1", "finding": self.f1, "favors": favors}),
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
    }
}

fn column(name: &str, region: SeriesRegion, gate: GateVerdict, rise: f64) -> ColumnTrend {
    ColumnTrend {
        name: name.into(),
        unit: Unit::Bytes,
        kind: ColumnKind::Gauge,
        region,
        site: None,
        finding: None,
        candidates: vec![],
        n: 8,
        mk: None,
        sen: None,
        halves: None,
        every_delta: None,
        noise: 0.0,
        rise,
        shape: TrendShape::Stair,
        gate,
        gate_failed: vec![],
        gate_note: None,
        resized: false,
    }
}

/// A run with budget `budget` whose finding F1 (`shop/cache.py:4`, heap) is a declared target of
/// harness H1. H1 ran once (series S1, trend T1: `os.rss` and the live heap column `native.live_bytes`
/// passed, and F1 read `verdict`). The series has a heap residual row over its threshold with the
/// open item O1.
fn scene(budget: &str, verdict: Verdict) -> R {
    let dir = tempfile::tempdir().unwrap();
    for (rel, text) in [
        ("shop/__init__.py", ""),
        (
            "shop/cache.py",
            "import functools\n\n\n@functools.lru_cache(None)\ndef price_for(handler, key):\n    return (handler, key)\n",
        ),
    ] {
        let p = dir.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let o = command(dir.path())
        .args(["run", "init", "--session-id", "t", "--budget", budget])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let mut r = R {
        dir,
        run_dir,
        f1: String::new(),
    };
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
    ] {
        r.ok(a);
    }
    r.f1 = command(r.dir.path())
        .args(["ledger", "list", "--all"])
        .output()
        .map(|o| {
            String::from_utf8(o.stdout)
                .unwrap()
                .lines()
                .filter_map(|l| serde_json::from_str::<Value>(l).ok())
                .find(|l| l["site"] == "shop/cache.py:4")
                .map(|l| l["id"].as_str().unwrap().to_owned())
                .unwrap()
        })
        .unwrap();
    let m: HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": [r.f1], "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&r.run_dir).unwrap();
    fs::create_dir_all(r.run_dir.join("series")).unwrap();
    fs::write(
        r.run_dir.join("series/S1.json"),
        json!({"id": "S1", "source": {"kind": "harness", "harness": "H1", "mode": "fresh"},
            "axis": "n", "columns": [], "points": []})
        .to_string(),
    )
    .unwrap();
    fs::write(
        r.run_dir.join("harness/H1/run-S1.json"),
        json!({"series": ["S1"]}).to_string(),
    )
    .unwrap();
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Repo {
            commit: "c".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "1".into(),
        },
        authority: Authority::Test,
        kind: EvidenceKind::Workload,
        result: json!({"harness": "H1", "mode": "driver", "runtime": "python", "series": ["S1"]}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
    l.open_item_add(
        "unexplained growth: 1117 bytes/iteration in heap (S1); largest row S1; attribute it first: dla harness run H1 --mode fresh --capture alloc; then add a hypothesis or record it unresolved",
        None,
        "dla gap --residual",
    )
    .unwrap();
    drop(l);
    CoverageManifest::update(&r.run_dir, |m| {
        m.set_residual(vec![ResidualRow {
            series: Some("S1".into()),
            region: Region::Heap,
            total_slope: 1117.0,
            attributed_slope: 0.0,
            residual: 1117.0,
            unit: "bytes/n".into(),
            threshold: 710.0,
            open_item: Some("O1".parse().unwrap()),
            listed: None,
        }]);
    })
    .unwrap();
    TrendReport {
        schema_version: 1,
        id: "T1".into(),
        series: "S1".into(),
        thresholds: GateThresholds::default(),
        columns: vec![
            column("os.rss", SeriesRegion::Process, GateVerdict::Pass, 9e6),
            column("os.vm_hwm", SeriesRegion::Process, GateVerdict::Pass, 9e6),
            column("native.peak_bytes", SeriesRegion::Heap, GateVerdict::Pass, 3.4e6),
            column("native.live_bytes", SeriesRegion::Heap, GateVerdict::Pass, 3.44e6),
            column("os.vm_data", SeriesRegion::Process, GateVerdict::Pass, 3.4e6),
            column("os.rss_anon", SeriesRegion::Process, GateVerdict::Pass, 9e6),
        ],
        targets: vec![TargetVerdict {
            finding: r.f1.parse::<FindingId>().unwrap(),
            verdict,
            column: None,
            detail: "the heap of H1 grew (S1 native.live_bytes stair, +3.44 MB) and nothing measured its holder; its writer lines ran 1913 times".into(),
            declared: true,
            writers: vec![],
        }],
    }
    .save(&r.run_dir)
    .unwrap();
    r
}

// ---------------------------------------------------------------------------------------
// Y5: a no-probe target in a harness whose heap grew is a lead
// ---------------------------------------------------------------------------------------

#[test]
fn a_lead_is_not_closed_over_the_floor_and_close_rest_leaves_it() {
    let r = scene("3h", Verdict::GrewUntied);
    let out = r.ok(&["ledger", "close-rest"]);
    assert_eq!(r.state_of(&r.f1), "candidate", "{out}");
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        &r.f1,
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "x",
    ]);
    assert_ne!(code, 0, "{v}");
    let text = v.to_string();
    assert!(
        text.contains("growth-not-measured")
            && text.contains(&format!("dla alloc-test H1 --finding {}", r.f1)),
        "{text}"
    );
}

#[test]
fn a_lead_is_closed_at_the_floor_and_after_an_alloc_test() {
    let r = scene("90s", Verdict::GrewUntied);
    r.ok(&["ledger", "close-rest"]);
    assert_ne!(
        r.state_of(&r.f1),
        "candidate",
        "at the floor the run can finish"
    );
    let r = scene("3h", Verdict::GrewUntied);
    r.alloc_test("non-leak");
    r.ok(&["ledger", "close-rest"]);
    assert_ne!(r.state_of(&r.f1), "candidate", "an alloc-test decided it");
}

#[test]
fn a_target_that_only_executed_is_not_held_by_the_lead_rule() {
    let r = scene("3h", Verdict::Executed);
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        &r.f1,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "its code ran",
    ]);
    // Its second look holds it (`target-pending`), not a measured growth.
    let _ = code;
    assert_ne!(v["error"]["code"], "growth-not-measured", "{v}");
}

#[test]
fn the_remedy_of_a_lead_closed_at_the_floor_names_the_heap_and_never_says_to_write_a_unit() {
    let r = scene("90s", Verdict::GrewUntied);
    r.ok(&["ledger", "close-rest"]);
    let ledger = fs::read_to_string(r.run_dir.join("ledger.jsonl")).unwrap();
    let remedy = ledger
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["type"] == "transition" && v["finding"] == r.f1.as_str())
        .filter_map(|v| v["remedy"].as_str().map(str::to_owned))
        .next_back()
        .unwrap();
    assert!(
        remedy.contains("the heap of H1 grew") && remedy.contains("nothing decided it"),
        "{remedy}"
    );
    assert!(!remedy.contains("write a unit"), "{remedy}");
    assert!(!remedy.contains("show no growth"), "{remedy}");
}

// ---------------------------------------------------------------------------------------
// Y4: a measured heap growth is not closed by text while budget remains
// ---------------------------------------------------------------------------------------

#[test]
fn an_item_of_a_measured_heap_growth_is_not_closed_by_text_over_the_floor() {
    let r = scene("3h", Verdict::GrewUntied);
    let (code, v) = r.resolve_o1("inconclusive", "live heap flat");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "heap-growth-undecided", "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains(&format!("dla alloc-test H1 --finding {}", r.f1)),
        "{fix}"
    );
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("native.live_bytes"),
        "{v}"
    );
}

#[test]
fn without_a_lead_the_refusal_names_the_capture_with_the_snapshot() {
    let r = scene("3h", Verdict::Executed);
    let (_, v) = r.resolve_o1("inconclusive", "x");
    assert_eq!(v["error"]["code"], "heap-growth-undecided", "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(
        fix.starts_with("dla harness run H1 --mode fresh --capture alloc")
            && fix.contains("snapshot"),
        "{fix}"
    );
    // The capture alone cannot release it: the alloc-test of the harness comes after it.
    assert!(fix.ends_with("   ;   dla alloc-test H1"), "{fix}");
}

impl R {
    /// An allocation-stack summary of the capture of H1: flat (no stack grew) or blind.
    fn stack_summary(&self, blind: bool) {
        let result = if blind {
            json!({"blind": true, "blind_text": "the capture is blind: the process's own live bytes rose 2.68 MB -> 6.49 MB while the shim's total rose 2.51 MB -> 2.51 MB"})
        } else {
            json!({"monotone": 0})
        };
        let mut l = Ledger::load(&self.run_dir).unwrap();
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

    /// The `fix` the refusal of resolving O1 names.
    fn refusal_fix(&self) -> String {
        let (_, v) = self.resolve_o1("inconclusive", "x");
        assert_eq!(v["error"]["code"], "heap-growth-undecided", "{v}");
        v["error"]["fix"].as_str().unwrap().to_owned()
    }
}

#[test]
fn after_a_flat_or_blind_capture_the_refusal_names_only_the_alloc_test() {
    // Taking the capture again cannot change what it said; only an alloc-test releases the item.
    for blind in [false, true] {
        let r = scene("3h", Verdict::Executed);
        r.stack_summary(blind);
        assert_eq!(r.refusal_fix(), "dla alloc-test H1", "blind {blind}");
    }
}

#[test]
fn with_leads_the_refusal_names_one_alloc_test_per_lead_and_no_capture() {
    let r = scene("3h", Verdict::GrewUntied);
    assert_eq!(
        r.refusal_fix(),
        format!("dla alloc-test H1 --finding {}", r.f1)
    );
}

#[test]
fn an_alloc_test_with_no_finding_releases_the_item_the_refusal_named() {
    let r = scene("3h", Verdict::Executed);
    let fix = r.refusal_fix();
    let last = fix.rsplit("   ;   ").next().unwrap();
    assert_eq!(last, "dla alloc-test H1");
    // The record `dla alloc-test H1` writes (no finding): its result names the harness.
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Repo {
            commit: "c".into(),
            path: None,
        },
        tool: Tool {
            name: "dla-alloc-test".into(),
            version: "1".into(),
        },
        authority: Authority::Test,
        kind: EvidenceKind::Discriminator,
        result: json!({"test": "plateau+purge", "harness": "H1", "finding": null, "favors": "inconclusive"}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
    drop(l);
    let (code, v) = r.resolve_o1("inconclusive", "decided by the alloc-test");
    assert_eq!(code, 0, "{v}");
}

#[test]
fn a_newer_blind_summary_wins_over_an_older_flat_one_in_the_refusal_and_the_gate_text() {
    let r = scene("3h", Verdict::Executed);
    r.stack_summary(false);
    r.stack_summary(true);
    let (_, v) = r.resolve_o1("inconclusive", "x");
    assert_eq!(v["error"]["code"], "heap-growth-undecided", "{v}");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("is blind") && !msg.contains("no stack grew"),
        "{msg}"
    );
    r.run(&["gate", "check", "--final"]);
    let gate = fs::read_to_string(r.run_dir.join("gate/final.json")).unwrap();
    assert!(
        gate.contains("was summarized and is blind") && !gate.contains("no stack grew across it"),
        "{gate}"
    );
}

#[test]
fn an_alloc_test_on_the_harness_or_the_floor_releases_the_item() {
    let r = scene("3h", Verdict::GrewUntied);
    r.alloc_test("leak");
    let (code, v) = r.resolve_o1("inconclusive", "decided by the alloc-test");
    assert_eq!(code, 0, "{v}");
    let r = scene("90s", Verdict::GrewUntied);
    let (code, v) = r.resolve_o1("inconclusive", "at the floor");
    assert_eq!(code, 0, "{v}");
}

#[test]
fn a_remedy_names_each_series_whose_row_is_above_its_threshold() {
    let r = scene("90s", Verdict::Executed);
    // A second row of another series over its threshold, one under it, on the same item.
    CoverageManifest::update(&r.run_dir, |m| {
        let mut rows = m.residual.clone();
        for (series, residual, threshold) in [("S9", 98.0, 24.0), ("S11", 3.0, 7200.0)] {
            rows.push(ResidualRow {
                series: Some(series.into()),
                region: Region::Heap,
                total_slope: residual,
                attributed_slope: 0.0,
                residual,
                unit: "bytes/n".into(),
                threshold,
                open_item: Some("O1".parse().unwrap()),
                listed: None,
            });
        }
        m.set_residual(rows);
    })
    .unwrap();
    let (code, v) = r.resolve_o1("inconclusive", "S1 shows no stack");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "remedy-misses-series", "{v}");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("S9") && !msg.contains("S11"), "{msg}");
    let (code, v) = r.resolve_o1("inconclusive", "S1 and S9: the floor was reached");
    assert_eq!(code, 0, "{v}");
}

#[test]
fn the_gate_text_of_the_item_says_what_the_capture_of_its_harness_showed() {
    let r = scene("3h", Verdict::GrewUntied);
    let items = |r: &R| -> String {
        r.run(&["gate", "check", "--final"]);
        fs::read_to_string(r.run_dir.join("gate/final.json")).unwrap()
    };
    let before = items(&r);
    assert!(
        before.contains("attribute it first: dla harness run H1"),
        "{before}"
    );
    // The capture was summarized and was blind.
    let mut l = Ledger::load(&r.run_dir).unwrap();
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
        result: json!({"blind": true, "blind_text": "the capture is blind: the process's own live bytes rose 2.68 MB -> 6.49 MB while the shim's total rose 2.51 MB -> 2.51 MB"}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
    drop(l);
    let after = items(&r);
    assert!(!after.contains("attribute it first"), "{after}");
    assert!(
        after.contains("was summarized and is blind") && after.contains("2.68 MB"),
        "{after}"
    );
}

// ---------------------------------------------------------------------------------------
// Y3: outputs that drop what matters say so
// ---------------------------------------------------------------------------------------

#[test]
fn the_trend_of_six_passed_columns_prints_the_live_heap_column_and_the_counts() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    let o = command(p)
        .args(["run", "init", "--session-id", "t", "--budget", "3h"])
        .output()
        .unwrap();
    let init: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = p.join(".dla").join(init["run_id"].as_str().unwrap());
    // Eight points of six rising columns: steady process columns (small p), a heap stair.
    let names = [
        "os.rss",
        "os.rss_anon",
        "os.vm_hwm",
        "native.peak_bytes",
        "native.live_bytes",
        "os.vm_data",
    ];
    fs::create_dir_all(run_dir.join("series")).unwrap();
    let points: Vec<Value> = (0..12)
        .map(|i| {
            let live = 2_700_000.0 + 400_000.0 * f64::from(i / 3);
            json!({"x": 440.0 * f64::from(i + 1), "t_ms": 100 * i, "values": [
                50e6 + 90_000.0 * f64::from(i), 40e6 + 90_000.0 * f64::from(i), 51e6 + 90_000.0 * f64::from(i),
                live, live, 60e6 + live,
            ]})
        })
        .collect();
    fs::write(
        run_dir.join("series/S1.json"),
        json!({"schema_version": 1, "id": "S1",
            "source": {"kind": "harness", "harness": "H1", "mode": "fresh", "runtime": "rust"},
            "axis": "n", "warmup": 0,
            "columns": names.iter().map(|n| json!({"name": n, "unit": "bytes", "kind": "gauge", "scope": "process", "source": "test",
                "region": if n.starts_with("native") { "heap" } else { "process" }})).collect::<Vec<_>>(),
            "points": points})
        .to_string(),
    )
    .unwrap();
    let o = command(p).args(["trend", "S1"]).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    let passed: Vec<&str> = v["passed"]
        .as_array()
        .unwrap_or_else(|| panic!("{text}"))
        .iter()
        .filter_map(|c| c["column"].as_str())
        .collect();
    assert!(passed.contains(&"native.live_bytes"), "{v}");
    // The live heap column leads the passed list, then the other heap column, then the process ones.
    assert_eq!(passed[0], "native.live_bytes", "{v}");
    let by = &v["by_region"];
    assert!(
        by["heap"].as_u64().unwrap() >= 1 && by["process"].as_u64().unwrap() >= 1,
        "{v}"
    );
}

#[test]
fn the_remedy_says_the_captures_could_not_see_the_state_when_the_summary_was_blind() {
    // F1 only ran (no lead); the summarized capture of its harness is blind: close-rest must not
    // write "the captures show no growth at its writer lines".
    let r = scene("90s", Verdict::Executed);
    let mut l = Ledger::load(&r.run_dir).unwrap();
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
        result: json!({"blind": true, "blind_text": "the capture is blind: the process's own live bytes rose 2.68 MB -> 6.49 MB while the shim's total rose 2.51 MB -> 2.51 MB"}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
    drop(l);
    r.ok(&["ledger", "close-rest"]);
    let remedy = remedy_of(&r, &r.f1);
    assert!(
        remedy.contains("could not see the state (blind: the capture is blind")
            && !remedy.contains("show no growth"),
        "{remedy}"
    );
}

fn remedy_of(r: &R, finding: &str) -> String {
    fs::read_to_string(r.run_dir.join("ledger.jsonl"))
        .unwrap()
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["type"] == "transition" && v["finding"] == finding)
        .filter_map(|v| v["remedy"].as_str().map(str::to_owned))
        .next_back()
        .unwrap_or_else(|| panic!("no remedy for {finding}"))
}
