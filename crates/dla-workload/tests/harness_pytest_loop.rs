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
fn the_loop_clears_the_addopts_of_the_project() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    // The project's own configuration asks for an option no plugin knows: pytest refuses to start
    // with it, so the loop only runs when it clears addopts for its own pytest calls.
    let fx = project_with(
        &leak_project(&[(
            "pytest.ini",
            "[pytest]\naddopts = --no-such-option-of-any-plugin\n",
        )]),
        true,
    );
    gen_ok(&fx, &opts());
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let line = line_of(STORE, "# KEEP");
    let vals = column(&s, &format!("py.blocks@pkg/store.py:{line}"));
    assert_eq!(vals.len(), 9, "{vals:?}");
    assert!(vals.iter().all(Option::is_some), "{vals:?}");
    assert_eq!(s.points.len(), 9);
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

// ---------------------------------------------------------------------------------------
// A run that pytest ends early is not a unit
// ---------------------------------------------------------------------------------------

/// A `conftest.py` whose collection hook counts the runs in a file next to it and raises from the
/// run `from_run` on (only that run when `only` is set): pytest ends such a run with an internal
/// error and no test executed.
fn cutting_conftest(from_run: u32, only: bool) -> String {
    let test = if only { "n == FROM" } else { "n >= FROM" };
    format!(
        "\
import os

FROM = {from_run}
COUNT = os.path.join(os.path.dirname(os.path.abspath(__file__)), \"runs.txt\")


def pytest_collection(session):
    try:
        n = int(open(COUNT).read()) + 1
    except OSError:
        n = 1
    with open(COUNT, \"w\") as fh:
        fh.write(str(n))
    if {test}:
        raise RuntimeError(\"collection cut in run %d\" % n)
"
    )
}

fn loop_files(extra: &[(&'static str, String)]) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = vec![
        (
            "pyproject.toml".into(),
            "[project]\nname = \"pkg\"\nversion = \"1\"\n".into(),
        ),
        ("pkg/__init__.py".into(), String::new()),
        ("pkg/store.py".into(), STORE.into()),
        ("tests/test_keeps.py".into(), TEST_KEEPS.into()),
        ("tests/test_quiet.py".into(), TEST_QUIET.into()),
    ];
    files.extend(extra.iter().map(|(p, t)| ((*p).to_owned(), t.clone())));
    files
}

fn project_owned(files: &[(String, String)]) -> Fx {
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    project_with(&refs, true)
}

fn harness_log(fx: &Fx) -> String {
    fs::read_dir(fx.ctx.run_dir.join("harness/H1/logs"))
        .unwrap()
        .flatten()
        .map(|e| fs::read_to_string(e.path()).unwrap_or_default())
        .collect::<String>()
}

fn keep_column(s: &SeriesFile) -> Vec<f64> {
    let line = line_of(STORE, "# KEEP");
    column(s, &format!("py.blocks@pkg/store.py:{line}"))
        .iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect()
}

#[test]
fn a_run_that_pytest_cut_is_made_again_and_the_series_has_no_flat_step() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_owned(&loop_files(&[("conftest.py", cutting_conftest(2, true))]));
    gen_ok(&fx, &opts());
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let vals = keep_column(&s);
    assert_eq!(vals.len(), 9, "{vals:?}");
    assert!(
        vals.windows(2).all(|w| w[1] - w[0] == 1.0),
        "one block more at every step, the cut run is not a step of its own: {vals:?}"
    );
    let cut: Vec<&String> = s
        .limitations
        .iter()
        .filter(|l| l.contains("was ended early by pytest"))
        .collect();
    assert_eq!(
        cut.len(),
        1,
        "one note for the one cut run: {:?}",
        s.limitations
    );
    assert!(
        cut[0].starts_with("run 2 of the tests was ended early by pytest (exit code 3")
            && cut[0].contains("collection cut in run 2")
            && cut[0].contains("is made again"),
        "{}",
        cut[0]
    );
    let log = harness_log(&fx);
    assert!(
        log.contains("DLA-PYTEST summary: units=9") && log.contains("cut_units=0 repeats=1"),
        "{log}"
    );
    assert!(
        s.limitations.iter().all(|l| !l.contains("counted as dead")),
        "no unit was lost: {:?}",
        s.limitations
    );
}

#[test]
fn a_run_that_pytest_cuts_every_time_is_repeated_within_the_bounds_and_the_cut_units_are_counted() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_owned(&loop_files(&[("conftest.py", cutting_conftest(2, false))]));
    gen_ok(&fx, &opts());
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let log = harness_log(&fx);
    // Run 2 is made again twice, run 3 twice more (4 in the loop); every later run is cut once
    // and counts as a dead unit at once. Eight of the nine runs are cut.
    assert!(
        log.contains("DLA-PYTEST summary: units=9") && log.contains("cut_units=8 repeats=4"),
        "{log}"
    );
    let notes: Vec<&String> = s
        .limitations
        .iter()
        .filter(|l| l.contains("unit(s) of the loop were cut by pytest"))
        .collect();
    assert_eq!(notes.len(), 1, "one summary note: {:?}", s.limitations);
    assert!(
        notes[0].starts_with("8 unit(s) of the loop were cut by pytest and counted as dead (run 2, 3, 4, 5, 6, 7, 8, 9) after 4 repeat(s)"),
        "{}",
        notes[0]
    );
    let again = s
        .limitations
        .iter()
        .filter(|l| l.contains("is not a unit and is made again"))
        .count();
    assert_eq!(again, 4, "{:?}", s.limitations);
}

#[test]
fn a_test_file_that_fails_to_import_costs_its_own_tests_and_is_named_once() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let bad = "import a_module_that_does_not_exist_anywhere\n\n\ndef test_never():\n    pass\n";
    let fx = project_owned(&loop_files(&[("tests/test_import.py", bad.into())]));
    gen_ok(&fx, &opts());
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let vals = keep_column(&s);
    assert_eq!(vals.len(), 9, "{vals:?}");
    assert!(vals.windows(2).all(|w| w[1] - w[0] == 1.0), "{vals:?}");
    let named: Vec<&String> = s
        .limitations
        .iter()
        .filter(|l| l.contains("tests/test_import.py"))
        .collect();
    assert_eq!(named.len(), 1, "named once: {:?}", s.limitations);
    assert!(
        named[0].contains("failed to collect")
            && named[0].contains("a_module_that_does_not_exist_anywhere"),
        "{}",
        named[0]
    );
    let log = harness_log(&fx);
    assert!(log.contains("cut_units=0 repeats=0"), "{log}");
}

/// Run 2 only: after the tests of `test_a.py` have been reported, the session ends with an
/// internal error (an exception raised from `pytest_runtest_logfinish`, exit code 3), so the
/// run is cut after some tests ran and before the others.
const MID_CUT_CONFTEST: &str = "\
import os

COUNT = os.path.join(os.path.dirname(os.path.abspath(__file__)), \"runs.txt\")
RUN = [0]


def pytest_sessionstart(session):
    try:
        n = int(open(COUNT).read()) + 1
    except OSError:
        n = 1
    with open(COUNT, \"w\") as fh:
        fh.write(str(n))
    RUN[0] = n


def pytest_runtest_logfinish(nodeid, location):
    if RUN[0] == 2 and \"test_a\" in nodeid:
        raise RuntimeError(\"session ended after test_a in run 2\")
";

#[test]
fn a_run_cut_after_some_tests_ran_is_not_a_unit_and_is_made_again() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let keep = |name: &str| {
        format!("KEPT = []\n\n\ndef keep_{name}():\n    KEPT.append(object())  # KEEP\n")
    };
    let test = |name: &str| {
        format!("from pkg import {name}\n\n\ndef test_{name}():\n    {name}.keep_{name}()\n")
    };
    let files = vec![
        (
            "pyproject.toml".to_owned(),
            "[project]\nname = \"pkg\"\nversion = \"1\"\n".to_owned(),
        ),
        ("pkg/__init__.py".to_owned(), String::new()),
        ("pkg/a.py".to_owned(), keep("a")),
        ("pkg/b.py".to_owned(), keep("b")),
        ("tests/test_a.py".to_owned(), test("a")),
        ("tests/test_b.py".to_owned(), test("b")),
        ("conftest.py".to_owned(), MID_CUT_CONFTEST.to_owned()),
    ];
    let fx = project_owned(&files);
    gen_ok(&fx, &opts());
    let out = run_ok(&fx, "H1", &RunOptions::default());
    let s = series_of(&fx, &out);
    let line = line_of(&keep("a"), "# KEEP");
    let col = |m: &str| -> Vec<f64> {
        column(&s, &format!("py.blocks@pkg/{m}.py:{line}"))
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect()
    };
    let (a, b) = (col("a"), col("b"));
    eprintln!("line a: {a:?}\nline b: {b:?}");
    assert_eq!(a.len(), 9, "{a:?}");
    // The second file's line rises by one at every step: the cut run never reached it.
    assert!(b.windows(2).all(|w| w[1] - w[0] == 1.0), "{b:?}");
    // The first file's tests ran twice in round 2 (the cut run and the repeat): a double step
    // there, one at every other step, and the x axis counts one unit per round.
    let steps: Vec<f64> = a.windows(2).map(|w| w[1] - w[0]).collect();
    assert_eq!(steps.iter().filter(|d| **d == 2.0).count(), 1, "{a:?}");
    assert!(steps.iter().all(|d| *d == 1.0 || *d == 2.0), "{a:?}");
    assert_eq!(s.points.last().map(|p| p.x), Some(9.0));
    let cut: Vec<&String> = s
        .limitations
        .iter()
        .filter(|l| l.contains("was ended early by pytest"))
        .collect();
    assert_eq!(cut.len(), 1, "{:?}", s.limitations);
    assert!(
        cut[0].starts_with("run 2 of the tests was ended early by pytest (exit code 3, ")
            && cut[0].contains("session ended after test_a in run 2")
            && cut[0].contains("the tests that ran before the cut are in the round's figures"),
        "{}",
        cut[0]
    );
    let log = harness_log(&fx);
    assert!(
        log.contains("DLA-PYTEST summary: units=9") && log.contains("cut_units=0 repeats=1"),
        "{log}"
    );
}

// ---------------------------------------------------------------------------------------
// A command that was cut leaves its process behind: the next run of the harness stops it
// ---------------------------------------------------------------------------------------

fn leftover(fx: &Fx, command: &str, wrong_start: bool) -> std::process::Child {
    use std::os::unix::process::CommandExt;

    use dla_core::state::{ProcessKind, ProcessRecord, ProcessRole};
    let mut c = Command::new("sleep");
    c.arg("120").process_group(0);
    let child = c.spawn().unwrap();
    let start = if wrong_start {
        Some("2001-01-01T00:00:00Z".to_owned())
    } else {
        dla_sample::os::process_start_time(child.id())
    };
    fx.ctx
        .register_process(ProcessRecord {
            pid: child.id(),
            start_time: start,
            kind: ProcessKind::Harness,
            command: command.to_owned(),
            started_at: now_utc(),
            stopped_at: None,
            role: ProcessRole::Started,
            worker: None,
            cleanup: Vec::new(),
        })
        .unwrap();
    child
}

#[test]
fn the_process_an_earlier_command_of_the_harness_left_is_stopped_and_nothing_else() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(&leak_project(&[]), true);
    gen_ok(&fx, &opts());
    let m = fx.ctx.harness("H1").unwrap();
    let command = m.run.argv.join(" ");
    let mut mine = leftover(&fx, &command, false);
    // Another harness's process, a process whose recorded start time is not the live one (a
    // reused pid) and a process of the same command that is not ours to stop are left alone.
    let mut other = leftover(&fx, "python3 /elsewhere/dla_pytest_loop.py", false);
    let mut reused = leftover(&fx, &command, true);
    let notes = dla_workload::harness::limit::stop_leftovers(&fx.ctx, &m);
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(
        notes[0].contains(&format!("stopped process {}", mine.id())),
        "{notes:?}"
    );
    assert!(mine.wait().is_ok_and(|s| !s.success()));
    assert!(
        other.try_wait().unwrap().is_none(),
        "another harness's process lives"
    );
    assert!(
        reused.try_wait().unwrap().is_none(),
        "a reused pid is not signalled"
    );
    let st = RunState::load(&fx.ctx.run_dir).unwrap();
    assert!(
        st.processes
            .iter()
            .any(|p| p.pid == mine.id() && p.stopped_at.is_some())
    );
    assert!(
        st.processes
            .iter()
            .filter(|p| p.pid == other.id() || p.pid == reused.id())
            .all(|p| p.stopped_at.is_none())
    );
    // Nothing is left to stop the second time.
    assert!(dla_workload::harness::limit::stop_leftovers(&fx.ctx, &m).is_empty());
    // A `dla harness run` that is alive is a run in progress: what it started is its own.
    let mut running = leftover(&fx, &command, false);
    let mut other_run = leftover(&fx, "dla harness run", false);
    // (the record of the other command is of the kind `command`, as `dla` registers it)
    dla_core::RunState::update(&fx.ctx.run_dir, |s| {
        let r = s
            .processes
            .iter_mut()
            .rev()
            .find(|p| p.pid == other_run.id())
            .unwrap();
        r.stopped_at = None;
        r.kind = dla_core::state::ProcessKind::Command;
        Ok(())
    })
    .unwrap();
    assert!(
        dla_workload::harness::limit::stop_leftovers(&fx.ctx, &m).is_empty(),
        "another command of the harness is running"
    );
    assert!(running.try_wait().unwrap().is_none());
    let _ = running.kill();
    let _ = other_run.kill();
    let _ = running.wait();
    let _ = other_run.wait();
    let _ = other.kill();
    let _ = reused.kill();
    let _ = other.wait();
    let _ = reused.wait();
}

#[test]
fn a_run_of_the_harness_stops_the_leftover_and_says_so() {
    let _g = lock();
    if !have_pytest() {
        return;
    }
    let fx = project_with(&leak_project(&[]), true);
    gen_ok(&fx, &opts());
    let m = fx.ctx.harness("H1").unwrap();
    let mut mine = leftover(&fx, &m.run.argv.join(" "), false);
    let out = run_ok(&fx, "H1", &RunOptions::default());
    assert_eq!(out.left_stopped.len(), 1, "{:?}", out.left_stopped);
    assert!(
        mine.wait().is_ok_and(|s| !s.success()),
        "the leftover is dead"
    );
    assert_eq!(out.points, 9, "the run itself went on to its end");
}

#[allow(dead_code)]
fn _unused(_: &Path) {}
