//! The discriminator evidence convention.
//!
//! Every `discriminator` record written by DLA has in its `result`
//! `{"test": ..., "favors": "leak"|"non-leak"|"inconclusive", "supports": [...],
//! "finding": "F3"|null, ...test-specific numbers}`. The ledger reads `favors` to judge
//! `confirmed` and `refuted` moves ([`favors_of`]).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    evidence::{EvidenceKind, EvidenceRecord},
    ids::FindingId,
};

string_enum! {
    /// Which side of the leak question a discriminator supports.
    pub enum Favors {
        /// The growth is a leak.
        Leak = "leak",
        /// The growth has a non-leak explanation.
        NonLeak = "non-leak",
        /// The test could not tell.
        Inconclusive = "inconclusive",
    }
}

/// The typed part of a discriminator `result`; any other field is kept in `extra`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct DiscriminatorResult {
    /// `plateau`, `purge`, `gc`, `keys` or a combination such as `plateau+purge`.
    pub test: String,
    /// The verdict.
    pub favors: Favors,
    /// `N-*` ids for a non-leak verdict, `["leak"]` for a leak verdict.
    #[serde(default)]
    pub supports: Vec<String>,
    /// The finding the test was bound to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<FindingId>,
    /// Test-specific numbers.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl DiscriminatorResult {
    /// A result with no supports, finding or extra fields.
    pub fn new(test: &str, favors: Favors) -> Self {
        Self {
            test: test.to_owned(),
            favors,
            supports: Vec::new(),
            finding: None,
            extra: Map::new(),
        }
    }

    /// Reads a result; `None` when `test` or `favors` is missing or `favors` is not
    /// one of the three spellings, or `finding` is not a finding ID.
    pub fn parse(result: &Value) -> Option<Self> {
        serde_json::from_value(result.clone()).ok()
    }

    /// The result as a JSON value for an evidence record.
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// What a discriminator evidence record favors; `None` for any other kind of record
/// and for a `result` that does not carry a valid `favors`.
pub fn favors_of(record: &EvidenceRecord) -> Option<Favors> {
    if record.kind != EvidenceKind::Discriminator {
        return None;
    }
    record
        .result
        .get("favors")?
        .as_str()?
        .parse::<Favors>()
        .ok()
}

/// The finding a discriminator record is bound to, if its `result` names one.
pub fn finding_of(record: &EvidenceRecord) -> Option<FindingId> {
    record.result.get("finding")?.as_str()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{Authority, EvidenceCoverage, Source, Tool};
    use crate::ids::EvidenceId;
    use serde_json::json;

    fn record(kind: EvidenceKind, result: Value) -> EvidenceRecord {
        EvidenceRecord {
            evidence_id: EvidenceId::new(1),
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: Authority::Test,
            kind,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        }
    }

    #[test]
    fn favors_is_read_from_discriminators_only() {
        for (text, want) in [
            ("leak", Favors::Leak),
            ("non-leak", Favors::NonLeak),
            ("inconclusive", Favors::Inconclusive),
        ] {
            let r = record(EvidenceKind::Discriminator, json!({"favors": text}));
            assert_eq!(favors_of(&r), Some(want));
        }
        let other = record(EvidenceKind::GrowthSeries, json!({"favors": "leak"}));
        assert_eq!(favors_of(&other), None);
        for bad in [json!({}), json!({"favors": "Leak"}), json!({"favors": 1})] {
            assert_eq!(favors_of(&record(EvidenceKind::Discriminator, bad)), None);
        }
    }

    #[test]
    fn the_typed_result_keeps_extra_fields() {
        let v = json!({"test":"plateau+purge","favors":"non-leak","supports":["N-WARM","N-HWM"],
            "finding":"F3","column":"probe:A","columns":[{"name":"a"}]});
        let r = DiscriminatorResult::parse(&v).unwrap();
        assert_eq!(r.favors, Favors::NonLeak);
        assert_eq!(r.finding, Some(FindingId::new(3)));
        assert_eq!(r.extra["column"], "probe:A");
        assert_eq!(r.to_value(), v);
        let rec = record(EvidenceKind::Discriminator, v);
        assert_eq!(finding_of(&rec), Some(FindingId::new(3)));
        let unbound = record(
            EvidenceKind::Discriminator,
            json!({"test":"keys","favors":"leak","finding":null,"supports":["leak"]}),
        );
        assert_eq!(finding_of(&unbound), None);
        assert!(DiscriminatorResult::parse(&json!({"favors":"leak"})).is_none());
    }
}
