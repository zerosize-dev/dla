//! `dla census --lang <l>` → [`dla_census::census`].

use std::path::{Path, PathBuf};

use clap::Args;
use dla_core::runtime::Runtime;
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res, parse::parse_enum, stage};

/// Flags of `dla census`.
#[derive(Args, Debug)]
pub struct CensusArgs {
    /// Runtime to enumerate: jvm | go | rust | node | python | native.
    #[arg(long, value_parser = parse_enum::<Runtime>)]
    lang: Runtime,
}

/// Runs `dla census`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: CensusArgs) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    stage::finish(
        "census",
        json!({"lang": a.lang.as_str()}),
        dla_census::census(&tc, a.lang),
    )
}
