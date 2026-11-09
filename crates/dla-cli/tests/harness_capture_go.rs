//! `dla harness run --capture alloc,nonheap` on a Go harness through the binary: heap
//! profiles, goroutine profiles and dumps, the goroutineleak pass, the printed
//! `dla summarize` commands, and the coverage row of the goroutineleak profile.
//!
//! The expected shape depends on the Go toolchain of the machine, which the test reads the
//! way `dla probe` does: Go 1.27 or newer writes the leak profile in the main run, an older
//! one with a cached 1.27 toolchain runs the separate pass, and otherwise the coverage row
//! is `missing` with the install command. Each test skips, printing the reason, when `go`
//! is not installed.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
    time::Duration,
};

use assert_cmd::Command;
use serde_json::Value;

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/fixtures")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

fn have_go() -> bool {
    let ok = Std::new("go")
        .arg("version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipping: go is not on PATH");
    }
    ok
}

struct Out {
    code: i32,
    json: Value,
    text: String,
}

fn dla_with(project: &Path, args: &[&str], env: &[(&str, &str)]) -> Out {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
        "DLA_CAPTURE",
        "DLA_CAPTURE_DIR",
    ] {
        c.env_remove(var);
    }
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    assert!(
        text.len() <= dla_core::MAX_OUTPUT_BYTES,
        "output over the bound: {} bytes: {text}",
        text.len()
    );
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla_with(project, args, &[]);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    assert_eq!(o.json["ok"], true, "{}", o.text);
    o.json
}

/// A project copy of `fixture` with a run, consent for tiers 1 and 2, a census and the
/// workload discovered, and harness `H1` generated.
fn prepared(fixture: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    copy_dir(&fixtures_root().join(fixture).join("project"), &project);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "go"]);
    ok(&project, &["workload", "discover", "--lang", "go"]);
    ok(&project, &["harness", "gen", "--lang", "go"]);
    (dir, project, run_dir)
}

fn captures(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("harness/H1/captures.json")).unwrap())
        .unwrap()
}

fn files_ending(c: &Value, tail: &str) -> Vec<String> {
    c["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["path"].as_str())
        .filter(|p| p.ends_with(tail))
        .map(str::to_owned)
        .collect()
}

fn coverage_row(run_dir: &Path) -> Value {
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("coverage.json")).unwrap()).unwrap();
    cov["workers"]["go"]["backends"]["goroutineleak"].clone()
}

/// The printed summarize command that mentions `needle`, as arguments after `dla`.
fn command_with(c: &Value, needle: &str) -> Vec<String> {
    c["summarize"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|s| s.contains(needle))
        .unwrap_or_else(|| panic!("no summarize command with {needle}: {c}"))
        .split(' ')
        .skip(1)
        .map(str::to_owned)
        .collect()
}

fn run_summarize(project: &Path, args: &[String]) -> Value {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    ok(project, &a)
}

fn leak_expected(project: &Path) -> &'static str {
    let g = dla_probe::go_toolchain::probe(project, Duration::from_secs(30));
    match g.goroutineleak.as_str() {
        "native" => "native",
        "experiment" => "experiment",
        "toolchain" => "toolchain",
        _ => "missing",
    }
}

#[test]
fn a_leaking_service_gives_profiles_a_tied_growth_column_and_a_leak_group() {
    if !have_go() {
        return;
    }
    let (_d, project, run_dir) = prepared("go-subscription-workers");
    let expected = leak_expected(&project);
    let run = ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "alloc,nonheap",
            "--iterations",
            "3",
        ],
    );
    assert!(
        run["summary"]["capture"]["files"].as_u64().unwrap() >= 12,
        "{run}"
    );
    let c = captures(&run_dir);
    assert_eq!(files_ending(&c, ".heap.pb.gz").len(), 4, "{c}");
    assert_eq!(files_ending(&c, ".goroutine.txt").len(), 4);
    assert_eq!(files_ending(&c, ".goroutine.pb.gz").len(), 4);
    for f in files_ending(&c, ".heap.pb.gz") {
        assert!(run_dir.join(&f).is_file(), "{f}");
    }

    // Goroutines grow with the replay size and tie to the go statement the census found.
    let v = run_summarize(&project, &command_with(&c, "go-1.goroutine.txt"));
    assert_eq!(v["backend"], "go-goroutine");
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"][0], "F2", "{v}");
    let growth = v["growth_evidence"].as_str().unwrap();
    let ledger = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    assert!(
        ledger.contains("internal/notify/subscription.go:33"),
        "{growth}"
    );

    // The heap profiles diff into stacks with project sites.
    let h = run_summarize(&project, &command_with(&c, "alloc-stack-diff"));
    assert_eq!(h["backend"], "go-pprof-heap");
    assert_eq!(h["live"], true);
    assert_eq!(h["sampled"], true);

    // The goroutineleak profile, by the way this machine's toolchain reaches it.
    let row = coverage_row(&run_dir);
    let leaks = files_ending(&c, ".goroutineleak.txt");
    match expected {
        "missing" => {
            assert_eq!(row["status"], "missing", "{row}");
            assert_eq!(row["install_hint"], "dla install go-toolchain-1.27");
            assert!(leaks.is_empty());
            assert!(
                c["limitations"]
                    .to_string()
                    .contains("goroutineleak: missing"),
                "{c}"
            );
        }
        other => {
            assert_eq!(row["status"], "ran", "{other}: {row}");
            assert!(!leaks.is_empty(), "{other}: no leak file in {c}");
            let l = run_summarize(&project, &command_with(&c, "goroutineleak.txt"));
            assert_eq!(l["backend"], "go-goroutineleak");
            assert!(l["total"].as_u64().unwrap() > 0, "{l}");
            assert_eq!(
                l["groups"][0]["site"], "internal/notify/subscription.go:33",
                "{l}"
            );
            assert_eq!(l["tied"][0], "F2", "{l}");
            assert_eq!(
                l["groups"][0]["blocked_at"],
                "internal/notify/subscription.go:40"
            );
        }
    }
}

#[test]
fn a_service_that_closes_its_workers_leaks_nothing() {
    if !have_go() {
        return;
    }
    let (_d, project, run_dir) = prepared("go-subscription-workers-closed");
    let expected = leak_expected(&project);
    ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "alloc,nonheap",
            "--iterations",
            "3",
        ],
    );
    let c = captures(&run_dir);
    let v = run_summarize(&project, &command_with(&c, "go-1.goroutine.txt"));
    assert_eq!(v["gate"], "fail", "{v}");
    assert_eq!(v["tied"], serde_json::json!([]));
    if expected != "missing" {
        let l = run_summarize(&project, &command_with(&c, "goroutineleak.txt"));
        assert_eq!(l["total"], 0, "{l}");
        assert!(
            l["needed"].as_str().unwrap().contains("proves nothing"),
            "{l}"
        );
    }
}

#[test]
fn a_run_without_the_nonheap_kind_writes_no_goroutine_files() {
    if !have_go() {
        return;
    }
    let (_d, project, run_dir) = prepared("go-buffer-pool");
    ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "alloc",
            "--iterations",
            "3",
        ],
    );
    let c = captures(&run_dir);
    assert_eq!(files_ending(&c, ".heap.pb.gz").len(), 4, "{c}");
    assert!(files_ending(&c, "goroutine.txt").is_empty());
    assert!(files_ending(&c, "goroutineleak.txt").is_empty());
    // No goroutine capture was asked for: no coverage decision about the leak profile.
    assert!(coverage_row(&run_dir).is_null());
}

#[test]
fn without_a_newer_toolchain_the_leak_profile_is_a_named_gap_and_the_run_goes_on() {
    if !have_go() {
        return;
    }
    let (_d, project, run_dir) = prepared("go-subscription-workers");
    // An empty module cache has no go1.27 toolchain to switch to.
    let cache = tempfile::tempdir().unwrap();
    let cache_s = cache.path().to_str().unwrap();
    let q = dla_probe::go_toolchain::query(&project, "go", Duration::from_secs(30));
    let version = q
        .version_text
        .as_deref()
        .and_then(dla_probe::go_toolchain::parse_version);
    if dla_probe::go_toolchain::decide(version, &[]) != dla_probe::go_toolchain::Leak::Missing {
        eprintln!("skipping: this Go ({version:?}) has the goroutineleak profile itself");
        return;
    }
    let o = dla_with(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "nonheap",
            "--iterations",
            "3",
        ],
        &[("GOMODCACHE", cache_s)],
    );
    // The project has no dependencies, so building against an empty cache works.
    assert_eq!(o.code, 0, "{}", o.text);
    let c = captures(&run_dir);
    assert_eq!(files_ending(&c, ".goroutine.txt").len(), 4, "{c}");
    assert!(files_ending(&c, ".goroutineleak.txt").is_empty());
    let row = coverage_row(&run_dir);
    assert_eq!(row["status"], "missing", "{row}");
    assert_eq!(row["install_hint"], "dla install go-toolchain-1.27");
    assert!(
        c["limitations"]
            .to_string()
            .contains("goroutineleak: missing")
    );
    // Printed commands never include a leak command, and the dumps still do.
    assert!(
        c["summarize"].to_string().contains("go-1.goroutine.txt"),
        "{c}"
    );
    assert!(!c["summarize"].to_string().contains("goroutineleak.txt"));
}
