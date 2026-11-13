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
        w.contains("harness H1 did not build, so no workload iteration was recorded")
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
