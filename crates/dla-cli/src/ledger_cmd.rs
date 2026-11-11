//! `dla ledger add`, `resolve`, `annotate`, `test`, `confirm`, `refute`, `done`, `merge`
//! and `verify`.
//!
//! With `--worker <name>` every write goes to the worker's slice `ledger.<name>.jsonl`
//! and a bare `F3` means the worker's own `<name>/F3`.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use dla_core::{
    EvidenceId, FindingId, FindingState, Ledger, NewFinding, OpenItemId, Origin, Resolution,
    RunState, UnresolvedReason,
    checklist::{AnswerStatus, Checklist, ChecklistAnswer},
    error::codes as core_codes,
    finding::{Explanation, ExplanationTag, Holder, Locus, Region, RepairClass, Shape},
    merge::MergeReport,
    record::FindingAnnotation,
    slice::slice_file_name,
    verify as core_verify,
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res, codes},
    ledger_m6,
    parse::parse_enum,
};

/// `dla ledger ...`
#[derive(Subcommand, Debug)]
pub enum LedgerCmd {
    /// Add a finding (always starts as `candidate`).
    Add(AddArgs),
    /// Disposition a finding or open item, or answer a checklist item.
    Resolve(ResolveArgs),
    /// Record writers, removers, holder, owner, region or site of a finding (no state change).
    Annotate(AnnotateArgs),
    /// Move a finding under test; needs a harness record, and a non-leak sibling for a
    /// leak-side finding.
    Test(MoveArgs),
    /// Confirm a finding (see `dla ledger test`); leak-side needs passing growth, a site,
    /// a holder and a refuted N-* sibling.
    Confirm(MoveArgs),
    /// Refute a finding with a discriminator whose verdict opposes it.
    Refute(MoveArgs),
    /// End this worker's slice (`dla --worker <name> ledger done`); worker mode only.
    Done,
    /// Merge the workers' slices into the main ledger, then apply the cross-worker gate:
    /// duplicates, contradictions, unfinished workers, runtimes without a worker. Core
    /// only; safe to repeat.
    Merge,
    /// Record what an independent verifier concluded about a finding from its bundle
    /// (core only).
    Verify(VerifyArgs),
    /// Label a confirmed or strong finding with an archetype, when the evidence the
    /// archetype requires is recorded; or `--as UNCLASSIFIED --note`; or `--suggest`.
    Label(ledger_m6::LabelArgs),
    /// Mark a confirmed leak-side finding strong, on a second independent class of evidence
    /// (retention or intervention).
    Strong(ledger_m6::StrongArgs),
    /// Record a checked explanation of a confirmed or strong leak-side finding.
    Explain(ledger_m6::ExplainArgs),
}

/// Flags of `dla ledger verify`.
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// The finding (F3).
    #[arg(value_parser = parse_enum::<FindingId>)]
    finding: FindingId,
    /// The verifier's site, `file:line`, or `none` when it found none.
    #[arg(long)]
    site: String,
    /// The verifier's holder: static/global | registration-table | container-field |
    /// thread-local | blocked-stack | closure/continuation | native-handle/global-ref |
    /// cycle-only
    #[arg(long, value_parser = parse_enum::<Holder>)]
    holder: Holder,
    /// The verifier's mechanism, in words.
    #[arg(long)]
    mechanism: String,
    /// The verifier's label: an archetype ID or UNCLASSIFIED. With archetypes off only
    /// UNCLASSIFIED is accepted and no label is compared.
    #[arg(long, default_value = "UNCLASSIFIED")]
    label: String,
    /// The sha256 `dla bundle` printed for the bundle the verifier read.
    #[arg(long)]
    bundle: String,
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
    /// What holds the memory; only set when the finding has no holder yet, or changed with --evidence:
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
    /// `file:line` of the finding; only set when the finding has no site yet.
    #[arg(long, value_name = "FILE:LINE")]
    site: Option<String>,
    /// Growth evidence (heap-graph, dump or live) that observed the holder; lets --holder
    /// change a holder already set. Comma separated.
    #[arg(long, value_delimiter = ',', value_parser = parse_enum::<EvidenceId>)]
    evidence: Vec<EvidenceId>,
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
    /// Refused from schema 6: use `dla ledger explain`, which checks the explanation.
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
/// Runs one `dla ledger ...` subcommand.
pub fn dispatch(ctx: &Ctx, data: Option<&PathBuf>, cmd: LedgerCmd) -> Res<Value> {
    match cmd {
        LedgerCmd::Add(a) => add(ctx, a),
        LedgerCmd::Resolve(a) => resolve(ctx, a),
        LedgerCmd::Annotate(a) => annotate(ctx, a),
        LedgerCmd::Test(a) => move_to(ctx, a, FindingState::UnderTest),
        LedgerCmd::Confirm(a) => move_to(ctx, a, FindingState::Confirmed),
        LedgerCmd::Refute(a) => move_to(ctx, a, FindingState::Refuted),
        LedgerCmd::Done => done(ctx),
        LedgerCmd::Merge => merge(ctx),
        LedgerCmd::Verify(a) => verify(ctx, data, a),
        LedgerCmd::Label(a) => ledger_m6::label(ctx, data, a),
        LedgerCmd::Strong(a) => ledger_m6::strong(ctx, a),
        LedgerCmd::Explain(a) => ledger_m6::explain(ctx, a),
    }
}

/// Opens the ledger writes go to: the main one, or the worker's slice.
pub(crate) fn open(ctx: &Ctx) -> Res<Ledger> {
    RunState::load(&ctx.run_dir)?;
    Ok(Ledger::load_for(&ctx.run_dir, ctx.worker.as_deref())?)
}

/// Pointer to a line of the file `ledger` appends to.
pub(crate) fn pointer(ctx: &Ctx, seq: u64) -> String {
    match &ctx.worker {
        Some(w) => format!("{}#L{seq}", ctx.pointer(&slice_file_name(w))),
        None => ctx.ledger_line(seq),
    }
}

fn core_only(what: &str, fix: String) -> Failure {
    Failure::refuse(
        dla_core::slice::codes::CORE_ONLY,
        format!("{what} is for the core; this command was run with --worker"),
        "the core (no --worker and no DLA_WORKER)",
        fix,
    )
}

pub fn add(ctx: &Ctx, a: AddArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
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
    let warnings = match &nf.site {
        Some(site) => check_site_line(ctx, "--site", site, nf.owner.as_deref())?,
        None => Vec::new(),
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
        "record": pointer(ctx, line.seq),
    });
    if let Some(p) = parent {
        out["parent"] = Value::String(p.to_string());
    }
    if !warnings.is_empty() {
        out["warnings"] = json!(warnings);
    }
    Ok(out)
}

/// Runs `dla ledger annotate`.
pub fn annotate(ctx: &Ctx, a: AnnotateArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let a_finding = ledger.own_finding_id(&a.finding);
    let owner = a
        .owner
        .clone()
        .or_else(|| ledger.finding(&a_finding).and_then(|f| f.owner.clone()));
    let mut warnings: Vec<String> = Vec::new();
    for w in &a.writers {
        warnings.extend(check_site_line(ctx, "--writer", w, owner.as_deref())?);
    }
    if let Some(site) = &a.site {
        warnings.extend(check_site_line(ctx, "--site", site, owner.as_deref())?);
    }
    let line = ledger.annotate_record(FindingAnnotation {
        writers: a.writers,
        removers: a.removers,
        note: a.note,
        holder: a.holder,
        owner: a.owner,
        region: a.region,
        site: a.site,
        evidence: a.evidence,
        ..FindingAnnotation::new(a.finding.clone())
    })?;
    let dla_core::Record::FindingAnnotated(rec) = &line.record else {
        unreachable!("annotate appends a finding_annotated record");
    };
    let mut out = json!({
        "ok": true,
        "id": a_finding.to_string(),
        "writers": rec.writers.len(),
        "removers": rec.removers.len(),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
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
    if let Some(s) = &rec.site {
        out["site"] = json!(s);
    }
    if !warnings.is_empty() {
        out["warnings"] = json!(warnings);
    }
    if let Some(old) = rec.replaced_holder {
        out["replaced_holder"] = json!(old.as_str());
        out["evidence"] = json!(
            rec.evidence
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
    }
    Ok(out)
}

/// Runs `dla ledger test`, `confirm` or `refute`: one transition citing evidence.
pub fn move_to(ctx: &Ctx, a: MoveArgs, to: FindingState) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let by = match to {
        FindingState::UnderTest => "dla ledger test",
        FindingState::Confirmed => "dla ledger confirm",
        _ => "dla ledger refute",
    };
    let finding = ledger.own_finding_id(&a.finding);
    let evidence: Vec<EvidenceId> = a
        .evidence
        .iter()
        .map(|e| ledger.own_evidence_id(e))
        .collect();
    let from = ledger.finding(&finding).map(|f| f.state);
    let line = ledger.transition(&finding, to, None, None, evidence.clone(), by)?;
    let mut out = json!({
        "ok": true,
        "finding": finding.to_string(),
        "from": from.map(|s| s.to_string()),
        "to": to.as_str(),
        "evidence": evidence.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    });
    if to == FindingState::Confirmed
        && let Some(w) = ledger.confirm_warning(&finding)
    {
        out["warning"] = json!(w);
    }
    Ok(out)
}

/// Runs `dla ledger resolve`.
pub fn resolve(ctx: &Ctx, a: ResolveArgs) -> Res<Value> {
    let target = classify(&a.target)?;
    let mut ledger = open(ctx)?;
    match target {
        Target::Finding(id) => resolve_finding(ctx, &mut ledger, id, a),
        Target::OpenItem(id) => resolve_open_item(ctx, &mut ledger, id, a),
        Target::Checklist(item) => resolve_checklist(ctx, &mut ledger, item, a),
    }
}

fn resolve_finding(ctx: &Ctx, ledger: &mut Ledger, id: FindingId, a: ResolveArgs) -> Res<Value> {
    let id = ledger.own_finding_id(&id);
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
        "record": pointer(ctx, line.seq),
    }))
}

fn resolve_open_item(ctx: &Ctx, ledger: &mut Ledger, id: OpenItemId, a: ResolveArgs) -> Res<Value> {
    let id = ledger.own_open_item_id(&id);
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
        "record": pointer(ctx, line.seq),
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
        "record": pointer(ctx, line.seq),
    }))
}

/// Runs `dla --worker <name> ledger done`: ends the worker's slice.
fn done(ctx: &Ctx) -> Res<Value> {
    let Some(w) = ctx.worker.clone() else {
        return Err(Failure::refuse(
            dla_core::slice::codes::WORKER_ONLY,
            "`ledger done` ends a worker's slice, and no worker is named",
            "--worker <name> (or DLA_WORKER)",
            format!("dla --run {} --worker <name> ledger done", ctx.run_id),
        ));
    };
    let mut ledger = open(ctx)?;
    if let Some(d) = ledger.slice_info(&w).and_then(|r| r.done.as_ref()) {
        return Ok(json!({
            "ok": true,
            "worker": w,
            "already_done": true,
            "open": d.open,
            "record": pointer(ctx, ledger.lines().len() as u64),
        }));
    }
    let line = ledger.finish_slice()?;
    let dla_core::Record::SliceDone(d) = &line.record else {
        unreachable!("finish_slice appends a slice_done record");
    };
    Ok(json!({
        "ok": true,
        "worker": w,
        "open": d.open,
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    }))
}

/// Runs `dla ledger merge`.
fn merge(ctx: &Ctx) -> Res<Value> {
    if let Some(w) = &ctx.worker {
        return Err(core_only(
            "`ledger merge`",
            format!("dla --run {} ledger merge   (not --worker {w})", ctx.run_id),
        ));
    }
    RunState::load(&ctx.run_dir)?;
    let r: MergeReport = dla_core::merge::merge(&ctx.run_dir)?;
    let mut out = json!({
        "ok": true,
        "merged_lines": r.merged_lines,
        "total_lines": r.total_lines,
        "duplicates": r.duplicates,
        "contradictions": r.contradictions.len(),
        "unfinished": r.unfinished,
        "status": r.status.as_str(),
    });
    if !r.torn_tail.is_empty() {
        out["torn_tail"] = json!(r.torn_tail);
    }
    if !r.invalid_slices.is_empty() {
        out["invalid_slices"] = json!(r.invalid_slices);
    }
    if !r.uncovered_runtimes.is_empty() {
        out["uncovered_runtimes"] = json!(r.uncovered_runtimes);
    }
    if !r.unfinished.is_empty() {
        out["next"] = json!(
            "dla run status   (each unfinished worker has an open item; it closes by itself when the worker ends its slice and the merge is repeated)"
        );
    }
    Ok(out)
}

/// Checks that `sha` is the manifest hash of a bundle `dla bundle` wrote for `finding` in
/// this run (`bundle/<F>` or a repeat `bundle/<F>.<n>`), so a verification always points
/// at the files the verifier was given.
fn bundle_of(ctx: &Ctx, finding: &FindingId, sha: &str) -> Res<()> {
    use sha2::{Digest, Sha256};
    let base = finding.to_string().replace('/', "_");
    let dir = ctx.run_dir.join("bundle");
    let found = std::fs::read_dir(&dir).ok().is_some_and(|rd| {
        rd.filter_map(Result::ok).any(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let ours = name == base
                || name
                    .strip_prefix(&base)
                    .and_then(|rest| rest.strip_prefix('.'))
                    .is_some_and(|n| n.parse::<u32>().is_ok());
            ours && std::fs::read(e.path().join("manifest.json"))
                .is_ok_and(|b| hex::encode(Sha256::digest(&b)) == sha)
        })
    });
    if found {
        return Ok(());
    }
    Err(Failure::refuse(
        codes::UNKNOWN_BUNDLE,
        format!(
            "no bundle of {finding} in run {} has manifest sha256 {sha}",
            ctx.run_id
        ),
        format!("the manifest_sha256 that `dla bundle {finding}` printed"),
        format!("dla bundle {finding}"),
    ))
}

/// Runs `dla ledger verify`.
fn verify(ctx: &Ctx, data: Option<&PathBuf>, a: VerifyArgs) -> Res<Value> {
    if ctx.worker.is_some() {
        return Err(core_only(
            "`ledger verify`",
            format!(
                "dla --run {} ledger verify {} --site {} --holder {} --mechanism \"<text>\" --bundle {}",
                ctx.run_id, a.finding, a.site, a.holder, a.bundle
            ),
        ));
    }
    let site = core_verify::parse_verifier_site(&a.site).map_err(|m| {
        Failure::refuse(
            core_codes::INVALID_SITE,
            m,
            "--site file:line, or none",
            format!(
                "dla ledger verify {} --site src/Main.java:42 --holder {} --mechanism \"<text>\" --bundle {}",
                a.finding, a.holder, a.bundle
            ),
        )
    })?;
    bundle_of(ctx, &a.finding, &a.bundle)?;
    let mut ledger = open(ctx)?;
    let table = ledger_m6::archetype_table(data)?;
    let (line, item) = ledger.verify(
        &a.finding,
        site,
        a.holder,
        &a.mechanism,
        &a.label,
        &a.bundle,
        dla_core::labels::Archetypes::from_table(table.as_ref()),
    )?;
    let dla_core::Record::Verification(v) = &line.record else {
        unreachable!("verify appends a verification record");
    };
    let mut out = json!({
        "ok": true,
        "finding": a.finding.to_string(),
        "agrees_site": v.agrees_site,
        "agrees_holder": v.agrees_holder,
        "agrees_label": v.agrees_label,
        "verified": core_verify::agrees(v),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    });
    if let Some(i) = item
        && let dla_core::Record::OpenItem(o) = &i.record
    {
        out["open_item"] = json!(o.id.to_string());
        out["disagreement"] = json!(o.text);
    }
    Ok(out)
}

// ------------------------------------------------------------------ site lines

/// Comment syntax of a source file, by extension.
struct Comments {
    line: &'static [&'static str],
    block: Option<(&'static str, &'static str)>,
}

fn comments_for(file: &str) -> Comments {
    let ext = file
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "java" | "kt" | "kts" | "scala" | "groovy" | "js" | "mjs" | "cjs" | "jsx" | "ts"
        | "tsx" | "go" | "rs" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "cs" | "swift" => {
            Comments {
                line: &["//"],
                block: Some(("/*", "*/")),
            }
        }
        "py" | "rb" | "sh" => Comments {
            line: &["#"],
            block: None,
        },
        _ => Comments {
            line: &[],
            block: None,
        },
    }
}

/// Per line of `text`: whether it holds anything but blanks and comments.
fn code_lines(text: &str, syntax: &Comments) -> Vec<bool> {
    let mut in_block = false;
    let mut out = Vec::new();
    for line in text.lines() {
        let b = line.as_bytes();
        let (mut i, mut code, mut quote) = (0, false, None::<u8>);
        while i < b.len() {
            if in_block {
                match syntax.block {
                    Some((_, end)) if b[i..].starts_with(end.as_bytes()) => {
                        in_block = false;
                        i += end.len();
                    }
                    _ => i += 1,
                }
                continue;
            }
            let c = b[i];
            if let Some(q) = quote {
                if c == b'\\' {
                    i += 2;
                    continue;
                }
                if c == q {
                    quote = None;
                }
                i += 1;
                continue;
            }
            if c.is_ascii_whitespace() {
                i += 1;
                continue;
            }
            if syntax.line.iter().any(|m| b[i..].starts_with(m.as_bytes())) {
                break;
            }
            if let Some((start, _)) = syntax.block
                && b[i..].starts_with(start.as_bytes())
            {
                in_block = true;
                i += start.len();
                continue;
            }
            code = true;
            if matches!(c, b'"' | b'\'' | b'`') {
                quote = Some(c);
            }
            i += 1;
        }
        out.push(code);
    }
    out
}

/// The last component of an owner such as `com.acme.Cache.entries` or `Cache::items`.
fn identifier(owner: &str) -> Option<&str> {
    owner
        .trim()
        .trim_end_matches("()")
        .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
        .find(|p| !p.is_empty())
        .filter(|p| p.chars().count() >= 2)
}

fn site_refusal(site: &str, what: String, file: &str, ident: Option<&str>) -> Failure {
    let grep = match ident {
        Some(i) => format!("grep -n '{i}' {file}"),
        None => format!("grep -n '<the holder or field name>' {file}"),
    };
    Failure::refuse(
        core_codes::INVALID_SITE,
        format!("{site}: {what}"),
        format!("a line of {file} that holds code"),
        format!(
            "take the line number from `{grep}` (or `cat -n {file}`), not from counting an unnumbered listing"
        ),
    )
}

/// Checks that `site` (`file:line`) names a line of code that exists in the project file:
/// refuses a line outside the file, past its end, or blank or comment-only; returns a
/// warning when the line does not contain the owner's identifier, naming the nearest
/// lines that do. A site that does not parse, or a file that is not under the project, is
/// left to the ledger's own check (the file case is a warning: the line was not verified).
fn check_site_line(ctx: &Ctx, flag: &str, site: &str, owner: Option<&str>) -> Res<Vec<String>> {
    let Some((file, line)) = site
        .trim()
        .rsplit_once(':')
        .and_then(|(f, l)| Some((f, l.parse::<usize>().ok()?)))
    else {
        return Ok(Vec::new());
    };
    let Some(project) = ctx.run_dir.parent().and_then(Path::parent) else {
        return Ok(Vec::new());
    };
    let ident = owner.and_then(identifier);
    // A site names a file of the project: an absolute path or one that climbs out of the
    // project is never read, so no line of a file outside it is quoted back.
    let inside = Path::new(file).components().all(|c| {
        matches!(
            c,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    });
    let read = if inside {
        std::fs::read(project.join(file)).ok()
    } else {
        None
    };
    let Some(bytes) = read else {
        return Ok(vec![format!(
            "{flag} {site}: {file} was not found under the project, so the line was not checked"
        )]);
    };
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    if line == 0 || line > lines.len() {
        return Err(site_refusal(
            site,
            format!(
                "{flag} line {line} is outside {file}, which has {} line(s)",
                lines.len()
            ),
            file,
            ident,
        ));
    }
    let has_code = code_lines(&text, &comments_for(file));
    if !has_code.get(line - 1).copied().unwrap_or(true) {
        let kind = if lines[line - 1].trim().is_empty() {
            "blank"
        } else {
            "a comment"
        };
        return Err(site_refusal(
            site,
            format!("{flag} line {line} of {file} is {kind}, not code"),
            file,
            ident,
        ));
    }
    let Some(ident) = ident else {
        return Ok(Vec::new());
    };
    if lines[line - 1].contains(ident) {
        return Ok(Vec::new());
    }
    let mut near: Vec<usize> = (1..=lines.len())
        .filter(|n| *n != line && lines[n - 1].contains(ident))
        .collect();
    near.sort_by_key(|n| (n.abs_diff(line), *n));
    near.truncate(3);
    let shown: Vec<String> = near
        .iter()
        .map(|n| {
            let t: String = lines[n - 1].trim().chars().take(60).collect();
            format!("{n}: {t}")
        })
        .collect();
    Ok(vec![if shown.is_empty() {
        format!(
            "{flag} {site}: line {line} does not contain `{ident}`, and no other line of {file} does"
        )
    } else {
        format!(
            "{flag} {site}: line {line} does not contain `{ident}`; nearest lines that do: {}",
            shown.join(" | ")
        )
    }])
}
