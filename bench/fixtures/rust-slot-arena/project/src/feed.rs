/// Deterministic stand-in for the booking requests.
pub struct Feed {
    state: u64,
}

const PARTIES: [&str; 8] =
    ["Okafor", "Lindqvist", "Mbeki", "Tanaka", "Rossi", "Haddad", "Novak", "Silva"];
const NOTES: [&str; 4] = ["aisle", "window", "front row", "accessible"];

impl Feed {
    pub fn new(seed: u64) -> Self {
        Feed { state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1 }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn seat(&mut self) -> u32 {
        (self.next() % 5000) as u32
    }

    pub fn party(&mut self) -> &'static str {
        PARTIES[(self.next() % PARTIES.len() as u64) as usize]
    }

    pub fn note(&mut self) -> &'static str {
        NOTES[(self.next() % NOTES.len() as u64) as usize]
    }

    pub fn confirms(&mut self) -> bool {
        self.next() % 3 == 0
    }
}
