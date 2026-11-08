//! HPROF reader tests: synthetic heaps with exact expectations, a real JVM dump when a
//! JDK is installed, and the committed dump.

mod common;

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use common::hprof_builder::{Model, model};
use dla_parsers::{
    hprof::{HeapSummary, HolderKind, Options, summarize},
    inflate::crc32,
};

fn opts(dir: &Path) -> Options {
    Options {
        tmp_dir: Some(dir.to_path_buf()),
        ..Options::default()
    }
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, bytes).unwrap();
    p
}

/// A gzip member made of stored blocks (no external tool needed).
fn stored_gzip(data: &[u8]) -> Vec<u8> {
    let mut v = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3];
    let mut chunks: Vec<&[u8]> = data.chunks(65535).collect();
    if chunks.is_empty() {
        chunks.push(&[]);
    }
    let last = chunks.len() - 1;
    for (i, c) in chunks.iter().enumerate() {
        v.push(u8::from(i == last));
        v.extend_from_slice(&(c.len() as u16).to_le_bytes());
        v.extend_from_slice(&(!(c.len() as u16)).to_le_bytes());
        v.extend_from_slice(c);
    }
    v.extend_from_slice(&crc32(data).to_le_bytes());
    v.extend_from_slice(&(data.len() as u32).to_le_bytes());
    v
}

fn holder<'a>(s: &'a HeapSummary, key: &str) -> &'a dla_parsers::hprof::Holder {
    s.tracked.iter().find(|h| h.key == key).unwrap_or_else(|| {
        panic!(
            "no holder {key}; have {:?}",
            s.holders.iter().map(|h| &h.key).collect::<Vec<_>>()
        )
    })
}

fn check_model(s: &HeapSummary, m: &Model, id_size: u32, n_items: u64) {
    assert_eq!(s.header.id_size, id_size);
    assert_eq!(s.header.version, "JAVA PROFILE 1.0.2");
    assert!(!s.truncated, "{:?}", s.warnings);
    assert!(s.warnings.is_empty(), "{:?}", s.warnings);
    // The known static holder comes first, with exact retained bytes and count, even
    // though the class loader reaches the same class (the class-loader pitfall).
    let top = &s.holders[0];
    assert_eq!(top.key, "static:app.Holder.ITEMS");
    assert_eq!(top.kind, HolderKind::Static);
    assert_eq!(top.owner_class, "app.Holder");
    assert_eq!(top.field.as_deref(), Some("ITEMS"));
    assert_eq!(top.retained, m.items_retained);
    assert_eq!(top.dominated_count, m.items_count);
    assert_eq!(top.path[0], "root class app.Holder");
    assert_eq!(
        top.path[1],
        "static app.Holder.ITEMS -> java.util.ArrayList"
    );
    assert_eq!(top.path[2], "ArrayList.elementData -> java.lang.Object[]");
    let acc = top.accumulation.as_ref().unwrap();
    assert_eq!(acc.class, "java.lang.Object[]");
    assert_eq!(acc.children, n_items);
    // The loader holds the class only through the artificial class root.
    let l = holder(s, "static:java.lang.ClassLoader.scl");
    assert_eq!(l.retained, m.loader_retained);
    // Reference.referent is not followed: the weakly held array is not retained.
    let w = holder(s, "static:app.Holder.WATCH");
    assert_eq!(w.retained, m.weak_shallow);
    assert_eq!(w.dominated_count, 1);
    // Threads, JNI globals and other roots.
    let t = holder(s, "thread:worker-1");
    assert_eq!(t.kind, HolderKind::Thread);
    assert_eq!(t.retained, m.thread_retained);
    assert_eq!(t.context.len(), 2);
    assert!(
        t.context[0].ends_with(".run(Worker.java:42)"),
        "{:?}",
        t.context
    );
    let j = holder(s, "jni-global:app.Blob");
    assert_eq!(j.kind, HolderKind::JniGlobal);
    assert_eq!(j.retained, m.jni_retained);
    let h = holder(s, "instance:vm-internal>app.Hub");
    assert_eq!(h.kind, HolderKind::Instance);
    assert_eq!(h.retained, m.hub_retained);
    // Roots by kind, including the extended kinds.
    for (k, n) in [
        ("thread-obj", 1),
        ("java-frame", 1),
        ("jni-global", 1),
        ("vm-internal", 1),
        ("interned-string", 1),
        ("jni-monitor", 1),
        ("unreachable", 1),
    ] {
        assert_eq!(
            s.gc_roots_by_kind.get(k),
            Some(&n),
            "{k}: {:?}",
            s.gc_roots_by_kind
        );
    }
    // Unreachable through strong references: the weak referent, the unreachable-root
    // object and the data-less array.
    assert_eq!(s.counts.unreachable, 3);
    assert_eq!(s.counts.classes, 15);
    assert_eq!(s.top_classes[0].class, "byte[]");
    assert!(s.top_classes.len() <= 25 && s.holders.len() <= 20);
    assert!(s.notes.iter().any(|n| n.contains("approximate")));
}

#[test]
fn model_heap_with_8_byte_ids() {
    let dir = tempfile::tempdir().unwrap();
    let m = model(8, 1_700_000_000_000, 300, 50);
    let p = write(dir.path(), "m8.hprof", &m.dump);
    let s = summarize(&p, &opts(dir.path())).unwrap();
    assert_eq!(s.header.timestamp_ms, 1_700_000_000_000);
    check_model(&s, &m, 8, 300);
    assert!(s.stats.compressed_oops);
}

#[test]
fn model_heap_with_4_byte_ids() {
    let dir = tempfile::tempdir().unwrap();
    let m = model(4, 5, 120, 20);
    let p = write(dir.path(), "m4.hprof", &m.dump);
    let s = summarize(&p, &opts(dir.path())).unwrap();
    check_model(&s, &m, 4, 120);
}

#[test]
fn gzip_input_gives_the_same_summary_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let m = model(8, 77, 200, 10);
    let plain = summarize(&write(dir.path(), "m.hprof", &m.dump), &opts(dir.path())).unwrap();
    // Detected by magic bytes, not by the name.
    let gz = write(dir.path(), "m.dump", &stored_gzip(&m.dump));
    let work = dir.path().join("work");
    let s = summarize(&gz, &opts(&work)).unwrap();
    assert!(s.stats.gzip);
    assert_eq!(s.holders, plain.holders);
    assert_eq!(s.counts, plain.counts);
    assert_eq!(s.top_classes, plain.top_classes);
    assert_eq!(
        fs::read_dir(&work).unwrap().count(),
        0,
        "temp file left behind"
    );
}

#[test]
fn system_gzip_levels_give_the_same_summary() {
    if Command::new("gzip").arg("--version").output().is_err() {
        eprintln!("skipped: gzip not found");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let m = model(8, 77, 400, 10);
    let raw = write(dir.path(), "m.hprof", &m.dump);
    let plain = summarize(&raw, &opts(dir.path())).unwrap();
    for level in ["-1", "-6", "-9"] {
        let out = Command::new("gzip")
            .args([level, "-c"])
            .arg(&raw)
            .output()
            .unwrap();
        assert!(out.status.success());
        let gz = write(dir.path(), &format!("m{level}.hprof.gz"), &out.stdout);
        let s = summarize(&gz, &opts(dir.path())).unwrap();
        assert_eq!(s.holders, plain.holders, "gzip {level}");
        assert_eq!(s.counts, plain.counts, "gzip {level}");
    }
}

#[test]
fn caps_set_truncated() {
    let dir = tempfile::tempdir().unwrap();
    let m = model(8, 1, 100, 10);
    let p = write(dir.path(), "m.hprof", &m.dump);
    let s = summarize(
        &p,
        &Options {
            max_objects: 50,
            ..opts(dir.path())
        },
    )
    .unwrap();
    assert!(s.truncated);
    assert!(s.holders.is_empty());
    assert!(
        s.warnings.iter().any(|w| w.contains("object cap")),
        "{:?}",
        s.warnings
    );
    let s = summarize(
        &p,
        &Options {
            max_passes: 1,
            ..opts(dir.path())
        },
    )
    .unwrap();
    assert!(s.truncated);
    assert!(
        s.warnings.iter().any(|w| w.contains("pass")),
        "{:?}",
        s.warnings
    );
    assert!(!s.holders.is_empty());
}

#[test]
fn bad_inputs_are_errors() {
    let dir = tempfile::tempdir().unwrap();
    let m = model(8, 1, 10, 2);
    let o = opts(dir.path());
    let e = summarize(
        &write(dir.path(), "a.hprof", b"not a dump at all, just text\n"),
        &o,
    )
    .unwrap_err();
    assert!(e.to_string().contains("not an HPROF"), "{e}");
    let e = summarize(&write(dir.path(), "b.hprof", b""), &o).unwrap_err();
    assert!(e.to_string().contains("not an HPROF"), "{e}");
    // Cut in the middle of a record.
    let cut = &m.dump[..m.dump.len() - 200];
    let e = summarize(&write(dir.path(), "c.hprof", cut), &o).unwrap_err();
    assert!(e.to_string().contains("truncated"), "{e}");
    // A gzip whose payload is not a dump, and a truncated gzip.
    let e = summarize(
        &write(dir.path(), "d.hprof.gz", &stored_gzip(b"hello world")),
        &o,
    )
    .unwrap_err();
    assert!(e.to_string().contains("not an HPROF"), "{e}");
    let gz = stored_gzip(&m.dump);
    let e = summarize(&write(dir.path(), "e.hprof.gz", &gz[..gz.len() / 2]), &o).unwrap_err();
    assert!(e.to_string().contains("inflate"), "{e}");
    let e = summarize(&dir.path().join("missing.hprof"), &o).unwrap_err();
    assert!(e.to_string().contains("i/o"), "{e}");
}

#[test]
fn unknown_sub_record_is_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut d = model(8, 1, 10, 2).dump;
    // Replace the unknown top-level record's tag with a heap segment of one bad
    // sub-record: tag 0x1c, length 3, sub-tag 0x77.
    let at = d.len() - 14 - 9; // the 0x99 record (9 header + 5 body) precedes HEAP_DUMP_END (9)
    assert_eq!(d[at], 0x99);
    d[at] = 0x1c;
    d[at + 9] = 0x77;
    let s = summarize(&write(dir.path(), "w.hprof", &d), &opts(dir.path())).unwrap();
    assert!(
        s.warnings
            .iter()
            .any(|w| w.contains("unknown heap sub-record 0x77")),
        "{:?}",
        s.warnings
    );
    assert_eq!(s.holders[0].key, "static:app.Holder.ITEMS");
}

// ---------------------------------------------------------------------------------------
// Real JVM dumps
// ---------------------------------------------------------------------------------------

fn check_journal(s: &HeapSummary, n: u64) {
    assert_eq!(s.header.id_size, 8);
    assert!(!s.truncated, "{:?}", s.warnings);
    let top = &s.holders[0];
    // Not ClassLoader.scl: the class-loader pitfall.
    assert_eq!(top.key, "static:ledger.Journal.ENTRIES");
    assert_eq!(top.kind, HolderKind::Static);
    // The list, its array, and an entry, a string and a byte array per entry.
    assert_eq!(top.dominated_count, 2 + 3 * n);
    // Entry 24 + String 24 + byte[] 24..32 per entry, plus the array and the list.
    let lo = 72 * n;
    let hi = 84 * n + 8 * n + 64;
    assert!(
        (lo..=hi).contains(&top.retained),
        "retained {} not in {lo}..={hi}",
        top.retained
    );
    let acc = top.accumulation.as_ref().unwrap();
    assert_eq!(acc.class, "java.lang.Object[]");
    assert_eq!(acc.children, n);
    // Reference.referent is not followed: the 64 KiB array is held only weakly.
    if let Some(w) = s
        .tracked
        .iter()
        .find(|h| h.key == "static:ledger.Journal.SCRATCH")
    {
        assert!(w.retained < 200, "{}", w.retained);
    }
    assert!(s.counts.unreachable >= 1);
    // The loader holder does not own the application class.
    if let Some(l) = s
        .tracked
        .iter()
        .find(|h| h.key == "static:java.lang.ClassLoader.scl")
    {
        assert!(l.retained < top.retained / 4, "{}", l.retained);
    }
    // Top classes are sane: entries counted exactly and sized like the JVM does.
    let e = s
        .top_classes
        .iter()
        .find(|c| c.class == "ledger.Journal$Entry")
        .unwrap();
    assert_eq!(e.count, n);
    assert_eq!(e.shallow, 24 * n);
    assert_eq!(s.top_classes[0].class, "byte[]");
}

#[test]
fn committed_journal_dump() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/journal.hprof.gz");
    let dir = tempfile::tempdir().unwrap();
    let s = summarize(&p, &opts(dir.path())).unwrap();
    assert!(s.stats.gzip);
    check_journal(&s, 2000);
}

fn have(tool: &str) -> bool {
    Command::new(tool).arg("-version").output().is_ok()
        || Command::new(tool).arg("-help").output().is_ok()
}

#[test]
fn generated_journal_dumps_when_a_jdk_is_present() {
    if !(have("javac") && have("jcmd") && have("java")) {
        eprintln!("skipped: javac, java and jcmd are needed");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src/ledger");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("Journal.java"), include_str!("data/Journal.java")).unwrap();
    let out = dir.path().join("out");
    let st = Command::new("javac")
        .arg("-d")
        .arg(&out)
        .arg(src.join("Journal.java"))
        .status()
        .unwrap();
    assert!(st.success());
    for n in [1500u64, 3000] {
        let dump = dir.path().join(format!("j{n}.hprof"));
        let mut child = Command::new("java")
            .args([
                "-Xmx16m",
                "-Xms16m",
                "-XX:+UseSerialGC",
                "--limit-modules",
                "java.base",
                "-cp",
            ])
            .arg(&out)
            .args(["ledger.Journal", &n.to_string()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut line = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut line)
            .unwrap();
        assert!(line.starts_with("ready"), "{line:?}");
        let st = Command::new("jcmd")
            .arg(child.id().to_string())
            .arg("GC.heap_dump")
            .arg(&dump)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        child.stdin.take().unwrap().flush().unwrap();
        drop(child.stdin.take());
        let _ = child.wait();
        assert!(st.success());
        let s = summarize(&dump, &opts(dir.path())).unwrap();
        check_journal(&s, n);
    }
}

// ---- hostile inputs

use common::hprof_builder::{B, T_INT};

fn many_fields(n: usize) -> Vec<(String, u8)> {
    (0..n).map(|i| (format!("f{i}"), T_INT)).collect()
}

fn class_with_fields(b: &mut B, id: u64, sup: u64, n: usize) {
    let owned = many_fields(n);
    let fields: Vec<(&str, u8)> = owned.iter().map(|(s, t)| (s.as_str(), *t)).collect();
    b.class_dump(id, sup, 0, 0, &fields, &[]);
}

fn expect_malformed(bytes: &[u8], what: &str) {
    let dir = tempfile::tempdir().unwrap();
    let t0 = std::time::Instant::now();
    let e = summarize(&write(dir.path(), "h.hprof", bytes), &opts(dir.path())).unwrap_err();
    assert!(e.to_string().contains(what), "{e}");
    assert!(
        matches!(e, dla_parsers::hprof::HprofError::Malformed(_)),
        "{e}"
    );
    assert!(t0.elapsed().as_secs() < 20, "took {:?}", t0.elapsed());
}

#[test]
fn self_super_class_with_many_fields_is_refused_quickly() {
    let mut b = B::new(8, 1);
    let c = b.declare("app.Loop");
    class_with_fields(&mut b, c, c, 65535);
    expect_malformed(&b.finish(1), "loops");
}

#[test]
fn super_cycle_is_refused_quickly() {
    let mut b = B::new(8, 1);
    let x = b.declare("app.X");
    let y = b.declare("app.Y");
    let z = b.declare("app.Z");
    class_with_fields(&mut b, x, y, 100);
    class_with_fields(&mut b, y, z, 100);
    class_with_fields(&mut b, z, x, 100);
    expect_malformed(&b.finish(1), "loops");
}

#[test]
fn a_chain_with_huge_field_counts_is_refused_quickly() {
    let mut b = B::new(8, 1);
    let ids: Vec<u64> = (0..20).map(|i| b.declare(&format!("app.C{i}"))).collect();
    for (i, &id) in ids.iter().enumerate() {
        let sup = ids.get(i + 1).copied().unwrap_or(0);
        class_with_fields(&mut b, id, sup, 65535);
    }
    expect_malformed(&b.finish(1), "instance fields");
}

#[test]
fn a_gzip_that_inflates_past_the_limit_is_refused_and_cleaned_up() {
    let dir = tempfile::tempdir().unwrap();
    let m = model(8, 5, 200, 10);
    let gz = write(dir.path(), "m.hprof.gz", &stored_gzip(&m.dump));
    let work = dir.path().join("work");
    let o = Options {
        max_inflated: Some(1000),
        ..opts(&work)
    };
    let e = summarize(&gz, &o).unwrap_err();
    assert!(
        matches!(e, dla_parsers::hprof::HprofError::InflateTooLarge(1000)),
        "{e}"
    );
    assert_eq!(fs::read_dir(&work).unwrap().count(), 0, "temp file left");
    // The default limit (a multiple of the compressed size, at least 64 MiB) passes it.
    assert!(summarize(&gz, &opts(&work)).is_ok());
}

#[test]
fn a_real_deflate_bomb_stops_at_the_default_ratio() {
    // 1 GiB of zeros as ~1 MiB of deflate (stored-free, fixed-Huffman runs would be
    // long to build; use max-length matches of a dynamic-free fixed block instead).
    let dir = tempfile::tempdir().unwrap();
    // Fixed-Huffman block: literal 0, then repeated (length 258, distance 1) matches.
    let mut bits: Vec<bool> = vec![true, true, false]; // BFINAL=1, BTYPE=01 (LSB first)
    let push_code = |bits: &mut Vec<bool>, code: u32, len: u32| {
        for i in (0..len).rev() {
            bits.push((code >> i) & 1 == 1);
        }
    };
    push_code(&mut bits, 0x30, 8); // literal 0
    let total_matches = 1_000_000usize; // 258 MB
    for _ in 0..total_matches {
        push_code(&mut bits, 0xc5, 8); // length symbol 285 (258)
        push_code(&mut bits, 0, 5); // distance code 0 (1)
    }
    push_code(&mut bits, 0, 7); // end of block
    let mut raw = vec![0u8; bits.len().div_ceil(8)];
    for (i, b) in bits.iter().enumerate() {
        if *b {
            raw[i / 8] |= 1 << (i % 8);
        }
    }
    let mut gz = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3];
    gz.extend_from_slice(&raw);
    gz.extend_from_slice(&[0; 8]); // trailer is never reached
    let p = write(dir.path(), "bomb.hprof.gz", &gz);
    let work = dir.path().join("work");
    let o = Options {
        max_inflated: Some(1 << 20),
        ..opts(&work)
    };
    let t0 = std::time::Instant::now();
    let e = summarize(&p, &o).unwrap_err();
    assert!(
        matches!(e, dla_parsers::hprof::HprofError::InflateTooLarge(_)),
        "{e}"
    );
    assert!(t0.elapsed().as_secs() < 20);
    assert_eq!(fs::read_dir(&work).unwrap().count(), 0, "temp file left");
}

#[test]
fn two_adjacent_statics_to_one_object_are_both_holders() {
    let mut b = B::new(8, 1);
    let obj = b.declare("java.lang.Object");
    let host = b.declare("app.Host");
    let payload = b.byte_array(&[1u8; 4000]);
    b.class_dump(obj, 0, 0, 0, &[], &[]);
    b.class_dump(host, obj, 0, 0, &[], &[("A", payload), ("B", payload)]);
    b.root(0x05, host, 0, 0);
    let dir = tempfile::tempdir().unwrap();
    let s = summarize(
        &write(dir.path(), "s.hprof", &b.finish(1)),
        &opts(dir.path()),
    )
    .unwrap();
    let keys: Vec<&str> = s.tracked.iter().map(|h| h.key.as_str()).collect();
    assert!(keys.contains(&"static:app.Host.A"), "{keys:?}");
    assert!(keys.contains(&"static:app.Host.B"), "{keys:?}");
}
