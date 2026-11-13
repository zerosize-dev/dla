//! A batch of postings received from the bank, and the postings it contains.

use std::cell::RefCell;
use std::rc::Rc;

pub struct Batch {
    pub id: u64,
    postings: RefCell<Vec<Rc<Posting>>>,
}

pub struct Posting {
    pub account: u32,
    pub cents: i64,
    pub reference: u32,
    batch: Rc<Batch>,
}

impl Batch {
    pub fn new(id: u64) -> Rc<Batch> {
        Rc::new(Batch { id, postings: RefCell::new(Vec::new()) })
    }

    /// Records a posting in this batch; the posting can name its batch for reconciliation.
    pub fn add(self: &Rc<Batch>, account: u32, cents: i64, reference: u32) -> Rc<Posting> {
        let posting = Rc::new(Posting { account, cents, reference, batch: Rc::clone(self) });
        self.postings.borrow_mut().push(Rc::clone(&posting));
        posting
    }

    pub fn postings(&self) -> Vec<Rc<Posting>> {
        self.postings.borrow().clone()
    }

    pub fn net(&self) -> i64 {
        self.postings.borrow().iter().map(|p| p.cents).sum()
    }

    pub fn len(&self) -> usize {
        self.postings.borrow().len()
    }
}

impl Posting {
    pub fn batch_id(&self) -> u64 {
        self.batch.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_batch_sums_its_postings() {
        let batch = Batch::new(7);
        batch.add(1, 500, 11);
        batch.add(2, -200, 12);
        assert_eq!(batch.net(), 300);
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn a_posting_knows_its_batch() {
        let batch = Batch::new(9);
        let posting = batch.add(3, 100, 13);
        assert_eq!(posting.batch_id(), 9);
        assert_eq!(batch.postings()[0].reference, 13);
    }
}
