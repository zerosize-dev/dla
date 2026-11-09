//! `heaptrack_print` text and its folded-stack output.
//!
//! Record with `heaptrack -o <name> <program>` and print with
//! `heaptrack_print <name>.gz -l -n <N>` (add `-d <older>.gz` to diff two runs, `-m0` to turn
//! backtrace merging off). The raw `.gz` file is not parsed.
//!
//! Sections read: `MOST CALLS TO ALLOCATION FUNCTIONS`, `PEAK MEMORY CONSUMERS`,
//! `MOST TEMPORARY ALLOCATIONS` and `MEMORY LEAKS` (with `-l`), plus the closing totals. With
//! merged backtraces (the default) a record is a shared allocator-side prefix followed by one
//! or more caller blocks; each caller block becomes its own stack (prefix then callers).
//!
//! Figures are rounded by `heaptrack_print` (`3.02M`, `548B`) and read back as 1024-based
//! sizes, so they are accurate to about 1% except for `B` values. For exact byte counts use
//! [`parse_folded`] on the file written by `-F <file> --flamegraph-cost-type leaked`.
//!
//! In a diff, figures can be negative; [`StackStat`] keeps the sign, and the profile
//! conversions keep only stacks that grew. Rust allocator-internal frames at the top of a
//! stack (`alloc::alloc`, `__rust_alloc`, `RawVec`, `<alloc::...>`) are trimmed, and Rust
//! symbols are demangled.

use serde::{Deserialize, Serialize};

use super::ParseError;
use super::frames::{make_frame, split_location, trim_allocator_frames};
use crate::model::{AllocGroup, AllocProfile, Frame};

/// One stack with its figures from one section.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct StackStat {
    /// Innermost (allocation site) first.
    pub frames: Vec<Frame>,
    /// Leaked bytes, or bytes at peak, depending on the section; 0 in the temporary section.
    pub bytes: i64,
    /// Calls (allocations) behind the figure; in the temporary section the temporary count.
    pub count: i64,
    /// Temporary section only: allocations in total from this stack.
    pub total: Option<i64>,
}

/// The totals `heaptrack_print` prints at the end.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Totals {
    pub runtime_s: Option<f64>,
    pub allocation_calls: Option<i64>,
    pub temporary_allocations: Option<i64>,
    pub peak_heap_bytes: Option<i64>,
    pub peak_rss_bytes: Option<i64>,
    pub leaked_bytes: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct HeaptrackReport {
    /// `Debuggee command was:` lines, in the order printed. In a diff there are two, but
    /// heaptrack_print reads both files at once and prints each when its read finishes, so
    /// the position does not say which run is the new one and which is the base.
    pub commands: Vec<String>,
    pub is_diff: bool,
    /// `MEMORY LEAKS` (needs `-l`).
    pub leaks: Vec<StackStat>,
    /// `PEAK MEMORY CONSUMERS`.
    pub peak: Vec<StackStat>,
    /// `MOST CALLS TO ALLOCATION FUNCTIONS`: bytes are the peak consumption of the stack.
    pub calls: Vec<StackStat>,
    /// `MOST TEMPORARY ALLOCATIONS`.
    pub temporary: Vec<StackStat>,
    pub totals: Totals,
}

impl HeaptrackReport {
    /// The leak section as a profile (stacks that grew or leaked). Empty without `-l`.
    pub fn leak_profile(&self) -> AllocProfile {
        profile("heaptrack", &self.leaks, "leaked")
    }

    /// The peak section as a profile.
    pub fn peak_profile(&self) -> AllocProfile {
        profile("heaptrack", &self.peak, "peak")
    }
}

fn profile(source: &str, stats: &[StackStat], kind: &str) -> AllocProfile {
    AllocProfile {
        source: source.to_owned(),
        timestamp_ms: None,
        sampled: false,
        groups: stats
            .iter()
            .filter(|s| s.bytes > 0)
            .map(|s| AllocGroup {
                frames: s.frames.clone(),
                live_bytes: s.bytes as u64,
                live_blocks: s.count.max(0) as u64,
                total_bytes: None,
                root_path: Vec::new(),
                kind: Some(kind.to_owned()),
            })
            .collect(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Section {
    None,
    Calls,
    Peak,
    Temporary,
    Leaks,
}

/// Parses `heaptrack_print` output.
pub fn parse(input: &str) -> Result<HeaptrackReport, ParseError> {
    let mut report = HeaptrackReport::default();
    let mut saw_marker = false;
    let mut section = Section::None;
    let mut cur = Group::default();

    for raw in input.lines() {
        let line = raw.trim_end();
        let t = line.trim();
        if let Some(cmd) = t.strip_prefix("Debuggee command was:") {
            report.commands.push(cmd.trim().to_owned());
            saw_marker = true;
            continue;
        }
        if t.starts_with("reading diff file") {
            report.is_diff = true;
            saw_marker = true;
            continue;
        }
        let new_section = match t {
            "MOST CALLS TO ALLOCATION FUNCTIONS" => Some(Section::Calls),
            "PEAK MEMORY CONSUMERS" => Some(Section::Peak),
            "MOST TEMPORARY ALLOCATIONS" => Some(Section::Temporary),
            "MEMORY LEAKS" => Some(Section::Leaks),
            _ => None,
        };
        if let Some(s) = new_section {
            cur.flush(section, &mut report);
            section = s;
            saw_marker = true;
            continue;
        }
        if let Some((key, value)) = t.split_once(':')
            && totals_line(key, value.trim(), &mut report.totals)
        {
            cur.flush(section, &mut report);
            section = Section::None;
            saw_marker = true;
            continue;
        }
        if section == Section::None {
            continue;
        }
        if t.is_empty() {
            cur.flush(section, &mut report);
            continue;
        }
        if let Some(h) = header(t, section) {
            if h.sub {
                cur.start_sub(h);
            } else {
                cur.flush(section, &mut report);
                cur.start(h);
            }
            continue;
        }
        if !cur.open {
            return Err(ParseError::new(format!(
                "unexpected line in a heaptrack_print section: {t:?} (expected a header such as \"3.02M leaked over 3000 calls from\")"
            )));
        }
        // `and 33.81K from 1 other places`: heaptrack's own elision of a merged record.
        if t.starts_with("and ") && t.ends_with("other places") {
            continue;
        }
        if let Some(loc) = t.strip_prefix("at ") {
            if let Some(f) = cur.last_frame() {
                let (file, line_no) = split_location(loc.trim());
                f.file = Some(file);
                f.line = line_no;
            }
        } else if let Some(module) = t.strip_prefix("in ") {
            if let Some(f) = cur.last_frame() {
                f.module = Some(module.trim().to_owned());
            }
        } else {
            cur.push_frame(make_frame(t, None, None, None));
        }
    }
    cur.flush(section, &mut report);
    if report.commands.len() >= 2 {
        report.is_diff = true;
    }
    if !saw_marker {
        return Err(ParseError::new(
            "not heaptrack_print output: no \"Debuggee command was:\" line, section title or totals (run `heaptrack_print <file>.gz -l`)",
        ));
    }
    Ok(report)
}

struct Header {
    sub: bool,
    bytes: i64,
    count: i64,
    total: Option<i64>,
}

#[derive(Default)]
struct Group {
    open: bool,
    head: Option<Header>,
    prefix: Vec<Frame>,
    subs: Vec<(Header, Vec<Frame>)>,
}

impl Group {
    fn start(&mut self, h: Header) {
        self.open = true;
        self.head = Some(h);
        self.prefix.clear();
        self.subs.clear();
    }

    fn start_sub(&mut self, h: Header) {
        if self.open {
            self.subs.push((h, Vec::new()));
        }
    }

    fn push_frame(&mut self, f: Frame) {
        match self.subs.last_mut() {
            Some((_, frames)) => frames.push(f),
            None => self.prefix.push(f),
        }
    }

    fn last_frame(&mut self) -> Option<&mut Frame> {
        match self.subs.last_mut() {
            Some((_, frames)) => frames.last_mut(),
            None => self.prefix.last_mut(),
        }
    }

    fn flush(&mut self, section: Section, report: &mut HeaptrackReport) {
        if !self.open {
            return;
        }
        self.open = false;
        let Some(head) = self.head.take() else {
            return;
        };
        let prefix = std::mem::take(&mut self.prefix);
        let subs = std::mem::take(&mut self.subs);
        let make = |h: &Header, frames: Vec<Frame>| StackStat {
            frames: trim_allocator_frames(frames),
            bytes: h.bytes,
            count: h.count,
            total: h.total,
        };
        let mut stats = Vec::new();
        if subs.is_empty() {
            stats.push(make(&head, prefix));
        } else {
            for (h, callers) in subs {
                let mut frames = prefix.clone();
                frames.extend(callers);
                stats.push(make(&h, frames));
            }
        }
        let target = match section {
            Section::Calls => &mut report.calls,
            Section::Peak => &mut report.peak,
            Section::Temporary => &mut report.temporary,
            Section::Leaks => &mut report.leaks,
            Section::None => return,
        };
        target.extend(stats);
    }
}

fn header(t: &str, section: Section) -> Option<Header> {
    let (body, sub) = if let Some(b) = t.strip_suffix(" from:") {
        (b, true)
    } else {
        (t.strip_suffix(" from")?, false)
    };
    let w: Vec<&str> = body.split_whitespace().collect();
    let first = w.first()?;
    if !first.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        return None;
    }
    let after = |key: &str| {
        w.iter()
            .position(|x| *x == key)
            .and_then(|i| w.get(i + 1))
            .copied()
    };
    match section {
        Section::Leaks | Section::Peak => {
            let bytes = parse_size(first)?;
            if !(w.contains(&"leaked") || w.contains(&"consumed")) {
                return None;
            }
            let count = after("over")?.parse().ok()?;
            Some(Header {
                sub,
                bytes,
                count,
                total: None,
            })
        }
        Section::Calls => {
            if !w.contains(&"calls") {
                return None;
            }
            let count = first.parse().ok()?;
            let bytes = parse_size(after("with")?)?;
            Some(Header {
                sub,
                bytes,
                count,
                total: None,
            })
        }
        Section::Temporary => {
            if !w.contains(&"temporary") {
                return None;
            }
            let count = first.parse().ok()?;
            let total = after("of")?.parse().ok()?;
            Some(Header {
                sub,
                bytes: 0,
                count,
                total: Some(total),
            })
        }
        Section::None => None,
    }
}

/// `3.02M`, `73.73K`, `548B`, `-64B`, `1.5G` as bytes (1024-based).
pub fn parse_size(text: &str) -> Option<i64> {
    let (neg, rest) = match text.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, text),
    };
    let unit = rest.chars().last()?;
    let shift: u32 = match unit {
        'B' => 0,
        'K' => 10,
        'M' => 20,
        'G' => 30,
        'T' => 40,
        _ => return None,
    };
    let number = &rest[..rest.len() - 1];
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    let value: f64 = number.parse().ok()?;
    let bytes = (value * (1u64 << shift) as f64).round();
    if !bytes.is_finite() || bytes > i64::MAX as f64 {
        return None;
    }
    let bytes = bytes as i64;
    Some(if neg { -bytes } else { bytes })
}

fn totals_line(key: &str, value: &str, t: &mut Totals) -> bool {
    let first = || value.split_whitespace().next();
    match key {
        "total runtime" => {
            t.runtime_s = first()
                .map(|v| v.trim_end_matches('.'))
                .and_then(|v| v.strip_suffix('s'))
                .and_then(|v| v.parse().ok());
        }
        "calls to allocation functions" => {
            t.allocation_calls = first().and_then(|v| v.parse().ok())
        }
        "temporary memory allocations" => {
            t.temporary_allocations = first().and_then(|v| v.parse().ok())
        }
        "peak heap memory consumption" => t.peak_heap_bytes = first().and_then(parse_size),
        "peak RSS (including heaptrack overhead)" => {
            t.peak_rss_bytes = first().and_then(parse_size)
        }
        "total memory leaked" => t.leaked_bytes = first().and_then(parse_size),
        _ => return false,
    }
    true
}

/// What the numbers of a folded-stack file are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldedCost {
    /// Bytes (`--flamegraph-cost-type leaked` or `peak`): becomes `live_bytes`.
    Bytes,
    /// Allocation counts (`allocations` or `temporary`): becomes `live_blocks`.
    Calls,
}

/// Parses the folded stacks written by `heaptrack_print -F <file>`: one line per stack,
/// `outermost (file);...;innermost (file); <number>`. Lines with a zero cost are skipped.
pub fn parse_folded(input: &str, cost: FoldedCost) -> Result<AllocProfile, ParseError> {
    let mut groups = Vec::new();
    for (n, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((stack, number)) = line.rsplit_once(' ') else {
            return Err(ParseError::new(format!(
                "folded stack line {} has no cost: {line:?} (expected \"a (f.c);b (f.c); 1234\")",
                n + 1
            )));
        };
        let value: u64 = number.parse().map_err(|_| {
            ParseError::new(format!(
                "folded stack line {} ends in {number:?}, not a number (expected \"a (f.c);b (f.c); 1234\")",
                n + 1
            ))
        })?;
        if value == 0 {
            continue;
        }
        let mut frames: Vec<Frame> = stack
            .split(';')
            .map(str::trim)
            .filter(|f| !f.is_empty())
            .map(
                |f| match f.strip_suffix(')').and_then(|g| g.rsplit_once(" (")) {
                    Some((name, file)) => make_frame(name, Some(file.to_owned()), None, None),
                    None => make_frame(f, None, None, None),
                },
            )
            .collect();
        frames.reverse();
        let (live_bytes, live_blocks) = match cost {
            FoldedCost::Bytes => (value, 0),
            FoldedCost::Calls => (0, value),
        };
        groups.push(AllocGroup {
            frames: trim_allocator_frames(frames),
            live_bytes,
            live_blocks,
            total_bytes: None,
            root_path: Vec::new(),
            kind: None,
        });
    }
    Ok(AllocProfile {
        source: "heaptrack".to_owned(),
        timestamp_ms: None,
        sampled: false,
        groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(parse_size("548B"), Some(548));
        assert_eq!(parse_size("-64B"), Some(-64));
        assert_eq!(parse_size("1.50K"), Some(1536));
        assert_eq!(parse_size("3.02M"), Some(3_166_700));
        assert_eq!(parse_size("2G"), Some(2 << 30));
        for bad in ["", "B", "12", "1.2.3K", "xM", "-", "9e9G"] {
            assert_eq!(parse_size(bad), None, "{bad}");
        }
    }

    #[test]
    fn headers() {
        let h = header("3.02M leaked over 3000 calls from", Section::Leaks).unwrap();
        assert_eq!((h.sub, h.bytes, h.count), (false, 3_166_700, 3000));
        let h = header("1.02M consumed over 2000 calls from:", Section::Peak).unwrap();
        assert!(h.sub);
        let h = header(
            "507 calls to allocation functions with 1.05M peak consumption from",
            Section::Calls,
        )
        .unwrap();
        assert_eq!((h.count, h.bytes), (507, 1_101_005));
        let h = header(
            "-1 temporary allocations of 0 allocations in total (-inf%) from",
            Section::Temporary,
        )
        .unwrap();
        assert_eq!((h.count, h.total), (-1, Some(0)));
        assert!(header("add_session", Section::Leaks).is_none());
        assert!(header("3 leaked over x calls from", Section::Leaks).is_none());
    }
}
