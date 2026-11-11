//! Duplicates of one leak: the rule behind `dla ledger duplicate F --of G`.
//!
//! Several findings can be raised for one leak (a detector finding, the census finding and
//! the growth-series findings of a harness). The cross-worker merge folds findings of
//! different workers that share a site and a holder; this rule serves the findings of one
//! scope. The tie rule allows exact identity only, so two leak-side top-level findings are
//! duplicates when both hold:
//!
//! - they are in one scope: the same worker (or both in the core ledger) and, when both
//!   sites are known, the same runtime by the language of the site;
//! - they have the same canonical owner (equality, not a dotted suffix), or the same site
//!   `file:line`.
//!
//! A finding without an owner and a site is a duplicate of nothing.

use crate::{
    archetypes::runtime_of_file,
    attribution::canonical,
    error::Refusal,
    finding::{Finding, FindingState, Polarity},
    ids::FindingId,
    ledger::Ledger,
    m6::codes,
};

/// Why two findings are duplicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateBy {
    /// The owners name one field.
    Owner(String, String),
    /// The sites are the same `file:line`.
    Site(String, String),
}

impl DuplicateBy {
    /// One line for the answer of the command.
    pub fn describe(&self) -> String {
        match self {
            Self::Owner(a, b) => format!("same owner ({a} / {b})"),
            Self::Site(a, b) => format!("same site ({a} / {b})"),
        }
    }
}

/// Whether two owners are the same field: equal after the census forms are made one
/// ([`canonical`]). Both must be qualified (a class or module part): a bare name matches
/// nothing, and a dotted suffix is not enough.
fn same_owner(a: &str, b: &str) -> bool {
    matches!((canonical(a.trim()), canonical(b.trim())), (Some(x), Some(y)) if x == y)
}

fn site_runtime(f: &Finding) -> Option<&'static str> {
    f.site
        .as_deref()
        .and_then(|s| s.rsplit_once(':').map(|(file, _)| file))
        .and_then(runtime_of_file)
}

/// Whether `f` and `g` are in one scope: the same worker (or both core) and, when both
/// sites name a language, the same runtime.
pub fn same_scope(f: &Finding, g: &Finding) -> bool {
    f.id.prefix() == g.id.prefix()
        && match (site_runtime(f), site_runtime(g)) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        }
}

/// How `f` and `g` are duplicates, or `None`: the same scope and the same canonical
/// owner, or the same site. Owners first. Symmetric.
pub fn duplicate_by(f: &Finding, g: &Finding) -> Option<DuplicateBy> {
    if !same_scope(f, g) {
        return None;
    }
    if let (Some(a), Some(b)) = (&f.owner, &g.owner)
        && same_owner(a, b)
    {
        return Some(DuplicateBy::Owner(a.clone(), b.clone()));
    }
    if let (Some(a), Some(b)) = (&f.site, &g.site)
        && a.trim() == b.trim()
    {
        return Some(DuplicateBy::Site(a.clone(), b.clone()));
    }
    None
}

fn is_hypothesis(f: &Finding) -> bool {
    f.parent.is_some() || f.hypothesis.is_some() || f.origin.as_str() == "hypothesis"
}

/// Why `f` cannot be one side of a duplicate pair; `None` when it can.
fn side_gap(f: &Finding, role: &str) -> Option<String> {
    if f.id.is_sibling() || is_hypothesis(f) {
        return Some(format!(
            "{} is a hypothesis, not a finding of the leak ({role})",
            f.id
        ));
    }
    if f.polarity() != Polarity::Leak {
        return Some(format!("{} is not leak-side ({role})", f.id));
    }
    None
}

/// The refusal for `duplicate` of `dup` and `canonical`, or how they are duplicates.
pub fn check(
    ledger: &Ledger,
    dup: &FindingId,
    canonical: &FindingId,
) -> Result<DuplicateBy, Refusal> {
    let state = |why: String, fix: String| {
        Refusal::new(
            codes::DUPLICATE_STATE,
            why,
            "two distinct leak-side findings, neither a hypothesis, the first not merged yet",
            fix,
        )
    };
    let get = |id: &FindingId| {
        ledger.finding(id).ok_or_else(|| {
            Refusal::new(
                crate::error::codes::UNKNOWN_FINDING,
                format!("{id} does not exist in the ledger"),
                format!("finding {id}"),
                "dla run status",
            )
        })
    };
    let f = get(dup)?;
    let g = get(canonical)?;
    if dup == canonical {
        return Err(state(
            format!("{dup} cannot be a duplicate of itself"),
            format!("dla ledger duplicate {dup} --of <the other finding of the leak>"),
        ));
    }
    if let Some(why) = side_gap(f, "duplicate").or_else(|| side_gap(g, "--of")) {
        return Err(state(why, "dla run status".to_owned()));
    }
    if let Some(c) = ledger.merged_into(dup) {
        return Err(state(
            format!("{dup} is already merged into {c}"),
            format!("dla run status   ({c} is its canonical finding)"),
        ));
    }
    if let Some(c) = ledger.merged_into(canonical) {
        let fix = if c == dup {
            format!(
                "dla run status   ({dup} is already the canonical finding of {canonical}; nothing to merge)"
            )
        } else {
            format!("dla ledger duplicate {dup} --of {c}")
        };
        return Err(state(
            format!("{canonical} is itself a duplicate of {c}"),
            fix,
        ));
    }
    if let Some(w) = ledger.prefix()
        && (dup.prefix() != Some(w) || canonical.prefix() != Some(w))
    {
        return Err(state(
            format!("{dup} and {canonical} are not both findings of worker {w}"),
            "dla --worker <name> ledger duplicate <own finding> --of <own finding>".to_owned(),
        ));
    }
    if f.state == FindingState::Refuted || g.state == FindingState::Refuted {
        return Err(state(
            "a refuted finding is a verdict, not a duplicate".to_owned(),
            "dla run status".to_owned(),
        ));
    }
    duplicate_by(f, g).ok_or_else(|| {
        Refusal::new(
            codes::DUPLICATE_NOT_SAME,
            format!(
                "{dup} (owner {}, site {}) is not a duplicate of {canonical} (owner {}, site {}): the owners are not the same and the sites are not the same file:line (or the two are in different workers or runtimes)",
                f.owner.as_deref().unwrap_or("none"),
                f.site.as_deref().unwrap_or("none"),
                g.owner.as_deref().unwrap_or("none"),
                g.site.as_deref().unwrap_or("none"),
            ),
            "the same owner or the same site file:line, both findings in one worker and runtime",
            format!(
                "if they are one leak, record the owner or site they share (dla ledger annotate {dup} --owner <owner> --evidence <E>); if they are two leaks, label each: dla ledger label {dup} --suggest"
            ),
        )
    })
}

/// The labelled findings `f` is a duplicate of, in ID order: leak-side top-level findings
/// that carry a label decision, are not merged themselves and share an owner or a site
/// with `f`.
pub fn labelled_twins<'a>(ledger: &'a Ledger, f: &Finding) -> Vec<&'a Finding> {
    ledger
        .findings()
        .filter(|g| g.id != f.id)
        .filter(|g| matches!(g.state, FindingState::Confirmed | FindingState::Strong))
        .filter(|g| side_gap(g, "").is_none())
        .filter(|g| ledger.prefix().is_none() || g.id.prefix() == f.id.prefix())
        .filter(|g| ledger.merged_into(&g.id).is_none())
        .filter(|g| ledger.label_record(&g.id).is_some())
        .filter(|g| duplicate_by(f, g).is_some())
        .collect()
}
