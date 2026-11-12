//! What a copy of a project takes, shared by the probe's build copy and the harness's
//! scratch copy: the git file listing, the build outputs left out (objects, untracked
//! libraries) and the symlink rule.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
};

/// Project files per `git ls-files` (tracked, plus untracked files that are not ignored),
/// relative to `project`, sorted; or `None` when `project` is not a git work tree, git is
/// missing, or git lists no file (an enclosing repository ignores the project).
pub fn git_files(project: &Path) -> Option<Vec<PathBuf>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(project)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let top = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["rev-parse", "--show-prefix"])
        .output()
        .ok()?;
    if !top.status.success() {
        return None;
    }
    let mut files: Vec<PathBuf> = out
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .filter_map(|s| std::str::from_utf8(s).ok())
        .map(PathBuf::from)
        .collect();
    files.sort();
    files.dedup();
    // An enclosing repository that ignores the project lists nothing: no listing at all,
    // so a copy walks the tree instead of copying nothing.
    (!files.is_empty()).then_some(files)
}

/// The text of the link `from` (inside `src`) for its copy in `dest`: an absolute target
/// inside `src` is re-pointed into `dest`, so the copy never leads back to the original.
#[cfg(unix)]
pub fn repointed_link(src: &Path, dest: &Path, from: &Path) -> std::io::Result<PathBuf> {
    let target = std::fs::read_link(from)?;
    if !target.is_absolute() {
        return Ok(target);
    }
    let mut roots = vec![src.to_path_buf()];
    if let Ok(c) = src.canonicalize() {
        roots.push(c);
    }
    for root in roots {
        if let Ok(inside) = target.strip_prefix(&root) {
            return Ok(dest.join(inside));
        }
    }
    Ok(target)
}

/// The tracked paths of the work tree at `project`, to tell a shipped library from a
/// build output.
pub fn git_tracked(project: &Path) -> HashSet<PathBuf> {
    Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["ls-files", "-z", "--cached"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            o.stdout
                .split(|b| *b == 0)
                .filter_map(|s| std::str::from_utf8(s).ok())
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

/// Object files: compiler outputs that record the path of the tree they were built in.
/// A copy never takes them unless git tracks them: `make` would find them newer than their sources and not
/// rebuild them, and the shim would then attribute allocations through the debug paths of
/// the other tree.
pub const OBJECT_EXTS: [&str; 3] = ["o", "obj", "lo"];

/// Archives and shared libraries: the same, but a project may ship a prebuilt one.
pub const LIBRARY_EXTS: [&str; 3] = ["a", "so", "dylib"];

pub fn has_ext(rel: &Path, exts: &[&str]) -> bool {
    rel.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| exts.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// True for an archive or shared library, versioned names included: `libx.a`, `libx.so`,
/// `libx.so.1`, `libx.so.1.2.3` and `libx.dylib`.
pub fn is_library(rel: &Path) -> bool {
    if has_ext(rel, &LIBRARY_EXTS) {
        return true;
    }
    let Some(name) = rel.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let mut parts = name.rsplit('.').peekable();
    let mut versions = 0;
    while parts
        .peek()
        .is_some_and(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    {
        parts.next();
        versions += 1;
    }
    versions > 0
        && parts.next().is_some_and(|p| p.eq_ignore_ascii_case("so"))
        && parts.next().is_some()
}

/// Whether a copy leaves the library `rel` out: only when the project is a git work tree,
/// git does not track it (a build output of another tree) and it is not under a root-level
/// build directory. Outside git every library counts as shipped.
pub fn drops_library(rel: &Path, from_git: bool, tracked: &HashSet<PathBuf>) -> bool {
    from_git && is_library(rel) && !tracked.contains(rel) && !under_root_build_dir(rel)
}

pub fn under_root_build_dir(rel: &Path) -> bool {
    rel.components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .is_some_and(|c| ROOT_BUILD_DIRS.contains(&c))
}

/// Root-level directories that hold build configuration or dependencies, copied whole
/// under a scope.
pub const ROOT_BUILD_DIRS: [&str; 4] = ["gradle", "buildSrc", ".mvn", "vendor"];

/// Whether a copy leaves `rel` out as a build output of another tree: an object file that
/// git does not track (outside git there is no telling, so every object is left out), a
/// library per [`drops_library`]. A tracked object is a shipped input (a prebuilt `blob.o`
/// the Makefile links) and is kept. Both copies build in the files this keeps.
pub fn is_dropped_build_output(rel: &Path, from_git: bool, tracked: &HashSet<PathBuf>) -> bool {
    (has_ext(rel, &OBJECT_EXTS) && !(from_git && tracked.contains(rel)))
        || drops_library(rel, from_git, tracked)
}

/// The text of the link `src` (inside `project`) for its copy in `dest`: relative links
/// keep their text, absolute ones are re-pointed into `dest`. The reason as an error when
/// the link dangles or points outside the project, which no copy takes.
pub fn link_for_copy(project: &Path, dest: &Path, src: &Path) -> Result<PathBuf, String> {
    let target = std::fs::read_link(src).map_err(|e| format!("unreadable symlink: {e}"))?;
    let abs = if target.is_absolute() {
        target.clone()
    } else {
        src.parent().unwrap_or(project).join(&target)
    };
    let root = project
        .canonicalize()
        .map_err(|e| format!("project not resolvable: {e}"))?;
    let real = abs
        .canonicalize()
        .map_err(|_| "dangling symlink".to_owned())?;
    let Ok(inside) = real.strip_prefix(&root) else {
        return Err("symlink pointing outside the project".to_owned());
    };
    Ok(if target.is_absolute() {
        dest.join(inside)
    } else {
        target
    })
}
