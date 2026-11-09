//! `alloc-stack-diff` over Valgrind memcheck XML (`--leak-check=full --show-reachable=yes
//! --xml=yes`): the leaked and still-reachable blocks of a run, by stack, with the leak
//! kind kept.
//!
//! One file is the at-exit view of one process. Three or more files of runs with growing
//! inputs (the points `dla harness run <H> --tool valgrind` writes) give a live-bytes
//! series per stack. A stack ties to a finding only by its first project frame (rule 2 of
//! the tie rule): the finding's own site or one of its recorded writers. Memcheck names no
//! holder, so no stack ties by holder; the vgdb chain (`valgrind-vgdb`, heap-graph) does.
//!
//! The module also holds the helpers the other native profile readers share: the stack
//! frames of a tool run in the harness scratch copy are mapped back to project paths.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use dla_core::evidence::{Authority, tool_class};
use dla_parsers::{
    model::{AllocProfile, Frame},
    native::valgrind,
};

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    alloc_stack::{self, ProfileCapture, ProfileOrigin},
    input::Input,
};
use crate::fail::{Failure, Res};

/// Largest XML file read.
const MAX_BYTES: u64 = 64 << 20;

/// The memcheck XML reader.
pub struct ValgrindMemcheck;

/// Maps stack-frame paths to project paths. A tool run by the harness works in a scratch
/// copy (`.dla/<run>/harness/H<n>/work/`), so its frames name files there; a bare file
/// name (`decoder.c`) is found in the project when exactly one file has it.
pub struct Rebase {
    project: PathBuf,
    by_name: std::cell::OnceCell<BTreeMap<String, Vec<String>>>,
}

impl Rebase {
    /// For frames of a run over `project`.
    pub fn new(project: &Path) -> Self {
        Self {
            project: project.to_path_buf(),
            by_name: std::cell::OnceCell::new(),
        }
    }

    fn names(&self) -> &BTreeMap<String, Vec<String>> {
        self.by_name.get_or_init(|| {
            let mut m: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let mut stack = vec![(self.project.clone(), 0usize)];
            let mut seen = 0usize;
            while let Some((dir, depth)) = stack.pop() {
                let Ok(rd) = std::fs::read_dir(&dir) else {
                    continue;
                };
                for e in rd.flatten() {
                    seen += 1;
                    if seen > 20_000 {
                        return m;
                    }
                    let name = e.file_name().to_string_lossy().into_owned();
                    let p = e.path();
                    if p.is_dir() {
                        let skip = name.starts_with('.')
                            || matches!(name.as_str(), "target" | "node_modules" | "build");
                        if !skip && depth < 6 {
                            stack.push((p, depth + 1));
                        }
                    } else if let Ok(rel) = p.strip_prefix(&self.project) {
                        m.entry(name)
                            .or_default()
                            .push(rel.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
            m
        })
    }

    /// The project path for `file`, or `file` itself when it maps nowhere.
    pub fn file(&self, file: &str) -> String {
        let f = file.replace('\\', "/");
        if let Some(i) = f.find("/harness/H") {
            let rest = &f[i + "/harness/H".len()..];
            let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
            if digits > 0
                && let Some(after) = rest[digits..].strip_prefix("/work/")
            {
                return after.to_owned();
            }
        }
        if !f.contains('/') && !f.is_empty() {
            let name = f.rsplit('\\').next().unwrap_or(&f);
            if let Some(v) = self.names().get(name)
                && let [one] = v.as_slice()
            {
                return one.clone();
            }
        }
        f
    }

    /// `frame` with its file mapped.
    pub fn frame(&self, frame: &mut Frame) {
        if let Some(f) = &frame.file {
            frame.file = Some(self.file(f));
        }
    }

    /// Every frame of `profile`, mapped.
    pub fn profile(&self, profile: &mut AllocProfile) {
        for g in &mut profile.groups {
            for f in &mut g.frames {
                self.frame(f);
            }
        }
    }
}

impl Adapter for ValgrindMemcheck {
    fn name(&self) -> &'static str {
        "valgrind-memcheck"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "native"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::LEAK_CHECKER
    }
    fn formats(&self) -> &'static str {
        "Valgrind memcheck XML (valgrind --leak-check=full --show-reachable=yes --xml=yes --xml-file=<file>)"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        h.contains("<valgrindoutput>") && !h.contains("<tool>dhat</tool>")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let rebase = Rebase::new(&ctx.tc.project);
        let mut captures = Vec::new();
        let mut incomplete = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let report = valgrind::parse(&text).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{} is not a memcheck XML report: {}", i.shown(), e.message()),
                    "Valgrind memcheck XML written with --xml=yes --xml-file=<file>",
                    "dla harness run <H> --tool valgrind   or   valgrind --leak-check=full --show-reachable=yes --xml=yes --xml-file=<file> <program>",
                )
            })?;
            if !report.complete {
                incomplete.push(i.shown().to_owned());
            }
            let mut profile = report.profile;
            rebase.profile(&mut profile);
            captures.push(ProfileCapture {
                label: i.shown().to_owned(),
                profile,
                sha256: i.sha256.clone(),
            });
        }
        let mut limitations = vec![
            "memcheck scans memory at exit: still-reachable blocks are live at exit, lost blocks have no pointer left; each file is one process".into(),
            "the files are ordered as given, and are runs of different lengths, not dumps of one process".into(),
        ];
        if !incomplete.is_empty() {
            limitations.push(format!(
                "the report of {} did not finish (the process was killed): its records are partial",
                incomplete.join(", ")
            ));
        }
        alloc_stack::run_diff(
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
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch_paths_map_back_to_the_project() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("src")).unwrap();
        std::fs::write(d.path().join("src/decoder.c"), "x").unwrap();
        let r = Rebase::new(d.path());
        assert_eq!(
            r.file("/p/.dla/20261109-1/harness/H2/work/src/decoder.c"),
            "src/decoder.c"
        );
        assert_eq!(r.file("decoder.c"), "src/decoder.c");
        assert_eq!(r.file("nothere.c"), "nothere.c");
        assert_eq!(r.file("/usr/include/x.h"), "/usr/include/x.h");
    }

    #[test]
    fn a_bare_name_with_two_files_stays_bare() {
        let d = tempfile::tempdir().unwrap();
        for p in ["a", "b"] {
            std::fs::create_dir_all(d.path().join(p)).unwrap();
            std::fs::write(d.path().join(p).join("util.c"), "x").unwrap();
        }
        assert_eq!(Rebase::new(d.path()).file("util.c"), "util.c");
    }
}
