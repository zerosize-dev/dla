//! `non-heap` over the Python harness's resource files (`py-<n>.nonheap.json`,
//! `dla harness run --capture nonheap`): threads by name pattern, pending asyncio tasks by
//! coroutine and frame, and ResourceWarnings by allocation site.
//!
//! The agent records where each thread and task was created (the project frame that called
//! `Thread(...)` or `create_task(...)`), so a group's column ties to the finding at that
//! line or among its writers (the tie rule's site identity). Thread names are data: a
//! pattern is shown only when it is plain program text (see `jvm_nonheap`), else as a
//! shape. Whole-process counts (`py.threads`, `py.tasks`) are metrics and tie to nothing.

use dla_core::{
    evidence::{Authority, tool_class},
    runtime::Runtime,
};
use dla_parsers::model::{NonHeapMetric, NonHeapSnapshot, ResourceGroup};
use serde::Deserialize;

use super::{
    Adapter, AdapterOut, Ctx, Op, Opts,
    input::Input,
    non_heap::{self, SnapCapture, SnapOrigin},
    py_tracemalloc::frame,
    redact::Redactor,
};
use crate::fail::{Failure, Res};

/// Largest file read.
const MAX_BYTES: u64 = 16 << 20;
/// Most groups read per kind.
const MAX_GROUPS: usize = 400;
/// Longest thread-name pattern shown as it is.
const PATTERN_MAX: usize = 48;
/// Longest word in a pattern shown as it is.
const WORD_MAX: usize = 20;
/// The source the snapshot reports.
const SOURCE: &str = "py-resources";

/// The Python resources reader.
pub struct PyResources;

#[derive(Deserialize, Default)]
struct Resources {
    #[serde(default)]
    dla_py_resources: Option<u32>,
    #[serde(default)]
    threads: Threads,
    #[serde(default)]
    tasks: Tasks,
    #[serde(default)]
    resource_warnings: Warnings,
}

#[derive(Deserialize, Default)]
struct Threads {
    #[serde(default)]
    total: u64,
    #[serde(default)]
    groups: Vec<ThreadGroup>,
}

#[derive(Deserialize, Default)]
struct ThreadGroup {
    #[serde(default)]
    pattern: String,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    count: u64,
}

#[derive(Deserialize, Default)]
struct Tasks {
    #[serde(default)]
    total: u64,
    #[serde(default)]
    groups: Vec<TaskGroup>,
}

#[derive(Deserialize, Default)]
struct TaskGroup {
    #[serde(default)]
    coro: String,
    #[serde(default)]
    frame: Option<String>,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    count: u64,
}

#[derive(Deserialize, Default)]
struct Warnings {
    #[serde(default)]
    total: u64,
    #[serde(default)]
    groups: Vec<WarnGroup>,
}

#[derive(Deserialize, Default)]
struct WarnGroup {
    #[serde(default)]
    what: String,
    #[serde(default)]
    site: Option<String>,
    #[serde(default)]
    count: u64,
}

/// True when a normalized thread-name pattern is plain program text: words of letters
/// and `*` (a digit run) around separators. A word longer than `WORD_MAX`, or with more
/// than two `*`, looks like a value (an id, a key) and is not plain.
fn plain_pattern(p: &str) -> bool {
    if p.is_empty() || p.len() > PATTERN_MAX {
        return false;
    }
    let mut word = 0usize;
    let mut stars = 0usize;
    for c in p.chars() {
        match c {
            c if c.is_ascii_alphabetic() || c == '*' => {
                word += 1;
                if c == '*' {
                    stars += 1;
                }
                if word > WORD_MAX || stars > 2 {
                    return false;
                }
            }
            '-' | '_' | ' ' | '.' | '#' | '/' | ':' | '|' | '[' | ']' | '(' | ')' => {
                word = 0;
                stars = 0;
            }
            _ => return false,
        }
    }
    true
}

/// The key of a `name:<pattern>` group: the pattern as it is when it is plain program
/// text or the run has raw-values consent, else its shape.
fn thread_key(pattern: &str, red: &Redactor) -> String {
    if red.mode() == "raw" || (plain_pattern(pattern) && red.name(pattern) == pattern) {
        format!("name:{pattern}")
    } else {
        format!("name:{}", red.value(pattern))
    }
}

/// The key of a ResourceWarning group: the agent's label (digit runs as `*`) when it is plain
/// program text or the run has raw-values consent, else its shape (a warning text can carry a
/// file name, a URL or a token).
fn warn_key(what: &str, red: &Redactor) -> String {
    if red.mode() == "raw" || (plain_pattern(what) && red.name(what) == what) {
        format!("warn:{what}")
    } else {
        format!("warn:{}", red.value(what))
    }
}

/// The frame a `file:line` site names; a site that does not end in a line number has none.
fn site_frame(
    site: &Option<String>,
    project: &std::path::Path,
) -> Option<dla_parsers::model::Frame> {
    let (file, line) = site.as_deref()?.rsplit_once(':')?;
    let line: u32 = line.parse().ok()?;
    (!file.is_empty() && line > 0).then(|| frame(project, "", file, line))
}

fn count_metric(category: &str, name: &str, v: u64) -> NonHeapMetric {
    NonHeapMetric {
        category: category.to_owned(),
        name: name.to_owned(),
        value: i64::try_from(v).unwrap_or(i64::MAX),
        unit: "count".to_owned(),
    }
}

fn snapshot(r: &Resources, red: &Redactor, project: &std::path::Path) -> NonHeapSnapshot {
    let metrics = vec![
        count_metric("py", "threads", r.threads.total),
        count_metric("py", "tasks", r.tasks.total),
        count_metric("py", "resource_warnings", r.resource_warnings.total),
    ];
    let mut groups = Vec::new();
    for g in r.threads.groups.iter().take(MAX_GROUPS) {
        groups.push(ResourceGroup {
            kind: "thread".to_owned(),
            key: thread_key(&g.pattern, red),
            count: g.count,
            site: site_frame(&g.site, project),
        });
    }
    for g in r.tasks.groups.iter().take(MAX_GROUPS) {
        let key = match &g.frame {
            Some(f) => format!("coro:{}@{f}", g.coro),
            None => format!("coro:{}", g.coro),
        };
        groups.push(ResourceGroup {
            kind: "task".to_owned(),
            key,
            count: g.count,
            site: site_frame(&g.site, project),
        });
    }
    for g in r.resource_warnings.groups.iter().take(MAX_GROUPS) {
        groups.push(ResourceGroup {
            kind: "handle".to_owned(),
            key: warn_key(&g.what, red),
            count: g.count,
            site: site_frame(&g.site, project),
        });
    }
    NonHeapSnapshot {
        source: SOURCE.to_owned(),
        timestamp_ms: None,
        metrics,
        groups,
    }
}

impl Adapter for PyResources {
    fn name(&self) -> &'static str {
        "py-resources"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "python"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::RUNTIME_COUNTER
    }
    fn formats(&self) -> &'static str {
        "Python harness resources (py-<n>.nonheap.json, {\"dla_py_resources\":1, \"threads\", \"tasks\"})"
    }
    fn sniff(&self, input: &Input) -> bool {
        input.head_text().contains("\"dla_py_resources\"")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let mut captures = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let bad = |why: String| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{}: {why}", i.shown()),
                    "py-<n>.nonheap.json written by the Python harness",
                    "dla harness run <H> --capture nonheap",
                )
            };
            let r: Resources = serde_json::from_str(&text).map_err(|e| bad(e.to_string()))?;
            if r.dla_py_resources.is_none() {
                return Err(bad("no `dla_py_resources` version".to_owned()));
            }
            captures.push(SnapCapture {
                label: i.shown().to_owned(),
                snap: snapshot(&r, &ctx.redactor, &ctx.tc.project),
                sha256: i.sha256.clone(),
            });
        }
        non_heap::run_diff(
            ctx,
            opts,
            captures,
            &SnapOrigin {
                tool: self.name(),
                class: self.tool_class(),
                authority: Authority::Live,
                runtime: Some(Runtime::Python),
                order_by_time: false,
                limitations: vec![
                    "a thread's or task's site is the project frame that created it, not the frame it runs in; a task has a site only when asyncio ran in debug mode (the agent turns it on)".into(),
                    "thread names are shown only as a plain pattern with digit runs replaced by *; any other name is a shape".into(),
                    "ResourceWarning counts are cumulative (a warning is raised when an unclosed object is collected) and their site is where the object was allocated".into(),
                    "the files are ordered as given; each is one point".into(),
                ],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> Redactor {
        Redactor::new(b"salt", false)
    }

    #[test]
    fn groups_carry_creation_sites_and_totals_stay_metrics() {
        let r: Resources = serde_json::from_str(
            r#"{"dla_py_resources":1,
                "threads":{"total":3,"groups":[{"pattern":"Thread-* (poll)","site":"app/poller.py:12","count":2},
                                               {"pattern":"MainThread","site":null,"count":1}]},
                "tasks":{"total":4,"groups":[{"coro":"heartbeat","frame":"app/hb.py:9","site":"app/main.py:30","count":4}]},
                "resource_warnings":{"total":5,"groups":[{"what":"unclosed file","site":"app/io.py:4","count":5}]}}"#,
        )
        .unwrap();
        let s = snapshot(&r, &red(), std::path::Path::new("/proj"));
        assert!(
            s.metrics
                .iter()
                .any(|m| m.name == "threads" && m.value == 3)
        );
        let t = s
            .groups
            .iter()
            .find(|g| g.kind == "thread" && g.count == 2)
            .unwrap();
        assert_eq!(t.key, "name:Thread-* (poll)");
        assert_eq!(t.site.as_ref().unwrap().line, Some(12));
        let m = s
            .groups
            .iter()
            .find(|g| g.key == "name:MainThread")
            .unwrap();
        assert!(m.site.is_none());
        let k = s.groups.iter().find(|g| g.kind == "task").unwrap();
        assert_eq!(k.key, "coro:heartbeat@app/hb.py:9");
        assert_eq!(
            k.site.as_ref().unwrap().file.as_deref(),
            Some("app/main.py")
        );
        let w = s.groups.iter().find(|g| g.kind == "handle").unwrap();
        assert_eq!(w.key, "warn:unclosed file");
    }

    #[test]
    fn a_value_like_thread_name_is_a_shape() {
        let k = thread_key("AKIAIOSFODNN*EXAMPLE*", &red());
        assert!(!k.contains("AKIA"), "{k}");
        assert!(thread_key("pool-*-worker", &red()).ends_with("pool-*-worker"));
        // A warning label with a value in it is shaped like a thread name; a plain one stays.
        assert_eq!(warn_key("unclosed file", &red()), "warn:unclosed file");
        let k = warn_key("unclosed socket to AKIAIOSFODNN7EXAMPLE1", &red());
        assert!(!k.contains("AKIAIOSFODNN7EXAMPLE1"), "{k}");
    }

    #[test]
    fn a_bad_site_is_dropped() {
        let p = std::path::Path::new("/p");
        assert!(site_frame(&Some("no-line".into()), p).is_none());
        assert!(site_frame(&Some("a.py:0".into()), p).is_none());
        assert!(site_frame(&None, p).is_none());
    }
}
