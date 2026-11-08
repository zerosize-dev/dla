//! Result types shared with `dla-core` (the trend report embeds them).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Mann-Kendall trend test.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MannKendall {
    /// S statistic: sum of sign(y_j - y_i) over i < j.
    pub s: i64,
    /// Variance of S, corrected for ties.
    pub var_s: f64,
    /// Standardised statistic with continuity correction.
    pub z: f64,
    /// One-sided p-value for an increasing trend (exact for small samples without ties).
    pub p_increasing: f64,
    /// Kendall's tau: S / (n(n-1)/2).
    pub tau: f64,
}

/// Theil-Sen slope with Sen's confidence interval.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TheilSen {
    /// Median of pairwise slopes.
    pub slope: f64,
    /// Median of y - slope * x.
    pub intercept: f64,
    /// Lower bound of the confidence interval.
    pub lo: f64,
    /// Upper bound of the confidence interval.
    pub hi: f64,
}

/// Theil-Sen slopes of the first and second half of the points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HalfSplit {
    /// Slope of the first half.
    pub first: f64,
    /// Slope of the second half.
    pub second: f64,
    /// `second / first` when `first > 0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ratio: Option<f64>,
}

/// CPython's every-delta rule for integer counters: every consecutive delta is at least one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EveryDelta {
    /// Number of deltas examined.
    pub deltas: u32,
    /// True when every delta is >= 1.
    pub all_at_least_one: bool,
    /// Smallest delta.
    pub min: f64,
    /// Largest delta.
    pub max: f64,
}

/// Shape of a measured series after warm-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TrendShape {
    /// Fewer than three points.
    Insufficient,
    /// No trend beyond noise.
    Flat,
    /// Falling.
    Declining,
    /// Steady growth.
    Steady,
    /// Growth in discrete steps.
    Stair,
    /// A transient rise that returns.
    Spike,
    /// Growth that levels off.
    Plateau,
}

/// Outcome of the slope gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum GateVerdict {
    /// The series shows growth that passes the gate.
    Pass,
    /// It does not.
    Fail,
}

/// Thresholds of the slope gate. Provisional until the M7 calibration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GateThresholds {
    /// Minimum post-warm-up points.
    pub min_points: u32,
    /// Significance level for Mann-Kendall and the Sen interval.
    pub alpha: f64,
    /// Minimum second-half / first-half slope ratio (non-decaying growth).
    pub min_half_ratio: f64,
    /// Rise must exceed this multiple of the noise.
    pub noise_multiple: f64,
    /// False until the thresholds are calibrated on the benchmark.
    pub calibrated: bool,
}

impl Default for GateThresholds {
    fn default() -> Self {
        Self {
            min_points: 8,
            alpha: 0.05,
            min_half_ratio: 0.5,
            noise_multiple: 3.0,
            calibrated: false,
        }
    }
}
