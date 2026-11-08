//! `dla harness gen` and `dla harness run`: loop harnesses (spec 4.2 and 4.3).

use std::path::{Path, PathBuf};

use clap::Subcommand;
use dla_core::{FindingId, runtime::Runtime, series::KeyMode};
use dla_workload::harness::{
    r#gen::{GenOptions, generate},
    run::{RunOptions, run as run_harness},
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
    stage,
};

/// `dla harness ...`
#[derive(Subcommand, Debug)]
pub enum HarnessCmd {
    /// Write a loop harness for a workload into the run directory (runs nothing).
    Gen(GenArgs),
    /// Build and run a harness, writing one series per key mode.
    Run(RunArgs),
}

/// Whether probes are derived from the census.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbesChoice {
    /// From census candidates.
    Auto,
    /// Only the `--probe` findings.
    None,
}

impl std::str::FromStr for ProbesChoice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(Self::Auto),
            "none" => Ok(Self::None),
            other => Err(format!("{other:?} is not auto or none")),
        }
    }
}

/// Flags of `dla harness gen`.
#[derive(clap::Args, Debug)]
pub struct GenArgs {
    /// Runtime: jvm | go | rust | node | python | native.
    #[arg(long, value_parser = parse_enum::<Runtime>)]
    lang: Runtime,
    /// Workload ID from `dla workload discover` (default: the best-ranked main or script
    /// workload of the runtime that takes an iteration count).
    #[arg(long, value_name = "ID")]
    workload: Option<String>,
    /// Driver mode: write a loop test with hooks to fill in, run in one process.
    #[arg(long)]
    driver: bool,
    /// How the workload takes an iteration count, with `{n}`: "--iterations {n}", "-n {n}",
    /// "{n}" (overrides detection).
    #[arg(long, value_name = "TEMPLATE", allow_hyphen_values = true)]
    iter_arg: Option<String>,
    /// Units of work added per point (default: the largest count is about four times the
    /// workload's default).
    #[arg(long, value_name = "UNITS")]
    step: Option<u64>,
    /// Measured points after the warm-up.
    #[arg(long, default_value_t = 8)]
    iterations: u32,
    /// Warm-up points.
    #[arg(long, default_value_t = 1)]
    warmup: u32,
    /// Add a probe for the holder of this finding (repeatable).
    #[arg(long = "probe", value_name = "F", value_parser = parse_enum::<FindingId>)]
    probe: Vec<FindingId>,
    /// Probes derived from the census candidates: auto (default) or none.
    #[arg(long, default_value = "auto", value_parser = parse_enum::<ProbesChoice>)]
    probes: ProbesChoice,
    /// Driver mode: the Go package directory or the Java package to drive.
    #[arg(long, value_name = "DIR|NAME")]
    package: Option<String>,
}

/// Which key modes `dla harness run` measures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeChoice(Vec<KeyMode>);

impl std::str::FromStr for ModeChoice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "fresh" => Ok(Self(vec![KeyMode::Fresh])),
            "repeated" => Ok(Self(vec![KeyMode::Repeated])),
            "both" => Ok(Self(vec![KeyMode::Fresh, KeyMode::Repeated])),
            "native" => Ok(Self(vec![KeyMode::Native])),
            other => Err(format!("{other:?} is not fresh, repeated, both or native")),
        }
    }
}

/// Flags of `dla harness run`.
#[derive(clap::Args, Debug)]
pub struct RunArgs {
    /// Harness ID from `dla harness gen` (`H1`).
    #[arg(value_name = "H")]
    harness: String,
    /// Key modes: fresh | repeated | both (driver mode) or native (command mode); default:
    /// the manifest's.
    #[arg(long, value_parser = parse_enum::<ModeChoice>)]
    mode: Option<ModeChoice>,
    /// Measured points after the warm-up, overriding the manifest.
    #[arg(long, value_name = "N")]
    iterations: Option<u32>,
    /// Extra points after the measured ones (the plateau window of `dla alloc-test`).
    #[arg(long, value_name = "K", default_value_t = 0)]
    extra: u32,
    /// Purge applied at the last point: trim | gc | free-os-memory | trim-native-heap |
    /// jemalloc | empty-cache | none.
    #[arg(long, value_name = "KIND")]
    purge: Option<String>,
}

/// Runs `dla harness`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: HarnessCmd) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    match a {
        HarnessCmd::Gen(g) => {
            let lang = g.lang;
            let opts = GenOptions {
                lang,
                workload: g.workload,
                driver: g.driver,
                iter_arg: g.iter_arg,
                step: g.step,
                iterations: g.iterations,
                warmup: g.warmup,
                probe_findings: g.probe,
                auto_probes: g.probes == ProbesChoice::Auto,
                package: g.package,
            };
            stage::finish(
                "harness gen",
                json!({"lang": lang.as_str()}),
                generate(&tc, &opts),
            )
        }
        HarnessCmd::Run(r) => {
            if !r.harness.starts_with('H') {
                return Err(Failure::usage(
                    format!("{:?} is not a harness ID", r.harness),
                    "a harness ID such as H1",
                    "dla harness run H1",
                ));
            }
            let opts = RunOptions {
                modes: r.mode.map(|m| m.0).unwrap_or_default(),
                iterations: r.iterations,
                extra: r.extra,
                purge: r.purge,
                ..RunOptions::default()
            };
            let effects = json!({
                "target": format!("the workload of harness {} in its scratch copy under the run directory", r.harness),
                "expected_signal": "one series per key mode: memory, threads, handles and probe sizes per point",
                "stopping": "after the last point, or a point's timeout (the process group is killed)",
                "effects": "builds and runs project code in the scratch copy with DLA's agent loaded; the project source is not edited",
                "cleanup": "every process is registered in state.json and stopped; the scratch copy stays for inspection",
            });
            stage::finish(
                "harness run",
                json!({"harness": r.harness, "effects": effects}),
                run_harness(&tc, &r.harness, &opts),
            )
        }
    }
}
