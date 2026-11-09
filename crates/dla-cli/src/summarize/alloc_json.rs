//! `alloc-stack-diff` over DLA allocation profile JSON (`dla_parsers::model::AllocProfile`):
//! the format the shim and the tool parsers write as their parsed output. It is the
//! framework's own reader, with no external tool.

use dla_core::evidence::{Authority, tool_class};
use dla_parsers::model::AllocProfile;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest profile file read.
const MAX_BYTES: u64 = 64 << 20;

/// The AllocProfile JSON reader.
pub struct AllocJson;

impl Adapter for AllocJson {
    fn name(&self) -> &'static str {
        "dla-alloc-json"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "any"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        "DLA allocation profile JSON ({\"source\", \"sampled\", \"groups\": [...]})"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        h.trim_start().starts_with('{') && h.contains("\"groups\"") && h.contains("\"sampled\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let profile: AllocProfile = serde_json::from_str(&text).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{} is not an allocation profile: {e}", i.shown()),
                    "allocation profile JSON with `source`, `sampled` and `groups`",
                    "write the profile with the tool's parser, or take it again",
                )
            })?;
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
                authority: Authority::Test,
                runtime: None,
                region: dla_core::series::SeriesRegion::Heap,
                order_by_time: true,
                windowed: false,
                limitations: vec![
                    "read from parsed profile JSON: the figures are the producing tool's".into(),
                ],
            },
        )
    }
}
