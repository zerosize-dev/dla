//! Via holders (shared retention), deproxied holder names and the full class histogram,
//! on small synthetic heaps with known shapes.

mod common;

use std::{fs, path::Path};

use common::hprof_builder::{B, F, T_BYTE, T_OBJ};
use dla_parsers::hprof::{HeapSummary, HolderKind, Options, summarize};

const N: usize = 100;

fn run(dir: &Path, name: &str, bytes: &[u8]) -> HeapSummary {
    let p = dir.join(name);
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

/// Boilerplate classes and the thread of `main`; returns (builder, ids...).
struct Base {
    b: B,
    obj: u64,
    class_cls: u64,
    string: u64,
    thread: u64,
    obj_arr: u64,
    byte_arr: u64,
    cont: u64,
    loader: u64,
}

fn base(cont_name: &str) -> Base {
    let mut b = B::new(8, 1_700_000_000_000);
    let obj = b.declare("java.lang.Object");
    let class_cls = b.declare("java.lang.Class");
    let string = b.declare("java.lang.String");
    let thread = b.declare("java.lang.Thread");
    let obj_arr = b.declare("[Ljava/lang/Object;");
    let byte_arr = b.declare("[B");
    let cont = b.declare(cont_name);
    let loader_cls = b.declare("jdk.internal.loader.ClassLoaders$AppClassLoader");
    let loader = b.instance(loader_cls, &[]);
    b.class_dump(loader_cls, obj, 0, 8, &[], &[]);
    Base {
        b,
        obj,
        class_cls,
        string,
        thread,
        obj_arr,
        byte_arr,
        cont,
        loader,
    }
}

fn conts(x: &mut Base) -> Vec<u64> {
    (0..N)
        .map(|i| {
            let payload = x.b.byte_array(&vec![(i % 200) as u8; 2000]);
            x.b.instance(x.cont, &[F::Id(payload)])
        })
        .collect()
}

fn thread_main(x: &mut Base, local: u64) {
    let tname = x.b.java_string(x.string, "main");
    let tobj = x.b.instance(x.thread, &[F::Id(tname)]);
    x.b.root(0x08, tobj, 1, 5);
    x.b.root(0x03, local, 1, 0);
    x.b.frame(0x50, "runBlocking", "Builders.kt", 1, 40);
    x.b.trace(5, 1, &[0x50]);
}

fn common_classes(x: &mut Base) {
    x.b.class_dump(x.obj, 0, 0, 0, &[], &[]);
    x.b.class_dump(x.class_cls, x.obj, 0, 0, &[], &[]);
    x.b.class_dump(
        x.string,
        x.obj,
        0,
        0,
        &[("value", T_OBJ), ("coder", T_BYTE), ("hash", 10)],
        &[],
    );
    x.b.class_dump(x.thread, x.obj, 0, 0, &[("name", T_OBJ)], &[]);
    x.b.class_dump(x.obj_arr, x.obj, 0, 0, &[], &[]);
    x.b.class_dump(x.byte_arr, x.obj, 0, 0, &[], &[]);
    x.b.class_dump(x.cont, x.obj, x.loader, 0, &[("payload", T_OBJ)], &[]);
}

/// The coroutine case: continuations reachable from a static map and from the job tree
/// of the main thread, so no single field dominates them.
fn coroutine_heap() -> Vec<u8> {
    let mut x = base("shop.LeakyKt$handle$1");
    let cs = conts(&mut x);
    let registry_cls = x.b.declare("shop.Registry");
    let job_cls = x.b.declare("shop.Job");
    let waiters_cls = x.b.declare("shop.Waiters");
    let table = x.b.obj_array(x.obj_arr, &cs);
    let registry = x.b.instance(registry_cls, &[F::Id(table)]);
    let children = x.b.obj_array(x.obj_arr, &cs);
    let job = x.b.instance(job_cls, &[F::Id(children)]);
    thread_main(&mut x, job);
    common_classes(&mut x);
    x.b.class_dump(registry_cls, x.obj, 0, 0, &[("table", T_OBJ)], &[]);
    x.b.class_dump(job_cls, x.obj, 0, 0, &[("children", T_OBJ)], &[]);
    x.b.class_dump(waiters_cls, x.obj, x.loader, 0, &[], &[("gates", registry)]);
    x.b.finish(2)
}

#[test]
fn shared_retention_gets_a_static_via_holder() {
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), "co.hprof", &coroutine_heap());
    // The dominator-based holders miss the continuations: the static holds only the
    // registry and its table.
    let waiters = s
        .tracked
        .iter()
        .find(|h| h.key == "static:shop.Waiters.gates")
        .expect("static holder");
    assert!(waiters.retained < 3_000, "{}", waiters.retained);
    // The via holder finds the static on the shortest path and keeps the class.
    assert_eq!(s.via_holders.len(), 1, "{:?}", s.via_holders);
    let v = &s.via_holders[0];
    assert_eq!(v.key, "static:shop.Waiters.gates");
    assert_eq!(v.kind, HolderKind::Static);
    assert_eq!(v.owner_class, "shop.Waiters");
    assert_eq!(v.field, "gates");
    assert_eq!(v.dominator, "root");
    assert_eq!(v.accumulation.class, "shop.LeakyKt$handle$1");
    assert_eq!(v.accumulation.children, N as u64);
    assert!(v.retained >= (N as u64) * 2000, "{}", v.retained);
    assert_eq!(v.path[0], "root class shop.Waiters");
    assert!(
        v.path[1].starts_with("static gates -> shop.Registry"),
        "{:?}",
        v.path
    );
    assert!(
        v.path.last().unwrap().ends_with("shop.LeakyKt$handle$1"),
        "{:?}",
        v.path
    );
    // It is part of the JSON summary.
    let j = serde_json::to_string(&s).unwrap();
    assert!(j.contains("\"via_holders\""));
}

/// Two instance roots reach the set; one is a proxy class. No static field is involved.
#[test]
fn instance_via_holder_is_deproxied_and_keeps_the_raw_name() {
    let mut x = base("shop.Task");
    let cs = conts(&mut x);
    let hub_cls = x.b.declare("shop.Hub$$SpringCGLIB$$0");
    let job_cls = x.b.declare("shop.Job");
    let data = x.b.obj_array(x.obj_arr, &cs);
    let hub = x.b.instance(hub_cls, &[F::Id(data)]);
    x.b.root(0x8d, hub, 0, 0);
    let children = x.b.obj_array(x.obj_arr, &cs);
    let job = x.b.instance(job_cls, &[F::Id(children)]);
    thread_main(&mut x, job);
    common_classes(&mut x);
    x.b.class_dump(hub_cls, x.obj, 0, 0, &[("data", T_OBJ)], &[]);
    x.b.class_dump(job_cls, x.obj, 0, 0, &[("children", T_OBJ)], &[]);
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), "inst.hprof", &x.b.finish(1));
    let v = s.via_holders.first().expect("a via holder");
    assert_eq!(v.key, "field:shop.Hub.data");
    assert_eq!(v.kind, HolderKind::Instance);
    assert_eq!(v.owner_class, "shop.Hub");
    assert_eq!(
        v.raw_owner_class.as_deref(),
        Some("shop.Hub$$SpringCGLIB$$0")
    );
    assert_eq!(v.accumulation.class, "shop.Task");
    assert_eq!(v.accumulation.children, N as u64);
    assert!(
        v.path[0].starts_with("root instance shop.Hub$$SpringCGLIB$$0"),
        "{:?}",
        v.path
    );
    // The root holder of the proxy is normalized too, with the raw class kept.
    let h = s
        .tracked
        .iter()
        .find(|h| h.key == "instance:vm-internal>shop.Hub")
        .expect("normalized instance holder");
    assert_eq!(h.owner_class, "shop.Hub");
    assert_eq!(
        h.raw_owner_class.as_deref(),
        Some("shop.Hub$$SpringCGLIB$$0")
    );
}

/// Frame locals of one thread hold the set directly: there is no field to report.
#[test]
fn thread_local_roots_have_no_via_field() {
    let mut x = base("shop.Task");
    let cs = conts(&mut x);
    let tname = x.b.java_string(x.string, "main");
    let tobj = x.b.instance(x.thread, &[F::Id(tname)]);
    x.b.root(0x08, tobj, 1, 5);
    for c in &cs {
        x.b.root(0x03, *c, 1, 0);
    }
    common_classes(&mut x);
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), "locals.hprof", &x.b.finish(1));
    assert!(s.via_holders.is_empty(), "{:?}", s.via_holders);
    assert!(s.tracked.iter().any(|h| h.key == "thread:main"));
}

/// Sessions with a payload of `bytes`, `n` of them.
fn sessions(x: &mut Base, n: usize, bytes: usize) -> Vec<u64> {
    (0..n)
        .map(|i| {
            let payload = x.b.byte_array(&vec![(i % 200) as u8; bytes]);
            x.b.instance(x.cont, &[F::Id(payload)])
        })
        .collect()
}

/// Two statics hold disjoint parts of one shared class; each gets only its own bytes.
#[test]
fn via_bytes_are_split_per_field() {
    let mut x = base("shop.Session");
    let a = sessions(&mut x, 10, 20_000);
    let bs = sessions(&mut x, 60, 20_000);
    let all: Vec<u64> = a.iter().chain(&bs).copied().collect();
    let arr_a = x.b.obj_array(x.obj_arr, &a);
    let arr_b = x.b.obj_array(x.obj_arr, &bs);
    let jobs = x.b.obj_array(x.obj_arr, &all);
    let cache_a = x.b.declare("shop.CacheA");
    let cache_b = x.b.declare("shop.CacheB");
    thread_main(&mut x, jobs);
    common_classes(&mut x);
    x.b.class_dump(cache_a, x.obj, x.loader, 0, &[], &[("items", arr_a)]);
    x.b.class_dump(cache_b, x.obj, x.loader, 0, &[], &[("items", arr_b)]);
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), "split.hprof", &x.b.finish(1));
    let get = |k: &str| s.via_holders.iter().find(|v| v.key == k);
    let va = get("static:shop.CacheA.items").unwrap_or_else(|| panic!("{:?}", s.via_holders));
    let vb = get("static:shop.CacheB.items").unwrap_or_else(|| panic!("{:?}", s.via_holders));
    assert_eq!(va.accumulation.children, 10);
    assert_eq!(vb.accumulation.children, 60);
    assert!(
        va.retained >= 200_000 && va.retained < 260_000,
        "{}",
        va.retained
    );
    assert!(
        vb.retained >= 1_200_000 && vb.retained < 1_400_000,
        "{}",
        vb.retained
    );
}

/// HotSpot's `<resolved_references>` pseudo static is not a holder.
#[test]
fn pseudo_statics_are_not_via_holders() {
    let mut x = base("shop.Session");
    let strs: Vec<u64> = (0..100)
        .map(|i| x.b.java_string(x.string, &format!("{i:0>2000}")))
        .collect();
    let refs = x.b.obj_array(x.obj_arr, &strs);
    let jobs = x.b.obj_array(x.obj_arr, &strs);
    let owner = x.b.declare("shop.Owner");
    thread_main(&mut x, jobs);
    common_classes(&mut x);
    x.b.class_dump(
        owner,
        x.obj,
        x.loader,
        0,
        &[],
        &[("<resolved_references>", refs)],
    );
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), "pseudo.hprof", &x.b.finish(1));
    assert!(
        s.via_holders.iter().all(|v| !v.key.contains('<')),
        "{:?}",
        s.via_holders
    );
    assert!(s.via_holders.is_empty(), "{:?}", s.via_holders);
}

/// A list longer than the old 4096-node walk cap still reports the static that starts it.
#[test]
fn long_chain_keeps_the_real_root() {
    let mut x = base("shop.Session");
    let node_cls = x.b.declare("shop.Node");
    let reg_cls = x.b.declare("shop.Registry");
    let mut next = 0u64;
    let mut nodes = Vec::new();
    for i in 0..6000usize {
        let payload = x.b.byte_array(&[(i % 200) as u8; 100]);
        next = x.b.instance(node_cls, &[F::Id(next), F::Id(payload)]);
        nodes.push(next);
    }
    for n in &nodes {
        x.b.root(0x01, *n, 0, 0);
    }
    common_classes(&mut x);
    x.b.class_dump(
        node_cls,
        x.obj,
        0,
        0,
        &[("next", T_OBJ), ("payload", T_OBJ)],
        &[],
    );
    x.b.class_dump(reg_cls, x.obj, x.loader, 0, &[], &[("head", next)]);
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), "chain.hprof", &x.b.finish(1));
    let v = s
        .via_holders
        .first()
        .unwrap_or_else(|| panic!("{:?}", s.via_holders));
    assert_eq!(v.key, "static:shop.Registry.head", "{:?}", v.path);
    assert_eq!(v.path[0], "root class shop.Registry");
}

#[test]
fn full_histogram_is_exposed() {
    let dir = tempfile::tempdir().unwrap();
    let s = run(dir.path(), "co2.hprof", &coroutine_heap());
    assert!(s.histogram.len() >= s.top_classes.len());
    assert_eq!(&s.histogram[..s.top_classes.len()], &s.top_classes[..]);
    let h = s.to_histogram();
    assert_eq!(h.source, "hprof");
    assert_eq!(h.timestamp_ms, Some(1_700_000_000_000));
    let cont = h
        .rows
        .iter()
        .find(|r| r.name == "shop.LeakyKt$handle$1")
        .unwrap();
    assert_eq!(cont.count, N as u64);
    let bytes = h.rows.iter().find(|r| r.name == "byte[]").unwrap();
    assert!(bytes.count >= N as u64);
    // The histogram is not part of the summary file.
    assert!(!serde_json::to_string(&s).unwrap().contains("\"histogram\""));
}

/// Manual check on a real dump: `DLA_HPROF=/path/x.hprof cargo test -p dla-parsers --test
/// hprof_via -- --ignored --nocapture`.
#[test]
#[ignore = "needs DLA_HPROF"]
fn print_via_holders_of_a_real_dump() {
    let Ok(p) = std::env::var("DLA_HPROF") else {
        return;
    };
    let s = summarize(Path::new(&p), &Options::default()).unwrap();
    println!("{}", serde_json::to_string_pretty(&s.via_holders).unwrap());
    for h in s.holders.iter().take(6) {
        println!("{} {} {:?}", h.key, h.retained, h.raw_owner_class);
    }
}
