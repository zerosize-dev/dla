//! `dla-bench close-calls`: the fixtures whose runs disagree on their graded outcome, and the
//! command that adds the two extra runs.
//!
//! A close call is a fixture whose runs of one arm do not all have the same graded outcome
//! (`bench/GRADING.md`, "Agreement and close calls"). It gets runs 4 and 5.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use serde::Serialize;

use crate::{
    Res, grade,
    model::{Group, MetricsFile},
};

/// The default fixtures directory of `dla-bench`.
const DEFAULT_FIXTURES: &str = "bench/fixtures";

/// Runs a close call gets in total.
pub const CLOSE_CALL_RUNS: u64 = 5;

/// Flags of `dla-bench close-calls`.
#[derive(Args, Debug)]
pub struct CloseCallArgs {
    /// A results directory (graded in memory) or a `metrics.json` file.
    pub results: PathBuf,
    /// Only this arm: `<ablation>` (both archetype settings) or `<ablation>/<on|off>`.
    /// Repeatable. Default: every arm of the results.
    #[arg(long)]
    pub arm: Vec<String>,
    /// Directory with the fixtures (used when `results` is a directory).
    #[arg(long, default_value = DEFAULT_FIXTURES)]
    pub fixtures: PathBuf,
    /// Print JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

/// One fixture of one arm whose runs disagree.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CloseCall {
    /// Ablation.
    pub ablation: String,
    /// `on` or `off`.
    pub archetypes: String,
    /// Fixture ID.
    pub fixture: String,
    /// Graded runs so far.
    pub runs: u64,
    /// `(run, outcome)` in run order.
    pub outcomes: Vec<(u32, String)>,
}

impl CloseCall {
    /// True when the fixture still lacks runs 4 and 5.
    pub fn needs_more_runs(&self) -> bool {
        self.runs < CLOSE_CALL_RUNS
    }
}

/// Whether `g` is selected by the `--arm` values (all arms when `arms` is empty).
fn selected(g: &Group, arms: &[String]) -> bool {
    arms.is_empty()
        || arms.iter().any(|a| match a.split_once('/') {
            Some((abl, arch)) => abl == g.ablation && arch == g.archetypes,
            None => *a == g.ablation,
        })
}

/// The close calls of every selected arm, in arm and fixture order.
pub fn find(m: &MetricsFile, arms: &[String]) -> Res<Vec<CloseCall>> {
    let mut out = Vec::new();
    for g in m.groups.iter().filter(|g| selected(g, arms)) {
        for (id, f) in &g.fixtures {
            if f.agreement.runs == 0 && f.runs > 0 {
                return Err(format!(
                    "{} {}: the metrics have no agreement block; grade again with this dla-bench",
                    g.ablation, id
                ));
            }
            if f.agreement.runs >= 2 && !f.agreement.unanimous {
                out.push(CloseCall {
                    ablation: g.ablation.clone(),
                    archetypes: g.archetypes.clone(),
                    fixture: id.clone(),
                    runs: f.agreement.runs,
                    outcomes: f
                        .agreement
                        .outcomes
                        .iter()
                        .map(|o| (o.run, o.outcome.clone()))
                        .collect(),
                });
            }
        }
    }
    Ok(out)
}

/// Flags the printed command repeats from the earlier invocation.
#[derive(Debug, Clone, Copy, Default)]
pub struct CommandOpts<'a> {
    /// The model of the earlier runs (`--model`).
    pub model: Option<&'a str>,
    /// The fixtures directory, when it is not the default (`--fixtures-dir`).
    pub fixtures_dir: Option<&'a Path>,
}

/// The `dla-bench run` command that adds runs 4 and 5 of the close calls of one arm, with
/// the default options. See [`command_with`].
pub fn command(
    ablation: &str,
    archetypes: &str,
    calls: &[&CloseCall],
    out: &Path,
) -> Option<String> {
    command_with(ablation, archetypes, calls, out, CommandOpts::default())
}

/// The `dla-bench run` command that adds runs 4 and 5 of the close calls of one arm. The
/// runner skips the runs that are already finished, so `--runs 5` only starts runs 4 and 5.
/// `None` when no close call of the arm needs more runs.
pub fn command_with(
    ablation: &str,
    archetypes: &str,
    calls: &[&CloseCall],
    out: &Path,
    opts: CommandOpts<'_>,
) -> Option<String> {
    let ids: Vec<&str> = calls
        .iter()
        .filter(|c| c.needs_more_runs())
        .map(|c| c.fixture.as_str())
        .collect();
    if ids.is_empty() {
        return None;
    }
    let mut cmd = format!(
        "dla-bench run --ablation {ablation} --archetypes {archetypes} --fixtures {}",
        ids.join(",")
    );
    if let Some(dir) = opts.fixtures_dir {
        cmd.push_str(&format!(" --fixtures-dir {}", dir.display()));
    }
    if let Some(model) = opts.model {
        cmd.push_str(&format!(" --model {model}"));
    }
    cmd.push_str(&format!(
        " --runs {CLOSE_CALL_RUNS} --out {}",
        out.display()
    ));
    Some(cmd)
}

/// The models named in the `run.json` files of the given fixtures of one arm, sorted. Runs
/// 4 and 5 must use the model of runs 1 to 3: the runner does not check it.
pub fn models_of(out: &Path, ablation: &str, archetypes: &str, fixtures: &[&str]) -> Vec<String> {
    let arm = out
        .join("runs")
        .join(ablation)
        .join(format!("archetypes-{archetypes}"));
    let mut models = std::collections::BTreeSet::new();
    for fx in fixtures {
        let Ok(rd) = fs::read_dir(arm.join(fx)) else {
            continue;
        };
        for run in rd.flatten() {
            let Ok(text) = fs::read_to_string(run.path().join("run.json")) else {
                continue;
            };
            if let Some(m) = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v["model"].as_str().map(str::to_owned))
            {
                models.insert(m);
            }
        }
    }
    models.into_iter().collect()
}

fn load(args: &CloseCallArgs) -> Res<(MetricsFile, PathBuf)> {
    if args.results.is_file() {
        let text = fs::read_to_string(&args.results)
            .map_err(|e| format!("{}: {e}", args.results.display()))?;
        let m =
            serde_json::from_str(&text).map_err(|e| format!("{}: {e}", args.results.display()))?;
        // `<results>/metrics.json`: the results directory is the parent.
        let dir = args
            .results
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        return Ok((m, dir));
    }
    let m = grade::grade_dir(&args.results, &args.fixtures)?;
    Ok((m, args.results.clone()))
}

/// Runs the subcommand; the text to print.
pub fn run(args: &CloseCallArgs) -> Res<String> {
    let (m, out_dir) = load(args)?;
    let calls = find(&m, &args.arm)?;
    let fixtures_dir =
        (args.fixtures != Path::new(DEFAULT_FIXTURES)).then_some(args.fixtures.as_path());
    // The command and the models its runs must use, for one arm.
    let build = |abl: &str, arch: &str, here: &[&CloseCall]| -> (Option<String>, Vec<String>) {
        let ids: Vec<&str> = here.iter().map(|c| c.fixture.as_str()).collect();
        let models = models_of(&out_dir, abl, arch, &ids);
        let model = (models.len() == 1).then(|| models[0].as_str());
        let cmd = command_with(
            abl,
            arch,
            here,
            &out_dir,
            CommandOpts {
                model,
                fixtures_dir,
            },
        );
        (cmd, models)
    };
    let mut arms: Vec<(&str, &str)> = calls
        .iter()
        .map(|c| (c.ablation.as_str(), c.archetypes.as_str()))
        .collect();
    arms.dedup();
    if args.json {
        let items: Vec<serde_json::Value> = arms
            .iter()
            .map(|(abl, arch)| {
                let here: Vec<&CloseCall> = calls
                    .iter()
                    .filter(|c| c.ablation == *abl && c.archetypes == *arch)
                    .collect();
                let (command, models) = build(abl, arch, &here);
                serde_json::json!({
                    "ablation": abl,
                    "archetypes": arch,
                    "close_calls": here,
                    "models": models,
                    "command": command,
                })
            })
            .collect();
        return serde_json::to_string_pretty(&items).map_err(|e| e.to_string());
    }
    let mut s = String::new();
    if calls.is_empty() {
        s.push_str("no close calls: the runs of every fixture agree");
        return Ok(s);
    }
    for (abl, arch) in arms {
        let here: Vec<&CloseCall> = calls
            .iter()
            .filter(|c| c.ablation == abl && c.archetypes == arch)
            .collect();
        s.push_str(&format!(
            "{abl} archetypes-{arch}: {} close call(s)\n",
            here.len()
        ));
        for c in &here {
            let runs: Vec<String> = c
                .outcomes
                .iter()
                .map(|(n, o)| format!("#{n} {o}"))
                .collect();
            let note = if c.needs_more_runs() {
                ""
            } else {
                "  (already has 5 runs)"
            };
            s.push_str(&format!("  {}: {}{note}\n", c.fixture, runs.join(" | ")));
        }
        let (cmd, models) = build(abl, arch, &here);
        if models.len() > 1 {
            s.push_str(&format!(
                "  the earlier runs used several models ({}); add --model <one of them> to the command\n",
                models.join(", ")
            ));
        }
        match cmd {
            Some(cmd) => s.push_str(&format!("  add runs 4 and 5: {cmd}\n")),
            None => s.push_str("  no fixture needs more runs\n"),
        }
    }
    Ok(s.trim_end().to_owned())
}
