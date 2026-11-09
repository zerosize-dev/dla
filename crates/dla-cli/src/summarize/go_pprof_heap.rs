//! Go heap profiles (`pprof.Lookup("heap").WriteTo(f, 0)`, gzipped protobuf), read with
//! DLA's own pprof reader and no Go toolchain:
//!
//! * `go-pprof-heap` (`alloc-stack-diff`): the in-use bytes and objects of every
//!   allocation stack. A stack whose first project frame is a finding's site or recorded
//!   writer ties to it (rule 2, site identity); a stack never names a holder.
//! * `go-pprof-heap-functions` (`type-histogram-diff`): one row per allocating function.
//!   Rows are functions, not types, so they never tie by type; the diff still shows what
//!   grows.
//!
//! The in-use figures are what the last completed collection found live, so write the
//! profile right after `runtime.GC()` (the harness does). The profile is a sample taken
//! with `runtime.MemProfileRate`; its sizes are estimates. Profiles from one process
//! are ordered by the time inside each file.

use std::fs;

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
    series::SeriesRegion,
};
use dla_parsers::go::{heap, pprof};

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    histogram::{self, Capture, Origin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest profile file read.
pub const MAX_BYTES: u64 = 64 << 20;
/// Largest file the sniffer decodes to decide.
const SNIFF_BYTES: u64 = 32 << 20;

/// Reads and decodes a profile file, refusing one that is too big or damaged.
pub fn load(input: &Input, expected: &str, fix: &str) -> Result<pprof::Profile, Failure> {
    if input.size > MAX_BYTES {
        return Err(Failure::refuse(
            "insufficient-resources",
            format!(
                "{} is {} bytes, over the {MAX_BYTES}-byte limit for a pprof profile",
                input.shown(),
                input.size
            ),
            "a profile of a plausible size",
            "write the profile again with a lower runtime.MemProfileRate cost, or fewer goroutines",
        ));
    }
    let bytes =
        fs::read(&input.abs).map_err(|e| Failure::from(dla_core::Error::io(&input.abs, e)))?;
    pprof::read(&bytes).map_err(|e| {
        Failure::refuse(
            "malformed-artifact",
            format!("{}: {}", input.shown(), e.message()),
            expected,
            fix,
        )
    })
}

/// Gives a bare file name (`main.go`, a file at the project root) the project's directory,
/// so the site resolver can tell it is a project file.
pub fn root_file(project: &std::path::Path, file: &str) -> String {
    if !file.contains('/') && !file.is_empty() && project.join(file).is_file() {
        return project.join(file).to_string_lossy().into_owned();
    }
    file.to_owned()
}

/// True when the file decodes as a pprof profile and `pred` accepts it.
pub fn sniff_profile(input: &Input, pred: impl Fn(&pprof::Profile) -> bool) -> bool {
    // The inflater writes in chunks, so the 8 KB head of the input is not always enough to
    // see the start of a compressed profile: read the file (bounded), look at its first
    // bytes cheaply, and decode only what looks like a profile.
    if input.size > SNIFF_BYTES || input.head.len() < 2 {
        return false;
    }
    let Ok(bytes) = fs::read(&input.abs) else {
        return false;
    };
    pprof::looks_like_profile(&bytes) && pprof::read(&bytes).is_ok_and(|p| pred(&p))
}

const EXPECTED: &str =
    "the file pprof.Lookup(\"heap\").WriteTo(f, 0) writes (gzipped protobuf with inuse_space)";
const FIX: &str = "write the heap profile after runtime.GC() with pprof.Lookup(\"heap\").WriteTo(f, 0) (dla harness run --capture alloc does), and pass that file";

fn limitations() -> Vec<String> {
    vec![
        "a Go heap profile is a sample (runtime.MemProfileRate): sizes are estimates, and objects smaller than the rate can be missed".into(),
        "in-use figures are as of the last completed collection: write the profile after runtime.GC()".into(),
        "a profile carries no holder: a stack ties to a finding only through its first project frame being the finding's site or writer".into(),
    ]
}

/// The heap profile reader for `alloc-stack-diff`.
pub struct GoPprofHeap;

impl Adapter for GoPprofHeap {
    fn name(&self) -> &'static str {
        "go-pprof-heap"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "go"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        "Go heap profile (pprof.Lookup(\"heap\").WriteTo(f, 0), .pb.gz)"
    }
    fn sniff(&self, input: &Input) -> bool {
        sniff_profile(input, heap::is_heap_profile)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let p = load(i, EXPECTED, FIX)?;
            let profile = heap::to_alloc_profile(&p).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{}: {}", i.shown(), e.message()),
                    EXPECTED,
                    FIX,
                )
            })?;
            let mut profile = profile;
            for g in &mut profile.groups {
                for f in &mut g.frames {
                    if let Some(file) = &f.file {
                        f.file = Some(root_file(&ctx.tc.project, file));
                    }
                }
            }
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
                runtime: Some(Runtime::Go),
                region: SeriesRegion::Heap,
                order_by_time: true,
                limitations: limitations(),
            },
        )
    }
}

/// The heap profile reader for `type-histogram-diff`, by allocating function.
pub struct GoPprofHeapFunctions;

impl Adapter for GoPprofHeapFunctions {
    fn name(&self) -> &'static str {
        "go-pprof-heap-functions"
    }
    fn op(&self) -> Op {
        Op::TypeHistogramDiff
    }
    fn runtime(&self) -> &'static str {
        "go"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::HISTOGRAM
    }
    fn formats(&self) -> &'static str {
        "Go heap profile (pprof.Lookup(\"heap\").WriteTo(f, 0), .pb.gz), as a histogram by allocating function"
    }
    fn sniff(&self, input: &Input) -> bool {
        sniff_profile(input, heap::is_heap_profile)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let p = load(i, EXPECTED, FIX)?;
            let hist = heap::to_histogram(&p).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{}: {}", i.shown(), e.message()),
                    EXPECTED,
                    FIX,
                )
            })?;
            captures.push(Capture {
                label: i.shown().to_owned(),
                hist,
                sha256: i.sha256.clone(),
            });
        }
        histogram::run_diff(
            ctx,
            opts,
            captures,
            &Origin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Live,
                runtime: Some(Runtime::Go),
                partial: false,
                order_by_time: true,
                limitations: vec![
                    "rows are the first project function of each allocation stack, not types: they never tie to a finding by type".into(),
                    "a Go heap profile is a sample (runtime.MemProfileRate) taken at the last collection: sizes and counts are estimates".into(),
                ],
            },
        )
    }
}
