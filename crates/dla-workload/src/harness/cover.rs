//! Exercised entry points from coverage data.
//!
//! A harness run exercises the entry points that `dla workload discover` listed in
//! `coverage.json` (`file::Owner.name`). What ran is read from the agents' own data:
//!
//! | Runtime | Source | Match |
//! |---|---|---|
//! | JVM | loaded project classes in the dump | the entry's file is the class's source file |
//! | Python | `module:qualname@file:line` items in the dump | `file::qualname` |
//! | Node | `NODE_V8_COVERAGE` JSON files in `cov/` | file and function name, count above 0 |
//! | Go | `-coverprofile` files in `cov/`, read with `go tool cover -func` | file and function name, coverage above 0.0% |
//! | Rust, native | none | recorded as a blind spot |

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use dla_core::{
    Refusal,
    coverage::{CoverageManifest, NotExercisable},
    harness::DumpCoverage,
    runtime::Runtime,
    toolctx::ToolCtx,
};
use serde_json::Value;

/// What a run exercised.
#[derive(Debug, Clone, Default)]
pub struct Exercised {
    /// Entry-point IDs (`file::Owner.name`) that ran, a subset of the discovered ones.
    pub entries: BTreeSet<String>,
    /// What the data could not tell.
    pub limitations: Vec<String>,
}

/// `file` and bare function name of an entry-point ID.
fn split_entry(id: &str) -> Option<(&str, &str)> {
    let (file, rest) = id.split_once("::")?;
    Some((file, rest.rsplit('.').next().unwrap_or(rest)))
}

/// The part of an entry-point ID after `::` (`Owner.name` or `name`).
fn qualified(id: &str) -> Option<&str> {
    id.split_once("::").map(|(_, q)| q)
}

/// Project-relative form of `path` (absolute inside `work`, or already relative).
fn relative_to(work: &Path, path: &str) -> Option<String> {
    let p = Path::new(path);
    if !p.is_absolute() {
        return Some(path.trim_start_matches("./").replace('\\', "/"));
    }
    let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let root = comparable(&work.to_string_lossy());
    [canon.to_string_lossy().into_owned(), path.to_owned()]
        .iter()
        .find_map(|cand| {
            let c = comparable(cand);
            let rest = c.strip_prefix(&root)?.strip_prefix('/')?;
            // Keep the original spelling of the remainder (case matters on Unix).
            let orig = cand.replace('\\', "/");
            let start = orig.len().checked_sub(rest.len())?;
            Some(orig.get(start..).unwrap_or(rest).to_owned())
        })
}

/// A path as a string that compares equal across spellings: forward slashes, no
/// Windows verbatim prefix (`\\?\`), and lower case on Windows.
fn comparable(path: &str) -> String {
    let p = path.strip_prefix(r"\\?\").unwrap_or(path).replace('\\', "/");
    let p = p.trim_end_matches('/').to_owned();
    if cfg!(windows) { p.to_lowercase() } else { p }
}

/// The local path of a `file://` URL as V8 writes it: percent-decoded, and on Windows
/// without the slash before the drive letter (`file:///C:/a` is `C:/a`).
fn file_url_path(url: &str) -> Option<String> {
    let rest = url.strip_prefix("file://")?;
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(&rest[i + 1..i + 3], 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    let path = String::from_utf8(out).ok()?;
    let b = path.as_bytes();
    if b.len() >= 3 && b[0] == b'/' && b[1].is_ascii_alphabetic() && b[2] == b':' {
        return Some(path[1..].to_owned());
    }
    Some(path)
}

/// Items of the dump coverage sections, following a `file` pointer when there is one.
fn dump_items(dump_cov: &[DumpCoverage], hdir: &Path) -> (BTreeSet<String>, Vec<String>) {
    let mut items = BTreeSet::new();
    let mut notes = Vec::new();
    for c in dump_cov {
        items.extend(c.items.iter().cloned());
        let Some(file) = &c.file else { continue };
        let path = if Path::new(file).is_absolute() {
            PathBuf::from(file)
        } else {
            hdir.join(file)
        };
        let Ok(text) = fs::read_to_string(&path) else {
            notes.push(format!("coverage file {file} could not be read"));
            continue;
        };
        match serde_json::from_str::<Value>(&text) {
            Ok(Value::Array(a)) => {
                items.extend(a.iter().filter_map(|v| v.as_str().map(str::to_owned)))
            }
            Ok(Value::Object(o)) => {
                if let Some(Value::Array(a)) = o.get("items") {
                    items.extend(a.iter().filter_map(|v| v.as_str().map(str::to_owned)));
                }
            }
            _ => items.extend(
                text.lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(str::to_owned),
            ),
        }
    }
    (items, notes)
}

fn jvm_entries(discovered: &[String], loaded: &BTreeSet<String>) -> BTreeSet<String> {
    // Top-level class paths (`shop/OrderCache`) and simple names of every loaded class.
    let tops: BTreeSet<String> = loaded
        .iter()
        .map(|c| c.split('$').next().unwrap_or(c).replace(['.', '/'], "/"))
        .collect();
    let simple: BTreeSet<&str> = loaded
        .iter()
        .filter_map(|c| c.rsplit(['.', '$', '/']).next())
        .collect();
    discovered
        .iter()
        .filter(|id| {
            let Some((file, _)) = id.split_once("::") else {
                return false;
            };
            let stem = file.rsplit_once('.').map_or(file, |(s, _)| s);
            let stem_name = stem.rsplit('/').next().unwrap_or(stem);
            let owner = qualified(id)
                .and_then(|q| q.rsplit_once('.').map(|(o, _)| o))
                .map(|o| o.rsplit('.').next().unwrap_or(o))
                .unwrap_or(stem_name);
            let file_loaded = tops
                .iter()
                .any(|t| stem == t || stem.ends_with(&format!("/{t}")));
            file_loaded && (owner == stem_name || simple.contains(owner))
        })
        .cloned()
        .collect()
}

fn python_entries(
    discovered: &BTreeSet<&str>,
    items: &BTreeSet<String>,
    work: &Path,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for item in items {
        // `module:qualname@file:line`
        let Some((left, right)) = item.rsplit_once('@') else {
            continue;
        };
        let Some((_, qualname)) = left.split_once(':') else {
            continue;
        };
        let file = right.rsplit_once(':').map_or(right, |(f, _)| f);
        let Some(file) = relative_to(work, file) else {
            continue;
        };
        let qualname = qualname.replace(".<locals>", "");
        let id = format!("{file}::{qualname}");
        if discovered.contains(id.as_str()) {
            out.insert(id);
        }
    }
    out
}

/// `(project-relative file, function name)` pairs with a positive count in the
/// `NODE_V8_COVERAGE` files of `cov_dir`.
fn node_functions(cov_dir: &Path, work: &Path) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    let Ok(rd) = fs::read_dir(cov_dir) else {
        return out;
    };
    for e in rd.filter_map(Result::ok) {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&p) else {
            continue;
        };
        let Ok(doc) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        for script in doc
            .get("result")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(url) = script.get("url").and_then(Value::as_str) else {
                continue;
            };
            let Some(path) = file_url_path(url) else {
                continue;
            };
            let Some(rel) = relative_to(work, &path) else {
                continue;
            };
            if rel.starts_with("node_modules/") || rel.contains("/node_modules/") {
                continue;
            }
            for f in script
                .get("functions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let name = f.get("functionName").and_then(Value::as_str).unwrap_or("");
                let count = f
                    .get("ranges")
                    .and_then(Value::as_array)
                    .and_then(|r| r.first())
                    .and_then(|r| r.get("count"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                if !name.is_empty() && count > 0.0 {
                    out.insert((rel.clone(), name.to_owned()));
                }
            }
        }
    }
    out
}

/// `(path as printed, function name)` pairs with coverage above 0.0% in the Go profiles
/// of `cov_dir`, read with `go tool cover -func` run in `work`.
fn go_functions(cov_dir: &Path, work: &Path) -> (BTreeSet<(String, String)>, Vec<String>) {
    let mut out = BTreeSet::new();
    let mut notes = Vec::new();
    let Ok(rd) = fs::read_dir(cov_dir) else {
        return (out, notes);
    };
    let mut files: Vec<PathBuf> = rd
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("out"))
        .collect();
    files.sort();
    for f in files {
        let res = Command::new("go")
            .args(["tool", "cover", "-func"])
            .arg(&f)
            .current_dir(work)
            .env("GOPROXY", "off")
            .env("GOFLAGS", "-mod=mod")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        let Ok(o) = res else {
            notes.push("go is not on PATH: Go coverage profiles were not read".to_owned());
            return (out, notes);
        };
        if !o.status.success() {
            notes.push(format!(
                "go tool cover failed on {}",
                f.file_name()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
            ));
            continue;
        }
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            let mut parts = line.split_whitespace();
            let (Some(loc), Some(name), Some(pct)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            if loc.starts_with("total:") {
                continue;
            }
            let pct: f64 = pct.trim_end_matches('%').parse().unwrap_or(0.0);
            // `path/file.go:33:`
            let loc = loc.trim_end_matches(':');
            let Some((path, _line)) = loc.rsplit_once(':') else {
                continue;
            };
            if pct > 0.0 {
                out.insert((path.to_owned(), name.to_owned()));
            }
        }
    }
    (out, notes)
}

/// The discovered entry points of `runtime` that the coverage data shows ran.
///
/// `dump_cov` are the coverage sections of the measured processes' dump records;
/// `cov_dir` is the harness `cov/` directory; `hdir` the harness directory; `work` the
/// scratch project.
pub fn exercised(
    runtime: Runtime,
    discovered: &[String],
    dump_cov: &[DumpCoverage],
    hdir: &Path,
    cov_dir: &Path,
    work: &Path,
) -> Exercised {
    let work = work.canonicalize().unwrap_or_else(|_| work.to_path_buf());
    let mut res = Exercised::default();
    if discovered.is_empty() {
        res.limitations.push(
            "no entry points are listed for this runtime: run dla workload discover before the harness to see which ones ran".to_owned(),
        );
        return res;
    }
    match runtime {
        Runtime::Jvm => {
            let (items, notes) = dump_items(dump_cov, hdir);
            res.limitations.extend(notes);
            if items.is_empty() {
                res.limitations
                    .push("the JVM agent reported no loaded classes".to_owned());
            }
            res.entries = jvm_entries(discovered, &items);
        }
        Runtime::Python => {
            let (items, notes) = dump_items(dump_cov, hdir);
            res.limitations.extend(notes);
            if items.is_empty() {
                res.limitations.push(
                    "the Python agent reported no executed functions (sys.monitoring needs Python 3.12 or newer)"
                        .to_owned(),
                );
            }
            let set: BTreeSet<&str> = discovered.iter().map(String::as_str).collect();
            res.entries = python_entries(&set, &items, &work);
        }
        Runtime::Node => {
            let fns = node_functions(cov_dir, &work);
            if fns.is_empty() {
                res.limitations
                    .push("no NODE_V8_COVERAGE data was written to cov/".to_owned());
            }
            res.entries = discovered
                .iter()
                .filter(|id| {
                    split_entry(id)
                        .is_some_and(|(f, n)| fns.contains(&(f.to_owned(), n.to_owned())))
                })
                .cloned()
                .collect();
        }
        Runtime::Go => {
            let (fns, notes) = go_functions(cov_dir, &work);
            res.limitations.extend(notes);
            if fns.is_empty() && res.limitations.is_empty() {
                res.limitations
                    .push("no Go coverage profile was written to cov/".to_owned());
            }
            res.entries = discovered
                .iter()
                .filter(|id| {
                    split_entry(id).is_some_and(|(f, n)| {
                        fns.iter()
                            .any(|(p, name)| name == n && (p == f || p.ends_with(&format!("/{f}"))))
                    })
                })
                .cloned()
                .collect();
        }
        Runtime::Rust | Runtime::Native => {
            res.limitations.push(format!(
                "exercised entry points are not measured for {}: the allocator shim sees allocations, not calls",
                runtime.as_str()
            ));
        }
    }
    res
}

/// Writes the run's coverage into `coverage.json`: the worker's iterations and the
/// exercised entry points. When nothing usable ran, every discovered entry point that is
/// not already exercised gets `not_exercisable = reason`.
///
/// Returns the limitations of the coverage data.
pub fn record(
    ctx: &ToolCtx,
    runtime: Runtime,
    outcome: &CoverageOutcome<'_>,
) -> Result<Vec<String>, Refusal> {
    let worker = runtime.as_str();
    let discovered = CoverageManifest::load(&ctx.run_dir)
        .map_err(|e| e.to_refusal())?
        .and_then(|m| {
            m.workers
                .get(worker)
                .map(|w| w.entry_points.discovered.clone())
        })
        .unwrap_or_default();
    let mut limitations = Vec::new();
    let found = match outcome {
        CoverageOutcome::Ran {
            dump_cov,
            hdir,
            cov_dir,
            work,
            ..
        } => {
            let ex = exercised(runtime, &discovered, dump_cov, hdir, cov_dir, work);
            limitations.extend(ex.limitations);
            ex.entries
        }
        CoverageOutcome::Failed { .. } => BTreeSet::new(),
    };
    CoverageManifest::update(&ctx.run_dir, |m| match outcome {
        CoverageOutcome::Ran { iterations, .. } => {
            m.set_iterations(worker, *iterations);
            m.add_exercised(worker, found.iter().cloned());
            if matches!(runtime, Runtime::Rust | Runtime::Native) {
                m.add_blind_spot(
                    worker,
                    "the allocator shim does not record which entry points ran",
                );
            }
            // An entry point that ran is no longer unrunnable.
            m.worker_mut(worker)
                .entry_points
                .not_exercisable
                .retain(|n| !found.contains(&n.entry));
        }
        CoverageOutcome::Failed { reason } => {
            let w = m.worker_mut(worker);
            let exercised: BTreeSet<String> = w.entry_points.exercised.iter().cloned().collect();
            let todo: Vec<String> = w
                .entry_points
                .discovered
                .iter()
                .filter(|e| !exercised.contains(*e))
                .cloned()
                .collect();
            w.entry_points
                .not_exercisable
                .retain(|n| !todo.contains(&n.entry));
            w.entry_points
                .not_exercisable
                .extend(todo.into_iter().map(|entry| NotExercisable {
                    entry,
                    reason: (*reason).to_owned(),
                }));
        }
    })
    .map_err(|e| e.to_refusal())?;
    Ok(limitations)
}

/// What to record in coverage after a run.
#[derive(Debug)]
pub enum CoverageOutcome<'a> {
    /// Points were measured.
    Ran {
        /// Post-warm-up points times step (or units).
        iterations: u64,
        /// Coverage sections of the measured processes' records.
        dump_cov: Vec<DumpCoverage>,
        /// Harness directory.
        hdir: &'a Path,
        /// `cov/` directory.
        cov_dir: &'a Path,
        /// Scratch project.
        work: &'a Path,
    },
    /// The build failed or no point produced data: `reason` is `build_fails` or
    /// `no_harness`.
    Failed {
        /// The `not_exercisable` reason.
        reason: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn d(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn jvm_classes_map_to_the_entries_of_their_source_file() {
        let discovered = d(&[
            "src/shop/OrderCache.java::OrderCache.store",
            "src/shop/Catalog.java::Catalog.find",
            "src/shop/Order.java::Order.Line.total",
        ]);
        let loaded: BTreeSet<String> = ["shop.OrderCache", "shop.Order$Line"]
            .map(String::from)
            .into();
        let got = jvm_entries(&discovered, &loaded);
        assert!(got.contains("src/shop/OrderCache.java::OrderCache.store"));
        assert!(got.contains("src/shop/Order.java::Order.Line.total"));
        assert!(!got.iter().any(|e| e.contains("Catalog")));
    }

    #[test]
    fn python_items_become_entries_by_file_and_qualname() {
        let disc: BTreeSet<&str> = [
            "reports/builder.py::ReportBuilder.category_totals",
            "reports/service.py::handle",
        ]
        .into();
        let items: BTreeSet<String> = [
            "reports.builder:ReportBuilder.category_totals@reports/builder.py:18",
            "reports.service:handle@reports/service.py:1",
            "reports.service:handle.<locals>.inner@reports/service.py:3",
            "reports.other:f@reports/other.py:1",
        ]
        .map(String::from)
        .into();
        let got = python_entries(&disc, &items, Path::new("/w"));
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn file_urls_decode_and_drop_the_slash_before_a_drive() {
        assert_eq!(file_url_path("file:///C:/a%20b/x.js").as_deref(), Some("C:/a b/x.js"));
        assert_eq!(file_url_path("file:///home/u/x.js").as_deref(), Some("/home/u/x.js"));
        assert_eq!(file_url_path("node:internal/x"), None);
        assert_eq!(comparable(r"\\?\C:\W\"), if cfg!(windows) { "c:/w" } else { "C:/W" });
    }

    #[test]
    fn node_v8_files_give_functions_with_calls() {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().canonicalize().unwrap();
        let cov = work.join("cov");
        fs::create_dir_all(&cov).unwrap();
        // As V8 writes it: forward slashes, and `file:///C:/...` on Windows.
        let shown = work.to_string_lossy().replace('\\', "/");
        let shown = shown.strip_prefix("//?/").unwrap_or(&shown).to_owned();
        let url = if shown.starts_with('/') {
            format!("file://{shown}/src/heartbeat.js")
        } else {
            format!("file:///{shown}/src/heartbeat.js")
        };
        fs::write(
            cov.join("coverage-1.json"),
            json!({"result":[{"url":url,"functions":[
                {"functionName":"startHeartbeat","ranges":[{"count":5}]},
                {"functionName":"neverCalled","ranges":[{"count":0}]},
                {"functionName":"","ranges":[{"count":9}]}]},
                {"url":"node:internal/x","functions":[{"functionName":"f","ranges":[{"count":1}]}]}]})
            .to_string(),
        )
        .unwrap();
        let fns = node_functions(&cov, &work);
        assert_eq!(fns.len(), 1);
        assert!(fns.contains(&("src/heartbeat.js".to_owned(), "startHeartbeat".to_owned())));
        let ex = exercised(
            Runtime::Node,
            &d(&[
                "src/heartbeat.js::startHeartbeat",
                "src/heartbeat.js::neverCalled",
            ]),
            &[],
            &work,
            &cov,
            &work,
        );
        assert_eq!(ex.entries.len(), 1);
    }

    #[test]
    fn rust_and_native_measure_nothing_and_say_so() {
        let ex = exercised(
            Runtime::Native,
            &d(&["src/a.c::f"]),
            &[],
            Path::new("/h"),
            Path::new("/c"),
            Path::new("/w"),
        );
        assert!(ex.entries.is_empty());
        assert!(ex.limitations[0].contains("not measured"));
    }
}
