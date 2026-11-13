/// Deterministic stand-in for the checkout traffic.
pub struct Feed {
    state: u64,
    serial: u64,
}

const CUSTOMERS: [&str; 6] = ["Kovacs", "Dubois", "Adeyemi", "Nilsson", "Ferrari", "Yilmaz"];

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

    pub fn order_id(&mut self) -> String {
        self.serial += 1;
        format!("A-{:07}", self.serial)
    }

    pub fn customer(&mut self) -> &'static str {
        CUSTOMERS[(self.next() % CUSTOMERS.len() as u64) as usize]
    }

    /// Ticks between an order and its payment.
    pub fn payment_lag(&mut self) -> u64 {
        1 + self.next() % 60
    }
}
