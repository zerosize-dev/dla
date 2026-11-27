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
//! Growth a dump series measured (a summarizer's `growth-series` record of authority `dump`
//! whose gate passed, or a `dump` discriminator that favors leak) is a lead of the same kind
//! ([`dump_grown`]), though no trend exists for it: a heap-graph series has no harness in its
//! file and `dla trend` never ran over it.
//!
//! Whole-process columns tie to no finding and stay with the residual rows.

use std::collections::BTreeSet;

use crate::{
    evidence::{Authority, EvidenceKind, EvidenceRecord},
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
    /// Set when the growth was measured over dumps (no trend exists for the series): see
    /// [`dump_grown`]. `harness` is then the harness the dumps came from, empty when unknown.
    pub dump: Option<DumpLead>,
    /// The newest alloc-test bound to the family on a harness about the finding that is not made
    /// for it (a test loop, the project's own workload, or about it only through a probe, a
    /// column, cited evidence or a capture) and that did not decide the lead: it favored leak or
    /// was inconclusive, and growth on that harness may be kept by another holder. See
    /// [`decided_by_alloc_test`].
    pub shared: Option<SharedAllocTest>,
}

/// An alloc-test that ran on a harness not made for the finding and so did not decide its growth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedAllocTest {
    /// Its discriminator evidence.
    pub evidence: EvidenceId,
    /// The harness it ran on.
    pub harness: String,
    /// The runtime of that harness (`jvm`, `node`, `python`, `go`, `rust`), for the command that
    /// makes a driver for the finding.
    pub runtime: String,
    /// What it favored.
    pub favors: crate::discriminator::Favors,
    /// Why the harness is not made for the finding, in words.
    pub why: String,
}

/// A growth a dump series measured, as a lead to decide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DumpLead {
    /// The newest record that makes the lead: a passing dump growth record with a column that
    /// names the finding, or a dump discriminator that favors leak.
    pub evidence: EvidenceId,
    /// The newest dump discriminator that favors leak and names the finding's family, the
    /// result the summarizer's `needed` line cites.
    pub discriminator: Option<EvidenceId>,
    /// The command that wrote the series (`dla summarize heap-graph`), when its file says.
    pub command: Option<String>,
    /// The commands that decide it, each accepted by the ledger as printed, in order.
    pub commands: Vec<String>,
}

impl Grown {
    /// What grew, in words: the column, its series and how it was measured.
    pub fn grew_text(&self) -> String {
        match &self.dump {
            Some(d) => format!(
                "its holder column {} of {} grew over every dump (a dump series of {})",
                self.column,
                self.series,
                d.command.as_deref().unwrap_or("a summarizer")
            ),
            None => format!(
                "its column {} of {} grew (it passed the growth gate, or it is a non-flat allocation-site column)",
                self.column, self.series
            ),
        }
    }

    /// The commands that decide it, in order: the non-leak sibling when it is missing, the test
    /// (the ledger lets a candidate and a closed `unresolved` or `suspected_static_only`
    /// finding go under test), the alloc-test. For growth measured over dumps: the dump route
    /// ([`confirm_commands`]).
    pub fn commands(&self) -> Vec<String> {
        if let Some(d) = &self.dump {
            return d.commands.clone();
        }
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
            match (&self.shared, self.pytest_loop) {
                (Some(s), _) => DriverRoute::Shared {
                    runtime: &s.runtime,
                    harness: &s.harness,
                },
                (None, true) => DriverRoute::PytestLoop,
                (None, false) => DriverRoute::Own,
            },
        )
    }
}

/// Where the alloc-test of a growth goes (see [`commands`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverRoute<'a> {
    /// On the harness that measured the growth.
    Own,
    /// The harness is a pytest loop: a small driver of its own first.
    PytestLoop,
    /// An alloc-test already ran on `harness` (a test loop, the project's own workload, ...),
    /// which is not made for the finding and did not decide it: a driver made for the finding
    /// first. `runtime` is the `--lang` of the harness.
    Shared {
        /// The `--lang` value of the harness.
        runtime: &'a str,
        /// The harness the alloc-test ran on.
        harness: &'a str,
    },
}

/// The commands that decide the growth of `finding` measured by harness `harness`. When the
/// series comes from a pytest loop (`pytest_loop`), the alloc-test is not run on that harness
/// first: it re-runs the whole test suite per run (minutes), so a small driver of its own for
/// the line comes first (seconds per run), and the alloc-test is on that harness. When an
/// alloc-test of a harness not made for the finding did not decide it ([`DriverRoute::Shared`]),
/// the same goes for every runtime: a driver made for the finding, then its alloc-test.
pub fn commands(
    finding: &FindingId,
    harness: &str,
    growth: Option<&str>,
    workload: Option<&str>,
    has_sibling: bool,
    can_test: bool,
    route: DriverRoute<'_>,
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
    match route {
        DriverRoute::PytestLoop => {
            out.push(format!(
                "dla harness gen --lang python --driver --probe {finding}   (a small driver of its own for the line: a unit that calls the function of that line; a run takes seconds, where an alloc-test on {harness} re-runs the whole test suite per run, minutes)"
            ));
            out.push("dla harness run <H>".to_owned());
            out.push("dla trend <S>".to_owned());
            out.push(format!(
                "dla alloc-test <H> --finding {finding}   (on the harness just made, not on {harness})"
            ));
        }
        DriverRoute::Shared {
            runtime,
            harness: shared,
        } => {
            out.push(format!(
                "dla harness gen --lang {runtime} --driver --probe {finding}   (a driver made for {finding}: its unit calls the code that adds to the holder, and nothing else of the program; on {shared} many objects are alive together, so growth there may be kept by another holder)"
            ));
            out.push("dla harness run <H'>".to_owned());
            out.push("dla trend <S>".to_owned());
            out.push(format!(
                "dla alloc-test <H'> --finding {finding}   (on the harness just made, not on {shared})"
            ));
        }
        DriverRoute::Own => {
            out.push(format!("dla alloc-test {harness} --finding {finding}"));
        }
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

/// The leak-side roots whose alloc-tests speak for `root`: itself and the findings merged into it
/// (`dla ledger duplicate`), which are the same leak counted once.
fn merged_roots(ledger: &crate::ledger::Ledger, root: &FindingId) -> Vec<FindingId> {
    std::iter::once(root.clone())
        .chain(ledger.duplicates_of(root).into_iter().cloned())
        .collect()
}

/// True when an alloc-test evidence is bound to the family of `root`, or of a finding merged
/// into it.
pub(crate) fn alloc_tested(view: &RunView, root: &FindingId) -> bool {
    let roots = merged_roots(&view.ledger, root);
    let family: Vec<FindingId> = view
        .ledger
        .findings()
        .filter(|f| roots.contains(&f.id) || f.parent.as_ref().is_some_and(|p| roots.contains(p)))
        .map(|f| f.id.clone())
        .collect();
    view.ledger.evidence_in_order().any(|e| {
        e.kind == EvidenceKind::Discriminator
            && e.result.get("test").and_then(|t| t.as_str()) == Some("plateau+purge")
            && crate::discriminator::finding_of(e).is_some_and(|f| family.contains(&f))
    })
}

/// What the alloc-tests of a family say about the growth of its root, for [`grown_unmeasured`].
enum AllocVerdict {
    /// An alloc-test decided it: the lead is lifted.
    Decided,
    /// None did. `shared` is the newest alloc-test that ran on a harness about the finding but
    /// not made for it, when that is the one that fell short.
    Open(Option<SharedAllocTest>),
}

/// Whether an alloc-test decided the growth of the leak-side root `root`, for the lead of
/// [`grown_unmeasured`] (a passed column of a harness series that points at it).
///
/// An alloc-test says something about a finding only on a harness about it ([`harness_about`]),
/// and growth on a harness not made for it (a test loop, the project's own workload, or a harness
/// about it only through a probe, a column, cited evidence or a capture) may be kept alive by
/// another holder: many objects of the program are alive together there. So of the alloc-tests
/// bound to the family ([`bound_alloc_tests`]) on a harness about the root, the newest decides
/// (time is evidence order, as in [`route_to_confirm`]):
///
/// * it favors non-leak: decided, whatever the harness;
/// * it ran on a hook driver made for the finding (`Target` or `EntryPath`): decided, whatever it
///   favors. Favors leak: the confirm route ([`route_to_confirm`]) and the refusal of a close
///   take over; inconclusive: the close is the model's to make;
/// * it favors leak or is inconclusive on any other harness: not decided, the lead stays and
///   carries that alloc-test ([`SharedAllocTest`]). A newer one on a driver made for the
///   finding lifts it; a newer one on a shared harness replaces an older one made for it, so the
///   lead is back.
///
/// * a driver made for the finding was asked for (`dla harness gen --lang <rt> --driver --probe`)
///   and refused with `driver-unavailable` for the runtime of that harness: decided, as nothing
///   better can be built (a Rust crate with only binaries).
///
/// With no alloc-test on a harness about the root the lead stays, and so it does for a view with no
/// run directory (no manifests to read), where any alloc-test decides, as [`alloc_tested`] says.
fn decided_by_alloc_test(view: &RunView, root: &FindingId) -> AllocVerdict {
    use crate::discriminator::Favors;
    let Some(dir) = view.run_dir.as_deref() else {
        return if alloc_tested(view, root) {
            AllocVerdict::Decided
        } else {
            AllocVerdict::Open(None)
        };
    };
    // The alloc-tests of the findings merged into `root` speak for it: a merge counts the leak
    // once and must not drop what a driver made for one of its findings measured.
    let roots = merged_roots(&view.ledger, root);
    let family: Vec<FindingId> = view
        .ledger
        .findings()
        .filter(|f| roots.contains(&f.id) || f.parent.as_ref().is_some_and(|p| roots.contains(p)))
        .map(|f| f.id.clone())
        .collect();
    let records: Vec<&EvidenceRecord> = view.ledger.evidence_in_order().collect();
    let newest = bound_alloc_tests(&records, &family)
        .into_iter()
        .rev()
        .find_map(|(e, favors)| {
            let harness = e.result.get("harness").and_then(|h| h.as_str())?;
            let m = crate::harness::HarnessManifest::load(dir, harness).ok()?;
            let about = roots.iter().find_map(|r| harness_about(view, &m, r))?;
            Some((e, favors, m, about))
        });
    let Some((e, favors, m, about)) = newest else {
        return AllocVerdict::Open(None);
    };
    let made_for_it =
        m.is_hook_driver() && matches!(about, HarnessAbout::Target | HarnessAbout::EntryPath);
    if favors == Favors::NonLeak || made_for_it {
        return AllocVerdict::Decided;
    }
    // No driver made for the finding can be built here (a Rust crate with only binaries): `dla
    // harness gen --driver --probe` was refused with `driver-unavailable` and recorded, so the
    // alloc-test of the harness there is the best measurement the project allows.
    if crate::harness::HarnessAttempt::driver_unavailable(dir, m.runtime.as_str(), &family) {
        return AllocVerdict::Decided;
    }
    AllocVerdict::Open(Some(SharedAllocTest {
        evidence: e.evidence_id.clone(),
        harness: m.id.clone(),
        runtime: m.runtime.as_str().to_owned(),
        favors,
        why: not_made_for(&m, &about),
    }))
}

/// Why harness `m`, about a finding as `about` says, is not made for it (a test loop, the
/// project's own workload, or how it is about the finding when that is all it is).
pub fn not_made_for(m: &crate::harness::HarnessManifest, about: &HarnessAbout) -> String {
    match about {
        HarnessAbout::OwnWorkload if m.is_test_loop() => "a test loop".to_owned(),
        HarnessAbout::OwnWorkload => "the project's own workload".to_owned(),
        HarnessAbout::Probe => "about it only through a holder probe".to_owned(),
        HarnessAbout::Column(c) => format!("about it only through its column {c}"),
        HarnessAbout::Evidence(e) => format!("about it only through the evidence {e} it cites"),
        HarnessAbout::Capture(e) => format!("about it only through the capture summary {e}"),
        HarnessAbout::Target | HarnessAbout::EntryPath => {
            "a harness that is not a hook driver".to_owned()
        }
    }
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
/// harness last wrote, whose digests are compared (the digest stored in the list, see
/// [`crate::harness::CaptureFile::digest`]; a file deleted to keep the run under its size limit
/// keeps its entry and digest, so the link holds). A capture that a later run replaced no longer
/// matches, so this link says nothing for it.
fn made_from_capture_of(
    run_dir: &std::path::Path,
    e: &crate::evidence::EvidenceRecord,
    harness: &str,
) -> bool {
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
    // The digest stored when the file was listed; the file is hashed only for a list an older
    // version wrote. A deleted (`pruned`) file keeps its digest, so the link holds after it.
    files
        .iter()
        .any(|f| f.digest(run_dir).is_some_and(|d| used.contains(&d)))
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
    for ev in ledger.evidence_in_order() {
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
        for ev in ledger.evidence_in_order() {
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
        .evidence_in_order()
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
        .evidence_in_order()
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
        let records: Vec<&EvidenceRecord> = ledger.evidence_in_order().collect();
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
    let records: Vec<&EvidenceRecord> = ledger.evidence_in_order().collect();
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
/// verdict `grew`, that no alloc-test decided ([`decided_by_alloc_test`]), in the order of the trends. Only open findings and findings closed `unresolved` or
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
            // A finding merged into another is the same leak: its growth is the canonical's,
            // whatever state the canonical is in (a canonical closed `unresolved` lists it again
            // until an alloc-test decides it; a merge must not take it out of the gate).
            let root = f.root_id();
            let root = view.ledger.merged_into(&root).cloned().unwrap_or(root);
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
            {
                continue;
            }
            let shared = match decided_by_alloc_test(view, &root) {
                AllocVerdict::Decided => continue,
                AllocVerdict::Open(shared) => shared,
            };
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
                    .evidence_in_order()
                    .filter(|e| e.kind == want)
                    .find(|e| e.result.get("trend").and_then(|v| v.as_str()) == Some(t.id.as_str()))
                    .map(|e| e.evidence_id.to_string()),
                workload: view
                    .ledger
                    .evidence_in_order()
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
                dump: None,
                shared,
            });
        }
    }
    dump_grown(view, &mut out);
    out
}

/// The harness a dump growth record `records[at]` was measured from: the capture files of the
/// summarize call that wrote it. The call writes one `retention-path` record per dump right
/// before the growth record, each with the dump file as its artifact source, so the harness is
/// the `harness/<H>/` of those paths, when they agree on one.
fn dump_harness(records: &[&EvidenceRecord], at: usize) -> Option<String> {
    let n = records[at]
        .result
        .get("n")
        .and_then(|n| n.as_u64())
        .map_or(usize::MAX, |n| n as usize);
    let mut found: Vec<&str> = Vec::new();
    for e in records[..at]
        .iter()
        .rev()
        // The per-dump records of the same summarize call: whatever kind the summarizer writes
        // (retention paths, histograms, non-heap tables), back to the growth or discriminator
        // records of the call before it.
        .take_while(|e| {
            e.authority == Authority::Dump
                && !is_growth_record(e)
                && e.kind != EvidenceKind::Discriminator
        })
        .take(n)
    {
        if let crate::evidence::Source::Artifact { path, .. } = &e.source
            && let Some(h) = crate::measured_holders::harness_of_path(path)
            && !found.contains(&h)
        {
            found.push(h);
        }
    }
    match found.as_slice() {
        [h] => Some((*h).to_owned()),
        _ => None,
    }
}

/// Growth measured over dumps that nobody decided, as a lead of the same kind as a passed trend
/// column. A summarizer (`dla summarize heap-graph` and its siblings) writes a dump series with
/// its growth evidence and no trend, so [`grown_unmeasured`] cannot read it from `view.trends`;
/// this reads the evidence instead. The rules, kept here and nowhere else:
///
/// * What counts: a `growth-series` or `unit-count-series` record of authority `dump` whose
///   gate passed ([`crate::machine::is_passing_growth`]) with a passed column that names a
///   finding (`finding`, else the first of `candidates` that exists), and a `discriminator` of
///   authority `dump` that favors leak and names a finding. The finding must be a leak-side root
///   (a column or discriminator naming a non-leak sibling counts for nothing), open or closed
///   `unresolved` or `suspected_static_only`, not merged, and not already listed through a trend
///   column.
/// * What lifts it: evidence order is time order, and the newest decisive record about the
///   family of the root decides. A dump discriminator naming the family that does not favor leak
///   (non-leak or inconclusive), or an alloc-test bound to the family ([`bound_alloc_tests`]),
///   newer than the newest record that makes the lead, lifts it; a newer growth record or leak
///   discriminator brings it back.
/// * The harness: [`dump_harness`] of the growth record of the series; empty when none is known.
/// * The commands: [`confirm_commands`] with the newest leak discriminator, the dump route the
///   summarizer prints. With no discriminator to cite (a growth record alone) the sibling needs
///   a result that cannot be named, so the commands are the alloc-test route of [`commands`].
fn dump_grown(view: &RunView, out: &mut Vec<Grown>) {
    use crate::discriminator::{Favors, favors_of, finding_of};
    let ledger = &view.ledger;
    let records: Vec<&EvidenceRecord> = ledger.evidence_in_order().collect();
    let series_of = |e: &EvidenceRecord| {
        e.result
            .get("series")
            .and_then(|s| s.as_str())
            .map(str::to_owned)
    };
    // (record index, root, column, series, is a discriminator)
    let mut hits: Vec<(usize, FindingId, String, Option<String>, bool)> = Vec::new();
    for (at, e) in records.iter().enumerate() {
        if e.authority != Authority::Dump {
            continue;
        }
        let series = series_of(e);
        if series.as_ref().is_some_and(|s| view.gap_series.contains(s)) {
            continue;
        }
        let mut named: Vec<(FindingId, String)> = Vec::new();
        if crate::machine::is_passing_growth(e) {
            let Some(g) = crate::trend::GrowthSummary::from_result(&e.result) else {
                continue;
            };
            for c in g
                .passed
                .iter()
                .filter(|c| !is_whole_process(&c.column))
                // A column of the measurement's own holder (the driver's statics, the agent, the
                // profiler) is no lead: nothing about the project decides it.
                .filter(|c| !view.measurement().is_own_column(&c.column))
            {
                let found = c
                    .finding
                    .iter()
                    .chain(c.candidates.iter())
                    .filter_map(|f| f.parse::<FindingId>().ok())
                    .find(|f| ledger.finding(f).is_some());
                if let Some(f) = found {
                    named.push((f, c.column.clone()));
                }
            }
        } else if e.kind == EvidenceKind::Discriminator
            && favors_of(e) == Some(Favors::Leak)
            && let Some(f) = finding_of(e).filter(|f| ledger.finding(f).is_some())
        {
            let column = e
                .result
                .get("column")
                .and_then(|c| c.as_str())
                .unwrap_or("(a dump column)")
                .to_owned();
            named.push((f, column));
        }
        let is_disc = e.kind == EvidenceKind::Discriminator;
        for (f, column) in named {
            let Some(ff) = ledger.finding(&f) else {
                continue;
            };
            if ff.polarity() != Polarity::Leak || f.is_sibling() {
                continue;
            }
            hits.push((at, ff.root_id(), column, series.clone(), is_disc));
        }
    }
    // The harness a dump series was captured from: the first growth record of the series
    // whose per-dump records name one.
    let series_harness = |s: &str| -> Option<String> {
        records
            .iter()
            .enumerate()
            .filter(|(_, e)| is_growth_record(e) && series_of(e).as_deref() == Some(s))
            .find_map(|(i, _)| dump_harness(&records, i))
    };
    // The one harness the run captured, when only one did: a summary whose records name no
    // harness (a non-heap table, a histogram) can then only be of that capture.
    let captured: BTreeSet<String> = records
        .iter()
        .filter(|e| matches!(e.authority, Authority::Dump | Authority::Live))
        .filter_map(|e| match &e.source {
            crate::evidence::Source::Artifact { path, .. } => {
                crate::measured_holders::harness_of_path(path).map(str::to_owned)
            }
            _ => None,
        })
        .collect();
    let only_captured: Option<String> = (captured.len() == 1)
        .then(|| captured.iter().next().cloned())
        .flatten();
    let mut roots: Vec<FindingId> = Vec::new();
    for (_, r, ..) in &hits {
        if !roots.contains(r) {
            roots.push(r.clone());
        }
    }
    for root in roots {
        let Some(rf) = ledger.finding(&root) else {
            continue;
        };
        if rf.polarity() != Polarity::Leak
            || root.is_sibling()
            || ledger.merged_into(&root).is_some()
            || !matches!(
                rf.state,
                FindingState::Candidate
                    | FindingState::UnderTest
                    | FindingState::Unresolved
                    | FindingState::SuspectedStaticOnly
            )
            || out.iter().any(|g| g.finding == root)
            // The holder is the measurement's own (a finding added for it before this was
            // known): no growth of it is a lead of the project.
            || rf
                .owner
                .as_deref()
                .is_some_and(|o| view.measurement().is_own(o))
        {
            continue;
        }
        let family: Vec<FindingId> = ledger
            .findings()
            .filter(|f| f.id == root || f.parent.as_ref() == Some(&root))
            .map(|f| f.id.clone())
            .collect();
        let Some(lead) = hits.iter().filter(|h| h.1 == root).max_by_key(|h| h.0) else {
            continue;
        };
        let (at, _, column, series, _) = lead;
        // The newest decisive record decides: a newer dump discriminator that does not favor
        // leak lifts the lead, and so does a newer alloc-test bound to the family whose newest
        // one favors non-leak. An alloc-test that favors leak, or cannot tell, does not: the dump
        // discriminator is about this holder, and an alloc-test that favors leak gives a
        // confirm route only on a driver made for the finding (`route_to_confirm`), so lifting
        // the lead on it would let `close-rest` close a finding that two measurements call a leak.
        let later = &records[at + 1..];
        let lifted = later.iter().any(|e| {
            e.kind == EvidenceKind::Discriminator
                && e.authority == Authority::Dump
                && favors_of(e) != Some(Favors::Leak)
                && finding_of(e).is_some_and(|f| family.contains(&f))
        }) || bound_alloc_tests(later, &family)
            .last()
            .is_some_and(|(_, f)| *f == Favors::NonLeak);
        if lifted || explained_since(ledger, &family, &records[*at].evidence_id) {
            continue;
        }
        // The growth record of the series and the harness its dumps came from.
        let growth_at = if is_growth_record(records[*at]) {
            Some(*at)
        } else {
            series.as_ref().and_then(|s| {
                records
                    .iter()
                    .rposition(|e| is_growth_record(e) && series_of(e).as_ref() == Some(s))
            })
        };
        // The harness of the series: the first growth record of it whose dumps name one (a
        // later `dla trend` of the same series writes a growth record with no dumps before it).
        let harness = series
            .as_deref()
            .and_then(&series_harness)
            .or_else(|| growth_at.and_then(|g| dump_harness(&records, g)))
            .unwrap_or_default();
        // A leak confirmed outside this family on growth over the dumps of the same capture (a
        // record of the same series, or of a series whose capture is of the same harness)
        // accounts for it: the other holders that grew in the same process's dumps are the
        // parts that leak keeps alive (the fields of the object that leaks, what its holder
        // retains), not leads of their own. It is no longer a lead and may be closed; a second,
        // separate leak in the same capture is then closed `unresolved` with its growth on
        // record.
        // Every dump series this finding grew in, and the harnesses they were captured from: a
        // part of a leak grows in each capture that leak grows in.
        let grew_in: BTreeSet<String> = hits
            .iter()
            .filter(|h| h.1 == root)
            .filter_map(|h| h.3.clone())
            .collect();
        let mut grew_under: BTreeSet<String> =
            grew_in.iter().filter_map(|s| series_harness(s)).collect();
        if !harness.is_empty() {
            grew_under.insert(harness.clone());
        }
        let explained = ledger
            .findings()
            .filter(|f| {
                matches!(f.state, FindingState::Confirmed | FindingState::Strong)
                    && f.polarity() == Polarity::Leak
                    && !f.id.is_sibling()
                    && !family.contains(&f.id)
            })
            .flat_map(|f| f.supports.iter())
            .filter_map(|e| records.iter().position(|r| &r.evidence_id == e))
            .filter(|&i| is_dump_axis(records[i]))
            .any(|i| {
                let cited = series_of(records[i]);
                cited.as_ref().is_some_and(|c| grew_in.contains(c))
                    || cited
                        .as_deref()
                        .and_then(&series_harness)
                        .or_else(|| dump_harness(&records, i))
                        .or_else(|| only_captured.clone())
                        .is_some_and(|h| grew_under.contains(&h))
            });
        if explained {
            continue;
        }
        let disc = hits
            .iter()
            .filter(|h| h.1 == root && h.4 && records[h.0].authority == Authority::Dump)
            .filter(|h| favors_of(records[h.0]) == Some(Favors::Leak))
            .max_by_key(|h| h.0)
            .map(|h| records[h.0].evidence_id.clone());
        let growth = growth_at.map(|g| records[g].evidence_id.to_string());
        let workload = (!harness.is_empty())
            .then(|| workload_evidence(ledger, &harness))
            .flatten()
            .map(|w| w.to_string());
        let has_sibling = ledger
            .findings()
            .any(|s| s.parent.as_ref() == Some(&root) && s.polarity() == Polarity::NonLeak);
        let mut commands = match &disc {
            Some(d) => confirm_commands(ledger, rf, &harness, d),
            None => commands(
                &root,
                if harness.is_empty() {
                    "<harness>"
                } else {
                    &harness
                },
                growth.as_deref(),
                workload.as_deref(),
                has_sibling,
                crate::machine::is_allowed(rf.state, FindingState::UnderTest),
                DriverRoute::Own,
            ),
        };
        // No placeholder blocks the route: when it needs an annotation that cannot be filled
        // (`--site <file:line>`, `--holder <holder>`), the alloc-test of the harness the dumps came
        // from decides it first, as long as none was run for the family (one that favors
        // non-leak lifts the lead above; one that favors leak leaves the annotations to make).
        if !harness.is_empty()
            && bound_alloc_tests(&records, &family).is_empty()
            && commands
                .iter()
                .any(|c| c.contains("dla ledger annotate") && c.contains('<'))
        {
            commands.insert(
                0,
                format!(
                    "dla alloc-test {harness} --finding {root}   (decides it without the annotations the commands after it ask for: if it favors non-leak the lead is lifted, if it favors leak its next lists the way on)"
                ),
            );
        }
        // The other way out: the growth is the project's own doing and no leak.
        let lead_ev = disc.as_ref().unwrap_or(&records[*at].evidence_id);
        let alternative = explanation_route(
            ledger,
            rf,
            lead_ev,
            workload.as_deref().or(growth.as_deref()),
            latest_dump(&records, growth_at).as_deref(),
        );
        if let Some(last) = commands.last_mut() {
            last.push_str(&format!(
                "   (if the growth is not a leak of the project: {alternative})"
            ));
        }
        let command = series
            .as_deref()
            .zip(view.run_dir.as_deref())
            .and_then(|(s, d)| crate::series::SeriesFile::load(d, s).ok())
            .and_then(|s| s.source.command);
        out.push(Grown {
            finding: root.clone(),
            state: rf.state,
            harness,
            series: series.clone().unwrap_or_default(),
            column: column.clone(),
            growth,
            workload,
            pytest_loop: false,
            has_sibling,
            same_leak: None,
            dump: Some(DumpLead {
                evidence: records[*at].evidence_id.clone(),
                discriminator: disc,
                command,
                commands,
            }),
            shared: None,
        });
    }
}

/// True when a non-leak sibling in `family` was confirmed after the ledger line of the lead's
/// record `lead`: a confirmed non-leak explanation decides the dump lead, and the newest record
/// decides, as for the other lifts (a newer growth record or leak discriminator brings it back,
/// because it is a later line than the confirm).
fn explained_since(
    ledger: &crate::ledger::Ledger,
    family: &[FindingId],
    lead: &EvidenceId,
) -> bool {
    let Some(lead_at) = ledger.evidence_order(lead) else {
        return false;
    };
    family.iter().any(|f| {
        ledger.confirmed_order(f).is_some_and(|at| at > lead_at)
            && ledger
                .finding(f)
                .is_some_and(|x| x.polarity() == Polarity::NonLeak)
    })
}

/// The newest dump file of the series whose growth record is `records[growth_at]`: the artifact
/// path of the last per-dump record written right before it (the dumps are recorded oldest
/// first).
fn latest_dump(records: &[&EvidenceRecord], growth_at: Option<usize>) -> Option<String> {
    let at = growth_at?;
    let n = records[at]
        .result
        .get("n")
        .and_then(|n| n.as_u64())
        .map_or(usize::MAX, |n| n as usize);
    records[..at]
        .iter()
        .rev()
        .take_while(|e| {
            e.authority == Authority::Dump
                && !is_growth_record(e)
                && e.kind != EvidenceKind::Discriminator
        })
        .take(n)
        .find_map(|e| match &e.source {
            crate::evidence::Source::Artifact { path, .. } => Some(path.clone()),
            _ => None,
        })
}

/// What `result.refute_if` of the explanation sibling says.
const EXPLANATION_REFUTE_IF: &str = "a caller of the project adds to the holder";

/// The way out of a dump lead when the growth is no leak of the project: a non-leak sibling of
/// the root that says what keeps the holder (`N-OBSERVER`, the measurement; `N-INTENT`, retention
/// the code asks for), tested, confirmed on the retention-path record of the holder (authority
/// dump) and then the root refuted by it. The commands are accepted as printed, but for the
/// choice between the two hypotheses. `lead` is the dump record the sibling is added on,
/// `test_evidence` what puts it under test (the harness's workload evidence, or on an artifact run
/// the growth record), `latest_dump` the newest dump of the series, which `dla summarize
/// heap-graph --finding` reads for the record of the holder when none names it yet (one dump
/// writes the retention records and no series).
pub fn explanation_route(
    ledger: &crate::ledger::Ledger,
    root: &crate::finding::Finding,
    lead: &EvidenceId,
    test_evidence: Option<&str>,
    latest_dump: Option<&str>,
) -> String {
    let id = &root.id;
    let record = ledger
        .evidence_in_order()
        .filter(|e| crate::machine::shows_holder_of(e, root))
        .last()
        .map(|e| e.evidence_id.to_string());
    let open = ledger.findings().find(|s| {
        s.parent.as_ref() == Some(id)
            && crate::machine::is_explanation_of_holder(s)
            && !s.state.is_dispositioned()
    });
    let mut steps: Vec<String> = Vec::new();
    let sibling = match open {
        Some(s) => s.id.clone(),
        None => {
            let sib = ledger.next_sibling_id(id);
            steps.push(format!(
                "dla ledger add --sibling-of {id} --origin hypothesis --hypothesis N-OBSERVER|N-INTENT --refute-if \"{EXPLANATION_REFUTE_IF}\" --evidence {lead}"
            ));
            sib
        }
    };
    if open.is_none_or(|s| s.state == FindingState::Candidate) {
        steps.push(format!(
            "dla ledger test {sibling} --evidence {}",
            test_evidence.unwrap_or("<workload evidence of the harness>")
        ));
    }
    let cite = match &record {
        Some(r) => r.clone(),
        None => {
            if let Some(dump) = latest_dump {
                steps.push(format!("dla summarize heap-graph {dump} --finding {id}"));
            }
            "<the retention-path record that summarize wrote for the holder>".to_owned()
        }
    };
    steps.push(format!("dla ledger confirm {sibling} --evidence {cite}"));
    // The lead is lifted; the root is closed with the explanation, not refuted (a retention record
    // shows what keeps the holder, not that the growth stops).
    steps.push(format!(
        "dla ledger resolve {id} --as unresolved --reason inconclusive --remedy \"explained by {sibling}: <what keeps the holder, from {cite}>\""
    ));
    steps.join("; ")
}

/// A growth or discriminator record of a dump series (a summarizer's, read from captures), of
/// either authority a summarizer writes (`dump` for a heap dump, `live` for a table it read
/// from the running process).
fn is_dump_axis(e: &EvidenceRecord) -> bool {
    let kind_fits = is_growth_record(e) || e.kind == EvidenceKind::Discriminator;
    match e.authority {
        // A heap dump's summary: whatever axis its series has.
        Authority::Dump => kind_fits,
        // A table read from the running process at each capture: only its dump series.
        Authority::Live => {
            kind_fits
                && (e.kind == EvidenceKind::Discriminator
                    || e.result.get("axis").and_then(|a| a.as_str()) == Some("dump"))
        }
        _ => false,
    }
}

/// A growth-class record (`growth-series` or `unit-count-series`).
fn is_growth_record(e: &EvidenceRecord) -> bool {
    matches!(
        e.kind,
        EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries
    )
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
            description: if g.dump.is_some() {
                format!(
                    "{} is closed {} but {} and nothing decided it since",
                    g.finding,
                    g.state,
                    g.grew_text()
                )
            } else if let Some(sh) = &g.shared {
                format!(
                    "{} is closed {} but {}, and the alloc-test {} on {} ({}, not a driver made for {}) favored {}: growth there may be kept by another holder",
                    g.finding,
                    g.state,
                    g.grew_text(),
                    sh.evidence,
                    sh.harness,
                    sh.why,
                    g.finding,
                    sh.favors
                )
            } else {
                format!(
                    "{} is closed {} but {} and no alloc-test decided it",
                    g.finding,
                    g.state,
                    g.grew_text()
                )
            },
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
                    thread_init: false,
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

    /// [`scene`] with F4 at F1's site, F1 merged into it, and a one-column trend of series S1
    /// (harness H1) whose passed site column at `a.py:1` points at F1; `tested` adds an
    /// alloc-test of F1 that favored `favors`. The findings `grown_unmeasured` lists.
    fn grown_after_merging_f1_into_f4(tested: Option<&str>) -> Vec<String> {
        let (_d, mut l) = scene();
        let mut nf = NewFinding::new(
            Origin::parse("census").unwrap(),
            vec![crate::EvidenceId::new(1)],
        );
        nf.site = Some("a.py:1".into());
        l.add_finding(nf).unwrap();
        let f4 = FindingId::new(4);
        l.duplicate_finding(&f1(), &f4).unwrap();
        assert_eq!(l.merged_into(&f1()), Some(&f4));
        if let Some(favors) = tested {
            alloc_test(&mut l, favors, "H1");
        }
        let state =
            crate::state::RunState::init("/p", None, 3600, None, crate::state::now_utc()).unwrap();
        let mut view = RunView::new(state, l, None);
        let col: crate::trend::ColumnTrend = serde_json::from_value(serde_json::json!({
            "name": SITE_COLUMN, "unit": "count", "kind": "counter", "region": "heap",
            "site": "a.py:1", "n": 8, "noise": 0.0, "rise": 100.0, "shape": "steady",
            "gate": "pass", "gate_failed": [],
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

    #[test]
    fn the_growth_of_a_finding_merged_into_another_is_the_canonicals_and_its_alloc_tests_count() {
        // Merged away, the growth the column measured is not dropped: the canonical carries it.
        assert_eq!(grown_after_merging_f1_into_f4(None), ["F4"]);
        // An alloc-test of the merged finding decides the leak's growth, whichever way it went:
        // the merge did not forget it (a loop-free view: any alloc-test decides).
        for favors in ["leak", "non-leak", "inconclusive"] {
            assert!(
                grown_after_merging_f1_into_f4(Some(favors)).is_empty(),
                "{favors}"
            );
        }
    }

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
        let plain = commands(
            &f,
            "H1",
            Some("E8"),
            Some("E6"),
            true,
            true,
            DriverRoute::Own,
        );
        assert_eq!(plain.last().unwrap(), "dla alloc-test H1 --finding F2");
        let loop_cmds = commands(
            &f,
            "H1",
            Some("E8"),
            Some("E6"),
            true,
            true,
            DriverRoute::PytestLoop,
        );
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

    /// Rewrites the list of `harness` so its one file carries `sha256` and `pruned`.
    fn list_with_digest(dir: &std::path::Path, harness: &str, sha256: &str, pruned: bool) {
        std::fs::write(
            dir.join(format!("harness/{harness}/captures.json")),
            serde_json::json!({"files": [{
                "kind": "alloc", "path": format!("harness/{harness}/out/heap.txt"),
                "label": "p1", "seq": 1, "bytes": 7, "captured_at": "2026-01-01T00:00:00Z",
                "sha256": sha256, "pruned": pruned}]})
            .to_string(),
        )
        .unwrap();
    }

    #[test]
    fn a_deleted_capture_still_links_its_summary_through_the_stored_digest() {
        use sha2::{Digest, Sha256};
        let (d, view) = with_capture_summary("H1", "capture");
        let digest = hex::encode(Sha256::digest(b"capture"));
        list_with_digest(d.path(), "H1", &digest, true);
        std::fs::remove_file(d.path().join("harness/H1/out/heap.txt")).unwrap();
        let m = manifest("H1", serde_json::json!({"targets": ["F1"]}));
        assert!(matches!(
            about(&view, &m, 3),
            Some(HarnessAbout::Capture(_))
        ));
    }

    #[test]
    fn the_stored_digest_is_used_and_the_file_is_not_hashed_when_it_is_there() {
        use sha2::{Digest, Sha256};
        let (d, view) = with_capture_summary("H1", "capture");
        let m = manifest("H1", serde_json::json!({"targets": ["F1"]}));
        // The file now holds other bytes, but the list says what it held when it was listed:
        // the link follows the list.
        std::fs::write(d.path().join("harness/H1/out/heap.txt"), "changed").unwrap();
        let digest = hex::encode(Sha256::digest(b"capture"));
        list_with_digest(d.path(), "H1", &digest, false);
        assert!(matches!(
            about(&view, &m, 3),
            Some(HarnessAbout::Capture(_))
        ));
        // A stored digest the summary does not list says no, whatever the file holds now.
        std::fs::write(d.path().join("harness/H1/out/heap.txt"), "capture").unwrap();
        list_with_digest(d.path(), "H1", &"0".repeat(64), false);
        assert_eq!(about(&view, &m, 3), None);
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

    // ---- growth measured on a shared loop

    /// A view over [`scene`] whose series S1 of harness H1 grew the column `probe:x` tied to F1;
    /// H1 is built from `loop_extra`, and a hook driver H2 targets F1.
    fn shared_loop(l: Ledger, loop_extra: serde_json::Value) -> (tempfile::TempDir, RunView) {
        let dir = tempfile::tempdir().unwrap();
        manifest("H1", loop_extra).save(dir.path()).unwrap();
        manifest(
            "H2",
            serde_json::json!({"runtime": "jvm", "targets": ["F1"]}),
        )
        .save(dir.path())
        .unwrap();
        let mut view = view_with(l, serde_json::json!({"name": "probe:x", "finding": "F1"}));
        view.run_dir = Some(dir.path().to_path_buf());
        (dir, view)
    }

    fn junit_loop() -> serde_json::Value {
        serde_json::json!({"runtime": "jvm", "tools": [crate::harness::JUNIT_TOOL]})
    }

    fn grown_f1(view: &RunView) -> Option<Grown> {
        grown_unmeasured(view)
            .into_iter()
            .find(|g| g.finding == f1())
    }

    #[test]
    fn a_loop_alloc_test_that_favors_leak_or_is_inconclusive_leaves_the_lead() {
        for favors in ["leak", "inconclusive"] {
            let (_d, mut l) = scene();
            let e = alloc_test(&mut l, favors, "H1");
            let (_r, view) = shared_loop(l, junit_loop());
            let g = grown_f1(&view).unwrap_or_else(|| panic!("{favors}: no lead"));
            let shared = g.shared.clone().unwrap();
            assert_eq!(shared.evidence, e);
            assert_eq!(shared.harness, "H1");
            assert_eq!(shared.runtime, "jvm");
            assert_eq!(shared.favors.to_string(), favors);
            assert_eq!(shared.why, "a test loop");
            let cmds = g.commands();
            // The driver made for the finding comes first, then its alloc-test: not on the loop.
            let gen_at = cmds
                .iter()
                .position(|c| {
                    c.starts_with(
                        "dla harness gen --lang jvm --driver --probe F1   (a driver made for F1",
                    )
                })
                .unwrap_or_else(|| panic!("{cmds:?}"));
            assert!(cmds[gen_at].contains("on H1 many objects are alive together"));
            assert_eq!(cmds[gen_at + 1], "dla harness run <H'>");
            assert_eq!(cmds[gen_at + 2], "dla trend <S>");
            assert_eq!(
                cmds[gen_at + 3],
                "dla alloc-test <H'> --finding F1   (on the harness just made, not on H1)"
            );
            assert!(
                !cmds.iter().any(|c| c.starts_with("dla alloc-test H1")),
                "{cmds:?}"
            );
        }
    }

    #[test]
    fn a_loop_alloc_test_that_favors_non_leak_lifts_the_lead() {
        let (_d, mut l) = scene();
        alloc_test(&mut l, "non-leak", "H1");
        let (_r, view) = shared_loop(l, junit_loop());
        assert_eq!(grown_f1(&view), None);
    }

    #[test]
    fn an_alloc_test_on_a_driver_made_for_the_finding_decides_whatever_it_favors() {
        for (favors, route) in [("leak", true), ("inconclusive", false), ("non-leak", false)] {
            let (_d, mut l) = scene();
            growth(&mut l, "S2");
            alloc_test(&mut l, "leak", "H1");
            let e = alloc_test(&mut l, favors, "H2");
            let (_r, view) = shared_loop(l, junit_loop());
            assert_eq!(grown_f1(&view), None, "{favors}");
            // Favoring leak it is the confirm route that holds the finding now.
            let r = route_to_confirm(&view, &f1());
            assert_eq!(r.is_some(), route, "{favors}");
            if let Some(r) = r {
                assert_eq!((r.discriminator, r.harness.as_str()), (e, "H2"));
            }
        }
    }

    #[test]
    fn the_newest_alloc_test_decides_between_a_driver_made_for_it_and_the_loop() {
        // Inconclusive on the driver made for it, then a newer leak on the loop: the lead is back.
        let (_d, mut l) = scene();
        alloc_test(&mut l, "inconclusive", "H2");
        let loop_test = alloc_test(&mut l, "leak", "H1");
        let (_r, view) = shared_loop(l, junit_loop());
        let g = grown_f1(&view).expect("the newest alloc-test is the loop's");
        assert_eq!(g.shared.unwrap().evidence, loop_test);
        // The other order: the driver's result is the newest and decides.
        let (_d, mut l) = scene();
        alloc_test(&mut l, "leak", "H1");
        alloc_test(&mut l, "inconclusive", "H2");
        let (_r, view) = shared_loop(l, junit_loop());
        assert_eq!(grown_f1(&view), None);
        // A leak on the driver, then a leak on the loop: the loop's is the newest, so there is no
        // route (`route_to_confirm` reads the same rule) and the lead is back.
        let (_d, mut l) = scene();
        alloc_test(&mut l, "leak", "H2");
        alloc_test(&mut l, "leak", "H1");
        let (_r, view) = shared_loop(l, junit_loop());
        assert!(grown_f1(&view).is_some());
        assert_eq!(route_to_confirm(&view, &f1()), None);
    }

    #[test]
    fn a_driver_refused_as_unavailable_lets_the_loop_alloc_test_decide() {
        let f = f1();
        for (lang, finding, lifted) in [
            // The refusal of the runtime of the loop, for the finding: nothing better can be built.
            ("jvm", "F1", true),
            // Another runtime, or another finding: the lead stays.
            ("python", "F1", false),
            ("jvm", "F2", false),
        ] {
            let (_d, mut l) = scene();
            alloc_test(&mut l, "leak", "H1");
            let (r, view) = shared_loop(l, junit_loop());
            assert!(grown_f1(&view).is_some());
            crate::harness::HarnessAttempt::save_driver_unavailable(
                r.path(),
                lang,
                &[finding.parse().unwrap()],
                "driver mode needs a crate with a library target",
            )
            .unwrap();
            assert_eq!(grown_f1(&view).is_none(), lifted, "{lang} {finding} {f}");
        }
    }

    #[test]
    fn the_projects_own_workload_and_a_probe_only_harness_behave_like_the_test_loop() {
        for (extra, why) in [
            (
                serde_json::json!({"mode": "command", "workload": "go:main:.:./cmd/x"}),
                "the project's own workload",
            ),
            (
                serde_json::json!({"probes": [{"id": "P1", "finding": "F1",
                    "target": {"kind": "py-module", "module": "m"}}]}),
                "about it only through a holder probe",
            ),
        ] {
            let (_d, mut l) = scene();
            alloc_test(&mut l, "leak", "H1");
            let (_r, view) = shared_loop(l, extra.clone());
            let g = grown_f1(&view).unwrap_or_else(|| panic!("{extra}"));
            assert_eq!(g.shared.unwrap().why, why);
        }
    }

    #[test]
    fn an_alloc_test_on_a_harness_not_about_the_finding_decides_nothing() {
        let (_d, mut l) = scene();
        alloc_test(&mut l, "non-leak", "H9");
        let (_r, view) = shared_loop(l, junit_loop());
        // No manifest H9: the alloc-test is not about the finding on any harness we know.
        let g = grown_f1(&view).unwrap();
        assert_eq!(g.shared, None);
    }

    #[test]
    fn without_a_run_directory_any_alloc_test_still_lifts_the_lead() {
        let (_d, mut l) = scene();
        alloc_test(&mut l, "leak", "H1");
        let mut view = view_with(l, serde_json::json!({"name": "probe:x", "finding": "F1"}));
        view.run_dir = None;
        assert_eq!(grown_f1(&view), None);
    }

    #[test]
    fn a_closed_finding_that_a_loop_alloc_test_did_not_decide_is_listed_in_its_words() {
        let (_d, mut l) = scene();
        let e = alloc_test(&mut l, "leak", "H1");
        close_f1(&mut l);
        let (_r, view) = shared_loop(l, junit_loop());
        let grown = grown_unmeasured(&view);
        let items = items(&view, &grown);
        assert_eq!(items.len(), 1);
        let d = &items[0].description;
        assert!(d.contains("F1 is closed unresolved"), "{d}");
        assert!(
            d.contains(&format!(
                "the alloc-test {e} on H1 (a test loop, not a driver made for F1) favored leak: growth there may be kept by another holder"
            )),
            "{d}"
        );
        assert!(!d.contains("no alloc-test decided it"), "{d}");
        assert!(
            items[0]
                .fix
                .contains("dla harness gen --lang jvm --driver --probe F1"),
            "{}",
            items[0].fix
        );
    }

    // ---- growth measured over dumps

    /// One dump record of authority `dump`.
    fn dump_ev(
        l: &mut Ledger,
        kind: EvidenceKind,
        path: Option<&str>,
        result: serde_json::Value,
    ) -> EvidenceId {
        l.add_evidence(NewEvidence {
            source: match path {
                Some(p) => Source::Artifact {
                    path: p.into(),
                    sha256: "0".repeat(64),
                },
                None => Source::Repo {
                    commit: "c".into(),
                    path: None,
                },
            },
            tool: Tool {
                name: "dla-summarize".into(),
                version: "1".into(),
            },
            authority: Authority::Dump,
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

    /// What `dla summarize heap-graph` writes over three dumps of harness H1: a retention record
    /// per dump, the growth record with a passed column that names `finding`. Returns its id.
    fn dump_growth(l: &mut Ledger, series: &str, finding: &str) -> EvidenceId {
        for i in 1..=3 {
            dump_ev(
                l,
                EvidenceKind::RetentionPath,
                Some(&format!(
                    ".dla/R1/harness/H1/out/p{i}/capture/7/heap-{i}.heapsnapshot"
                )),
                serde_json::json!({"holders": []}),
            );
        }
        dump_ev(
            l,
            EvidenceKind::GrowthSeries,
            None,
            serde_json::json!({"series": series, "axis": "dump", "n": 3, "gate": "pass",
                "passed": [{"column": "holder:app.Conn.request", "finding": finding}]}),
        )
    }

    fn dump_disc(l: &mut Ledger, favors: &str, finding: &str, series: &str) -> EvidenceId {
        dump_ev(
            l,
            EvidenceKind::Discriminator,
            None,
            serde_json::json!({"test": "dump-plateau", "favors": favors, "finding": finding,
                "series": series, "column": "holder:app.Conn.request",
                "reason": "retained bytes and count grew over every interval"}),
        )
    }

    /// A view over the ledger with no trend, the way a run with only a summarizer's series is.
    fn no_trend_view(l: Ledger) -> RunView {
        let state =
            crate::state::RunState::init("/p", None, 3600, None, crate::state::now_utc()).unwrap();
        RunView::new(state, l, None)
    }

    fn close_f1(l: &mut Ledger) {
        l.transition(
            &f1(),
            FindingState::Unresolved,
            Some(crate::finding::UnresolvedReason::Inconclusive),
            Some("x".into()),
            vec![],
            "t",
        )
        .unwrap();
    }

    #[test]
    fn a_dump_growth_with_a_leak_discriminator_is_grown_and_decided_by_the_dump_route() {
        let (_d, mut l) = scene();
        let g = dump_growth(&mut l, "S4", "F1");
        let d = dump_disc(&mut l, "leak", "F1", "S4");
        let view = no_trend_view(l);
        let grown = grown_unmeasured(&view);
        assert_eq!(grown.len(), 1, "{grown:?}");
        let g1 = &grown[0];
        assert_eq!((g1.finding.clone(), g1.series.as_str()), (f1(), "S4"));
        assert_eq!(g1.column, "holder:app.Conn.request");
        // The harness comes from the capture files of the dumps.
        assert_eq!(g1.harness, "H1");
        let dump = g1.dump.as_ref().unwrap();
        assert_eq!(dump.discriminator.as_ref(), Some(&d));
        // The route needs a holder nobody can fill in, so the alloc-test of the harness the dumps
        // came from is offered first; the rest is the confirm route, with the way out of a growth
        // that is no leak of the project after its last command.
        let cmds = g1.commands();
        assert_eq!(cmds.len(), 4, "{cmds:?}");
        assert!(
            cmds[0].starts_with("dla alloc-test H1 --finding F1   ("),
            "{cmds:?}"
        );
        assert_eq!(cmds[1], format!("dla ledger refute F1.n1 --evidence {d}"));
        assert_eq!(cmds[2], "dla ledger annotate F1 --holder <holder>");
        assert!(
            cmds[3].starts_with(&format!("dla ledger confirm F1 --evidence {g}   (if the growth is not a leak of the project: dla ledger add --sibling-of F1 --origin hypothesis --hypothesis N-OBSERVER|N-INTENT")),
            "{cmds:?}"
        );
        assert!(
            cmds[3].contains("dla ledger test F1.n2 --evidence")
                && cmds[3].contains("dla ledger confirm F1.n2 --evidence")
                && cmds[3].contains("dla ledger resolve F1 --as unresolved --reason inconclusive")
                && !cmds[3].contains("dla ledger refute F1 --evidence"),
            "{cmds:?}"
        );
        assert!(g1.grew_text().contains("grew over every dump"));
    }

    #[test]
    fn once_an_alloc_test_ran_or_nothing_needs_filling_in_the_route_has_no_alloc_test_first() {
        // An alloc-test of the family that favored leak: the annotations are the model's to make.
        let (_d, mut l) = scene();
        dump_growth(&mut l, "S4", "F1");
        dump_disc(&mut l, "leak", "F1", "S4");
        alloc_test(&mut l, "leak", "H1");
        let grown = grown_unmeasured(&no_trend_view(l));
        let cmds = grown[0].commands();
        assert!(
            cmds.iter().all(|c| !c.starts_with("dla alloc-test")),
            "{cmds:?}"
        );
        assert!(
            cmds.iter()
                .any(|c| c.contains("annotate F1 --holder <holder>"))
        );
    }

    /// F1 annotated with the owner and holder the dump growth names, and the leak discriminator.
    fn annotated_dump_lead() -> (tempfile::TempDir, Ledger, EvidenceId) {
        let (d, mut l) = scene();
        l.annotate_record(FindingAnnotation {
            owner: Some("app.Conn.request".into()),
            holder: Some(crate::finding::Holder::StaticGlobal),
            ..FindingAnnotation::new(f1())
        })
        .unwrap();
        dump_growth(&mut l, "S4", "F1");
        let disc = dump_disc(&mut l, "leak", "F1", "S4");
        (d, l, disc)
    }

    /// The N-OBSERVER sibling of F1 added on `lead`, put under test, and confirmed on the
    /// retention-path record of the holder.
    fn confirm_explanation(l: &mut Ledger, lead: &EvidenceId) -> FindingId {
        let workload = dump_ev(
            l,
            EvidenceKind::Workload,
            None,
            serde_json::json!({"harness": "H1", "mode": "driver", "runtime": "jvm"}),
        );
        let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![lead.clone()]);
        nf.sibling_of = Some(f1());
        nf.hypothesis = Some("N-OBSERVER".into());
        nf.refute_if = Some("a caller of the project adds to the holder".into());
        let sib = match l.add_finding(nf).unwrap().record {
            crate::Record::FindingAdded { finding } => finding.id,
            other => panic!("{other:?}"),
        };
        l.transition(
            &sib,
            FindingState::UnderTest,
            None,
            None,
            vec![workload],
            "t",
        )
        .unwrap();
        let record = dump_ev(
            l,
            EvidenceKind::RetentionPath,
            Some(".dla/R1/harness/H1/out/p3/capture/7/heap-3.heapsnapshot"),
            serde_json::json!({"holders": [{"key": "static:app.Conn.request", "retained": 9}]}),
        );
        l.transition(&sib, FindingState::Confirmed, None, None, vec![record], "t")
            .unwrap();
        sib
    }

    #[test]
    fn a_confirmed_non_leak_explanation_lifts_the_dump_lead_and_a_newer_growth_brings_it_back() {
        let (_d, mut l, disc) = annotated_dump_lead();
        assert_eq!(grown_unmeasured(&no_trend_view(l.clone())).len(), 1);
        confirm_explanation(&mut l, &disc);
        assert!(
            grown_unmeasured(&no_trend_view(l.clone())).is_empty(),
            "the newest record is the confirmed explanation"
        );
        // A newer growth record of the holder: the newest record decides again.
        dump_growth(&mut l, "S5", "F1");
        let grown = grown_unmeasured(&no_trend_view(l));
        assert_eq!(grown.len(), 1, "{grown:?}");
        assert_eq!(grown[0].series, "S5");
    }

    #[test]
    fn the_route_of_a_dump_lead_cites_the_retention_record_that_names_its_holder_when_there_is_one()
    {
        let (_d, mut l, _disc) = annotated_dump_lead();
        let record = dump_ev(
            &mut l,
            EvidenceKind::RetentionPath,
            Some(".dla/R1/harness/H1/out/p3/capture/7/heap-3.heapsnapshot"),
            serde_json::json!({"holders": [{"key": "static:app.Conn.request", "retained": 9}]}),
        );
        let grown = grown_unmeasured(&no_trend_view(l));
        let last = grown[0].commands().pop().unwrap();
        assert!(
            last.contains(&format!("dla ledger confirm F1.n2 --evidence {record}"))
                && last.contains(&format!(
                    "dla ledger resolve F1 --as unresolved --reason inconclusive --remedy \"explained by F1.n2: <what keeps the holder, from {record}>\""
                )),
            "{last}"
        );
        assert!(!last.contains("dla summarize heap-graph"), "{last}");
    }

    #[test]
    fn a_column_of_the_measurements_own_holder_is_no_dump_lead() {
        let (_d, mut l) = scene();
        for i in 1..=3 {
            dump_ev(
                &mut l,
                EvidenceKind::RetentionPath,
                Some(&format!(
                    ".dla/R1/harness/H1/out/p{i}/capture/7/heap-{i}.heapsnapshot"
                )),
                serde_json::json!({"holders": []}),
            );
        }
        dump_ev(
            &mut l,
            EvidenceKind::GrowthSeries,
            None,
            serde_json::json!({"series": "S4", "axis": "dump", "n": 3, "gate": "pass",
                "passed": [{"column": "holder:static:dla.harness.DlaDump.ROOT_CLASS",
                            "finding": "F1"}]}),
        );
        assert!(grown_unmeasured(&no_trend_view(l)).is_empty());
    }

    #[test]
    fn a_dump_growth_alone_is_a_lead_and_a_discriminator_alone_is_one() {
        let (_d, mut l) = scene();
        dump_growth(&mut l, "S4", "F1");
        let grown = grown_unmeasured(&no_trend_view(l));
        assert_eq!(grown.len(), 1);
        assert!(grown[0].dump.as_ref().unwrap().discriminator.is_none());
        // The discriminator by itself, with no growth record of its series: no harness known.
        let (_d, mut l) = scene();
        let d = dump_disc(&mut l, "leak", "F1", "S4");
        let grown = grown_unmeasured(&no_trend_view(l));
        assert_eq!(grown.len(), 1);
        assert_eq!(grown[0].harness, "");
        assert_eq!(
            grown[0].commands()[0],
            format!("dla ledger refute F1.n1 --evidence {d}")
        );
    }

    #[test]
    fn a_closed_finding_with_a_dump_growth_is_listed_in_words_that_name_the_dump() {
        for close in [true, false] {
            let (_d, mut l) = scene();
            dump_growth(&mut l, "S4", "F1");
            dump_disc(&mut l, "leak", "F1", "S4");
            if close {
                close_f1(&mut l);
            }
            let view = no_trend_view(l);
            let grown = grown_unmeasured(&view);
            assert_eq!(grown.len(), 1, "close={close}");
            let items = items(&view, &grown);
            if close {
                assert_eq!(items.len(), 1);
                let d = &items[0].description;
                assert!(d.contains("F1 is closed unresolved"), "{d}");
                assert!(
                    d.contains("holder column holder:app.Conn.request of S4"),
                    "{d}"
                );
                assert!(d.contains("grew over every dump"), "{d}");
                assert!(d.contains("nothing decided it since"), "{d}");
                assert!(!d.contains("no alloc-test decided it"), "{d}");
                assert!(
                    items[0].fix.contains("dla ledger confirm F1"),
                    "{}",
                    items[0].fix
                );
            } else {
                // An open one is listed as a finding item by the status, not here.
                assert!(items.is_empty());
            }
        }
    }

    #[test]
    fn a_newer_non_leak_dump_discriminator_or_alloc_test_lifts_the_lead() {
        for lift in ["non-leak", "inconclusive"] {
            let (_d, mut l) = scene();
            dump_growth(&mut l, "S4", "F1");
            dump_disc(&mut l, "leak", "F1", "S4");
            dump_disc(&mut l, lift, "F1", "S4");
            assert!(grown_unmeasured(&no_trend_view(l)).is_empty(), "{lift}");
        }
        // A newer alloc-test bound to the family that favors non-leak lifts it.
        let (_d, mut l) = scene();
        dump_growth(&mut l, "S4", "F1");
        dump_disc(&mut l, "leak", "F1", "S4");
        alloc_test(&mut l, "non-leak", "H1");
        assert!(grown_unmeasured(&no_trend_view(l)).is_empty());
        // One that favors leak, or cannot tell, does not: on a harness that is not a driver
        // made for the finding it gives no confirm route, and the dump still says leak.
        for favors in ["leak", "inconclusive"] {
            let (_d, mut l) = scene();
            dump_growth(&mut l, "S4", "F1");
            dump_disc(&mut l, "leak", "F1", "S4");
            alloc_test(&mut l, favors, "H1");
            let g = grown_unmeasured(&no_trend_view(l));
            assert_eq!(g.len(), 1, "{favors}");
            assert!(g[0].dump.is_some(), "{favors}");
        }
        // The newest alloc-test decides among several.
        let (_d, mut l) = scene();
        dump_growth(&mut l, "S4", "F1");
        dump_disc(&mut l, "leak", "F1", "S4");
        alloc_test(&mut l, "leak", "H1");
        alloc_test(&mut l, "non-leak", "H1");
        assert!(grown_unmeasured(&no_trend_view(l)).is_empty());
        // The non-leak discriminator names the sibling: the family is read.
        let (_d, mut l) = scene();
        dump_growth(&mut l, "S4", "F1");
        dump_disc(&mut l, "non-leak", "F1.n1", "S4");
        assert!(grown_unmeasured(&no_trend_view(l)).is_empty());
        // An older one does not: growth measured after it is the newest word.
        let (_d, mut l) = scene();
        dump_disc(&mut l, "non-leak", "F1", "S3");
        dump_growth(&mut l, "S4", "F1");
        dump_disc(&mut l, "leak", "F1", "S4");
        assert_eq!(grown_unmeasured(&no_trend_view(l)).len(), 1);
    }

    #[test]
    fn a_dump_growth_of_a_sibling_a_decided_root_or_a_failed_gate_is_no_lead() {
        // The growth names the non-leak sibling.
        let (_d, mut l) = scene();
        dump_growth(&mut l, "S4", "F1.n1");
        dump_disc(&mut l, "leak", "F1.n1", "S4");
        assert!(grown_unmeasured(&no_trend_view(l)).is_empty());
        // The root is refuted (a confirmed one is covered by the CLI test: the confirm itself
        // needs the ledger's whole route).
        let (_d, mut l) = scene();
        dump_growth(&mut l, "S4", "F1");
        let d = dump_disc(&mut l, "non-leak", "F1", "S4");
        l.transition(&f1(), FindingState::Refuted, None, None, vec![d], "t")
            .unwrap();
        assert!(grown_unmeasured(&no_trend_view(l)).is_empty());
        // A growth record whose gate failed, or one that is not a dump's.
        let (_d, mut l) = scene();
        dump_ev(
            &mut l,
            EvidenceKind::GrowthSeries,
            None,
            serde_json::json!({"series": "S4", "gate": "fail",
                "passed": [{"column": "holder:app.Conn.request", "finding": "F1"}]}),
        );
        add_ev(
            &mut l,
            EvidenceKind::GrowthSeries,
            serde_json::json!({"series": "S5", "gate": "pass",
                "passed": [{"column": "holder:app.Conn.request", "finding": "F1"}]}),
        );
        assert!(grown_unmeasured(&no_trend_view(l)).is_empty());
    }

    #[test]
    fn a_finding_with_a_trend_column_and_a_dump_growth_is_one_lead() {
        let (_d, mut l) = scene();
        dump_growth(&mut l, "S4", "F1");
        dump_disc(&mut l, "leak", "F1", "S4");
        let mut view = no_trend_view(l);
        let col: crate::trend::ColumnTrend = serde_json::from_value(serde_json::json!({
            "name": "py.blocks@a.py:1", "unit": "count", "kind": "counter", "region": "heap",
            "site": "a.py:1", "n": 8, "noise": 0.0, "rise": 100.0, "shape": "steady",
            "gate": "pass", "gate_failed": [], "candidates": ["F1"],
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
        let grown = grown_unmeasured(&view);
        assert_eq!(grown.len(), 1, "{grown:?}");
        // The trend's own entry stands.
        assert!(grown[0].dump.is_none());
        assert_eq!(items(&view, &grown).len(), 0);
    }

    #[test]
    fn dumps_of_two_harnesses_name_no_harness() {
        let (_d, mut l) = scene();
        for (i, h) in ["H1", "H2", "H1"].iter().enumerate() {
            dump_ev(
                &mut l,
                EvidenceKind::RetentionPath,
                Some(&format!(".dla/R1/harness/{h}/out/p{i}/heap-1.heapsnapshot")),
                serde_json::json!({}),
            );
        }
        dump_ev(
            &mut l,
            EvidenceKind::GrowthSeries,
            None,
            serde_json::json!({"series": "S4", "n": 3, "gate": "pass",
                "passed": [{"column": "holder:x", "finding": "F1"}]}),
        );
        let grown = grown_unmeasured(&no_trend_view(l));
        assert_eq!(grown[0].harness, "");
    }

    #[test]
    fn an_explanation_confirmed_in_a_workers_slice_is_after_a_lead_of_the_main_ledger() {
        // The lead (a growth record) is a line of the main ledger, which a worker's slice does
        // not hold; the worker confirms a non-leak sibling of its own finding afterwards.
        let dir = tempfile::tempdir().unwrap();
        crate::state::RunState::init("/p", None, 3600, None, crate::state::now_utc())
            .unwrap()
            .save(dir.path())
            .unwrap();
        let mut main = Ledger::create(dir.path()).unwrap();
        let hit = add_ev(&mut main, EvidenceKind::StaticHit, serde_json::json!({}));
        let ws = add_ev(
            &mut main,
            EvidenceKind::Workload,
            serde_json::json!({"harness": "H1"}),
        );
        let flat = add_ev(
            &mut main,
            EvidenceKind::Discriminator,
            serde_json::json!({"test": "plateau+purge", "favors": "non-leak", "finding": "w/F1",
                "harness": "H1", "series": "S1"}),
        );
        let lead = growth(&mut main, "S1");
        let mut w = Ledger::load_for(dir.path(), Some("w")).unwrap();
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![hit.clone()]);
        nf.site = Some("a.py:1".into());
        nf.holder = Some(crate::finding::Holder::StaticGlobal);
        w.add_finding(nf).unwrap();
        let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![hit]);
        nf.sibling_of = Some("w/F1".parse().unwrap());
        nf.hypothesis = Some("N-RELEASE".into());
        nf.refute_if = Some(REFUTE_IF.into());
        nf.remover = Some("a.py:30".into());
        w.add_finding(nf).unwrap();
        let (root, sib): (FindingId, FindingId) =
            ("w/F1".parse().unwrap(), "w/F1.n1".parse().unwrap());
        let family = vec![root.clone(), sib.clone()];
        // Nothing confirmed yet.
        assert!(!explained_since(&w, &family, &lead));
        for (f, to, by) in [
            (&root, FindingState::UnderTest, &ws),
            (&root, FindingState::Refuted, &flat),
            (&sib, FindingState::UnderTest, &ws),
            (&sib, FindingState::Confirmed, &flat),
        ] {
            w.transition(f, to, None, None, vec![by.clone()], "t")
                .unwrap();
        }
        assert!(
            explained_since(&w, &family, &lead),
            "confirmed after the lead"
        );
        // A growth record of the worker after the confirm brings the lead back.
        let newer = growth(&mut w, "S2");
        assert_eq!(newer.to_string(), "w/E1");
        assert!(!explained_since(&w, &family, &newer));
    }
}
