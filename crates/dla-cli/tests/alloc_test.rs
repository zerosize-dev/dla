//! `dla alloc-test` through the binary on copies of the Python benchmark pair: the leak
//! (`python-report-memo`) favors leak and ends confirmed through the commands it prints;
//! its bounded twin favors non-leak and the candidate is refuted with the discriminator.
//! Both skip, printing the reason, when python3 is missing.

use std::{
    fs,
    path::{Path, PathBuf},
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

fn on_path(tool: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(tool).is_file()))
}

/// True (after printing why) when the test cannot run here.
fn skip_without_python() -> bool {
    if on_path("python3") {
        return false;
    }
    eprintln!("skipping: python3 is not on PATH (alloc-test runs the Python harness)");
    true
}

struct Out {
    code: i32,
    json: Value,
    text: String,
}

fn dla(project: &Path, args: &[&str]) -> Out {
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
    ] {
        c.env_remove(var);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    assert!(
        text.len() <= dla_core::MAX_OUTPUT_BYTES,
        "output over the bound: {} bytes",
        text.len()
    );
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    assert_eq!(o.json["ok"], true, "{}", o.text);
    o.json
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let o = dla(project, args);
    assert_eq!(o.code, 2, "{args:?}: {}", o.text);
    let e = &o.json["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(
            e[k].as_str().is_some_and(|s| !s.is_empty()),
            "no {k}: {}",
            o.text
        );
    }
    e.clone()
}

/// A project copy with discovery and one harness run (S1, trended) done.
struct Prepared {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
    /// The workload evidence of the harness run.
    workload: String,
}

fn prepared(fixture: &str) -> Prepared {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    copy_dir(&fixtures_root().join(fixture).join("project"), &project);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "python"]);
    ok(&project, &["workload", "discover"]);
    ok(
        &project,
        &["harness", "gen", "--lang", "python", "--step", "150"],
    );
    let run = ok(&project, &["harness", "run", "H1"]);
    let workload = run["evidence"][0].as_str().unwrap().to_owned();
    ok(&project, &["trend", "S1"]);
    Prepared {
        _dir: dir,
        project,
        run_dir,
        workload,
    }
}

/// The finding raised at `site`.
fn finding_at(run_dir: &Path, site: &str) -> String {
    let text = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["type"] == "finding_added" && v["finding"]["site"] == site)
        .find_map(|v| v["finding"]["id"].as_str().map(str::to_owned))
        .unwrap_or_else(|| panic!("no finding at {site}"))
}

/// The ledger command a `next` entry names, with its placeholders filled.
fn command_of(next: &str) -> Option<Vec<String>> {
    let cmd = next.split("   #").next()?.split("   (").next()?.trim();
    let words: Vec<String> = cmd
        .strip_prefix("dla ")?
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    if words.first().map(String::as_str) != Some("ledger")
        || !matches!(
            words.get(1).map(String::as_str),
            Some("test" | "refute" | "confirm" | "annotate")
        )
    {
        return None;
    }
    Some(
        words
            .into_iter()
            .map(|w| {
                if w == "<holder>" {
                    "static/global".to_owned()
                } else {
                    w
                }
            })
            .collect(),
    )
}

fn run_next(project: &Path, next: &Value, verbs: &[&str]) -> Vec<String> {
    let mut done = Vec::new();
    for n in next.as_array().unwrap() {
        let Some(words) = command_of(n.as_str().unwrap()) else {
            continue;
        };
        if !verbs.contains(&words[1].as_str()) {
            continue;
        }
        let args: Vec<&str> = words.iter().map(String::as_str).collect();
        ok(project, &args);
        done.push(words[1].clone());
    }
    done
}

fn state_of(run_dir: &Path, finding: &str) -> String {
    let text = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["type"] == "transition" && v["finding"] == finding)
        .filter_map(|v| v["to"].as_str().map(str::to_owned))
        .next_back()
        .unwrap_or_else(|| "candidate".to_owned())
}

#[test]
fn the_leak_favors_leak_and_is_confirmed_with_the_printed_commands() {
    if skip_without_python() {
        return;
    }
    let p = prepared("python-report-memo");
    let f = finding_at(&p.run_dir, "reports/builder.py:14");
    // The non-leak sibling the ledger requires before the finding can be tested.
    ok(
        &p.project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            &p.workload,
        ],
    );

    let a = ok(&p.project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "leak", "{a}");
    assert_eq!(a["supports"][0], "leak");
    assert_eq!(a["finding"], f.as_str());
    assert_eq!(a["command"], "alloc-test");
    let (series, trend, evidence) = (
        a["series"].as_str().unwrap(),
        a["trend"].as_str().unwrap(),
        a["evidence"].as_str().unwrap(),
    );
    assert_ne!(series, "S1");
    assert!(p.run_dir.join(format!("series/{series}.json")).is_file());
    assert!(p.run_dir.join(format!("trend/{trend}.json")).is_file());
    let columns = a["columns"].as_array().unwrap();
    assert!(
        columns
            .iter()
            .any(|c| c["name"].as_str().unwrap().starts_with("probe:")
                && c["plateau"] == "growth-continues"),
        "{a}"
    );

    // The series is an alloc-test series with the purge event on its last point.
    let s: Value = serde_json::from_str(
        &fs::read_to_string(p.run_dir.join(format!("series/{series}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(s["source"]["kind"], "alloc-test");
    assert!(
        s["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["kind"] == "purge:trim")
    );
    assert_eq!(s["points"].as_array().unwrap().len(), 1 + 8 + 8);

    // The evidence is a bounded discriminator with the full numbers in alloc/A1.json.
    let ledger = fs::read_to_string(p.run_dir.join("ledger.jsonl")).unwrap();
    let rec: Value = ledger
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|v| v["evidence_id"] == evidence)
        .unwrap();
    assert_eq!(rec["kind"], "discriminator");
    assert_eq!(rec["authority"], "test");
    assert_eq!(rec["raw"], "alloc/A1.json");
    assert_eq!(rec["result"]["test"], "plateau+purge");
    assert_eq!(rec["result"]["favors"], "leak");
    assert_eq!(rec["result"]["finding"], f.as_str());
    assert!(serde_json::to_string(&rec["result"]).unwrap().len() <= 2048);
    let full: Value =
        serde_json::from_str(&fs::read_to_string(p.run_dir.join("alloc/A1.json")).unwrap())
            .unwrap();
    assert!(full["columns"].as_array().unwrap().len() >= columns.len());

    // Refute the sibling, test and confirm with the commands alloc-test printed.
    let done = run_next(
        &p.project,
        &a["next"],
        &["refute", "test", "annotate", "confirm"],
    );
    assert!(
        done.contains(&"refute".to_owned()),
        "{done:?} {}",
        a["next"]
    );
    assert!(done.contains(&"confirm".to_owned()), "{done:?}");
    assert_eq!(state_of(&p.run_dir, &f), "confirmed");
    // The ledger agrees the sibling is out.
    assert_eq!(state_of(&p.run_dir, &format!("{f}.n1")), "refuted");
}

#[test]
fn the_bounded_twin_favors_non_leak_and_the_candidate_is_refuted() {
    if skip_without_python() {
        return;
    }
    let p = prepared("python-report-memo-bounded");
    let f = finding_at(&p.run_dir, "reports/builder.py:14");
    // The hypothesis the test is about: a cache that only warms up.
    ok(
        &p.project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            &p.workload,
        ],
    );
    let a = ok(&p.project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "non-leak", "{a}");
    let supports: Vec<&str> = a["supports"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        supports.iter().all(|s| s.starts_with("N-")) && supports.contains(&"N-WARM"),
        "{supports:?}"
    );
    assert!(
        a["columns"].as_array().unwrap().iter().any(|c| c["name"]
            .as_str()
            .unwrap()
            .starts_with("probe:")
            && c["favors"] == "non-leak"),
        "{a}"
    );

    // The printed commands refute the leak-side finding and confirm the sibling.
    let done = run_next(&p.project, &a["next"], &["refute", "test", "confirm"]);
    assert_eq!(done, ["refute", "test", "confirm"], "{}", a["next"]);
    assert_eq!(state_of(&p.run_dir, &f), "refuted");
    assert_eq!(state_of(&p.run_dir, &format!("{f}.n1")), "confirmed");
}

#[test]
fn refusals_name_the_missing_item_and_the_fixing_command() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path();
    ok(project, &["run", "init", "--session-id", "t"]);
    // No harness yet.
    let e = refused(project, &["alloc-test", "H1"]);
    assert!(
        e["fix"].as_str().unwrap().contains("dla harness gen"),
        "{e}"
    );
    // Not a harness ID.
    let o = dla(project, &["alloc-test", "X1"]);
    assert_eq!(o.code, 1, "{}", o.text);
    // An unknown purge kind is a usage error from the parser.
    let o = dla(project, &["alloc-test", "H1", "--purge", "shake"]);
    assert_ne!(o.code, 0);
}
