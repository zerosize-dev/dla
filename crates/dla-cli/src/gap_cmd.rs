//! `dla gap [<S>] [--residual]`.
//!
//! Without `--residual`: splits whole-process memory (`os.rss`) of a harness series into
//! the part the runtime accounts for and the rest (`gap.unexplained`), writes the split
//! as a derived series, trends both columns and adds one `non-heap` evidence record.
//!
//! With `--residual`: totals the growth of each region (heap, off-heap, non-heap), subtracts
//! what confirmed and strong findings explain, writes the `coverage.residual` rows, keeps
//! the open items for regions over the threshold and adds one `residual` evidence record.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    RunState,
    error::codes,
    evidence::Authority,
    series::{SERIES_DIR, SeriesFile, SeriesSourceKind, numbered},
    toolctx::ToolCtx,
};
use serde_json::Value;

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    stage,
};

#[path = "gap/fit.rs"]
mod fit;
#[path = "gap/residual.rs"]
mod residual;
#[path = "gap/split.rs"]
mod split;

/// Refusal code when the series lacks the columns the split needs.
pub(crate) const GAP_UNAVAILABLE: &str = "gap-unavailable";

/// Flags of `dla gap`.
#[derive(Args, Debug)]
pub struct GapArgs {
    /// The series (S3); default: the newest harness series of this run.
    series: Option<String>,
    /// Instead of splitting, total the growth of each region, subtract what confirmed
    /// findings explain and write the rest as coverage residual rows and open items.
    #[arg(long)]
    residual: bool,
}

/// Evidence authority of a series: live for a sampled process, test otherwise.
pub(crate) fn series_authority(s: &SeriesFile) -> Authority {
    if s.source.kind == SeriesSourceKind::Sample {
        Authority::Live
    } else {
        Authority::Test
    }
}

/// Series ids in the run, oldest first.
fn series_ids(run_dir: &Path) -> Vec<(u32, String)> {
    let Ok(rd) = fs::read_dir(run_dir.join(SERIES_DIR)) else {
        return Vec::new();
    };
    let mut ids: Vec<(u32, String)> = rd
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(".json")?.to_owned();
            Some((numbered('S', &id)?, id))
        })
        .collect();
    ids.sort();
    ids
}

/// The series to work on: the named one, else the newest harness series.
fn pick(tc: &ToolCtx, arg: Option<&str>) -> Res<SeriesFile> {
    let ids = series_ids(&tc.run_dir);
    if let Some(id) = arg {
        if !ids.iter().any(|(_, i)| i == id) {
            let have: Vec<&str> = ids.iter().rev().take(8).map(|(_, i)| i.as_str()).collect();
            return Err(Failure::refuse(
                codes::INVALID_VALUE,
                format!("{id:?} is not a series in run {}", tc.run_id),
                "an existing series ID (S3)",
                if have.is_empty() {
                    "dla harness run <H>   to write a series".to_owned()
                } else {
                    format!(
                        "dla gap {}   (series in this run: {})",
                        have[0],
                        have.join(", ")
                    )
                },
            ));
        }
        let s = SeriesFile::load(&tc.run_dir, id)?;
        if s.source.kind == SeriesSourceKind::Gap {
            let from = s.source.derived_from.as_deref().unwrap_or("<S>");
            return Err(Failure::refuse(
                codes::INVALID_VALUE,
                format!("{id} is itself a gap series (derived from {from})"),
                "the series the split was made from",
                format!("dla gap {from}"),
            ));
        }
        return Ok(s);
    }
    for (_, id) in ids.iter().rev() {
        let s = SeriesFile::load(&tc.run_dir, id)?;
        if s.source.kind == SeriesSourceKind::Harness {
            return Ok(s);
        }
    }
    Err(Failure::refuse(
        codes::INVALID_VALUE,
        format!("run {} has no harness series to split", tc.run_id),
        "a series written by dla harness run",
        "dla harness run <H>   (or name a series: dla gap S3)",
    ))
}

/// The newest gap series derived from `id`.
fn derived_of(tc: &ToolCtx, id: &str) -> Res<Option<SeriesFile>> {
    for (_, d) in series_ids(&tc.run_dir).iter().rev() {
        let s = SeriesFile::load(&tc.run_dir, d)?;
        if s.source.kind == SeriesSourceKind::Gap && s.source.derived_from.as_deref() == Some(id) {
            return Ok(Some(s));
        }
    }
    Ok(None)
}

/// Runs `dla gap`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: GapArgs) -> Res<Value> {
    RunState::load(&ctx.run_dir)?;
    let tc = stage::tool_ctx(project, ctx, data);
    let series = pick(&tc, a.series.as_deref())?;
    if a.residual {
        let derived = derived_of(&tc, &series.id)?;
        residual::run(&tc, &series, derived.as_ref())
    } else {
        split::run(&tc, &series)
    }
}
