//! Workloads a loop can run: `.dla/<run>/workload/workloads.json`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{coverage::NotExercisable, runtime::Runtime};

string_enum! {
    /// What a workload is.
    pub enum WorkloadKind {
        /// A unit or integration test run.
        Test = "test",
        /// A program entry point (`main`).
        Main = "main",
        /// A development server.
        DevServer = "dev-server",
        /// A script or console entry.
        Script = "script",
    }
}

/// One runnable workload.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Workload {
    /// Stable identifier, for example `go:test:./...`.
    pub id: String,
    /// What it is.
    pub kind: WorkloadKind,
    /// Runtime.
    pub runtime: Runtime,
    /// Exact command line.
    pub command: String,
    /// Working directory, project-relative (`.` for the root).
    pub cwd: String,
    /// Project-relative file that defines it.
    pub file: String,
    /// Project entry points it references (public functions, handlers, methods).
    #[serde(default)]
    pub entry_points: Vec<String>,
    /// Why this workload cannot run (`build_fails` when its runtime's probe build
    /// failed); absent when it can.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_exercisable: Option<String>,
    /// Rank: the number of entry points it exercises. Higher runs first.
    pub rank: u32,
}

/// The `workloads.json` document.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WorkloadReport {
    /// Schema version.
    pub schema_version: u32,
    /// Workloads, ranked (highest first).
    #[serde(default)]
    pub workloads: Vec<Workload>,
    /// Every entry point found in the project.
    #[serde(default)]
    pub entry_points: Vec<String>,
    /// Entry points (never workloads) that cannot be exercised, with the reason
    /// (`no_harness`, `build_fails`). Workloads carry their own `not_exercisable`.
    #[serde(default)]
    pub not_exercisable: Vec<NotExercisable>,
    /// Runtimes whose build failed in `dla probe --build`.
    #[serde(default)]
    pub build_failed: Vec<Runtime>,
}
