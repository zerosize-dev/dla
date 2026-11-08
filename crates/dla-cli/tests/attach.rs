//! End-to-end tests of `dla attach`: refusals (consent tier, resources, target kind), a fake
//! `jcmd` for the registry/evidence flow, and real targets (a JVM, Node.js, a C program for
//! heaptrack, Python with memray). A test whose tool is missing prints why and passes.
#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command as Proc, Stdio},
    time::{Duration, Instant},
};

use assert_cmd::Command;
use serde_json::Value;

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "DLA_ATTACH_FREE_DISK",
        "DLA_ATTACH_AVAILABLE_MEMORY",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn run_with(project: &Path, env: &[(&str, &str)], args: &[&str]) -> (i32, Value) {
    let mut c = dla(project);
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), v)
}

fn ok(project: &Path, env: &[(&str, &str)], args: &[&str]) -> Value {
    let (code, v) = run_with(project, env, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

fn refused(project: &Path, env: &[(&str, &str)], args: &[&str]) -> Value {
    let (code, v) = run_with(project, env, args);
    assert_eq!(code, 2, "{args:?}: {v}");
    let e = &v["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "no {k}: {v}");
    }
    e.clone()
}

struct Project {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
    /// `$DLA_DATA_DIR`, with `tools/` for fake tools.
    data: tempfile::TempDir,
}

impl Project {
    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn attach(&self, extra_env: &[(&str, &str)], args: &[&str]) -> (i32, Value) {
        let data = self.data.path().display().to_string();
        let mut env = vec![("DLA_DATA_DIR", data.as_str())];
        env.extend_from_slice(extra_env);
        let mut a = vec!["attach"];
        a.extend_from_slice(args);
        run_with(self.path(), &env, &a)
    }

    fn attach_ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.attach(&[], args);
        assert_eq!(code, 0, "{args:?}: {v}");
        assert_eq!(v["ok"], true, "{v}");
        v
    }

    fn tool(&self, name: &str, body: &str) {
        let dir = self.data.path().join("tools");
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn ledger(&self) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn state(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir.join("state.json")).unwrap()).unwrap()
    }

    fn evidence(&self, id: &str) -> Value {
        self.ledger()
            .into_iter()
            .find(|l| l["evidence_id"] == id)
            .unwrap_or_else(|| panic!("no evidence {id}"))
    }
}

fn project_with(tiers: Option<&str>) -> Project {
    let dir = tempfile::tempdir().unwrap();
    let init = ok(dir.path(), &[], &["run", "init", "--session-id", "t"]);
    let run_dir = dir
        .path()
        .join(".dla")
        .join(init["run_id"].as_str().unwrap());
    if let Some(t) = tiers {
        ok(dir.path(), &[], &["consent", "--tiers", t]);
    }
    Project {
        dir,
        run_dir,
        data: tempfile::tempdir().unwrap(),
    }
}

fn project() -> Project {
    project_with(Some("1-4"))
}

/// A child that is killed and reaped when dropped.
struct Guard(Child, #[allow(dead_code)] Option<tempfile::TempDir>);

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

fn sleeper() -> Guard {
    Guard(
        Proc::new("sleep")
            .arg("300")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
        None,
    )
}

/// A `sleep` process whose executable is called `java`, standing in for a JVM that a fake
/// `jcmd` answers for.
fn java_sleeper() -> Guard {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("java");
    let sleep = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|d| d.join("sleep"))
        .find(|p| p.is_file())
        .expect("sleep on PATH");
    fs::copy(sleep, &exe).unwrap();
    // A fork in another test thread can still hold the copy's write handle for a moment,
    // and exec then fails with ETXTBSY ("text file busy"): retry briefly.
    let mut tries = 0;
    let child = loop {
        match Proc::new(&exe)
            .arg("300")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => break c,
            Err(e) if e.raw_os_error() == Some(26) && tries < 50 => {
                tries += 1;
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => panic!("spawn {}: {e}", exe.display()),
        }
    };
    Guard(child, Some(dir))
}

/// Starts `cmd` and waits for a line containing `ready` on stdout.
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
    std::thread::spawn(move || {
        let mut sink = String::new();
        while reader.read_line(&mut sink).is_ok_and(|n| n > 0) {
            sink.clear();
        }
    });
    Guard(child, None)
}

fn have(tool: &str, flag: &str) -> bool {
    Proc::new(tool)
        .arg(flag)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

const FAKE_JCMD: &str = r#"
pid="$1"; cmd="$2"
case "$cmd" in
  VM.version) echo "$pid:"; echo "JDK 21.0.1";;
  GC.heap_info) echo "$pid:"; echo " garbage-first heap   total 10240K, used 8192K [0x0, 0x1)";;
  GC.heap_dump) printf 'JAVA PROFILE 1.0.2\0\0\0\0\0\0\0\0' > "$3"; echo "$pid:"; echo "Heap dump file created [26 bytes in 0.001 secs]";;
  GC.class_histogram) echo "$pid:"; echo " num     #instances         #bytes  class name (module)"; echo "-------------------------------------------------------"; echo "   1:            10           8000  [B (java.base@21)"; echo "Total            10           8000";;
  *) echo "unexpected $cmd" >&2; exit 1;;
esac
"#;

fn sha256_hex(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(fs::read(path).unwrap()))
}

// ---- refusals ----

#[test]
fn no_consent_and_too_low_a_tier_are_refused_naming_the_fix() {
    let target = sleeper();
    let pid = target.pid().to_string();
    let p = project_with(None);
    let e = refused(
        p.path(),
        &[],
        &["attach", "jcmd", "--pid", &pid, "--op", "histogram"],
    );
    assert_eq!(e["code"], "consent-required");
    assert!(e["fix"].as_str().unwrap().contains("dla consent"), "{e}");

    // Tier 1 only: a heap dump needs tier 4 and says so.
    let p = project_with(Some("1"));
    let e = refused(
        p.path(),
        &[],
        &["attach", "jcmd", "--pid", &pid, "--op", "heap-dump"],
    );
    assert_eq!(e["code"], "consent-required");
    assert!(e["message"].as_str().unwrap().contains("tier 4"), "{e}");
    // Tiers 1-3 still refuse a heap dump but allow a thread dump's tier.
    let p = project_with(Some("1-3"));
    let e = refused(
        p.path(),
        &[],
        &[
            "attach",
            "node-inspector",
            "--pid",
            &pid,
            "--op",
            "heap-snapshot",
        ],
    );
    assert!(e["message"].as_str().unwrap().contains("tier 4"), "{e}");
    // Nothing was written for the refused attaches.
    assert!(!p.run_dir.join("attach").join("P1").exists());
}

#[test]
fn bad_flags_are_refused_with_the_valid_choices() {
    let target = sleeper();
    let pid = target.pid().to_string();
    let p = project();
    let e = refused(
        p.path(),
        &[],
        &["attach", "gdb", "--pid", &pid, "--op", "x"],
    );
    assert_eq!(e["code"], "unknown-backend");
    assert!(e["message"].as_str().unwrap().contains("heaptrack"), "{e}");
    let e = refused(
        p.path(),
        &[],
        &["attach", "jcmd", "--pid", &pid, "--op", "record"],
    );
    assert!(e["message"].as_str().unwrap().contains("heap-dump"), "{e}");
    let e = refused(
        p.path(),
        &[],
        &[
            "attach",
            "jcmd",
            "--pid",
            &pid,
            "--op",
            "jfr",
            "--duration",
            "soon",
        ],
    );
    assert_eq!(e["code"], "invalid-value");
    let e = refused(
        p.path(),
        &[],
        &["attach", "jcmd", "--pid", "4194000", "--op", "histogram"],
    );
    assert!(
        e["message"].as_str().unwrap().contains("not running"),
        "{e}"
    );
}

#[test]
fn a_node_inspector_attach_to_another_kind_of_process_sends_no_signal() {
    let mut target = sleeper();
    let pid = target.pid().to_string();
    let p = project();
    let e = refused(
        p.path(),
        &[],
        &["attach", "node-inspector", "--pid", &pid, "--op", "memory"],
    );
    assert_eq!(e["code"], "target-not-ready");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("SIGUSR1 is not sent"),
        "{e}"
    );
    assert!(target.alive(), "sleep would have died of SIGUSR1");
}

#[test]
fn memray_refuses_a_process_that_is_not_python() {
    let target = sleeper();
    let pid = target.pid().to_string();
    let p = project();
    let e = refused(
        p.path(),
        &[],
        &["attach", "memray", "--pid", &pid, "--op", "record"],
    );
    assert_eq!(e["code"], "target-not-ready");
    assert!(e["message"].as_str().unwrap().contains("CPython"), "{e}");
}

#[test]
fn a_missing_tool_is_refused_with_the_install_command() {
    let target = sleeper();
    let pid = target.pid().to_string();
    let p = project();
    if Proc::new("sh")
        .args(["-c", "command -v grf"])
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        eprintln!("skipping: grf is installed here");
        return;
    }
    let e = refused(
        p.path(),
        &[],
        &["attach", "goref", "--pid", &pid, "--op", "heap-profile"],
    );
    assert_eq!(e["code"], "tool-missing");
    assert!(e["fix"].as_str().unwrap().contains("go install"), "{e}");
}

// ---- the flow, against a fake jcmd ----

#[test]
fn insufficient_disk_or_memory_is_refused_before_anything_is_created() {
    let target = java_sleeper();
    let pid = target.pid().to_string();
    let p = project();
    p.tool("jcmd", FAKE_JCMD);
    let args = ["attach", "jcmd", "--pid", &pid, "--op", "heap-dump"];
    let data = p.data.path().display().to_string();
    let e = refused(
        p.path(),
        &[("DLA_DATA_DIR", &data), ("DLA_ATTACH_FREE_DISK", "1000")],
        &args,
    );
    assert_eq!(e["code"], "insufficient-resources");
    assert!(e["message"].as_str().unwrap().contains("free disk"), "{e}");
    assert!(e["fix"].as_str().unwrap().contains("--op histogram"), "{e}");
    let e = refused(
        p.path(),
        &[
            ("DLA_DATA_DIR", &data),
            ("DLA_ATTACH_AVAILABLE_MEMORY", "1000000"),
        ],
        &args,
    );
    assert_eq!(e["code"], "insufficient-resources");
    assert!(e["message"].as_str().unwrap().contains("80%"), "{e}");
    // No per-attach directory, no registry record, no evidence.
    assert!(!p.run_dir.join("attach/P1").exists());
    assert!(
        p.state()["processes"]
            .as_array()
            .is_none_or(|a| a.is_empty())
    );
    assert!(p.ledger().iter().all(|l| l["kind"] != "artifact"));
}

#[test]
fn a_fake_jcmd_attach_writes_files_a_registry_record_and_one_artifact_evidence() {
    let mut target = java_sleeper();
    let pid = target.pid();
    let p = project();
    p.tool("jcmd", FAKE_JCMD);
    let v = p.attach_ok(&[
        "jcmd",
        "--pid",
        &pid.to_string(),
        "--op",
        "heap-dump",
        "--out",
        "first",
    ]);
    assert_eq!(v["backend"], "jcmd");
    assert_eq!(v["op"], "heap-dump");
    assert_eq!(v["preflight"]["tier"], 4);
    assert_eq!(v["preflight"]["decision"], "ok");
    assert_eq!(v["preflight"]["est_dump"], 8192 * 1024);
    assert_eq!(v["next"].as_str().unwrap().split(' ').next(), Some("dla"));
    let file = &v["files"][0];
    let rel = file["path"].as_str().unwrap();
    assert!(rel.ends_with("attach/P1/first.hprof"), "{rel}");
    let on_disk = p.path().join(rel);
    assert_eq!(file["sha256"], sha256_hex(&on_disk));
    assert_eq!(file["bytes"], fs::metadata(&on_disk).unwrap().len());
    assert!(p.run_dir.join("attach/P1/preflight.json").is_file());

    // One evidence of kind artifact, authority live, source process.
    let ev = p.evidence(v["evidence"].as_str().unwrap());
    assert_eq!(ev["kind"], "artifact");
    assert_eq!(ev["authority"], "live");
    assert_eq!(ev["source"]["process"]["pid"], pid);
    let r = &ev["result"];
    assert_eq!(r["backend"], "jcmd");
    assert_eq!(r["op"], "heap-dump");
    assert_eq!(r["files"][0]["path"], "attach/P1/first.hprof");
    assert_eq!(r["files"][0]["sha256"], file["sha256"]);
    assert_eq!(r["preflight"]["decision"], "ok");
    assert!(r["durations"]["total_ms"].is_number());
    assert!(r["cleanup_done"].is_array());
    assert_eq!(ev["raw"], "attach/P1/preflight.json");
    assert_eq!(
        p.ledger()
            .iter()
            .filter(|l| l["kind"] == "artifact")
            .count(),
        1
    );

    // The registry holds an attached profiler record, stopped, with the cleanup notes; the
    // target keeps running.
    let st = p.state();
    let rec = st["processes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["role"] == "attached")
        .expect("attached record");
    assert_eq!(rec["kind"], "profiler");
    assert_eq!(rec["pid"], pid);
    assert!(rec["stopped_at"].is_string(), "{rec}");
    let cleanup = rec["cleanup"].as_array().unwrap();
    assert!(
        cleanup
            .iter()
            .all(|c| c.as_str().unwrap().starts_with("note ")),
        "{cleanup:?}"
    );
    assert!(
        cleanup
            .iter()
            .any(|c| c.as_str().unwrap().contains(".java_pid"))
    );
    assert!(target.alive());

    // A second attach gets the next directory.
    let v = p.attach_ok(&["jcmd", "--pid", &pid.to_string(), "--op", "histogram"]);
    assert!(v["dir"].as_str().unwrap().ends_with("attach/P2"));
    assert_eq!(v["summary"]["classes"], 1);
}

#[test]
fn a_process_dla_started_needs_only_tier_one() {
    let target = java_sleeper();
    let pid = target.pid();
    let p = project_with(Some("1"));
    p.tool("jcmd", FAKE_JCMD);
    // Without a registry entry a histogram needs tier 2.
    let args = ["jcmd", "--pid", &pid.to_string(), "--op", "histogram"];
    let (code, v) = p.attach(&[], &args);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "consent-required");
    // Register the sleeper as a started process (state.json is the registry).
    let start = dla_sample::os::process_start_time(pid).unwrap();
    let mut st = p.state();
    st["processes"] = serde_json::json!([{
        "pid": pid, "start_time": start, "kind": "harness", "command": "sleep 300",
        "started_at": "2026-01-01T00:00:00Z"
    }]);
    fs::write(
        p.run_dir.join("state.json"),
        serde_json::to_string(&st).unwrap(),
    )
    .unwrap();
    let v = p.attach_ok(&args);
    assert_eq!(v["consent_tier"], 1, "{v}");
    assert_eq!(v["started_by_dla"], true);
    assert_eq!(v["preflight"]["tier"], 2, "the operation itself is tier 2");
}

#[test]
fn a_fake_goref_run_is_marked_unverified_everywhere() {
    let target = sleeper();
    let p = project();
    p.tool(
        "grf",
        "echo \"successfully output to $4\"; echo profile > \"$4\"",
    );
    let v = p.attach_ok(&[
        "goref",
        "--pid",
        &target.pid().to_string(),
        "--op",
        "heap-profile",
    ]);
    assert_eq!(v["unverified"], true);
    assert_eq!(v["preflight"]["unverified"], true);
    let ev = p.evidence(v["evidence"].as_str().unwrap());
    assert_eq!(ev["result"]["unverified"], true);
    assert!(
        ev["limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l.as_str().unwrap().contains("unverified"))
    );
}

// ---- real targets ----

const HOLDER_JAVA: &str = r#"
import java.util.*;
public class Holder {
    static final List<byte[]> PAGES = new ArrayList<>();
    public static void main(String[] a) throws Exception {
        for (int i = 0; i < 64; i++) PAGES.add(new byte[1 << 20]);
        System.out.println("ready");
        while (true) Thread.sleep(1000);
    }
}
"#;

fn start_jvm(dir: &Path, flags: &[&str]) -> Option<Guard> {
    if !have("javac", "-version") || !have("java", "-version") || !have("jcmd", "-h") {
        eprintln!("skipping: no JDK with javac, java and jcmd");
        return None;
    }
    fs::write(dir.join("Holder.java"), HOLDER_JAVA).unwrap();
    let out = Proc::new("javac")
        .arg("Holder.java")
        .current_dir(dir)
        .env_remove("JAVA_TOOL_OPTIONS")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut c = Proc::new("java");
    c.args(flags).args(["-cp", ".", "Holder"]).current_dir(dir);
    Some(start_ready(c, "ready"))
}

#[test]
fn a_real_jvm_gives_a_histogram_a_thread_dump_nmt_a_recording_and_a_heap_dump() {
    let work = tempfile::tempdir().unwrap();
    let Some(mut jvm) = start_jvm(
        work.path(),
        &["-XX:NativeMemoryTracking=summary", "-Xmx512m"],
    ) else {
        return;
    };
    let pid = jvm.pid().to_string();
    let p = project();

    let v = p.attach_ok(&["jcmd", "--pid", &pid, "--op", "histogram"]);
    assert!(
        v["summary"]["total_bytes"].as_u64().unwrap() > 60 << 20,
        "{v}"
    );
    assert!(
        v["summary"]["top"][0]["class"]
            .as_str()
            .unwrap()
            .starts_with("[B"),
        "{v}"
    );

    let v = p.attach_ok(&["jcmd", "--pid", &pid, "--op", "thread-dump"]);
    assert!(v["summary"]["threads"].as_u64().unwrap() >= 2, "{v}");

    let v = p.attach_ok(&["jcmd", "--pid", &pid, "--op", "nmt"]);
    assert!(v["summary"]["committed"].as_u64().unwrap() > 0, "{v}");

    let v = p.attach_ok(&["jcmd", "--pid", &pid, "--op", "jfr", "--duration", "2s"]);
    assert!(v["files"][0]["bytes"].as_u64().unwrap() > 1000, "{v}");

    let v = p.attach_ok(&["jcmd", "--pid", &pid, "--op", "heap-dump", "--out", "one"]);
    assert_eq!(v["summary"]["hprof_header_ok"], true, "{v}");
    let rel = v["files"][0]["path"].as_str().unwrap();
    let dump = p.path().join(rel);
    let head = fs::read(&dump).unwrap();
    assert!(head.starts_with(b"JAVA PROFILE 1.0.2"));
    // The dump holds the 64 MiB of pages, so it is at least that big.
    assert!(fs::metadata(&dump).unwrap().len() > 60 << 20);
    // The summarizer reads it when the build has one.
    let (code, s) = run_with(p.path(), &[], &["summarize", "heap-graph", rel]);
    if code == 0 {
        assert_eq!(s["ok"], true, "{s}");
    } else {
        assert_eq!(s["error"]["code"], "not-implemented", "{s}");
    }
    fs::remove_file(&dump).unwrap();
    assert!(jvm.alive());
}

#[test]
fn nmt_off_in_a_real_jvm_is_refused_with_the_flag() {
    let work = tempfile::tempdir().unwrap();
    let Some(jvm) = start_jvm(work.path(), &["-XX:NativeMemoryTracking=off", "-Xmx256m"]) else {
        return;
    };
    let p = project();
    let e = refused(
        p.path(),
        &[],
        &[
            "attach",
            "jcmd",
            "--pid",
            &jvm.pid().to_string(),
            "--op",
            "nmt",
        ],
    );
    assert_eq!(e["code"], "target-not-ready");
    assert!(
        e["fix"]
            .as_str()
            .unwrap()
            .contains("NativeMemoryTracking=summary"),
        "{e}"
    );
    assert!(
        p.state()["processes"]
            .as_array()
            .is_none_or(|a| a.is_empty())
    );
}

#[test]
fn a_real_node_process_gives_memory_and_a_streamed_heap_snapshot() {
    if !have("node", "--version") {
        eprintln!("skipping: node is not installed");
        return;
    }
    let mut c = Proc::new("node");
    c.args([
        "-e",
        "const keep=[]; for(let i=0;i<20000;i++){keep.push({id:i,s:'item-'+i,a:[i]});} console.log('ready'); setInterval(()=>{},1000)",
    ]);
    let mut node = start_ready(c, "ready");
    let pid = node.pid().to_string();
    let p = project();
    let v = p.attach_ok(&["node-inspector", "--pid", &pid, "--op", "memory"]);
    assert!(
        v["summary"]["memory"]["heapUsed"].as_u64().unwrap() > 1 << 20,
        "{v}"
    );
    assert!(
        v["cleanup_left"][0].as_str().unwrap().starts_with("note "),
        "{v}"
    );
    let v = p.attach_ok(&["node-inspector", "--pid", &pid, "--op", "heap-snapshot"]);
    assert!(v["summary"]["chunks"].as_u64().unwrap() >= 1, "{v}");
    let rel = v["files"][0]["path"].as_str().unwrap();
    // A V8 snapshot is one JSON document.
    let snap: Value =
        serde_json::from_str(&fs::read_to_string(p.path().join(rel)).unwrap()).unwrap();
    assert!(snap["snapshot"]["meta"].is_object());
    assert!(snap["nodes"].as_array().unwrap().len() > 1000);
    let ev = p.evidence(v["evidence"].as_str().unwrap());
    assert_eq!(ev["kind"], "artifact");
    assert_eq!(ev["result"]["op"], "heap-snapshot");
    assert!(node.alive());
}

#[test]
fn heaptrack_records_a_c_program_and_leaves_no_fifo() {
    if !have("gcc", "--version")
        || !have("heaptrack", "--version")
        || !have("heaptrack_print", "--version")
        || !have("gdb", "--version")
    {
        eprintln!("skipping: needs gcc, heaptrack, heaptrack_print and gdb");
        return;
    }
    if fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope").is_ok_and(|s| s.trim() != "0") {
        eprintln!("skipping: kernel.yama.ptrace_scope is not 0, heaptrack --pid refuses");
        return;
    }
    let work = tempfile::tempdir().unwrap();
    let src = work.path().join("grow.c");
    fs::write(
        &src,
        "#include <stdlib.h>\n#include <string.h>\n#include <unistd.h>\n#include <stdio.h>\nvoid *take_page(size_t n){ void *p = malloc(n); memset(p, 1, n); return p; }\nint main(){ puts(\"ready\"); fflush(stdout); for(;;){ take_page(50000); usleep(50000); } }\n",
    )
    .unwrap();
    let exe = work.path().join("grow");
    let cc = Proc::new("gcc")
        .args(["-g", "-O0", "-o"])
        .arg(&exe)
        .arg(&src)
        .status()
        .unwrap();
    assert!(cc.success());
    let mut prog = start_ready(Proc::new(&exe), "ready");
    let pid = prog.pid().to_string();
    let fifos_before = count_fifos();
    let p = project();
    let v = p.attach_ok(&[
        "heaptrack",
        "--pid",
        &pid,
        "--op",
        "record",
        "--duration",
        "3s",
    ]);
    assert!(
        v["summary"]["allocation_calls"].as_u64().unwrap() > 10,
        "{v}"
    );
    assert_eq!(
        v["summary"]["peak_consumers"][0]["function"], "take_page",
        "{v}"
    );
    let names: Vec<&str> = v["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(
        names
            .iter()
            .any(|n| n.ends_with("heaptrack.gz") || n.ends_with(".zst")),
        "{names:?}"
    );
    assert_eq!(
        count_fifos(),
        fifos_before,
        "a heaptrack FIFO was left in /tmp"
    );
    let st = prog.0.try_wait().unwrap();
    assert!(st.is_none(), "the target must keep running: {st:?}");
    std::thread::sleep(Duration::from_millis(800));
    let st = prog.0.try_wait().unwrap();
    assert!(st.is_none(), "the target died after the recording: {st:?}");
    let ev = p.evidence(v["evidence"].as_str().unwrap());
    assert_eq!(ev["coverage"]["status"], "partial");
}

fn count_fifos() -> usize {
    fs::read_dir("/tmp")
        .map(|d| {
            d.filter_map(Result::ok)
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("heaptrack_fifo")
                })
                .count()
        })
        .unwrap_or(0)
}

#[test]
fn memray_records_a_python_process_when_memray_is_importable() {
    // DLA_TEST_MEMRAY_PYTHONPATH points at a directory holding memray (pip install --target).
    let site = std::env::var("DLA_TEST_MEMRAY_PYTHONPATH").ok();
    let mut probe = Proc::new("python3");
    probe
        .args(["-c", "import memray"])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(s) = &site {
        probe.env("PYTHONPATH", s);
    }
    if !probe.status().is_ok_and(|s| s.success()) {
        eprintln!("skipping: memray is not importable (set DLA_TEST_MEMRAY_PYTHONPATH)");
        return;
    }
    if !have("gdb", "--version") {
        eprintln!("skipping: gdb is not installed (python below 3.14 needs it)");
        return;
    }
    let mut c = Proc::new("python3");
    c.args([
        "-c",
        "import time\nkeep=[]\nprint('ready',flush=True)\nwhile True:\n    keep.append(bytearray(50000)); time.sleep(0.05)\n",
    ]);
    if let Some(s) = &site {
        c.env("PYTHONPATH", s);
    }
    let mut py = start_ready(c, "ready");
    let p = project();
    let (code, v) = p.attach(
        &[],
        &[
            "memray",
            "--pid",
            &py.pid().to_string(),
            "--op",
            "record",
            "--duration",
            "3s",
        ],
    );
    assert_eq!(code, 0, "{v}");
    assert!(v["summary"]["allocations"].as_u64().unwrap() > 5, "{v}");
    assert!(v["files"].as_array().unwrap().len() >= 2, "{v}");
    assert!(py.alive());
}

#[test]
fn jcmd_does_not_signal_a_process_that_is_not_a_jvm() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("got-sigquit");
    let script = format!(
        "trap 'echo quit > {}; exit 0' QUIT; echo ready; while :; do sleep 0.1; done",
        marker.display()
    );
    let mut cmd = Proc::new("sh");
    cmd.args(["-c", &script]);
    let mut target = start_ready(cmd, "ready");
    let called = dir.path().join("jcmd-called");
    let p = project();
    p.tool(
        "jcmd",
        &format!("touch {}; kill -QUIT $1", called.display()),
    );
    let (code, v) = p.attach(
        &[],
        &[
            "jcmd",
            "--pid",
            &target.pid().to_string(),
            "--op",
            "histogram",
        ],
    );
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "target-not-ready");
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("no signal was sent"),
        "{v}"
    );
    std::thread::sleep(Duration::from_millis(500));
    assert!(!called.exists() && !marker.exists(), "SIGQUIT was sent");
    assert!(target.alive());
}

#[test]
fn out_cannot_name_a_file_dla_writes_into_the_attach_directory() {
    let target = java_sleeper();
    let pid = target.pid().to_string();
    let p = project();
    p.tool("jcmd", FAKE_JCMD);
    for name in ["preflight.json", "heaptrack.log", "x.stats.json"] {
        let (code, v) = p.attach(
            &[],
            &["jcmd", "--pid", &pid, "--op", "histogram", "--out", name],
        );
        assert_eq!(code, 2, "{name}: {v}");
        assert!(
            v["error"]["message"]
                .as_str()
                .unwrap()
                .contains("writes itself"),
            "{v}"
        );
    }
    assert!(!p.run_dir.join("attach/P1").exists());
}

#[test]
fn a_failed_attach_stays_open_so_abort_can_run_what_is_left() {
    let target = java_sleeper();
    let pid = target.pid().to_string();
    let p = project();
    let stopped = p.path().join("jfr-stopped");
    p.tool(
        "jcmd",
        &format!(
            r#"case "$2" in
  VM.version) echo "JDK 21.0.1";;
  JFR.start) echo "Could not start recording";;
  JFR.stop) touch {};;
  *) exit 1;;
esac"#,
            stopped.display()
        ),
    );
    let (code, v) = p.attach(
        &[],
        &["jcmd", "--pid", &pid, "--op", "jfr", "--duration", "1s"],
    );
    assert_eq!(code, 2, "{v}");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("JFR.stop") && msg.contains("dla run abort"),
        "{msg}"
    );
    let st = p.state();
    let rec = &st["processes"][0];
    assert!(
        rec["stopped_at"].is_null(),
        "abort skips stopped records: {rec}"
    );
    assert!(
        rec["cleanup"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str().unwrap().contains("JFR.stop")),
        "{rec}"
    );
}

#[test]
fn memray_never_executes_the_targets_argv0() {
    if !have("python3", "--version") || !have("bash", "--version") {
        eprintln!("skipped: python3 or bash is missing");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let evil = dir.path().join("evil.sh");
    let pwned = dir.path().join("pwned");
    fs::write(&evil, format!("#!/bin/sh\ntouch {}\n", pwned.display())).unwrap();
    fs::set_permissions(&evil, fs::Permissions::from_mode(0o755)).unwrap();
    let mut cmd = Proc::new("bash");
    cmd.args([
        "-c",
        &format!(
            "exec -a {} python3 -c 'import sys,time; print(\"ready\", flush=True); time.sleep(120)'",
            evil.display()
        ),
    ]);
    let target = start_ready(cmd, "ready");
    let p = project();
    let (_, v) = p.attach(
        &[("PYTHONPATH", "")],
        &[
            "memray",
            "--pid",
            &target.pid().to_string(),
            "--op",
            "record",
            "--duration",
            "1s",
        ],
    );
    assert!(!pwned.exists(), "argv[0] of the target was executed: {v}");
    if v["ok"] == false {
        assert!(!v.to_string().contains("evil.sh"), "{v}");
    }
}
