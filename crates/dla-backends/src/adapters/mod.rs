//! One adapter per static backend.
//!
//! Every adapter exposes `run(&Job) -> Outcome`. It finds the tool (tools directory,
//! then `PATH`), builds the command with caches under the run's `tmp/`, runs it with a
//! timeout, and turns the machine-readable output into [`StaticHit`]s. The parsing
//! functions are pure and tested against recorded outputs in `tests/data/`.

use std::{path::PathBuf, time::Duration};

use dla_core::{
    detectors::Detector,
    runtime::Runtime,
    state::{Scope, ScopeKind},
    static_hit::StaticHit,
    toolctx::ToolCtx,
};
use regex::Regex;
use serde_json::{Value, json};

use crate::{
    exec::{self, Cmd, Output},
    fsx::{PathMap, ToolEnv},
};

pub mod codeql;
pub mod eslint;
pub mod go;
pub mod jvm;
pub mod python;
pub mod rapx;
pub mod sonar;

/// Everything an adapter needs for one run.
pub struct Job<'a> {
    /// The run context.
    pub ctx: &'a ToolCtx,
    /// Runtime being swept.
    pub lang: Runtime,
    /// The detector rows of this backend for this runtime.
    pub rules: Vec<&'a Detector>,
    /// Where tools are found.
    pub env: &'a ToolEnv,
    /// `tmp/<lang>-<backend>` under the run directory (created).
    pub scratch: PathBuf,
    /// Hard limit for the main tool run.
    pub timeout: Duration,
    /// The run's scope (`None`: everything is in scope).
    pub scope: Option<Scope>,
    /// Backends of this sweep that already ran (not blocked, not missing), in order; the
    /// server-only Sonar row names them.
    pub ran: Vec<String>,
}

impl Job<'_> {
    /// The project root.
    pub fn project(&self) -> &std::path::Path {
        &self.ctx.project
    }

    /// The rule identifiers of the rows, in table order.
    pub fn rule_ids(&self) -> Vec<&str> {
        self.rules.iter().map(|d| d.rule.as_str()).collect()
    }

    /// True when the table lists `rule`.
    pub fn wants(&self, rule: &str) -> bool {
        self.rules.iter().any(|d| d.rule == rule)
    }

    /// Whether a project-relative path (`/` separators) is inside the run's scope.
    pub fn in_scope(&self, rel: &str) -> bool {
        self.scope.as_ref().is_none_or(|s| s.includes(rel))
    }

    /// Whether a walk or a build unit rooted at the project-relative directory `rel_dir`
    /// (`""` is the project root) can hold in-scope files.
    pub fn may_contain(&self, rel_dir: &str) -> bool {
        self.scope.as_ref().is_none_or(|s| s.may_contain(rel_dir))
    }

    /// The directory a tool that takes an input directory is pointed at, relative to the
    /// project: `.` for the repository scope, the scope path for a module or service
    /// when that directory exists.
    pub fn target(&self) -> String {
        match &self.scope {
            Some(s) if s.kind != ScopeKind::Repository => {
                let p = s.path.trim().trim_matches('/').trim_start_matches("./");
                if !p.is_empty() && p != "." && !p.contains("..") && self.project().join(p).is_dir()
                {
                    return p.to_owned();
                }
                ".".to_owned()
            }
            _ => ".".to_owned(),
        }
    }

    /// Excluded paths of the scope that exist as project-relative `/` paths.
    pub fn excluded(&self) -> Vec<String> {
        self.scope
            .as_ref()
            .map(|s| {
                s.excluded
                    .iter()
                    .map(|e| {
                        e.trim()
                            .trim_matches('/')
                            .trim_start_matches("./")
                            .to_owned()
                    })
                    .filter(|e| !e.is_empty() && !e.contains(".."))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Maps tool paths onto project-relative paths.
    pub fn paths(&self) -> PathMap {
        PathMap::new(self.project())
    }
}

/// What running one backend came to.
pub enum Outcome {
    /// The tool is not available.
    Missing {
        /// Why.
        reason: String,
    },
    /// The tool exists but did not produce usable output.
    Blocked {
        /// Why.
        reason: String,
        /// What was captured, for the raw file.
        raw: Value,
    },
    /// The tool ran.
    Ran(Box<Ran>),
}

/// A successful tool run.
#[derive(Debug, Clone, Default)]
pub struct Ran {
    /// Tool version (`unknown` when unreadable).
    pub version: String,
    /// The main command.
    pub command: Vec<String>,
    /// Exit code of the main command.
    pub exit: Option<i32>,
    /// Hits for rules the table lists (not yet de-duplicated).
    pub hits: Vec<StaticHit>,
    /// Load, parse or compile errors the tool reported.
    pub load_errors: u64,
    /// Limits of this run (partial classpath, light mode, ...).
    pub limitations: Vec<String>,
    /// A note stored in coverage (`not applicable` reasons).
    pub note: Option<String>,
    /// Everything captured, for `sweep/<lang>-<backend>.json`.
    pub raw: Value,
}

impl Outcome {
    /// A blocked outcome that stores the run records.
    pub fn blocked(reason: impl Into<String>, runs: Vec<Value>) -> Self {
        Self::Blocked {
            reason: reason.into(),
            raw: json!({ "runs": runs }),
        }
    }

    /// A missing outcome.
    pub fn missing(reason: impl Into<String>) -> Self {
        Self::Missing {
            reason: reason.into(),
        }
    }
}

/// Runs the adapter of `backend`. `dla` is handled by the caller.
pub fn run(backend: &str, job: &Job) -> Outcome {
    match backend {
        "staticcheck" => go::staticcheck(job),
        "golangci-lint" => go::golangci(job),
        "govet" => go::govet(job),
        "ruff" => python::ruff(job),
        "pylint" => python::pylint(job),
        "eslint" => eslint::run(job),
        "errorprone" => jvm::errorprone(job),
        "spotbugs" => jvm::spotbugs(job),
        "detekt" => jvm::detekt(job),
        "rapx" => rapx::run(job),
        "codeql" => codeql::run(job),
        "sonar" => sonar::run(job),
        other => Outcome::missing(format!("no adapter for backend {other:?}")),
    }
}

/// Bytes of a stream kept in the raw record.
const RAW_STREAM_CAP: usize = 8 * 1024 * 1024;

fn cap(s: &str) -> Value {
    if s.len() <= RAW_STREAM_CAP {
        return Value::String(s.to_owned());
    }
    let mut end = RAW_STREAM_CAP;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    Value::String(format!("{}\n[truncated]", &s[..end]))
}

/// The record of one process run for the raw file. `stdout` is embedded as JSON when
/// it parses as one document.
pub fn run_record(cmd: &Cmd, out: &Output) -> Value {
    let mut v = json!({
        "command": cmd.display(),
        "cwd": cmd.cwd.as_deref().map(dla_core::paths::to_slash),
        "exit_code": out.exit,
        "timed_out": out.timed_out,
        "truncated": out.truncated,
        "millis": out.millis,
        "stderr": cap(&out.stderr),
    });
    let parsed = if out.stdout.len() <= RAW_STREAM_CAP {
        serde_json::from_str::<Value>(out.stdout.trim()).ok()
    } else {
        None
    };
    match parsed {
        Some(j) => v["stdout_json"] = j,
        None => v["stdout"] = cap(&out.stdout),
    }
    if let Some(e) = &out.spawn_error {
        v["spawn_error"] = json!(e);
    }
    v
}

/// Adds a file the tool wrote (parsed when JSON) to a run record.
pub fn attach_file(rec: &mut Value, key: &str, text: &str) {
    match serde_json::from_str::<Value>(text) {
        Ok(j) if text.len() <= RAW_STREAM_CAP => rec[key] = j,
        _ => rec[key] = cap(text),
    }
}

/// The first line of a tool's version output that carries a version number.
pub fn version_of(cmd: Cmd) -> String {
    let out = exec::run(&cmd.timeout(Duration::from_secs(60)));
    let re = Regex::new(r"\d+(?:\.\d+)+(?:[-+.][0-9A-Za-z]+)*").expect("static regex");
    for text in [&out.stdout, &out.stderr] {
        for line in text.lines() {
            if line.starts_with("Picked up") || line.starts_with("SLF4J") {
                continue;
            }
            if let Some(m) = re.find(line) {
                return m.as_str().to_owned();
            }
        }
    }
    "unknown".to_owned()
}

/// Builds a hit with a single-line, bounded message.
pub fn hit(backend: &str, rule: &str, file: &str, line: u32, message: &str) -> StaticHit {
    let msg: String = message
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .take(300)
        .collect();
    StaticHit {
        backend: backend.to_owned(),
        rule: rule.to_owned(),
        file: file.to_owned(),
        line: line.max(1),
        message: msg,
    }
}

/// Keeps one hit per `(rule, file, line)`, in first-seen order.
pub fn dedupe(hits: Vec<StaticHit>) -> Vec<StaticHit> {
    let mut seen = std::collections::HashSet::new();
    hits.into_iter()
        .filter(|h| seen.insert((h.rule.clone(), h.file.clone(), h.line)))
        .collect()
}

/// Comma-joined rule list.
pub fn csv(ids: &[&str]) -> String {
    ids.join(",")
}

/// Redirects every cache and config directory a tool might use into the run's
/// `tmp/`, so nothing is written under `HOME`.
pub fn isolate(cmd: Cmd, job: &Job) -> Cmd {
    let t = job.scratch.join("cache");
    cmd.env("XDG_CACHE_HOME", t.join("xdg").into_os_string())
        .env("XDG_CONFIG_HOME", t.join("config").into_os_string())
        .env("XDG_DATA_HOME", t.join("data").into_os_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_keeps_the_first_line_and_clamps_the_line() {
        let h = hit("ruff", "B019", "a.py", 0, "first line\nsecond");
        assert_eq!(h.line, 1);
        assert_eq!(h.message, "first line");
        assert_eq!(h.origin(), "detector:ruff/B019");
        assert_eq!(h.site(), "a.py:1");
    }

    #[test]
    fn dedupe_keeps_first_per_site() {
        let a = hit("x", "R", "f", 3, "one");
        let b = hit("x", "R", "f", 3, "two");
        let c = hit("x", "R", "f", 4, "three");
        let d = dedupe(vec![a, b, c]);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].message, "one");
    }

    #[cfg(unix)]
    #[test]
    fn version_of_reads_the_first_numbered_line() {
        let v = version_of(
            Cmd::new("sh").args(["-c", "echo 'Picked up X 1.1'; echo 'tool v2.5.1-rc2 built'"]),
        );
        assert_eq!(v, "2.5.1-rc2");
        assert_eq!(
            version_of(Cmd::new("sh").args(["-c", "echo none"])),
            "unknown"
        );
    }

    #[test]
    fn run_record_embeds_json_stdout() {
        let out = Output {
            exit: Some(1),
            stdout: "{\"a\":1}".into(),
            ..Output::default()
        };
        let r = run_record(&Cmd::new("x").arg("y"), &out);
        assert_eq!(r["stdout_json"]["a"], 1);
        assert_eq!(r["exit_code"], 1);
        assert_eq!(r["command"][1], "y");
        let out = Output {
            stdout: "plain".into(),
            ..Output::default()
        };
        assert_eq!(run_record(&Cmd::new("x"), &out)["stdout"], "plain");
    }
}
