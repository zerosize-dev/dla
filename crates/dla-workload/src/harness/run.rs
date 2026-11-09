//! `dla harness run` and the point runner `dla alloc-test` reuses (spec 4.3).
//!
//! # Flow
//!
//! 1. [`run_points`] needs consent tier 1, builds the harness once (the build marker
//!    `.built` in the harness directory stops a second build), then measures:
//!    * command mode: one process per point `k = 1..=warmup+iterations+extra` with
//!      `n = step * k`, key mode `native`;
//!    * driver mode: one process per key mode whose checkpoint records are the points.
//! 2. Every measured process is registered in `state.json`, sampled with
//!    [`dla_sample::os::Sampler`] (process tree, 100 ms) and killed with its process group
//!    at `timeout_secs`.
//! 3. [`super::dumps`] chooses the measured process's record, [`super::symbolize`] maps
//!    allocator sites to source lines, [`super::build`] writes one series per key mode and
//!    [`super::cover`] records iterations and exercised entry points.
//! 4. [`run`] adds the `workload` evidence (authority test) naming the harness.
//!
//! # Environment of a measured process
//!
//! The manifest's `run.env` is merged over the inherited environment (`JAVA_TOOL_OPTIONS`
//! and `NODE_OPTIONS` appended with a space; `PYTHONPATH`, `LD_PRELOAD` and
//! `DYLD_INSERT_LIBRARIES` prepended with the path separator), then `DLA_HARNESS_OUT`,
//! `DLA_HARNESS_PROBES`, `DLA_PROJECT`, `DLA_ITER`, `DLA_TICK_MS`, `DLA_PURGE` (last point
//! only, command mode) and, in driver mode, `DLA_KEY_MODE`, `DLA_STEP`, `DLA_WARMUP`,
//! `DLA_ROUNDS` (the measured rounds: `iterations + extra`), `DLA_REPEAT` are set. Tokens
//! `{n}`, `{label}`, `{mode}`, `{out}` and `{cov}` in argv and env values are replaced.

use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use dla_core::{
    Refusal,
    error::Error,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind},
    harness::{CaptureFile, CommandSpec, DumpCoverage, HarnessManifest, HarnessMode},
    series::{Axis, KeyMode, SeriesSourceKind},
    state::{ProcessKind, ProcessRecord, now_utc},
    toolctx::{EvidenceSpec, StageOutcome, ToolCtx},
};
use dla_sample::os::{self, OsSample, Sampler, SamplerConfig, SamplerOutput};
use regex::Regex;
use serde_json::{Value, json};

mod tool;
mod vgdb;

use super::{
    AGENT, COV, DRIVER, LOGS, OUT, PROBES,
    build::{PointInput, SeriesParams, build_series},
    capture::{self, AgentNotes, CaptureSet, NativeTool},
    cover::{self, CoverageOutcome},
    dumps::{self, DumpFile, Rec},
    harness_dir,
    symbolize::{Sites, symbolize},
};

/// Default per-process timeout when the manifest has none.
const DEFAULT_TIMEOUT_SECS: u64 = 300;
/// Timeout of the build step.
const BUILD_TIMEOUT_SECS: u64 = 600;
/// OS sampling interval.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(100);
/// Default size of the repeated key space of a driver.
const DEFAULT_REPEAT: u64 = 64;
/// File in the harness directory recording the build that was done.
const BUILD_MARKER: &str = ".built";
/// Detector hits kept.
const MAX_DETECTORS: usize = 50;

/// Options of one harness execution.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// Key modes to run; empty means the manifest's `key_modes`.
    pub modes: Vec<KeyMode>,
    /// Post-warm-up points, overriding the manifest.
    pub iterations: Option<u32>,
    /// Extra points after the post-warm-up points (alloc-test plateau window).
    pub extra: u32,
    /// Purge applied at the last point (`DLA_PURGE`).
    pub purge: Option<String>,
    /// Source kind written into the series (`harness` or `alloc-test`).
    pub source_kind: SeriesSourceKind,
    /// Command mode only: the first point `k` to run (1 = all). `dla alloc-test` runs
    /// only the extra points after a complete harness series and joins the two.
    pub first_point: u32,
    /// Per-process timeout in seconds, overriding the manifest.
    pub timeout_secs: Option<u64>,
    /// Raw captures of the JVM agent (`--capture`); empty means none.
    pub capture: CaptureSet,
    /// Heap dumps a driver takes and command-mode points that take heavy captures
    /// (default [`capture::DEFAULT_DUMPS`]).
    pub capture_dumps: Option<u32>,
    /// Run the points under this tool instead of measuring them (`--tool`): no series is
    /// written, the tool's output is.
    pub tool: Option<NativeTool>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            modes: Vec::new(),
            iterations: None,
            extra: 0,
            purge: None,
            source_kind: SeriesSourceKind::Harness,
            first_point: 1,
            timeout_secs: None,
            capture: CaptureSet::default(),
            capture_dumps: None,
            tool: None,
        }
    }
}

/// A native allocation site tied to the managed method that calls into it.
#[derive(Debug, Clone, PartialEq)]
pub struct FfiSite {
    /// `native/src/lib.rs:40`.
    pub site: String,
    /// The exported symbol the site is in (`Java_shop_codec_Native_open`).
    pub export: String,
    /// The managed method (`shop.codec.Native.open`).
    pub managed: String,
    /// Where that method is declared (`kotlin/shop/Native.kt:7`).
    pub declared_at: Option<String>,
    /// Live bytes at the site in the last point.
    pub live_bytes: f64,
}

/// The sites of the last point that sit in a JNI export of the manifest's boundary, largest
/// first.
fn ffi_sites(manifest: &HarnessManifest, points: &[PointInput]) -> Vec<FfiSite> {
    let (Some(ffi), Some(sites)) = (
        manifest.ffi.as_ref(),
        points.iter().rev().find_map(|p| p.sites.as_ref()),
    ) else {
        return Vec::new();
    };
    let mut out: Vec<FfiSite> = sites
        .functions
        .iter()
        .filter_map(|(site, func)| {
            let m = ffi.methods.iter().find(|m| m.export == *func)?;
            Some(FfiSite {
                site: site.clone(),
                export: m.export.clone(),
                managed: m.managed.clone(),
                declared_at: m.declared_at.clone(),
                live_bytes: sites.by_site.get(site).map_or(0.0, |t| t.bytes),
            })
        })
        .collect();
    out.sort_by(|a, b| b.live_bytes.total_cmp(&a.live_bytes));
    out
}

/// What one execution produced.
#[derive(Debug, Clone, Default)]
pub struct RunOutput {
    /// Series IDs written, one per key mode, in mode order.
    pub series: Vec<String>,
    /// Points per series (warm-up included).
    pub points: u32,
    /// Measured processes started.
    pub processes: u32,
    /// Library detector hits (`{lib, text, site}`).
    pub detectors: Vec<Value>,
    /// Missing named tools and other gaps.
    pub gaps: Vec<String>,
    /// What this execution could not see.
    pub limitations: Vec<String>,
    /// Run-relative log paths.
    pub logs: Vec<String>,
    /// Key modes run, in order.
    pub modes: Vec<KeyMode>,
    /// Smallest and largest `x` of the points kept.
    pub x_range: Option<(f64, f64)>,
    /// Exit codes of the measured processes by label (`None`: killed or no code).
    pub exits: Vec<(String, Option<i32>)>,
    /// Post-warm-up units measured (post-warm-up points times step).
    pub units: u64,
    /// Files `--capture` or `--tool` left in the harness `out/` directory.
    pub captures: Vec<CaptureFile>,
    /// The tool the points ran under, when `--tool` was given.
    pub tool: Option<NativeTool>,
    /// Native allocation sites in the JNI exports of the project (JVM harness with a
    /// native library).
    pub ffi_sites: Vec<FfiSite>,
}

fn failed(why: impl Into<String>, missing: impl Into<String>, fix: impl Into<String>) -> Refusal {
    Refusal::new("harness-failed", why, missing, fix)
}

fn io_refusal(path: &Path, e: std::io::Error) -> Refusal {
    Error::io(path, e).to_refusal()
}

// ---------------------------------------------------------------------------------------
// Environment and tokens

/// Replaces the tokens of a command in `s`.
fn subst(s: &str, tokens: &[(&str, String)]) -> String {
    let mut out = s.to_owned();
    for (k, v) in tokens {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// How a manifest env key combines with the inherited value.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Merge {
    Replace,
    Append(char),
    Prepend(char),
}

fn merge_rule(key: &str) -> Merge {
    let path_sep = if cfg!(windows) { ';' } else { ':' };
    match key {
        "JAVA_TOOL_OPTIONS" | "NODE_OPTIONS" | "GRADLE_OPTS" => Merge::Append(' '),
        "PYTHONPATH" => Merge::Prepend(path_sep),
        "LD_PRELOAD" | "DYLD_INSERT_LIBRARIES" | "LD_LIBRARY_PATH" | "DYLD_LIBRARY_PATH" => {
            Merge::Prepend(':')
        }
        _ => Merge::Replace,
    }
}

/// The environment variables to set for `spec_env`, merged over the inherited values
/// (and over earlier entries of the same key).
fn merged_env(spec_env: &[(String, String)], tokens: &[(&str, String)]) -> Vec<(String, String)> {
    let mut cur: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in spec_env {
        let v = subst(v, tokens);
        let old = cur
            .get(k)
            .cloned()
            .or_else(|| std::env::var(k).ok())
            .filter(|o| !o.is_empty());
        let new = match (merge_rule(k), old) {
            (Merge::Append(sep), Some(o)) => format!("{o}{sep}{v}"),
            (Merge::Prepend(sep), Some(o)) => format!("{v}{sep}{o}"),
            _ => v,
        };
        cur.insert(k.clone(), new);
    }
    cur.into_iter().collect()
}

// ---------------------------------------------------------------------------------------
// Running one process

struct ExecSpec<'a> {
    argv: Vec<String>,
    cwd: PathBuf,
    env: Vec<(String, String)>,
    log: PathBuf,
    /// Standard error goes here instead of `log` (a command whose standard output is a
    /// report that must stay clean).
    stderr_log: Option<PathBuf>,
    timeout: Duration,
    sample: bool,
    command: &'a str,
    /// Ask the stopped process questions through `vgdb` (valgrind runs only).
    vgdb: Option<vgdb::VgdbPlan>,
}

#[derive(Default)]
struct ExecResult {
    pid: Option<u32>,
    code: Option<i32>,
    timed_out: bool,
    wall_ms: u64,
    sampler: Option<SamplerOutput>,
    spawn_error: Option<String>,
    /// What the vgdb chain wrote, when one ran.
    vgdb: Option<vgdb::VgdbOutcome>,
}

/// Kills the process group of `pid` (`dla_core::proc`: `kill -s KILL -- -pgid`, or
/// `taskkill /T /F` on Windows).
fn kill_group(pid: u32) {
    let _ = dla_core::proc::kill_group(pid, "KILL");
}

/// Spawns, registers, samples and waits for one process; kills its group at the timeout.
fn exec(ctx: &ToolCtx, spec: &ExecSpec<'_>) -> Result<ExecResult, Refusal> {
    let mut res = ExecResult::default();
    let Some((program, args)) = spec.argv.split_first() else {
        res.spawn_error = Some("the command has an empty argv".to_owned());
        return Ok(res);
    };
    if let Some(dir) = spec.log.parent() {
        fs::create_dir_all(dir).map_err(|e| io_refusal(dir, e))?;
    }
    let log = File::create(&spec.log).map_err(|e| io_refusal(&spec.log, e))?;
    let log2 = match &spec.stderr_log {
        Some(p) => File::create(p).map_err(|e| io_refusal(p, e))?,
        None => log.try_clone().map_err(|e| io_refusal(&spec.log, e))?,
    };
    let program_path = if program.contains(['/', '\\']) && Path::new(program).is_relative() {
        spec.cwd.join(program)
    } else {
        PathBuf::from(program)
    };
    let mut cmd = Command::new(&program_path);
    cmd.args(args)
        .current_dir(&spec.cwd)
        .envs(spec.env.iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log2));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let started = Instant::now();
    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            res.spawn_error = Some(format!("cannot start {program}: {e}"));
            return Ok(res);
        }
    };
    let pid = child.id();
    res.pid = Some(pid);
    let start_time = os::process_start_time(pid);
    let mut guard = ChildGuard {
        ctx,
        child,
        pid,
        start_time: start_time.clone(),
        registered: false,
        done: false,
    };
    live_add(pid, start_time.clone());
    ctx.register_process(ProcessRecord {
        pid,
        start_time: start_time.clone(),
        kind: ProcessKind::Harness,
        command: spec.command.to_owned(),
        started_at: now_utc(),
        stopped_at: None,
        role: dla_core::state::ProcessRole::Started,
        worker: ctx.worker.clone(),
        cleanup: Vec::new(),
    })?;
    guard.registered = true;
    let sampler = spec
        .sample
        .then(|| {
            Sampler::start(
                pid,
                SamplerConfig {
                    interval: SAMPLE_INTERVAL,
                    tree: true,
                },
            )
            .ok()
        })
        .flatten();
    let stop = Arc::new(AtomicBool::new(false));
    let chain = spec.vgdb.clone().map(|plan| {
        let stop = Arc::clone(&stop);
        let timeout = spec.timeout;
        thread::spawn(move || {
            vgdb::drive(&plan, pid, &stop, move || {
                timeout.saturating_sub(started.elapsed())
            })
        })
    });
    loop {
        match guard.child.try_wait() {
            Ok(Some(status)) => {
                res.code = status.code();
                break;
            }
            Ok(None) => {
                if started.elapsed() >= spec.timeout {
                    kill_group(pid);
                    let _ = guard.child.kill();
                    let _ = guard.child.wait();
                    res.timed_out = true;
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                res.spawn_error = Some(format!("waiting for {program} failed: {e}"));
                kill_group(pid);
                let _ = guard.child.kill();
                let _ = guard.child.wait();
                break;
            }
        }
    }
    res.wall_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    stop.store(true, Ordering::Relaxed);
    res.vgdb = chain.and_then(|t| t.join().ok());
    // Anything the process left running in its group goes with it.
    kill_group(pid);
    res.sampler = sampler.map(Sampler::stop);
    guard.finish()?;
    Ok(res)
}

/// Owns a spawned measured process: unless [`ChildGuard::finish`] ran, dropping it (an
/// early `?` return, a panic) kills the process group and marks the record stopped.
struct ChildGuard<'a> {
    ctx: &'a ToolCtx,
    child: Child,
    pid: u32,
    start_time: Option<String>,
    registered: bool,
    done: bool,
}

impl ChildGuard<'_> {
    /// Marks the record stopped; the process has ended.
    fn finish(&mut self) -> Result<(), Refusal> {
        self.done = true;
        live_remove(self.pid);
        if self.registered {
            self.ctx
                .mark_process_stopped(self.pid, self.start_time.as_deref())?;
        }
        Ok(())
    }
}

impl Drop for ChildGuard<'_> {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        kill_group(self.pid);
        let _ = self.child.kill();
        let _ = self.child.wait();
        live_remove(self.pid);
        if self.registered {
            let _ = self
                .ctx
                .mark_process_stopped(self.pid, self.start_time.as_deref());
        }
    }
}

// ---------------------------------------------------------------------------------------
// Signals

/// Processes of this run that are alive: `(pid, start_time)`.
type Live = Arc<Mutex<Vec<(u32, Option<String>)>>>;

thread_local! {
    /// The live list of the run executing on this thread.
    static LIVE: RefCell<Option<Live>> = const { RefCell::new(None) };
}

fn live_add(pid: u32, start_time: Option<String>) {
    LIVE.with(|l| {
        if let Some(live) = l.borrow().as_ref()
            && let Ok(mut v) = live.lock()
        {
            v.push((pid, start_time));
        }
    });
}

fn live_remove(pid: u32) {
    LIVE.with(|l| {
        if let Some(live) = l.borrow().as_ref()
            && let Ok(mut v) = live.lock()
        {
            v.retain(|(p, _)| *p != pid);
        }
    });
}

/// For the duration of a run: SIGINT and SIGTERM kill the process groups of the run's
/// measured processes, mark them stopped in `state.json` and end `dla` with a non-zero
/// status (128 + the signal number).
struct SignalGuard {
    #[cfg(unix)]
    handle: Option<signal_hook::iterator::Handle>,
    #[cfg(unix)]
    thread: Option<thread::JoinHandle<()>>,
}

impl SignalGuard {
    fn install(ctx: &ToolCtx) -> Self {
        let live: Live = Arc::new(Mutex::new(Vec::new()));
        LIVE.with(|l| *l.borrow_mut() = Some(Arc::clone(&live)));
        #[cfg(unix)]
        {
            use signal_hook::{
                consts::{SIGINT, SIGTERM},
                iterator::Signals,
            };
            let Ok(mut signals) = Signals::new([SIGINT, SIGTERM]) else {
                return Self {
                    handle: None,
                    thread: None,
                };
            };
            let handle = signals.handle();
            let ctx = ctx.clone();
            let thread = thread::spawn(move || {
                if let Some(sig) = signals.forever().next() {
                    let procs = live.lock().map(|v| v.clone()).unwrap_or_default();
                    for (pid, start) in &procs {
                        kill_group(*pid);
                        let _ = ctx.mark_process_stopped(*pid, start.as_deref());
                    }
                    std::process::exit(128 + sig);
                }
            });
            Self {
                handle: Some(handle),
                thread: Some(thread),
            }
        }
        #[cfg(not(unix))]
        {
            let _ = (ctx, live);
            Self {}
        }
    }
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        LIVE.with(|l| *l.borrow_mut() = None);
        #[cfg(unix)]
        {
            if let Some(h) = self.handle.take() {
                h.close();
            }
            if let Some(t) = self.thread.take() {
                let _ = t.join();
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Paths of a harness run

struct Paths {
    hdir: PathBuf,
    work: PathBuf,
    out: PathBuf,
    logs: PathBuf,
    cov: PathBuf,
    probes: Option<PathBuf>,
}

impl Paths {
    fn new(ctx: &ToolCtx, manifest: &HarnessManifest) -> Result<Self, Refusal> {
        let hdir = harness_dir(&ctx.run_dir, &manifest.id);
        for sub in [OUT, LOGS, COV] {
            let d = hdir.join(sub);
            fs::create_dir_all(&d).map_err(|e| io_refusal(&d, e))?;
        }
        let hdir = hdir.canonicalize().map_err(|e| io_refusal(&hdir, e))?;
        let work = ctx.run_dir.join(&manifest.scratch);
        let work = work.canonicalize().map_err(|_| {
            failed(
                format!(
                    "the scratch project of {} does not exist at {}",
                    manifest.id,
                    work.display()
                ),
                format!("{} (scratch copy of the project)", manifest.scratch),
                "dla harness gen to write the harness again",
            )
        })?;
        let probes = hdir.join(PROBES);
        Ok(Self {
            out: hdir.join(OUT),
            logs: hdir.join(LOGS),
            cov: hdir.join(COV),
            probes: probes.is_file().then_some(probes),
            hdir,
            work,
        })
    }

    fn rel(&self, ctx: &ToolCtx, p: &Path) -> String {
        let run = ctx
            .run_dir
            .canonicalize()
            .unwrap_or_else(|_| ctx.run_dir.clone());
        p.strip_prefix(&run)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

fn cwd_of(paths: &Paths, spec: &CommandSpec) -> PathBuf {
    paths.work.join(&spec.cwd)
}

// ---------------------------------------------------------------------------------------
// Build

/// FNV-1a content hash of the files under `dir` (names and bytes, sorted); 0 when absent.
fn hash_dir(dir: &Path) -> u64 {
    fn feed(h: &mut u64, bytes: &[u8]) {
        for b in bytes {
            *h ^= u64::from(*b);
            *h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if e.file_name() != "__pycache__" {
                    stack.push(e.path());
                }
            } else if ft.is_file() {
                files.push(e.path());
            }
        }
    }
    files.sort();
    for f in files {
        let rel = f
            .strip_prefix(dir)
            .unwrap_or(&f)
            .to_string_lossy()
            .into_owned();
        feed(&mut h, rel.as_bytes());
        feed(&mut h, &[0]);
        feed(&mut h, &fs::read(&f).unwrap_or_default());
        feed(&mut h, &[0xff]);
    }
    h
}

/// Runs the build step once. A failed build is the refusal `harness-failed`.
fn build_once(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    paths: &Paths,
    out: &mut RunOutput,
) -> Result<(), Refusal> {
    let Some(build) = &manifest.build else {
        return Ok(());
    };
    let marker = paths.hdir.join(BUILD_MARKER);
    // The build step may only copy the driver into the scratch copy, so the fingerprint
    // covers the content of the driver and agent files too: an edited driver rebuilds.
    let fingerprint = format!(
        "{}\ndriver={:016x} agent={:016x}",
        serde_json::to_string(build).unwrap_or_default(),
        hash_dir(&paths.hdir.join(DRIVER)),
        hash_dir(&paths.hdir.join(AGENT)),
    );
    if fs::read_to_string(&marker).is_ok_and(|m| m == fingerprint) {
        return Ok(());
    }
    let tokens = [
        ("cov", paths.cov.to_string_lossy().into_owned()),
        ("out", paths.out.to_string_lossy().into_owned()),
    ];
    let log = paths.logs.join("build.log");
    out.logs.push(paths.rel(ctx, &log));
    let command = subst(&build.argv.join(" "), &tokens);
    let res = exec(
        ctx,
        &ExecSpec {
            argv: build.argv.iter().map(|a| subst(a, &tokens)).collect(),
            cwd: cwd_of(paths, build),
            env: merged_env(&build.env, &tokens),
            log: log.clone(),
            stderr_log: None,
            timeout: Duration::from_secs(BUILD_TIMEOUT_SECS),
            sample: false,
            command: &command,
            vgdb: None,
        },
    )?;
    let log_rel = paths.rel(ctx, &log);
    let why = if let Some(e) = &res.spawn_error {
        Some(e.clone())
    } else if res.timed_out {
        Some(format!(
            "the build did not finish in {BUILD_TIMEOUT_SECS} s and was killed"
        ))
    } else if res.code != Some(0) {
        Some(format!(
            "the build exited with {}",
            res.code
                .map_or_else(|| "a signal".to_owned(), |c| format!("code {c}"))
        ))
    } else {
        None
    };
    if let Some(why) = why {
        cover::record(
            ctx,
            manifest.runtime,
            &CoverageOutcome::Failed {
                reason: crate::BUILD_FAILS,
            },
        )?;
        return Err(failed(
            format!("harness {} did not build: {why}", manifest.id),
            format!("a successful build; see {}", ctx.pointer(&log_rel)),
            format!(
                "fix the build in {}, then dla harness run {}",
                ctx.pointer(&manifest.scratch),
                manifest.id
            ),
        ));
    }
    fs::write(&marker, fingerprint).map_err(|e| io_refusal(&marker, e))?;
    Ok(())
}

// ---------------------------------------------------------------------------------------
// Points

/// Environment every measured process gets.
struct PointEnv<'a> {
    manifest: &'a HarnessManifest,
    paths: &'a Paths,
    label: String,
    mode: KeyMode,
    n: u64,
    purge: Option<&'a str>,
    /// Driver rounds after the warm-up.
    rounds: u32,
    /// `--capture` kinds, with whether this point takes the heavy ones.
    capture: Option<(CaptureSet, bool)>,
    /// Heap dumps a driver takes.
    dumps: u32,
}

impl PointEnv<'_> {
    fn dump_path(&self) -> PathBuf {
        self.paths.out.join(&self.label).join("dump.jsonl")
    }

    fn capture_dir(&self) -> PathBuf {
        self.paths.out.join(&self.label).join(capture::DIR)
    }

    fn exec_spec<'a>(
        &self,
        timeout: Duration,
        command: &'a str,
        env_extra: &[(&str, String)],
    ) -> ExecSpec<'a> {
        let m = self.manifest;
        let tokens = [
            ("n", self.n.to_string()),
            ("label", self.label.clone()),
            ("mode", self.mode.as_str().to_owned()),
            ("out", self.dump_path().to_string_lossy().into_owned()),
            ("cov", self.paths.cov.to_string_lossy().into_owned()),
        ];
        let mut env = merged_env(&m.run.env, &tokens);
        let mut set = |k: &str, v: String| {
            env.retain(|(ek, _)| ek != k);
            env.push((k.to_owned(), v));
        };
        set(
            "DLA_HARNESS_OUT",
            self.dump_path().to_string_lossy().into_owned(),
        );
        if let Some(p) = &self.paths.probes {
            set("DLA_HARNESS_PROBES", p.to_string_lossy().into_owned());
        }
        set(
            "DLA_PROJECT",
            self.paths.work.to_string_lossy().into_owned(),
        );
        set("DLA_ITER", self.n.to_string());
        set("DLA_TICK_MS", "100".to_owned());
        if let Some(p) = self.purge {
            set("DLA_PURGE", p.to_owned());
        }
        if let Some((kinds, full)) = self.capture {
            let v = kinds.env_value(full);
            if !v.is_empty() {
                set(dla_core::harness::ENV_CAPTURE, v);
                set(
                    dla_core::harness::ENV_CAPTURE_DIR,
                    self.capture_dir().to_string_lossy().into_owned(),
                );
                set(dla_core::harness::ENV_CAPTURE_DUMPS, self.dumps.to_string());
            }
        }
        if m.mode == HarnessMode::Driver {
            set("DLA_KEY_MODE", self.mode.as_str().to_owned());
            set("DLA_STEP", m.step.to_string());
            set("DLA_WARMUP", m.warmup.to_string());
            set("DLA_ROUNDS", self.rounds.to_string());
            if !m.run.env.iter().any(|(k, _)| k == "DLA_REPEAT") {
                set("DLA_REPEAT", DEFAULT_REPEAT.to_string());
            }
        }
        for (k, v) in env_extra {
            set(k, (*v).to_owned());
        }
        ExecSpec {
            argv: m.run.argv.iter().map(|a| subst(a, &tokens)).collect(),
            cwd: cwd_of(self.paths, &m.run),
            env,
            log: self.paths.logs.join(format!("{}.log", self.label)),
            stderr_log: None,
            timeout,
            sample: true,
            command,
            vgdb: None,
        }
    }
}

/// The OS figures of the sampler at `t_ms` (or the last ones when `None`) and the largest
/// value of each column up to then.
fn os_at(
    sampler: Option<&SamplerOutput>,
    t_ms: Option<u64>,
) -> (BTreeMap<String, f64>, BTreeMap<String, f64>) {
    let Some(s) = sampler else {
        return Default::default();
    };
    let pick: Option<&OsSample> = match t_ms {
        None => s.samples.last(),
        Some(t) => s.samples.iter().min_by_key(|x| x.t_ms.abs_diff(t)),
    };
    let upto = pick.map_or(0, |p| p.t_ms);
    let mut peaks: BTreeMap<String, f64> = BTreeMap::new();
    if t_ms.is_none() {
        peaks.clone_from(&s.peaks);
    } else {
        for x in s.samples.iter().filter(|x| x.t_ms <= upto) {
            for (k, v) in &x.values {
                let e = peaks.entry(k.clone()).or_insert(*v);
                *e = e.max(*v);
            }
        }
    }
    (pick.map(|p| p.values.clone()).unwrap_or_default(), peaks)
}

fn sites_of(rec: &Rec, paths: &Paths) -> Option<Sites> {
    let raw = rec.raw_sites.as_ref()?;
    Some(symbolize(
        raw,
        rec.modules.as_ref(),
        rec.dump.exe.as_deref(),
        &paths.work,
    ))
}

/// Everything a mode's run collects.
#[derive(Default)]
struct ModeRun {
    points: Vec<PointInput>,
    warmup: u32,
    limitations: Vec<String>,
    dump_cov: Vec<DumpCoverage>,
    detectors: Vec<Value>,
    processes: u32,
    logs: Vec<String>,
    exits: Vec<(String, Option<i32>)>,
    /// Processes that exited with a non-zero code.
    failed_exits: u32,
    /// Labels whose log is scanned for detector lines.
    scan_logs: Vec<(String, PathBuf)>,
    /// Capture files of the points.
    captures: Vec<CaptureFile>,
}

/// Whether this run asks the JVM agent for raw captures.
fn capture_active(manifest: &HarnessManifest, opts: &RunOptions) -> bool {
    !opts.capture.is_empty() && manifest.runtime == dla_core::runtime::Runtime::Jvm
}

/// Lists the capture files of the point `pe` after its process ended, using what the agent
/// said about them, and notes the captures that failed.
#[allow(clippy::too_many_arguments)]
fn gather_captures(
    ctx: &ToolCtx,
    paths: &Paths,
    pe: &PointEnv<'_>,
    pid: Option<u32>,
    file: &DumpFile,
    x: Option<f64>,
    killed: bool,
    run: &mut ModeRun,
) {
    let mut notes = AgentNotes::default();
    for r in file
        .records
        .iter()
        .filter(|r| pid.is_none() || r.dump.pid == pid)
    {
        if let Some(c) = &r.dump.captures {
            notes.records.push((r.dump.x, c.clone()));
        }
    }
    let label_dir = paths.out.join(&pe.label);
    let rel = paths.rel(ctx, &label_dir);
    let (files, left_out) = capture::gather(&label_dir, &rel, &pe.label, pid, &notes, x, killed);
    if !left_out.is_empty() {
        run.limitations.push(format!(
            "{}: {} capture file(s) were not listed because the agent did not confirm them (the process was killed or the capture failed): {}",
            pe.label,
            left_out.len(),
            clip(&left_out.join(", "), 160)
        ));
    }
    for (kind, error) in notes.errors() {
        run.limitations.push(format!(
            "{}: {kind} capture failed: {}",
            pe.label,
            clip(&error, 160)
        ));
    }
    run.captures.extend(files);
}

fn exit_note(label: &str, res: &ExecResult) -> Option<String> {
    match res.code {
        Some(0) => None,
        Some(c) => Some(format!("{label} exited with code {c}")),
        None => Some(format!("{label} ended without an exit code (signal)")),
    }
}

fn run_command_mode(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
    paths: &Paths,
    mode: KeyMode,
    timeout: Duration,
) -> Result<ModeRun, Refusal> {
    let iterations = opts.iterations.unwrap_or(manifest.iterations);
    let total = manifest.warmup + iterations + opts.extra;
    let mut run = ModeRun::default();
    let mut kept_warm = 0;
    let capturing = capture_active(manifest, opts);
    let dumps = opts.capture_dumps.unwrap_or(capture::DEFAULT_DUMPS);
    let heavy = capture::select_points(total, manifest.warmup, dumps);
    for k in opts.first_point.max(1)..=total {
        let n = manifest.step.saturating_mul(u64::from(k));
        let label = format!("p{k}-{}", mode.as_str());
        let last = k == total;
        let pe = PointEnv {
            manifest,
            paths,
            label: label.clone(),
            mode,
            n,
            purge: if last { opts.purge.as_deref() } else { None },
            rounds: 0,
            capture: capturing.then_some((opts.capture, heavy.contains(&k))),
            dumps,
        };
        let dump = pe.dump_path();
        if let Some(d) = dump.parent() {
            fs::create_dir_all(d).map_err(|e| io_refusal(d, e))?;
        }
        let _ = fs::remove_file(&dump);
        if capturing {
            let _ = fs::remove_dir_all(pe.capture_dir());
        }
        let command = subst(
            &manifest.run.argv.join(" "),
            &[("n", n.to_string()), ("label", label.clone())],
        );
        let spec = pe.exec_spec(timeout, &command, &[]);
        let res = exec(ctx, &spec)?;
        run.processes += 1;
        let log_rel = paths.rel(ctx, &spec.log);
        run.logs.push(log_rel.clone());
        run.scan_logs.push((label.clone(), spec.log.clone()));
        run.exits.push((label.clone(), res.code));
        let early_file = dumps::read_dumps(&dump);
        if capturing {
            let pid =
                dumps::select_measured(&early_file, manifest.runtime, Some(n)).and_then(|s| s.pid);
            gather_captures(
                ctx,
                paths,
                &pe,
                pid,
                &early_file,
                Some(n as f64),
                res.timed_out,
                &mut run,
            );
        }
        if let Some(e) = &res.spawn_error {
            run.limitations.push(format!(
                "point {label} (n={n}): {e}; later points were not run"
            ));
            break;
        }
        if res.timed_out {
            run.limitations.push(format!(
                "point {label} (n={n}) did not finish in {} s and was killed (log {log_rel}); it is missing and larger points were not run",
                timeout.as_secs()
            ));
            break;
        }
        if let Some(c) = res.code.filter(|c| *c != 0) {
            // A crashed point (the agent's exit hook still writes a record) is not a
            // measurement of the workload.
            run.failed_exits += 1;
            run.limitations.push(format!(
                "point {label} (n={n}) exited with code {c}; it is dropped (log {log_rel})"
            ));
            continue;
        }
        let file = early_file;
        if file.bad_lines > 0 {
            run.limitations.push(format!(
                "{label}: {} unreadable dump lines were skipped",
                file.bad_lines
            ));
        }
        let sel = dumps::select_measured(&file, manifest.runtime, Some(n));
        let (os_last, os_peak) = os_at(res.sampler.as_ref(), None);
        if let Some(s) = &res.sampler {
            run.limitations.extend(s.limitations.iter().cloned());
        }
        let (rec, sites) = match sel {
            Some(sel) => {
                run.limitations
                    .extend(sel.limitations.iter().map(|l| format!("{label}: {l}")));
                let rec = dumps::merge_shim(sel.rec, &file);
                let sites = sites_of(&rec, paths);
                (Some(rec), sites)
            }
            None => (None, None),
        };
        if rec.is_none() {
            if res.code != Some(0) {
                run.limitations.push(format!(
                    "point {label} (n={n}) {} and wrote no dump record; it is missing (log {log_rel})",
                    exit_note(&label, &res).unwrap_or_default()
                ));
                continue;
            }
            run.limitations.push(format!(
                "point {label} (n={n}) wrote no dump record: only operating-system figures are available (log {log_rel})"
            ));
        } else if let Some(note) = exit_note(&label, &res) {
            run.limitations
                .push(format!("{note}; its dump record was used (log {log_rel})"));
        }
        if let Some(r) = &rec {
            run.dump_cov.extend(r.dump.coverage.clone());
            run.detectors.extend(
                r.dump
                    .detectors
                    .iter()
                    .flatten()
                    .map(|d| json!({"lib": d.lib, "text": d.text, "site": d.site, "label": label})),
            );
        }
        if k <= manifest.warmup {
            kept_warm += 1;
        }
        let t_ms = rec
            .as_ref()
            .and_then(|r| r.dump.t_ms)
            .unwrap_or(res.wall_ms);
        run.points.push(PointInput {
            x: n as f64,
            t_ms,
            rec,
            sites,
            root_pid: res.pid,
            os_last,
            os_peak,
        });
    }
    run.warmup = kept_warm;
    Ok(run)
}

fn run_driver_mode(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
    paths: &Paths,
    mode: KeyMode,
    timeout: Duration,
) -> Result<ModeRun, Refusal> {
    let rounds = opts.iterations.unwrap_or(manifest.iterations) + opts.extra;
    let total_rounds = manifest.warmup + rounds;
    let label = format!("p1-{}", mode.as_str());
    let capturing = capture_active(manifest, opts);
    let dumps_wanted = opts.capture_dumps.unwrap_or(capture::DEFAULT_DUMPS);
    let pe = PointEnv {
        manifest,
        paths,
        label: label.clone(),
        mode,
        n: manifest.step.saturating_mul(u64::from(total_rounds)),
        purge: opts.purge.as_deref(),
        rounds,
        capture: capturing.then_some((opts.capture, true)),
        dumps: dumps_wanted,
    };
    let dump = pe.dump_path();
    if let Some(d) = dump.parent() {
        fs::create_dir_all(d).map_err(|e| io_refusal(d, e))?;
    }
    let _ = fs::remove_file(&dump);
    if capturing {
        let _ = fs::remove_dir_all(pe.capture_dir());
    }
    let command = manifest.run.argv.join(" ");
    let spec = pe.exec_spec(timeout, &command, &[]);
    let res = exec(ctx, &spec)?;
    let mut run = ModeRun {
        processes: 1,
        ..ModeRun::default()
    };
    let log_rel = paths.rel(ctx, &spec.log);
    run.logs.push(log_rel.clone());
    run.scan_logs.push((label.clone(), spec.log.clone()));
    run.exits.push((label.clone(), res.code));
    run.failed_exits = u32::from(res.code.is_some_and(|c| c != 0));
    if let Some(e) = &res.spawn_error {
        run.limitations.push(format!("driver {label}: {e}"));
        return Ok(run);
    }
    if res.timed_out {
        run.limitations.push(format!(
            "driver {label} did not finish in {} s and was killed (log {log_rel}); the rounds written so far are kept",
            timeout.as_secs()
        ));
    } else if let Some(note) = exit_note(&label, &res) {
        run.limitations.push(format!("{note} (log {log_rel})"));
    }
    if let Some(s) = &res.sampler {
        run.limitations.extend(s.limitations.iter().cloned());
    }
    let file: DumpFile = dumps::read_dumps(&dump);
    if file.bad_lines > 0 {
        run.limitations.push(format!(
            "{label}: {} unreadable dump lines were skipped",
            file.bad_lines
        ));
    }
    let (cps, pid, notes) = dumps::select_checkpoints(&file, manifest.runtime);
    run.limitations.extend(notes);
    if capturing {
        gather_captures(ctx, paths, &pe, pid, &file, None, res.timed_out, &mut run);
    }
    if cps.is_empty() {
        run.limitations.push(format!(
            "driver {label} wrote no checkpoint record (log {log_rel}): did the hooks run?"
        ));
        return Ok(run);
    }
    if cps.len() < total_rounds as usize {
        run.limitations.push(format!(
            "driver {label} wrote {} of {total_rounds} rounds",
            cps.len()
        ));
    }
    let last = cps.len() - 1;
    for (i, rec) in cps.into_iter().enumerate() {
        // The allocator shim of a JVM that calls a native library reports once, at exit:
        // its figures belong to the last round.
        let merged;
        let rec: &Rec = if i == last {
            merged = dumps::merge_shim(rec, &file);
            &merged
        } else {
            rec
        };
        let t_ms = rec.dump.t_ms;
        let (os_last, os_peak) = os_at(res.sampler.as_ref(), t_ms);
        let x = rec
            .dump
            .x
            .unwrap_or_else(|| manifest.step as f64 * (i + 1) as f64);
        run.dump_cov.extend(rec.dump.coverage.clone());
        run.detectors.extend(
            rec.dump
                .detectors
                .iter()
                .flatten()
                .map(|d| json!({"lib": d.lib, "text": d.text, "site": d.site, "label": label})),
        );
        let sites = if i == last || rec.raw_sites.is_some() {
            sites_of(rec, paths)
        } else {
            None
        };
        run.points.push(PointInput {
            x,
            t_ms: t_ms.unwrap_or(res.wall_ms),
            rec: Some(rec.clone()),
            sites,
            root_pid: res.pid,
            os_last,
            os_peak,
        });
    }
    run.warmup = manifest
        .warmup
        .min(u32::try_from(run.points.len()).unwrap_or(0));
    Ok(run)
}

// ---------------------------------------------------------------------------------------
// Detectors

/// Environment of the extra detector pass, from the manifest's abort-type detectors
/// (`-W error::ResourceWarning`, `PYTHONWARNINGS=error::ResourceWarning`, a
/// `MaxListenersExceededWarning` capture); empty when there are none.
fn abort_detector_env(manifest: &HarnessManifest) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = Vec::new();
    for d in &manifest.detectors {
        let lower = d.to_lowercase();
        if let Some((k, v)) = d.split_once('=')
            && k.chars()
                .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
            && (lower.contains("error::") || lower.contains("maxlisteners"))
        {
            env.push((k.to_owned(), v.to_owned()));
        } else if lower.contains("error::") {
            let filter = d
                .trim_start_matches("-W")
                .trim()
                .trim_start_matches('=')
                .to_owned();
            env.push(("PYTHONWARNINGS".to_owned(), filter));
        } else if lower.contains("maxlisteners") || lower.contains("node-warning") {
            env.push(("NODE_OPTIONS".to_owned(), "--trace-warnings".to_owned()));
        }
    }
    env
}

/// The log lines that mark a leak a library reported.
fn scan_log(label: &str, text: &str, site_re: &Regex) -> Vec<Value> {
    let rules: [(&str, &str); 5] = [
        ("LEAK:", "netty"),
        ("ResourceWarning", "python"),
        ("Task was destroyed but it is pending", "asyncio"),
        ("MaxListenersExceededWarning", "node"),
        ("Connection leak detection triggered", "hikari"),
    ];
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((_, lib)) = rules.iter().find(|(pat, _)| line.contains(pat)) else {
            continue;
        };
        let t: String = line.trim().chars().take(200).collect();
        let site = site_re.find(line).map(|m| m.as_str().to_owned());
        out.push(json!({"lib": lib, "text": t, "site": site, "label": label}));
    }
    out
}

/// Merges duplicate detector hits (same lib and text) into one with a count.
fn dedupe_detectors(hits: Vec<Value>) -> Vec<Value> {
    let mut out: Vec<(String, Value, u64)> = Vec::new();
    for h in hits {
        let key = format!("{}|{}", h["lib"], h["text"]);
        match out.iter_mut().find(|(k, _, _)| *k == key) {
            Some((_, _, c)) => *c += 1,
            None => out.push((key, h, 1)),
        }
    }
    out.into_iter()
        .take(MAX_DETECTORS)
        .map(|(_, mut v, c)| {
            v["count"] = json!(c);
            v
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Entry points

/// Builds (once) and measures `manifest`, writing one series per key mode. Needs tier 1.
/// Writes no evidence: [`run`] and `dla alloc-test` add their own.
pub fn run_points(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
) -> Result<RunOutput, Refusal> {
    ctx.require_tier(1, "dla harness run")?;
    let paths = Paths::new(ctx, manifest)?;
    let mut out = RunOutput {
        gaps: manifest.gaps.clone(),
        ..RunOutput::default()
    };
    let _signals = SignalGuard::install(ctx);
    build_once(ctx, manifest, &paths, &mut out)?;
    if let Some(tool) = opts.tool {
        tool::run_tool_points(ctx, manifest, opts, &paths, tool, &mut out)?;
        return Ok(out);
    }
    if capture_active(manifest, opts) && opts.capture.heap && manifest.mode == HarnessMode::Command
    {
        out.limitations.push(
            "command-mode heap dumps come from separate processes: dla summarize heap-graph compares dumps of one process only, so use a driver (dla harness gen --driver) for a dump series"
                .to_owned(),
        );
    }
    if !opts.capture.is_empty() && manifest.runtime != dla_core::runtime::Runtime::Jvm {
        out.limitations.push(format!(
            "--capture is available for jvm harnesses; {} writes no capture files (use --tool for rust and native code)",
            manifest.runtime.as_str()
        ));
    }

    let secs = opts.timeout_secs.unwrap_or(manifest.timeout_secs);
    let timeout = Duration::from_secs(if secs == 0 {
        DEFAULT_TIMEOUT_SECS
    } else {
        secs
    });
    let command_mode = manifest.mode == HarnessMode::Command;
    let modes: Vec<KeyMode> = if command_mode {
        if opts.modes.iter().any(|m| *m != KeyMode::Native) {
            out.limitations.push(
                "command mode has one key mode (native): the requested fresh/repeated modes only exist in driver mode"
                    .to_owned(),
            );
        }
        vec![KeyMode::Native]
    } else {
        let m = if opts.modes.is_empty() {
            manifest.key_modes.clone()
        } else {
            opts.modes.clone()
        };
        if m.is_empty() {
            vec![KeyMode::Fresh, KeyMode::Repeated]
        } else {
            m
        }
    };

    let mut dump_cov: Vec<DumpCoverage> = Vec::new();
    let mut hits: Vec<Value> = Vec::new();
    let mut scan_logs: Vec<(String, PathBuf)> = Vec::new();
    let mut usable_modes = 0;
    let mut max_post_units = 0u64;
    let mut first_log = None;
    let mut harness_failed: Option<Refusal> = None;
    for mode in modes {
        let run = if command_mode {
            run_command_mode(ctx, manifest, opts, &paths, mode, timeout)?
        } else {
            run_driver_mode(ctx, manifest, opts, &paths, mode, timeout)?
        };
        out.processes += run.processes;
        out.logs.extend(run.logs.iter().cloned());
        out.exits.extend(run.exits.iter().cloned());
        out.captures.extend(run.captures.iter().cloned());
        first_log = first_log.or_else(|| run.logs.first().cloned());
        scan_logs.extend(run.scan_logs.iter().cloned());
        dump_cov.extend(run.dump_cov.iter().cloned());
        hits.extend(run.detectors.iter().cloned());
        if run.processes > 0 && run.failed_exits == run.processes {
            let logs = run.logs.join(", ");
            let codes = run
                .exits
                .iter()
                .map(|(l, c)| {
                    format!(
                        "{l}: {}",
                        c.map_or_else(|| "signal".to_owned(), |c| c.to_string())
                    )
                })
                .take(4)
                .collect::<Vec<_>>()
                .join(", ");
            out.limitations.extend(run.limitations.iter().cloned());
            out.limitations.push(format!(
                "every measured process of mode {} exited with a non-zero code ({codes}); no series was written for it",
                mode.as_str()
            ));
            harness_failed.get_or_insert_with(|| {
                failed(
                    format!(
                        "every measured process of mode {} exited with a non-zero code ({codes}): the harness itself failed, so no series was written",
                        mode.as_str()
                    ),
                    format!("a harness that runs to completion; see the logs {logs}"),
                    format!(
                        "read the logs, fix the harness or its hooks, then dla harness run {}",
                        manifest.id
                    ),
                )
            });
            continue;
        }
        let mut limitations = run.limitations.clone();
        if run.points.is_empty() {
            out.limitations.extend(limitations);
            out.limitations
                .push(format!("no usable point in mode {}", mode.as_str()));
            continue;
        }
        if run.points.iter().any(|p| p.rec.is_none()) {
            limitations.push(
                "points without a dump record carry the sampler's last reading before exit (100 ms interval) for os.* columns"
                    .to_owned(),
            );
        }
        let sites_ffi = ffi_sites(manifest, &run.points);
        for f in sites_ffi.iter().take(3) {
            limitations.push(format!(
                "native.live_bytes@{} is {}, the native method {}{} ({} B live at the last point)",
                f.site,
                f.export,
                f.managed,
                f.declared_at
                    .as_ref()
                    .map_or_else(String::new, |d| format!(" declared at {d}")),
                f.live_bytes as u64
            ));
        }
        if !sites_ffi.is_empty() {
            out.ffi_sites = sites_ffi;
        }
        let params = SeriesParams {
            id: ctx.next_series_id(),
            kind: opts.source_kind,
            mode,
            axis: if command_mode {
                Axis::N
            } else {
                Axis::Iteration
            },
            warmup: run.warmup,
            command: manifest.run.argv.join(" "),
            limitations,
            purge: opts.purge.clone(),
        };
        let series = build_series(ctx, manifest, &params, &run.points);
        series.save(&ctx.run_dir).map_err(|e| e.to_refusal())?;
        out.points = out
            .points
            .max(u32::try_from(series.points.len()).unwrap_or(u32::MAX));
        let post = series.points.len().saturating_sub(series.warmup as usize);
        max_post_units = max_post_units.max(post as u64 * manifest.step.max(1));
        let (lo, hi) = (
            series.points.first().map_or(0.0, |p| p.x),
            series.points.last().map_or(0.0, |p| p.x),
        );
        out.x_range = Some(match out.x_range {
            Some((a, b)) => (a.min(lo), b.max(hi)),
            None => (lo, hi),
        });
        for l in &series.limitations {
            if !out.limitations.contains(l) {
                out.limitations.push(l.clone());
            }
        }
        out.modes.push(mode);
        out.series.push(series.id.clone());
        usable_modes += 1;
    }

    // Detector pass at the warm-up n: abort-type detectors would end a measured run.
    let det_env = abort_detector_env(manifest);
    if command_mode && !det_env.is_empty() && usable_modes > 0 {
        let n = manifest
            .step
            .saturating_mul(u64::from(manifest.warmup.max(1)));
        let pe = PointEnv {
            manifest,
            paths: &paths,
            label: "detect".to_owned(),
            mode: KeyMode::Native,
            n,
            purge: None,
            rounds: 0,
            capture: None,
            dumps: 0,
        };
        let dump = pe.dump_path();
        if let Some(d) = dump.parent() {
            fs::create_dir_all(d).map_err(|e| io_refusal(d, e))?;
        }
        let _ = fs::remove_file(&dump);
        let extra: Vec<(&str, String)> = Vec::new();
        let command = format!("{} (detector pass)", manifest.run.argv.join(" "));
        let mut spec = pe.exec_spec(timeout, &command, &extra);
        // Abort-type detector settings merge like the manifest's own env.
        let tokens: [(&str, String); 0] = [];
        for (k, v) in merged_env(&det_env, &tokens) {
            spec.env.retain(|(ek, _)| *ek != k);
            // Keep what the manifest already put in a combinable variable.
            let prior = manifest
                .run
                .env
                .iter()
                .find(|(ek, _)| *ek == k)
                .map(|(_, pv)| pv.clone());
            let v = match (merge_rule(&k), prior) {
                (Merge::Append(sep), Some(p)) => format!("{p}{sep}{v}"),
                (Merge::Prepend(sep), Some(p)) => format!("{v}{sep}{p}"),
                _ => v,
            };
            spec.env.push((k, v));
        }
        spec.sample = false;
        let res = exec(ctx, &spec)?;
        out.processes += 1;
        out.logs.push(paths.rel(ctx, &spec.log));
        if res.timed_out {
            out.limitations.push(format!(
                "the detector pass at n={n} did not finish in {} s and was killed",
                timeout.as_secs()
            ));
        }
        scan_logs.push(("detect".to_owned(), spec.log.clone()));
    }

    let site_re = Regex::new(r"[\w./-]+\.\w{1,4}:\d+").expect("static regex");
    for (label, log) in &scan_logs {
        if let Ok(text) = fs::read_to_string(log) {
            hits.extend(scan_log(label, &text, &site_re));
        }
    }
    out.detectors = dedupe_detectors(hits);

    if usable_modes == 0 {
        if let Some(r) = harness_failed {
            cover::record(
                ctx,
                manifest.runtime,
                &CoverageOutcome::Failed {
                    reason: crate::NO_HARNESS,
                },
            )?;
            return Err(r);
        }
        cover::record(
            ctx,
            manifest.runtime,
            &CoverageOutcome::Failed {
                reason: crate::NO_HARNESS,
            },
        )?;
        let log = first_log.unwrap_or_else(|| format!("harness/{}/logs", manifest.id));
        return Err(failed(
            format!(
                "harness {} produced no usable point: {}",
                manifest.id,
                out.limitations
                    .iter()
                    .take(2)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            format!("at least one measured point; see {}", ctx.pointer(&log)),
            format!(
                "fix the cause in the log, or regenerate with dla harness gen, then dla harness run {}",
                manifest.id
            ),
        ));
    }

    if capture_active(manifest, opts) {
        record_capture_coverage(ctx, manifest, opts, &mut out)?;
    }
    let cov_limits = cover::record(
        ctx,
        manifest.runtime,
        &CoverageOutcome::Ran {
            iterations: max_post_units,
            dump_cov,
            hdir: &paths.hdir,
            cov_dir: &paths.cov,
            work: &paths.work,
        },
    )?;
    out.limitations.extend(cov_limits);
    out.units = max_post_units;
    Ok(out)
}

/// One coverage row per requested capture kind: `ran` with the number of files, or `blocked`
/// when the agent wrote none (the reason is among the run's limitations).
fn record_capture_coverage(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
    out: &mut RunOutput,
) -> Result<(), Refusal> {
    use dla_core::{
        coverage::{BackendCoverage, CoverageManifest},
        harness::CaptureKind,
    };
    let rows = [
        (opts.capture.heap, "heap", CaptureKind::Heap),
        (opts.capture.histogram, "histogram", CaptureKind::Histogram),
        (opts.capture.jfr, "jfr", CaptureKind::Jfr),
        (opts.capture.nonheap, "nonheap", CaptureKind::NonHeap),
    ];
    let mut blocked = Vec::new();
    CoverageManifest::update(&ctx.run_dir, |m| {
        for (on, name, kind) in rows {
            if !on {
                continue;
            }
            let n = out.captures.iter().filter(|c| c.kind == kind).count() as u64;
            let cov = if n > 0 {
                BackendCoverage::ran(None, n)
            } else {
                blocked.push(name);
                BackendCoverage::blocked(format!(
                    "the jvm agent wrote no {name} file (see the run's limitations)"
                ))
            };
            m.note_backend(manifest.runtime.as_str(), &format!("harness-{name}"), cov);
        }
    })
    .map_err(|e| e.to_refusal())?;
    for name in blocked {
        out.limitations
            .push(format!("--capture {name}: no file was written"));
    }
    Ok(())
}

fn ffi_site_json(f: &FfiSite) -> Value {
    json!({
        "site": f.site,
        "method": f.managed,
        "live_bytes": f.live_bytes as u64,
    })
}

/// Bytes and milliseconds of the captures, in total and by kind as `[files, bytes, ms]`
/// (bounded: one entry per kind).
fn capture_cost(out: &RunOutput) -> Value {
    let mut by_kind: BTreeMap<&str, (u64, u64, u64)> = BTreeMap::new();
    for c in &out.captures {
        let e = by_kind.entry(c.kind.as_str()).or_default();
        e.0 += 1;
        e.1 += c.bytes;
        e.2 += c.ms.unwrap_or(0);
    }
    let kinds: serde_json::Map<String, Value> = by_kind
        .iter()
        .map(|(k, (n, b, ms))| ((*k).to_owned(), json!([n, b, ms])))
        .collect();
    json!({
        "files": out.captures.len(),
        "bytes": out.captures.iter().map(|c| c.bytes).sum::<u64>(),
        "ms": out.captures.iter().filter_map(|c| c.ms).sum::<u64>(),
        "kinds": kinds,
    })
}

/// The full capture report written next to the manifest.
fn capture_report(out: &RunOutput, commands: &[String]) -> Value {
    json!({
        "tool": out.tool.map(NativeTool::as_str),
        "cost": capture_cost(out),
        "files": out.captures,
        "summarize": commands,
        "limitations": out.limitations,
    })
}

/// Cuts text to `max` characters.
fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

/// `dla harness run <id>`: [`run_points`] plus one `workload` evidence (authority test)
/// whose result names the harness.
pub fn run(ctx: &ToolCtx, id: &str, opts: &RunOptions) -> Result<StageOutcome, Refusal> {
    ctx.require_time("harness run")?;
    let mut manifest = ctx.harness(id)?;
    let out = run_points(ctx, &manifest, opts)?;
    let captured = !opts.capture.is_empty() || opts.tool.is_some();
    let mut commands = capture::summarize_commands(&out.captures, &|p| ctx.pointer(p), false);
    let mut compact = capture::summarize_commands(&out.captures, &|p| ctx.pointer(p), true);
    if captured && opts.tool.is_none() {
        // A JVM harness with the allocator shim: the shim's exit records are a
        // type-histogram-diff input of their own.
        let shim = capture::shim_dump_commands(
            &harness_dir(&ctx.run_dir, &manifest.id).join(OUT),
            &manifest.id,
            &|p| ctx.pointer(p),
        );
        for list in [&mut commands, &mut compact] {
            let at = list
                .iter()
                .rposition(|x| x.starts_with("dla summarize type-histogram-diff"))
                .map_or(0, |i| i + 1);
            list.splice(at..at, shim.iter().cloned());
        }
    }
    if captured {
        // The manifest lists what the latest capturing run left in out/.
        manifest.captures.clone_from(&out.captures);
        manifest.save(&ctx.run_dir).map_err(|e| e.to_refusal())?;
        let list = capture_report(&out, &commands);
        let path = ctx
            .run_dir
            .join("harness")
            .join(&manifest.id)
            .join("captures.json");
        let text = serde_json::to_string_pretty(&list).unwrap_or_default() + "\n";
        fs::write(&path, text).map_err(|e| io_refusal(&path, e))?;
    }

    // Full result in a file; the evidence carries a bounded summary of it.
    let full = json!({
        "harness": manifest.id,
        "mode": manifest.mode.as_str(),
        "modes": out.modes.iter().map(|m| m.as_str()).collect::<Vec<_>>(),
        "series": out.series,
        "points": out.points,
        "x_range": out.x_range,
        "units": out.units,
        "processes": out.processes,
        "exits": out.exits.iter().map(|(l, c)| json!({"label": l, "code": c})).collect::<Vec<_>>(),
        "detectors": out.detectors,
        "gaps": out.gaps,
        "limitations": out.limitations,
        "logs": out.logs,
        "tool": out.tool.map(NativeTool::as_str),
        "captures": out.captures,
        "ffi_sites": out.ffi_sites.iter().map(ffi_site_json).collect::<Vec<_>>(),
    });
    let first = out
        .series
        .first()
        .cloned()
        .or_else(|| out.tool.map(|t| t.as_str().to_owned()))
        .unwrap_or_default();
    let raw_rel = format!("harness/{}/run-{first}.json", manifest.id);
    let raw_path = ctx.run_dir.join(&raw_rel);
    let text = serde_json::to_string_pretty(&full).unwrap_or_default() + "\n";
    fs::write(&raw_path, text).map_err(|e| io_refusal(&raw_path, e))?;

    let hits = &out.detectors;
    let ids: Vec<&str> = out.series.iter().map(String::as_str).collect();
    let mut result = json!({
        "harness": manifest.id,
        "workload": manifest.workload,
        "mode": manifest.mode.as_str(),
        "key_modes": out.modes.iter().map(|m| m.as_str()).collect::<Vec<_>>(),
        "points": out.points,
        "warmup": manifest.warmup,
        "series": ids,
        "processes": out.processes,
        "failed_processes": out.exits.iter().filter(|(_, c)| *c != Some(0)).count(),
        "detectors": {
            "count": hits.len(),
            "first": hits.iter().take(3).map(|h| json!({
                "lib": h["lib"], "text": h["text"].as_str().map(|t| clip(t, 100)),
            })).collect::<Vec<_>>(),
        },
        "gaps": out.gaps.iter().take(4).map(|g| clip(g, 120)).collect::<Vec<_>>(),
        "limitations": {
            "count": out.limitations.len(),
            "first": out.limitations.iter().take(3).map(|l| clip(l, 140)).collect::<Vec<_>>(),
        },
    });
    if manifest.mode == HarnessMode::Command {
        result["n_range"] = json!(out.x_range);
        result["step"] = json!(manifest.step);
    } else {
        result["units"] = json!(out.units);
    }
    if captured {
        result["capture"] = capture_cost(&out);
        if let Some(t) = out.tool {
            result["tool"] = json!(t.as_str());
        }
    }
    if !out.ffi_sites.is_empty() {
        result["ffi"] = json!(
            out.ffi_sites
                .iter()
                .take(2)
                .map(ffi_site_json)
                .collect::<Vec<_>>()
        );
    }
    // Keep the evidence under its size bound whatever the run said.
    while serde_json::to_vec(&result).map_or(0, |v| v.len()) > 1900 {
        let trimmed = ["gaps", "limitations", "detectors"].iter().any(|k| {
            let arr = match k {
                &"gaps" => result.get_mut("gaps"),
                other => result.get_mut(*other).and_then(|v| v.get_mut("first")),
            };
            match arr.and_then(Value::as_array_mut) {
                Some(a) if !a.is_empty() => {
                    a.pop();
                    true
                }
                _ => false,
            }
        });
        if !trimmed {
            break;
        }
    }

    let source = ctx.artifact_source(&format!("harness/{}/harness.json", manifest.id))?;
    let mut spec = EvidenceSpec::new(
        EvidenceKind::Workload,
        "dla",
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = Authority::Test;
    spec.source = Some(source);
    spec.raw = Some(raw_rel.clone());
    spec.coverage = EvidenceCoverage {
        status: CoverageStatus::Partial,
        examined: Some(u64::from(out.points)),
        total: Some(u64::from(
            manifest.warmup + opts.iterations.unwrap_or(manifest.iterations) + opts.extra,
        )),
    };
    spec.limitations = out
        .limitations
        .iter()
        .take(8)
        .map(|l| clip(l, 200))
        .collect();
    let mut ledger = ctx.ledger()?;
    let ev = ctx.add_evidence(&mut ledger, spec)?;

    let mut artifacts: Vec<String> = out
        .series
        .iter()
        .map(|s| ctx.pointer(&format!("series/{s}.json")))
        .collect();
    artifacts.push(ctx.pointer(&raw_rel));
    let mut summary = json!({
        "harness": manifest.id,
        "series": out.series,
        "points": out.points,
    });
    if out.tool.is_some()
        && let Value::Object(m) = &mut summary
    {
        m.remove("points");
        m.remove("series");
    }
    if !out.ffi_sites.is_empty() {
        summary["ffi"] = json!(
            out.ffi_sites
                .iter()
                .take(2)
                .map(ffi_site_json)
                .collect::<Vec<_>>()
        );
    }
    if captured {
        artifacts.push(ctx.pointer(&format!("harness/{}/captures.json", manifest.id)));
        summary["capture"] = capture_cost(&out);
        if let Some(t) = out.tool {
            summary["tool"] = json!(t.as_str());
        }
        // The CLI output is bounded at 2048 bytes and the envelope around the summary takes
        // about 800: the commands get what is left, whole or not at all.
        let used = summary.to_string().len();
        let budget = capture::SUMMARY_BUDGET.saturating_sub(used + 40);
        let (shown, left) = capture::fit(&compact, budget);
        if !shown.is_empty() {
            summary["summarize"] = json!(shown);
        }
        if left > 0 {
            summary["more_commands"] = json!(format!(
                "{left} more in harness/{}/captures.json",
                manifest.id
            ));
        }
    }
    let next = out.series.first().map_or_else(
        || {
            commands
                .first()
                .map(|_| "run the dla summarize commands listed".to_owned())
                .unwrap_or_default()
        },
        |s| format!("dla trend {s}"),
    );
    summary["next"] = json!(next);
    Ok(StageOutcome {
        evidence: vec![ev.to_string()],
        artifacts,
        summary,
        ..StageOutcome::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_replaced_everywhere_they_appear() {
        let t = [("n", "300".to_owned()), ("label", "p3-native".to_owned())];
        assert_eq!(subst("--n={n} {label} {n}", &t), "--n=300 p3-native 300");
        assert_eq!(subst("{unknown}", &t), "{unknown}");
    }

    #[test]
    fn gradle_options_and_library_paths_keep_the_inherited_value() {
        assert!(matches!(merge_rule("GRADLE_OPTS"), Merge::Append(' ')));
        assert!(matches!(merge_rule("LD_LIBRARY_PATH"), Merge::Prepend(':')));
        assert!(matches!(
            merge_rule("DYLD_LIBRARY_PATH"),
            Merge::Prepend(':')
        ));
    }

    #[test]
    fn env_keys_append_prepend_or_replace() {
        // SAFETY-free: only reads the process environment, which these keys are in.
        let key_a = "DLA_TEST_ONLY_UNSET_A";
        let env = merged_env(
            &[
                (
                    "JAVA_TOOL_OPTIONS".to_owned(),
                    "-javaagent:/a.jar".to_owned(),
                ),
                (key_a.to_owned(), "x".to_owned()),
                ("PYTHONPATH".to_owned(), "/agent".to_owned()),
            ],
            &[],
        );
        let get = |k: &str| env.iter().find(|(ek, _)| ek == k).map(|(_, v)| v.clone());
        let jto = get("JAVA_TOOL_OPTIONS").unwrap();
        assert!(jto.ends_with("-javaagent:/a.jar"));
        if let Ok(old) = std::env::var("JAVA_TOOL_OPTIONS")
            && !old.is_empty()
        {
            assert_eq!(jto, format!("{old} -javaagent:/a.jar"));
        }
        assert_eq!(get(key_a).as_deref(), Some("x"));
        let pp = get("PYTHONPATH").unwrap();
        assert!(pp.starts_with("/agent"));
    }

    #[test]
    fn library_lines_in_logs_become_detector_hits_once() {
        let re = Regex::new(r"[\w./-]+\.\w{1,4}:\d+").unwrap();
        let text = "ok\nLEAK: ByteBuf.release() was not called\nResourceWarning: unclosed file at app/io.py:12\nplain\n";
        let hits = scan_log("p1-native", text, &re);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[1]["lib"], "python");
        let again = dedupe_detectors([hits.clone(), hits].concat());
        assert_eq!(again.len(), 2);
        assert_eq!(again[0]["count"], 2);
    }

    #[test]
    fn native_sites_in_jni_exports_are_tied_to_the_managed_method() {
        use dla_core::harness::{FfiHarness, FfiMethod};
        let mut m = super::tests_support::manifest();
        m.runtime = dla_core::runtime::Runtime::Jvm;
        m.ffi = Some(FfiHarness {
            kind: "jni".into(),
            library: Some("codec".into()),
            native_dir: Some("native".into()),
            methods: vec![FfiMethod {
                export: "Java_shop_codec_Native_open".into(),
                managed: "shop.codec.Native.open".into(),
                declared_at: Some("kotlin/shop/Native.kt:9".into()),
            }],
        });
        let mut sites = Sites::default();
        sites.by_site.insert(
            "native/src/lib.rs:40".into(),
            crate::harness::symbolize::SiteTotals {
                bytes: 900.0,
                blocks: 3.0,
            },
        );
        sites.by_site.insert(
            "native/src/lib.rs:43".into(),
            crate::harness::symbolize::SiteTotals {
                bytes: 50.0,
                blocks: 1.0,
            },
        );
        sites.functions.insert(
            "native/src/lib.rs:40".into(),
            "Java_shop_codec_Native_open".into(),
        );
        sites.functions.insert(
            "native/src/lib.rs:43".into(),
            "Java_shop_codec_Native_other".into(),
        );
        let points = vec![PointInput {
            sites: Some(sites),
            ..PointInput::default()
        }];
        let found = ffi_sites(&m, &points);
        assert_eq!(found.len(), 1, "only the exports of the boundary count");
        assert_eq!(found[0].site, "native/src/lib.rs:40");
        assert_eq!(found[0].managed, "shop.codec.Native.open");
        assert_eq!(
            found[0].declared_at.as_deref(),
            Some("kotlin/shop/Native.kt:9")
        );
        // No boundary, no sites.
        m.ffi = None;
        assert!(ffi_sites(&m, &points).is_empty());
    }

    #[test]
    fn abort_detectors_map_to_an_environment() {
        let mut m = super::tests_support::manifest();
        m.detectors = vec![
            "-W error::ResourceWarning".into(),
            "PYTHONASYNCIODEBUG=1".into(),
        ];
        let env = abort_detector_env(&m);
        assert_eq!(
            env,
            vec![(
                "PYTHONWARNINGS".to_owned(),
                "error::ResourceWarning".to_owned()
            )]
        );
        m.detectors = vec!["MaxListenersExceededWarning".into()];
        assert_eq!(abort_detector_env(&m)[0].0, "NODE_OPTIONS");
        m.detectors.clear();
        assert!(abort_detector_env(&m).is_empty());
    }
}

#[cfg(test)]
mod tests_support {
    use dla_core::{
        harness::{AgentKind, CommandSpec, HarnessManifest, HarnessMode},
        runtime::Runtime,
        series::KeyMode,
    };

    pub fn manifest() -> HarnessManifest {
        HarnessManifest {
            schema_version: 1,
            id: "H1".into(),
            runtime: Runtime::Python,
            mode: HarnessMode::Command,
            workload: None,
            scratch: "harness/H1/work".into(),
            build: None,
            run: CommandSpec {
                argv: vec!["python3".into()],
                cwd: ".".into(),
                env: vec![],
            },
            iter_arg: None,
            warmup: 1,
            iterations: 8,
            step: 10,
            key_modes: vec![KeyMode::Native],
            probes: vec![],
            agent: AgentKind::None,
            detectors: vec![],
            tools: vec![],
            gaps: vec![],
            timeout_secs: 30,
            ffi: None,
            captures: vec![],
        }
    }
}
