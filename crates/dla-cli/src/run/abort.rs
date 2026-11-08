//! `dla run abort`: stop what the run started, undo what it attached, mark it aborted.

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use clap::Args;
use dla_core::{
    RunState,
    proc::{self, CLEANUP_TIMEOUT, CleanupAction, RunOutcome},
    state::{ProcessRecord, ProcessRole, RunStatus, now_utc},
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
};

/// Flags of `dla run abort`.
#[derive(Args, Debug)]
pub struct AbortArgs {
    /// Why the run is abandoned (recorded as the run's incomplete reason).
    #[arg(long)]
    reason: Option<String>,
}

/// Grace between SIGTERM and SIGKILL, and the wait after each signal.
const GRACE: Duration = Duration::from_secs(2);

/// What happened to one record.
enum Fate {
    Killed,
    Cleaned,
    Gone(String),
    Failed(String),
}

/// A record's identity in the state file: pid plus both timestamps.
type Key = (u32, Option<String>, DateTime<Utc>);

fn key(p: &ProcessRecord) -> Key {
    (p.pid, p.start_time.clone(), p.started_at)
}

fn wait_gone(pid: u32, limit: Duration) -> bool {
    let start = Instant::now();
    while proc::is_alive(pid) {
        if start.elapsed() >= limit {
            return false;
        }
        sleep(Duration::from_millis(50));
    }
    true
}

/// SIGTERM to the group, a grace period, SIGKILL; the identity was verified before.
fn stop_started(pid: u32) -> Fate {
    let term = proc::kill_group(pid, "TERM") || proc::kill_pid(pid, "TERM");
    if term && wait_gone(pid, GRACE) {
        return Fate::Killed;
    }
    let kill = proc::kill_group(pid, "KILL") || proc::kill_pid(pid, "KILL");
    if wait_gone(pid, GRACE) {
        Fate::Killed
    } else if kill {
        Fate::Failed(format!("pid {pid} survived SIGKILL"))
    } else {
        Fate::Failed(format!("could not signal pid {pid}"))
    }
}

fn run_cleanup(p: &ProcessRecord) -> Fate {
    let mut failures = Vec::new();
    for entry in &p.cleanup {
        match CleanupAction::parse(entry) {
            Ok(CleanupAction::Note(_)) => {}
            Ok(CleanupAction::Rm(path)) => match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => failures.push(format!("rm {}: {e}", dla_core::paths::to_slash(&path))),
            },
            Ok(CleanupAction::Run(argv)) => match proc::run_bounded(&argv, CLEANUP_TIMEOUT) {
                RunOutcome::Ok => {}
                RunOutcome::Failed(why) => failures.push(why),
                RunOutcome::TimedOut => failures.push(format!("{}: timed out after 10 s", argv[0])),
            },
            Err(why) => failures.push(why),
        }
    }
    if failures.is_empty() {
        Fate::Cleaned
    } else {
        Fate::Failed(failures.join("; "))
    }
}

fn handle(p: &ProcessRecord) -> Fate {
    match p.role {
        ProcessRole::Attached => run_cleanup(p),
        ProcessRole::Started => {
            let live = dla_sample::os::process_start_time(p.pid);
            if !proc::is_alive(p.pid) {
                Fate::Gone("process was already gone at abort".to_owned())
            } else if proc::identity_matches(p, live.as_deref()) {
                stop_started(p.pid)
            } else {
                Fate::Gone(format!(
                    "pid {} no longer is the recorded process (start time {} now {}); not signalled",
                    p.pid,
                    p.start_time.as_deref().unwrap_or("unknown"),
                    live.as_deref().unwrap_or("unreadable"),
                ))
            }
        }
    }
}

/// Runs `dla run abort`.
pub fn run(ctx: &Ctx, a: AbortArgs) -> Res<Value> {
    if let Some(w) = &ctx.worker {
        return Err(Failure::refuse(
            dla_core::error::codes::INVALID_VALUE,
            format!("run abort is a core command and this call carries --worker {w}"),
            "a call without --worker",
            format!("dla --run {} run abort", ctx.run_id),
        ));
    }
    // Snapshot, then act without holding the run lock (a stop takes seconds), then
    // apply the outcome to the then-current state in one locked update.
    let snapshot = RunState::load(&ctx.run_dir)?;
    let (mut killed, mut cleaned, mut gone, mut failed) = (0u64, 0u64, 0u64, 0u64);
    let mut skipped = 0u64;
    let mut notes: Vec<String> = Vec::new();
    let mut done: Vec<(Key, Option<String>)> = Vec::new();
    for p in &snapshot.processes {
        if p.stopped_at.is_some() {
            skipped += 1;
            continue;
        }
        match handle(p) {
            Fate::Killed => {
                killed += 1;
                done.push((key(p), None));
            }
            Fate::Cleaned => {
                cleaned += 1;
                done.push((key(p), None));
            }
            Fate::Gone(note) => {
                gone += 1;
                notes.push(format!("pid {}: {note}", p.pid));
                done.push((key(p), Some(note)));
            }
            Fate::Failed(why) => {
                failed += 1;
                notes.push(format!("pid {}: {why}", p.pid));
            }
        }
    }
    let given = a.reason.filter(|r| !r.trim().is_empty());
    let stamp = now_utc();
    let status = RunState::update(&ctx.run_dir, |st| {
        for p in st.processes.iter_mut().filter(|p| p.stopped_at.is_none()) {
            if let Some((_, note)) = done.iter().find(|(k, _)| *k == key(p)) {
                p.stopped_at = Some(stamp);
                if let Some(n) = note {
                    p.cleanup.push(format!("note abort: {n}"));
                }
            }
        }
        // A repeated abort without a new reason keeps the first one.
        let keep = st.status == RunStatus::Aborted && given.is_none();
        st.status = RunStatus::Aborted;
        if !keep || st.incomplete_reason.is_none() {
            st.incomplete_reason = Some(
                given
                    .clone()
                    .unwrap_or_else(|| "aborted by dla run abort".to_owned()),
            );
        }
        Ok((st.status, st.incomplete_reason.clone().unwrap_or_default()))
    })?;
    let (status, reason) = status;
    notes.truncate(3);
    Ok(json!({
        "ok": failed == 0,
        "run_id": ctx.run_id,
        "status": status.as_str(),
        "reason": reason,
        "killed": killed,
        "cleaned": cleaned,
        "gone": gone,
        "skipped": skipped,
        "failed": failed,
        "notes": notes,
    }))
}
