//! JVM tool parsers on real output (committed under tests/fixtures/jvm, produced by JDK 21
//! tools against a small Java program with a static session cache) and on malformed input.

use std::{fs, path::PathBuf};

use dla_parsers::jvm::{classloader, deproxy, first_project_frame, jfr, nmt, thread_dump};

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/jvm")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn jfr_old_object_sample_groups_by_stack_and_root() {
    let p = jfr::parse(&fixture("old_object_sample.txt")).unwrap();
    assert_eq!(p.source, "jfr-old-object-sample");
    assert!(p.sampled);
    let holder = "static:com.acme.SessionCache.SESSIONS";
    // The payloads: one group, eight samples of 2 kB each, held by the static map.
    let payload = p
        .groups
        .iter()
        .find(|g| {
            g.kind.as_deref() == Some("byte[]")
                && g.root_path.first().map(String::as_str) == Some(holder)
        })
        .expect("payload group");
    assert_eq!(payload.live_blocks, 8);
    assert_eq!(payload.live_bytes, 8 * 2048);
    assert_eq!(payload.frames[0].function, "com.acme.Session.<init>");
    assert_eq!(payload.frames[0].line, Some(9));
    // The proxy frame is kept raw; the first project frame skips nothing here but is
    // identifiable.
    assert!(
        payload
            .frames
            .iter()
            .any(|f| f.function.contains("$$SpringCGLIB$$"))
    );
    let site = first_project_frame(&payload.frames).unwrap();
    assert_eq!(site.function, "com.acme.Session.<init>");
    assert!(
        payload
            .root_path
            .contains(&"java.util.HashMap$Node".to_owned())
    );
    // The insertion site comes for free from the table resize sample.
    let resize = p
        .groups
        .iter()
        .find(|g| {
            g.frames
                .iter()
                .any(|f| f.function == "com.acme.SessionCache.register")
        })
        .expect("resize group");
    assert_eq!(resize.root_path[0], holder);
    // Groups are largest first.
    assert!(
        p.groups
            .windows(2)
            .all(|w| w[0].live_bytes >= w[1].live_bytes)
    );
}

#[test]
fn jfr_noise_keeps_its_own_holder_and_roots_without_statics_have_none() {
    let samples = jfr::parse_samples(&fixture("old_object_sample.txt")).unwrap();
    assert!(
        samples
            .iter()
            .any(|s| s.holder.as_deref() == Some("static:jdk.jfr.internal.StringPool.cache"))
    );
    let none = samples
        .iter()
        .find(|s| s.holder.is_none())
        .expect("a sample without a static holder");
    assert!(
        none.root_path[0].starts_with("root:"),
        "{:?}",
        none.root_path
    );
}

#[test]
fn jfr_malformed() {
    assert!(jfr::parse("").unwrap().groups.is_empty());
    let e = jfr::parse("{ \"recording\": {} }").unwrap_err();
    assert!(e.to_string().contains("jfr print"), "{e}");
    let full = fixture("old_object_sample.txt");
    let e = jfr::parse(&full[..full.len() / 2]).unwrap_err();
    assert!(
        e.to_string().contains("truncated") || e.to_string().contains("bracketed"),
        "{e}"
    );
    // An event without an object block.
    let e = jfr::parse("jdk.OldObjectSample {\n  objectSize = 1 bytes\n}\n").unwrap_err();
    assert!(e.to_string().contains("object"), "{e}");
}

#[test]
fn thread_dump_real() {
    let s = thread_dump::parse(&fixture("thread_dump.json")).unwrap();
    assert_eq!(s.source, "jvm-thread-dump");
    assert_eq!(s.metrics[0].value, 25);
    let g = |k: &str| {
        s.groups
            .iter()
            .find(|g| g.key == k)
            .unwrap_or_else(|| panic!("{k}"))
    };
    assert_eq!(g("name:worker-*").count, 12);
    assert_eq!(g("name:pool-*-thread-*").count, 4);
    assert_eq!(g("state:sleeping").count, 17);
    // The 12 workers and the pool threads sleep in lambdas of com.acme.Main.
    let site = s
        .groups
        .iter()
        .filter(|g| g.key.starts_with("site:com.acme.Main.lambda"))
        .map(|g| g.count)
        .sum::<u64>();
    assert_eq!(site, 16);
    assert!(g("state:no-stack").count >= 1);
}

#[test]
fn thread_dump_malformed() {
    let full = fixture("thread_dump.json");
    let e = thread_dump::parse(&full[..full.len() / 2]).unwrap_err();
    assert!(e.to_string().contains("not valid JSON"), "{e}");
    assert!(thread_dump::parse("[]").is_err());
    assert!(thread_dump::parse("").is_err());
}

#[test]
fn classloader_stats_real() {
    let s = classloader::parse(&fixture("classloader_stats.txt")).unwrap();
    assert_eq!(s.groups[0].key, "<boot class loader>");
    assert_eq!(s.groups[0].count, 1852);
    let total = |n: &str| s.metrics.iter().find(|m| m.name == n).unwrap().value;
    assert_eq!(total("total_loaders"), 3);
    assert_eq!(total("total_classes"), 1896);
}

#[test]
fn classloader_stats_malformed() {
    assert!(classloader::parse("").is_err());
    let full = fixture("classloader_stats.txt");
    let cut = full.replace("1852", "18x2");
    assert!(classloader::parse(&cut).is_err());
}

#[test]
fn nmt_real() {
    let sum = nmt::parse_summary(&fixture("nmt_summary.txt")).unwrap();
    let find = |s: &dla_parsers::model::NonHeapSnapshot, c: &str, n: &str| {
        s.metrics
            .iter()
            .find(|m| m.category == c && m.name == n)
            .map(|m| m.value)
    };
    assert_eq!(find(&sum, "nmt.committed", "Total"), Some(368_654));
    assert_eq!(find(&sum, "nmt.committed", "Thread"), Some(2934));
    assert_eq!(find(&sum, "nmt.committed", "Java Heap"), Some(262_144));
    let diff = nmt::parse_diff(&fixture("nmt_summary_diff.txt")).unwrap();
    assert_eq!(diff.source, "jvm-nmt-diff");
    assert_eq!(find(&diff, "nmt.committed.delta", "Total"), Some(791));
    assert_eq!(find(&diff, "nmt.committed.delta", "Class"), Some(77));
    assert_eq!(find(&diff, "nmt.reserved.delta", "Arena Chunk"), Some(-63));
}

#[test]
fn nmt_malformed() {
    assert!(nmt::parse_summary("").is_err());
    let e = nmt::parse_diff("Native memory tracking is not enabled").unwrap_err();
    assert!(e.to_string().contains("-XX:NativeMemoryTracking=summary"));
    let bad = fixture("nmt_summary.txt").replace("committed=368654KB", "committed=lots");
    assert!(nmt::parse_summary(&bad).is_err());
}

#[test]
fn deproxy_keeps_raw() {
    let d = deproxy::deproxy("com.acme.SessionService$$SpringCGLIB$$0");
    assert_eq!(d.name, "com.acme.SessionService");
    assert_eq!(d.raw, "com.acme.SessionService$$SpringCGLIB$$0");
}
