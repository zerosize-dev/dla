mod journal;
mod ledger;
mod outbox;
mod source;
mod sync;

const WAVE: u64 = 100;

fn waves_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: ledger-sync [--iterations N]");
                std::process::exit(2);
            }),
        None => 500,
    }
}

fn main() {
    let batches = waves_from_args() * WAVE;
    let mut sync = sync::Sync::new(8);
    let mut net_total = 0i64;
    for _ in 0..batches {
        net_total += sync.run_once();
    }
    let lines = sync.finish();
    let totals = sync.totals(net_total, lines);
    println!(
        "batches={} accounts={} net={} last_flush={} bytes_written={}",
        totals.batches, totals.accounts, totals.net, totals.lines_written, totals.bytes_written
    );
}
