//! `heaptrack --pid`: record the allocations of a running native process for a while.
//!
//! The wrapper script injects `libheaptrack_inject` with gdb and then waits for its pipeline
//! to end. DLA ends it the controlled way: after `--duration` it calls `heaptrack_stop()`
//! through gdb, then waits for the script to finish on its own. The script is never killed
//! (a signal skips its cleanup and leaves an empty recording and the FIFO).
//!
//! The script's own EXIT cleanup calls `heaptrack_stop()` a second time whenever the process
//! still exists, and the second call aborts the target (SIGABRT in `pthread_kill`, measured).
//! So the script runs with a `gdb` shim first on its `PATH`: the shim passes every call to the
//! real gdb, except that a `heaptrack_stop` call after DLA's own stop is a no-op. DLA's stop
//! goes through the shim too (`--dla-stop`), which writes the marker that makes the later
//! call a no-op. The marker is written before the stop, because the script's cleanup starts
//! the moment the recording ends while gdb is still returning; a failed stop removes it.

use std::{
    fs::File,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde_json::{Map, json};

use super::{
    Backend, Detail, Done, Plan, Request, Tracker,
    parse::parse_heaptrack_print,
    resources::Estimate,
    util::{AttachError, Cleanup, clip, output_name, same_process},
};
use crate::exec;

const MIB: u64 = 1 << 20;
const INJECT_WAIT: Duration = Duration::from_secs(90);
const FINISH_WAIT: Duration = Duration::from_secs(120);

pub(crate) struct Prep {
    heaptrack: PathBuf,
    print: PathBuf,
    gdb: PathBuf,
}

/// The `gdb` shim script (see the module documentation). `real_gdb` runs every call; the
/// marker file exists once DLA stopped the recording.
pub fn shim_script(real_gdb: &Path, marker: &Path) -> String {
    let q = |p: &Path| format!("'{}'", p.display().to_string().replace('\'', "'\\''"));
    format!(
        "#!/bin/sh\n\
REAL={real}\n\
MARK={mark}\n\
if [ \"$1\" = \"--dla-stop\" ]; then\n\
  : > \"$MARK\"\n\
  \"$REAL\" --batch-silent -n -iex=\"set auto-solib-add off\" -iex=\"set language c\" -p \"$2\" \\\n\
    --eval-command=\"sharedlibrary libheaptrack_inject\" \\\n\
    --eval-command=\"call (void) heaptrack_stop()\" --eval-command=\"detach\"\n\
  rc=$?\n\
  [ \"$rc\" -eq 0 ] || rm -f \"$MARK\"\n\
  exit \"$rc\"\n\
fi\n\
case \"$*\" in\n\
  *heaptrack_stop*) [ -e \"$MARK\" ] && exit 0 ;;\n\
esac\n\
exec \"$REAL\" \"$@\"\n",
        real = q(real_gdb),
        mark = q(marker)
    )
}

/// The FIFO the wrapper script with process ID `script_pid` creates.
pub fn fifo_path(script_pid: u32) -> PathBuf {
    PathBuf::from(format!("/tmp/heaptrack_fifo{script_pid}"))
}

pub(crate) fn plan(req: &Request) -> Result<Plan, AttachError> {
    let pid = req.target.pid;
    let find = |n: &str| exec::find_exe(n, &req.tool_dirs);
    let (Some(heaptrack), Some(print)) = (find("heaptrack"), find("heaptrack_print")) else {
        return Err(AttachError::tool_missing(
            "heaptrack or heaptrack_print is not installed",
            "heaptrack and heaptrack_print on PATH",
            "apt-get install heaptrack gdb   (Linux only)",
        ));
    };
    let Some(gdb) = find("gdb") else {
        return Err(AttachError::tool_missing(
            "heaptrack --pid injects through gdb, which is not installed",
            "gdb on PATH",
            "apt-get install gdb",
        ));
    };
    if let Ok(scope) = std::fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope")
        && scope.trim() != "0"
    {
        return Err(AttachError::not_ready(
            format!(
                "kernel.yama.ptrace_scope is {}, and heaptrack --pid refuses to attach unless it is 0",
                scope.trim()
            ),
            "kernel.yama.ptrace_scope = 0",
            "echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope   (then repeat the command)",
        ));
    }
    // heaptrack interposes the dynamic allocator: a statically linked target records nothing.
    if let Ok(maps) = std::fs::read_to_string(format!("/proc/{pid}/maps"))
        && !maps.is_empty()
        && !maps.contains("libc.so")
        && !maps.contains("libc-")
        && !maps.contains("ld-musl")
    {
        return Err(AttachError::not_ready(
            format!(
                "process {pid} maps no shared C library, so there is no malloc for heaptrack to hook"
            ),
            "a dynamically linked native process",
            "use the runtime's own tool (jcmd, node-inspector, memray, goref) instead",
        ));
    }
    let secs = req.duration.as_secs().max(1);
    let file = match &req.out_name {
        Some(n) => output_name(n, "gz").map_err(|m| {
            AttachError::not_ready(
                m,
                "--out as a plain file name",
                "dla attach ... --out run1.gz",
            )
        })?,
        None => "heaptrack.gz".to_owned(),
    };
    Ok(Plan {
        backend: Backend::Heaptrack,
        op: req.op,
        tool: format!("heaptrack {}", heaptrack.display()),
        expected_signal: "a heaptrack recording (.gz) and a heaptrack_print summary of the peak consumers with call stacks".into(),
        stopping: format!("after {secs}s (--duration) DLA calls heaptrack_stop() through gdb, then waits for the heaptrack script to finish"),
        effects: vec![
            format!("gdb attaches to {pid} with ptrace and injects libheaptrack_inject; the target stops for a moment, and every malloc and free is traced afterwards, which slows it"),
            "heaptrack runs in a directory under the run directory; it creates the FIFO /tmp/heaptrack_fifo<script pid>".to_owned(),
            "the injected library stays mapped in the target after the recording".into(),
        ],
        cleanup: vec![
            "rm /tmp/heaptrack_fifo<script pid> if the script leaves it".into(),
            "run the gdb heaptrack_stop call if the attach is aborted".into(),
            format!("note libheaptrack_inject stays mapped in process {pid} until it exits"),
        ],
        estimate: Estimate {
            dump_bytes: 8 * MIB + secs * MIB,
            peak_bytes: 256 * MIB,
            basis: format!("no pre-estimate exists: 8 MiB plus 1 MiB per second of --duration ({secs}s) assumed; the interpreter process buffers the stream"),
        },
        notes: vec![format!("gdb {}", gdb.display())],
        unverified: false,
        file,
        detail: Detail::Heaptrack(Prep { heaptrack, print, gdb }),
    })
}

fn log_tail(path: &Path) -> String {
    let t = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = t
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("No such file or directory"))
        .collect();
    clip(&lines[lines.len().saturating_sub(3)..].join(" | "), 240)
}

fn wait_child(child: &mut Child, max: Duration) -> bool {
    let end = Instant::now() + max;
    while Instant::now() < end {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

/// The recording the script wrote: `heaptrack.<comm>.<n>.gz` (or `.zst`) in `dir`.
pub fn find_recording(dir: &Path) -> Option<PathBuf> {
    let mut hits: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.starts_with("heaptrack.") && (n.ends_with(".gz") || n.ends_with(".zst"))
        })
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    hits.sort();
    hits.pop().map(|(_, p)| p)
}

pub(crate) fn run(
    p: &Prep,
    plan: &Plan,
    req: &Request,
    out_dir: &Path,
    t: &mut Tracker<'_>,
) -> Result<Done, AttachError> {
    let pid = req.target.pid;
    let secs = req.duration.as_secs().max(1);
    let log_path = out_dir.join("heaptrack.log");
    let log = File::create(&log_path).map_err(|e| {
        AttachError::failed(
            format!("cannot create {}: {e}", log_path.display()),
            "check free disk",
        )
    })?;
    let log2 = log
        .try_clone()
        .map_err(|e| AttachError::failed(e.to_string(), "repeat the command"))?;
    // The gdb shim that keeps the script's own cleanup from stopping the recording twice.
    let shim_dir = out_dir.join(".shim");
    let marker = shim_dir.join("stopped");
    let shim = shim_dir.join("gdb");
    let write_shim = || -> std::io::Result<()> {
        std::fs::create_dir_all(&shim_dir)?;
        std::fs::write(&shim, shim_script(&p.gdb, &marker))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    };
    write_shim().map_err(|e| {
        AttachError::failed(
            format!("cannot write the gdb shim in {}: {e}", shim_dir.display()),
            "check free disk",
        )
    })?;
    let stop = Cleanup::Run(vec![
        shim.display().to_string(),
        "--dla-stop".into(),
        pid.to_string(),
    ]);
    let path_env = {
        let mut dirs = vec![shim_dir.clone()];
        dirs.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        std::env::join_paths(dirs).unwrap_or_default()
    };
    t.note(format!(
        "libheaptrack_inject stays mapped in process {pid} until it exits"
    ));
    let started = Instant::now();
    let mut child = Command::new(&p.heaptrack)
        .args(["--pid", &pid.to_string()])
        .current_dir(out_dir)
        .env("PATH", &path_env)
        .stdin(Stdio::null())
        .stdout(log)
        .stderr(log2)
        .spawn()
        .map_err(|e| {
            AttachError::failed(
                format!("cannot start heaptrack: {e}"),
                "check heaptrack is executable",
            )
        })?;
    let fifo = fifo_path(child.id());
    t.need(Cleanup::Rm(fifo.clone()));
    t.need(stop.clone());

    // Wait for the injection to finish.
    let injected = loop {
        let text = std::fs::read_to_string(&log_path).unwrap_or_default();
        if text.contains("injection finished") {
            break true;
        }
        if matches!(child.try_wait(), Ok(Some(_))) || started.elapsed() > INJECT_WAIT {
            break false;
        }
        thread::sleep(Duration::from_millis(100));
    };
    if !injected {
        let tail = log_tail(&log_path);
        let exited = matches!(child.try_wait(), Ok(Some(_)));
        if !exited && same_process(&req.target) {
            // The script is still running: stop the recording the controlled way.
            let stopped =
                exec::run(to_command(&stop), Duration::from_secs(30)).is_ok_and(|r| r.success);
            if stopped {
                t.keep(&stop);
            }
            wait_child(&mut child, Duration::from_secs(30));
        }
        let _ = std::fs::remove_file(&fifo);
        return Err(AttachError::failed(
            format!("heaptrack did not finish injecting: {tail}"),
            "check gdb may attach to the process (same user, or root; ptrace_scope 0)",
        ));
    }
    let inject_ms = started.elapsed().as_millis() as u64;

    // Record for --duration.
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return Err(AttachError::failed(
                format!("heaptrack ended before --duration: {}", log_tail(&log_path)),
                "repeat the command",
            ));
        }
        if !same_process(&req.target) {
            t.forget_runs();
            wait_child(&mut child, Duration::from_secs(20));
            return Err(AttachError::failed(
                format!(
                    "process {pid} exited (or its pid was reused) while heaptrack was recording"
                ),
                "repeat the command on a running process",
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Stop: heaptrack_stop() through gdb, then let the script finish by itself.
    if !same_process(&req.target) {
        t.forget_runs();
        return Err(AttachError::failed(
            format!("process {pid} exited (or its pid was reused) before heaptrack was stopped"),
            "repeat the command on a running process",
        ));
    }
    let t_stop = Instant::now();
    let r = exec::run(to_command(&stop), Duration::from_secs(60)).map_err(|e| {
        AttachError::failed(
            format!("gdb heaptrack_stop: {e}"),
            "run the stop command from the process registry by hand",
        )
    })?;
    if !r.success {
        return Err(AttachError::failed(
            format!("gdb heaptrack_stop failed: {}", clip(r.stderr.trim(), 160)),
            "run: dla run abort   to clean up",
        ));
    }
    t.keep(&stop);
    let finished = wait_child(&mut child, FINISH_WAIT);
    if !finished {
        return Err(AttachError::failed(
            "the heaptrack script did not finish after the recording was stopped (it is left running, not killed)",
            "wait for it to exit, then repeat; killing it can SIGPIPE the target",
        ));
    }
    let stop_ms = t_stop.elapsed().as_millis() as u64;
    if fifo.exists() {
        if std::fs::remove_file(&fifo).is_ok() {
            t.done(
                &Cleanup::Rm(fifo.clone()),
                format!("removed {}", fifo.display()),
            );
        }
    } else {
        t.keep(&Cleanup::Rm(fifo.clone()));
    }

    // Name the recording as asked and print it.
    let Some(raw) = find_recording(out_dir) else {
        return Err(AttachError::failed(
            format!("heaptrack wrote no recording: {}", log_tail(&log_path)),
            "repeat the command",
        ));
    };
    let target_path = out_dir.join(&plan.file);
    if raw != target_path {
        std::fs::rename(&raw, &target_path).map_err(|e| {
            AttachError::failed(
                format!("cannot rename the recording: {e}"),
                "check the attach directory",
            )
        })?;
    }
    let bytes = std::fs::metadata(&target_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let mut done = Done::default();
    done.files.push(plan.file.clone());
    let mut summary = Map::new();
    summary.insert("recording_bytes".into(), json!(bytes));
    let mut c = Command::new(&p.print);
    c.args(["-n", "10"]).arg(&target_path);
    match exec::run(c, Duration::from_secs(180)) {
        Ok(r) if r.success => {
            let txt_name = format!(
                "{}-print.txt",
                plan.file.split('.').next().unwrap_or("heaptrack")
            );
            if std::fs::write(out_dir.join(&txt_name), &r.stdout).is_ok() {
                done.files.push(txt_name);
            }
            let s = parse_heaptrack_print(&r.stdout, 5);
            summary.insert("allocation_calls".into(), json!(s.allocation_calls));
            summary.insert("peak_heap_bytes".into(), json!(s.peak_heap_bytes));
            summary.insert("leaked_bytes".into(), json!(s.leaked_bytes));
            summary.insert("runtime_s".into(), json!(s.runtime_s));
            summary.insert(
                "peak_consumers".into(),
                json!(
                    s.peak_consumers
                        .iter()
                        .map(|c| json!({
                            "bytes": c.bytes, "calls": c.calls,
                            "function": clip(&c.function, 60),
                            "at": c.location.as_deref().map(|l| clip(l, 100)),
                        }))
                        .collect::<Vec<_>>()
                ),
            );
        }
        Ok(r) => done.limitations.push(format!(
            "heaptrack_print failed: {}",
            clip(r.stderr.trim(), 120)
        )),
        Err(e) => done
            .limitations
            .push(format!("heaptrack_print failed: {e}")),
    }
    done.files.push("heaptrack.log".to_owned());
    if std::fs::remove_dir_all(&shim_dir).is_ok() {
        t.did("removed the gdb shim");
    }
    done.limitations.push("only allocations made after the attach are recorded; source lines need debug symbols in the target".into());
    done.summary = serde_json::Value::Object(summary);
    done.durations.insert("inject_ms".into(), json!(inject_ms));
    done.durations.insert("stop_ms".into(), json!(stop_ms));
    Ok(done)
}

fn to_command(c: &Cleanup) -> Command {
    let Cleanup::Run(argv) = c else {
        return Command::new("true");
    };
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attach::util::Cleanup as C;

    #[test]
    fn the_shim_stops_once_and_makes_the_scripts_second_stop_a_no_op() {
        let text = shim_script(Path::new("/usr/bin/gdb"), Path::new("/run/it's/stopped"));
        assert!(text.starts_with("#!/bin/sh\n"));
        assert!(text.contains("REAL='/usr/bin/gdb'"));
        // A quote in a path cannot end the quoting.
        assert!(text.contains("MARK='/run/it'\\''s/stopped'"), "{text}");
        assert!(text.contains("call (void) heaptrack_stop()"));
        assert!(text.contains("sharedlibrary libheaptrack_inject"));
        assert!(text.contains(": > \"$MARK\""));
        assert!(text.contains("*heaptrack_stop*) [ -e \"$MARK\" ] && exit 0"));
        assert!(text.contains("exec \"$REAL\" \"$@\""));
        // The abort action survives the registry grammar.
        let c = C::Run(vec![
            "/x/.shim/gdb".into(),
            "--dla-stop".into(),
            "42".into(),
        ]);
        assert_eq!(C::parse(&c.render()), Some(c));
    }

    #[test]
    fn the_fifo_follows_the_script_pid() {
        assert_eq!(fifo_path(77), PathBuf::from("/tmp/heaptrack_fifo77"));
        assert!(C::parse(&C::Rm(fifo_path(77)).render()).is_some());
    }

    #[test]
    fn the_newest_recording_is_found() {
        let d = std::env::temp_dir().join(format!("dla-ht-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        assert!(find_recording(&d).is_none());
        std::fs::write(d.join("heaptrack.leak.12.gz"), b"x").unwrap();
        std::fs::write(d.join("notes.gz"), b"x").unwrap();
        assert_eq!(find_recording(&d), Some(d.join("heaptrack.leak.12.gz")));
        std::fs::remove_dir_all(&d).unwrap();
    }
}
