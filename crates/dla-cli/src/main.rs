//! The `dla` command line.
//!
//! Every subcommand prints one JSON object of at most 2048 bytes on stdout, except
//! `dla report`, which prints the rendered markdown. Exit codes: 0 ok, 1 error
//! (I/O, parse, usage), 2 refused by a rule, 3 schema version mismatch.

mod alloc_cmd;
mod budget;
mod census_cmd;
mod consent_cmd;
mod ctx;
mod fail;
mod gap_cmd;
mod git;
mod gitignore;
mod harness_cmd;
mod install_cmd;
mod ledger_cmd;
mod out;
mod parse;
mod probe_cmd;
mod report;
mod run;
mod sample_cmd;
mod schema_cmd;
mod stage;
mod sweep_cmd;
mod trend_cmd;
mod workload_cmd;

use std::{io::Write, path::PathBuf, process::ExitCode};

use clap::{
    CommandFactory, Parser, Subcommand,
    error::{ContextKind, ContextValue, ErrorKind},
};
use dla_core::FindingState;
use serde_json::{Value, json};

use crate::{
    ctx::{Ctx, project_root},
    fail::{Failure, Res},
};

/// DLA: leak investigation ledger and report tooling.
#[derive(Parser, Debug)]
#[command(
    name = "dla",
    bin_name = "dla",
    version,
    about,
    propagate_version = true
)]
struct Cli {
    /// Project directory (default: the git toplevel above the cwd, else the cwd).
    #[arg(long, global = true, value_name = "DIR")]
    project: Option<PathBuf>,
    /// Run ID (default: $DLA_RUN, else .dla/current).
    #[arg(long, global = true, value_name = "ID")]
    run: Option<String>,
    /// Plugin data files directory (default: $DLA_PLUGIN_ROOT/data, else the data/
    /// directory of the checkout the executable lives in).
    #[arg(long, global = true, value_name = "DIR")]
    data: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create and inspect investigation runs.
    #[command(subcommand)]
    Run(run::RunCmd),
    /// Append to the ledger.
    #[command(subcommand)]
    Ledger(ledger_cmd::LedgerCmd),
    /// Render the report, write report.md and store its hash in state.json.
    Report,
    /// Record the run's one consent: privilege tiers and whether raw values are allowed.
    Consent(consent_cmd::ConsentArgs),
    /// Write or check the JSON Schema files.
    Schema(schema_cmd::SchemaArgs),
    /// Probe the project: runtimes, versions, tools, privileges, test runners, libraries.
    Probe(probe_cmd::ProbeArgs),
    /// Run the static backends of one runtime and add their hits as candidates.
    Sweep(sweep_cmd::SweepArgs),
    /// Enumerate process-lifetime holders of one runtime as candidates.
    Census(census_cmd::CensusArgs),
    /// Discover what a loop can run.
    #[command(subcommand)]
    Workload(workload_cmd::WorkloadCmd),
    /// Install a static analysis backend into the plugin data directory.
    Install(install_cmd::InstallArgs),
    /// Generate and run loop harnesses over a discovered workload.
    #[command(subcommand)]
    Harness(harness_cmd::HarnessCmd),
    /// Sample a running process's memory, threads, handles and runtime counters into a series.
    Sample(sample_cmd::SampleArgs),
    /// Trend tests, slope gate and shape for every column of a series.
    Trend(trend_cmd::TrendArgs),
    /// Split process memory into runtime-accounted and unexplained; with --residual, what confirmed findings leave unexplained.
    Gap(gap_cmd::GapArgs),
    /// Re-run a harness with extra cycles and a trim or purge, and record which hypothesis the result favours.
    AllocTest(alloc_cmd::AllocTestArgs),
}

/// What a command prints on success.
enum Output {
    Json(Value),
    Markdown(String),
}

fn dispatch(cli: Cli) -> Res<Output> {
    let project = cli.project.as_deref();
    let run_arg = cli.run.as_deref();
    let data = cli.data.as_ref();
    let ctx = || Ctx::resolve(project_root(project)?, run_arg);
    match cli.command {
        Command::Run(run::RunCmd::Init(a)) => run::init(project, a).map(Output::Json),
        Command::Run(run::RunCmd::Status) => run::run_status(&ctx()?).map(Output::Json),
        Command::Ledger(ledger_cmd::LedgerCmd::Add(a)) => {
            ledger_cmd::add(&ctx()?, a).map(Output::Json)
        }
        Command::Ledger(ledger_cmd::LedgerCmd::Resolve(a)) => {
            ledger_cmd::resolve(&ctx()?, a).map(Output::Json)
        }
        Command::Ledger(ledger_cmd::LedgerCmd::Annotate(a)) => {
            ledger_cmd::annotate(&ctx()?, a).map(Output::Json)
        }
        Command::Ledger(ledger_cmd::LedgerCmd::Test(a)) => {
            ledger_cmd::move_to(&ctx()?, a, FindingState::UnderTest).map(Output::Json)
        }
        Command::Ledger(ledger_cmd::LedgerCmd::Confirm(a)) => {
            ledger_cmd::move_to(&ctx()?, a, FindingState::Confirmed).map(Output::Json)
        }
        Command::Ledger(ledger_cmd::LedgerCmd::Refute(a)) => {
            ledger_cmd::move_to(&ctx()?, a, FindingState::Refuted).map(Output::Json)
        }
        Command::Report => {
            let r = report::run(&ctx()?)?;
            eprintln!("report: {} sha256={}", r.path, r.sha256);
            Ok(Output::Markdown(r.markdown))
        }
        Command::Consent(a) => {
            let ctx = ctx()?;
            consent_cmd::run(&ctx, &project_root(project)?, a).map(Output::Json)
        }
        Command::Schema(a) => schema_cmd::run(a).map(Output::Json),
        Command::Probe(a) => {
            probe_cmd::run(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
        Command::Sweep(a) => {
            sweep_cmd::run(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
        Command::Census(a) => {
            census_cmd::run(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
        Command::Workload(workload_cmd::WorkloadCmd::Discover(a)) => {
            workload_cmd::discover(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
        Command::Install(a) => {
            install_cmd::run(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
        Command::Harness(c) => {
            harness_cmd::run(&ctx()?, &project_root(project)?, data, c).map(Output::Json)
        }
        Command::Sample(a) => {
            sample_cmd::run(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
        Command::Trend(a) => {
            trend_cmd::run(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
        Command::Gap(a) => {
            gap_cmd::run(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
        Command::AllocTest(a) => {
            alloc_cmd::run(&ctx()?, &project_root(project)?, data, a).map(Output::Json)
        }
    }
}

fn print_json(v: Value) {
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{}", out::bound(v));
}

/// The subcommand path in clap's `Usage:` line, e.g. `dla ledger add`.
fn usage_path(rendered: &str) -> Vec<String> {
    let Some(usage) = rendered
        .lines()
        .find_map(|l| l.trim_start().strip_prefix("Usage:"))
    else {
        return vec!["dla".to_owned()];
    };
    let mut path: Vec<String> = Vec::new();
    for w in usage.split_whitespace() {
        if w.starts_with(['-', '<', '[']) {
            break;
        }
        path.push(w.to_owned());
    }
    if path.is_empty() {
        path.push("dla".to_owned());
    }
    path
}

/// The subcommand path named on the command line, e.g. `dla ledger add`. Flags and
/// their values are skipped; matching stops at the first word that is not a subcommand
/// of the current command.
fn args_path() -> Vec<String> {
    let mut cmd = Cli::command();
    let mut path = vec!["dla".to_owned()];
    for a in std::env::args().skip(1) {
        if a.starts_with('-') {
            continue;
        }
        if let Some(sub) = cmd.find_subcommand(&a).cloned() {
            path.push(a);
            cmd = sub;
        }
    }
    path
}

/// Names of the subcommands under `path` (`["dla", "run"]` gives `init`, `status`).
fn subcommand_names(path: &[String]) -> Vec<String> {
    let mut cmd = Cli::command();
    for name in path.iter().skip(1) {
        let Some(sub) = cmd.find_subcommand(name).cloned() else {
            return Vec::new();
        };
        cmd = sub;
    }
    cmd.get_subcommands()
        .filter(|c| !c.is_hide_set() && c.get_name() != "help")
        .map(|c| c.get_name().to_owned())
        .collect()
}

fn usage_failure(e: &clap::Error) -> Failure {
    let rendered = e.to_string();
    let from_usage = usage_path(&rendered);
    let from_args = args_path();
    let path = if from_args.len() > from_usage.len() {
        from_args
    } else {
        from_usage
    };
    let path_text = path.join(" ");
    let fix = format!("{path_text} --help");
    if matches!(
        e.kind(),
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            | ErrorKind::MissingSubcommand
            | ErrorKind::InvalidSubcommand
    ) {
        let subs = subcommand_names(&path);
        let list = subs.join(", ");
        let what = if e.kind() == ErrorKind::InvalidSubcommand {
            let bad = match e.get(ContextKind::InvalidSubcommand) {
                Some(ContextValue::String(s)) => format!(" (unknown: {s})"),
                _ => String::new(),
            };
            format!("`{path_text}` has no such subcommand{bad}; available: {list}")
        } else {
            format!("`{path_text}` needs a subcommand; available: {list}")
        };
        return Failure::usage(what, format!("a subcommand: {list}"), fix);
    }
    // The whole message, up to the usage block: it names the offending argument.
    let message = rendered
        .lines()
        .take_while(|l| !l.trim_start().starts_with("Usage:"))
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("For more information"))
        .map(|l| l.trim_start_matches("error: "))
        .collect::<Vec<_>>()
        .join(" ");
    let message = if message.is_empty() {
        "invalid command line".to_owned()
    } else {
        message
    };
    let names = match e.get(ContextKind::InvalidArg) {
        Some(ContextValue::Strings(v)) => v.join(", "),
        Some(ContextValue::String(s)) => s.clone(),
        _ => String::new(),
    };
    let missing = if names.is_empty() {
        format!("valid arguments for `{path_text}`")
    } else if e.kind() == ErrorKind::MissingRequiredArgument {
        format!("required argument(s): {names}")
    } else {
        format!("a valid use of: {names}")
    };
    Failure::usage(message, missing, fix)
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) => {
            let _ = e.print();
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            let f = usage_failure(&e);
            print_json(json!({"ok": false, "error": f.refusal}));
            return ExitCode::from(1);
        }
    };
    match dispatch(cli) {
        Ok(Output::Json(v)) => {
            print_json(v);
            ExitCode::SUCCESS
        }
        Ok(Output::Markdown(md)) => {
            let mut stdout = std::io::stdout().lock();
            let _ = stdout.write_all(md.as_bytes());
            if !md.ends_with('\n') {
                let _ = stdout.write_all(b"\n");
            }
            ExitCode::SUCCESS
        }
        Err(f) => {
            print_json(json!({"ok": false, "error": f.refusal}));
            ExitCode::from(u8::try_from(f.exit).unwrap_or(1))
        }
    }
}
