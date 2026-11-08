//! `memray attach`: record the allocations of a running CPython process for a while.

use std::{
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

use serde_json::{Map, json};

use super::{
    Backend, Detail, Done, Plan, Request, Tracker,
    parse::parse_memray_stats,
    resources::Estimate,
    util::{AttachError, Cleanup, clip, first_error_line, output_name, proc_environ, wait_stable},
};
use crate::exec;

const MIB: u64 = 1 << 20;

pub(crate) struct Prep {
    /// The command that runs memray (`memray`, or `python3 -m memray`).
    cmd: Vec<String>,
    /// Environment the tool needs to find memray (`PYTHONPATH` of the target).
    env: Vec<(String, String)>,
    /// Whether `memray detach` exists and can be run without the extra environment.
    detach: bool,
    method: &'static str,
}

/// `(major, minor, patch, memray version)` printed by the import check.
pub fn parse_python_check(out: &str) -> Option<((u32, u32, u32), String)> {
    let mut it = out.split_whitespace();
    let v = (
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    );
    Some((v, it.next().unwrap_or("unknown").to_owned()))
}

/// The attach method memray's `auto` picks: `sys.remote_exec` from Python 3.14, a debugger
/// before.
pub fn method_for(version: (u32, u32, u32)) -> &'static str {
    if version >= (3, 14, 0) {
        "sys.remote_exec"
    } else {
        "gdb"
    }
}

fn interpreter_candidates(req: &Request) -> Vec<PathBuf> {
    let mut c = Vec::new();
    if let Some(a0) = req.target.args.first() {
        let p = PathBuf::from(a0);
        if p.is_absolute() && p.is_file() {
            c.push(p);
        } else if !a0.contains('/')
            && let Some(found) = exec::find_exe(a0, &[])
        {
            c.push(found);
        }
    }
    c.push(PathBuf::from(format!("/proc/{}/exe", req.target.pid)));
    c.dedup();
    c
}

pub(crate) fn plan(req: &Request) -> Result<Plan, AttachError> {
    let pid = req.target.pid;
    let name = req.target.exe_name();
    if !name.starts_with("python") {
        return Err(AttachError::not_ready(
            format!(
                "process {pid} runs {name:?}, not a CPython interpreter; memray attaches to CPython only"
            ),
            "a CPython process",
            "use dla attach heaptrack for native processes, or jcmd / node-inspector / goref for their runtimes",
        ));
    }
    let env_of_target = proc_environ(pid);
    let mut env = Vec::new();
    if let Some(pp) = env_of_target.get("PYTHONPATH") {
        env.push(("PYTHONPATH".to_owned(), pp.clone()));
    }
    // 1. memray must be importable in the TARGET's interpreter: the injected code runs
    //    `import memray` there.
    let check = "import sys, memray; print(*sys.version_info[:3], memray.__version__)";
    let mut last = String::new();
    let mut found = None;
    for cand in interpreter_candidates(req) {
        let mut c = Command::new(&cand);
        c.args(["-c", check]);
        for (k, v) in &env {
            c.env(k, v);
        }
        match exec::run(c, Duration::from_secs(20)) {
            Ok(r) if r.success => {
                if let Some(x) = parse_python_check(&r.stdout) {
                    found = Some((cand, x));
                    break;
                }
                last = format!("unexpected output {:?}", clip(r.stdout.trim(), 60));
            }
            Ok(r) => last = first_error_line("", r.stderr.lines().last().unwrap_or("")),
            Err(e) => last = e,
        }
    }
    let Some((interp, (ver, memray_ver))) = found else {
        return Err(AttachError::not_ready(
            format!("memray cannot be imported by the interpreter of process {pid} ({last})"),
            "memray importable in the target's Python environment",
            format!(
                "install it there (for example: {} -m pip install memray), restart nothing: the target only needs the package on its sys.path; then repeat the command",
                interp_hint(req)
            ),
        ));
    };
    let method = method_for(ver);
    if method == "sys.remote_exec"
        && env_of_target
            .get("PYTHON_DISABLE_REMOTE_DEBUG")
            .is_some_and(|v| !v.is_empty() && v != "0")
    {
        return Err(AttachError::not_ready(
            format!(
                "process {pid} runs with PYTHON_DISABLE_REMOTE_DEBUG set, so sys.remote_exec is off"
            ),
            "a Python 3.14 process with remote debugging enabled",
            "restart the target without PYTHON_DISABLE_REMOTE_DEBUG, then repeat the command",
        ));
    }
    if method == "gdb"
        && exec::find_exe("gdb", &[]).is_none()
        && exec::find_exe("lldb", &[]).is_none()
    {
        return Err(AttachError::tool_missing(
            format!(
                "Python {}.{}.{} needs gdb or lldb to inject memray, and neither is installed",
                ver.0, ver.1, ver.2
            ),
            "gdb or lldb on PATH",
            "install gdb (apt-get install gdb), or run the target on Python 3.14 or later",
        ));
    }
    // 2. The memray command itself.
    let mut extra = req.tool_dirs.clone();
    if let Some(parent) = interp.parent().filter(|_| interp.is_absolute()) {
        extra.push(parent.to_path_buf());
    }
    let (cmd, runs_alone) = if let Some(m) = exec::find_exe("memray", &extra) {
        (vec![m.display().to_string()], true)
    } else if let Some(py) = exec::find_exe("python3", &[]) {
        (
            vec![
                py.display().to_string(),
                "-m".to_owned(),
                "memray".to_owned(),
            ],
            false,
        )
    } else {
        return Err(AttachError::tool_missing(
            "the memray command is not installed",
            "memray on PATH",
            "pip install memray",
        ));
    };
    // The tool must see memray too: try it (also tells whether `detach` exists).
    let mut c = Command::new(&cmd[0]);
    c.args(&cmd[1..]).arg("--help");
    for (k, v) in &env {
        c.env(k, v);
    }
    let help = exec::run(c, Duration::from_secs(20)).map_err(|e| {
        AttachError::tool_missing(
            format!("memray did not start: {e}"),
            "a working memray command",
            "pip install memray",
        )
    })?;
    if !help.success {
        return Err(AttachError::tool_missing(
            format!(
                "memray did not start: {}",
                clip(&first_error_line(&help.stderr, &help.stdout), 120)
            ),
            "a working memray command (import memray in the interpreter that runs it)",
            "pip install memray   (or run dla with PYTHONPATH pointing at it)",
        ));
    }
    let detach = help.stdout.contains("detach") && (runs_alone || env.is_empty());

    let secs = req.duration.as_secs().max(1);
    let (stem, ext) = ("capture", "bin");
    let file = match &req.out_name {
        Some(n) => output_name(n, ext).map_err(|m| {
            AttachError::not_ready(
                m,
                "--out as a plain file name",
                "dla attach ... --out capture.bin",
            )
        })?,
        None => format!("{stem}.{ext}"),
    };
    let mut notes = vec![format!(
        "target Python {}.{}.{}, memray {memray_ver}, attach method {method}",
        ver.0, ver.1, ver.2
    )];
    if let Ok(scope) = std::fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope")
        && method == "gdb"
        && scope.trim() != "0"
    {
        notes.push(format!("kernel.yama.ptrace_scope={}: the debugger attach may be refused unless dla runs as root", scope.trim()));
    }
    let effects = if method == "gdb" {
        vec![
            format!("a debugger (gdb/lldb) attaches to {pid} with ptrace and injects the memray tracker; the target stops for a moment (about 0.5 s)"),
            format!("allocations are tracked for {secs}s: every malloc and Python allocation is hooked, which slows the target"),
            "memray imports in the target and its tracker thread writes the capture file from the target process".to_owned(),
        ]
    } else {
        vec![
            format!(
                "sys.remote_exec runs the memray tracker bootstrap in {pid} without a debugger"
            ),
            format!(
                "allocations are tracked for {secs}s: every malloc and Python allocation is hooked, which slows the target"
            ),
        ]
    };
    Ok(Plan {
        backend: Backend::Memray,
        op: req.op,
        tool: format!("memray {memray_ver} ({method})"),
        expected_signal: "a memray capture (.bin) with the allocations made after the attach, summarized by memray stats".into(),
        stopping: format!("the tracker stops itself after {secs}s (--duration); memray detach ends it early"),
        effects,
        cleanup: vec![
            if detach {
                format!("run memray detach {pid} if the attach is aborted")
            } else {
                format!("note memray detach {pid} ends the tracking by hand")
            },
            "note the memray module stays imported in the target until it exits".to_owned(),
        ],
        estimate: Estimate {
            dump_bytes: 16 * MIB + secs * MIB,
            peak_bytes: 128 * MIB,
            basis: format!("no pre-estimate exists: 16 MiB plus 1 MiB per second of --duration ({secs}s) assumed"),
        },
        notes,
        unverified: false,
        file,
        detail: Detail::Memray(Prep { cmd, env, detach, method }),
    })
}

fn interp_hint(req: &Request) -> String {
    req.target
        .args
        .first()
        .cloned()
        .unwrap_or_else(|| "python3".to_owned())
}

fn tracer_pid(pid: u32) -> Option<u32> {
    std::fs::read_to_string(format!("/proc/{pid}/status"))
        .ok()?
        .lines()
        .find_map(|l| l.strip_prefix("TracerPid:"))?
        .trim()
        .parse()
        .ok()
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
    let path = out_dir.join(&plan.file);
    t.note("the memray module stays imported in the target until it exits");
    t.need(Cleanup::Rm(path.clone()));
    let detach = Cleanup::Run(
        p.cmd
            .iter()
            .cloned()
            .chain(["detach".to_owned(), pid.to_string()])
            .collect(),
    );
    if p.detach {
        t.need(detach.clone());
    }
    let started = Instant::now();
    let mut c = Command::new(&p.cmd[0]);
    c.args(&p.cmd[1..])
        .args(["attach", "--method", "auto", "-o"])
        .arg(&path)
        .args(["-f", "--duration", &secs.to_string(), &pid.to_string()]);
    for (k, v) in &p.env {
        c.env(k, v);
    }
    let r = exec::run(c, Duration::from_secs(120))
        .map_err(|e| AttachError::failed(format!("memray attach: {e}"), "repeat the command"))?;
    if !r.success {
        let why = first_error_line(&r.stderr, &r.stdout);
        return Err(AttachError::failed(
            format!("memray attach failed: {}", clip(&why, 200)),
            "check that memray is importable in the target and the debugger may attach (same user, or root)",
        ));
    }
    let attach_ms = started.elapsed().as_millis() as u64;
    // The tracker runs inside the target for `secs`; wait for it to end by itself.
    let end = Instant::now() + Duration::from_secs(secs + 1);
    while Instant::now() < end {
        if !crate::os::is_alive(pid) {
            return Err(AttachError::failed(
                format!("process {pid} exited while memray was tracking it"),
                "repeat the command on a running process",
            ));
        }
        thread::sleep(Duration::from_millis(200));
    }
    let bytes = wait_stable(&path, Duration::from_secs(20), Duration::from_secs(1));
    let Some(bytes) = bytes else {
        return Err(AttachError::failed(
            format!("the memray capture {} did not appear", path.display()),
            "repeat the command; check the target is a CPython process that imports memray",
        ));
    };
    t.keep(&Cleanup::Rm(path.clone()));
    if p.detach {
        t.keep(&detach);
    }
    let mut done = Done::default();
    done.files.push(plan.file.clone());
    // memray stats --json writes its own file.
    let stats_name = format!("{}.stats.json", plan.file.trim_end_matches(".bin"));
    let stats_path = out_dir.join(&stats_name);
    let mut c = Command::new(&p.cmd[0]);
    c.args(&p.cmd[1..])
        .args(["stats", "--json", "-f", "-o"])
        .arg(&stats_path)
        .arg(&path);
    for (k, v) in &p.env {
        c.env(k, v);
    }
    let mut summary = Map::new();
    summary.insert("capture_bytes".into(), json!(bytes));
    summary.insert("method".into(), json!(p.method));
    match exec::run(c, Duration::from_secs(120)) {
        Ok(r) if r.success => {
            let text = std::fs::read_to_string(&stats_path).unwrap_or_default();
            match parse_memray_stats(&text, 3) {
                Some(s) => {
                    summary.insert("allocations".into(), json!(s.total_allocations));
                    summary.insert("bytes_allocated".into(), json!(s.total_bytes_allocated));
                    summary.insert("peak_memory".into(), json!(s.peak_memory));
                    summary.insert(
                        "top".into(),
                        json!(
                            s.top_by_size
                                .iter()
                                .map(|(l, b)| json!({"location": clip(l, 100), "bytes": b}))
                                .collect::<Vec<_>>()
                        ),
                    );
                    done.files.push(stats_name);
                }
                None => done
                    .limitations
                    .push("memray stats --json wrote a file DLA could not read".into()),
            }
        }
        Ok(r) => done.limitations.push(format!(
            "memray stats failed: {}",
            clip(&first_error_line(&r.stderr, &r.stdout), 120)
        )),
        Err(e) => done.limitations.push(format!("memray stats failed: {e}")),
    }
    if let Some(tp) = tracer_pid(pid)
        && tp != 0
    {
        done.limitations.push(format!(
            "process {pid} is still traced by {tp} after the capture"
        ));
    }
    done.limitations.push("only allocations made after the attach are recorded: this shows growth during the window, not what the process already held".into());
    done.summary = serde_json::Value::Object(summary);
    done.durations.insert("attach_ms".into(), json!(attach_ms));
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_check_output_is_parsed() {
        assert_eq!(
            parse_python_check("3 13 18 1.20.0\n"),
            Some(((3, 13, 18), "1.20.0".to_owned()))
        );
        assert_eq!(parse_python_check("3 14"), None);
        assert_eq!(parse_python_check("garbage"), None);
    }

    #[test]
    fn the_attach_method_follows_the_python_version() {
        assert_eq!(method_for((3, 13, 18)), "gdb");
        assert_eq!(method_for((3, 14, 0)), "sys.remote_exec");
        assert_eq!(method_for((3, 15, 1)), "sys.remote_exec");
        assert_eq!(method_for((3, 9, 1)), "gdb");
    }
}
