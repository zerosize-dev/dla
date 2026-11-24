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
fn python_keeps_heap_histogram_alloc_and_nonheap() {
    let all: capture::CaptureSet = "all".parse().unwrap();
    let py = all.intersect(&capture::supported(dla_core::runtime::Runtime::Python));
    assert_eq!(py.names(), ["heap", "histogram", "nonheap", "alloc"]);
    let heap: capture::CaptureSet = "heap".parse().unwrap();
    assert_eq!(
        heap.intersect(&capture::supported(dla_core::runtime::Runtime::Python))
            .names(),
        ["heap"]
    );
}

#[test]
fn the_holder_chains_file_is_a_heap_capture_with_a_heap_graph_command_of_its_own() {
    let c = capture::classify("py-1.retain.json").unwrap();
    assert_eq!((c.0, c.1), (CaptureKind::Heap, 1));
    assert!(capture::classify("py-x.retain.json").is_none());
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
    let files = vec![
        file(CaptureKind::Alloc, "p1-fresh", 1, 100.0, "alloc"),
        file(CaptureKind::Heap, "p1-fresh", 1, 400.0, "retain"),
    ];
    let cmds = capture::summarize_commands(&files, &|p| p.to_owned(), false);
    assert_eq!(
        cmds,
        [
            "dla summarize heap-graph p/capture/7/py-1.retain.json",
            "dla summarize alloc-stack-diff p/capture/7/py-1.alloc.json"
        ]
    );
    // Command mode, one process per point: only the newest file is read.
    let points: Vec<CaptureFile> = (1..=3u32)
        .map(|k| {
            let mut f = file(
                CaptureKind::Heap,
                &format!("p{k}-native"),
                1,
                f64::from(k) * 100.0,
                "retain",
            );
            f.path = format!("p{k}/capture/{k}/py-1.retain.json");
            f
        })
        .collect();
    let cmds = capture::summarize_commands(&points, &|p| p.to_owned(), false);
    assert_eq!(
        cmds,
        ["dla summarize heap-graph p3/capture/3/py-1.retain.json"]
    );
}

#[test]
fn the_trim_purge_reads_before_from_the_checkpoints_own_reading() {
    if !have_python() || !cfg!(target_os = "linux") {
        eprintln!("skipped: the trim purge reads /proc/self/status on Linux");
        return;
    }
    let run = Run::new();
    run.python(&["-c", "pass"], &[("DLA_PURGE", "trim")]);
    let recs = run.records();
    let rec = recs.iter().find(|r| r["phase"] == "exit").unwrap();
    let read = |section: &Value, key: &str| {
        section[key]
            .as_f64()
            .unwrap_or_else(|| panic!("{key} in {rec}"))
    };
    let purge = &rec["purge"];
    assert_eq!(purge["kind"], "trim", "{rec}");
    // `before` is the checkpoint's own reading, not a new one. Between the two readings the
    // purge runs `malloc_trim` only: a module imported there (`platform` adds about 160 KB of
    // anonymous memory in a bare interpreter, ctypes about 2 MB, a part of it file-backed) shows
    // as memory the purge failed to return, and a trim that gave back up to that much would
    // read as retained.
    for key in ["os.rss", "os.rss_file"] {
        let own = read(&rec["values"], key);
        let before = read(&purge["before"], key);
        let after = read(&purge["after"], key);
        assert_eq!(
            before, own,
            "{key}: before against the checkpoint's reading"
        );
        assert!(
            after - before <= 32.0 * 1024.0,
            "{key}: after {after} stands above before {before}: the purge step loaded something"
        );
    }
}

#[test]
fn every_process_loads_the_trim_library_at_start_up_not_only_the_purged_one() {
    if !have_python() || !cfg!(target_os = "linux") {
        eprintln!("skipped: the trim purge is for Linux");
        return;
    }
    // The purge is asked only of the process of the last point (`DLA_PURGE` is set for that one):
    // the others must carry the same load, or the last point stands above the series by it.
    let run = Run::new();
    run.python(
        &[
            "-c",
            "import sys; sys.exit(0 if 'ctypes' in sys.modules else 3)",
        ],
        &[],
    );
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

#[test]
fn coverage_lists_functions_of_the_source_root_outside_the_scratch_copy() {
    if !have_python() || !python3_ok("import sys; sys.monitoring") {
        return;
    }
    // A package installed in editable mode is imported from the project the copy was made from.
    let run = Run::new();
    let src = run.dir.path().join("source");
    fs::create_dir_all(src.join("lib")).unwrap();
    fs::create_dir_all(src.join(".dla")).unwrap();
    fs::write(src.join("lib/tool.py"), "def work():\n    return 1\n").unwrap();
    fs::write(src.join(".dla/hidden.py"), "def secret():\n    return 2\n").unwrap();
    fs::write(
        run.project.join("use_lib.py"),
        format!(
            "import runpy\nimport lib.tool\nlib.tool.work()\n\
             runpy.run_path({:?})['secret']()\n",
            src.join(".dla/hidden.py")
        ),
    )
    .unwrap();
    let path = format!("{}:{}", run.agent.display(), src.display());
    let items = |with_root: bool| -> Vec<String> {
        let _ = fs::remove_file(run.out());
        let mut env = vec![("PYTHONPATH", path.as_str())];
        if with_root {
            env.push(("DLA_SOURCE_ROOT", src.to_str().unwrap()));
        }
        run.python(&["use_lib.py"], &env);
        let recs = run.records();
        let rec = recs.iter().find(|r| r["phase"] == "exit").unwrap();
        rec["coverage"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.as_str().unwrap().to_string())
            .collect()
    };
    let with = items(true);
    assert!(
        with.iter().any(|i| i == "lib.tool:work@lib/tool.py:1"),
        "{with:?}"
    );
    assert!(with.iter().all(|i| !i.contains("secret")), "{with:?}");
    let without = items(false);
    assert!(
        without.iter().all(|i| !i.contains("lib.tool")),
        "{without:?}"
    );
}

/// Runs `script` as `<project>/<name>` under the agent and returns every record it wrote. A
/// script calls `sitecustomize.checkpoint(k)` where it wants a reading.
fn records_of(run: &Run, name: &str, script: &str) -> Vec<Value> {
    fs::write(run.project.join(name), script).unwrap();
    run.python(&[name], &[]);
    run.records()
}

fn frozen(rec: &Value) -> std::collections::BTreeMap<String, f64> {
    rec["groups"]["py.gc_frozen"]
        .as_object()
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.as_f64().unwrap()))
                .collect()
        })
        .unwrap_or_default()
}

fn frozen_limitation(rec: &Value) -> bool {
    rec["limitations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l.as_str().unwrap().contains("permanent generation"))
}

const CYCLES: &str = "import gc, sitecustomize\n\
     class Node:\n\
     \x20   def __init__(self):\n\
     \x20       self.me = self\n\
     def churn(n):\n\
     \x20   return [Node() for _ in range(n)]\n";

#[test]
fn a_freeze_in_a_loop_is_attributed_to_its_call_site_and_rises() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let recs = records_of(
        &run,
        "freezer.py",
        &format!(
            "{CYCLES}def batch(k):\n\
             \x20   held = churn(2000)\n\
             \x20   gc.freeze()\n\
             \x20   del held\n\
             \x20   sitecustomize.checkpoint(k)\n\
             for k in range(1, 4):\n\
             \x20   batch(k)\n"
        ),
    );
    let cps: Vec<&Value> = recs.iter().filter(|r| r["phase"] == "checkpoint").collect();
    assert_eq!(cps.len(), 3, "{recs:?}");
    let series: Vec<f64> = cps
        .iter()
        .map(|r| {
            let g = frozen(r);
            assert_eq!(g.keys().collect::<Vec<_>>(), ["freezer.py:9"], "{g:?}");
            g["freezer.py:9"]
        })
        .collect();
    assert!(
        series[1] - series[0] >= 1500.0 && series[2] - series[1] >= 1500.0,
        "{series:?}"
    );
    assert!(cps.iter().all(|r| frozen_limitation(r)), "{cps:?}");
}

#[test]
fn the_shares_of_several_sites_sum_to_the_current_freeze_count() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let recs = records_of(
        &run,
        "two.py",
        &format!(
            "{CYCLES}def early():\n\
             \x20   gc.freeze()\n\
             def temp():\n\
             \x20   held = [[] for _ in range(4000)]\n\
             \x20   gc.freeze()\n\
             \x20   del held\n\
             def later():\n\
             \x20   held = churn(3000)\n\
             \x20   gc.freeze()\n\
             \x20   del held\n\
             early()\n\
             temp()\n\
             later()\n\
             later()\n\
             sitecustomize.checkpoint(1, True)\n"
        ),
    );
    let rec = recs.iter().find(|r| r["phase"] == "checkpoint").unwrap();
    let g = frozen(rec);
    assert_eq!(
        g.keys().cloned().collect::<Vec<_>>(),
        ["two.py:11", "two.py:15", "two.py:8"],
        "{g:?}"
    );
    let count: f64 = rec["limitations"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|l| {
            l.as_str()?
                .strip_prefix("gc.freeze moved ")?
                .split(' ')
                .next()?
                .parse()
                .ok()
        })
        .expect("limitation names the count");
    assert_eq!(g.values().sum::<f64>(), count, "{g:?}");
    assert!(g["two.py:15"] >= 1500.0, "{g:?}");
}

#[test]
fn one_freeze_stays_flat_and_an_unfreeze_with_a_collection_empties_the_group() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let once = records_of(
        &run,
        "once.py",
        &format!(
            "{CYCLES}def setup():\n\
             \x20   held = churn(3000)\n\
             \x20   gc.freeze()\n\
             \x20   del held\n\
             setup()\n\
             for k in range(1, 4):\n\
             \x20   churn(3000)\n\
             \x20   gc.collect()\n\
             \x20   sitecustomize.checkpoint(k)\n"
        ),
    );
    let series: Vec<f64> = once
        .iter()
        .filter(|r| r["phase"] == "checkpoint")
        .map(|r| frozen(r)["once.py:9"])
        .collect();
    assert_eq!(series.len(), 3);
    assert!(series[2] - series[0] <= 50.0, "{series:?}");
    let _ = fs::remove_file(run.out());
    let cycled = records_of(
        &run,
        "cycled.py",
        &format!(
            "{CYCLES}def batch(k):\n\
             \x20   held = churn(3000)\n\
             \x20   gc.freeze()\n\
             \x20   del held\n\
             \x20   gc.unfreeze()\n\
             \x20   gc.collect()\n\
             \x20   sitecustomize.checkpoint(k)\n\
             for k in range(1, 4):\n\
             \x20   batch(k)\n"
        ),
    );
    for r in cycled.iter().filter(|r| r["phase"] == "checkpoint") {
        assert_eq!(frozen(r).get("cycled.py:9"), Some(&0.0), "{r}");
        assert!(!frozen_limitation(r), "{r}");
    }
}

#[test]
fn without_a_freeze_there_is_no_group_and_a_failing_freeze_reaches_the_target() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    let none = records_of(
        &run,
        "plain.py",
        &format!("{CYCLES}churn(500)\ngc.collect()\nsitecustomize.checkpoint(1)\n"),
    );
    assert!(
        none.iter()
            .all(|r| r.get("groups").is_none() && !frozen_limitation(r)),
        "{none:?}"
    );
    let _ = fs::remove_file(run.out());
    // A bad call fails with the interpreter's own error, and the agent writes nothing about it.
    let bad = records_of(
        &run,
        "bad.py",
        "import gc, sitecustomize\n\
         def f():\n\
         \x20   try:\n\
         \x20       gc.freeze(1)\n\
         \x20   except TypeError:\n\
         \x20       print('typeerror')\n\
         f()\n\
         sitecustomize.checkpoint(1)\n",
    );
    let rec = bad.iter().find(|r| r["phase"] == "checkpoint").unwrap();
    assert_eq!(frozen(rec).values().sum::<f64>(), 0.0, "{rec}");
}

#[test]
fn a_probe_never_imports_and_says_so_for_shapes_a_project_really_has() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    // A module that would leave a mark if it were imported, a package whose parent alone is
    // loaded, and specs that name no module at all.
    fs::write(
        run.project.join("sidecar.py"),
        "open('imported-sidecar', 'w').close()\n",
    )
    .unwrap();
    fs::create_dir_all(run.project.join("pkg")).unwrap();
    fs::write(run.project.join("pkg/__init__.py"), "LOADED = [1, 2, 3]\n").unwrap();
    fs::write(
        run.project.join("pkg/sub.py"),
        "open('imported-sub', 'w').close()\nHIDDEN = {}\n",
    )
    .unwrap();
    let probes = run.dir.path().join("probes.json");
    fs::write(
        &probes,
        r#"{"probes": [
            {"id": "P1", "target": {"kind": "py-module", "module": "sidecar"}},
            {"id": "P2", "target": {"kind": "py-module", "module": "pkg.sub"}},
            {"id": "P3", "target": {"kind": "py-global", "module": "pkg", "name": "LOADED"}},
            {"id": "P4", "target": {"kind": "py-module"}},
            {"id": "P5", "target": {"kind": "py-global", "module": "sidecar", "name": "X"}}
        ]}"#,
    )
    .unwrap();
    run.python(
        &["-c", "import pkg"],
        &[("DLA_HARNESS_PROBES", probes.to_str().unwrap())],
    );
    assert!(!run.project.join("imported-sidecar").exists());
    assert!(!run.project.join("imported-sub").exists());
    let rec = run.records().pop().unwrap();
    let probes = rec["probes"].as_array().unwrap();
    let error_of = |id: &str| {
        probes
            .iter()
            .find(|p| p["id"] == id)
            .map(|p| p["error"].as_str().unwrap_or("").to_owned())
            .unwrap_or_else(|| panic!("no record for {id}: {probes:?}"))
    };
    for id in ["P1", "P2", "P4", "P5"] {
        assert!(
            error_of(id).contains("not loaded"),
            "{id}: {}",
            error_of(id)
        );
    }
    let loaded = probes.iter().find(|p| p["id"] == "P3").unwrap();
    assert_eq!(loaded["size"], 3, "a module the workload imported is read");
    let limits = rec["limitations"].as_array().unwrap();
    assert!(
        limits.iter().any(|l| l
            .as_str()
            .unwrap()
            .contains("were not loaded by the workload and were not imported")
            && l.as_str().unwrap().contains("pkg.sub")),
        "{limits:?}"
    );
}

// ---------------------------------------------------------------------------------------
// `dla harness run --capture heap`: the file, its listing and the raw-values switch

/// A run directory with consent for tier 1 (raw values as given), the agent files in
/// `harness/H1/agent` and a scratch project with one module that keeps an object per call under
/// one dictionary key.
struct Driven {
    _dir: tempfile::TempDir,
    ctx: dla_core::toolctx::ToolCtx,
    hdir: PathBuf,
}

fn driven(raw_values: bool) -> Driven {
    use dla_core::{
        Ledger,
        state::{Consent, ConsentGiver, RunState, now_utc},
    };
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    let run_dir = project.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(project.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.consent = Some(Consent {
        tiers: vec![1],
        raw_values,
        given_by: ConsentGiver::Runner,
        recorded_at: now_utc(),
    });
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = dla_core::toolctx::ToolCtx {
        project,
        run_dir: run_dir.clone(),
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    };
    let hdir = run_dir.join("harness/H1");
    let (work, agent) = (hdir.join("work"), hdir.join("agent"));
    fs::create_dir_all(&agent).unwrap();
    fs::create_dir_all(&work).unwrap();
    for e in fs::read_dir(repo().join("harness/python"))
        .unwrap()
        .flatten()
    {
        if e.path().is_file() {
            fs::copy(e.path(), agent.join(e.file_name())).unwrap();
        }
    }
    fs::write(
        work.join("held.py"),
        "KEPT = {}\n\n\ndef keep(i):\n    KEPT.setdefault('zz_marker_key', []).append(object())\n",
    )
    .unwrap();
    Driven {
        _dir: dir,
        ctx,
        hdir,
    }
}

/// Runs the driver of `d` with `capture` and returns the outcome's run directory listing.
fn run_driven(d: &Driven, capture: &str) {
    run_driven_with(d, capture, "import held", "_state['held'].keep(i)", None);
}

/// [`run_driven`] with the import line of the setup hook, the unit hook and a directory of the
/// project's source tree that is on the path but not in the scratch copy (an editable install).
fn run_driven_with(d: &Driven, capture: &str, imports: &str, unit: &str, extra: Option<&Path>) {
    use dla_core::{
        harness::{AgentKind, CommandSpec, HarnessManifest, HarnessMode},
        runtime::Runtime,
        series::KeyMode,
    };
    let (work, agent) = (d.hdir.join("work"), d.hdir.join("agent"));
    let setup = format!(
        "import os, held\n    {imports}\n    _state['held'] = held\n    open({:?}, 'w').write(os.environ.get('DLA_RAW_VALUES', 'unset'))",
        d.hdir.join("raw-env.txt").to_string_lossy()
    );
    let tmpl = fs::read_to_string(agent.join("dla_harness.py")).unwrap();
    let driver = tmpl
        .replace("@@DLA_STEP@@", "40")
        .replace("@@DLA_WARMUP@@", "1")
        .replace("@@DLA_ROUNDS@@", "3")
        .replace("@@DLA_REPEAT@@", "50")
        .replace(
            "raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            &setup,
        )
        .replace(
            "raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            unit,
        );
    fs::create_dir_all(d.hdir.join("driver")).unwrap();
    fs::write(d.hdir.join("driver/dla_harness.py"), &driver).unwrap();
    fs::write(work.join("dla_harness.py"), &driver).unwrap();
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
                work.join("dla_harness.py").to_string_lossy().into_owned(),
            ],
            cwd: ".".into(),
            env: vec![(
                "PYTHONPATH".to_owned(),
                match extra {
                    Some(e) => format!("{}:{}:{}", agent.display(), work.display(), e.display()),
                    None => format!("{}:{}", agent.display(), work.display()),
                },
            )],
        },
        iter_arg: Some("{n}".into()),
        warmup: 1,
        iterations: 3,
        step: 40,
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
    m.save(&d.ctx.run_dir).unwrap();
    fs::write(d.hdir.join("probes.json"), r#"{"probes":[]}"#).unwrap();
    let opts = dla_workload::harness::run::RunOptions {
        capture: capture.parse().unwrap(),
        ..Default::default()
    };
    dla_workload::harness::run::run(&d.ctx, "H1", &opts).unwrap();
}

fn retain_file(d: &Driven) -> PathBuf {
    let mut found = Vec::new();
    let out = d.hdir.join("out");
    for e in walk(&out) {
        if e.file_name().is_some_and(|n| n == "py-1.retain.json") {
            found.push(e);
        }
    }
    assert_eq!(found.len(), 1, "{found:?}");
    found.pop().unwrap()
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            v.extend(walk(&p));
        } else {
            v.push(p);
        }
    }
    v
}

#[test]
fn a_heap_capture_run_writes_the_chains_file_lists_it_and_prints_its_summarize_command() {
    if !have_python() {
        return;
    }
    let d = driven(false);
    run_driven(&d, "heap");
    let file = retain_file(&d);
    let doc = json(&file);
    assert_eq!(doc["dla_pyretain"], 1, "{doc}");
    let sites = doc["sites"].as_array().unwrap();
    assert!(
        sites.iter().any(|s| s["site"]
            .as_str()
            .is_some_and(|x| x.starts_with("held.py:"))
            && s["chains"][0]["root"]["kind"] == "module-global"
            && s["chains"][0]["root"]["key"] == "KEPT"),
        "{doc}"
    );
    // captures.json lists it as a heap capture and prints the heap-graph command for it.
    let list = json(&d.hdir.join("captures.json"));
    let files = list["files"].as_array().unwrap();
    let entry = files
        .iter()
        .find(|f| {
            f["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("py-1.retain.json"))
        })
        .unwrap_or_else(|| panic!("{list}"));
    assert_eq!(entry["kind"], "heap", "{entry}");
    assert_eq!(entry["seq"], 1);
    let cmds: Vec<&str> = list["summarize"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        cmds.iter()
            .any(|c| c.starts_with("dla summarize heap-graph ") && c.ends_with("py-1.retain.json")),
        "{cmds:?}"
    );
    // The agent's own record lists the file too (the run lists only what the agent confirms).
    let out_records: Vec<Value> = walk(&d.hdir.join("out"))
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .flat_map(|p| {
            fs::read_to_string(p)
                .unwrap()
                .lines()
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .filter_map(|l| serde_json::from_str(&l).ok())
        .collect();
    assert!(
        out_records
            .iter()
            .flat_map(captures)
            .any(|c| c["kind"] == "heap" && c["file"] == "py-1.retain.json"),
        "{out_records:?}"
    );
    // `all` includes it for a Python harness.
    let d = driven(false);
    run_driven(&d, "all");
    retain_file(&d);
}

#[test]
fn the_agent_gets_raw_values_only_when_the_runs_consent_allows_them() {
    if !have_python() {
        return;
    }
    let without = driven(false);
    run_driven(&without, "heap");
    assert_eq!(
        fs::read_to_string(without.hdir.join("raw-env.txt")).unwrap(),
        "unset"
    );
    let text = fs::read_to_string(retain_file(&without)).unwrap();
    assert!(!text.contains("zz_marker_key"), "{text}");
    let with = driven(true);
    run_driven(&with, "heap");
    assert_eq!(
        fs::read_to_string(with.hdir.join("raw-env.txt")).unwrap(),
        "1"
    );
    let text = fs::read_to_string(retain_file(&with)).unwrap();
    assert!(text.contains("zz_marker_key"), "{text}");
    // A run that asks for no heap capture never passes the switch, whatever the consent.
    let none = driven(true);
    run_driven(&none, "histogram");
    assert_eq!(
        fs::read_to_string(none.hdir.join("raw-env.txt")).unwrap(),
        "unset"
    );
}

// ---------------------------------------------------------------------------------------
// the walk to the holder does not go through a long shared chain once per object

/// A module-level object that stores an exception and raises that same object again, 16 000
/// times, with six lines that keep a list per call: every kept object of every line is held through
/// the one traceback list of about 32 000 objects.
const RERAISE_SCRIPT: &str = "\
class Holder:
    def __init__(self, ex):
        self.ex = ex

    def __getattr__(self, name):
        raise self.ex


_h = Holder(ImportError('missing'))


def unit(i):
    a1 = [i]
    a2 = [i]
    a3 = [i]
    a4 = [i]
    a5 = [i]
    a6 = [i]
    try:
        _h.something
    except ImportError:
        pass
    return len(a1) + len(a2) + len(a3) + len(a4) + len(a5) + len(a6)


for n in range(16000):
    unit(n)
";

#[test]
fn a_chain_every_kept_object_runs_through_is_walked_once() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    fs::write(run.project.join("reraise_big.py"), RERAISE_SCRIPT).unwrap();
    run.python(&["reraise_big.py"], &[("DLA_CAPTURE", "heap")]);
    let doc = json(&run.pid_dir().join("py-1.retain.json"));
    // The capture's own time, as the agent reports it: the walk that went through the list again
    // for each object took the whole 60 s bound here, the walk that remembers its way takes a few
    // seconds. The bound leaves room for a slow machine and is far from the old time.
    let rec = run
        .records()
        .into_iter()
        .rev()
        .find(|r| !captures(r).is_empty())
        .unwrap();
    let heap = captures(&rec)
        .into_iter()
        .find(|c| c["kind"] == "heap")
        .unwrap();
    let ms = heap["ms"].as_u64().unwrap();
    assert!(ms < 20_000, "the heap capture took {ms} ms");
    // Every line that has kept objects has chains, and nothing was cut by the time bound.
    let sites = doc["sites"].as_array().unwrap();
    let lines: Vec<&Value> = sites
        .iter()
        .filter(|s| {
            s["site"]
                .as_str()
                .is_some_and(|x| x.starts_with("reraise_big.py:"))
                && s["kept"].as_u64().unwrap() >= 40
        })
        .collect();
    assert!(lines.len() >= 6, "{doc}");
    for s in &lines {
        let objects: u64 = s["chains"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["count"].as_u64().unwrap())
            .sum();
        assert_eq!(objects, s["kept"].as_u64().unwrap(), "{s}");
        assert!(
            s["chains"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["root"]["kind"] == "module-global"),
            "{s}"
        );
    }
    let text = doc["limitations"].to_string();
    assert!(!text.contains("time bound"), "{text}");
    // The run of tracebacks is one step with its length.
    let longest = lines
        .iter()
        .flat_map(|s| s["chains"].as_array().unwrap())
        .flat_map(|c| c["steps"].as_array().unwrap())
        .filter(|st| st["type"] == "traceback")
        .filter_map(|st| st["repeat"].as_u64())
        .max()
        .unwrap();
    assert!(longest > 10_000, "{longest}");
}

#[test]
fn a_step_is_named_like_the_growing_lines_in_the_copy_in_the_source_tree_and_never_by_an_absolute_path()
 {
    if !have_python() {
        return;
    }
    let d = driven(false);
    // A module of the project's source tree that the scratch copy does not hold (an editable
    // install runs from there): it keeps a function and a bound method of a standard library
    // object per call, in a module list.
    let src = d.ctx.project.join("lib");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("editable.py"),
        "import textwrap\n\nKEPT = []\n\n\ndef keep(i):\n    f = lambda: i\n    KEPT.append(f)\n    KEPT.append(textwrap.TextWrapper().fill)\n",
    )
    .unwrap();
    run_driven_with(
        &d,
        "heap",
        "import editable\n    _state['editable'] = editable",
        "_state['held'].keep(i)\n    _state['editable'].keep(i)",
        Some(&src),
    );
    let doc = json(&retain_file(&d));
    let steps: Vec<(String, String)> = doc["sites"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["chains"].as_array().unwrap())
        .flat_map(|c| c["steps"].as_array().unwrap())
        .filter_map(|st| {
            Some((
                st["type"].as_str()?.to_owned(),
                st["at"].as_str()?.to_owned(),
            ))
        })
        .collect();
    let sites: Vec<&str> = doc["sites"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["site"].as_str())
        .collect();
    assert!(
        sites.iter().any(|s| s.starts_with("lib/editable.py:")),
        "{sites:?}"
    );
    assert!(
        steps
            .iter()
            .any(|(t, at)| t == "function" && at.starts_with("lib/editable.py:")),
        "{steps:?}"
    );
    assert!(
        steps
            .iter()
            .any(|(t, at)| t == "method" && at.starts_with("<outside>/textwrap.py:")),
        "{steps:?}"
    );
    let text = doc.to_string();
    assert!(
        !text.contains(d.ctx.project.to_string_lossy().as_ref()) && !text.contains("\"/"),
        "an absolute path in the file: {text}"
    );
}

#[test]
fn a_run_of_tracebacks_lists_the_places_it_names_in_the_order_met() {
    if !have_python() {
        return;
    }
    let run = Run::new();
    // One stored exception raised through three different lines in turn: the run of tracebacks is
    // one step, and `at` is the first line only.
    fs::write(
        run.project.join("reraise_ats.py"),
        "class Holder:\n    def __init__(self, ex):\n        self.ex = ex\n\n    def __getattr__(self, name):\n        raise self.ex\n\n\n_h = Holder(ImportError('x'))\n\n\ndef first(i):\n    keep = [i]\n    try:\n        _h.a\n    except ImportError:\n        pass\n    return len(keep)\n\n\ndef second(i):\n    try:\n        _h.b\n    except ImportError:\n        pass\n\n\nfor n in range(300):\n    first(n)\n    second(n)\n",
    )
    .unwrap();
    run.python(&["reraise_ats.py"], &[("DLA_CAPTURE", "heap")]);
    let doc = json(&run.pid_dir().join("py-1.retain.json"));
    let steps: Vec<&Value> = doc["sites"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["chains"].as_array().unwrap())
        .flat_map(|c| c["steps"].as_array().unwrap())
        .filter(|st| st["type"] == "traceback" && st["repeat"].as_u64().unwrap_or(0) > 100)
        .collect();
    assert!(!steps.is_empty(), "{doc}");
    for st in steps {
        let ats: Vec<&str> = st["ats"]
            .as_array()
            .unwrap_or_else(|| panic!("{st}"))
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert!(ats.len() >= 2 && ats.len() <= 6, "{ats:?}");
        assert_eq!(ats[0], st["at"].as_str().unwrap(), "{st}");
        let mut distinct = ats.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), ats.len(), "{ats:?}");
        assert!(
            ats.iter().all(|a| a.starts_with("reraise_ats.py:")),
            "{ats:?}"
        );
    }
}

#[test]
fn a_walk_through_a_long_chain_of_a_project_run_from_its_source_tree_is_not_slowed_by_naming_places()
 {
    if !have_python() {
        return;
    }
    let run = Run::new();
    // The source tree (an editable install runs from there) and a file outside both trees.
    let src = run.dir.path().join("src");
    let outside = run.dir.path().join("outside");
    fs::create_dir_all(src.join("lib")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(
        outside.join("ext.py"),
        "def call(h):\n    h.ext_attribute\n",
    )
    .unwrap();
    fs::write(
        src.join("lib/reraise_src.py"),
        "import ext\n\n\nclass Holder:\n    def __init__(self, ex):\n        self.ex = ex\n\n    def __getattr__(self, name):\n        raise self.ex\n\n\n_h = Holder(ImportError('x'))\n\n\ndef unit(i):\n    a1 = [i]\n    a2 = [i]\n    a3 = [i]\n    a4 = [i]\n    a5 = [i]\n    a6 = [i]\n    for n in range(4):\n        try:\n            if n == 0:\n                _h.one\n            elif n == 1:\n                _h.two\n            elif n == 2:\n                _h.three\n            else:\n                ext.call(_h)\n        except ImportError:\n            pass\n    return len(a1) + len(a2) + len(a3) + len(a4) + len(a5) + len(a6)\n",
    )
    .unwrap();
    fs::write(
        run.project.join("run_src.py"),
        "import atexit, os, sys\n\ncalls = [0]\nreal = os.path.realpath\n\n\ndef counting(*a, **k):\n    calls[0] += 1\n    return real(*a, **k)\n\n\nos.path.realpath = counting\nfrom lib import reraise_src\n\nfor n in range(4000):\n    reraise_src.unit(n)\n# The agent's dump runs now, then the calls it made are written.\natexit._run_exitfuncs()\nopen(sys.argv[1], 'w').write(str(calls[0]))\n",
    )
    .unwrap();
    let path = format!(
        "{}:{}:{}:{}",
        run.agent.display(),
        run.project.display(),
        src.display(),
        outside.display()
    );
    let count_file = run.dir.path().join("realpath-calls.txt");
    run.python(
        &["run_src.py", count_file.to_str().unwrap()],
        &[
            ("DLA_CAPTURE", "heap"),
            ("DLA_SOURCE_ROOT", src.to_str().unwrap()),
            ("PYTHONPATH", &path),
        ],
    );
    let doc = json(&run.pid_dir().join("py-1.retain.json"));
    let rec = run
        .records()
        .into_iter()
        .rev()
        .find(|r| !captures(r).is_empty())
        .unwrap();
    let ms = captures(&rec)
        .into_iter()
        .find(|c| c["kind"] == "heap")
        .unwrap()["ms"]
        .as_u64()
        .unwrap();
    assert!(ms < 20_000, "the heap capture took {ms} ms");
    // A file name is resolved once, not once for every traceback of the run: the whole process,
    // the 16 000 raises included, makes a few thousand calls at most (an uncached walk makes more
    // than a hundred thousand, which on a deep path of a real source tree is the minute it took).
    let calls: u64 = fs::read_to_string(&count_file).unwrap().parse().unwrap();
    assert!(calls < 5_000, "{calls} calls of os.path.realpath");
    let lines: Vec<&Value> = doc["sites"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| {
            s["site"]
                .as_str()
                .is_some_and(|x| x.starts_with("lib/reraise_src.py:"))
                && s["kept"].as_u64().unwrap() >= 40
        })
        .collect();
    assert!(lines.len() >= 6, "{doc}");
    for s in &lines {
        assert!(!s["chains"].as_array().unwrap().is_empty(), "{s}");
    }
    assert!(!doc["limitations"].to_string().contains("time bound"));
    // The places of the run are named by the rule of the lines, and the file outside is no path.
    let text = doc.to_string();
    assert!(text.contains("<outside>/ext.py:"), "{text}");
    assert!(!text.contains(run.dir.path().to_str().unwrap()), "{text}");
    eprintln!("heap capture of the source-tree fixture: {ms} ms, {calls} calls");
}
