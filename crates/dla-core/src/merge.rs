//! `dla ledger merge`: worker slices into the main ledger, then the cross-worker gate.
//!
//! For every registered worker, and every `ledger.<worker>.jsonl` present, the slice is
//! replayed against the main ledger as it was when the slice began, and the lines not
//! merged yet are appended to `ledger.jsonl` verbatim (new `seq`, original `ts`, the
//! line's `worker` set), followed by one `slice_merged` record. A re-run appends only
//! the lines after the last `slice_merged.lines`, so merging is idempotent and can be
//! repeated while workers still write.
//!
//! The gate then looks across workers:
//!
//! 1. leak-side findings of different workers with equal (site, holder) become one
//!    (`finding_merged`: the more advanced state is canonical, ties go to the lower
//!    worker name);
//! 2. equal (site, holder) confirmed by one worker and refuted by another opens an open
//!    item;
//! 3. a worker with no `slice_done`, or registered with no slice, opens
//!    "worker <w> did not finish (slice ends at line N)", is marked `killed` (or
//!    `over-budget` when its deadline passed) and makes the run `incomplete`;
//! 4. a runtime the probe found that no worker covers becomes a blind spot in
//!    `coverage.json`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde::Serialize;

use crate::{
    coverage::CoverageManifest,
    error::{Error, Refusal},
    finding::{Finding, FindingState, Polarity, UNCLASSIFIED, UnresolvedReason},
    ids::FindingId,
    ledger::Ledger,
    probe::ProbeReport,
    record::{Resolution, SliceMerged},
    slice::{self, codes as m4},
    state::{RunState, RunStatus, WorkerStatus, now_utc},
};

/// The `opened_by` of open items the merge raises.
pub const MERGE_BY: &str = "dla ledger merge";
/// The text `incomplete_reason` starts with when the merge made the run incomplete.
pub const UNFINISHED_REASON: &str = "workers did not finish";

/// Two findings of different workers recognised as the same.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Duplicate {
    /// The finding that stays.
    pub canonical: String,
    /// The finding folded into it.
    pub duplicate: String,
}

/// A worker that did not end its slice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unfinished {
    /// The worker.
    pub worker: String,
    /// Lines its slice holds (0 when it never wrote).
    pub slice_lines: u64,
    /// `killed` or `over-budget`.
    pub status: String,
}

/// What a merge did.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MergeReport {
    /// Slice lines appended by this run, per worker (workers with nothing new are absent).
    pub merged_lines: BTreeMap<String, u64>,
    /// Slice lines merged in total, per worker with a slice.
    pub total_lines: BTreeMap<String, u64>,
    /// Workers whose slice ended in a torn line that was dropped.
    pub torn_tail: Vec<String>,
    /// Stalled workers that ended their slice with findings or items still open (an item
    /// was opened for each).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stalled: Vec<String>,
    /// Findings folded together by this run.
    pub duplicates: Vec<Duplicate>,
    /// Open items this run raised for disagreeing workers.
    pub contradictions: Vec<String>,
    /// Workers without `slice_done`.
    pub unfinished: Vec<Unfinished>,
    /// Probed runtimes no worker covers.
    pub uncovered_runtimes: Vec<String>,
    /// Workers whose slice cannot be replayed: it was left out of this merge.
    pub invalid_slices: Vec<String>,
    /// The run status after the merge.
    pub status: RunStatus,
}

fn invalid_slice(worker: &str, e: &Error) -> Error {
    let r = e.to_refusal();
    Refusal::new(
        m4::SLICE_INVALID,
        format!("slice of {worker} cannot be merged: {}", r.message),
        format!("a valid ledger.{worker}.jsonl"),
        format!(
            "correct or delete the bad line of ledger.{worker}.jsonl (a torn last line is dropped by itself), then dla ledger merge"
        ),
    )
    .into()
}

/// How far a finding got, for choosing the canonical one of a duplicate pair.
fn advance(s: FindingState) -> u8 {
    match s {
        FindingState::Strong => 6,
        FindingState::Confirmed => 5,
        FindingState::UnderTest => 4,
        FindingState::SuspectedStaticOnly => 3,
        FindingState::Unresolved => 2,
        FindingState::Candidate => 1,
        FindingState::Refuted => 0,
    }
}

fn is_confirmed(s: FindingState) -> bool {
    matches!(s, FindingState::Confirmed | FindingState::Strong)
}

/// Leak-side top-level findings of workers that name both a site and a holder, grouped
/// by that pair. (A finding without either never merges: nothing says what it is the
/// same as.)
fn groups(ledger: &Ledger) -> BTreeMap<(String, String), Vec<Finding>> {
    let mut out: BTreeMap<(String, String), Vec<Finding>> = BTreeMap::new();
    for f in ledger.findings() {
        if f.id.prefix().is_none() || f.id.is_sibling() || f.polarity() != Polarity::Leak {
            continue;
        }
        if let (Some(site), Some(holder)) = (&f.site, f.holder) {
            out.entry((site.clone(), holder.to_string()))
                .or_default()
                .push(f.clone());
        }
    }
    out
}

/// Whether an open item, resolved or not, starts with `starts_with`.
fn has_open_item(ledger: &Ledger, starts_with: &str) -> bool {
    ledger
        .open_items()
        .any(|o| o.item.text.starts_with(starts_with))
}

/// Merges every slice of the run in `run_dir` into its main ledger and applies the
/// cross-worker gate. See the module docs.
pub fn merge(run_dir: &Path) -> Result<MergeReport, Error> {
    // A slice or the ledger left behind by a write killed before its seal is dla's own
    // doing, not an edit: seal it as it is before the merge reads it.
    crate::integrity::settle(run_dir)?;
    let state = RunState::load(run_dir)?;
    let mut main = Ledger::load(run_dir)?;

    let mut names: BTreeSet<String> = state.workers.iter().map(|w| w.name.clone()).collect();
    names.extend(slice::slice_workers(run_dir)?);

    let mut report = MergeReport {
        merged_lines: BTreeMap::new(),
        total_lines: BTreeMap::new(),
        torn_tail: Vec::new(),
        stalled: Vec::new(),
        duplicates: Vec::new(),
        contradictions: Vec::new(),
        unfinished: Vec::new(),
        uncovered_runtimes: Vec::new(),
        invalid_slices: Vec::new(),
        status: state.status,
    };
    // worker -> (lines in slice, finished)
    let mut slice_facts: BTreeMap<String, (u64, bool)> = BTreeMap::new();

    // ---- 1. slices into the main ledger
    for w in &names {
        let sp = slice::slice_path(run_dir, w);
        let text = match slice::read_text(&sp) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(Error::io(&sp, e)),
        };
        let cut = slice::split(&text);
        let replayed = match Ledger::load_slice_from(run_dir, w, &text) {
            Ok(r) => r,
            Err(e) => {
                // One bad slice must not stop the others: note it and carry on. Lines
                // merged from it earlier stay; the worker counts as unfinished.
                let why = invalid_slice(w, &e).to_refusal().message;
                let prefix = format!("slice of worker {w} is invalid");
                if !has_open_item(&main, &prefix) {
                    main.open_item_add(
                        &format!(
                            "{prefix}: {why} (correct or delete the bad line of ledger.{w}.jsonl, then dla ledger merge)"
                        ),
                        None,
                        MERGE_BY,
                    )?;
                }
                report.invalid_slices.push(w.clone());
                let merged = main.merged_lines(w);
                slice_facts.insert(w.clone(), (merged, false));
                report.total_lines.insert(w.clone(), merged);
                continue;
            }
        };
        let n = replayed.lines().len();
        let done = replayed.slice_info(w).is_some_and(|r| r.done.is_some());
        if cut.torn.is_some() {
            report.torn_tail.push(w.clone());
        }
        slice_facts.insert(w.clone(), (n as u64, done));
        let before = usize::try_from(main.merged_lines(w)).unwrap_or(usize::MAX);
        let torn_unseen = cut.torn.is_some() && !main.merged_torn_tail(w);
        if n > before || torn_unseen {
            let merged = SliceMerged {
                worker: w.clone(),
                lines: n as u64,
                sha256: cut.sha256_of_first(n),
                torn_tail: cut.torn.is_some(),
                done,
            };
            main.append_merged(w, &replayed.lines()[before..], merged)?;
            report.merged_lines.insert(w.clone(), (n - before) as u64);
        }
        report.total_lines.insert(w.clone(), n as u64);
    }

    // ---- 2. contradictions, then duplicates
    for ((site, holder), members) in groups(&main) {
        let confirmed: Vec<&Finding> = members.iter().filter(|f| is_confirmed(f.state)).collect();
        let refuted: Vec<&Finding> = members
            .iter()
            .filter(|f| f.state == FindingState::Refuted)
            .collect();
        for c in &confirmed {
            for r in &refuted {
                if c.id.prefix() == r.id.prefix() {
                    continue;
                }
                let text = format!(
                    "workers disagree on {site} / {holder}: {} confirmed it, {} refuted it",
                    c.id, r.id
                );
                if main.open_items().any(|o| o.item.text == text) {
                    continue;
                }
                main.open_item_add(&text, Some(c.id.clone()), MERGE_BY)?;
                report.contradictions.push(text);
            }
        }
        // Refuted findings are verdicts, not duplicates. Findings merged earlier stay in
        // the pool: a duplicate that has since moved further than its canonical takes
        // over (a swap), and one that gained supports is folded in again.
        let mut live: Vec<&Finding> = members
            .iter()
            .filter(|f| f.state != FindingState::Refuted)
            .collect();
        if live.len() < 2 {
            continue;
        }
        live.sort_by(|a, b| {
            advance(b.state)
                .cmp(&advance(a.state))
                .then_with(|| a.id.prefix().cmp(&b.id.prefix()))
                .then_with(|| a.id.cmp(&b.id))
        });
        let canonical = live[0];
        for dup in live.iter().skip(1) {
            if dup.id.prefix() == canonical.id.prefix() {
                continue;
            }
            match main.merged_into(&dup.id) {
                // Already folded into this one: only the supports may be out of date.
                Some(c) if *c == canonical.id => {
                    if !dup.supports.iter().all(|e| canonical.supports.contains(e)) {
                        main.merge_findings(&canonical.id, &dup.id)?;
                    }
                }
                // Folded into another finding of the group: that one is handled itself.
                Some(_) => {}
                None => {
                    // A canonical that is itself a duplicate of `dup` swaps with it; one
                    // that is a duplicate of something else waits for that finding.
                    if main
                        .merged_into(&canonical.id)
                        .is_some_and(|c| *c != dup.id)
                    {
                        continue;
                    }
                    merge_pair(&mut main, &mut report, &canonical.id, &dup.id)?;
                }
            }
        }
    }

    // ---- 2b. a label on a duplicate is not copied to its canonical finding
    let pairs: Vec<(FindingId, FindingId)> = main
        .findings()
        .filter_map(|f| main.merged_into(&f.id).map(|c| (f.id.clone(), c.clone())))
        .collect();
    for (dup, canonical) in pairs {
        let label = |id: &FindingId| main.finding(id).map(|f| f.archetype.clone());
        let (Some(dl), Some(cl)) = (label(&dup), label(&canonical)) else {
            continue;
        };
        if dl == UNCLASSIFIED || cl != UNCLASSIFIED {
            continue;
        }
        let text = format!(
            "label {dl} on duplicate {dup} not carried to {canonical}: dla ledger label {canonical} --as {dl}"
        );
        if main.open_items().any(|o| o.item.text == text) {
            continue;
        }
        main.open_item_add(&text, Some(canonical.clone()), MERGE_BY)?;
    }

    // ---- 3. unfinished workers
    let now = now_utc();
    let mut status_of: BTreeMap<String, WorkerStatus> = BTreeMap::new();
    for w in &names {
        let (lines, done) = slice_facts.get(w).copied().unwrap_or((0, false));
        if done {
            status_of.insert(w.clone(), WorkerStatus::Done);
            // It finished after an earlier merge called it unfinished: close that item.
            let open: Vec<_> = main
                .open_items()
                .filter(|o| {
                    o.resolution.is_none()
                        && o.item
                            .text
                            .starts_with(&format!("worker {w} did not finish"))
                })
                .map(|o| o.item.id.clone())
                .collect();
            for id in open {
                main.resolve_open_item(
                    &id,
                    Resolution::Unresolved {
                        reason: UnresolvedReason::TimeBudget,
                        remedy: format!("worker {w} finished after the merge that raised this"),
                    },
                )?;
            }
            // The stop gate let the worker go after three attempts with its work still
            // open, and it then ended its slice: say so, once.
            let stalled = state.worker(w).is_some_and(|e| e.stalled);
            let mine = |id: &str| id.split_once('/').is_some_and(|(p, _)| p == w);
            let left = main
                .findings()
                .filter(|f| {
                    mine(&f.id.to_string())
                        && main.merged_into(&f.id).is_none()
                        && !f.state.is_dispositioned()
                })
                .count()
                + main
                    .open_items()
                    .filter(|o| o.resolution.is_none() && mine(&o.item.id.to_string()))
                    .count();
            if stalled && left > 0 && !has_open_item(&main, &format!("worker {w} stalled")) {
                main.open_item_add(
                    &format!(
                        "worker {w} stalled: the stop gate let it go after repeated stop attempts with no change, and {left} of its findings or items are still open"
                    ),
                    None,
                    MERGE_BY,
                )?;
                report.stalled.push(w.clone());
            }
            continue;
        }
        let over = state.worker(w).is_some_and(|e| now > e.deadline);
        let status = if over {
            WorkerStatus::OverBudget
        } else {
            WorkerStatus::Killed
        };
        status_of.insert(w.clone(), status);
        report.unfinished.push(Unfinished {
            worker: w.clone(),
            slice_lines: lines,
            status: status.to_string(),
        });
        // Resolved items count: a core that closed it has accepted the worker's end.
        if !has_open_item(&main, &format!("worker {w} did not finish")) {
            let stalled = if state.worker(w).is_some_and(|e| e.stalled) {
                ", stalled: the stop gate let it go"
            } else {
                ""
            };
            main.open_item_add(
                &format!("worker {w} did not finish (slice ends at line {lines}{stalled})"),
                None,
                MERGE_BY,
            )?;
        }
    }

    // ---- 4. runtimes with no worker
    let covered: BTreeSet<String> = names
        .iter()
        .map(|w| {
            main.slice_info(w)
                .and_then(|r| r.started.as_ref())
                .map(|s| s.runtime.clone())
                .or_else(|| state.worker(w).map(|e| e.runtime.clone()))
                .unwrap_or_else(|| w.clone())
        })
        .collect();
    if !names.is_empty()
        && let Some(probe) = read_probe(run_dir)
    {
        let missing: Vec<String> = probe
            .runtimes
            .iter()
            .map(|r| r.runtime.to_string())
            .filter(|r| !covered.contains(r))
            .collect();
        if !missing.is_empty() {
            CoverageManifest::update(run_dir, |m| {
                for rt in &missing {
                    m.add_blind_spot(
                        rt,
                        "no worker covered this runtime: its code was not investigated (dla run dispatch --workers <runtimes> starts one per runtime)",
                    );
                }
            })?;
        }
        report.uncovered_runtimes = missing;
    }

    // ---- state: worker statuses and the run status
    let unfinished: Vec<&Unfinished> = report.unfinished.iter().collect();
    let reason = format!(
        "{UNFINISHED_REASON}: {}",
        unfinished
            .iter()
            .map(|u| format!("{} (slice ends at line {})", u.worker, u.slice_lines))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let new_status = RunState::update(run_dir, |s| {
        for e in &mut s.workers {
            if let Some(st) = status_of.get(&e.name) {
                e.status = *st;
            }
        }
        if !unfinished.is_empty() {
            if matches!(
                s.status,
                RunStatus::Active | RunStatus::Complete | RunStatus::Incomplete
            ) {
                s.status = RunStatus::Incomplete;
                s.incomplete_reason = Some(reason.clone());
            }
        } else if s.status == RunStatus::Incomplete
            && s.incomplete_reason
                .as_deref()
                .is_some_and(|r| r.starts_with(UNFINISHED_REASON))
        {
            s.status = RunStatus::Active;
            s.incomplete_reason = None;
        }
        Ok(s.status)
    })?;
    report.status = new_status;
    Ok(report)
}

fn merge_pair(
    main: &mut Ledger,
    report: &mut MergeReport,
    canonical: &FindingId,
    duplicate: &FindingId,
) -> Result<(), Error> {
    main.merge_findings(canonical, duplicate)?;
    report.duplicates.push(Duplicate {
        canonical: canonical.to_string(),
        duplicate: duplicate.to_string(),
    });
    Ok(())
}

fn read_probe(run_dir: &Path) -> Option<ProbeReport> {
    let text = fs::read_to_string(run_dir.join("probe").join("probe.json")).ok()?;
    serde_json::from_str(&text).ok()
}
