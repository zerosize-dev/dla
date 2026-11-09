//! `jcmd <pid> Thread.dump_to_file -format=json <file>` JSON to a [`NonHeapSnapshot`].
//!
//! The JSON lists the threads of every thread container with their stacks. JDK 21 does
//! not print a thread state, so a coarse state is derived from the innermost frame; a
//! `state` key is used when a newer JDK prints one. Groups (kind `thread`):
//! - `state:<state>`: threads by state,
//! - `name:<pattern>`: threads by normalized name (`pool-3-thread-7` is `pool-*-thread-*`),
//! - `frame:<Class.method>`: threads by innermost project frame of the current stack.
//!
//! The `frame:` groups never set `site`: JDK 21 JSON thread dumps carry no creation
//! site, and the frame a thread is running in now is not where it was created.

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
    let mut by_frame: BTreeMap<String, u64> = BTreeMap::new();
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
                *by_frame.entry(f.function).or_default() += 1;
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
        "frame",
        by_frame.into_iter().map(|(k, n)| (k, n, None)).collect(),
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

/// A coarse state: `parked`, `sleeping`, `waiting`, `io`, `running`, or `no-stack`
/// (threads the VM owns print no frames). The leading run of JDK frames (at most
/// `LEAD_FRAMES`) is scanned and the first recognized marker wins, so a virtual thread
/// parked through `VirtualThread.park` or a socket wait below `Net.poll` is not `running`.
fn derive_state(stack: &[Frame]) -> String {
    const LEAD_FRAMES: usize = 6;
    if stack.is_empty() {
        return "no-stack".into();
    }
    let lead: Vec<&str> = stack
        .iter()
        .take(LEAD_FRAMES)
        .take_while(|f| f.module.is_some() || is_library_name(&f.function))
        .map(|f| f.function.as_str())
        .collect();
    for f in &lead {
        let s = if f.ends_with("VirtualThread.parkNanos") {
            if lead.iter().any(|g| g.ends_with("VirtualThread.sleepNanos")) {
                "sleeping"
            } else {
                "parked"
            }
        } else if f.ends_with("Unsafe.park")
            || f.contains("LockSupport.park")
            || f.ends_with("VirtualThread.park")
        {
            "parked"
        } else if f.contains("Thread.sleep") {
            "sleeping"
        } else if f.contains("Object.wait") {
            "waiting"
        } else if f.contains("accept0")
            || f.ends_with("Net.accept")
            || f.ends_with("Net.poll")
            || f.contains("epollWait")
            || f.contains("EPoll.wait")
            || f.contains("SocketDispatcher.read")
            || f.contains("SocketDispatcher.write")
            || f.contains("FileInputStream.read")
            || f.contains("NioSocketImpl.park")
        {
            "io"
        } else {
            continue;
        };
        return s.into();
    }
    "running".into()
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
        let fr = g("frame:com.acme.Worker.run");
        assert_eq!(fr.count, 2);
        assert!(
            s.groups.iter().all(|g| g.site.is_none()),
            "no creation site"
        );
        assert!(!s.groups.iter().any(|g| g.key.starts_with("site:")));
    }

    #[test]
    fn virtual_thread_and_socket_states() {
        let j = r#"{"threadDump":{"threadContainers":[{"threads":[
          {"name":"","stack":["java.base\/java.lang.VirtualThread.park(VirtualThread.java:596)","java.base\/java.lang.System$2.parkVirtualThread(System.java:2848)","java.base\/java.util.concurrent.locks.LockSupport.park(LockSupport.java:221)","com.acme.W.run(W.java:3)"]},
          {"name":"vs","stack":["java.base\/java.lang.VirtualThread.parkNanos(VirtualThread.java:631)","java.base\/java.lang.VirtualThread.sleepNanos(VirtualThread.java:803)","java.base\/java.lang.Thread.sleep(Thread.java:507)","com.acme.W.run(W.java:4)"]},
          {"name":"acc","stack":["java.base\/sun.nio.ch.Net.accept(Native Method)","java.base\/sun.nio.ch.NioSocketImpl.accept(NioSocketImpl.java:748)","com.acme.S.run(S.java:9)"]},
          {"name":"spin","stack":["com.acme.Spin.run(Spin.java:1)"]}
        ]}]}}"#;
        let s = parse(j).unwrap();
        let g = |k: &str| s.groups.iter().find(|g| g.key == k).map(|g| g.count);
        assert_eq!(g("state:parked"), Some(1));
        assert_eq!(g("state:sleeping"), Some(1));
        assert_eq!(g("state:io"), Some(1));
        assert_eq!(g("state:running"), Some(1));
    }

    #[test]
    fn malformed() {
        let e = parse("not json").unwrap_err();
        assert!(e.message().contains("not valid JSON"));
        let e = parse(r#"{"other":1}"#).unwrap_err();
        assert!(e.message().contains("threadContainers"));
    }
}
