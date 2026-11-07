//! `dla schema [--out DIR] [--check]`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{Error, fsutil::write_atomic, schema};
use serde_json::{Value, json};

use crate::fail::{Failure, Res, codes};

/// Flags of `dla schema`.
#[derive(Args, Debug)]
pub struct SchemaArgs {
    /// Directory to write `<name>.schema.json` files into.
    #[arg(long, default_value = "schemas")]
    out: PathBuf,
    /// Write nothing; exit 1 if the files in --out differ from the generated schemas.
    #[arg(long)]
    check: bool,
}

/// Runs `dla schema`.
pub fn run(a: SchemaArgs) -> Res<Value> {
    let files: Vec<(String, String)> = schema::all()
        .into_iter()
        .map(|(name, s)| (format!("{name}.schema.json"), schema::render(&s)))
        .collect();
    let out = a.out.to_string_lossy().replace('\\', "/");
    if a.check {
        return check(&a.out, &out, &files);
    }
    fs::create_dir_all(&a.out).map_err(|e| Error::io(&a.out, e))?;
    let mut written = 0;
    for (name, text) in &files {
        let p = a.out.join(name);
        if fs::read_to_string(&p).ok().as_deref() != Some(text) {
            write_atomic(&p, text.as_bytes())?;
            written += 1;
        }
    }
    Ok(json!({
        "ok": true,
        "out": out,
        "files": files.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        "written": written,
    }))
}

fn check(dir: &Path, out: &str, files: &[(String, String)]) -> Res<Value> {
    let bad: Vec<&str> = files
        .iter()
        .filter(|(n, text)| fs::read_to_string(dir.join(n)).ok().as_deref() != Some(text.as_str()))
        .map(|(n, _)| n.as_str())
        .collect();
    if bad.is_empty() {
        return Ok(json!({"ok": true, "out": out, "checked": files.len()}));
    }
    Err(Failure::with_exit(
        1,
        dla_core::Refusal::new(
            codes::SCHEMA_DRIFT,
            format!(
                "{} schema file(s) in {out} are missing or out of date",
                bad.len()
            ),
            bad.join(", "),
            format!("dla schema --out {out}"),
        ),
    ))
}
