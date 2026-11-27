//! The step reading of a gauge column: the points that set a new maximum by more than the
//! column's own scatter, the gaps between them, and how long the next gap is expected to be.
//!
//! A container that doubles its capacity (a `Vec`, a hash map, a slice, a list: every runtime's
//! byte columns) grows in steps that come ever further apart, and any finite window of it ends in
//! a flat stretch about as long as all the growth before it. Both `dla alloc-test` (an extra
//! window that must say whether the growth went on) and the trend gate (a whole series) read that
//! shape here, so they cannot disagree about what a step is.

use crate::trend::{noise, theil_sen};

/// A step of a column that grows in steps is a new maximum of more than this many units of the
/// window's own scatter around its trend: the scatter of a regular staircase is about a third of
/// its step, so its steps pass, and the few-KB jitter of a level column (a rise or two of one
/// scatter) does not.
pub const STEP_SCATTERS: f64 = 2.0;

/// Every new maximum of `points` over `noise`, as `(x, rise over the maximum before it)`, counted
/// from `start` (the column at the end of the warm-up; the first point when there is none).
pub fn rises(points: &[(f64, f64)], start: Option<f64>, noise: f64) -> Vec<(f64, f64)> {
    let mut pts = points.iter();
    let first = pts.next();
    let Some(mut max) = start.or(first.map(|p| p.1)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    // With a `start`, the first point is a reading like the others; without one it is the base.
    let rest = if start.is_some() { points.iter() } else { pts };
    for &(x, y) in rest {
        if y > max + noise {
            out.push((x, y - max));
        }
        max = max.max(y);
    }
    out
}

/// The `x` of every point that sets a new maximum by more than `noise` (see [`rises`]).
pub fn rise_xs(points: &[(f64, f64)], start: Option<f64>, noise: f64) -> Vec<f64> {
    rises(points, start, noise)
        .into_iter()
        .map(|r| r.0)
        .collect()
}

/// The separate steps of one window.
#[derive(Debug, Clone, PartialEq)]
pub struct Steps {
    /// `x` of each step.
    pub xs: Vec<f64>,
    /// Rise of each step over the maximum before it.
    pub rises: Vec<f64>,
    /// The window's scatter around its Theil-Sen trend (one unit of [`STEP_SCATTERS`]).
    pub scatter: f64,
}

/// The steps of `points`: `None` for a window that did not grow in separate steps (fewer than two
/// rises, or a rise at every interval, which is a ramp: its scatter around a straight fit is small
/// and the ordinary slope test serves).
pub fn steps(points: &[(f64, f64)]) -> Option<Steps> {
    let trend = theil_sen(points, 0.05)?;
    let scatter = noise(points, &trend);
    steps_over(points, scatter, STEP_SCATTERS * scatter)
}

/// A step of the trend gate rises by at least this share of the series' highest level: a
/// doubling container's smallest steps are far above it, the bytes of drift on a level are not.
pub const STEP_SHARE: f64 = 0.01;

/// The steps of a whole series for the trend gate. A doubling container's scatter around a
/// straight fit is its steps (a third of the rise or more), so [`steps`] can not see them there;
/// what scatters a stair is the change from one point to the next, which is zero while it holds a
/// level. A step is a new maximum of more than [`STEP_SCATTERS`] units of the robust scale of
/// those changes (1.4826 times their median absolute deviation) and of at least [`STEP_SHARE`] of
/// the series' highest level.
pub fn trend_steps(points: &[(f64, f64)]) -> Option<Steps> {
    let deltas: Vec<f64> = points.windows(2).map(|w| w[1].1 - w[0].1).collect();
    let med = median(&deltas)?;
    let scatter = 1.4826 * median(&deltas.iter().map(|d| (d - med).abs()).collect::<Vec<_>>())?;
    let top = points.iter().fold(0.0_f64, |m, p| m.max(p.1.abs()));
    steps_over(
        points,
        scatter,
        (STEP_SCATTERS * scatter).max(STEP_SHARE * top),
    )
}

fn median(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_unstable_by(f64::total_cmp);
    let m = s.len() / 2;
    Some(if s.len() % 2 == 1 {
        s[m]
    } else {
        (s[m - 1] + s[m]) / 2.0
    })
}

fn steps_over(points: &[(f64, f64)], scatter: f64, threshold: f64) -> Option<Steps> {
    let found = rises(points, None, threshold);
    if found.len() < 2 || found.len() >= points.len().saturating_sub(1) {
        return None;
    }
    Some(Steps {
        xs: found.iter().map(|r| r.0).collect(),
        rises: found.iter().map(|r| r.1).collect(),
        scatter,
    })
}

impl Steps {
    /// How far apart the steps are.
    pub fn gaps(&self) -> Vec<f64> {
        self.xs.windows(2).map(|w| w[1] - w[0]).collect()
    }

    /// `x` of the last step.
    pub fn last_x(&self) -> f64 {
        self.xs[self.xs.len() - 1]
    }

    /// How long after the last step the next one is expected, given one point spacing.
    ///
    /// A container that doubles its capacity steps ever further apart (each gap about twice the
    /// one before), so when each gap is at least the one before (one point spacing of slack; a
    /// single gap qualifies, and the smaller early steps of a doubling can lie under the scatter
    /// that marks a step) the next gap is expected to be `2 * (last gap + one spacing)`;
    /// otherwise the longest gap seen.
    pub fn expected_gap(&self, per_point: f64) -> f64 {
        let gaps = self.gaps();
        let longest = gaps.iter().copied().fold(0.0_f64, f64::max);
        let growing = !gaps.is_empty() && gaps.windows(2).all(|w| w[1] + per_point >= w[0]);
        match gaps.last() {
            Some(last) if growing => longest.max(2.0 * (last + per_point)),
            _ => longest,
        }
    }

    /// Whether no step is smaller than the one before by more than one scatter (a doubling's
    /// steps double; a warm-up followed by small growth shrinks).
    pub fn never_shrink(&self) -> bool {
        self.rises.windows(2).all(|w| w[1] + self.scatter >= w[0])
    }
}

/// Points of a window that starts at `first_x` and has `len` points `per_point` apart that would
/// run past the step expected `expected` after `last_step`.
pub fn points_needed(
    last_step: f64,
    expected: f64,
    first_x: f64,
    per_point: f64,
    len: usize,
) -> u32 {
    let needed = if per_point > 0.0 {
        ((last_step + expected - first_x) / per_point).floor() as u32 + 2
    } else {
        len as u32 + 1
    };
    needed.max(len as u32 + 1)
}
