//! The attach backends against fake tools (shell scripts in a tools directory) and a plain
//! `sleep` process, so no JVM, Node.js or debugger is needed.
#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

use dla_sample::attach::{
    self, AttachError, Backend, Cleanup, DiskFigures, ErrorKind, MemoryAvailable, Op, Request,
    inspect_target, plan, preflight, run,
};

/// A child that is killed when dropped.
struct Sleeper(Child);

impl Sleeper {
    fn start() -> Self {
        Self(
            Command::new("sleep")
                .arg("120")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        )
    }
}

impl Drop for Sleeper {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn script(dir: &Path, name: &str, body: &str) {
    let p = dir.join(name);
    fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

const FAKE_JCMD: &str = r#"
pid="$1"; cmd="$2"
case "$cmd" in
  VM.version) echo "$pid:"; echo "JDK 21.0.1";;
  GC.heap_info) echo "$pid:"; echo " garbage-first heap   total 10240K, used 8192K [0x0, 0x1)";;
  GC.heap_dump) printf 'JAVA PROFILE 1.0.2\0\0\0\0\0\0\0\0' > "$3"; echo "$pid:"; echo "Heap dump file created [26 bytes in 0.001 secs]";;
  GC.class_histogram) echo "$pid:"; echo " num     #instances         #bytes  class name (module)"; echo "-------------------------------------------------------"; echo "   1:            10           8000  [B (java.base@21)"; echo "   2:             5            400  java.lang.String (java.base@21)"; echo "Total            15           8400";;
  Thread.dump_to_file) if [ -n "$FAKE_NO_JSON_DUMP" ]; then echo "Unknown diagnostic command" >&2; exit 1; fi; echo '{"threadDump":{"threadContainers":[{"threads":[{},{}]}]}}' > "$4"; echo "$pid:";;
  Thread.print) echo "$pid:"; echo '"main" #1 prio=5'; echo '"Signal Dispatcher" #4';;
  VM.native_memory) echo "$pid:"; echo "Native memory tracking is not enabled";;
  *) echo "unexpected $cmd" >&2; exit 1;;
esac
"#;

struct Fixture {
    _dir: tempfile::TempDir,
    tools: PathBuf,
    dump_root: PathBuf,
    out: PathBuf,
    target: Sleeper,
}

fn fixture(tool: &str, body: &str) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let tools = dir.path().join("tools");
    fs::create_dir_all(&tools).unwrap();
    script(&tools, tool, body);
    let dump_root = dir.path().join("attach");
    let out = dump_root.join("P1");
    fs::create_dir_all(&out).unwrap();
    Fixture {
        tools,
        dump_root,
        out,
        target: Sleeper::start(),
        _dir: dir,
    }
}

fn request(f: &Fixture, backend: Backend, op: Op) -> Request {
    Request {
        target: inspect_target(f.target.0.id()).expect("the sleeper runs"),
        backend,
        op,
        duration: Duration::from_secs(2),
        out_name: None,
        dump_root: f.dump_root.clone(),
        tool_dirs: vec![f.tools.clone()],
    }
}

fn run_plan(req: &Request, f: &Fixture) -> (attach::Finished, Vec<String>) {
    let p = plan(req).expect("plan");
    let mut registered = Vec::new();
    let fin = run(p, req, &f.out, &mut |c: &Cleanup| {
        registered.push(c.render())
    });
    (fin, registered)
}

#[test]
fn histogram_is_parsed_and_written() {
    let f = fixture("jcmd", FAKE_JCMD);
    let req = request(&f, Backend::Jcmd, Op::Histogram);
    let (fin, _) = run_plan(&req, &f);
    let done = fin.result.expect("histogram");
    assert_eq!(done.files, ["histogram.txt"]);
    assert_eq!(done.summary["classes"], 2);
    assert_eq!(done.summary["total_bytes"], 8400);
    assert!(f.out.join("histogram.txt").is_file());
    assert!(done.durations.contains_key("total_ms"));
    // The attach socket note is all that is left.
    assert_eq!(fin.cleanup_left.len(), 1);
    assert!(matches!(&fin.cleanup_left[0], Cleanup::Note(t) if t.contains(".java_pid")));
}

#[test]
fn heap_dump_checks_the_header_and_registers_the_partial_file_while_running() {
    let f = fixture("jcmd", FAKE_JCMD);
    let mut req = request(&f, Backend::Jcmd, Op::HeapDump);
    req.out_name = Some("before".into());
    let p = plan(&req).unwrap();
    // The estimate comes from GC.heap_info used (8192K).
    assert_eq!(p.estimate.dump_bytes, 8192 << 10);
    assert!(p.effects.iter().any(|e| e.contains("full GC")));
    let mut registered = Vec::new();
    let fin = run(p, &req, &f.out, &mut |c: &Cleanup| {
        registered.push(c.clone())
    });
    let done = fin.result.expect("heap dump");
    assert_eq!(done.files, ["before.hprof"]);
    assert_eq!(done.summary["hprof_header_ok"], true);
    assert!(
        registered
            .iter()
            .any(|c| matches!(c, Cleanup::Rm(p) if p.ends_with("P1/before.hprof")))
    );
    // Once the dump exists it is no longer scheduled for removal.
    assert!(
        fin.cleanup_left
            .iter()
            .all(|c| !matches!(c, Cleanup::Rm(_)))
    );
}

#[test]
fn a_failed_heap_dump_removes_the_partial_file_and_says_what_it_did() {
    // The fake writes a file and then fails.
    let body = FAKE_JCMD.replace(
        "GC.heap_dump) printf",
        "GC.heap_dump) echo partial > \"$3\"; echo 'Unable to create file' >&2; exit 1; printf",
    );
    let f = fixture("jcmd", &body);
    let req = request(&f, Backend::Jcmd, Op::HeapDump);
    let (fin, _) = run_plan(&req, &f);
    let err = fin.result.expect_err("must fail");
    assert_eq!(err.kind, ErrorKind::Failed);
    assert!(!f.out.join("heap.hprof").exists());
    assert!(fin.cleanup_done.iter().any(|d| d.starts_with("removed ")));
}

#[test]
fn nmt_off_is_refused_in_the_plan_with_the_flag_to_start_the_jvm_with() {
    let f = fixture("jcmd", FAKE_JCMD);
    let req = request(&f, Backend::Jcmd, Op::Nmt);
    let e: AttachError = plan(&req).unwrap_err();
    assert_eq!(e.kind, ErrorKind::NotReady);
    assert!(e.message.contains("not enabled"), "{}", e.message);
    assert!(
        e.fix.contains("-XX:NativeMemoryTracking=summary"),
        "{}",
        e.fix
    );
}

#[test]
fn thread_dump_falls_back_to_text_before_jdk_21() {
    let body = format!("export FAKE_NO_JSON_DUMP=1\n{FAKE_JCMD}");
    let f = fixture("jcmd", &body);
    let req = request(&f, Backend::Jcmd, Op::ThreadDump);
    let (fin, _) = run_plan(&req, &f);
    let done = fin.result.expect("thread dump");
    assert_eq!(done.files, ["threads.txt"]);
    assert_eq!(done.summary["format"], "text");
    assert_eq!(done.summary["threads"], 2);
    assert!(!done.limitations.is_empty());
}

#[test]
fn thread_dump_json_counts_threads() {
    let f = fixture("jcmd", FAKE_JCMD);
    let req = request(&f, Backend::Jcmd, Op::ThreadDump);
    let (fin, _) = run_plan(&req, &f);
    let done = fin.result.expect("thread dump");
    assert_eq!(done.files, ["threads.json"]);
    assert_eq!(done.summary["threads"], 2);
}

#[test]
fn a_dump_directory_the_jvm_user_cannot_write_is_refused_in_the_plan() {
    let f = fixture("jcmd", FAKE_JCMD);
    let mut req = request(&f, Backend::Jcmd, Op::HeapDump);
    // Pretend the JVM runs as another user.
    let me = req.target.uid.unwrap();
    if me == 0 {
        return; // root passes every check
    }
    req.target.uid = Some(me + 1);
    req.target.gids = vec![];
    fs::set_permissions(&f.dump_root, fs::Permissions::from_mode(0o700)).unwrap();
    let e = plan(&req).unwrap_err();
    assert_eq!(e.kind, ErrorKind::NotReady);
    assert!(e.message.contains("cannot write"), "{}", e.message);
}

#[test]
fn the_preflight_gate_refuses_a_dump_that_does_not_fit_and_names_both_rules() {
    let f = fixture("jcmd", FAKE_JCMD);
    let req = request(&f, Backend::Jcmd, Op::HeapDump);
    let p = plan(&req).unwrap();
    let ok = preflight(
        &req.target,
        &p,
        DiskFigures {
            run_dir_free: Some(1 << 40),
            dump_dir_free: Some(1 << 40),
        },
        MemoryAvailable {
            host: Some(1 << 40),
            cgroup: None,
        },
    );
    assert_eq!(ok.decision, "ok");
    assert_eq!(ok.tier.tier, 4);
    assert_eq!(ok.target.pid, req.target.pid);
    assert!(ok.compact().to_string().len() < 700);
    let bad = preflight(
        &req.target,
        &p,
        DiskFigures {
            run_dir_free: Some(1 << 40),
            dump_dir_free: Some(1000),
        },
        MemoryAvailable {
            host: Some(1 << 40),
            cgroup: Some(1 << 20),
        },
    );
    assert_eq!(bad.decision, "refused");
    assert_eq!(bad.reasons.len(), 2, "{:?}", bad.reasons);
}

#[test]
fn a_missing_tool_is_reported_with_the_install_command() {
    // An empty tools directory: neither the fake nor (normally) a real grf exists.
    let f = fixture("unrelated", "exit 0");
    let req = request(&f, Backend::Goref, Op::HeapProfile);
    match plan(&req) {
        Err(e) => {
            assert_eq!(e.kind, ErrorKind::ToolMissing);
            assert!(
                e.fix
                    .contains("go install github.com/cloudwego/goref/cmd/grf")
            );
        }
        Ok(_) => eprintln!("skipping: a real grf is installed here"),
    }
}

#[test]
fn goref_is_marked_unverified_and_its_profile_is_collected() {
    let f = fixture(
        "grf",
        "echo \"successfully output to $4\"; echo profile > \"$4\"",
    );
    let req = request(&f, Backend::Goref, Op::HeapProfile);
    let p = plan(&req).unwrap();
    assert!(p.unverified);
    assert!(p.effects.iter().any(|e| e.contains("SUSPENDS")));
    assert!(p.estimate.peak_bytes >= 256 << 20);
    let mut reg = Vec::new();
    let fin = run(p, &req, &f.out, &mut |c: &Cleanup| reg.push(c.render()));
    let done = fin.result.expect("goref");
    assert_eq!(done.files, ["grf.out"]);
    assert_eq!(done.summary["unverified"], true);
    assert!(
        done.limitations
            .iter()
            .any(|l| l.contains("never executed"))
    );
    assert!(reg.iter().any(|r| r.starts_with("rm ")));
    assert!(reg.iter().any(|r| r.starts_with("note ")));
}

#[test]
fn memray_and_node_refuse_a_process_of_another_kind_without_touching_it() {
    let f = fixture("unrelated", "exit 0");
    let req = request(&f, Backend::Memray, Op::Record);
    let e = plan(&req).unwrap_err();
    assert_eq!(e.kind, ErrorKind::NotReady);
    assert!(e.message.contains("CPython"), "{}", e.message);
    let req = request(&f, Backend::NodeInspector, Op::Memory);
    let e = plan(&req).unwrap_err();
    assert_eq!(e.kind, ErrorKind::NotReady);
    assert!(e.message.contains("SIGUSR1 is not sent"), "{}", e.message);
    // sleep(1) would have died of SIGUSR1.
    assert!(attach::inspect_target(f.target.0.id()).is_some());
}
