//! `alloc-stack-diff` over DHAT JSON: `valgrind --tool=dhat --dhat-out-file=<file>`, or the
//! file the `dhat` crate's `Profiler` writes. The live figures are those at the end of the
//! run (`eb`, `ebk`); `tb` is the total allocated, which is churn and never a growth
//! column. A second signal next to memcheck or heaptrack, where `dhat-rs` is present.

use dla_core::evidence::{Authority, tool_class};
use dla_parsers::native::dhat;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
    valgrind::Rebase,
};
use crate::fail::{Failure, Res};

/// Largest file read.
const MAX_BYTES: u64 = 64 << 20;

/// The DHAT JSON reader.
pub struct Dhat;

impl Adapter for Dhat {
    fn name(&self) -> &'static str {
        "dhat"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "native"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        "DHAT JSON (valgrind --tool=dhat --dhat-out-file, or dhat-rs; dhatFileVersion 1 or 2)"
    }
    fn sniff(&self, input: &Input) -> bool {
        input.head_text().contains("\"dhatFileVersion\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let rebase = Rebase::new(&ctx.tc.project);
        let mut captures = Vec::new();
        let mut modes: Vec<String> = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let report = dhat::parse(&text).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{} is not a DHAT file: {}", i.shown(), e.message()),
                    "the JSON file DHAT writes at exit",
                    "valgrind --tool=dhat --dhat-out-file=<file> <program>   or run the program with dhat::Profiler",
                )
            })?;
            if !modes.contains(&report.mode) {
                modes.push(report.mode.clone());
            }
            let mut profile = report.profile;
            rebase.profile(&mut profile);
            captures.push(ProfileCapture {
                label: i.shown().to_owned(),
                profile,
                sha256: i.sha256.clone(),
            });
        }
        let mut limitations = vec![
            "live figures are the bytes and blocks still allocated at the end of the run (eb, ebk); bytes allocated over the run are churn, not retention".into(),
            "each file is one process; the files are ordered as given".into(),
        ];
        if modes.iter().any(|m| m != "heap" && m != "rust-heap") {
            limitations.push(format!(
                "DHAT mode {} does not record live bytes at exit: only heap profiles do",
                modes.join(", ")
            ));
        }
        alloc_stack::run_diff(
            ctx,
            opts,
            captures,
            &ProfileOrigin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Live,
                runtime: None,
                region: dla_core::series::SeriesRegion::OffHeap,
                order_by_time: false,
                windowed: false,
                limitations,
            },
        )
    }
}
