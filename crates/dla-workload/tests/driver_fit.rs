//! The driver templates fit their rounds into the process's time limit (`DLA_TIME_LIMIT_SECS`):
//! a unit so slow that one full round alone would outlast the limit still delivers all its
//! points, because the step is lowered inside the first round, and the process ends by itself
//! (a thread that never ends does not hold it). Node is covered through the binary in
//! `dla-cli/tests/node_short.rs`. Each test skips, printing why, without its toolchain.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::Value;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn have(tool: &str, arg: &str) -> bool {
    let ok = Command::new(tool)
        .arg(arg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipped: {tool} is not available");
    }
    ok
}

/// A template with its placeholders filled: 300 units a round of 100 ms each is 30 s a round,
/// against a 16 s limit.
fn fill(text: &str, extra: &[(&str, &str)]) -> String {
    let mut s = text.to_owned();
    for (k, v) in [
        ("DLA_STEP", "300"),
        ("DLA_WARMUP", "1"),
        ("DLA_ROUNDS", "4"),
        ("DLA_REPEAT", "8"),
    ]
    .iter()
    .chain(extra)
    {
        s = s.replace(&format!("@@{k}@@"), v);
    }
    s
}

/// The checkpoint records of the run and the limitation lines of the last one.
fn read(out: &Path) -> (Vec<i64>, Vec<String>) {
    let text = fs::read_to_string(out).expect("no records were written");
    let recs: Vec<Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|r: &Value| r["phase"] == "checkpoint")
        .collect();
    let xs = recs.iter().map(|r| r["x"].as_i64().unwrap()).collect();
    let notes = recs
        .iter()
        .flat_map(|r| r["limitations"].as_array().cloned().unwrap_or_default())
        .filter_map(|l| l.as_str().map(str::to_owned))
        .filter(|l| l.starts_with("the driver lowered its step"))
        .collect();
    (xs, notes)
}

fn check(out: &Path, started: std::time::Instant) {
    let (xs, notes) = read(out);
    assert_eq!(xs.len(), 5, "every point arrives: {xs:?}");
    assert!(xs[0] < 300, "the first round was cut short: {xs:?}");
    assert!(
        notes
            .iter()
            .any(|n| n.contains("in round 1 (after 10 units of it)")),
        "{notes:?}"
    );
    assert!(started.elapsed().as_secs() < 16, "{:?}", started.elapsed());
}

#[test]
fn the_python_driver_is_fitted_inside_its_first_round_and_ends_by_itself() {
    if !have("python3", "--version") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let t = fs::read_to_string(repo().join("harness/python/dla_harness.py")).unwrap();
    let t = fill(&t, &[])
        .replace(
            "raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            "import threading; threading.Thread(target=lambda: time.sleep(600)).start()",
        )
        .replace(
            "raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            "time.sleep(0.1)",
        );
    fs::write(dir.path().join("dla_harness.py"), t).unwrap();
    let out = dir.path().join("out.jsonl");
    let started = std::time::Instant::now();
    let st = Command::new("python3")
        .arg("dla_harness.py")
        .current_dir(dir.path())
        .env(
            "PYTHONPATH",
            format!("{}:.", repo().join("harness/python").display()),
        )
        .env("DLA_HARNESS_OUT", &out)
        .env("DLA_TIME_LIMIT_SECS", "16")
        .status()
        .unwrap();
    assert!(st.success());
    check(&out, started);
}

#[test]
fn the_go_driver_is_fitted_inside_its_first_round() {
    if !have("go", "version") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("go.mod"), "module slow\n\ngo 1.21\n").unwrap();
    fs::write(dir.path().join("slow.go"), "package slow\n").unwrap();
    let t = fs::read_to_string(repo().join("harness/go/zz_dla_driver_test.go.tmpl")).unwrap();
    let t = fill(&t, &[("DLA_PACKAGE", "slow")])
        .replace("panic(\"DLA-HOOK setup has not been filled in\")", "")
        .replace(
            "panic(\"DLA-HOOK unit has not been filled in\")",
            "dlatime.Sleep(100 * dlatime.Millisecond)",
        );
    fs::write(dir.path().join("zz_dla_driver_test.go"), t).unwrap();
    let out = dir.path().join("out.jsonl");
    let started = std::time::Instant::now();
    let o = Command::new("go")
        .args([
            "test",
            "-vet=off",
            "-count=1",
            "-run",
            "^TestDLADriver$",
            ".",
        ])
        .current_dir(dir.path())
        .env("DLA_HARNESS_OUT", &out)
        .env("DLA_TIME_LIMIT_SECS", "16")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stdout));
    check(&out, started);
}

#[test]
fn the_jvm_driver_is_fitted_inside_its_first_round_and_ends_by_itself() {
    if !have("javac", "-version") || !have("java", "-version") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src/demo");
    fs::create_dir_all(&src).unwrap();
    let t = fs::read_to_string(repo().join("harness/jvm/DlaHarness.java")).unwrap();
    let t = fill(&t, &[("DLA_PACKAGE", "demo"), ("DLA_CLASS", "Drv")])
        .replace(
            "throw new UnsupportedOperationException(\"DLA-HOOK setup has not been filled in\");",
            "new Thread(() -> { try { Thread.sleep(600000); } catch (Exception e) {} }).start();",
        )
        .replace(
            "throw new UnsupportedOperationException(\"DLA-HOOK unit has not been filled in\");",
            "Thread.sleep(100);",
        );
    fs::write(src.join("Drv.java"), t).unwrap();
    let classes = dir.path().join("cls");
    fs::create_dir_all(&classes).unwrap();
    let jvm = repo().join("harness/jvm");
    let st = Command::new("javac")
        .arg("-d")
        .arg(&classes)
        .args(["DlaDump.java", "DlaCapture.java", "DlaAgent.java"].map(|f| jvm.join(f)))
        .arg(src.join("Drv.java"))
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(st.success());
    let out = dir.path().join("out.jsonl");
    let started = std::time::Instant::now();
    let st = Command::new("java")
        .arg("-cp")
        .arg(&classes)
        .arg("demo.Drv")
        .env("DLA_HARNESS_OUT", &out)
        .env("DLA_TIME_LIMIT_SECS", "16")
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(st.success());
    check(&out, started);
}

#[test]
fn the_rust_driver_is_fitted_inside_its_first_round() {
    if !have("rustc", "--version") {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let t = fs::read_to_string(repo().join("harness/rust/dla_harness.rs")).unwrap();
    let t = fill(&t, &[("DLA_CRATE", "demo"), ("DLA_UNSAFE_EXTERN", "")])
        .replace(
            "todo!(\"DLA-HOOK setup has not been filled in\")",
            "State::default()",
        )
        .replace(
            "todo!(\"DLA-HOOK unit has not been filled in\")",
            "std::thread::sleep(std::time::Duration::from_millis(100))",
        );
    fs::write(dir.path().join("d.rs"), t).unwrap();
    let st = Command::new("rustc")
        .args(["--edition", "2021", "-O", "d.rs", "-o", "d"])
        .current_dir(dir.path())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert!(st.success());
    let out = dir.path().join("out.jsonl");
    let started = std::time::Instant::now();
    let st = Command::new(dir.path().join("d"))
        .env("DLA_HARNESS_OUT", &out)
        .env("DLA_TIME_LIMIT_SECS", "16")
        .status()
        .unwrap();
    assert!(st.success());
    check(&out, started);
}
