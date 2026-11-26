//! Holders that belong to the measurement and not to the project.
//!
//! A heap dump of a measured process holds more than the project's own state: the driver the
//! harness (or the model, who fills its hooks) runs keeps its encoders and fixtures in statics on
//! purpose, the DLA agent keeps its own classes and records, and the profiler a capture starts
//! keeps its own caches. None of these is a leak of the project, and no evidence about the project
//! decides them, so a growing one must make no finding, no dump lead and no residual-capture item.
//! One predicate, [`Measurement::is_own`], says whose a holder is; every place that turns a dump
//! holder into a finding or an item asks it.
//!
//! The owner belongs to the measurement when it is:
//!
//! * a class or module of the driver sources of a harness (`harness/<H>/driver/`) or of the agent
//!   files (`harness/<H>/agent/`), named the way the runtime names it: JVM `<package>.<File>` with
//!   the package the file declares (Kotlin also `<File>Kt`), plus the class the manifest runs of a
//!   JVM driver; Python the module `<file stem>`; Node the module path segment
//!   `harness.<H>.driver.<file stem>` (or `.agent.`), the form the Node summarizer builds from the
//!   script path;
//! * the namespace of the JVM agent, [`JVM_AGENT_NAMESPACE`];
//! * the profiler the capture starts, [`PROFILER_NAMESPACE`] (JFR).
//!
//! JDK, runtime and third-party classes stay the project's to answer for (the project can fill a
//! JDK cache), and so does a `thread:<name>` holder (it can hold the project's own thread-local
//! values): neither is excluded here.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::harness::{AgentKind, HarnessManifest};

/// The package of the JVM agent (`harness/jvm/DlaAgent.java`, `DlaDump.java`, `DlaCapture.java`).
pub const JVM_AGENT_NAMESPACE: &str = "dla.harness.";
/// The package of the flight recorder a JVM capture starts.
pub const PROFILER_NAMESPACE: &str = "jdk.jfr.";

/// The classes and modules of the driver and agent sources of one run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Measurement {
    /// JVM classes (`app.DlaLoop`).
    classes: BTreeSet<String>,
    /// Python modules (`dla_harness`).
    modules: BTreeSet<String>,
    /// Node module path segments (`harness.H1.driver.dla-harness`).
    segments: BTreeSet<String>,
}

/// Files below `dir`, at most `depth` levels down.
fn files_below(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            if depth > 0 {
                files_below(&p, depth - 1, out);
            }
        } else {
            out.push(p);
        }
    }
}

/// The package a JVM source file declares (`""` for the default package).
fn declared_package(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("package "))
        .map(|rest| {
            rest.trim_start()
                .chars()
                .take_while(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '$'))
                .collect::<String>()
        })
        .unwrap_or_default()
}

/// True for the characters of a qualified class name.
fn class_like(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '$'))
        && !s.starts_with(|c: char| c.is_ascii_digit() || c == '.')
}

/// `holder` without the kind prefix of a holder key (`static:`, `field:`, `thread-local:`,
/// `closure:`, `registry:<kind>:`). A `thread:` key keeps its prefix: it names no class.
fn owner_of(holder: &str) -> &str {
    let h = holder.trim();
    for p in ["static:", "field:", "thread-local:", "closure:"] {
        if let Some(rest) = h.strip_prefix(p) {
            return rest.trim();
        }
    }
    if let Some(rest) = h.strip_prefix("registry:")
        && let Some((_, owner)) = rest.split_once(':')
    {
        return owner.trim();
    }
    h
}

/// True when `owner` is the class `class` or a member or nested class of it.
fn within_class(owner: &str, class: &str) -> bool {
    owner
        .strip_prefix(class)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.') || rest.starts_with('$'))
}

/// True when `owner` is the module `module` or a name below it.
fn within_module(owner: &str, module: &str) -> bool {
    owner
        .strip_prefix(module)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

/// True when `segment` occurs in `owner` between dots (or at its ends).
fn has_segment(owner: &str, segment: &str) -> bool {
    owner.match_indices(segment).any(|(at, _)| {
        let before = owner[..at].chars().next_back();
        let after = owner[at + segment.len()..].chars().next();
        before.is_none_or(|c| c == '.') && after.is_none_or(|c| c == '.')
    })
}

/// True when `holder` (a holder key or an owner) is in the namespace of the JVM agent or of the
/// profiler: the part of [`Measurement::is_own`] that needs no run.
pub fn in_measurement_namespace(holder: &str) -> bool {
    let o = owner_of(holder);
    o.starts_with(JVM_AGENT_NAMESPACE) || o.starts_with(PROFILER_NAMESPACE)
}

impl Measurement {
    /// What the run directory's harnesses say: the driver and agent sources of each
    /// (`harness/<H>/driver/`, `harness/<H>/agent/`) and, for a JVM driver, the class its manifest
    /// runs.
    pub fn of_run(run_dir: &Path) -> Self {
        let mut m = Self::default();
        let Ok(rd) = fs::read_dir(run_dir.join("harness")) else {
            return m;
        };
        let mut ids: Vec<String> = rd
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| {
                n.strip_prefix('H')
                    .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
            })
            .collect();
        ids.sort();
        for id in ids {
            let base = run_dir.join("harness").join(&id);
            for dir in ["driver", "agent"] {
                let mut files = Vec::new();
                files_below(&base.join(dir), 3, &mut files);
                for f in files {
                    m.add_file(&id, dir, &f);
                }
            }
            if let Ok(man) = HarnessManifest::load(run_dir, &id)
                && man.agent == AgentKind::DriverJvm
                && let Some(last) = man.run.argv.last()
                && class_like(last)
            {
                m.classes.insert(last.clone());
            }
        }
        m
    }

    fn add_file(&mut self, harness: &str, dir: &str, file: &Path) {
        let Some(stem) = file.file_stem().and_then(|s| s.to_str()) else {
            return;
        };
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        match ext {
            "java" | "kt" => {
                let text = fs::read_to_string(file).unwrap_or_default();
                let pkg = declared_package(&text);
                let qualify = |name: &str| {
                    if pkg.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{pkg}.{name}")
                    }
                };
                self.classes.insert(qualify(stem));
                if ext == "kt" {
                    self.classes.insert(qualify(&format!("{stem}Kt")));
                }
            }
            "py" => {
                self.modules.insert(stem.to_owned());
            }
            "js" | "cjs" | "mjs" | "ts" => {
                self.segments
                    .insert(format!("harness.{harness}.{dir}.{stem}"));
            }
            _ => {}
        }
    }

    /// True when nothing of the run's driver or agent sources is known.
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.modules.is_empty() && self.segments.is_empty()
    }

    /// True when the holder (a key such as `static:app.DlaLoop.TCP`, or an owner such as
    /// `app.DlaLoop.TCP`) belongs to the measurement: see the module's documentation.
    pub fn is_own(&self, holder: &str) -> bool {
        if in_measurement_namespace(holder) {
            return true;
        }
        let o = owner_of(holder);
        self.classes.iter().any(|c| within_class(o, c))
            || self.modules.iter().any(|m| within_module(o, m))
            || self.segments.iter().any(|s| has_segment(o, s))
    }

    /// The name of a column of a dump series (`holder:static:app.DlaLoop.TCP`, with or without a
    /// `.count` suffix), when it is the column of a holder of the measurement.
    pub fn is_own_column(&self, column: &str) -> bool {
        let Some(key) = column.strip_prefix("holder:") else {
            return false;
        };
        self.is_own(key.strip_suffix(".count").unwrap_or(key)) || self.is_own(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_with(files: &[(&str, &str)]) -> (tempfile::TempDir, Measurement) {
        let dir = tempfile::tempdir().unwrap();
        for (rel, text) in files {
            let p = dir.path().join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        let m = Measurement::of_run(dir.path());
        (dir, m)
    }

    #[test]
    fn the_driver_class_of_a_jvm_harness_is_the_measurements() {
        let (_d, m) = run_with(&[(
            "harness/H1/driver/DlaLoop.java",
            "package com.acme.shop;\n\nclass DlaLoop {}\n",
        )]);
        assert!(m.is_own("static:com.acme.shop.DlaLoop.TCP"));
        assert!(m.is_own("com.acme.shop.DlaLoop.MASK"));
        assert!(m.is_own("field:com.acme.shop.DlaLoop$Inner.cache"));
        // A class of the project beside it, and one whose name only starts alike, are not.
        assert!(!m.is_own("static:com.acme.shop.Cache.ENTRIES"));
        assert!(!m.is_own("static:com.acme.shop.DlaLoopback.X"));
    }

    #[test]
    fn a_driver_in_the_default_package_and_a_kotlin_driver_are_found() {
        let (_d, m) = run_with(&[
            (
                "harness/H2/driver/DlaJUnitLoop.java",
                "class DlaJUnitLoop {}\n",
            ),
            (
                "harness/H3/driver/Loop.kt",
                "package app.bench\n\nobject Loop\n",
            ),
        ]);
        assert!(m.is_own("DlaJUnitLoop.SEEN"));
        assert!(m.is_own("static:app.bench.Loop.X"));
        assert!(m.is_own("static:app.bench.LoopKt.X"));
        assert!(!m.is_own("app.bench.Other.X"));
    }

    #[test]
    fn the_agent_and_the_profiler_are_the_measurements_whatever_the_run() {
        let m = Measurement::default();
        assert!(m.is_own("static:dla.harness.DlaDump.ROOT_CLASS"));
        assert!(m.is_own("dla.harness.DlaAgent$1.state"));
        assert!(m.is_own("static:jdk.jfr.internal.StringPool.cache"));
        assert!(
            !m.is_own("static:dla.shop.Cache.X"),
            "only the agent's package"
        );
    }

    #[test]
    fn jdk_caches_threads_and_project_classes_are_not_the_measurements() {
        let (_d, m) = run_with(&[(
            "harness/H1/driver/DlaLoop.java",
            "package app;\nclass DlaLoop {}\n",
        )]);
        for h in [
            "static:java.net.InetAddress.cache",
            "static:java.net.InetAddress.expirySet",
            "thread:main",
            "thread-local:app.Session.CURRENT",
            "static:app.Cache.ENTRIES",
            "static:com.fasterxml.jackson.core.Base.X",
        ] {
            assert!(!m.is_own(h), "{h}");
        }
    }

    #[test]
    fn python_and_node_driver_and_agent_modules_are_found() {
        let (_d, m) = run_with(&[
            ("harness/H1/driver/dla_harness.py", "x = 1\n"),
            ("harness/H1/agent/sitecustomize.py", "x = 1\n"),
            ("harness/H2/driver/dla-harness.cjs", "x\n"),
            ("harness/H2/agent/dla-preload.cjs", "x\n"),
        ]);
        assert!(m.is_own("static:dla_harness._seen"));
        assert!(m.is_own("static:sitecustomize.state"));
        assert!(!m.is_own("static:pkg.dla_harness_like.x"));
        assert!(!m.is_own("static:pkg.api.providers"));
        assert!(m.is_own("static:.dla.r1.harness.H2.driver.dla-harness.cache"));
        assert!(m.is_own("closure:.dla.r1.harness.H2.agent.dla-preload.timer"));
        assert!(!m.is_own("static:src.scheduler.shared"));
    }

    #[test]
    fn the_class_a_jvm_manifest_runs_is_the_measurements() {
        let dir = tempfile::tempdir().unwrap();
        let m: HarnessManifest = serde_json::from_value(serde_json::json!({
            "id": "H1", "runtime": "jvm", "mode": "driver", "scratch": "x",
            "run": {"argv": ["java", "@/r/run.args", "app.DlaLoop"], "cwd": "."},
            "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
            "agent": "driver-jvm", "timeout_secs": 60,
        }))
        .unwrap();
        m.save(dir.path()).unwrap();
        let own = Measurement::of_run(dir.path());
        assert!(own.is_own("static:app.DlaLoop.TCP"));
        assert!(!own.is_own("static:app.Cache.TCP"));
    }

    #[test]
    fn a_column_of_a_holder_of_the_measurement_is_its_own() {
        let (_d, m) = run_with(&[(
            "harness/H1/driver/DlaLoop.java",
            "package app;\nclass DlaLoop {}\n",
        )]);
        assert!(m.is_own_column("holder:static:app.DlaLoop.TCP"));
        assert!(m.is_own_column("holder:static:app.DlaLoop.TCP.count"));
        assert!(!m.is_own_column("holder:static:app.Cache.ENTRIES"));
        assert!(!m.is_own_column("jvm.heap_used"));
    }
}
