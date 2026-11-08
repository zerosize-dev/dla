//! `dla trend`. Owned by its implementer in M3 phase 2.

use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::Value;

use crate::{ctx::Ctx, fail::Res};

/// Flags of `dla trend`.
#[derive(Args, Debug)]
pub struct TrendArgs {}

/// Runs `dla trend`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: TrendArgs) -> Res<Value> {
    let _ = (ctx, project, data, a);
    Err(dla_core::toolctx::not_implemented("dla trend").into())
}
