//! The run as a report sees it, and the completeness gate.

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    checklist::ChecklistEntry,
    coverage::CoverageManifest,
    error::Error,
    finding::FindingState,
    ledger::Ledger,
    slice::{self, SliceSummary},
    state::{RunState, RunStatus},
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
        Ok(Self::new(state, ledger, coverage).with_slices(slices))
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

    fn bump(&mut self, s: FindingState) {
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
    /// Per-state counts.
    pub by_state: StateCounts,
    /// Open items without a resolution.
    pub open_items: usize,
    /// Checklist items without an answer.
    pub checklist_missing: usize,
    /// Checklist items in total.
    pub checklist_total: usize,
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

const RESOLVE_FINDING: &str = "--as unresolved --reason time_budget --remedy \"<text>\"";

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
        if f.state.is_dispositioned() {
            counts.dispositioned += 1;
        } else {
            blocking.push(BlockingItem {
                kind: BlockingKind::Finding,
                id: f.id.to_string(),
                description: format!("finding {} is {} and has no disposition", f.id, f.state),
                fix: format!("dla ledger resolve {} {RESOLVE_FINDING}", f.id),
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
                fix: format!("dla ledger resolve {} {RESOLVE_FINDING}", o.item.id),
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
            fix: "dla run init".to_owned(),
        });
    }
    ReportStatus {
        complete: blocking.is_empty(),
        blocking,
        counts,
    }
}
