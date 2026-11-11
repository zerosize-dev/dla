//! `dla intervene <H> --finding F (--stub-writer FILE:LINE... | --patch FILE.diff) [--rerun-control]`.
//!
//! Edits a clone of a harness, runs the control's points on the clone and judges whether the
//! growth the finding was confirmed on went away. The project and the harness the finding
//! was confirmed on are never changed: the edit lands in the new harness's `work/` copy.
//!
//! # Steps
//!
//! 1. Preconditions: the finding is confirmed and leak-side; one of its cited passing growth
//!    records is a trend of this harness with a passing column tied to the finding (the
//!    control).
//! 2. The edit is planned against the harness's `work/` (read only) and checked against the
//!    finding's recorded sites ([`intervene::plan`]). A refusal here leaves no clone.
//! 3. The harness is cloned ([`intervene::clone`]), the edit written into the clone, and the
//!    clone built and run with the control's mode, point count and warm-up.
//! 4. The treated series is analysed with the control trend's thresholds and judged by
//!    [`dla_stats::judge_intervention`].
//! 5. One `intervention` evidence record is written, whatever the effect. Only
//!    `growth-removed` with every guard true can make the finding `strong`; `growth-remains`
//!    and `inconclusive` never refute anything.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    EvidenceId, Finding, FindingId, FindingState, Ledger, MAX_RESULT_BYTES, attribution,
    evidence::{Authority, EvidenceKind, Signal, tool_class},
    finding::Polarity,
    fsutil::{reserve_numbered, to_pretty_json, write_atomic},
    series::{KeyMode, SeriesFile, SeriesSourceKind},
    toolctx::{EvidenceSpec, ToolCtx},
    trend::{ColumnTrend, GrowthSummary, TrendReport, probe_column_matches_owner},
};
use dla_stats::{
    ColumnView, Effect, GateThresholds, GateVerdict, Guards, Treated, intervene::REMAINS_FRACTION,
    judge_intervention,
};
use dla_workload::harness::run::{BUILD_MARKER, RunOptions, run_points};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
    stage,
    trend_cmd::analysis,
};

#[path = "intervene/clone.rs"]
mod clone;
#[path = "intervene/diff.rs"]
mod diff;
#[path = "intervene/plan.rs"]
mod plan;

/// Refusal codes of `dla intervene`.
pub mod codes {
    /// The finding is not confirmed.
    pub const STATE: &str = "intervene-state";
    /// The finding is not leak-side.
    pub const LEAK_ONLY: &str = "intervene-leak-only";
    /// No cited growth record of this harness has a passing column tied to the finding.
    pub const NO_CONTROL: &str = "intervene-no-control";
    /// A `--stub-writer` site is not a recorded writer of the finding.
    pub const NOT_WRITER: &str = "intervene-not-writer";
    /// The line is not a standalone call, expression or assignment statement.
    pub const NOT_STATEMENT: &str = "intervene-not-statement";
    /// The edit touches a file or a place it may not.
    pub const SCOPE: &str = "intervene-scope";
    /// The diff cannot be read or does not apply.
    pub const PATCH: &str = "intervene-patch";
    /// The diff changes too many lines.
    pub const TOO_LARGE: &str = "intervene-too-large";
}

/// Directory of intervention raw files (`intervene/I<n>/`).
pub const INTERVENE_DIR: &str = "intervene";
/// The sentence every intervention result carries.
const SCRATCH_NOTE: &str = "tested in a scratch copy; the project was not changed";

/// Flags of `dla intervene`.
#[derive(Args, Debug)]
pub struct InterveneArgs {
    /// The harness the finding was confirmed on (H1). It is cloned; the project and this
    /// harness are never changed.
    pub harness: String,
    /// The confirmed finding under test (F3).
    #[arg(long, value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// `file:line` of a writer statement to replace with a no-op (repeatable).
    #[arg(
        long = "stub-writer",
        value_name = "FILE:LINE",
        conflicts_with = "patch",
        required_unless_present = "patch"
    )]
    pub stub_writers: Vec<String>,
    /// A unified diff to apply to the scratch copy (files under the harness `work/` only).
    #[arg(long, value_name = "FILE.diff")]
    pub patch: Option<PathBuf>,
    /// Also re-run the unedited clone once, to check the control reproduces.
    #[arg(long)]
    pub rerun_control: bool,
}

/// The control: the trend the finding was confirmed on.
struct Control {
    /// The growth evidence that holds the trend.
    growth: EvidenceId,
    series: SeriesFile,
    report: TrendReport,
    column: ColumnTrend,
}

fn view(c: &ColumnTrend) -> ColumnView {
    ColumnView {
        slope: c.sen.map(|s| s.slope),
        lo: c.sen.map(|s| s.lo),
        gate: c.gate,
        shape: c.shape,
    }
}

/// The finding's control: the newest cited passing growth record whose trend is of harness
/// `h` and has a passing column tied to the finding.
fn find_control(tc: &ToolCtx, ledger: &Ledger, f: &Finding, h: &str) -> Result<Control, Failure> {
    let mut seen: Vec<String> = Vec::new();
    for id in f.supports.iter().rev() {
        let Some(e) = ledger.evidence_by_id(id) else {
            continue;
        };
        if !dla_core::machine::is_passing_growth(e) {
            continue;
        }
        let Some(s) = GrowthSummary::from_result(&e.result) else {
            continue;
        };
        let (Some(trend), Some(series_id)) = (s.trend.as_deref(), s.series.as_deref()) else {
            continue;
        };
        let Some(col) = s
            .passed
            .iter()
            .find(|c| c.tied_to_owner(&f.id, f.site.as_deref(), f.owner.as_deref()))
        else {
            continue;
        };
        let Ok(series) = SeriesFile::load(&tc.run_dir, series_id) else {
            continue;
        };
        if series.source.harness.as_deref() != Some(h) {
            seen.push(format!(
                "{id} ({series_id} is of {})",
                series.source.harness.as_deref().unwrap_or("no harness")
            ));
            continue;
        }
        let Ok(report) = TrendReport::load(&tc.run_dir, trend) else {
            continue;
        };
        let Some(column) = report
            .columns
            .iter()
            .find(|c| c.name == col.column && c.gate == GateVerdict::Pass)
            .cloned()
        else {
            continue;
        };
        return Ok(Control {
            growth: id.clone(),
            series,
            report,
            column,
        });
    }
    Err(Failure::refuse(
        codes::NO_CONTROL,
        format!(
            "{} cites no passing growth record of {h} with a column tied to it{}",
            f.id,
            if seen.is_empty() {
                String::new()
            } else {
                format!(" ({})", seen.join("; "))
            }
        ),
        format!(
            "a trend of a {h} series with a passing column tied to {}",
            f.id
        ),
        format!(
            "dla harness run {h}   then   dla trend <series> --finding {}   then   dla ledger confirm {} --evidence <trend evidence id>",
            f.id, f.id
        ),
    ))
}

fn sites_of(list: &[String]) -> Vec<plan::Site> {
    let mut out: Vec<plan::Site> = list.iter().filter_map(|s| plan::Site::parse(s)).collect();
    out.sort();
    out.dedup();
    out
}

/// The sites an edit may touch, from annotations, the census and the finding.
fn anchors(ctx: &Ctx, project: &Path, ledger: &Ledger, f: &Finding) -> plan::Anchors {
    let (aw, ar) = ledger.recorded_sites(&f.id);
    let resolved = f
        .owner
        .as_deref()
        .and_then(|o| attribution::resolve(&ctx.run_dir, project, o));
    let mut writers = aw;
    let mut removers = ar;
    let mut declared: Vec<String> = Vec::new();
    if let Some(r) = &resolved {
        writers.extend(r.writers.iter().cloned());
        removers.extend(r.removers.iter().cloned());
        declared.push(r.site.clone());
    }
    if let Some(s) = &f.site {
        writers.push(s.clone());
        declared.push(s.clone());
    }
    let mut near: Vec<String> = writers.clone();
    near.extend(removers);
    near.extend(declared);
    plan::Anchors {
        finding: f.id.to_string(),
        writers: sites_of(&writers),
        near: sites_of(&near),
        owner: f.owner.clone(),
    }
}

fn sha_hex(parts: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    hex::encode(h.finalize())
}

fn same_points(a: &SeriesFile, b: &SeriesFile) -> bool {
    a.source.mode == b.source.mode
        && a.warmup == b.warmup
        && a.points.len() == b.points.len()
        && a.points
            .iter()
            .zip(&b.points)
            .all(|(p, q)| (p.x - q.x).abs() < 1e-9)
}

/// What one run of the clone gave.
struct Measured {
    series: SeriesFile,
    trend: Vec<ColumnTrend>,
    run_ok: bool,
    limitations: Vec<String>,
}

enum Ran {
    Ok(Box<Measured>),
    /// The harness failed (`harness-failed`); `build` says whether it was the build.
    Failed {
        build: bool,
        why: String,
    },
}

fn run_clone(
    tc: &ToolCtx,
    manifest: &dla_core::harness::HarnessManifest,
    ctl: &Control,
    th: &GateThresholds,
    mode: KeyMode,
) -> Res<Ran> {
    let post = ctl
        .series
        .points
        .len()
        .saturating_sub(ctl.series.warmup as usize);
    let opts = RunOptions {
        modes: vec![mode],
        iterations: Some(u32::try_from(post).unwrap_or(u32::MAX)),
        source_kind: SeriesSourceKind::Intervention,
        pace_seed: control_pace(ctl, manifest.mode),
        ..RunOptions::default()
    };
    let out = match run_points(tc, manifest, &opts) {
        Ok(o) => o,
        Err(r) if r.code == "harness-failed" => {
            return Ok(Ran::Failed {
                build: r.message.contains("did not build"),
                why: r.message,
            });
        }
        Err(r) => return Err(r.into()),
    };
    let Some(id) = out.series.first() else {
        return Ok(Ran::Failed {
            build: false,
            why: format!("harness {} produced no series", manifest.id),
        });
    };
    let series = SeriesFile::load(&tc.run_dir, id)?;
    let idx: Vec<usize> = (0..series.columns.len()).collect();
    let trend = analysis::analyze_series(&series, &idx, th);
    // A run the deadline cut short is not a run of the plan: no verdict from it.
    let run_ok = !out.exits.is_empty()
        && out.exits.iter().all(|(_, c)| *c == Some(0))
        && out.partial.is_empty();
    let mut limitations = out.limitations;
    limitations.extend(out.gaps);
    Ok(Ran::Ok(Box::new(Measured {
        series,
        trend,
        run_ok,
        limitations,
    })))
}

/// The control's `(x, seconds)` pairs: what a run of the clone is expected to cost.
fn control_pace(ctl: &Control, mode: dla_core::harness::HarnessMode) -> Vec<(f64, f64)> {
    dla_workload::harness::run::pace_samples(&ctl.series, mode)
}

/// Refuses before anything is cloned when the unedited rerun and the edited run are
/// expected to take longer than the run has left.
fn require_time_for_runs(
    tc: &ToolCtx,
    ctl: &Control,
    src: &dla_core::harness::HarnessManifest,
    runs: u32,
) -> Res<()> {
    let Some((left, deadline)) = tc.time_left() else {
        return Ok(());
    };
    let mut pace = dla_workload::harness::pace::Pace::new();
    for (x, secs) in control_pace(ctl, src.mode) {
        pace.push(x, secs);
    }
    let one: f64 = if src.mode == dla_core::harness::HarnessMode::Command {
        ctl.series.points.iter().map(|p| pace.predict(p.x)).sum()
    } else {
        pace.predict(ctl.series.points.last().map_or(0.0, |p| p.x))
    };
    let left = left.as_secs_f64();
    let usable = (left - dla_workload::harness::pace::reserve_secs(left)).max(0.0);
    if one * f64::from(runs) <= usable {
        return Ok(());
    }
    let suggestion = if runs > 1 && one <= usable {
        format!(
            "dla intervene {} --finding <F> ...   (without --rerun-control: one run of about {} s)",
            src.id,
            one.ceil() as u64
        )
    } else {
        String::new()
    };
    Err(tc
        .time_budget_refusal(
            &format!("dla intervene {}", src.id),
            (one * f64::from(runs)).ceil() as u64,
            usable.floor() as u64,
            &deadline,
            &suggestion,
        )
        .into())
}

fn column_json(series: &str, trend: &str, name: &str, c: Option<&ColumnTrend>) -> Value {
    match c {
        Some(c) => json!({
            "series": series, "trend": trend, "column": name,
            "slope": c.sen.map(|s| round(s.slope)),
            "lo": c.sen.map(|s| round(s.lo)),
            "gate": c.gate, "shape": c.shape,
        }),
        None => json!({"series": series, "trend": trend, "column": name, "present": false}),
    }
}

fn round(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The control's passing columns that are not the finding's: another holder or `os.rss`.
fn other_passing<'a>(ctl: &'a Control, f: &Finding) -> Vec<&'a ColumnTrend> {
    ctl.report
        .columns
        .iter()
        .filter(|c| c.gate == GateVerdict::Pass && c.name != ctl.column.name)
        .filter(|c| {
            !(c.finding.as_ref() == Some(&f.id)
                || c.candidates.contains(&f.id)
                || f.site.is_some() && c.site == f.site
                || f.owner
                    .as_deref()
                    .is_some_and(|o| probe_column_matches_owner(&c.name, o)))
        })
        .collect()
}

fn present(t: &ColumnTrend) -> bool {
    t.n > 0
}

/// The kind part of a column name that embeds an identity: the text before `@` of a site
/// group (`alloc@Foo.java:41`), or the prefix of a holder or probe column.
fn column_kind(name: &str) -> Option<&str> {
    if let Some((base, _)) = name.split_once('@') {
        return Some(base);
    }
    ["probe:", "holder:"]
        .into_iter()
        .find(|p| name.starts_with(p))
}

/// A column of the treated series that looks like the control column under another name:
/// the same kind, not a column the control series had. A moved site line or a renamed
/// owner produces one; the growth was then renamed, not removed.
fn renamed_twin<'a>(
    control: &str,
    control_columns: &[ColumnTrend],
    treated: &'a [ColumnTrend],
) -> Option<&'a str> {
    let kind = column_kind(control)?;
    treated
        .iter()
        .filter(|t| present(t) && t.name != control)
        .filter(|t| column_kind(&t.name) == Some(kind))
        .find(|t| !control_columns.iter().any(|c| c.name == t.name))
        .map(|t| t.name.as_str())
}

/// Columns read from the operating system or derived from it for the whole process or
/// container: they exist whatever the edit did, so they never prove the run worked. The
/// agent's own totals (`jvm.heap_used`, `go.heap_live`) do: only a running agent writes them.
fn whole_process(name: &str) -> bool {
    ["os.", "cgroup.", "fd.", "gap."]
        .into_iter()
        .any(|p| name.starts_with(p))
}

/// True when the same source as the control column (the in-process agent or the allocator
/// shim) still produced a column in the treated run, so a missing control column means the
/// edit removed what wrote it. `treated` lists `(name, source, has data)` per treated
/// column. Whole-process columns (`os.*`, `cgroup.*`, `fd.*`, `gap.*`: read from the operating system, not by the agent or shim) never count, and a
/// control that is itself one has no source to prove anything.
fn source_alive(
    control: &str,
    control_source: Option<&str>,
    treated: &[(&str, &str, bool)],
) -> bool {
    if whole_process(control) {
        return false;
    }
    let Some(src) = control_source else {
        return false;
    };
    treated
        .iter()
        .any(|(n, s, there)| *there && *n != control && !whole_process(n) && *s == src)
}

/// Fits the evidence `result` in [`MAX_RESULT_BYTES`] by shortening the site list.
fn fit(mut v: Value) -> Value {
    for keep in [8usize, 4, 2, 1, 0] {
        if serde_json::to_string(&v).map_or(usize::MAX, |s| s.len()) <= MAX_RESULT_BYTES {
            return v;
        }
        if let Some(Value::Array(a)) = v.get_mut("sites") {
            a.truncate(keep);
        }
    }
    v
}

/// `dla intervene`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: InterveneArgs) -> Res<Value> {
    if dla_core::series::numbered('H', &a.harness).is_none() {
        return Err(Failure::usage(
            format!("{:?} is not a harness ID", a.harness),
            "a harness ID such as H1",
            "dla intervene H1 --finding F1 --stub-writer src/Foo.java:41",
        ));
    }
    let tc = stage::tool_ctx(project, ctx, data);
    tc.require_tier(1, "dla intervene")?;
    tc.require_time("intervene")?;
    let src = tc.harness(&a.harness)?;
    let mut ledger = tc.ledger()?;
    let f = ledger.finding(&a.finding).cloned().ok_or_else(|| {
        Failure::refuse(
            dla_core::error::codes::UNKNOWN_FINDING,
            format!("{} is not in the ledger", a.finding),
            "an existing finding for --finding",
            "dla run status   (lists findings)",
        )
    })?;
    if f.polarity() != Polarity::Leak {
        return Err(Failure::refuse(
            codes::LEAK_ONLY,
            format!(
                "{} is a non-leak hypothesis; an intervention tests a leak-side finding",
                f.id
            ),
            "a leak-side finding",
            "dla run status",
        ));
    }
    if f.state != FindingState::Confirmed {
        return Err(Failure::refuse(
            codes::STATE,
            format!(
                "{} is {}; an intervention needs a confirmed finding",
                f.id, f.state
            ),
            format!("{} confirmed", f.id),
            format!(
                "dla ledger confirm {} --evidence <growth evidence id>",
                f.id
            ),
        ));
    }
    let ctl = find_control(&tc, &ledger, &f, &a.harness)?;
    let anchors = anchors(ctx, project, &ledger, &f);
    let src_dir = dla_workload::harness::harness_dir(&tc.run_dir, &src.id);
    let owned = plan::harness_owned_names(&src_dir);
    let work = src_dir.join(dla_workload::harness::WORK);

    // The control series says how long a run of the clone takes: refuse up front when
    // the run (twice with --rerun-control) does not fit in the time left.
    require_time_for_runs(&tc, &ctl, &src, if a.rerun_control { 2 } else { 1 })?;

    // Plan against the harness's own copy; nothing is cloned if this refuses.
    let edit = match &a.patch {
        Some(p) => {
            let text = fs::read_to_string(p).map_err(|e| dla_core::Error::io(p, e))?;
            plan::patch_plan(&work, &owned, &anchors, &text)?
        }
        None => plan::stub_plan(&work, &owned, &anchors, &a.stub_writers)?,
    };

    // Raw directory first: the diff is kept whatever the run does.
    let (n, raw_dir) = reserve_numbered(&tc.run_dir.join(INTERVENE_DIR), 'I', "", true)
        .map_err(|e| e.to_refusal())?;
    let raw_rel = format!("{INTERVENE_DIR}/I{n}");
    let diff_path = raw_dir.join("patch.diff");
    write_atomic(&diff_path, edit.diff.as_bytes())?;
    let diff_sha = sha_hex(&[edit.diff.as_bytes()]);

    // Clone, then the unedited run (only with --rerun-control), then the edit.
    let cloned = clone::clone_harness(&tc, &src)?;
    let th = ctl.report.thresholds;
    let mode = ctl
        .series
        .source
        .mode
        .or_else(|| src.key_modes.first().copied())
        .unwrap_or(KeyMode::Native);
    let ctl_view = view(&ctl.column);
    let mut limitations: Vec<String> = vec![
        SCRATCH_NOTE.to_owned(),
        "gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders".to_owned(),
        "exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked".to_owned(),
    ];
    if matches!(
        src.runtime,
        dla_core::runtime::Runtime::Native | dla_core::runtime::Runtime::Rust
    ) {
        limitations.push(
            "native and Rust harnesses have no holder probes: the column is a site group from the allocator shim or an os column, and an edit elsewhere may not move os columns past the gate"
                .to_owned(),
        );
    }

    let mut reproduced: Option<bool> = None;
    let mut rerun_json = Value::Null;
    if a.rerun_control {
        match run_clone(&tc, &cloned.manifest, &ctl, &th, mode)? {
            Ran::Ok(m) => {
                let col = m.trend.iter().find(|c| c.name == ctl.column.name);
                let ok = m.run_ok
                    && col.is_some_and(|c| {
                        c.gate == GateVerdict::Pass
                            && c.sen
                                .zip(ctl_view.slope)
                                .is_some_and(|(s, cs)| s.slope >= REMAINS_FRACTION * cs)
                    });
                reproduced = Some(ok);
                rerun_json = column_json(&m.series.id, "", &ctl.column.name, col);
            }
            Ran::Failed { why, .. } => {
                reproduced = Some(false);
                limitations.push(format!("the unedited rerun failed: {}", clip(&why, 120)));
            }
        }
    }

    plan::apply_plan(&cloned.work, &edit)?;
    // The build fingerprint does not cover `work/`: force the rebuild.
    let marker =
        dla_workload::harness::harness_dir(&tc.run_dir, &cloned.manifest.id).join(BUILD_MARKER);
    if marker.exists() {
        fs::remove_file(&marker).map_err(|e| dla_core::Error::io(&marker, e))?;
    }

    let ran = run_clone(&tc, &cloned.manifest, &ctl, &th, mode)?;
    let mut guards = Guards {
        build_ok: true,
        run_ok: true,
        same_points: true,
        edit_scoped: true,
        exercised: None,
        positive_control: None,
        control_reproduced: reproduced,
    };
    let (effect, treated_json, treated_series, source_rel);
    match ran {
        Ran::Failed { build, why } => {
            guards.build_ok = !build;
            guards.run_ok = false;
            guards.same_points = false;
            limitations.push(clip(&why, 200));
            effect = Effect::Invalid;
            treated_json =
                json!({"series": null, "trend": null, "column": ctl.column.name, "present": false});
            treated_series = None;
            source_rel = format!("{raw_rel}/patch.diff");
        }
        Ran::Ok(m) => {
            guards.run_ok = m.run_ok;
            guards.same_points = same_points(&ctl.series, &m.series);
            let others = other_passing(&ctl, &f);
            if !others.is_empty() {
                guards.positive_control = Some(
                    others
                        .iter()
                        .any(|o| m.trend.iter().any(|t| t.name == o.name && present(t))),
                );
            }
            let col = m
                .trend
                .iter()
                .find(|c| c.name == ctl.column.name && present(c));
            let twin = if col.is_none() {
                renamed_twin(&ctl.column.name, &ctl.report.columns, &m.trend)
            } else {
                None
            };
            if let Some(t) = twin {
                limitations.push(format!(
                    "the treated series has {t} where the control had {}: the edit renamed the column (a moved site line or a renamed owner), so the growth is not shown removed",
                    ctl.column.name
                ));
            }
            let source_alive = twin.is_none() && col.is_none() && {
                let control_source = ctl
                    .series
                    .columns
                    .iter()
                    .find(|c| c.name == ctl.column.name)
                    .map(|c| c.source.as_str());
                let treated_cols: Vec<(&str, &str, bool)> = m
                    .series
                    .columns
                    .iter()
                    .map(|sc| {
                        let there = m.trend.iter().any(|t| t.name == sc.name && present(t));
                        (sc.name.as_str(), sc.source.as_str(), there)
                    })
                    .collect();
                source_alive(&ctl.column.name, control_source, &treated_cols)
            };
            let other_columns = if a.patch.is_some() {
                if col.is_none() && twin.is_none() {
                    limitations.push(format!(
                        "the treated series lacks the control column {}: a patch can rename an owner or move a site, so a patch must keep the measured column to be judged",
                        ctl.column.name
                    ));
                }
                false
            } else {
                if col.is_none() && twin.is_none() && !source_alive {
                    limitations.push(format!(
                        "the treated series lacks the control column {} and no column of the same source (the in-process agent or the allocator shim) was produced, so the run does not show the writer was removed",
                        ctl.column.name
                    ));
                }
                source_alive
            };
            effect = judge_intervention(
                &ctl_view,
                &Treated {
                    column: col.map(view),
                    other_columns,
                },
                &guards,
            );
            // The treated trend: a trend report like any other, plus a copy in the raw dir.
            let tid = tc.next_trend_id();
            let report = TrendReport {
                schema_version: dla_core::trend::TREND_SCHEMA_VERSION,
                id: tid.clone(),
                series: m.series.id.clone(),
                thresholds: th,
                columns: m.trend.clone(),
            };
            report.save(&tc.run_dir)?;
            write_atomic(
                &raw_dir.join("trend.json"),
                to_pretty_json(&report).as_bytes(),
            )?;
            treated_json = column_json(&m.series.id, &tid, &ctl.column.name, col);
            limitations.extend(m.limitations.iter().take(2).map(|l| clip(l, 160)));
            source_rel = format!("series/{}.json", m.series.id);
            treated_series = Some(m.series.id.clone());
        }
    }

    let series_bytes = treated_series
        .as_ref()
        .and_then(|id| fs::read(SeriesFile::path_in(&tc.run_dir, id)).ok())
        .unwrap_or_default();
    let signal = Signal::new(
        "dla-intervene",
        tool_class::INTERVENTION,
        &sha_hex(&[edit.diff.as_bytes(), &series_bytes]),
    );
    let control_trend = ctl.report.id.clone();
    let mut guard_json = serde_json::to_value(guards).unwrap_or(Value::Null);
    if let Some(o) = guard_json.as_object_mut()
        && o.get("control_reproduced") == Some(&Value::Null)
    {
        o.remove("control_reproduced");
    }
    let mut result = json!({
        "test": "intervention",
        "kind": edit.kind,
        "finding": f.id.to_string(),
        "harness": a.harness,
        "treated_harness": cloned.manifest.id,
        "sites": edit.sites,
        "diff_sha256": diff_sha,
        "control": column_json(&ctl.series.id, &control_trend, &ctl.column.name, Some(&ctl.column)),
        "treated": treated_json,
        "effect": effect.as_str(),
        "guards": guard_json,
        "signal": signal.to_value(),
    });
    if !rerun_json.is_null() {
        result["rerun"] = rerun_json;
    }
    let result = fit(result);

    let mut spec = EvidenceSpec::new(
        EvidenceKind::Intervention,
        "dla",
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = Authority::Test;
    spec.raw = Some(raw_rel.clone());
    spec.source = Some(tc.artifact_source(&source_rel)?);
    spec.limitations = limitations.clone();
    let evidence = tc.add_evidence(&mut ledger, spec)?;

    let mut out = json!({
        "ok": true,
        "command": "intervene",
        "finding": f.id.to_string(),
        "kind": edit.kind,
        "harness": a.harness,
        "treated_harness": cloned.manifest.id,
        "evidence": evidence.to_string(),
        "effect": effect.as_str(),
        "guards": guard_json,
        "control": column_json(&ctl.series.id, &control_trend, &ctl.column.name, Some(&ctl.column)),
        "treated": treated_json,
        "sites": edit.sites,
        "changed_lines": edit.changed_lines,
        "cloned_files": cloned.files,
        "diff_sha256": diff_sha,
        "artifacts": [tc.pointer(&raw_rel)],
        "note": SCRATCH_NOTE,
        "limitations": limitations.iter().take(4).collect::<Vec<_>>(),
        "next": next_commands(&f, &ctl, &a, effect, &evidence, &guard_json, &edit),
    });
    if !out["treated"]["present"].as_bool().unwrap_or(true) && effect == Effect::GrowthRemoved {
        out["why"] = json!("the treated series has no such column: the edit removed what wrote it");
    }
    Ok(out)
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}...")
    }
}

fn next_commands(
    f: &Finding,
    ctl: &Control,
    a: &InterveneArgs,
    effect: Effect,
    evidence: &EvidenceId,
    guards: &Value,
    edit: &plan::Plan,
) -> Vec<String> {
    match effect {
        Effect::GrowthRemoved => vec![format!("dla ledger strong {} --evidence {evidence}", f.id)],
        Effect::GrowthRemains => vec![
            format!(
                "dla ledger annotate {} --writer <file:line>   (another site adds to the holder, or this one is not the cause)",
                f.id
            ),
            format!(
                "dla intervene {} --finding {} --stub-writer <file:line> ...   (stub every writer in one run)",
                a.harness, f.id
            ),
            "growth-remains does not refute the finding".to_owned(),
        ],
        Effect::Inconclusive => vec![format!(
            "dla intervene {} --finding {} --patch <fix.diff>   (a stronger edit), or leave {} confirmed",
            a.harness, f.id, f.id
        )],
        Effect::Invalid => {
            let failed: Vec<&str> = guards
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(_, v)| v.as_bool() == Some(false))
                .map(|(k, _)| k.as_str())
                .collect();
            vec![format!(
                "guard(s) false: {}; read the logs under harness/<treated>/logs, change the edit ({} line(s) changed) and run dla intervene again; {} stays confirmed (control {} in {})",
                failed.join(", "),
                edit.changed_lines,
                f.id,
                ctl.column.name,
                ctl.growth
            )]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(name: &str) -> ColumnTrend {
        serde_json::from_value(json!({
            "name": name, "unit": "count", "kind": "counter", "region": "heap",
            "n": 8, "noise": 0.0, "rise": 0.0, "shape": "flat", "gate": "fail"
        }))
        .unwrap()
    }

    #[test]
    fn a_moved_site_or_renamed_owner_is_a_twin_not_a_removal() {
        let control = vec![col("alloc@src/Foo.java:41"), col("os.rss")];
        let ctl = "alloc@src/Foo.java:41";
        // The comment line above the writer moved the site.
        let treated = vec![col("alloc@src/Foo.java:42"), col("os.rss")];
        assert_eq!(
            renamed_twin(ctl, &control, &treated),
            Some("alloc@src/Foo.java:42")
        );
        // The writer is gone: only columns the control had remain.
        assert_eq!(renamed_twin(ctl, &control, &[col("os.rss")]), None);
        // A column the control also had is another holder, not a twin.
        let two = vec![col("alloc@src/Foo.java:41"), col("alloc@src/Bar.java:9")];
        assert_eq!(
            renamed_twin(ctl, &two, &[col("alloc@src/Bar.java:9")]),
            None
        );
        // A renamed owner under probe columns.
        let c = vec![col("probe:com.x.Cache.items")];
        assert_eq!(
            renamed_twin(
                "probe:com.x.Cache.items",
                &c,
                &[col("probe:com.x.Store.items")]
            ),
            Some("probe:com.x.Store.items")
        );
        assert_eq!(renamed_twin("os.rss", &control, &treated), None);
    }

    #[test]
    fn a_missing_column_is_removal_only_when_the_same_source_still_worked() {
        let agent = Some("jvm-agent");
        let treated =
            |extra: &[(&'static str, &'static str)]| -> Vec<(&'static str, &'static str, bool)> {
                let mut v = vec![
                    ("os.rss", "proc-status", true),
                    ("cgroup.current", "cgroup", true),
                ];
                v.extend(extra.iter().map(|(n, s)| (*n, *s, true)));
                v
            };
        for control in [
            "stack:shop.Cache.put",
            "type:shop.Item",
            "alloc@src/Foo.java:41",
            "probe:shop.Cache.items",
            "holder:shop.Cache.items",
        ] {
            // The agent still produced another column: the writer is gone.
            let t = treated(&[("probe:shop.Other.items", "jvm-agent")]);
            assert!(source_alive(control, agent, &t), "{control}");
            // A total written by the same agent shows the agent ran.
            let total = treated(&[("jvm.heap_used", "jvm-agent")]);
            assert!(source_alive(control, agent, &total), "{control}");
            // Only whole-process columns, or another source: no proof.
            assert!(!source_alive(control, agent, &treated(&[])), "{control}");
            let other = treated(&[("go.heap_live", "go-harness")]);
            assert!(!source_alive(control, agent, &other), "{control}");
            // A same-source column without data does not count.
            let empty = vec![("probe:shop.Other.items", "jvm-agent", false)];
            assert!(!source_alive(control, agent, &empty), "{control}");
        }
        // The shim family: os.* and fd.* never count, an allocator column does.
        let shim = Some("shim");
        let only_os = vec![("os.rss", "shim", true), ("fd.socket", "shim", true)];
        assert!(!source_alive("alloc@src/a.c:9", shim, &only_os));
        let with_site = vec![("alloc@src/b.c:3", "shim", true)];
        assert!(source_alive("alloc@src/a.c:9", shim, &with_site));
        let totals = vec![("native.live_bytes", "shim", true)];
        assert!(source_alive("alloc@src/a.c:9", shim, &totals));
    }

    #[test]
    fn an_os_rss_only_control_is_never_proven_removed() {
        let t = vec![
            ("os.rss", "proc-status", true),
            ("os.fds", "proc-fd", true),
            ("jvm.heap_used", "proc-status", true),
        ];
        assert!(!source_alive("os.rss", Some("proc-status"), &t));
        assert!(!source_alive("cgroup.current", Some("cgroup"), &t));
        assert!(!source_alive("alloc@a.c:1", None, &t));
    }
}
