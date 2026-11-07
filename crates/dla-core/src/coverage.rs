//! Coverage manifest (`coverage.json`): what each worker could and could not see.
//! Types only in M0; the report reads the file when present.

use std::{collections::BTreeMap, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    finding::Region,
    fsutil::load_versioned,
    ids::OpenItemId,
    state::{COVERAGE_FILE, Scope},
    version::Artifact,
};

string_enum! {
    /// Whether a backend ran.
    pub enum BackendStatus {
        /// It ran.
        Ran = "ran",
        /// It exists but could not run.
        Blocked = "blocked",
        /// It is not installed.
        Missing = "missing",
    }
}

/// One backend's coverage.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct BackendCoverage {
    /// Ran, blocked or missing.
    pub status: BackendStatus,
    /// Why it was blocked or missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// How to install or enable it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_hint: Option<String>,
    /// Backend version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Items it enumerated.
    pub enumerated: u64,
    /// Items that were dispositioned.
    pub dispositioned: u64,
}

/// An entry point that could not be exercised.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct NotExercisable {
    /// The entry point.
    pub entry: String,
    /// Why not.
    pub reason: String,
}

/// Entry points of a worker's runtime.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct EntryPoints {
    /// Found by static reading.
    #[serde(default)]
    pub discovered: Vec<String>,
    /// Actually run.
    #[serde(default)]
    pub exercised: Vec<String>,
    /// Found but not runnable.
    #[serde(default)]
    pub not_exercisable: Vec<NotExercisable>,
}

/// Coverage of one worker.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WorkerCoverage {
    /// Runtime name.
    pub runtime: String,
    /// Backends by name.
    #[serde(default)]
    pub backends: BTreeMap<String, BackendCoverage>,
    /// Entry points.
    #[serde(default)]
    pub entry_points: EntryPoints,
    /// Workload iterations run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u64>,
}

/// Growth left unattributed in one region.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ResidualRow {
    /// Memory region.
    pub region: Region,
    /// Total growth slope.
    pub total_slope: f64,
    /// Slope attributed to findings.
    pub attributed_slope: f64,
    /// Total minus attributed.
    pub residual: f64,
    /// Unit of the slopes.
    pub unit: String,
    /// Threshold above which the residual must be explained.
    pub threshold: f64,
    /// Open item covering an over-threshold residual.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_item: Option<OpenItemId>,
}

/// A thing a runtime's tooling cannot observe.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct BlindSpot {
    /// Runtime name.
    pub runtime: String,
    /// What cannot be seen.
    pub text: String,
}

/// `coverage.json`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct CoverageManifest {
    /// Schema version.
    pub schema_version: u32,
    /// Scope covered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
    /// Coverage by worker.
    #[serde(default)]
    pub workers: BTreeMap<String, WorkerCoverage>,
    /// Residual rows.
    #[serde(default)]
    pub residual: Vec<ResidualRow>,
    /// Declared blind spots.
    #[serde(default)]
    pub blind_spots: Vec<BlindSpot>,
}

impl CoverageManifest {
    /// Loads `<run_dir>/coverage.json`; `None` if the file does not exist.
    pub fn load(run_dir: &Path) -> Result<Option<Self>, Error> {
        let p = run_dir.join(COVERAGE_FILE);
        if !p.exists() {
            return Ok(None);
        }
        load_versioned(&p, Artifact::Coverage).map(Some)
    }
}
