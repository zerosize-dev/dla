//! `dla summarize heap-graph` and `type-histogram-diff` over V8 heap snapshots
//! (`.heapsnapshot`, from `v8.writeHeapSnapshot` or the inspector), read with DLA's own
//! reader (`dla_parsers::node`).
//!
//! # heap-graph
//!
//! Each snapshot is summarized once (`summaries/`) into holders: the named references
//! (a module variable, a closure variable, a timer callback or event listener, an object
//! property) that alone keep a growing structure alive. Each is recorded as one
//! `retention-path` evidence. Three or more snapshots of ONE process form a series
//! (`series/S<n>.json`) with a `holder:<key>` column per holder, a `growth-series`
//! evidence that applies the dump-series rule, a `dump-plateau` discriminator per holder
//! of interest, and a candidate finding for a passing holder that no finding owns.
//!
//! A column is tied by the tie rule's rule 1 only: the holder's owner (`src.scheduler.shared`,
//! module path plus name, the form the census uses) equals the finding's owner. The module
//! comes from the snapshot itself: the `script` of the closures that use a context, or of
//! the class constructor. Snapshots that name no such script give a bare name, which ties
//! only as a shorter form of exactly one finding.
//!
//! Snapshots of separate processes are not a series, except the per-point snapshots of one
//! command-mode harness run (listed in its `captures.json`, one process per point at its own
//! n): those form a series over n. Otherwise the harness writes the snapshots of
//! one process into one directory named by its pid, so inputs from different pid
//! directories are refused; the object ids V8 assigns must also grow from one snapshot to
//! the next, and most of the earlier snapshot's ids must survive.

use std::{
    collections::BTreeSet,
    path::Path,
    time::{Duration, Instant},
};

use dla_core::{
    FindingId, Ledger, NewFinding, Origin,
    discriminator::DiscriminatorResult,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, Source, tool_class,
    },
    finding::{Holder as FindingHolder, Region},
    fsutil::{to_pretty_json, write_atomic},
    ids::EvidenceId,
    runtime::Runtime,
    series::{
        Axis, Column, ColumnKind, ColumnScope, Point, SeriesFile, SeriesRegion, SeriesSource,
        SeriesSourceKind, Unit,
    },
    toolctx::EvidenceSpec,
};
use dla_parsers::{
    model::{Histogram, HistogramRow},
    node::{HolderKind, HolderOptions, HolderSummary, NodeError, Options, Snapshot, sniff},
};
use serde::Serialize;
use serde_json::{Map, Value, json};

use super::{
    Adapter, AdapterOut, CandidateRow, Ctx, Op, Opts, Recorded, emit, growth,
    histogram::{self, Capture, Origin as HistOrigin},
    input::Input,
    rule::{self, Observed},
    tie,
};
use crate::fail::{Failure, Res};

/// Tool name in the evidence.
const TOOL: &str = "dla-v8-snapshot";
/// Holders a series tracks (two columns each; a series file holds 64).
const SERIES_HOLDERS: usize = 32;
/// Most `dump-plateau` discriminators one run writes.
const MAX_DISCRIMINATORS: usize = 5;
/// Holders named in a retention-path evidence.
const EVIDENCE_HOLDERS: usize = 3;
/// Passing holders named in a growth-series evidence.
const EVIDENCE_PASSED: usize = 4;
/// New candidate findings one run adds at most.
const MAX_NEW_CANDIDATES: usize = 3;
/// Longest the dominator computation may run per snapshot.
const DOMINATOR_TIME: Duration = Duration::from_secs(300);
/// Longest a name is shown.
const CLIP: usize = 100;

const FORMATS: &str =
    "V8 heap snapshot (.heapsnapshot, from v8.writeHeapSnapshot or the inspector)";

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push_str("...");
        t
    }
}

/// True for a V8 heap snapshot, by name or by its first bytes.
pub fn is_snapshot(input: &Input) -> bool {
    input.has_suffix(".heapsnapshot") || sniff(&input.head)
}

fn read_failure(rel: &str, e: &NodeError) -> Failure {
    match e {
        NodeError::NotThisFormat(m) => Failure::refuse(
            "unsupported-format",
            format!("{rel} is not a V8 heap snapshot: {m}"),
            "a .heapsnapshot file",
            "dla summarize heap-graph <snapshot>.heapsnapshot",
        ),
        NodeError::Malformed(_) | NodeError::Truncated(_) => Failure::refuse(
            "malformed-artifact",
            format!("{rel} cannot be read: {e}"),
            "a complete heap snapshot",
            "take the snapshot again (v8.writeHeapSnapshot, or dla harness run <H> --capture heap) and rerun",
        ),
        NodeError::TooLarge(_) => Failure::refuse(
            "insufficient-resources",
            format!("{rel} cannot be read: {e}"),
            "a snapshot that fits in memory (reading takes about four times its size)",
            "free memory, or take the snapshot of a smaller heap (a shorter run), then rerun",
        ),
        NodeError::Io(io) => Failure::from(dla_core::Error::io(
            rel,
            std::io::Error::new(io.kind(), io.to_string()),
        )),
    }
}

// ---------------------------------------------------------------------------------------
// names
// ---------------------------------------------------------------------------------------

/// `src/a/b.js` as the dotted module path the census uses (`src.a.b`).
fn module_path(rel: &str) -> String {
    let stem = rel.rsplit_once('.').map_or(rel, |(s, _)| s);
    stem.trim_start_matches("./").replace('/', ".")
}

/// The module name of a script path from a snapshot, and whether it is a project file.
/// A project file is named relative to the project; a path of another machine is matched
/// by its longest tail that exists in the project; Node internals and packages keep a
/// name that carries no directory of the machine.
pub fn module_of(script: &str, project: &Path) -> (String, bool) {
    let s = script.replace('\\', "/");
    if s.starts_with("node:") {
        return (s, false);
    }
    if let Ok(rel) = Path::new(&s).strip_prefix(project) {
        let rel = rel.to_string_lossy().replace('\\', "/");
        // A package under the project is a dependency, not the program.
        let in_modules = rel.starts_with("node_modules/") || rel.contains("/node_modules/");
        return (module_path(&rel), !in_modules);
    }
    let parts: Vec<&str> = s
        .split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect();
    // A package of another machine is never a project file, whatever its tail matches.
    if let Some(i) = s.find("node_modules/") {
        return (module_path(&s[i..]), false);
    }
    // A tail of one component matches only a path that is one component: `index.js` of
    // another machine is not the project's root `index.js`.
    for i in 0..parts.len() {
        if parts.len() - i < 2 && parts.len() > 1 {
            break;
        }
        let tail = parts[i..].join("/");
        if project.join(&tail).is_file() {
            return (module_path(&tail), true);
        }
    }
    let base = parts.last().copied().unwrap_or("");
    (module_path(base), false)
}

fn join(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(".")
}

/// True for a plain identifier, a property or variable name of the program.
fn plain_ident(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        && !s.chars().next().is_some_and(|c| c.is_ascii_digit())
        && s.chars().filter(char::is_ascii_digit).count() <= 3
}

/// Classes that say nothing about which finding they belong to.
fn generic_class(c: &str) -> bool {
    matches!(
        c,
        "Object" | "Array" | "Map" | "Set" | "WeakMap" | "WeakSet" | "Promise" | "Function"
    )
}

/// One holder of one snapshot, named for the ledger.
#[derive(Debug, Clone)]
struct View {
    /// Identity across snapshots: `static:src.scheduler.shared`.
    key: String,
    /// `module.name`, the form the census and the tie rule use.
    owner: String,
    holder: FindingHolder,
    retained: u64,
    count: u64,
    /// True when the script is a project file.
    project: bool,
    /// True for Node's own modules (`node:internal/...`).
    internal: bool,
    generic: bool,
}

/// A name from the heap shown without data in it: a property or variable name stays when
/// it is a plain identifier, any other becomes a salted shape (an object used as a
/// dictionary has keys that are ids or users).
fn safe_name(name: &str, red: &super::redact::Redactor) -> String {
    if plain_ident(name) {
        red.name(name)
    } else {
        red.value(name)
    }
}

/// Redacts a holder before anything is written or keyed: its name (a shape unless the
/// name of a variable, a property or a callback is a plain identifier), its owner, its
/// path and its accumulation class. Strings of the heap are never read, so the data that
/// can remain is in names.
fn sanitize(h: &mut dla_parsers::node::Holder, red: &super::redact::Redactor) {
    // The properties of a generic container are its dictionary keys, which are data
    // (V8 does not tell a key from a property name): they are shaped, stable within a run.
    h.name = if h.kind == HolderKind::Instance && generic_class(&h.owner) {
        red.value(&h.name)
    } else if h.kind == HolderKind::Instance {
        safe_name(&h.name, red)
    } else {
        red.name(&h.name)
    };
    h.owner = red.name(&h.owner);
    for hop in &mut h.path {
        let generic = generic_class(&hop.node);
        hop.node = red.name(&hop.node);
        if let Some(n) = hop.edge.strip_prefix('.')
            && hop.edge != "..."
        {
            hop.edge = if generic {
                format!(".{}", red.value(n))
            } else {
                format!(".{}", safe_name(n, red))
            };
        }
    }
    if let Some(a) = &mut h.accumulation {
        a.class = red.name(&a.class);
    }
}

fn view_of(h: &dla_parsers::node::Holder, project: &Path) -> View {
    let (module, in_project) = h
        .script
        .as_deref()
        .map_or((String::new(), false), |s| module_of(s, project));
    let name = h.name.clone();
    let (key, owner, holder) = match (h.kind, h.registry.as_deref()) {
        (HolderKind::Static, _) => {
            let o = join(&[&module, &h.owner, &name]);
            (format!("static:{o}"), o, FindingHolder::StaticGlobal)
        }
        (HolderKind::Closure, _) => {
            let o = join(&[&module, &h.owner, &name]);
            (
                format!("closure:{o}"),
                o,
                FindingHolder::ClosureContinuation,
            )
        }
        (HolderKind::Registration, Some("timers")) => {
            // The census names a timer by the function that starts it.
            let o = if h.owner.is_empty() {
                join(&[&module, &name])
            } else {
                join(&[&module, &h.owner])
            };
            (
                format!("registry:timers:{}", join(&[&module, &h.owner, &name])),
                o,
                FindingHolder::RegistrationTable,
            )
        }
        (HolderKind::Registration, _) => {
            let o = join(&[&module, &h.owner, &name]);
            (
                format!("registry:events:{o}"),
                o,
                FindingHolder::RegistrationTable,
            )
        }
        (HolderKind::Instance, _) => {
            let o = join(&[&module, &h.owner, &name]);
            (format!("field:{o}"), o, FindingHolder::ContainerField)
        }
    };
    View {
        key,
        owner,
        holder,
        retained: h.retained,
        count: h.count,
        project: in_project,
        internal: module.starts_with("node:"),
        generic: generic_class(&h.owner),
    }
}

// ---------------------------------------------------------------------------------------
// loading
// ---------------------------------------------------------------------------------------

/// The pid directory of an input: the harness writes `<capture dir>/<pid>/heap-<n>`.
fn pid_dir(input: &Input) -> Option<String> {
    let name = input.abs.parent()?.file_name()?.to_str()?;
    (!name.is_empty() && name.bytes().all(|b| b.is_ascii_digit())).then(|| name.to_owned())
}

/// What the comparability check keeps of one snapshot.
struct Identity {
    rel: String,
    pid: Option<String>,
    max_id: u32,
    /// Sorted object ids.
    ids: Vec<u32>,
}

/// Orders snapshots by their largest object id and checks they can be one process.
fn order_and_check(ids: &mut [Identity]) -> Result<(), String> {
    ids.sort_by(|a, b| a.max_id.cmp(&b.max_id).then_with(|| a.rel.cmp(&b.rel)));
    for w in ids.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if let (Some(x), Some(y)) = (&a.pid, &b.pid)
            && x != y
        {
            return Err(format!(
                "{} and {} come from different processes (pid directories {x} and {y})",
                a.rel, b.rel
            ));
        }
        if b.max_id <= a.max_id {
            return Err(format!(
                "{} and {} end at the same object id {}: the same snapshot twice, or snapshots of separate processes (ids grow within one process)",
                a.rel, b.rel, a.max_id
            ));
        }
        // How many of the earlier ids the later snapshot still has.
        let (mut i, mut j, mut same) = (0usize, 0usize, 0usize);
        while i < a.ids.len() && j < b.ids.len() {
            match a.ids[i].cmp(&b.ids[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    same += 1;
                    i += 1;
                    j += 1;
                }
            }
        }
        if same * 2 < a.ids.len() {
            return Err(format!(
                "{} keeps only {same} of the {} objects of {}: not the same process",
                b.rel,
                a.ids.len(),
                a.rel
            ));
        }
    }
    Ok(())
}

fn not_comparable(why: &str) -> Failure {
    Failure::refuse(
        "dumps-not-comparable",
        format!("the snapshots do not look like one process: {why}"),
        "snapshots of one running process (what can be checked: one pid directory, object ids that grow, most earlier ids still present)",
        "take the snapshots in driver mode (dla harness run <H> --capture heap) and pass the heap-*.heapsnapshot of one pid directory, or pass the per-point snapshots of one command-mode harness run (listed in its captures.json)",
    )
}

fn load(rel: &str, abs: &Path) -> Res<Snapshot> {
    Snapshot::read_file(abs, &Options::default()).map_err(|e| read_failure(rel, &e))
}

// ---------------------------------------------------------------------------------------
// heap-graph
// ---------------------------------------------------------------------------------------

/// The heap-graph reader.
pub struct NodeHeapGraph;

impl Adapter for NodeHeapGraph {
    fn name(&self) -> &'static str {
        "node-heapsnapshot"
    }
    fn op(&self) -> Op {
        Op::HeapGraph
    }
    fn runtime(&self) -> &'static str {
        "node"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::DUMP_PARSER
    }
    fn formats(&self) -> &'static str {
        FORMATS
    }
    fn sniff(&self, input: &Input) -> bool {
        is_snapshot(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        heap_graph(ctx, inputs, opts)
    }
}

#[derive(Serialize)]
struct SummaryFile<'a> {
    file: &'a str,
    #[serde(flatten)]
    summary: &'a HolderSummary,
}

struct Dump {
    rel: String,
    sha256: String,
    summary: HolderSummary,
    views: Vec<View>,
    summary_file: String,
    /// The iteration count `n` of the point, for a per-point series (separate processes).
    x: Option<f64>,
}

impl Dump {
    fn view(&self, key: &str) -> Option<&View> {
        self.views.iter().find(|v| v.key == key)
    }
}

/// `<stem>-<sha8>.json`.
fn summary_name(rel: &str, sha: &str) -> String {
    let file = rel.rsplit('/').next().unwrap_or(rel);
    let stem = file.strip_suffix(".heapsnapshot").unwrap_or(file);
    format!("{}-{}.json", emit::clean_stem(stem), &sha[..8])
}

fn short_path(rel: &str) -> String {
    if rel.len() <= 48 {
        return rel.to_owned();
    }
    let parts: Vec<&str> = rel.rsplitn(3, '/').collect();
    format!(".../{}/{}", parts.get(1).unwrap_or(&""), parts[0])
}

fn complete(n: u64) -> EvidenceCoverage {
    EvidenceCoverage {
        status: CoverageStatus::Complete,
        examined: Some(n),
        total: Some(n),
    }
}

fn needed(n: usize) -> String {
    let more = rule::MIN_DUMPS.saturating_sub(n);
    format!(
        "{more} more snapshot{} of the same process after more traffic ({} confirm growth; two cannot tell a warm-up fill from a leak); rerun with all snapshots",
        if more == 1 { "" } else { "s" },
        rule::MIN_DUMPS
    )
}

fn top_json(v: &View) -> Value {
    json!({"key": clip(&v.key, CLIP), "retained": v.retained, "dominated": v.count})
}

fn heap_graph(ctx: &Ctx<'_>, inputs: &[Input], a: &Opts) -> Res<AdapterOut> {
    let tc = ctx.tc;
    // ---- read (no writes yet)
    // Harness captures of command mode are one process per point: a per-point series.
    let per_point = super::compat::per_point(&tc.run_dir, inputs).map_err(|why| {
        Failure::refuse(
            "dumps-not-comparable",
            format!("the snapshots are harness captures that do not form one per-point series: {why}"),
            "three or more heap snapshots of one command-mode harness run, one per point, at distinct n",
            "dla summarize heap-graph <the heap-*.heapsnapshot files of one dla harness run --capture heap, one per point>",
        )
    })?;
    let mut dumps: Vec<Dump> = Vec::new();
    let mut idents: Vec<Identity> = Vec::new();
    for (k, i) in inputs.iter().enumerate() {
        let snap = load(&i.rel, &i.abs)?;
        let dom = snap
            .dominators(Some(Instant::now() + DOMINATOR_TIME))
            .map_err(|e| read_failure(&i.rel, &e))?;
        let bfs = snap.bfs();
        let mut summary = snap.holders(&dom, &bfs, &HolderOptions::default());
        for h in &mut summary.holders {
            sanitize(h, &ctx.redactor);
        }
        idents.push(Identity {
            rel: i.rel.clone(),
            pid: pid_dir(i),
            max_id: summary.max_id,
            ids: snap.sorted_ids(),
        });
        drop((dom, bfs, snap));
        let views: Vec<View> = summary
            .holders
            .iter()
            .map(|h| view_of(h, &tc.project))
            .filter(|v| !v.internal)
            .collect();
        // The stored summary names the module, never the path of the machine.
        for h in &mut summary.holders {
            h.script = h.script.take().map(|s| module_of(&s, &tc.project).0);
        }
        dumps.push(Dump {
            rel: i.rel.clone(),
            sha256: i.sha256.clone(),
            summary_file: format!("summaries/{}", summary_name(&i.rel, &i.sha256)),
            summary,
            views,
            x: per_point.as_ref().map(|xs| xs[k]),
        });
        tc.require_time("dla summarize heap-graph")?;
    }
    if per_point.is_some() {
        // Separate processes: object ids are no clock, and the points are ordered by n.
        dumps.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        order_and_check(&mut idents).map_err(|why| not_comparable(&why))?;
        // The dumps in the order of the checked identities.
        dumps.sort_by_key(|d| {
            idents
                .iter()
                .position(|x| x.rel == d.rel)
                .unwrap_or(usize::MAX)
        });
    }
    let mut ledger_ref = ctx.ledger()?;
    let ledger: &mut Ledger = &mut ledger_ref;
    if let Some(f) = &a.finding
        && ledger.finding(f).is_none()
    {
        return Err(Failure::from(dla_core::Refusal::new(
            dla_core::error::codes::UNKNOWN_FINDING,
            format!("no finding {f} in the ledger"),
            "an existing finding",
            "dla ledger show",
        )));
    }

    // ---- per-snapshot summaries and retention-path evidence
    let dir = tc.stage_dir("summaries")?;
    let mut dump_json = Vec::new();
    let mut retention: Vec<EvidenceId> = Vec::new();
    let mut recorded: Vec<Recorded> = Vec::new();
    let mut last_raw: Option<emit::RawRef> = None;
    for d in &dumps {
        let path = dir.join(d.summary_file.trim_start_matches("summaries/"));
        let body = to_pretty_json(&SummaryFile {
            file: &d.rel,
            summary: &d.summary,
        });
        write_atomic(&path, body.as_bytes())?;
        let raw = emit::RawRef {
            rel: d.summary_file.clone(),
            sha256: emit::sha_hex(body.as_bytes()),
        };
        let top: Vec<Value> = d
            .views
            .iter()
            .take(EVIDENCE_HOLDERS)
            .map(top_json)
            .collect();
        let result = json!({
            "file": d.rel,
            "objects": d.summary.reachable,
            "bytes": d.summary.total_bytes,
            "holders": top,
        });
        let mut spec = EvidenceSpec::new(
            EvidenceKind::RetentionPath,
            TOOL,
            env!("CARGO_PKG_VERSION"),
            result,
        );
        spec.authority = Authority::Dump;
        spec.source = Some(Source::Artifact {
            path: d.rel.clone(),
            sha256: d.sha256.clone(),
        });
        spec.coverage = EvidenceCoverage {
            status: CoverageStatus::Complete,
            examined: Some(d.summary.reachable),
            total: Some(d.summary.nodes),
        };
        spec.limitations = d.summary.notes.clone();
        let signal = Signal::new(TOOL, tool_class::DUMP_PARSER, &d.sha256);
        let rec = ctx.record(ledger, spec, &signal, &raw)?;
        dump_json.push(json!({
            "file": short_path(&d.rel),
            "top_holder": d.views.first().map(|v| clip(&v.key, 56)),
            "evidence": rec.id,
        }));
        retention.push(rec.id.clone());
        recorded.push(rec);
        last_raw = Some(raw);
    }
    let mut out = Map::new();
    out.insert("summaries".into(), json!(tc.pointer("summaries/")));
    out.insert("dumps".into(), json!(dump_json));
    out.insert("redaction".into(), json!(ctx.redactor.mode()));
    let n = dumps.len();
    let enumerated = dumps.iter().map(|d| d.summary.reachable).sum();
    if n == 1 {
        out.insert("series".into(), Value::Null);
        out.insert("gate".into(), json!("fail"));
        out.insert("reasons".into(), json!(["snapshots < 3"]));
        out.insert("needed".into(), json!(needed(n)));
        return Ok(AdapterOut {
            evidence: recorded,
            raw: last_raw,
            body: out,
            enumerated,
            ..AdapterOut::default()
        });
    }
    let mut cands: Vec<CandidateRow> = Vec::new();
    let series_raw = series_part(
        ctx,
        ledger,
        a,
        &dumps,
        &retention,
        &mut out,
        &mut recorded,
        &mut cands,
    )?;
    Ok(AdapterOut {
        evidence: recorded,
        candidates: cands,
        raw: Some(series_raw),
        body: out,
        enumerated,
        ..AdapterOut::default()
    })
}

/// The finding a passing holder belongs to, and why.
struct Tie {
    finding: FindingId,
    /// The holder kind the snapshots show.
    holder: FindingHolder,
    also: Vec<FindingId>,
    site: Option<String>,
    why: tie::TieReason,
}

#[allow(clippy::too_many_arguments)]
fn series_part(
    ctx: &Ctx<'_>,
    ledger: &mut Ledger,
    a: &Opts,
    dumps: &[Dump],
    retention: &[EvidenceId],
    out: &mut Map<String, Value>,
    recorded: &mut Vec<Recorded>,
    cands: &mut Vec<CandidateRow>,
) -> Res<emit::RawRef> {
    let tc = ctx.tc;
    let n = dumps.len();
    let per_point: Option<Vec<f64>> = dumps.iter().map(|d| d.x).collect();
    // ---- holders tracked: the largest by peak retained bytes
    let mut peak: Vec<(String, u64)> = Vec::new();
    for d in dumps {
        for v in &d.views {
            match peak.iter_mut().find(|(k, _)| *k == v.key) {
                Some((_, p)) => *p = (*p).max(v.retained),
                None => peak.push((v.key.clone(), v.retained)),
            }
        }
    }
    peak.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
    let keys: Vec<String> = peak
        .into_iter()
        .take(SERIES_HOLDERS)
        .map(|(k, _)| k)
        .collect();
    let observed = |key: &str| Observed {
        bytes: dumps
            .iter()
            .map(|d| d.view(key).map(|v| v.retained))
            .collect(),
        counts: dumps.iter().map(|d| d.view(key).map(|v| v.count)).collect(),
    };
    let latest = |key: &str| dumps.iter().rev().find_map(|d| d.view(key));

    // ---- the dump-series rule
    let mut passed: Vec<(String, rule::Growth)> = Vec::new();
    let mut best_failure: Option<(String, Vec<String>, u64)> = None;
    for key in &keys {
        match rule::gate(&observed(key)) {
            Ok(g) => passed.push((key.clone(), g)),
            Err(why) => {
                let size = latest(key).map_or(0, |v| v.retained);
                if best_failure.as_ref().is_none_or(|b| size > b.2) && n >= rule::MIN_DUMPS {
                    best_failure = Some((key.clone(), why, size));
                }
            }
        }
    }
    passed.sort_by_key(|(k, _)| std::cmp::Reverse(latest(k).map_or(0, |v| v.retained)));

    // ---- ties: a passing holder belongs to the finding whose owner it is
    let latest_retention = retention[retention.len() - 1].clone();
    let who = tie::Who::new(tc.worker.as_deref(), Some(Runtime::Node));
    let holder_owners: Vec<String> = keys
        .iter()
        .filter_map(|k| latest(k))
        .map(|v| v.owner.clone())
        .collect();
    let mut cand_json = Vec::new();
    let mut tie_of: Vec<(String, Tie)> = Vec::new();
    let mut added = 0usize;
    for (key, _) in &passed {
        let Some(v) = latest(key) else { continue };
        let matches: Vec<(FindingId, tie::TieReason)> =
            tie::findings_by_holder(ledger, &who, &v.owner, &holder_owners, None)
                .into_iter()
                .map(|f| (f, tie::TieReason::holder(&v.owner)))
                .collect();
        let r = tie::resolve(matches, a.finding.as_ref());
        let (fid, also, why) = match r.tie {
            Some((f, why)) => (f, r.also, why),
            None => {
                // No finding owns it. With `--finding` nothing is added: the command
                // says what is missing. A project holder that is not a generic container
                // becomes a candidate (no site: a snapshot has no source line).
                if a.finding.is_some()
                    || !r.ambiguous.is_empty()
                    || !v.project
                    || v.generic
                    || added >= MAX_NEW_CANDIDATES
                    || canonical_owner(&v.owner).is_none()
                {
                    continue;
                }
                let mut nf = NewFinding::new(
                    Origin::parse("growth-series").map_err(dla_core::Refusal::from)?,
                    vec![latest_retention.clone()],
                );
                nf.holder = Some(v.holder);
                nf.owner = Some(v.owner.clone());
                nf.region = Some(Region::Heap);
                let line = ledger.add_finding(nf).map_err(|e| e.to_refusal())?;
                let dla_core::Record::FindingAdded { finding } = line.record else {
                    continue;
                };
                added += 1;
                cand_json.push(json!({"finding": finding.id, "owner": clip(&v.owner, CLIP), "holder": v.holder}));
                cands.push(CandidateRow {
                    finding: finding.id.clone(),
                    owner: Some(v.owner.clone()),
                    holder: Some(v.holder),
                    site: None,
                    region: Some(Region::Heap),
                    origin: "growth-series".into(),
                    added: true,
                    via: None,
                });
                (finding.id, Vec::new(), tie::TieReason::holder(&v.owner))
            }
        };
        if !cands.iter().any(|c| c.finding == fid) {
            let f = ledger.finding(&fid);
            cands.push(CandidateRow {
                finding: fid.clone(),
                owner: f.and_then(|x| x.owner.clone()),
                holder: Some(v.holder),
                site: tie::site_of(ledger, &fid),
                region: Some(Region::Heap),
                origin: "growth-series".into(),
                added: false,
                via: None,
            });
        }
        let site = tie::site_of(ledger, &fid);
        tie_of.push((
            key.clone(),
            Tie {
                finding: fid,
                holder: v.holder,
                also,
                site,
                why,
            },
        ));
    }
    let tie_for = |key: &str| tie_of.iter().find(|(k, _)| k == key).map(|(_, t)| t);

    // ---- the series file
    let id = tc.next_series_id();
    let mut columns = Vec::new();
    for key in &keys {
        let t = tie_for(key);
        for (suffix, unit, kind) in [
            ("", Unit::Bytes, ColumnKind::Gauge),
            (".count", Unit::Count, ColumnKind::Counter),
        ] {
            columns.push(Column {
                name: format!("holder:{key}{suffix}"),
                unit,
                kind,
                region: SeriesRegion::Heap,
                baseline: false,
                after_gc: false,
                scope: ColumnScope::Process,
                site: t.and_then(|t| t.site.clone()),
                finding: t.map(|t| t.finding.clone()),
                source: "v8-heapsnapshot".into(),
            });
        }
    }
    let points: Vec<Point> = dumps
        .iter()
        .enumerate()
        .map(|(i, d)| Point {
            x: d.x.unwrap_or((i + 1) as f64),
            t_ms: 0,
            values: keys
                .iter()
                .flat_map(|k| {
                    let v = d.view(k);
                    [v.map(|v| v.retained as f64), v.map(|v| v.count as f64)]
                })
                .collect(),
        })
        .collect();
    let mut source = SeriesSource::new(SeriesSourceKind::Dump);
    source.command = Some("dla summarize heap-graph".into());
    let series = SeriesFile {
        schema_version: 1,
        id: id.clone(),
        source,
        axis: if per_point.is_some() {
            Axis::N
        } else {
            Axis::Dump
        },
        warmup: 0,
        columns,
        points,
        events: Vec::new(),
        limitations: match &per_point {
            Some(xs) => vec![
                format!("series: per-point; x is the iteration count n of each point ({}); every point is a separate process and heap snapshot", super::compat::n_list(xs)),
                "retained bytes follow the dominator tree; a snapshot records no time".into(),
                "holders below 1 KiB retained in a snapshot have no figure at that point".into(),
            ],
            None => vec![
                "x is the snapshot number in object-id order (ids grow within one process); a snapshot records no time".into(),
                "each point comes from a separate heap snapshot of one process; retained bytes follow the dominator tree".into(),
                "holders below 1 KiB retained in a snapshot have no figure at that point".into(),
            ],
        },
    };
    series.save(&tc.run_dir)?;

    // ---- the growth-series evidence
    let gate_pass = !passed.is_empty();
    let mut reasons: Vec<String> = Vec::new();
    if n < rule::MIN_DUMPS {
        reasons.push(format!("snapshots < {}", rule::MIN_DUMPS));
    } else if !gate_pass {
        reasons.push("no holder grows strictly in bytes and count across all snapshots".into());
        if let Some((k, why, _)) = &best_failure {
            reasons.push(format!("largest: {}: {}", clip(k, 60), why.join("; ")));
        }
    }
    // The shown holders: the best of each tied finding first, then the rest by size.
    let mut order: Vec<usize> = Vec::new();
    let mut covered: Vec<&FindingId> = Vec::new();
    for (i, (k, _)) in passed.iter().enumerate() {
        if let Some(t) = tie_for(k)
            && !covered.contains(&&t.finding)
        {
            covered.push(&t.finding);
            order.push(i);
        }
    }
    for i in 0..passed.len() {
        if !order.contains(&i) {
            order.push(i);
        }
    }
    let rows: Vec<Value> = order
        .iter()
        .take(EVIDENCE_PASSED)
        .map(|&i| {
            let (key, g) = &passed[i];
            let slope = g.byte_deltas.iter().sum::<u64>() as f64 / g.byte_deltas.len() as f64;
            let mut v = json!({
                "column": format!("holder:{}", clip(key, CLIP)),
                "region": "heap",
                "unit": "bytes",
                "slope": slope.round() as u64,
                "shape": rule::shape(g),
                "last_over_first": (g.last_over_first * 100.0).round() / 100.0,
            });
            if let Some(t) = tie_for(key) {
                v["finding"] = json!(t.finding);
                let mut c = vec![t.finding.clone()];
                c.extend(t.also.iter().take(2).cloned());
                v["candidates"] = json!(c);
                v["tie"] = t.why.to_json();
                if let Some(s) = &t.site {
                    v["site"] = json!(clip(s, 160));
                }
            }
            v
        })
        .collect();
    let mut base = json!({
        "series": id,
        "axis": if per_point.is_some() { "n" } else { "dump" },
        "n": n,
        "gate": if gate_pass { "pass" } else { "fail" },
        "failed": keys.len() - passed.len(),
        "reasons": reasons,
    });
    if let Some(xs) = &per_point {
        base["series_kind"] = json!("per-point");
        base["n_values"] = json!(xs);
    }
    let (result, kept) = growth::fit_passed(base, rows, passed.len());
    let shown: Vec<usize> = order.iter().take(kept).copied().collect();
    let series_rel = format!("series/{id}.json");
    let mut spec = EvidenceSpec::new(
        EvidenceKind::GrowthSeries,
        TOOL,
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = Authority::Dump;
    let source = tc.artifact_source(&series_rel)?;
    let series_sha = match &source {
        Source::Artifact { sha256, .. } => sha256.clone(),
        _ => String::new(),
    };
    spec.source = Some(source);
    spec.coverage = complete(n as u64);
    spec.limitations = vec![
        format!(
            "dump-series rule: at least {} snapshots, retained bytes and dominated count strictly increasing across every pair, last interval at least {:.0}% of the first (provisional, not calibrated)",
            rule::MIN_DUMPS,
            rule::LAST_OVER_FIRST * 100.0
        ),
        match &per_point {
            Some(xs) => format!("series: per-point; one process per point at n = {} (harness captures; checked: snapshot format)", super::compat::n_list(xs)),
            None => "the snapshots are assumed to come from one process (checked: one pid directory, growing object ids, surviving ids)".into(),
        },
    ];
    let shas: Vec<&str> = dumps.iter().map(|d| d.sha256.as_str()).collect();
    let signal = Signal::new(TOOL, tool_class::DUMP_PARSER, &emit::combined_sha(&shas));
    let series_raw = emit::RawRef {
        rel: series_rel.clone(),
        sha256: series_sha,
    };
    let growth_ev = ctx.record(ledger, spec, &signal, &series_raw)?;
    let growth_id = growth_ev.id.clone();
    recorded.push(growth_ev);

    // ---- discriminators
    let mut interest: Vec<String> = passed.iter().map(|(k, _)| k.clone()).collect();
    let biggest = keys
        .iter()
        .filter_map(|k| latest(k))
        .map(|v| v.retained)
        .max()
        .unwrap_or(0);
    for key in &keys {
        let o = observed(key);
        let moved = o.bytes.iter().flatten().collect::<BTreeSet<_>>().len() > 1;
        let sizable = latest(key).is_some_and(|v| v.retained * 10 >= biggest);
        if moved && sizable && !interest.contains(key) {
            interest.push(key.clone());
        }
    }
    interest.truncate(MAX_DISCRIMINATORS);
    let mut disc_json = Vec::new();
    for key in &interest {
        let Some(p) = rule::plateau(&observed(key)) else {
            continue;
        };
        let finding: Option<String> = tie_for(key).map(|t| t.finding.to_string());
        let mut r = DiscriminatorResult::new("dump-plateau", p.favors);
        r.supports = p.supports.iter().map(|s| (*s).to_owned()).collect();
        r.finding = finding.as_deref().and_then(|f| f.parse().ok());
        r.extra.insert(
            "column".into(),
            json!(format!("holder:{}", clip(key, CLIP))),
        );
        r.extra.insert("series".into(), json!(id));
        r.extra.insert("dumps".into(), json!(n));
        if let Value::Object(m) = p.extra() {
            r.extra.extend(m);
        }
        let mut spec = EvidenceSpec::new(
            EvidenceKind::Discriminator,
            TOOL,
            env!("CARGO_PKG_VERSION"),
            r.to_value(),
        );
        spec.authority = Authority::Dump;
        spec.source = Some(tc.artifact_source(&series_rel)?);
        spec.coverage = complete(n as u64);
        spec.limitations = vec![
            "dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)".into(),
        ];
        let rec = ctx.record(ledger, spec, &signal, &series_raw)?;
        disc_json.push(json!({"holder": clip(key, 48), "evidence": rec.id, "favors": p.favors, "finding": finding}));
        recorded.push(rec);
    }

    out.insert("series".into(), json!(id));
    if let Some(xs) = &per_point {
        out.insert("series_kind".into(), json!("per-point"));
        out.insert("n_values".into(), json!(xs));
    }
    out.insert(
        "gate".into(),
        json!(if gate_pass { "pass" } else { "fail" }),
    );
    out.insert("reasons".into(), json!(reasons));
    out.insert("growth_evidence".into(), json!(growth_id));
    out.insert(
        "passed".into(),
        json!(
            shown
                .iter()
                .map(|&i| clip(&passed[i].0, 60))
                .collect::<Vec<_>>()
        ),
    );
    let mut tied_ids: Vec<&FindingId> = Vec::new();
    for &i in &shown {
        if let Some(t) = tie_for(&passed[i].0)
            && !tied_ids.contains(&&t.finding)
        {
            tied_ids.push(&t.finding);
        }
    }
    out.insert("tied".into(), json!(tied_ids));
    // What the snapshots show about each tied finding's holder: the finding may name
    // another kind (the census calls a setInterval a blocked stack).
    let observed_holders: Map<String, Value> = shown
        .iter()
        .filter_map(|&i| tie_for(&passed[i].0))
        .take(3)
        .map(|t| (t.finding.to_string(), json!(t.holder)))
        .collect();
    out.insert("observed_holder".into(), Value::Object(observed_holders));
    out.insert(
        "candidates".into(),
        json!(
            cand_json
                .iter()
                .take(MAX_NEW_CANDIDATES)
                .collect::<Vec<_>>()
        ),
    );
    // The command output is bounded: the rest of the discriminators are in the ledger.
    out.insert(
        "discriminators".into(),
        json!(disc_json.iter().take(3).collect::<Vec<_>>()),
    );
    if disc_json.len() > 3 {
        out.insert("discriminators_more".into(), json!(disc_json.len() - 3));
    }
    let needed_text = if n < rule::MIN_DUMPS {
        needed(n)
    } else if !gate_pass {
        "growth over every interval: take another snapshot after more traffic, rerun with all snapshots".to_owned()
    } else if let Some(f) = tied_ids.first() {
        let seen = shown.iter().find_map(|&i| {
            tie_for(&passed[i].0)
                .filter(|t| &t.finding == *f)
                .map(|t| t.holder)
        });
        let named = ledger.finding(f).and_then(|x| x.holder);
        holder_needed(f, seen, named, &growth_id)
    } else if let Some(f) = &a.finding {
        format!(
            "no passing column ties to {f} by holder, site or type: a holder ties to the finding whose owner is its module path plus name (for example src.scheduler.shared); annotate {f}'s owner in that form"
        )
    } else {
        "no passing holder is tied in the evidence: dla census --lang node gives each module variable and field an owner, which a holder then ties to".to_owned()
    };
    out.insert("needed".into(), json!(needed_text));
    Ok(series_raw)
}

/// The next step for a tied finding, given the holder the snapshots show and the one the
/// ledger has. The ledger fills a holder that is not set and refuses to change one that is.
fn holder_needed(
    f: &impl std::fmt::Display,
    seen: Option<FindingHolder>,
    named: Option<FindingHolder>,
    growth_id: &impl std::fmt::Display,
) -> String {
    match seen {
        Some(h) if named.is_none() => format!(
            "the snapshots show holder {h} for {f}: dla ledger annotate {f} --holder {h}; then a refuted non-leak sibling, then dla ledger confirm {f} --evidence {growth_id}"
        ),
        // The ledger refuses to change a holder that is set: say so, and name the route
        // it accepts instead of a command it would refuse.
        Some(h) if named != Some(h) => format!(
            "the snapshots show holder {h} for {f}, which already has holder {}; annotate cannot change it: record the corrected holder on a new hypothesis (dla ledger add --origin hypothesis --sibling-of {f} --hypothesis \"<cause>\" --refute-if \"<result>\" --evidence {growth_id}), or keep {f}'s holder; then a refuted non-leak sibling and dla ledger confirm --evidence {growth_id}",
            named.map_or_else(String::new, |x| x.to_string())
        ),
        _ => format!(
            "a refuted non-leak sibling and a holder, then dla ledger confirm {f} --evidence {growth_id}"
        ),
    }
}

fn canonical_owner(owner: &str) -> Option<String> {
    dla_core::attribution::canonical(owner)
}

// ---------------------------------------------------------------------------------------
// type-histogram-diff
// ---------------------------------------------------------------------------------------

/// The class histogram of heap snapshots.
pub struct NodeHeapTypes;

impl Adapter for NodeHeapTypes {
    fn name(&self) -> &'static str {
        "node-heapsnapshot-types"
    }
    fn op(&self) -> Op {
        Op::TypeHistogramDiff
    }
    fn runtime(&self) -> &'static str {
        "node"
    }
    fn tool_class(&self) -> &'static str {
        // The same reader on the same snapshot as heap-graph: one signal.
        tool_class::DUMP_PARSER
    }
    fn formats(&self) -> &'static str {
        FORMATS
    }
    fn sniff(&self, input: &Input) -> bool {
        is_snapshot(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let snap = load(&i.rel, &i.abs)?;
            let hist: Histogram = snap.histogram();
            drop(snap);
            captures.push(Capture {
                label: i.shown().to_owned(),
                hist: Histogram {
                    rows: hist
                        .rows
                        .into_iter()
                        .map(|r| HistogramRow {
                            name: r.name,
                            count: r.count,
                            bytes: r.bytes,
                        })
                        .collect(),
                    ..hist
                },
                sha256: i.sha256.clone(),
            });
            ctx.tc.require_time("dla summarize type-histogram-diff")?;
        }
        histogram::run_diff(
            ctx,
            opts,
            captures,
            &HistOrigin {
                tool: TOOL,
                class: self.tool_class(),
                authority: Authority::Dump,
                runtime: Some(Runtime::Node),
                partial: false,
                order_by_time: false,
                limitations: vec![
                    "sizes are shallow self sizes of V8 nodes; strings are one row, never named after their text".into(),
                    "the snapshots are compared in the order given; snapshots of separate processes compare by class only".into(),
                    "a class histogram ties to no finding: only a holder of a heap-graph series does".into(),
                ],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn modules_are_named_like_the_census_does() {
        let project = Path::new("/work/proj");
        assert_eq!(
            module_of("/work/proj/src/scheduler.js", project),
            ("src.scheduler".to_owned(), true)
        );
        assert_eq!(
            module_of("node:internal/timers", project),
            ("node:internal/timers".to_owned(), false)
        );
        // A path from another machine and no such file here: only the tail is kept.
        let (m, p) = module_of("/home/ci/build/app/lib/x.js", project);
        assert_eq!((m.as_str(), p), ("x", false));
        let (m, _) = module_of("/home/u/app/node_modules/left-pad/index.js", project);
        assert_eq!(m, "node_modules.left-pad.index");
    }

    #[test]
    fn a_foreign_path_is_matched_by_its_longest_existing_tail() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/heartbeat.js"), "x").unwrap();
        let (m, p) = module_of("/other/machine/proj/src/heartbeat.js", dir.path());
        assert_eq!((m.as_str(), p), ("src.heartbeat", true));
    }

    #[test]
    fn snapshots_of_different_processes_or_unrelated_ids_are_not_a_series() {
        let id = |rel: &str, pid: Option<&str>, max: u32, ids: &[u32]| Identity {
            rel: rel.into(),
            pid: pid.map(str::to_owned),
            max_id: max,
            ids: ids.to_vec(),
        };
        let mut ok = vec![
            id("b", Some("7"), 30, &[1, 3, 5, 7, 20, 30]),
            id("a", Some("7"), 10, &[1, 3, 5, 7, 10]),
        ];
        order_and_check(&mut ok).unwrap();
        assert_eq!(ok[0].rel, "a");
        let mut pids = vec![
            id("a", Some("1"), 10, &[1]),
            id("b", Some("2"), 20, &[1, 2]),
        ];
        assert!(
            order_and_check(&mut pids)
                .unwrap_err()
                .contains("different processes")
        );
        let mut same = vec![id("a", None, 10, &[1, 2]), id("b", None, 10, &[1, 2])];
        assert!(order_and_check(&mut same).is_err());
        let mut gone = vec![
            id("a", None, 10, &[1, 2, 3, 4]),
            id("b", None, 20, &[9, 20]),
        ];
        assert!(
            order_and_check(&mut gone)
                .unwrap_err()
                .contains("not the same process")
        );
    }

    #[test]
    fn a_foreign_package_is_not_a_project_file_even_when_its_basename_is() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("index.js"), "x").unwrap();
        let (m, p) = module_of("/ci/app/node_modules/lodash/index.js", dir.path());
        assert_eq!((m.as_str(), p), ("node_modules.lodash.index", false));
        // A one-component tail of a longer foreign path is not a match either.
        let (m, p) = module_of("/ci/app/lib/index.js", dir.path());
        assert_eq!((m.as_str(), p), ("index", false));
        // A package under the project is a dependency.
        let inside = dir.path().join("node_modules/x/index.js");
        let (_, p) = module_of(inside.to_str().unwrap(), dir.path());
        assert!(!p);
        // A bare file name is the project's own file.
        let (m, p) = module_of("index.js", dir.path());
        assert_eq!((m.as_str(), p), ("index", true));
    }

    #[test]
    fn dictionary_keys_of_generic_containers_are_shaped() {
        use dla_parsers::node::{Holder, Hop};
        let red = super::super::redact::Redactor::new(b"salt", false);
        let mut h = Holder {
            kind: HolderKind::Instance,
            registry: None,
            name: "alice_0b".into(),
            owner: "Object".into(),
            script: None,
            retained: 1,
            count: 1,
            instances: 1,
            path: vec![
                Hop {
                    node: "Object".into(),
                    edge: ".alice_0b".into(),
                },
                Hop {
                    node: "Cache".into(),
                    edge: ".items".into(),
                },
            ],
            accumulation: None,
        };
        sanitize(&mut h, &red);
        assert!(!h.name.contains("alice"), "{}", h.name);
        assert!(!h.path[0].edge.contains("alice"), "{}", h.path[0].edge);
        assert_eq!(h.path[1].edge, ".items");
        // A user class keeps its field names.
        let mut u = Holder {
            name: "items".into(),
            owner: "Cache".into(),
            path: vec![],
            ..h
        };
        sanitize(&mut u, &red);
        assert_eq!(u.name, "items");
    }

    #[test]
    fn an_annotate_command_is_named_only_for_a_finding_without_a_holder() {
        let seen = Some(FindingHolder::RegistrationTable);
        let none = holder_needed(&"F6", seen, None, &"E9");
        assert!(
            none.contains("ledger annotate F6 --holder registration-table"),
            "{none}"
        );
        let other = holder_needed(&"F6", seen, Some(FindingHolder::BlockedStack), &"E9");
        assert!(!other.contains("ledger annotate"), "{other}");
        assert!(
            other.contains("already has holder blocked-stack"),
            "{other}"
        );
        assert!(other.contains("--sibling-of F6"), "{other}");
        let same = holder_needed(&"F6", seen, seen, &"E9");
        assert!(!same.contains("annotate"), "{same}");
    }

    #[test]
    fn data_like_property_names_are_not_plain() {
        assert!(plain_ident("latest"));
        assert!(plain_ident("_onTimeout"));
        assert!(!plain_ident("user-12345"));
        assert!(!plain_ident("9f8e7d6c5b4a"));
        assert!(!plain_ident(""));
    }
}
