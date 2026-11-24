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
    fs,
    path::Path,
};

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
    series::SeriesRegion,
};
use dla_parsers::node::{heapprofile, sourcemap::ScriptMaps};
use serde_json::json;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest profile file read.
const MAX_BYTES: u64 = 256 << 20;

/// The V8 heap profile reader.
pub struct NodeHeapProfile;

/// Source maps by script path, loaded once.
#[derive(Default)]
struct Maps {
    by_script: RefCell<ScriptMaps>,
    mapped: Cell<u64>,
}

impl Maps {
    fn lookup(&self, script: &str, line: u32, col: u32) -> Option<(String, u32)> {
        let (file, l) = self
            .by_script
            .borrow_mut()
            .lookup(Path::new(script), line, col)?;
        self.mapped.set(self.mapped.get() + 1);
        Some((file.to_string_lossy().replace('\\', "/"), l + 1))
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
                windowed: true,
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
