//! Worker slices, `ledger done`, `ledger merge` and `ledger verify` through the binary.

use std::{
    fs,
    path::{Path, PathBuf},
};

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
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

struct Outcome {
    code: i32,
    stdout: String,
    json: Value,
}

fn run(project: &Path, args: &[&str]) -> Outcome {
    let out = dla(project).args(args).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.len() <= dla_core::MAX_OUTPUT_BYTES,
        "stdout is {} bytes: {stdout}",
        stdout.len()
    );
    let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    Outcome {
        code: out.status.code().unwrap(),
        stdout,
        json,
    }
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = run(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.stdout);
    assert_eq!(o.json["ok"], true);
    o.json
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let o = run(project, args);
    assert_eq!(o.code, 2, "{args:?}: {}", o.stdout);
    assert_eq!(o.json["ok"], false);
    let e = &o.json["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(
            e[k].as_str().is_some_and(|s| !s.is_empty()),
            "refusal lacks {k}: {}",
            o.stdout
        );
    }
    e.clone()
}

/// A project with a run and the given workers registered by `dla run dispatch`.
fn project(workers: &str) -> (tempfile::TempDir, String, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let init = ok(dir.path(), &["run", "init", "--session-id", "t"]);
    let id = init["run_id"].as_str().unwrap().to_owned();
    if !workers.is_empty() {
        ok(dir.path(), &["run", "dispatch", "--workers", workers]);
    }
    let run_dir = dir.path().join(".dla").join(&id);
    (dir, id, run_dir)
}

fn add(project: &Path, worker: Option<&str>, site: &str) -> Value {
    let mut args = Vec::new();
    if let Some(w) = worker {
        args.extend(["--worker", w]);
    }
    args.extend([
        "ledger",
        "add",
        "--origin",
        "census",
        "--evidence",
        "E1",
        "--site",
        site,
        "--holder",
        "static/global",
    ]);
    ok(project, &args)
}

fn state(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("state.json")).unwrap()).unwrap()
}

fn worker_status(run_dir: &Path, name: &str) -> String {
    state(run_dir)["workers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["name"] == name)
        .unwrap()["status"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn a_worker_writes_prefixed_ids_to_its_slice_and_the_core_merges_them() {
    let (dir, id, run_dir) = project("jvm,rust");
    let p = dir.path();

    // IDs are prefixed, the pointer names the slice, the main ledger is untouched.
    let main_before = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    let a = add(p, Some("jvm"), "src/Registry.java:42");
    assert_eq!(a["id"], "jvm/F1");
    assert_eq!(a["record"], format!(".dla/{id}/ledger.jvm.jsonl#L2"));
    assert_eq!(
        fs::read_to_string(run_dir.join("ledger.jvm.jsonl"))
            .unwrap()
            .lines()
            .count(),
        2
    );
    assert_eq!(
        fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap(),
        main_before
    );
    assert_eq!(worker_status(&run_dir, "jvm"), "running");
    assert_eq!(worker_status(&run_dir, "rust"), "registered");

    // A bare F1 is the worker's own: a sibling and a resolution through it.
    let s = ok(
        p,
        &[
            "--worker",
            "jvm",
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
            "--sibling-of",
            "F1",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues",
        ],
    );
    assert_eq!(s["id"], "jvm/F1.n1");
    assert_eq!(s["parent"], "jvm/F1");
    let r = ok(
        p,
        &[
            "--worker",
            "jvm",
            "ledger",
            "resolve",
            "F1",
            "--as",
            "unresolved",
            "--reason",
            "not_exercised",
            "--remedy",
            "later",
        ],
    );
    assert_eq!(r["id"], "jvm/F1");
    assert_eq!(r["state"], "unresolved(not_exercised)");
    // The environment variable works as the flag does.
    let out = dla(p)
        .env("DLA_WORKER", "rust")
        .args([
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "src/Registry.java:42",
            "--holder",
            "static/global",
        ])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], "rust/F1");

    // A worker cannot change the main ledger's findings, merge or verify.
    let e = refused(p, &["--worker", "jvm", "ledger", "merge"]);
    assert_eq!(e["code"], "core-only");
    assert!(e["fix"].as_str().unwrap().contains("ledger merge"));
    let e = refused(
        p,
        &[
            "--worker",
            "jvm",
            "ledger",
            "verify",
            "jvm/F1",
            "--site",
            "none",
            "--holder",
            "static/global",
            "--mechanism",
            "m",
            "--bundle",
            &"a".repeat(64),
        ],
    );
    assert_eq!(e["code"], "core-only");

    // done: refused without a worker, idempotent with one.
    let e = refused(p, &["ledger", "done"]);
    assert_eq!(e["code"], "worker-only");
    assert!(e["fix"].as_str().unwrap().contains("--worker"));
    let d = ok(p, &["--worker", "jvm", "ledger", "done"]);
    assert_eq!(d["open"], serde_json::json!(["jvm/F1.n1"]));
    assert_eq!(worker_status(&run_dir, "jvm"), "done");
    let again = ok(p, &["--worker", "jvm", "ledger", "done"]);
    assert_eq!(again["already_done"], true);
    // The slice is closed to further writes.
    let e = refused(
        p,
        &[
            "--worker",
            "jvm",
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
        ],
    );
    assert_eq!(e["code"], "slice-closed");
    assert!(
        !e["fix"].as_str().unwrap().contains("run dispatch"),
        "dispatch refuses a registered worker: {e}"
    );
    assert!(e["fix"].as_str().unwrap().contains("ledger merge"), "{e}");
    ok(p, &["--worker", "rust", "ledger", "done"]);

    // The core merges: both slices, one duplicate, nothing unfinished.
    let m = ok(p, &["ledger", "merge"]);
    assert_eq!(m["merged_lines"]["jvm"], 5);
    assert_eq!(m["merged_lines"]["rust"], 3);
    assert_eq!(m["unfinished"], serde_json::json!([]));
    assert_eq!(m["duplicates"][0]["canonical"], "jvm/F1");
    assert_eq!(m["duplicates"][0]["duplicate"], "rust/F1");
    assert_eq!(m["status"], "active");
    // And is idempotent.
    let m2 = ok(p, &["ledger", "merge"]);
    assert_eq!(m2["merged_lines"], serde_json::json!({}));
    assert_eq!(m2["duplicates"], serde_json::json!([]));
    let ledger = dla_core::Ledger::load(&run_dir).unwrap();
    assert_eq!(ledger.merged_lines("jvm"), 5);

    // The report shows the workers, the merge and the duplicate.
    let out = dla(p).arg("report").output().unwrap();
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.contains("## Workers"), "{md}");
    assert!(
        md.contains("- **jvm** (jvm): status done; 5 lines merged; slice done; torn tail: no"),
        "{md}"
    );
    assert!(md.contains("- Merged duplicates: rust/F1"), "{md}");
    assert!(md.contains("## Merged duplicates"), "{md}");
    assert!(!md.contains("### rust/F1"), "{md}");
}

#[test]
fn verify_agrees_or_opens_an_item_and_the_report_says_so() {
    let (dir, _id, run_dir) = project("");
    let p = dir.path();
    let f = add(p, None, "src/A.java:40");
    let fid = f["id"].as_str().unwrap().to_owned();
    ok(
        p,
        &["ledger", "annotate", &fid, "--holder", "static/global"],
    )
    .get("ok");
    let b = ok(p, &["bundle", &fid]);
    let bundle = b["manifest_sha256"].as_str().unwrap().to_owned();
    // A hash no bundle of this finding has is refused.
    let e = refused(
        p,
        &[
            "ledger",
            "verify",
            &fid,
            "--site",
            "none",
            "--holder",
            "static/global",
            "--mechanism",
            "m",
            "--bundle",
            &"c".repeat(64),
        ],
    );
    assert_eq!(e["code"], "unknown-bundle");
    assert!(e["fix"].as_str().unwrap().contains("dla bundle"), "{e}");
    let v = ok(
        p,
        &[
            "ledger",
            "verify",
            &fid,
            "--site",
            "src/A.java:42",
            "--holder",
            "static/global",
            "--mechanism",
            "listeners are never removed",
            "--bundle",
            &bundle,
        ],
    );
    assert_eq!(v["verified"], true);
    assert!(v.get("open_item").is_none());
    let v = ok(
        p,
        &[
            "ledger",
            "verify",
            &fid,
            "--site",
            "none",
            "--holder",
            "thread-local",
            "--mechanism",
            "a thread local",
            "--label",
            "UNCLASSIFIED",
            "--bundle",
            &bundle,
        ],
    );
    assert_eq!(v["verified"], false);
    assert_eq!(v["open_item"], "O1");
    let text = v["disagreement"].as_str().unwrap();
    assert!(
        text.contains("src/A.java:40") && text.contains("thread-local"),
        "{text}"
    );

    let out = dla(p).arg("report").output().unwrap();
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(
        md.contains("- Verified: DISAGREES (site: finding `src/A.java:40` vs verifier none"),
        "{md}"
    );
    assert!(
        dla_core::Ledger::load(&run_dir)
            .unwrap()
            .verifications(&"F1".parse().unwrap())
            .len()
            == 2
    );

    // Refusals name the missing item and the fix.
    let e = refused(
        p,
        &[
            "ledger",
            "verify",
            &fid,
            "--site",
            "src/A.java",
            "--holder",
            "static/global",
            "--mechanism",
            "m",
            "--bundle",
            &bundle,
        ],
    );
    assert_eq!(e["code"], "invalid-site");
    let e = refused(
        p,
        &[
            "ledger",
            "verify",
            &fid,
            "--site",
            "none",
            "--holder",
            "static/global",
            "--mechanism",
            "m",
            "--bundle",
            "not-a-hash",
        ],
    );
    assert_eq!(e["code"], "unknown-bundle");
    assert!(e["fix"].as_str().unwrap().contains("dla bundle"));
}

#[test]
fn an_artifact_run_refuses_a_site_on_add_and_annotate() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("dumps")).unwrap();
    fs::write(dir.path().join("dumps/a.hprof.gz"), b"x").unwrap();
    ok(
        dir.path(),
        &[
            "run",
            "init",
            "--session-id",
            "t",
            "--entry",
            "artifact",
            "--artifact",
            "dumps/a.hprof.gz",
        ],
    );
    let e = refused(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "src/A.java:1",
        ],
    );
    assert_eq!(e["code"], "site-forbidden");
    ok(
        dir.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--owner",
            "a.B.c",
        ],
    );
    let e = refused(
        dir.path(),
        &["ledger", "annotate", "F1", "--site", "src/A.java:1"],
    );
    assert_eq!(e["code"], "site-forbidden");
    assert!(e["fix"].as_str().unwrap().contains("--owner"));
    // The report's Entry line names the artifacts.
    let out = dla(dir.path()).arg("report").output().unwrap();
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.contains("Entry: artifact (`dumps/a.hprof.gz`)"), "{md}");
}

#[cfg(unix)]
#[test]
fn a_killed_worker_leaves_a_valid_merged_ledger_and_an_incomplete_run() {
    use std::{
        os::unix::process::CommandExt,
        process::{Command as Std, Stdio},
        time::{Duration, Instant},
    };

    let (dir, id, run_dir) = project("jvm,rust");
    let p = dir.path();
    ok(p, &["--worker", "rust", "ledger", "done"]);

    // A real loop of `dla --worker jvm` processes appending findings, in its own process
    // group so the whole loop (shell and the dla in flight) dies together.
    let script = r#"i=0; while :; do "$DLA_BIN" --project "$DLA_PROJECT" --run "$DLA_RUN_ID" --worker jvm ledger add --origin census --evidence E1 --site "src/A.java:$((i % 900 + 1))" --holder static/global >/dev/null; i=$((i+1)); done"#;
    let mut child = Std::new("sh")
        .arg("-c")
        .arg(script)
        .env("DLA_BIN", env!("CARGO_BIN_EXE_dla"))
        .env("DLA_PROJECT", p)
        .env("DLA_RUN_ID", &id)
        .env_remove("DLA_RUN")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .unwrap();
    let slice = run_dir.join("ledger.jvm.jsonl");
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let n = fs::read_to_string(&slice).map_or(0, |t| t.matches('\n').count());
        if n >= 25 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the worker loop wrote only {n} lines"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let killed = Std::new("kill")
        .args(["-KILL", "--", &format!("-{}", child.id())])
        .status()
        .unwrap();
    assert!(killed.success());
    child.wait().unwrap();

    // The slice replays: at most one line (the torn one) is lost.
    let text = fs::read_to_string(&slice).unwrap();
    let complete = text.matches('\n').count();
    let replayed = dla_core::Ledger::load_for(&run_dir, Some("jvm")).unwrap();
    assert_eq!(replayed.lines().len(), complete);
    let torn = usize::from(!text.ends_with('\n'));
    assert!(torn <= 1);
    assert_eq!(replayed.torn_tail().is_some(), torn == 1);

    // Merge: the slice is copied, the worker is unfinished, the run is incomplete.
    let m = ok(p, &["ledger", "merge"]);
    assert_eq!(m["merged_lines"]["jvm"], complete as u64);
    assert_eq!(m["status"], "incomplete");
    assert_eq!(m["unfinished"][0]["worker"], "jvm");
    assert_eq!(m["unfinished"][0]["status"], "killed");
    assert_eq!(m["unfinished"][0]["slice_lines"], complete as u64);
    assert_eq!(m.get("torn_tail").is_some(), torn == 1);

    let merged = dla_core::Ledger::load(&run_dir).unwrap();
    assert_eq!(merged.merged_lines("jvm"), complete as u64);
    assert!(merged.open_items().any(|o| o.resolution.is_none()
        && o.item.text == format!("worker jvm did not finish (slice ends at line {complete})")));
    let findings = merged
        .findings()
        .filter(|f| f.id.prefix() == Some("jvm"))
        .count();
    assert_eq!(
        findings,
        complete - 1,
        "every complete line but slice_started is a finding"
    );

    let st = ok(p, &["run", "status"]);
    assert_eq!(st["status"], "incomplete");
    assert_eq!(st["complete"], false);
    assert_eq!(worker_status(&run_dir, "jvm"), "killed");

    // The report says so, and a second merge changes nothing.
    let out = dla(p).arg("report").output().unwrap();
    let md = String::from_utf8(out.stdout).unwrap();
    assert!(md.contains("Status: INCOMPLETE"), "{md}");
    assert!(md.contains("status killed"), "{md}");
    assert!(md.contains(&format!(
        "worker jvm did not finish (slice ends at line {complete})"
    )));
    let before = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    ok(p, &["ledger", "merge"]);
    assert_eq!(
        fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap(),
        before
    );
}

#[test]
fn parallel_adds_by_one_worker_leave_a_valid_slice_with_distinct_ids() {
    let (dir, _id, run_dir) = project("node");
    let p = dir.path();
    // Start the slice, then race several writers.
    add(p, Some("node"), "a.js:1");
    let children: Vec<_> = (2..10)
        .map(|n| {
            let mut c = std::process::Command::new(assert_cmd::cargo::cargo_bin("dla"));
            c.arg("--project").arg(p);
            for var in [
                "DLA_RUN",
                "DLA_WORKER",
                "CLAUDE_SESSION_ID",
                "CLAUDE_CODE_SESSION_ID",
            ] {
                c.env_remove(var);
            }
            c.stdout(std::process::Stdio::null())
                .args([
                    "--worker",
                    "node",
                    "ledger",
                    "add",
                    "--origin",
                    "census",
                    "--evidence",
                    "E1",
                    "--site",
                    &format!("a.js:{n}"),
                    "--holder",
                    "static/global",
                ])
                .spawn()
                .unwrap()
        })
        .collect();
    for mut c in children {
        assert!(c.wait().unwrap().success());
    }
    let text = fs::read_to_string(run_dir.join("ledger.node.jsonl")).unwrap();
    let mut seqs = Vec::new();
    let mut ids = Vec::new();
    for l in text.lines() {
        let v: Value = serde_json::from_str(l).unwrap();
        seqs.push(v["seq"].as_u64().unwrap());
        if v["type"] == "finding_added" {
            ids.push(v["finding"]["id"].as_str().unwrap().to_owned());
        }
    }
    assert_eq!(seqs, (1..=seqs.len() as u64).collect::<Vec<_>>());
    assert_eq!(ids.len(), 9);
    let mut uniq = ids.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), 9, "{ids:?}");
    // The slice replays and merges.
    let m = ok(p, &["ledger", "merge"]);
    assert_eq!(m["merged_lines"]["node"], seqs.len() as u64);
}

#[test]
fn a_bad_slice_does_not_stop_the_merge_of_the_others() {
    let (dir, _id, run_dir) = project("go,rust");
    let p = dir.path();
    add(p, Some("go"), "a.go:1");
    add(p, Some("rust"), "z.rs:1");
    ok(p, &["--worker", "rust", "ledger", "done"]);
    let sp = run_dir.join("ledger.go.jsonl");
    let mut t = fs::read_to_string(&sp).unwrap();
    t = t.replacen("\"seq\":2", "\"seq\":7", 1);
    fs::write(&sp, t).unwrap();
    let m = ok(p, &["ledger", "merge"]);
    assert_eq!(m["invalid_slices"][0], "go");
    assert_eq!(m["merged_lines"]["rust"], 3);
    assert_eq!(worker_status(&run_dir, "go"), "killed");
    assert_eq!(worker_status(&run_dir, "rust"), "done");
}
