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
    if matches!(a.lang, Runtime::Jvm | Runtime::Python)
        && (found("candidates") || found("skipped_duplicates"))
        && let Some(note) = discover_for_the_queue(ctx, &tc, a.lang)
    {
        out["workloads"] = json!(note);
    }
    Ok(out)
}

/// The measurement queue of a JVM project starts with the project's own tests in a loop, and it
/// reads the tests from the workload discovery result. A run that has none gets one here,
/// exactly as `dla workload discover` would write it (files are read, nothing is run), so the
/// first pass does not depend on a step the caller may skip.
///
/// The queue of a Python project starts with the loop only when the discovery lists a Python
/// test file, so for Python the result is written only then: a project without test files keeps
/// a run with no discovery result, and the ledger rule that a discovery result turns on does
/// not start to apply to it.
///
/// Returns the line the census output carries; `None` when a result is already there, nothing
/// was written or discovery was refused (the census stands either way, and `dla workload
/// discover` says why).
fn discover_for_the_queue(
    ctx: &Ctx,
    tc: &dla_core::toolctx::ToolCtx,
    lang: Runtime,
) -> Option<String> {
    let file = tc
        .run_dir
        .join("workload")
        .join(dla_workload::report_file(ctx.worker.as_deref()));
    if file.exists() {
        return None;
    }
    let python = lang == Runtime::Python;
    let found = dla_workload::discover_when(
        tc,
        crate::workload_cmd::effective_lang(None, ctx.worker.as_deref()),
        |report| !python || dla_core::testloop::python_tests(report).is_some(),
    )
    .ok()??;
    let tail = if python {
        "the queue of a Python project that has test files starts with the project's own tests"
    } else {
        "the queue of a JVM project starts with the project's own tests"
    };
    Some(format!(
        "discovered for the measurement queue ({} workload(s), evidence {}): {tail}; dla workload discover repeats it",
        found.summary["workloads"].as_u64().unwrap_or(0),
        found.evidence.join(", ")
    ))
}
