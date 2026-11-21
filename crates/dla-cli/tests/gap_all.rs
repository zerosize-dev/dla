//! `dla gap --residual --all` writes the residual row of every series that has none, with what
//! `dla gap <S> --residual` writes for each.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn command(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1,2,3");
    c
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = command(project).args(args).output().unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "{args:?}: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    serde_json::from_slice(&o.stdout).unwrap()
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

fn residual_rows(project: &Path) -> Value {
    let dir = fs::read_dir(project.join(".dla"))
        .unwrap()
        .flatten()
        .find(|e| e.path().join("coverage.json").is_file())
        .unwrap()
        .path();
    let doc: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("coverage.json")).unwrap()).unwrap();
    doc["residual"].clone()
}

fn start(project: &Path) {
    write(project, "shop/__init__.py", "");
    write(
        project,
        "shop/cache.py",
        "import functools\n\n\n@functools.lru_cache(None)\ndef price_for(handler, key):\n    return (handler, key)\n",
    );
    ok(
        project,
        &["run", "init", "--session-id", "t", "--queue", "off"],
    );
    ok(project, &["sweep", "--lang", "python"]);
    ok(project, &["census", "--lang", "python"]);
    ok(project, &["workload", "discover", "--lang", "python"]);
    ok(project, &["harness", "gen", "--lang", "python", "--driver"]);
    let run_dir = fs::read_dir(project.join(".dla"))
        .unwrap()
        .flatten()
        .find(|e| e.path().join("harness").is_dir())
        .unwrap()
        .path();
    let p: PathBuf = run_dir.join("harness/H1/driver/dla_harness.py");
    let text = fs::read_to_string(&p).unwrap();
    fs::write(
        &p,
        text.replace(
            "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            "    from shop import cache\n    _state['cache'] = cache",
        )
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            "    _state['cache'].price_for('h', key)",
        ),
    )
    .unwrap();
}

#[test]
fn all_writes_the_rows_one_by_one_would_and_says_so_in_one_summary() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    start(&a);
    // Two runs of the harness: four series.
    ok(&a, &["harness", "run", "H1", "--mode", "both"]);
    ok(&a, &["harness", "run", "H1", "--mode", "both"]);
    let b = dir.path().join("b");
    copy_dir(&a, &b);

    let out = ok(&a, &["gap", "--residual", "--all"]);
    assert_eq!(out["all"], true, "{out}");
    let written = out["written"].as_u64().unwrap();
    assert_eq!(written, 4, "{out}");
    assert!(out.to_string().len() < 1500, "bounded: {out}");

    for s in ["S1", "S2", "S3", "S4"] {
        ok(&b, &["gap", s, "--residual"]);
    }
    let (ra, rb) = (residual_rows(&a), residual_rows(&b));
    assert_eq!(ra.as_array().unwrap().len(), rb.as_array().unwrap().len());
    assert_eq!(ra, rb);
    assert!(
        ra.as_array()
            .unwrap()
            .iter()
            .all(|r| r["series"].is_string())
    );

    // A second call has nothing left to write.
    let again = ok(&a, &["gap", "--residual", "--all"]);
    assert_eq!(again["written"], 0, "{again}");
    assert_eq!(again["already_had_a_row"], 4, "{again}");
}

#[test]
fn a_run_with_no_series_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("p");
    write(&p, "a.py", "X = {}\n");
    ok(&p, &["run", "init", "--session-id", "t", "--queue", "off"]);
    let out = ok(&p, &["gap", "--residual", "--all"]);
    assert_eq!(out["written"], 0, "{out}");
    assert_eq!(out["series"], serde_json::json!([]), "{out}");
}

#[test]
fn all_needs_residual_and_no_series() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("p");
    write(&p, "a.py", "X = {}\n");
    ok(&p, &["run", "init", "--session-id", "t", "--queue", "off"]);
    for args in [
        &["gap", "--all"][..],
        &["gap", "S1", "--residual", "--all"][..],
    ] {
        let o = command(&p).args(args).output().unwrap();
        assert_ne!(o.status.code(), Some(0), "{args:?}");
    }
}
