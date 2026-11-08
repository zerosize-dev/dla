//! Probe derivation from census candidates and top-level name scans (spec 4.6).
//!
//! Owned by the harness-gen engineer in M3 phase 2.

use dla_core::{FindingId, harness::HarnessManifest, toolctx::ToolCtx};

/// The site and finding of a probe result path reported by an agent (for example
/// `shop.OrderCache.ORDERS_BY_ID`, `src/scheduler.js:shared.latest`,
/// `ingest.registry._LISTENERS`): the census entry whose owner matches, else the
/// top-level declaration of that name, else nothing.
pub fn resolve_site(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    probe_path: &str,
) -> (Option<String>, Option<FindingId>) {
    let _ = (ctx, manifest, probe_path);
    (None, None)
}
