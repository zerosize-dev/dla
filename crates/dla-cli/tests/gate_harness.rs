//! A harness `dla harness gen` wrote must be run, or its blocker recorded, before a finding
//! of its runtime is closed with `no_harness`, `not_exercised` or `time_budget` while the
//! budget has room to run it: `dla ledger resolve` refuses (`harness-not-run`) and the final
//! gate lists the harness as missing (`harness_not_run`).

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, Ledger, NewEvidence, NewFinding, Origin, Source, Tool,
    evidence::{EvidenceCoverage, Signal},
    harness::{
        AgentKind, AttemptOutcome, CommandSpec, HarnessAttempt, HarnessManifest, HarnessMode,
    },
    runtime::Runtime,
    series::KeyMode,
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
    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }
    /// `(kind, id)` of every item the final gate lists (the full file).
    fn missing(&self, worker: Option<&str>) -> Vec<(String, String)> {
        let mut args = vec![];
        if let Some(w) = worker {
            args.extend(["--worker", w]);
        }
        args.extend(["gate", "check", "--final"]);
        let (_, shown) = self.run(&args);
        // Worker checks print the first items only; the harness item comes early enough.
        let full = if worker.is_some() {
            shown
        } else {
            serde_json::from_str(
                &fs::read_to_string(self.run_dir().join("gate/final.json")).unwrap(),
            )
            .unwrap()
        };
        full["missing"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                (
                    m["kind"].as_str().unwrap().to_owned(),
                    m["id"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    }
    fn not_run(&self, worker: Option<&str>) -> Vec<String> {
        self.missing(worker)
            .into_iter()
            .filter(|(k, _)| k == "harness_not_run")
            .map(|(_, id)| id)
            .collect()
    }
    /// Closes F1 (of `worker`'s slice) with `reason`; returns the refusal code, if any.
    fn close(&self, worker: Option<&str>, reason: &str) -> Option<String> {
        let mut args = vec![];
        if let Some(w) = worker {
            args.extend(["--worker", w]);
        }
        args.extend([
            "ledger",
            "resolve",
            "F1",
            "--as",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
        ]);
        let (code, v) = self.run(&args);
        if code == 0 {
            None
        } else {
            Some(v["error"]["code"].as_str().unwrap().to_owned())
        }
    }
    fn set_state(&self, f: impl FnOnce(&mut Value)) {
        let path = self.run_dir().join("state.json");
        let mut st: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        f(&mut st);
        fs::write(path, st.to_string()).unwrap();
    }
}

fn evidence(kind: EvidenceKind, result: Value) -> NewEvidence {
    let mut result = result;
    result["signal"] = Signal::new("t", "x", "src").to_value();
    NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "0".into(),
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

fn manifest(runtime: Runtime) -> HarnessManifest {
    manifest_of("H1", runtime)
}

fn manifest_of(id: &str, runtime: Runtime) -> HarnessManifest {
    HarnessManifest {
        schema_version: 1,
        id: id.into(),
        runtime,
        mode: HarnessMode::Driver,
        workload: Some("W1".into()),
        scratch: format!("harness/{id}/work"),
        build: None,
        run: CommandSpec {
            argv: vec!["true".into()],
            cwd: ".".into(),
            env: vec![],
        },
        iter_arg: None,
        warmup: 1,
        iterations: 8,
        step: 2,
        key_modes: vec![KeyMode::Fresh],
        probes: vec![],
        agent: AgentKind::DriverJvm,
        detectors: vec![],
        tools: vec![],
        gaps: vec![],
        timeout_secs: 300,
        ffi: None,
        captures: vec![],
    }
}

/// A run with harness H1 of `runtime` generated (manifest plus the `workload` evidence gen
/// writes) and one candidate F1. With `worker`, both live in that worker's slice.
fn scene(runtime: Runtime, worker: Option<&str>) -> Run {
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
    if let Some(w) = worker {
        r.ok(&["run", "dispatch", "--workers", w]);
    }
    manifest(runtime).save(&r.run_dir()).unwrap();
    unfilled_driver(&r, "H1");
    gen_evidence(&r, worker);
    let mut l = Ledger::load_for(&r.run_dir(), worker).unwrap();
    let e = l.evidence().last().unwrap().evidence_id.clone();
    let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![e]);
    nf.site = Some("a/B.kt:1".into());
    l.add_finding(nf).unwrap();
    r
}

fn gen_evidence(r: &Run, worker: Option<&str>) {
    gen_evidence_of(r, worker, "H1");
}

fn gen_evidence_of(r: &Run, worker: Option<&str>, id: &str) {
    let mut l = Ledger::load_for(&r.run_dir(), worker).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Workload,
        json!({"harness": id, "mode": "driver"}),
    ))
    .unwrap();
}

/// The driver file `dla harness gen` writes, hooks still the template's.
fn unfilled_driver(r: &Run, id: &str) {
    let d = r.run_dir().join(format!("harness/{id}/driver"));
    fs::create_dir_all(&d).unwrap();
    fs::write(
        d.join("DlaHarness.java"),
        "// DLA-HOOK setup: build the services\nthrow new UnsupportedOperationException(\"DLA-HOOK setup has not been filled in\");\n",
    )
    .unwrap();
}

/// Writes a failed attempt for `id` with `reason`.
fn failed_attempt(r: &Run, id: &str, reason: &str) {
    HarnessAttempt {
        harness: id.into(),
        started_at: HarnessAttempt::now(),
        finished_at: HarnessAttempt::now(),
        outcome: AttemptOutcome::Failed,
        reason: Some(reason.to_owned()),
        message: None,
    }
    .save(&r.run_dir())
    .unwrap();
}

fn answer_workload_blocker(r: &Run, worker: Option<&str>, reason: &str) {
    let mut args = vec![];
    if let Some(w) = worker {
        args.extend(["--worker", w]);
    }
    args.extend([
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        reason,
        "--remedy",
        "H1 needs a running Arango",
    ]);
    r.ok(&args);
}

#[test]
fn an_unrun_harness_refuses_the_covered_reasons_and_the_final_gate_lists_it() {
    let r = scene(Runtime::Jvm, None);
    for reason in ["no_harness", "not_exercised", "time_budget"] {
        assert_eq!(
            r.close(None, reason).as_deref(),
            Some("harness-not-run"),
            "{reason}"
        );
    }
    let (_, v) = r.run(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "no_harness",
        "--remedy",
        "x",
    ]);
    let e = &v["error"];
    let msg = e["message"].as_str().unwrap();
    assert!(
        msg.contains("F1 cannot be closed with no_harness")
            && msg.contains("harness H1 (jvm) was generated and never run")
            && msg.contains("budget minutes remain"),
        "{msg}"
    );
    let fix = e["fix"].as_str().unwrap();
    assert!(
        fix.contains("dla harness run H1")
            && fix.contains("--junit")
            && fix.contains("intake.workload")
            && fix.contains("ask the user"),
        "{fix}"
    );
    // The finding is still open: nothing was written.
    assert_eq!(
        Ledger::load(&r.run_dir())
            .unwrap()
            .findings()
            .next()
            .unwrap()
            .state,
        dla_core::FindingState::Candidate
    );
}

#[test]
fn the_reasons_that_name_a_blocker_are_never_refused() {
    for reason in [
        "env_unavailable",
        "build_fails",
        "tool_missing",
        "privilege_declined",
    ] {
        let r = scene(Runtime::Jvm, None);
        assert_eq!(r.close(None, reason), None, "{reason}");
        assert!(r.not_run(None).is_empty(), "{reason}");
    }
}

#[test]
fn a_ledger_closed_before_the_rule_is_listed_by_the_final_gate() {
    let r = scene(Runtime::Jvm, None);
    // Written the way an older binary did: straight to the ledger, no CLI check.
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    l.transition(
        &"F1".parse().unwrap(),
        dla_core::FindingState::Unresolved,
        Some(dla_core::UnresolvedReason::NoHarness),
        Some("x".into()),
        vec![],
        "t",
    )
    .unwrap();
    assert_eq!(r.not_run(None), ["H1"]);
    let items: Value =
        serde_json::from_str(&fs::read_to_string(r.run_dir().join("gate/final.json")).unwrap())
            .unwrap();
    let item = items["missing"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["kind"] == "harness_not_run")
        .unwrap();
    assert!(
        item["description"]
            .as_str()
            .unwrap()
            .contains("harness H1 (jvm) was generated and never run, 1 finding(s)"),
        "{item}"
    );
    // `dla run status` and the Stop hook carry the same list (intake answered, so it is
    // among the first items).
    for item in [
        "intake.target",
        "intake.scope",
        "intake.symptom",
        "intake.runtimes",
        "intake.build",
        "intake.workload",
        "intake.consent",
        "intake.budget",
    ] {
        r.ok(&[
            "ledger",
            "resolve",
            item,
            "--status",
            "answered",
            "--evidence",
            "E1",
        ]);
    }
    let st = r.ok(&["run", "status"]);
    assert!(
        st["blocking"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["id"] == "H1"),
        "{st}"
    );
    let stop = base(r.dir.path())
        .args(["gate", "stop-hook"])
        .write_stdin(json!({"cwd": r.dir.path(), "hook_event_name": "Stop"}).to_string())
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&stop.stdout).unwrap();
    assert_eq!(v["decision"], "block", "{v}");
    let reason = v["reason"].as_str().unwrap();
    assert!(
        reason.contains("- H1: harness H1 (jvm) was generated and never run")
            && reason.contains("dla harness run H1"),
        "{reason}"
    );
}

#[test]
fn no_finding_closed_means_no_missing_item() {
    let r = scene(Runtime::Jvm, None);
    assert!(r.not_run(None).is_empty());
}

#[test]
fn an_attempt_of_any_outcome_clears_the_rule() {
    for outcome in [AttemptOutcome::Ran, AttemptOutcome::Failed] {
        let r = scene(Runtime::Jvm, None);
        HarnessAttempt {
            harness: "H1".into(),
            started_at: HarnessAttempt::now(),
            finished_at: HarnessAttempt::now(),
            outcome,
            reason: (outcome == AttemptOutcome::Failed).then(|| "build_fails".to_owned()),
            message: None,
        }
        .save(&r.run_dir())
        .unwrap();
        assert_eq!(r.close(None, "no_harness"), None, "{outcome:?}");
        assert!(r.not_run(None).is_empty(), "{outcome:?}");
    }
}

#[test]
fn a_run_file_or_a_harness_series_clears_the_rule() {
    let r = scene(Runtime::Jvm, None);
    fs::write(r.run_dir().join("harness/H1/run-S1.json"), "{}").unwrap();
    assert_eq!(r.close(None, "no_harness"), None);

    let r = scene(Runtime::Jvm, None);
    fs::create_dir_all(r.run_dir().join("series")).unwrap();
    fs::write(
        r.run_dir().join("series/S1.json"),
        json!({"schema_version": 1, "id": "S1", "source": {"kind": "harness", "harness": "H1"},
               "axis": "n", "warmup": 0, "columns": [], "points": []})
        .to_string(),
    )
    .unwrap();
    assert_eq!(r.close(None, "not_exercised"), None);
}

#[test]
fn a_blocker_on_intake_workload_after_generation_clears_the_rule() {
    for reason in [
        "env_unavailable",
        "tool_missing",
        "build_fails",
        "privilege_declined",
    ] {
        let r = scene(Runtime::Jvm, None);
        answer_workload_blocker(&r, None, reason);
        assert_eq!(r.close(None, "no_harness"), None, "{reason}");
        assert!(r.not_run(None).is_empty(), "{reason}");
    }
}

#[test]
fn the_same_answer_before_generation_does_not_clear_it() {
    let r = scene(Runtime::Jvm, None);
    // Re-create the order: answer first, then (re)generate H1.
    let dir = tempfile::tempdir().unwrap();
    let o = base(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2")
        .args(["run", "init", "--session-id", "t", "--entry", "repository"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let early = Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    };
    answer_workload_blocker(&early, None, "env_unavailable");
    manifest(Runtime::Jvm).save(&early.run_dir()).unwrap();
    gen_evidence(&early, None);
    let mut l = Ledger::load(&early.run_dir()).unwrap();
    let e = l.evidence().last().unwrap().evidence_id.clone();
    l.add_finding(NewFinding::new(
        Origin::parse("hypothesis").unwrap(),
        vec![e],
    ))
    .unwrap();
    assert_eq!(
        early.close(None, "no_harness").as_deref(),
        Some("harness-not-run")
    );
    // An answer that is not a blocker (answered) does not clear it either.
    let e = r.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "answered",
        "--evidence",
        "E1",
    ]);
    assert_eq!(e["ok"], true);
    assert_eq!(
        r.close(None, "no_harness").as_deref(),
        Some("harness-not-run")
    );
}

#[test]
fn under_the_budget_floor_or_in_a_stopped_run_the_rule_does_not_apply() {
    // 60 minute budget: the floor is 12 minutes (20%).
    let r = scene(Runtime::Jvm, None);
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(11 * 60)));
    assert_eq!(r.close(None, "time_budget"), None, "11 minutes left");

    let r = scene(Runtime::Jvm, None);
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(13 * 60)));
    assert_eq!(
        r.close(None, "time_budget").as_deref(),
        Some("harness-not-run")
    );

    // A short budget: the floor is 10 minutes, not 20%.
    let r = scene(Runtime::Jvm, None);
    r.set_state(|st| {
        st["budget"]["seconds"] = json!(1800);
        st["budget"]["deadline"] = json!(rfc3339_in(9 * 60));
    });
    assert_eq!(r.close(None, "no_harness"), None, "9 of 30 minutes");

    for status in ["aborted", "incomplete"] {
        let r = scene(Runtime::Jvm, None);
        r.set_state(|st| st["status"] = json!(status));
        assert_eq!(r.close(None, "no_harness"), None, "{status}");
    }
}

fn rfc3339_in(secs: u64) -> String {
    (dla_core::state::now_utc() + chrono::Duration::seconds(i64::try_from(secs).unwrap()))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[test]
fn a_finding_of_another_runtime_is_not_held_by_the_harness() {
    let r = scene(Runtime::Jvm, None);
    // F1 cites a census holder in Go: its runtime is go, which has no harness.
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::StaticHit,
        json!({"lang": "go", "site": "a.go:1"}),
    ))
    .unwrap();
    let e = l.evidence().last().unwrap().evidence_id.clone();
    let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![e]);
    nf.site = Some("a.go:1".into());
    l.add_finding(nf).unwrap();
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "F2",
        "--as",
        "unresolved",
        "--reason",
        "no_harness",
        "--remedy",
        "x",
    ]);
    assert_eq!(code, 0, "{v}");
}

#[test]
fn no_manifest_means_no_rule() {
    let r = scene(Runtime::Jvm, None);
    fs::remove_file(r.run_dir().join("harness/H1/harness.json")).unwrap();
    assert_eq!(r.close(None, "no_harness"), None);
    assert!(r.not_run(None).is_empty());
}

#[test]
fn worker_mode_refuses_and_lists_the_harness_of_its_own_runtime() {
    let r = scene(Runtime::Jvm, Some("jvm"));
    assert_eq!(
        r.close(Some("jvm"), "no_harness").as_deref(),
        Some("harness-not-run")
    );
    // Closed by an older binary: the worker's final gate lists it.
    {
        let mut l = Ledger::load_for(&r.run_dir(), Some("jvm")).unwrap();
        l.transition(
            &"jvm/F1".parse().unwrap(),
            dla_core::FindingState::Unresolved,
            Some(dla_core::UnresolvedReason::NoHarness),
            Some("x".into()),
            vec![],
            "t",
        )
        .unwrap();
    }
    assert_eq!(r.not_run(Some("jvm")), ["H1"]);
    // The worker's blocker answer clears it.
    answer_workload_blocker(&r, Some("jvm"), "env_unavailable");
    assert!(r.not_run(Some("jvm")).is_empty());
}

#[test]
fn an_intervention_clone_is_not_a_generated_harness() {
    // `dla intervene` clones H1 into H2 and runs it without `dla harness run`: no
    // attempt, and no series when the effect is invalid. H1 itself was tried.
    let r = scene(Runtime::Jvm, None);
    failed_attempt(&r, "H1", "no_harness");
    manifest_of("H2", Runtime::Jvm).save(&r.run_dir()).unwrap();
    assert_eq!(r.close(None, "not_exercised"), None);
    assert!(r.not_run(None).is_empty());
}

#[test]
fn a_failure_on_a_driver_with_template_hooks_does_not_clear_the_rule() {
    let r = scene(Runtime::Jvm, None);
    failed_attempt(&r, "H1", "hooks_unfilled");
    assert_eq!(
        r.close(None, "no_harness").as_deref(),
        Some("harness-not-run")
    );
    // Once the hooks are filled in and the harness fails on its own, it is tried.
    failed_attempt(&r, "H1", "no_harness");
    assert_eq!(r.close(None, "no_harness"), None);
}

#[test]
fn a_refusal_says_what_the_last_attempt_was() {
    let msg = |r: &Run| {
        let (_, v) = r.run(&[
            "ledger",
            "resolve",
            "F1",
            "--as",
            "unresolved",
            "--reason",
            "no_harness",
            "--remedy",
            "x",
        ]);
        v["error"]["message"].as_str().unwrap().to_owned()
    };
    let r = scene(Runtime::Jvm, None);
    assert!(msg(&r).contains("was generated and never run"));
    failed_attempt(&r, "H1", "hooks_unfilled");
    assert!(
        msg(&r).contains("never run with its DLA-HOOK bodies filled in"),
        "{}",
        msg(&r)
    );
    failed_attempt(&r, "H1", "interrupted");
    assert!(
        msg(&r).contains("was interrupted before it measured"),
        "{}",
        msg(&r)
    );
}

#[test]
fn another_harness_of_the_runtime_that_measured_releases_an_abandoned_one() {
    let r = scene(Runtime::Jvm, None);
    // H1 is the unfilled driver; H3 (a command harness) ran.
    manifest_of("H3", Runtime::Jvm).save(&r.run_dir()).unwrap();
    gen_evidence_of(&r, None, "H3");
    assert_eq!(
        r.close(None, "not_exercised").as_deref(),
        Some("harness-not-run")
    );
    fs::write(r.run_dir().join("harness/H3/run-S1.json"), "{}").unwrap();
    assert_eq!(r.close(None, "not_exercised"), None);
    assert!(r.not_run(None).is_empty());

    // A failed H3 does not release H1.
    let r = scene(Runtime::Jvm, None);
    manifest_of("H3", Runtime::Jvm).save(&r.run_dir()).unwrap();
    gen_evidence_of(&r, None, "H3");
    failed_attempt(&r, "H3", "no_harness");
    assert_eq!(
        r.close(None, "not_exercised").as_deref(),
        Some("harness-not-run")
    );
}

#[test]
fn the_fix_names_the_hooks_only_for_a_driver_with_template_hooks() {
    let fix = |r: &Run| {
        let (_, v) = r.run(&[
            "ledger",
            "resolve",
            "F1",
            "--as",
            "unresolved",
            "--reason",
            "no_harness",
            "--remedy",
            "x",
        ]);
        v["error"]["fix"].as_str().unwrap().to_owned()
    };
    let r = scene(Runtime::Jvm, None);
    let f = fix(&r);
    assert!(f.contains("DLA-HOOK") && f.contains("--junit"), "{f}");

    let r = scene(Runtime::Python, None);
    let f = fix(&r);
    assert!(f.contains("DLA-HOOK") && !f.contains("--junit"), "{f}");

    // A harness with nothing to fill (a command harness, the JUnit driver).
    let r = scene(Runtime::Jvm, None);
    fs::remove_dir_all(r.run_dir().join("harness/H1/driver")).unwrap();
    let f = fix(&r);
    assert!(
        f.starts_with("dla harness run H1.") && !f.contains("DLA-HOOK"),
        "{f}"
    );
}

#[cfg(unix)]
#[test]
fn a_run_stopped_by_a_signal_writes_an_interrupted_attempt() {
    use std::{process::Stdio, time::Duration};
    let r = scene(Runtime::Native, None);
    let mut m = manifest_of("H1", Runtime::Native);
    m.mode = HarnessMode::Command;
    m.agent = AgentKind::None;
    m.iter_arg = Some("{n}".into());
    m.run.argv = vec!["sh".into(), "-c".into(), "sleep 30".into()];
    m.save(&r.run_dir()).unwrap();
    fs::remove_dir_all(r.run_dir().join("harness/H1/driver")).unwrap();
    for d in ["work", "agent"] {
        fs::create_dir_all(r.run_dir().join("harness/H1").join(d)).unwrap();
    }
    fs::write(
        r.run_dir().join("harness/H1/probes.json"),
        "{\"probes\":[]}",
    )
    .unwrap();
    let bin = Command::cargo_bin("dla").unwrap();
    let mut cmd = std::process::Command::new(bin.get_program());
    cmd.arg("--project").arg(r.dir.path());
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_PROJECT_DIR",
        "DLA_CONSENT_RAW_VALUES",
    ] {
        cmd.env_remove(var);
    }
    let mut child = cmd
        .env("DLA_CONSENT_TIERS", "1,2")
        .args(["harness", "run", "H1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let path = HarnessAttempt::path_in(&r.run_dir(), "H1");
    // The measured process writes its log after the stop handler is installed: wait for
    // it instead of sleeping a fixed time (a loaded host may be slow to start).
    let logs = r.run_dir().join("harness/H1/logs");
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    while !fs::read_dir(&logs).is_ok_and(|mut d| d.next().is_some()) {
        assert!(
            std::time::Instant::now() < deadline,
            "the run never started"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!path.exists(), "the run ended early");
    let pid = child.id().to_string();
    let _ = std::process::Command::new("kill")
        .args(["-TERM", &pid])
        .status();
    let status = child.wait().unwrap();
    assert!(!status.success());
    let a: HarnessAttempt = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(a.outcome, AttemptOutcome::Failed);
    assert_eq!(a.reason.as_deref(), Some("interrupted"));
    // A stopped run measured nothing: the rule still holds, and says why.
    assert_eq!(
        r.close(None, "no_harness").as_deref(),
        Some("harness-not-run")
    );
    failed_attempt(&r, "H1", "no_harness");
    assert_eq!(r.close(None, "no_harness"), None);
}

/// A real run had 362 findings closed with no_harness and a Stop hook with a 10 s
/// timeout: the rule must cost about one load of the series, not one per closed finding.
/// Before the fix the final gate took about 6 s on this scene in a debug build (362
/// findings x 2 loads of 30 series; 37 s for `gate check --final` on 30 series files of
/// 21 KB holding 12 columns x 60 points, which cost more to load); now it takes a
/// fraction of a second.
#[test]
fn the_rule_costs_one_load_of_the_series_not_one_per_closed_finding() {
    let r = scene(Runtime::Jvm, None);
    let run_dir = r.run_dir();
    failed_attempt(&r, "H1", "hooks_unfilled");
    fs::create_dir_all(run_dir.join("series")).unwrap();
    let pad: Vec<String> = (0..300)
        .map(|i| format!("limitation number {i} padding the series file to a realistic size"))
        .collect();
    for n in 1..=30 {
        fs::write(
            run_dir.join(format!("series/S{n}.json")),
            json!({"schema_version": 1, "id": format!("S{n}"),
                   "source": {"kind": "harness", "harness": "H7"},
                   "axis": "n", "warmup": 0, "columns": [], "points": [],
                   "limitations": pad})
            .to_string(),
        )
        .unwrap();
    }
    let mut l = Ledger::load(&run_dir).unwrap();
    let e = l.evidence().last().unwrap().evidence_id.clone();
    for _ in 1..362 {
        let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![e.clone()]);
        nf.site = Some("a/B.kt:1".into());
        l.add_finding(nf).unwrap();
    }
    let ids: Vec<_> = l.findings().map(|f| f.id.clone()).collect();
    assert_eq!(ids.len(), 362);
    for id in &ids {
        l.transition(
            id,
            dla_core::FindingState::Unresolved,
            Some(dla_core::UnresolvedReason::NoHarness),
            Some("x".into()),
            vec![],
            "t",
        )
        .unwrap();
    }
    let started = std::time::Instant::now();
    let view = dla_core::RunView::load(&run_dir).unwrap();
    let missing = dla_core::gate::missing(&view, dla_core::gate::GateTarget::Final, None);
    let took = started.elapsed();
    eprintln!("final gate over 362 closed findings and 30 series: {took:?}");
    let item = missing
        .iter()
        .find(|m| m.kind == "harness_not_run")
        .expect("H1 is listed");
    assert!(
        item.description.contains("362 finding(s)"),
        "{}",
        item.description
    );
    assert!(
        took < std::time::Duration::from_secs(1),
        "the final gate took {took:?} with 362 closed findings and 30 series"
    );

    // The commands a user waits for: the Stop hook has a 10 s timeout.
    let timed = |label: &str, f: &dyn Fn()| {
        let t = std::time::Instant::now();
        f();
        let took = t.elapsed();
        eprintln!("{label}: {took:?}");
        assert!(
            took < std::time::Duration::from_secs(3),
            "{label} took {took:?}"
        );
    };
    timed("gate check --final", &|| {
        r.run(&["gate", "check", "--final"]);
    });
    timed("run status", &|| {
        r.ok(&["run", "status"]);
    });
    timed("stop hook", &|| {
        base(r.dir.path())
            .args(["gate", "stop-hook"])
            .write_stdin(json!({"cwd": r.dir.path(), "hook_event_name": "Stop"}).to_string())
            .output()
            .unwrap();
    });
    timed("report", &|| {
        r.run(&["report"]);
    });
}
