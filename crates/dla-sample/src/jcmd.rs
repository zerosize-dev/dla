//! The `jcmd` backend: `GC.heap_info`, `VM.native_memory summary` and `PerfCounter.print`
//! against a local JVM.
//!
//! Columns: `jvm.heap_used`, `jvm.heap_committed`, `jvm.nonheap_used` (Metaspace used),
//! `jvm.nmt_committed` (only when the JVM runs with `-XX:NativeMemoryTracking`),
//! `jvm.threads` (`java.threads.live`), `jvm.classes_loaded` (the classes loaded now: the
//! cumulative counter minus the unloads) and `jvm.classes_unloaded`. None of them is read after a
//! forced collection: `jcmd` does not force one.

use std::{collections::BTreeMap, path::PathBuf, process::Command, thread, time::Duration};

use crate::{
    backend::{Backend, BackendError, OpenCtx, Reading},
    exec,
};

/// `jcmd` starts a JVM per call; polling faster than this only burns CPU.
const MIN_INTERVAL: Duration = Duration::from_secs(1);
/// Longest one `jcmd` call may take.
const CALL_TIMEOUT: Duration = Duration::from_secs(15);
/// Environment variables that make `jcmd` itself print or behave differently.
const JAVA_ENV: [&str; 3] = ["JAVA_TOOL_OPTIONS", "JDK_JAVA_OPTIONS", "_JAVA_OPTIONS"];

/// Heap figures from `GC.heap_info`, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeapInfo {
    /// Bytes in use.
    pub used: u64,
    /// Bytes committed (the heap's current total size).
    pub committed: u64,
    /// Metaspace bytes in use, when the output has the line.
    pub metaspace_used: Option<u64>,
}

/// What `VM.native_memory summary` said.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nmt {
    /// Native memory tracking is off in the target JVM.
    Disabled,
    /// Total committed bytes.
    Committed(u64),
}

/// A size such as `262144K`, `20M`, `4G`, `123KB`, `7B` or `512` in bytes (K is 1024).
pub fn parse_size(token: &str) -> Option<u64> {
    let t = token.trim().trim_end_matches([',', ';']);
    let split = t.find(|c: char| !c.is_ascii_digit()).unwrap_or(t.len());
    let (digits, unit) = t.split_at(split);
    let n: u64 = digits.parse().ok()?;
    let mult: u64 = match unit.to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kb" => 1 << 10,
        "m" | "mb" => 1 << 20,
        "g" | "gb" => 1 << 30,
        "t" | "tb" => 1 << 40,
        _ => return None,
    };
    n.checked_mul(mult)
}

const KEYWORDS: [&str; 4] = ["total", "used", "committed", "capacity"];

/// `keyword -> bytes` for the first mention of each keyword in a line, where the size
/// either follows (`total 221184K`) or precedes (`8192K committed`) the keyword.
fn keyword_sizes(line: &str) -> BTreeMap<&'static str, u64> {
    let words: Vec<&str> = line
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|w| !w.is_empty())
        .collect();
    let size_at = |i: Option<usize>| i.and_then(|j| words.get(j)).and_then(|s| parse_size(s));
    let is_kw = |w: &str| KEYWORDS.contains(&w.to_ascii_lowercase().as_str());
    // `total 221184K, used 2302K` puts the size after the keyword, `8192K committed, 2048K
    // used` before it; the first keyword of the line says which.
    let size_first = words
        .iter()
        .position(|w| is_kw(w))
        .is_some_and(|i| size_at(i.checked_sub(1)).is_some());
    let mut out = BTreeMap::new();
    for (i, w) in words.iter().enumerate() {
        let Some(kw) = KEYWORDS.iter().find(|k| **k == w.to_ascii_lowercase()) else {
            continue;
        };
        // "max capacity 4096M" is the limit, not the current size.
        if i > 0 && words[i - 1].eq_ignore_ascii_case("max") {
            continue;
        }
        let v = if size_first {
            size_at(i.checked_sub(1))
        } else {
            size_at(Some(i + 1))
        };
        if let Some(v) = v {
            out.entry(*kw).or_insert(v);
        }
    }
    out
}

/// Parses `GC.heap_info` for the G1, Parallel, Serial, ZGC, Shenandoah and Epsilon layouts.
/// Lines such as `Picked up JAVA_TOOL_OPTIONS: ...` and the leading `<pid>:` are ignored.
pub fn parse_heap_info(text: &str) -> Option<HeapInfo> {
    let mut info = HeapInfo::default();
    let mut found = false;
    let mut sized_next = false;
    for raw in text.lines() {
        let line = raw.trim().to_ascii_lowercase();
        if line.is_empty() || line.starts_with("picked up ") {
            continue;
        }
        let generic = sized_next;
        sized_next = false;
        if generic {
            let k = keyword_sizes(&line);
            if let (Some(&used), Some(&committed)) =
                (k.get("used"), k.get("committed").or_else(|| k.get("total")))
            {
                info.used += used;
                info.committed += committed;
                found = true;
                continue;
            }
        }
        if line.starts_with("metaspace") {
            info.metaspace_used = keyword_sizes(&line).get("used").copied();
            continue;
        }
        if line.starts_with("shenandoah heap") || line.starts_with("epsilon heap") {
            sized_next = true;
            continue;
        }
        const HEAPS: [&str; 5] = [
            "garbage-first heap",
            "psyounggen",
            "paroldgen",
            "def new generation",
            "tenured generation",
        ];
        const SINGLE: [&str; 1] = ["zheap"];
        if HEAPS.iter().chain(&SINGLE).any(|h| line.starts_with(h)) {
            let k = keyword_sizes(&line);
            let size = k.get("total").or_else(|| k.get("capacity"));
            if let (Some(&used), Some(&committed)) = (k.get("used"), size) {
                info.used += used;
                info.committed += committed;
                found = true;
            }
        }
    }
    found.then_some(info)
}

/// Parses `VM.native_memory summary`.
pub fn parse_nmt(text: &str) -> Option<Nmt> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("native memory tracking is not enabled")
        || lower.contains("not enabled")
        || lower.contains("not been enabled")
    {
        return Some(Nmt::Disabled);
    }
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("Total:"))?;
    let (_, rest) = line.split_once("committed=")?;
    let token = rest.split([',', ' ']).next()?;
    parse_size(token).map(Nmt::Committed)
}

/// Parses `PerfCounter.print` into `jvm.threads`, `jvm.classes_loaded` and
/// `jvm.classes_unloaded`.
///
/// `java.cls.loadedClasses` counts every class ever loaded that did not come from the CDS
/// archive, and `java.cls.sharedLoadedClasses` those that did (checked on JDK 21: an idle JVM
/// reads 7 and 447, a process that defined and dropped 30 000 classes read 30 119 and 737 and
/// `ClassLoadingMXBean.getLoadedClassCount()` read 856 = 30 119 + 737 - 30 000). The harness column
/// `jvm.classes_loaded` is the number loaded now; so that both sources mean the same, this column
/// is the sum of both cumulative counts minus both unload counts, and the unloads are their own column.
pub fn parse_perf_counters(text: &str) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    let (mut loaded, mut unloaded) = (None, None);
    let (mut shared_loaded, mut shared_unloaded) = (0.0, 0.0);
    for line in text.lines() {
        let Some((key, value)) = line.trim().split_once('=') else {
            continue;
        };
        let Ok(v) = value.trim().parse::<f64>() else {
            continue;
        };
        match key {
            "java.threads.live" => {
                out.insert("jvm.threads".to_owned(), v);
            }
            "java.cls.loadedClasses" => loaded = Some(v),
            "java.cls.unloadedClasses" => unloaded = Some(v),
            "java.cls.sharedLoadedClasses" => shared_loaded = v,
            "java.cls.sharedUnloadedClasses" => shared_unloaded = v,
            _ => {}
        }
    }
    if let Some(l) = loaded {
        let now = l + shared_loaded - unloaded.unwrap_or(0.0) - shared_unloaded;
        out.insert("jvm.classes_loaded".to_owned(), now);
    }
    if let Some(u) = unloaded {
        out.insert("jvm.classes_unloaded".to_owned(), u + shared_unloaded);
    }
    out
}

/// A `jcmd` attached to one JVM.
#[derive(Debug)]
struct Jcmd {
    exe: PathBuf,
    pid: u32,
    nmt: NmtState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NmtState {
    Unknown,
    On,
    Off,
}

/// Finds `jcmd` and returns the backend.
pub fn open(ctx: &OpenCtx) -> Result<Box<dyn Backend>, BackendError> {
    let mut extra = Vec::new();
    if let Some(home) = std::env::var_os("JAVA_HOME") {
        extra.push(PathBuf::from(home).join("bin"));
    }
    let exe = exec::find_exe("jcmd", &extra).ok_or_else(|| {
        BackendError::missing(
            "jcmd is not installed, so JVM counters are unavailable",
            "install a JDK (jcmd ships with the JDK, not the JRE) and put its bin directory on PATH",
        )
    })?;
    Ok(Box::new(Jcmd {
        exe,
        pid: ctx.pid,
        nmt: NmtState::Unknown,
    }))
}

impl Jcmd {
    fn call(&self, command: &str) -> Result<String, String> {
        let mut cmd = Command::new(&self.exe);
        for var in JAVA_ENV {
            cmd.env_remove(var);
        }
        cmd.arg(self.pid.to_string()).arg(command);
        let run = exec::run(cmd, CALL_TIMEOUT).map_err(|e| format!("jcmd {command}: {e}"))?;
        if run.success {
            return Ok(run.stdout);
        }
        let detail = [&run.stderr, &run.stdout]
            .iter()
            .flat_map(|s| s.lines())
            .map(str::trim)
            .find(|l| !l.is_empty() && !l.starts_with("Picked up ") && !l.ends_with(':'))
            .unwrap_or("it exited with an error")
            .to_owned();
        Err(format!("jcmd {command}: {detail}"))
    }
}

impl Backend for Jcmd {
    fn name(&self) -> &'static str {
        "jcmd"
    }

    fn runtime(&self) -> &'static str {
        "jvm"
    }

    fn min_interval(&self) -> Duration {
        MIN_INTERVAL
    }

    fn poll(&mut self) -> Result<Reading, BackendError> {
        let want_nmt = self.nmt != NmtState::Off;
        // Each jcmd is its own JVM start; run them side by side.
        let (heap, perf, nmt) = thread::scope(|s| {
            let heap = s.spawn(|| self.call("GC.heap_info"));
            let perf = s.spawn(|| self.call("PerfCounter.print"));
            let nmt = want_nmt.then(|| s.spawn(|| self.call("VM.native_memory summary")));
            (
                heap.join()
                    .unwrap_or_else(|_| Err("jcmd thread panicked".into())),
                perf.join()
                    .unwrap_or_else(|_| Err("jcmd thread panicked".into())),
                nmt.map(|h| {
                    h.join()
                        .unwrap_or_else(|_| Err("jcmd thread panicked".into()))
                }),
            )
        });
        let heap = heap.map_err(BackendError::blocked)?;
        let info = parse_heap_info(&heap).ok_or_else(|| {
            BackendError::blocked(
                "jcmd GC.heap_info output was not recognised (unknown collector layout)",
            )
        })?;
        let mut reading = Reading::default();
        reading
            .values
            .insert("jvm.heap_used".into(), info.used as f64);
        reading
            .values
            .insert("jvm.heap_committed".into(), info.committed as f64);
        if let Some(m) = info.metaspace_used {
            reading.values.insert("jvm.nonheap_used".into(), m as f64);
        }
        match perf {
            Ok(text) => reading.values.extend(parse_perf_counters(&text)),
            Err(e) => reading.notes.push(e),
        }
        match nmt {
            Some(Ok(text)) => match parse_nmt(&text) {
                Some(Nmt::Committed(b)) => {
                    self.nmt = NmtState::On;
                    reading.values.insert("jvm.nmt_committed".into(), b as f64);
                }
                Some(Nmt::Disabled) => {
                    self.nmt = NmtState::Off;
                    reading.notes.push(
                        "native memory tracking is off in the target JVM (start it with -XX:NativeMemoryTracking=summary for jvm.nmt_committed)"
                            .into(),
                    );
                }
                None => reading
                    .notes
                    .push("VM.native_memory summary output was not recognised".into()),
            },
            Some(Err(e)) => {
                // "not enabled" can also arrive as a non-zero exit.
                if e.to_ascii_lowercase().contains("not enabled") {
                    self.nmt = NmtState::Off;
                    reading.notes.push(
                        "native memory tracking is off in the target JVM (start it with -XX:NativeMemoryTracking=summary for jvm.nmt_committed)"
                            .into(),
                    );
                } else {
                    reading.notes.push(e);
                }
            }
            None => {}
        }
        Ok(reading)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const G1: &str = "Picked up JAVA_TOOL_OPTIONS: -Dx=y\n3905:\n garbage-first heap   total 221184K, used 2302K [0x000000072a400000, 0x0000000800000000)\n  region size 2048K, 1 young (2048K), 0 survivors (0K)\n Metaspace       used 78K, committed 320K, reserved 1114112K\n  class space    used 5K, committed 128K, reserved 1048576K\n";
    const PARALLEL: &str = "3988:\n PSYoungGen      total 64000K, used 5529K [0x00000007b8c00000, 0x00000007bd300000, 0x0000000800000000)\n  eden space 55296K, 10% used [0x00000007b8c00000,0x00000007b91666a0,0x00000007bc200000)\n  from space 8704K, 0% used [0x00000007bca80000,0x00000007bca80000,0x00000007bd300000)\n  to   space 8704K, 0% used [0x00000007bc200000,0x00000007bc200000,0x00000007bca80000)\n ParOldGen       total 146432K, used 1073K [0x000000072a400000, 0x0000000733300000, 0x00000007b8c00000)\n  object space 146432K, 0% used [0x000000072a400000,0x000000072a50c710,0x0000000733300000)\n Metaspace       used 78K, committed 320K, reserved 1114112K\n";
    const SERIAL: &str = "12:\n def new generation   total 9216K, used 2132K [0x00000000fec00000, 0x00000000ff600000, 0x00000000ff600000)\n  eden space 8192K,  26% used [0x00000000fec00000, 0x00000000fee15240, 0x00000000ff400000)\n tenured generation   total 20480K, used 0K [0x00000000ff600000, 0x0000000100a00000, 0x0000000100a00000)\n   the space 20480K,   0% used [0x00000000ff600000, 0x00000000ff600000, 0x00000000ff800200, 0x0000000100a00000)\n Metaspace       used 1000K, committed 1024K, reserved 1056768K\n";
    const ZGC: &str = "7:\n ZHeap           used 20M, capacity 64M, max capacity 4096M\n Metaspace       used 10K, committed 64K, reserved 1M\n";
    const SHENANDOAH: &str = "7:\nShenandoah Heap\n 65536K max, 65536K soft max, 8192K committed, 2048K used\n 10 x 1024K regions\n";

    #[test]
    fn heap_info_g1() {
        let h = parse_heap_info(G1).unwrap();
        assert_eq!(h.used, 2302 * 1024);
        assert_eq!(h.committed, 221_184 * 1024);
        assert_eq!(h.metaspace_used, Some(78 * 1024));
    }

    #[test]
    fn heap_info_other_collectors() {
        let h = parse_heap_info(PARALLEL).unwrap();
        assert_eq!(h.used, (5529 + 1073) * 1024);
        assert_eq!(h.committed, (64_000 + 146_432) * 1024);
        let h = parse_heap_info(SERIAL).unwrap();
        assert_eq!((h.used, h.committed), (2132 * 1024, (9216 + 20_480) * 1024));
        let h = parse_heap_info(ZGC).unwrap();
        assert_eq!((h.used, h.committed), (20 << 20, 64 << 20));
        let h = parse_heap_info(SHENANDOAH).unwrap();
        assert_eq!((h.used, h.committed), (2048 * 1024, 8192 * 1024));
        assert!(parse_heap_info("1:\nnothing here\n").is_none());
    }

    #[test]
    fn nmt_on_and_off() {
        let on = "3905:\n\nNative Memory Tracking:\n\n(Omitting categories weighting less than 1KB)\n\nTotal: reserved=5012208KB, committed=292172KB\n       malloc: 12072KB #5965\n";
        assert_eq!(parse_nmt(on), Some(Nmt::Committed(292_172 * 1024)));
        assert_eq!(
            parse_nmt("3988:\nNative memory tracking is not enabled\n"),
            Some(Nmt::Disabled)
        );
        let mb = "Total: reserved=5GB, committed=2MB\n";
        assert_eq!(parse_nmt(mb), Some(Nmt::Committed(2 << 20)));
        assert_eq!(parse_nmt("garbage"), None);
    }

    #[test]
    fn perf_counters() {
        let text = "3905:\njava.ci.totalTime=919432\njava.cls.loadedClasses=543\njava.property.java.class.path=\".\"\njava.rt.vmArgs=\"-Da=b -XX:x=1\"\njava.threads.live=7\njava.threads.daemon=6\n";
        let m = parse_perf_counters(text);
        assert_eq!(m["jvm.threads"], 7.0);
        assert_eq!(m["jvm.classes_loaded"], 543.0);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn the_cumulative_class_counter_becomes_the_number_loaded_now() {
        // Read on JDK 21 from a process that defined and dropped 30 000 hidden classes: the
        // shared (CDS) classes are not in `loadedClasses`.
        let text = "993:\njava.cls.loadedClasses=30121\njava.cls.sharedLoadedClasses=736\njava.cls.unloadedClasses=30000\njava.threads.live=12\n";
        let m = parse_perf_counters(text);
        assert_eq!(m["jvm.classes_loaded"], 121.0 + 736.0);
        assert_eq!(m["jvm.classes_unloaded"], 30_000.0);
        assert_eq!(m["jvm.threads"], 12.0);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn sizes() {
        assert_eq!(parse_size("10K"), Some(10_240));
        assert_eq!(parse_size("3MB,"), Some(3 << 20));
        assert_eq!(parse_size("7"), Some(7));
        assert_eq!(parse_size("x"), None);
        assert_eq!(parse_size("5%"), None);
    }
}
