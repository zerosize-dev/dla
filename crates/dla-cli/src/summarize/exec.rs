//! Running an external tool for an adapter: output goes to a file under `tmp/<worker>/`,
//! is parsed by the adapter, and is never relayed on stdout.
#![allow(dead_code)] // used by the tool-backed adapters of the next phase

use std::{
    ffi::OsString,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use dla_core::Error;

use super::{Ctx, ToolReq};
use crate::fail::{Failure, Res};

/// Bytes of the tool's stderr kept for a message.
const STDERR_TAIL: u64 = 400;

/// How a tool run ended.
#[derive(Debug)]
pub struct ToolRun {
    /// The file the tool's stdout went to.
    pub out: PathBuf,
    /// True when it exited with status 0 in time.
    pub ok: bool,
    /// Exit code, when it exited.
    pub code: Option<i32>,
    /// True when it was killed at the timeout.
    pub timed_out: bool,
    /// The end of its stderr.
    pub stderr_tail: String,
}

/// Runs `program args` with stdout in the file `out` (stdin closed, its own process group,
/// killed after `timeout`). Stderr is kept in a side file, read for its tail and removed.
pub fn run_to_file(
    program: &Path,
    args: &[OsString],
    out: &Path,
    timeout: Duration,
) -> std::io::Result<ToolRun> {
    let err_path = out.with_extension("stderr");
    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(fs::File::create(out)?)
        .stderr(fs::File::create(&err_path)?);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn()?;
    let start = Instant::now();
    let (status, timed_out) = loop {
        if let Some(s) = child.try_wait()? {
            break (Some(s), false);
        }
        if start.elapsed() >= timeout {
            let _ = dla_core::proc::kill_group(child.id(), "KILL");
            let _ = child.kill();
            let _ = child.wait();
            break (None, true);
        }
        std::thread::sleep(Duration::from_millis(15));
    };
    let mut tail = String::new();
    if let Ok(mut f) = fs::File::open(&err_path) {
        let len = f.metadata().map_or(0, |m| m.len());
        if len > STDERR_TAIL {
            let _ = std::io::Seek::seek(&mut f, std::io::SeekFrom::Start(len - STDERR_TAIL));
        }
        let mut buf = Vec::new();
        let _ = f.read_to_end(&mut buf);
        tail = String::from_utf8_lossy(&buf).trim().to_owned();
    }
    let _ = fs::remove_file(&err_path);
    Ok(ToolRun {
        out: out.to_path_buf(),
        ok: status.is_some_and(|s| s.success()),
        code: status.and_then(|s| s.code()),
        timed_out,
        stderr_tail: tail,
    })
}

impl Ctx<'_> {
    /// Runs the adapter's tool: found in the tools directory, `JAVA_HOME/bin` or `PATH`,
    /// stdout into `tmp/<worker>/<out_name>`. A tool that cannot be found or started is a
    /// `tool-missing` refusal; the caller parses the output file and decides what a bad
    /// exit means.
    pub fn run_tool(
        &self,
        req: &ToolReq,
        args: &[OsString],
        out_name: &str,
        timeout: Duration,
    ) -> Res<ToolRun> {
        let tool = dla_probe::Backend::dynamic(req.binary).ok_or_else(|| {
            Failure::refuse(
                super::TOOL_MISSING,
                format!("{} is not a tool DLA knows", req.binary),
                "a tool in the probe registry",
                "report this as a dla bug",
            )
        })?;
        let tools = self.tc.tools_dir();
        let found = dla_probe::Backend::locate_dynamic(
            tool,
            tools.as_deref(),
            self.env.java_home.as_deref(),
            self.env.path.as_deref(),
        )
        .ok_or_else(|| {
            Failure::refuse(
                super::TOOL_MISSING,
                format!(
                    "{} was not found in the tools directory, JAVA_HOME or PATH",
                    req.binary
                ),
                format!("{} on this machine", req.binary),
                if req.install_hint.is_empty() {
                    tool.install_hint.to_owned()
                } else {
                    req.install_hint.to_owned()
                },
            )
        })?;
        let out = self.tc.tmp_dir()?.join(out_name);
        run_to_file(&found.path, args, &out, timeout)
            .map_err(|e| Failure::from(Error::io(&found.path, e)))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn sh(script: &str) -> Vec<OsString> {
        vec!["-c".into(), script.into()]
    }

    #[test]
    fn stdout_goes_to_the_file_and_stderr_to_the_tail() {
        let d = tempfile::tempdir().unwrap();
        let out = d.path().join("o.txt");
        let r = run_to_file(
            Path::new("/bin/sh"),
            &sh("echo hello; echo oops >&2; exit 3"),
            &out,
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(!r.ok && !r.timed_out);
        assert_eq!(r.code, Some(3));
        assert_eq!(fs::read_to_string(&out).unwrap().trim(), "hello");
        assert_eq!(r.stderr_tail, "oops");
        assert!(!out.with_extension("stderr").exists());
    }

    #[test]
    fn a_tool_that_hangs_is_killed_at_the_timeout() {
        let d = tempfile::tempdir().unwrap();
        let out = d.path().join("o.txt");
        let t = Instant::now();
        let r = run_to_file(
            Path::new("/bin/sh"),
            &sh("sleep 30"),
            &out,
            Duration::from_millis(200),
        )
        .unwrap();
        assert!(r.timed_out && !r.ok);
        assert!(t.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_missing_program_is_an_io_error() {
        let d = tempfile::tempdir().unwrap();
        assert!(
            run_to_file(
                Path::new("/nonexistent/tool"),
                &[],
                &d.path().join("o"),
                Duration::from_secs(1)
            )
            .is_err()
        );
    }
}
