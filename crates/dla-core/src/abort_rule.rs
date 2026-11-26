//! What ending a run on purpose (`dla run abort`) may skip, and what it may not.
//!
//! While budget is left and the run can still measure something, an abort needs a blocker:
//! the user asked to stop, or something the run cannot change stands in the way. An abort
//! never takes the verifier away: the confirmed findings that no verification record covers
//! stay to be verified, and the stop hook and the report say so.

use std::path::Path;

use crate::{
    RunView,
    finding::{FindingState, Polarity},
    gate::{self, GateTarget, Missing},
    ids::FindingId,
    state::RunStatus,
};

/// The kinds `--blocker` takes.
pub const BLOCKER_KINDS: [&str; 6] = [
    "user",
    "tool",
    "build",
    "environment",
    "privilege",
    "refusal",
];

/// Shortest reason, in words, that can name a blocker.
const MIN_REASON_WORDS: usize = 3;

/// Final-gate items other than `queue` and `growth` whose fix is a measurement.
const MEASUREMENT_KINDS: [&str; 5] = [
    "harness_not_run",
    "series",
    "trend_column",
    "budget_left_unrevisited",
    "trigger",
];

/// Words that say a thing is in the way; a reason with one of them names a blocker even when
/// it also counts the work left.
const BLOCKER_CUES: [&str; 22] = [
    "missing",
    "not installed",
    "not found",
    "no such",
    "fails",
    "failed",
    "cannot",
    "can't",
    "can not",
    "denied",
    "refused",
    "unavailable",
    "no access",
    "permission",
    "does not compile",
    "doesn't compile",
    "does not build",
    "no network",
    "offline",
    "without",
    "error",
    "not available",
];

/// Words that make a number in a reason a count of work left.
const WORK_WORDS: [&str; 12] = [
    "candidate",
    "target",
    "queue",
    "remaining",
    "left",
    "hypothes",
    "batch",
    "finding",
    "open item",
    "unmeasured",
    "undecided",
    "minutes",
];

/// A validated `--blocker`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocker {
    /// One of [`BLOCKER_KINDS`].
    pub kind: String,
    /// The reason, as given (may be empty for `user`).
    pub reason: String,
    /// The refusal code, for `refusal`.
    pub code: Option<String>,
}

impl Blocker {
    /// The run's incomplete reason for an abort with this blocker.
    pub fn incomplete_reason(&self) -> String {
        match (self.kind.as_str(), &self.code) {
            ("user", _) if self.reason.is_empty() => "aborted at the user's request".to_owned(),
            ("user", _) => format!("aborted at the user's request: {}", self.reason),
            ("refusal", Some(code)) => {
                format!("aborted with blocker refusal {code}: {}", self.reason)
            }
            (kind, _) => format!("aborted with blocker {kind}: {}", self.reason),
        }
    }
}

/// Checks the `--blocker`, `--reason` and `--code` of an abort. The error is the code and the
/// message of the refusal and its fix: a whole `dla run abort ...` command that is accepted as
/// printed once its `<placeholders>` and its one `a|b|c` choice are filled in.
pub fn check_blocker(
    kind: &str,
    reason: Option<&str>,
    code: Option<&str>,
) -> Result<Blocker, (&'static str, String, String)> {
    let reason = reason.map(str::trim).unwrap_or_default();
    if !BLOCKER_KINDS.contains(&kind) {
        return Err((
            crate::error::codes::INVALID_VALUE,
            format!(
                "--blocker {kind:?} is not a blocker kind (one of {})",
                BLOCKER_KINDS.join(", ")
            ),
            "dla run abort --blocker tool|build|environment|privilege --reason \"<what blocks the run>\""
                .to_owned(),
        ));
    }
    if kind != "refusal" && code.is_some() {
        return Err((
            crate::error::codes::INVALID_VALUE,
            format!("--code goes with --blocker refusal only, not --blocker {kind}"),
            format!("dla run abort --blocker {kind} --reason \"<what blocks the run>\""),
        ));
    }
    if kind == "user" {
        return Ok(Blocker {
            kind: kind.to_owned(),
            reason: reason.to_owned(),
            code: None,
        });
    }
    let code = if kind == "refusal" {
        let c = code.map(str::trim).unwrap_or_default();
        let shaped = !c.is_empty()
            && c.starts_with(|ch: char| ch.is_ascii_lowercase())
            && c.chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
        if !shaped {
            return Err((
                "missing-argument",
                "--blocker refusal needs --code <the code of the refusal>, as printed in the refusal's `error.code` (lowercase words joined by hyphens, such as time-left)".to_owned(),
                "dla run abort --blocker refusal --code <refusal code> --reason \"<what blocks the run: the refusal the run cannot satisfy>\"".to_owned(),
            ));
        }
        Some(c.to_owned())
    } else {
        None
    };
    if let Some(why) = reason_is_not_a_blocker(reason) {
        return Err((
            "missing-argument",
            format!("--blocker {kind} needs a --reason that names what blocks the run: {why}"),
            match &code {
                Some(c) => format!(
                    "dla run abort --blocker {kind} --code {c} --reason \"<what blocks the run: the {kind} that is missing or fails>\""
                ),
                None => format!(
                    "dla run abort --blocker {kind} --reason \"<what blocks the run: the {kind} that is missing or fails>\""
                ),
            },
        ));
    }
    Ok(Blocker {
        kind: kind.to_owned(),
        reason: reason.to_owned(),
        code,
    })
}

/// Why `reason` names no blocker: empty, too short, or only a count of the work left.
fn reason_is_not_a_blocker(reason: &str) -> Option<&'static str> {
    if reason.is_empty() {
        return Some("it is empty");
    }
    let words = reason
        .split_whitespace()
        .filter(|w| w.chars().any(char::is_alphabetic))
        .count();
    if words < MIN_REASON_WORDS {
        return Some("it is too short to name a blocker");
    }
    let lower = reason.to_lowercase();
    let counts =
        lower.chars().any(|c| c.is_ascii_digit()) && WORK_WORDS.iter().any(|w| lower.contains(w));
    if counts && !BLOCKER_CUES.iter().any(|c| lower.contains(c)) {
        return Some("it only counts the work left, and what is left is what the run can still do");
    }
    None
}

/// The work a run can still do while its budget is over the floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenWork {
    /// Minutes left to the run's deadline.
    pub left: u64,
    /// The budget in minutes.
    pub total: u64,
    /// The queue lines (one per runtime whose measurement queue is open).
    pub queues: Vec<String>,
    /// The findings whose growth was measured and not decided.
    pub growths: Vec<String>,
    /// The command to run first.
    pub first: String,
}

impl OpenWork {
    /// What is open, in a sentence.
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(q) = self.queues.first() {
            let more = self.queues.len() - 1;
            parts.push(format!(
                "queue: {}{}",
                q.trim_end(),
                if more > 0 {
                    format!(" (and {more} more queue(s))")
                } else {
                    String::new()
                }
            ));
        }
        if !self.growths.is_empty() {
            parts.push(format!(
                "{} undecided growth(s): {}",
                self.growths.len(),
                self.growths.join(", ")
            ));
        }
        if parts.is_empty() {
            parts.push("a measurement the final gate names".to_owned());
        }
        parts.join("; ")
    }
}

/// The first command of a fix that lists several: a queue's round (its first command is the
/// next batch), or a growth's commands in order (the first, without its comment).
fn first_command(m: &Missing) -> String {
    let first = if m.kind == "queue" {
        m.fix.split(", ").next().unwrap_or(&m.fix)
    } else {
        let f = m.fix.split("   then   ").next().unwrap_or(&m.fix);
        f.split("   (").next().unwrap_or(f).trim()
    };
    let mut out = first.to_owned();
    if out.len() > 240 {
        let mut cut = 240;
        while !out.is_char_boundary(cut) {
            cut -= 1;
        }
        out.truncate(cut);
    }
    out
}

/// The work an abort would leave undone: `Some` while the budget is over the floor, no
/// command was refused for time with nothing smaller that fits, and an open queue, an
/// undecided growth or another measurement the final gate names is open. `None` for a run that
/// is over (aborted, merged incomplete), under the floor or with nothing open.
pub fn open_work(view: &RunView, run_dir: &Path) -> Option<OpenWork> {
    let (left, total) = gate::budget_over_floor(&view.state, None)?;
    if crate::toolctx::nothing_fits_since(run_dir, None, view.state.budget.started_at) {
        return None;
    }
    let missing = gate::missing(view, GateTarget::Final, None);
    let queues: Vec<&Missing> = missing.iter().filter(|m| m.kind == "queue").collect();
    let growth_items: Vec<&Missing> = missing.iter().filter(|m| m.kind == "growth").collect();
    let mut growths: Vec<String> = growth_items.iter().map(|m| m.id.clone()).collect();
    // A finding still open whose growth was measured and not decided.
    let grown = crate::grown::grown_unmeasured(view);
    let mut grown_first: Option<String> = None;
    for m in missing.iter().filter(|m| m.kind == "finding") {
        if let Some(g) = grown.iter().find(|g| g.finding.to_string() == m.id) {
            if !growths.contains(&m.id) {
                growths.push(m.id.clone());
            }
            if grown_first.is_none() {
                grown_first = g.commands().first().cloned();
            }
        }
    }
    let other = missing
        .iter()
        .find(|m| MEASUREMENT_KINDS.contains(&m.kind.as_str()));
    let first = if let Some(q) = queues.first() {
        first_command(q)
    } else if let Some(g) = growth_items.first() {
        first_command(g)
    } else if let Some(c) = grown_first {
        c.split("   (").next().unwrap_or(&c).trim().to_owned()
    } else {
        first_command(other?)
    };
    Some(OpenWork {
        left,
        total,
        queues: queues.iter().map(|m| m.description.clone()).collect(),
        growths,
        first,
    })
}

/// The confirmed leak-side findings that no verification record covers, in ID order: what an
/// abort must not leave unverified. A finding merged into another is carried by its twin.
pub fn unverified_confirmed(view: &RunView) -> Vec<FindingId> {
    let mut out: Vec<FindingId> = view
        .ledger
        .findings()
        .filter(|f| {
            f.state == FindingState::Confirmed
                && f.polarity() == Polarity::Leak
                && view.ledger.merged_into(&f.id).is_none()
                && view.ledger.verifications(&f.id).is_empty()
        })
        .map(|f| f.id.clone())
        .collect();
    out.sort();
    out
}

/// The commands that verify a finding, in order.
pub fn verify_steps(id: &FindingId) -> String {
    format!(
        "dla bundle {id}, then one dla:verifier subagent given only the bundle directory, the run id, {id} and the sha, then dla ledger verify {id} --site <file:line|none> --holder <h> --mechanism \"<text>\" --label <ID|UNCLASSIFIED> --bundle <sha>"
    )
}

/// Whether the run is aborted.
pub fn is_aborted(view: &RunView) -> bool {
    view.state.status == RunStatus::Aborted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_needs_no_reason_and_says_so() {
        let b = check_blocker("user", None, None).unwrap();
        assert_eq!(b.incomplete_reason(), "aborted at the user's request");
        let b = check_blocker("user", Some(" enough "), None).unwrap();
        assert_eq!(
            b.incomplete_reason(),
            "aborted at the user's request: enough"
        );
    }

    #[test]
    fn a_blocker_of_the_environment_needs_a_reason_that_is_not_a_count() {
        for kind in ["tool", "build", "environment", "privilege"] {
            assert!(check_blocker(kind, None, None).is_err(), "{kind}");
            assert!(check_blocker(kind, Some("  "), None).is_err(), "{kind}");
            assert!(check_blocker(kind, Some("no go"), None).is_err(), "{kind}");
            assert!(
                check_blocker(
                    kind,
                    Some("4 leak hypotheses confirmed; 139 ranked candidates not targeted"),
                    None
                )
                .is_err(),
                "{kind}"
            );
            let b = check_blocker(kind, Some("the go toolchain is not installed"), None).unwrap();
            assert_eq!(
                b.incomplete_reason(),
                format!("aborted with blocker {kind}: the go toolchain is not installed")
            );
        }
        // A count next to what is in the way still names the blocker.
        assert!(
            check_blocker(
                "build",
                Some("the build fails with 12 errors, so no candidates can run"),
                None
            )
            .is_ok()
        );
    }

    #[test]
    fn a_refusal_blocker_needs_a_code_and_only_it() {
        let reason = Some("the capture needs a privilege the run does not have");
        assert!(check_blocker("refusal", reason, None).is_err());
        assert!(check_blocker("refusal", reason, Some("Not A Code")).is_err());
        let b = check_blocker("refusal", reason, Some("time-left")).unwrap();
        assert_eq!(b.code.as_deref(), Some("time-left"));
        assert!(
            b.incomplete_reason()
                .starts_with("aborted with blocker refusal time-left: ")
        );
        assert!(check_blocker("tool", reason, Some("time-left")).is_err());
        assert!(check_blocker("sleepy", reason, None).is_err());
    }
}
