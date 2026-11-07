//! Errors and refusals.
//!
//! A [`Refusal`] is a rule-based "no" that always says what is missing and which
//! command fixes it. [`Error`] is the umbrella type returned by every fallible
//! operation in this crate and maps onto the CLI exit codes.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::version::SchemaError;

/// Exit code: success.
pub const EXIT_OK: i32 = 0;
/// Exit code: I/O or parse error.
pub const EXIT_ERROR: i32 = 1;
/// Exit code: refused by a rule.
pub const EXIT_REFUSED: i32 = 2;
/// Exit code: run directory written by a different schema version.
pub const EXIT_SCHEMA: i32 = 3;

/// Stable refusal codes.
pub mod codes {
    /// A finding or checklist answer cites no evidence.
    pub const MISSING_EVIDENCE: &str = "missing-evidence";
    /// A cited evidence ID does not exist (earlier in the ledger).
    pub const UNKNOWN_EVIDENCE: &str = "unknown-evidence";
    /// A referenced finding does not exist.
    pub const UNKNOWN_FINDING: &str = "unknown-finding";
    /// A sibling's parent does not exist.
    pub const UNKNOWN_PARENT: &str = "unknown-parent";
    /// A referenced open item does not exist.
    pub const UNKNOWN_OPEN_ITEM: &str = "unknown-open-item";
    /// A checklist item ID is not in the checklist.
    pub const UNKNOWN_CHECKLIST_ITEM: &str = "unknown-checklist-item";
    /// An ID was used twice.
    pub const DUPLICATE_ID: &str = "duplicate-id";
    /// The open item already has a resolution.
    pub const ALREADY_RESOLVED: &str = "already-resolved";
    /// An ID or enum value did not parse.
    pub const INVALID_VALUE: &str = "invalid-value";
    /// The origin string is not one of the allowed forms.
    pub const INVALID_ORIGIN: &str = "invalid-origin";
    /// An attempt to set the archetype through `ledger add`.
    pub const ARCHETYPE_LOCKED: &str = "archetype-locked";
    /// A new finding must start as `candidate`.
    pub const NOT_CANDIDATE: &str = "state-not-candidate";
    /// The move is not an edge of the state machine.
    pub const ILLEGAL_TRANSITION: &str = "illegal-transition";
    /// The record's `from` differs from the replayed state.
    pub const FROM_MISMATCH: &str = "from-mismatch";
    /// `unresolved` / `suspected_static_only` without a reason.
    pub const REASON_REQUIRED: &str = "reason-required";
    /// A reason without a non-empty remedy.
    pub const REMEDY_REQUIRED: &str = "remedy-required";
    /// A reason on a move that does not take one.
    pub const UNEXPECTED_REASON: &str = "unexpected-reason";
    /// `suspected_static_only` without static evidence.
    pub const STATIC_EVIDENCE_REQUIRED: &str = "static-evidence-required";
    /// `refuted` without a discriminator evidence record in `contradicts`.
    pub const DISCRIMINATOR_REQUIRED: &str = "discriminator-required";
    /// Evidence `result` over the size bound.
    pub const RESULT_TOO_LARGE: &str = "result-too-large";
    /// A sibling without `hypothesis` and `refute_if`, or other sibling misuse.
    pub const SIBLING_FIELDS: &str = "sibling-fields-required";
    /// The finding's `parent` disagrees with its ID.
    pub const PARENT_MISMATCH: &str = "parent-mismatch";
    /// The `siblings` list is maintained by the ledger only.
    pub const SIBLINGS_MANAGED: &str = "siblings-managed";
    /// A required text field is empty.
    pub const EMPTY_FIELD: &str = "empty-field";
    /// Sequence number gap or reordering.
    pub const BAD_SEQ: &str = "bad-seq";
    /// Blank line inside a ledger.
    pub const BLANK_LINE: &str = "blank-line";
    /// A ledger line is not valid JSON for a record.
    pub const MALFORMED: &str = "malformed-line";
    /// The run was aborted or is incomplete.
    pub const RUN_NOT_ACTIVE: &str = "run-not-active";
}

/// A rule-based refusal: what was refused, what is missing, the command that fixes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{message} (missing: {missing}; fix: {fix})")]
pub struct Refusal {
    /// Stable machine-readable code, see [`codes`].
    pub code: String,
    /// One-sentence explanation.
    pub message: String,
    /// The item that is missing or wrong.
    pub missing: String,
    /// A concrete command line (or edit) that resolves the refusal.
    pub fix: String,
}

impl Refusal {
    /// Builds a refusal.
    pub fn new(
        code: &str,
        message: impl Into<String>,
        missing: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
            missing: missing.into(),
            fix: fix.into(),
        }
    }
}

/// A refusal raised while replaying a ledger, located at a line.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{path}line {line}: {refusal}", path = path_prefix(.path))]
pub struct ReplayError {
    /// File the line came from, when replaying from disk.
    pub path: Option<PathBuf>,
    /// 1-based line number.
    pub line: u64,
    /// True for unparseable JSON, false for a rule violation.
    pub malformed: bool,
    /// What is wrong and how to fix it.
    pub refusal: Refusal,
}

fn path_prefix(p: &Option<PathBuf>) -> String {
    p.as_ref()
        .map(|p| format!("{}: ", p.display()))
        .unwrap_or_default()
}

/// A string did not match any variant of a closed enum.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid {kind} {input:?}: expected one of {expected}")]
pub struct ParseEnumError {
    /// Enum type name.
    pub kind: &'static str,
    /// The rejected input.
    pub input: String,
    /// Accepted spellings joined with `|`.
    pub expected: String,
}

/// Umbrella error for operations in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Filesystem failure.
    #[error("{}: {source}", path.display())]
    Io {
        /// Path involved.
        path: PathBuf,
        /// Underlying error.
        source: std::io::Error,
    },
    /// A run-directory file is not valid JSON for its type.
    #[error("{}: {message}", path.display())]
    Malformed {
        /// Path involved.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// Schema version mismatch (exit code 3).
    #[error(transparent)]
    Schema(#[from] SchemaError),
    /// Ledger replay refused a line.
    #[error(transparent)]
    Replay(Box<ReplayError>),
    /// A rule refused the operation.
    #[error(transparent)]
    Refused(#[from] Refusal),
}

impl From<ReplayError> for Error {
    fn from(e: ReplayError) -> Self {
        Error::Replay(Box::new(e))
    }
}

impl Error {
    /// Wraps an I/O error with the path involved.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }

    /// The process exit code the CLI should use for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Io { .. } | Error::Malformed { .. } => EXIT_ERROR,
            Error::Schema(_) => EXIT_SCHEMA,
            Error::Replay(r) if r.malformed => EXIT_ERROR,
            Error::Replay(_) | Error::Refused(_) => EXIT_REFUSED,
        }
    }

    /// Renders any error in the `{code, message, missing, fix}` shape.
    pub fn to_refusal(&self) -> Refusal {
        match self {
            Error::Refused(r) => r.clone(),
            Error::Replay(r) => Refusal {
                message: r.to_string(),
                ..r.refusal.clone()
            },
            Error::Schema(s) => Refusal::new(s.code(), s.to_string(), s.missing(), s.fix()),
            Error::Io { path, .. } => Refusal::new(
                "io-error",
                self.to_string(),
                path.display().to_string(),
                "check that the path exists and is readable and writable",
            ),
            Error::Malformed { path, .. } => Refusal::new(
                "malformed-file",
                self.to_string(),
                path.display().to_string(),
                "restore the file or start a new run with `dla run init`",
            ),
        }
    }
}
