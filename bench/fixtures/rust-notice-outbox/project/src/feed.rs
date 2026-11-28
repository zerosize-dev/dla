/// One alert raised by a field station.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Notice {
    pub id: u64,
    pub station: u32,
    pub level: u8,
    pub readings: [f64; 4],
}

/// Deterministic stand-in for the stations' radio links.
pub struct Feed {
    state: u64,
    next_id: u64,
}

impl Feed {
    pub fn new(seed: u64) -> Self {
        Feed {
            state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            next_id: 1,
        }
    }

    fn step(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn next_notice(&mut self) -> Notice {
        let id = self.next_id;
        self.next_id += 1;
        let r = self.step();
        Notice {
            id,
            station: (r % 640) as u32,
            level: (r >> 20) as u8 % 5,
            readings: [
                (r % 1000) as f64 / 10.0,
                ((r >> 8) % 1000) as f64 / 10.0,
                ((r >> 16) % 1000) as f64 / 10.0,
                ((r >> 24) % 1000) as f64 / 10.0,
            ],
        }
    }

    pub fn batch(&mut self, n: usize) -> Vec<Notice> {
        (0..n).map(|_| self.next_notice()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_consecutive_and_the_stream_is_repeatable() {
        let mut a = Feed::new(5);
        let mut b = Feed::new(5);
        let xs = a.batch(10);
        assert_eq!(xs, b.batch(10));
        assert_eq!(xs[0].id, 1);
        assert_eq!(xs[9].id, 10);
    }

    #[test]
    fn stations_stay_in_range() {
        let mut f = Feed::new(9);
        assert!(f.batch(500).iter().all(|n| n.station < 640 && n.level < 5));
    }
}
