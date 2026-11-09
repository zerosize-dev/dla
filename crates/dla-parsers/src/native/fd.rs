//! Classification of `/proc/<pid>/fd` link targets (what `readlink` returns for each
//! descriptor) into kinds, and counts per kind.
//!
//! | Target | Kind |
//! |---|---|
//! | `socket:[123]` | `socket` |
//! | `pipe:[123]` | `pipe` |
//! | `anon_inode:[eventfd]`, `anon_inode:[eventpoll]`, `anon_inode:[timerfd]` | `eventfd`, `eventpoll`, `timerfd` (the inner name; `anon_inode:inotify` gives `inotify`) |
//! | `/memfd:name (deleted)` | `memfd` |
//! | `/tmp/x (deleted)` | `deleted` |
//! | `/dev/null`, `/dev/pts/0` | `dev` |
//! | any other absolute path | `file` |
//! | anything else | `other` |
//!
//! Kind names use only `a-z`, `0-9` and `_`, so `fd.<kind>` is a valid column name. The same
//! rules are used by `dla sample` (crates/dla-sample/src/os/fd.rs keeps its own copy because
//! that crate does not depend on this one).

use std::collections::BTreeMap;

use crate::model::{NonHeapSnapshot, ResourceGroup};

/// The kind of one link target.
pub fn classify(target: &str) -> String {
    let t = target.trim();
    if let Some(inner) = t.strip_prefix("anon_inode:") {
        let inner = inner.trim_start_matches('[').trim_end_matches(']');
        let name = sanitize(inner);
        return if name.is_empty() {
            "anon_inode".to_owned()
        } else {
            name
        };
    }
    if t.starts_with("socket:[") {
        return "socket".to_owned();
    }
    if t.starts_with("pipe:[") {
        return "pipe".to_owned();
    }
    if t.starts_with("/memfd:") {
        return "memfd".to_owned();
    }
    if t.ends_with(" (deleted)") {
        return "deleted".to_owned();
    }
    if t.starts_with("/dev/") {
        return "dev".to_owned();
    }
    if t.starts_with('/') {
        return "file".to_owned();
    }
    "other".to_owned()
}

fn sanitize(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| match c.to_ascii_lowercase() {
            c @ ('a'..='z' | '0'..='9' | '_') => c,
            _ => '_',
        })
        .collect();
    s.trim_matches('_').to_owned()
}

/// Counts per kind, in kind order.
pub fn counts<'a, I: IntoIterator<Item = &'a str>>(targets: I) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for t in targets {
        *out.entry(classify(t)).or_insert(0) += 1;
    }
    out
}

/// The counts as [`ResourceGroup`]s (`kind` is `fd`, `key` the fd kind), most numerous first.
pub fn groups<'a, I: IntoIterator<Item = &'a str>>(targets: I) -> Vec<ResourceGroup> {
    let mut v: Vec<ResourceGroup> = counts(targets)
        .into_iter()
        .map(|(key, count)| ResourceGroup {
            kind: "fd".to_owned(),
            key,
            count,
            site: None,
        })
        .collect();
    v.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.key.cmp(&b.key)));
    v
}

/// A non-heap snapshot (source `proc-fd`) with the groups and an `fd` / `total` metric.
pub fn snapshot<'a, I: IntoIterator<Item = &'a str>>(targets: I) -> NonHeapSnapshot {
    let targets: Vec<&str> = targets.into_iter().collect();
    NonHeapSnapshot {
        source: "proc-fd".to_owned(),
        timestamp_ms: None,
        metrics: vec![crate::model::NonHeapMetric {
            category: "fd".to_owned(),
            name: "total".to_owned(),
            value: targets.len() as i64,
            unit: "count".to_owned(),
        }],
        groups: groups(targets.iter().copied()),
    }
}

/// Targets from a captured listing: one target per line, or `ls -l` / `readlink` style lines
/// (`3 -> socket:[123]`, `lrwx------ 1 u g 64 Nov 9 07:00 4 -> /tmp/x (deleted)`). Blank
/// lines are skipped.
pub fn targets_from_listing(text: &str) -> Vec<&str> {
    text.lines()
        .map(|l| {
            // Split at the first arrow: the part before it is the fd number (and `ls -l`
            // fields), the target may itself contain ` -> `. A plain target line is kept whole.
            match l.split_once(" -> ") {
                Some((head, t))
                    if head
                        .split_whitespace()
                        .next_back()
                        .is_some_and(|n| n.bytes().all(|b| b.is_ascii_digit())) =>
                {
                    t
                }
                _ => l,
            }
            .trim()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds() {
        for (target, kind) in [
            ("socket:[48112]", "socket"),
            ("pipe:[9023]", "pipe"),
            ("anon_inode:[eventfd]", "eventfd"),
            ("anon_inode:[eventpoll]", "eventpoll"),
            ("anon_inode:[timerfd]", "timerfd"),
            ("anon_inode:[signalfd]", "signalfd"),
            ("anon_inode:inotify", "inotify"),
            ("anon_inode:bpf-map", "bpf_map"),
            ("anon_inode:", "anon_inode"),
            ("/memfd:jit (deleted)", "memfd"),
            ("/tmp/#682325 (deleted)", "deleted"),
            ("/dev/null", "dev"),
            ("/dev/pts/3", "dev"),
            ("/etc/hostname", "file"),
            ("/home/user/with space.txt", "file"),
            ("net:[4026531840]", "other"),
            ("", "other"),
        ] {
            assert_eq!(classify(target), kind, "{target}");
        }
    }

    #[test]
    fn listing_forms() {
        let text = "3 -> socket:[1]\nlrwx------ 1 u g 64 Nov  9 07:00 4 -> /tmp/x (deleted)\n\n/etc/hostname\n";
        assert_eq!(
            targets_from_listing(text),
            ["socket:[1]", "/tmp/x (deleted)", "/etc/hostname"]
        );
    }
}
