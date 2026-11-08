//! Process helpers shared by the commands that start, attach to and abort processes:
//! group kill, liveness, identity matching and the cleanup-action grammar.
//!
//! `dla-core` does not depend on `dla-sample`, so the live start time used for
//! identity matching is passed in by the caller.

use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use chrono::DateTime;

use crate::state::ProcessRecord;

/// How far apart (seconds) a recorded and a live start time may be and still name the
/// same process.
pub const IDENTITY_TOLERANCE_SECS: i64 = 2;

/// Longest a `run` cleanup action may take.
pub const CLEANUP_TIMEOUT: Duration = Duration::from_secs(10);

/// Runs `kill -s <signal> -- <target>`: the `kill` binary when one can be started,
/// else the shell builtin (minimal images ship no `kill` binary). Only a failure to
/// start the binary falls back; a non-zero exit (no such process) is the answer.
#[cfg(unix)]
fn unix_kill(signal: &str, target: &str) -> bool {
    unix_kill_with("kill", signal, target)
}

#[cfg(unix)]
fn unix_kill_with(program: &str, signal: &str, target: &str) -> bool {
    let quiet = |c: &mut Command| {
        c.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    };
    match quiet(Command::new(program).args(["-s", signal, "--", target])) {
        Ok(s) => s.success(),
        Err(_) => quiet(
            Command::new("sh")
                .arg("-c")
                .arg(r#"kill -s "$1" -- "$2""#)
                .arg("dla-kill")
                .arg(signal)
                .arg(target),
        )
        .is_ok_and(|s| s.success()),
    }
}

/// Sends `signal` (`TERM`, `KILL`, ... without the `SIG` prefix) to the process group
/// `pgid`. Returns whether the signal was delivered. On unix this runs
/// `kill -s <SIG> -- -<pgid>` (dash rejects `kill -KILL -- -pgid`); on Windows it runs
/// `taskkill /T` (with `/F` for `KILL`).
pub fn kill_group(pgid: u32, signal: &str) -> bool {
    if pgid == 0 || signal.is_empty() || !signal.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    #[cfg(unix)]
    {
        unix_kill(signal, &format!("-{pgid}"))
    }
    #[cfg(windows)]
    {
        let mut c = Command::new("taskkill");
        if signal.eq_ignore_ascii_case("KILL") {
            c.arg("/F");
        }
        c.args(["/T", "/PID", &pgid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = signal;
        false
    }
}

/// Sends `signal` to the single process `pid` (not its group); for a process that is
/// not a group leader. Returns whether the signal was delivered.
pub fn kill_pid(pid: u32, signal: &str) -> bool {
    if pid == 0 || signal.is_empty() || !signal.chars().all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    #[cfg(unix)]
    {
        unix_kill(signal, &pid.to_string())
    }
    #[cfg(windows)]
    {
        let mut c = Command::new("taskkill");
        if signal.eq_ignore_ascii_case("KILL") {
            c.arg("/F");
        }
        c.args(["/PID", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

/// Whether `pid` is a running process (a zombie awaiting reaping counts as gone).
pub fn is_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            // `pid (comm) S ...`: the state letter follows the last `)`.
            return stat
                .rsplit_once(')')
                .and_then(|(_, rest)| rest.trim_start().chars().next())
                .is_some_and(|state| state != 'Z' && state != 'X');
        }
        false
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        // `ps -o stat=` prints the state letters, or nothing (exit 1) when the pid is gone.
        // A zombie (`Z`) has exited and only waits to be reaped, so it is not alive; `kill -0`
        // would say it is.
        Command::new("ps")
            .args(["-o", "stat=", "-p", &pid.to_string()])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .is_ok_and(|o| {
                let stat = String::from_utf8_lossy(&o.stdout);
                let stat = stat.trim();
                o.status.success() && !stat.is_empty() && !stat.starts_with('Z')
            })
    }
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .stdin(Stdio::null())
            .output()
            .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

/// Whether the live process is the one `record` describes: the record has a start
/// time, the caller could read the live one, and the two are within
/// [`IDENTITY_TOLERANCE_SECS`]. Anything unverifiable is a mismatch, so a caller that
/// signals only on a match never signals a reused pid.
pub fn identity_matches(record: &ProcessRecord, live_start_time: Option<&str>) -> bool {
    let (Some(recorded), Some(live)) = (record.start_time.as_deref(), live_start_time) else {
        return false;
    };
    match (
        DateTime::parse_from_rfc3339(recorded),
        DateTime::parse_from_rfc3339(live),
    ) {
        (Ok(a), Ok(b)) => (a - b).num_seconds().abs() <= IDENTITY_TOLERANCE_SECS,
        _ => recorded == live,
    }
}

/// One entry of a [`ProcessRecord::cleanup`] list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanupAction {
    /// `rm <absolute path>`: delete a file DLA created.
    Rm(PathBuf),
    /// `run <JSON array argv>`: run a command (10 s timeout).
    Run(Vec<String>),
    /// `note <text>`: nothing to execute; reported as is.
    Note(String),
}

impl CleanupAction {
    /// Parses one cleanup entry; the error says what is wrong with it.
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        let (verb, rest) = text.split_once(' ').unwrap_or((text, ""));
        let rest = rest.trim();
        match verb {
            "rm" => {
                let p = PathBuf::from(rest);
                if rest.is_empty() || !p.is_absolute() {
                    return Err(format!("rm needs an absolute path, got {rest:?}"));
                }
                Ok(Self::Rm(p))
            }
            "run" => {
                let argv: Vec<String> = serde_json::from_str(rest)
                    .map_err(|e| format!("run needs a JSON array of strings: {e}"))?;
                if argv.is_empty() || argv[0].is_empty() {
                    return Err("run needs a non-empty argv".to_owned());
                }
                Ok(Self::Run(argv))
            }
            "note" => Ok(Self::Note(rest.to_owned())),
            _ => Err(format!(
                "unknown cleanup action {text:?}; use `rm <absolute path>`, `run <JSON argv>` or `note <text>`"
            )),
        }
    }
}

/// How a bounded child command ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    /// Exited with status 0.
    Ok,
    /// Could not start, exited non-zero, or was killed; the text says why.
    Failed(String),
    /// Ran past the timeout and was killed.
    TimedOut,
}

/// Runs `argv` with null stdio and kills it after `timeout`.
pub fn run_bounded(argv: &[String], timeout: Duration) -> RunOutcome {
    let Some((program, args)) = argv.split_first() else {
        return RunOutcome::Failed("empty argv".to_owned());
    };
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return RunOutcome::Failed(format!("{program}: {e}")),
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return RunOutcome::Ok,
            Ok(Some(status)) => return RunOutcome::Failed(format!("{program}: {status}")),
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return RunOutcome::TimedOut;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return RunOutcome::Failed(format!("{program}: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ProcessKind, ProcessRole, now_utc};

    fn rec(start: Option<&str>) -> ProcessRecord {
        ProcessRecord {
            pid: 1,
            start_time: start.map(str::to_owned),
            kind: ProcessKind::Harness,
            command: "x".into(),
            started_at: now_utc(),
            stopped_at: None,
            role: ProcessRole::Started,
            worker: None,
            cleanup: Vec::new(),
        }
    }

    #[test]
    fn identity_tolerates_two_seconds_and_refuses_the_unverifiable() {
        let r = rec(Some("2026-01-01T10:00:00Z"));
        assert!(identity_matches(&r, Some("2026-01-01T10:00:00Z")));
        assert!(identity_matches(&r, Some("2026-01-01T10:00:02Z")));
        assert!(identity_matches(&r, Some("2026-01-01T09:59:58Z")));
        assert!(!identity_matches(&r, Some("2026-01-01T10:00:03Z")));
        assert!(!identity_matches(&r, None));
        assert!(!identity_matches(&rec(None), Some("2026-01-01T10:00:00Z")));
        assert!(identity_matches(&rec(Some("boot+5")), Some("boot+5")));
        assert!(!identity_matches(&rec(Some("boot+5")), Some("boot+6")));
    }

    #[test]
    fn cleanup_grammar() {
        let abs = if cfg!(windows) { r"C:\tmp\x" } else { "/tmp/x" };
        assert_eq!(
            CleanupAction::parse(&format!("rm {abs}")).unwrap(),
            CleanupAction::Rm(abs.into())
        );
        assert!(CleanupAction::parse("rm relative/x").is_err());
        assert!(CleanupAction::parse("rm").is_err());
        assert_eq!(
            CleanupAction::parse(r#"run ["memray","detach","123"]"#).unwrap(),
            CleanupAction::Run(vec!["memray".into(), "detach".into(), "123".into()])
        );
        assert!(CleanupAction::parse("run memray detach").is_err());
        assert!(CleanupAction::parse("run []").is_err());
        assert_eq!(
            CleanupAction::parse("note left for the JVM").unwrap(),
            CleanupAction::Note("left for the JVM".into())
        );
        assert!(CleanupAction::parse("remove /tmp/x").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn bounded_runs_report_success_failure_and_timeout() {
        let a = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            run_bounded(&a(&["true"]), Duration::from_secs(5)),
            RunOutcome::Ok
        );
        assert!(matches!(
            run_bounded(&a(&["false"]), Duration::from_secs(5)),
            RunOutcome::Failed(_)
        ));
        assert!(matches!(
            run_bounded(&a(&["/nonexistent/x"]), Duration::from_secs(5)),
            RunOutcome::Failed(_)
        ));
        assert_eq!(
            run_bounded(&a(&["sleep", "5"]), Duration::from_millis(150)),
            RunOutcome::TimedOut
        );
    }

    #[cfg(unix)]
    #[test]
    fn kill_falls_back_to_the_shell_builtin_when_the_binary_cannot_start() {
        use std::os::unix::process::CommandExt;
        let mut child = Command::new("sleep")
            .arg("30")
            .process_group(0)
            .spawn()
            .unwrap();
        let pid = child.id();
        assert!(unix_kill_with(
            "dla-no-such-kill-binary",
            "KILL",
            &format!("-{pid}")
        ));
        let status = child.wait().unwrap();
        assert!(!status.success());
        // A binary that starts and fails (no such process) is the answer, not a fallback.
        assert!(!unix_kill_with("kill", "KILL", "-999999999"));
    }

    #[test]
    fn liveness_of_this_process() {
        assert!(is_alive(std::process::id()));
        assert!(!is_alive(0));
        assert!(!kill_group(0, "KILL"));
        assert!(!kill_group(5, "K ILL"));
    }
}
