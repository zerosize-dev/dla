//! The Python agent's raw captures (`DLA_CAPTURE`): tracemalloc stacks, memray's report, object
//! counts by type and thread/task/ResourceWarning creation sites, written by
//! `harness/python/sitecustomize.py`.
//!
//! These tests start `python3` with the agent first on `PYTHONPATH`, the way `dla harness run`
//! does, on copies of the benchmark fixtures. Each test prints why it is skipped when `python3`
//! is missing; the memray report is checked only when memray imports in that interpreter, and
//! the missing-memray path (a capture entry with the install hint) when it does not.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use dla_core::harness::{CaptureFile, CaptureKind};
use dla_workload::harness::capture::{self, select_points};
use serde_json::Value;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn python3_ok(code: &str) -> bool {
    Command::new("python3")
        .args(["-c", code])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn have_python() -> bool {
    // 3.8 is the oldest the agent supports; the tests use only the standard library.
    let ok = python3_ok("import sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)");
    if !ok {
        eprintln!("skipped: python3 (3.8 or newer) is not available");
    }
    ok
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if src.is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

struct Run {
    dir: tempfile::TempDir,
    project: PathBuf,
    agent: PathBuf,
}

impl Run {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("project");
        copy_dir(
            &repo().join("bench/fixtures/python-ingest-two-leaks/project"),
            &project,
        );
        // The agent directory holds what `dla harness gen` writes there.
        let agent = dir.path().join("agent");
        fs::create_dir_all(&agent).unwrap();
        for f in ["sitecustomize.py", "dla_memray_leaks.py"] {
            fs::copy(repo().join("harness/python").join(f), agent.join(f)).unwrap();
        }
        Self {
            dir,
            project,
            agent,
        }
    }

    fn out(&self) -> PathBuf {
        self.dir.path().join("out.jsonl")
    }

    fn cap(&self) -> PathBuf {
        self.dir.path().join("cap")
    }

    /// Runs `python3 <args>` in the project with the agent first on `PYTHONPATH`.
    fn python(&self, args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
        let mut c = Command::new("python3");
        c.args(args)
            .current_dir(&self.project)
            .env(
                "PYTHONPATH",
                format!("{}:{}", self.agent.display(), self.project.display()),
            )
            .env("DLA_HARNESS_OUT", self.out())
            .env("DLA_PROJECT", &self.project)
            .env("DLA_CAPTURE_DIR", self.cap())
            .env_remove("DLA_CAPTURE")
            .env_remove("DLA_WARMUP")
            .env_remove("DLA_ROUNDS")
            .env_remove("DLA_CAPTURE_DUMPS")
            .env_remove("DLA_TRACEMALLOC_FRAMES")
            .env_remove("DLA_PY_MEMRAY")
            .env_remove("PYTHONASYNCIODEBUG");
        for (k, v) in env {
            c.env(k, v);
        }
        let o = c.output().unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        o
    }

    fn records(&self) -> Vec<Value> {
        fs::read_to_string(self.out())
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    /// The one `<cap>/<pid>` directory.
    fn pid_dir(&self) -> PathBuf {
        let mut dirs: Vec<PathBuf> = fs::read_dir(self.cap())
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .collect();
        assert_eq!(dirs.len(), 1, "{dirs:?}");
        dirs.pop().unwrap()
    }
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn captures(rec: &Value) -> Vec<Value> {
    rec["captures"].as_array().cloned().unwrap_or_default()
}

fn json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn has_memray() -> bool {
    python3_ok("import memray")
}

#[test]
fn file_names_classify_and_each_format_gets_its_own_command() {
    for (name, kind, seq) in [
        ("py-3.alloc.json", CaptureKind::Alloc, 3),
        ("py-1.memray.json", CaptureKind::Alloc, 1),
        ("py-2.types.json", CaptureKind::Histogram, 2),
        ("py-4.nonheap.json", CaptureKind::NonHeap, 4),
    ] {
        let c = capture::classify(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!((c.0, c.1), (kind, seq), "{name}");
    }
    assert!(capture::classify("py-x.alloc.json").is_none());
    assert!(capture::classify("py-1.other.json").is_none());

    let file = |kind, label: &str, seq: u32, x: f64, tail: &str| CaptureFile {
        kind,
        path: format!("p/capture/7/py-{seq}.{tail}.json"),
        label: label.to_owned(),
        seq,
        x: Some(x),
        role: None,
        bytes: 10,
        ms: None,
        captured_at: String::new(),
    };
    let mut files = Vec::new();
    for k in 1..=3u32 {
        let x = f64::from(k) * 100.0;
        files.push(file(CaptureKind::Alloc, "p1-fresh", k, x, "alloc"));
        files.push(file(CaptureKind::Alloc, "p1-fresh", k, x, "memray"));
        files.push(file(CaptureKind::Histogram, "p1-fresh", k, x, "types"));
        files.push(file(CaptureKind::NonHeap, "p1-fresh", k, x, "nonheap"));
    }
    let cmds = capture::summarize_commands(&files, &|p| p.to_owned(), false);
    let op_of = |c: &String| c.split(' ').nth(2).unwrap().to_owned();
    let ops: Vec<String> = cmds.iter().map(op_of).collect();
    assert_eq!(
        ops,
        [
            "type-histogram-diff",
            "alloc-stack-diff",
            "alloc-stack-diff",
            "non-heap"
        ],
        "{cmds:?}"
    );
    // No command mixes two formats.
    for c in &cmds {
        let tails: std::collections::BTreeSet<&str> = c
            .split(' ')
            .skip(3)
            .map(|f| f.rsplit('.').nth(1).unwrap())
            .collect();
        assert_eq!(tails.len(), 1, "{c}");
        assert_eq!(c.split(' ').skip(3).count(), 3, "{c}");
    }
    assert!(cmds[1].contains(".alloc.json") && cmds[2].contains(".memray.json"));
}

#[test]
fn python_keeps_histogram_alloc_and_nonheap() {
    let all: capture::CaptureSet = "all".parse().unwrap();
    let py = all.intersect(&capture::supported(dla_core::runtime::Runtime::Python));
    assert_eq!(py.names(), ["histogram", "nonheap", "alloc"]);
}

#[test]
fn command_mode_writes_every_kind_at_exit_and_lists_it() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    run.python(
        &["-m", "ingest.load", "--iterations", "300"],
        &[("DLA_CAPTURE", "histogram,alloc,nonheap")],
    );
    let dir = run.pid_dir();
    let memray = has_memray();
    let mut want = vec!["py-1.alloc.json", "py-1.nonheap.json", "py-1.types.json"];
    if memray {
        want.insert(1, "py-1.memray.json");
    }
    assert_eq!(names(&dir), want);
    let recs = run.records();
    let rec = recs.iter().find(|r| r["phase"] == "exit").unwrap();
    let caps = captures(rec);
    let errors: Vec<&Value> = caps.iter().filter(|c| c.get("error").is_some()).collect();
    if memray {
        assert!(errors.is_empty(), "{errors:?}");
    } else {
        // memray is absent from this interpreter: one entry says so, with the install hint.
        assert_eq!(errors.len(), 1, "{caps:?}");
        let e = errors[0]["error"].as_str().unwrap();
        assert!(
            e.contains("memray is not installed") && e.contains("pip install memray"),
            "{e}"
        );
        assert_eq!(errors[0]["kind"], "alloc");
    }
    for c in caps.iter().filter(|c| c.get("error").is_none()) {
        let file = c["file"].as_str().unwrap();
        assert_eq!(
            fs::metadata(dir.join(file)).unwrap().len(),
            c["bytes"].as_u64().unwrap(),
            "{file}"
        );
        assert!(capture::classify(file).is_some(), "{file}");
    }
    // tracemalloc: innermost frame first, the leak line of the project among the stacks.
    let alloc = json(&dir.join("py-1.alloc.json"));
    assert_eq!(alloc["dla_pyalloc"], 1);
    assert_eq!(alloc["frames"], 10);
    let stacks = alloc["stacks"].as_array().unwrap();
    assert!(!stacks.is_empty() && stacks.len() <= 150);
    let first_project = |s: &Value| -> Option<String> {
        s["frames"].as_array().unwrap().iter().find_map(|f| {
            let file = f[0].as_str().unwrap();
            file.starts_with("ingest/")
                .then(|| format!("{file}:{}", f[1]))
        })
    };
    assert!(
        stacks
            .iter()
            .any(|s| first_project(s).as_deref() == Some("ingest/watchers.py:8")),
        "no stack at the watcher's allocation line"
    );
    // The agent's own work is not in the stacks.
    let agent = fs::canonicalize(&run.agent).unwrap();
    let from_agent =
        stacks.iter().any(|s| {
            s["frames"].as_array().unwrap().iter().any(|f| {
                fs::canonicalize(f[0].as_str().unwrap()).is_ok_and(|p| p.starts_with(&agent))
            })
        });
    assert!(!from_agent, "agent frames leaked into the stacks");
    // Types: the project's classes, counts after a full collection.
    let types = json(&dir.join("py-1.types.json"));
    assert_eq!(types["dla_pytypes"], 1);
    let record = types["types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "ingest.models.Record")
        .expect("the project's Record type is counted");
    assert!(record["count"].as_u64().unwrap() >= 300);
    // The record keeps its 20-row top for the harness-types reader.
    assert_eq!(rec["types"].as_array().unwrap().len(), 20);
    // Resources: the main thread, no task, no warning.
    let res = json(&dir.join("py-1.nonheap.json"));
    assert_eq!(res["dla_py_resources"], 1);
    assert_eq!(res["threads"]["groups"][0]["pattern"], "MainThread");
    if memray {
        let m = json(&dir.join("py-1.memray.json"));
        assert_eq!(m["dla_memray"], 1);
        let top = &m["stacks"][0];
        assert!(top["size"].as_u64().unwrap() > 0);
        let agent = fs::canonicalize(&run.agent).unwrap();
        let from_agent = m["stacks"].as_array().unwrap().iter().any(|s| {
            s["frames"].as_array().unwrap().iter().any(|f| {
                fs::canonicalize(f[1].as_str().unwrap()).is_ok_and(|p| p.starts_with(&agent))
            })
        });
        assert!(!from_agent, "agent frames leaked into the report");
        // The raw memray file is not kept.
        assert!(!dir.join("memray.bin").exists());
    }
}

#[test]
fn the_frame_count_comes_from_the_harness_variable() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    run.python(
        &["-m", "ingest.load", "--iterations", "100"],
        &[
            ("DLA_CAPTURE", "alloc"),
            ("DLA_TRACEMALLOC_FRAMES", "3"),
            ("DLA_PY_MEMRAY", "0"),
        ],
    );
    let alloc = json(&run.pid_dir().join("py-1.alloc.json"));
    assert_eq!(alloc["frames"], 3);
    assert!(
        alloc["stacks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["frames"].as_array().unwrap().len() <= 3)
    );
    // memray stays out when asked, and says nothing.
    assert_eq!(names(&run.pid_dir()), ["py-1.alloc.json"]);
    let recs = run.records();
    let rec = recs.iter().find(|r| r["phase"] == "exit").unwrap();
    assert_eq!(captures(rec).len(), 1);
}

#[test]
fn without_capture_nothing_is_written_and_one_frame_is_traced() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    run.python(&["-m", "ingest.load", "--iterations", "50"], &[]);
    assert!(!run.cap().exists());
    let recs = run.records();
    let rec = recs.iter().find(|r| r["phase"] == "exit").unwrap();
    assert!(rec.get("captures").is_none());
    assert!(rec["values"]["py.traced"].as_u64().unwrap() > 0);
}

/// A driver with its hooks filled: the fixture's pipeline, one batch per unit.
fn write_driver(run: &Run) -> PathBuf {
    let template = fs::read_to_string(repo().join("harness/python/dla_harness.py")).unwrap();
    let text = template
        .replace("@@DLA_STEP@@", "50")
        .replace("@@DLA_WARMUP@@", "1")
        .replace("@@DLA_ROUNDS@@", "5")
        .replace("@@DLA_REPEAT@@", "10")
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            "    from ingest.pipeline import Pipeline\n    _state['p'] = Pipeline()",
        )
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            "    _state['p'].process_batch([f'{1700000000 + i},probe-001,1.5,north' for _ in range(2)])",
        );
    // The driver lives apart from the agent, as `dla harness gen` writes it.
    let dir = run.dir.path().join("driver");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("dla_harness.py");
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn driver_mode_takes_the_heavy_captures_at_the_selected_checkpoints() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let driver = write_driver(&run);
    run.python(
        &[driver.to_str().unwrap()],
        &[
            ("DLA_CAPTURE", "histogram,alloc,nonheap"),
            ("DLA_WARMUP", "1"),
            ("DLA_ROUNDS", "5"),
            ("DLA_STEP", "50"),
            ("DLA_CAPTURE_DUMPS", "4"),
            ("DLA_PY_MEMRAY", "0"),
        ],
    );
    // The same rule as the JVM and Node agents: the end of the warm-up, the last round and
    // evenly spaced rounds between.
    let want: Vec<u32> = select_points(6, 1, 4).into_iter().collect();
    assert_eq!(want.len(), 4);
    let recs = run.records();
    let with_caps: Vec<u64> = recs
        .iter()
        .filter(|r| r["phase"] == "checkpoint" && !captures(r).is_empty())
        .map(|r| r["x"].as_u64().unwrap())
        .collect();
    let expect: Vec<u64> = want.iter().map(|k| u64::from(*k) * 50).collect();
    assert_eq!(with_caps, expect);
    let dir = run.pid_dir();
    assert_eq!(
        names(&dir),
        [
            "py-1.alloc.json",
            "py-1.nonheap.json",
            "py-1.types.json",
            "py-2.alloc.json",
            "py-2.nonheap.json",
            "py-2.types.json",
            "py-3.alloc.json",
            "py-3.nonheap.json",
            "py-3.types.json",
            "py-4.alloc.json",
            "py-4.nonheap.json",
            "py-4.types.json"
        ]
    );
    // The stacks of the watcher class grow from the first file to the last.
    let live = |n: u32| -> u64 {
        json(&dir.join(format!("py-{n}.alloc.json")))["stacks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["frames"][0][0] == "ingest/watchers.py")
            .map(|s| s["size"].as_u64().unwrap())
            .sum()
    };
    assert!(
        live(1) > 0 && live(4) > live(1) * 2,
        "{} {}",
        live(1),
        live(4)
    );
}

#[test]
fn threads_tasks_and_unclosed_files_are_recorded_with_their_creation_sites() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let script = run.project.join("gateway.py");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../dla-cli/tests/fixtures/py-sources/gateway.py"),
        &script,
    )
    .unwrap();
    run.python(&["gateway.py", "30"], &[("DLA_CAPTURE", "nonheap")]);
    let res = json(&run.pid_dir().join("py-1.nonheap.json"));
    let group = |section: &str, key: &str, val: &str| -> Value {
        res[section]["groups"]
            .as_array()
            .unwrap()
            .iter()
            .find(|g| g[key] == val)
            .cloned()
            .unwrap_or_else(|| panic!("no {val} in {res}"))
    };
    let hb = group("threads", "pattern", "heartbeat-*");
    assert_eq!(
        (hb["count"].as_u64(), hb["site"].as_str()),
        (Some(30), Some("gateway.py:24"))
    );
    let task = group("tasks", "coro", "_watch");
    assert_eq!(
        (task["count"].as_u64(), task["site"].as_str()),
        (Some(30), Some("gateway.py:27"))
    );
    assert_eq!(task["frame"], "gateway.py:20");
    let file = group("resource_warnings", "what", "unclosed file");
    assert_eq!(
        (file["count"].as_u64(), file["site"].as_str()),
        (Some(30), Some("gateway.py:28"))
    );
    assert_eq!(res["threads"]["total"], 31);
}

/// Runs `script` as `<project>/<name>` under the agent with only `nonheap` captured.
fn nonheap_of(run: &Run, name: &str, script: &str) -> Value {
    fs::write(run.project.join(name), script).unwrap();
    run.python(&[name], &[("DLA_CAPTURE", "nonheap")]);
    json(&run.pid_dir().join("py-1.nonheap.json"))
}

fn groups<'a>(res: &'a Value, section: &str) -> &'a Vec<Value> {
    res[section]["groups"].as_array().unwrap()
}

#[test]
fn a_thread_made_in_a_plain_class_init_is_sited_at_that_line() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let res = nonheap_of(
        &run,
        "watcher.py",
        "import threading, time\n\
         class Watcher:\n\
         \x20   def __init__(self):\n\
         \x20       self.t = threading.Thread(target=time.sleep, args=(30,), daemon=True, name='poller-1')\n\
         \x20       self.t.start()\n\
         \n\
         w = Watcher()\n",
    );
    let g = groups(&res, "threads")
        .iter()
        .find(|g| g["pattern"] == "poller-*")
        .unwrap_or_else(|| panic!("{res}"));
    assert_eq!(g["site"], "watcher.py:4", "{g}");
}

#[test]
fn a_thread_subclass_init_is_still_skipped() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let res = nonheap_of(
        &run,
        "sub.py",
        "import threading, time\n\
         class T(threading.Thread):\n\
         \x20   def __init__(self):\n\
         \x20       super().__init__(daemon=True, name='sub-1')\n\
         \x20   def run(self):\n\
         \x20       time.sleep(30)\n\
         t = T()\n\
         t.start()\n",
    );
    let g = groups(&res, "threads")
        .iter()
        .find(|g| g["pattern"] == "sub-*")
        .unwrap_or_else(|| panic!("{res}"));
    assert_eq!(g["site"], "sub.py:7", "{g}");
}

#[test]
fn resource_warning_labels_are_plain_and_digit_free() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let res = nonheap_of(
        &run,
        "warn.py",
        "import warnings\n\
         warnings.warn('leaked conn to db.internal password=hunter2 pid 4412', ResourceWarning)\n\
         warnings.warn('subprocess 4412 is still running', ResourceWarning)\n\
         warnings.warn('subprocess 977 is still running', ResourceWarning)\n",
    );
    let text = res.to_string();
    assert!(
        !text.contains("hunter2") && !text.contains("4412"),
        "{text}"
    );
    let whats: Vec<(&str, u64)> = groups(&res, "resource_warnings")
        .iter()
        .map(|g| (g["what"].as_str().unwrap(), g["count"].as_u64().unwrap()))
        .collect();
    assert!(
        whats.contains(&("subprocess * is still running", 2)),
        "{whats:?}"
    );
    assert!(whats.contains(&("ResourceWarning", 1)), "{whats:?}");
}

#[test]
fn nonheap_alone_sites_a_file_opened_inside_the_stdlib() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let res = nonheap_of(
        &run,
        "tmp.py",
        "import tempfile\n\
         def make():\n\
         \x20   tempfile.TemporaryFile()\n\
         make()\n",
    );
    let g = groups(&res, "resource_warnings")
        .iter()
        .find(|g| g["site"] == "tmp.py:3")
        .unwrap_or_else(|| panic!("{res}"));
    assert!(g["count"].as_u64().unwrap() >= 1);
}

#[test]
fn the_first_alloc_capture_baselines_every_stack() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let mut src = String::from("import sitecustomize as dla\nkeep = []\n");
    for _ in 0..200 {
        src.push_str("keep.append(bytearray(500))\n");
    }
    src.push_str("out, lim = [], []\ndla._capture_tracemalloc(out, dla._capture_dir(), lim)\n");
    src.push_str("assert len(dla._tm_first) >= 200, len(dla._tm_first)\n");
    fs::write(run.project.join("base.py"), src).unwrap();
    run.python(
        &["base.py"],
        &[("DLA_CAPTURE", "alloc"), ("DLA_PY_MEMRAY", "0")],
    );
}

#[test]
fn the_memray_helper_finds_memray_through_site() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let site = run.dir.path().join("elsewhere");
    fs::create_dir_all(site.join("memray")).unwrap();
    fs::write(
        site.join("memray/__init__.py"),
        "class FileReader:\n    def __init__(self, p):\n        raise RuntimeError('fake reader')\n",
    )
    .unwrap();
    let helper = run.agent.join("dla_memray_leaks.py");
    let go = |extra: &[&str]| {
        let o = Command::new("python3")
            .args(["-I", helper.to_str().unwrap(), "x.bin", "o.json"])
            .args(extra)
            .current_dir(run.dir.path())
            .output()
            .unwrap();
        String::from_utf8_lossy(&o.stderr).into_owned()
    };
    if python3_ok("import memray") {
        eprintln!("skipped: a real memray is installed");
        return;
    }
    assert!(go(&[]).contains("cannot be imported"));
    let with_site = go(&["--site", site.to_str().unwrap()]);
    assert!(
        with_site.contains("cannot read") && with_site.contains("fake reader"),
        "{with_site}"
    );
}

#[test]
fn a_driver_reports_the_functions_it_ran_in_its_checkpoints() {
    if !have_python() || !python3_ok("import sys; sys.monitoring") {
        return;
    }
    let run = Run::new();
    let driver = write_driver(&run);
    run.python(&[driver.to_str().unwrap()], &[("DLA_PY_MEMRAY", "0")]);
    let recs = run.records();
    // A driver writes no `exit` record, so the coverage section rides on its checkpoints
    // (the first, the second, the fourth... and the last).
    assert!(recs.iter().all(|r| r["phase"] != "exit"), "{recs:?}");
    let last = recs
        .iter()
        .rfind(|r| r["phase"] == "checkpoint")
        .expect("a checkpoint");
    let cov = &last["coverage"];
    assert_eq!(cov["kind"], "functions", "{last}");
    let items = cov["items"].as_array().unwrap();
    assert!(
        items.iter().any(|i| i.as_str().unwrap().contains("ingest")
            && i.as_str().unwrap().contains("process_batch")),
        "{items:?}"
    );
    // The first checkpoint carries it too, so a driver cut off early still says what ran.
    let first = recs.iter().find(|r| r["phase"] == "checkpoint").unwrap();
    assert_eq!(first["coverage"]["kind"], "functions", "{first}");
}
