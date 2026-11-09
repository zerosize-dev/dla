mod feed;
mod registry;
mod session;
mod window;

use feed::Feed;
use registry::Registry;
use session::Session;

fn iterations_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: meter-gateway [--iterations N]");
                std::process::exit(2);
            }),
        None => 1000,
    }
}

fn main() {
    let iterations = iterations_from_args();
    let mut feed = Feed::new(3);
    let mut registry = Registry::default();
    let mut readings = 0u64;
    let mut mean_total = 0i64;
    let mut last_device = String::new();

    for n in 0..iterations {
        let session = Session::open(&mut registry, feed.device(n));
        for _ in 0..40 {
            registry.dispatch(feed.next_reading());
            readings += 1;
        }
        mean_total += session.mean().unwrap_or(0);
        last_device = session.device().to_string();
        session.close(&mut registry);
    }

    println!(
        "sessions={} readings={} mean_total={} active={} last={}",
        iterations,
        readings,
        mean_total,
        registry.len(),
        last_device
    );
}
