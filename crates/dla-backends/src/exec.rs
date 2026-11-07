//! Running external tools: bounded output, a hard timeout, no inherited stdin.

use std::{
    ffi::OsString,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

/// Bytes kept per output stream; anything beyond is read and dropped.
pub const MAX_CAPTURE: usize = 64 * 1024 * 1024;

/// A command to run.
#[derive(Debug, Clone)]
pub struct Cmd {
    /// Program path or name.
    pub program: OsString,
    /// Arguments.
    pub args: Vec<OsString>,
    /// Working directory.
    pub cwd: Option<PathBuf>,
    /// Environment variables to set.
    pub env: Vec<(OsString, OsString)>,
    /// Environment variables to remove.
    pub env_remove: Vec<OsString>,
    /// Hard limit.
    pub timeout: Duration,
}

impl Cmd {
    /// A command with a five-minute timeout.
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: Vec::new(),
            env_remove: Vec::new(),
            timeout: Duration::from_secs(300),
        }
    }

    /// Adds one argument.
    pub fn arg(mut self, a: impl Into<OsString>) -> Self {
        self.args.push(a.into());
        self
    }

    /// Adds several arguments.
    pub fn args<I, S>(mut self, a: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(a.into_iter().map(Into::into));
        self
    }

    /// Sets the working directory.
    pub fn cwd(mut self, d: impl Into<PathBuf>) -> Self {
        self.cwd = Some(d.into());
        self
    }

    /// Sets one environment variable.
    pub fn env(mut self, k: impl Into<OsString>, v: impl Into<OsString>) -> Self {
        self.env.push((k.into(), v.into()));
        self
    }

    /// Removes one environment variable.
    pub fn env_remove(mut self, k: impl Into<OsString>) -> Self {
        self.env_remove.push(k.into());
        self
    }

    /// Sets the timeout.
    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = t;
        self
    }

    /// The command as one list of strings (program first), for the raw record.
    pub fn display(&self) -> Vec<String> {
        std::iter::once(&self.program)
            .chain(self.args.iter())
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    /// The value of an environment variable set on this command.
    pub fn env_value(&self, key: &str) -> Option<String> {
        self.env
            .iter()
            .rev()
            .find(|(k, _)| k.to_string_lossy() == key)
            .map(|(_, v)| v.to_string_lossy().into_owned())
    }
}

/// What a run produced.
#[derive(Debug, Clone, Default)]
pub struct Output {
    /// Exit code; `None` when the process did not exit normally.
    pub exit: Option<i32>,
    /// Captured standard output (lossy UTF-8).
    pub stdout: String,
    /// Captured standard error (lossy UTF-8).
    pub stderr: String,
    /// The timeout fired and the process was killed.
    pub timed_out: bool,
    /// A stream exceeded [`MAX_CAPTURE`].
    pub truncated: bool,
    /// The process could not be started.
    pub spawn_error: Option<String>,
    /// Wall time.
    pub millis: u64,
}

impl Output {
    /// The last `n` lines of stderr, or of stdout when stderr is empty.
    pub fn tail(&self, n: usize) -> String {
        let src = if self.stderr.trim().is_empty() {
            &self.stdout
        } else {
            &self.stderr
        };
        let lines: Vec<&str> = src.lines().filter(|l| !l.trim().is_empty()).collect();
        let from = lines.len().saturating_sub(n);
        lines[from..].join("\n")
    }

    /// A short reason this run counts as failed, or `None`.
    pub fn failure_reason(&self) -> Option<String> {
        if let Some(e) = &self.spawn_error {
            return Some(format!("could not start: {e}"));
        }
        if self.timed_out {
            return Some("timed out".to_owned());
        }
        if self.truncated {
            return Some("output exceeded the capture limit".to_owned());
        }
        None
    }
}

struct Capture {
    data: Vec<u8>,
    truncated: bool,
}

fn drain(mut r: impl Read + Send + 'static) -> thread::JoinHandle<Capture> {
    thread::spawn(move || {
        let mut data = Vec::new();
        let mut truncated = false;
        let mut buf = [0u8; 16 * 1024];
        loop {
            match r.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if data.len() < MAX_CAPTURE {
                        let take = n.min(MAX_CAPTURE - data.len());
                        data.extend_from_slice(&buf[..take]);
                        truncated |= take < n;
                    } else {
                        truncated = true;
                    }
                }
            }
        }
        Capture { data, truncated }
    })
}

fn collect(h: Option<thread::JoinHandle<Capture>>) -> Capture {
    let Some(h) = h else {
        return Capture {
            data: Vec::new(),
            truncated: false,
        };
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    while !h.is_finished() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    if h.is_finished() {
        h.join().unwrap_or(Capture {
            data: Vec::new(),
            truncated: false,
        })
    } else {
        // A grandchild still holds the pipe; give up on the rest of the stream.
        Capture {
            data: Vec::new(),
            truncated: true,
        }
    }
}

#[cfg(unix)]
fn kill_tree(child: &mut std::process::Child) {
    let pid = child.id();
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

#[cfg(not(unix))]
fn kill_tree(child: &mut std::process::Child) {
    let _ = Command::new("taskkill")
        .args(["/F", "/T", "/PID", &child.id().to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

/// Runs `cmd` to completion or until its timeout, capturing both streams.
pub fn run(cmd: &Cmd) -> Output {
    let started = Instant::now();
    let mut c = Command::new(&cmd.program);
    c.args(&cmd.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(d) = &cmd.cwd {
        c.current_dir(d);
    }
    for k in &cmd.env_remove {
        c.env_remove(k);
    }
    for (k, v) in &cmd.env {
        c.env(k, v);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    let mut child = match c.spawn() {
        Ok(ch) => ch,
        Err(e) => {
            return Output {
                spawn_error: Some(e.to_string()),
                millis: millis(started),
                ..Output::default()
            };
        }
    };
    let out_h = child.stdout.take().map(drain);
    let err_h = child.stderr.take().map(drain);
    let deadline = started + cmd.timeout;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) => {
                if Instant::now() >= deadline {
                    timed_out = true;
                    kill_tree(&mut child);
                    break child.wait().ok();
                }
                thread::sleep(Duration::from_millis(15));
            }
            Err(_) => break None,
        }
    };
    let so = collect(out_h);
    let se = collect(err_h);
    Output {
        exit: status.and_then(|s| s.code()),
        stdout: String::from_utf8_lossy(&so.data).into_owned(),
        stderr: String::from_utf8_lossy(&se.data).into_owned(),
        timed_out,
        truncated: so.truncated || se.truncated,
        spawn_error: None,
        millis: millis(started),
    }
}

fn millis(t: Instant) -> u64 {
    u64::try_from(t.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn captures_both_streams_and_the_exit_code() {
        let o = run(&Cmd::new("sh").args(["-c", "echo out; echo err >&2; exit 3"]));
        assert_eq!(o.exit, Some(3));
        assert_eq!(o.stdout.trim(), "out");
        assert_eq!(o.stderr.trim(), "err");
        assert!(!o.timed_out && o.failure_reason().is_none());
    }

    #[test]
    fn a_timeout_kills_the_process_tree() {
        let o = run(&Cmd::new("sh")
            .args(["-c", "sleep 30 & sleep 30"])
            .timeout(Duration::from_millis(200)));
        assert!(o.timed_out);
        assert!(o.millis < 5000, "took {} ms", o.millis);
        assert_eq!(o.failure_reason().as_deref(), Some("timed out"));
    }

    #[test]
    fn a_missing_program_is_a_spawn_error() {
        let o = run(&Cmd::new("definitely-not-a-program-dla"));
        assert!(o.spawn_error.is_some());
        assert!(o.failure_reason().unwrap().starts_with("could not start"));
    }

    #[test]
    fn env_is_set_and_removed() {
        let o = run(&Cmd::new("sh")
            .args(["-c", "echo \"$A-${B:-unset}\""])
            .env("A", "1")
            .env_remove("B"));
        assert_eq!(o.stdout.trim(), "1-unset");
    }

    #[test]
    fn tail_prefers_stderr() {
        let o = Output {
            stdout: "a\nb".into(),
            stderr: "x\n\ny\nz".into(),
            ..Output::default()
        };
        assert_eq!(o.tail(2), "y\nz");
    }
}
