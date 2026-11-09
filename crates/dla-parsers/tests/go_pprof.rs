//! Real Go profiles (go1.27.0): the pprof reader, heap profiles as stacks and histograms,
//! goroutine profiles, the `debug=2` dump and the goroutineleak text.

use std::{fs, path::PathBuf};

use dla_parsers::go::{
    dump::{self, group_by_creator},
    goroutine, heap, pprof,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/go")
        .join(name)
}

fn profile(name: &str) -> pprof::Profile {
    pprof::read(&fs::read(fixture(name)).unwrap()).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn text(name: &str) -> String {
    fs::read_to_string(fixture(name)).unwrap()
}

#[test]
fn heap_profiles_read_and_grow() {
    let sizes: Vec<u64> = (1..=3)
        .map(|i| {
            let p = profile(&format!("heap-{i}.pb.gz"));
            assert!(heap::is_heap_profile(&p));
            assert_eq!(p.period_type.kind, "space");
            assert_eq!(p.period, 64);
            let a = heap::to_alloc_profile(&p).unwrap();
            assert!(a.sampled && a.timestamp_ms.is_some());
            a.groups.iter().map(|g| g.live_bytes).sum()
        })
        .collect();
    assert!(sizes[0] < sizes[1] && sizes[1] < sizes[2], "{sizes:?}");
}

#[test]
fn a_leaking_stack_is_found_with_a_project_relative_site() {
    let a = heap::to_alloc_profile(&profile("heap-3.pb.gz")).unwrap();
    // The scratch marker is stripped, so the frame names the project file.
    let hit = a.groups.iter().any(|g| {
        g.frames.iter().any(|f| {
            f.function.contains("notify.")
                && f.file.as_deref() == Some("internal/notify/subscription.go")
        })
    });
    assert!(hit, "no stack through internal/notify/subscription.go");
    let main_frame = a
        .groups
        .iter()
        .flat_map(|g| &g.frames)
        .find(|f| f.function == "main.AKIAIOSFODNN7EXAMPLE1")
        .expect("the held buffers");
    assert_eq!(main_frame.file.as_deref(), Some("cmd/scratch/main.go"));
    // Standard library frames are not project paths.
    assert!(a.groups.iter().flat_map(|g| &g.frames).any(|f| {
        f.file
            .as_deref()
            .is_some_and(|p| p.starts_with("$GOROOT/src/"))
    }));
}

#[test]
fn the_function_histogram_names_the_allocating_project_function() {
    let h = heap::to_histogram(&profile("heap-3.pb.gz")).unwrap();
    assert!(!h.rows.is_empty());
    assert!(
        h.rows
            .iter()
            .any(|r| r.name == "main.AKIAIOSFODNN7EXAMPLE1" && r.bytes >= 3 * 3 * 2048),
        "{:?}",
        h.rows
    );
}

#[test]
fn goroutine_profiles_group_by_blocked_frame() {
    let p = profile("goroutine-3.pb.gz");
    assert!(goroutine::is_goroutine_profile(&p));
    assert!(!heap::is_heap_profile(&p));
    let s = goroutine::profile_snapshot(&p).unwrap();
    // Thirty subscription workers and nine parked ones, plus the main goroutine.
    assert_eq!(s.metrics[0].value, 40);
    let run = s
        .groups
        .iter()
        .find(|g| g.key == "blocked:example.com/notifyd/internal/notify.(*Subscription).run")
        .expect("the subscription worker group");
    assert_eq!(run.count, 30);
    assert!(s.groups.iter().all(|g| g.site.is_none()));
    assert!(heap::to_alloc_profile(&p).is_err());
}

#[test]
fn the_dump_groups_goroutines_by_creation_site() {
    let d = dump::parse(&text("goroutine-3.txt")).unwrap();
    assert_eq!(d.goroutines.len(), 40);
    let groups = group_by_creator(&d.goroutines, false);
    assert_eq!(groups.len(), 2);
    let g = &groups[0];
    assert_eq!(
        g.function,
        "example.com/notifyd/internal/notify.newSubscription"
    );
    assert_eq!(g.file.as_deref(), Some("internal/notify/subscription.go"));
    assert_eq!(g.line, Some(33));
    assert_eq!(g.count, 30);
    assert_eq!(g.wait, "select");
    let b = g.blocked.as_ref().unwrap();
    assert_eq!(
        (b.file.as_deref(), b.line),
        (Some("internal/notify/subscription.go"), Some(40))
    );
    // The plain profile shows `(leaked)` for the goroutines an earlier leak profile of the
    // same process proved stuck (rounds 1 and 2).
    assert_eq!(g.leaked, 20);
    let s = goroutine::dump_snapshot(&d);
    assert!(
        s.groups
            .iter()
            .any(|g| g.key == "wait:select" && g.count == 30)
    );
}

#[test]
fn the_leak_text_marks_only_the_goroutines_the_runtime_proved_stuck() {
    let d = dump::parse(&text("goroutineleak-3.txt")).unwrap();
    let leaked = group_by_creator(&d.goroutines, true);
    assert_eq!(leaked.len(), 1, "{leaked:?}");
    let g = &leaked[0];
    assert_eq!(
        (g.file.as_deref(), g.line, g.count),
        (Some("internal/notify/subscription.go"), Some(33), 30)
    );
    assert_eq!(g.wait, "select");
    // The goroutines parked on a global channel are alive, not leaked: the blind spot.
    let parked = group_by_creator(&d.goroutines, false)
        .into_iter()
        .find(|g| g.function == "main.AKIAIOSFODNN7EXAMPLE1")
        .unwrap();
    assert_eq!((parked.count, parked.leaked), (9, 0));
}

#[test]
fn text_and_binary_inputs_are_told_apart_without_panics() {
    let bin = fs::read(fixture("heap-1.pb.gz")).unwrap();
    assert!(dump::parse(&String::from_utf8_lossy(&bin)).is_err());
    assert!(pprof::read(b"goroutine 1 [running]:\n").is_err());
    // Any prefix of a real profile is refused or read, never a panic.
    for cut in (0..bin.len()).step_by(97) {
        let _ = pprof::read(&bin[..cut]);
    }
}

#[test]
fn a_profile_is_recognised_from_its_first_bytes() {
    for name in ["heap-3.pb.gz", "goroutine-3.pb.gz", "heap-1.pb.gz"] {
        let bytes = fs::read(fixture(name)).unwrap();
        assert!(pprof::looks_like_profile(&bytes), "{name}");
    }
    assert!(!pprof::looks_like_profile(
        &fs::read(fixture("goroutine-3.txt")).unwrap()
    ));
}
