//! `dla ledger add`, `resolve`, `annotate`, `test`, `confirm`, `refute`, `done`, `merge`
//! and `verify`.
//!
//! With `--worker <name>` every write goes to the worker's slice `ledger.<name>.jsonl`
//! and a bare `F3` means the worker's own `<name>/F3`.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use dla_core::{
    EvidenceId, FindingId, FindingState, Ledger, NewFinding, OpenItemId, Origin, Resolution,
    RunState, RunView, UnresolvedReason,
    checklist::{AnswerStatus, Checklist, ChecklistAnswer},
    error::codes as core_codes,
    finding::{Explanation, ExplanationTag, Holder, Locus, Region, RepairClass, Shape},
    gate,
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

#[path = "ledger_queue.rs"]
mod ledger_queue;
#[path = "ledger_release.rs"]
mod ledger_release;

pub use ledger_queue::list;

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
    /// Merge a finding into another finding of the same leak (the same qualified owner, or
    /// the same `file:line` site) in the same worker and runtime; core and worker slices.
    Duplicate(ledger_m6::DuplicateArgs),
    /// The findings in rank order, one line of JSON each: id, state, tier, rank among the
    /// candidates ranked for measurement, origin, site, holder, owner, writer and remover
    /// counts, and the queue status (pending, settled with the harnesses that targeted it,
    /// not-eligible). Read only.
    List(ledger_queue::ListArgs),
    /// Close every open candidate that the measurement queue does not hold back, through the
    /// rules of `dla ledger resolve`: suspected_static_only when it has static evidence, else
    /// unresolved, with reason not_exercised (time_budget when the budget reached its floor)
    /// and a remedy built from its history. Under-test findings are not touched. Prints what
    /// it closed and what it left open.
    CloseRest(ledger_queue::CloseRestArgs),
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
    /// Owning field or container on the retention path; only set when none is recorded (an
    /// owner is set once; when it is wrong, add a finding with the right owner and merge the
    /// old one into it with `dla ledger duplicate`).
    #[arg(long)]
    owner: Option<String>,
    /// heap | off-heap | non-heap; only set when none is recorded.
    #[arg(long, value_parser = parse_enum::<Region>)]
    region: Option<Region>,
    /// `file:line` of the finding; only set when the finding has no site yet.
    #[arg(long, value_name = "FILE:LINE")]
    site: Option<String>,
    /// Growth evidence (heap-graph, dump or live) that observed the holder; lets --holder
    /// change a holder already set. With --owner it is the evidence the owner warning
    /// compares. Comma separated.
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
    /// `file:line` of the line that releases the holder (the stop, close, dispose, unsubscribe
    /// or cancel the API requires); required with `--hypothesis N-RELEASE`, refused otherwise.
    #[arg(long, value_name = "FILE:LINE")]
    remover: Option<String>,
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
    /// not_exercised | inconclusive (measured, undecided) | no_harness | env_unavailable |
    /// build_fails | privilege_declined | tool_missing | time_budget (only when the deadline
    /// is near, or a command was refused for time with nothing smaller that fits)
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
    /// intake.workload with --status unresolved: the runtime the answer is about (python,
    /// native, jvm, ...). A blocker reason closes the queue of this runtime only; it is
    /// required when more than one runtime has ranked candidates.
    #[arg(long)]
    runtime: Option<String>,
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
        LedgerCmd::Duplicate(a) => ledger_m6::duplicate(ctx, a),
        LedgerCmd::List(a) => ledger_queue::list(ctx, a).map(Value::String),
        LedgerCmd::CloseRest(a) => ledger_queue::close_rest(ctx, a),
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
    nf.remover = a.remover;
    nf.holder = a.holder;
    nf.owner = a.owner;
    nf.region = a.region;
    nf.shape = a.shape;
    nf.locus = a.locus;
    nf.site = a.site;
    // A finding added from growth evidence takes the site of the first passing column that
    // names one (an allocation stack's first project frame) when none is given.
    if nf.site.is_none() && a.origin == "growth-series" {
        // The largest of them: a capture summary's finding stands for the evidence's untied
        // columns, and its site is the largest one's.
        nf.site = nf.supports.iter().find_map(|e| {
            let passed = ledger.evidence_by_id(e)?.result.get("passed")?.as_array()?;
            let size = |c: &Value| c.get("slope").and_then(Value::as_f64).map_or(0.0, f64::abs);
            passed
                .iter()
                .filter(|c| c.get("site").and_then(Value::as_str).is_some())
                .max_by(|a, b| size(a).total_cmp(&size(b)))
                .and_then(|c| c.get("site")?.as_str().map(str::to_owned))
        });
    }
    nf.amplifiers = a.amplifiers;
    nf.repair_class = a.repair_class;
    nf.explanation = match (a.explanation, a.explanation_tag) {
        (Some(text), Some(tag)) => Some(Explanation { text, tag }),
        _ => None,
    };
    let mut warnings = match &nf.site {
        Some(site) => check_site_line(ctx, "--site", site, nf.owner.as_deref())?,
        None => Vec::new(),
    };
    warnings.extend(owner_warning(
        &ledger,
        nf.owner.as_deref(),
        &nf.supports,
        None,
    ));
    let (nf_owner, nf_site) = (nf.owner.clone(), nf.site.clone());
    warnings.extend(held_line_warning(
        &ledger,
        nf_owner.as_deref(),
        nf_site.as_deref(),
    ));
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

/// A warning when `owner` is the holder a retention record of the Python holder chains names and
/// `site` is none of the lines that holder keeps: a finding at one of them has a column of its
/// own that was measured. Not a refusal.
fn held_line_warning(ledger: &Ledger, owner: Option<&str>, site: Option<&str>) -> Option<String> {
    let (owner, site) = (owner?, site?.trim());
    let mut held: Vec<String> = Vec::new();
    for rec in ledger.evidence() {
        for entry in dla_core::holders::entries_owned_by(rec, owner) {
            let lines = dla_core::holders::entry_sites(entry);
            if lines.contains(&site) {
                return None;
            }
            for l in lines {
                if !held.iter().any(|h| h == l) {
                    held.push(l.to_owned());
                }
            }
        }
    }
    if held.is_empty() {
        return None;
    }
    let shown: Vec<&str> = held.iter().take(3).map(String::as_str).collect();
    Some(format!(
        "--site {site} is none of the lines the holder {owner} keeps in the retention record ({}{}); a finding at one of them has a measured column of its own: dla ledger add ... --site <one of them> --owner {owner}",
        shown.join(", "),
        if held.len() > 3 { ", ..." } else { "" }
    ))
}

/// A warning when `owner` names no holder in the growth-series or retention-path evidence
/// the command cites (for an annotation without `--evidence`, the evidence the finding
/// cites). Such an owner ties no heap-graph column or retention record to the finding.
/// The warning lists up to three holder owners those records name, in the form that ties.
/// Nothing is said when the cited records name no holder at all (nothing to compare).
fn owner_warning(
    ledger: &Ledger,
    owner: Option<&str>,
    cited: &[EvidenceId],
    finding: Option<&FindingId>,
) -> Option<String> {
    let owner = owner?;
    let mut ids: Vec<EvidenceId> = cited.to_vec();
    if ids.is_empty()
        && let Some(f) = finding.and_then(|f| ledger.finding(f))
    {
        ids = f.supports.clone();
    }
    let mut named: Vec<String> = Vec::new();
    let mut seen: Vec<&EvidenceId> = Vec::new();
    for id in &ids {
        let Some(rec) = ledger.evidence_by_id(&ledger.own_evidence_id(id)) else {
            continue;
        };
        let owners = dla_core::holders::record_holder_owners(rec);
        if owners.is_empty() {
            continue;
        }
        if dla_core::holders::record_names_owner(rec, owner) {
            return None;
        }
        seen.push(id);
        for o in owners {
            if !named.contains(&o) {
                named.push(o);
            }
        }
    }
    if named.is_empty() {
        return None;
    }
    let ambiguous = ids.iter().any(|id| {
        ledger
            .evidence_by_id(&ledger.own_evidence_id(id))
            .is_some_and(|r| {
                matches!(
                    dla_core::holders::owner_naming(r, owner),
                    dla_core::holders::OwnerNaming::Ambiguous(_)
                )
            })
    });
    Some(format!(
        "--owner {owner} {} {}; a heap-graph column or retention record ties to a finding only by the owner of its holder, which these records name as {}{}. An owner is set once: if it is wrong, add a finding with one of them at the same site (dla ledger add --origin growth-series --evidence <one of {}> --site <the same file:line> --owner <one of them>) and merge this one into it (dla ledger duplicate <this finding> --of <new>)",
        if ambiguous {
            "is a short form of several holders in"
        } else {
            "names no holder in"
        },
        seen.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        named.iter().take(3).cloned().collect::<Vec<_>>().join(", "),
        if named.len() > 3 { ", ..." } else { "" },
        seen.iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", "),
    ))
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
    warnings.extend(owner_warning(
        &ledger,
        a.owner.as_deref(),
        &a.evidence,
        Some(&a_finding),
    ));
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
    if to == FindingState::Confirmed {
        refuse_registration_only(ctx, &finding, &evidence)?;
        ledger_release::refuse_held_confirm(ctx, &finding)?;
    }
    if matches!(to, FindingState::Confirmed | FindingState::Refuted) {
        refuse_unrelated_alloc_test(ctx, &finding, &evidence)?;
    }
    let note = ledger_release::judge_release_move(ctx, &finding, &evidence, to)?;
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
    if let Some(n) = note {
        out["note"] = json!(n);
    }
    Ok(out)
}

/// Refuses to cite, for `confirm` or `refute`, the alloc-test evidence of a harness that is not
/// about the finding ([`dla_core::grown::harness_about`]). `dla alloc-test <H> --finding <F>`
/// already refuses to write such evidence for a named finding; what remains is the evidence of an
/// alloc-test run without `--finding` (it names no finding, so the ledger's family check lets it
/// through) and evidence written before that rule.
fn refuse_unrelated_alloc_test(ctx: &Ctx, id: &FindingId, evidence: &[EvidenceId]) -> Res<()> {
    let Ok(view) = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref()) else {
        return Ok(());
    };
    let Some(finding) = view.ledger.finding(id) else {
        return Ok(());
    };
    for e in view
        .ledger
        .evidence()
        .filter(|e| evidence.contains(&e.evidence_id))
    {
        if e.kind != dla_core::evidence::EvidenceKind::Discriminator
            || e.result.get("test").and_then(Value::as_str) != Some("plateau+purge")
        {
            continue;
        }
        let Some(h) = e.result.get("harness").and_then(Value::as_str) else {
            continue;
        };
        let Ok(manifest) = dla_core::harness::HarnessManifest::load(&ctx.run_dir, h) else {
            continue;
        };
        if dla_core::grown::harness_about(&view, &manifest, &finding.root_id()).is_none() {
            return Err(crate::alloc_cmd::harness_not_about(
                &view,
                h,
                manifest.runtime,
                finding,
                &format!("the alloc-test evidence {} of it", e.evidence_id),
            ));
        }
    }
    Ok(())
}

/// The harness and the function when the trigger of the leak-side finding `id` is `direct`:
/// the only growth measured comes from a unit that calls the registration function itself.
pub(crate) fn direct_trigger(
    run_dir: &std::path::Path,
    worker: Option<&str>,
    id: &FindingId,
) -> Option<(String, String, String)> {
    use dla_core::trigger::{Trigger, Triggers};
    let view = RunView::load_for(run_dir, worker).ok()?;
    let finding = view.ledger.finding(id)?;
    if finding.id.is_sibling() || finding.polarity() != dla_core::finding::Polarity::Leak {
        return None;
    }
    match Triggers::of(&view).trigger(id) {
        Trigger::Direct {
            harness, writer, ..
        } => {
            let rt =
                gate::finding_runtime(&view, finding).unwrap_or_else(|| "<runtime>".to_owned());
            Some((harness, writer, rt))
        }
        _ => None,
    }
}

/// Refuses to confirm a leak-side finding whose trigger is `direct`: the census reads every
/// writer of the holder as a start-up registration and the only growth measured comes from a
/// unit that calls the registration function itself with new keys.
fn refuse_registration_only(ctx: &Ctx, id: &FindingId, evidence: &[EvidenceId]) -> Res<()> {
    let Some((harness, writer, rt)) = direct_trigger(&ctx.run_dir, ctx.worker.as_deref(), id)
    else {
        return Ok(());
    };
    let Ok(view) = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref()) else {
        return Ok(());
    };
    let mut fns: Vec<String> = view
        .ledger
        .annotation(id)
        .map(|a| a.facts.writer_sites.as_slice())
        .unwrap_or_default()
        .iter()
        .filter_map(|s| s.function.clone())
        .collect();
    fns.sort();
    fns.dedup();
    let writers = if fns.is_empty() {
        writer.clone()
    } else {
        fns.join(", ")
    };
    let ev = evidence
        .first()
        .map_or_else(|| "<growth evidence>".to_owned(), ToString::to_string);
    Err(Failure::refuse(
        core_codes::REGISTRATION_ONLY,
        format!(
            "{id} cannot be confirmed as a leak on this evidence: the census reads every writer of the holder as a start-up registration ({writers}), and the only growth measured ({harness}) comes from a unit that calls {writer} itself with new keys, which is growth by design."
        ),
        format!(
            "growth that a caller of the public API causes without calling the registration function {writer}"
        ),
        format!(
            "one route decides both readings: an entry-path harness whose unit uses the public API without calling {writer}: dla harness gen --lang {rt} --driver --entry {id}, fill the hooks and VARIES.txt, dla harness run <H> --mode both, dla trend <S>, dla alloc-test <H> --finding {id}; if the table grows there, dla ledger confirm {id} again; if it stays flat, that alloc-test favors non-leak: dla ledger refute {id} --evidence <the alloc-test evidence>, and to record the reading dla ledger add --sibling-of {id} --origin hypothesis --hypothesis N-INTENT --refute-if \"run-time input reaches the registration call\" --evidence {ev}, dla ledger test <the sibling id the add prints> --evidence <harness evidence>, dla ledger confirm <that id> --evidence <the alloc-test evidence>"
        ),
    ))
}

/// Runs `dla ledger resolve`.
pub fn resolve(ctx: &Ctx, a: ResolveArgs) -> Res<Value> {
    let target = classify(&a.target)?;
    let mut ledger = open(ctx)?;
    match target {
        Target::Finding(id) => resolve_finding(ctx, &mut ledger, id, a, None),
        Target::OpenItem(id) => resolve_open_item(ctx, &mut ledger, id, a),
        Target::Checklist(item) => resolve_checklist(ctx, &mut ledger, item, a),
    }
}

/// What `dla ledger close-rest` loads once and every refusal of [`resolve_finding`] reads in
/// place of loading the run again for each finding: the run view as it was before the first
/// close, its ranking and its queues. Closing a finding moves it from `candidate` to
/// `unresolved` or `suspected_static_only`, which keeps it in the ranked set and leaves every
/// other finding's runtime, rank and pending standing as it was, so the refusals read the
/// same facts they would read from disk.
pub(crate) struct Loaded<'a> {
    /// The run as loaded before the first close.
    pub view: &'a RunView,
    /// [`dla_core::queue::ranked`] of the view.
    pub ranked: &'a [dla_core::queue::Ranked],
    /// [`dla_core::queue::queues`] of the view.
    pub queues: &'a [dla_core::queue::RuntimeQueue],
    /// [`dla_core::grown::grown_unmeasured`] of the view.
    pub grown: &'a [dla_core::grown::Grown],
}

pub(crate) fn resolve_finding(
    ctx: &Ctx,
    ledger: &mut Ledger,
    id: FindingId,
    a: ResolveArgs,
    loaded: Option<&Loaded<'_>>,
) -> Res<Value> {
    let id = ledger.own_finding_id(&id);
    reject(
        "a finding",
        &[
            ("--status", a.status.is_some()),
            ("--by", a.by.is_some()),
            ("--note", a.note.is_some()),
            ("--runtime", a.runtime.is_some()),
        ],
    )?;
    let to = match a.disposition {
        Some(ResolveAs::Unresolved) => FindingState::Unresolved,
        Some(ResolveAs::SuspectedStaticOnly) => FindingState::SuspectedStaticOnly,
        None => {
            return Err(missing_arg(
                "--as",
                format!(
                    "dla ledger resolve {id} --as unresolved --reason {} --remedy \"<text>\"",
                    ledger.supported_reason(&id)
                ),
            ));
        }
    };
    let from = ledger.finding(&id).map(|f| f.state);
    // A finding a passed growth column points at is decided by an alloc-test, not closed
    // unmeasured; the floor rule (`time_budget`) and an aborted run keep their own.
    if a.reason != Some(UnresolvedReason::TimeBudget) {
        refuse_grown_unmeasured(ctx, &id, loaded)?;
    }
    // A finding whose newest alloc-test favors leak is confirmed or refuted, never closed.
    refuse_leak_favored(ctx, &id, a.reason, loaded)?;
    if let Some(reason) = a.reason {
        refuse_target_pending(ctx, &id, reason, loaded)?;
        refuse_unrun_harness(ctx, &id, reason, loaded)?;
        refuse_no_harness(ctx, &id, reason, loaded)?;
        if reason == UnresolvedReason::TimeBudget {
            // After the ledger's own checks, so a missing remedy or an unknown finding is
            // reported as such, and before anything is written.
            ledger.check_transition_only(
                &id,
                to,
                a.reason,
                a.remedy.clone(),
                a.evidence.clone(),
            )?;
            refuse_time_left(
                ctx,
                &id.to_string(),
                ledger.finding_added_at(&id),
                ledger.supported_reason(&id),
                to == FindingState::SuspectedStaticOnly,
            )?;
        }
    }
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

/// Refuses to close a ranked candidate with `no_harness`, `not_exercised` or `time_budget`
/// while its runtime's measurement queue is open and no workload has settled it: the fix is
/// the batch command that targets the next candidates.
fn refuse_target_pending(
    ctx: &Ctx,
    id: &FindingId,
    reason: UnresolvedReason,
    loaded: Option<&Loaded<'_>>,
) -> Res<()> {
    if !gate::closes_without_exercise(reason) {
        return Ok(());
    }
    let own;
    let view = match loaded {
        Some(l) => l.view,
        None => {
            own = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
            &own
        }
    };
    if !view.state.queue {
        return Ok(());
    }
    let own_ranked;
    let ranked: &[dla_core::queue::Ranked] = match loaded {
        Some(l) => l.ranked,
        None => {
            own_ranked = dla_core::queue::ranked(view);
            &own_ranked
        }
    };
    let Some(r) = ranked
        .iter()
        .find(|r| &r.id == id && r.state == FindingState::Candidate)
    else {
        return Ok(());
    };
    let Some(runtime) = r.runtime.clone() else {
        return Ok(());
    };
    let own_queue;
    let q = match loaded {
        Some(l) => l.queues.iter().find(|q| q.runtime == runtime && q.open()),
        None => {
            own_queue = dla_core::queue::open_queue(view, &runtime, ctx.worker.as_deref());
            own_queue.as_ref()
        }
    };
    let Some(q) = q else {
        return Ok(());
    };
    let Some((rank, tier)) = q.rank_of(id) else {
        return Ok(());
    };
    let (left, total) = q.minutes.unwrap_or((0, 0));
    // A candidate a harness missed waits for a harness of its own (a focus harness).
    if let Some(m) = q.missed.get(id) {
        return Err(Failure::refuse(
            core_codes::TARGET_PENDING,
            format!(
                "{id} cannot be closed with {reason}: it is ranked {rank} (tier {tier}) among the {runtime} candidates; {} did not reach it ({} {}), and it is given up only after a focus harness (at most 3 targets) missed it too; {left} of {total} budget minutes remain.",
                m.harness,
                m.verdict.as_str(),
                m.detail
            ),
            format!(
                "a focus harness for {id} (dla harness gen --next gives the best-ranked missed candidates a harness of their own), or an unresolved reason that says what it needs (env_unavailable, tool_missing, build_fails, privilege_declined)"
            ),
            format!(
                "{} (the focus batch: the best-ranked pending candidate was missed), fill the hooks so each unit calls the writer function of each target with a new key, dla harness run <H> --mode both, dla trend S<n>; or dla ledger resolve {id} --as unresolved --reason env_unavailable --remedy \"<what it needs>\"; or, when no driver can reach the {runtime} candidates at all, dla ledger resolve intake.workload --status unresolved --reason <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy \"<text>\" --runtime {runtime} (that closes the queue of {runtime} only){}",
                q.missed_command(id),
                if m.verdict == dla_core::trend::Verdict::Unknown {
                    format!(". {id} read `unknown`: {}", dla_core::queue::NO_COVERAGE)
                } else {
                    String::new()
                }
            ),
        ));
    }
    Err(Failure::refuse(
        core_codes::TARGET_PENDING,
        format!(
            "{id} cannot be closed with {reason}: it is ranked {rank} (tier {tier}) among the {runtime} candidates and no workload has reached it; {left} of {total} budget minutes remain."
        ),
        format!(
            "a workload whose units reach {id} (a harness that targets it, then dla trend), or the blocker of the {runtime} workload recorded"
        ),
        dla_core::queue::steps(q, &view.state.run_id),
    ))
}

/// Refuses `--reason time_budget` while more time is left than the shared floor,
/// `max(10 min, 20 % of the budget)` (the worker's own deadline in a worker), unless a tool
/// refused a command with nothing smaller that fits after `added` (see
/// [`dla_core::toolctx::nothing_fits_since`]); a refusal whose fix names a smaller command does
/// not count, since the smaller command can still be run. The reason says the budget ran out;
/// with an hour left it is not true, and the finding is told the reason the evidence supports.
fn refuse_time_left(
    ctx: &Ctx,
    what: &str,
    added: Option<chrono::DateTime<chrono::Utc>>,
    supported: UnresolvedReason,
    static_only: bool,
) -> Res<()> {
    // Only the state: in a worker the slice ledger is open (and locked) by the caller.
    let state = RunState::load(&ctx.run_dir)?;
    let Some((left, total)) = gate::budget_over_floor(&state, ctx.worker.as_deref()) else {
        return Ok(());
    };
    let since = added.unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC);
    if dla_core::toolctx::nothing_fits_since(&ctx.run_dir, ctx.worker.as_deref(), since) {
        return Ok(());
    }
    let floor =
        (total * gate::HARNESS_MIN_LEFT_PERCENT / 100).max(gate::HARNESS_MIN_LEFT_SECS / 60);
    let whose = if ctx.worker.is_some() {
        "of this worker"
    } else {
        "of the core"
    };
    Err(Failure::refuse(
        core_codes::TIME_LEFT,
        format!(
            "{what} cannot be closed with time_budget: {left} of {total} budget minutes remain (the reason holds below {floor}) and no command {whose} was refused with nothing smaller that fits since it was added."
        ),
        format!(
            "less than {floor} minutes left, or a command refused with nothing smaller that fits"
        ),
        if static_only {
            format!(
                "a finding that rests on static evidence was never exercised: dla ledger resolve {what} --as suspected_static_only --reason not_exercised --remedy \"<text>\" --evidence <static-hit or census-holder id>; or measure it: the run has time"
            )
        } else {
            format!(
                "use the reason the evidence supports: dla ledger resolve {what} --as unresolved --reason {supported} --remedy \"<text>\"   (the reason is inconclusive when a measurement is tied to it, else not_exercised); or measure it: the run has time"
            )
        },
    ))
}

/// True when the run has a `harness/H<n>` directory. Without one the rule cannot hold, and
/// the run view (every series file) is not loaded for each closure.
fn has_harness_dir(run_dir: &Path) -> bool {
    std::fs::read_dir(run_dir.join(dla_core::harness::HARNESS_DIR)).is_ok_and(|rd| {
        rd.flatten()
            .any(|e| dla_core::series::numbered('H', &e.file_name().to_string_lossy()).is_some())
    })
}

/// Refuses to close `id` without exercising it (`no_harness`, `not_exercised`,
/// `time_budget`) while a harness of its runtime was generated and never run and the
/// budget has room to run it. The same rule feeds the final gate.
fn refuse_unrun_harness(
    ctx: &Ctx,
    id: &FindingId,
    reason: UnresolvedReason,
    loaded: Option<&Loaded<'_>>,
) -> Res<()> {
    if !gate::closes_without_exercise(reason) || !has_harness_dir(&ctx.run_dir) {
        return Ok(());
    }
    let own;
    let view = match loaded {
        Some(l) => l.view,
        None => {
            own = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
            &own
        }
    };
    let Some(finding) = view.ledger.finding(id) else {
        return Ok(());
    };
    let Some(runtime) = gate::finding_runtime(view, finding) else {
        return Ok(());
    };
    let Some(u) = gate::harness_not_run(view, &runtime, ctx.worker.as_deref()) else {
        return Ok(());
    };
    Err(Failure::refuse(
        core_codes::HARNESS_NOT_RUN,
        format!(
            "{id} cannot be closed with {reason}: harness {} ({}) {}, and {} of {} budget minutes remain.",
            u.id,
            u.runtime,
            u.state(),
            u.minutes_left,
            u.minutes_total
        ),
        if u.test_loop && !u.build_failed {
            format!("{} run to its end", u.id)
        } else {
            format!(
                "{} {}, or its blocker recorded on intake.workload with --runtime {}",
                u.id,
                if u.build_failed {
                    "built and run"
                } else {
                    "run"
                },
                u.runtime
            )
        },
        u.fix(&view.state.run_id),
    ))
}

/// Refuses to close `id` without exercising it (`no_harness`, `not_exercised`,
/// `time_budget`) when its runtime has discovered entry points and no harness was ever
/// generated, while the budget has room to make one. The blocker reasons
/// (`env_unavailable`, `build_fails`, `tool_missing`, `privilege_declined`) and a blocker
/// recorded on `intake.workload` are never refused. The same rule feeds the final gate
/// (`budget_left_unrevisited`).
fn refuse_no_harness(
    ctx: &Ctx,
    id: &FindingId,
    reason: UnresolvedReason,
    loaded: Option<&Loaded<'_>>,
) -> Res<()> {
    if !gate::closes_without_exercise(reason) {
        return Ok(());
    }
    // The cheap parts first: the closure of a finding is the common call, and a run with no
    // discovered entry points, or with its budget floor passed, needs neither the ledger
    // nor the series.
    let state = RunState::load(&ctx.run_dir)?;
    if gate::budget_over_floor(&state, ctx.worker.as_deref()).is_none() {
        return Ok(());
    }
    let own;
    let view = match loaded {
        Some(l) => {
            if !l.view.coverage.as_ref().is_some_and(|c| {
                c.workers
                    .values()
                    .any(|w| !w.entry_points.discovered.is_empty())
            }) {
                return Ok(());
            }
            l.view
        }
        None => {
            let Some(cov) = dla_core::coverage::CoverageManifest::load(&ctx.run_dir)? else {
                return Ok(());
            };
            if !cov
                .workers
                .values()
                .any(|w| !w.entry_points.discovered.is_empty())
            {
                return Ok(());
            }
            let ledger = Ledger::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
            let mut v = RunView::new(state, ledger, Some(cov));
            v.run_dir = Some(ctx.run_dir.clone());
            own = v;
            &own
        }
    };
    let Some(finding) = view.ledger.finding(id) else {
        return Ok(());
    };
    let Some(runtime) = gate::finding_runtime(view, finding) else {
        return Ok(());
    };
    let Some(n) = gate::harness_not_generated(view, &runtime, ctx.worker.as_deref()) else {
        return Ok(());
    };
    Err(Failure::refuse(
        core_codes::HARNESS_NOT_GENERATED,
        format!(
            "{id} cannot be closed with {reason}: {} entry point(s) of {} were discovered, no harness was generated, and {} of {} budget minutes remain.",
            n.entry_points, n.runtime, n.minutes_left, n.minutes_total
        ),
        format!(
            "a harness for {} (dla harness gen), or its blocker recorded",
            n.runtime
        ),
        n.fix(),
    ))
}

fn resolve_open_item(ctx: &Ctx, ledger: &mut Ledger, id: OpenItemId, a: ResolveArgs) -> Res<Value> {
    let id = ledger.own_open_item_id(&id);
    reject(
        "an open item",
        &[
            ("--status", a.status.is_some()),
            ("--note", a.note.is_some()),
            ("--evidence", !a.evidence.is_empty()),
            ("--runtime", a.runtime.is_some()),
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
                        "dla ledger resolve {id} --as unresolved --reason {} --remedy \"<text>\"",
                        ledger.open_item_supported_reason(&id)
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
                    "dla ledger resolve {id} --as unresolved --reason {} --remedy \"<text>\"",
                    ledger.open_item_supported_reason(&id)
                ),
            ));
        }
        (None, None) => {
            return Err(missing_arg(
                "--as or --by",
                format!(
                    "dla ledger resolve {id} --as unresolved --reason {} --remedy \"<text>\"   or   dla ledger resolve {id} --by F7",
                    ledger.open_item_supported_reason(&id)
                ),
            ));
        }
    };
    if let Resolution::Unresolved {
        reason: UnresolvedReason::TimeBudget,
        ..
    } = &resolution
    {
        ledger.check_open_item_resolution(&id, resolution.clone())?;
        refuse_time_left(
            ctx,
            &id.to_string(),
            ledger.open_item_added_at(&id),
            ledger.open_item_supported_reason(&id),
            false,
        )?;
    }
    if let Resolution::Unresolved { reason, .. } = &resolution
        && *reason != UnresolvedReason::TimeBudget
    {
        refuse_uncaptured_residual(ctx, &id)?;
    }
    let line = ledger.resolve_open_item(&id, resolution)?;
    Ok(json!({
        "ok": true,
        "id": id.to_string(),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    }))
}

/// Refuses to leave the open item of a heap residual above its threshold unresolved while the
/// largest row's series has no summarized capture written after it (see
/// [`gate::uncaptured_residuals`]) and its runtime takes `--capture`: the
/// capture and its `dla summarize` commands say who keeps the growth. An aborted run is not held
/// to it, and `time_budget` keeps its own rule.
fn refuse_uncaptured_residual(ctx: &Ctx, id: &OpenItemId) -> Res<()> {
    let view = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
    if view.state.status == dla_core::state::RunStatus::Aborted {
        return Ok(());
    }
    let Some(u) = gate::uncaptured_residual_of(&view, &id.to_string()) else {
        return Ok(());
    };
    Err(Failure::refuse(
        core_codes::CAPTURE_NOT_SUMMARIZED,
        format!(
            "{id} cannot be left unresolved: it is a heap residual above its threshold and no summarized capture of its largest row ({}) exists: no retention or allocation-stack summary computed from capture files under harness/{}/ was written after the first series of the driver that wrote it (a capture taken before it, or of an earlier driver, says nothing of it; a capture run that records it could not write stacks releases it)",
            u.series.join(", "),
            u.harness
        ),
        format!(
            "a capture of {} taken for {} and its summaries (who keeps the growth), then the item resolved by what they show",
            u.harness,
            u.series.join(", ")
        ),
        u.fix(),
    ))
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
    if a.runtime.is_some() && (item != "intake.workload" || status != AnswerStatus::Unresolved) {
        return Err(Failure::refuse(
            codes::UNSUPPORTED_FLAG,
            format!(
                "--runtime does not apply to {item} with --status {status}: only intake.workload with --status unresolved names a runtime",
                status = status.as_str()
            ),
            "--runtime only on `intake.workload --status unresolved`",
            "dla ledger resolve intake.workload --status unresolved --reason <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy \"<text>\" --runtime <runtime>",
        ));
    }
    let runtime = workload_blocker_runtime(ctx, &item, status, a.reason, a.runtime.as_deref())?;
    refuse_contradicted_workload_blocker(ctx, &item, status, a.reason, runtime.as_deref())?;
    refuse_unfinished_test_loop(ctx, &item, status, a.reason, runtime.as_deref())?;
    let line = ledger.answer_checklist(ChecklistAnswer {
        item: item.clone(),
        status,
        evidence: a.evidence,
        note: a.note,
        reason: a.reason,
        remedy: a.remedy,
        runtime,
    })?;
    Ok(json!({
        "ok": true,
        "id": item,
        "status": status.as_str(),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    }))
}

/// Refuses to close `id` (`unresolved` or `suspected_static_only`) while a passed growth
/// column points at it and no alloc-test decided it (one that favors non-leak, or one on a driver
/// made for the finding; a leak or inconclusive result on a test loop or the project's own
/// workload does not): the fix is the commands that decide it. An aborted run is not held to it.
fn refuse_grown_unmeasured(ctx: &Ctx, id: &FindingId, loaded: Option<&Loaded<'_>>) -> Res<()> {
    let own;
    let own_grown;
    let (view, grown): (&RunView, &[dla_core::grown::Grown]) = match loaded {
        Some(l) => (l.view, l.grown),
        None => {
            own = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
            own_grown = dla_core::grown::grown_unmeasured(&own);
            (&own, &own_grown)
        }
    };
    if view.state.status == dla_core::state::RunStatus::Aborted {
        return Ok(());
    }
    let Some(g) = grown.iter().find(|g| &g.finding == id) else {
        return Ok(());
    };
    let (message, wanted) = if g.dump.is_some() {
        (
            format!(
                "{id} cannot be closed: {}, and nothing decided it since",
                g.grew_text()
            ),
            format!(
                "{id} decided by the result of that dump series: its non-leak sibling refuted and the finding confirmed (growth measured is decided, not closed)"
            ),
        )
    } else if let Some(l) = &g.longer {
        (
            format!("{id} cannot be closed: {}, and {}", g.grew_text(), l.text()),
            format!(
                "the alloc-test of {id} run with --extra {} (the window it named), or the budget spent down to its floor (growth measured is decided, not closed)",
                l.needed
            ),
        )
    } else if let Some(sh) = &g.shared {
        (
            format!(
                "{id} cannot be closed: {}, and the alloc-test {} on {} ({}, not a driver made for {id}) favored {}: growth there may be kept by another holder",
                g.grew_text(),
                sh.evidence,
                sh.harness,
                sh.why,
                sh.favors
            ),
            format!(
                "an alloc-test of {id} on a driver made for it (growth measured is decided, not closed)"
            ),
        )
    } else if g.pytest_loop {
        (
            format!(
                "{id} cannot be closed: {}, and no alloc-test is bound to it",
                g.grew_text()
            ),
            format!(
                "an alloc-test of {id} on a harness of its own, not on {} (the loop re-runs the whole test suite per run); growth measured is decided, not closed",
                g.harness
            ),
        )
    } else {
        (
            format!(
                "{id} cannot be closed: {}, and no alloc-test is bound to it",
                g.grew_text()
            ),
            format!(
                "an alloc-test of {id} on {} (growth measured is decided, not closed)",
                g.harness
            ),
        )
    };
    Err(Failure::refuse(
        core_codes::GROWTH_NOT_MEASURED,
        message,
        wanted,
        g.commands().join("   then   "),
    ))
}

/// Refuses to close a finding, or a sibling of one, that has a route to confirm
/// ([`dla_core::grown::route_to_confirm`]): an alloc-test on a harness about it favors leak and
/// nothing newer says otherwise. The fix is the route, with the ids of this run. `time_budget` is
/// held to it while the budget is over the floor (the rule `time-left` has for the reason); an
/// aborted run is not held to it.
fn refuse_leak_favored(
    ctx: &Ctx,
    id: &FindingId,
    reason: Option<UnresolvedReason>,
    loaded: Option<&Loaded<'_>>,
) -> Res<()> {
    let own;
    let view: &RunView = match loaded {
        Some(l) => l.view,
        None => {
            own = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
            &own
        }
    };
    if view.state.status == dla_core::state::RunStatus::Aborted {
        return Ok(());
    }
    if reason == Some(UnresolvedReason::TimeBudget)
        && gate::budget_over_floor(&view.state, ctx.worker.as_deref()).is_none()
    {
        return Ok(());
    }
    let Some(route) = dla_core::grown::route_for(view, id) else {
        return Ok(());
    };
    Err(Failure::refuse(
        core_codes::LEAK_FAVORED,
        format!("{}.", route.refusal_message(id)),
        format!(
            "{} confirmed (its non-leak siblings refuted by {}), or refuted by a newer alloc-test that favors non-leak",
            route.finding, route.discriminator
        ),
        route.fix(),
    ))
}

/// The runtime a workload answer is stored with. For `intake.workload` as `unresolved`:
/// `--runtime` must name a runtime of the run (one with ranked candidates or one the probe
/// detected). With a blocker reason (`tool_missing`, `env_unavailable`, `build_fails`,
/// `privilege_declined`: the ones that close a measurement queue) the runtime is required
/// when more than one runtime has ranked candidates, implied when exactly one has, and left
/// out when none has. Any other answer returns `--runtime` as given.
fn workload_blocker_runtime(
    ctx: &Ctx,
    item: &str,
    status: AnswerStatus,
    reason: Option<UnresolvedReason>,
    given: Option<&str>,
) -> Res<Option<String>> {
    if item != "intake.workload" || status != AnswerStatus::Unresolved {
        return Ok(None);
    }
    let blocker = reason.is_some_and(gate::is_blocker_reason);
    if given.is_none() && !blocker {
        return Ok(None);
    }
    let view = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
    let queues = dla_core::queue::queues(&view, ctx.worker.as_deref());
    let with_candidates: Vec<String> = queues.iter().map(|q| q.runtime.clone()).collect();
    let mut valid: std::collections::BTreeSet<String> = with_candidates.iter().cloned().collect();
    if let Some(c) = &view.coverage {
        valid.extend(c.workers.keys().cloned());
    }
    let pending = |rt: &str| {
        queues
            .iter()
            .find(|q| q.runtime == rt)
            .map_or(0, |q| q.pending.len())
    };
    let list = |names: &mut dyn Iterator<Item = &String>| {
        names
            .map(|r| format!("{r} ({} pending)", pending(r)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let reason_name = reason.map_or("<reason>".to_owned(), |r| r.to_string());
    let fix_for = |rt: &str| {
        format!(
            "dla ledger resolve intake.workload --status unresolved --reason {reason_name} --remedy \"<text>\" --runtime {rt}"
        )
    };
    if let Some(rt) = given {
        if !valid.contains(rt) {
            let names = valid.iter().cloned().collect::<Vec<_>>().join(", ");
            return Err(Failure::refuse(
                core_codes::INVALID_VALUE,
                format!("--runtime {rt} is not a runtime of this run"),
                format!("one of: {names}"),
                fix_for(valid.iter().next().map_or("<runtime>", String::as_str)),
            ));
        }
        return Ok(Some(rt.to_owned()));
    }
    match with_candidates.as_slice() {
        [] => Ok(None),
        [one] => Ok(Some(one.clone())),
        many => Err(Failure::refuse(
            codes::MISSING_ARGUMENT,
            format!(
                "--runtime is required: a blocker closes the queue of one runtime, and {} runtimes have ranked candidates ({})",
                many.len(),
                list(&mut many.iter())
            ),
            "--runtime <the runtime the blocker is about>",
            fix_for(&many[0]),
        )),
    }
}

/// Refuses `intake.workload` as `unresolved` with a blocker reason for `runtime` when the
/// run's own records contradict it: a harness of that runtime ran and wrote a series. A probe
/// that reads flat is a verdict about one target, not about the workload. A blocker for a
/// runtime whose harnesses wrote no series is accepted whatever the other runtimes did.
fn refuse_contradicted_workload_blocker(
    ctx: &Ctx,
    item: &str,
    status: AnswerStatus,
    reason: Option<UnresolvedReason>,
    runtime: Option<&str>,
) -> Res<()> {
    let Some(runtime) = runtime else {
        return Ok(());
    };
    if item != "intake.workload"
        || status != AnswerStatus::Unresolved
        || !reason.is_some_and(gate::is_blocker_reason)
    {
        return Ok(());
    }
    let view = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
    let Some((harness, series)) = gate::workload_ran(&view, runtime) else {
        return Ok(());
    };
    let reason = reason.map_or(String::new(), |r| r.to_string());
    Err(Failure::refuse(
        core_codes::WORKLOAD_RAN,
        format!(
            "intake.workload cannot be recorded as {reason} for {runtime}: harness {harness} ran and wrote series {series}, so a {runtime} workload can be run"
        ),
        format!(
            "a blocker that the run's records do not contradict (no harness of {runtime} wrote a series), not {reason}"
        ),
        format!(
            "a probe that reads flat is a verdict about one target, not about the workload: read dla trend {series} (its targets and next lines), give the target a unit that reaches it with a new key or a size reading, and run {harness} again; or measure the findings a passed column points at (dla ledger list, dla gate check --final)"
        ),
    ))
}

/// Refuses `intake.workload` as `unresolved` with a blocker reason for `runtime` (every runtime
/// when the answer names none) while that runtime has a generated test loop whose run did not
/// reach an end. A test loop needs nothing written, so "no harness can be made" cannot hold for
/// it, and a run that never started or was cut before its first result shows nothing about the
/// environment. A loop with a series (`workload-ran`) or a recorded failed attempt is judged by
/// the other rules.
fn refuse_unfinished_test_loop(
    ctx: &Ctx,
    item: &str,
    status: AnswerStatus,
    reason: Option<UnresolvedReason>,
    runtime: Option<&str>,
) -> Res<()> {
    if item != "intake.workload"
        || status != AnswerStatus::Unresolved
        || !reason.is_some_and(gate::is_blocker_reason)
    {
        return Ok(());
    }
    let view = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
    let Some((harness, rt)) = gate::unfinished_test_loop(&view, runtime) else {
        return Ok(());
    };
    let reason = reason.map_or(String::new(), |r| r.to_string());
    Err(Failure::refuse(
        core_codes::LOOP_NOT_FINISHED,
        format!(
            "intake.workload cannot be recorded as {reason} for {rt}: the test loop {harness} of {rt} needs nothing written, and its run did not reach an end (never started, or cut before its first result), so nothing shows yet that the environment cannot run the workload"
        ),
        format!("{harness} run to its end, then the blocker if nothing can run, not {reason}"),
        format!(
            "dla harness run {harness} as a plain foreground command with the tool's longest timeout, and wait for its end (it stops by itself within its time limit); a run that fails records why and the queue goes on with the driver batches; then, if nothing can run, record the blocker"
        ),
    ))
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
    // A verifier often names the line that adds to the holder; the finding records the
    // declaration and the writers the census found. A site in the same function as one of
    // those is the same place.
    let same_place: Vec<String> = match (&site, ctx.run_dir.parent().and_then(Path::parent)) {
        (Some(v), Some(project)) => {
            let own = ledger.finding(&a.finding).and_then(|f| f.site.clone());
            let noted = ledger.annotation(&a.finding).map_or_else(Vec::new, |n| {
                n.writers.iter().chain(&n.removers).cloned().collect()
            });
            own.iter()
                .chain(noted.iter())
                .any(|k| crate::trend_cmd::verdicts::same_function(project, k, v))
                .then(|| v.clone())
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    };
    let (line, item) = ledger.verify_among(
        &a.finding,
        site,
        a.holder,
        &a.mechanism,
        &a.label,
        &a.bundle,
        dla_core::labels::Archetypes::from_table(table.as_ref()),
        &same_place,
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
