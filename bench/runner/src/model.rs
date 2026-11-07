//! Typed contracts: `expected.toml`, `findings.json`, `run.json`, `metrics.json`.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Runtime of a fixture.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Runtime {
    /// JVM languages.
    Jvm,
    /// Go.
    Go,
    /// Rust.
    Rust,
    /// Node.js and TypeScript.
    Node,
    /// Python.
    Python,
    /// C and other native code.
    Native,
}

/// Category of a fixture.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// A leak matching one taxonomy row.
    ArchetypeLeak,
    /// The bounded twin of a trap.
    DecoyControl,
    /// A real leak that fits no taxonomy row.
    ArchetypeFree,
    /// Growth that is not a leak.
    NonLeak,
    /// The pattern is in the code but nothing can run.
    Impossible,
    /// Two independent leaks.
    MultiLeak,
}

impl Category {
    /// The kebab-case spelling used in files.
    pub fn as_str(self) -> &'static str {
        match self {
            Category::ArchetypeLeak => "archetype-leak",
            Category::DecoyControl => "decoy-control",
            Category::ArchetypeFree => "archetype-free",
            Category::NonLeak => "non-leak",
            Category::Impossible => "impossible",
            Category::MultiLeak => "multi-leak",
        }
    }
}

/// What holds the retained memory.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holder {
    /// A static field, singleton or module global.
    #[serde(rename = "static/global")]
    StaticGlobal,
    /// A listener, timer, callback or subscription list.
    #[serde(rename = "registration-table")]
    RegistrationTable,
    /// A collection field of a long-lived object.
    #[serde(rename = "container-field")]
    ContainerField,
    /// Thread-local or async-context storage.
    #[serde(rename = "thread-local")]
    ThreadLocal,
    /// A thread, goroutine, task or coroutine that never finishes.
    #[serde(rename = "blocked-stack")]
    BlockedStack,
    /// A captured scope.
    #[serde(rename = "closure/continuation")]
    ClosureContinuation,
    /// Native memory, a handle or a global reference outside the managed heap.
    #[serde(rename = "native-handle/global-ref")]
    NativeHandleGlobalRef,
    /// Objects reachable only from each other.
    #[serde(rename = "cycle-only")]
    CycleOnly,
}

impl Holder {
    /// The spelling used in files.
    pub fn as_str(self) -> &'static str {
        match self {
            Holder::StaticGlobal => "static/global",
            Holder::RegistrationTable => "registration-table",
            Holder::ContainerField => "container-field",
            Holder::ThreadLocal => "thread-local",
            Holder::BlockedStack => "blocked-stack",
            Holder::ClosureContinuation => "closure/continuation",
            Holder::NativeHandleGlobalRef => "native-handle/global-ref",
            Holder::CycleOnly => "cycle-only",
        }
    }
}

/// Memory region.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Region {
    /// Managed heap.
    Heap,
    /// Native memory.
    OffHeap,
    /// Runtime-internal memory outside the managed heap.
    NonHeap,
}

fn yes() -> bool {
    true
}

fn full_share() -> f64 {
    1.0
}

/// The task given to the model.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Task {
    /// The prompt, symptom first; it never names the cause.
    pub prompt: String,
}

/// One expected leak.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Leak {
    /// Identifier within the fixture, such as `L1`.
    pub id: String,
    /// Project-relative `path:line` of the retaining or allocating line.
    pub site: String,
    /// Inclusive line range that counts as the site; defaults to `[line, line]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<[u32; 2]>,
    /// Other places a correct answer may name, as `path:line` or `path:start-end`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alt_sites: Vec<String>,
    /// What holds the memory.
    pub holder: Holder,
    /// Where the memory lives.
    pub region: Region,
    /// Archetype ID (`L-*`, `N-*`) or `UNCLASSIFIED`.
    #[schemars(regex(pattern = r"^(L-[A-Z]+|N-[A-Z]+|UNCLASSIFIED)$"))]
    pub cause: String,
    /// Share of the total growth (relevant with more than one leak).
    #[serde(default = "full_share")]
    pub share: f64,
    /// False when only static evidence is obtainable.
    #[serde(default = "yes")]
    pub confirmable: bool,
}

/// One expected non-leak.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NonLeak {
    /// Identifier within the fixture, such as `N1`.
    pub id: String,
    /// Optional project-relative `path:line`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Non-leak ID (`N-*`) or `UNCLASSIFIED`.
    #[schemars(regex(pattern = r"^(L-[A-Z]+|N-[A-Z]+|UNCLASSIFIED)$"))]
    pub cause: String,
    /// The observation that rules the leak out.
    pub discriminator: String,
}

/// `expected.toml`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    /// Schema version; 1.
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    /// Same as the directory name.
    pub id: String,
    /// Runtime.
    pub runtime: Runtime,
    /// Category.
    pub category: Category,
    /// Trap/control partner, reciprocal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair: Option<String>,
    /// False when no harness can run here.
    pub dynamic_confirmation: bool,
    /// The task.
    pub task: Task,
    /// Expected leaks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub leak: Vec<Leak>,
    /// Expected non-leaks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_leak: Vec<NonLeak>,
}

/// One entry of the `findings.json` that runs without dla write.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
pub struct FindingEntry {
    /// Project-relative `path:line`.
    pub site: String,
    /// One of the holder vocabulary.
    pub holder: String,
    /// `heap`, `off-heap` or `non-heap`.
    pub region: String,
    /// `leak-confirmed`, `leak-suspected` or `not-a-leak`.
    pub verdict: String,
    /// What keeps the memory alive and why it is never released.
    pub mechanism: String,
    /// Optional label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    /// What was run or measured.
    pub evidence: String,
    /// For `not-a-leak`: the observation that rules the leak out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,
}

/// `findings.json`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
pub struct FindingsFile {
    /// The conclusions.
    pub findings: Vec<FindingEntry>,
}

/// Token usage of one run.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default, PartialEq)]
pub struct Usage {
    /// Input tokens.
    #[serde(default)]
    pub input_tokens: u64,
    /// Output tokens.
    #[serde(default)]
    pub output_tokens: u64,
    /// Cache read input tokens.
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    /// Cache creation input tokens.
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
}

/// `run.json`, written by the runner.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct RunJson {
    /// Schema version; 1.
    pub schema_version: u32,
    /// Fixture ID.
    pub fixture: String,
    /// `no-skill`, `prose`, `gates` or `full`.
    pub ablation: String,
    /// `on` or `off`.
    pub archetypes: String,
    /// Run index, starting at 1.
    pub run: u32,
    /// Exit code of the `claude` process; -1 when it was killed.
    pub exit_code: i32,
    /// True when the timeout fired.
    pub timed_out: bool,
    /// Wall-clock seconds.
    pub wall_seconds: f64,
    /// Token usage from the final result event.
    pub usage: Usage,
    /// Cost from the final result event.
    pub cost_usd: Option<f64>,
    /// Turns from the final result event.
    pub num_turns: Option<u64>,
    /// Model from the init event.
    pub model: Option<String>,
    /// Start time, RFC 3339.
    pub started_at: String,
    /// Runner-side error, if any.
    pub error: Option<String>,
}

/// A ratio metric.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Ratio {
    /// Numerator.
    pub num: u64,
    /// Denominator.
    pub den: u64,
    /// `round6(num / den)`, null when `den` is 0.
    pub value: Option<f64>,
    /// One entry per run index.
    pub per_run: Vec<PerRun>,
    /// Mean of the non-null per-run values.
    pub mean: Option<f64>,
    /// Population standard deviation of the non-null per-run values.
    pub stdev: Option<f64>,
}

/// One run index of a ratio metric.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct PerRun {
    /// Run index.
    pub run: u32,
    /// Numerator.
    pub num: u64,
    /// Denominator.
    pub den: u64,
    /// `round6(num / den)`, null when `den` is 0.
    pub value: Option<f64>,
}

/// The twelve ratio metrics.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Metrics {
    /// Found leaks over the leak set.
    pub leak_recall: Ratio,
    /// Found leaks over leaks with cause UNCLASSIFIED.
    pub archetype_free_recall: Ratio,
    /// Flagged leaks over the leak set.
    pub flagged_recall: Ratio,
    /// Other leaks found in multi-leak runs.
    pub second_leak_recall: Ratio,
    /// Found leaks whose holder is right.
    pub grounded_attribution: Ratio,
    /// Found leaks with an archetype whose label is right.
    pub label_accuracy: Ratio,
    /// Found UNCLASSIFIED leaks given an archetype label.
    pub forced_fit_rate: Ratio,
    /// Impossible runs that claim.
    pub overclaim_rate: Ratio,
    /// Decoy and non-leak runs that claim.
    pub false_confirmation_rate: Ratio,
    /// Non-leak runs with no claim and a rule-out.
    pub non_leak_precision: Ratio,
    /// Trap runs that miss a leak.
    pub trap_miss_rate: Ratio,
    /// Decoy runs that claim.
    pub trap_false_alarm_rate: Ratio,
}

/// Unexplained growth share.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Unexplained {
    /// Residual over total slope.
    pub value: Option<f64>,
    /// Runs with a non-empty residual.
    pub runs: u64,
}

/// Sum and mean of an integer field.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct SumMean {
    /// Sum.
    pub sum: u64,
    /// Mean per run.
    pub mean: Option<f64>,
}

/// Sum, mean and max.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct WallSeconds {
    /// Sum.
    pub sum: f64,
    /// Mean per run.
    pub mean: Option<f64>,
    /// Largest.
    pub max: f64,
}

/// Cost.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Cost {
    /// Sum over runs with a numeric cost.
    pub sum: f64,
    /// Mean over those runs.
    pub mean: Option<f64>,
}

/// Where findings came from.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct Sources {
    /// Runs graded from the ledger.
    pub ledger: u64,
    /// Runs graded from findings.json.
    pub findings: u64,
    /// Runs with neither.
    pub none: u64,
}

/// Usage of a group.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct UsageStats {
    /// Graded runs in the group.
    pub runs: u64,
    /// Run directories skipped for lack of `run.json`.
    pub incomplete: u64,
    /// Runs whose exit code is missing, not an integer or not 0, or that timed out.
    pub errored: u64,
    /// Source of the findings per run.
    pub sources: Sources,
    /// Input tokens.
    pub input_tokens: SumMean,
    /// Output tokens.
    pub output_tokens: SumMean,
    /// Cache read tokens.
    pub cache_read_input_tokens: SumMean,
    /// Cache creation tokens.
    pub cache_creation_input_tokens: SumMean,
    /// Wall clock.
    pub wall_seconds: WallSeconds,
    /// Cost.
    pub cost_usd: Cost,
}

/// Counts for one expected leak over the runs of a fixture.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct LeakCounts {
    /// Runs in which the leak was found.
    pub found: u64,
    /// Runs in which the leak was flagged.
    pub flagged: u64,
    /// Runs in which it was found with the right holder.
    pub grounded: u64,
}

/// Per fixture statistics of a group.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct FixtureStats {
    /// Fixture category.
    pub category: String,
    /// Runs.
    pub runs: u64,
    /// Runs with at least one claim.
    pub claim_runs: u64,
    /// Runs with at least one rule-out.
    pub rule_out_runs: u64,
    /// Counts per expected leak.
    pub leaks: BTreeMap<String, LeakCounts>,
}

/// One ablation with one archetypes setting.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Group {
    /// Ablation.
    pub ablation: String,
    /// `on` or `off`.
    pub archetypes: String,
    /// The twelve ratio metrics.
    pub metrics: Metrics,
    /// Unexplained growth share.
    pub unexplained_growth_share: Unexplained,
    /// Usage.
    pub usage: UsageStats,
    /// Per fixture statistics.
    pub fixtures: BTreeMap<String, FixtureStats>,
}

/// `metrics.json`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct MetricsFile {
    /// Schema version; 1.
    pub schema_version: u32,
    /// Groups sorted by ablation, then archetypes.
    pub groups: Vec<Group>,
}
