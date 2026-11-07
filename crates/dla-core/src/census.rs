//! One holder enumerated by `dla census`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::finding::{Holder, Region};

/// A process-lifetime holder: something that can keep objects, threads or handles
/// alive for as long as the process runs.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct CensusEntry {
    /// What kind of declaration it is (for example `static-field`, `module-global`,
    /// `singleton-field`, `registry`, `spawn`, `handle`).
    pub kind: String,
    /// Holder class in the ledger vocabulary.
    pub holder: Holder,
    /// Memory region.
    pub region: Region,
    /// Declaration, spawn or acquire site, `file:line` (project-relative).
    pub site: String,
    /// Qualified name of the owning field or container.
    pub owner: String,
    /// Heuristic `file:line` sites that add to it.
    #[serde(default)]
    pub writers: Vec<String>,
    /// Heuristic `file:line` sites that remove from it.
    #[serde(default)]
    pub removers: Vec<String>,
}
