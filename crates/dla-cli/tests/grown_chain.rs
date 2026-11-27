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
        gate_note: None,
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
fn a_target_read_unknown_does_not_make_the_workload_unrunnable() {
    let (r, _, _) = run_with_a_passed_column("3h");
    // One target of the run reads `unknown` (nothing can tell whether its code ran); the
    // others are read. `unknown` is a statement about a target's coverage, not the workload.
    let path = r.run_dir.join("trend/T1.json");
    let mut t: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    t["targets"] = serde_json::json!([
        {"finding": "F998", "verdict": "constant", "detail": "", "declared": true},
        {"finding": "F999", "verdict": "unknown", "detail": "", "declared": true},
    ]);
    fs::write(&path, t.to_string()).unwrap();
    for reason in ["tool_missing", "build_fails"] {
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
    }
}

/// Rewrites the trend of `run_with_a_passed_column` so that its column lists no candidate (the
/// trend ran before the finding of the column was added): only the column's site names it.
fn forget_the_candidates(r: &R) {
    let path = r.run_dir.join("trend/T1.json");
    let mut t: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    t["columns"][0]["candidates"] = json!([]);
    // The column's own site is the declaration of F2.
    t["columns"][0]["site"] = json!("shop/reg.py:2");
    t["columns"][0]["name"] = json!("probe:Registry._by_key@shop/reg.py:2");
    fs::write(&path, t.to_string()).unwrap();
}

fn close_f2(r: &R) -> (i32, Value) {
    r.run(&[
        "ledger",
        "resolve",
        "F2",
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ])
}

#[test]
fn a_column_points_at_the_finding_at_its_site_even_when_the_trend_did_not_list_it() {
    // A finding added from a column after the trend ran is not among the trend's candidates;
    // the resolve must refuse what the final gate will list once the trend is read again.
    let (r, _, _) = run_with_a_passed_column("3h");
    forget_the_candidates(&r);
    let (code, v) = close_f2(&r);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
    assert!(
        r.final_gate().iter().any(|m| m["id"] == "F2"),
        "the gate lists it too"
    );
}

#[test]
fn growth_at_another_site_of_a_measured_leak_may_be_recorded_as_part_of_it() {
    let (r, w, growth) = run_with_a_passed_column("3h");
    // F4 is another finding of the same leak (the same site as F2), under test and with an
    // alloc-test of harness H1 that favored leak.
    let add = r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &growth,
        "--site",
        "shop/reg.py:2",
        "--holder",
        "static/global",
        "--owner",
        "Registry._by_key",
    ]);
    let g = add["id"].as_str().unwrap().to_owned();
    r.ok(&[
        "ledger",
        "add",
        "--sibling-of",
        &g,
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM",
        "--refute-if",
        "growth continues over the extra cycles",
        "--evidence",
        &growth,
    ]);
    r.ok(&["ledger", "test", &g, "--evidence", &w]);
    let mut l = Ledger::load(&r.run_dir).unwrap();
    let discriminator = |harness: &str| {
        evidence(
            EvidenceKind::Discriminator,
            json!({"test": "plateau+purge", "favors": "leak", "supports": ["leak"],
                "finding": g, "harness": harness}),
        )
    };
    // An alloc-test of another harness does not make it the same leak's measurement.
    l.add_evidence(discriminator("H9")).unwrap();
    drop(l);
    let (code, v) = close_f2(&r);
    assert_ne!(code, 0, "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(!fix.contains("dla ledger duplicate"), "{fix}");
    assert!(fix.contains("dla alloc-test H1 --finding F2"), "{fix}");
    // One of H1, on a finding under test: the duplicate route is offered, the alloc-test too.
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(discriminator("H1")).unwrap();
    drop(l);
    let (code, v) = close_f2(&r);
    assert_ne!(code, 0, "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    let dup = format!("dla ledger duplicate F2 --of {g}");
    assert!(fix.contains(&dup), "{fix}");
    assert!(fix.contains("dla alloc-test H1 --finding F2"), "{fix}");
    // close-rest and the final gate offer it too.
    let out = r.ok(&["ledger", "close-rest"]);
    let why = out["left"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["id"] == "F2")
        .map(|l| l["why"].to_string())
        .unwrap_or_default();
    assert!(why.contains(&dup), "{out}");
    // Taking the route settles it: F2 is part of the measured finding and the gate lists nothing
    // for it.
    r.ok(&["ledger", "duplicate", "F2", "--of", &g]);
    assert!(
        !r.final_gate().iter().any(|m| m["id"] == "F2"),
        "{:?}",
        r.final_gate()
    );
}

/// Adds a capture summary to the run of `run_with_a_passed_column`: series S2 (source kind
/// `dump`), its growth evidence and its trend T2 with `n` growing stack columns, the first
/// `tied` to F2. Returns the evidence id.
fn a_capture_summary(r: &R, n: usize, tied: bool) -> String {
    fs::write(
        r.run_dir.join("series/S2.json"),
        json!({"id": "S2", "source": {"kind": "dump"},
            "axis": "n", "columns": [], "points": []})
        .to_string(),
    )
    .unwrap();
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::GrowthSeries,
        json!({"trend": "T2", "series": "S2", "n": 8, "passed": [], "failed": 0, "gate": "pass"}),
    ))
    .unwrap();
    let e = l.evidence().last().unwrap().evidence_id.to_string();
    drop(l);
    let columns: Vec<ColumnTrend> = (0..n)
        .map(|i| ColumnTrend {
            name: format!("stack:pkg/transport.(*http2Server).handle{i}"),
            unit: Unit::Bytes,
            kind: ColumnKind::Gauge,
            region: dla_core::series::SeriesRegion::Heap,
            site: None,
            finding: (tied && i == 0).then(|| dla_core::FindingId::new(2)),
            candidates: vec![],
            n: 8,
            mk: None,
            sen: None,
            halves: None,
            every_delta: None,
            noise: 0.0,
            rise: 100.0 + i as f64,
            shape: TrendShape::Steady,
            gate: GateVerdict::Fail,
            gate_failed: vec![],
            gate_note: None,
        })
        .collect();
    TrendReport {
        schema_version: 1,
        id: "T2".into(),
        series: "S2".into(),
        thresholds: GateThresholds::default(),
        columns,
        targets: vec![],
    }
    .save(&r.run_dir)
    .unwrap();
    e
}

fn items_of_t2(r: &R) -> Vec<Value> {
    r.final_gate()
        .into_iter()
        .filter(|m| m["id"].as_str().is_some_and(|i| i.starts_with("T2:")))
        .collect()
}

#[test]
fn untied_stack_columns_of_one_capture_summary_are_one_action() {
    let (r, _, _) = run_with_a_passed_column("3h");
    let e = a_capture_summary(&r, 30, false);
    let items = items_of_t2(&r);
    assert_eq!(items.len(), 1, "{items:?}");
    let d = items[0]["description"].as_str().unwrap();
    assert!(
        d.contains("30 growing column(s) of the capture summary T2"),
        "{d}"
    );
    // The largest one leads (the rise grows with the index).
    assert!(d.contains("handle29"), "{d}");
    assert!(
        items[0]["fix"].as_str().unwrap().starts_with(&format!(
            "dla ledger add --origin growth-series --evidence {e}"
        )),
        "{items:?}"
    );
    // One add accounts for all of them; the finding is the one obligation left.
    r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &e,
    ]);
    assert!(items_of_t2(&r).is_empty(), "{:?}", items_of_t2(&r));
    let md =
        String::from_utf8(command(r.dir.path()).arg("report").output().unwrap().stdout).unwrap();
    assert!(
        md.contains("Also growing in the same capture: 30 column(s) of T2 tied to no finding"),
        "{md}"
    );
}

#[test]
fn the_stacks_of_a_summary_whose_finding_an_alloc_test_decided_need_nothing() {
    let (r, _, _) = run_with_a_passed_column("3h");
    a_capture_summary(&r, 12, true);
    // F2 is tied to one column; the 11 untied ones wait for the evidence's one action.
    assert_eq!(items_of_t2(&r).len(), 1, "{:?}", items_of_t2(&r));
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Discriminator,
        json!({"test": "plateau+purge", "favors": "leak", "supports": ["leak"],
            "finding": "F2", "harness": "H1"}),
    ))
    .unwrap();
    drop(l);
    // The alloc-test decided F2: the other stacks of that capture are context of it.
    assert!(items_of_t2(&r).is_empty(), "{:?}", items_of_t2(&r));
}

/// The summary trend of `a_capture_summary` shaped like a goroutine capture: the total
/// `metric:goroutine/total` and two non-flat groups with the one evidence, the first tied to F2
/// by `candidates`, the second to nothing.
fn a_summary_with_a_total_and_two_groups(r: &R, alloc_tested: bool) -> String {
    let e = a_capture_summary(r, 0, false);
    let col = |name: &str, candidates: Vec<dla_core::FindingId>| ColumnTrend {
        name: name.into(),
        unit: Unit::Count,
        kind: ColumnKind::Gauge,
        region: dla_core::series::SeriesRegion::Heap,
        site: None,
        finding: None,
        candidates,
        n: 8,
        mk: None,
        sen: None,
        halves: None,
        every_delta: None,
        noise: 0.0,
        rise: 10.0,
        shape: TrendShape::Steady,
        gate: GateVerdict::Fail,
        gate_failed: vec![],
        gate_note: None,
    };
    TrendReport {
        schema_version: 1,
        id: "T2".into(),
        series: "S2".into(),
        thresholds: GateThresholds::default(),
        columns: vec![
            col("metric:goroutine/total", vec![]),
            col("group:a", vec![dla_core::FindingId::new(2)]),
            col("group:b", vec![]),
        ],
        targets: vec![],
    }
    .save(&r.run_dir)
    .unwrap();
    // F2 was decided by an alloc-test and is dispositioned.
    let mut l = Ledger::load(&r.run_dir).unwrap();
    if alloc_tested {
        l.add_evidence(evidence(
            EvidenceKind::Discriminator,
            json!({"test": "plateau+purge", "favors": "leak", "supports": ["leak"],
                "finding": "F2", "harness": "H1"}),
        ))
        .unwrap();
    }
    l.transition(
        &dla_core::FindingId::new(2),
        dla_core::FindingState::Unresolved,
        Some(if alloc_tested {
            dla_core::UnresolvedReason::Inconclusive
        } else {
            dla_core::UnresolvedReason::NotExercised
        }),
        Some("x".into()),
        vec![],
        "t",
    )
    .unwrap();
    e
}

#[test]
fn the_total_of_a_summary_follows_the_capture_rule_for_its_groups() {
    let (r, _, _) = run_with_a_passed_column("3h");
    let e = a_summary_with_a_total_and_two_groups(&r, true);
    // group:a is tied to the alloc-tested F2; group:b has the same evidence and is context of
    // it. Nothing waits for group:b, so the total is accounted for.
    let items = items_of_t2(&r);
    assert!(items.is_empty(), "{items:?}");
    let _ = e;
}

#[test]
fn the_total_of_a_summary_is_listed_with_a_command_when_its_capture_is_not_accounted_for() {
    let (r, _, _) = run_with_a_passed_column("3h");
    // F2 is dispositioned but no alloc-test decided it: group:b is not context of anything.
    let e = a_summary_with_a_total_and_two_groups(&r, false);
    let items = items_of_t2(&r);
    let total = items
        .iter()
        .find(|m| m["id"] == "T2:metric:goroutine/total")
        .unwrap_or_else(|| panic!("{items:?}"));
    let fix = total["fix"].as_str().unwrap();
    assert!(
        fix.starts_with(&format!(
            "dla ledger add --origin growth-series --evidence {e}"
        )),
        "{fix}"
    );
    assert!(!fix.contains("their own trend_column items"), "{fix}");
    // The capture's own item names the same command, so one run clears both.
    assert!(
        items.iter().any(|m| m["id"] == "T2:group:b"
            && m["fix"].as_str().unwrap().starts_with(&format!(
                "dla ledger add --origin growth-series --evidence {e}"
            ))),
        "{items:?}"
    );
    // The total and the capture's item name one command: they are one action.
    let full: Value =
        serde_json::from_str(&fs::read_to_string(r.run_dir.join("gate/final.json")).unwrap())
            .unwrap();
    let actions = full["actions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["id"].as_str().is_some_and(|i| i.starts_with("T2:")))
        .count();
    assert_eq!(actions, 1, "{full}");
    let added = r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &e,
    ]);
    // What is left waits for the disposition of the finding it made. It names the finding and
    // a re-check, never a closing command: the finding's own item gives its route.
    let f = added["id"].as_str().unwrap();
    let items = items_of_t2(&r);
    assert_eq!(items.len(), 1, "{items:?}");
    let fix = items[0]["fix"].as_str().unwrap();
    assert!(
        fix.starts_with("dla gate check --final") && fix.contains(&format!("after {f} has")),
        "{items:?}"
    );
    assert!(!fix.contains("ledger resolve"), "{fix}");
    let all = r.final_gate();
    assert!(
        all.iter().any(|m| m["kind"] == "finding" && m["id"] == f),
        "the finding the total waits for has its own item: {all:?}"
    );
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
    // The alloc-test evidence (written directly: a real alloc-test needs a built harness). It
    // ran on a driver made for F2 (H2 targets it): one on H1, where the growth was measured and
    // which is not about F2 by a target, would not decide it (`a_loop_alloc_test_...` below).
    save_harness(&r, "H2", json!({"targets": ["F2"]}));
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Discriminator,
        json!({"test": "plateau+purge", "favors": "inconclusive", "supports": [], "finding": "F2",
            "harness": "H2"}),
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

/// Saves harness `id` of the run, a Python hook driver with the fields of `extra` set.
fn save_harness(r: &R, id: &str, extra: Value) {
    let mut v = json!({
        "id": id, "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "agent": "driver-python", "timeout_secs": 60,
    });
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    serde_json::from_value::<HarnessManifest>(v)
        .unwrap()
        .save(&r.run_dir)
        .unwrap();
}

/// An alloc-test of F2 on harness `harness` that `favors` (written directly, as above);
/// returns its evidence id.
fn alloc_test_of_f2(r: &R, harness: &str, favors: &str) -> String {
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Discriminator,
        json!({"test": "plateau+purge", "favors": favors, "supports": [], "finding": "F2",
            "harness": harness}),
    ))
    .unwrap();
    l.evidence().last().unwrap().evidence_id.to_string()
}

/// The run of [`run_with_a_passed_column`] whose harness H1 (the one that measured the growth of
/// F2) is the project's own pytest loop, and an alloc-test of F2 on it that `favors`.
fn a_loop_measured_it(budget: &str, favors: &str) -> (R, String) {
    let (r, _, _) = run_with_a_passed_column(budget);
    save_harness(
        &r,
        "H1",
        json!({"tools": [dla_core::harness::PYTEST_TOOL], "targets": []}),
    );
    let e = alloc_test_of_f2(&r, "H1", favors);
    (r, e)
}

fn resolve_f2(r: &R, reason: &str) -> (i32, Value) {
    r.run(&[
        "ledger",
        "resolve",
        "F2",
        "--as",
        "unresolved",
        "--reason",
        reason,
        "--remedy",
        "x",
    ])
}

#[test]
fn a_loop_alloc_test_that_favored_leak_or_said_nothing_does_not_let_the_growth_be_closed() {
    for favors in ["leak", "inconclusive"] {
        let (r, e) = a_loop_measured_it("3h", favors);
        let (code, v) = resolve_f2(&r, "inconclusive");
        assert_ne!(code, 0, "{favors}: {v}");
        assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
        let message = v["error"]["message"].as_str().unwrap();
        assert!(
            message.contains(&format!(
                "the alloc-test {e} on H1 (a test loop, not a driver made for F2) favored {favors}: growth there may be kept by another holder"
            )),
            "{message}"
        );
        assert!(
            v["error"]["missing"]
                .as_str()
                .unwrap()
                .contains("an alloc-test of F2 on a driver made for it"),
            "{v}"
        );
        let fix = v["error"]["fix"].as_str().unwrap();
        assert!(
            fix.contains("dla harness gen --lang python --driver --probe F2"),
            "{fix}"
        );
        assert!(
            fix.contains(
                "dla alloc-test <H'> --finding F2   (on the harness just made, not on H1)"
            ),
            "{fix}"
        );
        assert!(!fix.contains("dla alloc-test H1"), "{fix}");
        // The gate lists it with the same way out.
        let missing = r.final_gate();
        let item = missing
            .iter()
            .find(|m| m["id"] == "F2" && m["fix"].as_str().unwrap().contains("--driver --probe F2"))
            .unwrap_or_else(|| panic!("{missing:?}"));
        assert_eq!(item["kind"], "finding", "{item}");
    }
}

/// An inconclusive alloc-test of F2 on the hook driver H2 (made for it) that ran `ran` extra
/// points and, when `needed` is given, named that window as the one that would decide it
/// (written directly, as above); returns its evidence id.
fn inconclusive_on_a_driver(r: &R, ran: u32, needed: Option<u32>) -> String {
    save_harness(r, "H2", json!({"targets": ["F2"]}));
    let mut result = json!({"test": "plateau+purge", "favors": "inconclusive", "supports": [],
        "finding": "F2", "harness": "H2", "extra": ran, "column": "native.live_bytes"});
    if let Some(n) = needed {
        result["points_needed"] = n.into();
    }
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(evidence(EvidenceKind::Discriminator, result))
        .unwrap();
    l.evidence().last().unwrap().evidence_id.to_string()
}

#[test]
fn an_inconclusive_alloc_test_that_names_a_longer_window_holds_the_close_while_the_budget_pays_for_it()
 {
    let (r, _, _) = run_with_a_passed_column("3h");
    let e = inconclusive_on_a_driver(&r, 8, Some(20));
    let (code, v) = resolve_f2(&r, "inconclusive");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
    let message = v["error"]["message"].as_str().unwrap();
    assert!(
        message.contains(&format!(
            "the alloc-test {e} on H2 was inconclusive: it ran 8 extra points and says 20 would decide it"
        )),
        "{message}"
    );
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(
        fix.starts_with("dla alloc-test H2 --finding F2 --extra 20"),
        "{fix}"
    );
    // The gate lists it with the same first fix (and the Stop hook holds the stop on that list).
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| m["id"] == "F2" && m["fix"].as_str().unwrap().starts_with("dla alloc-test H2"))
        .unwrap_or_else(|| panic!("{missing:?}"));
    assert_eq!(item["kind"], "finding", "{item}");
    // The alloc-test run with that window decides it as before, whatever it favors.
    inconclusive_on_a_driver(&r, 20, Some(20));
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
fn a_longer_window_does_not_hold_the_close_at_the_budget_floor_or_without_points_needed() {
    // At the floor of the budget (5 minutes of a 5 minute budget: under 10) the old rule holds.
    let (r, _, _) = run_with_a_passed_column("5m");
    inconclusive_on_a_driver(&r, 8, Some(20));
    let (code, v) = resolve_f2(&r, "inconclusive");
    assert_eq!(code, 0, "{v}");
    // Evidence of an older version names no window: the close is the model's to make.
    let (r, _, _) = run_with_a_passed_column("3h");
    inconclusive_on_a_driver(&r, 8, None);
    let (code, v) = resolve_f2(&r, "inconclusive");
    assert_eq!(code, 0, "{v}");
}

#[test]
fn a_loop_alloc_test_that_favored_non_leak_lets_the_growth_be_closed() {
    let (r, _) = a_loop_measured_it("3h", "non-leak");
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
fn an_alloc_test_on_a_driver_made_for_the_finding_lets_the_close_be_accepted() {
    let (r, _) = a_loop_measured_it("3h", "leak");
    assert_ne!(resolve_f2(&r, "inconclusive").0, 0);
    save_harness(&r, "H2", json!({"targets": ["F2"]}));
    alloc_test_of_f2(&r, "H2", "inconclusive");
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
fn close_rest_leaves_the_finding_a_loop_alloc_test_did_not_decide() {
    let (r, e) = a_loop_measured_it("3h", "leak");
    let out = r.ok(&["ledger", "close-rest"]);
    let f2 = out["left"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["id"] == "F2")
        .unwrap_or_else(|| panic!("F2 not left: {out}"));
    let why = f2["why"].as_str().unwrap();
    assert!(
        why.contains(&format!(
            "the alloc-test {e} on H1 (a test loop, not a driver made for F2) favored leak"
        )),
        "{why}"
    );
    assert!(why.contains("--driver --probe F2"), "{why}");
    assert!(!why.contains("no alloc-test"), "{why}");
    let v = r.ok(&["run", "status"]);
    assert!(v.to_string().contains("F2"), "{v}");
    let ledger = Ledger::load(&r.run_dir).unwrap();
    let f = ledger.finding(&dla_core::FindingId::new(2)).unwrap();
    assert_eq!(f.state, dla_core::FindingState::Candidate);
}

#[test]
fn the_floor_rule_still_lets_the_time_budget_close_it_and_the_gate_keeps_the_item() {
    let (r, e) = a_loop_measured_it("90s", "leak");
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
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| m["id"] == "F2" && m["kind"] == "growth")
        .unwrap_or_else(|| panic!("{missing:?}"));
    let d = item["description"].as_str().unwrap();
    assert!(
        d.contains(
            "F2 is closed unresolved but its column probe:Registry._by_key@shop/reg.py:5 of S1 grew"
        ),
        "{d}"
    );
    assert!(
        d.contains(&format!(
            "the alloc-test {e} on H1 (a test loop, not a driver made for F2) favored leak: growth there may be kept by another holder"
        )),
        "{d}"
    );
    assert!(!d.contains("no alloc-test decided it"), "{d}");
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .contains("dla harness gen --lang python --driver --probe F2"),
        "{item}"
    );
}

#[test]
fn an_aborted_run_is_not_held_to_it_and_the_floor_keeps_its_own_rule() {
    let (r, _, _) = run_with_a_passed_column("3h");
    r.ok(&["run", "abort", "--blocker", "user", "--reason", "x"]);
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

/// F4: another finding of the leak of F2 (the same site and owner), with a sibling.
fn add_twin_of_f2(r: &R, growth: &str) -> String {
    let add = r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        growth,
        "--site",
        "shop/reg.py:2",
        "--holder",
        "static/global",
        "--owner",
        "Registry._by_key",
    ]);
    add["id"].as_str().unwrap().to_owned()
}

/// An alloc-test of `finding` on `harness` that `favors` (written directly); its evidence id.
fn alloc_test_of(r: &R, finding: &str, harness: &str, favors: &str) -> String {
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Discriminator,
        json!({"test": "plateau+purge", "favors": favors, "supports": [], "finding": finding,
            "harness": harness}),
    ))
    .unwrap();
    l.evidence().last().unwrap().evidence_id.to_string()
}

/// F2 has a driver of its own (H2) whose alloc-test favored leak; its twin F4 has a newer
/// alloc-test on a driver of its own (H3) that favored non-leak. `closed_twin` closes F4 first.
fn a_leak_route_on_f2_and_a_newer_non_leak_on_its_twin(closed_twin: bool) -> (R, String, String) {
    let (r, _, growth) = run_with_a_passed_column("3h");
    let g = add_twin_of_f2(&r, &growth);
    save_harness(&r, "H2", json!({"targets": ["F2"]}));
    let favored = alloc_test_of(&r, "F2", "H2", "leak");
    let (code, v) = resolve_f2(&r, "inconclusive");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "leak-favored", "{v}");
    save_harness(&r, "H3", json!({"targets": [g]}));
    alloc_test_of(&r, &g, "H3", "non-leak");
    if closed_twin {
        r.ok(&[
            "ledger",
            "resolve",
            &g,
            "--as",
            "unresolved",
            "--reason",
            "inconclusive",
            "--remedy",
            "x",
        ]);
    }
    (r, g, favored)
}

#[test]
fn a_merge_that_would_leave_nothing_holding_a_leak_verdict_is_refused_with_the_other_way() {
    for closed_twin in [false, true] {
        let (r, g, d) = a_leak_route_on_f2_and_a_newer_non_leak_on_its_twin(closed_twin);
        // F2 is the finding the leak verdict is about; F4 has none: F2 cannot be the duplicate.
        let (code, v) = r.run(&["ledger", "duplicate", "F2", "--of", &g]);
        assert_ne!(code, 0, "closed {closed_twin}: {v}");
        assert_eq!(v["error"]["code"], "leak-favored", "{v}");
        let message = v["error"]["message"].as_str().unwrap();
        assert!(
            message.contains(&format!("F2 cannot be merged into {g}")) && message.contains(&d),
            "{message}"
        );
        assert_eq!(
            v["error"]["fix"],
            format!("dla ledger duplicate {g} --of F2"),
            "{v}"
        );
        // Nothing was recorded: F2 still has its route and cannot be closed unresolved.
        let (code, v) = resolve_f2(&r, "inconclusive");
        assert_ne!(code, 0, "{v}");
        assert_eq!(v["error"]["code"], "leak-favored", "{v}");
        // The other way is accepted, and the canonical F2 cannot be closed unresolved while the
        // alloc-test on its own driver is the newest word.
        r.ok(&["ledger", "duplicate", &g, "--of", "F2"]);
        let (code, v) = resolve_f2(&r, "inconclusive");
        assert_ne!(code, 0, "closed {closed_twin}: {v}");
        assert_eq!(v["error"]["code"], "leak-favored", "{v}");
        assert!(v["error"]["message"].as_str().unwrap().contains(&d), "{v}");
    }
}

#[test]
fn a_merge_is_free_when_the_canonical_has_a_route_too_or_the_duplicate_has_none() {
    // Both with a route: F2 keeps its own through the canonical's.
    let (r, _, growth) = run_with_a_passed_column("3h");
    let g = add_twin_of_f2(&r, &growth);
    save_harness(&r, "H2", json!({"targets": ["F2"]}));
    alloc_test_of(&r, "F2", "H2", "leak");
    save_harness(&r, "H3", json!({"targets": [g]}));
    alloc_test_of(&r, &g, "H3", "leak");
    r.ok(&["ledger", "duplicate", "F2", "--of", &g]);
    // The duplicate with a non-leak alloc-test has no route to lose.
    let (r, _, growth) = run_with_a_passed_column("3h");
    let g = add_twin_of_f2(&r, &growth);
    save_harness(&r, "H2", json!({"targets": ["F2"]}));
    alloc_test_of(&r, "F2", "H2", "non-leak");
    r.ok(&["ledger", "duplicate", "F2", "--of", &g]);
}

#[test]
fn a_finding_merged_into_a_closed_one_keeps_its_measured_growth_in_the_gate() {
    // F2 is tied to a passed column and nothing decided it. F4, its twin, is closed
    // unresolved first (nothing grows at its own line), then F2 is merged into it.
    let (r, _, growth) = run_with_a_passed_column("3h");
    let g = add_twin_of_f2(&r, &growth);
    r.ok(&[
        "ledger",
        "resolve",
        &g,
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "x",
    ]);
    let listed = |r: &R| -> Vec<String> {
        r.final_gate()
            .iter()
            .filter(|m| m["id"] == "F2" || m["id"] == g.as_str())
            .map(|m| format!("{} {}", m["id"], m["kind"]))
            .collect()
    };
    assert!(!listed(&r).is_empty(), "before the merge F2 is listed");
    r.ok(&["ledger", "duplicate", "F2", "--of", &g]);
    // The leak is counted once, and the growth the passed column measured is still not decided:
    // the canonical carries it, whatever state it was closed in.
    let items = r.final_gate();
    let item = items
        .iter()
        .find(|m| m["kind"] == "growth" && m["id"] == g.as_str())
        .unwrap_or_else(|| panic!("{items:?}"));
    assert!(
        item["description"]
            .as_str()
            .unwrap()
            .contains("no alloc-test decided it"),
        "{item}"
    );
    assert!(
        item["fix"].as_str().unwrap().contains(&g),
        "the fix names the canonical: {item}"
    );
}
