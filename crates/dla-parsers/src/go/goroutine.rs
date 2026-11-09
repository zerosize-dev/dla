//! Goroutines as a [`NonHeapSnapshot`]: from the `debug=2` text (grouped by the `created
//! by` site) and from the goroutine profile protobuf (grouped by the frame each stack is
//! blocked in).
//!
//! Groups (kind `goroutine`):
//! - `created:<function>`: goroutines made by one function. The group carries the `go`
//!   statement as its `site`, which is where the goroutine was created, so it can tie to a
//!   finding by site. Text only.
//! - `wait:<reason>`: goroutines by wait reason (`select`, `chan receive`). Text only.
//! - `blocked:<function>`: goroutines by the first non-runtime frame of their stack. It is
//!   where the goroutine is now and never a creation site, so it has no `site`. Profile
//!   only.

use std::collections::BTreeMap;

use super::{
    ParseError,
    dump::{self, Dump},
    normalize_path,
    pprof::Profile,
};
use crate::model::{Frame, NonHeapMetric, NonHeapSnapshot, ResourceGroup};

/// The `source` of snapshots read from the `debug=2` text.
pub const DUMP_SOURCE: &str = "go-goroutine-dump";
/// The `source` of snapshots read from the goroutine profile protobuf.
pub const PROFILE_SOURCE: &str = "go-goroutine-profile";

/// Groups kept per kind; the rest are counted in the `groups_dropped` metric.
const MAX_GROUPS_PER_KIND: usize = 100;

/// True when `p` is a goroutine profile.
pub fn is_goroutine_profile(p: &Profile) -> bool {
    p.sample_types.len() == 1 && p.sample_types[0].kind == "goroutine"
}

fn metric(name: &str, value: u64) -> NonHeapMetric {
    NonHeapMetric {
        category: "goroutine".into(),
        name: name.into(),
        value: i64::try_from(value).unwrap_or(i64::MAX),
        unit: "count".into(),
    }
}

fn clip_groups(groups: &mut Vec<ResourceGroup>, kind_prefix: &str, dropped: &mut u64) {
    // Biggest first, then the key, so the cut is deterministic.
    let mut mine: Vec<ResourceGroup> = Vec::new();
    groups.retain(|g| {
        if g.key.starts_with(kind_prefix) {
            mine.push(g.clone());
            false
        } else {
            true
        }
    });
    mine.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.key.cmp(&b.key)));
    if mine.len() > MAX_GROUPS_PER_KIND {
        *dropped += (mine.len() - MAX_GROUPS_PER_KIND) as u64;
        mine.truncate(MAX_GROUPS_PER_KIND);
    }
    groups.extend(mine);
}

/// The snapshot of a `debug=2` dump.
pub fn dump_snapshot(d: &Dump) -> NonHeapSnapshot {
    let total = d.goroutines.len() as u64;
    let leaked = d.goroutines.iter().filter(|g| g.leaked).count() as u64;
    let mut groups: Vec<ResourceGroup> = dump::group_by_creator(&d.goroutines, false)
        .into_iter()
        .map(|g| ResourceGroup {
            kind: "goroutine".into(),
            key: format!("created:{}", g.function),
            count: g.count,
            site: Some(Frame {
                function: g.function,
                file: g.file,
                line: g.line,
                module: None,
            }),
        })
        .collect();
    let mut waits: BTreeMap<&str, u64> = BTreeMap::new();
    for g in &d.goroutines {
        *waits.entry(g.wait.as_str()).or_default() += 1;
    }
    groups.extend(waits.into_iter().map(|(w, n)| ResourceGroup {
        kind: "goroutine".into(),
        key: format!("wait:{w}"),
        count: n,
        site: None,
    }));
    let mut dropped = 0;
    clip_groups(&mut groups, "created:", &mut dropped);
    clip_groups(&mut groups, "wait:", &mut dropped);
    let mut metrics = vec![metric("total", total), metric("leaked", leaked)];
    if dropped > 0 {
        metrics.push(metric("groups_dropped", dropped));
    }
    NonHeapSnapshot {
        source: DUMP_SOURCE.into(),
        timestamp_ms: None,
        metrics,
        groups,
    }
}

/// The snapshot of a goroutine profile protobuf.
pub fn profile_snapshot(p: &Profile) -> Result<NonHeapSnapshot, ParseError> {
    if !is_goroutine_profile(p) {
        return Err(ParseError::new(format!(
            "the profile has the sample types [{}], not a goroutine profile; expected the file `pprof.Lookup(\"goroutine\").WriteTo(f, 0)` writes",
            p.sample_types
                .iter()
                .map(|t| t.kind.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let mut total = 0u64;
    let mut by: BTreeMap<String, u64> = BTreeMap::new();
    for s in &p.samples {
        let n = u64::try_from(s.values[0]).unwrap_or(0);
        total = total.saturating_add(n);
        let frames = p.frames(s);
        let at = frames
            .iter()
            .find(|(f, file, _)| {
                !f.is_empty()
                    && !f.starts_with("runtime.")
                    && !normalize_path(file).starts_with("$GOROOT/src/runtime/")
            })
            .or(frames.first());
        let key = match at {
            Some((f, _, _)) if !f.is_empty() => format!("blocked:{f}"),
            _ => "blocked:(unknown)".to_owned(),
        };
        let e = by.entry(key).or_default();
        *e = e.saturating_add(n);
    }
    let mut groups: Vec<ResourceGroup> = by
        .into_iter()
        .map(|(key, count)| ResourceGroup {
            kind: "goroutine".into(),
            key,
            count,
            site: None,
        })
        .collect();
    let mut dropped = 0;
    clip_groups(&mut groups, "blocked:", &mut dropped);
    let mut metrics = vec![metric("total", total)];
    if dropped > 0 {
        metrics.push(metric("groups_dropped", dropped));
    }
    Ok(NonHeapSnapshot {
        source: PROFILE_SOURCE.into(),
        timestamp_ms: u64::try_from(p.time_nanos / 1_000_000)
            .ok()
            .filter(|t| *t > 0),
        metrics,
        groups,
    })
}

#[cfg(test)]
mod tests {
    use super::{super::pprof::build::*, *};

    #[test]
    fn text_groups_carry_the_creation_site_and_no_current_frames() {
        let d = dump::parse(
            "goroutine 5 [chan receive]:\nmain.w()\n\t/p/a.go:5 +0x1\ncreated by main.start in goroutine 1\n\t/p/a.go:9 +0x2\n\ngoroutine 6 [chan receive]:\nmain.w()\n\t/p/a.go:5 +0x1\ncreated by main.start in goroutine 1\n\t/p/a.go:9 +0x2\n",
        )
        .unwrap();
        let s = dump_snapshot(&d);
        let g = s
            .groups
            .iter()
            .find(|g| g.key == "created:main.start")
            .unwrap();
        assert_eq!(g.count, 2);
        assert_eq!(g.site.as_ref().unwrap().line, Some(9));
        let w = s
            .groups
            .iter()
            .find(|g| g.key == "wait:chan receive")
            .unwrap();
        assert!(w.site.is_none());
        assert_eq!(s.metrics[0].value, 2);
    }

    #[test]
    fn profile_groups_by_blocked_frame_never_by_site() {
        let mut p = P::new();
        p.sample_type("goroutine", "count");
        p.function(1, "runtime.gopark", "/g/src/runtime/proc.go");
        p.function(2, "example.com/n.run", "/w/n.go");
        p.location(1, &[(1, 10)]);
        p.location(2, &[(2, 40)]);
        p.sample(&[1, 2], &[500]);
        p.time(1_700_000_000_000_000_000);
        let prof = super::super::pprof::decode(&p.finish()).unwrap();
        let s = profile_snapshot(&prof).unwrap();
        assert_eq!(s.groups.len(), 1);
        assert_eq!(s.groups[0].key, "blocked:example.com/n.run");
        assert_eq!(s.groups[0].count, 500);
        assert!(s.groups[0].site.is_none());
        assert_eq!(s.metrics[0].value, 500);
        assert_eq!(s.timestamp_ms, Some(1_700_000_000_000));
    }
}
