//! DLA loop driver for a crate with a library target. One process, one `checkpoint` record per
//! round (x = units completed), written to the file named by `DLA_HARNESS_OUT`.
//!
//! Fill the two DLA-HOOK bodies below and nothing else; keep the loop and the allocator as they
//! are. The crate under test is `@@DLA_CRATE@@` (its library target).
//!
//! Keys: `DLA_KEY_MODE=fresh` uses key = i (every unit sees a new key); `DLA_KEY_MODE=repeated`
//! uses key = i % repeat (a bounded key space). Rounds: `DLA_WARMUP` warm-up rounds, then
//! `DLA_ROUNDS` measured rounds of `DLA_STEP` units each (defaults are baked in at generation).
//!
//! Build and run from the scratch project root (the driver is `src/bin/dla_harness.rs`):
//!
//! ```text
//! CARGO_PROFILE_RELEASE_DEBUG=line-tables-only cargo build --release --bin dla_harness
//! DLA_HARNESS_OUT=<harness dir>/out/0.jsonl target/release/dla_harness
//! ```
//!
//! Do not preload the allocator shim into this process: the counting allocator below already
//! measures it (the shim stays inert with `DLA_SHIM_DISABLE=1`).
//!
//! dhat-rs variant (only when the `dhat` crate resolves offline): add `dhat` as a dev-dependency,
//! replace `GLOBAL` with `#[global_allocator] static ALLOC: dhat::Alloc = dhat::Alloc;`, start a
//! `dhat::Profiler::builder().testing().build()` in `main` and call
//! `dhat::assert_eq!(dhat::HeapStats::get().curr_blocks, before)` after the last round. dhat
//! replaces the counting allocator, so the `native.*` values below are then unavailable.

#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

/// Counting wrapper over the system allocator: live bytes, live blocks and the peak of live bytes.
struct Counting;

static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE_BLOCKS: AtomicU64 = AtomicU64::new(0);
static PEAK_BYTES: AtomicU64 = AtomicU64::new(0);

fn add(bytes: usize) {
    let now = LIVE_BYTES.fetch_add(bytes as u64, Relaxed) + bytes as u64;
    LIVE_BLOCKS.fetch_add(1, Relaxed);
    PEAK_BYTES.fetch_max(now, Relaxed);
}

fn sub(bytes: usize) {
    LIVE_BYTES.fetch_sub(bytes as u64, Relaxed);
    LIVE_BLOCKS.fetch_sub(1, Relaxed);
}

// SAFETY: every method forwards to `System` unchanged and only updates atomic counters.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            add(layout.size());
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            add(layout.size());
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        sub(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            // Same block, new size: adjust the byte count only.
            let old = layout.size() as u64;
            let new = new_size as u64;
            if new >= old {
                let now = LIVE_BYTES.fetch_add(new - old, Relaxed) + (new - old);
                PEAK_BYTES.fetch_max(now, Relaxed);
            } else {
                LIVE_BYTES.fetch_sub(old - new, Relaxed);
            }
        }
        p
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

const STEP: u64 = @@DLA_STEP@@;
const WARMUP: u64 = @@DLA_WARMUP@@;
const ROUNDS: u64 = @@DLA_ROUNDS@@;
const REPEAT: u64 = @@DLA_REPEAT@@;

/// Values of the run, built once in [`setup`] and used by [`unit`].
#[derive(Default)]
struct State {
    // DLA-HOOK state: add the fields `setup()` fills and `unit()` uses (services, caches, pools)
}

fn setup() -> State {
    // DLA-HOOK setup: use `@@DLA_CRATE@@::...` to build the objects under test once, before the first round, and return them in `State`
    todo!("DLA-HOOK setup has not been filled in")
}

fn unit(state: &mut State, i: u64, key: u64) {
    // DLA-HOOK unit: perform exactly one unit of the workload for `key` (derive request ids/names/payloads from `key` so a repeated key reproduces an earlier request); `i` is the 1-based unit number
    let _ = (state, i, key);
    todo!("DLA-HOOK unit has not been filled in")
}

// DLA-HOOK tokio: only for projects that depend on tokio (tokio >= 1.39, `num_alive_tasks` is
// unverified offline): add `tokio` features `rt` to the scratch Cargo.toml if missing, create or
// reuse the runtime in `setup()`, keep its `Handle` in `State`, and return
// `Some(handle.metrics().num_alive_tasks() as u64)` here. Leave it returning `None` otherwise.
fn tokio_alive_tasks(_state: &State) -> Option<u64> {
    None
}

/// Values read from `/proc/self/status` (bytes; absent when unreadable).
fn os_values() -> Vec<(&'static str, u64)> {
    let mut out = Vec::new();
    let Ok(text) = std::fs::read_to_string("/proc/self/status") else {
        return out;
    };
    let keys: [(&str, &'static str, u64); 8] = [
        ("VmRSS:", "os.rss", 1024),
        ("RssAnon:", "os.rss_anon", 1024),
        ("RssFile:", "os.rss_file", 1024),
        ("RssShmem:", "os.rss_shmem", 1024),
        ("VmSwap:", "os.swap", 1024),
        ("VmData:", "os.vm_data", 1024),
        ("VmHWM:", "os.vm_hwm", 1024),
        ("Threads:", "os.threads", 1),
    ];
    for line in text.lines() {
        for (prefix, name, scale) in keys {
            // Nested rather than a let chain: the driver must build on every edition.
            if let Some(rest) = line.strip_prefix(prefix) {
                if let Some(v) = rest
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                {
                    out.push((name, v * scale));
                }
            }
        }
    }
    out
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn malloc_trim() -> Option<String> {
    unsafe extern "C" {
        fn malloc_trim(pad: usize) -> i32;
    }
    // SAFETY: glibc's `malloc_trim` takes a padding size and has no other preconditions.
    let released = unsafe { malloc_trim(0) };
    Some(format!("malloc_trim(0) -> {released}"))
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn malloc_trim() -> Option<String> {
    None
}

fn json_map(pairs: &[(&str, u64)]) -> String {
    let body: Vec<String> = pairs.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
    format!("{{{}}}", body.join(","))
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn values(state: &State) -> Vec<(&'static str, u64)> {
    let mut v = vec![
        ("native.live_bytes", LIVE_BYTES.load(Relaxed)),
        ("native.live_blocks", LIVE_BLOCKS.load(Relaxed)),
        ("native.peak_bytes", PEAK_BYTES.load(Relaxed)),
    ];
    v.extend(os_values());
    if let Some(n) = tokio_alive_tasks(state) {
        v.push(("tokio.alive_tasks", n));
    }
    v
}

/// Lines for the limitations of every following record: what the driver changed.
static NOTES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

fn note(text: String) {
    // Nested rather than a let chain: the driver must build on every edition.
    if let Ok(mut notes) = NOTES.lock() {
        if !notes.contains(&text) {
            notes.push(text);
        }
    }
}

/// The smallest step the fitting leaves: with fewer units per round the per-round costs and the
/// allocator noise swamp a leak of one entry per unit.
const MIN_STEP: u64 = 10;

/// Time limit of this process in seconds (`dla harness run` sets `DLA_TIME_LIMIT_SECS` from the
/// harness timeout; 0 or unset = no fitting). Once `MIN_STEP` units of a round have run (inside the
/// first round, so one slow round cannot outlast the limit before the first look) and after every
/// round, the step of the round in progress and of the rounds left is lowered when the units left
/// would not fit (margin: 15 percent and 5 seconds), never below `MIN_STEP`. `in_round` units of
/// round `round` have run; `left` rounds follow it.
#[allow(clippy::too_many_arguments)]
fn fit(
    limit: f64,
    started: Instant,
    loop_start: Instant,
    round: u64,
    total: u64,
    in_round: u64,
    done: u64,
    step: &mut u64,
    warned: &mut bool,
) {
    if limit <= 0.0 || done == 0 {
        return;
    }
    let left = (total - round) as f64;
    let per_unit = loop_start.elapsed().as_secs_f64() / done as f64;
    let budget = limit * 0.85 - 5.0 - started.elapsed().as_secs_f64();
    let need = |step: u64| per_unit * ((step - in_round.min(step)) as f64 + step as f64 * left);
    if need(*step) <= budget {
        return;
    }
    let units = (budget / per_unit).max(0.0) as u64;
    let lowered = ((units + in_round) / (left as u64 + 1)).min(*step).max(MIN_STEP.min(*step));
    if lowered < *step {
        note(format!(
            "the driver lowered its step from {} to {lowered} units per round in round {round} (after {in_round} units of it): one unit takes {:.1} ms and the {limit} s time limit would cut the process (the x axis is still the cumulative unit count)",
            *step,
            per_unit * 1000.0
        ));
        *step = lowered;
    }
    if !*warned && need(*step) > budget {
        *warned = true;
        note(format!(
            "the unit is too slow for a series: even {} units per round (the floor) needs {:.0} s for the units left and {:.0} s remain; make the unit cheaper (one call, no sleeps, smaller payloads) or raise timeout_secs in harness.json",
            *step,
            need(*step),
            budget.max(0.0)
        ));
    }
}

/// Appends one `checkpoint` record. On the last round `DLA_PURGE=trim` also trims the malloc heap
/// and records the OS values before and after.
fn checkpoint(state: &State, started: Instant, done: u64, last: bool) {
    let Ok(path) = env::var("DLA_HARNESS_OUT") else {
        return;
    };
    let vals = values(state);
    let mut limitations: Vec<String> = NOTES.lock().map(|n| n.clone()).unwrap_or_default();
    let mut purge = String::new();
    let kind = env::var("DLA_PURGE").unwrap_or_default();
    if last && !kind.is_empty() && kind != "none" {
        if kind == "trim" {
            let before = os_values();
            match malloc_trim() {
                Some(detail) => {
                    let after = os_values();
                    purge = format!(
                        ",\"purge\":{{\"kind\":\"trim\",\"before\":{},\"after\":{},\"detail\":{}}}",
                        json_map(&before),
                        json_map(&after),
                        json_str(&detail)
                    );
                }
                None => limitations.push("purge trim needs glibc malloc_trim".to_string()),
            }
        } else {
            limitations.push(format!(
                "purge '{kind}' is not supported by the rust driver"
            ));
        }
    }
    let argv: Vec<String> = env::args().map(|a| json_str(&a)).collect();
    let exe = env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let lims: Vec<String> = limitations.iter().map(|l| json_str(l)).collect();
    let line = format!(
        "{{\"dla_harness\":1,\"phase\":\"checkpoint\",\"pid\":{},\"argv\":[{}],\"exe\":{},\"runtime\":\"rust\",\"x\":{done},\"t_ms\":{},\"gc_forced\":false,\"values\":{}{purge},\"groups\":{{}},\"limitations\":[{}]}}\n",
        std::process::id(),
        argv.join(","),
        json_str(&exe),
        started.elapsed().as_millis(),
        json_map(&vals),
        lims.join(",")
    );
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}

fn main() {
    let mut step: u64 = env::var("DLA_STEP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(STEP);
    let step_default = step;
    let limit: f64 = env::var("DLA_TIME_LIMIT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0);
    let warmup: u64 = env::var("DLA_WARMUP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(WARMUP);
    let rounds: u64 = env::var("DLA_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(ROUNDS);
    let repeat = env::var("DLA_REPEAT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(REPEAT)
        .max(1);
    let repeated = env::var("DLA_KEY_MODE").is_ok_and(|m| m == "repeated");
    let started = Instant::now();
    let mut state = setup();
    let total = warmup + rounds;
    let loop_start = Instant::now();
    let mut warned = false;
    let mut done = 0;
    for round in 1..=total {
        let mut n = 0;
        while n < step {
            n += 1;
            done += 1;
            let key = if repeated { done % repeat } else { done };
            unit(&mut state, done, key);
            if n == MIN_STEP.min(step_default) {
                fit(limit, started, loop_start, round, total, n, done, &mut step, &mut warned);
            }
        }
        checkpoint(&state, started, done, round == total);
        if round < total {
            fit(limit, started, loop_start, round + 1, total, 0, done, &mut step, &mut warned);
        }
    }
    // The process ends when `main` returns: threads and servers of the units do not hold it.
}
