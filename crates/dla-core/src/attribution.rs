//! Owner-to-site resolution: from a heap holder key or owner to the census entry's
//! declaration site and writers.
//!
//! A heap dump names a holder by class and field (`static:shop.codec.Audit.trail`), never
//! by source line. The census of a repository run lists the same field with its
//! declaration line and the sites that add to it and remove from it. [`resolve`] joins
//! the two, and says which way it found the site ([`Via`]):
//!
//! 1. `census`: the entry of `census/<lang>.json` whose owner matches;
//! 2. `source-scan`: no census entry, so the project is searched for `<Class>.java` or
//!    `<Class>.kt` and the field's declaration line in it.
//!
//! Nothing is guessed: an owner that matches several census entries, or a field whose
//! declaration cannot be found, resolves to nothing. An artifact run has no source, so
//! callers do not use a site from here there (the ledger refuses it).

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::{
    census::CensusEntry,
    finding::{Holder, Region},
    paths::to_slash,
};

/// How a site was found.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// A census entry names the owner.
    #[serde(rename = "census")]
    Census,
    /// A scan of the project's `<Class>.java` or `<Class>.kt` found the declaration.
    #[serde(rename = "source-scan")]
    SourceScan,
}

impl Via {
    /// The word used in output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Census => "census",
            Self::SourceScan => "source-scan",
        }
    }
}

/// The source facts for one owner.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Resolved {
    /// The owner as resolved: `pkg.Class.field`.
    pub owner: String,
    /// Declaration site, `file:line`, project-relative.
    pub site: String,
    /// How it was found.
    pub via: Via,
    /// Census kind (`static-field`, ...), for a census hit.
    pub kind: Option<String>,
    /// Census holder class, for a census hit.
    pub holder: Option<Holder>,
    /// Census region, for a census hit.
    pub region: Option<Region>,
    /// Heuristic `file:line` sites that add to the owner (census only).
    pub writers: Vec<String>,
    /// Heuristic `file:line` sites that remove from it (census only).
    pub removers: Vec<String>,
}

/// Removes a generated-class suffix from one class name segment: Spring CGLIB
/// (`X$$SpringCGLIB$$0`, `X$$EnhancerBySpringCGLIB$$ab12`) and other `$$` proxies.
fn deproxy(segment: &str) -> &str {
    segment.split("$$").next().unwrap_or(segment)
}

/// The owner `pkg.Class.field` in a holder key or owner string, or `None` for a key that
/// names no field (`thread:main`, `jni-global:...`, `instance:...`). Accepts
/// `static:<pkg.Class>.<field>`, `<pkg.Class>.<field>` and `<pkg.Class$Inner>.<field>`;
/// proxy suffixes are removed and `$` becomes `.`.
pub fn normalize(key_or_owner: &str) -> Option<String> {
    let s = key_or_owner.trim();
    let s = match s.split_once(':') {
        Some(("static", rest)) => rest,
        Some(_) => return None,
        None => s,
    };
    if s.is_empty() || s.contains(char::is_whitespace) || s.contains('>') {
        return None;
    }
    let cleaned: Vec<String> = s
        .split('.')
        .map(deproxy)
        .map(|p| p.replace('$', "."))
        .collect();
    let joined = cleaned.join(".");
    let joined = joined.trim_matches('.').to_owned();
    // At least `Class.field`.
    (joined.split('.').filter(|p| !p.is_empty()).count() >= 2).then_some(joined)
}

fn dotted_suffix(long: &str, short: &str) -> bool {
    long == short
        || long
            .strip_suffix(short)
            .is_some_and(|rest| rest.ends_with('.'))
}

/// [`normalize`], then the Kotlin forms the census and the JVM name differently are made
/// one: a companion field `Svc.Companion.CACHE` is `Svc.CACHE` (the JVM keeps it as a
/// static of `Svc`), and a top-level property `pkg.AKt.top` of file `A.kt` is `pkg.top`
/// (the census names it by package).
pub fn canonical(key_or_owner: &str) -> Option<String> {
    let n = normalize(key_or_owner)?;
    let mut parts: Vec<&str> = n.split('.').collect();
    let mut i = 0;
    while i < parts.len() {
        if parts[i] == "Companion" && i > 0 && i + 1 < parts.len() {
            parts.remove(i);
        } else {
            i += 1;
        }
    }
    // `<pkg>.<File>Kt.<prop>`: the facade class sits right before the property, after a
    // lower-case package.
    let n = parts.len();
    if n >= 3
        && parts[n - 2].len() > 2
        && parts[n - 2].ends_with("Kt")
        && parts[n - 2].starts_with(char::is_uppercase)
        && parts[n - 3].starts_with(char::is_lowercase)
    {
        parts.remove(n - 2);
    }
    (parts.len() >= 2).then(|| parts.join("."))
}

/// True when `short` is a proper trailing run of dotted components of `long` (both
/// canonical): `Audit.trail` of `shop.codec.Audit.trail`.
pub fn is_shorter_form(long: &str, short: &str) -> bool {
    long != short && dotted_suffix(long, short)
}

/// True when the two owners name one field: equal, or one is a trailing run of dotted
/// components of the other (`Audit.trail` and `shop.codec.Audit.trail`), after
/// [`canonical`]. A bare field name matches nothing: both need a class part.
pub fn owners_match(a: &str, b: &str) -> bool {
    let (Some(a), Some(b)) = (canonical(a), canonical(b)) else {
        return false;
    };
    dotted_suffix(&a, &b) || dotted_suffix(&b, &a)
}

/// Every entry of `<run_dir>/census/*.json`, in file-name order. Unreadable files and
/// files without `entries` are skipped.
pub fn census_entries(run_dir: &Path) -> Vec<CensusEntry> {
    let dir = run_dir.join("census");
    let Ok(rd) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = rd
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let Ok(text) = fs::read_to_string(&f) else {
            continue;
        };
        let Ok(doc) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(entries) = doc
            .get("entries")
            .and_then(|e| serde_json::from_value::<Vec<CensusEntry>>(e.clone()).ok())
        {
            out.extend(entries);
        }
    }
    out
}

/// The census entry for `owner`: an exact owner first, else the only entry that matches
/// by dotted suffix. Several matches resolve to nothing.
pub fn find_entry<'e>(entries: &'e [CensusEntry], owner: &str) -> Option<&'e CensusEntry> {
    let want = canonical(owner)?;
    let exact: Vec<&CensusEntry> = entries
        .iter()
        .filter(|e| canonical(&e.owner).as_deref() == Some(want.as_str()))
        .collect();
    if exact.len() == 1 {
        return exact.first().copied();
    }
    if exact.len() > 1 {
        return None;
    }
    let loose: Vec<&CensusEntry> = entries
        .iter()
        .filter(|e| owners_match(&e.owner, &want))
        .collect();
    if loose.len() == 1 {
        loose.first().copied()
    } else {
        None
    }
}

/// Resolves `key_or_owner` for a run: the census of `run_dir` first, then a source scan
/// of `project`. `None` when neither finds the field. Use a [`Resolver`] to resolve many
/// owners: it reads the census and walks the project once.
pub fn resolve(run_dir: &Path, project: &Path, key_or_owner: &str) -> Option<Resolved> {
    Resolver::new(run_dir, project).resolve(key_or_owner)
}

/// Directories a source scan never enters.
const SKIP_DIRS: [&str; 9] = [
    ".git",
    ".dla",
    "target",
    "build",
    "node_modules",
    "out",
    ".gradle",
    ".idea",
    "dist",
];

/// Most files a scan visits.
const MAX_SCAN_FILES: usize = 40_000;

/// Largest source file read.
const MAX_SOURCE_BYTES: u64 = 2 << 20;

/// The Java and Kotlin files of a project, by file name.
#[derive(Debug, Default, Clone)]
pub struct SourceIndex {
    files: std::collections::BTreeMap<String, Vec<PathBuf>>,
}

impl SourceIndex {
    /// Walks `project` once (skipping build output, hidden directories and
    /// `node_modules`; at most 40,000 files) and indexes `.java` and `.kt` files.
    pub fn build(project: &Path) -> Self {
        let mut idx = Self::default();
        let mut seen = 0usize;
        idx.walk(project, &mut seen);
        idx
    }

    fn walk(&mut self, dir: &Path, seen: &mut usize) {
        let Ok(rd) = fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = rd.filter_map(Result::ok).collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            if *seen >= MAX_SCAN_FILES {
                return;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
                    continue;
                }
                self.walk(&e.path(), seen);
            } else if ft.is_file() {
                *seen += 1;
                if name.ends_with(".java") || name.ends_with(".kt") {
                    self.files.entry(name).or_default().push(e.path());
                }
            }
        }
    }

    /// The source files that hold class `fqn` (`pkg.Outer.Inner`; nested classes are
    /// looked up in the outer class's file). When the name has a package, a file counts
    /// only if its path contains `<pkg path>/<Name>.` or its `package` line says so; a
    /// file of another package with the same simple name is never returned. A Kotlin
    /// facade class `AKt` is looked up as `A.kt`. A bare class name (no package) matches
    /// the files of that name.
    pub fn files_of_class(&self, fqn: &str) -> Vec<PathBuf> {
        let Some(top) = top_class(fqn) else {
            return Vec::new();
        };
        let mut names: Vec<&str> = vec![top.name];
        if let Some(stem) = top.name.strip_suffix("Kt").filter(|s| !s.is_empty()) {
            names.push(stem);
        }
        let mut found: Vec<(PathBuf, &str)> = Vec::new();
        for name in names {
            for ext in ["java", "kt"] {
                if let Some(v) = self.files.get(&format!("{name}.{ext}")) {
                    found.extend(v.iter().cloned().map(|p| (p, name)));
                }
            }
        }
        if top.pkg_path.is_empty() {
            return found.into_iter().map(|(p, _)| p).collect();
        }
        let pkg_dotted = top.pkg_path.replace('/', ".");
        found
            .into_iter()
            .filter(|(p, name)| match package_of(p) {
                Some(pkg) => pkg == pkg_dotted,
                None => {
                    let slash = format!("/{}", to_slash(p));
                    slash.contains(&format!("/{}/{name}.", top.pkg_path))
                }
            })
            .map(|(p, _)| p)
            .collect()
    }
}

/// The `package` a Java or Kotlin file declares, read from its first lines.
fn package_of(path: &Path) -> Option<String> {
    use std::io::{BufRead, BufReader};
    let f = fs::File::open(path).ok()?;
    for line in BufReader::new(f).lines().take(200) {
        let line = line.ok()?;
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("package ") {
            let name = rest.trim().trim_end_matches(';').trim();
            return (!name.is_empty() && !name.contains(char::is_whitespace))
                .then(|| name.to_owned());
        }
    }
    None
}

struct TopClass<'a> {
    name: &'a str,
    pkg_path: String,
}

/// The outermost class in a dotted class path and its package as a path.
fn top_class(fqn: &str) -> Option<TopClass<'_>> {
    let parts: Vec<&str> = fqn.split('.').filter(|p| !p.is_empty()).collect();
    let top_at = parts
        .iter()
        .position(|p| p.chars().next().is_some_and(char::is_uppercase))
        .unwrap_or(parts.len().checked_sub(1)?);
    Some(TopClass {
        name: parts.get(top_at)?,
        pkg_path: parts[..top_at].join("/"),
    })
}

/// Joins census entries and a source scan for one run. Cheap to build; reads the census
/// and walks the project only when first needed.
pub struct Resolver {
    run_dir: PathBuf,
    project: PathBuf,
    entries: std::sync::OnceLock<Vec<CensusEntry>>,
    index: std::sync::OnceLock<SourceIndex>,
}

impl Resolver {
    /// A resolver for the run at `run_dir` over `project`.
    pub fn new(run_dir: &Path, project: &Path) -> Self {
        Self {
            run_dir: run_dir.to_path_buf(),
            project: project.to_path_buf(),
            entries: std::sync::OnceLock::new(),
            index: std::sync::OnceLock::new(),
        }
    }

    /// The census entries of the run.
    pub fn entries(&self) -> &[CensusEntry] {
        self.entries.get_or_init(|| census_entries(&self.run_dir))
    }

    /// The project's source index.
    pub fn index(&self) -> &SourceIndex {
        self.index.get_or_init(|| SourceIndex::build(&self.project))
    }

    /// See [`resolve`].
    pub fn resolve(&self, key_or_owner: &str) -> Option<Resolved> {
        let owner = normalize(key_or_owner)?;
        if let Some(e) = find_entry(self.entries(), &owner) {
            return Some(Resolved {
                owner: e.owner.clone(),
                site: e.site.clone(),
                via: Via::Census,
                kind: Some(e.kind.clone()),
                holder: Some(e.holder),
                region: Some(e.region),
                writers: e.writers.clone(),
                removers: e.removers.clone(),
            });
        }
        let site = self.scan_source(&owner)?;
        Some(Resolved {
            owner,
            site,
            via: Via::SourceScan,
            kind: None,
            holder: None,
            region: None,
            writers: Vec::new(),
            removers: Vec::new(),
        })
    }

    /// The declaration line of the field of `owner` (`pkg.Class.field`) in the class's
    /// source file, as `file:line` relative to the project. `None` when there is no such
    /// file, or when more than one file declares it.
    pub fn scan_source(&self, owner: &str) -> Option<String> {
        let owner = normalize(owner)?;
        let (class_path, field) = owner.rsplit_once('.')?;
        let mut hits: Vec<(PathBuf, u32)> = Vec::new();
        for p in self.index().files_of_class(class_path) {
            let Ok(md) = fs::metadata(&p) else { continue };
            if md.len() > MAX_SOURCE_BYTES {
                continue;
            }
            let Ok(text) = fs::read_to_string(&p) else {
                continue;
            };
            if let Some(line) = declaration_line(&text, field) {
                hits.push((p, line));
            }
        }
        if hits.len() != 1 {
            return None;
        }
        let (path, line) = hits.remove(0);
        let rel = path.strip_prefix(&self.project).ok()?;
        Some(format!("{}:{line}", to_slash(rel)))
    }

    /// The `file:line` of a stack frame in the project, or `None` for a frame outside it.
    /// A frame with a file path is checked against the project directly; a JVM frame
    /// (`pkg.Class.method`, with or without a file name) is looked up as a source file
    /// of its class. `line` of 0 or `None` gives the file's first line.
    pub fn frame_site(
        &self,
        function: &str,
        file: Option<&str>,
        line: Option<u32>,
    ) -> Option<String> {
        let line = line.filter(|l| *l > 0).unwrap_or(1);
        if let Some(f) = file.filter(|f| f.contains('/') || f.contains('\\')) {
            let f = f.replace('\\', "/");
            let p = Path::new(&f);
            // A harness run executes a copy of the project under
            // `.dla/<run>/harness/H<n>/work`: a frame in that copy is the project's own
            // file at the path below it. Other files of a run directory (the harness
            // agent) are not project files.
            if let Some(rest) = below_scratch(&f) {
                return self
                    .project
                    .join(rest)
                    .is_file()
                    .then(|| format!("{rest}:{line}"));
            }
            let rel = if p.is_absolute() {
                p.strip_prefix(&self.project).ok().map(to_slash)
            } else {
                self.project.join(p).is_file().then(|| f.clone())
            }?;
            if rel == ".dla" || rel.starts_with(".dla/") {
                return None;
            }
            return Some(format!("{rel}:{line}"));
        }
        let class_path = function.rsplit_once('.')?.0;
        let class_path = normalize(&format!("{class_path}.x"))?;
        let class_path = class_path.rsplit_once('.')?.0;
        let files = self.index().files_of_class(class_path);
        if files.len() != 1 {
            return None;
        }
        let rel = files[0].strip_prefix(&self.project).ok()?;
        Some(format!("{}:{line}", to_slash(rel)))
    }
}

/// The path below the scratch copy of a harness run, `src/a.js` of
/// `/p/.dla/r1/harness/H1/work/src/a.js` (slash form), or `None` for any other path.
/// The scratch directory must sit in a `.dla/<run>` directory.
pub fn below_scratch(path: &str) -> Option<&str> {
    let mut from = 0;
    while let Some(i) = path[from..].find("/harness/H") {
        let at = from + i;
        from = at + 1;
        let rest = &path[at + "/harness/H".len()..];
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        let Some(below) = rest[digits..].strip_prefix("/work/").filter(|_| digits > 0) else {
            continue;
        };
        // The two components above `harness` are `.dla` and the run id.
        let in_run_dir = path[..at].rsplit_once('/').is_some_and(|(above, run)| {
            !run.is_empty() && (above == ".dla" || above.ends_with("/.dla"))
        });
        if in_run_dir && !below.is_empty() {
            return Some(below);
        }
    }
    None
}

/// Finds the declaration line of `field` in source `text` (Java or Kotlin): a line where
/// the field name is followed by `=`, `;`, `:` or `,` and preceded by a type or a
/// `val`/`var`, not by `.`, `(` or an operator. Returns the 1-based line.
pub fn declaration_line(text: &str, field: &str) -> Option<u32> {
    if field.is_empty() {
        return None;
    }
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("//") || t.starts_with('*') || t.starts_with("/*") {
            continue;
        }
        let bytes = line.as_bytes();
        let mut from = 0;
        while let Some(at) = line[from..].find(field) {
            let start = from + at;
            let end = start + field.len();
            from = end;
            let before_ok = start == 0 || !is_ident(bytes[start - 1]);
            let after_ok = end >= bytes.len() || !is_ident(bytes[end]);
            if !before_ok || !after_ok {
                continue;
            }
            let prev = line[..start].trim_end().chars().next_back();
            let next = line[end..].trim_start().chars().next();
            let prev_is_type =
                prev.is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '>' | ']' | '?'));
            let next_is_decl = matches!(next, Some('=' | ';' | ':' | ',') | None);
            let word_before = line[..start]
                .split_whitespace()
                .next_back()
                .unwrap_or_default();
            let keyword_stmt = matches!(word_before, "return" | "else" | "throw" | "new");
            if prev_is_type && next_is_decl && !keyword_stmt {
                return u32::try_from(i + 1).ok();
            }
        }
    }
    None
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_in_the_harness_copy_is_the_project_file_below_it() {
        assert_eq!(
            below_scratch("/p/.dla/20261111-1-ab/harness/H1/work/src/handler.js"),
            Some("src/handler.js")
        );
        assert_eq!(
            below_scratch(".dla/r/harness/H12/work/a/b.go"),
            Some("a/b.go")
        );
        // Not a scratch copy: no `.dla/<run>` above, no digits, the agent directory.
        assert_eq!(below_scratch("/p/harness/H1/work/src/a.js"), None);
        assert_eq!(below_scratch("/p/.dla/r/harness/Hx/work/a.js"), None);
        assert_eq!(
            below_scratch("/p/.dla/r/harness/H1/agent/dla-preload.js"),
            None
        );
        assert_eq!(below_scratch("/p/.dla/r/harness/H1/work/"), None);
    }

    #[test]
    fn frame_site_maps_the_harness_copy_to_the_project_path() {
        let project = tempfile::tempdir().unwrap();
        let run = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("src")).unwrap();
        fs::write(project.path().join("src/handler.js"), "x\n").unwrap();
        let r = Resolver::new(run.path(), project.path());
        let p = project.path().to_string_lossy().replace('\\', "/");
        let scratch = format!("{p}/.dla/20261111-1-ab/harness/H1/work/src/handler.js");
        assert_eq!(
            r.frame_site("handle", Some(&scratch), Some(10)).as_deref(),
            Some("src/handler.js:10")
        );
        // The same file as the project path, and a copy of another run directory.
        assert_eq!(
            r.frame_site("handle", Some(&format!("{p}/src/handler.js")), Some(10))
                .as_deref(),
            Some("src/handler.js:10")
        );
        assert_eq!(
            r.frame_site(
                "handle",
                Some("/elsewhere/.dla/r2/harness/H3/work/src/handler.js"),
                Some(4)
            )
            .as_deref(),
            Some("src/handler.js:4")
        );
        // A file of the run directory that is not a project file is no frame.
        let agent = format!("{p}/.dla/20261111-1-ab/harness/H1/agent/dla-preload.js");
        assert_eq!(r.frame_site("hook", Some(&agent), Some(161)), None);
        // A copy path whose file the project does not have is no frame.
        let gone = format!("{p}/.dla/20261111-1-ab/harness/H1/work/src/nope.js");
        assert_eq!(r.frame_site("f", Some(&gone), Some(1)), None);
    }

    #[test]
    fn normalize_accepts_keys_and_owners() {
        assert_eq!(
            normalize("static:shop.codec.Audit.trail").as_deref(),
            Some("shop.codec.Audit.trail")
        );
        assert_eq!(normalize("Audit.trail").as_deref(), Some("Audit.trail"));
        assert_eq!(
            normalize("static:a.Outer$Inner.field").as_deref(),
            Some("a.Outer.Inner.field")
        );
        assert_eq!(
            normalize("static:a.Svc$$SpringCGLIB$$0.cache").as_deref(),
            Some("a.Svc.cache")
        );
        assert_eq!(normalize("thread:main"), None);
        assert_eq!(normalize("jni-global:a.B"), None);
        assert_eq!(normalize("instance:root>a.B"), None);
        assert_eq!(normalize("field"), None);
        assert_eq!(normalize(""), None);
    }

    #[test]
    fn owners_match_by_dotted_suffix_only() {
        assert!(owners_match("shop.codec.Audit.trail", "Audit.trail"));
        assert!(owners_match(
            "static:shop.codec.Audit.trail",
            "shop.codec.Audit.trail"
        ));
        assert!(!owners_match("shop.codec.Audit.trail", "udit.trail"));
        assert!(!owners_match("shop.codec.Audit.trail", "trail"));
        assert!(!owners_match(
            "shop.codec.Audit.trail",
            "shop.codec.Audit.other"
        ));
    }

    #[test]
    fn declaration_lines_are_found_for_java_and_kotlin() {
        let java = "package a;\npublic class C {\n  // items\n  static final List<byte[]> ITEMS = new ArrayList<>();\n  void f() { ITEMS.add(x); }\n}\n";
        assert_eq!(declaration_line(java, "ITEMS"), Some(4));
        let kt = "object Audit {\n    val trail = mutableListOf<String>()\n    fun add(s: String) { trail.add(s) }\n}\n";
        assert_eq!(declaration_line(kt, "trail"), Some(2));
        assert_eq!(declaration_line(kt, "missing"), None);
        let usage_only = "class C {\n void f() { this.cache = null; return cache; }\n}\n";
        assert_eq!(declaration_line(usage_only, "cache"), None);
    }
}
