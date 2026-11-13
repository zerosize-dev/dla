mod frames;
mod relay;
mod traffic;
mod wire;

use relay::Relay;
use traffic::Traffic;

fn packets_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: parcel-relay [--iterations N]");
                std::process::exit(2);
            }),
        None => 1000,
    }
}

fn main() {
    let packets = packets_from_args();
    let mut traffic = Traffic::new(6);
    let mut relay = Relay::new(4096);
    let mut frames = 0usize;

    for _ in 0..packets {
        let packet = traffic.packet();
        match relay.feed(&packet) {
            Ok(n) => frames += n,
            Err(e) => {
                eprintln!("link error: {e}");
                std::process::exit(1);
            }
        }
    }

    let bytes: u64 = (0..64).map(|c| relay.delivered(c)).sum();
    println!(
        "packets={} frames={} bytes={} controls={} heartbeats={}",
        packets,
        frames,
        bytes,
        relay.controls(),
        relay.heartbeats()
    );
}
