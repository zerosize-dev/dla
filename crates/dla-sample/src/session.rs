//! A sampling session: OS figures on every tick plus runtime backends polled on their own
//! threads, merged into one table of points.
//!
//! Backends can be slow (`jcmd` starts a JVM per call), so each polls on its own thread at its
//! own pace. A tick takes the OS sample on time and attaches whatever readings the backends
//! delivered since the previous tick. A backend that had nothing new leaves its columns missing
//! at that tick; nothing is repeated or interpolated.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    backend::{Backend, BackendError, Reading},
    os::{LargestDescendant, SampleError, TickSampler},
};

/// Settings of a session.
#[derive(Debug, Clone, Copy)]
pub struct SessionConfig {
    /// The process to sample.
    pub pid: u32,
    /// Time between OS samples.
    pub interval: Duration,
    /// Longest time to sample for.
    pub duration: Duration,
    /// Sum OS figures over the process tree.
    pub tree: bool,
}

/// How a backend fared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendState {
    /// At least one poll produced figures.
    Ran,
    /// Every poll failed.
    Blocked,
}

/// What happened to one backend during the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendReport {
    /// Backend name.
    pub name: String,
    /// Runtime worker key.
    pub runtime: String,
    /// Ran or blocked.
    pub state: BackendState,
    /// Why it was blocked, or the last failure of a backend that mostly ran.
    pub reason: Option<String>,
    /// Polls that produced figures.
    pub readings: u64,
    /// Polls that failed (those after the process exited are not counted).
    pub failures: u64,
}

/// One tick: the OS figures and the backend figures that arrived since the last tick.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TickPoint {
    /// Milliseconds since the session started.
    pub t_ms: u64,
    /// Column name to value.
    pub values: BTreeMap<String, f64>,
}

/// Everything a session collected.
#[derive(Debug, Clone, Default)]
pub struct SessionOutput {
    /// The ticks in time order (at least one).
    pub points: Vec<TickPoint>,
    /// OS limitations and backend notes, without duplicates.
    pub limitations: Vec<String>,
    /// One entry per backend, in the order given.
    pub backends: Vec<BackendReport>,
    /// The sampled process exited before the duration passed.
    pub exited: bool,
    /// An unexpected failure that ended sampling early.
    pub error: Option<String>,
    /// Largest descendant by resident memory (tree mode).
    pub largest_descendant: Option<LargestDescendant>,
    /// Milliseconds from the start to the last tick.
    pub elapsed_ms: u64,
}

struct Msg {
    idx: usize,
    t_ms: u64,
    result: Result<Reading, BackendError>,
}

fn worker(
    idx: usize,
    mut backend: Box<dyn Backend>,
    origin: Instant,
    tx: Sender<Msg>,
    stop: Arc<AtomicBool>,
) {
    let min = backend.min_interval();
    let mut ever_ok = false;
    while !stop.load(Ordering::Relaxed) {
        let started = Instant::now();
        let result = backend.poll();
        let ok = result.is_ok();
        let t_ms = origin.elapsed().as_millis() as u64;
        if tx.send(Msg { idx, t_ms, result }).is_err() {
            return;
        }
        ever_ok |= ok;
        // A backend that has never worked will not start to; do not hammer it.
        if !ever_ok && !ok {
            return;
        }
        while started.elapsed() < min && !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(20));
        }
    }
}

#[derive(Default)]
struct Book {
    pending: Vec<Option<Reading>>,
    readings: Vec<u64>,
    failures: Vec<u64>,
    last_error: Vec<Option<String>>,
    /// Failure times, kept to discount those that follow the process's exit.
    failed_at: Vec<Vec<u64>>,
    notes: BTreeSet<String>,
}

impl Book {
    fn new(n: usize) -> Self {
        Self {
            pending: vec![None; n],
            readings: vec![0; n],
            failures: vec![0; n],
            last_error: vec![None; n],
            failed_at: vec![Vec::new(); n],
            notes: BTreeSet::new(),
        }
    }

    fn drain(&mut self, rx: &Receiver<Msg>) {
        while let Ok(m) = rx.try_recv() {
            match m.result {
                Ok(r) => {
                    self.readings[m.idx] += 1;
                    self.notes.extend(r.notes.iter().cloned());
                    self.pending[m.idx] = Some(r);
                }
                Err(e) => {
                    self.failures[m.idx] += 1;
                    self.failed_at[m.idx].push(m.t_ms);
                    self.last_error[m.idx] = Some(e.message);
                }
            }
        }
    }
}

/// Samples `config.pid` until it exits or `config.duration` passes.
///
/// Fails only when the first OS sample fails (the process does not exist). Backend failures
/// are reported in [`SessionOutput::backends`].
pub fn run(
    config: SessionConfig,
    backends: Vec<Box<dyn Backend>>,
) -> Result<SessionOutput, SampleError> {
    let origin = Instant::now();
    let mut sampler = TickSampler::new(config.pid, config.tree, origin);
    let first = sampler.sample()?;
    let infos: Vec<(String, String, &'static [&'static str])> = backends
        .iter()
        .map(|b| {
            (
                b.name().to_owned(),
                b.runtime().to_owned(),
                b.zero_fill_prefixes(),
            )
        })
        .collect();
    let (tx, rx) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let handles: Vec<_> = backends
        .into_iter()
        .enumerate()
        .map(|(idx, b)| {
            let (tx, stop) = (tx.clone(), Arc::clone(&stop));
            thread::spawn(move || worker(idx, b, origin, tx, stop))
        })
        .collect();
    drop(tx);

    let mut book = Book::new(infos.len());
    let mut os_limits: BTreeSet<String> = first.limitations.iter().cloned().collect();
    let mut out = SessionOutput::default();
    // Which backends delivered at each point, for zero-filling counts.
    let mut delivered: Vec<Vec<bool>> = Vec::new();
    let mut sample = first;
    let mut next = origin;
    loop {
        book.drain(&rx);
        os_limits.extend(sample.limitations.iter().cloned());
        let mut values = sample.values;
        let mut got = vec![false; infos.len()];
        for (i, slot) in book.pending.iter_mut().enumerate() {
            if let Some(r) = slot.take() {
                values.extend(r.values);
                got[i] = true;
            }
        }
        out.points.push(TickPoint {
            t_ms: sample.t_ms,
            values,
        });
        delivered.push(got);
        out.elapsed_ms = sample.t_ms;

        next += config.interval;
        let now = Instant::now();
        while next <= now {
            next += config.interval;
        }
        if next.duration_since(origin) > config.duration {
            break;
        }
        thread::sleep(next.saturating_duration_since(Instant::now()));
        match sampler.sample() {
            Ok(s) => sample = s,
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

    stop.store(true, Ordering::Relaxed);
    for h in handles {
        let _ = h.join();
    }
    book.drain(&rx);
    // Readings that finished after the last tick still belong to the last point.
    if let (Some(last), Some(got)) = (out.points.last_mut(), delivered.last_mut()) {
        for (i, slot) in book.pending.iter_mut().enumerate() {
            if !got[i]
                && let Some(r) = slot.take()
            {
                last.values.extend(r.values);
                got[i] = true;
            }
        }
    }
    out.largest_descendant = sampler.largest_descendant();

    // Counts that a backend omits when they drop to zero.
    for (i, (_, _, prefixes)) in infos.iter().enumerate() {
        if prefixes.is_empty() {
            continue;
        }
        let columns: BTreeSet<String> = out
            .points
            .iter()
            .flat_map(|p| p.values.keys())
            .filter(|c| prefixes.iter().any(|pre| c.starts_with(pre)))
            .cloned()
            .collect();
        for (p, got) in out.points.iter_mut().zip(&delivered) {
            if got[i] {
                for c in &columns {
                    p.values.entry(c.clone()).or_insert(0.0);
                }
            }
        }
    }

    let exit_t = out.exited.then_some(out.elapsed_ms);
    let mut limitations = os_limits;
    limitations.extend(book.notes.iter().cloned());
    for (i, (name, runtime, _)) in infos.into_iter().enumerate() {
        let failures = book.failed_at[i]
            .iter()
            .filter(|&&t| exit_t.is_none_or(|e| t < e))
            .count() as u64;
        let state = if book.readings[i] > 0 {
            BackendState::Ran
        } else {
            BackendState::Blocked
        };
        let reason = (failures > 0 || state == BackendState::Blocked)
            .then(|| book.last_error[i].clone())
            .flatten();
        if state == BackendState::Blocked && reason.is_none() && book.failures[i] == 0 {
            // It never got to poll (the session was shorter than its first poll).
            limitations.insert(format!(
                "{name}: no reading arrived before sampling ended (its first poll takes longer than the duration)"
            ));
        }
        if let Some(r) = &reason {
            if state == BackendState::Ran {
                limitations.insert(format!(
                    "{name}: {failures} poll(s) failed, the last with: {r}"
                ));
            } else {
                limitations.insert(r.clone());
            }
        }
        out.backends.push(BackendReport {
            name,
            runtime,
            state,
            reason: reason.or_else(|| {
                (state == BackendState::Blocked)
                    .then(|| "no reading arrived before sampling ended".to_owned())
            }),
            readings: book.readings[i],
            failures,
        });
    }
    out.limitations = limitations.into_iter().collect();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        name: &'static str,
        polls: u32,
        fail_after: Option<u32>,
        zero: bool,
    }

    impl Backend for Fake {
        fn name(&self) -> &'static str {
            self.name
        }
        fn runtime(&self) -> &'static str {
            "fake"
        }
        fn min_interval(&self) -> Duration {
            Duration::from_millis(30)
        }
        fn poll(&mut self) -> Result<Reading, BackendError> {
            self.polls += 1;
            if self.fail_after.is_some_and(|n| self.polls > n) {
                return Err(BackendError::blocked("fake went away"));
            }
            let mut r = Reading::default();
            r.values.insert("fake.count".into(), f64::from(self.polls));
            if self.zero && self.polls % 2 == 0 {
                r.values.insert("fake.site@a.rs:1".into(), 3.0);
            }
            r.notes.push("fake is approximate".into());
            Ok(r)
        }
        fn zero_fill_prefixes(&self) -> &'static [&'static str] {
            if self.zero { &["fake.site@"] } else { &[] }
        }
    }

    fn config(ms: u64) -> SessionConfig {
        SessionConfig {
            pid: std::process::id(),
            interval: Duration::from_millis(50),
            duration: Duration::from_millis(ms),
            tree: false,
        }
    }

    #[test]
    fn merges_backend_readings_into_ticks() {
        let b: Box<dyn Backend> = Box::new(Fake {
            name: "fake",
            polls: 0,
            fail_after: None,
            zero: true,
        });
        let out = run(config(400), vec![b]).unwrap();
        assert!(out.points.len() >= 4, "{}", out.points.len());
        assert!(out.points.iter().all(|p| p.values.contains_key("os.rss")));
        let counts: Vec<f64> = out
            .points
            .iter()
            .filter_map(|p| p.values.get("fake.count").copied())
            .collect();
        assert!(counts.len() >= 3);
        assert!(counts.windows(2).all(|w| w[1] > w[0]), "{counts:?}");
        // Ticks between readings have no value; zero-fill gives 0 where the backend answered
        // without the site, never where it did not answer.
        let sites: Vec<(bool, Option<f64>)> = out
            .points
            .iter()
            .map(|p| {
                (
                    p.values.contains_key("fake.count"),
                    p.values.get("fake.site@a.rs:1").copied(),
                )
            })
            .collect();
        assert!(
            sites
                .iter()
                .all(|(answered, site)| *answered || site.is_none())
        );
        assert!(sites.iter().any(|(_, s)| *s == Some(3.0)));
        assert!(sites.iter().any(|(a, s)| *a && *s == Some(0.0)));
        assert_eq!(out.backends[0].state, BackendState::Ran);
        assert!(out.limitations.contains(&"fake is approximate".to_owned()));
    }

    #[test]
    fn a_backend_that_never_works_is_blocked_and_sampling_continues() {
        let b: Box<dyn Backend> = Box::new(Fake {
            name: "broken",
            polls: 0,
            fail_after: Some(0),
            zero: false,
        });
        let out = run(config(200), vec![b]).unwrap();
        assert!(out.points.len() >= 2);
        assert_eq!(out.backends[0].state, BackendState::Blocked);
        assert_eq!(out.backends[0].reason.as_deref(), Some("fake went away"));
        assert!(out.limitations.contains(&"fake went away".to_owned()));
        assert!(
            out.points
                .iter()
                .all(|p| !p.values.contains_key("fake.count"))
        );
    }

    #[test]
    fn a_backend_that_fails_later_is_reported_as_partial() {
        let b: Box<dyn Backend> = Box::new(Fake {
            name: "flaky",
            polls: 0,
            fail_after: Some(2),
            zero: false,
        });
        let out = run(config(400), vec![b]).unwrap();
        assert_eq!(out.backends[0].state, BackendState::Ran);
        assert!(out.backends[0].failures >= 1);
        assert!(out.limitations.iter().any(|l| l.starts_with("flaky:")));
    }

    #[cfg(unix)]
    #[test]
    fn stops_when_the_process_exits() {
        let mut child = std::process::Command::new("sleep")
            .arg("0.3")
            .spawn()
            .unwrap();
        let pid = child.id();
        let waiter = thread::spawn(move || child.wait());
        let mut cfg = config(5000);
        cfg.pid = pid;
        let out = run(cfg, vec![]).unwrap();
        waiter.join().unwrap().unwrap();
        assert!(out.exited);
        assert!(out.elapsed_ms < 3000);
    }

    #[test]
    fn a_missing_process_is_an_error() {
        let mut cfg = config(100);
        cfg.pid = 4_000_000;
        assert!(matches!(
            run(cfg, vec![]),
            Err(SampleError::NoSuchProcess(_))
        ));
    }
}
