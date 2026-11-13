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
    series::{KeyMode, SeriesFile, SeriesRegion},
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

/// Major version of the `java` on the path (8 for `1.8.0`); 0 when it cannot be read.
fn java_major() -> u32 {
    let Ok(o) = Command::new("java")
        .arg("-version")
        .env_remove("JAVA_TOOL_OPTIONS")
        .output()
    else {
        return 0;
    };
    let text = String::from_utf8_lossy(&o.stderr).into_owned();
    let Some(v) = text.split('"').nth(1) else {
        return 0;
    };
    let mut parts = v.split(['.', '-', '+']);
    let first: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    if first == 1 {
        parts.next().and_then(|p| p.parse().ok()).unwrap_or(0)
    } else {
        first
    }
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

/// The purge record's `before` is the reading the column's series holds at the last point
/// (after that point's forced collections), so per-step garbage collected by the purge is
/// not counted as released memory: a small retained growth plus large per-step garbage is
/// a `retained` purge.
/// With the pre-collection `before` the garbage looked like most of the growth (ratio 0.6).
#[test]
fn node_gc_purge_of_retained_growth_with_garbage_is_retained() {
    if !have(&["node"]) {
        return;
    }
    let fx = fixture("", &["node"], &[1]);
    fs::write(
        fx.work.join("grow.js"),
        "'use strict';\n\
         const n = Number.parseInt(process.argv[2], 10);\n\
         globalThis.KEEP = [];\n\
         let garbage = null;\n\
         for (let i = 0; i < n; i++) {\n\
           if (i % 10 === 0) KEEP.push({ i, s: 'k' + i });\n\
           garbage = new Array(2000).fill(i);\n\
         }\n",
    )
    .unwrap();
    let preload = fx.agent.join("dla-preload.js");
    let mut m = manifest(
        Runtime::Node,
        &["node", "grow.js", "{n}"],
        vec![(
            s("NODE_OPTIONS"),
            format!("--require {}", preload.display()),
        )],
        20000,
    );
    m.agent = AgentKind::NodePreload;
    write_manifest(&fx, &m);
    let opts = RunOptions {
        purge: Some(s("gc")),
        ..RunOptions::default()
    };
    let out = run_points(&fx.ctx, &m, &opts).unwrap();
    let series = load(&fx, &out, 0);
    let ev = series
        .events
        .iter()
        .find(|e| e.kind == "purge:gc")
        .unwrap_or_else(|| panic!("no purge event: {:?}", out.limitations));
    let detail: serde_json::Value = serde_json::from_str(ev.detail.as_deref().unwrap()).unwrap();
    let num = |side: &str, key: &str| detail[side][key].as_f64().unwrap();
    let i = series.column_index("node.heap_used").unwrap();
    let last = series.points.last().unwrap().values[i].unwrap();
    // `before` is the series' own last reading, not a pre-collection one.
    assert_eq!(num("before", "node.heap_used"), last, "{detail}");
    let warm = series.points[series.warmup as usize - 1].values[i].unwrap();
    let growth = num("before", "node.heap_used") - warm;
    let drop = num("before", "node.heap_used") - num("after", "node.heap_used");
    assert!(growth > 0.0, "{detail}");
    assert!(
        drop <= 0.2 * growth,
        "retained growth looks released: {detail}"
    );
    for key in [
        "node.heap_total",
        "node.external",
        "node.array_buffers",
        "os.rss",
    ] {
        assert!(
            detail["before"].get(key).is_some() && detail["after"].get(key).is_some(),
            "{key}: {detail}"
        );
    }
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

/// What the build step of [`run_wrapped`] leaves at `./framedec`.
#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
enum Built {
    /// The binary itself.
    Plain,
    /// A libtool-style wrapper script that execs the real binary under another name.
    Wrapper,
    /// A symbolic link to the real binary under another name.
    Link,
}

/// Builds the shim, makes a manifest for `argv` with `DLA_SHIM_ONLY=shim_only` whose build step
/// leaves `built` at `./framedec`, and runs it. None when the host lacks a tool.
#[cfg(target_os = "linux")]
fn run_wrapped(argv: &[&str], shim_only: &str, built: Built) -> Option<(Fx, RunOutput)> {
    if !have(&["cc", "make", "addr2line", "sh"]) {
        return None;
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
    let env = vec![
        (s("LD_PRELOAD"), lib.to_string_lossy().into_owned()),
        (s("DLA_SHIM_ONLY"), s(shim_only)),
    ];
    let mut m = manifest(Runtime::Native, argv, env, 1000);
    m.build = Some(spec(&match built {
        Built::Wrapper => vec![
            "sh",
            "-c",
            "make -s && mkdir -p .libs && mv framedec .libs/framedec-real && printf '#!/bin/sh\\nexec \"$(dirname \"$0\")/.libs/framedec-real\" \"$@\"\\n' > framedec && chmod +x framedec",
        ],
        Built::Link => vec![
            "sh",
            "-c",
            "make -s && mkdir -p .libs && mv framedec .libs/framedec-real && ln -s .libs/framedec-real framedec",
        ],
        Built::Plain => vec!["make", "-s"],
    }));
    m.agent = AgentKind::AllocShim;
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    Some((fx, out))
}

// The build step turns `./framedec` into a libtool-style wrapper script that execs the real
// binary under another name. `gen` narrowed the shim to `framedec`, a name no process
// carries once the build has run, so the run must drop `DLA_SHIM_ONLY` and still measure
// the site columns.
// The shim is preloaded with `LD_PRELOAD` and symbolized with GNU `addr2line`: Linux only.
#[cfg(target_os = "linux")]
#[test]
fn a_build_that_makes_the_program_a_wrapper_script_is_measured_without_the_narrowing() {
    let live = "native.live_bytes@src/decoder.c:29";
    let Some((fx, out)) = run_wrapped(
        &["./framedec", "--iterations", "{n}"],
        "framedec",
        Built::Wrapper,
    ) else {
        return;
    };
    let series = load(&fx, &out, 0);
    assert!(
        passes(&series, live),
        "no site column: {:?} {:?}",
        series.limitations,
        series.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    assert!(
        out.limitations
            .iter()
            .any(|l| l.contains("not a native executable")),
        "{:?}",
        out.limitations
    );
    // A plain binary keeps the narrowing and says nothing about it.
    let (fx2, out2) = run_wrapped(
        &["./framedec", "--iterations", "{n}"],
        "framedec",
        Built::Plain,
    )
    .unwrap();
    assert!(
        !out2
            .limitations
            .iter()
            .any(|l| l.contains("not a native executable")),
        "{:?}",
        out2.limitations
    );
    assert!(passes(&load(&fx2, &out2, 0), live));
}

// The build step makes `./framedec` a symbolic link to the real binary under another name.
// The shim compares the executable's resolved name (`/proc/self/exe`), so the run narrows to
// `framedec-real` instead of the link's name and the site columns are measured.
#[cfg(target_os = "linux")]
#[test]
fn a_build_that_makes_the_program_a_link_narrows_to_the_resolved_name() {
    let Some((fx, out)) = run_wrapped(
        &["./framedec", "--iterations", "{n}"],
        "framedec",
        Built::Link,
    ) else {
        return;
    };
    assert!(
        passes(&load(&fx, &out, 0), "native.live_bytes@src/decoder.c:29"),
        "{:?}",
        out.limitations
    );
    assert!(
        out.limitations
            .iter()
            .any(|l| l.contains("narrowed to framedec-real")),
        "{:?}",
        out.limitations
    );
}

// A run piece `gen` could not split into words becomes `sh -c "<piece>"`; `gen` still
// narrowed to the first word of the piece, so the check reads that word, not `sh`.
#[cfg(target_os = "linux")]
#[test]
fn a_wrapper_started_through_sh_c_is_measured_without_the_narrowing() {
    let Some((fx, out)) = run_wrapped(
        &["sh", "-c", "./framedec --iterations {n}"],
        "framedec",
        Built::Wrapper,
    ) else {
        return;
    };
    assert!(
        passes(&load(&fx, &out, 0), "native.live_bytes@src/decoder.c:29"),
        "{:?}",
        out.limitations
    );
    assert!(
        out.limitations
            .iter()
            .any(|l| l.contains("not a native executable")),
        "{:?}",
        out.limitations
    );
}

// `DLA_SHIM_ONLY=java` (the JNI harnesses) is not a name taken from the program, so a
// script as the program must not drop it, nor claim it was not narrowed to that name.
#[cfg(target_os = "linux")]
#[test]
fn a_narrowing_to_another_name_is_kept_when_the_program_is_a_script() {
    let Some((_fx, out)) = run_wrapped(
        &["./framedec", "--iterations", "{n}"],
        "java",
        Built::Wrapper,
    ) else {
        return;
    };
    assert!(
        !out.limitations
            .iter()
            .any(|l| l.contains("not a native executable")),
        "{:?}",
        out.limitations
    );
    // The narrowing stayed: `java` matches no process here, so nothing is measured.
    assert!(
        load(&_fx, &out, 0)
            .column_index("native.live_bytes@src/decoder.c:29")
            .is_none(),
        "the narrowing to `java` was dropped"
    );
}

// The shape `gen` writes for a piece with a redirect: an assignment whose value holds a
// '/' comes before the program and is skipped, as `gen` skips it.
#[cfg(target_os = "linux")]
#[test]
fn an_env_word_with_a_slash_in_a_sh_c_body_does_not_hide_the_wrapper() {
    let Some((fx, out)) = run_wrapped(
        &[
            "sh",
            "-c",
            "LD_LIBRARY_PATH=/usr/lib ./framedec --iterations {n} 2>/dev/null",
        ],
        "framedec",
        Built::Wrapper,
    ) else {
        return;
    };
    assert!(
        passes(&load(&fx, &out, 0), "native.live_bytes@src/decoder.c:29"),
        "{:?}",
        out.limitations
    );
    assert!(
        out.limitations
            .iter()
            .any(|l| l.contains("not a native executable")),
        "{:?}",
        out.limitations
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
    // The non-heap columns the alloc-test judges a warm-up by: the NMT categories (committed
    // bytes, from the summary the agent already reads), the unloads and the compile time.
    for c in [
        "jvm.nmt.metaspace",
        "jvm.nmt.code",
        "jvm.nmt.symbol",
        "jvm.classes_unloaded",
        "jvm.jit_seconds",
    ] {
        let i = leak
            .column_index(c)
            .unwrap_or_else(|| panic!("no column {c}"));
        assert_eq!(
            leak.columns[i].region,
            SeriesRegion::NonHeap,
            "{c} is a non-heap column"
        );
    }
    let code = post(leak, "jvm.nmt.code");
    assert!(
        code.iter().all(|p| p.1 > 100_000.0),
        "code cache bytes {code:?}"
    );
    assert!(
        leak.column_index("jvm.metaspace_used").is_none(),
        "the pool readings stand in only without NMT"
    );
    let ctl = &results[1];
    assert!(!passes(ctl, probe));
    let ctl_rise = rise(ctl);
    assert!(
        ctl_rise < leak_rise / 3.0,
        "bounded heap rise {ctl_rise} against the leak's {leak_rise}"
    );
}

/// Without native memory tracking the agent keeps the pool readings (Metaspace and the code
/// cache) in place of the NMT categories, and says why.
#[test]
fn jvm_agent_without_nmt_reads_the_pools_instead_of_the_categories() {
    if !have(&["java", "javac", "jar"]) {
        return;
    }
    let fx = fixture("jvm-order-cache", &["jvm"], &[1]);
    let jar = build_jvm_agent(&fx);
    let mut m = manifest(
        Runtime::Jvm,
        &["java", "-cp", "out", "shop.Main", "{n}"],
        vec![(
            s("JAVA_TOOL_OPTIONS"),
            format!("-javaagent:{}", jar.display()),
        )],
        1000,
    );
    m.build = Some(spec(&[
        "sh",
        "-c",
        "rm -rf out && mkdir out && javac -d out src/shop/*.java",
    ]));
    m.agent = AgentKind::JvmAgent;
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let series = load(&fx, &out, 0);
    for c in [
        "jvm.metaspace_used",
        "jvm.codecache_used",
        "jvm.nonheap_used",
    ] {
        let i = series
            .column_index(c)
            .unwrap_or_else(|| panic!("no column {c}"));
        assert_eq!(series.columns[i].region, SeriesRegion::NonHeap, "{c}");
    }
    assert!(
        series
            .columns
            .iter()
            .all(|c| !c.name.starts_with("jvm.nmt")),
        "{:?}",
        series.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    assert!(
        series
            .limitations
            .iter()
            .any(|l| l.contains("NMT off") && l.contains("jvm.nmt.<category>")),
        "{:?}",
        series.limitations
    );
}

/// A static `ThreadLocal` field is measured through the values it has on the live threads:
/// the pooled workers of `jvm-invoice-render` keep every segment they rendered, the trimmed
/// control cuts its lists back, and the ThreadLocal object itself is flat in both.
#[test]
fn jvm_agent_probe_measures_threadlocal_values_on_pooled_threads() {
    if !have(&["java", "javac", "jar"]) {
        return;
    }
    let mut results = Vec::new();
    for name in ["jvm-invoice-render", "jvm-invoice-render-trimmed"] {
        let fx = fixture(name, &["jvm"], &[1]);
        let jar = build_jvm_agent(&fx);
        let mut m = manifest(
            Runtime::Jvm,
            &["java", "-cp", "out", "billing.Main", "{n}"],
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
            "rm -rf out && mkdir out && javac -d out src/billing/*.java",
        ]));
        m.agent = AgentKind::JvmAgent;
        m.probes = vec![ProbeSpec {
            id: s("P1"),
            finding: None,
            target: ProbeTarget::JvmStatic {
                class: s("billing.RenderContext"),
                field: s("SEGMENTS"),
            },
            site: Some(s("src/billing/RenderContext.java:8")),
        }];
        write_manifest(&fx, &m);
        let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
        results.push(load(&fx, &out, 0));
    }
    let probe = "probe:billing.RenderContext.SEGMENTS";
    let leak = &results[0];
    assert!(passes(leak, probe), "the pooled workers' lists grow");
    assert!(passes(leak, &format!("{probe}.bytes")));
    // Every invoice adds about a dozen segments to the list of the worker that rendered it.
    let sizes = post(leak, &format!("{probe}.size"));
    let (first, last) = (sizes.first().unwrap(), sizes.last().unwrap());
    assert!(
        (last.1 - first.1) / (last.0 - first.0) > 5.0,
        "segments per invoice in {sizes:?}"
    );
    let ctl = &results[1];
    assert!(!passes(ctl, probe), "the trimmed lists stay short");
}

/// A program whose static `ClassLoader` defines `n` classes (`flat`: always 3). Each class is
/// a tiny class file written by hand, so the loader's own fields never change.
const LOADER_PROGRAM: &str = r#"import java.io.*;
public class Gen {
    static final class Defs extends ClassLoader {
        Defs() { super(Gen.class.getClassLoader()); }
        Class<?> define(String name, byte[] b) { return defineClass(name, b, 0, b.length); }
    }
    static final Defs LOADER = new Defs();
    static byte[] classFile(String name) throws IOException {
        ByteArrayOutputStream bo = new ByteArrayOutputStream();
        DataOutputStream o = new DataOutputStream(bo);
        o.writeInt(0xCAFEBABE); o.writeShort(0); o.writeShort(52);
        o.writeShort(5);
        o.writeByte(1); o.writeUTF(name);
        o.writeByte(7); o.writeShort(1);
        o.writeByte(1); o.writeUTF("java/lang/Object");
        o.writeByte(7); o.writeShort(3);
        o.writeShort(0x20); o.writeShort(2); o.writeShort(4);
        o.writeShort(0); o.writeShort(0); o.writeShort(0); o.writeShort(0);
        return bo.toByteArray();
    }
    public static void main(String[] a) throws Exception {
        int n = Integer.parseInt(a[0]);
        int defined = a[1].equals("flat") ? Math.min(n, 3) : n;
        for (int i = 0; i < defined; i++) {
            String name = "gen.G" + i;
            LOADER.define(name, classFile(name.replace('.', '/')));
        }
    }
}
"#;

/// The probe of a static `ClassLoader` counts the classes it defined: its own fields are
/// flat however many classes it holds, so a shallow walk would read as a plateau and clear
/// the loader. A loader that defines a fixed few classes stays flat.
#[test]
fn jvm_agent_probe_counts_the_classes_a_static_class_loader_defined() {
    if !have(&["java", "javac", "jar"]) {
        return;
    }
    let mut results = Vec::new();
    for mode in ["grow", "flat"] {
        let fx = fixture("", &["jvm"], &[1]);
        fs::write(fx.work.join("Gen.java"), LOADER_PROGRAM).unwrap();
        let jar = build_jvm_agent(&fx);
        let mut m = manifest(
            Runtime::Jvm,
            &["java", "-cp", "out", "Gen", "{n}", mode],
            vec![(
                s("JAVA_TOOL_OPTIONS"),
                format!("-javaagent:{}", jar.display()),
            )],
            200,
        );
        m.build = Some(spec(&[
            "sh",
            "-c",
            "rm -rf out && mkdir out && javac -d out Gen.java",
        ]));
        m.agent = AgentKind::JvmAgent;
        m.probes = vec![ProbeSpec {
            id: s("P1"),
            finding: None,
            target: ProbeTarget::JvmStatic {
                class: s("Gen"),
                field: s("LOADER"),
            },
            site: Some(s("Gen.java:8")),
        }];
        write_manifest(&fx, &m);
        let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
        results.push(load(&fx, &out, 0));
    }
    let probe = "probe:Gen.LOADER";
    let grow = &results[0];
    assert!(passes(grow, probe), "the classes the loader defined grow");
    assert!(passes(grow, &format!("{probe}.classes")));
    let counts = post(grow, &format!("{probe}.classes"));
    let (first, last) = (counts.first().unwrap(), counts.last().unwrap());
    assert!(
        ((last.1 - first.1) / (last.0 - first.0) - 1.0).abs() < 0.05,
        "one class per unit in {counts:?}"
    );
    assert!(
        !passes(grow, &format!("{probe}.bytes")),
        "a loader with no field of its own has a flat byte figure"
    );
    let flat = &results[1];
    assert!(!passes(flat, &format!("{probe}.classes")));
}

/// A project subclass of `ClassLoader` that keeps a list of byte arrays (`cache`), defining
/// no class (`flat`: the list stays at 3 entries).
const CACHE_LOADER_PROGRAM: &str = r#"import java.util.*;
public class Gen {
    static final class Defs extends ClassLoader {
        final List<byte[]> cache = new ArrayList<>();
        Defs() { super(Gen.class.getClassLoader()); }
    }
    static final Defs LOADER = new Defs();
    public static void main(String[] a) throws Exception {
        int n = Integer.parseInt(a[0]);
        int kept = a[1].equals("flat") ? Math.min(n, 3) : n;
        for (int i = 0; i < kept; i++) {
            LOADER.cache.add(new byte[2048]);
        }
    }
}
"#;

/// The probe of a static `ClassLoader` also walks the loader's own fields (those below
/// `java.lang.ClassLoader`): a cache that grows inside a project loader grows the byte
/// column, while the classes it defined stay at zero.
#[test]
fn jvm_agent_probe_walks_the_own_fields_of_a_static_class_loader() {
    if !have(&["java", "javac", "jar"]) {
        return;
    }
    let mut results = Vec::new();
    for mode in ["grow", "flat"] {
        let fx = fixture("", &["jvm"], &[1]);
        fs::write(fx.work.join("Gen.java"), CACHE_LOADER_PROGRAM).unwrap();
        let jar = build_jvm_agent(&fx);
        let mut m = manifest(
            Runtime::Jvm,
            &["java", "-cp", "out", "Gen", "{n}", mode],
            vec![(
                s("JAVA_TOOL_OPTIONS"),
                format!("-javaagent:{}", jar.display()),
            )],
            200,
        );
        m.build = Some(spec(&[
            "sh",
            "-c",
            "rm -rf out && mkdir out && javac -d out Gen.java",
        ]));
        m.agent = AgentKind::JvmAgent;
        m.probes = vec![ProbeSpec {
            id: s("P1"),
            finding: None,
            target: ProbeTarget::JvmStatic {
                class: s("Gen"),
                field: s("LOADER"),
            },
            site: Some(s("Gen.java:7")),
        }];
        write_manifest(&fx, &m);
        let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
        results.push(load(&fx, &out, 0));
    }
    let probe = "probe:Gen.LOADER";
    let grow = &results[0];
    assert!(passes(grow, &format!("{probe}.bytes")), "the cache grows");
    assert!(passes(grow, probe), "its objects grow");
    assert!(
        !passes(grow, &format!("{probe}.classes")),
        "no class is defined"
    );
    let bytes = post(grow, &format!("{probe}.bytes"));
    let (first, last) = (bytes.first().unwrap(), bytes.last().unwrap());
    assert!(
        (last.1 - first.1) / (last.0 - first.0) > 2000.0,
        "a 2 KiB array per unit in {bytes:?}"
    );
    let flat = &results[1];
    assert!(!passes(flat, &format!("{probe}.bytes")));
}

/// A `ThreadLocal` that no live thread holds a value of is not a measurement of zero: the
/// values may sit in virtual threads, a finished thread or nowhere yet, and a column of
/// zeros would read as a flat series. The probe abstains: no column, the reason in the
/// limitations, and no stale `--add-opens` hint.
#[test]
fn jvm_agent_probe_abstains_when_no_live_thread_holds_a_threadlocal_value() {
    if !have(&["java", "javac", "jar"]) {
        return;
    }
    let fx = fixture("", &["jvm"], &[1]);
    fs::write(
        fx.work.join("Idle.java"),
        "import java.util.*;\npublic class Idle {\n  static final ThreadLocal<List<String>> TL = ThreadLocal.withInitial(ArrayList::new);\n  public static void main(String[] a) throws Exception { Thread.sleep(200); }\n}\n",
    )
    .unwrap();
    let jar = build_jvm_agent(&fx);
    let mut m = manifest(
        Runtime::Jvm,
        &["java", "-cp", "out", "Idle", "{n}"],
        vec![(
            s("JAVA_TOOL_OPTIONS"),
            format!("-javaagent:{}", jar.display()),
        )],
        1000,
    );
    m.build = Some(spec(&[
        "sh",
        "-c",
        "rm -rf out && mkdir out && javac -d out Idle.java",
    ]));
    m.agent = AgentKind::JvmAgent;
    m.probes = vec![ProbeSpec {
        id: s("P1"),
        finding: None,
        target: ProbeTarget::JvmStatic {
            class: s("Idle"),
            field: s("TL"),
        },
        site: None,
    }];
    write_manifest(&fx, &m);
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let series = load(&fx, &out, 0);
    assert!(
        series
            .columns
            .iter()
            .all(|c| !c.name.starts_with("probe:Idle.TL")),
        "an empty measurement is no column: {:?}",
        series.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    let said: Vec<&String> = out
        .limitations
        .iter()
        .filter(|l| l.contains("Idle.TL"))
        .collect();
    assert!(
        said.iter()
            .any(|l| l.contains("no live platform thread holds a value")),
        "{:?}",
        out.limitations
    );
    assert!(
        said.iter()
            .all(|l| !l.contains("--add-opens needed") && !l.contains("(needs --add-opens"))
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
    // base heap size, which a ratio is not. The bound leaves room for the after-GC reading
    // (398 KB on a macOS runner) and stays far above the test JVM's flat heap.
    let heap = post(&series, "jvm.heap_used");
    let rise = heap.last().unwrap().1 - heap.first().unwrap().1;
    assert!(
        rise > 200_000.0,
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
    // threads.json needs Thread.dump_to_file (JDK 19 or later; verified on 21); older JDKs
    // write the other three.
    if java_major() >= 19 {
        assert_eq!(count(NonHeap), 8, "{:?}", out.limitations);
    } else {
        assert_eq!(count(NonHeap), 6, "{:?}", out.limitations);
        assert!(
            out.limitations
                .iter()
                .any(|l| l.contains("Thread.dump_to_file") && l.contains("JDK 21")),
            "{:?}",
            out.limitations
        );
    }
    for c in &out.captures {
        let abs = fx.ctx.run_dir.join(&c.path);
        assert_eq!(fs::metadata(&abs).unwrap().len(), c.bytes, "{}", c.path);
        assert!(c.bytes > 0 && c.captured_at.ends_with('Z'));
    }
    // The recording's chunks live inside the capture directory, not in the temp directory.
    let jfr = out.captures.iter().find(|c| c.kind == Jfr).unwrap();
    let jdir = fx.ctx.run_dir.join(&jfr.path);
    assert!(jdir.parent().unwrap().join("jfr-repo").is_dir());
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

// A selected `java` executable that runs a build tool (compiler, Gradle) stays untouched by the
// shim, so a detached Gradle daemon is not left with the shim loaded.
#[cfg(target_os = "linux")]
#[test]
fn shim_skips_tool_jvms_but_not_the_measured_one() {
    if !have(&["cc"]) {
        return;
    }
    let Some(truth) = ["/bin/true", "/usr/bin/true"]
        .iter()
        .map(Path::new)
        .find(|p| p.is_file())
    else {
        eprintln!("skipped: no true binary");
        return;
    };
    let fx = fixture("", &["native"], &[1]);
    let lib = fx.agent.join("libdlashim.so");
    let st = Command::new("cc")
        .args(["-std=gnu11", "-O2", "-g", "-fPIC", "-shared", "-o"])
        .arg(&lib)
        .arg(fx.agent.join("dla_alloc_shim.c"))
        .args(["-ldl", "-lpthread"])
        .status()
        .unwrap();
    assert!(st.success());
    let java = fx.work.join("java");
    fs::copy(truth, &java).unwrap();
    let wrote = |args: &[&str], tag: &str| {
        let out = fx.work.join(format!("dump-{tag}.jsonl"));
        let st = Command::new(&java)
            .args(args)
            .env("LD_PRELOAD", &lib)
            .env("DLA_SHIM_ONLY", "java")
            .env("DLA_HARNESS_OUT", &out)
            .status()
            .unwrap();
        assert!(st.success());
        out.is_file() && fs::metadata(&out).unwrap().len() > 0
    };
    assert!(wrote(&["-cp", "a:b", "app.Main"], "app"), "measured JVM");
    assert!(!wrote(
        &[
            "-cp",
            "a:b",
            "org.gradle.launcher.daemon.bootstrap.GradleDaemon",
            "8"
        ],
        "gd"
    ));
    assert!(!wrote(
        &["-Xmx1g", "-m", "jdk.compiler/com.sun.tools.javac.Main"],
        "jc"
    ));
    assert!(!wrote(&["-jar", "/x/sbt-launch.jar"], "sbt"));
    // A tool name only on the class path does not hide the program.
    assert!(wrote(&["-cp", "/x/org.gradle.jar:b", "app.Main"], "cp"));
}

// Notes the agent makes before the workload starts (a flight recording that could not start)
// must reach the exit record, which the harness reads, not only a tick record.
#[test]
fn premain_capture_notes_reach_the_exit_record() {
    if !have(&["java", "javac", "jar"]) {
        return;
    }
    let fx = fixture("", &["jvm"], &[1]);
    let jar = build_jvm_agent(&fx);
    fs::create_dir_all(fx.work.join("app")).unwrap();
    fs::write(
        fx.work.join("app/Nap.java"),
        "public class Nap { public static void main(String[] a) throws Exception { Thread.sleep(600); } }\n",
    )
    .unwrap();
    assert!(
        Command::new("javac")
            .args(["-d", "out", "app/Nap.java"])
            .current_dir(&fx.work)
            .env_remove("JAVA_TOOL_OPTIONS")
            .status()
            .unwrap()
            .success()
    );
    // The capture directory sits under a regular file, so the recording cannot start.
    let blocker = fx.work.join("blocker");
    fs::write(&blocker, "x").unwrap();
    let dump = fx.work.join("dump.jsonl");
    let st = Command::new("java")
        .args(["-cp", "out", "Nap"])
        .current_dir(&fx.work)
        .env("JAVA_TOOL_OPTIONS", format!("-javaagent:{}", jar.display()))
        .env("DLA_HARNESS_OUT", &dump)
        .env("DLA_TICK_MS", "100")
        .env("DLA_CAPTURE", "jfr")
        .env("DLA_CAPTURE_DIR", blocker.join("sub"))
        .status()
        .unwrap();
    assert!(st.success());
    let text = fs::read_to_string(&dump).unwrap();
    let recs: Vec<serde_json::Value> = text
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    assert!(recs.iter().any(|r| r["phase"] == "tick"), "{text}");
    let exit = recs.iter().find(|r| r["phase"] == "exit").expect("exit");
    let lims = exit["limitations"].to_string();
    assert!(
        lims.contains("flight recording could not be started"),
        "{lims}"
    );
}

/// A native program behind a wrapper script, built in the fixture's work directory.
#[cfg(target_os = "linux")]
fn wrapped_leak_fixture() -> Fx {
    let fx = fixture("", &["native"], &[1]);
    fs::write(
        fx.work.join("leak.c"),
        "#include <stdlib.h>\n#include <string.h>\nvoid *leaky_alloc(size_t n) { void *p = malloc(n); memset(p, 1, n); return p; }\nint main(int c, char **v) { int n = c > 1 ? atoi(v[1]) : 1; for (int i = 0; i < n; i++) leaky_alloc(1024); return 0; }\n",
    )
    .unwrap();
    assert!(
        Command::new("cc")
            .args(["-g", "-O0", "-o", "leak", "leak.c"])
            .current_dir(&fx.work)
            .status()
            .unwrap()
            .success()
    );
    fs::write(fx.work.join("run.sh"), "#!/bin/sh\n./leak \"$@\"\n").unwrap();
    fx
}

// valgrind follows a wrapper's children into one XML file per process; the report kept is the
// program's, not the shell's.
#[cfg(target_os = "linux")]
#[test]
fn valgrind_under_a_wrapper_reports_the_program_not_the_shell() {
    if !have(&["cc", "valgrind", "sh"]) {
        return;
    }
    let fx = wrapped_leak_fixture();
    for argv in [
        &["sh", "./run.sh", "{n}"][..],
        &["sh", "-c", "./leak 2>/dev/null {n}"][..],
        &["./leak", "{n}"][..],
    ] {
        let mut m = manifest(Runtime::Native, argv, vec![], 100);
        m.warmup = 1;
        m.iterations = 2;
        write_manifest(&fx, &m);
        let out = run_points(
            &fx.ctx,
            &m,
            &RunOptions {
                tool: Some(dla_workload::harness::capture::NativeTool::Valgrind),
                ..RunOptions::default()
            },
        )
        .unwrap();
        let f = out
            .captures
            .iter()
            .find(|c| c.path.ends_with("valgrind.xml"))
            .unwrap_or_else(|| panic!("{argv:?}: {:?}", out.limitations));
        let xml = fs::read_to_string(fx.ctx.run_dir.join(&f.path)).unwrap();
        assert!(xml.contains("leaky_alloc"), "{argv:?}: no program frames");
        assert!(xml.trim_end().ends_with("</valgrindoutput>"), "{argv:?}");
    }
}

// heaptrack cannot follow children: a wrapper run command is refused, not reported as `ran`.
#[cfg(target_os = "linux")]
#[test]
fn heaptrack_refuses_a_wrapper_run_command() {
    if !have(&["cc", "heaptrack", "sh"]) {
        return;
    }
    let fx = wrapped_leak_fixture();
    for argv in [&["sh", "./run.sh", "{n}"][..], &["./run.sh", "{n}"][..]] {
        let mut m = manifest(Runtime::Native, argv, vec![], 100);
        m.iterations = 2;
        write_manifest(&fx, &m);
        let err = run_points(
            &fx.ctx,
            &m,
            &RunOptions {
                tool: Some(dla_workload::harness::capture::NativeTool::Heaptrack),
                ..RunOptions::default()
            },
        )
        .unwrap_err();
        assert!(err.message.contains("wrapper"), "{argv:?}: {}", err.message);
    }
}

// ---------------------------------------------------------------------------------------
// The run's time budget

/// Sets the run deadline `secs` seconds from now.
fn deadline_in(fx: &Fx, secs: i64) {
    RunState::update(&fx.ctx.run_dir, |s| {
        s.budget.deadline = now_utc() + std::time::Duration::from_secs(secs as u64);
        Ok(())
    })
    .unwrap();
}

fn series_written(fx: &Fx) -> usize {
    fs::read_dir(fx.ctx.run_dir.join("series"))
        .map(|d| d.count())
        .unwrap_or(0)
}

#[test]
fn a_run_that_cannot_fit_the_deadline_is_refused_before_its_last_points() {
    if !have(&["sh", "awk"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    // The point of size n sleeps n/250 s: 0.4 s, 0.8 s, ... 3.2 s, 14.4 s in all, against a
    // 6 s budget. After the first two points (1.2 s) the rest needs about three times the
    // time left, however small a machine's start-up cost is.
    let mut m = manifest(
        Runtime::Native,
        &["sh", "-c", "sleep $(awk 'BEGIN{print {n}/250}')"],
        vec![],
        100,
    );
    m.iterations = 8;
    write_manifest(&fx, &m);
    deadline_in(&fx, 6);
    let started = std::time::Instant::now();
    let err = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap_err();
    assert_eq!(err.code, "time-budget", "{err:?}");
    assert!(
        err.message.contains("estimated at")
            && err.message.contains("remain before the run deadline"),
        "{err:?}"
    );
    // The fix names a smaller plan that fits.
    assert!(err.fix.contains("--iterations"), "{err:?}");
    let fewer: u32 = err
        .fix
        .split("--iterations ")
        .nth(1)
        .and_then(|t| t.split_whitespace().next())
        .and_then(|t| t.parse().ok())
        .expect("a number after --iterations");
    assert!((1..8).contains(&fewer), "{err:?}");
    assert!(err.fix.contains("--step"), "{err:?}");
    // Only the first points ran, and no series was written.
    assert!(
        started.elapsed().as_secs_f64() < 3.0,
        "{:?}",
        started.elapsed()
    );
    assert_eq!(RunState::load(&fx.ctx.run_dir).unwrap().processes.len(), 2);
    assert_eq!(series_written(&fx), 0);
}

#[test]
fn a_point_that_outlasts_the_deadline_is_cut_and_the_partial_series_says_so() {
    if !have(&["sh"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    // Fast points until n = 500, then a point that would take 30 s.
    let mut m = manifest(
        Runtime::Native,
        &[
            "sh",
            "-c",
            "if [ {n} -ge 500 ]; then sleep 30; else sleep 0.05; fi",
        ],
        vec![],
        100,
    );
    m.iterations = 8;
    m.timeout_secs = 120;
    write_manifest(&fx, &m);
    deadline_in(&fx, 4);
    let started = std::time::Instant::now();
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    assert!(
        started.elapsed().as_secs_f64() < 12.0,
        "{:?}",
        started.elapsed()
    );
    assert!(!out.partial.is_empty(), "{:?}", out.limitations);
    assert!(out.partial[0].contains("run deadline"), "{:?}", out.partial);
    let series = load(&fx, &out, 0);
    let xs: Vec<f64> = series.points.iter().map(|p| p.x).collect();
    assert_eq!(xs, [100.0, 200.0, 300.0, 400.0]);
    assert!(
        series
            .limitations
            .iter()
            .any(|l| l.starts_with("partial run:")),
        "{:?}",
        series.limitations
    );
    let state = RunState::load(&fx.ctx.run_dir).unwrap();
    assert!(
        state
            .processes
            .iter()
            .all(|p| !dla_sample::os::is_alive(p.pid))
    );
}

#[test]
fn a_known_pace_refuses_before_the_first_point() {
    if !have(&["sh", "awk"]) {
        return;
    }
    let fx = fixture("", &[], &[1]);
    let mut m = manifest(
        Runtime::Native,
        &["sh", "-c", "sleep $(awk 'BEGIN{print {n}/1000}')"],
        vec![],
        100,
    );
    m.iterations = 3;
    write_manifest(&fx, &m);
    // A first run with time to spare gives the pace (0.1 s per 100 units).
    run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    let before = RunState::load(&fx.ctx.run_dir).unwrap().processes.len();
    // The same plan with 40 more points and 3 s left: refused without starting one.
    deadline_in(&fx, 3);
    let opts = RunOptions {
        iterations: Some(40),
        ..RunOptions::default()
    };
    let err = run_points(&fx.ctx, &m, &opts).unwrap_err();
    assert_eq!(err.code, "time-budget", "{err:?}");
    assert_eq!(
        RunState::load(&fx.ctx.run_dir).unwrap().processes.len(),
        before,
        "no point was started"
    );
}
