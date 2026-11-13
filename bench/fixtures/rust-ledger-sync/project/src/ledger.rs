use std::collections::HashMap;

use crate::journal::Batch;

/// Account balances in cents.
#[derive(Default)]
pub struct Ledger {
    balances: HashMap<u32, i64>,
    applied: u64,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies every posting of the batch and returns its net amount.
    pub fn apply(&mut self, batch: &Batch) -> i64 {
        for posting in batch.postings() {
            *self.balances.entry(posting.account).or_insert(0) += posting.cents;
        }
        self.applied += 1;
        batch.net()
    }

    pub fn balance(&self, account: u32) -> i64 {
        self.balances.get(&account).copied().unwrap_or(0)
    }

    pub fn accounts(&self) -> usize {
        self.balances.len()
    }

    pub fn applied(&self) -> u64 {
        self.applied
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balances_follow_the_postings() {
        let mut ledger = Ledger::new();
        let batch = Batch::new(1);
        batch.add(10, 700, 1);
        batch.add(10, -250, 2);
        batch.add(11, 40, 3);
        assert_eq!(ledger.apply(&batch), 490);
        assert_eq!(ledger.balance(10), 450);
        assert_eq!(ledger.balance(11), 40);
        assert_eq!(ledger.balance(12), 0);
        assert_eq!(ledger.accounts(), 2);
        assert_eq!(ledger.applied(), 1);
    }
}
