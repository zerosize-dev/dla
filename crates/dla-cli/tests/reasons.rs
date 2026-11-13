//! Honest closing reasons through the CLI: `time_budget` is refused while the budget has
//! room (`time-left`) unless a tool refused a command for time, and `inconclusive` needs a
//! measurement tied to the finding (`measurement-required`).

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, Ledger, NewEvidence, NewFinding, Origin, Source, Tool,
    evidence::EvidenceCoverage,
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

fn rfc3339_in(secs: i64) -> String {
    (chrono::Utc::now() + chrono::TimeDelta::seconds(secs))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn evidence(kind: EvidenceKind, authority: Authority, result: Value) -> NewEvidence {
    NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
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

impl Run {
    fn run_dir(&self) -> std::path::PathBuf {
        self.dir.path().join(".dla").join(&self.run_id)
    }
    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = base(self.dir.path()).args(args).output().unwrap();
        let v = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
        (o.status.code().unwrap(), v)
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }
    /// Closes `target` unresolved with `reason`; the refusal code, if any.
    fn close(&self, worker: Option<&str>, target: &str, reason: &str) -> Option<String> {
        let mut args = vec![];
        if let Some(w) = worker {
            args.extend(["--worker", w]);
        }
        args.extend([
            "ledger",
            "resolve",
            target,
            "--as",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
        ]);
        let (code, v) = self.run(&args);
        if code == 0 {
            None
        } else {
            Some(v["error"]["code"].as_str().unwrap().to_owned())
        }
    }
    fn set_state(&self, f: impl FnOnce(&mut Value)) {
        let path = self.run_dir().join("state.json");
        let mut st: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        f(&mut st);
        fs::write(path, st.to_string()).unwrap();
    }
    /// What a tool leaves when it refuses a command for time.
    fn note_time_refusal(&self, worker: Option<&str>) {
        use std::io::Write;
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.run_dir().join(dla_core::toolctx::TIME_REFUSALS_FILE))
            .unwrap();
        let line = json!({"ts": rfc3339_in(1), "worker": worker});
        writeln!(f, "{line}").unwrap();
    }
}

/// A fresh run (60 minute budget) with a static hit E1 and a candidate F1 on it.
fn scene() -> Run {
    let dir = tempfile::tempdir().unwrap();
    let o = base(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2")
        .args(["run", "init", "--session-id", "t", "--entry", "repository"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let r = Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    };
    let mut l = Ledger::load_for(&r.run_dir(), None).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::StaticHit,
        Authority::Static,
        json!({}),
    ))
    .unwrap();
    let e = l.evidence().last().unwrap().evidence_id.clone();
    let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![e]);
    nf.site = Some("a/B.kt:1".into());
    l.add_finding(nf).unwrap();
    r
}

#[test]
fn time_budget_is_refused_while_the_budget_has_room() {
    let r = scene();
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "time_budget",
        "--remedy",
        "x",
    ]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "time-left");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("of 60 budget minutes remain"), "{msg}");
    // The fix names the reason the evidence supports: nothing was measured for F1.
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(fix.contains("--reason not_exercised"), "{fix}");
    // Nothing was written.
    let state = Ledger::load_for(&r.run_dir(), None)
        .unwrap()
        .finding(&dla_core::FindingId::new(1))
        .unwrap()
        .state;
    assert_eq!(state, dla_core::FindingState::Candidate);
    // The reason that is true is accepted.
    assert_eq!(r.close(None, "F1", "not_exercised"), None);
}

#[test]
fn the_ledgers_own_checks_come_first() {
    let r = scene();
    // A missing remedy and an unknown finding are reported as such, not as time-left.
    let (_, v) = r.run(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "time_budget",
    ]);
    assert_eq!(v["error"]["code"], "remedy-required", "{v}");
    assert_eq!(
        r.close(None, "F9", "time_budget").as_deref(),
        Some("unknown-finding")
    );
    // An unknown evidence id too.
    let (_, v) = r.run(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "time_budget",
        "--remedy",
        "x",
        "--evidence",
        "E99",
    ]);
    assert_eq!(v["error"]["code"], "unknown-evidence", "{v}");
}

#[test]
fn time_budget_is_accepted_when_the_deadline_is_near() {
    // The floor of a 60 minute budget is 12 minutes.
    let r = scene();
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(11 * 60)));
    assert_eq!(r.close(None, "F1", "time_budget"), None, "11 minutes left");

    let r = scene();
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(13 * 60)));
    assert_eq!(
        r.close(None, "F1", "time_budget").as_deref(),
        Some("time-left"),
        "13 minutes left"
    );

    // A short budget: the floor is 10 minutes, not 20 %.
    let r = scene();
    r.set_state(|st| {
        st["budget"]["seconds"] = json!(1800);
        st["budget"]["deadline"] = json!(rfc3339_in(9 * 60));
    });
    assert_eq!(r.close(None, "F1", "time_budget"), None, "9 of 30 minutes");

    // A spent deadline.
    let r = scene();
    r.set_state(|st| st["budget"]["deadline"] = json!(rfc3339_in(-5)));
    assert_eq!(r.close(None, "F1", "time_budget"), None);
}

#[test]
fn a_command_refused_for_time_after_the_finding_made_it_honest() {
    let r = scene();
    assert_eq!(
        r.close(None, "F1", "time_budget").as_deref(),
        Some("time-left")
    );
    // A tool refused a command for time (it leaves a note), 59 minutes left.
    r.note_time_refusal(None);
    assert_eq!(r.close(None, "F1", "time_budget"), None);
}

#[test]
fn a_refusal_from_before_the_finding_does_not_count() {
    let dir = tempfile::tempdir().unwrap();
    let o = base(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2")
        .args(["run", "init", "--session-id", "t", "--entry", "repository"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let r = Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    };
    // The note is older than the finding.
    let path = r.run_dir().join(dla_core::toolctx::TIME_REFUSALS_FILE);
    fs::write(
        &path,
        format!("{}\n", json!({"ts": rfc3339_in(-600), "worker": null})),
    )
    .unwrap();
    let mut l = Ledger::load_for(&r.run_dir(), None).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::StaticHit,
        Authority::Static,
        json!({}),
    ))
    .unwrap();
    let e = l.evidence().last().unwrap().evidence_id.clone();
    l.add_finding(NewFinding::new(
        Origin::parse("hypothesis").unwrap(),
        vec![e],
    ))
    .unwrap();
    assert_eq!(
        r.close(None, "F1", "time_budget").as_deref(),
        Some("time-left")
    );
}

#[test]
fn an_open_item_is_held_to_the_same_rule() {
    let r = scene();
    let mut l = Ledger::load_for(&r.run_dir(), None).unwrap();
    l.open_item_add("something open", None, "manual").unwrap();
    assert_eq!(
        r.close(None, "O1", "time_budget").as_deref(),
        Some("time-left")
    );
    r.note_time_refusal(None);
    assert_eq!(r.close(None, "O1", "time_budget"), None);
}

#[test]
fn a_workers_own_deadline_and_its_own_refusals_count() {
    let r = scene();
    r.ok(&["run", "dispatch", "--workers", "go"]);
    {
        // The slice lock is held while the ledger is open: drop it before the CLI runs.
        let mut l = Ledger::load_for(&r.run_dir(), Some("go")).unwrap();
        l.add_evidence(evidence(
            EvidenceKind::StaticHit,
            Authority::Static,
            json!({}),
        ))
        .unwrap();
        let e = l.evidence().last().unwrap().evidence_id.clone();
        l.add_finding(NewFinding::new(
            Origin::parse("hypothesis").unwrap(),
            vec![e],
        ))
        .unwrap();
    }
    let w = Some("go");
    assert_eq!(
        r.close(w, "F1", "time_budget").as_deref(),
        Some("time-left")
    );
    // The core's refusal is not the worker's.
    r.note_time_refusal(None);
    assert_eq!(
        r.close(w, "F1", "time_budget").as_deref(),
        Some("time-left")
    );
    // The worker's own is.
    r.note_time_refusal(Some("go"));
    assert_eq!(r.close(w, "F1", "time_budget"), None);
}

#[test]
fn the_time_left_fix_of_a_static_only_finding_stays_static_only() {
    let r = scene();
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "suspected_static_only",
        "--reason",
        "time_budget",
        "--remedy",
        "x",
    ]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "time-left");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains("--as suspected_static_only --reason not_exercised")
            && !fix.contains("--as unresolved"),
        "{fix}"
    );
}

#[test]
fn a_workers_nearer_deadline_makes_time_budget_true_for_its_findings() {
    // The run has 60 minutes left, the worker 5: the worker's own deadline decides.
    let r = scene();
    r.ok(&["run", "dispatch", "--workers", "go"]);
    {
        let mut l = Ledger::load_for(&r.run_dir(), Some("go")).unwrap();
        l.add_evidence(evidence(
            EvidenceKind::StaticHit,
            Authority::Static,
            json!({}),
        ))
        .unwrap();
        let e = l.evidence().last().unwrap().evidence_id.clone();
        l.add_finding(NewFinding::new(
            Origin::parse("hypothesis").unwrap(),
            vec![e],
        ))
        .unwrap();
    }
    let w = Some("go");
    assert_eq!(
        r.close(w, "F1", "time_budget").as_deref(),
        Some("time-left"),
        "both deadlines far"
    );
    r.set_state(|st| {
        let ws = st["workers"].as_array_mut().unwrap();
        ws[0]["deadline"] = json!(rfc3339_in(5 * 60));
    });
    assert_eq!(
        r.close(w, "F1", "time_budget"),
        None,
        "the worker has 5 minutes left, the run 60"
    );
}

#[test]
fn inconclusive_needs_a_measurement_tied_to_the_finding() {
    let r = scene();
    // Only a static hit and no harness: nothing was measured.
    let (code, v) = r.run(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "x",
    ]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "measurement-required");
    assert!(v["error"]["fix"].as_str().unwrap().contains("alloc-test"));

    // A discriminator bound to F1: measured, and it came back inconclusive.
    let mut l = Ledger::load_for(&r.run_dir(), None).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Discriminator,
        Authority::Test,
        json!({"test": "plateau", "favors": "inconclusive", "supports": [], "finding": "F1"}),
    ))
    .unwrap();
    let v = r.ok(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "x",
    ]);
    assert_eq!(v["state"], "unresolved(inconclusive)");
}

#[test]
fn the_unfinished_finding_fix_names_the_reason_and_never_time_budget() {
    let r = scene();
    let v = r.ok(&["run", "status"]);
    let text = v.to_string();
    assert!(
        text.contains("--as unresolved --reason not_exercised --remedy"),
        "{text}"
    );
    assert!(!text.contains("--reason time_budget"), "{text}");
}
