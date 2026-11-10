//! The bounded `result` of a growth evidence record.

use std::collections::BTreeMap;

use dla_core::{
    MAX_RESULT_BYTES,
    evidence::Signal,
    series::{ColumnKind, SeriesFile},
    trend::{ColumnTrend, GrowthSummary, PassedColumn},
};
use dla_stats::GateVerdict;
use serde_json::Value;

/// Keeps four significant digits so a result stays small.
fn round(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    format!("{x:.3e}").parse().unwrap_or(x)
}

/// The `passed` entry of a column that passed the gate.
pub fn passed_column(c: &ColumnTrend) -> PassedColumn {
    PassedColumn {
        column: c.name.clone(),
        region: Some(c.region),
        site: c.site.clone(),
        finding: c.finding.as_ref().map(ToString::to_string),
        candidates: c.candidates.iter().map(ToString::to_string).collect(),
        slope: c.sen.map(|s| round(s.slope)),
        lo: c.sen.map(|s| round(s.lo)),
        p: c.mk.map(|m| round(m.p_increasing)),
        ratio: c.halves.and_then(|h| h.ratio).map(round),
        shape: Some(c.shape),
        unit: Some(c.unit),
    }
}

/// True when a passed column is bound to a finding or a site.
fn bound_to_something(c: &ColumnTrend) -> bool {
    c.finding.is_some() || c.site.is_some() || !c.candidates.is_empty()
}

/// The passing columns of `columns`, most important first: bound to a finding or a
/// site, then by p-value (columns without a p last), then by name.
pub fn passing<'a>(columns: &[&'a ColumnTrend]) -> Vec<&'a ColumnTrend> {
    let mut v: Vec<&ColumnTrend> = columns
        .iter()
        .copied()
        .filter(|c| c.gate == GateVerdict::Pass)
        .collect();
    v.sort_by(|a, b| {
        let key = |c: &ColumnTrend| {
            (
                !bound_to_something(c),
                c.mk.map_or(f64::INFINITY, |m| m.p_increasing),
            )
        };
        let (ka, kb) = (key(a), key(b));
        ka.0.cmp(&kb.0)
            .then(ka.1.total_cmp(&kb.1))
            .then_with(|| a.name.cmp(&b.name))
    });
    v
}

/// The summary for the columns of one kind (gauge or counter) of `series`, written
/// into trend `trend_id`. `passed` is cut from the end until the serialized result
/// fits in [`MAX_RESULT_BYTES`]; `truncated` counts the cut entries.
pub fn summarize(
    trend_id: &str,
    series: &SeriesFile,
    columns: &[&ColumnTrend],
    signal: &Signal,
) -> Value {
    let ordered = passing(columns);
    let mut shapes: BTreeMap<String, u32> = BTreeMap::new();
    for c in columns {
        let name = serde_json::to_value(c.shape)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        *shapes.entry(name).or_default() += 1;
    }
    let mut summary = GrowthSummary {
        trend: Some(trend_id.to_owned()),
        series: Some(series.id.clone()),
        workload: series.source.workload.clone(),
        mode: series.source.mode.map(|m| m.as_str().to_owned()),
        axis: Some(series.axis.as_str().to_owned()),
        n: Some(
            u32::try_from(series.points.len().saturating_sub(series.warmup as usize))
                .unwrap_or(u32::MAX),
        ),
        passed: ordered.iter().map(|c| passed_column(c)).collect(),
        failed: Some(u32::try_from(columns.len() - ordered.len()).unwrap_or(u32::MAX)),
        shapes,
        gate: Some(if ordered.is_empty() {
            GateVerdict::Fail
        } else {
            GateVerdict::Pass
        }),
        truncated: None,
    };
    let mut cut = 0u32;
    loop {
        let mut v = serde_json::to_value(&summary).unwrap_or(Value::Null);
        if let Value::Object(m) = &mut v {
            m.insert("signal".to_owned(), signal.to_value());
        }
        let size = serde_json::to_string(&v).map_or(usize::MAX, |s| s.len());
        if size <= MAX_RESULT_BYTES || summary.passed.is_empty() {
            return v;
        }
        // Cut in chunks proportional to the overshoot so 64 columns need few rounds.
        let per = (size / summary.passed.len()).max(1);
        let drop = ((size - MAX_RESULT_BYTES) / per + 1).min(summary.passed.len());
        summary.passed.truncate(summary.passed.len() - drop);
        cut += u32::try_from(drop).unwrap_or(u32::MAX);
        summary.truncated = Some(cut);
    }
}

/// Splits `columns` by kind: (gauges, counters).
pub fn by_kind(columns: &[ColumnTrend]) -> (Vec<&ColumnTrend>, Vec<&ColumnTrend>) {
    columns.iter().partition(|c| c.kind == ColumnKind::Gauge)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trend_cmd::analysis::{
        analyze_series,
        testutil::{Col, plateau, rising, series},
    };
    use dla_core::FindingId;
    use dla_stats::GateThresholds;

    fn sig() -> Signal {
        Signal::new("dla-trend", "harness-series", &"ab".repeat(32))
    }

    fn many(n: usize) -> (SeriesFile, Vec<ColumnTrend>) {
        let cols: Vec<Col> = (0..n)
            .map(|i| {
                let mut c = Col::counter(
                    &format!(
                        "go.goroutines@internal/pkg/very/long/path/to/subscription_{i}.go:{}",
                        10 + i
                    ),
                    &rising(10.0, 2.0 + i as f64),
                )
                .site(&format!(
                    "internal/pkg/very/long/path/to/subscription_{i}.go:{}",
                    10 + i
                ));
                if i % 2 == 0 {
                    c = c.finding(FindingId::new(1));
                }
                c
            })
            .collect();
        let s = series("S1", 1, cols);
        let idx: Vec<usize> = (0..n).collect();
        let mut t = analyze_series(&s, &idx, &GateThresholds::default());
        for c in &mut t {
            c.candidates = vec![FindingId::new(1), FindingId::new(2), FindingId::new(3)];
        }
        (s, t)
    }

    #[test]
    fn sixty_four_passing_columns_stay_under_the_bound() {
        let (s, t) = many(64);
        let refs: Vec<&ColumnTrend> = t.iter().collect();
        assert!(t.iter().all(|c| c.gate == GateVerdict::Pass));
        let v = summarize("T2", &s, &refs, &sig());
        let text = serde_json::to_string(&v).unwrap();
        assert!(text.len() <= MAX_RESULT_BYTES, "{}", text.len());
        let g = GrowthSummary::from_result(&v).unwrap();
        assert!(g.passed_gate());
        let cut = g.truncated.expect("some passed columns are cut");
        assert_eq!(g.passed.len() as u32 + cut, 64);
        assert_eq!(g.failed, Some(0));
        assert_eq!(g.shapes["steady"], 64);
        assert_eq!(g.axis.as_deref(), Some("n"));
        assert_eq!(g.n, Some(8));
    }

    #[test]
    fn bound_columns_come_first_then_by_p() {
        let s = series(
            "S1",
            1,
            vec![
                Col::gauge("os.rss", &rising(1e7, 1e6)),
                Col::gauge("jvm.heap_used", &rising(1e7, 1e6)).site("a.java:3"),
                Col::gauge(
                    "noisy",
                    &[
                        1e7, 1.0e7, 1.1e7, 1.05e7, 1.2e7, 1.15e7, 1.3e7, 1.25e7, 1.4e7,
                    ],
                ),
            ],
        );
        let t = analyze_series(&s, &[0, 1, 2], &GateThresholds::default());
        let refs: Vec<&ColumnTrend> = t.iter().collect();
        let names: Vec<&str> = passing(&refs).iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names[0], "jvm.heap_used");
        assert!(names.contains(&"os.rss"));
        let v = summarize("T1", &s, &refs, &sig());
        let g = GrowthSummary::from_result(&v).unwrap();
        assert_eq!(g.passed[0].column, "jvm.heap_used");
        assert_eq!(g.passed[0].site.as_deref(), Some("a.java:3"));
        assert!(g.truncated.is_none());
    }

    #[test]
    fn no_passing_column_gives_a_failing_gate_and_counts() {
        let s = series("S1", 1, vec![Col::counter("a", &plateau(100.0))]);
        let t = analyze_series(&s, &[0], &GateThresholds::default());
        let refs: Vec<&ColumnTrend> = t.iter().collect();
        let g = GrowthSummary::from_result(&summarize("T1", &s, &refs, &sig())).unwrap();
        assert!(!g.passed_gate());
        assert_eq!(g.gate, Some(GateVerdict::Fail));
        assert_eq!(g.failed, Some(1));
        assert_eq!(g.shapes["plateau"], 1);
    }
}
