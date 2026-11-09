//! `alloc-stack-diff` over JFR `jdk.OldObjectSample` events.
//!
//! Two adapters share one reader:
//!
//! * `jfr-old-object-text` reads the text `jfr print` wrote. It needs no tool.
//! * `jfr-old-object-sample` reads a `.jfr` recording (or the same text) and runs the
//!   `jfr` tool itself: `jfr print --stack-depth 64 --events jdk.OldObjectSample <file>`
//!   with English number formats. The output goes to `tmp/<worker>/` and is removed
//!   after it is parsed.
//!
//! Each event is one sampled old object with its allocation stack and, when the recording
//! used `path-to-gc-roots=true`, the reference chain to a GC root. The parser
//! (`dla_parsers::jvm::jfr`) groups them by (allocation stack, root path, type) and puts
//! the holder key (`static:<Class>.<field>`, the form of the HPROF reader) first in the
//! root path. The alloc-stack engine does the rest, under the tie rule: a stack ties by its
//! holder only, and its first project frame becomes a writer only for a holder tie.
//!
//! The engine follows each root holder's total (a `holder:<key>` column) as well as the
//! single stacks. OldObjectSample samples old objects; it is not a census. A count series over several
//! recordings is weak evidence, and every result says `sampled: true`.

use std::{ffi::OsString, fs, time::Duration};

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
    series::SeriesRegion,
};
use dla_parsers::{jvm::jfr, model::AllocProfile};

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts, ToolReq,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest printed text read, bytes.
const MAX_TEXT: u64 = 512 << 20;
/// Longest the `jfr` tool may run per recording.
const TOOL_TIMEOUT: Duration = Duration::from_secs(300);
/// Stack frames `jfr print` keeps (its default of 5 cuts the allocation site off).
const STACK_DEPTH: &str = "64";
/// First bytes of a JFR recording: `FLR` and a zero byte.
const MAGIC: &[u8] = b"FLR\0";
/// The line that starts an event in the printed text.
const EVENT_LINE: &str = "jdk.OldObjectSample {";

const FORMATS: &str = "JFR recording (.jfr) with jdk.OldObjectSample events, or the text of `jfr print --events jdk.OldObjectSample`";

/// True for a binary recording.
fn is_recording(input: &Input) -> bool {
    input.head.starts_with(MAGIC)
}

/// True for the printed text of OldObjectSample events.
fn is_printed(input: &Input) -> bool {
    !is_recording(input) && input.head_text().contains(EVENT_LINE)
}

/// Runs `jfr print` over a recording and returns the text.
fn print_recording(ctx: &Ctx<'_>, req: &ToolReq, input: &Input) -> Res<String> {
    let args: Vec<OsString> = [
        "-J-Duser.language=en",
        "-J-Duser.country=US",
        "print",
        "--stack-depth",
        STACK_DEPTH,
        "--events",
        "jdk.OldObjectSample",
    ]
    .iter()
    .map(OsString::from)
    .chain(std::iter::once(input.abs.as_os_str().to_owned()))
    .collect();
    let out_name = format!("jfr-{}.txt", &input.sha256[..16]);
    let run = ctx.run_tool(req, &args, &out_name, TOOL_TIMEOUT)?;
    let text = if run.ok {
        fs::read_to_string(&run.out)
            .or_else(|_| fs::read(&run.out).map(|b| String::from_utf8_lossy(&b).into_owned()))
            .ok()
    } else {
        None
    };
    // The tool's output can be large and holds names from the program: it is parsed and
    // removed, only the summary stays.
    let _ = fs::remove_file(&run.out);
    match text {
        Some(t) => Ok(t),
        None if run.timed_out => Err(Failure::refuse(
            "tool-failed",
            format!(
                "jfr print did not finish within {} s on {}",
                TOOL_TIMEOUT.as_secs(),
                input.shown()
            ),
            "a recording that jfr can print in time",
            "take a shorter recording, or run `jfr print --events jdk.OldObjectSample` yourself and pass the text",
        )),
        None => Err(Failure::refuse(
            "malformed-artifact",
            format!(
                "jfr could not print {} (exit {}): {}",
                input.shown(),
                run.code
                    .map_or_else(|| "none".to_owned(), |c| c.to_string()),
                run.stderr_tail.chars().take(200).collect::<String>()
            ),
            "a complete JFR recording",
            "let the JVM finish writing it (dumponexit=true), or take another with jcmd <pid> JFR.dump",
        )),
    }
}

fn parse_text(input: &Input, text: &str) -> Res<AllocProfile> {
    jfr::parse(text).map_err(|e| {
        Failure::refuse(
            "malformed-artifact",
            format!("{}: {}", input.shown(), e.message()),
            "complete `jfr print --events jdk.OldObjectSample` output",
            "jfr print --stack-depth 64 --events jdk.OldObjectSample <recording.jfr>",
        )
    })
}

fn run_inputs(
    ctx: &Ctx<'_>,
    inputs: &[Input],
    opts: &Opts,
    name: &'static str,
    tool: Option<&ToolReq>,
) -> Res<AdapterOut> {
    let mut captures = Vec::new();
    for i in inputs {
        let text = if is_recording(i) {
            let Some(req) = tool else {
                return Err(Failure::refuse(
                    "unsupported-format",
                    format!(
                        "{} is a binary recording, which {name} does not read",
                        i.shown()
                    ),
                    "the text of `jfr print`, or the jfr-old-object-sample backend",
                    "dla summarize alloc-stack-diff --backend jfr-old-object-sample <recording.jfr>",
                ));
            };
            print_recording(ctx, req, i)?
        } else {
            i.read_text(MAX_TEXT)?
        };
        let profile = parse_text(i, &text)?;
        captures.push(ProfileCapture {
            label: i.shown().to_owned(),
            profile,
            sha256: i.sha256.clone(),
        });
        ctx.tc.require_time("dla summarize alloc-stack-diff")?;
    }
    alloc_stack::run_diff_with(
        ctx,
        opts,
        captures,
        &ProfileOrigin {
            tool: name,
            class: tool_class::ALLOC_PROFILER,
            authority: Authority::Live,
            runtime: Some(Runtime::Jvm),
            region: SeriesRegion::Heap,
            // The printed time has no zone, so the recordings stay in the given order.
            order_by_time: false,
            limitations: vec![
                "OldObjectSample samples old objects that are still live; it is not a census, so counts and bytes are estimates and a count series over recordings is weak".into(),
                "holders need path-to-gc-roots=true and the recording started before the leak".into(),
                "the recordings are ordered as given".into(),
            ],
        },
        // The samples of one holder are followed as a total: single stacks come and go.
        true,
    )
}

/// The reader of printed OldObjectSample text; no tool needed.
pub struct JfrOldObjectText;

impl Adapter for JfrOldObjectText {
    fn name(&self) -> &'static str {
        "jfr-old-object-text"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "jvm"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        "text of `jfr print --events jdk.OldObjectSample` (starts events with `jdk.OldObjectSample {`)"
    }
    fn sniff(&self, input: &Input) -> bool {
        is_printed(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        run_inputs(ctx, inputs, opts, self.name(), None)
    }
}

/// The recording reader: runs `jfr print` itself.
pub struct JfrOldObjectSample;

impl JfrOldObjectSample {
    const TOOL: ToolReq = ToolReq {
        binary: "jfr",
        install_hint: "",
    };
}

impl Adapter for JfrOldObjectSample {
    fn name(&self) -> &'static str {
        "jfr-old-object-sample"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "jvm"
    }
    fn tool(&self) -> Option<ToolReq> {
        Some(Self::TOOL)
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        FORMATS
    }
    fn sniff(&self, input: &Input) -> bool {
        is_recording(input) || is_printed(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        run_inputs(ctx, inputs, opts, self.name(), Some(&Self::TOOL))
    }
}
