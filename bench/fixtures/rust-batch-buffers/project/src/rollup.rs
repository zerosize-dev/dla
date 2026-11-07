//! Collects the samples of one batch and reduces them to a summary.

use crate::sample::Sample;
use crate::stats::SeriesStats;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Summary {
    pub samples: usize,
    pub series: usize,
    pub busiest: String,
    pub busiest_count: u32,
    pub hottest_mean: f64,
}

pub struct Rollup {
    samples: Vec<Sample>,
    stats: HashMap<String, SeriesStats>,
    pub batches: u64,
}

impl Rollup {
    pub fn new() -> Rollup {
        Rollup { samples: Vec::new(), stats: HashMap::new(), batches: 0 }
    }

    pub fn push(&mut self, sample: Sample) {
        self.samples.push(sample);
    }

    /// Reduces everything pushed since the last flush and gets ready for the next batch.
    pub fn flush(&mut self) -> Summary {
        for sample in &self.samples {
            self.stats.entry(sample.series.clone()).or_default().add(sample.value);
        }

        let mut busiest = String::new();
        let mut busiest_count = 0;
        let mut hottest_mean = f64::NEG_INFINITY;
        for (name, stats) in &self.stats {
            // Ties on the count go to the smaller name so the result is repeatable.
            if stats.count > busiest_count || (stats.count == busiest_count && *name < busiest) {
                busiest = name.clone();
                busiest_count = stats.count;
            }
            hottest_mean = hottest_mean.max(stats.mean());
        }

        let summary = Summary {
            samples: self.samples.len(),
            series: self.stats.len(),
            busiest,
            busiest_count,
            hottest_mean: if self.stats.is_empty() { 0.0 } else { hottest_mean },
        };

        self.samples.clear();
        self.stats.clear();
        self.batches += 1;
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(series: &str, value: f64) -> Sample {
        Sample { series: series.to_string(), timestamp: 0, value }
    }

    #[test]
    fn summarises_a_batch() {
        let mut rollup = Rollup::new();
        rollup.push(sample("a", 1.0));
        rollup.push(sample("b", 4.0));
        rollup.push(sample("a", 3.0));
        let summary = rollup.flush();
        assert_eq!(summary.samples, 3);
        assert_eq!(summary.series, 2);
        assert_eq!(summary.busiest, "a");
        assert_eq!(summary.hottest_mean, 4.0);
    }

    #[test]
    fn batches_do_not_see_each_other() {
        let mut rollup = Rollup::new();
        rollup.push(sample("a", 1.0));
        rollup.flush();
        rollup.push(sample("b", 2.0));
        let summary = rollup.flush();
        assert_eq!((summary.samples, summary.series), (1, 1));
        assert_eq!(rollup.batches, 2);
    }
}
