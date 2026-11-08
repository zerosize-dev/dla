//! The CLI failure type: a refusal plus the process exit code.

use dla_core::{
    Error, Refusal,
    error::{EXIT_ERROR, EXIT_REFUSED},
};

/// CLI-level refusal codes (core owns the rule codes).
pub mod codes {
    /// A required flag is missing.
    pub const MISSING_ARGUMENT: &str = "missing-argument";
    /// A flag does not apply to this kind of target.
    pub const UNSUPPORTED_FLAG: &str = "unsupported-flag";
    /// The resolve target is not a finding, open item or checklist item.
    pub const UNKNOWN_TARGET: &str = "unknown-target";
    /// There is no run to operate on.
    pub const NO_RUN: &str = "no-run";
    /// The named run directory does not exist.
    pub const UNKNOWN_RUN: &str = "unknown-run";
    /// The command line did not parse.
    pub const USAGE: &str = "usage";
    /// Generated schema files differ from the committed ones.
    pub const SCHEMA_DRIFT: &str = "schema-drift";
    /// `ledger verify` names a bundle hash no bundle of the finding has.
    pub const UNKNOWN_BUNDLE: &str = "unknown-bundle";
}

/// Why a command did not succeed, with the exit code to use.
#[derive(Debug)]
pub struct Failure {
    /// Process exit code (1, 2 or 3).
    pub exit: i32,
    /// What to print.
    pub refusal: Refusal,
}

/// Result of a command.
pub type Res<T> = Result<T, Failure>;

impl From<Error> for Failure {
    fn from(e: Error) -> Self {
        Failure {
            exit: e.exit_code(),
            refusal: e.to_refusal(),
        }
    }
}

impl From<Refusal> for Failure {
    fn from(refusal: Refusal) -> Self {
        Failure {
            exit: EXIT_REFUSED,
            refusal,
        }
    }
}

impl Failure {
    /// A refusal built from parts (exit 2).
    pub fn refuse(
        code: &str,
        message: impl Into<String>,
        missing: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Refusal::new(code, message, missing, fix).into()
    }

    /// A usage or parse problem (exit 1).
    pub fn usage(
        message: impl Into<String>,
        missing: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Failure {
            exit: EXIT_ERROR,
            refusal: Refusal::new(codes::USAGE, message, missing, fix),
        }
    }

    /// A failure with an explicit exit code.
    pub fn with_exit(exit: i32, refusal: Refusal) -> Self {
        Failure { exit, refusal }
    }
}
