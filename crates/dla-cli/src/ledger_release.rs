//! The command-layer checks of `N-RELEASE` (see `dla_core::release`): a confirm held by drivers
//! that disagree or by an explanation that names a conditional release nobody ran, and the
//! evidence an `N-RELEASE` sibling is confirmed or refuted on. They read the run directory
//! (harness manifests, line counts), which the ledger rules never do.

use dla_core::{
    EvidenceId, FindingId, RunView,
    discriminator::Favors,
    evidence::EvidenceRecord,
    finding::{FindingState, Polarity},
    release,
};
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res};

fn view_of(ctx: &Ctx) -> Option<RunView> {
    RunView::load_for(&ctx.run_dir, ctx.worker.as_deref()).ok()
}

/// Refuses `dla ledger confirm <F>` of a leak-side finding while two drivers made for it
/// disagree (`drivers-disagree`) or its explanation names a conditional remover that no run with
/// the release called has decided (`release-untested`).
pub fn refuse_held_confirm(ctx: &Ctx, id: &FindingId) -> Res<()> {
    let Some(view) = view_of(ctx) else {
        return Ok(());
    };
    let Some(f) = view.ledger.finding(id) else {
        return Ok(());
    };
    // A finding that cannot move to confirmed (refuted is final) is left to the ledger's own refusal.
    if f.polarity() != Polarity::Leak
        || !dla_core::machine::is_allowed(f.state, FindingState::Confirmed)
    {
        return Ok(());
    }
    let root = f.root_id();
    if let Some(d) = release::drivers_disagree(&view, &root) {
        return Err(release::drivers_disagree_refusal(&view, id, &d).into());
    }
    // The explanation of the finding itself or of its root names the remover.
    let explained = [Some(f), view.ledger.finding(&root)]
        .into_iter()
        .flatten()
        .find_map(|x| release::release_untested(&view.ledger, x));
    if let Some(u) = explained {
        return Err(release::release_untested_refusal(&view, &u).into());
    }
    Ok(())
}

/// Judges the evidence of `dla ledger confirm` or `refute` of an `N-RELEASE` sibling: an
/// alloc-test on a driver made for the finding in which the remover line ran. Returns the note
/// to print when the runtime could not count the line.
pub fn judge_release_move(
    ctx: &Ctx,
    id: &FindingId,
    evidence: &[EvidenceId],
    to: FindingState,
) -> Res<Option<String>> {
    if !matches!(to, FindingState::Confirmed | FindingState::Refuted) {
        return Ok(None);
    }
    let Some(view) = view_of(ctx) else {
        return Ok(None);
    };
    let Some(s) = view.ledger.finding(id).filter(|f| release::is_release(f)) else {
        return Ok(None);
    };
    // What the move judges: what the sibling already cites plus what the move cites.
    let own = if to == FindingState::Confirmed {
        &s.supports
    } else {
        &s.contradicts
    };
    let cited: Vec<&EvidenceRecord> = own
        .iter()
        .chain(evidence)
        .filter_map(|e| view.ledger.evidence_by_id(e))
        .collect();
    let want = if to == FindingState::Confirmed {
        Favors::NonLeak
    } else {
        Favors::Leak
    };
    let d = release::check_release_decision(&view, s, &cited, want)?;
    Ok(d.note)
}

/// What `dla ledger list` adds to the row of an `N-RELEASE` sibling: its remover and, when the
/// runtime could not count the line the decision rests on, the note `remover run not shown`.
pub fn list_facts(view: &RunView, id: &FindingId) -> Option<Value> {
    let s = view.ledger.finding(id).filter(|f| release::is_release(f))?;
    let remover = s.remover.clone()?;
    let mut v = json!({"remover": remover});
    if let Some(n) = release::decision_note(view, s) {
        v["note"] = Value::String(n);
    }
    Some(v)
}
