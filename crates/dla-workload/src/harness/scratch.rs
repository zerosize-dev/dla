//! The scratch copy of the project a harness runs in (`work/`).
//!
//! DLA never edits project sources: the copy is the project's tracked files (or, outside a
//! git repository, every file except the build-output directories of
//! [`crate::model::SKIP_DIRS`] and `.dla`), plus whatever generation drops in.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use dla_census::parse::Lang;
use dla_core::{Refusal, runtime::Runtime, state::Scope};
use serde::{Deserialize, Serialize};

use super::WORK;

use crate::model::SKIP_DIRS;
#[cfg(unix)]
use dla_probe::project_copy::repointed_link;
use dla_probe::project_copy::{
    git_files, git_tracked, is_dropped_build_output, link_for_copy, under_root_build_dir,
};

/// Most files copied into a scratch tree.
const MAX_FILES: usize = 100_000;

/// What was copied.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CopyReport {
    /// Files copied.
    pub files: usize,
    /// True when the file list came from `git ls-files`.
    pub from_git: bool,
    /// Entries skipped (symlinks, unreadable files, over the cap).
    pub skipped: usize,
    /// One manifest gap per reason something was not copied.
    pub gaps: Vec<String>,
    /// Files left out because they lie outside the run's scope.
    pub out_of_scope: usize,
    /// Non-test source files among them, by runtime.
    pub out_of_scope_sources: BTreeMap<Runtime, u64>,
    /// Python harness: built extension modules (untracked `*.so`, `*.pyd`, `*.dylib` next to
    /// Python sources) that were kept.
    pub extensions_kept: usize,
    /// Node harness: files of build-output directories that git ignores and that a
    /// `package.json` names as entry points (`main`, `module`, `bin`, `exports`, `files`).
    pub build_output_kept: usize,
    /// Node harness: `node_modules` directories linked (not copied) into the copy.
    pub node_modules_linked: usize,
}

/// Directories skipped wherever they are: they never hold project sources.
const SKIP_ANYWHERE: [&str; 4] = ["node_modules", "__pycache__", ".git", ".dla"];

/// Copies one file with its permissions and its modification time, so a copy builds the
/// same as its source: `make` compares the times of sources, headers and outputs, and a
/// copy that stamps every file with the time and order of the copy changes what is rebuilt.
/// The content goes through a handle opened for writing and the time is set on that handle;
/// the permissions are copied last, so a read-only source (mode 0444) keeps its time on
/// every platform too.
fn copy_file(from: &Path, to: &Path) -> std::io::Result<()> {
    let meta = fs::metadata(from)?;
    let mut src = fs::File::open(from)?;
    let mut dst = fs::File::create(to)?;
    std::io::copy(&mut src, &mut dst)?;
    if let Ok(t) = meta.modified() {
        // Best effort: a copy with a fresh time is still a copy.
        let _ = dst.set_modified(t);
    }
    drop(dst);
    fs::set_permissions(to, meta.permissions())
}

/// Directories of [`SKIP_DIRS`] that are skipped only at the project root: a package named
/// `build` or `out` deeper in the tree is source.
fn skip_at_root(name: &str) -> bool {
    SKIP_DIRS.contains(&name) && !SKIP_ANYWHERE.contains(&name)
}

/// Every file and symlink under `root` except skipped directories, relative, sorted; and
/// the root-level directories left out.
fn walk_files(root: &Path) -> (Vec<PathBuf>, Vec<String>) {
    let mut out = Vec::new();
    let mut left_out = Vec::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let Ok(rd) = fs::read_dir(root.join(&rel)) else {
            continue;
        };
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            let name = e.file_name();
            let name_s = name.to_string_lossy();
            let child = rel.join(&name);
            if ft.is_dir() {
                if SKIP_ANYWHERE.contains(&name_s.as_ref()) {
                    continue;
                }
                if rel.as_os_str().is_empty() && skip_at_root(&name_s) {
                    left_out.push(name_s.into_owned());
                    continue;
                }
                stack.push(child);
            } else if ft.is_file() || ft.is_symlink() {
                out.push(child);
            }
        }
    }
    out.sort();
    left_out.sort();
    (out, left_out)
}

fn is_dla_path(rel: &Path) -> bool {
    rel.components()
        .next()
        .is_some_and(|c| c.as_os_str() == ".dla")
}

/// Recreates the symlink `src` at `to` when it points inside `project`; the reason it was
/// not copied otherwise.
fn copy_symlink(project: &Path, dest: &Path, src: &Path, to: &Path) -> Result<(), String> {
    let link = link_for_copy(project, dest, src)?;
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&link, to).map_err(|e| format!("cannot create symlink: {e}"))
    }
    #[cfg(not(unix))]
    {
        let _ = (link, to);
        Err("symlinks are not copied on this platform".to_owned())
    }
}

/// Whether a file outside the scope is still needed to build what is inside it: any
/// non-source file in a directory above the scope (`svc/go.mod` for the scope
/// `svc/a`, a parent `pom.xml` or `Cargo.toml`) and everything under the root-level
/// build directories. Build logic (Gradle scripts, `buildSrc`) is always kept, inside the
/// scope or not: the build needs it, and the run never measures it.
fn keeps_for_build(scope: &Scope, rel: &Path, slash: &str) -> bool {
    if under_root_build_dir(rel) || dla_core::paths::is_build_logic(slash) {
        return true;
    }
    if Lang::from_path(rel).is_some() {
        return false;
    }
    let root: Vec<&str> = scope
        .path
        .split(['/', '\\'])
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    let dir: Vec<&str> = slash.split('/').collect();
    let dir = &dir[..dir.len() - 1];
    dir.len() < root.len() && root[..dir.len()] == *dir
}

/// Copies the scratch tree `src` into `dest` (created) without build outputs: the
/// directories of [`SKIP_ANYWHERE`] wherever they are, the root-level ones of
/// [`SKIP_DIRS`], and what [`is_dropped_build_output`] leaves out: object files git does
/// not track in `project` (every object outside git) and untracked archives and shared
/// libraries (the project copy's rule: a shipped input stays, a build output goes). Files
/// keep their modification times. Symlinks are recreated as they are on Unix and skipped
/// elsewhere. `dla intervene` clones a harness's `work/` this way. Returns the files copied.
pub fn copy_tree(src: &Path, dest: &Path, project: &Path) -> std::io::Result<usize> {
    copy_tree_for(src, dest, project, None)
}

/// Like [`copy_tree`], for the clone of a harness of `runtime`: a Python harness keeps the built
/// extension modules its copy holds (see [`copy_project_for`]), because a clone has no build step
/// either.
pub fn copy_tree_for(
    src: &Path,
    dest: &Path,
    project: &Path,
    runtime: Option<Runtime>,
) -> std::io::Result<usize> {
    let python = runtime == Some(Runtime::Python);
    let mut py_dirs: HashMap<PathBuf, bool> = HashMap::new();
    fs::create_dir_all(dest)?;
    let (files, _left_out) = walk_files(src);
    let from_git = git_files(project).is_some();
    let tracked = if from_git {
        git_tracked(project)
    } else {
        HashSet::new()
    };
    let mut copied = 0;
    for rel in files {
        let extension =
            python && is_python_extension(&rel) && holds_python_sources(src, &rel, &mut py_dirs);
        if !extension && is_dropped_build_output(&rel, from_git, &tracked) {
            continue;
        }
        let from = src.join(&rel);
        let to = dest.join(&rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        let meta = fs::symlink_metadata(&from)?;
        if meta.is_symlink() {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(repointed_link(src, dest, &from)?, &to)?;
                copied += 1;
            }
            continue;
        }
        copy_file(&from, &to)?;
        copied += 1;
    }
    Ok(copied)
}

/// Copies the project into `dest` (created). The `.dla` directory is never copied.
pub fn copy_project(project: &Path, dest: &Path) -> std::io::Result<CopyReport> {
    copy_project_scoped(project, dest, None)
}

/// Like [`copy_project`], restricted to the run's `scope`: only files inside it are
/// copied, plus the files at the project root (the build files a module needs).
pub fn copy_project_scoped(
    project: &Path,
    dest: &Path,
    scope: Option<&Scope>,
) -> std::io::Result<CopyReport> {
    copy_project_for(project, dest, scope, None)
}

/// True for a built extension module of a Python package: a file named `*.so`, `*.pyd` or
/// `*.dylib` (`_native.cpython-312-x86_64-linux-gnu.so` included).
fn is_python_extension(rel: &Path) -> bool {
    rel.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| ["so", "pyd", "dylib"].contains(&e.to_ascii_lowercase().as_str()))
}

/// True when the directory of `rel` holds a Python source file of the project.
fn holds_python_sources(project: &Path, rel: &Path, cache: &mut HashMap<PathBuf, bool>) -> bool {
    let dir = rel.parent().map(Path::to_path_buf).unwrap_or_default();
    *cache.entry(dir.clone()).or_insert_with(|| {
        fs::read_dir(project.join(&dir)).is_ok_and(|rd| {
            rd.flatten()
                .any(|e| e.path().extension().is_some_and(|x| x == "py"))
        })
    })
}

/// The built extension modules a Python project's git listing does not hold: untracked files
/// git ignores (the usual state of a `build_ext --inplace`) next to Python sources. Virtual
/// environments, installed packages, hidden directories and the root-level build directories
/// are not the project.
fn ignored_python_extensions(
    project: &Path,
    listed: &HashSet<PathBuf>,
    cache: &mut HashMap<PathBuf, bool>,
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::new()];
    let mut seen = 0usize;
    while let Some(rel) = stack.pop() {
        let Ok(rd) = fs::read_dir(project.join(&rel)) else {
            continue;
        };
        for e in rd.flatten() {
            seen += 1;
            if seen > MAX_FILES * 4 {
                return out;
            }
            let Ok(ft) = e.file_type() else { continue };
            let name = e.file_name();
            let name_s = name.to_string_lossy();
            let child = rel.join(&name);
            if ft.is_dir() {
                if name_s.starts_with('.')
                    || SKIP_ANYWHERE.contains(&name_s.as_ref())
                    || ["site-packages", "dist-packages"].contains(&name_s.as_ref())
                    || (rel.as_os_str().is_empty() && skip_at_root(&name_s))
                    || e.path().join("pyvenv.cfg").is_file()
                {
                    continue;
                }
                stack.push(child);
            } else if ft.is_file()
                && is_python_extension(&child)
                && !listed.contains(&child)
                && holds_python_sources(project, &child, cache)
            {
                out.push(child);
            }
        }
    }
    out.sort();
    out
}

/// The string values of a `package.json` value, depth first.
fn json_strings(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::String(s) => out.push(s.clone()),
        serde_json::Value::Array(a) => a.iter().for_each(|x| json_strings(x, out)),
        serde_json::Value::Object(o) => o.values().for_each(|x| json_strings(x, out)),
        _ => {}
    }
}

/// The paths (relative to the package directory, no glob part, no `./`) that a `package.json`
/// names as entry points: `main`, `module`, `bin`, every string in `exports`, and the
/// entries of `files`. Negations, absolute paths and paths that leave the package are left out.
fn package_entry_paths(text: &str) -> Vec<PathBuf> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    let mut raw = Vec::new();
    for key in ["main", "module", "bin", "exports", "files"] {
        if let Some(x) = v.get(key) {
            json_strings(x, &mut raw);
        }
    }
    let mut out: Vec<PathBuf> = Vec::new();
    for r in raw {
        if r.starts_with('!') {
            continue;
        }
        let mut path = PathBuf::new();
        for c in r.split('/') {
            if c.contains(['*', '?', '[', '{']) {
                break;
            }
            match c {
                "" | "." => {}
                ".." => {
                    path = PathBuf::new();
                    break;
                }
                c => path.push(c),
            }
        }
        if !path.as_os_str().is_empty() && !path.is_absolute() && !out.contains(&path) {
            out.push(path);
        }
    }
    out
}

/// The files of the directories (or the single files) that the `package.json` of each of
/// `package_dirs` names as entry points and that git ignores, in the project, relative, sorted.
/// The unit is the topmost ignored directory on the way to an entry point (`lib` for
/// `lib/node/index.js`), so the whole build output comes along; an entry point that is the
/// only ignored thing on its way is taken alone. Files `listed` by git are not repeated.
fn ignored_entry_files(
    project: &Path,
    package_dirs: &[PathBuf],
    listed: &HashSet<PathBuf>,
) -> Vec<PathBuf> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    // Every prefix of every entry point, below the package directory.
    let mut prefixes: Vec<PathBuf> = Vec::new();
    let mut entries: Vec<(PathBuf, PathBuf)> = Vec::new();
    for dir in package_dirs {
        let Ok(text) = fs::read_to_string(project.join(dir).join("package.json")) else {
            continue;
        };
        for e in package_entry_paths(&text) {
            let full = dir.join(&e);
            if full
                .components()
                .any(|c| SKIP_ANYWHERE.iter().any(|s| c.as_os_str() == *s))
                || !project.join(&full).exists()
            {
                continue;
            }
            let mut acc = dir.clone();
            for c in e.components() {
                acc.push(c);
                if !prefixes.contains(&acc) {
                    prefixes.push(acc.clone());
                }
            }
            entries.push((dir.clone(), full));
        }
    }
    if prefixes.is_empty() {
        return Vec::new();
    }
    let Ok(mut child) = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["check-ignore", "-z", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Vec::new();
    };
    if let Some(mut stdin) = child.stdin.take() {
        let mut input = Vec::new();
        for p in &prefixes {
            input.extend_from_slice(dla_core::paths::to_slash(p).as_bytes());
            input.push(0);
        }
        let _ = stdin.write_all(&input);
    }
    let Ok(out) = child.wait_with_output() else {
        return Vec::new();
    };
    let ignored: HashSet<PathBuf> = out
        .stdout
        .split(|b| *b == 0)
        .filter_map(|s| std::str::from_utf8(s).ok())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();
    let mut roots: Vec<PathBuf> = Vec::new();
    for (dir, full) in entries {
        let mut acc = dir.clone();
        let rel_to_dir = full.strip_prefix(&dir).unwrap_or(&full).to_path_buf();
        for c in rel_to_dir.components() {
            acc.push(c);
            if ignored.contains(&acc) {
                if !roots.iter().any(|r| acc.starts_with(r)) {
                    roots.retain(|r| !r.starts_with(&acc));
                    roots.push(acc.clone());
                }
                break;
            }
        }
    }
    let mut files = Vec::new();
    for root in roots {
        let abs = project.join(&root);
        if abs.is_file() {
            files.push(root);
            continue;
        }
        let mut stack = vec![PathBuf::new()];
        while let Some(rel) = stack.pop() {
            let Ok(rd) = fs::read_dir(abs.join(&rel)) else {
                continue;
            };
            for e in rd.flatten() {
                let Ok(ft) = e.file_type() else { continue };
                let child = rel.join(e.file_name());
                if ft.is_dir() {
                    if !SKIP_ANYWHERE.iter().any(|s| e.file_name() == *s) {
                        stack.push(child);
                    }
                } else if ft.is_file() {
                    files.push(root.join(child));
                }
            }
        }
    }
    files.retain(|f| !listed.contains(f) && !is_dla_path(f));
    files.sort();
    files.dedup();
    files
}

/// Links `<dest>/<dir>/node_modules` to the project's `<dir>/node_modules` when the project has
/// one. True when a link was made; false when there is nothing to link or the copy already holds
/// the path; the reason as an error when the platform or the file system refuses. The link is
/// a symbolic link (a directory junction is not made), so on Windows it needs the privilege
/// to create symbolic links; without it the reason is reported and nothing else changes.
fn link_node_modules(project: &Path, dest: &Path, dir: &Path) -> Result<bool, String> {
    let from = project.join(dir).join("node_modules");
    if !from.is_dir() {
        return Ok(false);
    }
    let to = dest.join(dir).join("node_modules");
    if fs::symlink_metadata(&to).is_ok() {
        return Ok(false);
    }
    let target = from
        .canonicalize()
        .map_err(|e| format!("cannot resolve it: {e}"))?;
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, &to)
            .map(|()| true)
            .map_err(|e| format!("cannot create symlink: {e}"))
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(&target, &to)
            .map(|()| true)
            .map_err(|e| {
                format!("cannot create symlink (needs the privilege to create symbolic links): {e}")
            })
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (target, to);
        Err("symlinks are not available on this platform".to_owned())
    }
}

/// Like [`copy_project_scoped`], for a harness of `runtime`. A Python harness has no build step,
/// so the built extension modules of the project (untracked `*.so`, `*.pyd` or `*.dylib` files
/// that lie in a directory holding Python sources) are kept: left out, a package whose
/// extension modules were built in place would fail at import in the copy. Another runtime's
/// copy is unchanged: there the build step recreates the objects and libraries.
pub fn copy_project_for(
    project: &Path,
    dest: &Path,
    scope: Option<&Scope>,
    runtime: Option<Runtime>,
) -> std::io::Result<CopyReport> {
    copy_project_with(project, dest, scope, &[], runtime)
}

/// Like [`copy_project_for`], and the directories in `also` are copied as well when a `scope`
/// cuts the copy: a Go driver for a package of one module needs the other modules of the
/// repository that module requires (the work copy's `go.work` takes them from the copy).
pub fn copy_project_with(
    project: &Path,
    dest: &Path,
    scope: Option<&Scope>,
    also: &[Scope],
    runtime: Option<Runtime>,
) -> std::io::Result<CopyReport> {
    let python = runtime == Some(Runtime::Python);
    let node = runtime == Some(Runtime::Node);
    let mut package_dirs: Vec<PathBuf> = Vec::new();
    let mut py_dirs: HashMap<PathBuf, bool> = HashMap::new();
    fs::create_dir_all(dest)?;
    let (files, from_git, left_out) = match git_files(project) {
        Some(f) => (f, true, Vec::new()),
        None => {
            let (f, l) = walk_files(project);
            (f, false, l)
        }
    };
    let tracked = if from_git {
        git_tracked(project)
    } else {
        HashSet::new()
    };
    let mut report = CopyReport {
        from_git,
        ..CopyReport::default()
    };
    let mut objects = 0usize;
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    let mut skip = |report: &mut CopyReport, why: String| {
        report.skipped += 1;
        *reasons.entry(why).or_default() += 1;
    };
    for rel in files {
        if is_dla_path(&rel) {
            continue;
        }
        if let Some(scope) = scope {
            let slash = dla_core::paths::to_slash(&rel);
            let at_root = rel.components().count() == 1;
            let kept = |s: &Scope| s.includes(&slash) || keeps_for_build(s, &rel, &slash);
            if !at_root && !kept(scope) && !also.iter().any(kept) {
                report.out_of_scope += 1;
                if let Some(lang) = Lang::from_path(&rel)
                    && !dla_census::walk::is_test_path(&slash)
                {
                    *report
                        .out_of_scope_sources
                        .entry(lang.runtime())
                        .or_default() += 1;
                }
                continue;
            }
        }
        // A Python harness has no build step: the built extension modules stay.
        let extension = python
            && is_python_extension(&rel)
            && holds_python_sources(project, &rel, &mut py_dirs);
        if extension && !tracked.contains(&rel) {
            report.extensions_kept += 1;
        }
        // Build outputs of another tree: rebuilt in the copy, never copied. Outside git
        // every library counts as shipped; in git only the tracked ones do.
        if !extension && is_dropped_build_output(&rel, from_git, &tracked) {
            objects += 1;
            continue;
        }
        if report.files >= MAX_FILES {
            skip(&mut report, format!("over the {MAX_FILES}-file cap"));
            continue;
        }
        let src = project.join(&rel);
        // `git ls-files` lists symlinks and deleted-but-tracked paths too.
        let Ok(meta) = fs::symlink_metadata(&src) else {
            skip(&mut report, "listed but missing".to_owned());
            continue;
        };
        let to = dest.join(&rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        if meta.is_symlink() {
            match copy_symlink(project, dest, &src, &to) {
                Ok(()) => report.files += 1,
                Err(why) => skip(&mut report, why),
            }
            continue;
        }
        if !meta.is_file() {
            skip(&mut report, "not a regular file".to_owned());
            continue;
        }
        if copy_file(&src, &to).is_err() {
            skip(&mut report, "unreadable file".to_owned());
            continue;
        }
        report.files += 1;
        if node && rel.file_name().is_some_and(|n| n == "package.json") {
            package_dirs.push(rel.parent().map(Path::to_path_buf).unwrap_or_default());
        }
    }
    let mut node_gaps: Vec<String> = Vec::new();
    if node && !package_dirs.is_empty() {
        if from_git {
            let listed: HashSet<PathBuf> =
                git_files(project).unwrap_or_default().into_iter().collect();
            for rel in ignored_entry_files(project, &package_dirs, &listed) {
                if report.files >= MAX_FILES {
                    break;
                }
                let (src, to) = (project.join(&rel), dest.join(&rel));
                if let Some(parent) = to.parent() {
                    fs::create_dir_all(parent)?;
                }
                if copy_file(&src, &to).is_ok() {
                    report.build_output_kept += 1;
                    report.files += 1;
                }
            }
        }
        for dir in &package_dirs {
            match link_node_modules(project, dest, dir) {
                Ok(true) => report.node_modules_linked += 1,
                Ok(false) => {}
                Err(why) => node_gaps.push(format!(
                    "scratch copy: node_modules of {} not linked ({why}); a driver must reach the dependencies another way",
                    if dir.as_os_str().is_empty() {
                        ".".to_owned()
                    } else {
                        dla_core::paths::to_slash(dir)
                    }
                )),
            }
        }
    }
    if python && from_git {
        let listed: HashSet<PathBuf> = git_files(project).unwrap_or_default().into_iter().collect();
        for rel in ignored_python_extensions(project, &listed, &mut py_dirs) {
            if is_dla_path(&rel) {
                continue;
            }
            if let Some(scope) = scope {
                let slash = dla_core::paths::to_slash(&rel);
                if rel.components().count() > 1 && !scope.includes(&slash) {
                    continue;
                }
            }
            let (src, to) = (project.join(&rel), dest.join(&rel));
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            if copy_file(&src, &to).is_ok() {
                report.extensions_kept += 1;
                report.files += 1;
            }
        }
    }
    for (why, n) in reasons {
        report
            .gaps
            .push(format!("scratch copy: {n} path(s) not copied ({why})"));
    }
    if report.extensions_kept > 0 {
        report.gaps.push(format!(
            "scratch copy: {} built extension module(s) kept (untracked *.so, *.pyd or *.dylib files next to Python sources: a Python harness has no build step)",
            report.extensions_kept
        ));
    }
    if report.build_output_kept > 0 {
        report.gaps.push(format!(
            "scratch copy: {} file(s) of build output that git ignores kept (a package.json names them as entry points: main, module, bin, exports or files; a Node harness has no build step)",
            report.build_output_kept
        ));
    }
    if report.node_modules_linked > 0 {
        report.gaps.push(format!(
            "scratch copy: {} node_modules director{} linked to the project's own, not copied (a symbolic link at the same relative path)",
            report.node_modules_linked,
            if report.node_modules_linked == 1 { "y" } else { "ies" }
        ));
    }
    report.gaps.extend(node_gaps);
    if objects > 0 {
        report.gaps.push(format!(
            "scratch copy: {objects} object file(s) and untracked librar{} not copied (they are rebuilt in the copy)",
            if objects == 1 { "y" } else { "ies" }
        ));
    }
    if report.out_of_scope > 0 {
        report.gaps.push(format!(
            "scratch copy: {} file(s) outside the run's scope not copied (only the scope, the files at the project root, the build files above the scope and the root-level build directories are)",
            report.out_of_scope
        ));
    }
    if !left_out.is_empty() {
        report.gaps.push(format!(
            "scratch copy outside git: top-level build-output director{} not copied: {}",
            if left_out.len() == 1 { "y" } else { "ies" },
            left_out.join(", ")
        ));
    }
    Ok(report)
}

// ---------------------------------------------------------------------------------------
// Thinning: a harness copy that keeps only what differs from the project

/// List of the files taken out of a harness's `work/`, in the harness directory.
pub const THIN_LIST: &str = "work-thinned.json";
/// Lock file of a harness's `work/`, in the harness directory. Its modification time is the
/// harness's "last used" time.
pub const WORK_LOCK: &str = "work.lock";
/// `dla_work_thin` of the list.
const THIN_VERSION: u32 = 1;
/// Paths a `work-copy-stale` refusal names.
const STALE_NAMED: usize = 3;
/// The harnesses used last that [`thin_run`] callers keep whole at the end of `harness gen`.
pub const KEEP_RECENT: usize = 4;
/// Environment variable that turns thinning off (a fill still works).
pub const KEEP_WORK_ENV: &str = "DLA_KEEP_WORK";

#[derive(Debug, Serialize, Deserialize)]
struct ThinFile {
    path: String,
    len: u64,
    digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThinList {
    dla_work_thin: u32,
    files: Vec<ThinFile>,
    bytes: u64,
}

/// What a thin removed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ThinReport {
    /// Files removed.
    pub files: usize,
    /// Their size in bytes.
    pub bytes: u64,
}

/// Why a fill did nothing.
#[derive(Debug)]
pub enum FillError {
    /// Listed files whose project file has changed or gone since the thin (all of them).
    Stale(Vec<String>),
    /// The list or a file could not be read or written.
    Io(dla_core::Error),
}

/// True when `DLA_KEEP_WORK` asks for whole copies.
pub fn thinning_off() -> bool {
    std::env::var_os(KEEP_WORK_ENV).is_some_and(|v| !v.is_empty() && v != "0")
}

fn read_full(r: &mut impl std::io::Read, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut n = 0;
    while n < buf.len() {
        let k = r.read(&mut buf[n..])?;
        if k == 0 {
            break;
        }
        n += k;
    }
    Ok(n)
}

/// True when both files hold the same bytes.
fn same_bytes(a: &Path, b: &Path) -> std::io::Result<bool> {
    let (mut fa, mut fb) = (fs::File::open(a)?, fs::File::open(b)?);
    let (mut ba, mut bb) = (vec![0u8; 64 << 10], vec![0u8; 64 << 10]);
    loop {
        let (na, nb) = (read_full(&mut fa, &mut ba)?, read_full(&mut fb, &mut bb)?);
        if na != nb || ba[..na] != bb[..nb] {
            return Ok(false);
        }
        if na == 0 {
            return Ok(true);
        }
    }
}

/// The files of `work` that the project holds byte for byte, with their length and digest.
/// Symlinks, directories and everything that differs are left out. A file that cannot be
/// read is not identical.
fn identical_files(work: &Path, project: &Path) -> Vec<ThinFile> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let Ok(rd) = fs::read_dir(work.join(&rel)) else {
            continue;
        };
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            let child = rel.join(e.file_name());
            if ft.is_dir() {
                stack.push(child);
                continue;
            }
            if !ft.is_file() {
                continue;
            }
            let Some(path) = child.to_str().map(|_| dla_core::paths::to_slash(&child)) else {
                continue;
            };
            let (from, theirs) = (work.join(&child), project.join(&child));
            let (Ok(wm), Ok(pm)) = (fs::symlink_metadata(&from), fs::symlink_metadata(&theirs))
            else {
                continue;
            };
            if !pm.is_file()
                || wm.len() != pm.len()
                || wm.permissions() != pm.permissions()
                || !same_bytes(&from, &theirs).unwrap_or(false)
            {
                continue;
            }
            let Ok((len, digest)) = dla_core::integrity::measure(&theirs) else {
                continue;
            };
            if len == wm.len() {
                out.push(ThinFile { path, len, digest });
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// A listed path as a relative path that stays below its root; `None` for anything else.
fn listed_rel(path: &str) -> Option<PathBuf> {
    let rel = PathBuf::from(path);
    (!path.is_empty()
        && rel
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_))))
    .then_some(rel)
}

/// Removes from `work` every regular file the project holds with the same length and bytes.
/// Files that differ, files the project does not hold, symlinks and all directories stay.
/// The list of the files to remove is written to `list` (a temporary file, then a rename)
/// before the first file goes, so a thin that stops half way leaves a list that [`fill_files`]
/// can use. Nothing is written when no file is identical.
pub fn thin_files(work: &Path, list: &Path, project: &Path) -> std::io::Result<ThinReport> {
    let files = identical_files(work, project);
    if files.is_empty() {
        return Ok(ThinReport::default());
    }
    let bytes = files.iter().map(|f| f.len).sum();
    let doc = ThinList {
        dla_work_thin: THIN_VERSION,
        files,
        bytes,
    };
    let text = serde_json::to_string_pretty(&doc).map_err(std::io::Error::other)?;
    dla_core::fsutil::write_atomic(list, text.as_bytes()).map_err(|e| match e {
        dla_core::Error::Io { source, .. } => source,
        other => std::io::Error::other(other.to_string()),
    })?;
    let mut report = ThinReport::default();
    for f in &doc.files {
        let Some(rel) = listed_rel(&f.path) else {
            continue;
        };
        match fs::remove_file(work.join(rel)) {
            Ok(()) => {
                report.files += 1;
                report.bytes += f.len;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(report)
}

/// Brings back the files `list` names that are missing in `work`, from `project`, then
/// deletes `list`. Every missing file is checked first (length and digest as listed); when one
/// or more do not match or are gone from the project, nothing is changed and the error names
/// them all. A listed file that is still in `work` (a thin that was cut short) stays as it is.
/// Returns the files brought back; no list means nothing to do.
pub fn fill_files(work: &Path, list: &Path, project: &Path) -> Result<usize, FillError> {
    let text = match fs::read_to_string(list) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(FillError::Io(dla_core::Error::io(list, e))),
    };
    let doc: ThinList = serde_json::from_str(&text)
        .ok()
        .filter(|d: &ThinList| d.dla_work_thin == THIN_VERSION)
        .ok_or_else(|| {
            FillError::Io(dla_core::Error::Malformed {
                path: list.to_path_buf(),
                message: "not a thinned-copy list of this version".to_owned(),
            })
        })?;
    let (mut todo, mut stale) = (Vec::new(), Vec::new());
    for f in &doc.files {
        let Some(rel) = listed_rel(&f.path) else {
            stale.push(f.path.clone());
            continue;
        };
        if fs::symlink_metadata(work.join(&rel)).is_ok() {
            continue;
        }
        let theirs = project.join(&rel);
        let intact = fs::symlink_metadata(&theirs).is_ok_and(|m| m.is_file())
            && dla_core::integrity::measure(&theirs)
                .is_ok_and(|(len, digest)| len == f.len && digest == f.digest);
        if intact {
            todo.push((rel, theirs));
        } else {
            stale.push(f.path.clone());
        }
    }
    if !stale.is_empty() {
        return Err(FillError::Stale(stale));
    }
    for (rel, from) in &todo {
        let to = work.join(rel);
        let io = |p: &Path, e| FillError::Io(dla_core::Error::io(p, e));
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        }
        // A copy that is cut short must never look like a finished file.
        let mut tmp = to.clone().into_os_string();
        tmp.push(".dla-fill");
        let tmp = PathBuf::from(tmp);
        let done = copy_file(from, &tmp).and_then(|()| fs::rename(&tmp, &to));
        if let Err(e) = done {
            let _ = fs::remove_file(&tmp);
            return Err(io(&to, e));
        }
    }
    fs::remove_file(list).map_err(|e| FillError::Io(dla_core::Error::io(list, e)))?;
    Ok(todo.len())
}

/// A shared lock on a harness's `work/`, held while a process reads or runs in it. Dropping
/// it releases the lock.
#[derive(Debug)]
pub struct WorkGuard {
    _lock: Option<fs::File>,
}

fn open_lock(hdir: &Path) -> std::io::Result<fs::File> {
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(hdir.join(WORK_LOCK))
}

fn io_refusal_at(path: &Path, e: std::io::Error) -> Refusal {
    dla_core::Error::io(path, e).to_refusal()
}

fn stale_refusal(hdir: &Path, lang: &str, stale: &[String]) -> Refusal {
    let id = hdir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let named = stale
        .iter()
        .take(STALE_NAMED)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    let more = stale.len().saturating_sub(STALE_NAMED);
    Refusal::new(
        dla_core::error::codes::WORK_COPY_STALE,
        format!(
            "harness {id} keeps only the files that differ from the project, and {} file(s) it needs back changed or went from the project since: {named}{}",
            stale.len(),
            if more > 0 {
                format!(" and {more} more")
            } else {
                String::new()
            }
        ),
        format!("the project files as they were when {id} was thinned"),
        format!(
            "dla harness gen --lang {lang} (the new harness copies the project as it is now; the run's seals cover only the run's own files, so the changed project needs no `dla run reseal`), or put the named files back as they were"
        ),
    )
}

/// Takes the shared lock on the `work/` of the harness in `hdir` and, when that copy was
/// thinned, brings its files back first (under the exclusive lock, then the shared lock is
/// taken again). The returned guard must be held for as long as the caller reads or runs in
/// `work/`. Sets the modification time of `work.lock` to now. Refuses with `work-copy-stale`
/// when a file to bring back no longer matches the project.
pub fn use_work(hdir: &Path, project: &Path, lang: &str) -> Result<WorkGuard, Refusal> {
    use std::io::ErrorKind::Unsupported;
    let (work, list) = (hdir.join(WORK), hdir.join(THIN_LIST));
    loop {
        let f = open_lock(hdir).map_err(|e| io_refusal_at(&hdir.join(WORK_LOCK), e))?;
        match f.lock_shared() {
            Ok(()) => {}
            // A file system without locks: no protection, but the fill below still works.
            Err(e) if e.kind() == Unsupported => {
                fill_files(&work, &list, project).map_err(|e| fill_refusal(hdir, lang, e))?;
                return Ok(WorkGuard { _lock: None });
            }
            Err(e) => return Err(io_refusal_at(&hdir.join(WORK_LOCK), e)),
        }
        if !list.exists() {
            // Best effort: the time only orders harnesses for the thinner.
            let _ = f.set_modified(std::time::SystemTime::now());
            return Ok(WorkGuard { _lock: Some(f) });
        }
        f.unlock()
            .and_then(|()| f.lock())
            .map_err(|e| io_refusal_at(&hdir.join(WORK_LOCK), e))?;
        if list.exists() {
            fill_files(&work, &list, project).map_err(|e| fill_refusal(hdir, lang, e))?;
        }
        drop(f);
    }
}

fn fill_refusal(hdir: &Path, lang: &str, e: FillError) -> Refusal {
    match e {
        FillError::Stale(paths) => stale_refusal(hdir, lang, &paths),
        FillError::Io(e) => e.to_refusal(),
    }
}

/// What [`thin_harness`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Thinned {
    /// Files were removed (possibly none: nothing was identical).
    Done(ThinReport),
    /// The harness already has a list.
    Already,
    /// Another process holds the lock.
    Busy,
}

/// The time a harness was last used: the modification time of `work.lock`, else of the
/// harness directory.
fn used_time(hdir: &Path) -> std::time::SystemTime {
    fs::metadata(hdir.join(WORK_LOCK))
        .or_else(|_| fs::metadata(hdir))
        .and_then(|m| m.modified())
        .unwrap_or(std::time::UNIX_EPOCH)
}

/// How often [`thin_harness`] asks again for a lock that reads as held, and the pause between
/// two tries. A process that another thread is starting holds a copy of every open file until
/// it execs, so a lock released a moment ago can read as held for an instant; a user of the copy
/// holds it for a whole build or run, which no short wait outlasts.
const BUSY_RETRIES: u32 = 20;
const BUSY_PAUSE: std::time::Duration = std::time::Duration::from_millis(10);

/// Thins the `work/` of the harness in `hdir` against `project`, unless it is thinned already
/// or another process holds its lock (the exclusive lock is tried for a moment, see
/// [`BUSY_RETRIES`], never waited for).
pub fn thin_harness(hdir: &Path, project: &Path) -> std::io::Result<Thinned> {
    let list = hdir.join(THIN_LIST);
    if list.exists() {
        return Ok(Thinned::Already);
    }
    let used = used_time(hdir);
    let existed = hdir.join(WORK_LOCK).exists();
    let f = open_lock(hdir)?;
    if !existed {
        // The lock file is new: it must not make the harness look used just now.
        let _ = f.set_modified(used);
    }
    let mut asked = 0;
    loop {
        match f.try_lock() {
            Ok(()) => break,
            Err(fs::TryLockError::WouldBlock) if asked < BUSY_RETRIES => {
                asked += 1;
                std::thread::sleep(BUSY_PAUSE);
            }
            Err(fs::TryLockError::WouldBlock) => return Ok(Thinned::Busy),
            Err(fs::TryLockError::Error(e)) => return Err(e),
        }
    }
    if list.exists() {
        return Ok(Thinned::Already);
    }
    thin_files(&hdir.join(WORK), &list, project).map(Thinned::Done)
}

/// What [`thin_run`] did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunThin {
    /// Harnesses thinned (that freed at least one file).
    pub thinned: Vec<String>,
    /// Bytes freed.
    pub bytes: u64,
    /// Harnesses left alone for a reason other than being thinned already: `(id, reason)`.
    pub skipped: Vec<(String, String)>,
}

/// Thins the harnesses of the run in `run_dir`, except `except` and the `keep_recent` used
/// last. A harness without a manifest is being generated and is left alone. Never fails: a
/// harness that cannot be thinned is named in [`RunThin::skipped`].
pub fn thin_run(
    run_dir: &Path,
    project: &Path,
    keep_recent: usize,
    except: Option<&str>,
) -> RunThin {
    let mut found: Vec<(std::time::SystemTime, u64, String, PathBuf)> = Vec::new();
    if let Ok(rd) = fs::read_dir(run_dir.join(dla_core::harness::HARNESS_DIR)) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Some(n) = name
                .strip_prefix('H')
                .and_then(|d| d.parse::<u64>().ok())
                .filter(|_| e.path().join(super::MANIFEST).is_file())
            else {
                continue;
            };
            found.push((used_time(&e.path()), n, name, e.path()));
        }
    }
    found.sort_by_key(|a| std::cmp::Reverse((a.0, a.1)));
    let mut out = RunThin::default();
    for (_, _, id, hdir) in found.into_iter().skip(keep_recent) {
        if except == Some(id.as_str()) {
            continue;
        }
        match thin_harness(&hdir, project) {
            Ok(Thinned::Done(r)) if r.files > 0 => {
                out.thinned.push(id);
                out.bytes += r.bytes;
            }
            Ok(Thinned::Done(_) | Thinned::Already) => {}
            Ok(Thinned::Busy) => out
                .skipped
                .push((id, "in use by another process".to_owned())),
            Err(e) => out.skipped.push((id, e.to_string())),
        }
    }
    out.thinned
        .sort_by_key(|id| id[1..].parse::<u64>().unwrap_or(0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_probe::project_copy::is_library;
    use std::process::Command;

    #[cfg(unix)]
    #[test]
    fn copy_tree_repoints_an_absolute_link_into_the_copy() {
        let d = tempfile::tempdir().unwrap();
        let src = d.path().join("H1").join("work");
        fs::create_dir_all(src.join("real")).unwrap();
        fs::write(src.join("real/F.java"), "x").unwrap();
        std::os::unix::fs::symlink(src.join("real"), src.join("linkdir")).unwrap();
        let dest = d.path().join("H2").join("work");
        copy_tree(&src, &dest, &src).unwrap();
        let to = fs::read_link(dest.join("linkdir")).unwrap();
        assert_eq!(
            to,
            dest.join("real"),
            "the copy must not lead back to the original"
        );
    }

    #[test]
    fn a_copy_keeps_modification_times() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        fs::create_dir_all(p.join("src")).unwrap();
        let base = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        // A header that sorts before the sources, an old source and a newer one.
        for (f, age) in [("src/a.h", 30), ("src/b.c", 20), ("src/z.c", 10)] {
            fs::write(p.join(f), f).unwrap();
            let t = base + std::time::Duration::from_secs(age);
            fs::File::options()
                .write(true)
                .open(p.join(f))
                .unwrap()
                .set_modified(t)
                .unwrap();
        }
        let (via_project, via_tree) = (d.path().join("w"), d.path().join("t"));
        copy_project(&p, &via_project).unwrap();
        copy_tree(&p, &via_tree, &p).unwrap();
        for dest in [&via_project, &via_tree] {
            for f in ["src/a.h", "src/b.c", "src/z.c"] {
                let want = fs::metadata(p.join(f)).unwrap().modified().unwrap();
                let got = fs::metadata(dest.join(f)).unwrap().modified().unwrap();
                assert_eq!(got, want, "{f} in {}", dest.display());
            }
        }
    }

    fn git(p: &Path, args: &[&str]) -> bool {
        Command::new("git")
            .arg("-C")
            .arg(p)
            .args(args)
            .output()
            .is_ok_and(|o| o.status.success())
    }

    #[test]
    fn a_copy_leaves_object_files_to_be_rebuilt() {
        // A C project whose header sorts before its objects: the copy must hold no object
        // that `make` would take as up to date and keep debug paths of another tree in.
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        for f in [
            "Makefile",
            "src/a.h",
            "src/main.c",
            "src/main.o",
            "src/util.obj",
            "build_helper.lo",
            "prebuilt/blob.o",
            "libs/prebuilt.a",
            "libs/built.a",
            "libs/built.so",
            "libs/built.so.1",
            "libs/built.so.1.2.3",
            "libs/built.1.dylib",
            "libs/notes.so.txt",
            "vendor/dep/dep.a",
        ] {
            let path = p.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, f).unwrap();
        }
        // Outside git every library counts as shipped, in the copy and in the clone.
        let plain = d.path().join("plain");
        copy_project(&p, &plain).unwrap();
        let plain_clone = d.path().join("plain-clone");
        copy_tree(&plain, &plain_clone, &p).unwrap();
        for dest in [&plain, &plain_clone] {
            for kept in ["libs/built.a", "libs/built.so.1", "libs/prebuilt.a"] {
                assert!(dest.join(kept).is_file(), "{kept} in {}", dest.display());
            }
            assert!(!dest.join("src/main.o").exists());
            // Outside git there is no telling a shipped object from a build output.
            assert!(!dest.join("prebuilt/blob.o").exists());
        }
        // In a git work tree only the tracked libraries are shipped.
        if !git(&p, &["init", "-q"]) {
            return;
        }
        assert!(git(
            &p,
            &[
                "add",
                "Makefile",
                "src/a.h",
                "src/main.c",
                "prebuilt/blob.o",
                "libs/prebuilt.a"
            ]
        ));
        let dest = d.path().join("w");
        let r = copy_project(&p, &dest).unwrap();
        for gone in ["src/main.o", "src/util.obj", "build_helper.lo"] {
            assert!(!dest.join(gone).exists(), "{gone} copied");
        }
        for gone in [
            "libs/built.a",
            "libs/built.so",
            "libs/built.so.1",
            "libs/built.so.1.2.3",
            "libs/built.1.dylib",
        ] {
            assert!(!dest.join(gone).exists(), "{gone} copied");
        }
        for kept in [
            "Makefile",
            "src/a.h",
            "src/main.c",
            "prebuilt/blob.o",
            "libs/prebuilt.a",
            "libs/notes.so.txt",
            "vendor/dep/dep.a",
        ] {
            assert!(dest.join(kept).is_file(), "{kept} missing");
        }
        assert!(
            r.gaps.iter().any(|g| g.contains("object file")),
            "{:?}",
            r.gaps
        );
        // The clone of a harness tree takes the same decision: an object or library git
        // tracks in the project stays (a prebuilt one the build links), the others are rebuilt.
        let clone = d.path().join("c");
        copy_tree(&p, &clone, &p).unwrap();
        for gone in [
            "src/main.o",
            "src/util.obj",
            "build_helper.lo",
            "libs/built.a",
            "libs/built.so",
            "libs/built.so.1",
            "libs/built.so.1.2.3",
            "libs/built.1.dylib",
        ] {
            assert!(!clone.join(gone).exists(), "{gone} cloned");
        }
        for kept in [
            "prebuilt/blob.o",
            "libs/prebuilt.a",
            "libs/notes.so.txt",
            "vendor/dep/dep.a",
            "src/a.h",
        ] {
            assert!(clone.join(kept).is_file(), "{kept} not cloned");
        }
    }

    #[test]
    fn library_names_include_versioned_shared_objects() {
        for lib in [
            "libx.a",
            "libx.so",
            "libx.dylib",
            "libx.so.1",
            "libx.so.1.2.3",
            "libx.1.dylib",
        ] {
            assert!(is_library(Path::new(lib)), "{lib}");
        }
        for not in [
            "so",
            "x.so.txt",
            "a.1",
            "libx.1",
            "notes.txt",
            "x.so.",
            "1.2",
        ] {
            assert!(!is_library(Path::new(not)), "{not}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_copy_keeps_the_time_of_a_read_only_source() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let from = d.path().join("lease.h");
        fs::write(&from, "x").unwrap();
        let old = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
        fs::File::options()
            .write(true)
            .open(&from)
            .unwrap()
            .set_modified(old)
            .unwrap();
        fs::set_permissions(&from, fs::Permissions::from_mode(0o444)).unwrap();
        let to = d.path().join("copy.h");
        copy_file(&from, &to).unwrap();
        let meta = fs::metadata(&to).unwrap();
        assert_eq!(meta.modified().unwrap(), old);
        assert_eq!(meta.permissions().mode() & 0o777, 0o444);
    }

    #[test]
    fn make_rebuilds_every_object_in_a_copy_of_a_built_project() {
        // The native-tombstone-table shape: a header that sorts before the objects. A
        // copy that stamped files in listing order left `main.o` newer than the header
        // and `make` rebuilt only some objects. Needs `make`; skipped without it.
        if Command::new("make").arg("--version").output().is_err() {
            return;
        }
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        fs::create_dir_all(&p).unwrap();
        fs::write(
            p.join("Makefile"),
            "OBJS = main.o util.o\nall: $(OBJS)\n\ttouch app\n%.o: %.c lease.h\n\ttouch $@\n",
        )
        .unwrap();
        for f in ["lease.h", "main.c", "util.c"] {
            fs::write(p.join(f), f).unwrap();
        }
        assert!(
            Command::new("make")
                .arg("-C")
                .arg(&p)
                .output()
                .unwrap()
                .status
                .success()
        );
        assert!(p.join("main.o").is_file());
        let dest = d.path().join("w");
        copy_project(&p, &dest).unwrap();
        let plan = Command::new("make")
            .arg("-n")
            .arg("-C")
            .arg(&dest)
            .output()
            .unwrap();
        let plan = String::from_utf8_lossy(&plan.stdout).into_owned();
        assert!(
            plan.contains("touch main.o") && plan.contains("touch util.o"),
            "{plan}"
        );
    }

    #[test]
    fn walk_copy_skips_build_outputs_and_dla() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        for f in [
            "a.txt",
            "src/b.rs",
            "target/x.o",
            ".dla/r/state.json",
            "node_modules/m/i.js",
        ] {
            let path = p.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "x").unwrap();
        }
        let dest = d.path().join("w");
        let r = copy_project(&p, &dest).unwrap();
        assert_eq!(r.files, 2, "{r:?}");
        assert!(dest.join("src/b.rs").is_file());
        assert!(!dest.join("target").exists() && !dest.join(".dla").exists());
        assert!(r.gaps.iter().any(|g| g.contains("target")), "{:?}", r.gaps);
    }

    #[test]
    fn copy_tree_drops_build_outputs_and_keeps_sources() {
        let d = tempfile::tempdir().unwrap();
        let w = d.path().join("work");
        for f in [
            "main.go",
            "pkg/a/a.go",
            "target/debug/x",
            "build/classes/A.class",
            "pkg/__pycache__/m.pyc",
            "node_modules/m/i.js",
            "pkg/build/B.java",
        ] {
            let path = w.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, f).unwrap();
        }
        let dest = d.path().join("clone");
        assert_eq!(copy_tree(&w, &dest, &w).unwrap(), 3);
        assert_eq!(
            fs::read_to_string(dest.join("pkg/a/a.go")).unwrap(),
            "pkg/a/a.go"
        );
        // A package named `build` below the root is source.
        assert!(dest.join("pkg/build/B.java").is_file());
        for gone in ["target", "build", "node_modules", "pkg/__pycache__"] {
            assert!(!dest.join(gone).exists(), "{gone}");
        }
    }

    #[test]
    fn a_copy_cut_to_one_scope_also_holds_the_extra_scopes() {
        use dla_core::state::ScopeKind;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        for f in ["svc/a/a.go", "lib/l.go", "other/o.go"] {
            let path = p.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "x").unwrap();
        }
        let module = |path: &str| Scope {
            kind: ScopeKind::Module,
            path: path.into(),
            excluded: Vec::new(),
        };
        let dest = d.path().join("w");
        copy_project_with(&p, &dest, Some(&module("svc/a")), &[module("lib")], None).unwrap();
        assert!(dest.join("svc/a/a.go").is_file() && dest.join("lib/l.go").is_file());
        assert!(!dest.join("other").exists());
    }

    #[test]
    fn scoped_copy_keeps_the_scope_and_root_files_only() {
        use dla_core::state::ScopeKind;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        for f in [
            "pom.xml",
            "svc/a/A.java",
            "svc/a/gen/G.java",
            "svc/b/B.java",
            "svc/b/b.py",
            "svc/b/BTest.java",
        ] {
            let path = p.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "x").unwrap();
        }
        let scope = Scope {
            kind: ScopeKind::Module,
            path: "svc/a".into(),
            excluded: vec!["svc/a/gen".into()],
        };
        let dest = d.path().join("w");
        let r = copy_project_scoped(&p, &dest, Some(&scope)).unwrap();
        assert!(dest.join("pom.xml").is_file() && dest.join("svc/a/A.java").is_file());
        assert!(!dest.join("svc/b").exists() && !dest.join("svc/a/gen").exists());
        assert_eq!(r.files, 2, "{r:?}");
        assert_eq!(r.out_of_scope, 4);
        assert_eq!(r.out_of_scope_sources[&Runtime::Jvm], 2);
        assert_eq!(r.out_of_scope_sources[&Runtime::Python], 1);
        assert!(r.gaps.iter().any(|g| g.contains("outside the run's scope")));
    }

    #[test]
    fn scoped_copy_keeps_build_files_above_the_scope_and_root_build_dirs() {
        use dla_core::state::ScopeKind;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        for f in [
            "svc/go.mod",
            "svc/go.sum",
            "svc/main.go",
            "svc/a/a.go",
            "svc/b/b.go",
            "svc/b/go.txt",
            "gradle/libs.versions.toml",
            "buildSrc/build.gradle.kts",
            "vendor/m/m.go",
            "other/Cargo.toml",
        ] {
            let path = p.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "x").unwrap();
        }
        let scope = Scope {
            kind: ScopeKind::Module,
            path: "svc/a".into(),
            excluded: vec![],
        };
        let dest = d.path().join("w");
        copy_project_scoped(&p, &dest, Some(&scope)).unwrap();
        for kept in [
            "svc/go.mod",
            "svc/go.sum",
            "svc/a/a.go",
            "gradle/libs.versions.toml",
            "buildSrc/build.gradle.kts",
            "vendor/m/m.go",
        ] {
            assert!(dest.join(kept).is_file(), "{kept} missing");
        }
        for dropped in ["svc/main.go", "svc/b", "other"] {
            assert!(!dest.join(dropped).exists(), "{dropped} copied");
        }
    }

    #[test]
    fn scoped_copy_keeps_kotlin_dsl_build_logic_above_and_inside_excluded_parts() {
        use dla_core::state::ScopeKind;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        for f in [
            "svc/build.gradle.kts",
            "svc/settings.gradle.kts",
            "svc/docker.gradle.kts",
            "svc/b/build.gradle.kts",
            "svc/a/src/A.kt",
            "svc/a/gen/G.kt",
            "svc/b/B.kt",
        ] {
            let path = p.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "x").unwrap();
        }
        let scope = Scope {
            kind: ScopeKind::Module,
            path: "svc/a".into(),
            excluded: vec!["svc/docker.gradle.kts".into()],
        };
        let dest = d.path().join("w");
        let r = copy_project_scoped(&p, &dest, Some(&scope)).unwrap();
        for kept in [
            "svc/build.gradle.kts",
            "svc/settings.gradle.kts",
            "svc/docker.gradle.kts",
            "svc/b/build.gradle.kts",
            "svc/a/src/A.kt",
        ] {
            assert!(dest.join(kept).is_file(), "{kept} missing");
        }
        assert!(!dest.join("svc/b/B.kt").exists());
        assert_eq!(
            r.out_of_scope_sources[&Runtime::Jvm],
            1,
            "only svc/b/B.kt counts; build logic is kept: {r:?}"
        );
    }

    #[test]
    fn nested_build_dirs_are_source_and_inside_symlinks_are_kept() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("p");
        for f in [
            "app/build/gen.py",
            "pkg/out/x.py",
            "build/o.bin",
            "a/node_modules/m.js",
        ] {
            let path = p.join(f);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "x").unwrap();
        }
        fs::write(d.path().join("secret"), "s").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("app/build/gen.py", p.join("link.py")).unwrap();
            std::os::unix::fs::symlink(d.path().join("secret"), p.join("out.txt")).unwrap();
        }
        let dest = d.path().join("w");
        let r = copy_project(&p, &dest).unwrap();
        assert!(dest.join("app/build/gen.py").is_file());
        assert!(dest.join("pkg/out/x.py").is_file());
        assert!(!dest.join("build").exists() && !dest.join("a/node_modules").exists());
        #[cfg(unix)]
        {
            assert_eq!(fs::read_to_string(dest.join("link.py")).unwrap(), "x");
            assert!(
                fs::symlink_metadata(dest.join("link.py"))
                    .unwrap()
                    .is_symlink()
            );
            assert!(!dest.join("out.txt").exists());
            assert!(
                r.gaps.iter().any(|g| g.contains("outside the project")),
                "{:?}",
                r.gaps
            );
        }
        assert!(r.gaps.iter().any(|g| g.contains("build")), "{:?}", r.gaps);
    }
    // ---- thinning ----

    /// A project and a harness directory holding a copy of it: `a.txt`, `sub/b.txt`,
    /// `run.sh` (executable) and `c.txt`; the copy has the same files, `c.txt` changed, a new
    /// `d.txt`, an empty directory and (Unix) a symlink.
    struct Thin {
        _d: tempfile::TempDir,
        project: PathBuf,
        hdir: PathBuf,
    }

    impl Thin {
        fn new() -> Self {
            let d = tempfile::tempdir().unwrap();
            let project = d.path().join("project");
            let hdir = d.path().join("H1");
            fs::create_dir_all(project.join("sub")).unwrap();
            fs::create_dir_all(&hdir).unwrap();
            let old = std::time::SystemTime::now() - std::time::Duration::from_secs(7200);
            for (f, text) in [
                ("a.txt", "alpha"),
                ("sub/b.txt", "bravo"),
                ("run.sh", "#!/bin/sh\n"),
                ("c.txt", "charlie"),
            ] {
                let path = project.join(f);
                fs::write(&path, text).unwrap();
                fs::File::options()
                    .write(true)
                    .open(&path)
                    .unwrap()
                    .set_modified(old)
                    .unwrap();
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(project.join("run.sh"), fs::Permissions::from_mode(0o755))
                    .unwrap();
            }
            // The project holds a regular file with the bytes of what the copy has as a link.
            #[cfg(unix)]
            fs::write(project.join("link"), "alpha").unwrap();
            copy_project(&project, &hdir.join(WORK)).unwrap();
            fs::write(hdir.join(WORK).join("c.txt"), "charlia").unwrap();
            fs::write(hdir.join(WORK).join("d.txt"), "driver").unwrap();
            fs::create_dir_all(hdir.join(WORK).join("empty")).unwrap();
            #[cfg(unix)]
            {
                fs::remove_file(hdir.join(WORK).join("link")).unwrap();
                std::os::unix::fs::symlink("a.txt", hdir.join(WORK).join("link")).unwrap();
            }
            Self {
                _d: d,
                project,
                hdir,
            }
        }

        fn work(&self) -> PathBuf {
            self.hdir.join(WORK)
        }

        fn list(&self) -> PathBuf {
            self.hdir.join(THIN_LIST)
        }

        fn listed(&self) -> Vec<String> {
            let v: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(self.list()).unwrap()).unwrap();
            v["files"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| f["path"].as_str().unwrap().to_owned())
                .collect()
        }
    }

    #[test]
    fn thin_removes_identical_files_only_and_lists_exactly_those() {
        let t = Thin::new();
        let r = thin_files(&t.work(), &t.list(), &t.project).unwrap();
        let w = t.work();
        assert_eq!(t.listed(), ["a.txt", "run.sh", "sub/b.txt"]);
        assert_eq!(r.files, 3);
        assert_eq!(
            r.bytes,
            ("alpha".len() + "#!/bin/sh\n".len() + "bravo".len()) as u64
        );
        for gone in ["a.txt", "run.sh", "sub/b.txt"] {
            assert!(!w.join(gone).exists(), "{gone} should be gone");
        }
        // A changed file, a new file and a directory (empty or not) stay.
        assert_eq!(fs::read_to_string(w.join("c.txt")).unwrap(), "charlia");
        assert!(w.join("d.txt").is_file());
        assert!(w.join("empty").is_dir() && w.join("sub").is_dir());
        #[cfg(unix)]
        assert!(fs::symlink_metadata(w.join("link")).unwrap().is_symlink());
        let doc: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(t.list()).unwrap()).unwrap();
        assert_eq!(doc["dla_work_thin"], 1);
        assert_eq!(doc["bytes"], r.bytes);
        let first = &doc["files"][0];
        assert_eq!(first["len"], 5);
        assert_eq!(first["digest"].as_str().unwrap().len(), 64);
    }

    #[test]
    fn thin_writes_no_list_when_nothing_is_identical() {
        let d = tempfile::tempdir().unwrap();
        let (p, w) = (d.path().join("p"), d.path().join("w"));
        fs::create_dir_all(&p).unwrap();
        fs::create_dir_all(&w).unwrap();
        fs::write(p.join("a"), "1").unwrap();
        fs::write(w.join("a"), "2").unwrap();
        fs::write(w.join("new"), "x").unwrap();
        let list = d.path().join(THIN_LIST);
        assert_eq!(thin_files(&w, &list, &p).unwrap(), ThinReport::default());
        assert!(!list.exists() && w.join("a").is_file() && w.join("new").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn a_file_with_other_permissions_than_the_project_is_kept() {
        use std::os::unix::fs::PermissionsExt;
        let t = Thin::new();
        fs::set_permissions(t.work().join("a.txt"), fs::Permissions::from_mode(0o600)).unwrap();
        thin_files(&t.work(), &t.list(), &t.project).unwrap();
        assert!(t.work().join("a.txt").is_file());
        assert_eq!(t.listed(), ["run.sh", "sub/b.txt"]);
    }

    #[test]
    fn fill_brings_back_bytes_time_and_the_executable_bit() {
        let t = Thin::new();
        thin_files(&t.work(), &t.list(), &t.project).unwrap();
        let n = fill_files(&t.work(), &t.list(), &t.project).unwrap();
        assert_eq!(n, 3);
        assert!(!t.list().exists(), "the list goes once the files are back");
        for f in ["a.txt", "run.sh", "sub/b.txt"] {
            let (mine, theirs) = (t.work().join(f), t.project.join(f));
            assert_eq!(fs::read(&mine).unwrap(), fs::read(&theirs).unwrap(), "{f}");
            assert_eq!(
                fs::metadata(&mine).unwrap().modified().unwrap(),
                fs::metadata(&theirs).unwrap().modified().unwrap(),
                "{f} keeps the project's modification time"
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(t.work().join("run.sh"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o755
            );
        }
        assert_eq!(
            fs::read_to_string(t.work().join("c.txt")).unwrap(),
            "charlia"
        );
    }

    #[test]
    fn a_project_file_changed_after_the_thin_refuses_the_fill_and_changes_nothing() {
        let t = Thin::new();
        thin_files(&t.work(), &t.list(), &t.project).unwrap();
        fs::write(t.project.join("a.txt"), "ALPHA").unwrap();
        fs::remove_file(t.project.join("sub/b.txt")).unwrap();
        let before = fs::read_to_string(t.list()).unwrap();
        match fill_files(&t.work(), &t.list(), &t.project) {
            Err(FillError::Stale(paths)) => assert_eq!(paths, ["a.txt", "sub/b.txt"]),
            other => panic!("{other:?}"),
        }
        // The file that still matches was not brought back either.
        assert!(!t.work().join("run.sh").exists());
        assert!(!t.work().join("a.txt").exists());
        assert_eq!(fs::read_to_string(t.list()).unwrap(), before);
    }

    #[test]
    fn a_cut_thin_fills_the_others_and_keeps_the_file_that_is_still_there() {
        let t = Thin::new();
        thin_files(&t.work(), &t.list(), &t.project).unwrap();
        // A thin that stopped before it removed `a.txt`: the file is there, as the user left it.
        fs::write(t.work().join("a.txt"), "left behind").unwrap();
        let n = fill_files(&t.work(), &t.list(), &t.project).unwrap();
        assert_eq!(n, 2);
        assert_eq!(
            fs::read_to_string(t.work().join("a.txt")).unwrap(),
            "left behind"
        );
        assert!(t.work().join("run.sh").is_file() && t.work().join("sub/b.txt").is_file());
        assert!(!t.list().exists());
    }

    #[test]
    fn a_list_that_tries_to_leave_the_copy_is_stale_not_followed() {
        let t = Thin::new();
        fs::write(
            t.list(),
            r#"{"dla_work_thin":1,"files":[{"path":"../x","len":1,"digest":"00"}],"bytes":1}"#,
        )
        .unwrap();
        assert!(matches!(
            fill_files(&t.work(), &t.list(), &t.project),
            Err(FillError::Stale(p)) if p == ["../x"]
        ));
    }

    #[test]
    fn a_second_thin_of_a_thinned_harness_does_nothing() {
        let t = Thin::new();
        let first = thin_harness(&t.hdir, &t.project).unwrap();
        assert!(
            matches!(first, Thinned::Done(r) if r.files == 3),
            "{first:?}"
        );
        let list = fs::read_to_string(t.list()).unwrap();
        // Something new that is identical to the project appears: the list is not touched.
        fs::write(t.work().join("a.txt"), "alpha").unwrap();
        assert_eq!(thin_harness(&t.hdir, &t.project).unwrap(), Thinned::Already);
        assert_eq!(fs::read_to_string(t.list()).unwrap(), list);
        assert!(t.work().join("a.txt").is_file());
    }

    #[test]
    fn a_harness_whose_lock_is_held_is_not_thinned_and_one_in_use_is_filled_first() {
        let t = Thin::new();
        let held = open_lock(&t.hdir).unwrap();
        held.lock_shared().unwrap();
        assert_eq!(thin_harness(&t.hdir, &t.project).unwrap(), Thinned::Busy);
        assert!(!t.list().exists() && t.work().join("a.txt").is_file());
        drop(held);
        thin_harness(&t.hdir, &t.project).unwrap();
        assert!(!t.work().join("a.txt").exists());
        let guard = use_work(&t.hdir, &t.project, "python").unwrap();
        assert!(t.work().join("a.txt").is_file() && !t.list().exists());
        // While a user holds the copy, the thinner leaves it alone.
        assert_eq!(thin_harness(&t.hdir, &t.project).unwrap(), Thinned::Busy);
        drop(guard);
        assert!(matches!(
            thin_harness(&t.hdir, &t.project).unwrap(),
            Thinned::Done(_)
        ));
    }

    #[test]
    fn a_lock_that_is_held_only_for_a_moment_is_waited_for() {
        // What a process another thread is starting does to a lock that was just released:
        // it reads as held for an instant. The thinner must not answer "busy" for that.
        let t = Thin::new();
        let held = open_lock(&t.hdir).unwrap();
        held.lock_shared().unwrap();
        let release = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            drop(held);
        });
        assert!(matches!(
            thin_harness(&t.hdir, &t.project).unwrap(),
            Thinned::Done(_)
        ));
        release.join().unwrap();
    }

    #[test]
    fn using_a_copy_sets_the_last_used_time() {
        let t = Thin::new();
        let lock = open_lock(&t.hdir).unwrap();
        let old = std::time::SystemTime::now() - std::time::Duration::from_secs(86_400);
        lock.set_modified(old).unwrap();
        drop(lock);
        let _g = use_work(&t.hdir, &t.project, "python").unwrap();
        let used = fs::metadata(t.hdir.join(WORK_LOCK))
            .unwrap()
            .modified()
            .unwrap();
        assert!(
            used > old + std::time::Duration::from_secs(3600),
            "{used:?}"
        );
    }

    #[test]
    fn the_refusal_for_a_stale_copy_names_the_count_and_the_first_paths() {
        let t = Thin::new();
        for i in 0..6 {
            fs::write(t.project.join(format!("m{i}.txt")), "m").unwrap();
            fs::write(t.work().join(format!("m{i}.txt")), "m").unwrap();
        }
        thin_harness(&t.hdir, &t.project).unwrap();
        for i in 0..6 {
            fs::write(t.project.join(format!("m{i}.txt")), "changed").unwrap();
        }
        let r = match use_work(&t.hdir, &t.project, "python") {
            Err(r) => r,
            Ok(_) => panic!("the project changed"),
        };
        assert_eq!(r.code, "work-copy-stale");
        assert!(r.message.contains("6 file(s)"), "{}", r.message);
        assert!(
            r.message.contains("m0.txt, m1.txt, m2.txt and 3 more"),
            "{}",
            r.message
        );
        assert!(r.fix.contains("dla harness gen --lang python"), "{}", r.fix);
        assert!(r.fix.contains("reseal"), "{}", r.fix);
        // Nothing was brought back and the list stays.
        assert!(t.list().exists() && !t.work().join("a.txt").exists());
    }

    #[test]
    fn thin_run_keeps_the_last_used_and_the_new_one() {
        let d = tempfile::tempdir().unwrap();
        let project = d.path().join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("f"), "same").unwrap();
        let run = d.path().join("run");
        let base = std::time::SystemTime::now() - std::time::Duration::from_secs(10_000);
        for n in 1..=7u64 {
            let h = run.join("harness").join(format!("H{n}"));
            fs::create_dir_all(h.join(WORK)).unwrap();
            fs::write(h.join(WORK).join("f"), "same").unwrap();
            fs::write(h.join(crate::harness::MANIFEST), "{}").unwrap();
            // H3 was used last of all but H7; H7 is the new one.
            let age = if n == 3 { 900 } else { n * 10 };
            let when = base + std::time::Duration::from_secs(age);
            if n == 2 && cfg!(unix) {
                // No lock file yet: the time of the directory says when it was used. (Windows
                // does not open a directory this way to set its time: there H2 gets a lock
                // file like the others.)
                fs::File::open(&h).unwrap().set_modified(when).unwrap();
            } else {
                open_lock(&h).unwrap().set_modified(when).unwrap();
            }
        }
        // A directory without a manifest is being generated: left alone.
        fs::create_dir_all(run.join("harness/H8").join(WORK)).unwrap();
        fs::write(run.join("harness/H8").join(WORK).join("f"), "same").unwrap();
        let r = thin_run(&run, &project, KEEP_RECENT, Some("H7"));
        // Used last: H3 (900), H7 (70), H6 (60), H5 (50) are kept; H7 also as the new one.
        assert_eq!(r.thinned, ["H1", "H2", "H4"]);
        assert_eq!(r.bytes, 12);
        assert!(r.skipped.is_empty(), "{:?}", r.skipped);
        assert!(run.join("harness/H8").join(WORK).join("f").is_file());
        for kept in ["H3", "H5", "H6", "H7"] {
            assert!(
                run.join("harness")
                    .join(kept)
                    .join(WORK)
                    .join("f")
                    .is_file()
            );
        }
        // The lock file the thinner made for H2 did not make it look used just now: the four
        // used last are still the same four, and nothing else is left to thin.
        let second = thin_run(&run, &project, KEEP_RECENT, None);
        assert!(second.thinned.is_empty(), "{second:?}");
        // The new one is left alone whatever its age.
        let again = thin_run(&run, &project, 0, Some("H7"));
        assert_eq!(again.thinned, ["H3", "H5", "H6"]);
        assert!(run.join("harness/H7").join(WORK).join("f").is_file());
    }
    /// A TypeScript-shaped Node project: sources tracked, `lib` and `node_modules` ignored.
    #[cfg(unix)]
    fn node_project(d: &Path, main: &str) -> Option<PathBuf> {
        let p = d.join("p");
        for (f, text) in [
            (".gitignore", "lib\nnode_modules\n"),
            (
                "package.json",
                &format!("{{\"name\":\"x\",\"main\":\"{main}\"}}"),
            ),
            ("src/a.ts", "export const a = 1;\n"),
            ("lib/index.js", "exports.a = 1;\n"),
            ("lib/deep/chunk.js", "x\n"),
            ("lib/build/inner.js", "y\n"),
            ("node_modules/dep/index.js", "d\n"),
        ] {
            fs::create_dir_all(p.join(f).parent().unwrap()).unwrap();
            fs::write(p.join(f), text).unwrap();
        }
        (git(&p, &["init", "-q"]) && git(&p, &["add", "."])).then_some(p)
    }

    #[cfg(unix)]
    #[test]
    fn a_node_copy_takes_the_ignored_entry_point_directory_and_links_node_modules() {
        let d = tempfile::tempdir().unwrap();
        let Some(p) = node_project(d.path(), "./lib/index.js") else {
            return;
        };
        let dest = d.path().join("w");
        let r = copy_project_for(&p, &dest, None, Some(Runtime::Node)).unwrap();
        for f in [
            "lib/index.js",
            "lib/deep/chunk.js",
            "lib/build/inner.js",
            "src/a.ts",
        ] {
            assert!(dest.join(f).is_file(), "{f}");
        }
        assert_eq!(r.build_output_kept, 3);
        assert_eq!(r.node_modules_linked, 1);
        let link = dest.join("node_modules");
        assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
        assert_eq!(
            fs::read_link(&link).unwrap(),
            p.join("node_modules").canonicalize().unwrap()
        );
        assert!(link.join("dep/index.js").is_file());
        assert!(
            r.gaps
                .iter()
                .any(|g| g.contains("3 file(s) of build output"))
        );
        assert!(r.gaps.iter().any(|g| g.contains("node_modules director")));
        // Another runtime's copy is unchanged.
        let other = d.path().join("o");
        let r = copy_project_for(&p, &other, None, Some(Runtime::Go)).unwrap();
        assert!(!other.join("lib").exists() && !other.join("node_modules").exists());
        assert_eq!((r.build_output_kept, r.node_modules_linked), (0, 0));
    }

    #[cfg(unix)]
    #[test]
    fn a_node_copy_without_ignored_entry_points_is_unchanged() {
        let d = tempfile::tempdir().unwrap();
        let Some(p) = node_project(d.path(), "./src/a.ts") else {
            return;
        };
        let dest = d.path().join("w");
        let r = copy_project_for(&p, &dest, None, Some(Runtime::Node)).unwrap();
        assert_eq!(r.build_output_kept, 0);
        assert!(!dest.join("lib").exists());
        assert!(!r.gaps.iter().any(|g| g.contains("build output")));
        // No package.json at all: nothing is linked either.
        let q = d.path().join("q");
        fs::create_dir_all(q.join("node_modules")).unwrap();
        fs::write(q.join("a.js"), "1").unwrap();
        let dest = d.path().join("w2");
        let r = copy_project_for(&q, &dest, None, Some(Runtime::Node)).unwrap();
        assert_eq!((r.build_output_kept, r.node_modules_linked), (0, 0));
        assert!(!dest.join("node_modules").exists());
    }

    #[cfg(unix)]
    #[test]
    fn thinning_keeps_the_node_modules_link_and_thins_the_copied_build_output() {
        let d = tempfile::tempdir().unwrap();
        let Some(p) = node_project(d.path(), "./lib/index.js") else {
            return;
        };
        let (work, list) = (d.path().join("work"), d.path().join("work-thinned.json"));
        copy_project_for(&p, &work, None, Some(Runtime::Node)).unwrap();
        let t = thin_files(&work, &list, &p).unwrap();
        assert!(t.files >= 4, "{t:?}");
        assert!(!work.join("lib/index.js").exists());
        assert!(
            fs::symlink_metadata(work.join("node_modules"))
                .unwrap()
                .is_symlink()
        );
        assert!(work.join("node_modules/dep/index.js").is_file());
        fill_files(&work, &list, &p).unwrap();
        assert!(work.join("lib/index.js").is_file());
        assert!(work.join("node_modules/dep/index.js").is_file());
    }
}
