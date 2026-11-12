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
    gzip::{GZIP_MAGIC, gunzip_prefix},
    model::{Category, Entry, Expected, LEAK_CAUSES, NON_LEAK_CAUSES, Runtime},
    schema_cmd,
};

const SOURCE_EXTENSIONS: [&str; 11] = [
    "java", "kt", "go", "rs", "js", "mjs", "cjs", "ts", "py", "c", "h",
];
/// Source file extensions per runtime, for the multi-runtime rule.
fn runtime_extensions(r: Runtime) -> &'static [&'static str] {
    match r {
        Runtime::Jvm => &["java", "kt"],
        Runtime::Go => &["go"],
        Runtime::Rust => &["rs"],
        Runtime::Node => &["js", "mjs", "cjs", "ts"],
        Runtime::Python => &["py"],
        Runtime::Native => &["c", "h"],
    }
}

/// What a heap dump file name must look like inside `project/dumps/`.
const DUMP_SUFFIX: &str = ".hprof.gz";
/// How much of a dump is inflated to read its header.
const DUMP_HEADER_BYTES: usize = 64;
/// The text every HPROF file starts with.
const HPROF_MAGIC: &str = "JAVA PROFILE 1.0.";
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
    let toml_path = dir.join("expected.toml");
    let entry = fs::read_to_string(&toml_path)
        .ok()
        .and_then(|t| toml::from_str::<Expected>(&t).ok())
        .map(|e| e.entry)
        .unwrap_or_default();
    if let Some(p) = &project {
        check_project(p, entry, name, &mut out);
    }
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

/// The header of a heap dump file: the HPROF text and the timestamp in milliseconds.
fn hprof_header(inflated: &[u8]) -> Result<u64, String> {
    if !inflated.starts_with(HPROF_MAGIC.as_bytes()) {
        return Err(format!(
            "does not inflate to a file starting with {HPROF_MAGIC:?}"
        ));
    }
    let Some(nul) = inflated.iter().position(|&c| c == 0) else {
        return Err("the HPROF header text has no terminator".to_owned());
    };
    let Some(ts) = inflated.get(nul + 1 + 4..nul + 1 + 12) else {
        return Err("the HPROF header ends before its timestamp".to_owned());
    };
    let mut b = [0u8; 8];
    b.copy_from_slice(ts);
    Ok(u64::from_be_bytes(b))
}

/// The artifact rules: dumps under `dumps/`, gzip-wrapped, each an HPROF file, with distinct
/// header timestamps.
fn check_dumps(p: &Project, out: &mut Vec<String>) {
    let dumps: Vec<(&String, &Vec<u8>)> = p
        .files
        .iter()
        .filter(|(path, _)| path.starts_with("dumps/"))
        .collect();
    if !dumps.iter().any(|(path, _)| path.ends_with(DUMP_SUFFIX)) {
        out.push(format!(
            "artifact fixture: project/dumps/ needs at least one *{DUMP_SUFFIX} file"
        ));
    }
    let mut stamps: BTreeMap<u64, &str> = BTreeMap::new();
    for (path, data) in dumps {
        if !path.ends_with(DUMP_SUFFIX) {
            out.push(format!(
                "project/{path}: dumps are stored as *{DUMP_SUFFIX}; this name does not end so"
            ));
            continue;
        }
        if data.len() < 2 || data[..2] != GZIP_MAGIC {
            out.push(format!(
                "project/{path}: does not start with the gzip magic 1f 8b"
            ));
            continue;
        }
        match gunzip_prefix(data, DUMP_HEADER_BYTES).and_then(|h| hprof_header(&h)) {
            Err(e) => out.push(format!("project/{path}: {e}")),
            Ok(ts) => {
                if let Some(other) = stamps.insert(ts, path) {
                    out.push(format!(
                        "project/{path}: header timestamp {ts} equals that of project/{other}; a series needs distinct timestamps"
                    ));
                }
            }
        }
    }
}

fn check_project(p: &Project, entry: Entry, id: &str, out: &mut Vec<String>) {
    // Fixture ids are `<runtime>-<name>`: a file that carries one tells the model which
    // fixture it is in. (A hyphen-less id is a synthetic one of the validator's tests.)
    let id = if id.contains('-') { id } else { "\u{0}" };
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
        if path.contains(id) {
            out.push(format!(
                "project/{path}: the path contains the fixture id {id:?}"
            ));
        }
        scan_hints(&path.replace('_', " "), |w| {
            out.push(format!(
                "project/{path}: file name contains hint word {w:?}"
            ));
        });
        if let Ok(text) = std::str::from_utf8(data) {
            for (i, line) in text.lines().enumerate() {
                if line.contains(id) {
                    out.push(format!(
                        "project/{path}:{}: the line contains the fixture id {id:?}",
                        i + 1
                    ));
                }
                scan_hints(&line.replace('_', " "), |w| {
                    out.push(format!("project/{path}:{}: hint word {w:?}", i + 1));
                });
            }
        }
    }
    match entry {
        Entry::Repository => {
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
            if p.files.keys().any(|f| f.ends_with(".kt")) {
                let build = p
                    .files
                    .get("build.sh")
                    .and_then(|b| std::str::from_utf8(b).ok())
                    .unwrap_or("");
                if !build.contains("kotlinc") {
                    out.push(
                        "project/ has .kt sources, so it needs a build.sh that compiles with kotlinc when present and else with the embeddable compiler (see FIXTURES.md)".to_owned(),
                    );
                }
            }
        }
        Entry::Artifact => {
            check_dumps(p, out);
            if source_files > 0 {
                out.push(format!(
                    "artifact fixture: project/ holds {source_files} source file(s); an artifact fixture has dumps and no source"
                ));
            }
        }
        Entry::Symptom => {
            if source_files > 0 {
                out.push(format!(
                    "symptom fixture: project/ holds {source_files} source file(s); it may hold only non-source notes"
                ));
            }
        }
    }
}

fn scan_hints(text: &str, mut hit: impl FnMut(&str)) {
    for m in hint_re().find_iter(text) {
        hit(&m.as_str().to_lowercase());
    }
}

/// An owner is a `package.Class.field` string: no spaces, no path or line parts.
fn check_owner(who: &str, owner: &str, out: &mut Vec<String>) {
    if owner.is_empty()
        || owner != owner.trim()
        || owner.contains(|c: char| c.is_whitespace() || c == '/' || c == '\\' || c == ':')
    {
        out.push(format!(
            "{who}: owner {owner:?} must be a package.Class.field string without spaces, slashes or colons"
        ));
    }
}

/// The multi-runtime rules: `runtimes` holds the primary `runtime` once per entry, at least two
/// entries, each with source files in `project/`; a per-leak `runtime` is one of them.
fn check_runtimes(exp: &Expected, project: Option<&Project>, out: &mut Vec<String>) {
    let all: Vec<Runtime> = match &exp.runtimes {
        Some(list) => {
            if list.len() < 2 {
                out.push(
                    "runtimes lists fewer than two runtimes; omit it for one runtime".to_owned(),
                );
            }
            if !list.contains(&exp.runtime) {
                out.push(format!(
                    "runtime {:?} (the primary) must be one of runtimes",
                    exp.runtime.as_str()
                ));
            }
            let mut seen = std::collections::BTreeSet::new();
            for r in list {
                if !seen.insert(r.as_str()) {
                    out.push(format!("runtimes lists {:?} twice", r.as_str()));
                }
            }
            list.clone()
        }
        None => vec![exp.runtime],
    };
    let per_leak = exp
        .leak
        .iter()
        .map(|l| (format!("leak {}", l.id), l.runtime))
        .chain(
            exp.non_leak
                .iter()
                .map(|n| (format!("non_leak {}", n.id), n.runtime)),
        );
    for (who, rt) in per_leak {
        if let Some(rt) = rt
            && !all.contains(&rt)
        {
            out.push(format!(
                "{who}: runtime {:?} is not one of the fixture's runtimes",
                rt.as_str()
            ));
        }
    }
    if exp.runtimes.is_some()
        && exp.entry == Entry::Repository
        && let Some(p) = project
    {
        for r in &all {
            let has_source = p.files.keys().any(|f| {
                f.rsplit_once('.')
                    .is_some_and(|(_, e)| runtime_extensions(*r).contains(&e))
            });
            if !has_source {
                out.push(format!(
                    "runtimes lists {:?} but project/ has no {} source files",
                    r.as_str(),
                    runtime_extensions(*r).join("/")
                ));
            }
        }
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
    let go_mod = project
        .and_then(|p| p.files.get("go.mod"))
        .and_then(|b| std::str::from_utf8(b).ok());
    crate::fixture_toolchain::check(exp, go_mod, out);
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
    check_runtimes(exp, project, out);
    // An artifact or symptom fixture has no source tree, so a site cannot be located.
    let located = exp.entry == Entry::Repository;
    for l in &exp.leak {
        if let Some(owner) = &l.owner {
            check_owner(&format!("leak {}", l.id), owner, out);
        }
        let Some(site) = &l.site else {
            if l.owner.is_none() {
                out.push(format!(
                    "leak {}: needs a site, or an owner when there is no site",
                    l.id
                ));
            }
            if !l.alt_sites.is_empty() || l.lines.is_some() {
                out.push(format!("leak {}: lines and alt_sites need a site", l.id));
            }
            continue;
        };
        let Some(c) = site_re().captures(site) else {
            out.push(format!("leak {}: site {:?} is not path:line", l.id, site));
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
        if let Some(p) = project.filter(|_| located) {
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
        if let Some(owner) = &n.owner {
            check_owner(&format!("non_leak {}", n.id), owner, out);
        }
        let Some(site) = &n.site else { continue };
        let Some(c) = site_re().captures(site) else {
            out.push(format!("non_leak {}: site {site:?} is not path:line", n.id));
            continue;
        };
        let line: u32 = c[2].parse().unwrap_or(0);
        if let Some(p) = project.filter(|_| located)
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
                check_pair_roles(exp, &other, partner, out);
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

/// A pair is one trap (`archetype-leak`, or `archetype-free` when its cause is
/// UNCLASSIFIED) and one `decoy-control`, in one runtime. A version pair differs in the
/// version of a tool, never in which tools the `[toolchain]` names.
fn check_pair_roles(exp: &Expected, other: &Expected, partner: &str, out: &mut Vec<String>) {
    let is_trap = |c: Category| matches!(c, Category::ArchetypeLeak | Category::ArchetypeFree);
    let ok = (is_trap(exp.category) && other.category == Category::DecoyControl)
        || (exp.category == Category::DecoyControl && is_trap(other.category));
    if !ok {
        out.push(format!(
            "pair {partner}: a pair is one trap (archetype-leak or archetype-free) and one decoy-control, not {} and {}",
            exp.category.as_str(),
            other.category.as_str()
        ));
    }
    if exp.runtime != other.runtime {
        out.push(format!("pair {partner} has a different runtime"));
    }
    let tools = |e: &Expected| {
        e.toolchain
            .as_ref()
            .map(crate::fixture_toolchain::Toolchain::tool_names)
            .unwrap_or_default()
    };
    if tools(exp) != tools(other) {
        out.push(format!(
            "pair {partner} names other tools in [toolchain] ({:?} against {:?}); a version pair differs only in versions",
            tools(exp),
            tools(other)
        ));
    }
}
