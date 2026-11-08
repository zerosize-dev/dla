//! `dla bundle <F>`: a finding's label-free evidence bundle for an independent verifier.

use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::Value;

use crate::{ctx::Ctx, fail::Res};

/// Flags of `dla bundle`.
#[derive(Args, Debug)]
pub struct BundleArgs {
    /// The finding (F3, jvm/F3).
    pub finding: String,
}

/// Runs `dla bundle`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: BundleArgs) -> Res<Value> {
    let _ = (ctx, project, data, a);
    Err(dla_core::toolctx::not_implemented("dla bundle").into())
}
