//! `dla-bench validate`: every rule of `bench/FIXTURES.md`.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use regex::Regex;
use serde_json::Value;

use crate::{
    Res,
    fixture::{fixture_dirs, load_expected},
    model::{Category, Expected, LEAK_CAUSES, NON_LEAK_CAUSES},
    schema_cmd,
};

const SOURCE_EXTENSIONS: [&str; 11] = [
    "java", "kt", "go", "rs", "js", "mjs", "cjs", "ts", "py", "c", "h",
];
const BUILD_DIRS: [&str; 4] = ["target", "node_modules", "__pycache__", "build"];
const BUILD_EXTENSIONS: [&str; 6] = ["class", "o", "pyc", "so", "dll", "exe"];

fn hint_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(leak\w*|todo|fixme|fix|bug|unbounded|bounded|never (?:freed|removed|cleared|closed|stopped|released))\b",
        )
        .expect("valid regex")
    })
}

fn site_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(.+):([0-9]+)$").expect("valid regex"))
}

fn alt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(.+):([0-9]+)(?:-([0-9]+))?$").expect("valid regex"))
}

/// The files of a fixture's `project/`: relative path with `/` separators and content.
struct Project {
    files: BTreeMap<String, Vec<u8>>,
}

impl Project {
    fn load(dir: &Path) -> Res<Project> {
        let mut files = BTreeMap::new();
        walk(dir, dir, &mut files)?;
        Ok(Project { files })
    }

    fn line_count(&self, path: &str) -> Option<u32> {
        let b = self.files.get(path)?;
        let n = b.iter().filter(|&&c| c == b'\n').count() + usize::from(!b.ends_with(b"\n"));
        u32::try_from(n).ok()
    }
}

fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Res<()> {
    let rd = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(root, &p, out)?;
        } else {
            let rel = p
                .strip_prefix(root)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            let data = fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            out.insert(rel, data);
        }
    }
    Ok(())
}

/// Checks `path:line` (or a range) against the project; `Err` carries the problem.
fn check_location(project: &Project, path: &str, lo: u32, hi: u32) -> Result<(), String> {
    if path.starts_with('/') || path.contains('\\') || path.split('/').any(|c| c == "..") {
        return Err(format!(
            "{path}: must be a project-relative path with / separators"
        ));
    }
    let Some(count) = project.line_count(path) else {
        return Err(format!("{path}: no such file in project/"));
    };
    if lo == 0 || lo > hi || hi > count {
        return Err(format!(
            "{path}:{lo}-{hi}: outside the file ({count} lines)"
        ));
    }
    Ok(())
}

/// Validates every fixture under `fixtures`; the problems as `fixture: problem` lines.
pub fn validate_dir(fixtures: &Path, schemas: &Path) -> Res<(usize, Vec<String>)> {
    let dirs = fixture_dirs(fixtures)?;
    let schema_text = fs::read_to_string(schemas.join("expected.schema.json"))
        .unwrap_or_else(|_| schema_cmd::expected_schema());
    let schema: Value =
        serde_json::from_str(&schema_text).map_err(|e| format!("expected.schema.json: {e}"))?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| format!("expected.schema.json does not compile: {e}"))?;
    let mut problems = Vec::new();
    for d in &dirs {
        let name = d
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        for p in validate_fixture(fixtures, d, &name, &validator) {
            problems.push(format!("{name}: {p}"));
        }
    }
    Ok((dirs.len(), problems))
}

fn validate_fixture(
    fixtures: &Path,
    dir: &Path,
    name: &str,
    validator: &jsonschema::Validator,
) -> Vec<String> {
    let mut out = Vec::new();
    if !dir.join("README.md").is_file() {
        out.push("README.md is missing".to_owned());
    }
    let project = if dir.join("project").is_dir() {
        match Project::load(&dir.join("project")) {
            Ok(p) if p.files.is_empty() => {
                out.push("project/ is empty".to_owned());
                None
            }
            Ok(p) => Some(p),
            Err(e) => {
                out.push(e);
                None
            }
        }
    } else {
        out.push("project/ is missing".to_owned());
        None
    };
    if let Some(p) = &project {
        check_project(p, &mut out);
    }
    let toml_path = dir.join("expected.toml");
    let Ok(text) = fs::read_to_string(&toml_path) else {
        out.push("expected.toml is missing or unreadable".to_owned());
        return out;
    };
    let value: toml::Value = match toml::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            out.push(format!("expected.toml: {e}"));
            return out;
        }
    };
    let inst = match serde_json::to_value(&value) {
        Ok(v) => v,
        Err(e) => {
            out.push(format!("expected.toml: {e}"));
            return out;
        }
    };
    let schema_errors: Vec<String> = validator
        .iter_errors(&inst)
        .map(|e| format!("expected.toml schema: {} at {}", e, e.instance_path))
        .collect();
    if !schema_errors.is_empty() {
        out.extend(schema_errors);
        return out;
    }
    let exp: Expected = match toml::from_str(&text) {
        Ok(e) => e,
        Err(e) => {
            out.push(format!("expected.toml: {e}"));
            return out;
        }
    };
    check_semantics(fixtures, name, &exp, project.as_ref(), &mut out);
    out
}

fn check_project(p: &Project, out: &mut Vec<String>) {
    let mut source_files = 0;
    let mut source_lines = 0;
    for (path, data) in &p.files {
        let components: Vec<&str> = path.split('/').collect();
        let ext = path.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
        if components[..components.len() - 1]
            .iter()
            .any(|c| BUILD_DIRS.contains(c))
            || BUILD_EXTENSIONS.contains(&ext)
        {
            out.push(format!(
                "project/{path}: build output must not be committed"
            ));
        }
        if SOURCE_EXTENSIONS.contains(&ext) {
            source_files += 1;
            source_lines += p.line_count(path).unwrap_or(0);
        }
        scan_hints(&path.replace('_', " "), |w| {
            out.push(format!(
                "project/{path}: file name contains hint word {w:?}"
            ));
        });
        if let Ok(text) = std::str::from_utf8(data) {
            for (i, line) in text.lines().enumerate() {
                scan_hints(&line.replace('_', " "), |w| {
                    out.push(format!("project/{path}:{}: hint word {w:?}", i + 1));
                });
            }
        }
    }
    if !(3..=10).contains(&source_files) {
        out.push(format!(
            "project/ has {source_files} source files; the rule is 3 to 10"
        ));
    }
    if !(150..=600).contains(&source_lines) {
        out.push(format!(
            "project/ has {source_lines} source lines; the rule is 150 to 600"
        ));
    }
}

fn scan_hints(text: &str, mut hit: impl FnMut(&str)) {
    for m in hint_re().find_iter(text) {
        hit(&m.as_str().to_lowercase());
    }
}

fn check_semantics(
    fixtures: &Path,
    name: &str,
    exp: &Expected,
    project: Option<&Project>,
    out: &mut Vec<String>,
) {
    if exp.id != name {
        out.push(format!("id {:?} must equal the directory name", exp.id));
    }
    let mut seen = std::collections::BTreeSet::new();
    for l in &exp.leak {
        if !seen.insert(l.id.clone()) {
            out.push(format!("leak id {} is used twice", l.id));
        }
    }
    for n in &exp.non_leak {
        if !seen.insert(n.id.clone()) {
            out.push(format!("non_leak id {} is used twice", n.id));
        }
    }
    for l in &exp.leak {
        if !LEAK_CAUSES.contains(&l.cause.as_str()) {
            out.push(format!(
                "leak {}: cause {:?} must be one of {} (an L-* archetype or UNCLASSIFIED)",
                l.id,
                l.cause,
                LEAK_CAUSES.join(", ")
            ));
        }
    }
    for n in &exp.non_leak {
        if !NON_LEAK_CAUSES.contains(&n.cause.as_str()) {
            out.push(format!(
                "non_leak {}: cause {:?} must be one of {} (an N-* archetype or UNCLASSIFIED)",
                n.id,
                n.cause,
                NON_LEAK_CAUSES.join(", ")
            ));
        }
    }
    for l in &exp.leak {
        let Some(c) = site_re().captures(&l.site) else {
            out.push(format!("leak {}: site {:?} is not path:line", l.id, l.site));
            continue;
        };
        let line: u32 = c[2].parse().unwrap_or(0);
        let (lo, hi) = l.lines.map_or((line, line), |[a, b]| (a, b));
        if !(lo <= line && line <= hi) {
            out.push(format!(
                "leak {}: lines [{lo}, {hi}] must contain the site line {line}",
                l.id
            ));
        }
        if let Some(p) = project {
            if let Err(e) = check_location(p, &c[1], lo.min(line).max(1), hi.max(line)) {
                out.push(format!("leak {}: site {e}", l.id));
            }
            for alt in &l.alt_sites {
                let Some(a) = alt_re().captures(alt) else {
                    out.push(format!(
                        "leak {}: alt_site {alt:?} is not path:line or path:start-end",
                        l.id
                    ));
                    continue;
                };
                let alo: u32 = a[2].parse().unwrap_or(0);
                let ahi: u32 = a
                    .get(3)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(alo);
                if let Err(e) = check_location(p, &a[1], alo, ahi) {
                    out.push(format!("leak {}: alt_site {e}", l.id));
                }
            }
        }
    }
    for n in &exp.non_leak {
        let Some(site) = &n.site else { continue };
        let Some(c) = site_re().captures(site) else {
            out.push(format!("non_leak {}: site {site:?} is not path:line", n.id));
            continue;
        };
        let line: u32 = c[2].parse().unwrap_or(0);
        if let Some(p) = project
            && let Err(e) = check_location(p, &c[1], line, line)
        {
            out.push(format!("non_leak {}: site {e}", n.id));
        }
    }
    if exp.leak.len() > 1 {
        let sum: f64 = exp.leak.iter().map(|l| l.share).sum();
        if (sum - 1.0).abs() > 0.01 {
            out.push(format!("leak shares sum to {sum}, not 1.0 (within 0.01)"));
        }
    }
    if let Some(partner) = &exp.pair {
        match load_expected(fixtures, partner) {
            Ok(other) => {
                if other.pair.as_deref() != Some(name) {
                    out.push(format!(
                        "pair {partner} does not name {name} back (it has {:?})",
                        other.pair
                    ));
                }
                if other.task.prompt != exp.task.prompt {
                    out.push(format!("pair {partner} has a different task.prompt"));
                }
            }
            Err(e) => out.push(format!("pair {partner}: {e}")),
        }
    }
    match exp.category {
        Category::ArchetypeLeak | Category::ArchetypeFree | Category::MultiLeak => {
            if exp.leak.is_empty() {
                out.push(format!(
                    "category {} needs at least one [[leak]]",
                    exp.category.as_str()
                ));
            }
            if exp.category == Category::MultiLeak && exp.leak.len() < 2 {
                out.push("category multi-leak needs at least two [[leak]]".to_owned());
            }
        }
        Category::DecoyControl | Category::NonLeak => {
            if !exp.leak.is_empty() {
                out.push(format!(
                    "category {} must have no [[leak]]",
                    exp.category.as_str()
                ));
            }
            if exp.non_leak.is_empty() {
                out.push(format!(
                    "category {} needs at least one [[non_leak]]",
                    exp.category.as_str()
                ));
            }
        }
        Category::Impossible => {
            if exp.dynamic_confirmation {
                out.push("category impossible needs dynamic_confirmation = false".to_owned());
            }
            for l in exp.leak.iter().filter(|l| l.confirmable) {
                out.push(format!(
                    "category impossible: leak {} must have confirmable = false",
                    l.id
                ));
            }
        }
    }
}
