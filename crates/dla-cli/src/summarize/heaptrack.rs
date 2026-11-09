//! `alloc-stack-diff` over heaptrack: `heaptrack_print` text (the leaks section, with `-l`),
//! its `-d` diff text, `-F` folded stacks with exact bytes (cost type leaked, named so), and a raw `heaptrack.*.zst` or
//! `.gz` recording, which this adapter prints through `heaptrack_print` first.
//!
//! One report is the leak view of one run. Three or more reports of runs with growing
//! inputs give a live-bytes series per stack. A stack ties only by its first project frame
//! (rule 2 of the tie rule): the finding's own site or one of its recorded writers.
//!
//! The reader of text and folded files needs no tool, so the adapter is always available.
//! A raw recording needs `heaptrack_print`: without it the run is refused with
//! `tool-missing` and a `missing` coverage row named after the tool.

use std::{ffi::OsString, time::Duration};

use dla_core::{
    coverage::BackendCoverage,
    evidence::{Authority, tool_class},
};
use dla_parsers::{
    model::AllocProfile,
    native::heaptrack::{self, FoldedCost},
};

use super::{
    Adapter, AdapterOut, CoverageRow, Ctx, Op, Opts, ToolReq,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
    valgrind::Rebase,
    write_coverage,
};
use crate::fail::{Failure, Res};

/// Largest text file read.
const MAX_BYTES: u64 = 64 << 20;
/// Longest `heaptrack_print` run.
const PRINT_TIMEOUT: Duration = Duration::from_secs(300);

/// The heaptrack reader.
pub struct Heaptrack;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Text,
    Folded,
    Raw,
}

fn kind_of(input: &Input) -> Option<Kind> {
    let head = input.head_text();
    if head.contains("MOST CALLS TO ALLOCATION FUNCTIONS")
        || head.contains("finished reading file, now analyzing data")
        || head.contains("Debuggee command was:")
    {
        return Some(Kind::Text);
    }
    let name = input.name_lower();
    let zst = input.head.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]);
    let gz = input.head.starts_with(&[0x1f, 0x8b]);
    if (zst || gz) && name.contains("heaptrack") {
        return Some(Kind::Raw);
    }
    let folded_name = name.ends_with(".folded") || name.ends_with(".fold");
    if folded_name || looks_folded(&head) {
        return Some(Kind::Folded);
    }
    None
}

/// True when the file name says the folded stacks were written with the `leaked` cost
/// type. The format has no marker of its own; the name is the person's statement of it.
fn leaked_name(input: &Input) -> bool {
    input.name_lower().contains("leaked")
}

/// Lines of `frame;frame;frame <number>` and nothing else.
fn looks_folded(head: &str) -> bool {
    let lines: Vec<&str> = head.lines().filter(|l| !l.trim().is_empty()).collect();
    // The last line of the head may be cut.
    let lines = &lines[..lines.len().saturating_sub(1)];
    !lines.is_empty()
        && lines.iter().all(|l| {
            l.contains(';')
                && !l.starts_with(['{', '<', '#'])
                && l.rsplit_once(' ')
                    .is_some_and(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
}

impl Heaptrack {
    fn raw_to_text(&self, ctx: &Ctx<'_>, input: &Input) -> Res<String> {
        let req = ToolReq {
            binary: "heaptrack_print",
            install_hint: "",
        };
        let args: Vec<OsString> = [
            OsString::from("-f"),
            input.abs.clone().into_os_string(),
            "-l".into(),
            "-p0".into(),
            "-a0".into(),
            "-T0".into(),
            "-n".into(),
            "200".into(),
            // No merge of backtraces and room for every sub-stack: merged, heaptrack_print
            // lists the five biggest and elides the rest ("and N from M other places").
            "-m0".into(),
            "-s".into(),
            "50".into(),
        ]
        .into();
        let out_name = format!("heaptrack-{}.txt", &input.sha256[..12]);
        let run = match ctx.run_tool(&req, &args, &out_name, PRINT_TIMEOUT) {
            Ok(r) => r,
            Err(f) => {
                if f.refusal.code == super::TOOL_MISSING {
                    let row = CoverageRow {
                        worker: "native".to_owned(),
                        backend: "heaptrack_print".to_owned(),
                        cov: BackendCoverage::missing(
                            f.refusal.message.clone(),
                            f.refusal.fix.clone(),
                        ),
                    };
                    write_coverage(ctx.tc, vec![row])?;
                }
                return Err(f);
            }
        };
        if !run.ok {
            return Err(Failure::refuse(
                "malformed-artifact",
                format!(
                    "heaptrack_print could not read {}: {}",
                    input.shown(),
                    if run.timed_out {
                        "it timed out".to_owned()
                    } else {
                        run.stderr_tail.clone()
                    }
                ),
                "a complete heaptrack recording (heaptrack.<program>.<pid>.zst or .gz)",
                "record again with heaptrack <program> and let the program exit",
            ));
        }
        let text = std::fs::read_to_string(&run.out)
            .map_err(|e| Failure::from(dla_core::Error::io(&run.out, e)))?;
        Ok(text)
    }
}

impl Adapter for Heaptrack {
    fn name(&self) -> &'static str {
        "heaptrack"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "native"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        "heaptrack_print text with -l (or its -d diff), -F folded stacks written with --flamegraph-cost-type leaked and named *leaked*, or a raw heaptrack.*.zst/.gz recording (needs heaptrack_print)"
    }
    fn sniff(&self, input: &Input) -> bool {
        kind_of(input).is_some()
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let kinds: Vec<Kind> = inputs.iter().filter_map(kind_of).collect();
        if kinds.windows(2).any(|w| w[0] != w[1]) {
            return Err(Failure::refuse(
                "mixed-formats",
                "the inputs mix heaptrack text, folded stacks and raw recordings".to_owned(),
                "inputs of one heaptrack format per command",
                "run one `dla summarize alloc-stack-diff <files>` per format",
            ));
        }
        let rebase = Rebase::new(&ctx.tc.project);
        let mut captures = Vec::new();
        let mut limitations = Vec::new();
        let mut diffs: Vec<bool> = Vec::new();
        let mut raw_printed = 0u64;
        for i in inputs {
            let kind = kinds.first().copied().unwrap_or(Kind::Text);
            let mut profile: AllocProfile = match kind {
                Kind::Folded => {
                    if !leaked_name(i) {
                        return Err(Failure::refuse(
                            "unsupported-format",
                            format!(
                                "{} is a folded stack file whose cost type is not known: heaptrack_print writes allocation counts by default, and counts are churn, not retained bytes",
                                i.shown()
                            ),
                            "a folded file written with --flamegraph-cost-type leaked, named so (the name holds \"leaked\")",
                            "heaptrack_print -f <recording> -F leaked.folded --flamegraph-cost-type leaked",
                        ));
                    }
                    let text = i.read_text(MAX_BYTES)?;
                    heaptrack::parse_folded(&text, FoldedCost::Bytes).map_err(|e| {
                        Failure::refuse(
                            "malformed-artifact",
                            format!("{} is not a folded stack file: {}", i.shown(), e.message()),
                            "heaptrack_print -F <file> --flamegraph-cost-type leaked output",
                            "heaptrack_print -f <recording> -F out.folded --flamegraph-cost-type leaked",
                        )
                    })?
                }
                Kind::Text | Kind::Raw => {
                    let text = if kind == Kind::Raw {
                        raw_printed += 1;
                        self.raw_to_text(ctx, i)?
                    } else {
                        i.read_text(MAX_BYTES)?
                    };
                    if !text.contains("MEMORY LEAKS") {
                        limitations.push(format!(
                            "{} has no MEMORY LEAKS section: it was printed without -l, so it shows no leaks",
                            i.shown()
                        ));
                    }
                    let elided = text
                        .lines()
                        .filter(|l| {
                            let t = l.trim();
                            t.starts_with("and ") && t.ends_with("other places")
                        })
                        .count();
                    if elided > 0 {
                        limitations.push(format!(
                            "{} elides {elided} group(s) of stacks (\"and N from M other places\"): their bytes are not read, so a growing site can be missing; print with -m0 -s 50",
                            i.shown()
                        ));
                    }
                    let report = heaptrack::parse(&text).map_err(|e| {
                        Failure::refuse(
                            "malformed-artifact",
                            format!(
                                "{} is not heaptrack_print output: {}",
                                i.shown(),
                                e.message()
                            ),
                            "heaptrack_print output that ran to its end",
                            "heaptrack_print -f <recording> -l -p0 -a0 -T0 -n 200 -m0 -s 50 > heaptrack.txt",
                        )
                    })?;
                    diffs.push(report.is_diff);
                    report.leak_profile()
                }
            };
            rebase.profile(&mut profile);
            captures.push(ProfileCapture {
                label: i.shown().to_owned(),
                profile,
                sha256: i.sha256.clone(),
            });
        }
        if diffs.iter().any(|d| *d) && !diffs.iter().all(|d| *d) {
            return Err(Failure::refuse(
                "mixed-formats",
                "the inputs mix heaptrack_print reports and a -d diff, whose figures are changes between two runs".to_owned(),
                "reports of runs, or diffs, one kind per command",
                "run one `dla summarize alloc-stack-diff <files>` for the reports and another for the diff",
            ));
        }
        let diff = diffs.iter().any(|d| *d);
        limitations.push(
            "figures are the bytes heaptrack counts as leaked: still allocated when the program exits".into(),
        );
        if kinds.first() == Some(&Kind::Folded) {
            limitations.push(
                "folded stacks carry no line numbers and no cost-type marker: the file name says they are leaked bytes, and no frame has a site line".into(),
            );
        }
        if kinds.first() == Some(&Kind::Text) || kinds.first() == Some(&Kind::Raw) {
            limitations.push(
                "heaptrack_print rounds sizes (3.02M): figures are accurate to about 1%; -F folded files carry exact bytes".into(),
            );
        }
        if diff {
            limitations.push(
                "a -d diff lists the growth between two runs, so its figures are changes, not totals"
                    .into(),
            );
        }
        let mut out = alloc_stack::run_diff(
            ctx,
            opts,
            captures,
            &ProfileOrigin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Live,
                runtime: None,
                region: dla_core::series::SeriesRegion::OffHeap,
                order_by_time: false,
                windowed: false,
                limitations,
            },
        )?;
        if raw_printed > 0 {
            out.coverage.push(CoverageRow {
                worker: "native".to_owned(),
                backend: "heaptrack_print".to_owned(),
                cov: BackendCoverage::ran(None, raw_printed),
            });
        }
        Ok(out)
    }
}
