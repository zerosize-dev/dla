//! `dla ledger close-rest` writes only what a run measured into the remedy of a finding it closes.
//! A target whose best verdict is `executed` (its function ran; the harness had no probe for the
//! holder) read "its code ran and showed no growth": nothing measured the holder. The remedy says
//! the holder was not measured, unless some run did: a probe column made for the finding, or a
//! site or holder column at its site or writers. A capture summary of the harness with no column
//! at the writer lines adds that the captures show no growth there.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, FindingId, HarnessManifest, Ledger, NewEvidence, Source, Tool,
    evidence::EvidenceCoverage,
    series::{ColumnKind, Unit},
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
}

impl R {
    fn ok(&self, args: &[&str]) -> Value {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        assert_eq!(o.status.code(), Some(0), "{args:?}: {text}");
        serde_json::from_str(&text).unwrap_or(Value::Null)
    }
    /// The remedy `close-rest` wrote for `finding`.
    fn remedy_of(&self, finding: &str) -> String {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter(|v| v["type"] == "transition" && v["finding"] == finding)
            .filter_map(|v| v["remedy"].as_str().map(str::to_owned))
            .next_back()
            .unwrap_or_else(|| panic!("no remedy for {finding}"))
    }
}

/// A run whose finding F1 (`shop/cache.py:4`) is a declared target of harness H1, which ran once
/// (series S1, trend T1) and read it `executed`. `column` is the one column of the trend, if any.
fn run_with(column: Option<ColumnTrend>) -> (R, String) {
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
        .args(["run", "init", "--session-id", "t", "--budget", "90s"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let r = R { dir, run_dir };
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
    ] {
        r.ok(a);
    }
    let f1 = {
        let o = command(r.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        String::from_utf8(o.stdout)
            .unwrap()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .find(|l| l["site"] == "shop/cache.py:4")
            .map(|l| l["id"].as_str().unwrap().to_owned())
            .unwrap()
    };
    let m: HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": [f1], "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&r.run_dir).unwrap();
    fs::create_dir_all(r.run_dir.join("series")).unwrap();
    fs::write(
        r.run_dir.join("series/S1.json"),
        json!({"id": "S1", "source": {"kind": "harness", "harness": "H1"},
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
    drop(l);
    TrendReport {
        schema_version: 1,
        id: "T1".into(),
        series: "S1".into(),
        thresholds: GateThresholds::default(),
        columns: column.into_iter().collect(),
        targets: vec![TargetVerdict {
            finding: f1.parse::<FindingId>().unwrap(),
            verdict: Verdict::Executed,
            column: None,
            detail: "function price_for at shop/cache.py:4 was executed".into(),
            declared: true,
            writers: vec![],
        }],
    }
    .save(&r.run_dir)
    .unwrap();
    (r, f1)
}

fn flat_column(name: &str, finding: Option<&str>, site: Option<&str>) -> ColumnTrend {
    ColumnTrend {
        name: name.into(),
        unit: Unit::Count,
        kind: ColumnKind::Gauge,
        region: dla_core::series::SeriesRegion::Heap,
        site: site.map(str::to_owned),
        finding: finding.map(|f| f.parse().unwrap()),
        candidates: vec![],
        n: 8,
        mk: None,
        sen: None,
        halves: None,
        every_delta: None,
        noise: 0.0,
        rise: 0.0,
        shape: TrendShape::Flat,
        gate: GateVerdict::Fail,
        gate_failed: vec![],
        gate_note: None,
        resized: false,
    }
}

#[test]
fn an_executed_target_with_no_measured_holder_is_closed_as_not_measured() {
    let (r, f1) = run_with(None);
    r.ok(&["ledger", "close-rest"]);
    let remedy = r.remedy_of(&f1);
    assert!(
        remedy.contains("its code ran; its holder was not measured (python has no probe for it)"),
        "{remedy}"
    );
    assert!(!remedy.contains("showed no growth"), "{remedy}");
}

#[test]
fn an_executed_target_whose_holder_a_probe_column_measured_says_it_showed_no_growth() {
    let (r, f1) = run_with(Some(flat_column(
        "probe:price_for.cache@shop/cache.py:4",
        Some("F1"),
        Some("shop/cache.py:4"),
    )));
    r.ok(&["ledger", "close-rest"]);
    let remedy = r.remedy_of(&f1);
    assert!(
        remedy.contains("its code ran and showed no growth"),
        "{remedy}"
    );
    assert!(!remedy.contains("not measured"), "{remedy}");
}

#[test]
fn a_capture_summary_with_no_column_at_the_writer_lines_adds_what_the_captures_show() {
    let (r, f1) = run_with(None);
    // A summarized allocation-stack capture of H1.
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Artifact {
            path: "harness/H1/out/p1-alloc/dump.jsonl".into(),
            sha256: "0".into(),
        },
        tool: Tool {
            name: "dla".into(),
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
    drop(l);
    r.ok(&["ledger", "close-rest"]);
    let remedy = r.remedy_of(&f1);
    assert!(
        remedy.contains("its holder was not measured (python has no probe for it) and the captures of H1 show no growth at its writer lines"),
        "{remedy}"
    );
}
