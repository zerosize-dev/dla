mod parser;
mod section;
mod service;
mod source;
mod toc;

use service::Service;

fn iterations_from_args() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--iterations") {
        Some(i) => args
            .get(i + 1)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| {
                eprintln!("usage: outline-service [--iterations N]");
                std::process::exit(2);
            }),
        None => 1000,
    }
}

fn main() {
    let iterations = iterations_from_args();
    let mut service = Service::default();
    let mut headings = 0usize;
    let mut body_lines = 0usize;
    let mut toc_bytes = 0usize;
    let mut deepest = 0usize;

    for seq in 0..iterations {
        let text = source::document(seq);
        match service.handle(&text) {
            Ok(response) => {
                headings += response.headings;
                body_lines += response.body_lines;
                toc_bytes += response.toc_bytes;
                deepest = deepest.max(response.deepest);
            }
            Err(err) => eprintln!("document {seq}: {err}"),
        }
    }

    println!(
        "documents={} rejected={} headings={} body_lines={} toc_bytes={} deepest={}",
        service.handled, service.rejected, headings, body_lines, toc_bytes, deepest
    );
}
