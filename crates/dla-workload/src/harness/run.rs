//! `dla harness run` and the point runner `dla alloc-test` reuses (spec 4.3).
//!
//! Owned by the harness-run engineer in M3 phase 2.

use dla_core::{
    Refusal,
    harness::HarnessManifest,
    series::{KeyMode, SeriesSourceKind},
    toolctx::{StageOutcome, ToolCtx, not_implemented},
};
use serde_json::Value;

/// Options of one harness execution.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// Key modes to run; empty means the manifest's `key_modes`.
    pub modes: Vec<KeyMode>,
    /// Post-warm-up points, overriding the manifest.
    pub iterations: Option<u32>,
    /// Extra points after the post-warm-up points (alloc-test plateau window).
    pub extra: u32,
    /// Purge applied at the last point (`DLA_PURGE`).
    pub purge: Option<String>,
    /// Source kind written into the series (`harness` or `alloc-test`).
    pub source_kind: SeriesSourceKind,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            modes: Vec::new(),
            iterations: None,
            extra: 0,
            purge: None,
            source_kind: SeriesSourceKind::Harness,
        }
    }
}

/// What one execution produced.
#[derive(Debug, Clone, Default)]
pub struct RunOutput {
    /// Series IDs written, one per key mode, in mode order.
    pub series: Vec<String>,
    /// Points per series (warm-up included).
    pub points: u32,
    /// Measured processes started.
    pub processes: u32,
    /// Library detector hits (`{lib, text, site}`).
    pub detectors: Vec<Value>,
    /// Missing named tools and other gaps.
    pub gaps: Vec<String>,
    /// What this execution could not see.
    pub limitations: Vec<String>,
    /// Run-relative log paths.
    pub logs: Vec<String>,
}

/// Builds (once) and measures `manifest`, writing one series per key mode. Needs tier 1.
/// Writes no evidence: [`run`] and `dla alloc-test` add their own.
pub fn run_points(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
) -> Result<RunOutput, Refusal> {
    let _ = (ctx, manifest, opts);
    Err(not_implemented("dla harness run"))
}

/// `dla harness run <id>`: [`run_points`] plus one `workload` evidence (authority test)
/// whose result names the harness.
pub fn run(ctx: &ToolCtx, id: &str, opts: &RunOptions) -> Result<StageOutcome, Refusal> {
    let _ = (ctx, id, opts);
    Err(not_implemented("dla harness run"))
}
