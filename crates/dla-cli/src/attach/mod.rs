//! The parts of `dla attach` besides the command flow: refusals, the process registry and
//! the artifact evidence.

pub mod evidence;
pub mod registry;

use std::time::Duration;

use dla_sample::attach::{AttachError, ErrorKind};

use crate::fail::Failure;

/// Refusal code: the box lacks disk or memory for the attach.
pub const INSUFFICIENT_RESOURCES: &str = "insufficient-resources";
/// Refusal code: a tool the backend needs is not installed.
pub const TOOL_MISSING: &str = "tool-missing";
/// Refusal code: the target does not meet a precondition.
pub const TARGET_NOT_READY: &str = "target-not-ready";

/// Turns a backend error into the CLI refusal.
pub fn refusal(e: AttachError) -> Failure {
    let code = match e.kind {
        ErrorKind::ToolMissing => TOOL_MISSING,
        ErrorKind::NotReady => TARGET_NOT_READY,
        ErrorKind::Failed => dla_core::error::codes::TOOL_FAILED,
    };
    Failure::refuse(code, e.message, e.missing, e.fix)
}

/// Parses `30s`, `500ms`, `2m` or `1h`; `None` when it is not one of those or is zero.
pub fn parse_duration(text: &str) -> Option<Duration> {
    let t = text.trim();
    let (digits, unit_ms) = if let Some(d) = t.strip_suffix("ms") {
        (d, 1)
    } else if let Some(d) = t.strip_suffix('s') {
        (d, 1000)
    } else if let Some(d) = t.strip_suffix('m') {
        (d, 60_000)
    } else {
        (t.strip_suffix('h')?, 3_600_000)
    };
    let n: u64 = digits.parse().ok()?;
    n.checked_mul(unit_ms)
        .filter(|ms| *ms > 0)
        .map(Duration::from_millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(parse_duration("30s"), Some(Duration::from_secs(30)));
        assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
        assert_eq!(parse_duration("500ms"), Some(Duration::from_millis(500)));
        assert_eq!(parse_duration("0s"), None);
        assert_eq!(parse_duration("30"), None);
        assert_eq!(parse_duration("xs"), None);
    }

    #[test]
    fn backend_errors_become_refusals_with_the_fix() {
        let f = refusal(AttachError::tool_missing(
            "no jcmd",
            "jcmd",
            "install a JDK",
        ));
        assert_eq!(f.refusal.code, TOOL_MISSING);
        assert_eq!(f.exit, 2);
        assert_eq!(f.refusal.fix, "install a JDK");
        let f = refusal(AttachError::not_ready("off", "NMT", "restart"));
        assert_eq!(f.refusal.code, TARGET_NOT_READY);
        let f = refusal(AttachError::failed("boom", "retry"));
        assert_eq!(f.refusal.code, "tool-failed");
    }
}
