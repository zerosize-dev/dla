//! DLA core contracts.
//!
//! Types and rules shared by every DLA component: identifiers, evidence records,
//! findings, the append-only [`Ledger`] with its replay validation, the finding
//! [state machine](machine), [run state](state), the [coverage manifest](coverage),
//! the intake [checklist](checklist), [schema versioning](version), the
//! completeness gate ([`status`]) and [value redaction](redact). The M3 contracts are
//! the [`series`] file, the [`trend`] report, the [`harness`] manifest and dump record,
//! and the [`discriminator`] result convention. The M2 stage
//! contracts live here too: [`detectors`] tables, the [`probe`] report,
//! [`static_hit`]s, [`census`] entries, [`workload`]s, the [`paths`] helpers and the
//! [`toolctx`] context every stage crate writes through.
//!
//! Every rule refusal is a [`Refusal`] that names what is missing and the command
//! that fixes it. The crate contains no unsafe code and no network code.

#[macro_use]
mod macros;

pub mod census;
pub mod checklist;
pub mod consent;
pub mod coverage;
pub mod detectors;
pub mod discriminator;
pub mod error;
pub mod evidence;
pub mod finding;
pub mod fsutil;
pub mod harness;
pub mod ids;
pub mod ledger;
pub mod machine;
pub mod paths;
pub mod probe;
pub mod record;
pub mod redact;
pub mod runtime;
pub mod schema;
pub mod series;
pub mod state;
pub mod static_hit;
mod strict;
pub mod toolctx;
pub mod trend;
pub mod version;
pub mod view;
pub mod workload;

/// Schema version written to `state.json` and `coverage.json`.
pub const SCHEMA_VERSION: u32 = 1;
/// Schema version written to every new ledger line. Version 1 lines (M0 to M2) still
/// replay, under the M0 rules; version 2 lines replay under the M3 rules.
pub const LEDGER_SCHEMA_VERSION: u32 = 2;
/// Bound on the serialized size of an evidence `result`.
pub const MAX_RESULT_BYTES: usize = 2048;
/// Bound on the size of CLI stdout.
pub const MAX_OUTPUT_BYTES: usize = 2048;

pub use checklist::{AnswerStatus, Checklist, ChecklistAnswer, ChecklistEntry, ChecklistItem};
pub use coverage::CoverageManifest;
pub use discriminator::{DiscriminatorResult, Favors};
pub use error::{Error, ParseEnumError, Refusal, ReplayError};
pub use evidence::{
    Authority, EvidenceClass, EvidenceKind, EvidenceRecord, NewEvidence, Source, Tool,
};
pub use finding::{Finding, FindingState, Origin, UNCLASSIFIED, UnresolvedReason};
pub use harness::{HarnessDump, HarnessManifest};
pub use ids::{EvidenceId, FindingId, OpenItemId};
pub use ledger::{Annotation, Ledger, NewFinding, OpenItemState};
pub use record::{FindingAnnotation, LedgerLine, Record, Resolution};
pub use series::SeriesFile;
pub use state::{RunState, RunStatus, Stage};
pub use trend::TrendReport;
pub use view::{ReportStatus, RunView, status};
