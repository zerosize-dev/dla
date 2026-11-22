//! The Python driver agent's allocation-site columns (`py.blocks@<file:line>`): the live traced
//! blocks made at one project line, one value at every point of the series.
//!
//! These tests run a real `python3` driver (the template of `harness/python/dla_harness.py` with
//! its two hooks filled) on a small project: one function that keeps an object per call in a
//! module list, one that allocates and frees, a test file whose own code keeps objects and
//! functions that allocate through a library function. A test prints why it is skipped when
//! `python3` is missing.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use dla_core::{
    Ledger,
    harness::{AgentKind, CommandSpec, HarnessManifest, HarnessMode},
    runtime::Runtime,
    series::{KeyMode, SeriesFile},
    state::{Consent, ConsentGiver, RunState, now_utc},
    toolctx::ToolCtx,
};
use dla_workload::harness::run::{RunOptions, run_points};
use tempfile::TempDir;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn have_python() -> bool {
    let ok = Command::new("python3")
        .args([
            "-c",
            "import sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipped: python3 (3.8 or newer) is not available");
    }
    ok
}

struct Fx {
    _dir: TempDir,
    ctx: ToolCtx,
    hdir: PathBuf,
    work: PathBuf,
    agent: PathBuf,
}

/// A run directory with consent for tier 1 and harness `H1`: an empty scratch project and the
/// agent files of `harness/python`.
fn fixture() -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
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
        run_dir: run_dir.clone(),
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    };
    let hdir = run_dir.join("harness/H1");
    let work = hdir.join("work");
    let agent = hdir.join("agent");
    fs::create_dir_all(&agent).unwrap();
    fs::create_dir_all(&work).unwrap();
    for e in fs::read_dir(repo().join("harness/python")).unwrap() {
        let e = e.unwrap();
        if e.path().is_file() {
            fs::copy(e.path(), agent.join(e.file_name())).unwrap();
        }
    }
    Fx {
        _dir: dir,
        ctx,
        hdir,
        work,
        agent,
    }
}

fn write(fx: &Fx, rel: &str, text: &str) {
    let p = fx.work.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// Runs the driver with `setup` and `unit` as its hooks (`step` units per round, `warmup` and
/// `rounds` as given) in key mode fresh, with `env` added to the process environment. Returns
/// the series.
fn run_driver(
    fx: &Fx,
    setup: &str,
    unit: &str,
    step: u64,
    warmup: u32,
    rounds: u32,
    env: &[(&str, &str)],
) -> SeriesFile {
    run_driver_named(fx, "dla_harness.py", setup, unit, step, warmup, rounds, env)
}

/// [`run_driver`] with the driver file under another name (`dla_pytest_loop.py`, the loop over
/// the project's own tests, is a harness file too).
#[allow(clippy::too_many_arguments)]
fn run_driver_named(
    fx: &Fx,
    name: &str,
    setup: &str,
    unit: &str,
    step: u64,
    warmup: u32,
    rounds: u32,
    env: &[(&str, &str)],
) -> SeriesFile {
    let tmpl = fs::read_to_string(fx.agent.join("dla_harness.py")).unwrap();
    let driver = tmpl
        .replace("@@DLA_STEP@@", &step.to_string())
        .replace("@@DLA_WARMUP@@", &warmup.to_string())
        .replace("@@DLA_ROUNDS@@", &rounds.to_string())
        .replace("@@DLA_REPEAT@@", "50")
        .replace(
            "raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            setup,
        )
        .replace(
            "raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            unit,
        );
    fs::create_dir_all(fx.hdir.join("driver")).unwrap();
    fs::write(fx.hdir.join("driver").join(name), &driver).unwrap();
    // The same driver inside the project: a harness file that lies in the project is not counted.
    fs::write(fx.work.join(name), &driver).unwrap();
    let mut env_vars = vec![(
        "PYTHONPATH".to_owned(),
        format!("{}:{}", fx.agent.display(), fx.work.display()),
    )];
    env_vars.extend(env.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())));
    let m = HarnessManifest {
        sibling_of: Default::default(),
        purpose: None,
        targets: Vec::new(),
        schema_version: 1,
        id: "H1".into(),
        runtime: Runtime::Python,
        mode: HarnessMode::Driver,
        workload: Some("test".into()),
        scratch: "harness/H1/work".into(),
        build: None,
        run: CommandSpec {
            argv: vec![
                "python3".into(),
                fx.work.join(name).to_string_lossy().into_owned(),
            ],
            cwd: ".".into(),
            env: env_vars,
        },
        iter_arg: Some("{n}".into()),
        warmup,
        iterations: rounds,
        step,
        key_modes: vec![KeyMode::Fresh],
        probes: vec![],
        agent: AgentKind::DriverPython,
        detectors: vec![],
        tools: vec![],
        gaps: vec![],
        timeout_secs: 120,
        ffi: None,
        captures: vec![],
    };
    m.save(&fx.ctx.run_dir).unwrap();
    fs::write(fx.hdir.join("probes.json"), r#"{"probes":[]}"#).unwrap();
    let out = run_points(&fx.ctx, &m, &RunOptions::default()).unwrap();
    assert_eq!(out.series.len(), 1, "{:?}", out.series);
    SeriesFile::load(&fx.ctx.run_dir, &out.series[0]).unwrap()
}

fn block_columns(s: &SeriesFile) -> Vec<String> {
    s.columns
        .iter()
        .filter(|c| c.name.starts_with("py.blocks@"))
        .map(|c| c.name.clone())
        .collect()
}

/// The 1-based line of `needle` in `text`.
fn line_of(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("{needle} not in the text"))
        + 1
}

const STORE: &str = "\
KEPT = []


def keep(i):
    KEPT.append(object())  # KEEP


def churn(i):
    tmp = [object() for _ in range(5)]  # CHURN
    return len(tmp)
";

const HOLD: &str = "\
HELD = []


def hold(i):
    HELD.append(object())
";

/// A function that keeps objects only in the first measured round (calls 51 to 100 of a 50-unit
/// round with one warm-up round): it grows between the last warm-up point and the first measured
/// one and never after, so it is flat from the first point after the warm-up.
const EARLY: &str = "\
KEPT = []


def keep_early(i):
    if 50 < i <= 100:
        KEPT.append(object())  # EARLY
";

fn project(fx: &Fx) {
    write(fx, "store.py", STORE);
    write(fx, "early.py", EARLY);
    write(fx, "tests/__init__.py", "");
    write(fx, "tests/test_hold.py", HOLD);
}

const SETUP: &str = "import store\n    from tests import test_hold\n    import early\n    _state['store'] = store\n    _state['hold'] = test_hold\n    _state['early'] = early\n    _state['driver_keeps'] = []";
// The last line keeps an object in the driver's own hook, which is a harness file.
const UNIT: &str = "_state['store'].keep(i)\n    _state['store'].churn(i)\n    _state['hold'].hold(i)\n    _state['early'].keep_early(i)\n    _state['driver_keeps'].append(object())";

#[test]
fn a_site_that_keeps_one_object_per_call_has_a_column_with_a_value_at_every_point() {
    if !have_python() {
        return;
    }
    let fx = fixture();
    project(&fx);
    let s = run_driver(&fx, SETUP, UNIT, 50, 1, 6, &[]);
    let line = line_of(STORE, "# KEEP");
    let name = format!("py.blocks@store.py:{line}");
    assert_eq!(
        block_columns(&s),
        std::slice::from_ref(&name),
        "only the keeping line grew"
    );
    let i = s.column_index(&name).unwrap();
    // A value at every point, the warm-up and the first one included.
    let values = s.values(i);
    assert_eq!(values.len(), 7);
    assert!(values.iter().all(Option::is_some), "{values:?}");
    // One block more per call: 50 per point.
    for w in values.windows(2) {
        assert_eq!(w[1].unwrap() - w[0].unwrap(), 50.0, "{values:?}");
    }
    let col = &s.columns[i];
    assert_eq!(
        col.site.as_deref(),
        Some(format!("store.py:{line}").as_str())
    );
    assert_eq!(col.unit, dla_core::series::Unit::Count);
    assert_eq!(col.region, dla_core::series::SeriesRegion::Heap);
    // The line that frees has no column, and neither has the test file's own keeping line.
    assert!(!block_columns(&s).iter().any(|c| c.contains("CHURN")));
    assert!(!block_columns(&s).iter().any(|c| c.contains("test_hold")));
    // The driver's own hook keeps an object per call too, and the project holds a copy of the
    // driver: a harness file has no column.
    assert!(!block_columns(&s).iter().any(|c| c.contains("dla_harness")));
    // A site that grew only while the warm-up ended has no column: the count is taken from the
    // first checkpoint after the warm-up.
    assert!(!block_columns(&s).iter().any(|c| c.contains("early.py")));
}

/// `n` lines in one function, line `k` keeps a list of `k + 1` objects per call, so the growth of
/// the lines differs and the largest are known.
fn many_lines(n: usize) -> String {
    let mut text = String::from("KEPT = []\n\n\ndef keep_many(i):\n");
    for k in 0..n {
        text.push_str(&format!(
            "    KEPT.append([object() for _ in range({})])  # LINE{k}\n",
            k + 1
        ));
    }
    text
}

#[test]
fn at_most_twelve_sites_get_a_column_the_largest_growth_first_and_the_rest_are_counted() {
    if !have_python() {
        return;
    }
    let fx = fixture();
    let text = many_lines(15);
    write(&fx, "many.py", &text);
    let s = run_driver(
        &fx,
        "import many\n    _state['many'] = many",
        "_state['many'].keep_many(i)",
        20,
        1,
        4,
        &[],
    );
    let cols = block_columns(&s);
    assert_eq!(cols.len(), 12, "{cols:?}");
    // The three smallest (1, 2 and 3 objects per call) have none, the twelve largest do.
    for k in 0..15 {
        let name = format!(
            "py.blocks@many.py:{}",
            line_of(&text, &format!("# LINE{k}"))
        );
        assert_eq!(cols.contains(&name), k >= 3, "line {k}: {cols:?}");
    }
    assert!(
        s.limitations
            .iter()
            .any(|l| l == "3 more allocation sites grew and have no column"),
        "{:?}",
        s.limitations
    );
    // Every kept column has a value at every point.
    for c in &cols {
        let i = s.column_index(c).unwrap();
        assert!(s.values(i).iter().all(Option::is_some), "{c}");
    }
}

const VIA_LIBRARY: &str = "\
import copy

KEPT = []


def keep_via_library(i):
    KEPT.append(copy.copy([i]))  # VIA
";

#[test]
fn a_block_made_inside_a_library_function_is_counted_at_the_project_line_when_the_depth_allows() {
    if !have_python() {
        return;
    }
    let line = line_of(VIA_LIBRARY, "# VIA");
    let name = format!("py.blocks@via.py:{line}");
    let run = |env: &[(&str, &str)]| {
        let fx = fixture();
        write(&fx, "via.py", VIA_LIBRARY);
        run_driver(
            &fx,
            "import via\n    _state['via'] = via",
            "_state['via'].keep_via_library(i)",
            50,
            1,
            4,
            env,
        )
    };
    // The copy is made by `list.copy` called from copy.py: the project line is the second frame.
    let two = run(&[("DLA_TRACEMALLOC_FRAMES", "2")]);
    let i = two
        .column_index(&name)
        .unwrap_or_else(|| panic!("{:?}", block_columns(&two)));
    let v = two.values(i);
    assert!(v.iter().all(Option::is_some), "{v:?}");
    assert_eq!(
        v[4].unwrap() - v[0].unwrap(),
        4.0 * 50.0 * 2.0,
        "two blocks per call: {v:?}"
    );
    // With one frame (the default of a driver run) the innermost frame is in the library: no
    // site, and the limitation line says so.
    for env in [&[("DLA_TRACEMALLOC_FRAMES", "1")][..], &[][..]] {
        let one = run(env);
        assert!(block_columns(&one).is_empty(), "{:?}", block_columns(&one));
        assert!(
            one.limitations
                .iter()
                .any(|l| l.starts_with("py.blocks sees only the innermost frame of a block")),
            "{:?}",
            one.limitations
        );
    }
}

#[test]
fn a_process_that_stopped_tracing_gets_no_columns_and_a_limitation() {
    if !have_python() {
        return;
    }
    let fx = fixture();
    project(&fx);
    let s = run_driver(
        &fx,
        &format!("import tracemalloc\n    tracemalloc.stop()\n    {SETUP}"),
        UNIT,
        50,
        1,
        4,
        &[],
    );
    assert!(block_columns(&s).is_empty(), "{:?}", block_columns(&s));
    assert!(
        s.limitations
            .iter()
            .any(|l| l == "tracemalloc not tracing: the py.blocks columns are unavailable"),
        "{:?}",
        s.limitations
    );
}

#[test]
fn a_library_installed_in_editable_mode_is_counted_at_its_own_path_in_the_source_tree() {
    if !have_python() {
        return;
    }
    let fx = fixture();
    // The source tree of the project: a package the scratch copy does not hold (an editable
    // install imports from here), and a file under the run directory, which is never counted.
    let src = fx.ctx.project.join("lib");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("editable.py"), STORE).unwrap();
    let extra = fx.hdir.join("extra");
    fs::create_dir_all(&extra).unwrap();
    fs::write(extra.join("tool.py"), STORE).unwrap();
    let line = line_of(STORE, "# KEEP");
    let s = run_driver(
        &fx,
        "import editable\n    import tool\n    _state['editable'] = editable\n    _state['tool'] = tool",
        "_state['editable'].keep(i)\n    _state['tool'].keep(i)",
        50,
        1,
        4,
        &[(
            "PYTHONPATH",
            &format!(
                "{}:{}:{}:{}",
                fx.agent.display(),
                fx.work.display(),
                src.display(),
                extra.display()
            ),
        )],
    );
    let want = format!("py.blocks@lib/editable.py:{line}");
    assert_eq!(block_columns(&s), [want], "{:?}", block_columns(&s));
}

/// A module with 320 live allocation sites (one object per line, kept for the whole run).
fn many_live_sites() -> String {
    let mut text = String::from("LIVE = []\n\n\ndef noop(i):\n    return i\n\n\n");
    for k in 0..320 {
        text.push_str(&format!("LIVE.append(object())  # SITE{k}\n"));
    }
    text
}

/// py.traced from the second measured point to the last.
fn traced_rise(s: &SeriesFile) -> f64 {
    let i = s.column_index("py.traced").unwrap();
    let v: Vec<f64> = s.values(i).into_iter().flatten().collect();
    // One warm-up point, so the second measured point is index 2.
    v[v.len() - 1] - v[2]
}

/// The most py.traced may rise from the second measured point to the last on a unit with no
/// leak and 320 live sites. Measured: see docs/VERIFY.md (fixed code and the mutation that keeps
/// the counts of every checkpoint in the agent).
const NO_LEAK_TRACED_RISE_BOUND: f64 = 8192.0;

#[test]
fn counting_the_sites_does_not_make_a_run_without_a_leak_read_as_growth() {
    if !have_python() {
        return;
    }
    let fx = fixture();
    write(&fx, "live.py", &many_live_sites());
    let s = run_driver(
        &fx,
        "import live\n    _state['live'] = live",
        "_state['live'].noop(i)",
        50,
        1,
        8,
        &[],
    );
    let rise = traced_rise(&s);
    eprintln!("py.traced rise from the second measured point to the last: {rise}");
    assert!(
        rise <= NO_LEAK_TRACED_RISE_BOUND,
        "py.traced rose by {rise} bytes"
    );
    assert!(block_columns(&s).is_empty(), "{:?}", block_columns(&s));
}

#[test]
fn the_pytest_loop_driver_is_a_harness_file_too() {
    if !have_python() {
        return;
    }
    let fx = fixture();
    project(&fx);
    // The driver keeps an object per unit in its own hook; the project holds a copy of the
    // driver named as the pytest loop is: neither has a column.
    let s = run_driver_named(&fx, "dla_pytest_loop.py", SETUP, UNIT, 50, 1, 6, &[]);
    let line = line_of(STORE, "# KEEP");
    assert_eq!(
        block_columns(&s),
        [format!("py.blocks@store.py:{line}")],
        "only the keeping line of the project has a column"
    );
}

/// The file-name rule of the agent's test-file test, one file per rule, in a directory named
/// `Tests` (a capital T: not a test directory by the rule, as in the library this was found on),
/// next to a file that is not a test file. Every file keeps one object per call.
#[test]
fn the_file_name_rule_of_a_test_file_holds_without_a_test_directory() {
    if !have_python() {
        return;
    }
    let fx = fixture();
    let keep = "KEPT = []\n\n\ndef keep(i):\n    KEPT.append(object())  # KEEP\n";
    for f in [
        "Tests/test_prefix.py",
        "Tests/suffix_test.py",
        "Tests/conftest.py",
        "Tests/helper.py",
    ] {
        write(&fx, f, keep);
    }
    write(&fx, "Tests/__init__.py", "");
    let setup = "import importlib\n    _state['mods'] = [importlib.import_module('Tests.' + n) for n in ('test_prefix', 'suffix_test', 'conftest', 'helper')]";
    let s = run_driver(
        &fx,
        setup,
        "for m in _state['mods']:\n        m.keep(i)",
        50,
        1,
        4,
        &[],
    );
    let line = line_of(keep, "# KEEP");
    assert_eq!(
        block_columns(&s),
        [format!("py.blocks@Tests/helper.py:{line}")],
        "only the file that is no test file by its name has a column"
    );
}
