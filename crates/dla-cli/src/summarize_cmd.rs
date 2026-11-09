//! `dla summarize <op> [--backend <name>|auto] [--finding F] <inputs...>`, and
//! `dla summarize --list`.
//!
//! This file is the command line and the dispatch only. The framework (the adapter trait,
//! the registry, selection, the bounded output) and the adapters are in `summarize/`.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use dla_core::FindingId;
use dla_parsers::hprof::DEFAULT_MAX_OBJECTS;
use serde_json::Value;

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
    stage,
    summarize::{self, Choice, Env, Op, Request},
};

/// Default rows in a ranking.
const DEFAULT_TOP: usize = 5;

/// `dla summarize ...`
#[derive(Args, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub struct SummarizeArgs {
    /// Print the adapters: operation, runtime, backend, tool and whether the tool is
    /// available here.
    #[arg(long)]
    pub list: bool,
    #[command(subcommand)]
    pub op: Option<SummarizeCmd>,
}

/// The four operations.
#[derive(Subcommand, Debug)]
pub enum SummarizeCmd {
    /// Retention paths and holder growth from heap dumps (`.hprof`, `.hprof.gz`); two or
    /// more form a dump series.
    HeapGraph(OpArgs),
    /// Type or class histograms across 2 or more captures (jcmd class histogram text,
    /// heap dumps, harness dump files).
    TypeHistogramDiff(OpArgs),
    /// Allocation stacks across 1 or more profiles.
    AllocStackDiff(OpArgs),
    /// Threads, descriptors, class loaders and native memory across captures.
    NonHeap(OpArgs),
}

/// Flags of every `dla summarize <op>`.
#[derive(Args, Debug)]
pub struct OpArgs {
    /// Input files of one process, relative to the project root, absolute, or under
    /// `.dla/<run>/` (`attach/P<n>/...`, `harness/H<n>/out/...`). Several are ordered by
    /// their own timestamps when they have them, else as given.
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
    /// The adapter: a name from `dla summarize --list`, or `auto` to pick by the format
    /// of the inputs (DLA's own parsers first).
    #[arg(long, default_value = "auto")]
    pub backend: Choice,
    /// Narrow the result to this finding: for heap-graph the discriminator binds to the
    /// holder the finding's owner names; for the other operations only a growth column
    /// that already ties to the finding by holder, site or type is tied to it. It ties
    /// nothing by itself.
    #[arg(long, value_parser = parse_enum::<FindingId>)]
    pub finding: Option<FindingId>,
    /// Analyze at most this many objects per dump (heap-graph); above it the summary is
    /// truncated.
    #[arg(long, default_value_t = DEFAULT_MAX_OBJECTS)]
    pub max_objects: usize,
    /// Rows shown in a ranking.
    #[arg(long, default_value_t = DEFAULT_TOP)]
    pub top: usize,
}

impl SummarizeCmd {
    fn parts(self) -> (Op, OpArgs) {
        match self {
            Self::HeapGraph(a) => (Op::HeapGraph, a),
            Self::TypeHistogramDiff(a) => (Op::TypeHistogramDiff, a),
            Self::AllocStackDiff(a) => (Op::AllocStackDiff, a),
            Self::NonHeap(a) => (Op::NonHeap, a),
        }
    }
}

/// Runs `dla summarize`. `ctx` resolves the run lazily: `--list` needs none.
pub fn run(
    ctx: impl FnOnce() -> Res<Ctx>,
    project: &Path,
    data: Option<&PathBuf>,
    a: SummarizeArgs,
) -> Res<Value> {
    let env = Env::current();
    if a.list {
        let tools = dla_core::paths::tools_dir();
        return Ok(summarize::list(tools.as_deref(), &env));
    }
    let Some(cmd) = a.op else {
        let ops = Op::ALL.map(Op::name).join(", ");
        return Err(Failure::usage(
            format!("`dla summarize` needs an operation or --list; operations: {ops}"),
            format!("an operation: {ops}"),
            "dla summarize --list",
        ));
    };
    let (op, args) = cmd.parts();
    let ctx = ctx()?;
    let tc = stage::tool_ctx(project, &ctx, data);
    summarize::execute(
        &tc,
        Request {
            op,
            choice: args.backend,
            finding: args.finding,
            files: args.files,
            max_objects: args.max_objects,
            top: args.top,
        },
        &env,
    )
}
