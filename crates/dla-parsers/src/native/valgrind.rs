//! Valgrind memcheck XML (`--leak-check=full --show-reachable=yes --xml=yes`).
//!
//! Leak records (`Leak_DefinitelyLost`, `Leak_IndirectlyLost`, `Leak_PossiblyLost`,
//! `Leak_StillReachable`) become [`AllocGroup`]s with the leak kind, bytes, blocks and the
//! stack. Other errors (invalid reads and so on) are listed by kind. The leading frames that
//! belong to Valgrind's own malloc replacement (`vgpreload_*`) are dropped.
//!
//! Leak kinds are `definitely-lost`, `indirectly-lost`, `possibly-lost` and `still-reachable`.
//! A document that ends early (the process was killed) is parsed as far as it goes and the
//! report says it is not complete.

use serde::{Deserialize, Serialize};

use super::ParseError;
use super::frames::make_frame;
use super::xml::{self, Node};
use crate::model::{AllocGroup, AllocProfile, Frame};

/// A parsed memcheck run.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ValgrindReport {
    /// Leak records as allocation groups, in Valgrind's loss record order.
    pub profile: AllocProfile,
    /// The command line of the program (the `Command:` preamble line), when present.
    pub command: Option<String>,
    pub pid: Option<u32>,
    /// Kinds of the non-leak errors, e.g. `InvalidRead`, in order of appearance.
    pub other_errors: Vec<String>,
    /// Whether the run reported `FINISHED` and the document is whole.
    pub complete: bool,
}

/// Parses a memcheck XML document.
pub fn parse(input: &str) -> Result<ValgrindReport, ParseError> {
    let doc = xml::parse(input)?;
    let Some(root) = doc.root.child("valgrindoutput") else {
        return Err(ParseError::new(
            "not a Valgrind XML document: no <valgrindoutput> element (run valgrind with --xml=yes --xml-file=<file>)",
        ));
    };
    if let Some(tool) = root
        .child_text("tool")
        .or_else(|| root.child_text("protocoltool"))
        && tool != "memcheck"
    {
        return Err(ParseError::new(format!(
            "Valgrind tool is {tool}, expected memcheck (leak records come from --tool=memcheck)"
        )));
    }
    let mut report = ValgrindReport {
        profile: AllocProfile {
            source: "valgrind".to_owned(),
            ..AllocProfile::default()
        },
        pid: root.child_text("pid").and_then(|p| p.parse().ok()),
        command: root.child("preamble").and_then(|p| {
            p.children_named("line")
                .find_map(|l| l.text.strip_prefix("Command:").map(|c| c.trim().to_owned()))
        }),
        ..ValgrindReport::default()
    };
    let finished = root
        .children_named("status")
        .any(|s| s.child_text("state") == Some("FINISHED"));
    report.complete = finished && !doc.truncated;

    for error in root.children_named("error") {
        let Some(kind) = error.child_text("kind") else {
            continue;
        };
        match leak_kind(kind) {
            // A record cut off before its figures were written is not a record.
            Some(_) if doc.truncated && !has_figures(error) => {}
            Some(label) => report.profile.groups.push(leak_group(error, label)),
            None => report.other_errors.push(kind.to_owned()),
        }
    }
    Ok(report)
}

fn has_figures(error: &Node) -> bool {
    error
        .child("xwhat")
        .is_some_and(|w| w.child("leakedbytes").is_some())
}

fn leak_kind(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "Leak_DefinitelyLost" => "definitely-lost",
        "Leak_IndirectlyLost" => "indirectly-lost",
        "Leak_PossiblyLost" => "possibly-lost",
        "Leak_StillReachable" => "still-reachable",
        _ => return None,
    })
}

fn leak_group(error: &Node, kind: &str) -> AllocGroup {
    let what = error.child("xwhat").or_else(|| error.child("what"));
    let number = |name: &str| {
        what.and_then(|w| w.child_text(name))
            .and_then(|t| t.parse::<u64>().ok())
            .unwrap_or(0)
    };
    let mut frames: Vec<Frame> = error
        .child("stack")
        .map(|s| s.children_named("frame").map(frame).collect())
        .unwrap_or_default();
    let skip = frames
        .iter()
        .take_while(|f| {
            f.module
                .as_deref()
                .is_some_and(|m| m.contains("/vgpreload_"))
        })
        .count();
    if skip < frames.len() {
        frames.drain(..skip);
    }
    AllocGroup {
        frames,
        live_bytes: number("leakedbytes"),
        live_blocks: number("leakedblocks"),
        total_bytes: None,
        root_path: Vec::new(),
        kind: Some(kind.to_owned()),
    }
}

fn frame(node: &Node) -> Frame {
    let function = node
        .child_text("fn")
        .or_else(|| node.child_text("ip"))
        .unwrap_or("???");
    let file = node
        .child_text("file")
        .map(|f| match node.child_text("dir") {
            Some(dir) if !f.starts_with('/') => format!("{}/{f}", dir.trim_end_matches('/')),
            _ => f.to_owned(),
        });
    let line = node
        .child_text("line")
        .and_then(|l| l.parse::<u32>().ok())
        .filter(|&l| l > 0);
    make_frame(
        function,
        file,
        line,
        node.child_text("obj").map(str::to_owned),
    )
}
