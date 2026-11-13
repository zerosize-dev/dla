mod feed;
mod orders;
mod scheduler;
mod task;

use std::collections::HashMap;

use feed::Feed;
use orders::Orders;

const TICK_MS: u64 = 7;
const WAVE: u64 = 200;

fn waves_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: order-timers [--iterations N]");
                std::process::exit(2);
            }),
        None => 1000,
    }
}

fn main() {
    let total = waves_from_args() * WAVE;
    let mut feed = Feed::new(5);
    let mut orders = Orders::new();
    let mut payments: HashMap<u64, Vec<String>> = HashMap::new();
    let mut paid = 0u64;
    let mut now = 0u64;

    for tick in 0..total {
        now += TICK_MS;
        let id = feed.order_id();
        orders.place(&id, feed.customer());
        payments.entry(tick + feed.payment_lag()).or_default().push(id);
        for id in payments.remove(&tick).unwrap_or_default() {
            if orders.pay(&id).is_some() {
                paid += 1;
            }
        }
        orders.tick(now);
    }

    println!(
        "orders={} paid={} reminded={} cancelled={} awaiting={} timers={}",
        total,
        paid,
        orders.reminded(),
        orders.cancelled(),
        orders.awaiting_payment(),
        orders.timers_armed()
    );

    let status = if orders.cancelled() == 0 { 0 } else { 2 };
    std::process::exit(status);
}
