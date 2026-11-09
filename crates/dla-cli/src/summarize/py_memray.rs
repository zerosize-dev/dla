//! `alloc-stack-diff` over the Python harness's memray report (`py-<n>.memray.json`,
//! `dla harness run --capture alloc` with memray installed in the target's interpreter).
//!
//! memray's own file is never read here: the helper `dla_memray_leaks.py` runs with the
//! target's interpreter and writes the allocations still live when the tracker stopped, per
//! stack, as bounded JSON. The report is the state at the end of the run, so one file is a
//! leak-at-exit view and files of several runs of growing size give the series. A stack
//! ties by its first project frame, like every stack without a holder.

use std::path::Path;

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
    series::SeriesRegion,
};
use dla_parsers::model::{AllocGroup, AllocProfile};
use serde::Deserialize;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
    py_tracemalloc::frame,
};
use crate::fail::{Failure, Res};

/// Largest file read.
const MAX_BYTES: u64 = 32 << 20;
/// Most stacks read from one file.
const MAX_STACKS: usize = 2_000;
/// Most frames read from one stack.
const MAX_FRAMES: usize = 32;

/// The memray report reader.
pub struct PyMemray;

#[derive(Deserialize, Default)]
struct Report {
    #[serde(default)]
    dla_memray: Option<u32>,
    #[serde(default)]
    stacks: Vec<Stack>,
}

#[derive(Deserialize, Default)]
struct Stack {
    #[serde(default)]
    size: u64,
    #[serde(default)]
    n: u64,
    #[serde(default)]
    allocator: Option<String>,
    /// `[function, file, line]`, innermost first.
    #[serde(default)]
    frames: Vec<(String, String, u32)>,
}

fn parse(text: &str, shown: &str, project: &Path) -> Res<AllocProfile> {
    let bad = |why: String| {
        Failure::refuse(
            "malformed-artifact",
            format!("{shown}: {why}"),
            "py-<n>.memray.json written by the Python harness",
            "dla harness run <H> --capture alloc   (with memray installed in the target's interpreter)",
        )
    };
    let r: Report = serde_json::from_str(text).map_err(|e| bad(e.to_string()))?;
    if r.dla_memray.is_none() {
        return Err(bad("no `dla_memray` version".to_owned()));
    }
    let groups = r
        .stacks
        .into_iter()
        .take(MAX_STACKS)
        .map(|s| AllocGroup {
            frames: s
                .frames
                .iter()
                .take(MAX_FRAMES)
                .map(|(function, file, line)| frame(project, function, file, *line))
                .collect(),
            live_bytes: s.size,
            live_blocks: s.n,
            total_bytes: None,
            root_path: Vec::new(),
            kind: s.allocator,
        })
        .collect();
    Ok(AllocProfile {
        source: "py-memray".to_owned(),
        timestamp_ms: None,
        sampled: false,
        groups,
    })
}

impl Adapter for PyMemray {
    fn name(&self) -> &'static str {
        "py-memray"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "python"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        "Python harness memray report (py-<n>.memray.json, {\"dla_memray\":1, \"stacks\"})"
    }
    fn sniff(&self, input: &Input) -> bool {
        input.head_text().contains("\"dla_memray\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            captures.push(ProfileCapture {
                label: i.shown().to_owned(),
                profile: parse(&text, i.shown(), &ctx.tc.project)?,
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
                runtime: Some(Runtime::Python),
                region: SeriesRegion::Heap,
                order_by_time: false,
                limitations: vec![
                    "memray reports the allocations still live when its tracker stopped at the end of the run, so a file is a leak-at-exit view; a series needs files from runs of growing size".into(),
                    "memray traces the Python stack only (no native frames); allocations the agent made after the tracker stopped are not in it".into(),
                    "the report keeps the largest stacks by live bytes".into(),
                    "the files are ordered as given".into(),
                ],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_keep_function_names_and_allocators() {
        let text = r#"{"dla_memray":1,"stacks":[
            {"size":4810048,"n":14981,"allocator":"malloc","frames":[
              ["__init__","ingest/watchers.py",8],["process_batch","ingest/pipeline.py",27]]}]}"#;
        let p = parse(text, "x", Path::new("/proj")).unwrap();
        let g = &p.groups[0];
        assert_eq!((g.live_bytes, g.live_blocks), (4_810_048, 14_981));
        assert_eq!(g.frames[0].function, "ingest.watchers.__init__");
        assert_eq!(g.frames[1].file.as_deref(), Some("ingest/pipeline.py"));
        assert_eq!(g.kind.as_deref(), Some("malloc"));
    }

    #[test]
    fn a_tracemalloc_file_is_not_a_report() {
        assert!(parse(r#"{"dla_pyalloc":1}"#, "x", Path::new("/p")).is_err());
    }
}
