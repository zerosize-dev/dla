//! Small statistics helpers shared by the gap split and the residual table.
//!
//! The per-column analysis is a small copy of what `dla trend` does for a gauge column
//! (this command stays self-contained and does not call the trend command).

use dla_core::{
    series::{ColumnScope, SeriesFile},
    trend::ColumnTrend,
};
use dla_stats::{GateThresholds, GateVerdict, analyze, noise, theil_sen};

/// A Theil-Sen fit of one column over its post-warm-up points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    /// Slope per x unit.
    pub slope: f64,
    /// Robust residual scale around the fit.
    pub noise: f64,
    /// Distance between the first and the last post-warm-up x.
    pub span: f64,
}

/// Fits column `idx`; `None` with fewer than three finite post-warm-up points.
pub fn fit_column(series: &SeriesFile, idx: usize) -> Option<Fit> {
    let pts = series.post_warmup(idx);
    if pts.len() < 3 || pts.iter().any(|&(x, y)| !x.is_finite() || !y.is_finite()) {
        return None;
    }
    let sen = theil_sen(&pts, GateThresholds::default().alpha)?;
    let span = pts.last()?.0 - pts.first()?.0;
    Some(Fit {
        slope: sen.slope,
        noise: noise(&pts, &sen),
        span,
    })
}

/// Analyses a gauge column with the default gate thresholds.
pub fn analyze_gauge(series: &SeriesFile, idx: usize, th: &GateThresholds) -> ColumnTrend {
    let col = &series.columns[idx];
    let a = analyze(&series.post_warmup(idx), None, false, th);
    let mut gate = a.gate;
    let mut gate_failed = a.gate_failed;
    if col.scope == ColumnScope::Container {
        gate = GateVerdict::Fail;
        gate_failed.push("container scope".to_owned());
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

/// Keeps four significant digits so a result stays small.
pub fn round(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    format!("{x:.3e}").parse().unwrap_or(x)
}

/// A number for a sentence: no more digits than are meaningful.
pub fn human(x: f64) -> String {
    let a = x.abs();
    if a >= 100.0 {
        format!("{x:.0}")
    } else if a >= 1.0 {
        format!("{x:.1}")
    } else {
        format!("{x:.3}")
    }
}
