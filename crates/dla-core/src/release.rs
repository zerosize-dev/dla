//! `N-RELEASE`: growth that exists only while the caller skips a release the API requires.
//!
//! A library that needs `stop()`, `close()`, `dispose()`, `unsubscribe()` or `cancel()` called
//! when a unit of work ends is not leaking when a caller skips it: the growth is the caller's
//! misuse, and the library's own leak at that holder (if any) is another one, found with the
//! release called. Three rules follow from that, all here:
//!
//! * **N-RELEASE is decided by a run that called the release.** The sibling names the line that
//!   releases the holder (its `remover`). An alloc-test decides it only when it ran on a harness
//!   made for the finding and the remover line ran there ([`remover_run`]): a flat series from a
//!   driver that never reached the release says nothing about what the release does. Where a
//!   runtime cannot count lines (the JVM, native code) the harness is accepted and the decision
//!   carries the note [`NOT_SHOWN`].
//! * **Drivers that disagree hold the confirm** ([`drivers_disagree`]): two drivers made for the
//!   finding whose newest alloc-tests favor leak and non-leak do not decide it until something
//!   names and tests what differs between them.
//! * **An explanation that names a conditional remover asks for the release run**
//!   ([`release_untested`]).
//!
//! The ledger itself never reads a run directory, so these checks run in the command layer
//! (`dla ledger confirm`, `refute`) and in the gate, over a [`RunView`].

use std::path::Path;

use serde_json::Value;

use crate::{
    discriminator::Favors,
    error::Refusal,
    evidence::{EvidenceKind, EvidenceRecord},
    finding::{Finding, FindingState, Polarity, nonleak_id},
    grown::{HarnessAbout, bound_alloc_tests, harness_about},
    harness::{HarnessManifest, LineRuns},
    ids::{EvidenceId, FindingId},
    ledger::Ledger,
    record::{Record, RemovalClaim},
    view::RunView,
};

/// The id of the archetype.
pub const ID: &str = "N-RELEASE";

/// The note a decision on a runtime that cannot count lines carries, printed by the report and
/// by `dla ledger show`.
pub const NOT_SHOWN: &str = "remover run not shown";

/// Refusal codes of this module.
pub mod codes {
    /// Two drivers made for the finding disagree and nothing names the difference.
    pub const DRIVERS_DISAGREE: &str = "drivers-disagree";
    /// The explanation says the removal is conditional and the release was never tested.
    pub const RELEASE_UNTESTED: &str = "release-untested";
    /// The alloc-test cited for an `N-RELEASE` sibling is not one that called the release.
    pub const RELEASE_NOT_RUN: &str = "release-not-run";
}

/// True for a non-leak sibling whose hypothesis is `N-RELEASE`.
pub fn is_release(f: &Finding) -> bool {
    f.polarity() == Polarity::NonLeak && f.hypothesis.as_deref().and_then(nonleak_id) == Some(ID)
}

/// The `N-RELEASE` siblings of the family rooted at `root`, in ledger order.
pub fn release_siblings<'a>(ledger: &'a Ledger, root: &FindingId) -> Vec<&'a Finding> {
    ledger
        .findings()
        .filter(|f| f.parent.as_ref() == Some(root) && is_release(f))
        .collect()
}

/// True when an `N-RELEASE` sibling of `root` is refuted: the release was called and the holder
/// still grew, so the growth is a leak of the project.
pub fn release_refuted(ledger: &Ledger, root: &FindingId) -> bool {
    release_siblings(ledger, root)
        .iter()
        .any(|s| s.state == FindingState::Refuted)
}

/// The confirmed `N-RELEASE` sibling of `root`, if any: the growth is flat with the release
/// called, so it exists only while callers skip it.
pub fn release_confirmed<'a>(ledger: &'a Ledger, root: &FindingId) -> Option<&'a Finding> {
    release_siblings(ledger, root)
        .into_iter()
        .find(|s| s.state == FindingState::Confirmed)
}

/// What a run showed about the remover line of an `N-RELEASE` sibling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoverRun {
    /// The line ran this many times.
    Ran(u64),
    /// The runtime counted the line and it never ran.
    NotRun,
    /// The runtime counts lines, but the run of the harness did not ask about this one (it ran
    /// before the sibling named it): the run has to be repeated.
    NotAsked,
    /// The runtime cannot count the line here; the string says why.
    NotShown(String),
}

/// One count of a coverage record of a harness run.
enum Counted {
    Count(u64),
    Unknown(String),
    Absent,
}

fn same_file(printed: &str, file: &str) -> bool {
    let printed = printed.trim_start_matches("./");
    let file = file.trim_start_matches("./");
    printed == file
        || printed.ends_with(&format!("/{file}"))
        || file.ends_with(&format!("/{printed}"))
}

fn split_site(site: &str) -> Option<(&str, u64)> {
    let (file, line) = site.rsplit_once(':')?;
    Some((file, line.parse().ok()?))
}

/// The coverage record `executed.json` of the last run of `harness`, read for `remover`.
fn counted_in_coverage(run_dir: &Path, harness: &str, remover: &str) -> Result<Counted, String> {
    let path = HarnessManifest::dir_in(run_dir, harness).join("executed.json");
    let Some(doc) = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    else {
        return Err("the harness run wrote no coverage record".to_owned());
    };
    if doc.get("kind").and_then(Value::as_str) != Some("functions") {
        let note = doc.get("note").and_then(Value::as_str).unwrap_or("");
        return Err(if note.is_empty() {
            "the coverage record of the run lists no functions, so no line is counted".to_owned()
        } else {
            note.to_owned()
        });
    }
    let Some((file, line)) = split_site(remover) else {
        return Ok(Counted::Absent);
    };
    let Some(entries) = doc.get("lines").and_then(Value::as_array) else {
        return Ok(Counted::Absent);
    };
    let hit = entries.iter().find(|e| {
        e.get("line").and_then(Value::as_u64) == Some(line)
            && e.get("file")
                .and_then(Value::as_str)
                .is_some_and(|f| same_file(f, file))
    });
    Ok(match hit {
        None => Counted::Absent,
        Some(e) => match e.get("count").and_then(Value::as_u64) {
            Some(n) => Counted::Count(n),
            None => Counted::Unknown(
                e.get("why")
                    .and_then(Value::as_str)
                    .unwrap_or("the coverage run could not count the line")
                    .to_owned(),
            ),
        },
    })
}

/// How often the remover line ran in the run of `manifest` the alloc-test measured (`series`).
///
/// | Runtime | Source |
/// |---|---|
/// | Rust, Go, Node | the `lines` of `harness/<H>/executed.json` (coverage regions, blocks, ranges) |
/// | Python | `harness/<H>/lines-<S>.json`, the agent's `line_counts` per point |
/// | JVM, native | not counted: [`RemoverRun::NotShown`] |
pub fn remover_run(
    run_dir: &Path,
    manifest: &HarnessManifest,
    series: Option<&str>,
    remover: &str,
) -> RemoverRun {
    let by = |c: Counted| match c {
        Counted::Count(0) => RemoverRun::NotRun,
        Counted::Count(n) => RemoverRun::Ran(n),
        Counted::Unknown(why) => RemoverRun::NotShown(why),
        Counted::Absent => RemoverRun::NotAsked,
    };
    match manifest.runtime.as_str() {
        "rust" | "go" | "node" => match counted_in_coverage(run_dir, &manifest.id, remover) {
            Ok(c) => by(c),
            Err(why) => RemoverRun::NotShown(why),
        },
        "python" => {
            let runs = series
                .and_then(|s| LineRuns::load(run_dir, &manifest.id, s))
                .or_else(|| newest_line_runs(run_dir, &manifest.id));
            let Some(runs) = runs else {
                return RemoverRun::NotShown(
                    "the Python agent wrote no line counts for the run".to_owned(),
                );
            };
            let Some((file, line)) = split_site(remover) else {
                return RemoverRun::NotAsked;
            };
            let counts = runs
                .sites
                .iter()
                .find(|(k, _)| split_site(k).is_some_and(|(f, l)| l == line && same_file(f, file)));
            match counts {
                None => RemoverRun::NotAsked,
                Some((_, per_point)) => {
                    let best = per_point.iter().flatten().fold(0.0_f64, |a, b| a.max(*b));
                    if per_point.iter().all(Option::is_none) {
                        RemoverRun::NotShown("no point of the run counted the line".to_owned())
                    } else if best > 0.0 {
                        RemoverRun::Ran(best as u64)
                    } else {
                        RemoverRun::NotRun
                    }
                }
            }
        }
        other => RemoverRun::NotShown(format!(
            "the {other} harness records which classes loaded, not how often a line ran"
        )),
    }
}

fn newest_line_runs(run_dir: &Path, harness: &str) -> Option<LineRuns> {
    let dir = HarnessManifest::dir_in(run_dir, harness);
    let mut found: Vec<(std::time::SystemTime, LineRuns)> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.starts_with("lines-") && n.ends_with(".json")
        })
        .filter_map(|e| {
            let at = e.metadata().ok()?.modified().ok()?;
            let runs =
                serde_json::from_str::<LineRuns>(&std::fs::read_to_string(e.path()).ok()?).ok()?;
            Some((at, runs))
        })
        .collect();
    found.sort_by_key(|(at, _)| *at);
    found.pop().map(|(_, r)| r)
}

/// An alloc-test as the decision rules read it.
#[derive(Debug, Clone)]
pub struct Driver {
    /// The harness it ran on.
    pub harness: String,
    /// Its discriminator evidence.
    pub evidence: EvidenceId,
    /// What it favored.
    pub favors: Favors,
    /// The series it measured.
    pub series: Option<String>,
}

/// True when `m` is a hook driver made for the family of `root`: the finding is its target or its
/// entry path (a test loop, the project's own workload and a harness about the finding only
/// through a column, a probe or cited evidence are not).
pub fn made_for(view: &RunView, m: &HarnessManifest, root: &FindingId) -> bool {
    m.is_hook_driver()
        && matches!(
            harness_about(view, m, root),
            Some(HarnessAbout::Target | HarnessAbout::EntryPath)
        )
}

fn harness_of(e: &EvidenceRecord) -> Option<&str> {
    e.result.get("harness").and_then(Value::as_str)
}

fn family_ids(ledger: &Ledger, root: &FindingId) -> Vec<FindingId> {
    ledger
        .findings()
        .filter(|f| &f.id == root || f.parent.as_ref() == Some(root))
        .map(|f| f.id.clone())
        .collect()
}

/// The newest alloc-test of every harness made for the family of `root`, oldest first.
pub fn drivers_of(view: &RunView, root: &FindingId) -> Vec<Driver> {
    let Some(dir) = view.run_dir.as_deref() else {
        return Vec::new();
    };
    let family = family_ids(&view.ledger, root);
    let records: Vec<&EvidenceRecord> = view.ledger.evidence().collect();
    let mut newest: Vec<Driver> = Vec::new();
    let mut made: std::collections::BTreeMap<String, bool> = Default::default();
    for (e, favors) in bound_alloc_tests(&records, &family) {
        let Some(h) = harness_of(e) else { continue };
        let ok = *made.entry(h.to_owned()).or_insert_with(|| {
            HarnessManifest::load(dir, h)
                .ok()
                .is_some_and(|m| made_for(view, &m, root))
        });
        if !ok {
            continue;
        }
        newest.retain(|d| d.harness != h);
        newest.push(Driver {
            harness: h.to_owned(),
            evidence: e.evidence_id.clone(),
            favors,
            series: e
                .result
                .get("series")
                .and_then(Value::as_str)
                .map(str::to_owned),
        });
    }
    newest
}

fn seq_of_evidence(ledger: &Ledger, id: &EvidenceId) -> Option<u64> {
    ledger.lines().iter().find_map(|l| match &l.record {
        Record::Evidence(e) if &e.evidence_id == id => Some(l.seq),
        _ => None,
    })
}

fn seq_of_finding(ledger: &Ledger, id: &FindingId) -> Option<u64> {
    ledger.lines().iter().find_map(|l| match &l.record {
        Record::FindingAdded { finding } if &finding.id == id => Some(l.seq),
        _ => None,
    })
}

/// Two drivers made for a finding whose newest alloc-tests favor leak and non-leak.
#[derive(Debug, Clone)]
pub struct Disagreement {
    /// The root of the family.
    pub root: FindingId,
    /// The newest driver that favors leak.
    pub leak: Driver,
    /// The newest driver that favors non-leak.
    pub flat: Driver,
}

/// The disagreement that holds the confirm of a finding of the family rooted at `root`, if there
/// is one.
///
/// Among the newest alloc-test of each harness made for the family, one favors leak and one
/// favors non-leak, and for no such pair has a non-leak sibling of the root added after the older
/// of the two been refuted by an alloc-test newer than both, on a harness made for the finding
/// that is not the leak-favoring one of the pair. A rerun of the flat harness that now favors leak settles it by itself
/// (the newest of that harness is then a leak), and so does a third driver together with the
/// sibling that names the difference.
pub fn drivers_disagree(view: &RunView, root: &FindingId) -> Option<Disagreement> {
    let drivers = drivers_of(view, root);
    let leaks: Vec<&Driver> = drivers
        .iter()
        .filter(|d| d.favors == Favors::Leak)
        .collect();
    let flats: Vec<&Driver> = drivers
        .iter()
        .filter(|d| d.favors == Favors::NonLeak)
        .collect();
    let (leak, flat) = (leaks.last()?, flats.last()?);
    let ledger = &view.ledger;
    let all = drivers_all(view, root);
    // A pair is settled by a refuted non-leak sibling added after the older of the two, whose
    // refuting alloc-test is newer than both and ran on a driver made for the finding other than
    // the leak-favoring one of the pair.
    let settled = |l: &Driver, n: &Driver| -> Option<bool> {
        let (ls, ns) = (
            seq_of_evidence(ledger, &l.evidence)?,
            seq_of_evidence(ledger, &n.evidence)?,
        );
        let (older, newer) = (ls.min(ns), ls.max(ns));
        Some(
            ledger
                .findings()
                .filter(|s| {
                    s.parent.as_ref() == Some(root)
                        && s.polarity() == Polarity::NonLeak
                        && s.state == FindingState::Refuted
                        && seq_of_finding(ledger, &s.id).is_some_and(|q| q > older)
                })
                .any(|s| {
                    s.contradicts.iter().any(|d| {
                        all.iter().any(|x| {
                            &x.evidence == d && x.favors == Favors::Leak && x.harness != l.harness
                        }) && seq_of_evidence(ledger, d).is_some_and(|q| q > newer)
                    })
                }),
        )
    };
    let any_settled = leaks
        .iter()
        .any(|l| flats.iter().any(|n| settled(l, n) == Some(true)));
    if any_settled {
        return None;
    }
    Some(Disagreement {
        root: root.clone(),
        leak: (*leak).clone(),
        flat: (*flat).clone(),
    })
}

/// Every alloc-test bound to the family on a harness made for it (not only the newest of each).
fn drivers_all(view: &RunView, root: &FindingId) -> Vec<Driver> {
    let Some(dir) = view.run_dir.as_deref() else {
        return Vec::new();
    };
    let family = family_ids(&view.ledger, root);
    let records: Vec<&EvidenceRecord> = view.ledger.evidence().collect();
    bound_alloc_tests(&records, &family)
        .into_iter()
        .filter_map(|(e, favors)| {
            let h = harness_of(e)?;
            let m = HarnessManifest::load(dir, h).ok()?;
            made_for(view, &m, root).then(|| Driver {
                harness: h.to_owned(),
                evidence: e.evidence_id.clone(),
                favors,
                series: e
                    .result
                    .get("series")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            })
        })
        .collect()
}

/// The route that decides a disagreement, or a conditional remover nobody tested, in order:
/// the `N-RELEASE` sibling (its remover prefilled from the explanation when there is one), its
/// test and an alloc-test of a driver that calls the release on every unit; otherwise the sibling
/// that names another difference. Every id is the real one of the run, except the line of the
/// release (`<file:line>`), the text of `--refute-if` and the evidence the alloc-test prints.
pub fn release_route(
    view: &RunView,
    root: &FindingId,
    remover: Option<&str>,
    driver: Option<&Driver>,
) -> String {
    let driver = driver.cloned().or_else(|| drivers_of(view, root).pop());
    // Without a driver made for the finding the newest alloc-test of its family names the
    // harness and the evidence; without one the harness of the newest workload record and the
    // evidence the root cites.
    let family = family_ids(&view.ledger, root);
    let records: Vec<&EvidenceRecord> = view.ledger.evidence().collect();
    let newest = bound_alloc_tests(&records, &family).pop().map(|(e, _)| e);
    let h = driver
        .as_ref()
        .map(|d| d.harness.clone())
        .or_else(|| newest.and_then(|e| harness_of(e).map(str::to_owned)))
        .or_else(|| {
            records.iter().rev().find_map(|e| {
                (e.kind == EvidenceKind::Workload)
                    .then(|| crate::harness::harness_id_of(&e.result).map(str::to_owned))
                    .flatten()
            })
        })
        .unwrap_or_else(|| "<H>".to_owned());
    let ev = driver
        .as_ref()
        .map(|d| d.evidence.to_string())
        .or_else(|| newest.map(|e| e.evidence_id.to_string()))
        .or_else(|| {
            view.ledger
                .finding(root)
                .and_then(|f| f.supports.first())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "<id>".to_owned());
    let workload = view
        .ledger
        .evidence()
        .filter(|e| {
            e.kind == EvidenceKind::Workload
                && crate::harness::harness_id_of(&e.result) == Some(h.as_str())
        })
        .last()
        .map_or_else(|| ev.clone(), |e| e.evidence_id.to_string());
    let sib = view.ledger.next_sibling_id(root);
    let remover = remover.unwrap_or("<file:line>");
    format!(
        "if the flat driver calls a release the other skips: dla ledger add --sibling-of {root} --origin hypothesis --hypothesis N-RELEASE --remover {remover} --refute-if \"<result>\" --evidence {ev}   then   dla ledger test {sib} --evidence {workload}   then   dla alloc-test {h} --finding {root}   (a driver that calls the release on every unit; for a thread-local or per-thread holder also from the pooled or worker threads the project's callers use), then dla ledger confirm {sib} --evidence <the id it prints> if the series is flat, or dla ledger refute {sib} --evidence <the id it prints> if it grows   or, when the difference is another: dla ledger add --sibling-of {root} --origin hypothesis --hypothesis N-WARM --refute-if \"<result>\" --evidence {ev}   (the N-* id that names it), a new alloc-test on a driver made for {root}, then dla ledger refute {sib} --evidence <the id it prints>"
    )
}

/// The refusal of a confirm held by [`Disagreement`].
pub fn drivers_disagree_refusal(view: &RunView, id: &FindingId, d: &Disagreement) -> Refusal {
    Refusal::new(
        codes::DRIVERS_DISAGREE,
        format!(
            "{id} cannot be confirmed: two drivers made for it disagree. {} ({}) favors non-leak and {} ({}) favors leak. What does {} do that {} does not?",
            d.flat.harness,
            d.flat.evidence,
            d.leak.harness,
            d.leak.evidence,
            d.flat.harness,
            d.leak.harness
        ),
        format!(
            "the difference between {} and {} named in a sibling and tested by a new alloc-test",
            d.flat.harness, d.leak.harness
        ),
        release_route(view, &d.root, None, Some(&d.flat)),
    )
}

/// What `release-untested` names: a finding whose explanation says its removal is conditional.
#[derive(Debug, Clone)]
pub struct Untested {
    /// The finding.
    pub finding: FindingId,
    /// The remover its explanation names.
    pub remover: String,
}

/// The explanation of `f` when it says `removal: conditional` with a remover and no `N-RELEASE`
/// sibling of its root has been decided by a run with the release called (refuted, or confirmed
/// which counts the finding apart).
pub fn release_untested(ledger: &Ledger, f: &Finding) -> Option<Untested> {
    if f.polarity() != Polarity::Leak {
        return None;
    }
    let x = ledger.explanation(&f.id)?;
    if x.removal != RemovalClaim::Conditional {
        return None;
    }
    let remover = x.remover.as_deref()?.to_owned();
    let root = f.root_id();
    if release_refuted(ledger, &root) || release_confirmed(ledger, &root).is_some() {
        return None;
    }
    Some(Untested {
        finding: f.id.clone(),
        remover,
    })
}

/// The refusal of a confirm of a finding explained as conditional whose release was not tested.
pub fn release_untested_refusal(view: &RunView, u: &Untested) -> Refusal {
    let root = u.finding.parent().unwrap_or_else(|| u.finding.clone());
    Refusal::new(
        codes::RELEASE_UNTESTED,
        format!(
            "{} cannot be confirmed as a leak: its explanation says the holder is released by {} only under a condition, and no run called that release (an N-RELEASE sibling of {root} refuted). A caller that skips a release the API requires is misusing it, which is no leak of the library",
            u.finding, u.remover
        ),
        format!(
            "an N-RELEASE sibling of {root} refuted by an alloc-test with {} called",
            u.remover
        ),
        release_route(view, &root, Some(&u.remover), None),
    )
}

/// How the evidence cited for an `N-RELEASE` sibling was judged.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Decision {
    /// The note to print when the runtime could not count the remover line.
    pub note: Option<String>,
}

/// Judges the evidence cited to confirm (`want` favors non-leak) or refute (`want` favors leak)
/// the `N-RELEASE` sibling `s`: it must be an alloc-test (`plateau+purge`) bound to the family
/// that favors `want`, on a harness made for the finding, with the remover line of the sibling
/// run ([`remover_run`]); where the runtime cannot count the line the decision carries
/// [`NOT_SHOWN`]. Any other evidence is refused, with the fix naming the alloc-test that would
/// do when the ledger holds one.
pub fn check_release_decision(
    view: &RunView,
    s: &Finding,
    cited: &[&EvidenceRecord],
    want: Favors,
) -> Result<Decision, Refusal> {
    let root = s.root_id();
    let verb = if want == Favors::NonLeak {
        "confirm"
    } else {
        "refute"
    };
    let remover = s.remover.clone().unwrap_or_default();
    let family = family_ids(&view.ledger, &root);
    let run_dir = view.run_dir.as_deref();
    let all: Vec<&EvidenceRecord> = view.ledger.evidence().collect();
    let alloc: Vec<(&EvidenceRecord, Favors)> = bound_alloc_tests(&all, &family);

    // How one alloc-test fares: `Ok(note)` when it decides, `Err(why)` when it does not.
    let judge = |e: &EvidenceRecord, favors: Favors| -> Result<Option<String>, String> {
        let id = &e.evidence_id;
        if favors != want {
            return Err(format!("{id} favors {favors}, not {want}"));
        }
        let (Some(h), Some(dir)) = (harness_of(e), run_dir) else {
            return Err(format!("{id} names no harness of this run"));
        };
        let Ok(m) = HarnessManifest::load(dir, h) else {
            return Err(format!(
                "{id} ran on {h}, which this run has no manifest for"
            ));
        };
        if !made_for(view, &m, &root) {
            return Err(format!(
                "{id} ran on {h}, which is not a driver made for {root} (a test loop, the project's own workload or a driver about it only through a column or a probe)"
            ));
        }
        let series = e.result.get("series").and_then(Value::as_str);
        match remover_run(dir, &m, series, &remover) {
            RemoverRun::Ran(_) => Ok(None),
            RemoverRun::NotShown(_) => Ok(Some(NOT_SHOWN.to_owned())),
            RemoverRun::NotRun => Err(format!(
                "{id} ran on {h}, where the remover line {remover} ran 0 times: a run that never reached the release says nothing about what the release does"
            )),
            RemoverRun::NotAsked => Err(format!(
                "{id} ran on {h}, whose coverage run did not count {remover} (it ran before the sibling named it)"
            )),
        }
    };

    let mut why: Vec<String> = Vec::new();
    for e in cited {
        if e.kind != EvidenceKind::Discriminator {
            why.push(format!(
                "{} is a {} record, not an alloc-test",
                e.evidence_id,
                e.kind.as_str()
            ));
            continue;
        }
        match alloc.iter().find(|(a, _)| a.evidence_id == e.evidence_id) {
            None => why.push(format!(
                "{} is not an alloc-test (plateau+purge) bound to the family of {root}",
                e.evidence_id
            )),
            Some((a, favors)) => match judge(a, *favors) {
                Ok(note) => return Ok(Decision { note }),
                Err(w) => why.push(w),
            },
        }
    }
    if cited.is_empty() {
        why.push("it cites no evidence".to_owned());
    }
    // The newest alloc-test of the ledger that would decide it, for the fix.
    let usable = alloc
        .iter()
        .rev()
        .find(|(e, f)| judge(e, *f).is_ok())
        .map(|(e, _)| e.evidence_id.clone());
    let id = &s.id;
    let (code, fix) = match &usable {
        Some(u) => (
            codes::RELEASE_NOT_RUN,
            format!("dla ledger {verb} {id} --evidence {u}"),
        ),
        None => (
            codes::RELEASE_NOT_RUN,
            format!(
                "dla alloc-test <H> --finding {root} on a driver made for {root} that calls {remover} on every unit (dla harness run <H> first when the line has to be counted), then dla ledger {verb} {id} --evidence <that alloc-test>"
            ),
        ),
    };
    Err(Refusal::new(
        code,
        format!(
            "{id} (N-RELEASE: the release is {remover}) cannot be {}d on this evidence: {}",
            verb,
            why.join("; ")
        ),
        format!(
            "an alloc-test that favors {want} on a driver made for {root} in which the remover line {remover} ran"
        ),
        fix,
    ))
}

/// The note a decided `N-RELEASE` sibling carries: [`NOT_SHOWN`] when the alloc-test it was
/// decided on ran on a runtime that cannot count the remover line, else `None`.
pub fn decision_note(view: &RunView, s: &Finding) -> Option<String> {
    let dir = view.run_dir.as_deref()?;
    let remover = s.remover.as_deref()?;
    let root = s.root_id();
    let family = family_ids(&view.ledger, &root);
    let all: Vec<&EvidenceRecord> = view.ledger.evidence().collect();
    let alloc = bound_alloc_tests(&all, &family);
    s.supports
        .iter()
        .chain(&s.contradicts)
        .filter_map(|id| alloc.iter().find(|(e, _)| &e.evidence_id == id))
        .find_map(|(e, _)| {
            let m = HarnessManifest::load(dir, harness_of(e)?).ok()?;
            let series = e.result.get("series").and_then(Value::as_str);
            matches!(
                remover_run(dir, &m, series, remover),
                RemoverRun::NotShown(_)
            )
            .then(|| NOT_SHOWN.to_owned())
        })
}

/// The confirmed leaks (`confirmed` roots) whose family holds a confirmed `N-RELEASE` sibling:
/// the growth is flat with the release called, so they are counted apart from the leaks.
pub fn grows_only_while_skipped(ledger: &Ledger) -> Vec<&Finding> {
    ledger
        .findings()
        .filter(|f| {
            f.state == FindingState::Confirmed
                && f.polarity() == Polarity::Leak
                && f.parent.is_none()
                && ledger.merged_into(&f.id).is_none()
                && release_confirmed(ledger, &f.id).is_some()
        })
        .collect()
}

#[cfg(test)]
mod tests;
