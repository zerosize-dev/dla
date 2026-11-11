//! Archetype labels: the rules behind `dla ledger label` and the replay check of a
//! `finding_labeled` line.
//!
//! [`plan`] runs on the writing side. It reads the archetype entry (the table is passed
//! in; replay never reads `data/`), picks the first `requires` alternative the finding's
//! evidence meets, and builds the record with the proof it was accepted on. [`check`]
//! runs at replay: it re-checks only what the record carries (the finding's state,
//! polarity, the cited evidence, the kinds, the tie of each record to the finding), so a
//! plugin update that edits a table row cannot make an old ledger fail to replay.
//!
//! One tie rule decides whether a record counts for a finding:
//!
//! * growth kinds: a passing growth record with a passing column tied to the finding by
//!   its `finding`, an equal site, `candidates` or a probe named after its owner;
//! * retention and allocation kinds: [`names_holder`], or the record's `finding` /
//!   `candidates` name the finding;
//! * a discriminator: its `finding` names the finding; for an `N-*` label it must favor
//!   non-leak, name no finding outside the family and not support another `N-*` only;
//! * an intervention: it tested the finding, removed the growth and has no false guard;
//! * kinds whose result names no finding (census, static hit, histogram, non-heap,
//!   should-be-dead, residual): `finding`/`candidates` in the result, or citation by the
//!   finding itself (its origin and the moves up to its confirmation). A sibling's or the
//!   root's citation does not make a record count: the family is the default pool of
//!   candidates for `--evidence`, not a tie.
//!
//! Never "the same run". A record of authority `static` counts only as `static-hit` or
//! `census-holder`.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    archetypes::{Entry, Table},
    detectors::ARCHETYPE_IDS,
    discriminator::{Favors, favors_of, finding_of},
    error::Refusal,
    evidence::{Authority, EvidenceKind, EvidenceRecord},
    finding::{Finding, FindingState, Holder, Polarity, UNCLASSIFIED, nonleak_id},
    holders::{InterventionResult, names_holder},
    ids::{EvidenceId, FindingId},
    m6::codes,
    machine::{EvidenceLookup, FindingLookup, is_passing_growth},
    record::{FindingLabeled, LabelSatisfied},
    trend::GrowthSummary,
};

/// Whether archetype labels are available in this process.
#[derive(Debug, Clone, Copy)]
pub enum Archetypes<'a> {
    /// `DLA_ARCHETYPES=off` or no `data/archetypes.toml`: only `UNCLASSIFIED` is accepted.
    Off,
    /// The loaded table.
    On(&'a Table),
}

impl<'a> Archetypes<'a> {
    /// `On` for a table, `Off` for none.
    pub fn from_table(t: Option<&'a Table>) -> Self {
        t.map_or(Self::Off, Self::On)
    }
}

/// A request to label a finding.
#[derive(Debug, Clone)]
pub struct LabelInput<'a> {
    /// The finding.
    pub finding: &'a FindingId,
    /// An archetype ID or `UNCLASSIFIED`.
    pub label: &'a str,
    /// Evidence to count (empty: the evidence the finding and its family cite).
    pub evidence: &'a [EvidenceId],
    /// Why nothing fits (required for `UNCLASSIFIED`) or any note.
    pub note: Option<&'a str>,
    /// The archetype table, or off.
    pub archetypes: Archetypes<'a>,
}

/// Lowercase hex SHA-256 of the entry's content: the proof of which table row a label
/// was accepted against.
pub fn entry_sha256(e: &Entry) -> String {
    let kinds = |v: &[Vec<EvidenceKind>]| {
        v.iter()
            .map(|a| a.iter().map(|k| k.as_str()).collect::<Vec<_>>().join("+"))
            .collect::<Vec<_>>()
            .join("|")
    };
    let text = format!(
        "{}\nholders={}\nregions={}\nrequires={}\ndisconfirm={}\n",
        e.id,
        e.holders
            .iter()
            .map(|h| h.as_str())
            .collect::<Vec<_>>()
            .join(","),
        e.regions
            .iter()
            .map(|r| r.as_str())
            .collect::<Vec<_>>()
            .join(","),
        kinds(&e.requires),
        e.disconfirm.join(","),
    );
    hex::encode(Sha256::digest(text.as_bytes()))
}

/// The command that produces evidence of `kind`, for refusal text.
pub fn producer_hint(kind: EvidenceKind, finding: &FindingId) -> String {
    match kind {
        EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries => format!(
            "dla harness run <H>, dla trend <S> --finding {finding}   (a passing column tied to {finding})"
        ),
        EvidenceKind::RetentionPath => format!(
            "dla harness run <H> --capture heap, dla summarize heap-graph <dumps> --finding {finding}   (native: the vgdb chain)"
        ),
        EvidenceKind::AllocStack => format!(
            "dla summarize alloc-stack-diff <profiles> --finding {finding}   (a profile with root paths names the holder)"
        ),
        EvidenceKind::HistogramDiff => {
            format!("dla summarize type-histogram-diff <captures> --finding {finding}")
        }
        EvidenceKind::Discriminator => {
            format!(
                "dla alloc-test <H> --finding {finding}   or   dla trend <S> --against <S2> --finding {finding}"
            )
        }
        EvidenceKind::Intervention => {
            format!("dla intervene <H> --finding {finding} --stub-writer <file:line>")
        }
        EvidenceKind::CensusHolder => "dla census --lang <lang>".to_owned(),
        EvidenceKind::StaticHit => "dla sweep --lang <lang>".to_owned(),
        EvidenceKind::NonHeap => "dla summarize non-heap <captures>   or   dla gap".to_owned(),
        EvidenceKind::ShouldBeDead => "dla harness run <H> --capture heap".to_owned(),
        other => format!("a {other} record tied to {finding}"),
    }
}

/// A short form of [`producer_hint`]: the command alone, for `--suggest` rows.
pub fn producer_command(kind: EvidenceKind, finding: &FindingId) -> String {
    match kind {
        EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries => {
            format!("dla trend <S> --finding {finding}")
        }
        EvidenceKind::RetentionPath => {
            format!("dla summarize heap-graph <dumps> --finding {finding}")
        }
        EvidenceKind::AllocStack => {
            format!("dla summarize alloc-stack-diff <profiles> --finding {finding}")
        }
        EvidenceKind::Discriminator => format!("dla alloc-test <H> --finding {finding}"),
        other => {
            let hint = producer_hint(other, finding);
            hint.split("   ").next().unwrap_or(&hint).to_owned()
        }
    }
}

fn refuse(code: &str, message: String, missing: impl Into<String>, fix: String) -> Refusal {
    Refusal::new(code, message, missing, fix)
}

/// The findings a label judges against: the finding itself for a leak-side one, its whole
/// family for a non-leak sibling (growth is measured for the root, the discriminator is
/// bound to the root).
fn subjects<'a>(findings: &'a dyn FindingLookup, f: &'a Finding) -> Vec<&'a Finding> {
    match f.polarity() {
        Polarity::Leak => vec![f],
        Polarity::NonLeak => findings.family(&f.id),
    }
}

fn names_subject(result: &Value, subjects: &[&Finding]) -> bool {
    let named = |s: &str| subjects.iter().any(|f| f.id.to_string() == s);
    if result
        .get("finding")
        .and_then(Value::as_str)
        .is_some_and(named)
    {
        return true;
    }
    result
        .get("candidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|c| {
            c.as_str().is_some_and(named)
                || c.get("finding").and_then(Value::as_str).is_some_and(named)
        })
}

/// Why `e` does not count for the finding behind `subjects`; `Ok` when it does.
/// `label` is the `N-*` id being claimed, if any.
fn tie(
    subjects: &[&Finding],
    cited: &[EvidenceId],
    e: &EvidenceRecord,
    label: Option<&str>,
) -> Result<(), String> {
    let id = &e.evidence_id;
    let f = subjects[0];
    if e.authority == Authority::Static
        && !matches!(e.kind, EvidenceKind::StaticHit | EvidenceKind::CensusHolder)
    {
        return Err(format!("{id} has authority static"));
    }
    match e.kind {
        EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries => {
            if !is_passing_growth(e) {
                return Err(format!(
                    "{id} is not a passing growth record (gate pass, authority test, live or dump)"
                ));
            }
            let tied = GrowthSummary::from_result(&e.result).is_some_and(|s| {
                s.passed.iter().any(|c| {
                    subjects
                        .iter()
                        .any(|f| c.tied_to_owner(&f.id, f.site.as_deref(), f.owner.as_deref()))
                })
            });
            if tied {
                Ok(())
            } else {
                Err(format!(
                    "{id} has no passing column tied to {} (by finding, site, candidates or a probe named after its owner)",
                    f.id
                ))
            }
        }
        EvidenceKind::Discriminator => {
            let bound = finding_of(e);
            if let Some(n) = label {
                if favors_of(e) != Some(Favors::NonLeak) {
                    return Err(format!("{id} does not favor non-leak"));
                }
                if bound
                    .as_ref()
                    .is_some_and(|b| !subjects.iter().any(|s| &s.id == b))
                {
                    return Err(format!("{id} is bound to a finding outside the family"));
                }
                let supports: Vec<&str> = e
                    .result
                    .get("supports")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect();
                if !supports.is_empty() && !supports.contains(&n) {
                    return Err(format!("{id} supports {}, not {n}", supports.join(", ")));
                }
                Ok(())
            } else if bound
                .as_ref()
                .is_some_and(|b| subjects.iter().any(|s| &s.id == b))
                || names_subject(&e.result, subjects)
            {
                // A discriminator that rules the leak out is no evidence for a leak archetype.
                if favors_of(e) == Some(Favors::Leak) {
                    Ok(())
                } else {
                    Err(format!("{id} does not favor leak"))
                }
            } else {
                Err(format!(
                    "{id} is not bound to {} (its finding is {})",
                    f.id,
                    bound.map_or("none".to_owned(), |b| b.to_string())
                ))
            }
        }
        EvidenceKind::RetentionPath | EvidenceKind::AllocStack => {
            if subjects.iter().any(|s| names_holder(e, s)) || names_subject(&e.result, subjects) {
                Ok(())
            } else {
                Err(format!(
                    "{id} names no holder of {} (no holder key is its owner, no allocation site is near its site, and its result names another finding)",
                    f.id
                ))
            }
        }
        EvidenceKind::Intervention => {
            let Some(r) = InterventionResult::of(e) else {
                return Err(format!("{id} is not an intervention"));
            };
            match r.why_not_removal_of(&f.id) {
                None => Ok(()),
                Some(why) => Err(format!("{id}: {why}")),
            }
        }
        _ => {
            if names_subject(&e.result, subjects) || cited.contains(id) {
                Ok(())
            } else {
                Err(format!(
                    "{id} (a {} record) names no finding and {} does not cite it",
                    e.kind, f.id
                ))
            }
        }
    }
}

/// The records the finding itself cites (its origin and every move that cited evidence,
/// confirmation included). A record of a kind whose result names no finding counts for
/// the finding only through this list, never because a sibling or the root cites it.
fn own_cited(f: &Finding) -> Vec<EvidenceId> {
    f.supports.clone()
}

/// True for a kind whose result names no finding: such a record counts for a finding only
/// when the finding itself cites it (or the result names it).
pub fn cited_only(kind: EvidenceKind) -> bool {
    !matches!(
        kind,
        EvidenceKind::GrowthSeries
            | EvidenceKind::UnitCountSeries
            | EvidenceKind::Discriminator
            | EvidenceKind::RetentionPath
            | EvidenceKind::AllocStack
            | EvidenceKind::Intervention
    )
}

/// The alternatives of `requires` with the fewest kinds still missing from `have`, in the
/// order of the list: `(index, missing kinds)`. Empty alternatives are skipped.
fn closest_alternatives(
    requires: &[Vec<EvidenceKind>],
    have: &dyn Fn(EvidenceKind) -> bool,
) -> Vec<(usize, Vec<EvidenceKind>)> {
    let missing: Vec<(usize, Vec<EvidenceKind>)> = requires
        .iter()
        .enumerate()
        .filter(|(_, alt)| !alt.is_empty())
        .map(|(i, alt)| (i, alt.iter().copied().filter(|k| !have(*k)).collect()))
        .collect();
    let Some(best) = missing.iter().map(|(_, m)| m.len()).min() else {
        return Vec::new();
    };
    missing
        .into_iter()
        .filter(|(_, m)| m.len() == best)
        .collect()
}

/// The evidence the finding and its family cite, in order, without repeats.
fn default_pool(findings: &dyn FindingLookup, f: &Finding) -> Vec<EvidenceId> {
    let mut out: Vec<EvidenceId> = Vec::new();
    for m in findings.family(&f.id) {
        for e in m.supports.iter().chain(&m.contradicts) {
            if !out.contains(e) {
                out.push(e.clone());
            }
        }
    }
    out
}

/// Why `f` cannot take a label of this polarity; `None` when it can.
fn polarity_gap(f: &Finding, label: &str) -> Option<Refusal> {
    let id = &f.id;
    if label.starts_with("L-") && f.polarity() != Polarity::Leak {
        return Some(refuse(
            codes::LABEL_POLARITY,
            format!("{id} is a non-leak hypothesis; {label} labels a leak-side finding"),
            "an N-* label on a non-leak sibling",
            format!(
                "dla ledger label {id} --as {}",
                f.hypothesis
                    .as_deref()
                    .and_then(nonleak_id)
                    .unwrap_or("N-WARM")
            ),
        ));
    }
    if label.starts_with("N-") {
        if f.polarity() != Polarity::NonLeak {
            return Some(refuse(
                codes::LABEL_POLARITY,
                format!(
                    "{id} is a leak-side finding; {label} labels a non-leak sibling (a finding whose hypothesis begins with that N-* id)"
                ),
                "an L-* label on a leak-side finding",
                format!("dla ledger label {id} --suggest"),
            ));
        }
        let want = f.hypothesis.as_deref().and_then(nonleak_id);
        if want != Some(label) {
            return Some(refuse(
                codes::LABEL_HYPOTHESIS_MISMATCH,
                format!(
                    "{id} proposes {}, not {label}",
                    want.unwrap_or("a hypothesis without an N-* id")
                ),
                format!("--as {}", want.unwrap_or("<the N-* id of its hypothesis>")),
                format!("dla ledger label {id} --as {}", want.unwrap_or("N-WARM")),
            ));
        }
    }
    None
}

fn state_gap(f: &Finding) -> Option<Refusal> {
    if matches!(f.state, FindingState::Confirmed | FindingState::Strong) {
        return None;
    }
    Some(refuse(
        codes::LABEL_STATE,
        format!(
            "{} is {}: a label is earned by confirmation",
            f.id,
            f.state_label()
        ),
        "a confirmed or strong finding",
        format!(
            "dla ledger confirm {} --evidence <growth or discriminator id>",
            f.id
        ),
    ))
}

fn unknown_archetype(label: &str, table: &Table) -> Refusal {
    refuse(
        codes::UNKNOWN_ARCHETYPE,
        format!("{label:?} is not an archetype ID"),
        "an archetype ID or UNCLASSIFIED",
        format!(
            "use one of: {}, UNCLASSIFIED",
            table.ids().collect::<Vec<_>>().join(", ")
        ),
    )
}

/// Read only: the refusal `plan` gives for the checks that need no evidence (archetypes
/// off, an unknown archetype, the finding's state, the label's polarity), in the order
/// `plan` makes them. `Ok` for UNCLASSIFIED, which has its own rules.
pub fn precheck(
    findings: &dyn FindingLookup,
    id: &FindingId,
    label: &str,
    archetypes: Archetypes<'_>,
) -> Result<(), Refusal> {
    let label = label.trim();
    if label == UNCLASSIFIED {
        return Ok(());
    }
    let Some(f) = findings.finding(id) else {
        return Err(refuse(
            crate::error::codes::UNKNOWN_FINDING,
            format!("label refers to {id}, which does not exist in the ledger"),
            format!("finding {id}"),
            "dla run status".to_owned(),
        ));
    };
    let table = match archetypes {
        Archetypes::Off => {
            return Err(refuse(
                codes::ARCHETYPES_OFF,
                format!(
                    "archetypes are off (DLA_ARCHETYPES=off or no data/archetypes.toml): {id} stays UNCLASSIFIED"
                ),
                "archetypes on, or --as UNCLASSIFIED --note",
                format!(
                    "dla ledger label {id} --as UNCLASSIFIED --note \"<holder and mechanism in words>\""
                ),
            ));
        }
        Archetypes::On(t) => t,
    };
    if table.get(label).is_none() {
        return Err(unknown_archetype(label, table));
    }
    if let Some(r) = state_gap(f) {
        return Err(r);
    }
    match polarity_gap(f, label) {
        Some(r) => Err(r),
        None => Ok(()),
    }
}

/// Builds the `finding_labeled` record for `input`, or the refusal that names what is
/// missing. Reads only the ledger and the archetype table.
pub fn plan(
    evidence: &dyn EvidenceLookup,
    findings: &dyn FindingLookup,
    input: &LabelInput<'_>,
) -> Result<FindingLabeled, Refusal> {
    let id = input.finding;
    let f = findings.finding(id).ok_or_else(|| {
        refuse(
            crate::error::codes::UNKNOWN_FINDING,
            format!("label refers to {id}, which does not exist in the ledger"),
            format!("finding {id}"),
            "dla run status".to_owned(),
        )
    })?;
    let label = input.label.trim();
    let note = input.note.map(str::trim).filter(|n| !n.is_empty());
    if label == UNCLASSIFIED {
        let Some(n) = note else {
            return Err(refuse(
                crate::error::codes::EMPTY_FIELD,
                format!("labelling {id} UNCLASSIFIED needs a note saying why nothing fits"),
                "--note \"<why no archetype fits>\"",
                format!(
                    "dla ledger label {id} --as UNCLASSIFIED --note \"<holder and mechanism in words>\""
                ),
            ));
        };
        if let Some(r) = state_gap(f) {
            return Err(r);
        }
        return Ok(FindingLabeled {
            finding: id.clone(),
            label: UNCLASSIFIED.to_owned(),
            previous: f.archetype.clone(),
            satisfied: LabelSatisfied::default(),
            entry_sha256: String::new(),
            region_mismatch: false,
            note: Some(n.to_owned()),
        });
    }
    let table = match input.archetypes {
        Archetypes::Off => {
            return Err(refuse(
                codes::ARCHETYPES_OFF,
                format!(
                    "archetypes are off (DLA_ARCHETYPES=off or no data/archetypes.toml): {id} stays UNCLASSIFIED"
                ),
                "archetypes on, or --as UNCLASSIFIED --note",
                format!(
                    "dla ledger label {id} --as UNCLASSIFIED --note \"<holder and mechanism in words>\""
                ),
            ));
        }
        Archetypes::On(t) => t,
    };
    let Some(entry) = table.get(label) else {
        return Err(unknown_archetype(label, table));
    };
    if let Some(r) = state_gap(f) {
        return Err(r);
    }
    if let Some(r) = polarity_gap(f, label) {
        return Err(r);
    }
    let subs = subjects(findings, f);
    let cited = own_cited(f);
    let nonleak = label.starts_with("N-").then_some(label);
    let explicit = !input.evidence.is_empty();
    let pool: Vec<EvidenceId> = if explicit {
        input.evidence.to_vec()
    } else {
        default_pool(findings, f)
    };
    // Which pool records count, by kind; why the others do not.
    let mut tied: BTreeMap<EvidenceKind, Vec<EvidenceId>> = BTreeMap::new();
    let mut why_not: Vec<String> = Vec::new();
    let mut uncited: Vec<(EvidenceKind, EvidenceId)> = Vec::new();
    for e in &pool {
        let Some(rec) = evidence.evidence(e) else {
            return Err(refuse(
                crate::error::codes::UNKNOWN_EVIDENCE,
                format!("label cites {e}, which does not exist in the ledger"),
                format!("evidence {e}"),
                format!(
                    "dla ledger label {id} --as {label}   (without --evidence, or with an existing id)"
                ),
            ));
        };
        match tie(&subs, &cited, rec, nonleak) {
            Ok(()) => tied.entry(rec.kind).or_default().push(e.clone()),
            Err(why) => {
                // Only a record excluded for lack of citation (not for authority) whose
                // kind one alternative of this archetype needs.
                let static_only = rec.authority == Authority::Static
                    && !matches!(
                        rec.kind,
                        EvidenceKind::StaticHit | EvidenceKind::CensusHolder
                    );
                if cited_only(rec.kind)
                    && !static_only
                    && entry.requires.iter().any(|alt| alt.contains(&rec.kind))
                {
                    uncited.push((rec.kind, e.clone()));
                }
                why_not.push(why);
            }
        }
    }
    let met = entry
        .requires
        .iter()
        .find(|alt| !alt.is_empty() && alt.iter().all(|k| tied.contains_key(k)));
    let Some(alt) = met else {
        if !explicit {
            // Records of the ledger that tie to the finding without being cited by it.
            let mut wide: BTreeMap<EvidenceKind, Vec<EvidenceId>> = BTreeMap::new();
            for e in evidence.evidence_ids() {
                if let Some(rec) = evidence.evidence(&e)
                    && tie(&subs, &cited, rec, nonleak).is_ok()
                {
                    wide.entry(rec.kind).or_default().push(e);
                }
            }
            if let Some(alt) = entry
                .requires
                .iter()
                .find(|alt| !alt.is_empty() && alt.iter().all(|k| wide.contains_key(k)))
            {
                let mut ev: Vec<String> = Vec::new();
                for k in alt {
                    if let Some(e) = wide.get(k).and_then(|v| v.first()) {
                        let e = e.to_string();
                        if !ev.contains(&e) {
                            ev.push(e);
                        }
                    }
                }
                return Err(refuse(
                    codes::LABEL_EVIDENCE_MISSING,
                    format!(
                        "{label} on {id}: {id} does not cite the records that meet it, but the ledger has them ({})",
                        ev.join(", ")
                    ),
                    format!("--evidence {}", ev.join(",")),
                    format!(
                        "dla ledger label {id} --as {label} --evidence {}",
                        ev.join(",")
                    ),
                ));
            }
        }
        return Err(evidence_missing(
            id, label, entry, &tied, &why_not, &uncited, explicit,
        ));
    };
    if explicit && !why_not.is_empty() {
        // Evidence the writer named must itself count; a typo should not pass silently.
        return Err(refuse(
            codes::LABEL_EVIDENCE_MISSING,
            format!(
                "{label} on {id}: some evidence named with --evidence does not count: {}",
                why_not.join("; ")
            ),
            "evidence tied to the finding",
            format!(
                "dla ledger label {id} --as {label}   (without --evidence uses the evidence {id} cites)"
            ),
        ));
    }
    // Holder and disconfirmation are checked once the evidence is there, so the refusal
    // that comes first is the one a reader can act on.
    if !entry.holders.is_empty() && !f.holder.is_some_and(|h| entry.holders.contains(&h)) {
        let have = f.holder.map_or("none", Holder::as_str);
        let fits: Vec<&str> = entry.holders.iter().map(|h| h.as_str()).collect();
        return Err(refuse(
            codes::LABEL_HOLDER,
            format!(
                "{id} has holder {have}, which {label} does not fit (it fits: {})",
                fits.join(", ")
            ),
            format!("a holder among {}", fits.join(", ")),
            format!(
                "if the holder is wrong, correct it with evidence: dla ledger annotate {id} --holder <holder> --evidence <heap-graph growth evidence id>; otherwise pick another archetype: dla ledger label {id} --suggest"
            ),
        ));
    }
    if label.starts_with("L-") {
        let open: Vec<String> = findings
            .family(&f.id)
            .into_iter()
            .filter(|s| s.polarity() == Polarity::NonLeak && s.state != FindingState::Refuted)
            .filter(|s| {
                s.hypothesis
                    .as_deref()
                    .and_then(nonleak_id)
                    .is_some_and(|n| entry.disconfirm.iter().any(|d| d == n))
            })
            .map(|s| {
                format!(
                    "{} ({}, {})",
                    s.id,
                    s.hypothesis
                        .as_deref()
                        .and_then(nonleak_id)
                        .unwrap_or("N-*"),
                    s.state
                )
            })
            .collect();
        if !open.is_empty() {
            let root = f.root_id();
            return Err(refuse(
                codes::LABEL_DISCONFIRM_OPEN,
                format!(
                    "{label} needs these sibling hypotheses refuted first, and they are not: {}",
                    open.join(", ")
                ),
                "each listed sibling refuted by a discriminator",
                format!(
                    "dla alloc-test <H> --finding {root}   then   dla ledger refute <sibling> --evidence <discriminator id>"
                ),
            ));
        }
    }
    let kinds: Vec<String> = {
        let mut k: Vec<String> = Vec::new();
        for x in alt {
            let s = x.as_str().to_owned();
            if !k.contains(&s) {
                k.push(s);
            }
        }
        k
    };
    let mut ev: Vec<EvidenceId> = Vec::new();
    for x in alt {
        if let Some(first) = tied.get(x).and_then(|v| v.first())
            && !ev.contains(first)
        {
            ev.push(first.clone());
        }
    }
    let region_mismatch = f
        .region
        .is_some_and(|r| !entry.regions.is_empty() && !entry.regions.contains(&r));
    Ok(FindingLabeled {
        finding: id.clone(),
        label: label.to_owned(),
        previous: f.archetype.clone(),
        satisfied: LabelSatisfied {
            kinds,
            evidence: ev,
        },
        entry_sha256: entry_sha256(entry),
        region_mismatch,
        note: note.map(str::to_owned),
    })
}

fn evidence_missing(
    id: &FindingId,
    label: &str,
    entry: &Entry,
    tied: &BTreeMap<EvidenceKind, Vec<EvidenceId>>,
    why_not: &[String],
    uncited: &[(EvidenceKind, EvidenceId)],
    explicit: bool,
) -> Refusal {
    let mut alts: Vec<String> = Vec::new();
    for alt in entry.requires.iter().take(4) {
        let missing: Vec<EvidenceKind> = alt
            .iter()
            .copied()
            .filter(|k| !tied.contains_key(k))
            .collect();
        alts.push(format!(
            "[{}] lacks {}",
            alt.iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(" + "),
            missing
                .iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    // The command for every closest alternative (fewest kinds missing, ties included), in
    // the order of the list, each command once.
    let closest = closest_alternatives(&entry.requires, &|k| tied.contains_key(&k));
    let mut seen: Vec<String> = Vec::new();
    let mut routes: Vec<String> = Vec::new();
    for (_, missing) in &closest {
        let mut route: Vec<String> = Vec::new();
        for k in missing {
            let h = producer_hint(*k, id);
            if !seen.contains(&h) {
                seen.push(h.clone());
                route.push(h);
            }
        }
        if !route.is_empty() {
            routes.push(route.join("   and   "));
        }
    }
    let mut message = format!(
        "{label} on {id}: no alternative of its required evidence is met by records tied to the finding: {}",
        alts.join("; ")
    );
    if !why_not.is_empty() {
        let shown: Vec<&str> = why_not.iter().take(3).map(String::as_str).collect();
        message.push_str(&format!(" (not counted: {})", shown.join("; ")));
    }
    let mut fix = if routes.is_empty() {
        format!("dla ledger label {id} --suggest")
    } else {
        routes.join("   or   ")
    };
    // A record that names no finding counts only when the finding cites it.
    let mut blocked: Vec<String> = Vec::new();
    for (k, e) in uncited {
        let s = format!("{e} ({k})");
        if !blocked.contains(&s) {
            blocked.push(s);
        }
    }
    if !blocked.is_empty() {
        let others: Vec<String> = entry
            .requires
            .iter()
            .filter(|alt| {
                !alt.is_empty() && !alt.iter().any(|k| cited_only(*k) && !tied.contains_key(k))
            })
            .map(|alt| {
                format!(
                    "[{}]",
                    alt.iter()
                        .map(|k| k.as_str())
                        .collect::<Vec<_>>()
                        .join(" + ")
                )
            })
            .collect();
        let tail = if others.is_empty() {
            format!(
                "no other alternative of {label} exists for a confirmed finding, so choose another row or UNCLASSIFIED"
            )
        } else {
            format!(
                "a confirmed finding uses the alternatives {}",
                others.join(" or ")
            )
        };
        fix.push_str(&format!(
            "   (note: {} names no finding, so it counts only when {id} cites it; a finding cites a record when it moves, dla ledger test|confirm {id} --evidence <E>, so produce it before confirmation; {tail})",
            blocked.join(", "),
        ));
    }
    refuse(
        codes::LABEL_EVIDENCE_MISSING,
        message,
        format!(
            "one alternative of {label}'s requires, each kind tied to {id}{}",
            if explicit {
                " (from the --evidence given)"
            } else {
                ""
            }
        ),
        fix,
    )
}

/// The replay check of a `finding_labeled` line. Reads only the ledger and the record.
pub fn check(
    evidence: &dyn EvidenceLookup,
    findings: &dyn FindingLookup,
    rec: &FindingLabeled,
) -> Result<(), Refusal> {
    let id = &rec.finding;
    let f = findings.finding(id).ok_or_else(|| {
        refuse(
            crate::error::codes::UNKNOWN_FINDING,
            format!("finding_labeled refers to {id}, which does not exist earlier in the ledger"),
            format!("finding {id}"),
            "dla run status".to_owned(),
        )
    })?;
    if rec.previous != f.archetype {
        return Err(refuse(
            codes::LABEL_STATE,
            format!(
                "label of {id} says it replaces {:?}, but the finding is labelled {:?}",
                rec.previous, f.archetype
            ),
            format!("previous {:?}", f.archetype),
            format!("dla ledger label {id} --as {}", rec.label),
        ));
    }
    if let Some(r) = state_gap(f) {
        return Err(r);
    }
    if rec.note.as_deref().is_some_and(|n| n.trim().is_empty()) {
        return Err(refuse(
            crate::error::codes::EMPTY_FIELD,
            format!("the note of the label on {id} is blank"),
            "note text, or omit --note",
            format!("dla ledger label {id} --as {} --note \"<text>\"", rec.label),
        ));
    }
    if rec.label == UNCLASSIFIED {
        if rec.note.is_none() {
            return Err(refuse(
                crate::error::codes::EMPTY_FIELD,
                format!("labelling {id} UNCLASSIFIED needs a note"),
                "--note",
                format!("dla ledger label {id} --as UNCLASSIFIED --note \"<why nothing fits>\""),
            ));
        }
        return Ok(());
    }
    if !ARCHETYPE_IDS.contains(&rec.label.as_str()) {
        return Err(refuse(
            codes::UNKNOWN_ARCHETYPE,
            format!("{:?} is not an archetype ID", rec.label),
            "an archetype ID or UNCLASSIFIED",
            format!("use one of: {}, UNCLASSIFIED", ARCHETYPE_IDS.join(", ")),
        ));
    }
    if let Some(r) = polarity_gap(f, &rec.label) {
        return Err(r);
    }
    if !crate::verify::is_sha256(&rec.entry_sha256) {
        return Err(refuse(
            codes::LABEL_EVIDENCE_MISSING,
            format!("the label on {id} carries no valid entry_sha256"),
            "entry_sha256, the hash of the archetype entry used",
            format!("dla ledger label {id} --as {}", rec.label),
        ));
    }
    let subs = subjects(findings, f);
    let cited = own_cited(f);
    let nonleak = rec.label.starts_with("N-").then_some(rec.label.as_str());
    let mut by_kind: BTreeMap<EvidenceKind, Vec<EvidenceId>> = BTreeMap::new();
    for e in &rec.satisfied.evidence {
        let Some(r) = evidence.evidence(e) else {
            return Err(refuse(
                crate::error::codes::UNKNOWN_EVIDENCE,
                format!("the label on {id} cites {e}, which does not exist earlier in the ledger"),
                format!("evidence {e}"),
                format!("dla ledger label {id} --as {}", rec.label),
            ));
        };
        if let Err(why) = tie(&subs, &cited, r, nonleak) {
            return Err(refuse(
                codes::LABEL_EVIDENCE_MISSING,
                format!("the label on {id} rests on a record that does not count: {why}"),
                format!("evidence tied to {id}"),
                format!("dla ledger label {id} --as {} --suggest", rec.label),
            ));
        }
        by_kind.entry(r.kind).or_default().push(e.clone());
    }
    if rec.satisfied.kinds.is_empty() {
        return Err(refuse(
            codes::LABEL_EVIDENCE_MISSING,
            format!("the label on {id} states no evidence kinds"),
            "the kinds of the met alternative",
            format!("dla ledger label {id} --as {}", rec.label),
        ));
    }
    for k in &rec.satisfied.kinds {
        let ok = k
            .parse::<EvidenceKind>()
            .ok()
            .is_some_and(|kind| by_kind.contains_key(&kind));
        if !ok {
            return Err(refuse(
                codes::LABEL_EVIDENCE_MISSING,
                format!(
                    "the label on {id} says kind {k} is met, but no cited record of that kind counts"
                ),
                format!("a {k} record tied to {id}"),
                format!("dla ledger label {id} --as {} --suggest", rec.label),
            ));
        }
    }
    Ok(())
}

/// One archetype in the answer to `dla ledger label --suggest`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SuggestRow {
    /// The archetype ID.
    pub id: String,
    /// `met` (an alternative is met and nothing else stops the label), `blocked` (an
    /// alternative is met but the holder or a sibling stops it) or `missing`.
    pub status: &'static str,
    /// Kinds of the met alternative, for `met` and `blocked`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub met: Vec<String>,
    /// The records that meet them: pass them to `--evidence` (a record the finding does
    /// not cite yet is found by its tie to the finding).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
    /// For `missing`: the fewest kinds still needed (the closest alternative).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
    /// For `missing`: the kinds the other equally close alternatives still need, one list
    /// per alternative, in the order of the `requires` list.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub or_missing: Vec<Vec<String>>,
    /// What stops a met alternative from becoming the label.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,
    /// The entry's one-line mechanism.
    pub summary: String,
}

/// Read-only: per archetype that fits the finding's polarity, whether it can be labelled
/// now and what is missing otherwise. Met rows first, then the rows missing evidence (the
/// closest first), then the rows a wrong holder or an open sibling blocks.
pub fn suggest(
    evidence: &dyn EvidenceLookup,
    findings: &dyn FindingLookup,
    finding: &FindingId,
    table: &Table,
) -> Result<Vec<SuggestRow>, Refusal> {
    let f = findings.finding(finding).ok_or_else(|| {
        refuse(
            crate::error::codes::UNKNOWN_FINDING,
            format!("{finding} does not exist in the ledger"),
            format!("finding {finding}"),
            "dla run status".to_owned(),
        )
    })?;
    let subs = subjects(findings, f);
    let cited = own_cited(f);
    // Everything the finding cites, then every other record of the ledger: whether one
    // counts is the tie rule's to say.
    let mut pool = default_pool(findings, f);
    for e in evidence.evidence_ids() {
        if !pool.contains(&e) {
            pool.push(e);
        }
    }
    let mut rows: Vec<SuggestRow> = Vec::new();
    for entry in table.entries() {
        let leak_side = f.polarity() == Polarity::Leak;
        if entry.id.starts_with("L-") != leak_side {
            continue;
        }
        if !leak_side && f.hypothesis.as_deref().and_then(nonleak_id) != Some(entry.id.as_str()) {
            continue;
        }
        let nonleak = (!leak_side).then_some(entry.id.as_str());
        let mut tied: BTreeMap<EvidenceKind, Vec<EvidenceId>> = BTreeMap::new();
        for e in &pool {
            if let Some(rec) = evidence.evidence(e)
                && tie(&subs, &cited, rec, nonleak).is_ok()
            {
                tied.entry(rec.kind).or_default().push(e.clone());
            }
        }
        let alts: Vec<Vec<EvidenceKind>> = entry
            .requires
            .iter()
            .map(|a| {
                a.iter()
                    .copied()
                    .filter(|k| !tied.contains_key(k))
                    .collect()
            })
            .collect();
        let summary = entry.summary.clone();
        if let Some(i) = alts.iter().position(Vec::is_empty) {
            let mut blocked_by = Vec::new();
            if !entry.holders.is_empty() && !f.holder.is_some_and(|h| entry.holders.contains(&h)) {
                blocked_by.push(format!(
                    "holder {} is not among {}",
                    f.holder.map_or("none", Holder::as_str),
                    entry
                        .holders
                        .iter()
                        .map(|h| h.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            for s in findings.family(&f.id) {
                if leak_side
                    && s.polarity() == Polarity::NonLeak
                    && s.state != FindingState::Refuted
                    && s.hypothesis
                        .as_deref()
                        .and_then(nonleak_id)
                        .is_some_and(|n| entry.disconfirm.iter().any(|d| d == n))
                {
                    blocked_by.push(format!("{} is not refuted", s.id));
                }
            }
            let mut chosen: Vec<String> = Vec::new();
            for k in &entry.requires[i] {
                if let Some(e) = tied.get(k).and_then(|v| v.first()) {
                    let e = e.to_string();
                    if !chosen.contains(&e) {
                        chosen.push(e);
                    }
                }
            }
            rows.push(SuggestRow {
                id: entry.id.clone(),
                status: if blocked_by.is_empty() {
                    "met"
                } else {
                    "blocked"
                },
                met: entry.requires[i]
                    .iter()
                    .map(|k| k.as_str().to_owned())
                    .collect(),
                evidence: chosen,
                missing: Vec::new(),
                or_missing: Vec::new(),
                blocked_by,
                summary,
            });
        } else {
            let mut closest = closest_alternatives(&entry.requires, &|k| tied.contains_key(&k));
            // A confirmed finding can no longer cite a record of a kind that names no
            // finding: when the closest alternative needs one, show the closest one it can
            // still meet (every row stays; with none to meet, the closest stays).
            if matches!(f.state, FindingState::Confirmed | FindingState::Strong) {
                let meetable = |alt: &[EvidenceKind]| {
                    alt.iter().all(|k| tied.contains_key(k) || !cited_only(*k))
                };
                let reachable: Vec<Vec<EvidenceKind>> = entry
                    .requires
                    .iter()
                    .filter(|a| !a.is_empty() && meetable(a))
                    .cloned()
                    .collect();
                if !reachable.is_empty() {
                    closest = closest_alternatives(&reachable, &|k| tied.contains_key(&k));
                }
            }
            let names = |m: &[EvidenceKind]| -> Vec<String> {
                m.iter().map(|k| k.as_str().to_owned()).collect()
            };
            let mut sets: Vec<Vec<String>> = Vec::new();
            for (_, m) in &closest {
                let n = names(m);
                if !sets.contains(&n) {
                    sets.push(n);
                }
            }
            let mut it = sets.into_iter();
            let first = it.next().unwrap_or_default();
            rows.push(SuggestRow {
                id: entry.id.clone(),
                status: "missing",
                met: Vec::new(),
                evidence: Vec::new(),
                missing: first,
                or_missing: it.collect(),
                blocked_by: Vec::new(),
                summary,
            });
        }
    }
    let rank = |r: &SuggestRow| match r.status {
        "met" => (0, 0),
        "blocked" => (2, 0),
        _ => (1, r.missing.len()),
    };
    rows.sort_by(|a, b| rank(a).cmp(&rank(b)).then(a.id.cmp(&b.id)));
    Ok(rows)
}
