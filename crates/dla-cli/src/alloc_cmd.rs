//! `dla alloc-test`. Owned by its implementer in M3 phase 2.

use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::Value;

use crate::{ctx::Ctx, fail::Res};

/// Flags of `dla alloc-test`.
#[derive(Args, Debug)]
pub struct AllocTestArgs {}

/// Runs `dla alloc-test`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: AllocTestArgs) -> Res<Value> {
    let _ = (ctx, project, data, a);
    Err(dla_core::toolctx::not_implemented("dla alloc-test").into())
}
