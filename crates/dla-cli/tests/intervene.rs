//! `dla intervene` through the binary on tiny generated projects.
//!
//! Python and Go run here (each test skips, printing why, when its toolchain is missing).
//! JVM is not covered: its agent jar and driver are built from sources that change with the
//! benchmark fixtures, and a JVM harness takes much longer than a tiny test should.
//!
//! What is checked: a stubbed real writer removes the growth and `dla ledger strong`
//! accepts the record; a harmless patch leaves the growth in place and `strong` refuses;
//! the refusals (not a writer, not a statement, outside the allowed region, driver, test
//! file, too large); the project and the original harness are never changed.
#![cfg(unix)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::Value;

fn on_path(tool: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(tool).is_file()))
}

/// True (after printing why) when the toolchain is missing.
fn skip_without(tool: &str) -> bool {
    if on_path(tool) {
        return false;
    }
    eprintln!("skipping: {tool} is not on PATH (dla intervene builds and runs the harness)");
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

/// The refusal's code, after checking it names what is missing and how to fix it.
fn refused(project: &Path, args: &[&str]) -> (String, Value) {
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
    (e["code"].as_str().unwrap().to_owned(), e.clone())
}

fn write_project(root: &Path, files: &[(&str, &str)]) {
    for (rel, text) in files {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
}

/// Every file of the project outside `.dla`, by path.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for e in fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.file_name().is_some_and(|n| n == ".dla") {
                continue;
            }
            if p.is_dir() {
                walk(root, &p, out);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                out.insert(rel, fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn command_of(next: &str) -> Option<Vec<String>> {
    let cmd = next.split("   #").next()?.split("   (").next()?.trim();
    let words: Vec<String> = cmd
        .strip_prefix("dla ")?
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    if words.first().map(String::as_str) != Some("ledger")
        || !matches!(words.get(1).map(String::as_str), Some("refute" | "confirm"))
    {
        return None;
    }
    Some(words)
}

fn run_next(project: &Path, next: &Value) {
    for n in next.as_array().unwrap() {
        if let Some(words) = command_of(n.as_str().unwrap()) {
            let args: Vec<&str> = words.iter().map(String::as_str).collect();
            ok(project, &args);
        }
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

fn state_of(run_dir: &Path, finding: &str) -> String {
    let text = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["type"] == "transition" && v["finding"] == finding)
        .filter_map(|v| v["to"].as_str().map(str::to_owned))
        .next_back()
        .unwrap_or_else(|| "candidate".to_owned())
}

fn evidence_record(run_dir: &Path, id: &str) -> Value {
    let text = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap();
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|v| v["evidence_id"] == id)
        .unwrap_or_else(|| panic!("no evidence {id}"))
}

/// A project with a confirmed finding on harness `H1`.
struct Confirmed {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
    finding: String,
}

/// Generates the project, runs discovery, harness and trend, and confirms the finding at
/// `site` with the commands the tools print. `gen_args` are extra `harness gen` arguments.
fn confirmed(files: &[(&str, &str)], lang: &str, gen_args: &[&str], site: &str) -> Confirmed {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write_project(&project, files);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", lang]);
    ok(&project, &["workload", "discover", "--lang", lang]);
    let mut gen_cmd = vec!["harness", "gen", "--lang", lang, "--step", "150"];
    gen_cmd.extend_from_slice(gen_args);
    ok(&project, &gen_cmd);
    let run = ok(&project, &["harness", "run", "H1"]);
    let workload = run["evidence"][0].as_str().unwrap().to_owned();
    ok(&project, &["trend", "S1"]);
    let finding = finding_at(&run_dir, site);
    ok(
        &project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &finding,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            &workload,
        ],
    );
    ok(
        &project,
        &["ledger", "test", &finding, "--evidence", &workload],
    );
    let a = ok(&project, &["alloc-test", "H1", "--finding", &finding]);
    assert_eq!(a["favors"], "leak", "{a}");
    run_next(&project, &a["next"]);
    assert_eq!(state_of(&run_dir, &finding), "confirmed");
    Confirmed {
        _dir: dir,
        project,
        run_dir,
        finding,
    }
}

// ---------------------------------------------------------------------------------------
// Python

const PY_FILES: [(&str, &str); 4] = [
    ("app/__init__.py", ""),
    (
        "app/audit.py",
        "\"\"\"Request audit trail.\"\"\"\n\nTRAIL = []\nSEEN_IDS = set()\n\n\ndef note(entry):\n    TRAIL.append(entry)\n\n\ndef touch(key):\n    SEEN_IDS.add(key % 4)\n",
    ),
    (
        "app/service.py",
        "from . import audit\n\n\ndef handle(n):\n    audit.note({\"request\": n, \"payload\": \"x\" * 64})\n    audit.touch(n)\n    return n * 2\n",
    ),
    (
        "main.py",
        "import sys\n\nfrom app import service\n\n\ndef main():\n    n = int(sys.argv[1]) if len(sys.argv) > 1 else 100\n    total = 0\n    for i in range(n):\n        total += service.handle(i)\n    print(total)\n\n\nif __name__ == \"__main__\":\n    main()\n",
    ),
];

fn python() -> Option<Confirmed> {
    if skip_without("python3") {
        return None;
    }
    Some(confirmed(&PY_FILES, "python", &[], "app/audit.py:3"))
}

fn write_diff(c: &Confirmed, name: &str, text: &str) -> String {
    let p = c.run_dir.join(name);
    fs::write(&p, text).unwrap();
    p.to_string_lossy().into_owned()
}

#[test]
fn python_stubbing_the_real_writer_removes_the_growth_and_strong_accepts_it() {
    let Some(c) = python() else { return };
    let before = snapshot(&c.project);
    let h1_work = snapshot(&c.run_dir.join("harness/H1/work"));
    let f = c.finding.as_str();

    let i = ok(
        &c.project,
        &[
            "intervene",
            "H1",
            "--finding",
            f,
            "--stub-writer",
            "app/audit.py:8",
        ],
    );
    assert_eq!(i["effect"], "growth-removed", "{i}");
    assert_eq!(i["treated_harness"], "H2");
    assert_eq!(i["kind"], "stub-writer");
    assert_eq!(i["control"]["trend"], "T1");
    let guards = i["guards"].as_object().unwrap();
    for g in ["build_ok", "run_ok", "same_points", "edit_scoped"] {
        assert_eq!(guards[g], true, "{g}: {i}");
    }
    assert!(guards["exercised"].is_null());
    assert!(i["note"].as_str().unwrap().contains("scratch copy"));
    let evidence = i["evidence"].as_str().unwrap();
    assert!(
        i["next"][0]
            .as_str()
            .unwrap()
            .starts_with(&format!("dla ledger strong {f} --evidence {evidence}")),
        "{i}"
    );

    // The evidence record: kind, authority, shape, signal, bounded.
    let rec = evidence_record(&c.run_dir, evidence);
    assert_eq!(rec["kind"], "intervention");
    assert_eq!(rec["authority"], "test");
    assert_eq!(rec["result"]["test"], "intervention");
    assert_eq!(rec["result"]["finding"], f);
    assert_eq!(rec["result"]["effect"], "growth-removed");
    assert_eq!(rec["result"]["signal"]["class"], "intervention");
    assert_eq!(rec["result"]["signal"]["tool"], "dla-intervene");
    assert_eq!(rec["result"]["treated"]["gate"], "fail");
    assert!(serde_json::to_string(&rec["result"]).unwrap().len() <= 2048);
    assert_eq!(rec["raw"], "intervene/I1");
    let diff = fs::read_to_string(c.run_dir.join("intervene/I1/patch.diff")).unwrap();
    assert!(diff.contains("-    TRAIL.append(entry)") && diff.contains("+    pass"));
    assert!(c.run_dir.join("intervene/I1/trend.json").is_file());
    assert_eq!(rec["result"]["diff_sha256"], i["diff_sha256"]);

    // The edit is in the clone only.
    let edited = fs::read_to_string(c.run_dir.join("harness/H2/work/app/audit.py")).unwrap();
    assert!(edited.contains("    pass\n") && !edited.contains("TRAIL.append"));
    assert_eq!(before, snapshot(&c.project), "the project changed");
    assert_eq!(
        h1_work,
        snapshot(&c.run_dir.join("harness/H1/work")),
        "the original harness changed"
    );

    // The ledger accepts the record as the second class.
    let s = ok(&c.project, &["ledger", "strong", f, "--evidence", evidence]);
    assert_eq!(s["to"], "strong");
    assert_eq!(state_of(&c.run_dir, f), "strong");
}

#[test]
fn python_rerun_control_reproduces_and_is_recorded() {
    let Some(c) = python() else { return };
    let i = ok(
        &c.project,
        &[
            "intervene",
            "H1",
            "--finding",
            &c.finding,
            "--stub-writer",
            "app/audit.py:8",
            "--rerun-control",
        ],
    );
    assert_eq!(i["effect"], "growth-removed", "{i}");
    assert_eq!(i["guards"]["control_reproduced"], true, "{i}");
}

#[test]
fn python_a_harmless_patch_leaves_the_growth_and_strong_refuses() {
    let Some(c) = python() else { return };
    let f = c.finding.as_str();
    // A statement that is not a recorded writer cannot be stubbed.
    let (code, _) = refused(
        &c.project,
        &[
            "intervene",
            "H1",
            "--finding",
            f,
            "--stub-writer",
            "app/audit.py:12",
        ],
    );
    assert_eq!(code, "intervene-not-writer");
    assert!(
        !c.run_dir.join("harness/H2").exists(),
        "a refusal leaves no clone"
    );

    // A patch near the writer that does not change what is stored.
    let diff = "--- a/app/audit.py\n+++ b/app/audit.py\n@@ -10,3 +10,3 @@\n \n def touch(key):\n-    SEEN_IDS.add(key % 4)\n+    SEEN_IDS.add(key % 8)\n";
    let path = write_diff(&c, "harmless.diff", diff);
    let i = ok(
        &c.project,
        &["intervene", "H1", "--finding", f, "--patch", &path],
    );
    assert_eq!(i["effect"], "growth-remains", "{i}");
    assert_eq!(i["kind"], "patch");
    assert!(
        i["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.as_str().unwrap().contains("does not refute"))
    );
    let evidence = i["evidence"].as_str().unwrap();

    // The attempt is recorded; it neither makes the finding strong nor refutes it.
    let (code, e) = refused(&c.project, &["ledger", "strong", f, "--evidence", evidence]);
    assert!(code.contains("strong"), "{code}");
    assert!(
        e["message"].as_str().unwrap().contains("did not remove"),
        "{e}"
    );
    assert_eq!(state_of(&c.run_dir, f), "confirmed");
}

#[test]
fn python_refusals_name_the_reason() {
    let Some(c) = python() else { return };
    let f = c.finding.as_str();
    let before = snapshot(&c.project);

    // A recorded writer that is not a standalone statement (a def line).
    ok(
        &c.project,
        &["ledger", "annotate", f, "--writer", "app/audit.py:7"],
    );
    let (code, e) = refused(
        &c.project,
        &[
            "intervene",
            "H1",
            "--finding",
            f,
            "--stub-writer",
            "app/audit.py:7",
        ],
    );
    assert_eq!(code, "intervene-not-statement", "{e}");

    // A patch far from every recorded site.
    let far = "--- a/main.py\n+++ b/main.py\n@@ -6,3 +6,3 @@\n def main():\n-    n = int(sys.argv[1]) if len(sys.argv) > 1 else 100\n+    n = 100\n     total = 0\n";
    let path = write_diff(&c, "far.diff", far);
    let (code, e) = refused(
        &c.project,
        &["intervene", "H1", "--finding", f, "--patch", &path],
    );
    assert_eq!(code, "intervene-scope", "{e}");
    assert!(e["message"].as_str().unwrap().contains("within 10 lines"));

    // A patch that touches the harness driver, then a test file.
    let driver = "--- a/driver/loop.py\n+++ b/driver/loop.py\n@@ -1 +1 @@\n-a\n+b\n";
    let path = write_diff(&c, "driver.diff", driver);
    let (code, _) = refused(
        &c.project,
        &["intervene", "H1", "--finding", f, "--patch", &path],
    );
    assert_eq!(code, "intervene-scope");
    let test = "--- a/tests/test_audit.py\n+++ b/tests/test_audit.py\n@@ -1 +1 @@\n-a\n+b\n";
    let path = write_diff(&c, "test.diff", test);
    let (code, _) = refused(
        &c.project,
        &["intervene", "H1", "--finding", f, "--patch", &path],
    );
    assert_eq!(code, "intervene-scope");

    // A path that leaves work/.
    let escape = "--- a/../driver/x.py\n+++ b/../driver/x.py\n@@ -1 +1 @@\n-a\n+b\n";
    let path = write_diff(&c, "escape.diff", escape);
    let (code, _) = refused(
        &c.project,
        &["intervene", "H1", "--finding", f, "--patch", &path],
    );
    assert_eq!(code, "intervene-scope");

    // A diff that does not apply, and one that is too large.
    let stale = "--- a/app/audit.py\n+++ b/app/audit.py\n@@ -8,1 +8,1 @@\n-    TRAIL.add(entry)\n+    pass\n";
    let path = write_diff(&c, "stale.diff", stale);
    let (code, _) = refused(
        &c.project,
        &["intervene", "H1", "--finding", f, "--patch", &path],
    );
    assert_eq!(code, "intervene-patch");
    let mut big = String::from(
        "--- a/app/audit.py\n+++ b/app/audit.py\n@@ -8,1 +8,42 @@\n     TRAIL.append(entry)\n",
    );
    for i in 0..41 {
        big.push_str(&format!("+    # line {i}\n"));
    }
    let path = write_diff(&c, "big.diff", &big);
    let (code, _) = refused(
        &c.project,
        &["intervene", "H1", "--finding", f, "--patch", &path],
    );
    assert_eq!(code, "intervene-too-large");

    assert!(
        !c.run_dir.join("harness/H2").exists(),
        "a refusal leaves no clone"
    );
    assert!(
        !c.run_dir.join("intervene").exists()
            || fs::read_dir(c.run_dir.join("intervene"))
                .unwrap()
                .next()
                .is_none()
    );
    assert_eq!(before, snapshot(&c.project));
}

#[test]
fn python_a_candidate_finding_is_not_tested() {
    let Some(c) = python() else { return };
    // The second holder was never confirmed.
    let other = finding_at(&c.run_dir, "app/audit.py:4");
    let (code, _) = refused(
        &c.project,
        &[
            "intervene",
            "H1",
            "--finding",
            &other,
            "--stub-writer",
            "app/audit.py:12",
        ],
    );
    assert_eq!(code, "intervene-state");
    let (code, _) = refused(
        &c.project,
        &[
            "intervene",
            "H9",
            "--finding",
            &c.finding,
            "--stub-writer",
            "app/audit.py:8",
        ],
    );
    assert!(!code.is_empty());
}

// ---------------------------------------------------------------------------------------
// Go

const GO_FILES: [(&str, &str); 5] = [
    ("go.mod", "module example.com/trail\n\ngo 1.21\n"),
    (
        "main.go",
        "package main\n\nimport (\n\t\"fmt\"\n\t\"os\"\n\t\"strconv\"\n)\n\nfunc main() {\n\tn := 100\n\tif len(os.Args) > 1 {\n\t\tif v, err := strconv.Atoi(os.Args[1]); err == nil {\n\t\t\tn = v\n\t\t}\n\t}\n\ttotal := 0\n\tfor i := 0; i < n; i++ {\n\t\ttotal += handle(i)\n\t}\n\tfmt.Println(total)\n}\n",
    ),
    (
        "worker.go",
        "package main\n\nvar gate = make(chan struct{})\n\nvar seen = map[int]bool{}\n\nfunc startWorker(id int) {\n\tgo func() {\n\t\t<-gate\n\t\t_ = id\n\t}()\n}\n\nfunc touch(key int) {\n\tseen[key%4] = true\n}\n",
    ),
    (
        "service.go",
        "package main\n\nfunc handle(n int) int {\n\tstartWorker(n)\n\ttouch(n)\n\treturn n * 2\n}\n",
    ),
    (
        "doc.go",
        "// Package main is a tiny worker pool.\npackage main\n",
    ),
];

#[test]
fn go_stubbing_the_call_that_starts_the_goroutines_removes_the_growth() {
    if skip_without("go") {
        return;
    }
    let c = confirmed(
        &GO_FILES,
        "go",
        &["--workload", "go:main:.:.", "--iter-arg", "{n}"],
        "worker.go:8",
    );
    let f = c.finding.as_str();
    let before = snapshot(&c.project);

    // Nothing records `service.go:4` as a writer yet, so the stub is refused with the fix.
    let (code, e) = refused(
        &c.project,
        &[
            "intervene",
            "H1",
            "--finding",
            f,
            "--stub-writer",
            "service.go:4",
        ],
    );
    assert_eq!(code, "intervene-not-writer");
    assert!(e["fix"].as_str().unwrap().contains("ledger annotate"));
    ok(
        &c.project,
        &["ledger", "annotate", f, "--writer", "service.go:4"],
    );

    // `touch(n)` is a statement of the same function, but not a writer.
    let (code, _) = refused(
        &c.project,
        &[
            "intervene",
            "H1",
            "--finding",
            f,
            "--stub-writer",
            "service.go:5",
        ],
    );
    assert_eq!(code, "intervene-not-writer");

    let i = ok(
        &c.project,
        &[
            "intervene",
            "H1",
            "--finding",
            f,
            "--stub-writer",
            "service.go:4",
        ],
    );
    assert_eq!(i["effect"], "growth-removed", "{i}");
    assert_eq!(i["control"]["column"], "go.goroutines@worker.go:8");
    // The goroutine column is gone from the treated series; the rest of it is there.
    assert_eq!(i["treated"]["present"], false, "{i}");
    assert_eq!(i["guards"]["build_ok"], true);
    let edited = fs::read_to_string(c.run_dir.join("harness/H2/work/service.go")).unwrap();
    assert!(edited.contains("\t;\n\ttouch(n)"), "{edited}");
    assert_eq!(before, snapshot(&c.project));

    let evidence = i["evidence"].as_str().unwrap();
    ok(&c.project, &["ledger", "strong", f, "--evidence", evidence]);
    assert_eq!(state_of(&c.run_dir, f), "strong");
}

#[test]
fn go_a_stub_that_stops_the_build_is_invalid_and_never_evidence() {
    if skip_without("go") {
        return;
    }
    let c = confirmed(
        &GO_FILES,
        "go",
        &["--workload", "go:main:.:.", "--iter-arg", "{n}"],
        "worker.go:8",
    );
    let f = c.finding.as_str();
    // Removing the only use of a local makes Go refuse to compile: `declared and not used`.
    let diff = "--- a/service.go\n+++ b/service.go\n@@ -3,3 +3,4 @@\n func handle(n int) int {\n \tstartWorker(n)\n+\tunused := 1\n \ttouch(n)\n";
    ok(
        &c.project,
        &["ledger", "annotate", f, "--writer", "service.go:4"],
    );
    let path = write_diff(&c, "broken.diff", diff);
    let i = ok(
        &c.project,
        &["intervene", "H1", "--finding", f, "--patch", &path],
    );
    assert_eq!(i["effect"], "invalid", "{i}");
    assert_eq!(i["guards"]["build_ok"], false, "{i}");
    let evidence = i["evidence"].as_str().unwrap();
    let (code, _) = refused(&c.project, &["ledger", "strong", f, "--evidence", evidence]);
    assert!(code.contains("strong"), "{code}");
    assert_eq!(state_of(&c.run_dir, f), "confirmed");
}
