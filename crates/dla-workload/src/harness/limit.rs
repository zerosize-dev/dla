//! How long one `dla harness run` of a test loop may take, and the processes an earlier command
//! of the same harness left behind.
//!
//! The agent that drives the tool may run one foreground command for ten minutes at most, and a
//! command that is cut leaves its measured process running. So the whole command of a test-loop
//! harness (the pytest loop and the JUnit repeat harness) ends by itself within
//! `DLA_COMMAND_SECS` (default 420 s). From that number the tool gives the loop its process
//! limit: the command minus the start-up and reading time around the process, and, with
//! `--capture`, minus the time the capture may take. The loop fits its rounds into the limit
//! it gets (`DLA_TIME_LIMIT_SECS`), so the limit has to be honest.

use std::time::{Duration, Instant};

use dla_core::{
    harness::HarnessManifest,
    proc,
    state::{ProcessKind, ProcessRole, RunState},
    toolctx::ToolCtx,
};

use super::capture::CaptureSet;

/// Default seconds one `dla harness run` of a test loop may take, end to end.
pub const COMMAND_SECS_DEFAULT: u64 = 420;
/// Seconds kept for what happens around the measured process: the start of `dla`, reading the
/// ledger, the build marker and the dump records, writing the series and the evidence. Measured
/// on a small project: under 1 s; the rest is room for a large dump file.
pub const START_UP_SECS: u64 = 10;
/// Seconds one allocation profile may take on a large heap (a snapshot of every traced block,
/// grouped by stack and written). Measured on a small project: under 0.2 s.
pub const PROFILE_SECS: u64 = 4;
/// Time bound of the retention capture when `DLA_HEAP_SECS` is not set (the agent's default).
pub const HEAP_SECS_DEFAULT: f64 = 60.0;
/// The loop never gets less than this share of the process (a third), so a large capture
/// allowance cannot leave it nothing.
const LOOP_SHARE_MIN: u64 = 3;

/// Seconds the whole command may take: `DLA_COMMAND_SECS`, else 420.
pub fn command_secs() -> u64 {
    std::env::var("DLA_COMMAND_SECS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(COMMAND_SECS_DEFAULT)
}

/// The time bound of the retention capture, as the agent reads it (`DLA_HEAP_SECS`).
pub fn heap_secs() -> f64 {
    std::env::var("DLA_HEAP_SECS")
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
        .unwrap_or(HEAP_SECS_DEFAULT)
}

/// What one measured process of a test loop gets.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// Seconds the command may take (`DLA_COMMAND_SECS`).
    pub command: u64,
    /// Seconds kept for start-up and reading.
    pub startup: u64,
    /// Seconds kept for the capture the process makes (0 without `--capture`).
    pub capture: u64,
    /// Seconds before the process is killed.
    pub process: u64,
    /// Seconds the loop fits its rounds into (`DLA_TIME_LIMIT_SECS`).
    pub loop_secs: u64,
}

impl Plan {
    /// The plan of the next process: `started` is when the command began, `modes_left` the key
    /// modes still to run (each gets an equal share), `cap` the longest the manifest or the
    /// caller allows a process, and `time_left` what the run's own deadline leaves.
    pub fn of(
        started: Instant,
        modes_left: usize,
        capture: &CaptureSet,
        dumps: u32,
        python: bool,
        cap: Duration,
        time_left: Option<f64>,
    ) -> Self {
        let command = command_secs();
        let left = command.saturating_sub(started.elapsed().as_secs());
        let share = left / modes_left.max(1) as u64;
        let mut process = share.saturating_sub(START_UP_SECS).max(1);
        process = process.min(cap.as_secs().max(1));
        if let Some(t) = time_left {
            process = process.min((t.floor() as u64).max(1));
        }
        let capture = Self::capture_secs(capture, dumps, python);
        // A capture that does not fit leaves the loop a third of the process at least.
        let loop_secs = process
            .saturating_sub(capture)
            .max(process / LOOP_SHARE_MIN)
            .max(1);
        Self {
            command,
            startup: START_UP_SECS,
            capture,
            process,
            loop_secs,
        }
    }

    /// Seconds the capture of `set` may take after the last round: the retention walk
    /// (`DLA_HEAP_SECS`) and the allocation profiles.
    pub fn capture_secs(set: &CaptureSet, dumps: u32, python: bool) -> u64 {
        if set.is_empty() {
            return 0;
        }
        let mut secs = 0.0;
        if python {
            if set.heap {
                secs += heap_secs();
            }
            if set.alloc || set.histogram || set.nonheap {
                secs += (PROFILE_SECS * u64::from(dumps.max(1))) as f64;
            }
        } else {
            secs += (PROFILE_SECS * u64::from(dumps.max(1))) as f64;
        }
        secs.ceil() as u64
    }

    /// The sentence that says once: this command ends by itself.
    pub fn ends_by_itself(&self) -> String {
        format!(
            "this command ends by itself within {} minutes (DLA_COMMAND_SECS={}); do not wrap it in timeout",
            self.command.div_ceil(60),
            self.command
        )
    }

    /// The one `_attention` entry that says what the loop shortened and why, from the notes
    /// its process wrote; `None` when it shortened nothing.
    pub fn attention(&self, limitations: &[String]) -> Option<String> {
        let mut what: Vec<String> = Vec::new();
        for l in limitations {
            if let Some(rest) = l.strip_prefix("the test loop cut its rounds from ") {
                let nums: Vec<&str> = rest.split(' ').collect();
                if let (Some(a), Some(b)) = (nums.first(), nums.get(2)) {
                    what.push(format!("rounds {a} to {b}"));
                }
            } else if let Some(rest) = l.strip_prefix("the test loop lowered its step from ") {
                let nums: Vec<&str> = rest.split(' ').collect();
                if let (Some(a), Some(b)) = (nums.first(), nums.get(2)) {
                    what.push(format!("step {a} to {b}"));
                }
            }
        }
        let dropped = limitations
            .iter()
            .filter(|l| l.starts_with("dropped test file ") && l.contains("would not fit"))
            .count();
        if dropped > 0 {
            what.push(format!("{dropped} slowest test file(s) dropped"));
        }
        if what.is_empty() {
            return None;
        }
        what.dedup();
        let capture = if self.capture > 0 {
            format!(" and {} s capture", self.capture)
        } else {
            String::new()
        };
        Some(format!(
            "SHORTENED TO FIT {} s: the loop got {} s ({} s start-up{capture} kept back): {}",
            self.command,
            self.loop_secs,
            self.startup,
            what.join(", ")
        ))
    }
}

/// Stops the processes an earlier command of this harness left running (a command that was cut
/// by a timeout leaves its measured process behind), by process group after the recorded start
/// time matched the live process, never by name. Returns one sentence per process stopped or
/// not stopped.
pub fn stop_leftovers(ctx: &ToolCtx, manifest: &HarnessManifest) -> Vec<String> {
    let Ok(state) = RunState::load(&ctx.run_dir) else {
        return Vec::new();
    };
    // The text of the command up to its first placeholder names the harness's program and
    // driver file; a record of another harness never starts with it.
    let joined = manifest.run.argv.join(" ");
    let prefix = joined.split('{').next().unwrap_or("").trim_end();
    if prefix.is_empty() {
        return Vec::new();
    }
    let mut notes = Vec::new();
    for p in &state.processes {
        if p.stopped_at.is_some()
            || p.role != ProcessRole::Started
            || p.kind != ProcessKind::Harness
            || p.worker != ctx.worker
            || p.pid == std::process::id()
            || !p.command.starts_with(prefix)
            || !proc::is_alive(p.pid)
        {
            continue;
        }
        let live = dla_sample::os::process_start_time(p.pid);
        if !proc::identity_matches(p, live.as_deref()) {
            continue;
        }
        match proc::stop_started(p.pid) {
            Ok(()) => {
                let _ = ctx.mark_process_stopped(p.pid, p.start_time.as_deref());
                notes.push(format!(
                    "stopped process {} that an earlier {} command left running",
                    p.pid, manifest.id
                ));
            }
            Err(why) => notes.push(format!("could not stop the leftover process: {why}")),
        }
    }
    notes
}
