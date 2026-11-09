//! `dla summarize <op> [--backend <name>|auto] [--finding F] <inputs...>`: captured files
//! into bounded evidence.
//!
//! # Shape
//!
//! Four operations (heap-graph, type-histogram-diff, alloc-stack-diff, non-heap) sit
//! behind one [`Adapter`] trait. An adapter reads files of one format (DLA's own parser,
//! or an external tool whose output it parses) and returns an [`AdapterOut`]. Everything
//! around it is shared:
//!
//! * [`input`]: resolving files (project-relative, absolute, or under `.dla/<run>/`);
//! * [`emit`]: the bounded result, the `raw` pointer with its sha256, the two-signals
//!   record, and ids for `histograms/`, `allocs/` and `nonheap/`;
//! * [`redact`]: values (labels, env values, URL queries) become shapes unless the run
//!   has raw-values consent;
//! * [`tie`] and [`growth`]: growth columns tied to findings, so the ledger accepts a
//!   confirmation that cites them;
//! * the registry below, `--backend` and `auto` selection, and a coverage row per
//!   adapter tool.
//!
//! # Adding an adapter
//!
//! Write one file `summarize/<name>.rs` with a unit struct that implements [`Adapter`]:
//! parse the inputs into the shared models (`dla_parsers::model`) and hand them to the
//! operation's engine ([`histogram::run_diff`], [`alloc_stack::run_diff`],
//! [`non_heap::run_diff`]). Then add one line to [`registry`] and one conformance
//! fixture under `tests/fixtures/summarize/<name>/` (the conformance test fails without
//! it).

pub mod alloc_json;
pub mod alloc_stack;
pub mod compat;
pub mod emit;
pub mod exec;
pub mod growth;
pub mod harness_types;
pub mod histogram;
pub mod hprof;
pub mod hprof_histogram;
pub mod input;
pub mod jcmd_histogram;
pub mod non_heap;
pub mod nonheap_json;
pub mod redact;
pub mod rule;
pub mod tie;

use std::{
    cell::{RefCell, RefMut},
    ffi::OsString,
    path::PathBuf,
    time::Duration,
};

use dla_core::{
    CoverageManifest, FindingId, Ledger, Refusal, RunState,
    coverage::BackendCoverage,
    error::codes as core_codes,
    evidence::{Authority, EvidenceKind, Signal},
    finding::{Holder, Region},
    ids::EvidenceId,
    state::EntryPath,
    toolctx::{EvidenceSpec, ToolCtx},
};
use serde_json::{Map, Value, json};

use crate::fail::{Failure, Res};

/// Refusal code: an explicit `--backend` needs a tool that is not installed.
pub const TOOL_MISSING: &str = "tool-missing";

/// The four operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    /// Retention paths and holder growth from heap dumps.
    HeapGraph,
    /// Type or class histograms across captures.
    TypeHistogramDiff,
    /// Allocation stacks across captures.
    AllocStackDiff,
    /// Threads, descriptors, loaders and native memory.
    NonHeap,
}

impl Op {
    /// All operations, in the order the help lists them.
    pub const ALL: [Op; 4] = [
        Op::HeapGraph,
        Op::TypeHistogramDiff,
        Op::AllocStackDiff,
        Op::NonHeap,
    ];

    /// The name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Self::HeapGraph => "heap-graph",
            Self::TypeHistogramDiff => "type-histogram-diff",
            Self::AllocStackDiff => "alloc-stack-diff",
            Self::NonHeap => "non-heap",
        }
    }

    /// The `command` field of the output.
    pub fn command(self) -> String {
        format!("summarize {}", self.name())
    }
}

/// An external tool an adapter runs, found through the probe registry (tools directory,
/// `JAVA_HOME/bin`, `PATH`).
#[derive(Debug, Clone, Copy)]
pub struct ToolReq {
    /// The tool's registry name, which is also its executable name (`jfr`,
    /// `heaptrack_print`, `valgrind`).
    pub binary: &'static str,
    /// How to get it. Empty: the registry's own hint.
    pub install_hint: &'static str,
}

/// What an adapter reads and how the run is set up.
pub struct Opts {
    /// `--finding`: the finding the result is for.
    pub finding: Option<FindingId>,
    /// `--max-objects` (heap-graph).
    pub max_objects: usize,
    /// `--top`: rows shown in a ranking.
    pub top: usize,
}

/// Run context handed to an adapter.
pub struct Ctx<'a> {
    /// The tool context (project, run directory, worker).
    pub tc: &'a ToolCtx,
    /// Value redaction for this run.
    pub redactor: redact::Redactor,
    /// True when the run starts from captured artifacts: there is no source, so no site.
    pub artifact_run: bool,
    /// Where tools are looked for (`PATH`, `JAVA_HOME`); read by [`Ctx::run_tool`].
    #[allow(dead_code)]
    pub env: Env,
    ledger: RefCell<Option<Ledger>>,
    resolver: dla_core::attribution::Resolver,
}

impl<'a> Ctx<'a> {
    /// Builds the context of a run.
    pub fn new(tc: &'a ToolCtx, env: &Env) -> Result<Self, Refusal> {
        let state = RunState::load(&tc.run_dir).map_err(|e| e.to_refusal())?;
        Ok(Self {
            tc,
            redactor: redact::Redactor::from_state(&state),
            artifact_run: state.entry == EntryPath::Artifact,
            env: env.clone(),
            ledger: RefCell::new(None),
            resolver: dla_core::attribution::Resolver::new(&tc.run_dir, &tc.project),
        })
    }

    /// The run's ledger, loaded on first use. One instance serves the whole command, so
    /// ids stay consistent between the adapter and the framework.
    pub fn ledger(&self) -> Result<RefMut<'_, Ledger>, Refusal> {
        let mut slot = self.ledger.borrow_mut();
        if slot.is_none() {
            *slot = Some(self.tc.ledger()?);
        }
        Ok(RefMut::map(slot, |l| l.as_mut().expect("loaded above")))
    }

    /// Owner-to-site resolution for this run. Artifact runs hold no source: the
    /// resolver is for repository runs.
    pub fn resolver(&self) -> &dla_core::attribution::Resolver {
        &self.resolver
    }

    /// The authority a file-based observation has: files on an artifact run are `dump`
    /// (the process is gone), else `base`.
    pub fn authority(&self, base: Authority) -> Authority {
        if self.artifact_run {
            Authority::Dump
        } else {
            base
        }
    }

    /// Records one evidence: the `signal` goes into the result, `raw` is the full
    /// output, and the result is checked against the bound.
    pub fn record(
        &self,
        ledger: &mut Ledger,
        spec: EvidenceSpec,
        signal: &Signal,
        raw: &emit::RawRef,
    ) -> Result<Recorded, Refusal> {
        emit::record(self.tc, ledger, spec, signal, raw)
    }
}

/// One evidence record an adapter wrote.
#[derive(Debug, Clone)]
pub struct Recorded {
    /// Its id.
    pub id: EvidenceId,
    /// Its kind.
    #[allow(dead_code)]
    pub kind: EvidenceKind,
    /// How it was obtained.
    #[allow(dead_code)]
    pub authority: Authority,
}

/// A candidate finding an adapter added or tied to.
#[derive(Debug, Clone)]
pub struct CandidateRow {
    /// The finding.
    pub finding: FindingId,
    /// Its owner, when known.
    pub owner: Option<String>,
    /// What holds the memory.
    pub holder: Option<Holder>,
    /// `file:line`, when the run has source and the owner resolved.
    pub site: Option<String>,
    /// Memory region.
    pub region: Option<Region>,
    /// How the finding was raised (`growth-series`).
    #[allow(dead_code)]
    pub origin: String,
    /// True when this call added the finding; false when it tied to an existing one.
    pub added: bool,
    /// How the site was found (`census`, `source-scan`).
    pub via: Option<&'static str>,
}

impl CandidateRow {
    /// The compact JSON form used in output.
    pub fn to_json(&self) -> Value {
        let mut v = json!({"finding": self.finding});
        if let Some(o) = &self.owner {
            v["owner"] = json!(o);
        }
        if let Some(h) = self.holder {
            v["holder"] = json!(h);
        }
        if let Some(s) = &self.site {
            v["site"] = json!(s);
        }
        if let Some(r) = self.region {
            v["region"] = json!(r);
        }
        if let Some(via) = self.via {
            v["via"] = json!(via);
        }
        if !self.added {
            v["tied"] = json!(true);
        }
        v
    }
}

/// A coverage row to write: worker (runtime), backend name and what happened.
#[derive(Debug, Clone)]
pub struct CoverageRow {
    /// Worker key: the runtime name.
    pub worker: String,
    /// Backend name.
    pub backend: String,
    /// Ran, missing or blocked.
    pub cov: BackendCoverage,
}

/// What an adapter returns.
#[derive(Debug, Default)]
pub struct AdapterOut {
    /// Evidence it wrote.
    pub evidence: Vec<Recorded>,
    /// Candidates it added or tied to.
    pub candidates: Vec<CandidateRow>,
    /// Extra coverage rows (the framework adds the adapter's own row).
    pub coverage: Vec<CoverageRow>,
    /// Keys of the command's JSON output beyond `ok`, `command`, `backend`, `evidence`
    /// and `raw`.
    pub body: Map<String, Value>,
    /// The main full output.
    pub raw: Option<emit::RawRef>,
    /// Items the tool enumerated (for the coverage row).
    pub enumerated: u64,
}

/// A tool-backed or native reader of one format for one operation.
pub trait Adapter: Sync {
    /// Name used with `--backend` and in coverage (`dla-hprof`, `jcmd-class-histogram`).
    fn name(&self) -> &'static str;
    /// The operation it serves.
    fn op(&self) -> Op;
    /// The runtime it serves: `jvm`, `rust`, `go`, `node`, `python`, or `any`.
    fn runtime(&self) -> &'static str;
    /// The external tool it runs, if any. DLA's own parsers return `None`.
    fn tool(&self) -> Option<ToolReq> {
        None
    }
    /// The tool class recorded in the evidence `signal` (see
    /// `dla_core::evidence::tool_class`).
    fn tool_class(&self) -> &'static str;
    /// What the adapter reads, for a refusal that lists supported formats.
    fn formats(&self) -> &'static str;
    /// True when the input looks like this adapter's format (magic, header, extension).
    fn sniff(&self, input: &input::Input) -> bool;
    /// Reads the inputs (already resolved and sniffed) and writes evidence.
    fn run(&self, ctx: &Ctx<'_>, inputs: &[input::Input], opts: &Opts) -> Res<AdapterOut>;
}

/// Every adapter, DLA's own parsers before external ones within an operation. Adding an
/// adapter is one line here.
pub fn registry() -> &'static [&'static dyn Adapter] {
    static ALL: &[&dyn Adapter] = &[
        &hprof::DlaHprof,
        &hprof_histogram::HprofHistogram,
        &jcmd_histogram::JcmdClassHistogram,
        &harness_types::HarnessTypes,
        &alloc_json::AllocJson,
        &nonheap_json::NonHeapJson,
    ];
    ALL
}

/// Whether an adapter's tool can run here.
#[derive(Debug, Clone)]
pub enum Availability {
    /// No tool needed, or the tool was found.
    Ready {
        /// Version line, when asked for and known.
        version: Option<String>,
    },
    /// The tool is not installed.
    Missing {
        /// What is missing.
        reason: String,
        /// How to get it.
        hint: String,
    },
}

/// The environment variables a tool lookup reads, so a test can pass its own.
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// `PATH`.
    pub path: Option<OsString>,
    /// `JAVA_HOME`.
    pub java_home: Option<OsString>,
}

impl Env {
    /// The process environment.
    pub fn current() -> Self {
        Self {
            path: std::env::var_os("PATH"),
            java_home: std::env::var_os("JAVA_HOME"),
        }
    }
}

/// Checks `a`'s tool in `env` and the tools tree `tools`, asking for its version when `with_version`.
pub fn availability(
    a: &dyn Adapter,
    tools: Option<&std::path::Path>,
    env: &Env,
    with_version: bool,
) -> Availability {
    let Some(req) = a.tool() else {
        return Availability::Ready { version: None };
    };
    let Some(tool) = dla_probe::Backend::dynamic(req.binary) else {
        return Availability::Missing {
            reason: format!("{} is not a tool DLA knows", req.binary),
            hint: req.install_hint.to_owned(),
        };
    };
    let r = dla_probe::Backend::resolve_dynamic(
        tool,
        tools,
        env.java_home.as_deref(),
        env.path.as_deref(),
        with_version,
        Duration::from_secs(5),
    );
    match r.found {
        Some(_) => Availability::Ready { version: r.version },
        None => Availability::Missing {
            reason: format!(
                "{} was not found in the tools directory, JAVA_HOME or PATH",
                req.binary
            ),
            hint: if req.install_hint.is_empty() {
                tool.install_hint.to_owned()
            } else {
                req.install_hint.to_owned()
            },
        },
    }
}

/// The coverage worker key of an adapter: its runtime, or the worker name for a generic
/// adapter.
pub fn worker_key(a: &dyn Adapter, tc: &ToolCtx) -> String {
    match a.runtime() {
        "any" => tc.worker.clone().unwrap_or_else(|| "dla".to_owned()),
        r => r.to_owned(),
    }
}

/// Writes coverage rows (never downgrading a backend that ran).
pub fn write_coverage(tc: &ToolCtx, rows: Vec<CoverageRow>) -> Result<(), Refusal> {
    if rows.is_empty() {
        return Ok(());
    }
    CoverageManifest::update(&tc.run_dir, |m| {
        for r in rows {
            m.note_backend(&r.worker, &r.backend, r.cov);
        }
    })
    .map(|_| ())
    .map_err(|e| e.to_refusal())
}

/// The `--list` table: one row per adapter.
pub fn list(tools: Option<&std::path::Path>, env: &Env) -> Value {
    let rows: Vec<Value> = registry()
        .iter()
        .map(|a| {
            let (tool, available) = match a.tool() {
                None => (Value::Null, true),
                Some(t) => (
                    json!(t.binary),
                    matches!(
                        availability(*a, tools, env, false),
                        Availability::Ready { .. }
                    ),
                ),
            };
            json!({
                "op": a.op().name(),
                "runtime": a.runtime(),
                "backend": a.name(),
                "tool": tool,
                "available": available,
            })
        })
        .collect();
    json!({"ok": true, "command": "summarize --list", "adapters": rows})
}

/// The adapters of one operation, in registry order.
pub fn adapters_of(op: Op) -> Vec<&'static dyn Adapter> {
    registry()
        .iter()
        .copied()
        .filter(|a| a.op() == op)
        .collect()
}

/// `--backend`: a name or `auto`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    /// Pick by sniffing the inputs.
    Auto,
    /// Use this adapter.
    Named(String),
}

impl std::str::FromStr for Choice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        let t = s.trim();
        if t.is_empty() {
            return Err("empty backend name".to_owned());
        }
        Ok(if t == "auto" {
            Self::Auto
        } else {
            Self::Named(t.to_owned())
        })
    }
}

fn names(list: &[&dyn Adapter]) -> String {
    list.iter().map(|a| a.name()).collect::<Vec<_>>().join(", ")
}

/// What `select` decided.
pub struct Selected {
    /// The adapter to run.
    pub adapter: &'static dyn Adapter,
    /// Coverage rows for adapters skipped because their tool is missing.
    pub skipped: Vec<CoverageRow>,
}

/// Why `select` found nothing to run.
pub enum NoAdapter {
    /// Eligible adapters exist but none of their tools is installed; the rows record
    /// the gap and the command still succeeds.
    ToolsMissing(Vec<CoverageRow>),
}

/// Picks the adapter for `op` and the inputs. An explicit name must exist, must sniff
/// every input and must have its tool (else `tool-missing`, exit 2, after a coverage
/// row). `auto` takes the first adapter, DLA's own parsers first, whose sniffer accepts
/// every input and whose tool is present; adapters skipped for a missing tool leave a
/// `missing` row.
pub fn select_in(
    all: Vec<&'static dyn Adapter>,
    tc: &ToolCtx,
    op: Op,
    choice: &Choice,
    inputs: &[input::Input],
    env: &Env,
) -> Res<Result<Selected, NoAdapter>> {
    let all_sniff = |a: &dyn Adapter| inputs.iter().all(|i| a.sniff(i));
    let unsupported = |why: &str| {
        let supported: Vec<String> = all.iter().map(|a| a.formats().to_owned()).collect();
        Failure::refuse(
            "unsupported-format",
            format!("{why} (supported: {})", supported.join("; ")),
            format!("inputs in a format a {} adapter reads", op.name()),
            format!(
                "dla summarize --list   then   dla summarize {} <supported file>",
                op.name()
            ),
        )
    };
    let first_unsniffed = |a: &dyn Adapter| {
        inputs
            .iter()
            .find(|i| !a.sniff(i))
            .map_or_else(String::new, |i| i.shown().to_owned())
    };
    match choice {
        Choice::Named(n) => {
            let Some(a) = all.iter().copied().find(|a| a.name() == n) else {
                return Err(Failure::refuse(
                    core_codes::UNKNOWN_BACKEND,
                    format!(
                        "no {} backend named {n:?}; the backends for it are: {}",
                        op.name(),
                        names(&all)
                    ),
                    format!("--backend one of: auto, {}", names(&all)),
                    format!("dla summarize {} --backend auto <files>", op.name()),
                ));
            };
            if !all_sniff(a) {
                return Err(unsupported(&format!(
                    "{} is not in the format {} reads",
                    first_unsniffed(a),
                    a.name()
                )));
            }
            if let Availability::Missing { reason, hint } =
                availability(a, tc.tools_dir().as_deref(), env, false)
            {
                let row = CoverageRow {
                    worker: worker_key(a, tc),
                    backend: a.name().to_owned(),
                    cov: BackendCoverage::missing(reason.clone(), hint.clone()),
                };
                write_coverage(tc, vec![row])?;
                return Err(Failure::refuse(
                    TOOL_MISSING,
                    format!(
                        "--backend {n} needs {}: {reason}",
                        a.tool().map_or("a tool", |t| t.binary)
                    ),
                    format!(
                        "{} on this machine",
                        a.tool().map_or("the tool", |t| t.binary)
                    ),
                    format!(
                        "{hint}   then   dla summarize {} --backend {n} <files>",
                        op.name()
                    ),
                ));
            }
            Ok(Ok(Selected {
                adapter: a,
                skipped: Vec::new(),
            }))
        }
        Choice::Auto => {
            let eligible: Vec<&'static dyn Adapter> =
                all.iter().copied().filter(|a| all_sniff(*a)).collect();
            if eligible.is_empty() {
                let who = inputs
                    .iter()
                    .find(|i| !all.iter().any(|a| a.sniff(i)))
                    .or(inputs.first())
                    .map_or_else(String::new, |i| i.shown().to_owned());
                return Err(unsupported(&format!(
                    "{who} is not in a format any {} backend reads",
                    op.name()
                )));
            }
            let mut skipped = Vec::new();
            for a in eligible {
                match availability(a, tc.tools_dir().as_deref(), env, false) {
                    Availability::Ready { .. } => {
                        return Ok(Ok(Selected {
                            adapter: a,
                            skipped,
                        }));
                    }
                    Availability::Missing { reason, hint } => skipped.push(CoverageRow {
                        worker: worker_key(a, tc),
                        backend: a.name().to_owned(),
                        cov: BackendCoverage::missing(reason, hint),
                    }),
                }
            }
            Ok(Err(NoAdapter::ToolsMissing(skipped)))
        }
    }
}

/// Everything `dla summarize <op>` needs from the command line.
pub struct Request {
    /// The operation.
    pub op: Op,
    /// `--backend`.
    pub choice: Choice,
    /// `--finding`.
    pub finding: Option<FindingId>,
    /// Input files as given.
    pub files: Vec<PathBuf>,
    /// `--max-objects`.
    pub max_objects: usize,
    /// `--top`.
    pub top: usize,
}

/// Runs one operation end to end: inputs, adapter choice, the adapter, coverage rows and
/// the command's JSON output.
pub fn execute(tc: &ToolCtx, req: Request, env: &Env) -> Res<Value> {
    execute_in(adapters_of(req.op), tc, req, env)
}

/// [`execute`] over an explicit adapter list (the adapters of `req.op`).
pub fn execute_in(
    all: Vec<&'static dyn Adapter>,
    tc: &ToolCtx,
    req: Request,
    env: &Env,
) -> Res<Value> {
    tc.require_time(&format!("dla {}", req.op.command()))?;
    let inputs = input::resolve_all(tc, &req.files, req.op)?;
    let selected = select_in(all, tc, req.op, &req.choice, &inputs, env)?;
    let selected = match selected {
        Ok(s) => s,
        Err(NoAdapter::ToolsMissing(rows)) => {
            let gaps: Vec<Value> = rows
                .iter()
                .map(|r| {
                    json!({
                        "backend": r.backend,
                        "reason": r.cov.reason,
                        "install_hint": r.cov.install_hint,
                    })
                })
                .collect();
            write_coverage(tc, rows)?;
            return Ok(json!({
                "ok": true,
                "command": req.op.command(),
                "backend": null,
                "evidence": [],
                "coverage_gap": gaps,
                "needed": "install the missing tool (see coverage_gap), then rerun; the run continues without this evidence",
            }));
        }
    };
    let a = selected.adapter;
    let ctx = Ctx::new(tc, env)?;
    let opts = Opts {
        finding: req.finding,
        max_objects: req.max_objects,
        top: req.top,
    };
    let version = match availability(a, tc.tools_dir().as_deref(), env, true) {
        Availability::Ready { version } => version,
        Availability::Missing { .. } => None,
    };
    let result = a.run(&ctx, &inputs, &opts);
    let mut rows = selected.skipped;
    let out = match result {
        Ok(o) => o,
        Err(f) => {
            if a.tool().is_some() {
                rows.push(CoverageRow {
                    worker: worker_key(a, tc),
                    backend: a.name().to_owned(),
                    cov: BackendCoverage::blocked(f.refusal.message.clone()),
                });
                write_coverage(tc, rows)?;
            }
            return Err(f);
        }
    };
    rows.push(CoverageRow {
        worker: worker_key(a, tc),
        backend: a.name().to_owned(),
        cov: BackendCoverage::ran(version, out.enumerated),
    });
    rows.extend(out.coverage.iter().cloned());
    let gaps: Vec<Value> = rows
        .iter()
        .filter(|r| r.cov.status != dla_core::coverage::BackendStatus::Ran)
        .map(|r| {
            json!({
                "backend": r.backend,
                "reason": r.cov.reason,
                "install_hint": r.cov.install_hint,
            })
        })
        .collect();
    write_coverage(tc, rows)?;
    let mut v = Map::new();
    v.insert("ok".into(), json!(true));
    v.insert("command".into(), json!(req.op.command()));
    v.insert("backend".into(), json!(a.name()));
    v.extend(out.body);
    v.insert(
        "evidence".into(),
        json!(
            out.evidence
                .iter()
                .map(|e| e.id.to_string())
                .collect::<Vec<_>>()
        ),
    );
    if let Some(r) = &out.raw {
        v.insert("raw".into(), r.to_json());
    }
    if !gaps.is_empty() {
        v.insert("coverage_gap".into(), json!(gaps));
    }
    Ok(Value::Object(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::{coverage::BackendStatus, state::now_utc};
    use std::fs;

    /// A reader that needs `jfr`, with its own format.
    struct NeedsJfr;
    impl Adapter for NeedsJfr {
        fn name(&self) -> &'static str {
            "fake-jfr"
        }
        fn op(&self) -> Op {
            Op::AllocStackDiff
        }
        fn runtime(&self) -> &'static str {
            "jvm"
        }
        fn tool(&self) -> Option<ToolReq> {
            Some(ToolReq {
                binary: "jfr",
                install_hint: "",
            })
        }
        fn tool_class(&self) -> &'static str {
            "alloc-profiler"
        }
        fn formats(&self) -> &'static str {
            "fake recordings"
        }
        fn sniff(&self, i: &input::Input) -> bool {
            i.has_suffix(".rec")
        }
        fn run(&self, _: &Ctx<'_>, _: &[input::Input], _: &Opts) -> Res<AdapterOut> {
            Ok(AdapterOut::default())
        }
    }

    /// The same format, read without a tool by a parser of our own.
    struct Native;
    impl Adapter for Native {
        fn name(&self) -> &'static str {
            "fake-native"
        }
        fn op(&self) -> Op {
            Op::AllocStackDiff
        }
        fn runtime(&self) -> &'static str {
            "jvm"
        }
        fn tool_class(&self) -> &'static str {
            "alloc-profiler"
        }
        fn formats(&self) -> &'static str {
            "fake recordings, parsed here"
        }
        fn sniff(&self, i: &input::Input) -> bool {
            i.has_suffix(".rec")
        }
        fn run(&self, _: &Ctx<'_>, _: &[input::Input], _: &Opts) -> Res<AdapterOut> {
            Ok(AdapterOut {
                enumerated: 7,
                ..AdapterOut::default()
            })
        }
    }

    fn project() -> (tempfile::TempDir, ToolCtx) {
        let dir = tempfile::tempdir().unwrap();
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
        st.save(&run_dir).unwrap();
        dla_core::Ledger::create(&run_dir).unwrap();
        fs::write(dir.path().join("a.rec"), "x").unwrap();
        let tc = ToolCtx {
            project: dir.path().to_path_buf(),
            run_dir,
            run_id: "r1".into(),
            source_commit: "abc".into(),
            data_flag: None,
            worker: None,
        };
        (dir, tc)
    }

    fn request(choice: Choice) -> Request {
        Request {
            op: Op::AllocStackDiff,
            choice,
            finding: None,
            files: vec![PathBuf::from("a.rec")],
            max_objects: 1000,
            top: 5,
        }
    }

    fn no_tools() -> (tempfile::TempDir, Env) {
        let empty = tempfile::tempdir().unwrap();
        let env = Env {
            path: Some(empty.path().as_os_str().to_owned()),
            java_home: None,
        };
        (empty, env)
    }

    #[test]
    fn an_explicit_backend_without_its_tool_refuses_and_records_a_gap() {
        let (_d, tc) = project();
        let (_e, env) = no_tools();
        let all: Vec<&'static dyn Adapter> = vec![&NeedsJfr, &Native];
        let f = execute_in(all, &tc, request("fake-jfr".parse().unwrap()), &env).unwrap_err();
        assert_eq!(f.exit, 2);
        assert_eq!(f.refusal.code, TOOL_MISSING);
        assert!(f.refusal.fix.contains("JDK"), "{}", f.refusal.fix);
        let cov = CoverageManifest::load(&tc.run_dir).unwrap().unwrap();
        let row = &cov.workers["jvm"].backends["fake-jfr"];
        assert_eq!(row.status, BackendStatus::Missing);
        assert!(row.install_hint.as_deref().unwrap().contains("JDK"));
    }

    #[test]
    fn auto_falls_back_to_the_next_adapter_and_keeps_the_gap() {
        let (_d, tc) = project();
        let (_e, env) = no_tools();
        // The tool-backed adapter is listed first here, to show the fall-back.
        let all: Vec<&'static dyn Adapter> = vec![&NeedsJfr, &Native];
        let v = execute_in(all, &tc, request(Choice::Auto), &env).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["backend"], "fake-native");
        assert_eq!(v["coverage_gap"][0]["backend"], "fake-jfr");
        let cov = CoverageManifest::load(&tc.run_dir).unwrap().unwrap();
        let b = &cov.workers["jvm"].backends;
        assert_eq!(b["fake-jfr"].status, BackendStatus::Missing);
        assert_eq!(b["fake-native"].status, BackendStatus::Ran);
        assert_eq!(b["fake-native"].enumerated, 7);
    }

    #[test]
    fn auto_with_only_missing_tools_still_succeeds_with_a_coverage_row() {
        let (_d, tc) = project();
        let (_e, env) = no_tools();
        let all: Vec<&'static dyn Adapter> = vec![&NeedsJfr];
        let v = execute_in(all, &tc, request(Choice::Auto), &env).unwrap();
        assert_eq!(v["ok"], true);
        assert!(v["backend"].is_null());
        assert_eq!(v["evidence"].as_array().unwrap().len(), 0);
        assert_eq!(v["coverage_gap"][0]["backend"], "fake-jfr");
        assert!(v["coverage_gap"][0]["install_hint"].is_string());
        let cov = CoverageManifest::load(&tc.run_dir).unwrap().unwrap();
        assert_eq!(
            cov.workers["jvm"].backends["fake-jfr"].status,
            BackendStatus::Missing
        );
    }

    #[test]
    fn a_tool_found_through_java_home_counts_as_available() {
        let (_d, tc) = project();
        let jh = tempfile::tempdir().unwrap();
        let bin = jh.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let jfr = bin.join("jfr");
        fs::write(&jfr, "#!/bin/sh\necho jfr 21\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&jfr, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let (_e, mut env) = no_tools();
        env.java_home = Some(jh.path().as_os_str().to_owned());
        let all: Vec<&'static dyn Adapter> = vec![&NeedsJfr, &Native];
        let v = execute_in(all, &tc, request(Choice::Auto), &env).unwrap();
        assert_eq!(v["backend"], "fake-jfr");
        assert!(v.get("coverage_gap").is_none(), "{v}");
        #[cfg(unix)]
        {
            let cov = CoverageManifest::load(&tc.run_dir).unwrap().unwrap();
            let row = &cov.workers["jvm"].backends["fake-jfr"];
            assert_eq!(row.status, BackendStatus::Ran);
            assert_eq!(row.version.as_deref(), Some("jfr 21"));
        }
    }

    #[test]
    fn the_registry_has_unique_names_and_every_tool_is_known_to_the_probe_registry() {
        let mut seen = std::collections::BTreeSet::new();
        for a in registry() {
            assert!(seen.insert(a.name()), "duplicate adapter name {}", a.name());
            if let Some(t) = a.tool() {
                assert!(
                    dla_probe::Backend::dynamic(t.binary).is_some(),
                    "{}: tool {} is not in the probe registry",
                    a.name(),
                    t.binary
                );
            }
        }
        // DLA's own parsers come before tool-backed ones within an operation.
        for op in Op::ALL {
            let v = adapters_of(op);
            let first_tool = v.iter().position(|a| a.tool().is_some());
            let last_native = v.iter().rposition(|a| a.tool().is_none());
            if let (Some(t), Some(n)) = (first_tool, last_native) {
                assert!(
                    n < t,
                    "{}: a native adapter is listed after a tool-backed one",
                    op.name()
                );
            }
        }
    }
}
