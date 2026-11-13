use crate::journal::Batch;
use std::rc::Rc;

/// Deterministic stand-in for the bank's batch feed.
pub struct Source {
    state: u64,
    serial: u64,
}

impl Source {
    pub fn new(seed: u64) -> Self {
        Source { state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1, serial: 0 }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Next batch: four to seven postings over a pool of 2000 accounts.
    pub fn next_batch(&mut self) -> Rc<Batch> {
        self.serial += 1;
        let batch = Batch::new(self.serial);
        let count = 4 + self.next() % 4;
        for _ in 0..count {
            let account = (self.next() % 2000) as u32;
            let cents = (self.next() % 90_000) as i64 - 30_000;
            let reference = (self.next() % 1_000_000) as u32;
            batch.add(account, cents, reference);
        }
        batch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batches_have_four_to_seven_postings() {
        let mut source = Source::new(1);
        for _ in 0..50 {
            let n = source.next_batch().len();
            assert!((4..=7).contains(&n), "{n}");
        }
    }

    #[test]
    fn the_feed_is_repeatable() {
        let a = Source::new(4).next_batch().net();
        let b = Source::new(4).next_batch().net();
        assert_eq!(a, b);
    }
}
