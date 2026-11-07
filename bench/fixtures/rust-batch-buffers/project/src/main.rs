mod feed;
mod rollup;
mod sample;
mod stats;

use rollup::Rollup;
use sample::Sample;

fn batches_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: rollup-agent [--iterations N]");
                std::process::exit(2);
            }),
        None => 500,
    }
}

fn main() {
    let batches = batches_from_args();
    let mut rollup = Rollup::new();
    let mut samples = 0usize;
    let mut series = 0usize;
    let mut rejected = 0usize;
    let mut peak_batch = 0usize;
    let mut hottest = 0.0f64;

    for seq in 0..batches {
        for line in feed::batch(seq) {
            match Sample::parse(&line) {
                Ok(sample) => rollup.push(sample),
                Err(_) => rejected += 1,
            }
        }
        let summary = rollup.flush();
        samples += summary.samples;
        series += summary.series;
        peak_batch = peak_batch.max(summary.samples);
        hottest = hottest.max(summary.hottest_mean);
    }

    println!(
        "batches={} samples={} series={} rejected={} largest_batch={} hottest_mean={:.2}",
        rollup.batches, samples, series, rejected, peak_batch, hottest
    );
}
