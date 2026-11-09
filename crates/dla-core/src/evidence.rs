//! Evidence records: one ledger line per tool result.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    MAX_RESULT_BYTES,
    error::{Refusal, codes},
    ids::EvidenceId,
};

string_enum! {
    /// How strongly a piece of evidence was obtained.
    pub enum Authority {
        /// Read from source or configuration without running anything.
        Static = "static",
        /// Produced by a test or targeted experiment.
        Test = "test",
        /// Observed on a live process.
        Live = "live",
        /// Read from a captured dump.
        Dump = "dump",
    }
}

string_enum! {
    /// The closed set of evidence kinds.
    pub enum EvidenceKind {
        /// Run configuration captured at init.
        RunConfig = "run-config",
        /// Result of probing the environment.
        Probe = "probe",
        /// A static detector hit.
        StaticHit = "static-hit",
        /// A holder found by a census.
        CensusHolder = "census-holder",
        /// A workload execution record.
        Workload = "workload",
        /// A growth measurement series.
        GrowthSeries = "growth-series",
        /// A per-unit count series.
        UnitCountSeries = "unit-count-series",
        /// A retention path to a root.
        RetentionPath = "retention-path",
        /// An allocation stack.
        AllocStack = "alloc-stack",
        /// A before/after histogram difference.
        HistogramDiff = "histogram-diff",
        /// A profile of blocked stacks.
        BlockedStackProfile = "blocked-stack-profile",
        /// Objects unreachable at exit.
        UnreachableAtExit = "unreachable-at-exit",
        /// Objects that should be dead but are not.
        ShouldBeDead = "should-be-dead",
        /// An experiment that separates two hypotheses.
        Discriminator = "discriminator",
        /// A change made to the target to test a cause.
        Intervention = "intervention",
        /// A measurement outside the heap.
        NonHeap = "non-heap",
        /// Growth left unattributed.
        Residual = "residual",
        /// Files captured from a live process by an attached tool (heap dump, profile).
        Artifact = "artifact",
    }
}

string_enum! {
    /// Evidence classes used by the "strong" rule: two measurements of the same
    /// class count once.
    pub enum EvidenceClass {
        /// Growth measurements.
        Growth = "growth",
        /// Retention structure.
        Retention = "retention",
        /// Allocation sites.
        Allocation = "allocation",
        /// Interventions.
        Intervention = "intervention",
        /// Discriminators.
        Discriminator = "discriminator",
        /// Static reading.
        Static = "static",
        /// Everything else.
        Context = "context",
    }
}

impl EvidenceKind {
    /// The class of this kind.
    pub fn class(self) -> EvidenceClass {
        match self {
            Self::GrowthSeries | Self::UnitCountSeries => EvidenceClass::Growth,
            Self::RetentionPath | Self::HistogramDiff => EvidenceClass::Retention,
            Self::AllocStack => EvidenceClass::Allocation,
            Self::Intervention => EvidenceClass::Intervention,
            Self::Discriminator => EvidenceClass::Discriminator,
            Self::StaticHit | Self::CensusHolder => EvidenceClass::Static,
            Self::RunConfig
            | Self::Probe
            | Self::Workload
            | Self::BlockedStackProfile
            | Self::UnreachableAtExit
            | Self::ShouldBeDead
            | Self::NonHeap
            | Self::Residual
            | Self::Artifact => EvidenceClass::Context,
        }
    }
}

/// Where a piece of evidence came from.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// A file on disk.
    Artifact {
        /// Path of the artifact.
        path: String,
        /// SHA-256 of its contents, lowercase hex.
        sha256: String,
    },
    /// A running process.
    Process {
        /// Process ID.
        pid: u32,
        /// Process start time (platform string) to disambiguate PID reuse.
        start_time: String,
    },
    /// A source repository.
    Repo {
        /// Commit hash, or a project path when no commit is readable.
        commit: String,
        /// Path inside the repository.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
}

impl Source {
    /// Short human description used in indexes.
    pub fn describe(&self) -> String {
        match self {
            Source::Artifact { path, sha256 } => {
                format!("artifact {path} ({})", &sha256[..sha256.len().min(12)])
            }
            Source::Process { pid, start_time } => format!("process {pid} @ {start_time}"),
            Source::Repo { commit, path } => match path {
                Some(p) => format!("repo {commit}:{p}"),
                None => format!("repo {commit}"),
            },
        }
    }
}

/// The backend that produced the evidence.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Tool {
    /// Backend name.
    pub name: String,
    /// Backend version.
    pub version: String,
}

string_enum! {
    /// How much of the subject the evidence covers.
    pub enum CoverageStatus {
        /// The whole subject was examined.
        Complete = "complete",
        /// Part of the subject was examined.
        Partial = "partial",
        /// It is not known how much was examined.
        Unknown = "unknown",
    }
}

/// Coverage of one evidence record.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct EvidenceCoverage {
    /// Overall status.
    pub status: CoverageStatus,
    /// How many units were examined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub examined: Option<u64>,
    /// How many units exist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

impl Default for EvidenceCoverage {
    fn default() -> Self {
        Self {
            status: CoverageStatus::Unknown,
            examined: None,
            total: None,
        }
    }
}

/// A question the evidence leaves open and how to answer it.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Unknown {
    /// What is not known.
    pub question: String,
    /// What would resolve it.
    pub resolve_by: String,
}

/// One tool result.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct EvidenceRecord {
    /// Identifier, allocated by the ledger.
    pub evidence_id: EvidenceId,
    /// Where it came from.
    pub source: Source,
    /// Which backend produced it.
    pub tool: Tool,
    /// How it was obtained.
    pub authority: Authority,
    /// What it is.
    pub kind: EvidenceKind,
    /// The bounded result (serialized size at most [`MAX_RESULT_BYTES`]).
    pub result: Value,
    /// Path, relative to the run directory, of the full output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    /// How much of the subject it covers.
    #[serde(default)]
    pub coverage: EvidenceCoverage,
    /// Known limits of this evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
    /// Open questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknowns: Vec<Unknown>,
}

/// Evidence as supplied by a caller; the ledger allocates the ID.
#[derive(Debug, Clone, PartialEq)]
pub struct NewEvidence {
    /// Where it came from.
    pub source: Source,
    /// Which backend produced it.
    pub tool: Tool,
    /// How it was obtained.
    pub authority: Authority,
    /// What it is.
    pub kind: EvidenceKind,
    /// The bounded result.
    pub result: Value,
    /// Path of the full output, relative to the run directory.
    pub raw: Option<String>,
    /// Coverage.
    pub coverage: EvidenceCoverage,
    /// Known limits.
    pub limitations: Vec<String>,
    /// Open questions.
    pub unknowns: Vec<Unknown>,
}

impl NewEvidence {
    /// Attaches the allocated ID.
    pub fn into_record(self, evidence_id: EvidenceId) -> EvidenceRecord {
        EvidenceRecord {
            evidence_id,
            source: self.source,
            tool: self.tool,
            authority: self.authority,
            kind: self.kind,
            result: self.result,
            raw: self.raw,
            coverage: self.coverage,
            limitations: self.limitations,
            unknowns: self.unknowns,
        }
    }
}

/// Serialized size of an evidence `result`.
pub fn result_size(result: &Value) -> usize {
    serde_json::to_vec(result).map_or(usize::MAX, |v| v.len())
}

impl EvidenceRecord {
    /// Refuses a `result` whose serialized form exceeds [`MAX_RESULT_BYTES`].
    pub fn check_result_size(&self) -> Result<(), Refusal> {
        let size = result_size(&self.result);
        if size <= MAX_RESULT_BYTES {
            return Ok(());
        }
        Err(Refusal::new(
            codes::RESULT_TOO_LARGE,
            format!("evidence result is {size} bytes, over the {MAX_RESULT_BYTES}-byte bound"),
            format!("a result of at most {MAX_RESULT_BYTES} bytes"),
            "write the full output to a file under the run directory, set `raw` to its relative path, and keep only a summary in `result`",
        ))
    }
}

// ---------------------------------------------------------------------------------------
// Two signals
// ---------------------------------------------------------------------------------------

/// Tool classes of a [`Signal`]: what kind of observation a tool makes. Two tools of one
/// class (a second parser reading the same dump) make one signal, not two.
pub mod tool_class {
    /// A reader of a heap dump file.
    pub const DUMP_PARSER: &str = "dump-parser";
    /// A class or type histogram text or JSON from a live process or a harness.
    pub const HISTOGRAM: &str = "histogram";
    /// A sampling or tracing allocation profiler.
    pub const ALLOC_PROFILER: &str = "alloc-profiler";
    /// A leak checker that scans memory at exit (valgrind, LSan).
    pub const LEAK_CHECKER: &str = "leak-checker";
    /// Counters read from the runtime or the OS (threads, fds, native memory).
    pub const RUNTIME_COUNTER: &str = "runtime-counter";
    /// Stack groups from the allocator shim.
    pub const ALLOC_SHIM: &str = "alloc-shim";
}

/// Which observation a piece of evidence rests on: the tool, its class and the source
/// file(s). Carried in the evidence `result` as `signal`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    /// Tool name, e.g. `dla-hprof`.
    pub tool: String,
    /// Tool class, see [`tool_class`].
    pub class: String,
    /// SHA-256 of the source: the input file, or for several inputs the hash of their
    /// sorted hashes.
    pub source_sha256: String,
}

impl Signal {
    /// A signal from its parts.
    pub fn new(tool: &str, class: &str, source_sha256: &str) -> Self {
        Self {
            tool: tool.to_owned(),
            class: class.to_owned(),
            source_sha256: source_sha256.to_owned(),
        }
    }

    /// The signal in a bounded `result`'s `signal` key, if it has a valid one.
    pub fn of_result(result: &Value) -> Option<Self> {
        serde_json::from_value(result.get("signal")?.clone()).ok()
    }

    /// The signal of a record.
    pub fn of(record: &EvidenceRecord) -> Option<Self> {
        Self::of_result(&record.result)
    }

    /// The JSON form stored under `signal`.
    pub fn to_value(&self) -> Value {
        serde_json::json!({
            "tool": self.tool,
            "class": self.class,
            "source_sha256": self.source_sha256,
        })
    }
}

/// True when `a` and `b` are two independent signals: the tool, the tool class and the
/// source all differ. The DLA HPROF reader and another dump parser on one dump are one
/// signal (same class, same source). Recorded only: what counts as `strong` is decided
/// elsewhere.
pub fn independent(a: &Signal, b: &Signal) -> bool {
    a.tool != b.tool && a.class != b.class && a.source_sha256 != b.source_sha256
}

/// [`independent`] over two records; false when either carries no signal.
pub fn records_independent(a: &EvidenceRecord, b: &EvidenceRecord) -> bool {
    match (Signal::of(a), Signal::of(b)) {
        (Some(x), Some(y)) => independent(&x, &y),
        _ => false,
    }
}

#[cfg(test)]
mod signal_tests {
    use super::*;

    fn s(tool: &str, class: &str, src: &str) -> Signal {
        Signal::new(tool, class, src)
    }

    #[test]
    fn two_parsers_on_one_dump_are_one_signal() {
        let a = s("dla-hprof", tool_class::DUMP_PARSER, "aa");
        let b = s("mat", tool_class::DUMP_PARSER, "aa");
        assert!(!independent(&a, &b));
    }

    #[test]
    fn same_tool_class_on_other_sources_is_one_signal() {
        let a = s("dla-hprof", tool_class::DUMP_PARSER, "aa");
        let b = s("mat", tool_class::DUMP_PARSER, "bb");
        assert!(!independent(&a, &b));
    }

    #[test]
    fn a_dump_and_a_histogram_of_other_files_are_independent() {
        let a = s("dla-hprof", tool_class::DUMP_PARSER, "aa");
        let b = s("jcmd-class-histogram", tool_class::HISTOGRAM, "bb");
        assert!(independent(&a, &b));
        assert!(independent(&b, &a));
    }

    #[test]
    fn same_tool_is_never_independent() {
        let a = s("jfr", tool_class::ALLOC_PROFILER, "aa");
        let b = s("jfr", tool_class::RUNTIME_COUNTER, "bb");
        assert!(!independent(&a, &b));
    }

    #[test]
    fn signal_round_trips_through_a_result() {
        let sig = s("dla-hprof", tool_class::DUMP_PARSER, "ab12");
        let result = serde_json::json!({"x": 1, "signal": sig.to_value()});
        assert_eq!(Signal::of_result(&result), Some(sig));
        assert_eq!(Signal::of_result(&serde_json::json!({"signal": 3})), None);
        assert_eq!(Signal::of_result(&serde_json::json!({})), None);
    }
}
