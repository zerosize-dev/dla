use crate::wire;

/// Deterministic stand-in for a link: about one frame in five is a heartbeat or a control frame.
pub struct Traffic {
    state: u64,
}

impl Traffic {
    pub fn new(seed: u64) -> Self {
        Traffic { state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1 }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// The bytes of one packet: a few frames back to back.
    pub fn packet(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        for _ in 0..(2 + self.next() % 4) {
            let roll = self.next() % 10;
            let channel = (self.next() % 64) as u16;
            if roll == 0 {
                out.extend(wire::heartbeat());
            } else if roll == 1 {
                out.extend(wire::control(channel, (self.next() % 8) as u8));
            } else {
                let len = 16 + (self.next() % 900) as usize;
                out.extend(wire::data(channel, &vec![(channel & 0xff) as u8; len]));
            }
        }
        out
    }
}
