//! DLA-owned static rules (backend `dla`): AST checks over the tree-sitter parsers of
//! `dla-census` for leak shapes that no off-the-shelf rule covers.
//!
//! Entry point: [`run`]. Rules are per runtime (`jvm`, `node`, `go`, `rust`, `python`,
//! `native`); each module lists its rules in a `RULES` table (id, suggested archetype,
//! precision prior) that must agree with the `backend = "dla"` rows of `data/detectors/<lang>.toml`
//! (a unit test checks this). The rules look at the whole project at once, because
//! "no removal anywhere" is a project-wide claim, and favour precision: a hit says
//! "this shape is present and nothing in the project undoes it", which is a candidate
//! for the dynamic stages, not a verdict.

use std::path::Path;

use dla_census::parse::{Lang, parse, tree_sitter::Tree};
use dla_core::{error::Refusal, runtime::Runtime, static_hit::StaticHit, toolctx::ToolCtx};

pub mod ast;
mod chain;
mod go;
mod jvm;
mod native;
mod node;
mod python;
mod rust;

/// Backend name used in detector tables, coverage and candidate origins.
pub const BACKEND: &str = "dla";

/// Directories never scanned.
const SKIP_DIRS: [&str; 17] = [
    ".git",
    ".dla",
    "target",
    "build",
    "node_modules",
    "venv",
    ".venv",
    "out",
    "dist",
    "__pycache__",
    "vendor",
    "site-packages",
    ".gradle",
    ".idea",
    ".tox",
    "third_party",
    "bench-results",
];

/// Largest source file parsed, in bytes.
const MAX_FILE_BYTES: u64 = 1_500_000;

/// One parsed source file.
pub struct SrcFile {
    /// Project-relative path with `/` separators.
    pub rel: String,
    /// Grammar.
    pub lang: Lang,
    /// File content.
    pub text: String,
    /// Syntax tree.
    pub tree: Tree,
}

/// Static description of one rule, mirrored by the detector fragment files.
#[derive(Debug, Clone, Copy)]
pub struct RuleInfo {
    /// Rule name; the detector id is `dla/<rule>`.
    pub rule: &'static str,
    /// Suggested archetype tag.
    pub archetype: &'static str,
    /// Prior probability that a hit is a real leak.
    pub prior: f64,
}

/// Every rule of a runtime.
pub fn rules(rt: Runtime) -> &'static [RuleInfo] {
    match rt {
        Runtime::Jvm => jvm::RULES,
        Runtime::Node => node::RULES,
        Runtime::Go => go::RULES,
        Runtime::Rust => rust::RULES,
        Runtime::Python => python::RULES,
        Runtime::Native => native::RULES,
    }
}

pub(crate) fn hit(rule: &str, file: &SrcFile, line: u32, message: impl Into<String>) -> StaticHit {
    StaticHit {
        backend: BACKEND.to_owned(),
        rule: rule.to_owned(),
        file: file.rel.clone(),
        line,
        message: message.into(),
    }
}

/// True for test sources, which the rules do not look at.
pub(crate) fn is_test_path(rel: &str) -> bool {
    let lower = rel.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or("");
    let stem = name.split('.').next().unwrap_or("");
    lower.split('/').any(|d| {
        matches!(
            d,
            "test" | "tests" | "__tests__" | "testdata" | "spec" | "specs"
        )
    }) || name.ends_with("_test.go")
        || name.contains(".test.")
        || name.contains(".spec.")
        || name == "conftest.py"
        || stem.starts_with("test_")
        || stem.ends_with("_test")
        || stem.ends_with("test") && stem.len() > 4 && name.ends_with(".java")
        || stem.ends_with("tests") && name.ends_with(".java")
        || stem.ends_with("test") && name.ends_with(".kt")
}

fn collect(
    dir: &Path,
    root: &Path,
    langs: &[Lang],
    inside: &dyn Fn(&str) -> bool,
    out: &mut Vec<SrcFile>,
) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd.filter_map(Result::ok).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for e in entries {
        let path = e.path();
        let Ok(ft) = e.file_type() else { continue };
        let name = e.file_name().to_string_lossy().into_owned();
        if ft.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            collect(&path, root, langs, inside, out);
        } else if ft.is_file() {
            let Some(lang) = Lang::from_path(&path) else {
                continue;
            };
            if !langs.contains(&lang) {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            // Build logic (Gradle scripts, buildSrc) is not part of the measured process: not
            // even parsed.
            if dla_core::paths::is_build_logic(&rel) || is_test_path(&rel) || !inside(&rel) {
                continue;
            }
            if e.metadata().map(|m| m.len()).unwrap_or(0) > MAX_FILE_BYTES {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(tree) = parse(lang, &text) {
                out.push(SrcFile {
                    rel,
                    lang,
                    text,
                    tree,
                });
            }
        }
    }
}

/// Runs the DLA rules of `rt` over the sources under `ctx.project`.
///
/// Test sources and build output are skipped. The hits are sorted by file, line and rule
/// and never repeat a `(rule, file, line)`.
pub fn run(ctx: &ToolCtx, rt: Runtime) -> Result<Vec<StaticHit>, Refusal> {
    let mut files = Vec::new();
    let scope = ctx.scope()?;
    let inside = |rel: &str| scope.as_ref().is_none_or(|s| s.includes(rel));
    collect(
        &ctx.project,
        &ctx.project,
        &Lang::of_runtime(rt),
        &inside,
        &mut files,
    );
    Ok(analyze(&files, rt))
}

/// Runs the rules of `rt` over already parsed files.
pub fn analyze(files: &[SrcFile], rt: Runtime) -> Vec<StaticHit> {
    let mut hits = match rt {
        Runtime::Jvm => jvm::check(files),
        Runtime::Node => node::check(files),
        Runtime::Go => go::check(files),
        Runtime::Rust => rust::check(files),
        Runtime::Python => python::check(files),
        Runtime::Native => native::check(files),
    };
    hits.sort_by(|a, b| (&a.file, a.line, &a.rule).cmp(&(&b.file, b.line, &b.rule)));
    hits.dedup_by(|a, b| a.file == b.file && a.line == b.line && a.rule == b.rule);
    hits
}

/// Parses `(path, source)` pairs; the path decides the grammar. For tests.
#[cfg(test)]
pub(crate) fn parse_sources(sources: &[(&str, &str)]) -> Vec<SrcFile> {
    sources
        .iter()
        .map(|(rel, text)| {
            let lang = Lang::from_path(Path::new(rel)).expect("known extension");
            SrcFile {
                rel: (*rel).to_owned(),
                lang,
                text: (*text).to_owned(),
                tree: parse(lang, text).expect("parse"),
            }
        })
        .collect()
}

/// Hits of one rule, as `file:line` strings. For tests.
#[cfg(test)]
pub(crate) fn sites(hits: &[StaticHit], rule: &str) -> Vec<String> {
    hits.iter()
        .filter(|h| h.rule == rule)
        .map(StaticHit::site)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLES: [(Runtime, &str, &str); 6] = [
        (
            Runtime::Jvm,
            "jvm",
            include_str!("../../../../data/detectors/jvm.toml"),
        ),
        (
            Runtime::Go,
            "go",
            include_str!("../../../../data/detectors/go.toml"),
        ),
        (
            Runtime::Rust,
            "rust",
            include_str!("../../../../data/detectors/rust.toml"),
        ),
        (
            Runtime::Node,
            "node",
            include_str!("../../../../data/detectors/node.toml"),
        ),
        (
            Runtime::Python,
            "python",
            include_str!("../../../../data/detectors/python.toml"),
        ),
        (
            Runtime::Native,
            "native",
            include_str!("../../../../data/detectors/native.toml"),
        ),
    ];

    /// Build logic (Gradle scripts, `buildSrc`) is not parsed at all, so no rule can fire there.
    #[test]
    fn build_logic_is_not_collected() {
        let d = tempfile::tempdir().unwrap();
        for (rel, body) in [
            ("build.gradle.kts", "val extra = mutableListOf<String>()\n"),
            (
                "docker.gradle.kts",
                "val props = mutableMapOf<String, String>()\n",
            ),
            (
                "buildSrc/src/main/kotlin/Conventions.kt",
                "object Conventions { val registry = mutableMapOf<String, String>() }\n",
            ),
            (
                "src/main/kotlin/app/Cache.kt",
                "object Cache { val entries = mutableMapOf<String, String>() }\n",
            ),
            (
                "scripts/tool.main.kts",
                "val seen = mutableSetOf<String>()\n",
            ),
        ] {
            let p = d.path().join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, body).unwrap();
        }
        let mut out = Vec::new();
        collect(
            d.path(),
            d.path(),
            &Lang::of_runtime(Runtime::Jvm),
            &|_| true,
            &mut out,
        );
        let mut rels: Vec<&str> = out.iter().map(|f| f.rel.as_str()).collect();
        rels.sort_unstable();
        assert_eq!(
            rels,
            ["scripts/tool.main.kts", "src/main/kotlin/app/Cache.kt"]
        );
    }

    /// The shipped detector tables hold exactly one `dla` row per rule here,
    /// with the same tag, prior and verified flag.
    #[test]
    fn detector_tables_match_the_rule_tables() {
        for (rt, name, text) in TABLES {
            let file = dla_core::detectors::DetectorFile::parse(
                text,
                &format!("data/detectors/{name}.toml"),
            )
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
            let rows: Vec<_> = file
                .detector
                .iter()
                .filter(|d| d.backend == "dla")
                .collect();
            let table = rules(rt);
            assert_eq!(rows.len(), table.len(), "{name}: dla row count");
            for info in table {
                let id = format!("dla/{}", info.rule);
                let d = rows
                    .iter()
                    .find(|d| d.id == id)
                    .unwrap_or_else(|| panic!("{name}: no row for {id}"));
                assert_eq!(d.backend, "dla");
                assert_eq!(d.rule, info.rule);
                assert_eq!(d.suggested_archetype, info.archetype, "{id}");
                assert!((d.precision_prior - info.prior).abs() < 1e-9, "{id}");
                assert!(d.verified, "{id} must be verified");
                assert!(info.rule.starts_with(&format!("{name}-")), "{id} naming");
            }
        }
    }

    #[test]
    fn test_paths_are_skipped() {
        for p in [
            "src/test/java/FooTest.java",
            "pkg/a_test.go",
            "tests/test_a.py",
            "src/a.test.js",
            "src/FooTests.java",
        ] {
            assert!(is_test_path(p), "{p}");
        }
        for p in [
            "src/main/java/Foo.java",
            "pkg/a.go",
            "src/latest.py",
            "src/contest.ts",
        ] {
            assert!(!is_test_path(p), "{p}");
        }
    }

    #[test]
    fn run_walks_a_directory_and_skips_build_output() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("node_modules/x")).unwrap();
        let code = "setInterval(() => {}, 1000);\nfunction f() { setInterval(() => {}, 5); }\n";
        std::fs::write(root.join("src/a.js"), code).unwrap();
        std::fs::write(root.join("node_modules/x/b.js"), code).unwrap();
        let files = {
            let mut v = Vec::new();
            collect(
                root,
                root,
                &Lang::of_runtime(Runtime::Node),
                &|_| true,
                &mut v,
            );
            v
        };
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].rel, "src/a.js");
        let hits = analyze(&files, Runtime::Node);
        assert!(!hits.is_empty());
        assert!(
            hits.iter()
                .all(|h| h.file == "src/a.js" && h.backend == "dla")
        );
    }
}
