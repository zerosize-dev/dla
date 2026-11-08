//! `dla run abort`: stop what the run started, undo what it attached, mark it aborted.

use std::{
    path::{Component, Path},
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

/// The directory name of a JVM attach file `.java_pid<pid>` / `.attach_pid<pid>`.
fn is_jvm_attach_file(name: &str) -> bool {
    [".java_pid", ".attach_pid"].iter().any(|p| {
        name.strip_prefix(p)
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
    })
}

/// Whether `path` is a plain absolute path with no `.` or `..` parts.
fn is_plain_absolute(path: &Path) -> bool {
    path.is_absolute()
        && path
            .components()
            .all(|c| !matches!(c, Component::ParentDir | Component::CurDir))
}

/// Whether `rm <path>` may run: DLA only ever registers files inside the run
/// directory, the JVM attach files `.java_pid<pid>` / `.attach_pid<pid>` and the
/// heaptrack fifo `/tmp/heaptrack_fifo<pid>`. `state.json` can come from a cloned
/// repository, so nothing else is deleted.
fn rm_allowed(run_dir: &Path, path: &Path) -> bool {
    if !is_plain_absolute(path) {
        return false;
    }
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if is_jvm_attach_file(name) {
        return true;
    }
    if name
        .strip_prefix("heaptrack_fifo")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        && path.parent() == Some(Path::new("/tmp"))
    {
        return true;
    }
    // Inside the run directory, compared on resolved paths so a symlink cannot lead out.
    let Ok(base) = std::fs::canonicalize(run_dir) else {
        return false;
    };
    let Some(parent) = path.parent().and_then(|p| std::fs::canonicalize(p).ok()) else {
        return false;
    };
    parent.starts_with(&base)
}

/// Whether `run <argv>` may run: the program is one DLA registers, `jcmd`, `memray`
/// (or the python interpreter running `-m memray`), or the heaptrack stop shim
/// `attach/<id>/.shim/gdb` inside the run directory.
fn run_allowed(run_dir: &Path, argv: &[String]) -> bool {
    let Some(program) = argv.first() else {
        return false;
    };
    let path = Path::new(program);
    let base = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    match base {
        "jcmd" | "memray" => path.is_absolute() || path.components().count() == 1,
        "gdb" => {
            let attach = run_dir.join("attach");
            is_plain_absolute(path)
                && path.starts_with(&attach)
                && path.parent().and_then(Path::file_name) == Some(std::ffi::OsStr::new(".shim"))
                && argv.get(1).map(String::as_str) == Some("--dla-stop")
        }
        b if b == "python3" || b == "python" => {
            path.is_absolute()
                && argv.get(1).map(String::as_str) == Some("-m")
                && argv.get(2).map(String::as_str) == Some("memray")
        }
        _ => false,
    }
}

/// Runs the recorded cleanup. With `process_gone` the `run` actions (which name the
/// pid) are skipped: the pid is no longer the recorded process.
fn run_cleanup(run_dir: &Path, p: &ProcessRecord, process_gone: bool) -> Fate {
    let mut failures = Vec::new();
    let mut skipped = false;
    for entry in &p.cleanup {
        match CleanupAction::parse(entry) {
            Ok(CleanupAction::Note(_)) => {}
            Ok(CleanupAction::Rm(path)) => {
                if !rm_allowed(run_dir, &path) {
                    failures.push(format!(
                        "refused rm {}: not a file DLA registers",
                        dla_core::paths::to_slash(&path)
                    ));
                    continue;
                }
                match std::fs::remove_file(&path) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => {
                        failures.push(format!("rm {}: {e}", dla_core::paths::to_slash(&path)));
                    }
                }
            }
            Ok(CleanupAction::Run(argv)) => {
                if !run_allowed(run_dir, &argv) {
                    failures.push(format!("refused run {}: not a tool DLA registers", argv[0]));
                } else if process_gone {
                    skipped = true;
                } else {
                    match proc::run_bounded(&argv, CLEANUP_TIMEOUT) {
                        RunOutcome::Ok => {}
                        RunOutcome::Failed(why) => failures.push(why),
                        RunOutcome::TimedOut => {
                            failures.push(format!("{}: timed out after 10 s", argv[0]));
                        }
                    }
                }
            }
            Err(why) => failures.push(why),
        }
    }
    if !failures.is_empty() {
        Fate::Failed(failures.join("; "))
    } else if skipped {
        Fate::Gone(format!(
            "pid {} no longer is the attached process; its detach commands were not run",
            p.pid
        ))
    } else {
        Fate::Cleaned
    }
}

fn handle(run_dir: &Path, p: &ProcessRecord) -> Fate {
    match p.role {
        ProcessRole::Attached => {
            // With a recorded start time the target must still be that process;
            // otherwise a reused pid would be attached to by gdb or memray.
            let gone = p.start_time.is_some() && {
                let live = dla_sample::os::process_start_time(p.pid);
                !proc::is_alive(p.pid) || !proc::identity_matches(p, live.as_deref())
            };
            run_cleanup(run_dir, p, gone)
        }
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

/// Whether the recorded project root names `project` (compared as resolved paths).
fn same_project(recorded: &str, project: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    canon(Path::new(recorded)) == canon(project)
}

/// Runs `dla run abort`.
pub fn run(ctx: &Ctx, project: &Path, a: AbortArgs) -> Res<Value> {
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
    // `state.json` can come from a cloned repository: it must be this project's run
    // before anything it names is executed or deleted.
    if !same_project(&snapshot.project_root, project) {
        return Err(Failure::refuse(
            dla_core::error::codes::INVALID_VALUE,
            format!(
                "run {} records project {}, which is not this project ({}); nothing was stopped or deleted",
                ctx.run_id,
                snapshot.project_root,
                dla_core::paths::to_slash(project)
            ),
            "a run created by dla run init in this project",
            "dla run init",
        ));
    }
    let (mut killed, mut cleaned, mut gone, mut failed) = (0u64, 0u64, 0u64, 0u64);
    let mut skipped = 0u64;
    let mut notes: Vec<String> = Vec::new();
    let mut done: Vec<(Key, Option<String>)> = Vec::new();
    for p in &snapshot.processes {
        if p.stopped_at.is_some() {
            skipped += 1;
            continue;
        }
        match handle(&ctx.run_dir, p) {
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
