/// A seat held for a party until it is confirmed or released.
#[derive(Debug, Clone, PartialEq)]
pub struct Hold {
    pub seat: u32,
    pub party: String,
    pub note: String,
    pub confirmed: bool,
}

impl Hold {
    pub fn new(seat: u32, party: &str, note: &str) -> Self {
        Hold { seat, party: party.to_string(), note: note.to_string(), confirmed: false }
    }

    pub fn summary(&self) -> String {
        let state = if self.confirmed { "confirmed" } else { "pending" };
        format!("seat {} for {} ({})", self.seat, self.party, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_hold_is_pending() {
        let hold = Hold::new(12, "Okafor", "window");
        assert!(!hold.confirmed);
        assert_eq!(hold.summary(), "seat 12 for Okafor (pending)");
    }
}
