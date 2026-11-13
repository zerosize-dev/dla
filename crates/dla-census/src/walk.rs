//! Project file discovery for the census.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dla_core::runtime::Runtime;

use crate::{model::SourceFile, parse::Lang};

/// Directories never descended into.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".dla",
    "target",
    "build",
    "node_modules",
    "venv",
    ".venv",
    "env",
    "out",
    "dist",
    "__pycache__",
    ".gradle",
    ".idea",
    "site-packages",
    "vendor",
    ".mypy_cache",
    ".pytest_cache",
];

/// Largest source file read (bytes); bigger files are generated or data.
const MAX_FILE_BYTES: u64 = 2_000_000;

/// Every file under `root`, as sorted `/`-separated relative paths, for which
/// `keep(name)` holds. When `through_build_dirs` is set, build output directories are
/// searched too (class files live there).
pub fn list_files(
    root: &Path,
    through_build_dirs: bool,
    keep: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack: Vec<(PathBuf, String)> = vec![(root.to_path_buf(), String::new())];
    while let Some((dir, rel)) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Ok(ft) = e.file_type() else { continue };
            let child_rel = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            if ft.is_dir() {
                let build_dir = matches!(name.as_str(), "target" | "build" | "out" | "dist");
                let always = matches!(name.as_str(), ".git" | ".dla" | "node_modules");
                if always
                    || (SKIP_DIRS.contains(&name.as_str()) && !(through_build_dirs && build_dir))
                {
                    continue;
                }
                stack.push((e.path(), child_rel));
            } else if ft.is_file() && keep(&child_rel) {
                out.push(child_rel);
            }
        }
    }
    out.sort();
    out
}

/// True for files that hold tests rather than production code.
pub fn is_test_path(rel: &str) -> bool {
    let lower = rel.to_ascii_lowercase();
    let comps: Vec<&str> = lower.split('/').collect();
    let file = comps.last().copied().unwrap_or("");
    let dir_test = comps[..comps.len().saturating_sub(1)].iter().any(|c| {
        matches!(
            *c,
            "test" | "tests" | "__tests__" | "spec" | "specs" | "testdata" | "testing" | "e2e"
        )
    });
    let orig = rel.rsplit('/').next().unwrap_or(rel);
    let stem = orig.rsplit_once('.').map_or(orig, |(s, _)| s);
    let lstem = stem.to_ascii_lowercase();
    dir_test
        || stem.ends_with("Test")
        || stem.ends_with("Tests")
        || stem.ends_with("IT")
        || lstem.ends_with("_test")
        || lstem.ends_with("_tests")
        || lstem.ends_with("-test")
        || lstem.ends_with(".test")
        || lstem.ends_with(".spec")
        || lstem.starts_with("test_")
        || lstem.starts_with("test-")
        || file == "conftest.py"
}

/// All source files of a runtime under `root`, read into memory. Unreadable and
/// oversized files are skipped.
pub fn source_files(root: &Path, runtime: Runtime) -> Vec<SourceFile> {
    source_files_in(root, runtime, &|_| true).0
}

/// Number of non-test source files of `runtime` under `root` that `in_scope` rejects
/// (names only; nothing is read).
pub fn count_out_of_scope(root: &Path, runtime: Runtime, in_scope: &dyn Fn(&str) -> bool) -> u64 {
    let rels = list_files(root, false, &|rel| {
        Lang::from_path(Path::new(rel)).is_some_and(|l| l.runtime() == runtime)
    });
    rels.iter()
        .filter(|r| !dla_core::paths::is_build_logic(r) && !in_scope(r) && !is_test_path(r))
        .count() as u64
}

/// Number of non-test source files of `runtime` under `root` that are build logic
/// (`*.gradle.kts`, `buildSrc/**`): they are never read (names only).
pub fn count_build_logic(root: &Path, runtime: Runtime) -> u64 {
    let rels = list_files(root, false, &|rel| {
        Lang::from_path(Path::new(rel)).is_some_and(|l| l.runtime() == runtime)
    });
    rels.iter()
        .filter(|r| dla_core::paths::is_build_logic(r) && !is_test_path(r))
        .count() as u64
}

/// Like [`source_files`], restricted by `in_scope` (project-relative path -> inside the
/// run's scope). Returns the files read and the number of non-test source files of the
/// runtime that were skipped because they lie outside the scope.
pub fn source_files_in(
    root: &Path,
    runtime: Runtime,
    in_scope: &dyn Fn(&str) -> bool,
) -> (Vec<SourceFile>, u64) {
    let rels = list_files(root, false, &|rel| {
        Lang::from_path(Path::new(rel)).is_some_and(|l| l.runtime() == runtime)
    });
    let mut out = Vec::new();
    let mut skipped = 0u64;
    for rel in rels {
        // Build logic is not part of the measured process: not read, not counted as out of
        // scope (it has its own count, `count_build_logic`).
        if dla_core::paths::is_build_logic(&rel) {
            continue;
        }
        if !in_scope(&rel) {
            if !is_test_path(&rel) {
                skipped += 1;
            }
            continue;
        }
        let path = root.join(&rel);
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if meta.len() > MAX_FILE_BYTES {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Some(lang) = Lang::from_path(Path::new(&rel)) else {
            continue;
        };
        out.push(SourceFile {
            is_test: is_test_path(&rel),
            rel,
            text,
            lang,
        });
    }
    (out, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        for p in [
            "src/test/java/a/ChargeServiceTest.java",
            "test/shop/OrderServiceTest.java",
            "internal/x/render_test.go",
            "tests/test_ingest.py",
            "test/a.test.js",
            "pkg/test_cache.c",
            "tests/helper.rs",
        ] {
            assert!(is_test_path(p), "{p}");
        }
        for p in [
            "src/shop/Catalog.java",
            "internal/render/pool.go",
            "ingest/registry.py",
            "src/latest.js",
        ] {
            assert!(!is_test_path(p), "{p}");
        }
    }

    #[test]
    fn walk_skips_build_and_hidden_dirs_unless_asked() {
        let t = tempfile::tempdir().unwrap();
        for f in [
            "a/X.java",
            "node_modules/m/Y.java",
            "target/classes/Z.class",
            ".git/c",
        ] {
            let p = t.path().join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, "x").unwrap();
        }
        let all = list_files(t.path(), false, &|_| true);
        assert_eq!(all, ["a/X.java"]);
        let with_build = list_files(t.path(), true, &|_| true);
        assert_eq!(with_build, ["a/X.java", "target/classes/Z.class"]);
    }
}
