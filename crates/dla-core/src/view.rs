//! The run as a report sees it, and the completeness gate.

use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    checklist::ChecklistEntry,
    coverage::CoverageManifest,
    error::Error,
    evidence::{Authority, EvidenceKind, EvidenceRecord},
    finding::FindingState,
    integrity::{self, Tamper},
    ledger::Ledger,
    slice::{self, SliceSummary},
    state::{REPORT_FILE, RunState, RunStatus},
    trend::TrendReport,
};

/// Everything the report and `dla run status` need.
#[derive(Debug, Clone)]
pub struct RunView {
    /// `state.json`.
    pub state: RunState,
    /// The replayed ledger.
    pub ledger: Ledger,
    /// `coverage.json`, when present.
    pub coverage: Option<CoverageManifest>,
    /// The standard checklist with the latest answers.
    pub checklist: Vec<ChecklistEntry>,
    /// The slice files present in the run directory (empty for a view built in memory).
    pub slices: Vec<SliceSummary>,
    /// The run directory, for a view loaded from disk.
    pub run_dir: Option<PathBuf>,
    /// The trend reports (`trend/T<n>.json`), oldest first.
    pub trends: Vec<TrendReport>,
    /// The series files present (`S1`, `S2`, ...), for a view loaded from disk.
    pub series_ids: Vec<String>,
    /// The series `dla gap` derived (source kind `gap`: the accounted and unexplained
    /// split of another series). They are the residual accounting itself, so the gates do
    /// not ask for their trend or for a disposition of their columns.
    pub gap_series: std::collections::BTreeSet<String>,
    /// The harnesses (`H1`) that some series names as its source (`source.harness`),
    /// built in the same pass over the series files as `gap_series`.
    pub harness_series: std::collections::BTreeSet<String>,
    /// Protected files other than `report.md` that no longer match their seal.
    pub tamper: Vec<Tamper>,
    /// True when `report.md` differs from the seal `dla` wrote at render time.
    pub report_modified: bool,
    /// True when the run has no `integrity/` directory (written before schema 6): nothing
    /// is sealed, which is not tampering.
    pub unsealed_run: bool,
}

impl RunView {
    /// Assembles a view; the checklist is derived from the ledger.
    pub fn new(state: RunState, ledger: Ledger, coverage: Option<CoverageManifest>) -> Self {
        let checklist = ledger.checklist();
        Self {
            state,
            ledger,
            coverage,
            checklist,
            slices: Vec::new(),
            run_dir: None,
            trends: Vec::new(),
            series_ids: Vec::new(),
            gap_series: std::collections::BTreeSet::new(),
            harness_series: std::collections::BTreeSet::new(),
            tamper: Vec::new(),
            report_modified: false,
            unsealed_run: false,
        }
    }

    /// The same view with the slice files found next to the ledger.
    pub fn with_slices(mut self, slices: Vec<SliceSummary>) -> Self {
        self.slices = slices;
        self
    }

    /// Loads `state.json`, replays `ledger.jsonl`, reads `coverage.json` if present and
    /// summarises the worker slices next to the ledger.
    pub fn load(run_dir: &Path) -> Result<Self, Error> {
        let state = RunState::load(run_dir)?;
        let ledger = Ledger::load(run_dir)?;
        let coverage = CoverageManifest::load(run_dir)?;
        let slices = slice::summaries(run_dir)?;
        Ok(Self::new(state, ledger, coverage)
            .with_slices(slices)
            .with_disk_facts(run_dir))
    }

    /// Worker mode: `worker`'s slice replayed on top of the main ledger (see
    /// [`Ledger::load_for`]). `None` loads the whole run.
    pub fn load_for(run_dir: &Path, worker: Option<&str>) -> Result<Self, Error> {
        let Some(w) = worker else {
            return Self::load(run_dir);
        };
        let state = RunState::load(run_dir)?;
        let ledger = Ledger::load_for(run_dir, Some(w))?;
        let coverage = CoverageManifest::load(run_dir)?;
        let slices = slice::summaries(run_dir)?;
        Ok(Self::new(state, ledger, coverage)
            .with_slices(slices)
            .with_disk_facts(run_dir))
    }

    /// Adds what lives next to the ledger: the trend reports, the series files and the
    /// integrity check.
    pub fn with_disk_facts(mut self, run_dir: &Path) -> Self {
        self.run_dir = Some(run_dir.to_path_buf());
        self.trends = TrendReport::load_all(run_dir);
        self.series_ids = crate::series::list_ids(run_dir);
        self.gap_series.clear();
        self.harness_series.clear();
        for id in &self.series_ids {
            let Ok(f) = crate::series::SeriesFile::load(run_dir, id) else {
                continue;
            };
            if f.source.kind == crate::series::SeriesSourceKind::Gap {
                self.gap_series.insert(id.clone());
            }
            if let Some(h) = f.source.harness {
                self.harness_series.insert(h);
            }
        }
        self.unsealed_run = !integrity::is_sealed_run(run_dir);
        for t in integrity::verify(run_dir) {
            if t.file == REPORT_FILE {
                self.report_modified = true;
            } else {
                self.tamper.push(t);
            }
        }
        self
    }

    /// Slices that hold lines the main ledger has not merged yet.
    pub fn unmerged_slices(&self) -> Vec<&SliceSummary> {
        self.slices
            .iter()
            .filter(|s| s.lines > self.ledger.merged_lines(&s.worker))
            .collect()
    }
}

/// What kind of thing blocks completeness.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockingKind {
    /// A finding in `candidate` or `under_test`.
    Finding,
    /// An open item without a resolution.
    OpenItem,
    /// A checklist item without an answer.
    ChecklistItem,
    /// The run is aborted or incomplete.
    Run,
    /// A protected file no longer matches the seal `dla` wrote.
    Integrity,
}

/// One thing that must be dispositioned, with the command that does it.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct BlockingItem {
    /// What kind of item.
    pub kind: BlockingKind,
    /// Finding, open item or checklist item ID (the run ID for `run`).
    pub id: String,
    /// Short description.
    pub description: String,
    /// The command that clears it.
    pub fix: String,
}

/// Number of findings in each state.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StateCounts {
    /// `candidate`.
    pub candidate: usize,
    /// `under_test`.
    pub under_test: usize,
    /// `confirmed`.
    pub confirmed: usize,
    /// `strong`.
    pub strong: usize,
    /// `refuted`.
    pub refuted: usize,
    /// `suspected_static_only`.
    pub suspected_static_only: usize,
    /// `unresolved`.
    pub unresolved: usize,
}

impl StateCounts {
    /// This count less `other`, state by state (saturating).
    pub fn minus(&self, other: &StateCounts) -> StateCounts {
        StateCounts {
            candidate: self.candidate.saturating_sub(other.candidate),
            under_test: self.under_test.saturating_sub(other.under_test),
            confirmed: self.confirmed.saturating_sub(other.confirmed),
            strong: self.strong.saturating_sub(other.strong),
            refuted: self.refuted.saturating_sub(other.refuted),
            suspected_static_only: self
                .suspected_static_only
                .saturating_sub(other.suspected_static_only),
            unresolved: self.unresolved.saturating_sub(other.unresolved),
        }
    }

    /// The count for one state.
    pub fn get(&self, s: FindingState) -> usize {
        match s {
            FindingState::Candidate => self.candidate,
            FindingState::UnderTest => self.under_test,
            FindingState::Confirmed => self.confirmed,
            FindingState::Strong => self.strong,
            FindingState::Refuted => self.refuted,
            FindingState::SuspectedStaticOnly => self.suspected_static_only,
            FindingState::Unresolved => self.unresolved,
        }
    }

    /// Adds one finding in state `s`.
    pub fn bump(&mut self, s: FindingState) {
        let slot = match s {
            FindingState::Candidate => &mut self.candidate,
            FindingState::UnderTest => &mut self.under_test,
            FindingState::Confirmed => &mut self.confirmed,
            FindingState::Strong => &mut self.strong,
            FindingState::Refuted => &mut self.refuted,
            FindingState::SuspectedStaticOnly => &mut self.suspected_static_only,
            FindingState::Unresolved => &mut self.unresolved,
        };
        *slot += 1;
    }
}

/// Counts shown in the report header.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatusCounts {
    /// All findings.
    pub enumerated: usize,
    /// Findings in confirmed, strong, refuted, suspected_static_only or unresolved.
    pub dispositioned: usize,
    /// Per-state counts of all findings, leak hypotheses and non-leak explanations alike.
    pub by_state: StateCounts,
    /// Per-state counts of the non-leak explanations (`N-*` siblings) alone; the leak
    /// hypotheses are `by_state` less this.
    #[serde(default)]
    pub non_leak: StateCounts,
    /// Open items without a resolution.
    pub open_items: usize,
    /// Checklist items without an answer.
    pub checklist_missing: usize,
    /// Checklist items in total.
    pub checklist_total: usize,
    /// True when something was measured: see [`measured`].
    pub measured: bool,
}

/// The result of the completeness gate.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct ReportStatus {
    /// True iff nothing blocks.
    pub complete: bool,
    /// Blocking items in a stable order: findings, open items, checklist, run.
    pub blocking: Vec<BlockingItem>,
    /// Counts.
    pub counts: StatusCounts,
}

/// The arguments that disposition a finding or open item as unresolved, with the reason
/// the evidence supports: `inconclusive` when it was measured, else `not_exercised`.
/// `time_budget` is never pre-filled here: it is true only when the deadline is near.
fn resolve_args(reason: crate::finding::UnresolvedReason) -> String {
    format!("--as unresolved --reason {reason} --remedy \"<text>\"")
}

/// The fix text for a run that is aborted or incomplete, by the reason it was stamped.
fn incomplete_fix(view: &RunView) -> String {
    let reason = view.state.incomplete_reason.as_deref().unwrap_or("");
    if view.state.status == RunStatus::Incomplete
        && reason.starts_with(crate::merge::UNFINISHED_REASON)
    {
        "let the unfinished workers end (dla --worker <name> ledger done), then dla ledger merge"
            .to_owned()
    } else if view.state.status == RunStatus::Incomplete
        && reason.starts_with(crate::gate::STOP_BLOCKED_REASON)
    {
        "the stop hook stamped this run: the report lists what was still open. Disposition those items (dla ledger resolve <F> --as unresolved --reason <reason> --remedy \"<text>\") and run dla report, or dla run init to start a new run"
            .to_owned()
    } else {
        "dla run init".to_owned()
    }
}

/// Applies the gate: COMPLETE iff nothing blocks. An `unresolved` finding with a
/// valid reason does not block. A finding merged into another worker's (`finding_merged`)
/// is neither enumerated nor dispositioned: its twin carries it.
pub fn status(view: &RunView) -> ReportStatus {
    let mut blocking = Vec::new();
    let mut counts = StatusCounts::default();

    for f in view.ledger.findings() {
        // A duplicate merged into another worker's finding is counted there.
        if view.ledger.merged_into(&f.id).is_some() {
            continue;
        }
        counts.enumerated += 1;
        counts.by_state.bump(f.state);
        if f.polarity() == crate::finding::Polarity::NonLeak {
            counts.non_leak.bump(f.state);
        }
        if f.state.is_dispositioned() {
            counts.dispositioned += 1;
        } else {
            blocking.push(BlockingItem {
                kind: BlockingKind::Finding,
                id: f.id.to_string(),
                description: format!("finding {} is {} and has no disposition", f.id, f.state),
                fix: format!(
                    "dla ledger resolve {} {}",
                    f.id,
                    resolve_args(view.ledger.supported_reason(&f.id))
                ),
            });
        }
    }
    for o in view.ledger.open_items() {
        if o.resolution.is_none() {
            counts.open_items += 1;
            blocking.push(BlockingItem {
                kind: BlockingKind::OpenItem,
                id: o.item.id.to_string(),
                description: format!("open item {}: {}", o.item.id, o.item.text),
                fix: format!(
                    "dla ledger resolve {} {}",
                    o.item.id,
                    resolve_args(view.ledger.open_item_supported_reason(&o.item.id))
                ),
            });
        }
    }
    counts.checklist_total = view.checklist.len();
    for c in &view.checklist {
        if c.answer.is_none() {
            counts.checklist_missing += 1;
            blocking.push(BlockingItem {
                kind: BlockingKind::ChecklistItem,
                id: c.item.id.clone(),
                description: format!(
                    "checklist item {} is unanswered: {}",
                    c.item.id, c.item.question
                ),
                fix: format!(
                    "dla ledger resolve {} --status answered --evidence E1",
                    c.item.id
                ),
            });
        }
    }
    if matches!(
        view.state.status,
        RunStatus::Aborted | RunStatus::Incomplete
    ) {
        let why = view
            .state
            .incomplete_reason
            .as_deref()
            .map(|r| format!(": {r}"))
            .unwrap_or_default();
        blocking.push(BlockingItem {
            kind: BlockingKind::Run,
            id: view.state.run_id.clone(),
            description: format!("run is {}{why}", view.state.status),
            fix: incomplete_fix(view),
        });
    }
    for t in &view.tamper {
        // A write killed before its seal is not an edit: a merge (or the worker's next
        // write) settles it.
        let interrupted = view
            .run_dir
            .as_deref()
            .is_some_and(|d| crate::integrity::interrupted(d, &t.file));
        let (what, fix) = if interrupted {
            (
                "a dla write of this file was interrupted before it was sealed",
                "dla ledger merge (a worker's own dla ledger done also settles its slice)"
                    .to_owned(),
            )
        } else {
            (
                "",
                "restore the file from a copy, or, if the change is intended, dla run reseal --reason \"<why the file changed outside dla>\""
                    .to_owned(),
            )
        };
        let note = if what.is_empty() {
            String::new()
        } else {
            format!(" ({what})")
        };
        blocking.push(BlockingItem {
            kind: BlockingKind::Integrity,
            id: t.file.clone(),
            description: format!("tamper: {}{note}", t.describe()),
            fix,
        });
    }
    counts.measured = measured(view).any();
    ReportStatus {
        complete: blocking.is_empty(),
        blocking,
        counts,
    }
}

/// What the run measured, as opposed to read. Every count comes from ledger evidence
/// whose authority is `test`, `live` or `dump` (run-config and probe records never
/// count), plus the coverage manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Measured {
    /// Harness runs: measured `workload` records (what `dla harness run` writes; workload
    /// discovery and `dla harness gen` records are static and do not count), or the
    /// coverage workers that record iterations or exercised entry points when there are
    /// more.
    pub harness_runs: usize,
    /// Series: the run's non-gap series files (a derived `gap` series is not counted).
    /// `dla trend` and the summarizers write several growth-series records per series, so
    /// records are not counted; only a view with no series files at all (not loaded from
    /// a run directory) falls back to its measured `growth-series` and `unit-count-series`
    /// records.
    pub series: usize,
    /// Captures: measured records of a capture kind (retention paths, allocation stacks,
    /// histogram diffs, blocked-stack profiles, unreachable-at-exit and should-be-dead
    /// sets, non-heap readings, attached artifacts). Residual rows, discriminator results,
    /// interventions and the split `dla gap` writes for a series are analysis, not captures, and are not counted.
    pub captures: usize,
    /// True when a coverage worker row records iterations or exercised entry points.
    pub exercised: bool,
    /// Measured ledger records of any kind (the counts above keep only some of them).
    pub records: usize,
}

impl Measured {
    /// True when anything was measured.
    pub fn any(&self) -> bool {
        self.records > 0 || self.exercised
    }
}

/// True for a record that measures something: authority `test`, `live` or `dump`, and a
/// kind that is neither `run-config` nor `probe`. A `workload` record counts here (the
/// harness ran); whether it says anything about one finding is a separate question.
pub fn is_measurement(e: &EvidenceRecord) -> bool {
    matches!(
        e.authority,
        Authority::Test | Authority::Live | Authority::Dump
    ) && !matches!(e.kind, EvidenceKind::RunConfig | EvidenceKind::Probe)
}

/// What the run measured. A pure function of the view, so the report, `dla run status`
/// and the stop hook agree. Static hits, census holders, the probe, workload discovery and
/// a generated (never run) harness measure nothing.
pub fn measured(view: &RunView) -> Measured {
    let mut m = Measured::default();
    let mut series_records = 0;
    for e in view.ledger.evidence() {
        if !is_measurement(e) {
            continue;
        }
        m.records += 1;
        match e.kind {
            EvidenceKind::Workload => m.harness_runs += 1,
            EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries => series_records += 1,
            EvidenceKind::Residual | EvidenceKind::Discriminator | EvidenceKind::Intervention => {}
            // `dla gap S` records the split of a series as a non-heap record naming the
            // derived gap series: analysis of a series already counted, not a reading.
            EvidenceKind::NonHeap
                if e.result
                    .get("derived")
                    .and_then(|d| d.as_str())
                    .is_some_and(|d| view.gap_series.contains(d)) => {}
            _ => m.captures += 1,
        }
    }
    let files = view
        .series_ids
        .iter()
        .filter(|id| !view.gap_series.contains(*id))
        .count();
    m.series = if view.series_ids.is_empty() {
        series_records
    } else {
        files
    };
    let workers = view.coverage.as_ref().map_or(0, |c| {
        c.workers
            .values()
            .filter(|w| w.iterations.is_some_and(|n| n > 0) || !w.entry_points.exercised.is_empty())
            .count()
    });
    m.exercised = workers > 0;
    m.harness_runs = m.harness_runs.max(workers);
    m
}
