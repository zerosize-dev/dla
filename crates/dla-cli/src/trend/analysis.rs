//! Per-column analysis: a series column becomes a [`ColumnTrend`].

use dla_core::{
    Refusal,
    error::codes,
    series::{ColumnKind, ColumnScope, SeriesFile, SeriesSourceKind, not_a_growth_signal},
    trend::ColumnTrend,
};
use dla_stats::{GateThresholds, GateVerdict, TrendShape, analyze};

use crate::summarize::rule;

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
    if let Some(reason) = not_a_growth_signal(&col.name) {
        gate = GateVerdict::Fail;
        gate_failed.push(reason.to_owned());
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
        gate_note: a.gate_note.filter(|_| gate == GateVerdict::Pass),
        resized: false,
    }
}

/// Analyses the selected columns. A series `dla summarize` wrote from captures (source kind
/// `dump`) is judged by the capture rule `dla summarize` applied to it ([`capture_column`]); every
/// other series by the slope and every-delta gates.
pub fn analyze_series(series: &SeriesFile, idx: &[usize], th: &GateThresholds) -> Vec<ColumnTrend> {
    idx.iter()
        .map(|&i| {
            if is_capture_series(series) {
                capture_column(series, i, th)
            } else {
                analyze_column(series, i, th)
            }
        })
        .collect()
}

/// True for a series `dla summarize` wrote from capture files (source kind `dump`): three to a
/// few captures, one point each, which the capture rule judges.
pub fn is_capture_series(series: &SeriesFile) -> bool {
    series.source.kind == SeriesSourceKind::Dump
}

/// What `dla trend` says about the rule that judged a series, for its report and its evidence.
pub fn rule_line(series: &SeriesFile) -> Option<String> {
    is_capture_series(series).then(|| {
        format!(
            "judged by the capture-series rule of `dla summarize`, not the {}-point slope gate (the thresholds of this report are read for the statistics only): at least {} captures, bytes strictly increasing across every pair, the block count strictly increasing too or one constant positive number (resized), the last interval at least {:.0}% of the first{} (provisional, not calibrated)",
            GateThresholds::default().min_points,
            rule::MIN_DUMPS,
            rule::LAST_OVER_FIRST * 100.0,
            if rule::series_xs(series).is_some() {
                " (compared as rates over the series' own x)"
            } else {
                ""
            }
        )
    })
}

/// Column `idx` of a capture series: the statistics of [`analyze_column`] (slope, noise, halves
/// are shown as they are), and the gate of the capture rule ([`rule::judge_column`]): a column
/// passes exactly when `dla summarize` passed it, whatever the point count. The failure reasons
/// are the rule's. A column that passes reads `steady` or `stair` by the rule's own shape (the
/// byte increases within a factor of two of each other or not) and carries `resized`; a column
/// that fails does not read as growing, whatever the statistics of three points say (a rise that
/// stopped reads `plateau`). A container-scope column and a column that is no growth signal keep
/// the verdict [`analyze_column`] gave them.
pub fn capture_column(series: &SeriesFile, idx: usize, th: &GateThresholds) -> ColumnTrend {
    let col = &series.columns[idx];
    let mut t = analyze_column(series, idx, th);
    if col.scope == ColumnScope::Container || not_a_growth_signal(&col.name).is_some() {
        return t;
    }
    match rule::judge_column(series, idx) {
        Ok(g) => {
            t.gate = GateVerdict::Pass;
            t.gate_failed.clear();
            t.gate_note = None;
            t.shape = match rule::shape(&g) {
                "steady" => TrendShape::Steady,
                _ => TrendShape::Stair,
            };
            t.resized = g.resized;
        }
        Err(why) => {
            t.gate = GateVerdict::Fail;
            t.gate_failed = why;
            t.gate_note = None;
            if matches!(t.shape, TrendShape::Steady | TrendShape::Stair) {
                t.shape = TrendShape::Plateau;
            }
        }
    }
    t
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
    use dla_core::series::NOT_GROWTH_REASON;
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
    fn memory_returned_to_the_os_is_analysed_but_never_passes_as_growth() {
        // The shape of `go.released` in a run of a flat Go fixture: lumps the scavenger returned,
        // a stair. As any other byte column it passes.
        let v = [
            12.0e6, 13.0e6, 13.4e6, 14.0e6, 18.5e6, 19.0e6, 19.5e6, 24.0e6, 24.5e6,
        ];
        let other = series("S1", 1, vec![Col::gauge("go.total", &v)]);
        let t = analyze_column(&other, 0, &th());
        assert_eq!(t.gate, GateVerdict::Pass, "{:?}", t.gate_failed);
        assert_eq!(t.shape, TrendShape::Stair);
        let s = series("S1", 1, vec![Col::gauge("go.released", &v)]);
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.gate, GateVerdict::Fail);
        assert_eq!(t.gate_failed, [NOT_GROWTH_REASON]);
        // Recorded and shown: the figures and the shape are still there.
        assert_eq!(t.shape, TrendShape::Stair);
        assert!(t.sen.is_some() && t.rise > 0.0);
    }

    #[test]
    fn too_few_points_fail_with_a_reason() {
        let s = series("S1", 1, vec![Col::counter("a", &[1.0, 2.0, 3.0, 4.0])]);
        let t = analyze_column(&s, 0, &th());
        assert_eq!(t.gate, GateVerdict::Fail);
        assert!(t.gate_failed.iter().any(|r| r.starts_with("n <")));
    }

    /// A series `dla summarize` wrote: source kind `dump`, three captures at x = 440, 880, 1760
    /// units of work.
    fn capture_series(cols: Vec<Col>) -> SeriesFile {
        let mut s = series("S1", 0, cols);
        s.source = dla_core::series::SeriesSource::new(SeriesSourceKind::Dump);
        s.source.command = Some("dla summarize alloc-stack-diff".into());
        s.axis = dla_core::series::Axis::Iteration;
        for (p, x) in s.points.iter_mut().zip([440.0, 880.0, 1760.0]) {
            p.x = x;
        }
        s
    }

    #[test]
    fn a_capture_series_of_three_points_is_judged_by_the_capture_rule_and_carries_resized() {
        let s = capture_series(vec![
            Col::gauge("stack:grow", &[1e6, 2e6, 4e6]).site("src/a.rs:3"),
            Col::counter("stack:grow.count", &[100.0, 200.0, 400.0]).site("src/a.rs:3"),
            Col::gauge("stack:buf", &[1_048_576.0, 2_097_152.0, 4_194_304.0]),
            Col::counter("stack:buf.count", &[1.0, 1.0, 1.0]),
            // A fill that stopped: the last interval is 2% of the first.
            Col::gauge("stack:fill", &[500.0, 800.0, 806.0]),
            Col::counter("stack:fill.count", &[5.0, 8.0, 9.0]),
            // Bytes that rise evenly over a count that falls: a steady line to the statistics
            // of three points, no growth to the rule.
            Col::gauge("stack:drop", &[1000.0, 2000.0, 3000.0]),
            Col::counter("stack:drop.count", &[30.0, 20.0, 10.0]),
        ]);
        let t = analyze_series(&s, &[0, 1, 2, 3, 4, 5, 6, 7], &th());
        let by = |n: &str| t.iter().find(|c| c.name == n).unwrap();
        for n in [
            "stack:grow",
            "stack:grow.count",
            "stack:buf",
            "stack:buf.count",
        ] {
            assert_eq!(
                by(n).gate,
                GateVerdict::Pass,
                "{n}: {:?}",
                by(n).gate_failed
            );
            assert!(by(n).gate_failed.is_empty(), "{n}");
            assert_eq!(by(n).shape, TrendShape::Steady, "{n}");
        }
        // The count column of a block that is resized reads `resized`, not flat.
        assert!(by("stack:buf.count").resized && by("stack:buf").resized);
        assert!(!by("stack:grow").resized && !by("stack:grow.count").resized);
        // A fill that stopped fails with the rule's reasons (not the eight-point gate's), in
        // both of its columns, and does not read as growing.
        for n in [
            "stack:fill",
            "stack:fill.count",
            "stack:drop",
            "stack:drop.count",
        ] {
            let c = by(n);
            assert_eq!(c.gate, GateVerdict::Fail, "{n}");
            assert!(
                c.gate_failed
                    .iter()
                    .any(|r| r.contains("last interval") || r.contains("count does not increase")),
                "{:?}",
                c.gate_failed
            );
            assert!(c.gate_failed.iter().all(|r| !r.starts_with("n <")), "{n}");
            assert!(
                !matches!(c.shape, TrendShape::Steady | TrendShape::Stair),
                "{n}"
            );
        }
        // The same figures in a harness series are the eight-point gate's: three points fail it.
        let mut h = capture_series(vec![Col::gauge("stack:grow", &[1e6, 2e6, 4e6])]);
        h.source = dla_core::series::SeriesSource::new(SeriesSourceKind::Harness);
        let t = analyze_series(&h, &[0], &th());
        assert_eq!(t[0].gate, GateVerdict::Fail);
        assert!(t[0].gate_failed.iter().any(|r| r.starts_with("n <")));
        assert!(!t[0].resized);
    }

    #[test]
    fn a_capture_column_has_the_verdict_the_summary_rule_gives_the_same_figures() {
        // The column of a capture series passes exactly when the rule passes its bytes and
        // counts, at the series' own x (a doubling at x doubling: the same rate).
        let cases: [([f64; 3], [f64; 3], bool); 6] = [
            ([100.0, 200.0, 400.0], [1.0, 2.0, 4.0], true),
            ([100.0, 200.0, 400.0], [1.0, 1.0, 1.0], true),
            ([100.0, 200.0, 400.0], [1.0, 2.0, 2.0], false),
            ([100.0, 100.0, 400.0], [1.0, 2.0, 4.0], false),
            ([100.0, 200.0, 250.0], [1.0, 2.0, 3.0], false),
            ([100.0, 200.0, 400.0], [0.0, 0.0, 0.0], false),
        ];
        for (bytes, counts, pass) in cases {
            let s = capture_series(vec![
                Col::gauge("stack:x", &bytes),
                Col::counter("stack:x.count", &counts),
            ]);
            let t = analyze_series(&s, &[0, 1], &th());
            assert!(
                t.iter().all(|c| (c.gate == GateVerdict::Pass) == pass),
                "{bytes:?} {counts:?}: {t:?}"
            );
            let rule = rule::judge_column(&s, 0).is_ok();
            assert_eq!(rule, pass, "{bytes:?} {counts:?}");
        }
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
