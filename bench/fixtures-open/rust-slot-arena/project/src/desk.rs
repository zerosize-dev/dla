use std::collections::HashMap;

use crate::arena::{Arena, Handle};
use crate::hold::Hold;

/// The booking desk: one hold per seat at a time.
pub struct Desk {
    holds: Arena<Hold>,
    by_seat: HashMap<u32, Handle>,
}

#[derive(Debug, PartialEq)]
pub enum Refusal {
    SeatTaken,
    NoSuchHold,
}

impl Desk {
    pub fn new() -> Self {
        Desk { holds: Arena::new(), by_seat: HashMap::new() }
    }

    pub fn place(&mut self, seat: u32, party: &str, note: &str) -> Result<Handle, Refusal> {
        if self.by_seat.contains_key(&seat) {
            return Err(Refusal::SeatTaken);
        }
        let handle = self.holds.insert(party, Hold::new(seat, party, note));
        self.by_seat.insert(seat, handle);
        Ok(handle)
    }

    pub fn confirm(&mut self, handle: Handle) -> Result<String, Refusal> {
        let hold = self.holds.get_mut(handle).ok_or(Refusal::NoSuchHold)?;
        hold.confirmed = true;
        Ok(hold.summary())
    }

    pub fn release(&mut self, handle: Handle) -> Result<Hold, Refusal> {
        let hold = self.holds.remove(handle).ok_or(Refusal::NoSuchHold)?;
        self.by_seat.remove(&hold.seat);
        Ok(hold)
    }

    pub fn holder_of(&self, seat: u32) -> Option<&str> {
        let handle = self.by_seat.get(&seat)?;
        self.holds.get(*handle).map(|h| h.party.as_str())
    }

    pub fn active(&self) -> usize {
        self.holds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seat_has_one_hold() {
        let mut desk = Desk::new();
        desk.place(4, "Lindqvist", "aisle").unwrap();
        assert_eq!(desk.place(4, "Mbeki", ""), Err(Refusal::SeatTaken));
        assert_eq!(desk.holder_of(4), Some("Lindqvist"));
    }

    #[test]
    fn release_frees_the_seat() {
        let mut desk = Desk::new();
        let handle = desk.place(9, "Tanaka", "").unwrap();
        let hold = desk.release(handle).unwrap();
        assert_eq!(hold.party, "Tanaka");
        assert_eq!(desk.holder_of(9), None);
        assert_eq!(desk.release(handle).unwrap_err(), Refusal::NoSuchHold);
        assert!(desk.place(9, "Rossi", "").is_ok());
    }

    #[test]
    fn confirm_marks_the_hold() {
        let mut desk = Desk::new();
        let handle = desk.place(2, "Haddad", "").unwrap();
        assert_eq!(desk.confirm(handle).unwrap(), "seat 2 for Haddad (confirmed)");
        assert_eq!(desk.active(), 1);
    }
}
