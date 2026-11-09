//! `dla-bench dynamic --without`, with a stand-in for the dla executable and fake tools.
#![cfg(unix)]

use std::{
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn write(p: &Path, text: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn script(p: &Path, body: &str) {
    write(p, &format!("#!/bin/sh\n{body}\n"));
    fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
}

/// One tiny fixture of category impossible: the stand-in dla never records a finding.
fn fixtures(root: &Path) {
    write(
        &root.join("fx-one/project/src/Main.java"),
        "class Main {}\n",
    );
    write(
        &root.join("fx-one/expected.toml"),
        r#"schema_version = 1
id = "fx-one"
runtime = "jvm"
category = "impossible"
dynamic_confirmation = false

[task]
prompt = "The service grows."

[[leak]]
id = "L1"
site = "src/Main.java:1"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
confirmable = false
"#,
    );
}

/// The stand-in dla: every call prepares the run directory and writes a coverage manifest in
/// which `jcmd` is `ran` when the call can still find a jcmd, `missing` otherwise. The tool
/// is looked up the way dla does: on PATH, then under JAVA_HOME.
const FAKE_DLA: &str = r#"
if [ -n "$FAKE_CRASH" ] && [ -f .dla/current ]; then echo boom >&2; exit 101; fi
mkdir -p .dla/R1
echo R1 > .dla/current
: > .dla/R1/ledger.jsonl
st=missing
if command -v jcmd >/dev/null 2>&1; then st=ran; fi
if [ -n "$JAVA_HOME" ] && [ -x "$JAVA_HOME/bin/jcmd" ]; then st=ran; fi
vg=missing
if command -v valgrind >/dev/null 2>&1; then vg=ran; fi
echo "jcmd=$st valgrind=$vg" >> "$FAKE_OUT"
printf '{"workers":{"jvm":{"backends":{"jcmd":{"status":"%s"},"valgrind":{"status":"%s"},"heaptrack":{"status":"ran"}}}}}\n' "$st" "$vg" > .dla/R1/coverage.json
echo '{"ok":true}'
"#;

struct Setup {
    root: tempfile::TempDir,
    java_home: PathBuf,
    path: OsString,
}

fn setup() -> Setup {
    let root = tempfile::tempdir().unwrap();
    let t = root.path();
    fixtures(&t.join("fixtures"));
    fs::create_dir_all(t.join("repo/data")).unwrap();
    write(&t.join("repo/data/empty.txt"), "x\n");
    script(&t.join("dla"), FAKE_DLA);
    script(&t.join("tools/jcmd"), "exit 0");
    script(&t.join("tools/valgrind"), "exit 0");
    let java_home = t.join("jdk");
    script(&java_home.join("bin/jcmd"), "exit 0");
    let path = std::env::join_paths([
        t.join("tools"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ])
    .unwrap();
    Setup {
        root,
        java_home,
        path,
    }
}

fn dynamic(s: &Setup, extra: &[&str]) -> Output {
    dynamic_env(s, extra, &[])
}

fn dynamic_env(s: &Setup, extra: &[&str], envs: &[(&str, &str)]) -> Output {
    let t = s.root.path();
    Command::new(env!("CARGO_BIN_EXE_dla-bench"))
        .arg("dynamic")
        .arg("--fixtures")
        .arg(t.join("fixtures"))
        .arg("--repo-root")
        .arg(t.join("repo"))
        .arg("--dla-bin")
        .arg(t.join("dla"))
        .arg("--json")
        .arg(t.join("out.json"))
        .args(extra)
        .env("PATH", &s.path)
        .env("JAVA_HOME", &s.java_home)
        .env("FAKE_OUT", s.root.path().join("seen.txt"))
        .envs(envs.iter().copied())
        .output()
        .unwrap()
}

fn row(s: &Setup) -> serde_json::Value {
    let text = fs::read_to_string(s.root.path().join("out.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    v["fixtures"][0].clone()
}

fn seen(s: &Setup) -> String {
    fs::read_to_string(s.root.path().join("seen.txt")).unwrap_or_default()
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn without_a_tool_it_is_gone_from_every_call_and_the_run_completes() {
    let s = setup();
    let out = dynamic(&s, &["--without", "jcmd"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("ok: 1 fixture(s) pass"),
        "{}",
        text(&out)
    );
    let r = row(&s);
    assert_eq!(r["hidden_backend_rows"], 1, "{r}");
    // JAVA_HOME points at a jcmd, and it was unset for the call; valgrind stays visible.
    let seen = seen(&s);
    assert!(!seen.is_empty());
    for line in seen.lines() {
        assert_eq!(line, "jcmd=missing valgrind=ran", "{seen}");
    }
}

#[test]
fn the_control_run_sees_the_tool() {
    let s = setup();
    let out = dynamic(&s, &[]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(row(&s).get("hidden_backend_rows").is_none());
    for line in seen(&s).lines() {
        assert_eq!(line, "jcmd=ran valgrind=ran");
    }
}

#[test]
fn several_tools_can_be_hidden_at_once() {
    let s = setup();
    let out = dynamic(&s, &["--without", "jcmd,valgrind"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(row(&s)["hidden_backend_rows"], 2);
    for line in seen(&s).lines() {
        assert_eq!(line, "jcmd=missing valgrind=missing");
    }
}

#[test]
fn a_hidden_backend_whose_row_still_reads_ran_fails_the_run() {
    // The stand-in reports heaptrack as ran whatever the PATH is.
    let s = setup();
    let out = dynamic(&s, &["--without", "heaptrack"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(
        text(&out).contains("hidden tool heaptrack"),
        "{}",
        text(&out)
    );
}

#[test]
fn a_bad_tool_name_is_refused() {
    let s = setup();
    let out = dynamic(&s, &["--without", "bin/jcmd"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("not a tool name"));
}

#[test]
fn a_crash_of_dla_in_a_without_run_fails_the_run() {
    let s = setup();
    let out = dynamic_env(&s, &["--without", "jcmd"], &[("FAKE_CRASH", "1")]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(text(&out).contains("crashed"), "{}", text(&out));
}

#[test]
fn hiding_a_tool_no_row_depends_on_fails_the_run() {
    let s = setup();
    let out = dynamic(&s, &["--without", "git"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(
        text(&out).contains("no coverage row depends on it"),
        "{}",
        text(&out)
    );
}

#[test]
fn a_relative_path_entry_does_not_break_the_run() {
    let mut s = setup();
    let t = s.root.path().to_path_buf();
    let cwd = std::env::current_dir().unwrap();
    // A relative entry that holds a copy of valgrind: it must not hide the later one.
    let rel = cwd.join("target/without-rel-run");
    let _ = fs::remove_dir_all(&rel);
    script(&rel.join("valgrind"), "exit 0");
    s.path = std::env::join_paths([
        PathBuf::from("target/without-rel-run"),
        t.join("tools"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ])
    .unwrap();
    let out = dynamic(&s, &["--without", "jcmd"]);
    let _ = fs::remove_dir_all(&rel);
    assert!(out.status.success(), "{}", text(&out));
    for line in seen(&s).lines() {
        assert_eq!(line, "jcmd=missing valgrind=ran", "{line}");
    }
}
