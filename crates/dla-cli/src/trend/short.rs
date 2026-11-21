//! A series with fewer points after warm-up than the gate needs is not a quiet series.
//!
//! The gate fails such a column for the point count only (`n < 8`) when every other condition
//! holds; those columns are listed here, the strongest first, and the first `next` line says
//! how to measure at full length. The stored trend keeps `gate: fail` for them.

use dla_core::{
    Ledger,
    series::{ColumnKind, SeriesFile, Unit},
    trend::ColumnTrend,
};
use dla_stats::{GateThresholds, GateVerdict};
use serde_json::{Value, json};

/// The most columns `short` lists.
pub const MAX_SHORT: usize = 12;

/// The points a series has after its warm-up, and the points the gate needs.
pub fn points(series: &SeriesFile, th: &GateThresholds) -> (usize, usize) {
    (
        series.points.len().saturating_sub(series.warmup as usize),
        th.min_points as usize,
    )
}

/// True when the series has too few points for the gate.
pub fn is_short(series: &SeriesFile, th: &GateThresholds) -> bool {
    let (n, need) = points(series, th);
    n < need
}

/// True when a column failed the gate for its point count and nothing else.
fn only_count(c: &ColumnTrend, th: &GateThresholds) -> bool {
    c.gate == GateVerdict::Fail
        && c.unit != Unit::Seconds
        && c.gate_failed.len() == 1
        && c.gate_failed[0] == format!("n < {}", th.min_points)
        && c.sen.is_some_and(|s| s.slope > 0.0)
}

/// Columns that fail only for the point count: site and probe columns with candidates first,
/// then other counters, then byte columns; at most [`MAX_SHORT`].
pub fn columns<'a>(cols: &'a [ColumnTrend], th: &GateThresholds) -> Vec<&'a ColumnTrend> {
    let mut v: Vec<&ColumnTrend> = cols.iter().filter(|c| only_count(c, th)).collect();
    let rank = |c: &ColumnTrend| -> u8 {
        if !c.candidates.is_empty() || c.finding.is_some() {
            0
        } else if c.site.is_some() || c.name.starts_with("probe:") {
            1
        } else if c.kind == ColumnKind::Counter {
            2
        } else {
            3
        }
    };
    v.sort_by(|a, b| {
        rank(a).cmp(&rank(b)).then(
            b.rise
                .abs()
                .total_cmp(&a.rise.abs())
                .then(a.name.cmp(&b.name)),
        )
    });
    v.truncate(MAX_SHORT);
    v
}

fn slope_text(c: &ColumnTrend) -> String {
    let s = c.sen.map_or(0.0, |s| s.slope);
    if s.fract() == 0.0 || s.abs() >= 100.0 {
        format!("{s:.0}")
    } else {
        format!("{s:.2}")
    }
}

/// The `short` output field.
pub fn json_list(short: &[&ColumnTrend]) -> Value {
    Value::Array(
        short
            .iter()
            .map(|c| {
                json!({
                    "column": c.name,
                    "slope": c.sen.map(|s| s.slope),
                    "candidates": c.candidates.iter().map(ToString::to_string).collect::<Vec<_>>(),
                })
            })
            .collect(),
    )
}

/// The first `next` line of a short series. `rerun` is the command that measures at full
/// length (without `--iterations`), when the series came from a harness.
pub fn next_line(
    ledger: &Ledger,
    short: &[&ColumnTrend],
    n: usize,
    need: usize,
    rerun: Option<&str>,
) -> String {
    let strongest: Vec<String> = short
        .iter()
        .take(3)
        .map(|c| {
            let ids: Vec<String> = c
                .candidates
                .iter()
                .filter(|f| ledger.finding(f).is_some())
                .map(ToString::to_string)
                .collect();
            if ids.is_empty() {
                format!("{} (+{} per iteration)", c.name, slope_text(c))
            } else {
                format!(
                    "{} (+{} per iteration, finding {})",
                    c.name,
                    slope_text(c),
                    ids.join("/")
                )
            }
        })
        .collect();
    let what = if strongest.is_empty() {
        "no column rises".to_owned()
    } else {
        format!("rising: {}", strongest.join("; "))
    };
    let tail = match rerun {
        Some(cmd) => format!("{cmd}   # full length, then dla trend"),
        None => "measure with more points, then dla trend".to_owned(),
    };
    format!(
        "SERIES TOO SHORT: {n} points after warm-up, the gate needs {need}, so nothing can pass and no target is settled by it; {what}. Run {tail}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_stats::{GateThresholds, analyze};

    fn column(name: &str, ys: &[f64], counter: bool, site: Option<&str>) -> ColumnTrend {
        let pts: Vec<(f64, f64)> = ys
            .iter()
            .enumerate()
            .map(|(i, y)| ((i as f64 + 1.0) * 10.0, *y))
            .collect();
        let a = analyze(&pts, None, counter, &GateThresholds::default());
        ColumnTrend {
            name: name.into(),
            unit: if counter { Unit::Count } else { Unit::Bytes },
            kind: if counter {
                ColumnKind::Counter
            } else {
                ColumnKind::Gauge
            },
            region: dla_core::series::SeriesRegion::Heap,
            site: site.map(str::to_owned),
            finding: None,
            candidates: Vec::new(),
            n: a.n,
            mk: a.mk,
            sen: a.sen,
            halves: a.halves,
            every_delta: a.every_delta,
            noise: a.noise,
            rise: a.rise,
            shape: a.shape,
            gate: a.gate,
            gate_failed: a.gate_failed,
        }
    }

    #[test]
    fn only_the_columns_that_fail_for_the_count_alone_are_listed_site_columns_first() {
        let th = GateThresholds::default();
        let cols = vec![
            column("node.heap_used", &[10., 20., 30., 40.], false, None),
            column("node.listeners", &[10., 20., 30., 40.], true, None),
            column("node.listeners@a.js:3", &[1., 2., 3., 4.], true, Some("a.js:3")),
            column("os.rss", &[5., 5., 5., 5.], false, None),
        ];
        assert!(cols[0].gate_failed == ["n < 8"], "{:?}", cols[0].gate_failed);
        let s = columns(&cols, &th);
        let names: Vec<&str> = s.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            ["node.listeners@a.js:3", "node.listeners", "node.heap_used"]
        );
    }
}
