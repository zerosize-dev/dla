//! `type-histogram-diff` over harness dump files (`harness/H<n>/out/<pid>.jsonl`): the
//! `types` rows (top types by count and bytes) of each checkpoint, exit or tick line.

use dla_core::{
    HarnessDump,
    evidence::{Authority, tool_class},
    runtime::Runtime,
};
use dla_parsers::model::{Histogram, HistogramRow};

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    histogram::{self, Capture, Origin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest dump file read.
const MAX_BYTES: u64 = 64 << 20;

/// The harness `types` reader.
pub struct HarnessTypes;

fn first_line(input: &Input) -> String {
    input
        .head_text()
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
        .to_owned()
}

impl Adapter for HarnessTypes {
    fn name(&self) -> &'static str {
        "harness-types"
    }
    fn op(&self) -> Op {
        Op::TypeHistogramDiff
    }
    fn runtime(&self) -> &'static str {
        "any"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::HISTOGRAM
    }
    fn formats(&self) -> &'static str {
        "harness dump jsonl (harness/H<n>/out/<pid>.jsonl) with `types` rows"
    }
    fn sniff(&self, input: &Input) -> bool {
        let l = first_line(input);
        l.starts_with('{') && l.contains("\"dla_harness\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        // The runtime the dumps name (every harness writes `runtime`), so the `needed`
        // text matches it; the first dump with types that names a parsable runtime decides, none
        // leaves it unset.
        let mut runtime: Option<Runtime> = None;
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let (dumps, _bad) = HarnessDump::parse_lines(&text);
            let mut k = 0;
            for d in dumps {
                let Some(types) = d.types.filter(|t| !t.is_empty()) else {
                    continue;
                };
                k += 1;
                if runtime.is_none() {
                    runtime = d.runtime.as_deref().and_then(|r| r.parse().ok());
                }
                captures.push(Capture {
                    label: format!("{}#{k}", i.shown()),
                    hist: Histogram {
                        source: "harness-types".into(),
                        // Times are relative to the process start, so the lines of one
                        // file are in order already; a second file would not compare.
                        timestamp_ms: None,
                        rows: types
                            .into_iter()
                            .filter_map(|t| {
                                Some(HistogramRow {
                                    name: t.name?,
                                    count: t.count.map_or(0, |c| c.max(0.0) as u64),
                                    bytes: t.bytes.map_or(0, |b| b.max(0.0) as u64),
                                })
                            })
                            .collect(),
                    },
                    sha256: i.sha256.clone(),
                });
            }
        }
        if captures.is_empty() {
            return Err(Failure::refuse(
                "malformed-artifact",
                "the harness dump files hold no `types` rows".to_owned(),
                "dump files from a harness run that captured types",
                "dla harness run <H> --capture histogram   (harnesses write types at each checkpoint)",
            ));
        }
        histogram::run_diff(
            ctx,
            opts,
            captures,
            &Origin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Test,
                runtime,
                partial: true,
                order_by_time: false,
                limitations: vec![
                    "the harness lists only the top types, so a type outside the list at a checkpoint has no figure there".into(),
                    "captures are the dump lines in file order".into(),
                ],
            },
        )
    }
}
