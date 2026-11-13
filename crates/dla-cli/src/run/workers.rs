//! `dla run dispatch`: registers workers with their shares of the remaining budget.

use std::path::PathBuf;

use chrono::{SecondsFormat, TimeDelta};
use clap::Args;
use dla_core::{
    RunState, RunStatus,
    error::codes as core_codes,
    paths,
    runtime::Runtime,
    state::{self, WorkerEntry, WorkerStatus},
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
};

/// Flags of `dla run dispatch`.
#[derive(Args, Debug)]
pub struct DispatchArgs {
    /// Comma-separated runtime names to register as workers (jvm, go, rust, node, python, native).
    #[arg(long, value_delimiter = ',', required = true)]
    workers: Vec<String>,
    /// Fraction of the remaining budget kept back for the core (merge, verification, report).
    #[arg(long, default_value_t = 0.15)]
    reserve: f64,
}

pub(super) fn refuse_worker_mode(ctx: &Ctx, what: &str) -> Res<()> {
    match &ctx.worker {
        None => Ok(()),
        Some(w) => Err(Failure::refuse(
            core_codes::INVALID_VALUE,
            format!("{what} is a core command and this call carries --worker {w}"),
            "a call without --worker (the core runs it, a worker never does)",
            format!("dla --run {} {what}", ctx.run_id),
        )),
    }
}

/// The language reference of `runtime`: `<plugin root>/skills/detect-leak-anywhere/references/<runtime>.md`.
fn reference_path(root: Option<&PathBuf>, runtime: &str) -> (String, bool) {
    let rel = format!("skills/detect-leak-anywhere/references/{runtime}.md");
    match root {
        Some(r) => {
            let p = r.join(&rel);
            (paths::to_slash(&p), p.is_file())
        }
        None => (format!("<plugin root>/{rel}"), false),
    }
}

fn rfc3339(t: chrono::DateTime<chrono::Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Runs `dla run dispatch`.
pub fn dispatch(ctx: &Ctx, a: DispatchArgs) -> Res<Value> {
    refuse_worker_mode(ctx, "run dispatch")?;
    if !(0.0..1.0).contains(&a.reserve) {
        return Err(Failure::refuse(
            core_codes::INVALID_VALUE,
            format!("--reserve {} is not a fraction in [0, 1)", a.reserve),
            "a reserve such as 0.15",
            "dla run dispatch --workers jvm,rust --reserve 0.15",
        ));
    }
    let mut names: Vec<String> = Vec::new();
    for w in &a.workers {
        let w = w.trim();
        let runtime: Runtime = w.parse().map_err(|_| {
            Failure::refuse(
                core_codes::INVALID_VALUE,
                format!("{w:?} is not a runtime name"),
                format!("a worker name from {}", Runtime::choices()),
                "dla run dispatch --workers jvm,rust",
            )
        })?;
        if names.iter().any(|n| n == runtime.as_str()) {
            return Err(Failure::refuse(
                core_codes::DUPLICATE_ID,
                format!("worker {w} is listed twice"),
                "each worker once",
                "dla run dispatch --workers jvm,rust",
            ));
        }
        names.push(runtime.as_str().to_owned());
    }
    let registered = RunState::update(&ctx.run_dir, |st| {
        if st.status != RunStatus::Active {
            return Err(dla_core::Error::Refused(dla_core::Refusal::new(
                core_codes::RUN_NOT_ACTIVE,
                format!(
                    "run {} is {}, so it cannot take workers",
                    st.run_id, st.status
                ),
                "an active run",
                "dla run init",
            )));
        }
        if let Some(n) = names.iter().find(|n| st.worker(n).is_some()) {
            return Err(dla_core::Error::Refused(dla_core::Refusal::new(
                core_codes::DUPLICATE_ID,
                format!("worker {n} is already registered"),
                format!(
                    "a worker name that is not registered yet (registered: {})",
                    st.workers
                        .iter()
                        .map(|w| w.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                "dla run status   (then dispatch only the runtimes that have no worker)",
            )));
        }
        let now = state::now_utc();
        let remaining = (st.budget.deadline - now).num_seconds();
        if remaining <= 0 {
            return Err(dla_core::Error::Refused(dla_core::Refusal::new(
                core_codes::TIME_BUDGET,
                format!(
                    "nothing remains of the run budget (deadline {})",
                    rfc3339(st.budget.deadline)
                ),
                "time before the run deadline",
                dla_core::toolctx::deadline_fix(false, Some(st)),
            )));
        }
        // Truncation is intended: whole seconds, never more than the pool.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let pool = (remaining as f64 * (1.0 - a.reserve)).floor() as u64;
        let share = pool / names.len() as u64;
        if share == 0 {
            return Err(dla_core::Error::Refused(dla_core::Refusal::new(
                core_codes::TIME_BUDGET,
                format!(
                    "{remaining} s remain; after the reserve each of {} workers would get 0 s",
                    names.len()
                ),
                "at least 1 s per worker",
                "dla run dispatch --workers <fewer runtimes> --reserve 0",
            )));
        }
        let deadline = i64::try_from(share)
            .ok()
            .and_then(TimeDelta::try_seconds)
            .and_then(|d| now.checked_add_signed(d))
            .unwrap_or(st.budget.deadline);
        let mut out = Vec::new();
        for n in &names {
            let entry = WorkerEntry {
                name: n.clone(),
                runtime: n.clone(),
                share_secs: share,
                deadline,
                started_at: None,
                status: WorkerStatus::Registered,
                slice: format!("ledger.{n}.jsonl"),
                stalled: false,
            };
            st.workers.push(entry.clone());
            out.push(entry);
        }
        Ok((out, st.scope.clone()))
    })?;
    let (entries, scope) = registered;
    let root = paths::plugin_root();
    let workers: Vec<Value> = entries
        .iter()
        .map(|w| {
            let (reference, exists) = reference_path(root.as_ref(), &w.runtime);
            json!({
                "name": w.name,
                "deadline": rfc3339(w.deadline),
                "share_secs": w.share_secs,
                "reference": reference,
                "reference_exists": exists,
                "prefix": format!("dla --run {} --worker {}", ctx.run_id, w.name),
            })
        })
        .collect();
    Ok(json!({
        "ok": true,
        "run_id": ctx.run_id,
        "scope": scope,
        "reserve": a.reserve,
        "workers": workers,
        "next": "start one dla:worker subagent per worker with these facts, wait for all, then: dla ledger merge",
    }))
}

/// Number of lines in a slice file (0 when absent); a last line without a newline counts.
fn line_count(path: &std::path::Path) -> u64 {
    let Ok(bytes) = std::fs::read(path) else {
        return 0;
    };
    let nl = bytes.iter().filter(|b| **b == b'\n').count() as u64;
    nl + u64::from(bytes.last().is_some_and(|b| *b != b'\n'))
}

/// The `entry`, `workers` and `running` parts of `dla run status`.
pub fn status_parts(ctx: &Ctx, st: &RunState) -> Value {
    let workers: Vec<Value> = st
        .workers
        .iter()
        .map(|w| {
            json!({
                "name": w.name,
                "status": w.status.as_str(),
                "deadline": rfc3339(w.deadline),
                "slice_lines": line_count(&ctx.run_dir.join(&w.slice)),
            })
        })
        .collect();
    let running: Vec<&state::ProcessRecord> = st
        .processes
        .iter()
        .filter(|p| p.stopped_at.is_none())
        .collect();
    let shown: Vec<Value> = running
        .iter()
        .take(5)
        .map(|p| {
            json!({
                "pid": p.pid,
                "kind": p.kind.as_str(),
                "role": p.role.as_str(),
                "worker": p.worker,
            })
        })
        .collect();
    json!({
        "entry": st.entry.as_str(),
        "artifacts": st.artifacts,
        "workers": workers,
        "running": shown,
        "running_total": running.len(),
    })
}
