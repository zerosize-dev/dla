//! `dla probe [--build]` → [`dla_probe::run`].

use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res, stage};

/// Flags of `dla probe`.
#[derive(Args, Debug)]
pub struct ProbeArgs {
    /// Also run each detected runtime's build command (needs consent tier 1).
    #[arg(long)]
    build: bool,
}

/// Runs `dla probe`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: ProbeArgs) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    let opts = dla_probe::ProbeOptions { build: a.build };
    stage::finish(
        "probe",
        json!({"build": a.build}),
        dla_probe::run(&tc, &opts),
    )
}
