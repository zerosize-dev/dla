//! Archetype table: `data/archetypes.toml`.
//!
//! One entry per archetype ID in [`crate::detectors::ARCHETYPE_IDS`]. An entry says which
//! holders and regions the archetype fits, which evidence a label must rest on
//! (`requires`), which non-leak siblings must be refuted first (`disconfirm`), and
//! the one-line mechanism. The file is keyed by archetype ID:
//!
//! ```toml
//! [L-COLLECT]
//! holders = ["container-field", "static/global"]
//! regions = ["heap"]
//! requires = [["unit-count-series"], ["retention-path"]]   # OR of ANDs
//! disconfirm = ["N-WARM", "N-HWM"]
//! signals = ["jvm.threads"]                               # hints only in M6
//! repair = ["bound/evict"]
//! summary = "A collection gains an entry per unit of work and nothing removes it."
//! ```
//!
//! Unknown keys and any string outside the closed enums fail the load with the file and
//! key named. `dla ledger label` reads this table; ledger replay never does (a label
//! record carries its own proof).
//!
//! The content is `data/archetypes.toml`; the tests in `crates/dla-core/tests/archetypes_data.rs`
//! keep it consistent with the ID list, the fixtures and the evidence producers.

use std::{collections::BTreeMap, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::{Refusal, codes},
    evidence::EvidenceKind,
    finding::{Holder, Region, RepairClass},
};

/// File name inside the plugin `data/` directory.
pub const FILE_NAME: &str = "archetypes.toml";
/// Environment variable that turns archetypes off when set to `off`.
pub const ENV_ARCHETYPES: &str = "DLA_ARCHETYPES";

/// Evidence kinds that some command or adapter writes today. An archetype `requires` list
/// may name only these kinds (a test in `dla-cli` checks that each is produced).
///
/// `blocked-stack-profile` and `unreachable-at-exit` have no producer and stay out.
/// `intervention` is produced by `dla intervene` (M6 package C); until that command lands
/// the producer test lists it in an explicit allow-list.
pub const PRODUCED_KINDS: [EvidenceKind; 16] = [
    EvidenceKind::RunConfig,
    EvidenceKind::Probe,
    EvidenceKind::StaticHit,
    EvidenceKind::CensusHolder,
    EvidenceKind::Workload,
    EvidenceKind::GrowthSeries,
    EvidenceKind::UnitCountSeries,
    EvidenceKind::RetentionPath,
    EvidenceKind::AllocStack,
    EvidenceKind::HistogramDiff,
    EvidenceKind::ShouldBeDead,
    EvidenceKind::Discriminator,
    EvidenceKind::Intervention,
    EvidenceKind::NonHeap,
    EvidenceKind::Residual,
    EvidenceKind::Artifact,
];

/// One archetype.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The archetype ID (`L-*` or `N-*`).
    pub id: String,
    /// Holders the archetype fits; empty for `N-*`.
    pub holders: Vec<Holder>,
    /// Memory regions the archetype fits.
    pub regions: Vec<Region>,
    /// Alternatives (OR); each alternative is a list of evidence kinds (AND).
    pub requires: Vec<Vec<EvidenceKind>>,
    /// `N-*` IDs whose sibling hypothesis must be refuted before a label.
    pub disconfirm: Vec<String>,
    /// Series or column names that hint at the archetype (documented, not consumed in M6).
    pub signals: Vec<String>,
    /// Repair classes that fit.
    pub repair: Vec<RepairClass>,
    /// One plain line: the mechanism.
    pub summary: String,
}

/// The loaded table, ordered by ID.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Table {
    entries: BTreeMap<String, Entry>,
}

impl Table {
    /// The entry for `id`.
    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.entries.get(id)
    }

    /// The IDs in the table, sorted.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    /// The entries, sorted by ID.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the table has no entry.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Parses the text of `archetypes.toml`; `origin` names the file in refusals.
    pub fn parse(text: &str, origin: &str) -> Result<Self, Refusal> {
        let bad = |what: String| {
            Refusal::new(
                codes::INVALID_DATA_FILE,
                format!("{origin}: {what}"),
                origin,
                "restore data/archetypes.toml from the plugin",
            )
        };
        let raw: BTreeMap<String, RawEntry> =
            toml::from_str(text).map_err(|e| bad(e.to_string().replace('\n', " ")))?;
        let mut entries = BTreeMap::new();
        for (id, r) in raw {
            if !crate::detectors::ARCHETYPE_IDS.contains(&id.as_str()) {
                return Err(bad(format!("[{id}] is not an archetype ID")));
            }
            let entry = Entry {
                id: id.clone(),
                holders: r.holders,
                regions: r.regions,
                requires: r.requires,
                disconfirm: r.disconfirm,
                signals: r.signals,
                repair: r.repair,
                summary: r.summary,
            };
            entries.insert(id, entry);
        }
        Ok(Self { entries })
    }
}

/// One entry of `archetypes.toml` as written in the file (the key is the archetype ID).
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RawEntry {
    /// Holders the archetype fits; empty for `N-*`.
    #[serde(default)]
    pub holders: Vec<Holder>,
    /// Memory regions the archetype fits.
    #[serde(default)]
    pub regions: Vec<Region>,
    /// Alternatives (outer list: any one); each alternative lists evidence kinds that must
    /// all be present (inner list: all).
    #[serde(default)]
    pub requires: Vec<Vec<EvidenceKind>>,
    /// `N-*` IDs whose sibling hypothesis must be refuted before a label.
    #[serde(default)]
    pub disconfirm: Vec<String>,
    /// Series names or name prefixes that hint at the archetype. Documentation only in M6:
    /// no code reads them.
    #[serde(default)]
    pub signals: Vec<String>,
    /// Repair classes that fit; empty for `N-*`.
    #[serde(default)]
    pub repair: Vec<RepairClass>,
    /// One plain line: the mechanism.
    pub summary: String,
}

/// The whole file, keyed by archetype ID. Exists for the JSON Schema export.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
#[serde(transparent)]
pub struct ArchetypesFile(pub BTreeMap<String, RawEntry>);

/// Loads `<data_dir>/archetypes.toml`: `Ok(None)` when the file is absent (archetypes
/// are off), a refusal that names the file when it cannot be read or is invalid.
pub fn load(data_dir: &Path) -> Result<Option<Table>, Refusal> {
    let path = data_dir.join(FILE_NAME);
    let origin = crate::paths::to_slash(&path);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(Refusal::new(
                codes::INVALID_DATA_FILE,
                format!("cannot read {origin}: {e}"),
                origin,
                "restore data/archetypes.toml from the plugin",
            ));
        }
    };
    Table::parse(&text, &origin).map(Some)
}

/// False when `DLA_ARCHETYPES=off` (any letter case) in `env`. The table being absent is a
/// separate way of being off: see [`load`].
pub fn archetypes_enabled(env: &dyn Fn(&str) -> Option<String>) -> bool {
    !env(ENV_ARCHETYPES).is_some_and(|v| v.trim().eq_ignore_ascii_case("off"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[L-COLLECT]
holders = ["container-field"]
regions = ["heap"]
requires = [["unit-count-series"], ["retention-path", "growth-series"]]
disconfirm = ["N-WARM"]
repair = ["bound/evict"]
summary = "A collection gains entries and nothing removes them."
"#;

    #[test]
    fn absent_file_is_none() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(load(d.path()).unwrap(), None);
    }

    #[test]
    fn parses_a_valid_entry() {
        let t = Table::parse(SAMPLE, "x").unwrap();
        let e = t.get("L-COLLECT").unwrap();
        assert_eq!(e.holders, vec![Holder::ContainerField]);
        assert_eq!(e.requires.len(), 2);
        assert_eq!(t.ids().collect::<Vec<_>>(), vec!["L-COLLECT"]);
    }

    #[test]
    fn unknown_key_and_bad_enum_name_the_file() {
        let r = Table::parse(&SAMPLE.replace("regions", "region"), "data/a.toml").unwrap_err();
        assert!(r.message.contains("data/a.toml"));
        let r = Table::parse(&SAMPLE.replace("heap", "stack"), "data/a.toml").unwrap_err();
        assert_eq!(r.code, codes::INVALID_DATA_FILE);
        let r = Table::parse(&SAMPLE.replace("L-COLLECT", "L-NOPE"), "data/a.toml").unwrap_err();
        assert!(r.message.contains("L-NOPE"));
    }

    #[test]
    fn env_off_disables() {
        let on = |_: &str| None;
        let off = |_: &str| Some("OFF".to_owned());
        assert!(archetypes_enabled(&on));
        assert!(!archetypes_enabled(&off));
    }
}
