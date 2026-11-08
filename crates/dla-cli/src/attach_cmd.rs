//! `dla attach <backend> --pid <P> --op <op>`: attach a tool to a process DLA did not start.

use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::Value;

use crate::{ctx::Ctx, fail::Res};

/// Flags of `dla attach`.
#[derive(Args, Debug)]
pub struct AttachArgs {
    /// Backend: jcmd, node-inspector, memray, heaptrack, goref.
    pub backend: String,
    /// Target process ID.
    #[arg(long)]
    pub pid: u32,
    /// Operation, for example histogram or heap-dump.
    #[arg(long)]
    pub op: String,
}

/// Runs `dla attach`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: AttachArgs) -> Res<Value> {
    let _ = (ctx, project, data, a);
    Err(dla_core::toolctx::not_implemented("dla attach").into())
}
