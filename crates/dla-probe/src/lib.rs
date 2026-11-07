//! `dla probe`: what runtimes, tools, privileges, test runners and libraries the project has.
//!
//! # Contract
//!
//! The crate exposes ONE entry point, [`run`]. It does its own writes through the
//! [`ToolCtx`] (see `dla_core::toolctx` for the design and the rules every stage
//! follows) and returns a bounded [`StageOutcome`]. The CLI (`dla-cli`,
//! `probe_cmd.rs`) only parses arguments, builds the context and prints the outcome.
//!
//! What [`run`] must write:
//!
//! * `.dla/<run>/probe/probe.json`: the full `dla_core::probe::ProbeReport`;
//! * one evidence record of kind `probe` (bounded summary, `raw` = `probe/probe.json`);
//! * `coverage.json` via `CoverageManifest::update`: for every detected runtime
//!   (worker key = the runtime name) each registry backend as `missing` (with the
//!   install hint) or, when found, recorded as available (status `ran`, `version` set,
//!   `enumerated` 0) until a sweep updates it. Entry points are not its business.
//!
//! Probe adds no candidates.

use dla_core::{
    Refusal,
    runtime::Runtime,
    toolctx::{StageOutcome, ToolCtx, not_implemented},
};

/// Options of `dla probe`.
#[derive(Debug, Clone, Default)]
pub struct ProbeOptions {
    /// `--build`: also run each detected runtime's build command. Requires consent tier
    /// 1 (`ctx.require_tier(1, ..)`, whose fix is `dla consent --tiers 1-4`); without
    /// it the call refuses and writes nothing.
    pub build: bool,
}

/// Probes the project at `ctx.project`.
///
/// Detects runtimes from build files and sources, reads toolchain versions (each with a
/// timeout; failures are recorded, never fatal), finds every registry backend on PATH
/// or in `ctx.tools_dir()`, collects privilege facts, test runners and the library
/// harness flags, and optionally builds. Writes the files listed in the crate docs and
/// returns the evidence ID in [`StageOutcome::evidence`] and a summary
/// (runtimes, tools found/missing, build results) in [`StageOutcome::summary`].
///
/// Returns a [`Refusal`] when `opts.build` is set without consent tier 1, or (until
/// implemented) with code `not-implemented`.
pub fn run(ctx: &ToolCtx, opts: &ProbeOptions) -> Result<StageOutcome, Refusal> {
    let _ = (ctx, opts, Runtime::ALL);
    Err(not_implemented("dla probe"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stub_refuses_with_a_stable_code() {
        let ctx = ToolCtx {
            project: ".".into(),
            run_dir: ".".into(),
            run_id: "r".into(),
            source_commit: "c".into(),
            data_flag: None,
        };
        let e = run(&ctx, &ProbeOptions::default()).unwrap_err();
        assert_eq!(e.code, dla_core::error::codes::NOT_IMPLEMENTED);
    }
}
