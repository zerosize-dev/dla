//! Growth the run measured and has not decided.
//!
//! A passed growth column of a harness series points at a finding: the finding the column was
//! probed for, else the first candidate of its site (the choice `dla trend` makes when it prints
//! `dla ledger add --sibling-of`). Until an alloc-test evidence bound to that finding exists the
//! finding is not closed unmeasured: `close-rest` leaves it, `ledger resolve` refuses to close it,
//! and the final gate lists it with the commands that decide it. A runtime detector hit that
//! names a site is a lead of the same kind while nothing measured the finding at that site.
//!
//! Whole-process columns tie to no finding and stay with the residual rows.

use crate::{
    evidence::EvidenceKind,
    finding::{FindingState, Polarity},
    gate::is_whole_process,
    ids::FindingId,
    series::Unit,
    trend::Verdict,
    view::{BlockingItem, BlockingKind, RunView},
};
use dla_stats::GateVerdict;

/// The words the refutation condition of the non-leak sibling carries.
pub const REFUTE_IF: &str = "growth continues over the extra cycles";

/// A finding whose growth was measured and not decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grown {
    /// The root finding.
    pub finding: FindingId,
    /// Its state.
    pub state: FindingState,
    /// The harness of the series.
    pub harness: String,
    /// The series.
    pub series: String,
    /// The passed column that points at it.
    pub column: String,
    /// The growth evidence `dla trend` wrote for the column.
    pub growth: Option<String>,
    /// The latest `workload` evidence of the harness.
    pub workload: Option<String>,
    /// Whether the finding has a non-leak sibling.
    pub has_sibling: bool,
}

impl Grown {
    /// The commands that decide it, in order: the non-leak sibling when it is missing, the test
    /// (the ledger lets a candidate and a closed `unresolved` or `suspected_static_only`
    /// finding go under test), the alloc-test.
    pub fn commands(&self) -> Vec<String> {
        commands(
            &self.finding,
            &self.harness,
            self.growth.as_deref(),
            self.workload.as_deref(),
            self.has_sibling,
            matches!(
                self.state,
                FindingState::Candidate
                    | FindingState::Unresolved
                    | FindingState::SuspectedStaticOnly
            ),
        )
    }
}

/// The commands that decide the growth of `finding` measured by harness `harness`.
pub fn commands(
    finding: &FindingId,
    harness: &str,
    growth: Option<&str>,
    workload: Option<&str>,
    has_sibling: bool,
    can_test: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    if !has_sibling {
        out.push(format!(
            "dla ledger add --sibling-of {finding} --origin hypothesis --hypothesis N-WARM --refute-if \"{REFUTE_IF}\" --evidence {}",
            growth.unwrap_or("<growth evidence of dla trend>")
        ));
    }
    if can_test {
        out.push(format!(
            "dla ledger test {finding} --evidence {}",
            workload.unwrap_or("<workload evidence of the harness>")
        ));
    }
    out.push(format!("dla alloc-test {harness} --finding {finding}"));
    out
}

/// True when an alloc-test evidence is bound to the family of `root`.
fn alloc_tested(view: &RunView, root: &FindingId) -> bool {
    let family: Vec<FindingId> = view
        .ledger
        .findings()
        .filter(|f| &f.id == root || f.parent.as_ref() == Some(root))
        .map(|f| f.id.clone())
        .collect();
    view.ledger.evidence().any(|e| {
        e.kind == EvidenceKind::Discriminator
            && e.result.get("test").and_then(|t| t.as_str()) == Some("plateau+purge")
            && crate::discriminator::finding_of(e).is_some_and(|f| family.contains(&f))
    })
}

/// Findings a passed growth column of a harness series points at that no alloc-test evidence is
/// bound to, in the order of the trends. Only open findings and findings closed `unresolved` or
/// `suspected_static_only` count: the others have been decided.
pub fn grown_unmeasured(view: &RunView) -> Vec<Grown> {
    let mut out: Vec<Grown> = Vec::new();
    for t in &view.trends {
        if view.gap_series.contains(&t.series) {
            continue;
        }
        let Some(harness) = view.series_harness.get(&t.series) else {
            continue;
        };
        for c in t
            .columns
            .iter()
            .filter(|c| c.gate == GateVerdict::Pass && c.unit != Unit::Seconds)
            .filter(|c| !is_whole_process(&c.name))
        {
            let Some(f) = c
                .finding
                .iter()
                .chain(c.candidates.iter())
                .find_map(|f| view.ledger.finding(f))
            else {
                continue;
            };
            let root = f.root_id();
            let Some(rf) = view.ledger.finding(&root) else {
                continue;
            };
            if rf.polarity() != Polarity::Leak
                || root.is_sibling()
                || view.ledger.merged_into(&root).is_some()
                || !matches!(
                    rf.state,
                    FindingState::Candidate
                        | FindingState::Unresolved
                        | FindingState::SuspectedStaticOnly
                )
                || out.iter().any(|g| g.finding == root)
                || alloc_tested(view, &root)
            {
                continue;
            }
            let want = match c.kind {
                crate::series::ColumnKind::Gauge => EvidenceKind::GrowthSeries,
                crate::series::ColumnKind::Counter => EvidenceKind::UnitCountSeries,
            };
            out.push(Grown {
                finding: root.clone(),
                state: rf.state,
                harness: harness.clone(),
                series: t.series.clone(),
                column: c.name.clone(),
                growth: view
                    .ledger
                    .evidence()
                    .filter(|e| e.kind == want)
                    .find(|e| e.result.get("trend").and_then(|v| v.as_str()) == Some(t.id.as_str()))
                    .map(|e| e.evidence_id.to_string()),
                workload: view
                    .ledger
                    .evidence()
                    .filter(|e| e.kind == EvidenceKind::Workload)
                    .filter(|e| {
                        e.result.get("harness").and_then(|v| v.as_str()) == Some(harness.as_str())
                    })
                    .last()
                    .map(|e| e.evidence_id.to_string()),
                has_sibling: view
                    .ledger
                    .findings()
                    .any(|s| s.parent.as_ref() == Some(&root) && s.polarity() == Polarity::NonLeak),
            });
        }
    }
    out
}

/// A runtime detector hit that names a site, with nothing yet measuring the holder there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lead {
    /// The registration site of the hit.
    pub site: String,
    /// The finding at the site, when there is one.
    pub finding: Option<FindingId>,
    /// The harness whose run saw it.
    pub harness: String,
    /// Its series.
    pub series: String,
}

/// The detector hits with a site of the run's harness series that nothing decided: the finding
/// at the site (or, with none, the hit itself) has no measurement verdict (`grew`, `changed`,
/// `constant` or `executed` in a trend) and no alloc-test evidence. A finding that a passed
/// column points at is in [`grown_unmeasured`] and is left out here.
pub fn detector_leads(view: &RunView, grown: &[Grown]) -> Vec<Lead> {
    let Some(run_dir) = view.run_dir.as_deref() else {
        return Vec::new();
    };
    let mut out: Vec<Lead> = Vec::new();
    for (series, harness) in &view.series_harness {
        let Some(rec) = crate::harness::run_record_of(run_dir, harness, series) else {
            continue;
        };
        for site in rec.detectors.iter().filter_map(|d| {
            d["site"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
        }) {
            if out.iter().any(|l| l.site == site) {
                continue;
            }
            let finding = view
                .ledger
                .findings()
                .find(|f| {
                    f.site.as_deref() == Some(site.as_str())
                        && f.polarity() == Polarity::Leak
                        && !f.id.is_sibling()
                        && view.ledger.merged_into(&f.id).is_none()
                })
                .map(|f| f.id.clone());
            if let Some(f) = &finding {
                let state = view.ledger.finding(f).map(|x| x.state);
                let decided = matches!(
                    state,
                    Some(
                        FindingState::UnderTest
                            | FindingState::Confirmed
                            | FindingState::Strong
                            | FindingState::Refuted
                    )
                );
                let measured = view.trends.iter().any(|t| {
                    t.targets.iter().any(|v| {
                        &v.finding == f
                            && matches!(
                                v.verdict,
                                Verdict::Grew
                                    | Verdict::Changed
                                    | Verdict::Constant
                                    | Verdict::Executed
                            )
                    })
                });
                if decided || measured || grown.iter().any(|g| &g.finding == f) || alloc_tested(view, f)
                {
                    continue;
                }
            }
            out.push(Lead {
                site,
                finding,
                harness: harness.clone(),
                series: series.clone(),
            });
        }
    }
    out
}

/// The `growth` items of the status: a finding a passed column points at that is closed
/// unmeasured (an open one is listed as a `finding` item, whose fix these commands replace),
/// and a detector lead. `grown` is [`grown_unmeasured`] of the view.
pub fn items(view: &RunView, grown: &[Grown]) -> Vec<BlockingItem> {
    let mut out = Vec::new();
    for g in grown.iter().filter(|g| g.state != FindingState::Candidate) {
        out.push(BlockingItem {
            kind: BlockingKind::Growth,
            id: g.finding.to_string(),
            description: format!(
                "{} is closed {} but its column {} of {} passed the growth gate and no alloc-test decided it",
                g.finding, g.state, g.column, g.series
            ),
            fix: g.commands().join("   then   "),
        });
    }
    for l in detector_leads(view, grown) {
        let runtime = view
            .series_harness
            .get(&l.series)
            .and_then(|h| {
                view.run_dir
                    .as_deref()
                    .and_then(|d| crate::harness::HarnessManifest::load(d, h).ok())
            })
            .map(|m| m.runtime.as_str().to_owned());
        let (id, fix) = match (&l.finding, runtime) {
            (Some(f), Some(rt)) => (
                f.to_string(),
                format!(
                    "dla harness gen --lang {rt} --driver --probe {f}   # {f} holds it, {} did not measure it; copy the hooks of {}",
                    l.harness, l.harness
                ),
            ),
            (Some(f), None) => (f.to_string(), format!("measure {f}: dla harness gen --driver --probe {f}")),
            (None, _) => (
                l.site.clone(),
                format!(
                    "dla ledger add --origin growth-series --evidence <growth evidence of {}> --site {} --holder registration-table   # no finding there",
                    l.series, l.site
                ),
            ),
        };
        out.push(BlockingItem {
            kind: BlockingKind::Growth,
            id,
            description: format!(
                "a runtime detector hit of {} ({}) names {}: {} has no measurement verdict",
                l.harness,
                l.series,
                l.site,
                l.finding
                    .as_ref()
                    .map_or("the hit".to_owned(), ToString::to_string)
            ),
            fix,
        });
    }
    out
}
