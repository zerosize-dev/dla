//! `dla harness`. Owned by its implementer in M3 phase 2.

use std::path::{Path, PathBuf};

use clap::Subcommand;
use serde_json::Value;

use crate::{ctx::Ctx, fail::Res};

/// `dla harness ...`
#[derive(Subcommand, Debug)]
pub enum HarnessCmd {
    /// Write a loop harness for a workload into the run directory.
    Gen(GenArgs),
    /// Build and run a harness, writing one series per key mode.
    Run(RunArgs),
}

/// Flags of `dla harness gen`.
#[derive(clap::Args, Debug)]
pub struct GenArgs {}

/// Flags of `dla harness run`.
#[derive(clap::Args, Debug)]
pub struct RunArgs {}

/// Runs `dla harness`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: HarnessCmd) -> Res<Value> {
    let _ = (ctx, project, data, a);
    Err(dla_core::toolctx::not_implemented("dla harness").into())
}
