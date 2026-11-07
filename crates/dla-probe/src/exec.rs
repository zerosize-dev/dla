//! Running child processes with a timeout and bounded combined output.

use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Most output bytes kept (the tail is kept).
const MAX_OUTPUT: usize = 256 * 1024;

/// What a child process did.
#[derive(Debug, Clone, Default)]
pub struct Output {
    /// Exit code, when the process ended normally.
    pub code: Option<i32>,
    /// True when the exit status was success.
    pub success: bool,
    /// True when the timeout killed the process.
    pub timed_out: bool,
    /// Combined stdout and stderr (lossy UTF-8, tail-bounded).
    pub text: String,
    /// Why the process could not start.
    pub spawn_error: Option<String>,
}

/// Runs `program args` in `cwd` with extra `env`, killing it (and, on Unix, its
/// process group) after `timeout`.
pub fn run(
    program: &Path,
    args: &[&str],
    cwd: Option<&Path>,
    env: &[(&str, &str)],
    timeout: Duration,
) -> Output {
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(d) = cwd {
        cmd.current_dir(d);
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Output {
                spawn_error: Some(e.to_string()),
                ..Output::default()
            };
        }
    };
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    if let Some(mut o) = child.stdout.take() {
        let tx = tx.clone();
        thread::spawn(move || pump(&mut o, &tx));
    }
    if let Some(mut e) = child.stderr.take() {
        let tx = tx.clone();
        thread::spawn(move || pump(&mut e, &tx));
    }
    drop(tx);

    let start = Instant::now();
    let mut buf: Vec<u8> = Vec::new();
    let mut status = None;
    let mut exited_at: Option<Instant> = None;
    let mut killed_at: Option<Instant> = None;
    let mut timed_out = false;
    loop {
        match rx.recv_timeout(Duration::from_millis(20)) {
            Ok(chunk) => {
                buf.extend_from_slice(&chunk);
                if buf.len() > MAX_OUTPUT * 2 {
                    buf.drain(..buf.len() - MAX_OUTPUT);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if status.is_none() {
            if let Ok(Some(s)) = child.try_wait() {
                status = Some(s);
                exited_at = Some(Instant::now());
            } else if killed_at.is_none() && start.elapsed() >= timeout {
                timed_out = true;
                kill_tree(&mut child);
                killed_at = Some(Instant::now());
            }
        }
        // Pipes held open by stray descendants must not hang us.
        let linger = exited_at.or(killed_at);
        if let Some(t) = linger
            && t.elapsed() > Duration::from_millis(1500)
        {
            break;
        }
    }
    let status = status.or_else(|| child.wait().ok());
    if buf.len() > MAX_OUTPUT {
        buf.drain(..buf.len() - MAX_OUTPUT);
    }
    Output {
        code: status.and_then(|s| s.code()),
        success: !timed_out && status.is_some_and(|s| s.success()),
        timed_out,
        text: String::from_utf8_lossy(&buf).into_owned(),
        spawn_error: None,
    }
}

fn pump(r: &mut impl Read, tx: &mpsc::Sender<Vec<u8>>) {
    let mut chunk = [0u8; 8192];
    loop {
        match r.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if tx.send(chunk[..n].to_vec()).is_err() {
                    break;
                }
            }
        }
    }
}

fn kill_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

/// The last `n` non-empty lines of `text`, joined, each shortened to 300 chars.
pub fn tail_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(n);
    lines[start..]
        .iter()
        .map(|l| shorten(l, 300))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `s` cut to `max` chars with an ellipsis.
pub fn shorten(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(max).collect();
        t.push_str("...");
        t
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn captures_both_streams_and_the_exit_code() {
        let o = run(
            Path::new("sh"),
            &["-c", "echo out; echo err 1>&2; exit 3"],
            None,
            &[],
            Duration::from_secs(10),
        );
        assert_eq!(o.code, Some(3));
        assert!(!o.success && !o.timed_out);
        assert!(o.text.contains("out") && o.text.contains("err"));
    }

    #[test]
    fn a_missing_program_is_a_spawn_error() {
        let o = run(
            Path::new("definitely-not-a-program-xyz"),
            &[],
            None,
            &[],
            Duration::from_secs(1),
        );
        assert!(o.spawn_error.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn the_timeout_kills_the_whole_process_group() {
        let t = Instant::now();
        let o = run(
            Path::new("sh"),
            &["-c", "sleep 30 & sleep 30"],
            None,
            &[],
            Duration::from_millis(300),
        );
        assert!(o.timed_out && !o.success);
        assert!(t.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn tail_keeps_the_last_lines() {
        assert_eq!(tail_lines("a\n\nb\nc\nd", 2), "c\nd");
    }
}
