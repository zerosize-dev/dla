//! The scratch copy of the project a harness runs in (`work/`).
//!
//! DLA never edits project sources: the copy is the project's tracked files (or, outside a
//! git repository, every file except the build-output directories of
//! [`crate::model::SKIP_DIRS`] and `.dla`), plus whatever generation drops in.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::model::SKIP_DIRS;

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
}

/// Project files per `git ls-files` (tracked, plus untracked files that are not ignored),
/// or `None` when `project` is not a git work tree or git is missing.
fn git_files(project: &Path) -> Option<Vec<PathBuf>> {
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
    Some(files)
}

/// Directories skipped wherever they are: they never hold project sources.
const SKIP_ANYWHERE: [&str; 4] = ["node_modules", "__pycache__", ".git", ".dla"];

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
    let target = fs::read_link(src).map_err(|e| format!("unreadable symlink: {e}"))?;
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
    // Relative links keep their text; absolute ones are re-pointed into the copy.
    let link = if target.is_absolute() {
        dest.join(inside)
    } else {
        target
    };
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

/// Copies the project into `dest` (created). The `.dla` directory is never copied.
pub fn copy_project(project: &Path, dest: &Path) -> std::io::Result<CopyReport> {
    fs::create_dir_all(dest)?;
    let (files, from_git, left_out) = match git_files(project) {
        Some(f) => (f, true, Vec::new()),
        None => {
            let (f, l) = walk_files(project);
            (f, false, l)
        }
    };
    let mut report = CopyReport {
        from_git,
        ..CopyReport::default()
    };
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    let mut skip = |report: &mut CopyReport, why: String| {
        report.skipped += 1;
        *reasons.entry(why).or_default() += 1;
    };
    for rel in files {
        if is_dla_path(&rel) {
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
        if fs::copy(&src, &to).is_err() {
            skip(&mut report, "unreadable file".to_owned());
            continue;
        }
        report.files += 1;
    }
    for (why, n) in reasons {
        report
            .gaps
            .push(format!("scratch copy: {n} path(s) not copied ({why})"));
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
