//! Running statistics for one series within a batch.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeriesStats {
    pub count: u32,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

impl Default for SeriesStats {
    fn default() -> Self {
        SeriesStats { count: 0, sum: 0.0, min: f64::INFINITY, max: f64::NEG_INFINITY }
    }
}

impl SeriesStats {
    pub fn add(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_extremes_and_mean() {
        let mut s = SeriesStats::default();
        for v in [2.0, 8.0, 5.0] {
            s.add(v);
        }
        assert_eq!((s.count, s.min, s.max), (3, 2.0, 8.0));
        assert_eq!(s.mean(), 5.0);
    }

    #[test]
    fn empty_stats_have_zero_mean() {
        assert_eq!(SeriesStats::default().mean(), 0.0);
    }
}
