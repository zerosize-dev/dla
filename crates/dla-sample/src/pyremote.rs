//! The `python-remote` backend: `sys.remote_exec` (Python 3.14+) runs a small generated
//! script inside the target that appends one JSON line of statistics to a file; the backend
//! reads that line back.
//!
//! Columns: `py.traced` and `py.traced_peak` (only while the target has `tracemalloc` on),
//! `py.gc_objects` (`len(gc.get_objects())`). The script also records the `gc.get_count()`
//! generation counts in the stats file. The target runs the script at its next safe point on
//! its main thread; nothing else of the target is touched.
//!
//! Both the interpreter that calls `remote_exec` and the target must be Python 3.14 or newer.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;

use crate::{
    backend::{Backend, BackendError, OpenCtx, Reading},
    exec,
};

const MIN_INTERVAL: Duration = Duration::from_millis(500);
/// How long to wait for the target to run the script.
const RESULT_WAIT: Duration = Duration::from_secs(5);
const CALL_TIMEOUT: Duration = Duration::from_secs(10);

/// The text of the limitation recorded when no Python 3.14+ is available.
pub const NEEDS_314: &str = "python remote exec needs Python 3.14+";

/// The script injected into the target. `stats_path` is where one JSON object per run is
/// appended.
pub fn script(stats_path: &Path) -> String {
    let path = serde_json::to_string(&stats_path.to_string_lossy()).unwrap_or_default();
    format!(
        "def _dla_stats(path):\n\
\x20   import gc, json, time\n\
\x20   d = {{'t': time.time(), 'gc_objects': len(gc.get_objects()), 'gc_counts': list(gc.get_count())}}\n\
\x20   try:\n\
\x20       import tracemalloc\n\
\x20       if tracemalloc.is_tracing():\n\
\x20           cur, peak = tracemalloc.get_traced_memory()\n\
\x20           d['traced'] = cur\n\
\x20           d['traced_peak'] = peak\n\
\x20   except Exception:\n\
\x20       pass\n\
\x20   with open(path, 'a') as f:\n\
\x20       f.write(json.dumps(d) + '\\n')\n\
_dla_stats({path})\n\
del _dla_stats\n"
    )
}

/// Reading from one JSON line of the stats file.
pub fn parse_stats_line(line: &str) -> Result<Reading, String> {
    let v: Value = serde_json::from_str(line.trim()).map_err(|e| format!("bad stats line: {e}"))?;
    let mut r = Reading::default();
    for (key, column) in [
        ("gc_objects", "py.gc_objects"),
        ("traced", "py.traced"),
        ("traced_peak", "py.traced_peak"),
    ] {
        if let Some(n) = v[key].as_f64() {
            r.values.insert(column.to_owned(), n);
        }
    }
    if r.values.is_empty() {
        return Err("stats line has no figures".to_owned());
    }
    if !r.values.contains_key("py.traced") {
        r.notes
            .push("tracemalloc is not tracing in the target, so py.traced is missing".to_owned());
    }
    Ok(r)
}

/// `(major, minor)` from `Python 3.13.5` style `--version` output.
pub fn parse_version(text: &str) -> Option<(u32, u32)> {
    let v = text.trim().strip_prefix("Python ")?;
    let mut it = v.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it
        .next()?
        .trim_end_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .ok()?;
    Some((major, minor))
}

fn interpreter_version(exe: &Path) -> Option<(u32, u32)> {
    let mut cmd = Command::new(exe);
    cmd.arg("--version");
    let run = exec::run(cmd, Duration::from_secs(10)).ok()?;
    parse_version(&format!("{}{}", run.stdout, run.stderr))
}

/// The first of `python3.14`, `python3`, `python` that is 3.14 or newer, with the versions
/// of the ones that were too old.
fn find_interpreter() -> (Option<PathBuf>, Vec<String>) {
    let mut old = Vec::new();
    for name in ["python3.14", "python3.15", "python3", "python"] {
        let Some(exe) = exec::find_exe(name, &[]) else {
            continue;
        };
        match interpreter_version(&exe) {
            Some(v) if v >= (3, 14) => return (Some(exe), old),
            Some((a, b)) => old.push(format!("{name} is {a}.{b}")),
            None => {}
        }
    }
    (None, old)
}

#[derive(Debug)]
struct PyRemote {
    python: PathBuf,
    pid: u32,
    script: PathBuf,
    stats: PathBuf,
}

/// Finds a Python 3.14+ interpreter and writes the injected script.
pub fn open(ctx: &OpenCtx) -> Result<Box<dyn Backend>, BackendError> {
    let (python, old) = find_interpreter();
    let Some(python) = python else {
        let mut e = BackendError::missing(
            NEEDS_314,
            "install Python 3.14 or newer and run the target on it; sys.remote_exec does not exist before 3.14",
        );
        if !old.is_empty() {
            e.hint = Some(format!(
                "{} ({})",
                e.hint.unwrap_or_default(),
                old.join(", ")
            ));
        }
        return Err(e);
    };
    fs::create_dir_all(&ctx.scratch).map_err(|e| {
        BackendError::blocked(format!(
            "python-remote: cannot create {}: {e}",
            ctx.scratch.display()
        ))
    })?;
    let stats = ctx.scratch.join(format!("sample-py-{}.jsonl", ctx.pid));
    let script_path = ctx.scratch.join(format!("sample-py-{}.py", ctx.pid));
    fs::write(&script_path, script(&stats)).map_err(|e| {
        BackendError::blocked(format!(
            "python-remote: cannot write {}: {e}",
            script_path.display()
        ))
    })?;
    Ok(Box::new(PyRemote {
        python,
        pid: ctx.pid,
        script: script_path,
        stats,
    }))
}

impl PyRemote {
    fn lines(&self) -> Vec<String> {
        fs::read_to_string(&self.stats)
            .map(|t| t.lines().map(str::to_owned).collect())
            .unwrap_or_default()
    }
}

impl Backend for PyRemote {
    fn name(&self) -> &'static str {
        "python-remote"
    }

    fn runtime(&self) -> &'static str {
        "python"
    }

    fn min_interval(&self) -> Duration {
        MIN_INTERVAL
    }

    fn poll(&mut self) -> Result<Reading, BackendError> {
        let before = self.lines().len();
        let mut cmd = Command::new(&self.python);
        cmd.arg("-c").arg(format!(
            "import sys; sys.remote_exec({}, {})",
            self.pid,
            serde_json::to_string(&self.script.to_string_lossy()).unwrap_or_default()
        ));
        let run = exec::run(cmd, CALL_TIMEOUT)
            .map_err(|e| BackendError::blocked(format!("python-remote: {e}")))?;
        if !run.success {
            let why = run
                .stderr
                .lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("sys.remote_exec failed");
            return Err(BackendError::blocked(format!("python-remote: {why}")));
        }
        let deadline = Instant::now() + RESULT_WAIT;
        loop {
            let lines = self.lines();
            if lines.len() > before {
                return parse_stats_line(&lines[lines.len() - 1])
                    .map_err(|e| BackendError::blocked(format!("python-remote: {e}")));
            }
            if Instant::now() >= deadline {
                return Err(BackendError::blocked(
                    "python-remote: the target did not run the injected script within 5s",
                ));
            }
            thread::sleep(Duration::from_millis(25));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_is_valid_python_text_with_the_path() {
        let s = script(Path::new("/tmp/run dir/stats.jsonl"));
        assert!(s.contains("_dla_stats(\"/tmp/run dir/stats.jsonl\")"));
        assert!(s.contains("tracemalloc.is_tracing()"));
        assert!(s.contains("len(gc.get_objects())"));
        // Indentation of the body survived the line continuations.
        assert!(s.contains("\n    try:\n        import tracemalloc\n"));
        if let Some(py) = exec::find_exe("python3", &[]) {
            let dir = std::env::temp_dir().join(format!("dla-pyremote-{}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            let stats = dir.join("s.jsonl");
            let file = dir.join("s.py");
            fs::write(&file, script(&stats)).unwrap();
            let mut c = Command::new(py);
            c.arg(&file);
            let run = exec::run(c, Duration::from_secs(30)).unwrap();
            assert!(run.success, "{}", run.stderr);
            let line = fs::read_to_string(&stats).unwrap();
            let r = parse_stats_line(line.lines().next().unwrap()).unwrap();
            assert!(r.values["py.gc_objects"] > 0.0);
            let _ = fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn stats_lines() {
        let r = parse_stats_line(r#"{"t": 1.0, "gc_objects": 1200, "gc_counts": [1,2,3], "traced": 5000, "traced_peak": 9000}"#).unwrap();
        assert_eq!(r.values["py.traced"], 5000.0);
        assert_eq!(r.values["py.traced_peak"], 9000.0);
        assert_eq!(r.values["py.gc_objects"], 1200.0);
        assert!(r.notes.is_empty());
        let r = parse_stats_line(r#"{"gc_objects": 10}"#).unwrap();
        assert_eq!(r.notes.len(), 1);
        assert!(parse_stats_line("{}").is_err());
        assert!(parse_stats_line("x").is_err());
    }

    #[test]
    fn versions() {
        assert_eq!(parse_version("Python 3.13.5\n"), Some((3, 13)));
        assert_eq!(parse_version("Python 3.14.0rc1"), Some((3, 14)));
        assert_eq!(parse_version("Python 3.15.0a1"), Some((3, 15)));
        assert_eq!(parse_version("nope"), None);
    }
}
