//! The Node agent in ES-module packages (`"type": "module"` in package.json), and the names the
//! agent files have had.
//!
//! The agent is a `.cjs` file, so `--require` loads it as CommonJS in any package. Probes of
//! module-level bindings of an ES module are read through a `module.registerHooks()` load hook
//! that appends a closure hand-off to the probed module's source. Each test prints why it is
//! skipped when `node` is missing.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use dla_core::{
    Ledger,
    harness::{AgentKind, CommandSpec, HarnessManifest, HarnessMode, ProbeSpec, ProbeTarget},
    runtime::Runtime,
    series::{KeyMode, SeriesFile},
    state::{Consent, ConsentGiver, RunState, now_utc},
    toolctx::ToolCtx,
};
use dla_stats::{GateThresholds, GateVerdict, analyze};
use dla_workload::harness::run::{RunOptions, RunOutput, run_points};
use tempfile::TempDir;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn have_node() -> bool {
    let ok = Command::new("node")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipped: node is not available");
    }
    ok
}

struct Fx {
    _dir: TempDir,
    ctx: ToolCtx,
    hdir: PathBuf,
    work: PathBuf,
    agent: PathBuf,
}

/// A run directory with consent for tier 1 and harness `H1` holding an empty scratch project
/// and the Node agent file under `agent/`.
fn fixture() -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    let run_dir = project.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(project.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.consent = Some(Consent {
        tiers: vec![1],
        raw_values: false,
        given_by: ConsentGiver::Runner,
        recorded_at: now_utc(),
    });
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
    fs::create_dir_all(&work).unwrap();
    fs::copy(
        repo().join("harness/node/dla-preload.cjs"),
        agent.join("dla-preload.cjs"),
    )
    .unwrap();
    Fx {
        _dir: dir,
        ctx,
        hdir,
        work,
        agent,
    }
}

fn s(v: &str) -> String {
    v.to_owned()
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// The ES-module project: a module-level `Map` that grows per key, a static `Map` and a bounded
/// control in `store.js`; a class that registers a listener per call on a long-lived emitter in
/// `bus.js`; `load.js` is the entry file.
fn esm_project(fx: &Fx) {
    let work = &fx.work;
    // The original project's package.json: the harness directory (agent and driver) sits below it.
    write(&fx.ctx.project, "package.json", "{\"type\":\"module\"}\n");
    write(
        work,
        "package.json",
        "{\"name\":\"shop\",\"type\":\"module\"}\n",
    );
    write(
        work,
        "src/store.js",
        "export const ordersById = new Map();\n\
         export const recentLimit = new Map();\n\
         export class Store {\n  static audit = new Map();\n}\n\n\
         export function remember(id) {\n\
         \x20 ordersById.set(id, { id, lines: [id, id + 1] });\n\
         \x20 recentLimit.set(id % 8, id);\n\
         \x20 Store.audit.set(`a${id % 4}`, id);\n}\n\
         export function neverRun() {\n  return 1;\n}\n",
    );
    write(
        work,
        "src/bus.js",
        "import { EventEmitter } from 'node:events';\n\n\
         export const bus = new EventEmitter();\n\
         bus.setMaxListeners(0);\n\n\
         export class Watcher {\n  watch(id) {\n    bus.on('tick', () => id);\n  }\n}\n",
    );
    write(
        work,
        "load.js",
        "import { remember } from './src/store.js';\n\
         import { Watcher } from './src/bus.js';\n\n\
         const watcher = new Watcher();\n\
         const n = Number.parseInt(process.argv[2], 10);\n\
         for (let i = 0; i < n; i += 1) {\n  remember(i);\n  watcher.watch(i);\n}\n",
    );
}

/// The same project as CommonJS: `listeners` and `ordersById` grow, `recentLimit` is bounded.
fn cjs_project(work: &Path) {
    write(work, "package.json", "{\"name\":\"shop\"}\n");
    write(
        work,
        "load.js",
        "'use strict';\n\
         const { EventEmitter } = require('node:events');\n\
         const ordersById = new Map();\n\
         const recentLimit = new Map();\n\
         const bus = new EventEmitter();\n\
         bus.setMaxListeners(0);\n\
         const n = Number.parseInt(process.argv[2], 10);\n\
         for (let i = 0; i < n; i += 1) {\n\
         \x20 ordersById.set(i, { i });\n\
         \x20 recentLimit.set(i % 8, i);\n\
         \x20 bus.on('tick', () => i);\n}\n\
         module.exports = { bus };\n",
    );
}

fn manifest(node_options: String) -> HarnessManifest {
    HarnessManifest {
        sibling_of: Default::default(),
        purpose: None,
        targets: Vec::new(),
        schema_version: 1,
        id: "H1".into(),
        runtime: Runtime::Node,
        mode: HarnessMode::Command,
        workload: Some("test".into()),
        scratch: "harness/H1/work".into(),
        build: None,
        run: CommandSpec {
            argv: vec![s("node"), s("load.js"), s("{n}")],
            cwd: ".".into(),
            env: vec![
                (s("NODE_OPTIONS"), node_options),
                (s("NODE_V8_COVERAGE"), s("{cov}")),
            ],
        },
        iter_arg: Some("{n}".into()),
        warmup: 1,
        iterations: 8,
        step: 500,
        key_modes: vec![KeyMode::Native],
        probes: Vec::new(),
        agent: AgentKind::NodePreload,
        detectors: vec![],
        tools: vec![],
        gaps: vec![],
        timeout_secs: 120,
        ffi: None,
        captures: vec![],
    }
}

fn probe(id: &str, target: ProbeTarget) -> ProbeSpec {
    ProbeSpec {
        id: id.into(),
        finding: None,
        target,
        site: None,
    }
}

/// Runs the manifest; every `node-module` probe carries `names`, as `dla harness gen` writes it.
fn run_it(fx: &Fx, m: &HarnessManifest, names: &[&str]) -> (RunOutput, SeriesFile) {
    m.save(&fx.ctx.run_dir).unwrap();
    let mut probes = serde_json::to_value(&m.probes).unwrap();
    for p in probes.as_array_mut().unwrap() {
        if p["target"]["kind"] == "node-module" {
            p["target"]["names"] = serde_json::json!(names);
        }
    }
    fs::write(
        fx.hdir.join("probes.json"),
        serde_json::json!({ "probes": probes }).to_string(),
    )
    .unwrap();
    let out = run_points(&fx.ctx, m, &RunOptions::default()).unwrap();
    let series = SeriesFile::load(&fx.ctx.run_dir, &out.series[0]).unwrap();
    (out, series)
}

fn has(series: &SeriesFile, name: &str) -> bool {
    series.column_index(name).is_some()
}

fn passes(series: &SeriesFile, name: &str) -> bool {
    let i = series.column_index(name).unwrap_or_else(|| {
        panic!(
            "no column {name}; have {:?}",
            series.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
        )
    });
    let counter = series.columns[i].kind == dla_core::series::ColumnKind::Counter;
    let pts = series.post_warmup(i);
    let tail = (series.warmup > 0)
        .then(|| series.points.get(series.warmup as usize - 1))
        .flatten()
        .and_then(|p| Some((p.x, p.values.get(i).copied().flatten()?)));
    analyze(&pts, tail, counter, &GateThresholds::default()).gate == GateVerdict::Pass
}

fn require(fx: &Fx, file: &str) -> String {
    format!("--require {}", fx.agent.join(file).display())
}

fn store_probes() -> Vec<ProbeSpec> {
    vec![probe(
        "P1",
        ProbeTarget::NodeModule {
            file: s("src/store.js"),
        },
    )]
}

/// Command mode on an ES-module entry file: before, the agent (a `.js` file in the package) died
/// with `require is not defined in ES module scope`; the probes of the module-level `Map`s read
/// the live bindings, the listener column names the registration site in the ES module, and the
/// coverage of executed functions names the ES-module files.
#[test]
fn es_module_project_in_command_mode_is_measured_by_probes_listeners_and_coverage() {
    if !have_node() {
        return;
    }
    let fx = fixture();
    esm_project(&fx);
    let mut m = manifest(require(&fx, "dla-preload.cjs"));
    m.probes = store_probes();
    dla_core::coverage::CoverageManifest::update(&fx.ctx.run_dir, |c| {
        c.set_entry_points(
            "node",
            dla_core::coverage::EntryPoints {
                discovered: vec![s("src/store.js::remember"), s("src/store.js::neverRun")],
                ..Default::default()
            },
        );
    })
    .unwrap();
    let (out, series) = run_it(&fx, &m, &["ordersById", "recentLimit", "Store"]);
    // The grown Map: verdict grew; the bounded control and the static Map: constant.
    assert!(passes(&series, "probe:src/store.js:ordersById.size"));
    assert!(!passes(&series, "probe:src/store.js:recentLimit.size"));
    assert!(has(&series, "probe:src/store.js:Store.audit.size"));
    assert!(!passes(&series, "probe:src/store.js:Store.audit.size"));
    assert!(passes(&series, "probe:src/store.js:ordersById"));
    // The listener column names the registration site (line 8 of bus.js).
    let site = "node.listeners@src/bus.js:8";
    assert!(passes(&series, site), "{:?}", out.limitations);
    let i = series.column_index(site).unwrap();
    assert_eq!(series.columns[i].site.as_deref(), Some("src/bus.js:8"));
    assert_eq!(series.points[8].values[i], Some(4500.0));
    assert!(passes(&series, "node.listeners"));
    // Coverage reads the ES-module files by their file URLs.
    let cov = dla_core::CoverageManifest::load(&fx.ctx.run_dir)
        .unwrap()
        .unwrap();
    assert_eq!(
        cov.workers["node"].entry_points.exercised,
        vec![s("src/store.js::remember")],
        "{:?}",
        out.limitations
    );
}

/// The probe of an ES-module binding on a Node without `module.registerHooks`: no column, the
/// record names the version needed, a limitation line says so; the listener column stays.
#[test]
fn es_module_probe_without_register_hooks_says_which_node_is_needed() {
    if !have_node() {
        return;
    }
    let fx = fixture();
    esm_project(&fx);
    write(
        &fx.agent,
        "no-hooks.cjs",
        "require('module').registerHooks = undefined;\n",
    );
    let options = format!(
        "{} {}",
        require(&fx, "no-hooks.cjs"),
        require(&fx, "dla-preload.cjs")
    );
    let mut m = manifest(options);
    m.probes = store_probes();
    let (out, series) = run_it(&fx, &m, &["ordersById", "recentLimit", "Store"]);
    assert!(!has(&series, "probe:src/store.js:ordersById.size"));
    assert!(
        out.limitations
            .iter()
            .any(|l| l.contains("22.15") && l.contains("23.5") && l.contains("registerHooks")),
        "{:?}",
        out.limitations
    );
    assert!(passes(&series, "node.listeners@src/bus.js:8"));
}

/// A CommonJS project (no `"type"`) is measured as before: the module-wrapper accessor reads the
/// bindings, and no ES-module hook is involved.
#[test]
fn commonjs_project_is_measured_unchanged() {
    if !have_node() {
        return;
    }
    let fx = fixture();
    cjs_project(&fx.work);
    let mut m = manifest(require(&fx, "dla-preload.cjs"));
    m.probes = vec![probe("P1", ProbeTarget::NodeModule { file: s("load.js") })];
    let (out, series) = run_it(&fx, &m, &["ordersById", "recentLimit"]);
    assert!(
        has(&series, "probe:load.js:ordersById.size"),
        "{:?}",
        out.limitations
    );
    assert!(passes(&series, "probe:load.js:ordersById.size"));
    assert!(!passes(&series, "probe:load.js:recentLimit.size"));
    assert!(passes(&series, "node.listeners@load.js:11"));
}

/// A harness directory written by an older version names its agent `dla-preload.js`; `dla harness
/// run` uses the names the manifest records, so it keeps running (in a CommonJS project).
#[test]
fn a_harness_with_the_old_js_agent_name_still_runs() {
    if !have_node() {
        return;
    }
    let fx = fixture();
    cjs_project(&fx.work);
    fs::rename(
        fx.agent.join("dla-preload.cjs"),
        fx.agent.join("dla-preload.js"),
    )
    .unwrap();
    let mut m = manifest(require(&fx, "dla-preload.js"));
    m.probes = vec![probe("P1", ProbeTarget::NodeModule { file: s("load.js") })];
    let (_out, series) = run_it(&fx, &m, &["ordersById", "recentLimit"]);
    assert!(passes(&series, "probe:load.js:ordersById.size"));
    assert!(passes(&series, "node.listeners@load.js:11"));
}
