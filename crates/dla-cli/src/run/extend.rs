//! `dla run extend <dur>` and the `budget` object of `dla run status`.

use std::path::Path;

use chrono::SecondsFormat;
use clap::Args;
use dla_core::{
    Ledger, Refusal, RunState, RunStatus, RunView,
    error::codes as core_codes,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool,
    },
    state::{self, Budget},
    timebudget,
};
use serde_json::{Value, json};

use crate::{budget, ctx::Ctx, fail::Res, git};

/// Flags of `dla run extend`.
#[derive(Args, Debug)]
pub struct ExtendArgs {
    /// How much to add: 90s, 45m or 2h (`auto` is not accepted here). The new deadline is the
    /// later of the current deadline and now, plus this.
    dur: String,
}

fn evidence_id(ledger: &Ledger, seq: u64) -> String {
    ledger
        .lines()
        .iter()
        .find(|l| l.seq == seq)
        .and_then(|l| match &l.record {
            dla_core::Record::Evidence(e) => Some(e.evidence_id.to_string()),
            _ => None,
        })
        .unwrap_or_default()
}

fn rfc3339(t: chrono::DateTime<chrono::Utc>) -> String {
    t.to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// Seconds from `now` to `deadline`, zero when it has passed.
fn left(deadline: chrono::DateTime<chrono::Utc>, now: chrono::DateTime<chrono::Utc>) -> u64 {
    u64::try_from((deadline - now).num_seconds()).unwrap_or(0)
}

/// Runs `dla run extend`: moves the run deadline and every unfinished worker deadline
/// later, and records a `run-config` evidence record (static authority, so it never counts
/// as a measurement) naming the extension. Core only; refused on a run that is not active.
pub fn run(ctx: &Ctx, project: &Path, a: &ExtendArgs) -> Res<Value> {
    super::workers::refuse_worker_mode(ctx, "run extend")?;
    let seconds = budget::parse_fixed(&a.dur).map_err(|mut f| {
        if a.dur.trim().eq_ignore_ascii_case("auto") {
            f.refusal.message =
                "invalid extension \"auto\": an extension is a fixed duration".to_owned();
        } else {
            // The parser words its refusal for `dla run init --budget`.
            f.refusal.message =
                f.refusal
                    .message
                    .replacen("invalid budget", "invalid extension", 1);
        }
        f.refusal.missing =
            "a fixed duration such as 90s, 45m or 2h (auto is not accepted)".to_owned();
        f.refusal.fix = "dla run extend 30m".to_owned();
        f
    })?;
    let root_text = dla_core::paths::to_slash(project);
    let commit = git::head_commit(project).unwrap_or(root_text);
    let out = RunState::update(&ctx.run_dir, |st| {
        if st.status != RunStatus::Active {
            return Err(Refusal::new(
                core_codes::RUN_NOT_ACTIVE,
                format!(
                    "run {} is {}, so its budget cannot be extended",
                    st.run_id, st.status
                ),
                "an active run",
                "dla run init",
            )
            .into());
        }
        let now = state::now_utc();
        let previous = st.budget.deadline;
        let deadline = st.extend_budget(now, seconds)?;
        let mut ledger = Ledger::load(&ctx.run_dir)?;
        let line = ledger.add_evidence(NewEvidence {
            source: Source::Repo {
                commit: commit.clone(),
                path: None,
            },
            tool: Tool {
                name: "dla".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            authority: Authority::Static,
            kind: EvidenceKind::RunConfig,
            result: json!({
                "extend_budget_seconds": seconds,
                "previous_deadline": rfc3339(previous),
                "deadline": rfc3339(deadline),
                "budget_seconds": st.budget.seconds,
                "extensions": st.budget.extensions.len(),
            }),
            raw: None,
            coverage: EvidenceCoverage {
                status: CoverageStatus::Complete,
                examined: None,
                total: None,
            },
            limitations: Vec::new(),
            unknowns: Vec::new(),
        })?;
        Ok((
            deadline,
            previous,
            st.budget.seconds,
            st.budget.extensions.len(),
            line.seq,
            evidence_id(&ledger, line.seq),
            st.workers
                .iter()
                .map(|w| json!({"name": w.name, "deadline": rfc3339(w.deadline)}))
                .collect::<Vec<_>>(),
        ))
    })?;
    let (deadline, previous, total, n, seq, evidence, workers) = out;
    let now = state::now_utc();
    Ok(json!({
        "ok": true,
        "run_id": ctx.run_id,
        "extended_by_seconds": seconds,
        "previous_deadline": rfc3339(previous),
        "deadline": rfc3339(deadline),
        "budget_seconds": total,
        "extensions": n,
        "remaining_seconds": left(deadline, now),
        "minutes_left": left(deadline, now) / 60,
        "workers": workers,
        "evidence": evidence,
        "ledger": ctx.ledger_line(seq),
    }))
}

/// The `budget` object of `dla run status`.
///
/// - `seconds`: the budget length (start to deadline, extensions included).
/// - `remaining_seconds`: time to the nearest deadline (the worker's in worker mode, and
///   the run's); zero once one has passed.
/// - `auto`, `settled` (the sizing is final), `settled_workers` (the workers whose first
///   passing gate has sized the budget), `estimate` (`{runtimes, candidates, minutes,
///   total_minutes}`, once sized) and `extensions` (`[{at, seconds, lapsed_seconds,
///   deadline}]`; `lapsed_seconds` is how long the run had already been past its deadline
///   when it was extended, zero for an extension made in time).
/// - `to_finish_minutes`: [`timebudget::remaining`], that is `3 x (non-duplicate findings
///   that are open or were closed with reason time_budget) + 15 x (their runtimes that
///   have no harness run that succeeded)`; `to_finish_findings` is the first factor.
pub fn status_json(view: &RunView, worker: Option<&str>) -> Value {
    let b: &Budget = &view.state.budget;
    let now = state::now_utc();
    let mut deadline = b.deadline;
    if let Some(w) = worker.and_then(|w| view.state.worker(w)) {
        deadline = deadline.min(w.deadline);
    }
    let r = timebudget::remaining(view);
    json!({
        "seconds": b.seconds,
        "remaining_seconds": left(deadline, now),
        "deadline": rfc3339(b.deadline),
        "auto": b.auto,
        "settled": b.settled,
        "settled_workers": b.settled_workers,
        "estimate": b.estimate,
        "extensions": b.extensions,
        "to_finish_minutes": r.minutes,
        "to_finish_findings": r.findings,
    })
}
