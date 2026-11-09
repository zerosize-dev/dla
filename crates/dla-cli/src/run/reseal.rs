//! `dla run reseal --reason`: record an audit line and seal the protected files again.
//!
//! Skeleton: the flags are final, the command refuses with `not-implemented`. The gates
//! package (M6 package D) owns this file and replaces the stub.

use clap::Args;
use dla_core::error::codes as core_codes;
use serde_json::Value;

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
};

/// Flags of `dla run reseal`.
#[derive(Args, Debug)]
pub struct ResealArgs {
    /// Why the protected files changed outside `dla` (recorded in the ledger).
    #[arg(long)]
    pub reason: String,
}

/// `dla run reseal`.
pub fn run(_ctx: &Ctx, _a: ResealArgs) -> Res<Value> {
    Err(Failure::refuse(
        core_codes::NOT_IMPLEMENTED,
        "`dla run reseal` is not implemented in this build",
        "dla run reseal",
        "use a dla build that includes integrity seals (M6 package D)",
    ))
}
