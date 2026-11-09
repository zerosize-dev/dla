//! `non-heap` over file descriptor listings: counts per kind (socket, pipe, eventfd,
//! eventpoll, timerfd, inotify, memfd, deleted, dev, file, other).
//!
//! Two inputs are read: `ls -l /proc/<pid>/fd` (or `readlink` style) listings, and harness
//! dump files whose records carry the `fd.<kind>` values the allocator shim and the sampler
//! write. Only per-kind counts leave the reader, never a path or a socket name. A kind's
//! count is a whole-process figure with no creation site, so no column is ever tied to a
//! finding: it says a kind of descriptor grows, and a creation site or a probe must say
//! which holder owns them.

use dla_core::{
    HarnessDump,
    evidence::{Authority, tool_class},
};
use dla_parsers::{
    model::{NonHeapMetric, NonHeapSnapshot, ResourceGroup},
    native::fd,
};

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    input::Input,
    non_heap::{self, SnapCapture, SnapOrigin},
};
use crate::fail::{Failure, Res};

/// Largest file read.
const MAX_BYTES: u64 = 16 << 20;

/// The descriptor listing and dump reader.
pub struct ProcFd;

fn is_dump(input: &Input) -> bool {
    let h = input.head_text();
    let first = h.lines().find(|l| !l.trim().is_empty()).unwrap_or_default();
    first.starts_with('{') && first.contains("\"dla_harness\"") && h.contains("\"fd.")
}

/// A listing line: `lrwx------ 1 u g 64 Nov 9 07:53 3 -> socket:[1]` or `3 -> pipe:[2]`.
fn is_listing_line(l: &str) -> bool {
    match l.split_once(" -> ") {
        Some((head, target)) => {
            !target.trim().is_empty()
                && head
                    .split_whitespace()
                    .next_back()
                    .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit()))
        }
        None => false,
    }
}

fn is_listing(input: &Input) -> bool {
    let h = input.head_text();
    if h.trim_start().starts_with('{') || h.contains('\0') {
        return false;
    }
    let lines: Vec<&str> = h.lines().filter(|l| !l.trim().is_empty()).collect();
    // The last line of the head may be cut.
    let lines = if input.size > input.head.len() as u64 {
        &lines[..lines.len().saturating_sub(1)]
    } else {
        &lines[..]
    };
    let n = lines.iter().filter(|l| is_listing_line(l)).count();
    n > 0 && n * 2 >= lines.len()
}

fn snapshot_of_counts<'a>(kinds: impl IntoIterator<Item = (&'a str, u64)>) -> NonHeapSnapshot {
    let mut groups: Vec<ResourceGroup> = kinds
        .into_iter()
        .filter(|(_, n)| *n > 0)
        .map(|(k, n)| ResourceGroup {
            kind: "fd".to_owned(),
            key: k.to_owned(),
            count: n,
            site: None,
        })
        .collect();
    groups.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.key.cmp(&b.key)));
    let total: u64 = groups.iter().map(|g| g.count).sum();
    NonHeapSnapshot {
        source: "proc-fd".to_owned(),
        timestamp_ms: None,
        metrics: vec![NonHeapMetric {
            category: "fd".to_owned(),
            name: "total".to_owned(),
            value: i64::try_from(total).unwrap_or(i64::MAX),
            unit: "count".to_owned(),
        }],
        groups,
    }
}

impl Adapter for ProcFd {
    fn name(&self) -> &'static str {
        "proc-fd"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "any"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::RUNTIME_COUNTER
    }
    fn formats(&self) -> &'static str {
        "ls -l /proc/<pid>/fd listings, or harness dump jsonl with fd.<kind> values"
    }
    fn sniff(&self, input: &Input) -> bool {
        is_dump(input) || is_listing(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut caps: Vec<(Option<f64>, SnapCapture)> = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            if is_dump(i) {
                let (dumps, _bad) = HarnessDump::parse_lines(&text);
                let mut k = 0;
                for d in dumps {
                    let Some(values) = d.values else { continue };
                    let kinds: Vec<(&str, u64)> = values
                        .iter()
                        .filter_map(|(name, v)| {
                            let kind = name.strip_prefix("fd.")?;
                            Some((kind, v.map_or(0, |v| v.max(0.0) as u64)))
                        })
                        .collect();
                    if kinds.is_empty() {
                        continue;
                    }
                    k += 1;
                    caps.push((
                        d.x,
                        SnapCapture {
                            label: format!("{}#{k}", i.shown()),
                            snap: snapshot_of_counts(kinds),
                            sha256: i.sha256.clone(),
                        },
                    ));
                }
            } else {
                let targets = fd::targets_from_listing(&text);
                if targets.is_empty() {
                    return Err(Failure::refuse(
                        "malformed-artifact",
                        format!("{} lists no descriptors", i.shown()),
                        "an ls -l /proc/<pid>/fd listing with `N -> target` lines",
                        "ls -l /proc/<pid>/fd > fds.txt   (as the process owner, while it runs)",
                    ));
                }
                caps.push((
                    None,
                    SnapCapture {
                        label: i.shown().to_owned(),
                        snap: fd::snapshot(targets.iter().copied()),
                        sha256: i.sha256.clone(),
                    },
                ));
            }
        }
        if caps.is_empty() {
            return Err(Failure::refuse(
                "malformed-artifact",
                "the dump files hold no fd.<kind> values".to_owned(),
                "dump files of a Linux harness run (the shim and the sampler write fd.<kind>)",
                "dla harness run <H>   on Linux, then dla summarize non-heap <the dump files>",
            ));
        }
        // Records carry their position on the axis; listings keep the order given.
        if caps.iter().all(|(x, _)| x.is_some()) {
            caps.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        }
        non_heap::run_diff(
            ctx,
            opts,
            caps.into_iter().map(|(_, c)| c).collect(),
            &SnapOrigin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Live,
                runtime: None,
                order_by_time: false,
                limitations: vec![
                    "descriptors are counted by kind: a count says a kind of descriptor grows, not which holder owns them".into(),
                    "whole-process counts have no creation site, so no column is tied to a finding".into(),
                ],
            },
        )
    }
}
