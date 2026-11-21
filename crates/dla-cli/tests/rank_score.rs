//! The order of candidates through the binary: `dla ledger list` prints each row's score and
//! the three signals that matter most, in descending score, and `dla harness gen --next`
//! brings the holders of one owner into the same batch.
//!
//! The project: ten modules each with a module-level dict that only `main` fills (start-up
//! only, so a low score), and a `Registry` class with a field a public method fills with its
//! caller's key (a high score) and a field only its constructor fills (a low one).

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn project(dir: &Path) {
    write(dir, "shop/__init__.py", "");
    for i in 0..10 {
        write(
            dir,
            &format!("shop/m{i}.py"),
            &format!("_C{i} = {{}}\n\n\ndef main(key):\n    _C{i}[key] = 1\n"),
        );
    }
    write(
        dir,
        "shop/reg.py",
        "class Registry:\n    def __init__(self):\n        self._hot = {}\n        self._cold = {}\n        self._cold['boot'] = 1\n\n    def add(self, key, value):\n        self._hot[key] = value\n",
    );
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
    c
}

struct Run {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

impl Run {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path());
        let o = command(dir.path())
            .env("DLA_CONSENT_TIERS", "1,2,3")
            .args(["run", "init", "--session-id", "t", "--budget", "3h"])
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(v["ok"], true, "{v}");
        let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
        let r = Self { dir, run_dir };
        for a in [
            &["sweep", "--lang", "python"][..],
            &["census", "--lang", "python"][..],
            &["workload", "discover", "--lang", "python"][..],
        ] {
            r.ok(a);
        }
        r
    }

    fn ok(&self, args: &[&str]) -> Value {
        let o = command(self.dir.path()).args(args).output().unwrap();
        assert!(o.status.success(), "{args:?}: {}", String::from_utf8_lossy(&o.stdout));
        serde_json::from_slice(&o.stdout).unwrap_or(Value::Null)
    }

    fn list(&self) -> Vec<Value> {
        let o = command(self.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        text.lines()
            .skip(1)
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }
}

#[test]
fn ledger_list_prints_the_score_and_why_in_descending_order() {
    let r = Run::new();
    let rows = r.list();
    let ranked: Vec<&Value> = rows.iter().filter(|l| l["rank"].is_number()).collect();
    assert!(ranked.len() >= 12, "{rows:?}");
    let scores: Vec<i64> = ranked.iter().map(|l| l["score"].as_i64().unwrap()).collect();
    assert!(scores.windows(2).all(|w| w[0] >= w[1]), "{scores:?}");
    // The field a public method fills with its caller's key is first.
    assert_eq!(ranked[0]["site"], "shop/reg.py:3", "{ranked:?}");
    let why = ranked[0]["why"].as_array().unwrap();
    assert_eq!(why.len(), 3, "{why:?}");
    for w in why {
        let w = w.as_str().unwrap();
        assert!(w.contains(' ') && (w.contains('+') || w.contains('-')), "{w}");
    }
    assert!(
        why.iter().any(|w| w.as_str().unwrap().starts_with("w_grow +")),
        "{why:?}"
    );
    // The field only the constructor fills is behind the ten module-level dicts that only
    // `main` fills with its caller's key.
    let cold = ranked.iter().position(|l| l["site"] == "shop/reg.py:4").unwrap();
    assert!(cold >= 8, "{cold}: {scores:?}");
    // The tier is printed as before.
    assert!(ranked[0]["tier"].is_number());
}

#[test]
fn next_brings_the_fields_of_one_struct_into_the_same_batch() {
    let r = Run::new();
    let out = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    let targets = out["summary"]["targets"].as_array().unwrap();
    // Eight by rank, and the field of the same class that rank put beyond them.
    assert_eq!(targets.len(), 9, "{out}");
    let cold = targets
        .iter()
        .find(|t| t["site"] == "shop/reg.py:4")
        .unwrap_or_else(|| panic!("{targets:?}"));
    let hot = targets.iter().find(|t| t["site"] == "shop/reg.py:3").unwrap();
    assert_eq!(cold["sibling_of"], hot["finding"], "{cold}");
    assert!(hot["sibling_of"].is_null());
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/harness.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["sibling_of"][cold["finding"].as_str().unwrap()], hot["finding"]);
    assert_eq!(manifest["targets"].as_array().unwrap().len(), 9);
    let driver = fs::read_to_string(
        r.run_dir.join("harness/H1/driver/dla_harness.py"),
    )
    .unwrap();
    assert!(
        driver.contains(&format!(
            "(sibling_of {}: state of the same owner)",
            hot["finding"].as_str().unwrap()
        )),
        "{driver}"
    );
    // The status and the queue's next batch show the same nine.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["next"].as_array().unwrap().len(), 0, "{st}");
}

#[test]
fn next_with_a_count_takes_that_many_by_rank_and_the_siblings_of_them() {
    let r = Run::new();
    let out = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next", "1"]);
    let targets = out["summary"]["targets"].as_array().unwrap();
    // F1 by rank is the hot field; its sibling is the cold one, whatever its rank.
    assert_eq!(targets.len(), 2, "{out}");
    assert_eq!(targets[0]["site"], "shop/reg.py:3");
    assert_eq!(targets[1]["site"], "shop/reg.py:4");
    assert_eq!(targets[1]["sibling_of"], targets[0]["finding"]);
}
