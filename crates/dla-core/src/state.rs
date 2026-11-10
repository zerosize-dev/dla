//! Run state: `.dla/<run-id>/state.json`, rewritten atomically by dla only.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use chrono::{DateTime, TimeDelta, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    SCHEMA_VERSION,
    error::Error,
    fsutil::{load_versioned, to_pretty_json, with_run_lock, write_atomic},
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

string_enum! {
    /// How DLA came to know a process.
    pub enum ProcessRole {
        /// DLA started it and may stop it.
        Started = "started",
        /// DLA attached a tool to a process it did not start; it never kills the target.
        Attached = "attached",
    }
}

string_enum! {
    /// Which of the four entry paths a run takes.
    pub enum EntryPath {
        /// A source repository (the default).
        Repository = "repository",
        /// A running process DLA did not start.
        Process = "process",
        /// Captured artifacts only (heap dumps, profiles).
        Artifact = "artifact",
        /// A symptom description without a target.
        Symptom = "symptom",
    }
}

string_enum! {
    /// Lifecycle of one worker.
    pub enum WorkerStatus {
        /// Registered by `dla run dispatch`, nothing written yet.
        Registered = "registered",
        /// Its slice has been started.
        Running = "running",
        /// It wrote `slice_done`.
        Done = "done",
        /// It stopped without `slice_done`.
        Killed = "killed",
        /// Its deadline passed before `slice_done`.
        OverBudget = "over-budget",
    }
}

impl Stage {
    /// The stages `dla gate check --stage` accepts, in order.
    pub const GATED: &'static [Stage] = &[
        Stage::Intake,
        Stage::Probe,
        Stage::Sweep,
        Stage::Classify,
        Stage::Confirm,
        Stage::Attribute,
        Stage::RuleOut,
    ];

    /// Position of the stage in the investigation (declaration order).
    pub fn rank(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }

    /// True when this stage is `other` or comes after it.
    pub fn at_least(self, other: Stage) -> bool {
        self >= other
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
    /// Whether DLA started the process or attached to it (older records: started).
    #[serde(default = "default_role")]
    pub role: ProcessRole,
    /// The worker that registered it, in worker mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker: Option<String>,
    /// Cleanup actions for an attached tool, in order (for example
    /// `remove /tmp/.java_pid123`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cleanup: Vec<String>,
}

fn default_role() -> ProcessRole {
    ProcessRole::Started
}

/// One worker registered by `dla run dispatch`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WorkerEntry {
    /// Worker name (the runtime name, for example `jvm`).
    pub name: String,
    /// Runtime the worker covers.
    pub runtime: String,
    /// Its share of the remaining budget, in seconds.
    pub share_secs: u64,
    /// Absolute deadline.
    pub deadline: DateTime<Utc>,
    /// When its slice started, once it has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    /// Lifecycle status.
    pub status: WorkerStatus,
    /// Slice file name inside the run directory: `ledger.<name>.jsonl`.
    pub slice: String,
    /// The SubagentStop gate blocked the worker three times with no ledger change and let
    /// it stop; merge turns its unfinished work into an open item.
    #[serde(default, skip_serializing_if = "is_false")]
    pub stalled: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
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
    /// Entry path (older state files: repository).
    #[serde(default = "default_entry")]
    pub entry: EntryPath,
    /// Artifact paths relative to the project root (artifact runs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
    /// Workers registered by `dla run dispatch`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workers: Vec<WorkerEntry>,
    /// Stages that passed `dla gate check --stage`, in the order they passed. Written only
    /// by a passing gate check.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages_passed: Vec<Stage>,
    /// The stages each worker passed with `dla --worker <name> gate check --stage <s>`. A
    /// worker's own checks never touch `stages_passed`: the read hook lets a subagent read
    /// the archetype tables once its worker passed `classify` here.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub worker_stages_passed: BTreeMap<String, Vec<Stage>>,
    /// Consecutive stop-hook blocks with an unchanged ledger digest.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub stop_blocks: u32,
    /// Digest (ledger length plus hash of the missing-item ids) of the last stop-hook block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_block_digest: Option<String>,
    /// The same counter for each worker's SubagentStop hook, by worker name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub worker_stop: BTreeMap<String, StopCounter>,
}

/// Consecutive stop-hook blocks with an unchanged digest.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default, PartialEq, Eq)]
pub struct StopCounter {
    /// Blocks in a row with this digest.
    pub blocks: u32,
    /// Digest of the last block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}

impl StopCounter {
    /// Counts one more failing stop attempt with `digest`: the same digest as the last one
    /// adds to the count, a different one starts again at 1. Returns the new count.
    pub fn bump(&mut self, digest: &str) -> u32 {
        if self.digest.as_deref() == Some(digest) {
            self.blocks = self.blocks.saturating_add(1);
        } else {
            self.blocks = 1;
            self.digest = Some(digest.to_owned());
        }
        self.blocks
    }
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

fn default_entry() -> EntryPath {
    EntryPath::Repository
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
            entry: EntryPath::Repository,
            artifacts: Vec::new(),
            workers: Vec::new(),
            stages_passed: Vec::new(),
            worker_stages_passed: BTreeMap::new(),
            stop_blocks: 0,
            stop_block_digest: None,
            worker_stop: BTreeMap::new(),
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

    /// Writes `<run_dir>/state.json` atomically (temp file, then rename) and seals it.
    pub fn save(&self, run_dir: &Path) -> Result<(), Error> {
        let guard = crate::integrity::Guard::new(run_dir, STATE_FILE);
        let text = to_pretty_json(self);
        guard.expect(text.as_bytes());
        write_atomic(&run_dir.join(STATE_FILE), text.as_bytes())?;
        guard.seal()?;
        Ok(())
    }

    /// Read-modify-write of `state.json`: loads the current state, applies `f` and
    /// saves it, returning `f`'s value. Every writer goes through this.
    ///
    /// The whole load-modify-save runs under the exclusive lock on
    /// `<run_dir>/run.lock` ([`with_run_lock`]), so concurrent workers never lose an
    /// update. `f` must not call `update` itself. When `f` fails nothing is saved.
    pub fn update<T>(
        run_dir: &Path,
        f: impl FnOnce(&mut RunState) -> Result<T, Error>,
    ) -> Result<T, Error> {
        with_run_lock(run_dir, || {
            let mut state = Self::load(run_dir)?;
            let out = f(&mut state)?;
            state.save(run_dir)?;
            Ok(out)
        })
    }

    /// True when `dla gate check --stage` passed for `stage`.
    pub fn stage_passed(&self, stage: Stage) -> bool {
        self.stages_passed.contains(&stage)
    }

    /// Records a passing `gate check --stage` and moves `stage` on to the next gated one.
    pub fn record_stage_passed(&mut self, stage: Stage) {
        if !self.stages_passed.contains(&stage) {
            self.stages_passed.push(stage);
        }
        let next = Stage::ALL
            .get(stage.rank() + 1)
            .copied()
            .filter(|n| n.rank() > self.stage.rank());
        if let Some(n) = next {
            self.stage = n;
        }
    }

    /// True when the worker's own `gate check --stage` passed for `stage`.
    pub fn worker_stage_passed(&self, worker: &str, stage: Stage) -> bool {
        self.worker_stages_passed
            .get(worker)
            .is_some_and(|v| v.contains(&stage))
    }

    /// Records a passing `--worker <worker> gate check --stage`.
    pub fn record_worker_stage_passed(&mut self, worker: &str, stage: Stage) {
        let v = self
            .worker_stages_passed
            .entry(worker.to_owned())
            .or_default();
        if !v.contains(&stage) {
            v.push(stage);
        }
    }

    /// The registered worker named `name`.
    pub fn worker(&self, name: &str) -> Option<&WorkerEntry> {
        self.workers.iter().find(|w| w.name == name)
    }
}

/// The components of a project-relative `/`-separated path: `None` when it is
/// absolute or climbs out with `..`; `.` and empty parts are dropped.
fn components(path: &str) -> Option<Vec<&str>> {
    let path = path.trim();
    if path.starts_with('/') || path.starts_with('\\') || path.as_bytes().get(1) == Some(&b':') {
        return None;
    }
    let mut out = Vec::new();
    for part in path.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => return None,
            p => out.push(p),
        }
    }
    Some(out)
}

fn is_prefix(prefix: &[&str], of: &[&str]) -> bool {
    of.len() >= prefix.len() && of[..prefix.len()] == *prefix
}

impl Scope {
    /// The scope's own root as components (empty for a repository scope); `None` for a
    /// module or service scope whose path is absolute or climbs out with `..`, which
    /// matches nothing (fail closed) rather than the whole repository.
    fn root(&self) -> Option<Vec<&str>> {
        match self.kind {
            ScopeKind::Repository => Some(Vec::new()),
            ScopeKind::Module | ScopeKind::Service => components(&self.path),
        }
    }

    fn excluded_hit(&self, rel: &[&str]) -> bool {
        self.excluded
            .iter()
            .any(|e| components(e).is_some_and(|ex| !ex.is_empty() && is_prefix(&ex, rel)))
    }

    /// Checks that the scope says what it means: a module or service path is a
    /// non-empty path relative to the project root without `..`, and every excluded
    /// path is a plain relative path (no `..`, no absolute path, no glob characters,
    /// which are not supported). The error names the offending value. Existence under
    /// the project is checked by the caller, which knows the project root.
    pub fn validate(&self) -> Result<(), String> {
        if self.kind != ScopeKind::Repository {
            match components(&self.path) {
                Some(c) if !c.is_empty() => {}
                _ => {
                    return Err(format!(
                        "scope path {:?} must be a non-empty path relative to the project root, without ..",
                        self.path
                    ));
                }
            }
        }
        for e in &self.excluded {
            match components(e) {
                Some(c) if !c.is_empty() => {}
                _ => {
                    return Err(format!(
                        "excluded path {e:?} must be a non-empty path relative to the project root, without .."
                    ));
                }
            }
            if e.contains(['*', '?', '[', '{']) {
                return Err(format!(
                    "excluded path {e:?} has a glob character; exclusions are plain paths (a directory excludes everything under it)"
                ));
            }
        }
        Ok(())
    }

    /// Whether a project-relative path (`/` separators) is inside the scope: for a
    /// `repository` scope everything except the excluded paths; for a `module` or
    /// `service` scope what lies under `path`, minus the excluded paths. Matching is
    /// by path component (`src/app` does not include `src/application`); a path with
    /// `..` or an absolute path is never inside.
    pub fn includes(&self, rel_path: &str) -> bool {
        let Some(rel) = components(rel_path) else {
            return false;
        };
        self.root().is_some_and(|root| is_prefix(&root, &rel)) && !self.excluded_hit(&rel)
    }

    /// Whether a walk should descend into the project-relative directory `rel_dir`:
    /// it is inside the scope, or an ancestor of the scope's path, and not excluded.
    pub fn may_contain(&self, rel_dir: &str) -> bool {
        let Some(rel) = components(rel_dir) else {
            return false;
        };
        let Some(root) = self.root() else {
            return false;
        };
        (is_prefix(&root, &rel) || is_prefix(&rel, &root)) && !self.excluded_hit(&rel)
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
