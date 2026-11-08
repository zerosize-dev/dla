//! `dla workload discover [--lang <l>]` → [`dla_workload::discover`].

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use dla_core::runtime::Runtime;
use serde_json::{Value, json};

use crate::{ctx::Ctx, fail::Res, parse::parse_enum, stage};

/// `dla workload ...`
#[derive(Subcommand, Debug)]
pub enum WorkloadCmd {
    /// Find the tests, mains and dev servers a loop can run, ranked by entry points.
    Discover(DiscoverArgs),
}

/// Flags of `dla workload discover`.
#[derive(Args, Debug)]
pub struct DiscoverArgs {
    /// Limit to one runtime: jvm | go | rust | node | python | native (default: all detected).
    #[arg(long, value_parser = parse_enum::<Runtime>)]
    lang: Option<Runtime>,
}

/// Runs `dla workload discover`.
pub fn discover(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: DiscoverArgs) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    let lang = effective_lang(a.lang, ctx.worker.as_deref());
    stage::finish(
        "workload discover",
        json!({"lang": lang.map(|l| l.as_str())}),
        dla_workload::discover(&tc, lang),
    )
}

/// `--lang`, else the worker's own runtime (a worker's slice only speaks for its
/// runtime, as `dla probe` does), else every detected runtime.
fn effective_lang(explicit: Option<Runtime>, worker: Option<&str>) -> Option<Runtime> {
    explicit.or_else(|| worker.and_then(|w| w.parse::<Runtime>().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_worker_discovers_only_its_own_runtime() {
        assert_eq!(effective_lang(None, Some("go")), Some(Runtime::Go));
        assert_eq!(
            effective_lang(Some(Runtime::Rust), Some("go")),
            Some(Runtime::Rust)
        );
        assert_eq!(effective_lang(None, Some("main")), None);
        assert_eq!(effective_lang(None, None), None);
    }
}
