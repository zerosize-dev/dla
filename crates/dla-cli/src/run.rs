//! `dla run init`, `dla run status`, `dla run dispatch` and `dla run abort`.

mod abort;
mod reseal;
mod workers;

use clap::{Args, Subcommand};
use dla_core::{
    FindingState, Ledger, MAX_OUTPUT_BYTES, MAX_RESULT_BYTES, Record, RunState, RunView, consent,
    error::codes as core_codes,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool,
    },
    state::{self, EntryPath, STATE_FILE, Scope, ScopeKind},
    status,
};
use serde_json::{Value, json};

use crate::{
    budget, consent_cmd,
    ctx::{Ctx, project_root},
    fail::{Failure, Res},
    git, gitignore,
    parse::parse_enum,
};

/// `dla run ...`
#[derive(Subcommand, Debug)]
pub enum RunCmd {
    /// Create a run directory, its state and an empty ledger.
    Init(InitArgs),
    /// Show run id, stage, status, entry path, workers, running processes, counts and the first blocking items.
    Status,
    /// Register one worker per runtime with its share of the remaining budget (core only).
    Dispatch(workers::DispatchArgs),
    /// Stop the processes the run started, undo what it attached, and mark the run aborted (core only).
    Abort(abort::AbortArgs),
    /// Record an audit line and seal the protected run files again after an out-of-band change (core only).
    Reseal(reseal::ResealArgs),
}

/// Flags of `dla run init`.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Time budget: 90s, 45m or 2h (default: DLA_DEFAULT_BUDGET, else 60m).
    #[arg(long)]
    budget: Option<String>,
    /// Session ID (default: CLAUDE_SESSION_ID, then CLAUDE_CODE_SESSION_ID, else "unknown").
    #[arg(long)]
    session_id: Option<String>,
    /// Scope kind: repository, module or service.
    #[arg(long, value_parser = parse_enum::<ScopeKind>, requires = "scope_path")]
    scope: Option<ScopeKind>,
    /// Scope path (with --scope).
    #[arg(long, requires = "scope")]
    scope_path: Option<String>,
    /// Excluded path, repeatable (with --scope).
    #[arg(long, requires = "scope")]
    exclude: Vec<String>,
    /// Entry path: repository (default), process, artifact or symptom.
    #[arg(long, value_parser = parse_enum::<EntryPath>)]
    entry: Option<EntryPath>,
    /// Artifact file or directory relative to the project root, repeatable (entry artifact only, required there).
    #[arg(long)]
    artifact: Vec<String>,
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// Longest accepted session ID, in characters.
const MAX_SESSION_ID_CHARS: usize = 256;

/// Everything `run init` can refuse is checked here, before anything is written.
fn run_config_result(
    root_text: &str,
    session: &str,
    seconds: u64,
    scope: &Option<Scope>,
    entry: EntryPath,
    artifacts: &[String],
) -> Res<Value> {
    if session.chars().count() > MAX_SESSION_ID_CHARS {
        return Err(Failure::refuse(
            core_codes::INVALID_VALUE,
            format!(
                "the session id is {} characters; the limit is {MAX_SESSION_ID_CHARS}",
                session.chars().count()
            ),
            format!("a session id of at most {MAX_SESSION_ID_CHARS} characters"),
            "dla run init --session-id <short id>   (or unset CLAUDE_SESSION_ID / CLAUDE_CODE_SESSION_ID)",
        ));
    }
    let result = json!({
        "project_root": root_text,
        "session_id": session,
        "budget_seconds": seconds,
        "scope": scope,
        "entry": entry.as_str(),
        "artifacts": artifacts,
    });
    let size = serde_json::to_string(&result).map_or(usize::MAX, |t| t.len());
    if size > MAX_RESULT_BYTES {
        return Err(Failure::refuse(
            core_codes::RESULT_TOO_LARGE,
            format!(
                "the run configuration is {size} bytes, over the {MAX_RESULT_BYTES}-byte bound for an evidence record"
            ),
            "a shorter --scope-path, fewer or shorter --exclude or --artifact entries, or a shorter project path",
            "dla run init --scope module --scope-path <short path> --exclude <short path>",
        ));
    }
    Ok(result)
}

/// Checks `--entry` against `--artifact` and returns the artifact paths as stored:
/// relative to the project root, `/`-separated, existing. Artifact runs need at least
/// one; the other entry paths take none.
fn artifact_paths(
    project: &std::path::Path,
    entry: EntryPath,
    given: &[String],
) -> Res<Vec<String>> {
    if entry != EntryPath::Artifact {
        if let Some(first) = given.first() {
            return Err(Failure::refuse(
                core_codes::INVALID_VALUE,
                format!("--artifact {first} is only allowed with --entry artifact"),
                "--entry artifact, or no --artifact",
                format!("dla run init --entry artifact --artifact {first}"),
            ));
        }
        return Ok(Vec::new());
    }
    if given.is_empty() {
        return Err(Failure::refuse(
            core_codes::INVALID_VALUE,
            "an artifact run needs the artifacts to examine",
            "--artifact <path relative to the project root> (repeatable)",
            "dla run init --entry artifact --artifact dumps/heap1.hprof.gz --artifact dumps/heap2.hprof.gz",
        ));
    }
    let mut out: Vec<String> = Vec::new();
    for g in given {
        let path = std::path::Path::new(g);
        let rel = if path.is_absolute() {
            dla_core::paths::rel_slash(project, path)
        } else {
            Some(dla_core::paths::slash(g))
        }
        .map(|r| r.trim_start_matches("./").to_owned());
        let rel = rel
            .filter(|r| !r.is_empty() && !r.split('/').any(|c| c == "..") && !r.starts_with('/'))
            .ok_or_else(|| {
                Failure::refuse(
                    core_codes::INVALID_VALUE,
                    format!("artifact {g:?} is not a path inside the project"),
                    "an artifact path relative to the project root, without ..",
                    "dla run init --entry artifact --artifact <relative path>",
                )
            })?;
        if !project.join(&rel).exists() {
            return Err(Failure::refuse(
                core_codes::INVALID_VALUE,
                format!(
                    "artifact {rel} does not exist under {}",
                    dla_core::paths::to_slash(project)
                ),
                format!("the file {rel}"),
                format!(
                    "dla run init --entry artifact --artifact {rel}   (after copying it into the project)"
                ),
            ));
        }
        if !out.contains(&rel) {
            out.push(rel);
        }
    }
    Ok(out)
}

/// Refuses a scope that would not mean what the user typed: an absolute, `..` or
/// empty path, a glob exclude, or a module/service path that is not a directory
/// under the project. Nothing is silently widened to the whole repository.
fn check_scope(sc: &Scope) -> Res<()> {
    sc.validate().map_err(|why| {
        Failure::refuse(
            core_codes::INVALID_VALUE,
            why,
            "--scope-path <path relative to the project root> and plain relative --exclude paths",
            "dla run init --scope module --scope-path <path relative to the project root>",
        )
    })
}

/// A module or service scope must name a directory of the project.
fn check_scope_dir(project: &std::path::Path, sc: &Scope) -> Res<()> {
    if sc.kind != ScopeKind::Repository {
        let rel = dla_core::paths::slash(&sc.path);
        if !project.join(rel.trim_start_matches("./")).is_dir() {
            return Err(Failure::refuse(
                core_codes::INVALID_VALUE,
                format!(
                    "scope path {} is not a directory under {}",
                    sc.path,
                    dla_core::paths::to_slash(project)
                ),
                "an existing directory of the project as --scope-path",
                "dla run init --scope module --scope-path <path relative to the project root>",
            ));
        }
    }
    Ok(())
}

/// Runs one `dla run ...` subcommand. `ctx` resolves the run (not needed by `init`).
pub fn dispatch(
    project_arg: Option<&std::path::Path>,
    ctx: impl Fn() -> Res<Ctx>,
    cmd: RunCmd,
) -> Res<Value> {
    match cmd {
        RunCmd::Init(a) => init(project_arg, a),
        RunCmd::Status => run_status(&ctx()?),
        RunCmd::Dispatch(a) => workers::dispatch(&ctx()?, a),
        RunCmd::Abort(a) => abort::run(&ctx()?, &project_root(project_arg)?, a),
        RunCmd::Reseal(a) => reseal::run(&ctx()?, a),
    }
}

/// Runs `dla run init`. Nothing is created unless every input is valid, and a run
/// directory created by a failing init is removed again.
pub fn init(project_arg: Option<&std::path::Path>, a: InitArgs) -> Res<Value> {
    let project = project_root(project_arg)?;
    let seconds = match a.budget.or_else(|| env_nonempty("DLA_DEFAULT_BUDGET")) {
        Some(b) => budget::parse(&b)?,
        None => budget::DEFAULT_SECONDS,
    };
    let session = a
        .session_id
        .or_else(|| env_nonempty("CLAUDE_SESSION_ID"))
        .or_else(|| env_nonempty("CLAUDE_CODE_SESSION_ID"));
    let scope = match (a.scope, a.scope_path) {
        (Some(kind), Some(path)) => Some(Scope {
            kind,
            path,
            excluded: a.exclude,
        }),
        _ => None,
    };
    if let Some(sc) = &scope {
        check_scope(sc)?;
    }
    let entry = a.entry.unwrap_or(EntryPath::Repository);
    let artifacts = artifact_paths(&project, entry, &a.artifact)?;
    let headless = headless_consent()?;
    let root_text = dla_core::paths::to_slash(&project);
    let session_text = session
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown")
        .to_owned();
    let result = run_config_result(
        &root_text,
        &session_text,
        seconds,
        &scope,
        entry,
        &artifacts,
    )?;
    if let Some(sc) = &scope {
        check_scope_dir(&project, sc)?;
    }
    let mut st = RunState::init(
        &root_text,
        session.as_deref(),
        seconds,
        scope.clone(),
        state::now_utc(),
    )?;
    st.consent = headless;
    st.entry = entry;
    st.artifacts = artifacts;
    let ctx = Ctx {
        run_dir: state::run_dir(&project, &st.run_id),
        run_id: st.run_id.clone(),
        worker: None,
    };
    // The ignore entry goes in first so a run's salt can never be staged by accident.
    let ignored = gitignore::ensure_dla_ignored(&project)?;
    let dla_dir = project.join(state::DLA_DIR);
    let dla_dir_existed = dla_dir.exists();
    std::fs::create_dir_all(&dla_dir).map_err(|e| dla_core::Error::io(&dla_dir, e))?;
    // `create_dir`, not `create_dir_all`: an existing directory is never adopted, so
    // the cleanup below can only ever remove a directory this call created.
    std::fs::create_dir(&ctx.run_dir).map_err(|e| dla_core::Error::io(&ctx.run_dir, e))?;
    let built = (|| -> Res<u64> {
        st.save(&ctx.run_dir)?;
        let mut ledger = Ledger::create(&ctx.run_dir)?;
        let commit = git::head_commit(&project).unwrap_or_else(|| root_text.clone());
        let line = ledger.add_evidence(NewEvidence {
            source: Source::Repo { commit, path: None },
            tool: Tool {
                name: "dla".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            authority: Authority::Static,
            kind: EvidenceKind::RunConfig,
            result,
            raw: None,
            coverage: EvidenceCoverage {
                status: CoverageStatus::Complete,
                examined: None,
                total: None,
            },
            limitations: Vec::new(),
            unknowns: Vec::new(),
        })?;
        if let Some(c) = &st.consent {
            consent_cmd::append_evidence(&mut ledger, &project, c)?;
        }
        state::write_current(&project, &st.run_id)?;
        Ok(line.seq)
    })();
    let seq = match built {
        Ok(seq) => seq,
        Err(f) => {
            let _ = std::fs::remove_dir_all(&ctx.run_dir);
            if !dla_dir_existed {
                let _ = std::fs::remove_dir(&dla_dir);
            }
            return Err(f);
        }
    };
    let mut out = json!({
        "ok": true,
        "run_id": st.run_id,
        "stage": st.stage.as_str(),
        "status": st.status.as_str(),
        "session_id": st.session_id,
        "budget_seconds": seconds,
        "entry": st.entry.as_str(),
        "artifacts": st.artifacts,
        "evidence": "E1",
        "gitignore": ignored.as_str(),
        "record": ctx.pointer(STATE_FILE),
        "ledger": ctx.ledger_line(seq),
    });
    if let Some(c) = &st.consent {
        out["consent"] = json!({
            "tiers": c.tiers,
            "raw_values": c.raw_values,
            "given_by": c.given_by.as_str(),
            "evidence": "E2",
        });
    }
    Ok(out)
}

/// Consent given by the runner through `DLA_CONSENT_TIERS` (and `DLA_CONSENT_RAW_VALUES`),
/// so an unattended run needs no model action. `None` when the tiers are not set.
fn headless_consent() -> Res<Option<state::Consent>> {
    let Some(tiers) = env_nonempty("DLA_CONSENT_TIERS") else {
        return Ok(None);
    };
    let tiers = consent::parse_tiers(&tiers).map_err(|mut r| {
        r.message = format!("DLA_CONSENT_TIERS: {}", r.message);
        r.fix = "set DLA_CONSENT_TIERS to 1-4 (or a list such as 1,2,3), or unset it".to_owned();
        Failure::from(r)
    })?;
    let raw = env_nonempty("DLA_CONSENT_RAW_VALUES").is_some_and(|v| consent::flag_on(&v));
    let c = consent::build(
        tiers,
        raw,
        state::ConsentGiver::Runner,
        None,
        state::now_utc(),
    )
    .map_err(Failure::from)?;
    Ok(Some(c))
}

/// Runs `dla run status`.
pub fn run_status(ctx: &Ctx) -> Res<Value> {
    let (view, rs) = match ctx.worker.as_deref() {
        Some(w) => worker_status(ctx, w)?,
        None => {
            let view = RunView::load(&ctx.run_dir)?;
            let rs = status(&view);
            (view, rs)
        }
    };
    let total = rs.blocking.len();
    let shown: Vec<_> = rs.blocking.iter().take(3).collect();
    let revisit = revisit_entries(&view, ctx.worker.as_deref());
    let revisit_total = revisit.len();
    let mut out = json!({
        "ok": true,
        "run_id": view.state.run_id,
        "stage": view.state.stage.as_str(),
        "status": view.state.status.as_str(),
        "complete": rs.complete,
        "counts": rs.counts,
        "blocking_total": total,
        "blocking": shown,
        "record": ctx.pointer(STATE_FILE),
        "ledger": ctx.pointer(dla_core::state::LEDGER_FILE),
        "report": view.state.report.as_ref().map(|r| r.path.clone()),
    });
    if let (Value::Object(o), Value::Object(extra)) =
        (&mut out, workers::status_parts(ctx, &view.state))
    {
        o.extend(extra);
    }
    if let Some(w) = &ctx.worker {
        out["worker"] = json!(w);
    } else {
        // Core mode: slices whose lines the main ledger has not merged, and slices
        // that end in a torn fragment (a worker killed mid-write).
        let unmerged: Vec<Value> = view
            .unmerged_slices()
            .into_iter()
            .map(|s| {
                json!({
                    "worker": s.worker,
                    "lines": s.lines,
                    "merged": view.ledger.merged_lines(&s.worker),
                    "done": s.done,
                    "torn_tail": s.torn_tail,
                })
            })
            .collect();
        let torn: Vec<&str> = view
            .slices
            .iter()
            .filter(|s| s.torn_tail)
            .map(|s| s.worker.as_str())
            .collect();
        out["unmerged_slices"] = Value::Array(unmerged);
        out["torn_tails"] = json!(torn);
    }
    out["revisit"] = Value::Array(Vec::new());
    out["revisit_total"] = json!(revisit_total);
    let mut truncated = total > shown.len();
    // Add revisit entries one by one while the whole object stays inside the output bound.
    let room = MAX_OUTPUT_BYTES - 1 - 40;
    for (n, entry) in revisit.into_iter().enumerate() {
        let mut trial = out.clone();
        if let Some(Value::Array(a)) = trial.get_mut("revisit") {
            a.push(entry);
        }
        if n >= MAX_REVISIT || serde_json::to_string(&trial).map_or(usize::MAX, |t| t.len()) > room
        {
            truncated = true;
            break;
        }
        out = trial;
    }
    if truncated {
        out["truncated"] = Value::Bool(true);
    }
    Ok(out)
}

/// The status view of one worker: its slice replayed over the main ledger, with only
/// what the worker itself must act on in `blocking`: its own findings and open items
/// (IDs carry `<worker>/`) and the run-level items (an aborted run, a spent time
/// budget). The checklist and other workers' items belong to the core.
fn worker_status(ctx: &Ctx, w: &str) -> Res<(RunView, dla_core::view::ReportStatus)> {
    use dla_core::view::{BlockingItem, BlockingKind};
    let state = RunState::load(&ctx.run_dir)?;
    let ledger = Ledger::load_for(&ctx.run_dir, Some(w))?;
    let coverage = dla_core::CoverageManifest::load(&ctx.run_dir)?;
    let view = RunView::new(state, ledger, coverage);
    let mut rs = status(&view);
    let mine = |id: &str| id.split_once('/').is_some_and(|(p, _)| p == w);
    rs.blocking.retain(|b| match b.kind {
        BlockingKind::Finding | BlockingKind::OpenItem => mine(&b.id),
        BlockingKind::ChecklistItem => false,
        BlockingKind::Run => true,
    });
    // The counts describe this worker's slice, not the whole run.
    let mut counts = dla_core::view::StatusCounts::default();
    for f in view.ledger.findings().filter(|f| mine(&f.id.to_string())) {
        counts.enumerated += 1;
        counts.dispositioned += usize::from(f.state.is_dispositioned());
        let slot = &mut counts.by_state;
        match f.state {
            FindingState::Candidate => slot.candidate += 1,
            FindingState::UnderTest => slot.under_test += 1,
            FindingState::Confirmed => slot.confirmed += 1,
            FindingState::Strong => slot.strong += 1,
            FindingState::Refuted => slot.refuted += 1,
            FindingState::SuspectedStaticOnly => slot.suspected_static_only += 1,
            FindingState::Unresolved => slot.unresolved += 1,
        }
    }
    counts.open_items = rs
        .blocking
        .iter()
        .filter(|b| b.kind == BlockingKind::OpenItem)
        .count();
    rs.counts = counts;
    let now = state::now_utc();
    let deadline = view
        .state
        .worker(w)
        .map(|e| (format!("worker {w}"), e.deadline));
    for (who, at) in deadline
        .into_iter()
        .chain([("run".to_owned(), view.state.budget.deadline)])
    {
        if now > at {
            rs.blocking.push(BlockingItem {
                kind: BlockingKind::Run,
                id: view.state.run_id.clone(),
                description: format!(
                    "the {who} time budget is spent (deadline {}); no new stage may start",
                    at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                ),
                fix: format!(
                    "dla --worker {w} ledger resolve <F> --as unresolved --reason time_budget --remedy \"<text>\", then dla --worker {w} ledger done"
                ),
            });
            break;
        }
    }
    rs.complete = rs.blocking.is_empty();
    Ok((view, rs))
}

/// At most this many `revisit` entries are listed; `revisit_total` has the real count.
const MAX_REVISIT: usize = 10;

/// Longest remedy text kept in a `revisit` entry, in characters.
const MAX_REMEDY_CHARS: usize = 120;

/// Findings that need another look (`unresolved`, `suspected_static_only`), in ledger
/// order: `{id, state, reason, remedy}`. They do not block the report, so `blocking`
/// never lists them.
fn revisit_entries(view: &RunView, worker: Option<&str>) -> Vec<Value> {
    view.ledger
        .lines()
        .iter()
        .filter_map(|l| match &l.record {
            Record::FindingAdded { finding } => view.ledger.finding(&finding.id),
            _ => None,
        })
        .filter(|f| worker.is_none_or(|w| f.id.prefix() == Some(w)))
        .filter(|f| {
            matches!(
                f.state,
                FindingState::Unresolved | FindingState::SuspectedStaticOnly
            )
        })
        .map(|f| {
            let remedy = f.remedy.clone().unwrap_or_default();
            let remedy = if remedy.chars().count() > MAX_REMEDY_CHARS {
                let cut: String = remedy.chars().take(MAX_REMEDY_CHARS).collect();
                format!("{cut}...")
            } else {
                remedy
            };
            json!({
                "id": f.id.to_string(),
                "state": f.state.as_str(),
                "reason": f.reason.map(|r| r.as_str()),
                "remedy": remedy,
            })
        })
        .collect()
}
