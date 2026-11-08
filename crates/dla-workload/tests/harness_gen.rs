//! Iteration detection, probe derivation, site resolution and `harness gen` against copies
//! of the benchmark fixtures (`bench/fixtures/*/project`, read-only).

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use dla_core::{
    Ledger, RunState,
    harness::{AgentKind, HarnessManifest, HarnessMode, ProbeTarget},
    runtime::Runtime,
    state::now_utc,
    toolctx::ToolCtx,
    workload::{WorkloadKind, WorkloadReport},
};
use dla_workload::{
    harness::{
        r#gen::{GenOptions, generate, on_path},
        probes::{python_module, resolve_site},
    },
    model::Scan,
};

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/fixtures")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

/// A copy of fixture `name` with an initialised run.
fn fixture(name: &str) -> (tempfile::TempDir, ToolCtx) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    copy_dir(&fixtures_root().join(name).join("project"), &project);
    let run_dir = project.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(project.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project,
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
    };
    (dir, ctx)
}

fn prepare(name: &str, rt: Runtime) -> (tempfile::TempDir, ToolCtx) {
    let (d, ctx) = fixture(name);
    dla_census::census(&ctx, rt).unwrap();
    dla_workload::discover(&ctx, Some(rt)).unwrap();
    (d, ctx)
}

fn opts(rt: Runtime) -> GenOptions {
    GenOptions {
        lang: rt,
        workload: None,
        driver: false,
        iter_arg: None,
        step: None,
        iterations: 8,
        warmup: 1,
        probe_findings: Vec::new(),
        auto_probes: true,
        package: None,
    }
}

fn env<'a>(c: &'a dla_core::harness::CommandSpec, k: &str) -> Option<&'a str> {
    c.env.iter().find(|(e, _)| e == k).map(|(_, v)| v.as_str())
}

fn probes_json(ctx: &ToolCtx, id: &str) -> serde_json::Value {
    let p = ctx.run_dir.join("harness").join(id).join("probes.json");
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

fn workload_evidence_names_harness(ctx: &ToolCtx, id: &str) {
    let ledger = ctx.ledger().unwrap();
    let hit = ledger.evidence().any(|e| {
        e.kind.as_str() == "workload" && dla_core::harness::harness_id_of(&e.result) == Some(id)
    });
    assert!(hit, "no workload evidence names {id}");
}

#[test]
fn every_fixture_main_workload_has_an_iteration_argument() {
    let mut seen = 0;
    let mut missing = Vec::new();
    for e in fs::read_dir(fixtures_root()).unwrap().flatten() {
        let project = e.path().join("project");
        if !project.is_dir() {
            continue;
        }
        let scan = Scan::new(&project);
        let (report, _) = dla_workload::build_report(&scan, None, &BTreeSet::new());
        let report: WorkloadReport = report;
        for w in report
            .workloads
            .iter()
            .filter(|w| matches!(w.kind, WorkloadKind::Main | WorkloadKind::Script))
        {
            seen += 1;
            if w.iteration_arg.is_none() {
                missing.push(format!("{}: {}", e.file_name().to_string_lossy(), w.id));
            }
        }
    }
    assert!(seen >= 18, "only {seen} main/script workloads");
    assert!(missing.is_empty(), "no iteration argument: {missing:?}");
}

#[test]
fn detected_arguments_and_defaults() {
    let want = [
        ("go-buffer-pool", "-n {n}", 2000),
        ("jvm-order-cache", "{n}", 5000),
        ("native-frame-decoder", "--iterations {n}", 2000),
        ("node-socket-heartbeat", "--iterations {n}", 2000),
        ("python-report-memo", "--iterations {n}", 2000),
        ("rust-batch-buffers", "--iterations {n}", 500),
    ];
    for (name, template, default) in want {
        let scan = Scan::new(&fixtures_root().join(name).join("project"));
        let (report, _) = dla_workload::build_report(&scan, None, &BTreeSet::new());
        let w = report
            .workloads
            .iter()
            .find(|w| matches!(w.kind, WorkloadKind::Main | WorkloadKind::Script))
            .unwrap();
        assert_eq!(w.iteration_arg.as_deref(), Some(template), "{name}");
        assert_eq!(w.iteration_default, Some(default), "{name}");
    }
}

#[test]
fn python_command_mode_harness() {
    let (_d, ctx) = prepare("python-report-memo", Runtime::Python);
    let out = generate(&ctx, &opts(Runtime::Python)).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.mode, HarnessMode::Command);
    assert_eq!(m.agent, AgentKind::PySitecustomize);
    assert_eq!(
        m.run.argv,
        ["python3", "-m", "reports.load", "--iterations", "{n}"]
    );
    let agent_dir = ctx.run_dir.join("harness/H1/agent");
    assert_eq!(
        env(&m.run, "PYTHONPATH"),
        Some(dla_core::paths::to_slash(&agent_dir).as_str())
    );
    assert!(agent_dir.join("sitecustomize.py").is_file());
    assert!(
        ctx.run_dir
            .join("harness/H1/work/reports/load.py")
            .is_file()
    );
    assert!(!ctx.run_dir.join("harness/H1/work/.dla").exists());
    assert_eq!((m.warmup, m.iterations, m.step), (1, 8, 440));
    assert!(
        m.detectors.is_empty()
            || m.detectors
                .iter()
                .all(|d| d.starts_with('-') || d.contains('='))
    );
    // Probes: the memoised method with its finding and site, plus the module.
    let attr = m
        .probes
        .iter()
        .find(|p| matches!(&p.target, ProbeTarget::PyAttr { qualname, .. } if qualname == "ReportBuilder.category_totals"))
        .expect("py-attr probe");
    assert_eq!(attr.site.as_deref(), Some("reports/builder.py:14"));
    assert!(attr.finding.is_some());
    assert!(m.probes.iter().any(
        |p| matches!(&p.target, ProbeTarget::PyModule { module } if module == "reports.builder")
    ));
    assert_eq!(
        probes_json(&ctx, "H1")["probes"].as_array().unwrap().len(),
        m.probes.len()
    );
    workload_evidence_names_harness(&ctx, "H1");
    assert_eq!(out.summary["harness"], "H1");
    assert_eq!(out.summary["next"], "dla harness run H1");
    // A second harness gets the next id.
    generate(&ctx, &opts(Runtime::Python)).unwrap();
    assert!(ctx.run_dir.join("harness/H2/harness.json").is_file());
}

#[test]
fn node_command_mode_resolves_the_npm_script() {
    let (_d, ctx) = prepare("node-retry-scheduler", Runtime::Node);
    generate(&ctx, &opts(Runtime::Node)).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.run.argv, ["node", "load.js", "--iterations", "{n}"]);
    let preload = env(&m.run, "NODE_OPTIONS").unwrap();
    assert!(
        preload.starts_with("--require ") && preload.ends_with("dla-preload.js"),
        "{preload}"
    );
    assert_eq!(env(&m.run, "NODE_V8_COVERAGE"), Some("{cov}"));
    assert!(
        ctx.run_dir
            .join("harness/H1/agent/dla-preload.js")
            .is_file()
    );
    let doc = probes_json(&ctx, "H1");
    let node = doc["probes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["target"]["kind"] == "node-module" && p["target"]["file"] == "src/scheduler.js")
        .expect("node-module probe for the scheduler");
    let names: Vec<&str> = node["target"]["names"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| n.as_str())
        .collect();
    assert!(names.contains(&"shared"), "{names:?}");
}

#[test]
fn go_command_mode_writes_the_test_wrapper() {
    let (_d, ctx) = prepare("go-buffer-pool", Runtime::Go);
    generate(&ctx, &opts(Runtime::Go)).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.agent, AgentKind::GoTestWrapper);
    let run = m.run.argv.join(" ");
    assert!(run.contains("-run ^TestDLAHarness$"), "{run}");
    assert!(run.ends_with("-coverpkg=./... ./cmd/invoiced"), "{run}");
    assert_eq!(env(&m.run, "DLA_ARGS"), Some(r#"["-n","{n}"]"#));
    let wrapper = ctx
        .run_dir
        .join("harness/H1/work/cmd/invoiced/zz_dla_harness_test.go");
    let text = fs::read_to_string(wrapper).unwrap();
    assert!(text.contains("package main") && !text.contains("@@DLA_"));
    assert!(m.probes.is_empty(), "go command mode has no GoVar probes");
    assert!(m.build.is_some());
}

#[test]
fn jvm_command_mode_builds_the_agent() {
    if !(on_path("javac") && on_path("jar")) {
        eprintln!("skipped: no JDK");
        return;
    }
    let (_d, ctx) = prepare("jvm-order-cache", Runtime::Jvm);
    generate(&ctx, &opts(Runtime::Jvm)).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.run.argv, ["sh", "./build.sh", "run", "{n}"]);
    assert!(m.build.is_none());
    let opts_text = env(&m.run, "JAVA_TOOL_OPTIONS").unwrap();
    assert!(
        opts_text.contains("-javaagent:") && opts_text.contains("dla-agent.jar"),
        "{opts_text}"
    );
    assert!(opts_text.contains("-XX:NativeMemoryTracking=summary"));
    assert!(ctx.run_dir.join("harness/H1/agent/dla-agent.jar").is_file());
    let stat = m
        .probes
        .iter()
        .find(|p| matches!(&p.target, ProbeTarget::JvmStatic { field, .. } if field == "ORDERS_BY_ID"))
        .expect("static probe");
    assert_eq!(stat.site.as_deref(), Some("src/shop/OrderCache.java:8"));
    assert!(stat.finding.is_some());
    assert!(m.probes.iter().any(
        |p| matches!(&p.target, ProbeTarget::JvmClass { class } if class == "shop.OrderCache")
    ));
    assert!(m.gaps.iter().any(|g| g.contains("unit tests")));
}

#[test]
fn rust_and_native_command_mode_use_the_shim() {
    if !on_path("cc") {
        eprintln!("skipped: no cc");
        return;
    }
    let (_d, ctx) = prepare("rust-batch-buffers", Runtime::Rust);
    generate(&ctx, &opts(Runtime::Rust)).unwrap();
    let m = ctx.harness("H1").unwrap();
    let build = m.build.as_ref().unwrap();
    assert_eq!(
        build.argv,
        ["cargo", "build", "--release", "--bin", "rollup-agent"]
    );
    assert_eq!(
        env(build, "CARGO_PROFILE_RELEASE_DEBUG"),
        Some("line-tables-only")
    );
    assert!(
        env(build, "CARGO_TARGET_DIR")
            .unwrap()
            .ends_with("harness/H1/work/target")
    );
    assert!(
        m.run.argv[0].ends_with("target/release/rollup-agent"),
        "{:?}",
        m.run.argv
    );
    assert_eq!(&m.run.argv[1..], ["--iterations", "{n}"]);
    assert_eq!(m.agent, AgentKind::AllocShim);
    if cfg!(target_os = "linux") {
        assert!(
            env(&m.run, "LD_PRELOAD")
                .unwrap()
                .ends_with("libdlashim.so")
        );
        assert!(ctx.run_dir.join("harness/H1/agent/libdlashim.so").is_file());
    }
    assert_eq!(m.step, 110);

    let (_d2, ctx2) = prepare("native-frame-decoder", Runtime::Native);
    generate(&ctx2, &opts(Runtime::Native)).unwrap();
    let n = ctx2.harness("H1").unwrap();
    assert_eq!(n.build.as_ref().unwrap().argv, ["make"]);
    assert_eq!(n.run.argv, ["./framedec", "--iterations", "{n}"]);
}

#[test]
fn driver_mode_instantiates_templates() {
    let (_d, ctx) = prepare("python-report-memo", Runtime::Python);
    let mut o = opts(Runtime::Python);
    o.driver = true;
    let out = generate(&ctx, &o).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.mode, HarnessMode::Driver);
    assert_eq!(m.agent, AgentKind::DriverPython);
    assert_eq!(m.key_modes.len(), 2);
    let file = ctx.run_dir.join("harness/H1/driver/dla_harness.py");
    let text = fs::read_to_string(file).unwrap();
    assert!(!text.contains("@@DLA_") && text.contains("DLA-HOOK setup"));
    assert_eq!(out.summary["hooks"], serde_json::json!(["setup", "unit"]));
    assert!(m.iter_arg.is_none());

    let (_d, ctx) = prepare("node-retry-scheduler", Runtime::Node);
    let mut o = opts(Runtime::Node);
    o.driver = true;
    generate(&ctx, &o).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.agent, AgentKind::DriverNode);
    assert!(env(&m.run, "DLA_AGENT").is_some());

    let (_d, ctx) = prepare("go-subscription-workers", Runtime::Go);
    let mut o = opts(Runtime::Go);
    o.driver = true;
    generate(&ctx, &o).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.agent, AgentKind::DriverGo);
    assert!(m.run.argv.join(" ").contains("^TestDLADriver$"));
    let placed = fs::read_dir(ctx.run_dir.join("harness/H1/work/internal/workload"))
        .into_iter()
        .chain(fs::read_dir(
            ctx.run_dir.join("harness/H1/work/internal/notify"),
        ))
        .flatten()
        .flatten()
        .any(|e| e.file_name() == "zz_dla_driver_test.go");
    assert!(placed, "driver test not copied into a package");
    assert!(
        m.probes
            .iter()
            .any(|p| matches!(p.target, ProbeTarget::GoVar { .. }))
    );

    if on_path("javac") && on_path("jar") {
        let (_d, ctx) = prepare("jvm-order-cache", Runtime::Jvm);
        let mut o = opts(Runtime::Jvm);
        o.driver = true;
        generate(&ctx, &o).unwrap();
        let m = ctx.harness("H1").unwrap();
        assert_eq!(m.agent, AgentKind::DriverJvm);
        let driver = ctx.run_dir.join("harness/H1/driver/DlaLoop.java");
        assert!(
            fs::read_to_string(driver)
                .unwrap()
                .starts_with("package shop;")
        );
        // The project's own build never sees the driver.
        assert!(
            !ctx.run_dir
                .join("harness/H1/work/src/shop/DlaLoop.java")
                .exists()
        );
        assert_eq!(m.run.argv.last().map(String::as_str), Some("shop.DlaLoop"));
        let build = &m.build.as_ref().unwrap().argv[2];
        assert!(
            build.starts_with("sh ./build.sh && javac -cp out:")
                && build.ends_with("driver/DlaLoop.java"),
            "{build}"
        );
    }
}

#[test]
fn rust_driver_refuses_a_bin_only_crate() {
    let (_d, ctx) = prepare("rust-batch-buffers", Runtime::Rust);
    let mut o = opts(Runtime::Rust);
    o.driver = true;
    let r = generate(&ctx, &o).unwrap_err();
    assert_eq!(r.code, "driver-unavailable", "{r}");
    assert!(r.fix.contains("dla harness gen --lang rust"), "{r}");
    // The failed attempt leaves no half-written harness behind.
    assert!(!ctx.run_dir.join("harness/H1").exists());
}

#[test]
fn refusals_name_the_missing_item_and_the_fix() {
    let (_d, ctx) = prepare("python-report-memo", Runtime::Python);
    // Explicit test workload: no iteration argument.
    let mut o = opts(Runtime::Python);
    o.workload = Some("python:test:tests/test_reports.py".into());
    let r = generate(&ctx, &o).unwrap_err();
    assert_eq!(r.code, "iteration-arg-unknown", "{r}");
    assert!(r.fix.contains("--iter-arg"), "{r}");
    // --iter-arg without {n}.
    let mut o = opts(Runtime::Python);
    o.iter_arg = Some("--iterations".into());
    assert_eq!(generate(&ctx, &o).unwrap_err().code, "invalid-value");
    // No go workloads in a python project.
    let r = generate(&ctx, &opts(Runtime::Go)).unwrap_err();
    assert_eq!(r.code, "workload-required", "{r}");
    // Unknown workload id.
    let mut o = opts(Runtime::Python);
    o.workload = Some("python:main:nope".into());
    assert_eq!(generate(&ctx, &o).unwrap_err().code, "invalid-value");
    // Unknown --probe finding.
    let mut o = opts(Runtime::Python);
    o.probe_findings = vec!["F99".parse().unwrap()];
    assert_eq!(generate(&ctx, &o).unwrap_err().code, "unknown-finding");
    assert!(
        !ctx.run_dir.join("harness").exists()
            || fs::read_dir(ctx.run_dir.join("harness")).unwrap().count() == 0
    );
}

#[test]
fn explicit_iter_arg_and_step_win() {
    let (_d, ctx) = prepare("python-report-memo", Runtime::Python);
    let mut o = opts(Runtime::Python);
    o.iter_arg = Some("-n {n}".into());
    o.step = Some(7);
    o.iterations = 4;
    o.warmup = 2;
    o.auto_probes = false;
    generate(&ctx, &o).unwrap();
    let m = ctx.harness("H1").unwrap();
    // The workload's own --iterations value is not replaced by a different template: the
    // given template is appended.
    assert_eq!(m.run.argv, ["python3", "-m", "reports.load", "-n", "{n}"]);
    assert_eq!((m.warmup, m.iterations, m.step), (2, 4, 7));
    assert!(m.probes.is_empty());
}

#[test]
fn site_resolution_uses_the_census_then_declarations() {
    let (_d, ctx) = prepare("python-ingest-two-leaks", Runtime::Python);
    generate(&ctx, &opts(Runtime::Python)).unwrap();
    let m = ctx.harness("H1").unwrap();
    let (site, finding) = resolve_site(&ctx, &m, "ingest.registry._LISTENERS");
    assert_eq!(site.as_deref(), Some("ingest/registry.py:3"));
    assert!(finding.is_some());
    // Not a census holder: found by its declaration line, no finding.
    let (site, finding) = resolve_site(&ctx, &m, "ingest.parser.parse_line");
    assert_eq!(
        site.as_deref().map(|s| s.starts_with("ingest/parser.py:")),
        Some(true)
    );
    assert!(finding.is_none());
    assert_eq!(
        resolve_site(&ctx, &m, "ingest.nothing.at_all"),
        (None, None)
    );
    assert_eq!(
        python_module(&ctx.project, "ingest/registry.py"),
        "ingest.registry"
    );

    let (_d, ctx) = prepare("node-retry-scheduler", Runtime::Node);
    generate(&ctx, &opts(Runtime::Node)).unwrap();
    let m = ctx.harness("H1").unwrap();
    // The property breakdown resolves to the field that holds the growth.
    let (site, finding) = resolve_site(&ctx, &m, "src/scheduler.js:shared.latest");
    assert_eq!(site.as_deref(), Some("src/scheduler.js:15"));
    assert!(finding.is_some());
    let (site, finding) = resolve_site(&ctx, &m, "src/scheduler.js:shared");
    assert_eq!(site.as_deref(), Some("src/scheduler.js:90"));
    assert!(finding.is_some());
    let (site, _) = resolve_site(&ctx, &m, "src/pending.js:PendingList");
    assert!(site.is_some_and(|s| s.starts_with("src/pending.js:")));

    let (_d, ctx) = prepare("jvm-order-cache", Runtime::Jvm);
    let m = HarnessManifest {
        probes: Vec::new(),
        ..jvm_manifest(&ctx)
    };
    let (site, finding) = resolve_site(&ctx, &m, "shop.OrderCache.ORDERS_BY_ID");
    assert_eq!(site.as_deref(), Some("src/shop/OrderCache.java:8"));
    assert!(finding.is_some());
    // A static the census did not list is found in the source.
    let (site, finding) = resolve_site(&ctx, &m, "shop.OrderCache.MAX_ENTRIES");
    assert!(finding.is_none());
    let _ = site;
}

fn jvm_manifest(ctx: &ToolCtx) -> HarnessManifest {
    use dla_core::harness::CommandSpec;
    HarnessManifest {
        schema_version: 1,
        id: "H9".into(),
        runtime: Runtime::Jvm,
        mode: HarnessMode::Command,
        workload: None,
        scratch: "harness/H9/work".into(),
        build: None,
        run: CommandSpec {
            argv: vec!["java".into()],
            cwd: ".".into(),
            env: Vec::new(),
        },
        iter_arg: None,
        warmup: 1,
        iterations: 8,
        step: 1,
        key_modes: Vec::new(),
        probes: Vec::new(),
        agent: AgentKind::JvmAgent,
        detectors: Vec::new(),
        tools: Vec::new(),
        gaps: Vec::new(),
        timeout_secs: 300,
    }
    .tap(|_| {
        let _ = ctx;
    })
}

trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }
}
impl<T> Tap for T {}

#[test]
fn probe_flag_adds_the_holder_of_a_finding_and_none_skips_the_census() {
    let (_d, ctx) = prepare("python-ingest-two-leaks", Runtime::Python);
    let ledger = ctx.ledger().unwrap();
    let pipeline = ledger
        .findings()
        .find(|f| f.site.as_deref() == Some("ingest/pipeline.py:15"))
        .expect("census finding for Pipeline.per_sensor")
        .id
        .clone();
    drop(ledger);
    let mut o = opts(Runtime::Python);
    o.auto_probes = false;
    o.probe_findings = vec![pipeline.clone()];
    generate(&ctx, &o).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.probes.len(), 1, "{:?}", m.probes);
    assert_eq!(m.probes[0].finding.as_ref(), Some(&pipeline));
    assert!(
        matches!(&m.probes[0].target, ProbeTarget::PyModule { module } if module == "ingest.pipeline")
    );
    // A finding whose site is another runtime's code cannot be probed here.
    let r = dla_workload::harness::probes::derive(
        &ctx,
        Runtime::Node,
        false,
        false,
        &m.probes[0].finding.iter().cloned().collect::<Vec<_>>(),
        None,
    )
    .unwrap_err();
    assert_eq!(r.code, "invalid-value", "{r}");
    assert!(r.message.contains("not node code"), "{r}");
}

#[test]
fn a_workload_below_the_project_root_keeps_its_cwd() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    fs::create_dir_all(project.join("svc/src")).unwrap();
    fs::write(
        project.join("svc/package.json"),
        r#"{"name":"svc","scripts":{"load":"node load.js --iterations 300"}}"#,
    )
    .unwrap();
    fs::write(
        project.join("svc/load.js"),
        "const args = { iterations: 300 };\nfor (let i = 2; i < process.argv.length; i += 2) { args[process.argv[i].replace(/^--/, '')] = Number(process.argv[i + 1]); }\nconsole.log(args);\n",
    )
    .unwrap();
    let run_dir = project.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(project.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project,
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
    };
    dla_workload::discover(&ctx, Some(Runtime::Node)).unwrap();
    generate(&ctx, &opts(Runtime::Node)).unwrap();
    let m = ctx.harness("H1").unwrap();
    assert_eq!(m.run.cwd, "svc");
    assert_eq!(m.run.argv, ["node", "load.js", "--iterations", "{n}"]);
    assert_eq!(
        m.step, 70,
        "twice the visible default of 300 over nine points"
    );
    assert!(ctx.run_dir.join("harness/H1/work/svc/load.js").is_file());
}

#[test]
fn a_non_git_project_keeps_nested_build_packages_and_reports_what_it_skipped() {
    let (_d, ctx) = prepare("python-report-memo", Runtime::Python);
    let p = &ctx.project;
    fs::create_dir_all(p.join("reports/build")).unwrap();
    fs::write(p.join("reports/build/gen.py"), "X = 1\n").unwrap();
    fs::create_dir_all(p.join("build")).unwrap();
    fs::write(p.join("build/out.o"), "o").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("reports/build/gen.py", p.join("alias.py")).unwrap();
        std::os::unix::fs::symlink("/etc/hostname", p.join("host.txt")).unwrap();
    }
    generate(&ctx, &opts(Runtime::Python)).unwrap();
    let work = ctx.run_dir.join("harness/H1/work");
    assert!(work.join("reports/build/gen.py").is_file());
    assert!(!work.join("build").exists());
    let m = ctx.harness("H1").unwrap();
    assert!(
        m.gaps
            .iter()
            .any(|g| g.contains("build-output") && g.contains("build")),
        "{:?}",
        m.gaps
    );
    #[cfg(unix)]
    {
        assert_eq!(
            fs::read_to_string(work.join("alias.py")).unwrap(),
            "X = 1\n"
        );
        assert!(!work.join("host.txt").exists());
        assert!(
            m.gaps.iter().any(|g| g.contains("outside the project")),
            "{:?}",
            m.gaps
        );
    }
}
