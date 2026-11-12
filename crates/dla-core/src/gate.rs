//! Stage and final gates: `dla gate check`.
//!
//! [`evaluate`] is a pure function of a [`RunView`]: the same list feeds `dla gate check`,
//! the stop hooks and `dla run status`, so there is one source of truth for what is still
//! missing. Stages are cumulative: `--stage X` also requires everything the stages before
//! it require. `--final` requires everything the report needs.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use dla_stats::{GateVerdict, TrendShape};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    checklist::AnswerStatus,
    coverage::{BackendStatus, CoverageManifest},
    evidence::EvidenceKind,
    finding::{Finding, FindingState, Polarity, UnresolvedReason},
    ids::FindingId,
    record::Record,
    series::ColumnKind,
    state::{EntryPath, Stage},
    trend::{ColumnTrend, TrendReport},
    view::{BlockingItem, BlockingKind, RunView, status},
};

/// Start of the `incomplete_reason` the stop hook writes when it let a run stop at the
/// third stop attempt with no change to the ledger (the first two were blocked).
pub const STOP_BLOCKED_REASON: &str = "stop hook let the stop through after";

/// How many missing items `dla gate check` prints; the full list goes to a file.
pub const SHOWN_MISSING: usize = 5;

/// A run the `dla` rules do not govern before it has a ledger of its own: artifact and
/// symptom runs have no probe, sweep or series.
fn probes_and_sweeps(view: &RunView) -> bool {
    matches!(view.state.entry, EntryPath::Repository | EntryPath::Process)
}

/// What a gate checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateTarget {
    /// The requirements of one stage (cumulative).
    Stage(Stage),
    /// Everything the report needs.
    Final,
}

/// One thing that keeps a gate from passing.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    /// What kind of item (`finding`, `open_item`, `checklist_item`, `run`, `coverage`, ...).
    pub kind: String,
    /// The finding, open item, checklist item or run ID the item is about.
    pub id: String,
    /// Short description.
    pub description: String,
    /// The command that clears it.
    pub fix: String,
}

/// The result of a gate.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct GateReport {
    /// True when nothing is missing.
    pub ok: bool,
    /// The stage name, or `final`.
    pub stage: String,
    /// Number of missing items in total.
    pub missing_total: usize,
    /// The first [`SHOWN_MISSING`] items; the full list is in `<run>/gate/<stage>.json`.
    pub missing: Vec<Missing>,
    /// Where the full list is.
    pub pointer: String,
}

impl GateTarget {
    /// The name used in output and file names: the stage, or `final`.
    pub fn name(self) -> &'static str {
        match self {
            GateTarget::Stage(s) => s.as_str(),
            GateTarget::Final => "final",
        }
    }

    /// True when the target includes the requirements of `stage`.
    fn includes(self, stage: Stage) -> bool {
        match self {
            GateTarget::Stage(s) => s.rank() >= stage.rank(),
            GateTarget::Final => true,
        }
    }
}

/// The stored file for the full list: `gate/<name>.json` relative to the run directory.
pub fn pointer_for(target: GateTarget) -> String {
    format!("gate/{}.json", target.name())
}

/// Evaluates a gate on `view`. `worker` limits the check to one worker's slice.
pub fn evaluate(view: &RunView, target: GateTarget, worker: Option<&str>) -> GateReport {
    let all = missing(view, target, worker);
    GateReport {
        ok: all.is_empty(),
        stage: target.name().to_owned(),
        missing_total: all.len(),
        missing: all.into_iter().take(SHOWN_MISSING).collect(),
        pointer: pointer_for(target),
    }
}

fn kind_name(k: BlockingKind) -> String {
    serde_json::to_value(k)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn from_blocking(b: BlockingItem) -> Missing {
    Missing {
        kind: kind_name(b.kind),
        id: b.id,
        description: b.description,
        fix: b.fix,
    }
}

struct Out(Vec<Missing>);

impl Out {
    fn add(
        &mut self,
        kind: &str,
        id: impl Into<String>,
        description: impl Into<String>,
        fix: impl Into<String>,
    ) {
        self.0.push(Missing {
            kind: kind.to_owned(),
            id: id.into(),
            description: description.into(),
            fix: fix.into(),
        });
    }
}

/// The full list of what a gate still needs, in a stable order: the blockers of
/// [`status`] first (findings, open items, checklist, run), then the stage and final
/// requirements.
pub fn missing(view: &RunView, target: GateTarget, worker: Option<&str>) -> Vec<Missing> {
    match worker {
        Some(w) => worker_missing(view, target, w),
        None => core_missing(view, target),
    }
}

fn core_missing(view: &RunView, target: GateTarget) -> Vec<Missing> {
    let mut out = Out(Vec::new());
    let is_final = target == GateTarget::Final;
    let base = status(view).blocking;
    for b in base {
        let keep = is_final
            || match b.kind {
                BlockingKind::Run | BlockingKind::ChecklistItem | BlockingKind::Integrity => true,
                BlockingKind::Finding | BlockingKind::OpenItem => false,
            };
        if keep {
            out.0.push(from_blocking(b));
        }
    }
    if target.includes(Stage::Intake) {
        intake(view, &mut out);
    }
    if target.includes(Stage::Probe) && probes_and_sweeps(view) {
        probe(view, &mut out);
    }
    if target.includes(Stage::Sweep) && probes_and_sweeps(view) {
        sweep(view, &mut out);
    }
    if target.includes(Stage::Classify) {
        classify(view, is_final, &mut out);
    }
    if target.includes(Stage::Confirm) {
        confirm(view, &mut out);
    }
    if target.includes(Stage::Attribute) {
        attribute(view, &mut out);
    }
    if target.includes(Stage::RuleOut) {
        rule_out(view, &mut out);
    }
    if is_final {
        final_items(view, &mut out);
    }
    out.0
}

// ------------------------------------------------------------------- intake

fn intake(view: &RunView, out: &mut Out) {
    // (d) the checklist, re-validated at gate time: the cited evidence must exist in the
    // merged ledger, an unresolved answer needs its reason and remedy.
    for c in &view.checklist {
        let Some(a) = &c.answer else { continue };
        let id = c.item.id.clone();
        match a.status {
            AnswerStatus::Answered | AnswerStatus::Partial => {
                if a.evidence.is_empty() {
                    out.add(
                        "checklist_item",
                        id.clone(),
                        format!("checklist item {id} is {} and cites no evidence", a.status),
                        format!("dla ledger resolve {id} --status answered --evidence E1"),
                    );
                    continue;
                }
                if let Some(bad) = a
                    .evidence
                    .iter()
                    .find(|e| view.ledger.evidence_by_id(e).is_none())
                {
                    out.add(
                        "checklist_item",
                        id.clone(),
                        format!("checklist item {id} cites {bad}, which is not in the ledger"),
                        format!("dla ledger resolve {id} --status answered --evidence E1"),
                    );
                }
            }
            AnswerStatus::Unresolved => {
                if a.reason.is_none() || a.remedy.as_deref().is_none_or(|r| r.trim().is_empty()) {
                    out.add(
                        "checklist_item",
                        id.clone(),
                        format!("checklist item {id} is unresolved without a reason and a remedy"),
                        format!(
                            "dla ledger resolve {id} --status unresolved --reason tool_missing --remedy \"<text>\""
                        ),
                    );
                }
            }
        }
    }
    let consent_unresolved = view
        .checklist
        .iter()
        .find(|c| c.item.id == "intake.consent")
        .and_then(|c| c.answer.as_ref())
        .is_some_and(|a| a.status == AnswerStatus::Unresolved);
    if view.state.consent.is_none() && !consent_unresolved {
        out.add(
            "consent",
            view.state.run_id.clone(),
            "no consent is recorded for this run",
            "dla consent --tiers 1,2",
        );
    }
}

// -------------------------------------------------------------------- probe

fn probe(view: &RunView, out: &mut Out) {
    if !view
        .ledger
        .evidence()
        .any(|e| e.kind == EvidenceKind::Probe)
    {
        out.add(
            "probe",
            view.state.run_id.clone(),
            "no probe evidence is recorded",
            "dla probe",
        );
    }
    let workers = view.coverage.as_ref().map_or(0, |c| c.workers.len());
    // No runtime was detected: the answer to `intake.runtimes` may say so as unresolved.
    let runtimes_unresolved = view
        .checklist
        .iter()
        .find(|c| c.item.id == "intake.runtimes")
        .and_then(|c| c.answer.as_ref())
        .is_some_and(|a| a.status == AnswerStatus::Unresolved);
    if workers == 0 && !runtimes_unresolved {
        out.add(
            "coverage",
            view.state.run_id.clone(),
            "coverage.json has no runtime row: no runtime was detected or probed",
            "dla probe",
        );
    }
    if let Some(cov) = &view.coverage {
        for (w, wc) in &cov.workers {
            for (b, bc) in &wc.backends {
                let silent = match bc.status {
                    BackendStatus::Ran => false,
                    BackendStatus::Blocked => {
                        bc.reason.as_deref().is_none_or(|r| r.trim().is_empty())
                    }
                    BackendStatus::Missing => {
                        bc.reason.as_deref().is_none_or(|r| r.trim().is_empty())
                            && bc
                                .install_hint
                                .as_deref()
                                .is_none_or(|r| r.trim().is_empty())
                    }
                };
                if silent {
                    out.add(
                        "coverage",
                        format!("{w}/{b}"),
                        format!(
                            "backend {b} of {w} is {} with no reason or install hint",
                            bc.status
                        ),
                        format!("dla sweep --lang {w}"),
                    );
                }
            }
        }
    }
}

// -------------------------------------------------------------------- sweep

/// The runtimes `dla probe` detected from the project's sources and build files, over
/// every probe of the run. `None` when no probe evidence names any (the probe gate says
/// so): every runtime with a coverage row is then held to the sweep.
fn probed_runtimes(view: &RunView) -> Option<BTreeSet<String>> {
    let found: BTreeSet<String> = view
        .ledger
        .evidence()
        .filter(|e| e.kind == EvidenceKind::Probe)
        .filter_map(|e| e.result.get("runtimes").and_then(|v| v.as_array()))
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    (!found.is_empty()).then_some(found)
}

fn sweep(view: &RunView, out: &mut Out) {
    let Some(cov) = &view.coverage else { return };
    // A capture tool (valgrind, heaptrack) adds the `native` runtime to the coverage of a
    // Rust or JVM project: only a runtime the probe found in the sources needs its own
    // census and sweep.
    let probed = probed_runtimes(view);
    for (w, wc) in cov
        .workers
        .iter()
        .filter(|(w, _)| probed.as_ref().is_none_or(|p| p.contains(*w)))
    {
        if !wc.backends.contains_key("census") {
            out.add(
                "coverage",
                format!("{w}/census"),
                format!("no census has run for runtime {w}"),
                format!("dla census --lang {w}"),
            );
        }
        // The built-in `dla` rules run in every sweep, so their row says a sweep happened.
        // `dla probe` does not write that row: it is not a sweep.
        if !wc.backends.contains_key("dla") {
            out.add(
                "coverage",
                format!("{w}/sweep"),
                format!("no sweep has run for runtime {w}"),
                format!("dla sweep --lang {w}"),
            );
        }
    }
    for a in accounting(view).iter().filter(|a| a.hits > a.enumerated) {
        out.add(
            "accounting",
            format!("{}/{}", a.worker, a.backend),
            format!(
                "backend {} of {} reported {} hit(s) and the ledger accounts for {} (hits without a candidate)",
                a.backend, a.worker, a.hits, a.enumerated
            ),
            format!("dla sweep --lang {}   (a hit needs a candidate or a recorded duplicate)", a.worker),
        );
    }
}

// ---------------------------------------------------------------- accounting

/// Disposition accounting of one backend of one worker, derived from the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendAccount {
    /// The coverage worker key (the runtime).
    pub worker: String,
    /// The backend (`census` for the census).
    pub backend: String,
    /// What the backend reported in `coverage.json` (hits).
    pub hits: u64,
    /// Findings of this backend plus recorded duplicates.
    pub enumerated: u64,
    /// Those in a dispositioned state; a duplicate counts when its twin does.
    pub dispositioned: u64,
    /// The findings of this backend that are not dispositioned.
    pub open: Vec<FindingId>,
}

fn backend_of(origin: &str) -> Option<String> {
    if origin == "census" {
        return Some("census".to_owned());
    }
    let rest = origin.strip_prefix("detector:")?;
    Some(rest.split_once('/').map_or(rest, |(b, _)| b).to_owned())
}

/// The coverage worker key of a finding: the runtime of its worker prefix, else the
/// `lang` of the sweep or census evidence it cites, else the only worker there is.
fn worker_key(view: &RunView, cov: &CoverageManifest, f: &Finding) -> Option<String> {
    if let Some(p) = f.id.prefix() {
        let rt = view.state.worker(p).map_or(p, |w| w.runtime.as_str());
        return Some(rt.to_owned());
    }
    for e in &f.supports {
        if let Some(rec) = view.ledger.evidence_by_id(e)
            && matches!(
                rec.kind,
                EvidenceKind::StaticHit | EvidenceKind::CensusHolder
            )
            && let Some(l) = rec.result.get("lang").and_then(|v| v.as_str())
        {
            return Some(l.to_owned());
        }
    }
    (cov.workers.len() == 1)
        .then(|| cov.workers.keys().next().cloned())
        .flatten()
}

/// The accounting of every sweep and census backend in `coverage.json`: findings with
/// that origin backend plus recorded duplicates against the findings that are dispositioned.
/// Backends that are not a source of findings (sampling, harness tools) are not listed.
pub fn accounting(view: &RunView) -> Vec<BackendAccount> {
    let Some(cov) = &view.coverage else {
        return Vec::new();
    };
    // Which (runtime, backend) pairs produced hit evidence at all.
    let mut sources: BTreeSet<(String, String)> = BTreeSet::new();
    for e in view.ledger.evidence() {
        let lang = e.result.get("lang").and_then(|v| v.as_str());
        match (e.kind, lang) {
            (EvidenceKind::StaticHit, Some(l)) => {
                sources.insert((l.to_owned(), e.tool.name.clone()));
            }
            (EvidenceKind::CensusHolder, Some(l)) => {
                sources.insert((l.to_owned(), "census".to_owned()));
            }
            _ => {}
        }
    }
    // (runtime, backend) -> (findings, dispositioned findings, open ids)
    let mut by: BTreeMap<(String, String), (u64, u64, Vec<FindingId>)> = BTreeMap::new();
    for f in view.ledger.findings() {
        let Some(backend) = backend_of(f.origin.as_str()) else {
            continue;
        };
        let Some(w) = worker_key(view, cov, f) else {
            continue;
        };
        let twin = view.ledger.merged_into(&f.id);
        let disp = match twin {
            Some(c) => view
                .ledger
                .finding(c)
                .is_some_and(|c| c.state.is_dispositioned()),
            None => f.state.is_dispositioned(),
        };
        let slot = by.entry((w, backend)).or_default();
        slot.0 += 1;
        if disp {
            slot.1 += 1;
        } else {
            slot.2.push(f.id.clone());
        }
    }
    let mut out = Vec::new();
    for (w, wc) in &cov.workers {
        for (b, bc) in &wc.backends {
            let key = (w.clone(), b.clone());
            if !sources.contains(&key) {
                continue;
            }
            let (n, d, open) = by.remove(&key).unwrap_or_default();
            // A duplicate counts as dispositioned when its twin does; a backend with an
            // open finding cannot say which twin that is, so none is counted then.
            let dup_disp = if open.is_empty() { bc.duplicates } else { 0 };
            out.push(BackendAccount {
                worker: w.clone(),
                backend: b.clone(),
                hits: bc.enumerated,
                enumerated: n + bc.duplicates,
                dispositioned: d + dup_disp,
                open,
            });
        }
    }
    out
}

/// Writes the derived `dispositioned` counts into `coverage.json`.
pub fn write_back_accounting(
    run_dir: &std::path::Path,
    accounts: &[BackendAccount],
) -> Result<(), crate::error::Error> {
    if accounts.is_empty() {
        return Ok(());
    }
    let current = CoverageManifest::load(run_dir)?;
    let unchanged = current.as_ref().is_some_and(|m| {
        accounts.iter().all(|a| {
            m.workers
                .get(&a.worker)
                .and_then(|w| w.backends.get(&a.backend))
                .is_some_and(|b| b.dispositioned == a.dispositioned)
        })
    });
    if unchanged {
        return Ok(());
    }
    CoverageManifest::update(run_dir, |m| {
        for a in accounts {
            if let Some(b) = m
                .workers
                .get_mut(&a.worker)
                .and_then(|w| w.backends.get_mut(&a.backend))
            {
                b.dispositioned = a.dispositioned;
            }
        }
    })?;
    Ok(())
}

// ----------------------------------------------------------------- classify

/// A trend column that is not flat: its gate passed, or its shape is steady or stair.
pub fn non_flat(c: &ColumnTrend) -> bool {
    c.gate == GateVerdict::Pass || matches!(c.shape, TrendShape::Steady | TrendShape::Stair)
}

fn evidence_for_column(view: &RunView, t: &TrendReport, c: &ColumnTrend) -> Option<String> {
    let want = match c.kind {
        ColumnKind::Gauge => EvidenceKind::GrowthSeries,
        ColumnKind::Counter => EvidenceKind::UnitCountSeries,
    };
    view.ledger
        .evidence()
        .filter(|e| e.kind == want)
        .find(|e| e.result.get("trend").and_then(|v| v.as_str()) == Some(t.id.as_str()))
        .map(|e| e.evidence_id.to_string())
}

/// True for a column that measures the whole process or a runtime total (`os.*`, `cgroup.*`,
/// `fd.*`, `jvm.*`, `go.*`, `node.*`, `py.*` and `native.*` totals, `metric:*/total`, the gap
/// split). Such a column is never tied to a finding (the tie rule): no single holder or site
/// explains it. A site-group column (`name@file:line`), a holder or probe column and a
/// per-type count are not whole-process columns.
pub fn is_whole_process(name: &str) -> bool {
    let base = name.strip_suffix(".pre_gc").unwrap_or(name);
    if base.contains('@') {
        return false;
    }
    if base.starts_with("jvm.class:") || base.starts_with("py.type:") {
        return false;
    }
    if base.starts_with("metric:") {
        return base.ends_with("/total");
    }
    [
        "os.", "cgroup.", "fd.", "gap.", "jvm.", "go.", "node.", "py.", "native.",
    ]
    .iter()
    .any(|p| base.starts_with(p))
}

/// True for a per-type instance count of a JVM heap (`jvm.class:<name>.count`). It belongs to
/// the heap, but the tie rule ties a type to a finding only through its accumulation class or its field's element type, and never for the
/// generic types (`byte[]`, `String`, collection nodes), so most of these columns have no
/// finding to tie to. The heap residual rows account for them, as they do for a total.
pub fn is_type_count(name: &str) -> bool {
    let base = name.strip_suffix(".pre_gc").unwrap_or(name);
    !base.contains('@') && base.starts_with("jvm.class:")
}

/// The findings tied to a trend column: the one it was probed for, its candidates, and
/// the findings that cite the growth evidence `dla trend` wrote for it. A whole-process
/// column has none.
fn tied_findings<'a>(view: &'a RunView, t: &TrendReport, c: &ColumnTrend) -> Vec<&'a Finding> {
    if is_whole_process(&c.name) {
        return Vec::new();
    }
    let mut ids: Vec<FindingId> = Vec::new();
    ids.extend(c.finding.iter().cloned());
    ids.extend(c.candidates.iter().cloned());
    if let Some(ev) = evidence_for_column(view, t, c)
        && let Ok(eid) = ev.parse::<crate::ids::EvidenceId>()
    {
        ids.extend(
            view.ledger
                .findings()
                .filter(|f| f.supports.contains(&eid))
                .map(|f| f.id.clone()),
        );
    }
    // `dla alloc-test` writes a trend for its plateau and purge test and a discriminator
    // that names the trend and the columns it tested: a finding that cites that
    // discriminator is tied to the columns it names.
    for e in view.ledger.evidence().filter(|e| {
        e.kind == EvidenceKind::Discriminator
            && e.result.get("trend").and_then(|v| v.as_str()) == Some(t.id.as_str())
            && e.result
                .get("columns")
                .and_then(|v| v.as_array())
                .is_some_and(|cols| {
                    cols.iter()
                        .any(|x| x.get("name").and_then(|n| n.as_str()) == Some(c.name.as_str()))
                })
    }) {
        ids.extend(
            view.ledger
                .findings()
                .filter(|f| {
                    f.supports.contains(&e.evidence_id) || f.contradicts.contains(&e.evidence_id)
                })
                .map(|f| f.id.clone()),
        );
    }
    ids.sort();
    ids.dedup();
    ids.iter().filter_map(|i| view.ledger.finding(i)).collect()
}

/// The number in a trend id (`T3` is 3).
fn trend_number(id: &str) -> Option<u64> {
    id.strip_prefix('T')?.parse().ok()
}

/// True when a later trend of the same series analysed the column again and that newer
/// measurement is itself non-flat: the gate holds the column to the newer trend. A later
/// trend in which the column is flat (a shorter window, other options) does not retire the
/// older growth.
fn superseded(view: &RunView, t: &TrendReport, c: &ColumnTrend) -> bool {
    let Some(mine) = trend_number(&t.id) else {
        return false;
    };
    view.trends.iter().any(|u| {
        u.series == t.series
            && trend_number(&u.id).is_some_and(|n| n > mine)
            && u.columns.iter().any(|d| d.name == c.name && non_flat(d))
    })
}

/// The residual rows that account for column `c` of `series`: the rows of the column's own
/// region (all of the series' rows for a column of no region), or all of the series' rows
/// when none is of that region. A row that does not record its series covers nothing.
fn column_rows<'a>(
    cov: &'a crate::coverage::CoverageManifest,
    series: &str,
    c: &ColumnTrend,
) -> Vec<&'a crate::coverage::ResidualRow> {
    use crate::{finding::Region, series::SeriesRegion};
    let all: Vec<_> = cov
        .residual
        .iter()
        .filter(|r| r.series.as_deref() == Some(series))
        .collect();
    let region = match crate::series::column_meta(&c.name).region {
        SeriesRegion::Heap => Some(Region::Heap),
        SeriesRegion::OffHeap => Some(Region::OffHeap),
        SeriesRegion::NonHeap => Some(Region::NonHeap),
        SeriesRegion::Process => None,
    };
    let own: Vec<_> = all
        .iter()
        .copied()
        .filter(|r| region.is_none_or(|g| r.region == g))
        .collect();
    if own.is_empty() { all } else { own }
}

/// True when the residual accounting covers a whole-process column: `dla gap --residual`
/// wrote rows for the series the column's trend analysed (`series`; a row that does not
/// record its series covers nothing), and the rows of the column's region (all of that
/// series' rows for a column of no region) are under their threshold or have a resolved
/// open item.
fn residual_covers(view: &RunView, series: &str, c: &ColumnTrend) -> bool {
    let Some(cov) = &view.coverage else {
        return false;
    };
    let rows = column_rows(cov, series, c);
    if rows.is_empty() {
        return false;
    }
    rows.iter().all(|r| {
        r.residual <= r.threshold
            || r.open_item
                .as_ref()
                .is_some_and(|id| residual_item_resolved(view, id, &r.region.to_string()))
    })
}

/// The fix for a whole-process column (or per-type count) that the residual accounting of
/// `series` does not yet cover. Before the series has a gap row, `dla gap` writes it. Once
/// it has one, running `dla gap` again only reopens or keeps items, so the fix lists the
/// open items to resolve instead (a resolved item makes the row cover the column).
fn residual_fix(view: &RunView, series: &str, c: &ColumnTrend) -> String {
    let gap = format!(
        "dla gap {series} --residual   (rows under the threshold, or over it with a resolved open item, account for the column)"
    );
    let Some(cov) = &view.coverage else {
        return gap;
    };
    let rows = column_rows(cov, series, c);
    if rows.is_empty() {
        return gap;
    }
    let open: Vec<String> = rows
        .iter()
        .filter(|r| r.residual > r.threshold)
        .filter_map(|r| r.open_item.as_ref().map(|id| (id, r)))
        .filter(|(id, r)| !residual_item_resolved(view, id, &r.region.to_string()))
        .map(|(id, _)| id.to_string())
        .collect();
    if open.is_empty() {
        // A row over its threshold without an item: only `dla gap` raises it.
        return gap;
    }
    open.iter()
        .map(|id| {
            format!(
                "dla ledger resolve {id} --by <F>   or   dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\""
            )
        })
        .collect::<Vec<_>>()
        .join("   ;   ")
}

/// True for the total of a summarize-made series (`metric:<category>/total`): the sum of the
/// group columns the same snapshots produced.
pub fn is_group_total(name: &str) -> bool {
    let base = name.strip_suffix(".pre_gc").unwrap_or(name);
    base.starts_with("metric:") && base.ends_with("/total")
}

/// The group columns of a trend that are not flat, and those of them that are not yet
/// tied to a finding (a dispositioned one for the final gate).
fn group_columns_state<'a>(
    view: &RunView,
    t: &'a TrendReport,
    is_final: bool,
) -> (usize, Vec<&'a str>) {
    let mut non_flat_groups = 0;
    let mut open = Vec::new();
    for g in t
        .columns
        .iter()
        .filter(|g| g.name.starts_with("group:") && non_flat(g))
    {
        non_flat_groups += 1;
        let tied = tied_findings(view, t, g);
        let ok = if is_final {
            tied.iter().any(|f| f.state.is_dispositioned())
        } else {
            !tied.is_empty()
        };
        if !(ok || superseded(view, t, g) || covered_by_open_item(view, t, g)) {
            open.push(g.name.as_str());
        }
    }
    (non_flat_groups, open)
}

/// True when a resolved open item mentions the column and its trend or series.
fn covered_by_open_item(view: &RunView, t: &TrendReport, c: &ColumnTrend) -> bool {
    view.ledger.open_items().any(|o| {
        o.resolution.is_some()
            && o.item.text.contains(&c.name)
            && (o.item.text.contains(&t.id) || o.item.text.contains(&t.series))
    })
}

fn classify(view: &RunView, is_final: bool, out: &mut Out) {
    if !probes_and_sweeps(view) {
        return;
    }
    // Every series was analysed.
    let analysed: BTreeSet<&str> = view.trends.iter().map(|t| t.series.as_str()).collect();
    for s in &view.series_ids {
        // A series `dla gap` derived is the residual accounting of another series; the
        // residual rows of that series (and their open items) govern it.
        if !analysed.contains(s.as_str()) && !view.gap_series.contains(s) {
            out.add(
                "series",
                s.clone(),
                format!("series {s} has no trend analysis"),
                format!("dla trend {s}"),
            );
        }
    }
    // (c) every non-flat column is tied to a finding (dispositioned for the final gate) or
    // covered by a resolved open item.
    for t in &view.trends {
        // The trend of a series `dla gap` derived measures the accounting, not the target:
        // its gap.unexplained growth is governed by the residual rows of the source series.
        if view.gap_series.contains(&t.series) {
            continue;
        }
        for c in t.columns.iter().filter(|c| non_flat(c)) {
            if is_whole_process(&c.name) {
                // Never tied to a finding. The classify stage leaves it to the final gate,
                // which takes the residual accounting, an open item, or, for the total of
                // a summarize-made series, its groups: the total is their sum, so it is
                // accounted for when every non-flat group column of the trend is tied to a
                // dispositioned finding (and at least one group is non-flat).
                let groups = is_group_total(&c.name).then(|| group_columns_state(view, t, true));
                if !is_final
                    || groups
                        .as_ref()
                        .is_some_and(|(n, open)| *n > 0 && open.is_empty())
                    || residual_covers(view, &t.series, c)
                    || superseded(view, t, c)
                    || covered_by_open_item(view, t, c)
                {
                    continue;
                }
                if let Some((n, open)) = groups {
                    let (why, fix) = if n == 0 {
                        (
                            "is the total of its group columns, none of which is non-flat, so nothing accounts for it".to_owned(),
                            format!(
                                "dla trend {}   (after dla summarize on more snapshots; the groups must show the growth)",
                                t.series
                            ),
                        )
                    } else {
                        (
                            format!(
                                "is never tied to a finding; it is the sum of its group columns, which account for it once each non-flat one is tied to a dispositioned finding (not yet: {})",
                                open.join(", ")
                            ),
                            format!(
                                "tie the group columns of {} first (their own trend_column items): dla ledger add --origin growth-series --evidence <growth evidence of the group column>, then dla ledger resolve <F>",
                                t.id
                            ),
                        )
                    };
                    out.add(
                        "trend_column",
                        format!("{}:{}", t.id, c.name),
                        format!("column {} of {} ({}) {why}", c.name, t.id, c.shape_name()),
                        fix,
                    );
                    continue;
                }
                out.add(
                    "trend_column",
                    format!("{}:{}", t.id, c.name),
                    format!(
                        "column {} of {} ({}) measures the whole process, is not flat and is never tied to a finding; it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item",
                        c.name,
                        t.id,
                        c.shape_name()
                    ),
                    residual_fix(view, &t.series, c),
                );
                continue;
            }
            let tied = tied_findings(view, t, c);
            let ok = if is_final {
                tied.iter().any(|f| f.state.is_dispositioned())
            } else {
                !tied.is_empty()
            };
            if ok || superseded(view, t, c) || covered_by_open_item(view, t, c) {
                continue;
            }
            // A per-type count that ties to no finding is accounted for by the residual rows
            // of its series (the classify stage leaves that to the final gate, as it does for
            // a whole-process column). A count tied only to undispositioned findings needs
            // their disposition, below.
            if is_type_count(&c.name) && tied.is_empty() {
                if !is_final || residual_covers(view, &t.series, c) {
                    continue;
                }
                out.add(
                    "trend_column",
                    format!("{}:{}", t.id, c.name),
                    format!(
                        "column {} of {} ({}) counts the instances of one type and is tied to no finding (a generic type never ties); it is accounted for by the residual rows, which are missing or over their threshold without a resolved open item",
                        c.name,
                        t.id,
                        c.shape_name()
                    ),
                    residual_fix(view, &t.series, c),
                );
                continue;
            }
            let fix = match evidence_for_column(view, t, c) {
                Some(e) => format!(
                    "dla ledger add --origin growth-series --evidence {e}   # column {}",
                    c.name
                ),
                None => format!("dla trend {}", t.series),
            };
            let why = if tied.is_empty() {
                "is tied to no finding"
            } else {
                "is tied only to findings that have no disposition"
            };
            out.add(
                "trend_column",
                format!("{}:{}", t.id, c.name),
                format!(
                    "column {} of {} ({}) is not flat and {why}",
                    c.name,
                    t.id,
                    c.shape_name()
                ),
                fix,
            );
        }
    }
}

trait ShapeName {
    fn shape_name(&self) -> String;
}

impl ShapeName for ColumnTrend {
    fn shape_name(&self) -> String {
        serde_json::to_value(self.shape)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default()
    }
}

// ------------------------------------------------------------------ confirm

fn confirm(view: &RunView, out: &mut Out) {
    for f in view.ledger.findings() {
        if view.ledger.merged_into(&f.id).is_some() {
            continue;
        }
        if f.state == FindingState::UnderTest {
            out.add(
                "finding",
                f.id.to_string(),
                format!("finding {} is under test and has no result", f.id),
                format!(
                    "dla ledger confirm {}   or   dla ledger refute {}   (with --evidence)",
                    f.id, f.id
                ),
            );
        }
    }
    // A finding tied to growth that passed the gate must have been tested.
    for t in &view.trends {
        for c in t.columns.iter().filter(|c| c.gate == GateVerdict::Pass) {
            for f in tied_findings(view, t, c) {
                if f.state == FindingState::Candidate && view.ledger.merged_into(&f.id).is_none() {
                    out.add(
                        "finding",
                        f.id.to_string(),
                        format!(
                            "finding {} is tied to growth that passed the gate ({} of {}) and was never tested",
                            f.id, c.name, t.id
                        ),
                        format!(
                            "dla ledger test {}   or   dla ledger resolve {} --as unresolved --reason not_exercised --remedy \"<text>\"",
                            f.id, f.id
                        ),
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------- attribute

fn is_leak_side(f: &Finding) -> bool {
    f.polarity() == Polarity::Leak
}

fn confirmed_leaks(view: &RunView) -> impl Iterator<Item = &Finding> {
    view.ledger.findings().filter(|f| {
        matches!(f.state, FindingState::Confirmed | FindingState::Strong)
            && is_leak_side(f)
            && view.ledger.merged_into(&f.id).is_none()
    })
}

fn attribute(view: &RunView, out: &mut Out) {
    for f in confirmed_leaks(view) {
        let has_where = f.site.is_some() || f.owner.is_some();
        if f.holder.is_none() || !has_where {
            let mut missing = Vec::new();
            if f.holder.is_none() {
                missing.push("a holder");
            }
            if !has_where {
                missing.push("a site or owner");
            }
            out.add(
                "attribution",
                f.id.to_string(),
                format!(
                    "finding {} is {} and has no {}",
                    f.id,
                    f.state,
                    missing.join(" and ")
                ),
                format!(
                    "dla ledger annotate {} --holder <kind> --owner <field> --evidence E..",
                    f.id
                ),
            );
        }
    }
}

// ------------------------------------------------------------------ rule out

fn rule_out(view: &RunView, out: &mut Out) {
    for f in view.ledger.findings() {
        if view.ledger.merged_into(&f.id).is_some()
            || !is_leak_side(f)
            || !matches!(
                f.state,
                FindingState::Confirmed | FindingState::Strong | FindingState::UnderTest
            )
        {
            continue;
        }
        let root = f.root_id();
        let siblings: Vec<&Finding> = view
            .ledger
            .findings()
            .filter(|s| s.parent.as_ref() == Some(&root) && s.polarity() == Polarity::NonLeak)
            .collect();
        if siblings.is_empty() {
            out.add(
                "rule_out",
                f.id.to_string(),
                format!("finding {} has no non-leak sibling hypothesis", f.id),
                format!(
                    "dla ledger add --sibling-of {root} --origin hypothesis --hypothesis N-WARM --refute-if \"<result>\" --evidence E.."
                ),
            );
            continue;
        }
        for s in siblings.iter().filter(|s| !s.state.is_dispositioned()) {
            out.add(
                "rule_out",
                s.id.to_string(),
                format!(
                    "sibling {} ({}) of {} has no disposition",
                    s.id,
                    s.hypothesis.as_deref().unwrap_or("N-*"),
                    f.id
                ),
                format!("dla alloc-test <harness> --finding {root}   then   dla ledger refute {} --evidence <discriminator id>", s.id),
            );
        }
    }
}

// -------------------------------------------------------------------- final

/// True when the open item a residual row names is resolved. A worker raises `O1` in its
/// slice and the merge renames it `<worker>/O1`, while the coverage row keeps the plain
/// id: an unprefixed id also matches the same number under a worker prefix when that
/// item is about the same region and resolved.
fn residual_item_resolved(view: &RunView, id: &crate::ids::OpenItemId, region: &str) -> bool {
    if view
        .ledger
        .open_item(id)
        .is_some_and(|o| o.resolution.is_some())
    {
        return true;
    }
    if id.prefix().is_some() {
        return false;
    }
    let tag = format!(" in {region} (");
    view.ledger.open_items().any(|o| {
        o.item.id.prefix().is_some()
            && o.item.id.number() == id.number()
            && o.resolution.is_some()
            && o.item.text.contains(&tag)
    })
}

fn final_items(view: &RunView, out: &mut Out) {
    unrun_harness_items(view, None, out);
    // (b) per backend: every hit has a candidate and every candidate a disposition. The
    // undispositioned findings themselves are listed by `status`.
    for a in accounting(view) {
        if a.hits <= a.enumerated && a.dispositioned != a.enumerated && a.open.is_empty() {
            out.add(
                "accounting",
                format!("{}/{}", a.worker, a.backend),
                format!(
                    "backend {} of {}: {} of {} enumerated item(s) are dispositioned",
                    a.backend, a.worker, a.dispositioned, a.enumerated
                ),
                "dla ledger resolve <F> --as unresolved --reason time_budget --remedy \"<text>\"",
            );
        }
    }
    // (e) residual rows over their threshold have a resolved open item.
    if let Some(cov) = &view.coverage {
        for r in cov.residual.iter().filter(|r| r.residual > r.threshold) {
            let resolved = r
                .open_item
                .as_ref()
                .is_some_and(|id| residual_item_resolved(view, id, &r.region.to_string()));
            if !resolved {
                out.add(
                    "residual",
                    r.region.to_string(),
                    format!(
                        "residual growth in {} ({:.3} {}, threshold {:.3}) has no resolved open item",
                        r.region, r.residual, r.unit, r.threshold
                    ),
                    match &r.open_item {
                        Some(id) => format!(
                            "dla ledger resolve {id} --by <F>   or   dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\""
                        ),
                        None => "dla gap --residual   then resolve the open item it raises".to_owned(),
                    },
                );
            }
        }
    }
    // Every confirmed or strong leak-side finding is explained (schema 6 ledgers).
    for f in confirmed_leaks(view) {
        if view.ledger.explanation(&f.id).is_none() {
            out.add(
                "explanation",
                f.id.to_string(),
                format!("finding {} is {} and has no stored explanation", f.id, f.state),
                if view.state.entry == EntryPath::Artifact {
                    format!(
                        "dla ledger explain {} --tag inferred --removal not-found --text \"<owner, holder, what adds to it, no remover found>\"   (an artifact run has no source: no --adds)",
                        f.id
                    )
                } else {
                    format!(
                        "dla ledger explain {} --tag inferred --adds FILE:LINE --removal never --text \"<site, holder, who adds, why nothing removes>\"",
                        f.id
                    )
                },
            );
        }
    }
    // Every confirmed or strong leak-side finding has a label decision: an archetype ID, or
    // UNCLASSIFIED with a note (the only label with archetypes off).
    for f in confirmed_leaks(view) {
        if view.ledger.label_record(&f.id).is_none() {
            // Another finding of the same leak (same owner or site) already carries a
            // label: one leak is one finding, so the duplicate is merged into it.
            let twins = crate::duplicate::labelled_twins(&view.ledger, f);
            let fix = match twins.first() {
                Some(g) => format!(
                    "dla ledger duplicate {} --of {}   ({} is the same leak as {}, which is labelled: merge it); if it is another leak: dla ledger label {} --suggest, then dla ledger label {} --as <ID>, or dla ledger label {} --as UNCLASSIFIED --note \"<holder and mechanism in words>\"",
                    f.id, g.id, f.id, g.id, f.id, f.id, f.id
                ),
                None => format!(
                    "dla ledger label {} --suggest, then dla ledger label {} --as <ID>, or dla ledger label {} --as UNCLASSIFIED --note \"<holder and mechanism in words>\"",
                    f.id, f.id, f.id
                ),
            };
            out.add(
                "label",
                f.id.to_string(),
                format!(
                    "finding {} is {} and has no label decision (an archetype ID, or UNCLASSIFIED with a note)",
                    f.id, f.state
                ),
                fix,
            );
        }
    }
    // No slice left unmerged, no torn tail.
    for s in view.unmerged_slices() {
        out.add(
            "slice",
            s.worker.clone(),
            format!(
                "slice of worker {} has {} line(s) and {} merged",
                s.worker,
                s.lines,
                view.ledger.merged_lines(&s.worker)
            ),
            "dla ledger merge",
        );
    }
    // A torn line a merge has already seen and dropped is accounted for.
    for s in view.slices.iter().filter(|s| {
        s.torn_tail
            && !(view.ledger.merged_torn_tail(&s.worker)
                && view.ledger.merged_lines(&s.worker) == s.lines)
    }) {
        out.add(
            "slice",
            s.worker.clone(),
            format!(
                "slice of worker {} ends in a torn line (the worker was killed mid-write)",
                s.worker
            ),
            "dla ledger merge",
        );
    }
}

// ------------------------------------------------------------ unrun harnesses

/// The rule for a generated harness that was never run applies only while more than
/// `max(HARNESS_MIN_LEFT_SECS, HARNESS_MIN_LEFT_PERCENT % of the budget)` is left: below
/// that a finding closed with `time_budget` is honest.
pub const HARNESS_MIN_LEFT_SECS: u64 = 600;
/// See [`HARNESS_MIN_LEFT_SECS`].
pub const HARNESS_MIN_LEFT_PERCENT: u64 = 20;

/// A harness ID (`H1`).
pub type HarnessId = String;

/// A generated harness that was never run, with the budget left that makes that matter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrunHarness {
    /// `H1`.
    pub id: HarnessId,
    /// Its runtime (`jvm`).
    pub runtime: String,
    /// Whole minutes of budget left (the nearest of the run's and the worker's deadline).
    pub minutes_left: u64,
    /// Whole minutes of the run's budget.
    pub minutes_total: u64,
    /// The driver's DLA-HOOK bodies are still the template's.
    pub hooks_unfilled: bool,
    /// The reason of its last attempt when one exists but is not a try (`hooks_unfilled`,
    /// `interrupted`), for the wording of a refusal.
    pub last_attempt: Option<String>,
}

impl UnrunHarness {
    /// What happened to the harness, for a refusal or a missing item: "was generated and
    /// never run", or what the last attempt shows.
    pub fn state(&self) -> &'static str {
        match self.last_attempt.as_deref() {
            Some(crate::harness::ATTEMPT_HOOKS_UNFILLED) => {
                "was never run with its DLA-HOOK bodies filled in"
            }
            Some(crate::harness::ATTEMPT_INTERRUPTED) => {
                "was interrupted before it measured anything"
            }
            _ => "was generated and never run",
        }
    }

    /// The command that clears the rule, for a refusal or a missing item.
    pub fn fix(&self, run_id: &str) -> String {
        let h = &self.id;
        let run = if self.hooks_unfilled {
            let junit = if self.runtime == "jvm" {
                "; for a JVM project with JUnit 5 tests, dla harness gen --lang jvm --junit runs an existing test in a loop instead"
            } else {
                ""
            };
            format!(
                "dla harness run {h} (a driver harness: fill the DLA-HOOK bodies in .dla/{run_id}/harness/{h}/driver/ first{junit})"
            )
        } else {
            format!("dla harness run {h}")
        };
        format!(
            "{run}. If {h} cannot run, record why: dla ledger resolve intake.workload --status unresolved --reason <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy \"<what blocks {h}>\". If only the user can unblock it (services, credentials, data), ask the user before closing the run."
        )
    }
}

/// True for the reasons that close a finding without exercising it: the ones the rule
/// covers. `env_unavailable`, `build_fails`, `tool_missing` and `privilege_declined` name a
/// blocker and are never refused.
pub fn closes_without_exercise(reason: UnresolvedReason) -> bool {
    matches!(
        reason,
        UnresolvedReason::NoHarness | UnresolvedReason::NotExercised | UnresolvedReason::TimeBudget
    )
}

/// The reasons that record a blocker on `intake.workload`.
fn is_blocker_reason(reason: UnresolvedReason) -> bool {
    matches!(
        reason,
        UnresolvedReason::EnvUnavailable
            | UnresolvedReason::ToolMissing
            | UnresolvedReason::BuildFails
            | UnresolvedReason::PrivilegeDeclined
    )
}

/// The harness directories of the run, `H1`, `H2`, ... in numeric order.
fn harness_ids(run_dir: &Path) -> Vec<(u32, HarnessId)> {
    let Ok(rd) = std::fs::read_dir(run_dir.join(crate::harness::HARNESS_DIR)) else {
        return Vec::new();
    };
    let mut ids: Vec<(u32, HarnessId)> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            Some((crate::series::numbered('H', &name)?, name))
        })
        .collect();
    ids.sort();
    ids
}

/// Where the ledger recorded that harness `h` was generated: the sequence number of its
/// first `workload` evidence without a `series` key (a run's evidence has one).
fn generated_at(view: &RunView, h: &str) -> Option<u64> {
    view.ledger.lines().iter().find_map(|l| match &l.record {
        Record::Evidence(e)
            if e.kind == EvidenceKind::Workload
                && crate::harness::harness_id_of(&e.result) == Some(h)
                && e.result.get("series").is_none() =>
        {
            Some(l.seq)
        }
        _ => None,
    })
}

/// True when `intake.workload` was answered `unresolved` with a blocker reason and a
/// remedy after harness `h` was generated.
fn blocker_recorded(view: &RunView, h: &str) -> bool {
    let after = generated_at(view, h).unwrap_or(0);
    view.ledger.lines().iter().any(|l| {
        l.seq > after
            && matches!(&l.record, Record::ChecklistAnswer(a)
                if a.item == "intake.workload"
                    && a.status == AnswerStatus::Unresolved
                    && a.reason.is_some_and(is_blocker_reason)
                    && a.remedy.as_deref().is_some_and(|r| !r.trim().is_empty()))
    })
}

/// The attempt record of harness `h`, when one was written and parses.
fn attempt_of(run_dir: &Path, h: &str) -> Option<crate::harness::HarnessAttempt> {
    let text = std::fs::read_to_string(crate::harness::HarnessAttempt::path_in(run_dir, h)).ok()?;
    serde_json::from_str(&text).ok()
}

/// True when harness `h` measured something: an attempt that ran (or ran partly), a
/// `run-*.json`, or a series whose source is the harness (`view.harness_series`, read
/// once with the series files).
fn harness_ran(view: &RunView, run_dir: &Path, h: &str) -> bool {
    use crate::harness::AttemptOutcome;
    if attempt_of(run_dir, h).is_some_and(|a| a.outcome != AttemptOutcome::Failed) {
        return true;
    }
    let dir = crate::harness::HarnessManifest::dir_in(run_dir, h);
    let has_run_file = std::fs::read_dir(&dir).is_ok_and(|rd| {
        rd.flatten().any(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.starts_with("run-") && n.ends_with(".json")
        })
    });
    has_run_file || view.harness_series.contains(h)
}

/// True when harness `h` was tried: it measured something, or `dla harness run` wrote a
/// failed attempt for it. A failure on a driver whose hooks were never filled in is not a
/// try (the template's own error says nothing about the workload), and neither is a run
/// the caller stopped (`interrupted`).
fn harness_tried(view: &RunView, run_dir: &Path, h: &str) -> bool {
    harness_ran(view, run_dir, h)
        || attempt_of(run_dir, h).is_some_and(|a| !attempt_is_no_try(a.reason.as_deref()))
}

/// An attempt reason that says nothing about the workload: the driver's template hooks
/// failed, or the caller stopped the run before it measured.
fn attempt_is_no_try(reason: Option<&str>) -> bool {
    matches!(
        reason,
        Some(crate::harness::ATTEMPT_HOOKS_UNFILLED | crate::harness::ATTEMPT_INTERRUPTED)
    )
}

/// The first harness of `runtime` that `dla harness gen` wrote and nothing tried: its
/// `harness.json` loads, the ledger has its generation evidence (a clone made by
/// `dla intervene` has none), it was not tried (see `harness_tried`), and the blocker is
/// not recorded (`intake.workload` answered `unresolved` with a blocker reason and a remedy
/// after the harness was generated). `None` when another harness of the runtime measured
/// something (an abandoned driver does not hold the rule), and for a view without a run
/// directory. Pure of the clock; see [`harness_not_run`] for the budget part.
pub fn unrun_harness(view: &RunView, runtime: &str) -> Option<HarnessId> {
    let run_dir = view.run_dir.as_deref()?;
    let generated: Vec<HarnessId> = harness_ids(run_dir)
        .into_iter()
        .filter_map(|(_, id)| {
            let m = crate::harness::HarnessManifest::load(run_dir, &id).ok()?;
            (m.runtime.as_str() == runtime && generated_at(view, &id).is_some()).then_some(id)
        })
        .collect();
    if generated.iter().any(|id| harness_ran(view, run_dir, id)) {
        return None;
    }
    generated
        .into_iter()
        .find(|id| !harness_tried(view, run_dir, id) && !blocker_recorded(view, id))
}

/// Minutes left to the nearest deadline (the worker's, in worker mode, and the run's) and
/// the budget in minutes, when the rule applies: the run is still going and more than
/// the floor is left.
fn budget_for_rule(view: &RunView, worker: Option<&str>) -> Option<(u64, u64)> {
    use crate::state::RunStatus;
    if matches!(
        view.state.status,
        RunStatus::Aborted | RunStatus::Incomplete
    ) {
        return None;
    }
    let now = crate::state::now_utc();
    let mut deadline = view.state.budget.deadline;
    if let Some(w) = worker.and_then(|w| view.state.worker(w)) {
        deadline = deadline.min(w.deadline);
    }
    let left = u64::try_from((deadline - now).num_seconds()).unwrap_or(0);
    let total = view.state.budget.seconds;
    let floor = HARNESS_MIN_LEFT_SECS.max(total * HARNESS_MIN_LEFT_PERCENT / 100);
    (left > floor).then_some((left / 60, total / 60))
}

/// The unrun harness of `runtime` while the rule applies (the run is going and more than
/// the floor of budget is left), else `None`. `worker` is the worker whose slice `view`
/// is, in worker mode.
pub fn harness_not_run(
    view: &RunView,
    runtime: &str,
    worker: Option<&str>,
) -> Option<UnrunHarness> {
    let (minutes_left, minutes_total) = budget_for_rule(view, worker)?;
    let id = unrun_harness(view, runtime)?;
    let hooks_unfilled = view
        .run_dir
        .as_deref()
        .is_some_and(|d| crate::harness::hooks_unfilled(d, &id));
    let last_attempt = view
        .run_dir
        .as_deref()
        .and_then(|d| attempt_of(d, &id))
        .and_then(|a| a.reason)
        .filter(|r| attempt_is_no_try(Some(r)));
    Some(UnrunHarness {
        id,
        last_attempt,
        hooks_unfilled,
        runtime: runtime.to_owned(),
        minutes_left,
        minutes_total,
    })
}

/// The runtime of a finding for the rule: [`worker_key`], else the only runtime that has a
/// harness.
pub fn finding_runtime(view: &RunView, f: &Finding) -> Option<String> {
    finding_runtime_with(view, f, &mut None)
}

/// [`finding_runtime`] with the only-runtime-that-has-a-harness fallback computed once per
/// caller: `only` holds it after the first use (`Some(None)`: there is no single runtime).
fn finding_runtime_with(
    view: &RunView,
    f: &Finding,
    only: &mut Option<Option<String>>,
) -> Option<String> {
    let empty;
    let cov = match &view.coverage {
        Some(c) => c,
        None => {
            empty = CoverageManifest::new(None);
            &empty
        }
    };
    worker_key(view, cov, f).or_else(|| {
        only.get_or_insert_with(|| {
            let run_dir = view.run_dir.as_deref()?;
            let mut rts: BTreeSet<String> = BTreeSet::new();
            for (_, id) in harness_ids(run_dir) {
                if let Ok(m) = crate::harness::HarnessManifest::load(run_dir, &id) {
                    rts.insert(m.runtime.as_str().to_owned());
                }
            }
            (rts.len() == 1).then(|| rts.into_iter().next()).flatten()
        })
        .clone()
    })
}

/// One missing item per unrun harness that has a finding closed without exercise while the
/// rule applies. `worker` limits the findings to a worker's own. The answer for a runtime
/// is computed once, not once per finding: the cost is one pass over the findings plus one
/// evaluation per runtime.
fn unrun_harness_items(view: &RunView, worker: Option<&str>, out: &mut Out) {
    let mut only: Option<Option<String>> = None;
    let mut by_runtime: BTreeMap<String, Option<UnrunHarness>> = BTreeMap::new();
    let mut counts: BTreeMap<HarnessId, (UnrunHarness, usize)> = BTreeMap::new();
    for f in view.ledger.findings() {
        let closed = matches!(
            f.state,
            FindingState::Unresolved | FindingState::SuspectedStaticOnly
        ) && f.reason.is_some_and(closes_without_exercise);
        if !closed {
            continue;
        }
        if let Some(w) = worker
            && f.id.prefix() != Some(w)
        {
            continue;
        }
        let Some(rt) = finding_runtime_with(view, f, &mut only) else {
            continue;
        };
        let unrun = by_runtime
            .entry(rt.clone())
            .or_insert_with(|| harness_not_run(view, &rt, worker));
        if let Some(u) = unrun {
            counts.entry(u.id.clone()).or_insert((u.clone(), 0)).1 += 1;
        }
    }
    let mut items: Vec<_> = counts.into_values().collect();
    items.sort_by_key(|(u, _)| crate::series::numbered('H', &u.id));
    for (u, n) in items {
        out.add(
            "harness_not_run",
            u.id.clone(),
            format!(
                "harness {} ({}) {}, {n} finding(s) are closed with no_harness, not_exercised or time_budget, and {} of {} budget minutes remain",
                u.id, u.runtime, u.state(), u.minutes_left, u.minutes_total
            ),
            u.fix(&view.state.run_id),
        );
    }
}

// ------------------------------------------------------------------- worker

/// The gate for one worker's slice: its own findings and open items are dispositioned,
/// the backends it ran account for their hits, and for the final target its slice is done.
fn worker_missing(view: &RunView, target: GateTarget, w: &str) -> Vec<Missing> {
    let mut out = Out(Vec::new());
    let mine = |id: &str| id.split_once('/').is_some_and(|(p, _)| p == w);
    for b in status(view).blocking {
        let keep = match b.kind {
            BlockingKind::Finding | BlockingKind::OpenItem => mine(&b.id),
            BlockingKind::ChecklistItem | BlockingKind::Integrity => false,
            BlockingKind::Run => true,
        };
        if keep {
            out.0.push(from_blocking(b));
        }
    }
    // Time budget: the worker's own deadline and the run's.
    let now = crate::state::now_utc();
    let deadline = view
        .state
        .worker(w)
        .map(|e| (format!("worker {w}"), e.deadline));
    for (who, at) in deadline
        .into_iter()
        .chain([("run".to_owned(), view.state.budget.deadline)])
    {
        if now > at {
            out.add(
                "run",
                view.state.run_id.clone(),
                format!(
                    "the {who} time budget is spent (deadline {}); no new stage may start",
                    at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                ),
                format!(
                    "dla --worker {w} ledger resolve <F> --as unresolved --reason time_budget --remedy \"<text>\", then dla --worker {w} ledger done"
                ),
            );
            break;
        }
    }
    let runtime = view.state.worker(w).map_or(w, |e| e.runtime.as_str());
    for a in accounting(view).into_iter().filter(|a| a.worker == runtime) {
        if a.hits > a.enumerated {
            out.add(
                "accounting",
                format!("{}/{}", a.worker, a.backend),
                format!(
                    "backend {} reported {} hit(s) and the ledger accounts for {}",
                    a.backend, a.hits, a.enumerated
                ),
                format!("dla --worker {w} sweep --lang {}", a.worker),
            );
        }
    }
    if target == GateTarget::Final {
        unrun_harness_items(view, Some(w), &mut out);
        let done = view.slices.iter().any(|s| s.worker == w && s.done)
            || view.ledger.slice_info(w).is_some_and(|s| s.done.is_some());
        if !done {
            out.add(
                "slice",
                w.to_owned(),
                format!("worker {w} has not ended its slice"),
                format!("dla --worker {w} ledger done"),
            );
        }
    }
    out.0
}
