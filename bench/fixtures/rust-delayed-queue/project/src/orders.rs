use std::collections::HashMap;

use crate::scheduler::{Scheduler, Timer};
use crate::task::{Kind, Task};

/// Unpaid orders are cancelled after thirty days.
const PAYMENT_WINDOW: u64 = 30 * 24 * 3600 * 1000;
const REMINDER_DELAY: u64 = 200;

struct Open {
    customer: String,
    expiry: Timer,
}

pub struct Orders {
    scheduler: Scheduler,
    open: HashMap<String, Open>,
    cancelled: u64,
    reminded: u64,
}

impl Orders {
    pub fn new() -> Self {
        Orders { scheduler: Scheduler::new(), open: HashMap::new(), cancelled: 0, reminded: 0 }
    }

    pub fn place(&mut self, order: &str, customer: &str) {
        let expiry = self.scheduler.schedule(PAYMENT_WINDOW, Task::new(Kind::Expire, order, customer));
        self.scheduler.schedule(REMINDER_DELAY, Task::new(Kind::Remind, order, customer));
        self.open.insert(order.to_string(), Open { customer: customer.to_string(), expiry });
    }

    /// Records a payment. Returns the customer, or `None` for an unknown order.
    pub fn pay(&mut self, order: &str) -> Option<String> {
        let open = self.open.remove(order)?;
        self.scheduler.cancel(&open.expiry);
        Some(open.customer)
    }

    pub fn tick(&mut self, now: u64) {
        for task in self.scheduler.advance(now) {
            match task.kind {
                Kind::Expire => {
                    if self.is_open_for(&task) {
                        self.open.remove(&task.order);
                        self.cancelled += 1;
                    }
                }
                Kind::Remind => {
                    if self.is_open_for(&task) {
                        self.reminded += 1;
                    }
                }
            }
        }
    }

    fn is_open_for(&self, task: &Task) -> bool {
        self.open.get(&task.order).is_some_and(|o| o.customer == task.customer)
    }

    pub fn awaiting_payment(&self) -> usize {
        self.open.len()
    }

    pub fn timers_armed(&self) -> usize {
        self.scheduler.armed()
    }

    pub fn reminded(&self) -> u64 {
        self.reminded
    }

    pub fn cancelled(&self) -> u64 {
        self.cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_before_the_reminder_sends_none() {
        let mut o = Orders::new();
        o.place("A-1", "Kovacs");
        assert_eq!(o.pay("A-1").as_deref(), Some("Kovacs"));
        o.tick(1000);
        assert_eq!(o.reminded(), 0);
        assert_eq!(o.awaiting_payment(), 0);
    }

    #[test]
    fn an_unpaid_order_is_reminded_then_cancelled() {
        let mut o = Orders::new();
        o.place("A-2", "Dubois");
        o.tick(500);
        assert_eq!(o.reminded(), 1);
        assert_eq!(o.awaiting_payment(), 1);
        o.tick(PAYMENT_WINDOW + 1);
        assert_eq!(o.cancelled(), 1);
        assert_eq!(o.awaiting_payment(), 0);
        assert_eq!(o.timers_armed(), 0);
    }

    #[test]
    fn paying_an_unknown_order_is_refused() {
        let mut o = Orders::new();
        assert_eq!(o.pay("nope"), None);
    }
}
