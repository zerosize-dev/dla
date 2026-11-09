//! `type-histogram-diff` over `jcmd <pid> GC.class_histogram` text files (from
//! `dla attach jcmd --op histogram`, or saved by hand).

use dla_core::evidence::{Authority, tool_class};
use dla_core::runtime::Runtime;
use dla_parsers::model::{Histogram, HistogramRow};
use dla_sample::attach::parse_histogram;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    histogram::{self, Capture, Origin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest histogram text read.
const MAX_BYTES: u64 = 64 << 20;

/// The jcmd class histogram text reader.
pub struct JcmdClassHistogram;

impl Adapter for JcmdClassHistogram {
    fn name(&self) -> &'static str {
        "jcmd-class-histogram"
    }
    fn op(&self) -> Op {
        Op::TypeHistogramDiff
    }
    fn runtime(&self) -> &'static str {
        "jvm"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::HISTOGRAM
    }
    fn formats(&self) -> &'static str {
        "jcmd GC.class_histogram text (\" num #instances #bytes class name\")"
    }
    fn sniff(&self, input: &Input) -> bool {
        let head = input.head_text();
        head.contains("#instances") && head.contains("class name")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let Some(h) = parse_histogram(&text) else {
                return Err(Failure::refuse(
                    "malformed-artifact",
                    format!("{} holds no class histogram rows", i.shown()),
                    "a complete GC.class_histogram output",
                    "dla attach jcmd --pid <pid> --op histogram   and pass the file it writes",
                ));
            };
            let hist = Histogram {
                source: "jcmd-class-histogram".into(),
                timestamp_ms: None,
                rows: h
                    .rows
                    .into_iter()
                    .map(|r| HistogramRow {
                        name: r.class,
                        count: r.instances,
                        bytes: r.bytes,
                    })
                    .collect(),
            };
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
                runtime: Some(Runtime::Jvm),
                partial: false,
                order_by_time: false,
                limitations: vec![
                    "a class histogram counts live and not yet collected objects; take captures after a full collection (the attach histogram does) to compare them".into(),
                    "the files carry no time, so they are ordered as given".into(),
                ],
            },
        )
    }
}
