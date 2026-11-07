#![allow(dead_code)]

use dla_core::{
    Authority, EvidenceId, EvidenceKind, FindingId, Ledger, NewEvidence, NewFinding, Origin,
    RunState, Source, Tool, evidence::EvidenceCoverage, state::now_utc,
};
use serde_json::json;

pub fn evidence(kind: EvidenceKind, authority: Authority) -> NewEvidence {
    NewEvidence {
        source: Source::Repo {
            commit: "abc123".into(),
            path: None,
        },
        tool: Tool {
            name: "test".into(),
            version: "0".into(),
        },
        authority,
        kind,
        result: json!({"n": 1}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    }
}

pub fn eid(n: u32) -> EvidenceId {
    EvidenceId::new(n)
}

pub fn fid(n: u32) -> FindingId {
    FindingId::new(n)
}

pub fn origin(s: &str) -> Origin {
    Origin::parse(s).unwrap()
}

/// A memory-only ledger holding one static and one discriminator evidence record.
pub fn ledger() -> Ledger {
    let mut l = Ledger::from_text("").unwrap();
    l.add_evidence(evidence(EvidenceKind::StaticHit, Authority::Static))
        .unwrap();
    l.add_evidence(evidence(EvidenceKind::Discriminator, Authority::Test))
        .unwrap();
    l.add_evidence(evidence(EvidenceKind::GrowthSeries, Authority::Live))
        .unwrap();
    l
}

/// Adds F1 (supported by E1) and returns its ID.
pub fn add_f1(l: &mut Ledger) -> FindingId {
    l.add_finding(NewFinding::new(origin("census"), vec![eid(1)]))
        .unwrap();
    fid(1)
}

pub fn run_state() -> RunState {
    RunState::init("/tmp/project", Some("s1"), 3600, None, now_utc()).unwrap()
}

/// Hand-written ledger text with the given extra lines after one evidence line.
pub fn line(seq: u64, body: &str) -> String {
    format!("{{\"schema_version\":1,\"seq\":{seq},\"ts\":\"2026-01-01T00:00:00Z\",{body}}}\n")
}

pub const EVIDENCE_BODY: &str = r#""type":"evidence","evidence_id":"E1","source":{"repo":{"commit":"abc"}},"tool":{"name":"t","version":"1"},"authority":"static","kind":"static-hit","result":{}"#;
