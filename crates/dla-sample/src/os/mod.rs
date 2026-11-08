//! Operating-system sampling of a process: memory, descriptors, threads and container memory.
//!
//! Column names follow the series naming of the DLA contract. Units are bytes for memory and a
//! plain count for descriptors, threads and handles.
//!
//! | Platform | Columns |
//! |---|---|
//! | Linux | `os.rss`, `os.rss_anon`, `os.rss_file`, `os.rss_shmem`, `os.swap`, `os.vm_data`, `os.vm_hwm`, `os.pss`, `os.pss_anon`, `os.private_dirty`, `os.fds`, `os.threads`, and `cgroup.current`, `cgroup.anon`, `cgroup.file` (container scope) |
//! | macOS | `os.rss` (resident size), `os.phys_footprint`, `os.threads`, `os.fds` |
//! | Windows | `os.rss` (working set), `os.vm_hwm` (peak working set), `os.private`, `os.pagefile`, `os.handles`, `os.threads` |
//!
//! A value that cannot be read is absent from [`OsSample::values`] and explained in
//! [`OsSample::limitations`]. It is never reported as zero. The only error is a process that does
//! not exist ([`SampleError::NoSuchProcess`]).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

mod time;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as imp;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as imp;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as imp;

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
mod other;
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
use other as imp;

/// Column holding the resident memory of a process on every platform.
pub const RSS_COLUMN: &str = "os.rss";

/// Prefix of the container-wide columns, which are never summed over a process tree.
const CONTAINER_PREFIX: &str = "cgroup.";

/// Reasons a sample could not be taken.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SampleError {
    /// The process does not exist (or has exited and only awaits reaping).
    #[error("process {0} does not exist")]
    NoSuchProcess(u32),
    /// Process sampling is not implemented for this operating system.
    #[error("process sampling is not supported on this operating system")]
    Unsupported,
    /// An operating-system call failed in a way that is not "no such process".
    #[error("sampling process {pid} failed: {message}")]
    Os {
        /// The process being sampled.
        pid: u32,
        /// What failed.
        message: String,
    },
}

/// One reading of a process.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OsSample {
    /// Milliseconds since the [`Sampler`] started; `0` for a bare [`sample`] call.
    pub t_ms: u64,
    /// Column name to value (bytes for memory, a count for descriptors, threads and handles).
    pub values: BTreeMap<String, f64>,
    /// Why some columns are missing, one entry per cause.
    pub limitations: Vec<String>,
}

impl OsSample {
    pub(crate) fn put(&mut self, column: &str, value: u64) {
        self.values.insert(column.to_owned(), value as f64);
    }

    pub(crate) fn limit(&mut self, text: impl Into<String>) {
        let text = text.into();
        if !self.limitations.contains(&text) {
            self.limitations.push(text);
        }
    }
}

/// Reads the current figures of `pid`.
///
/// Unreadable values are left out and explained in the limitations. Fails only when the process
/// does not exist.
pub fn sample(pid: u32) -> Result<OsSample, SampleError> {
    imp::sample(pid)
}

/// A string identifying when `pid` started (UTC, `YYYY-MM-DDTHH:MM:SSZ`), or `None` when the
/// process does not exist or its start time cannot be read.
///
/// Together with the pid it tells a process from a later one that reuses the pid. Linux derives it
/// from `/proc/<pid>/stat` field 22 and the boot time, macOS from `pbi_start_tvsec`, Windows from
/// `GetProcessTimes`; resolution is one second.
pub fn process_start_time(pid: u32) -> Option<String> {
    imp::process_start_time(pid)
}

/// Whether `pid` is a running process (a zombie awaiting reaping counts as not alive).
pub fn is_alive(pid: u32) -> bool {
    imp::is_alive(pid)
}

/// All transitive child processes of `pid`, sorted, excluding `pid` itself.
pub fn descendants(pid: u32) -> Vec<u32> {
    imp::descendants(pid)
}

/// Sampler settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplerConfig {
    /// Time between samples (the time a sample takes counts towards it).
    pub interval: Duration,
    /// Sum the figures of the process and all its descendants.
    pub tree: bool,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(200),
            tree: false,
        }
    }
}

/// The descendant with the largest resident memory seen during a tree run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LargestDescendant {
    /// Its process id.
    pub pid: u32,
    /// Its largest observed [`RSS_COLUMN`] value in bytes.
    pub rss: u64,
}

/// Everything a [`Sampler`] collected.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SamplerOutput {
    /// The samples in time order (at least the first one).
    pub samples: Vec<OsSample>,
    /// Largest value seen per column.
    pub peaks: BTreeMap<String, f64>,
    /// Union of the limitations of all samples.
    pub limitations: Vec<String>,
    /// Largest descendant by resident memory (tree mode, when any descendant was seen).
    pub largest_descendant: Option<LargestDescendant>,
    /// Whether sampling ended because the process exited (rather than [`Sampler::stop`]).
    pub exited: bool,
    /// An unexpected failure that ended sampling early.
    pub error: Option<String>,
}

/// Samples a process on a background thread until it exits or [`Sampler::stop`] is called.
#[derive(Debug)]
pub struct Sampler {
    stop: Sender<()>,
    handle: JoinHandle<SamplerOutput>,
}

impl Sampler {
    /// Takes a first sample of `pid` and starts the background thread.
    ///
    /// Fails when the first sample fails, which means the process does not exist.
    pub fn start(pid: u32, config: SamplerConfig) -> Result<Self, SampleError> {
        let origin = Instant::now();
        let mut tracker = Tracker::default();
        let first = take(pid, config.tree, &mut tracker, origin)?;
        let (stop, rx) = mpsc::channel();
        let handle = std::thread::Builder::new()
            .name(format!("dla-sample-{pid}"))
            .spawn(move || run(pid, config, origin, first, tracker, rx))
            .map_err(|e| SampleError::Os {
                pid,
                message: format!("cannot start sampler thread: {e}"),
            })?;
        Ok(Self { stop, handle })
    }

    /// Whether the sampling thread is still running (false once the process has exited).
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }

    /// Stops sampling and returns what was collected. Safe to call after the process exited.
    pub fn stop(self) -> SamplerOutput {
        // The thread may already have finished, which closes the channel; that is fine.
        let _ = self.stop.send(());
        self.handle.join().unwrap_or_else(|_| SamplerOutput {
            error: Some("sampler thread panicked".to_owned()),
            ..SamplerOutput::default()
        })
    }
}

/// Takes one OS reading at a time on the caller's thread, for callers that interleave other
/// per-tick work (runtime counters) with the OS sample.
#[derive(Debug)]
pub struct TickSampler {
    pid: u32,
    tree: bool,
    origin: Instant,
    tracker: Tracker,
}

impl TickSampler {
    /// A sampler of `pid` whose `t_ms` counts from `origin`. With `tree`, figures are summed
    /// over the process and its descendants as in [`Sampler`].
    pub fn new(pid: u32, tree: bool, origin: Instant) -> Self {
        Self {
            pid,
            tree,
            origin,
            tracker: Tracker::default(),
        }
    }

    /// Reads the process now. Fails with [`SampleError::NoSuchProcess`] once it has exited.
    pub fn sample(&mut self) -> Result<OsSample, SampleError> {
        take(self.pid, self.tree, &mut self.tracker, self.origin)
    }

    /// The descendant with the largest resident memory seen so far (tree mode).
    pub fn largest_descendant(&self) -> Option<LargestDescendant> {
        self.tracker.largest
    }
}

#[derive(Debug, Default)]
struct Tracker {
    largest: Option<LargestDescendant>,
}

impl Tracker {
    fn observe(&mut self, pid: u32, sample: &OsSample) {
        let Some(&rss) = sample.values.get(RSS_COLUMN) else {
            return;
        };
        let rss = rss as u64;
        if self.largest.is_none_or(|l| rss > l.rss) {
            self.largest = Some(LargestDescendant { pid, rss });
        }
    }
}

fn take(
    pid: u32,
    tree: bool,
    tracker: &mut Tracker,
    origin: Instant,
) -> Result<OsSample, SampleError> {
    let mut total = imp::sample(pid)?;
    if tree {
        for child in imp::descendants(pid) {
            match imp::sample(child) {
                Ok(s) => {
                    tracker.observe(child, &s);
                    merge(&mut total, s);
                }
                // A short-lived child that vanished between the scan and the read.
                Err(SampleError::NoSuchProcess(_)) => {}
                Err(e) => total.limit(e.to_string()),
            }
        }
    }
    total.t_ms = origin.elapsed().as_millis() as u64;
    Ok(total)
}

/// Adds a descendant's figures to the running total. Container figures are shared by every
/// process in the container, so only the root's are kept.
fn merge(total: &mut OsSample, other: OsSample) {
    for (name, value) in other.values {
        if name.starts_with(CONTAINER_PREFIX) {
            continue;
        }
        *total.values.entry(name).or_insert(0.0) += value;
    }
    for text in other.limitations {
        total.limit(text);
    }
}

fn run(
    pid: u32,
    config: SamplerConfig,
    origin: Instant,
    first: OsSample,
    mut tracker: Tracker,
    rx: mpsc::Receiver<()>,
) -> SamplerOutput {
    let mut out = SamplerOutput {
        samples: vec![first],
        ..SamplerOutput::default()
    };
    loop {
        let tick = Instant::now();
        match rx.recv_timeout(config.interval.saturating_sub(tick.elapsed())) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
        }
        match take(pid, config.tree, &mut tracker, origin) {
            Ok(s) => out.samples.push(s),
            Err(SampleError::NoSuchProcess(_)) => {
                out.exited = true;
                break;
            }
            Err(e) => {
                out.error = Some(e.to_string());
                break;
            }
        }
    }
    let mut limitations = BTreeSet::new();
    for s in &out.samples {
        for (name, &value) in &s.values {
            let peak = out.peaks.entry(name.clone()).or_insert(value);
            *peak = peak.max(value);
        }
        limitations.extend(s.limitations.iter().cloned());
    }
    out.limitations = limitations.into_iter().collect();
    out.largest_descendant = tracker.largest;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_sums_process_figures_and_keeps_root_container_figures() {
        let mut a = OsSample::default();
        a.put("os.rss", 100);
        a.put("cgroup.current", 5_000);
        let mut b = OsSample::default();
        b.put("os.rss", 50);
        b.put("os.fds", 3);
        b.put("cgroup.current", 5_000);
        b.limit("x");
        merge(&mut a, b);
        assert_eq!(a.values["os.rss"], 150.0);
        assert_eq!(a.values["os.fds"], 3.0);
        assert_eq!(a.values["cgroup.current"], 5_000.0);
        assert_eq!(a.limitations, ["x"]);
    }
}
