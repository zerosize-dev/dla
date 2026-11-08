//! `jcmd` operations: class histogram, heap dump, thread dump, native memory summary and a
//! JFR recording.

use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use serde_json::{Map, Value, json};

use super::{
    Backend, Detail, Done, Op, Plan, Request, Tracker,
    parse::{parse_histogram, parse_jfr_summary, parse_nmt_summary},
    resources::Estimate,
    util::{
        AttachError, Cleanup, clip, dir_writable_by, first_error_line, output_name, same_process,
        wait_stable,
    },
};
use crate::exec;

const JAVA_ENV: [&str; 3] = ["JAVA_TOOL_OPTIONS", "JDK_JAVA_OPTIONS", "_JAVA_OPTIONS"];
const MIB: u64 = 1 << 20;
const HPROF_MAGIC: &[u8] = b"JAVA PROFILE 1.0.";

pub(crate) struct Prep {
    exe: PathBuf,
    /// The `jfr` tool next to `jcmd`, when there is one.
    jfr: Option<PathBuf>,
    /// NMT text read while planning (the NMT operation only).
    nmt_text: Option<String>,
}

fn find_tool(name: &str, req: &Request) -> Option<PathBuf> {
    // An explicit tools directory wins over JAVA_HOME and PATH.
    let mut extra: Vec<PathBuf> = req.tool_dirs.clone();
    if let Some(home) = std::env::var_os("JAVA_HOME") {
        extra.push(PathBuf::from(home).join("bin"));
    }
    exec::find_exe(name, &extra)
}

fn call(exe: &Path, pid: u32, args: &[&str], timeout: Duration) -> Result<String, String> {
    let mut cmd = Command::new(exe);
    for var in JAVA_ENV {
        cmd.env_remove(var);
    }
    cmd.arg(pid.to_string()).args(args);
    let what = args.first().copied().unwrap_or("");
    let run = exec::run(cmd, timeout).map_err(|e| format!("jcmd {what}: {e}"))?;
    if run.success {
        return Ok(run.stdout);
    }
    Err(format!(
        "jcmd {what}: {}",
        first_error_line(&run.stderr, &run.stdout)
    ))
}

fn default_file(op: Op) -> (&'static str, &'static str) {
    match op {
        Op::Histogram => ("histogram", "txt"),
        Op::HeapDump => ("heap", "hprof"),
        Op::ThreadDump => ("threads", "json"),
        Op::Nmt => ("nmt", "txt"),
        _ => ("recording", "jfr"),
    }
}

/// Whether the maps text of a process lists the JVM library.
pub(crate) fn maps_show_jvm(maps: &str) -> bool {
    maps.lines().any(|l| {
        l.rsplit('/')
            .next()
            .is_some_and(|f| f.starts_with("libjvm.") || f == "jvm.dll")
    })
}

/// Positive evidence that `pid` is a JVM: a `java` executable, the JVM library in its maps, or
/// the JVM's own `hsperfdata` file. `jcmd` signals (SIGQUIT) any process it is pointed at, so
/// it must never see a pid that fails this test.
pub(crate) fn is_jvm(req: &Request) -> bool {
    let pid = req.target.pid;
    let name = req.target.exe_name().to_ascii_lowercase();
    if matches!(name.as_str(), "java" | "javaw" | "java.exe" | "javaw.exe") {
        return true;
    }
    if std::fs::read_to_string(format!("/proc/{pid}/maps")).is_ok_and(|m| maps_show_jvm(&m)) {
        return true;
    }
    let tmp = std::env::temp_dir();
    let has_perf = |dir: &Path| {
        std::fs::read_dir(dir).is_ok_and(|rd| {
            rd.filter_map(Result::ok).any(|e| {
                e.file_name().to_string_lossy().starts_with("hsperfdata_")
                    && e.path().join(pid.to_string()).exists()
            })
        })
    };
    has_perf(&tmp) || has_perf(Path::new("/tmp"))
}

pub(crate) fn plan(req: &Request) -> Result<Plan, AttachError> {
    let pid = req.target.pid;
    if !is_jvm(req) {
        return Err(AttachError::not_ready(
            format!(
                "process {pid} ({}) shows no sign of being a JVM; jcmd was not run and no signal was sent",
                req.target.exe_name()
            ),
            "a JVM (a java executable, libjvm in its maps, or an hsperfdata file)",
            "check the pid with `jcmd -l` or `ps`; jcmd would send SIGQUIT to a process that is not a JVM",
        ));
    }
    let Some(exe) = find_tool("jcmd", req) else {
        return Err(AttachError::tool_missing(
            "jcmd is not installed, so the JVM cannot be inspected",
            "jcmd on PATH or JAVA_HOME/bin",
            "install a JDK (jcmd ships with the JDK, not the JRE) and put its bin directory on PATH",
        ));
    };
    let jfr = exe.parent().map(|d| d.join("jfr")).filter(|p| p.is_file());
    // The target must accept the attach before anything else is planned.
    let version = call(&exe, pid, &["VM.version"], Duration::from_secs(20)).map_err(|e| {
        let fix = if e.contains("not permitted") || e.contains("Permission") {
            "run dla as the same user as the JVM (or as root), then repeat the command".to_owned()
        } else {
            format!("check that process {pid} is a JVM that allows attach (no -XX:+DisableAttachMechanism)")
        };
        AttachError::not_ready(
            format!("process {pid} did not accept a jcmd attach: {e}"),
            "a JVM that accepts jcmd attach from this user",
            fix,
        )
    })?;
    let ver = version
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("JDK ") || l.contains("VM version"))
        .unwrap_or("JVM")
        .to_owned();

    let (stem, ext) = default_file(req.op);
    let file = match &req.out_name {
        Some(n) => output_name(n, ext).map_err(|m| {
            AttachError::not_ready(
                m,
                "--out as a plain file name",
                "dla attach ... --out heap.hprof",
            )
        })?,
        None => format!("{stem}.{ext}"),
    };

    // The JVM writes the dump, so its user must be able to write the directory.
    if matches!(req.op, Op::HeapDump | Op::ThreadDump | Op::Jfr) {
        let root = req.dump_root.display().to_string();
        if root.contains(char::is_whitespace) {
            return Err(AttachError::not_ready(
                format!("jcmd cannot take the path {root:?}: it contains whitespace"),
                "a run directory path without spaces",
                "run dla from a project path without spaces (or set --project to one)",
            ));
        }
        if let Some(uid) = req.target.uid
            && let Err(why) = dir_writable_by(&req.dump_root, uid, &req.target.gids)
        {
            return Err(AttachError::not_ready(
                format!("the JVM (user {uid}) cannot write under the run directory: {why}"),
                "a dump directory the target's user can write",
                "start the run from a directory the JVM's user can write, or run dla as that user",
            ));
        }
    }

    let mut notes = vec![ver.clone()];
    let mut nmt_text = None;
    let (estimate, signal, stopping, effects): (Estimate, String, String, Vec<String>) = match req.op {
        Op::Histogram => (
            Estimate {
                dump_bytes: 4 * MIB,
                peak_bytes: 32 * MIB,
                basis: "text output, a few MiB for large heaps".into(),
            },
            "a table of instances and bytes per class, largest first".into(),
            "the command returns".into(),
            vec![
                "jcmd GC.class_histogram: the JVM forces a full GC and walks the live heap in a safepoint (milliseconds to about a second)".into(),
                "the attach mechanism creates /tmp/.java_pid<pid> in the target's /tmp".into(),
            ],
        ),
        Op::HeapDump => {
            let info = call(&exe, pid, &["GC.heap_info"], Duration::from_secs(20))
                .ok()
                .and_then(|t| crate::jcmd::parse_heap_info(&t));
            let (used, basis) = match info {
                Some(i) => (
                    i.used,
                    format!("GC.heap_info used = {} bytes (the file is close to the live bytes, so this over-estimates)", i.used),
                ),
                None => {
                    let rss = req.target.rss.unwrap_or(1 << 30);
                    (rss, format!("GC.heap_info unreadable; the resident size of {rss} bytes is used"))
                }
            };
            (
                Estimate {
                    dump_bytes: used,
                    peak_bytes: 64 * MIB,
                    basis,
                },
                "an HPROF file starting with 'JAVA PROFILE 1.0.2'".into(),
                "the command returns after the JVM wrote the whole file".into(),
                vec![
                    "jcmd GC.heap_dump: the JVM forces a full GC (live objects only) and stops all threads while it writes the dump (about 0.1-0.4 s per 200 MB)".into(),
                    format!("the JVM, not DLA, writes {}/<attach dir>/{file} with mode 0600", req.dump_root.display()),
                    "the dump holds the values of live objects and may contain secrets".into(),
                    "the attach mechanism creates /tmp/.java_pid<pid> in the target's /tmp".into(),
                ],
            )
        }
        Op::ThreadDump => (
            Estimate {
                dump_bytes: 4 * MIB,
                peak_bytes: 32 * MIB,
                basis: "JSON thread dump, kilobytes per hundred threads".into(),
            },
            "a JSON document with threadDump.threadContainers[].threads[]".into(),
            "the command returns".into(),
            vec![
                "jcmd Thread.dump_to_file -format=json: a short safepoint; the JVM writes the file".into(),
                "the attach mechanism creates /tmp/.java_pid<pid> in the target's /tmp".into(),
            ],
        ),
        Op::Nmt => {
            let text = call(&exe, pid, &["VM.native_memory", "summary"], Duration::from_secs(30))
                .map_err(|e| AttachError::failed(e, "repeat the command"))?;
            if parse_nmt_summary(&text).is_none() {
                return Err(AttachError::not_ready(
                    format!("native memory tracking is not enabled in process {pid}"),
                    "a JVM started with -XX:NativeMemoryTracking=summary",
                    "restart the JVM with -XX:NativeMemoryTracking=summary (it cannot be switched on later), then dla attach jcmd --pid <pid> --op nmt",
                ));
            }
            nmt_text = Some(text);
            (
                Estimate {
                    dump_bytes: MIB,
                    peak_bytes: 16 * MIB,
                    basis: "text summary of about 50 lines".into(),
                },
                "totals and per-category reserved and committed bytes".into(),
                "the command returns".into(),
                vec![
                    "jcmd VM.native_memory summary: reads the tracking counters, no pause".into(),
                    "the attach mechanism creates /tmp/.java_pid<pid> in the target's /tmp".into(),
                ],
            )
        }
        _ => {
            let secs = req.duration.as_secs().max(1);
            (
                Estimate {
                    dump_bytes: 32 * MIB,
                    peak_bytes: 64 * MIB,
                    basis: "a profile-setting recording is a few hundred KiB per second; 32 MiB assumed".into(),
                },
                format!("a .jfr recording of {secs}s with allocation samples (class and stack, never object contents)"),
                format!("the recording ends after {secs}s and the JVM writes the file"),
                vec![
                    format!("jcmd JFR.start settings=profile duration={secs}s: allocation, GC and execution sampling at a few percent overhead for {secs}s"),
                    "the JVM writes the file with its own user; the recording also stores the JVM arguments and environment variables, which may contain secrets".into(),
                    "the attach mechanism creates /tmp/.java_pid<pid> in the target's /tmp".into(),
                ],
            )
        }
    };
    let cleanup = vec![
        format!("note /tmp/.java_pid{pid} is the JVM's attach socket and is left for the JVM"),
        "the artifact stays under the attach directory".to_owned(),
    ];
    notes.push(format!("jcmd {}", exe.display()));
    Ok(Plan {
        backend: Backend::Jcmd,
        op: req.op,
        tool: format!("jcmd ({})", clip(&ver, 80)),
        expected_signal: signal,
        stopping,
        effects,
        cleanup,
        estimate,
        notes,
        unverified: false,
        file,
        detail: Detail::Jcmd(Prep { exe, jfr, nmt_text }),
    })
}

fn failed(e: String) -> AttachError {
    AttachError::failed(e, "repeat the command; check the JVM is still running")
}

fn read_head(path: &Path, n: usize) -> Vec<u8> {
    use std::io::Read;
    let mut buf = vec![0u8; n];
    let got = std::fs::File::open(path)
        .and_then(|mut f| f.read(&mut buf))
        .unwrap_or(0);
    buf.truncate(got);
    buf
}

pub(crate) fn run(
    p: &Prep,
    plan: &Plan,
    req: &Request,
    out_dir: &Path,
    t: &mut Tracker<'_>,
) -> Result<Done, AttachError> {
    let pid = req.target.pid;
    let path = out_dir.join(&plan.file);
    let started = Instant::now();
    t.note(format!(
        "/tmp/.java_pid{pid} is the JVM's attach socket and is left for the JVM"
    ));
    let mut done = Done::default();
    let mut dur = Map::new();
    match req.op {
        Op::Histogram => {
            let text = call(
                &p.exe,
                pid,
                &["GC.class_histogram"],
                Duration::from_secs(300),
            )
            .map_err(failed)?;
            std::fs::write(&path, &text).map_err(|e| {
                AttachError::failed(
                    format!("cannot write {}: {e}", path.display()),
                    "check free disk",
                )
            })?;
            let h = parse_histogram(&text).ok_or_else(|| {
                AttachError::failed(
                    "the class histogram was not recognised",
                    "repeat the command",
                )
            })?;
            done.summary = json!({
                "classes": h.rows.len(),
                "total_instances": h.total_instances,
                "total_bytes": h.total_bytes,
                "top": h.rows.iter().take(5).map(|r| json!({
                    "class": clip(&r.class, 60), "instances": r.instances, "bytes": r.bytes
                })).collect::<Vec<_>>(),
            });
        }
        Op::HeapDump => {
            t.need(Cleanup::Rm(path.clone()));
            let arg = path.display().to_string();
            let t0 = Instant::now();
            let out = call(
                &p.exe,
                pid,
                &["GC.heap_dump", &arg],
                Duration::from_secs(900),
            )
            .map_err(failed)?;
            dur.insert("dump_ms".into(), json!(t0.elapsed().as_millis() as u64));
            let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if bytes == 0 {
                let why = if out.contains("Unable to create") || out.contains("rror") {
                    first_error_line("", &out)
                } else {
                    "the JVM reported no dump file".to_owned()
                };
                return Err(AttachError::failed(
                    format!("the heap dump was not written: {why}"),
                    "check that the JVM's user can write the attach directory, then repeat",
                ));
            }
            let magic = read_head(&path, 18);
            let ok = magic.starts_with(HPROF_MAGIC);
            if !ok {
                done.limitations
                    .push("the file does not start with 'JAVA PROFILE 1.0.x'".into());
            }
            t.keep(&Cleanup::Rm(path.clone()));
            done.summary = json!({
                "bytes": bytes,
                "header": String::from_utf8_lossy(&magic).trim_end_matches('\0'),
                "hprof_header_ok": ok,
            });
            done.limitations
                .push("the dump holds live objects only (jcmd forces a full GC first)".into());
        }
        Op::ThreadDump => {
            let arg = path.display().to_string();
            t.need(Cleanup::Rm(path.clone()));
            let r = call(
                &p.exe,
                pid,
                &["Thread.dump_to_file", "-format=json", &arg],
                Duration::from_secs(60),
            );
            let mut summary = Map::new();
            if r.is_ok() && path.is_file() {
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                let (containers, threads) = count_threads(&text);
                summary.insert("format".into(), json!("json"));
                summary.insert("containers".into(), json!(containers));
                summary.insert("threads".into(), json!(threads));
            } else {
                // Thread.dump_to_file is JDK 21+: fall back to the text thread dump.
                let why = r.err().unwrap_or_else(|| "no file was written".into());
                let text = call(
                    &p.exe,
                    pid,
                    &["Thread.print", "-l"],
                    Duration::from_secs(60),
                )
                .map_err(|e| failed(format!("{why}; the text fallback also failed: {e}")))?;
                let txt_name = plan.file.replace(".json", ".txt");
                let txt = out_dir.join(&txt_name);
                std::fs::write(&txt, &text).map_err(|e| {
                    AttachError::failed(
                        format!("cannot write {}: {e}", txt.display()),
                        "check free disk",
                    )
                })?;
                done.files.push(txt_name);
                summary.insert("format".into(), json!("text"));
                summary.insert(
                    "threads".into(),
                    json!(text.lines().filter(|l| l.starts_with('"')).count()),
                );
                done.limitations.push(format!(
                    "Thread.dump_to_file is not available ({}); the text dump was used",
                    clip(&why, 100)
                ));
            }
            t.keep(&Cleanup::Rm(path.clone()));
            done.summary = Value::Object(summary);
        }
        Op::Nmt => {
            let text = p.nmt_text.clone().unwrap_or_default();
            std::fs::write(&path, &text).map_err(|e| {
                AttachError::failed(
                    format!("cannot write {}: {e}", path.display()),
                    "check free disk",
                )
            })?;
            let n = parse_nmt_summary(&text).ok_or_else(|| {
                AttachError::failed(
                    "the native memory summary was not recognised",
                    "repeat the command",
                )
            })?;
            let mut cats = n.categories.clone();
            cats.sort_by_key(|c| std::cmp::Reverse(c.committed));
            done.summary = json!({
                "reserved": n.reserved,
                "committed": n.committed,
                "top": cats.iter().take(5).map(|c| json!({
                    "category": c.name, "committed": c.committed
                })).collect::<Vec<_>>(),
            });
        }
        _ => {
            let secs = req.duration.as_secs().max(1);
            let name = format!("dla{pid}");
            let arg = path.display().to_string();
            t.need(Cleanup::Rm(path.clone()));
            let stop = Cleanup::Run(vec![
                p.exe.display().to_string(),
                pid.to_string(),
                "JFR.stop".into(),
                format!("name={name}"),
            ]);
            t.need(stop.clone());
            let start = call(
                &p.exe,
                pid,
                &[
                    "JFR.start",
                    &format!("name={name}"),
                    "settings=profile",
                    &format!("duration={secs}s"),
                    &format!("filename={arg}"),
                ],
                Duration::from_secs(60),
            )
            .map_err(failed)?;
            if start.contains("Could not") || start.contains("Unable") {
                return Err(AttachError::failed(
                    format!("JFR did not start: {}", first_error_line("", &start)),
                    "check the JVM supports JFR (JDK 11 or later)",
                ));
            }
            let limit = Duration::from_secs(secs + 30);
            let mut size =
                wait_stable(&path, Duration::from_secs(secs + 3), Duration::from_secs(1));
            if !same_process(&req.target) {
                t.forget_runs();
                return Err(AttachError::failed(
                    format!("process {pid} exited (or its pid was reused) during the recording"),
                    "repeat the command on a running JVM",
                ));
            }
            if size.is_none() {
                // The recording ended but the file is not there yet: ask for the dump.
                let _ = call(
                    &p.exe,
                    pid,
                    &[
                        "JFR.dump",
                        &format!("name={name}"),
                        &format!("filename={arg}"),
                    ],
                    Duration::from_secs(60),
                );
                size = wait_stable(
                    &path,
                    limit.saturating_sub(started.elapsed()),
                    Duration::from_secs(1),
                );
            }
            let Some(bytes) = size else {
                return Err(AttachError::failed(
                    format!("the JFR file {} did not appear", path.display()),
                    "repeat the command with a longer --duration",
                ));
            };
            if !same_process(&req.target) {
                t.forget_runs();
                return Err(AttachError::failed(
                    format!("process {pid} exited (or its pid was reused) before JFR.stop"),
                    "repeat the command on a running JVM",
                ));
            }
            let _ = call(
                &p.exe,
                pid,
                &["JFR.stop", &format!("name={name}")],
                Duration::from_secs(20),
            );
            t.keep(&Cleanup::Rm(path.clone()));
            t.keep(&stop);
            let mut summary = Map::new();
            summary.insert("bytes".into(), json!(bytes));
            summary.insert("recording_s".into(), json!(secs));
            if let Some(jfr) = &p.jfr {
                let mut c = Command::new(jfr);
                c.arg("summary").arg(&path);
                if let Ok(r) = exec::run(c, Duration::from_secs(60))
                    && r.success
                {
                    const WANTED: [&str; 5] = [
                        "jdk.ObjectAllocationSample",
                        "jdk.ExecutionSample",
                        "jdk.GarbageCollection",
                        "jdk.JavaMonitorEnter",
                        "jdk.NativeMethodSample",
                    ];
                    let events: Map<String, Value> = parse_jfr_summary(&r.stdout, usize::MAX)
                        .into_iter()
                        .filter(|e| WANTED.contains(&e.name.as_str()))
                        .map(|e| (e.name.replace("jdk.", ""), json!(e.count)))
                        .collect();
                    summary.insert("events".into(), Value::Object(events));
                }
            }
            done.summary = Value::Object(summary);
            done.limitations.push(
                "the recording holds allocation samples (class and stack), not a heap census"
                    .into(),
            );
        }
    }
    if !done.files.iter().any(|f| f == &plan.file) && path.is_file() {
        done.files.insert(0, plan.file.clone());
    }
    done.durations = dur;
    Ok(done)
}

/// (containers, threads) of a JSON thread dump.
pub(crate) fn count_threads(json_text: &str) -> (usize, usize) {
    let Ok(v) = serde_json::from_str::<Value>(json_text) else {
        return (0, 0);
    };
    let containers = v["threadDump"]["threadContainers"].as_array();
    let c = containers.map_or(0, Vec::len);
    let n = containers.map_or(0, |a| {
        a.iter()
            .map(|c| c["threads"].as_array().map_or(0, Vec::len))
            .sum()
    });
    (c, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_jvm_library_in_maps_is_recognised() {
        let jvm = "7f00-7f01 r-xp 0 08:01 5 /usr/lib/jvm/java-21/lib/server/libjvm.so\n";
        assert!(maps_show_jvm(jvm));
        assert!(maps_show_jvm(
            "1-2 r-xp 0 0:0 0 C:\\jdk\\bin\\server/jvm.dll\n"
        ));
        assert!(!maps_show_jvm(
            "7f00-7f01 r-xp 0 08:01 5 /usr/lib/libc.so.6\n"
        ));
        assert!(!maps_show_jvm(
            "7f00-7f01 r-xp 0 08:01 5 /opt/libjvmti-fake.txt\n"
        ));
    }

    #[test]
    fn thread_dump_json_is_counted() {
        let j = r#"{"threadDump":{"threadContainers":[{"container":"root","threads":[{},{}]},{"threads":[{}]}]}}"#;
        assert_eq!(count_threads(j), (2, 3));
        assert_eq!(count_threads("nope"), (0, 0));
    }

    #[test]
    fn default_files_have_an_extension_each() {
        assert_eq!(default_file(Op::HeapDump), ("heap", "hprof"));
        assert_eq!(default_file(Op::Jfr), ("recording", "jfr"));
    }
}
