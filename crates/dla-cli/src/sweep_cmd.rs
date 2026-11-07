//! `dla sweep --lang <l>` → [`dla_backends::sweep`].

use std::path::{Path, PathBuf};

use clap::Args;
use dla_core::runtime::Runtime;
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res, parse::parse_enum, stage};

/// Flags of `dla sweep`.
#[derive(Args, Debug)]
pub struct SweepArgs {
    /// Runtime to sweep: jvm | go | rust | node | python | native.
    #[arg(long, value_parser = parse_enum::<Runtime>)]
    lang: Runtime,
}

/// Runs `dla sweep`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: SweepArgs) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    stage::finish(
        "sweep",
        json!({"lang": a.lang.as_str()}),
        dla_backends::sweep(&tc, a.lang),
    )
}
