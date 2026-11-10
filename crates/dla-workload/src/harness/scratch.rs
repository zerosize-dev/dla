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

use dla_census::parse::Lang;
use dla_core::{runtime::Runtime, state::Scope};

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
    /// Files left out because they lie outside the run's scope.
    pub out_of_scope: usize,
    /// Non-test source files among them, by runtime.
    pub out_of_scope_sources: BTreeMap<Runtime, u64>,
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

/// Root-level directories that hold build configuration or dependencies, copied whole
/// under a scope.
const ROOT_BUILD_DIRS: [&str; 4] = ["gradle", "buildSrc", ".mvn", "vendor"];

/// Whether a file outside the scope is still needed to build what is inside it: any
/// non-source file in a directory above the scope (`svc/go.mod` for the scope
/// `svc/a`, a parent `pom.xml` or `Cargo.toml`) and everything under the root-level
/// build directories.
fn keeps_for_build(scope: &Scope, rel: &Path, slash: &str) -> bool {
    if rel
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .is_some_and(|c| ROOT_BUILD_DIRS.contains(&c))
    {
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
/// directories of [`SKIP_ANYWHERE`] wherever they are, and the root-level ones of
/// [`SKIP_DIRS`]. Symlinks are recreated as they are on Unix and skipped elsewhere.
/// `dla intervene` clones a harness's `work/` this way. Returns the files copied.
pub fn copy_tree(src: &Path, dest: &Path) -> std::io::Result<usize> {
    fs::create_dir_all(dest)?;
    let (files, _left_out) = walk_files(src);
    let mut copied = 0;
    for rel in files {
        let from = src.join(&rel);
        let to = dest.join(&rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        let meta = fs::symlink_metadata(&from)?;
        if meta.is_symlink() {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(fs::read_link(&from)?, &to)?;
                copied += 1;
            }
            continue;
        }
        fs::copy(&from, &to)?;
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
        if let Some(scope) = scope {
            let slash = dla_core::paths::to_slash(&rel);
            let at_root = rel.components().count() == 1;
            if !at_root && !scope.includes(&slash) && !keeps_for_build(scope, &rel, &slash) {
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
        assert_eq!(copy_tree(&w, &dest).unwrap(), 3);
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
