//! A leak whose own drivers disagree is not confirmed until the difference is tested, and growth
//! that exists only while the caller skips a release the API requires is no leak of the library.
//!
//! The scene is the one a real run showed, built from records: an encoder whose holder
//! (`lib/enc.py:5`) is released by `stop()` (`lib/enc.py:30`). H10 stops every unit and its
//! alloc-test favored non-leak; H11 skips `stop()` on half the units and its alloc-test, the
//! newer one, favored leak.

use std::{fs, path::PathBuf};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, HarnessManifest, Ledger, NewEvidence, Source, Tool,
    evidence::EvidenceCoverage, harness::LineRuns,
};
use serde_json::{Value, json};

fn command(project: &std::path::Path) -> Command {
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

struct R {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
    runtime: String,
    /// The workload evidence of each harness and the growth evidence.
    growth: String,
}

impl R {
    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        (
            o.status.code().unwrap(),
            serde_json::from_str(&text).unwrap_or(Value::Null),
        )
    }
    /// The stdout of a command that prints lines of JSON (`dla ledger list`) or Markdown.
    fn text(&self, args: &[&str]) -> String {
        let o = command(self.dir.path()).args(args).output().unwrap();
        assert!(o.status.success(), "{args:?}");
        String::from_utf8(o.stdout).unwrap()
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }
    fn refused(&self, args: &[&str], code: &str) -> Value {
        let (exit, v) = self.run(args);
        assert_ne!(exit, 0, "{args:?} was accepted: {v}");
        assert_eq!(v["error"]["code"], code, "{args:?}: {v}");
        v["error"].clone()
    }
    fn evidence(&self, kind: EvidenceKind, result: Value) -> String {
        let mut l = Ledger::load(&self.run_dir).unwrap();
        l.add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: Authority::Test,
            kind,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        l.evidence().last().unwrap().evidence_id.to_string()
    }
    /// An alloc-test of `harness` bound to F1 that favors `favors`.
    fn alloc(&self, harness: &str, favors: &str) -> String {
        self.evidence(
            EvidenceKind::Discriminator,
            json!({"test": "plateau+purge", "favors": favors, "supports": [],
                "finding": "F1", "harness": harness, "series": "S1"}),
        )
    }
    /// A driver that targets F1 and the workload evidence that ran it.
    fn driver(&self, harness: &str) -> String {
        let m: HarnessManifest = serde_json::from_value(json!({
            "id": harness, "runtime": self.runtime, "mode": "driver",
            "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
            "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
            "targets": ["F1"], "agent": "driver-python", "timeout_secs": 60,
        }))
        .unwrap();
        m.save(&self.run_dir).unwrap();
        self.evidence(
            EvidenceKind::Workload,
            json!({"harness": harness, "mode": "driver", "runtime": self.runtime,
                "series": ["S1"]}),
        )
    }
    /// What the Python agent counted at `lib/enc.py:30` over the points of a run of `harness`.
    fn release_counts(&self, harness: &str, counts: Value) {
        LineRuns {
            dla_line_runs: 1,
            series: "S1".into(),
            sites: serde_json::from_value(json!({"lib/enc.py:30": counts})).unwrap(),
            unavailable: None,
        }
        .save(&self.run_dir, harness)
        .unwrap();
    }
    /// What the Python agent counted at each of `sites` over the points of a run of `harness`.
    fn release_counts_at(&self, harness: &str, sites: Value) {
        LineRuns {
            dla_line_runs: 1,
            series: "S1".into(),
            sites: serde_json::from_value(sites).unwrap(),
            unavailable: None,
        }
        .save(&self.run_dir, harness)
        .unwrap();
    }
    /// `dla ledger add` of the `N-RELEASE` sibling of F1 naming `lib/enc.py:30`; returns its id.
    fn add_release(&self, evidence: &str) -> String {
        self.add_release_at(evidence, "lib/enc.py:30")
    }
    /// `dla ledger add` of the `N-RELEASE` sibling of F1 naming `remover`; returns its id.
    fn add_release_at(&self, evidence: &str, remover: &str) -> String {
        let v = self.ok(&[
            "ledger",
            "add",
            "--sibling-of",
            "F1",
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-RELEASE",
            "--remover",
            remover,
            "--refute-if",
            "the holder still grows with stop() called on every unit",
            "--evidence",
            evidence,
        ]);
        v["id"].as_str().unwrap().to_owned()
    }
    fn final_gate(&self) -> Vec<Value> {
        let _ = self.run(&["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }
    fn report(&self) -> String {
        let o = command(self.dir.path()).arg("report").output().unwrap();
        String::from_utf8(o.stdout).unwrap()
    }
}

/// F1 at `lib/enc.py:5` (a thread-local holder), its sibling F1.n1 (`N-WARM`), F1 under test,
/// the harness H10 and the passing growth evidence tied to F1.
fn scene(runtime: &str) -> (R, String) {
    let dir = tempfile::tempdir().unwrap();
    let text: String = (1..=40).map(|i| format!("x{i} = {i}\n")).collect();
    fs::create_dir_all(dir.path().join("lib")).unwrap();
    fs::write(dir.path().join("lib/enc.py"), text).unwrap();
    let o = command(dir.path())
        .args(["run", "init", "--session-id", "t", "--budget", "3h"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let mut r = R {
        dir,
        run_dir,
        runtime: runtime.to_owned(),
        growth: String::new(),
    };
    r.growth = r.evidence(
        EvidenceKind::GrowthSeries,
        json!({"trend": "T1", "series": "S1", "n": 8, "gate": "pass", "failed": 0,
            "passed": [{"column": "probe:Encoder.holder", "finding": "F1",
                "site": "lib/enc.py:5"}]}),
    );
    let g = r.growth.clone();
    r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &g,
        "--site",
        "lib/enc.py:5",
        "--holder",
        "thread-local",
        "--owner",
        "Encoder.holder",
    ]);
    r.ok(&[
        "ledger",
        "add",
        "--sibling-of",
        "F1",
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM",
        "--refute-if",
        "growth continues over the extra cycles",
        "--evidence",
        &g,
    ]);
    let w10 = r.driver("H10");
    r.ok(&["ledger", "test", "F1", "--evidence", &w10]);
    (r, w10)
}

/// The scene of the real run: H10 (stops every unit) favored non-leak, then H11 (skips `stop()`
/// on half of them) favored leak. Returns the run, the two alloc-tests and H10's workload.
fn disagreeing(runtime: &str) -> (R, String, String, String) {
    let (r, w10) = scene(runtime);
    let flat = r.alloc("H10", "non-leak");
    r.driver("H11");
    let leak = r.alloc("H11", "leak");
    (r, flat, leak, w10)
}

#[test]
fn drivers_that_disagree_hold_the_confirm_and_the_refusal_asks_what_differs() {
    let (r, flat, leak, _) = disagreeing("python");
    let g = r.growth.clone();
    let e = r.refused(
        &["ledger", "confirm", "F1", "--evidence", &g],
        "drivers-disagree",
    );
    let msg = e["message"].as_str().unwrap();
    assert!(
        msg.contains(&format!("H10 ({flat}) favors non-leak")),
        "{msg}"
    );
    assert!(msg.contains(&format!("H11 ({leak}) favors leak")), "{msg}");
    assert!(
        msg.ends_with("What does H10 do that H11 does not?"),
        "{msg}"
    );
    let fix = e["fix"].as_str().unwrap();
    // The release route first, the other difference after; every id is the run's own.
    let release = fix
        .find("--hypothesis N-RELEASE --remover <file:line>")
        .expect(fix);
    let other = fix.find("or, when the difference is another").expect(fix);
    assert!(release < other, "{fix}");
    for want in [
        format!(
            "dla ledger add --sibling-of F1 --origin hypothesis --hypothesis N-RELEASE --remover <file:line> --refute-if \"<result>\" --evidence {flat}"
        ),
        "dla ledger test F1.n2 --evidence E".to_owned(),
        "dla alloc-test H10 --finding F1".to_owned(),
        "pooled or worker threads".to_owned(),
    ] {
        assert!(fix.contains(&want), "{want}\n{fix}");
    }
    // The finding was not touched.
    let list = r.text(&["ledger", "list", "--state", "under_test"]);
    assert!(list.contains("\"id\":\"F1\""), "{list}");
}

#[test]
fn a_single_driver_or_drivers_that_agree_confirm_as_before() {
    for favors in [
        vec![("H11", "leak")],
        vec![("H10", "leak"), ("H11", "leak")],
    ] {
        let (r, _, _) = {
            let (r, w) = scene("python");
            (r, w, ())
        };
        let mut last = String::new();
        for (h, f) in &favors {
            if *h == "H11" {
                r.driver("H11");
            }
            last = r.alloc(h, f);
        }
        let g = r.growth.clone();
        r.ok(&["ledger", "refute", "F1.n1", "--evidence", &last]);
        r.ok(&["ledger", "confirm", "F1", "--evidence", &g]);
    }
    // Two flat drivers do not hold anything either: the root is refuted, not confirmed.
    let (r, _) = scene("python");
    r.driver("H11");
    let a = r.alloc("H10", "non-leak");
    r.alloc("H11", "non-leak");
    r.ok(&["ledger", "refute", "F1", "--evidence", &a]);
}

#[test]
fn n_release_confirmed_on_the_flat_driver_refutes_the_root_and_the_root_cannot_be_confirmed() {
    let (r, flat, _, w10) = disagreeing("python");
    let sib = r.add_release(&flat);
    assert_eq!(sib, "F1.n2");
    r.ok(&["ledger", "test", &sib, "--evidence", &w10]);
    // The remover line ran on H10 (the agent counted it over the points).
    r.release_counts("H10", json!([0, 4, 9]));
    // The root is closed by the same alloc-test: it favors non-leak on a driver made for it.
    r.ok(&["ledger", "refute", "F1", "--evidence", &flat]);
    let v = r.ok(&["ledger", "confirm", &sib, "--evidence", &flat]);
    assert!(
        v.get("note").is_none(),
        "the Python agent counts lines: {v}"
    );
    // F1 is refuted, which is final: it cannot be confirmed as a leak.
    let g = r.growth.clone();
    let (code, v) = r.run(&["ledger", "confirm", "F1", "--evidence", &g]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "illegal-transition", "{v}");
    // The list row names the release; the report counts the explanation, not a leak.
    let list = r.text(&["ledger", "list"]);
    assert!(list.contains("lib/enc.py:30"), "{list}");
    let md = r.report();
    assert!(md.contains("## Explained, not leaks"), "{md}");
    assert!(md.contains("leak hypotheses: confirmed 0"), "{md}");
    assert!(md.contains("non-leak explanations: confirmed 1"), "{md}");
}

#[test]
fn only_a_run_in_which_the_release_ran_decides_n_release() {
    let (r, flat, _, w10) = disagreeing("python");
    let sib = r.add_release(&flat);
    r.ok(&["ledger", "test", &sib, "--evidence", &w10]);
    r.ok(&["ledger", "refute", "F1", "--evidence", &flat]);
    // The agent counted the line and it never ran: the flat series says nothing about the release.
    r.release_counts("H10", json!([0, 0, 0]));
    let e = r.refused(
        &["ledger", "confirm", &sib, "--evidence", &flat],
        "release-not-run",
    );
    assert!(
        e["message"].as_str().unwrap().contains("ran 0 times"),
        "{e}"
    );
    // The run did not ask about the line at all (it ran before the sibling named it).
    r.release_counts("H10", json!([]));
    fs::remove_file(r.run_dir.join("harness/H10/lines-S1.json")).unwrap();
    LineRuns {
        dla_line_runs: 1,
        series: "S1".into(),
        sites: serde_json::from_value(json!({"lib/enc.py:5": [1, 2]})).unwrap(),
        unavailable: None,
    }
    .save(&r.run_dir, "H10")
    .unwrap();
    let e = r.refused(
        &["ledger", "confirm", &sib, "--evidence", &flat],
        "release-not-run",
    );
    assert!(
        e["message"].as_str().unwrap().contains("did not count"),
        "{e}"
    );
    // An unrelated record decides nothing (a sibling that cites only that record), and the fix
    // names the alloc-test that would.
    r.release_counts("H10", json!([0, 2, 5]));
    let g = r.growth.clone();
    let other = r.add_release(&g);
    r.ok(&["ledger", "test", &other, "--evidence", &w10]);
    let e = r.refused(
        &["ledger", "confirm", &other, "--evidence", &g],
        "release-not-run",
    );
    assert!(
        e["message"].as_str().unwrap().contains("not an alloc-test"),
        "{e}"
    );
    assert_eq!(
        e["fix"],
        format!("dla ledger confirm {other} --evidence {flat}")
    );
    // An alloc-test of the project's test loop is no driver made for the finding.
    let mut loop_m: HarnessManifest = serde_json::from_value(json!({
        "id": "H12", "runtime": "python", "mode": "driver", "tools": ["pytest-loop"],
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    loop_m.targets = vec![];
    loop_m.save(&r.run_dir).unwrap();
    let on_loop = r.alloc("H12", "non-leak");
    let e = r.refused(
        &["ledger", "confirm", &other, "--evidence", &on_loop],
        "release-not-run",
    );
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("not a driver made for"),
        "{e}"
    );
    // With the line counted the same move passes.
    r.ok(&["ledger", "confirm", &sib, "--evidence", &flat]);
}

#[test]
fn a_runtime_that_cannot_count_the_line_is_accepted_with_a_note_the_report_prints() {
    let (r, flat, _, w10) = disagreeing("jvm");
    let sib = r.add_release(&flat);
    r.ok(&["ledger", "test", &sib, "--evidence", &w10]);
    r.ok(&["ledger", "refute", "F1", "--evidence", &flat]);
    let v = r.ok(&["ledger", "confirm", &sib, "--evidence", &flat]);
    assert_eq!(v["note"], "remover run not shown", "{v}");
    let list = r.text(&["ledger", "list"]);
    assert!(list.contains("remover run not shown"), "{list}");
    assert!(
        r.report().contains("remover run not shown"),
        "{}",
        r.report()
    );
}

#[test]
fn a_python_run_whose_agent_could_not_count_lines_is_noted_and_one_with_no_record_is_refused() {
    let (r, flat, _, w10) = disagreeing("python");
    let sib = r.add_release(&flat);
    r.ok(&["ledger", "test", &sib, "--evidence", &w10]);
    r.ok(&["ledger", "refute", "F1", "--evidence", &flat]);
    // No lines file at all: nothing shows the release ran.
    r.refused(
        &["ledger", "confirm", &sib, "--evidence", &flat],
        "release-not-run",
    );
    // The file `dla harness run` writes when the records of the run carry the agent's note that
    // it could not count lines (another tool already traces the process).
    LineRuns {
        dla_line_runs: 1,
        series: "S1".into(),
        sites: Default::default(),
        unavailable: Some(
            "line counts unavailable: another tool already traces this process (sys.settrace)"
                .into(),
        ),
    }
    .save(&r.run_dir, "H10")
    .unwrap();
    let v = r.ok(&["ledger", "confirm", &sib, "--evidence", &flat]);
    assert_eq!(v["note"], "remover run not shown", "{v}");
}

#[test]
fn a_rerun_of_the_flat_driver_that_now_favors_leak_refutes_the_sibling_and_the_confirm_passes() {
    let (r, flat, _, w10) = disagreeing("python");
    let sib = r.add_release(&flat);
    r.ok(&["ledger", "test", &sib, "--evidence", &w10]);
    r.release_counts("H10", json!([0, 3, 8]));
    // H10 again with a longer --extra: it now favors leak, with stop() called on every unit.
    let rerun = r.alloc("H10", "leak");
    r.ok(&["ledger", "refute", &sib, "--evidence", &rerun]);
    r.ok(&["ledger", "refute", "F1.n1", "--evidence", &rerun]);
    let g = r.growth.clone();
    r.ok(&["ledger", "confirm", "F1", "--evidence", &g]);
}

#[test]
fn a_third_driver_and_the_sibling_it_refutes_settle_the_disagreement() {
    let (r, flat, leak, w10) = disagreeing("python");
    let sib = r.add_release(&flat);
    r.ok(&["ledger", "test", &sib, "--evidence", &w10]);
    // Held until the sibling is refuted by an alloc-test newer than both.
    let g = r.growth.clone();
    r.ok(&["ledger", "refute", "F1.n1", "--evidence", &leak]);
    r.refused(
        &["ledger", "confirm", "F1", "--evidence", &g],
        "drivers-disagree",
    );
    r.driver("H12");
    r.release_counts("H12", json!([0, 6, 12]));
    let third = r.alloc("H12", "leak");
    r.ok(&["ledger", "refute", &sib, "--evidence", &third]);
    r.ok(&["ledger", "confirm", "F1", "--evidence", &g]);
}

/// Records that F1 is explained with `removal`, by the line `lib/enc.py:30`.
fn explain_removed(r: &R, removal: &str) {
    r.ok(&[
        "ledger",
        "annotate",
        "F1",
        "--writer",
        "lib/enc.py:9",
        "--remover",
        "lib/enc.py:30",
    ]);
    r.ok(&[
        "ledger",
        "explain",
        "F1",
        "--tag",
        "inferred",
        "--adds",
        "lib/enc.py:9",
        "--removal",
        removal,
        "--remover",
        "lib/enc.py:30",
        "--text",
        "An encoder adds its thread-local holder at lib/enc.py:5 through lib/enc.py:9, and lib/enc.py:30 releases it only when stop() is called.",
    ]);
}

#[test]
fn an_explanation_that_the_remover_runs_on_release_holds_the_confirm_until_the_release_ran() {
    let (r, _) = scene("python");
    let leak = r.alloc("H10", "leak");
    r.ok(&["ledger", "refute", "F1.n1", "--evidence", &leak]);
    explain_removed(&r, "on-release");
    let g = r.growth.clone();
    let e = r.refused(
        &["ledger", "confirm", "F1", "--evidence", &g],
        "release-untested",
    );
    let fix = e["fix"].as_str().unwrap();
    assert!(
        fix.contains("--hypothesis N-RELEASE --remover lib/enc.py:30"),
        "{fix}"
    );
    assert!(
        e["message"].as_str().unwrap().contains("lib/enc.py:30"),
        "{e}"
    );
    // The sibling refuted by an alloc-test with the release called lifts it.
    let sib = r.add_release(&leak);
    let w = r.evidence(
        EvidenceKind::Workload,
        json!({"harness": "H10", "mode": "driver", "runtime": "python", "series": ["S1"]}),
    );
    r.ok(&["ledger", "test", &sib, "--evidence", &w]);
    r.release_counts("H10", json!([0, 5, 9]));
    r.ok(&["ledger", "refute", &sib, "--evidence", &leak]);
    r.ok(&["ledger", "confirm", "F1", "--evidence", &g]);
    assert!(
        !r.final_gate()
            .iter()
            .any(|m| m["kind"] == "release-untested"),
        "the release was tested"
    );
}

#[test]
fn after_the_confirm_the_final_gate_lists_the_untested_release_and_the_report_counts_it_apart() {
    let (r, _) = scene("python");
    let leak = r.alloc("H10", "leak");
    r.ok(&["ledger", "refute", "F1.n1", "--evidence", &leak]);
    let g = r.growth.clone();
    r.ok(&["ledger", "confirm", "F1", "--evidence", &g]);
    explain_removed(&r, "on-release");
    let item = r
        .final_gate()
        .into_iter()
        .find(|m| m["kind"] == "release-untested")
        .expect("the gate lists the item");
    assert_eq!(item["id"], "F1");
    let fix = item["fix"].as_str().unwrap();
    assert!(
        fix.contains("--hypothesis N-RELEASE --remover lib/enc.py:30"),
        "{fix}"
    );
    let md = r.report();
    assert!(md.contains("leak hypotheses: confirmed 1"), "{md}");
    // The flat driver made for the finding, with stop() called on every unit: the growth exists
    // only while callers skip it.
    r.driver("H11");
    let flat = r.alloc("H11", "non-leak");
    let sib = r.add_release(&flat);
    let w = r.evidence(
        EvidenceKind::Workload,
        json!({"harness": "H11", "mode": "driver", "runtime": "python", "series": ["S1"]}),
    );
    r.ok(&["ledger", "test", &sib, "--evidence", &w]);
    r.release_counts("H11", json!([0, 7, 15]));
    r.ok(&["ledger", "confirm", &sib, "--evidence", &flat]);
    assert!(
        !r.final_gate()
            .iter()
            .any(|m| m["kind"] == "release-untested"),
        "the release was tested: {:?}",
        r.final_gate()
    );
    let md = r.report();
    assert!(
        md.contains("confirmed 0, grows only while callers skip the release: 1"),
        "{md}"
    );
    assert!(md.contains("; no leak confirmed"), "{md}");
    assert!(
        md.contains("Grows only while callers skip the release: F1.n2 confirmed"),
        "{md}"
    );
}

#[test]
fn a_conditional_remover_neither_holds_the_confirm_nor_adds_a_gate_item() {
    let (r, _) = scene("python");
    let leak = r.alloc("H10", "leak");
    r.ok(&["ledger", "refute", "F1.n1", "--evidence", &leak]);
    explain_removed(&r, "conditional");
    let g = r.growth.clone();
    r.ok(&["ledger", "confirm", "F1", "--evidence", &g]);
    assert!(
        !r.final_gate()
            .iter()
            .any(|m| m["kind"] == "release-untested"),
        "{:?}",
        r.final_gate()
    );
}

#[test]
fn a_sibling_that_names_another_release_does_not_lift_the_hold_of_an_explanation_that_relies_on_one()
 {
    let (r, _) = scene("python");
    let leak = r.alloc("H10", "leak");
    r.ok(&["ledger", "refute", "F1.n1", "--evidence", &leak]);
    explain_removed(&r, "on-release");
    let g = r.growth.clone();
    r.refused(
        &["ledger", "confirm", "F1", "--evidence", &g],
        "release-untested",
    );
    let w = r.evidence(
        EvidenceKind::Workload,
        json!({"harness": "H10", "mode": "driver", "runtime": "python", "series": ["S1"]}),
    );
    // A release the run did call (lib/enc.py:35), refuted where it ran: the explanation relies on
    // lib/enc.py:30, which no run called.
    let other = r.add_release_at(&leak, "lib/enc.py:35");
    r.ok(&["ledger", "test", &other, "--evidence", &w]);
    r.release_counts_at("H10", json!({"lib/enc.py:35": [0, 5, 9]}));
    r.ok(&["ledger", "refute", &other, "--evidence", &leak]);
    let e = r.refused(
        &["ledger", "confirm", "F1", "--evidence", &g],
        "release-untested",
    );
    let fix = e["fix"].as_str().unwrap();
    assert!(
        fix.contains("--hypothesis N-RELEASE --remover lib/enc.py:30"),
        "{fix}"
    );
    assert!(
        r.final_gate()
            .iter()
            .all(|m| m["kind"] != "release-untested" || m["id"] != "F1"),
        "F1 is not confirmed yet, the gate lists confirmed leaks only"
    );
    // The sibling that names the line the explanation relies on, refuted where it ran, lifts it.
    let same = r.add_release_at(&leak, "lib/enc.py:30");
    r.ok(&["ledger", "test", &same, "--evidence", &w]);
    r.release_counts_at(
        "H10",
        json!({"lib/enc.py:35": [0, 5, 9], "lib/enc.py:30": [0, 2, 4]}),
    );
    r.ok(&["ledger", "refute", &same, "--evidence", &leak]);
    r.ok(&["ledger", "confirm", "F1", "--evidence", &g]);
}

/// F1 is confirmed and explained as released by `lib/enc.py:30` only when the caller calls it;
/// a sibling of F1.n1 on `H11` names `lib/enc.py:35` instead.
fn confirmed_with_an_on_release_explanation(r: &R) -> String {
    let leak = r.alloc("H10", "leak");
    r.ok(&["ledger", "refute", "F1.n1", "--evidence", &leak]);
    let g = r.growth.clone();
    r.ok(&["ledger", "confirm", "F1", "--evidence", &g]);
    explain_removed(r, "on-release");
    assert!(
        r.final_gate()
            .iter()
            .any(|m| m["kind"] == "release-untested"),
        "the release was not tested"
    );
    r.driver("H11");
    leak
}

fn workload_of(r: &R, harness: &str) -> String {
    r.evidence(
        EvidenceKind::Workload,
        json!({"harness": harness, "mode": "driver", "runtime": "python", "series": ["S1"]}),
    )
}

#[test]
fn a_confirmed_sibling_naming_another_release_clears_the_hold_and_the_report_counts_the_finding_apart()
 {
    let (r, _) = scene("python");
    confirmed_with_an_on_release_explanation(&r);
    let w = workload_of(&r, "H11");
    let flat = r.alloc("H11", "non-leak");
    let other = r.add_release_at(&flat, "lib/enc.py:35");
    r.ok(&["ledger", "test", &other, "--evidence", &w]);
    r.release_counts_at("H11", json!({"lib/enc.py:35": [0, 7, 15]}));
    r.ok(&["ledger", "confirm", &other, "--evidence", &flat]);
    // The gate and the report agree: the finding grows only while callers skip a release.
    assert!(
        !r.final_gate()
            .iter()
            .any(|m| m["kind"] == "release-untested"),
        "{:?}",
        r.final_gate()
    );
    let md = r.report();
    assert!(
        md.contains("confirmed 0, grows only while callers skip the release: 1"),
        "{md}"
    );
}

#[test]
fn a_refuted_sibling_naming_another_release_keeps_the_hold_and_the_finding_stays_a_leak() {
    let (r, _) = scene("python");
    let leak = confirmed_with_an_on_release_explanation(&r);
    let w = workload_of(&r, "H10");
    let other = r.add_release_at(&leak, "lib/enc.py:35");
    r.ok(&["ledger", "test", &other, "--evidence", &w]);
    r.release_counts_at("H10", json!({"lib/enc.py:35": [0, 5, 9]}));
    r.ok(&["ledger", "refute", &other, "--evidence", &leak]);
    assert!(
        r.final_gate()
            .iter()
            .any(|m| m["kind"] == "release-untested" && m["id"] == "F1"),
        "{:?}",
        r.final_gate()
    );
    let md = r.report();
    assert!(md.contains("leak hypotheses: confirmed 1"), "{md}");
}

#[test]
fn a_runtime_that_counts_lines_is_refused_without_a_coverage_record_and_accepted_with_one() {
    for runtime in ["rust", "go", "node"] {
        let (r, flat, _, w10) = disagreeing(runtime);
        let sib = r.add_release(&flat);
        r.ok(&["ledger", "test", &sib, "--evidence", &w10]);
        r.ok(&["ledger", "refute", "F1", "--evidence", &flat]);
        // No `executed.json`: nothing shows the release ran, so the decision is refused and the
        // fix names the command that writes the record.
        let e = r.refused(
            &["ledger", "confirm", &sib, "--evidence", &flat],
            "release-not-run",
        );
        assert!(
            e["message"]
                .as_str()
                .unwrap()
                .contains("no coverage record"),
            "{runtime}: {e}"
        );
        let fix = e["fix"].as_str().unwrap();
        assert!(
            fix.starts_with("dla alloc-test H10 --finding F1"),
            "{runtime}: {fix}"
        );
        let executed = r.run_dir.join("harness/H10/executed.json");
        let write = |count: u64| {
            fs::create_dir_all(executed.parent().unwrap()).unwrap();
            fs::write(
                &executed,
                json!({"runtime": runtime, "kind": "functions",
                    "lines": [{"file": "lib/enc.py", "line": 30, "count": count}]})
                .to_string(),
            )
            .unwrap();
        };
        // A record that counted the line 0 times is refused too.
        write(0);
        r.refused(
            &["ledger", "confirm", &sib, "--evidence", &flat],
            "release-not-run",
        );
        write(6);
        let v = r.ok(&["ledger", "confirm", &sib, "--evidence", &flat]);
        assert!(
            v.get("note").is_none(),
            "{runtime}: the line was counted: {v}"
        );
    }
}

/// A driver `harness` that targets the worker's finding `w1/F1`, and its workload evidence (main).
fn worker_driver(r: &R, harness: &str) -> String {
    let m: HarnessManifest = serde_json::from_value(json!({
        "id": harness, "runtime": "python", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": ["w1/F1"], "agent": "driver-python", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&r.run_dir).unwrap();
    r.evidence(
        EvidenceKind::Workload,
        json!({"harness": harness, "mode": "driver", "runtime": "python", "series": ["S1"]}),
    )
}

#[test]
fn a_worker_settles_two_disagreeing_drivers_of_the_main_ledger_with_a_sibling_of_its_own() {
    let (r, _) = scene("python");
    let g = r.growth.clone();
    // The main ledger holds the two drivers that disagree, on harnesses made for the worker's
    // finding: H20 favors leak (older), H21 favors non-leak.
    let w20 = worker_driver(&r, "H20");
    let w21 = worker_driver(&r, "H21");
    worker_driver(&r, "H22");
    let alloc = |h: &str, favors: &str| {
        r.evidence(
            EvidenceKind::Discriminator,
            json!({"test": "plateau+purge", "favors": favors, "supports": [],
                "finding": "w1/F1", "harness": h, "series": "S1"}),
        )
    };
    let _leak = alloc("H20", "leak");
    let _flat = alloc("H21", "non-leak");
    let as_worker = |args: &[&str]| {
        let mut v = vec!["--worker", "w1"];
        v.extend_from_slice(args);
        r.run(&v)
    };
    let ok = |args: &[&str]| {
        let (code, v) = as_worker(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    };
    let v = ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &g,
        "--site",
        "lib/enc.py:5",
        "--holder",
        "thread-local",
        "--owner",
        "Encoder.holder",
    ]);
    assert_eq!(v["id"], "w1/F1", "{v}");
    // A sibling of the worker's own, added after both drivers.
    let v = ok(&[
        "ledger",
        "add",
        "--sibling-of",
        "F1",
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM",
        "--refute-if",
        "growth continues over the extra cycles",
        "--evidence",
        &g,
    ]);
    let sib = v["id"].as_str().unwrap().to_owned();
    assert_eq!(sib, "w1/F1.n1", "{v}");
    ok(&["ledger", "test", "F1", "--evidence", &w20]);
    ok(&["ledger", "test", &sib, "--evidence", &w21]);
    // Held: the drivers disagree and the sibling is not refuted yet.
    let (code, v) = as_worker(&["ledger", "confirm", "F1", "--evidence", &g]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "drivers-disagree", "{v}");
    // The printed route, followed by the worker: the sibling is refuted by its own alloc-test on
    // a third driver made for the finding that favors leak.
    let mut slice = Ledger::load_for(&r.run_dir, Some("w1")).unwrap();
    slice
        .add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: Authority::Test,
            kind: EvidenceKind::Discriminator,
            result: json!({"test": "plateau+purge", "favors": "leak", "supports": [],
                "finding": "w1/F1", "harness": "H22", "series": "S1"}),
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
    let own = slice
        .evidence_in_order()
        .last()
        .unwrap()
        .evidence_id
        .to_string();
    drop(slice);
    assert_eq!(own, "w1/E1");
    ok(&["ledger", "refute", &sib, "--evidence", &own]);
    // The main drivers are older than the worker's sibling and its alloc-test, whatever their
    // lines are called: the confirm passes.
    ok(&["ledger", "confirm", "F1", "--evidence", &g]);
}
