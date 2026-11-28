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
    ids::{EvidenceId, FindingId},
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

/// A live heap column an alloc-test measured as still growing in its extra window.
#[derive(Debug, Clone, PartialEq)]
pub struct HeapNotStopping {
    /// The evidence record of the alloc-test.
    pub evidence_id: EvidenceId,
    /// The live heap column (`native.live_bytes`).
    pub column: String,
    /// How far the column rose over the extra window, when the record says.
    pub extra_rise: Option<f64>,
    /// The slope of the column over the extra window, when the record says.
    pub extra_slope: Option<f64>,
}

/// The live heap columns of the discriminator records bound to `finding` that say the level did
/// not stop: the column `favors: leak` and its plateau verdict is `growth-continues`. A column
/// of another kind (a process column, a high-water mark), a column that favors anything else
/// and a record bound to another finding are not listed. Oldest record first.
pub fn live_heap_growth_continues<'a>(
    records: impl IntoIterator<Item = &'a EvidenceRecord>,
    finding: &FindingId,
) -> Vec<HeapNotStopping> {
    let mut out = Vec::new();
    for r in records {
        if r.kind != EvidenceKind::Discriminator || finding_of(r).as_ref() != Some(finding) {
            continue;
        }
        let Some(columns) = r.result.get("columns").and_then(Value::as_array) else {
            continue;
        };
        for c in columns {
            let name = c.get("name").and_then(Value::as_str).unwrap_or_default();
            let plateau = c.get("plateau");
            let verdict = plateau
                .and_then(|p| p.get("verdict"))
                .and_then(Value::as_str);
            if crate::trend::is_live_heap_column(name)
                && c.get("favors").and_then(Value::as_str) == Some("leak")
                && verdict == Some("growth-continues")
            {
                let num = |k: &str| plateau.and_then(|p| p.get(k)).and_then(Value::as_f64);
                out.push(HeapNotStopping {
                    evidence_id: r.evidence_id.clone(),
                    column: name.to_owned(),
                    extra_rise: num("extra_rise"),
                    extra_slope: num("extra_slope"),
                });
            }
        }
    }
    out
}

/// True when `text` names the evidence record `id` (`E32`) as a whole word: `E320` and `XE32`
/// do not name it.
pub fn names_evidence(text: &str, id: &EvidenceId) -> bool {
    let want = id.to_string();
    text.match_indices(&want).any(|(i, _)| {
        let before = text[..i].chars().next_back();
        let after = text[i + want.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{Authority, EvidenceCoverage, Source, Tool};
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
    fn alloc_test(id: u32, finding: &str, columns: Value) -> EvidenceRecord {
        let mut r = record(
            EvidenceKind::Discriminator,
            json!({"test": "plateau+purge", "favors": "leak", "finding": finding, "columns": columns}),
        );
        r.evidence_id = EvidenceId::new(id);
        r
    }

    fn col(name: &str, favors: &str, verdict: &str) -> Value {
        json!({"name": name, "favors": favors,
            "plateau": {"verdict": verdict, "extra_rise": 573000.0, "extra_slope": 186.0}})
    }

    #[test]
    fn a_live_heap_column_that_favors_leak_and_keeps_growing_is_listed() {
        let f: FindingId = "F1".parse().unwrap();
        let r = alloc_test(
            32,
            "F1",
            json!([col("native.live_bytes", "leak", "growth-continues")]),
        );
        let got = live_heap_growth_continues([&r], &f);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].evidence_id, EvidenceId::new(32));
        assert_eq!(got[0].column, "native.live_bytes");
        assert_eq!(got[0].extra_rise, Some(573000.0));
        assert_eq!(got[0].extra_slope, Some(186.0));
    }

    #[test]
    fn other_columns_verdicts_and_findings_are_not_listed() {
        let f: FindingId = "F1".parse().unwrap();
        for c in [
            col("native.live_bytes", "non-leak", "growth-continues"),
            col("native.live_bytes", "leak", "plateau"),
            col("os.rss", "leak", "growth-continues"),
            col("native.peak_bytes", "leak", "growth-continues"),
        ] {
            let r = alloc_test(1, "F1", json!([c]));
            assert!(live_heap_growth_continues([&r], &f).is_empty());
        }
        let other = alloc_test(
            1,
            "F2",
            json!([col("native.live_bytes", "leak", "growth-continues")]),
        );
        assert!(live_heap_growth_continues([&other], &f).is_empty());
    }

    #[test]
    fn a_text_names_an_evidence_id_only_as_a_whole_word() {
        let id = EvidenceId::new(32);
        assert!(names_evidence("E32 shows growth", &id));
        assert!(names_evidence("see (E32).", &id));
        assert!(!names_evidence("E320 shows", &id));
        assert!(!names_evidence("XE32", &id));
        assert!(!names_evidence("no id", &id));
    }
}
