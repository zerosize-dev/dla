//! Integrity seals for the files `dla` protects.
//!
//! `<run>/integrity/<file>.seal` holds the length and SHA-256 of a protected file as `dla`
//! last wrote it. [`verify`] reports every file that no longer matches. Hooks are a guard
//! rail; the seals are what makes an out-of-band edit visible.
//!
//! Skeleton: the signatures are final, the bodies are stubs (M6 package D).

use std::path::Path;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Directory inside the run directory that holds the seals.
pub const INTEGRITY_DIR: &str = "integrity";
/// Suffix of a seal file.
pub const SEAL_SUFFIX: &str = ".seal";

/// The seal of one protected file.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct Seal {
    /// Length in bytes.
    pub len: u64,
    /// SHA-256, lowercase hex.
    pub sha256: String,
    /// When the seal was written.
    pub written_at: DateTime<Utc>,
}

/// Why a protected file failed verification.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TamperKind {
    /// The file's length or hash differs from its seal.
    Modified,
    /// The file exists and has no seal in a run that has an `integrity/` directory.
    Unsealed,
    /// The seal exists and the file does not.
    Missing,
}

/// A protected file that does not match its seal.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct Tamper {
    /// File name relative to the run directory.
    pub file: String,
    /// What is wrong.
    pub kind: TamperKind,
    /// What the seal says (`len:sha256`), when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sealed: Option<String>,
    /// What the file is now (`len:sha256`), when it exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found: Option<String>,
}

/// Seals `file` (a name relative to `run_dir`) as it is now and writes the seal file.
/// Callers hold the run lock that guarded the write of `file`.
///
/// Skeleton: writes nothing and returns an empty seal.
pub fn seal_file(run_dir: &Path, file: &str) -> Result<Seal, Error> {
    let _ = (run_dir, file);
    Ok(Seal {
        len: 0,
        sha256: String::new(),
        written_at: crate::state::now_utc(),
    })
}

/// Checks every protected file of the run against its seal. A run without an `integrity/`
/// directory (written before schema 6) reports nothing here.
///
/// Skeleton: always empty.
pub fn verify(run_dir: &Path) -> Vec<Tamper> {
    let _ = run_dir;
    Vec::new()
}
