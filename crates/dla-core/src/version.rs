//! Schema versioning and the migration mechanism.
//!
//! `state.json`, `coverage.json` and every ledger line carry `schema_version`.
//! A newer version is refused; an older one is migrated step by step through a
//! table of `n -> n+1` functions, or refused plainly when no path exists. The
//! tables are empty because no document has needed a migration yet.
//!
//! Ledger lines are the exception to migration: versions 1 and 2 are both current.
//! Each line keeps its version, because the version selects the rule set the line
//! is replayed under (1: M0 rules, 2: M3 rules).

use serde_json::Value;

use crate::{LEDGER_SCHEMA_VERSION, SCHEMA_VERSION};

/// One migration step from `from` to `from + 1`, working on raw JSON.
#[derive(Clone, Copy)]
pub struct Migration {
    /// Version this step upgrades from.
    pub from: u32,
    /// The upgrade; returns the document in version `from + 1` shape.
    pub apply: fn(Value) -> Result<Value, String>,
}

/// Steps for ledger lines.
pub const LEDGER_MIGRATIONS: &[Migration] = &[];
/// Steps for `state.json`.
pub const STATE_MIGRATIONS: &[Migration] = &[];
/// Steps for `coverage.json`.
pub const COVERAGE_MIGRATIONS: &[Migration] = &[];

/// Which kind of document is being versioned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Artifact {
    /// A ledger line.
    LedgerLine,
    /// `state.json`.
    State,
    /// `coverage.json`.
    Coverage,
}

impl Artifact {
    /// The migration table for this artifact.
    pub fn migrations(self) -> &'static [Migration] {
        match self {
            Artifact::LedgerLine => LEDGER_MIGRATIONS,
            Artifact::State => STATE_MIGRATIONS,
            Artifact::Coverage => COVERAGE_MIGRATIONS,
        }
    }
}

/// Why a version could not be accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaErrorKind {
    /// Written by a newer dla.
    Newer,
    /// Older, and no migration path exists.
    CannotMigrate,
    /// A migration step failed.
    MigrationFailed(String),
}

/// A schema version mismatch; the CLI exits with code 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError {
    /// What was being read, e.g. `state.json` or `ledger line 4`.
    pub what: String,
    /// Version found in the document.
    pub found: u32,
    /// Version this binary reads.
    pub supported: u32,
    /// The failure.
    pub kind: SchemaErrorKind,
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            SchemaErrorKind::Newer => write!(
                f,
                "{}: written by a newer dla (schema {}, this binary reads {}); upgrade the plugin",
                self.what, self.found, self.supported
            ),
            SchemaErrorKind::CannotMigrate => write!(
                f,
                "{}: cannot migrate schema {}; start a new run with `dla run init`",
                self.what, self.found
            ),
            SchemaErrorKind::MigrationFailed(why) => write!(
                f,
                "{}: migrating schema {} failed: {why}; start a new run with `dla run init`",
                self.what, self.found
            ),
        }
    }
}

impl std::error::Error for SchemaError {}

impl SchemaError {
    /// Stable refusal code.
    pub fn code(&self) -> &'static str {
        match self.kind {
            SchemaErrorKind::Newer => "schema-newer",
            SchemaErrorKind::CannotMigrate => "schema-unmigratable",
            SchemaErrorKind::MigrationFailed(_) => "schema-migration-failed",
        }
    }
    /// What is missing.
    pub fn missing(&self) -> String {
        format!("a document in schema {}", self.supported)
    }
    /// The command or step that fixes it.
    pub fn fix(&self) -> &'static str {
        match self.kind {
            SchemaErrorKind::Newer => "upgrade the dla plugin",
            _ => "dla run init",
        }
    }
}

/// Failure of [`upgrade`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeError {
    /// `schema_version` missing or not a non-negative integer.
    Malformed(String),
    /// Version mismatch.
    Schema(SchemaError),
}

/// The `schema_version` of a raw document.
pub fn read_version(doc: &Value) -> Option<u32> {
    doc.get("schema_version")?
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
}

/// Brings `doc` to [`SCHEMA_VERSION`] using the artifact's migration table. A ledger
/// line is accepted unchanged when its version is between 1 and
/// [`LEDGER_SCHEMA_VERSION`].
pub fn upgrade(doc: Value, artifact: Artifact, what: &str) -> Result<Value, UpgradeError> {
    if artifact == Artifact::LedgerLine {
        let found = read_version(&doc).ok_or_else(|| {
            UpgradeError::Malformed(format!(
                "{what}: missing or invalid `schema_version` (expected a non-negative integer)"
            ))
        })?;
        if (1..=LEDGER_SCHEMA_VERSION).contains(&found) {
            return Ok(doc);
        }
        let kind = if found > LEDGER_SCHEMA_VERSION {
            SchemaErrorKind::Newer
        } else {
            SchemaErrorKind::CannotMigrate
        };
        return Err(UpgradeError::Schema(SchemaError {
            what: what.to_owned(),
            found,
            supported: LEDGER_SCHEMA_VERSION,
            kind,
        }));
    }
    upgrade_with(doc, artifact.migrations(), SCHEMA_VERSION, what)
}

/// Like [`upgrade`] with an explicit table and target version (used by tests and
/// by future callers that carry their own tables).
pub fn upgrade_with(
    mut doc: Value,
    migrations: &[Migration],
    supported: u32,
    what: &str,
) -> Result<Value, UpgradeError> {
    let found = read_version(&doc).ok_or_else(|| {
        UpgradeError::Malformed(format!(
            "{what}: missing or invalid `schema_version` (expected a non-negative integer)"
        ))
    })?;
    let err = |kind| {
        UpgradeError::Schema(SchemaError {
            what: what.to_owned(),
            found,
            supported,
            kind,
        })
    };
    if found > supported {
        return Err(err(SchemaErrorKind::Newer));
    }
    let mut v = found;
    while v < supported {
        let step = migrations
            .iter()
            .find(|m| m.from == v)
            .ok_or_else(|| err(SchemaErrorKind::CannotMigrate))?;
        doc = (step.apply)(doc).map_err(|why| err(SchemaErrorKind::MigrationFailed(why)))?;
        v += 1;
        if let Some(obj) = doc.as_object_mut() {
            obj.insert("schema_version".to_owned(), Value::from(v));
        }
    }
    Ok(doc)
}
