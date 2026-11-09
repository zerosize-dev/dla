//! `dla probe`: what runtimes, tools, privileges, test runners and libraries the project has.
//!
//! # Contract
//!
//! The crate exposes ONE entry point, [`run`]. It does its own writes through the
//! [`ToolCtx`] (see `dla_core::toolctx` for the design and the rules every stage
//! follows) and returns a bounded [`StageOutcome`]. The CLI (`dla-cli`,
//! `probe_cmd.rs`) only parses arguments, builds the context and prints the outcome.
//!
//! What [`run`] writes:
//!
//! * `.dla/<run>/probe/probe.json`: the full `dla_core::probe::ProbeReport`;
//! * one evidence record of kind `probe` (bounded summary, `raw` = `probe/probe.json`);
//! * `coverage.json` via `CoverageManifest::update`: for every detected runtime
//!   (worker key = the runtime name) each registry backend as `missing` (with the
//!   install hint) or, when found, recorded as available (status `ran`, `version` set,
//!   `enumerated` 0) until a sweep updates it. An entry a later stage already filled
//!   (a sweep that enumerated, a blocked run) is never downgraded.
//!
//! Probe adds no candidates and never writes into the project, except what the
//! project's own build command writes when `--build` is given.
//!
//! # Modules
//!
//! * `scan`: the file walk, runtime detection, declared versions, test runners and
//!   libraries with their harness flags;
//! * `ffi`: foreign-function boundaries (a JNI library loaded by JVM sources and the native
//!   code that exports it), written to the report's `boundaries`;
//! * `registry`: the backend registry (binary names, install hints) and tool lookup on
//!   PATH and in the tools directory;
//! * `privilege`: uid, ptrace scope, container hints, macOS SIP;
//! * `build`: the per-runtime build command and its timeout handling;
//! * `exec`: child processes with a timeout.

mod build;
mod exec;
mod ffi;
mod privilege;
mod registry;
mod scan;

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};

use dla_core::{
    CoverageManifest, Refusal,
    coverage::{BackendCoverage, BackendStatus},
    error::Error,
    evidence::{CoverageStatus, EvidenceCoverage, EvidenceKind},
    fsutil::{to_pretty_json, write_atomic},
    probe::{BuildProbe, BuildStatus, ProbeReport, ToolVersion},
    runtime::Runtime,
    toolctx::{EvidenceSpec, StageOutcome, ToolCtx},
};
use serde_json::json;

pub use registry::{BACKENDS, Backend};

/// Schema version of `probe.json`.
const PROBE_SCHEMA_VERSION: u32 = 1;

/// Options of `dla probe`.
#[derive(Debug, Clone)]
pub struct ProbeOptions {
    /// `--build`: also run each detected runtime's build command. Requires consent tier
    /// 1 (`ctx.require_tier(1, ..)`, whose fix is `dla consent --tiers 1-4`); without
    /// it the call refuses and writes nothing.
    pub build: bool,
    /// Time allowed for each build command.
    pub build_timeout: Duration,
    /// Time allowed for each version query.
    pub tool_timeout: Duration,
    /// Restrict detection, tool queries and builds to this runtime. The CLI sets it in
    /// worker mode (a worker's name is its runtime's name).
    pub runtime: Option<Runtime>,
}

impl Default for ProbeOptions {
    fn default() -> Self {
        Self {
            build: false,
            build_timeout: Duration::from_secs(300),
            tool_timeout: Duration::from_secs(15),
            runtime: None,
        }
    }
}

/// Probes the project at `ctx.project`.
///
/// Detects runtimes from build files and sources, reads toolchain versions (each with a
/// timeout; failures are recorded, never fatal), finds every registry backend on PATH
/// or in `ctx.tools_dir()`, collects privilege facts, test runners and the library
/// harness flags, and optionally builds. Writes the files listed in the crate docs and
/// returns the evidence ID in [`StageOutcome::evidence`] and a summary
/// (runtimes, tools found/missing, build results) in [`StageOutcome::summary`].
///
/// Returns a [`Refusal`] when `opts.build` is set without consent tier 1.
pub fn run(ctx: &ToolCtx, opts: &ProbeOptions) -> Result<StageOutcome, Refusal> {
    let path_var = std::env::var_os("PATH");
    run_with(ctx, opts, ctx.tools_dir().as_deref(), path_var.as_deref())
}

/// [`run`] with the tools directory and PATH given explicitly.
pub fn run_with(
    ctx: &ToolCtx,
    opts: &ProbeOptions,
    tools_dir: Option<&Path>,
    path_var: Option<&std::ffi::OsStr>,
) -> Result<StageOutcome, Refusal> {
    if opts.build {
        ctx.require_time("dla probe --build")?;
        ctx.require_tier(1, "dla probe --build")?;
    }
    let stage = ctx.stage_dir("probe")?;
    let tmp = ctx.tmp_dir()?;
    let tree = scan::Tree::walk(&ctx.project);
    let mut detected = scan::detect(&tree);
    if let Some(rt) = opts.runtime {
        detected.retain(|d| d.probe.runtime == rt);
    }
    let runtimes: Vec<Runtime> = detected.iter().map(|d| d.probe.runtime).collect();

    for d in &mut detected {
        d.probe.toolchain = toolchain(d, opts.tool_timeout);
    }
    let tools = registry::probe_tools(&runtimes, tools_dir, path_var, opts.tool_timeout);
    let found = scan::find_runners_and_libraries(&tree, &runtimes);
    let privilege = privilege::collect(opts.tool_timeout);
    let builds: Vec<BuildProbe> = if opts.build {
        detected
            .iter()
            .map(|d| build::run_one(&tree, d, &tmp, opts.build_timeout))
            .collect()
    } else {
        Vec::new()
    };

    let report = ProbeReport {
        schema_version: PROBE_SCHEMA_VERSION,
        project: dla_core::paths::to_slash(&ctx.project),
        runtimes: detected.iter().map(|d| d.probe.clone()).collect(),
        tools,
        privilege,
        test_runners: found.test_runners,
        libraries: found.libraries,
        builds,
        boundaries: ffi::detect(&tree),
    };

    let report_name = report_file(ctx.worker.as_deref());
    let raw_path = stage.join(&report_name);
    write_atomic(&raw_path, to_pretty_json(&report).as_bytes()).map_err(|e| e.to_refusal())?;

    let summary = summarize(&report);
    let mut ledger = ctx.ledger()?;
    let mut spec = EvidenceSpec::new(
        EvidenceKind::Probe,
        "dla-probe",
        env!("CARGO_PKG_VERSION"),
        bounded(summary.evidence_result.clone(), &summary.counts),
    );
    spec.raw = Some(format!("probe/{report_name}"));
    spec.coverage = EvidenceCoverage {
        status: if tree.truncated {
            CoverageStatus::Partial
        } else {
            CoverageStatus::Complete
        },
        examined: Some(tree.files.len() as u64),
        total: None,
    };
    if tree.truncated {
        spec.limitations
            .push("the file walk stopped at its file limit".to_owned());
    }
    let evidence = ctx.add_evidence(&mut ledger, spec)?;

    record_coverage(ctx, &report)?;

    Ok(StageOutcome {
        evidence: vec![evidence.to_string()],
        candidates: 0,
        skipped_duplicates: 0,
        artifacts: vec![ctx.pointer(&format!("probe/{report_name}"))],
        summary: summary.cli,
    })
}

/// Pieces of the summaries derived from a report.
struct Summary {
    evidence_result: serde_json::Value,
    counts: serde_json::Value,
    cli: serde_json::Value,
}

fn summarize(r: &ProbeReport) -> Summary {
    let runtimes: Vec<&str> = r.runtimes.iter().map(|x| x.runtime.as_str()).collect();
    let present: Vec<String> = r
        .tools
        .iter()
        .filter(|t| t.available && t.name != "dla")
        .map(|t| format!("{}:{}", t.runtime.as_str(), t.name))
        .collect();
    let missing: Vec<String> = r
        .tools
        .iter()
        .filter(|t| !t.available)
        .map(|t| format!("{}:{}", t.runtime.as_str(), t.name))
        .collect();
    let builds: serde_json::Map<String, serde_json::Value> = r
        .builds
        .iter()
        .map(|b| (b.runtime.as_str().to_owned(), json!(b.status.as_str())))
        .collect();
    let runners: Vec<&str> = r.test_runners.iter().map(|t| t.name.as_str()).collect();
    let libs: Vec<&str> = r.libraries.iter().map(|l| l.library.as_str()).collect();
    let flags: usize = r.libraries.iter().map(|l| l.harness_flags.len()).sum();
    let evidence_result = json!({
        "runtimes": runtimes,
        "tools_found": present,
        "tools_missing": missing,
        "test_runners": runners,
        "libraries": libs,
        "builds": builds,
        "root": r.privilege.root,
        "ptrace_scope": r.privilege.ptrace_scope,
    });
    let counts = json!({
        "runtimes": runtimes,
        "tools_found": present.len(),
        "tools_missing": missing.len(),
        "test_runners": runners.len(),
        "libraries": libs.len(),
        "builds": builds,
    });
    let cli = json!({
        "runtimes": runtimes,
        "tools_found": present,
        "tools_missing": missing,
        "test_runners": runners,
        "libraries": libs,
        "harness_flags": flags,
        "builds": builds,
        "boundaries": r.boundaries.iter().map(|b| format!("{}:{}", b.kind, b.library.as_deref().unwrap_or("?"))).collect::<Vec<_>>(),
        "privilege": {
            "root": r.privilege.root,
            "ptrace_scope": r.privilege.ptrace_scope,
            "container": !r.privilege.container.is_empty(),
        },
    });
    Summary {
        evidence_result,
        counts,
        cli,
    }
}

/// `full` when it fits the evidence size limit, else the compact `counts` form.
fn bounded(full: serde_json::Value, counts: &serde_json::Value) -> serde_json::Value {
    let limit = dla_core::MAX_RESULT_BYTES - 200;
    if full.to_string().len() <= limit {
        full
    } else if counts.to_string().len() <= limit {
        counts.clone()
    } else {
        json!({"note": "summary too large; see probe.json"})
    }
}

fn record_coverage(ctx: &ToolCtx, r: &ProbeReport) -> Result<(), Refusal> {
    CoverageManifest::update(&ctx.run_dir, |m| {
        for t in &r.tools {
            let key = t.runtime.as_str();
            let keep = m
                .workers
                .get(key)
                .and_then(|w| w.backends.get(&t.name))
                .is_some_and(|b| match b.status {
                    BackendStatus::Blocked => true,
                    BackendStatus::Ran => b.enumerated > 0 || b.dispositioned > 0,
                    BackendStatus::Missing => false,
                });
            if keep {
                continue;
            }
            let cov = if t.available {
                BackendCoverage::ran(t.version.clone(), 0)
            } else {
                let note = BACKENDS
                    .iter()
                    .find(|b| b.name == t.name)
                    .map_or("", |b| b.hint_note);
                let mut reason = "not found on PATH or in the tools directory".to_owned();
                if !note.is_empty() {
                    reason.push_str("; ");
                    reason.push_str(note);
                }
                BackendCoverage::missing(reason, t.install_hint.clone().unwrap_or_default())
            };
            m.set_backend(key, &t.name, cov);
        }
        for rt in &r.runtimes {
            m.worker_mut(rt.runtime.as_str());
        }
    })
    .map_err(|e: Error| e.to_refusal())?;
    Ok(())
}

/// Toolchain versions for one detected runtime, queried in parallel.
fn toolchain(d: &scan::Detected, timeout: Duration) -> Vec<ToolVersion> {
    let has_file = |name: &str| {
        d.build_files
            .iter()
            .any(|f| f.rsplit('/').next() == Some(name))
    };
    let mut q: Vec<(&str, &[&str], &str)> = Vec::new();
    match d.probe.runtime {
        Runtime::Jvm => {
            q.push(("java", &["-version"], ""));
            q.push(("javac", &["-version"], ""));
            if d.has_kotlin {
                q.push(("kotlinc", &["-version"], ""));
            }
            if has_file("pom.xml") {
                q.push(("mvn", &["--version"], ""));
            }
            if d.build_files.iter().any(|f| f.contains("gradle")) {
                q.push(("gradle", &["--version"], "Gradle"));
            }
        }
        Runtime::Go => q.push(("go", &["version"], "")),
        Runtime::Rust => {
            q.push(("rustc", &["--version"], ""));
            q.push(("cargo", &["--version"], ""));
        }
        Runtime::Node => {
            q.push(("node", &["--version"], ""));
            if has_file("package.json") {
                q.push(("npm", &["--version"], ""));
            }
        }
        Runtime::Python => q.push((build::python_program(), &["--version"], "")),
        Runtime::Native => {
            q.push(("cc", &["--version"], ""));
            if d.build_files.iter().any(|f| f.ends_with("akefile")) {
                q.push(("make", &["--version"], ""));
            }
        }
    }
    let path_var: Option<OsString> = std::env::var_os("PATH");
    let results: Vec<ToolVersion> = std::thread::scope(|s| {
        let hs: Vec<_> = q
            .iter()
            .map(|(name, args, prefix)| {
                let pv = path_var.as_deref();
                s.spawn(move || tool_version(name, args, prefix, pv, timeout))
            })
            .collect();
        hs.into_iter()
            .zip(&q)
            .map(|(h, (name, _, _))| {
                h.join().unwrap_or(ToolVersion {
                    name: (*name).to_owned(),
                    version: None,
                    error: Some("version query panicked".into()),
                })
            })
            .collect()
    });
    results
}

fn tool_version(
    name: &str,
    args: &[&str],
    line_prefix: &str,
    path_var: Option<&std::ffi::OsStr>,
    timeout: Duration,
) -> ToolVersion {
    let fail = |e: &str| ToolVersion {
        name: name.to_owned(),
        version: None,
        error: Some(e.to_owned()),
    };
    let Some(found) = registry::locate(name, None, path_var) else {
        return fail("not found on PATH");
    };
    let o = exec::run(&found.path, args, None, &[], timeout);
    if let Some(e) = o.spawn_error {
        return fail(&format!("could not start: {e}"));
    }
    if o.timed_out {
        return fail("timed out");
    }
    let text = if line_prefix.is_empty() {
        o.text.clone()
    } else {
        o.text
            .lines()
            .filter(|l| l.starts_with(line_prefix))
            .collect::<Vec<_>>()
            .join("\n")
    };
    match registry::version_line(&text) {
        Some(v) if o.success || !text.is_empty() => ToolVersion {
            name: name.to_owned(),
            version: Some(v),
            error: None,
        },
        _ => fail(&format!("exited with {:?}", o.code)),
    }
}

/// File name of the probe report under `probe/`: `probe.json` for the core (it is run-level
/// and read by `dla ledger merge` and `dla run dispatch`), `<worker>.json` in worker mode,
/// so a worker's probe (restricted to its runtime) never replaces the core's.
pub fn report_file(worker: Option<&str>) -> String {
    worker.map_or_else(|| "probe.json".to_owned(), |w| format!("{w}.json"))
}

/// The path of `probe.json` under a run directory.
pub fn report_path(run_dir: &Path) -> PathBuf {
    run_dir.join("probe").join("probe.json")
}

/// The probe report a stage of `worker` reads: its own `probe/<worker>.json` when it
/// ran one, else the core's `probe/probe.json`.
pub fn report_path_for(run_dir: &Path, worker: Option<&str>) -> PathBuf {
    if let Some(w) = worker {
        let own = run_dir.join("probe").join(report_file(Some(w)));
        if own.is_file() {
            return own;
        }
    }
    report_path(run_dir)
}

/// Reads `probe.json` of a run, when present (used by later stages to see build
/// failures).
pub fn load_report(run_dir: &Path) -> Option<ProbeReport> {
    let text = std::fs::read_to_string(report_path(run_dir)).ok()?;
    serde_json::from_str(&text).ok()
}

/// True when the run's probe recorded a failed or timed-out build for `rt`.
pub fn build_failed(report: &ProbeReport, rt: Runtime) -> bool {
    report
        .builds
        .iter()
        .any(|b| b.runtime == rt && matches!(b.status, BuildStatus::Failed | BuildStatus::Timeout))
}

// Unix only: the tests stand in for tools with `#!/bin/sh` scripts made executable by
// mode bits, which Windows has no equivalent for.
#[cfg(all(test, unix))]
mod tests;
