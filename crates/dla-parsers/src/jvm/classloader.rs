//! `jcmd <pid> VM.classloader_stats` text to a [`NonHeapSnapshot`]: loaders and classes
//! by loader type.

use std::collections::BTreeMap;

use super::ParseError;
use crate::model::{NonHeapMetric, NonHeapSnapshot, ResourceGroup};

/// The `source` of the snapshots this module returns.
pub const SOURCE: &str = "jvm-classloader-stats";

/// Parses the output. Groups (`kind` `classloader`) hold the class count per loader type;
/// hidden classes are a group per type named `<type> + hidden classes`. Metrics:
/// category `classloader.loaders` (loaders per type), `classloader.classes` (classes per
/// type), and `classloader` `total_loaders` and `total_classes`.
pub fn parse(text: &str) -> Result<NonHeapSnapshot, ParseError> {
    let mut in_table = false;
    let mut by_type: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    let mut hidden: BTreeMap<String, u64> = BTreeMap::new();
    let mut last_type: Option<String> = None;
    let mut totals: Option<(u64, u64)> = None;
    for line in text.lines() {
        let t = line.trim();
        if !in_table {
            if t.starts_with("ClassLoader") && t.contains("Classes") && t.ends_with("Type") {
                in_table = true;
            }
            continue;
        }
        if t.is_empty() {
            continue;
        }
        if let Some(rest) = t.strip_prefix("Total =") {
            let nums: Vec<u64> = rest
                .split_whitespace()
                .filter_map(|w| w.parse().ok())
                .collect();
            if nums.len() >= 2 {
                totals = Some((nums[0], nums[1]));
            }
            continue;
        }
        if t.starts_with("ChunkSz:") || t.starts_with("BlockSz:") {
            continue;
        }
        let toks: Vec<&str> = t.split_whitespace().collect();
        if toks.len() >= 3 && toks.ends_with(&["hidden", "classes"]) {
            // `36 112640 70752 + hidden classes`
            let n: u64 = toks[0].parse().map_err(|_| bad(line))?;
            let ty = last_type.clone().unwrap_or_else(|| "<unknown>".into());
            {
                let e = hidden.entry(ty).or_default();
                *e = e.saturating_add(n);
            }
            continue;
        }
        if toks.len() >= 7 && toks[0].starts_with("0x") {
            let classes: u64 = toks[3].parse().map_err(|_| bad(line))?;
            // The type is what follows the six numeric columns, spaces kept.
            let ty = type_of(t).ok_or_else(|| bad(line))?;
            let e = by_type.entry(ty.clone()).or_default();
            e.0 = e.0.saturating_add(1);
            e.1 = e.1.saturating_add(classes);
            last_type = Some(ty);
        }
    }
    if !in_table {
        return Err(ParseError::new(
            "no class loader table found; expected the output of `jcmd <pid> VM.classloader_stats` (a `ClassLoader Parent CLD* Classes ChunkSz BlockSz Type` header)",
        ));
    }
    let mut snap = NonHeapSnapshot {
        source: SOURCE.to_owned(),
        ..NonHeapSnapshot::default()
    };
    for (ty, (loaders, classes)) in &by_type {
        snap.metrics
            .push(metric("classloader.loaders", ty, *loaders));
        snap.metrics
            .push(metric("classloader.classes", ty, *classes));
        snap.groups.push(ResourceGroup {
            kind: "classloader".into(),
            key: ty.clone(),
            count: *classes,
            site: None,
        });
    }
    for (ty, n) in &hidden {
        snap.groups.push(ResourceGroup {
            kind: "classloader".into(),
            key: format!("{ty} + hidden classes"),
            count: *n,
            site: None,
        });
    }
    let (tl, tc) = totals.unwrap_or_else(|| {
        let l = by_type.values().fold(0u64, |a, v| a.saturating_add(v.0));
        let c = by_type
            .values()
            .map(|v| v.1)
            .chain(hidden.values().copied())
            .fold(0u64, u64::saturating_add);
        (l, c)
    });
    snap.metrics
        .push(metric("classloader", "total_loaders", tl));
    snap.metrics
        .push(metric("classloader", "total_classes", tc));
    snap.groups
        .sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.key.cmp(&b.key)));
    Ok(snap)
}

fn metric(category: &str, name: &str, value: u64) -> NonHeapMetric {
    NonHeapMetric {
        category: category.into(),
        name: name.into(),
        value: i64::try_from(value).unwrap_or(i64::MAX),
        unit: "count".into(),
    }
}

fn bad(line: &str) -> ParseError {
    ParseError::new(format!(
        "cannot read the class loader row `{}`; expected `<loader> <parent> <cld> <classes> <chunk> <block> <type>`",
        line.trim()
    ))
}

/// The text after the first six whitespace-separated columns.
fn type_of(t: &str) -> Option<String> {
    let mut rest = t;
    for _ in 0..6 {
        rest = rest.trim_start();
        let end = rest.find(char::is_whitespace)?;
        rest = &rest[end..];
    }
    let ty = rest.trim();
    (!ty.is_empty()).then(|| ty.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL: &str = "24012:
ClassLoader         Parent              CLD*               Classes   ChunkSz   BlockSz  Type
0x00007f8c17069008  0x0000000000000000  0x00007f8c9019cb30       1      6144      2640  jdk.internal.loader.ClassLoaders$PlatformClassLoader
0x00007f8c17068770  0x00007f8c17069008  0x00007f8c90198220       7     23552     17440  jdk.internal.loader.ClassLoaders$AppClassLoader
0x0000000000000000  0x0000000000000000  0x00007f8c90113290    1852   4653056   4402200  <boot class loader>
                                                                36    112640     70752   + hidden classes
Total = 3                                                     1896   4795392   4493032  
ChunkSz: Total size of all allocated metaspace chunks
";

    #[test]
    fn real_output() {
        let s = parse(REAL).unwrap();
        assert_eq!(s.source, SOURCE);
        let boot = s
            .groups
            .iter()
            .find(|g| g.key == "<boot class loader>")
            .unwrap();
        assert_eq!(boot.count, 1852);
        let hidden = s
            .groups
            .iter()
            .find(|g| g.key == "<boot class loader> + hidden classes")
            .unwrap();
        assert_eq!(hidden.count, 36);
        let total = s
            .metrics
            .iter()
            .find(|m| m.name == "total_loaders")
            .unwrap();
        assert_eq!(total.value, 3);
    }

    #[test]
    fn huge_counts_do_not_overflow() {
        let t = "ClassLoader Parent CLD* Classes ChunkSz BlockSz Type
0xa 0x0 0x0 18446744073709551615 1 2 T
0xb 0x0 0x0 18446744073709551615 1 2 T
                  18446744073709551615 1 2 + hidden classes
                  5 1 2 + hidden classes
";
        let s = parse(t).unwrap();
        let tc = s
            .metrics
            .iter()
            .find(|m| m.name == "total_classes")
            .unwrap();
        assert_eq!(tc.value, i64::MAX);
    }

    #[test]
    fn malformed_input() {
        let e = parse("hello\nworld\n").unwrap_err();
        assert!(e.message().contains("VM.classloader_stats"));
        let e =
            parse("ClassLoader Parent CLD* Classes ChunkSz BlockSz Type\n0xabc 0x0 0x0 xx 1 2 T\n")
                .unwrap_err();
        assert!(e.message().contains("class loader row"));
    }
}
