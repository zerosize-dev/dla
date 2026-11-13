use crate::job::Job;

/// Deterministic stand-in for the export requests: two to six sections of 6 to 126 kB.
pub struct Feed {
    state: u64,
    serial: u64,
}

impl Feed {
    pub fn new(seed: u64) -> Self {
        Feed { state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1, serial: 0 }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn next_job(&mut self) -> Job {
        self.serial += 1;
        let sections = 2 + (self.next() % 5) as usize;
        let sizes = (0..sections).map(|_| 6_000 + (self.next() % 120_000) as usize).collect();
        Job { id: self.serial, sections: sizes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_have_two_to_six_sections() {
        let mut feed = Feed::new(1);
        for _ in 0..100 {
            let n = feed.next_job().sections.len();
            assert!((2..=6).contains(&n));
        }
    }
}
