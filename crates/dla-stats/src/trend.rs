//! Trend primitives: Mann-Kendall, Theil-Sen, half-split, every-delta and the noise scale.
//!
//! # Algorithms
//!
//! **Mann-Kendall.** `S = sum_{i<j} sign(y_j - y_i)`. The variance of `S` under the null of
//! no trend is `[n(n-1)(2n+5) - sum_t t(t-1)(2t+5)] / 18`, the sum running over groups of
//! `t` tied values. `Z = (S - 1)/sqrt(var)` for `S > 0`, `(S + 1)/sqrt(var)` for `S < 0`
//! and `0` otherwise (continuity correction); with zero variance (all values equal) `Z = 0`.
//! The one-sided p-value for an increasing trend is `1 - Phi(Z)`, except for `n <= 10` with
//! no ties, where it is the exact `P(S >= s)` under a uniformly random permutation. That
//! distribution comes from the Mahonian numbers (permutations of `n` by inversion count `k`,
//! built by dynamic programming: adding the m-th element adds 0..m-1 inversions) and
//! `S = n(n-1)/2 - 2 * inversions`. `tau = S / (n(n-1)/2)`.
//!
//! **Theil-Sen.** The slope is the median of `(y_j - y_i)/(x_j - x_i)` over all pairs with
//! `x_j != x_i` (zero-dx pairs are skipped); the intercept is the median of
//! `y - slope * x`. Sen's interval uses `C = z_{1-alpha/2} * sqrt(var_s)`, `M1 = (N - C)/2`,
//! `M2 = (N + C)/2` over the `N` ascending pairwise slopes. In 1-based ranks the bounds are
//! the slopes at rank `floor(M1)` and rank `ceil(M2) + 1` (the usual Gilbert/Sen
//! construction, rounded outwards), i.e. 0-based indices `floor(M1) - 1` and `ceil(M2)`,
//! clamped into `0..N`. `var_s` is the tie-corrected variance of the Mann-Kendall `S` of the
//! `y` values (in `x` order).
//!
//! **Half split.** Theil-Sen slopes of the first and the last `ceil(n/2)` points (the middle
//! point is shared when `n` is odd); `ratio = second / first` when `first > 0`.
//!
//! **Every delta.** CPython's refleak rule `all(delta >= 1)` over consecutive `y` deltas.
//!
//! **Noise.** `1.4826 * MAD` where MAD is the median absolute deviation, about the median of
//! the residuals, of `y - (intercept + slope * x)`.
//!
//! All functions expect points ordered by ascending `x` and finite values; they return `None`
//! (or zero for [`noise`]) when the input is too short or contains NaN or infinities.

use crate::normal::{norm_quantile, norm_sf};
use crate::types::{EveryDelta, HalfSplit, MannKendall, TheilSen};

/// Scale turning a MAD into a standard deviation estimate for normal data.
const MAD_TO_SIGMA: f64 = 1.4826;

/// Largest sample size for which the exact Mann-Kendall distribution is used.
const EXACT_MAX_N: usize = 10;

fn all_finite(points: &[(f64, f64)]) -> bool {
    points.iter().all(|&(x, y)| x.is_finite() && y.is_finite())
}

/// Median of a slice sorted ascending; `NaN` for an empty slice.
fn median_sorted(v: &[f64]) -> f64 {
    let n = v.len();
    match n {
        0 => f64::NAN,
        _ if n % 2 == 1 => v[n / 2],
        _ => 0.5 * (v[n / 2 - 1] + v[n / 2]),
    }
}

/// Median of an unsorted slice (sorts a copy).
fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_unstable_by(f64::total_cmp);
    median_sorted(&s)
}

/// Sizes of the groups of equal values in `ys`.
fn tie_groups(ys: &[f64]) -> Vec<u64> {
    let mut s = ys.to_vec();
    s.sort_unstable_by(f64::total_cmp);
    let mut groups = Vec::new();
    let mut i = 0;
    while i < s.len() {
        let mut j = i + 1;
        while j < s.len() && s[j] == s[i] {
            j += 1;
        }
        groups.push((j - i) as u64);
        i = j;
    }
    groups
}

/// Tie-corrected variance of the Mann-Kendall `S` for `ys`.
fn var_s_of(ys: &[f64]) -> f64 {
    let n = ys.len() as f64;
    let ties: f64 = tie_groups(ys)
        .iter()
        .map(|&t| {
            let t = t as f64;
            t * (t - 1.0) * (2.0 * t + 5.0)
        })
        .sum();
    (n * (n - 1.0) * (2.0 * n + 5.0) - ties) / 18.0
}

/// Exact `P(S >= s)` for `n` distinct values under a random permutation.
fn exact_p_increasing(n: usize, s: i64) -> f64 {
    // counts[k] = number of permutations of n elements with k inversions.
    let mut counts = vec![1_u64];
    for m in 2..=n {
        let max_inv = m * (m - 1) / 2;
        let mut next = vec![0_u64; max_inv + 1];
        for (k, slot) in next.iter_mut().enumerate() {
            let lo = k.saturating_sub(m - 1);
            let hi = k.min(counts.len() - 1);
            if lo <= hi {
                *slot = counts[lo..=hi].iter().sum();
            }
        }
        counts = next;
    }
    let total_pairs = (n * (n - 1) / 2) as i64;
    let max_inv = total_pairs - s; // S >= s  <=>  2 * inversions <= total_pairs - s
    if max_inv < 0 {
        return 0.0;
    }
    let limit = ((max_inv / 2) as usize).min(counts.len() - 1);
    let favourable: u64 = counts[..=limit].iter().sum();
    let all: u64 = counts.iter().sum();
    favourable as f64 / all as f64
}

/// Mann-Kendall trend test of `ys` taken in order.
///
/// Returns `None` for fewer than three values or any non-finite value.
pub fn mann_kendall(ys: &[f64]) -> Option<MannKendall> {
    let n = ys.len();
    if n < 3 || ys.iter().any(|y| !y.is_finite()) {
        return None;
    }
    let mut s: i64 = 0;
    for i in 0..n {
        for j in i + 1..n {
            s += match ys[j].partial_cmp(&ys[i]) {
                Some(std::cmp::Ordering::Greater) => 1,
                Some(std::cmp::Ordering::Less) => -1,
                _ => 0,
            };
        }
    }
    let var_s = var_s_of(ys);
    let z = if var_s <= 0.0 {
        0.0
    } else if s > 0 {
        (s as f64 - 1.0) / var_s.sqrt()
    } else if s < 0 {
        (s as f64 + 1.0) / var_s.sqrt()
    } else {
        0.0
    };
    let no_ties = tie_groups(ys).iter().all(|&t| t == 1);
    let p_increasing = if n <= EXACT_MAX_N && no_ties {
        exact_p_increasing(n, s)
    } else {
        norm_sf(z)
    };
    let pairs = (n * (n - 1) / 2) as f64;
    Some(MannKendall {
        s,
        var_s,
        z,
        p_increasing,
        tau: s as f64 / pairs,
    })
}

/// Pairwise slopes of `points`, skipping pairs with equal `x`.
fn pairwise_slopes(points: &[(f64, f64)]) -> Vec<f64> {
    let n = points.len();
    let mut out = Vec::with_capacity(n * n.saturating_sub(1) / 2);
    for i in 0..n {
        for j in i + 1..n {
            let dx = points[j].0 - points[i].0;
            if dx != 0.0 {
                out.push((points[j].1 - points[i].1) / dx);
            }
        }
    }
    out
}

/// Theil-Sen slope and intercept without a confidence interval, as `(slope, intercept)`.
fn sen_fit(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    if points.len() < 2 || !all_finite(points) {
        return None;
    }
    let mut slopes = pairwise_slopes(points);
    if slopes.is_empty() {
        return None;
    }
    slopes.sort_unstable_by(f64::total_cmp);
    let slope = median_sorted(&slopes);
    let resid: Vec<f64> = points.iter().map(|&(x, y)| y - slope * x).collect();
    Some((slope, median(&resid)))
}

/// Theil-Sen estimator with Sen's `1 - alpha` confidence interval.
///
/// `points` must be ordered by ascending `x`. Returns `None` for fewer than three points,
/// non-finite values, or when every `x` is equal. `alpha` is clamped to `[1e-12, 0.999]`.
pub fn theil_sen(points: &[(f64, f64)], alpha: f64) -> Option<TheilSen> {
    if points.len() < 3 || !all_finite(points) {
        return None;
    }
    let mut slopes = pairwise_slopes(points);
    if slopes.is_empty() {
        return None;
    }
    slopes.sort_unstable_by(f64::total_cmp);
    let slope = median_sorted(&slopes);
    let resid: Vec<f64> = points.iter().map(|&(x, y)| y - slope * x).collect();
    let intercept = median(&resid);

    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    let alpha = if alpha.is_finite() {
        alpha.clamp(1e-12, 0.999)
    } else {
        0.05
    };
    let c = norm_quantile(1.0 - alpha / 2.0) * var_s_of(&ys).max(0.0).sqrt();
    let big_n = slopes.len() as f64;
    let m1 = ((big_n - c) / 2.0).floor();
    let m2 = ((big_n + c) / 2.0).ceil();
    let last = slopes.len() - 1;
    let lo_idx = (m1 - 1.0).clamp(0.0, last as f64) as usize;
    let hi_idx = m2.clamp(0.0, last as f64) as usize;
    Some(TheilSen {
        slope,
        intercept,
        lo: slopes[lo_idx],
        hi: slopes[hi_idx],
    })
}

/// Theil-Sen slopes of the first and last `ceil(n/2)` points.
///
/// Returns `None` for fewer than three points or when a half has no usable slope.
pub fn half_split(points: &[(f64, f64)]) -> Option<HalfSplit> {
    let n = points.len();
    if n < 3 || !all_finite(points) {
        return None;
    }
    let h = n.div_ceil(2);
    let (first, _) = sen_fit(&points[..h])?;
    let (second, _) = sen_fit(&points[n - h..])?;
    let ratio = (first > 0.0).then(|| second / first);
    Some(HalfSplit {
        first,
        second,
        ratio,
    })
}

/// CPython's every-delta rule over consecutive `y` values.
///
/// Returns `None` for fewer than two points or non-finite values.
pub fn every_delta(points: &[(f64, f64)]) -> Option<EveryDelta> {
    if points.len() < 2 || !all_finite(points) {
        return None;
    }
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for w in points.windows(2) {
        let d = w[1].1 - w[0].1;
        min = min.min(d);
        max = max.max(d);
    }
    Some(EveryDelta {
        deltas: (points.len() - 1) as u32,
        all_at_least_one: min >= 1.0,
        min,
        max,
    })
}

/// Robust residual scale: `1.4826 * MAD` of `y - (intercept + slope * x)`.
///
/// Zero for fewer than three points.
pub fn noise(points: &[(f64, f64)], sen: &TheilSen) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }
    let resid: Vec<f64> = points
        .iter()
        .map(|&(x, y)| y - (sen.intercept + sen.slope * x))
        .collect();
    let med = median(&resid);
    let dev: Vec<f64> = resid.iter().map(|r| (r - med).abs()).collect();
    MAD_TO_SIGMA * median(&dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pts(ys: &[f64]) -> Vec<(f64, f64)> {
        ys.iter().enumerate().map(|(i, &y)| (i as f64, y)).collect()
    }

    /// Calls `f` with every permutation of `0..n` (Heap's algorithm).
    fn for_each_permutation(n: usize, mut f: impl FnMut(&[f64])) {
        fn rec(k: usize, a: &mut Vec<f64>, f: &mut impl FnMut(&[f64])) {
            if k == 1 {
                f(a);
                return;
            }
            rec(k - 1, a, f);
            for i in 0..k - 1 {
                if k % 2 == 0 {
                    a.swap(i, k - 1);
                } else {
                    a.swap(0, k - 1);
                }
                rec(k - 1, a, f);
            }
        }
        let mut a: Vec<f64> = (0..n).map(|i| i as f64).collect();
        rec(n, &mut a, &mut f);
    }

    fn brute_s(a: &[f64]) -> i64 {
        let mut s = 0;
        for i in 0..a.len() {
            for j in i + 1..a.len() {
                s += if a[j] > a[i] { 1 } else { -1 };
            }
        }
        s
    }

    #[test]
    fn exact_p_matches_enumeration_of_all_permutations() {
        for n in 4..=8_usize {
            // Brute force: histogram of S over all n! permutations.
            let mut hist = std::collections::BTreeMap::<i64, u64>::new();
            let mut total = 0_u64;
            for_each_permutation(n, |a| {
                *hist.entry(brute_s(a)).or_default() += 1;
                total += 1;
            });
            // Every permutation must report exactly P(S' >= S) from the histogram.
            for_each_permutation(n, |a| {
                let mk = mann_kendall(a).unwrap();
                let tail: u64 = hist.range(mk.s..).map(|(_, c)| c).sum();
                let want = tail as f64 / total as f64;
                assert!(
                    (mk.p_increasing - want).abs() < 1e-12,
                    "n={n} s={} got {} want {want}",
                    mk.s,
                    mk.p_increasing
                );
                assert_eq!(mk.s, brute_s(a));
            });
        }
    }

    #[test]
    fn exact_p_of_monotone_sequence_is_one_over_n_factorial() {
        let ys: Vec<f64> = (0..8).map(f64::from).collect();
        let mk = mann_kendall(&ys).unwrap();
        assert_eq!(mk.s, 28);
        assert!((mk.p_increasing - 1.0 / 40320.0).abs() < 1e-15);
        assert!((mk.tau - 1.0).abs() < 1e-15);
        let ys: Vec<f64> = (0..10).map(f64::from).collect();
        assert!((mann_kendall(&ys).unwrap().p_increasing - 1.0 / 3_628_800.0).abs() < 1e-18);
    }

    #[test]
    fn normal_approximation_with_ties() {
        // n = 5, y = [1, 1, 2, 3, 3]: pairs: S = 8 (10 pairs, 2 ties).
        // ties: two groups of t=2 -> sum t(t-1)(2t+5) = 2*18 = 36
        // var = (5*4*15 - 36)/18 = 264/18 = 14.666...
        let mk = mann_kendall(&[1.0, 1.0, 2.0, 3.0, 3.0]).unwrap();
        assert_eq!(mk.s, 8);
        assert!((mk.var_s - 264.0 / 18.0).abs() < 1e-12);
        let z = 7.0 / (264.0_f64 / 18.0).sqrt();
        assert!((mk.z - z).abs() < 1e-12);
        assert!((mk.p_increasing - norm_sf(z)).abs() < 1e-12);
    }

    #[test]
    fn normal_approximation_for_n_above_ten() {
        let ys: Vec<f64> = (0..12).map(f64::from).collect();
        let mk = mann_kendall(&ys).unwrap();
        assert_eq!(mk.s, 66);
        let var = 12.0 * 11.0 * 29.0 / 18.0;
        assert!((mk.var_s - var).abs() < 1e-12);
        assert!((mk.z - 65.0 / var.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn all_ties_and_degenerate_inputs() {
        let mk = mann_kendall(&[2.0; 9]).unwrap();
        assert_eq!((mk.s, mk.z, mk.var_s), (0, 0.0, 0.0));
        assert!((mk.p_increasing - 0.5).abs() < 1e-12);
        assert!(mann_kendall(&[1.0, 2.0]).is_none());
        assert!(mann_kendall(&[1.0, f64::NAN, 3.0]).is_none());
        assert!(mann_kendall(&[1.0, f64::INFINITY, 3.0]).is_none());
        // Decreasing: S < 0 gives a high one-sided p.
        let dec: Vec<f64> = (0..9).rev().map(f64::from).collect();
        assert!(mann_kendall(&dec).unwrap().p_increasing > 0.999);
    }

    // Hand-worked dataset: x = 1..5, y = [1, 3, 2, 5, 4].
    // Pairwise slopes (i, j): (1,2)=2 (1,3)=1/2 (1,4)=4/3 (1,5)=3/4 (2,3)=-1 (2,4)=1
    // (2,5)=1/3 (3,4)=3 (3,5)=1 (4,5)=-1.
    // Sorted: -1, -1, 1/3, 1/2, 3/4, 1, 1, 4/3, 2, 3 (N = 10); median = (3/4 + 1)/2 = 0.875.
    // Residuals y - 0.875 x: 0.125, 1.25, -0.625, 1.5, -0.375 -> sorted -0.625, -0.375,
    // 0.125, 1.25, 1.5; intercept 0.125.
    // S = 8 positive - 2 negative = 6, var_s = 5*4*15/18 = 16.667, sd = 4.0825.
    // C = 1.96 * 4.0825 = 8.002; M1 = 0.999 -> floor 0; M2 = 9.001 -> ceil 10.
    // 0-based indices floor(M1)-1 = -1 -> 0 and ceil(M2) = 10 -> 9 (clamped): [-1, 3].
    #[test]
    fn theil_sen_hand_worked_dataset() {
        let p: Vec<(f64, f64)> = [1.0, 3.0, 2.0, 5.0, 4.0]
            .iter()
            .enumerate()
            .map(|(i, &y)| ((i + 1) as f64, y))
            .collect();
        let sen = theil_sen(&p, 0.05).unwrap();
        assert!((sen.slope - 0.875).abs() < 1e-12);
        assert!((sen.intercept - 0.125).abs() < 1e-12);
        assert_eq!((sen.lo, sen.hi), (-1.0, 3.0));
        // A wide alpha shrinks the interval: alpha = 0.5 -> z = 0.6745, C = 2.754,
        // M1 = 3.623 -> floor 3 -> index 2 -> 1/3; M2 = 6.377 -> ceil 7 -> index 7 -> 4/3.
        let sen = theil_sen(&p, 0.5).unwrap();
        assert!((sen.lo - 1.0 / 3.0).abs() < 1e-12);
        assert!((sen.hi - 4.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn theil_sen_exact_line_and_outlier_robustness() {
        let line: Vec<(f64, f64)> = (0..12)
            .map(|i| (f64::from(i), 2.0 * f64::from(i) + 1.0))
            .collect();
        let sen = theil_sen(&line, 0.05).unwrap();
        assert!((sen.slope - 2.0).abs() < 1e-12 && (sen.intercept - 1.0).abs() < 1e-12);
        assert!((sen.lo - 2.0).abs() < 1e-12 && (sen.hi - 2.0).abs() < 1e-12);
        let mut bad = line.clone();
        bad[5].1 = 1000.0;
        assert!((theil_sen(&bad, 0.05).unwrap().slope - 2.0).abs() < 1e-9);
    }

    #[test]
    fn theil_sen_skips_equal_x_and_refuses_bad_input() {
        // Two samples per x; the zero-dx pairs are skipped.
        let p = [(0.0, 0.0), (0.0, 1.0), (1.0, 2.0), (1.0, 3.0), (2.0, 4.0)];
        assert!(theil_sen(&p, 0.05).is_some());
        let same_x = [(1.0, 0.0), (1.0, 1.0), (1.0, 2.0)];
        assert!(theil_sen(&same_x, 0.05).is_none());
        assert!(theil_sen(&[(0.0, 0.0), (1.0, 1.0)], 0.05).is_none());
        assert!(theil_sen(&[(0.0, 0.0), (1.0, f64::NAN), (2.0, 1.0)], 0.05).is_none());
    }

    #[test]
    fn half_split_uses_shared_middle_point() {
        // n = 5 -> halves of 3 points: [0,1,2] and [2,3,4] (indices).
        let p = pts(&[0.0, 1.0, 2.0, 2.0, 2.0]);
        let h = half_split(&p).unwrap();
        assert!((h.first - 1.0).abs() < 1e-12);
        assert!((h.second - 0.0).abs() < 1e-12);
        assert_eq!(h.ratio, Some(0.0));
        let flat = half_split(&pts(&[1.0, 1.0, 1.0, 1.0])).unwrap();
        assert_eq!(flat.ratio, None);
        assert!(half_split(&pts(&[1.0, 2.0])).is_none());
    }

    #[test]
    fn every_delta_rule() {
        let ok = every_delta(&pts(&[0.0, 1.0, 4.0, 5.0])).unwrap();
        assert!(ok.all_at_least_one);
        assert_eq!((ok.deltas, ok.min, ok.max), (3, 1.0, 3.0));
        let zero = every_delta(&pts(&[0.0, 1.0, 1.0, 2.0])).unwrap();
        assert!(!zero.all_at_least_one && zero.min == 0.0);
        assert!(
            !every_delta(&pts(&[0.0, 0.5, 2.0]))
                .unwrap()
                .all_at_least_one
        );
        assert!(every_delta(&pts(&[1.0])).is_none());
    }

    #[test]
    fn noise_is_scaled_mad_of_residuals() {
        // y = x + e with e = [0, 1, -1, 0, 1, -1, 0]: residual MAD about the median.
        let e = [0.0, 1.0, -1.0, 0.0, 1.0, -1.0, 0.0];
        let p: Vec<(f64, f64)> = e
            .iter()
            .enumerate()
            .map(|(i, e)| (i as f64, i as f64 + e))
            .collect();
        let sen = theil_sen(&p, 0.05).unwrap();
        let resid: Vec<f64> = p
            .iter()
            .map(|&(x, y)| y - sen.intercept - sen.slope * x)
            .collect();
        let med = median(&resid);
        let mad = median(&resid.iter().map(|r| (r - med).abs()).collect::<Vec<_>>());
        assert!((noise(&p, &sen) - 1.4826 * mad).abs() < 1e-12);
        assert!(noise(&p, &sen) > 0.0);
        assert_eq!(noise(&p[..2], &sen), 0.0);
        let line = pts(&[0.0, 1.0, 2.0, 3.0]);
        assert_eq!(noise(&line, &theil_sen(&line, 0.05).unwrap()), 0.0);
    }
}
