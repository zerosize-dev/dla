//! The Node agent's raw captures (`DLA_CAPTURE`): heap snapshots, sampling heap profiles and
//! resource creation sites, written by `harness/node/dla-preload.js`.
//!
//! These tests start `node` with the agent preloaded, the way `dla harness run` does, on copies
//! of the benchmark fixtures. Each test prints why it is skipped when `node` is missing.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use dla_workload::harness::capture::{self, select_points};
use serde_json::Value;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn have_node() -> bool {
    let ok = Command::new("node")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipped: node is not available");
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
}

impl Run {
    fn new(fixture: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("project");
        if fixture.is_empty() {
            fs::create_dir_all(&project).unwrap();
        } else {
            copy_dir(
                &repo().join("bench/fixtures").join(fixture).join("project"),
                &project,
            );
        }
        Self { dir, project }
    }

    fn out(&self) -> PathBuf {
        self.dir.path().join("out.jsonl")
    }

    fn cap(&self) -> PathBuf {
        self.dir.path().join("cap")
    }

    /// Runs `node <args>` in the project with the agent preloaded and `env` set.
    fn node(&self, args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
        let preload = repo().join("harness/node/dla-preload.js");
        let mut c = Command::new("node");
        c.args(args)
            .current_dir(&self.project)
            .env("NODE_OPTIONS", format!("--require {}", preload.display()))
            .env("DLA_HARNESS_OUT", self.out())
            .env("DLA_PROJECT", &self.project)
            .env("DLA_CAPTURE_DIR", self.cap())
            .env_remove("DLA_CAPTURE")
            .env_remove("DLA_WARMUP")
            .env_remove("DLA_ROUNDS")
            .env_remove("DLA_CAPTURE_DUMPS");
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

#[test]
fn command_mode_writes_every_kind_at_exit_and_lists_it() {
    if !have_node() {
        return;
    }
    let run = Run::new("node-socket-heartbeat");
    run.node(
        &["load.js", "--iterations", "300"],
        &[("DLA_CAPTURE", "heap,alloc,nonheap")],
    );
    let dir = run.pid_dir();
    assert_eq!(
        names(&dir),
        [
            "alloc-1.heapprofile",
            "heap-1.heapsnapshot",
            "resources.json",
            "scripts.json"
        ]
    );
    let recs = run.records();
    let rec = recs.iter().find(|r| r["phase"] == "exit").unwrap();
    let caps = captures(rec);
    assert_eq!(caps.len(), 4, "{caps:?}");
    for c in &caps {
        assert!(c.get("error").is_none(), "{c}");
        let file = c["file"].as_str().unwrap();
        assert_eq!(
            fs::metadata(dir.join(file)).unwrap().len(),
            c["bytes"].as_u64().unwrap(),
            "{file}"
        );
        // The harness classifies every file the agent names.
        assert!(capture::classify(file).is_some(), "{file}");
    }
    let kinds: Vec<&str> = caps.iter().map(|c| c["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["alloc", "heap", "nonheap", "nonheap"]);

    // The snapshot is v8's: meta first, strings last, locations present.
    let snap = json(&dir.join("heap-1.heapsnapshot"));
    assert!(snap["snapshot"]["meta"]["node_fields"].is_array());
    assert!(snap["snapshot"]["node_count"].as_u64().unwrap() > 1000);
    assert!(snap["strings"].as_array().unwrap().len() > 1000);
    // Script ids of the sidecar are the script ids of the snapshot's locations.
    let fields = snap["snapshot"]["meta"]["location_fields"]
        .as_array()
        .unwrap();
    let at = fields.iter().position(|f| f == "script_id").unwrap();
    let locs = snap["locations"].as_array().unwrap();
    let in_snapshot: std::collections::BTreeSet<u64> = locs
        .chunks(fields.len())
        .map(|c| c[at].as_u64().unwrap())
        .collect();
    let scripts = json(&dir.join("scripts.json"));
    let heartbeat = scripts["scripts"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, u)| u.as_str().unwrap().ends_with("/src/heartbeat.js"))
        .map(|(id, _)| id.parse::<u64>().unwrap())
        .expect("heartbeat.js is in scripts.json");
    assert!(in_snapshot.contains(&heartbeat));

    // The sampling profile holds the allocation site of the leak.
    let prof = fs::read_to_string(dir.join("alloc-1.heapprofile")).unwrap();
    let prof: Value = serde_json::from_str(&prof).unwrap();
    assert!(prof["head"]["children"].is_array());
    assert!(prof.to_string().contains("heartbeat.js"));

    // Resources: the leaked timers are counted at the setInterval site.
    let res = json(&dir.join("resources.json"));
    assert_eq!(res["dla_resources"], 1);
    let timers = &res["types"]["Timeout"];
    assert_eq!(timers["source"], "timers");
    assert_eq!(timers["sites"]["src/heartbeat.js:15"], 300, "{timers}");
    assert!(res.to_string().len() < 4096);
}

#[test]
fn the_cleared_control_has_no_live_timer_site() {
    if !have_node() {
        return;
    }
    let run = Run::new("node-socket-heartbeat-cleared");
    run.node(
        &["load.js", "--iterations", "300"],
        &[("DLA_CAPTURE", "nonheap")],
    );
    let res = json(&run.pid_dir().join("resources.json"));
    assert_eq!(res["types"]["Timeout"]["tracked"], 0, "{res}");
    assert_eq!(
        res["types"]["Timeout"]["sites"].as_object().unwrap().len(),
        0
    );
}

#[test]
fn nothing_is_written_without_capture_and_unknown_kinds_are_limitations() {
    if !have_node() {
        return;
    }
    let run = Run::new("node-socket-heartbeat-cleared");
    run.node(&["load.js", "--iterations", "20"], &[]);
    assert!(!run.cap().exists());
    assert!(captures(&run.records()[0]).is_empty());

    let run = Run::new("node-socket-heartbeat-cleared");
    run.node(
        &["load.js", "--iterations", "20"],
        &[("DLA_CAPTURE", "histogram,jfr")],
    );
    let rec = &run.records()[0];
    assert!(captures(rec).is_empty());
    let lim = rec["limitations"].to_string();
    assert!(
        lim.contains("histogram is a jvm capture") && lim.contains("jfr is a jvm capture"),
        "{lim}"
    );
}

#[test]
fn a_snapshot_too_big_for_the_memory_is_refused_with_the_figures() {
    if !have_node() {
        return;
    }
    let run = Run::new("node-socket-heartbeat-cleared");
    run.node(
        &["load.js", "--iterations", "50"],
        &[("DLA_CAPTURE", "heap"), ("DLA_CAPTURE_MEM_BYTES", "1000")],
    );
    let rec = &run.records()[0];
    let caps = captures(rec);
    assert_eq!(caps.len(), 1);
    let err = caps[0]["error"].as_str().unwrap();
    assert!(
        err.starts_with("preflight:") && err.contains("12x heapUsed"),
        "{err}"
    );
    assert!(
        !run.pid_dir().join("heap-1.heapsnapshot").exists(),
        "no file for a refused snapshot"
    );
}

/// A driver that calls `checkpoint` after each of `total` rounds, keeping `held` alive.
const DRIVER: &str = r#"
const dla = require(process.env.DLA_AGENT);
const held = [];
const total = Number(process.env.TOTAL);
(async () => {
  for (let round = 1; round <= total; round += 1) {
    for (let i = 0; i < 200; i += 1) held.push({ round, i, pad: 'x'.repeat(64) });
    await new Promise((resolve) => setImmediate(resolve));
    dla.checkpoint(round * 200, round === total);
  }
})();
"#;

#[test]
fn driver_dumps_are_placed_like_the_rust_selection_and_come_from_one_process() {
    if !have_node() {
        return;
    }
    let agent = repo().join("harness/node/dla-preload.js");
    for (warmup, rounds, dumps) in [(1u32, 8u32, 4u32), (2, 7, 3), (1, 3, 4), (3, 12, 5)] {
        let run = Run::new("");
        fs::write(run.project.join("driver.js"), DRIVER).unwrap();
        let total = (warmup + rounds).to_string();
        run.node(
            &["driver.js"],
            &[
                ("DLA_CAPTURE", "heap,alloc,nonheap"),
                ("DLA_AGENT", agent.to_str().unwrap()),
                ("TOTAL", &total),
                ("DLA_WARMUP", &warmup.to_string()),
                ("DLA_ROUNDS", &rounds.to_string()),
                ("DLA_CAPTURE_DUMPS", &dumps.to_string()),
            ],
        );
        let want = select_points(warmup + rounds, warmup, dumps);
        let recs = run.records();
        let with_heap: Vec<u32> = recs
            .iter()
            .enumerate()
            .filter(|(_, r)| captures(r).iter().any(|c| c["kind"] == "heap"))
            .map(|(i, _)| i as u32 + 1)
            .collect();
        assert_eq!(
            with_heap,
            want.iter().copied().collect::<Vec<_>>(),
            "warmup {warmup} rounds {rounds} dumps {dumps}"
        );
        let dir = run.pid_dir();
        let snaps = names(&dir)
            .into_iter()
            .filter(|n| n.ends_with(".heapsnapshot"))
            .count();
        assert_eq!(snaps, want.len());
        // The resources are written once, at the last round.
        assert!(
            captures(recs.last().unwrap())
                .iter()
                .any(|c| c["file"] == "resources.json")
        );
        assert!(dir.join("resources.json").is_file());
        // Each heavy checkpoint also closes a profile window; the last one is the final.
        let profiles = names(&dir)
            .into_iter()
            .filter(|n| n.ends_with(".heapprofile"))
            .count();
        assert_eq!(profiles, want.len());
        // Snapshots grow with the held objects: they are successive dumps of one process.
        if want.len() >= 2 {
            let size = |n: usize| {
                fs::metadata(dir.join(format!("heap-{n}.heapsnapshot")))
                    .unwrap()
                    .len()
            };
            assert!(size(want.len()) > size(1), "{warmup} {rounds} {dumps}");
        }
    }
}
