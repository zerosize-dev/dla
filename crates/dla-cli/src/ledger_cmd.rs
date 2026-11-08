//! `dla ledger add`, `resolve`, `annotate`, `test`, `confirm` and `refute`.

use clap::{Args, Subcommand, ValueEnum};
use dla_core::{
    EvidenceId, FindingId, FindingState, Ledger, NewFinding, OpenItemId, Origin, Resolution,
    RunState, UnresolvedReason,
    checklist::{AnswerStatus, Checklist, ChecklistAnswer},
    error::codes as core_codes,
    finding::{Explanation, ExplanationTag, Holder, Locus, Region, RepairClass, Shape},
    record::FindingAnnotation,
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res, codes},
    parse::parse_enum,
};

/// `dla ledger ...`
#[derive(Subcommand, Debug)]
pub enum LedgerCmd {
    /// Add a finding (always starts as `candidate`).
    Add(AddArgs),
    /// Disposition a finding or open item, or answer a checklist item.
    Resolve(ResolveArgs),
    /// Record writers, removers, holder, owner or region of a finding (no state change).
    Annotate(AnnotateArgs),
    /// Move a finding under test; needs a harness record, and a non-leak sibling for a
    /// leak-side finding.
    Test(MoveArgs),
    /// Confirm a finding (see `dla ledger test`); leak-side needs passing growth, a site,
    /// a holder and a refuted N-* sibling.
    Confirm(MoveArgs),
    /// Refute a finding with a discriminator whose verdict opposes it.
    Refute(MoveArgs),
}

/// Flags of `dla ledger test`, `confirm` and `refute`.
#[derive(Args, Debug)]
pub struct MoveArgs {
    /// The finding (F3, F3.n1).
    #[arg(value_parser = parse_enum::<FindingId>)]
    finding: FindingId,
    /// Evidence IDs the move cites, comma separated.
    #[arg(long, value_delimiter = ',', value_parser = parse_enum::<EvidenceId>)]
    evidence: Vec<EvidenceId>,
}

/// Flags of `dla ledger annotate`.
#[derive(Args, Debug)]
pub struct AnnotateArgs {
    /// The finding (F3, F3.n1).
    #[arg(value_parser = parse_enum::<FindingId>)]
    finding: FindingId,
    /// `file:line` of a site that adds to the holder (repeatable).
    #[arg(long = "writer", value_name = "FILE:LINE")]
    writers: Vec<String>,
    /// `file:line` of a site that removes from the holder (repeatable).
    #[arg(long = "remover", value_name = "FILE:LINE")]
    removers: Vec<String>,
    /// Free-text note.
    #[arg(long)]
    note: Option<String>,
    /// What holds the memory; only set when the finding has no holder yet:
    /// static/global | registration-table | container-field | thread-local | blocked-stack |
    /// closure/continuation | native-handle/global-ref | cycle-only
    #[arg(long, value_parser = parse_enum::<Holder>)]
    holder: Option<Holder>,
    /// Owning field or container on the retention path; only set when none is recorded.
    #[arg(long)]
    owner: Option<String>,
    /// heap | off-heap | non-heap; only set when none is recorded.
    #[arg(long, value_parser = parse_enum::<Region>)]
    region: Option<Region>,
}

/// Flags of `dla ledger add`.
#[derive(Args, Debug)]
pub struct AddArgs {
    /// Where the finding came from: detector:<backend>/<rule>, census, growth-series,
    /// archetype-row, known-leak:<id>, residual, verifier or hypothesis.
    #[arg(long)]
    origin: String,
    /// Supporting evidence IDs, comma separated (at least one is required).
    #[arg(long, value_delimiter = ',', value_parser = parse_enum::<EvidenceId>)]
    evidence: Vec<EvidenceId>,
    /// `file:line` of the suspected site.
    #[arg(long)]
    site: Option<String>,
    /// static/global | registration-table | container-field | thread-local | blocked-stack |
    /// closure/continuation | native-handle/global-ref | cycle-only
    #[arg(long, value_parser = parse_enum::<Holder>)]
    holder: Option<Holder>,
    /// Owning field or container on the retention path.
    #[arg(long)]
    owner: Option<String>,
    /// heap | off-heap | non-heap
    #[arg(long, value_parser = parse_enum::<Region>)]
    region: Option<Region>,
    /// steady | stair | spike | plateau
    #[arg(long, value_parser = parse_enum::<Shape>)]
    shape: Option<Shape>,
    /// app | dependency | runtime
    #[arg(long, value_parser = parse_enum::<Locus>)]
    locus: Option<Locus>,
    /// release-regular | release-exceptional | remove-entry | bound/evict | deregister |
    /// stop/cancel | clear-reference | restructure-sync | return-ownership
    #[arg(long, value_parser = parse_enum::<RepairClass>)]
    repair_class: Option<RepairClass>,
    /// Amplifier, comma separated.
    #[arg(long, value_delimiter = ',')]
    amplifiers: Vec<String>,
    /// Archetype tag suggested for later labelling (never filters anything).
    #[arg(long)]
    suggested_archetype: Option<String>,
    /// Add as a sibling hypothesis of this top-level finding.
    #[arg(long, value_parser = parse_enum::<FindingId>)]
    sibling_of: Option<FindingId>,
    /// The cause this sibling proposes (with --sibling-of).
    #[arg(long)]
    hypothesis: Option<String>,
    /// The result that would refute this sibling (with --sibling-of).
    #[arg(long)]
    refute_if: Option<String>,
    /// Explanation text (with --explanation-tag).
    #[arg(long, requires = "explanation_tag")]
    explanation: Option<String>,
    /// observed | inferred
    #[arg(long, value_parser = parse_enum::<ExplanationTag>, requires = "explanation")]
    explanation_tag: Option<ExplanationTag>,
    /// Refused: only `dla ledger label` assigns an archetype.
    #[arg(long, hide = true)]
    archetype: Option<String>,
}

/// The dispositions `ledger resolve` can give a finding.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveAs {
    /// Left unresolved, with a reason and a remedy.
    #[value(name = "unresolved")]
    Unresolved,
    /// Only static evidence exists.
    #[value(name = "suspected_static_only", alias = "suspected-static-only")]
    SuspectedStaticOnly,
}

/// Flags of `dla ledger resolve`.
#[derive(Args, Debug)]
pub struct ResolveArgs {
    /// A finding (F3, F3.n1), an open item (O2) or a checklist item (intake.scope).
    target: String,
    /// Findings and open items: unresolved | suspected_static_only.
    #[arg(long = "as", value_enum)]
    disposition: Option<ResolveAs>,
    /// time_budget | not_exercised | no_harness | env_unavailable | build_fails |
    /// privilege_declined | tool_missing
    #[arg(long, value_parser = parse_enum::<UnresolvedReason>)]
    reason: Option<UnresolvedReason>,
    /// What would resolve it (required with a reason).
    #[arg(long)]
    remedy: Option<String>,
    /// Open items: a new hypothesis F7 now covers this item.
    #[arg(long, value_parser = parse_enum::<FindingId>)]
    by: Option<FindingId>,
    /// Checklist items: answered | partial | unresolved.
    #[arg(long, value_parser = parse_enum::<AnswerStatus>)]
    status: Option<AnswerStatus>,
    /// Evidence IDs, comma separated (checklist answers; findings may cite more).
    #[arg(long, value_delimiter = ',', value_parser = parse_enum::<EvidenceId>)]
    evidence: Vec<EvidenceId>,
    /// Checklist items: free-text note.
    #[arg(long)]
    note: Option<String>,
}

const BY: &str = "dla ledger resolve";

/// What a resolve target refers to.
enum Target {
    Finding(FindingId),
    OpenItem(OpenItemId),
    Checklist(String),
}

fn classify(raw: &str) -> Res<Target> {
    if let Ok(o) = raw.parse::<OpenItemId>() {
        return Ok(Target::OpenItem(o));
    }
    if let Ok(f) = raw.parse::<FindingId>() {
        return Ok(Target::Finding(f));
    }
    let list = Checklist::standard();
    if list.contains(raw) {
        return Ok(Target::Checklist(raw.to_owned()));
    }
    let ids: Vec<_> = list.items.iter().map(|i| i.id.as_str()).collect();
    Err(Failure::refuse(
        codes::UNKNOWN_TARGET,
        format!("{raw:?} is not a finding, an open item or a checklist item"),
        "a finding (F3, F3.n1), open item (O2) or checklist item",
        format!("dla run status   (checklist items: {})", ids.join(", ")),
    ))
}

fn reject(target: &str, flags: &[(&str, bool)]) -> Res<()> {
    for (flag, present) in flags {
        if *present {
            return Err(Failure::refuse(
                codes::UNSUPPORTED_FLAG,
                format!("{flag} does not apply to {target}"),
                format!("{target} without {flag}"),
                "dla ledger resolve --help",
            ));
        }
    }
    Ok(())
}

fn missing_arg(flag: &str, fix: String) -> Failure {
    Failure::refuse(
        codes::MISSING_ARGUMENT,
        format!("{flag} is required here"),
        flag,
        fix,
    )
}

/// Runs `dla ledger add`.
pub fn add(ctx: &Ctx, a: AddArgs) -> Res<Value> {
    RunState::load(&ctx.run_dir)?;
    let mut ledger = Ledger::load(&ctx.run_dir)?;
    let origin = Origin::parse(&a.origin).map_err(|e| {
        Failure::refuse(
            core_codes::INVALID_ORIGIN,
            e.to_string(),
            "a valid --origin",
            "dla ledger add --origin census --evidence E1",
        )
    })?;
    let mut nf = NewFinding::new(origin, a.evidence);
    nf.sibling_of = a.sibling_of;
    nf.archetype = a.archetype;
    nf.suggested_archetype = a.suggested_archetype;
    nf.hypothesis = a.hypothesis;
    nf.refute_if = a.refute_if;
    nf.holder = a.holder;
    nf.owner = a.owner;
    nf.region = a.region;
    nf.shape = a.shape;
    nf.locus = a.locus;
    nf.site = a.site;
    nf.amplifiers = a.amplifiers;
    nf.repair_class = a.repair_class;
    nf.explanation = match (a.explanation, a.explanation_tag) {
        (Some(text), Some(tag)) => Some(Explanation { text, tag }),
        _ => None,
    };
    let line = ledger.add_finding(nf)?;
    // Read the new finding from the line just appended, never from the ledger's
    // ID-ordered map: a sibling or any non-maximal ID is not the last entry there.
    let dla_core::Record::FindingAdded { finding } = &line.record else {
        unreachable!("add_finding appends a finding_added record");
    };
    let (id, state, parent) = (
        finding.id.to_string(),
        finding.state.to_string(),
        finding.parent.clone(),
    );
    let mut out = json!({
        "ok": true,
        "id": id,
        "state": state,
        "seq": line.seq,
        "record": ctx.ledger_line(line.seq),
    });
    if let Some(p) = parent {
        out["parent"] = Value::String(p.to_string());
    }
    Ok(out)
}

/// Runs `dla ledger annotate`.
pub fn annotate(ctx: &Ctx, a: AnnotateArgs) -> Res<Value> {
    RunState::load(&ctx.run_dir)?;
    let mut ledger = Ledger::load(&ctx.run_dir)?;
    let line = ledger.annotate_record(FindingAnnotation {
        writers: a.writers,
        removers: a.removers,
        note: a.note,
        holder: a.holder,
        owner: a.owner,
        region: a.region,
        ..FindingAnnotation::new(a.finding.clone())
    })?;
    let dla_core::Record::FindingAnnotated(rec) = &line.record else {
        unreachable!("annotate appends a finding_annotated record");
    };
    let mut out = json!({
        "ok": true,
        "id": a.finding.to_string(),
        "writers": rec.writers.len(),
        "removers": rec.removers.len(),
        "seq": line.seq,
        "record": ctx.ledger_line(line.seq),
    });
    if let Some(h) = rec.holder {
        out["holder"] = json!(h.as_str());
    }
    if let Some(o) = &rec.owner {
        out["owner"] = json!(o);
    }
    if let Some(r) = rec.region {
        out["region"] = json!(r.as_str());
    }
    Ok(out)
}

/// Runs `dla ledger test`, `confirm` or `refute`: one transition citing evidence.
pub fn move_to(ctx: &Ctx, a: MoveArgs, to: FindingState) -> Res<Value> {
    RunState::load(&ctx.run_dir)?;
    let mut ledger = Ledger::load(&ctx.run_dir)?;
    let by = match to {
        FindingState::UnderTest => "dla ledger test",
        FindingState::Confirmed => "dla ledger confirm",
        _ => "dla ledger refute",
    };
    let from = ledger.finding(&a.finding).map(|f| f.state);
    let line = ledger.transition(&a.finding, to, None, None, a.evidence.clone(), by)?;
    let mut out = json!({
        "ok": true,
        "finding": a.finding.to_string(),
        "from": from.map(|s| s.to_string()),
        "to": to.as_str(),
        "evidence": a.evidence.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "seq": line.seq,
        "record": ctx.ledger_line(line.seq),
    });
    if to == FindingState::Confirmed
        && let Some(w) = ledger.confirm_warning(&a.finding)
    {
        out["warning"] = json!(w);
    }
    Ok(out)
}

/// Runs `dla ledger resolve`.
pub fn resolve(ctx: &Ctx, a: ResolveArgs) -> Res<Value> {
    RunState::load(&ctx.run_dir)?;
    let target = classify(&a.target)?;
    let mut ledger = Ledger::load(&ctx.run_dir)?;
    match target {
        Target::Finding(id) => resolve_finding(ctx, &mut ledger, id, a),
        Target::OpenItem(id) => resolve_open_item(ctx, &mut ledger, id, a),
        Target::Checklist(item) => resolve_checklist(ctx, &mut ledger, item, a),
    }
}

fn resolve_finding(ctx: &Ctx, ledger: &mut Ledger, id: FindingId, a: ResolveArgs) -> Res<Value> {
    reject(
        "a finding",
        &[
            ("--status", a.status.is_some()),
            ("--by", a.by.is_some()),
            ("--note", a.note.is_some()),
        ],
    )?;
    let to = match a.disposition {
        Some(ResolveAs::Unresolved) => FindingState::Unresolved,
        Some(ResolveAs::SuspectedStaticOnly) => FindingState::SuspectedStaticOnly,
        None => {
            return Err(missing_arg(
                "--as",
                format!(
                    "dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\""
                ),
            ));
        }
    };
    let from = ledger.finding(&id).map(|f| f.state);
    let line = ledger.transition(&id, to, a.reason, a.remedy, a.evidence, BY)?;
    let label = ledger
        .finding(&id)
        .map(|f| f.state_label())
        .unwrap_or_default();
    Ok(json!({
        "ok": true,
        "id": id.to_string(),
        "from": from.map(|s| s.to_string()),
        "state": label,
        "seq": line.seq,
        "record": ctx.ledger_line(line.seq),
    }))
}

fn resolve_open_item(ctx: &Ctx, ledger: &mut Ledger, id: OpenItemId, a: ResolveArgs) -> Res<Value> {
    reject(
        "an open item",
        &[
            ("--status", a.status.is_some()),
            ("--note", a.note.is_some()),
            ("--evidence", !a.evidence.is_empty()),
        ],
    )?;
    if a.by.is_some() && (a.disposition.is_some() || a.reason.is_some() || a.remedy.is_some()) {
        return Err(Failure::refuse(
            codes::UNSUPPORTED_FLAG,
            "--by cannot be combined with --as, --reason or --remedy",
            "either --by <finding> or --as unresolved with a reason and remedy",
            format!("dla ledger resolve {id} --by F7"),
        ));
    }
    let resolution = match (a.by, a.disposition) {
        (Some(f), _) => Resolution::Hypothesis(f),
        (None, Some(ResolveAs::Unresolved)) => {
            let reason = a.reason.ok_or_else(|| {
                missing_arg(
                    "--reason",
                    format!(
                        "dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\""
                    ),
                )
            })?;
            let remedy = a.remedy.ok_or_else(|| {
                missing_arg(
                    "--remedy",
                    format!(
                        "dla ledger resolve {id} --as unresolved --reason {reason} --remedy \"<text>\""
                    ),
                )
            })?;
            Resolution::Unresolved { reason, remedy }
        }
        (None, Some(ResolveAs::SuspectedStaticOnly)) => {
            return Err(Failure::refuse(
                codes::UNSUPPORTED_FLAG,
                "an open item cannot be resolved as suspected_static_only; that is a finding state",
                "--as unresolved or --by <finding>",
                format!(
                    "dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\""
                ),
            ));
        }
        (None, None) => {
            return Err(missing_arg(
                "--as or --by",
                format!(
                    "dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\"   or   dla ledger resolve {id} --by F7"
                ),
            ));
        }
    };
    let line = ledger.resolve_open_item(&id, resolution)?;
    Ok(json!({
        "ok": true,
        "id": id.to_string(),
        "seq": line.seq,
        "record": ctx.ledger_line(line.seq),
    }))
}

fn resolve_checklist(ctx: &Ctx, ledger: &mut Ledger, item: String, a: ResolveArgs) -> Res<Value> {
    reject(
        "a checklist item",
        &[("--as", a.disposition.is_some()), ("--by", a.by.is_some())],
    )?;
    let status = a.status.ok_or_else(|| {
        missing_arg(
            "--status",
            format!("dla ledger resolve {item} --status answered --evidence E1"),
        )
    })?;
    let line = ledger.answer_checklist(ChecklistAnswer {
        item: item.clone(),
        status,
        evidence: a.evidence,
        note: a.note,
        reason: a.reason,
        remedy: a.remedy,
    })?;
    Ok(json!({
        "ok": true,
        "id": item,
        "status": status.as_str(),
        "seq": line.seq,
        "record": ctx.ledger_line(line.seq),
    }))
}
