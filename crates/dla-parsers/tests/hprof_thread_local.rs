//! Thread-local values: retention through `Thread.threadLocals` -> `ThreadLocalMap.table` ->
//! `Entry.value` is attributed to the static field that holds the `ThreadLocal`, on small
//! synthetic heaps with known shapes.

mod common;

use std::{fs, path::Path};

use common::hprof_builder::{B, F, T_BYTE, T_OBJ};
use dla_parsers::hprof::{HeapSummary, HolderKind, Options, summarize};

fn run(dir: &Path, bytes: &[u8]) -> HeapSummary {
    let p = dir.join("tl.hprof");
    fs::write(&p, bytes).unwrap();
    let s = summarize(
        &p,
        &Options {
            tmp_dir: Some(dir.to_path_buf()),
            ..Options::default()
        },
    )
    .unwrap();
    assert!(!s.truncated, "{:?}", s.warnings);
    s
}

/// Two pooled threads, each with one thread-local value (a buffer of `bytes`). The
/// ThreadLocal is held by a static field of `shop.Ctx` when `in_static`, else by an
/// instance field of `shop.Holder` that a third root keeps.
fn heap(in_static: bool, bytes: usize) -> Vec<u8> {
    build(in_static, false, bytes)
}

/// As [`heap`]; with `pooled` an executor (`shop.Pool.WORKERS`) also reaches the threads, as a
/// thread pool does, so that no thread is dominated by its own root.
fn build(in_static: bool, pooled: bool, bytes: usize) -> Vec<u8> {
    build_ex(in_static, pooled, bytes, false, false)
}

/// As [`build`]; `alias` also holds the ThreadLocal in a static field of `shop.Holder`, and
/// `inherit` makes each thread's `inheritableThreadLocals` the same map as its `threadLocals`.
fn build_ex(in_static: bool, pooled: bool, bytes: usize, alias: bool, inherit: bool) -> Vec<u8> {
    let mut b = B::new(8, 1_700_000_000_000);
    let obj = b.declare("java.lang.Object");
    let class_cls = b.declare("java.lang.Class");
    let string = b.declare("java.lang.String");
    let thread = b.declare("java.lang.Thread");
    let obj_arr = b.declare("[Ljava/lang/Object;");
    let byte_arr = b.declare("[B");
    let reference = b.declare("java.lang.ref.Reference");
    let weak = b.declare("java.lang.ref.WeakReference");
    let entry = b.declare("java.lang.ThreadLocal$ThreadLocalMap$Entry");
    let map = b.declare("java.lang.ThreadLocal$ThreadLocalMap");
    let tl_cls = b.declare("java.lang.ThreadLocal");
    let buf_cls = b.declare("shop.Buffer");
    let ctx = b.declare("shop.Ctx");
    let holder_cls = b.declare("shop.Holder");
    let pool_cls = b.declare("shop.Pool");
    let loader_cls = b.declare("jdk.internal.loader.ClassLoaders$AppClassLoader");
    let loader = b.instance(loader_cls, &[]);
    b.class_dump(loader_cls, obj, 0, 8, &[], &[]);

    let tl = b.instance(tl_cls, &[]);
    let mut threads = Vec::new();
    for (i, name) in ["pool-1", "pool-2"].into_iter().enumerate() {
        let payload = b.byte_array(&vec![i as u8; bytes]);
        let value = b.instance(buf_cls, &[F::Id(payload)]);
        // Entry data: its own `value`, then the inherited `referent`.
        let e = b.instance(entry, &[F::Id(value), F::Id(tl)]);
        let table = b.obj_array(obj_arr, &[e]);
        let m = b.instance(map, &[F::Id(table)]);
        let tname = b.java_string(string, name);
        let inh = if inherit { m } else { 0 };
        let t = b.instance(thread, &[F::Id(tname), F::Id(m), F::Id(inh)]);
        b.root(0x08, t, i as u32 + 1, 5);
        threads.push(t);
    }
    let workers = b.obj_array(obj_arr, &threads);
    if pooled {
        b.class_dump(pool_cls, obj, loader, 0, &[], &[("WORKERS", workers)]);
    }
    b.class_dump(obj, 0, 0, 0, &[], &[]);
    b.class_dump(class_cls, obj, 0, 0, &[], &[]);
    b.class_dump(
        string,
        obj,
        0,
        0,
        &[("value", T_OBJ), ("coder", T_BYTE), ("hash", 10)],
        &[],
    );
    b.class_dump(
        thread,
        obj,
        0,
        0,
        &[
            ("name", T_OBJ),
            ("threadLocals", T_OBJ),
            ("inheritableThreadLocals", T_OBJ),
        ],
        &[],
    );
    b.class_dump(obj_arr, obj, 0, 0, &[], &[]);
    b.class_dump(byte_arr, obj, 0, 0, &[], &[]);
    b.class_dump(reference, obj, 0, 0, &[("referent", T_OBJ)], &[]);
    b.class_dump(weak, reference, 0, 0, &[], &[]);
    b.class_dump(entry, weak, 0, 0, &[("value", T_OBJ)], &[]);
    b.class_dump(map, obj, 0, 0, &[("table", T_OBJ)], &[]);
    b.class_dump(tl_cls, obj, 0, 0, &[], &[]);
    b.class_dump(buf_cls, obj, loader, 0, &[("payload", T_OBJ)], &[]);
    if in_static {
        b.class_dump(ctx, obj, loader, 0, &[], &[("BUF", tl)]);
        let alias_statics: Vec<(&str, u64)> = if alias { vec![("CTX", tl)] } else { vec![] };
        b.class_dump(holder_cls, obj, loader, 0, &[("tl", T_OBJ)], &alias_statics);
    } else {
        let h = b.instance(holder_cls, &[F::Id(tl)]);
        b.root(0x05, h, 0, 0);
        b.class_dump(ctx, obj, loader, 0, &[], &[]);
        b.class_dump(holder_cls, obj, loader, 0, &[("tl", T_OBJ)], &[]);
    }
    b.finish(2)
}

#[test]
fn values_on_pooled_threads_belong_to_the_static_threadlocal_field() {
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), &heap(true, 5000));
    let h = s
        .tracked
        .iter()
        .find(|h| h.key == "thread-local:shop.Ctx.BUF")
        .unwrap_or_else(|| panic!("no thread-local holder: {:?}", s.tracked));
    assert_eq!(h.kind, HolderKind::ThreadLocal);
    assert_eq!(h.owner_class, "shop.Ctx");
    assert_eq!(h.field.as_deref(), Some("BUF"));
    // Both buffers (2 x 5000 bytes) are counted once, under the one key.
    assert!(h.retained >= 10_000, "{}", h.retained);
    assert!(h.dominated_count >= 4, "{}", h.dominated_count);
    assert!(h.path[0].contains("shop.Ctx.BUF"), "{:?}", h.path);
    // The threads no longer hold those bytes.
    for t in s.tracked.iter().filter(|h| h.kind == HolderKind::Thread) {
        assert!(t.retained < 2_000, "{} {}", t.key, t.retained);
    }
    assert!(s.tracked.iter().any(|h| h.key == "thread:pool-1"));
}

#[test]
fn the_holder_grows_with_the_values_and_keeps_its_key() {
    let dir = tempfile::tempdir().unwrap();
    let small = run(dir.path(), &heap(true, 2000));
    let big = run(dir.path(), &heap(true, 8000));
    let find = |s: &HeapSummary| {
        s.tracked
            .iter()
            .find(|h| h.key == "thread-local:shop.Ctx.BUF")
            .map(|h| h.retained)
            .unwrap()
    };
    assert!(find(&big) > find(&small) + 10_000);
}

#[test]
fn a_threadlocal_that_no_static_field_holds_stays_with_its_thread() {
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), &heap(false, 5000));
    assert!(
        s.tracked.iter().all(|h| h.kind != HolderKind::ThreadLocal),
        "{:?}",
        s.tracked
    );
    let t = s.tracked.iter().find(|h| h.key == "thread:pool-1").unwrap();
    assert!(t.retained >= 5_000, "{}", t.retained);
}

#[test]
fn values_on_threads_that_an_executor_also_reaches_are_still_attributed() {
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), &build(true, true, 5000));
    let h = s
        .tracked
        .iter()
        .find(|h| h.key == "thread-local:shop.Ctx.BUF")
        .unwrap_or_else(|| panic!("no thread-local holder: {:?}", s.tracked));
    assert!(h.retained >= 10_000, "{}", h.retained);
    // The threads are shared (the executor reaches them), so they retain nothing themselves
    // and the values are counted once, under the static field.
    for t in s.tracked.iter().filter(|h| h.kind == HolderKind::Thread) {
        assert!(t.retained < 2_000, "{} {}", t.key, t.retained);
    }
}

#[test]
fn a_value_in_both_thread_maps_is_counted_once() {
    let dir = tempfile::tempdir().unwrap();
    let once = run(dir.path(), &build_ex(true, false, 5000, false, false));
    let twice = run(dir.path(), &build_ex(true, false, 5000, false, true));
    let get = |s: &HeapSummary| {
        let h = s
            .tracked
            .iter()
            .find(|h| h.key == "thread-local:shop.Ctx.BUF")
            .unwrap();
        (h.retained, h.dominated_count)
    };
    assert_eq!(get(&once), get(&twice));
}

#[test]
fn a_threadlocal_held_by_two_static_fields_is_not_credited_to_either() {
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), &build_ex(true, false, 5000, true, false));
    assert!(
        s.tracked.iter().all(|h| h.kind != HolderKind::ThreadLocal),
        "{:?}",
        s.tracked
    );
    // The values stay with their threads.
    assert!(
        s.tracked
            .iter()
            .any(|h| h.kind == HolderKind::Thread && h.retained >= 5_000)
    );
}
