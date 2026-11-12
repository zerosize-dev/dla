//! The JVM adapters of `dla summarize`: JFR old-object samples, thread dumps, class loader
//! stats, native memory tracking, and the via holders of the HPROF reader.
//!
//! The conformance test (`summarize_conformance.rs`) runs every adapter over its fixture.
//! These tests check what is specific to the JVM: the holder tie of a JFR root path, the
//! writers it learns, the redaction of thread names, that thread groups and runtime totals
//! never tie, and that via holders become holders with columns, ties and candidates.

#[path = "../../dla-parsers/tests/common/hprof_builder.rs"]
mod hprof_builder;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as Proc,
};

use assert_cmd::Command;
use serde_json::{Value, json};

use hprof_builder::{B, F, T_BYTE, T_OBJ};

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

fn run_with(project: &Path, args: &[&str], env: &[(&str, Option<&str>)]) -> (i32, String, Value) {
    let mut c = dla(project);
    c.args(args);
    for (k, v) in env {
        match v {
            Some(v) => c.env(k, v),
            None => c.env_remove(k),
        };
    }
    let out = c.output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), text, v)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, _, v) = run_with(project, args, &[]);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let (code, _, v) = run_with(project, args, &[]);
    assert_eq!(code, 2, "{args:?}: {v}");
    v["error"].clone()
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

    fn put(&self, rel: &str, bytes: &[u8]) -> String {
        let p = self.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
        rel.to_owned()
    }

    fn ledger_lines(&self) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn evidence(&self, kind: &str) -> Vec<Value> {
        self.ledger_lines()
            .into_iter()
            .filter(|e| e["type"] == "evidence" && e["kind"] == kind)
            .collect()
    }

    fn findings(&self) -> Vec<Value> {
        self.ledger_lines()
            .into_iter()
            .filter(|e| e["type"] == "finding_added")
            .map(|e| e["finding"].clone())
            .collect()
    }

    /// The writers recorded for a finding by annotations, in order.
    fn writers(&self, f: &str) -> Vec<String> {
        self.ledger_lines()
            .into_iter()
            .filter(|e| e["type"] == "finding_annotated" && e["finding"] == f)
            .flat_map(|e| e["writers"].as_array().cloned().unwrap_or_default())
            .map(|w| w.as_str().unwrap().to_owned())
            .collect()
    }

    fn coverage(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir.join("coverage.json")).unwrap())
            .unwrap()
    }

    fn run_texts(&self) -> Vec<(PathBuf, String)> {
        fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
            for e in fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if let Ok(t) = fs::read_to_string(&p) {
                    out.push((p, t));
                }
            }
        }
        let mut v = Vec::new();
        walk(&self.run_dir, &mut v);
        v
    }
}

/// A census finding as `dla census` leaves it: owner, declaration site, writers.
fn census_finding(p: &Project, owner: &str, site: &str, writers: &[&str]) -> String {
    let v = ok(
        p.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            site,
            "--owner",
            owner,
            "--holder",
            "static/global",
            "--region",
            "heap",
        ],
    );
    let id = v["id"].as_str().unwrap().to_owned();
    if !writers.is_empty() {
        let mut args = vec!["ledger", "annotate", id.as_str()];
        for w in writers {
            args.extend(["--writer", w]);
        }
        ok(p.path(), &args);
    }
    id
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/summarize")
}

// ---------------------------------------------------------------------------------------
// JFR old-object samples
// ---------------------------------------------------------------------------------------

/// One printed `jdk.OldObjectSample` event in the form `jfr print` writes it: a byte array
/// held by a list held by a static field of `holder_class`, allocated in `alloc` (frames,
/// innermost first). `root_static` is the field name, or `None` for a thread-stack root.
fn event(holder_class: &str, root_static: Option<&str>, alloc: &[(&str, u32)]) -> String {
    let chain = match root_static {
        Some(f) => format!(
            "    byte[2048]\n    [94] : java.lang.Object[2776]\n    elementData : java.util.ArrayList Size: 2400\n    {f} : java.lang.Class Class Name: {holder_class}\n"
        ),
        None => "    byte[2048]\n".to_owned(),
    };
    let root = match root_static {
        Some(_) => "  root = {\n    description = N/A\n    system = \"Class Loader Data\"\n    type = \"<unknown>\"\n  }\n".to_owned(),
        None => "  root = {\n    description = \"Thread Name: worker\"\n    system = \"Threads\"\n    type = \"Stack Variable\"\n  }\n".to_owned(),
    };
    let frames: String = alloc
        .iter()
        .map(|(f, l)| format!("    {f}(int) line: {l}\n"))
        .collect();
    format!(
        "jdk.OldObjectSample {{\n  startTime = 11:33:00.923 (2026-11-09)\n  objectSize = 2.0 kB\n  objectAge = 14.3 s\n  object =  [\n{chain}  ]\n  arrayElements = 2048\n{root}  eventThread = \"main\" (javaThreadId = 1)\n  stackTrace = [\n{frames}  ]\n}}\n\n"
    )
}

/// A recording of `n` samples held by `ENTRIES`, plus `other` held by a thread stack.
fn recording(n: usize, other: usize) -> String {
    let mut t = String::new();
    for _ in 0..n {
        t += &event(
            "shop.Cache",
            Some("ENTRIES"),
            &[("shop.Cache.put", 12), ("shop.Main.main", 5)],
        );
    }
    for _ in 0..other {
        t += &event(
            "shop.Cache",
            None,
            &[("shop.Cache.put", 12), ("shop.Main.main", 5)],
        );
    }
    t
}

const CACHE_JAVA: &str = "package shop;\n\nimport java.util.ArrayList;\nimport java.util.List;\n\npublic final class Cache {\n    // the leak\n    static final List<byte[]> ENTRIES = new ArrayList<>();\n    static int pad;\n    static void x() {}\n\n    static void put(int n) {\n        ENTRIES.add(new byte[n]);\n    }\n}\n";

fn jfr_project(counts: &[(usize, usize)]) -> (Project, Vec<String>, String) {
    let p = Project::new();
    p.put("src/shop/Cache.java", CACHE_JAVA.as_bytes());
    let f = census_finding(&p, "shop.Cache.ENTRIES", "src/shop/Cache.java:8", &[]);
    let files = counts
        .iter()
        .enumerate()
        .map(|(i, (n, o))| p.put(&format!("in/r{i}.txt"), recording(*n, *o).as_bytes()))
        .collect();
    (p, files, f)
}

fn alloc_diff(p: &Project, files: &[String], extra: &[&str]) -> Value {
    let mut args = vec!["summarize", "alloc-stack-diff"];
    args.extend(files.iter().map(String::as_str));
    args.extend_from_slice(extra);
    ok(p.path(), &args)
}

#[test]
fn a_jfr_holder_ties_to_the_census_finding_and_learns_the_writer() {
    let (p, files, f) = jfr_project(&[(4, 1), (9, 1), (15, 1)]);
    let v = alloc_diff(&p, &files, &[]);
    assert_eq!(v["backend"], "jfr-old-object-text", "{v}");
    assert_eq!(v["sampled"], true);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([f]), "{v}");
    let g = &p.evidence("growth-series")[0];
    let passed = g["result"]["passed"].as_array().unwrap();
    let holder = passed
        .iter()
        .find(|c| c["column"] == "holder:static:shop.Cache.ENTRIES")
        .unwrap_or_else(|| panic!("no holder column: {g}"));
    assert_eq!(holder["finding"], f.as_str());
    assert_eq!(holder["tie"]["rule"], "holder");
    assert_eq!(holder["tie"]["key"], "shop.Cache.ENTRIES");
    // The first project frame of the holder's stacks is a writer, learned by the holder tie.
    assert!(
        p.writers(&f).contains(&"src/shop/Cache.java:12".to_owned()),
        "{:?}",
        p.writers(&f)
    );
    // The thread-stack samples have no holder: they are never in a tied column.
    for c in passed {
        if c["column"].as_str().unwrap().contains("root:") {
            assert!(c.get("finding").is_none(), "{c}");
        }
    }
    // The signal names the tool and the evidence is sampled.
    assert_eq!(g["result"]["signal"]["tool"], "jfr-old-object-text");
}

#[test]
fn one_or_two_recordings_give_no_growth_column() {
    let (p, files, _f) = jfr_project(&[(4, 0), (9, 0)]);
    let v = alloc_diff(&p, &files, &[]);
    assert_eq!(v["mode"], "diff");
    assert_eq!(v["gate"], "fail", "{v}");
    assert!(p.evidence("growth-series").is_empty());
    let (p, files, _f) = jfr_project(&[(4, 0)]);
    let v = alloc_diff(&p, &files, &[]);
    assert_eq!(v["mode"], "single");
    assert_eq!(v["gate"], "fail");
    assert!(
        v["top"][0]["root"]
            .as_str()
            .unwrap()
            .contains("shop.Cache.ENTRIES")
    );
    assert!(p.evidence("growth-series").is_empty());
}

#[test]
fn a_sampled_count_that_does_not_keep_growing_is_not_a_column() {
    // A sample count that saturates: 4, 14, 15. The last step is under half of the first.
    let (p, files, _f) = jfr_project(&[(4, 0), (14, 0), (15, 0)]);
    let v = alloc_diff(&p, &files, &[]);
    assert_eq!(v["gate"], "fail", "{v}");
    assert_eq!(v["tied"], json!([]));
    assert!(p.writers("F1").is_empty());
}

#[test]
fn a_holder_that_is_not_the_findings_holder_does_not_tie_by_site() {
    // The finding's owner is another field of the class. The samples name ENTRIES as their
    // holder, so even though their allocation frame is the finding's writer they tie to
    // nothing and no writer is learned.
    let p = Project::new();
    p.put("src/shop/Cache.java", CACHE_JAVA.as_bytes());
    let f = census_finding(
        &p,
        "shop.Cache.OTHER",
        "src/shop/Cache.java:9",
        &["src/shop/Cache.java:12"],
    );
    let files: Vec<String> = [4, 9, 15]
        .iter()
        .enumerate()
        .map(|(i, n)| p.put(&format!("in/r{i}.txt"), recording(*n, 0).as_bytes()))
        .collect();
    let v = alloc_diff(&p, &files, &["--finding", &f]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([]), "{v}");
    assert!(
        v["needed"]
            .as_str()
            .unwrap()
            .contains("no passing column ties to"),
        "{v}"
    );
    assert_eq!(p.writers(&f), vec!["src/shop/Cache.java:12".to_owned()]);
}

#[test]
fn samples_with_no_static_holder_never_tie_by_site() {
    // All samples hang from a thread stack. Their allocation frame is the finding's
    // writer, but a root path that names no holder is not a holder identity.
    let p = Project::new();
    p.put("src/shop/Cache.java", CACHE_JAVA.as_bytes());
    let f = census_finding(
        &p,
        "shop.Cache.ENTRIES",
        "src/shop/Cache.java:8",
        &["src/shop/Cache.java:12"],
    );
    let files: Vec<String> = [4, 9, 15]
        .iter()
        .enumerate()
        .map(|(i, n)| p.put(&format!("in/r{i}.txt"), recording(0, *n).as_bytes()))
        .collect();
    let v = alloc_diff(&p, &files, &[]);
    assert_eq!(v["tied"], json!([]), "{v}");
    let _ = f;
}

#[test]
fn a_planted_secret_in_a_holder_name_is_shaped_unless_raw_values_are_allowed() {
    let secret = "AKIAIOSFODNN7EXAMPLE1";
    let mk = |p: &Project| -> Vec<String> {
        [4, 9, 15]
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let t = recording(*n, 0).replace("ENTRIES", secret);
                p.put(&format!("in/r{i}.txt"), t.as_bytes())
            })
            .collect()
    };
    let p = Project::new();
    let files = mk(&p);
    let (code, text, _v) = {
        let mut args = vec!["summarize", "alloc-stack-diff"];
        args.extend(files.iter().map(String::as_str));
        run_with(p.path(), &args, &[])
    };
    assert_eq!(code, 0);
    assert!(!text.contains(secret));
    for (path, t) in p.run_texts() {
        assert!(!t.contains(secret), "{}", path.display());
    }
    // With consent the key is kept (so the check above can fail).
    let q = Project::new();
    ok(q.path(), &["consent", "--tiers", "1", "--raw-values"]);
    let files = mk(&q);
    alloc_diff(&q, &files, &[]);
    assert!(q.run_texts().iter().any(|(_, t)| t.contains(secret)));
}

#[test]
fn a_holder_missing_from_a_recording_is_a_gap_not_a_zero() {
    // The sampled holder is first seen in recording 1: a reservoir sample that missed it
    // (or a root path that did not resolve) in recording 0 does not make its total 0.
    let (p, files, _f) = jfr_project(&[(0, 1), (8, 1), (16, 1)]);
    let v = alloc_diff(&p, &files, &[]);
    assert_eq!(v["gate"], "fail", "{v}");
    assert_eq!(v["tied"], json!([]), "{v}");
    assert!(p.writers("F1").is_empty());
    // A holder in every recording that keeps growing still passes.
    let (p, files, f) = jfr_project(&[(4, 1), (8, 1), (16, 1)]);
    let v = alloc_diff(&p, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([f]), "{v}");
}

#[test]
fn a_key_like_sampled_type_is_shaped_in_the_raw_file() {
    let secret = "AKIAIOSFODNN7EXAMPLE1";
    let p = Project::new();
    let files: Vec<String> = [4, 9, 15]
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let t = recording(*n, 0).replace("byte[2048]", &format!("shop.Tok_{secret}"));
            p.put(&format!("in/r{i}.txt"), t.as_bytes())
        })
        .collect();
    let (code, text, _v) = {
        let mut args = vec!["summarize", "alloc-stack-diff"];
        args.extend(files.iter().map(String::as_str));
        run_with(p.path(), &args, &[])
    };
    assert_eq!(code, 0);
    assert!(!text.contains(secret));
    let raw: Vec<_> = p
        .run_texts()
        .into_iter()
        .filter(|(path, _)| path.to_string_lossy().contains("allocs"))
        .collect();
    assert!(!raw.is_empty());
    assert!(
        raw.iter()
            .any(|(_, t)| t.contains("\"kind\":\"shop.Tok_<")
                || t.contains("\"kind\": \"shop.Tok_<")),
        "{raw:?}"
    );
    for (path, t) in p.run_texts() {
        assert!(!t.contains(secret), "{}", path.display());
    }
}

/// A stub `jfr` on a PATH of its own: the script text runs in `sh` with the recording as
/// the last argument.
#[cfg(unix)]
fn stub_jfr(body: &str) -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("jfr");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    d
}

#[cfg(unix)]
fn run_with_stub(p: &Project, stub: &Path) -> (i32, Value) {
    let path = format!("{}:/usr/bin:/bin", stub.display());
    let (code, _t, v) = run_with(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "jfr-old-object-sample",
            "in/a.jfr",
        ],
        &[("PATH", Some(path.as_str())), ("JAVA_HOME", None)],
    );
    (code, v)
}

#[cfg(unix)]
#[test]
fn printed_output_over_the_cap_is_refused_and_removed() {
    let p = Project::new();
    let mut rec = b"FLR\0\0\x02\0\x01".to_vec();
    rec.extend_from_slice(&[0u8; 64]);
    p.put("in/a.jfr", &rec);
    // A sparse 600 MB file on stdout: over the 512 MB cap, nothing is written to disk.
    let stub = stub_jfr("truncate -s 600M /dev/stdout");
    let (code, v) = run_with_stub(&p, stub.path());
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "insufficient-resources", "{v}");
    assert!(v["error"]["message"].as_str().unwrap().contains("limit"));
    for (path, _t) in p.run_texts() {
        assert!(
            !path.to_string_lossy().contains("jfr-"),
            "{}",
            path.display()
        );
    }
    assert!(
        fs::read_dir(p.run_dir.join("tmp"))
            .map(|d| d.flatten().all(|e| e.metadata().unwrap().len() == 0))
            .unwrap_or(true)
    );
}

#[cfg(unix)]
#[test]
fn a_failed_print_shows_the_end_of_stderr_not_a_banner() {
    let p = Project::new();
    let mut rec = b"FLR\0\0\x02\0\x01".to_vec();
    rec.extend_from_slice(&[0u8; 64]);
    p.put("in/a.jfr", &rec);
    let banner = "Picked up JAVA_TOOL_OPTIONS: -Dhttp.nonProxyHosts=localhost|127.*|10.*|172.16.*|172.17.*|172.18.*|172.19.*|172.20.*|172.21.*|172.22.*|172.23.*|172.24.*|172.25.*|172.26.*|172.27.*";
    let stub = stub_jfr(&format!(
        "echo '{banner}' >&2\necho 'jfr print: could not read recording' >&2\nexit 1"
    ));
    let (code, v) = run_with_stub(&p, stub.path());
    assert_eq!(code, 2, "{v}");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("could not read recording"), "{msg}");
    assert!(!msg.ends_with('\n'), "{msg:?}");
}

fn have(tool: &str) -> bool {
    Proc::new(tool)
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn a_recording_is_printed_with_the_jfr_tool() {
    if !have("java") || !have("javac") || !Path::new("/usr/bin/true").exists() && cfg!(unix) {
        eprintln!("skipped: no JDK on PATH");
        return;
    }
    let jfr_ok = Proc::new("jfr")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !jfr_ok {
        eprintln!("skipped: no jfr tool on PATH");
        return;
    }
    let work = tempfile::tempdir().unwrap();
    let src = work.path().join("Hold.java");
    fs::write(
        &src,
        "public class Hold {\n  static final java.util.List<byte[]> KEEP = new java.util.ArrayList<>();\n  public static void main(String[] a) throws Exception {\n    for (int i = 0; i < 16; i++) { for (int j = 0; j < 2000; j++) KEEP.add(new byte[2048]); System.gc(); Thread.sleep(250); }\n  }\n}\n",
    )
    .unwrap();
    let st = Proc::new("javac")
        .arg("-d")
        .arg(work.path())
        .arg(&src)
        .output()
        .unwrap();
    assert!(st.status.success(), "javac failed");
    let rec = work.path().join("hold.jfr");
    let st = Proc::new("java")
        .arg("-XX:StartFlightRecording=settings=profile,path-to-gc-roots=true,dumponexit=true,filename=hold.jfr")
        .arg("-cp")
        .arg(work.path())
        .arg("Hold")
        .current_dir(work.path())
        .output()
        .unwrap();
    assert!(st.status.success(), "java failed");
    assert!(rec.is_file());
    let p = Project::new();
    let bytes = fs::read(&rec).unwrap();
    p.put("in/hold.jfr", &bytes);
    let v = ok(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "jfr-old-object-sample",
            "in/hold.jfr",
        ],
    );
    assert_eq!(v["backend"], "jfr-old-object-sample");
    assert_eq!(v["mode"], "single");
    assert_eq!(v["sampled"], true);
    assert_eq!(v["gate"], "fail");
    let top = v["top"].as_array().unwrap();
    // OldObjectSample is a sample: a JVM build can end a short run with none. The tool path
    // (run, parse, clean up) is still checked; the holder only when something was sampled.
    if top.is_empty() {
        eprintln!("note: the recording holds no old-object samples on this JVM: {v}");
    } else {
        assert!(
            top.iter()
                .any(|r| r["root"].as_str().is_some_and(|r| r.contains("Hold.KEEP"))),
            "{v}"
        );
    }
    // The tool output is removed after it is parsed.
    for (path, _t) in p.run_texts() {
        assert!(
            !path.to_string_lossy().contains("jfr-"),
            "{}",
            path.display()
        );
    }
    let cov = p.coverage();
    let row = cov["workers"]["jvm"]["backends"]["jfr-old-object-sample"].clone();
    assert_eq!(row["status"], "ran", "{cov}");
}

#[test]
fn a_recording_without_the_jfr_tool_is_a_gap_in_auto_and_a_refusal_when_named() {
    let p = Project::new();
    let mut rec = b"FLR\0\0\x02\0\x01".to_vec();
    rec.extend_from_slice(&[0u8; 64]);
    p.put("in/a.jfr", &rec);
    let empty = tempfile::tempdir().unwrap();
    let env = [
        ("PATH", Some(empty.path().to_str().unwrap())),
        ("JAVA_HOME", None),
        ("DLA_DATA_DIR", None),
    ];
    let (code, _t, v) = run_with(
        p.path(),
        &["summarize", "alloc-stack-diff", "in/a.jfr"],
        &env,
    );
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["evidence"], json!([]));
    assert_eq!(v["coverage_gap"][0]["backend"], "jfr-old-object-sample");
    assert!(v["coverage_gap"][0]["install_hint"].is_string());
    let (code, _t, v) = run_with(
        p.path(),
        &[
            "summarize",
            "alloc-stack-diff",
            "--backend",
            "jfr-old-object-sample",
            "in/a.jfr",
        ],
        &env,
    );
    assert_eq!(code, 2);
    assert_eq!(v["error"]["code"], "tool-missing");
    // Printed text needs no tool: auto reads it with the text adapter.
    p.put("in/a.txt", recording(2, 0).as_bytes());
    let (code, _t, v) = run_with(
        p.path(),
        &["summarize", "alloc-stack-diff", "in/a.txt"],
        &env,
    );
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["backend"], "jfr-old-object-text");
}

#[test]
fn a_truncated_print_is_refused_with_the_fix() {
    let p = Project::new();
    let t = recording(2, 0);
    p.put("in/cut.txt", &t.as_bytes()[..t.len() - 60]);
    let e = refused(p.path(), &["summarize", "alloc-stack-diff", "in/cut.txt"]);
    assert_eq!(e["code"], "malformed-artifact");
    assert!(e["fix"].as_str().unwrap().contains("jfr print"));
}

// ---------------------------------------------------------------------------------------
// thread dumps, class loaders, native memory
// ---------------------------------------------------------------------------------------

fn thread_dump(threads: &[(String, &str)]) -> String {
    let list: Vec<Value> = threads
        .iter()
        .enumerate()
        .map(|(i, (name, frame))| {
            json!({"tid": (i + 1).to_string(), "name": name, "stack": [
                "java.base/jdk.internal.misc.Unsafe.park(Native Method)",
                "java.base/java.util.concurrent.locks.LockSupport.park(LockSupport.java:371)",
                frame
            ]})
        })
        .collect();
    json!({"threadDump": {"processId": "1", "time": "2026-11-09T00:00:00Z",
        "runtimeVersion": "21", "threadContainers": [{"container": "<root>", "parent": null,
        "owner": null, "threads": list}]}})
    .to_string()
}

fn dumps(p: &Project, names: impl Fn(usize) -> Vec<String>) -> Vec<String> {
    (0..3)
        .map(|i| {
            let threads: Vec<(String, &str)> = names(i)
                .into_iter()
                .map(|n| (n, "shop.Worker.run(Worker.java:20)"))
                .collect();
            p.put(&format!("in/t{i}.json"), thread_dump(&threads).as_bytes())
        })
        .collect()
}

fn nonheap(p: &Project, files: &[String]) -> Value {
    let mut args = vec!["summarize", "non-heap"];
    args.extend(files.iter().map(String::as_str));
    ok(p.path(), &args)
}

#[test]
fn thread_names_show_as_plain_patterns_or_shapes_and_never_tie() {
    let p = Project::new();
    p.put(
        "src/shop/Worker.java",
        format!("package shop;\n\npublic final class Worker {{\n    static void spawn() {{\n        new Thread(Worker::run).start();\n    }}\n{}    static void run() {{}}\n}}\n", "    static int pad;\n".repeat(14)).as_bytes(),
    );
    // A finding whose site and writer are the frame the threads run in now.
    census_finding(
        &p,
        "shop.Worker.THREADS",
        "src/shop/Worker.java:20",
        &["src/shop/Worker.java:20"],
    );
    let files = dumps(&p, |i| {
        let mut v: Vec<String> = (0..(2 + i * 3))
            .map(|k| format!("pool-1-thread-{k}"))
            .collect();
        v.push("AKIAIOSFODNN7EXAMPLE1".to_owned());
        v.push("main".to_owned());
        v
    });
    let v = nonheap(&p, &files);
    assert_eq!(v["backend"], "jvm-thread-dump");
    assert_eq!(v["gate"], "pass", "{v}");
    // The pattern of the pool is program text and shows; the key does not.
    let text = v.to_string();
    assert!(text.contains("name:pool-*-thread-*"), "{text}");
    assert!(!text.contains("AKIA"), "{text}");
    for (path, t) in p.run_texts() {
        assert!(!t.contains("AKIAIOSFODNN7EXAMPLE1"), "{}", path.display());
    }
    // The frame matches the finding's site and writer, and still nothing ties.
    assert_eq!(v["tied"], json!([]), "{v}");
    let g = &p.evidence("growth-series")[0];
    for c in g["result"]["passed"].as_array().unwrap() {
        assert!(c.get("finding").is_none(), "{c}");
    }
    // The name shape is the same in every dump, so the group is one column.
    let raw: Value = serde_json::from_str(
        &fs::read_to_string(p.run_dir.join(v["raw"]["path"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    let shaped: Vec<&Value> = raw["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["name"].as_str().unwrap().contains("name:<"))
        .collect();
    assert_eq!(shaped.len(), 1, "{raw}");
}

#[test]
fn with_raw_values_a_thread_name_pattern_is_kept() {
    let p = Project::new();
    ok(p.path(), &["consent", "--tiers", "1", "--raw-values"]);
    let files = dumps(&p, |i| {
        let mut v: Vec<String> = (0..(2 + i)).map(|k| format!("w{k}")).collect();
        v.push("token-AKIAIOSFODNN7EXAMPLE1".to_owned());
        v
    });
    let v = nonheap(&p, &files);
    assert_eq!(v["redaction"], "raw");
    assert!(
        p.run_texts()
            .iter()
            .any(|(_, t)| t.contains("token-AKIAIOSFODNN*EXAMPLE*"))
    );
}

#[test]
fn native_memory_summaries_and_diffs_are_not_mixed() {
    let p = Project::new();
    let nmt = fixtures().join("jvm-nmt");
    let a = p.put("in/nmt1.txt", &fs::read(nmt.join("nmt1.txt")).unwrap());
    // A diff is a summary with deltas on the lines.
    let diff = "24012:\n\nNative Memory Tracking:\n\nTotal: reserved=5064030KB +183KB, committed=368654KB +791KB\n\n-                     Class (reserved=1048771KB +13KB, committed=707KB +77KB)\n";
    let b = p.put("in/nmt-diff.txt", diff.as_bytes());
    let e = refused(p.path(), &["summarize", "non-heap", &a, &b]);
    assert_eq!(e["code"], "mixed-formats", "{e}");
    assert!(e["message"].as_str().unwrap().contains("diff"));
    // One diff alone is a single view with delta metrics.
    let v = ok(p.path(), &["summarize", "non-heap", &b]);
    assert_eq!(v["backend"], "jvm-nmt");
    assert_eq!(v["mode"], "single");
    let top = v["top"].to_string();
    assert!(top.contains("nmt.committed.delta"), "{top}");
}

#[test]
fn native_memory_that_is_off_is_refused_with_the_flag() {
    let p = Project::new();
    p.put(
        "in/off.txt",
        b"1234:\nNative Memory Tracking:\n\nTotal: reserved=1KB, committed=1KB\nNative memory tracking is not enabled\n",
    );
    let e = refused(p.path(), &["summarize", "non-heap", "in/off.txt"]);
    assert_eq!(e["code"], "malformed-artifact");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("NativeMemoryTracking")
    );
}

#[test]
fn class_loader_and_native_memory_growth_are_columns_that_tie_to_nothing() {
    let p = Project::new();
    census_finding(
        &p,
        "shop.Loaders.ALL",
        "src/shop/Loaders.java:3",
        &["src/shop/Loaders.java:9"],
    );
    let cl = fixtures().join("jvm-classloader-stats");
    let files: Vec<String> = (1..=3)
        .map(|i| {
            p.put(
                &format!("in/cl{i}.txt"),
                &fs::read(cl.join(format!("classloader{i}.txt"))).unwrap(),
            )
        })
        .collect();
    let v = nonheap(&p, &files);
    assert_eq!(v["backend"], "jvm-classloader-stats");
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([]));
    let nmt = fixtures().join("jvm-nmt");
    let files: Vec<String> = (1..=3)
        .map(|i| {
            p.put(
                &format!("in/nmt{i}.txt"),
                &fs::read(nmt.join(format!("nmt{i}.txt"))).unwrap(),
            )
        })
        .collect();
    let v = nonheap(&p, &files);
    assert_eq!(v["backend"], "jvm-nmt");
    assert_eq!(v["tied"], json!([]));
    for g in p.evidence("growth-series") {
        for c in g["result"]["passed"].as_array().unwrap() {
            assert!(c.get("finding").is_none(), "{c}");
        }
    }
}

/// A `VM.classloader_stats` text: `loader` defined `classes` classes in one CLD, next to the
/// application loader.
fn loader_stats(loader: &str, classes: u32) -> String {
    format!(
        "1411:\n\nClassLoader         Parent              CLD*               Classes   ChunkSz   BlockSz  Type\n\
         0x00007f93c4002800  0x00007f93c3068770  0x00007f9434186f90  {classes:>6}   6144000     94400  {loader}\n\
         0x00007f93c3068770  0x00007f93c3069008  0x00007f9434145360       4     12288     10432  jdk.internal.loader.ClassLoaders$AppClassLoader\n\
         Total = 2                                                     {total}   4574208    337968  \n\
         ChunkSz: Total size of all allocated metaspace chunks\n\
         BlockSz: Total size of all allocated metaspace blocks (each chunk has several blocks)\n",
        total = classes + 4,
    )
}

/// A project whose static `LOADER` of type `field_type` is a census finding.
fn loader_project(field_type: &str, package: &str) -> (Project, String) {
    let p = Project::new();
    p.put(
        "src/quotes/RuleLoader.java",
        b"package quotes;\n\nfinal class RuleLoader extends ClassLoader {\n}\n",
    );
    p.put(
        "src/quotes/PricingRules.java",
        format!(
            "package {package};\n\nimport java.nio.charset.StandardCharsets;\n\nfinal class PricingRules {{\n\n    private static final {field_type} LOADER = new {field_type}();\n}}\n"
        )
        .as_bytes(),
    );
    let id = census_finding(
        &p,
        &format!("{package}.PricingRules.LOADER"),
        "src/quotes/PricingRules.java:7",
        &[],
    );
    (p, id)
}

fn loader_files(p: &Project, loader: &str) -> Vec<String> {
    (0..3)
        .map(|i| {
            p.put(
                &format!("in/ld{i}.txt"),
                loader_stats(loader, 100 + 400 * i).as_bytes(),
            )
        })
        .collect()
}

#[test]
fn a_loader_group_ties_to_the_finding_whose_holder_is_declared_with_that_loader_class() {
    let (p, id) = loader_project("RuleLoader", "quotes");
    let files = loader_files(&p, "quotes.RuleLoader");
    let v = nonheap(&p, &files);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([id]), "{v}");
    let growth = p.evidence("growth-series");
    let passed = growth.last().unwrap()["result"]["passed"]
        .as_array()
        .unwrap()
        .clone();
    let col = passed
        .iter()
        .find(|c| c["column"] == "group:classloader/quotes.RuleLoader")
        .unwrap_or_else(|| panic!("{passed:?}"));
    assert_eq!(col["finding"], json!(id), "{col}");
    assert_eq!(col["tie"]["rule"], "type", "{col}");
    assert_eq!(col["tie"]["key"], "quotes.RuleLoader", "{col}");
}

#[test]
fn a_loader_group_ties_by_exact_class_identity_only() {
    // The holder is declared with a loader of another package of the same simple name.
    let (p, _) = loader_project("RuleLoader", "shop");
    p.put(
        "src/shop/RuleLoader.java",
        b"package shop;\n\nfinal class RuleLoader extends ClassLoader {\n}\n",
    );
    let v = nonheap(&p, &loader_files(&p, "quotes.RuleLoader"));
    assert_eq!(v["tied"], json!([]), "{v}");
    // The group is a longer name that starts with the holder's type.
    let (p, _) = loader_project("RuleLoader", "quotes");
    let v = nonheap(&p, &loader_files(&p, "quotes.RuleLoaderPlus"));
    assert_eq!(v["tied"], json!([]), "{v}");
    // A JDK loader never ties, whatever the holder's declared type.
    let (p, _) = loader_project("RuleLoader", "quotes");
    let v = nonheap(
        &p,
        &loader_files(&p, "jdk.internal.loader.ClassLoaders$AppClassLoader"),
    );
    assert_eq!(v["tied"], json!([]), "{v}");
    // A finding of the same file that holds something else is not tied.
    let (p, _) = loader_project("java.util.List", "quotes");
    let v = nonheap(&p, &loader_files(&p, "quotes.RuleLoader"));
    assert_eq!(v["tied"], json!([]), "{v}");
}

#[test]
fn two_findings_holding_the_same_loader_class_tie_none() {
    let (p, a) = loader_project("RuleLoader", "quotes");
    p.put(
        "src/quotes/Other.java",
        b"package quotes;\n\nfinal class Other {\n\n\n\n    static RuleLoader SECOND = null;\n}\n",
    );
    let b = census_finding(&p, "quotes.Other.SECOND", "src/quotes/Other.java:7", &[]);
    let v = nonheap(&p, &loader_files(&p, "quotes.RuleLoader"));
    assert_eq!(v["tied"], json!([]), "{v}");
    let amb = v["ambiguous"].to_string();
    assert!(amb.contains(&a) && amb.contains(&b), "{v}");
}

#[test]
fn the_list_names_the_jvm_adapters_and_their_tools() {
    let d = tempfile::tempdir().unwrap();
    let v = ok(d.path(), &["summarize", "--list", "--runtime", "jvm"]);
    let rows = v["adapters"].as_array().unwrap();
    let find = |n: &str| {
        rows.iter()
            .find(|r| r["backend"] == n)
            .unwrap_or_else(|| panic!("no adapter {n}: {v}"))
    };
    assert_eq!(find("jfr-old-object-sample")["tool"], "jfr");
    assert_eq!(find("jfr-old-object-sample")["op"], "alloc-stack-diff");
    assert!(find("jfr-old-object-text")["tool"].is_null());
    for n in ["jvm-thread-dump", "jvm-classloader-stats", "jvm-nmt"] {
        assert_eq!(find(n)["op"], "non-heap");
        assert_eq!(find(n)["runtime"], "jvm");
        assert!(find(n)["tool"].is_null());
    }
}

// ---------------------------------------------------------------------------------------
// via holders
// ---------------------------------------------------------------------------------------

const T0: u64 = 1_800_000_000_000;

/// The coroutine case of `hprof_via.rs`: `n` continuations reachable from a static map and
/// from the job tree of the main thread, so no single field dominates them.
fn coroutine_dump(ts: u64, n: usize) -> Vec<u8> {
    let mut b = B::new(8, ts);
    let obj = b.declare("java.lang.Object");
    let class_cls = b.declare("java.lang.Class");
    let string = b.declare("java.lang.String");
    let thread = b.declare("java.lang.Thread");
    let obj_arr = b.declare("[Ljava/lang/Object;");
    let byte_arr = b.declare("[B");
    let cont = b.declare("shop.LeakyKt$handle$1");
    let loader_cls = b.declare("jdk.internal.loader.ClassLoaders$AppClassLoader");
    let loader = b.instance(loader_cls, &[]);
    b.class_dump(loader_cls, obj, 0, 8, &[], &[]);
    let cs: Vec<u64> = (0..n)
        .map(|i| {
            let payload = b.byte_array(&vec![(i % 200) as u8; 2000]);
            b.instance(cont, &[F::Id(payload)])
        })
        .collect();
    let registry_cls = b.declare("shop.Registry");
    let job_cls = b.declare("shop.Job");
    let waiters_cls = b.declare("shop.Waiters");
    let table = b.obj_array(obj_arr, &cs);
    let registry = b.instance(registry_cls, &[F::Id(table)]);
    let children = b.obj_array(obj_arr, &cs);
    let job = b.instance(job_cls, &[F::Id(children)]);
    let tname = b.java_string(string, "main");
    let tobj = b.instance(thread, &[F::Id(tname)]);
    b.root(0x08, tobj, 1, 5);
    b.root(0x03, job, 1, 0);
    b.frame(0x50, "runBlocking", "Builders.kt", 1, 40);
    b.trace(5, 1, &[0x50]);
    b.class_dump(obj, 0, 0, 0, &[], &[]);
    b.class_dump(class_cls, obj, 0, 0, &[], &[]);
    b.class_dump(
        string,
        obj,
        0,
        0,
        &[("value", T_OBJ), ("coder", T_BYTE), ("hash", 10)],
        &[],
    );
    b.class_dump(thread, obj, 0, 0, &[("name", T_OBJ)], &[]);
    b.class_dump(obj_arr, obj, 0, 0, &[], &[]);
    b.class_dump(byte_arr, obj, 0, 0, &[], &[]);
    b.class_dump(cont, obj, loader, 0, &[("payload", T_OBJ)], &[]);
    b.class_dump(registry_cls, obj, 0, 0, &[("table", T_OBJ)], &[]);
    b.class_dump(job_cls, obj, 0, 0, &[("children", T_OBJ)], &[]);
    b.class_dump(waiters_cls, obj, loader, 0, &[], &[("gates", registry)]);
    b.finish(2)
}

fn coroutine_dumps(p: &Project) -> Vec<String> {
    [100, 200, 300]
        .iter()
        .enumerate()
        .map(|(i, n)| {
            p.put(
                &format!("dumps/d{i}.hprof"),
                &coroutine_dump(T0 + 30_000 * i as u64, *n),
            )
        })
        .collect()
}

fn heap_graph(p: &Project, files: &[String], extra: &[&str]) -> Value {
    let mut args = vec!["summarize", "heap-graph"];
    args.extend(files.iter().map(String::as_str));
    args.extend_from_slice(extra);
    ok(p.path(), &args)
}

const WAITERS_KT: &str =
    "package shop\n\nobject Waiters {\n    // the leak\n    val gates = Registry()\n}\n";

#[test]
fn a_via_holder_is_listed_followed_in_the_series_and_tied_by_its_owner() {
    let p = Project::new();
    p.put("src/shop/Waiters.kt", WAITERS_KT.as_bytes());
    let f = census_finding(&p, "shop.Waiters.gates", "src/shop/Waiters.kt:5", &[]);
    let files = coroutine_dumps(&p);
    let v = heap_graph(&p, &files, &[]);
    // Listed as a holder in the output.
    let via = v["via_holders"].as_array().unwrap_or_else(|| panic!("{v}"));
    assert_eq!(via[0]["key"], "static:shop.Waiters.gates");
    assert_eq!(via[0]["by"], "root");
    // A column of its own that passes the gate and ties by holder identity.
    assert_eq!(v["gate"], "pass", "{v}");
    let g = &p.evidence("growth-series")[0];
    let col = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == "holder:static:shop.Waiters.gates")
        .unwrap_or_else(|| panic!("{g}"));
    assert_eq!(col["finding"], f.as_str());
    assert_eq!(col["tie"]["rule"], "holder");
    // The holder kind observed rides on the passing column, for the ledger.
    assert_eq!(col["holder_kind"], "static/global");
    assert_eq!(v["tied"], json!([f]));
    // The retention-path evidence names it too.
    let r = &p.evidence("retention-path")[2];
    assert_eq!(r["result"]["via"][0]["key"], "static:shop.Waiters.gates");
    // The series file has the column with its figures.
    let s: Value = serde_json::from_str(
        &fs::read_to_string(
            p.run_dir
                .join(format!("series/{}.json", v["series"].as_str().unwrap())),
        )
        .unwrap(),
    )
    .unwrap();
    let c = s["columns"]
        .as_array()
        .unwrap()
        .iter()
        .position(|c| c["name"] == "holder:static:shop.Waiters.gates")
        .unwrap();
    let vals: Vec<f64> = s["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pt| pt["values"][c].as_f64().unwrap())
        .collect();
    assert!(vals[0] < vals[1] && vals[1] < vals[2], "{vals:?}");
}

/// Two pooled threads whose `ThreadLocal` (held by the static `shop.Ctx.BUF`) holds a buffer of
/// `n` kilobytes each, as the entries of their `threadLocals` maps.
fn thread_local_dump(ts: u64, n: usize) -> Vec<u8> {
    thread_local_dump_with(ts, n, 0)
}

/// As [`thread_local_dump`], with `big` static fields of other classes (`shop.Big<k>.DATA`)
/// that each keep 400 KB alive, so they are the largest holders of every dump.
fn thread_local_dump_with(ts: u64, n: usize, big: usize) -> Vec<u8> {
    let mut b = B::new(8, ts);
    let obj = b.declare("java.lang.Object");
    let class_cls = b.declare("java.lang.Class");
    let string = b.declare("java.lang.String");
    let thread = b.declare("java.lang.Thread");
    let obj_arr = b.declare("[Ljava/lang/Object;");
    let byte_arr = b.declare("[B");
    let reference = b.declare("java.lang.ref.Reference");
    let weak = b.declare("java.lang.ref.WeakReference");
    let entry = b.declare("java.lang.ThreadLocal$ThreadLocalMap$Entry");
    let map = b.declare("java.lang.ThreadLocal$ThreadLocalMap");
    let tl_cls = b.declare("java.lang.ThreadLocal");
    let buf_cls = b.declare("shop.Buffer");
    let ctx = b.declare("shop.Ctx");
    let loader_cls = b.declare("jdk.internal.loader.ClassLoaders$AppClassLoader");
    let loader = b.instance(loader_cls, &[]);
    b.class_dump(loader_cls, obj, 0, 8, &[], &[]);
    let tl = b.instance(tl_cls, &[]);
    for (i, name) in ["pool-1", "pool-2"].into_iter().enumerate() {
        // `n` segments of 1 KB: the object count grows with the values, as for a list.
        let segs: Vec<u64> = (0..n).map(|_| b.byte_array(&vec![i as u8; 1024])).collect();
        let payload = b.obj_array(obj_arr, &segs);
        let value = b.instance(buf_cls, &[F::Id(payload)]);
        let e = b.instance(entry, &[F::Id(value), F::Id(tl)]);
        let table = b.obj_array(obj_arr, &[e]);
        let m = b.instance(map, &[F::Id(table)]);
        let tname = b.java_string(string, name);
        let t = b.instance(thread, &[F::Id(tname), F::Id(m), F::Id(0)]);
        b.root(0x08, t, i as u32 + 1, 5);
    }
    for k in 0..big {
        let cls = b.declare(&format!("shop.Big{k}"));
        let data = b.byte_array(&vec![7u8; 400_000]);
        b.class_dump(cls, obj, loader, 0, &[], &[("DATA", data)]);
    }
    b.class_dump(obj, 0, 0, 0, &[], &[]);
    b.class_dump(class_cls, obj, 0, 0, &[], &[]);
    b.class_dump(
        string,
        obj,
        0,
        0,
        &[("value", T_OBJ), ("coder", T_BYTE), ("hash", 10)],
        &[],
    );
    b.class_dump(
        thread,
        obj,
        0,
        0,
        &[
            ("name", T_OBJ),
            ("threadLocals", T_OBJ),
            ("inheritableThreadLocals", T_OBJ),
        ],
        &[],
    );
    b.class_dump(obj_arr, obj, 0, 0, &[], &[]);
    b.class_dump(byte_arr, obj, 0, 0, &[], &[]);
    b.class_dump(reference, obj, 0, 0, &[("referent", T_OBJ)], &[]);
    b.class_dump(weak, reference, 0, 0, &[], &[]);
    b.class_dump(entry, weak, 0, 0, &[("value", T_OBJ)], &[]);
    b.class_dump(map, obj, 0, 0, &[("table", T_OBJ)], &[]);
    b.class_dump(tl_cls, obj, 0, 0, &[], &[]);
    b.class_dump(buf_cls, obj, loader, 0, &[("payload", T_OBJ)], &[]);
    b.class_dump(ctx, obj, loader, 0, &[], &[("BUF", tl)]);
    b.finish(2)
}

#[test]
fn thread_local_values_on_pooled_threads_tie_to_the_static_field_and_report_thread_local() {
    let p = Project::new();
    p.put(
        "src/shop/Ctx.java",
        b"package shop;\n\nfinal class Ctx {\n    // the leak\n    static final ThreadLocal<Buffer> BUF = new ThreadLocal<>();\n}\n",
    );
    let f = census_finding(&p, "shop.Ctx.BUF", "src/shop/Ctx.java:5", &[]);
    let files: Vec<String> = [20, 40, 60]
        .iter()
        .enumerate()
        .map(|(i, n)| {
            p.put(
                &format!("dumps/d{i}.hprof"),
                &thread_local_dump(T0 + 30_000 * i as u64, *n),
            )
        })
        .collect();
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([f]), "{v}");
    let g = &p.evidence("growth-series")[0];
    let col = g["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == "holder:thread-local:shop.Ctx.BUF")
        .unwrap_or_else(|| panic!("{g}"));
    assert_eq!(col["finding"], f.as_str());
    assert_eq!(col["tie"]["rule"], "holder");
    assert_eq!(col["holder_kind"], "thread-local");
    // No thread holder was added as a second candidate for the same values.
    assert!(
        p.findings().iter().all(|x| x["origin"] == "census"),
        "{:?}",
        p.findings()
    );
}

#[test]
fn a_via_holder_without_a_finding_becomes_a_candidate_with_the_declaration_site() {
    let p = Project::new();
    p.put("src/shop/Waiters.kt", WAITERS_KT.as_bytes());
    let files = coroutine_dumps(&p);
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["gate"], "pass", "{v}");
    let c = &v["candidates"][0];
    assert_eq!(c["owner"], "shop.Waiters.gates", "{v}");
    assert_eq!(c["site"], "src/shop/Waiters.kt:5", "{v}");
    assert_eq!(p.findings().len(), 1);
}

#[test]
fn via_holders_on_one_dump_are_listed_and_a_series_needs_three() {
    let p = Project::new();
    let files = vec![p.put("dumps/d0.hprof", &coroutine_dump(T0, 100))];
    let v = heap_graph(&p, &files, &[]);
    assert_eq!(v["via_holders"][0]["key"], "static:shop.Waiters.gates");
    assert_eq!(v["gate"], "fail");
    assert!(p.evidence("growth-series").is_empty());
}

fn thread_local_files(p: &Project, big: usize) -> Vec<String> {
    [20, 40, 60]
        .iter()
        .enumerate()
        .map(|(i, n)| {
            p.put(
                &format!("dumps/d{i}.hprof"),
                &thread_local_dump_with(T0 + 30_000 * i as u64, *n, big),
            )
        })
        .collect()
}

fn holder_keys(r: &Value) -> Vec<&str> {
    r["result"]["holders"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["key"].as_str().unwrap())
        .collect()
}

#[test]
fn retention_paths_state_the_holder_kind_of_each_holder() {
    let p = Project::new();
    let files = thread_local_files(&p, 0);
    heap_graph(&p, &files, &[]);
    let r = p.evidence("retention-path");
    assert_eq!(r.len(), 3);
    let holders = r[2]["result"]["holders"].as_array().unwrap();
    assert!(!holders.is_empty());
    for h in holders {
        assert!(
            h["holder_kind"].is_string(),
            "every holder entry states its kind: {h}"
        );
    }
    let tl = holders
        .iter()
        .find(|h| h["key"] == "thread-local:shop.Ctx.BUF")
        .unwrap_or_else(|| panic!("{:?}", holders));
    assert_eq!(tl["holder_kind"], "thread-local");
}

#[test]
fn the_holder_a_finding_names_is_in_every_retention_path_record_even_when_small() {
    let p = Project::new();
    p.put(
        "src/shop/Ctx.java",
        b"package shop;\n\nfinal class Ctx {\n    // the leak\n    static final ThreadLocal<Buffer> BUF = new ThreadLocal<>();\n}\n",
    );
    let f = census_finding(&p, "shop.Ctx.BUF", "src/shop/Ctx.java:5", &[]);
    let files = thread_local_files(&p, 3);
    // Without --finding the three largest holders are the other classes'.
    heap_graph(&p, &files, &[]);
    for r in p.evidence("retention-path") {
        assert!(
            !holder_keys(&r).contains(&"thread-local:shop.Ctx.BUF"),
            "{r}"
        );
    }
    // With --finding the holder it resolved to is named by every dump's record.
    let q = Project::new();
    q.put(
        "src/shop/Ctx.java",
        b"package shop;\n\nfinal class Ctx {\n    // the leak\n    static final ThreadLocal<Buffer> BUF = new ThreadLocal<>();\n}\n",
    );
    let g = census_finding(&q, "shop.Ctx.BUF", "src/shop/Ctx.java:5", &[]);
    assert_eq!(f, g);
    let files = thread_local_files(&q, 3);
    let v = heap_graph(&q, &files, &["--finding", &g]);
    assert_eq!(v["tied"], json!([g]), "{v}");
    let recs = q.evidence("retention-path");
    assert_eq!(recs.len(), 3);
    for r in &recs {
        let keys = holder_keys(r);
        assert!(keys.contains(&"thread-local:shop.Ctx.BUF"), "{r}");
        assert!(keys.len() <= 4, "the largest three and the finding's: {r}");
        assert!(
            serde_json::to_string(&r["result"]).unwrap().len() <= 2048,
            "{r}"
        );
        let h = r["result"]["holders"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["key"] == "thread-local:shop.Ctx.BUF")
            .unwrap();
        assert_eq!(h["holder_kind"], "thread-local");
    }
}
