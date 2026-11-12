//! `dla harness run` writes `harness/H<n>/attempt.json` on every path once the manifest has
//! loaded: a run, a failed build, a refusal before any point, and an expired budget.
#![cfg(unix)]

use std::{fs, path::Path, process::Command};

use dla_core::{
    Ledger,
    harness::{
        AgentKind, AttemptOutcome, CommandSpec, HarnessAttempt, HarnessManifest, HarnessMode,
    },
    runtime::Runtime,
    series::KeyMode,
    state::{Consent, ConsentGiver, RunState, now_utc},
    toolctx::ToolCtx,
};
use dla_workload::harness::run::{RunOptions, run};

fn have_sh() -> bool {
    Command::new("sh")
        .args(["-c", "true"])
        .status()
        .is_ok_and(|s| s.success())
}

fn ctx(dir: &Path, tiers: &[u8], expired: bool) -> ToolCtx {
    let project = dir.join("project");
    let run_dir = project.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(project.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    if !tiers.is_empty() {
        st.consent = Some(Consent {
            tiers: tiers.to_vec(),
            raw_values: false,
            given_by: ConsentGiver::Runner,
            recorded_at: now_utc(),
        });
    }
    if expired {
        st.budget.deadline = now_utc() - chrono_secs(60);
    }
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    ToolCtx {
        project,
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    }
}

fn chrono_secs(s: i64) -> std::time::Duration {
    std::time::Duration::from_secs(u64::try_from(s).unwrap())
}

fn manifest(argv: &[&str], build: Option<&[&str]>) -> HarnessManifest {
    let spec = |a: &[&str]| CommandSpec {
        argv: a.iter().map(|s| (*s).to_owned()).collect(),
        cwd: ".".into(),
        env: vec![],
    };
    HarnessManifest {
        schema_version: 1,
        id: "H1".into(),
        runtime: Runtime::Native,
        mode: HarnessMode::Command,
        workload: Some("test".into()),
        scratch: "harness/H1/work".into(),
        build: build.map(spec),
        run: spec(argv),
        iter_arg: Some("{n}".into()),
        warmup: 1,
        iterations: 2,
        step: 10,
        key_modes: vec![KeyMode::Native],
        probes: vec![],
        agent: AgentKind::None,
        detectors: vec![],
        tools: vec![],
        gaps: vec![],
        timeout_secs: 60,
        ffi: None,
        captures: vec![],
    }
}

fn install(ctx: &ToolCtx, m: &HarnessManifest) {
    fs::create_dir_all(ctx.run_dir.join("harness/H1/work")).unwrap();
    fs::create_dir_all(ctx.run_dir.join("harness/H1/agent")).unwrap();
    m.save(&ctx.run_dir).unwrap();
    fs::write(
        ctx.run_dir.join("harness/H1/probes.json"),
        "{\"probes\":[]}",
    )
    .unwrap();
}

fn attempt(ctx: &ToolCtx) -> HarnessAttempt {
    let text = fs::read_to_string(HarnessAttempt::path_in(&ctx.run_dir, "H1"))
        .expect("attempt.json was not written");
    serde_json::from_str(&text).unwrap()
}

#[test]
fn a_successful_run_writes_a_ran_attempt() {
    if !have_sh() {
        eprintln!("skipped: sh is not available");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let ctx = ctx(dir.path(), &[1], false);
    install(&ctx, &manifest(&["sh", "-c", "sleep 0.2; echo {n}"], None));
    assert!(!HarnessAttempt::path_in(&ctx.run_dir, "H1").exists());
    run(&ctx, "H1", &RunOptions::default()).unwrap();
    let a = attempt(&ctx);
    assert_eq!(a.harness, "H1");
    assert_eq!(a.outcome, AttemptOutcome::Ran);
    assert_eq!(a.reason, None);
    assert!(a.started_at <= a.finished_at, "{a:?}");
}

#[test]
fn a_failed_build_writes_a_failed_attempt_with_reason_build_fails() {
    if !have_sh() {
        eprintln!("skipped: sh is not available");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let ctx = ctx(dir.path(), &[1], false);
    install(
        &ctx,
        &manifest(&["true"], Some(&["sh", "-c", "echo boom >&2; exit 3"])),
    );
    let err = run(&ctx, "H1", &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "harness-failed");
    let a = attempt(&ctx);
    assert_eq!(a.outcome, AttemptOutcome::Failed);
    assert_eq!(a.reason.as_deref(), Some("build_fails"));
    assert!(a.message.unwrap().contains("did not build"));
}

#[test]
fn a_run_with_no_usable_point_writes_reason_no_harness() {
    if !have_sh() {
        eprintln!("skipped: sh is not available");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let ctx = ctx(dir.path(), &[1], false);
    install(&ctx, &manifest(&["sh", "-c", "exit 4"], None));
    let err = run(&ctx, "H1", &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "harness-failed");
    let a = attempt(&ctx);
    assert_eq!(a.outcome, AttemptOutcome::Failed);
    assert_eq!(a.reason.as_deref(), Some("no_harness"));
}

#[test]
fn a_refusal_before_any_point_writes_the_refusal_code() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = ctx(dir.path(), &[], false);
    install(&ctx, &manifest(&["true"], None));
    let err = run(&ctx, "H1", &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "consent-required");
    let a = attempt(&ctx);
    assert_eq!(a.outcome, AttemptOutcome::Failed);
    assert_eq!(a.reason.as_deref(), Some("consent-required"));
}

#[test]
fn an_expired_budget_writes_reason_time_budget() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = ctx(dir.path(), &[1], true);
    install(&ctx, &manifest(&["true"], None));
    let err = run(&ctx, "H1", &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "time-budget");
    let a = attempt(&ctx);
    assert_eq!(a.outcome, AttemptOutcome::Failed);
    assert_eq!(a.reason.as_deref(), Some("time_budget"));
}

#[test]
fn an_unknown_harness_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = ctx(dir.path(), &[1], false);
    assert!(run(&ctx, "H9", &RunOptions::default()).is_err());
    assert!(!HarnessAttempt::path_in(&ctx.run_dir, "H9").exists());
}

#[test]
fn a_failure_on_a_driver_with_template_hooks_writes_reason_hooks_unfilled() {
    if !have_sh() {
        eprintln!("skipped: sh is not available");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let ctx = ctx(dir.path(), &[1], false);
    install(&ctx, &manifest(&["sh", "-c", "exit 4"], None));
    let driver = ctx.run_dir.join("harness/H1/driver");
    fs::create_dir_all(&driver).unwrap();
    fs::write(
        driver.join("dla_harness.py"),
        "def setup():\n    # DLA-HOOK setup: build it\n    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")\n",
    )
    .unwrap();
    assert!(run(&ctx, "H1", &RunOptions::default()).is_err());
    assert_eq!(attempt(&ctx).reason.as_deref(), Some("hooks_unfilled"));

    // The hooks filled in (the comment stays): the same failure is a plain failed attempt.
    fs::write(
        driver.join("dla_harness.py"),
        "def setup():\n    # DLA-HOOK setup: build it\n    return 1\n",
    )
    .unwrap();
    assert!(run(&ctx, "H1", &RunOptions::default()).is_err());
    assert_eq!(attempt(&ctx).reason.as_deref(), Some("no_harness"));
}

#[test]
fn a_failing_junit_repeat_harness_does_not_send_the_caller_to_hooks() {
    if !have_sh() {
        eprintln!("skipped: sh is not available");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let c1 = ctx(dir.path(), &[1], false);
    let mut m = manifest(&["sh", "-c", "exit 4"], None);
    m.tools = vec!["junit-platform-launcher".into(), "gradle".into()];
    install(&c1, &m);
    let err = run(&c1, "H1", &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "harness-failed");
    assert!(
        !err.fix.contains("fix the harness or its hooks"),
        "{}",
        err.fix
    );
    assert!(err.fix.contains("tail of the logs"), "{}", err.fix);
    assert!(err.fix.contains("env_unavailable"), "{}", err.fix);
    assert!(err.fix.contains("intake.workload"), "{}", err.fix);

    // A harness that is not a JUnit repeat keeps the hooks wording.
    let dir = tempfile::tempdir().unwrap();
    let ctx2 = ctx(dir.path(), &[1], false);
    install(&ctx2, &manifest(&["sh", "-c", "exit 4"], None));
    let err = run(&ctx2, "H1", &RunOptions::default()).unwrap_err();
    assert!(
        err.fix.contains("fix the harness or its hooks"),
        "{}",
        err.fix
    );
}
