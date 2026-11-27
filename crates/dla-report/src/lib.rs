//! Report rendering for DLA.
//!
//! [`render_markdown`] turns a [`RunView`] into the Markdown report. It is a pure
//! function of the view: it reads no clock and no file, so the same input yields
//! byte-identical output. The completeness gate itself lives in `dla-core`
//! ([`dla_core::status`]); this crate only formats it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use dla_core::{
    EvidenceId, EvidenceRecord, Finding, FindingState, Measured, Resolution, RunView, UNCLASSIFIED,
    coverage::{BackendStatus, CoverageManifest, ResidualRow},
    discriminator::favors_of,
    evidence::CoverageStatus,
    finding::Polarity,
    gate::{self, GateTarget, Missing},
    machine::is_passing_growth,
    measured, status,
    trend::{GrowthSummary, PassedColumn},
};

/// Placeholder for an absent value in a table.
const DASH: &str = "\u{2014}";

/// Renders the run as a Markdown report. Deterministic: the same input yields
/// byte-identical output.
pub fn render_markdown(view: &RunView) -> String {
    let st = gate_status(view);
    let mut out = String::new();
    QUEUE.with(|q| *q.borrow_mut() = dla_core::queue::ReportFacts::of(view));
    TRIGGERS.with(|t| *t.borrow_mut() = dla_core::trigger::ReportFacts::of(view));

    out.push_str("# DLA report\n\n");
    let _ = writeln!(out, "Run: {}\n", code(&view.state.run_id));

    header(&mut out, view, &st);
    if !st.complete {
        open_items(&mut out, &st);
    }
    integrity(&mut out, view);
    workers(&mut out, view);
    findings(&mut out, view);
    explained(&mut out, view);
    merged_duplicates(&mut out, view);
    ruled_out(&mut out, view);
    residual_section(&mut out, view);
    blind_spots(&mut out, view);
    checklist(&mut out, view);
    evidence_index(&mut out, view);
    QUEUE.with(|q| *q.borrow_mut() = None);
    TRIGGERS.with(|t| *t.borrow_mut() = None);
    out
}

// ---------------------------------------------------------------- helpers

/// What the report says about completeness.
struct ReportStatus {
    complete: bool,
    blocking: Vec<Missing>,
    counts: dla_core::view::StatusCounts,
}

/// The status line and the open list come from the final gate, the same evaluation
/// `dla gate check --final` and `dla run status` use, so the three cannot disagree. The
/// counts come from the findings and the checklist.
fn gate_status(view: &RunView) -> ReportStatus {
    let all = gate::missing(view, GateTarget::Final, None);
    // Actions, as the gate and the status show them: items that wait for the same thing are
    // one line.
    let complete = all.is_empty();
    let blocking = gate::actions(&all).list;
    ReportStatus {
        complete,
        blocking,
        counts: status(view).counts,
    }
}

/// Collapses whitespace runs (including newlines) so free text stays on one line.
fn inline(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Escapes text for a table cell: one line, `|` escaped.
fn cell(s: &str) -> String {
    let one = inline(s);
    if one.is_empty() {
        return DASH.to_owned();
    }
    one.replace('\\', "\\\\").replace('|', "\\|")
}

/// An inline code span that survives backticks in the content.
fn code(s: &str) -> String {
    let s = inline(s);
    let longest = s
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or_default();
    let fence = "`".repeat(longest + 1);
    let pad = if s.starts_with('`') || s.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{s}{pad}{fence}")
}

/// A code span for use inside a table cell.
fn code_cell(s: &str) -> String {
    code(s).replace('|', "\\|")
}

fn ids(list: &[EvidenceId]) -> String {
    list.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn sites(list: &[String]) -> String {
    list.iter().map(|s| code(s)).collect::<Vec<_>>().join(", ")
}

/// Writers or removers listed in a static-only finding's section; the rest are counted.
const LISTED_SITES: usize = 5;

/// `sites`, cut after `cap` entries with the number left out.
fn capped_sites(list: &[String], cap: usize) -> String {
    if list.len() <= cap {
        return sites(list);
    }
    format!(
        "{}, and {} more (in the ledger annotation)",
        sites(&list[..cap]),
        list.len() - cap
    )
}

/// The archetype label: the archetype ID, or `UNCLASSIFIED` at the same weight as any
/// other label (it is a result, not a claim of novelty).
fn label(f: &Finding) -> String {
    inline(&f.archetype)
}

/// `- Label: ...` for a finding with a label record: the archetype and the evidence it
/// rests on; for `UNCLASSIFIED` the note that says why nothing fits.
fn label_line(view: &RunView, f: &Finding) -> Option<String> {
    let rec = view.ledger.label_record(&f.id)?;
    if rec.label == UNCLASSIFIED {
        return Some(match &rec.note {
            Some(n) => format!("Label: {UNCLASSIFIED} ({})", inline(n)),
            None => format!("Label: {UNCLASSIFIED}"),
        });
    }
    let kind_of = |id: &EvidenceId| {
        view.ledger
            .evidence_by_id(id)
            .map_or_else(String::new, |e| format!(" {}", e.kind))
    };
    let ev: Vec<String> = rec
        .satisfied
        .evidence
        .iter()
        .map(|id| format!("{id}{}", kind_of(id)))
        .collect();
    let mut s = format!(
        "Label: {} (evidence: {})",
        inline(&rec.label),
        ev.join(", ")
    );
    if rec.region_mismatch {
        s.push_str("; region does not match the archetype's");
    }
    if let Some(n) = &rec.note {
        s.push_str(&format!("; {}", inline(n)));
    }
    Some(s)
}

/// `- Strong: ...` naming the second class of evidence and the growth record it is
/// independent of.
fn strong_line(view: &RunView, f: &Finding) -> Option<String> {
    if f.state != FindingState::Strong {
        return None;
    }
    let cited: Vec<&EvidenceRecord> = f
        .supports
        .iter()
        .filter_map(|id| view.ledger.evidence_by_id(id))
        .collect();
    let second = cited.iter().copied().find(|e| match e.kind {
        dla_core::EvidenceKind::RetentionPath | dla_core::EvidenceKind::AllocStack => {
            dla_core::holders::names_holder(e, f)
        }
        dla_core::EvidenceKind::Intervention => dla_core::holders::InterventionResult::of(e)
            .is_some_and(|r| r.why_not_removal_of(&f.id).is_none()),
        _ => false,
    })?;
    let growth = cited
        .iter()
        .copied()
        .filter(|e| is_passing_growth(e))
        .find(|g| dla_core::evidence::records_independent(g, second));
    let what = match second.kind {
        dla_core::EvidenceKind::Intervention => "intervention that removed the growth".to_owned(),
        k => format!("{k} that names the holder"),
    };
    Some(match growth {
        Some(g) => format!(
            "Strong: second class {} {what}, independent of the growth record {} ({})",
            second.evidence_id,
            g.evidence_id,
            inline(&g.tool.name)
        ),
        None => format!("Strong: second class {} {what}", second.evidence_id),
    })
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

// ---------------------------------------------------------------- sections

thread_local! {
    /// The queue facts of the report being rendered (`None` outside `render_markdown`).
    static QUEUE: std::cell::RefCell<Option<dla_core::queue::ReportFacts>> =
        const { std::cell::RefCell::new(None) };
}

thread_local! {
    /// The trigger facts of the report being rendered (`None` outside `render_markdown`, and
    /// for a view without a run directory).
    static TRIGGERS: std::cell::RefCell<Option<dla_core::trigger::ReportFacts>> =
        const { std::cell::RefCell::new(None) };
}

/// How a closed finding was targeted, for its section or table row.
fn targeted_text(id: &dla_core::FindingId) -> Option<String> {
    QUEUE.with(|q| q.borrow().as_ref().and_then(|f| f.targeted(id)))
}

fn header(out: &mut String, view: &RunView, st: &ReportStatus) {
    let c = &st.counts;
    if st.complete {
        out.push_str("Status: COMPLETE\n\n");
    } else {
        let _ = writeln!(out, "Status: INCOMPLETE ({} open)\n", st.blocking.len());
    }
    let _ = writeln!(out, "{}\n", measured_line(&measured(view)));
    for w in coverage_warnings(view) {
        let _ = writeln!(out, "{w}\n");
    }
    let _ = writeln!(out, "Time: {}\n", dla_core::timebudget::report_line(view));
    if let Some(line) = history_line(view) {
        let _ = writeln!(out, "History: {line}\n");
    }
    let _ = writeln!(out, "Entry: {}\n", entry_line(view));
    let _ = writeln!(out, "Scope: {}\n", scope_line(view));
    let disputed = disputed_confirmed(view);
    let skipped = dla_core::release::grows_only_while_skipped(&view.ledger);
    let released_strong = skipped
        .iter()
        .filter(|f| f.state == FindingState::Strong)
        .count();
    // A finding goes in one bucket of the line: the verifier's word comes first, so a confirmed
    // finding it disputes is counted as disputed even when its family holds a confirmed
    // `N-RELEASE` sibling (its own section says so).
    let released_confirmed = skipped
        .iter()
        .filter(|f| f.state == FindingState::Confirmed)
        .filter(|f| !disputed.iter().any(|(id, _)| id == &f.id))
        .count();
    let labels: Vec<String> = disputed.into_iter().map(|(_, label)| label).collect();
    let mut line = findings_line(c, &labels, (released_confirmed, released_strong));
    if let Some(clause) = TRIGGERS.with(|t| t.borrow().as_ref().and_then(|f| f.clause())) {
        line.push_str("; ");
        line.push_str(&clause);
    }
    if let Some(clause) = unverified_clause(view) {
        line.push_str("; ");
        line.push_str(&clause);
    }
    let _ = writeln!(out, "{line}\n");
    let _ = writeln!(out, "Residual: {}\n", residual_line(view));
    let _ = writeln!(out, "Coverage: {}\n", coverage_line(view.coverage.as_ref()));
    if let Some(line) = QUEUE.with(|q| q.borrow().as_ref().and_then(|f| f.line())) {
        let _ = writeln!(out, "Queue: {line}\n");
    }
}

/// The times the stop hook stamped the run incomplete and the run went on, for the report's
/// history: the status says what the run is now, this what it was. `None` without a stamp.
fn history_line(view: &RunView) -> Option<String> {
    let lifted: Vec<String> = view
        .state
        .stop_stamps
        .iter()
        .filter_map(|s| {
            s.lifted_at.map(|l| {
                format!(
                    "stamped incomplete by the stop hook at {} and lifted at {} when work went on",
                    s.stamped_at.format("%H:%M:%SZ"),
                    l.format("%H:%M:%SZ")
                )
            })
        })
        .collect();
    (!lifted.is_empty()).then(|| lifted.join("; "))
}

/// The label the blind verifier read for each confirmed leak-side finding it reads as a
/// non-leak (`N-*`) although the finding carries a leak label (or none): the latest verification
/// of the finding decides. A disagreement between two leak labels is not one.
fn disputed_label(view: &RunView, f: &Finding) -> Option<String> {
    if f.state != FindingState::Confirmed || is_explanation(f) || f.archetype.starts_with("N-") {
        return None;
    }
    view.ledger
        .verifications(&f.id)
        .last()
        .filter(|v| v.label.starts_with("N-"))
        .map(|v| v.label.clone())
}

/// [`disputed_label`] of every finding it applies to, with the finding.
fn disputed_confirmed(view: &RunView) -> Vec<(dla_core::FindingId, String)> {
    view.ledger
        .findings()
        .filter(|f| view.ledger.merged_into(&f.id).is_none())
        .filter_map(|f| disputed_label(view, f).map(|l| (f.id.clone(), l)))
        .collect()
}

/// What a run aborted before the verifier ran says of a confirmed finding that no verification
/// record covers.
const NOT_VERIFIED_ABORTED: &str = "not verified: the run was aborted before the verifier ran";

/// `; not verified: ...` for the `Verified:` line of a finding of an aborted run, else nothing.
fn not_verified_note(view: &RunView) -> String {
    if view.state.status == dla_core::state::RunStatus::Aborted {
        format!("; {NOT_VERIFIED_ABORTED}")
    } else {
        String::new()
    }
}

/// The clause of the `Findings:` line that counts, apart, the confirmed findings of an aborted
/// run that the verifier never read; `None` when there are none or the run was not aborted.
fn unverified_clause(view: &RunView) -> Option<String> {
    if view.state.status != dla_core::state::RunStatus::Aborted {
        return None;
    }
    let n = dla_core::abort_rule::unverified_confirmed(view).len();
    (n > 0).then(|| {
        format!("confirmed but not verified (the run was aborted before the verifier ran) {n}")
    })
}

/// The `Findings:` line. The leak hypotheses and the non-leak explanations (the `N-*`
/// siblings) are counted apart: an explanation that held is `confirmed` on its side and
/// its leak-side root is `refuted` on the other, so one test read as two leaks when the two
/// were added together. When something was measured and no leak is confirmed or strong,
/// the line says so in words.
///
/// Every finding is in one bucket, so the counts add up: `disputed` are the labels the verifier
/// read for the confirmed findings it disputes, and `released` the confirmed and strong leaks
/// whose family holds a confirmed `N-RELEASE` sibling, those already disputed left out (the
/// caller does that: it knows which findings they are).
fn findings_line(
    c: &dla_core::view::StatusCounts,
    disputed: &[String],
    released: (usize, usize),
) -> String {
    let leak = c.by_state.minus(&c.non_leak);
    let n = &c.non_leak;
    // A confirmed finding the blind verifier reads as a non-leak (`N-*`) is not plain
    // "confirmed": it is counted apart, with what the verifier read.
    let apart = disputed.len().min(leak.confirmed);
    let mut labels: Vec<&str> = disputed.iter().map(String::as_str).collect();
    labels.sort_unstable();
    labels.dedup();
    // A confirmed finding whose family holds a confirmed `N-RELEASE` sibling grows only while
    // callers skip a release the API requires: it is counted apart from the confirmed leaks.
    let (released, released_strong) = (
        released.0.min(leak.confirmed - apart),
        released.1.min(leak.strong),
    );
    let mut confirmed = if apart > 0 {
        format!(
            "{}, confirmed but disputed by the verifier (reads {}) {apart}",
            leak.confirmed - apart - released,
            labels.join(", ")
        )
    } else {
        (leak.confirmed - released).to_string()
    };
    if released > 0 {
        confirmed.push_str(&format!(
            ", grows only while callers skip the release: {released}"
        ));
    }
    let mut strong = (leak.strong - released_strong).to_string();
    if released_strong > 0 {
        strong.push_str(&format!(
            ", grows only while callers skip the release: {released_strong}"
        ));
    }
    let mut line = format!(
        "Findings: {} enumerated, {} dispositioned; leak hypotheses: confirmed {}, strong {}, refuted {}, suspected {}, unresolved {}; non-leak explanations: confirmed {}, refuted {}, suspected {}, unresolved {}",
        c.enumerated,
        c.dispositioned,
        confirmed,
        strong,
        leak.refuted,
        leak.suspected_static_only,
        leak.unresolved,
        n.confirmed,
        n.refuted,
        n.suspected_static_only,
        n.unresolved
    );
    if c.measured && leak.confirmed + leak.strong - released - released_strong == 0 {
        line.push_str("; no leak confirmed");
    }
    line
}

/// `Coverage warning:` lines, right after `Measured:`: one per runtime that has discovered
/// entry points and exercised none of them, or fewer than half. The coverage line states
/// the counts as data; this says what they mean for the findings, the cause, and the
/// step that would measure the rest.
fn coverage_warnings(view: &RunView) -> Vec<String> {
    let Some(cov) = &view.coverage else {
        return Vec::new();
    };
    cov.workers
        .iter()
        .filter_map(|(rt, w)| {
            let discovered = w.entry_points.discovered.len();
            let exercised = w.entry_points.exercised.len();
            let iterations = w.iterations.unwrap_or(0);
            // A runtime that cannot record which entry points ran says "0 exercised"
            // after any run: that is its limit, not a result. With no iteration at all
            // the warning still holds.
            if discovered == 0
                || (exercised > 0 && exercised * 2 >= discovered)
                || (iterations > 0 && !gate::exercised_is_measured(cov, rt))
            {
                return None;
            }
            let step = gate::open_harness_step(view, rt);
            let cause = if exercised > 0 {
                "most entry points were never called".to_owned()
            } else if iterations == 0 {
                // A failed build is said once: by the step that follows when it is the
                // build step, else here (when no newer harness was tried since).
                match gate::unbuilt_harness(view, rt) {
                    Some(h) if gate::build_step_harness(view, rt).is_none() => format!(
                        "harness {h} did not build, so no workload iteration was recorded for it"
                    ),
                    _ => "no workload iteration was recorded for it".to_owned(),
                }
            } else {
                let loaded = match w.project_classes {
                    Some(1) => "; 1 project class loaded".to_owned(),
                    Some(n) if n > 1 => format!("; {n} project classes loaded"),
                    _ => String::new(),
                };
                format!(
                    "{} ran and none called an entry point, so the driver did not reach the code under test{loaded}",
                    plural_n(iterations, "iteration")
                )
            };
            let next = match step {
                gate::HarnessStep::Do(step) => format!("To measure: {step}."),
                gate::HarnessStep::Blocked(why) => {
                    format!("Blocked: intake.workload records {}.", why.trim_end_matches(['.', ' ']))
                }
            };
            Some(format!(
                "Coverage warning: this run exercised {exercised} of {discovered} entry points ({}), so findings outside them rest on reading the code and are hypotheses, not measured leaks. Cause: {cause}. {next}",
                inline(rt),
            ))
        })
        .collect()
}

/// The `Measured:` line, right after `Status:` (which stays exactly as it was: graders
/// match it). A run that measured nothing says so, in words that cannot be read as a
/// clean result.
fn measured_line(m: &Measured) -> String {
    if !m.any() {
        return "Measured: nothing. No harness ran and nothing was sampled or captured, so every finding rests on reading the code: this report neither shows nor rules out a leak.".to_owned();
    }
    format!(
        "Measured: {}, {}, {}",
        plural(m.harness_runs, "harness run", "harness runs"),
        plural(m.series, "series", "series"),
        plural(m.captures, "capture", "captures"),
    )
}

/// The entry path, with the artifacts of an artifact run.
fn entry_line(view: &RunView) -> String {
    let st = &view.state;
    if st.artifacts.is_empty() {
        return st.entry.to_string();
    }
    format!(
        "{} ({})",
        st.entry,
        st.artifacts
            .iter()
            .map(|a| code(a))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// The scope kind and path, the excluded paths, and the source files left out of each
/// runtime because they lie outside the scope (from the coverage manifest).
fn scope_line(view: &RunView) -> String {
    let (kind, path, excluded) = match &view.state.scope {
        Some(sc) => (
            sc.kind.to_string(),
            sc.path.as_str(),
            sc.excluded.as_slice(),
        ),
        None => ("repository".to_owned(), ".", &[][..]),
    };
    let head = if path.is_empty() || path == "." {
        kind
    } else {
        format!("{kind} {}", code(path))
    };
    let excluded = if excluded.is_empty() {
        "none".to_owned()
    } else {
        excluded
            .iter()
            .map(|e| code(e))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let out_of_scope: Vec<String> = view
        .coverage
        .iter()
        .flat_map(|c| c.workers.iter())
        .filter(|(_, w)| w.out_of_scope > 0)
        .map(|(name, w)| format!("{} {}", inline(name), plural_n(w.out_of_scope, "file")))
        .collect();
    let out_of_scope = if out_of_scope.is_empty() {
        "none".to_owned()
    } else {
        out_of_scope.join(", ")
    };
    // Build logic (Gradle scripts, buildSrc) is never read: named only when some was found.
    let build_logic: Vec<String> = view
        .coverage
        .iter()
        .flat_map(|c| c.workers.iter())
        .filter(|(_, w)| w.build_logic > 0)
        .map(|(name, w)| format!("{} {}", inline(name), plural_n(w.build_logic, "file")))
        .collect();
    let build_logic = if build_logic.is_empty() {
        String::new()
    } else {
        format!("; build logic not read: {}", build_logic.join(", "))
    };
    let left_out = left_out_text(view);
    format!("{head}; excluded: {excluded}; out of scope: {out_of_scope}{build_logic}{left_out}")
}

/// What the static stages left out by default (vendored directories, generated files, the
/// runtimes whose whole code base lies there) and what the user brought back. Empty when
/// neither applies.
fn left_out_text(view: &RunView) -> String {
    let mut out = String::new();
    if let Some(l) = view.coverage.as_ref().map(|c| &c.left_out)
        && !l.is_empty()
    {
        let mut parts: Vec<String> = l
            .directories
            .iter()
            .map(|d| {
                format!(
                    "{} ({}, {})",
                    code(&d.path),
                    d.reason.as_str(),
                    plural_n(d.files, "file")
                )
            })
            .collect();
        if l.generated_files > 0 {
            parts.push(format!("{} generated", plural_n(l.generated_files, "file")));
        }
        for r in &l.emptied_runtimes {
            parts.push(format!(
                "no {} code is analysed, all of it is in left-out paths",
                inline(&r.runtime)
            ));
        }
        out.push_str(&format!(
            "; left out by default: {} (`dla run init --include <path>` brings a path back)",
            parts.join(", ")
        ));
    }
    if !view.state.include.is_empty() {
        out.push_str(&format!(
            "; brought back: {}",
            view.state
                .include
                .iter()
                .map(|p| code(p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    out
}

/// The iterations run and entry points exercised per worker, or `not measured` when the
/// manifest records neither.
fn coverage_line(coverage: Option<&CoverageManifest>) -> String {
    let Some(cov) = coverage else {
        return "not measured".to_owned();
    };
    let parts: Vec<String> = cov
        .workers
        .iter()
        .filter(|(_, w)| {
            w.iterations.is_some()
                || !w.entry_points.discovered.is_empty()
                || !w.entry_points.exercised.is_empty()
        })
        .map(|(name, w)| {
            let iterations = w
                .iterations
                .map_or_else(|| "no iterations".to_owned(), |n| plural_n(n, "iteration"));
            let exercised = w.entry_points.exercised.len();
            let discovered = w.entry_points.discovered.len().max(exercised);
            // A run that executed the project's classes but matched no listed entry point
            // must not read as if nothing ran.
            let project = match w.project_classes {
                Some(1) if exercised == 0 => ", 1 project class loaded".to_owned(),
                Some(n) if exercised == 0 && n > 1 => format!(", {n} project classes loaded"),
                _ => String::new(),
            };
            // No entry point is listed for the runtime (`dla workload discover` did not run, or
            // found none for a library): "0 of 0" would read as nothing covered, and nothing
            // was enumerated to be covered.
            if discovered == 0 {
                return format!(
                    "{} {iterations}, entry points not enumerated (none listed for this runtime){project}",
                    inline(name)
                );
            }
            format!(
                "{} {iterations}, {exercised} of {discovered} entry points exercised{project}",
                inline(name)
            )
        })
        .collect();
    if parts.is_empty() {
        "not measured".to_owned()
    } else {
        parts.join("; ")
    }
}

fn plural_n(n: u64, noun: &str) -> String {
    format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
}

fn residual_line(view: &RunView) -> String {
    let Some(cov) = view.coverage.as_ref() else {
        return "not measured".to_owned();
    };
    if cov.residual.is_empty() {
        return "not measured".to_owned();
    }
    if cov.residual.len() <= RESIDUAL_HEADER_ROWS {
        return cov
            .residual
            .iter()
            .map(|r| residual_row(view, r))
            .collect::<Vec<_>>()
            .join("; ");
    }
    // Many series: the header names the ones above their threshold (with the open item that
    // covers each) and counts the rest; every row is in the section `## Residual by series`.
    let (above, below): (Vec<&ResidualRow>, Vec<&ResidualRow>) =
        cov.residual.iter().partition(|r| r.residual > r.threshold);
    let mut parts: Vec<String> = above
        .iter()
        .take(RESIDUAL_HEADER_ROWS)
        .map(|r| residual_row(view, r))
        .collect();
    if above.len() > RESIDUAL_HEADER_ROWS {
        parts.push(format!(
            "{} more above threshold",
            above.len() - RESIDUAL_HEADER_ROWS
        ));
    }
    if !below.is_empty() {
        parts.push(format!("{} series below threshold", below.len()));
    }
    format!(
        "{} measured; {} (all rows: section Residual by series)",
        plural_n(cov.residual.len() as u64, "row"),
        parts.join("; ")
    )
}

/// Residual rows the header line spells out one by one.
const RESIDUAL_HEADER_ROWS: usize = 3;

/// `## Residual by series`: every row, when the header line only summarizes them.
fn residual_section(out: &mut String, view: &RunView) {
    let Some(cov) = &view.coverage else { return };
    if cov.residual.len() <= RESIDUAL_HEADER_ROWS {
        return;
    }
    out.push_str("## Residual by series\n\n");
    for r in &cov.residual {
        let _ = writeln!(out, "- {}", residual_row(view, r));
    }
    out.push('\n');
}

/// How long a remedy may be in a residual row before it is cut.
const REMEDY_CLIP: usize = 120;

fn residual_row(view: &RunView, r: &ResidualRow) -> String {
    let series = r
        .series
        .as_deref()
        .map_or(String::new(), |s| format!("{s} "));
    let mut s = format!(
        "{series}{} {} {} unattributed of {} total (threshold {})",
        r.region,
        num(r.residual),
        r.unit,
        num(r.total_slope),
        num(r.threshold)
    );
    if r.residual > r.threshold {
        s.push_str(" ABOVE THRESHOLD");
        if let Some(o) = &r.open_item {
            match dla_core::gate::residual_row_resolution(view, r) {
                None => {
                    let _ = write!(s, ", see {o}");
                }
                Some(res) => {
                    let how = match res {
                        Resolution::Unresolved { reason, remedy } => {
                            let remedy = inline(remedy);
                            let clipped: String = remedy.chars().take(REMEDY_CLIP).collect();
                            let dots = if remedy.chars().count() > REMEDY_CLIP {
                                "..."
                            } else {
                                ""
                            };
                            format!("resolved as {reason}: {clipped}{dots}")
                        }
                        Resolution::Hypothesis(f) => format!("resolved by {f}"),
                    };
                    if dla_core::gate::residual_row_covered(view, r) {
                        let _ = write!(s, ", {o} {how}");
                    } else {
                        let _ = write!(
                            s,
                            ", NOT COVERED: {o} {how}, but this row was listed after that resolution"
                        );
                    }
                }
            }
        }
    }
    s
}

fn open_items(out: &mut String, st: &ReportStatus) {
    out.push_str("## Open items\n\n");
    for b in &st.blocking {
        // Descriptions may already end in punctuation (a checklist question ends in "?").
        let d = inline(b.description.trim_end());
        let stop = if d.ends_with(['.', '?', '!']) {
            ""
        } else {
            "."
        };
        let _ = writeln!(
            out,
            "- **{}**: {d}{stop} Fix: {}",
            inline(&b.id),
            code(&b.fix)
        );
    }
    out.push('\n');
}

/// Reseals recorded in the ledger, and a note when `report.md` was edited after the last
/// render. Nothing is printed for a run with neither.
fn integrity(out: &mut String, view: &RunView) {
    let reseals = view.ledger.reseals();
    if reseals.is_empty() && !view.report_modified {
        return;
    }
    out.push_str("## Integrity\n\n");
    for r in reseals {
        let _ = writeln!(
            out,
            "- Resealed {}: {}",
            r.files
                .iter()
                .map(|f| code(f))
                .collect::<Vec<_>>()
                .join(", "),
            inline(&r.reason)
        );
    }
    if view.report_modified {
        out.push_str(
            "- `report.md` was modified after it was rendered; this report replaces it.\n",
        );
    }
    out.push('\n');
}

fn rank(s: FindingState) -> u8 {
    match s {
        FindingState::Strong => 0,
        FindingState::Confirmed => 1,
        FindingState::SuspectedStaticOnly => 2,
        FindingState::Unresolved => 3,
        FindingState::Candidate => 4,
        FindingState::UnderTest => 5,
        FindingState::Refuted => 6,
    }
}

fn classes(view: &RunView, f: &Finding) -> String {
    let set: BTreeSet<&'static str> = f
        .supports
        .iter()
        .chain(&f.contradicts)
        .filter_map(|id| view.ledger.evidence_by_id(id))
        .map(|e| e.kind.class().as_str())
        .collect();
    set.into_iter().collect::<Vec<_>>().join(", ")
}

/// The first this many `suspected_static_only` findings, in rank order, get a full
/// section; the rest are rows of one table.
const FULL_STATIC_ONLY: usize = 15;

/// The sentence above the `suspected_static_only` findings.
const TIER_NOTE: &str = dla_core::rank::TIER_NOTE;

/// The rank of a finding, 1 (strongest) to 6 (demoted). Pure function of the finding and
/// its census annotation, so the order is the same on every run.
fn static_tier(view: &RunView, f: &Finding) -> u8 {
    dla_core::rank::tier(f, view.ledger.annotation(&f.id))
}

/// The origin cell of a table row: the origin, then the other origins that raised the site.
fn origins_cell(f: &Finding) -> String {
    let mut cell = code_cell(f.origin.as_str());
    for o in &f.also_origins {
        cell.push_str(", ");
        cell.push_str(&code_cell(o.as_str()));
    }
    cell
}

fn findings(out: &mut String, view: &RunView) {
    out.push_str("## Findings\n\n");
    let mut list: Vec<&Finding> = view
        .ledger
        .findings()
        .filter(|f| f.state != FindingState::Refuted && view.ledger.merged_into(&f.id).is_none())
        .filter(|f| !is_explanation(f))
        .collect();
    // Stable: inside a state the ledger order stays; inside suspected_static_only the
    // tier decides first.
    // Confirmed and strong leaks lead, those a caller of the public API was shown to trigger
    // first.
    let triggered =
        |f: &Finding| TRIGGERS.with(|t| t.borrow().as_ref().is_some_and(|t| t.reproduced(&f.id)));
    list.sort_by_key(|f| {
        let (tier, multi) = if f.state == FindingState::SuspectedStaticOnly {
            (static_tier(view, f), dla_core::rank::multi_origin_key(f))
        } else {
            (0, 0)
        };
        let top = matches!(f.state, FindingState::Confirmed | FindingState::Strong);
        (
            u8::from(!top),
            // Confirmed and disputed by the verifier: after the undisputed ones.
            u8::from(disputed_label(view, f).is_some()),
            u8::from(top && !triggered(f)),
            rank(f.state),
            tier,
            multi,
        )
    });
    if list.is_empty() {
        let explained = view
            .ledger
            .findings()
            .any(|f| is_explanation(f) && view.ledger.merged_into(&f.id).is_none());
        out.push_str(if explained {
            "No leak hypotheses to report; the confirmed findings are non-leak explanations, listed below.\n\n"
        } else {
            "No findings to report.\n\n"
        });
        return;
    }
    let mut static_seen = 0;
    let mut table: Vec<&Finding> = Vec::new();
    let flush = |out: &mut String, table: &mut Vec<&Finding>| {
        if !table.is_empty() {
            static_table(out, table);
            table.clear();
        }
    };
    for f in list {
        if f.state == FindingState::SuspectedStaticOnly {
            if static_seen == 0 {
                let _ = writeln!(out, "{TIER_NOTE}\n");
            }
            static_seen += 1;
            if static_seen > FULL_STATIC_ONLY {
                table.push(f);
                continue;
            }
        } else {
            flush(out, &mut table);
        }
        finding_section(out, view, f);
    }
    flush(out, &mut table);
}

/// A non-leak explanation that held: an `N-*` sibling in `confirmed`. It says why the
/// growth is not a leak, so it is listed apart from the leak hypotheses.
fn is_explanation(f: &Finding) -> bool {
    f.state == FindingState::Confirmed && f.polarity() == Polarity::NonLeak
}

/// `## Explained, not leaks`: the confirmed non-leak explanations, with the evidence that
/// decided them. Left out when there are none.
fn explained(out: &mut String, view: &RunView) {
    let list: Vec<&Finding> = view
        .ledger
        .findings()
        .filter(|f| is_explanation(f) && view.ledger.merged_into(&f.id).is_none())
        .collect();
    if list.is_empty() {
        return;
    }
    out.push_str("## Explained, not leaks\n\n");
    let _ = writeln!(
        out,
        "{} non-leak {} held: the growth covered has a benign cause (a warm-up, a bounded cache, load that scales with traffic, a high-water mark), so each is an explanation, not a leak and not a defect.\n",
        list.len(),
        if list.len() == 1 {
            "explanation"
        } else {
            "explanations"
        }
    );
    for f in list {
        finding_section(out, view, f);
    }
}

/// `#### Other static-only findings (<n>)`: one row per finding, no remedy per row. A
/// remedy shared by every row is printed once above the table.
fn static_table(out: &mut String, rows: &[&Finding]) {
    let _ = writeln!(out, "#### Other static-only findings ({})\n", rows.len());
    let first = rows.first().and_then(|f| f.remedy.as_deref());
    if let Some(r) = first
        && rows.iter().all(|f| f.remedy.as_deref() == Some(r))
    {
        let _ = writeln!(out, "Remedy for all of them: {}\n", inline(r));
    }
    let targeted = QUEUE.with(|q| q.borrow().is_some());
    if targeted {
        out.push_str(
            "| ID | Origin | Holder | Owner or site | Reason | Targeted |\n|---|---|---|---|---|---|\n",
        );
    } else {
        out.push_str("| ID | Origin | Holder | Owner or site | Reason |\n|---|---|---|---|---|\n");
    }
    for f in rows {
        let place = f.owner.as_deref().or(f.site.as_deref());
        let tail = if targeted {
            format!(
                " {} |",
                targeted_text(&f.id).map_or_else(|| DASH.to_owned(), |t| inline(&t))
            )
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |{tail}",
            f.id,
            origins_cell(f),
            f.holder.map_or_else(|| DASH.to_owned(), |h| h.to_string()),
            place.map_or_else(|| DASH.to_owned(), code_cell),
            f.reason.map_or_else(|| DASH.to_owned(), |r| r.to_string()),
        );
    }
    out.push('\n');
}

/// A number without trailing zeros: whole numbers print as integers, others with at
/// most four decimals.
fn num(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 {
        return format!("{x:.0}");
    }
    let t = format!("{x:.4}");
    t.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// `<column> <slope> <unit>/<axis> (<shape>, gate pass, <evidence>)` for a confirmed or
/// strong leak-side finding: the passing column of its growth evidence that is tied to it
/// (by `finding`, equal site or `candidates`). When the cited evidence has passing columns
/// but none is tied, the line says so and names them rather than presenting another
/// holder's column as this finding's growth. A non-leak finding has none.
fn growth_line(view: &RunView, f: &Finding) -> Option<String> {
    if !matches!(f.state, FindingState::Confirmed | FindingState::Strong)
        || f.polarity() != Polarity::Leak
    {
        return None;
    }
    let mut untied: Option<(Vec<String>, EvidenceId)> = None;
    let mut bare: Option<EvidenceId> = None;
    for e in f
        .supports
        .iter()
        .filter_map(|id| view.ledger.evidence_by_id(id))
        .filter(|e| is_passing_growth(e))
    {
        let Some(summary) = GrowthSummary::from_result(&e.result) else {
            bare.get_or_insert_with(|| e.evidence_id.clone());
            continue;
        };
        // The column at the finding's own site is its growth before any other tied column.
        if let Some(c) = summary
            .passed
            .iter()
            .filter(|c| c.tied_to_owner(&f.id, f.site.as_deref(), f.owner.as_deref()))
            .min_by_key(|c| f.site.is_none() || c.site.as_deref() != f.site.as_deref())
        {
            return Some(growth_text(c, &summary, &e.evidence_id));
        }
        if untied.is_none() && !summary.passed.is_empty() {
            let names = summary.passed.iter().map(|c| c.column.clone()).collect();
            untied = Some((names, e.evidence_id.clone()));
        }
        if summary.passed.is_empty() {
            bare.get_or_insert_with(|| e.evidence_id.clone());
        }
    }
    match (untied, bare) {
        (Some((names, id)), _) => {
            let list = names
                .iter()
                .take(5)
                .map(|n| code(n))
                .collect::<Vec<_>>()
                .join(", ");
            let more = names.len().saturating_sub(5);
            let more = if more > 0 {
                format!(" and {more} more")
            } else {
                String::new()
            };
            Some(format!(
                "not tied to this finding (passing columns of {id}: {list}{more})"
            ))
        }
        (None, Some(id)) => Some(format!("gate pass, {id}")),
        (None, None) => None,
    }
}

/// `Explained by <N-ID> (discriminator E<n>)` for a confirmed non-leak finding.
fn explained_by(view: &RunView, f: &Finding) -> Option<String> {
    if !matches!(f.state, FindingState::Confirmed | FindingState::Strong)
        || f.polarity() != Polarity::NonLeak
    {
        return None;
    }
    let h = f.hypothesis.as_deref()?;
    // The hypothesis reads `N-WARM: <text>`; name only the id.
    let h = h.split(':').next().unwrap_or(h).trim();
    let d = f
        .supports
        .iter()
        .filter_map(|id| view.ledger.evidence_by_id(id))
        .find(|e| e.kind == dla_core::EvidenceKind::Discriminator)?;
    Some(format!(
        "Explained by {} (discriminator {})",
        inline(h),
        d.evidence_id
    ))
}

fn growth_text(c: &PassedColumn, s: &GrowthSummary, id: &EvidenceId) -> String {
    let mut text = code(&c.column);
    if let Some(slope) = c.slope {
        let unit = c.unit.map_or("units", |u| u.as_str());
        let axis = s.axis.as_deref().unwrap_or("x");
        let _ = write!(text, " {} {unit}/{}", num(slope), inline(axis));
    }
    let shape = c
        .shape
        .and_then(|sh| serde_json::to_value(sh).ok())
        .and_then(|v| v.as_str().map(str::to_owned));
    match shape {
        Some(sh) => format!("{text} ({sh}, gate pass, {id})"),
        None => format!("{text} (gate pass, {id})"),
    }
}

/// `<test> (<favors>) <evidence>` for each discriminator that refuted the finding.
fn ruled_out_by(view: &RunView, f: &Finding) -> Option<String> {
    let parts: Vec<String> = f
        .contradicts
        .iter()
        .filter_map(|id| view.ledger.evidence_by_id(id))
        .filter(|e| e.kind == dla_core::EvidenceKind::Discriminator)
        .map(|e| {
            let test = e
                .result
                .get("test")
                .and_then(|t| t.as_str())
                .map_or_else(|| "discriminator".to_owned(), inline);
            match favors_of(e) {
                Some(fv) => format!("{test} ({fv}) {}", e.evidence_id),
                None => format!("{test} {}", e.evidence_id),
            }
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join("; "))
}

/// The untied growing columns of the capture summary whose evidence a growth-series finding
/// stands for: the finding is one obligation for all of them, so the report lists them (a
/// count and the three largest) instead of hiding them.
fn also_growing_line(view: &RunView, f: &Finding) -> Option<String> {
    if f.origin.as_str() != "growth-series" {
        return None;
    }
    for e in f
        .supports
        .iter()
        .filter_map(|e| view.ledger.evidence_by_id(e))
    {
        let Some(tid) = e.result.get("trend").and_then(|t| t.as_str()) else {
            continue;
        };
        let Some(t) = view.trends.iter().find(|t| t.id == tid) else {
            continue;
        };
        if !view.summary_series.contains(&t.series) {
            continue;
        }
        let mut cols: Vec<&dla_core::trend::ColumnTrend> = t
            .columns
            .iter()
            .filter(|c| dla_core::gate::non_flat(c) && !dla_core::gate::is_whole_process(&c.name))
            // The capture rule's own pass: a column that failed it does not grow.
            .filter(|c| dla_core::grown::counts_as_growth(c))
            .filter(|c| c.finding.is_none() && c.candidates.is_empty())
            .collect();
        if cols.is_empty() {
            continue;
        }
        cols.sort_by(|a, b| b.rise.abs().total_cmp(&a.rise.abs()));
        let names: Vec<String> = cols.iter().take(3).map(|c| code(&c.name)).collect();
        return Some(format!(
            "{} column(s) of {} tied to no finding; the largest: {}{}",
            cols.len(),
            t.id,
            names.join(", "),
            if cols.len() > 3 { ", ..." } else { "" }
        ));
    }
    None
}

fn finding_section(out: &mut String, view: &RunView, f: &Finding) {
    match (&f.parent, &f.hypothesis) {
        (Some(_), Some(h)) => {
            let _ = writeln!(
                out,
                "### {}: Hypothesis {} ({})\n",
                f.id,
                inline(h),
                label(f)
            );
        }
        _ => {
            let _ = writeln!(out, "### {}: **{}**\n", f.id, label(f));
        }
    }
    let _ = writeln!(out, "- State: {}", f.state_label());
    let _ = writeln!(out, "- Origin: {}", code(f.origin.as_str()));
    if !f.also_origins.is_empty() {
        let also: Vec<String> = f.also_origins.iter().map(|o| code(o.as_str())).collect();
        let _ = writeln!(out, "- Also raised by: {}", also.join(", "));
    }
    if let Some(l) = label_line(view, f) {
        let _ = writeln!(out, "- {l}");
    }
    if let Some(g) = growth_line(view, f) {
        let _ = writeln!(out, "- Growth: {g}");
    }
    if let Some(g) = also_growing_line(view, f) {
        let _ = writeln!(out, "- Also growing in the same capture: {g}");
    }
    if let Some(t) = TRIGGERS.with(|t| {
        t.borrow()
            .as_ref()
            .and_then(|t| t.line(&f.id).map(str::to_owned))
    }) {
        let _ = writeln!(out, "- Trigger: {}", inline(&t));
    }
    if let Some(x) = explained_by(view, f) {
        let _ = writeln!(out, "- {x}");
    }
    let cl = classes(view, f);
    if !cl.is_empty() {
        let _ = writeln!(out, "- Evidence classes: {cl}");
    }
    if let Some(x) = strong_line(view, f) {
        let _ = writeln!(out, "- {x}");
    }
    if let Some(h) = f.holder {
        let _ = writeln!(out, "- Holder: {h}");
    }
    if let Some(o) = &f.owner {
        let _ = writeln!(out, "- Owner: {}", code(o));
    }
    if let Some(r) = f.region {
        let _ = writeln!(out, "- Region: {r}");
    }
    if let Some(s) = f.shape {
        let _ = writeln!(out, "- Shape: {s}");
    }
    if let Some(l) = f.locus {
        let _ = writeln!(out, "- Locus: {l}");
    }
    if let Some(s) = &f.site {
        let _ = writeln!(out, "- Site: {}", code(s));
    }
    if let Some(a) = view.ledger.annotation(&f.id) {
        // A census finding can name dozens of writers; on a static-only finding (one of
        // hundreds on a service) the first few identify it and the rest are in the ledger.
        let cap = if f.state == FindingState::SuspectedStaticOnly {
            LISTED_SITES
        } else {
            usize::MAX
        };
        if !a.writers.is_empty() {
            let _ = writeln!(out, "- Writers: {}", capped_sites(&a.writers, cap));
        }
        if !a.removers.is_empty() {
            let _ = writeln!(out, "- Removers: {}", capped_sites(&a.removers, cap));
        }
        for n in &a.notes {
            let _ = writeln!(out, "- Annotation: {}", inline(n));
        }
    }
    if let Some(v) = view.ledger.verifications(&f.id).last() {
        let _ = writeln!(out, "- Verified: {}", verified_text(f, v));
    } else if matches!(f.state, FindingState::Confirmed | FindingState::Strong)
        && f.polarity() == Polarity::Leak
    {
        let _ = writeln!(out, "- Verified: no{}", not_verified_note(view));
    }
    let dups = view.ledger.duplicates_of(&f.id);
    if !dups.is_empty() {
        let d: Vec<String> = dups.iter().map(ToString::to_string).collect();
        let _ = writeln!(out, "- Merged duplicates: {}", d.join(", "));
    }
    if let Some(r) = f.repair_class {
        let _ = writeln!(out, "- Repair class: {r}");
    }
    if !f.amplifiers.is_empty() {
        let a: Vec<String> = f.amplifiers.iter().map(|a| inline(a)).collect();
        let _ = writeln!(out, "- Amplifiers: {}", a.join(", "));
    }
    if let Some(p) = &f.parent {
        let _ = writeln!(out, "- Sibling hypothesis of: {p}");
    }
    if let Some(h) = &f.hypothesis {
        let _ = writeln!(out, "- Hypothesis: {}", inline(h));
    }
    if let Some(r) = &f.refute_if {
        let _ = writeln!(out, "- Refuted if: {}", inline(r));
    }
    if let Some(r) = &f.remover {
        let note = dla_core::release::decision_note(view, f)
            .map_or_else(String::new, |n| format!(" ({n})"));
        let _ = writeln!(out, "- Release the caller must make: {}{note}", code(r));
    }
    if let Some(s) = dla_core::release::release_confirmed(&view.ledger, &f.id).filter(|_| {
        matches!(f.state, FindingState::Confirmed | FindingState::Strong)
            && f.polarity() == Polarity::Leak
    }) {
        let _ = writeln!(
            out,
            "- Grows only while callers skip the release: {} confirmed (the release {} called on every unit leaves the series flat); counted apart from the confirmed leaks",
            s.id,
            s.remover.as_deref().map_or_else(String::new, code)
        );
    }
    if !f.siblings.is_empty() {
        let s: Vec<String> = f.siblings.iter().map(ToString::to_string).collect();
        let _ = writeln!(out, "- Sibling hypotheses: {}", s.join(", "));
    }
    if let Some(s) = &f.suggested_archetype {
        let _ = writeln!(out, "- Suggested archetype (unconfirmed): {}", inline(s));
    }
    if let Some(r) = f.reason {
        let _ = writeln!(out, "- Reason: {r}");
    }
    if matches!(
        f.state,
        FindingState::SuspectedStaticOnly | FindingState::Unresolved
    ) && let Some(t) = targeted_text(&f.id)
    {
        let _ = writeln!(out, "- {t}");
    }
    if let Some(r) = &f.remedy {
        let _ = writeln!(out, "- Remedy: {}", inline(r));
    }
    let _ = writeln!(out, "- Supporting evidence: {}", ids(&f.supports));
    if !f.contradicts.is_empty() {
        let _ = writeln!(out, "- Contradicting evidence: {}", ids(&f.contradicts));
    }
    if let Some(e) = &f.explanation {
        let _ = writeln!(out, "- Explanation ({}): {}", e.tag, inline(&e.text));
        if let Some(x) = view.ledger.explanation(&f.id) {
            let _ = writeln!(
                out,
                "- Explanation structure: {}",
                inline(&dla_core::explain::structured_line(x))
            );
        }
    }
    out.push('\n');
}

/// `agrees (...)` or the disagreement, naming both values and both mechanisms.
fn verified_text(f: &Finding, v: &dla_core::Verification) -> String {
    let bundle: String = v.bundle.chars().take(12).collect();
    let site = v.site.as_deref().map_or_else(|| "none".to_owned(), code);
    let label_ok = v.agrees_label.unwrap_or(true);
    if v.agrees_site && v.agrees_holder && label_ok {
        return format!(
            "agrees (verifier site {site}, holder {}, label {}; bundle {bundle})",
            v.holder,
            inline(&v.label)
        );
    }
    let mut parts = Vec::new();
    if !v.agrees_site {
        parts.push(format!(
            "site: finding {} vs verifier {site}",
            f.site.as_deref().map_or_else(|| "none".to_owned(), code)
        ));
    }
    if !v.agrees_holder {
        parts.push(format!(
            "holder: finding {} vs verifier {}",
            f.holder.map_or("none", |h| h.as_str()),
            v.holder
        ));
    }
    if !label_ok {
        parts.push(format!(
            "label: finding {} vs verifier {}",
            inline(&f.archetype),
            inline(&v.label)
        ));
    }
    let own = f
        .explanation
        .as_ref()
        .map_or_else(|| "not stated".to_owned(), |e| inline(&e.text));
    format!(
        "DISAGREES ({}; bundle {bundle}); finding mechanism: {own}; verifier mechanism: {}",
        parts.join("; "),
        inline(&v.mechanism)
    )
}

/// The Workers section: per worker its status, the slice lines merged, whether it
/// finished and whether a torn final line was dropped; then the slices with lines the
/// main ledger has not merged. Omitted for a run without workers.
fn workers(out: &mut String, view: &RunView) {
    let mut names: BTreeSet<&str> = view.state.workers.iter().map(|w| w.name.as_str()).collect();
    names.extend(view.ledger.slices().map(|(w, _)| w));
    names.extend(view.slices.iter().map(|s| s.worker.as_str()));
    if names.is_empty() {
        return;
    }
    out.push_str("## Workers\n\n");
    for w in &names {
        let entry = view.state.worker(w);
        let info = view.ledger.slice_info(w);
        let file = view.slices.iter().find(|s| s.worker == *w);
        let status = entry.map_or("unregistered".to_owned(), |e| e.status.to_string());
        let runtime = entry
            .map(|e| e.runtime.clone())
            .or_else(|| {
                info.and_then(|i| i.started.as_ref())
                    .map(|s| s.runtime.clone())
            })
            .unwrap_or_else(|| (*w).to_owned());
        let merged = view.ledger.merged_lines(w);
        let lines = match file {
            Some(f) if f.lines > merged => format!("{merged} of {} lines merged", f.lines),
            _ => plural_n(merged, "line") + " merged",
        };
        let done = info.is_some_and(|i| i.done.is_some()) || file.is_some_and(|f| f.done);
        let torn = info
            .and_then(|i| i.merged.as_ref())
            .is_some_and(|m| m.torn_tail)
            || file.is_some_and(|f| f.torn_tail);
        let _ = writeln!(
            out,
            "- **{}** ({}): status {status}; {lines}; slice {}; torn tail: {}",
            inline(w),
            inline(&runtime),
            if done { "done" } else { "not done" },
            if torn {
                "yes, final line dropped"
            } else {
                "no"
            }
        );
    }
    out.push('\n');
    let unmerged = view.unmerged_slices();
    if !unmerged.is_empty() {
        out.push_str("### Unmerged slices\n\n");
        for s in unmerged {
            let _ = writeln!(
                out,
                "- **{}**: {} in ledger.{}.jsonl, {} merged. Fix: {}",
                inline(&s.worker),
                plural_n(s.lines, "line"),
                inline(&s.worker),
                view.ledger.merged_lines(&s.worker),
                code("dla ledger merge")
            );
        }
        out.push('\n');
    }
}

/// Findings folded into another finding of the same leak (a cross-worker merge or `dla ledger duplicate`).
fn merged_duplicates(out: &mut String, view: &RunView) {
    let mut rows: Vec<String> = Vec::new();
    for f in view.ledger.findings() {
        if let Some(c) = view.ledger.merged_into(&f.id) {
            rows.push(format!(
                "- **{}** merged into **{c}** (a duplicate of the same leak; its evidence {} is kept on {c})",
                f.id,
                if f.supports.is_empty() {
                    DASH.to_owned()
                } else {
                    ids(&f.supports)
                }
            ));
        }
    }
    if rows.is_empty() {
        return;
    }
    out.push_str("## Merged duplicates\n\n");
    for r in rows {
        let _ = writeln!(out, "{r}");
    }
    out.push('\n');
}

fn ruled_out(out: &mut String, view: &RunView) {
    out.push_str("## Ruled out\n\n");
    let mut any = false;
    for f in view
        .ledger
        .findings()
        .filter(|f| f.state == FindingState::Refuted && view.ledger.merged_into(&f.id).is_none())
    {
        any = true;
        let disc: Vec<EvidenceId> = f
            .contradicts
            .iter()
            .filter(|id| {
                view.ledger
                    .evidence_by_id(id)
                    .is_some_and(|e| e.kind.class() == dla_core::EvidenceClass::Discriminator)
            })
            .cloned()
            .collect();
        let _ = writeln!(
            out,
            "- **{}** ({}): discriminator {}",
            f.id,
            label(f),
            if disc.is_empty() {
                DASH.to_owned()
            } else {
                ids(&disc)
            }
        );
        if let Some(r) = ruled_out_by(view, f) {
            let _ = writeln!(out, "  - Ruled out by: {r}");
        }
        if let Some(h) = &f.hypothesis {
            let _ = writeln!(out, "  - Hypothesis: {}", inline(h));
        }
        if let Some(r) = &f.refute_if {
            let _ = writeln!(out, "  - Refuted if: {}", inline(r));
        }
        if !f.contradicts.is_empty() {
            let _ = writeln!(out, "  - Contradicting evidence: {}", ids(&f.contradicts));
        }
        if !f.supports.is_empty() {
            let _ = writeln!(out, "  - Originally supported by: {}", ids(&f.supports));
        }
    }
    if !any {
        out.push_str("None.\n");
    }
    out.push('\n');
}

/// Entry points per worker listed one by one under "Entry points not exercisable"; the
/// rest are counted by reason (a discovery over a large service marks hundreds).
const LISTED_UNEXERCISABLE: usize = 15;

fn blind_spots(out: &mut String, view: &RunView) {
    out.push_str("## Blind spots and unknowns\n\n");

    out.push_str("### Coverage gaps\n\n");
    let mut any = false;
    if let Some(cov) = &view.coverage {
        for (worker, w) in &cov.workers {
            for (name, b) in &w.backends {
                if b.status == BackendStatus::Ran {
                    continue;
                }
                any = true;
                let mut line = format!("- {worker}/{name}: {}", b.status);
                if let Some(r) = &b.reason {
                    let _ = write!(line, " ({})", inline(r));
                }
                if let Some(h) = &b.install_hint {
                    let _ = write!(line, ". Install hint: {}", inline(h));
                }
                let _ = writeln!(out, "{line}");
            }
        }
    }
    none_if(out, any, view.coverage.is_none());

    out.push_str("### Runtime blind spots\n\n");
    let mut any = false;
    if let Some(cov) = &view.coverage {
        for b in &cov.blind_spots {
            any = true;
            let _ = writeln!(out, "- {}: {}", inline(&b.runtime), inline(&b.text));
        }
    }
    none_if(out, any, view.coverage.is_none());

    out.push_str("### Entry points not exercisable\n\n");
    let mut any = false;
    if let Some(cov) = &view.coverage {
        for (worker, w) in &cov.workers {
            for n in w
                .entry_points
                .not_exercisable
                .iter()
                .take(LISTED_UNEXERCISABLE)
            {
                any = true;
                let _ = writeln!(out, "- {worker}: {}: {}", code(&n.entry), inline(&n.reason));
            }
            let rest = w
                .entry_points
                .not_exercisable
                .iter()
                .skip(LISTED_UNEXERCISABLE)
                .fold(BTreeMap::<&str, usize>::new(), |mut m, n| {
                    *m.entry(n.reason.as_str()).or_default() += 1;
                    m
                });
            if !rest.is_empty() {
                let total: usize = rest.values().sum();
                let by: Vec<String> = rest
                    .iter()
                    .map(|(r, n)| format!("{n} x {}", inline(r)))
                    .collect();
                let _ = writeln!(
                    out,
                    "- {worker}: and {total} more not listed ({}); the full list is in `coverage.json`",
                    by.join("; ")
                );
            }
        }
    }
    none_if(out, any, view.coverage.is_none());

    out.push_str("### Evidence limitations and unknowns\n\n");
    let mut any = false;
    for e in view.ledger.evidence() {
        any |= evidence_gaps(out, e);
    }
    none_if(out, any, false);
}

/// Writes the limitations, unknowns and partial coverage of one record.
fn evidence_gaps(out: &mut String, e: &EvidenceRecord) -> bool {
    let mut any = false;
    if e.coverage.status == CoverageStatus::Partial {
        any = true;
        let extent = match (e.coverage.examined, e.coverage.total) {
            (Some(x), Some(t)) => format!(" ({x} of {t} examined)"),
            _ => String::new(),
        };
        let _ = writeln!(out, "- {}: partial coverage{extent}", e.evidence_id);
    }
    for l in &e.limitations {
        any = true;
        let _ = writeln!(out, "- {}: limitation: {}", e.evidence_id, inline(l));
    }
    for u in &e.unknowns {
        any = true;
        let _ = writeln!(
            out,
            "- {}: unknown: {} Resolve by: {}",
            e.evidence_id,
            inline(&u.question),
            inline(&u.resolve_by)
        );
    }
    any
}

fn none_if(out: &mut String, any: bool, no_manifest: bool) {
    if !any {
        out.push_str(if no_manifest {
            "No coverage manifest recorded.\n"
        } else {
            "None recorded.\n"
        });
    }
    out.push('\n');
}

fn checklist(out: &mut String, view: &RunView) {
    out.push_str("## Checklist\n\n");
    let _ = writeln!(
        out,
        "{} answered of {}.\n",
        view.checklist.iter().filter(|c| c.answer.is_some()).count(),
        plural(view.checklist.len(), "item", "items")
    );
    out.push_str("| Item | Question | Status | Evidence | Note |\n|---|---|---|---|---|\n");
    for c in &view.checklist {
        let mut status = c.status_label().to_owned();
        let mut evidence = DASH.to_owned();
        let mut note = DASH.to_owned();
        match &c.answer {
            None => status = "**MISSING**".to_owned(),
            Some(a) => {
                if !a.evidence.is_empty() {
                    evidence = ids(&a.evidence);
                }
                if let Some(n) = &a.note {
                    note = n.clone();
                }
                if let (Some(r), Some(m)) = (a.reason, &a.remedy) {
                    note = format!("{r}: {m}");
                }
            }
        }
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            code_cell(&c.item.id),
            cell(&c.item.question),
            cell(&status),
            cell(&evidence),
            cell(&note)
        );
    }
    out.push('\n');
}

fn evidence_index(out: &mut String, view: &RunView) {
    out.push_str("## Evidence index\n\n");
    let mut rows = view.ledger.evidence().peekable();
    if rows.peek().is_none() {
        out.push_str("No evidence recorded.\n");
        return;
    }
    out.push_str("| ID | Kind | Authority | Tool | Source | Raw |\n|---|---|---|---|---|---|\n");
    for e in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            e.evidence_id,
            e.kind,
            e.authority,
            cell(&format!("{} {}", e.tool.name, e.tool.version)),
            cell(&e.source.describe()),
            e.raw.as_deref().map_or_else(|| DASH.to_owned(), cell)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::{
        Ledger, Origin,
        state::{RunState, now_utc},
    };

    #[test]
    fn labelled_and_unclassified_findings_share_heading_shape() {
        let view = RunView::new(
            RunState::init("/p", None, 60, None, now_utc()).unwrap(),
            Ledger::from_text("").unwrap(),
            None,
        );
        let mk = |archetype: &str| {
            let mut f = Finding::candidate(
                "F1".parse().unwrap(),
                Origin::parse("census").unwrap(),
                vec!["E1".parse().unwrap()],
            );
            f.archetype = archetype.to_owned();
            f
        };
        let mut a = String::new();
        finding_section(&mut a, &view, &mk("A-REGISTRY"));
        let mut b = String::new();
        finding_section(&mut b, &view, &mk(UNCLASSIFIED));
        assert!(a.starts_with("### F1: **A-REGISTRY**\n"));
        assert!(b.starts_with("### F1: **UNCLASSIFIED**\n"));
        assert_eq!(a.replace("A-REGISTRY", "UNCLASSIFIED"), b);
        let mut c = mk(UNCLASSIFIED);
        c.state = FindingState::Confirmed;
        let mut t = String::new();
        finding_section(&mut t, &view, &c);
        assert!(t.starts_with("### F1: **UNCLASSIFIED**\n"));
        c.state = FindingState::Strong;
        assert_eq!(label(&c), "UNCLASSIFIED");
        for st in [
            FindingState::Candidate,
            FindingState::UnderTest,
            FindingState::Refuted,
            FindingState::Unresolved,
            FindingState::SuspectedStaticOnly,
        ] {
            c.state = st;
            assert_eq!(label(&c), "UNCLASSIFIED", "{st}");
        }
    }

    #[test]
    fn a_site_raised_by_a_rule_and_the_census_shows_both_origins_and_ranks_as_the_better() {
        let mut f = Finding::candidate(
            "F1".parse().unwrap(),
            Origin::parse("detector:dla/python-unbounded-cache").unwrap(),
            vec!["E1".parse().unwrap()],
        );
        f.state = FindingState::SuspectedStaticOnly;
        f.site = Some("a.py:3".into());
        let view = RunView::new(
            RunState::init("/p", None, 60, None, now_utc()).unwrap(),
            Ledger::from_text("").unwrap(),
            None,
        );
        // A DLA rule hit alone ranks 1; with a census entry that has no writers it still ranks 1.
        assert_eq!(static_tier(&view, &f), 1);
        f.also_origins.push(Origin::parse("census").unwrap());
        assert_eq!(static_tier(&view, &f), 1);
        let mut section = String::new();
        finding_section(&mut section, &view, &f);
        assert!(
            section.contains(
                "- Origin: `detector:dla/python-unbounded-cache`\n- Also raised by: `census`\n"
            ),
            "{section}"
        );
        assert_eq!(
            origins_cell(&f),
            "`detector:dla/python-unbounded-cache`, `census`"
        );
    }

    #[test]
    fn sibling_heading_shows_its_hypothesis() {
        let view = RunView::new(
            RunState::init("/p", None, 60, None, now_utc()).unwrap(),
            Ledger::from_text("").unwrap(),
            None,
        );
        let mut f = Finding::candidate(
            "F1.n1".parse().unwrap(),
            Origin::parse("hypothesis").unwrap(),
            vec!["E1".parse().unwrap()],
        );
        f.parent = Some("F1".parse().unwrap());
        f.hypothesis = Some("N-WARM".to_owned());
        f.refute_if = Some("flat after warm-up".to_owned());
        let mut out = String::new();
        finding_section(&mut out, &view, &f);
        assert!(
            out.starts_with("### F1.n1: Hypothesis N-WARM (UNCLASSIFIED)\n"),
            "{out}"
        );
    }

    #[test]
    fn every_count_of_the_findings_line_adds_up_for_confirmed_and_strong_leaks() {
        let mut c = dla_core::view::StatusCounts {
            enumerated: 6,
            ..Default::default()
        };
        c.by_state.confirmed = 3;
        c.by_state.strong = 3;
        // The caller leaves a disputed finding out of the released count, so with three
        // confirmed (one disputed, one released, one plain) and three strong (two released):
        let line = findings_line(&c, &["N-INTENT".to_owned()], (1, 2));
        assert!(
            line.contains("confirmed 1, confirmed but disputed by the verifier (reads N-INTENT) 1, grows only while callers skip the release: 1, strong 1, grows only while callers skip the release: 2, refuted 0"),
            "{line}"
        );
        assert!(!line.contains("no leak confirmed"), "{line}");
        // Nothing disputed: the confirmed are the plain ones and the released.
        let line = findings_line(&c, &[], (2, 3));
        assert!(
            line.contains("confirmed 1, grows only while callers skip the release: 2, strong 0, grows only while callers skip the release: 3, refuted 0"),
            "{line}"
        );
        // Everything released: no leak is confirmed.
        c.measured = true;
        let line = findings_line(&c, &[], (3, 3));
        assert!(line.ends_with("; no leak confirmed"), "{line}");
        // A disputed finding that is also released is one finding: a plain count that left it
        // in both would hide the plain one. Disputed and released counts never exceed the whole.
        let line = findings_line(&c, &["N-INTENT".to_owned()], (2, 0));
        assert!(
            line.contains("confirmed 0, confirmed but disputed by the verifier (reads N-INTENT) 1, grows only while callers skip the release: 2"),
            "{line}"
        );
    }

    #[test]
    fn open_item_descriptions_have_no_doubled_punctuation() {
        let st = ReportStatus {
            complete: false,
            blocking: vec![Missing {
                kind: "checklist_item".into(),
                id: "intake.budget".into(),
                description: "checklist item intake.budget is unanswered: How much time?".into(),
                fix: "dla ledger resolve intake.budget --status answered --evidence E1".into(),
            }],
            counts: Default::default(),
        };
        let mut out = String::new();
        open_items(&mut out, &st);
        assert!(!out.contains("?."), "{out}");
        assert!(out.contains("time? Fix:"), "{out}");
    }
}
