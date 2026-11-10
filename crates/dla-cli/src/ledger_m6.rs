//! `dla ledger label`, `strong` and `explain` (schema 6).
//!
//! Each command loads the ledger the writes go to (the main one, or a worker's slice),
//! hands the request to `dla-core`, which builds the record with the proof it was accepted
//! on, and prints a short answer. The rules live in `dla_core::{labels, machine, explain}`.

use std::path::PathBuf;

use clap::Args;
use dla_core::{
    EvidenceId, FindingId, FindingState, Ledger, Record, RemovalClaim,
    archetypes::{self, Table},
    attribution,
    error::codes as core_codes,
    finding::ExplanationTag,
    labels::Archetypes,
    paths,
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    ledger_cmd::{open, pointer},
    parse::parse_enum,
};

/// Most archetypes `--suggest` lists.
const SUGGEST_ROWS: usize = 6;

/// Loads the archetype table: `None` (archetypes off) when `DLA_ARCHETYPES=off`, when no
/// plugin data directory can be found or when `data/archetypes.toml` is absent. A table
/// that exists and is invalid is a refusal that names the file.
pub fn archetype_table(data: Option<&PathBuf>) -> Res<Option<Table>> {
    let on = archetypes::archetypes_enabled(&|k| std::env::var(k).ok());
    if !on {
        return Ok(None);
    }
    let Ok(dir) = paths::data_files_dir(data.map(PathBuf::as_path)) else {
        return Ok(None);
    };
    Ok(archetypes::load(&dir)?)
}

/// Flags of `dla ledger label`.
#[derive(Args, Debug)]
pub struct LabelArgs {
    /// The finding (F3, F3.n1).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// The archetype ID (L-* on a leak-side finding, N-* on a non-leak sibling) or
    /// UNCLASSIFIED. DLA never picks a label.
    #[arg(long = "as", value_name = "ID", conflicts_with = "suggest")]
    pub label: Option<String>,
    /// Evidence IDs that meet the archetype's `requires`, comma separated (default: the
    /// evidence the finding cites or is supported by).
    #[arg(long, value_delimiter = ',', value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Vec<EvidenceId>,
    /// Why nothing fits (required with `--as UNCLASSIFIED`).
    #[arg(long)]
    pub note: Option<String>,
    /// Read only: per archetype, the met alternative or the missing evidence kinds.
    #[arg(long)]
    pub suggest: bool,
}

/// Flags of `dla ledger strong`.
#[derive(Args, Debug)]
pub struct StrongArgs {
    /// The finding (F3).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// The second independent record: a retention record or an intervention, comma
    /// separated. A new `dla trend` record may be listed too when the one that confirmed
    /// the finding has no signal.
    #[arg(long, value_delimiter = ',', required = true, value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Vec<EvidenceId>,
}

/// Flags of `dla ledger explain`.
#[derive(Args, Debug)]
pub struct ExplainArgs {
    /// The finding (F3).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// inferred | observed (observed needs --evidence).
    #[arg(long, value_parser = parse_enum::<ExplanationTag>)]
    pub tag: ExplanationTag,
    /// `file:line` of the sites that add to the holder, comma separated (default: the
    /// recorded writers).
    #[arg(long, value_delimiter = ',', value_name = "FILE:LINE")]
    pub adds: Vec<String>,
    /// never | not-found | conditional | wrong-key | unreachable
    #[arg(long, value_parser = parse_enum::<RemovalClaim>)]
    pub removal: RemovalClaim,
    /// `file:line` of the remover (required with conditional, wrong-key, unreachable).
    #[arg(long, value_name = "FILE:LINE")]
    pub remover: Option<String>,
    /// A retention, intervention or tied growth record that shows the pairing.
    #[arg(long, value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Option<EvidenceId>,
    /// The explanation (40 to 600 characters, one line). It names the site, the holder, an
    /// adding site and how removal fails. DLA checks that these are stated and consistent
    /// with the recorded writers and removers; it cannot check that they are true.
    #[arg(long)]
    pub text: String,
}

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let cut: String = s.chars().take(n.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

/// `dla ledger label`.
pub fn label(ctx: &Ctx, data: Option<&PathBuf>, a: LabelArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let table = archetype_table(data)?;
    if a.suggest {
        return suggest(&ledger, &a, table.as_ref());
    }
    let Some(as_) = a.label.as_deref() else {
        return Err(Failure::refuse(
            core_codes::EMPTY_FIELD,
            "label needs --as <ID> or --suggest",
            "--as <archetype ID | UNCLASSIFIED> or --suggest",
            format!("dla ledger label {} --suggest", a.finding),
        ));
    };
    let line = ledger.label(
        &a.finding,
        as_,
        &a.evidence,
        a.note.as_deref(),
        Archetypes::from_table(table.as_ref()),
    )?;
    let Record::FindingLabeled(rec) = &line.record else {
        unreachable!("label appends a finding_labeled record");
    };
    let mut out = json!({
        "ok": true,
        "finding": rec.finding.to_string(),
        "label": rec.label,
        "previous": rec.previous,
        "kinds": rec.satisfied.kinds,
        "evidence": rec.satisfied.evidence.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    });
    if rec.region_mismatch {
        let f = ledger.finding(&rec.finding);
        let have = f.and_then(|f| f.region).map_or("none", |r| r.as_str());
        let fits: Vec<&str> = table
            .as_ref()
            .and_then(|t| t.get(&rec.label))
            .map(|e| e.regions.iter().map(|r| r.as_str()).collect())
            .unwrap_or_default();
        out["warning"] = json!(format!(
            "the finding's region {have} is not among the regions {} fits ({}); the label stands and the mismatch is recorded",
            rec.label,
            fits.join(", ")
        ));
    }
    Ok(out)
}

fn suggest(ledger: &Ledger, a: &LabelArgs, table: Option<&Table>) -> Res<Value> {
    let Some(table) = table else {
        return Ok(json!({
            "ok": true,
            "finding": a.finding.to_string(),
            "archetypes": "off",
            "note": "archetypes are off (DLA_ARCHETYPES=off or no data/archetypes.toml): only `dla ledger label --as UNCLASSIFIED --note` applies",
            "rows": [],
        }));
    };
    let rows = ledger.suggest_labels(&a.finding, table)?;
    let total = rows.len();
    let shown: Vec<Value> = rows
        .into_iter()
        .take(SUGGEST_ROWS)
        .map(|mut r| {
            r.summary = clip(&r.summary, 70);
            serde_json::to_value(r).unwrap_or(Value::Null)
        })
        .collect();
    let mut out = json!({
        "ok": true,
        "finding": a.finding.to_string(),
        "archetypes": "on",
        "rows": shown,
    });
    if total > SUGGEST_ROWS {
        out["more"] = json!(total - SUGGEST_ROWS);
    }
    out["next"] = json!(format!(
        "dla ledger label {} --as <ID> [--evidence E1,E2]   (a met row is ready; DLA never picks)",
        a.finding
    ));
    Ok(out)
}

/// `dla ledger strong`.
pub fn strong(ctx: &Ctx, a: StrongArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let finding = ledger.own_finding_id(&a.finding);
    let evidence: Vec<EvidenceId> = a
        .evidence
        .iter()
        .map(|e| ledger.own_evidence_id(e))
        .collect();
    let from = ledger.finding(&finding).map(|f| f.state);
    let line = ledger.transition(
        &finding,
        FindingState::Strong,
        None,
        None,
        evidence.clone(),
        "dla ledger strong",
    )?;
    Ok(json!({
        "ok": true,
        "finding": finding.to_string(),
        "from": from.map(|s| s.to_string()),
        "to": FindingState::Strong.as_str(),
        "evidence": evidence.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    }))
}

/// The writers and removers the census found for the finding's owner.
fn census_sites(ctx: &Ctx, ledger: &Ledger, finding: &FindingId) -> (Vec<String>, Vec<String>) {
    let Some(owner) = ledger.finding(finding).and_then(|f| f.owner.clone()) else {
        return (Vec::new(), Vec::new());
    };
    let Some(project) = ctx.run_dir.parent().and_then(|p| p.parent()) else {
        return (Vec::new(), Vec::new());
    };
    attribution::resolve(&ctx.run_dir, project, &owner)
        .map_or((Vec::new(), Vec::new()), |r| (r.writers, r.removers))
}

/// `dla ledger explain`.
pub fn explain(ctx: &Ctx, a: ExplainArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let finding = ledger.own_finding_id(&a.finding);
    let (cw, cr) = census_sites(ctx, &ledger, &finding);
    let line = ledger.explain(
        &finding,
        a.tag,
        &a.adds,
        a.removal,
        a.remover.as_deref(),
        a.evidence.as_ref(),
        &a.text,
        &cw,
        &cr,
    )?;
    let Record::FindingExplained(rec) = &line.record else {
        unreachable!("explain appends a finding_explained record");
    };
    Ok(json!({
        "ok": true,
        "finding": rec.finding.to_string(),
        "tag": rec.tag.as_str(),
        "adds": rec.adds,
        "removal": rec.removal.as_str(),
        "remover": rec.remover,
        "checked": "the text states the site, the holder, an adding site and the removal, consistently with the recorded writers and removers; DLA does not check that it is true",
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    }))
}
