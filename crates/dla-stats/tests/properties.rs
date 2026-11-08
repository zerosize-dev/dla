//! Property tests: invariance under shifting and positive scaling of the measured values.

use dla_stats::{GateThresholds, analyze, mann_kendall, theil_sen};
use proptest::prelude::*;

fn pts(ys: &[f64]) -> Vec<(f64, f64)> {
    ys.iter().enumerate().map(|(i, &y)| (i as f64, y)).collect()
}

/// Values on a grid of exactly representable numbers so shifts and scales are exact.
fn series() -> impl Strategy<Value = Vec<f64>> {
    prop::collection::vec((-64_i32..64).prop_map(|v| f64::from(v) * 8.0), 3..30)
}

proptest! {
    #[test]
    fn shift_and_scale_keep_gate_and_shape(ys in series(), shift in -32_i32..32, scale in 1_u32..16) {
        let th = GateThresholds::default();
        let base = analyze(&pts(&ys), None, false, &th);
        let shift = f64::from(shift) * 8.0;
        let scale = f64::from(scale);
        let shifted: Vec<f64> = ys.iter().map(|y| y + shift).collect();
        let scaled: Vec<f64> = ys.iter().map(|y| y * scale).collect();
        let s = analyze(&pts(&shifted), None, false, &th);
        let m = analyze(&pts(&scaled), None, false, &th);
        prop_assert_eq!(s.gate, base.gate);
        prop_assert_eq!(m.gate, base.gate);
        prop_assert_eq!(s.shape, base.shape);
        prop_assert_eq!(m.shape, base.shape);
        prop_assert_eq!(s.mk.map(|k| k.s), base.mk.map(|k| k.s));
        prop_assert_eq!(m.mk.map(|k| k.s), base.mk.map(|k| k.s));
    }

    #[test]
    fn slope_scales_linearly_and_ignores_shift(ys in series(), shift in -32_i32..32, scale in 1_u32..16) {
        let base = theil_sen(&pts(&ys), 0.05).unwrap();
        let k = f64::from(scale);
        let shifted: Vec<f64> = ys.iter().map(|y| y + f64::from(shift)).collect();
        let scaled: Vec<f64> = ys.iter().map(|y| y * k).collect();
        let s = theil_sen(&pts(&shifted), 0.05).unwrap();
        let m = theil_sen(&pts(&scaled), 0.05).unwrap();
        let tol = 1e-9 * (1.0 + base.slope.abs() * k);
        prop_assert!((s.slope - base.slope).abs() <= tol);
        prop_assert!((m.slope - k * base.slope).abs() <= tol);
        prop_assert!((m.lo - k * base.lo).abs() <= tol && (m.hi - k * base.hi).abs() <= tol);
        prop_assert!((s.lo - base.lo).abs() <= tol && (s.hi - base.hi).abs() <= tol);
    }

    #[test]
    fn slope_scales_with_x(ys in series(), scale in 1_u32..16) {
        let k = f64::from(scale);
        let base = theil_sen(&pts(&ys), 0.05).unwrap();
        let stretched: Vec<(f64, f64)> = ys.iter().enumerate().map(|(i, &y)| (i as f64 * k, y)).collect();
        let m = theil_sen(&stretched, 0.05).unwrap();
        prop_assert!((m.slope * k - base.slope).abs() <= 1e-9 * (1.0 + base.slope.abs()));
    }

    #[test]
    fn reversing_negates_s_and_p_is_a_probability(ys in series()) {
        let fwd = mann_kendall(&ys).unwrap();
        let mut r = ys.clone();
        r.reverse();
        let rev = mann_kendall(&r).unwrap();
        prop_assert_eq!(fwd.s, -rev.s);
        prop_assert!((0.0..=1.0).contains(&fwd.p_increasing));
        prop_assert!(fwd.tau.abs() <= 1.0);
    }
}
