//! How long the points of a harness take, and what fits in the time a run has left.
//!
//! `dla harness run` and `dla alloc-test` run a workload many times at growing sizes.
//! The run's deadline (`state.json`) is the only budget the model sees, so these
//! commands estimate their duration from the points already measured (an earlier series
//! of the same harness, or the first points of this run) and refuse up front when the
//! estimate exceeds the time left. A [`Pace`] is that estimate: seconds as a function of
//! the point's size `x`.

/// Fewest extra points `dla alloc-test` can use (its plateau window).
pub const MIN_EXTRA: u32 = 3;

/// Seconds kept back from the deadline for what follows the points (trend, ledger).
pub fn reserve_secs(left_secs: f64) -> f64 {
    20.0_f64.min(left_secs / 10.0)
}

/// Measured `(x, seconds)` pairs and the line through them.
#[derive(Debug, Default, Clone)]
pub struct Pace {
    samples: Vec<(f64, f64)>,
}

impl Pace {
    /// An empty pace: nothing is known, [`Pace::predict`] says 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a measured point of size `x` that took `secs`.
    pub fn push(&mut self, x: f64, secs: f64) {
        if x.is_finite() && secs.is_finite() && x > 0.0 && secs >= 0.0 {
            self.samples.push((x, secs));
        }
    }

    /// Whether no point was measured.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Whether the samples have at least two distinct sizes (a line can be fitted).
    pub fn has_slope(&self) -> bool {
        self.samples
            .iter()
            .any(|(x, _)| (*x - self.samples[0].0).abs() > f64::EPSILON)
    }

    /// Seconds a point of size `x` is expected to take. One size known: proportional
    /// to it (an overestimate when start-up dominates). Two or more: the least-squares
    /// line, with a negative intercept replaced by a line through the origin and never
    /// below the slowest measured point of a smaller size.
    pub fn predict(&self, x: f64) -> f64 {
        let n = self.samples.len() as f64;
        if self.samples.is_empty() {
            return 0.0;
        }
        let through_origin = |s: &[(f64, f64)]| {
            let (sxy, sxx) = s
                .iter()
                .fold((0.0, 0.0), |(a, b), (x, y)| (a + x * y, b + x * x));
            if sxx > 0.0 { sxy / sxx } else { 0.0 }
        };
        let raw = if !self.has_slope() {
            through_origin(&self.samples) * x
        } else {
            let mx = self.samples.iter().map(|s| s.0).sum::<f64>() / n;
            let my = self.samples.iter().map(|s| s.1).sum::<f64>() / n;
            let sxx: f64 = self.samples.iter().map(|s| (s.0 - mx).powi(2)).sum();
            let sxy: f64 = self.samples.iter().map(|s| (s.0 - mx) * (s.1 - my)).sum();
            let slope = if sxx > 0.0 { sxy / sxx } else { 0.0 };
            let intercept = my - slope * mx;
            if slope <= 0.0 {
                // Time does not grow with size: the slowest point so far.
                self.samples.iter().map(|s| s.1).fold(0.0, f64::max)
            } else if intercept < 0.0 {
                through_origin(&self.samples) * x
            } else {
                intercept + slope * x
            }
        };
        raw.max(0.0)
    }

    /// Seconds the points of sizes `xs` are expected to take together.
    pub fn total(&self, xs: impl IntoIterator<Item = f64>) -> f64 {
        xs.into_iter().map(|x| self.predict(x)).sum()
    }
}

/// The largest count in `1..max` whose points (`xs_for(count)`) fit in `usable` seconds.
pub fn largest_count(
    pace: &Pace,
    usable: f64,
    max: u32,
    xs_for: impl Fn(u32) -> Vec<f64>,
) -> Option<u32> {
    (1..max).rev().find(|c| pace.total(xs_for(*c)) <= usable)
}

/// The largest step in `1..step` for which `xs_for(step)` fits in `usable` seconds.
pub fn largest_step(
    pace: &Pace,
    usable: f64,
    step: u64,
    xs_for: impl Fn(u64) -> Vec<f64>,
) -> Option<u64> {
    if step <= 1 || pace.total(xs_for(1)) > usable {
        return None;
    }
    let (mut lo, mut hi) = (1u64, step - 1);
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if pace.total(xs_for(mid)) <= usable {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    Some(lo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_sample_is_proportional() {
        let mut p = Pace::new();
        p.push(100.0, 2.0);
        assert!((p.predict(300.0) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn a_line_with_startup_cost_is_fitted() {
        let mut p = Pace::new();
        p.push(100.0, 3.0);
        p.push(200.0, 4.0);
        assert!((p.predict(1000.0) - 12.0).abs() < 1e-9);
    }

    #[test]
    fn flat_time_predicts_the_slowest_point() {
        let mut p = Pace::new();
        p.push(100.0, 3.0);
        p.push(200.0, 2.0);
        assert!((p.predict(900.0) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn empty_pace_predicts_nothing() {
        assert_eq!(Pace::new().predict(5.0), 0.0);
    }

    #[test]
    fn the_largest_fitting_count_and_step() {
        let mut p = Pace::new();
        p.push(100.0, 1.0);
        let xs = |c: u32| (1..=c).map(|k| f64::from(k) * 100.0).collect::<Vec<_>>();
        // 1 + 2 + 3 + 4 = 10 s for four points.
        assert_eq!(largest_count(&p, 10.0, 9, xs), Some(4));
        assert_eq!(largest_count(&p, 0.5, 9, xs), None);
        let by_step = |s: u64| (1..=4).map(|k| (s * k) as f64).collect::<Vec<_>>();
        // step 100 costs 10 s, step 50 costs 5 s.
        assert_eq!(largest_step(&p, 5.0, 100, by_step), Some(50));
    }
}
