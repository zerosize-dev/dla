#![allow(dead_code)]

mod feed;
mod link;
mod outbox;
mod pool;

use std::sync::Arc;

use feed::Feed;
use link::Link;
use pool::Pool;

const WORKERS: usize = 4;
const WAVE: usize = 240;

fn iterations_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: relay-desk [--iterations N]");
                std::process::exit(2);
            }),
        None => 1000,
    }
}

fn main() {
    let iterations = iterations_from_args();
    let link = Arc::new(Link::new(512));
    link.attach();
    let mut pool = Pool::start(WORKERS, Arc::clone(&link));
    let mut feed = Feed::new(11);
    let mut raised = 0usize;

    for _ in 0..iterations {
        // Each wave is split between the workers in four batches.
        for _ in 0..WORKERS {
            raised += pool.deliver(feed.batch(WAVE / WORKERS));
        }
        // The uplink takes what the wave left in the queue.
        link.collect();
    }

    println!(
        "waves={} raised={} delivered={} waiting={}",
        iterations,
        raised,
        link.delivered(),
        link.waiting()
    );
    // The workers belong to the process: it ends here, whatever they still hold.
    std::process::exit(0);
}
