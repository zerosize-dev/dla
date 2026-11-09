//! Known-leak entries checked against the project.
//!
//! `data/known-leaks.toml` (see `dla_core::known_leaks`) lists documented behavior that
//! looks like a leak or hides one. [`evaluate`] checks each entry of a detected runtime
//! against the project's files and returns one verdict per entry: `true` (the files show
//! the condition), `false` (they rule it out) or `maybe` (they cannot decide). The
//! predicates `all`, `any` and `not` follow three-valued logic, so `maybe` survives
//! negation and only a `false` part makes an `all` false.
//!
//! The verdicts are written to probe evidence. They are context for the analysis and
//! never findings: an entry says what to look for, not that the project leaks.
//!
//! What the files can show:
//!
//! * the runtime version: the `go` line of go.mod for Go, the installed toolchain for
//!   the others; unknown gives `maybe`;
//! * libraries: the manifests read by [`crate::manifests`]. A library missing from a
//!   manifest that is not a full lock file may still arrive transitively; the `why`
//!   says so. With no manifest at all the answer is `maybe`;
//! * config keys: properties and YAML files, `-Dkey=value` flags, `KEY=value` lines,
//!   `GODEBUG` settings, `godebug` lines of go.mod and `//go:debug` lines in the
//!   project's files. Settings made only by the deployment environment are not seen;
//! * source patterns: a regex over the project's source files of the entry's runtime.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use dla_core::{
    known_leaks::{ConfigReq, KnownLeak, KnownLeaksFile, Predicate, VersionRange, parse_version},
    probe::{KnownLeakVerdict, Verdict},
    runtime::Runtime,
};
use regex::Regex;

use crate::{
    manifests::{self, Lib, Manifests, read_capped},
    scan::{Detected, Tree},
};

/// Longest `why`.
const MAX_WHY: usize = 160;
/// Most source files read per runtime.
const MAX_SOURCES: usize = 20_000;
/// Most bytes read for sources and config files together.
const MAX_CORPUS: u64 = 64 * 1024 * 1024;
/// Largest source or config file read.
const MAX_FILE: u64 = 512 * 1024;
/// Most config pairs kept.
const MAX_PAIRS: usize = 100_000;

/// A three-valued truth value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tri {
    True,
    False,
    Maybe,
}

impl Tri {
    fn not(self) -> Tri {
        match self {
            Tri::True => Tri::False,
            Tri::False => Tri::True,
            Tri::Maybe => Tri::Maybe,
        }
    }
}

impl From<Tri> for Verdict {
    fn from(t: Tri) -> Verdict {
        match t {
            Tri::True => Verdict::True,
            Tri::False => Verdict::False,
            Tri::Maybe => Verdict::Maybe,
        }
    }
}

type Res = (Tri, String);

/// Loads `known-leaks.toml` from the plugin data directory. `Ok(None)` when the file is
/// absent (the archetypes-off ablation deletes it); `Err` with the reason when it
/// cannot be read or is invalid.
pub fn load(data_dir: &Path) -> Result<Option<KnownLeaksFile>, String> {
    let path: PathBuf = data_dir.join("known-leaks.toml");
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| format!("known-leaks.toml: {e}"))?;
    KnownLeaksFile::parse(&text, "data/known-leaks.toml")
        .map(Some)
        .map_err(|r| r.to_string())
}

fn source_exts(rt: Runtime) -> &'static [&'static str] {
    match rt {
        Runtime::Jvm => &["java", "kt"],
        Runtime::Go => &["go"],
        Runtime::Rust => &["rs"],
        Runtime::Node => &["ts", "tsx", "js", "jsx", "mjs", "cjs"],
        Runtime::Python => &["py"],
        Runtime::Native => &["c", "cc", "cpp", "cxx", "h", "hpp"],
    }
}

fn is_config_file(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel).to_ascii_lowercase();
    let ext = name.rsplit_once('.').map_or("", |(_, e)| e);
    matches!(
        ext,
        "properties" | "yml" | "yaml" | "sh" | "env" | "conf" | "cfg" | "ini"
    ) || matches!(
        name.as_str(),
        "go.mod"
            | "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
            | "package.json"
            | "makefile"
            | "jvm.config"
            | ".env"
            | "pyproject.toml"
    ) || name.starts_with("dockerfile")
        || name.starts_with("docker-compose")
}

fn norm_key(k: &str) -> String {
    k.chars()
        .filter(|c| !matches!(c, '.' | '-' | '_'))
        .flat_map(char::to_lowercase)
        .collect()
}

fn unquote(v: &str) -> String {
    v.trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .trim()
        .to_owned()
}

/// `(normalized key, value)` pairs found in `text`.
fn extract_pairs(text: &str, yaml: bool, out: &mut Vec<(String, String)>) {
    let dflag = Regex::new(r#"-D([\w.\-]+)=([^\s"'\\]+)"#).expect("static regex");
    let kv = Regex::new(r#"(?m)^\s*(?:export\s+)?([A-Za-z_][\w.\-]*)\s*[=:]\s*["']?([^\s#"']*)"#)
        .expect("static regex");
    let debug = Regex::new(r#"(?m)(?://go:debug|^\s*godebug|GODEBUG\s*[=:])\s*["']?([\w.=,\- ]+)"#)
        .expect("static regex");
    for c in dflag.captures_iter(text) {
        out.push((norm_key(&c[1]), unquote(&c[2])));
    }
    // Gradle `systemProperty("k", "v")` / `systemProperty 'k', 'v'`.
    let sysprop = Regex::new(r#"systemProperty\s*\(?\s*["']([\w.\-]+)["']\s*,\s*["']?([^\s"')]+)"#)
        .expect("static regex");
    for c in sysprop.captures_iter(text) {
        out.push((norm_key(&c[1]), unquote(&c[2])));
    }
    // Maven surefire `<systemPropertyVariables><k>v</k>` (any `<k>v</k>` element).
    let xml = Regex::new(r#"<([\w.\-]+)>\s*([^<>\s]+)\s*</([\w.\-]+)>"#).expect("static regex");
    for c in xml.captures_iter(text) {
        if c[1] == c[3] && out.len() < MAX_PAIRS {
            out.push((norm_key(&c[1]), unquote(&c[2])));
        }
    }
    for c in debug.captures_iter(text) {
        for item in c[1].split([',', ' ']) {
            if let Some((k, v)) = item.split_once('=') {
                out.push((norm_key(k), unquote(v)));
            }
        }
    }
    if yaml {
        let line = Regex::new(r#"^(\s*)([\w.\-/]+):\s*(.*)$"#).expect("static regex");
        let mut stack: Vec<(usize, String)> = Vec::new();
        for l in text.lines() {
            if l.trim_start().starts_with(['#', '-']) {
                continue;
            }
            let Some(c) = line.captures(l) else { continue };
            let indent = c[1].len();
            while stack.last().is_some_and(|(i, _)| *i >= indent) {
                stack.pop();
            }
            let value = c[3].split(" #").next().unwrap_or("").trim();
            let mut key = stack
                .iter()
                .map(|(_, k)| k.as_str())
                .collect::<Vec<_>>()
                .join(".");
            if !key.is_empty() {
                key.push('.');
            }
            key.push_str(&c[2]);
            if value.is_empty() {
                stack.push((indent, c[2].to_owned()));
            } else {
                out.push((norm_key(&key), unquote(value)));
            }
            if out.len() >= MAX_PAIRS {
                return;
            }
        }
    }
    for c in kv.captures_iter(text) {
        if out.len() >= MAX_PAIRS {
            return;
        }
        out.push((norm_key(&c[1]), unquote(&c[2])));
    }
}

/// What the project's files say, read once.
struct Facts {
    runtimes: Vec<Runtime>,
    versions: BTreeMap<Runtime, Option<String>>,
    mf: Manifests,
    config: Vec<(String, String)>,
    sources: BTreeMap<Runtime, Vec<(String, String)>>,
    incomplete: bool,
}

/// The runtime version a check compares: the `go` line of go.mod for Go, else the
/// installed toolchain.
fn runtime_version(d: &Detected) -> Option<String> {
    if d.probe.runtime == Runtime::Go
        && let Some(v) = &d.probe.declared_version
    {
        return Some(v.clone());
    }
    d.probe
        .toolchain
        .iter()
        .find_map(|t| t.version.clone())
        .filter(|v| parse_version(v).is_some())
}

impl Facts {
    fn gather(tree: &Tree, detected: &[Detected]) -> Facts {
        let runtimes: Vec<Runtime> = detected.iter().map(|d| d.probe.runtime).collect();
        let versions = detected
            .iter()
            .map(|d| (d.probe.runtime, runtime_version(d)))
            .collect();
        let mf = manifests::read(tree, &runtimes);
        let mut config = Vec::new();
        let mut sources: BTreeMap<Runtime, Vec<(String, String)>> = BTreeMap::new();
        let mut budget = MAX_CORPUS;
        let mut incomplete = tree.truncated;
        for rel in &tree.files {
            let name = rel.rsplit('/').next().unwrap_or(rel);
            let ext = name.rsplit_once('.').map_or("", |(_, e)| e);
            let src_rt = runtimes
                .iter()
                .copied()
                .find(|rt| source_exts(*rt).contains(&ext));
            let cfg = is_config_file(rel);
            if src_rt.is_none() && !cfg {
                continue;
            }
            if let Some(rt) = src_rt
                && sources.get(&rt).is_some_and(|v| v.len() >= MAX_SOURCES)
            {
                incomplete = true;
                continue;
            }
            if budget == 0 {
                incomplete = true;
                break;
            }
            let Some(text) = read_capped(&tree.root, rel, MAX_FILE) else {
                continue;
            };
            budget = budget.saturating_sub(text.len() as u64);
            if cfg || text.contains("//go:debug") {
                let yaml = ext == "yml" || ext == "yaml";
                extract_pairs(&text, yaml, &mut config);
            }
            if let Some(rt) = src_rt {
                sources.entry(rt).or_default().push((rel.clone(), text));
            }
        }
        Facts {
            runtimes,
            versions,
            mf,
            config,
            sources,
            incomplete,
        }
    }

    fn eval(&self, rt: Runtime, p: &Predicate) -> Res {
        match p {
            Predicate::All(v) => {
                let rs: Vec<Res> = v.iter().map(|x| self.eval(rt, x)).collect();
                if let Some(r) = rs.iter().find(|r| r.0 == Tri::False) {
                    return r.clone();
                }
                if let Some(r) = rs.iter().find(|r| r.0 == Tri::Maybe) {
                    return r.clone();
                }
                (
                    Tri::True,
                    rs.last().map(|r| r.1.clone()).unwrap_or_default(),
                )
            }
            Predicate::Any(v) => {
                let rs: Vec<Res> = v.iter().map(|x| self.eval(rt, x)).collect();
                if let Some(r) = rs.iter().find(|r| r.0 == Tri::True) {
                    return r.clone();
                }
                if let Some(r) = rs.iter().find(|r| r.0 == Tri::Maybe) {
                    return r.clone();
                }
                (
                    Tri::False,
                    rs.first().map(|r| r.1.clone()).unwrap_or_default(),
                )
            }
            Predicate::Not(b) => {
                let (t, why) = self.eval(rt, b);
                (t.not(), why)
            }
            Predicate::RuntimeVersion(range) => self.runtime_in(rt, range),
            Predicate::Library(l) => self.library(rt, &[l.name.as_str()], &l.versions),
            Predicate::Config(c) => self.config_set(c),
            Predicate::SourceRegex(r) => self.source(rt, r),
        }
    }

    fn runtime_in(&self, rt: Runtime, range: &str) -> Res {
        let Ok(r) = VersionRange::parse(range) else {
            return (Tri::Maybe, "bad version range".to_owned());
        };
        match self
            .versions
            .get(&rt)
            .and_then(|v| v.as_deref())
            .and_then(parse_version)
        {
            None => (Tri::Maybe, format!("{} version unknown", rt.as_str())),
            Some(v) => {
                let shown = v.iter().map(u64::to_string).collect::<Vec<_>>().join(".");
                if r.contains(&v) {
                    (Tri::True, format!("{} {shown} in {range}", rt.as_str()))
                } else {
                    (
                        Tri::False,
                        format!("{} {shown} not in {range}", rt.as_str()),
                    )
                }
            }
        }
    }

    fn library(&self, rt: Runtime, names: &[&str], range: &str) -> Res {
        if names.len() == 1 && Runtime::ALL.iter().any(|r| r.as_str() == names[0]) {
            return self.runtime_in(rt, range);
        }
        let Ok(r) = VersionRange::parse(range) else {
            return (Tri::Maybe, "bad version range".to_owned());
        };
        let libs: &[Lib] = self.mf.libs.get(&rt).map_or(&[], Vec::as_slice);
        let hits: Vec<&Lib> = libs
            .iter()
            .filter(|l| names.iter().any(|n| name_matches(&l.name, n)))
            .collect();
        let label = names[0];
        if hits.is_empty() {
            return if self.mf.complete.contains(&rt) {
                (Tri::False, format!("{label} not in the lock file"))
            } else if self.mf.read.contains(&rt) {
                // An empty result never refutes: a transitive or version-catalog use is
                // not seen in a manifest that is not a full lock file.
                (
                    Tri::Maybe,
                    format!("{label} not declared; a transitive or catalog use is not seen"),
                )
            } else {
                (Tri::Maybe, "no manifest read".to_owned())
            };
        }
        if r.is_any() {
            return (
                Tri::True,
                format!("{} present ({})", hits[0].name, hits[0].file),
            );
        }
        let mut unknown: Option<&Lib> = None;
        for l in &hits {
            match l.version.as_deref().and_then(parse_version) {
                Some(v) if r.contains(&v) => {
                    return (
                        Tri::True,
                        format!(
                            "{} {} in {} ({})",
                            l.name,
                            l.version.as_deref().unwrap_or(""),
                            range,
                            l.file
                        ),
                    );
                }
                Some(_) => {}
                None => unknown = Some(l),
            }
        }
        match unknown {
            Some(l) => (
                Tri::Maybe,
                format!("{} version not pinned in {}", l.name, l.file),
            ),
            None => (Tri::False, format!("{label} version outside {range}")),
        }
    }

    fn config_set(&self, c: &ConfigReq) -> Res {
        let key = norm_key(&c.key);
        let mut seen_other: Option<&str> = None;
        for (k, v) in &self.config {
            if *k != key {
                continue;
            }
            match &c.value {
                None => return (Tri::True, format!("{} is set", c.key)),
                Some(want) if v.eq_ignore_ascii_case(want) => {
                    return (Tri::True, format!("{}={want}", c.key));
                }
                Some(_) => seen_other = Some(v),
            }
        }
        if let Some(v) = seen_other {
            let shown: String = v.chars().take(40).collect();
            return (
                Tri::False,
                format!("{} is set to another value", shown_key(&c.key, &shown)),
            );
        }
        if self.incomplete {
            (
                Tri::Maybe,
                format!("{} not seen; the file walk was cut", c.key),
            )
        } else {
            (Tri::False, format!("{} not set in project files", c.key))
        }
    }

    fn source(&self, rt: Runtime, pattern: &str) -> Res {
        let Ok(re) = Regex::new(pattern) else {
            return (Tri::Maybe, "bad source_regex".to_owned());
        };
        if let Some(files) = self.sources.get(&rt) {
            for (rel, text) in files {
                if re.is_match(text) {
                    return (Tri::True, format!("pattern in {rel}"));
                }
            }
        }
        if self.incomplete {
            (
                Tri::Maybe,
                "pattern not seen; the file walk was cut".to_owned(),
            )
        } else {
            (Tri::False, "pattern not in the sources".to_owned())
        }
    }
}

fn shown_key(key: &str, value: &str) -> String {
    format!("{key} ({value})")
}

/// True when manifest name `lib` is the library `want`: equal, the artifact part of a
/// `group:artifact` name, or inside the group of a `group:*` pattern.
fn name_matches(lib: &str, want: &str) -> bool {
    let want = want.to_ascii_lowercase();
    if let Some(group) = want.strip_suffix(":*") {
        return lib.split_once(':').is_some_and(|(g, _)| g == group);
    }
    lib == want || lib.rsplit_once(':').is_some_and(|(_, a)| a == want)
}

fn short(mut s: String) -> String {
    if s.len() > MAX_WHY {
        let mut end = MAX_WHY;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
    }
    s
}

/// Checks every entry of `file` whose runtime is in `detected`. Entries of runtimes the
/// project does not use are skipped.
pub(crate) fn evaluate(
    tree: &Tree,
    detected: &[Detected],
    file: &KnownLeaksFile,
) -> Vec<KnownLeakVerdict> {
    let entries: Vec<&KnownLeak> = file
        .leak
        .iter()
        .filter(|e| detected.iter().any(|d| d.probe.runtime == e.runtime))
        .collect();
    if entries.is_empty() {
        return Vec::new();
    }
    let facts = Facts::gather(tree, detected);
    debug_assert!(!facts.runtimes.is_empty());
    entries
        .into_iter()
        .map(|e| {
            let (t, why) = check(&facts, e);
            KnownLeakVerdict {
                id: e.id.clone(),
                verdict: t.into(),
                why: short(why),
            }
        })
        .collect()
}

fn check(f: &Facts, e: &KnownLeak) -> Res {
    let mut names: Vec<&str> = vec![e.library.as_str()];
    names.extend(e.aliases.iter().map(String::as_str));
    let lib = f.library(e.runtime, &names, &e.versions);
    if lib.0 == Tri::False {
        return lib;
    }
    let Some(w) = &e.when else { return lib };
    let when = f.eval(e.runtime, w);
    match (lib.0, when.0) {
        (Tri::False, _) => lib,
        (_, Tri::False) => when,
        (Tri::Maybe, _) => lib,
        (_, Tri::Maybe) => when,
        (Tri::True, Tri::True) => (Tri::True, short(format!("{}; {}", lib.1, when.1))),
    }
}
