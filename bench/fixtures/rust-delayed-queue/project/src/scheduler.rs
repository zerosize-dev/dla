use std::cell::Cell;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::rc::Rc;

use crate::task::Task;

/// Returned by `schedule`; cancelling it stops the task from running.
pub struct Timer {
    cancelled: Rc<Cell<bool>>,
}

struct Entry {
    due: u64,
    seq: u64,
    task: Box<Task>,
    cancelled: Rc<Cell<bool>>,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        (self.due, self.seq) == (other.due, other.seq)
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.due, self.seq).cmp(&(other.due, other.seq))
    }
}

/// Runs tasks in due order against a clock the caller advances.
pub struct Scheduler {
    now: u64,
    seq: u64,
    queue: BinaryHeap<Reverse<Entry>>,
    armed: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler { now: 0, seq: 0, queue: BinaryHeap::new(), armed: 0 }
    }

    pub fn schedule(&mut self, delay: u64, task: Task) -> Timer {
        self.seq += 1;
        let cancelled = Rc::new(Cell::new(false));
        self.queue.push(Reverse(Entry { due: self.now + delay, seq: self.seq, task: Box::new(task), cancelled: Rc::clone(&cancelled) }));
        self.armed += 1;
        Timer { cancelled }
    }

    pub fn cancel(&mut self, timer: &Timer) -> bool {
        if timer.cancelled.replace(true) {
            return false;
        }
        self.armed -= 1;
        true
    }

    /// Moves the clock to `now` and returns the tasks that came due, oldest first.
    pub fn advance(&mut self, now: u64) -> Vec<Task> {
        self.now = now;
        let mut due = Vec::new();
        while let Some(Reverse(top)) = self.queue.peek() {
            if top.due > now {
                break;
            }
            let Reverse(entry) = self.queue.pop().unwrap();
            if entry.cancelled.get() {
                continue;
            }
            entry.cancelled.set(true);
            self.armed -= 1;
            due.push(*entry.task);
        }
        due
    }

    /// Tasks that are scheduled and not cancelled.
    pub fn armed(&self) -> usize {
        self.armed
    }

    #[cfg(test)]
    pub fn queued(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Kind;

    fn task(order: &str) -> Task {
        Task::new(Kind::Remind, order, "c")
    }

    #[test]
    fn tasks_run_in_due_order() {
        let mut s = Scheduler::new();
        s.schedule(30, task("late"));
        s.schedule(10, task("early"));
        let first = s.advance(20);
        assert_eq!(first.iter().map(|t| t.order.as_str()).collect::<Vec<_>>(), ["early"]);
        let second = s.advance(40);
        assert_eq!(second[0].order, "late");
        assert_eq!(s.armed(), 0);
    }

    #[test]
    fn a_cancelled_task_does_not_run() {
        let mut s = Scheduler::new();
        let t = s.schedule(5, task("x"));
        assert!(s.cancel(&t));
        assert!(!s.cancel(&t));
        assert!(s.advance(10).is_empty());
        assert_eq!(s.armed(), 0);
        assert_eq!(s.queued(), 0);
    }

    #[test]
    fn armed_counts_only_live_tasks() {
        let mut s = Scheduler::new();
        let a = s.schedule(5, task("a"));
        s.schedule(6, task("b"));
        s.cancel(&a);
        assert_eq!(s.armed(), 1);
    }
}
