//! Project file discovery for the census.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dla_core::{runtime::Runtime, vendored::Defaults};

use crate::{model::SourceFile, parse::Lang};

/// Directories never descended into. Vendored directories, `site-packages` and virtual
/// environments are not here: [`Defaults`] decides them, so that `--include` can bring them
/// back.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".dla",
    "target",
    "build",
    "venv",
    ".venv",
    "env",
    "out",
    "dist",
    "__pycache__",
    ".gradle",
    ".idea",
    ".mypy_cache",
    ".pytest_cache",
];

/// Largest source file read (bytes); bigger files are generated or data.
const MAX_FILE_BYTES: u64 = 2_000_000;

/// Every file under `root`, as sorted `/`-separated relative paths, for which
/// `keep(name)` holds. When `through_build_dirs` is set, build output directories are
/// searched too (class files live there). Vendored directories and generated files are left
/// out ([`Defaults`] without `--include`).
pub fn list_files(
    root: &Path,
    through_build_dirs: bool,
    keep: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    list_files_with(
        root,
        through_build_dirs,
        keep,
        &Defaults::for_project(root, &[]),
    )
}

/// [`list_files`] with explicit default exclusions (the run's `--include` paths).
pub fn list_files_with(
    root: &Path,
    through_build_dirs: bool,
    keep: &dyn Fn(&str) -> bool,
    defaults: &Defaults,
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
                let always = matches!(name.as_str(), ".git" | ".dla");
                if always
                    || defaults.prune_dir(&child_rel).is_some()
                    || (SKIP_DIRS.contains(&name.as_str()) && !(through_build_dirs && build_dir))
                {
                    continue;
                }
                stack.push((e.path(), child_rel));
            } else if ft.is_file() && keep(&child_rel) && defaults.walk_file(&child_rel).is_none() {
                out.push(child_rel);
            }
        }
    }
    out.sort();
    out
}

/// True for a TypeScript declaration file (`*.d.ts`, `*.d.mts`, `*.d.cts`): it holds types
/// and no code that could hold anything.
pub fn is_declaration_file(rel: &str) -> bool {
    let lower = rel.to_ascii_lowercase();
    [".d.ts", ".d.mts", ".d.cts"]
        .iter()
        .any(|e| lower.ends_with(e))
}

/// Resolves `.` and `..` in a `/`-separated path without touching the file system; `None`
/// when the path leaves its root.
fn normalize_slash(path: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for c in path.split('/') {
        match c {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            c => parts.push(c),
        }
    }
    Some(parts.join("/"))
}

/// True for a JavaScript file that is the build output of sources of the project: it ends with a
/// `sourceMappingURL` comment, the map it names is a file next to it, and the map lists at
/// least one source (below its `sourceRoot`) that is a file of the project. The sources are
/// read instead. A map that cannot be read, an inline map and a map whose sources are not in
/// the project (a map of a vendored bundle) leave the file as it is.
pub fn is_built_from_sources(root: &Path, rel: &str) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut f) = fs::File::open(root.join(rel)) else {
        return false;
    };
    let Ok(len) = f.metadata().map(|m| m.len()) else {
        return false;
    };
    let tail = len.min(1024);
    let mut buf = vec![0u8; tail as usize];
    if f.seek(SeekFrom::Start(len - tail)).is_err() || f.read_exact(&mut buf).is_err() {
        return false;
    }
    let tail = String::from_utf8_lossy(&buf);
    let Some(url) = tail
        .lines()
        .rev()
        .find_map(|l| {
            let l = l.trim();
            l.strip_prefix("//# sourceMappingURL=")
                .or_else(|| l.strip_prefix("//@ sourceMappingURL="))
        })
        .map(str::trim)
    else {
        return false;
    };
    if url.is_empty() || url.starts_with("data:") || url.contains("://") {
        return false;
    }
    let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
    let join = |base: &str, name: &str| {
        if base.is_empty() {
            name.to_owned()
        } else {
            format!("{base}/{name}")
        }
    };
    let Some(map_rel) = normalize_slash(&join(dir, url)) else {
        return false;
    };
    let Ok(text) = fs::read_to_string(root.join(&map_rel)) else {
        return false;
    };
    let Ok(map) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    let map_dir = map_rel.rsplit_once('/').map_or("", |(d, _)| d);
    let source_root = map.get("sourceRoot").and_then(|v| v.as_str()).unwrap_or("");
    let base = join(map_dir, source_root);
    map.get("sources")
        .and_then(|v| v.as_array())
        .is_some_and(|sources| {
            sources
                .iter()
                .filter_map(|s| s.as_str())
                .filter(|s| !s.contains("://") && !s.starts_with('/'))
                .filter_map(|s| normalize_slash(&join(&base, s)))
                .any(|p| p != rel && root.join(&p).is_file())
        })
}

/// What the Node census does not read: declaration files and the build output of sources it
/// reads instead.
fn is_node_derived(root: &Path, rel: &str) -> bool {
    is_declaration_file(rel)
        || (matches!(
            Path::new(rel).extension().and_then(|e| e.to_str()),
            Some("js" | "mjs" | "cjs")
        ) && is_built_from_sources(root, rel))
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
    source_files_in(root, runtime, &|_| true, &Defaults::for_project(root, &[])).0
}

/// Number of non-test source files of `runtime` under `root` that `in_scope` rejects
/// (names only; nothing is read).
pub fn count_out_of_scope(
    root: &Path,
    runtime: Runtime,
    in_scope: &dyn Fn(&str) -> bool,
    defaults: &Defaults,
) -> u64 {
    let rels = list_files_with(
        root,
        false,
        &|rel| Lang::from_path(Path::new(rel)).is_some_and(|l| l.runtime() == runtime),
        defaults,
    );
    rels.iter()
        .filter(|r| {
            !dla_core::paths::is_build_logic(r)
                && !in_scope(r)
                && !is_test_path(r)
                && !(runtime == Runtime::Node && is_node_derived(root, r))
        })
        .count() as u64
}

/// Number of non-test source files of `runtime` under `root` that are build logic
/// (`*.gradle.kts`, `buildSrc/**`): they are never read (names only).
pub fn count_build_logic(root: &Path, runtime: Runtime, defaults: &Defaults) -> u64 {
    let rels = list_files_with(
        root,
        false,
        &|rel| Lang::from_path(Path::new(rel)).is_some_and(|l| l.runtime() == runtime),
        defaults,
    );
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
    defaults: &Defaults,
) -> (Vec<SourceFile>, u64) {
    let rels = list_files_with(
        root,
        false,
        &|rel| Lang::from_path(Path::new(rel)).is_some_and(|l| l.runtime() == runtime),
        defaults,
    );
    let mut out = Vec::new();
    let mut skipped = 0u64;
    for rel in rels {
        // Build logic is not part of the measured process: not read, not counted as out of
        // scope (it has its own count, `count_build_logic`).
        if dla_core::paths::is_build_logic(&rel) {
            continue;
        }
        // Declaration files and the build output of sources that are read themselves: not
        // candidates, not counted as out of scope.
        if runtime == Runtime::Node && is_node_derived(root, &rel) {
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
    fn vendored_and_generated_sources_are_not_listed_unless_brought_back() {
        let t = tempfile::tempdir().unwrap();
        let deep = format!(
            "{}/* Generated by Cython, mentioned deep inside */\n",
            "int n;\n".repeat(30)
        );
        for (f, text) in [
            ("app/a.c", "int a;\n"),
            ("app/_fast.c", "/* Generated by Cython 3.0.10 */\nint g;\n"),
            ("app/notes.c", deep.as_str()),
            ("vendor/llhttp/p.c", "int p;\n"),
            ("third_party/z/z.c", "int z;\n"),
            ("lib/node_modules/m/m.c", "int m;\n"),
            ("svc/x.pb.cc", "int q;\n"),
        ] {
            let p = t.path().join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        let c = |r: &str| r.ends_with(".c") || r.ends_with(".cc");
        assert_eq!(list_files(t.path(), false, &c), ["app/a.c", "app/notes.c"]);
        let back = Defaults::for_project(t.path(), &["vendor/llhttp".into(), "app/_fast.c".into()]);
        assert_eq!(
            list_files_with(t.path(), false, &c, &back),
            ["app/_fast.c", "app/a.c", "app/notes.c", "vendor/llhttp/p.c"]
        );
    }

    fn node_shape(extra_map: &str) -> tempfile::TempDir {
        let t = tempfile::tempdir().unwrap();
        for (f, text) in [
            (
                "src/reg.ts",
                "const L: any[] = [];\nexport function on(f: any) {\n  L.push(f);\n}\n",
            ),
            ("src/util.ts", "export const n = 1;\n"),
            (
                "lib/node/chunk-AB12.js",
                "const L = [];\nfunction on(f) { L.push(f); }\nmodule.exports = { on };\n//# sourceMappingURL=chunk-AB12.js.map\n",
            ),
            ("lib/node/chunk-AB12.js.map", extra_map),
            (
                "lib/node/index.d.ts",
                "export declare class A { x: number[]; }\n",
            ),
            (
                "lib/node/index.d.mts",
                "export declare class B { y: number[]; }\n",
            ),
            ("lib/plain.js", "const Q = [];\nmodule.exports = Q;\n"),
        ] {
            let p = t.path().join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        t
    }

    #[test]
    fn node_census_reads_sources_not_declarations_or_their_bundles() {
        let t = node_shape(
            r#"{"version":3,"sources":["../../src/reg.ts","../../src/util.ts"],"mappings":""}"#,
        );
        let read: Vec<String> = source_files(t.path(), Runtime::Node)
            .into_iter()
            .map(|f| f.rel)
            .collect();
        // Before: the bundle and both declaration files were read too (5 files for 2 sources).
        assert_eq!(read, ["lib/plain.js", "src/reg.ts", "src/util.ts"]);
        assert!(is_declaration_file("a/b.D.CTS") && !is_declaration_file("a/d.ts.js"));
        let in_scope = |_: &str| false;
        let defaults = Defaults::for_project(t.path(), &[]);
        // Out-of-scope counting leaves them out too: the plain file and both sources count.
        assert_eq!(
            count_out_of_scope(t.path(), Runtime::Node, &in_scope, &defaults),
            3
        );
    }

    #[test]
    fn a_bundle_whose_map_names_no_project_file_is_still_read() {
        // A map of sources that are not in the project, an unreadable map and an inline map.
        let t =
            node_shape(r#"{"version":3,"sources":["../../node_modules/x/y.ts"],"mappings":""}"#);
        let read: Vec<String> = source_files(t.path(), Runtime::Node)
            .into_iter()
            .map(|f| f.rel)
            .collect();
        assert!(
            read.contains(&"lib/node/chunk-AB12.js".to_owned()),
            "{read:?}"
        );
        assert!(
            !read
                .iter()
                .any(|r| r.ends_with(".d.ts") || r.ends_with(".d.mts"))
        );
        fs::write(t.path().join("lib/node/chunk-AB12.js.map"), "not json").unwrap();
        assert!(!is_built_from_sources(t.path(), "lib/node/chunk-AB12.js"));
        fs::write(
            t.path().join("lib/node/chunk-AB12.js"),
            "x();\n//# sourceMappingURL=data:application/json;base64,e30=\n",
        )
        .unwrap();
        assert!(!is_built_from_sources(t.path(), "lib/node/chunk-AB12.js"));
        // A sourceRoot is honoured.
        fs::write(
            t.path().join("lib/node/chunk-AB12.js"),
            "x();\n//# sourceMappingURL=chunk-AB12.js.map\n",
        )
        .unwrap();
        fs::write(
            t.path().join("lib/node/chunk-AB12.js.map"),
            r#"{"version":3,"sourceRoot":"../..","sources":["src/reg.ts"],"mappings":""}"#,
        )
        .unwrap();
        assert!(is_built_from_sources(t.path(), "lib/node/chunk-AB12.js"));
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
