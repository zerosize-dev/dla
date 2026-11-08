//! Per-column analysis: a series column becomes a [`ColumnTrend`].

use dla_core::{
    Refusal,
    error::codes,
    series::{ColumnKind, ColumnScope, SeriesFile},
    trend::ColumnTrend,
};
use dla_stats::{GateThresholds, GateVerdict, analyze};

/// The gate-failure reason of a column that describes the whole container.
pub const CONTAINER_REASON: &str = "container scope";

/// Indexes of the columns to analyse: all of them, or the named ones in series order.
pub fn select(series: &SeriesFile, names: &[String]) -> Result<Vec<usize>, Refusal> {
    if names.is_empty() {
        return Ok((0..series.columns.len()).collect());
    }
    for n in names {
        if series.column_index(n).is_none() {
            let have: Vec<&str> = series
                .columns
                .iter()
                .take(8)
                .map(|c| c.name.as_str())
                .collect();
            return Err(Refusal::new(
                codes::INVALID_VALUE,
                format!("series {} has no column {n:?}", series.id),
                "an existing --column name",
                format!(
                    "dla trend {} --column {}   (columns include: {})",
                    series.id,
                    have.first().copied().unwrap_or("os.rss"),
                    have.join(", ")
                ),
            ));
        }
    }
    Ok((0..series.columns.len())
        .filter(|&i| names.contains(&series.columns[i].name))
        .collect())
}

/// The last warm-up point of column `idx` that has a value.
fn warmup_tail(series: &SeriesFile, idx: usize) -> Option<(f64, f64)> {
    series
        .points
        .iter()
        .take(series.warmup as usize)
        .rev()
        .find_map(|p| Some((p.x, p.values.get(idx).copied().flatten()?)))
}

/// Analyses column `idx` over its post-warm-up points: counters use the every-delta
/// gate (with the warm-up tail as the first delta's start), gauges the slope gate.
/// A container-scope column is analysed but never passes: its figure is not the
/// measured process's.
pub fn analyze_column(series: &SeriesFile, idx: usize, th: &GateThresholds) -> ColumnTrend {
    let col = &series.columns[idx];
    let counter = col.kind == ColumnKind::Counter;
    let pts = series.post_warmup(idx);
    let tail = counter.then(|| warmup_tail(series, idx)).flatten();
    let a = analyze(&pts, tail, counter, th);
    let mut gate = a.gate;
    let mut gate_failed = a.gate_failed;
    if col.scope == ColumnScope::Container {
        gate = GateVerdict::Fail;
        gate_failed.push(CONTAINER_REASON.to_owned());
    }
    ColumnTrend {
        name: col.name.clone(),
        unit: col.unit,
        kind: col.kind,
        region: col.region,
        site: col.site.clone(),
        finding: col.finding.clone(),
        candidates: Vec::new(),
        n: a.n,
        mk: a.mk,
        sen: a.sen,
        halves: a.halves,
        every_delta: a.every_delta,
        noise: a.noise,
        rise: a.rise,
        shape: a.shape,
        gate,
        gate_failed,
    }
}

/// Analyses the selected columns.
pub fn analyze_series(series: &SeriesFile, idx: &[usize], th: &GateThresholds) -> Vec<ColumnTrend> {
    idx.iter().map(|&i| analyze_column(series, i, th)).collect()
}

#[cfg(test)]
pub mod testutil {
    //! Synthetic series for the unit tests.

    use dla_core::{
        FindingId,
        runtime::Runtime,
        series::{
            Axis, Column, ColumnKind, ColumnScope, Point, SERIES_SCHEMA_VERSION, SeriesFile,
            SeriesRegion, SeriesSource, SeriesSourceKind, Unit,
        },
    };

    /// A column spec: name, kind, scope, site, finding, values (one per point).
    pub struct Col {
        pub name: String,
        pub kind: ColumnKind,
        pub scope: ColumnScope,
        pub site: Option<String>,
        pub finding: Option<FindingId>,
        pub values: Vec<Option<f64>>,
    }

    impl Col {
        pub fn counter(name: &str, values: &[f64]) -> Self {
            Self::new(name, ColumnKind::Counter, values)
        }
        pub fn gauge(name: &str, values: &[f64]) -> Self {
            Self::new(name, ColumnKind::Gauge, values)
        }
        fn new(name: &str, kind: ColumnKind, values: &[f64]) -> Self {
            Self {
                name: name.to_owned(),
                kind,
                scope: ColumnScope::Process,
                site: None,
                finding: None,
                values: values.iter().map(|v| Some(*v)).collect(),
            }
        }
        pub fn site(mut self, s: &str) -> Self {
            self.site = Some(s.to_owned());
            self
        }
        pub fn finding(mut self, f: FindingId) -> Self {
            self.finding = Some(f);
            self
        }
        pub fn container(mut self) -> Self {
            self.scope = ColumnScope::Container;
            self
        }
    }

    /// A series with `warmup` leading warm-up points, x = 1000 * k.
    pub fn series(id: &str, warmup: u32, cols: Vec<Col>) -> SeriesFile {
        let n = cols.iter().map(|c| c.values.len()).max().unwrap_or(0);
        let mut src = SeriesSource::new(SeriesSourceKind::Harness);
        src.harness = Some("H1".into());
        src.runtime = Some(Runtime::Go);
        SeriesFile {
            schema_version: SERIES_SCHEMA_VERSION,
            id: id.to_owned(),
            source: src,
            axis: Axis::N,
            warmup,
            columns: cols
                .iter()
                .map(|c| Column {
                    name: c.name.clone(),
                    unit: if c.kind == ColumnKind::Counter {
                        Unit::Count
                    } else {
                        Unit::Bytes
                    },
                    kind: c.kind,
                    region: SeriesRegion::Heap,
                    baseline: false,
                    after_gc: true,
                    scope: c.scope,
                    site: c.site.clone(),
                    finding: c.finding.clone(),
                    source: "test".into(),
                })
                .collect(),
            points: (0..n)
                .map(|k| Point {
                    x: 1000.0 * (k as f64 + 1.0),
                    t_ms: 100 * k as u64,
                    values: cols
                        .iter()
                        .map(|c| c.values.get(k).copied().flatten())
                        .collect(),
                })
                .collect(),
            events: vec![],
            limitations: vec![],
        }
    }

    /// 1 warm-up + 8 measured values, rising by `step` each point from `start`.
    pub fn rising(start: f64, step: f64) -> Vec<f64> {
        (0..9).map(|k| start + step * k as f64).collect()
    }

    /// 1 warm-up point then a rise that levels off at `cap`.
    pub fn plateau(cap: f64) -> Vec<f64> {
        let mut v = vec![0.0];
        v.extend((1..9).map(|k| cap * (1.0 - (-(k as f64) * 1.2).exp())));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::*;
    use super::*;
    use dla_stats::TrendShape;

    fn th() -> GateThresholds {
        GateThresholds::default()
    }

    #[test]
    fn a_steady_counter_passes_with_its_warmup_delta() {
        let s = series(
            "S1",
            1,
            vec![Col::counter("probe:Cache.entries", &rising(10.0, 25.0)).site("src/cache.go:9")],
        );
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.n, 8);
        assert_eq!(t.gate, GateVerdict::Pass, "{:?}", t.gate_failed);
        assert_eq!(t.shape, TrendShape::Steady);
        assert!(t.every_delta.unwrap().all_at_least_one);
        assert_eq!(t.site.as_deref(), Some("src/cache.go:9"));
        assert!(t.sen.unwrap().slope > 0.0);
    }

    #[test]
    fn a_drop_between_warmup_and_first_point_fails_the_counter() {
        let mut v = rising(10.0, 25.0);
        v[0] = 500.0;
        let s = series("S1", 1, vec![Col::counter("go.goroutines", &v)]);
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.gate, GateVerdict::Fail);
        assert!(t.gate_failed.iter().any(|r| r == "delta < 1"));
    }

    #[test]
    fn a_plateauing_probe_fails_the_gate_with_a_plateau_shape() {
        let s = series(
            "S1",
            1,
            vec![Col::counter("probe:Cache.entries", &plateau(1000.0))],
        );
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.gate, GateVerdict::Fail);
        assert_eq!(t.shape, TrendShape::Plateau);
        // Same data as a gauge: the slope gate fails on the decaying half too.
        let s = series(
            "S1",
            1,
            vec![Col::gauge("probe:Cache.bytes", &plateau(1e6))],
        );
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.gate, GateVerdict::Fail);
        assert!(!t.gate_failed.is_empty());
    }

    #[test]
    fn a_growing_gauge_passes() {
        let s = series("S1", 1, vec![Col::gauge("os.rss", &rising(1e7, 4e5))]);
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.gate, GateVerdict::Pass, "{:?}", t.gate_failed);
        assert!(t.halves.unwrap().ratio.unwrap() > 0.5);
    }

    #[test]
    fn a_container_column_is_analysed_but_never_passes() {
        let s = series(
            "S1",
            1,
            vec![Col::gauge("cgroup.current", &rising(1e7, 4e5)).container()],
        );
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.gate, GateVerdict::Fail);
        assert_eq!(t.gate_failed, [CONTAINER_REASON]);
        assert_eq!(t.shape, TrendShape::Steady);
        assert!(t.sen.is_some());
    }

    #[test]
    fn too_few_points_fail_with_a_reason() {
        let s = series("S1", 1, vec![Col::counter("a", &[1.0, 2.0, 3.0, 4.0])]);
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.gate, GateVerdict::Fail);
        assert!(t.gate_failed.iter().any(|r| r.starts_with("n <")));
    }

    #[test]
    fn missing_values_are_skipped_and_select_checks_names() {
        let mut c = Col::gauge("a", &rising(1.0, 1.0));
        c.values[3] = None;
        let s = series("S1", 1, vec![c, Col::gauge("b", &rising(1.0, 1.0))]);
        assert_eq!(analyze_column(&s, 0, &th()).n, 7);
        assert_eq!(select(&s, &["b".into()]).unwrap(), [1]);
        assert_eq!(select(&s, &[]).unwrap(), [0, 1]);
        let r = select(&s, &["zz".into()]).unwrap_err();
        assert_eq!(r.code, codes::INVALID_VALUE);
        assert!(r.fix.contains("--column a"), "{}", r.fix);
    }
}
