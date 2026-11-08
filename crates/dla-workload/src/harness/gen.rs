//! `dla harness gen` (spec 4.2).
//!
//! Owned by the harness-gen engineer in M3 phase 2.

use dla_core::{
    FindingId, Refusal,
    runtime::Runtime,
    toolctx::{StageOutcome, ToolCtx, not_implemented},
};

/// Options of `dla harness gen`.
#[derive(Debug, Clone)]
pub struct GenOptions {
    /// Runtime of the harness.
    pub lang: Runtime,
    /// Workload ID from discovery (default: the best-ranked main or script workload with an
    /// iteration argument).
    pub workload: Option<String>,
    /// Driver mode instead of command mode.
    pub driver: bool,
    /// Iteration argument template with `{n}`, overriding detection.
    pub iter_arg: Option<String>,
    /// Units per point (command mode: n_k = step * k).
    pub step: Option<u64>,
    /// Post-warm-up points (default 8).
    pub iterations: u32,
    /// Warm-up points (default 1).
    pub warmup: u32,
    /// Findings to add a probe for.
    pub probe_findings: Vec<FindingId>,
    /// Derive probes from census candidates.
    pub auto_probes: bool,
    /// Go/JVM package (directory or name) for driver mode.
    pub package: Option<String>,
}

/// Writes a harness and its manifest; see the module docs.
pub fn generate(ctx: &ToolCtx, opts: &GenOptions) -> Result<StageOutcome, Refusal> {
    let _ = (ctx, opts);
    Err(not_implemented("dla harness gen"))
}
