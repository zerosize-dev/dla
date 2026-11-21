//! Keep measuring while time is left: a finding of a runtime with discovered entry points
//! and no harness cannot be closed without exercise (`harness-not-generated`), and the
//! final gate lists `coverage_contradiction` and `budget_left_unrevisited`. Each has an
//! escape an unattended run can take. `dla run status` carries the harness counters.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, CoverageManifest, EvidenceKind, Ledger, NewEvidence, Source, Tool,
    coverage::EntryPoints,
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
    /// Closes `id` unresolved with `reason`; the refusal, if any.
    fn close(&self, id: &str, reason: &str) -> Option<Value> {
        let (code, v) = self.run(&[
            "ledger",
            "resolve",
            id,
            "--as",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
        ]);
        (code != 0).then(|| v["error"].clone())
    }
    /// `(kind, id, description, fix)` of every item the final gate lists (the full file).
    fn missing(&self) -> Vec<(String, String, String, String)> {
        let _ = self.run(&["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir().join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                let s = |k: &str| m[k].as_str().unwrap().to_owned();
                (s("kind"), s("id"), s("description"), s("fix"))
            })
            .collect()
    }
    fn items(&self, kind: &str) -> Vec<(String, String, String)> {
        self.missing()
            .into_iter()
            .filter(|m| m.0 == kind)
            .map(|m| (m.1, m.2, m.3))
            .collect()
    }
    fn set_state(&self, f: impl FnOnce(&mut Value)) {
        let path = self.run_dir().join("state.json");
        let mut st: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        f(&mut st);
        fs::write(path, st.to_string()).unwrap();
    }
    /// What a tool leaves when it refuses a command for time (`worker` null for the core).
    fn note_time_refusal(&self, worker: Option<&str>) {
        self.note_time_refusal_with(worker, true);
    }
    /// The same, saying whether nothing smaller than the refused command would have fit.
    fn note_time_refusal_with(&self, worker: Option<&str>, none_fits: bool) {
        use std::io::Write;
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.run_dir().join(dla_core::toolctx::TIME_REFUSALS_FILE))
            .unwrap();
        let line = json!({"ts": rfc3339_in(60), "worker": worker, "none_fits": none_fits});
        writeln!(f, "{line}").unwrap();
    }
    fn workload_blocker(&self, reason: &str) {
        self.ok(&[
            "ledger",
            "resolve",
            "intake.workload",
            "--status",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "the service needs a database.",
        ]);
    }
}

fn rfc3339_in(secs: u64) -> String {
    (dla_core::state::now_utc() + chrono::Duration::seconds(i64::try_from(secs).unwrap()))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// A symptom run (no probe or sweep stage, so the gate asks for the checklist and the
/// dispositions only) with finding F1 open, a 60 minute budget with all of it left, and
/// the checklist answered.
fn scene() -> Run {
    scene_with_budget("60m")
}

fn scene_with_budget(budget: &str) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let o = base(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2")
        .args([
            "run",
            "init",
            "--session-id",
            "t",
            "--entry",
            "symptom",
            "--budget",
            budget,
        ])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let r = Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    };
    r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        "E1",
    ]);
    for item in [
        "target", "scope", "symptom", "runtimes", "build", "workload", "consent", "budget",
    ] {
        r.ok(&[
            "ledger",
            "resolve",
            &format!("intake.{item}"),
            "--status",
            "answered",
            "--evidence",
            "E1",
        ]);
    }
    r
}

/// Records `discovered` entry points, `exercised` of them run, `iterations` for `jvm`.
fn coverage(r: &Run, discovered: usize, exercised: usize, iterations: Option<u64>, classes: u64) {
    let names = |n: usize| {
        (0..n)
            .map(|i| format!("a/B{i}.kt::B{i}.f"))
            .collect::<Vec<_>>()
    };
    CoverageManifest::update(&r.run_dir(), |m| {
        m.set_entry_points(
            "jvm",
            EntryPoints {
                discovered: names(discovered),
                exercised: names(exercised),
                not_exercisable: vec![],
            },
        );
        if let Some(n) = iterations {
            m.set_iterations("jvm", n);
        }
        if classes > 0 {
            m.set_project_classes("jvm", classes);
        }
    })
    .unwrap();
}

/// The `intervention` evidence `dla intervene` writes: it names the treated clone.
fn intervention_evidence(r: &Run, harness: &str, treated: &str) {
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    let mut result =
        json!({"test": "intervention", "harness": harness, "treated_harness": treated});
    result["signal"] = Signal::new("t", "x", "src").to_value();
    l.add_evidence(NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "0".into(),
        },
        authority: Authority::Test,
        kind: EvidenceKind::Intervention,
        result,
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
}

/// The `workload` evidence `dla harness gen` writes for harness `id`.
fn generation_evidence(r: &Run, id: &str) {
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    let mut result = json!({"harness": id, "mode": "driver"});
    result["signal"] = Signal::new("t", "x", "src").to_value();
    l.add_evidence(NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "0".into(),
        },
        authority: Authority::Test,
        kind: EvidenceKind::Workload,
        result,
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
}

fn harness_manifest(r: &Run, id: &str) {
    harness_manifest_in(r, id, HarnessMode::Driver);
}

fn harness_manifest_in(r: &Run, id: &str, mode: HarnessMode) {
    harness_manifest_with(r, id, mode, &[]);
}

/// A JUnit repeat: a driver whose manifest names the JUnit launcher and which has no hooks.
fn junit_manifest(r: &Run, id: &str) {
    harness_manifest_with(r, id, HarnessMode::Driver, &["junit-platform-launcher"]);
}

fn harness_manifest_with(r: &Run, id: &str, mode: HarnessMode, tools: &[&str]) {
    HarnessManifest {
        sibling_of: Default::default(),
        purpose: None,
        targets: Vec::new(),
        schema_version: 1,
        id: id.into(),
        runtime: Runtime::Jvm,
        mode,
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
        tools: tools.iter().map(|t| (*t).to_owned()).collect(),
        gaps: vec![],
        timeout_secs: 300,
        ffi: None,
        captures: vec![],
    }
    .save(&r.run_dir())
    .unwrap();
}

/// The `Coverage warning:` line of the rendered report, empty when there is none.
fn warning(r: &Run) -> String {
    let o = base(r.dir.path()).args(["report"]).output().unwrap();
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .find(|l| l.starts_with("Coverage warning: "))
        .unwrap_or_default()
        .to_owned()
}

fn failed_attempt(r: &Run, id: &str, reason: &str) {
    HarnessAttempt {
        harness: id.into(),
        started_at: HarnessAttempt::now(),
        finished_at: HarnessAttempt::now(),
        outcome: AttemptOutcome::Failed,
        reason: Some(reason.into()),
        message: Some("harness H1 did not build: the build exited with code 1".into()),
    }
    .save(&r.run_dir())
    .unwrap();
}

/// The attempt record of a harness that ran.
fn ran_attempt(r: &Run, id: &str) {
    HarnessAttempt {
        harness: id.into(),
        started_at: HarnessAttempt::now(),
        finished_at: HarnessAttempt::now(),
        outcome: AttemptOutcome::Ran,
        reason: None,
        message: None,
    }
    .save(&r.run_dir())
    .unwrap();
}

/// Writes `workload/workloads.json` with one jvm main workload (`iteration_arg` or none).
fn workloads(r: &Run, iteration_arg: Option<&str>) {
    fs::create_dir_all(r.run_dir().join("workload")).unwrap();
    let doc = json!({
        "schema_version": 1,
        "workloads": [{
            "id": "W1", "kind": "main", "runtime": "jvm", "command": "java App", "cwd": ".",
            "file": "App.java", "entry_points": [], "rank": 1,
            "iteration_arg": iteration_arg
        }]
    });
    fs::write(r.run_dir().join("workload/workloads.json"), doc.to_string()).unwrap();
}

// ------------------------------------------------------------------

#[test]
fn entry_points_and_no_harness_refuse_the_closes_that_say_nothing_ran() {
    let r = scene();
    coverage(&r, 2, 0, None, 0);
    for reason in ["no_harness", "not_exercised", "time_budget"] {
        let e = r.close("F1", reason).unwrap_or_else(|| panic!("{reason}"));
        assert_eq!(e["code"], "harness-not-generated", "{reason}: {e}");
    }
    let e = r.close("F1", "no_harness").unwrap();
    let msg = e["message"].as_str().unwrap();
    assert!(
        msg.contains("F1 cannot be closed with no_harness")
            && msg.contains("2 entry point(s) of jvm were discovered")
            && msg.contains("no harness was generated")
            && msg.contains("of 60 budget minutes remain"),
        "{msg}"
    );
    let fix = e["fix"].as_str().unwrap();
    assert!(
        fix.contains("dla harness gen --lang jvm")
            && fix.contains("intake.workload")
            && fix.contains("ask the user"),
        "{fix}"
    );
    // Nothing was written.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["counts"]["by_state"]["candidate"], 1, "{st}");
}

#[test]
fn a_blocker_reason_is_never_refused() {
    for reason in [
        "env_unavailable",
        "build_fails",
        "tool_missing",
        "privilege_declined",
    ] {
        let r = scene();
        coverage(&r, 2, 0, None, 0);
        assert!(r.close("F1", reason).is_none(), "{reason}");
    }
}

#[test]
fn a_blocker_on_the_workload_item_clears_the_rule() {
    let r = scene();
    coverage(&r, 2, 0, None, 0);
    assert!(r.close("F1", "not_exercised").is_some());
    r.workload_blocker("env_unavailable");
    assert!(r.close("F1", "not_exercised").is_none());
}

#[test]
fn under_the_floor_without_entry_points_or_with_a_harness_the_rule_is_silent() {
    // 60 minute budget: the floor is 12 minutes.
    let r = scene();
    coverage(&r, 2, 0, None, 0);
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(11 * 60)));
    assert!(r.close("F1", "time_budget").is_none(), "11 minutes left");

    let r = scene();
    coverage(&r, 2, 0, None, 0);
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(13 * 60)));
    assert_eq!(
        r.close("F1", "no_harness").unwrap()["code"],
        "harness-not-generated",
        "13 minutes left"
    );

    // No entry point was discovered: nothing to build a harness for.
    let r = scene();
    coverage(&r, 0, 0, None, 0);
    assert!(r.close("F1", "no_harness").is_none());

    // All of them are marked not exercisable.
    let r = scene();
    CoverageManifest::update(&r.run_dir(), |m| {
        m.set_entry_points(
            "jvm",
            EntryPoints {
                discovered: vec!["a/B.kt::B.f".into()],
                exercised: vec![],
                not_exercisable: vec![dla_core::coverage::NotExercisable {
                    entry: "a/B.kt::B.f".into(),
                    reason: "needs a queue".into(),
                }],
            },
        );
    })
    .unwrap();
    assert!(r.close("F1", "no_harness").is_none());

    // Marked `no_harness` (no test or main workload names them): a harness is what
    // would reach them, so the rule still applies.
    let r = scene();
    CoverageManifest::update(&r.run_dir(), |m| {
        m.set_entry_points(
            "jvm",
            EntryPoints {
                discovered: vec!["a/B.kt::B.f".into()],
                exercised: vec![],
                not_exercisable: vec![dla_core::coverage::NotExercisable {
                    entry: "a/B.kt::B.f".into(),
                    reason: "no_harness".into(),
                }],
            },
        );
    })
    .unwrap();
    assert_eq!(
        r.close("F1", "no_harness").unwrap()["code"],
        "harness-not-generated"
    );

    // A harness of the runtime exists (the unrun-harness rule governs it, not this one).
    let r = scene();
    coverage(&r, 2, 0, None, 0);
    harness_manifest(&r, "H1");
    assert!(r.close("F1", "no_harness").is_none());

    // No coverage manifest at all.
    let r = scene();
    assert!(r.close("F1", "no_harness").is_none());
}

// ------------------------------------------------------------------

#[test]
fn zero_exercised_after_iterations_is_a_gate_item_with_an_escape() {
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, Some(750), 37);
    harness_manifest(&r, "H1");
    let items = r.items("coverage_contradiction");
    assert_eq!(items.len(), 1, "{items:?}");
    let (id, desc, fix) = &items[0];
    assert_eq!(id, "jvm");
    assert!(
        desc.contains("750 iteration(s) ran and 0 of 225 entry points were exercised")
            && desc.contains("37 project classes loaded"),
        "{desc}"
    );
    // H1 is a driver: its own hooks are the step, and it is the one to run again.
    assert!(
        fix.contains("fill the DLA-HOOK bodies of H1")
            && fix.contains("dla harness run H1")
            && !fix.contains("harness gen")
            && fix.contains("intake.workload --status unresolved --reason not_exercised --remedy"),
        "{fix}"
    );
    // H1 is a command harness (no hooks): the step makes a driver and names the one it
    // prints, never the harness that produced the zero.
    let c = scene();
    c.close("F1", "not_exercised");
    coverage(&c, 225, 0, Some(750), 37);
    harness_manifest_in(&c, "H1", HarnessMode::Command);
    let fix = c.items("coverage_contradiction")[0].2.clone();
    assert!(
        fix.contains("dla harness gen --lang jvm --driver")
            && fix.contains("the harness it prints")
            && !fix.contains("dla harness run H1"),
        "{fix}"
    );
    // The run is not complete while it is listed, and it is after the escape.
    assert_eq!(r.ok(&["run", "status"])["complete"], false);
    r.workload_blocker("not_exercised");
    assert!(r.items("coverage_contradiction").is_empty());
    assert_eq!(r.ok(&["run", "status"])["complete"], true);
}

#[test]
fn the_contradiction_escape_is_not_an_early_stop() {
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, Some(750), 37);
    // An hour is left: "no time" is not a reason the driver cannot reach the code.
    r.workload_blocker("time_budget");
    assert_eq!(r.items("coverage_contradiction").len(), 1);
    r.workload_blocker("not_exercised");
    assert!(r.items("coverage_contradiction").is_empty());
    // With the budget spent, time_budget is the honest reason and clears it.
    let r = scene_with_budget("5m");
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, Some(750), 37);
    r.workload_blocker("time_budget");
    assert!(r.items("coverage_contradiction").is_empty());
}

#[test]
fn runtimes_that_never_record_exercised_entry_points_are_not_contradictions() {
    for rt in ["rust", "native"] {
        let r = scene();
        r.close("F1", "not_exercised");
        CoverageManifest::update(&r.run_dir(), |m| {
            m.set_entry_points(
                rt,
                EntryPoints {
                    discovered: vec!["src/lib.rs::f".into(), "src/lib.rs::g".into()],
                    exercised: vec![],
                    not_exercisable: vec![],
                },
            );
            m.set_iterations(rt, 880);
        })
        .unwrap();
        assert!(r.items("coverage_contradiction").is_empty(), "{rt}");
    }
    // A runtime that records a blind spot for "which entry points ran" is the same.
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 4, 0, Some(100), 0);
    assert_eq!(r.items("coverage_contradiction").len(), 1);
    CoverageManifest::update(&r.run_dir(), |m| {
        m.add_blind_spot("jvm", "the agent cannot say which entry points ran");
    })
    .unwrap();
    assert!(r.items("coverage_contradiction").is_empty());
}

#[test]
fn the_contradiction_needs_iterations_entry_points_and_nothing_exercised() {
    for (discovered, exercised, iterations) in [
        (225, 1, Some(750)),
        (225, 0, None),
        (225, 0, Some(0)),
        (0, 0, Some(750)),
    ] {
        let r = scene();
        r.close("F1", "not_exercised");
        coverage(&r, discovered, exercised, iterations, 0);
        assert!(
            r.items("coverage_contradiction").is_empty(),
            "{discovered} {exercised} {iterations:?}"
        );
    }
}

#[test]
fn time_budget_closes_with_time_left_are_listed_until_reclosed() {
    let r = scene();
    // Closed under the floor, as an honest close; the deadline then moves out (an extension).
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(5 * 60)));
    assert!(r.close("F1", "time_budget").is_none());
    assert!(r.items("budget_left_unrevisited").is_empty());
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(50 * 60)));
    let items = r.items("budget_left_unrevisited");
    assert_eq!(items.len(), 1, "{items:?}");
    let (id, desc, fix) = &items[0];
    assert_eq!(id, "F1");
    assert!(
        desc.contains("1 finding(s) are closed with time_budget (first F1)")
            && desc.contains(" of 60 budget minutes remain"),
        "{desc}"
    );
    assert!(
        fix.contains("dla ledger test F1 --evidence")
            && fix.contains("intake.workload --status unresolved"),
        "{fix}"
    );
    // The escape: the workload item records that nothing can be measured.
    r.workload_blocker("env_unavailable");
    assert!(r.items("budget_left_unrevisited").is_empty());
}

#[test]
fn only_the_cores_own_refusal_clears_a_time_budget_close() {
    let r = scene();
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(5 * 60)));
    assert!(r.close("F1", "time_budget").is_none());
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(50 * 60)));
    // A worker's refusal is not a command the core was refused.
    r.note_time_refusal(Some("go"));
    assert_eq!(r.items("budget_left_unrevisited").len(), 1);
    // The core's own is.
    r.note_time_refusal(None);
    assert!(r.items("budget_left_unrevisited").is_empty());
}

#[test]
fn findings_closed_before_any_harness_are_listed_with_the_command_that_makes_one() {
    let r = scene();
    // Closed while the run knew no entry point, as in a ledger written before the rule.
    assert!(r.close("F1", "not_exercised").is_none());
    coverage(&r, 3, 0, None, 0);
    let items = r.items("budget_left_unrevisited");
    assert_eq!(items.len(), 1, "{items:?}");
    let (id, desc, fix) = &items[0];
    assert_eq!(id, "jvm");
    assert!(
        desc.contains("1 finding(s) of jvm are closed without exercise")
            && desc.contains("3 discovered entry point(s) have no harness")
            && desc.contains("of 60 budget minutes remain"),
        "{desc}"
    );
    assert!(fix.contains("dla harness gen --lang jvm"), "{fix}");
    // Escapes: the blocker on the workload item; a harness; less than the floor left.
    let escaped = |f: &dyn Fn(&Run)| {
        let r = scene();
        r.close("F1", "not_exercised");
        coverage(&r, 3, 0, None, 0);
        f(&r);
        r.items("budget_left_unrevisited").is_empty()
    };
    assert!(escaped(&|r| r.workload_blocker("build_fails")));
    assert!(escaped(&|r| harness_manifest(r, "H1")));
    assert!(escaped(&|r| r.set_state(
        |st| st["budget"]["deadline"] = json!(rfc3339_in(11 * 60))
    )));
    // A blocker reason closes nothing the remaining time could change.
    let r = scene();
    r.close("F1", "build_fails");
    coverage(&r, 3, 0, None, 0);
    assert!(r.items("budget_left_unrevisited").is_empty());
}

// ------------------------------------------------------------------

#[test]
fn run_status_counts_harnesses_and_static_only_closes_and_names_the_next_command() {
    let r = scene();
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["harnesses_generated"], 0, "{st}");
    assert_eq!(st["harnesses_run"], 0);
    assert_eq!(st["closed_static_only"], 0);
    // F1 is open: the next command is the gate's first fix.
    assert!(st["blocking"][0]["fix"].is_string());
    assert_eq!(st["next_command"], "dla gate check --final", "{st}");

    // F1 closed, run complete, time left: the next command is the first revisit remedy.
    assert!(r.close("F1", "not_exercised").is_none());
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["complete"], true, "{st}");
    // Its remedy is prose, not a command: the report is next.
    assert_eq!(st["next_command"], "dla report", "{st}");
    // A remedy that is a `dla` command is the next command.
    let r = scene();
    r.ok(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "dla harness gen --lang jvm",
    ]);
    assert_eq!(
        r.ok(&["run", "status"])["next_command"],
        "dla harness gen --lang jvm"
    );
    // A finding closed on a blocker only the user can clear is not a next step.
    let r = scene();
    assert!(r.close("F1", "env_unavailable").is_none());
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["revisit_total"], 1, "{st}");
    assert_eq!(st["next_command"], "dla report", "{st}");
    // Under the floor nothing more is asked for: write the report.
    let r = scene_with_budget("5m");
    assert!(r.close("F1", "not_exercised").is_none());
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["complete"], true, "{st}");
    assert_eq!(st["next_command"], "dla report", "{st}");

    // A static-only close is counted.
    let r = scene();
    let ledger = r.run_dir().join("ledger.jsonl");
    let seq = fs::read_to_string(&ledger).unwrap().lines().count() + 1;
    let mut text = fs::read_to_string(&ledger).unwrap();
    text.push_str(&format!(
        "{{\"schema_version\":1,\"seq\":{seq},\"ts\":\"2026-01-02T03:05:00Z\",\"type\":\"evidence\",\"evidence_id\":\"E99\",\"source\":{{\"repo\":{{\"commit\":\"abc\"}}}},\"tool\":{{\"name\":\"scan\",\"version\":\"1\"}},\"authority\":\"static\",\"kind\":\"static-hit\",\"result\":{{}},\"coverage\":{{\"status\":\"complete\"}}}}\n"
    ));
    fs::write(&ledger, text).unwrap();
    r.ok(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "suspected_static_only",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
        "--evidence",
        "E99",
    ]);
    assert_eq!(r.ok(&["run", "status"])["closed_static_only"], 1);

    // A manifest without generation evidence is not a generated harness; with it, the
    // harness is generated, and run once an attempt of any outcome but a failure exists.
    let r = scene();
    harness_manifest(&r, "H1");
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["harnesses_generated"], 0, "{st}");
    generation_evidence(&r, "H1");
    let st = r.ok(&["run", "status"]);
    assert_eq!(
        (
            st["harnesses_generated"].as_u64(),
            st["harnesses_run"].as_u64()
        ),
        (Some(1), Some(0)),
        "{st}"
    );
    HarnessAttempt {
        harness: "H1".into(),
        started_at: HarnessAttempt::now(),
        finished_at: HarnessAttempt::now(),
        outcome: AttemptOutcome::Ran,
        reason: None,
        message: None,
    }
    .save(&r.run_dir())
    .unwrap();
    let st = r.ok(&["run", "status"]);
    assert_eq!(
        (
            st["harnesses_generated"].as_u64(),
            st["harnesses_run"].as_u64()
        ),
        (Some(1), Some(1)),
        "{st}"
    );
    // The key set only grew.
    for key in [
        "ok",
        "run_id",
        "stage",
        "stages_passed",
        "status",
        "complete",
        "counts",
        "blocking_total",
        "blocking",
        "budget",
        "record",
        "ledger",
        "report",
        "tamper",
        "revisit",
        "revisit_total",
        "harnesses_generated",
        "harnesses_run",
        "closed_static_only",
        "next_command",
    ] {
        assert!(st.get(key).is_some(), "{key} missing: {st}");
    }
}

// ------------------------------------------------------------------

#[test]
fn a_junit_repeat_has_no_hooks_so_the_step_makes_a_hook_driver() {
    // The only harness that ran is a JUnit repeat that reached none of 225 entry points.
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, Some(750), 37);
    junit_manifest(&r, "H1");
    ran_attempt(&r, "H1");
    let fix = r.items("coverage_contradiction")[0].2.clone();
    assert!(
        fix.contains("the project's tests reach 0 of 225 entry points")
            && fix.contains("dla harness gen --lang jvm --driver")
            && fix.contains("fill the DLA-HOOK bodies of the harness it prints")
            && fix.contains("dla harness run <the harness it prints>")
            && !fix.contains("DLA-HOOK bodies of H1")
            && !fix.contains("dla harness run H1"),
        "{fix}"
    );
    // The report's warning names the same step.
    let w = warning(&r);
    assert!(
        w.contains("To measure: the project's tests reach 0 of 225 entry points; make a hook driver with dla harness gen --lang jvm --driver"),
        "{w}"
    );
    // A minority of entry points: the warning counts them and names the hook driver.
    let m = scene();
    m.close("F1", "not_exercised");
    coverage(&m, 225, 20, Some(750), 37);
    junit_manifest(&m, "H1");
    ran_attempt(&m, "H1");
    let w = warning(&m);
    assert!(
        w.contains("most entry points were never called")
            && w.contains("the project's tests reach 20 of 225 entry points")
            && w.contains("dla harness gen --lang jvm --driver")
            && !w.contains("DLA-HOOK bodies of H1"),
        "{w}"
    );
    // Once a hook driver H2 exists, it is the one to fill and run.
    harness_manifest(&m, "H2");
    let w = warning(&m);
    assert!(
        w.contains("fill the DLA-HOOK bodies of H2") && w.contains("dla harness run H2"),
        "{w}"
    );
}

#[test]
fn the_no_harness_step_follows_what_gen_would_do() {
    // No workload with an iteration argument: gen refuses, so the step names --driver.
    let r = scene();
    coverage(&r, 10, 0, None, 0);
    let w = warning(&r);
    assert!(
        w.contains("To measure: dla harness gen --lang jvm --driver"),
        "{w}"
    );
    workloads(&r, None);
    assert!(warning(&r).contains("--driver"), "{}", warning(&r));
    // A workload that takes an iteration count: plain gen works.
    workloads(&r, Some("--iterations {n}"));
    let w = warning(&r);
    assert!(
        w.contains("To measure: dla harness gen --lang jvm.") && !w.contains("--driver"),
        "{w}"
    );
}

#[test]
fn a_recorded_workload_blocker_replaces_the_step_in_the_warning() {
    let r = scene();
    coverage(&r, 10, 0, None, 0);
    r.workload_blocker("env_unavailable");
    let w = warning(&r);
    assert!(
        w.contains(
            "Blocked: intake.workload records env_unavailable: the service needs a database."
        ) && !w.contains("..")
            && !w.contains("To measure"),
        "{w}"
    );
}

#[test]
fn a_harness_whose_build_failed_is_not_answered_with_another_driver() {
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, None, 0);
    harness_manifest_in(&r, "H1", HarnessMode::Command);
    generation_evidence(&r, "H1");
    failed_attempt(&r, "H1", "build_fails");
    let w = warning(&r);
    assert!(
        w.contains(
            "Cause: no workload iteration was recorded for it. To measure: harness H1 did not build"
        ) && w.matches("did not build").count() == 1
            && w.contains("fix the build error")
            && w.contains("--reason build_fails")
            && !w.contains("harness gen"),
        "{w}"
    );
    // The same harness that ran and zero-hit is not a build failure.
    let c = scene();
    c.close("F1", "not_exercised");
    coverage(&c, 225, 0, Some(750), 37);
    harness_manifest_in(&c, "H1", HarnessMode::Command);
    failed_attempt(&c, "H1", "no_harness");
    let fix = c.items("coverage_contradiction")[0].2.clone();
    assert!(fix.contains("dla harness gen --lang jvm --driver"), "{fix}");
    // A hook driver H2 whose build failed comes before filling it: it is newer than the
    // command harness H1 that ran.
    let d = scene();
    d.close("F1", "not_exercised");
    coverage(&d, 225, 0, Some(750), 37);
    harness_manifest_in(&d, "H1", HarnessMode::Command);
    ran_attempt(&d, "H1");
    harness_manifest(&d, "H2");
    failed_attempt(&d, "H2", "build_fails");
    let fix = d.items("coverage_contradiction")[0].2.clone();
    assert!(
        fix.contains("harness H2 did not build") && !fix.contains("harness gen"),
        "{fix}"
    );
}

#[test]
fn an_older_build_failure_does_not_hide_the_hook_driver_that_ran() {
    // H1 never built; the hook driver H2 built, ran and reached no entry point: fill H2.
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, Some(750), 37);
    harness_manifest_in(&r, "H1", HarnessMode::Command);
    failed_attempt(&r, "H1", "build_fails");
    harness_manifest(&r, "H2");
    ran_attempt(&r, "H2");
    let fix = r.items("coverage_contradiction")[0].2.clone();
    assert!(
        fix.contains("fill the DLA-HOOK bodies of H2")
            && !fix.contains("did not build")
            && !fix.contains("build_fails"),
        "{fix}"
    );
    let w = warning(&r);
    assert!(
        w.contains("fill the DLA-HOOK bodies of H2") && !w.contains("did not build"),
        "{w}"
    );
    // A JUnit repeat that failed its build, then a hook driver that reached a few: the
    // warning names the hook driver, not the old build.
    let j = scene();
    j.close("F1", "not_exercised");
    coverage(&j, 225, 20, Some(750), 37);
    junit_manifest(&j, "H1");
    failed_attempt(&j, "H1", "build_fails");
    harness_manifest(&j, "H2");
    ran_attempt(&j, "H2");
    let w = warning(&j);
    assert!(
        w.contains("fill the DLA-HOOK bodies of H2")
            && !w.contains("did not build")
            && !w.contains("the project's tests reach"),
        "{w}"
    );
}

#[test]
fn an_older_hook_driver_that_never_built_does_not_hide_a_newer_one_that_ran() {
    // H1 is a hook driver whose build failed; H2, a newer hook driver, built and ran: with
    // none and with a minority of the entry points reached, the step names H2 and never
    // offers the build_fails escape.
    for exercised in [0, 20] {
        let r = scene();
        r.close("F1", "not_exercised");
        coverage(&r, 225, exercised, Some(750), 37);
        harness_manifest(&r, "H1");
        failed_attempt(&r, "H1", "build_fails");
        harness_manifest(&r, "H2");
        ran_attempt(&r, "H2");
        let w = warning(&r);
        assert!(
            w.contains("fill the DLA-HOOK bodies of H2")
                && !w.contains("did not build")
                && !w.contains("build_fails"),
            "{exercised}: {w}"
        );
        if exercised == 0 {
            let fix = r.items("coverage_contradiction")[0].2.clone();
            assert!(
                fix.contains("fill the DLA-HOOK bodies of H2, then dla harness run H2")
                    || fix.contains("fill the DLA-HOOK bodies of H2 so its driver calls the entry points, then dla harness run H2"),
                "{fix}"
            );
            assert!(!fix.contains("build_fails") && !fix.contains("H1"), "{fix}");
        }
    }
    // Only a failed hook driver and a JUnit repeat that ran: the build step.
    let j = scene();
    j.close("F1", "not_exercised");
    coverage(&j, 225, 0, Some(750), 37);
    harness_manifest(&j, "H1");
    failed_attempt(&j, "H1", "build_fails");
    junit_manifest(&j, "H2");
    ran_attempt(&j, "H2");
    let fix = j.items("coverage_contradiction")[0].2.clone();
    assert!(
        fix.contains("harness H1 did not build") && !fix.contains("fill the DLA-HOOK bodies"),
        "{fix}"
    );
}

#[test]
fn the_step_names_the_newest_built_hook_driver_and_never_an_intervention_clone() {
    // Two hook drivers that both built: the newest is the one to fill.
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 20, Some(750), 37);
    harness_manifest(&r, "H1");
    ran_attempt(&r, "H1");
    harness_manifest(&r, "H2");
    ran_attempt(&r, "H2");
    let w = warning(&r);
    assert!(w.contains("fill the DLA-HOOK bodies of H2"), "{w}");
    // H3 is the edited clone `dla intervene` made of H1: it keeps the mode and tools, so
    // it is no driver to fill, whether its edited build ran or never left a record.
    for ran in [true, false] {
        let c = scene();
        c.close("F1", "not_exercised");
        coverage(&c, 225, 20, Some(750), 37);
        harness_manifest(&c, "H1");
        ran_attempt(&c, "H1");
        harness_manifest(&c, "H2");
        if ran {
            ran_attempt(&c, "H2");
        }
        intervention_evidence(&c, "H1", "H2");
        let w = warning(&c);
        assert!(
            w.contains("fill the DLA-HOOK bodies of H1") && !w.contains("of H2"),
            "{ran}: {w}"
        );
    }
    // An intervention that stopped before it wrote its record: the clone is marked from
    // the moment it is made, so it is still no driver to fill.
    let m = scene();
    m.close("F1", "not_exercised");
    coverage(&m, 225, 20, Some(750), 37);
    harness_manifest(&m, "H1");
    ran_attempt(&m, "H1");
    harness_manifest(&m, "H2");
    std::fs::write(
        m.run_dir()
            .join("harness/H2")
            .join(dla_core::harness::CLONE_MARKER),
        "H1",
    )
    .unwrap();
    let w = warning(&m);
    assert!(
        w.contains("fill the DLA-HOOK bodies of H1") && !w.contains("of H2"),
        "{w}"
    );
    // Two hook drivers that never built, then a command harness that ran: the newest
    // failed build is the step.
    let n = scene();
    n.close("F1", "not_exercised");
    coverage(&n, 225, 0, Some(750), 37);
    harness_manifest(&n, "H1");
    failed_attempt(&n, "H1", "build_fails");
    harness_manifest(&n, "H2");
    failed_attempt(&n, "H2", "build_fails");
    harness_manifest_in(&n, "H3", HarnessMode::Command);
    ran_attempt(&n, "H3");
    let fix = n.items("coverage_contradiction")[0].2.clone();
    assert!(fix.contains("harness H2 did not build"), "{fix}");
    // A hook driver whose build failed and nothing else: the build step.
    let o = scene();
    o.close("F1", "not_exercised");
    coverage(&o, 225, 0, Some(750), 37);
    harness_manifest(&o, "H1");
    failed_attempt(&o, "H1", "build_fails");
    let fix = o.items("coverage_contradiction")[0].2.clone();
    assert!(
        fix.contains("harness H1 did not build") && !fix.contains("fill the DLA-HOOK bodies"),
        "{fix}"
    );
    // A hook driver that ran, then a newer driver whose build failed: the driver that
    // built is still the one to fill, with no build_fails escape.
    let e = scene();
    e.close("F1", "not_exercised");
    coverage(&e, 225, 20, Some(750), 37);
    harness_manifest(&e, "H1");
    ran_attempt(&e, "H1");
    harness_manifest(&e, "H2");
    failed_attempt(&e, "H2", "build_fails");
    let w = warning(&e);
    assert!(
        w.contains("fill the DLA-HOOK bodies of H1") && !w.contains("build_fails"),
        "{w}"
    );
}

#[test]
fn a_failed_build_is_said_once_even_when_the_step_is_another_harness() {
    // H1 never built and H2 was generated and not run: the cause names H1's build, the
    // step runs H2.
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, None, 0);
    harness_manifest(&r, "H1");
    failed_attempt(&r, "H1", "build_fails");
    harness_manifest(&r, "H2");
    let w = warning(&r);
    assert!(
        w.contains(
            "Cause: harness H1 did not build, so no workload iteration was recorded for it."
        ) && w.contains("dla harness run H2")
            && w.matches("did not build").count() == 1,
        "{w}"
    );
}

/// A failed build is said once whenever the step is not that build: in the cause when a
/// newer harness waits to be run or the workload is blocked, and not at all when a newer
/// harness was tried and failed another way.
#[test]
fn a_failed_build_is_said_once_for_every_step_that_is_not_the_build() {
    let said = |w: &str| w.matches("did not build").count();
    // (a) a command harness that never built, a newer one generated and not run.
    let a = scene();
    a.close("F1", "not_exercised");
    coverage(&a, 225, 0, None, 0);
    harness_manifest_in(&a, "H1", HarnessMode::Command);
    failed_attempt(&a, "H1", "build_fails");
    harness_manifest_in(&a, "H2", HarnessMode::Command);
    generation_evidence(&a, "H2");
    let w = warning(&a);
    assert!(
        w.contains("Cause: harness H1 did not build")
            && w.contains("dla harness run H2")
            && said(&w) == 1,
        "{w}"
    );
    // (b) a hook driver that never built, then a JUnit repeat generated and not run.
    let b = scene();
    b.close("F1", "not_exercised");
    coverage(&b, 225, 0, None, 0);
    harness_manifest(&b, "H1");
    failed_attempt(&b, "H1", "build_fails");
    junit_manifest(&b, "H2");
    generation_evidence(&b, "H2");
    let w = warning(&b);
    assert!(
        w.contains("Cause: harness H1 did not build")
            && w.contains("dla harness run H2")
            && said(&w) == 1,
        "{w}"
    );
    // (c) a hook driver that never built and an unresolved workload.
    let c = scene();
    c.close("F1", "not_exercised");
    coverage(&c, 225, 0, None, 0);
    harness_manifest(&c, "H1");
    failed_attempt(&c, "H1", "build_fails");
    c.workload_blocker("env_unavailable");
    let w = warning(&c);
    assert!(
        w.contains("Cause: harness H1 did not build") && w.contains("Blocked:") && said(&w) == 1,
        "{w}"
    );
    // A newer driver was tried and failed another way: the old build is not the cause,
    // and the step names the newer driver.
    let d = scene();
    d.close("F1", "not_exercised");
    coverage(&d, 225, 0, None, 0);
    harness_manifest(&d, "H1");
    failed_attempt(&d, "H1", "build_fails");
    harness_manifest(&d, "H2");
    failed_attempt(&d, "H2", "no_harness");
    let w = warning(&d);
    assert!(
        !w.contains("did not build") && w.contains("fill the DLA-HOOK bodies of H2"),
        "{w}"
    );
}

#[test]
fn the_hook_driver_that_never_built_is_sent_to_its_build_error() {
    // H1 is a hook driver whose build failed; a later command harness H2 ran and reached
    // nothing. Filling the hooks of H1 would meet the same build error.
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, Some(750), 37);
    harness_manifest(&r, "H1");
    failed_attempt(&r, "H1", "build_fails");
    harness_manifest_in(&r, "H2", HarnessMode::Command);
    ran_attempt(&r, "H2");
    let fix = r.items("coverage_contradiction")[0].2.clone();
    assert!(
        fix.contains("harness H1 did not build") && !fix.contains("fill the DLA-HOOK bodies of H1"),
        "{fix}"
    );
    let w = warning(&r);
    assert!(
        w.contains("harness H1 did not build") && !w.contains("fill the DLA-HOOK bodies of H1"),
        "{w}"
    );
}

#[test]
fn the_tests_count_is_named_only_when_the_junit_repeat_is_what_ran() {
    // A command harness ran and a JUnit repeat was only generated: the tests never ran.
    let r = scene();
    r.close("F1", "not_exercised");
    coverage(&r, 225, 0, Some(750), 37);
    harness_manifest_in(&r, "H1", HarnessMode::Command);
    ran_attempt(&r, "H1");
    junit_manifest(&r, "H2");
    let fix = r.items("coverage_contradiction")[0].2.clone();
    assert!(
        !fix.contains("the project's tests reach")
            && fix.contains("dla harness gen --lang jvm --driver"),
        "{fix}"
    );
    // Both ran: the count belongs to both, not to the tests.
    ran_attempt(&r, "H2");
    let fix = r.items("coverage_contradiction")[0].2.clone();
    assert!(!fix.contains("the project's tests reach"), "{fix}");
}

#[test]
fn the_refusal_of_a_close_for_lack_of_a_harness_gives_the_same_step_as_the_warning() {
    // No workload with an iteration argument: gen would refuse, so the refusal names --driver.
    let r = scene();
    coverage(&r, 2, 0, None, 0);
    let e = r.close("F1", "no_harness").unwrap();
    assert!(
        e.to_string()
            .contains("dla harness gen --lang jvm --driver, fill the DLA-HOOK bodies"),
        "{e}"
    );
    workloads(&r, Some("--iterations {n}"));
    let e = r.close("F1", "no_harness").unwrap();
    let text = e.to_string();
    assert!(
        text.contains("dla harness gen --lang jvm, then dla harness run <H>")
            && !text.contains("--driver"),
        "{e}"
    );
}

#[test]
fn a_harness_refused_for_time_is_no_try_after_the_budget_was_extended() {
    // `dla harness run H1` refused at a spent deadline wrote a `time_budget` attempt; the
    // user then extended the budget. Nothing was measured: H1 is still the step, and the
    // closes that say nothing ran are refused with `dla harness run H1`.
    let r = scene();
    coverage(&r, 225, 0, None, 0);
    harness_manifest(&r, "H1");
    generation_evidence(&r, "H1");
    failed_attempt(&r, "H1", "time_budget");
    let e = r
        .close("F1", "not_exercised")
        .expect("the close is refused");
    assert_eq!(e["code"], "harness-not-run", "{e}");
    let text = e.to_string();
    assert!(
        text.contains("dla harness run H1") && text.contains("was refused for time"),
        "{text}"
    );
    // The final gate and the report name the same step, not a new driver.
    let (_, _, _, fix) = r
        .missing()
        .into_iter()
        .find(|m| m.1 == "F1")
        .expect("F1 is listed");
    assert!(fix.contains("dla harness run H1"), "{fix}");
    // Under the floor the rule is off, as for any unrun harness.
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(60)));
    assert!(r.close("F1", "not_exercised").is_none());
    let w = warning(&r);
    assert!(w.contains("dla harness run H1"), "{w}");
}

#[test]
fn a_harness_that_never_built_refuses_the_closes_and_names_the_build_step() {
    let r = scene();
    coverage(&r, 225, 0, None, 0);
    harness_manifest(&r, "H1");
    generation_evidence(&r, "H1");
    failed_attempt(&r, "H1", "build_fails");
    for reason in ["not_exercised", "no_harness", "time_budget"] {
        let e = r.close("F1", reason).expect("the close is refused");
        let text = e.to_string();
        assert!(
            text.contains("did not build")
                && text.contains("fix the build error that `dla harness run H1` prints")
                && text.contains(
                    "--reason <env_unavailable|tool_missing|build_fails|privilege_declined>"
                ),
            "{reason}: {text}"
        );
    }
    // The open finding's fix in the final gate is that step, not a close.
    let (_, _, _, fix) = r
        .missing()
        .into_iter()
        .find(|m| m.1 == "F1")
        .expect("F1 is listed");
    assert!(
        fix.contains("fix the build error") && !fix.contains("dla ledger resolve F1"),
        "{fix}"
    );
    // The escapes: the blocker reason on the finding, or the blocker on the workload item.
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "build_fails",
        "--remedy",
        "the build needs a private registry",
    ]);
    assert_eq!(code, 0, "{v}");
    // A harness of the runtime that ran lifts the rule: the build failure is no longer all there is.
    let c = scene();
    coverage(&c, 225, 0, Some(750), 37);
    harness_manifest(&c, "H1");
    generation_evidence(&c, "H1");
    failed_attempt(&c, "H1", "build_fails");
    harness_manifest_in(&c, "H2", HarnessMode::Command);
    generation_evidence(&c, "H2");
    ran_attempt(&c, "H2");
    assert!(c.close("F1", "not_exercised").is_none());
    // The blocker recorded on the workload item clears it.
    let d = scene();
    coverage(&d, 225, 0, None, 0);
    harness_manifest(&d, "H1");
    generation_evidence(&d, "H1");
    failed_attempt(&d, "H1", "build_fails");
    d.workload_blocker("build_fails");
    assert!(d.close("F1", "not_exercised").is_none());
}

#[test]
fn a_refusal_that_names_a_smaller_command_does_not_make_time_budget_honest() {
    let r = scene();
    // 38 of 60 minutes left, a command refused with a smaller one on offer: it can still run.
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(38 * 60)));
    r.note_time_refusal_with(None, false);
    let e = r.close("F1", "time_budget").expect("refused");
    assert_eq!(e["code"], "time-left", "{e}");
    assert!(e.to_string().contains("nothing smaller that fits"), "{e}");
    // The same refusal with nothing smaller that fits makes the reason honest.
    r.note_time_refusal_with(None, true);
    assert!(r.close("F1", "time_budget").is_none());
    // And the final gate lists a close made after only a refusal that named a smaller command.
    let c = scene();
    c.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(5 * 60)));
    assert!(c.close("F1", "time_budget").is_none());
    c.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(50 * 60)));
    c.note_time_refusal_with(None, false);
    assert_eq!(c.items("budget_left_unrevisited").len(), 1);
}

#[test]
fn an_open_finding_names_the_harness_step_where_its_close_would_be_refused() {
    // No harness, entry points discovered, no workload to repeat: the step is a driver.
    let r = scene();
    coverage(&r, 225, 0, None, 0);
    let fix_of = |r: &Run| {
        r.missing()
            .into_iter()
            .find(|m| m.1 == "F1")
            .map(|m| m.3)
            .expect("F1 is listed")
    };
    let fix = fix_of(&r);
    assert!(
        fix.contains("dla harness gen --lang jvm --driver")
            && !fix.contains("dla ledger resolve F1"),
        "{fix}"
    );
    // A generated harness that never ran: run it.
    harness_manifest(&r, "H1");
    generation_evidence(&r, "H1");
    let fix = fix_of(&r);
    assert!(fix.contains("dla harness run H1"), "{fix}");
    // `dla run status` carries the same fix.
    let st = r.ok(&["run", "status"]);
    assert!(
        st["blocking"][0]["fix"]
            .as_str()
            .unwrap()
            .contains("dla harness run H1"),
        "{st}"
    );
    // Once a harness ran the close is accepted and the fix is the close.
    ran_attempt(&r, "H1");
    let fix = fix_of(&r);
    assert_eq!(fix, "dla ledger close-rest", "{fix}");
}

#[test]
fn the_coverage_contradiction_is_listed_before_the_open_findings() {
    let r = scene();
    coverage(&r, 225, 0, Some(750), 37);
    harness_manifest(&r, "H1");
    ran_attempt(&r, "H1");
    let missing = r.missing();
    let pos = |kind: &str| missing.iter().position(|m| m.0 == kind);
    let (c, f) = (pos("coverage_contradiction"), pos("finding"));
    assert!(c.is_some() && f.is_some(), "{missing:?}");
    assert!(c < f, "{missing:?}");
}
