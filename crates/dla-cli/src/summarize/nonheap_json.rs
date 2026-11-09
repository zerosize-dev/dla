//! `non-heap` over DLA non-heap snapshot JSON (`dla_parsers::model::NonHeapSnapshot`): the
//! format the tool parsers write as their parsed output. The framework's own reader, with
//! no external tool.

use dla_core::evidence::{Authority, tool_class};
use dla_parsers::model::NonHeapSnapshot;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    input::Input,
    non_heap::{self, SnapCapture, SnapOrigin},
};
use crate::fail::{Failure, Res};

/// Largest snapshot file read.
const MAX_BYTES: u64 = 64 << 20;

/// The NonHeapSnapshot JSON reader.
pub struct NonHeapJson;

impl Adapter for NonHeapJson {
    fn name(&self) -> &'static str {
        "dla-nonheap-json"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "any"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::RUNTIME_COUNTER
    }
    fn formats(&self) -> &'static str {
        "DLA non-heap snapshot JSON ({\"source\", \"metrics\": [...], \"groups\": [...]})"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        h.trim_start().starts_with('{') && h.contains("\"metrics\"") && h.contains("\"groups\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let snap: NonHeapSnapshot = serde_json::from_str(&text).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{} is not a non-heap snapshot: {e}", i.shown()),
                    "non-heap snapshot JSON with `source`, `metrics` and `groups`",
                    "write the snapshot with the tool's parser, or take it again",
                )
            })?;
            captures.push(SnapCapture {
                label: i.shown().to_owned(),
                snap,
                sha256: i.sha256.clone(),
            });
        }
        non_heap::run_diff(
            ctx,
            opts,
            captures,
            &SnapOrigin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Test,
                runtime: None,
                order_by_time: true,
                limitations: vec![
                    "read from parsed snapshot JSON: the figures are the producing tool's".into(),
                ],
            },
        )
    }
}
