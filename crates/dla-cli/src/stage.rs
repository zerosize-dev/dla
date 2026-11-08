//! Shared plumbing for the stage commands (`probe`, `sweep`, `census`, `workload`,
//! `install`): build the [`ToolCtx`] a crate writes through and print its outcome.

use std::path::{Path, PathBuf};

use dla_core::toolctx::{StageOutcome, ToolCtx};
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res, git};

/// The context handed to a stage crate.
pub fn tool_ctx(project: &Path, ctx: &Ctx, data: Option<&PathBuf>) -> ToolCtx {
    let source_commit =
        git::head_commit(project).unwrap_or_else(|| dla_core::paths::to_slash(project));
    ToolCtx {
        project: project.to_path_buf(),
        run_dir: ctx.run_dir.clone(),
        run_id: ctx.run_id.clone(),
        source_commit,
        data_flag: data.cloned(),
        worker: ctx.worker.clone(),
    }
}

/// Turns a stage crate's result into the command's JSON object. `extra` is merged in
/// (for example `{"lang": "go"}`).
pub fn finish(
    command: &str,
    extra: Value,
    result: Result<StageOutcome, dla_core::Refusal>,
) -> Res<Value> {
    let out = result?;
    let mut v = json!({
        "ok": true,
        "command": command,
        "evidence": out.evidence,
        "candidates": out.candidates,
        "skipped_duplicates": out.skipped_duplicates,
        "artifacts": out.artifacts,
        "summary": out.summary,
    });
    if let (Value::Object(m), Value::Object(e)) = (&mut v, extra) {
        m.extend(e);
    }
    Ok(v)
}
