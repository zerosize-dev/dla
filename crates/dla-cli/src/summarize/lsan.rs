//! `alloc-stack-diff` over LeakSanitizer text (`ASAN_OPTIONS=detect_leaks=1`, or a program
//! built with `-fsanitize=leak`): the leak report printed at exit, in `single` mode.
//!
//! LeakSanitizer scans for leaks at exit and reports direct and indirect leaks by
//! allocation stack. It is a pre-check: `use_globals=0` makes it report blocks only a
//! global still points at, so a reachable-from-a-global holder is not a leak to it.
//! Several reports of runs with growing inputs diff like memcheck's, but one report never
//! yields a passing growth column.

use dla_core::evidence::{Authority, tool_class};
use dla_parsers::native::lsan;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
    valgrind::Rebase,
};
use crate::fail::{Failure, Res};

/// Largest report read.
const MAX_BYTES: u64 = 64 << 20;

/// The LeakSanitizer text reader.
pub struct Lsan;

impl Adapter for Lsan {
    fn name(&self) -> &'static str {
        "lsan"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "native"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::LEAK_CHECKER
    }
    fn formats(&self) -> &'static str {
        "LeakSanitizer text (ERROR: LeakSanitizer: detected memory leaks)"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        h.contains("LeakSanitizer")
            || (h.contains("Direct leak of") && h.contains("allocated from"))
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let rebase = Rebase::new(&ctx.tc.project);
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let report = lsan::parse(&text).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{} holds no usable LeakSanitizer report: {}", i.shown(), e.message()),
                    "a LeakSanitizer report that ran to its summary",
                    "rerun the program with ASAN_OPTIONS=detect_leaks=1 (or LSAN_OPTIONS=report_objects=0) and save stderr",
                )
            })?;
            let mut profile = report.profile;
            rebase.profile(&mut profile);
            captures.push(ProfileCapture {
                label: i.shown().to_owned(),
                profile,
                sha256: i.sha256.clone(),
            });
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
                limitations: vec![
                    "LeakSanitizer reports blocks nothing points at (direct) or only leaked blocks point at (indirect): memory a global still holds is not reported unless use_globals=0".into(),
                    "each report is one process at exit; the reports are ordered as given".into(),
                ],
            },
        )
    }
}
