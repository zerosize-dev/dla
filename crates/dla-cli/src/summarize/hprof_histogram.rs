//! `type-histogram-diff` over HPROF heap dumps: the class histogram the DLA reader derives
//! (the largest classes by shallow bytes), one capture per dump.

use std::fs;

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
};
use dla_parsers::{
    hprof::{self, HeapSummary, Options},
    model::{Histogram, HistogramRow},
};

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    histogram::{self, Capture, Origin},
    hprof::{is_hprof, summary_name},
    input::Input,
};
use crate::fail::{Failure, Res};

/// The class histogram of heap dumps.
pub struct HprofHistogram;

/// A summary an earlier `dla summarize heap-graph` of this run wrote for the same dump.
fn cached(ctx: &Ctx<'_>, i: &Input) -> Option<HeapSummary> {
    let p = ctx
        .tc
        .run_dir
        .join("summaries")
        .join(summary_name(&i.rel, &i.sha256));
    serde_json::from_str(&fs::read_to_string(p).ok()?).ok()
}

impl Adapter for HprofHistogram {
    fn name(&self) -> &'static str {
        "dla-hprof-histogram"
    }
    fn op(&self) -> Op {
        Op::TypeHistogramDiff
    }
    fn runtime(&self) -> &'static str {
        "jvm"
    }
    fn tool_class(&self) -> &'static str {
        // The same reader on the same dump as heap-graph: one signal.
        tool_class::DUMP_PARSER
    }
    fn formats(&self) -> &'static str {
        ".hprof and .hprof.gz heap dumps"
    }
    fn sniff(&self, input: &Input) -> bool {
        is_hprof(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let tmp = ctx.tc.tmp_dir()?;
        let parse_opts = Options {
            tmp_dir: Some(tmp),
            max_objects: opts.max_objects,
            ..Options::default()
        };
        let mut captures = Vec::new();
        for i in inputs {
            let summary = match cached(ctx, i) {
                Some(s) => s,
                None => hprof::summarize(&i.abs, &parse_opts).map_err(|e| {
                    Failure::refuse(
                        "malformed-artifact",
                        format!("{} cannot be read: {e}", i.shown()),
                        "a complete heap dump",
                        "take the dump again (jcmd <pid> GC.heap_dump <file>) and rerun",
                    )
                })?,
            };
            captures.push(Capture {
                label: i.shown().to_owned(),
                hist: Histogram {
                    source: "hprof".into(),
                    timestamp_ms: Some(summary.header.timestamp_ms),
                    rows: summary
                        .top_classes
                        .iter()
                        .map(|c| HistogramRow {
                            name: c.class.clone(),
                            count: c.count,
                            bytes: c.shallow,
                        })
                        .collect(),
                },
                sha256: i.sha256.clone(),
            });
            ctx.tc.require_time("dla summarize type-histogram-diff")?;
        }
        histogram::run_diff(
            ctx,
            opts,
            captures,
            &Origin {
                tool: "dla-hprof",
                class: self.tool_class(),
                authority: Authority::Dump,
                runtime: Some(Runtime::Jvm),
                partial: true,
                order_by_time: true,
                limitations: vec![
                    "bytes are approximate shallow sizes (HPROF records no object sizes)".into(),
                    "only the 25 classes with the most shallow bytes are listed per dump".into(),
                ],
            },
        )
    }
}
