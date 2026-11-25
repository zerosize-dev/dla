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
    /// With --residual: write the residual row of every series that has none, exactly as
    /// `dla gap <S> --residual` does for each, and print one bounded summary.
    #[arg(long, requires = "residual", conflicts_with = "series")]
    all: bool,
}

/// Evidence authority of a series: live for a sampled process, dump for a series of
/// heap dumps, test otherwise.
pub(crate) fn series_authority(s: &SeriesFile) -> Authority {
    match s.source.kind {
        SeriesSourceKind::Sample => Authority::Live,
        SeriesSourceKind::Dump => Authority::Dump,
        _ => Authority::Test,
    }
}

/// Series ids in the run, oldest first.
fn series_ids(run_dir: &Path) -> Vec<(u32, String)> {
    let Ok(rd) = fs::read_dir(run_dir.join(SERIES_DIR)) else {
        return Vec::new();
    };
    let mut ids: Vec<(u32, String)> = rd
        .filter_map(Result::ok)
        // An empty file is an id reserved by a writer that has not finished.
        .filter(|e| dla_core::fsutil::is_written(&e.path()))
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
    if a.all {
        return run_all(&tc);
    }
    let series = pick(&tc, a.series.as_deref())?;
    if a.residual {
        let derived = derived_of(&tc, &series.id)?;
        residual::run(&tc, &series, derived.as_ref())
    } else {
        split::run(&tc, &series)
    }
}

/// Writes the residual rows of `series` exactly as `dla gap <S> --residual` does (the gap series
/// derived from it is used when there is one). `Ok(None)` when the series has no byte column to
/// total (`residual-unavailable`): nothing is written then. Used by `dla trend`.
pub fn write_residual_rows(tc: &ToolCtx, series: &SeriesFile) -> Res<Option<Value>> {
    let derived = derived_of(tc, &series.id)?;
    match residual::run(tc, series, derived.as_ref()) {
        Ok(v) => Ok(Some(v)),
        Err(f) if f.refusal.code == "residual-unavailable" => Ok(None),
        Err(f) => Err(f),
    }
}

/// Series listed by name in the summary of `--residual --all`; the rest are counted.
const SHOWN: usize = 8;

/// `dla gap --residual --all`: the residual row of every series that has none. Each series goes
/// through the same code as `dla gap <S> --residual` (including the gap series derived from
/// it, when there is one); a series with no byte column to total is skipped and named.
fn run_all(tc: &ToolCtx) -> Res<Value> {
    let have: std::collections::BTreeSet<String> =
        dla_core::coverage::CoverageManifest::load(&tc.run_dir)?
            .map(|m| m.residual.iter().filter_map(|r| r.series.clone()).collect())
            .unwrap_or_default();
    let (mut written, mut skipped) = (Vec::new(), Vec::new());
    let mut above: Vec<Value> = Vec::new();
    let mut covered = 0usize;
    for (_, id) in series_ids(&tc.run_dir) {
        let s = SeriesFile::load(&tc.run_dir, &id)?;
        if s.source.kind == SeriesSourceKind::Gap {
            continue;
        }
        if have.contains(&id) {
            covered += 1;
            continue;
        }
        let derived = derived_of(tc, &id)?;
        match residual::run(tc, &s, derived.as_ref()) {
            Ok(v) => {
                for row in v["rows"].as_array().into_iter().flatten() {
                    let over = row["residual"].as_f64().unwrap_or(0.0)
                        > row["threshold"].as_f64().unwrap_or(0.0);
                    if over {
                        above.push(serde_json::json!({
                            "series": id,
                            "region": row["region"],
                            "open_item": row["open_item"],
                        }));
                    }
                }
                written.push(id);
            }
            Err(f) if f.refusal.code == "residual-unavailable" => skipped.push(id),
            Err(f) => return Err(f),
        }
    }
    let list = |v: &[String]| -> Vec<String> { v.iter().take(SHOWN).cloned().collect() };
    let mut out = serde_json::json!({
        "ok": true,
        "command": "gap",
        "residual": true,
        "all": true,
        "written": written.len(),
        "series": list(&written),
        "already_had_a_row": covered,
        "no_byte_column": skipped.len(),
        "above_threshold": above.iter().take(SHOWN).collect::<Vec<_>>(),
    });
    if written.len() > SHOWN {
        out["series_more"] = serde_json::json!(written.len() - SHOWN);
    }
    if !skipped.is_empty() {
        out["skipped"] = serde_json::json!(list(&skipped));
    }
    if above.len() > SHOWN {
        out["above_threshold_more"] = serde_json::json!(above.len() - SHOWN);
    }
    out["next"] = serde_json::json!(if above.is_empty() {
        "no region has unexplained growth over its threshold; dla gate check --final lists what is left".to_owned()
    } else {
        "each series above its threshold has an open item: dla ledger resolve <O> --by <F> or --as unresolved --reason inconclusive --remedy \"<text>\"; dla gate check --final prints the fix per item".to_owned()
    });
    Ok(out)
}
