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
/// Seconds the last allocation profile may take (a snapshot of every traced block, grouped by
/// stack and written). Measured on a small project (3 test files of 3 tests, pytest loaded):
/// 0.55 s with one frame per block and 11 s with the ten frames `--capture alloc` keeps, plus
/// about 7 s for the rest of that checkpoint. The loop measures the checkpoints it has written
/// and counts them for the rounds left; this is the room for the last one.
pub const PROFILE_SECS: u64 = 12;
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
    /// caller allows a process.
    pub fn of(
        started: Instant,
        modes_left: usize,
        capture: &CaptureSet,
        dumps: u32,
        python: bool,
        cap: Duration,
    ) -> Option<Self> {
        if !Self::applies(capture, std::env::var_os("DLA_COMMAND_SECS").is_some()) {
            return None;
        }
        Some(Self::compute(
            command_secs(),
            heap_secs(),
            started.elapsed().as_secs(),
            modes_left,
            (capture, dumps, python),
            cap,
        ))
    }

    /// Whether the command's own bound holds for a run: with `--capture` always (the agent's tool
    /// cuts a long command, and the capture is what such a command was cut in), without it only
    /// when `DLA_COMMAND_SECS` is set. A plain loop keeps the limit of its manifest: on a large
    /// library it takes longer than the command bound, and a cut run writes no series.
    pub fn applies(capture: &CaptureSet, explicit: bool) -> bool {
        explicit || !capture.is_empty()
    }

    /// [`Plan::of`] from its numbers: the command's seconds, the retention bound, the seconds
    /// gone, the modes left, the capture (`set`, dumps, whether the runtime is Python) and the
    /// longest a process may run.
    pub fn compute(
        command: u64,
        heap: f64,
        elapsed: u64,
        modes_left: usize,
        capture: (&CaptureSet, u32, bool),
        cap: Duration,
    ) -> Self {
        let share = command.saturating_sub(elapsed) / modes_left.max(1) as u64;
        let process = share
            .saturating_sub(START_UP_SECS)
            .max(1)
            .min(cap.as_secs().max(1));
        let capture = Self::capture_secs(capture.0, capture.1, capture.2, heap);
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

    /// Seconds the capture of `set` may take after the last round: the retention walk (`heap`
    /// seconds, `DLA_HEAP_SECS`) and the last allocation profile.
    pub fn capture_secs(set: &CaptureSet, dumps: u32, python: bool, heap: f64) -> u64 {
        if set.is_empty() {
            return 0;
        }
        let mut secs = 0.0;
        if python {
            if set.heap {
                secs += heap;
            }
            if set.alloc || set.histogram || set.nonheap {
                secs += PROFILE_SECS as f64;
            }
        } else {
            // A JVM loop does not measure its checkpoints: every dump point is kept back.
            secs += (PROFILE_SECS * u64::from(dumps.max(1))) as f64;
        }
        secs.ceil() as u64
    }

    /// The sentence of a plain loop, which keeps the limit of its manifest.
    pub fn plain_limit(limit_secs: u64) -> String {
        format!(
            "this loop runs up to its limit of {} minutes per key mode and the tool may move the command to the background: wait for it; do not wrap it in timeout",
            limit_secs.div_ceil(60)
        )
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
                    what.push(format!("rounds {a} to {}", b.trim_end_matches(':')));
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
    // Another `dla harness run` that is alive is a run in progress, not a command that was cut:
    // what it started is its own.
    let another_runs = state.processes.iter().any(|p| {
        p.kind == ProcessKind::Command
            && p.stopped_at.is_none()
            && p.pid != std::process::id()
            && p.command.starts_with("dla harness run")
            && proc::is_alive(p.pid)
            && proc::identity_matches(p, dla_sample::os::process_start_time(p.pid).as_deref())
    });
    if another_runs {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn set(heap: bool, alloc: bool) -> CaptureSet {
        CaptureSet {
            heap,
            alloc,
            ..CaptureSet::default()
        }
    }

    #[test]
    fn a_run_without_a_capture_gives_the_loop_the_command_less_the_start_up() {
        let none = CaptureSet::default();
        let p = Plan::compute(420, 60.0, 0, 1, (&none, 4, true), Duration::from_secs(1800));
        assert_eq!((p.process, p.loop_secs, p.capture), (410, 410, 0));
        // The time already gone (a build) is not given again; the manifest's own cap holds.
        let p = Plan::compute(420, 60.0, 30, 1, (&none, 4, true), Duration::from_secs(100));
        assert_eq!((p.process, p.loop_secs), (100, 100));
        let p = Plan::compute(
            420,
            60.0,
            30,
            1,
            (&none, 4, true),
            Duration::from_secs(1800),
        );
        assert_eq!(p.process, 380);
    }

    #[test]
    fn a_capture_keeps_back_the_retention_bound_and_the_last_profile() {
        let c = set(true, true);
        let p = Plan::compute(420, 60.0, 0, 1, (&c, 4, true), Duration::from_secs(1800));
        assert_eq!(p.capture, 72, "{p:?}");
        assert_eq!((p.process, p.loop_secs), (410, 338));
        // `DLA_HEAP_SECS` moves it; heap alone has no profile in it.
        let p = Plan::compute(
            420,
            10.0,
            0,
            1,
            (&set(true, false), 4, true),
            Duration::from_secs(1800),
        );
        assert_eq!((p.capture, p.loop_secs), (10, 400));
        // A bound larger than the process leaves the loop a third of it.
        let p = Plan::compute(40, 60.0, 0, 1, (&c, 4, true), Duration::from_secs(1800));
        assert_eq!((p.process, p.loop_secs), (30, 10));
        // The JVM loop does not measure its checkpoints: every dump point is kept back.
        let p = Plan::compute(
            420,
            60.0,
            0,
            1,
            (&set(true, false), 4, false),
            Duration::from_secs(1800),
        );
        assert_eq!(p.capture, 48);
    }

    #[test]
    fn a_plain_run_keeps_its_manifest_limit_and_a_capture_run_gets_the_plan() {
        let none = CaptureSet::default();
        let cap = set(true, true);
        assert!(
            !Plan::applies(&none, false),
            "a plain run has no command bound"
        );
        assert!(Plan::applies(&none, true), "DLA_COMMAND_SECS set bounds it");
        assert!(
            Plan::applies(&cap, false),
            "a capture run has the 420 s plan"
        );
        let p = Plan::compute(
            COMMAND_SECS_DEFAULT,
            60.0,
            0,
            1,
            (&cap, 4, true),
            Duration::from_secs(1800),
        );
        assert_eq!((p.command, p.process), (420, 410));
    }

    #[test]
    fn key_modes_share_what_is_left() {
        let none = CaptureSet::default();
        let p = Plan::compute(420, 60.0, 0, 2, (&none, 4, true), Duration::from_secs(1800));
        assert_eq!(p.process, 200);
        let p = Plan::compute(
            420,
            60.0,
            190,
            1,
            (&none, 4, true),
            Duration::from_secs(1800),
        );
        assert_eq!(p.process, 220);
    }

    #[test]
    fn the_entry_says_what_was_shortened_and_nothing_when_nothing_was() {
        let p = Plan::compute(
            420,
            60.0,
            0,
            1,
            (&set(true, true), 4, true),
            Duration::from_secs(1800),
        );
        assert_eq!(p.attention(&[]), None);
        let l = vec![
            "the test loop cut its rounds from 9 to 3: one run takes 2.7 s".to_owned(),
            "the test loop lowered its step from 2 to 1 runs of the tests per round in round 1".to_owned(),
            "dropped test file tests/a.py: it took 9.0 s of 12.0 s per run and the series would not fit 338 s (left out from run 2 on)".to_owned(),
            "dropped test file tests/b.py: none of its tests passed in the first run".to_owned(),
        ];
        let text = p.attention(&l).unwrap();
        assert!(
            text.starts_with("SHORTENED TO FIT 420 s: the loop got 338 s (10 s start-up and 72 s capture kept back): rounds 9 to 3, step 2 to 1, 1 slowest test file(s) dropped"),
            "{text}"
        );
        assert!(
            p.ends_by_itself().contains("within 7 minutes")
                && p.ends_by_itself().contains("do not wrap it in timeout")
        );
    }
}
