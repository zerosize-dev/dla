//! Ledger line and record types.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    checklist::ChecklistAnswer,
    evidence::EvidenceRecord,
    finding::{Finding, FindingState, UnresolvedReason},
    ids::{EvidenceId, FindingId, OpenItemId},
};

/// A state change of a finding.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Transition {
    /// The finding.
    pub finding: FindingId,
    /// State before; must match the replayed state.
    pub from: FindingState,
    /// State after.
    pub to: FindingState,
    /// Required with `unresolved` and `suspected_static_only`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<UnresolvedReason>,
    /// Required with a reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
    /// Evidence cited by the move: added to `contradicts` for `refuted`, to
    /// `supports` otherwise.
    #[serde(default)]
    pub evidence: Vec<EvidenceId>,
    /// The dla command that made the move.
    pub by: String,
}

/// Something left to do.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct OpenItem {
    /// Identifier.
    pub id: OpenItemId,
    /// What is open.
    pub text: String,
    /// Finding it relates to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<FindingId>,
    /// Who opened it.
    pub opened_by: String,
}

/// How an open item was closed.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    /// Left unresolved with a reason and a remedy.
    Unresolved {
        /// Why.
        reason: UnresolvedReason,
        /// What would resolve it.
        remedy: String,
    },
    /// Covered by a new hypothesis.
    Hypothesis(FindingId),
}

/// Closes an open item.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct OpenItemResolved {
    /// The open item.
    pub id: OpenItemId,
    /// How it was closed.
    pub resolution: Resolution,
}

/// The body of a ledger line, tagged by `type`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Record {
    /// A tool result.
    Evidence(EvidenceRecord),
    /// A new finding, in state `candidate`.
    FindingAdded {
        /// The finding.
        finding: Finding,
    },
    /// A finding changed state.
    Transition(Transition),
    /// An open item was raised.
    OpenItem(OpenItem),
    /// An open item was closed.
    OpenItemResolved(OpenItemResolved),
    /// A checklist item was answered.
    ChecklistAnswer(ChecklistAnswer),
}

/// One line of `ledger.jsonl`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct LedgerLine {
    /// Schema version of this line.
    pub schema_version: u32,
    /// 1-based sequence number; consecutive.
    pub seq: u64,
    /// When the line was appended (RFC 3339, UTC).
    pub ts: DateTime<Utc>,
    /// The record.
    #[serde(flatten)]
    pub record: Record,
}
