//! `non-heap` over Go goroutines, read with DLA's own parsers (no Go toolchain):
//!
//! * the `debug=2` text of `pprof.Lookup("goroutine")` (also what a crash prints): goroutines
//!   are grouped by the `created by` frame. That frame is where the goroutine was made, so a
//!   group ties to the finding whose site (or recorded writer) is that `go` statement.
//! * the goroutine profile protobuf (`WriteTo(f, 0)`): goroutines are grouped by the frame
//!   they are blocked in now. That is not a creation site, so these groups never tie.
//!
//! The wait reason of a goroutine (`select`, `chan receive`) is a group of its own and
//! never ties. Label values and call arguments are not read.

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
};
use dla_parsers::{
    go::{dump, goroutine, heap},
    model::NonHeapSnapshot,
};

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    go_pprof_heap::{load, root_file, sniff_profile},
    input::Input,
    non_heap::{self, SnapCapture, SnapOrigin},
};
use crate::fail::{Failure, Res};

/// Largest dump text read.
const MAX_BYTES: u64 = 128 << 20;

/// True when the file is the goroutineleak profile: named for it, or with its own header.
pub fn is_leak_profile(input: &Input) -> bool {
    input.name_lower().contains("goroutineleak") || dump::has_leak_header(&input.head_text())
}

fn is_dump(input: &Input) -> bool {
    !is_leak_profile(input)
        && !input.head.starts_with(&[0x1f, 0x8b])
        && dump::looks_like_dump(&input.head_text())
}

fn is_profile(input: &Input) -> bool {
    sniff_profile(input, |p| {
        goroutine::is_goroutine_profile(p) && !heap::is_heap_profile(p)
    })
}

/// Gives the creation sites of a snapshot a project path where the file sits at the root.
fn root_sites(ctx: &Ctx<'_>, snap: &mut NonHeapSnapshot) {
    for g in &mut snap.groups {
        if let Some(f) = g.site.as_mut()
            && let Some(file) = &f.file
        {
            f.file = Some(root_file(&ctx.tc.project, file));
        }
    }
}

/// The goroutine reader.
pub struct GoGoroutine;

impl Adapter for GoGoroutine {
    fn name(&self) -> &'static str {
        "go-goroutine"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "go"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::RUNTIME_COUNTER
    }
    fn formats(&self) -> &'static str {
        "Go goroutine dump text (pprof.Lookup(\"goroutine\").WriteTo(f, 2)) or goroutine profile (WriteTo(f, 0), .pb.gz)"
    }
    fn sniff(&self, input: &Input) -> bool {
        is_dump(input) || is_profile(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let dumps = inputs.iter().filter(|i| is_dump(i)).count();
        if dumps != 0 && dumps != inputs.len() {
            return Err(Failure::refuse(
                "mixed-formats",
                "the inputs mix goroutine dump text and goroutine profiles: the text groups by creation site, the profile by blocked frame".to_owned(),
                "inputs of one goroutine format per command",
                "run one `dla summarize non-heap <files>` for the .txt dumps and one for the .pb.gz profiles",
            ));
        }
        let mut snaps = Vec::new();
        for i in inputs {
            let mut snap = if dumps > 0 {
                let text = i.read_text(MAX_BYTES)?;
                let d = dump::parse(&text).map_err(|e| {
                    Failure::refuse(
                        "malformed-artifact",
                        format!("{}: {}", i.shown(), e.message()),
                        "the text pprof.Lookup(\"goroutine\").WriteTo(f, 2) writes",
                        "dla harness run --capture nonheap   and pass the goroutine-<n>.goroutine.txt files it writes",
                    )
                })?;
                goroutine::dump_snapshot(&d)
            } else {
                let p = load(
                    i,
                    "the file pprof.Lookup(\"goroutine\").WriteTo(f, 0) writes",
                    "dla harness run --capture nonheap   and pass the goroutine profiles it writes",
                )?;
                goroutine::profile_snapshot(&p).map_err(|e| {
                    Failure::refuse(
                        "malformed-artifact",
                        format!("{}: {}", i.shown(), e.message()),
                        "a goroutine profile",
                        "pprof.Lookup(\"goroutine\").WriteTo(f, 0)",
                    )
                })?
            };
            root_sites(ctx, &mut snap);
            snaps.push((i.shown().to_owned(), snap, i.sha256.clone()));
        }
        let by_site = dumps > 0;
        let captures = snaps
            .into_iter()
            .map(|(label, snap, sha256)| SnapCapture {
                label,
                snap,
                sha256,
            })
            .collect();
        non_heap::run_diff(
            ctx,
            opts,
            captures,
            &SnapOrigin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Live,
                runtime: Some(Runtime::Go),
                order_by_time: true,
                limitations: if by_site {
                    vec![
                        "goroutines are grouped by the `created by` frame: a group ties to a finding only when that go statement is the finding's site or a recorded writer".into(),
                        "goroutines without a creator (main, runtime) are not grouped; wait reasons and totals never tie".into(),
                    ]
                } else {
                    vec![
                        "a goroutine profile (protobuf) holds the stack a goroutine is blocked in now, not where it was created: its groups never tie to a finding by site; use the debug=2 text".into(),
                    ]
                },
            },
        )
    }
}
