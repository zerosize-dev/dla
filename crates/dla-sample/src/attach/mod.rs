//! Attaching a tool to a process DLA did not start: `jcmd`, the Node.js inspector, `memray`,
//! `heaptrack` and `goref`.
//!
//! Every backend has two steps. [`plan`] only reads (the target's identity, which tools exist,
//! a read-only size probe) and returns a [`Plan`]: the expected signal, the stopping condition,
//! the effects, the cleanup and an [`Estimate`] of the dump size and the extra memory. The
//! caller turns the plan into a [`Preflight`] and refuses when the gate fails. Only then does
//! [`run`] touch the target. Cleanup actions use the grammar of `dla run abort` ([`Cleanup`]).
//!
//! Test hooks of the caller (not of this crate): none. The numbers the gate reads are plain
//! arguments of [`preflight`].

mod bpftrace;
mod goref;
mod heaptrack;
mod jcmd;
mod memray;
mod node;
mod parse;
mod resources;
mod util;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Serialize;
use serde_json::{Map, Value, json};

pub use parse::{
    HeaptrackSummary, Histogram, HistogramRow, JfrEvent, MemraySummary, NmtCategory, NmtSummary,
    PeakConsumer, parse_heaptrack_print, parse_heaptrack_size, parse_histogram, parse_jfr_summary,
    parse_memray_stats, parse_nmt_summary,
};
pub use resources::{
    Estimate, MemoryAvailable, cgroup_available_at, cgroup_path, cgroup_v1_available,
    cgroup_v2_available, dump_exceeds_disk, free_disk, gate, human, parse_meminfo_available,
    peak_exceeds_memory, read_memory_available,
};
pub use util::{
    AttachError, Cleanup, ErrorKind, TargetInfo, dir_writable_by, inspect_target, mode_grants,
    output_name, own_uid, parse_status_ids,
};

/// An attach backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Backend {
    /// `jcmd` against a JVM.
    Jcmd,
    /// The Node.js inspector over a websocket.
    NodeInspector,
    /// `memray attach` against CPython.
    Memray,
    /// `heaptrack --pid` against a native process.
    Heaptrack,
    /// `grf attach` against a Go process.
    Goref,
    /// `bpftrace` over the mmap, munmap and brk tracepoints, filtered to one process.
    Bpftrace,
}

/// An operation of a backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Op {
    /// `GC.class_histogram`.
    Histogram,
    /// `GC.heap_dump`.
    HeapDump,
    /// `Thread.dump_to_file -format=json`.
    ThreadDump,
    /// `VM.native_memory summary`.
    Nmt,
    /// `JFR.start` for a duration.
    Jfr,
    /// `process.memoryUsage()`.
    Memory,
    /// `HeapProfiler.takeHeapSnapshot`.
    HeapSnapshot,
    /// Record allocations for a duration (memray, heaptrack).
    Record,
    /// `grf attach`.
    HeapProfile,
}

impl Backend {
    /// All backends, in the order the help lists them.
    pub const ALL: [Backend; 6] = [
        Backend::Jcmd,
        Backend::NodeInspector,
        Backend::Memray,
        Backend::Heaptrack,
        Backend::Goref,
        Backend::Bpftrace,
    ];

    /// The name used on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Self::Jcmd => "jcmd",
            Self::NodeInspector => "node-inspector",
            Self::Memray => "memray",
            Self::Heaptrack => "heaptrack",
            Self::Goref => "goref",
            Self::Bpftrace => "bpftrace",
        }
    }

    /// Parses a command-line name.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.name() == name)
    }

    /// The operations of this backend.
    pub fn ops(self) -> &'static [Op] {
        match self {
            Self::Jcmd => &[
                Op::Histogram,
                Op::HeapDump,
                Op::ThreadDump,
                Op::Nmt,
                Op::Jfr,
            ],
            Self::NodeInspector => &[Op::Memory, Op::HeapSnapshot],
            Self::Memray | Self::Heaptrack | Self::Bpftrace => &[Op::Record],
            Self::Goref => &[Op::HeapProfile],
        }
    }
}

impl Op {
    /// The name used on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Self::Histogram => "histogram",
            Self::HeapDump => "heap-dump",
            Self::ThreadDump => "thread-dump",
            Self::Nmt => "nmt",
            Self::Jfr => "jfr",
            Self::Memory => "memory",
            Self::HeapSnapshot => "heap-snapshot",
            Self::Record => "record",
            Self::HeapProfile => "heap-profile",
        }
    }
}

/// Resolves `--backend`/`--op` text; the error names the valid choices.
pub fn resolve(backend: &str, op: &str) -> Result<(Backend, Op), String> {
    let Some(b) = Backend::parse(backend) else {
        let all: Vec<_> = Backend::ALL.iter().map(|b| b.name()).collect();
        return Err(format!(
            "unknown backend {backend:?}; the backends are {}",
            all.join(", ")
        ));
    };
    match b.ops().iter().find(|o| o.name() == op) {
        Some(o) => Ok((b, *o)),
        None => {
            let ops: Vec<_> = b.ops().iter().map(|o| o.name()).collect();
            Err(format!(
                "{} has no operation {op:?}; its operations are {}",
                b.name(),
                ops.join(", ")
            ))
        }
    }
}

/// The consent tier an operation needs and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TierNeed {
    /// Tier 1 to 4.
    pub tier: u8,
    /// One line for the refusal and the preflight.
    pub why: &'static str,
}

/// The tier of `(backend, op)` for a target DLA did not start.
///
/// * jcmd histogram, thread dump, NMT and the Node memory counters observe a running process
///   through its attach mechanism: tier 2.
/// * `GC.heap_dump` and `HeapProfiler.takeHeapSnapshot` capture memory contents: tier 4.
/// * JFR records allocation samples (class and stack, never object contents) through the same
///   attach mechanism as the other jcmd diagnostics, with no debugger and no kernel facility:
///   tier 2. The file also carries the JVM arguments and environment variables, so it is
///   treated as a file that may hold secrets and stays under `.dla/` like every artifact.
/// * `memray attach` (gdb injection below Python 3.14, `sys.remote_exec` on 3.14), `heaptrack
///   --pid` and `grf attach` inject through the debugger interface: tier 3. They capture
///   allocation stacks and sizes, not contents.
/// * `bpftrace` loads eBPF programs on the memory-mapping tracepoints (kernel level, root):
///   tier 3. It counts mapped bytes per call stack and reads no memory contents.
pub fn tier_for(backend: Backend, op: Op) -> TierNeed {
    let (tier, why) = match (backend, op) {
        (Backend::Jcmd, Op::HeapDump) => (4, "a heap dump captures memory contents"),
        (Backend::NodeInspector, Op::HeapSnapshot) => {
            (4, "a heap snapshot captures memory contents")
        }
        (Backend::Jcmd, _) => (2, "jcmd attaches to a running JVM"),
        (Backend::NodeInspector, _) => (2, "the Node inspector runs a script in a running process"),
        (Backend::Memray, _) => (
            3,
            "memray injects its tracker through a debugger or remote exec",
        ),
        (Backend::Heaptrack, _) => (3, "heaptrack injects a library through a debugger (ptrace)"),
        (Backend::Goref, _) => (3, "goref reads the process through a debugger (ptrace)"),
        (Backend::Bpftrace, _) => (3, "bpftrace loads eBPF programs into the kernel (root)"),
    };
    TierNeed { tier, why }
}

/// Everything [`plan`] needs to know about the request.
#[derive(Debug, Clone)]
pub struct Request {
    /// The target.
    pub target: TargetInfo,
    /// Backend.
    pub backend: Backend,
    /// Operation.
    pub op: Op,
    /// How long recording operations run.
    pub duration: Duration,
    /// `--out`: the name of the main artifact.
    pub out_name: Option<String>,
    /// The directory holding the per-attach directories (`<run>/attach`); it must exist.
    pub dump_root: PathBuf,
    /// Extra directories searched for tools before `PATH` (the plugin tools directory).
    pub tool_dirs: Vec<PathBuf>,
}

/// What a backend intends to do, computed without changing the target.
pub struct Plan {
    /// Backend.
    pub backend: Backend,
    /// Operation.
    pub op: Op,
    /// The tool that will run.
    pub tool: String,
    /// What the attach should produce.
    pub expected_signal: String,
    /// When it ends.
    pub stopping: String,
    /// What it does to the target and the box.
    pub effects: Vec<String>,
    /// What is left behind (human text).
    pub cleanup: Vec<String>,
    /// Size estimates for the gate.
    pub estimate: Estimate,
    /// Facts the caller should show (for example the Python version).
    pub notes: Vec<String>,
    /// Set when the backend could not be executed where it was written.
    pub unverified: bool,
    pub(crate) file: String,
    pub(crate) detail: Detail,
}

impl std::fmt::Debug for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plan")
            .field("backend", &self.backend)
            .field("op", &self.op)
            .field("tool", &self.tool)
            .finish()
    }
}

pub(crate) enum Detail {
    Jcmd(jcmd::Prep),
    Node(Box<node::Prep>),
    Memray(memray::Prep),
    Heaptrack(heaptrack::Prep),
    Goref(goref::Prep),
    Bpftrace(bpftrace::Prep),
}

/// Reads what is needed to plan the attach; changes nothing on the target (the Node
/// inspector may have to be opened with `SIGUSR1`, which the plan lists as an effect).
pub fn plan(req: &Request) -> Result<Plan, AttachError> {
    match req.backend {
        Backend::Jcmd => jcmd::plan(req),
        Backend::NodeInspector => node::plan(req),
        Backend::Memray => memray::plan(req),
        Backend::Heaptrack => heaptrack::plan(req),
        Backend::Goref => goref::plan(req),
        Backend::Bpftrace => bpftrace::plan(req),
    }
}

/// What a finished attach produced.
#[derive(Debug, Clone, Default)]
pub struct Done {
    /// Artifact file names inside the attach directory, main artifact first.
    pub files: Vec<String>,
    /// A bounded, backend-specific summary.
    pub summary: Value,
    /// Limits of the data.
    pub limitations: Vec<String>,
    /// Timings in milliseconds (`total`, plus backend phases such as `dump`).
    pub durations: Map<String, Value>,
}

/// The end of an attach, successful or not, with the cleanup state either way.
#[derive(Debug)]
pub struct Finished {
    /// Result of the operation.
    pub result: Result<Done, AttachError>,
    /// Cleanup the backend carried out itself.
    pub cleanup_done: Vec<String>,
    /// Cleanup that remains (notes, and actions that failed).
    pub cleanup_left: Vec<Cleanup>,
}

/// Collects the cleanup actions as a backend creates the need for them.
pub struct Tracker<'a> {
    sink: &'a mut dyn FnMut(&Cleanup),
    left: Vec<Cleanup>,
    done: Vec<String>,
}

impl<'a> Tracker<'a> {
    fn new(sink: &'a mut dyn FnMut(&Cleanup)) -> Self {
        Self {
            sink,
            left: Vec::new(),
            done: Vec::new(),
        }
    }

    /// Records that `c` must run if the attach is aborted; the sink registers it.
    pub(crate) fn need(&mut self, c: Cleanup) {
        (self.sink)(&c);
        self.left.push(c);
    }

    /// Records a note that stays after the attach.
    pub(crate) fn note(&mut self, text: impl Into<String>) {
        self.left.push(Cleanup::Note(text.into()));
    }

    /// Something was cleaned up that never needed registering.
    pub(crate) fn did(&mut self, what: impl Into<String>) {
        self.done.push(what.into());
    }

    /// `c` is no longer needed because the step it guarded finished: it leaves the list
    /// without being reported as carried out.
    pub(crate) fn keep(&mut self, c: &Cleanup) {
        self.left.retain(|l| l != c);
    }

    /// The target is no longer the process that was inspected (it exited and the pid may be
    /// reused): the commands that address it by pid must never run, so they leave the list.
    pub(crate) fn forget_runs(&mut self) {
        self.left.retain(|l| !matches!(l, Cleanup::Run(_)));
    }

    /// `c` was carried out: it moves from the left list to the done list.
    pub(crate) fn done(&mut self, c: &Cleanup, what: impl Into<String>) {
        self.left.retain(|l| l != c);
        self.done.push(what.into());
    }
}

/// Carries out the attach in `out_dir` (the per-attach directory, which exists). `register` is
/// called with each cleanup action as soon as it becomes necessary, so an abort in the middle
/// can run it.
pub fn run(
    plan: Plan,
    req: &Request,
    out_dir: &Path,
    register: &mut dyn FnMut(&Cleanup),
) -> Finished {
    let mut t = Tracker::new(register);
    let started = std::time::Instant::now();
    // The plan may be old (the gate, the consent prompt): the pid must still be the process
    // that was inspected before anything is sent to it.
    let result = if !util::same_process(&req.target) {
        Err(AttachError::not_ready(
            format!(
                "process {} exited or was replaced before the attach started",
                req.target.pid
            ),
            "the process that was inspected, still running",
            "repeat the command on the running process",
        ))
    } else {
        run_backend(&plan, req, out_dir, &mut t)
    };
    finish_run(result, started, t)
}

fn run_backend(
    plan: &Plan,
    req: &Request,
    out_dir: &Path,
    t: &mut Tracker<'_>,
) -> Result<Done, AttachError> {
    match plan.detail {
        Detail::Jcmd(ref p) => jcmd::run(p, plan, req, out_dir, t),
        Detail::Node(ref p) => node::run(p, plan, req, out_dir, t),
        Detail::Memray(ref p) => memray::run(p, plan, req, out_dir, t),
        Detail::Heaptrack(ref p) => heaptrack::run(p, plan, req, out_dir, t),
        Detail::Goref(ref p) => goref::run(p, plan, req, out_dir, t),
        Detail::Bpftrace(ref p) => bpftrace::run(p, plan, req, out_dir, t),
    }
}

fn finish_run(
    result: Result<Done, AttachError>,
    started: std::time::Instant,
    mut t: Tracker<'_>,
) -> Finished {
    // A failed attach removes the partial files it registered for removal.
    if result.is_err() {
        let rms: Vec<Cleanup> = t
            .left
            .iter()
            .filter(|c| matches!(c, Cleanup::Rm(_)))
            .cloned()
            .collect();
        for c in rms {
            if let Cleanup::Rm(p) = &c
                && (std::fs::remove_file(p).is_ok() || !p.exists())
            {
                t.done(&c, format!("removed {}", p.display()));
            }
        }
    }
    let result = result.map(|mut d| {
        d.durations.insert(
            "total_ms".into(),
            json!(started.elapsed().as_millis() as u64),
        );
        d
    });
    Finished {
        result,
        cleanup_done: t.done,
        cleanup_left: t.left,
    }
}

/// The target section of a preflight.
#[derive(Debug, Clone, Serialize)]
pub struct PreflightTarget {
    /// Process ID.
    pub pid: u32,
    /// Executable.
    pub exe: Option<String>,
    /// User ID.
    pub uid: Option<u32>,
    /// Start time.
    pub start_time: Option<String>,
}

/// Free disk figures.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DiskFigures {
    /// Free bytes under the run directory.
    pub run_dir_free: Option<u64>,
    /// Free bytes under the dump directory.
    pub dump_dir_free: Option<u64>,
}

/// What is computed before anything touches the target.
#[derive(Debug, Clone, Serialize)]
pub struct Preflight {
    /// The process.
    pub target: PreflightTarget,
    /// Backend name.
    pub backend: String,
    /// Operation name.
    pub op: String,
    /// Tool used.
    pub tool: String,
    /// Expected signal.
    pub expected_signal: String,
    /// Stopping condition.
    pub stopping: String,
    /// Effects.
    pub effects: Vec<String>,
    /// Cleanup.
    pub cleanup: Vec<String>,
    /// Free disk.
    pub disk: DiskFigures,
    /// Memory.
    pub memory: MemoryAvailable,
    /// Estimate.
    pub estimate: Estimate,
    /// Consent tier and why.
    pub tier: TierNeed,
    /// Notes from the plan.
    pub notes: Vec<String>,
    /// `ok` or `refused`.
    pub decision: String,
    /// Why it was refused.
    pub reasons: Vec<String>,
    /// True when the backend was written from documentation and never executed.
    pub unverified: bool,
}

/// Builds the preflight and applies the gate: dump x1.2 against the smaller free disk, peak
/// against 0.8 x the available memory.
pub fn preflight(
    target: &TargetInfo,
    plan: &Plan,
    disk: DiskFigures,
    memory: MemoryAvailable,
) -> Preflight {
    let free = match (disk.run_dir_free, disk.dump_dir_free) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    };
    let reasons = gate(&plan.estimate, free, memory.effective());
    let mut notes = plan.notes.clone();
    if free.is_none() {
        notes.push("free disk could not be read, so the disk rule was not applied".into());
    }
    if memory.effective().is_none() {
        notes.push("available memory could not be read, so the memory rule was not applied".into());
    }
    Preflight {
        target: PreflightTarget {
            pid: target.pid,
            exe: target.exe.clone(),
            uid: target.uid,
            start_time: target.start_time.clone(),
        },
        backend: plan.backend.name().to_owned(),
        op: plan.op.name().to_owned(),
        tool: plan.tool.clone(),
        expected_signal: plan.expected_signal.clone(),
        stopping: plan.stopping.clone(),
        effects: plan.effects.clone(),
        cleanup: plan.cleanup.clone(),
        disk,
        memory,
        estimate: plan.estimate.clone(),
        tier: tier_for(plan.backend, plan.op),
        notes,
        decision: if reasons.is_empty() { "ok" } else { "refused" }.to_owned(),
        reasons,
        unverified: plan.unverified,
    }
}

impl Preflight {
    /// A short form for the command output and the evidence (a few hundred bytes).
    pub fn compact(&self) -> Value {
        let mut v = json!({
            "pid": self.target.pid,
            "exe": self.target.exe.as_deref().map(|e| util::clip(e, 80)),
            "uid": self.target.uid,
            "signal": util::clip(&self.expected_signal, 120),
            "stop": util::clip(&self.stopping, 100),
            "est_dump": self.estimate.dump_bytes,
            "est_peak": self.estimate.peak_bytes,
            "free_disk": match (self.disk.run_dir_free, self.disk.dump_dir_free) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            },
            "mem_available": self.memory.effective(),
            "tier": self.tier.tier,
            "decision": self.decision,
        });
        if self.unverified {
            v["unverified"] = Value::Bool(true);
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backends_and_ops_resolve_with_helpful_errors() {
        assert_eq!(
            resolve("jcmd", "heap-dump").unwrap(),
            (Backend::Jcmd, Op::HeapDump)
        );
        assert_eq!(
            resolve("node-inspector", "memory").unwrap(),
            (Backend::NodeInspector, Op::Memory)
        );
        let e = resolve("gdb", "x").unwrap_err();
        assert!(
            e.contains("jcmd, node-inspector, memray, heaptrack, goref"),
            "{e}"
        );
        let e = resolve("jcmd", "record").unwrap_err();
        assert!(
            e.contains("histogram, heap-dump, thread-dump, nmt, jfr"),
            "{e}"
        );
    }

    #[test]
    fn every_op_of_every_backend_has_a_name_that_resolves_back() {
        for b in Backend::ALL {
            for &o in b.ops() {
                assert_eq!(resolve(b.name(), o.name()).unwrap(), (b, o));
            }
        }
    }

    #[test]
    fn tiers_follow_the_spec() {
        let t = |b, o| tier_for(b, o).tier;
        assert_eq!(t(Backend::Jcmd, Op::Histogram), 2);
        assert_eq!(t(Backend::Jcmd, Op::ThreadDump), 2);
        assert_eq!(t(Backend::Jcmd, Op::Nmt), 2);
        assert_eq!(t(Backend::Jcmd, Op::Jfr), 2);
        assert_eq!(t(Backend::Jcmd, Op::HeapDump), 4);
        assert_eq!(t(Backend::NodeInspector, Op::Memory), 2);
        assert_eq!(t(Backend::NodeInspector, Op::HeapSnapshot), 4);
        assert_eq!(t(Backend::Memray, Op::Record), 3);
        assert_eq!(t(Backend::Heaptrack, Op::Record), 3);
        assert_eq!(t(Backend::Goref, Op::HeapProfile), 3);
        assert_eq!(t(Backend::Bpftrace, Op::Record), 3);
    }
}
