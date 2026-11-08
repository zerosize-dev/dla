//! The process registry side of an attach: tier by identity, the attached-tool record and its
//! cleanup list.

use chrono::{DateTime, Utc};
use dla_core::{
    RunState,
    state::{ProcessKind, ProcessRecord, ProcessRole},
};
use dla_sample::attach::Cleanup;

/// Whether two process start times (RFC 3339) are the same instant within 2 s. The Linux
/// figure is derived from the boot time, which a clock step can move by a second.
pub fn same_start(a: &str, b: &str) -> bool {
    match (
        DateTime::parse_from_rfc3339(a),
        DateTime::parse_from_rfc3339(b),
    ) {
        (Ok(x), Ok(y)) => (x - y).num_milliseconds().abs() <= 2000,
        _ => a == b,
    }
}

/// Whether DLA started the live process `pid` (a running registry record with the same
/// start time). The pid alone proves nothing: pids are reused.
pub fn started_by_dla(state: &RunState, pid: u32, start: Option<&str>) -> bool {
    state.processes.iter().any(|p| {
        p.role == ProcessRole::Started
            && p.pid == pid
            && p.stopped_at.is_none()
            && match (p.start_time.as_deref(), start) {
                (Some(a), Some(b)) => same_start(a, b),
                _ => false,
            }
    })
}

/// The record for one attach: the target's identity, the tool, no cleanup yet.
pub fn new_record(
    pid: u32,
    start_time: Option<&str>,
    command: &str,
    worker: Option<&str>,
) -> ProcessRecord {
    ProcessRecord {
        pid,
        start_time: start_time.map(str::to_owned),
        kind: ProcessKind::Profiler,
        command: command.to_owned(),
        started_at: Utc::now(),
        stopped_at: None,
        role: ProcessRole::Attached,
        worker: worker.map(str::to_owned),
        cleanup: Vec::new(),
    }
}

fn find<'a>(state: &'a mut RunState, pid: u32, command: &str) -> Option<&'a mut ProcessRecord> {
    state
        .processes
        .iter_mut()
        .rev()
        .find(|p| p.role == ProcessRole::Attached && p.pid == pid && p.command == command)
}

/// Appends a cleanup action to the attach's record as soon as the backend needs it.
pub fn add_cleanup(run_dir: &std::path::Path, pid: u32, command: &str, action: &Cleanup) {
    let text = action.render();
    let _ = RunState::update(run_dir, |s| {
        if let Some(p) = find(s, pid, command)
            && !p.cleanup.contains(&text)
        {
            p.cleanup.push(text);
        }
        Ok(())
    });
}

/// Marks the attach finished (the target keeps running) and replaces the cleanup list by what
/// is still left to do.
pub fn finish(
    run_dir: &std::path::Path,
    pid: u32,
    command: &str,
    left: &[Cleanup],
) -> Result<(), dla_core::Error> {
    RunState::update(run_dir, |s| {
        if let Some(p) = find(s, pid, command) {
            p.stopped_at = Some(Utc::now());
            p.cleanup = left.iter().map(Cleanup::render).collect();
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(role: ProcessRole, start: Option<&str>) -> RunState {
        let mut s = RunState::init("/p", None, 60, None, Utc::now()).unwrap();
        let mut r = new_record(77, start, "node app.js", None);
        r.role = role;
        r.kind = ProcessKind::DevServer;
        s.processes.push(r);
        s
    }

    #[test]
    fn a_started_process_needs_a_matching_start_time() {
        let s = state_with(ProcessRole::Started, Some("2026-01-01T00:00:10Z"));
        assert!(started_by_dla(&s, 77, Some("2026-01-01T00:00:11Z")));
        assert!(
            !started_by_dla(&s, 77, Some("2026-01-01T00:05:00Z")),
            "pid reuse"
        );
        assert!(!started_by_dla(&s, 77, None));
        assert!(!started_by_dla(&s, 78, Some("2026-01-01T00:00:10Z")));
    }

    #[test]
    fn an_attached_tool_record_is_not_a_started_process() {
        let s = state_with(ProcessRole::Attached, Some("2026-01-01T00:00:10Z"));
        assert!(!started_by_dla(&s, 77, Some("2026-01-01T00:00:10Z")));
    }

    #[test]
    fn stopped_processes_do_not_count() {
        let mut s = state_with(ProcessRole::Started, Some("2026-01-01T00:00:10Z"));
        s.processes[0].stopped_at = Some(Utc::now());
        assert!(!started_by_dla(&s, 77, Some("2026-01-01T00:00:10Z")));
    }

    #[test]
    fn records_carry_the_attached_role() {
        let r = new_record(5, Some("x"), "attach P1: jcmd histogram", Some("jvm"));
        assert_eq!(r.role, ProcessRole::Attached);
        assert_eq!(r.kind, ProcessKind::Profiler);
        assert_eq!(r.worker.as_deref(), Some("jvm"));
        assert!(r.cleanup.is_empty() && r.stopped_at.is_none());
    }
}
