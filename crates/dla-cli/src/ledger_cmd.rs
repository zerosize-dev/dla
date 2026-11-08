//! `dla ledger add`, `resolve`, `annotate`, `test`, `confirm`, `refute`, `done`, `merge`
//! and `verify`.
//!
//! With `--worker <name>` every write goes to the worker's slice `ledger.<name>.jsonl`
//! and a bare `F3` means the worker's own `<name>/F3`.

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
    /// The verifier's label: an archetype ID or UNCLASSIFIED.
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
    /// `file:line` of the finding; only set when the finding has no site yet.
    #[arg(long, value_name = "FILE:LINE")]
    site: Option<String>,
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
/// Runs one `dla ledger ...` subcommand.
pub fn dispatch(ctx: &Ctx, cmd: LedgerCmd) -> Res<Value> {
    match cmd {
        LedgerCmd::Add(a) => add(ctx, a),
        LedgerCmd::Resolve(a) => resolve(ctx, a),
        LedgerCmd::Annotate(a) => annotate(ctx, a),
        LedgerCmd::Test(a) => move_to(ctx, a, FindingState::UnderTest),
        LedgerCmd::Confirm(a) => move_to(ctx, a, FindingState::Confirmed),
        LedgerCmd::Refute(a) => move_to(ctx, a, FindingState::Refuted),
        LedgerCmd::Done => done(ctx),
        LedgerCmd::Merge => merge(ctx),
        LedgerCmd::Verify(a) => verify(ctx, a),
    }
}

/// Opens the ledger writes go to: the main one, or the worker's slice.
fn open(ctx: &Ctx) -> Res<Ledger> {
    RunState::load(&ctx.run_dir)?;
    Ok(Ledger::load_for(&ctx.run_dir, ctx.worker.as_deref())?)
}

/// Pointer to a line of the file `ledger` appends to.
fn pointer(ctx: &Ctx, seq: u64) -> String {
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
    Ok(out)
}

/// Runs `dla ledger annotate`.
pub fn annotate(ctx: &Ctx, a: AnnotateArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let a_finding = ledger.own_finding_id(&a.finding);
    let line = ledger.annotate_record(FindingAnnotation {
        writers: a.writers,
        removers: a.removers,
        note: a.note,
        holder: a.holder,
        owner: a.owner,
        region: a.region,
        site: a.site,
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
    if !r.uncovered_runtimes.is_empty() {
        out["uncovered_runtimes"] = json!(r.uncovered_runtimes);
    }
    if !r.unfinished.is_empty() {
        out["next"] = json!(
            "dla run status   (each unfinished worker has an open item; resolve it or rerun the worker)"
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
fn verify(ctx: &Ctx, a: VerifyArgs) -> Res<Value> {
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
    let (line, item) = ledger.verify(
        &a.finding,
        site,
        a.holder,
        &a.mechanism,
        &a.label,
        &a.bundle,
    )?;
    let dla_core::Record::Verification(v) = &line.record else {
        unreachable!("verify appends a verification record");
    };
    let mut out = json!({
        "ok": true,
        "finding": a.finding.to_string(),
        "agrees_site": v.agrees_site,
        "agrees_holder": v.agrees_holder,
        "verified": v.agrees_site && v.agrees_holder,
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
