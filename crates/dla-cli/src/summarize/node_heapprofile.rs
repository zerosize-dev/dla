//! `alloc-stack-diff` over V8 sampling heap profiles (`.heapprofile`, from `--heap-prof`
//! or the inspector's `HeapProfiler.stopSampling`), read with DLA's own reader.
//!
//! A sample charged to a native builtin (`Array.push`, `repeat`) is attributed to the
//! nearest JavaScript frame. When a script has a source map next to it (`<script>.map`),
//! the frame's generated line and column are mapped to the original file and line, so a
//! bundle shows the source the code was written in; the output says how many frames were
//! mapped.
//!
//! A profile covers the objects allocated in its window that are still alive when it is
//! read. The profiles of a driver run are windows, not cumulative: a stack that keeps
//! more live bytes in every window is a signal, and the heap-graph series is the growth
//! check.

use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
    series::SeriesRegion,
};
use dla_parsers::node::{heapprofile, sourcemap::SourceMap};
use serde_json::json;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest profile file read.
const MAX_BYTES: u64 = 256 << 20;
/// Largest source map file read.
const MAP_BYTES: u64 = 32 << 20;
/// Most segments of a source map decoded.
const MAP_SEGMENTS: usize = 2_000_000;

/// The V8 heap profile reader.
pub struct NodeHeapProfile;

/// Removes `.` and `..` components without touching the disk.
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            Component::CurDir => {}
            c => out.push(c.as_os_str()),
        }
    }
    out
}

/// Source maps by script path, loaded once.
#[derive(Default)]
struct Maps {
    by_script: RefCell<BTreeMap<String, Option<SourceMap>>>,
    mapped: Cell<u64>,
}

impl Maps {
    fn lookup(&self, script: &str, line: u32, col: u32) -> Option<(String, u32)> {
        let mut by = self.by_script.borrow_mut();
        let map = by.entry(script.to_owned()).or_insert_with(|| {
            let p = format!("{script}.map");
            let meta = fs::metadata(&p).ok()?;
            if !meta.is_file() || meta.len() > MAP_BYTES {
                return None;
            }
            SourceMap::parse(&fs::read_to_string(&p).ok()?, MAP_SEGMENTS).ok()
        });
        let o = map.as_ref()?.lookup(line, col)?;
        let dir = Path::new(script).parent().unwrap_or(Path::new(""));
        let file = if Path::new(o.source).is_absolute() {
            PathBuf::from(o.source)
        } else {
            normalize(&dir.join(o.source))
        };
        self.mapped.set(self.mapped.get() + 1);
        Some((file.to_string_lossy().replace('\\', "/"), o.line + 1))
    }
}

impl Adapter for NodeHeapProfile {
    fn name(&self) -> &'static str {
        "node-heapprofile"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "node"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_PROFILER
    }
    fn formats(&self) -> &'static str {
        "V8 sampling heap profile (.heapprofile, from --heap-prof or HeapProfiler.stopSampling)"
    }
    fn sniff(&self, input: &Input) -> bool {
        input.has_suffix(".heapprofile") || heapprofile::sniff(&input.head)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let maps = Maps::default();
        let mut captures = Vec::new();
        for i in inputs {
            if i.size > MAX_BYTES {
                return Err(Failure::refuse(
                    "insufficient-resources",
                    format!(
                        "{} is {} MB, over the {} MB limit",
                        i.shown(),
                        i.size >> 20,
                        MAX_BYTES >> 20
                    ),
                    "a heap profile of a plausible size",
                    "use a larger --heap-prof-interval, or a shorter run",
                ));
            }
            let f = fs::File::open(&i.abs)
                .map_err(|e| Failure::from(dla_core::Error::io(&i.abs, e)))?;
            let profile = heapprofile::parse_with(f, &mut |s, l, c| maps.lookup(s, l, c))
                .map_err(|e| {
                    Failure::refuse(
                        "malformed-artifact",
                        format!("{}: {e}", i.shown()),
                        "a complete V8 heap profile",
                        "take the profile again (node --heap-prof, or dla harness run <H> --capture alloc)",
                    )
                })?;
            captures.push(ProfileCapture {
                label: i.shown().to_owned(),
                profile,
                sha256: i.sha256.clone(),
            });
            ctx.tc.require_time("dla summarize alloc-stack-diff")?;
        }
        let mut out = alloc_stack::run_diff(
            ctx,
            opts,
            captures,
            &ProfileOrigin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Live,
                runtime: Some(Runtime::Node),
                region: SeriesRegion::Heap,
                order_by_time: false,
                limitations: vec![
                    "a sampling heap profile: counts and bytes are estimates, and small allocations may never be sampled".into(),
                    "each profile covers the objects allocated in its own window that are still alive when it is read; later windows do not hold earlier ones".into(),
                    "samples in native builtins are charged to the nearest JavaScript frame".into(),
                    "the profiles are ordered as given".into(),
                ],
            },
        )?;
        out.body.insert(
            "source_maps".into(),
            json!({"frames_mapped": maps.mapped.get()}),
        );
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_normalized_without_the_disk() {
        assert_eq!(
            normalize(Path::new("/work/out/../src/a.ts")),
            PathBuf::from("/work/src/a.ts")
        );
        assert_eq!(normalize(Path::new("./x/./y")), PathBuf::from("x/y"));
        assert_eq!(normalize(Path::new("../a")), PathBuf::from("../a"));
    }

    #[test]
    fn a_bundle_frame_maps_to_its_source() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("out/app.js");
        fs::create_dir_all(bundle.parent().unwrap()).unwrap();
        fs::write(&bundle, "x").unwrap();
        fs::write(
            dir.path().join("out/app.js.map"),
            r#"{"version":3,"sources":["../src/a.ts"],"mappings":"AAAA,uBACe"}"#,
        )
        .unwrap();
        let maps = Maps::default();
        let (file, line) = maps.lookup(bundle.to_str().unwrap(), 0, 25).unwrap();
        assert!(file.ends_with("/src/a.ts"), "{file}");
        assert_eq!(line, 2);
        assert_eq!(maps.mapped.get(), 1);
        // A script with no map is left alone.
        assert!(maps.lookup("/nowhere/x.js", 0, 0).is_none());
    }
}
