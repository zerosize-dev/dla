//! DLA core contracts.
//!
//! Types and rules shared by every DLA component: identifiers, evidence records,
//! findings, the append-only [`Ledger`] with its replay validation, the finding
//! [state machine](machine), [run state](state), the [coverage manifest](coverage),
//! the intake [checklist](checklist), [schema versioning](version), the
//! completeness gate ([`status`]), [value redaction](redact), and the M4 [worker
//! slices](slice), their [merge](merge) and the [verification](verify) record. The M3 contracts are
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

pub mod attribution;
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
pub mod known_leaks;
pub mod ledger;
pub mod machine;
pub mod merge;
pub mod paths;
pub mod probe;
pub mod proc;
pub mod record;
pub mod redact;
pub mod runtime;
pub mod schema;
pub mod series;
pub mod slice;
pub mod state;
pub mod static_hit;
mod strict;
pub mod toolctx;
pub mod trend;
pub mod verify;
pub mod version;
pub mod view;
pub mod workload;

/// Schema version written to `state.json` and `coverage.json`.
pub const SCHEMA_VERSION: u32 = 1;
/// Schema version written to every new ledger line. Version 1 lines (M0 to M2) still
/// replay, under the M0 rules; version 2 lines replay under the M3 rules; version 3
/// lines (M4) replay under the M3 rules plus the artifact-path rules, and may carry a
/// `worker` and the slice, merge and verification records; version 4 lines (M5) also
/// require a leak-side confirmation to cite a passing growth column tied to the finding;
/// version 5 lines may also change a holder already set, when the annotation cites heap-graph
/// growth evidence whose passing column tied to the finding observed that holder.
pub const LEDGER_SCHEMA_VERSION: u32 = 5;
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
pub use ledger::{Annotation, HolderChange, Ledger, NewFinding, OpenItemState};
pub use record::{
    FindingAnnotation, FindingMerged, LedgerLine, Record, Resolution, SliceDone, SliceMerged,
    SliceStarted, Verification,
};
pub use series::SeriesFile;
pub use state::{RunState, RunStatus, Stage};
pub use trend::TrendReport;
pub use view::{ReportStatus, RunView, status};
