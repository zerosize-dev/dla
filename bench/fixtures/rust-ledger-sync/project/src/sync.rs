use crate::ledger::Ledger;
use crate::outbox::{Outbox, Statement};
use crate::source::Source;

pub struct Sync {
    source: Source,
    ledger: Ledger,
    outbox: Outbox,
    sink: Vec<u8>,
    batches: u64,
}

pub struct Totals {
    pub batches: u64,
    pub accounts: usize,
    pub net: i64,
    pub lines_written: usize,
    pub bytes_written: usize,
}

impl Sync {
    pub fn new(seed: u64) -> Self {
        Sync {
            source: Source::new(seed),
            ledger: Ledger::new(),
            outbox: Outbox::new(),
            sink: Vec::new(),
            batches: 0,
        }
    }

    /// Pulls one batch from the bank, applies it and queues its statement lines.
    pub fn run_once(&mut self) -> i64 {
        let batch = self.source.next_batch();
        let net = self.ledger.apply(&batch);
        self.batches += 1;
        let postings = batch.postings();
        self.outbox.queue(Statement {
            batch: batch.id,
            postings: postings.len(),
            net,
            accounts: self.ledger.accounts(),
            first_ref: postings[0].reference,
            last_ref: postings[postings.len() - 1].reference,
        });
        if self.batches % 50 == 0 {
            self.sink.clear();
            let _ = self.outbox.flush(&mut self.sink);
        }
        net
    }

    pub fn totals(&self, net: i64, lines: usize) -> Totals {
        Totals {
            batches: self.batches,
            accounts: self.ledger.accounts(),
            net,
            lines_written: lines,
            bytes_written: self.outbox.bytes_written(),
        }
    }

    pub fn finish(&mut self) -> usize {
        self.sink.clear();
        self.outbox.flush(&mut self.sink).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_run_applies_one_batch() {
        let mut sync = Sync::new(2);
        sync.run_once();
        sync.run_once();
        assert_eq!(sync.batches, 2);
        assert_eq!(sync.ledger.applied(), 2);
    }

    #[test]
    fn the_outbox_is_flushed_every_fifty_batches() {
        let mut sync = Sync::new(3);
        for _ in 0..50 {
            sync.run_once();
        }
        assert_eq!(sync.outbox.waiting(), 0);
        assert!(!sync.sink.is_empty());
        sync.run_once();
        assert_eq!(sync.outbox.waiting(), 1);
        assert_eq!(sync.finish(), 1);
    }
}
