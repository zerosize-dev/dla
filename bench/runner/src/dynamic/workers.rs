//! Multi-runtime fixtures: one worker per runtime, then `ledger merge`.
//!
//! The core creates the run and runs the probe. When the fixture declares two or more
//! runtimes it registers one worker per runtime with `dla run dispatch`, the fixed policy of
//! [`super`] then runs once per runtime with `--worker <runtime>` on every command (the
//! workers run one after the other here; a real session runs them in parallel), each worker
//! ends its slice with `ledger done`, and the core merges the slices and reports.
//!
//! The acceptance of the milestone is checked after the merge: every worker's slice merged
//! (`merged_lines` above zero), `coverage.json` has an entry for every runtime and the merge
//! left no worker unfinished.

use std::fs;

use serde::Serialize;
use serde_json::Value;

use super::{Fx, analyse, finish, make_harnesses, strings};

/// One worker of a multi-runtime fixture, as the merge saw it.
#[derive(Serialize, Debug, Clone, Default)]
pub struct WorkerRow {
    /// Worker name (the runtime).
    pub name: String,
    /// Harnesses the worker ran.
    pub harnesses: Vec<String>,
    /// Slice lines the merge appended to the main ledger.
    pub merged_lines: u64,
}

/// The merge outcome of a multi-runtime fixture.
#[derive(Serialize, Debug, Clone, Default)]
pub struct MergeRow {
    /// `status` the merge printed.
    pub status: String,
    /// Runtimes with an entry in `coverage.json`.
    pub coverage_runtimes: Vec<String>,
    /// Duplicate findings the merge joined.
    pub duplicates: usize,
    /// Contradictions the merge opened.
    pub contradictions: u64,
    /// Workers the merge found unfinished.
    pub unfinished: Vec<String>,
}

/// Runs the workers and the merge. `builds` is the probe's build result per runtime.
pub(super) fn drive(fx: &mut Fx, runtimes: &[String], builds: &Value) {
    let declared = fx.plan.runtimes;
    if runtimes.len() < declared {
        fx.row.failures.push(format!(
            "the fixture declares {declared} runtimes but the probe found {}: {}",
            runtimes.len(),
            runtimes.join(",")
        ));
    }
    if runtimes.len() < 2 {
        return;
    }
    let list = runtimes.join(",");
    let dispatch = fx.call(&["run", "dispatch", "--workers", &list]);
    if !dispatch.ok {
        fx.row
            .failures
            .push("run dispatch failed: no worker was registered".to_owned());
        return;
    }
    fx.step(format!("dispatched workers {list}"));
    let mut workers: Vec<WorkerRow> = Vec::new();
    for rt in runtimes {
        fx.worker = Some(rt.clone());
        fx.call(&["sweep", "--lang", rt]);
        fx.call(&["census", "--lang", rt]);
        fx.call(&["workload", "discover"]);
        let (ran, missing) = make_harnesses(fx, std::slice::from_ref(rt), builds);
        let ids: Vec<String> = ran.iter().map(|r| r.harness.clone()).collect();
        fx.row.harnesses.extend(ids.iter().cloned());
        fx.row
            .series
            .extend(ran.iter().flat_map(|r| r.series.clone()));
        if !ran.is_empty() {
            analyse(fx, &ran);
        }
        finish(fx, &ran, if ran.is_empty() { missing } else { None }, false);
        let done = fx.call(&["ledger", "done"]);
        fx.step(format!(
            "worker {rt}: {} harness(es), slice {}",
            ids.len(),
            if done.ok { "done" } else { "not closed" }
        ));
        fx.worker = None;
        workers.push(WorkerRow {
            name: rt.clone(),
            harnesses: ids,
            merged_lines: 0,
        });
    }
    let merge = fx.call(&["ledger", "merge"]);
    let mut row = MergeRow {
        status: merge.json["status"].as_str().unwrap_or("").to_owned(),
        duplicates: merge.json["duplicates"].as_array().map_or(0, Vec::len),
        contradictions: merge.json["contradictions"].as_u64().unwrap_or(0),
        unfinished: strings(&merge.json["unfinished"]),
        ..MergeRow::default()
    };
    if !merge.ok {
        fx.row
            .failures
            .push(format!("ledger merge failed: {}", merge.message()));
    }
    for w in &mut workers {
        w.merged_lines = merge.json["merged_lines"][w.name.as_str()]
            .as_u64()
            .unwrap_or(0);
        if merge.ok && w.merged_lines == 0 {
            fx.row.failures.push(format!(
                "ledger merge: worker {} merged no line into the main ledger",
                w.name
            ));
        }
    }
    if !row.unfinished.is_empty() {
        fx.row.failures.push(format!(
            "ledger merge: worker(s) not finished: {}",
            row.unfinished.join(",")
        ));
    }
    if let Ok(run_dir) = fx.run_dir() {
        let cov = fs::read_to_string(run_dir.join("coverage.json"))
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .unwrap_or(Value::Null);
        let mut found: Vec<String> = cov["workers"]
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| v["runtime"].as_str().unwrap_or(k).to_owned())
                    .collect()
            })
            .unwrap_or_default();
        found.sort();
        found.dedup();
        for rt in runtimes {
            if !found.contains(rt) {
                fx.row
                    .failures
                    .push(format!("coverage.json has no entry for runtime {rt}"));
            }
        }
        row.coverage_runtimes = found;
    }
    fx.step(format!(
        "merged: {}; status {}; {} duplicate(s), {} contradiction(s); coverage {}",
        workers
            .iter()
            .map(|w| format!("{}={}", w.name, w.merged_lines))
            .collect::<Vec<_>>()
            .join(", "),
        row.status,
        row.duplicates,
        row.contradictions,
        row.coverage_runtimes.join(",")
    ));
    fx.row.workers = workers;
    fx.row.merge = Some(row);
    fx.call(&["report"]);
}
