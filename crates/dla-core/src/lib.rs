//! DLA core contracts.
//!
//! Types and rules shared by every DLA component: identifiers, evidence records,
//! findings, the append-only [`Ledger`] with its replay validation, the finding
//! [state machine](machine), [run state](state), the [coverage manifest](coverage),
//! the intake [checklist](checklist), [schema versioning](version), the
//! completeness gate ([`status`]) and [value redaction](redact).
//!
//! Every rule refusal is a [`Refusal`] that names what is missing and the command
//! that fixes it. The crate contains no unsafe code and no network code.

#[macro_use]
mod macros;

pub mod checklist;
pub mod coverage;
pub mod error;
pub mod evidence;
pub mod finding;
pub mod fsutil;
pub mod ids;
pub mod ledger;
pub mod machine;
pub mod record;
pub mod redact;
pub mod schema;
pub mod state;
pub mod version;
pub mod view;

/// Schema version written to `state.json`, `coverage.json` and every ledger line.
pub const SCHEMA_VERSION: u32 = 1;
/// Bound on the serialized size of an evidence `result`.
pub const MAX_RESULT_BYTES: usize = 2048;
/// Bound on the size of CLI stdout.
pub const MAX_OUTPUT_BYTES: usize = 2048;

pub use checklist::{AnswerStatus, Checklist, ChecklistAnswer, ChecklistEntry, ChecklistItem};
pub use coverage::CoverageManifest;
pub use error::{Error, ParseEnumError, Refusal, ReplayError};
pub use evidence::{
    Authority, EvidenceClass, EvidenceKind, EvidenceRecord, NewEvidence, Source, Tool,
};
pub use finding::{Finding, FindingState, Origin, UNCLASSIFIED, UnresolvedReason};
pub use ids::{EvidenceId, FindingId, OpenItemId};
pub use ledger::{Ledger, NewFinding, OpenItemState};
pub use record::{LedgerLine, Record, Resolution};
pub use state::{RunState, RunStatus, Stage};
pub use view::{ReportStatus, RunView, status};
