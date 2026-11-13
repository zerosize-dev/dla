//! What a run's time budget is sized from and what finishing it would still take: the
//! numbers behind the `auto` budget, `dla run status` and the report's `Time:` line.

use std::{collections::BTreeSet, path::Path};

use crate::{
    Ledger,
    evidence::{Authority, EvidenceKind},
    finding::{Finding, UnresolvedReason},
    gate::finding_runtime,
    slice,
    state::BudgetEstimate,
    view::RunView,
};

/// Minutes one finding needs to be measured, for [`Remaining`].
pub const FINISH_MINUTES_PER_FINDING: u64 = 3;
/// Minutes a runtime without a successful harness run needs for build and harness, for
/// [`Remaining`].
pub const FINISH_MINUTES_PER_RUNTIME: u64 = 15;

/// True for a finding that counts toward the budget: a top-level finding (a sibling
/// hypothesis is tested by its parent's runs) that is not a duplicate merged into another.
fn counts(view: &RunView, f: &Finding) -> bool {
    !f.id.is_sibling() && view.ledger.merged_into(&f.id).is_none()
}

fn runtime_set<'a>(
    view: &RunView,
    findings: impl Iterator<Item = &'a Finding>,
) -> BTreeSet<String> {
    findings
        .filter_map(|f| finding_runtime(view, f))
        .collect::<BTreeSet<_>>()
}

/// A finding that counts toward the budget: its site (findings at one site count once) and
/// its runtime.
struct Counted {
    id: String,
    site: Option<String>,
    runtime: Option<String>,
}

/// The findings of `view` that count toward the budget and satisfy `keep`. Every state
/// counts: a finding already dispositioned was still a candidate to measure.
fn counted(
    view: &RunView,
    keep: impl Fn(&Finding) -> bool,
    runtime_of: impl Fn(&Finding) -> Option<String>,
) -> Vec<Counted> {
    view.ledger
        .findings()
        .filter(|f| counts(view, f) && keep(f))
        .map(|f| Counted {
            id: f.id.to_string(),
            site: f.site.clone(),
            runtime: runtime_of(f),
        })
        .collect()
}

/// R and C of `all`, each distinct site and id once.
fn size(all: Vec<Counted>, min_runtimes: u64) -> BudgetEstimate {
    let mut seen_sites: BTreeSet<String> = BTreeSet::new();
    let mut seen_ids: BTreeSet<String> = BTreeSet::new();
    let mut runtimes: BTreeSet<String> = BTreeSet::new();
    let mut candidates = 0u64;
    for c in all {
        if !seen_ids.insert(c.id) || c.site.is_some_and(|s| !seen_sites.insert(s)) {
            continue;
        }
        candidates += 1;
        if let Some(r) = c.runtime {
            runtimes.insert(r);
        }
    }
    let mut r = runtimes.len() as u64;
    if r == 0 && candidates > 0 {
        r = 1;
    }
    BudgetEstimate::new(r.max(min_runtimes), candidates)
}

/// The size of an `auto` budget from the view a gate check sees: R is the number of
/// runtimes with at least one candidate, C the number of candidates; the budget is
/// `20 + 15*R + 3*C` minutes between 60 and 180 (see [`crate::state::auto`]). A candidate
/// is a top-level, non-duplicate finding of any state (sibling hypotheses are not counted),
/// and findings at one site count once (at the sweep gate the census and the sweep rules
/// each open a candidate at the same holder, and duplicates are merged only later). When
/// candidates exist but none names a runtime, R is 1.
///
/// In worker mode (`worker` is `Some`) R is at least the number of workers the run
/// dispatched. The view holds only the worker's own slice and the main ledger; see
/// [`estimate_across`] for the estimate over every slice.
pub fn estimate(view: &RunView, worker: Option<&str>) -> BudgetEstimate {
    let all = counted(view, |_| true, |f| finding_runtime(view, f));
    let min = if worker.is_some() {
        view.state.workers.len() as u64
    } else {
        0
    };
    size(all, min)
}

/// [`estimate`] over a dispatched run: the main ledger plus the slice of every registered
/// worker, each loaded read only (no lock, no write). A slice that cannot be read counts
/// as empty, so a damaged one never stops a gate. R is at least the number of workers.
pub fn estimate_across(run_dir: &Path, view: &RunView) -> BudgetEstimate {
    let state = &view.state;
    let mut all: Vec<Counted> = Vec::new();
    if let Ok(main) = Ledger::load(run_dir) {
        let mut main_view = RunView::new(state.clone(), main, view.coverage.clone());
        main_view.run_dir = Some(run_dir.to_path_buf());
        all.extend(counted(
            &main_view,
            |_| true,
            |f| finding_runtime(&main_view, f),
        ));
    }
    for w in &state.workers {
        let Ok(text) = slice::read_text(&slice::slice_path(run_dir, &w.name)) else {
            continue;
        };
        let Ok(ledger) = Ledger::load_slice_from(run_dir, &w.name, &text) else {
            continue;
        };
        let slice_view = RunView::new(state.clone(), ledger, None);
        let name = w.name.as_str();
        all.extend(counted(
            &slice_view,
            |f| f.id.prefix() == Some(name),
            |_| Some(w.runtime.clone()),
        ));
    }
    size(all, state.workers.len() as u64)
}

/// What measuring the rest of a run would take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remaining {
    /// Non-duplicate findings that are open or were closed with reason `time_budget`.
    pub findings: u64,
    /// Their runtimes that have no harness run that succeeded.
    pub runtimes_without_run: u64,
    /// `3 * findings + 15 * runtimes_without_run`, in minutes.
    pub minutes: u64,
    /// How many of the findings were closed with `time_budget`.
    pub closed_for_time: u64,
    /// The same cost counted over only those closed findings: what the report's `Time:`
    /// line quotes, so its count and its minutes describe the same findings.
    pub closed_minutes: u64,
}

/// Minutes still needed to measure what the budget left open: three per non-duplicate
/// top-level finding that is open or closed with `time_budget`, plus fifteen per runtime
/// of those findings that has no harness run that succeeded (a measured `workload` record
/// of a harness of that runtime with no failed process, or a coverage row of that runtime
/// with iterations or exercised entry points). `dla run status` prints it as
/// `to_finish_minutes`; the report's `Time:` line quotes it for the findings closed for
/// lack of time.
pub fn remaining(view: &RunView) -> Remaining {
    let wanted: Vec<&Finding> = view
        .ledger
        .findings()
        .filter(|f| counts(view, f))
        .filter(|f| !f.state.is_dispositioned() || closed_for_time(f))
        .collect();
    let closed: Vec<&Finding> = wanted
        .iter()
        .copied()
        .filter(|f| f.state.is_dispositioned())
        .collect();
    let ran = ran_runtimes(view);
    let (findings, missing, minutes) = cost(view, &wanted, &ran);
    let (_, _, closed_minutes) = cost(view, &closed, &ran);
    Remaining {
        findings,
        runtimes_without_run: missing,
        minutes,
        closed_for_time: closed.len() as u64,
        closed_minutes,
    }
}

/// True for a finding closed with reason `time_budget`, whichever closing state carries it
/// (`unresolved` or `suspected_static_only`).
fn closed_for_time(f: &Finding) -> bool {
    f.state.needs_reason() && f.reason == Some(UnresolvedReason::TimeBudget)
}

/// `(findings, runtimes without a harness run, minutes)` for `wanted`.
fn cost(view: &RunView, wanted: &[&Finding], ran: &BTreeSet<String>) -> (u64, u64, u64) {
    let mut runtimes = runtime_set(view, wanted.iter().copied());
    if runtimes.is_empty() && !wanted.is_empty() {
        runtimes.insert(String::new());
    }
    let missing = runtimes
        .iter()
        .filter(|r| r.is_empty() && ran.is_empty() || !r.is_empty() && !ran.contains(*r))
        .count() as u64;
    let findings = wanted.len() as u64;
    (
        findings,
        missing,
        FINISH_MINUTES_PER_FINDING * findings + FINISH_MINUTES_PER_RUNTIME * missing,
    )
}

/// Runtimes with a harness run that succeeded.
fn ran_runtimes(view: &RunView) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(c) = &view.coverage {
        for (rt, w) in &c.workers {
            if w.iterations.is_some_and(|n| n > 0) || !w.entry_points.exercised.is_empty() {
                out.insert(rt.clone());
            }
        }
    }
    let Some(run_dir) = view.run_dir.as_deref() else {
        return out;
    };
    for e in view.ledger.evidence() {
        if e.kind != EvidenceKind::Workload || e.authority != Authority::Test {
            continue;
        }
        let failed = e
            .result
            .get("failed_processes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if failed > 0 {
            continue;
        }
        if let Some(id) = e.result.get("harness").and_then(|v| v.as_str())
            && let Ok(m) = crate::harness::HarnessManifest::load(run_dir, id)
        {
            out.insert(m.runtime.as_str().to_owned());
        }
    }
    out
}

/// Whole minutes in `secs`, rounded up.
fn minutes_up(secs: u64) -> u64 {
    secs.div_ceil(60)
}

/// A duration as `3h`, `2h58m`, `1m30s`, `45m` or `30s`: every non-zero unit, none dropped.
pub fn human(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    let mut out = String::new();
    for (n, unit) in [(h, 'h'), (m, 'm'), (s, 's')] {
        if n > 0 {
            out.push_str(&format!("{n}{unit}"));
        }
    }
    if out.is_empty() {
        out.push_str("0s");
    }
    out
}

/// A duration to the minute, seconds dropped under a minute-long figure: `3h`, `2h58m`,
/// `45m`, `30s`. For time used, where the seconds are noise.
fn human_coarse(secs: u64) -> String {
    if secs < 60 {
        return format!("{secs}s");
    }
    human(secs / 60 * 60)
}

/// The line `dla gate check` prints when it settles an `auto` budget.
pub fn settled_line(e: &BudgetEstimate) -> String {
    let capped = if e.minutes > e.total_minutes {
        format!(", capped at {}", human(e.total_minutes * 60))
    } else if e.minutes < e.total_minutes {
        format!(", raised to {}", human(e.total_minutes * 60))
    } else {
        String::new()
    };
    format!(
        "budget (auto): {} for {} and {} {} ({} + {}x{} + {}x{} = {} min{capped})",
        human(e.total_minutes * 60),
        plural(e.runtimes, "runtime", "runtimes"),
        e.candidates,
        if e.candidates == 1 {
            "candidate"
        } else {
            "candidates"
        },
        crate::state::auto::BASE_MINUTES,
        crate::state::auto::PER_RUNTIME_MINUTES,
        e.runtimes,
        crate::state::auto::PER_CANDIDATE_MINUTES,
        e.candidates,
        e.minutes,
    )
}

fn plural(n: u64, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The `Time:` line of the report (without the label): the budget, how it was sized, the
/// extensions, the time used (to the newest ledger line), and, when findings were closed `time_budget`, what measuring
/// them would take.
pub fn report_line(view: &RunView) -> String {
    let b = &view.state.budget;
    let total = b.seconds;
    let how = match (&b.estimate, b.auto) {
        (Some(e), _) => format!(
            "auto: {}, {} {}",
            plural(e.runtimes, "runtime", "runtimes"),
            e.candidates,
            if e.candidates == 1 {
                "candidate"
            } else {
                "candidates"
            }
        ),
        (None, true) => "auto, not sized".to_owned(),
        (None, false) => String::new(),
    };
    let ext: u64 = b.extensions.iter().map(|x| x.seconds).sum();
    let lapsed: u64 = b.extensions.iter().map(|x| x.lapsed_seconds).sum();
    let mut notes: Vec<String> = Vec::new();
    if !how.is_empty() {
        notes.push(how);
    }
    if ext > 0 {
        let past = if lapsed > 0 {
            format!(", plus {} spent past the deadline", human(lapsed))
        } else {
            String::new()
        };
        notes.push(format!("extended by {}{past}", human(ext)));
    }
    let notes = if notes.is_empty() {
        String::new()
    } else {
        format!(" ({})", notes.join("; "))
    };
    // The newest ledger line: the report is rendered from the ledger, so the line is
    // deterministic for one ledger (a clock read would change every render).
    let end = view
        .ledger
        .lines()
        .iter()
        .map(|l| l.ts)
        .max()
        .unwrap_or(b.started_at);
    let used = u64::try_from((end - b.started_at).num_seconds().max(0)).unwrap_or(0);
    let mut line = format!(
        "budget {}{notes}, {} used.",
        human(total),
        human_coarse(used)
    );
    let r = remaining(view);
    if r.closed_for_time > 0 {
        let needed = r.closed_minutes;
        let total_needed = minutes_up(total) + needed;
        line.push_str(&format!(
            " {} {} closed for lack of time; measuring {} needs about {needed} more minutes (a new run with --budget {}m).",
            r.closed_for_time,
            if r.closed_for_time == 1 { "finding was" } else { "findings were" },
            if r.closed_for_time == 1 { "it" } else { "them" },
            total_needed,
        ));
    }
    line
}
