//! `dla run reseal --reason`: record an audit line and seal the protected files again.
//!
//! The seals make an out-of-band edit visible. This is the one way to accept such an
//! edit: the files that no longer match their seal are named in an `integrity_resealed`
//! ledger line with the operator's reason (the report lists it), then every protected
//! file is sealed as it is now. On a run written before schema 6 it starts sealing.

use clap::Args;
use dla_core::{
    Ledger, error::codes as core_codes, fsutil::with_run_lock, integrity, view::RunView,
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
};

/// Flags of `dla run reseal`.
#[derive(Args, Debug)]
pub struct ResealArgs {
    /// Why the protected files changed outside `dla` (recorded in the ledger).
    #[arg(long)]
    pub reason: String,
}

/// `dla run reseal`.
pub fn run(ctx: &Ctx, a: ResealArgs) -> Res<Value> {
    if ctx.worker.is_some() {
        return Err(Failure::refuse(
            dla_core::slice::codes::CORE_ONLY,
            "`dla run reseal` is for the core: a worker cannot accept a change to the protected files",
            "the core (no --worker)",
            "dla run reseal --reason \"<why the files changed outside dla>\"",
        ));
    }
    let reason = a.reason.trim();
    if reason.is_empty() {
        return Err(Failure::refuse(
            core_codes::EMPTY_FIELD,
            "--reason is empty",
            "the reason the protected files changed outside dla",
            "dla run reseal --reason \"<why the files changed outside dla>\"",
        ));
    }
    let dir = &ctx.run_dir;
    let was_unsealed = !integrity::is_sealed_run(dir);
    let mut changed: Vec<String> = integrity::verify(dir).into_iter().map(|t| t.file).collect();
    if was_unsealed {
        integrity::init_dir(dir)?;
        changed = integrity::existing_protected(dir);
    }
    if changed.is_empty() {
        return Err(Failure::refuse(
            core_codes::INVALID_VALUE,
            "every protected file matches its seal: there is nothing to reseal",
            "a protected file that changed outside dla",
            "dla run status   (the `tamper` field lists the files that do not match)",
        ));
    }
    changed.sort();
    changed.dedup();
    // The audit line first, so the ledger is sealed with the line in it.
    let mut ledger = Ledger::load(dir)?;
    let line = ledger.integrity_resealed(changed.clone(), reason)?;
    let sealed = with_run_lock(dir, || {
        // A seal whose file is gone is dropped; every file that exists is sealed again.
        for t in integrity::verify(dir) {
            if t.kind == integrity::TamperKind::Missing {
                let _ = std::fs::remove_file(integrity::dir(dir).join(format!(
                    "{}{}",
                    t.file,
                    integrity::SEAL_SUFFIX
                )));
            }
        }
        integrity::seal_all(dir)
    })?;
    let view = RunView::load(dir)?;
    Ok(json!({
        "ok": true,
        "resealed": changed,
        "sealed": sealed,
        "reason": reason,
        "started_sealing": was_unsealed,
        "ledger": ctx.ledger_line(line.seq),
        "tamper": view.tamper.len(),
    }))
}
