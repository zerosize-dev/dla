//! `dla ledger close-rest` on a few hundred candidates gives what closing each one with
//! `dla ledger resolve` gives: the same state and reason for every finding, in one pass.

use std::{collections::BTreeMap, fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceId, EvidenceKind, Ledger, NewEvidence, NewFinding, Origin, Source, Tool,
    evidence::EvidenceCoverage,
};
use serde_json::{Value, json};

const N: u32 = 200;

fn command(project: &Path) -> Command {
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
    c
}

/// A run of `N` candidates: the even ones rest on a static hit, the odd ones on the run's own
/// first record only.
fn run_with_candidates() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.py"), "X = {}\n").unwrap();
    let o = command(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2,3")
        .args(["run", "init", "--session-id", "t", "--queue", "off"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let mut l = Ledger::load(&run_dir).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Repo {
            commit: "c".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "1".into(),
        },
        authority: Authority::Static,
        kind: EvidenceKind::StaticHit,
        result: json!({"lang": "python"}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
    // The static hit is the newest evidence record.
    let hit = EvidenceId::new(l.lines().len() as u32);
    for i in 1..=N {
        let support = if i % 2 == 0 {
            hit.clone()
        } else {
            EvidenceId::new(1)
        };
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![support]);
        nf.site = Some(format!("pkg/m{i}.py:3"));
        l.add_finding(nf).unwrap();
    }
    (dir, run_dir)
}

fn states(project: &Path) -> BTreeMap<String, String> {
    let o = command(project)
        .args(["ledger", "list", "--all"])
        .output()
        .unwrap();
    String::from_utf8(o.stdout)
        .unwrap()
        .lines()
        .skip(1)
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .map(|v| {
            (
                v["id"].as_str().unwrap().to_owned(),
                v["state"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn closing_hundreds_of_candidates_in_one_pass_equals_closing_them_one_by_one() {
    let (bulk, _) = run_with_candidates();
    let (single, _) = run_with_candidates();

    let start = std::time::Instant::now();
    let o = command(bulk.path())
        .args(["ledger", "close-rest"])
        .output()
        .unwrap();
    let took = start.elapsed();
    let out: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(out["closed"], N, "{out}");
    eprintln!("close-rest of {N} candidates took {took:?}");

    // One by one with `ledger resolve`: the disposition close-rest picks for a finding with
    // static evidence is suspected_static_only, else unresolved, both not_exercised.
    for i in 1..=N {
        let (as_, _) = if i % 2 == 0 {
            ("suspected_static_only", ())
        } else {
            ("unresolved", ())
        };
        let o = command(single.path())
            .args([
                "ledger",
                "resolve",
                &format!("F{i}"),
                "--as",
                as_,
                "--reason",
                "not_exercised",
                "--remedy",
                "x",
            ])
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "F{i}: {}",
            String::from_utf8_lossy(&o.stdout)
        );
    }
    let (a, b) = (states(bulk.path()), states(single.path()));
    assert_eq!(a.len(), N as usize);
    assert_eq!(a, b);
    assert_eq!(a["F2"], "suspected_static_only(not_exercised)");
    assert_eq!(a["F1"], "unresolved(not_exercised)");
}
