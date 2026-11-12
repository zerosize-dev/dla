//! The scratch copy of the project a harness runs in (`work/`).
//!
//! DLA never edits project sources: the copy is the project's tracked files (or, outside a
//! git repository, every file except the build-output directories of
//! [`crate::model::SKIP_DIRS`] and `.dla`), plus whatever generation drops in.

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use dla_census::parse::Lang;
use dla_core::{runtime::Runtime, state::Scope};

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
/// build directories.
fn keeps_for_build(scope: &Scope, rel: &Path, slash: &str) -> bool {
    if under_root_build_dir(rel) {
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
        if is_dropped_build_output(&rel, from_git, &tracked) {
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
        // Build outputs of another tree: rebuilt in the copy, never copied. Outside git
        // every library counts as shipped; in git only the tracked ones do.
        if is_dropped_build_output(&rel, from_git, &tracked) {
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
    }
    for (why, n) in reasons {
        report
            .gaps
            .push(format!("scratch copy: {n} path(s) not copied ({why})"));
    }
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
