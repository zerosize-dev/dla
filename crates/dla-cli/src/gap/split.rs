//! `dla gap <S>`: whole-process memory split into runtime-accounted and unexplained.

use dla_core::{
    EvidenceId, RunState,
    evidence::{CoverageStatus, EvidenceCoverage, EvidenceKind},
    runtime::Runtime,
    series::{
        Column, Point, SERIES_SCHEMA_VERSION, SeriesFile, SeriesSource, SeriesSourceKind,
        column_meta,
    },
    toolctx::{EvidenceSpec, ToolCtx},
    trend::{TREND_DIR, TREND_SCHEMA_VERSION, TrendReport},
};
use dla_stats::{GateThresholds, GateVerdict};
use serde_json::{Value, json};

use super::{GAP_UNAVAILABLE, fit, series_authority};
use crate::fail::{Failure, Res};

/// The RSS column the split subtracts from.
const RSS: &str = "os.rss";

/// How the runtime-accounted figure is built from series columns.
struct Plan {
    /// `(column index, sign)` terms of the sum.
    terms: Vec<(usize, f64)>,
    /// The formula, for the output.
    label: String,
    /// A limitation the formula implies.
    limitation: Option<String>,
}

fn unavailable(series: &SeriesFile, runtime: &str, missing: &[&str]) -> Failure {
    let h = series.source.harness.as_deref().unwrap_or("<H>");
    Failure::refuse(
        GAP_UNAVAILABLE,
        format!(
            "series {} ({runtime}) has no {} column, so runtime-accounted memory cannot be computed",
            series.id,
            missing.join(" or ")
        ),
        format!("column {} in {}", missing.join(", "), series.id),
        format!(
            "dla harness run {h}   (a runtime agent or driver writes it; command mode without an agent records only os.* columns)"
        ),
    )
}

fn plan(series: &SeriesFile) -> Res<Plan> {
    let Some(runtime) = series.source.runtime else {
        return Err(Failure::refuse(
            GAP_UNAVAILABLE,
            format!(
                "series {} records no runtime, so the accounted columns are unknown",
                series.id
            ),
            "source.runtime in the series",
            "dla harness run <H>   (series written by a harness carry the runtime)",
        ));
    };
    let rt = runtime.as_str();
    let has = |name: &str| series.column_index(name);
    let has_values =
        |name: &str| has(name).is_some_and(|i| series.values(i).iter().any(Option::is_some));
    let need = |names: &[&str]| -> Res<Vec<usize>> {
        names
            .iter()
            .map(|n| has(n).ok_or_else(|| unavailable(series, rt, names)))
            .collect()
    };
    let p = match runtime {
        Runtime::Jvm if has_values("jvm.nmt_committed") => Plan {
            terms: need(&["jvm.nmt_committed"])?.into_iter().map(|i| (i, 1.0)).collect(),
            label: "jvm.nmt_committed".into(),
            limitation: None,
        },
        Runtime::Jvm => Plan {
            terms: need(&["jvm.heap_committed", "jvm.nonheap_committed"])?
                .into_iter()
                .map(|i| (i, 1.0))
                .collect(),
            label: "jvm.heap_committed + jvm.nonheap_committed".into(),
            limitation: Some(
                "NMT off: accounted is jvm.heap_committed + jvm.nonheap_committed, which misses thread stacks, code cache and GC structures"
                    .into(),
            ),
        },
        Runtime::Node => Plan {
            terms: need(&["node.heap_total", "node.external"])?
                .into_iter()
                .map(|i| (i, 1.0))
                .collect(),
            label: "node.heap_total + node.external".into(),
            limitation: None,
        },
        Runtime::Python => Plan {
            terms: need(&["py.traced"])?.into_iter().map(|i| (i, 1.0)).collect(),
            label: "py.traced".into(),
            limitation: Some(
                "py.traced counts only memory tracemalloc sees (Python allocations), not C extensions"
                    .into(),
            ),
        },
        Runtime::Go => {
            let ix = need(&["go.total", "go.released"])?;
            Plan {
                terms: vec![(ix[0], 1.0), (ix[1], -1.0)],
                label: "go.total - go.released".into(),
                limitation: None,
            }
        }
        Runtime::Rust | Runtime::Native => Plan {
            terms: need(&["native.live_bytes"])?.into_iter().map(|i| (i, 1.0)).collect(),
            label: "native.live_bytes".into(),
            limitation: None,
        },
    };
    Ok(p)
}

fn derived_column(name: &str) -> Column {
    let m = column_meta(name);
    Column {
        name: name.to_owned(),
        unit: m.unit,
        kind: m.kind,
        region: m.region,
        baseline: m.baseline,
        after_gc: m.after_gc,
        scope: m.scope,
        site: None,
        finding: None,
        source: "gap".to_owned(),
    }
}

/// The derived series of `src`: `gap.accounted` and `gap.unexplained` per point.
fn derive(id: String, src: &SeriesFile) -> Res<(SeriesFile, String)> {
    let rss = src.column_index(RSS).ok_or_else(|| {
        let h = src.source.harness.as_deref().unwrap_or("<H>");
        Failure::refuse(
            GAP_UNAVAILABLE,
            format!(
                "series {} has no {RSS} column to subtract the accounted memory from",
                src.id
            ),
            format!("column {RSS} in {}", src.id),
            format!("dla harness run {h}   (the harness samples the OS columns for every point)"),
        )
    })?;
    let plan = plan(src)?;
    let mut negative = 0u32;
    let points: Vec<Point> = src
        .points
        .iter()
        .map(|p| {
            let get = |i: usize| p.values.get(i).copied().flatten();
            let accounted = plan
                .terms
                .iter()
                .map(|&(i, sign)| get(i).map(|v| v * sign))
                .sum::<Option<f64>>();
            let unexplained = accounted.zip(get(rss)).map(|(a, r)| r - a);
            if unexplained.is_some_and(|u| u < 0.0) {
                negative += 1;
            }
            Point {
                x: p.x,
                t_ms: p.t_ms,
                values: vec![accounted, unexplained],
            }
        })
        .collect();
    let mut limitations = src.limitations.clone();
    limitations.extend(plan.limitation.clone());
    if negative > 0 {
        limitations.push(format!(
            "gap.unexplained is negative at {negative} point(s): the accounted figure exceeds {RSS} (reserved but untouched memory)"
        ));
    }
    let mut source = SeriesSource::new(SeriesSourceKind::Gap);
    source.harness.clone_from(&src.source.harness);
    source.workload.clone_from(&src.source.workload);
    source.mode = src.source.mode;
    source.runtime = src.source.runtime;
    source.pid = src.source.pid;
    source.command.clone_from(&src.source.command);
    source.derived_from = Some(src.id.clone());
    let out = SeriesFile {
        schema_version: SERIES_SCHEMA_VERSION,
        id,
        source,
        axis: src.axis,
        warmup: src.warmup,
        columns: vec![
            derived_column("gap.accounted"),
            derived_column("gap.unexplained"),
        ],
        points,
        events: src.events.clone(),
        limitations,
    };
    Ok((out, plan.label))
}

fn gate_name(g: GateVerdict) -> &'static str {
    if g == GateVerdict::Pass {
        "pass"
    } else {
        "fail"
    }
}

/// Runs the split on series `src` and returns the command's JSON object.
pub fn run(tc: &ToolCtx, src: &SeriesFile) -> Res<Value> {
    RunState::load(&tc.run_dir)?;
    let (derived, label) = derive(tc.next_series_id(), src)?;
    derived.save(&tc.run_dir)?;
    let derived_rel = format!("series/{}.json", derived.id);

    let th = GateThresholds::default();
    let cols: Vec<_> = (0..derived.columns.len())
        .map(|i| fit::analyze_gauge(&derived, i, &th))
        .collect();
    let trend_id = tc.next_trend_id();
    TrendReport {
        schema_version: TREND_SCHEMA_VERSION,
        id: trend_id.clone(),
        series: derived.id.clone(),
        thresholds: th,
        columns: cols.clone(),
    }
    .save(&tc.run_dir)?;
    let (accounted, unexplained) = (&cols[0], &cols[1]);
    let slope = |c: &dla_core::trend::ColumnTrend| c.sen.map(|s| fit::round(s.slope));
    let route = (unexplained.gate == GateVerdict::Pass).then_some("native");

    let result = json!({
        "series": src.id,
        "derived": derived.id,
        "trend": trend_id,
        "accounted_slope": slope(accounted),
        "unexplained_slope": slope(unexplained),
        "unexplained_gate": gate_name(unexplained.gate),
        "unexplained_shape": unexplained.shape,
        "route": route,
    });
    let mut spec = EvidenceSpec::new(
        EvidenceKind::NonHeap,
        "dla",
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = series_authority(src);
    spec.raw = Some(format!("{TREND_DIR}/{trend_id}.json"));
    spec.source = Some(tc.artifact_source(&derived_rel)?);
    spec.coverage = EvidenceCoverage {
        status: CoverageStatus::Complete,
        examined: Some(2),
        total: Some(2),
    };
    spec.limitations = derived.limitations.clone();
    spec.limitations
        .push("gate thresholds are provisional (not calibrated)".to_owned());
    let mut ledger = tc.ledger()?;
    let ev: EvidenceId = tc.add_evidence(&mut ledger, spec)?;

    let h = src.source.harness.as_deref().unwrap_or("<H>");
    let next: Vec<String> = if route.is_some() {
        vec![
            "read references/native.md (growth outside the runtime's accounting)".to_owned(),
            format!("dla alloc-test {h} --purge trim"),
            format!("dla gap {} --residual", src.id),
        ]
    } else {
        vec![format!(
            "dla gap {} --residual   (growth no finding explains yet)",
            src.id
        )]
    };
    let mut limitations = derived.limitations.clone();
    limitations.truncate(4);
    Ok(json!({
        "ok": true,
        "command": "gap",
        "series": src.id,
        "derived": derived.id,
        "trend": trend_id,
        "accounted": label,
        "evidence": [ev.to_string()],
        "accounted_slope": slope(accounted),
        "unexplained_slope": slope(unexplained),
        "unexplained_gate": gate_name(unexplained.gate),
        "unexplained_shape": unexplained.shape,
        "route": route,
        "artifacts": [tc.pointer(&derived_rel), tc.pointer(&format!("{TREND_DIR}/{trend_id}.json"))],
        "limitations": limitations,
        "next": next,
    }))
}
