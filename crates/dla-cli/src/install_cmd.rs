//! `dla install <backend>` → [`dla_backends::install`].

use std::path::{Path, PathBuf};

use clap::Args;
use dla_core::{detectors::KNOWN_BACKENDS, error::codes as core_codes};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    stage,
};

/// Flags of `dla install`.
#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Backend to install (dla, staticcheck, golangci-lint, govet, errorprone, spotbugs,
    /// detekt, codeql, sonar, ruff, pylint, eslint, rapx).
    backend: String,
}

/// Runs `dla install`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: InstallArgs) -> Res<Value> {
    if !KNOWN_BACKENDS.contains(&a.backend.as_str()) {
        return Err(Failure::refuse(
            core_codes::UNKNOWN_BACKEND,
            format!("{:?} is not a known backend", a.backend),
            format!("one of: {}", KNOWN_BACKENDS.join(", ")),
            "dla install <backend>",
        ));
    }
    let tc = stage::tool_ctx(project, ctx, data);
    stage::finish(
        "install",
        json!({"backend": a.backend}),
        dla_backends::install(&tc, &a.backend),
    )
}
