//! Raw captures of `dla harness run --capture` and `--tool`.
//!
//! The JVM agent (`harness/jvm/DlaCapture.java`) writes heap dumps, class histograms, a flight
//! recording and the non-heap reports of a measured process into
//! `out/<label>/capture/<pid>/`; `--tool valgrind|heaptrack` runs a point under the tool and
//! leaves its output in `out/<label>/<tool>/`. This module chooses which points capture what,
//! lists the files that exist after the run (with size, capture time and the agent's cost
//! figures) and builds the `dla summarize` commands that read them.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use dla_core::{
    harness::{CaptureFile, CaptureKind, DumpCapture},
    runtime::Runtime,
};

/// Directory (inside `out/<label>/`) the JVM agent writes to.
pub const DIR: &str = "capture";
/// Heap dumps a driver takes by default.
pub const DEFAULT_DUMPS: u32 = 4;
/// Points a tool run (valgrind, heaptrack) measures in command mode.
pub const TOOL_POINTS: u32 = 3;
/// Bytes the `summary` of `dla harness run` may take in all (the rest of the 2048-byte
/// output is the envelope: effects, artifacts, evidence).
pub const SUMMARY_BUDGET: usize = 1120;
/// Most class histograms one printed command lists (the full command is in `captures.json`).
pub const COMPACT_HISTOGRAMS: usize = 4;

/// Which captures were asked for (`--capture heap,histogram,jfr,nonheap`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaptureSet {
    /// HPROF heap dumps.
    pub heap: bool,
    /// Full class histogram text.
    pub histogram: bool,
    /// Flight recording with old-object samples.
    pub jfr: bool,
    /// Thread dump, class loader statistics, native memory (JVM); active resources by
    /// creation site (Node).
    pub nonheap: bool,
    /// Sampling allocation profile of the objects still alive (Node `.heapprofile`).
    pub alloc: bool,
}

impl CaptureSet {
    /// Nothing is captured.
    pub fn is_empty(&self) -> bool {
        !(self.heap || self.histogram || self.jfr || self.nonheap || self.alloc)
    }

    /// The kinds of `self` that are also in `other`.
    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            heap: self.heap && other.heap,
            histogram: self.histogram && other.histogram,
            jfr: self.jfr && other.jfr,
            nonheap: self.nonheap && other.nonheap,
            alloc: self.alloc && other.alloc,
        }
    }

    /// The kinds, in the order they are listed.
    pub fn names(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        for (on, n) in [
            (self.heap, "heap"),
            (self.histogram, "histogram"),
            (self.jfr, "jfr"),
            (self.nonheap, "nonheap"),
            (self.alloc, "alloc"),
        ] {
            if on {
                v.push(n);
            }
        }
        v
    }

    /// The `DLA_CAPTURE` value for one process. `full` is false for a point that only
    /// writes the cheap histogram (command mode, points outside the selection).
    pub fn env_value(&self, full: bool) -> String {
        let mut v = Vec::new();
        if self.histogram {
            v.push("histogram");
        }
        if full {
            if self.heap {
                v.push("heap");
            }
            if self.jfr {
                v.push("jfr");
            }
            if self.nonheap {
                v.push("nonheap");
            }
            if self.alloc {
                v.push("alloc");
            }
        }
        v.join(",")
    }
}

impl FromStr for CaptureSet {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        let mut set = Self::default();
        for part in s.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            match part {
                "heap" => set.heap = true,
                "histogram" => set.histogram = true,
                "jfr" => set.jfr = true,
                "nonheap" | "non-heap" => set.nonheap = true,
                "alloc" => set.alloc = true,
                "all" => {
                    set = Self {
                        heap: true,
                        histogram: true,
                        jfr: true,
                        nonheap: true,
                        alloc: true,
                    }
                }
                "none" => set = Self::default(),
                other => {
                    return Err(format!(
                        "{other:?} is not a capture kind (heap, histogram, jfr, nonheap, alloc, all, none)"
                    ));
                }
            }
        }
        Ok(set)
    }
}

/// The capture kinds the agent of `rt` writes (`--capture` asks for more than a runtime has
/// at its own risk: the run drops the rest and says so).
pub fn supported(rt: Runtime) -> CaptureSet {
    match rt {
        Runtime::Jvm => CaptureSet {
            heap: true,
            histogram: true,
            jfr: true,
            nonheap: true,
            alloc: false,
        },
        Runtime::Node => CaptureSet {
            heap: true,
            histogram: false,
            jfr: false,
            nonheap: true,
            alloc: true,
        },
        // Go: heap profiles (alloc) and goroutine profiles with the goroutineleak profile
        // (nonheap).
        Runtime::Go => CaptureSet {
            heap: false,
            histogram: false,
            jfr: false,
            nonheap: true,
            alloc: true,
        },
        // Python: type counters (histogram), tracemalloc and memray stacks (alloc), threads,
        // tasks and ResourceWarnings by creation site (nonheap).
        Runtime::Python => CaptureSet {
            heap: false,
            histogram: true,
            jfr: false,
            nonheap: true,
            alloc: true,
        },
        _ => CaptureSet::default(),
    }
}

/// A native tool a point can run under (`--tool`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTool {
    /// `valgrind --tool=memcheck` with XML output.
    Valgrind,
    /// `heaptrack` with a `heaptrack_print` report.
    Heaptrack,
}

impl NativeTool {
    /// The tool name as the CLI and the coverage rows spell it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Valgrind => "valgrind",
            Self::Heaptrack => "heaptrack",
        }
    }

    /// Binary to find on PATH or in the tools directory.
    pub fn binary(self) -> &'static str {
        self.as_str()
    }

    /// The command that installs the tool.
    pub fn install_hint(self) -> &'static str {
        match self {
            Self::Valgrind => {
                "install valgrind (Linux): apt-get install valgrind, or put it in $DLA_DATA_DIR/tools/bin"
            }
            Self::Heaptrack => {
                "install heaptrack (Linux): apt-get install heaptrack, or put it in $DLA_DATA_DIR/tools/bin"
            }
        }
    }

    /// The capture kind of the files the tool writes.
    pub fn kind(self) -> CaptureKind {
        match self {
            Self::Valgrind => CaptureKind::Valgrind,
            Self::Heaptrack => CaptureKind::Heaptrack,
        }
    }
}

impl FromStr for NativeTool {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "valgrind" => Ok(Self::Valgrind),
            "heaptrack" => Ok(Self::Heaptrack),
            other => Err(format!("{other:?} is not valgrind or heaptrack")),
        }
    }
}

/// The points `k` (1-based, of `total`) that take the heavy captures in command mode: the
/// end of the warm-up, the last point and evenly spaced points between, at most `want`.
pub fn select_points(total: u32, warmup: u32, want: u32) -> BTreeSet<u32> {
    let mut set = BTreeSet::new();
    if total == 0 {
        return set;
    }
    let want = want.max(1);
    if total <= want {
        set.extend(1..=total);
        return set;
    }
    let first = warmup.clamp(1, total);
    set.insert(first);
    set.insert(total);
    let interior = want.saturating_sub(2);
    for j in 1..=interior {
        let at =
            f64::from(first) + f64::from(total - first) * f64::from(j) / f64::from(interior + 1);
        set.insert(at.round() as u32);
    }
    set
}

/// What a file in a capture directory is: kind, sequence number and role.
pub fn classify(name: &str) -> Option<(CaptureKind, u32, &'static str)> {
    let seq_of = |prefix: &str, suffix: &str| -> Option<u32> {
        name.strip_prefix(prefix)?
            .strip_suffix(suffix)?
            .parse()
            .ok()
    };
    if let Some(n) = seq_of("heap-", ".hprof") {
        return Some((CaptureKind::Heap, n, "heap dump (HPROF)"));
    }
    if let Some(n) = seq_of("heap-", ".heapsnapshot") {
        return Some((CaptureKind::Heap, n, "heap snapshot (v8)"));
    }
    if let Some(n) = seq_of("alloc-", ".heapprofile") {
        return Some((
            CaptureKind::Alloc,
            n,
            "sampling heap profile (live objects)",
        ));
    }
    // Go: go-<n>.<what>.<ext>
    if let Some(n) = seq_of("go-", ".heap.pb.gz") {
        return Some((
            CaptureKind::Alloc,
            n,
            "heap profile (in-use bytes by stack, after GC)",
        ));
    }
    if let Some(n) = seq_of("go-", ".goroutine.pb.gz") {
        return Some((CaptureKind::NonHeap, n, "goroutine profile (protobuf)"));
    }
    if let Some(n) = seq_of("go-", ".goroutine.txt") {
        return Some((
            CaptureKind::NonHeap,
            n,
            "goroutine dump (debug=2, created-by frames)",
        ));
    }
    if let Some(n) = seq_of("go-", ".goroutineleak.txt") {
        return Some((CaptureKind::NonHeap, n, "goroutineleak profile (debug=2)"));
    }
    // Python: py-<n>.<what>.json
    if let Some(n) = seq_of("py-", ".alloc.json") {
        return Some((
            CaptureKind::Alloc,
            n,
            "tracemalloc live bytes by allocation stack",
        ));
    }
    if let Some(n) = seq_of("py-", ".memray.json") {
        return Some((
            CaptureKind::Alloc,
            n,
            "memray allocations still live at the end",
        ));
    }
    if let Some(n) = seq_of("py-", ".types.json") {
        return Some((CaptureKind::Histogram, n, "python object counts by type"));
    }
    if let Some(n) = seq_of("py-", ".nonheap.json") {
        return Some((
            CaptureKind::NonHeap,
            n,
            "threads, asyncio tasks and resource warnings by creation site",
        ));
    }
    if let Some(n) = seq_of("histogram-", ".txt") {
        return Some((CaptureKind::Histogram, n, "class histogram"));
    }
    match name {
        "recording.jfr" => Some((
            CaptureKind::Jfr,
            1,
            "flight recording with old-object samples",
        )),
        "threads.json" => Some((CaptureKind::NonHeap, 1, "thread dump (json)")),
        "classloader-stats.txt" => Some((CaptureKind::NonHeap, 2, "class loader statistics")),
        "nmt-summary.txt" => Some((CaptureKind::NonHeap, 3, "native memory summary")),
        "nmt-diff.txt" => Some((
            CaptureKind::NonHeap,
            4,
            "native memory change since baseline",
        )),
        "resources.json" => Some((
            CaptureKind::NonHeap,
            5,
            "active resources by type and creation site",
        )),
        "scripts.json" => Some((
            CaptureKind::NonHeap,
            6,
            "script id to url map (sidecar of the heap snapshots)",
        )),
        "valgrind.xml" => Some((CaptureKind::Valgrind, 1, "valgrind memcheck XML")),
        "vgdb-1-leak.txt" => Some((CaptureKind::Valgrind, 2, "vgdb leak_check answer")),
        "vgdb-2-blocks.txt" => Some((CaptureKind::Valgrind, 3, "vgdb block_list answer")),
        "vgdb-3-who.txt" => Some((CaptureKind::Valgrind, 4, "vgdb who_points_at walk")),
        "heaptrack.txt" => Some((CaptureKind::Heaptrack, 1, "heaptrack_print report")),
        "heaptrack-diff.txt" => Some((
            CaptureKind::Heaptrack,
            2,
            "heaptrack_print diff, first to last point",
        )),
        _ => None,
    }
}

/// RFC 3339 UTC text of a file time, to whole seconds.
pub fn rfc3339(t: std::time::SystemTime) -> String {
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs()) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// The agent's own account of the captures of one point: file name to (record, x).
#[derive(Debug, Clone, Default)]
pub struct AgentNotes {
    /// `(x of the record, capture entries)` for every record that listed captures.
    pub records: Vec<(Option<f64>, Vec<DumpCapture>)>,
}

impl AgentNotes {
    fn find(&self, file: &str) -> Option<(Option<f64>, &DumpCapture)> {
        self.records.iter().find_map(|(x, caps)| {
            caps.iter()
                .find(|c| c.file.as_deref() == Some(file))
                .map(|c| (*x, c))
        })
    }

    /// Entries the agent reported as failed: `(kind, error)`.
    pub fn errors(&self) -> Vec<(String, String)> {
        self.records
            .iter()
            .flat_map(|(_, caps)| caps)
            .filter_map(|c| {
                Some((
                    c.kind.clone().unwrap_or_else(|| "capture".to_owned()),
                    c.error.clone()?,
                ))
            })
            .collect()
    }
}

/// Removes the capture directories of processes other than `keep` (the test and tool JVMs a
/// build script starts) and returns the directory of the measured one.
fn prune(base: &Path, keep: Option<u32>) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(base) else {
        return Vec::new();
    };
    let mut kept = Vec::new();
    for e in rd.flatten() {
        let path = e.path();
        if !path.is_dir() {
            continue;
        }
        let pid: Option<u32> = e.file_name().to_string_lossy().parse().ok();
        if keep.is_some() && pid != keep {
            let _ = fs::remove_dir_all(&path);
        } else {
            kept.push(path);
        }
    }
    kept.sort();
    kept
}

/// Lists the capture files of one point after its process ended. `label_dir` is
/// `out/<label>`, `run_rel` the run-relative path of that directory. Directories of other
/// processes than `pid` are deleted when `pid` is known.
///
/// A heap dump or histogram is listed only when the agent's note names the file and has no
/// error: a process killed during `dumpHeap` leaves a truncated HPROF with no note. The flight
/// recording has no note (the JVM writes it at exit), so it is listed only when the process
/// ended by itself (`killed` false). The second value names the files left out.
pub fn gather(
    label_dir: &Path,
    run_rel: &str,
    label: &str,
    pid: Option<u32>,
    notes: &AgentNotes,
    x_default: Option<f64>,
    killed: bool,
) -> (Vec<CaptureFile>, Vec<String>) {
    let base = label_dir.join(DIR);
    let mut out = Vec::new();
    let mut left_out = Vec::new();
    for dir in prune(&base, pid) {
        let pid_name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        let mut files: Vec<_> = rd.flatten().collect();
        files.sort_by_key(|e| e.file_name());
        for e in files {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some((kind, seq, role)) = classify(&name) else {
                continue;
            };
            let Ok(md) = e.metadata() else { continue };
            if !md.is_file() || md.len() == 0 {
                // An empty file is the leftover of a failed capture.
                continue;
            }
            let note = notes.find(&name);
            let confirmed = match kind {
                CaptureKind::Heap | CaptureKind::Histogram | CaptureKind::Alloc => {
                    note.is_some_and(|(_, c)| c.error.is_none())
                }
                CaptureKind::Jfr => !killed,
                _ => true,
            };
            if !confirmed {
                left_out.push(format!("{pid_name}/{name}"));
                continue;
            }
            out.push(CaptureFile {
                kind,
                path: format!("{run_rel}/{DIR}/{pid_name}/{name}"),
                label: label.to_owned(),
                seq,
                x: note.and_then(|(x, _)| x).or(x_default),
                role: Some(
                    note.and_then(|(_, c)| c.role.clone())
                        .unwrap_or_else(|| role.to_owned()),
                ),
                bytes: md.len(),
                ms: note.and_then(|(_, c)| c.ms),
                captured_at: rfc3339(md.modified().unwrap_or(std::time::UNIX_EPOCH)),
            });
        }
    }
    (out, left_out)
}

/// The `dla summarize` commands that read `files`, most useful first. `pointer` turns a
/// run-relative path into the path printed in the command.
pub fn summarize_commands(
    files: &[CaptureFile],
    pointer: &dyn Fn(&str) -> String,
    compact: bool,
) -> Vec<String> {
    // Group by the series the files belong to: one group per driver label; command mode
    // has one file set per point, so the labels are merged into one group.
    let command_mode = files.iter().all(|f| f.label.ends_with("-native"));
    let mut groups: Vec<(String, Vec<&CaptureFile>)> = Vec::new();
    for f in files {
        let key = if command_mode {
            "native".to_owned()
        } else {
            f.label.clone()
        };
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push(f),
            None => groups.push((key, vec![f])),
        }
    }
    // Fresh keys first: the repeated series is the plateau control.
    groups.sort_by_key(|(k, _)| k.ends_with("repeated"));
    let order = |f: &&CaptureFile| (f.x.map_or(0, |x| x as u64), f.label.clone(), f.seq);
    let mut cmds = Vec::new();
    let mut push = |op: &str, kind: CaptureKind, tail: &[&str], group: &[&CaptureFile]| {
        let mut sel: Vec<&CaptureFile> = group
            .iter()
            .copied()
            .filter(|f| {
                // Python files have formats of their own: they are listed by their tail only.
                let python = f
                    .path
                    .rsplit('/')
                    .next()
                    .is_some_and(|n| n.starts_with("py-"));
                f.kind == kind
                    && if tail.is_empty() {
                        !python
                    } else {
                        tail.iter().any(|t| f.path.ends_with(t))
                    }
            })
            .collect();
        sel.sort_by_key(order);
        if sel.is_empty() {
            return;
        }
        if compact && kind == CaptureKind::Histogram && sel.len() > COMPACT_HISTOGRAMS {
            // First, last and evenly spaced files between.
            let n = sel.len();
            let keep: Vec<usize> = (0..COMPACT_HISTOGRAMS)
                .map(|i| i * (n - 1) / (COMPACT_HISTOGRAMS - 1))
                .collect();
            sel = keep.into_iter().map(|i| sel[i]).collect();
        }
        let paths: Vec<String> = sel.iter().map(|f| pointer(&f.path)).collect();
        cmds.push(format!("dla summarize {op} {}", paths.join(" ")));
    };
    // Dumps of separate processes (command mode) are no series for heap-graph, which
    // reads successive dumps of one process: only driver groups get the command.
    if !command_mode {
        for (_, g) in &groups {
            push("heap-graph", CaptureKind::Heap, &[], g);
        }
    }
    for (_, g) in &groups {
        push("type-histogram-diff", CaptureKind::Histogram, &[], g);
        // Python: object counts by type.
        push(
            "type-histogram-diff",
            CaptureKind::Histogram,
            &[".types.json"],
            g,
        );
        // Node: the class histogram of the snapshots (several processes are fine here).
        push(
            "type-histogram-diff",
            CaptureKind::Heap,
            &[".heapsnapshot"],
            g,
        );
    }
    if let Some((_, g)) = groups.first() {
        push("alloc-stack-diff", CaptureKind::Jfr, &[], g);
        // Node: one command per series of sampling profiles.
        push("alloc-stack-diff", CaptureKind::Alloc, &[], g);
        // Python: tracemalloc stacks and memray's live-at-exit report are two formats.
        push("alloc-stack-diff", CaptureKind::Alloc, &[".alloc.json"], g);
        push("alloc-stack-diff", CaptureKind::Alloc, &[".memray.json"], g);
        // One command per file kind: each format has its own adapter, and `summarize`
        // refuses inputs of mixed formats.
        for tail in [
            "threads.json",
            "classloader-stats.txt",
            "nmt-diff.txt",
            "resources.json",
            ".nonheap.json",
            // Go: the goroutine dumps (growth by creation site) and the leak profiles.
            ".goroutine.txt",
            ".goroutineleak.txt",
        ] {
            push("non-heap", CaptureKind::NonHeap, &[tail], g);
        }
        // Valgrind: the vgdb answers are a heap-graph input, the XML an alloc-stack-diff
        // input. Heaptrack reports and its diff are two formats.
        push(
            "heap-graph",
            CaptureKind::Valgrind,
            &["vgdb-1-leak.txt", "vgdb-2-blocks.txt", "vgdb-3-who.txt"],
            g,
        );
        push(
            "alloc-stack-diff",
            CaptureKind::Valgrind,
            &["valgrind.xml"],
            g,
        );
        push(
            "alloc-stack-diff",
            CaptureKind::Heaptrack,
            &["heaptrack.txt"],
            g,
        );
        push(
            "alloc-stack-diff",
            CaptureKind::Heaptrack,
            &["heaptrack-diff.txt"],
            g,
        );
    }
    cmds
}

/// File name of the allocator shim's exit records, split out of a JVM process's dump.
pub const SHIM_DUMP: &str = "shim-dump.jsonl";

/// Splits the allocator shim's exit records (with allocation stacks) out of each
/// `<out_dir>/<label>/dump.jsonl` that also holds a managed runtime's records, and returns
/// the `dla summarize type-histogram-diff` commands that read them (one per series: fresh
/// keys first, then the repeated control). The `shim-stack-groups` adapter refuses a dump
/// whose head carries the runtime's `types` rows, so it reads the split file. A harness
/// without a shim, or a native one whose dump already is the shim's, gets no command.
pub fn shim_dump_commands(
    out_dir: &Path,
    harness_id: &str,
    pointer: &dyn Fn(&str) -> String,
) -> Vec<String> {
    let Ok(rd) = fs::read_dir(out_dir) else {
        return Vec::new();
    };
    // (series suffix, point number, label)
    let mut found: Vec<(String, u64, String)> = Vec::new();
    for e in rd.flatten() {
        let label = e.file_name().to_string_lossy().into_owned();
        let Ok(text) = fs::read_to_string(e.path().join("dump.jsonl")) else {
            continue;
        };
        let mut shim_lines: Vec<&str> = Vec::new();
        let mut managed = false;
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            if v.get("dla_harness").is_none() {
                continue;
            }
            let native = v
                .get("runtime")
                .and_then(|r| r.as_str())
                .is_some_and(|r| r.eq_ignore_ascii_case("native"));
            if !native {
                managed = true;
            } else if v.get("phase").and_then(|p| p.as_str()) == Some("exit")
                && v.get("raw_sites")
                    .and_then(|r| r.as_object())
                    .is_some_and(|o| !o.is_empty())
            {
                shim_lines.push(line);
            }
        }
        if !managed || shim_lines.is_empty() {
            continue;
        }
        let file = e.path().join(SHIM_DUMP);
        if fs::write(&file, shim_lines.join("\n") + "\n").is_err() {
            continue;
        }
        let rest = label.strip_prefix('p').unwrap_or(&label);
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        let suffix = rest[digits.len()..].trim_start_matches('-').to_owned();
        found.push((suffix, digits.parse().unwrap_or(0), label));
    }
    found.sort_by(|a, b| {
        (a.0.ends_with("repeated"), &a.0, a.1, &a.2).cmp(&(
            b.0.ends_with("repeated"),
            &b.0,
            b.1,
            &b.2,
        ))
    });
    let mut cmds: Vec<String> = Vec::new();
    let mut i = 0;
    while i < found.len() {
        let j = found[i..]
            .iter()
            .position(|f| f.0 != found[i].0)
            .map_or(found.len(), |n| i + n);
        let paths: Vec<String> = found[i..j]
            .iter()
            .map(|f| {
                pointer(&format!(
                    "harness/{harness_id}/{}/{}/{SHIM_DUMP}",
                    super::OUT,
                    f.2
                ))
            })
            .collect();
        cmds.push(format!(
            "dla summarize type-histogram-diff {}",
            paths.join(" ")
        ));
        i = j;
    }
    cmds
}

/// Keeps the leading commands that fit `budget` bytes whole, and says how many were left
/// out (a cut command would be useless).
pub fn fit(commands: &[String], budget: usize) -> (Vec<String>, usize) {
    let mut used = 0;
    let mut out = Vec::new();
    for c in commands {
        if used + c.len() + 4 > budget {
            break;
        }
        used += c.len() + 4;
        out.push(c.clone());
    }
    let left = commands.len() - out.len();
    (out, left)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(kind: CaptureKind, label: &str, seq: u32, x: f64, path: &str) -> CaptureFile {
        CaptureFile {
            kind,
            path: path.to_owned(),
            label: label.to_owned(),
            seq,
            x: Some(x),
            role: None,
            bytes: 10,
            ms: None,
            captured_at: "2026-01-01T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn capture_set_parses_kinds_and_rejects_others() {
        let s: CaptureSet = "heap, histogram".parse().unwrap();
        assert!(s.heap && s.histogram && !s.jfr && !s.nonheap);
        assert_eq!(s.names(), ["heap", "histogram"]);
        let all: CaptureSet = "all".parse().unwrap();
        assert_eq!(all.names().len(), 5);
        let node: CaptureSet = "alloc,heap".parse().unwrap();
        assert!(node.alloc && node.heap && !node.jfr);
        assert_eq!(node.env_value(true), "heap,alloc");
        assert!("none".parse::<CaptureSet>().unwrap().is_empty());
        assert!("heap,bogus".parse::<CaptureSet>().is_err());
    }

    #[test]
    fn runtimes_keep_only_the_kinds_their_agent_writes() {
        let all: CaptureSet = "all".parse().unwrap();
        let jvm = all.intersect(&supported(Runtime::Jvm));
        assert_eq!(jvm.names(), ["heap", "histogram", "jfr", "nonheap"]);
        let node = all.intersect(&supported(Runtime::Node));
        assert_eq!(node.names(), ["heap", "nonheap", "alloc"]);
        let go = all.intersect(&supported(Runtime::Go));
        assert_eq!(go.names(), ["nonheap", "alloc"]);
        let py = all.intersect(&supported(Runtime::Python));
        assert_eq!(py.names(), ["histogram", "nonheap", "alloc"]);
        assert!(all.intersect(&supported(Runtime::Rust)).is_empty());
    }

    #[test]
    fn go_file_names_classify() {
        let c = classify("go-3.heap.pb.gz").unwrap();
        assert_eq!((c.0, c.1), (CaptureKind::Alloc, 3));
        for name in [
            "go-2.goroutine.pb.gz",
            "go-2.goroutine.txt",
            "go-2.goroutineleak.txt",
        ] {
            let c = classify(name).unwrap();
            assert_eq!((c.0, c.1), (CaptureKind::NonHeap, 2), "{name}");
        }
        assert!(classify("go-x.heap.pb.gz").is_none());
        assert!(classify("go-1.heap.txt").is_none());
    }

    #[test]
    fn go_series_print_one_command_per_operation_and_format() {
        let f = |kind, k: u32, tail: &str| {
            file(
                kind,
                "p1-fresh",
                k,
                f64::from(k) * 100.0,
                &format!("p/capture/7/go-{k}.{tail}"),
            )
        };
        let mut files = Vec::new();
        for k in 1..=3 {
            files.push(f(CaptureKind::Alloc, k, "heap.pb.gz"));
            files.push(f(CaptureKind::NonHeap, k, "goroutine.pb.gz"));
            files.push(f(CaptureKind::NonHeap, k, "goroutine.txt"));
            files.push(f(CaptureKind::NonHeap, k, "goroutineleak.txt"));
        }
        let cmds = summarize_commands(&files, &|p| p.to_owned(), false);
        let heap = cmds
            .iter()
            .find(|c| c.contains("alloc-stack-diff"))
            .unwrap();
        assert!(heap.contains("go-1.heap.pb.gz") && heap.contains("go-3.heap.pb.gz"));
        // Goroutine dumps and leak profiles are separate commands (separate adapters), and
        // the protobuf goroutine profiles are not printed: they hold no creation site.
        let dumps = cmds
            .iter()
            .find(|c| c.ends_with("go-3.goroutine.txt"))
            .unwrap();
        assert!(dumps.starts_with("dla summarize non-heap "));
        assert!(!dumps.contains("leak"));
        let leaks = cmds
            .iter()
            .find(|c| c.ends_with("go-3.goroutineleak.txt"))
            .unwrap();
        assert!(!leaks.contains("go-1.goroutine.txt"));
        assert!(
            !cmds.iter().any(|c| c.contains("goroutine.pb.gz")),
            "{cmds:?}"
        );
    }

    #[test]
    fn node_file_names_classify() {
        let c = classify("heap-3.heapsnapshot").unwrap();
        assert_eq!((c.0, c.1), (CaptureKind::Heap, 3));
        let c = classify("alloc-2.heapprofile").unwrap();
        assert_eq!((c.0, c.1), (CaptureKind::Alloc, 2));
        assert_eq!(classify("resources.json").unwrap().0, CaptureKind::NonHeap);
        assert_eq!(classify("scripts.json").unwrap().0, CaptureKind::NonHeap);
        assert!(classify("alloc-x.heapprofile").is_none());
    }

    #[test]
    fn node_series_print_one_command_per_operation_and_format() {
        let snap = |k: u32, x: f64| {
            file(
                CaptureKind::Heap,
                "p1-fresh",
                k,
                x,
                &format!("p/capture/7/heap-{k}.heapsnapshot"),
            )
        };
        let mut files = vec![snap(1, 100.0), snap(2, 200.0), snap(3, 300.0)];
        files.push(file(
            CaptureKind::Alloc,
            "p1-fresh",
            1,
            100.0,
            "p/capture/7/alloc-1.heapprofile",
        ));
        files.push(file(
            CaptureKind::NonHeap,
            "p1-fresh",
            5,
            300.0,
            "p/capture/7/resources.json",
        ));
        files.push(file(
            CaptureKind::NonHeap,
            "p1-fresh",
            6,
            300.0,
            "p/capture/7/scripts.json",
        ));
        let cmds = summarize_commands(&files, &|p| p.to_owned(), false);
        let ops: Vec<&str> = cmds.iter().map(|c| c.split(' ').nth(2).unwrap()).collect();
        assert_eq!(
            ops,
            [
                "heap-graph",
                "type-histogram-diff",
                "alloc-stack-diff",
                "non-heap"
            ],
            "{cmds:?}"
        );
        assert!(cmds[0].ends_with(
            "heap-1.heapsnapshot p/capture/7/heap-2.heapsnapshot p/capture/7/heap-3.heapsnapshot"
        ));
        assert!(cmds[3].ends_with("resources.json"), "{cmds:?}");
        assert!(!cmds.iter().any(|c| c.contains("scripts.json")));
        // Snapshots of separate processes (command mode) give the histogram only.
        let cmd_files: Vec<CaptureFile> = [1, 2, 3]
            .iter()
            .map(|k| {
                file(
                    CaptureKind::Heap,
                    &format!("p{k}-native"),
                    1,
                    f64::from(*k) * 100.0,
                    &format!("p{k}/capture/{k}/heap-1.heapsnapshot"),
                )
            })
            .collect();
        let cmds = summarize_commands(&cmd_files, &|p| p.to_owned(), false);
        assert_eq!(cmds.len(), 1, "{cmds:?}");
        assert!(cmds[0].starts_with("dla summarize type-histogram-diff "));
    }

    #[test]
    fn env_value_keeps_heavy_kinds_for_selected_points_only() {
        let all: CaptureSet = "heap,histogram,jfr,nonheap".parse().unwrap();
        assert_eq!(all.env_value(true), "histogram,heap,jfr,nonheap");
        assert_eq!(all.env_value(false), "histogram");
    }

    #[test]
    fn point_selection_covers_warmup_end_middle_and_last() {
        assert_eq!(
            select_points(9, 1, 4).into_iter().collect::<Vec<_>>(),
            [1, 4, 6, 9]
        );
        assert_eq!(select_points(3, 1, 4).len(), 3);
        assert_eq!(
            select_points(9, 1, 3).into_iter().collect::<Vec<_>>(),
            [1, 5, 9]
        );
        assert!(select_points(0, 1, 3).is_empty());
        assert_eq!(select_points(9, 20, 3).into_iter().collect::<Vec<_>>(), [9]);
    }

    #[test]
    fn file_names_classify() {
        assert_eq!(classify("heap-3.hprof").unwrap().0, CaptureKind::Heap);
        assert_eq!(classify("heap-3.hprof").unwrap().1, 3);
        assert_eq!(
            classify("histogram-12.txt").unwrap().0,
            CaptureKind::Histogram
        );
        assert_eq!(classify("recording.jfr").unwrap().0, CaptureKind::Jfr);
        assert_eq!(classify("nmt-diff.txt").unwrap().0, CaptureKind::NonHeap);
        assert!(classify("heap-x.hprof").is_none());
        assert!(classify("notes.md").is_none());
    }

    #[test]
    fn time_formats_as_utc() {
        use std::time::{Duration, UNIX_EPOCH};
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        assert_eq!(
            rfc3339(UNIX_EPOCH + Duration::from_secs(1_700_000_000)),
            "2023-11-14T22:13:20Z"
        );
        assert_eq!(
            rfc3339(UNIX_EPOCH + Duration::from_secs(1_709_164_800)),
            "2024-02-29T00:00:00Z"
        );
    }

    #[test]
    fn gather_lists_the_measured_process_and_drops_the_others() {
        let dir = tempfile::tempdir().unwrap();
        let label_dir = dir.path().join("p1-fresh");
        for (pid, name, size) in [
            ("100", "heap-1.hprof", 5),
            ("100", "histogram-1.txt", 7),
            ("100", "heap-2.hprof", 9),
            ("100", "recording.jfr", 4),
            ("100", "empty.jfr", 0),
            ("200", "heap-1.hprof", 5),
        ] {
            let d = label_dir.join(DIR).join(pid);
            fs::create_dir_all(&d).unwrap();
            fs::write(d.join(name), vec![b'x'; size]).unwrap();
        }
        let note = |kind: &str, file: &str, error: Option<&str>| DumpCapture {
            kind: Some(kind.into()),
            file: Some(file.into()),
            role: None,
            bytes: Some(5),
            ms: Some(42),
            error: error.map(str::to_owned),
        };
        let notes = AgentNotes {
            records: vec![(
                Some(2000.0),
                vec![
                    note("heap", "heap-1.hprof", None),
                    note("histogram", "histogram-1.txt", None),
                    // The dump failed part way: the file is incomplete.
                    note("heap", "heap-2.hprof", Some("IOException")),
                ],
            )],
        };
        let args = |killed| {
            gather(
                &label_dir,
                "harness/H1/out/p1-fresh",
                "p1-fresh",
                Some(100),
                &notes,
                None,
                killed,
            )
        };
        // A killed process: the flight recording and the dump with an error note stay out,
        // and so does a dump without any note.
        fs::write(label_dir.join(DIR).join("100").join("heap-3.hprof"), b"cut").unwrap();
        let (files, left_out) = args(true);
        assert_eq!(files.len(), 2, "{files:?}");
        assert!(files.iter().all(|f| f.kind != CaptureKind::Jfr));
        assert_eq!(left_out.len(), 3, "{left_out:?}");
        assert!(left_out.iter().any(|l| l.ends_with("heap-3.hprof")));
        assert!(left_out.iter().any(|l| l.ends_with("heap-2.hprof")));
        // A process that ended by itself keeps its recording.
        let (files, left_out) = args(false);
        assert_eq!(files.len(), 3);
        assert_eq!(left_out.len(), 2);
        let heap = files.iter().find(|f| f.kind == CaptureKind::Heap).unwrap();
        assert_eq!(
            heap.path,
            "harness/H1/out/p1-fresh/capture/100/heap-1.hprof"
        );
        assert_eq!((heap.ms, heap.x, heap.bytes), (Some(42), Some(2000.0), 5));
        assert!(!label_dir.join(DIR).join("200").exists());
    }

    #[test]
    fn commands_group_driver_files_by_label_and_fit_a_budget() {
        let mut files = Vec::new();
        for i in 1..=3 {
            files.push(file(
                CaptureKind::Heap,
                "p1-fresh",
                i,
                f64::from(i) * 100.0,
                &format!("h/p1-fresh/capture/9/heap-{i}.hprof"),
            ));
            files.push(file(
                CaptureKind::Histogram,
                "p1-fresh",
                i,
                f64::from(i) * 100.0,
                &format!("h/p1-fresh/capture/9/histogram-{i}.txt"),
            ));
        }
        files.push(file(
            CaptureKind::Jfr,
            "p1-fresh",
            1,
            300.0,
            "h/p1-fresh/capture/9/recording.jfr",
        ));
        files.push(file(
            CaptureKind::NonHeap,
            "p1-fresh",
            1,
            300.0,
            "h/p1-fresh/capture/9/threads.json",
        ));
        files.push(file(
            CaptureKind::NonHeap,
            "p1-fresh",
            2,
            300.0,
            "h/p1-fresh/capture/9/nmt-summary.txt",
        ));
        let cmds = summarize_commands(&files, &|p| format!(".dla/r/{p}"), false);
        assert_eq!(cmds.len(), 4);
        assert_eq!(
            cmds[0],
            "dla summarize heap-graph .dla/r/h/p1-fresh/capture/9/heap-1.hprof .dla/r/h/p1-fresh/capture/9/heap-2.hprof .dla/r/h/p1-fresh/capture/9/heap-3.hprof"
        );
        assert!(cmds[1].starts_with("dla summarize type-histogram-diff"));
        assert!(cmds[3].starts_with("dla summarize non-heap") && cmds[3].contains("threads.json"));
        assert!(!cmds[3].contains("nmt-summary"));
        let (kept, left) = fit(&cmds, cmds[0].len() + cmds[1].len() + 10);
        assert_eq!((kept.len(), left), (2, 2));
        let (none, all) = fit(&cmds, 10);
        assert_eq!((none.len(), all), (0, 4));
    }

    #[test]
    fn shim_records_of_a_jvm_dump_are_split_out_for_the_command() {
        let d = std::env::temp_dir().join(format!("dla-shimdump-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        for k in [1u32, 10, 2] {
            let dir = d.join(format!("p{k}-native"));
            fs::create_dir_all(&dir).unwrap();
            let text = concat!(
                r#"{"dla_harness":1,"phase":"exit","runtime":"jvm","pid":7,"types":[{"name":"A"}]}"#,
                "\n",
                r#"{"dla_harness":1,"phase":"exit","runtime":"native","pid":7,"raw_sites":{"0x1":{"bytes":8}}}"#,
                "\n"
            );
            fs::write(dir.join("dump.jsonl"), text).unwrap();
        }
        // A native-only dump gets no split file.
        let nat = d.join("p3-native");
        fs::create_dir_all(&nat).unwrap();
        fs::write(
            nat.join("dump.jsonl"),
            r#"{"dla_harness":1,"phase":"exit","runtime":"native","raw_sites":{"0x1":{"bytes":8}}}"#,
        )
        .unwrap();
        let cmds = shim_dump_commands(&d, "H1", &|p| format!("r/{p}"));
        assert_eq!(cmds.len(), 1, "{cmds:?}");
        let c = &cmds[0];
        assert!(
            c.starts_with(
                "dla summarize type-histogram-diff r/harness/H1/out/p1-native/shim-dump.jsonl"
            ),
            "{c}"
        );
        assert!(
            c.find("p2-native").unwrap() < c.find("p10-native").unwrap(),
            "{c}"
        );
        assert!(!c.contains("p3-native"), "{c}");
        let split = fs::read_to_string(d.join("p1-native").join(SHIM_DUMP)).unwrap();
        assert_eq!(split.lines().count(), 1);
        assert!(!split.contains("\"types\""));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn compact_commands_list_four_histograms() {
        let files: Vec<_> = (1..=9u32)
            .map(|i| {
                file(
                    CaptureKind::Histogram,
                    "p1-fresh",
                    i,
                    f64::from(i),
                    &format!("h/histogram-{i}.txt"),
                )
            })
            .collect();
        let full = summarize_commands(&files, &|p| p.to_owned(), false);
        let short = summarize_commands(&files, &|p| p.to_owned(), true);
        assert_eq!(full[0].matches(".txt").count(), 9);
        assert_eq!(short[0].matches(".txt").count(), 4);
        assert!(short[0].contains("histogram-1.txt") && short[0].contains("histogram-9.txt"));
    }

    #[test]
    fn command_mode_files_of_many_points_form_one_group() {
        let files: Vec<_> = [(1u32, 1000.0), (5, 5000.0), (9, 9000.0)]
            .iter()
            .map(|(k, x)| {
                file(
                    CaptureKind::Histogram,
                    &format!("p{k}-native"),
                    1,
                    *x,
                    &format!("h/p{k}-native/capture/1/histogram-1.txt"),
                )
            })
            .collect();
        let cmds = summarize_commands(&files, &|p| p.to_owned(), false);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].find("p1-native").unwrap() < cmds[0].find("p9-native").unwrap());
    }

    #[test]
    fn command_mode_heap_dumps_get_no_heap_graph_command() {
        let files: Vec<_> = [(1u32, 1000.0), (5, 5000.0)]
            .iter()
            .map(|(k, x)| {
                file(
                    CaptureKind::Heap,
                    &format!("p{k}-native"),
                    1,
                    *x,
                    &format!("h/p{k}-native/capture/1/heap-1.hprof"),
                )
            })
            .collect();
        assert!(summarize_commands(&files, &|p| p.to_owned(), false).is_empty());
    }

    #[test]
    fn tool_files_get_one_command_per_op_and_format() {
        let mut files = Vec::new();
        for (k, x) in [(1u32, 1000.0), (5, 5000.0), (9, 9000.0)] {
            let l = format!("p{k}-native");
            for (kind, seq, name) in [
                (CaptureKind::Valgrind, 1, "valgrind.xml"),
                (CaptureKind::Valgrind, 2, "vgdb-1-leak.txt"),
                (CaptureKind::Valgrind, 3, "vgdb-2-blocks.txt"),
                (CaptureKind::Valgrind, 4, "vgdb-3-who.txt"),
                (CaptureKind::Heaptrack, 1, "heaptrack.txt"),
            ] {
                files.push(file(kind, &l, seq, x, &format!("h/{l}/{name}")));
            }
        }
        files.push(file(
            CaptureKind::Heaptrack,
            "p9-native",
            2,
            9000.0,
            "h/p9-native/heaptrack-diff.txt",
        ));
        let cmds = summarize_commands(&files, &|p| p.to_owned(), false);
        assert_eq!(cmds.len(), 4, "{cmds:#?}");
        // vgdb answers: heap-graph, points in order, three files each, no XML.
        assert!(cmds[0].starts_with("dla summarize heap-graph h/p1-native/vgdb-1-leak.txt"));
        assert_eq!(cmds[0].matches("vgdb-").count(), 9);
        assert!(!cmds[0].contains(".xml") && !cmds[0].contains("heaptrack"));
        // XML: alloc-stack-diff.
        assert!(cmds[1].starts_with("dla summarize alloc-stack-diff h/p1-native/valgrind.xml"));
        assert_eq!(cmds[1].matches("valgrind.xml").count(), 3);
        // heaptrack reports and the diff are separate commands.
        assert_eq!(cmds[2].matches("heaptrack.txt").count(), 3);
        assert!(!cmds[2].contains("heaptrack-diff"));
        assert_eq!(
            cmds[3],
            "dla summarize alloc-stack-diff h/p9-native/heaptrack-diff.txt"
        );
    }

    #[test]
    fn vgdb_files_classify_as_valgrind_answers() {
        for (name, seq) in [
            ("vgdb-1-leak.txt", 2),
            ("vgdb-2-blocks.txt", 3),
            ("vgdb-3-who.txt", 4),
        ] {
            let (kind, s, _) = classify(name).unwrap();
            assert_eq!((kind, s), (CaptureKind::Valgrind, seq));
        }
    }
}
