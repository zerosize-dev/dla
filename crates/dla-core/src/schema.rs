//! JSON Schema export for the contract types.

use schemars::{Schema, schema_for};

use crate::{
    checklist::Checklist, coverage::CoverageManifest, evidence::EvidenceRecord, finding::Finding,
    record::LedgerLine, state::RunState,
};

/// Every exported schema as `(name, schema)`; `dla schema` writes
/// `<name>.schema.json`. The order is fixed.
pub fn all() -> Vec<(&'static str, Schema)> {
    vec![
        ("evidence-record", schema_for!(EvidenceRecord)),
        ("finding", schema_for!(Finding)),
        ("ledger-line", schema_for!(LedgerLine)),
        ("run-state", schema_for!(RunState)),
        ("coverage-manifest", schema_for!(CoverageManifest)),
        ("checklist", schema_for!(Checklist)),
    ]
}

/// Renders a schema as pretty JSON with a trailing newline. Object keys are
/// sorted, so the output is deterministic.
pub fn render(schema: &Schema) -> String {
    crate::fsutil::to_pretty_json(schema)
}
