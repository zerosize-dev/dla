//! A Python harness record that lost sections (probes, type counts, groups) is said in the run
//! output and in the series, and a target whose probe has no column reads `unknown`, not
//! `not_loaded` (which claims the code never loaded). The loss is forced here: the agent copy of
//! the harness writes a type count that is not a number (the typed reader drops the sections it
//! cannot read, as it does for any record it cannot parse whole) and cannot read the freeze site.
//! Skips, printing the reason, without python3.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

fn on_path(tool: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(tool).is_file()))
}

fn dla(project: &Path, args: &[&str]) -> Value {
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
    let o = c
        .args(args)
        .env("DLA_CONSENT_TIERS", "1-2")
        .output()
        .unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    assert_eq!(o.status.code(), Some(0), "{args:?}: {text}");
    v
}

fn project(dir: &Path) {
    fs::create_dir_all(dir.join("batcher")).unwrap();
    fs::write(dir.join("batcher/__init__.py"), "").unwrap();
    fs::write(
        dir.join("batcher/worker.py"),
        "import gc\n\n\nclass Step:\n    def __init__(self, parent, n):\n        self.parent = parent\n        self.n = n\n\n\nclass Job:\n    def __init__(self, n):\n        self.steps = [Step(self, k) for k in range(n)]\n\n\nclass Worker:\n    def run_batch(self, size):\n        jobs = [Job(8) for _ in range(size)]\n        total = sum(len(j.steps) for j in jobs)\n        self.after_batch()\n        return total\n\n    def after_batch(self):\n        gc.freeze()\n",
    )
    .unwrap();
    fs::write(
        dir.join("batcher/__main__.py"),
        "import argparse\n\nfrom .worker import Worker\n\n\ndef run(iterations):\n    worker = Worker()\n    total = 0\n    for _ in range(iterations // 25):\n        total += worker.run_batch(25)\n    return total\n\n\ndef main():\n    parser = argparse.ArgumentParser()\n    parser.add_argument('--iterations', type=int, default=2000)\n    print(run(parser.parse_args().iterations))\n\n\nif __name__ == '__main__':\n    main()\n",
    )
    .unwrap();
}

/// `trend`'s verdict counts of the targets that are not declared by the harness.
fn other(t: &Value, verdict: &str) -> u64 {
    t["other_targets"][verdict].as_u64().unwrap_or(0)
}

#[test]
fn a_reduced_record_is_said_and_its_target_reads_unknown_not_not_loaded() {
    if !on_path("python3") {
        eprintln!("skipping: python3 is not on PATH");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("project");
    project(&p);
    let init = dla(&p, &["run", "init", "--session-id", "t"]);
    let run_dir = p.join(".dla").join(init["run_id"].as_str().unwrap());
    dla(&p, &["census", "--lang", "python"]);
    dla(&p, &["workload", "discover"]);
    dla(&p, &["harness", "gen", "--lang", "python", "--step", "150"]);
    // Force the loss in the agent copy of this harness.
    let agent = run_dir.join("harness/H1/agent/sitecustomize.py");
    let text = fs::read_to_string(&agent).unwrap();
    let text = text
        .replace(
            "        rec[\"types\"] = types_full[:20]\n",
            "        rec[\"types\"] = types_full[:20]\n        rec[\"types\"][0][\"count\"] = \"many\"\n",
        )
        .replace(
            "def _creation_site(skip):\n",
            "def _creation_site(skip):\n    if skip == 2:\n        raise RuntimeError(\"forced\")\n",
        );
    assert!(text.contains("\"many\"") && text.contains("forced"));
    fs::write(&agent, text).unwrap();
    let run = dla(&p, &["harness", "run", "H1"]);
    let attention = run["_attention"].to_string();
    assert!(attention.contains("REDUCED RECORD"), "{run}");
    let series: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("series/S1.json")).unwrap()).unwrap();
    let limits = series["limitations"].to_string();
    assert!(
        limits.contains("reduced record: the record of point")
            && limits.contains("lost probes, types"),
        "{limits}"
    );
    // The freeze group survives the loss of its site: the count is kept under `<non-project>`.
    let columns = series["columns"].to_string();
    assert!(columns.contains("py.gc_frozen@<non-project>"), "{columns}");
    let t = dla(&p, &["trend", "S1"]);
    assert_eq!(other(&t, "unknown"), 1, "{t}");
    assert_eq!(other(&t, "not_loaded"), 0, "{t}");
}
