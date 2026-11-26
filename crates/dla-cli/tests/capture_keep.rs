//! The size limit of the large captures of a run (`DLA_CAPTURE_KEEP_MB`) and the free-disk check
//! before a capture (`DLA_DISK_FLOOR_MB`, `DLA_FREE_DISK_BYTES`), through the binary.
//!
//! The sets of other harnesses are faked: `captures.json` files and sparse dump files of chosen
//! sizes under a real run directory, with the summary file a `dla summarize heap-graph` leaves
//! for the sets a summary read. The capture that triggers the limit is a real Node run (those
//! tests skip, printing the reason, when `node` is not installed).
#![cfg(unix)]

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const MB: u64 = 1024 * 1024;

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

fn have_node() -> bool {
    let ok = Std::new("node")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipping: node is not on PATH");
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
        "DLA_CAPTURE_KEEP_MB",
        "DLA_DISK_FLOOR_MB",
        "DLA_FREE_DISK_BYTES",
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

/// A project copy of the Node fixture with a run, consent for tiers 1 and 2, discovery and one
/// generated command-mode harness (`H1`).
fn prepared() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    copy_dir(
        &fixtures_root()
            .join("node-socket-heartbeat")
            .join("project"),
        &project,
    );
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "node"]);
    ok(&project, &["workload", "discover", "--lang", "node"]);
    ok(&project, &["harness", "gen", "--lang", "node"]);
    (dir, project, run_dir)
}

/// What a faked set looks like.
struct Fake<'a> {
    harness: &'a str,
    label: &'a str,
    /// Dump files of the set, each `mb` megabytes (sparse on disk).
    files: u32,
    mb: u64,
    captured_at: &'a str,
    /// A summary read the set: the per-snapshot summary file of each dump is there.
    read: bool,
}

fn sha_of(path: &Path) -> String {
    hex::encode(Sha256::digest(fs::read(path).unwrap()))
}

/// Writes the files of `f` and the `captures.json` of its harness; returns the run-relative
/// paths and digests.
fn fake_set(run_dir: &Path, f: &Fake<'_>) -> Vec<(String, String)> {
    let hdir = run_dir.join("harness").join(f.harness);
    let mut files = Vec::new();
    let mut listed = Vec::new();
    for i in 1..=f.files {
        let rel = format!(
            "harness/{}/out/{}/capture/100/heap-{i}.heapsnapshot",
            f.harness, f.label
        );
        let abs = run_dir.join(&rel);
        fs::create_dir_all(abs.parent().unwrap()).unwrap();
        // Sparse: no disk is used, the first bytes make the digest of each file its own.
        let mut file = fs::File::create(&abs).unwrap();
        file.set_len(f.mb * MB).unwrap();
        file.write_all(format!("{}-{}-{i}", f.harness, f.label).as_bytes())
            .unwrap();
        drop(file);
        let digest = sha_of(&abs);
        if f.read {
            let dir = run_dir.join("summaries");
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join(format!("heap-{i}-{}.json", &digest[..8])), "{}").unwrap();
        }
        listed.push(json!({
            "kind": "heap", "path": rel, "label": f.label, "seq": i, "x": f64::from(i) * 100.0,
            "bytes": f.mb * MB, "captured_at": f.captured_at, "sha256": digest,
        }));
        files.push((rel, digest));
    }
    let command = format!(
        "dla summarize heap-graph {}",
        files
            .iter()
            .map(|(r, _)| format!(
                ".dla/{}/{r}",
                run_dir.file_name().unwrap().to_string_lossy()
            ))
            .collect::<Vec<_>>()
            .join(" ")
    );
    fs::create_dir_all(&hdir).unwrap();
    fs::write(
        hdir.join("captures.json"),
        serde_json::to_string_pretty(&json!({
            "files": listed, "summarize": [command], "limitations": [],
        }))
        .unwrap(),
    )
    .unwrap();
    files
}

fn captures_of(run_dir: &Path, harness: &str) -> Vec<Value> {
    let text =
        fs::read_to_string(run_dir.join("harness").join(harness).join("captures.json")).unwrap();
    serde_json::from_str::<Value>(&text).unwrap()["files"]
        .as_array()
        .unwrap()
        .clone()
}

fn attention(out: &Value) -> String {
    out["_attention"]
        .as_array()
        .or_else(|| out["summary"]["_attention"].as_array())
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

const OLD: &str = "2026-01-01T00:00:0";

#[test]
fn the_oldest_read_sets_go_until_the_total_fits_and_stay_listed_as_pruned() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared();
    let mut digests = Vec::new();
    for (i, h) in ["H5", "H6", "H7"].iter().enumerate() {
        digests.push(fake_set(
            &run_dir,
            &Fake {
                harness: h,
                label: "p1-fresh",
                files: 2,
                mb: 20,
                captured_at: &format!("{OLD}{}Z", i + 1),
                read: true,
            },
        ));
    }
    // 3 x 40 MB of earlier sets, and the run writes about 70 MB: limit 130 leaves room for
    // the newest earlier set and the run's own.
    let o = dla_with(
        &project,
        &["harness", "run", "H1", "--capture", "heap", "--dumps", "3"],
        &[("DLA_CAPTURE_KEEP_MB", "130")],
    );
    assert_eq!(o.code, 0, "{}", o.text);
    let s = &o.json["summary"];
    let pruned = s["pruned_captures"].as_array().unwrap();
    let names: Vec<(&str, &str)> = pruned
        .iter()
        .map(|p| (p["harness"].as_str().unwrap(), p["label"].as_str().unwrap()))
        .collect();
    assert_eq!(names, [("H5", "p1-fresh"), ("H6", "p1-fresh")], "{s}");
    assert_eq!(pruned[0]["files"], 2);
    assert_eq!(pruned[0]["bytes"], 40 * MB);
    let line = attention(&o.json);
    assert!(
        line.contains("deleted the heap dumps of H5 p1-fresh (40 MB)")
            && line.contains("H6 p1-fresh")
            && line.contains("to stay under DLA_CAPTURE_KEEP_MB=130"),
        "{line}"
    );
    // The files of the two oldest are gone, the newest earlier set and the run's own stay.
    for (h, (rel, _)) in [("H5", &digests[0][0]), ("H6", &digests[1][0])] {
        assert!(!run_dir.join(rel).exists(), "{h} should be deleted");
    }
    assert!(run_dir.join(&digests[2][0].0).is_file());
    // The list keeps the entries with their digests, marked pruned.
    for (i, h) in ["H5", "H6"].iter().enumerate() {
        let files = captures_of(&run_dir, h);
        assert_eq!(files.len(), 2);
        for (f, (_, digest)) in files.iter().zip(&digests[i]) {
            assert_eq!(f["pruned"], true, "{f}");
            assert_eq!(f["sha256"].as_str().unwrap(), digest);
        }
    }
    assert!(
        captures_of(&run_dir, "H7")
            .iter()
            .all(|f| f.get("pruned").is_none())
    );
    // The run's own set is whole, with digests that match the files.
    let own = captures_of(&run_dir, "H1");
    let heaps: Vec<&Value> = own.iter().filter(|f| f["kind"] == "heap").collect();
    assert_eq!(heaps.len(), 3);
    for f in &heaps {
        assert!(f.get("pruned").is_none());
        let abs = run_dir.join(f["path"].as_str().unwrap());
        assert_eq!(f["sha256"].as_str().unwrap(), sha_of(&abs), "{f}");
    }
    // The manifest of a deleted set's harness is not needed for the list; the raw run output
    // says what went.
    let raw: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("harness/H1/run-S1.json")).unwrap())
            .unwrap();
    assert_eq!(raw["pruned_captures"].as_array().unwrap().len(), 2);
}

#[test]
fn a_set_nothing_read_is_never_deleted_even_when_the_total_stays_over_the_limit() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared();
    // H6 is the oldest and nothing read it; H5 and H7 were read.
    let mut sets = Vec::new();
    for (h, at, read) in [("H5", "2", true), ("H6", "1", false), ("H7", "3", true)] {
        sets.push(fake_set(
            &run_dir,
            &Fake {
                harness: h,
                label: "p1-fresh",
                files: 1,
                mb: 40,
                captured_at: &format!("{OLD}{at}Z"),
                read,
            },
        ));
    }
    let o = dla_with(
        &project,
        &["harness", "run", "H1", "--capture", "heap", "--dumps", "3"],
        &[("DLA_CAPTURE_KEEP_MB", "1")],
    );
    assert_eq!(o.code, 0, "{}", o.text);
    let names: Vec<&str> = o.json["summary"]["pruned_captures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["harness"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["H5", "H7"], "{}", o.text);
    assert!(
        run_dir.join(&sets[1][0].0).is_file(),
        "the unread set stays"
    );
    assert!(!run_dir.join(&sets[0][0].0).exists());
    assert!(!run_dir.join(&sets[2][0].0).exists());
    assert!(
        captures_of(&run_dir, "H6")
            .iter()
            .all(|f| f.get("pruned").is_none())
    );
    // The run's own dumps stay whatever the limit says.
    let own = captures_of(&run_dir, "H1");
    assert!(
        own.iter()
            .filter(|f| f["kind"] == "heap")
            .all(|f| run_dir.join(f["path"].as_str().unwrap()).is_file())
    );
}

/// The model picks the dumps it summarizes: a set one of whose dumps a summary read counts as
/// read, and goes like a fully read one.
#[test]
fn a_set_of_which_a_summary_read_one_dump_is_read() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared();
    let partly = fake_set(
        &run_dir,
        &Fake {
            harness: "H5",
            label: "p1-fresh",
            files: 3,
            mb: 40,
            captured_at: &format!("{OLD}1Z"),
            read: false,
        },
    );
    // A per-snapshot summary of the first dump only.
    let dir = run_dir.join("summaries");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("heap-1-{}.json", &partly[0].1[..8])), "{}").unwrap();
    let o = dla_with(
        &project,
        &["harness", "run", "H1", "--capture", "heap", "--dumps", "3"],
        &[("DLA_CAPTURE_KEEP_MB", "1")],
    );
    assert_eq!(o.code, 0, "{}", o.text);
    let names: Vec<&str> = o.json["summary"]["pruned_captures"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["harness"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["H5"], "{}", o.text);
    for (rel, _) in &partly {
        assert!(!run_dir.join(rel).exists(), "{rel}");
    }
}

#[test]
fn a_run_under_the_limit_deletes_nothing() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared();
    let set = fake_set(
        &run_dir,
        &Fake {
            harness: "H5",
            label: "p1-fresh",
            files: 1,
            mb: 40,
            captured_at: "2026-01-01T00:00:01Z",
            read: true,
        },
    );
    let o = dla_with(
        &project,
        &["harness", "run", "H1", "--capture", "heap", "--dumps", "3"],
        &[("DLA_CAPTURE_KEEP_MB", "4096")],
    );
    assert_eq!(o.code, 0, "{}", o.text);
    assert!(o.json["summary"].get("pruned_captures").is_none());
    assert!(!o.text.contains("DLA_CAPTURE_KEEP_MB"), "{}", o.text);
    assert!(run_dir.join(&set[0].0).is_file());
}

#[test]
fn free_disk_under_the_floor_with_nothing_to_delete_refuses_before_any_process() {
    let (_d, project, run_dir) = prepared();
    // One earlier set of 40 MB dumps that nothing read: the largest dump is 40 MB, a run of the
    // default four dumps writes about 160 MB.
    let set = fake_set(
        &run_dir,
        &Fake {
            harness: "H5",
            label: "p1-fresh",
            files: 1,
            mb: 40,
            captured_at: "2026-01-01T00:00:01Z",
            read: false,
        },
    );
    let o = dla_with(
        &project,
        &["harness", "run", "H1", "--capture", "heap"],
        &[
            ("DLA_FREE_DISK_BYTES", &(250 * MB).to_string()),
            ("DLA_DISK_FLOOR_MB", "100"),
        ],
    );
    assert_eq!(o.code, 2, "{}", o.text);
    let e = &o.json["error"];
    assert_eq!(e["code"], "insufficient-resources", "{}", o.text);
    let message = e["message"].as_str().unwrap();
    assert!(message.contains("free disk is 250 MB"), "{message}");
    assert!(message.contains("about 160 MB"), "{message}");
    assert!(
        message.contains("H5 p1-fresh (40 MB)") && message.contains("no summary read them"),
        "{message}"
    );
    let fix = e["fix"].as_str().unwrap();
    // The command of the unread set, and the largest number of dumps that fits: (250 - 100) / 40.
    assert!(
        fix.contains("dla summarize heap-graph")
            && fix.contains("harness/H5/out/p1-fresh/capture/100/heap-1.heapsnapshot"),
        "{fix}"
    );
    assert!(
        fix.contains("dla harness run H1 --capture heap --dumps 3"),
        "{fix}"
    );
    // Nothing started: no build, no process directory, no series, and the unread set is whole.
    assert_eq!(
        fs::read_dir(run_dir.join("harness/H1/out"))
            .unwrap()
            .count(),
        0,
        "no point directory was made"
    );
    assert!(!run_dir.join("harness/H1/.built").exists());
    assert!(!run_dir.join("series").join("S1.json").exists());
    assert!(run_dir.join(&set[0].0).is_file());
}

#[test]
fn when_not_even_three_dumps_fit_the_fix_says_to_free_disk() {
    let (_d, project, run_dir) = prepared();
    fake_set(
        &run_dir,
        &Fake {
            harness: "H5",
            label: "p1-fresh",
            files: 1,
            mb: 40,
            captured_at: "2026-01-01T00:00:01Z",
            read: true,
        },
    );
    // Free disk is under the floor already, and the set that could go is the only one: it goes,
    // and the disk is still short.
    let o = dla_with(
        &project,
        &["harness", "run", "H1", "--capture", "heap"],
        &[
            ("DLA_FREE_DISK_BYTES", &(150 * MB).to_string()),
            ("DLA_DISK_FLOOR_MB", "100"),
        ],
    );
    assert_eq!(o.code, 2, "{}", o.text);
    let fix = o.json["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains("even 3 dumps do not fit") && fix.contains("free disk"),
        "{fix}"
    );
    // The read set was deleted to make room before the refusal.
    assert_eq!(captures_of(&run_dir, "H5")[0]["pruned"], true);
}

#[test]
fn a_set_a_summary_read_is_deleted_to_make_room_and_the_run_goes_on() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared();
    let set = fake_set(
        &run_dir,
        &Fake {
            harness: "H5",
            label: "p1-fresh",
            files: 1,
            mb: 40,
            captured_at: "2026-01-01T00:00:01Z",
            read: true,
        },
    );
    // Free 250 MB, the run would write 3 x 40 MB and keep 100 MB: short by 0, with the set: ok.
    // Without deleting, 250 - 120 = 130 >= 100 passes, so make the floor 200 to need the room.
    let o = dla_with(
        &project,
        &["harness", "run", "H1", "--capture", "heap", "--dumps", "3"],
        &[
            ("DLA_FREE_DISK_BYTES", &(300 * MB).to_string()),
            ("DLA_DISK_FLOOR_MB", "200"),
        ],
    );
    assert_eq!(o.code, 0, "{}", o.text);
    assert!(!run_dir.join(&set[0].0).exists());
    let pruned = &o.json["summary"]["pruned_captures"];
    assert_eq!(pruned[0]["harness"], "H5", "{}", o.text);
    assert!(attention(&o.json).contains("deleted the heap dumps of H5"));
}

#[test]
fn a_summarize_of_a_deleted_capture_is_refused_with_the_capture_command() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    copy_dir(
        &fixtures_root()
            .join("node-socket-heartbeat")
            .join("project"),
        &project,
    );
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_id = init["run_id"].as_str().unwrap().to_owned();
    let run_dir = project.join(".dla").join(&run_id);
    let mut paths = Vec::new();
    for i in 1..=3 {
        paths.push(format!(
            "harness/H5/out/p1-fresh/capture/100/heap-{i}.heapsnapshot"
        ));
    }
    let files: Vec<Value> = paths
        .iter()
        .enumerate()
        .map(|(i, p)| {
            json!({"kind": "heap", "path": p, "label": "p1-fresh", "seq": i + 1,
                   "bytes": 5, "captured_at": "2026-01-01T00:00:01Z",
                   "sha256": "ab".repeat(32), "pruned": true})
        })
        .collect();
    fs::create_dir_all(run_dir.join("harness/H5")).unwrap();
    fs::write(
        run_dir.join("harness/H5/captures.json"),
        json!({"files": files}).to_string(),
    )
    .unwrap();
    let given = format!(".dla/{run_id}/{}", paths[0]);
    let o = dla_with(&project, &["summarize", "heap-graph", &given], &[]);
    assert_eq!(o.code, 2, "{}", o.text);
    let message = o.json["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("was deleted") && message.contains("DLA_CAPTURE_KEEP_MB"),
        "{message}"
    );
    assert_eq!(
        o.json["error"]["fix"],
        "dla harness run H5 --mode fresh --capture heap --dumps 3"
    );
    // The run-relative spelling is refused the same way, and a file that was never listed is a
    // plain not-found.
    let o = dla_with(&project, &["summarize", "heap-graph", &paths[1]], &[]);
    assert!(
        o.json["error"]["fix"]
            .as_str()
            .is_some_and(|f| f.starts_with("dla harness run H5 ")),
        "{}",
        o.text
    );
    let o = dla_with(
        &project,
        &[
            "summarize",
            "heap-graph",
            "harness/H5/out/nothing.heapsnapshot",
        ],
        &[],
    );
    assert!(
        o.json["error"]["message"]
            .as_str()
            .is_some_and(|m| m.ends_with("is not a file")),
        "{}",
        o.text
    );
}

#[test]
fn a_real_capture_run_lists_the_digest_of_each_file_it_wrote() {
    if !have_node() {
        return;
    }
    let (_d, project, run_dir) = prepared();
    ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "heap,nonheap",
            "--dumps",
            "3",
        ],
    );
    let files = captures_of(&run_dir, "H1");
    assert!(files.len() >= 6, "{files:?}");
    for f in &files {
        let abs = run_dir.join(f["path"].as_str().unwrap());
        assert_eq!(f["sha256"].as_str().unwrap(), sha_of(&abs), "{f}");
    }
    // The manifest lists the same digests.
    let m: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("harness/H1/harness.json")).unwrap())
            .unwrap();
    for f in m["captures"].as_array().unwrap() {
        let abs = run_dir.join(f["path"].as_str().unwrap());
        assert_eq!(f["sha256"].as_str().unwrap(), sha_of(&abs));
    }
}
