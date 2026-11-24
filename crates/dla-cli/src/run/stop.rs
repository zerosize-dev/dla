//! `dla run stop`: stop what the run started and undo what it attached, and leave the run open.
//! What a command that was cut (a timeout, a closed terminal) left running is the reason: the
//! run itself is fine, so nothing is marked aborted and everything can still be recorded.

use std::path::Path;

use clap::Args;
use dla_core::{RunState, state::now_utc};
use serde_json::{Value, json};

use super::abort::stop_processes;
use crate::{ctx::Ctx, fail::Res};

/// Flags of `dla run stop` (none).
#[derive(Args, Debug)]
pub struct StopArgs {}

/// What `dla run stop` says about the run.
const GOES_ON: &str = "the run goes on: it is still active and nothing was marked aborted; run the cut command again, or the next one";

/// Runs `dla run stop`. A call that carries `--worker` stops the processes that worker
/// registered; a call without it stops every process of the run.
pub fn run(ctx: &Ctx, project: &Path, _a: StopArgs) -> Res<Value> {
    let stopped = stop_processes(ctx, project, ctx.worker.as_deref(), "stop")?;
    let stamp = now_utc();
    let status = RunState::update(&ctx.run_dir, |st| {
        stopped.apply(st, stamp, "stop");
        Ok(st.status)
    })?;
    Ok(json!({
        "ok": stopped.failed == 0,
        "run_id": ctx.run_id,
        "message": GOES_ON,
        "status": status.as_str(),
        "killed": stopped.killed,
        "cleaned": stopped.cleaned,
        "gone": stopped.gone,
        "skipped": stopped.skipped,
        "failed": stopped.failed,
        "notes": stopped.notes,
    }))
}
