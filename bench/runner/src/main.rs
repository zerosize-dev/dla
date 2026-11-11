//! `dla-bench` command line.

use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use dla_bench::{
    calib, closecalls, compare, dynamic_cmd, gates, grade, run, schema_cmd, static_cmd, table,
    validate,
};

#[derive(Parser)]
#[command(name = "dla-bench", version, about = "DLA benchmark runner and grader")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Write (or check) the JSON Schemas of the benchmark files.
    Schema(schema_cmd::SchemaArgs),
    /// Validate every fixture against its schema and the fixture rules.
    Validate {
        /// Directory with the fixtures.
        #[arg(long, default_value = "bench/fixtures")]
        fixtures: PathBuf,
        /// Directory with the JSON Schemas.
        #[arg(long, default_value = "bench/schemas")]
        schemas: PathBuf,
    },
    /// Run the benchmark.
    Run(Box<run::RunArgs>),
    /// Grade a results directory into metrics.json.
    Grade {
        /// Results directory.
        results: PathBuf,
        /// Directory with the fixtures.
        #[arg(long, default_value = "bench/fixtures")]
        fixtures: PathBuf,
        /// Output file (default: <results>/metrics.json).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Render <results>/metrics.json as <results>/table.md.
    Table {
        /// Results directory.
        results: PathBuf,
    },
    /// Assemble the plugin variant of an ablation into a directory.
    Variant {
        /// Ablation (prose, gates or full).
        #[arg(long, value_enum)]
        ablation: run::Ablation,
        /// Whether archetype material is present.
        #[arg(long, value_enum, default_value = "on")]
        archetypes: run::Archetypes,
        /// Destination directory (created).
        #[arg(long)]
        out: PathBuf,
        /// Repository root holding the plugin sources.
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
        /// Directory with the ablation files.
        #[arg(long, default_value = "bench/ablations")]
        ablations_dir: PathBuf,
    },
    /// Static acceptance: probe, sweep, census and workload discovery on every fixture.
    Static(static_cmd::StaticArgs),
    /// Dynamic acceptance: harness loops, trend, alloc-test and the M3 ledger rules on every fixture.
    Dynamic(dynamic_cmd::DynamicArgs),
    /// List the fixtures whose runs disagree, and the command that adds runs 4 and 5.
    CloseCalls(closecalls::CloseCallArgs),
    /// Evaluate the ten release gates on a graded metrics.json (exit 1 when one fails).
    Gates(gates::GatesArgs),
    /// Extract calibration statistics from the log directory of `dla-bench dynamic`.
    CalibExtract(calib::CalibArgs),
    /// Exit 0 when two JSON files are equal as parsed JSON.
    Compare {
        /// First file.
        a: PathBuf,
        /// Second file.
        b: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Cmd::Gates(a) = &cli.cmd {
        // The table goes to stdout whether or not every gate passes.
        return match gates::run(a) {
            Ok((text, ok)) => {
                println!("{}", text.trim_end());
                if ok {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                }
            }
            Err(msg) => {
                eprintln!("{msg}");
                ExitCode::from(2)
            }
        };
    }
    let result = match cli.cmd {
        Cmd::Schema(a) => schema_cmd::run(&a),
        Cmd::Validate { fixtures, schemas } => validate::validate_dir(&fixtures, &schemas)
            .and_then(|(n, problems)| {
                if problems.is_empty() {
                    Ok(format!("ok: {n} fixture(s) valid"))
                } else {
                    Err(problems.join("\n"))
                }
            }),
        #[cfg(unix)]
        Cmd::Run(a) => run::run(&a),
        #[cfg(not(unix))]
        Cmd::Run(_) => Err(
            "dla-bench run needs Linux or macOS: the child sessions run in the Claude Code sandbox"
                .to_owned(),
        ),
        Cmd::Grade {
            results,
            fixtures,
            out,
        } => {
            let out = out.unwrap_or_else(|| results.join("metrics.json"));
            grade::grade_to_file(&results, &fixtures, &out)
                .map(|m| format!("graded {} group(s) into {}", m.groups.len(), out.display()))
        }
        Cmd::Table { results } => table::write_table(&results).map(|p| format!("wrote {p}")),
        Cmd::Variant {
            ablation,
            archetypes,
            out,
            repo_root,
            ablations_dir,
        } => run::build_variant(
            &run::VariantSpec {
                repo_root,
                ablations_dir,
                ablation,
                archetypes_off: archetypes == run::Archetypes::Off,
            },
            &out,
        )
        .map(|()| format!("wrote {}", out.display())),
        Cmd::Static(a) => static_cmd::run(&a),
        Cmd::Dynamic(a) => dynamic_cmd::run(&a),
        Cmd::CloseCalls(a) => closecalls::run(&a),
        Cmd::CalibExtract(a) => calib::run(&a),
        Cmd::Gates(_) => unreachable!("handled above"),
        Cmd::Compare { a, b } => compare::compare_files(&a, &b).and_then(|d| {
            if d.is_empty() {
                Ok("equal".to_owned())
            } else {
                Err(d.join("\n"))
            }
        }),
    };
    match result {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::from(1)
        }
    }
}
