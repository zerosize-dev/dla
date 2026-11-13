mod feed;
mod job;
mod pool;
mod render;

use feed::Feed;
use pool::Pool;

const WORKERS: usize = 8;
const WAVE: usize = 48;

fn jobs_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: export-workers [--iterations N]");
                std::process::exit(2);
            }),
        None => 2000,
    }
}

fn main() {
    let total = jobs_from_args();
    let mut feed = Feed::new(21);
    let pool = Pool::start(WORKERS);
    let (mut jobs, mut bytes, mut digest) = (0u64, 0u64, 0u64);

    while jobs < total {
        let wave = WAVE.min((total - jobs) as usize);
        for _ in 0..wave {
            pool.submit(feed.next_job());
        }
        for outcome in pool.collect(wave) {
            jobs += 1;
            bytes += outcome.bytes as u64;
            digest ^= outcome.checksum;
        }
    }

    println!("jobs={} bytes={} digest={:016x}", jobs, bytes, digest);
    let status = if jobs == total { 0 } else { 1 };
    std::process::exit(status);
}
