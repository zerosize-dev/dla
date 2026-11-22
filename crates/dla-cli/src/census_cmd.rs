//! `dla census --lang <l>` → [`dla_census::census`].

use std::path::{Path, PathBuf};

use clap::Args;
use dla_core::runtime::Runtime;
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res, parse::parse_enum, stage};

/// Flags of `dla census`.
#[derive(Args, Debug)]
pub struct CensusArgs {
    /// Runtime to enumerate: jvm | go | rust | node | python | native.
    #[arg(long, value_parser = parse_enum::<Runtime>)]
    lang: Runtime,
}

/// Runs `dla census`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: CensusArgs) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    let mut out = stage::finish(
        "census",
        json!({"lang": a.lang.as_str()}),
        dla_census::census(&tc, a.lang),
    )?;
    let found = |k: &str| out[k].as_u64().unwrap_or(0) > 0;
    if a.lang == Runtime::Jvm
        && (found("candidates") || found("skipped_duplicates"))
        && let Some(note) = discover_for_the_queue(ctx, &tc)
    {
        out["workloads"] = json!(note);
    }
    Ok(out)
}

/// The measurement queue of a JVM project starts with the project's own tests in a loop, and it
/// reads the tests from the workload discovery result. A run that has none gets one here,
/// exactly as `dla workload discover` would write it (files are read, nothing is run), so the
/// first pass does not depend on a step the caller may skip. Returns the line the census
/// output carries; `None` when a result is already there or discovery was refused (the census
/// stands either way, and `dla workload discover` says why).
fn discover_for_the_queue(ctx: &Ctx, tc: &dla_core::toolctx::ToolCtx) -> Option<String> {
    let file = tc
        .run_dir
        .join("workload")
        .join(dla_workload::report_file(ctx.worker.as_deref()));
    if file.exists() {
        return None;
    }
    let lang = crate::workload_cmd::effective_lang(None, ctx.worker.as_deref());
    let found = dla_workload::discover(tc, lang).ok()?;
    Some(format!(
        "discovered for the measurement queue ({} workloads, evidence {}): the queue of a JVM project starts with the project's own tests; dla workload discover repeats it",
        found.summary["workloads"].as_u64().unwrap_or(0),
        found.evidence.join(", ")
    ))
}
