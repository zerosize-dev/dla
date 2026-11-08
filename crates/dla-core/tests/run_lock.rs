//! The run lock: no lost update across threads and processes; race-free id allocation.

use std::{collections::BTreeSet, fs, path::Path, process::Command, sync::Arc, thread};

use dla_core::{
    CoverageManifest, Ledger, RunState, fsutil::is_written, state::now_utc, toolctx::ToolCtx,
};

const UPDATES: u64 = 50;
const WORKERS: u64 = 8;

fn new_run() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let run_dir = dir.path().join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(dir.path().to_str().unwrap(), Some("s"), 0, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    (dir, run_dir)
}

fn ctx(run_dir: &Path) -> ToolCtx {
    ToolCtx {
        project: run_dir.parent().unwrap().parent().unwrap().to_path_buf(),
        run_dir: run_dir.to_path_buf(),
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    }
}

fn bump(run_dir: &Path, n: u64) {
    for _ in 0..n {
        RunState::update(run_dir, |s| {
            s.budget.seconds += 1;
            Ok(())
        })
        .unwrap();
        CoverageManifest::update(run_dir, |m| {
            let w = m.worker_mut("go");
            w.iterations = Some(w.iterations.unwrap_or(0) + 1);
        })
        .unwrap();
    }
}

/// Entry point of the child processes spawned by `threads_and_processes_lose_no_update`.
#[test]
fn child_worker() {
    let Ok(dir) = std::env::var("DLA_STRESS_RUN_DIR") else {
        return;
    };
    bump(Path::new(&dir), UPDATES);
}

#[test]
fn threads_and_processes_lose_no_update() {
    let (_d, run_dir) = new_run();
    let exe = std::env::current_exe().unwrap();
    let mut children = Vec::new();
    for _ in 0..WORKERS {
        children.push(
            Command::new(&exe)
                .args(["--exact", "child_worker", "--test-threads=1"])
                .env("DLA_STRESS_RUN_DIR", &run_dir)
                .spawn()
                .unwrap(),
        );
    }
    let run_dir = Arc::new(run_dir);
    let threads: Vec<_> = (0..WORKERS)
        .map(|_| {
            let r = Arc::clone(&run_dir);
            thread::spawn(move || bump(&r, UPDATES))
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    for mut c in children {
        assert!(c.wait().unwrap().success());
    }
    let total = 2 * WORKERS * UPDATES;
    assert_eq!(RunState::load(&run_dir).unwrap().budget.seconds, total);
    let cov = CoverageManifest::load(&run_dir).unwrap().unwrap();
    assert_eq!(cov.workers["go"].iterations, Some(total));
    assert!(run_dir.join("run.lock").is_file());
}

#[test]
fn a_failing_update_saves_nothing() {
    let (_d, run_dir) = new_run();
    let r: Result<(), _> = RunState::update(&run_dir, |s| {
        s.budget.seconds = 99;
        Err(dla_core::Error::io(".", std::io::Error::other("no")))
    });
    assert!(r.is_err());
    assert_eq!(RunState::load(&run_dir).unwrap().budget.seconds, 0);
}

#[test]
fn concurrent_id_allocation_never_collides() {
    let (_d, run_dir) = new_run();
    let tc = Arc::new(ctx(&run_dir));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let tc = Arc::clone(&tc);
            thread::spawn(move || {
                let mut ids = Vec::new();
                for _ in 0..25 {
                    ids.push(tc.next_series_id());
                    ids.push(tc.next_trend_id());
                    ids.push(tc.next_harness_id());
                    ids.push(tc.next_alloc_id());
                    ids.push(tc.next_attach_id());
                }
                ids
            })
        })
        .collect();
    let all: Vec<String> = threads
        .into_iter()
        .flat_map(|t| t.join().unwrap())
        .collect();
    let unique: BTreeSet<_> = all.iter().collect();
    assert_eq!(all.len(), 8 * 25 * 5);
    assert_eq!(unique.len(), all.len(), "an id was handed out twice");
    // Reserved: the placeholders exist now; files are empty, directories are dirs.
    assert!(run_dir.join("series/S1.json").is_file());
    assert!(!is_written(&run_dir.join("series/S1.json")));
    assert!(run_dir.join("harness/H1").is_dir());
    assert!(run_dir.join("attach/P200").is_dir());
    assert!(run_dir.join("alloc/A200.json").is_file());
    // The next id continues after the reserved ones, not after the written ones.
    assert_eq!(tc.next_series_id(), "S201");
}

#[test]
fn bundle_dirs_are_reserved_per_finding() {
    let (_d, run_dir) = new_run();
    let tc = ctx(&run_dir);
    let a = tc.next_bundle_dir("jvm_F3").unwrap();
    let b = tc.next_bundle_dir("jvm_F3").unwrap();
    let c = tc.next_bundle_dir("F1").unwrap();
    assert_eq!(a, run_dir.join("bundle/jvm_F3"));
    assert_eq!(b, run_dir.join("bundle/jvm_F3.2"));
    assert_eq!(c, run_dir.join("bundle/F1"));
}
