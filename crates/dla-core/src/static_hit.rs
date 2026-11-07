//! One hit of a static backend, as adapters report it before it becomes a candidate.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A static analysis hit, normalised from a backend's SARIF or JSON output.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct StaticHit {
    /// Backend name (one of the detector backends).
    pub backend: String,
    /// The backend's rule identifier.
    pub rule: String,
    /// Project-relative file with `/` separators.
    pub file: String,
    /// 1-based line.
    pub line: u32,
    /// The backend's message, trimmed.
    pub message: String,
}

impl StaticHit {
    /// The candidate origin: `detector:<backend>/<rule>`.
    pub fn origin(&self) -> String {
        format!("detector:{}/{}", self.backend, self.rule)
    }

    /// The candidate site: `file:line`.
    pub fn site(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}
