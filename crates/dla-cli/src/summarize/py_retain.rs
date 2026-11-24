//! `dla summarize heap-graph` over the Python harness's holder chains (`py-<n>.retain.json`,
//! `dla harness run --capture heap`).
//!
//! A `py.blocks@<file:line>` column says where the surviving objects were made, not who keeps
//! them. The agent follows the live objects of the lines that grew to a root, and this adapter
//! turns the chains into one `retention-path` record whose `holders` have the shape the other
//! heap-graph adapters write (`key`, `holder_kind`), so the label rule, the bundle and the
//! report need no special case. One holder per distinct root:
//!
//! * a module global is `static:<module>.<key>` and a class attribute `static:<class>.<attribute>`,
//!   both static/global;
//! * a frame is `stack:<function>@<file:line>`, a blocked stack;
//! * objects nothing refers to, and a site whose blocks no object belongs to, are held from
//!   native code (or a reference was lost there): the holder is the site, kind
//!   native-handle/global-ref.
//!
//! Each holder lists the sites it holds with their block counts at the first and last dump
//! point, the number of kept objects found, and the chain with the most objects as `path` (a
//! frame key the bundle leaves unscrubbed, so a step with `at` can get a code excerpt). The
//! full chains of every holder are in the raw file, which also keeps the digest of the input
//! under `captures` (the place the allocation adapters keep theirs).
//!
//! A holder is tied to the findings whose owner names it (tie rule 1) and to the findings at a
//! site it holds. A holder that no finding names gets the command that adds one; a holder that
//! one names gets the command its route needs next.

use std::collections::BTreeMap;

use dla_core::{
    FindingId, Ledger,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, Source, tool_class,
    },
    finding::{Holder as FindingHolder, Polarity, Region},
    runtime::Runtime,
    toolctx::EvidenceSpec,
};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use super::{Adapter, AdapterOut, CandidateRow, Ctx, Op, Opts, Recorded, emit, input::Input, tie};
use crate::fail::{Failure, Res};

/// Tool name in the evidence.
const TOOL: &str = "dla-py-retain";
/// Largest file read.
const MAX_BYTES: u64 = 16 << 20;
/// Longest a name is shown.
const CLIP: usize = 100;
/// Holders the command's output lists.
const SHOWN_HOLDERS: usize = 4;
/// Commands the output carries.
const SHOWN_NEXT: usize = 3;
/// Sites one holder shows in the evidence record.
const RECORD_SITES: usize = 6;
/// Steps of the path one holder shows in the evidence record.
const RECORD_STEPS: usize = 8;
/// Ties the evidence record states.
const STATED_TIES: usize = 3;

/// What the record says about a site no Python object belongs to.
const UNREFERENCED: &str = "no Python container refers to these blocks: native code holds them, or a reference was lost there";

/// The reader of the Python holder chains.
pub struct PyRetain;

#[derive(Deserialize, Default)]
struct File {
    #[serde(default)]
    dla_pyretain: Option<u32>,
    #[serde(default)]
    frames: u64,
    #[serde(default)]
    sites: Vec<SiteIn>,
    #[serde(default)]
    limitations: Vec<String>,
}

#[derive(Deserialize, Default)]
struct SiteIn {
    #[serde(default)]
    site: String,
    #[serde(default)]
    blocks_first: Option<u64>,
    #[serde(default)]
    blocks_last: u64,
    #[serde(default)]
    found_tracked: u64,
    #[serde(default)]
    found_untracked: u64,
    #[serde(default)]
    kept: u64,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    chains: Vec<ChainIn>,
}

#[derive(Deserialize, Default)]
struct ChainIn {
    #[serde(default)]
    count: u64,
    #[serde(default)]
    root: RootIn,
    #[serde(default)]
    steps: Vec<StepIn>,
}

#[derive(Deserialize, Default)]
struct RootIn {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    module: String,
    #[serde(default)]
    key: String,
    #[serde(default, rename = "class")]
    class_name: String,
    #[serde(default)]
    attribute: String,
    #[serde(default)]
    function: String,
    #[serde(default)]
    at: String,
    /// Whose the root is, as the agent says it (`project`, `tests`); absent in a file of an
    /// older agent and for a root of a library.
    #[serde(default)]
    of: Option<String>,
}

#[derive(Deserialize, Default, Clone, Debug)]
struct StepIn {
    #[serde(default, rename = "type")]
    ty: String,
    #[serde(default)]
    via: Option<String>,
    #[serde(default)]
    at: Option<String>,
    #[serde(default)]
    repeat: Option<u64>,
    /// The distinct places a run of tracebacks names, in the order met (`at` is the first).
    #[serde(default)]
    ats: Vec<String>,
}

/// A holder holds a site (it ties the site's column) only when its chains carry at least this
/// share of the objects sampled for the site, a quarter; below it the site is a minor one.
const MAJOR_SHARE: f64 = 0.25;
/// A site whose block growth is at least this many blocks can be told to belong to native code.
const THIN_MIN_GROWTH: u64 = 20;
/// A site for which the objects found are fewer than this share of its block growth, a tenth,
/// belongs to native code as well: the Python holders of the few objects found are minor.
const THIN_SHARE: f64 = 0.1;
/// Types a holder lists in the evidence record.
const RECORD_TYPES: usize = 6;
/// Minor sites a holder lists in the evidence record.
const RECORD_MINOR: usize = 4;

/// One site a holder holds.
#[derive(Debug, Clone)]
struct Held {
    site: String,
    first: Option<u64>,
    last: u64,
    tracked: u64,
    untracked: u64,
    kept: u64,
}

/// One group of chains of a site that end at the holder.
#[derive(Debug, Clone)]
struct PathRow {
    site: String,
    count: u64,
    steps: Vec<StepIn>,
}

/// One holder: a distinct root.
#[derive(Debug, Clone)]
struct HolderOut {
    key: String,
    kind: FindingHolder,
    /// `module.name`, the form the tie rule uses; `None` for a stack or a native holder.
    owner: Option<String>,
    sites: Vec<Held>,
    /// Sites where the holder keeps a small part of the objects sampled: no tie, no command.
    minor: Vec<Held>,
    /// The types of the objects it keeps (the kept object of each chain and a step that repeats),
    /// with the number of objects.
    types: BTreeMap<String, u64>,
    kept: u64,
    paths: Vec<PathRow>,
    /// No Python container refers to the blocks or objects of this holder.
    unreferenced: bool,
    /// Whose it is: the project's own, or test code's; `None` for a frame, a native holder, a
    /// library's global and a holder an older agent did not classify.
    of: Option<Of>,
}

/// Whose a holder is (see [`dla_core::measured_holders`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Of {
    /// A module global or class attribute of a project source file that is not test code.
    Project,
    /// A root in test code or in a module of the test runner.
    Tests,
}

impl Of {
    fn as_str(self) -> &'static str {
        match self {
            Of::Project => "project",
            Of::Tests => "tests",
        }
    }

    fn parse(s: &str) -> Option<Of> {
        match s {
            "project" => Some(Of::Project),
            "tests" => Some(Of::Tests),
            _ => None,
        }
    }
}

/// What a module name alone says about a root of a file an older agent wrote: the driver module
/// and the modules of the test runner, and a module whose path has a `test` or `tests` part or
/// whose last part is `test_*`, `*_test` or `conftest`, are the tests'. Nothing says that a module
/// is the project's: that needs its file.
fn of_by_module(module: &str) -> Option<Of> {
    let parts: Vec<&str> = module.split('.').collect();
    let (top, last) = (parts.first()?, parts.last()?);
    let runner = [
        "pytest", "_pytest", "unittest", "pluggy", "nose", "nose2", "doctest",
    ];
    (module == "__main__"
        || runner.contains(top)
        || parts[..parts.len() - 1]
            .iter()
            .any(|p| *p == "test" || *p == "tests")
        || last.starts_with("test_")
        || last.ends_with("_test")
        || *last == "conftest")
        .then_some(Of::Tests)
}

impl Held {
    /// Blocks gained between the dump points; the last count when there was one dump point.
    fn growth(&self) -> u64 {
        self.last.saturating_sub(self.first.unwrap_or(0))
    }
}

impl HolderOut {
    /// The types it keeps, most frequent first.
    fn types_sorted(&self) -> Vec<(&str, u64)> {
        let mut v: Vec<(&str, u64)> = self.types.iter().map(|(k, n)| (k.as_str(), *n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        v
    }

    /// The chain with the most objects.
    fn best(&self) -> Option<&PathRow> {
        self.paths.iter().max_by_key(|p| p.count)
    }

    /// The site of a finding added for this holder, never a place outside the project: from the
    /// root side, the first step that names a project line in `at` or `ats`; of the lines of that
    /// step the one the holder keeps that grew most, else the first; a chain with no project line
    /// gives the line the holder keeps that grew most.
    fn finding_site(&self) -> Option<String> {
        let grew = |line: &str| self.sites.iter().find(|s| s.site == line).map(Held::growth);
        let from_chain = self.best().and_then(|p| {
            p.steps.iter().rev().find_map(|st| {
                let mut lines: Vec<&String> = Vec::new();
                for l in st.at.iter().chain(&st.ats) {
                    if is_project_line(l) && !lines.contains(&l) {
                        lines.push(l);
                    }
                }
                let held = lines
                    .iter()
                    .filter_map(|l| grew(l).map(|g| (g, *l)))
                    .max_by_key(|(g, _)| *g)
                    .map(|(_, l)| l);
                held.or_else(|| lines.first().copied()).cloned()
            })
        });
        from_chain.or_else(|| {
            self.sites
                .iter()
                .max_by_key(|s| s.growth())
                .map(|s| s.site.clone())
        })
    }
}

/// True for a `file:line` that names a line of the project: not a place outside it
/// (`<outside>/name:7`) and not a pseudo file (`<string>:3`).
fn is_project_line(place: &str) -> bool {
    !place.starts_with('<') && !place.starts_with('?') && !place.is_empty()
}

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push_str("...");
        t
    }
}

fn bad(i: &Input, why: impl std::fmt::Display) -> Failure {
    Failure::refuse(
        "malformed-artifact",
        format!("{}: {why}", i.shown()),
        "py-<n>.retain.json written by the Python harness",
        "dla harness run <H> --capture heap",
    )
}

fn parse(i: &Input) -> Res<File> {
    let text = i.read_text(MAX_BYTES)?;
    let f: File = serde_json::from_str(&text).map_err(|e| bad(i, e))?;
    if f.dla_pyretain.is_none() {
        return Err(bad(i, "no `dla_pyretain` version"));
    }
    Ok(f)
}

/// The holders of one file, largest first by kept objects. Chains that end at the depth limit
/// or reach no root name no holder; their count is the second value.
fn holders_of(f: &File, red: &super::redact::Redactor) -> (Vec<HolderOut>, u64) {
    let mut by_key: BTreeMap<String, HolderOut> = BTreeMap::new();
    let mut unrooted = 0u64;
    let name = |s: &str| red.name(s);
    let step = |s: &StepIn| StepIn {
        ty: name(&s.ty),
        via: s.via.as_deref().map(name),
        at: s.at.as_deref().map(name),
        repeat: s.repeat,
        ats: s.ats.iter().map(|a| name(a)).collect(),
    };
    /// One group of chains of a site, before it is told which holder it belongs to.
    struct Piece {
        key: String,
        kind: FindingHolder,
        owner: Option<String>,
        unreferenced: bool,
        of: Option<Of>,
        count: u64,
        steps: Vec<StepIn>,
    }
    for site in &f.sites {
        let found = site.found_tracked + site.found_untracked;
        let growth = site
            .blocks_first
            .map_or(0, |b| site.blocks_last.saturating_sub(b));
        let thin = growth >= THIN_MIN_GROWTH && (found as f64) < THIN_SHARE * growth as f64;
        let held = Held {
            site: name(&site.site),
            first: site.blocks_first,
            last: site.blocks_last,
            tracked: site.found_tracked,
            untracked: site.found_untracked,
            kept: 0,
        };
        let native = |count: u64, steps: Vec<StepIn>| Piece {
            key: held.site.clone(),
            kind: FindingHolder::NativeHandleGlobalRef,
            owner: None,
            unreferenced: true,
            of: None,
            count,
            steps,
        };
        let mut pieces: Vec<Piece> = Vec::new();
        if site.note.is_some() && site.chains.is_empty() {
            pieces.push(native(0, Vec::new()));
        }
        for c in &site.chains {
            let steps: Vec<StepIn> = c.steps.iter().map(step).collect();
            let of = c.root.of.as_deref().and_then(Of::parse);
            let piece = |key: String, kind, owner: Option<String>, of: Option<Of>| Piece {
                key,
                kind,
                owner,
                unreferenced: false,
                of,
                count: c.count,
                steps: steps.clone(),
            };
            match c.root.kind.as_str() {
                "module-global" => {
                    let owner = name(&format!("{}.{}", c.root.module, c.root.key));
                    pieces.push(piece(
                        format!("static:{owner}"),
                        FindingHolder::StaticGlobal,
                        Some(owner),
                        of.or_else(|| of_by_module(&c.root.module)),
                    ));
                }
                "class-attribute" => {
                    let owner = name(&format!("{}.{}", c.root.class_name, c.root.attribute));
                    pieces.push(piece(
                        format!("static:{owner}"),
                        FindingHolder::StaticGlobal,
                        Some(owner),
                        of,
                    ));
                }
                "frame" => pieces.push(piece(
                    format!("stack:{}@{}", name(&c.root.function), name(&c.root.at)),
                    FindingHolder::BlockedStack,
                    None,
                    None,
                )),
                "no-referrer" => pieces.push(native(c.count, steps.clone())),
                _ => unrooted += c.count,
            }
        }
        // A site that belongs to native code as much as to anything: its own native entry too.
        if thin && !pieces.iter().any(|p| p.key == held.site) {
            pieces.push(native(0, Vec::new()));
        }
        // What each holder keeps of the objects sampled for this site.
        let mut share: BTreeMap<String, u64> = BTreeMap::new();
        for p in &pieces {
            *share.entry(p.key.clone()).or_default() += p.count;
        }
        for p in pieces {
            let total = share[&p.key];
            let native_entry = p.unreferenced;
            let major = native_entry
                || (!thin && site.kept > 0 && (total as f64) >= MAJOR_SHARE * site.kept as f64);
            let h = by_key.entry(p.key.clone()).or_insert_with(|| HolderOut {
                key: p.key.clone(),
                kind: p.kind,
                owner: p.owner.clone(),
                sites: Vec::new(),
                minor: Vec::new(),
                types: BTreeMap::new(),
                kept: 0,
                paths: Vec::new(),
                unreferenced: p.unreferenced,
                of: p.of,
            });
            h.kept += p.count;
            let list = if major { &mut h.sites } else { &mut h.minor };
            match list.iter_mut().find(|s| s.site == held.site) {
                Some(s) => s.kept += p.count,
                None => list.push(Held {
                    kept: p.count,
                    ..held.clone()
                }),
            }
            if let Some(first) = p.steps.first() {
                *h.types.entry(first.ty.clone()).or_default() += p.count;
            }
            for st in p.steps.iter().skip(1).filter(|st| st.repeat.is_some()) {
                *h.types.entry(st.ty.clone()).or_default() += p.count;
            }
            if !p.steps.is_empty() {
                h.paths.push(PathRow {
                    site: held.site.clone(),
                    count: p.count,
                    steps: p.steps,
                });
            }
        }
    }
    let mut v: Vec<HolderOut> = by_key.into_values().collect();
    for h in &mut v {
        h.sites.sort_by_key(|s| std::cmp::Reverse(s.growth()));
        h.minor.sort_by_key(|s| std::cmp::Reverse(s.growth()));
    }
    v.sort_by(|a, b| b.kept.cmp(&a.kept).then_with(|| a.key.cmp(&b.key)));
    (v, unrooted)
}

fn steps_json(steps: &[StepIn]) -> Value {
    json!(
        steps
            .iter()
            .map(|s| {
                let mut o = Map::new();
                o.insert("type".into(), json!(clip(&s.ty, CLIP)));
                if let Some(v) = &s.via {
                    o.insert("via".into(), json!(clip(v, CLIP)));
                }
                if let Some(a) = &s.at {
                    o.insert("at".into(), json!(clip(a, CLIP)));
                }
                if let Some(r) = s.repeat {
                    o.insert("repeat".into(), json!(r));
                }
                if !s.ats.is_empty() {
                    o.insert(
                        "ats".into(),
                        json!(s.ats.iter().map(|a| clip(a, CLIP)).collect::<Vec<_>>()),
                    );
                }
                Value::Object(o)
            })
            .collect::<Vec<_>>()
    )
}

/// A path cut to about `n` steps: the first and the last step stay, and the steps in between
/// that say where code was (`at`) or that repeat stay longest; a run of steps cut is one `...`.
fn short_steps(steps: &[StepIn], n: usize) -> Vec<StepIn> {
    if steps.len() <= n {
        return steps.to_vec();
    }
    let last = steps.len() - 1;
    let mut keep = vec![true; steps.len()];
    let mut over = steps.len() - n;
    // Plain middle steps go first, then the ones with a place, from the kept object's side.
    for plain in [true, false] {
        for k in 1..last {
            if over == 0 {
                break;
            }
            let marked = steps[k].at.is_some() || steps[k].repeat.is_some();
            if keep[k] && marked != plain {
                keep[k] = false;
                over -= 1;
            }
        }
    }
    let mut v: Vec<StepIn> = Vec::new();
    for (k, s) in steps.iter().enumerate() {
        if keep[k] {
            v.push(s.clone());
        } else if v.last().is_none_or(|p| p.ty != "...") {
            v.push(StepIn {
                ty: "...".into(),
                ..StepIn::default()
            });
        }
    }
    v
}

/// `of` (whose the holder is) and, for a holder of the project that keeps lines, the line a
/// finding added for it gets (`finding_site`) and how many lines it keeps (`lines`): what the gate
/// reads to say that no finding covers it.
fn insert_of(o: &mut Map<String, Value>, h: &HolderOut) {
    let Some(of) = h.of else { return };
    o.insert("of".into(), json!(of.as_str()));
    if of == Of::Project && !h.sites.is_empty() {
        o.insert("lines".into(), json!(h.sites.len()));
        if let Some(site) = h.finding_site() {
            o.insert("finding_site".into(), json!(site));
        }
    }
}

/// True for a holder of the project that keeps at least one major line and that no finding
/// covers (no finding has its owner): the first thing to act on.
fn needs_a_finding(h: &HolderOut, ledger: &Ledger) -> bool {
    h.of == Some(Of::Project)
        && !h.sites.is_empty()
        && h.owner
            .as_deref()
            .is_some_and(|o| !dla_core::measured_holders::covered(ledger, o))
}

/// The holder as the evidence record shows it.
fn record_entry(h: &HolderOut, sites: usize, steps: usize, types: usize) -> Value {
    let mut o = Map::new();
    o.insert("key".into(), json!(clip(&h.key, CLIP)));
    o.insert("holder_kind".into(), json!(h.kind));
    o.insert("kept".into(), json!(h.kept));
    insert_of(&mut o, h);
    o.insert(
        "sites".into(),
        json!(
            h.sites
                .iter()
                .take(sites)
                .map(|s| json!({"site": clip(&s.site, CLIP), "first": s.first, "last": s.last}))
                .collect::<Vec<_>>()
        ),
    );
    if h.sites.len() > sites {
        o.insert("more_sites".into(), json!(h.sites.len() - sites));
    }
    if !h.minor.is_empty() {
        o.insert(
            "minor_sites".into(),
            json!(
                h.minor
                    .iter()
                    .take(RECORD_MINOR.min(types))
                    .map(|s| clip(&s.site, CLIP))
                    .collect::<Vec<_>>()
            ),
        );
    }
    let tys = h.types_sorted();
    if !tys.is_empty() && types > 0 {
        o.insert(
            "types".into(),
            json!(
                tys.iter()
                    .take(types)
                    .map(|(t, _)| clip(t, CLIP))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if h.unreferenced {
        o.insert("unreferenced".into(), json!(UNREFERENCED));
    }
    if let Some(p) = h.best() {
        o.insert("path".into(), steps_json(&short_steps(&p.steps, steps)));
    }
    Value::Object(o)
}

/// The holder as the raw file shows it: every site and every chain.
fn raw_entry(h: &HolderOut) -> Value {
    let mut o = Map::new();
    o.insert("key".into(), json!(h.key));
    o.insert("holder_kind".into(), json!(h.kind));
    if let Some(owner) = &h.owner {
        o.insert("owner".into(), json!(owner));
    }
    o.insert("kept".into(), json!(h.kept));
    insert_of(&mut o, h);
    o.insert(
        "sites".into(),
        json!(
            h.sites
                .iter()
                .map(|s| json!({
                    "site": s.site,
                    "first": s.first,
                    "last": s.last,
                    "found_tracked": s.tracked,
                    "found_untracked": s.untracked,
                    "kept": s.kept,
                }))
                .collect::<Vec<_>>()
        ),
    );
    o.insert(
        "minor_sites".into(),
        json!(
            h.minor
                .iter()
                .map(|s| json!({
                    "site": s.site,
                    "first": s.first,
                    "last": s.last,
                    "found_tracked": s.tracked,
                    "found_untracked": s.untracked,
                    "kept": s.kept,
                }))
                .collect::<Vec<_>>()
        ),
    );
    o.insert(
        "types".into(),
        json!(
            h.types_sorted()
                .iter()
                .map(|(t, n)| json!({"type": t, "objects": n}))
                .collect::<Vec<_>>()
        ),
    );
    if h.unreferenced {
        o.insert("unreferenced".into(), json!(UNREFERENCED));
    }
    o.insert(
        "paths".into(),
        json!(
            h.paths
                .iter()
                .map(|p| json!({"site": p.site, "count": p.count, "path": steps_json(&p.steps)}))
                .collect::<Vec<_>>()
        ),
    );
    Value::Object(o)
}

/// The harness a capture file belongs to: `<H>` of `.../harness/<H>/out/...`.
fn harness_of(rel: &str) -> Option<&str> {
    let mut it = rel.split('/');
    while let Some(p) = it.next() {
        if p == "harness" {
            return it.next().filter(|h| !h.is_empty());
        }
    }
    None
}

/// The growth evidence of a trend that has a passing column for `site`.
fn growth_evidence_of(ledger: &Ledger, site: &str) -> Option<String> {
    let column = format!("py.blocks@{site}");
    ledger
        .evidence()
        .filter(|e| {
            matches!(
                e.kind,
                EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries
            )
        })
        .filter(|e| {
            e.result
                .get("passed")
                .and_then(Value::as_array)
                .is_some_and(|p| {
                    p.iter().any(|c| {
                        c.get("column")
                            .and_then(Value::as_str)
                            .is_some_and(|n| n.strip_suffix(".pre_gc").unwrap_or(n) == column)
                    })
                })
        })
        .map(|e| e.evidence_id.to_string())
        .last()
}

/// The findings of a holder: the leak-side findings whose owner names it, and those at a site
/// it holds.
fn findings_of(
    ledger: &Ledger,
    who: &tie::Who,
    h: &HolderOut,
    owners: &[String],
) -> Vec<(FindingId, tie::TieReason)> {
    let mut m: Vec<(FindingId, tie::TieReason)> = Vec::new();
    if let Some(o) = &h.owner {
        for f in tie::findings_by_holder(ledger, who, o, owners, None) {
            m.push((f, tie::TieReason::holder(o)));
        }
    }
    for s in &h.sites {
        for f in tie::findings_at_site(ledger, who, &s.site) {
            m.push((f, tie::TieReason::site(&s.site)));
        }
    }
    m.retain(|(f, _)| {
        ledger
            .finding(f)
            .is_some_and(|x| x.polarity() == Polarity::Leak)
    });
    m
}

/// The command a holder that names no finding gets.
fn add_command(h: &HolderOut, evidence: &str) -> String {
    let site = h.finding_site().unwrap_or_default();
    let owner = h
        .owner
        .as_ref()
        .map(|o| format!(" --owner {o}"))
        .unwrap_or_default();
    format!(
        "dla ledger add --origin growth-series --evidence {evidence} --site {site}{owner} --region heap --holder {}",
        h.kind
    )
}

/// The command the route of the finding that names a holder needs next.
fn route_command(
    ledger: &Ledger,
    f: &FindingId,
    h: &HolderOut,
    harness: Option<&str>,
    evidence: &str,
) -> String {
    let Some(finding) = ledger.finding(f) else {
        return format!("dla ledger show {f}");
    };
    if finding.holder.is_none() {
        return format!("dla ledger annotate {f} --holder {}", h.kind);
    }
    let growth = h
        .sites
        .iter()
        .find_map(|s| growth_evidence_of(ledger, &s.site));
    let has_sibling = ledger.findings().any(|x| x.parent.as_ref() == Some(f));
    let can_test = matches!(
        finding.state,
        dla_core::FindingState::Candidate
            | dla_core::FindingState::Unresolved
            | dla_core::FindingState::SuspectedStaticOnly
    );
    let cmds = dla_core::grown::commands(
        f,
        harness.unwrap_or("<H>"),
        growth.as_deref().or(Some(evidence)),
        None,
        has_sibling,
        can_test,
        false,
    );
    cmds.into_iter().next().unwrap_or_default()
}

impl Adapter for PyRetain {
    fn name(&self) -> &'static str {
        "py-retain"
    }
    fn op(&self) -> Op {
        Op::HeapGraph
    }
    fn runtime(&self) -> &'static str {
        "python"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::DUMP_PARSER
    }
    fn formats(&self) -> &'static str {
        "Python harness holder chains (py-<n>.retain.json, {\"dla_pyretain\":1, \"sites\"})"
    }
    fn sniff(&self, input: &Input) -> bool {
        input.head_text().contains("\"dla_pyretain\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        retain(ctx, inputs, opts)
    }
}

fn retain(ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
    let tc = ctx.tc;
    let red = &ctx.redactor;
    let mut parsed = Vec::new();
    for i in inputs {
        parsed.push(parse(i)?);
    }
    let mut ledger_ref = ctx.ledger()?;
    let ledger: &mut Ledger = &mut ledger_ref;
    if let Some(f) = &opts.finding
        && ledger.finding(f).is_none()
    {
        return Err(Failure::from(dla_core::Refusal::new(
            dla_core::error::codes::UNKNOWN_FINDING,
            format!("no finding {f} in the ledger"),
            "an existing finding",
            "dla ledger show",
        )));
    }
    let who = tie::Who::new(tc.worker.as_deref(), Some(Runtime::Python));
    let mut out = AdapterOut::default();
    let mut shown_holders: Vec<Value> = Vec::new();
    let mut next: Vec<String> = Vec::new();
    let mut all_unrooted = 0u64;
    let mut unreferenced_sites = 0usize;
    let mut files: Vec<Value> = Vec::new();
    let mut attention: Vec<String> = Vec::new();
    let mut no_chain_total = 0usize;
    let mut no_chain_reason: Option<String> = None;
    for (i, f) in inputs.iter().zip(&parsed) {
        let (holders, unrooted) = holders_of(f, red);
        all_unrooted += unrooted;
        unreferenced_sites += holders
            .iter()
            .filter(|h| h.unreferenced)
            .map(|h| h.sites.len())
            .sum::<usize>();
        let owners: Vec<String> = holders.iter().filter_map(|h| h.owner.clone()).collect();
        let found: Vec<Vec<(FindingId, tie::TieReason)>> = holders
            .iter()
            .map(|h| findings_of(ledger, &who, h, &owners))
            .collect();
        let ties: Vec<tie::Resolved> = found
            .iter()
            .map(|m| tie::resolve(m.clone(), opts.finding.as_ref()))
            .collect();
        // ---- the raw file: every holder and chain, the digest of the input
        let (id, arel) = emit::next_id(tc, "retain", 'R')?;
        let mut limitations: Vec<String> = f.limitations.clone();
        if unrooted > 0 {
            limitations.push(format!(
                "{unrooted} kept object(s) ended at the depth limit or reached no root, so they name no holder"
            ));
        }
        if f.sites.iter().any(|s| s.blocks_first.is_none()) {
            limitations.push(
                "the file has one dump point (blocks_first is null): growth was not computed, and the sites are the largest ones at that point".to_owned(),
            );
        }
        // Sites that have kept objects and no chain: the walk did not finish.
        let no_chain = f
            .sites
            .iter()
            .filter(|s| s.kept > 0 && s.chains.is_empty())
            .count();
        let no_chain_why = f
            .limitations
            .iter()
            .find(|l| l.contains("have no chain") || l.contains("stopped the scan"))
            .map_or_else(
                || "no reason is given in the file".to_owned(),
                |l| clip(l, 160),
            );
        if no_chain > 0 {
            limitations.push(format!(
                "{no_chain} site(s) have kept objects and no chain: {no_chain_why}"
            ));
        }
        no_chain_total += no_chain;
        no_chain_reason.get_or_insert_with(|| no_chain_why.clone());
        let raw_value = json!({
            "id": id,
            "tool": TOOL,
            "mode": "single",
            "captures": [{
                "number": 1,
                "label": i.shown(),
                "frames": f.frames,
                "sites": f.sites.len(),
                "sha256": i.sha256,
            }],
            "holders": holders.iter().map(raw_entry).collect::<Vec<_>>(),
            "sites": f.sites.iter().map(|s| json!({
                "site": red.name(&s.site),
                "blocks_first": s.blocks_first,
                "blocks_last": s.blocks_last,
                "found_tracked": s.found_tracked,
                "found_untracked": s.found_untracked,
                "kept": s.kept,
                "unreferenced": (s.note.is_some() && s.chains.is_empty()).then_some(UNREFERENCED),
            })).collect::<Vec<_>>(),
            "limitations": limitations,
            "redaction": red.mode(),
        });
        let raw = emit::write_raw(tc, &arel, &raw_value)?;
        // ---- the evidence record
        let mut stated: Vec<Value> = Vec::new();
        let mut cited: Vec<FindingId> = Vec::new();
        for (h, r) in holders.iter().zip(&ties) {
            let Some((fid, why)) = &r.tie else { continue };
            if stated.len() >= STATED_TIES {
                break;
            }
            let mut all: Vec<FindingId> = std::iter::once(fid.clone())
                .chain(r.also.iter().cloned())
                .collect();
            all.sort();
            all.dedup();
            stated.push(json!({
                "holder": clip(&h.key, 40),
                "finding": fid,
                "candidates": all,
                "tie": why.to_json(),
            }));
            cited.extend(all);
        }
        cited.sort();
        cited.dedup();
        // Holders the findings name go first, then by kept objects.
        let mut order: Vec<usize> = (0..holders.len()).collect();
        // The record is bounded and the gate reads it: a holder of the project that no finding
        // covers stays in it ahead of the holders that say nothing it has to act on.
        let wanted: Vec<bool> = holders.iter().map(|h| needs_a_finding(h, ledger)).collect();
        order.sort_by_key(|&k| {
            (
                ties[k].tie.is_none(),
                !wanted[k],
                std::cmp::Reverse(holders[k].kept),
            )
        });
        let build =
            |n: usize, sites: usize, steps: usize, types: usize, with_ties: bool| -> Value {
                let mut base = Map::new();
                base.insert("file".into(), json!(clip(i.shown(), 80)));
                base.insert("sites".into(), json!(f.sites.len()));
                base.insert(
                    "kept".into(),
                    json!(holders.iter().map(|h| h.kept).sum::<u64>()),
                );
                base.insert(
                    "holders".into(),
                    json!(
                        order
                            .iter()
                            .take(n)
                            .map(|&k| record_entry(&holders[k], sites, steps, types))
                            .collect::<Vec<_>>()
                    ),
                );
                if holders.len() > n {
                    base.insert("more".into(), json!(holders.len() - n));
                }
                if no_chain > 0 {
                    base.insert(
                        "no_chain".into(),
                        json!({"sites": no_chain, "why": clip(&no_chain_why, 100)}),
                    );
                }
                if with_ties && !stated.is_empty() {
                    base.insert("finding".into(), json!(cited[0]));
                    base.insert("candidates".into(), json!(cited));
                    base.insert("ties".into(), json!(stated));
                }
                Value::Object(base)
            };
        let (mut n, mut sites, mut steps, mut types, mut with_ties) = (
            holders.len(),
            RECORD_SITES,
            RECORD_STEPS,
            RECORD_TYPES,
            true,
        );
        let mut result = build(n, sites, steps, types, with_ties);
        while dla_core::evidence::result_size(&result) > emit::RESULT_BUDGET {
            if steps > 4 {
                steps = 4;
            } else if types > 3 {
                types = 3;
            } else if sites > 2 {
                sites = 2;
            } else if n > 1 {
                n -= 1;
            } else if sites > 1 {
                sites = 1;
            } else if with_ties {
                with_ties = false;
            } else if types > 0 {
                types = 0;
            } else {
                break;
            }
            result = build(n, sites, steps, types, with_ties);
        }
        let mut spec = EvidenceSpec::new(
            EvidenceKind::RetentionPath,
            TOOL,
            env!("CARGO_PKG_VERSION"),
            result,
        );
        spec.authority = ctx.authority(Authority::Live);
        spec.source = Some(Source::Artifact {
            path: i.rel.clone(),
            sha256: i.sha256.clone(),
        });
        spec.coverage = EvidenceCoverage {
            status: CoverageStatus::Complete,
            examined: Some(f.sites.len() as u64),
            total: None,
        };
        spec.limitations = limitations;
        let signal = Signal::new(TOOL, tool_class::DUMP_PARSER, &i.sha256);
        let rec: Recorded = ctx.record(ledger, spec, &signal, &raw)?;
        let evidence = rec.id.to_string();
        // ---- what is said, and what comes next
        for (k, h) in holders.iter().enumerate() {
            let named = &ties[k];
            let mut row = json!({
                "key": clip(&h.key, 56),
                "holder_kind": h.kind,
                "kept": h.kept,
                "sites": h.sites.len(),
            });
            if h.unreferenced {
                row["unreferenced"] = json!(true);
            }
            match &named.tie {
                Some((fid, _)) => {
                    row["finding"] = json!(fid);
                    if shown_holders.len() < SHOWN_HOLDERS && next.len() < SHOWN_NEXT {
                        next.push(route_command(
                            ledger,
                            fid,
                            h,
                            harness_of(i.shown()),
                            &evidence,
                        ));
                    }
                    if !out.candidates.iter().any(|c| &c.finding == fid) {
                        let fx = ledger.finding(fid);
                        out.candidates.push(CandidateRow {
                            finding: fid.clone(),
                            owner: fx.and_then(|x| x.owner.clone()),
                            holder: Some(h.kind),
                            site: tie::site_of(ledger, fid),
                            region: Some(Region::Heap),
                            origin: "growth-series".into(),
                            added: false,
                            via: None,
                        });
                    }
                }
                None => {
                    if shown_holders.len() < SHOWN_HOLDERS
                        && next.len() < SHOWN_NEXT
                        && !h.sites.is_empty()
                    {
                        next.push(add_command(h, &evidence));
                    }
                }
            }
            if shown_holders.len() < SHOWN_HOLDERS {
                shown_holders.push(row);
            }
        }
        for h in holders.iter().filter(|h| needs_a_finding(h, ledger)) {
            if let Some(owner) = &h.owner {
                attention.push(format!(
                    "A HOLDER OF THE PROJECT HAS NO FINDING: {} keeps {} growing line(s) and no finding has the owner {owner}; act on it before anything else: {}",
                    clip(&h.key, CLIP),
                    h.sites.len(),
                    dla_core::measured_holders::add_command(
                        &evidence,
                        owner,
                        &h.finding_site().unwrap_or_default()
                    )
                ));
            }
        }
        files.push(
            json!({"file": clip(i.shown(), 80), "evidence": evidence, "holders": holders.len()}),
        );
        out.evidence.push(rec);
        out.raw = Some(raw);
        out.enumerated += f.sites.len() as u64;
    }
    let mut body = Map::new();
    // The first key: the output bound never cuts it, and a reader that looks only at the finding it
    // asked about still sees a holder that no finding covers.
    if !attention.is_empty() {
        body.insert("_attention".into(), json!(attention));
    }
    body.insert("files".into(), json!(files));
    body.insert("holders".into(), json!(shown_holders));
    if unreferenced_sites > 0 {
        body.insert("unreferenced_sites".into(), json!(unreferenced_sites));
    }
    if all_unrooted > 0 {
        body.insert("unrooted".into(), json!(all_unrooted));
    }
    if no_chain_total > 0 {
        body.insert("no_chain_sites".into(), json!(no_chain_total));
        body.insert("no_chain_why".into(), json!(no_chain_reason));
    }
    body.insert("next".into(), json!(next));
    body.insert("redaction".into(), json!(red.mode()));
    out.body = body;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(json: &str) -> File {
        serde_json::from_str(json).unwrap()
    }

    fn red() -> super::super::redact::Redactor {
        super::super::redact::Redactor::new(b"salt", false)
    }

    #[test]
    fn one_holder_per_distinct_root_with_the_sites_it_holds() {
        let f = file(
            r#"{"dla_pyretain":1,"frames":1,"sites":[
            {"site":"a.py:3","blocks_first":10,"blocks_last":50,"found_tracked":40,"found_untracked":0,"kept":40,
             "chains":[{"count":30,"root":{"kind":"module-global","module":"pkg.m","key":"_reg"},"steps":[{"type":"pkg.m.E","via":"[0]"},{"type":"list","via":"._reg"}]},
                       {"count":10,"root":{"kind":"class-attribute","class":"pkg.K","attribute":"items"},"steps":[{"type":"pkg.m.E","via":"[1]"}]}]},
            {"site":"a.py:9","blocks_first":5,"blocks_last":45,"found_tracked":40,"found_untracked":0,"kept":40,
             "chains":[{"count":40,"root":{"kind":"module-global","module":"pkg.m","key":"_reg"},"steps":[{"type":"int","via":".n"}]}]},
            {"site":"a.py:12","blocks_first":5,"blocks_last":45,"found_tracked":0,"found_untracked":0,"kept":0,"note":"x","chains":[]},
            {"site":"a.py:15","blocks_first":5,"blocks_last":45,"found_tracked":4,"found_untracked":0,"kept":4,
             "chains":[{"count":4,"root":{"kind":"frame","function":"_w","at":"a.py:20"},"steps":[{"type":"int"}]},
                       {"count":2,"root":{"kind":"depth-limit"},"steps":[{"type":"int"}]}]}]}"#,
        );
        let (h, unrooted) = holders_of(&f, &red());
        assert_eq!(unrooted, 2);
        let keys: Vec<&str> = h.iter().map(|h| h.key.as_str()).collect();
        assert_eq!(
            keys,
            [
                "static:pkg.m._reg",
                "static:pkg.K.items",
                "stack:_w@a.py:20",
                "a.py:12"
            ]
        );
        let reg = &h[0];
        assert_eq!(reg.kept, 70);
        assert_eq!(reg.kind, FindingHolder::StaticGlobal);
        assert_eq!(reg.owner.as_deref(), Some("pkg.m._reg"));
        assert_eq!(reg.sites.len(), 2);
        // The site that grew most (both 40) keeps the order of the larger growth, then input.
        assert_eq!(h[2].kind, FindingHolder::BlockedStack);
        assert!(h[2].owner.is_none());
        let native = &h[3];
        assert_eq!(native.kind, FindingHolder::NativeHandleGlobalRef);
        assert!(native.unreferenced && native.paths.is_empty());
        assert_eq!(native.sites[0].site, "a.py:12");
    }

    #[test]
    fn the_finding_site_is_the_step_nearest_the_root_that_has_one() {
        let f = file(
            r#"{"dla_pyretain":1,"sites":[{"site":"a.py:3","blocks_first":0,"blocks_last":9,"kept":5,"chains":[
            {"count":5,"root":{"kind":"module-global","module":"m","key":"_n"},"steps":[
              {"type":"X","via":"tb_frame","at":"a.py:3"},
              {"type":"traceback","via":"__traceback__","at":"a.py:14","repeat":7},
              {"type":"E","via":".ex"},
              {"type":"H","via":"._n"}]}]}]}"#,
        );
        let (h, _) = holders_of(&f, &red());
        assert_eq!(h[0].finding_site().as_deref(), Some("a.py:14"));
        let f = file(
            r#"{"dla_pyretain":1,"sites":[{"site":"a.py:3","blocks_first":0,"blocks_last":9,"kept":5,"chains":[
            {"count":5,"root":{"kind":"module-global","module":"m","key":"_n"},"steps":[{"type":"X","via":".a"}]}]}]}"#,
        );
        let (h, _) = holders_of(&f, &red());
        assert_eq!(h[0].finding_site().as_deref(), Some("a.py:3"));
    }

    fn holder_with_chain(steps: &str, sites: &[(&str, u64)]) -> HolderOut {
        let site_rows: Vec<String> = sites
            .iter()
            .map(|(s, last)| {
                format!(
                    r#"{{"site":"{s}","blocks_first":0,"blocks_last":{last},"found_tracked":10,"kept":10,
                    "chains":[{{"count":10,"root":{{"kind":"module-global","module":"m","key":"g"}},"steps":[{steps}]}}]}}"#
                )
            })
            .collect();
        let f = file(&format!(
            r#"{{"dla_pyretain":1,"sites":[{}]}}"#,
            site_rows.join(",")
        ));
        holders_of(&f, &red()).0.remove(0)
    }

    #[test]
    fn the_site_of_a_finding_is_never_a_place_outside_the_project() {
        // The only `at` of the chain is the driver's, outside: the held line that grew most.
        let h = holder_with_chain(
            r#"{"type":"traceback","via":"__traceback__","at":"<outside>/dla_harness.py:97","repeat":9},{"type":"E","via":".ex"}"#,
            &[("a.py:5", 10), ("a.py:9", 40)],
        );
        assert_eq!(h.finding_site().as_deref(), Some("a.py:9"));
        // A pseudo file is no line either.
        let h = holder_with_chain(
            r#"{"type":"frame","via":"tb_frame","at":"<string>:1"},{"type":"E","via":".ex"}"#,
            &[("a.py:5", 10)],
        );
        assert_eq!(h.finding_site().as_deref(), Some("a.py:5"));
    }

    #[test]
    fn a_run_that_names_several_lines_gives_the_one_the_holder_keeps_that_grew_most() {
        let steps = r#"{"type":"frame","via":"tb_frame","at":"a.py:90"},
            {"type":"traceback","via":"__traceback__","at":"<outside>/dla_harness.py:97","repeat":9,
             "ats":["<outside>/dla_harness.py:97","a.py:7","a.py:6","a.py:5"]},{"type":"E","via":".ex"}"#;
        // From the root side the run is met first; of its lines the held one that grew most.
        let h = holder_with_chain(steps, &[("a.py:5", 10), ("a.py:6", 30), ("a.py:8", 99)]);
        assert_eq!(h.finding_site().as_deref(), Some("a.py:6"));
        // None of them held: the first project line of the run.
        let h = holder_with_chain(steps, &[("a.py:8", 99)]);
        assert_eq!(h.finding_site().as_deref(), Some("a.py:7"));
        // A run with no project line falls back to the frame nearer the object.
        let h = holder_with_chain(
            r#"{"type":"frame","via":"tb_frame","at":"a.py:90"},{"type":"traceback","via":"__traceback__","at":"<outside>/x.py:1","repeat":3,"ats":["<outside>/x.py:1"]},{"type":"E","via":".ex"}"#,
            &[("a.py:8", 99)],
        );
        assert_eq!(h.finding_site().as_deref(), Some("a.py:90"));
    }

    #[test]
    fn a_long_path_keeps_the_ends() {
        let steps: Vec<StepIn> = (0..12)
            .map(|k| StepIn {
                ty: format!("T{k}"),
                ..StepIn::default()
            })
            .collect();
        let s = short_steps(&steps, 6);
        let names: Vec<&str> = s.iter().map(|s| s.ty.as_str()).collect();
        assert_eq!(names, ["T0", "...", "T7", "T8", "T9", "T10", "T11"]);
        // A step with a place stays when the plain ones are enough to cut.
        let mut placed = steps.clone();
        placed[3].at = Some("a.py:3".into());
        let s = short_steps(&placed, 6);
        assert!(s.iter().any(|s| s.ty == "T3"), "{s:?}");
        let names: Vec<&str> = s.iter().map(|s| s.ty.as_str()).collect();
        assert_eq!(names, ["T0", "...", "T3", "...", "T8", "T9", "T10", "T11"]);
    }

    /// One site of `kept` sampled objects, `found` of them found, growing from `first` to `last`,
    /// with the chains `(root key, count)` of module globals.
    fn site_json(
        site: &str,
        first: u64,
        last: u64,
        found: u64,
        kept: u64,
        chains: &[(&str, u64)],
    ) -> String {
        let chains: Vec<String> = chains
            .iter()
            .map(|(k, n)| {
                format!(
                    r#"{{"count":{n},"root":{{"kind":"module-global","module":"m","key":"{k}"}},"steps":[{{"type":"int","via":".n"}}]}}"#
                )
            })
            .collect();
        format!(
            r#"{{"site":"{site}","blocks_first":{first},"blocks_last":{last},"found_tracked":{found},"found_untracked":0,"kept":{kept},"chains":[{}]}}"#,
            chains.join(",")
        )
    }

    fn holders_for(sites: &[String]) -> Vec<HolderOut> {
        let f = file(&format!(
            r#"{{"dla_pyretain":1,"sites":[{}]}}"#,
            sites.join(",")
        ));
        holders_of(&f, &red()).0
    }

    #[test]
    fn a_holder_holds_a_site_only_with_a_quarter_of_the_objects_sampled_for_it() {
        // 40 objects sampled: 10 (exactly a quarter) hold the site, 9 do not.
        let h = holders_for(&[site_json(
            "a.py:1",
            0,
            100,
            40,
            40,
            &[("big", 31), ("edge", 9)],
        )]);
        let by = |k: &str| h.iter().find(|h| h.key == format!("static:m.{k}")).unwrap();
        assert_eq!(by("big").sites.len(), 1);
        assert!(by("edge").sites.is_empty() && by("edge").minor.len() == 1);
        let h = holders_for(&[site_json(
            "a.py:1",
            0,
            100,
            40,
            40,
            &[("big", 30), ("edge", 10)],
        )]);
        let by = |k: &str| h.iter().find(|h| h.key == format!("static:m.{k}")).unwrap();
        assert_eq!(by("edge").sites.len(), 1, "a quarter holds the site");
        assert!(by("edge").minor.is_empty());
    }

    #[test]
    fn a_site_with_few_objects_found_for_its_growth_is_native_as_well() {
        // Growth 1000: 99 objects found is under a tenth, 100 is not.
        let few = holders_for(&[site_json("a.py:1", 0, 1000, 99, 40, &[("g", 40)])]);
        let g = few.iter().find(|h| h.key == "static:m.g").unwrap();
        assert!(g.sites.is_empty() && g.minor.len() == 1, "{g:?}");
        let native = few.iter().find(|h| h.key == "a.py:1").unwrap();
        assert!(native.unreferenced && native.sites.len() == 1);
        let enough = holders_for(&[site_json("a.py:1", 0, 1000, 100, 40, &[("g", 40)])]);
        assert!(enough.iter().all(|h| h.key != "a.py:1"));
        assert_eq!(enough[0].sites.len(), 1);
        // A growth under 20 blocks says nothing about the share found.
        let small = holders_for(&[site_json("a.py:1", 0, 19, 1, 1, &[("g", 1)])]);
        assert!(small.iter().all(|h| h.key != "a.py:1"));
    }

    #[test]
    fn a_holder_lists_the_types_of_what_it_keeps_not_the_links_of_its_chains() {
        let f = file(
            r#"{"dla_pyretain":1,"sites":[{"site":"a.py:1","blocks_first":0,"blocks_last":9,"kept":6,"chains":[
            {"count":4,"root":{"kind":"module-global","module":"m","key":"g"},"steps":[
              {"type":"list"},{"type":"frame","via":"tb_frame","at":"a.py:2"},{"type":"dict","via":".d"},
              {"type":"traceback","via":"__traceback__","repeat":90},{"type":"ImportError","via":".ex"}]},
            {"count":2,"root":{"kind":"module-global","module":"m","key":"g"},"steps":[
              {"type":"m.Payload"},{"type":"dict","via":".d"}]}]}]}"#,
        );
        let h = holders_of(&f, &red()).0;
        let t: Vec<(&str, u64)> = h[0].types_sorted();
        assert_eq!(t, [("list", 4), ("traceback", 4), ("m.Payload", 2)]);
    }

    #[test]
    fn one_dump_point_is_read_and_says_growth_was_not_computed() {
        let f = file(
            r#"{"dla_pyretain":1,"sites":[{"site":"a.py:1","blocks_first":null,"blocks_last":50,"found_tracked":5,"kept":5,"chains":[
            {"count":5,"root":{"kind":"module-global","module":"m","key":"g"},"steps":[{"type":"int"}]}]}]}"#,
        );
        let h = holders_of(&f, &red()).0;
        assert_eq!(h[0].sites[0].first, None);
        assert_eq!(h[0].sites[0].growth(), 50);
        assert_eq!(h[0].finding_site().as_deref(), Some("a.py:1"));
    }

    #[test]
    fn the_harness_is_read_from_the_path() {
        assert_eq!(
            harness_of(".dla/R1/harness/H2/out/p1/capture/9/py-1.retain.json"),
            Some("H2")
        );
        assert_eq!(harness_of("in/py-1.retain.json"), None);
    }
    #[test]
    fn a_holder_says_whose_it_is_from_the_agent_or_else_from_the_module_name() {
        let f = file(
            r#"{"dla_pyretain":1,"sites":[{"site":"a.py:3","blocks_first":0,"blocks_last":60,"found_tracked":50,"kept":50,"chains":[
            {"count":20,"root":{"kind":"module-global","module":"pkg.api","key":"_providers","of":"project"},"steps":[{"type":"X","via":"[0]"}]},
            {"count":10,"root":{"kind":"module-global","module":"__main__","key":"_seen","of":"tests"},"steps":[{"type":"X","via":"[0]"}]},
            {"count":10,"root":{"kind":"module-global","module":"pkg.tests.test_api","key":"_old"},"steps":[{"type":"X","via":"[0]"}]},
            {"count":5,"root":{"kind":"class-attribute","class":"pkg.api.K","attribute":"items","of":"project"},"steps":[{"type":"X","via":"[1]"}]},
            {"count":5,"root":{"kind":"frame","function":"w","at":"a.py:9"},"steps":[{"type":"X"}]}]}]}"#,
        );
        let (h, _) = holders_of(&f, &red());
        let of = |key: &str| h.iter().find(|x| x.key == key).unwrap().of;
        assert_eq!(of("static:pkg.api._providers"), Some(Of::Project));
        assert_eq!(of("static:__main__._seen"), Some(Of::Tests));
        // An older file has no `of`: the module name says the tests' and never the project's.
        assert_eq!(of("static:pkg.tests.test_api._old"), Some(Of::Tests));
        assert_eq!(of("static:pkg.api.K.items"), Some(Of::Project));
        assert_eq!(of("stack:w@a.py:9"), None);
        for (m, want) in [
            ("conftest", Some(Of::Tests)),
            ("pkg.store_test", Some(Of::Tests)),
            ("_pytest.runner", Some(Of::Tests)),
            ("unittest.case", Some(Of::Tests)),
            ("pkg.store", None),
            ("pkg.contest", None),
            ("testing", None),
        ] {
            assert_eq!(of_by_module(m), want, "{m}");
        }
    }

    #[test]
    fn a_holder_of_the_project_carries_its_line_count_and_the_site_a_finding_gets() {
        let f = file(
            r#"{"dla_pyretain":1,"sites":[
            {"site":"a.py:3","blocks_first":0,"blocks_last":60,"found_tracked":50,"kept":50,"chains":[
              {"count":50,"root":{"kind":"module-global","module":"pkg.api","key":"_providers","of":"project"},"steps":[{"type":"X","via":"[0]"}]}]},
            {"site":"a.py:8","blocks_first":0,"blocks_last":30,"found_tracked":30,"kept":30,"chains":[
              {"count":30,"root":{"kind":"module-global","module":"__main__","key":"_seen","of":"tests"},"steps":[{"type":"X","via":"[0]"}]}]}]}"#,
        );
        let (h, _) = holders_of(&f, &red());
        let project = h.iter().find(|x| x.of == Some(Of::Project)).unwrap();
        let entry = record_entry(project, 6, 8, 6);
        assert_eq!(entry["of"], "project");
        assert_eq!(entry["lines"], 1);
        assert_eq!(entry["finding_site"], "a.py:3");
        assert_eq!(raw_entry(project)["finding_site"], "a.py:3");
        // The tests' holder says whose it is and nothing else.
        let tests = h.iter().find(|x| x.of == Some(Of::Tests)).unwrap();
        let entry = record_entry(tests, 6, 8, 6);
        assert_eq!(entry["of"], "tests");
        assert!(entry.get("finding_site").is_none() && entry.get("lines").is_none());
        // The add command a holder of the project gets is the gate's literal one.
        assert_eq!(
            dla_core::measured_holders::add_command("E7", "pkg.api._providers", "a.py:3"),
            "dla ledger add --origin growth-series --evidence E7 --owner pkg.api._providers --site a.py:3 --holder static/global --region heap"
        );
    }
}
