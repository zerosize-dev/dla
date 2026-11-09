//! The harness: the manifest `dla harness gen` writes and the dump records the
//! measured processes write.
//!
//! * [`HarnessManifest`] lives at `.dla/<run>/harness/H<n>/harness.json`.
//! * [`HarnessDump`] is one JSON line of `<harness dir>/out/<pid>.jsonl`, shared by
//!   every per-runtime agent, shim and driver. It is parsed leniently: agents are
//!   small scripts in six languages and a missing or extra field must not lose a
//!   measurement.
//!
//! A `workload` evidence record written by `harness gen` or `harness run` carries the
//! harness ID in `result.harness`; the ledger's `under_test` rule looks for it
//! ([`harness_id_of`]).

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::Error,
    fsutil::{to_pretty_json, write_atomic},
    ids::FindingId,
    runtime::Runtime,
    series::{KeyMode, numbered},
    version::{SchemaError, SchemaErrorKind, read_version},
};

/// Schema version of a harness manifest.
pub const HARNESS_SCHEMA_VERSION: u32 = 1;
/// Directory (inside the run directory) holding harness directories.
pub const HARNESS_DIR: &str = "harness";
/// Manifest file name inside a harness directory.
pub const MANIFEST_FILE: &str = "harness.json";
/// Version written in `dla_harness` by agents.
pub const DUMP_VERSION: u32 = 1;

/// Environment variable naming the file a measured process writes its dump lines to.
pub const ENV_OUT: &str = "DLA_HARNESS_OUT";
/// Environment variable naming the `probes.json` file.
pub const ENV_PROBES: &str = "DLA_HARNESS_PROBES";
/// Environment variable naming the purge to apply at exit.
pub const ENV_PURGE: &str = "DLA_PURGE";
/// Environment variable naming the scratch project root (to relativise sites).
pub const ENV_PROJECT: &str = "DLA_PROJECT";
/// Environment variable with the optional periodic dump interval in milliseconds.
pub const ENV_TICK_MS: &str = "DLA_TICK_MS";
/// Environment variable with the iteration count of this process.
pub const ENV_ITER: &str = "DLA_ITER";
/// Environment variable with the comma-separated capture kinds (`heap,histogram,jfr,nonheap`).
pub const ENV_CAPTURE: &str = "DLA_CAPTURE";
/// Environment variable naming the directory capture files are written to.
pub const ENV_CAPTURE_DIR: &str = "DLA_CAPTURE_DIR";
/// Environment variable with the number of heap dumps a driver takes (default 4).
pub const ENV_CAPTURE_DUMPS: &str = "DLA_CAPTURE_DUMPS";

string_enum! {
    /// How the harness drives the workload.
    pub enum HarnessMode {
        /// DLA runs the workload's own command once per point.
        Command = "command",
        /// A generated loop test runs inside one process.
        Driver = "driver",
    }
}

string_enum! {
    /// What measures the process.
    pub enum AgentKind {
        /// JVM premain agent.
        JvmAgent = "jvm-agent",
        /// Python `sitecustomize` module.
        PySitecustomize = "py-sitecustomize",
        /// Node `--require` preload.
        NodePreload = "node-preload",
        /// Go test wrapper around `main`.
        GoTestWrapper = "go-test-wrapper",
        /// `LD_PRELOAD` allocator shim.
        AllocShim = "alloc-shim",
        /// JVM loop test.
        DriverJvm = "driver-jvm",
        /// Python loop test.
        DriverPython = "driver-python",
        /// Node loop test.
        DriverNode = "driver-node",
        /// Go loop test.
        DriverGo = "driver-go",
        /// Rust loop test.
        DriverRust = "driver-rust",
        /// C or C++ loop test.
        DriverNative = "driver-native",
        /// Nothing beyond OS sampling.
        None = "none",
    }
}

string_enum! {
    /// What `dla harness run --capture` writes next to the series.
    pub enum CaptureKind {
        /// Heap dump (HPROF) of the measured process.
        Heap = "heap",
        /// Full class histogram text.
        Histogram = "histogram",
        /// Flight recording with old-object samples and paths to GC roots.
        Jfr = "jfr",
        /// Thread dump, class loader statistics and native memory summary.
        NonHeap = "nonheap",
        /// Valgrind memcheck XML (`--tool valgrind`).
        Valgrind = "valgrind",
        /// Heaptrack data and its printed report (`--tool heaptrack`).
        Heaptrack = "heaptrack",
    }
}

/// One file `dla harness run --capture` or `--tool` left in the harness `out/` directory.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct CaptureFile {
    /// What the file is.
    pub kind: CaptureKind,
    /// Run-relative path, for example `harness/H1/out/p1-fresh/capture/812/heap-2.hprof`.
    pub path: String,
    /// The measured point (`p3-native`, `p1-fresh`).
    pub label: String,
    /// Sequence number within the kind and label (1-based).
    pub seq: u32,
    /// Position on the axis (units completed or `n`), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// What this file is, in a few words (`threads.json`, `nmt summary`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Size in bytes.
    pub bytes: u64,
    /// Time the capture took in the measured process, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ms: Option<u64>,
    /// When the file was written (UTC, RFC 3339).
    pub captured_at: String,
}

/// A command to run in the scratch copy.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct CommandSpec {
    /// Program and arguments.
    pub argv: Vec<String>,
    /// Working directory, relative to the scratch root.
    pub cwd: String,
    /// Extra environment variables.
    #[serde(default)]
    pub env: Vec<(String, String)>,
}

/// What a probe reads.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ProbeTarget {
    /// Every static field of a JVM class.
    JvmClass {
        /// Fully qualified class name.
        class: String,
    },
    /// One static field of a JVM class.
    JvmStatic {
        /// Fully qualified class name.
        class: String,
        /// Field name.
        field: String,
    },
    /// Every module-level name of a Python module.
    PyModule {
        /// Module name.
        module: String,
    },
    /// One module-level Python name.
    PyGlobal {
        /// Module name.
        module: String,
        /// Name.
        name: String,
    },
    /// A Python attribute path such as `ReportBuilder.category_totals`.
    PyAttr {
        /// Module name.
        module: String,
        /// Qualified name inside the module.
        qualname: String,
    },
    /// Every top-level binding of a Node module file.
    NodeModule {
        /// Project-relative file.
        file: String,
    },
    /// One top-level binding of a Node module file.
    NodeBinding {
        /// Project-relative file.
        file: String,
        /// Binding name.
        name: String,
    },
    /// A Go package variable, reachable only from a driver test in that package.
    GoVar {
        /// Package directory.
        package_dir: String,
        /// Variable name.
        name: String,
    },
}

/// One holder probe.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ProbeSpec {
    /// `P1`.
    pub id: String,
    /// The finding the probe was derived from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<FindingId>,
    /// What to read.
    pub target: ProbeTarget,
    /// Definition site (`file:line`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
}

fn default_one() -> u32 {
    HARNESS_SCHEMA_VERSION
}

/// `harness/H<n>/harness.json`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct HarnessManifest {
    /// Schema version (1).
    #[serde(default = "default_one")]
    pub schema_version: u32,
    /// `H2`.
    pub id: String,
    /// Runtime of the workload.
    pub runtime: Runtime,
    /// Command or driver.
    pub mode: HarnessMode,
    /// Workload ID from discovery.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload: Option<String>,
    /// Run-relative path of the scratch copy, for example `harness/H2/work`.
    pub scratch: String,
    /// Built once before measuring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<CommandSpec>,
    /// The measured command; `{n}` is replaced by the iteration count in command mode.
    pub run: CommandSpec,
    /// How the iteration count is passed (`--iterations {n}`, `-n {n}`, `{n}`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iter_arg: Option<String>,
    /// Leading points that are warm-up (default 1).
    pub warmup: u32,
    /// Points after the warm-up (default 8).
    pub iterations: u32,
    /// Command mode: `n_k = step * k`; driver mode: units per round.
    pub step: u64,
    /// Driver mode: fresh and repeated; command mode: native.
    pub key_modes: Vec<KeyMode>,
    /// Holder probes.
    #[serde(default)]
    pub probes: Vec<ProbeSpec>,
    /// What measures the process.
    pub agent: AgentKind,
    /// Library detector flags switched on.
    #[serde(default)]
    pub detectors: Vec<String>,
    /// Named tools used (junit, pytest-memray, goleak, dhat).
    #[serde(default)]
    pub tools: Vec<String>,
    /// Gaps, for example "pytest-memray not installed: standard-library driver used".
    #[serde(default)]
    pub gaps: Vec<String>,
    /// Timeout per measured process in seconds (default 300).
    pub timeout_secs: u64,
    /// The allocator shim is loaded into the JVM because the project calls a native library
    /// (a JNI boundary found by `dla probe`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ffi: Option<FfiHarness>,
    /// Files written by `dla harness run --capture` or `--tool`, replaced by each run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub captures: Vec<CaptureFile>,
}

/// The foreign-function boundary a JVM harness measures.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct FfiHarness {
    /// `jni`.
    pub kind: String,
    /// The native library (`codec` for `System.loadLibrary("codec")`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library: Option<String>,
    /// Project-relative directory of the native crate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_dir: Option<String>,
    /// The exported JNI functions and the managed methods they implement.
    #[serde(default)]
    pub methods: Vec<FfiMethod>,
}

/// One JNI function and the managed method it implements.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct FfiMethod {
    /// The exported symbol (`Java_shop_codec_Native_open`).
    pub export: String,
    /// The managed method (`shop.codec.Native.open`).
    pub managed: String,
    /// Where the `native` or `external` declaration is (`kotlin/shop/Native.kt:7`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_at: Option<String>,
}

impl HarnessManifest {
    /// `<run_dir>/harness/<id>`.
    pub fn dir_in(run_dir: &Path, id: &str) -> PathBuf {
        run_dir.join(HARNESS_DIR).join(id)
    }

    /// `<run_dir>/harness/<id>/harness.json`.
    pub fn path_in(run_dir: &Path, id: &str) -> PathBuf {
        Self::dir_in(run_dir, id).join(MANIFEST_FILE)
    }

    /// Loads a manifest.
    pub fn load(run_dir: &Path, id: &str) -> Result<Self, Error> {
        let path = Self::path_in(run_dir, id);
        let text = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
        let malformed = |message: String| Error::Malformed {
            path: path.clone(),
            message,
        };
        let doc: Value = serde_json::from_str(&text).map_err(|e| malformed(e.to_string()))?;
        if let Some(found) = read_version(&doc)
            && found > HARNESS_SCHEMA_VERSION
        {
            return Err(Error::Schema(SchemaError {
                what: format!("{id}/{MANIFEST_FILE}"),
                found,
                supported: HARNESS_SCHEMA_VERSION,
                kind: SchemaErrorKind::Newer,
            }));
        }
        serde_json::from_value(doc).map_err(|e| malformed(e.to_string()))
    }

    /// Writes the manifest atomically, creating the harness directory.
    pub fn save(&self, run_dir: &Path) -> Result<PathBuf, Error> {
        let path = Self::path_in(run_dir, &self.id);
        if numbered('H', &self.id).is_none() {
            return Err(Error::Malformed {
                path,
                message: format!("harness id {:?} is not H<n>", self.id),
            });
        }
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
        }
        write_atomic(&path, to_pretty_json(self).as_bytes())?;
        Ok(path)
    }
}

/// The harness ID a `workload` evidence `result` names in `harness`, if any.
pub fn harness_id_of(result: &Value) -> Option<&str> {
    result
        .get("harness")?
        .as_str()
        .filter(|h| numbered('H', h).is_some())
}

string_enum! {
    /// When in the process a dump line was written.
    pub enum DumpPhase {
        /// One round of a driver.
        Checkpoint = "checkpoint",
        /// The end of the process.
        Exit = "exit",
        /// A periodic dump.
        Tick = "tick",
    }
}

/// A purge applied just before the last reading.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DumpPurge {
    /// `trim`, `gc`, `free-os-memory`, ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Values before the purge.
    #[serde(default)]
    pub before: BTreeMap<String, Option<f64>>,
    /// Values after the purge.
    #[serde(default)]
    pub after: BTreeMap<String, Option<f64>>,
}

/// One probe reading.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DumpProbe {
    /// Probe ID (`P1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Short name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Full path of the probed thing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Entries in the holder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    /// Objects reachable from it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reachable: Option<f64>,
    /// Estimated bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<f64>,
    /// Why it could not be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// A type with many instances.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DumpType {
    /// Type name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Live instances.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<f64>,
    /// Bytes they hold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<f64>,
}

/// What the process exercised.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DumpCoverage {
    /// `classes` or `functions`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// The items, inline.
    #[serde(default)]
    pub items: Vec<String>,
    /// Or a file holding them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

/// A library detector hit.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DumpDetector {
    /// Library name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lib: Option<String>,
    /// The log line or message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Where it was raised.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
}

/// One JSON line of `out/<pid>.jsonl`. Only `dla_harness` and `phase` are required.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct HarnessDump {
    /// Dump format version ([`DUMP_VERSION`]).
    pub dla_harness: u32,
    /// Checkpoint, exit or tick.
    pub phase: DumpPhase,
    /// Process ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Command line of the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub argv: Option<Vec<String>>,
    /// Executable path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exe: Option<String>,
    /// Runtime name (`jvm`, `go`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// Position on the axis (units completed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Milliseconds since the process started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_ms: Option<u64>,
    /// True when a collection was forced before reading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gc_forced: Option<bool>,
    /// Readings (after the forced collection where the runtime has one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<BTreeMap<String, Option<f64>>>,
    /// Readings taken just before the forced collection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values_pre_gc: Option<BTreeMap<String, Option<f64>>>,
    /// The purge applied, when `DLA_PURGE` was set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purge: Option<DumpPurge>,
    /// Grouped counters: base name to site to value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groups: Option<BTreeMap<String, BTreeMap<String, f64>>>,
    /// Unsymbolized allocation stacks from the allocator shim: stack key to
    /// `{live_bytes, live_blocks, allocs, frames: [{a, m, o}]}` (hex strings), kept as
    /// JSON because its shape belongs to the shim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_sites: Option<serde_json::Value>,
    /// Loaded modules the shim's frames refer to: `[{path, bias}]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modules: Option<serde_json::Value>,
    /// The shim's own bookkeeping counters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shim: Option<serde_json::Value>,
    /// Probe readings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probes: Option<Vec<DumpProbe>>,
    /// Top types by count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<DumpType>>,
    /// What was exercised.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<DumpCoverage>,
    /// Library detector hits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detectors: Option<Vec<DumpDetector>>,
    /// Known limits of this dump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limitations: Option<Vec<String>>,
    /// Files the agent wrote for `DLA_CAPTURE` at this record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captures: Option<Vec<DumpCapture>>,
}

/// One file an agent wrote next to a dump record (`DLA_CAPTURE`).
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DumpCapture {
    /// `heap`, `histogram`, `jfr` or `nonheap`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// File name inside `DLA_CAPTURE_DIR`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// What the file is (`threads.json`, `nmt summary`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    /// Milliseconds the capture took.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ms: Option<u64>,
    /// Why it was not written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl HarnessDump {
    /// Parses one line of a dump file. Unknown fields are ignored.
    pub fn parse_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }

    /// Parses every parseable line of a dump file's text; the second value counts the
    /// non-blank lines that did not parse.
    pub fn parse_lines(text: &str) -> (Vec<Self>, usize) {
        let mut bad = 0;
        let dumps = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| Self::parse_line(l).inspect_err(|_| bad += 1).ok())
            .collect();
        (dumps, bad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest() -> HarnessManifest {
        HarnessManifest {
            schema_version: 1,
            id: "H2".into(),
            runtime: Runtime::Go,
            mode: HarnessMode::Command,
            workload: Some("go:main:./cmd/x".into()),
            scratch: "harness/H2/work".into(),
            build: None,
            run: CommandSpec {
                argv: vec!["./x".into(), "-n".into(), "{n}".into()],
                cwd: ".".into(),
                env: vec![("A".into(), "1".into())],
            },
            iter_arg: Some("-n {n}".into()),
            warmup: 1,
            iterations: 8,
            step: 500,
            key_modes: vec![KeyMode::Native],
            probes: vec![ProbeSpec {
                id: "P1".into(),
                finding: Some(FindingId::new(1)),
                target: ProbeTarget::GoVar {
                    package_dir: "cmd/x".into(),
                    name: "cache".into(),
                },
                site: None,
            }],
            agent: AgentKind::GoTestWrapper,
            detectors: vec![],
            tools: vec![],
            gaps: vec!["goleak not available".into()],
            timeout_secs: 300,
            ffi: None,
            captures: vec![],
        }
    }

    #[test]
    fn manifest_round_trips_and_tags_the_probe_target() {
        let dir = tempfile::tempdir().unwrap();
        let m = manifest();
        let p = m.save(dir.path()).unwrap();
        assert_eq!(p, dir.path().join("harness/H2/harness.json"));
        assert_eq!(HarnessManifest::load(dir.path(), "H2").unwrap(), m);
        let v = serde_json::to_value(&m.probes[0]).unwrap();
        assert_eq!(v["target"]["kind"], "go-var");
        assert_eq!(v["target"]["package_dir"], "cmd/x");
    }

    #[test]
    fn dump_needs_only_the_marker_and_the_phase() {
        let d = HarnessDump::parse_line(r#"{"dla_harness":1,"phase":"exit"}"#).unwrap();
        assert_eq!(d.phase, DumpPhase::Exit);
        assert!(d.values.is_none() && d.pid.is_none());
        assert!(HarnessDump::parse_line(r#"{"phase":"exit"}"#).is_err());
        assert!(HarnessDump::parse_line(r#"{"dla_harness":1}"#).is_err());
    }

    #[test]
    fn dump_ignores_unknown_fields_and_reads_the_full_record() {
        let line = json!({
            "dla_harness":1,"phase":"checkpoint","pid":123,"argv":["a"],"exe":"/x","runtime":"jvm",
            "x":2000,"t_ms":1234,"gc_forced":true,
            "values":{"jvm.heap_used":123456,"jvm.threads":null},
            "values_pre_gc":{"jvm.heap_used":456789},
            "purge":{"kind":"trim","before":{"os.rss":10},"after":{"os.rss":5}},
            "groups":{"go.goroutines":{"a.go:3":8001}},
            "raw_sites":{"native.live_bytes":{"0x55":81920}},
            "probes":[{"id":"P1","name":"ORDERS","path":"shop.O.ORDERS","size":1750,"reachable":5400,"bytes":null,"error":null}],
            "types":[{"name":"shop.Order","count":1750,"bytes":84000}],
            "coverage":{"kind":"classes","items":["shop.O"]},
            "detectors":[{"lib":"netty","text":"LEAK","site":"x"}],
            "limitations":["l"],"future_field":{"a":1}
        });
        let d: HarnessDump = serde_json::from_value(line).unwrap();
        assert_eq!(d.pid, Some(123));
        assert_eq!(d.values.as_ref().unwrap()["jvm.threads"], None);
        assert_eq!(
            d.groups.as_ref().unwrap()["go.goroutines"]["a.go:3"],
            8001.0
        );
        assert_eq!(d.probes.as_ref().unwrap()[0].size, Some(1750.0));
        assert_eq!(d.purge.as_ref().unwrap().after["os.rss"], Some(5.0));
    }

    #[test]
    fn parse_lines_counts_bad_lines() {
        let text = "{\"dla_harness\":1,\"phase\":\"tick\"}\n\nnot json\n{\"dla_harness\":1,\"phase\":\"exit\"}\n";
        let (dumps, bad) = HarnessDump::parse_lines(text);
        assert_eq!((dumps.len(), bad), (2, 1));
    }

    #[test]
    fn harness_id_is_read_from_the_result() {
        assert_eq!(harness_id_of(&json!({"harness":"H2"})), Some("H2"));
        assert_eq!(harness_id_of(&json!({"harness":"x"})), None);
        assert_eq!(harness_id_of(&json!({"top":[]})), None);
    }
}
