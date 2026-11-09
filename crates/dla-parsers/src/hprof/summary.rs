//! The serializable result of summarizing one heap dump.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// What the summary rests on, to be stated next to every byte figure.
pub const SIZE_NOTES: [&str; 3] = [
    "byte figures are approximate: HPROF records no object sizes, so an instance is taken as an object header plus its fields and an array as an array header plus its elements, rounded up to 8 (the header and reference sizes assumed are listed in these notes; fields are not repacked or aligned)",
    "retained = shallow sizes summed over the dominator subtree of the strong graph (java.lang.ref.Reference.referent is not followed, so weakly or softly held objects are not counted)",
    "every class object is an artificial root, so a static field's target is retained by its class and class-loader retention is not attributed; stack frames are thread context only",
];

/// The dump's header.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HeaderInfo {
    /// `JAVA PROFILE 1.0.2`.
    pub version: String,
    /// Object identifier size in bytes (4 or 8).
    pub id_size: u32,
    /// Dump time, milliseconds since the epoch.
    pub timestamp_ms: u64,
}

/// Object and structure counts.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Counts {
    /// Instances, arrays and class objects.
    pub objects: u64,
    /// Loaded classes with a dump record.
    pub classes: u64,
    /// Plain instances.
    pub instances: u64,
    /// Object arrays.
    pub object_arrays: u64,
    /// Primitive arrays.
    pub primitive_arrays: u64,
    /// UTF8 string records.
    pub strings: u64,
    /// Stack frames.
    pub frames: u64,
    /// Reference edges kept (strong graph).
    pub edges: u64,
    /// GC root records.
    pub gc_roots: u64,
    /// Objects reachable from the roots through strong references.
    pub reachable: u64,
    /// Objects not reachable through strong references (weakly held or garbage).
    pub unreachable: u64,
}

/// One row of the class histogram.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClassStat {
    /// Class name with dots and `[]` for arrays.
    pub class: String,
    /// Objects of the class.
    pub count: u64,
    /// Approximate shallow bytes.
    pub shallow: u64,
}

/// What kind of root holds a holder.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolderKind {
    /// A static field of a class.
    #[serde(rename = "static")]
    Static,
    /// A thread: its object and what its frames reference.
    #[serde(rename = "thread")]
    Thread,
    /// JNI global references, grouped by referent class.
    #[serde(rename = "jni-global")]
    JniGlobal,
    /// Any other root object, named by the chain of labels from the root.
    #[serde(rename = "instance")]
    Instance,
}

impl HolderKind {
    /// The word used in keys and JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Thread => "thread",
            Self::JniGlobal => "jni-global",
            Self::Instance => "instance",
        }
    }
}

/// Where the retained memory piles up below a holder.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Accumulation {
    /// Class of the dominator-tree node where the heavy path fans out.
    pub class: String,
    /// Dominator-tree children of that node.
    pub children: u64,
}

/// An object graph root that retains memory.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Holder {
    /// Stable across dumps of one process: `static:<class>.<field>`, `thread:<name>`,
    /// `jni-global:<class>`, else `instance:<root kind>><class>`.
    pub key: String,
    /// Root kind.
    pub kind: HolderKind,
    /// Class owning the static field, the referent class, or the root object's class.
    pub owner_class: String,
    /// Static field name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// The class name as the dump has it, when `owner_class` (and `key`) were normalized
    /// from a generated proxy or lambda class name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_owner_class: Option<String>,
    /// Approximate retained bytes.
    pub retained: u64,
    /// Objects dominated (itself included).
    pub dominated_count: u64,
    /// Root, then the heavy path below it, at most 12 entries.
    #[serde(default)]
    pub path: Vec<String>,
    /// The fan-out point of the heavy path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accumulation: Option<Accumulation>,
    /// Top stack frames of a thread (context only, never a site).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<String>,
}

/// A field that keeps a set of objects alive although no root field dominates them.
///
/// The set is dominated by the super root (the objects hang from two or more roots, as
/// when a coroutine job tree and a static map both reach the same continuations) or by a
/// thread or JNI global. The field is found on the shortest strong path from a GC root
/// to a sample of the set: paths from static fields of application classes first, then
/// static fields of other classes, then thread, JNI and other roots.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ViaHolder {
    /// `static:<class>.<field>` (same form as [`Holder::key`]) or
    /// `field:<declaring class>.<field>` for an instance field.
    pub key: String,
    /// [`HolderKind::Static`] or [`HolderKind::Instance`].
    pub kind: HolderKind,
    /// Class declaring the field, proxy and lambda suffixes removed.
    pub owner_class: String,
    /// The class name as the dump has it, when `owner_class` differs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_owner_class: Option<String>,
    /// Field name.
    pub field: String,
    /// What dominates the set: `root`, `thread:<name>` or `jni-global:<class>`.
    pub dominator: String,
    /// Approximate retained bytes of the set.
    pub retained: u64,
    /// Objects dominated by the set's members (members included).
    pub dominated_count: u64,
    /// The accumulation class of the set: its class and how many members it has.
    pub accumulation: Accumulation,
    /// Root, then the edges down to a member, elided in the middle when long.
    #[serde(default)]
    pub path: Vec<String>,
}

/// Run statistics (not deterministic).
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Stats {
    /// File size read, bytes (after inflating, when the input was gzip).
    pub file_bytes: u64,
    /// Whether the input was gzip.
    pub gzip: bool,
    /// Dominator iterations.
    pub dominator_passes: u32,
    /// Compressed object references were inferred.
    pub compressed_oops: bool,
    /// Milliseconds spent inflating.
    pub inflate_ms: u64,
    /// Milliseconds in pass 1 (records).
    pub pass1_ms: u64,
    /// Milliseconds in pass 2 (edges).
    pub pass2_ms: u64,
    /// Milliseconds computing dominators and retained sizes.
    pub dominators_ms: u64,
    /// Peak resident set of the process afterwards, KiB (Linux only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak_rss_kib: Option<u64>,
}

/// The summary of one heap dump.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HeapSummary {
    /// Header.
    pub header: HeaderInfo,
    /// Counts.
    pub counts: Counts,
    /// GC roots by kind name.
    pub gc_roots_by_kind: BTreeMap<String, u64>,
    /// The 25 classes with the most shallow bytes.
    pub top_classes: Vec<ClassStat>,
    /// The 20 holders with the most retained bytes.
    pub holders: Vec<Holder>,
    /// Fields that keep memory alive when the dominator is a thread, a JNI global or the
    /// super root (see [`ViaHolder`]), largest first; at most 5.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub via_holders: Vec<ViaHolder>,
    /// How to read the byte figures.
    pub notes: Vec<String>,
    /// True when a cap stopped the analysis early: figures are partial.
    pub truncated: bool,
    /// Things worth knowing about this dump.
    pub warnings: Vec<String>,
    /// Run statistics.
    pub stats: Stats,
    /// Up to 200 holders (the first 60 with path and accumulation) for tracking a holder
    /// across dumps; not written to the summary file.
    #[serde(skip)]
    pub tracked: Vec<Holder>,
    /// Every class with live instances, most shallow bytes first (the same figures as
    /// `top_classes`, untruncated); not written to the summary file. See
    /// [`HeapSummary::to_histogram`].
    #[serde(skip)]
    pub histogram: Vec<ClassStat>,
    /// Sorted hashes of the loaded class names, to compare class sets across dumps; not
    /// written to the summary file.
    #[serde(skip)]
    pub class_hashes: Vec<u64>,
    /// The same for classes defined by a non-boot class loader (application and platform
    /// classes), which tell two programs on the same JDK apart; not written to the
    /// summary file.
    #[serde(skip)]
    pub app_class_hashes: Vec<u64>,
}

/// FNV-1a over `s`, the hash used for [`HeapSummary::class_hashes`].
pub fn name_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

impl HeapSummary {
    /// The full class table as a provider-neutral histogram (source `hprof`). Byte figures
    /// are the approximate shallow sizes of [`SIZE_NOTES`]; class names are as in
    /// `top_classes` (`byte[]`, `java.util.HashMap$Node`).
    pub fn to_histogram(&self) -> crate::model::Histogram {
        crate::model::Histogram {
            source: "hprof".to_owned(),
            timestamp_ms: Some(self.header.timestamp_ms),
            rows: self
                .histogram
                .iter()
                .map(|c| crate::model::HistogramRow {
                    name: c.class.clone(),
                    count: c.count,
                    bytes: c.shallow,
                })
                .collect(),
        }
    }
}
