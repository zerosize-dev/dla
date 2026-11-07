//! Census of process-lifetime holders, and the tree-sitter parsing layer.
//!
//! # Contract
//!
//! One entry point, [`census`]. It does its own writes through the [`ToolCtx`] (design
//! and shared rules: `dla_core::toolctx`) and returns a bounded [`StageOutcome`]; the
//! CLI (`census_cmd.rs`) only parses arguments and prints.
//!
//! Writes: the full list of `dla_core::census::CensusEntry` as
//! `.dla/<run>/census/<lang>.json`, ONE `census-holder` evidence record
//! (`raw` = that file), one candidate per holder (origin `census`, `holder`, `owner`,
//! `region`, `site`; `CandidateWriter` skips repeated sites), and for each holder with
//! writers or removers a `Ledger::annotate` line so the report lists them. The census
//! never reads archetype data or known-leak tables, and never sets
//! `suggested_archetype`.
//!
//! [`parse`] is the shared tree-sitter layer (`Lang`, `parse`, `captures`) also used
//! by the DLA rule backend and workload discovery.

pub mod parse;

use dla_core::{
    Refusal,
    runtime::Runtime,
    toolctx::{StageOutcome, ToolCtx, not_implemented},
};

/// Enumerates every static, global, registry, spawned unit and acquired handle of the
/// `lang` runtime under `ctx.project` and records them as candidates.
///
/// For the JVM, class files under the project are read when present and tree-sitter is
/// the fallback. Until implemented: refusal `not-implemented`.
pub fn census(ctx: &ToolCtx, lang: Runtime) -> Result<StageOutcome, Refusal> {
    let _ = (ctx, lang);
    Err(not_implemented("dla census"))
}
