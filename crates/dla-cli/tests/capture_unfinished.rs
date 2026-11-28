//! A shim dump with no exit record is a capture that is still being written, or whose process
//! was cut. `dla summarize` leaves it out with the sentence that says to wait for the harness
//! (when other captures are complete), or refuses the call (when it is all there is): a summary
//! of captures that are still arriving says nothing of the run.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

struct Project {
    dir: tempfile::TempDir,
    run_id: String,
}

impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let o = dla(dir.path())
            .args(["run", "init", "--session-id", "t"])
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        let run_id = v["run_id"].as_str().unwrap().to_owned();
        Self { dir, run_id }
    }

    /// Writes `harness/H4/out/<label>/dump.jsonl` under the run and returns its project path.
    fn dump(&self, label: &str, text: &str) -> String {
        let rel = format!(".dla/{}/harness/H4/out/{label}/dump.jsonl", self.run_id);
        let p = self.dir.path().join(&rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
        rel
    }

    fn summarize(&self, files: &[String]) -> (i32, Value) {
        let mut args = vec![
            "summarize",
            "type-histogram-diff",
            "--backend",
            "shim-stack-groups",
        ];
        args.extend(files.iter().map(String::as_str));
        let o = dla(self.dir.path()).args(&args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        (
            o.status.code().unwrap(),
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}")),
        )
    }

    fn ledger_text(&self) -> String {
        fs::read_to_string(
            self.dir
                .path()
                .join(".dla")
                .join(&self.run_id)
                .join("ledger.jsonl"),
        )
        .unwrap()
    }
}

/// The exit record of a shim process with two stacks.
fn exit_record(x: u64, a: u64, b: u64) -> String {
    format!(
        "{{\"dla_harness\":1,\"phase\":\"exit\",\"pid\":1,\"argv\":[\"x\"],\"exe\":\"/nonexistent/x\",\"runtime\":\"native\",\"x\":{x},\"t_ms\":1,\"gc_forced\":false,\"values\":{{}},\"groups\":{{}},\"raw_sites\":{{\"0x1\":{{\"live_bytes\":{a},\"live_blocks\":1,\"allocs\":1,\"frames\":[{{\"a\":\"0x10\",\"m\":0,\"o\":\"0x10\"}}]}},\"0x2\":{{\"live_bytes\":{b},\"live_blocks\":1,\"allocs\":1,\"frames\":[{{\"a\":\"0x20\",\"m\":0,\"o\":\"0x20\"}}]}}}},\"modules\":[{{\"path\":\"/nonexistent/x\",\"bias\":\"0x0\"}}]}}\n"
    )
}

/// What a dump holds while its process still runs: the driver's checkpoints and a line cut in the
/// middle, no exit record.
const UNFINISHED: &str = "{\"dla_harness\":1,\"phase\":\"checkpoint\",\"runtime\":\"rust\",\"x\":4000}\n{\"dla_harness\":1,\"phase\":\"checkpoint\",\"runtime\":\"rust\",\"x\":8000}\n{\"dla_harness\":1,\"phase\":\"exit\",\"pid\":1,\"runtime\":\"nat";

const WAIT: &str = "has no exit record: it is still being written or its process was cut; wait for `dla harness run H4` to end (dla run status), then summarize every capture it lists";

#[test]
fn a_capture_with_no_exit_record_is_named_and_left_out_of_a_summary_of_complete_ones() {
    let p = Project::new();
    let mut files: Vec<String> = [(1000, 10, 5), (2000, 20, 5), (4000, 40, 5)]
        .iter()
        .enumerate()
        .map(|(i, (x, a, b))| p.dump(&format!("p{}-alloc", i + 1), &exit_record(*x, *a, *b)))
        .collect();
    files.push(p.dump("p4-alloc", UNFINISHED));
    let (code, v) = p.summarize(&files);
    assert_eq!(code, 0, "{v}");
    let said = v["left_out"].to_string();
    assert!(
        said.contains(&format!("capture p4-alloc {WAIT}")),
        "the unfinished capture is named with the wait text: {said}"
    );
    assert!(!said.contains("p1-alloc"), "{said}");
    // The summary read the three complete captures, and its evidence says what was left out.
    let ledger = p.ledger_text();
    assert!(
        ledger.contains("capture p4-alloc has no exit record"),
        "the limitation is in the evidence: {ledger}"
    );
    assert!(
        !ledger.contains("p4-alloc/dump.jsonl\""),
        "the unfinished dump is no input of the summary"
    );
}

#[test]
fn a_call_over_only_unfinished_captures_is_refused_with_the_wait_text() {
    let p = Project::new();
    let files = vec![p.dump("p4-alloc", UNFINISHED)];
    let (code, v) = p.summarize(&files);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "capture-incomplete", "{v}");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains(&format!("capture p4-alloc {WAIT}")), "{msg}");
}

#[test]
fn complete_captures_alone_are_read_as_before() {
    let p = Project::new();
    let files: Vec<String> = [(1000, 10, 5), (2000, 20, 5), (4000, 40, 5)]
        .iter()
        .enumerate()
        .map(|(i, (x, a, b))| p.dump(&format!("p{}-alloc", i + 1), &exit_record(*x, *a, *b)))
        .collect();
    let (code, v) = p.summarize(&files);
    assert_eq!(code, 0, "{v}");
    assert!(v.get("left_out").is_none(), "{v}");
}

/// The `dump.jsonl` of a plain point of a Rust or native driver holds its checkpoints only: no
/// shim runs in that process, so it never gets an exit record, and it is complete as it is.
#[test]
fn a_plain_point_of_a_driver_is_never_called_unfinished() {
    let p = Project::new();
    let checkpoints = "{\"dla_harness\":1,\"phase\":\"checkpoint\",\"pid\":7,\"runtime\":\"rust\",\"x\":1000,\"t_ms\":1,\"gc_forced\":false,\"values\":{},\"groups\":{}}\n{\"dla_harness\":1,\"phase\":\"checkpoint\",\"pid\":7,\"runtime\":\"rust\",\"x\":2000,\"t_ms\":2,\"gc_forced\":false,\"values\":{},\"groups\":{}}\n";
    let mut files: Vec<String> = [(1000, 10, 5), (2000, 20, 5), (4000, 40, 5)]
        .iter()
        .enumerate()
        .map(|(i, (x, a, b))| p.dump(&format!("p{}-alloc", i + 1), &exit_record(*x, *a, *b)))
        .collect();
    files.push(p.dump("p1-fresh", checkpoints));
    let (_, v) = p.summarize(&files);
    assert!(
        !v.to_string().contains("has no exit record"),
        "a plain point is not a capture still being written: {v}"
    );
    let (_, v) = p.summarize(&[p.dump("p2-fresh", checkpoints)]);
    assert_ne!(v["error"]["code"], "capture-incomplete", "{v}");
    assert!(!v.to_string().contains("has no exit record"), "{v}");
}
