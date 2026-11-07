//! `dla run init` and `dla run status`.

use clap::{Args, Subcommand};
use dla_core::{
    Ledger, RunState, RunView,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool,
    },
    state::{self, STATE_FILE, Scope, ScopeKind},
    status,
};
use serde_json::{Value, json};

use crate::{
    budget,
    ctx::{Ctx, project_root},
    fail::Res,
    git, gitignore,
    parse::parse_enum,
};

/// `dla run ...`
#[derive(Subcommand, Debug)]
pub enum RunCmd {
    /// Create a run directory, its state and an empty ledger.
    Init(InitArgs),
    /// Show run id, stage, status, counts and the first blocking items.
    Status,
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
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// Runs `dla run init`.
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
    let root_text = project.to_string_lossy().into_owned();
    let st = RunState::init(
        &root_text,
        session.as_deref(),
        seconds,
        scope.clone(),
        state::now_utc(),
    )?;
    let ctx = Ctx {
        run_dir: state::run_dir(&project, &st.run_id),
        run_id: st.run_id.clone(),
    };
    std::fs::create_dir_all(&ctx.run_dir).map_err(|e| dla_core::Error::io(&ctx.run_dir, e))?;
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
        result: json!({
            "project_root": root_text,
            "session_id": st.session_id,
            "budget_seconds": seconds,
            "scope": scope,
        }),
        raw: None,
        coverage: EvidenceCoverage {
            status: CoverageStatus::Complete,
            examined: None,
            total: None,
        },
        limitations: Vec::new(),
        unknowns: Vec::new(),
    })?;
    state::write_current(&project, &st.run_id)?;
    let ignored = gitignore::ensure_dla_ignored(&project)?;
    Ok(json!({
        "ok": true,
        "run_id": st.run_id,
        "stage": st.stage.as_str(),
        "status": st.status.as_str(),
        "session_id": st.session_id,
        "budget_seconds": seconds,
        "evidence": "E1",
        "gitignore": ignored.as_str(),
        "record": ctx.pointer(STATE_FILE),
        "ledger": ctx.ledger_line(line.seq),
    }))
}

/// Runs `dla run status`.
pub fn run_status(ctx: &Ctx) -> Res<Value> {
    let view = RunView::load(&ctx.run_dir)?;
    let rs = status(&view);
    let total = rs.blocking.len();
    let shown: Vec<_> = rs.blocking.iter().take(3).collect();
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
    if total > shown.len() {
        out["truncated"] = Value::Bool(true);
    }
    Ok(out)
}
