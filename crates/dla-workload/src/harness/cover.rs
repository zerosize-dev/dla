//! Exercised entry points from coverage data.
//!
//! A harness run exercises the entry points that `dla workload discover` listed in
//! `coverage.json` (`file::Owner.name`). What ran is read from the agents' own data:
//!
//! | Runtime | Source | Match |
//! |---|---|---|
//! | JVM | loaded classes and the source files of the project's classes in the dump | the entry's file is the class's source file (by path, or by the `SourceFile` attribute) |
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
    /// JVM: distinct top-level classes of the project that were loaded, when the agent says.
    pub project_classes: Option<u64>,
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
    let p = path
        .strip_prefix(r"\\?\")
        .unwrap_or(path)
        .replace('\\', "/");
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

/// The directory part of a `/` separated path (`""` when there is none).
fn dir_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(d, _)| d)
}

/// The last component of a `/` separated path.
fn base_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// The JVM class a Kotlin file's top-level functions compile to: the `@file:JvmName` the
/// file declares, else the file name with `Kt` (`Util.kt` gives `UtilKt`).
fn facade_class(work: &Path, file: &str, stem_name: &str) -> String {
    let named = fs::read(work.join(file)).ok().and_then(|bytes| {
        let head = String::from_utf8_lossy(&bytes[..bytes.len().min(16 * 1024)]).into_owned();
        let at = head.find("@file:JvmName(")?;
        let rest = &head[at + "@file:JvmName(".len()..];
        let rest = rest.trim_start();
        let rest = rest
            .strip_prefix("name")
            .map(|r| r.trim_start().trim_start_matches('=').trim_start())
            .unwrap_or(rest);
        let rest = rest.strip_prefix('"')?;
        let name = &rest[..rest.find('"')?];
        (!name.is_empty()).then(|| name.to_owned())
    });
    named.unwrap_or_else(|| format!("{stem_name}Kt"))
}

/// The package of a source file as a `/` path (`com/acme/shop`, `""` for the default package),
/// read from the file itself; `None` when the file cannot be read.
fn declared_package(work: &Path, file: &str) -> Option<String> {
    let path = work.join(file);
    fs::metadata(&path).ok()?;
    Some(package_of(&path).unwrap_or_default().replace('.', "/"))
}

/// The discovered entry points whose file a loaded class came from.
///
/// A class belongs to a file in three ways, all by names the project's own files and the
/// loaded classes share (never by a name alone: `kotlin.text.StringsKt` is not the facade of
/// a project's `Strings.kt`):
/// - the file's path ends with the class's package path and name (the Java layout);
/// - the class's `SourceFile` attribute (`sources`, as `package/path/File.kt`) names a file
///   that declares that package, and the entry's owner class is itself loaded from that
///   package (a Kotlin file need not hold the class it is named for, nor sit in the
///   directory of its package);
/// - for a top-level function, the file's facade class (`UtilKt`, or the `@file:JvmName`)
///   is loaded from the file's own package.
fn jvm_entries(
    discovered: &[String],
    loaded: &BTreeSet<String>,
    sources: &BTreeSet<String>,
    work: &Path,
) -> BTreeSet<String> {
    // Top-level class paths (`shop/OrderCache`) and simple names of every loaded class.
    let tops: BTreeSet<String> = loaded
        .iter()
        .map(|c| c.split('$').next().unwrap_or(c).replace(['.', '/'], "/"))
        .collect();
    let simple: BTreeSet<&str> = loaded
        .iter()
        .filter_map(|c| c.rsplit(['.', '$', '/']).next())
        .collect();
    // Does a loaded class (or one nested in it) have this dotted name?
    let is_loaded = |name: &str| {
        loaded
            .iter()
            .any(|c| c == name || c.strip_prefix(name).is_some_and(|r| r.starts_with('$')))
    };
    discovered
        .iter()
        .filter(|id| {
            let Some((file, _)) = id.split_once("::") else {
                return false;
            };
            let stem = file.rsplit_once('.').map_or(file, |(s, _)| s);
            let stem_name = stem.rsplit('/').next().unwrap_or(stem);
            let qualified = qualified(id).unwrap_or_default();
            let package = declared_package(work, file);
            let Some((owner, _)) = qualified.rsplit_once('.') else {
                // A top-level function: it ran when the file's facade class is loaded.
                let Some(package) = package else {
                    return false;
                };
                let facade = facade_class(work, file, stem_name);
                let fqn = if package.is_empty() {
                    facade
                } else {
                    format!("{}/{facade}", package)
                };
                return tops.contains(&fqn);
            };
            let owner_simple = owner.rsplit('.').next().unwrap_or(owner);
            let by_path = tops
                .iter()
                .any(|t| stem == t || stem.ends_with(&format!("/{t}")))
                && (owner_simple == stem_name || simple.contains(owner_simple));
            let by_source = package.is_some_and(|package| {
                sources
                    .iter()
                    .any(|s| base_of(s) == base_of(file) && dir_of(s) == package)
                    && is_loaded(&if package.is_empty() {
                        owner.replace('.', "$")
                    } else {
                        format!("{}.{}", package.replace('/', "."), owner.replace('.', "$"))
                    })
            });
            by_path || by_source
        })
        .cloned()
        .collect()
}

/// The packages the project's own JVM sources declare, as the comma list `DLA_PROJECT_PACKAGES`
/// carries: every `package` of a non-test `.java` or `.kt` file under `work`, without the
/// packages inside another one listed (`com.acme` stands for `com.acme.shop`). Bounded in
/// files read and packages listed.
pub fn jvm_project_packages(work: &Path) -> String {
    // `buildSrc` holds build logic (kept in the scratch copy so the build runs), not the
    // program: its packages are not the project's.
    const SKIP: [&str; 10] = [
        "buildSrc",
        "build",
        "out",
        "target",
        "node_modules",
        "test",
        "tests",
        "testFixtures",
        "integrationTest",
        "androidTest",
    ];
    // Roots shared by unrelated code: never a project's package on their own.
    const SHARED: [&str; 12] = [
        "com", "org", "net", "io", "java", "javax", "jakarta", "kotlin", "kotlinx", "android",
        "androidx", "sun",
    ];
    let mut found: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![work.to_path_buf()];
    let mut files = 0usize;
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            let name = e.file_name().to_string_lossy().into_owned();
            let path = e.path();
            if ft.is_dir() {
                if !name.starts_with('.') && !SKIP.contains(&name.as_str()) {
                    stack.push(path);
                }
            } else if ft.is_file()
                && (name.ends_with(".java") || name.ends_with(".kt"))
                && files < 20_000
            {
                files += 1;
                if let Some(p) = package_of(&path) {
                    found.insert(p);
                }
            }
        }
    }
    let all: Vec<String> = found.into_iter().collect();
    let mut kept: Vec<String> = Vec::new();
    for p in &all {
        let covered = all
            .iter()
            .any(|q| q != p && p.starts_with(&format!("{q}.")) && !SHARED.contains(&q.as_str()));
        if !covered && !SHARED.contains(&p.as_str()) {
            kept.push(p.clone());
        }
    }
    kept.truncate(200);
    kept.join(",")
}

/// The `package` of a Java or Kotlin source file, read from its head.
fn package_of(path: &Path) -> Option<String> {
    use std::io::Read;
    let mut head = Vec::new();
    fs::File::open(path)
        .ok()?
        .take(8 * 1024)
        .read_to_end(&mut head)
        .ok()?;
    let text = String::from_utf8_lossy(&head);
    // A byte order mark in front of the first line is not part of it.
    let text = text.trim_start_matches('\u{feff}');
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("package ") {
            // `package com.acme // core` and `package com.acme /* core */`.
            let rest = rest.split("//").next().unwrap_or(rest);
            let rest = rest.split("/*").next().unwrap_or(rest);
            let name: String = rest
                .trim()
                .trim_end_matches(';')
                .trim()
                .trim_matches('`')
                .to_owned();
            let ok = !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '.');
            return ok.then_some(name);
        }
        if line.starts_with("import ") {
            return None;
        }
    }
    None
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
            let sources: BTreeSet<String> = dump_cov
                .iter()
                .flat_map(|c| c.sources.iter().cloned())
                .collect();
            res.project_classes = dump_cov.iter().filter_map(|c| c.project).max();
            res.entries = jvm_entries(discovered, &items, &sources, &work);
            if let Some(n) = res
                .project_classes
                .filter(|n| *n > 0 && res.entries.is_empty())
            {
                let loaded = if n == 1 {
                    "1 project class was".to_owned()
                } else {
                    format!("{n} project classes were")
                };
                res.limitations.push(format!(
                    "{loaded} loaded, but none holds a discovered entry point: the run executed project code outside the listed entry points (a class the driver or the tests call that no entry point names, or a file this match cannot place)"
                ));
            }
        }
        Runtime::Python => {
            let (items, notes) = dump_items(dump_cov, hdir);
            res.limitations.extend(notes);
            // An agent on 3.12+ always writes a `functions` section, empty or not: an empty one
            // is a driver that ran no listed code, not a missing monitor.
            let reported = dump_cov
                .iter()
                .any(|c| c.kind.as_deref() == Some("functions"));
            if items.is_empty() && !reported {
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
    let mut project_classes = None;
    let mut agent_blind = false;
    let found = match outcome {
        CoverageOutcome::Ran {
            dump_cov,
            hdir,
            cov_dir,
            work,
            ..
        } => {
            let ex = exercised(runtime, &discovered, dump_cov, hdir, cov_dir, work);
            agent_blind = runtime == Runtime::Python
                && ex
                    .limitations
                    .iter()
                    .any(|l| l.contains("sys.monitoring needs Python 3.12"));
            limitations.extend(ex.limitations);
            project_classes = ex.project_classes;
            ex.entries
        }
        CoverageOutcome::Failed { .. } => BTreeSet::new(),
    };
    CoverageManifest::update(&ctx.run_dir, |m| match outcome {
        CoverageOutcome::Ran { iterations, .. } => {
            m.set_iterations(worker, *iterations);
            if let Some(n) = project_classes {
                m.set_project_classes(worker, n);
            }
            m.add_exercised(worker, found.iter().cloned());
            if matches!(runtime, Runtime::Rust | Runtime::Native) {
                m.add_blind_spot(
                    worker,
                    "the allocator shim does not record which entry points ran",
                );
            }
            // A Python before 3.12 has no `sys.monitoring`: the empty list is a limit
            // of the interpreter, not a driver that missed the code.
            if agent_blind {
                m.add_blind_spot(
                    worker,
                    "the Python agent cannot say which entry points ran (sys.monitoring needs Python 3.12 or newer)",
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

    /// A Python before 3.12 reports no executed function: the empty result is a limit of
    /// the interpreter, recorded as a blind spot so the gate and the report do not read it
    /// as a driver that missed the code. A Python that did report functions records none.
    #[test]
    fn a_python_without_function_coverage_records_a_blind_spot() {
        // (items, kind of the section the agent wrote, blind spot expected). An empty
        // `functions` section is an agent that watched and saw nothing run: not blind.
        for (items, kind, blind) in [
            (vec![], None, true),
            (vec!["m.py::f".to_owned()], None, false),
            (vec![], Some("functions".to_owned()), false),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let run_dir = dir.path().join("run");
            fs::create_dir_all(&run_dir).unwrap();
            let ctx = ToolCtx {
                project: dir.path().to_path_buf(),
                run_dir: run_dir.clone(),
                run_id: "r".into(),
                source_commit: "c".into(),
                data_flag: None,
                worker: None,
            };
            CoverageManifest::update(&run_dir, |m| {
                m.set_entry_points(
                    "python",
                    dla_core::coverage::EntryPoints {
                        discovered: vec!["m.py::f".into(), "m.py::g".into()],
                        exercised: vec![],
                        not_exercisable: vec![],
                    },
                );
            })
            .unwrap();
            let dump = vec![DumpCoverage {
                items,
                kind,
                ..DumpCoverage::default()
            }];
            record(
                &ctx,
                Runtime::Python,
                &CoverageOutcome::Ran {
                    iterations: 3520,
                    dump_cov: dump,
                    hdir: dir.path(),
                    cov_dir: dir.path(),
                    work: dir.path(),
                },
            )
            .unwrap();
            let m = CoverageManifest::load(&run_dir).unwrap().unwrap();
            let got = m
                .blind_spots
                .iter()
                .any(|b| b.runtime == "python" && b.text.contains("which entry points ran"));
            assert_eq!(got, blind);
        }
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
        let got = jvm_entries(&discovered, &loaded, &BTreeSet::new(), Path::new("/w"));
        assert!(got.contains("src/shop/OrderCache.java::OrderCache.store"));
        assert!(got.contains("src/shop/Order.java::Order.Line.total"));
        assert!(!got.iter().any(|e| e.contains("Catalog")));
    }

    fn set(v: &[&str]) -> BTreeSet<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    const KOTLIN_ENTRIES: [&str; 7] = [
        "src/main/kotlin/billing/Invoices.kt::Invoices.total",
        "src/main/kotlin/com/acme/shop/Routes.kt::UserController.ping",
        "src/main/kotlin/com/acme/shop/Routes.kt::UserController.user",
        "src/main/kotlin/com/acme/shop/Routes.kt::routeCount",
        "src/main/kotlin/com/acme/shop/UserService.kt::UserService.lookup",
        "src/main/kotlin/com/acme/shop/UserService.kt::UserService.size",
        "src/main/kotlin/com/acme/shop/Util.kt::slug",
    ];

    /// A scratch copy holding `files` (path, text), as the harness's work directory does.
    fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (path, text) in files {
            let full = dir.path().join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, text).unwrap();
        }
        dir
    }

    fn shop() -> tempfile::TempDir {
        project(&[
            (
                "src/main/kotlin/billing/Invoices.kt",
                "package com.acme.shop.billing\n\nclass Invoices { fun total() = 1 }\n",
            ),
            (
                "src/main/kotlin/com/acme/shop/Routes.kt",
                "package com.acme.shop\n\nclass UserController { fun ping() = 1 }\nfun routeCount() = 2\n",
            ),
            (
                "src/main/kotlin/com/acme/shop/UserService.kt",
                "package com.acme.shop\n\nclass UserService\n",
            ),
            (
                "src/main/kotlin/com/acme/shop/Util.kt",
                "package com.acme.shop\n\nfun slug(s: String) = s\n",
            ),
        ])
    }

    #[test]
    fn a_kotlin_file_holding_a_class_of_another_name_is_matched_by_its_source_file() {
        // `Routes.kt` holds `UserController`: no loaded class path ends with `.../Routes`.
        let dir = shop();
        let loaded = set(&[
            "com.acme.shop.UserController",
            "com.acme.shop.UserController$ping$1",
        ]);
        let sources = set(&["com/acme/shop/Routes.kt"]);
        let got = jvm_entries(&d(&KOTLIN_ENTRIES), &loaded, &sources, dir.path());
        assert_eq!(
            got,
            set(&[
                "src/main/kotlin/com/acme/shop/Routes.kt::UserController.ping",
                "src/main/kotlin/com/acme/shop/Routes.kt::UserController.user",
            ]),
            "the top-level routeCount needs RoutesKt, and the other files are not loaded"
        );
        // Without the sources (an older agent) the old rule finds nothing: the defect.
        let got = jvm_entries(&d(&KOTLIN_ENTRIES), &loaded, &BTreeSet::new(), dir.path());
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn a_class_of_the_file_that_is_not_the_entry_owner_does_not_run_the_entry() {
        // `Reports.kt` holds `class Reports` (never used) and `data class ReportRow` (loaded).
        let dir = project(&[(
            "src/main/kotlin/com/acme/shop/Reports.kt",
            "package com.acme.shop\n\nclass Reports { fun build() = 1 }\ndata class ReportRow(val a: Int)\n",
        )]);
        let disc = d(&["src/main/kotlin/com/acme/shop/Reports.kt::Reports.build"]);
        let sources = set(&["com/acme/shop/Reports.kt"]);
        let got = jvm_entries(
            &disc,
            &set(&["com.acme.shop.ReportRow"]),
            &sources,
            dir.path(),
        );
        assert!(got.is_empty(), "{got:?}");
        let got = jvm_entries(
            &disc,
            &set(&["com.acme.shop.Reports"]),
            &sources,
            dir.path(),
        );
        assert_eq!(got.len(), 1);
        // The same for Java: a package-private helper of `Order.java` is not `Order`.
        let dir = project(&[(
            "src/main/java/shop/Order.java",
            "package shop;\n\npublic class Order { int total() { return 1; } }\nclass OrderHelper {}\n",
        )]);
        let disc = d(&["src/main/java/shop/Order.java::Order.total"]);
        let got = jvm_entries(
            &disc,
            &set(&["shop.OrderHelper"]),
            &set(&["shop/Order.java"]),
            dir.path(),
        );
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn a_kotlin_top_level_function_runs_when_its_facade_class_is_loaded() {
        let dir = shop();
        let disc = d(&KOTLIN_ENTRIES);
        let got = jvm_entries(
            &disc,
            &set(&["com.acme.shop.UtilKt"]),
            &set(&["com/acme/shop/Util.kt"]),
            dir.path(),
        );
        assert_eq!(got, set(&["src/main/kotlin/com/acme/shop/Util.kt::slug"]));
        // A class named like the file is not the facade: `Util` alone does not run `slug`.
        let got = jvm_entries(
            &disc,
            &set(&["com.acme.shop.Util"]),
            &set(&["com/acme/shop/Util.kt"]),
            dir.path(),
        );
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn a_library_class_named_like_a_facade_does_not_run_a_project_function() {
        // `kotlin.text.StringsKt` is loaded in every Kotlin program; the project's own
        // `Strings.kt` (package com.acme.text) never ran its `padLeft`.
        let dir = project(&[
            (
                "src/main/kotlin/com/acme/text/Strings.kt",
                "package com.acme.text\n\nfun padLeft(s: String) = s\n",
            ),
            (
                "src/main/kotlin/com/acme/other/Strings.kt",
                "package com.acme.other\n\nfun trim2(s: String) = s\n",
            ),
            (
                "src/main/kotlin/com/acme/Extensions.kt",
                "package com.acme\n\nfun toJson(s: String) = s\n",
            ),
        ]);
        let disc = d(&[
            "src/main/kotlin/com/acme/text/Strings.kt::padLeft",
            "src/main/kotlin/com/acme/other/Strings.kt::trim2",
            "src/main/kotlin/com/acme/Extensions.kt::toJson",
        ]);
        let loaded = set(&[
            "kotlin.text.StringsKt",
            "com.fasterxml.jackson.module.kotlin.ExtensionsKt",
        ]);
        let got = jvm_entries(&disc, &loaded, &BTreeSet::new(), dir.path());
        assert!(got.is_empty(), "{got:?}");
        // The project's own facade, loaded from the file's package, does run it.
        let got = jvm_entries(
            &disc,
            &set(&["com.acme.text.StringsKt", "kotlin.text.StringsKt"]),
            &BTreeSet::new(),
            dir.path(),
        );
        assert_eq!(
            got,
            set(&["src/main/kotlin/com/acme/text/Strings.kt::padLeft"])
        );
    }

    #[test]
    fn a_file_jvm_name_renames_the_facade_class() {
        let dir = project(&[(
            "src/main/kotlin/com/acme/shop/Util.kt",
            "@file:JvmName(\"Strings\")\n\npackage com.acme.shop\n\nfun slug(s: String) = s\n",
        )]);
        let disc = d(&["src/main/kotlin/com/acme/shop/Util.kt::slug"]);
        let sources = set(&["com/acme/shop/Util.kt"]);
        let got = jvm_entries(
            &disc,
            &set(&["com.acme.shop.Strings"]),
            &sources,
            dir.path(),
        );
        assert_eq!(got.len(), 1, "{got:?}");
        // The default facade name is no longer the one that holds the function.
        let got = jvm_entries(&disc, &set(&["com.acme.shop.UtilKt"]), &sources, dir.path());
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn a_kotlin_package_that_omits_the_root_of_its_directory_is_matched() {
        // `billing/Invoices.kt` declares `package com.acme.shop.billing`.
        let dir = shop();
        let loaded = set(&["com.acme.shop.billing.Invoices"]);
        let sources = set(&["com/acme/shop/billing/Invoices.kt"]);
        let got = jvm_entries(&d(&KOTLIN_ENTRIES), &loaded, &sources, dir.path());
        assert_eq!(
            got,
            set(&["src/main/kotlin/billing/Invoices.kt::Invoices.total"])
        );
        // The old path rule cannot see it.
        let got = jvm_entries(&d(&KOTLIN_ENTRIES), &loaded, &BTreeSet::new(), dir.path());
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn files_of_one_name_are_told_apart_by_the_package_they_declare() {
        let dir = project(&[
            (
                "src/main/kotlin/a/Config.kt",
                "package x.a\n\nclass Config { fun load() = 1 }\n",
            ),
            (
                "src/main/kotlin/b/Config.kt",
                "package x.b\n\nclass Config { fun load() = 1 }\n",
            ),
            (
                "src/main/kotlin/domain/User.kt",
                "package com.acme.shop.domain\n\nclass User { fun rename() = 1 }\n",
            ),
        ]);
        let disc = d(&[
            "src/main/kotlin/a/Config.kt::Config.load",
            "src/main/kotlin/b/Config.kt::Config.load",
            "src/main/kotlin/domain/User.kt::User.rename",
        ]);
        let got = jvm_entries(
            &disc,
            &set(&["x.b.Config"]),
            &set(&["x/b/Config.kt"]),
            dir.path(),
        );
        assert_eq!(got, set(&["src/main/kotlin/b/Config.kt::Config.load"]));
        // A package no file of that name declares places nothing, even for a name used once:
        // `com.acme.shop.api.User` is another class than the discovered `domain.User`.
        let got = jvm_entries(
            &disc,
            &set(&["x.c.Config", "com.acme.shop.api.User"]),
            &set(&["x/c/Config.kt", "com/acme/shop/api/User.kt"]),
            dir.path(),
        );
        assert!(got.is_empty(), "{got:?}");
        // A name used once with the root of its package omitted is placed by what it declares.
        let got = jvm_entries(
            &disc,
            &set(&["com.acme.shop.domain.User"]),
            &set(&["com/acme/shop/domain/User.kt"]),
            dir.path(),
        );
        assert_eq!(got, set(&["src/main/kotlin/domain/User.kt::User.rename"]));
    }

    #[test]
    fn a_package_line_with_a_comment_or_a_byte_order_mark_is_still_read() {
        let dir = project(&[
            (
                "src/main/kotlin/x/Commented.kt",
                "package com.acme.shop // core\n\nclass Svc { fun run() = 1 }\nfun slug() = 1\n",
            ),
            (
                "src/main/kotlin/y/Block.kt",
                "package com.acme.shop /* core */\n\nclass Blk { fun go() = 1 }\n",
            ),
            (
                "src/main/kotlin/z/Bom.kt",
                "\u{feff}package com.acme.bom\n\nclass Bom { fun go() = 1 }\nfun bomTop() = 1\n",
            ),
        ]);
        let disc = d(&[
            "src/main/kotlin/x/Commented.kt::Svc.run",
            "src/main/kotlin/x/Commented.kt::slug",
            "src/main/kotlin/y/Block.kt::Blk.go",
            "src/main/kotlin/z/Bom.kt::Bom.go",
            "src/main/kotlin/z/Bom.kt::bomTop",
        ]);
        let loaded = set(&[
            "com.acme.shop.Svc",
            "com.acme.shop.CommentedKt",
            "com.acme.shop.Blk",
            "com.acme.bom.Bom",
            "com.acme.bom.BomKt",
        ]);
        let sources = set(&[
            "com/acme/shop/Commented.kt",
            "com/acme/shop/Block.kt",
            "com/acme/bom/Bom.kt",
        ]);
        let got = jvm_entries(&disc, &loaded, &sources, dir.path());
        assert_eq!(
            got,
            set(&disc.iter().map(String::as_str).collect::<Vec<_>>())
        );
        assert_eq!(
            jvm_project_packages(dir.path()),
            "com.acme.bom,com.acme.shop"
        );
    }

    #[test]
    fn the_java_layout_still_matches_without_sources() {
        let disc = d(&["src/main/java/shop/OrderCache.java::OrderCache.store"]);
        let got = jvm_entries(
            &disc,
            &set(&["shop.OrderCache"]),
            &BTreeSet::new(),
            Path::new("/w"),
        );
        assert_eq!(got.len(), 1);
        // A class of the same simple name in another package does not place it.
        let got = jvm_entries(
            &disc,
            &set(&["other.Thing"]),
            &set(&["other/Thing.java"]),
            Path::new("/w"),
        );
        assert!(got.is_empty());
    }

    #[test]
    fn project_packages_come_from_the_non_test_sources_and_nest() {
        let dir = tempfile::tempdir().unwrap();
        let w = dir.path();
        for (rel, text) in [
            (
                "src/main/kotlin/com/acme/shop/A.kt",
                "package com.acme.shop\n\nclass A\n",
            ),
            (
                "src/main/kotlin/billing/B.kt",
                "package com.acme.shop.billing\nclass B\n",
            ),
            (
                "src/main/java/org/other/C.java",
                "/* c */\npackage org.other;\npublic class C {}\n",
            ),
            ("src/main/java/lone/D.java", "package lone;\nclass D {}\n"),
            ("src/test/kotlin/com/acme/T.kt", "package tests.only\n"),
            ("build/gen/G.java", "package generated;\n"),
            ("buildSrc/src/main/kotlin/Conv.kt", "package conventions\n"),
            ("src/main/kotlin/Top.kt", "package com\nclass Top\n"),
            ("src/main/kotlin/NoPkg.kt", "import a.b\npackage late\n"),
        ] {
            let p = w.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        // `com.acme.shop.billing` is inside `com.acme.shop`; a bare `com` is never a package of
        // its own; tests, build output and `buildSrc` are not the project.
        assert_eq!(jvm_project_packages(w), "com.acme.shop,lone,org.other");
    }

    #[test]
    fn loaded_project_classes_without_a_matched_entry_are_counted_and_said() {
        let dump = vec![DumpCoverage {
            kind: Some("classes".into()),
            items: vec!["com.acme.shop.Elsewhere".into()],
            project: Some(37),
            sources: vec!["com/acme/shop/Elsewhere.kt".into()],
            ..Default::default()
        }];
        let ex = exercised(
            Runtime::Jvm,
            &d(&KOTLIN_ENTRIES),
            &dump,
            Path::new("/h"),
            Path::new("/c"),
            Path::new("/w"),
        );
        assert!(ex.entries.is_empty());
        assert_eq!(ex.project_classes, Some(37));
        assert!(
            ex.limitations
                .iter()
                .any(|l| l.contains("37 project classes were loaded")),
            "{:?}",
            ex.limitations
        );
        // One loaded class says it in the singular.
        let dump = vec![DumpCoverage {
            kind: Some("classes".into()),
            items: vec!["com.acme.shop.Elsewhere".into()],
            project: Some(1),
            sources: vec!["com/acme/shop/Elsewhere.kt".into()],
            ..Default::default()
        }];
        let ex = exercised(
            Runtime::Jvm,
            &d(&KOTLIN_ENTRIES),
            &dump,
            Path::new("/h"),
            Path::new("/c"),
            Path::new("/w"),
        );
        assert!(
            ex.limitations
                .iter()
                .any(|l| l.starts_with("1 project class was loaded")),
            "{:?}",
            ex.limitations
        );
        // An entry that matched needs no such note.
        let dump = vec![DumpCoverage {
            items: vec!["com.acme.shop.UserService".into()],
            project: Some(2),
            sources: vec!["com/acme/shop/UserService.kt".into()],
            ..Default::default()
        }];
        let ex = exercised(
            Runtime::Jvm,
            &d(&KOTLIN_ENTRIES),
            &dump,
            Path::new("/h"),
            Path::new("/c"),
            Path::new("/w"),
        );
        assert_eq!(ex.entries.len(), 2);
        assert!(ex.limitations.is_empty(), "{:?}", ex.limitations);
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
        assert_eq!(
            file_url_path("file:///C:/a%20b/x.js").as_deref(),
            Some("C:/a b/x.js")
        );
        assert_eq!(
            file_url_path("file:///home/u/x.js").as_deref(),
            Some("/home/u/x.js")
        );
        assert_eq!(file_url_path("node:internal/x"), None);
        assert_eq!(
            comparable(r"\\?\C:\W\"),
            if cfg!(windows) { "c:/w" } else { "C:/W" }
        );
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
