use std::cell::RefCell;
use std::sync::Arc;

use crate::feed::Notice;
use crate::link::Link;

/// How many notices the overflow grows by when it is full: fewer reallocations.
const CHUNK: usize = 4096;

/// What one worker thread holds back when the link has no room for its notices.
pub struct Outbox {
    overflow: Vec<Notice>,
    sent: u64,
}

thread_local! {
    static OUTBOX: RefCell<Outbox> = const { RefCell::new(Outbox::new()) };
}

impl Outbox {
    const fn new() -> Self {
        Outbox { overflow: Vec::new(), sent: 0 }
    }

    fn put(&mut self, link: &Arc<Link>, n: Notice) {
        // Older notices first: what was held back goes before the new one.
        while let Some(front) = self.overflow.last().copied() {
            match link.offer(front) {
                Ok(()) => {
                    self.overflow.pop();
                    self.sent += 1;
                }
                Err(_) => break,
            }
        }
        match link.offer(n) {
            Ok(()) => self.sent += 1,
            Err(n) => self.hold(n),
        }
    }

    fn hold(&mut self, n: Notice) {
        let more = if self.overflow.len() == self.overflow.capacity() { CHUNK } else { 0 };
        self.overflow.reserve_exact(more); self.overflow.push(n);
    }
}

/// Sends a notice from the calling thread through its own outbox.
pub fn send(link: &Arc<Link>, n: Notice) {
    OUTBOX.with(|o| o.borrow_mut().put(link, n));
}

/// Notices the calling thread holds back and notices it has handed over.
pub fn held_back() -> (usize, u64) {
    OUTBOX.with(|o| {
        let o = o.borrow();
        (o.overflow.len(), o.sent)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::Feed;

    #[test]
    fn notices_go_straight_to_the_link_while_it_has_room() {
        let link = Arc::new(Link::new(8));
        let mut feed = Feed::new(3);
        for _ in 0..5 {
            send(&link, feed.next_notice());
        }
        assert_eq!(held_back(), (0, 5));
        assert_eq!(link.waiting(), 5);
    }

    #[test]
    fn a_full_link_makes_the_thread_hold_notices_back() {
        let link = Arc::new(Link::new(2));
        let mut feed = Feed::new(4);
        for _ in 0..6 {
            send(&link, feed.next_notice());
        }
        assert_eq!(held_back(), (4, 2));
        link.attach();
        link.collect();
        send(&link, feed.next_notice());
        assert_eq!(held_back().0, 3);
    }
}
