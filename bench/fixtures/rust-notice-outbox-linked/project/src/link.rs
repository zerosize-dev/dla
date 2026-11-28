use std::collections::VecDeque;
use std::sync::Mutex;

use crate::feed::Notice;

/// The hand-off between the desk's worker threads and the uplink: a queue of a fixed size that
/// the uplink empties when it is attached.
pub struct Link {
    slots: Mutex<Queue>,
    room: usize,
}

struct Queue {
    waiting: VecDeque<Notice>,
    attached: bool,
    delivered: u64,
}

impl Link {
    pub fn new(room: usize) -> Self {
        Link {
            slots: Mutex::new(Queue {
                waiting: VecDeque::with_capacity(room),
                attached: false,
                delivered: 0,
            }),
            room,
        }
    }

    /// Opens the uplink: from now on `collect` takes what is waiting.
    pub fn attach(&self) {
        self.slots.lock().unwrap().attached = true;
    }

    /// Offers a notice; it comes back when the queue has no room.
    pub fn offer(&self, n: Notice) -> Result<(), Notice> {
        let mut q = self.slots.lock().unwrap();
        if q.waiting.len() >= self.room {
            return Err(n);
        }
        q.waiting.push_back(n);
        Ok(())
    }

    /// Takes everything waiting to the uplink. Nothing moves while it is not attached.
    pub fn collect(&self) -> usize {
        let mut q = self.slots.lock().unwrap();
        if !q.attached {
            return 0;
        }
        let n = q.waiting.len();
        q.waiting.clear();
        q.delivered += n as u64;
        n
    }

    pub fn delivered(&self) -> u64 {
        self.slots.lock().unwrap().delivered
    }

    pub fn waiting(&self) -> usize {
        self.slots.lock().unwrap().waiting.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::Feed;

    #[test]
    fn a_full_queue_hands_the_notice_back() {
        let link = Link::new(2);
        let mut feed = Feed::new(1);
        assert!(link.offer(feed.next_notice()).is_ok());
        assert!(link.offer(feed.next_notice()).is_ok());
        assert!(link.offer(feed.next_notice()).is_err());
        assert_eq!(link.waiting(), 2);
    }

    #[test]
    fn collect_moves_nothing_before_attach_and_everything_after() {
        let link = Link::new(4);
        let mut feed = Feed::new(2);
        link.offer(feed.next_notice()).unwrap();
        assert_eq!(link.collect(), 0);
        link.attach();
        assert_eq!(link.collect(), 1);
        assert_eq!(link.delivered(), 1);
    }
}
