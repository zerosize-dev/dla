//! `dla ledger label`, `strong` and `explain` (schema 6).
//!
//! Skeleton: the flags are final, the commands refuse with `not-implemented`. The ledger
//! package (M6 package B) owns this file and replaces the stubs.

use clap::Args;
use dla_core::{
    EvidenceId, FindingId, RemovalClaim, error::codes as core_codes, finding::ExplanationTag,
};
use serde_json::Value;

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
};

fn not_implemented(what: &str, package: &str) -> Failure {
    Failure::refuse(
        core_codes::NOT_IMPLEMENTED,
        format!("`{what}` is not implemented in this build"),
        what,
        format!("use a dla build that includes {package}"),
    )
}

/// Flags of `dla ledger label`.
#[derive(Args, Debug)]
pub struct LabelArgs {
    /// The finding (F3, F3.n1).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// The archetype ID (L-* on a leak-side finding, N-* on a non-leak sibling) or
    /// UNCLASSIFIED. DLA never picks a label.
    #[arg(long = "as", value_name = "ID", conflicts_with = "suggest")]
    pub label: Option<String>,
    /// Evidence IDs that meet the archetype's `requires`, comma separated (default: the
    /// evidence the finding cites or is supported by).
    #[arg(long, value_delimiter = ',', value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Vec<EvidenceId>,
    /// Why nothing fits (required with `--as UNCLASSIFIED`).
    #[arg(long)]
    pub note: Option<String>,
    /// Read only: per archetype, the met alternative or the missing evidence kinds.
    #[arg(long)]
    pub suggest: bool,
}

/// Flags of `dla ledger strong`.
#[derive(Args, Debug)]
pub struct StrongArgs {
    /// The finding (F3).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// The second independent record: a retention record or an intervention, comma
    /// separated.
    #[arg(long, value_delimiter = ',', required = true, value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Vec<EvidenceId>,
}

/// Flags of `dla ledger explain`.
#[derive(Args, Debug)]
pub struct ExplainArgs {
    /// The finding (F3).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// inferred | observed (observed needs --evidence).
    #[arg(long, value_parser = parse_enum::<ExplanationTag>)]
    pub tag: ExplanationTag,
    /// `file:line` of the sites that add to the holder, comma separated (default: the
    /// recorded writers).
    #[arg(long, value_delimiter = ',', value_name = "FILE:LINE")]
    pub adds: Vec<String>,
    /// never | not-found | conditional | wrong-key | unreachable
    #[arg(long, value_parser = parse_enum::<RemovalClaim>)]
    pub removal: RemovalClaim,
    /// `file:line` of the remover (required with conditional, wrong-key, unreachable).
    #[arg(long, value_name = "FILE:LINE")]
    pub remover: Option<String>,
    /// A retention, intervention or tied growth record that shows the pairing.
    #[arg(long, value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Option<EvidenceId>,
    /// The explanation (40 to 600 characters). It names the site, the holder, an adding
    /// site and how removal fails.
    #[arg(long)]
    pub text: String,
}

/// `dla ledger label`.
pub fn label(_ctx: &Ctx, _a: LabelArgs) -> Res<Value> {
    Err(not_implemented(
        "dla ledger label",
        "ledger schema 6 (M6 package B)",
    ))
}

/// `dla ledger strong`.
pub fn strong(_ctx: &Ctx, _a: StrongArgs) -> Res<Value> {
    Err(not_implemented(
        "dla ledger strong",
        "ledger schema 6 (M6 package B)",
    ))
}

/// `dla ledger explain`.
pub fn explain(_ctx: &Ctx, _a: ExplainArgs) -> Res<Value> {
    Err(not_implemented(
        "dla ledger explain",
        "ledger schema 6 (M6 package B)",
    ))
}
