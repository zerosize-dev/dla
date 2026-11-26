//! An abort does not skip the verifier: the confirmed findings with no verification record are
//! listed by `dla run abort`, can still be bundled and verified afterwards (and count in the
//! report), the Stop hook of the aborted run holds the stop while one is unverified, and the
//! report says per finding that it was not verified.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, FindingState, Ledger, NewEvidence, NewFinding, Origin, Source, Tool,
    evidence::{EvidenceCoverage, Signal},
    finding::Holder,
};
use serde_json::{Value, json};

struct Run {
    dir: tempfile::TempDir,
    run_id: String,
}

fn base(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_ARCHETYPES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_PROJECT_DIR",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
    ] {
        c.env_remove(var);
    }
    c.arg("--project").arg(project);
    c
}

impl Run {
    fn path(&self) -> &Path {
        self.dir.path()
    }
    fn run_dir(&self) -> std::path::PathBuf {
        self.path().join(".dla").join(&self.run_id)
    }
    fn json(&self, args: &[&str]) -> (i32, Value) {
        let o = base(self.path()).args(args).output().unwrap();
        (
            o.status.code().unwrap(),
            serde_json::from_slice(&o.stdout).unwrap_or(Value::Null),
        )
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.json(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }
    fn state(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir().join("state.json")).unwrap())
            .unwrap()
    }
    fn report(&self) -> String {
        String::from_utf8(base(self.path()).arg("report").output().unwrap().stdout).unwrap()
    }
    /// The Stop hook, as Claude Code runs it: no `--project`, the run found from `cwd`.
    fn stop(&self) -> (Value, String) {
        let away = tempfile::tempdir().unwrap();
        let mut c = Command::cargo_bin("dla").unwrap();
        c.env_remove("CLAUDE_PROJECT_DIR").env_remove("DLA_RUN");
        let o = c
            .current_dir(away.path())
            .args(["gate", "stop-hook"])
            .write_stdin(json!({"cwd": self.path(), "hook_event_name": "Stop"}).to_string())
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(0));
        (
            serde_json::from_slice(&o.stdout).unwrap_or(Value::Null),
            String::from_utf8(o.stderr).unwrap(),
        )
    }
    /// `dla bundle F` and `dla ledger verify F` with the answer that agrees with the finding.
    fn verify(&self, f: &str, site: &str) -> Value {
        let b = self.ok(&["bundle", f]);
        let sha = b["manifest_sha256"].as_str().unwrap().to_owned();
        self.ok(&[
            "ledger",
            "verify",
            f,
            "--site",
            site,
            "--holder",
            "static/global",
            "--mechanism",
            "the holder keeps what it is given",
            "--bundle",
            &sha,
        ])
    }
}

fn evidence(
    kind: EvidenceKind,
    authority: Authority,
    tool: &str,
    class: &str,
    src: &str,
    mut result: Value,
) -> NewEvidence {
    result["signal"] = Signal::new(tool, class, src).to_value();
    NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: tool.into(),
            version: "0".into(),
        },
        authority,
        kind,
        result,
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    }
}

/// Adds, through the ledger API, `finding` (F<n>) confirmed at `site` with its refuted N-WARM
/// sibling.
fn confirm(l: &mut Ledger, n: u32, site: &str) {
    let mut ev = |kind, authority, tool: &str, class: &str, src: &str, result: Value| {
        l.add_evidence(evidence(kind, authority, tool, class, src, result))
            .unwrap();
    };
    ev(
        EvidenceKind::StaticHit,
        Authority::Static,
        "t",
        "x",
        &format!("s1-{n}"),
        json!({}),
    );
    ev(
        EvidenceKind::Workload,
        Authority::Test,
        "t",
        "x",
        &format!("s2-{n}"),
        json!({"harness": format!("H{n}"), "mode": "command", "runtime": "go"}),
    );
    ev(
        EvidenceKind::Discriminator,
        Authority::Test,
        "dla",
        "x",
        &format!("s3-{n}"),
        json!({"test": "plateau", "favors": "leak", "supports": ["leak"], "finding": null}),
    );
    ev(
        EvidenceKind::GrowthSeries,
        Authority::Test,
        "dla-trend",
        "harness-series",
        &format!("aa{n}"),
        json!({"trend": format!("T{n}"), "series": format!("S{n}"), "n": 8, "gate": "pass", "failed": 0,
        "passed": [{"column": "probe:CACHE", "region": "heap", "site": site, "finding": format!("F{n}"), "shape": "steady"}]}),
    );
    let pick = |l: &Ledger, kind: EvidenceKind, nth: usize| {
        l.evidence()
            .filter(|x| x.kind == kind)
            .nth(nth)
            .unwrap()
            .evidence_id
            .clone()
    };
    let idx = (n - 1) as usize;
    let first = pick(l, EvidenceKind::StaticHit, idx);
    let harness = pick(l, EvidenceKind::Workload, idx);
    let disc = pick(l, EvidenceKind::Discriminator, idx);
    let growth = pick(l, EvidenceKind::GrowthSeries, idx);
    let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![first.clone()]);
    nf.site = Some(site.into());
    nf.holder = Some(Holder::StaticGlobal);
    l.add_finding(nf).unwrap();
    let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![first]);
    nf.sibling_of = Some(format!("F{n}").parse().unwrap());
    nf.hypothesis = Some("N-WARM".into());
    nf.refute_if = Some("growth continues".into());
    l.add_finding(nf).unwrap();
    let id = |s: String| s.parse().unwrap();
    l.transition(
        &id(format!("F{n}.n1")),
        FindingState::Refuted,
        None,
        None,
        vec![disc],
        "t",
    )
    .unwrap();
    l.transition(
        &id(format!("F{n}")),
        FindingState::UnderTest,
        None,
        None,
        vec![harness],
        "t",
    )
    .unwrap();
    l.transition(
        &id(format!("F{n}")),
        FindingState::Confirmed,
        None,
        None,
        vec![growth],
        "t",
    )
    .unwrap();
}

/// A symptom run with two confirmed findings, F1 and F2.
fn two_confirmed() -> Run {
    let dir = tempfile::tempdir().unwrap();
    let o = base(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2")
        .args(["run", "init", "--session-id", "t", "--entry", "symptom"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let r = Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    };
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    confirm(&mut l, 1, "src/A.java:40");
    confirm(&mut l, 2, "src/B.java:12");
    drop(l);
    r
}

const NOT_VERIFIED: &str = "not verified: the run was aborted before the verifier ran";

#[test]
fn an_abort_lists_the_confirmed_findings_the_verifier_has_not_read() {
    let r = two_confirmed();
    r.verify("F1", "src/A.java:40");
    let (code, v) = r.json(&["run", "abort", "--blocker", "user"]);
    assert_eq!(code, 0, "{v}");
    // Only the finding without a record is listed, with the commands for it.
    let list = v["unverified"].as_array().unwrap();
    assert_eq!(list.len(), 1, "{v}");
    assert_eq!(list[0]["finding"], "F2");
    let steps = list[0]["verify"].as_str().unwrap();
    assert!(
        steps.starts_with("dla bundle F2, then one dla:verifier subagent")
            && steps.contains("dla ledger verify F2 --site"),
        "{steps}"
    );
    assert!(
        v["next"]
            .as_str()
            .unwrap()
            .contains("1 confirmed finding(s)"),
        "{v}"
    );
    // The text says what still counts after an abort.
    assert!(
        v["message"]
            .as_str()
            .unwrap()
            .contains("except a verifier's record"),
        "{v}"
    );
    // With both verified there is nothing to list.
    let q = two_confirmed();
    q.verify("F1", "src/A.java:40");
    q.verify("F2", "src/B.java:12");
    let (code, v) = q.json(&["run", "abort", "--blocker", "user"]);
    assert_eq!(code, 0, "{v}");
    assert!(v.get("unverified").is_none(), "{v}");
}

#[test]
fn a_verification_after_the_abort_is_accepted_and_counts_in_the_report() {
    let r = two_confirmed();
    r.verify("F1", "src/A.java:40");
    r.ok(&["run", "abort", "--blocker", "user"]);
    assert_eq!(r.state()["status"], "aborted");
    let md = r.report();
    assert!(md.contains("- Verified: agrees"), "{md}");
    // F2 is marked, per finding and on the Findings line, apart from the confirmed ones.
    assert!(
        md.contains(&format!("- Verified: no; {NOT_VERIFIED}")),
        "{md}"
    );
    assert!(
        md.contains("confirmed but not verified (the run was aborted before the verifier ran) 1"),
        "{md}"
    );
    // The verifier of F2 runs after the abort: accepted, and the report counts it.
    let v = r.verify("F2", "src/B.java:12");
    assert_eq!(v["verified"], true, "{v}");
    assert_eq!(r.state()["status"], "aborted");
    let md = r.report();
    assert!(!md.contains(NOT_VERIFIED), "{md}");
    assert!(!md.contains("confirmed but not verified"), "{md}");
    assert_eq!(md.matches("- Verified: agrees").count(), 2, "{md}");
}

#[test]
fn the_stop_hook_of_an_aborted_run_holds_the_stop_while_a_finding_is_unverified() {
    let r = two_confirmed();
    r.verify("F1", "src/A.java:40");
    r.ok(&["run", "abort", "--blocker", "user"]);
    let (v, _) = r.stop();
    assert_eq!(v["decision"], "block", "{v}");
    let reason = v["reason"].as_str().unwrap();
    assert!(reason.len() <= 1000, "{} bytes", reason.len());
    assert!(
        reason.contains("1 confirmed finding(s) have no verification record: F2")
            && reason.contains("An abort does not skip the verifier")
            && reason.contains("dla bundle F2")
            && reason.contains("dla ledger verify F"),
        "{reason}"
    );
    // The run stays aborted: the hold is not a stamp.
    assert_eq!(r.state()["status"], "aborted");
    // Verified (after the abort), the stop passes at once.
    r.verify("F2", "src/B.java:12");
    let (v, err) = r.stop();
    assert!(v.is_null() && err.is_empty(), "{v} {err}");
}

#[test]
fn the_hold_of_an_aborted_run_ends_after_the_stop_limit_without_a_stamp() {
    let r = two_confirmed();
    r.ok(&["run", "abort", "--blocker", "user"]);
    for n in 1..=2 {
        let (v, _) = r.stop();
        assert_eq!(v["decision"], "block", "attempt {n}: {v}");
        assert!(
            v["reason"]
                .as_str()
                .unwrap()
                .contains(&format!("Stop attempt {n} of 3")),
            "{v}"
        );
    }
    let (v, err) = r.stop();
    assert!(v.is_null(), "{v}");
    assert!(err.contains("the stop is allowed"), "{err}");
    assert_eq!(r.state()["status"], "aborted", "an abort is not restamped");
}

#[test]
fn the_fix_of_a_refused_blocker_is_printed_as_the_rule_returns_it() {
    let r = two_confirmed();
    for (args, fix) in [
        (
            vec!["run", "abort", "--blocker", "tool", "--code", "time-left"],
            "dla run abort --blocker tool --reason \"<what blocks the run>\"",
        ),
        (
            vec![
                "run",
                "abort",
                "--blocker",
                "refusal",
                "--reason",
                "the capture needs a privilege the run does not have",
            ],
            "dla run abort --blocker refusal --code <refusal code> --reason \"<what blocks the run: the refusal the run cannot satisfy>\"",
        ),
        (
            vec!["run", "abort", "--blocker", "sleepy"],
            "dla run abort --blocker tool|build|environment|privilege --reason \"<what blocks the run>\"",
        ),
    ] {
        let (code, v) = r.json(&args);
        assert_eq!(code, 2, "{args:?}: {v}");
        assert_eq!(v["error"]["fix"], fix, "{args:?}: {v}");
        // One `--blocker`, nothing repeated or stray in front of it.
        assert_eq!(
            v["error"]["fix"]
                .as_str()
                .unwrap()
                .matches("--blocker")
                .count(),
            1,
            "{v}"
        );
    }
    // Nothing was aborted by the refusals.
    assert_eq!(r.state()["status"], "active");
}
