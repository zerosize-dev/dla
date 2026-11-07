//! `dla workload discover [--lang <l>]` → [`dla_workload::discover`].

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use dla_core::runtime::Runtime;
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res, parse::parse_enum, stage};

/// `dla workload ...`
#[derive(Subcommand, Debug)]
pub enum WorkloadCmd {
    /// Find the tests, mains and dev servers a loop can run, ranked by entry points.
    Discover(DiscoverArgs),
}

/// Flags of `dla workload discover`.
#[derive(Args, Debug)]
pub struct DiscoverArgs {
    /// Limit to one runtime: jvm | go | rust | node | python | native (default: all detected).
    #[arg(long, value_parser = parse_enum::<Runtime>)]
    lang: Option<Runtime>,
}

/// Runs `dla workload discover`.
pub fn discover(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: DiscoverArgs) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    stage::finish(
        "workload discover",
        json!({"lang": a.lang.map(|l| l.as_str())}),
        dla_workload::discover(&tc, a.lang),
    )
}
