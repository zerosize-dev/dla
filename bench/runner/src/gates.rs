//! `dla-bench gates`: the ten release gates of the plan, evaluated on a graded `metrics.json`.
//!
//! The readings of each gate are in `bench/GRADING.md`, "Release gates".

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{
    Res,
    model::{Group, MetricsFile, Ratio},
};

/// Runs of an arm that the means and standard deviations use; later runs (close calls) add
/// to counts but not to the means.
pub const BASE_RUNS: u32 = 3;
/// The context cost may exceed the previous release by this share.
pub const CONTEXT_MARGIN: f64 = 0.10;
/// The core `SKILL.md` must be shorter than this many lines.
pub const SKILL_MAX_LINES: usize = 200;
/// Smoke subset of the context-cost gate (the `smoke` cases of the plugin evals).
pub const SMOKE_FIXTURES: [&str; 3] = [
    "native-vendor-codec",
    "python-report-memo",
    "python-report-memo-bounded",
];

const EPS: f64 = 1e-9;

/// Flags of `dla-bench gates`.
#[derive(Args, Debug)]
pub struct GatesArgs {
    /// A graded `metrics.json`.
    pub metrics: PathBuf,
    /// A baseline file: the M7 baselines and the previous release's metrics.
    #[arg(long)]
    pub baseline: Option<PathBuf>,
    /// Write the baselines of these metrics to this file. The metrics are judged first,
    /// against the values the file already holds (and `--baseline`, when given: the
    /// stricter of the two per value); a baseline already stored is replaced only by a
    /// stricter value. Nothing is written when a gate fails, except with `--baselines-only`.
    /// The previous-release block is replaced with these metrics unless `--baselines-only`
    /// is given.
    #[arg(long)]
    pub write_baseline: Option<PathBuf>,
    /// With `--write-baseline`, the M7 baseline run: write only the three baselines, even when
    /// a gate fails, and keep the file's own previous-release block as it is.
    #[arg(long)]
    pub baselines_only: bool,
    /// A name for the release block written by `--write-baseline`, such as `1.0.0-rc.1`.
    #[arg(long)]
    pub label: Option<String>,
    /// The ablation the gates judge, with archetypes on: the shipped arm.
    #[arg(long, default_value = "full")]
    pub ablation: String,
    /// The ablation of the recall gate (gate 2): its archetypes-on arm is compared with its
    /// archetypes-off arm. `gates` is the only ablation that has an off arm.
    #[arg(long, default_value = "gates")]
    pub recall_ablation: String,
    /// Fixtures of the smoke subset (comma separated).
    #[arg(long, value_delimiter = ',', default_values_t = SMOKE_FIXTURES.map(String::from))]
    pub smoke: Vec<String>,
    /// The core skill file; it must be shorter than 200 lines.
    #[arg(long, default_value = "skills/detect-leak-anywhere/SKILL.md")]
    pub skill: PathBuf,
    /// The archetype data file: every `L-*` table in it is a shipped row and needs a decoy
    /// and control pair. Skipped when the file does not exist.
    #[arg(long, default_value = "data/archetypes.toml")]
    pub rows: PathBuf,
    /// Print JSON instead of a table.
    #[arg(long)]
    pub json: bool,
}

/// Statistics of one ratio metric over the base runs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stat {
    /// Mean of the per-run ratios.
    pub mean: f64,
    /// Population standard deviation of the per-run ratios.
    pub stdev: f64,
    /// Mean denominator of one run.
    pub den_run: f64,
}

/// Mean, stdev and one run's denominator over the per-run entries `1..=base_runs` with a
/// denominator. `None` when no such entry exists.
pub fn stat(r: &Ratio, base_runs: u32) -> Option<Stat> {
    let runs: Vec<(f64, f64)> = r
        .per_run
        .iter()
        .filter(|p| p.run <= base_runs && p.den > 0)
        .map(|p| (p.num as f64 / p.den as f64, p.den as f64))
        .collect();
    if runs.is_empty() {
        return None;
    }
    let k = runs.len() as f64;
    let mean = runs.iter().map(|(v, _)| v).sum::<f64>() / k;
    let q: f64 = runs.iter().map(|(v, _)| (v - mean) * (v - mean)).sum();
    Some(Stat {
        mean,
        stdev: (q / k).sqrt(),
        den_run: runs.iter().map(|(_, d)| d).sum::<f64>() / k,
    })
}

/// Number of base runs that have data in the metric.
fn base_run_count(r: &Ratio, base_runs: u32) -> usize {
    r.per_run
        .iter()
        .filter(|p| p.run <= base_runs && p.den > 0)
        .count()
}

/// Which way a metric is better.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Better {
    /// A higher value is better (recall, precision).
    Higher,
    /// A lower value is better (rates of error).
    Lower,
}

/// The metrics the regression gate compares, with their direction.
pub fn compared_metrics(g: &Group) -> Vec<(&'static str, Better, Option<&Ratio>)> {
    let m = &g.metrics;
    vec![
        ("leak_recall", Better::Higher, Some(&m.leak_recall)),
        (
            "archetype_free_recall",
            Better::Higher,
            Some(&m.archetype_free_recall),
        ),
        ("flagged_recall", Better::Higher, Some(&m.flagged_recall)),
        (
            "second_leak_recall",
            Better::Higher,
            Some(&m.second_leak_recall),
        ),
        (
            "grounded_attribution",
            Better::Higher,
            Some(&m.grounded_attribution),
        ),
        ("label_accuracy", Better::Higher, Some(&m.label_accuracy)),
        ("forced_fit_rate", Better::Lower, Some(&m.forced_fit_rate)),
        ("overclaim_rate", Better::Lower, Some(&m.overclaim_rate)),
        (
            "false_confirmation_rate",
            Better::Lower,
            Some(&m.false_confirmation_rate),
        ),
        (
            "non_leak_precision",
            Better::Higher,
            Some(&m.non_leak_precision),
        ),
        ("trap_miss_rate", Better::Lower, Some(&m.trap_miss_rate)),
        (
            "trap_false_alarm_rate",
            Better::Lower,
            Some(&m.trap_false_alarm_rate),
        ),
        (
            "source_overclaim_rate",
            Better::Lower,
            m.source_overclaim_rate.as_ref(),
        ),
        (
            "undiscriminated_refutations",
            Better::Lower,
            m.undiscriminated_refutations.as_ref(),
        ),
        (
            "uncited_findings",
            Better::Lower,
            m.uncited_findings.as_ref(),
        ),
        (
            "uncited_checklist_answers",
            Better::Lower,
            m.uncited_checklist_answers.as_ref(),
        ),
        ("dropped_rate", Better::Lower, m.dropped_rate.as_ref()),
    ]
}

/// The baselines "set in M7": they never get looser.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Baselines {
    /// Second-leak recall (a floor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_leak_recall: Option<f64>,
    /// Forced-fit rate (a ceiling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forced_fit_rate: Option<f64>,
    /// Non-leak precision (a floor).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_leak_precision: Option<f64>,
}

/// The metrics of the previous release.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReleaseBlock {
    /// A name for the release.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The ablation the gates judged when the block was written (archetypes on).
    pub ablation: String,
    /// Statistics of every compared metric that has data, by `<ablation>/<archetypes>`
    /// and metric name.
    #[serde(default)]
    pub arms: BTreeMap<String, BTreeMap<String, Stat>>,
    /// Tokens per run on the smoke subset, by `<ablation>/<archetypes>`.
    #[serde(default)]
    pub tokens_per_run: BTreeMap<String, f64>,
}

/// The baseline file.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BaselineFile {
    /// Schema version; 1.
    pub schema_version: u32,
    /// The M7 baselines.
    #[serde(default)]
    pub baselines: Baselines,
    /// The previous release, when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<ReleaseBlock>,
}

fn read_baseline(p: &Path) -> Res<BaselineFile> {
    let text = fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let b: BaselineFile =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))?;
    if b.schema_version != 1 {
        return Err(format!(
            "{}: schema_version {} is not 1",
            p.display(),
            b.schema_version
        ));
    }
    Ok(b)
}

/// Outcome of one gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// The rule holds.
    Pass,
    /// The rule does not hold, or there is no data to show it.
    Fail,
}

/// One gate and what was read.
#[derive(Debug, Clone, Serialize)]
pub struct GateResult {
    /// Position in the plan's table, 1 to 10.
    pub n: u8,
    /// Short name.
    pub name: &'static str,
    /// The rule as read.
    pub rule: &'static str,
    /// Pass or fail.
    pub status: Status,
    /// Numbers and the reasons for a failure.
    pub detail: Vec<String>,
}

impl GateResult {
    fn new(n: u8, name: &'static str, rule: &'static str) -> Self {
        Self {
            n,
            name,
            rule,
            status: Status::Pass,
            detail: Vec::new(),
        }
    }

    fn fail(&mut self, why: impl Into<String>) {
        self.status = Status::Fail;
        self.detail.push(why.into());
    }

    fn note(&mut self, text: impl Into<String>) {
        self.detail.push(text.into());
    }
}

/// Everything the gates read besides the metrics.
pub struct Inputs<'a> {
    /// The graded metrics.
    pub metrics: &'a MetricsFile,
    /// The baseline in effect, when there is one.
    pub baseline: Option<&'a BaselineFile>,
    /// The ablation judged (archetypes on).
    pub ablation: &'a str,
    /// The ablation of the recall gate (it needs an archetypes-off arm).
    pub recall_ablation: &'a str,
    /// True when the baselines are being written from these metrics: a baseline value the
    /// file lacks is then set by this run and the gate passes with a note.
    pub writing: bool,
    /// Fixtures of the smoke subset.
    pub smoke: &'a [String],
    /// Lines of the core skill file; `None` when it could not be read.
    pub skill_lines: Option<usize>,
    /// Shipped `L-*` rows; `None` when no list was given.
    pub shipped_rows: Option<&'a [String]>,
}

fn group<'a>(m: &'a MetricsFile, ablation: &str, archetypes: &str) -> Option<&'a Group> {
    m.groups
        .iter()
        .find(|g| g.ablation == ablation && g.archetypes == archetypes)
}

fn fmt(x: f64) -> String {
    format!("{x:.4}")
}

/// A zero-count gate: `ratio.num == 0` with at least one run in the denominator.
fn zero_count(res: &mut GateResult, label: &str, r: Option<&Ratio>, need_den: bool) {
    match r {
        None => res.fail(format!(
            "{label}: the metrics do not have it; grade again with the current dla-bench"
        )),
        Some(r) if r.num > 0 => res.fail(format!("{label}: {}/{} (must be 0)", r.num, r.den)),
        Some(r) if r.den == 0 && need_den => {
            res.fail(format!("{label}: 0/0, nothing was measured"));
        }
        Some(r) => res.note(format!("{label}: {}/{}", r.num, r.den)),
    }
}

/// What the smoke subset says about the tokens of an arm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmokeTokens {
    /// None of the smoke fixtures has a run in the arm.
    NoRuns,
    /// The smoke fixtures have runs, but the metrics record no tokens (graded before the
    /// metrics carried `tokens_total`, or the runs have no usage block).
    NoTokens,
    /// Tokens per run: the summed `tokens_total` over the smoke fixtures that have runs, over
    /// their runs.
    PerRun(f64),
}

/// The tokens of the smoke subset in one arm.
pub fn smoke_tokens(g: &Group, smoke: &[String]) -> SmokeTokens {
    let (mut tokens, mut runs) = (0u64, 0u64);
    for id in smoke {
        if let Some(f) = g.fixtures.get(id) {
            tokens += f.tokens_total;
            runs += f.runs;
        }
    }
    if runs == 0 {
        SmokeTokens::NoRuns
    } else if tokens == 0 {
        SmokeTokens::NoTokens
    } else {
        SmokeTokens::PerRun(tokens as f64 / runs as f64)
    }
}

/// Tokens per run on the smoke subset; `None` unless the arm has runs with token counts.
pub fn smoke_tokens_per_run(g: &Group, smoke: &[String]) -> Option<f64> {
    match smoke_tokens(g, smoke) {
        SmokeTokens::PerRun(t) => Some(t),
        _ => None,
    }
}

/// Evaluates the ten gates.
pub fn evaluate(i: &Inputs) -> Vec<GateResult> {
    let on = group(i.metrics, i.ablation, "on");
    let recall_on = group(i.metrics, i.recall_ablation, "on");
    let recall_off = group(i.metrics, i.recall_ablation, "off");
    let mut out = Vec::new();
    let no_group = |res: &mut GateResult| {
        res.fail(format!("no group {}/on in the metrics", i.ablation));
    };

    // 1. No overclaiming.
    let mut g1 = GateResult::new(
        1,
        "no overclaiming",
        "zero confirmed or strong findings on impossible fixtures",
    );
    match on {
        Some(g) => zero_count(
            &mut g1,
            "overclaim_rate",
            Some(&g.metrics.overclaim_rate),
            true,
        ),
        None => no_group(&mut g1),
    }
    out.push(g1);

    // 2. Archetypes never cost recall.
    let mut g2 = GateResult::new(
        2,
        "archetypes never cost recall",
        "leak recall and archetype-free recall with archetypes on are at least those with archetypes off (the recall ablation, gates), mean of at least 3 runs",
    );
    match (recall_on, recall_off) {
        (Some(on), Some(off)) => {
            for (name, a, b) in [
                (
                    "leak_recall",
                    &on.metrics.leak_recall,
                    &off.metrics.leak_recall,
                ),
                (
                    "archetype_free_recall",
                    &on.metrics.archetype_free_recall,
                    &off.metrics.archetype_free_recall,
                ),
            ] {
                let (n_on, n_off) = (base_run_count(a, BASE_RUNS), base_run_count(b, BASE_RUNS));
                if n_on < BASE_RUNS as usize || n_off < BASE_RUNS as usize {
                    g2.fail(format!(
                        "{name}: {n_on} runs on and {n_off} runs off have data; {BASE_RUNS} are needed"
                    ));
                    continue;
                }
                let (Some(sa), Some(sb)) = (stat(a, BASE_RUNS), stat(b, BASE_RUNS)) else {
                    g2.fail(format!("{name}: no data"));
                    continue;
                };
                if sa.mean + EPS < sb.mean {
                    g2.fail(format!(
                        "{name}: on {} < off {}",
                        fmt(sa.mean),
                        fmt(sb.mean)
                    ));
                } else {
                    g2.note(format!(
                        "{name}: on {} >= off {}",
                        fmt(sa.mean),
                        fmt(sb.mean)
                    ));
                }
            }
        }
        (None, _) => g2.fail(format!("no group {}/on in the metrics", i.recall_ablation)),
        (_, None) => g2.fail(format!(
            "no group {}/off in the metrics: the comparison needs the archetypes-off arm",
            i.recall_ablation
        )),
    }
    out.push(g2);

    // 3. Second leaks are found.
    let mut g3 = GateResult::new(
        3,
        "second leaks are found",
        "second-leak recall (mean of the base runs) at or above the M7 baseline",
    );
    floor_gate(
        &mut g3,
        "second_leak_recall",
        on.map(|g| &g.metrics.second_leak_recall),
        i.baseline.and_then(|b| b.baselines.second_leak_recall),
        i.writing,
    );
    out.push(g3);

    // 4. No forced fits.
    let mut g4 = GateResult::new(
        4,
        "no forced fits",
        "forced-fit rate (mean of the base runs) at or below the M7 baseline",
    );
    match (
        on.map(|g| &g.metrics.forced_fit_rate),
        i.baseline.and_then(|b| b.baselines.forced_fit_rate),
    ) {
        (None, _) => no_group(&mut g4),
        (Some(r), base) => match (stat(r, BASE_RUNS), base) {
            (None, _) => g4.fail("forced_fit_rate: no data"),
            (Some(s), None) if i.writing => g4.note(format!(
                "forced_fit_rate {}: no stored baseline, this run sets it",
                fmt(s.mean)
            )),
            (_, None) => {
                g4.fail("no forced_fit_rate baseline: run the baseline run with --write-baseline")
            }
            (Some(s), Some(b)) if s.mean > b + EPS => {
                g4.fail(format!(
                    "forced_fit_rate {} > baseline {}",
                    fmt(s.mean),
                    fmt(b)
                ));
            }
            (Some(s), Some(b)) => {
                g4.note(format!(
                    "forced_fit_rate {} <= baseline {}",
                    fmt(s.mean),
                    fmt(b)
                ));
            }
        },
    }
    out.push(g4);

    // 5. Decoys hold.
    let mut g5 = GateResult::new(
        5,
        "decoys hold",
        "every shipped archetype row: its leak found in every run, no claim on its decoy",
    );
    match on {
        None => no_group(&mut g5),
        Some(g) if g.decoys.is_empty() && g.metrics.uncited_findings.is_none() => {
            // Metrics graded before M7 have no decoys block; the pairs may well have run.
            g5.fail(
                "the metrics do not have the decoys block; grade again with the current dla-bench",
            );
        }
        Some(g) => {
            if g.decoys.is_empty() {
                g5.fail("no archetype row has a decoy and control pair among the graded fixtures");
            }
            for (row, d) in &g.decoys {
                if d.pass {
                    g5.note(format!(
                        "{row}: pass (leak {}/{}, decoy claims {}/{})",
                        d.leak.found_runs, d.leak.runs, d.decoy.claim_runs, d.decoy.runs
                    ));
                } else {
                    g5.fail(format!(
                        "{row}: leak found in {}/{} runs, decoy claimed in {}/{} runs ({})",
                        d.leak.found_runs,
                        d.leak.runs,
                        d.decoy.claim_runs,
                        d.decoy.runs,
                        d.pairs.join(", ")
                    ));
                }
            }
            if let Some(rows) = i.shipped_rows {
                for row in rows {
                    if !g.decoys.contains_key(row) {
                        g5.fail(format!("{row}: shipped, but no pair was graded"));
                    }
                }
            }
        }
    }
    out.push(g5);

    // 6. Non-leaks are ruled out with evidence.
    let mut g6 = GateResult::new(
        6,
        "non-leaks are ruled out with evidence",
        "non-leak precision (mean of the base runs) at or above the M7 baseline; every refutation has a discriminator",
    );
    floor_gate(
        &mut g6,
        "non_leak_precision",
        on.map(|g| &g.metrics.non_leak_precision),
        i.baseline.and_then(|b| b.baselines.non_leak_precision),
        i.writing,
    );
    if let Some(g) = on {
        zero_count(
            &mut g6,
            "undiscriminated_refutations",
            g.metrics.undiscriminated_refutations.as_ref(),
            false,
        );
    }
    out.push(g6);

    // 7. Every claim is cited.
    let mut g7 = GateResult::new(
        7,
        "every claim is cited",
        "zero findings and zero checklist answers without evidence ids",
    );
    match on {
        Some(g) => {
            zero_count(
                &mut g7,
                "uncited_findings",
                g.metrics.uncited_findings.as_ref(),
                true,
            );
            zero_count(
                &mut g7,
                "uncited_checklist_answers",
                g.metrics.uncited_checklist_answers.as_ref(),
                false,
            );
        }
        None => no_group(&mut g7),
    }
    out.push(g7);

    // 8. Nothing is dropped.
    let mut g8 = GateResult::new(
        8,
        "nothing is dropped",
        "no run with a COMPLETE report holds an undispositioned candidate or an unresolved residual",
    );
    match on {
        Some(g) => zero_count(
            &mut g8,
            "dropped_rate",
            g.metrics.dropped_rate.as_ref(),
            true,
        ),
        None => no_group(&mut g8),
    }
    out.push(g8);

    // 9. Context cost.
    let mut g9 = GateResult::new(
        9,
        "context cost",
        "tokens per run on the smoke subset at most 110% of the previous release; core SKILL.md under 200 lines",
    );
    match i.skill_lines {
        Some(n) if n < SKILL_MAX_LINES => g9.note(format!("SKILL.md: {n} lines")),
        Some(n) => g9.fail(format!(
            "SKILL.md: {n} lines (must be under {SKILL_MAX_LINES})"
        )),
        None => g9.fail("SKILL.md could not be read"),
    }
    match on.map(|g| smoke_tokens(g, i.smoke)) {
        Some(SmokeTokens::PerRun(_)) => {}
        Some(SmokeTokens::NoTokens) => g9.fail(format!(
            "tokens_total: the metrics do not have it for the smoke fixtures in {}/on; grade again with the current dla-bench",
            i.ablation
        )),
        Some(SmokeTokens::NoRuns) | None => g9.fail(format!(
            "no smoke fixture ({}) has runs in {}/on",
            i.smoke.join(", "),
            i.ablation
        )),
    }
    // Every arm is reported; only the judged arm can fail the gate.
    for g in &i.metrics.groups {
        let key = format!("{}/{}", g.ablation, g.archetypes);
        let judged = g.ablation == i.ablation && g.archetypes == "on";
        let SmokeTokens::PerRun(now) = smoke_tokens(g, i.smoke) else {
            if !judged {
                g9.note(format!("{key}: no token counts for the smoke subset"));
            }
            continue;
        };
        let prev = i
            .baseline
            .and_then(|b| b.release.as_ref())
            .and_then(|r| r.tokens_per_run.get(&key));
        let over = |p: f64| now > p * (1.0 + CONTEXT_MARGIN) + EPS;
        let tag = if judged {
            ""
        } else {
            " (reported, not judged)"
        };
        match prev {
            None => g9.note(format!(
                "{key}: {} tokens per run (no previous release: this is the baseline){tag}",
                now.round()
            )),
            Some(&p) => {
                let line = format!(
                    "{key}: {} tokens per run {} {} (previous {} + 10%){tag}",
                    now.round(),
                    if over(p) { ">" } else { "<=" },
                    (p * (1.0 + CONTEXT_MARGIN)).round(),
                    p.round()
                );
                if over(p) && judged {
                    g9.fail(line);
                } else {
                    g9.note(line);
                }
            }
        }
    }
    out.push(g9);

    // 10. Regression.
    let mut g10 = GateResult::new(
        10,
        "regression",
        "in every arm present in both files, no metric worse than the previous release by more than max(2 x the larger stdev, one run's share in the previous release)",
    );
    match (on, i.baseline.and_then(|b| b.release.as_ref())) {
        (None, _) => no_group(&mut g10),
        (_, None) => g10.note("no previous release: nothing to compare"),
        (Some(_), Some(prev)) => {
            let mut compared = 0;
            for g in &i.metrics.groups {
                let key = format!("{}/{}", g.ablation, g.archetypes);
                let Some(old_arm) = prev.arms.get(&key) else {
                    continue;
                };
                compared += 1;
                for (name, better, ratio) in compared_metrics(g) {
                    let (Some(old), Some(now)) =
                        (old_arm.get(name), ratio.and_then(|r| stat(r, BASE_RUNS)))
                    else {
                        continue;
                    };
                    // One run's share of the metric in the previous release.
                    let share = 1.0 / old.den_run.max(1.0);
                    let tol = (2.0 * now.stdev.max(old.stdev)).max(share);
                    let worse = match better {
                        Better::Higher => old.mean - now.mean,
                        Better::Lower => now.mean - old.mean,
                    };
                    if worse > tol + EPS {
                        g10.fail(format!(
                            "{key} {name}: {} vs previous {} (worse by {}, allowed {})",
                            fmt(now.mean),
                            fmt(old.mean),
                            fmt(worse),
                            fmt(tol)
                        ));
                    }
                }
            }
            if compared == 0 {
                g10.fail("the previous release has none of the arms of these metrics");
            } else if g10.status == Status::Pass {
                g10.note(format!(
                    "{compared} arm(s) compared with {}",
                    prev.label.as_deref().unwrap_or("the previous release")
                ));
            }
        }
    }
    out.push(g10);
    out
}

/// A floor gate: the mean of the base runs must be at or above `base`.
fn floor_gate(
    res: &mut GateResult,
    name: &str,
    r: Option<&Ratio>,
    base: Option<f64>,
    writing: bool,
) {
    match (r, base) {
        (None, _) => res.fail("no group in the metrics"),
        (Some(r), base) => match (stat(r, BASE_RUNS), base) {
            (None, _) => res.fail(format!("{name}: no data")),
            (Some(s), None) if writing => res.note(format!(
                "{name} {}: no stored baseline, this run sets it",
                fmt(s.mean)
            )),
            (_, None) => res.fail(format!(
                "no {name} baseline: run the baseline run with --write-baseline"
            )),
            (Some(s), Some(b)) if s.mean + EPS < b => {
                res.fail(format!("{name} {} < baseline {}", fmt(s.mean), fmt(b)));
            }
            (Some(s), Some(b)) => {
                res.note(format!("{name} {} >= baseline {}", fmt(s.mean), fmt(b)))
            }
        },
    }
}

/// The baseline file these metrics give, merged with `prev` so no baseline gets looser.
pub fn make_baseline(
    m: &MetricsFile,
    ablation: &str,
    smoke: &[String],
    prev: Option<&BaselineFile>,
    baselines_only: bool,
    label: Option<&str>,
) -> Res<BaselineFile> {
    let g =
        group(m, ablation, "on").ok_or_else(|| format!("no group {ablation}/on in the metrics"))?;
    let old = prev.cloned().unwrap_or_default();
    let tighten = |old: Option<f64>, new: Option<f64>, higher: bool| match (old, new) {
        (Some(o), Some(n)) => Some(if higher { o.max(n) } else { o.min(n) }),
        (o, n) => o.or(n),
    };
    let mean = |r: &Ratio| stat(r, BASE_RUNS).map(|s| s.mean);
    let baselines = Baselines {
        second_leak_recall: tighten(
            old.baselines.second_leak_recall,
            mean(&g.metrics.second_leak_recall),
            true,
        ),
        forced_fit_rate: tighten(
            old.baselines.forced_fit_rate,
            mean(&g.metrics.forced_fit_rate),
            false,
        ),
        non_leak_precision: tighten(
            old.baselines.non_leak_precision,
            mean(&g.metrics.non_leak_precision),
            true,
        ),
    };
    let release = if baselines_only {
        old.release
    } else {
        match smoke_tokens(g, smoke) {
            SmokeTokens::PerRun(_) => {}
            SmokeTokens::NoTokens => {
                return Err(format!(
                    "the metrics have no token counts for the smoke fixtures in {ablation}/on: grade again with the current dla-bench"
                ));
            }
            SmokeTokens::NoRuns => {
                return Err(format!(
                    "no smoke fixture has runs in {ablation}/on: the context cost baseline would be empty"
                ));
            }
        }
        let arms = m
            .groups
            .iter()
            .map(|g| {
                let stats = compared_metrics(g)
                    .into_iter()
                    .filter_map(|(n, _, r)| Some((n.to_owned(), stat(r?, BASE_RUNS)?)))
                    .collect();
                (format!("{}/{}", g.ablation, g.archetypes), stats)
            })
            .collect();
        let tokens_per_run = m
            .groups
            .iter()
            .filter_map(|g| {
                Some((
                    format!("{}/{}", g.ablation, g.archetypes),
                    smoke_tokens_per_run(g, smoke)?,
                ))
            })
            .collect();
        Some(ReleaseBlock {
            label: label.map(str::to_owned),
            ablation: ablation.to_owned(),
            arms,
            tokens_per_run,
        })
    };
    Ok(BaselineFile {
        schema_version: 1,
        baselines,
        release,
    })
}

/// The baselines in effect when two files are given: per value the stricter of the two. The
/// previous-release block is the first file's, else the second's.
pub fn merge_baselines(a: Option<&BaselineFile>, b: Option<&BaselineFile>) -> Option<BaselineFile> {
    let (x, y) = match (a, b) {
        (None, None) => return None,
        (Some(x), None) | (None, Some(x)) => return Some(x.clone()),
        (Some(x), Some(y)) => (x, y),
    };
    let pick = |p: Option<f64>, q: Option<f64>, higher: bool| match (p, q) {
        (Some(p), Some(q)) => Some(if higher { p.max(q) } else { p.min(q) }),
        (p, q) => p.or(q),
    };
    Some(BaselineFile {
        schema_version: 1,
        baselines: Baselines {
            second_leak_recall: pick(
                x.baselines.second_leak_recall,
                y.baselines.second_leak_recall,
                true,
            ),
            forced_fit_rate: pick(
                x.baselines.forced_fit_rate,
                y.baselines.forced_fit_rate,
                false,
            ),
            non_leak_precision: pick(
                x.baselines.non_leak_precision,
                y.baselines.non_leak_precision,
                true,
            ),
        },
        release: x.release.clone().or_else(|| y.release.clone()),
    })
}

/// The `L-*` tables of the archetype data file.
pub fn shipped_rows(path: &Path) -> Res<Vec<String>> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let t: toml::Table = text
        .parse()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rows: Vec<String> = t.keys().filter(|k| k.starts_with("L-")).cloned().collect();
    rows.sort();
    Ok(rows)
}

/// Renders the results as a table.
pub fn render(results: &[GateResult]) -> String {
    let mut s = String::new();
    for r in results {
        let tag = match r.status {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
        };
        s.push_str(&format!("{:>2}  {tag}  {}\n", r.n, r.name));
        s.push_str(&format!("            rule: {}\n", r.rule));
        for d in &r.detail {
            s.push_str(&format!("            {d}\n"));
        }
    }
    let failed = results.iter().filter(|r| r.status == Status::Fail).count();
    s.push_str(&format!(
        "\n{} of {} gates pass\n",
        results.len() - failed,
        results.len()
    ));
    s
}

/// Runs the subcommand: the text to print and whether every gate passed.
pub fn run(a: &GatesArgs) -> Res<(String, bool)> {
    let text =
        fs::read_to_string(&a.metrics).map_err(|e| format!("{}: {e}", a.metrics.display()))?;
    let m: MetricsFile =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", a.metrics.display()))?;
    let given = a.baseline.as_deref().map(read_baseline).transpose()?;
    let existing = match &a.write_baseline {
        Some(p) if p.exists() => Some(read_baseline(p)?),
        _ => None,
    };
    // The metrics are judged against what is stored (the file and `--baseline`, the stricter
    // value of each), never against the values they are about to write.
    let in_effect = merge_baselines(given.as_ref(), existing.as_ref());
    let mut written = match &a.write_baseline {
        Some(_) => Some(make_baseline(
            &m,
            &a.ablation,
            &a.smoke,
            in_effect.as_ref(),
            a.baselines_only,
            a.label.as_deref(),
        )?),
        None => None,
    };
    let rows = if a.rows.exists() {
        Some(shipped_rows(&a.rows)?)
    } else {
        None
    };
    let skill_lines = fs::read_to_string(&a.skill).ok().map(|t| t.lines().count());
    let results = evaluate(&Inputs {
        metrics: &m,
        baseline: in_effect.as_ref(),
        ablation: &a.ablation,
        recall_ablation: &a.recall_ablation,
        writing: a.write_baseline.is_some(),
        smoke: &a.smoke,
        skill_lines,
        shipped_rows: rows.as_deref(),
    });
    let ok = results.iter().all(|r| r.status == Status::Pass);
    // `--baselines-only` keeps the previous-release block of the file as it is, not the one
    // of `--baseline`.
    if let Some(b) = &mut written
        && a.baselines_only
    {
        b.release = existing.as_ref().and_then(|e| e.release.clone());
    }
    // A failing gate refuses the write, except for the M7 baseline run (`--baselines-only`),
    // which writes only the three baselines.
    let refused: Vec<String> = results
        .iter()
        .filter(|r| r.status == Status::Fail)
        .map(|r| format!("{} {}", r.n, r.name))
        .collect();
    let refuse = !ok && !a.baselines_only;
    if let (Some(p), Some(b), false) = (&a.write_baseline, &written, refuse) {
        let mut body = serde_json::to_string_pretty(b).map_err(|e| e.to_string())?;
        body.push('\n');
        fs::write(p, body).map_err(|e| format!("{}: {e}", p.display()))?;
    }
    let mut out = if a.json {
        serde_json::to_string_pretty(&serde_json::json!({ "pass": ok, "gates": results }))
            .map_err(|e| e.to_string())?
    } else {
        render(&results)
    };
    if let Some(p) = &a.write_baseline {
        if refuse {
            out.push_str(&format!(
                "\nbaseline not written to {}: gate(s) failed ({}); fix them, or use --baselines-only on the M7 baseline run to write only the three baselines\n",
                p.display(),
                refused.join(", ")
            ));
        } else {
            out.push_str(&format!("\nbaseline written to {}\n", p.display()));
        }
    }
    Ok((out, ok))
}
