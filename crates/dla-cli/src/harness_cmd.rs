//! `dla harness gen` and `dla harness run`: loop harnesses (spec 4.2 and 4.3).

use std::path::{Path, PathBuf};

use clap::Subcommand;
use dla_core::{FindingId, runtime::Runtime, series::KeyMode};
use dla_workload::harness::{
    capture::{CaptureSet, NativeTool},
    r#gen::{GenOptions, generate},
    run::{RunOptions, run as run_harness},
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
    stage,
};

/// `dla harness ...`
#[derive(Subcommand, Debug)]
pub enum HarnessCmd {
    /// Write a loop harness for a workload into the run directory (runs nothing).
    Gen(GenArgs),
    /// Build and run a harness, writing one series per key mode.
    Run(RunArgs),
}

/// Whether probes are derived from the census.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbesChoice {
    /// From census candidates.
    Auto,
    /// Only the `--probe` findings.
    None,
}

impl std::str::FromStr for ProbesChoice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(Self::Auto),
            "none" => Ok(Self::None),
            other => Err(format!("{other:?} is not auto or none")),
        }
    }
}

/// Flags of `dla harness gen`.
#[derive(clap::Args, Debug)]
pub struct GenArgs {
    /// Runtime: jvm | go | rust | node | python | native.
    #[arg(long, value_parser = parse_enum::<Runtime>)]
    lang: Runtime,
    /// Workload ID from `dla workload discover` (default: the best-ranked main or script
    /// workload of the runtime that takes an iteration count).
    #[arg(long, value_name = "ID")]
    workload: Option<String>,
    /// Driver mode: run in one process. A loop test with hooks to fill in (always, also for a
    /// JUnit 5 test workload; --junit asks for the hookless JUnit repeat instead).
    #[arg(long)]
    driver: bool,
    /// How the workload takes an iteration count, with `{n}`: "--iterations {n}", "-n {n}",
    /// "{n}" (overrides detection).
    #[arg(long, value_name = "TEMPLATE", allow_hyphen_values = true)]
    iter_arg: Option<String>,
    /// Units of work added per point (default: the largest count is about four times the
    /// workload's default; 2 for the JUnit repeat, where a unit is one run of the tests).
    #[arg(long, value_name = "UNITS")]
    step: Option<u64>,
    /// Measured points after the warm-up.
    #[arg(long, default_value_t = 8)]
    iterations: u32,
    /// Warm-up points.
    #[arg(long, default_value_t = 1)]
    warmup: u32,
    /// Add a probe for the holder of this finding and declare it a target of the harness
    /// (repeatable; at most 8 targets with --next). A finding no probe can read is still a
    /// target: its probe is recorded as missing.
    #[arg(long = "probe", value_name = "F", value_parser = parse_enum::<FindingId>)]
    probe: Vec<FindingId>,
    /// Declare the next N pending candidates of the runtime's measurement queue, in rank
    /// order, as targets, exactly as if each were given with --probe (default and maximum 8
    /// in all). The driver lists each target with its writers, removers and what to do.
    /// With --junit (JVM, no --tests): the queue's next test-loop batch, up to 16 candidates
    /// that have a test class, with the test classes chosen for them; nothing to fill.
    #[arg(long, value_name = "N", num_args = 0..=1)]
    next: Option<Option<usize>>,
    /// An entry-path harness: a hook driver whose units call only the project's public entry
    /// points with data a caller controls, for these leak-side findings (comma-separated or
    /// repeated, at most 8, each one that grew in some harness). Its target block gives the
    /// callers chain of each writer function and where the key comes from; write the one line
    /// of VARIES.txt (what a caller varies per unit) before `dla harness run`. Needs --driver;
    /// excludes --probe and --next (refused with the fix).
    #[arg(long, value_name = "F[,F2,..]", value_delimiter = ',', value_parser = parse_enum::<FindingId>)]
    entry: Vec<FindingId>,
    /// Probes derived from the census candidates: auto (default) or none.
    #[arg(long, default_value = "auto", value_parser = parse_enum::<ProbesChoice>)]
    probes: ProbesChoice,
    /// Driver mode: the Go package directory or the Java package to drive.
    #[arg(long, value_name = "DIR|NAME")]
    package: Option<String>,
    /// JVM: run the project's own JUnit 5 tests in a loop in one JVM (a driver with no hooks to
    /// fill). Uses the JUnit test workload of a Gradle or Maven project (`--workload
    /// jvm:test:<file>` to choose). The tests must not need to be told an iteration count.
    #[arg(long)]
    junit: bool,
    /// Python: run the project's own tests with pytest in a loop in one process (a driver with
    /// no hooks to fill and no candidate to name). Without --tests: every test file the workload
    /// discovery lists under the working directory that has the most of them. Needs pytest in the
    /// harness interpreter (`python3` on PATH) and the project importable in place.
    #[arg(long)]
    pytest: bool,
    /// With --junit: the classes or methods to run, `pkg.Class` or `pkg.Class#method`,
    /// comma-separated or repeated (default: the workload's test class). A parameterized test is
    /// `pkg.Class#method(java.lang.String,int)`. With --pytest: test files or directories,
    /// relative to the working directory (node ids with `::` are not supported).
    #[arg(long, value_name = "CLASS[#METHOD],..|PATH,..")]
    tests: Vec<String>,
    /// Rust driver: crate features the driver build enables, comma-separated or repeated
    /// (`cargo build --features a,b`). A target in a module the crate gates behind a feature
    /// says so in its block.
    #[arg(long, value_name = "A[,B,..]", value_delimiter = ',')]
    features: Vec<String>,
    /// Rust driver: build without the crate's default features.
    #[arg(long)]
    no_default_features: bool,
}

/// Splits `--tests` on commas that are not inside the parentheses of a parameter list.
fn split_selectors(text: &str) -> Vec<String> {
    let (mut out, mut depth, mut cur) = (Vec::new(), 0usize, String::new());
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    out.push(cur);
    out.into_iter()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Which key modes `dla harness run` measures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeChoice(Vec<KeyMode>);

impl std::str::FromStr for ModeChoice {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "fresh" => Ok(Self(vec![KeyMode::Fresh])),
            "repeated" => Ok(Self(vec![KeyMode::Repeated])),
            "both" => Ok(Self(vec![KeyMode::Fresh, KeyMode::Repeated])),
            "native" => Ok(Self(vec![KeyMode::Native])),
            other => Err(format!("{other:?} is not fresh, repeated, both or native")),
        }
    }
}

/// Flags of `dla harness run`.
#[derive(clap::Args, Debug)]
pub struct RunArgs {
    /// Harness ID from `dla harness gen` (`H1`).
    #[arg(value_name = "H")]
    harness: String,
    /// Key modes: fresh | repeated | both (driver mode) or native (command mode); default:
    /// the manifest's.
    #[arg(long, value_parser = parse_enum::<ModeChoice>)]
    mode: Option<ModeChoice>,
    /// Measured points after the warm-up, overriding the manifest.
    #[arg(long, value_name = "N")]
    iterations: Option<u32>,
    /// Extra points after the measured ones (the plateau window of `dla alloc-test`).
    #[arg(long, value_name = "K", default_value_t = 0)]
    extra: u32,
    /// Purge applied at the last point: trim | gc | free-os-memory | trim-native-heap |
    /// jemalloc | empty-cache | none.
    #[arg(long, value_name = "KIND")]
    purge: Option<String>,
    /// Also write raw captures the `dla summarize` commands read, a comma list of kinds;
    /// `all` for every kind the runtime's agent writes. JVM: heap (HPROF dumps), histogram
    /// (class histogram per point), jfr (old-object samples), nonheap (thread dump, class
    /// loaders, native memory). Node: heap (V8 heap snapshots), alloc (sampling heap
    /// profiles), nonheap (resources by creation site). Go: alloc (heap profiles), nonheap
    /// (goroutine dumps and the goroutine leak profile). Python: histogram (type counts),
    /// alloc (tracemalloc, and memray when the program's interpreter has it), nonheap
    /// (threads, tasks, ResourceWarning sites). A series of heap dumps or snapshots needs
    /// one process (driver mode). Default: none.
    #[arg(long, value_name = "KINDS", value_parser = str::parse::<CaptureSet>)]
    capture: Option<CaptureSet>,
    /// Heap dumps per run with --capture heap (driver mode: spread over the rounds;
    /// command mode: the number of points that dump). Default 4.
    #[arg(long, value_name = "N", requires = "capture")]
    dumps: Option<u32>,
    /// Rust and native command-mode harnesses: run three of the points under valgrind
    /// (memcheck XML) or heaptrack (heaptrack_print report and a first-to-last diff) instead
    /// of measuring a series.
    #[arg(long, value_name = "TOOL", value_parser = str::parse::<NativeTool>, conflicts_with = "capture")]
    tool: Option<NativeTool>,
}

/// Runs `dla harness`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: HarnessCmd) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    match a {
        HarnessCmd::Gen(g) => {
            let lang = g.lang;
            let opts = GenOptions {
                lang,
                workload: g.workload,
                driver: g.driver,
                iter_arg: g.iter_arg,
                step: g.step,
                iterations: g.iterations,
                warmup: g.warmup,
                // A bare `--next` fills the harness up to the limit of targets.
                next: g.next.map(|n| {
                    n.unwrap_or_else(|| {
                        if g.junit {
                            // The test loop's batch: as many as the queue planned.
                            dla_core::testloop::MAX_TARGETS
                        } else {
                            dla_core::harness::MAX_TARGETS.saturating_sub(g.probe.len())
                        }
                    })
                }),
                probe_findings: g.probe,
                entry: g.entry,
                auto_probes: g.probes == ProbesChoice::Auto,
                package: g.package,
                junit: g.junit,
                pytest: g.pytest,
                tests: g.tests.iter().flat_map(|t| split_selectors(t)).collect(),
                features: g.features,
                no_default_features: g.no_default_features,
            };
            stage::finish(
                "harness gen",
                json!({"lang": lang.as_str()}),
                generate(&tc, &opts),
            )
        }
        HarnessCmd::Run(r) => {
            if !r.harness.starts_with('H') {
                return Err(Failure::usage(
                    format!("{:?} is not a harness ID", r.harness),
                    "a harness ID such as H1",
                    "dla harness run H1",
                ));
            }
            let opts = RunOptions {
                modes: r.mode.map(|m| m.0).unwrap_or_default(),
                iterations: r.iterations,
                extra: r.extra,
                purge: r.purge,
                capture: r.capture.unwrap_or_default(),
                capture_dumps: r.dumps,
                tool: r.tool,
                ..RunOptions::default()
            };
            let signal = if r.tool.is_some() {
                "one report per point (valgrind XML or a heaptrack_print text), no series"
            } else if r.capture.is_some_and(|c| !c.is_empty()) {
                "one series per key mode, plus the requested raw captures (heap dumps or snapshots, histograms, allocation profiles, non-heap reports) under out/"
            } else {
                "one series per key mode: memory, threads, handles and probe sizes per point"
            };
            let effects = json!({
                "target": format!("the workload of harness {} in its scratch copy under the run directory", r.harness),
                "expected_signal": signal,
                "stopping": "after the last point, or a point's timeout (the process group is killed)",
                "effects": "builds and runs project code in the scratch copy with DLA's agent loaded; the project source is not edited",
                "cleanup": "every process is registered in state.json and stopped; the scratch copy stays for inspection",
            });
            stage::finish(
                "harness run",
                json!({"harness": r.harness, "effects": effects}),
                run_harness(&tc, &r.harness, &opts),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::split_selectors;

    #[test]
    fn tests_flag_splits_on_commas_outside_parameter_lists() {
        assert_eq!(
            split_selectors("a.B, a.C#p(java.lang.String,int),a.D#m"),
            ["a.B", "a.C#p(java.lang.String,int)", "a.D#m"]
        );
        assert!(split_selectors(" , ").is_empty());
    }
}
