//! `dla summarize heap-graph <file>...`: heap dumps into retention and growth evidence.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde_json::Value;

use crate::{ctx::Ctx, fail::Res};

/// `dla summarize ...`
#[derive(Subcommand, Debug)]
pub enum SummarizeCmd {
    /// Summarize HPROF heap dumps (`.hprof`, `.hprof.gz`); two or more form a dump series.
    HeapGraph(HeapGraphArgs),
}

/// Flags of `dla summarize heap-graph`.
#[derive(Args, Debug)]
pub struct HeapGraphArgs {
    /// Dump files, relative to the project root or absolute.
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
}

/// Runs one `dla summarize ...` subcommand.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: SummarizeCmd) -> Res<Value> {
    let _ = (ctx, project, data, a);
    Err(dla_core::toolctx::not_implemented("dla summarize heap-graph").into())
}
