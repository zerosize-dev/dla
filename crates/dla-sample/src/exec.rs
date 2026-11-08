//! Running helper programs with a deadline, and finding them on `PATH`.

use std::{
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

/// What a helper printed and how it ended.
#[derive(Debug)]
pub struct Run {
    /// Whether it exited with status 0.
    pub success: bool,
    /// Standard output (lossy UTF-8).
    pub stdout: String,
    /// Standard error (lossy UTF-8).
    pub stderr: String,
}

/// Runs `cmd` to completion, killing it when `timeout` passes. Standard input is closed.
pub fn run(mut cmd: Command, timeout: Duration) -> Result<Run, String> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let drain = |mut pipe: Box<dyn Read + Send>| {
        thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = pipe.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).into_owned()
        })
    };
    let out = child.stdout.take().map(|p| drain(Box::new(p)));
    let err = child.stderr.take().map(|p| drain(Box::new(p)));
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Ok(s),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(format!("timed out after {}s", timeout.as_secs()));
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(e) => break Err(e.to_string()),
        }
    }?;
    let join = |h: Option<thread::JoinHandle<String>>| h.and_then(|h| h.join().ok());
    Ok(Run {
        success: status.success(),
        stdout: join(out).unwrap_or_default(),
        stderr: join(err).unwrap_or_default(),
    })
}

/// The first existing `name` (with `.exe` on Windows) in `extra_dirs`, then on `PATH`.
pub fn find_exe(name: &str, extra_dirs: &[PathBuf]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let dirs = extra_dirs
        .iter()
        .cloned()
        .chain(std::env::split_paths(&path));
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    dirs.map(|d| d.join(&file)).find(|p| p.is_file())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn captures_output_and_enforces_the_deadline() {
        let mut c = Command::new("sh");
        c.args(["-c", "echo out; echo err >&2; exit 3"]);
        let r = run(c, Duration::from_secs(5)).unwrap();
        assert!(!r.success);
        assert_eq!((r.stdout.trim(), r.stderr.trim()), ("out", "err"));
        let mut c = Command::new("sleep");
        c.arg("5");
        let t = Instant::now();
        assert!(run(c, Duration::from_millis(100)).is_err());
        assert!(t.elapsed() < Duration::from_secs(3));
        assert!(find_exe("sh", &[]).is_some());
        assert!(find_exe("no-such-tool-xyz", &[]).is_none());
    }
}
