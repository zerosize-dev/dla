//! `dla workload discover`: what a loop can run.
//!
//! # Contract
//!
//! One entry point, [`discover`]. It does its own writes through the [`ToolCtx`]
//! (design and shared rules: `dla_core::toolctx`) and returns a bounded
//! [`StageOutcome`]; the CLI (`workload_cmd.rs`) only parses arguments and prints.
//!
//! Writes: `.dla/<run>/workload/workloads.json` (a `dla_core::workload::WorkloadReport`),
//! ONE `workload` evidence record (`raw` = `workload/workloads.json`), and in
//! `coverage.json` for each runtime `entry_points.discovered` and
//! `entry_points.not_exercisable` (`exercised` stays empty until the M3 harness runs).
//! It adds no candidates. A runtime whose `dla probe --build` failed has every workload
//! marked `not_exercisable` with reason `build_fails`; entry points no workload
//! references get `no_harness`.

use dla_core::{
    Refusal,
    runtime::Runtime,
    toolctx::{StageOutcome, ToolCtx, not_implemented},
};

/// Finds tests, mains and dev servers of the project (all runtimes when `lang` is
/// `None`), ranks them by the entry points they exercise and records the result.
///
/// Builds only after consent tier 1; a failed build is recorded as a coverage gap, not
/// an error. Until implemented: refusal `not-implemented`.
pub fn discover(ctx: &ToolCtx, lang: Option<Runtime>) -> Result<StageOutcome, Refusal> {
    let _ = (ctx, lang);
    Err(not_implemented("dla workload discover"))
}
