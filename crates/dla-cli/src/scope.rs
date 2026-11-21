//! What every `dla` command of a run does around its own work: it registers itself while it
//! runs when it can take long (so the stop hook sees that the run is still working), settles
//! an `auto` budget once sweep and census have run, and lifts a stop-hook stamp when the run
//! goes on (the command wrote a ledger record).

use std::path::Path;

use dla_core::{
    CoverageManifest, RunState, RunView,
    state::{ProcessKind, ProcessRecord, ProcessRole, RunStatus, now_utc},
    timebudget,
};

use crate::ctx::Ctx;

/// A registered `dla` command: marks its record stopped when dropped.
pub struct Registered {
    run_dir: std::path::PathBuf,
    pid: u32,
    start_time: Option<String>,
}

impl Drop for Registered {
    fn drop(&mut self) {
        let (pid, start) = (self.pid, self.start_time.clone());
        let _ = RunState::update(&self.run_dir, |s| {
            if let Some(p) = s
                .processes
                .iter_mut()
                .rev()
                .find(|p| p.pid == pid && p.stopped_at.is_none() && p.start_time == start)
            {
                p.stopped_at = Some(now_utc());
            }
            Ok(())
        });
    }
}

/// Registers this process as the running `dla` command `command` of the run, as an attached
/// process (DLA never kills it). Best effort: a run that cannot be updated is simply not
/// registered.
pub fn register(ctx: &Ctx, command: &str) -> Option<Registered> {
    let pid = std::process::id();
    let start_time = dla_sample::os::process_start_time(pid);
    let record = ProcessRecord {
        pid,
        start_time: start_time.clone(),
        kind: ProcessKind::Command,
        command: command.to_owned(),
        started_at: now_utc(),
        stopped_at: None,
        role: ProcessRole::Attached,
        worker: ctx.worker.clone(),
        cleanup: Vec::new(),
    };
    RunState::update(&ctx.run_dir, |s| {
        // A record of an earlier command that died without marking itself stopped.
        s.processes.push(record);
        Ok(())
    })
    .ok()?;
    Some(Registered {
        run_dir: ctx.run_dir.clone(),
        pid,
        start_time,
    })
}

/// Total size of the run's ledger files (the main ledger and every worker slice): it grows
/// exactly when a command writes a record.
pub fn ledger_mark(run_dir: &Path) -> u64 {
    let Ok(rd) = std::fs::read_dir(run_dir) else {
        return 0;
    };
    rd.flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n == "ledger.jsonl" || (n.starts_with("ledger.") && n.ends_with(".jsonl"))
        })
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Lifts the stop-hook stamp of the run when the ledger grew since `before`: the run is going
/// again. Best effort.
pub fn lift_stamp_if_the_ledger_grew(ctx: &Ctx, before: u64) {
    if ledger_mark(&ctx.run_dir) <= before {
        return;
    }
    let _ = RunState::update(&ctx.run_dir, |s| Ok(s.lift_stop_stamp()));
}

/// True when sweep and census have run for some runtime: its coverage lists the `census`
/// backend and at least one other.
pub fn sweep_and_census_ran(cov: Option<&CoverageManifest>) -> bool {
    cov.is_some_and(|c| {
        c.workers
            .values()
            .any(|w| w.backends.contains_key("census") && w.backends.len() > 1)
    })
}

/// Settles an `auto` budget when sweep and census have run and it is not settled yet, with the
/// same formula and the same state change as the first passing gate check. The run's own
/// budget only (a worker's share follows the dispatch rules). Returns what it settled to and
/// the extensions already on top of it.
pub fn settle_auto_budget(ctx: &Ctx) -> Option<(dla_core::state::BudgetEstimate, u64)> {
    if ctx.worker.is_some() {
        return None;
    }
    let state = RunState::load(&ctx.run_dir).ok()?;
    if !state.budget.auto || state.budget.settled || matches!(state.status, RunStatus::Aborted) {
        return None;
    }
    let view = RunView::load(&ctx.run_dir).ok()?;
    if !sweep_and_census_ran(view.coverage.as_ref()) {
        return None;
    }
    let estimate = timebudget::estimate(&view, None);
    drop(view);
    RunState::update(&ctx.run_dir, |st| {
        let ext: u64 = st
            .budget
            .extensions
            .iter()
            .map(|x| x.seconds + x.lapsed_seconds)
            .sum();
        Ok(st.settle_budget(estimate.clone(), None).map(|_| {
            (
                st.budget
                    .estimate
                    .clone()
                    .unwrap_or_else(|| estimate.clone()),
                ext,
            )
        }))
    })
    .ok()
    .flatten()
}
