/// What a timer does when it comes due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Cancel an order nobody paid for.
    Expire,
    /// Nudge the customer about a pending payment.
    Remind,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub kind: Kind,
    pub order: String,
    pub customer: String,
    pub message: String,
}

impl Task {
    pub fn new(kind: Kind, order: &str, customer: &str) -> Self {
        let message = match kind {
            Kind::Expire => format!("order {order} for {customer} was not paid in time"),
            Kind::Remind => format!("order {order} for {customer} is waiting for payment"),
        };
        Task { kind, order: order.to_string(), customer: customer.to_string(), message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_name_the_order() {
        let t = Task::new(Kind::Remind, "A-17", "Kovacs");
        assert_eq!(t.message, "order A-17 for Kovacs is waiting for payment");
        let t = Task::new(Kind::Expire, "A-18", "Dubois");
        assert!(t.message.contains("A-18") && t.message.contains("not paid"));
    }
}
