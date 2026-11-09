//! `type-histogram-diff` over the Python harness's object counts (`py-<n>.types.json`,
//! `dla harness run --capture histogram`): the instances `gc.get_objects()` finds after a
//! full collection, counted by type name with their shallow `sys.getsizeof` bytes.
//!
//! A type whose count grows at every capture is a growth column; it ties to a finding only
//! by the tie rule's type identity (never for built-in containers).

use dla_core::evidence::{Authority, tool_class};
use dla_core::runtime::Runtime;
use dla_parsers::model::{Histogram, HistogramRow};
use serde::Deserialize;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    histogram::{self, Capture, Origin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest file read.
const MAX_BYTES: u64 = 16 << 20;
/// Most rows read from one file.
const MAX_ROWS: usize = 2_000;

/// The Python type-counter reader.
pub struct PyTypes;

#[derive(Deserialize, Default)]
struct Types {
    #[serde(default)]
    dla_pytypes: Option<u32>,
    #[serde(default)]
    types: Vec<Row>,
}

#[derive(Deserialize, Default)]
struct Row {
    #[serde(default)]
    name: String,
    #[serde(default)]
    count: u64,
    #[serde(default)]
    bytes: u64,
}

impl Adapter for PyTypes {
    fn name(&self) -> &'static str {
        "py-types"
    }
    fn op(&self) -> Op {
        Op::TypeHistogramDiff
    }
    fn runtime(&self) -> &'static str {
        "python"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::HISTOGRAM
    }
    fn formats(&self) -> &'static str {
        "Python harness type counts (py-<n>.types.json, {\"dla_pytypes\":1, \"types\"})"
    }
    fn sniff(&self, input: &Input) -> bool {
        input.head_text().contains("\"dla_pytypes\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let bad = |why: String| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{}: {why}", i.shown()),
                    "py-<n>.types.json written by the Python harness",
                    "dla harness run <H> --capture histogram",
                )
            };
            let t: Types = serde_json::from_str(&text).map_err(|e| bad(e.to_string()))?;
            if t.dla_pytypes.is_none() {
                return Err(bad("no `dla_pytypes` version".to_owned()));
            }
            captures.push(Capture {
                label: i.shown().to_owned(),
                hist: Histogram {
                    source: "py-types".to_owned(),
                    timestamp_ms: None,
                    rows: t
                        .types
                        .into_iter()
                        .take(MAX_ROWS)
                        .filter(|r| !r.name.is_empty())
                        .map(|r| HistogramRow {
                            name: r.name,
                            count: r.count,
                            bytes: r.bytes,
                        })
                        .collect(),
                },
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
                runtime: Some(Runtime::Python),
                partial: true,
                order_by_time: false,
                limitations: vec![
                    "the agent lists the 300 types with the most instances, so a type outside the list at a capture has no figure there".into(),
                    "bytes are shallow sys.getsizeof sizes: what an object points to is counted under its own type".into(),
                    "objects are those the garbage collector tracks, counted after a full collection; the files are ordered as given".into(),
                ],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_parse() {
        let t: Types = serde_json::from_str(
            r#"{"dla_pytypes":1,"types":[{"name":"ingest.models.Record","count":6000,"bytes":288000}]}"#,
        )
        .unwrap();
        assert_eq!(t.types[0].count, 6000);
        assert!(t.dla_pytypes.is_some());
    }
}
