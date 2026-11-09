//! `bpftrace`: watch the memory-mapping system calls of a running process for a while.
//!
//! Tier 3 (kernel level). The script traces `mmap`, `munmap` and `brk` for one process id: the
//! bytes mapped per symbolized user stack, the bytes unmapped, the calls, and how far the
//! program break moved. It sees growth that no runtime counter owns (arenas, mapped files,
//! allocator fragmentation) and names the call stacks that map it. It never reads the target's
//! memory.
//!
//! Prerequisites, each reported by name when missing: the `bpftrace` program, root (or the
//! BPF capabilities), tracefs with the syscall tracepoints, and kernel BTF. The run is bounded
//! three ways: the script exits itself after `--duration`, the wall clock adds a margin and
//! then interrupts and kills, and the output file is capped (interrupted when it grows past
//! the cap, so the maps still print).
//!
//! This backend has been written against the bpftrace documentation and tested with a fake
//! `bpftrace` program only: the plan says so (`unverified`).

use std::{
    fs::File,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde_json::{Map, json};

use super::{
    Backend, Detail, Done, Plan, Request, Tracker,
    resources::Estimate,
    util::{AttachError, Cleanup, ErrorKind, clip, output_name, own_uid, same_process},
};
use crate::exec;

const MIB: u64 = 1 << 20;
/// Largest output file kept; past it the tracer is interrupted so its maps print.
const OUTPUT_CAP: u64 = 8 * MIB;
/// Extra wall time after `--duration` before the tracer is interrupted.
#[cfg(not(test))]
const MARGIN: Duration = Duration::from_secs(30);
#[cfg(test)]
const MARGIN: Duration = Duration::from_secs(0);
/// How long an interrupted tracer may take to print its maps before it is killed.
#[cfg(not(test))]
const GRACE: Duration = Duration::from_secs(15);
#[cfg(test)]
const GRACE: Duration = Duration::from_secs(1);
/// Distinct stacks the maps may hold (`BPFTRACE_MAP_KEYS_MAX`).
const MAP_KEYS: u32 = 1024;
/// User stack depth.
const STACK_DEPTH: u32 = 8;
/// Stacks and frames kept in the summary.
const TOP_STACKS: usize = 3;
const TOP_FRAMES: usize = 4;

pub(crate) struct Prep {
    bin: PathBuf,
}

/// What the plan reads from the machine; tests pass their own.
#[derive(Debug, Clone)]
pub(crate) struct Sys {
    /// Whether the platform is Linux.
    pub linux: bool,
    /// Whether the process may load BPF programs (root).
    pub root: Option<bool>,
    /// Directories that may hold the tracefs `events/syscalls` tree.
    pub tracefs: Vec<PathBuf>,
    /// The kernel's BTF file.
    pub btf: PathBuf,
}

impl Sys {
    pub(crate) fn host() -> Self {
        Self {
            linux: cfg!(target_os = "linux"),
            root: own_uid().map(|u| u == 0),
            tracefs: vec![
                PathBuf::from("/sys/kernel/tracing"),
                PathBuf::from("/sys/kernel/debug/tracing"),
            ],
            btf: PathBuf::from("/sys/kernel/btf/vmlinux"),
        }
    }
}

/// One missing prerequisite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Gap {
    /// `bpftrace`, `root`, `tracefs`, `btf` or `linux`.
    pub name: &'static str,
    pub why: String,
    pub fix: &'static str,
}

/// The script. `{secs}` is the time cap. The process id is the first argument.
pub fn script(secs: u64) -> String {
    format!(
        "// DLA: bytes mapped per user stack, for one process. Argument 1 is the process id.\n\
tracepoint:syscalls:sys_enter_mmap /pid == $1/ {{\n\
  @mmap_bytes[ustack({STACK_DEPTH})] = sum(args->len);\n\
  @mmap_calls = count();\n\
}}\n\
tracepoint:syscalls:sys_enter_munmap /pid == $1/ {{\n\
  @munmap_bytes = sum(args->len);\n\
  @munmap_calls = count();\n\
}}\n\
tracepoint:syscalls:sys_enter_brk /pid == $1 && args->brk != 0/ {{\n\
  @brk_calls = count();\n\
  @brk_min = min(args->brk);\n\
  @brk_max = max(args->brk);\n\
}}\n\
interval:s:{secs} {{ exit(); }}\n"
    )
}

/// Which prerequisites are missing, in the order they are checked.
pub(crate) fn gaps(sys: &Sys, bpftrace: Option<&Path>) -> Vec<Gap> {
    let mut out = Vec::new();
    if !sys.linux {
        out.push(Gap {
            name: "linux",
            why: "bpftrace traces the Linux kernel; this is not Linux".into(),
            fix: "run the investigation on a Linux host (or use the runtime's own tool)",
        });
        return out;
    }
    if bpftrace.is_none() {
        out.push(Gap {
            name: "bpftrace",
            why: "the bpftrace program is not installed".into(),
            fix: "apt-get install bpftrace   (or dnf install bpftrace)",
        });
    }
    if sys.root != Some(true) {
        out.push(Gap {
            name: "root",
            why: "loading BPF programs needs root (or CAP_BPF and CAP_PERFMON)".into(),
            fix: "run dla as root (sudo), or grant the capabilities to bpftrace",
        });
    }
    let tracefs = sys
        .tracefs
        .iter()
        .any(|d| d.join("events/syscalls/sys_enter_mmap").is_dir());
    if !tracefs {
        out.push(Gap {
            name: "tracefs",
            why: "tracefs is not mounted or has no syscalls tracepoints (sys_enter_mmap)".into(),
            fix: "mount -t tracefs nodev /sys/kernel/tracing",
        });
    }
    if !sys.btf.is_file() {
        out.push(Gap {
            name: "btf",
            why: format!("the kernel has no BTF at {}", sys.btf.display()),
            fix: "use a kernel built with CONFIG_DEBUG_INFO_BTF=y",
        });
    }
    out
}

/// The error for a non-empty list of gaps: it names every missing prerequisite.
pub(crate) fn gap_error(gaps: &[Gap]) -> AttachError {
    let names: Vec<&str> = gaps.iter().map(|g| g.name).collect();
    let why: Vec<String> = gaps
        .iter()
        .map(|g| format!("{}: {}", g.name, g.why))
        .collect();
    let fix: Vec<&str> = gaps.iter().map(|g| g.fix).collect();
    let message = format!(
        "bpftrace cannot run here; missing prerequisite(s) {}: {}",
        names.join(", "),
        why.join("; ")
    );
    let missing = format!("bpftrace prerequisites: {}", names.join(", "));
    let fix = fix.join("   ;   ");
    if gaps.iter().any(|g| g.name == "bpftrace") {
        AttachError::tool_missing(message, &missing, &fix)
    } else {
        AttachError::not_ready(message, &missing, fix)
    }
}

pub(crate) fn plan(req: &Request) -> Result<Plan, AttachError> {
    plan_with(req, &Sys::host())
}

pub(crate) fn plan_with(req: &Request, sys: &Sys) -> Result<Plan, AttachError> {
    let bin = exec::find_exe("bpftrace", &req.tool_dirs);
    let missing = gaps(sys, bin.as_deref());
    let Some(bin) = bin.filter(|_| missing.is_empty()) else {
        return Err(gap_error(&missing));
    };
    let pid = req.target.pid;
    let secs = req.duration.as_secs().max(1);
    let file = match &req.out_name {
        Some(n) => output_name(n, "txt").map_err(|m| {
            AttachError::not_ready(
                m,
                "--out as a plain file name",
                "dla attach ... --out maps.txt",
            )
        })?,
        None => "bpftrace.txt".to_owned(),
    };
    Ok(Plan {
        backend: Backend::Bpftrace,
        op: req.op,
        tool: format!("bpftrace {}", bin.display()),
        expected_signal: "bytes mapped per symbolized user stack, bytes unmapped, call counts and the program break movement of the process during the window".into(),
        stopping: format!("the script exits after {secs}s (--duration); DLA interrupts it {}s later if it has not, and kills it {}s after that; the output is capped at {} MiB", MARGIN.as_secs(), GRACE.as_secs(), OUTPUT_CAP / MIB),
        effects: vec![
            "loads eBPF programs on the mmap, munmap and brk tracepoints of the whole kernel, filtered to one process id; every call of every process runs a few instructions of the filter".into(),
            format!("reads the user stacks of process {pid} at each mmap call; it never reads the target's memory contents"),
            "the programs are removed when bpftrace exits".into(),
        ],
        cleanup: vec![
            "interrupt the bpftrace process if the attach is aborted (kill -INT)".into(),
            "nothing stays loaded in the kernel after bpftrace exits".into(),
        ],
        estimate: Estimate {
            dump_bytes: MIB,
            peak_bytes: 128 * MIB,
            basis: format!("the output is capped at {} MiB; bpftrace compiles and holds up to {MAP_KEYS} stacks in kernel maps", OUTPUT_CAP / MIB),
        },
        notes: vec![format!("bpftrace {}", bin.display())],
        unverified: true,
        file,
        detail: Detail::Bpftrace(Prep { bin }),
    })
}

/// Totals and the largest stacks read from bpftrace's printed maps.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapSummary {
    /// Sum of the bytes of every counted `mmap` call.
    pub mmap_bytes: u64,
    /// Sum of the bytes of every counted `munmap` call.
    pub munmap_bytes: u64,
    /// `mmap` calls.
    pub mmap_calls: u64,
    /// `munmap` calls.
    pub munmap_calls: u64,
    /// `brk` calls that set the break.
    pub brk_calls: u64,
    /// Highest minus lowest break set, when the break moved.
    pub brk_growth: Option<u64>,
    /// `(bytes, frames innermost first)` of the stacks, largest first.
    pub stacks: Vec<(u64, Vec<String>)>,
}

/// Reads the maps bpftrace prints at exit. Lines it does not understand are skipped.
pub fn parse_maps(text: &str) -> MapSummary {
    let mut s = MapSummary::default();
    let mut brk: (Option<u64>, Option<u64>) = (None, None);
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix('@') else {
            continue;
        };
        if let Some(name) = rest.strip_suffix('[') {
            // A multi-line key: frames until `]: value`.
            let mut frames: Vec<String> = Vec::new();
            let mut value: Option<u64> = None;
            for l in lines.by_ref() {
                let l = l.trim();
                if let Some(v) = l.strip_prefix("]:") {
                    value = v.trim().parse().ok();
                    break;
                }
                if !l.is_empty() {
                    frames.push(l.to_owned());
                }
            }
            if let (Some(v), "mmap_bytes") = (value, name) {
                s.mmap_bytes = s.mmap_bytes.saturating_add(v);
                s.stacks.push((v, frames));
            }
            continue;
        }
        if let Some((name, tail)) = rest.split_once("]: ")
            && let Some((base, key)) = name.split_once('[')
        {
            if let (Ok(v), "mmap_bytes") = (tail.trim().parse::<u64>(), base) {
                s.mmap_bytes = s.mmap_bytes.saturating_add(v);
                s.stacks.push((v, vec![key.trim().to_owned()]));
            }
            continue;
        }
        let Some((name, value)) = rest.split_once(':') else {
            continue;
        };
        let Ok(v) = value.trim().parse::<u64>() else {
            continue;
        };
        match name.trim() {
            "mmap_calls" => s.mmap_calls = v,
            "munmap_calls" => s.munmap_calls = v,
            "munmap_bytes" => s.munmap_bytes = v,
            "brk_calls" => s.brk_calls = v,
            "brk_min" => brk.0 = Some(v),
            "brk_max" => brk.1 = Some(v),
            _ => {}
        }
    }
    if let (Some(lo), Some(hi)) = brk {
        s.brk_growth = Some(hi.saturating_sub(lo));
    }
    s.stacks.sort_by_key(|a| std::cmp::Reverse(a.0));
    s
}

fn interrupt(pid: u32) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-INT", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
    }
}

fn summary_json(m: &MapSummary, secs: u64) -> serde_json::Value {
    let top: Vec<serde_json::Value> = m
        .stacks
        .iter()
        .take(TOP_STACKS)
        .map(|(bytes, frames)| {
            json!({
                "bytes": bytes,
                "frames": frames.iter().take(TOP_FRAMES).map(|f| clip(f, 80)).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "seconds": secs,
        "mmap_bytes": m.mmap_bytes,
        "munmap_bytes": m.munmap_bytes,
        "mapped_net": i128::from(m.mmap_bytes) - i128::from(m.munmap_bytes),
        "mmap_calls": m.mmap_calls,
        "munmap_calls": m.munmap_calls,
        "brk_calls": m.brk_calls,
        "brk_growth": m.brk_growth,
        "stacks": m.stacks.len(),
        "top": top,
    })
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
    let out_path = out_dir.join(&plan.file);
    let script_name = "bpftrace.bt";
    let log_name = "bpftrace.log";
    let io = |what: &str, e: std::io::Error| {
        AttachError::failed(format!("{what}: {e}"), "check free disk and permissions")
    };
    std::fs::write(out_dir.join(script_name), script(secs))
        .map_err(|e| io("cannot write the script", e))?;
    let out = File::create(&out_path).map_err(|e| io("cannot create the output file", e))?;
    let log = File::create(out_dir.join(log_name)).map_err(|e| io("cannot create the log", e))?;
    t.need(Cleanup::Rm(out_path.clone()));
    let started = Instant::now();
    let mut child = Command::new(&p.bin)
        .arg(out_dir.join(script_name))
        .arg(pid.to_string())
        .env("BPFTRACE_MAP_KEYS_MAX", MAP_KEYS.to_string())
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(log)
        .spawn()
        .map_err(|e| {
            AttachError::failed(
                format!("cannot start bpftrace: {e}"),
                "check bpftrace is executable and dla runs as root",
            )
        })?;
    let stop = Cleanup::Run(vec!["kill".into(), "-INT".into(), child.id().to_string()]);
    t.need(stop.clone());

    let wall = Duration::from_secs(secs) + MARGIN;
    let mut interrupted_at: Option<Instant> = None;
    let mut truncated = false;
    let mut target_gone = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) => {}
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                // Reaped: the pid may be reused, so the interrupt must never run.
                t.keep(&stop);
                return Err(io("waiting for bpftrace", e));
            }
        }
        let size = std::fs::metadata(&out_path).map_or(0, |m| m.len());
        if interrupted_at.is_none() && (started.elapsed() >= wall || size > OUTPUT_CAP) {
            truncated = size > OUTPUT_CAP;
            interrupt(child.id());
            interrupted_at = Some(Instant::now());
        }
        if let Some(at) = interrupted_at
            && at.elapsed() >= GRACE
        {
            let _ = child.kill();
            let _ = child.wait();
            // Reaped: the pid may be reused, so the interrupt must never run.
            t.keep(&stop);
            return Err(AttachError::failed(
                "bpftrace did not stop after an interrupt and was killed; its maps were not printed",
                "repeat with a shorter --duration",
            ));
        }
        if !target_gone && !same_process(&req.target) {
            target_gone = true;
        }
        thread::sleep(Duration::from_millis(100));
    };
    t.keep(&stop);
    let text = read_capped(&out_path);
    let err_tail = std::fs::read_to_string(out_dir.join(log_name))
        .unwrap_or_default()
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .map(|l| clip(l.trim(), 200))
        .unwrap_or_default();
    if !status.success() && text.trim().is_empty() {
        return Err(AttachError {
            kind: ErrorKind::Failed,
            message: format!("bpftrace failed: {err_tail}"),
            missing: "a bpftrace run that attached its probes".into(),
            fix: "run as root on a kernel with tracefs and BTF; see bpftrace.log in the attach directory".into(),
        });
    }
    t.keep(&Cleanup::Rm(out_path.clone()));
    let maps = parse_maps(&text);
    let mut done = Done::default();
    done.files.push(plan.file.clone());
    done.files.push(script_name.to_owned());
    done.files.push(log_name.to_owned());
    done.summary = serde_json::Value::Object(match summary_json(&maps, secs) {
        serde_json::Value::Object(m) => m,
        _ => Map::new(),
    });
    done.limitations.push(
        "mmap and munmap lengths include file mappings and thread stacks; the net figure is the change in mapped address space, not resident memory".into(),
    );
    done.limitations.push(
        "only calls made during the window are counted: this shows growth in the window, not what the process already held".into(),
    );
    if truncated {
        done.limitations.push(format!(
            "the output passed {} MiB and the tracer was interrupted early",
            OUTPUT_CAP / MIB
        ));
    }
    if target_gone {
        done.limitations
            .push(format!("process {pid} exited during the window"));
    }
    if maps.stacks.is_empty() && maps.mmap_calls == 0 {
        done.limitations
            .push("no mmap call of the process was seen; an empty result does not clear it".into());
    }
    done.durations.insert(
        "trace_ms".into(),
        json!(started.elapsed().as_millis() as u64),
    );
    Ok(done)
}

fn read_capped(path: &Path) -> String {
    use std::io::Read;
    let Ok(f) = File::open(path) else {
        return String::new();
    };
    let mut buf = Vec::new();
    let _ = f.take(OUTPUT_CAP + MIB).read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const OUTPUT: &str = "Attaching 5 probes...\n@brk_calls: 12\n@brk_max: 140737488355328\n@brk_min: 140737488351232\n@mmap_bytes[\n    __mmap64+19\n    malloc+310\n    codec_open+31\n    main+88\n]: 6291456\n@mmap_bytes[\n    __mmap64+19\n    arena_grow+12\n]: 1048576\n@mmap_calls: 70\n@munmap_bytes: 2097152\n@munmap_calls: 5\n";

    fn sys(root: bool, tracefs: bool, btf: bool) -> (tempfile::TempDir, Sys) {
        let d = tempfile::tempdir().unwrap();
        if tracefs {
            fs::create_dir_all(d.path().join("tracing/events/syscalls/sys_enter_mmap")).unwrap();
        }
        if btf {
            fs::write(d.path().join("vmlinux"), b"btf").unwrap();
        }
        let s = Sys {
            linux: true,
            root: Some(root),
            tracefs: vec![d.path().join("tracing")],
            btf: d.path().join("vmlinux"),
        };
        (d, s)
    }

    #[test]
    fn the_script_is_bounded_and_filters_one_process() {
        let s = script(20);
        assert!(s.contains("interval:s:20 { exit(); }"));
        assert!(s.contains("/pid == $1/"));
        for probe in ["sys_enter_mmap", "sys_enter_munmap", "sys_enter_brk"] {
            assert!(s.contains(probe), "{probe}");
        }
        assert!(s.contains("ustack(8)"));
    }

    #[test]
    fn maps_are_totalled_and_the_stacks_ranked() {
        let m = parse_maps(OUTPUT);
        assert_eq!(m.mmap_bytes, 6291456 + 1048576);
        assert_eq!(m.munmap_bytes, 2097152);
        assert_eq!((m.mmap_calls, m.munmap_calls, m.brk_calls), (70, 5, 12));
        assert_eq!(m.brk_growth, Some(4096));
        assert_eq!(m.stacks.len(), 2);
        assert_eq!(m.stacks[0].0, 6291456);
        assert_eq!(m.stacks[0].1[2], "codec_open+31");
        let j = summary_json(&m, 20);
        assert_eq!(j["mapped_net"], 6291456 + 1048576 - 2097152);
        assert_eq!(j["top"][0]["frames"].as_array().unwrap().len(), TOP_FRAMES);
        assert!(j.to_string().len() < 900);
    }

    #[test]
    fn garbage_and_empty_output_parse_to_nothing() {
        for text in [
            "",
            "Attaching 5 probes...\nERROR: nope\n",
            "@x[\n@y: abc\n",
            "@a: 99999999999999999999999\n",
        ] {
            let m = parse_maps(text);
            assert_eq!(m.mmap_calls, 0, "{text:?}");
            assert!(m.stacks.is_empty());
        }
        // A one-line key is read too.
        let m = parse_maps("@mmap_bytes[0x7f00]: 4096\n");
        assert_eq!(m.mmap_bytes, 4096);
    }

    #[test]
    fn every_missing_prerequisite_is_named() {
        let (_d, s) = sys(false, false, false);
        let g = gaps(&s, None);
        let names: Vec<&str> = g.iter().map(|g| g.name).collect();
        assert_eq!(names, ["bpftrace", "root", "tracefs", "btf"]);
        let e = gap_error(&g);
        assert_eq!(e.kind, ErrorKind::ToolMissing);
        for n in names {
            assert!(e.message.contains(n), "{n}: {}", e.message);
        }
        assert!(e.fix.contains("apt-get install bpftrace"));
    }

    #[test]
    fn a_present_tool_with_missing_privilege_is_not_a_missing_tool() {
        let (_d, s) = sys(false, true, true);
        let g = gaps(&s, Some(Path::new("/usr/bin/bpftrace")));
        assert_eq!(g.iter().map(|g| g.name).collect::<Vec<_>>(), ["root"]);
        assert_eq!(gap_error(&g).kind, ErrorKind::NotReady);
        let (_d, ok) = sys(true, true, true);
        assert!(gaps(&ok, Some(Path::new("/usr/bin/bpftrace"))).is_empty());
    }

    #[test]
    fn off_linux_there_is_one_gap() {
        let s = Sys {
            linux: false,
            root: Some(true),
            tracefs: vec![],
            btf: PathBuf::new(),
        };
        let g = gaps(&s, Some(Path::new("/x")));
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].name, "linux");
    }

    #[cfg(unix)]
    mod fake {
        use std::os::unix::fs::PermissionsExt;

        use super::*;
        use crate::attach::{Op, TargetInfo};

        fn fake_bpftrace(dir: &Path, body: &str) {
            let p = dir.join("bpftrace");
            fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
            fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        }

        fn request(tools: &Path) -> Request {
            Request {
                target: TargetInfo {
                    pid: std::process::id(),
                    exe: None,
                    uid: None,
                    gids: vec![],
                    start_time: None,
                    args: vec![],
                    rss: None,
                },
                backend: Backend::Bpftrace,
                op: Op::Record,
                duration: Duration::from_secs(1),
                out_name: None,
                dump_root: PathBuf::new(),
                tool_dirs: vec![tools.to_path_buf()],
            }
        }

        #[test]
        fn a_fake_tool_is_planned_and_run_to_a_summary() {
            let tools = tempfile::tempdir().unwrap();
            let body = format!("cat <<'EOF'\n{OUTPUT}EOF");
            fake_bpftrace(tools.path(), &body);
            let (_d, s) = sys(true, true, true);
            let req = request(tools.path());
            let plan = plan_with(&req, &s).unwrap();
            assert!(plan.unverified);
            let Detail::Bpftrace(prep) = &plan.detail else {
                panic!("wrong detail")
            };
            let out = tempfile::tempdir().unwrap();
            let mut sink = |_: &Cleanup| {};
            let mut t = Tracker::new(&mut sink);
            let done = run(prep, &plan, &req, out.path(), &mut t).unwrap();
            assert_eq!(done.files[0], "bpftrace.txt");
            assert_eq!(done.summary["mmap_calls"], 70);
            assert_eq!(done.summary["top"][0]["bytes"], 6291456);
            assert!(out.path().join("bpftrace.bt").is_file());
            // The cleanup guarding the process and the file is released on success.
            assert!(t.left.is_empty(), "{:?}", t.left);
        }

        #[test]
        fn a_tracer_killed_after_the_grace_leaves_no_stale_interrupt() {
            let tools = tempfile::tempdir().unwrap();
            fake_bpftrace(tools.path(), "trap '' INT; while :; do sleep 1; done");
            let (_d, s) = sys(true, true, true);
            let req = request(tools.path());
            let plan = plan_with(&req, &s).unwrap();
            let Detail::Bpftrace(prep) = &plan.detail else {
                panic!("wrong detail")
            };
            let out = tempfile::tempdir().unwrap();
            let mut seen = Vec::new();
            let mut sink = |c: &Cleanup| seen.push(c.clone());
            let mut t = Tracker::new(&mut sink);
            let e = run(prep, &plan, &req, out.path(), &mut t).unwrap_err();
            assert!(e.message.contains("did not stop"), "{}", e.message);
            // The kill -INT <pid> entry was registered, and is gone: the pid is reaped.
            assert!(
                t.left.iter().all(|c| !matches!(c, Cleanup::Run(_))),
                "{:?}",
                t.left
            );
            assert!(seen.iter().any(|c| matches!(c, Cleanup::Run(_))));
        }

        #[test]
        fn a_tool_that_fails_without_output_is_an_error_with_the_log_line() {
            let tools = tempfile::tempdir().unwrap();
            fake_bpftrace(
                tools.path(),
                "echo 'ERROR: Could not resolve tracepoint' >&2; exit 1",
            );
            let (_d, s) = sys(true, true, true);
            let req = request(tools.path());
            let plan = plan_with(&req, &s).unwrap();
            let Detail::Bpftrace(prep) = &plan.detail else {
                panic!("wrong detail")
            };
            let out = tempfile::tempdir().unwrap();
            let mut sink = |_: &Cleanup| {};
            let mut t = Tracker::new(&mut sink);
            let e = run(prep, &plan, &req, out.path(), &mut t).unwrap_err();
            assert_eq!(e.kind, ErrorKind::Failed);
            assert!(
                e.message.contains("Could not resolve tracepoint"),
                "{}",
                e.message
            );
        }

        #[test]
        fn a_missing_privilege_stops_the_plan_before_anything_runs() {
            let tools = tempfile::tempdir().unwrap();
            fake_bpftrace(tools.path(), "exit 0");
            let (_d, s) = sys(false, true, true);
            let e = plan_with(&request(tools.path()), &s).unwrap_err();
            assert_eq!(e.kind, ErrorKind::NotReady);
            assert!(e.message.contains("root"));
        }

        #[test]
        fn a_missing_program_is_a_missing_tool() {
            let tools = tempfile::tempdir().unwrap();
            let (_d, s) = sys(true, true, true);
            // PATH may hold a real bpftrace: the test is about the planned error only when
            // none is found.
            if exec::find_exe("bpftrace", &[tools.path().to_path_buf()]).is_some() {
                eprintln!("skipped: a real bpftrace is on PATH");
                return;
            }
            let e = plan_with(&request(tools.path()), &s).unwrap_err();
            assert_eq!(e.kind, ErrorKind::ToolMissing);
        }
    }
}
