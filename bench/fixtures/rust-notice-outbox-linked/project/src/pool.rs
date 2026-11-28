use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use crate::feed::Notice;
use crate::link::Link;
use crate::outbox;

enum Order {
    Deliver(Vec<Notice>),
}

/// Worker threads that live as long as the process: each takes batches from its own channel and
/// sends every notice through its own outbox.
pub struct Pool {
    workers: Vec<Sender<Order>>,
    done: Receiver<usize>,
    turn: usize,
}

impl Pool {
    pub fn start(threads: usize, link: Arc<Link>) -> Pool {
        let (done_tx, done) = channel();
        let mut workers = Vec::new();
        for _ in 0..threads {
            let (tx, rx) = channel::<Order>();
            let link = Arc::clone(&link);
            let done_tx = done_tx.clone();
            thread::spawn(move || {
                for order in rx {
                    match order {
                        Order::Deliver(batch) => {
                            let n = batch.len();
                            for notice in batch {
                                outbox::send(&link, notice);
                            }
                            let _ = done_tx.send(n);
                        }
                    }
                }
            });
            workers.push(tx);
        }
        Pool { workers, done, turn: 0 }
    }

    /// Hands a batch to the next worker and waits until it has been sent.
    pub fn deliver(&mut self, batch: Vec<Notice>) -> usize {
        let at = self.turn % self.workers.len();
        self.turn += 1;
        self.workers[at].send(Order::Deliver(batch)).unwrap();
        self.done.recv().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::Feed;

    #[test]
    fn every_batch_is_sent_before_deliver_returns() {
        let link = Arc::new(Link::new(1000));
        let mut pool = Pool::start(3, Arc::clone(&link));
        let mut feed = Feed::new(6);
        let mut total = 0;
        for _ in 0..6 {
            total += pool.deliver(feed.batch(10));
        }
        assert_eq!(total, 60);
        assert_eq!(link.waiting(), 60);
    }
}
