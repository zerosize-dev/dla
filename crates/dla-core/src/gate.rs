//! Stage and final gates: `dla gate check`.
//!
//! [`evaluate`] is a pure function of a [`RunView`]: the same list feeds `dla gate check`,
//! the stop hooks and `dla run status`, so there is one source of truth for what is still
//! missing. Stages are cumulative: `--stage X` also requires everything the stages before
//! it require. `--final` requires everything the report needs.

use std::collections::{BTreeMap, BTreeSet};

use dla_stats::{GateVerdict, TrendShape};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    checklist::AnswerStatus,
    coverage::{BackendStatus, CoverageManifest},
    evidence::EvidenceKind,
    finding::{Finding, FindingState, Polarity},
    ids::FindingId,
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

/// True when the residual accounting covers a whole-process column: `dla gap --residual`
/// wrote rows for the series the column's trend analysed (`series`; a row that does not
/// record its series covers nothing), and the rows of the column's region (all of that
/// series' rows for a column of no region) are under their threshold or have a resolved
/// open item.
fn residual_covers(view: &RunView, series: &str, c: &ColumnTrend) -> bool {
    use crate::{finding::Region, series::SeriesRegion};
    let Some(cov) = &view.coverage else {
        return false;
    };
    let all: Vec<_> = cov
        .residual
        .iter()
        .filter(|r| r.series.as_deref() == Some(series))
        .collect();
    if all.is_empty() {
        return false;
    }
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
    let rows = if own.is_empty() { all } else { own };
    rows.iter().all(|r| {
        r.residual <= r.threshold
            || r.open_item
                .as_ref()
                .is_some_and(|id| residual_item_resolved(view, id, &r.region.to_string()))
    })
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
                    format!(
                        "dla gap {} --residual   (rows under the threshold, or over it with a resolved open item, account for the column)",
                        t.series
                    ),
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
                    "dla gap --residual   then resolve the open item it raises",
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
