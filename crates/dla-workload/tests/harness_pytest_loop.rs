//! The pytest loop (`dla harness gen --lang python --pytest`): the project's own tests in one
//! process, in a loop, with no hooks.
//!
//! The generation tests need no pytest (a refusal is one of them). The end-to-end tests run a
//! real `python3` driver on a small project and are skipped, with the reason printed, when
//! pytest does not import in `python3`. They never fail for a missing tool.
#![cfg(unix)]

use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use dla_core::{
    Ledger, Refusal,
    harness::{HarnessAttempt, HarnessManifest},
    runtime::Runtime,
    series::{KeyMode, SeriesFile},
    state::{Consent, ConsentGiver, RunState, now_utc},
    toolctx::{StageOutcome, ToolCtx},
};
use dla_workload::harness::{
    r#gen::{GenOptions, generate},
    run::{RunOptions, run_points},
};
use tempfile::TempDir;

const STORE: &str = "\
KEPT = []


def keep(i):
    KEPT.append(object())  # KEEP


def churn(i):
    tmp = [object() for _ in range(5)]  # CHURN
    return len(tmp)
";

const TEST_KEEPS: &str = "\
from pkg import store


def test_keeps_one_object_per_call():
    store.keep(1)
    store.churn(1)
";

const TEST_QUIET: &str = "\
def test_quiet():
    assert 1 + 1 == 2
";

const TEST_BROKEN: &str = "\
def test_one():
    assert False, \"broken one\"


def test_two():
    assert False, \"broken two\"
";

const TEST_SLOW: &str = "\
import time


def test_slow():
    time.sleep(4)
";

struct Fx {
    _dir: TempDir,
    ctx: ToolCtx,
}

/// A project with the given files, an initialised run with consent for tier 1, and (when
/// `discover`) the Python workload discovery done.
fn project_with(files: &[(&str, &str)], discover: bool) -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    for (rel, text) in files {
        let path = project.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let run_dir = project.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(project.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.consent = Some(Consent {
        tiers: vec![1],
        raw_values: false,
        given_by: ConsentGiver::Runner,
        recorded_at: now_utc(),
    });
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project,
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    };
    if discover {
        dla_workload::discover(&ctx, Some(Runtime::Python)).unwrap();
    }
    Fx { _dir: dir, ctx }
}

fn leak_project(extra: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    let mut files = vec![
        (
            "pyproject.toml",
            "[project]\nname = \"pkg\"\nversion = \"1\"\n",
        ),
        ("pkg/__init__.py", ""),
        ("pkg/store.py", STORE),
        ("tests/test_keeps.py", TEST_KEEPS),
        ("tests/test_quiet.py", TEST_QUIET),
        ("tests/test_broken.py", TEST_BROKEN),
    ];
    files.extend_from_slice(extra);
    files
}

fn opts() -> GenOptions {
    GenOptions {
        lang: Runtime::Python,
        workload: None,
        driver: false,
        iter_arg: None,
        step: None,
        iterations: 8,
        warmup: 1,
        probe_findings: Vec::new(),
        entry: Vec::new(),
        next: None,
        auto_probes: true,
        package: None,
        junit: false,
        pytest: true,
        tests: Vec::new(),
        features: Vec::new(),
        no_default_features: false,
    }
}

fn gen_ok(fx: &Fx, o: &GenOptions) -> StageOutcome {
    generate(&fx.ctx, o).unwrap_or_else(|r| panic!("gen refused: {r:?}"))
}

fn gen_err(fx: &Fx, o: &GenOptions) -> Refusal {
    generate(&fx.ctx, o).expect_err("gen should refuse")
}

/// True when pytest imports in the `python3` of PATH; the reason is printed when it does not.
fn have_pytest() -> bool {
    let ok = Command::new("python3")
        .args([
            "-c",
            "import pytest, sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipped: pytest does not import in python3 (3.8 or newer) on PATH");
    }
    ok
}

fn series_of(fx: &Fx, out: &dla_workload::harness::run::RunOutput) -> SeriesFile {
    assert_eq!(out.series.len(), 1, "{:?}", out.series);
    SeriesFile::load(&fx.ctx.run_dir, &out.series[0]).unwrap()
}

fn run_ok(fx: &Fx, id: &str, o: &RunOptions) -> dla_workload::harness::run::RunOutput {
    let m = fx.ctx.harness(id).unwrap();
    run_points(&fx.ctx, &m, o).unwrap_or_else(|r| panic!("run refused: {r:?}"))
}

/// The 1-based line of `needle` in `text`.
fn line_of(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("{needle} not in the text"))
        + 1
}

/// The values of column `name` at every point.
fn column(s: &SeriesFile, name: &str) -> Vec<Option<f64>> {
    let i = s.column_index(name).unwrap_or_else(|| {
        panic!(
            "no column {name}; have {:?}",
            s.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
        )
    });
    s.values(i)
}

/// The end-to-end tests take this: they are timing-sensitive and each starts real processes.
static PATH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    PATH_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn driver_text(fx: &Fx, id: &str) -> String {
    fs::read_to_string(
        fx.ctx
            .run_dir
            .join("harness")
            .join(id)
            .join("driver/dla_pytest_loop.py"),
    )
    .unwrap()
}

// ---------------------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------------------

#[test]
fn gen_writes_a_hookless_loop_over_the_test_files() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(&leak_project(&[]), true);
    let out = gen_ok(&fx, &opts());
    let m: HarnessManifest = fx.ctx.harness("H1").unwrap();
    assert!(m.is_test_loop() && !m.is_hook_driver() && m.is_pytest_loop());
    assert_eq!(m.key_modes, [KeyMode::Fresh]);
    assert_eq!((m.warmup, m.iterations, m.step), (1, 8, 1));
    assert_eq!(m.timeout_secs, 1800);
    assert_eq!(m.tools, ["pytest-loop"]);
    assert!(m.targets.is_empty());
    assert!(
        m.run.argv[1].ends_with("driver/dla_pytest_loop.py"),
        "{:?}",
        m.run.argv
    );
    let text = driver_text(&fx, "H1");
    assert!(!text.contains("@@DLA_"), "a placeholder is left");
    assert!(!text.contains("DLA-HOOK"), "a hook is left to fill");
    for f in [
        "tests/test_keeps.py",
        "tests/test_quiet.py",
        "tests/test_broken.py",
    ] {
        assert!(text.contains(f), "{f} not selected: {text}");
    }
    assert_eq!(out.summary["next"], "dla harness run H1");
    assert!(
        out.summary["instructions"]
            .as_str()
            .unwrap()
            .starts_with("nothing to fill: the driver runs the project's own tests in a loop"),
        "{}",
        out.summary
    );
}

#[test]
fn gen_discovers_the_workloads_when_the_run_has_none() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(&leak_project(&[]), false);
    assert!(!fx.ctx.run_dir.join("workload").exists());
    gen_ok(&fx, &opts());
    assert!(fx.ctx.run_dir.join("workload/workloads.json").is_file());
    assert!(driver_text(&fx, "H1").contains("tests/test_keeps.py"));
}

#[test]
fn gen_tests_names_files_or_directories_relative_to_the_working_directory() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(&leak_project(&[]), true);
    let o = GenOptions {
        tests: vec!["tests/test_keeps.py".into(), "./tests/".into()],
        ..opts()
    };
    gen_ok(&fx, &o);
    let text = driver_text(&fx, "H1");
    assert!(
        text.contains("TESTS = [\"tests/test_keeps.py\", \"tests\"]"),
        "{text}"
    );
    let o = GenOptions {
        tests: vec!["tests/absent.py".into()],
        ..opts()
    };
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "invalid-value", "{r:?}");
    assert!(
        r.message.contains("no such file or directory"),
        "{}",
        r.message
    );
}

#[test]
fn gen_refuses_pytest_for_another_language() {
    let fx = project_with(&leak_project(&[]), true);
    for lang in [Runtime::Jvm, Runtime::Go, Runtime::Node] {
        let r = gen_err(&fx, &GenOptions { lang, ..opts() });
        assert_eq!(r.code, "invalid-value", "{r:?}");
        assert!(
            r.message.contains("--pytest is for Python projects")
                && r.message.contains(lang.as_str()),
            "{}",
            r.message
        );
    }
    assert!(!fx.ctx.run_dir.join("harness/H1").exists());
}

#[test]
fn gen_refuses_next_junit_and_entry_with_pytest() {
    let fx = project_with(&leak_project(&[]), true);
    for o in [
        GenOptions {
            next: Some(2),
            ..opts()
        },
        GenOptions {
            junit: true,
            ..opts()
        },
        GenOptions {
            entry: vec![dla_core::FindingId::new(1)],
            ..opts()
        },
    ] {
        let r = gen_err(&fx, &o);
        assert_eq!(r.code, "invalid-value", "{r:?}");
        assert!(
            r.message
                .contains("--pytest runs the project's own tests and declares no candidate"),
            "{}",
            r.message
        );
    }
    assert!(!fx.ctx.run_dir.join("harness/H1").exists());
}

#[test]
fn gen_refuses_a_node_id_in_tests() {
    let fx = project_with(&leak_project(&[]), true);
    let o = GenOptions {
        tests: vec!["tests/test_keeps.py::test_keeps_one_object_per_call".into()],
        ..opts()
    };
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "invalid-value", "{r:?}");
    assert!(
        r.message.contains("node ids") && r.message.contains("::"),
        "{}",
        r.message
    );
    assert!(!fx.ctx.run_dir.join("harness/H1").exists());
}

#[test]
fn gen_refuses_a_project_without_test_files_and_names_the_driver_command() {
    let fx = project_with(
        &[
            ("pyproject.toml", "[project]\nname = \"pkg\"\n"),
            ("pkg/__init__.py", ""),
            ("pkg/store.py", STORE),
        ],
        true,
    );
    let r = gen_err(&fx, &opts());
    assert_eq!(r.code, "workload-required", "{r:?}");
    assert!(
        r.message.contains("lists no Python test file"),
        "{}",
        r.message
    );
    assert!(
        r.fix
            .contains("dla harness gen --lang python --driver --next"),
        "{}",
        r.fix
    );
    assert!(!fx.ctx.run_dir.join("harness/H1").exists());
    // No test file is no failed attempt of the loop: the queue never offered it.
    assert!(!HarnessAttempt::pytest_loop_refused(&fx.ctx.run_dir));
}

// ---------------------------------------------------------------------------------------
// The copy keeps built extension modules (Python harnesses only)
// ---------------------------------------------------------------------------------------

#[test]
fn a_python_harness_copy_keeps_built_extension_modules_and_a_native_one_does_not() {
    let _g = lock();
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    for (rel, text) in [
        ("pyproject.toml", "[project]\nname = \"pkg\"\n"),
        ("pkg/__init__.py", ""),
        ("pkg/_native.so", "not a real library"),
        ("pkg/_other.cpython-312-x86_64-linux-gnu.so", "x"),
        ("build/tmp.o", "object"),
        ("pkg/stray.o", "object"),
        ("libs/lonely.so", "no python source in this directory"),
    ] {
        let path = project.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(&project)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    if !git(&["init", "-q"]) {
        eprintln!("skipped: git is not available");
        return;
    }
    assert!(git(&["add", "pyproject.toml", "pkg/__init__.py"]));
    // `_other` is ignored by git (the usual state of an in-place build), `_native` is untracked.
    fs::write(project.join(".gitignore"), "*.cpython-*.so\n").unwrap();
    use dla_workload::harness::scratch::copy_project_for;
    let py = dir.path().join("py");
    let report = copy_project_for(&project, &py, None, Some(Runtime::Python)).unwrap();
    assert!(
        py.join("pkg/_native.so").is_file(),
        "the untracked module is kept"
    );
    assert!(
        py.join("pkg/_other.cpython-312-x86_64-linux-gnu.so")
            .is_file(),
        "the ignored module is kept"
    );
    assert!(!py.join("build/tmp.o").exists(), "an object file is not");
    assert!(!py.join("pkg/stray.o").exists(), "an object file is not");
    assert!(
        !py.join("libs/lonely.so").exists(),
        "a library in a directory with no Python source is not an extension module"
    );
    assert_eq!(report.extensions_kept, 2);
    assert!(
        report
            .gaps
            .iter()
            .any(|g| g.starts_with("scratch copy: 2 built extension module(s) kept")),
        "{:?}",
        report.gaps
    );
    for rt in [Runtime::Native, Runtime::Rust] {
        let other = dir.path().join(format!("other-{}", rt.as_str()));
        let report = copy_project_for(&project, &other, None, Some(rt)).unwrap();
        assert!(!other.join("pkg/_native.so").exists(), "{rt:?}");
        assert!(!other.join("build/tmp.o").exists(), "{rt:?}");
        assert!(!other.join("pkg/stray.o").exists(), "{rt:?}");
        assert_eq!(report.extensions_kept, 0, "{rt:?}");
        assert!(
            report.gaps.iter().all(|g| !g.contains("extension module")),
            "{rt:?}"
        );
    }
    // The clone of a Python harness (`dla intervene`) keeps them too; another runtime's does not.
    use dla_workload::harness::scratch::{copy_tree, copy_tree_for};
    let clone = dir.path().join("clone-py");
    copy_tree_for(&py, &clone, &project, Some(Runtime::Python)).unwrap();
    assert!(clone.join("pkg/_native.so").is_file());
    assert!(
        clone
            .join("pkg/_other.cpython-312-x86_64-linux-gnu.so")
            .is_file()
    );
    assert!(!clone.join("pkg/stray.o").exists());
    assert!(!clone.join("libs/lonely.so").exists());
    let clone = dir.path().join("clone-native");
    copy_tree_for(&py, &clone, &project, Some(Runtime::Native)).unwrap();
    assert!(!clone.join("pkg/_native.so").exists());
    let clone = dir.path().join("clone-plain");
    copy_tree(&py, &clone, &project).unwrap();
    assert!(!clone.join("pkg/_native.so").exists());
    // The old entry point is the unchanged copy of every runtime.
    let plain = dir.path().join("plain");
    dla_workload::harness::scratch::copy_project(&project, &plain).unwrap();
    assert!(!plain.join("pkg/_native.so").exists());
}

// ---------------------------------------------------------------------------------------
// The loop, end to end (a real python3 with pytest)
// ---------------------------------------------------------------------------------------

#[test]
fn the_loop_runs_with_no_edit_and_the_keeping_line_rises_per_run() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(&leak_project(&[]), true);
    gen_ok(&fx, &opts());
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let line = line_of(STORE, "# KEEP");
    let vals: Vec<f64> = column(&s, &format!("py.blocks@pkg/store.py:{line}"))
        .iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect();
    assert_eq!(vals.len(), 9, "{vals:?}");
    assert!(
        vals.windows(2).all(|w| w[1] > w[0]),
        "one block more per run, every point: {vals:?}"
    );
    assert_eq!(vals[8] - vals[0], 8.0, "{vals:?}");
    // The line that allocates and frees has no column.
    let churn = line_of(STORE, "# CHURN");
    assert!(
        s.columns
            .iter()
            .all(|c| c.name != format!("py.blocks@pkg/store.py:{churn}")),
        "a line that frees its blocks has no column"
    );
    // The x axis is the cumulative run count (step 1).
    assert_eq!(s.points.first().map(|p| p.x), Some(1.0));
    assert_eq!(s.points.last().map(|p| p.x), Some(9.0));
    // The file none of whose tests passed is dropped, once, with its first failure.
    let dropped: Vec<&String> = s
        .limitations
        .iter()
        .filter(|l| l.starts_with("dropped test file "))
        .collect();
    assert_eq!(dropped.len(), 1, "{:?}", s.limitations);
    assert!(
        dropped[0].starts_with("dropped test file tests/test_broken.py: none of its tests passed in the first run (first failure: ")
            && dropped[0].contains("broken one"),
        "{}",
        dropped[0]
    );
}

#[test]
fn a_project_where_every_test_fails_is_a_failed_harness_with_the_reason() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(
        &[
            ("pyproject.toml", "[project]\nname = \"pkg\"\n"),
            ("pkg/__init__.py", ""),
            ("tests/test_broken.py", TEST_BROKEN),
        ],
        true,
    );
    gen_ok(&fx, &opts());
    let m = fx.ctx.harness("H1").unwrap();
    let r = run_points(&fx.ctx, &m, &RunOptions::default()).expect_err("every test fails");
    assert_eq!(r.code, "harness-failed", "{r:?}");
    assert!(
        r.message
            .contains("DLA-PYTEST no test passed in any test file")
            && r.message.contains("broken one"),
        "{}",
        r.message
    );
    // The fix names the in-place build and the driver command, not hooks to fill.
    assert!(
        r.fix.contains("pip install -e") && r.fix.contains("build_ext --inplace"),
        "{}",
        r.fix
    );
    assert!(
        r.fix
            .contains("dla harness gen --lang python --driver --next")
            && r.fix.contains("no hooks to fill"),
        "{}",
        r.fix
    );
}

#[test]
fn the_tests_importing_the_project_from_outside_it_stop_the_loop() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(&leak_project(&[]), true);
    // Another copy of `pkg` outside the project, which a conftest puts first on the path (the
    // loop puts the working directory first, so only code that reorders the path can reach it).
    let other = fx._dir.path().join("elsewhere");
    fs::create_dir_all(other.join("pkg")).unwrap();
    fs::write(other.join("pkg/__init__.py"), "").unwrap();
    fs::write(other.join("pkg/store.py"), STORE).unwrap();
    fs::write(
        fx.ctx.project.join("tests/conftest.py"),
        format!(
            "import sys\nsys.path.insert(0, {:?})\n",
            other.display().to_string()
        ),
    )
    .unwrap();
    gen_ok(&fx, &opts());
    let m = fx.ctx.harness("H1").unwrap();
    let r = run_points(&fx.ctx, &m, &RunOptions::default()).expect_err("another copy is measured");
    assert_eq!(r.code, "harness-failed", "{r:?}");
    let want = format!(
        "DLA-PYTEST the tests import pkg from {}",
        other.join("pkg/__init__.py").display()
    );
    assert!(r.message.contains(&want), "{}", r.message);
    assert!(
        r.message
            .contains("which is outside the project: the loop would measure another copy")
            && r.message.contains("python3 -m pip install -e")
            && r.message.contains("then generate the harness again"),
        "{}",
        r.message
    );
}

#[test]
fn a_slow_file_is_dropped_to_fit_the_time_limit() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(
        &leak_project(&[("tests/test_slow.py", TEST_SLOW)])
            .into_iter()
            .filter(|(p, _)| *p != "tests/test_broken.py")
            .collect::<Vec<_>>(),
        true,
    );
    gen_ok(&fx, &opts());
    // A 30 s limit leaves 20.5 s: nine runs of the tests (4 s of sleep each) do not fit, the
    // quick files do.
    let mut m = fx.ctx.harness("H1").unwrap();
    m.timeout_secs = 30;
    m.save(&fx.ctx.run_dir).unwrap();
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let dropped: Vec<&String> = s
        .limitations
        .iter()
        .filter(|l| l.starts_with("dropped test file "))
        .collect();
    assert_eq!(dropped.len(), 1, "{:?}", s.limitations);
    assert!(
        dropped[0].starts_with("dropped test file tests/test_slow.py: it took ")
            && dropped[0].contains(" s of ")
            && dropped[0].contains(" s per run and the series would not fit 30 s"),
        "{}",
        dropped[0]
    );
    let line = line_of(STORE, "# KEEP");
    column(&s, &format!("py.blocks@pkg/store.py:{line}"));
    assert_eq!(
        s.points.last().map(|p| p.x),
        Some(9.0),
        "the whole series was written"
    );
}

#[test]
fn one_file_that_alone_does_not_fit_gives_no_usable_point() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(
        &[
            ("pyproject.toml", "[project]\nname = \"pkg\"\n"),
            ("tests/test_slow.py", TEST_SLOW),
        ],
        true,
    );
    gen_ok(&fx, &opts());
    let mut m = fx.ctx.harness("H1").unwrap();
    m.timeout_secs = 12;
    m.save(&fx.ctx.run_dir).unwrap();
    let r = run_points(&fx.ctx, &m, &RunOptions::default()).expect_err("too slow for a series");
    assert!(
        r.message
            .contains("DLA-PYTEST the tests are too slow for a series"),
        "{}",
        r.message
    );
}

#[test]
fn the_tests_of_a_later_run_that_fail_are_counted_once_and_the_series_is_written() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    // The second run of the tests has a failing test: a marker file flips it.
    let flaky = "\
import os


def test_flaky():
    marker = os.path.join(os.path.dirname(__file__), \"seen\")
    if os.path.exists(marker):
        assert False, \"second run fails\"
    open(marker, \"w\").close()
";
    let fx = project_with(
        &leak_project(&[("tests/test_flaky.py", flaky)])
            .into_iter()
            .filter(|(p, _)| *p != "tests/test_broken.py")
            .collect::<Vec<_>>(),
        true,
    );
    gen_ok(&fx, &opts());
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let later: Vec<&String> = s
        .limitations
        .iter()
        .filter(|l| l.contains("of the tests had") && l.contains("failure(s)"))
        .collect();
    assert_eq!(later.len(), 1, "once: {:?}", s.limitations);
    assert!(later[0].contains("second run fails"), "{}", later[0]);
    // A file that passed in the first run is not dropped.
    assert!(
        s.limitations
            .iter()
            .all(|l| !l.starts_with("dropped test file"))
    );
    let log = fs::read_dir(fx.ctx.run_dir.join("harness/H1/logs"))
        .unwrap()
        .flatten()
        .map(|e| fs::read_to_string(e.path()).unwrap_or_default())
        .collect::<String>();
    assert!(log.contains("DLA-PYTEST summary: units=9"), "{log}");
    assert!(log.contains("failed_units=8"), "{log}");
}

#[test]
fn a_project_whose_test_files_hold_no_test_fails_with_the_reason() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(
        &[
            ("pyproject.toml", "[project]\nname = \"pkg\"\n"),
            ("tests/test_empty.py", "VALUE = 1\n"),
        ],
        true,
    );
    gen_ok(&fx, &opts());
    let m = fx.ctx.harness("H1").unwrap();
    let r = run_points(&fx.ctx, &m, &RunOptions::default()).expect_err("no test");
    assert_eq!(r.code, "harness-failed", "{r:?}");
    assert!(
        r.message
            .contains("DLA-PYTEST no test found in tests/test_empty.py"),
        "{}",
        r.message
    );
}

#[test]
fn the_step_is_never_below_one_and_a_step_too_long_for_the_limit_is_lowered() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let one_second = "import time\n\n\ndef test_one_second():\n    time.sleep(1)\n";
    let fx = project_with(
        &[
            ("pyproject.toml", "[project]\nname = \"pkg\"\n"),
            ("pkg/__init__.py", ""),
            ("pkg/store.py", STORE),
            ("tests/test_keeps.py", TEST_KEEPS),
            ("tests/test_second.py", one_second),
        ],
        true,
    );
    // A step of 0 in the manifest: the loop runs one unit per round, the x axis counts runs.
    gen_ok(&fx, &opts());
    let mut m = fx.ctx.harness("H1").unwrap();
    m.step = 0;
    m.save(&fx.ctx.run_dir).unwrap();
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let xs: Vec<f64> = s.points.iter().map(|p| p.x).collect();
    assert_eq!(xs, (1..=9).map(f64::from).collect::<Vec<_>>(), "{xs:?}");

    // Three runs per round do not fit a 36 s limit at about a second a run: the step is lowered
    // (and said), and every round is still written.
    let fx = project_with(
        &[
            ("pyproject.toml", "[project]\nname = \"pkg\"\n"),
            ("pkg/__init__.py", ""),
            ("pkg/store.py", STORE),
            ("tests/test_keeps.py", TEST_KEEPS),
            ("tests/test_second.py", one_second),
        ],
        true,
    );
    gen_ok(
        &fx,
        &GenOptions {
            step: Some(3),
            ..opts()
        },
    );
    let mut m = fx.ctx.harness("H1").unwrap();
    assert_eq!(m.step, 3);
    m.timeout_secs = 36;
    m.save(&fx.ctx.run_dir).unwrap();
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let lowered: Vec<&String> = s
        .limitations
        .iter()
        .filter(|l| l.starts_with("the test loop lowered its step from 3 to "))
        .collect();
    assert_eq!(lowered.len(), 1, "{:?}", s.limitations);
    assert_eq!(s.points.len(), 9, "every round is written");
    assert!(
        s.points.last().map(|p| p.x).unwrap() < 27.0,
        "fewer runs than 9 rounds of 3"
    );
}

#[allow(dead_code)]
fn _unused(_: &Path) {}
