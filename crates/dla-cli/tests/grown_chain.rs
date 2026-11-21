//! A measured growth cannot be closed unmeasured. A small Python run: one harness ran and
//! wrote a series whose passed column points at a finding that is not a declared target, queue
//! candidates are pending and budget is left. The chain: the workload blocker is refused,
//! `close-rest` leaves the finding with commands that are accepted as printed, closing it is
//! refused, the final gate lists it, and an alloc-test evidence bound to it lets the rules decide.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, HarnessManifest, Ledger, NewEvidence, Source, Tool,
    evidence::EvidenceCoverage,
    series::{ColumnKind, Unit},
    trend::{ColumnTrend, TrendReport},
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
    fn final_gate(&self) -> Vec<Value> {
        let _ = self.run(&["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }
}

fn evidence(kind: EvidenceKind, result: Value) -> NewEvidence {
    NewEvidence {
        source: Source::Repo {
            commit: "c".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "1".into(),
        },
        authority: Authority::Test,
        kind,
        result,
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    }
}

/// A run on a project with three candidates (F1 the cached function, F2 and F3 two dicts),
/// harness H1 that targets F1 and ran once (series S1, trend T1), and a passed column
/// `probe:Registry._by_key@shop/reg.py:5` that points at F2. Returns the run, the id of the
/// workload evidence and of the growth evidence.
fn run_with_a_passed_column(budget: &str) -> (R, String, String) {
    let dir = tempfile::tempdir().unwrap();
    for (rel, text) in [
        ("shop/__init__.py", ""),
        (
            "shop/cache.py",
            "import functools\n\n\n@functools.lru_cache(None)\ndef price_for(handler, key):\n    return (handler, key)\n",
        ),
        (
            "shop/reg.py",
            "class Registry:\n    _by_key = {}\n\n    def put(self, key, value):\n        Registry._by_key[key] = value\n\n\nclass Bounded:\n    _recent = {}\n\n    def put(self, key, value):\n        Bounded._recent[key] = value\n        if len(Bounded._recent) > 10:\n            Bounded._recent.pop(next(iter(Bounded._recent)))\n",
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
    let r = R { dir, run_dir };
    for a in [
        &["sweep", "--lang", "python"][..],
        &["census", "--lang", "python"][..],
        &["workload", "discover", "--lang", "python"][..],
    ] {
        r.ok(a);
    }
    // The harness and its series.
    let m: HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": ["F1"], "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&r.run_dir).unwrap();
    let series = r.run_dir.join("series");
    fs::create_dir_all(&series).unwrap();
    fs::write(
        series.join("S1.json"),
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
    // The evidence: the workload record of the harness and the growth record of its trend.
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Workload,
        json!({"harness": "H1", "mode": "driver", "runtime": "python", "series": ["S1"]}),
    ))
    .unwrap();
    let workload = l.evidence().last().unwrap().evidence_id.to_string();
    l.add_evidence(evidence(
        EvidenceKind::GrowthSeries,
        json!({"trend": "T1", "series": "S1", "n": 8, "passed": [], "failed": 0, "gate": "pass"}),
    ))
    .unwrap();
    let growth = l.evidence().last().unwrap().evidence_id.to_string();
    drop(l);
    let col = ColumnTrend {
        name: "probe:Registry._by_key@shop/reg.py:5".into(),
        unit: Unit::Count,
        kind: ColumnKind::Gauge,
        region: dla_core::series::SeriesRegion::Heap,
        site: Some("shop/reg.py:5".into()),
        finding: None,
        candidates: vec![dla_core::FindingId::new(2)],
        n: 8,
        mk: None,
        sen: None,
        halves: None,
        every_delta: None,
        noise: 0.0,
        rise: 8.0,
        shape: TrendShape::Steady,
        gate: GateVerdict::Pass,
        gate_failed: vec![],
    };
    TrendReport {
        schema_version: 1,
        id: "T1".into(),
        series: "S1".into(),
        thresholds: GateThresholds::default(),
        columns: vec![col],
        targets: vec![],
    }
    .save(&r.run_dir)
    .unwrap();
    (r, workload, growth)
}

#[test]
fn a_workload_blocker_is_refused_while_a_harness_ran_for_every_runtime_with_candidates() {
    let (r, _, _) = run_with_a_passed_column("3h");
    for reason in [
        "tool_missing",
        "env_unavailable",
        "build_fails",
        "privilege_declined",
    ] {
        let (code, v) = r.run(&[
            "ledger",
            "resolve",
            "intake.workload",
            "--status",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
        ]);
        assert_ne!(code, 0, "{reason}: {v}");
        assert_eq!(v["error"]["code"], "workload-ran", "{v}");
        let msg = v["error"]["message"].as_str().unwrap();
        assert!(msg.contains("harness H1 ran and wrote series S1"), "{msg}");
    }
    // The queue was not closed by it.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["open"], true, "{st}");
}

#[test]
fn a_workload_blocker_is_accepted_when_no_harness_ran() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.py"), "X = {}\n").unwrap();
    let o = command(dir.path())
        .args(["run", "init", "--session-id", "t"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let r = R { dir, run_dir };
    r.ok(&["census", "--lang", "python"]);
    r.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "tool_missing",
        "--remedy",
        "no interpreter",
    ]);
}

#[test]
fn the_chain_from_a_passed_column_to_an_alloc_test_cannot_be_walked_around() {
    let (r, workload, growth) = run_with_a_passed_column("3h");
    let steps = [
        format!(
            "dla ledger add --sibling-of F2 --origin hypothesis --hypothesis N-WARM --refute-if \"growth continues over the extra cycles\" --evidence {growth}"
        ),
        format!("dla ledger test F2 --evidence {workload}"),
        "dla alloc-test H1 --finding F2".to_owned(),
    ];
    // close-rest leaves it (and everything else the queue holds) and prints the commands.
    let out = r.ok(&["ledger", "close-rest"]);
    assert_eq!(out["closed"], 0, "{out}");
    let f2 = out["left"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["id"] == "F2")
        .unwrap_or_else(|| panic!("F2 not left: {out}"));
    let why = f2["why"].as_str().unwrap();
    let mut at = 0;
    for s in &steps {
        let i = why[at..]
            .find(s.as_str())
            .unwrap_or_else(|| panic!("{s} not in order in {why}"));
        at += i + s.len();
    }
    // Closing it is refused with the same commands, whatever the reason.
    for reason in ["inconclusive", "not_exercised"] {
        let (code, v) = r.run(&[
            "ledger",
            "resolve",
            "F2",
            "--as",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
        ]);
        assert_ne!(code, 0, "{reason}: {v}");
        assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
        assert!(
            v["error"]["fix"].as_str().unwrap().contains(&steps[2]),
            "{v}"
        );
    }
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "F2",
        "--as",
        "suspected_static_only",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
    // The final gate lists it, with the alloc-test as the fix.
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| m["id"] == "F2" && m["fix"].as_str().unwrap().contains(&steps[2]))
        .unwrap_or_else(|| panic!("{missing:?}"));
    assert_eq!(item["kind"], "finding", "{item}");
    // The printed commands are accepted as printed, in order.
    r.ok(&[
        "ledger",
        "add",
        "--sibling-of",
        "F2",
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM",
        "--refute-if",
        "growth continues over the extra cycles",
        "--evidence",
        &growth,
    ]);
    r.ok(&["ledger", "test", "F2", "--evidence", &workload]);
    // Until an alloc-test evidence is bound to it, it is still listed (now as a test finding).
    assert!(
        r.final_gate()
            .iter()
            .any(|m| m["id"] == "F2" && m["fix"].as_str().unwrap().contains(&steps[2])),
        "still listed"
    );
    // The alloc-test evidence (written directly: a real alloc-test needs a built harness).
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Discriminator,
        json!({"test": "plateau+purge", "favors": "leak", "supports": ["leak"], "finding": "F2"}),
    ))
    .unwrap();
    drop(l);
    assert!(
        !r.final_gate()
            .iter()
            .any(|m| m["id"] == "F2" && m["fix"].as_str().unwrap().contains(&steps[2])),
        "the item goes with the alloc-test evidence"
    );
    // After it the existing rules decide: an inconclusive close is accepted, tied by it.
    r.ok(&[
        "ledger",
        "resolve",
        "F2",
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "x",
    ]);
}

#[test]
fn an_aborted_run_is_not_held_to_it_and_the_floor_keeps_its_own_rule() {
    let (r, _, _) = run_with_a_passed_column("3h");
    r.ok(&["run", "abort", "--reason", "x"]);
    r.ok(&[
        "ledger",
        "resolve",
        "F2",
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    let (r, _, _) = run_with_a_passed_column("90s");
    // At the floor the time_budget reason is the existing floor rule's.
    r.ok(&[
        "ledger",
        "resolve",
        "F2",
        "--as",
        "unresolved",
        "--reason",
        "time_budget",
        "--remedy",
        "x",
    ]);
}

/// A detector hit at `site` in the run record of series `series` of H1.
fn detector_hit(r: &R, series: &str, site: &str) {
    let p = r.run_dir.join(format!("harness/H1/run-{series}.json"));
    let mut rec: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    rec["detectors"] = json!([{"lib": "node", "site": site, "label": "p1-fresh", "count": 3,
        "sites": [{"site": site, "count": 10}], "text": "MaxListenersExceededWarning"}]);
    fs::write(&p, rec.to_string()).unwrap();
}

#[test]
fn a_detector_hit_that_nothing_measured_is_listed_by_the_final_gate_until_it_is() {
    let (r, _, _) = run_with_a_passed_column("3h");
    // F1 holds the hit's site and no trend gave it a verdict.
    detector_hit(&r, "S1", "shop/cache.py:4");
    let missing = r.final_gate();
    let lead = missing
        .iter()
        .find(|m| m["kind"] == "growth" && m["id"] == "F1")
        .unwrap_or_else(|| panic!("{missing:?}"));
    assert!(
        lead["description"]
            .as_str()
            .unwrap()
            .contains("shop/cache.py:4"),
        "{lead}"
    );
    assert!(
        lead["fix"].as_str().unwrap().contains("--probe F1"),
        "{lead}"
    );
    // A hit with no finding at its site is the hit itself.
    detector_hit(&r, "S1", "shop/nowhere.py:9");
    let missing = r.final_gate();
    assert!(
        missing
            .iter()
            .any(|m| m["kind"] == "growth" && m["id"] == "shop/nowhere.py:9"),
        "{missing:?}"
    );
    // A measurement verdict for the finding at the site clears it.
    detector_hit(&r, "S1", "shop/cache.py:4");
    let t = r.run_dir.join("trend/T1.json");
    let mut doc: Value = serde_json::from_str(&fs::read_to_string(&t).unwrap()).unwrap();
    doc["targets"] =
        json!([{"finding": "F1", "verdict": "constant", "declared": true, "detail": ""}]);
    fs::write(&t, doc.to_string()).unwrap();
    let missing = r.final_gate();
    assert!(
        !missing
            .iter()
            .any(|m| m["kind"] == "growth" && m["id"] == "F1"),
        "{missing:?}"
    );
}

#[test]
fn the_detector_line_of_dla_trend_prints_the_direct_steps_when_the_column_at_the_site_passed() {
    let (r, workload, _) = run_with_a_passed_column("3h");
    // A second series of H1 with a passed column at the hit's site (shop/reg.py:2).
    let points: Vec<Value> = (0..12)
        .map(|i| {
            json!({"x": f64::from(i + 1) * 10.0, "t_ms": i * 100,
                "values": [f64::from(i) * 5.0 + f64::from(i % 3)]})
        })
        .collect();
    fs::write(
        r.run_dir.join("series/S2.json"),
        json!({"schema_version": 1, "id": "S2",
            "source": {"kind": "harness", "harness": "H1", "mode": "fresh", "runtime": "python"},
            "axis": "iteration", "warmup": 1,
            "columns": [{"name": "node.listeners@shop/reg.py:2", "unit": "count", "kind": "gauge",
                "region": "heap", "baseline": false, "after_gc": false, "scope": "process",
                "site": "shop/reg.py:2", "source": "driver"}],
            "points": points, "events": [], "limitations": []})
        .to_string(),
    )
    .unwrap();
    fs::write(
        r.run_dir.join("harness/H1/run-S2.json"),
        json!({"series": ["S2"]}).to_string(),
    )
    .unwrap();
    detector_hit(&r, "S2", "shop/reg.py:2");
    let out = r.ok(&["trend", "S2"]);
    let next = out["next"].as_array().unwrap();
    let hit = next
        .iter()
        .map(|n| n.as_str().unwrap())
        .find(|n| n.starts_with("DETECTOR HIT"))
        .unwrap_or_else(|| panic!("{out}"));
    assert!(!hit.contains("dla harness gen"), "{hit} {out}");
    assert!(
        hit.contains("F2 holds it and its column node.listeners@shop/reg.py:2 passed"),
        "{hit}"
    );
    assert!(hit.contains("dla ledger add --sibling-of F2"), "{hit}");
    assert!(
        hit.contains(&format!("dla ledger test F2 --evidence {workload}")),
        "{hit}"
    );
    assert!(hit.contains("dla alloc-test H1 --finding F2"), "{hit}");
}
