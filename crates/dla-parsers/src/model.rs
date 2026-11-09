//! Provider-neutral models shared by the parsers and the `dla summarize` adapters.
//!
//! Every parser that feeds one of the four operations (heap-graph, type-histogram-diff,
//! alloc-stack-diff, non-heap) returns one of these types, so an adapter can diff or
//! group them without knowing which tool produced them. Names (types, functions, files)
//! are kept as the tool printed them; adapters normalize and redact.

use serde::{Deserialize, Serialize};

/// One type histogram: a row per type (a JVM class, a Rust type, a stack group).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Histogram {
    /// What produced the rows, e.g. `jcmd-class-histogram`, `hprof`, `shim-stack-groups`.
    pub source: String,
    /// Capture time in milliseconds since the Unix epoch, when the input records one.
    pub timestamp_ms: Option<u64>,
    pub rows: Vec<HistogramRow>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct HistogramRow {
    pub name: String,
    pub count: u64,
    pub bytes: u64,
}

/// One stack frame, innermost first in [`AllocGroup::frames`].
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Frame {
    /// Function or method, demangled where the parser can (`pkg.Class.method`, `crate::f`).
    pub function: String,
    /// Source file as the tool printed it (absolute, relative or bare name).
    pub file: Option<String>,
    pub line: Option<u32>,
    /// Shared object or module, when the tool names it.
    pub module: Option<String>,
}

/// Live (or sampled) allocations grouped by stack.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct AllocProfile {
    /// What produced the groups, e.g. `jfr-old-object-sample`, `heaptrack`, `dhat`, `valgrind`.
    pub source: String,
    pub timestamp_ms: Option<u64>,
    /// True when the tool samples, so counts and bytes are estimates.
    pub sampled: bool,
    pub groups: Vec<AllocGroup>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct AllocGroup {
    /// Innermost first.
    pub frames: Vec<Frame>,
    /// Bytes still live (or leaked) at the capture point.
    pub live_bytes: u64,
    pub live_blocks: u64,
    /// Bytes allocated in total by this stack, when the tool reports it.
    pub total_bytes: Option<u64>,
    /// What keeps the allocation alive, when the tool knows (a GC root path, a data
    /// symbol): outermost first, e.g. `["static shop.codec.Audit.trail", "java.util.ArrayList"]`.
    pub root_path: Vec<String>,
    /// Tool-specific kind, e.g. valgrind `definitely-lost` / `still-reachable`, or the
    /// JFR object type.
    pub kind: Option<String>,
}

/// Non-heap resources at one capture point: threads, file descriptors, class loaders,
/// native memory categories, goroutines, handles.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct NonHeapSnapshot {
    /// What produced it, e.g. `jvm-thread-dump`, `jvm-nmt`, `jvm-classloader-stats`, `proc-fd`.
    pub source: String,
    pub timestamp_ms: Option<u64>,
    /// Scalar metrics, e.g. category `nmt` name `Thread` value committed KB.
    pub metrics: Vec<NonHeapMetric>,
    /// Grouped resources, e.g. threads grouped by state or by creation site.
    pub groups: Vec<ResourceGroup>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct NonHeapMetric {
    pub category: String,
    pub name: String,
    pub value: i64,
    /// `count`, `bytes`, `kb`.
    pub unit: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ResourceGroup {
    /// `thread`, `fd`, `classloader`, `goroutine`, `handle`.
    pub kind: String,
    /// The grouping key, e.g. a thread state, a creation-site frame, an fd kind.
    pub key: String,
    pub count: u64,
    /// The project frame that created the resources, when known.
    pub site: Option<Frame>,
}
