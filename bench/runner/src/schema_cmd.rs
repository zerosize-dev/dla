//! `dla-bench schema`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use schemars::{JsonSchema, schema_for};

use crate::{
    Res,
    model::{Expected, FindingsFile, MetricsFile, RunJson},
};

/// Flags of `dla-bench schema`.
#[derive(Args, Debug)]
pub struct SchemaArgs {
    /// Directory to write `<name>.schema.json` files into.
    #[arg(long, default_value = "bench/schemas")]
    pub out: PathBuf,
    /// Write nothing; fail if the files in --out differ from the generated schemas.
    #[arg(long)]
    pub check: bool,
}

fn render<T: JsonSchema>() -> String {
    let mut text = serde_json::to_string_pretty(&schema_for!(T)).expect("schemas serialise");
    text.push('\n');
    text
}

/// All generated schemas as `(file name, text)`.
pub fn all() -> Vec<(String, String)> {
    vec![
        ("expected.schema.json".into(), render::<Expected>()),
        ("findings.schema.json".into(), render::<FindingsFile>()),
        ("run.schema.json".into(), render::<RunJson>()),
        ("metrics.schema.json".into(), render::<MetricsFile>()),
    ]
}

/// The generated `expected.toml` schema.
pub fn expected_schema() -> String {
    render::<Expected>()
}

/// Runs the subcommand; the message to print on success.
pub fn run(a: &SchemaArgs) -> Res<String> {
    let files = all();
    if a.check {
        return check(&a.out, &files);
    }
    fs::create_dir_all(&a.out).map_err(|e| format!("{}: {e}", a.out.display()))?;
    for (name, text) in &files {
        let p = a.out.join(name);
        if fs::read_to_string(&p).ok().as_deref() != Some(text.as_str()) {
            fs::write(&p, text).map_err(|e| format!("{}: {e}", p.display()))?;
        }
    }
    Ok(format!(
        "wrote {} schemas to {}",
        files.len(),
        a.out.display()
    ))
}

fn check(dir: &Path, files: &[(String, String)]) -> Res<String> {
    let bad: Vec<&str> = files
        .iter()
        .filter(|(n, text)| {
            // A Windows checkout may have turned LF into CRLF: line endings do not count.
            fs::read_to_string(dir.join(n))
                .ok()
                .map(|t| t.replace("\r\n", "\n"))
                .as_deref()
                != Some(text.as_str())
        })
        .map(|(n, _)| n.as_str())
        .collect();
    if bad.is_empty() {
        Ok(format!(
            "{} schemas in {} are up to date",
            files.len(),
            dir.display()
        ))
    } else {
        Err(format!(
            "schema files missing or out of date in {}: {}; fix: dla-bench schema --out {}",
            dir.display(),
            bad.join(", "),
            dir.display()
        ))
    }
}
