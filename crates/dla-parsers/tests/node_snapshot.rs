//! Real V8 snapshots (trimmed to the nodes around a leak): the class histogram, dominators,
//! retention paths and the holders with their classification.

use std::{path::PathBuf, time::Instant};

use dla_parsers::node::{Holder, HolderKind, HolderOptions, HolderSummary, Options, Snapshot};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/node")
        .join(name)
}

fn summary(name: &str) -> (Snapshot, HolderSummary) {
    let s = Snapshot::read_file(&fixture(name), &Options::default()).unwrap();
    let dom = s.dominators(None).unwrap();
    let bfs = s.bfs();
    let h = s.holders(&dom, &bfs, &HolderOptions::default());
    (s, h)
}

fn find<'a>(h: &'a HolderSummary, kind: HolderKind, name: &str) -> &'a Holder {
    h.holders
        .iter()
        .find(|x| x.kind == kind && x.name == name)
        .unwrap_or_else(|| panic!("no {kind:?} holder {name}: {:?}", h.holders))
}

#[test]
fn a_module_variable_is_a_static_holder_and_names_its_module() {
    let (_, h) = summary("retry-3.heapsnapshot");
    let shared = find(&h, HolderKind::Static, "shared");
    assert_eq!(
        shared.script.as_deref(),
        Some("/srv/webhook/src/scheduler.js")
    );
    assert!(shared.owner.is_empty());
    // The scheduler keeps its chain of attempt records alive: most of the heap below it.
    assert!(shared.retained > 30_000, "{}", shared.retained);
    assert!(shared.count > 600, "{}", shared.count);
    assert_eq!(shared.accumulation.as_ref().unwrap().class, "Object");
    // The path ends at the variable of a context.
    let last = shared.path.last().unwrap();
    assert_eq!(
        (last.node.as_str(), last.edge.as_str()),
        ("system / Context", ".shared")
    );
    // The field below it is a container field of the class, in the same module.
    let latest = find(&h, HolderKind::Instance, "latest");
    assert_eq!(latest.owner, "RetryScheduler");
    assert_eq!(
        latest.script.as_deref(),
        Some("/srv/webhook/src/scheduler.js")
    );
    // A link of the chain (an Object whose `previous` is an Object) is not a holder.
    assert!(h.holders.iter().all(|x| x.name != "previous"));
}

#[test]
fn growth_between_snapshots_shows_in_the_holder() {
    let (_, a) = summary("retry-1.heapsnapshot");
    let (_, b) = summary("retry-3.heapsnapshot");
    let ra = find(&a, HolderKind::Static, "shared");
    let rb = find(&b, HolderKind::Static, "shared");
    assert!(
        rb.retained > 2 * ra.retained,
        "{} {}",
        ra.retained,
        rb.retained
    );
    assert!(rb.count > ra.count);
    // Object ids only grow within one process.
    assert!(b.max_id > a.max_id);
}

#[test]
fn a_timer_callback_is_a_registration_and_names_the_function_that_started_it() {
    let (_, h) = summary("hb-3.heapsnapshot");
    let beat = find(&h, HolderKind::Registration, "beat");
    assert_eq!(beat.registry.as_deref(), Some("timers"));
    // The function that called setInterval comes from the scope info of the callback's
    // context, confirmed by a function of that name in the same script.
    assert_eq!(beat.owner, "startHeartbeat");
    assert_eq!(
        beat.script.as_deref(),
        Some("/srv/gateway/src/heartbeat.js")
    );
    assert!(beat.instances >= 20, "one per session: {}", beat.instances);
    // Each session's variable in the callback's context is a closure holder.
    let session = find(&h, HolderKind::Closure, "session");
    assert_eq!(session.owner, "startHeartbeat");
    assert!(session.retained <= beat.retained);
}

#[test]
fn dominator_sizes_add_up_over_the_reachable_heap() {
    let s = Snapshot::read_file(&fixture("retry-3.heapsnapshot"), &Options::default()).unwrap();
    let dom = s.dominators(None).unwrap();
    let total: u64 = (0..s.node_count())
        .filter(|&n| dom.reachable(n))
        .map(|n| s.self_size(n))
        .sum();
    assert_eq!(dom.total(), total);
    assert_eq!(dom.dominated(0) as usize, dom.reachable_count());
    // A dominator never retains less than what it dominates directly.
    for n in 1..s.node_count() {
        if let Some(d) = dom.idom(n) {
            assert!(dom.retained(d) >= dom.retained(n));
        }
    }
    // Every reachable node has a retention path that ends at it.
    let bfs = s.bfs();
    for n in (1..s.node_count()).filter(|&n| dom.reachable(n)).take(300) {
        let path = bfs.path_to(n);
        assert!(!path.is_empty());
        assert_eq!(path[0].0, 0);
    }
}

#[test]
fn the_class_histogram_counts_nodes_without_naming_strings() {
    let s = Snapshot::read_file(&fixture("retry-3.heapsnapshot"), &Options::default()).unwrap();
    let h = s.histogram();
    assert_eq!(h.source, "v8-heapsnapshot");
    let total: u64 = h.rows.iter().map(|r| r.count).sum();
    assert_eq!(total as usize, s.node_count());
    assert!(
        h.rows
            .iter()
            .any(|r| r.name == "RetryScheduler" && r.count == 1)
    );
    assert!(h.rows.iter().any(|r| r.name == "(string)"));
    assert!(h.rows.iter().any(|r| r.name.starts_with("(closure) ")));
    assert!(h.rows.windows(2).all(|w| w[0].bytes >= w[1].bytes));
}

#[test]
fn a_truncated_or_inconsistent_file_is_an_error_not_a_panic() {
    let bytes = std::fs::read(fixture("retry-3.heapsnapshot")).unwrap();
    for cut in [10, bytes.len() / 3, bytes.len() / 2, bytes.len() - 20] {
        assert!(
            Snapshot::read(&bytes[..cut], bytes.len() as u64).is_err(),
            "cut at {cut}"
        );
    }
    // A string table that is missing is a truncated write.
    let text = String::from_utf8(bytes).unwrap();
    let at = text.find("\"strings\"").unwrap();
    let no_strings = format!("{}}}", text[..at].trim_end_matches(','));
    assert!(Snapshot::read(no_strings.as_bytes(), no_strings.len() as u64).is_err());
}

#[test]
fn corrupted_snapshots_never_panic() {
    let base = std::fs::read(fixture("hb-3.heapsnapshot")).unwrap();
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as usize
    };
    let mut loaded = 0;
    for round in 0..400 {
        let mut bytes = base.clone();
        for _ in 0..1 + round % 4 {
            let at = next() % bytes.len();
            bytes[at] = b"0123456789,[]\"x"[next() % 15];
        }
        // Whatever the read says, nothing may panic, overflow or hang afterwards.
        if let Ok(s) = Snapshot::read(&bytes[..], bytes.len() as u64) {
            loaded += 1;
            let dom = s.dominators(None).unwrap();
            let bfs = s.bfs();
            let _ = s.holders(&dom, &bfs, &HolderOptions::default());
            let _ = s.histogram();
        }
    }
    // Most single-byte changes of an array leave a loadable file (a number changed); the
    // ones that break the structure are refused.
    assert!(loaded > 0 && loaded < 400, "{loaded}");
}

/// Prints the holders of the snapshot named by `DLA_NODE_SNAPSHOT` (a manual check on a
/// large file; the unit tests above use the trimmed fixtures).
#[test]
#[ignore = "manual: needs DLA_NODE_SNAPSHOT"]
fn explore_a_snapshot() {
    let Some(p) = std::env::var_os("DLA_NODE_SNAPSHOT") else {
        return;
    };
    let t = Instant::now();
    let s = Snapshot::read_file(&PathBuf::from(p), &Options::default()).unwrap();
    let load = t.elapsed();
    let dom = s.dominators(None).unwrap();
    let bfs = s.bfs();
    let h = s.holders(&dom, &bfs, &HolderOptions::default());
    eprintln!(
        "load {load:?}, total {:?}, nodes {}",
        t.elapsed(),
        s.node_count()
    );
    for x in h.holders.iter().take(12) {
        eprintln!(
            "{:?} {:?} owner={:?} name={:?} script={:?} ret={} n={} inst={} acc={:?}\n    {:?}",
            x.kind,
            x.registry,
            x.owner,
            x.name,
            x.script,
            x.retained,
            x.count,
            x.instances,
            x.accumulation,
            x.path
        );
    }
}
