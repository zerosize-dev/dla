use std::{fs, os::unix::fs::PermissionsExt};

use dla_core::{
    CoverageManifest, Ledger, RunState,
    coverage::BackendStatus,
    error::codes,
    probe::BuildStatus,
    state::{Consent, ConsentGiver, now_utc},
};

use super::*;

fn setup(tiers: Option<Vec<u8>>, files: &[(&str, &str)]) -> (tempfile::TempDir, ToolCtx) {
    let dir = tempfile::tempdir().unwrap();
    for (p, c) in files {
        let path = dir.path().join(p);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, c).unwrap();
    }
    let run_dir = dir.path().join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st = RunState::init(
        dir.path().to_str().unwrap(),
        Some("s"),
        3600,
        None,
        now_utc(),
    )
    .unwrap();
    st.run_id = "r1".into();
    st.consent = tiers.map(|tiers| Consent {
        tiers,
        raw_values: false,
        given_by: ConsentGiver::Runner,
        recorded_at: now_utc(),
    });
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project: dir.path().to_path_buf(),
        run_dir,
        run_id: "r1".into(),
        source_commit: "c".into(),
        data_flag: None,
        worker: None,
    };
    (dir, ctx)
}

fn empty_dirs() -> (tempfile::TempDir, OsString) {
    let t = tempfile::tempdir().unwrap();
    let pv = OsString::from(t.path().as_os_str());
    (t, pv)
}

#[test]
fn build_without_consent_refuses_and_writes_nothing() {
    let (_d, ctx) = setup(None, &[("go.mod", "go 1.22\n")]);
    let e = run(
        &ctx,
        &ProbeOptions {
            build: true,
            ..ProbeOptions::default()
        },
    )
    .unwrap_err();
    assert_eq!(e.code, codes::CONSENT_REQUIRED);
    assert_eq!(e.fix, "dla consent --tiers 1-4");
    assert!(!ctx.run_dir.join("probe").exists());
}

#[test]
fn probe_writes_report_evidence_and_coverage_with_missing_hints() {
    let (_d, ctx) = setup(
        None,
        &[
            ("go.mod", "module m\n\ngo 1.22\n"),
            ("m_test.go", "package m\nfunc TestA(t *testing.T) {}\n"),
            ("app.py", "import asyncio\n"),
        ],
    );
    let (_e, pv) = empty_dirs();
    let out = run_with(&ctx, &ProbeOptions::default(), None, Some(&pv)).unwrap();
    assert_eq!(out.candidates, 0);
    assert_eq!(out.evidence.len(), 1);

    let rep = load_report(&ctx.run_dir).unwrap();
    let rts: Vec<_> = rep.runtimes.iter().map(|r| r.runtime).collect();
    assert_eq!(rts, vec![Runtime::Go, Runtime::Python]);
    assert!(rep.builds.is_empty());
    assert!(rep.libraries.iter().any(|l| l.library == "asyncio"));
    assert!(rep.test_runners.iter().any(|t| t.name == "go test"));

    let ledger = ctx.ledger().unwrap();
    let ev = ledger.evidence().next().unwrap();
    assert_eq!(ev.kind, EvidenceKind::Probe);
    assert_eq!(ev.raw.as_deref(), Some("probe/probe.json"));

    let cov = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
    let go = &cov.workers["go"].backends;
    assert_eq!(go["staticcheck"].status, BackendStatus::Missing);
    assert_eq!(
        go["staticcheck"].install_hint.as_deref(),
        Some("dla install staticcheck")
    );
    // The built-in rules have no row until a sweep runs them.
    assert!(!go.contains_key("dla"));
    let py = &cov.workers["python"].backends;
    assert_eq!(py["ruff"].install_hint.as_deref(), Some("dla install ruff"));
    assert_eq!(py["codeql"].status, BackendStatus::Missing);
    assert!(!cov.workers.contains_key("jvm"));
}

#[test]
fn a_tool_in_the_tools_dir_is_recorded_as_available_with_its_version() {
    let (_d, ctx) = setup(None, &[("a.py", "x = 1\n")]);
    let tools = tempfile::tempdir().unwrap();
    let bin = tools.path().join("venv/bin");
    fs::create_dir_all(&bin).unwrap();
    let ruff = bin.join("ruff");
    fs::write(&ruff, "#!/bin/sh\necho 'ruff 0.9.9'\n").unwrap();
    fs::set_permissions(&ruff, fs::Permissions::from_mode(0o755)).unwrap();
    let (_e, pv) = empty_dirs();
    run_with(
        &ctx,
        &ProbeOptions::default(),
        Some(tools.path()),
        Some(&pv),
    )
    .unwrap();
    let cov = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
    let r = &cov.workers["python"].backends["ruff"];
    assert_eq!(r.status, BackendStatus::Ran);
    assert_eq!(r.version.as_deref(), Some("ruff 0.9.9"));
    let rep = load_report(&ctx.run_dir).unwrap();
    let t = rep.tools.iter().find(|t| t.name == "ruff").unwrap();
    assert_eq!(t.source.as_deref(), Some("tools-dir"));
}

#[test]
fn probe_does_not_downgrade_coverage_a_sweep_already_wrote() {
    let (_d, ctx) = setup(None, &[("go.mod", "go 1.22\n")]);
    CoverageManifest::update(&ctx.run_dir, |m| {
        m.set_backend(
            "go",
            "staticcheck",
            BackendCoverage::ran(Some("1".into()), 7),
        );
        m.set_backend(
            "go",
            "golangci-lint",
            BackendCoverage::blocked("compile error"),
        );
    })
    .unwrap();
    let (_e, pv) = empty_dirs();
    run_with(&ctx, &ProbeOptions::default(), None, Some(&pv)).unwrap();
    let cov = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
    assert_eq!(cov.workers["go"].backends["staticcheck"].enumerated, 7);
    assert_eq!(
        cov.workers["go"].backends["golangci-lint"].status,
        BackendStatus::Blocked
    );
}

#[test]
fn build_runs_with_consent_and_records_failure() {
    let (_d, ctx) = setup(
        Some(vec![1]),
        &[
            (
                "build.sh",
                "echo 'cannot resolve org.example:private:1.0' 1>&2\nexit 1\n",
            ),
            ("A.java", "class A {}"),
        ],
    );
    let (_e, pv) = empty_dirs();
    let opts = ProbeOptions {
        build: true,
        ..ProbeOptions::default()
    };
    let out = run_with(&ctx, &opts, None, Some(&pv)).unwrap();
    assert_eq!(out.summary["builds"]["jvm"], "failed");
    let rep = load_report(&ctx.run_dir).unwrap();
    assert!(build_failed(&rep, Runtime::Jvm));
    assert_eq!(rep.builds[0].status, BuildStatus::Failed);
    assert!(
        rep.builds[0]
            .log_tail
            .as_deref()
            .unwrap()
            .contains("cannot resolve")
    );
}

#[test]
fn the_evidence_result_stays_within_the_size_limit_for_a_large_project() {
    let (_d, ctx) = setup(
        None,
        &[
            (
                "pom.xml",
                "io.netty com.zaxxer.hikari io.micrometer org.apache.catalina",
            ),
            ("A.java", ""),
            ("go.mod", "go 1.22\n"),
            ("Cargo.toml", ""),
            ("package.json", "{}"),
            ("a.py", "import asyncio"),
            ("Makefile", "test:\n"),
            ("a.c", ""),
        ],
    );
    let (_e, pv) = empty_dirs();
    run_with(&ctx, &ProbeOptions::default(), None, Some(&pv)).unwrap();
    assert_eq!(ctx.ledger().unwrap().evidence().count(), 1);
}

#[test]
fn an_empty_project_probes_cleanly() {
    let (_d, ctx) = setup(None, &[("README", "hi")]);
    let (_e, pv) = empty_dirs();
    let out = run_with(&ctx, &ProbeOptions::default(), None, Some(&pv)).unwrap();
    assert_eq!(out.summary["runtimes"], serde_json::json!([]));
}

#[test]
fn a_worker_probes_only_its_runtime_into_its_own_file() {
    let (_d, mut ctx) = setup(
        None,
        &[("go.mod", "module m\n\ngo 1.22\n"), ("app.py", "x = 1\n")],
    );
    let (_e, pv) = empty_dirs();
    // The core's run-level probe.
    run_with(&ctx, &ProbeOptions::default(), None, Some(&pv)).unwrap();
    let core = fs::read(report_path(&ctx.run_dir)).unwrap();

    ctx.worker = Some("go".into());
    let opts = ProbeOptions {
        runtime: Some(Runtime::Go),
        ..ProbeOptions::default()
    };
    let out = run_with(&ctx, &opts, None, Some(&pv)).unwrap();
    assert_eq!(out.artifacts, [".dla/r1/probe/go.json"]);
    assert_eq!(fs::read(report_path(&ctx.run_dir)).unwrap(), core);
    let own: ProbeReport =
        serde_json::from_slice(&fs::read(ctx.run_dir.join("probe/go.json")).unwrap()).unwrap();
    let rts: Vec<_> = own.runtimes.iter().map(|r| r.runtime).collect();
    assert_eq!(rts, vec![Runtime::Go]);
    assert_eq!(
        report_path_for(&ctx.run_dir, Some("go")),
        ctx.run_dir.join("probe/go.json")
    );
    assert_eq!(
        report_path_for(&ctx.run_dir, Some("python")),
        report_path(&ctx.run_dir)
    );
}

#[test]
fn build_after_the_deadline_refuses_with_the_time_budget_code() {
    let (_d, ctx) = setup(Some(vec![1]), &[("go.mod", "go 1.22\n")]);
    RunState::update(&ctx.run_dir, |s| {
        s.budget.deadline = now_utc() - chrono::Duration::seconds(5);
        Ok(())
    })
    .unwrap();
    let e = run(
        &ctx,
        &ProbeOptions {
            build: true,
            ..ProbeOptions::default()
        },
    )
    .unwrap_err();
    assert_eq!(e.code, codes::TIME_BUDGET);
    assert!(!ctx.run_dir.join("probe").exists());
}
