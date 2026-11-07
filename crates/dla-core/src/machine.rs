//! The finding state machine.
//!
//! [`allowed_targets`] is the whole graph. [`check_transition`] validates a move:
//! first the graph, then a [`TransitionRules`] implementation that holds the
//! evidence rules attached to edges. M0 ships [`M0Rules`].

use crate::{
    error::{Refusal, codes},
    evidence::{Authority, EvidenceKind, EvidenceRecord},
    finding::{Finding, FindingState, UnresolvedReason},
    ids::EvidenceId,
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
}

/// Evidence rules attached to edges of the graph.
///
/// Extension point: later milestones implement this trait (or extend
/// [`M0Rules`]) to add the rules for `under_test`, `confirmed` and `strong`.
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

fn illegal(finding: &Finding, to: FindingState) -> Refusal {
    let from = finding.state;
    let targets = allowed_targets(from);
    let id = &finding.id;
    if targets.is_empty() {
        return Refusal::new(
            codes::ILLEGAL_TRANSITION,
            format!("{id} is {from}, which is terminal; it cannot move to {to}"),
            format!("a non-terminal state for {id}"),
            "dla ledger add --origin hypothesis --evidence E1 --sibling-of <finding> --hypothesis <cause> --refute-if <result>",
        );
    }
    let list: Vec<&str> = targets.iter().map(|s| s.as_str()).collect();
    Refusal::new(
        codes::ILLEGAL_TRANSITION,
        format!(
            "{id} cannot move from {from} to {to}; allowed: {}",
            list.join(", ")
        ),
        format!("a move from {from} to one of: {}", list.join(", ")),
        format!("dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\""),
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
                return Err(Refusal::new(
                    codes::UNKNOWN_EVIDENCE,
                    format!("transition of {id} cites {e}, which is not in the ledger"),
                    format!("evidence {e}"),
                    "cite an existing evidence ID; `dla run status` lists the ledger",
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
                    .any(|e| e.authority == Authority::Static);
                if !has_static {
                    return Err(Refusal::new(
                        codes::STATIC_EVIDENCE_REQUIRED,
                        format!(
                            "{id} cites no static evidence, so it cannot be suspected_static_only"
                        ),
                        "an evidence record with authority static",
                        format!(
                            "dla ledger resolve {id} --as suspected_static_only --reason not_exercised --remedy \"<text>\" --evidence <static evidence id>"
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
            // Extension point (later milestones): `under_test` needs a sibling
            // N-* hypothesis; `confirmed` needs growth passing the slope gate, a
            // site, a holder and a refuted N-* sibling; `strong` needs an
            // independent second evidence class (retention or intervention).
            // M0 enforces the graph only.
            FindingState::UnderTest | FindingState::Confirmed | FindingState::Strong => {}
            FindingState::Unresolved | FindingState::Candidate => {}
        }
        Ok(())
    }
}
