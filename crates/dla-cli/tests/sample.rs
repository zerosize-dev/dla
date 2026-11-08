//! End-to-end tests of `dla sample` against real processes: a plain sleeper, a JVM, a Go
//! program with `net/http/pprof`, Node.js with and without an inspector, and a process tree.
//! Tests skip (with a note on stderr) when the runtime they need is not installed.
#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{BufRead, BufReader},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command as Proc, Stdio},
    time::{Duration, Instant},
};

use assert_cmd::Command;
use serde_json::{Value, json};

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn run(project: &Path, args: &[&str]) -> (i32, Value) {
    let out = dla(project).args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), v)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_ne!(code, 0, "{args:?}: {v}");
    let e = &v["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "no {k}: {v}");
    }
    e.clone()
}

struct Project {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl Project {
    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn sample(&self, pid: u32, extra: &[&str]) -> Value {
        let pid = pid.to_string();
        let mut args = vec!["sample", "--pid", &pid];
        args.extend_from_slice(extra);
        ok(self.path(), &args)
    }

    fn series(&self, id: &str) -> Value {
        let text = fs::read_to_string(self.run_dir.join(format!("series/{id}.json"))).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn ledger(&self) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn coverage(&self) -> Value {
        let text = fs::read_to_string(self.run_dir.join("coverage.json")).unwrap();
        serde_json::from_str(&text).unwrap()
    }
}

fn project_with(tiers: Option<&str>) -> Project {
    let dir = tempfile::tempdir().unwrap();
    let init = ok(dir.path(), &["run", "init", "--session-id", "t"]);
    let run_dir = dir
        .path()
        .join(".dla")
        .join(init["run_id"].as_str().unwrap());
    if let Some(t) = tiers {
        ok(dir.path(), &["consent", "--tiers", t]);
    }
    Project { dir, run_dir }
}

fn project() -> Project {
    project_with(Some("1-4"))
}

/// A child that is killed and reaped when dropped.
struct Guard(Child);

impl Guard {
    fn pid(&self) -> u32 {
        self.0.id()
    }

    fn alive(&mut self) -> bool {
        self.0.try_wait().unwrap().is_none()
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Starts `cmd` and waits for it to print a line containing `ready` on stdout.
fn start_ready(mut cmd: Proc, ready: &str) -> Guard {
    cmd.stdout(Stdio::piped()).stderr(Stdio::null());
    let mut child = cmd.spawn().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    let deadline = Instant::now() + Duration::from_secs(60);
    while !line.contains(ready) {
        line.clear();
        assert!(Instant::now() < deadline, "never saw {ready:?}");
        let n = reader.read_line(&mut line).unwrap();
        assert!(n > 0, "process ended before printing {ready:?}");
    }
    // Keep draining so the child never blocks on a full pipe.
    std::thread::spawn(move || {
        let mut sink = String::new();
        while reader.read_line(&mut sink).is_ok_and(|n| n > 0) {
            sink.clear();
        }
    });
    Guard(child)
}

fn have(tool: &str, flag: &str) -> bool {
    Proc::new(tool)
        .arg(flag)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn columns(series: &Value) -> Vec<String> {
    series["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_owned())
        .collect()
}

/// The non-missing values of column `name`.
fn values(series: &Value, name: &str) -> Vec<f64> {
    let i = columns(series)
        .iter()
        .position(|c| c == name)
        .unwrap_or_else(|| {
            panic!("no column {name} in {:?}", columns(series));
        });
    series["points"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["values"][i].as_f64())
        .collect()
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn sleeper() -> Guard {
    Guard(Proc::new("sleep").arg("120").spawn().unwrap())
}

#[test]
fn samples_a_plain_process_into_a_series_with_evidence() {
    let p = project();
    let s = sleeper();
    let v = p.sample(
        s.pid(),
        &[
            "--duration",
            "2s",
            "--interval",
            "100ms",
            "--counters",
            "none",
        ],
    );
    assert_eq!(v["series"], "S1");
    assert_eq!(v["next"], "dla trend S1");
    assert!(v["ticks"].as_u64().unwrap() >= 10, "{v}");
    assert_eq!(v["backends"], json!({}));
    for k in [
        "target",
        "expected_signal",
        "stopping",
        "effects",
        "cleanup",
    ] {
        assert!(
            v["effects"][k].as_str().is_some_and(|s| !s.is_empty()),
            "{k}: {v}"
        );
    }
    assert!(v["effects"]["target"].as_str().unwrap().contains("tier 2"));
    assert!(
        v["effects"]["effects"]
            .as_str()
            .unwrap()
            .contains("read-only")
    );

    let series = p.series("S1");
    assert_eq!(series["source"]["kind"], "sample");
    assert_eq!(series["source"]["pid"], s.pid());
    // No raw-value consent: the base name and the argument count only.
    assert_eq!(series["source"]["command"], "sleep (1 argument)");
    assert_eq!(series["axis"], "seconds");
    assert_eq!(series["warmup"], 0);
    let cols = columns(&series);
    assert_eq!(cols[0], "os.rss");
    assert!(cols.contains(&"os.threads".to_owned()), "{cols:?}");
    assert!(values(&series, "os.rss").iter().all(|v| *v > 0.0));
    let rss = &series["columns"][0];
    assert_eq!(
        (rss["unit"].as_str(), rss["kind"].as_str()),
        (Some("bytes"), Some("gauge"))
    );
    assert_eq!(rss["baseline"], true);
    assert_eq!(rss["after_gc"], false);
    let pts = series["points"].as_array().unwrap();
    assert!(
        pts.windows(2)
            .all(|w| w[0]["x"].as_f64() <= w[1]["x"].as_f64())
    );

    // One growth-series evidence: live, about this process, no gate field.
    let ev: Vec<Value> = p
        .ledger()
        .into_iter()
        .filter(|l| l["kind"] == "growth-series")
        .collect();
    assert_eq!(ev.len(), 1);
    let ev = &ev[0];
    assert_eq!(v["evidence"], ev["evidence_id"]);
    assert_eq!(ev["authority"], "live");
    assert_eq!(ev["source"]["process"]["pid"], s.pid());
    assert_eq!(ev["raw"], "series/S1.json");
    let r = &ev["result"];
    assert_eq!(r["series"], "S1");
    assert_eq!(r["pid"], s.pid());
    assert_eq!(r["ticks"], v["ticks"]);
    assert!(r["duration_s"].as_f64().unwrap() >= 1.5);
    assert!(r.get("gate").is_none() && r.get("gates").is_none());
    assert!(r["columns"].as_array().unwrap().len() <= 24);
    assert!(r["limitations"].is_number());

    // A second sample gets the next ID.
    let v2 = p.sample(s.pid(), &["--duration", "300ms", "--counters", "none"]);
    assert_eq!(v2["series"], "S2");
}

#[test]
fn the_full_command_line_needs_raw_value_consent() {
    // `$0` carries the secret into the command line; the `; true` keeps the shell from exec'ing.
    let secret_holder = || {
        Guard(
            Proc::new("sh")
                .args(["-c", "sleep 120; true", "--token=s3cr3t-value"])
                .spawn()
                .unwrap(),
        )
    };
    let dump = |p: &Project, v: &Value| -> String {
        let series = fs::read_to_string(p.run_dir.join("series/S1.json")).unwrap();
        format!("{v}{series}{}", p.ledger().len())
    };
    let quick = ["--duration", "300ms", "--counters", "none"];

    let p = project();
    let s = secret_holder();
    let v = p.sample(s.pid(), &quick);
    assert!(!dump(&p, &v).contains("s3cr3t"), "{v}");
    assert_eq!(p.series("S1")["source"]["command"], "sh (3 arguments)");
    assert!(
        v["effects"]["target"]
            .as_str()
            .unwrap()
            .contains("sh (3 arguments)")
    );
    let ledger = fs::read_to_string(p.run_dir.join("ledger.jsonl")).unwrap();
    assert!(!ledger.contains("s3cr3t"));

    // With raw-value consent the full line is kept.
    let p = project_with(None);
    ok(p.path(), &["consent", "--tiers", "1-4", "--raw-values"]);
    let s = secret_holder();
    let v = p.sample(s.pid(), &quick);
    let command = p.series("S1")["source"]["command"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(command.contains("--token=s3cr3t-value"), "{command}");
    assert!(
        v["effects"]["target"]
            .as_str()
            .unwrap()
            .contains("sleep 120")
    );
}

#[test]
fn stops_when_the_process_exits() {
    let p = project();
    let mut c = Proc::new("sleep").arg("1").spawn().unwrap();
    let pid = c.id();
    let waiter = std::thread::spawn(move || c.wait());
    let t = Instant::now();
    let v = p.sample(
        pid,
        &[
            "--duration",
            "30s",
            "--interval",
            "100ms",
            "--counters",
            "none",
        ],
    );
    waiter.join().unwrap().unwrap();
    assert!(t.elapsed() < Duration::from_secs(15));
    assert_eq!(v["exited"], true);
}

#[test]
fn refusals_name_the_missing_item_and_the_fix() {
    let p = project();
    let e = refused(p.path(), &["sample"]);
    assert_eq!(e["code"], "missing-argument");
    assert!(e["missing"].as_str().unwrap().contains("--pid"));
    let s = sleeper();
    let pid = s.pid().to_string();
    for (flag, val) in [
        ("--interval", "200"),
        ("--duration", "0s"),
        ("--interval", "1ms"),
    ] {
        let e = refused(p.path(), &["sample", "--pid", &pid, flag, val]);
        assert_eq!(e["code"], "invalid-value", "{flag} {val}");
    }
    let e = refused(p.path(), &["sample", "--pid", &pid, "--counters", "perf"]);
    assert_eq!(e["code"], "invalid-value");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("unknown counters backend")
    );
    let e = refused(p.path(), &["sample", "--pid", "4000000"]);
    assert!(e["message"].as_str().unwrap().contains("not running"));
    // No consent at all.
    let bare = project_with(None);
    let e = refused(
        bare.path(),
        &["sample", "--pid", &pid, "--counters", "none"],
    );
    assert_eq!(e["code"], "consent-required");
    assert!(e["fix"].as_str().unwrap().contains("dla consent --tiers"));
    assert!(e["message"].as_str().unwrap().contains("tier 2"));
}

#[test]
fn a_process_dla_started_is_tier_one_and_others_are_tier_two() {
    let p = project_with(Some("1"));
    let s = sleeper();
    let pid = s.pid().to_string();
    // Not registered: tier 2, which this run's consent does not cover.
    let e = refused(p.path(), &["sample", "--pid", &pid, "--counters", "none"]);
    assert_eq!(e["code"], "consent-required");
    assert!(e["message"].as_str().unwrap().contains("tier 2"), "{e}");
    assert!(e["fix"].as_str().unwrap().contains("dla run init"));
    // Registered with its real start time: tier 1.
    let start = process_start_time(s.pid());
    let state_path = p.run_dir.join("state.json");
    let mut state: Value = serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    state["processes"] = json!([{
        "pid": s.pid(), "start_time": start, "kind": "harness", "command": "sleep 120",
        "started_at": "2026-01-01T00:00:00Z"
    }]);
    fs::write(&state_path, state.to_string()).unwrap();
    let v = p.sample(s.pid(), &["--duration", "300ms", "--counters", "none"]);
    assert!(
        v["effects"]["target"].as_str().unwrap().contains("tier 1"),
        "{v}"
    );
    // The same pid with a different start time is a different process: tier 2 again.
    state["processes"][0]["start_time"] = json!("1999-01-01T00:00:00Z");
    fs::write(&state_path, state.to_string()).unwrap();
    let e = refused(p.path(), &["sample", "--pid", &pid, "--counters", "none"]);
    assert!(e["message"].as_str().unwrap().contains("tier 2"));
}

fn process_start_time(pid: u32) -> String {
    // The sampler's own start time, read the same way `dla` reads it.
    let boot = fs::read_to_string("/proc/stat")
        .unwrap()
        .lines()
        .find_map(|l| {
            l.strip_prefix("btime ")
                .map(|b| b.trim().parse::<i64>().unwrap())
        })
        .unwrap();
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
    let after = &stat[stat.rfind(')').unwrap() + 2..];
    let ticks: i64 = after.split_whitespace().nth(19).unwrap().parse().unwrap();
    let secs = boot + ticks / 100;
    chrono::DateTime::from_timestamp(secs, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

#[test]
fn tree_mode_sums_descendants_and_records_the_largest() {
    let p = project();
    let sh = start_ready(
        {
            let mut c = Proc::new("sh");
            c.args(["-c", "sleep 120 & sleep 120 & echo ready; wait"]);
            c
        },
        "ready",
    );
    let v = p.sample(
        sh.pid(),
        &["--tree", "--duration", "600ms", "--counters", "none"],
    );
    let ev = p
        .ledger()
        .into_iter()
        .find(|l| l["kind"] == "growth-series")
        .unwrap();
    assert!(
        ev["result"]["largest_descendant"]["pid"].as_u64().unwrap() > 0,
        "{v}"
    );
    let tree_rss = values(&p.series("S1"), "os.rss")[0];
    let alone = project();
    let v2 = alone.sample(sh.pid(), &["--duration", "300ms", "--counters", "none"]);
    assert_eq!(v2["series"], "S1");
    assert!(tree_rss > values(&alone.series("S1"), "os.rss")[0]);
}

#[test]
fn tokio_and_python_remote_are_recorded_limitations() {
    let p = project();
    let s = sleeper();
    let v = p.sample(
        s.pid(),
        &["--duration", "300ms", "--counters", "tokio,python-remote"],
    );
    assert_eq!(v["backends"]["tokio"], "blocked");
    let series = p.series("S1");
    let lims: Vec<&str> = series["limitations"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        lims.contains(&"tokio metrics are read in-process by the Rust driver harness"),
        "{lims:?}"
    );
    let cov = p.coverage();
    assert_eq!(
        cov["workers"]["rust"]["backends"]["tokio"]["status"],
        "blocked"
    );
    let py_new = have("python3.14", "--version");
    if !py_new && !python_at_least_314() {
        assert_eq!(v["backends"]["python-remote"], "missing");
        assert!(
            lims.contains(&"python remote exec needs Python 3.14+"),
            "{lims:?}"
        );
        assert_eq!(
            cov["workers"]["python"]["backends"]["python-remote"]["status"],
            "missing"
        );
        assert!(cov["workers"]["python"]["backends"]["python-remote"]["install_hint"].is_string());
    }
}

fn python_at_least_314() -> bool {
    let out = Proc::new("python3")
        .args(["-c", "import sys; print(int(sys.version_info >= (3, 14)))"])
        .output();
    out.is_ok_and(|o| String::from_utf8_lossy(&o.stdout).trim() == "1")
}

// ---- JVM ----

const LEAK_JAVA: &str = r#"
import java.util.*;
public class Leak {
    static final List<byte[]> KEEP = new ArrayList<>();
    public static void main(String[] a) throws Exception {
        System.out.println("ready");
        while (true) {
            KEEP.add(new byte[8192]);
            Thread.sleep(2);
        }
    }
}
"#;

fn compile_leak(dir: &Path) -> bool {
    if !have("javac", "-version") || !have("java", "-version") || !have("jcmd", "-h") {
        eprintln!("skipping: no JDK with javac, java and jcmd");
        return false;
    }
    fs::write(dir.join("Leak.java"), LEAK_JAVA).unwrap();
    let out = Proc::new("javac")
        .arg("Leak.java")
        .current_dir(dir)
        .env_remove("JAVA_TOOL_OPTIONS")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    true
}

fn java(dir: &Path, flags: &[&str]) -> Guard {
    let mut c = Proc::new("java");
    c.args(flags)
        .args(["-cp", "."])
        .arg("Leak")
        .current_dir(dir);
    start_ready(c, "ready")
}

#[test]
fn jcmd_backend_reads_heap_threads_and_nmt() {
    let work = tempfile::tempdir().unwrap();
    if !compile_leak(work.path()) {
        return;
    }
    let p = project();
    let jvm = java(
        work.path(),
        &["-XX:NativeMemoryTracking=summary", "-Xmx256m"],
    );
    let v = p.sample(jvm.pid(), &["--duration", "6s", "--interval", "200ms"]);
    assert_eq!(v["backends"]["jcmd"], "ran", "{v}");
    let series = p.series("S1");
    assert_eq!(series["source"]["runtime"], "jvm");
    for c in [
        "jvm.heap_used",
        "jvm.heap_committed",
        "jvm.threads",
        "jvm.nmt_committed",
        "os.rss",
    ] {
        let vals = values(&series, c);
        assert!(!vals.is_empty(), "{c}");
        assert!(vals.iter().all(|v| *v > 0.0), "{c}: {vals:?}");
    }
    assert!(values(&series, "jvm.heap_used").len() >= 2);
    let meta = |name: &str| {
        series["columns"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap()
            .clone()
    };
    assert_eq!(meta("jvm.threads")["kind"], "counter");
    assert_eq!(meta("jvm.heap_used")["after_gc"], false);
    assert_eq!(meta("jvm.heap_used")["source"], "jcmd");
    assert_eq!(
        p.coverage()["workers"]["jvm"]["backends"]["jcmd"]["status"],
        "ran"
    );
}

#[test]
fn jcmd_without_native_memory_tracking_is_a_limitation() {
    let work = tempfile::tempdir().unwrap();
    if !compile_leak(work.path()) {
        return;
    }
    let p = project();
    // The environment may enable NMT through JAVA_TOOL_OPTIONS; the later flag wins.
    let jvm = java(work.path(), &["-XX:NativeMemoryTracking=off"]);
    let v = p.sample(jvm.pid(), &["--duration", "4s", "--counters", "jcmd"]);
    assert_eq!(v["backends"]["jcmd"], "ran", "{v}");
    let series = p.series("S1");
    assert!(!columns(&series).contains(&"jvm.nmt_committed".to_owned()));
    assert!(columns(&series).contains(&"jvm.heap_used".to_owned()));
    let lims = series["limitations"].to_string();
    assert!(lims.contains("native memory tracking is off"), "{lims}");
}

#[test]
fn auto_picks_jcmd_for_a_jvm() {
    let work = tempfile::tempdir().unwrap();
    if !compile_leak(work.path()) {
        return;
    }
    let p = project();
    let jvm = java(work.path(), &[]);
    let v = p.sample(jvm.pid(), &["--duration", "3s"]);
    assert_eq!(v["backends"]["jcmd"], "ran", "{v}");
}

// ---- Go ----

const GO_SRC: &str = r#"package main

import (
	"fmt"
	"net/http"
	_ "net/http/pprof"
	"os"
	"time"
)

var sink [][]byte

func worker() {
	for {
		time.Sleep(time.Hour)
	}
}

func main() {
	go func() {
		for {
			sink = append(sink, make([]byte, 4096))
			time.Sleep(2 * time.Millisecond)
		}
	}()
	go func() {
		for {
			go worker() // goroutine leak site
			time.Sleep(20 * time.Millisecond)
		}
	}()
	fmt.Println("ready")
	if err := http.ListenAndServe("127.0.0.1:"+os.Args[1], nil); err != nil {
		fmt.Println(err)
	}
}
"#;

#[test]
fn go_pprof_backend_reads_memstats_and_goroutine_sites() {
    if !have("go", "version") {
        eprintln!("skipping: go is not installed");
        return;
    }
    let work = tempfile::tempdir().unwrap();
    fs::write(work.path().join("main.go"), GO_SRC).unwrap();
    fs::write(work.path().join("go.mod"), "module leak\n\ngo 1.21\n").unwrap();
    let build = Proc::new("go")
        .args(["build", "-o", "leak", "."])
        .current_dir(work.path())
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let port = free_port();
    let mut c = Proc::new(work.path().join("leak"));
    c.arg(port.to_string()).current_dir(work.path());
    let go = start_ready(c, "ready");
    std::thread::sleep(Duration::from_millis(300));

    let p = project();
    let url = format!("go-pprof=http://127.0.0.1:{port}");
    let v = p.sample(
        go.pid(),
        &[
            "--duration",
            "3s",
            "--interval",
            "200ms",
            "--counters",
            &url,
        ],
    );
    assert_eq!(v["backends"]["go-pprof"], "ran", "{v}");
    let series = p.series("S1");
    assert_eq!(series["source"]["runtime"], "go");
    for c in [
        "go.heap_live",
        "go.total",
        "go.released",
        "go.stacks",
        "go.goroutines",
    ] {
        let vals = values(&series, c);
        assert!(vals.len() >= 3, "{c}: {vals:?}");
    }
    let heap = values(&series, "go.heap_live");
    assert!(heap.last().unwrap() > heap.first().unwrap(), "{heap:?}");
    // The goroutines started at main.go's `go worker()` line, grouped by site, grow.
    let site = columns(&series)
        .into_iter()
        .filter(|c| c.starts_with("go.goroutines@main.go:"))
        .max_by(|a, b| {
            let last = |c: &str| *values(&series, c).last().unwrap();
            last(a).total_cmp(&last(b))
        })
        .unwrap_or_else(|| panic!("no goroutine site column in {:?}", columns(&series)));
    let g = values(&series, &site);
    assert!(g.last().unwrap() > g.first().unwrap(), "{site}: {g:?}");
    let col = series["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == site.as_str())
        .unwrap();
    assert_eq!(col["kind"], "counter");
    assert!(
        col["site"].as_str().unwrap().starts_with("main.go:"),
        "{col}"
    );
    assert_eq!(
        p.coverage()["workers"]["go"]["backends"]["go-pprof"]["status"],
        "ran"
    );
    assert!(go.pid() > 0);
}

#[test]
fn go_pprof_on_a_dead_address_is_blocked_and_sampling_continues() {
    let p = project();
    let s = sleeper();
    let url = format!("go-pprof=http://127.0.0.1:{}", free_port());
    let v = p.sample(s.pid(), &["--duration", "500ms", "--counters", &url]);
    assert_eq!(v["backends"]["go-pprof"], "blocked");
    assert!(v["ticks"].as_u64().unwrap() >= 2);
    let cov = p.coverage();
    assert_eq!(
        cov["workers"]["go"]["backends"]["go-pprof"]["status"],
        "blocked"
    );
    assert!(
        cov["workers"]["go"]["backends"]["go-pprof"]["reason"]
            .as_str()
            .unwrap()
            .contains("pprof")
    );
    let e = refused(
        p.path(),
        &[
            "sample",
            "--pid",
            &s.pid().to_string(),
            "--counters",
            "go-pprof",
        ],
    );
    assert_eq!(e["code"], "invalid-value");
}

// ---- Node.js ----

const NODE_JS: &str = r#"
const timers = [];
setInterval(() => { timers.push(setTimeout(() => {}, 3600 * 1000)); }, 20);
const big = [];
setInterval(() => { big.push(Buffer.alloc(16384)); }, 10);
console.log('ready');
"#;

fn node(dir: &Path, flags: &[&str]) -> Guard {
    fs::write(dir.join("app.js"), NODE_JS).unwrap();
    let mut c = Proc::new("node");
    c.args(flags).arg("app.js").current_dir(dir);
    start_ready(c, "ready")
}

#[test]
fn node_inspect_backend_reads_memory_and_active_resources() {
    if !have("node", "--version") {
        eprintln!("skipping: node is not installed");
        return;
    }
    let work = tempfile::tempdir().unwrap();
    let n = node(work.path(), &["--inspect=0"]);
    let p = project();
    // `--inspect=0` picks a random port: the backend finds it from the listening sockets.
    let v = p.sample(n.pid(), &["--duration", "3s", "--interval", "200ms"]);
    assert_eq!(v["backends"]["node-inspect"], "ran", "{v}");
    let series = p.series("S1");
    assert_eq!(series["source"]["runtime"], "node");
    for c in [
        "node.heap_used",
        "node.heap_total",
        "node.external",
        "node.array_buffers",
    ] {
        let vals = values(&series, c);
        assert!(vals.len() >= 5, "{c}: {vals:?}");
    }
    let ab = values(&series, "node.array_buffers");
    assert!(ab.last().unwrap() > ab.first().unwrap(), "{ab:?}");
    let t = values(&series, "node.active.Timeout");
    assert!(t.last().unwrap() > t.first().unwrap(), "{t:?}");
    let col = series["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "node.active.Timeout")
        .unwrap();
    assert_eq!(col["kind"], "counter");
    assert_eq!(
        p.coverage()["workers"]["node"]["backends"]["node-inspect"]["status"],
        "ran"
    );
}

#[test]
fn node_inspect_port_can_be_named() {
    if !have("node", "--version") {
        eprintln!("skipping: node is not installed");
        return;
    }
    let work = tempfile::tempdir().unwrap();
    let port = free_port();
    let n = node(work.path(), &[&format!("--inspect=127.0.0.1:{port}")]);
    let p = project();
    let arg = format!("node-inspect={port}");
    let v = p.sample(n.pid(), &["--duration", "1s", "--counters", &arg]);
    assert_eq!(v["backends"]["node-inspect"], "ran", "{v}");
}

#[test]
fn auto_never_signals_a_node_without_an_inspector() {
    if !have("node", "--version") {
        eprintln!("skipping: node is not installed");
        return;
    }
    let work = tempfile::tempdir().unwrap();
    let mut n = node(work.path(), &[]);
    let p = project();
    let v = p.sample(n.pid(), &["--duration", "800ms"]);
    assert_eq!(v["backends"], json!({}), "{v}");
    assert!(n.alive(), "auto must not signal node");
    assert!(
        dla_listening(n.pid()).is_empty(),
        "no inspector should have been opened"
    );
}

fn dla_listening(pid: u32) -> Vec<u16> {
    // Port 9229 or any listener of the process.
    let inodes: Vec<String> = fs::read_dir(format!("/proc/{pid}/fd"))
        .map(|d| {
            d.filter_map(Result::ok)
                .filter_map(|e| fs::read_link(e.path()).ok())
                .map(|t| t.to_string_lossy().into_owned())
                .filter(|t| t.starts_with("socket:["))
                .collect()
        })
        .unwrap_or_default();
    let mut ports = Vec::new();
    for table in ["tcp", "tcp6"] {
        let text = fs::read_to_string(format!("/proc/{pid}/net/{table}")).unwrap_or_default();
        for line in text.lines().skip(1) {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.get(3) == Some(&"0A") && inodes.iter().any(|i| *i == format!("socket:[{}]", f[9]))
            {
                ports.push(u16::from_str_radix(f[1].rsplit(':').next().unwrap(), 16).unwrap());
            }
        }
    }
    ports
}

#[test]
fn explicit_node_inspect_signals_a_node_without_an_inspector() {
    if !have("node", "--version") {
        eprintln!("skipping: node is not installed");
        return;
    }
    if TcpListener::bind("127.0.0.1:9229").is_err() {
        eprintln!("skipping: port 9229 is taken");
        return;
    }
    let work = tempfile::tempdir().unwrap();
    let n = node(work.path(), &[]);
    let p = project();
    // Opening the inspector after SIGUSR1 counts against --duration; under a loaded test
    // run it can take a few seconds, so leave room for at least one reading.
    let v = p.sample(n.pid(), &["--duration", "8s", "--counters", "node-inspect"]);
    assert_eq!(v["backends"]["node-inspect"], "ran", "{v}");
    assert!(
        v["effects"]["effects"]
            .as_str()
            .unwrap()
            .contains("SIGUSR1")
    );
    assert!(
        v["effects"]["cleanup"]
            .as_str()
            .unwrap()
            .contains("inspector")
    );
    assert!(!values(&p.series("S1"), "node.heap_used").is_empty());
}

#[test]
fn signalling_is_refused_for_a_process_that_is_not_node() {
    let p = project();
    let mut s = sleeper();
    let v = p.sample(
        s.pid(),
        &["--duration", "500ms", "--counters", "node-inspect"],
    );
    assert_eq!(v["backends"]["node-inspect"], "blocked", "{v}");
    assert!(s.alive(), "a non-node process must never receive SIGUSR1");
}
