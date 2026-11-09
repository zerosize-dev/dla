use crate::window::Device;

/// Deterministic stand-in for the device drivers.
pub struct Feed {
    state: u64,
}

impl Feed {
    pub fn new(seed: u64) -> Self {
        Feed { state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1 }
    }

    pub fn next_reading(&mut self) -> i64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state % 10_000) as i64
    }

    pub fn device(&mut self, n: u64) -> Device {
        Device { name: format!("meter-{:05}-{}", n % 4000, self.next_reading() % 7) }
    }
}
