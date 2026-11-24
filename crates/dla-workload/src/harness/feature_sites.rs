//! Rust: what the features of a target's crate gate.
//!
//! [`feature_gate`](super::targets::feature_gate) reads the `mod` declarations from the file of
//! a target up to the crate root. That misses a crate whose features gate code inside
//! functions and on items (`#[cfg(feature = "x")] fn ..`, a block, a field, a `use`), with no
//! `mod` gated: a harness built without the feature then runs none of that code. This reader
//! counts, per feature that the crate's `default` list does not turn on, the sites in the
//! `src` of the crate, per file, and says which of them a harness build leaves off.
//!
//! A site is a `#[cfg(..)]`, `#![cfg(..)]`, `#[cfg_attr(..)]` (its condition, the part before
//! the first comma) or a `cfg!(..)` whose condition names `feature = "x"`, also inside
//! `all(..)`, `any(..)` and `not(..)`: code under `not(feature = "x")` runs only without the
//! feature, so it is a site of `x` too. As in `feature_gate`, a platform or target gate beside
//! the feature (`all(feature = "io", unix)`) does not hide the site. A condition that names
//! `test` is not counted: no driver build has it. A line comment, so a doc example, is not
//! read. The reader does not see a `#[path = ".."]` module, a `cfg_x! { .. }` macro wrapper
//! (see `feature_gate` for the gated `mod` it reads), `build.rs` and the tests, benches and
//! examples directories.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use dla_core::features::{FeatureGap, HarnessFeatures};

use super::targets::{Target, feature_gate};

/// The most files of one crate read.
const MAX_FILES: usize = 3000;

/// What the features of one crate gate.
#[derive(Debug, Clone, Default)]
pub struct CrateFeatures {
    /// The crate's directory (project-relative; empty for the project root).
    pub dir: PathBuf,
    /// Declared feature -> the entries it turns on.
    graph: BTreeMap<String, Vec<String>>,
    /// The features the `default` list turns on, directly or through other features.
    pub default_on: BTreeSet<String>,
    /// Feature -> file (project-relative, `/` separated) -> gated sites, for the declared
    /// features that have at least one.
    pub sites: BTreeMap<String, BTreeMap<String, usize>>,
}

/// The directory (project-relative) of the nearest `Cargo.toml` at or above `file`.
fn crate_dir(project: &Path, file: &str) -> Option<PathBuf> {
    let mut dir = Path::new(file).parent()?.to_path_buf();
    loop {
        if project.join(&dir).join("Cargo.toml").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// The features entries that name another feature of the crate: not `dep:x`, not `x/y`.
fn local_entry(e: &str) -> bool {
    !e.starts_with("dep:") && !e.contains('/')
}

/// Every feature reachable from `start` through the declared features.
fn closure<'a>(
    graph: &BTreeMap<String, Vec<String>>,
    start: impl Iterator<Item = &'a str>,
) -> BTreeSet<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = start
        .filter(|s| local_entry(s))
        .map(str::to_owned)
        .collect();
    while let Some(f) = stack.pop() {
        if !seen.insert(f.clone()) {
            continue;
        }
        for e in graph.get(&f).into_iter().flatten() {
            if local_entry(e) {
                stack.push(e.clone());
            }
        }
    }
    seen
}

/// The text of `cond(` .. `)` after the opening parenthesis at `from`, up to the matching one
/// (`None` when the file ends first). A string literal is skipped over.
fn balanced(text: &str, from: usize) -> Option<&str> {
    let (mut depth, mut in_str, mut esc) = (1i32, false, false);
    for (i, c) in text[from..].char_indices() {
        if in_str {
            match (esc, c) {
                (true, _) => esc = false,
                (false, '\\') => esc = true,
                (false, '"') => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[from..from + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// The part of a `cfg_attr(..)` before its first top-level comma: the condition.
fn first_arg(args: &str) -> &str {
    let (mut depth, mut in_str) = (0i32, false);
    for (i, c) in args.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '(' if !in_str => depth += 1,
            ')' if !in_str => depth -= 1,
            ',' if !in_str && depth == 0 => return &args[..i],
            _ => {}
        }
    }
    args
}

/// The gated sites of one source text, per feature name.
pub fn count_sites(text: &str) -> BTreeMap<String, usize> {
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    let (Ok(open), Ok(feature), Ok(test), Ok(string)) = (
        regex::Regex::new(r"#!?\[\s*(cfg_attr|cfg)\s*\(|\bcfg!\s*\("),
        regex::Regex::new(r#"feature\s*=\s*"([^"]+)""#),
        regex::Regex::new(r"\btest\b"),
        regex::Regex::new(r#""[^"]*""#),
    ) else {
        return out;
    };
    // Whole-line comments (also doc comments and their examples) are not code.
    let code: String = text
        .lines()
        .map(|l| {
            if l.trim_start().starts_with("//") {
                ""
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    for m in open.captures_iter(&code) {
        let Some(whole) = m.get(0) else { continue };
        let Some(args) = balanced(&code, whole.end()) else {
            continue;
        };
        let cond = if m.get(1).is_some_and(|g| g.as_str() == "cfg_attr") {
            first_arg(args)
        } else {
            args
        };
        // `test` is a word of the condition, not of a string in it.
        if test.is_match(&string.replace_all(cond, "\"\"")) {
            continue;
        }
        let names: BTreeSet<&str> = feature
            .captures_iter(cond)
            .filter_map(|c| c.get(1).map(|g| g.as_str()))
            .collect();
        for n in names {
            *out.entry(n.to_owned()).or_insert(0) += 1;
        }
    }
    out
}

fn rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(root) else { return };
    let mut entries: Vec<PathBuf> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if out.len() >= MAX_FILES {
            return;
        }
        if p.is_dir() {
            if !matches!(
                p.file_name().and_then(|n| n.to_str()),
                Some("target" | ".git")
            ) {
                rust_files(&p, out);
            }
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

impl CrateFeatures {
    /// Reads the crate of `file` (project-relative): `Cargo.toml` and the `src` directory.
    /// `None` when no `Cargo.toml` is above the file or it does not parse.
    pub fn read(project: &Path, file: &str) -> Option<Self> {
        let dir = crate_dir(project, file)?;
        let text = fs::read_to_string(project.join(&dir).join("Cargo.toml")).ok()?;
        let doc: toml::Table = toml::from_str(&text).ok()?;
        let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
        if let Some(t) = doc.get("features").and_then(toml::Value::as_table) {
            for (k, v) in t {
                let entries = v
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.as_str().map(str::to_owned))
                    .collect();
                graph.insert(k.clone(), entries);
            }
        }
        // An optional dependency is a feature of its own name unless some feature names it
        // as `dep:name`.
        let named: BTreeSet<String> = graph
            .values()
            .flatten()
            .filter_map(|e| e.strip_prefix("dep:").map(str::to_owned))
            .collect();
        let mut deps: Vec<&toml::Table> = Vec::new();
        if let Some(t) = doc.get("dependencies").and_then(toml::Value::as_table) {
            deps.push(t);
        }
        for t in doc
            .get("target")
            .and_then(toml::Value::as_table)
            .into_iter()
            .flat_map(toml::Table::values)
        {
            if let Some(d) = t.get("dependencies").and_then(toml::Value::as_table) {
                deps.push(d);
            }
        }
        for (name, v) in deps.into_iter().flatten() {
            let optional = v
                .as_table()
                .and_then(|t| t.get("optional"))
                .and_then(toml::Value::as_bool)
                == Some(true);
            if optional && !named.contains(name) {
                graph.entry(name.clone()).or_default();
            }
        }
        let default_on = closure(&graph, std::iter::once("default"));
        let mut files = Vec::new();
        rust_files(&project.join(&dir).join("src"), &mut files);
        let mut sites: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for f in files {
            let Ok(src) = fs::read_to_string(&f) else {
                continue;
            };
            let rel = f
                .strip_prefix(project)
                .unwrap_or(&f)
                .to_string_lossy()
                .replace('\\', "/");
            for (feat, n) in count_sites(&src) {
                // A feature the crate does not declare is not one a build can turn on.
                if graph.contains_key(&feat) {
                    sites.entry(feat).or_default().insert(rel.clone(), n);
                }
            }
        }
        Some(Self {
            dir,
            graph,
            default_on,
            sites,
        })
    }

    /// The features with gated sites that the `default` list does not turn on, with the sites
    /// per file.
    pub fn off_by_default(&self) -> Vec<(&str, &BTreeMap<String, usize>)> {
        self.sites
            .iter()
            .filter(|(f, _)| !self.default_on.contains(*f))
            .map(|(f, files)| (f.as_str(), files))
            .collect()
    }

    /// The features a build turns on: the default ones (unless `no_default`) and `features`,
    /// with every feature those turn on.
    pub fn enabled(&self, features: &[String], no_default: bool) -> BTreeSet<String> {
        let mut start: Vec<&str> = features.iter().map(String::as_str).collect();
        if !no_default {
            start.push("default");
        }
        closure(&self.graph, start.into_iter())
    }

    /// The features with gated sites that a build with `enabled` leaves off, for the target in
    /// `file`; `by_module` are the features that gate the module of the file (`feature_gate`).
    pub fn gaps(
        &self,
        file: &str,
        enabled: &BTreeSet<String>,
        by_module: &[String],
    ) -> Vec<FeatureGap> {
        let entry = |f: &str| {
            let base = self.dir.join("src");
            [base.join("lib.rs"), base.join("main.rs")]
                .iter()
                .any(|e| e.to_string_lossy().replace('\\', "/") == f)
        };
        self.sites
            .iter()
            .filter(|(f, _)| !enabled.contains(*f))
            .map(|(f, files)| FeatureGap {
                feature: f.clone(),
                file: file.to_owned(),
                in_file: files.get(file).copied().unwrap_or(0),
                in_crate: files.values().sum(),
                in_entry: files
                    .iter()
                    .filter(|(p, _)| entry(p))
                    .map(|(_, n)| *n)
                    .sum(),
                by_module: by_module.contains(f),
                default_on: self.default_on.contains(f),
            })
            .collect()
    }
}

/// Sets the feature gaps of each Rust target (the features of its crate with gated sites that
/// a build with `features` leaves off) and returns what the harness records beside its
/// manifest. `entry`: the harness is an entry-path one, which `--entry` generates again.
pub fn add_feature_gaps(
    project: &Path,
    targets: &mut [Target],
    features: &[String],
    no_default: bool,
    entry: bool,
) -> HarnessFeatures {
    let mut cache: BTreeMap<PathBuf, Option<CrateFeatures>> = BTreeMap::new();
    for t in targets.iter_mut() {
        let file = t.site.rsplit_once(':').map_or(t.site.as_str(), |(f, _)| f);
        if !file.ends_with(".rs") {
            continue;
        }
        let Some(dir) = crate_dir(project, file) else {
            continue;
        };
        let cf = cache
            .entry(dir)
            .or_insert_with(|| CrateFeatures::read(project, file));
        let Some(cf) = cf else { continue };
        let enabled = cf.enabled(features, no_default);
        let by_module = feature_gate(project, file);
        t.feature_gaps = cf.gaps(file, &enabled, &by_module);
    }
    let mut out = HarnessFeatures::default();
    let mut on: Vec<String> = features.to_vec();
    for t in targets.iter() {
        if t.feature_gaps.is_empty() {
            continue;
        }
        for g in &t.feature_gaps {
            if !on.contains(&g.feature) {
                on.push(g.feature.clone());
            }
        }
        out.gaps.insert(t.id.to_string(), t.feature_gaps.clone());
    }
    let ids: Vec<String> = targets.iter().map(|t| t.id.to_string()).collect();
    out.command = format!(
        "dla harness gen --lang rust --driver{} --features {} {}",
        if no_default {
            " --no-default-features"
        } else {
            ""
        },
        on.join(","),
        if entry {
            format!("--entry {}", ids.join(","))
        } else {
            ids.iter()
                .map(|i| format!("--probe {i}"))
                .collect::<Vec<_>>()
                .join(" ")
        }
    );
    for t in targets.iter_mut() {
        t.feature_command.clone_from(&out.command);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (rel, text) in files {
            let p = dir.path().join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        dir
    }

    const SPAN: &str = "pub struct Span;\n\nimpl Span {\n    pub fn new() -> Self {\n        #[cfg(feature = \"enable\")]\n        {\n            sink::open();\n        }\n        Span\n    }\n}\n\nimpl Drop for Span {\n    fn drop(&mut self) {\n        #[cfg(feature = \"enable\")]\n        sink::close();\n    }\n}\n\n#[cfg(feature = \"enable\")]\nmod sink {\n    pub fn open() {}\n    pub fn close() {}\n}\n";

    #[test]
    fn a_feature_off_by_default_gives_its_gated_sites_per_file() {
        let dir = tree(&[
            (
                "Cargo.toml",
                "[package]\nname = \"t\"\n\n[features]\nenable = [\"macros/enable\"]\n",
            ),
            ("src/lib.rs", "mod span;\npub use span::Span;\n"),
            ("src/span.rs", SPAN),
            (
                "src/other.rs",
                "pub fn f() {\n    #[cfg(feature = \"enable\")]\n    {}\n}\n",
            ),
        ]);
        let cf = CrateFeatures::read(dir.path(), "src/span.rs").unwrap();
        let off = cf.off_by_default();
        assert_eq!(off.len(), 1, "{off:?}");
        assert_eq!(off[0].0, "enable");
        assert_eq!(off[0].1["src/span.rs"], 3, "{off:?}");
        assert_eq!(off[0].1["src/other.rs"], 1, "{off:?}");
        let gaps = cf.gaps("src/span.rs", &cf.enabled(&[], false), &[]);
        assert_eq!(gaps.len(), 1);
        assert_eq!((gaps[0].in_file, gaps[0].in_crate), (3, 4), "{gaps:?}");
        assert!(!gaps[0].by_module && !gaps[0].default_on);
        // With the feature on, nothing is left off.
        let on = cf.enabled(&["enable".to_owned()], false);
        assert!(cf.gaps("src/span.rs", &on, &[]).is_empty());
    }

    #[test]
    fn a_default_feature_and_one_a_default_feature_turns_on_give_nothing() {
        let dir = tree(&[
            (
                "Cargo.toml",
                "[package]\nname = \"t\"\n\n[features]\ndefault = [\"std\"]\nstd = [\"alloc\"]\nalloc = []\nextra = []\n",
            ),
            (
                "src/lib.rs",
                "#[cfg(feature = \"std\")]\nfn a() {}\n#[cfg(feature = \"alloc\")]\nfn b() {}\n#[cfg(feature = \"extra\")]\nfn c() {}\n",
            ),
        ]);
        let cf = CrateFeatures::read(dir.path(), "src/lib.rs").unwrap();
        let off: Vec<&str> = cf.off_by_default().iter().map(|(f, _)| *f).collect();
        assert_eq!(off, ["extra"]);
        // `--no-default-features` turns the defaults off for the build, not for the rule.
        let g = cf.gaps("src/lib.rs", &cf.enabled(&[], true), &[]);
        let names: Vec<(&str, bool)> = g
            .iter()
            .map(|g| (g.feature.as_str(), g.default_on))
            .collect();
        assert_eq!(names, [("alloc", true), ("extra", false), ("std", true)]);
        // A feature asked for turns on what it names.
        let g = cf.gaps("src/lib.rs", &cf.enabled(&["std".to_owned()], true), &[]);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].feature, "extra");
    }

    #[test]
    fn not_feature_cfg_attr_and_the_cfg_macro_count_and_test_and_comments_do_not() {
        let src = "#[cfg(not(feature = \"x\"))]\nfn a() {}\n\n#[cfg_attr(feature = \"x\", derive(Debug))]\nstruct S;\n\nfn b() -> bool {\n    cfg!(feature = \"x\")\n}\n\n#[cfg(all(feature = \"x\", unix))]\nfn c() {}\n\n#[cfg(any(feature = \"x\", feature = \"y\"))]\nfn d() {}\n\n#[cfg(all(test, feature = \"x\"))]\nmod tests {}\n\n/// #[cfg(feature = \"x\")]\n// #[cfg(feature = \"x\")]\n#[cfg_attr(docsrs, doc(cfg(feature = \"x\")))]\nfn e() {}\n";
        let c = count_sites(src);
        assert_eq!(c["x"], 5, "{c:?}");
        assert_eq!(c["y"], 1, "{c:?}");
    }

    #[test]
    fn a_condition_over_several_lines_and_a_feature_the_crate_does_not_declare() {
        let dir = tree(&[
            (
                "Cargo.toml",
                "[package]\nname = \"t\"\n\n[dependencies]\nserde = { version = \"1\", optional = true }\n\n[features]\nreal = []\n",
            ),
            (
                "src/lib.rs",
                "#[cfg(all(\n    feature = \"real\",\n    unix\n))]\nfn a() {}\n#[cfg(feature = \"serde\")]\nfn b() {}\n#[cfg(feature = \"typo\")]\nfn c() {}\n",
            ),
        ]);
        let cf = CrateFeatures::read(dir.path(), "src/lib.rs").unwrap();
        let off: Vec<&str> = cf.off_by_default().iter().map(|(f, _)| *f).collect();
        // The optional dependency is a feature of its name; the typo is no feature.
        assert_eq!(off, ["real", "serde"]);
    }

    #[test]
    fn a_gated_module_is_named_as_before_and_marks_the_gap() {
        let dir = tree(&[
            (
                "Cargo.toml",
                "[package]\nname = \"t\"\n\n[features]\nrt = []\n",
            ),
            ("src/lib.rs", "#[cfg(feature = \"rt\")]\nmod task;\n"),
            ("src/task.rs", "pub static A: u8 = 0;\n"),
        ]);
        assert_eq!(feature_gate(dir.path(), "src/task.rs"), ["rt"]);
        let cf = CrateFeatures::read(dir.path(), "src/task.rs").unwrap();
        let by_module = feature_gate(dir.path(), "src/task.rs");
        let g = cf.gaps("src/task.rs", &cf.enabled(&[], false), &by_module);
        assert_eq!(g.len(), 1);
        // The declaration is a gated site of the crate, in its entry file.
        assert!(g[0].by_module);
        assert_eq!(
            (g[0].in_file, g[0].in_crate, g[0].in_entry),
            (0, 1, 1),
            "{g:?}"
        );
    }

    #[test]
    fn the_crate_of_a_file_is_the_nearest_cargo_toml() {
        let dir = tree(&[
            ("Cargo.toml", "[workspace]\nmembers = [\"m\"]\n"),
            (
                "m/Cargo.toml",
                "[package]\nname = \"m\"\n\n[features]\nf = []\n",
            ),
            ("m/src/lib.rs", "#[cfg(feature = \"f\")]\nfn a() {}\n"),
            ("m/src/deep/x.rs", "#[cfg(feature = \"f\")]\nfn b() {}\n"),
        ]);
        let cf = CrateFeatures::read(dir.path(), "m/src/deep/x.rs").unwrap();
        assert_eq!(cf.dir, Path::new("m"));
        assert_eq!(cf.sites["f"]["m/src/lib.rs"], 1);
        assert_eq!(cf.sites["f"]["m/src/deep/x.rs"], 1);
        let g = cf.gaps("m/src/deep/x.rs", &cf.enabled(&[], false), &[]);
        assert_eq!((g[0].in_file, g[0].in_crate, g[0].in_entry), (1, 2, 1));
    }
}
