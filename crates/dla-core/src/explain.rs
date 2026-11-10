//! Explanations: the rules behind `dla ledger explain` and the replay check of a
//! `finding_explained` line.
//!
//! An explanation is a structured claim plus text. The checks are deterministic and
//! prove that the explanation is consistent with what the ledger and the census record
//! and that it states the parts a reader needs; they do not prove that it is true. Truth
//! rests on the verifier (the bundle leaves the explanation out) and on an intervention.
//!
//! [`check`] is the replay check: it reads the record, the finding and the cited
//! evidence, never the census. [`plan`] adds what only the writing side can see: the
//! writers and removers the census found (passed in by the caller).

use crate::{
    error::{Refusal, codes as core_codes},
    evidence::{Authority, EvidenceKind, EvidenceRecord},
    finding::{ExplanationTag, Finding, FindingState, Polarity, check_site},
    holders::{InterventionResult, names_holder},
    ids::{EvidenceId, FindingId},
    m6::codes,
    machine::{EvidenceLookup, FindingLookup, is_passing_growth},
    record::{FindingExplained, RemovalClaim},
    trend::GrowthSummary,
};

/// Shortest and longest explanation text, in characters.
pub const TEXT_MIN: usize = 40;
/// See [`TEXT_MIN`].
pub const TEXT_MAX: usize = 600;

/// A request to explain a finding.
#[derive(Debug, Clone)]
pub struct ExplainInput<'a> {
    /// The finding.
    pub finding: &'a FindingId,
    /// Observed or inferred.
    pub tag: ExplanationTag,
    /// Adding sites (empty: the recorded writers).
    pub adds: &'a [String],
    /// What happens to the items afterwards.
    pub removal: RemovalClaim,
    /// The remover, for `conditional`, `wrong-key` and `unreachable`.
    pub remover: Option<&'a str>,
    /// Evidence that shows the pairing (required with `observed`).
    pub evidence: Option<&'a EvidenceId>,
    /// The text.
    pub text: &'a str,
    /// Writers recorded for the finding (annotations and census), `file:line`.
    pub writers: &'a [String],
    /// Removers recorded for the finding (annotations and census), `file:line`.
    pub removers: &'a [String],
}

fn refuse(code: &str, message: String, missing: impl Into<String>, fix: String) -> Refusal {
    Refusal::new(code, message, missing, fix)
}

fn word_at(hay: &str, needle: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut from = 0;
    while let Some(i) = hay[from..].find(needle) {
        let start = from + i;
        let end = start + needle.len();
        if !hay[..start].chars().next_back().is_some_and(ident)
            && !hay[end..].chars().next().is_some_and(ident)
        {
            return true;
        }
        from = end;
    }
    false
}

fn has_marker(lower: &str, marker: &str) -> bool {
    if marker.contains(' ') {
        lower.contains(marker)
    } else {
        word_at(lower, marker)
    }
}

/// Marker words per removal claim: at least one must occur in the text. This proves the
/// removal half is stated, not that it is true.
pub fn markers(removal: RemovalClaim) -> &'static [&'static str] {
    match removal {
        RemovalClaim::Never => &["never", "no code removes", "nothing removes", "not removed"],
        RemovalClaim::NotFound => &["no remover found", "found no remover", "not found"],
        RemovalClaim::Conditional => &["only when", "unless", "if"],
        RemovalClaim::WrongKey => &["key", "different"],
        RemovalClaim::Unreachable => &["never reached", "not reached", "dead"],
    }
}

/// True when `text` names `site` as `path:line` or `basename:line`.
pub fn names_site(text: &str, site: &str) -> bool {
    if text.contains(site) {
        return true;
    }
    let Some((file, line)) = site.rsplit_once(':') else {
        return false;
    };
    let base = file.rsplit(['/', '\\']).next().unwrap_or(file);
    text.contains(&format!("{base}:{line}"))
}

fn last_components(owner: &str, n: usize) -> String {
    let parts: Vec<&str> = owner.split('.').filter(|p| !p.is_empty()).collect();
    parts[parts.len().saturating_sub(n)..].join(".")
}

fn names_owner(text: &str, owner: &str) -> bool {
    text.contains(owner) || text.contains(&last_components(owner, 2))
}

fn observed_gap(evidence: &dyn EvidenceLookup, f: &Finding, id: &EvidenceId) -> Result<(), String> {
    let Some(e) = evidence.evidence(id) else {
        return Err(format!("{id} does not exist in the ledger"));
    };
    observed_ok(e, f)
}

fn observed_ok(e: &EvidenceRecord, f: &Finding) -> Result<(), String> {
    let id = &e.evidence_id;
    match e.kind {
        EvidenceKind::RetentionPath | EvidenceKind::AllocStack => {
            if names_holder(e, f) {
                Ok(())
            } else {
                Err(format!("{id} names no holder of {}", f.id))
            }
        }
        EvidenceKind::Intervention => match InterventionResult::of(e) {
            Some(r) => r
                .why_not_removal_of(&f.id)
                .map_or(Ok(()), |w| Err(format!("{id}: {w}"))),
            None => Err(format!("{id} is not an intervention")),
        },
        EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries => {
            let tied = is_passing_growth(e)
                && GrowthSummary::from_result(&e.result).is_some_and(|s| {
                    s.passed
                        .iter()
                        .any(|c| c.tied_to_owner(&f.id, f.site.as_deref(), f.owner.as_deref()))
                });
            if tied {
                Ok(())
            } else {
                Err(format!("{id} has no passing column tied to {}", f.id))
            }
        }
        other => Err(format!(
            "{id} is a {other} record; observed needs a retention path, an intervention or tied growth"
        )),
    }
}

/// The checks that need only the record, the finding and the evidence.
fn structural(
    evidence: &dyn EvidenceLookup,
    findings: &dyn FindingLookup,
    rec: &FindingExplained,
) -> Result<(), Refusal> {
    let id = &rec.finding;
    let f = findings.finding(id).ok_or_else(|| {
        refuse(
            core_codes::UNKNOWN_FINDING,
            format!("finding_explained refers to {id}, which does not exist earlier in the ledger"),
            format!("finding {id}"),
            "dla run status".to_owned(),
        )
    })?;
    if f.polarity() != Polarity::Leak
        || !matches!(f.state, FindingState::Confirmed | FindingState::Strong)
    {
        return Err(refuse(
            codes::EXPLAIN_STATE,
            format!(
                "{id} is {} ({}): only a confirmed or strong leak-side finding is explained",
                f.state_label(),
                if f.polarity() == Polarity::Leak {
                    "leak-side"
                } else {
                    "non-leak"
                }
            ),
            "a confirmed or strong leak-side finding",
            format!("dla ledger confirm {id} --evidence <growth evidence id>"),
        ));
    }
    let fix = |extra: &str| {
        format!(
            "dla ledger explain {id} --tag inferred --adds <file:line> --removal {} {extra}--text \"<what adds, what holds, how removal fails>\"",
            rec.removal
        )
    };
    let chars = rec.text.chars().count();
    if !(TEXT_MIN..=TEXT_MAX).contains(&chars) || rec.text.chars().any(char::is_control) {
        return Err(refuse(
            codes::EXPLAIN_TEXT,
            format!(
                "the explanation text of {id} has {chars} characters (and may not contain control characters or newlines); it must be {TEXT_MIN} to {TEXT_MAX}"
            ),
            format!("--text of {TEXT_MIN} to {TEXT_MAX} characters on one line"),
            fix(""),
        ));
    }
    let Some(holder) = f.holder else {
        return Err(refuse(
            core_codes::HOLDER_REQUIRED,
            format!("{id} has no holder, so there is nothing to explain the holding of"),
            "a holder on the finding",
            format!("dla ledger annotate {id} --holder <holder>"),
        ));
    };
    let text = rec.text.as_str();
    match (&f.site, &f.owner) {
        (Some(site), _) => {
            if !names_site(text, site) {
                return Err(refuse(
                    codes::EXPLAIN_MISSING_SITE,
                    format!("the text does not name the site {site} of {id}"),
                    format!("{site} (or its basename:line) in --text"),
                    fix(""),
                ));
            }
        }
        (None, Some(owner)) => {
            if !names_owner(text, owner) {
                return Err(refuse(
                    codes::EXPLAIN_MISSING_SITE,
                    format!(
                        "the text does not name the owner {owner} of {id} (an artifact finding has no site)"
                    ),
                    format!("{owner} in --text"),
                    fix(""),
                ));
            }
        }
        (None, None) => {
            return Err(refuse(
                codes::EXPLAIN_STATE,
                format!("{id} has neither a site nor an owner"),
                "a site, or an owner on an artifact run",
                format!("dla ledger annotate {id} --site <file:line>"),
            ));
        }
    }
    let lower = text.to_lowercase();
    let kind = holder.as_str();
    let tail = f
        .owner
        .as_deref()
        .map(|o| o.rsplit('.').next().unwrap_or(o))
        .filter(|t| t.len() >= 3);
    let holder_named = lower.contains(kind)
        || f.owner.as_deref().is_some_and(|o| names_owner(text, o))
        || tail.is_some_and(|t| word_at(text, t));
    if !holder_named {
        return Err(refuse(
            codes::EXPLAIN_MISSING_HOLDER,
            format!("the text does not name the holder of {id}: neither {kind:?} nor its owner"),
            format!("{kind} or the owner's name in --text"),
            fix(""),
        ));
    }
    if rec.adds.is_empty() {
        return Err(refuse(
            codes::EXPLAIN_ADDS,
            format!("the explanation of {id} names no site that adds to the holder"),
            "--adds file:line",
            fix("--adds <file:line> "),
        ));
    }
    for a in &rec.adds {
        if let Err(m) = check_site(a) {
            return Err(refuse(
                core_codes::INVALID_SITE,
                m,
                "adds written file:line",
                fix("--adds src/Main.java:42 "),
            ));
        }
    }
    if !rec.adds.iter().any(|a| names_site(text, a)) {
        return Err(refuse(
            codes::EXPLAIN_ADDS,
            format!(
                "the text names none of the adding sites ({})",
                rec.adds.join(", ")
            ),
            "one adding site as file:line in --text",
            fix(""),
        ));
    }
    match (rec.removal, rec.remover.as_deref()) {
        (RemovalClaim::Never | RemovalClaim::NotFound, Some(r)) => {
            return Err(refuse(
                codes::EXPLAIN_REMOVER,
                format!(
                    "removal {} says nothing removes the items, but a remover {r} is given",
                    rec.removal
                ),
                "no --remover with never or not-found",
                format!("dla ledger explain {id} --removal conditional --remover {r} ..."),
            ));
        }
        (RemovalClaim::Conditional | RemovalClaim::WrongKey | RemovalClaim::Unreachable, None) => {
            return Err(refuse(
                codes::EXPLAIN_REMOVER,
                format!("removal {} needs the remover that exists", rec.removal),
                "--remover file:line",
                fix("--remover <file:line> "),
            ));
        }
        (_, Some(r)) => {
            if let Err(m) = check_site(r) {
                return Err(refuse(
                    core_codes::INVALID_SITE,
                    m,
                    "remover written file:line",
                    fix("--remover src/Main.java:77 "),
                ));
            }
            if !names_site(text, r) {
                return Err(refuse(
                    codes::EXPLAIN_REMOVER,
                    format!("the text does not name the remover {r}"),
                    format!("{r} in --text"),
                    fix(""),
                ));
            }
        }
        (_, None) => {}
    }
    if !markers(rec.removal).iter().any(|m| has_marker(&lower, m)) {
        return Err(refuse(
            codes::EXPLAIN_REMOVAL_TEXT,
            format!(
                "the text does not state the removal {}: none of the words {} occurs",
                rec.removal,
                markers(rec.removal)
                    .iter()
                    .map(|m| format!("{m:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!("a phrase such as {:?} in --text", markers(rec.removal)[0]),
            fix(""),
        ));
    }
    match (rec.tag, rec.evidence.as_ref()) {
        (ExplanationTag::Observed, None) => {
            return Err(refuse(
                codes::EXPLAIN_TAG,
                format!(
                    "{id}: observed needs --evidence, a retention path, an intervention or growth tied to it"
                ),
                "--evidence <id>, or --tag inferred",
                fix("--evidence <id> "),
            ));
        }
        (ExplanationTag::Observed, Some(e)) => {
            observed_gap(evidence, f, e).map_err(|why| {
                refuse(
                    codes::EXPLAIN_TAG,
                    format!("{id}: observed does not rest on {e}: {why}"),
                    "a retention path naming the holder, an intervention that removed the growth, or growth tied to the finding",
                    format!("dla ledger explain {id} --tag inferred ..."),
                )
            })?;
        }
        (ExplanationTag::Inferred, Some(e)) if evidence.evidence(e).is_none() => {
            return Err(refuse(
                core_codes::UNKNOWN_EVIDENCE,
                format!(
                    "the explanation of {id} cites {e}, which does not exist earlier in the ledger"
                ),
                format!("evidence {e}"),
                fix(""),
            ));
        }
        _ => {}
    }
    if let Some(e) = &rec.evidence
        && let Some(r) = evidence.evidence(e)
        && r.authority == Authority::Static
        && rec.tag == ExplanationTag::Observed
    {
        return Err(refuse(
            codes::EXPLAIN_TAG,
            format!("{e} was read from source text, which is not an observation"),
            "evidence of authority test, live or dump",
            fix(""),
        ));
    }
    Ok(())
}

/// The replay check of a `finding_explained` line.
pub fn check(
    evidence: &dyn EvidenceLookup,
    findings: &dyn FindingLookup,
    rec: &FindingExplained,
) -> Result<(), Refusal> {
    structural(evidence, findings, rec)
}

/// Builds the record for `input`, or the refusal that names the missing part.
pub fn plan(
    evidence: &dyn EvidenceLookup,
    findings: &dyn FindingLookup,
    input: &ExplainInput<'_>,
) -> Result<FindingExplained, Refusal> {
    let id = input.finding;
    let f = findings.finding(id).ok_or_else(|| {
        refuse(
            core_codes::UNKNOWN_FINDING,
            format!("{id} does not exist in the ledger"),
            format!("finding {id}"),
            "dla run status".to_owned(),
        )
    })?;
    let mut adds: Vec<String> = Vec::new();
    for a in input.adds {
        if !adds.contains(a) {
            adds.push(a.clone());
        }
    }
    if adds.is_empty() {
        adds = input.writers.to_vec();
    }
    let rec = FindingExplained {
        finding: id.clone(),
        tag: input.tag,
        adds,
        removal: input.removal,
        remover: input.remover.map(str::to_owned),
        evidence: input.evidence.cloned(),
        text: input.text.trim().to_owned(),
    };
    // The cheapest fix first: a writer must be recorded at all; then the shape of the
    // record and its text; then what the recorded sites say.
    structural_state(findings, id)?;
    if input.writers.is_empty() {
        return Err(refuse(
            codes::EXPLAIN_NO_WRITER,
            format!(
                "no site that adds to the holder of {id} is recorded (annotations and census), so the adding half cannot be checked"
            ),
            "a recorded writer",
            format!("dla ledger annotate {id} --writer <file:line>"),
        ));
    }
    structural(evidence, findings, &rec)?;
    for a in &rec.adds {
        let known = input.writers.contains(a) || f.site.as_deref() == Some(a.as_str());
        if !known {
            return Err(refuse(
                codes::EXPLAIN_ADDS,
                format!(
                    "{a} is not a recorded writer of {id} (recorded: {})",
                    input
                        .writers
                        .iter()
                        .take(5)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                "adds among the recorded writers, or the finding's site",
                format!(
                    "dla ledger annotate {id} --writer {a}   (if it does add), or use one of the recorded writers"
                ),
            ));
        }
    }
    match rec.removal {
        RemovalClaim::Never | RemovalClaim::NotFound => {
            if let Some(r) = input.removers.first() {
                return Err(refuse(
                    codes::EXPLAIN_REMOVER_RECORDED,
                    format!(
                        "removal {} contradicts the recorded remover {r}: say conditional, wrong-key or unreachable and cite it",
                        rec.removal
                    ),
                    "a removal claim consistent with the recorded removers",
                    format!("dla ledger explain {id} --removal conditional --remover {r} ..."),
                ));
            }
        }
        _ => {
            if let Some(r) = rec.remover.as_deref()
                && !input.removers.iter().any(|x| x == r)
            {
                return Err(refuse(
                    codes::EXPLAIN_REMOVER,
                    format!(
                        "{r} is not a recorded remover of {id} (recorded: {})",
                        if input.removers.is_empty() {
                            "none".to_owned()
                        } else {
                            input
                                .removers
                                .iter()
                                .take(5)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ),
                    "a recorded remover",
                    format!(
                        "dla ledger annotate {id} --remover {r}   (if it does remove), or --removal never"
                    ),
                ));
            }
        }
    }
    Ok(rec)
}

fn structural_state(findings: &dyn FindingLookup, id: &FindingId) -> Result<(), Refusal> {
    let f = findings.finding(id).expect("checked by the caller");
    if f.polarity() != Polarity::Leak
        || !matches!(f.state, FindingState::Confirmed | FindingState::Strong)
    {
        return Err(refuse(
            codes::EXPLAIN_STATE,
            format!(
                "{id} is {}: only a confirmed or strong leak-side finding is explained",
                f.state_label()
            ),
            "a confirmed or strong leak-side finding",
            format!("dla ledger confirm {id} --evidence <growth evidence id>"),
        ));
    }
    if f.holder.is_none() {
        return Err(refuse(
            core_codes::HOLDER_REQUIRED,
            format!("{id} has no holder, so there is nothing to explain the holding of"),
            "a holder on the finding",
            format!("dla ledger annotate {id} --holder <holder>"),
        ));
    }
    Ok(())
}

/// A one-line rendering of the structured part, for the report.
pub fn structured_line(rec: &FindingExplained) -> String {
    let mut s = format!("adds: {}; removal: {}", rec.adds.join(", "), rec.removal);
    if let Some(r) = &rec.remover {
        s.push_str(&format!(" at {r}"));
    }
    if let Some(e) = &rec.evidence {
        s.push_str(&format!("; evidence: {e}"));
    }
    s
}
