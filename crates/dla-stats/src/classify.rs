//! Growth-shape classification, the slope gate and the one-call [`analyze`].
//!
//! # Definitions
//!
//! With `n` post-warm-up points, `sen` the Theil-Sen fit, `noise` the robust residual scale,
//! `rise = sen.slope * (x_last - x_first)` and `tol = noise_multiple * max(noise, eps)` with
//! `eps = 1e-9 * max|y|` (so a perfectly clean series does not get a zero tolerance), the
//! shape is the first of:
//!
//! 1. `insufficient`: fewer than three points, or a non-finite value.
//! 2. `spike`: `max(y) - max(y_first, y_last) > tol` and `|y_last - y_first| <= tol`: a
//!    transient excursion that returns. Checked before `flat` because a spike that returns
//!    leaves the Sen slope at about zero and would otherwise always read as flat.
//! 3. `flat`: Mann-Kendall `p_increasing >= alpha` for the increasing test and `|rise| <= tol`.
//! 4. `declining`: `sen.hi < 0`.
//! 5. `plateau`: the first half rises (slope `> 0` and first-half rise `> tol`; the spec's
//!    alternative "first-half Mann-Kendall `p < alpha`" is dropped because it lets pure
//!    noise with a lucky p-value through) and then levels off (half ratio `< 0.5`, or second-half rise
//!    `<= tol`).
//! 6. `flat` again when the overall slope is not positive (a weak, inconclusive decline; a
//!    single early step has a median pairwise slope of zero and is caught by `plateau`).
//! 7. `stair`: the largest `ceil(deltas / 3)` consecutive deltas hold at least 80% of
//!    `y_last - y_first` (which must exceed `tol`) while each remaining delta is `<= tol`.
//! 8. `steady` when `rise > tol`; otherwise `flat` (a significant but sub-noise drift).
//!
//! The gate for gauges passes iff `n >= min_points`, `p_increasing < alpha`, `sen.lo > 0`,
//! the second half is not decaying (`ratio >= min_half_ratio`; when the first-half slope is
//! not positive the ratio is undefined and the half passes if the second-half slope is
//! positive) and `rise > noise_multiple * noise`. For counters it passes iff
//! `n >= min_points` and every consecutive delta (including the one from the last warm-up
//! point to the first measured point) is at least one.

use crate::trend::{every_delta, half_split, mann_kendall, noise, theil_sen};
use crate::types::{
    EveryDelta, GateThresholds, GateVerdict, HalfSplit, MannKendall, TheilSen, TrendShape,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Everything a trend column needs, from one series.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Analysis {
    /// Number of points analysed (post-warm-up, finite).
    pub n: u32,
    /// Mann-Kendall test, when there are at least three points.
    pub mk: Option<MannKendall>,
    /// Theil-Sen fit with confidence interval.
    pub sen: Option<TheilSen>,
    /// First-half and second-half slopes.
    pub halves: Option<HalfSplit>,
    /// Every-delta rule, for counters only.
    pub every_delta: Option<EveryDelta>,
    /// Robust residual scale (1.4826 * MAD).
    pub noise: f64,
    /// `sen.slope` times the x-span of the points.
    pub rise: f64,
    /// Growth shape.
    pub shape: TrendShape,
    /// Gate outcome.
    pub gate: GateVerdict,
    /// Reasons the gate failed (empty on pass).
    pub gate_failed: Vec<String>,
}

/// Fitted pieces shared by classification and the gate.
struct Fit {
    mk: MannKendall,
    sen: TheilSen,
    halves: HalfSplit,
    noise: f64,
    rise: f64,
    tol: f64,
}

fn fit(points: &[(f64, f64)], th: &GateThresholds) -> Option<Fit> {
    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    let mk = mann_kendall(&ys)?;
    let sen = theil_sen(points, th.alpha)?;
    let halves = half_split(points)?;
    let noise = noise(points, &sen);
    let span = points[points.len() - 1].0 - points[0].0;
    let rise = sen.slope * span;
    let eps = 1e-9 * ys.iter().fold(0.0_f64, |m, y| m.max(y.abs()));
    Some(Fit {
        mk,
        sen,
        halves,
        noise,
        rise,
        tol: th.noise_multiple * noise.max(eps),
    })
}

fn span(points: &[(f64, f64)]) -> f64 {
    points[points.len() - 1].0 - points[0].0
}

fn classify_fit(points: &[(f64, f64)], f: &Fit, th: &GateThresholds) -> TrendShape {
    let n = points.len();
    let first = points[0].1;
    let last = points[n - 1].1;
    let max = points.iter().fold(f64::NEG_INFINITY, |m, p| m.max(p.1));

    if max - first.max(last) > f.tol && (last - first).abs() <= f.tol {
        return TrendShape::Spike;
    }
    if f.mk.p_increasing >= th.alpha && f.rise.abs() <= f.tol {
        return TrendShape::Flat;
    }
    if f.sen.hi < 0.0 {
        return TrendShape::Declining;
    }

    // Plateau: rises in the first half, then levels off.
    let h = n.div_ceil(2);
    let first_half = &points[..h];
    let first_rise = f.halves.first * span(first_half);
    let second_rise = f.halves.second * span(&points[n - h..]);
    let first_grows = f.halves.first > 0.0 && first_rise > f.tol;
    let levels_off = f.halves.ratio.is_some_and(|r| r < 0.5) || second_rise <= f.tol;
    if first_grows && levels_off {
        return TrendShape::Plateau;
    }

    if f.sen.slope <= 0.0 {
        return TrendShape::Flat;
    }

    // Stair: a few large deltas carry the rise.
    let mut deltas: Vec<f64> = points.windows(2).map(|w| w[1].1 - w[0].1).collect();
    let total = last - first;
    if total > f.tol {
        deltas.sort_unstable_by(|a, b| b.total_cmp(a));
        let k = deltas.len().div_ceil(3);
        let top: f64 = deltas[..k].iter().sum();
        if top >= 0.8 * total && deltas[k..].iter().all(|&d| d <= f.tol) {
            return TrendShape::Stair;
        }
    }
    if f.rise <= f.tol {
        return TrendShape::Flat;
    }
    TrendShape::Steady
}

/// Classifies the growth shape of `points` (ascending `x`, warm-up removed).
pub fn classify(points: &[(f64, f64)], thresholds: &GateThresholds) -> TrendShape {
    match fit(points, thresholds) {
        Some(f) => classify_fit(points, &f, thresholds),
        None => TrendShape::Insufficient,
    }
}

fn gauge_gate(n: usize, f: Option<&Fit>, th: &GateThresholds) -> Vec<String> {
    let mut why = Vec::new();
    if n < th.min_points as usize {
        why.push(format!("n < {}", th.min_points));
    }
    let Some(f) = f else {
        if n >= th.min_points as usize {
            why.push("series not analysable".to_string());
        }
        return why;
    };
    if f.mk.p_increasing >= th.alpha {
        why.push(format!("p >= {}", th.alpha));
    }
    if f.sen.lo <= 0.0 {
        why.push("sen lower bound <= 0".to_string());
    }
    let decaying = match f.halves.ratio {
        Some(r) => r < th.min_half_ratio,
        None => f.halves.second <= 0.0,
    };
    if decaying {
        why.push("decaying half-split".to_string());
    }
    if f.rise <= f.tol {
        why.push(format!("rise <= {} * noise", th.noise_multiple));
    }
    why
}

/// Slope gate for a gauge series: the verdict and the reasons it failed.
pub fn gate(points: &[(f64, f64)], thresholds: &GateThresholds) -> (GateVerdict, Vec<String>) {
    let f = fit(points, thresholds);
    let why = gauge_gate(points.len(), f.as_ref(), thresholds);
    (verdict(&why), why)
}

/// Every-delta gate for a counter series (`warmup_tail` is the last warm-up point, if any).
pub fn gate_counter(
    points: &[(f64, f64)],
    warmup_tail: Option<(f64, f64)>,
    thresholds: &GateThresholds,
) -> (GateVerdict, Vec<String>) {
    let ed = counter_deltas(points, warmup_tail);
    let why = counter_gate(points.len(), ed.as_ref(), thresholds);
    (verdict(&why), why)
}

fn verdict(why: &[String]) -> GateVerdict {
    if why.is_empty() {
        GateVerdict::Pass
    } else {
        GateVerdict::Fail
    }
}

fn counter_deltas(points: &[(f64, f64)], tail: Option<(f64, f64)>) -> Option<EveryDelta> {
    let mut all: Vec<(f64, f64)> = tail.into_iter().collect();
    all.extend_from_slice(points);
    every_delta(&all)
}

fn counter_gate(n: usize, ed: Option<&EveryDelta>, th: &GateThresholds) -> Vec<String> {
    let mut why = Vec::new();
    if n < th.min_points as usize {
        why.push(format!("n < {}", th.min_points));
    }
    match ed {
        Some(ed) if !ed.all_at_least_one => why.push("delta < 1".to_string()),
        None if n >= th.min_points as usize => why.push("no deltas".to_string()),
        _ => {}
    }
    why
}

/// Runs every statistic for one series.
///
/// `points` are the post-warm-up `(x, y)` values (sorted by `x` here if they are not);
/// `warmup_tail` is the last warm-up point, used only for the counter every-delta rule;
/// `counter` selects the counter gate (every delta >= 1) over the gauge gate. Non-finite
/// values make the result `insufficient` and the gate fail.
pub fn analyze(
    points: &[(f64, f64)],
    warmup_tail: Option<(f64, f64)>,
    counter: bool,
    thresholds: &GateThresholds,
) -> Analysis {
    let mut pts = points.to_vec();
    if pts.iter().any(|&(x, y)| !x.is_finite() || !y.is_finite()) {
        return Analysis {
            n: points.len() as u32,
            mk: None,
            sen: None,
            halves: None,
            every_delta: None,
            noise: 0.0,
            rise: 0.0,
            shape: TrendShape::Insufficient,
            gate: GateVerdict::Fail,
            gate_failed: vec!["non-finite value".to_string()],
        };
    }
    pts.sort_by(|a, b| a.0.total_cmp(&b.0));
    let n = pts.len();
    let f = fit(&pts, thresholds);
    let shape = f.as_ref().map_or(TrendShape::Insufficient, |f| {
        classify_fit(&pts, f, thresholds)
    });
    let ed = counter.then(|| counter_deltas(&pts, warmup_tail)).flatten();
    let why = if counter {
        counter_gate(n, ed.as_ref(), thresholds)
    } else {
        gauge_gate(n, f.as_ref(), thresholds)
    };
    Analysis {
        n: n as u32,
        mk: f.as_ref().map(|f| f.mk),
        sen: f.as_ref().map(|f| f.sen),
        halves: f.as_ref().map(|f| f.halves),
        every_delta: ed,
        noise: f.as_ref().map_or(0.0, |f| f.noise),
        rise: f.as_ref().map_or(0.0, |f| f.rise),
        shape,
        gate: verdict(&why),
        gate_failed: why,
    }
}
