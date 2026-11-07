//! Deterministic wire input for exercising the agent.

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const METRICS: [&str; 4] = ["cpu", "mem", "disk", "net"];

/// Number of samples in batch `seq`. Most batches carry a few hundred to a few thousand
/// readings; batch 11 is the catch-up burst that follows a collector restart.
pub fn batch_len(seq: u64) -> usize {
    if seq == 11 {
        return 60_000;
    }
    let mut rng = Lcg(seq ^ 0x9e37_79b9);
    300 + rng.below(5_700) as usize
}

/// The wire lines of batch `seq`.
pub fn batch(seq: u64) -> Vec<String> {
    let mut rng = Lcg(seq.wrapping_mul(40503).wrapping_add(1));
    let hosts = 40_000;
    (0..batch_len(seq))
        .map(|i| {
            let host = rng.below(hosts);
            let metric = METRICS[rng.below(METRICS.len() as u64) as usize];
            let value = rng.below(10_000) as f64 / 100.0;
            format!("host-{host}.{metric} {} {value}", 1_700_000_000 + seq * 60 + (i as u64 % 60))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batches_are_repeatable_and_sized_as_announced() {
        assert_eq!(batch(5), batch(5));
        assert_eq!(batch(5).len(), batch_len(5));
        assert_eq!(batch(11).len(), 60_000);
    }
}
