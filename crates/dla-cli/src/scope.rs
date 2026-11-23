//! What every `dla` command of a run does around its own work: it registers itself while it
//! runs when it can take long (so the stop hook sees that the run is still working), settles
//! an `auto` budget once sweep and census have run, and lifts a stop-hook stamp when the run
//! goes on (a command that works on the run lifts it before it starts).

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

/// Lifts the stop-hook stamp of the run, when it has one: the run is going again. Best effort.
pub fn lift_stop_stamp(ctx: &Ctx) {
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

/// True when sweep and census have run for `runtime`: its coverage lists the `census`
/// backend and at least one other.
pub fn sweep_and_census_ran_for(cov: Option<&CoverageManifest>, runtime: &str) -> bool {
    cov.is_some_and(|c| {
        c.workers.iter().any(|(key, w)| {
            (key == runtime || w.runtime == runtime)
                && w.backends.contains_key("census")
                && w.backends.len() > 1
        })
    })
}

/// True when the budget is ready to be sized by the command of `worker` (`None` for the run's
/// own): sweep and census have run, for the worker's runtime in a worker's command. In a
/// dispatched run (the state has registered workers) sizing belongs to the workers, so the
/// run's own command never sizes it: its own ledger holds no candidate of the workers' slices.
pub fn budget_ready(
    state: &RunState,
    cov: Option<&CoverageManifest>,
    worker: Option<&str>,
) -> bool {
    match worker {
        None => state.workers.is_empty() && sweep_and_census_ran(cov),
        Some(w) => state
            .workers
            .iter()
            .find(|x| x.name == w)
            .is_some_and(|x| sweep_and_census_ran_for(cov, &x.runtime)),
    }
}

/// Settles an `auto` budget when sweep and census have run and it is not settled yet, with the
/// same formula and the same state change as a passing gate check. In a run without workers it
/// sizes the budget from the run's own view and the sizing is final. In a dispatched run only a
/// worker's command sizes it (from the main ledger and every worker's slice), raise-only and
/// once per worker. Returns what it settled to and the extensions already on top of it.
pub fn settle_auto_budget(ctx: &Ctx) -> Option<(dla_core::state::BudgetEstimate, u64)> {
    let worker = ctx.worker.as_deref();
    let state = RunState::load(&ctx.run_dir).ok()?;
    if !state.budget.auto || state.budget.settled || matches!(state.status, RunStatus::Aborted) {
        return None;
    }
    if worker.is_some_and(|w| state.budget.settled_workers.iter().any(|x| x == w)) {
        return None;
    }
    let cov = CoverageManifest::load(&ctx.run_dir).ok()?;
    if !budget_ready(&state, cov.as_ref(), worker) {
        return None;
    }
    let view = RunView::load_for(&ctx.run_dir, worker).ok()?;
    let estimate = match worker {
        Some(_) => timebudget::estimate_across(&ctx.run_dir, &view),
        None => timebudget::estimate(&view, None),
    };
    drop(view);
    RunState::update(&ctx.run_dir, |st| {
        let ext: u64 = st
            .budget
            .extensions
            .iter()
            .map(|x| x.seconds + x.lapsed_seconds)
            .sum();
        Ok(st.settle_budget(estimate.clone(), worker).map(|_| {
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
