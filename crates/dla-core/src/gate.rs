//! Stage and final gates: `dla gate check`.
//!
//! [`evaluate`] is a pure function of a [`RunView`]: the same list feeds `dla gate check`,
//! the stop hooks and `dla run status`. This is the skeleton: it reuses [`crate::view::status`]
//! until the gate package (M6 package D) adds the stage requirements.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{state::Stage, view::RunView};

/// How many missing items `dla gate check` prints; the full list goes to a file.
pub const SHOWN_MISSING: usize = 5;

/// What a gate checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateTarget {
    /// The requirements of one stage (cumulative).
    Stage(Stage),
    /// Everything the report needs.
    Final,
}

/// One thing that keeps a gate from passing.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    /// What kind of item (`finding`, `open_item`, `checklist_item`, `run`, `coverage`, ...).
    pub kind: String,
    /// The finding, open item, checklist item or run ID the item is about.
    pub id: String,
    /// Short description.
    pub description: String,
    /// The command that clears it.
    pub fix: String,
}

/// The result of a gate.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct GateReport {
    /// True when nothing is missing.
    pub ok: bool,
    /// The stage name, or `final`.
    pub stage: String,
    /// Number of missing items in total.
    pub missing_total: usize,
    /// The first [`SHOWN_MISSING`] items; the full list is in `<run>/gate/<stage>.json`.
    pub missing: Vec<Missing>,
    /// Where the full list is.
    pub pointer: String,
}

impl GateTarget {
    /// The name used in output and file names: the stage, or `final`.
    pub fn name(self) -> &'static str {
        match self {
            GateTarget::Stage(s) => s.as_str(),
            GateTarget::Final => "final",
        }
    }
}

/// Evaluates a gate on `view`. `worker` limits the check to one worker's slice.
///
/// Skeleton: every target returns what [`crate::view::status`] blocks on.
pub fn evaluate(view: &RunView, target: GateTarget, worker: Option<&str>) -> GateReport {
    let _ = worker;
    let all: Vec<Missing> = crate::view::status(view)
        .blocking
        .into_iter()
        .map(|b| Missing {
            kind: serde_json::to_value(b.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default(),
            id: b.id,
            description: b.description,
            fix: b.fix,
        })
        .collect();
    let name = target.name();
    GateReport {
        ok: all.is_empty(),
        stage: name.to_owned(),
        missing_total: all.len(),
        missing: all.into_iter().take(SHOWN_MISSING).collect(),
        pointer: format!("gate/{name}.json"),
    }
}
