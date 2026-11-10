//! Measurement series: `.dla/<run>/series/S<n>.json`.
//!
//! A series is a table of columns sampled at the same points. Harness runs, live
//! samples, allocation tests and the gap analysis each write one; `dla trend` reads
//! them. Files are written atomically and validated on load and on save.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Refusal, codes},
    fsutil::{to_pretty_json, write_atomic},
    ids::FindingId,
    runtime::Runtime,
    version::{SchemaError, SchemaErrorKind, read_version},
};

/// Schema version of a series file.
pub const SERIES_SCHEMA_VERSION: u32 = 1;
/// Directory (inside the run directory) holding series files.
pub const SERIES_DIR: &str = "series";
/// Most columns one series may carry.
pub const MAX_COLUMNS: usize = 64;

string_enum! {
    /// Where a series came from.
    pub enum SeriesSourceKind {
        /// A harness run.
        Harness = "harness",
        /// A live sample of a process.
        Sample = "sample",
        /// An allocation test.
        AllocTest = "alloc-test",
        /// A derived runtime-accounted versus unexplained split.
        Gap = "gap",
        /// Holder sizes read from a series of heap dumps of one process.
        Dump = "dump",
        /// A harness run of a scratch copy with an edit applied (`dla intervene`).
        Intervention = "intervention",
    }
}

string_enum! {
    /// How the keys of the workload are produced.
    pub enum KeyMode {
        /// Every unit of work uses a new key.
        Fresh = "fresh",
        /// Every unit of work reuses the same keys.
        Repeated = "repeated",
        /// The keys are whatever the workload makes (command mode).
        Native = "native",
    }
}

string_enum! {
    /// What the x value of a point means.
    pub enum Axis {
        /// Units of work completed.
        Iteration = "iteration",
        /// Seconds since the start.
        Seconds = "seconds",
        /// The iteration count passed to the workload.
        N = "n",
        /// The position of a heap dump in a series ordered by capture time.
        Dump = "dump",
    }
}

string_enum! {
    /// Unit of a column.
    pub enum Unit {
        /// Bytes.
        Bytes = "bytes",
        /// A count of things.
        Count = "count",
        /// A ratio.
        Ratio = "ratio",
        /// Seconds.
        Seconds = "seconds",
    }
}

string_enum! {
    /// Whether a column is a level or an integer unit count.
    pub enum ColumnKind {
        /// A level (bytes, a ratio): judged by the slope gate.
        Gauge = "gauge",
        /// An integer unit count (goroutines, fds, entries): judged by the every-delta rule.
        Counter = "counter",
    }
}

string_enum! {
    /// Memory region of a column.
    pub enum SeriesRegion {
        /// Managed heap and holder probes.
        Heap = "heap",
        /// Native allocator, direct buffers, unexplained growth.
        OffHeap = "off-heap",
        /// Threads, goroutines, stacks, fds, timers, listeners, metaspace.
        NonHeap = "non-heap",
        /// Whole-process totals.
        Process = "process",
    }
}

string_enum! {
    /// What a column's figure covers.
    pub enum ColumnScope {
        /// The measured process.
        Process = "process",
        /// The whole container (cgroup figures).
        Container = "container",
    }
}

fn default_one() -> u32 {
    SERIES_SCHEMA_VERSION
}

/// Where a series came from.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct SeriesSource {
    /// Harness, sample, alloc-test or gap.
    pub kind: SeriesSourceKind,
    /// Harness ID (`H2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    /// Workload ID from discovery.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload: Option<String>,
    /// Key mode of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<KeyMode>,
    /// Runtime of the measured process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<Runtime>,
    /// Sampled PID (`sample --pid`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Command line of the measured process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The series a gap series was derived from (`S2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,
}

impl SeriesSource {
    /// A source of `kind` with every other field unset.
    pub fn new(kind: SeriesSourceKind) -> Self {
        Self {
            kind,
            harness: None,
            workload: None,
            mode: None,
            runtime: None,
            pid: None,
            command: None,
            derived_from: None,
        }
    }
}

/// One measured quantity.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Column {
    /// Stable name, for example `os.rss` or `go.goroutines@internal/notify/sub.go:33`.
    pub name: String,
    /// Unit.
    pub unit: Unit,
    /// Gauge or counter.
    pub kind: ColumnKind,
    /// Memory region.
    pub region: SeriesRegion,
    /// True for the always-sampled region series.
    #[serde(default)]
    pub baseline: bool,
    /// True when the value was read after a forced collection.
    #[serde(default)]
    pub after_gc: bool,
    /// Process or container.
    pub scope: ColumnScope,
    /// `file:line` this column is tied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Finding this column was probed for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<FindingId>,
    /// Origin of the figure: `proc-status`, `jvm-agent`, `go-harness`, `shim`, ...
    pub source: String,
}

/// One row of the table, aligned with the columns.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Point {
    /// Position on the axis.
    pub x: f64,
    /// Milliseconds since the start of the run.
    pub t_ms: u64,
    /// One value per column; `null` when the column could not be read at this point.
    pub values: Vec<Option<f64>>,
}

/// Something that happened between points.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct SeriesEvent {
    /// Index of the point the event follows.
    pub after_point: u32,
    /// `purge:trim`, `purge:gc`, ...
    pub kind: String,
    /// Free-form detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// `series/S<n>.json`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct SeriesFile {
    /// Schema version (1).
    #[serde(default = "default_one")]
    pub schema_version: u32,
    /// `S3`.
    pub id: String,
    /// Where it came from.
    pub source: SeriesSource,
    /// What x means.
    pub axis: Axis,
    /// Number of leading points that are warm-up.
    #[serde(default)]
    pub warmup: u32,
    /// The measured quantities (at most [`MAX_COLUMNS`]).
    pub columns: Vec<Column>,
    /// The rows, in x order.
    pub points: Vec<Point>,
    /// Events between points.
    #[serde(default)]
    pub events: Vec<SeriesEvent>,
    /// Known limits of this series.
    #[serde(default)]
    pub limitations: Vec<String>,
}

/// The number in an ID such as `S3` or `T12`, or `None` when `id` is not `<prefix><n>`.
pub fn numbered(prefix: char, id: &str) -> Option<u32> {
    let digits = id.strip_prefix(prefix)?;
    if digits.is_empty() || digits.starts_with('0') || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// The next number for `<prefix><n>` names in `dir`: the highest existing plus one.
/// A missing directory counts as empty. `suffix` is `".json"` for files or `""` for
/// directories.
pub fn next_number(dir: &Path, prefix: char, suffix: &str, dirs: bool) -> u32 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 1;
    };
    entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir() == dirs)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            numbered(prefix, name.strip_suffix(suffix)?)
        })
        .max()
        .unwrap_or(0)
        + 1
}

/// The IDs (`S1`, `S2`, ...) of the series files in `<run_dir>/series/` that hold content,
/// in numeric order. A reserved but unwritten placeholder does not count.
pub fn list_ids(run_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(run_dir.join(SERIES_DIR)) else {
        return Vec::new();
    };
    let mut ids: Vec<(u32, String)> = entries
        .filter_map(Result::ok)
        .filter(|e| crate::fsutil::is_written(&e.path()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(".json")?.to_owned();
            numbered('S', &id).map(|n| (n, id))
        })
        .collect();
    ids.sort();
    ids.into_iter().map(|(_, id)| id).collect()
}

fn invalid(why: impl Into<String>) -> Refusal {
    Refusal::new(
        codes::INVALID_SERIES,
        why,
        "a valid series file",
        "regenerate the series with the command that wrote it",
    )
}

impl SeriesFile {
    /// `<run_dir>/series/<id>.json`.
    pub fn path_in(run_dir: &Path, id: &str) -> PathBuf {
        run_dir.join(SERIES_DIR).join(format!("{id}.json"))
    }

    /// Index of the column called `name`.
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    /// The values of column `col`, one per point.
    pub fn values(&self, col: usize) -> Vec<Option<f64>> {
        self.points
            .iter()
            .map(|p| p.values.get(col).copied().flatten())
            .collect()
    }

    /// The `(x, y)` pairs of column `col` after the warm-up points, skipping missing values.
    pub fn post_warmup(&self, col: usize) -> Vec<(f64, f64)> {
        self.points
            .iter()
            .skip(self.warmup as usize)
            .filter_map(|p| Some((p.x, p.values.get(col).copied().flatten()?)))
            .collect()
    }

    /// Checks the invariants: version, ID, at most [`MAX_COLUMNS`] distinct non-empty
    /// column names, values aligned with the columns, finite x that never decreases,
    /// warm-up within the points and events that follow an existing point.
    pub fn validate(&self) -> Result<(), Refusal> {
        let id = &self.id;
        if self.schema_version != SERIES_SCHEMA_VERSION {
            return Err(invalid(format!(
                "series {id} has schema_version {}, expected {SERIES_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        if numbered('S', id).is_none() {
            return Err(invalid(format!("series id {id:?} is not S<n>")));
        }
        if self.columns.len() > MAX_COLUMNS {
            return Err(invalid(format!(
                "series {id} has {} columns; the cap is {MAX_COLUMNS}",
                self.columns.len()
            )));
        }
        let mut seen = HashSet::new();
        for c in &self.columns {
            if c.name.trim().is_empty() {
                return Err(invalid(format!("series {id} has a column with no name")));
            }
            if !seen.insert(c.name.as_str()) {
                return Err(invalid(format!(
                    "series {id} has two columns named {:?}",
                    c.name
                )));
            }
        }
        let mut last = f64::NEG_INFINITY;
        for (i, p) in self.points.iter().enumerate() {
            if p.values.len() != self.columns.len() {
                return Err(invalid(format!(
                    "series {id} point {i} has {} values for {} columns",
                    p.values.len(),
                    self.columns.len()
                )));
            }
            if !p.x.is_finite() || p.x < last {
                return Err(invalid(format!(
                    "series {id} point {i} has x {} after x {last}; x must be finite and non-decreasing",
                    p.x
                )));
            }
            if p.values.iter().flatten().any(|v| !v.is_finite()) {
                return Err(invalid(format!(
                    "series {id} point {i} has a value that is not finite"
                )));
            }
            last = p.x;
        }
        if self.warmup as usize > self.points.len() {
            return Err(invalid(format!(
                "series {id} has {} warm-up points but only {}",
                self.warmup,
                self.points.len()
            )));
        }
        if let Some(e) = self
            .events
            .iter()
            .find(|e| e.after_point as usize >= self.points.len())
        {
            return Err(invalid(format!(
                "series {id} has an event after point {}, which does not exist",
                e.after_point
            )));
        }
        Ok(())
    }

    /// Loads and validates `<run_dir>/series/<id>.json`.
    pub fn load(run_dir: &Path, id: &str) -> Result<Self, Error> {
        if numbered('S', id).is_none() {
            return Err(invalid(format!("series id {id:?} is not S<n>")).into());
        }
        Self::load_path(&Self::path_in(run_dir, id))
    }

    /// Loads and validates a series file at `path`.
    pub fn load_path(path: &Path) -> Result<Self, Error> {
        let text = fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        let malformed = |message: String| Error::Malformed {
            path: path.to_path_buf(),
            message,
        };
        let doc: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| malformed(e.to_string()))?;
        if let Some(found) = read_version(&doc)
            && found > SERIES_SCHEMA_VERSION
        {
            return Err(Error::Schema(SchemaError {
                what: path
                    .file_name()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
                found,
                supported: SERIES_SCHEMA_VERSION,
                kind: SchemaErrorKind::Newer,
            }));
        }
        let series: Self = serde_json::from_value(doc).map_err(|e| malformed(e.to_string()))?;
        series.validate()?;
        Ok(series)
    }

    /// Validates, then writes `<run_dir>/series/<id>.json` atomically.
    pub fn save(&self, run_dir: &Path) -> Result<PathBuf, Error> {
        self.validate()?;
        let path = Self::path_in(run_dir, &self.id);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
        }
        write_atomic(&path, to_pretty_json(self).as_bytes())?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(name: &str) -> Column {
        Column {
            name: name.into(),
            unit: Unit::Bytes,
            kind: ColumnKind::Gauge,
            region: SeriesRegion::Process,
            baseline: true,
            after_gc: false,
            scope: ColumnScope::Process,
            site: None,
            finding: None,
            source: "proc-status".into(),
        }
    }

    fn series() -> SeriesFile {
        SeriesFile {
            schema_version: 1,
            id: "S1".into(),
            source: SeriesSource::new(SeriesSourceKind::Harness),
            axis: Axis::N,
            warmup: 1,
            columns: vec![col("os.rss"), col("os.fds")],
            points: (0..4)
                .map(|i| Point {
                    x: f64::from(i) * 100.0,
                    t_ms: u64::from(i as u32) * 10,
                    values: vec![Some(1000.0 + f64::from(i)), None],
                })
                .collect(),
            events: vec![],
            limitations: vec![],
        }
    }

    #[test]
    fn round_trips_through_an_atomic_save() {
        let dir = tempfile::tempdir().unwrap();
        let s = series();
        let path = s.save(dir.path()).unwrap();
        assert_eq!(path, dir.path().join("series/S1.json"));
        assert_eq!(SeriesFile::load(dir.path(), "S1").unwrap(), s);
        let names: Vec<_> = fs::read_dir(dir.path().join("series"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["S1.json"]);
    }

    #[test]
    fn validation_rejects_each_broken_invariant() {
        let mut cases: Vec<(&str, SeriesFile)> = Vec::new();
        let mut s = series();
        s.points[2].values.pop();
        cases.push(("misaligned values", s));
        let mut s = series();
        s.points[3].x = 50.0;
        cases.push(("x decreases", s));
        let mut s = series();
        s.points[1].x = f64::NAN;
        cases.push(("x not finite", s));
        let mut s = series();
        s.columns = (0..=MAX_COLUMNS).map(|i| col(&format!("c{i}"))).collect();
        cases.push(("too many columns", s));
        let mut s = series();
        s.columns[1].name = "os.rss".into();
        cases.push(("duplicate column", s));
        let mut s = series();
        s.warmup = 9;
        cases.push(("warm-up beyond points", s));
        let mut s = series();
        s.id = "X1".into();
        cases.push(("bad id", s));
        let mut s = series();
        s.events.push(SeriesEvent {
            after_point: 4,
            kind: "purge:trim".into(),
            detail: None,
        });
        cases.push(("event after a missing point", s));
        for (what, s) in cases {
            let r = s.validate().expect_err(what);
            assert_eq!(r.code, codes::INVALID_SERIES, "{what}");
            assert!(
                s.save(tempfile::tempdir().unwrap().path()).is_err(),
                "{what}"
            );
        }
        // Equal x values are allowed (non-decreasing).
        let mut s = series();
        s.points[1].x = s.points[0].x;
        s.validate().unwrap();
    }

    #[test]
    fn exactly_the_cap_is_allowed() {
        let mut s = series();
        s.columns = (0..MAX_COLUMNS).map(|i| col(&format!("c{i}"))).collect();
        for p in &mut s.points {
            p.values = vec![Some(1.0); MAX_COLUMNS];
        }
        s.validate().unwrap();
    }

    #[test]
    fn post_warmup_drops_warmup_and_missing_values() {
        let s = series();
        assert_eq!(
            s.post_warmup(0),
            vec![(100.0, 1001.0), (200.0, 1002.0), (300.0, 1003.0)]
        );
        assert!(s.post_warmup(1).is_empty());
        assert_eq!(s.column_index("os.fds"), Some(1));
        assert_eq!(s.values(0)[0], Some(1000.0));
    }

    #[test]
    fn ids_are_allocated_after_the_highest_existing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(next_number(dir.path(), 'S', ".json", false), 1);
        let sub = dir.path().join(SERIES_DIR);
        fs::create_dir_all(&sub).unwrap();
        for n in [
            "S1.json",
            "S4.json",
            "S2.json",
            "notes.json",
            "S0.json",
            "T9.json",
        ] {
            fs::write(sub.join(n), "{}").unwrap();
        }
        assert_eq!(next_number(&sub, 'S', ".json", false), 5);
        assert_eq!(numbered('S', "S12"), Some(12));
        assert_eq!(numbered('S', "S012"), None);
        assert_eq!(numbered('S', "S"), None);
    }

    #[test]
    fn a_newer_schema_is_refused_with_the_schema_error() {
        let dir = tempfile::tempdir().unwrap();
        series().save(dir.path()).unwrap();
        let p = SeriesFile::path_in(dir.path(), "S1");
        let text = fs::read_to_string(&p)
            .unwrap()
            .replace("\"schema_version\": 1", "\"schema_version\": 2");
        fs::write(&p, text).unwrap();
        assert!(matches!(
            SeriesFile::load(dir.path(), "S1"),
            Err(Error::Schema(_))
        ));
    }
}

/// What a column name implies: unit, kind, region, and whether it is read after a
/// forced collection. Shared by every writer of series files so the same name always
/// means the same thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnMeta {
    /// Unit.
    pub unit: Unit,
    /// Gauge or counter.
    pub kind: ColumnKind,
    /// Memory region.
    pub region: SeriesRegion,
    /// Process or container.
    pub scope: ColumnScope,
    /// True for the always-sampled region series (OS and runtime totals).
    pub baseline: bool,
    /// True when the figure is read after a forced collection.
    pub after_gc: bool,
}

/// The catalog entry for a column name (spec section 1.1 naming).
///
/// * `<base>.pre_gc` is the same figure read before the forced collection
///   (`after_gc = false`); otherwise managed-runtime heap figures are after GC.
/// * `<base>@<file:line>` is a grouped counter of `<base>` tied to that site.
/// * `probe:<target>` counts entries or reachable objects (counter, heap);
///   `probe:<target>.bytes` is a byte estimate (gauge, heap).
/// * `jvm.class:<name>.count` and `py.type:<name>.count` are per-type instance counts.
///
/// Unknown names are gauges in bytes, region process, not baseline.
pub fn column_meta(name: &str) -> ColumnMeta {
    use ColumnKind::{Counter, Gauge};
    use SeriesRegion::{Heap, NonHeap, OffHeap, Process};
    use Unit::{Bytes, Count};
    let meta = |unit, kind, region, baseline, after_gc| ColumnMeta {
        unit,
        kind,
        region,
        scope: ColumnScope::Process,
        baseline,
        after_gc,
    };
    if let Some(base) = name.strip_suffix(".pre_gc") {
        let m = column_meta(base);
        return ColumnMeta {
            after_gc: false,
            baseline: false,
            ..m
        };
    }
    if let Some((base, _site)) = name.split_once('@') {
        let m = column_meta(base);
        return ColumnMeta {
            baseline: false,
            ..m
        };
    }
    if let Some(rest) = name.strip_prefix("probe:") {
        return if rest.ends_with(".bytes") {
            meta(Bytes, Gauge, Heap, false, true)
        } else {
            meta(Count, Counter, Heap, false, true)
        };
    }
    if (name.starts_with("jvm.class:") || name.starts_with("py.type:")) && name.ends_with(".count")
    {
        return meta(Count, Counter, Heap, false, true);
    }
    if name.starts_with("cgroup.") {
        return ColumnMeta {
            scope: ColumnScope::Container,
            ..meta(Bytes, Gauge, Process, true, false)
        };
    }
    match name {
        // Operating system, whole process.
        "os.rss" | "os.rss_anon" | "os.rss_file" | "os.rss_shmem" | "os.swap" | "os.vm_data"
        | "os.vm_hwm" | "os.pss" | "os.pss_anon" | "os.private_dirty" | "os.phys_footprint"
        | "os.private" | "os.pagefile" => meta(Bytes, Gauge, Process, true, false),
        "os.fds" | "os.handles" | "os.threads" => meta(Count, Counter, NonHeap, true, false),
        // JVM.
        "jvm.heap_used" => meta(Bytes, Gauge, Heap, true, true),
        "jvm.heap_committed" => meta(Bytes, Gauge, Heap, true, true),
        "jvm.nonheap_used" | "jvm.nonheap_committed" => meta(Bytes, Gauge, NonHeap, true, true),
        "jvm.nmt_committed" => meta(Bytes, Gauge, Process, true, true),
        "jvm.threads" | "jvm.classes_loaded" => meta(Count, Counter, NonHeap, true, true),
        // Go.
        "go.heap_live" | "go.heap_objects" | "go.heap_inuse" => {
            meta(Bytes, Gauge, Heap, true, true)
        }
        "go.total" | "go.released" => meta(Bytes, Gauge, Process, true, true),
        "go.stacks" => meta(Bytes, Gauge, NonHeap, true, true),
        "go.goroutines" => meta(Count, Counter, NonHeap, true, true),
        // Node.
        "node.heap_used" | "node.heap_total" => meta(Bytes, Gauge, Heap, true, true),
        "node.external" | "node.array_buffers" | "node.malloced_memory" => {
            meta(Bytes, Gauge, OffHeap, true, true)
        }
        "node.timers" | "node.listeners" | "node.native_contexts" | "node.detached_contexts" => {
            meta(Count, Counter, NonHeap, true, true)
        }
        // Python.
        "py.traced" | "py.traced_peak" => meta(Bytes, Gauge, Heap, true, true),
        "py.gc_objects" => meta(Count, Counter, Heap, true, true),
        // Allocator (Rust, native, shim).
        "native.live_bytes" | "native.peak_bytes" | "native.free_bytes" | "native.arena_bytes" => {
            meta(Bytes, Gauge, Heap, true, false)
        }
        "native.live_blocks" => meta(Count, Counter, Heap, true, false),
        // Gap split.
        "gap.accounted" => meta(Bytes, Gauge, Process, false, false),
        "gap.unexplained" => meta(Bytes, Gauge, OffHeap, false, false),
        _ if name.starts_with("node.active.") => meta(Count, Counter, NonHeap, false, true),
        // Open file descriptors by kind (`fd.socket`, `fd.pipe`, ...), from /proc or the shim.
        _ if name.starts_with("fd.") => meta(Count, Counter, NonHeap, true, false),
        _ => meta(Bytes, Gauge, Process, false, false),
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::*;

    #[test]
    fn names_map_to_the_documented_meaning() {
        let m = column_meta("go.goroutines@internal/notify/subscription.go:33");
        assert_eq!(
            (m.kind, m.region, m.baseline),
            (ColumnKind::Counter, SeriesRegion::NonHeap, false)
        );
        let m = column_meta("fd.socket");
        assert_eq!(
            (m.unit, m.kind, m.region),
            (Unit::Count, ColumnKind::Counter, SeriesRegion::NonHeap)
        );
        let m = column_meta("py.traced.pre_gc");
        assert_eq!((m.unit, m.after_gc), (Unit::Bytes, false));
        assert!(column_meta("py.traced").after_gc);
        assert_eq!(
            column_meta("probe:shop.OrderCache.ORDERS_BY_ID").kind,
            ColumnKind::Counter
        );
        assert_eq!(
            column_meta("probe:shop.OrderCache.ORDERS_BY_ID.bytes").kind,
            ColumnKind::Gauge
        );
        assert_eq!(column_meta("cgroup.current").scope, ColumnScope::Container);
        assert_eq!(column_meta("os.rss").region, SeriesRegion::Process);
        assert_eq!(
            column_meta("native.live_bytes@src/decoder.c:29").region,
            SeriesRegion::Heap
        );
        assert_eq!(column_meta("gap.unexplained").region, SeriesRegion::OffHeap);
    }
}
