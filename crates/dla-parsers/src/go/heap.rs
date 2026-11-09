//! A Go heap profile (`pprof.Lookup("heap").WriteTo(w, 0)`) as allocation stacks and as a
//! histogram by function.
//!
//! The profile has four values per stack: `alloc_objects`, `alloc_space`, `inuse_objects`
//! and `inuse_space`. The in-use values are what the last completed collection found
//! still live, so a profile written right after `runtime.GC()` is a live-bytes view and
//! two such profiles can be diffed. Sizes are the runtime's own estimates from sampling
//! (`runtime.MemProfileRate`), so the profile is `sampled`.

use std::collections::BTreeMap;

use super::{
    ParseError, is_std_function, normalize_path,
    pprof::{self, Profile},
};
use crate::model::{AllocGroup, AllocProfile, Frame, Histogram, HistogramRow};

/// The `source` of the allocation profiles this module returns.
pub const SOURCE: &str = "go-pprof-heap";
/// The `source` of the histograms this module returns.
pub const HISTOGRAM_SOURCE: &str = "go-pprof-heap-functions";

/// Frames kept per stack.
const MAX_FRAMES: usize = 48;

/// True when `p` is a heap profile: it has the in-use sample types.
pub fn is_heap_profile(p: &Profile) -> bool {
    p.value_index("inuse_space").is_some() && p.value_index("inuse_objects").is_some()
}

fn require_heap(p: &Profile) -> Result<(usize, usize, Option<usize>), ParseError> {
    match (p.value_index("inuse_space"), p.value_index("inuse_objects")) {
        (Some(bytes), Some(objects)) => Ok((bytes, objects, p.value_index("alloc_space"))),
        _ => Err(ParseError::new(format!(
            "the profile has the sample types [{}], not a heap profile (expected inuse_space and inuse_objects); expected the file `pprof.Lookup(\"heap\").WriteTo(f, 0)` writes",
            p.sample_types
                .iter()
                .map(|t| t.kind.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn clamp(v: i64) -> u64 {
    u64::try_from(v).unwrap_or(0)
}

fn frames_of(p: &Profile, s: &pprof::Sample) -> Vec<Frame> {
    p.frames(s)
        .into_iter()
        .take(MAX_FRAMES)
        .map(|(function, file, line)| Frame {
            function,
            file: Some(normalize_path(&file)).filter(|f| !f.is_empty()),
            line: u32::try_from(line).ok().filter(|l| *l > 0),
            module: None,
        })
        .collect()
}

/// The profile as allocation stacks: live bytes and blocks from the in-use values, bytes
/// allocated in total from `alloc_space`. Samples of one stack (the runtime keeps one per
/// size class) are added together; stacks with nothing live and nothing allocated are
/// dropped.
pub fn to_alloc_profile(p: &Profile) -> Result<AllocProfile, ParseError> {
    let (bytes, objects, alloc) = require_heap(p)?;
    // The key of a stack: its frames as `(function, file, line)`.
    type StackKey = Vec<(String, Option<String>, Option<u32>)>;
    let mut by_stack: BTreeMap<StackKey, AllocGroup> = BTreeMap::new();
    for s in &p.samples {
        let frames = frames_of(p, s);
        let key = frames
            .iter()
            .map(|f| (f.function.clone(), f.file.clone(), f.line))
            .collect();
        let g = by_stack.entry(key).or_insert_with(|| AllocGroup {
            frames,
            ..AllocGroup::default()
        });
        g.live_bytes = g.live_bytes.saturating_add(clamp(s.values[bytes]));
        g.live_blocks = g.live_blocks.saturating_add(clamp(s.values[objects]));
        if let Some(a) = alloc {
            g.total_bytes = Some(
                g.total_bytes
                    .unwrap_or(0)
                    .saturating_add(clamp(s.values[a])),
            );
        }
    }
    let groups = by_stack
        .into_values()
        .filter(|g| g.live_bytes > 0 || g.live_blocks > 0 || g.total_bytes.unwrap_or(0) > 0)
        .collect();
    Ok(AllocProfile {
        source: SOURCE.to_owned(),
        timestamp_ms: u64::try_from(p.time_nanos / 1_000_000)
            .ok()
            .filter(|t| *t > 0),
        sampled: true,
        groups,
    })
}

/// The label of a stack in the histogram: the innermost frame that is not the standard
/// library (the code that asked for the memory), else the innermost frame.
pub fn label_of(frames: &[Frame]) -> Option<&Frame> {
    frames
        .iter()
        .find(|f| !f.function.is_empty() && !is_std_function(&f.function))
        .or_else(|| frames.iter().find(|f| !f.function.is_empty()))
}

/// The profile as a histogram by function: one row per allocating function with the
/// in-use objects and bytes of its stacks.
pub fn to_histogram(p: &Profile) -> Result<Histogram, ParseError> {
    let (bytes, objects, _) = require_heap(p)?;
    let mut rows: BTreeMap<String, HistogramRow> = BTreeMap::new();
    for s in &p.samples {
        let (b, o) = (clamp(s.values[bytes]), clamp(s.values[objects]));
        if b == 0 && o == 0 {
            continue;
        }
        let frames = frames_of(p, s);
        let Some(f) = label_of(&frames) else { continue };
        let r = rows
            .entry(f.function.clone())
            .or_insert_with(|| HistogramRow {
                name: f.function.clone(),
                ..HistogramRow::default()
            });
        r.count = r.count.saturating_add(o);
        r.bytes = r.bytes.saturating_add(b);
    }
    Ok(Histogram {
        source: HISTOGRAM_SOURCE.to_owned(),
        timestamp_ms: u64::try_from(p.time_nanos / 1_000_000)
            .ok()
            .filter(|t| *t > 0),
        rows: rows.into_values().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::{super::pprof::build::*, *};

    fn heap(inuse: &[(i64, i64)]) -> Profile {
        let mut p = P::new();
        for (k, u) in [
            ("alloc_objects", "count"),
            ("alloc_space", "bytes"),
            ("inuse_objects", "count"),
            ("inuse_space", "bytes"),
        ] {
            p.sample_type(k, u);
        }
        p.function(1, "example.com/p.cache", "/w/harness/H1/work/p/cache.go");
        p.function(
            2,
            "strings.(*Builder).grow",
            "/usr/local/go/src/strings/builder.go",
        );
        p.function(3, "main.main", "/w/harness/H1/work/main.go");
        p.location(10, &[(2, 67)]);
        p.location(11, &[(1, 21)]);
        p.location(12, &[(3, 9)]);
        for (objs, bytes) in inuse {
            p.sample(&[10, 11, 12], &[objs * 2, bytes * 2, *objs, *bytes]);
        }
        p.time(1_700_000_000_000_000_000);
        super::super::pprof::decode(&p.finish()).unwrap()
    }

    #[test]
    fn stacks_carry_live_and_total_bytes_with_project_paths() {
        let prof = heap(&[(3, 300), (2, 200)]);
        assert!(is_heap_profile(&prof));
        let a = to_alloc_profile(&prof).unwrap();
        assert!(a.sampled);
        assert_eq!(a.timestamp_ms, Some(1_700_000_000_000));
        // The two samples share a stack and are added.
        assert_eq!(a.groups.len(), 1);
        let g = &a.groups[0];
        assert_eq!(
            (g.live_bytes, g.live_blocks, g.total_bytes),
            (500, 5, Some(1000))
        );
        assert_eq!(g.frames[1].file.as_deref(), Some("p/cache.go"));
        assert_eq!(g.frames[1].line, Some(21));
        assert_eq!(
            g.frames[0].file.as_deref(),
            Some("$GOROOT/src/strings/builder.go")
        );
    }

    #[test]
    fn the_histogram_is_labelled_by_the_first_project_function() {
        let h = to_histogram(&heap(&[(3, 300)])).unwrap();
        assert_eq!(h.rows.len(), 1);
        assert_eq!(h.rows[0].name, "example.com/p.cache");
        assert_eq!((h.rows[0].count, h.rows[0].bytes), (3, 300));
    }

    #[test]
    fn a_goroutine_profile_is_not_a_heap_profile() {
        let mut p = P::new();
        p.sample_type("goroutine", "count");
        p.sample(&[], &[3]);
        let prof = super::super::pprof::decode(&p.finish()).unwrap();
        assert!(!is_heap_profile(&prof));
        let e = to_alloc_profile(&prof).unwrap_err();
        assert!(e.message().contains("not a heap profile"), "{e}");
    }
}
