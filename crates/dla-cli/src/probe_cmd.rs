//! `dla probe [--build]` → [`dla_probe::run`].

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use dla_core::runtime::Runtime;
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res, stage};

/// Flags of `dla probe`.
#[derive(Args, Debug)]
pub struct ProbeArgs {
    /// Also run each detected runtime's build command (needs consent tier 1).
    #[arg(long)]
    build: bool,
    /// Seconds each build command may run before it is killed.
    #[arg(long, value_name = "SECS", default_value_t = 300, requires = "build")]
    build_timeout: u64,
}

/// Runs `dla probe`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: ProbeArgs) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    let opts = dla_probe::ProbeOptions {
        build: a.build,
        build_timeout: Duration::from_secs(a.build_timeout.max(1)),
        // A worker probes only its own runtime and writes probe/<worker>.json.
        runtime: ctx
            .worker
            .as_deref()
            .and_then(|w| w.parse::<Runtime>().ok()),
        ..dla_probe::ProbeOptions::default()
    };
    stage::finish(
        "probe",
        json!({"build": a.build}),
        dla_probe::run(&tc, &opts),
    )
}
