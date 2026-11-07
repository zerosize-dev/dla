//! The `dla` command line.
//!
//! Every subcommand prints one JSON object of at most 2048 bytes on stdout, except
//! `dla report`, which prints the rendered markdown. Exit codes: 0 ok, 1 error
//! (I/O, parse, usage), 2 refused by a rule, 3 schema version mismatch.

mod budget;
mod ctx;
mod fail;
mod git;
mod gitignore;
mod ledger_cmd;
mod out;
mod parse;
mod report;
mod run;
mod schema_cmd;

use std::{io::Write, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand, error::ErrorKind};
use serde_json::{Value, json};

use crate::{
    ctx::{Ctx, project_root},
    fail::{Failure, Res},
};

/// DLA: leak investigation ledger and report tooling.
#[derive(Parser, Debug)]
#[command(name = "dla", version, about, propagate_version = true)]
struct Cli {
    /// Project directory (default: the git toplevel above the cwd, else the cwd).
    #[arg(long, global = true, value_name = "DIR")]
    project: Option<PathBuf>,
    /// Run ID (default: $DLA_RUN, else .dla/current).
    #[arg(long, global = true, value_name = "ID")]
    run: Option<String>,
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
    /// Write or check the JSON Schema files.
    Schema(schema_cmd::SchemaArgs),
}

/// What a command prints on success.
enum Output {
    Json(Value),
    Markdown(String),
}

fn dispatch(cli: Cli) -> Res<Output> {
    let project = cli.project.as_deref();
    let run_arg = cli.run.as_deref();
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
        Command::Report => {
            let r = report::run(&ctx()?)?;
            eprintln!("report: {} sha256={}", r.path, r.sha256);
            Ok(Output::Markdown(r.markdown))
        }
        Command::Schema(a) => schema_cmd::run(a).map(Output::Json),
    }
}

fn print_json(v: Value) {
    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{}", out::bound(v));
}

fn usage_failure(e: &clap::Error) -> Failure {
    let text = e.to_string();
    let message = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("invalid command line")
        .trim_start_matches("error: ")
        .to_owned();
    Failure::usage(message, "valid arguments", "dla --help")
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
