//! The intake checklist and its answers.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{SCHEMA_VERSION, finding::UnresolvedReason, ids::EvidenceId, state::Stage};

/// One question the run must answer.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ChecklistItem {
    /// Stable ID such as `intake.scope`.
    pub id: String,
    /// Stage the question belongs to.
    pub stage: Stage,
    /// The question.
    pub question: String,
}

/// The checklist as a versioned document.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Checklist {
    /// Schema version.
    pub schema_version: u32,
    /// The questions, in order.
    pub items: Vec<ChecklistItem>,
}

const ITEMS: &[(&str, &str)] = &[
    (
        "intake.target",
        "Is the target a repository, a local process, an artifact, or only a symptom?",
    ),
    (
        "intake.scope",
        "Does the investigation cover the whole repository, one module or one service, and what is excluded?",
    ),
    (
        "intake.symptom",
        "What growth was observed, where, and over what time?",
    ),
    (
        "intake.runtimes",
        "Which language runtimes and processes are involved?",
    ),
    ("intake.build", "Does the project build locally?"),
    ("intake.workload", "What can exercise the suspected path?"),
    (
        "intake.consent",
        "Which intrusive steps has the user allowed, and are raw values permitted in evidence?",
    ),
    (
        "intake.budget",
        "How much wall-clock time may the investigation use?",
    ),
];

impl Checklist {
    /// The built-in checklist.
    pub fn standard() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            items: ITEMS
                .iter()
                .map(|(id, q)| ChecklistItem {
                    id: (*id).to_owned(),
                    stage: Stage::Intake,
                    question: (*q).to_owned(),
                })
                .collect(),
        }
    }

    /// True if `id` names an item.
    pub fn contains(&self, id: &str) -> bool {
        self.items.iter().any(|i| i.id == id)
    }
}

string_enum! {
    /// How completely a checklist item was answered.
    pub enum AnswerStatus {
        /// Fully answered.
        Answered = "answered",
        /// Partly answered.
        Partial = "partial",
        /// Not answerable now; carries reason and remedy.
        Unresolved = "unresolved",
    }
}

/// A `checklist_answer` ledger record body.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ChecklistAnswer {
    /// Checklist item ID.
    pub item: String,
    /// Answer status.
    pub status: AnswerStatus,
    /// Supporting evidence; at least one unless `unresolved`.
    #[serde(default)]
    pub evidence: Vec<EvidenceId>,
    /// Free-text note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Required with `unresolved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<UnresolvedReason>,
    /// Required with `unresolved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
}

/// A checklist item with its latest answer.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ChecklistEntry {
    /// The question.
    pub item: ChecklistItem,
    /// The latest answer, if any.
    pub answer: Option<ChecklistAnswer>,
}

impl ChecklistEntry {
    /// `answered`, `partial`, `unresolved` or `missing`.
    pub fn status_label(&self) -> &'static str {
        self.answer
            .as_ref()
            .map_or("missing", |a| a.status.as_str())
    }
}
