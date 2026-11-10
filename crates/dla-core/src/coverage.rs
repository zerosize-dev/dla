//! Coverage manifest (`coverage.json`): what each worker could and could not see.
//! Types only in M0; the report reads the file when present.

use std::{collections::BTreeMap, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    finding::Region,
    fsutil::{load_versioned, to_pretty_json, with_run_lock, write_atomic},
    ids::OpenItemId,
    state::{COVERAGE_FILE, RunState, Scope},
    version::Artifact,
};

string_enum! {
    /// Whether a backend ran.
    pub enum BackendStatus {
        /// It ran.
        Ran = "ran",
        /// It exists but could not run.
        Blocked = "blocked",
        /// It is not installed.
        Missing = "missing",
    }
}

/// One backend's coverage.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct BackendCoverage {
    /// Ran, blocked or missing.
    pub status: BackendStatus,
    /// Why it was blocked or missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// How to install or enable it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_hint: Option<String>,
    /// Backend version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Items it enumerated.
    pub enumerated: u64,
    /// Items that were dispositioned.
    pub dispositioned: u64,
    /// Hits that repeat an item already counted (same site, another rule or pass); `enumerated`
    /// counts hits, so findings plus duplicates account for it.
    #[serde(default)]
    pub duplicates: u64,
}

impl BackendCoverage {
    fn base(status: BackendStatus) -> Self {
        Self {
            status,
            reason: None,
            install_hint: None,
            version: None,
            enumerated: 0,
            dispositioned: 0,
            duplicates: 0,
        }
    }

    /// A backend that ran and enumerated `enumerated` items.
    pub fn ran(version: Option<String>, enumerated: u64) -> Self {
        Self {
            version,
            enumerated,
            ..Self::base(BackendStatus::Ran)
        }
    }

    /// A backend that is not installed, with the hint that installs it.
    pub fn missing(reason: impl Into<String>, install_hint: impl Into<String>) -> Self {
        Self {
            reason: Some(reason.into()),
            install_hint: Some(install_hint.into()),
            ..Self::base(BackendStatus::Missing)
        }
    }

    /// A backend that exists but could not run, and why.
    pub fn blocked(reason: impl Into<String>) -> Self {
        Self {
            reason: Some(reason.into()),
            ..Self::base(BackendStatus::Blocked)
        }
    }
}

/// An entry point that could not be exercised.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct NotExercisable {
    /// The entry point.
    pub entry: String,
    /// Why not.
    pub reason: String,
}

/// Entry points of a worker's runtime.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct EntryPoints {
    /// Found by static reading.
    #[serde(default)]
    pub discovered: Vec<String>,
    /// Actually run.
    #[serde(default)]
    pub exercised: Vec<String>,
    /// Found but not runnable.
    #[serde(default)]
    pub not_exercisable: Vec<NotExercisable>,
}

/// Coverage of one worker.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WorkerCoverage {
    /// Runtime name.
    pub runtime: String,
    /// Backends by name.
    #[serde(default)]
    pub backends: BTreeMap<String, BackendCoverage>,
    /// Entry points.
    #[serde(default)]
    pub entry_points: EntryPoints,
    /// Workload iterations run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u64>,
    /// Source files skipped because they lie outside the run's scope.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub out_of_scope: u64,
}

fn is_zero(n: &u64) -> bool {
    *n == 0
}

/// Growth left unattributed in one region.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ResidualRow {
    /// Memory region.
    pub region: Region,
    /// Total growth slope.
    pub total_slope: f64,
    /// Slope attributed to findings.
    pub attributed_slope: f64,
    /// Total minus attributed.
    pub residual: f64,
    /// Unit of the slopes.
    pub unit: String,
    /// Threshold above which the residual must be explained.
    pub threshold: f64,
    /// Open item covering an over-threshold residual.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_item: Option<OpenItemId>,
}

/// A thing a runtime's tooling cannot observe.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct BlindSpot {
    /// Runtime name.
    pub runtime: String,
    /// What cannot be seen.
    pub text: String,
}

/// `coverage.json`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct CoverageManifest {
    /// Schema version.
    pub schema_version: u32,
    /// Scope covered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
    /// Coverage by worker.
    #[serde(default)]
    pub workers: BTreeMap<String, WorkerCoverage>,
    /// Residual rows.
    #[serde(default)]
    pub residual: Vec<ResidualRow>,
    /// Declared blind spots.
    #[serde(default)]
    pub blind_spots: Vec<BlindSpot>,
}

impl WorkerCoverage {
    /// An empty worker for `runtime`.
    pub fn new(runtime: &str) -> Self {
        Self {
            runtime: runtime.to_owned(),
            backends: BTreeMap::new(),
            entry_points: EntryPoints::default(),
            iterations: None,
            out_of_scope: 0,
        }
    }
}

impl CoverageManifest {
    /// An empty manifest.
    pub fn new(scope: Option<Scope>) -> Self {
        Self {
            schema_version: crate::SCHEMA_VERSION,
            scope,
            workers: BTreeMap::new(),
            residual: Vec::new(),
            blind_spots: Vec::new(),
        }
    }

    /// Loads `<run_dir>/coverage.json`; `None` if the file does not exist.
    pub fn load(run_dir: &Path) -> Result<Option<Self>, Error> {
        let p = run_dir.join(COVERAGE_FILE);
        if !p.exists() {
            return Ok(None);
        }
        load_versioned(&p, Artifact::Coverage).map(Some)
    }

    /// Loads `<run_dir>/coverage.json`, or starts an empty manifest (with the scope
    /// from `state.json` when that can be read) if the file does not exist.
    pub fn load_or_create(run_dir: &Path) -> Result<Self, Error> {
        if let Some(m) = Self::load(run_dir)? {
            return Ok(m);
        }
        let scope = RunState::load(run_dir).ok().and_then(|s| s.scope);
        Ok(Self::new(scope))
    }

    /// The worker keyed `key`, created (with `runtime = key`) when absent.
    pub fn worker_mut(&mut self, key: &str) -> &mut WorkerCoverage {
        self.workers
            .entry(key.to_owned())
            .or_insert_with(|| WorkerCoverage::new(key))
    }

    /// Sets (replacing) one backend entry of a worker.
    pub fn set_backend(&mut self, worker: &str, backend: &str, cov: BackendCoverage) {
        self.worker_mut(worker)
            .backends
            .insert(backend.to_owned(), cov);
    }

    /// Records what a backend did, without downgrading: a `ran` entry that enumerated
    /// something stays when a later call reports the backend `missing` or `blocked`
    /// (another operation of the same backend may have run); every other combination
    /// replaces the entry. A `ran` entry keeps the larger `enumerated`.
    pub fn note_backend(&mut self, worker: &str, backend: &str, cov: BackendCoverage) {
        let w = self.worker_mut(worker);
        match w.backends.get_mut(backend) {
            Some(old) if old.status == BackendStatus::Ran && old.enumerated > 0 => {
                if cov.status == BackendStatus::Ran {
                    old.enumerated = old.enumerated.max(cov.enumerated);
                    if cov.version.is_some() {
                        old.version = cov.version;
                    }
                }
            }
            Some(old) => *old = cov,
            None => {
                w.backends.insert(backend.to_owned(), cov);
            }
        }
    }

    /// Replaces the entry-point lists of a worker.
    pub fn set_entry_points(&mut self, worker: &str, entry_points: EntryPoints) {
        self.worker_mut(worker).entry_points = entry_points;
    }

    /// Records how many workload iterations ran for a worker.
    pub fn set_iterations(&mut self, worker: &str, iterations: u64) {
        self.worker_mut(worker).iterations = Some(iterations);
    }

    /// Adds entry points that were exercised, keeping the list sorted and without
    /// duplicates. The entries already there stay.
    pub fn add_exercised(&mut self, worker: &str, entries: impl IntoIterator<Item = String>) {
        let list = &mut self.worker_mut(worker).entry_points.exercised;
        list.extend(entries);
        list.sort();
        list.dedup();
    }

    /// Replaces the residual rows of the regions named by `rows`; rows of other regions
    /// stay. The result is ordered by region.
    pub fn set_residual(&mut self, rows: Vec<ResidualRow>) {
        self.residual
            .retain(|old| rows.iter().all(|new| new.region != old.region));
        self.residual.extend(rows);
        self.residual.sort_by_key(|r| r.region);
    }

    /// Declares a blind spot of a runtime's tooling, once.
    pub fn add_blind_spot(&mut self, runtime: &str, text: &str) {
        let spot = BlindSpot {
            runtime: runtime.to_owned(),
            text: text.to_owned(),
        };
        if !self.blind_spots.contains(&spot) {
            self.blind_spots.push(spot);
        }
    }

    /// Writes the manifest to `<run_dir>/coverage.json` atomically and seals it.
    pub fn save(&self, run_dir: &Path) -> Result<(), Error> {
        let guard = crate::integrity::Guard::new(run_dir, COVERAGE_FILE);
        write_atomic(
            &run_dir.join(COVERAGE_FILE),
            to_pretty_json(self).as_bytes(),
        )?;
        guard.seal()?;
        Ok(())
    }

    /// Load-or-create, apply `f`, save: the one way commands update coverage. Other
    /// workers' entries are preserved. Runs under the same exclusive `run.lock` as
    /// [`RunState::update`], so concurrent workers never lose an update.
    pub fn update(
        run_dir: &Path,
        f: impl FnOnce(&mut CoverageManifest),
    ) -> Result<CoverageManifest, Error> {
        with_run_lock(run_dir, || {
            let mut m = Self::load_or_create(run_dir)?;
            f(&mut m);
            m.save(run_dir)?;
            Ok(m)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_backend_never_downgrades_a_backend_that_ran() {
        let mut m = CoverageManifest::new(None);
        m.note_backend("jvm", "jcmd", BackendCoverage::ran(Some("21".into()), 4));
        m.note_backend(
            "jvm",
            "jcmd",
            BackendCoverage::missing("gone", "install a JDK"),
        );
        assert_eq!(m.workers["jvm"].backends["jcmd"].status, BackendStatus::Ran);
        m.note_backend("jvm", "jcmd", BackendCoverage::ran(None, 9));
        assert_eq!(m.workers["jvm"].backends["jcmd"].enumerated, 9);
        assert_eq!(
            m.workers["jvm"].backends["jcmd"].version.as_deref(),
            Some("21")
        );
        // A missing entry is replaced by a later run, and a ran-nothing entry by a gap.
        m.note_backend(
            "jvm",
            "jfr",
            BackendCoverage::missing("no jfr", "install a JDK"),
        );
        m.note_backend("jvm", "jfr", BackendCoverage::ran(None, 1));
        assert_eq!(m.workers["jvm"].backends["jfr"].status, BackendStatus::Ran);
        m.note_backend("jvm", "x", BackendCoverage::ran(None, 0));
        m.note_backend("jvm", "x", BackendCoverage::blocked("why"));
        assert_eq!(
            m.workers["jvm"].backends["x"].status,
            BackendStatus::Blocked
        );
    }

    #[test]
    fn update_creates_then_merges_workers_and_backends() {
        let t = tempfile::tempdir().unwrap();
        assert!(CoverageManifest::load(t.path()).unwrap().is_none());
        CoverageManifest::update(t.path(), |m| {
            m.set_backend(
                "go",
                "staticcheck",
                BackendCoverage::missing("not on PATH", "dla install staticcheck"),
            );
            m.set_backend("go", "govet", BackendCoverage::ran(Some("1.24".into()), 3));
        })
        .unwrap();
        CoverageManifest::update(t.path(), |m| {
            m.set_backend("jvm", "spotbugs", BackendCoverage::blocked("no JDK"));
            m.set_backend("go", "staticcheck", BackendCoverage::ran(None, 7));
            m.set_entry_points(
                "go",
                EntryPoints {
                    discovered: vec!["main".into()],
                    ..EntryPoints::default()
                },
            );
        })
        .unwrap();
        let m = CoverageManifest::load(t.path()).unwrap().unwrap();
        assert_eq!(m.schema_version, crate::SCHEMA_VERSION);
        let go = &m.workers["go"];
        assert_eq!(go.runtime, "go");
        assert_eq!(go.backends["staticcheck"].status, BackendStatus::Ran);
        assert_eq!(go.backends["staticcheck"].enumerated, 7);
        assert!(go.backends["staticcheck"].install_hint.is_none());
        assert_eq!(go.backends["govet"].enumerated, 3);
        assert_eq!(go.entry_points.discovered, ["main"]);
        assert_eq!(
            m.workers["jvm"].backends["spotbugs"].status,
            BackendStatus::Blocked
        );
    }

    fn row(region: Region, residual: f64) -> ResidualRow {
        ResidualRow {
            region,
            total_slope: 10.0,
            attributed_slope: 10.0 - residual,
            residual,
            unit: "bytes/n".into(),
            threshold: 2.5,
            open_item: None,
        }
    }

    #[test]
    fn dynamic_helpers_keep_lists_sorted_and_replace_by_region() {
        let mut m = CoverageManifest::new(None);
        m.set_iterations("go", 4000);
        m.set_iterations("go", 8000);
        assert_eq!(m.workers["go"].iterations, Some(8000));
        m.add_exercised("go", ["b.Run".to_owned(), "a.Main".to_owned()]);
        m.add_exercised("go", ["b.Run".to_owned(), "c.Start".to_owned()]);
        assert_eq!(
            m.workers["go"].entry_points.exercised,
            ["a.Main", "b.Run", "c.Start"]
        );
        m.set_residual(vec![row(Region::OffHeap, 1.0), row(Region::Heap, 5.0)]);
        assert_eq!(m.residual[0].region, Region::Heap);
        m.set_residual(vec![row(Region::Heap, 0.0)]);
        assert_eq!(m.residual.len(), 2);
        assert_eq!(m.residual[0].residual, 0.0);
        assert_eq!(m.residual[1].region, Region::OffHeap);
        m.add_blind_spot("jvm", "native memory");
        m.add_blind_spot("jvm", "native memory");
        m.add_blind_spot("go", "cgo");
        assert_eq!(m.blind_spots.len(), 2);
    }

    #[test]
    fn missing_backend_keeps_its_hint_on_disk() {
        let t = tempfile::tempdir().unwrap();
        CoverageManifest::update(t.path(), |m| {
            m.set_backend(
                "python",
                "ruff",
                BackendCoverage::missing("ruff not found", "dla install ruff"),
            );
        })
        .unwrap();
        let text = std::fs::read_to_string(t.path().join(COVERAGE_FILE)).unwrap();
        assert!(text.contains("\"install_hint\": \"dla install ruff\""));
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn write_leaves_no_temporary_file() {
        let t = tempfile::tempdir().unwrap();
        CoverageManifest::update(t.path(), |m| {
            m.worker_mut("rust");
        })
        .unwrap();
        let names: Vec<_> = std::fs::read_dir(t.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n != crate::fsutil::RUN_LOCK_FILE)
            .collect();
        assert_eq!(names, [COVERAGE_FILE]);
    }

    #[test]
    fn a_corrupt_file_is_an_error_not_overwritten() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(t.path().join(COVERAGE_FILE), "{not json").unwrap();
        assert!(CoverageManifest::update(t.path(), |_| {}).is_err());
        let text = std::fs::read_to_string(t.path().join(COVERAGE_FILE)).unwrap();
        assert_eq!(text, "{not json");
    }
}
