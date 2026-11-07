//! Static analysis backends: `dla sweep` and `dla install`.
//!
//! # Contract
//!
//! Two entry points, [`sweep`] and [`install`]. Both do their own writes through the
//! [`ToolCtx`] (design and shared rules: `dla_core::toolctx`) and return a bounded
//! [`StageOutcome`]; the CLI (`sweep_cmd.rs`, `install_cmd.rs`) parses arguments,
//! builds the context and prints the outcome.
//!
//! [`sweep`] reads `<data dir>/detectors/<lang>.toml` through
//! `dla_core::detectors::DetectorFile::parse` (which refuses with file and line), runs
//! every backend the table names for the language, and writes per backend: raw output
//! `.dla/<run>/sweep/<backend>.json`, ONE `static-hit` evidence record, one candidate
//! per `(rule, site)` with origin `detector:<backend>/<rule>` (use
//! `dla_core::toolctx::CandidateWriter`), and the backend's entry in `coverage.json`
//! (`missing` + install hint, `blocked` + reason, or `ran` with `enumerated` = candidates
//! added). The always-available `dla` rule backend runs over the tree-sitter parsers of
//! `dla-census`.

use dla_core::{
    Refusal,
    runtime::Runtime,
    toolctx::{StageOutcome, ToolCtx, not_implemented},
};

/// Runs every static backend the detector table of `lang` names and turns hits into
/// candidates in the ledger.
///
/// Tools run with caches under `ctx.tmp_dir()`, offline flags where they exist, a
/// timeout, and never write into the project. The summary in [`StageOutcome::summary`]
/// lists, per backend, `ran` / `blocked` / `missing` and the number of candidates.
/// Refuses with `invalid-data-file` (file and line) when the table is invalid and with
/// `no-data-dir` when the data files cannot be found; until implemented, with
/// `not-implemented`.
pub fn sweep(ctx: &ToolCtx, lang: Runtime) -> Result<StageOutcome, Refusal> {
    let _ = (ctx, lang);
    Err(not_implemented("dla sweep"))
}

/// Installs one backend (`backend` is one of `dla_core::detectors::KNOWN_BACKENDS`, the
/// CLI has already checked that).
///
/// Requires consent tier 1 (`ctx.require_tier(1, ..)`). Self-contained analyzers go into
/// `ctx.tools_dir()`; language tools go through the project's package manager into the
/// tools directory; project files are never touched. On success the outcome's
/// `summary` carries `{"backend", "version"}`. On failure the call returns a [`Refusal`]
/// with code `tool-failed` whose `fix` is the exact command for the user, and the
/// backend stays `missing` in coverage. Until implemented: `not-implemented`.
pub fn install(ctx: &ToolCtx, backend: &str) -> Result<StageOutcome, Refusal> {
    let _ = (ctx, backend);
    Err(not_implemented("dla install"))
}
