//! Standard normal distribution helpers without external crates.
//!
//! `erfc` is evaluated with two classical expansions, both accurate to about 1e-14 in
//! double precision:
//!
//! - for `|x| < 2.5` the Maclaurin series of `erf`, `erf(x) = 2/sqrt(pi) * sum_n (-1)^n
//!   x^(2n+1) / (n! (2n+1))`, and `erfc = 1 - erf`;
//! - for `x >= 2.5` the continued fraction `erfc(x) = exp(-x^2)/sqrt(pi) * 1/(x + (1/2)/(x +
//!   1/(x + (3/2)/(x + 2/(x + ...)))))`, evaluated bottom-up with a fixed depth.
//!
//! Negative arguments use `erfc(-x) = 2 - erfc(x)`. The quantile function inverts the
//! survival function by bisection, which is slow but exact enough (1e-12) and has no
//! coefficient tables to get wrong.

use std::f64::consts::{FRAC_2_SQRT_PI, PI, SQRT_2};

/// Complementary error function.
pub fn erfc(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x < 0.0 {
        return 2.0 - erfc(-x);
    }
    if x < 2.5 {
        // Maclaurin series of erf.
        let x2 = x * x;
        let mut term = x; // (-1)^n x^(2n+1) / n!
        let mut sum = x;
        let mut n = 0.0_f64;
        while term.abs() > 1e-17 * sum.abs().max(1e-300) && n < 200.0 {
            n += 1.0;
            term *= -x2 / n;
            sum += term / (2.0 * n + 1.0);
        }
        1.0 - FRAC_2_SQRT_PI * sum
    } else if x > 27.0 {
        0.0
    } else {
        let mut frac = x;
        for k in (1..=60).rev() {
            frac = x + (f64::from(k) / 2.0) / frac;
        }
        (-x * x).exp() / (PI.sqrt() * frac)
    }
}

/// Standard normal cumulative distribution function, `P(Z <= z)`.
pub fn norm_cdf(z: f64) -> f64 {
    0.5 * erfc(-z / SQRT_2)
}

/// Standard normal survival function, `P(Z > z)`, accurate in the upper tail.
pub fn norm_sf(z: f64) -> f64 {
    0.5 * erfc(z / SQRT_2)
}

/// Standard normal quantile: the `z` with `P(Z <= z) = p`, for `0 < p < 1`.
///
/// Returns `NaN` outside the open unit interval.
pub fn norm_quantile(p: f64) -> f64 {
    if !(p > 0.0 && p < 1.0) {
        return f64::NAN;
    }
    // Work on the smaller tail for accuracy; the survival function is monotone decreasing.
    if p < 0.5 {
        return -norm_quantile(1.0 - p);
    }
    let target = 1.0 - p;
    let (mut lo, mut hi) = (0.0_f64, 40.0_f64);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if norm_sf(mid) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdf_matches_known_values() {
        let table = [
            (0.0, 0.5),
            (1.0, 0.841_344_746_068_542_9),
            (-1.0, 0.158_655_253_931_457_07),
            (1.96, 0.975_002_104_851_779_5),
            (3.0, 0.998_650_101_968_369_9),
            (-2.0, 0.022_750_131_948_179_195),
            (0.5, 0.691_462_461_274_013_1),
        ];
        for (z, want) in table {
            assert!((norm_cdf(z) - want).abs() < 1e-9, "cdf({z})");
        }
    }

    #[test]
    fn tails_are_accurate_relatively() {
        assert!((norm_sf(5.0) / 2.866_515_718_791_939e-7 - 1.0).abs() < 1e-8);
        assert!((norm_cdf(-6.0) / 9.865_876_450_376_946e-10 - 1.0).abs() < 1e-8);
        assert!((norm_sf(2.4999) + norm_cdf(-2.4999) - 2.0 * norm_sf(2.4999)).abs() < 1e-15);
    }

    #[test]
    fn series_and_fraction_agree_at_the_switch() {
        let below = erfc(2.5 - 1e-12);
        let above = erfc(2.5 + 1e-12);
        assert!((below - above).abs() < 1e-12);
        assert!((erfc(2.5) - 4.069_520_174_449_589e-4).abs() < 1e-14);
    }

    #[test]
    fn quantile_inverts_cdf() {
        assert!((norm_quantile(0.975) - 1.959_963_984_540_054).abs() < 1e-9);
        assert!((norm_quantile(0.5)).abs() < 1e-9);
        assert!((norm_quantile(0.025) + 1.959_963_984_540_054).abs() < 1e-9);
        assert!(norm_quantile(0.0).is_nan());
        assert!(norm_quantile(1.0).is_nan());
    }
}
