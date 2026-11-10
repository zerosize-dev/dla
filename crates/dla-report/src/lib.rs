//! Report rendering for DLA.
//!
//! [`render_markdown`] turns a [`RunView`] into the Markdown report. It is a pure
//! function of the view: it reads no clock and no file, so the same input yields
//! byte-identical output. The completeness gate itself lives in `dla-core`
//! ([`dla_core::status`]); this crate only formats it.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use dla_core::{
    EvidenceId, EvidenceRecord, Finding, FindingState, RunView, UNCLASSIFIED,
    coverage::{BackendStatus, CoverageManifest, ResidualRow},
    discriminator::favors_of,
    evidence::CoverageStatus,
    finding::Polarity,
    machine::is_passing_growth,
    status,
    trend::{GrowthSummary, PassedColumn},
};

/// Placeholder for an absent value in a table.
const DASH: &str = "\u{2014}";

/// Renders the run as a Markdown report. Deterministic: the same input yields
/// byte-identical output.
pub fn render_markdown(view: &RunView) -> String {
    let st = status(view);
    let mut out = String::new();

    out.push_str("# DLA report\n\n");
    let _ = writeln!(out, "Run: {}\n", code(&view.state.run_id));

    header(&mut out, view, &st);
    if !st.complete {
        open_items(&mut out, &st);
    }
    integrity(&mut out, view);
    workers(&mut out, view);
    findings(&mut out, view);
    merged_duplicates(&mut out, view);
    ruled_out(&mut out, view);
    blind_spots(&mut out, view);
    checklist(&mut out, view);
    evidence_index(&mut out, view);
    out
}

// ---------------------------------------------------------------- helpers

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

fn header(out: &mut String, view: &RunView, st: &dla_core::ReportStatus) {
    let c = &st.counts;
    if st.complete {
        out.push_str("Status: COMPLETE\n\n");
    } else {
        let _ = writeln!(out, "Status: INCOMPLETE ({} open)\n", st.blocking.len());
    }
    let _ = writeln!(out, "Entry: {}\n", entry_line(view));
    let _ = writeln!(out, "Scope: {}\n", scope_line(view));
    let s = &c.by_state;
    let _ = writeln!(
        out,
        "Findings: {} enumerated, {} dispositioned; confirmed {}, strong {}, refuted {}, suspected {}, unresolved {}\n",
        c.enumerated,
        c.dispositioned,
        s.confirmed,
        s.strong,
        s.refuted,
        s.suspected_static_only,
        s.unresolved
    );
    let _ = writeln!(out, "Residual: {}\n", residual_line(view.coverage.as_ref()));
    let _ = writeln!(out, "Coverage: {}\n", coverage_line(view.coverage.as_ref()));
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
    format!("{head}; excluded: {excluded}; out of scope: {out_of_scope}")
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
            format!(
                "{} {iterations}, {exercised} of {discovered} entry points exercised",
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

fn residual_line(coverage: Option<&CoverageManifest>) -> String {
    let Some(cov) = coverage else {
        return "not measured".to_owned();
    };
    if cov.residual.is_empty() {
        return "not measured".to_owned();
    }
    cov.residual
        .iter()
        .map(residual_row)
        .collect::<Vec<_>>()
        .join("; ")
}

fn residual_row(r: &ResidualRow) -> String {
    let mut s = format!(
        "{} {} {} unattributed of {} total (threshold {})",
        r.region, r.residual, r.unit, r.total_slope, r.threshold
    );
    if r.residual > r.threshold {
        s.push_str(" ABOVE THRESHOLD");
        if let Some(o) = &r.open_item {
            let _ = write!(s, ", see {o}");
        }
    }
    s
}

fn open_items(out: &mut String, st: &dla_core::ReportStatus) {
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

fn findings(out: &mut String, view: &RunView) {
    out.push_str("## Findings\n\n");
    let mut list: Vec<&Finding> = view
        .ledger
        .findings()
        .filter(|f| f.state != FindingState::Refuted && view.ledger.merged_into(&f.id).is_none())
        .collect();
    list.sort_by_key(|f| rank(f.state));
    if list.is_empty() {
        out.push_str("No findings to report.\n\n");
        return;
    }
    for f in list {
        finding_section(out, view, f);
    }
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
        if let Some(c) = summary
            .passed
            .iter()
            .find(|c| c.tied_to_owner(&f.id, f.site.as_deref(), f.owner.as_deref()))
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
    if let Some(l) = label_line(view, f) {
        let _ = writeln!(out, "- {l}");
    }
    if let Some(g) = growth_line(view, f) {
        let _ = writeln!(out, "- Growth: {g}");
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
        if !a.writers.is_empty() {
            let _ = writeln!(out, "- Writers: {}", sites(&a.writers));
        }
        if !a.removers.is_empty() {
            let _ = writeln!(out, "- Removers: {}", sites(&a.removers));
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
        let _ = writeln!(out, "- Verified: no");
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

/// Findings folded into another worker's finding with the same site and holder.
fn merged_duplicates(out: &mut String, view: &RunView) {
    let mut rows: Vec<String> = Vec::new();
    for f in view.ledger.findings() {
        if let Some(c) = view.ledger.merged_into(&f.id) {
            rows.push(format!(
                "- **{}** merged into **{c}** (same site and holder; its evidence {} is kept on {c})",
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
            for n in &w.entry_points.not_exercisable {
                any = true;
                let _ = writeln!(out, "- {worker}: {}: {}", code(&n.entry), inline(&n.reason));
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
    fn open_item_descriptions_have_no_doubled_punctuation() {
        let st = dla_core::ReportStatus {
            complete: false,
            blocking: vec![dla_core::view::BlockingItem {
                kind: dla_core::view::BlockingKind::ChecklistItem,
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
