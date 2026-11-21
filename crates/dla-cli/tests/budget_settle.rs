//! An `auto` budget settles on its own once sweep and census have run: the first command that
//! needs the budget afterwards sizes it, once, with the formula of the gate.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

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

/// `settled` is left out of `state.json` while it is false.
fn settled(b: &Value) -> bool {
    b["settled"].as_bool().unwrap_or(false)
}

struct R {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

impl R {
    fn ok(&self, args: &[&str]) -> Value {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        assert!(
            matches!(o.status.code(), Some(0) | Some(2)),
            "{args:?}: {text}"
        );
        serde_json::from_str(&text).unwrap_or(Value::Null)
    }
    fn budget(&self) -> Value {
        let st: Value =
            serde_json::from_str(&fs::read_to_string(self.run_dir.join("state.json")).unwrap())
                .unwrap();
        st["budget"].clone()
    }
}

/// A run with `--budget auto` on a small Python project, before sweep and census.
fn start() -> R {
    let dir = tempfile::tempdir().unwrap();
    for (rel, text) in [
        ("shop/__init__.py", ""),
        (
            "shop/reg.py",
            "class Registry:\n    _by_key = {}\n\n    def put(self, key, value):\n        Registry._by_key[key] = value\n",
        ),
    ] {
        let p = dir.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let o = command(dir.path())
        .args(["run", "init", "--session-id", "t", "--budget", "auto"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    R { dir, run_dir }
}

#[test]
fn nothing_settles_before_sweep_and_census_have_run() {
    let r = start();
    r.ok(&["run", "status"]);
    r.ok(&["sweep", "--lang", "python"]);
    r.ok(&["run", "status"]);
    assert!(!settled(&r.budget()), "{}", r.budget());
    assert_eq!(r.budget()["seconds"], 3600);
}

#[test]
fn each_command_that_needs_the_budget_settles_it_once_after_sweep_and_census() {
    let cases: [&[&str]; 4] = [
        &["run", "status"],
        &["ledger", "close-rest"],
        &["harness", "gen", "--lang", "python"],
        &["gate", "check", "--stage", "intake"],
    ];
    for args in cases {
        let r = start();
        r.ok(&["sweep", "--lang", "python"]);
        r.ok(&["census", "--lang", "python"]);
        // The census itself is not a command that needs the budget.
        assert!(!settled(&r.budget()), "{args:?}: {}", r.budget());
        let out = r.ok(args);
        let b = r.budget();
        assert!(settled(&b), "{args:?}: {b} {out}");
        // The formula: 20 + 15 per runtime + 3 per candidate, between 60 and 180 minutes.
        let e = &b["estimate"];
        let (rt, c) = (
            e["runtimes"].as_u64().unwrap(),
            e["candidates"].as_u64().unwrap(),
        );
        assert_eq!(rt, 1, "{b}");
        let minutes = (20 + 15 * rt + 3 * c).clamp(60, 180);
        assert_eq!(e["total_minutes"], minutes, "{b}");
        assert_eq!(b["seconds"], minutes * 60, "{b}");
        // Once: a second command changes nothing.
        r.ok(&["run", "status"]);
        assert_eq!(r.budget(), b, "{args:?}");
    }
}
