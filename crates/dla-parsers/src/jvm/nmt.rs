//! `jcmd <pid> VM.native_memory summary` and `summary.diff` text to a
//! [`NonHeapSnapshot`]: reserved and committed memory per category, and their deltas.
//!
//! Needs `-XX:NativeMemoryTracking=summary` on the JVM. All values are in KB.
//!
//! Metrics, all unit `kb`, name = the NMT category (`Thread`, `Class`, `Total`, ...):
//! category `nmt.committed` and `nmt.reserved`; a diff adds `nmt.committed.delta` and
//! `nmt.reserved.delta` (0 for a category the diff prints without a change).

use super::ParseError;
use crate::model::{NonHeapMetric, NonHeapSnapshot};

/// The `source` of a summary snapshot.
pub const SOURCE_SUMMARY: &str = "jvm-nmt";
/// The `source` of a diff snapshot.
pub const SOURCE_DIFF: &str = "jvm-nmt-diff";

#[derive(Debug, Default, Clone, Copy)]
struct Amount {
    reserved: i64,
    committed: i64,
    reserved_delta: i64,
    committed_delta: i64,
}

/// Parses `VM.native_memory summary`.
pub fn parse_summary(text: &str) -> Result<NonHeapSnapshot, ParseError> {
    let cats = parse_categories(text)?;
    Ok(snapshot(SOURCE_SUMMARY, cats, false))
}

/// Parses `VM.native_memory summary.diff`.
pub fn parse_diff(text: &str) -> Result<NonHeapSnapshot, ParseError> {
    let cats = parse_categories(text)?;
    Ok(snapshot(SOURCE_DIFF, cats, true))
}

fn snapshot(source: &str, cats: Vec<(String, Amount)>, diff: bool) -> NonHeapSnapshot {
    let mut snap = NonHeapSnapshot {
        source: source.to_owned(),
        ..NonHeapSnapshot::default()
    };
    let mut push = |category: &str, name: &str, value: i64| {
        snap.metrics.push(NonHeapMetric {
            category: category.into(),
            name: name.into(),
            value,
            unit: "kb".into(),
        });
    };
    for (name, a) in &cats {
        push("nmt.committed", name, a.committed);
        push("nmt.reserved", name, a.reserved);
        if diff {
            push("nmt.committed.delta", name, a.committed_delta);
            push("nmt.reserved.delta", name, a.reserved_delta);
        }
    }
    snap
}

fn parse_categories(text: &str) -> Result<Vec<(String, Amount)>, ParseError> {
    if text.contains("Native memory tracking is not enabled")
        || text.contains("Native Memory Tracking is not enabled")
    {
        return Err(ParseError::new(
            "native memory tracking is not enabled on the JVM; start it with -XX:NativeMemoryTracking=summary",
        ));
    }
    let mut out: Vec<(String, Amount)> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Total:") {
            out.insert(0, ("Total".to_owned(), amounts(rest, t)?));
        } else if let Some(rest) = t.strip_prefix('-')
            && let Some(p) = rest.find(" (reserved=")
        {
            let name = rest[..p].trim().to_owned();
            let inner = &rest[p + 2..];
            if name.is_empty() {
                continue;
            }
            out.push((name, amounts(inner, t)?));
        }
    }
    if !out.iter().any(|(n, _)| n == "Total") {
        return Err(ParseError::new(
            "no `Total: reserved=..., committed=...` line found; expected the output of `jcmd <pid> VM.native_memory summary` (or summary.diff)",
        ));
    }
    Ok(out)
}

/// Reads `reserved=1KB +2KB, committed=3KB -4KB` from `s`.
fn amounts(s: &str, line: &str) -> Result<Amount, ParseError> {
    let bad = || {
        ParseError::new(format!(
            "cannot read the native memory line `{line}`; expected `reserved=<n>KB[ +<d>KB], committed=<n>KB[ +<d>KB]`"
        ))
    };
    let (r, c) = (
        field(s, "reserved=").ok_or_else(bad)?,
        field(s, "committed=").ok_or_else(bad)?,
    );
    Ok(Amount {
        reserved: r.0,
        committed: c.0,
        reserved_delta: r.1,
        committed_delta: c.1,
    })
}

/// Value and optional delta after `key` in `s`, in KB.
fn field(s: &str, key: &str) -> Option<(i64, i64)> {
    let rest = &s[s.find(key)? + key.len()..];
    let mut it = rest.split([',', ')']).next()?.split_whitespace();
    let v = to_kb(it.next()?)?;
    let d = match it.next() {
        Some(d) => to_kb(d)?,
        None => 0,
    };
    Some((v, d))
}

/// `1234KB`, `+5MB`, `-2KB`, `7GB`, `900B` to KB.
fn to_kb(tok: &str) -> Option<i64> {
    let tok = tok.trim();
    let (sign, body) = match tok.as_bytes().first()? {
        b'+' => (1, &tok[1..]),
        b'-' => (-1, &tok[1..]),
        _ => (1, tok),
    };
    let split = body.find(|c: char| !c.is_ascii_digit())?;
    let (num, unit) = body.split_at(split);
    let n: i64 = num.parse().ok()?;
    let kb = match unit {
        "KB" => n,
        "MB" => n.checked_mul(1024)?,
        "GB" => n.checked_mul(1024 * 1024)?,
        "B" => n / 1024,
        _ => return None,
    };
    Some(sign * kb)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIFF: &str = "24012:

Native Memory Tracking:

Total: reserved=5064030KB +183KB, committed=368654KB +791KB

-                 Java Heap (reserved=3502080KB, committed=262144KB)
                            (mmap: reserved=3502080KB, committed=262144KB)
 
-                     Class (reserved=1048771KB +13KB, committed=707KB +77KB)
                            (classes #1896 +181)
-               Arena Chunk (reserved=10873KB -63KB, committed=10873KB -63KB)
";

    #[test]
    fn diff_has_deltas() {
        let s = parse_diff(DIFF).unwrap();
        let get = |cat: &str, name: &str| {
            s.metrics
                .iter()
                .find(|m| m.category == cat && m.name == name)
                .unwrap()
                .value
        };
        assert_eq!(get("nmt.committed", "Total"), 368_654);
        assert_eq!(get("nmt.committed.delta", "Total"), 791);
        assert_eq!(get("nmt.committed.delta", "Class"), 77);
        assert_eq!(get("nmt.reserved.delta", "Arena Chunk"), -63);
        assert_eq!(get("nmt.committed.delta", "Java Heap"), 0);
    }

    #[test]
    fn summary_has_no_delta_metrics() {
        let s = parse_summary(DIFF).unwrap();
        assert!(s.metrics.iter().all(|m| !m.category.ends_with("delta")));
        assert_eq!(s.source, SOURCE_SUMMARY);
    }

    #[test]
    fn disabled_and_malformed() {
        let e = parse_summary("Native memory tracking is not enabled\n").unwrap_err();
        assert!(e.message().contains("-XX:NativeMemoryTracking=summary"));
        let e = parse_summary("hello\n").unwrap_err();
        assert!(e.message().contains("Total"));
        let e = parse_summary("Total: reserved=oops\n").unwrap_err();
        assert!(e.message().contains("cannot read"));
        assert_eq!(to_kb("2MB"), Some(2048));
    }
}
