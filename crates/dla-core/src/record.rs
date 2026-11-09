//! Ledger line and record types.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    checklist::ChecklistAnswer,
    evidence::EvidenceRecord,
    finding::{Finding, FindingState, Holder, Region, UnresolvedReason},
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

/// Writers and removers found for a finding's holder, and its holder, owner, region and
/// site when none is recorded yet. It does not change the finding's state; the report lists
/// the sites under the finding.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct FindingAnnotation {
    /// The finding.
    pub finding: FindingId,
    /// `file:line` sites that add to the holder.
    #[serde(default)]
    pub writers: Vec<String>,
    /// `file:line` sites that remove from the holder.
    #[serde(default)]
    pub removers: Vec<String>,
    /// Free-text note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// What holds the memory; applied only when the finding has no holder yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<Holder>,
    /// Owning field or container; applied only when the finding has no owner yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Memory region; applied only when the finding has no region yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
    /// `file:line` of the finding; applied only when the finding has no site yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Growth evidence that backs a change of a holder already set (schema 5): a heap-graph
    /// `growth-series` with a passing column tied to the finding whose `holder_kind` is
    /// `holder`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceId>,
    /// The holder this annotation replaced; written by the ledger, an audit trail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_holder: Option<Holder>,
}

impl FindingAnnotation {
    /// An annotation of `finding` with nothing in it yet.
    pub fn new(finding: FindingId) -> Self {
        Self {
            finding,
            writers: Vec::new(),
            removers: Vec::new(),
            note: None,
            holder: None,
            owner: None,
            region: None,
            site: None,
            evidence: Vec::new(),
            replaced_holder: None,
        }
    }
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
    /// Writers and removers were recorded for a finding (no state change).
    FindingAnnotated(FindingAnnotation),
    /// A worker's slice began (first line of every `ledger.<worker>.jsonl`).
    SliceStarted(SliceStarted),
    /// A worker finished its slice.
    SliceDone(SliceDone),
    /// `dla ledger merge` copied lines of a slice into this ledger.
    SliceMerged(SliceMerged),
    /// Two workers' findings are the same (site, holder): one is a duplicate of the other.
    FindingMerged(FindingMerged),
    /// An independent verifier looked at a finding's bundle.
    Verification(Verification),
}

/// The first line of a worker slice.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct SliceStarted {
    /// The worker.
    pub worker: String,
    /// The runtime it covers (the worker name when it was not registered).
    pub runtime: String,
    /// Length of the main ledger when the slice began: the slice may cite main IDs
    /// created by lines up to this one.
    pub base_seq: u64,
    /// The worker's deadline (the run deadline when it was not registered).
    pub deadline: DateTime<Utc>,
}

/// The last line of a finished slice.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct SliceDone {
    /// The worker.
    pub worker: String,
    /// IDs of the slice's findings and open items that have no disposition.
    #[serde(default)]
    pub open: Vec<String>,
}

/// Written by `dla ledger merge` after it copied lines of a slice.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct SliceMerged {
    /// The worker.
    pub worker: String,
    /// Slice lines merged so far, in total (a re-run appends only lines after this).
    pub lines: u64,
    /// SHA-256 (lowercase hex) of the slice bytes those lines occupy.
    pub sha256: String,
    /// True when a torn final line was dropped.
    pub torn_tail: bool,
    /// True when the merged lines include `slice_done`.
    pub done: bool,
}

/// Replay semantics: the duplicate is excluded from the counts and the report's finding
/// list, and its supporting evidence is added to the canonical finding.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct FindingMerged {
    /// The finding that stays.
    pub canonical: FindingId,
    /// The finding of another worker with the same site and holder.
    pub duplicate: FindingId,
}

/// What an independent verifier concluded from a finding's evidence bundle alone.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Verification {
    /// The finding.
    pub finding: FindingId,
    /// The verifier's `file:line`, `None` when it found no site.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// The verifier's holder.
    pub holder: Holder,
    /// The verifier's mechanism, in words.
    pub mechanism: String,
    /// The verifier's label: an archetype ID or `UNCLASSIFIED`.
    pub label: String,
    /// SHA-256 of the bundle manifest the verifier read.
    pub bundle: String,
    /// The verifier's site is the finding's (same file, line within 3), or both are none.
    pub agrees_site: bool,
    /// The verifier's holder is the finding's.
    pub agrees_holder: bool,
}

/// One line of `ledger.jsonl`.
///
/// `worker` is the key of the worker a line belongs to. `dla ledger merge` sets it on the
/// lines it copies from a slice. The slice records (`slice_started`, `slice_done`,
/// `slice_merged`) carry the same key in their own body; on the wire it appears once,
/// and reading such a line sets both.
#[derive(JsonSchema, Debug, Clone, PartialEq)]
pub struct LedgerLine {
    /// Schema version of this line.
    pub schema_version: u32,
    /// 1-based sequence number; consecutive.
    pub seq: u64,
    /// When the line was appended (RFC 3339, UTC).
    pub ts: DateTime<Utc>,
    /// The worker whose slice the line came from; set by `dla ledger merge` (schema 3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker: Option<String>,
    /// The record.
    #[serde(flatten)]
    pub record: Record,
}

impl Record {
    /// The worker named in the record's own body (the slice records), if it has one.
    pub fn own_worker(&self) -> Option<&str> {
        match self {
            Record::SliceStarted(s) => Some(&s.worker),
            Record::SliceDone(d) => Some(&d.worker),
            Record::SliceMerged(m) => Some(&m.worker),
            _ => None,
        }
    }
}

#[derive(Serialize)]
struct LineOut<'a> {
    schema_version: u32,
    seq: u64,
    ts: &'a DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    worker: Option<&'a str>,
    #[serde(flatten)]
    record: &'a Record,
}

#[derive(Deserialize)]
struct LineIn {
    schema_version: u32,
    seq: u64,
    ts: DateTime<Utc>,
    #[serde(flatten)]
    rest: serde_json::Map<String, serde_json::Value>,
}

impl Serialize for LedgerLine {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        LineOut {
            schema_version: self.schema_version,
            seq: self.seq,
            ts: &self.ts,
            // A slice record writes the key itself.
            worker: self
                .worker
                .as_deref()
                .filter(|_| self.record.own_worker().is_none()),
            record: &self.record,
        }
        .serialize(s)
    }
}

impl<'de> Deserialize<'de> for LedgerLine {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let LineIn {
            schema_version,
            seq,
            ts,
            rest,
        } = LineIn::deserialize(d)?;
        let worker = match rest.get("worker") {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(w)) => Some(w.clone()),
            Some(_) => return Err(D::Error::custom("`worker` must be a string")),
        };
        let record: Record =
            serde_json::from_value(serde_json::Value::Object(rest)).map_err(D::Error::custom)?;
        Ok(Self {
            schema_version,
            seq,
            ts,
            worker,
            record,
        })
    }
}
