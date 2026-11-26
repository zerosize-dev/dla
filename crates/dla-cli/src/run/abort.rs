//! `dla run abort`: stop what the run started, undo what it attached, mark it aborted. The
//! stopping itself is shared with `dla run stop` (`stop_processes`), which leaves the run open.

use std::path::{Component, Path};

use chrono::{DateTime, Utc};
use clap::Args;
use dla_core::{
    RunState, RunView, abort_rule,
    error::codes as core_codes,
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
    /// What stands in the way of the run, when it is abandoned while budget is left and
    /// something is open: user, tool, build, environment, privilege or refusal. The reason
    /// then names it.
    #[arg(long, value_name = "KIND")]
    blocker: Option<String>,
    /// The code of the refusal the run cannot satisfy (with `--blocker refusal`).
    #[arg(long, value_name = "CODE")]
    code: Option<String>,
}

/// What `dla run abort` prints about the run it ended.
const ENDED: &str = "this run ends here: it is marked aborted for good and nothing more can be recorded in it that counts, except a verifier's record (dla bundle F, dla ledger verify F), which is still taken and counted; dla run init starts a new run; to stop only what a cut command left and keep the run, use dla run stop";

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

/// SIGTERM to the group, a grace period, SIGKILL; the identity was verified before.
fn stop_started(pid: u32) -> Fate {
    match proc::stop_started(pid) {
        Ok(()) => Fate::Killed,
        Err(why) => Fate::Failed(why),
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
    let file = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // Windows programs carry an `.exe` suffix (`jcmd.exe`); elsewhere the name is exact.
    let base = if cfg!(windows) {
        file.strip_suffix(".exe")
            .or_else(|| file.strip_suffix(".EXE"))
            .unwrap_or(file)
    } else {
        file
    };
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

fn handle(run_dir: &Path, p: &ProcessRecord, verb: &str) -> Fate {
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
                Fate::Gone(format!("process was already gone at {verb}"))
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

/// What stopping the run's processes did, to be applied to `state.json` by the caller.
pub(super) struct Stopped {
    pub killed: u64,
    pub cleaned: u64,
    pub gone: u64,
    pub skipped: u64,
    pub failed: u64,
    /// At most three, kept for the output.
    pub notes: Vec<String>,
    done: Vec<(Key, Option<String>)>,
}

impl Stopped {
    /// Marks the records this call stopped, in `st`, as stopped at `stamp`; a note is kept on a
    /// record whose process was already gone, under the name of the command (`verb`).
    pub(super) fn apply(&self, st: &mut RunState, stamp: DateTime<Utc>, verb: &str) {
        for p in st.processes.iter_mut().filter(|p| p.stopped_at.is_none()) {
            if let Some((_, note)) = self.done.iter().find(|(k, _)| *k == key(p)) {
                p.stopped_at = Some(stamp);
                if let Some(n) = note {
                    p.cleanup.push(format!("note {verb}: {n}"));
                }
            }
        }
    }
}

/// Stops the processes the run started and undoes what it attached (the records of `only`'s
/// worker alone when given), without touching the run's status. `state.json` can come from a
/// cloned repository: it must be this project's run before anything it names is executed or
/// deleted.
pub(super) fn stop_processes(
    ctx: &Ctx,
    project: &Path,
    only: Option<&str>,
    verb: &str,
) -> Res<Stopped> {
    // Snapshot, then act without holding the run lock (a stop takes seconds); the caller applies
    // the outcome to the then-current state in one locked update.
    let snapshot = RunState::load(&ctx.run_dir)?;
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
    let mut out = Stopped {
        killed: 0,
        cleaned: 0,
        gone: 0,
        skipped: 0,
        failed: 0,
        notes: Vec::new(),
        done: Vec::new(),
    };
    for p in &snapshot.processes {
        if p.stopped_at.is_some() || only.is_some_and(|w| p.worker.as_deref() != Some(w)) {
            out.skipped += u64::from(p.stopped_at.is_some());
            continue;
        }
        match handle(&ctx.run_dir, p, verb) {
            Fate::Killed => {
                out.killed += 1;
                out.done.push((key(p), None));
            }
            Fate::Cleaned => {
                out.cleaned += 1;
                out.done.push((key(p), None));
            }
            Fate::Gone(note) => {
                out.gone += 1;
                out.notes.push(format!("pid {}: {note}", p.pid));
                out.done.push((key(p), Some(note)));
            }
            Fate::Failed(why) => {
                out.failed += 1;
                out.notes.push(format!("pid {}: {why}", p.pid));
            }
        }
    }
    out.notes.truncate(3);
    Ok(out)
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
    // Budget left and work open: an abort needs a blocker the run cannot pass.
    let blocker = match a.blocker.as_deref() {
        Some(kind) => Some(
            abort_rule::check_blocker(kind, a.reason.as_deref(), a.code.as_deref()).map_err(
                |(code, message, fix)| {
                    Failure::refuse(
                        code,
                        message,
                        "a blocker of the run, with the reason that names it",
                        format!("dla run abort --blocker {kind} {fix}"),
                    )
                },
            )?,
        ),
        None => None,
    };
    if blocker.as_ref().is_none_or(|b| b.kind != "user") {
        // A run that cannot be loaded has no work to point at: the abort goes on to its own checks.
        if blocker.is_none()
            && let Some(open) = RunView::load(&ctx.run_dir)
                .ok()
                .and_then(|view| abort_rule::open_work(&view, &ctx.run_dir))
        {
            return Err(Failure::refuse(
                core_codes::BUDGET_LEFT,
                format!(
                    "{} of {} budget minutes remain and work is open ({}); an abort is no way out while the run can still measure",
                    open.left,
                    open.total,
                    open.describe()
                ),
                "the open work done, or a blocker the run cannot pass",
                format!(
                    "{}   or   dla run abort --blocker <user|tool|build|environment|privilege|refusal> --reason \"<what blocks the run>\"",
                    open.first
                ),
            ));
        }
    }
    let stopped = stop_processes(ctx, project, None, "abort")?;
    let given = match &blocker {
        Some(b) => Some(b.incomplete_reason()),
        None => a.reason.filter(|r| !r.trim().is_empty()),
    };
    let stamp = now_utc();
    let status = RunState::update(&ctx.run_dir, |st| {
        stopped.apply(st, stamp, "abort");
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
    // The verifier is not skipped by an abort: the confirmed findings it has not read are
    // listed with what to run.
    let unverified: Vec<Value> = RunView::load(&ctx.run_dir)
        .map(|v| abort_rule::unverified_confirmed(&v))
        .unwrap_or_default()
        .iter()
        .map(|id| json!({"finding": id.to_string(), "verify": abort_rule::verify_steps(id)}))
        .collect();
    let mut out = json!({
        "ok": stopped.failed == 0,
        "run_id": ctx.run_id,
        "message": ENDED,
        "status": status.as_str(),
        "reason": reason,
        "killed": stopped.killed,
        "cleaned": stopped.cleaned,
        "gone": stopped.gone,
        "skipped": stopped.skipped,
        "failed": stopped.failed,
        "notes": stopped.notes,
    });
    if !unverified.is_empty() {
        out["unverified"] = Value::Array(unverified.clone());
        out["next"] = json!(format!(
            "{} confirmed finding(s) have no verification record: run the verifier for each (dla bundle F, one dla:verifier subagent, dla ledger verify); the Stop hook holds the stop until they are verified",
            unverified.len()
        ));
    }
    Ok(out)
}
