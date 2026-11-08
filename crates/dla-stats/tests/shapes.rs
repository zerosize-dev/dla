//! Table-driven shape and gate tests over synthetic series with known answers.

use dla_stats::{
    Analysis, GateThresholds, GateVerdict, TrendShape, analyze, classify, gate, gate_counter,
};

/// Deterministic pseudo-noise in `[-1, 1)` (xorshift), so tests never flake.
fn noise_seq(n: usize, seed: u64) -> Vec<f64> {
    let mut s = seed | 1;
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            ((s >> 11) as f64 / (1_u64 << 53) as f64) * 2.0 - 1.0
        })
        .collect()
}

fn pts(ys: &[f64]) -> Vec<(f64, f64)> {
    ys.iter().enumerate().map(|(i, &y)| (i as f64, y)).collect()
}

fn series(n: usize, f: impl Fn(usize) -> f64) -> Vec<f64> {
    (0..n).map(f).collect()
}

fn run(ys: &[f64]) -> Analysis {
    analyze(&pts(ys), None, false, &GateThresholds::default())
}

struct Case {
    name: &'static str,
    ys: Vec<f64>,
    shape: TrendShape,
    gate: GateVerdict,
}

fn cases() -> Vec<Case> {
    use GateVerdict::{Fail, Pass};
    use TrendShape::*;
    let nz = noise_seq(40, 7);
    let mut out = Vec::new();
    let mut add = |name, ys: Vec<f64>, shape, gate| {
        out.push(Case {
            name,
            ys,
            shape,
            gate,
        })
    };

    for n in [8, 12, 20, 30] {
        add(
            "linear",
            series(n, |i| 100.0 + 5.0 * i as f64),
            Steady,
            Pass,
        );
    }
    add(
        "linear+noise",
        series(30, |i| 100.0 + 5.0 * i as f64 + 3.0 * nz[i]),
        Steady,
        Pass,
    );
    add(
        "linear+noise n=12",
        series(12, |i| 50.0 + 4.0 * i as f64 + 2.0 * nz[i]),
        Steady,
        Pass,
    );
    add(
        "exponential approach",
        series(24, |i| 1000.0 * (1.0 - (-(i as f64) / 3.0).exp())),
        Plateau,
        Fail,
    );
    add(
        "single early step",
        series(16, |i| if i < 3 { 100.0 } else { 200.0 }),
        Plateau,
        Fail,
    );
    add(
        "repeated steps",
        series(24, |i| 100.0 + 50.0 * (i / 6) as f64),
        Stair,
        Pass,
    );
    add(
        "spike",
        series(20, |i| if i == 9 { 500.0 } else { 100.0 + nz[i] }),
        Spike,
        Fail,
    );
    add(
        "flat noise",
        series(30, |i| 100.0 + 2.0 * nz[i]),
        Flat,
        Fail,
    );
    add("constant", vec![42.0; 15], Flat, Fail);
    add(
        "declining",
        series(20, |i| 500.0 - 7.0 * i as f64 + nz[i]),
        Declining,
        Fail,
    );
    add(
        "sawtooth",
        series(24, |i| 100.0 + 10.0 * (i % 4) as f64),
        Flat,
        Fail,
    );
    add("too short", series(5, |i| 10.0 * i as f64), Steady, Fail);
    add("n=3 line", vec![1.0, 2.0, 3.0], Steady, Fail);
    add("n=2", vec![1.0, 2.0], Insufficient, Fail);
    add("empty", vec![], Insufficient, Fail);
    out
}

#[test]
fn shapes_and_gate_table() {
    for c in cases() {
        let a = run(&c.ys);
        assert_eq!(a.shape, c.shape, "{}: shape (analysis {a:?})", c.name);
        assert_eq!(
            a.gate, c.gate,
            "{}: gate (reasons {:?})",
            c.name, a.gate_failed
        );
        assert_eq!(
            a.gate == GateVerdict::Pass,
            a.gate_failed.is_empty(),
            "{}: reasons mirror verdict",
            c.name
        );
        // classify and gate agree with analyze.
        let p = pts(&c.ys);
        let th = GateThresholds::default();
        assert_eq!(classify(&p, &th), a.shape, "{}", c.name);
        assert_eq!(gate(&p, &th).0, a.gate, "{}", c.name);
    }
}

#[test]
fn every_length_from_3_to_30_is_consistent() {
    let th = GateThresholds::default();
    for n in 3..=30 {
        let ys = series(n, |i| 10.0 + 2.0 * i as f64);
        let a = analyze(&pts(&ys), None, false, &th);
        assert_eq!(a.n as usize, n);
        assert!(a.mk.is_some() && a.sen.is_some() && a.halves.is_some());
        assert_eq!(a.shape, TrendShape::Steady, "n={n}");
        let expect = if n >= 8 {
            GateVerdict::Pass
        } else {
            GateVerdict::Fail
        };
        assert_eq!(a.gate, expect, "n={n}: {:?}", a.gate_failed);
        assert!((a.rise - 2.0 * (n - 1) as f64).abs() < 1e-9);
        if n < 8 {
            assert!(a.gate_failed.iter().any(|r| r == "n < 8"));
        }
    }
}

#[test]
fn gate_reasons_name_the_failure() {
    let flat = run(&series(20, |i| 100.0 + 2.0 * noise_seq(20, 3)[i]));
    assert!(
        flat.gate_failed.iter().any(|r| r.starts_with("p >=")),
        "{:?}",
        flat.gate_failed
    );
    let plateau = run(&series(24, |i| 1000.0 * (1.0 - (-(i as f64) / 3.0).exp())));
    assert!(
        plateau
            .gate_failed
            .iter()
            .any(|r| r == "decaying half-split")
    );
}

#[test]
fn all_ties_do_not_pass() {
    let a = run(&[5.0; 12]);
    assert_eq!(a.shape, TrendShape::Flat);
    assert_eq!(a.gate, GateVerdict::Fail);
    assert_eq!(a.mk.unwrap().var_s, 0.0);
    assert_eq!(a.noise, 0.0);
}

#[test]
fn late_onset_growth_passes_when_not_decaying() {
    // Flat for the first half, then growing: first-half slope is 0, second positive.
    let ys = series(20, |i| {
        if i < 10 {
            100.0
        } else {
            100.0 + 10.0 * (i - 9) as f64
        }
    });
    let a = run(&ys);
    assert_eq!(a.gate, GateVerdict::Pass, "{:?}", a.gate_failed);
}

fn counter_run(ys: &[f64], tail: Option<(f64, f64)>) -> Analysis {
    analyze(&pts(ys), tail, true, &GateThresholds::default())
}

#[test]
fn counters_follow_the_every_delta_rule() {
    let plus1 = series(10, |i| 5.0 + i as f64);
    let a = counter_run(&plus1, None);
    assert_eq!(a.gate, GateVerdict::Pass, "{:?}", a.gate_failed);
    let ed = a.every_delta.unwrap();
    assert!(ed.all_at_least_one && ed.deltas == 9 && ed.min == 1.0);

    let plus3 = series(10, |i| 3.0 * i as f64);
    assert_eq!(counter_run(&plus3, None).gate, GateVerdict::Pass);

    let mut zero = plus1.clone();
    zero[6] = zero[5];
    for v in &mut zero[7..] {
        *v -= 1.0;
    }
    let a = counter_run(&zero, None);
    assert_eq!(a.gate, GateVerdict::Fail);
    assert_eq!(a.gate_failed, vec!["delta < 1".to_string()]);

    // Too few points even though every delta is >= 1.
    let a = counter_run(&plus1[..5], None);
    assert_eq!(a.gate_failed, vec!["n < 8".to_string()]);

    // Gauges never carry every_delta.
    assert!(run(&plus1).every_delta.is_none());
}

#[test]
fn counter_warmup_tail_is_part_of_the_deltas() {
    let ys = series(9, |i| 100.0 + i as f64);
    // Tail one below the first measured point: delta 1 -> still passes.
    let ok = counter_run(&ys, Some((-1.0, 99.0)));
    assert_eq!(ok.gate, GateVerdict::Pass);
    assert_eq!(ok.every_delta.unwrap().deltas, 9);
    // Tail equal to the first point: a zero delta crosses the warm-up boundary.
    let bad = counter_run(&ys, Some((-1.0, 100.0)));
    assert_eq!(bad.gate, GateVerdict::Fail);
    let (v, why) = gate_counter(&pts(&ys), Some((-1.0, 100.0)), &GateThresholds::default());
    assert_eq!((v, why), (GateVerdict::Fail, vec!["delta < 1".to_string()]));
}

#[test]
fn non_finite_input_is_refused() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut ys = series(12, |i| i as f64);
        ys[4] = bad;
        let a = run(&ys);
        assert_eq!(a.shape, TrendShape::Insufficient);
        assert_eq!(a.gate, GateVerdict::Fail);
        assert_eq!(a.gate_failed, vec!["non-finite value".to_string()]);
        assert!(a.mk.is_none() && a.sen.is_none());
    }
}

#[test]
fn duplicate_x_values_are_tolerated() {
    let p: Vec<(f64, f64)> = (0..20)
        .map(|i| ((i / 2) as f64, 10.0 * (i / 2) as f64))
        .collect();
    let a = analyze(&p, None, false, &GateThresholds::default());
    assert!((a.sen.unwrap().slope - 10.0).abs() < 1e-9);
    // All x equal: no slope at all.
    let same: Vec<(f64, f64)> = (0..9).map(|i| (1.0, i as f64)).collect();
    let a = analyze(&same, None, false, &GateThresholds::default());
    assert_eq!(a.shape, TrendShape::Insufficient);
    assert_eq!(a.gate, GateVerdict::Fail);
}

#[test]
fn unsorted_x_is_sorted_by_analyze() {
    let mut p = pts(&series(12, |i| 3.0 * i as f64));
    p.reverse();
    let a = analyze(&p, None, false, &GateThresholds::default());
    assert_eq!(a.shape, TrendShape::Steady);
}
