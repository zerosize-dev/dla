//! `jcmd <pid> Thread.dump_to_file -format=json <file>` JSON to a [`NonHeapSnapshot`].
//!
//! The JSON lists the threads of every thread container with their stacks. JDK 21 does
//! not print a thread state, so a coarse state is derived from the innermost frame; a
//! `state` key is used when a newer JDK prints one. Groups (kind `thread`):
//! - `state:<state>`: threads by state,
//! - `name:<pattern>`: threads by normalized name (`pool-3-thread-7` is `pool-*-thread-*`),
//! - `site:<Class.method>`: threads by innermost project frame (with `site` set).

use std::collections::BTreeMap;

use serde_json::Value;

use super::{ParseError, class_of_function, deproxy, is_library_name, parse_java_frame};
use crate::model::{Frame, NonHeapMetric, NonHeapSnapshot, ResourceGroup};

/// The `source` of the snapshots this module returns.
pub const SOURCE: &str = "jvm-thread-dump";

/// Groups kept per kind; the rest are counted in the `groups_dropped` metric.
const MAX_GROUPS_PER_KIND: usize = 100;

/// Replaces every run of digits in a thread name with `*`.
pub fn normalize_thread_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut in_digits = false;
    for c in name.chars() {
        if c.is_ascii_digit() {
            if !in_digits {
                out.push('*');
            }
            in_digits = true;
        } else {
            in_digits = false;
            out.push(c);
        }
    }
    out
}

/// Parses the JSON text of a thread dump.
pub fn parse(text: &str) -> Result<NonHeapSnapshot, ParseError> {
    let v: Value = serde_json::from_str(text).map_err(|e| {
        ParseError::new(format!(
            "the thread dump is not valid JSON ({e}); expected the file written by `jcmd <pid> Thread.dump_to_file -format=json <file>`"
        ))
    })?;
    let containers = v
        .pointer("/threadDump/threadContainers")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ParseError::new(
                "no threadDump.threadContainers array; expected the JSON written by `jcmd <pid> Thread.dump_to_file -format=json <file>` (not the text format)",
            )
        })?;
    let mut total = 0u64;
    let mut by_state: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_name: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_site: BTreeMap<String, (u64, Frame)> = BTreeMap::new();
    for c in containers {
        let Some(threads) = c.get("threads").and_then(Value::as_array) else {
            continue;
        };
        for t in threads {
            total += 1;
            let name = t.get("name").and_then(Value::as_str).unwrap_or("");
            let stack: Vec<Frame> = t
                .get("stack")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(parse_java_frame)
                        .collect()
                })
                .unwrap_or_default();
            let state = t
                .get("state")
                .and_then(Value::as_str)
                .map(str::to_ascii_lowercase)
                .unwrap_or_else(|| derive_state(&stack));
            *by_state.entry(state).or_default() += 1;
            *by_name.entry(normalize_thread_name(name)).or_default() += 1;
            if let Some(f) = site_frame(&stack) {
                by_site
                    .entry(f.function.clone())
                    .and_modify(|e| e.0 += 1)
                    .or_insert((1, f));
            }
        }
    }
    let mut snap = NonHeapSnapshot {
        source: SOURCE.to_owned(),
        ..NonHeapSnapshot::default()
    };
    snap.metrics.push(NonHeapMetric {
        category: "threads".into(),
        name: "total".into(),
        value: total as i64,
        unit: "count".into(),
    });
    let mut dropped = 0u64;
    let mut push_kind = |prefix: &str, rows: Vec<(String, u64, Option<Frame>)>| {
        let mut rows = rows;
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        dropped += rows.len().saturating_sub(MAX_GROUPS_PER_KIND) as u64;
        for (key, count, site) in rows.into_iter().take(MAX_GROUPS_PER_KIND) {
            snap.groups.push(ResourceGroup {
                kind: "thread".into(),
                key: format!("{prefix}:{key}"),
                count,
                site,
            });
        }
    };
    push_kind(
        "state",
        by_state.into_iter().map(|(k, n)| (k, n, None)).collect(),
    );
    push_kind(
        "name",
        by_name.into_iter().map(|(k, n)| (k, n, None)).collect(),
    );
    push_kind(
        "site",
        by_site
            .into_iter()
            .map(|(k, (n, f))| (k, n, Some(f)))
            .collect(),
    );
    if dropped > 0 {
        snap.metrics.push(NonHeapMetric {
            category: "threads".into(),
            name: "groups_dropped".into(),
            value: dropped as i64,
            unit: "count".into(),
        });
    }
    Ok(snap)
}

/// A coarse state from the innermost frame: `parked`, `sleeping`, `waiting`, `io`,
/// `running`, or `no-stack` (threads the VM owns print no frames).
fn derive_state(stack: &[Frame]) -> String {
    let Some(top) = stack.first() else {
        return "no-stack".into();
    };
    let f = top.function.as_str();
    let s = if f.ends_with("Unsafe.park") || f.contains("LockSupport.park") {
        "parked"
    } else if f.contains("Thread.sleep") {
        "sleeping"
    } else if f.contains("Object.wait") {
        "waiting"
    } else if f.contains("accept0")
        || f.contains("epollWait")
        || f.contains("EPoll.wait")
        || f.contains("SocketDispatcher.read")
        || f.contains("FileInputStream.read")
        || f.contains("NioSocketImpl.park")
    {
        "io"
    } else {
        "running"
    };
    s.into()
}

/// The innermost project frame that is not a generated proxy class.
fn site_frame(stack: &[Frame]) -> Option<Frame> {
    let is_project = |f: &Frame| {
        f.module.is_none()
            && !is_library_name(&f.function)
            && !deproxy::is_generated(class_of_function(&f.function))
    };
    let found = stack.iter().find(|f| is_project(f)).or_else(|| {
        // Only generated project frames: use the first one, normalized.
        stack
            .iter()
            .find(|f| f.module.is_none() && !is_library_name(&f.function))
    })?;
    let mut f = found.clone();
    let class = deproxy::normalize(class_of_function(&f.function)).into_owned();
    if let Some((_, m)) = f.function.rsplit_once('.') {
        f.function = format!("{class}.{m}");
    }
    Some(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(normalize_thread_name("pool-3-thread-7"), "pool-*-thread-*");
        assert_eq!(normalize_thread_name("worker-12"), "worker-*");
        assert_eq!(normalize_thread_name("main"), "main");
        assert_eq!(
            normalize_thread_name("ForkJoinPool-1-worker-3"),
            "ForkJoinPool-*-worker-*"
        );
    }

    #[test]
    fn small_dump() {
        let j = r#"{"threadDump":{"processId":"1","time":"t","runtimeVersion":"21","threadContainers":[
          {"container":"<root>","parent":null,"owner":null,"threads":[
            {"tid":"1","name":"main","stack":["java.base\/java.lang.Thread.sleep0(Native Method)","com.acme.Main.main(Main.java:24)"]},
            {"tid":"9","name":"Signal Dispatcher","stack":[]},
            {"tid":"20","name":"pool-1-thread-1","stack":["java.base\/jdk.internal.misc.Unsafe.park(Native Method)","com.acme.Worker$$SpringCGLIB$$0.run(Worker.java:5)","com.acme.Worker.run(Worker.java:7)"]},
            {"tid":"21","name":"pool-1-thread-2","stack":["java.base\/jdk.internal.misc.Unsafe.park(Native Method)","com.acme.Worker.run(Worker.java:7)"]}
          ]}]}}"#;
        let s = parse(j).unwrap();
        assert_eq!(s.metrics[0].value, 4);
        let g = |k: &str| s.groups.iter().find(|g| g.key == k).unwrap();
        assert_eq!(g("state:parked").count, 2);
        assert_eq!(g("state:sleeping").count, 1);
        assert_eq!(g("state:no-stack").count, 1);
        assert_eq!(g("name:pool-*-thread-*").count, 2);
        let site = g("site:com.acme.Worker.run");
        assert_eq!(site.count, 2);
        assert_eq!(site.site.as_ref().unwrap().line, Some(7));
    }

    #[test]
    fn malformed() {
        let e = parse("not json").unwrap_err();
        assert!(e.message().contains("not valid JSON"));
        let e = parse(r#"{"other":1}"#).unwrap_err();
        assert!(e.message().contains("threadContainers"));
    }
}
