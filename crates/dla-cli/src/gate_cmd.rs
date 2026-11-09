//! `dla gate check | stop-hook | read-hook | write-hook | bash-hook`.
//!
//! Skeleton: the flags are final, the commands refuse with `not-implemented`. The gates
//! package (M6 package D) owns this file and replaces the stubs.

use clap::{Args, Subcommand};
use dla_core::{error::codes as core_codes, state::Stage};
use serde_json::Value;

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
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

fn not_implemented(what: &str) -> Failure {
    Failure::refuse(
        core_codes::NOT_IMPLEMENTED,
        format!("`{what}` is not implemented in this build"),
        what,
        "use a dla build that includes the gates (M6 package D)",
    )
}

/// Runs a `dla gate` subcommand. The hooks need no run for their own argument parsing, so
/// the run is resolved by the real implementation, not here.
pub fn run(_ctx: &dyn Fn() -> Res<Ctx>, cmd: GateCmd) -> Res<Value> {
    Err(not_implemented(match cmd {
        GateCmd::Check(_) => "dla gate check",
        GateCmd::StopHook(_) => "dla gate stop-hook",
        GateCmd::ReadHook => "dla gate read-hook",
        GateCmd::WriteHook => "dla gate write-hook",
        GateCmd::BashHook => "dla gate bash-hook",
    }))
}
