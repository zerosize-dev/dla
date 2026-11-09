//! `alloc-stack-diff` over the Python harness's tracemalloc files (`py-<n>.alloc.json`,
//! `dla harness run --capture alloc`).
//!
//! The agent writes, at each heavy checkpoint, the live bytes and blocks per allocation
//! stack (a snapshot after a full collection, stacks innermost frame first). The files are
//! JSON the agent wrote itself; no pickle and no tracemalloc dump is ever read. A stack
//! ties to the finding at its first project frame (or among that finding's writers), the
//! tie rule's site identity: tracemalloc names no holder.

use std::path::Path;

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
    series::SeriesRegion,
};
use dla_parsers::model::{AllocGroup, AllocProfile, Frame};
use serde::Deserialize;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest file read.
const MAX_BYTES: u64 = 32 << 20;
/// Most stacks read from one file.
const MAX_STACKS: usize = 2_000;
/// Most frames read from one stack.
const MAX_FRAMES: usize = 32;

/// The tracemalloc file reader.
pub struct PyTracemalloc;

#[derive(Deserialize, Default)]
struct AllocFile {
    #[serde(default)]
    dla_pyalloc: Option<u32>,
    #[serde(default)]
    stacks: Vec<Stack>,
}

#[derive(Deserialize, Default)]
struct Stack {
    #[serde(default)]
    size: u64,
    #[serde(default)]
    count: u64,
    /// `[file, line]`, innermost first.
    #[serde(default)]
    frames: Vec<(String, u32)>,
}

/// True when `file` is a path that names no project file by itself: absolute, or a
/// pseudo file such as `<frozen runpy>`.
fn is_anchored(file: &str) -> bool {
    file.starts_with('/')
        || file.starts_with('<')
        || file.get(1..3) == Some(":/")
        || file.starts_with("//")
}

/// A frame of a Python stack. A file at the root of the project has no `/` in it, which
/// the project check cannot tell from a bare name, so it is anchored at the project; every
/// other relative path is kept as the agent wrote it.
pub(super) fn frame(project: &Path, function: &str, file: &str, line: u32) -> Frame {
    let file = file.replace('\\', "/");
    let shown = if is_anchored(&file) || file.contains('/') {
        file
    } else {
        project.join(&file).to_string_lossy().replace('\\', "/")
    };
    let module = module_of(&shown);
    Frame {
        function: if function.is_empty() {
            module
        } else if module.is_empty() || module.starts_with('<') {
            function.to_owned()
        } else {
            format!("{module}.{function}")
        },
        file: Some(shown),
        line: Some(line),
        module: None,
    }
}

/// `ingest.registry` for `ingest/registry.py`: the module a tracemalloc frame is in (the
/// agent has the file and the line only).
fn module_of(file: &str) -> String {
    if is_anchored(file) {
        return file
            .rsplit('/')
            .next()
            .unwrap_or(file)
            .trim_end_matches(".py")
            .to_owned();
    }
    file.trim_end_matches(".py")
        .trim_end_matches("/__init__")
        .replace('/', ".")
}

fn parse(text: &str, shown: &str, project: &Path) -> Res<AllocProfile> {
    let bad = |why: String| {
        Failure::refuse(
            "malformed-artifact",
            format!("{shown}: {why}"),
            "py-<n>.alloc.json written by the Python harness",
            "dla harness run <H> --capture alloc",
        )
    };
    let f: AllocFile = serde_json::from_str(text).map_err(|e| bad(e.to_string()))?;
    if f.dla_pyalloc.is_none() {
        return Err(bad("no `dla_pyalloc` version".to_owned()));
    }
    let groups = f
        .stacks
        .into_iter()
        .take(MAX_STACKS)
        .map(|s| AllocGroup {
            frames: s
                .frames
                .iter()
                .take(MAX_FRAMES)
                .map(|(file, line)| frame(project, "", file, *line))
                .collect(),
            live_bytes: s.size,
            live_blocks: s.count,
            total_bytes: None,
            root_path: Vec::new(),
            kind: None,
        })
        .collect();
    Ok(AllocProfile {
        source: "py-tracemalloc".to_owned(),
        // Times are relative to each process start: the files are ordered as given.
        timestamp_ms: None,
        sampled: false,
        groups,
    })
}

impl Adapter for PyTracemalloc {
    fn name(&self) -> &'static str {
        "py-tracemalloc"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "python"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        "Python harness tracemalloc file (py-<n>.alloc.json, {\"dla_pyalloc\":1, \"stacks\"})"
    }
    fn sniff(&self, input: &Input) -> bool {
        input.head_text().contains("\"dla_pyalloc\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            captures.push(ProfileCapture {
                label: i.shown().to_owned(),
                profile: parse(&text, i.shown(), &ctx.tc.project)?,
                sha256: i.sha256.clone(),
            });
        }
        alloc_stack::run_diff(
            ctx,
            opts,
            captures,
            &ProfileOrigin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Live,
                runtime: Some(Runtime::Python),
                region: SeriesRegion::Heap,
                order_by_time: false,
                windowed: false,
                limitations: vec![
                    "tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it".into(),
                    "each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file".into(),
                    "stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame".into(),
                    "the files are ordered as given".into(),
                ],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_become_groups_innermost_first() {
        let text = r#"{"dla_pyalloc":1,"stacks":[
            {"size":4096,"count":2,"frames":[["ingest/registry.py",7],["ingest/pipeline.py",28],["<frozen runpy>",88]]},
            {"size":10,"count":1,"frames":[["leak.py",3]]}]}"#;
        let p = parse(text, "x", Path::new("/proj")).unwrap();
        assert_eq!(p.groups.len(), 2);
        let g = &p.groups[0];
        assert_eq!((g.live_bytes, g.live_blocks), (4096, 2));
        assert_eq!(g.frames[0].file.as_deref(), Some("ingest/registry.py"));
        assert_eq!(g.frames[0].function, "ingest.registry");
        assert_eq!(g.frames[0].line, Some(7));
        assert_eq!(g.frames[2].file.as_deref(), Some("<frozen runpy>"));
        // A file at the project root is anchored so the project check can find it.
        assert_eq!(p.groups[1].frames[0].file.as_deref(), Some("/proj/leak.py"));
        assert_eq!(p.groups[1].frames[0].function, "leak");
    }

    #[test]
    fn a_file_without_the_version_is_refused() {
        let e = parse(r#"{"stacks":[]}"#, "x", Path::new("/p"))
            .err()
            .unwrap();
        assert!(format!("{e:?}").contains("dla_pyalloc"));
    }

    #[test]
    fn module_names_follow_the_path() {
        assert_eq!(module_of("ingest/__init__.py"), "ingest");
        assert_eq!(module_of("a/b/c.py"), "a.b.c");
        assert_eq!(module_of("/usr/lib/python3/random.py"), "random");
    }
}
