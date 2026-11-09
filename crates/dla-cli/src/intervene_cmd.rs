//! `dla intervene`: edit a scratch copy of a harness, re-run it and judge whether the
//! growth went away.
//!
//! Skeleton: the flags are final, the command refuses with `not-implemented`. The
//! intervention package (M6 package C) owns this file and replaces the stub.

use std::path::PathBuf;

use clap::Args;
use dla_core::{FindingId, error::codes as core_codes};
use serde_json::Value;

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
};

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

/// `dla intervene`.
pub fn run(_ctx: &Ctx, _a: InterveneArgs) -> Res<Value> {
    Err(Failure::refuse(
        core_codes::NOT_IMPLEMENTED,
        "`dla intervene` is not implemented in this build",
        "dla intervene",
        "use a dla build that includes interventions (M6 package C)",
    ))
}
