mod arena;
mod desk;
mod feed;
mod hold;

use std::collections::VecDeque;

use desk::Desk;
use feed::Feed;

const WAITING_ROOM: usize = 300;
const WAVE: u64 = 500;

fn waves_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: booking-desk [--iterations N]");
                std::process::exit(2);
            }),
        None => 1000,
    }
}

fn main() {
    let waves = waves_from_args();
    let mut feed = Feed::new(11);
    let mut desk = Desk::new();
    let mut waiting = VecDeque::new();
    let (mut placed, mut refused, mut confirmed, mut released) = (0u64, 0u64, 0u64, 0u64);

    for _ in 0..waves * WAVE {
        let seat = feed.seat();
        match desk.place(seat, feed.party(), feed.note()) {
            Ok(handle) => {
                placed += 1;
                if feed.confirms() && desk.confirm(handle).is_ok() {
                    confirmed += 1;
                }
                waiting.push_back(handle);
            }
            Err(_) => refused += 1,
        }
        while waiting.len() > WAITING_ROOM {
            let oldest = waiting.pop_front().unwrap();
            if desk.release(oldest).is_ok() {
                released += 1;
            }
        }
    }

    println!(
        "requests={} placed={} refused={} confirmed={} released={} active={}",
        waves * WAVE,
        placed,
        refused,
        confirmed,
        released,
        desk.active()
    );

    let status = if desk.active() <= WAITING_ROOM { 0 } else { 1 };
    std::process::exit(status);
}
