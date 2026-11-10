//! `dla gate check | stop-hook | read-hook | write-hook | bash-hook`.
//!
//! `check` evaluates [`dla_core::gate::evaluate`] and records a passing stage. The four
//! hooks live in [`crate::gate_hooks`]: they read the Claude Code event on stdin, find the
//! run from its `cwd`, and exit the way the hook protocol wants.

use std::path::Path;

use clap::{Args, Subcommand};
use dla_core::{
    RunState, RunView,
    fsutil::{to_pretty_json, write_atomic},
    gate::{self, GateTarget},
    state::Stage,
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::Res,
    gate_hooks::{self, Hook},
    parse::parse_enum,
};

/// `dla gate ...`
#[derive(Subcommand, Debug)]
pub enum GateCmd {
    /// Check the requirements of one stage, or of the whole run with --final. Passing a
    /// stage records it in the run state. Exits non-zero when something is missing.
    Check(CheckArgs),
    /// Hook: Stop and SubagentStop. Blocks the stop while `gate check --final` fails.
    StopHook(StopHookArgs),
    /// Hook: PreToolUse for Read, Grep and Glob. Denies the archetype files before the
    /// classify gate has passed.
    ReadHook,
    /// Hook: PreToolUse for Write, Edit and NotebookEdit. Denies writes to protected run files.
    WriteHook,
    /// Hook: PreToolUse for Bash. Runs both checks on the command text (best effort).
    BashHook,
}

/// Flags of `dla gate check`. The worker is the global `--worker`.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// intake | probe | sweep | classify | confirm | attribute | rule_out
    #[arg(long, value_parser = parse_enum::<Stage>, conflicts_with = "final_gate", required_unless_present = "final_gate")]
    pub stage: Option<Stage>,
    /// Check everything the report needs.
    #[arg(long = "final")]
    pub final_gate: bool,
}

/// Flags of `dla gate stop-hook`.
#[derive(Args, Debug)]
pub struct StopHookArgs {
    /// SubagentStop: gate the worker named in the subagent transcript, not the run.
    #[arg(long)]
    pub subagent: bool,
}

/// What a `dla gate` subcommand prints.
pub enum GateOut {
    /// A JSON document and exit 0.
    Json(Value),
    /// Exact stdout, stderr and exit code (the hooks, and a failing check).
    Raw {
        /// Printed on stdout.
        stdout: String,
        /// Printed on stderr.
        stderr: String,
        /// Exit code.
        code: u8,
    },
}

/// Runs a `dla gate` subcommand. The hooks resolve the run themselves, so `ctx` is only
/// called by `check`.
pub fn run(project: Option<&Path>, ctx: &dyn Fn() -> Res<Ctx>, cmd: GateCmd) -> Res<GateOut> {
    let hook = match cmd {
        GateCmd::Check(a) => return check(&ctx()?, &a),
        GateCmd::StopHook(a) => Hook::Stop {
            subagent: a.subagent,
        },
        GateCmd::ReadHook => Hook::Read,
        GateCmd::WriteHook => Hook::Write,
        GateCmd::BashHook => Hook::Bash,
    };
    let out = gate_hooks::run(project, hook);
    Ok(GateOut::Raw {
        stdout: out.stdout,
        stderr: out.stderr,
        code: out.code,
    })
}

/// Name of the file holding the full list of a gate: `gate/<stage>.json`, with the worker
/// in the name in worker mode.
fn list_file(target: GateTarget, worker: Option<&str>) -> String {
    match worker {
        Some(w) => format!("gate/{}.{w}.json", target.name()),
        None => gate::pointer_for(target),
    }
}

fn check(ctx: &Ctx, a: &CheckArgs) -> Res<GateOut> {
    let target = match (a.stage, a.final_gate) {
        (Some(s), false) => GateTarget::Stage(s),
        _ => GateTarget::Final,
    };
    let worker = ctx.worker.as_deref();
    let view = RunView::load_for(&ctx.run_dir, worker)?;
    // The derived dispositioned counts go back to coverage.json (core mode only: a worker
    // sees its own slice, not the run).
    if worker.is_none() {
        gate::write_back_accounting(&ctx.run_dir, &gate::accounting(&view))?;
    }
    let all = gate::missing(&view, target, worker);
    let mut report = gate::evaluate(&view, target, worker);
    let file = list_file(target, worker);
    report.pointer = ctx.pointer(&file);
    let full = json!({
        "ok": report.ok,
        "stage": report.stage,
        "worker": worker,
        "missing_total": all.len(),
        "missing": all,
        "checked_at": dla_core::state::now_utc(),
    });
    let path = ctx.run_dir.join(&file);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| dla_core::Error::io(dir, e))?;
    }
    write_atomic(&path, to_pretty_json(&full).as_bytes())?;

    let mut out = serde_json::to_value(&report).unwrap_or(Value::Null);
    if let Some(w) = worker {
        out["worker"] = json!(w);
    }
    if report.ok
        && let GateTarget::Stage(s) = target
        && worker.is_none()
    {
        // Release the replayed view's files before the state is rewritten.
        let passed = RunState::update(&ctx.run_dir, |st| {
            st.record_stage_passed(s);
            Ok(st
                .stages_passed
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<_>>()
                .join(","))
        })?;
        out["stages_passed"] = json!(passed.split(',').collect::<Vec<_>>());
        out["recorded"] = json!(true);
    }
    if report.ok {
        return Ok(GateOut::Json(out));
    }
    let first = report
        .missing
        .first()
        .map(|m| format!(" First: {} ({}). Fix: {}", m.id, m.description, m.fix))
        .unwrap_or_default();
    Ok(GateOut::Raw {
        stdout: crate::out::bound(out).to_string(),
        stderr: format!(
            "dla gate check --{}: {} item(s) missing.{first}",
            if matches!(target, GateTarget::Final) {
                "final".to_owned()
            } else {
                format!("stage {}", target.name())
            },
            report.missing_total,
        ),
        code: 2,
    })
}
