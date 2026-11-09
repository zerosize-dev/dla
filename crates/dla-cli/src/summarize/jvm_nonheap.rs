//! `non-heap` over the JVM's own reports, read with DLA's parsers (no external tool):
//!
//! * `jvm-thread-dump`: `jcmd <pid> Thread.dump_to_file -format=json`,
//! * `jvm-classloader-stats`: `jcmd <pid> VM.classloader_stats`,
//! * `jvm-nmt`: `jcmd <pid> VM.native_memory summary` and `summary.diff`.
//!
//! Thread names are data. The parser groups threads by a normalized name (`pool-3-thread-7`
//! is `pool-*-thread-*`); a pattern is shown as it is only when it is plain program text
//! (short words and separators around `*`). Any other pattern, such as an id-like or key-like
//! word, becomes a salted shape. Thread-dump groups carry no creation site (the
//! JDK 21 JSON has none), so they never tie. Class loader and native-memory growth are
//! whole-runtime columns and never tie either.

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
};
use dla_parsers::{
    jvm::{classloader, nmt, thread_dump},
    model::NonHeapSnapshot,
};

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    input::Input,
    non_heap::{self, SnapCapture, SnapOrigin},
    redact::Redactor,
};
use crate::fail::{Failure, Res};

/// Largest report file read.
const MAX_BYTES: u64 = 64 << 20;
/// Longest thread-name pattern shown as it is.
const PATTERN_MAX: usize = 48;
/// Longest word in a pattern shown as it is.
const WORD_MAX: usize = 20;

fn refuse_parse(input: &Input, msg: &str, expected: &str, fix: &str) -> Failure {
    Failure::refuse(
        "malformed-artifact",
        format!("{}: {msg}", input.shown()),
        expected,
        fix,
    )
}

fn run_snaps(
    ctx: &Ctx<'_>,
    opts: &Opts,
    tool: &'static str,
    snaps: Vec<(String, NonHeapSnapshot, String)>,
    limitations: Vec<String>,
) -> Res<AdapterOut> {
    let captures = snaps
        .into_iter()
        .map(|(label, snap, sha256)| SnapCapture {
            label,
            snap,
            sha256,
        })
        .collect();
    non_heap::run_diff(
        ctx,
        opts,
        captures,
        &SnapOrigin {
            tool,
            class: tool_class::RUNTIME_COUNTER,
            authority: Authority::Live,
            runtime: Some(Runtime::Jvm),
            // The reports carry no time the parser reads, so they stay in the given order.
            order_by_time: false,
            limitations,
        },
    )
}

// ---------------------------------------------------------------------------------------
// thread dump
// ---------------------------------------------------------------------------------------

/// True when a normalized thread-name pattern is plain program text: words of letters
/// and `*` (a digit run) around separators. A word longer than `WORD_MAX`, or with more
/// than two `*`, looks like a value (an id, a key) and is not plain.
fn plain_pattern(p: &str) -> bool {
    if p.is_empty() || p.len() > PATTERN_MAX {
        return false;
    }
    let mut word = 0usize;
    let mut stars = 0usize;
    for c in p.chars() {
        match c {
            c if c.is_ascii_alphabetic() || c == '*' => {
                word += 1;
                if c == '*' {
                    stars += 1;
                }
                if word > WORD_MAX || stars > 2 {
                    return false;
                }
            }
            '-' | '_' | ' ' | '.' | '#' | '/' | ':' | '|' | '[' | ']' | '(' | ')' => {
                word = 0;
                stars = 0;
            }
            _ => return false,
        }
    }
    true
}

/// The key of a `name:<pattern>` group, shaped unless the pattern is plain program text
/// or the run has raw-values consent.
fn thread_group_key(key: &str, red: &Redactor) -> String {
    let Some(pattern) = key.strip_prefix("name:") else {
        return key.to_owned();
    };
    if red.mode() == "raw" || (plain_pattern(pattern) && red.name(pattern) == pattern) {
        key.to_owned()
    } else {
        format!("name:{}", red.value(pattern))
    }
}

/// The JVM thread dump reader.
pub struct JvmThreadDump;

impl Adapter for JvmThreadDump {
    fn name(&self) -> &'static str {
        "jvm-thread-dump"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "jvm"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::RUNTIME_COUNTER
    }
    fn formats(&self) -> &'static str {
        "JVM thread dump JSON (jcmd <pid> Thread.dump_to_file -format=json <file>)"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        h.trim_start().starts_with('{') && h.contains("\"threadDump\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut snaps = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let mut snap = thread_dump::parse(&text).map_err(|e| {
                refuse_parse(
                    i,
                    e.message(),
                    "the JSON file `jcmd <pid> Thread.dump_to_file -format=json <file>` writes",
                    "dla attach jcmd --pid <pid> --op threads   and pass the file it writes",
                )
            })?;
            for g in &mut snap.groups {
                g.key = thread_group_key(&g.key, &ctx.redactor);
            }
            snaps.push((i.shown().to_owned(), snap, i.sha256.clone()));
        }
        run_snaps(
            ctx,
            opts,
            self.name(),
            snaps,
            vec![
                "the JDK 21 thread dump has no creation site and no state: groups never tie to a finding by site".into(),
                "thread names are shown only as a plain normalized pattern; any other name is a shape".into(),
            ],
        )
    }
}

// ---------------------------------------------------------------------------------------
// class loader stats
// ---------------------------------------------------------------------------------------

/// The `VM.classloader_stats` reader.
pub struct JvmClassloaderStats;

impl Adapter for JvmClassloaderStats {
    fn name(&self) -> &'static str {
        "jvm-classloader-stats"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "jvm"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::RUNTIME_COUNTER
    }
    fn formats(&self) -> &'static str {
        "JVM class loader stats text (jcmd <pid> VM.classloader_stats)"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        h.contains("ClassLoader") && h.contains("CLD*") && h.contains("ChunkSz")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut snaps = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let snap = classloader::parse(&text).map_err(|e| {
                refuse_parse(
                    i,
                    e.message(),
                    "the output of `jcmd <pid> VM.classloader_stats`",
                    "jcmd <pid> VM.classloader_stats > classloader-stats.txt",
                )
            })?;
            snaps.push((i.shown().to_owned(), snap, i.sha256.clone()));
        }
        run_snaps(
            ctx,
            opts,
            self.name(),
            snaps,
            vec![
                "loaders and classes are whole-runtime counts: they never tie to a finding".into(),
            ],
        )
    }
}

// ---------------------------------------------------------------------------------------
// native memory tracking
// ---------------------------------------------------------------------------------------

/// True when an NMT report carries deltas (`summary.diff`).
fn nmt_is_diff(text: &str) -> bool {
    text.lines().any(|l| {
        let t = l.trim();
        (t.starts_with("Total:") || (t.starts_with('-') && t.contains(" (reserved=")))
            && (t.contains("KB +")
                || t.contains("KB -")
                || t.contains("MB +")
                || t.contains("MB -"))
    })
}

/// The `VM.native_memory` reader.
pub struct JvmNmt;

impl Adapter for JvmNmt {
    fn name(&self) -> &'static str {
        "jvm-nmt"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "jvm"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::RUNTIME_COUNTER
    }
    fn formats(&self) -> &'static str {
        "JVM native memory text (jcmd <pid> VM.native_memory summary or summary.diff)"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        h.contains("Native Memory Tracking") && h.contains("Total: reserved=")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut snaps = Vec::new();
        let mut kinds: Vec<(bool, &str)> = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let diff = nmt_is_diff(&text);
            let snap = if diff {
                nmt::parse_diff(&text)
            } else {
                nmt::parse_summary(&text)
            }
            .map_err(|e| {
                refuse_parse(
                    i,
                    e.message(),
                    "the output of `jcmd <pid> VM.native_memory summary` (or summary.diff) from a JVM started with -XX:NativeMemoryTracking=summary",
                    "restart the JVM with -XX:NativeMemoryTracking=summary, then jcmd <pid> VM.native_memory summary",
                )
            })?;
            kinds.push((diff, i.shown()));
            snaps.push((i.shown().to_owned(), snap, i.sha256.clone()));
        }
        if let (Some(a), Some(b)) = (
            kinds.first(),
            kinds
                .iter()
                .find(|k| Some(k.0) != kinds.first().map(|f| f.0)),
        ) {
            return Err(Failure::refuse(
                "mixed-formats",
                format!(
                    "{} is a native memory {} and {} is a {}: their figures are different columns",
                    a.1,
                    if a.0 { "diff" } else { "summary" },
                    b.1,
                    if b.0 { "diff" } else { "summary" },
                ),
                "native memory reports of one kind per command",
                "run one `dla summarize non-heap` for the summaries and one for the diffs",
            ));
        }
        run_snaps(
            ctx,
            opts,
            self.name(),
            snaps,
            vec![
                "native memory categories are whole-runtime figures: they never tie to a finding"
                    .into(),
                "values are in KB as NMT prints them (committed and reserved)".into(),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_patterns_pass_and_values_do_not() {
        for ok in [
            "main",
            "pool-*-thread-*",
            "http-nio-*-exec-*",
            "ForkJoinPool.commonPool-worker-*",
            "Reference Handler",
            "G* Young RemSet",
            "C* CompilerThread*",
            "kafka-producer-network-thread | producer-*",
        ] {
            assert!(plain_pattern(ok), "{ok}");
        }
        for bad in [
            "AKIAIOSFODNN*EXAMPLE*",
            "user@example.com",
            "worker-?",
            "",
            &"x-".repeat(25),
            &"w".repeat(21),
        ] {
            assert!(!plain_pattern(bad), "{bad}");
        }
    }

    #[test]
    fn only_name_groups_are_shaped() {
        let r = Redactor::new(b"salt", false);
        assert_eq!(
            thread_group_key("name:pool-*-thread-*", &r),
            "name:pool-*-thread-*"
        );
        assert_eq!(thread_group_key("state:parked", &r), "state:parked");
        let k = thread_group_key("name:AKIAIOSFODNN*EXAMPLE*", &r);
        assert!(k.starts_with("name:<") && !k.contains("AKIA"), "{k}");
        // The same name gives the same shape, so a series still joins it.
        assert_eq!(k, thread_group_key("name:AKIAIOSFODNN*EXAMPLE*", &r));
        let raw = Redactor::new(b"salt", true);
        assert_eq!(
            thread_group_key("name:AKIAIOSFODNN*EXAMPLE*", &raw),
            "name:AKIAIOSFODNN*EXAMPLE*"
        );
    }

    #[test]
    fn a_diff_is_told_from_a_summary() {
        assert!(nmt_is_diff(
            "Total: reserved=5141478KB +64084KB, committed=326222KB -1416KB\n"
        ));
        assert!(!nmt_is_diff(
            "Total: reserved=5141475KB, committed=326219KB\n-     Java Heap (reserved=1KB, committed=1KB)\n"
        ));
    }
}
