//! Parsers for what the attached tools print: `GC.class_histogram`, `VM.native_memory
//! summary`, `heaptrack_print`, `memray stats --json` and `jfr summary`.

use serde::Serialize;
use serde_json::Value;

/// One row of `GC.class_histogram`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistogramRow {
    /// Instances.
    pub instances: u64,
    /// Bytes.
    pub bytes: u64,
    /// Class name (with module when printed).
    pub class: String,
}

/// A parsed `GC.class_histogram`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Histogram {
    /// Rows in the order printed (largest first).
    pub rows: Vec<HistogramRow>,
    /// Total instances (the `Total` line).
    pub total_instances: u64,
    /// Total bytes.
    pub total_bytes: u64,
}

/// Parses `GC.class_histogram`. `None` when no row was found.
pub fn parse_histogram(text: &str) -> Option<Histogram> {
    let mut h = Histogram::default();
    let mut total = None;
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Total") {
            let mut it = rest.split_whitespace();
            if let (Some(i), Some(b)) = (it.next(), it.next())
                && let (Ok(i), Ok(b)) = (i.parse(), b.parse())
            {
                total = Some((i, b));
            }
            continue;
        }
        // "   1:   118   220000000  [B (java.base@21)"
        let Some((rank, rest)) = t.split_once(':') else {
            continue;
        };
        if rank.is_empty() || !rank.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let mut it = rest.split_whitespace();
        let (Some(i), Some(b)) = (it.next(), it.next()) else {
            continue;
        };
        let (Ok(instances), Ok(bytes)) = (i.parse(), b.parse()) else {
            continue;
        };
        let class = it.collect::<Vec<_>>().join(" ");
        h.rows.push(HistogramRow {
            instances,
            bytes,
            class,
        });
    }
    if h.rows.is_empty() {
        return None;
    }
    (h.total_instances, h.total_bytes) = total.unwrap_or_else(|| {
        (
            h.rows.iter().map(|r| r.instances).sum(),
            h.rows.iter().map(|r| r.bytes).sum(),
        )
    });
    Some(h)
}

/// One category of `VM.native_memory summary`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NmtCategory {
    /// Category name (`Java Heap`, `Class`, `Thread`...).
    pub name: String,
    /// Reserved bytes.
    pub reserved: u64,
    /// Committed bytes.
    pub committed: u64,
}

/// A parsed `VM.native_memory summary`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct NmtSummary {
    /// Total reserved bytes.
    pub reserved: u64,
    /// Total committed bytes.
    pub committed: u64,
    /// Categories, as printed.
    pub categories: Vec<NmtCategory>,
}

/// Parses `VM.native_memory summary`; `None` when there is no `Total:` line (tracking off).
pub fn parse_nmt_summary(text: &str) -> Option<NmtSummary> {
    let mut s = NmtSummary::default();
    let mut found = false;
    for line in text.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("Total:")
            && !found
            && let Some((r, c)) = reserved_committed(rest)
        {
            (s.reserved, s.committed) = (r, c);
            found = true;
        } else if let Some(rest) = t.strip_prefix("- ")
            && let Some((name, tail)) = rest.split_once(" (reserved=")
            && let Some((r, c)) = reserved_committed(&format!("reserved={tail}"))
        {
            s.categories.push(NmtCategory {
                name: name.trim().to_owned(),
                reserved: r,
                committed: c,
            });
        }
    }
    found.then_some(s)
}

fn reserved_committed(text: &str) -> Option<(u64, u64)> {
    let field = |key: &str| {
        let (_, rest) = text.split_once(key)?;
        let token = rest.split([',', ' ', ')']).next()?;
        crate::jcmd::parse_size(token)
    };
    Some((field("reserved=")?, field("committed=")?))
}

/// One entry of the heaptrack peak consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PeakConsumer {
    /// Bytes at the peak.
    pub bytes: u64,
    /// Allocation calls.
    pub calls: u64,
    /// Function name.
    pub function: String,
    /// `file:line` when symbols resolved.
    pub location: Option<String>,
}

/// The summary of `heaptrack_print`.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct HeaptrackSummary {
    /// Seconds the target ran under heaptrack.
    pub runtime_s: Option<f64>,
    /// Calls to allocation functions.
    pub allocation_calls: Option<u64>,
    /// Temporary allocations.
    pub temporary_allocations: Option<u64>,
    /// Peak heap bytes.
    pub peak_heap_bytes: Option<u64>,
    /// Peak RSS bytes (includes heaptrack's own overhead).
    pub peak_rss_bytes: Option<u64>,
    /// Bytes still allocated at the end of the recording.
    pub leaked_bytes: Option<u64>,
    /// Top peak consumers.
    pub peak_consumers: Vec<PeakConsumer>,
}

/// Bytes from a heaptrack size such as `44.30M`, `1.5K`, `12B` (decimal multiples).
pub fn parse_heaptrack_size(token: &str) -> Option<u64> {
    let t = token.trim();
    let split = t.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(t.len());
    let (num, unit) = t.split_at(split);
    let v: f64 = num.parse().ok()?;
    let mult = match unit {
        "" | "B" => 1.0,
        "K" => 1e3,
        "M" => 1e6,
        "G" => 1e9,
        "T" => 1e12,
        _ => return None,
    };
    Some((v * mult).round() as u64)
}

/// Parses the text `heaptrack_print` writes; keeps at most `top` peak consumers.
pub fn parse_heaptrack_print(text: &str, top: usize) -> HeaptrackSummary {
    let mut s = HeaptrackSummary::default();
    let lines: Vec<&str> = text.lines().collect();
    let first_num = |rest: &str| -> Option<u64> { rest.split_whitespace().next()?.parse().ok() };
    let mut in_peaks = false;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim();
        if t == "PEAK MEMORY CONSUMERS" {
            in_peaks = true;
        } else if !t.is_empty() && t.chars().all(|c| c.is_ascii_uppercase() || c == ' ') {
            in_peaks = false;
        }
        if let Some(r) = t.strip_prefix("total runtime: ") {
            s.runtime_s = r.trim_end_matches('.').trim_end_matches('s').parse().ok();
        } else if let Some(r) = t.strip_prefix("calls to allocation functions: ") {
            s.allocation_calls = first_num(r);
        } else if let Some(r) = t.strip_prefix("temporary memory allocations: ") {
            s.temporary_allocations = first_num(r);
        } else if let Some(r) = t.strip_prefix("peak heap memory consumption: ") {
            s.peak_heap_bytes = parse_heaptrack_size(r);
        } else if let Some(r) = t.strip_prefix("peak RSS (including heaptrack overhead): ") {
            s.peak_rss_bytes = parse_heaptrack_size(r);
        } else if let Some(r) = t.strip_prefix("total memory leaked: ") {
            s.leaked_bytes = parse_heaptrack_size(r);
        } else if in_peaks
            && s.peak_consumers.len() < top
            && let Some((size, rest)) = t.split_once(" peak memory consumed over ")
            && let Some(calls) = rest.strip_suffix(" calls from")
            && let (Some(bytes), Ok(calls)) = (parse_heaptrack_size(size), calls.parse())
        {
            let function = lines.get(i + 1).map_or("", |l| l.trim()).to_owned();
            let location = lines
                .get(i + 2)
                .map(|l| l.trim())
                .and_then(|l| l.strip_prefix("at "))
                .map(str::to_owned);
            s.peak_consumers.push(PeakConsumer {
                bytes,
                calls,
                function,
                location,
            });
        }
        i += 1;
    }
    s
}

/// What `memray stats --json` reports.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct MemraySummary {
    /// Allocations recorded.
    pub total_allocations: u64,
    /// Bytes allocated in total.
    pub total_bytes_allocated: u64,
    /// Peak memory in bytes.
    pub peak_memory: Option<u64>,
    /// Largest allocating locations (location, bytes).
    pub top_by_size: Vec<(String, u64)>,
}

/// Parses the JSON file `memray stats --json` writes.
pub fn parse_memray_stats(json_text: &str, top: usize) -> Option<MemraySummary> {
    let v: Value = serde_json::from_str(json_text).ok()?;
    Some(MemraySummary {
        total_allocations: v["total_num_allocations"].as_u64()?,
        total_bytes_allocated: v["total_bytes_allocated"].as_u64().unwrap_or(0),
        peak_memory: v["metadata"]["peak_memory"].as_u64(),
        top_by_size: v["top_allocations_by_size"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|e| Some((e["location"].as_str()?.to_owned(), e["size"].as_u64()?)))
                    .take(top)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// One event type of `jfr summary`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JfrEvent {
    /// Event type (`jdk.ObjectAllocationSample`).
    pub name: String,
    /// Events recorded.
    pub count: u64,
}

/// The `count` largest event types of `jfr summary`, most frequent first.
pub fn parse_jfr_summary(text: &str, top: usize) -> Vec<JfrEvent> {
    let mut v: Vec<JfrEvent> = text
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let name = it.next()?;
            let count: u64 = it.next()?.parse().ok()?;
            let _size: u64 = it.next()?.parse().ok()?;
            (name.contains('.') && it.next().is_none()).then(|| JfrEvent {
                name: name.to_owned(),
                count,
            })
        })
        .collect();
    v.sort_by_key(|e| std::cmp::Reverse(e.count));
    v.truncate(top);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    const HISTO: &str = "1234:\n num     #instances         #bytes  class name (module)\n-------------------------------------------------------\n   1:           118      220000000  [B (java.base@21.0.12)\n   2:         12345        1976000  java.lang.String (java.base@21.0.12)\n   3:            10            800  Foo$Bar\nTotal         12473      221976800\n";

    #[test]
    fn histogram_rows_and_total() {
        let h = parse_histogram(HISTO).unwrap();
        assert_eq!(h.rows.len(), 3);
        assert_eq!(h.rows[0].instances, 118);
        assert_eq!(h.rows[0].bytes, 220_000_000);
        assert_eq!(h.rows[0].class, "[B (java.base@21.0.12)");
        assert_eq!(h.rows[2].class, "Foo$Bar");
        assert_eq!((h.total_instances, h.total_bytes), (12473, 221_976_800));
        assert!(parse_histogram("nothing here").is_none());
    }

    #[test]
    fn histogram_without_total_line_sums_the_rows() {
        let h = parse_histogram("   1:  2  30  A\n   2:  3  40  B\n").unwrap();
        assert_eq!((h.total_instances, h.total_bytes), (5, 70));
    }

    const NMT: &str = "1234:\n\nNative Memory Tracking:\n\n(Omitting categories weighting less than 1KB)\n\nTotal: reserved=1500000KB, committed=345678KB\n       malloc: 1234KB #123\n       mmap:   reserved=1498766KB, committed=344444KB\n\n-                 Java Heap (reserved=262144KB, committed=32768KB)\n                            (mmap: reserved=262144KB, committed=32768KB)\n\n-                     Class (reserved=1048KB, committed=2KB)\n";

    #[test]
    fn nmt_totals_and_categories() {
        let n = parse_nmt_summary(NMT).unwrap();
        assert_eq!((n.reserved, n.committed), (1_500_000 << 10, 345_678 << 10));
        assert_eq!(n.categories.len(), 2);
        assert_eq!(n.categories[0].name, "Java Heap");
        assert_eq!(n.categories[0].committed, 32768 << 10);
        assert_eq!(n.categories[1].name, "Class");
        assert!(parse_nmt_summary("Native memory tracking is not enabled\n").is_none());
    }

    const HT: &str = "Debuggee command was: ../leak\nfinished reading file, now analyzing data:\n\nMOST CALLS TO ALLOCATION FUNCTIONS\n443 calls to allocation functions with 44.30M peak consumption from\nleak_site\n  at /x/leak.c:5\n  in /x/leak\n\nPEAK MEMORY CONSUMERS\n44.30M peak memory consumed over 443 calls from\nleak_site\n  at /x/leak.c:5\n  in /x/leak\n44.30M consumed over 443 calls from:\n    main\n      at /x/leak.c:6\n2.00K peak memory consumed over 3 calls from\nother\n\nMOST TEMPORARY ALLOCATIONS\n\ntotal runtime: 22.36s.\ncalls to allocation functions: 443 (19/s)\ntemporary memory allocations: 0 (0/s)\npeak heap memory consumption: 44.30M\npeak RSS (including heaptrack overhead): 51.04M\ntotal memory leaked: 44.30M\n";

    #[test]
    fn heaptrack_summary_and_peak_consumers() {
        let s = parse_heaptrack_print(HT, 5);
        assert_eq!(s.runtime_s, Some(22.36));
        assert_eq!(s.allocation_calls, Some(443));
        assert_eq!(s.temporary_allocations, Some(0));
        assert_eq!(s.peak_heap_bytes, Some(44_300_000));
        assert_eq!(s.peak_rss_bytes, Some(51_040_000));
        assert_eq!(s.leaked_bytes, Some(44_300_000));
        assert_eq!(s.peak_consumers.len(), 2);
        assert_eq!(s.peak_consumers[0].function, "leak_site");
        assert_eq!(s.peak_consumers[0].location.as_deref(), Some("/x/leak.c:5"));
        assert_eq!(s.peak_consumers[0].calls, 443);
        assert_eq!(s.peak_consumers[1].bytes, 2000);
        assert_eq!(s.peak_consumers[1].location, None);
        assert_eq!(parse_heaptrack_print(HT, 1).peak_consumers.len(), 1);
    }

    #[test]
    fn heaptrack_sizes_are_decimal() {
        assert_eq!(parse_heaptrack_size("44.30M"), Some(44_300_000));
        assert_eq!(parse_heaptrack_size("512B"), Some(512));
        assert_eq!(parse_heaptrack_size("1.5K"), Some(1500));
        assert_eq!(parse_heaptrack_size("x"), None);
    }

    #[test]
    fn memray_stats_json() {
        let j = r#"{"total_num_allocations":113,"total_bytes_allocated":9921923,
          "top_allocations_by_size":[{"location":"<module>:leak.py:6","size":9903331},{"location":"b","size":5}],
          "metadata":{"peak_memory":9919027}}"#;
        let m = parse_memray_stats(j, 1).unwrap();
        assert_eq!(m.total_allocations, 113);
        assert_eq!(m.peak_memory, Some(9_919_027));
        assert_eq!(
            m.top_by_size,
            [("<module>:leak.py:6".to_owned(), 9_903_331)]
        );
        assert!(parse_memray_stats("{}", 3).is_none());
        assert!(parse_memray_stats("not json", 3).is_none());
    }

    #[test]
    fn jfr_summary_events_by_count() {
        let t = " Version: 2.1\n Chunks: 1\n\n Event Type                              Count  Size (bytes)\n=============================================================\n jdk.GCPhasePause                           12          1000\n jdk.ObjectAllocationSample                200         90000\n jdk.ExecutionSample                         5           300\n";
        let e = parse_jfr_summary(t, 2);
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].name, "jdk.ObjectAllocationSample");
        assert_eq!(e[1].count, 12);
    }
}
