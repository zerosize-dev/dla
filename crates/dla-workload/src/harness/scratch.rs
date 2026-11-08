//! The scratch copy of the project a harness runs in (`work/`).
//!
//! DLA never edits project sources: the copy is the project's tracked files (or, outside a
//! git repository, every file except the build-output directories of
//! [`crate::model::SKIP_DIRS`] and `.dla`), plus whatever generation drops in.

use std::{
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

/// Every file under `root` except skipped directories, relative, sorted.
fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let Ok(rd) = fs::read_dir(root.join(&rel)) else {
            continue;
        };
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            let name = e.file_name();
            let child = rel.join(&name);
            if ft.is_dir() {
                if !SKIP_DIRS.contains(&name.to_string_lossy().as_ref()) {
                    stack.push(child);
                }
            } else if ft.is_file() {
                out.push(child);
            }
        }
    }
    out.sort();
    out
}

fn is_dla_path(rel: &Path) -> bool {
    rel.components()
        .next()
        .is_some_and(|c| c.as_os_str() == ".dla")
}

/// Copies the project into `dest` (created). The `.dla` directory is never copied.
pub fn copy_project(project: &Path, dest: &Path) -> std::io::Result<CopyReport> {
    fs::create_dir_all(dest)?;
    let (files, from_git) = match git_files(project) {
        Some(f) => (f, true),
        None => (walk_files(project), false),
    };
    let mut report = CopyReport {
        from_git,
        ..CopyReport::default()
    };
    for rel in files {
        if is_dla_path(&rel) {
            continue;
        }
        if report.files >= MAX_FILES {
            report.skipped += 1;
            continue;
        }
        let src = project.join(&rel);
        // `git ls-files` lists symlinks and deleted-but-tracked paths too.
        let Ok(meta) = fs::symlink_metadata(&src) else {
            report.skipped += 1;
            continue;
        };
        if !meta.is_file() {
            report.skipped += 1;
            continue;
        }
        let to = dest.join(&rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        if fs::copy(&src, &to).is_err() {
            report.skipped += 1;
            continue;
        }
        report.files += 1;
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
    }
}
