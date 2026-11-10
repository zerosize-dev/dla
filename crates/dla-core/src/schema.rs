//! JSON Schema export for the contract types.

use schemars::{Schema, schema_for};

use crate::{
    archetypes::ArchetypesFile,
    census::CensusEntry,
    checklist::Checklist,
    coverage::CoverageManifest,
    detectors::DetectorFile,
    evidence::EvidenceRecord,
    finding::Finding,
    harness::{HarnessDump, HarnessManifest},
    known_leaks::KnownLeaksFile,
    probe::ProbeReport,
    record::LedgerLine,
    series::SeriesFile,
    state::RunState,
    static_hit::StaticHit,
    trend::TrendReport,
    workload::{Workload, WorkloadReport},
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
        ("detectors", schema_for!(DetectorFile)),
        ("known-leaks", schema_for!(KnownLeaksFile)),
        ("archetypes", schema_for!(ArchetypesFile)),
        ("probe-report", schema_for!(ProbeReport)),
        ("static-hit", schema_for!(StaticHit)),
        ("census-entry", schema_for!(CensusEntry)),
        ("workload", schema_for!(Workload)),
        ("workload-report", schema_for!(WorkloadReport)),
        ("series", schema_for!(SeriesFile)),
        ("trend", schema_for!(TrendReport)),
        ("harness-manifest", schema_for!(HarnessManifest)),
        ("harness-dump", schema_for!(HarnessDump)),
    ]
}

/// Renders a schema as pretty JSON with a trailing newline. Object keys are
/// sorted, so the output is deterministic.
pub fn render(schema: &Schema) -> String {
    crate::fsutil::to_pretty_json(schema)
}
