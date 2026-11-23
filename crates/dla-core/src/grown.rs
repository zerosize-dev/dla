//! Growth the run measured and has not decided.
//!
//! A growth column of a harness series (one whose gate passed, or a non-flat allocation-site
//! column) points at a finding: the finding the column was
//! probed for, else the first candidate of its site (the choice `dla trend` makes when it prints
//! `dla ledger add --sibling-of`). Until an alloc-test evidence bound to that finding exists the
//! finding is not closed unmeasured: `close-rest` leaves it, `ledger resolve` refuses to close it,
//! and the final gate lists it with the commands that decide it. A runtime detector hit that
//! names a site is a lead of the same kind while nothing measured the finding at that site.
//!
//! Whole-process columns tie to no finding and stay with the residual rows.

use crate::{
    evidence::{EvidenceKind, EvidenceRecord},
    finding::{FindingState, Polarity},
    gate::is_whole_process,
    ids::{EvidenceId, FindingId},
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
pub fn finding_is_at_site(
    ledger: &crate::ledger::Ledger,
    f: &crate::finding::Finding,
    site: &str,
) -> bool {
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

/// Why a harness is about a finding (see [`harness_about`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessAbout {
    /// The finding (or the finding its sibling hangs from) is a declared target of the harness.
    Target,
    /// The harness is an entry-path harness of the finding.
    EntryPath,
    /// A holder probe of the harness was made for the finding.
    Probe,
    /// A trend of a series of the harness has this column, tied to the finding.
    Column(String),
    /// The finding cites this growth or unit-count evidence of a series of the harness.
    Evidence(crate::ids::EvidenceId),
    /// The harness runs the project's own workload (a command-mode harness with a workload, or a
    /// test loop), which is about the whole program.
    OwnWorkload,
    /// This allocation-stack or retention-path record names the finding and was made from a
    /// capture file of the harness.
    Capture(crate::ids::EvidenceId),
}

/// True when evidence `e` (an allocation-stack or retention-path summary) was made from a
/// capture file of harness `harness`: the raw file of the summary lists, in its `captures` array,
/// the sha256 of each input it read, and `harness/<H>/captures.json` lists the capture files the
/// harness last wrote, whose digests are compared. A capture that a later run replaced no longer
/// matches, so this link says nothing for it.
fn made_from_capture_of(
    run_dir: &std::path::Path,
    e: &crate::evidence::EvidenceRecord,
    harness: &str,
) -> bool {
    use sha2::{Digest, Sha256};
    let Some(raw) = e.raw.as_deref().filter(|r| {
        !std::path::Path::new(r)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    }) else {
        return false;
    };
    let read = |p: &std::path::Path| std::fs::read(p).ok();
    let Some(doc) =
        read(&run_dir.join(raw)).and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
    else {
        return false;
    };
    let used: Vec<String> = doc
        .get("captures")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
        .filter_map(|c| c.get("sha256").and_then(|d| d.as_str()).map(str::to_owned))
        .collect();
    if used.is_empty() {
        return false;
    }
    let list = crate::harness::HarnessManifest::dir_in(run_dir, harness).join("captures.json");
    let Some(files) = read(&list)
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| v.get("files").cloned())
        .and_then(|f| serde_json::from_value::<Vec<crate::harness::CaptureFile>>(f).ok())
    else {
        return false;
    };
    files.iter().any(|f| {
        read(&run_dir.join(&f.path))
            .map(|b| hex::encode(Sha256::digest(&b)))
            .is_some_and(|d| used.contains(&d))
    })
}

/// The leak-side root of `id` (a sibling counts through its parent).
fn root_of(ledger: &crate::ledger::Ledger, id: &FindingId) -> FindingId {
    ledger.finding(id).map_or_else(
        || id.parent().unwrap_or_else(|| id.clone()),
        |f| f.root_id(),
    )
}

/// True when the trend column `c` is tied to the root finding `root`: its `finding`, one of its
/// `candidates` (each read through its root), or, for an allocation-site count column, the
/// finding is at the column's site ([`finding_is_at_site`]).
pub fn column_tied_to(
    ledger: &crate::ledger::Ledger,
    c: &crate::trend::ColumnTrend,
    root: &FindingId,
) -> bool {
    c.finding
        .iter()
        .chain(c.candidates.iter())
        .any(|f| &root_of(ledger, f) == root)
        || (is_alloc_site_column(&c.name)
            && column_site(c).is_some_and(|site| {
                ledger
                    .finding(root)
                    .is_some_and(|f| finding_is_at_site(ledger, f, site))
            }))
}

/// Whether harness `manifest` is about `finding`, and why; `None` when it is not. An
/// `alloc-test` bound to a finding says something about it only on a harness that is about it:
/// the finding's root (a sibling counts through its parent) is read, and at least one of these
/// holds.
///
/// 1. The finding is among the targets of the manifest (`targets`, or the keys or values of
///    `sibling_of`), or a holder probe of the manifest was made for it; an entry-path harness
///    is told apart from a plain target.
/// 2. A trend of a series measured by this harness has a column tied to the finding
///    ([`column_tied_to`]): the column's `finding`, its `candidates`, or, for an allocation-site
///    column, the finding is at the column's site.
/// 3. The finding (or one of its siblings) cites, in `supports` or `contradicts`, growth or
///    unit-count evidence of a series of this harness.
///
/// 4. The harness runs the project's own workload and no hook driver: a command-mode harness with
///    a `workload`, or a test loop. It is about every finding of the run, since the workload is
///    about the whole program. This rule does not check whether the code of the finding ran in
///    that workload.
/// 5. An allocation-stack or retention-path record that names the finding
///    ([`crate::labels::names_finding`]) was made from a capture file of this harness (see
///    `made_from_capture_of`: the sha256 of the inputs the summary lists against the files of
///    `harness/<H>/captures.json`). This is how a hook driver whose capture named a finding that
///    is not among its targets is about it.
///
/// A hook driver made for other findings, whose series and captures have nothing tied to the
/// finding and that the finding does not cite, is not about it: whatever its whole-process
/// columns say, they say it of other code.
pub fn harness_about(
    view: &RunView,
    manifest: &crate::harness::HarnessManifest,
    finding: &FindingId,
) -> Option<HarnessAbout> {
    let ledger = &view.ledger;
    let root = root_of(ledger, finding);
    if manifest.is_test_loop()
        || (manifest.mode == crate::harness::HarnessMode::Command && manifest.workload.is_some())
    {
        return Some(HarnessAbout::OwnWorkload);
    }
    let is_root = |f: &FindingId| root_of(ledger, f) == root;
    if manifest.targets.iter().any(is_root)
        || manifest
            .sibling_of
            .iter()
            .any(|(k, v)| is_root(k) || is_root(v))
    {
        return Some(if manifest.is_entry_path() {
            HarnessAbout::EntryPath
        } else {
            HarnessAbout::Target
        });
    }
    if manifest
        .probes
        .iter()
        .any(|p| p.finding.as_ref().is_some_and(is_root))
    {
        return Some(HarnessAbout::Probe);
    }
    let of_harness = |series: &str| {
        view.series_harness.get(series).map(String::as_str) == Some(manifest.id.as_str())
    };
    for t in view.trends.iter().filter(|t| of_harness(&t.series)) {
        if let Some(c) = t.columns.iter().find(|c| column_tied_to(ledger, c, &root)) {
            return Some(HarnessAbout::Column(c.name.clone()));
        }
    }
    let family: Vec<&crate::finding::Finding> = ledger
        .findings()
        .filter(|f| f.id == root || f.parent.as_ref() == Some(&root))
        .collect();
    for ev in ledger.evidence() {
        if !matches!(
            ev.kind,
            EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries
        ) || !family.iter().any(|f| {
            f.supports.contains(&ev.evidence_id) || f.contradicts.contains(&ev.evidence_id)
        }) {
            continue;
        }
        let series = ev.result.get("series").and_then(|s| s.as_str());
        if series.is_some_and(of_harness) {
            return Some(HarnessAbout::Evidence(ev.evidence_id.clone()));
        }
    }
    if let Some(dir) = view.run_dir.as_deref()
        && let Some(rf) = ledger.finding(&root)
    {
        let subjects: Vec<&crate::finding::Finding> =
            std::iter::once(rf).chain(family.iter().copied()).collect();
        for ev in ledger.evidence() {
            if crate::labels::names_finding(ev, &subjects)
                && made_from_capture_of(dir, ev, &manifest.id)
            {
                return Some(HarnessAbout::Capture(ev.evidence_id.clone()));
            }
        }
    }
    None
}

/// The harnesses of the run that are about `finding` ([`harness_about`]), in numeric order, with
/// their manifests; empty for a view without a run directory.
pub fn harnesses_about(
    view: &RunView,
    finding: &FindingId,
) -> Vec<(crate::harness::HarnessManifest, HarnessAbout)> {
    let Some(dir) = view.run_dir.as_deref() else {
        return Vec::new();
    };
    crate::gate::harness_ids(dir)
        .into_iter()
        .filter_map(|(_, id)| crate::harness::HarnessManifest::load(dir, &id).ok())
        .filter_map(|m| harness_about(view, &m, finding).map(|why| (m, why)))
        .collect()
}

/// The harnesses (`H1`), oldest series first and each once, in whose latest trend a passed
/// allocation-site count column (`py.blocks@<file:line>`) is at a site where the leak-side finding
/// `f` is (`finding_is_at_site`): the runs that grew the finding's line.
pub fn site_column_harnesses(view: &RunView, f: &crate::finding::Finding) -> Vec<String> {
    let mut latest: Vec<&crate::trend::TrendReport> = Vec::new();
    for t in &view.trends {
        match latest.iter_mut().find(|l| l.series == t.series) {
            Some(l) => *l = t,
            None => latest.push(t),
        }
    }
    let mut out: Vec<String> = Vec::new();
    for t in latest {
        let Some(harness) = view.series_harness.get(&t.series) else {
            continue;
        };
        let at_site = t.columns.iter().any(|c| {
            c.gate == GateVerdict::Pass
                && c.unit != Unit::Seconds
                && is_alloc_site_column(&c.name)
                && column_site(c).is_some_and(|site| finding_is_at_site(&view.ledger, f, site))
        });
        if at_site && !out.contains(harness) {
            out.push(harness.clone());
        }
    }
    out
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

/// The series of harness `harness` that the evidence of the run names: the `series` list of each
/// of its `workload` records (what `dla harness run` wrote), then `extra`.
pub(crate) fn series_of_harness(
    records: &[&EvidenceRecord],
    harness: &str,
    extra: Option<&str>,
) -> Vec<String> {
    let mut out: Vec<String> = records
        .iter()
        .filter(|e| e.kind == EvidenceKind::Workload)
        .filter(|e| e.result.get("harness").and_then(|h| h.as_str()) == Some(harness))
        .flat_map(|e| {
            e.result
                .get("series")
                .and_then(|s| s.as_array())
                .into_iter()
                .flatten()
                .filter_map(|s| s.as_str().map(str::to_owned))
        })
        .collect();
    out.extend(extra.map(str::to_owned));
    out
}

/// The growth evidence a `confirm` of `root` should cite: the newest passing one tied to it
/// (`tied_to_owner`, the predicate the ledger uses), else the newest passing one a column of
/// which is the best for it. Among equals the ones of a series in `prefer` come first: the
/// series of the harness that was measured last.
pub fn growth_evidence(
    records: &[&EvidenceRecord],
    root: &crate::finding::Finding,
    prefer: &[String],
) -> Option<EvidenceId> {
    // Only what the confirm rule accepts: a growth record obtained by a test, a live observation
    // or a dump whose gate passed (`machine::is_passing_growth`). A record with static authority
    // can pass its gate and still be refused by `dla ledger confirm`.
    let growth = || {
        records
            .iter()
            .copied()
            .filter(|e| crate::machine::is_passing_growth(e))
    };
    let newest = |hits: Vec<&EvidenceRecord>| -> Option<EvidenceId> {
        hits.iter()
            .rev()
            .find(|e| {
                e.result
                    .get("series")
                    .and_then(|s| s.as_str())
                    .is_some_and(|s| prefer.iter().any(|p| p == s))
            })
            .or(hits.last())
            .map(|e| e.evidence_id.clone())
    };
    let tied = newest(
        growth()
            .filter(|e| {
                crate::trend::GrowthSummary::from_result(&e.result).is_some_and(|g| {
                    g.passed_gate()
                        && g.passed.iter().any(|c| {
                            c.tied_to_owner(&root.id, root.site.as_deref(), root.owner.as_deref())
                        })
                })
            })
            .collect(),
    );
    // Else the newest passing one, as before (its first column stands in for the tie).
    tied.or_else(|| {
        newest(
            growth()
                .filter(|e| {
                    crate::trend::GrowthSummary::from_result(&e.result).is_some_and(|g| {
                        g.passed_gate()
                            && g.best_column_for(
                                &root.id,
                                root.site.as_deref(),
                                root.owner.as_deref(),
                            )
                            .is_some()
                    })
                })
                .collect(),
        )
    })
}

/// The newest `workload` evidence of harness `harness`.
pub fn workload_evidence(ledger: &crate::ledger::Ledger, harness: &str) -> Option<EvidenceId> {
    ledger
        .evidence()
        .filter(|e| e.kind == EvidenceKind::Workload)
        .filter(|e| e.result.get("harness").and_then(|h| h.as_str()) == Some(harness))
        .last()
        .map(|e| e.evidence_id.clone())
}

/// The alloc-test discriminators (`plateau+purge`) bound to a finding of `family`, oldest first,
/// each with what it favors.
pub fn bound_alloc_tests<'a>(
    records: &[&'a EvidenceRecord],
    family: &[FindingId],
) -> Vec<(&'a EvidenceRecord, crate::discriminator::Favors)> {
    records
        .iter()
        .copied()
        .filter(|e| {
            e.kind == EvidenceKind::Discriminator
                && e.result.get("test").and_then(|t| t.as_str()) == Some("plateau+purge")
                && crate::discriminator::finding_of(e).is_some_and(|f| family.contains(&f))
        })
        .filter_map(|e| crate::discriminator::favors_of(e).map(|f| (e, f)))
        .collect()
}

/// The ledger commands that decide a leak-side root `root_f` after an alloc-test of `harness`
/// whose discriminator `evidence` favors leak: the refute of each open non-leak sibling (or the
/// add of the sibling and its refute, when there is none), the test when the root is still a
/// candidate, the annotations it lacks and the confirm with the growth evidence the ledger
/// accepts. `dla alloc-test` prints these as `next`; the gate, `dla run status` and the refusal
/// of a close print them too.
pub fn confirm_commands(
    ledger: &crate::ledger::Ledger,
    root_f: &crate::finding::Finding,
    harness: &str,
    evidence: &EvidenceId,
) -> Vec<String> {
    let root = &root_f.id;
    let mut next: Vec<String> = Vec::new();
    let siblings: Vec<&crate::finding::Finding> = ledger
        .findings()
        .filter(|s| s.parent.as_ref() == Some(root) && s.polarity() == Polarity::NonLeak)
        .collect();
    for s in siblings.iter().filter(|s| !s.state.is_dispositioned()) {
        next.push(format!("dla ledger refute {} --evidence {evidence}", s.id));
    }
    if siblings.is_empty() {
        next.push(format!(
            "dla ledger add --sibling-of {root} --origin hypothesis --hypothesis N-WARM --refute-if \"growth continues over the extra cycles\" --evidence {evidence}   (a leak needs a refuted non-leak sibling)"
        ));
        // The sibling that add creates is refuted by this same result; without that step the
        // confirm below is refused.
        next.push(format!(
            "dla ledger refute {root}.n1 --evidence {evidence}   (the sibling the add creates: use the id it prints)"
        ));
    }
    // A root that is open, or closed in a state the ledger lets go to `under_test`
    // (`suspected_static_only`, `unresolved`), still gets its test and confirm.
    let can_test = crate::machine::is_allowed(root_f.state, FindingState::UnderTest);
    if !root_f.state.is_dispositioned() || can_test {
        let records: Vec<&EvidenceRecord> = ledger.evidence().collect();
        let prefer = series_of_harness(
            &records,
            harness,
            ledger
                .evidence_by_id(evidence)
                .and_then(|e| e.result.get("series"))
                .and_then(|s| s.as_str()),
        );
        let growth = growth_evidence(&records, root_f, &prefer).map_or_else(
            || "<growth evidence from dla trend>".to_owned(),
            |e| e.to_string(),
        );
        if can_test && let Some(w) = workload_evidence(ledger, harness) {
            next.push(format!("dla ledger test {root} --evidence {w}"));
        }
        if root_f.holder.is_none() {
            next.push(format!("dla ledger annotate {root} --holder <holder>"));
        }
        if root_f.site.is_none() {
            next.push(format!("dla ledger annotate {root} --site <file:line>"));
        }
        next.push(format!("dla ledger confirm {root} --evidence {growth}"));
    }
    next
}

/// The way from the present state of a leak-side finding to its confirmation, when an alloc-test
/// decided it: see [`route_to_confirm`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmRoute {
    /// The leak-side root the route confirms.
    pub finding: FindingId,
    /// The discriminator of the newest alloc-test bound to it, which favors leak.
    pub discriminator: EvidenceId,
    /// The harness that alloc-test ran on.
    pub harness: String,
    /// The column that decided it, when the record names one.
    pub column: Option<String>,
    /// The ledger commands, in order, each accepted as printed.
    pub commands: Vec<String>,
}

impl ConfirmRoute {
    /// The commands as one fix line.
    pub fn fix(&self) -> String {
        self.commands.join("   then   ")
    }

    /// Why `id` (the finding or one of its siblings) is not closed unresolved.
    pub fn refusal_message(&self, id: &FindingId) -> String {
        let by = self
            .column
            .as_deref()
            .map_or(String::new(), |c| format!(" (decided by {c})"));
        format!(
            "{id} cannot be closed unresolved: the alloc-test {} on {} favors leak{by} and nothing measured since says otherwise",
            self.discriminator, self.harness
        )
    }
}

/// The route to confirm the leak-side root `root`, when it has one: an alloc-test bound to its
/// family, run on a hook driver made for it (the finding is among its targets, or it is the
/// entry-path harness of the finding: `Target` and `EntryPath` of [`harness_about`]), whose
/// discriminator favors leak and is the newest alloc-test on a harness about the finding (a newer
/// one on any such harness that favors non-leak, or says inconclusive, lifts it). An alloc-test of
/// the project's whole workload, of a test loop, or on a harness about the finding only through a
/// probe, a column, cited evidence or a capture gives no route: the growth there may be kept alive
/// by another holder, and closing the finding unresolved stays possible. The root has to be still
/// open (a candidate or under test) and its trigger not `direct` (a confirm of that is refused for another reason, and its fix is the entry-path
/// route). The commands are [`confirm_commands`].
pub fn route_to_confirm(view: &RunView, root: &FindingId) -> Option<ConfirmRoute> {
    let ledger = &view.ledger;
    let dir = view.run_dir.as_deref()?;
    let rf = ledger.finding(root)?;
    if rf.polarity() != Polarity::Leak
        || root.is_sibling()
        || ledger.merged_into(root).is_some()
        || !matches!(rf.state, FindingState::Candidate | FindingState::UnderTest)
    {
        return None;
    }
    let family: Vec<FindingId> = ledger
        .findings()
        .filter(|f| &f.id == root || f.parent.as_ref() == Some(root))
        .map(|f| f.id.clone())
        .collect();
    let records: Vec<&EvidenceRecord> = ledger.evidence().collect();
    // The newest alloc-test on any harness about the finding decides (a newer one that does not
    // favor leak lifts the route); the route needs it to have run on a driver made for the finding.
    let (disc, favors, made_for_it) = bound_alloc_tests(&records, &family)
        .into_iter()
        .rev()
        .find_map(|(e, f)| {
            let m = e
                .result
                .get("harness")
                .and_then(|h| h.as_str())
                .and_then(|h| crate::harness::HarnessManifest::load(dir, h).ok())?;
            let about = harness_about(view, &m, root)?;
            let made = m.is_hook_driver()
                && matches!(about, HarnessAbout::Target | HarnessAbout::EntryPath);
            Some((e, f, made))
        })?;
    if favors != crate::discriminator::Favors::Leak || !made_for_it {
        return None;
    }
    if matches!(
        crate::trigger::Triggers::of(view).trigger(root),
        crate::trigger::Trigger::Direct { .. }
    ) {
        return None;
    }
    let harness = disc.result.get("harness")?.as_str()?.to_owned();
    Some(ConfirmRoute {
        finding: root.clone(),
        discriminator: disc.evidence_id.clone(),
        column: disc
            .result
            .get("column")
            .and_then(|c| c.as_str())
            .map(str::to_owned),
        commands: confirm_commands(ledger, rf, &harness, &disc.evidence_id),
        harness,
    })
}

/// [`route_to_confirm`] for a finding or a sibling of one: the route of its root.
pub fn route_for(view: &RunView, id: &FindingId) -> Option<ConfirmRoute> {
    route_to_confirm(view, &root_of(&view.ledger, id))
}

/// True when a trend column is growth the run measured and nobody decided: its gate passed, or
/// it is an allocation-site count column (`py.blocks@<file:line>`) that is non-flat by the gate's
/// own predicate ([`crate::gate::non_flat`]: shape `steady` or `stair`). A site column can fail
/// the counter gate on one flat step (a unit that did not run) and still be a line that grew.
pub fn counts_as_growth(c: &crate::trend::ColumnTrend) -> bool {
    c.unit != Unit::Seconds
        && (c.gate == GateVerdict::Pass
            || (is_alloc_site_column(&c.name) && crate::gate::non_flat(c)))
}

/// Findings a growth column of a harness series points at (a column whose gate passed, or a
/// non-flat allocation-site column: [`counts_as_growth`]), or that a trend gave the
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
            .filter(|c| counts_as_growth(c))
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
                    .find(|f| finding_is_at_site(&view.ledger, f, site) && !f.id.is_sibling())
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
                "{} is closed {} but its column {} of {} grew (it passed the growth gate, or it is a non-flat allocation-site column) and no alloc-test decided it",
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
            let mut nf = NewFinding::new(
                Origin::parse("census").unwrap(),
                vec![crate::EvidenceId::new(1)],
            );
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

    /// The findings `grown_unmeasured` lists for a one-column trend of series S1 (harness H1)
    /// whose column is `name`, at the site `a.py:1` of F1, with the given shape and gate.
    fn grown_by_column(name: &str, shape: &str, gate: &str) -> Vec<String> {
        let (_d, l) = scene();
        let state =
            crate::state::RunState::init("/p", None, 3600, None, crate::state::now_utc()).unwrap();
        let mut view = RunView::new(state, l, None);
        let col: crate::trend::ColumnTrend = serde_json::from_value(serde_json::json!({
            "name": name, "unit": "count", "kind": "counter", "region": "heap",
            "site": "a.py:1", "n": 8, "noise": 0.0, "rise": 100.0, "shape": shape, "gate": gate,
            "gate_failed": if gate == "fail" { vec!["delta < 1"] } else { vec![] },
        }))
        .unwrap();
        view.trends = vec![crate::trend::TrendReport {
            schema_version: 1,
            id: "T1".into(),
            series: "S1".into(),
            thresholds: dla_stats::GateThresholds::default(),
            columns: vec![col],
            targets: vec![],
        }];
        view.series_harness.insert("S1".into(), "H1".into());
        grown_unmeasured(&view)
            .iter()
            .map(|g| g.finding.to_string())
            .collect()
    }

    const SITE_COLUMN: &str = "py.blocks@a.py:1";

    #[test]
    fn a_site_column_whose_gate_passed_points_at_the_finding_at_its_site() {
        assert_eq!(grown_by_column(SITE_COLUMN, "steady", "pass"), ["F1"]);
    }

    #[test]
    fn a_steady_or_stair_site_column_points_at_it_although_its_gate_failed() {
        assert_eq!(grown_by_column(SITE_COLUMN, "steady", "fail"), ["F1"]);
        assert_eq!(grown_by_column(SITE_COLUMN, "stair", "fail"), ["F1"]);
        // The same for the column of a series cut before the collection (`.pre_gc`).
        assert_eq!(
            grown_by_column("py.blocks@a.py:1.pre_gc", "steady", "fail"),
            ["F1"]
        );
    }

    #[test]
    fn a_flat_plateau_spike_or_falling_site_column_with_a_failed_gate_points_at_nothing() {
        for shape in ["flat", "plateau", "spike", "declining", "insufficient"] {
            assert!(
                grown_by_column(SITE_COLUMN, shape, "fail").is_empty(),
                "{shape}"
            );
        }
    }

    #[test]
    fn only_an_allocation_site_column_is_helped_by_its_shape() {
        // Another column at the same site (a type count, a probe) keeps the rule of the gate.
        assert!(grown_by_column("py.type:list.count", "steady", "fail").is_empty());
        assert_eq!(
            grown_by_column("py.type:list.count", "steady", "pass"),
            ["F1"]
        );
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

    /// A manifest of harness `id` with the given targets, probe finding and purpose.
    fn manifest(id: &str, extra: serde_json::Value) -> crate::harness::HarnessManifest {
        let mut v = serde_json::json!({
            "id": id, "runtime": "python", "mode": "driver",
            "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
            "warmup": 1, "iterations": 8, "step": 10, "key_modes": ["fresh"],
            "agent": "driver-python", "timeout_secs": 60,
        });
        for (k, x) in extra.as_object().unwrap() {
            v[k] = x.clone();
        }
        serde_json::from_value(v).unwrap()
    }

    /// A view over [`scene`] with a trend of series S1 of harness H1 holding `column`.
    fn view_with(l: Ledger, column: serde_json::Value) -> RunView {
        let state =
            crate::state::RunState::init("/p", None, 3600, None, crate::state::now_utc()).unwrap();
        let mut view = RunView::new(state, l, None);
        let col: crate::trend::ColumnTrend = serde_json::from_value(serde_json::json!({
            "unit": "count", "kind": "counter", "region": "heap", "n": 8, "noise": 0.0,
            "rise": 100.0, "shape": "steady", "gate": "pass", "gate_failed": [],
        }))
        .map(|mut v: serde_json::Value| {
            for (k, x) in column.as_object().unwrap() {
                v[k] = x.clone();
            }
            v
        })
        .and_then(serde_json::from_value)
        .unwrap();
        view.trends = vec![crate::trend::TrendReport {
            schema_version: 1,
            id: "T1".into(),
            series: "S1".into(),
            thresholds: dla_stats::GateThresholds::default(),
            columns: vec![col],
            targets: vec![],
        }];
        view.series_harness.insert("S1".into(), "H1".into());
        view
    }

    fn about(view: &RunView, m: &crate::harness::HarnessManifest, f: u32) -> Option<HarnessAbout> {
        harness_about(view, m, &FindingId::new(f))
    }

    #[test]
    fn a_harness_is_about_the_findings_it_targets_and_not_about_the_others() {
        let (_d, l) = scene();
        let view = view_with(l, serde_json::json!({"name": "py.traced", "unit": "bytes"}));
        let m = manifest("H1", serde_json::json!({"targets": ["F2"]}));
        assert_eq!(about(&view, &m, 2), Some(HarnessAbout::Target));
        assert_eq!(about(&view, &m, 3), None);
        // A sibling counts through its parent.
        assert_eq!(
            harness_about(&view, &m, &"F2.n1".parse().unwrap()),
            Some(HarnessAbout::Target)
        );
        let on_sibling = manifest("H2", serde_json::json!({"targets": ["F1.n1"]}));
        assert_eq!(about(&view, &on_sibling, 1), Some(HarnessAbout::Target));
    }

    #[test]
    fn the_keys_and_values_of_sibling_of_are_targets_too() {
        let (_d, l) = scene();
        let view = view_with(l, serde_json::json!({"name": "py.traced", "unit": "bytes"}));
        let m = manifest("H1", serde_json::json!({"sibling_of": {"F3": "F2"}}));
        assert_eq!(about(&view, &m, 3), Some(HarnessAbout::Target));
        assert_eq!(about(&view, &m, 2), Some(HarnessAbout::Target));
        assert_eq!(about(&view, &m, 1), None);
    }

    #[test]
    fn an_entry_path_harness_and_a_probe_harness_are_about_their_finding() {
        let (_d, l) = scene();
        let view = view_with(l, serde_json::json!({"name": "py.traced", "unit": "bytes"}));
        let entry = manifest(
            "H1",
            serde_json::json!({"targets": ["F3"], "purpose": "entry-path"}),
        );
        assert_eq!(about(&view, &entry, 3), Some(HarnessAbout::EntryPath));
        let probed = manifest(
            "H1",
            serde_json::json!({"probes": [{"id": "P1", "finding": "F2",
                "target": {"kind": "py-global", "module": "m", "name": "X"}}]}),
        );
        assert_eq!(about(&view, &probed, 2), Some(HarnessAbout::Probe));
        assert_eq!(about(&view, &probed, 3), None);
    }

    #[test]
    fn a_column_tied_to_the_finding_by_finding_candidate_or_site_makes_the_harness_about_it() {
        let m = manifest("H1", serde_json::json!({}));
        for (column, f, want) in [
            (
                serde_json::json!({"name": "probe:x", "finding": "F2"}),
                2,
                Some("probe:x"),
            ),
            (
                serde_json::json!({"name": "py.traced", "unit": "bytes", "candidates": ["F3"]}),
                3,
                Some("py.traced"),
            ),
            // A candidate that is a sibling counts through its parent.
            (
                serde_json::json!({"name": "probe:y", "candidates": ["F1.n1"]}),
                1,
                Some("probe:y"),
            ),
            // The column of a line counts for the finding at the line, or at a writer of it.
            (
                serde_json::json!({"name": "py.blocks@a.py:20", "site": "a.py:20"}),
                3,
                Some("py.blocks@a.py:20"),
            ),
            (
                serde_json::json!({"name": "py.blocks@a.py:12"}),
                2,
                Some("py.blocks@a.py:12"),
            ),
            // Another finding's line, a whole-process column and a site column of no finding.
            (serde_json::json!({"name": "py.blocks@a.py:20"}), 2, None),
            (
                serde_json::json!({"name": "py.traced", "unit": "bytes"}),
                2,
                None,
            ),
            (serde_json::json!({"name": "py.blocks@z.py:9"}), 3, None),
        ] {
            let (_d, l) = scene();
            let view = view_with(l, column.clone());
            assert_eq!(
                about(&view, &m, f),
                want.map(|c| HarnessAbout::Column(c.to_owned())),
                "{column}"
            );
        }
    }

    #[test]
    fn a_trend_of_another_harness_does_not_make_this_one_about_the_finding() {
        let (_d, l) = scene();
        let view = view_with(l, serde_json::json!({"name": "probe:x", "finding": "F2"}));
        assert_eq!(
            about(&view, &manifest("H1", serde_json::json!({})), 2),
            Some(HarnessAbout::Column("probe:x".into()))
        );
        assert_eq!(
            about(&view, &manifest("H2", serde_json::json!({})), 2),
            None
        );
    }

    #[test]
    fn the_growth_evidence_a_finding_cites_makes_the_harness_of_its_series_about_it() {
        let (_d, mut l) = scene();
        let _ev = l
            .add_evidence(NewEvidence {
                source: Source::Repo {
                    commit: "c".into(),
                    path: None,
                },
                tool: Tool {
                    name: "t".into(),
                    version: "1".into(),
                },
                authority: Authority::Static,
                kind: EvidenceKind::GrowthSeries,
                result: serde_json::json!({"series": "S1"}),
                raw: None,
                coverage: EvidenceCoverage::default(),
                limitations: vec![],
                unknowns: vec![],
            })
            .unwrap();
        let mut nf = NewFinding::new(
            Origin::parse("growth-series").unwrap(),
            vec![crate::EvidenceId::new(2)],
        );
        nf.site = Some("a.py:40".into());
        l.add_finding(nf).unwrap();
        let view = view_with(l, serde_json::json!({"name": "py.traced", "unit": "bytes"}));
        let h1 = manifest("H1", serde_json::json!({}));
        let h2 = manifest("H2", serde_json::json!({}));
        assert!(matches!(
            harness_about(&view, &h1, &FindingId::new(4)),
            Some(HarnessAbout::Evidence(_))
        ));
        // Another harness, and a finding that cites nothing of the series.
        assert_eq!(harness_about(&view, &h2, &FindingId::new(4)), None);
        assert_eq!(about(&view, &h1, 3), None);
    }

    #[test]
    fn a_harness_with_none_of_it_is_not_about_the_finding() {
        let (_d, l) = scene();
        let view = view_with(l, serde_json::json!({"name": "py.traced", "unit": "bytes"}));
        let m = manifest("H1", serde_json::json!({"targets": ["F1"]}));
        for f in [2, 3] {
            assert_eq!(about(&view, &m, f), None, "F{f}");
        }
    }

    #[test]
    fn a_harness_that_runs_the_projects_own_workload_is_about_every_finding() {
        let (_d, l) = scene();
        let view = view_with(l, serde_json::json!({"name": "py.traced", "unit": "bytes"}));
        let command = manifest(
            "H1",
            serde_json::json!({"mode": "command", "workload": "go:main:.:./cmd/x"}),
        );
        for f in [1, 2, 3] {
            assert_eq!(about(&view, &command, f), Some(HarnessAbout::OwnWorkload));
        }
        let test_loop = manifest(
            "H1",
            serde_json::json!({"tools": [crate::harness::PYTEST_TOOL]}),
        );
        assert_eq!(about(&view, &test_loop, 3), Some(HarnessAbout::OwnWorkload));
        // A command without a workload, and a hook driver with no tie, are not.
        let bare = manifest("H1", serde_json::json!({"mode": "command"}));
        assert_eq!(about(&view, &bare, 3), None);
        let hooks = manifest("H1", serde_json::json!({"targets": ["F1"]}));
        assert_eq!(about(&view, &hooks, 3), None);
    }

    /// A run directory with the capture file `harness/<harness>/out/heap.txt` listed in the
    /// `captures.json` of `harness`, and an allocation-stack record naming F3 whose raw file lists
    /// the digest of the capture file `summarised_from` (the content written for it).
    fn with_capture_summary(harness: &str, summarised_from: &str) -> (tempfile::TempDir, RunView) {
        use sha2::{Digest, Sha256};
        let dir = tempfile::tempdir().unwrap();
        let hd = dir.path().join(format!("harness/{harness}/out"));
        std::fs::create_dir_all(&hd).unwrap();
        std::fs::write(hd.join("heap.txt"), "capture").unwrap();
        std::fs::write(
            dir.path().join(format!("harness/{harness}/captures.json")),
            serde_json::json!({"files": [{
                "kind": "alloc", "path": format!("harness/{harness}/out/heap.txt"),
                "label": "p1", "seq": 1, "bytes": 7, "captured_at": "2026-01-01T00:00:00Z"}]})
            .to_string(),
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("alloc")).unwrap();
        std::fs::write(
            dir.path().join("alloc/A1.json"),
            serde_json::json!({"captures": [{"sha256":
                hex::encode(Sha256::digest(summarised_from.as_bytes()))}]})
            .to_string(),
        )
        .unwrap();
        let (_d, mut l) = scene();
        l.add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: Authority::Test,
            kind: EvidenceKind::AllocStack,
            result: serde_json::json!({"candidates": ["F3"]}),
            raw: Some("alloc/A1.json".into()),
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        let mut view = view_with(l, serde_json::json!({"name": "py.traced", "unit": "bytes"}));
        view.run_dir = Some(dir.path().to_path_buf());
        (dir, view)
    }

    #[test]
    fn a_hook_driver_whose_capture_summary_names_the_finding_is_about_it() {
        let (_d, view) = with_capture_summary("H1", "capture");
        let m = manifest("H1", serde_json::json!({"targets": ["F1"]}));
        assert!(matches!(
            about(&view, &m, 3),
            Some(HarnessAbout::Capture(_))
        ));
        // The summary names F3 only.
        assert_eq!(about(&view, &m, 2), None);
    }

    #[test]
    fn a_capture_summary_of_another_harness_or_of_other_files_does_not_count() {
        // The summary was made from the capture file of H1; H2 has its own file and digest.
        let (_d, view) = with_capture_summary("H1", "capture");
        let other = manifest("H2", serde_json::json!({"targets": ["F1"]}));
        assert_eq!(about(&view, &other, 3), None);
        // A summary whose inputs are not the files of the harness.
        let (_d, view) = with_capture_summary("H1", "something else");
        let m = manifest("H1", serde_json::json!({"targets": ["F1"]}));
        assert_eq!(about(&view, &m, 3), None);
    }

    fn add_ev(l: &mut Ledger, kind: EvidenceKind, result: serde_json::Value) -> EvidenceId {
        l.add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: Authority::Test,
            kind,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        l.evidence().last().unwrap().evidence_id.clone()
    }

    fn alloc_test(l: &mut Ledger, favors: &str, harness: &str) -> EvidenceId {
        add_ev(
            l,
            EvidenceKind::Discriminator,
            serde_json::json!({"test": "plateau+purge", "favors": favors, "finding": "F1",
                "harness": harness, "series": "S9", "column": "py.blocks@a.py:1"}),
        )
    }

    fn growth(l: &mut Ledger, series: &str) -> EvidenceId {
        add_ev(
            l,
            EvidenceKind::UnitCountSeries,
            serde_json::json!({"gate": "pass", "series": series, "passed": [
                {"column": "py.blocks@a.py:1", "site": "a.py:1", "candidates": ["F1"]}]}),
        )
    }

    /// A view over [`scene`] (F1 a candidate with the sibling F1.n1) whose run directory holds
    /// the harness H1, which targets F1 (`about`) or no finding.
    fn routed(l: Ledger, about: bool) -> (tempfile::TempDir, RunView) {
        let dir = tempfile::tempdir().unwrap();
        let m = manifest(
            "H1",
            serde_json::json!({ "targets": if about { vec!["F1"] } else { vec![] } }),
        );
        m.save(dir.path()).unwrap();
        let mut view = view_with(l, serde_json::json!({"name": "py.traced", "unit": "bytes"}));
        view.run_dir = Some(dir.path().to_path_buf());
        (dir, view)
    }

    fn f1() -> FindingId {
        FindingId::new(1)
    }

    #[test]
    fn the_route_is_the_refute_of_the_open_sibling_and_the_confirm_with_growth_evidence() {
        let (_d, mut l) = scene();
        let g = growth(&mut l, "S2");
        let d = alloc_test(&mut l, "leak", "H1");
        let (_r, view) = routed(l, true);
        let route = route_to_confirm(&view, &f1()).unwrap();
        assert_eq!(route.discriminator, d);
        assert_eq!(route.harness, "H1");
        assert_eq!(route.column.as_deref(), Some("py.blocks@a.py:1"));
        assert_eq!(
            route.commands,
            [
                format!("dla ledger refute F1.n1 --evidence {d}"),
                "dla ledger annotate F1 --holder <holder>".to_owned(),
                format!("dla ledger confirm F1 --evidence {g}"),
            ]
        );
        // A sibling reads the route of its root.
        assert_eq!(route_for(&view, &"F1.n1".parse().unwrap()), Some(route));
    }

    #[test]
    fn only_the_newest_alloc_test_decides_and_it_has_to_favor_leak() {
        for (later, lifted) in [("non-leak", true), ("inconclusive", true), ("leak", false)] {
            let (_d, mut l) = scene();
            growth(&mut l, "S2");
            alloc_test(&mut l, "leak", "H1");
            alloc_test(&mut l, later, "H1");
            let (_r, view) = routed(l, true);
            assert_eq!(route_to_confirm(&view, &f1()).is_none(), lifted, "{later}");
        }
        // A leak-favoring one after a non-leak one is a route again.
        let (_d, mut l) = scene();
        growth(&mut l, "S2");
        alloc_test(&mut l, "non-leak", "H1");
        let newest = alloc_test(&mut l, "leak", "H1");
        let (_r, view) = routed(l, true);
        assert_eq!(
            route_to_confirm(&view, &f1()).unwrap().discriminator,
            newest
        );
    }

    #[test]
    fn an_alloc_test_on_a_harness_that_is_not_about_the_finding_makes_no_route() {
        let (_d, mut l) = scene();
        growth(&mut l, "S2");
        alloc_test(&mut l, "leak", "H1");
        let (_r, view) = routed(l, false);
        assert_eq!(route_to_confirm(&view, &f1()), None);
    }

    /// [`routed`] with H1 built from `extra` and the trend column `column`.
    fn routed_with(
        l: Ledger,
        extra: serde_json::Value,
        column: serde_json::Value,
    ) -> (tempfile::TempDir, RunView) {
        let dir = tempfile::tempdir().unwrap();
        manifest("H1", extra).save(dir.path()).unwrap();
        let mut view = view_with(l, column);
        view.run_dir = Some(dir.path().to_path_buf());
        (dir, view)
    }

    #[test]
    fn only_a_driver_made_for_the_finding_gives_a_route() {
        let untied = serde_json::json!({"name": "py.traced", "unit": "bytes"});
        let tied = serde_json::json!({"name": "py.blocks@a.py:1", "site": "a.py:1"});
        let cases = [
            // A hook driver that targets the finding.
            (serde_json::json!({"targets": ["F1"]}), untied.clone(), true),
            // The whole workload of the project, a test loop, a driver about it only through a
            // column or a probe: no route.
            (
                serde_json::json!({"mode": "command", "workload": "w"}),
                untied.clone(),
                false,
            ),
            (
                serde_json::json!({"tools": [crate::harness::PYTEST_TOOL]}),
                untied.clone(),
                false,
            ),
            (serde_json::json!({}), tied, false),
            (
                serde_json::json!({"probes": [{"id": "P1", "finding": "F1", "target": {"kind": "py-module", "module": "m"}}]}),
                untied,
                false,
            ),
        ];
        for (extra, column, route) in cases {
            let (_d, mut l) = scene();
            growth(&mut l, "S2");
            alloc_test(&mut l, "leak", "H1");
            let (_r, view) = routed_with(l, extra.clone(), column);
            assert_eq!(route_to_confirm(&view, &f1()).is_some(), route, "{extra}");
        }
    }

    #[test]
    fn a_finding_that_is_decided_or_has_no_alloc_test_has_no_route() {
        let (_d, mut l) = scene();
        growth(&mut l, "S2");
        let (_r, view) = routed(l, true);
        assert_eq!(route_to_confirm(&view, &f1()), None);
        // The alloc-test is bound to F2's family, not F1's.
        let (_d, mut l) = scene();
        growth(&mut l, "S2");
        add_ev(
            &mut l,
            EvidenceKind::Discriminator,
            serde_json::json!({"test": "plateau+purge", "favors": "leak", "finding": "F2",
                "harness": "H1"}),
        );
        let (_r, view) = routed(l, true);
        assert_eq!(route_to_confirm(&view, &f1()), None);
        // Refuted, it needs no route.
        let (_d, mut l) = scene();
        growth(&mut l, "S2");
        let d = alloc_test(&mut l, "non-leak", "H1");
        l.transition(&f1(), FindingState::Refuted, None, None, vec![d], "t")
            .unwrap();
        alloc_test(&mut l, "leak", "H1");
        let (_r, view) = routed(l, true);
        assert_eq!(route_to_confirm(&view, &f1()), None);
    }

    #[test]
    fn the_growth_evidence_prefers_the_series_of_the_harness_that_was_measured() {
        let (_d, mut l) = scene();
        let on_h1 = growth(&mut l, "S2");
        add_ev(
            &mut l,
            EvidenceKind::Workload,
            serde_json::json!({"harness": "H1", "series": ["S2", "S3"]}),
        );
        let other = growth(&mut l, "S7");
        let d = alloc_test(&mut l, "leak", "H1");
        let root = l.finding(&f1()).unwrap().clone();
        let all: Vec<&EvidenceRecord> = l.evidence().collect();
        // Without a preference the newest one wins; with the harness of the alloc-test its own.
        assert_eq!(growth_evidence(&all, &root, &[]), Some(other));
        let cmds = confirm_commands(&l, &root, "H1", &d);
        assert_eq!(
            cmds.last().unwrap(),
            &format!("dla ledger confirm F1 --evidence {on_h1}")
        );
    }
}
