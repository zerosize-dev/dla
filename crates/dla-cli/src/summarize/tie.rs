//! Ties: which finding a growth column belongs to.
//!
//! The ledger refuses to confirm a leak finding unless a passing growth column it cites
//! is tied to it (by `finding`, by equal `site`, by `candidates`, or by a column named
//! after the owner). Whoever writes the column sets the tie, so a loose tie here is the
//! same hole as no check at all. When in doubt nothing is tied: the column is still
//! reported, and the output says what is missing.
//!
//! A passing column is tied to a finding only by one of three rules:
//!
//! 1. `holder`: the column measures the finding's holder (a `holder:<key>` column or a
//!    root path), whose owner equals the finding's owner after normalization. A shorter
//!    finding owner (`Cache.MAP`) matches only when exactly one finding and exactly one
//!    holder of the run match it.
//! 2. `site`: the column carries a creation or allocation site, and that site is the
//!    finding's own site or one of its recorded writers (never a remover). A column that
//!    names a holder ties by that holder only.
//! 3. `type`: a growing class equals the accumulation class a heap-graph summary of the
//!    run records for the finding's holder, or the element type of the finding's owner
//!    field as the source declares it (package-aware). Generic JDK and Kotlin types
//!    never tie. A type that matches several findings ties none and is reported as
//!    ambiguous.
//!
//! `--finding F` forces nothing: it narrows the tie to F among columns that already tie
//! to F by these rules.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use dla_core::{
    FindingId, Ledger,
    attribution::{canonical, is_shorter_form},
    evidence::EvidenceKind,
    finding::Finding,
    runtime::Runtime,
};
use serde_json::{Value, json};

/// Why a column is tied: the rule and the key it matched on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TieReason {
    /// `holder`, `site` or `type`.
    pub rule: &'static str,
    /// The matched owner, site or type.
    pub key: String,
}

impl TieReason {
    /// A tie by holder identity.
    pub fn holder(key: &str) -> Self {
        Self {
            rule: "holder",
            key: key.to_owned(),
        }
    }

    /// A tie by site identity.
    pub fn site(key: &str) -> Self {
        Self {
            rule: "site",
            key: key.to_owned(),
        }
    }

    /// A tie by type identity.
    pub fn ty(key: &str) -> Self {
        Self {
            rule: "type",
            key: key.to_owned(),
        }
    }

    /// `{"rule", "key"}`, the key clipped.
    pub fn to_json(&self) -> Value {
        let key: String = self.key.chars().take(100).collect();
        json!({"rule": self.rule, "key": key})
    }
}

/// Whose findings a column may tie to: the findings of its own runtime, and the worker's
/// own plus the core's. A finding is never tied across runtimes or workers.
#[derive(Debug, Clone, Default)]
pub struct Who {
    /// The worker the command runs for (`None`: the core).
    pub worker: Option<String>,
    /// The runtime of the measured process, when known.
    pub runtime: Option<Runtime>,
}

fn runtime_of_site(site: &str) -> Option<Runtime> {
    let file = site.rsplit_once(':').map_or(site, |(f, _)| f);
    match file.rsplit_once('.')?.1 {
        "java" | "kt" | "kts" | "scala" | "groovy" => Some(Runtime::Jvm),
        "rs" => Some(Runtime::Rust),
        "go" => Some(Runtime::Go),
        "py" => Some(Runtime::Python),
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" => Some(Runtime::Node),
        "c" | "cc" | "cpp" | "cxx" | "h" | "hpp" => Some(Runtime::Native),
        _ => None,
    }
}

impl Who {
    /// For a command run by `worker` over a process of `runtime`.
    pub fn new(worker: Option<&str>, runtime: Option<Runtime>) -> Self {
        Self {
            worker: worker.map(str::to_owned),
            runtime,
        }
    }

    /// True when a column of this scope may tie to `f`.
    pub fn sees(&self, f: &Finding) -> bool {
        let worker_ok =
            f.id.prefix()
                .is_none_or(|p| self.worker.as_deref() == Some(p));
        let runtime_ok = match (self.runtime, f.site.as_deref().and_then(runtime_of_site)) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        };
        worker_ok && runtime_ok
    }
}

/// What the rules say about one column.
#[derive(Debug, Default, Clone)]
pub struct Resolved {
    /// The finding the column is tied to (`--finding` when it is among the matches, else
    /// the first by id), and why.
    pub tie: Option<(FindingId, TieReason)>,
    /// The other findings the same exact identity ties (see `resolve`).
    pub also: Vec<FindingId>,
    /// Findings that match by the rules but are several: no tie was made.
    pub ambiguous: Vec<FindingId>,
}

/// Applies `--finding` and uniqueness to the rule matches of one column.
///
/// Exact identity (an exact holder, an exact site or recorded writer) is not ambiguous:
/// every finding at that place is tied, `tie` is the `--finding` one when it is among
/// them, else the first by id, and `also` lists the others. A type match (rule 3) that
/// hits several findings ties none and lists them as ambiguous, even with `only`. With
/// `only`, a column whose matches do not include that finding ties nothing.
pub fn resolve(mut matches: Vec<(FindingId, TieReason)>, only: Option<&FindingId>) -> Resolved {
    let mut seen = BTreeSet::new();
    matches.retain(|(f, _)| seen.insert(f.clone()));
    matches.sort_by(|a, b| a.0.cmp(&b.0));
    if matches.len() > 1 && matches.iter().any(|(_, r)| r.rule == "type") {
        return Resolved {
            tie: None,
            also: Vec::new(),
            ambiguous: matches.into_iter().map(|(f, _)| f).collect(),
        };
    }
    if let Some(only) = only
        && !matches.iter().any(|(f, _)| f == only)
    {
        return Resolved::default();
    }
    if matches.is_empty() {
        return Resolved::default();
    }
    let at = only
        .and_then(|o| matches.iter().position(|(f, _)| f == o))
        .unwrap_or(0);
    let first = matches.remove(at);
    Resolved {
        tie: Some(first),
        also: matches.into_iter().map(|(f, _)| f).collect(),
        ambiguous: Vec::new(),
    }
}

/// The findings of the holder with `owner` (`pkg.Class.field`): every finding whose
/// owner equals it after normalization (exact identity); else the only finding whose
/// owner is a shorter form of it (`Cache.MAP` for `shop.Cache.MAP`), when no other
/// holder in `holders` (the owners of the run's holders) is also matched by that shorter
/// owner. Several shorter forms give none.
pub fn findings_by_holder(
    ledger: &Ledger,
    who: &Who,
    owner: &str,
    holders: &[String],
    site: Option<&str>,
) -> Vec<FindingId> {
    let Some(want) = canonical(owner) else {
        return Vec::new();
    };
    let owned = |f: &Finding| f.owner.as_deref().and_then(canonical);
    let exact: Vec<FindingId> = ledger
        .findings()
        .filter(|f| who.sees(f))
        .filter(|f| owned(f).as_deref() == Some(want.as_str()))
        .map(|f| f.id.clone())
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    let shorter: Vec<(&FindingId, String)> = ledger
        .findings()
        .filter(|f| who.sees(f))
        .filter_map(|f| owned(f).map(|o| (&f.id, o)))
        .filter(|(_, o)| is_shorter_form(&want, o))
        .collect();
    let [(id, short)] = shorter.as_slice() else {
        return Vec::new();
    };
    // A shorter owner is weaker evidence: where the holder's declaration site is known
    // and the finding has another, they are not the same field.
    if let (Some(a), Some(b)) = (site, ledger.finding(id).and_then(|f| f.site.as_deref()))
        && a != b
    {
        return Vec::new();
    }
    let rivals = holders
        .iter()
        .filter_map(|h| canonical(h))
        .filter(|h| h != &want && (h == short || is_shorter_form(h, short)))
        .count();
    if rivals == 0 {
        vec![(*id).clone()]
    } else {
        Vec::new()
    }
}

/// The one finding of a holder, when exactly one matches (see `findings_by_holder`).
pub fn finding_by_holder(
    ledger: &Ledger,
    who: &Who,
    owner: &str,
    holders: &[String],
    site: Option<&str>,
) -> Option<FindingId> {
    let mut v = findings_by_holder(ledger, who, owner, holders, site);
    (v.len() == 1).then(|| v.remove(0))
}

/// The findings at `site` (`file:line`): the finding's own site, or one of its recorded
/// writers. Removers never count.
pub fn findings_at_site(ledger: &Ledger, who: &Who, site: &str) -> Vec<FindingId> {
    ledger
        .findings()
        .filter(|f| who.sees(f))
        .filter(|f| {
            f.site.as_deref() == Some(site)
                || ledger
                    .annotation(&f.id)
                    .is_some_and(|a| a.writers.iter().any(|w| w == site))
        })
        .map(|f| f.id.clone())
        .collect()
}

/// Findings at `site` that do not name another owner than the holder `owner`: a finding
/// of a different holder at the same line is not tied.
pub fn findings_at_site_of_holder(
    ledger: &Ledger,
    who: &Who,
    site: &str,
    owner: &str,
) -> Vec<FindingId> {
    let Some(want) = canonical(owner) else {
        return Vec::new();
    };
    findings_at_site(ledger, who, site)
        .into_iter()
        .filter(|id| {
            ledger
                .finding(id)
                .is_some_and(|f| match f.owner.as_deref().and_then(canonical) {
                    None => true,
                    Some(o) => o == want,
                })
        })
        .collect()
}

/// The site to put on a column tied to `f`: the finding's own site, unless another
/// finding has the same one (the ledger would tie that one by site as well).
pub fn site_of(ledger: &Ledger, f: &FindingId) -> Option<String> {
    let site = ledger.finding(f)?.site.clone()?;
    let shared = ledger
        .findings()
        .any(|g| &g.id != f && g.site.as_deref() == Some(site.as_str()));
    (!shared).then_some(site)
}

/// The tie for a heap holder: by holder identity, else by the declaration site when the
/// finding at that site names no other owner. `holders` are the run's holder owners.
pub fn finding_for_holder(
    ledger: &Ledger,
    who: &Who,
    owner: &str,
    site: Option<&str>,
    holders: &[String],
) -> Option<(FindingId, TieReason)> {
    if let Some(f) = finding_by_holder(ledger, who, owner, holders, site) {
        return Some((f, TieReason::holder(owner)));
    }
    let at = findings_at_site_of_holder(ledger, who, site?, owner);
    match at.as_slice() {
        [one] => Some((one.clone(), TieReason::site(site?))),
        _ => None,
    }
}

/// True for a recorded frame that allocates: a constructor, a static initializer or an
/// allocator. Such a frame is never learned as a writer.
pub fn is_allocator_frame(function: &str) -> bool {
    let f = function.trim();
    f.contains("<init>")
        || f.contains("<clinit>")
        || f.starts_with("alloc::")
        || f.contains("::alloc::")
        || [
            "malloc",
            "calloc",
            "realloc",
            "operator new",
            "__rust_alloc",
        ]
        .iter()
        .any(|a| f.contains(a))
}

/// The runtime a finding's site belongs to, by its file extension.
fn site_is_jvm(site: &str) -> bool {
    let file = site.rsplit_once(':').map_or(site, |(f, _)| f);
    ["java", "kt", "kts", "scala", "groovy"]
        .iter()
        .any(|e| file.rsplit('.').next() == Some(e))
}

// ---------------------------------------------------------------------------------------
// Type identity
// ---------------------------------------------------------------------------------------

const PRIMITIVES: [&str; 8] = [
    "byte", "char", "short", "int", "long", "float", "double", "boolean",
];

/// True for a type that tells nothing about which holder it belongs to: JDK and Kotlin
/// library classes (strings, boxed primitives, collections and their entries and nodes),
/// primitive arrays and arrays of those.
pub fn is_generic_type(ty: &str) -> bool {
    let base = ty.trim_end_matches("[]");
    if PRIMITIVES.contains(&base) {
        return true;
    }
    [
        "java.", "javax.", "jdk.", "sun.", "com.sun.", "kotlin.", "kotlinx.", "scala.",
    ]
    .iter()
    .any(|p| base.starts_with(p))
}

/// What the run's heap-graph summaries say about where each holder's memory piles up.
#[derive(Debug, Default)]
pub struct AccIndex {
    /// `(holder owner, accumulation class)`.
    rows: Vec<(String, String)>,
    /// Owners of every holder read (to tell a shorter owner's rivals).
    owners: Vec<String>,
}

fn passing_holder_keys(ledger: &Ledger) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for e in ledger.evidence() {
        if e.kind != EvidenceKind::GrowthSeries
            || e.tool.name != "dla-hprof"
            || e.result.get("gate").and_then(Value::as_str) != Some("pass")
        {
            continue;
        }
        for c in e
            .result
            .get("passed")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(k) = c
                .get("column")
                .and_then(Value::as_str)
                .and_then(|c| c.strip_prefix("holder:"))
            {
                keys.insert(k.to_owned());
            }
        }
    }
    keys
}

impl AccIndex {
    /// Reads `summaries/*.json` of the run directory: the accumulation class of each
    /// holder that passed a dump series of the run (`holder:<key>` in a `dla-hprof`
    /// growth evidence). Files that are not heap summaries are skipped.
    pub fn load(ledger: &Ledger, run_dir: &Path) -> Self {
        let mut idx = Self::default();
        let passing = passing_holder_keys(ledger);
        let Ok(rd) = fs::read_dir(run_dir.join("summaries")) else {
            return idx;
        };
        let mut files: Vec<_> = rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        files.sort();
        for f in files {
            let Ok(text) = fs::read_to_string(&f) else {
                continue;
            };
            let Ok(doc) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            let Some(holders) = doc.get("holders").and_then(Value::as_array) else {
                continue;
            };
            for h in holders {
                let kind = h.get("kind").and_then(Value::as_str).unwrap_or_default();
                let key = h.get("key").and_then(Value::as_str).unwrap_or_default();
                let owner = match (kind, h.get("field").and_then(Value::as_str)) {
                    ("static", Some(field)) => {
                        let class = h
                            .get("owner_class")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        format!("{class}.{field}")
                    }
                    _ => key.to_owned(),
                };
                idx.owners.push(owner.clone());
                if !passing.contains(key) {
                    continue;
                }
                if let Some(c) = h
                    .get("accumulation")
                    .and_then(|a| a.get("class"))
                    .and_then(Value::as_str)
                {
                    idx.rows.push((owner, c.to_owned()));
                }
            }
        }
        idx
    }

    /// Owners whose accumulation class is exactly `ty`.
    pub fn owners_with_class(&self, ty: &str) -> BTreeSet<&str> {
        self.rows
            .iter()
            .filter(|(_, c)| c == ty)
            .map(|(o, _)| o.as_str())
            .collect()
    }
}

fn find_ident(text: &str, name: &str) -> Option<usize> {
    let is_id = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(at) = text[from..].find(name) {
        let start = from + at;
        let end = start + name.len();
        from = end;
        if (start == 0 || !is_id(bytes[start - 1])) && (end >= bytes.len() || !is_id(bytes[end])) {
            return Some(start);
        }
    }
    None
}

/// The declaration of `field` that starts at `line`: that line, and the following ones
/// only while the statement visibly goes on (the name not yet seen, or a trailing `,`,
/// `<`, `(` or `=`), at most four lines.
fn declaration_text(text: &str, line: u32, field: &str) -> String {
    let mut decl = String::new();
    for l in text.lines().skip((line as usize).saturating_sub(1)).take(4) {
        decl.push_str(l);
        decl.push(' ');
        let t = l.trim_end();
        let goes_on = find_ident(&decl, field).is_none() || t.ends_with([',', '<', '(', '=', '&']);
        if !goes_on {
            break;
        }
    }
    decl
}

/// The last top-level type argument of the first generic in `s`, followed down through
/// nested generics: `Map<String, List<Session>>` gives `Session`.
fn last_type_argument(s: &str) -> Option<String> {
    let open = s.find('<')?;
    let mut depth = 0usize;
    let mut args: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut closed = false;
    for c in s[open + 1..].chars() {
        match c {
            '<' => {
                depth += 1;
                cur.push(c);
            }
            '>' if depth == 0 => {
                closed = true;
                break;
            }
            '>' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => args.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    if !closed {
        return None;
    }
    args.push(cur);
    let last = args.pop()?;
    let last = last.trim();
    let last = last
        .strip_prefix("? extends ")
        .or_else(|| last.strip_prefix("? super "))
        .or_else(|| last.strip_prefix("out "))
        .or_else(|| last.strip_prefix("in "))
        .unwrap_or(last)
        .trim();
    if last.contains('<') {
        return last_type_argument(last)
            .or_else(|| last.split('<').next().map(|h| h.trim().to_owned()));
    }
    let last = last.trim_end_matches('?').trim();
    (!last.is_empty() && last != "?" && last != "*").then(|| last.to_owned())
}

/// The element type of the field declared at `line`, as written: the last type argument
/// of its container (`List<Item>`, `Map<String, Item>`, `mutableListOf<Item>()`), or the
/// element of `Item[]`. `None` for a declaration with neither.
pub fn element_type(text: &str, line: u32, field: &str) -> Option<String> {
    let decl = declaration_text(text, line, field);
    let at = find_ident(&decl, field)?;
    let before = &decl[..at];
    let after = &decl[at + field.len()..];
    let region = if before.contains('<') || before.trim_end().ends_with("[]") {
        before.to_owned()
    } else if let Some(ann) = after.trim_start().strip_prefix(':') {
        ann.split('=').next().unwrap_or(ann).to_owned()
    } else {
        after.to_owned()
    };
    let name = if region.contains('<') {
        last_type_argument(&region)?
    } else {
        let t = region.trim_end();
        let t = t.strip_suffix("[]")?;
        t.rsplit(|c: char| c.is_whitespace() || c == '(')
            .next()?
            .to_owned()
    };
    let name = name.trim_start_matches('@');
    let name = name.trim_end_matches("[]").trim();
    (!name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '$' | '.')))
    .then(|| name.to_owned())
}

/// The `package` of source `text`.
fn package_of(text: &str) -> Option<String> {
    text.lines().take(200).find_map(|l| {
        let rest = l.trim().strip_prefix("package ")?;
        Some(rest.trim().trim_end_matches(';').trim().to_owned())
    })
}

/// The fully qualified names a type written as `written` in source `text` (the file
/// `file_stem`) can denote: qualified as written; else through an explicit import; else
/// in the file's own package, or nested in the file's top class. Wildcard imports are not
/// guessed.
pub fn qualify_type(text: &str, file_stem: &str, written: &str) -> Vec<String> {
    let (head, rest) = written
        .split_once('.')
        .map_or((written, ""), |(h, r)| (h, r));
    let tail = |base: &str| {
        if rest.is_empty() {
            base.to_owned()
        } else {
            format!("{base}.{rest}")
        }
    };
    if head.chars().next().is_some_and(char::is_lowercase) {
        return vec![written.to_owned()];
    }
    let mut out = Vec::new();
    let import = text.lines().find_map(|l| {
        let t = l.trim().strip_prefix("import ")?;
        let t = t.trim().trim_end_matches(';').trim();
        (t.rsplit('.').next() == Some(head) && !t.starts_with("static ")).then(|| t.to_owned())
    });
    if let Some(i) = import {
        out.push(tail(&i));
        return out;
    }
    let pkg = package_of(text).unwrap_or_default();
    let prefix = |p: &str| {
        if p.is_empty() {
            String::new()
        } else {
            format!("{p}.")
        }
    };
    // A class the file declares settles it: nested (indented) means inside the file's
    // top class and wins over a same-package class; top level means the package. A name
    // the file does not declare lives in another file of the package.
    let nested = head != file_stem && declares_nested(text, head);
    if nested {
        out.push(tail(&format!("{}{file_stem}.{head}", prefix(&pkg))));
    } else {
        out.push(tail(&format!("{}{head}", prefix(&pkg))));
    }
    out
}

/// True when `text` declares an indented (nested) type called `name`.
fn declares_nested(text: &str, name: &str) -> bool {
    text.lines().any(|l| {
        if !l.starts_with([' ', '\t']) {
            return false;
        }
        let words: Vec<&str> = l
            .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
            .filter(|w| !w.is_empty())
            .collect();
        words.windows(2).any(|w| {
            w[1] == name && matches!(w[0], "class" | "interface" | "enum" | "record" | "object")
        })
    })
}

/// The field's own declared type, as written, when it is a plain (non-generic, non-array)
/// type: `RequestContext latest;` (Java) or `var latest: RequestContext?` (Kotlin).
/// `None` for a generic, array or inferred declaration.
pub fn own_type(text: &str, line: u32, field: &str) -> Option<String> {
    let decl = declaration_text(text, line, field);
    let at = find_ident(&decl, field)?;
    let before = decl[..at].trim_end();
    let after = decl[at + field.len()..].trim_start();
    let written = if let Some(ann) = after.strip_prefix(':') {
        ann.split(['=', ',', ';'])
            .next()?
            .trim()
            .trim_end_matches('?')
    } else {
        before.rsplit(char::is_whitespace).next()?
    };
    let written = written.trim_start_matches('@').trim();
    let mut parts = written.split('.');
    let plain = !written.is_empty()
        && parts.all(|p| {
            !p.is_empty()
                && p.chars()
                    .all(|c| c.is_alphanumeric() || matches!(c, '_' | '$'))
        })
        && written
            .rsplit('.')
            .next()
            .and_then(|l| l.chars().next())
            .is_some_and(char::is_uppercase);
    plain.then(|| written.to_owned())
}

/// The classes a project's sources declare: `(package, simple name)`, read once.
#[derive(Debug, Default)]
struct ProjectClasses {
    declared: BTreeSet<(String, String)>,
}

impl ProjectClasses {
    fn scan(project: &Path) -> Self {
        let mut out = Self::default();
        let mut stack = vec![project.to_path_buf()];
        let mut files = 0usize;
        while let Some(dir) = stack.pop() {
            let Ok(rd) = fs::read_dir(&dir) else { continue };
            for e in rd.filter_map(Result::ok) {
                let path = e.path();
                let name = e.file_name();
                let name = name.to_string_lossy();
                let Ok(ft) = e.file_type() else { continue };
                if ft.is_dir() {
                    if !name.starts_with('.')
                        && !matches!(&*name, "target" | "build" | "node_modules" | "out")
                    {
                        stack.push(path);
                    }
                    continue;
                }
                if !ft.is_file()
                    || !["java", "kt", "kts", "scala"]
                        .iter()
                        .any(|x| name.ends_with(&format!(".{x}")))
                {
                    continue;
                }
                files += 1;
                if files > 5000 {
                    return out;
                }
                let Ok(text) = fs::read_to_string(&path) else {
                    continue;
                };
                let pkg = package_of(&text).unwrap_or_default();
                for l in text.lines() {
                    let words: Vec<&str> = l
                        .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
                        .filter(|w| !w.is_empty())
                        .collect();
                    for w in words.windows(2) {
                        if matches!(w[0], "class" | "interface" | "enum" | "record" | "object") {
                            out.declared.insert((pkg.clone(), w[1].to_owned()));
                        }
                    }
                }
            }
        }
        out
    }

    /// True when the project declares the class `qualified` (`pkg.Name`, or a nested
    /// `pkg.Outer.Inner`, whose outer class is the one checked).
    fn declares(&self, qualified: &str) -> bool {
        let segs: Vec<&str> = qualified.split('.').collect();
        let Some(first_cap) = segs
            .iter()
            .position(|s| s.chars().next().is_some_and(char::is_uppercase))
        else {
            return false;
        };
        let pkg = segs[..first_cap].join(".");
        self.declared.contains(&(pkg, segs[first_cap].to_owned()))
    }
}

/// What links a growing type to findings: accumulation classes of the run's heap
/// summaries, and the element types the findings' owners declare in the source.
#[derive(Debug, Default)]
pub struct TypeIndex {
    who: Who,
    acc: AccIndex,
    /// Finding, and the qualified element types its declaration can denote.
    declared: Vec<(FindingId, BTreeSet<String>)>,
}

impl TypeIndex {
    /// Reads the run's summaries and, on a repository run, the declaration line of every
    /// JVM finding that has a site and an owner.
    pub fn build(
        ledger: &Ledger,
        who: &Who,
        run_dir: &Path,
        project: &Path,
        artifact_run: bool,
    ) -> Self {
        let mut declared = Vec::new();
        let mut classes: Option<ProjectClasses> = None;
        for f in ledger.findings().filter(|f| who.sees(f)) {
            if artifact_run {
                break;
            }
            let (Some(site), Some(owner)) = (f.site.as_deref(), f.owner.as_deref()) else {
                continue;
            };
            if !site_is_jvm(site) {
                continue;
            }
            let Some((file, line)) = site.rsplit_once(':') else {
                continue;
            };
            let Ok(line) = line.parse::<u32>() else {
                continue;
            };
            let field = owner.rsplit('.').next().unwrap_or(owner);
            let Ok(text) = fs::read_to_string(project.join(file)) else {
                continue;
            };
            let stem = Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let mut names: BTreeSet<String> = BTreeSet::new();
            if let Some(written) = element_type(&text, line, field) {
                names.extend(
                    qualify_type(&text, stem, &written)
                        .into_iter()
                        .filter(|n| !is_generic_type(n)),
                );
            }
            // The field's own declared type counts when it is a class of the project
            // (not a JDK, Kotlin or generic type, and one the sources declare).
            if let Some(written) = own_type(&text, line, field) {
                let classes = classes.get_or_insert_with(|| ProjectClasses::scan(project));
                names.extend(
                    qualify_type(&text, stem, &written)
                        .into_iter()
                        .filter(|n| !is_generic_type(n) && classes.declares(n)),
                );
            }
            if !names.is_empty() {
                declared.push((f.id.clone(), names));
            }
        }
        Self {
            who: who.clone(),
            acc: AccIndex::load(ledger, run_dir),
            declared,
        }
    }

    /// The findings a growing type belongs to, with the rule's key: the type itself.
    /// A generic type belongs to none.
    pub fn findings_for_type(&self, ledger: &Ledger, ty: &str) -> Vec<(FindingId, TieReason)> {
        if is_generic_type(ty) {
            return Vec::new();
        }
        let mut out: BTreeMap<FindingId, TieReason> = BTreeMap::new();
        for owner in self.acc.owners_with_class(ty) {
            for f in findings_by_holder(ledger, &self.who, owner, &self.acc.owners, None) {
                out.entry(f).or_insert_with(|| TieReason::ty(ty));
            }
        }
        let dotted = ty.replace('$', ".");
        for (id, names) in &self.declared {
            if names.contains(&dotted) {
                out.entry(id.clone()).or_insert_with(|| TieReason::ty(ty));
            }
        }
        out.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_types_are_the_last_type_argument_or_the_array_element() {
        let java = "class C {\n    static final List<Entry> TRAIL = new ArrayList<>();\n    Item[] slots;\n    static Map<String, List<Session>> sessions =\n        new HashMap<>();\n    static Map<String, Session> by = new HashMap<>();\n}\n";
        assert_eq!(element_type(java, 2, "TRAIL").as_deref(), Some("Entry"));
        assert_eq!(element_type(java, 3, "slots").as_deref(), Some("Item"));
        assert_eq!(
            element_type(java, 4, "sessions").as_deref(),
            Some("Session")
        );
        assert_eq!(element_type(java, 6, "by").as_deref(), Some("Session"));
        let kt = "object Audit {\n    val trail = mutableListOf<Record>()\n    val m: MutableMap<String, Rec> = HashMap()\n    lateinit var x: Foo\n    val next = ArrayList<Other>()\n}\n";
        assert_eq!(element_type(kt, 2, "trail").as_deref(), Some("Record"));
        assert_eq!(element_type(kt, 3, "m").as_deref(), Some("Rec"));
        // No terminator: the next line's types are not taken.
        assert_eq!(element_type(kt, 4, "x"), None);
        assert_eq!(element_type(java, 99, "x"), None);
    }

    #[test]
    fn types_are_qualified_through_imports_and_the_package() {
        let src = "package shop.codec;\n\nimport shop.model.Record;\n\nclass Audit {}\n";
        assert_eq!(qualify_type(src, "Audit", "Record"), ["shop.model.Record"]);
        // Not declared in the file: another file of the package.
        assert_eq!(qualify_type(src, "Audit", "Entry"), ["shop.codec.Entry"]);
        // Declared nested in the file: only the nested class, never a package sibling.
        let nested =
            "package shop;\n\nclass Cache {\n    static class Entry {}\n    List<Entry> E;\n}\n";
        assert_eq!(qualify_type(nested, "Cache", "Entry"), ["shop.Cache.Entry"]);
        // Declared at top level in the file: the package.
        let top = "package shop;\n\nclass Cache {}\nclass Entry {}\n";
        assert_eq!(qualify_type(top, "Cache", "Entry"), ["shop.Entry"]);
        assert_eq!(qualify_type(src, "Audit", "a.b.Q"), ["a.b.Q"]);
    }

    #[test]
    fn generic_types_never_tie() {
        for t in [
            "java.lang.Object[]",
            "byte[]",
            "char[]",
            "java.lang.String",
            "java.lang.Long",
            "java.util.HashMap$Node",
            "java.util.concurrent.ConcurrentHashMap$Node[]",
            "kotlin.collections.ArrayList",
        ] {
            assert!(is_generic_type(t), "{t}");
        }
        assert!(!is_generic_type("shop.Item"));
        assert!(!is_generic_type("shop.Item[]"));
    }

    #[test]
    fn allocator_frames_are_recognized() {
        for f in [
            "com.acme.Session.<init>",
            "com.acme.Cfg.<clinit>",
            "alloc::vec::Vec::push",
            "malloc",
        ] {
            assert!(is_allocator_frame(f), "{f}");
        }
        assert!(!is_allocator_frame("shop::trail::record"));
        assert!(!is_allocator_frame("shop.Cache.put"));
    }

    #[test]
    fn own_types_are_plain_declared_types_only() {
        let java = "class C {\n    static RequestContext latest;\n    private static final List<Item> L = null;\n    static String name;\n    Item[] slots;\n    static a.b.Q q;\n}\n";
        assert_eq!(
            own_type(java, 2, "latest").as_deref(),
            Some("RequestContext")
        );
        assert_eq!(own_type(java, 3, "L"), None);
        assert_eq!(own_type(java, 5, "slots"), None);
        assert_eq!(own_type(java, 6, "q").as_deref(), Some("a.b.Q"));
        let kt = "object A {\n    var latest: RequestContext? = null\n    val m: Map<String, X> = HashMap()\n    val inferred = Foo()\n}\n";
        assert_eq!(own_type(kt, 2, "latest").as_deref(), Some("RequestContext"));
        assert_eq!(own_type(kt, 3, "m"), None);
        assert_eq!(own_type(kt, 4, "inferred"), None);
    }

    #[test]
    fn project_classes_are_found_by_package() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("src/gateway")).unwrap();
        fs::write(
            d.path().join("src/gateway/Ctx.java"),
            "package gateway;\n\nfinal class Ctx {\n    static class Inner {}\n}\n",
        )
        .unwrap();
        let c = ProjectClasses::scan(d.path());
        assert!(c.declares("gateway.Ctx"));
        assert!(c.declares("gateway.Ctx.Inner"));
        assert!(!c.declares("gateway.Map"));
        assert!(!c.declares("other.Ctx"));
    }

    #[test]
    fn resolve_exact_matches_tie_all_and_type_matches_stay_ambiguous() {
        let f = |s: &str| s.parse::<FindingId>().unwrap();
        let site = |id: &str| (f(id), TieReason::site("a.c:1"));
        let r = resolve(vec![site("F3"), site("F1")], None);
        assert_eq!(r.tie.unwrap().0, f("F1"));
        assert_eq!(r.also, [f("F3")]);
        assert!(r.ambiguous.is_empty());
        let r = resolve(vec![site("F1"), site("F3")], Some(&f("F3")));
        assert_eq!(r.tie.unwrap().0, f("F3"));
        assert_eq!(r.also, [f("F1")]);
        // `--finding` that is not among the exact matches ties nothing.
        assert!(
            resolve(vec![site("F1"), site("F3")], Some(&f("F2")))
                .tie
                .is_none()
        );
        let ty = |id: &str| (f(id), TieReason::ty("a.B"));
        let r = resolve(vec![ty("F1"), ty("F2")], None);
        assert!(r.tie.is_none() && r.also.is_empty());
        assert_eq!(r.ambiguous, [f("F1"), f("F2")]);
    }

    #[test]
    fn resolve_ties_one_match_and_reports_several() {
        let f = |s: &str| s.parse::<FindingId>().unwrap();
        let one = vec![(f("F1"), TieReason::ty("a.B"))];
        assert_eq!(resolve(one, None).tie.unwrap().0, f("F1"));
        let two = vec![
            (f("F1"), TieReason::ty("a.B")),
            (f("F2"), TieReason::ty("a.B")),
        ];
        let r = resolve(two.clone(), None);
        assert!(r.tie.is_none());
        assert_eq!(r.ambiguous, [f("F1"), f("F2")]);
        // An ambiguity is final: `--finding` neither settles it nor forces a tie.
        let r = resolve(two.clone(), Some(&f("F2")));
        assert!(r.tie.is_none());
        assert_eq!(r.ambiguous, [f("F1"), f("F2")]);
        assert!(resolve(two, Some(&f("F3"))).tie.is_none());
        let one = vec![(f("F1"), TieReason::ty("a.B"))];
        assert_eq!(resolve(one.clone(), Some(&f("F1"))).tie.unwrap().0, f("F1"));
        assert!(resolve(one, Some(&f("F2"))).tie.is_none());
        assert!(resolve(Vec::new(), Some(&f("F1"))).tie.is_none());
    }
}
