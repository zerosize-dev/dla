//! The finding state machine.
//!
//! [`allowed_targets`] is the whole graph. [`check_transition`] validates a move:
//! first the graph, then a [`TransitionRules`] implementation that holds the
//! evidence rules attached to edges. [`M0Rules`] checks the graph plus the reason,
//! remedy, static and discriminator rules; [`M3Rules`] adds the dynamic rules (a
//! harness, a non-leak sibling, the growth gate, a site, a holder, refuted
//! alternatives and the direction of discriminators). Ledger lines written with
//! schema version 1 replay under [`M0Rules`], version 2 under [`M3Rules`].

use crate::{
    discriminator::{Favors, favors_of, finding_of},
    error::{Refusal, codes},
    evidence::{Authority, EvidenceClass, EvidenceKind, EvidenceRecord},
    finding::{Finding, FindingState, Polarity, Region, Shape, UnresolvedReason},
    harness::harness_id_of,
    ids::{EvidenceId, FindingId},
    trend::{GrowthSummary, finding_region, finding_shape},
};

/// States reachable from `from` in one move.
pub fn allowed_targets(from: FindingState) -> &'static [FindingState] {
    use FindingState::*;
    match from {
        Candidate => &[UnderTest, Refuted, SuspectedStaticOnly, Unresolved],
        UnderTest => &[Confirmed, Refuted, SuspectedStaticOnly, Unresolved],
        Confirmed => &[Strong],
        SuspectedStaticOnly => &[UnderTest, Unresolved],
        Unresolved => &[Candidate, UnderTest],
        Refuted | Strong => &[],
    }
}

/// True if `from -> to` is an edge of the graph.
pub fn is_allowed(from: FindingState, to: FindingState) -> bool {
    allowed_targets(from).contains(&to)
}

/// Read access to evidence records by ID.
pub trait EvidenceLookup {
    /// The record with this ID, if it exists.
    fn evidence(&self, id: &EvidenceId) -> Option<&EvidenceRecord>;
    /// Every existing evidence ID, in order (used to make refusals actionable).
    fn evidence_ids(&self) -> Vec<EvidenceId>;
}

/// Read access to findings by ID.
pub trait FindingLookup {
    /// The finding with this ID, if it exists.
    fn finding(&self, id: &FindingId) -> Option<&Finding>;
    /// The family of `id`: its root (the finding itself, or its parent) followed by
    /// every sibling of the root. Empty when `id` does not exist.
    fn family(&self, id: &FindingId) -> Vec<&Finding>;
}

/// Everything a rule needs to judge a move.
pub struct TransitionContext<'a> {
    /// The finding as currently replayed.
    pub finding: &'a Finding,
    /// Requested target state.
    pub to: FindingState,
    /// Reason supplied with the move.
    pub reason: Option<UnresolvedReason>,
    /// Remedy supplied with the move.
    pub remedy: Option<&'a str>,
    /// Evidence cited by the move. Merged into `contradicts` for `refuted`,
    /// into `supports` otherwise.
    pub evidence: &'a [EvidenceId],
    /// Evidence index.
    pub lookup: &'a dyn EvidenceLookup,
    /// Finding index (families and polarity).
    pub findings: &'a dyn FindingLookup,
}

/// Evidence rules attached to edges of the graph.
///
/// Extension point: later milestones implement this trait (or extend [`M3Rules`])
/// to add the rules for `strong`.
pub trait TransitionRules {
    /// Judges a move whose edge is already known to exist.
    fn check(&self, ctx: &TransitionContext<'_>) -> Result<(), Refusal>;
}

/// Validates a move: the graph first, then `rules`.
pub fn check_transition(
    rules: &dyn TransitionRules,
    ctx: &TransitionContext<'_>,
) -> Result<(), Refusal> {
    let from = ctx.finding.state;
    if !is_allowed(from, ctx.to) {
        return Err(illegal(ctx.finding, ctx.to));
    }
    rules.check(ctx)
}

/// The command that performs the edge `finding -> to`, and whether it exists in this
/// build. Commands that arrive in a later milestone are labelled with that milestone.
fn edge_command(id: &crate::ids::FindingId, to: FindingState) -> (String, bool) {
    match to {
        FindingState::Unresolved => (
            format!(
                "dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\""
            ),
            true,
        ),
        FindingState::SuspectedStaticOnly => (
            format!(
                "dla ledger resolve {id} --as suspected_static_only --reason not_exercised --remedy \"<text>\" --evidence <static-hit or census-holder id>"
            ),
            true,
        ),
        FindingState::UnderTest => (
            format!("dla ledger test {id} --evidence <harness evidence id>"),
            true,
        ),
        FindingState::Confirmed => (
            format!("dla ledger confirm {id} --evidence <growth or discriminator evidence id>"),
            true,
        ),
        FindingState::Strong => (
            format!("dla ledger strong {id} --evidence <id> (available from M6)"),
            false,
        ),
        FindingState::Refuted => (
            format!("dla ledger refute {id} --evidence <discriminator id>"),
            true,
        ),
        FindingState::Candidate => (
            format!("dla ledger revisit {id} (available in a later milestone)"),
            false,
        ),
    }
}

fn illegal(finding: &Finding, to: FindingState) -> Refusal {
    let from = finding.state;
    let targets = allowed_targets(from);
    let id = &finding.id;
    if targets.is_empty() {
        let root = id.parent().unwrap_or_else(|| id.clone());
        return Refusal::new(
            codes::ILLEGAL_TRANSITION,
            format!(
                "{id} is {from}, which is final: it never changes state again, so it cannot move to {to}"
            ),
            format!("a new hypothesis instead of changing {id}"),
            format!(
                "dla ledger add --origin hypothesis --evidence <evidence id> --sibling-of {root} --hypothesis <cause> --refute-if <result>"
            ),
        );
    }
    let list: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();
    let message = if from == to {
        format!(
            "{id} is already {from}; it can only move to: {}",
            list.join(", ")
        )
    } else {
        format!(
            "{id} cannot move from {from} to {to}; allowed: {}",
            list.join(", ")
        )
    };
    // Build the fix from the source state: commands that exist now first.
    let cmds: Vec<(String, bool)> = targets.iter().map(|t| edge_command(id, *t)).collect();
    let now: Vec<&str> = cmds
        .iter()
        .filter(|(_, ok)| *ok)
        .map(|(c, _)| c.as_str())
        .collect();
    let later: Vec<&str> = cmds
        .iter()
        .filter(|(_, ok)| !*ok)
        .map(|(c, _)| c.as_str())
        .collect();
    let fix = match (now.is_empty(), later.is_empty()) {
        (false, true) => now.join("   or   "),
        (false, false) => format!("{}   (later: {})", now.join("   or   "), later.join("; ")),
        (true, _) => later.join("   or   "),
    };
    Refusal::new(
        codes::ILLEGAL_TRANSITION,
        message,
        format!("a move from {from} to one of: {}", list.join(", ")),
        fix,
    )
}

fn resolve_fix(id: &crate::ids::FindingId, to: FindingState) -> String {
    let as_ = if to == FindingState::SuspectedStaticOnly {
        "suspected_static_only"
    } else {
        "unresolved"
    };
    format!("dla ledger resolve {id} --as {as_} --reason time_budget --remedy \"<text>\"")
}

/// The rules enforced in M0.
#[derive(Debug, Clone, Copy, Default)]
pub struct M0Rules;

impl TransitionRules for M0Rules {
    fn check(&self, ctx: &TransitionContext<'_>) -> Result<(), Refusal> {
        let id = &ctx.finding.id;
        for e in ctx.evidence {
            if ctx.lookup.evidence(e).is_none() {
                let known: Vec<String> = ctx
                    .lookup
                    .evidence_ids()
                    .iter()
                    .map(|k| k.to_string())
                    .collect();
                return Err(Refusal::new(
                    codes::UNKNOWN_EVIDENCE,
                    format!("transition of {id} cites {e}, which is not in the ledger"),
                    format!("evidence {e}"),
                    format!(
                        "cite an existing evidence ID (existing evidence: {})",
                        if known.is_empty() {
                            "none".to_owned()
                        } else {
                            known.join(", ")
                        }
                    ),
                ));
            }
        }
        let to = ctx.to;
        if to.needs_reason() {
            if ctx.reason.is_none() {
                return Err(Refusal::new(
                    codes::REASON_REQUIRED,
                    format!("{id} cannot become {to} without a reason"),
                    format!(
                        "--reason, one of: {}",
                        UnresolvedReason::ALL
                            .iter()
                            .map(|r| r.as_str())
                            .collect::<Vec<_>>()
                            .join("|")
                    ),
                    resolve_fix(id, to),
                ));
            }
            if ctx.remedy.is_none_or(|r| r.trim().is_empty()) {
                return Err(Refusal::new(
                    codes::REMEDY_REQUIRED,
                    format!("{id} cannot become {to} without a remedy"),
                    "--remedy, the step that would resolve it",
                    resolve_fix(id, to),
                ));
            }
        } else if ctx.reason.is_some() || ctx.remedy.is_some() {
            return Err(Refusal::new(
                codes::UNEXPECTED_REASON,
                format!(
                    "a reason or remedy only applies to unresolved and suspected_static_only, not {to}"
                ),
                "no --reason and no --remedy",
                format!("repeat the move for {id} without --reason and --remedy"),
            ));
        }
        match to {
            FindingState::SuspectedStaticOnly => {
                let has_static = ctx
                    .finding
                    .supports
                    .iter()
                    .chain(ctx.evidence)
                    .filter_map(|e| ctx.lookup.evidence(e))
                    .any(|e| e.kind.class() == EvidenceClass::Static);
                if !has_static {
                    return Err(Refusal::new(
                        codes::STATIC_EVIDENCE_REQUIRED,
                        format!(
                            "{id} cites no static-class evidence (kind static-hit or census-holder), so it cannot be suspected_static_only; run-config and other context records do not count"
                        ),
                        "an evidence record of kind static-hit or census-holder",
                        format!(
                            "dla ledger resolve {id} --as suspected_static_only --reason not_exercised --remedy \"<text>\" --evidence <static-hit or census-holder id>"
                        ),
                    ));
                }
            }
            FindingState::Refuted => {
                let has_disc = ctx
                    .finding
                    .contradicts
                    .iter()
                    .chain(ctx.evidence)
                    .filter_map(|e| ctx.lookup.evidence(e))
                    .any(|e| e.kind == EvidenceKind::Discriminator);
                if !has_disc {
                    return Err(Refusal::new(
                        codes::DISCRIMINATOR_REQUIRED,
                        format!(
                            "{id} cannot be refuted without discriminator evidence in contradicts"
                        ),
                        "an evidence record of kind discriminator",
                        format!("dla ledger refute {id} --evidence <discriminator evidence id>"),
                    ));
                }
            }
            // `under_test` and `confirmed` are judged by [`M3Rules`]; `strong` needs an
            // independent second evidence class (retention or intervention) and
            // arrives with M6. M0 enforces the graph only.
            FindingState::UnderTest | FindingState::Confirmed | FindingState::Strong => {}
            FindingState::Unresolved | FindingState::Candidate => {}
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------
// M3 rules
// ---------------------------------------------------------------------------------------

/// True when `e` is a growth-class record (`growth-series` or `unit-count-series`)
/// obtained by a test, a live observation or a dump whose `result.gate` is `pass`.
pub fn is_passing_growth(e: &EvidenceRecord) -> bool {
    matches!(
        e.kind,
        EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries
    ) && matches!(
        e.authority,
        Authority::Test | Authority::Live | Authority::Dump
    ) && e.result.get("gate").and_then(|g| g.as_str()) == Some("pass")
}

/// What the ledger derives for a finding when it is confirmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DerivedAttributes {
    /// The shape of the first passing column, when the finding has none.
    pub shape: Option<Shape>,
    /// The region of that column, when the finding has none (`process` derives none).
    pub region: Option<Region>,
}

/// The passing growth evidence among `finding.supports`, in order.
fn passing_growth<'a>(
    finding: &'a Finding,
    lookup: &'a dyn EvidenceLookup,
) -> impl Iterator<Item = &'a EvidenceRecord> {
    finding
        .supports
        .iter()
        .filter_map(|e| lookup.evidence(e))
        .filter(|e| is_passing_growth(e))
}

/// The passing columns of the finding's growth evidence, tied columns first.
fn passing_columns(
    finding: &Finding,
    lookup: &dyn EvidenceLookup,
) -> (
    Vec<crate::trend::PassedColumn>,
    Vec<crate::trend::PassedColumn>,
) {
    let mut tied = Vec::new();
    let mut other = Vec::new();
    for e in passing_growth(finding, lookup) {
        let Some(summary) = GrowthSummary::from_result(&e.result) else {
            continue;
        };
        for c in summary.passed {
            if c.tied_to(&finding.id, finding.site.as_deref()) {
                tied.push(c);
            } else {
                other.push(c);
            }
        }
    }
    (tied, other)
}

/// The shape and region a confirmation derives from the finding's cited growth
/// evidence: the first passing column that is tied to the finding (by its `finding`,
/// by an equal site or by `candidates`), else the first passing column. Nothing is
/// derived for a non-leak finding, nor for a value the finding already has.
pub fn derive_attributes(finding: &Finding, lookup: &dyn EvidenceLookup) -> DerivedAttributes {
    if finding.polarity() != Polarity::Leak {
        return DerivedAttributes::default();
    }
    let (tied, other) = passing_columns(finding, lookup);
    let Some(col) = tied.first().or(other.first()) else {
        return DerivedAttributes::default();
    };
    DerivedAttributes {
        shape: finding
            .shape
            .is_none()
            .then(|| col.shape.and_then(finding_shape))
            .flatten(),
        region: finding
            .region
            .is_none()
            .then(|| col.region.and_then(finding_region))
            .flatten(),
    }
}

/// The warning `dla ledger confirm` prints when none of the passing columns cited for a
/// leak-side finding is tied to it (by `finding`, by an equal site or by `candidates`).
pub fn confirm_warning(finding: &Finding, lookup: &dyn EvidenceLookup) -> Option<String> {
    if finding.polarity() != Polarity::Leak {
        return None;
    }
    let (tied, _) = passing_columns(finding, lookup);
    if !tied.is_empty() {
        return None;
    }
    let id = &finding.id;
    Some(format!(
        "no passing growth column cited for {id} is tied to it (by finding, site or candidates): the growth may belong to another holder; add a probe for {id} and rerun the harness if it does"
    ))
}

/// The rule set of milestone 3: [`M0Rules`] plus the dynamic evidence rules.
///
/// * `under_test`: a harness `workload` record is cited (or already supported); a
///   leak-side finding also needs a non-leak sibling in its family.
/// * `confirmed`, leak-side: growth evidence whose gate passed, a site, a holder and a
///   refuted non-leak member of the family.
/// * `confirmed`, non-leak: a discriminator favoring non-leak and a refuted leak-side
///   root.
/// * `refuted`: a discriminator whose verdict opposes the finding's polarity.
#[derive(Debug, Clone, Copy, Default)]
pub struct M3Rules;

impl TransitionRules for M3Rules {
    fn check(&self, ctx: &TransitionContext<'_>) -> Result<(), Refusal> {
        M0Rules.check(ctx)?;
        match ctx.to {
            FindingState::UnderTest => under_test(ctx),
            FindingState::Confirmed => match ctx.finding.polarity() {
                Polarity::Leak => confirm_leak(ctx),
                Polarity::NonLeak => confirm_nonleak(ctx),
            },
            FindingState::Refuted => refute(ctx),
            _ => Ok(()),
        }
    }
}

/// The evidence a move is judged on: what the finding supports plus what the move cites.
fn judged<'a, 'b>(ctx: &'b TransitionContext<'a>) -> impl Iterator<Item = &'b EvidenceRecord> {
    ctx.finding
        .supports
        .iter()
        .chain(ctx.evidence)
        .filter_map(|e| ctx.lookup.evidence(e))
}

fn under_test(ctx: &TransitionContext<'_>) -> Result<(), Refusal> {
    let id = &ctx.finding.id;
    let has_harness =
        judged(ctx).any(|e| e.kind == EvidenceKind::Workload && harness_id_of(&e.result).is_some());
    if !has_harness {
        return Err(Refusal::new(
            codes::HARNESS_REQUIRED,
            format!(
                "{id} cannot go under test without a harness: no cited workload evidence names one (a record from `dla harness gen` or `dla harness run`; `dla workload discover` does not count)"
            ),
            "a workload evidence record written by dla harness gen or dla harness run",
            format!(
                "dla harness gen --lang <lang> [--workload <id>]   then   dla ledger test {id} --evidence <harness evidence id>"
            ),
        ));
    }
    if ctx.finding.polarity() == Polarity::Leak {
        let root = ctx.finding.root_id();
        let has_sibling = ctx
            .findings
            .family(id)
            .iter()
            .any(|f| f.polarity() == Polarity::NonLeak);
        if !has_sibling {
            return Err(Refusal::new(
                codes::SIBLING_REQUIRED,
                format!(
                    "{id} needs a non-leak sibling hypothesis (N-ALLOC, N-HEADROOM, N-WARM, N-HWM, N-SCALE, N-INTENT or N-OBSERVER) before it goes under test, so that something can rule it out"
                ),
                format!("a sibling of {root} whose hypothesis begins with an N-* id"),
                format!(
                    "dla ledger add --sibling-of {root} --origin hypothesis --hypothesis N-WARM --refute-if \"<result>\" --evidence <id>"
                ),
            ));
        }
    }
    Ok(())
}

/// The series named by the first growth-class record the move judges, for fix text.
fn series_hint(ctx: &TransitionContext<'_>) -> String {
    judged(ctx)
        .filter(|e| {
            matches!(
                e.kind,
                EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries
            )
        })
        .find_map(|e| e.result.get("series")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| "<series>".to_owned())
}

fn confirm_leak(ctx: &TransitionContext<'_>) -> Result<(), Refusal> {
    let id = &ctx.finding.id;
    if !judged(ctx).any(is_passing_growth) {
        return Err(Refusal::new(
            codes::GROWTH_GATE_FAILED,
            format!(
                "{id} cannot be confirmed: no cited growth-series or unit-count-series evidence (authority test, live or dump) has a passing gate"
            ),
            "growth evidence with result.gate = pass",
            format!(
                "dla trend {}   then   dla ledger confirm {id} --evidence <trend evidence id>",
                series_hint(ctx)
            ),
        ));
    }
    if ctx.finding.site.is_none() {
        return Err(Refusal::new(
            codes::SITE_REQUIRED,
            format!("{id} cannot be confirmed without a site (file:line)"),
            "a site on the finding",
            "a finding's site is set when it is added: dla ledger add --origin <origin> --evidence <id> --site <file:line> --holder <holder>   (then test and confirm that finding)",
        ));
    }
    if ctx.finding.holder.is_none() {
        return Err(Refusal::new(
            codes::HOLDER_REQUIRED,
            format!("{id} cannot be confirmed without a holder"),
            "a holder on the finding",
            format!("dla ledger annotate {id} --holder <holder> [--owner <text>]"),
        ));
    }
    let family = ctx.findings.family(id);
    let nonleak: Vec<&Finding> = family
        .iter()
        .copied()
        .filter(|f| f.polarity() == Polarity::NonLeak)
        .collect();
    if !nonleak.iter().any(|f| f.state == FindingState::Refuted) {
        let root = ctx.finding.root_id();
        let sibling = nonleak
            .first()
            .map_or_else(|| format!("{root}.n1"), |f| f.id.to_string());
        return Err(Refusal::new(
            codes::NONLEAK_REFUTATION_REQUIRED,
            format!(
                "{id} cannot be confirmed while no non-leak sibling is refuted: the alternatives (N-*) must be ruled out first"
            ),
            format!("a refuted N-* sibling of {root}"),
            format!(
                "dla alloc-test <harness> --finding {root}   then   dla ledger refute {sibling} --evidence <discriminator id>"
            ),
        ));
    }
    Ok(())
}

fn confirm_nonleak(ctx: &TransitionContext<'_>) -> Result<(), Refusal> {
    let id = &ctx.finding.id;
    let root_id = ctx.finding.root_id();
    check_direction(ctx, judged(ctx), Favors::NonLeak, "confirm")?;
    let root_refuted = ctx
        .findings
        .finding(&root_id)
        .is_some_and(|r| r.state == FindingState::Refuted);
    if !root_refuted {
        return Err(Refusal::new(
            codes::LEAK_REFUTATION_REQUIRED,
            format!(
                "{id} cannot be confirmed while its leak-side finding {root_id} is not refuted"
            ),
            format!("{root_id} refuted"),
            format!("dla ledger refute {root_id} --evidence <discriminator id favoring non-leak>"),
        ));
    }
    Ok(())
}

fn refute(ctx: &TransitionContext<'_>) -> Result<(), Refusal> {
    let want = match ctx.finding.polarity() {
        Polarity::Leak => Favors::NonLeak,
        Polarity::NonLeak => Favors::Leak,
    };
    let cited = ctx
        .finding
        .contradicts
        .iter()
        .chain(ctx.evidence)
        .filter_map(|e| ctx.lookup.evidence(e));
    check_direction(ctx, cited, want, "refute")
}

/// Requires a discriminator among `records` that favors `want` and, when it names a
/// finding, names one in the same family.
fn check_direction<'a>(
    ctx: &TransitionContext<'_>,
    records: impl Iterator<Item = &'a EvidenceRecord>,
    want: Favors,
    verb: &str,
) -> Result<(), Refusal> {
    let id = &ctx.finding.id;
    let family: Vec<FindingId> = ctx
        .findings
        .family(id)
        .iter()
        .map(|f| f.id.clone())
        .collect();
    let mut seen = Vec::new();
    for e in records.filter(|e| e.kind == EvidenceKind::Discriminator) {
        let favors = favors_of(e);
        let named = finding_of(e);
        if favors == Some(want) && named.as_ref().is_none_or(|g| family.contains(g)) {
            return Ok(());
        }
        let favors = favors.map_or("no valid favors", |f| f.as_str());
        match named {
            Some(g) if !family.contains(&g) => seen.push(format!(
                "{} is bound to {g}, outside the family of {id}",
                e.evidence_id
            )),
            _ => seen.push(format!("{} favors: {favors}", e.evidence_id)),
        }
    }
    let root = ctx.finding.root_id();
    let polarity = match ctx.finding.polarity() {
        Polarity::Leak => "leak-side",
        Polarity::NonLeak => "non-leak",
    };
    let (code, what) = if seen.is_empty() {
        (
            codes::DISCRIMINATOR_REQUIRED,
            "cites no discriminator".to_owned(),
        )
    } else {
        (
            codes::DISCRIMINATOR_DIRECTION,
            format!(
                "cites only discriminators that do not qualify ({})",
                seen.join("; ")
            ),
        )
    };
    Err(Refusal::new(
        code,
        format!(
            "{id} is {polarity}, so to {verb} it needs a discriminator that favors {want} and names no finding outside its family, but it {what}"
        ),
        format!("a discriminator with result.favors = {want}"),
        format!(
            "dla alloc-test <harness> --finding {root}   then   dla ledger {verb} {id} --evidence <discriminator id>"
        ),
    ))
}
