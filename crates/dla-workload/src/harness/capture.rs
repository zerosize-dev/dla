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

use dla_core::harness::{CaptureFile, CaptureKind, DumpCapture};

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
    /// Thread dump, class loader statistics, native memory.
    pub nonheap: bool,
}

impl CaptureSet {
    /// Nothing is captured.
    pub fn is_empty(&self) -> bool {
        !(self.heap || self.histogram || self.jfr || self.nonheap)
    }

    /// The kinds, in the order they are listed.
    pub fn names(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        for (on, n) in [
            (self.heap, "heap"),
            (self.histogram, "histogram"),
            (self.jfr, "jfr"),
            (self.nonheap, "nonheap"),
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
                "all" => {
                    set = Self {
                        heap: true,
                        histogram: true,
                        jfr: true,
                        nonheap: true,
                    }
                }
                "none" => set = Self::default(),
                other => {
                    return Err(format!(
                        "{other:?} is not a capture kind (heap, histogram, jfr, nonheap, all, none)"
                    ));
                }
            }
        }
        Ok(set)
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
        "valgrind.xml" => Some((CaptureKind::Valgrind, 1, "valgrind memcheck XML")),
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
pub fn gather(
    label_dir: &Path,
    run_rel: &str,
    label: &str,
    pid: Option<u32>,
    notes: &AgentNotes,
    x_default: Option<f64>,
) -> Vec<CaptureFile> {
    let base = label_dir.join(DIR);
    let mut out = Vec::new();
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
    out
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
                f.kind == kind && (tail.is_empty() || tail.iter().any(|t| f.path.ends_with(t)))
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
    for (_, g) in &groups {
        push("heap-graph", CaptureKind::Heap, &[], g);
    }
    for (_, g) in &groups {
        push("type-histogram-diff", CaptureKind::Histogram, &[], g);
    }
    if let Some((_, g)) = groups.first() {
        push("alloc-stack-diff", CaptureKind::Jfr, &[], g);
        push(
            "non-heap",
            CaptureKind::NonHeap,
            &["threads.json", "classloader-stats.txt", "nmt-diff.txt"],
            g,
        );
        push("heap-graph", CaptureKind::Valgrind, &[], g);
        push("alloc-stack-diff", CaptureKind::Heaptrack, &[], g);
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
        assert_eq!(all.names().len(), 4);
        assert!("none".parse::<CaptureSet>().unwrap().is_empty());
        assert!("heap,bogus".parse::<CaptureSet>().is_err());
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
            ("100", "empty.jfr", 0),
            ("200", "heap-1.hprof", 5),
        ] {
            let d = label_dir.join(DIR).join(pid);
            fs::create_dir_all(&d).unwrap();
            fs::write(d.join(name), vec![b'x'; size]).unwrap();
        }
        let notes = AgentNotes {
            records: vec![(
                Some(2000.0),
                vec![DumpCapture {
                    kind: Some("heap".into()),
                    file: Some("heap-1.hprof".into()),
                    role: None,
                    bytes: Some(5),
                    ms: Some(42),
                    error: None,
                }],
            )],
        };
        let files = gather(
            &label_dir,
            "harness/H1/out/p1-fresh",
            "p1-fresh",
            Some(100),
            &notes,
            None,
        );
        assert_eq!(files.len(), 2);
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
                    CaptureKind::Heap,
                    &format!("p{k}-native"),
                    1,
                    *x,
                    &format!("h/p{k}-native/capture/1/heap-1.hprof"),
                )
            })
            .collect();
        let cmds = summarize_commands(&files, &|p| p.to_owned(), false);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].find("p1-native").unwrap() < cmds[0].find("p9-native").unwrap());
    }
}
