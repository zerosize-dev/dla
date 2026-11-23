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
    /// The harness is a pytest loop (the project's own tests): an alloc-test on it re-runs the
    /// whole test suite per run.
    pub pytest_loop: bool,
    /// Whether the finding has a non-leak sibling.
    pub has_sibling: bool,
    /// A finding of the same harness whose alloc-test favours leak and that this finding is a
    /// duplicate of (the same owner or the same site, as `dla ledger duplicate` requires): the
    /// growth may be recorded as part of that finding instead of being tested again.
    pub same_leak: Option<(FindingId, String)>,
}

impl Grown {
    /// The commands that decide it, in order: the non-leak sibling when it is missing, the test
    /// (the ledger lets a candidate and a closed `unresolved` or `suspected_static_only`
    /// finding go under test), the alloc-test.
    pub fn commands(&self) -> Vec<String> {
        let own = self.own_commands();
        match &self.same_leak {
            Some((g, by)) => vec![format!(
                "dla ledger duplicate {} --of {g}   ({by}: the same leak as {g}, whose alloc-test on {} favored leak; if it is a leak of its own: {})",
                self.finding,
                self.harness,
                own.join("   then   ")
            )],
            None => own,
        }
    }

    /// The commands that decide it by its own alloc-test.
    fn own_commands(&self) -> Vec<String> {
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
            self.pytest_loop,
        )
    }
}

/// The commands that decide the growth of `finding` measured by harness `harness`. When the
/// series comes from a pytest loop (`pytest_loop`), the alloc-test is not run on that harness
/// first: it re-runs the whole test suite per run (minutes), so a small driver of its own for
/// the line comes first (seconds per run), and the alloc-test is on that harness.
pub fn commands(
    finding: &FindingId,
    harness: &str,
    growth: Option<&str>,
    workload: Option<&str>,
    has_sibling: bool,
    can_test: bool,
    pytest_loop: bool,
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
    if pytest_loop {
        out.push(format!(
            "dla harness gen --lang python --driver --probe {finding}   (a small driver of its own for the line: a unit that calls the function of that line; a run takes seconds, where an alloc-test on {harness} re-runs the whole test suite per run, minutes)"
        ));
        out.push("dla harness run <H>".to_owned());
        out.push("dla trend <S>".to_owned());
        out.push(format!(
            "dla alloc-test <H> --finding {finding}   (on the harness just made, not on {harness})"
        ));
    } else {
        out.push(format!("dla alloc-test {harness} --finding {finding}"));
    }
    out
}

/// True for an allocation-site count column of a Python driver run (`py.blocks@<file:line>`):
/// it counts what one line allocated, so only a finding at that line is tied to it.
pub fn is_alloc_site_column(column: &str) -> bool {
    column
        .strip_suffix(".pre_gc")
        .unwrap_or(column)
        .starts_with("py.blocks@")
}

/// The command that adds the finding a passing allocation-site count column of a Python driver
/// run (`py.blocks@<file:line>`) ties to, when no finding is at its site. `None` for any other
/// column. A leak of a Python object that nothing holds has no census candidate, so the column
/// is where the finding starts; the holder is read from the line.
pub fn site_column_add(column: &str, site: &str, evidence: &str) -> Option<String> {
    is_alloc_site_column(column).then(|| {
        format!(
            "dla ledger add --origin growth-series --evidence {evidence} --site {site} --region heap --holder <kind>   # no finding there; {column} then ties to it. Holder: read the line: a container of the project that keeps the objects (static/global, container-field, registration-table), or native-handle/global-ref when the line calls into an extension module or a C function and nothing in the program holds the result"
        )
    })
}

/// True when an alloc-test evidence is bound to the family of `root`.
pub(crate) fn alloc_tested(view: &RunView, root: &FindingId) -> bool {
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

/// True when the leak-side finding `f` is at `site` (`file:line`) for the tie of an
/// allocation-site count column: its own `site` is that line, or the census annotation of the
/// finding lists the line among the writers of its holder (`writer_sites`, and the plain
/// `writers` list). The column of a line that registers a new key per call counts what that
/// line keeps, so it belongs to the finding of the table it writes into, whose own site is the
/// line that defines the table. A non-leak finding is never at a site, and removers never count.
pub fn finding_is_at_site(ledger: &crate::ledger::Ledger, f: &crate::finding::Finding, site: &str) -> bool {
    let site = site.trim();
    if f.polarity() != Polarity::Leak {
        return false;
    }
    if f.site.as_deref().map(str::trim) == Some(site) {
        return true;
    }
    ledger.annotation(&f.id).is_some_and(|a| {
        a.writers.iter().any(|w| w.trim() == site)
            || a.facts.writer_sites.iter().any(|w| w.site.trim() == site)
    })
}

/// The `file:line` a column is tied to: its `site`, else the part of its name after `@`.
pub fn column_site(c: &crate::trend::ColumnTrend) -> Option<&str> {
    c.site
        .as_deref()
        .or_else(|| c.name.split_once('@').map(|(_, s)| s))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// A finding of `harness` that an alloc-test (on that harness) found to favour leak, that is
/// confirmed, strong or under test with that evidence, and that `f` is a duplicate of by
/// `dla ledger duplicate`'s own rule. The growth of `f` is then another site of that leak.
fn same_leak_as(view: &RunView, f: &FindingId, harness: &str) -> Option<(FindingId, String)> {
    view.ledger
        .evidence()
        .filter(|e| {
            e.kind == EvidenceKind::Discriminator
                && e.result.get("test").and_then(|t| t.as_str()) == Some("plateau+purge")
                && e.result.get("favors").and_then(|t| t.as_str()) == Some("leak")
                && (e.result.get("harness").and_then(|t| t.as_str()) == Some(harness))
        })
        .filter_map(crate::discriminator::finding_of)
        .map(|g| view.ledger.finding(&g).map_or(g.clone(), |x| x.root_id()))
        .filter(|g| g != f)
        .filter(|g| {
            view.ledger.finding(g).is_some_and(|x| {
                x.polarity() == Polarity::Leak
                    && matches!(
                        x.state,
                        FindingState::Confirmed | FindingState::Strong | FindingState::UnderTest
                    )
            })
        })
        .find_map(|g| {
            crate::duplicate::check(&view.ledger, f, &g)
                .ok()
                .map(|by| (g, by.describe()))
        })
}

/// Findings a passed growth column of a harness series points at, or that a trend gave the
/// verdict `grew`, that no alloc-test evidence is bound to, in the order of the trends. Only open findings and findings closed `unresolved` or
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
        // (finding, column, kind of its growth evidence) of every column that passed and points
        // at a finding, then every target the trend gave the verdict `grew`.
        let mut pointed: Vec<(FindingId, String, crate::series::ColumnKind)> = Vec::new();
        for c in t
            .columns
            .iter()
            .filter(|c| c.gate == GateVerdict::Pass && c.unit != Unit::Seconds)
            .filter(|c| !is_whole_process(&c.name))
        {
            if let Some(f) = c
                .finding
                .iter()
                .chain(c.candidates.iter())
                .find(|f| view.ledger.finding(f).is_some())
            {
                pointed.push((f.clone(), c.name.clone(), c.kind));
            } else if let Some(f) = column_site(c).and_then(|site| {
                // The trend lists the findings that existed when it ran; a finding added
                // after it (from this very column) is the one at the column's own site.
                view.ledger
                    .findings()
                    .find(|f| {
                        finding_is_at_site(&view.ledger, f, site) && !f.id.is_sibling()
                    })
                    .map(|f| f.id.clone())
            }) {
                pointed.push((f, c.name.clone(), c.kind));
            }
        }
        for v in t.targets.iter().filter(|v| v.verdict == Verdict::Grew) {
            let column = v
                .column
                .clone()
                .unwrap_or_else(|| "(a passed column)".into());
            let kind = t
                .columns
                .iter()
                .find(|c| Some(&c.name) == v.column.as_ref())
                .map_or(crate::series::ColumnKind::Gauge, |c| c.kind);
            pointed.push((v.finding.clone(), column, kind));
        }
        for (f, column, ckind) in pointed {
            let Some(f) = view.ledger.finding(&f) else {
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
                        | FindingState::UnderTest
                        | FindingState::Unresolved
                        | FindingState::SuspectedStaticOnly
                )
                || out.iter().any(|g| g.finding == root)
                || alloc_tested(view, &root)
            {
                continue;
            }
            let want = match ckind {
                crate::series::ColumnKind::Gauge => EvidenceKind::GrowthSeries,
                crate::series::ColumnKind::Counter => EvidenceKind::UnitCountSeries,
            };
            out.push(Grown {
                finding: root.clone(),
                state: rf.state,
                harness: harness.clone(),
                series: t.series.clone(),
                column,
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
                pytest_loop: view.run_dir.as_deref().is_some_and(|d| {
                    crate::harness::HarnessManifest::load(d, harness)
                        .is_ok_and(|m| m.is_pytest_loop())
                }),
                has_sibling: view
                    .ledger
                    .findings()
                    .any(|s| s.parent.as_ref() == Some(&root) && s.polarity() == Polarity::NonLeak),
                same_leak: same_leak_as(view, &root, harness),
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
                if decided
                    || measured
                    || grown.iter().any(|g| &g.finding == f)
                    || alloc_tested(view, f)
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
    // An open finding (candidate, under test) is listed as a `finding` item with these commands.
    for g in grown.iter().filter(|g| g.state.is_dispositioned()) {
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
            (Some(f), None) => (
                f.to_string(),
                format!("measure {f}: dla harness gen --driver --probe {f}"),
            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FindingAnnotation, Ledger, NewEvidence, NewFinding, Origin,
        census::{HolderFacts, SiteFact, SiteKind},
        evidence::{Authority, EvidenceCoverage, Source, Tool},
    };

    /// A ledger with F1 at `a.py:1` whose plain writer list has `a.py:6`, F2 at `a.py:10`
    /// whose census facts have the writer site `a.py:12`, F3 at `a.py:20` with no annotation,
    /// and F1.n1, a non-leak sibling of F1 whose own writer list has `a.py:30`.
    fn scene() -> (tempfile::TempDir, Ledger) {
        let dir = tempfile::tempdir().unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        l.add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: Authority::Static,
            kind: EvidenceKind::StaticHit,
            result: serde_json::json!({}),
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        for site in ["a.py:1", "a.py:10", "a.py:20"] {
            let mut nf =
                NewFinding::new(Origin::parse("census").unwrap(), vec![crate::EvidenceId::new(1)]);
            nf.site = Some(site.into());
            l.add_finding(nf).unwrap();
        }
        let mut nf = NewFinding::new(
            Origin::parse("hypothesis").unwrap(),
            vec![crate::EvidenceId::new(1)],
        );
        nf.sibling_of = Some(FindingId::new(1));
        nf.hypothesis = Some("N-WARM: warm-up".into());
        nf.refute_if = Some(REFUTE_IF.into());
        l.add_finding(nf).unwrap();
        l.annotate(&FindingId::new(1), vec!["a.py:6".into()], vec![], None)
            .unwrap();
        l.annotate_record(FindingAnnotation {
            facts: HolderFacts {
                writer_sites: vec![SiteFact {
                    site: "a.py:12".into(),
                    kind: SiteKind::Insert,
                    depth: 0,
                    function: None,
                    init: false,
                    startup: false,
                    startup_via: vec![],
                    param: false,
                    constant: false,
                    exported: false,
                }],
                ..Default::default()
            },
            ..FindingAnnotation::new(FindingId::new(2))
        })
        .unwrap();
        let sibling = l.findings().find(|f| f.id.is_sibling()).unwrap().id.clone();
        l.annotate(&sibling, vec!["a.py:30".into()], vec![], None)
            .unwrap();
        (dir, l)
    }

    fn at(l: &Ledger, site: &str) -> Vec<String> {
        l.findings()
            .filter(|f| finding_is_at_site(l, f, site))
            .map(|f| f.id.to_string())
            .collect()
    }

    #[test]
    fn a_finding_is_at_its_own_site() {
        let (_d, l) = scene();
        assert_eq!(at(&l, "a.py:1"), ["F1"]);
        assert_eq!(at(&l, " a.py:20 "), ["F3"]);
    }

    #[test]
    fn a_finding_is_at_a_writer_line_of_its_holder_listed_plainly_or_in_the_census_facts() {
        let (_d, l) = scene();
        assert_eq!(at(&l, "a.py:6"), ["F1"]);
        assert_eq!(at(&l, "a.py:12"), ["F2"]);
    }

    #[test]
    fn a_sibling_is_never_at_a_site() {
        let (_d, l) = scene();
        assert!(at(&l, "a.py:30").is_empty());
    }

    #[test]
    fn a_site_that_is_neither_has_no_finding_and_a_remover_does_not_count() {
        let (_d, mut l) = scene();
        assert!(at(&l, "a.py:7").is_empty());
        assert!(at(&l, "b.py:6").is_empty());
        l.annotate(&FindingId::new(3), vec![], vec!["a.py:25".into()], None)
            .unwrap();
        assert!(at(&l, "a.py:25").is_empty());
    }

    #[test]
    fn a_pytest_loop_growth_is_decided_on_a_small_driver_of_its_own_first() {
        let f = FindingId::new(2);
        let plain = commands(&f, "H1", Some("E8"), Some("E6"), true, true, false);
        assert_eq!(plain.last().unwrap(), "dla alloc-test H1 --finding F2");
        let loop_cmds = commands(&f, "H1", Some("E8"), Some("E6"), true, true, true);
        assert_eq!(
            &loop_cmds[1..],
            [
                "dla harness gen --lang python --driver --probe F2   (a small driver of its own for the line: a unit that calls the function of that line; a run takes seconds, where an alloc-test on H1 re-runs the whole test suite per run, minutes)",
                "dla harness run <H>",
                "dla trend <S>",
                "dla alloc-test <H> --finding F2   (on the harness just made, not on H1)",
            ]
        );
        // The alloc-test on the loop itself is not refused, only not the first thing printed.
        assert!(loop_cmds[0].starts_with("dla ledger test F2"));
        assert!(
            !loop_cmds
                .iter()
                .any(|c| c == "dla alloc-test H1 --finding F2")
        );
    }
}
