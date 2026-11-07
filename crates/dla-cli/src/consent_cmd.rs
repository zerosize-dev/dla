//! `dla consent`: the one consent record of a run.

use std::path::Path;

use clap::Args;
use dla_core::{
    Ledger, Refusal, RunState, RunStatus, consent,
    error::codes as core_codes,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool,
    },
    state::{self, Consent, ConsentGiver},
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    git,
    parse::parse_enum,
};

/// Flags of `dla consent`.
#[derive(Args, Debug)]
pub struct ConsentArgs {
    /// Allowed privilege tiers: 1-4, 1,2,3 or 1-2,4.
    #[arg(long)]
    tiers: String,
    /// Raw values may appear in evidence.
    #[arg(long)]
    raw_values: bool,
    /// Highest tier that may be granted (the plugin's privilege ceiling); tiers above it are refused.
    #[arg(long)]
    ceiling: Option<String>,
    /// Who gave the consent: user (default) or runner.
    #[arg(long, default_value = "user", value_parser = parse_enum::<ConsentGiver>)]
    given_by: ConsentGiver,
}

/// Appends the `run-config` evidence record that describes a consent and returns its
/// ledger line number.
pub fn append_evidence(
    ledger: &mut Ledger,
    project: &Path,
    consent: &Consent,
) -> Result<u64, dla_core::Error> {
    let root_text = project.to_string_lossy().into_owned();
    let commit = git::head_commit(project).unwrap_or(root_text);
    let line = ledger.add_evidence(NewEvidence {
        source: Source::Repo { commit, path: None },
        tool: Tool {
            name: "dla".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        authority: Authority::Static,
        kind: EvidenceKind::RunConfig,
        result: consent::evidence_result(consent),
        raw: None,
        coverage: EvidenceCoverage {
            status: CoverageStatus::Complete,
            examined: None,
            total: None,
        },
        limitations: Vec::new(),
        unknowns: Vec::new(),
    })?;
    Ok(line.seq)
}

fn evidence_id(ledger: &Ledger, seq: u64) -> String {
    ledger
        .lines()
        .iter()
        .find(|l| l.seq == seq)
        .and_then(|l| match &l.record {
            dla_core::Record::Evidence(e) => Some(e.evidence_id.to_string()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Runs `dla consent`.
pub fn run(ctx: &Ctx, project: &Path, a: ConsentArgs) -> Res<Value> {
    let tiers = consent::parse_tiers(&a.tiers).map_err(Failure::from)?;
    let ceiling = a
        .ceiling
        .as_deref()
        .map(consent::parse_ceiling)
        .transpose()
        .map_err(Failure::from)?;
    let mut st = RunState::load(&ctx.run_dir)?;
    if st.status != RunStatus::Active {
        return Err(Failure::from(Refusal::new(
            core_codes::RUN_NOT_ACTIVE,
            format!(
                "run {} is {}, so it cannot take consent",
                st.run_id, st.status
            ),
            "an active run",
            "dla run init",
        )));
    }
    if let Some(existing) = &st.consent {
        return Err(consent::already_given(existing).into());
    }
    let given = consent::build(tiers, a.raw_values, a.given_by, ceiling, state::now_utc())
        .map_err(Failure::from)?;
    let mut ledger = Ledger::load(&ctx.run_dir)?;
    let seq = append_evidence(&mut ledger, project, &given)?;
    let evidence = evidence_id(&ledger, seq);
    st.consent = Some(given.clone());
    st.save(&ctx.run_dir)?;
    Ok(json!({
        "ok": true,
        "run_id": st.run_id,
        "tiers": given.tiers,
        "raw_values": given.raw_values,
        "given_by": given.given_by.as_str(),
        "evidence": evidence,
        "record": ctx.pointer(state::STATE_FILE),
        "ledger": ctx.ledger_line(seq),
    }))
}
