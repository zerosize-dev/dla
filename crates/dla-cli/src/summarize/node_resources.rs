//! `non-heap` over the Node harness's `resources.json`: handles and other resources by
//! type, and by the project line that created them.
//!
//! `types.<T>.sites` counts the resources of type `T` that were created at a project
//! line and not yet destroyed. Each site is a group whose creation site is that line, so
//! its column ties to the finding at that line or among its writers (the tie rule's rule 2).
//! The counts of `getActiveResourcesInfo` are runtime totals and tie to nothing.

use std::collections::BTreeMap;

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
};
use dla_parsers::model::{Frame, NonHeapMetric, NonHeapSnapshot, ResourceGroup};
use serde::Deserialize;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    input::Input,
    non_heap::{self, SnapCapture, SnapOrigin},
};
use crate::fail::{Failure, Res};

/// Largest file read.
const MAX_BYTES: u64 = 16 << 20;
/// The source the snapshot reports.
const SOURCE: &str = "node-resources";

/// The Node resources reader.
pub struct NodeResources;

#[derive(Deserialize, Default)]
struct Resources {
    #[serde(default)]
    dla_resources: Option<u32>,
    #[serde(default)]
    active: BTreeMap<String, u64>,
    #[serde(default)]
    types: BTreeMap<String, TypeRow>,
}

#[derive(Deserialize, Default)]
struct TypeRow {
    #[serde(default)]
    tracked: u64,
    #[serde(default)]
    sites: BTreeMap<String, u64>,
    #[serde(default)]
    unattributed: u64,
}

/// `file:line` into a frame; a site that does not end in a line number has no frame.
fn site_frame(site: &str) -> Option<Frame> {
    let (file, line) = site.rsplit_once(':')?;
    let line: u32 = line.parse().ok()?;
    (!file.is_empty() && line > 0).then(|| Frame {
        function: String::new(),
        file: Some(file.replace('\\', "/")),
        line: Some(line),
        module: None,
    })
}

fn snapshot(r: &Resources) -> NonHeapSnapshot {
    let mut metrics = Vec::new();
    for (t, n) in &r.active {
        metrics.push(NonHeapMetric {
            category: "node.active".to_owned(),
            name: t.clone(),
            value: i64::try_from(*n).unwrap_or(i64::MAX),
            unit: "count".to_owned(),
        });
    }
    let mut groups = Vec::new();
    for (t, row) in &r.types {
        metrics.push(NonHeapMetric {
            category: "node.tracked".to_owned(),
            name: t.clone(),
            value: i64::try_from(row.tracked).unwrap_or(i64::MAX),
            unit: "count".to_owned(),
        });
        for (site, n) in &row.sites {
            groups.push(ResourceGroup {
                kind: "handle".to_owned(),
                key: format!("{t}@{site}"),
                count: *n,
                site: site_frame(site),
            });
        }
        if row.unattributed > 0 {
            groups.push(ResourceGroup {
                kind: "handle".to_owned(),
                key: format!("{t}@(unattributed)"),
                count: row.unattributed,
                site: None,
            });
        }
    }
    NonHeapSnapshot {
        source: SOURCE.to_owned(),
        timestamp_ms: None,
        metrics,
        groups,
    }
}

impl Adapter for NodeResources {
    fn name(&self) -> &'static str {
        "node-resources"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "node"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::RUNTIME_COUNTER
    }
    fn formats(&self) -> &'static str {
        "Node harness resources.json ({\"dla_resources\":1, \"active\", \"types\"})"
    }
    fn sniff(&self, input: &Input) -> bool {
        input.head_text().contains("\"dla_resources\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let r: Resources = serde_json::from_str(&text).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{} is not a resources file: {e}", i.shown()),
                    "resources.json written by the Node harness",
                    "dla harness run <H> --capture nonheap",
                )
            })?;
            if r.dla_resources.is_none() {
                return Err(Failure::refuse(
                    "malformed-artifact",
                    format!("{} has no `dla_resources` version", i.shown()),
                    "resources.json written by the Node harness",
                    "dla harness run <H> --capture nonheap",
                ));
            }
            captures.push(SnapCapture {
                label: i.shown().to_owned(),
                snap: snapshot(&r),
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
                authority: Authority::Live,
                runtime: Some(Runtime::Node),
                order_by_time: false,
                limitations: vec![
                    "sites are the first project frame of the stack that created the resource; a resource with no project frame is unattributed".into(),
                    "active counts come from getActiveResourcesInfo and leave out unref'd timers".into(),
                    "the files are ordered as given; each is one point".into(),
                ],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sites_become_creation_frames_and_totals_stay_metrics() {
        let r: Resources = serde_json::from_str(
            r#"{"dla_resources":1,"active":{"TCPSocketWrap":2},
                "types":{"Timeout":{"tracked":30,"sites":{"src/heartbeat.js:15":30},"unattributed":0},
                         "TCPSocketWrap":{"tracked":2,"sites":{"src/conn.js:3":1},"unattributed":1}}}"#,
        )
        .unwrap();
        let s = snapshot(&r);
        assert!(
            s.metrics
                .iter()
                .any(|m| m.category == "node.active" && m.value == 2)
        );
        let g = s
            .groups
            .iter()
            .find(|g| g.key == "Timeout@src/heartbeat.js:15")
            .unwrap();
        assert_eq!(g.count, 30);
        let f = g.site.as_ref().unwrap();
        assert_eq!(
            (f.file.as_deref(), f.line),
            (Some("src/heartbeat.js"), Some(15))
        );
        let u = s
            .groups
            .iter()
            .find(|g| g.key == "TCPSocketWrap@(unattributed)")
            .unwrap();
        assert!(u.site.is_none());
        assert!(site_frame("no-line").is_none());
        assert!(site_frame("a.js:0").is_none());
    }
}
