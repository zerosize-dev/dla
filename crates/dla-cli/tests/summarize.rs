//! End-to-end tests of `dla summarize heap-graph`: one dump, two dumps, a growing series, a
//! plateau series, refusals, and (with a JDK) dumps of a real JVM.

#[path = "../../dla-parsers/tests/common/hprof_builder.rs"]
mod hprof_builder;

use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command as Proc, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;
use sha2::{Digest, Sha256};

use hprof_builder::model;

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

fn run(project: &Path, args: &[&str]) -> (i32, Value) {
    let out = dla(project).args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), v)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!(code, 2, "{args:?}: {v}");
    let e = &v["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "no {k}: {v}");
    }
    e.clone()
}

struct Project {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let init = ok(dir.path(), &["run", "init", "--session-id", "t"]);
        let run_dir = dir
            .path()
            .join(".dla")
            .join(init["run_id"].as_str().unwrap());
        Self { dir, run_dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn put(&self, name: &str, bytes: &[u8]) -> String {
        let rel = format!("dumps/{name}");
        fs::create_dir_all(self.path().join("dumps")).unwrap();
        fs::write(self.path().join(&rel), bytes).unwrap();
        rel
    }

    /// The ledger lines of one type.
    fn lines(&self, ty: &str) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .filter(|v| v["type"] == ty)
            .collect()
    }

    fn evidence(&self, kind: &str) -> Vec<Value> {
        self.lines("evidence")
            .into_iter()
            .filter(|e| e["kind"] == kind)
            .collect()
    }
}

/// A gzip member of stored blocks.
fn gzip(data: &[u8]) -> Vec<u8> {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    let mut v = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3];
    let chunks: Vec<&[u8]> = data.chunks(65535).collect();
    for (i, c) in chunks.iter().enumerate() {
        v.push(u8::from(i == chunks.len() - 1));
        v.extend_from_slice(&(c.len() as u16).to_le_bytes());
        v.extend_from_slice(&(!(c.len() as u16)).to_le_bytes());
        v.extend_from_slice(c);
    }
    v.extend_from_slice(&(!crc).to_le_bytes());
    v.extend_from_slice(&(data.len() as u32).to_le_bytes());
    v
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

const T0: u64 = 1_800_000_000_000;

#[test]
fn one_dump_ends_with_a_request_for_more() {
    let p = Project::new();
    let m = model(8, T0, 200, 20);
    let rel = p.put("only.hprof", &m.dump);
    let v = ok(p.path(), &["summarize", "heap-graph", &rel]);
    assert_eq!(v["gate"], "fail");
    assert_eq!(v["reasons"][0], "dumps < 3");
    assert!(v["series"].is_null());
    assert!(
        v["needed"].as_str().unwrap().starts_with("2 more dumps"),
        "{v}"
    );
    assert_eq!(
        v["dumps"][0]["top_holder"]["key"],
        "static:app.Holder.ITEMS"
    );
    assert_eq!(v["dumps"][0]["top_holder"]["retained"], m.items_retained);
    assert_eq!(v["dumps"][0]["top_holder"]["dominated"], m.items_count);
    // One retention-path evidence: authority dump, the dump as the source with its sha256.
    let ev = p.evidence("retention-path");
    assert_eq!(ev.len(), 1);
    let e = &ev[0];
    assert_eq!(e["authority"], "dump");
    assert_eq!(e["source"]["artifact"]["path"], rel);
    assert_eq!(e["source"]["artifact"]["sha256"], sha(&m.dump));
    assert_eq!(e["result"]["holders"][0]["key"], "static:app.Holder.ITEMS");
    let raw = e["raw"].as_str().unwrap();
    assert!(raw.starts_with("summaries/only-"), "{raw}");
    let summary: Value =
        serde_json::from_str(&fs::read_to_string(p.run_dir.join(raw)).unwrap()).unwrap();
    assert_eq!(summary["header"]["timestamp_ms"], T0);
    assert_eq!(summary["holders"][0]["key"], "static:app.Holder.ITEMS");
    assert_eq!(summary["holders"][0]["accumulation"]["children"], 200);
    // No series, no growth evidence, no finding.
    assert!(p.evidence("growth-series").is_empty());
    assert!(p.lines("finding_added").is_empty());
    assert!(!p.run_dir.join("series").exists());
    // A gzip dump is read through the pure-Rust inflate, whatever its name.
    let gz = p.put("packed.dump", &gzip(&m.dump));
    let v = ok(p.path(), &["summarize", "heap-graph", &gz]);
    assert_eq!(v["dumps"][0]["top_holder"]["retained"], m.items_retained);
    assert_eq!(fs::read_dir(p.run_dir.join("tmp")).unwrap().count(), 0);
}

#[test]
fn two_dumps_fail_the_gate_and_are_ordered_by_timestamp() {
    let p = Project::new();
    let early = model(8, T0, 100, 5);
    let late = model(8, T0 + 60_000, 400, 5);
    // The names say the opposite of the timestamps.
    let a = p.put("a-late.hprof", &late.dump);
    let b = p.put("b-early.hprof", &early.dump);
    let v = ok(p.path(), &["summarize", "heap-graph", &a, &b]);
    assert_eq!(v["dumps"][0]["timestamp_ms"], T0);
    assert_eq!(v["dumps"][1]["timestamp_ms"], T0 + 60_000);
    assert_eq!(v["gate"], "fail");
    assert_eq!(v["reasons"][0], "dumps < 3");
    assert!(
        v["needed"].as_str().unwrap().starts_with("1 more dump "),
        "{v}"
    );
    assert_eq!(v["series"], "S1");
    assert_eq!(v["candidates"].as_array().unwrap().len(), 0);
    assert_eq!(p.evidence("retention-path").len(), 2);
    let g = &p.evidence("growth-series")[0];
    assert_eq!(g["authority"], "dump");
    assert_eq!(g["result"]["gate"], "fail");
    assert_eq!(g["result"]["reasons"][0], "dumps < 3");
    assert_eq!(g["result"]["passed"].as_array().unwrap().len(), 0);
    assert!(p.evidence("discriminator").is_empty());
    assert!(p.lines("finding_added").is_empty());
    // The series file has two columns per holder, in dump order.
    let s: Value =
        serde_json::from_str(&fs::read_to_string(p.run_dir.join("series/S1.json")).unwrap())
            .unwrap();
    assert_eq!(s["points"].as_array().unwrap().len(), 2);
    let names: Vec<&str> = s["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"holder:static:app.Holder.ITEMS"));
    assert!(names.contains(&"holder:static:app.Holder.ITEMS.count"));
    let i = names
        .iter()
        .position(|n| *n == "holder:static:app.Holder.ITEMS")
        .unwrap();
    assert_eq!(
        s["points"][0]["values"][i].as_f64(),
        Some(early.items_retained as f64)
    );
    assert_eq!(
        s["points"][1]["values"][i].as_f64(),
        Some(late.items_retained as f64)
    );
    assert_eq!(s["points"][1]["t_ms"], 60_000);
    assert_eq!(s["columns"][i]["unit"], "bytes");
    assert_eq!(s["columns"][i]["kind"], "gauge");
    assert_eq!(s["columns"][i]["region"], "heap");
    assert_eq!(s["columns"][i + 1]["kind"], "counter");
}

fn three(p: &Project, items: [usize; 3]) -> Vec<String> {
    (0..3)
        .map(|i| {
            let m = model(8, T0 + 30_000 * i as u64, items[i], 5);
            // Names deliberately out of order.
            p.put(&format!("dump-{}.hprof", 3 - i), &m.dump)
        })
        .collect()
}

#[test]
fn growing_series_passes_and_adds_a_site_free_candidate() {
    let p = Project::new();
    let files = three(&p, [200, 400, 600]);
    let args: Vec<&str> = ["summarize", "heap-graph"]
        .into_iter()
        .chain(files.iter().map(String::as_str))
        .collect();
    let v = ok(p.path(), &args);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["passed"][0], "static:app.Holder.ITEMS");
    assert_eq!(v["series"], "S1");
    // The candidate: owner, holder, region, origin growth-series, no site.
    let c = &v["candidates"][0];
    assert_eq!(c["owner"], "app.Holder.ITEMS");
    let added = p.lines("finding_added");
    assert_eq!(added.len(), 1);
    let f = &added[0]["finding"];
    assert_eq!(f["origin"], "growth-series");
    assert_eq!(f["owner"], "app.Holder.ITEMS");
    assert_eq!(f["holder"], "static/global");
    assert_eq!(f["region"], "heap");
    assert!(f.get("site").is_none() || f["site"].is_null(), "{f}");
    assert_eq!(f["state"], "candidate");
    // The growth evidence is tied to the candidate and cites the series.
    let g = &p.evidence("growth-series")[0];
    assert_eq!(g["authority"], "dump");
    assert_eq!(g["result"]["gate"], "pass");
    assert_eq!(
        g["result"]["passed"][0]["column"],
        "holder:static:app.Holder.ITEMS"
    );
    assert_eq!(g["result"]["passed"][0]["finding"], f["id"]);
    assert_eq!(g["source"]["artifact"]["path"], "series/S1.json");
    // The discriminator favors a leak and is bound to the candidate.
    let d = &p.evidence("discriminator")[0];
    assert_eq!(d["authority"], "dump");
    assert_eq!(d["result"]["test"], "dump-plateau");
    assert_eq!(d["result"]["favors"], "leak");
    assert_eq!(d["result"]["supports"][0], "leak");
    assert_eq!(d["result"]["finding"], f["id"]);
    assert_eq!(d["result"]["column"], "holder:static:app.Holder.ITEMS");
    // The dumps were read in timestamp order.
    let s: Value =
        serde_json::from_str(&fs::read_to_string(p.run_dir.join("series/S1.json")).unwrap())
            .unwrap();
    assert!(s["points"][0]["values"][0].as_f64() < s["points"][2]["values"][0].as_f64());
    // Running it again does not add the same owner twice and binds to the existing one.
    let v = ok(p.path(), &args);
    assert_eq!(v["candidates"].as_array().unwrap().len(), 0);
    assert_eq!(p.lines("finding_added").len(), 1);
    assert_eq!(v["discriminators"][0]["finding"], f["id"]);
    // The ledger still replays: the report reads it.
    let out = dla(p.path()).arg("report").output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn plateau_series_fails_the_gate_and_favors_a_non_leak() {
    let p = Project::new();
    let files = three(&p, [200, 300, 300]);
    let args: Vec<&str> = ["summarize", "heap-graph"]
        .into_iter()
        .chain(files.iter().map(String::as_str))
        .collect();
    let v = ok(p.path(), &args);
    assert_eq!(v["gate"], "fail", "{v}");
    assert!(
        v["reasons"][1]
            .as_str()
            .unwrap()
            .contains("static:app.Holder.ITEMS"),
        "{v}"
    );
    assert!(p.lines("finding_added").is_empty());
    let d = &p.evidence("discriminator")[0];
    assert_eq!(d["result"]["test"], "dump-plateau");
    assert_eq!(d["result"]["favors"], "non-leak");
    assert_eq!(
        d["result"]["supports"],
        serde_json::json!(["N-WARM", "N-HWM"])
    );
    assert_eq!(d["result"]["column"], "holder:static:app.Holder.ITEMS");
    assert!(d["result"]["finding"].is_null());
    assert_eq!(p.evidence("growth-series")[0]["result"]["gate"], "fail");
}

#[test]
fn finding_binds_the_discriminator_through_its_owner() {
    let p = Project::new();
    let files = three(&p, [200, 400, 600]);
    let base = ok(p.path(), &["summarize", "heap-graph", &files[0]]);
    assert_eq!(base["gate"], "fail");
    // An unknown finding and a finding without an owner are refused, writing nothing new.
    let before = p.lines("evidence").len();
    let e = refused(
        p.path(),
        &[
            "summarize",
            "heap-graph",
            &files[0],
            &files[1],
            &files[2],
            "--finding",
            "F9",
        ],
    );
    assert_eq!(e["code"], "unknown-finding");
    ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
        ],
    );
    let e = refused(
        p.path(),
        &[
            "summarize",
            "heap-graph",
            &files[0],
            &files[1],
            &files[2],
            "--finding",
            "F1",
        ],
    );
    assert_eq!(e["code"], "owner-required");
    assert!(
        e["fix"]
            .as_str()
            .unwrap()
            .contains("ledger annotate F1 --owner"),
        "{e}"
    );
    assert_eq!(p.lines("evidence").len(), before);
    ok(
        p.path(),
        &["ledger", "annotate", "F1", "--owner", "app.Holder.ITEMS"],
    );
    let v = ok(
        p.path(),
        &[
            "summarize",
            "heap-graph",
            &files[0],
            &files[1],
            &files[2],
            "--finding",
            "F1",
        ],
    );
    assert_eq!(v["discriminators"].as_array().unwrap().len(), 1);
    assert_eq!(v["discriminators"][0]["finding"], "F1");
    assert_eq!(v["discriminators"][0]["favors"], "leak");
    // The existing finding owns the owner: no new candidate.
    assert_eq!(v["candidates"].as_array().unwrap().len(), 0);
    ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--evidence",
            "E1",
        ],
    );
    ok(
        p.path(),
        &["ledger", "annotate", "F2", "--owner", "no.Such.field"],
    );
    let e = refused(
        p.path(),
        &[
            "summarize",
            "heap-graph",
            &files[0],
            &files[1],
            &files[2],
            "--finding",
            "F2",
        ],
    );
    assert_eq!(e["code"], "holder-not-found");
}

#[test]
fn refusals_name_the_missing_item_and_the_fix() {
    let p = Project::new();
    let m = model(8, T0, 50, 5);
    // Not a heap dump: the supported list is named.
    let txt = p.put("notes.txt", b"plain text");
    let e = refused(p.path(), &["summarize", "heap-graph", &txt]);
    assert_eq!(e["code"], "unsupported-format");
    assert!(e["message"].as_str().unwrap().contains(".hprof.gz"), "{e}");
    let bad = p.put("broken.hprof", b"JAVA PROFILE 9.9\0garbage");
    let e = refused(p.path(), &["summarize", "heap-graph", &bad]);
    assert_eq!(e["code"], "unsupported-format");
    let e = refused(
        p.path(),
        &["summarize", "heap-graph", "dumps/missing.hprof"],
    );
    assert_eq!(e["code"], "artifact-not-found");
    // A truncated dump.
    let cut = p.put("cut.hprof", &m.dump[..m.dump.len() - 300]);
    let e = refused(p.path(), &["summarize", "heap-graph", &cut]);
    assert_eq!(e["code"], "malformed-artifact");
    assert!(e["fix"].as_str().unwrap().contains("GC.heap_dump"), "{e}");
    // Dumps that cannot be one process: identifier size, equal timestamps, other classes.
    let a = p.put("a.hprof", &m.dump);
    let wide = p.put("w.hprof", &model(4, T0 + 5, 50, 5).dump);
    let e = refused(p.path(), &["summarize", "heap-graph", &a, &wide]);
    assert_eq!(e["code"], "dumps-not-comparable");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("identifier sizes differ"),
        "{e}"
    );
    let same = p.put("s.hprof", &model(8, T0, 60, 5).dump);
    let e = refused(p.path(), &["summarize", "heap-graph", &a, &same]);
    assert!(e["message"].as_str().unwrap().contains("timestamp"), "{e}");
    // Another program on the same JDK: its application classes are not in the later dump.
    let mut other = model(8, T0 + 9, 50, 5).dump;
    let at = other.windows(10).position(|w| w == b"app/Holder").unwrap();
    other[at..at + 3].copy_from_slice(b"web");
    let web = p.put("web.hprof", &other);
    let e = refused(p.path(), &["summarize", "heap-graph", &a, &web]);
    assert_eq!(e["code"], "dumps-not-comparable");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("application classes"),
        "{e}"
    );
    let e = refused(p.path(), &["summarize", "heap-graph", &a, &a]);
    assert_eq!(e["code"], "duplicate-artifact");
    // Nothing was written by the refusals.
    assert!(p.evidence("retention-path").is_empty());
    assert!(!p.run_dir.join("summaries").exists());
}

#[test]
fn a_dump_outside_the_project_is_read_by_absolute_path() {
    let p = Project::new();
    let other = tempfile::tempdir().unwrap();
    let m = model(8, T0, 30, 3);
    let abs = other.path().join("far.hprof");
    fs::write(&abs, &m.dump).unwrap();
    let v = ok(
        p.path(),
        &["summarize", "heap-graph", abs.to_str().unwrap()],
    );
    assert_eq!(
        v["dumps"][0]["top_holder"]["key"],
        "static:app.Holder.ITEMS"
    );
    let e = &p.evidence("retention-path")[0];
    assert_eq!(
        e["source"]["artifact"]["path"],
        abs.to_str().unwrap().replace('\\', "/")
    );
}

// ---------------------------------------------------------------------------------------
// A real JVM
// ---------------------------------------------------------------------------------------

const STREAM_JAVA: &str = r#"
package ledger;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.List;

public final class Stream {
    static final List<byte[]> HELD = new ArrayList<>();

    public static void main(String[] args) throws Exception {
        int cap = Integer.parseInt(args[0]);
        BufferedReader in = new BufferedReader(new InputStreamReader(System.in));
        String line;
        while ((line = in.readLine()) != null) {
            int n = Integer.parseInt(line.trim());
            for (int i = 0; i < n; i++) {
                HELD.add(new byte[200]);
                if (cap > 0 && HELD.size() > cap) {
                    HELD.remove(0);
                }
            }
            System.out.println("ok " + HELD.size());
            System.out.flush();
        }
    }
}
"#;

fn have(tool: &str) -> bool {
    Proc::new(tool).arg("-version").output().is_ok()
        || Proc::new(tool).arg("-help").output().is_ok()
}

/// Runs `ledger.Stream` with the given cap, dumping its heap after each of `steps` appends.
fn jvm_dumps(p: &Project, cap: u32, steps: &[u32]) -> Option<Vec<String>> {
    if !(have("javac") && have("jcmd") && have("java")) {
        eprintln!("skipped: javac, java and jcmd are needed");
        return None;
    }
    let work = tempfile::tempdir().unwrap();
    let src = work.path().join("src/ledger");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("Stream.java"), STREAM_JAVA).unwrap();
    let out = work.path().join("out");
    assert!(
        Proc::new("javac")
            .arg("-d")
            .arg(&out)
            .arg(src.join("Stream.java"))
            .status()
            .unwrap()
            .success()
    );
    let mut child = Proc::new("java")
        .args([
            "-Xmx32m",
            "-Xms32m",
            "-XX:+UseSerialGC",
            "--limit-modules",
            "java.base",
            "-cp",
        ])
        .arg(&out)
        .args(["ledger.Stream", &cap.to_string()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut files = Vec::new();
    for (i, n) in steps.iter().enumerate() {
        writeln!(stdin, "{n}").unwrap();
        stdin.flush().unwrap();
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        assert!(line.starts_with("ok"), "{line:?}");
        let rel = format!("dumps/jvm-{i}.hprof");
        fs::create_dir_all(p.path().join("dumps")).unwrap();
        let dest = p.path().join(&rel);
        let st = Proc::new("jcmd")
            .arg(child.id().to_string())
            .arg("GC.heap_dump")
            .arg(&dest)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(st.success());
        files.push(rel);
    }
    drop(stdin);
    let _ = child.wait();
    Some(files)
}

#[test]
fn real_jvm_growing_series_confirms_a_holder_without_a_site() {
    let p = Project::new();
    let Some(files) = jvm_dumps(&p, 0, &[2000, 2000, 2000]) else {
        return;
    };
    let args: Vec<&str> = ["summarize", "heap-graph"]
        .into_iter()
        .chain(files.iter().map(String::as_str))
        .collect();
    let v = ok(p.path(), &args);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["candidates"][0]["owner"], "ledger.Stream.HELD", "{v}");
    assert_eq!(v["discriminators"][0]["favors"], "leak");
    let f = &p.lines("finding_added")[0]["finding"];
    assert!(f.get("site").is_none() || f["site"].is_null());
}

#[test]
fn real_jvm_plateau_series_is_not_a_leak() {
    let p = Project::new();
    let Some(files) = jvm_dumps(&p, 2500, &[1500, 1500, 1500, 1500]) else {
        return;
    };
    let args: Vec<&str> = ["summarize", "heap-graph"]
        .into_iter()
        .chain(files.iter().map(String::as_str))
        .collect();
    let v = ok(p.path(), &args);
    assert_eq!(v["gate"], "fail", "{v}");
    assert_eq!(v["candidates"].as_array().unwrap().len(), 0);
    let d = p.evidence("discriminator");
    let held = d
        .iter()
        .find(|e| e["result"]["column"] == "holder:static:ledger.Stream.HELD")
        .unwrap();
    assert_eq!(held["result"]["favors"], "non-leak", "{held}");
}
