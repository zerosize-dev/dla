//! Run state: `.dla/<run-id>/state.json`, rewritten atomically by dla only.

use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeDelta, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    SCHEMA_VERSION,
    error::Error,
    fsutil::{load_versioned, to_pretty_json, write_atomic},
    version::Artifact,
};

/// Directory (under the project root) holding all runs.
pub const DLA_DIR: &str = ".dla";
/// File naming the current run, inside [`DLA_DIR`].
pub const CURRENT_FILE: &str = "current";
/// State file name inside a run directory.
pub const STATE_FILE: &str = "state.json";
/// Ledger file name inside a run directory.
pub const LEDGER_FILE: &str = "ledger.jsonl";
/// Coverage manifest file name inside a run directory.
pub const COVERAGE_FILE: &str = "coverage.json";
/// Rendered report file name inside a run directory.
pub const REPORT_FILE: &str = "report.md";

/// `<project>/.dla/<run-id>`.
pub fn run_dir(project_root: &Path, run_id: &str) -> PathBuf {
    project_root.join(DLA_DIR).join(run_id)
}

/// The current UTC time truncated to whole seconds (keeps ledger lines stable).
pub fn now_utc() -> DateTime<Utc> {
    let now = Utc::now();
    DateTime::from_timestamp(now.timestamp(), 0).unwrap_or(now)
}

string_enum! {
    /// Stage of the investigation.
    pub enum Stage {
        /// Collecting the target, scope and symptom.
        Intake = "intake",
        /// Probing the environment.
        Probe = "probe",
        /// Sweeping for candidates.
        Sweep = "sweep",
        /// Classifying candidates.
        Classify = "classify",
        /// Confirming growth.
        Confirm = "confirm",
        /// Attributing to a site and holder.
        Attribute = "attribute",
        /// Ruling out alternatives.
        RuleOut = "rule_out",
        /// Applying the completeness gate.
        Gate = "gate",
        /// Rendering the report.
        Report = "report",
    }
}

string_enum! {
    /// Overall status of a run.
    pub enum RunStatus {
        /// In progress.
        Active = "active",
        /// Finished with nothing outstanding.
        Complete = "complete",
        /// Stopped with outstanding items.
        Incomplete = "incomplete",
        /// Abandoned.
        Aborted = "aborted",
    }
}

string_enum! {
    /// Who recorded the consent.
    pub enum ConsentGiver {
        /// The user, interactively.
        User = "user",
        /// The runner, by policy.
        Runner = "runner",
    }
}

string_enum! {
    /// What a scope covers.
    pub enum ScopeKind {
        /// The whole repository.
        Repository = "repository",
        /// One module.
        Module = "module",
        /// One service.
        Service = "service",
    }
}

string_enum! {
    /// Kind of process DLA started.
    pub enum ProcessKind {
        /// A test or workload harness.
        Harness = "harness",
        /// A development server.
        DevServer = "dev-server",
        /// A container.
        Container = "container",
        /// A profiler.
        Profiler = "profiler",
    }
}

/// Consent given for intrusive steps.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Consent {
    /// Allowed intrusiveness tiers.
    pub tiers: Vec<u8>,
    /// Whether raw values may appear in evidence.
    pub raw_values: bool,
    /// Who gave it.
    pub given_by: ConsentGiver,
    /// When it was recorded.
    pub recorded_at: DateTime<Utc>,
}

/// Time budget.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Budget {
    /// Budget in seconds.
    pub seconds: u64,
    /// Start of the budget.
    pub started_at: DateTime<Utc>,
    /// Start plus budget.
    pub deadline: DateTime<Utc>,
}

/// What the investigation covers.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Scope {
    /// Repository, module or service.
    pub kind: ScopeKind,
    /// Path of the module or service (the repository root for `repository`).
    pub path: String,
    /// Excluded paths.
    #[serde(default)]
    pub excluded: Vec<String>,
}

/// A process DLA started and must clean up.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ProcessRecord {
    /// Process ID.
    pub pid: u32,
    /// Platform start time, to disambiguate PID reuse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// What it is.
    pub kind: ProcessKind,
    /// Command line.
    pub command: String,
    /// When it started.
    pub started_at: DateTime<Utc>,
    /// When it stopped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<DateTime<Utc>>,
}

/// The rendered report, pinned by hash.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ReportRef {
    /// Path of the report relative to the run directory.
    pub path: String,
    /// SHA-256 of the report bytes, lowercase hex.
    pub sha256: String,
    /// When it was rendered.
    pub rendered_at: DateTime<Utc>,
}

/// `state.json`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct RunState {
    /// Schema version.
    pub schema_version: u32,
    /// `YYYYMMDD-HHMMSS-xxxx` (UTC plus 4 hex).
    pub run_id: String,
    /// Host session ID, `unknown` if absent.
    pub session_id: String,
    /// Absolute project root.
    pub project_root: String,
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Version of dla that created the run.
    pub dla_version: String,
    /// Current stage.
    pub stage: Stage,
    /// Overall status.
    pub status: RunStatus,
    /// Why the run is incomplete or aborted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incomplete_reason: Option<String>,
    /// Consent, once given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consent: Option<Consent>,
    /// Time budget.
    pub budget: Budget,
    /// Investigation scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
    /// Processes DLA started.
    #[serde(default)]
    pub processes: Vec<ProcessRecord>,
    /// Random per-run salt (hex) for value redaction.
    pub redaction_salt: String,
    /// The rendered report, once rendered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<ReportRef>,
}

impl RunState {
    /// Creates the state for a new run: fresh run ID and redaction salt, stage
    /// `intake`, status `active`.
    pub fn init(
        project_root: &str,
        session_id: Option<&str>,
        budget_seconds: u64,
        scope: Option<Scope>,
        now: DateTime<Utc>,
    ) -> Result<Self, Error> {
        let mut suffix = [0u8; 2];
        let mut salt = [0u8; 16];
        let rnd = |buf: &mut [u8]| {
            getrandom::fill(buf).map_err(|e| Error::io(".", std::io::Error::other(e.to_string())))
        };
        rnd(&mut suffix)?;
        rnd(&mut salt)?;
        let secs = i64::try_from(budget_seconds).unwrap_or(i64::MAX);
        let deadline = TimeDelta::try_seconds(secs)
            .and_then(|d| now.checked_add_signed(d))
            .unwrap_or(DateTime::<Utc>::MAX_UTC);
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            run_id: format!("{}-{}", now.format("%Y%m%d-%H%M%S"), hex::encode(suffix)),
            session_id: session_id
                .filter(|s| !s.is_empty())
                .unwrap_or("unknown")
                .to_owned(),
            project_root: project_root.to_owned(),
            created_at: now,
            dla_version: env!("CARGO_PKG_VERSION").to_owned(),
            stage: Stage::Intake,
            status: RunStatus::Active,
            incomplete_reason: None,
            consent: None,
            budget: Budget {
                seconds: budget_seconds,
                started_at: now,
                deadline,
            },
            scope,
            processes: Vec::new(),
            redaction_salt: hex::encode(salt),
            report: None,
        })
    }

    /// The salt as bytes for [`crate::redact::shape`].
    pub fn salt_bytes(&self) -> Vec<u8> {
        hex::decode(&self.redaction_salt)
            .unwrap_or_else(|_| self.redaction_salt.clone().into_bytes())
    }

    /// Loads `<run_dir>/state.json`, refusing newer schemas and migrating older ones.
    pub fn load(run_dir: &Path) -> Result<Self, Error> {
        load_versioned(&run_dir.join(STATE_FILE), Artifact::State)
    }

    /// Writes `<run_dir>/state.json` atomically (temp file, then rename).
    pub fn save(&self, run_dir: &Path) -> Result<(), Error> {
        write_atomic(&run_dir.join(STATE_FILE), to_pretty_json(self).as_bytes())
    }
}

/// Writes `<project>/.dla/current` with the run ID.
pub fn write_current(project_root: &Path, run_id: &str) -> Result<(), Error> {
    write_atomic(
        &project_root.join(DLA_DIR).join(CURRENT_FILE),
        format!("{run_id}\n").as_bytes(),
    )
}

/// Reads `<project>/.dla/current`; `None` if absent or empty.
pub fn read_current(project_root: &Path) -> Result<Option<String>, Error> {
    let p = project_root.join(DLA_DIR).join(CURRENT_FILE);
    match std::fs::read_to_string(&p) {
        Ok(s) => {
            let s = s.trim();
            Ok((!s.is_empty()).then(|| s.to_owned()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::io(p, e)),
    }
}
