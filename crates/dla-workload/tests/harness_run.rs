//! End-to-end tests of `dla harness run` on copies of the benchmark fixtures.
//!
//! Unix only: on Windows the harness has no allocator shim, no process groups and no
//! `/proc`, and its command-mode support is a recorded gap (docs/VERIFY.md).
//!
//! Each test writes a manifest by hand (the way `dla harness gen` does), runs
//! [`run_points`] and checks the produced series: leak fixtures grow in the column that
//! names the leak, their paired controls stay flat. A test prints why it is skipped when
//! the toolchain it needs is missing.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use dla_core::{
    FindingId, Ledger,
    harness::{AgentKind, CommandSpec, HarnessManifest, HarnessMode, ProbeSpec, ProbeTarget},
    runtime::Runtime,
    series::{KeyMode, SeriesFile},
    state::{Consent, ConsentGiver, RunState, now_utc},
    toolctx::ToolCtx,
};
use dla_stats::{GateThresholds, GateVerdict, analyze};
use dla_workload::harness::run::{RunOptions, RunOutput, run, run_points};
use tempfile::TempDir;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Whether `tool` starts; prints the skip reason when not.
fn have(tools: &[&str]) -> bool {
    for t in tools {
        let args: &[&str] = match *t {
            "go" => &["version"],
            "java" | "javac" => &["-version"],
            "sh" => &["-c", "true"],
            _ => &["--version"],
        };
        let ok = Command::new(t)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            eprintln!("skipped: {t} is not available");
            return false;
        }
    }
    true
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if src.is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

struct Fx {
    _dir: TempDir,
    ctx: ToolCtx,
    hdir: PathBuf,
    work: PathBuf,
    agent: PathBuf,
}

/// A run directory with consent for `tiers`, and harness `H1` holding a scratch copy of
/// the fixture project and the agent files of `lang_dirs` (under `harness/`).
fn fixture(name: &str, lang_dirs: &[&str], tiers: &[u8]) -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
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
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project,
        run_dir: run_dir.clone(),
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    };
    let hdir = run_dir.join("harness/H1");
    let work = hdir.join("work");
    let agent = hdir.join("agent");
    fs::create_dir_all(&agent).unwrap();
    if name.is_empty() {
        fs::create_dir_all(&work).unwrap();
    } else {
        copy_dir(
            &repo().join("bench/fixtures").join(name).join("project"),
            &work,
        );
    }
    for l in lang_dirs {
        for e in fs::read_dir(repo().join("harness").join(l)).unwrap() {
            let e = e.unwrap();
            if e.path().is_file() {
                fs::copy(e.path(), agent.join(e.file_name())).unwrap();
            }
        }
    }
    Fx {
        _dir: dir,
        ctx,
        hdir,
        work,
        agent,
    }
}

fn manifest(
    runtime: Runtime,
    argv: &[&str],
    env: Vec<(String, String)>,
    step: u64,
) -> HarnessManifest {
    HarnessManifest {
        schema_version: 1,
        id: "H1".into(),
        runtime,
        mode: HarnessMode::Command,
        workload: Some("test".into()),
        scratch: "harness/H1/work".into(),
        build: None,
        run: CommandSpec {
            argv: argv.iter().map(|s| (*s).to_owned()).collect(),
            cwd: ".".into(),
            env,
        },
        iter_arg: Some("{n}".into()),
        warmup: 1,
        iterations: 8,
        step,
        key_modes: vec![KeyMode::Native],
        probes: vec![],
        agent: AgentKind::None,
        detectors: vec![],
        tools: vec![],
        gaps: vec![],
        timeout_secs: 120,
        ffi: None,
        captures: vec![],
    }
}

fn spec(argv: &[&str]) -> CommandSpec {
    CommandSpec {
        argv: argv.iter().map(|s| (*s).to_owned()).collect(),
        cwd: ".".into(),
        env: vec![],
    }
}

fn s(v: &str) -> String {
    v.to_owned()
}

fn write_manifest(fx: &Fx, m: &HarnessManifest) {
    m.save(&fx.ctx.run_dir).unwrap();
    let probes = serde_json::json!({"probes": m.probes});
    fs::write(fx.hdir.join("probes.json"), probes.to_string()).unwrap();
}

/// Post-warm-up `(x, y)` of a column.
fn post(series: &SeriesFile, name: &str) -> Vec<(f64, f64)> {
    let i = series.column_index(name).unwrap_or_else(|| {
        panic!(
            "no column {name}; have {:?}",
            series.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
        )
    });
    series.post_warmup(i)
}

fn passes(series: &SeriesFile, name: &str) -> bool {
    let i = series.column_index(name).unwrap();
    let counter = series.columns[i].kind == dla_core::series::ColumnKind::Counter;
    let pts = post(series, name);
    let tail = (series.warmup > 0)
        .then(|| series.points.get(series.warmup as usize - 1))
        .flatten()
        .and_then(|p| Some((p.x, p.values.get(i).copied().flatten()?)));
    analyze(&pts, tail, counter, &GateThresholds::default()).gate == GateVerdict::Pass
}

fn ratio(series: &SeriesFile, name: &str) -> f64 {
    let p = post(series, name);
    p.last().unwrap().1 / p.first().unwrap().1.max(1.0)
}

fn load(fx: &Fx, out: &RunOutput, i: usize) -> SeriesFile {
    let s = SeriesFile::load(&fx.ctx.run_dir, &out.series[i]).unwrap();
    // DLA_TEST_KEEP=<dir> keeps a copy of every series read, for debugging.
    if let Ok(dir) = std::env::var("DLA_TEST_KEEP") {
        let n = std::fs::read_dir(&dir).map_or(0, Iterator::count);
        let path = Path::new(&dir).join(format!("{n}-{:?}.json", s.source.runtime));
        let _ = fs::write(path, serde_json::to_string_pretty(&s).unwrap());
    }
    s
}

/// Compiles the JVM agent of `fx` into `agent/dla-agent.jar`.
fn build_jvm_agent(fx: &Fx) -> PathBuf {
    let classes = fx.agent.join("classes");
    fs::create_dir_all(&classes).unwrap();
    assert!(
        Command::new("javac")
            .arg("-d")
            .arg(&classes)
            .arg(fx.agent.join("DlaAgent.java"))
            .arg(fx.agent.join("DlaDump.java"))
            .arg(fx.agent.join("DlaCapture.java"))
            .env_remove("JAVA_TOOL_OPTIONS")
            .status()
            .unwrap()
            .success()
    );
    let jar = fx.agent.join("dla-agent.jar");
    assert!(
        Command::new("jar")
            .args(["--create", "--file"])
            .arg(&jar)
            .arg("--manifest")
            .arg(fx.agent.join("MANIFEST.MF"))
            .arg("-C")
            .arg(&classes)
            .arg(".")
            .env_remove("JAVA_TOOL_OPTIONS")
            .status()
            .unwrap()
            .success()
    );
    jar
}

#[test]
fn python_report_memo_leaks_in_its_probe_and_the_bounded_control_does_not() {
    if !have(&["python3"]) {
        return;
    }
    let mut results = Vec::new();
    for fixture_name in ["python-report-memo", "python-report-memo-bounded"] {
        let fx = fixture(fixture_name, &["python"], &[1]);
        let mut m = manifest(
            Runtime::Python,
            &["python3", "-m", "reports.load", "--iterations", "{n}"],
            vec![(s("PYTHONPATH"), fx.agent.to_string_lossy().into_owned())],
            250,
        );
        m.agent = AgentKind::PySitecustomize;
        m.probes = vec![ProbeSpec {
            id: s("P1"),
            finding: Some(FindingId::new(1)),
            target: ProbeTarget::PyAttr {
                module: s("reports.builder"),
                qualname: s("ReportBuilder.category_totals"),
            },
            site: Some(s("reports/builder.py:18")),
        }];
        write_manifest(&fx, &m);
        let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
        assert_eq!(out.series.len(), 1);
        assert_eq!(out.points, 9);
        let series = load(&fx, &out, 0);
        assert_eq!(series.warmup, 1);
        assert_eq!(series.points.len(), 9);
        assert_eq!(series.points[8].x, 2250.0);
        let probe = series
            .columns
            .iter()
            .find(|c| c.name.starts_with("probe:") && c.name.contains("category_totals"))
            .unwrap_or_else(|| {
                panic!(
                    "no probe column in {:?}",
                    series.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
                )
            })
            .clone();
        assert_eq!(probe.site.as_deref(), Some("reports/builder.py:18"));
        assert_eq!(probe.finding, Some(FindingId::new(1)));
        results.push((
            passes(&series, &probe.name),
            ratio(&series, "py.traced"),
            series,
        ));
    }
    let (leak_gate, leak_ratio, leak) = &results[0];
    let (ctl_gate, ctl_ratio, _) = &results[1];
    assert!(leak_gate, "the lru_cache probe must grow every point");
    assert!(
        *leak_ratio > 2.0,
        "py.traced of the leak grows: {leak_ratio}"
    );
    assert!(!ctl_gate, "the bounded cache stays at its maxsize");
    assert!(
        *ctl_ratio < 1.2,
        "py.traced of the control is flat: {ctl_ratio}"
    );
    assert!(leak.column_index("os.vm_hwm").is_some());
    assert!(leak.column_index("py.traced.pre_gc").is_some());
}

#[test]
fn python_run_writes_workload_evidence_coverage_and_process_records() {
    if !have(&["python3"]) {
        return;
    }
    let fx = fixture("python-report-memo", &["python"], &[1]);
    // Entry points as `dla workload discover` lists them.
    dla_core::coverage::CoverageManifest::update(&fx.ctx.run_dir, |c| {
        c.set_entry_points(
            "python",
            dla_core::coverage::EntryPoints {
                discovered: vec![
                    s("reports/builder.py::ReportBuilder.category_totals"),
                    s("reports/store.py::never_called_here"),
                ],
                ..Default::default()
            },
        );
    })
    .unwrap();
    let mut m = manifest(
        Runtime::Python,
        &["python3", "-m", "reports.load", "--iterations", "{n}"],
        vec![(s("PYTHONPATH"), fx.agent.to_string_lossy().into_owned())],
        250,
    );
    m.iterations = 3;
    write_manifest(&fx, &m);
    let o = run(&fx.ctx, "H1", &RunOptions::default()).unwrap();
    assert_eq!(o.summary["harness"], "H1");
    assert_eq!(o.summary["next"], "dla trend S1");
    let ledger = fx.ctx.ledger().unwrap();
    let ev = ledger.evidence().last().unwrap().clone();
    assert_eq!(ev.kind, dla_core::EvidenceKind::Workload);
    assert_eq!(ev.authority, dla_core::Authority::Test);
    assert_eq!(dla_core::harness::harness_id_of(&ev.result), Some("H1"));
    assert!(serde_json::to_vec(&ev.result).unwrap().len() <= 2048);
    assert_eq!(ev.result["series"][0], "S1");
    let cov = dla_core::CoverageManifest::load(&fx.ctx.run_dir)
        .unwrap()
        .unwrap();
    let w = &cov.workers["python"];
    assert_eq!(w.iterations, Some(750));
    assert!(
        w.entry_points
            .exercised
            .contains(&s("reports/builder.py::ReportBuilder.category_totals")),
        "exercised: {:?}",
        w.entry_points.exercised
    );
    assert!(
        !w.entry_points
            .exercised
            .iter()
            .any(|e| e.contains("never_called"))
    );
    let state = RunState::load(&fx.ctx.run_dir).unwrap();
    assert_eq!(state.processes.len(), 4);
    assert!(state.processes.iter().all(|p| p.stopped_at.is_some()));
}

#[test]
fn node_timers_are_counted_by_site_and_the_cleared_control_has_none() {
    if !have(&["node"]) {
        return;
    }
    let mut results = Vec::new();
    for name in ["node-socket-heartbeat", "node-socket-heartbeat-cleared"] {
        let fx = fixture(name, &["node"], &[1]);
        let preload = fx.agent.join("dla-preload.js");
        let mut m = manifest(
            Runtime::Node,
            &["node", "load.js", "--iterations", "{n}"],
            vec![
                (
                    s("NODE_OPTIONS"),
                    format!("--require {}", preload.display()),
                ),
                (s("NODE_V8_COVERAGE"), s("{cov}")),
            ],
            500,
        );
        m.agent = AgentKind::NodePreload;
        write_manifest(&fx, &m);
        dla_core::coverage::CoverageManifest::update(&fx.ctx.run_dir, |c| {
            c.set_entry_points(
                "node",
                dla_core::coverage::EntryPoints {
                    discovered: vec![
                        s("src/heartbeat.js::startHeartbeat"),
                        s("src/metrics.js::neverRun"),
                    ],
                    ..Default::default()
                },
            );
        })
        .unwrap();
        let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
        if name == "node-socket-heartbeat" {
            let cov = dla_core::CoverageManifest::load(&fx.ctx.run_dir)
                .unwrap()
                .unwrap();
            assert_eq!(
                cov.workers["node"].entry_points.exercised,
                vec![s("src/heartbeat.js::startHeartbeat")],
                "{:?}",
                out.limitations
            );
        }
        results.push(load(&fx, &out, 0));
    }
    let leak = &results[0];
    let name = "node.timers@src/heartbeat.js:15";
    assert!(
        passes(leak, name),
        "timers at the setInterval site grow by n each point"
    );
    let i = leak.column_index(name).unwrap();
    assert_eq!(leak.columns[i].site.as_deref(), Some("src/heartbeat.js:15"));
    assert_eq!(leak.points[8].values[i], Some(4500.0));
    assert!(ratio(leak, "node.heap_used") > 2.0);
    let ctl = &results[1];
    assert!(ctl.column_index(name).is_none() || !passes(ctl, name));
    assert!(ratio(ctl, "node.heap_used") < 1.3);
}

#[test]
fn go_goroutines_are_counted_by_creation_site() {
    if !have(&["go"]) {
        return;
    }
    let mut results = Vec::new();
    for (name, cmd) in [
        ("go-subscription-workers", "./cmd/notifyd"),
        ("go-subscription-workers-closed", "./cmd/notifyd"),
    ] {
        let fx = fixture(name, &["go"], &[1]);
        let tmpl = fs::read_to_string(fx.agent.join("zz_dla_harness_test.go.tmpl")).unwrap();
        fs::write(
            fx.work.join("cmd/notifyd/zz_dla_harness_test.go"),
            tmpl.replace("@@DLA_PACKAGE@@", "main"),
        )
        .unwrap();
        let mut m = manifest(
            Runtime::Go,
            &[
                "go",
                "test",
                "-vet=off",
                "-count=1",
                "-run",
                "^TestDLAHarness$",
                "-coverprofile",
                "{cov}/{label}.out",
                "-coverpkg=./...",
                cmd,
            ],
            vec![
                (s("DLA_ARGS"), s(r#"["-n","{n}"]"#)),
                (s("GOFLAGS"), s("-mod=mod")),
                (s("GOPROXY"), s("off")),
            ],
            500,
        );
        m.agent = AgentKind::GoTestWrapper;
        write_manifest(&fx, &m);
        dla_core::coverage::CoverageManifest::update(&fx.ctx.run_dir, |c| {
            c.set_entry_points(
                "go",
                dla_core::coverage::EntryPoints {
                    discovered: vec![s("internal/notify/hub.go::Hub.Open")],
                    ..Default::default()
                },
            );
        })
        .unwrap();
        let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
        let cov = dla_core::CoverageManifest::load(&fx.ctx.run_dir)
            .unwrap()
            .unwrap();
        if name == "go-subscription-workers" {
            assert!(
                cov.workers["go"]
                    .entry_points
                    .exercised
                    .contains(&s("internal/notify/hub.go::Hub.Open")),
                "go coverage: {:?} / {:?}",
                cov.workers["go"].entry_points,
                out.limitations
            );
        }
        results.push(load(&fx, &out, 0));
    }
    let leak = &results[0];
    let name = "go.goroutines@internal/notify/subscription.go:33";
    assert!(passes(leak, name));
    let i = leak.column_index(name).unwrap();
    assert_eq!(leak.points[8].values[i], Some(4500.0));
    assert!(ratio(leak, "go.heap_live") > 2.0);
    let ctl = &results[1];
    assert!(ctl.column_index(name).is_none());
    assert!(ratio(ctl, "go.heap_live") < 1.5);
}

// The shim is preloaded with `LD_PRELOAD` and symbolized with GNU `addr2line`: Linux only.
#[cfg(target_os = "linux")]
#[test]
fn native_shim_sites_are_symbolized_to_the_leaking_line() {
    if !have(&["cc", "make", "addr2line"]) {
        return;
    }
    let fx = fixture("native-frame-decoder", &["native"], &[1]);
    let lib = fx.agent.join("libdlashim.so");
    let st = Command::new("cc")
        .args([
            "-std=gnu11",
            "-O2",
            "-g",
            "-fPIC",
            "-shared",
            "-fno-optimize-sibling-calls",
            "-o",
        ])
        .arg(&lib)
        .arg(fx.agent.join("dla_alloc_shim.c"))
        .args(["-ldl", "-lpthread"])
        .status()
        .unwrap();
    assert!(st.success());
    let mut m = manifest(
        Runtime::Native,
        &["./framedec", "--iterations", "{n}"],
        vec![
            (s("LD_PRELOAD"), lib.to_string_lossy().into_owned()),
            (s("DLA_SHIM_ONLY"), s("framedec")),
        ],
        1000,
    );
    m.build = Some(spec(&["make", "-s"]));
    m.agent = AgentKind::AllocShim;
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let series = load(&fx, &out, 0);
    let live = "native.live_bytes@src/decoder.c:29";
    let blocks = "native.live_blocks@src/decoder.c:29";
    assert!(passes(&series, live), "{:?}", series.limitations);
    assert!(passes(&series, blocks));
    let i = series.column_index(live).unwrap();
    assert_eq!(series.columns[i].site.as_deref(), Some("src/decoder.c:29"));
    // 62 rejected frames at n=1000, 125 at 2000: one block per 16th frame.
    let b = series.column_index(blocks).unwrap();
    assert_eq!(series.points[0].values[b], Some(62.0));
    assert_eq!(series.points[1].values[b], Some(125.0));
    assert!(series.column_index("native.peak_bytes").is_some());
    assert!(series.column_index("os.vm_hwm").is_some());
    // A purge is asked for at the last point and recorded as a series event.
    let purged = run_points(
        &fx.ctx,
        &m,
        &RunOptions {
            iterations: Some(2),
            purge: Some(s("trim")),
            ..RunOptions::default()
        },
    )
    .unwrap();
    let ps = load(&fx, &purged, 0);
    assert_eq!(ps.events.len(), 1, "{:?}", ps.limitations);
    assert_eq!(ps.events[0].kind, "purge:trim");
    assert_eq!(ps.events[0].after_point, 2);
    // A second run does not rebuild.
    let before = fs::metadata(fx.work.join("framedec"))
        .unwrap()
        .modified()
        .unwrap();
    run_points(
        &fx.ctx,
        &m,
        &RunOptions {
            iterations: Some(2),
            ..RunOptions::default()
        },
    )
    .unwrap();
    assert_eq!(
        fs::metadata(fx.work.join("framedec"))
            .unwrap()
            .modified()
            .unwrap(),
        before
    );
}

// The shim is preloaded with `LD_PRELOAD` and symbolized with GNU `addr2line`: Linux only.
#[cfg(target_os = "linux")]
#[test]
fn native_peak_columns_carry_a_pool_plateau_that_live_bytes_hide() {
    if !have(&["cc", "make", "addr2line"]) {
        return;
    }
    let fx = fixture("native-glyph-cache", &["native"], &[1]);
    let lib = fx.agent.join("libdlashim.so");
    assert!(
        Command::new("cc")
            .args([
                "-std=gnu11",
                "-O2",
                "-g",
                "-fPIC",
                "-shared",
                "-fno-optimize-sibling-calls",
                "-o"
            ])
            .arg(&lib)
            .arg(fx.agent.join("dla_alloc_shim.c"))
            .args(["-ldl", "-lpthread"])
            .status()
            .unwrap()
            .success()
    );
    let mut m = manifest(
        Runtime::Native,
        &["./textrun", "--iterations", "{n}"],
        vec![(s("LD_PRELOAD"), lib.to_string_lossy().into_owned())],
        2000,
    );
    m.build = Some(spec(&["make", "-s"]));
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let series = load(&fx, &out, 0);
    // The cache is gone at exit: live bytes are flat, the high-water mark is in the peak.
    assert!(ratio(&series, "native.live_bytes") < 1.1);
    assert!(!passes(&series, "native.live_bytes"));
    assert!(series.column_index("native.peak_bytes").is_some());
}

#[test]
fn jvm_agent_heap_and_probe_show_the_order_cache_leak() {
    if !have(&["java", "javac", "jar"]) {
        return;
    }
    let mut results = Vec::new();
    for name in ["jvm-order-cache", "jvm-order-cache-bounded"] {
        let fx = fixture(name, &["jvm"], &[1]);
        let jar = build_jvm_agent(&fx);
        let mut m = manifest(
            Runtime::Jvm,
            &["java", "-cp", "out", "shop.Main", "{n}"],
            vec![(
                s("JAVA_TOOL_OPTIONS"),
                format!(
                    "-javaagent:{} -XX:NativeMemoryTracking=summary",
                    jar.display()
                ),
            )],
            1000,
        );
        m.build = Some(spec(&[
            "sh",
            "-c",
            "rm -rf out && mkdir out && javac -d out src/shop/*.java",
        ]));
        m.agent = AgentKind::JvmAgent;
        m.probes = vec![ProbeSpec {
            id: s("P1"),
            finding: None,
            target: ProbeTarget::JvmStatic {
                class: s("shop.OrderCache"),
                field: s("ORDERS_BY_ID"),
            },
            site: Some(s("src/shop/OrderCache.java:8")),
        }];
        write_manifest(&fx, &m);
        dla_core::coverage::CoverageManifest::update(&fx.ctx.run_dir, |c| {
            c.set_entry_points(
                "jvm",
                dla_core::coverage::EntryPoints {
                    discovered: vec![
                        s("src/shop/OrderCache.java::OrderCache.store"),
                        s("src/shop/NotLoaded.java::NotLoaded.f"),
                    ],
                    ..Default::default()
                },
            );
        })
        .unwrap();
        let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
        let cov = dla_core::CoverageManifest::load(&fx.ctx.run_dir)
            .unwrap()
            .unwrap();
        assert_eq!(
            cov.workers["jvm"].entry_points.exercised,
            vec![s("src/shop/OrderCache.java::OrderCache.store")],
            "{:?}",
            out.limitations
        );
        results.push(load(&fx, &out, 0));
    }
    let probe = "probe:shop.OrderCache.ORDERS_BY_ID";
    let leak = &results[0];
    assert!(passes(leak, probe));
    let i = leak.column_index(probe).unwrap();
    assert_eq!(
        leak.columns[i].site.as_deref(),
        Some("src/shop/OrderCache.java:8")
    );
    // Absolute rises, not ratios: the base heap depends on the JDK and on how loaded the
    // machine is, the per-request growth does not.
    let rise = |s: &SeriesFile| {
        let h = post(s, "jvm.heap_used");
        h.last().unwrap().1 - h.first().unwrap().1
    };
    let leak_rise = rise(leak);
    assert!(leak_rise > 400_000.0, "leak heap rise {leak_rise}");
    assert!(leak.column_index("jvm.nmt_committed").is_some());
    let ctl = &results[1];
    assert!(!passes(ctl, probe));
    let ctl_rise = rise(ctl);
    assert!(
        ctl_rise < leak_rise / 3.0,
        "bounded heap rise {ctl_rise} against the leak's {leak_rise}"
    );
}

#[test]
fn a_hung_point_is_killed_at_the_timeout_and_the_run_is_refused_with_the_log() {
    if !have(&["sh"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    let mut m = manifest(
        Runtime::Native,
        &["sh", "-c", "sleep 60 & sleep 60"],
        vec![],
        10,
    );
    m.timeout_secs = 1;
    m.iterations = 2;
    write_manifest(&fx, &m);
    let started = std::time::Instant::now();
    let err = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap_err();
    assert!(started.elapsed().as_secs() < 20);
    assert_eq!(err.code, "harness-failed");
    assert!(err.missing.contains("logs/p1-native.log"), "{err:?}");
    assert!(err.message.contains("did not finish"), "{err:?}");
    let state = RunState::load(&fx.ctx.run_dir).unwrap();
    assert_eq!(
        state.processes.len(),
        1,
        "later points are not run after a timeout"
    );
    assert!(state.processes[0].stopped_at.is_some());
    assert!(!dla_sample::os::is_alive(state.processes[0].pid));
}

#[test]
fn a_failing_build_is_refused_and_marks_the_runtime_build_fails() {
    if !have(&["sh"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    dla_core::coverage::CoverageManifest::update(&fx.ctx.run_dir, |c| {
        c.set_entry_points(
            "native",
            dla_core::coverage::EntryPoints {
                discovered: vec![s("src/a.c::f")],
                ..Default::default()
            },
        );
    })
    .unwrap();
    let mut m = manifest(Runtime::Native, &["true"], vec![], 10);
    m.build = Some(spec(&["sh", "-c", "echo boom >&2; exit 3"]));
    write_manifest(&fx, &m);
    let err = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "harness-failed");
    assert!(err.missing.contains("build.log"));
    assert!(
        fs::read_to_string(fx.hdir.join("logs/build.log"))
            .unwrap()
            .contains("boom")
    );
    let cov = dla_core::CoverageManifest::load(&fx.ctx.run_dir)
        .unwrap()
        .unwrap();
    assert_eq!(
        cov.workers["native"].entry_points.not_exercisable[0].reason,
        "build_fails"
    );
}

#[test]
fn consent_tier_one_is_required() {
    let fx = fixture("", &[], &[]);
    let m = manifest(Runtime::Native, &["true"], vec![], 10);
    write_manifest(&fx, &m);
    let err = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "consent-required");
    assert!(err.fix.contains("dla consent"));
}

#[test]
fn points_without_an_agent_still_give_os_columns_and_a_limitation() {
    if !have(&["sh"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    let mut m = manifest(
        Runtime::Native,
        &["sh", "-c", "sleep 0.3; echo {n}"],
        vec![],
        10,
    );
    m.iterations = 2;
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let series = load(&fx, &out, 0);
    assert_eq!(series.points.len(), 3);
    assert!(series.column_index("os.rss").is_some());
    assert!(series.column_index("os.vm_hwm").is_some());
    assert!(
        series
            .limitations
            .iter()
            .any(|l| l.contains("no dump record"))
    );
    let log = fs::read_to_string(fx.hdir.join("logs/p2-native.log")).unwrap();
    assert_eq!(log.trim(), "20");
}

#[test]
fn python_driver_mode_gives_a_series_per_key_mode_with_a_purge_event() {
    if !have(&["python3"]) {
        return;
    }
    let fx = fixture("", &["python"], &[1]);
    fs::write(fx.work.join("app_cache.py"), "CACHE = {}\n").unwrap();
    let tmpl = fs::read_to_string(fx.agent.join("dla_harness.py")).unwrap();
    let driver = tmpl
        .replace("@@DLA_STEP@@", "200")
        .replace("@@DLA_WARMUP@@", "1")
        .replace("@@DLA_ROUNDS@@", "8")
        .replace("@@DLA_REPEAT@@", "50")
        .replace(
            "raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            "import app_cache",
        )
        .replace(
            "raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            "import app_cache\n    app_cache.CACHE[key] = bytearray(2000)",
        );
    fs::create_dir_all(fx.hdir.join("driver")).unwrap();
    fs::write(fx.hdir.join("driver/dla_harness.py"), driver).unwrap();
    let sep = if cfg!(windows) { ";" } else { ":" };
    let mut m = manifest(
        Runtime::Python,
        &[
            "python3",
            &fx.hdir.join("driver/dla_harness.py").to_string_lossy(),
        ],
        vec![(
            s("PYTHONPATH"),
            format!("{}{sep}{}", fx.agent.display(), fx.work.display()),
        )],
        200,
    );
    m.mode = HarnessMode::Driver;
    m.key_modes = vec![KeyMode::Fresh, KeyMode::Repeated];
    m.agent = AgentKind::DriverPython;
    write_manifest(&fx, &m);
    let opts = RunOptions {
        purge: Some(s("gc")),
        ..RunOptions::default()
    };
    let out = run_points(&fx.ctx, &m, &opts).unwrap();
    assert_eq!(out.series, ["S1", "S2"]);
    assert_eq!(out.processes, 2);
    let fresh = load(&fx, &out, 0);
    let repeated = load(&fx, &out, 1);
    assert_eq!(fresh.source.mode, Some(KeyMode::Fresh));
    assert_eq!(repeated.source.mode, Some(KeyMode::Repeated));
    assert_eq!(fresh.axis, dla_core::series::Axis::Iteration);
    assert_eq!(fresh.points.len(), 9);
    assert_eq!(fresh.warmup, 1);
    assert_eq!(fresh.points[8].x, 1800.0);
    assert!(
        ratio(&fresh, "py.traced") > 2.0,
        "fresh keys grow the cache"
    );
    assert!(
        ratio(&repeated, "py.traced") < 1.15,
        "repeated keys plateau"
    );
    assert_eq!(fresh.events.len(), 1);
    assert_eq!(fresh.events[0].after_point, 8);
    assert_eq!(fresh.events[0].kind, "purge:gc");
    assert!(
        fresh.events[0]
            .detail
            .as_deref()
            .unwrap()
            .contains("before")
    );
}

#[test]
fn a_wrapper_script_that_runs_javac_and_tests_still_yields_the_main_jvm_record() {
    if !have(&["java", "javac", "jar", "sh"]) {
        return;
    }
    let fx = fixture("jvm-order-cache", &["jvm"], &[1]);
    let jar = build_jvm_agent(&fx);
    // build.sh compiles, runs the unit tests (a second JVM with the agent) and then Main.
    let mut m = manifest(
        Runtime::Jvm,
        &["sh", "./build.sh", "run", "{n}"],
        vec![(
            s("JAVA_TOOL_OPTIONS"),
            format!(
                "-javaagent:{} -XX:NativeMemoryTracking=summary",
                jar.display()
            ),
        )],
        1000,
    );
    m.iterations = 3;
    m.agent = AgentKind::JvmAgent;
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let series = load(&fx, &out, 0);
    assert_eq!(series.points.len(), 4);
    // Main leaks about 0.45 KB per request, so 2000 more requests add close to 1 MB;
    // the test JVM's heap does not depend on n. An absolute rise is robust to the JDK's
    // base heap size, which a ratio is not.
    let heap = post(&series, "jvm.heap_used");
    let rise = heap.last().unwrap().1 - heap.first().unwrap().1;
    assert!(
        rise > 400_000.0,
        "heap of Main, not of the test JVM: rise {rise} bytes over {heap:?}"
    );
    let dumps = fs::read_to_string(fx.hdir.join("out/p4-native/dump.jsonl")).unwrap();
    assert!(
        dumps.lines().count() >= 2,
        "the test JVM wrote a record too"
    );
    assert!(
        series
            .limitations
            .iter()
            .all(|l| !l.contains("no process had")),
        "{:?}",
        series.limitations
    );
    // Own-process figures: the sh wrapper and javac are not summed into os.rss.
    assert!(ratio(&series, "os.rss") < 4.0);
}

// ---------------------------------------------------------------------------------------
// Review fixes

fn run_failure_series(fx: &Fx) -> usize {
    fs::read_dir(fx.ctx.run_dir.join("series")).map_or(0, |d| d.flatten().count())
}

#[test]
fn an_edited_driver_is_copied_again_and_a_changed_driver_rebuilds() {
    if !have(&["sh"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    let driver_dir = fx.hdir.join("driver");
    fs::create_dir_all(&driver_dir).unwrap();
    let src = driver_dir.join("d.sh");
    fs::write(&src, "#!/bin/sh\necho first >&2\nexit 3\n").unwrap();
    // The way `dla harness gen` writes the build of a driver-mode harness: a copy of the
    // driver into the scratch project.
    let mut m = manifest(Runtime::Native, &["sh", "./d.sh"], vec![], 10);
    m.mode = HarnessMode::Driver;
    m.key_modes = vec![KeyMode::Fresh];
    m.build = Some(spec(&[
        "sh",
        "-c",
        &format!(
            "cp -f '{}' '{}'",
            src.display(),
            fx.work.join("d.sh").display()
        ),
    ]));
    write_manifest(&fx, &m);
    let err = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "harness-failed", "{err:?}");
    assert!(
        fs::read_to_string(fx.work.join("d.sh"))
            .unwrap()
            .contains("first")
    );
    // The model fixes the driver; the second run must use it.
    fs::write(&src, "#!/bin/sh\necho second >&2\nexit 0\n").unwrap();
    let _ = run_points(&fx.ctx, &m, &RunOptions::default());
    assert!(
        fs::read_to_string(fx.work.join("d.sh"))
            .unwrap()
            .contains("second"),
        "the edited driver was not picked up"
    );
    let log = fs::read_to_string(fx.hdir.join("logs/p1-fresh.log")).unwrap();
    assert!(log.contains("second"), "{log}");
}

#[test]
fn a_run_where_every_process_crashed_is_refused_and_writes_no_series() {
    if !have(&["python3"]) {
        return;
    }
    // The Python agent's exit hook writes a record even when the program crashes.
    let fx = fixture("python-report-memo", &["python"], &[1]);
    let mut m = manifest(
        Runtime::Python,
        &["python3", "-c", "import no_such_module_dla"],
        vec![(s("PYTHONPATH"), fx.agent.to_string_lossy().into_owned())],
        250,
    );
    m.iterations = 3;
    write_manifest(&fx, &m);
    let err = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "harness-failed", "{err:?}");
    assert!(err.message.contains("non-zero"), "{err:?}");
    assert!(err.missing.contains("logs/p1-native.log"), "{err:?}");
    assert_eq!(run_failure_series(&fx), 0);
}

#[test]
fn a_mix_of_failed_and_successful_points_keeps_the_successful_ones() {
    if !have(&["sh"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    let mut m = manifest(
        Runtime::Native,
        &["sh", "-c", "sleep 0.2; test {n} -ne 30"],
        vec![],
        10,
    );
    m.iterations = 3;
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let series = load(&fx, &out, 0);
    let xs: Vec<f64> = series.points.iter().map(|p| p.x).collect();
    assert_eq!(xs, [10.0, 20.0, 40.0]);
    assert!(
        series
            .limitations
            .iter()
            .any(|l| l.contains("p3-native") && l.contains("dropped")),
        "{:?}",
        series.limitations
    );
}

#[test]
fn inline_pythonpath_in_the_command_survives_the_agent_injection() {
    if !have(&["python3"]) {
        return;
    }
    use dla_core::workload::{Workload, WorkloadKind};
    use dla_workload::harness::commands::{Layout, Switches, plan};
    let fx = fixture("", &["python"], &[1]);
    fs::create_dir_all(fx.work.join("src/pkg")).unwrap();
    fs::write(fx.work.join("src/pkg/__init__.py"), "").unwrap();
    fs::write(
        fx.work.join("src/helper.py"),
        "def size(n):\n    return n\n",
    )
    .unwrap();
    fs::write(
        fx.work.join("src/pkg/main.py"),
        "import sys\nimport helper\nprint(helper.size(int(sys.argv[1])))\n",
    )
    .unwrap();
    let w = Workload {
        id: "x".into(),
        kind: WorkloadKind::Main,
        runtime: Runtime::Python,
        command: "PYTHONPATH=src python3 -m pkg.main".into(),
        cwd: ".".into(),
        file: "src/pkg/main.py".into(),
        entry_points: Vec::new(),
        not_exercisable: None,
        rank: 0,
        iteration_arg: None,
        iteration_default: None,
    };
    let layout = Layout {
        work: fx.work.clone(),
        agent: fx.agent.clone(),
        driver: fx.hdir.join("driver"),
    };
    let p = plan(&w, Some("{n}"), &layout, &Switches::default(), None).unwrap();
    let mut m = manifest(Runtime::Python, &[], vec![], 250);
    m.run = p.run;
    m.iterations = 2;
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    assert!(
        out.exits.iter().all(|(_, c)| *c == Some(0)),
        "{:?}",
        out.exits
    );
    let log = fs::read_to_string(fx.hdir.join("logs/p1-native.log")).unwrap();
    assert_eq!(log.trim(), "250");
}

// The shim is preloaded with `LD_PRELOAD` and symbolized with GNU `addr2line`: Linux only.
#[cfg(target_os = "linux")]
#[test]
fn a_native_command_through_a_shell_script_still_gives_shim_dumps() {
    if !have(&["cc", "make", "addr2line", "sh"]) {
        return;
    }
    use dla_core::workload::{Workload, WorkloadKind};
    use dla_workload::harness::commands::{Layout, Switches, plan};
    let fx = fixture("native-frame-decoder", &["native"], &[1]);
    let lib = fx.agent.join("libdlashim.so");
    assert!(
        Command::new("cc")
            .args(["-std=gnu11", "-O2", "-g", "-fPIC", "-shared", "-o"])
            .arg(&lib)
            .arg(fx.agent.join("dla_alloc_shim.c"))
            .args(["-ldl", "-lpthread"])
            .status()
            .unwrap()
            .success()
    );
    fs::write(
        fx.work.join("run.sh"),
        "#!/bin/sh\nexec ./framedec --iterations \"$1\"\n",
    )
    .unwrap();
    let w = Workload {
        id: "x".into(),
        kind: WorkloadKind::Script,
        runtime: Runtime::Native,
        command: "make && sh ./run.sh".into(),
        cwd: ".".into(),
        file: "run.sh".into(),
        entry_points: Vec::new(),
        not_exercisable: None,
        rank: 0,
        iteration_arg: None,
        iteration_default: None,
    };
    let layout = Layout {
        work: fx.work.clone(),
        agent: fx.agent.clone(),
        driver: fx.hdir.join("driver"),
    };
    let p = plan(&w, Some("{n}"), &layout, &Switches::default(), None).unwrap();
    assert!(p.run.env.iter().all(|(k, _)| k != "DLA_SHIM_ONLY"));
    let mut m = manifest(Runtime::Native, &[], vec![], 1000);
    m.run = p.run;
    // The test points the preload at the library it just built.
    m.run.env.retain(|(k, _)| k != "LD_PRELOAD");
    m.run
        .env
        .push((s("LD_PRELOAD"), lib.to_string_lossy().into_owned()));
    m.build = p.build;
    m.agent = AgentKind::AllocShim;
    m.iterations = 2;
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let series = load(&fx, &out, 0);
    assert!(
        series.column_index("native.peak_bytes").is_some(),
        "no shim figures: {:?}",
        series.limitations
    );
}

/// Child half of [`sigterm_kills_the_workload_and_marks_the_record_stopped`]: runs a long
/// harness in this very process, which the parent test signals.
#[test]
fn sigterm_child_runs_a_long_harness() {
    let Ok(dir) = std::env::var("DLA_TEST_SIGTERM_RUN") else {
        return;
    };
    let run_dir = PathBuf::from(dir);
    let project = run_dir.parent().unwrap().parent().unwrap().to_path_buf();
    let ctx = ToolCtx {
        project,
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    };
    let m = ctx.harness("H1").unwrap();
    let _ = run_points(&ctx, &m, &RunOptions::default());
}

#[cfg(unix)]
#[test]
fn sigterm_kills_the_workload_and_marks_the_record_stopped() {
    if !have(&["sh"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    let mut m = manifest(Runtime::Native, &["sh", "-c", "sleep 300"], vec![], 10);
    m.timeout_secs = 600;
    write_manifest(&fx, &m);
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "sigterm_child_runs_a_long_harness",
            "--nocapture",
        ])
        .env("DLA_TEST_SIGTERM_RUN", &fx.ctx.run_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Wait for the workload to be registered.
    let started = std::time::Instant::now();
    let pid = loop {
        let st = RunState::load(&fx.ctx.run_dir).unwrap();
        if let Some(p) = st.processes.first() {
            break p.pid;
        }
        assert!(started.elapsed().as_secs() < 60, "workload never started");
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    // Let the handler thread be installed long before (it is, at the start of the run).
    assert!(
        Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let o = child.wait_with_output().unwrap();
    let status = o.status;
    eprintln!("child: {status:?} {}", String::from_utf8_lossy(&o.stderr));
    assert!(!status.success(), "dla must exit non-zero when signalled");
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert!(!dla_sample::os::is_alive(pid), "the workload survived");
    let st = RunState::load(&fx.ctx.run_dir).unwrap();
    assert!(
        st.processes.iter().all(|p| p.stopped_at.is_some()),
        "{:?}",
        st.processes
    );
}

// The shim is preloaded with `LD_PRELOAD`: Linux only.
#[cfg(target_os = "linux")]
#[test]
fn shim_reports_descriptors_by_kind_and_serves_small_alignments() {
    if !have(&["cc"]) {
        return;
    }
    let fx = fixture("", &["native"], &[1]);
    let lib = fx.agent.join("libdlashim.so");
    let st = Command::new("cc")
        .args([
            "-std=gnu11",
            "-O2",
            "-g",
            "-fPIC",
            "-shared",
            "-fno-optimize-sibling-calls",
            "-o",
        ])
        .arg(&lib)
        .arg(fx.agent.join("dla_alloc_shim.c"))
        .args(["-ldl", "-lpthread"])
        .status()
        .unwrap();
    assert!(st.success());
    // A program with one descriptor of each kind, and an aligned_alloc below the pointer size
    // (the C++ runtime does this; posix_memalign alone rejects it).
    let src = fx.work.join("fds.c");
    fs::write(
        &src,
        r#"#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <sys/eventfd.h>
#include <sys/epoll.h>
int main(void) {
    int p[2], s[2];
    if (pipe(p) || socketpair(AF_UNIX, SOCK_STREAM, 0, s)) return 2;
    eventfd(0, 0);
    epoll_create1(0);
    open("/etc/hostname", O_RDONLY);
    open("/dev/null", O_RDONLY);
    char t[] = "/tmp/dlafdXXXXXX";
    int u = mkstemp(t);
    if (u < 0) return 3;
    unlink(t);
    void *a = aligned_alloc(4, 64);
    if (a == NULL) return 4;
    free(a);
    return 0;
}
"#,
    )
    .unwrap();
    let exe = fx.work.join("fds");
    assert!(
        Command::new("cc")
            .arg("-o")
            .arg(&exe)
            .arg(&src)
            .status()
            .unwrap()
            .success()
    );
    let out = fx.work.join("dump.jsonl");
    let st = Command::new(&exe)
        .env("LD_PRELOAD", &lib)
        .env("DLA_HARNESS_OUT", &out)
        .status()
        .unwrap();
    assert_eq!(
        st.code(),
        Some(0),
        "aligned_alloc(4, 64) must succeed under the shim"
    );
    let line = fs::read_to_string(&out).unwrap();
    let rec: serde_json::Value = serde_json::from_str(line.lines().last().unwrap()).unwrap();
    let v = &rec["values"];
    for (kind, min) in [
        ("fd.socket", 2.0),
        ("fd.pipe", 2.0),
        ("fd.file", 1.0),
        ("fd.deleted", 1.0),
        ("fd.dev", 1.0),
        ("fd.eventfd", 1.0),
        ("fd.eventpoll", 1.0),
        ("fd.timerfd", 0.0),
    ] {
        let got = v[kind]
            .as_f64()
            .unwrap_or_else(|| panic!("no {kind} in {v}"));
        assert!(got >= min, "{kind} = {got}, want at least {min}: {v}");
    }
    // The kinds add up to os.fds.
    let sum: f64 = v
        .as_object()
        .unwrap()
        .iter()
        .filter(|(k, _)| k.starts_with("fd."))
        .map(|(_, x)| x.as_f64().unwrap())
        .sum();
    assert_eq!(sum, v["os.fds"].as_f64().unwrap(), "{v}");
}

#[test]
fn jvm_agent_writes_the_requested_captures_and_lists_them() {
    if !have(&["java", "javac", "jar"]) {
        return;
    }
    let fx = fixture("", &["jvm"], &[1]);
    let jar = build_jvm_agent(&fx);
    fs::create_dir_all(fx.work.join("app")).unwrap();
    fs::write(
        fx.work.join("app/Leak.java"),
        "import java.util.*;\npublic class Leak {\n    static final Map<String, byte[]> CACHE = new HashMap<>();\n    public static void main(String[] a) throws Exception {\n        int n = Integer.parseInt(a[0]);\n        for (int i = 0; i < n; i++) CACHE.put(\"k\" + i, new byte[1024]);\n        System.out.println(CACHE.size());\n    }\n}\n",
    )
    .unwrap();
    assert!(
        Command::new("javac")
            .args(["-d", "out", "app/Leak.java"])
            .current_dir(&fx.work)
            .env_remove("JAVA_TOOL_OPTIONS")
            .status()
            .unwrap()
            .success()
    );
    let mut m = manifest(
        Runtime::Jvm,
        &["java", "-cp", "out", "Leak", "{n}"],
        vec![(
            s("JAVA_TOOL_OPTIONS"),
            format!(
                "-javaagent:{} -XX:NativeMemoryTracking=summary",
                jar.display()
            ),
        )],
        3000,
    );
    m.agent = AgentKind::JvmAgent;
    write_manifest(&fx, &m);
    let opts = RunOptions {
        iterations: Some(3),
        capture: "heap,histogram,jfr,nonheap".parse().unwrap(),
        capture_dumps: Some(2),
        ..RunOptions::default()
    };
    let out = run_points(&fx.ctx, &m, &opts).unwrap();
    use dla_core::harness::CaptureKind::{Heap, Histogram, Jfr, NonHeap};
    let count = |k| out.captures.iter().filter(|c| c.kind == k).count();
    // Four points: histograms at each, the heavy kinds at two (the end of the warm-up and
    // the last).
    assert_eq!(count(Histogram), 4, "{:?}", out.limitations);
    assert_eq!(count(Heap), 2);
    assert_eq!(count(Jfr), 2);
    assert_eq!(count(NonHeap), 8);
    for c in &out.captures {
        let abs = fx.ctx.run_dir.join(&c.path);
        assert_eq!(fs::metadata(&abs).unwrap().len(), c.bytes, "{}", c.path);
        assert!(c.bytes > 0 && c.captured_at.ends_with('Z'));
    }
    let heap = out.captures.iter().find(|c| c.kind == Heap).unwrap();
    assert!(heap.ms.is_some(), "the agent reports what the dump cost");
    assert!(heap.bytes > 1_000_000);
    // The text histogram is what jcmd prints.
    let hist = out.captures.iter().find(|c| c.kind == Histogram).unwrap();
    let text = fs::read_to_string(fx.ctx.run_dir.join(&hist.path)).unwrap();
    assert!(
        text.contains("#instances") && text.contains("java.lang.String"),
        "{}",
        &text[..text.len().min(300)]
    );
    // Without the option nothing is written.
    let plain = run_points(
        &fx.ctx,
        &m,
        &RunOptions {
            iterations: Some(1),
            ..RunOptions::default()
        },
    )
    .unwrap();
    assert!(plain.captures.is_empty());
}
