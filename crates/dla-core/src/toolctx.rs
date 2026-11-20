//! The context object every stage crate receives, and the helpers that write through
//! the ledger and the coverage manifest.
//!
//! # Design: the crate does the writes
//!
//! `dla probe`, `dla sweep`, `dla census`, `dla workload discover` and `dla install`
//! each call ONE public function in their crate. That function receives a
//! [`ToolCtx`] and performs its own ledger and coverage writes with the helpers here
//! ([`ToolCtx::add_evidence`], [`CandidateWriter`], [`CoverageManifest::update`]). It
//! returns a [`StageOutcome`], a small bounded summary the CLI prints as the
//! command's one JSON object, or a [`Refusal`] the CLI prints with exit code 2.
//!
//! Rules every stage follows:
//!
//! * one evidence record per tool run, with a bounded `result` and a `raw` pointer to
//!   the full output under `.dla/<run>/<stage>/` ([`ToolCtx::stage_dir`]);
//! * every candidate cites that evidence in `supports` ([`Candidate::evidence`]);
//! * caches and scratch files go under [`ToolCtx::tmp_dir`], never `HOME` and never
//!   the project source;
//! * the same `(origin, site)` is added at most once per run ([`CandidateWriter`]).

use std::{
    collections::{HashMap, HashSet},
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    CoverageManifest, FindingAnnotation, Ledger, NewFinding, Origin, Refusal, RunState,
    consent::format_tiers,
    error::{Error, codes},
    evidence::{Authority, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool, Unknown},
    finding::{Holder, Region, check_site},
    fsutil::reserve_numbered,
    harness::{HARNESS_DIR, HarnessManifest},
    ids::{EvidenceId, FindingId},
    paths,
    series::{SERIES_DIR, next_number},
    state::{ConsentGiver, DLA_DIR, ProcessRecord, Scope, now_utc},
    trend::TREND_DIR,
    vendored::{Defaults, StaticScope},
};

/// Directory of alloc-test reports (`alloc/A<n>.json`).
pub const ALLOC_DIR: &str = "alloc";
/// Directory of attach results (`attach/P<n>/`).
pub const ATTACH_DIR: &str = "attach";
/// Directory of verifier bundles (`bundle/<finding>/`).
pub const BUNDLE_DIR: &str = "bundle";

/// The fix of a refusal for a spent deadline: more time (core only, and asked of the user
/// first in an interactive run), else close what cannot be measured.
pub const DEADLINE_FIX: &str = "dla run extend 30m   (core only; in an interactive run ask the user first)   or   dla ledger resolve <F> --as unresolved --reason time_budget --remedy \"<text>\"";

/// The fix of a spent deadline where extending is not on offer: a worker (only the core
/// extends, and it waits on the workers) or an unattended run (it never extends). It
/// closes what cannot be measured.
pub const DEADLINE_FIX_CLOSE: &str = "dla ledger resolve <F> --as unresolved --reason time_budget --remedy \"<text>\"   (this run does not extend its budget: only the core of an interactive run does)";

/// File of the run directory where each time-budget refusal of a tool is noted, one JSON
/// line `{"ts", "worker", "none_fits"}` each (`worker` is null for the core; `none_fits` is
/// true when nothing smaller would have fit).
pub const TIME_REFUSALS_FILE: &str = "time_refusals.jsonl";

/// True when a tool refused a command for time after `since`, whether or not a smaller
/// command was on offer: in the core (`worker` None), only the core's own refusals (a worker's
/// never count); in a worker (`worker` is its name), that worker's own. A refusal made before
/// the latest `dla run extend` no longer counts: the extension gave the time back. The ledger
/// and the final gate use [`nothing_fits_since`] instead: a refusal that names a smaller
/// command leaves the measurement possible, so it does not justify `time_budget`.
pub fn time_refused_since(
    run_dir: &Path,
    worker: Option<&str>,
    since: chrono::DateTime<chrono::Utc>,
) -> bool {
    refused_since(run_dir, worker, since, false)
}

/// Like [`time_refused_since`], but only a refusal that said nothing smaller fits (the
/// deadline passed, or even the smallest run is too long) counts. Such a refusal puts the
/// run at the floor of the budget whatever time is left.
pub fn nothing_fits_since(
    run_dir: &Path,
    worker: Option<&str>,
    since: chrono::DateTime<chrono::Utc>,
) -> bool {
    refused_since(run_dir, worker, since, true)
}

fn refused_since(
    run_dir: &Path,
    worker: Option<&str>,
    since: chrono::DateTime<chrono::Utc>,
    only_none_fits: bool,
) -> bool {
    let Ok(text) = fs::read_to_string(run_dir.join(TIME_REFUSALS_FILE)) else {
        return false;
    };
    let extended = RunState::load(run_dir)
        .ok()
        .and_then(|s| s.budget.extensions.iter().map(|e| e.at).max());
    let since = extended.map_or(since, |at| since.max(at));
    text.lines().any(|l| {
        let Ok(v) = serde_json::from_str::<Value>(l) else {
            return false;
        };
        let Some(ts) = v
            .get("ts")
            .and_then(Value::as_str)
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        else {
            return false;
        };
        if only_none_fits && v.get("none_fits").and_then(Value::as_bool) != Some(true) {
            return false;
        }
        let by = v.get("worker").and_then(Value::as_str);
        // A refusal at the very instant of the extension predates it.
        ts >= since && (extended.is_none_or(|at| ts > at)) && by == worker
    })
}

/// True when the run is unattended: its consent came from the runner (`DLA_CONSENT_TIERS`
/// at init). A consent the user gave through `dla consent` does not make a run unattended.
pub fn is_unattended(state: &RunState) -> bool {
    state
        .consent
        .as_ref()
        .is_some_and(|c| c.given_by == ConsentGiver::Runner)
}

/// Which fix a spent deadline gets: [`DEADLINE_FIX`] for the core of an interactive run,
/// [`DEADLINE_FIX_CLOSE`] for a worker or an unattended run ([`is_unattended`]: the runner
/// gave the consent at init; every interactive run has a consent object too, given by the
/// user at intake).
/// `state` is `None` when `state.json` cannot be read (the interactive fix then).
pub fn deadline_fix(worker: bool, state: Option<&RunState>) -> &'static str {
    if worker || state.is_some_and(is_unattended) {
        DEADLINE_FIX_CLOSE
    } else {
        DEADLINE_FIX
    }
}

/// Where a stage runs and what it may do.
#[derive(Debug, Clone)]
pub struct ToolCtx {
    /// Absolute project root (the directory under analysis).
    pub project: PathBuf,
    /// `<project>/.dla/<run-id>`.
    pub run_dir: PathBuf,
    /// Run ID.
    pub run_id: String,
    /// Commit hash of the project's HEAD, or the project path when none is readable;
    /// recorded as the `repo` source of every evidence record.
    pub source_commit: String,
    /// Value of the global `--data` flag, if given.
    pub data_flag: Option<PathBuf>,
    /// Worker name in worker mode (global `--worker` / `DLA_WORKER`): ledger writes go
    /// to `ledger.<worker>.jsonl` and IDs carry the `<worker>/` prefix.
    pub worker: Option<String>,
}

impl ToolCtx {
    /// Plugin data files directory (`data/`): `--data`, else `$DLA_PLUGIN_ROOT/data`,
    /// else the enclosing checkout's `data/`; a refusal naming `--data` otherwise.
    pub fn data_files_dir(&self) -> Result<PathBuf, Refusal> {
        paths::data_files_dir(self.data_flag.as_deref())
    }

    /// `$DLA_DATA_DIR/tools`, where `dla install` puts analyzers; `None` when
    /// `$DLA_DATA_DIR` is unset.
    pub fn tools_dir(&self) -> Option<PathBuf> {
        paths::tools_dir()
    }

    /// `.dla/<run>/<rel>`, project-relative with `/` separators (what `raw` and
    /// artifact pointers use is relative to the run dir; this one is for CLI output).
    pub fn pointer(&self, rel: &str) -> String {
        format!("{DLA_DIR}/{}/{rel}", self.run_id)
    }

    /// Creates and returns `<run>/<stage>` (for example `sweep`, `probe`).
    pub fn stage_dir(&self, stage: &str) -> Result<PathBuf, Refusal> {
        make_dir(&self.run_dir.join(stage))
    }

    /// Creates and returns `<run>/tmp`, the home of tool caches and scratch output; in
    /// worker mode `<run>/tmp/<worker>`, so workers running a stage at the same time
    /// never share (and clobber) a database, SARIF file or inflate temp file.
    pub fn tmp_dir(&self) -> Result<PathBuf, Refusal> {
        let base = self.run_dir.join("tmp");
        match self.worker.as_deref() {
            Some(w) => make_dir(&base.join(w)),
            None => make_dir(&base),
        }
    }

    /// The consent tiers recorded in `state.json` (empty when no consent was given).
    pub fn consent_tiers(&self) -> Result<Vec<u8>, Error> {
        Ok(RunState::load(&self.run_dir)?
            .consent
            .map(|c| c.tiers)
            .unwrap_or_default())
    }

    /// Refuses unless consent includes `tier`. The refusal names the tier needed. When
    /// no consent was given yet, the fix is the run's one consent question,
    /// `dla consent --tiers 1-4`, which covers the tier. When consent was already given
    /// (there is one consent per run), the fix is a new run, whose consent covers the
    /// tiers given so far plus the one needed.
    pub fn require_tier(&self, tier: u8, what: &str) -> Result<(), Refusal> {
        let state = RunState::load(&self.run_dir).map_err(|e| e.to_refusal())?;
        let Some(consent) = state.consent else {
            // Consent is the run's one decision, taken by the user or the runner:
            // `dla consent` refuses in worker mode, so a worker is pointed at the
            // core call.
            let fix = if self.worker.is_some() {
                format!(
                    "ask the core to run: dla --run {} consent --tiers 1-4   (without --worker)",
                    self.run_id
                )
            } else {
                "dla consent --tiers 1-4".to_owned()
            };
            return Err(Refusal::new(
                codes::CONSENT_REQUIRED,
                format!("{what} needs consent tier {tier}, and this run has no consent yet"),
                format!("consent tier {tier} in state.json"),
                fix,
            ));
        };
        if consent.tiers.contains(&tier) {
            return Ok(());
        }
        let mut wanted = consent.tiers.clone();
        wanted.push(tier);
        let wanted = format_tiers(&wanted);
        Err(Refusal::new(
            codes::CONSENT_REQUIRED,
            format!(
                "{what} needs consent tier {tier}, but this run's consent covers only tier(s) {}; a run has one consent, so a new run is needed",
                format_tiers(&consent.tiers)
            ),
            format!("consent tier {tier} in state.json"),
            format!("dla run init   then   dla consent --tiers {wanted}"),
        ))
    }

    /// Reserves `<prefix><n>` in `dir` (see [`reserve_numbered`]); when the filesystem
    /// refuses, falls back to the next unreserved number so callers keep their
    /// infallible signature (their own write then reports the real I/O error).
    fn reserve(dir: &Path, prefix: char, suffix: &str, dirs: bool) -> String {
        let n = reserve_numbered(dir, prefix, suffix, dirs)
            .map_or_else(|_| next_number(dir, prefix, suffix, dirs), |(n, _)| n);
        format!("{prefix}{n}")
    }

    /// The ID the next series gets: `S<n>`. The id is reserved: `series/S<n>.json`
    /// exists (empty) when this returns, and a concurrent caller gets the next number.
    /// An empty file counts as absent for readers ([`crate::fsutil::is_written`]).
    pub fn next_series_id(&self) -> String {
        Self::reserve(&self.run_dir.join(SERIES_DIR), 'S', ".json", false)
    }

    /// The ID the next trend report gets: `T<n>`; `trend/T<n>.json` is reserved.
    pub fn next_trend_id(&self) -> String {
        Self::reserve(&self.run_dir.join(TREND_DIR), 'T', ".json", false)
    }

    /// The ID the next harness gets: `H<n>`; the directory `harness/H<n>` is reserved.
    pub fn next_harness_id(&self) -> String {
        Self::reserve(&self.run_dir.join(HARNESS_DIR), 'H', "", true)
    }

    /// The ID the next alloc-test report gets: `A<n>`; `alloc/A<n>.json` is reserved.
    pub fn next_alloc_id(&self) -> String {
        Self::reserve(&self.run_dir.join(ALLOC_DIR), 'A', ".json", false)
    }

    /// The ID the next attach gets: `P<n>`; the directory `attach/P<n>` is reserved.
    pub fn next_attach_id(&self) -> String {
        Self::reserve(&self.run_dir.join(ATTACH_DIR), 'P', "", true)
    }

    /// Reserves and returns the directory for a finding's bundle: `bundle/<name>`,
    /// or `bundle/<name>.2`, `.3`, ... when earlier bundles of the same finding exist.
    /// `name` is the finding id with `/` replaced by `_`.
    pub fn next_bundle_dir(&self, name: &str) -> Result<PathBuf, Refusal> {
        let base = self.run_dir.join(BUNDLE_DIR);
        fs::create_dir_all(&base).map_err(|e| Error::io(&base, e).to_refusal())?;
        let mut n = 1u32;
        loop {
            let path = if n == 1 {
                base.join(name)
            } else {
                base.join(format!("{name}.{n}"))
            };
            match fs::create_dir(&path) {
                Ok(()) => return Ok(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => n += 1,
                Err(e) => return Err(Error::io(&path, e).to_refusal()),
            }
        }
    }

    /// The manifest of harness `id`.
    pub fn harness(&self, id: &str) -> Result<HarnessManifest, Refusal> {
        HarnessManifest::load(&self.run_dir, id).map_err(|e| e.to_refusal())
    }

    /// `Source::Artifact` for the file at `rel`, a path relative to the run directory:
    /// the path (with `/` separators) and the SHA-256 of the file's bytes.
    pub fn artifact_source(&self, rel: &str) -> Result<Source, Refusal> {
        let path = self.run_dir.join(rel);
        let bytes = fs::read(&path).map_err(|e| Error::io(&path, e).to_refusal())?;
        Ok(Source::Artifact {
            path: rel.replace('\\', "/"),
            sha256: hex::encode(Sha256::digest(&bytes)),
        })
    }

    /// `Source::Process` for a process observed live.
    pub fn process_source(pid: u32, start_time: &str) -> Source {
        Source::Process {
            pid,
            start_time: start_time.to_owned(),
        }
    }

    /// Records a process in `state.json`'s `processes`, under the run lock. A record
    /// without a `worker` gets this context's worker.
    pub fn register_process(&self, mut record: ProcessRecord) -> Result<(), Refusal> {
        if record.worker.is_none() {
            record.worker.clone_from(&self.worker);
        }
        RunState::update(&self.run_dir, |state| {
            state.processes.push(record);
            Ok(())
        })
        .map_err(|e| e.to_refusal())
    }

    /// Marks the registered process `pid` (with `start_time`, when both sides have one)
    /// as stopped now. Returns whether a running record matched. Runs under the run
    /// lock.
    pub fn mark_process_stopped(
        &self,
        pid: u32,
        start_time: Option<&str>,
    ) -> Result<bool, Refusal> {
        RunState::update(&self.run_dir, |state| {
            let hit = state.processes.iter_mut().rev().find(|p| {
                p.pid == pid
                    && p.stopped_at.is_none()
                    && match (start_time, p.start_time.as_deref()) {
                        (Some(a), Some(b)) => a == b,
                        _ => true,
                    }
            });
            Ok(hit.map(|p| p.stopped_at = Some(now_utc())).is_some())
        })
        .map_err(|e| e.to_refusal())
    }

    /// Replays the run's ledger.
    pub fn ledger(&self) -> Result<Ledger, Refusal> {
        Ledger::load_for(&self.run_dir, self.worker.as_deref()).map_err(|e| e.to_refusal())
    }

    /// The run's scope from `state.json`; `None` when the run has none (everything is
    /// in scope). Loop callers read it once and call [`Scope::includes`].
    pub fn scope(&self) -> Result<Option<Scope>, Refusal> {
        Ok(RunState::load(&self.run_dir)
            .map_err(|e| e.to_refusal())?
            .scope)
    }

    /// The scope for the static stages (probe, sweep, census, workload discovery): the run's
    /// scope plus the default exclusions of vendored and generated code, minus the paths
    /// the user brought back (`dla run init --include`). The dynamic stages keep using
    /// [`ToolCtx::scope`] and [`ToolCtx::in_scope`]: a leak in such code that the workload
    /// triggers is still measured.
    pub fn static_scope(&self) -> Result<StaticScope, Refusal> {
        let state = RunState::load(&self.run_dir).map_err(|e| e.to_refusal())?;
        Ok(StaticScope {
            defaults: Defaults::for_project(&self.project, &state.include),
            scope: state.scope,
        })
    }

    /// Whether a project-relative path (`/` separators) is inside the run's scope.
    /// A run without a scope, or one whose state cannot be read, includes everything.
    pub fn in_scope(&self, rel: &str) -> bool {
        match self.scope() {
            Ok(Some(scope)) => scope.includes(rel),
            _ => true,
        }
    }

    /// Adds `count` to the number of source files of `runtime` skipped because they
    /// lie outside the scope (`coverage.json`, under the run lock).
    pub fn add_out_of_scope(&self, runtime: &str, count: u64) -> Result<(), Refusal> {
        if count == 0 {
            return Ok(());
        }
        CoverageManifest::update(&self.run_dir, |m| {
            let w = m.worker_mut(runtime);
            w.out_of_scope = w.out_of_scope.saturating_add(count);
        })
        .map(|_| ())
        .map_err(|e| e.to_refusal())
    }

    /// Refuses with code `time-budget` when the worker's deadline (worker mode, from the
    /// registry; an unregistered worker has only the run's) or the run's deadline has
    /// passed. `what` names the step for the message. The fix is [`deadline_fix`]: more time
    /// for the core of an interactive run, else the open finding recorded as unresolved for
    /// lack of time.
    pub fn require_time(&self, what: &str) -> Result<(), Refusal> {
        let state = RunState::load(&self.run_dir).map_err(|e| e.to_refusal())?;
        let now = now_utc();
        let mut limits = Vec::new();
        if let Some(w) = self.worker.as_deref().and_then(|w| state.worker(w)) {
            limits.push((format!("worker {}", w.name), w.deadline));
        }
        limits.push(("run".to_owned(), state.budget.deadline));
        for (who, deadline) in limits {
            if now > deadline {
                let when = deadline.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                self.note_time_refusal(true);
                return Err(Refusal::new(
                    codes::TIME_BUDGET,
                    format!("{what} cannot start: the {who} deadline {when} has passed"),
                    format!("time before the {who} deadline {when}"),
                    deadline_fix(self.worker.is_some(), Some(&state)),
                ));
            }
        }
        Ok(())
    }

    /// Notes that a command was refused for time, in the one place every time-budget refusal
    /// of a tool passes through ([`Self::require_time`] and [`Self::time_budget_refusal`]).
    /// `none_fits` records that nothing smaller would have fit. `dla ledger resolve --reason
    /// time_budget` accepts the reason while time is left only when a note with `none_fits`
    /// true is newer than the finding ([`nothing_fits_since`]); a note that named a smaller
    /// command does not count. A failure to write the note is ignored: the refusal itself is
    /// what the caller returns.
    fn note_time_refusal(&self, none_fits: bool) {
        let line = serde_json::json!({
            "ts": now_utc().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "worker": self.worker,
            "none_fits": none_fits,
        });
        let path = self.run_dir.join(TIME_REFUSALS_FILE);
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path) {
            use std::io::Write;
            let _ = writeln!(f, "{line}");
        }
    }

    /// Time left before the nearest deadline (the worker's, in worker mode, and the
    /// run's): zero when one has passed, `None` when `state.json` cannot be read (a
    /// caller then does not limit itself). `what` of the deadline is returned with it.
    pub fn time_left(&self) -> Option<(std::time::Duration, String)> {
        let state = RunState::load(&self.run_dir).ok()?;
        let now = now_utc();
        let mut limits = Vec::new();
        if let Some(w) = self.worker.as_deref().and_then(|w| state.worker(w)) {
            limits.push((format!("worker {}", w.name), w.deadline));
        }
        limits.push(("run".to_owned(), state.budget.deadline));
        limits
            .into_iter()
            .map(|(who, d)| {
                let when = d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                (
                    (d - now).to_std().unwrap_or(std::time::Duration::ZERO),
                    format!("{who} deadline {when}"),
                )
            })
            .min_by_key(|(left, _)| *left)
    }

    /// The `time-budget` refusal of a command whose estimated duration does not fit what
    /// is left. `what` names the command, `estimate_secs` and `usable_secs` are the
    /// estimate and the time the command may use, `suggestion` is the smaller command to
    /// run instead (empty: none fits).
    pub fn time_budget_refusal(
        &self,
        what: &str,
        estimate_secs: u64,
        usable_secs: u64,
        deadline: &str,
        suggestion: &str,
    ) -> Refusal {
        self.note_time_refusal(suggestion.is_empty());
        let fix = if suggestion.is_empty() {
            let state = RunState::load(&self.run_dir).ok();
            format!(
                "even the smallest run does not fit: {}",
                deadline_fix(self.worker.is_some(), state.as_ref())
            )
        } else {
            suggestion.to_owned()
        };
        Refusal::new(
            codes::TIME_BUDGET,
            format!(
                "{what} is estimated at {estimate_secs} s but only {usable_secs} s remain before the {deadline}, so it was not run"
            ),
            format!("a run of at most {usable_secs} s"),
            fix,
        )
    }

    /// Appends one evidence record and returns its ID. `raw` is relative to the run
    /// directory (for example `sweep/staticcheck.json`).
    pub fn add_evidence(
        &self,
        ledger: &mut Ledger,
        spec: EvidenceSpec,
    ) -> Result<EvidenceId, Refusal> {
        let line = ledger
            .add_evidence(NewEvidence {
                source: spec.source.unwrap_or_else(|| Source::Repo {
                    commit: self.source_commit.clone(),
                    path: None,
                }),
                tool: Tool {
                    name: spec.tool,
                    version: spec.tool_version,
                },
                authority: spec.authority,
                kind: spec.kind,
                result: spec.result,
                raw: spec.raw,
                coverage: spec.coverage,
                limitations: spec.limitations,
                unknowns: spec.unknowns,
            })
            .map_err(|e| e.to_refusal())?;
        match line.record {
            crate::Record::Evidence(e) => Ok(e.evidence_id),
            _ => Err(Refusal::new(
                codes::MALFORMED,
                "ledger returned a non-evidence line",
                "an evidence record",
                "report this as a dla bug",
            )),
        }
    }
}

fn make_dir(p: &Path) -> Result<PathBuf, Refusal> {
    fs::create_dir_all(p).map_err(|e| Error::io(p, e).to_refusal())?;
    Ok(p.to_path_buf())
}

/// What a stage records as evidence.
#[derive(Debug, Clone)]
pub struct EvidenceSpec {
    /// Evidence kind, for example `probe`, `static-hit`, `workload` or `growth-series`.
    pub kind: EvidenceKind,
    /// Tool name (the backend, or `dla`).
    pub tool: String,
    /// Tool version (`unknown` when it cannot be read).
    pub tool_version: String,
    /// How the evidence was obtained (`static` by default; `test`, `live` or `dump` for
    /// dynamic evidence).
    pub authority: Authority,
    /// Bounded summary (serialized size under [`crate::MAX_RESULT_BYTES`]).
    pub result: Value,
    /// Full output, relative to the run directory.
    pub raw: Option<String>,
    /// How much of the subject it covers.
    pub coverage: EvidenceCoverage,
    /// Known limits.
    pub limitations: Vec<String>,
    /// Where the evidence came from, when not the project's commit: an artifact file
    /// ([`ToolCtx::artifact_source`]) or a live process ([`ToolCtx::process_source`]).
    pub source: Option<Source>,
    /// Open questions the evidence leaves.
    pub unknowns: Vec<Unknown>,
}

impl EvidenceSpec {
    /// Static evidence of `kind` from `tool`, complete coverage, no limitations.
    pub fn new(kind: EvidenceKind, tool: &str, tool_version: &str, result: Value) -> Self {
        Self {
            kind,
            tool: tool.to_owned(),
            tool_version: tool_version.to_owned(),
            authority: Authority::Static,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: Vec::new(),
            source: None,
            unknowns: Vec::new(),
        }
    }
}

/// A candidate finding as a stage describes it.
#[derive(Debug, Clone, Default)]
pub struct Candidate {
    /// `detector:<backend>/<rule>` or `census`.
    pub origin: String,
    /// The evidence record of the run that produced it (becomes `supports`).
    pub evidence: Vec<EvidenceId>,
    /// `file:line`, project-relative.
    pub site: String,
    /// Holder class (census).
    pub holder: Option<Holder>,
    /// Owner (census).
    pub owner: Option<String>,
    /// Region (census).
    pub region: Option<Region>,
    /// Archetype tag; the empty string and `None` both mean no suggestion.
    pub suggested_archetype: Option<String>,
    /// What the origin says about the site (the rule's message); kept on the finding as a
    /// note when the site already has a finding of another origin.
    pub note: Option<String>,
}

/// Maps a `(file, line)` to the line that stands for its declaration (the `def` line of a
/// decorated function), so that a hit on a decorator and a census entry on the `def` are
/// one site. The identity maps every line to itself.
#[derive(Clone)]
pub struct SiteAnchor(std::sync::Arc<AnchorFn>);

type AnchorFn = dyn Fn(&str, u32) -> u32 + Send + Sync;

impl SiteAnchor {
    /// An anchor computed by `f`.
    pub fn new(f: impl Fn(&str, u32) -> u32 + Send + Sync + 'static) -> Self {
        Self(std::sync::Arc::new(f))
    }

    /// The anchor line of `line` in `file`.
    pub fn of(&self, file: &str, line: u32) -> u32 {
        (self.0)(file, line)
    }
}

impl fmt::Debug for SiteAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SiteAnchor")
    }
}

/// What [`CandidateWriter::add_outcome`] did with a candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Added {
    /// A new finding.
    New(FindingId),
    /// The site already had a finding of another origin: the candidate's origin, evidence,
    /// holder, owner, region and message were recorded on it (the finding's ID).
    Joined(FindingId),
    /// Nothing was added: a duplicate of the same origin and site, a site outside the scope or
    /// build logic.
    Skipped,
}

/// Which side of a site a static origin is on: a census entry, or a rule or analyzer hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Census,
    Detector,
}

impl Side {
    fn of(origin: &str) -> Option<Self> {
        if origin == "census" {
            Some(Self::Census)
        } else if origin.starts_with("detector:") {
            Some(Self::Detector)
        } else {
            None
        }
    }
}

/// The findings at one site, by the side that raised them.
#[derive(Debug, Default, Clone)]
struct AtSite {
    census: Option<FindingId>,
    detector: Option<FindingId>,
}

impl AtSite {
    fn slot(&mut self, side: Side) -> &mut Option<FindingId> {
        match side {
            Side::Census => &mut self.census,
            Side::Detector => &mut self.detector,
        }
    }
}

/// Adds candidates to a ledger, skipping an `(origin, site)` pair already present, and
/// recording a census entry at the site of a rule or analyzer finding (or the reverse) on that
/// finding: one site is one finding. Two rules or analyzers at one site stay two findings.
#[derive(Debug, Default)]
pub struct CandidateWriter {
    seen: HashSet<(String, String)>,
    anchor: Option<SiteAnchor>,
    by_site: Option<HashMap<(String, u32), AtSite>>,
    added: u64,
    joined: u64,
    skipped: u64,
    scope: Option<Scope>,
    defaults: Option<Defaults>,
    out_of_scope: u64,
    left_out: u64,
    build_logic: u64,
}

impl CandidateWriter {
    /// Starts from the `(origin, site)` pairs already in `ledger`.
    pub fn new(ledger: &Ledger) -> Self {
        let seen = ledger
            .findings()
            .filter_map(|f| f.site.clone().map(|s| (f.origin.to_string(), s)))
            .collect();
        Self {
            seen,
            anchor: None,
            by_site: None,
            added: 0,
            joined: 0,
            skipped: 0,
            scope: None,
            defaults: None,
            out_of_scope: 0,
            left_out: 0,
            build_logic: 0,
        }
    }

    /// Like [`CandidateWriter::new`], restricted to the run's scope: a candidate whose
    /// site file is outside it is skipped and counted, never added.
    pub fn for_ctx(ctx: &ToolCtx, ledger: &Ledger) -> Result<Self, Refusal> {
        Ok(Self::new(ledger).with_scope(ctx.scope()?))
    }

    /// Like [`CandidateWriter::for_ctx`] for the static stages: a candidate in vendored or
    /// generated code that the user did not bring back is skipped and counted in
    /// [`Self::left_out`].
    pub fn for_static(ctx: &ToolCtx, ledger: &Ledger) -> Result<Self, Refusal> {
        let st = ctx.static_scope()?;
        let mut w = Self::new(ledger).with_scope(st.scope);
        w.defaults = Some(st.defaults);
        Ok(w)
    }

    /// Restricts the writer to `scope` (`None`: no restriction).
    #[must_use]
    pub fn with_scope(mut self, scope: Option<Scope>) -> Self {
        self.scope = scope;
        self
    }

    /// Uses `anchor` to decide which lines are one site (see [`SiteAnchor`]).
    #[must_use]
    pub fn with_anchor(mut self, anchor: SiteAnchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    /// Adds `c` unless `(origin, site)` exists. Returns the new ID, or `None` when no new
    /// finding was added: a duplicate, a site outside the scope, a build-logic site (which
    /// is counted in [`Self::build_logic`]) or a site that already has a finding of another
    /// origin (see [`Self::add_outcome`]).
    pub fn add(&mut self, ledger: &mut Ledger, c: Candidate) -> Result<Option<FindingId>, Refusal> {
        Ok(match self.add_outcome(ledger, c)? {
            Added::New(id) => Some(id),
            Added::Joined(_) | Added::Skipped => None,
        })
    }

    fn site_key(&self, site: &str) -> Option<(String, u32)> {
        let (file, line) = site.rsplit_once(':')?;
        let line: u32 = line.parse().ok()?;
        let line = self.anchor.as_ref().map_or(line, |a| a.of(file, line));
        Some((file.to_owned(), line))
    }

    /// The findings of the ledger by site and side (a sibling hypothesis, a finding merged
    /// into another, a finding of another worker or of the core and a finding of any other
    /// origin are left out). A finding that joined
    /// the other side counts on both.
    fn sites(&mut self, ledger: &Ledger) -> &mut HashMap<(String, u32), AtSite> {
        if self.by_site.is_none() {
            let mut map: HashMap<(String, u32), AtSite> = HashMap::new();
            for f in ledger.findings() {
                // A worker records on its own findings only, the core on the core's.
                if f.parent.is_some()
                    || ledger.merged_into(&f.id).is_some()
                    || f.id.prefix() != ledger.worker()
                {
                    continue;
                }
                let Some(key) = f.site.as_deref().and_then(|s| self.site_key(s)) else {
                    continue;
                };
                for side in std::iter::once(&f.origin)
                    .chain(&f.also_origins)
                    .filter_map(|o| Side::of(o.as_str()))
                {
                    map.entry(key.clone())
                        .or_default()
                        .slot(side)
                        .get_or_insert_with(|| f.id.clone());
                }
            }
            self.by_site = Some(map);
        }
        self.by_site.as_mut().expect("just filled")
    }

    /// Like [`Self::add`], and says what happened to a candidate whose site already has a
    /// finding of another static origin: the candidate's origin, evidence, holder, owner,
    /// region (those the finding lacks) and message are recorded on that finding with one
    /// annotation, and no finding is added.
    pub fn add_outcome(&mut self, ledger: &mut Ledger, c: Candidate) -> Result<Added, Refusal> {
        check_site(&c.site).map_err(|m| {
            Refusal::new(
                codes::INVALID_SITE,
                m,
                "a site written file:line",
                "report this as a dla bug",
            )
        })?;
        // Build logic (Gradle scripts, buildSrc) runs in the build tool's process, not in the
        // measured one: never a candidate, whichever stage or backend found it.
        if let Some((file, _)) = c.site.rsplit_once(':')
            && paths::is_build_logic(file)
        {
            self.build_logic += 1;
            return Ok(Added::Skipped);
        }
        if let Some(scope) = &self.scope
            && let Some((file, _)) = c.site.rsplit_once(':')
            && !scope.includes(file)
        {
            self.out_of_scope += 1;
            return Ok(Added::Skipped);
        }
        if let Some(defaults) = &self.defaults
            && let Some((file, _)) = c.site.rsplit_once(':')
            && defaults.excluded(file).is_some()
        {
            self.left_out += 1;
            return Ok(Added::Skipped);
        }
        let key = (c.origin.clone(), c.site.clone());
        if self.seen.contains(&key) {
            self.skipped += 1;
            return Ok(Added::Skipped);
        }
        let origin = Origin::parse(&c.origin).map_err(|e| {
            Refusal::new(
                codes::INVALID_ORIGIN,
                e.to_string(),
                "a valid origin",
                "report this as a dla bug",
            )
        })?;
        let side = Side::of(&c.origin);
        let site_key = side.and_then(|_| self.site_key(&c.site));
        if let (Some(sk), Some(side)) = (&site_key, side) {
            // A census entry joins a rule or analyzer finding of its site, and a rule or
            // analyzer hit joins the census finding of its site: never two of the same side.
            let other = match side {
                Side::Census => Side::Detector,
                Side::Detector => Side::Census,
            };
            let existing = self
                .sites(ledger)
                .get_mut(sk)
                .and_then(|a| a.slot(other).clone());
            if let Some(existing) = existing {
                let out = self.join(ledger, &existing, origin, c, key)?;
                if matches!(out, Added::Joined(_)) {
                    // The finding now stands on this side as well.
                    self.sites(ledger)
                        .entry(sk.clone())
                        .or_default()
                        .slot(side)
                        .get_or_insert(existing);
                }
                return Ok(out);
            }
        }
        let mut nf = NewFinding::new(origin, c.evidence);
        nf.site = Some(c.site);
        nf.holder = c.holder;
        nf.owner = c.owner;
        nf.region = c.region;
        nf.suggested_archetype = c.suggested_archetype.filter(|a| !a.is_empty());
        let line = ledger.add_finding(nf).map_err(|e| e.to_refusal())?;
        self.seen.insert(key);
        self.added += 1;
        match line.record {
            crate::Record::FindingAdded { finding } => {
                if let (Some(sk), Some(side)) = (site_key, side) {
                    self.sites(ledger)
                        .entry(sk)
                        .or_default()
                        .slot(side)
                        .get_or_insert_with(|| finding.id.clone());
                }
                Ok(Added::New(finding.id))
            }
            _ => Ok(Added::Skipped),
        }
    }

    /// Records candidate `c` (of `origin`) on `existing`, the finding of the other side at its site.
    fn join(
        &mut self,
        ledger: &mut Ledger,
        existing: &FindingId,
        origin: Origin,
        c: Candidate,
        key: (String, String),
    ) -> Result<Added, Refusal> {
        let Some(f) = ledger.finding(existing) else {
            return Ok(Added::Skipped);
        };
        // The same origin again (an earlier run of the stage joined it): nothing to record.
        if f.origin == origin || f.also_origins.contains(&origin) {
            self.seen.insert(key);
            self.skipped += 1;
            return Ok(Added::Skipped);
        }
        let mut a = FindingAnnotation::new(existing.clone());
        a.also_origin = Some(origin.clone());
        a.also_supports = c.evidence.clone();
        let mut said: Vec<String> = Vec::new();
        // An annotation only fills what the finding lacks; a different value is said in the
        // note instead of refused.
        match (f.holder, c.holder) {
            (None, h) => a.holder = h,
            (Some(have), Some(want)) if have != want => said.push(format!("holder {want}")),
            _ => {}
        }
        match (&f.owner, &c.owner) {
            (None, o) => a.owner.clone_from(o),
            (Some(have), Some(want)) if have != want => said.push(format!("owner {want}")),
            _ => {}
        }
        match (f.region, c.region) {
            (None, r) => a.region = r,
            (Some(have), Some(want)) if have != want => said.push(format!("region {want}")),
            _ => {}
        }
        let mut note = format!("also raised by {origin} at {}", c.site);
        if let Some(m) = c.note.as_deref().map(str::trim).filter(|m| !m.is_empty()) {
            note.push_str(": ");
            note.push_str(m);
        }
        if let Some(arch) = c.suggested_archetype.as_deref().filter(|x| !x.is_empty()) {
            note.push_str(&format!("; suggests {arch}"));
        }
        if !said.is_empty() {
            note.push_str(&format!("; it names {}", said.join(", ")));
        }
        a.note = Some(note);
        ledger.annotate_record(a).map_err(|e| e.to_refusal())?;
        self.seen.insert(key);
        self.joined += 1;
        Ok(Added::Joined(existing.clone()))
    }

    /// Candidates added so far.
    pub fn added(&self) -> u64 {
        self.added
    }

    /// Candidates recorded on the finding of their site (another origin raised it first).
    pub fn joined(&self) -> u64 {
        self.joined
    }

    /// Duplicates skipped so far.
    pub fn skipped(&self) -> u64 {
        self.skipped
    }

    /// Candidates skipped so far because their site file is outside the scope.
    pub fn out_of_scope(&self) -> u64 {
        self.out_of_scope
    }

    /// Candidates skipped so far because their site file is vendored or generated code left
    /// out by default.
    pub fn left_out(&self) -> u64 {
        self.left_out
    }

    /// Candidates skipped so far because their site file is build logic.
    pub fn build_logic(&self) -> u64 {
        self.build_logic
    }
}

/// What a stage reports; the CLI prints it as the command's JSON object (bounded).
#[derive(Debug, Clone, Default, Serialize)]
pub struct StageOutcome {
    /// Evidence IDs written (one per tool run).
    pub evidence: Vec<String>,
    /// Candidates added to the ledger.
    pub candidates: u64,
    /// Candidates skipped because `(origin, site)` was already present.
    pub skipped_duplicates: u64,
    /// Run-relative-to-project pointers (`.dla/<run>/...`) to files written.
    pub artifacts: Vec<String>,
    /// Small stage-specific summary (keep it well under 1 KB).
    pub summary: Value,
}

/// The refusal a stage returns until its implementation exists.
pub fn not_implemented(what: &str) -> Refusal {
    Refusal::new(
        codes::NOT_IMPLEMENTED,
        format!("{what} is not implemented in this build"),
        what,
        "use a dla build that includes it",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CoverageManifest, EvidenceKind,
        evidence::{Authority, Unknown},
        state::{Consent, now_utc},
    };
    use serde_json::json;

    fn run(tiers: Option<Vec<u8>>) -> (tempfile::TempDir, ToolCtx) {
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join(".dla/r1");
        fs::create_dir_all(&run_dir).unwrap();
        let mut st = RunState::init(
            dir.path().to_str().unwrap(),
            Some("s"),
            3600,
            None,
            now_utc(),
        )
        .unwrap();
        st.run_id = "r1".into();
        st.consent = tiers.map(|tiers| Consent {
            tiers,
            raw_values: false,
            given_by: ConsentGiver::Runner,
            recorded_at: now_utc(),
        });
        st.save(&run_dir).unwrap();
        Ledger::create(&run_dir).unwrap();
        let ctx = ToolCtx {
            project: dir.path().to_path_buf(),
            run_dir,
            run_id: "r1".into(),
            source_commit: "abc".into(),
            data_flag: None,
            worker: None,
        };
        (dir, ctx)
    }

    #[test]
    fn require_tier_refuses_without_consent_and_names_the_fix() {
        let (_d, ctx) = run(None);
        let e = ctx.require_tier(1, "building").unwrap_err();
        assert_eq!(e.code, codes::CONSENT_REQUIRED);
        assert_eq!(e.fix, "dla consent --tiers 1-4");
        assert!(e.message.contains("tier 1"), "{}", e.message);
        let e = ctx.require_tier(3, "attaching").unwrap_err();
        assert_eq!(e.fix, "dla consent --tiers 1-4");
        assert!(e.message.contains("tier 3"), "{}", e.message);
        let (_d, ctx) = run(Some(vec![1, 2]));
        ctx.require_tier(1, "building").unwrap();
    }

    #[test]
    fn require_tier_after_consent_says_a_new_run_is_needed() {
        let (_d, ctx) = run(Some(vec![2]));
        let e = ctx.require_tier(1, "building").unwrap_err();
        assert_eq!(e.code, codes::CONSENT_REQUIRED);
        assert_eq!(e.fix, "dla run init   then   dla consent --tiers 1,2");
        assert!(
            e.message.contains("tier 1") && e.message.contains("new run"),
            "{}",
            e.message
        );
        let (_d, ctx) = run(Some(vec![1, 2, 3]));
        let e = ctx.require_tier(4, "heap dump").unwrap_err();
        assert_eq!(e.fix, "dla run init   then   dla consent --tiers 1-4");
    }

    #[test]
    fn ids_are_allocated_from_the_run_directory() {
        let (_d, ctx) = run(None);
        assert_eq!(
            (
                ctx.next_series_id(),
                ctx.next_trend_id(),
                ctx.next_harness_id()
            ),
            ("S1".to_owned(), "T1".to_owned(), "H1".to_owned())
        );
        for (dir, name) in [
            ("series", "S3.json"),
            ("series", "S1.json"),
            ("trend", "T7.json"),
        ] {
            let d = ctx.run_dir.join(dir);
            fs::create_dir_all(&d).unwrap();
            fs::write(d.join(name), "{}").unwrap();
        }
        fs::create_dir_all(ctx.run_dir.join("harness/H2/work")).unwrap();
        fs::create_dir_all(ctx.run_dir.join("harness/H10")).unwrap();
        fs::write(ctx.run_dir.join("harness/H11"), "a file, not a harness").unwrap();
        assert_eq!(ctx.next_series_id(), "S4");
        assert_eq!(ctx.next_trend_id(), "T8");
        // H11 is taken by a file, so the reservation moves on.
        assert_eq!(ctx.next_harness_id(), "H12");
    }

    #[test]
    fn processes_are_registered_and_marked_stopped() {
        let (_d, ctx) = run(Some(vec![1]));
        let rec = |pid, start: &str| ProcessRecord {
            pid,
            start_time: Some(start.to_owned()),
            kind: crate::state::ProcessKind::Harness,
            command: "x".into(),
            started_at: now_utc(),
            stopped_at: None,
            role: crate::state::ProcessRole::Started,
            worker: None,
            cleanup: Vec::new(),
        };
        ctx.register_process(rec(10, "a")).unwrap();
        ctx.register_process(rec(10, "b")).unwrap();
        assert!(!ctx.mark_process_stopped(10, Some("zzz")).unwrap());
        assert!(ctx.mark_process_stopped(10, Some("a")).unwrap());
        assert!(!ctx.mark_process_stopped(10, Some("a")).unwrap());
        assert!(ctx.mark_process_stopped(10, None).unwrap());
        assert!(!ctx.mark_process_stopped(11, None).unwrap());
        let st = RunState::load(&ctx.run_dir).unwrap();
        assert_eq!(st.processes.len(), 2);
        assert!(st.processes.iter().all(|p| p.stopped_at.is_some()));
        assert_eq!(st.consent.unwrap().tiers, vec![1], "other fields survive");
    }

    #[test]
    fn evidence_can_override_the_source_and_carry_unknowns() {
        let (_d, ctx) = run(None);
        fs::create_dir_all(ctx.run_dir.join("series")).unwrap();
        fs::write(ctx.run_dir.join("series/S1.json"), "abc").unwrap();
        let mut ledger = ctx.ledger().unwrap();
        let mut spec = EvidenceSpec::new(
            EvidenceKind::GrowthSeries,
            "dla",
            "0",
            json!({"gate": "pass"}),
        );
        spec.authority = Authority::Test;
        spec.source = Some(ctx.artifact_source("series/S1.json").unwrap());
        spec.unknowns = vec![Unknown {
            question: "q".into(),
            resolve_by: "r".into(),
        }];
        let id = ctx.add_evidence(&mut ledger, spec).unwrap();
        let reloaded = ctx.ledger().unwrap();
        let e = reloaded.evidence_by_id(&id).unwrap();
        assert_eq!(
            e.source,
            Source::Artifact {
                path: "series/S1.json".into(),
                sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()
            }
        );
        assert_eq!(e.unknowns.len(), 1);
        assert!(ctx.artifact_source("series/missing.json").is_err());
        assert_eq!(
            ToolCtx::process_source(5, "t"),
            Source::Process {
                pid: 5,
                start_time: "t".into()
            }
        );
    }

    #[test]
    fn stage_dirs_live_in_the_run_directory() {
        let (_d, ctx) = run(None);
        let s = ctx.stage_dir("sweep").unwrap();
        assert_eq!(s, ctx.run_dir.join("sweep"));
        assert_eq!(ctx.tmp_dir().unwrap(), ctx.run_dir.join("tmp"));
        let mut w1 = ctx.clone();
        w1.worker = Some("jvm".into());
        let mut w2 = ctx.clone();
        w2.worker = Some("python".into());
        assert_eq!(w1.tmp_dir().unwrap(), ctx.run_dir.join("tmp").join("jvm"));
        assert_ne!(w1.tmp_dir().unwrap(), w2.tmp_dir().unwrap());
        assert!(w2.tmp_dir().unwrap().is_dir());
        assert_eq!(ctx.pointer("sweep/x.json"), ".dla/r1/sweep/x.json");
    }

    #[test]
    fn candidates_cite_evidence_and_duplicates_are_skipped() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let ev = ctx
            .add_evidence(
                &mut ledger,
                EvidenceSpec::new(EvidenceKind::StaticHit, "ruff", "0.1", json!({"hits": 2})),
            )
            .unwrap();
        let cand = |site: &str, arch: &str| Candidate {
            origin: "detector:ruff/B019".into(),
            evidence: vec![ev.clone()],
            site: site.into(),
            suggested_archetype: Some(arch.into()),
            ..Candidate::default()
        };
        let mut w = CandidateWriter::new(&ledger);
        let first = w
            .add(&mut ledger, cand("a.py:3", "L-COLLECT"))
            .unwrap()
            .unwrap();
        assert!(
            w.add(&mut ledger, cand("a.py:3", "L-COLLECT"))
                .unwrap()
                .is_none()
        );
        assert!(w.add(&mut ledger, cand("a.py:4", "")).unwrap().is_some());
        assert_eq!((w.added(), w.skipped()), (2, 1));
        let f = ledger.finding(&first).unwrap();
        assert_eq!(f.supports, std::slice::from_ref(&ev));
        assert_eq!(f.suggested_archetype.as_deref(), Some("L-COLLECT"));
        assert_eq!(f.site.as_deref(), Some("a.py:3"));
        // An empty archetype means none.
        let second = ledger
            .findings()
            .find(|f| f.site.as_deref() == Some("a.py:4"))
            .unwrap();
        assert_eq!(second.suggested_archetype, None);
        // A new writer over the reloaded ledger still skips the same pair.
        let mut reloaded = ctx.ledger().unwrap();
        let mut w2 = CandidateWriter::new(&reloaded);
        assert!(w2.add(&mut reloaded, cand("a.py:3", "")).unwrap().is_none());
    }

    #[test]
    fn a_malformed_site_is_refused() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let mut w = CandidateWriter::new(&ledger);
        let e = w
            .add(
                &mut ledger,
                Candidate {
                    origin: "census".into(),
                    site: "nolines".into(),
                    ..Candidate::default()
                },
            )
            .unwrap_err();
        assert_eq!(e.code, codes::INVALID_SITE);
    }

    fn scope(kind: crate::state::ScopeKind, path: &str, excluded: &[&str]) -> Scope {
        Scope {
            kind,
            path: path.into(),
            excluded: excluded.iter().map(|e| (*e).to_owned()).collect(),
        }
    }

    #[test]
    fn scope_matching_is_by_component() {
        use crate::state::ScopeKind::{Module, Repository, Service};
        let repo = scope(Repository, ".", &["vendor", "src/gen/"]);
        let module = scope(Module, "services/api", &["services/api/target"]);
        let svc = scope(Service, "./svc/", &[]);
        let table: &[(&Scope, &str, bool)] = &[
            (&repo, "a.py", true),
            (&repo, "src/main/A.java", true),
            (&repo, "vendor/x.go", false),
            (&repo, "vendor", false),
            (&repo, "vendors/x.go", true),
            (&repo, "src/gen/x.rs", false),
            (&repo, "src/generated/x.rs", true),
            (&repo, "./a.py", true),
            (&repo, "../a.py", false),
            (&repo, "a/../b.py", false),
            (&repo, "/etc/passwd", false),
            (&repo, "a\\b.py", true),
            (&module, "services/api/src/A.java", true),
            (&module, "services/api", true),
            (&module, "services/apis/A.java", false),
            (&module, "services/api/target/x.class", false),
            (&module, "services/other/A.java", false),
            (&module, "services", false),
            (&module, "a.py", false),
            (&svc, "svc/a.go", true),
            (&svc, "svcx/a.go", false),
        ];
        for (sc, path, want) in table {
            assert_eq!(sc.includes(path), *want, "{:?} {path}", sc.path);
        }
        assert!(module.may_contain("services"));
        assert!(module.may_contain("services/api/src"));
        assert!(!module.may_contain("services/other"));
        assert!(!module.may_contain("services/api/target"));
        assert!(repo.may_contain("src"));
        assert!(!repo.may_contain("vendor"));
    }

    #[test]
    fn candidates_in_vendored_or_generated_code_are_left_out_of_the_static_stages() {
        let (d, ctx) = run(None);
        for (f, text) in [
            ("vendor/x/a.py", "A = {}\n"),
            ("app/gen.c", "/* Generated by Cython 3.0 */\n"),
            ("app/own.py", "B = {}\n"),
        ] {
            let p = d.path().join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        let mut l = ctx.ledger().unwrap();
        let ev = ctx
            .add_evidence(
                &mut l,
                EvidenceSpec::new(EvidenceKind::StaticHit, "ruff", "0.1", json!({"hits": 3})),
            )
            .unwrap();
        let cand = |site: &str| Candidate {
            origin: "detector:ruff/B019".into(),
            evidence: vec![ev.clone()],
            site: site.into(),
            ..Candidate::default()
        };
        let ledger = ctx.ledger().unwrap();
        let mut w = CandidateWriter::for_static(&ctx, &ledger).unwrap();
        for site in ["vendor/x/a.py:1", "app/gen.c:1", "app/own.py:1"] {
            w.add(&mut l, cand(site)).unwrap();
        }
        assert_eq!((w.added(), w.left_out(), w.out_of_scope()), (1, 2, 0));
        // The dynamic stages keep every site: a leak the workload triggers is still measured.
        let mut dynamic = CandidateWriter::for_ctx(&ctx, &ledger).unwrap();
        for site in ["vendor/x/a.py:2", "app/gen.c:2"] {
            dynamic.add(&mut l, cand(site)).unwrap();
        }
        assert_eq!((dynamic.added(), dynamic.left_out()), (2, 0));
        assert!(ctx.in_scope("vendor/x/a.py"));
    }

    #[test]
    fn candidates_outside_the_scope_are_counted_not_added() {
        let (_d, ctx) = run(None);
        RunState::update(&ctx.run_dir, |s| {
            s.scope = Some(scope(crate::state::ScopeKind::Module, "svc", &["svc/gen"]));
            Ok(())
        })
        .unwrap();
        assert!(ctx.in_scope("svc/a.py"));
        assert!(!ctx.in_scope("other/a.py"));
        let mut ledger = ctx.ledger().unwrap();
        let ev = ctx
            .add_evidence(
                &mut ledger,
                EvidenceSpec::new(EvidenceKind::StaticHit, "ruff", "0.1", json!({"hits": 2})),
            )
            .unwrap();
        let mut w = CandidateWriter::for_ctx(&ctx, &ledger).unwrap();
        let cand = |site: &str| Candidate {
            origin: "detector:ruff/B019".into(),
            evidence: vec![ev.clone()],
            site: site.into(),
            ..Candidate::default()
        };
        assert!(w.add(&mut ledger, cand("other/a.py:3")).unwrap().is_none());
        assert!(
            w.add(&mut ledger, cand("svc/gen/a.py:3"))
                .unwrap()
                .is_none()
        );
        assert!(w.add(&mut ledger, cand("svc/a.py:3")).unwrap().is_some());
        assert_eq!((w.added(), w.skipped(), w.out_of_scope()), (1, 0, 2));
        // An unscoped writer ignores the scope.
        let mut plain = CandidateWriter::new(&ledger);
        assert!(
            plain
                .add(&mut ledger, cand("other/a.py:3"))
                .unwrap()
                .is_some()
        );
        ctx.add_out_of_scope("python", 2).unwrap();
        ctx.add_out_of_scope("python", 3).unwrap();
        let cov = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
        assert_eq!(cov.workers["python"].out_of_scope, 5);
    }

    fn static_cand(origin: &str, ev: &EvidenceId, site: &str) -> Candidate {
        Candidate {
            origin: origin.into(),
            evidence: vec![ev.clone()],
            site: site.into(),
            ..Candidate::default()
        }
    }

    fn evidence_of(ctx: &ToolCtx, ledger: &mut Ledger, kind: EvidenceKind) -> EvidenceId {
        ctx.add_evidence(
            ledger,
            EvidenceSpec::new(kind, "t", "0", json!({"hits": 1})),
        )
        .unwrap()
    }

    #[test]
    fn a_rule_hit_and_a_census_entry_at_one_site_are_one_finding_in_either_order() {
        for rule_first in [true, false] {
            let (_d, ctx) = run(None);
            let mut ledger = ctx.ledger().unwrap();
            let e_rule = evidence_of(&ctx, &mut ledger, EvidenceKind::StaticHit);
            let e_census = evidence_of(&ctx, &mut ledger, EvidenceKind::CensusHolder);
            let rule = Candidate {
                suggested_archetype: Some("L-COLLECT".into()),
                note: Some("lru_cache(None) is unbounded".into()),
                ..static_cand(
                    "detector:dla/python-unbounded-cache",
                    &e_rule,
                    "pkg/app.py:82",
                )
            };
            let census = Candidate {
                holder: Some(Holder::StaticGlobal),
                owner: Some("pkg.app._build".into()),
                region: Some(Region::Heap),
                ..static_cand("census", &e_census, "pkg/app.py:82")
            };
            let mut w = CandidateWriter::new(&ledger);
            let (first, second) = if rule_first {
                (rule, census)
            } else {
                (census, rule)
            };
            let Added::New(id) = w.add_outcome(&mut ledger, first).unwrap() else {
                panic!("the first candidate is a new finding");
            };
            assert_eq!(
                w.add_outcome(&mut ledger, second).unwrap(),
                Added::Joined(id.clone())
            );
            assert_eq!((w.added(), w.joined(), w.skipped()), (1, 1, 0));
            // Replay from the file: one finding, both origins, both evidence, the census data.
            let ledger = ctx.ledger().unwrap();
            assert_eq!(ledger.findings().count(), 1, "rule_first={rule_first}");
            let f = ledger.finding(&id).unwrap();
            assert_eq!(f.supports.len(), 2);
            assert_eq!(f.also_origins.len(), 1);
            let (main, other) = if rule_first {
                ("detector:dla/python-unbounded-cache", "census")
            } else {
                ("census", "detector:dla/python-unbounded-cache")
            };
            assert_eq!(f.origin.as_str(), main);
            assert_eq!(f.also_origins[0].as_str(), other);
            assert_eq!(f.owner.as_deref(), Some("pkg.app._build"));
            assert_eq!(f.holder, Some(Holder::StaticGlobal));
            assert_eq!(f.region, Some(Region::Heap));
            let notes = &ledger.annotation(&id).unwrap().notes;
            assert!(
                notes
                    .iter()
                    .any(|n| n.contains(&format!("also raised by {other}"))),
                "{notes:?}"
            );
            if rule_first {
                assert!(notes.iter().any(|n| n.contains("census")), "{notes:?}");
            } else {
                assert!(
                    notes
                        .iter()
                        .any(|n| n.contains("lru_cache(None) is unbounded")
                            && n.contains("suggests L-COLLECT")),
                    "the rule's message and archetype are kept: {notes:?}"
                );
            }
        }
    }

    #[test]
    fn a_second_run_of_a_stage_does_not_join_again() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let e = evidence_of(&ctx, &mut ledger, EvidenceKind::StaticHit);
        let mut w = CandidateWriter::new(&ledger);
        w.add(&mut ledger, static_cand("detector:ruff/B019", &e, "a.py:3"))
            .unwrap();
        w.add(&mut ledger, static_cand("census", &e, "a.py:3"))
            .unwrap();
        let lines = ledger.lines().len();
        let mut again = ctx.ledger().unwrap();
        let mut w2 = CandidateWriter::new(&again);
        assert_eq!(
            w2.add_outcome(&mut again, static_cand("census", &e, "a.py:3"))
                .unwrap(),
            Added::Skipped
        );
        assert_eq!(again.lines().len(), lines, "nothing was written");
        assert_eq!(again.findings().count(), 1);
    }

    #[test]
    fn lines_of_one_declaration_are_one_site_and_other_lines_are_not() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let e = evidence_of(&ctx, &mut ledger, EvidenceKind::StaticHit);
        // Lines 3 to 5 are one decorated function; line 9 is a function of its own.
        let anchor = SiteAnchor::new(|_, line| if (3..=5).contains(&line) { 5 } else { line });
        let mut w = CandidateWriter::new(&ledger).with_anchor(anchor);
        let a = w
            .add_outcome(&mut ledger, static_cand("detector:dla/r", &e, "a.py:3"))
            .unwrap();
        let Added::New(id) = a else { panic!("{a:?}") };
        assert_eq!(
            w.add_outcome(&mut ledger, static_cand("census", &e, "a.py:5"))
                .unwrap(),
            Added::Joined(id.clone())
        );
        assert!(matches!(
            w.add_outcome(&mut ledger, static_cand("census", &e, "a.py:9"))
                .unwrap(),
            Added::New(_)
        ));
        assert!(matches!(
            w.add_outcome(&mut ledger, static_cand("census", &e, "b.py:5"))
                .unwrap(),
            Added::New(_)
        ));
        assert_eq!(ledger.findings().count(), 3);
    }

    #[test]
    fn a_growth_series_finding_is_never_joined_to_a_static_site() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let e = evidence_of(&ctx, &mut ledger, EvidenceKind::StaticHit);
        let mut w = CandidateWriter::new(&ledger);
        w.add(&mut ledger, static_cand("census", &e, "a.py:3"))
            .unwrap();
        let out = w
            .add_outcome(&mut ledger, static_cand("correlation", &e, "a.py:3"))
            .unwrap();
        assert!(matches!(out, Added::New(_)), "{out:?}");
        assert_eq!(ledger.findings().count(), 2);
    }

    #[test]
    fn two_rules_at_one_site_stay_two_findings_and_a_census_entry_joins_the_first() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let e = evidence_of(&ctx, &mut ledger, EvidenceKind::StaticHit);
        let mut w = CandidateWriter::new(&ledger);
        let a = w
            .add_outcome(&mut ledger, static_cand("detector:ruff/B019", &e, "a.py:3"))
            .unwrap();
        let b = w
            .add_outcome(
                &mut ledger,
                static_cand("detector:pylint/W1518", &e, "a.py:3"),
            )
            .unwrap();
        assert!(
            matches!((&a, &b), (Added::New(_), Added::New(_))),
            "{a:?} {b:?}"
        );
        let Added::New(first) = a else { unreachable!() };
        assert_eq!(
            w.add_outcome(&mut ledger, static_cand("census", &e, "a.py:3"))
                .unwrap(),
            Added::Joined(first)
        );
        // A later hit of a third analyzer joins the finding that carries the census entry.
        let c = w
            .add_outcome(&mut ledger, static_cand("detector:semgrep/x", &e, "a.py:3"))
            .unwrap();
        assert!(matches!(c, Added::Joined(_)), "{c:?}");
        assert_eq!(ledger.findings().count(), 2);
    }

    #[test]
    fn a_worker_never_records_on_a_finding_of_the_core() {
        let (_d, mut ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let e = evidence_of(&ctx, &mut ledger, EvidenceKind::StaticHit);
        let mut w = CandidateWriter::new(&ledger);
        w.add(&mut ledger, static_cand("detector:dla/r", &e, "a.py:3"))
            .unwrap();
        // The core's finding is read-only to a worker: its census entry is its own finding.
        ctx.worker = Some("python".into());
        let mut slice = ctx.ledger().unwrap();
        let e2 = evidence_of(&ctx, &mut slice, EvidenceKind::CensusHolder);
        let mut w = CandidateWriter::new(&slice);
        let out = w
            .add_outcome(&mut slice, static_cand("census", &e2, "a.py:3"))
            .unwrap();
        assert!(matches!(out, Added::New(_)), "{out:?}");
        // Its own rule hit and census entry join as in the core.
        let out = w
            .add_outcome(&mut slice, static_cand("detector:dla/other", &e2, "a.py:3"))
            .unwrap();
        assert!(matches!(out, Added::Joined(_)), "{out:?}");
    }

    #[test]
    fn a_different_owner_at_a_joined_site_is_said_in_the_note_not_refused() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let e = evidence_of(&ctx, &mut ledger, EvidenceKind::StaticHit);
        let mut w = CandidateWriter::new(&ledger);
        let first = Candidate {
            owner: Some("m.A.x".into()),
            ..static_cand("detector:dla/r", &e, "a.py:3")
        };
        let second = Candidate {
            owner: Some("m.B.y".into()),
            ..static_cand("census", &e, "a.py:3")
        };
        let Added::New(id) = w.add_outcome(&mut ledger, first).unwrap() else {
            panic!()
        };
        w.add_outcome(&mut ledger, second).unwrap();
        let f = ledger.finding(&id).unwrap();
        assert_eq!(f.owner.as_deref(), Some("m.A.x"));
        let notes = &ledger.annotation(&id).unwrap().notes;
        assert!(notes.iter().any(|n| n.contains("owner m.B.y")), "{notes:?}");
    }

    #[test]
    fn a_build_logic_site_is_never_a_candidate_and_is_counted() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let ev = ctx
            .add_evidence(
                &mut ledger,
                EvidenceSpec::new(EvidenceKind::StaticHit, "detekt", "2", json!({"hits": 4})),
            )
            .unwrap();
        let cand = |site: &str| Candidate {
            origin: "detector:detekt/GlobalCoroutineUsage".into(),
            evidence: vec![ev.clone()],
            site: site.into(),
            ..Candidate::default()
        };
        let mut w = CandidateWriter::for_ctx(&ctx, &ledger).unwrap();
        for site in [
            "docker.gradle.kts:4",
            "build.gradle.kts:3",
            "svc/buildSrc/src/main/kotlin/Conventions.kt:2",
        ] {
            assert!(w.add(&mut ledger, cand(site)).unwrap().is_none(), "{site}");
        }
        assert!(
            w.add(&mut ledger, cand("src/main/kotlin/App.kt:9"))
                .unwrap()
                .is_some()
        );
        assert!(
            w.add(&mut ledger, cand("scripts/tool.main.kts:1"))
                .unwrap()
                .is_some(),
            "a .main.kts script is a source"
        );
        assert_eq!((w.added(), w.build_logic(), w.out_of_scope()), (2, 3, 0));
    }

    #[test]
    fn require_time_names_the_deadline_and_the_fix() {
        let (_d, mut ctx) = run(None);
        ctx.require_time("sampling").unwrap();
        let past = now_utc() - chrono::TimeDelta::try_seconds(60).unwrap();
        // A run whose deadline passed.
        RunState::update(&ctx.run_dir, |s| {
            s.budget.deadline = past;
            Ok(())
        })
        .unwrap();
        let e = ctx.require_time("sampling").unwrap_err();
        assert_eq!(e.code, codes::TIME_BUDGET);
        assert!(e.message.contains("sampling") && e.message.contains("run deadline"));
        assert!(
            e.message
                .contains(&past.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        );
        assert_eq!(e.fix, super::DEADLINE_FIX);
        assert!(e.fix.starts_with("dla run extend 30m"), "{}", e.fix);
        assert!(e.fix.contains("--reason time_budget"), "{}", e.fix);
        // A worker past its own deadline while the run still has time.
        let future = now_utc() + chrono::TimeDelta::try_seconds(600).unwrap();
        RunState::update(&ctx.run_dir, |s| {
            s.budget.deadline = future;
            s.workers.push(crate::state::WorkerEntry {
                name: "jvm".into(),
                runtime: "jvm".into(),
                share_secs: 1,
                deadline: past,
                started_at: None,
                status: crate::state::WorkerStatus::Registered,
                slice: "ledger.jvm.jsonl".into(),
                stalled: false,
            });
            Ok(())
        })
        .unwrap();
        ctx.require_time("sampling").unwrap();
        ctx.worker = Some("jvm".into());
        let e = ctx.require_time("alloc-test").unwrap_err();
        assert!(e.message.contains("worker jvm deadline"), "{}", e.message);
        // An unregistered worker only has the run's deadline.
        ctx.worker = Some("go".into());
        ctx.require_time("alloc-test").unwrap();
    }

    #[test]
    fn every_time_budget_refusal_of_a_tool_is_noted_for_the_reason_check() {
        let before = now_utc() - chrono::TimeDelta::try_seconds(1).unwrap();
        // Nothing refused yet.
        let (_d, mut ctx) = run(None);
        assert!(!time_refused_since(&ctx.run_dir, None, before));
        // A command refused for lack of time notes it, whatever the deadline.
        let _ = ctx.time_budget_refusal("sample", 10, 5, "run deadline", "a smaller run");
        assert!(time_refused_since(&ctx.run_dir, None, before));
        // A suggestion of a smaller run means something smaller fits.
        assert!(!nothing_fits_since(&ctx.run_dir, None, before));
        let _ = ctx.time_budget_refusal("sample", 10, 5, "run deadline", "");
        assert!(nothing_fits_since(&ctx.run_dir, None, before));
        // Each side sees only its own: the core never a worker's, a worker never the core's.
        assert!(!time_refused_since(&ctx.run_dir, Some("jvm"), before));
        ctx.worker = Some("jvm".into());
        let _ = ctx.time_budget_refusal("sample", 10, 5, "worker deadline", "");
        assert!(time_refused_since(&ctx.run_dir, Some("jvm"), before));
        assert!(!time_refused_since(&ctx.run_dir, Some("go"), before));
        // A note older than the finding does not count (this run holds core lines).
        let later = now_utc() + chrono::TimeDelta::try_seconds(60).unwrap();
        assert!(!time_refused_since(&ctx.run_dir, None, later));
        assert!(!time_refused_since(&ctx.run_dir, Some("jvm"), later));
        // A run holding only a worker's line: the core sees nothing.
        let (_d, mut ctx) = run(None);
        ctx.worker = Some("jvm".into());
        let _ = ctx.time_budget_refusal("sample", 10, 5, "worker deadline", "");
        assert!(!time_refused_since(&ctx.run_dir, None, before));
        assert!(!nothing_fits_since(&ctx.run_dir, None, before));

        // `require_time` on a spent deadline is a refusal too.
        let (_d, ctx) = run(None);
        RunState::update(&ctx.run_dir, |s| {
            s.budget.deadline = now_utc() - chrono::TimeDelta::try_seconds(60).unwrap();
            Ok(())
        })
        .unwrap();
        assert!(!time_refused_since(&ctx.run_dir, None, before));
        ctx.require_time("sampling").unwrap_err();
        assert!(time_refused_since(&ctx.run_dir, None, before));
        assert!(nothing_fits_since(&ctx.run_dir, None, before));
        // A passing check notes nothing.
        let (_d, ctx) = run(None);
        ctx.require_time("sampling").unwrap();
        assert!(!time_refused_since(&ctx.run_dir, None, before));
    }

    #[test]
    fn a_refusal_before_an_extension_no_longer_counts() {
        let before = now_utc() - chrono::TimeDelta::try_seconds(1).unwrap();
        let (_d, ctx) = run(None);
        let _ = ctx.time_budget_refusal("sample", 10, 5, "run deadline", "");
        assert!(time_refused_since(&ctx.run_dir, None, before));
        // The extension gives the time back: the earlier refusal is spent.
        let later = now_utc() + chrono::TimeDelta::try_seconds(2).unwrap();
        RunState::update(&ctx.run_dir, |s| s.extend_budget(later, 7200).map(|_| ())).unwrap();
        assert!(!time_refused_since(&ctx.run_dir, None, before));
        assert!(!time_refused_since(&ctx.run_dir, Some("jvm"), before));
    }

    #[test]
    fn a_spent_deadline_offers_extending_only_to_the_core_of_an_interactive_run() {
        let past = now_utc() - chrono::TimeDelta::try_seconds(60).unwrap();
        // Interactive core (no consent object): extend, after asking the user.
        let (_d, mut ctx) = run(None);
        RunState::update(&ctx.run_dir, |s| {
            s.budget.deadline = past;
            Ok(())
        })
        .unwrap();
        let e = ctx.require_time("sampling").unwrap_err();
        assert!(e.fix.starts_with("dla run extend"), "{}", e.fix);
        // A worker cannot extend (and the core waits on it): close what cannot be measured.
        ctx.worker = Some("jvm".into());
        let e = ctx.require_time("sampling").unwrap_err();
        assert_eq!(e.fix, super::DEADLINE_FIX_CLOSE);
        assert!(!e.fix.contains("dla run extend"), "{}", e.fix);
        let e = ctx.time_budget_refusal("sample", 10, 5, "run deadline", "");
        assert!(!e.fix.contains("dla run extend"), "{}", e.fix);
        // An interactive core after `dla consent` (a consent object given by the user): the
        // user can still be asked, so it extends.
        let (_d, ctx) = run(Some(vec![1, 2]));
        RunState::update(&ctx.run_dir, |s| {
            s.budget.deadline = past;
            s.consent.as_mut().unwrap().given_by = ConsentGiver::User;
            Ok(())
        })
        .unwrap();
        let e = ctx.require_time("sampling").unwrap_err();
        assert_eq!(e.fix, super::DEADLINE_FIX);
        // An unattended core (consent given by the runner): nobody to ask, so it never extends.
        let (_d, ctx) = run(Some(vec![1, 2]));
        RunState::update(&ctx.run_dir, |s| {
            s.budget.deadline = past;
            Ok(())
        })
        .unwrap();
        let e = ctx.require_time("sampling").unwrap_err();
        assert_eq!(e.fix, super::DEADLINE_FIX_CLOSE);
        let e = ctx.time_budget_refusal("sample", 10, 5, "run deadline", "");
        assert!(!e.fix.contains("dla run extend"), "{}", e.fix);
    }

    #[test]
    fn registered_processes_carry_the_worker() {
        let (_d, mut ctx) = run(None);
        ctx.worker = Some("rust".into());
        let rec = |worker: Option<&str>| ProcessRecord {
            pid: 7,
            start_time: None,
            kind: crate::state::ProcessKind::Harness,
            command: "x".into(),
            started_at: now_utc(),
            stopped_at: None,
            role: crate::state::ProcessRole::Started,
            worker: worker.map(str::to_owned),
            cleanup: Vec::new(),
        };
        ctx.register_process(rec(None)).unwrap();
        ctx.register_process(rec(Some("jvm"))).unwrap();
        let st = RunState::load(&ctx.run_dir).unwrap();
        assert_eq!(st.processes[0].worker.as_deref(), Some("rust"));
        assert_eq!(st.processes[1].worker.as_deref(), Some("jvm"));
    }
}
