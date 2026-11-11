//! Separate `dla` processes appending to one ledger at the same time never repeat a
//! `seq` or an ID, and the ledger replays and stays sealed afterwards.

use std::{fs, path::Path, process::Command};

use assert_cmd::cargo::CommandCargoExt;
use serde_json::Value;

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

#[test]
fn parallel_processes_append_without_a_repeated_seq() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    let init = dla(p)
        .args(["run", "init", "--session-id", "t"])
        .output()
        .unwrap();
    let init: Value = serde_json::from_slice(&init.stdout).unwrap();
    let run_dir = p.join(".dla").join(init["run_id"].as_str().unwrap());
    let n = 14;
    let children: Vec<_> = (0..n)
        .map(|i| {
            dla(p)
                .args([
                    "ledger",
                    "add",
                    "--origin",
                    "census",
                    "--evidence",
                    "E1",
                    "--site",
                    &format!("src/File{i}.java:{}", i + 1),
                    "--holder",
                    "static/global",
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for c in children {
        let out = c.wait_with_output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
    let text = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    let seqs: Vec<u64> = text
        .lines()
        .map(|l| {
            serde_json::from_str::<Value>(l).unwrap()["seq"]
                .as_u64()
                .unwrap()
        })
        .collect();
    assert_eq!(seqs, (1..=seqs.len() as u64).collect::<Vec<_>>(), "{text}");
    assert!(seqs.len() > n, "every process appended");
    // A later command still replays the ledger and finds the seals in step.
    let st = dla(p).args(["run", "status"]).output().unwrap();
    let st: Value = serde_json::from_slice(&st.stdout).unwrap();
    assert_eq!(st["ok"], true, "{st}");
    assert_eq!(st["tamper"].as_array().map_or(0, Vec::len), 0, "{st}");
}
