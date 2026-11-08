//! Source scanning and the per-file facts workload discovery needs.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use dla_census::parse::{Lang, parse, tree_sitter::Node};
use dla_core::{runtime::Runtime, state::Scope, workload::WorkloadKind};

/// Directories that never hold project sources.
pub const SKIP_DIRS: &[&str] = &[
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
];
const MAX_FILE_BYTES: u64 = 1_500_000;
const MAX_FILES: usize = 20_000;

/// A public function, method or handler of the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// `path::Owner.name` or `path::name`.
    pub id: String,
    /// Bare name matched against identifiers.
    pub name: String,
    /// Identifiers referenced inside the definition (its own name excluded); the
    /// edges followed when a workload reaches this entry point.
    pub refs: BTreeSet<String>,
}

/// Facts about one source file.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub rel: String,
    pub lang: Lang,
    pub text: String,
    pub is_test: bool,
    /// Identifiers referenced (definition names excluded).
    pub idents: BTreeSet<String>,
    /// Rust only: identifiers inside inline test code.
    pub test_idents: BTreeSet<String>,
    /// Public entry points defined here (empty for test files).
    pub entries: Vec<Entry>,
}

/// A workload before its entry points are matched.
#[derive(Debug, Clone)]
pub struct Raw {
    pub id: String,
    pub kind: WorkloadKind,
    pub runtime: Runtime,
    pub command: String,
    pub cwd: String,
    pub file: String,
    /// Source files whose identifiers the workload references.
    pub srcs: Vec<String>,
    /// Extra identifiers (inline Rust tests).
    pub extra: BTreeSet<String>,
}

/// Project files by relative path.
pub struct Scan {
    root: std::path::PathBuf,
    /// Every non-skipped file, `/` separated.
    pub all: BTreeSet<String>,
    /// Parsed source files inside the run's scope.
    pub infos: BTreeMap<String, FileInfo>,
    /// The run's scope; files outside it are listed in `all` (build files above the
    /// scope are still needed to find a module's root) but never parsed.
    pub scope: Option<Scope>,
    /// Non-test source files outside the scope that were not parsed, by runtime.
    pub out_of_scope: BTreeMap<Runtime, u64>,
}

pub fn dir_of(rel: &str) -> &str {
    rel.rfind('/').map_or("", |i| &rel[..i])
}

pub fn join(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_owned()
    } else {
        format!("{dir}/{name}")
    }
}

/// `rel` relative to `dir` (which must be a prefix directory of `rel`).
pub fn strip_dir<'a>(rel: &'a str, dir: &str) -> &'a str {
    if dir.is_empty() {
        rel
    } else {
        rel.strip_prefix(dir)
            .and_then(|r| r.strip_prefix('/'))
            .unwrap_or(rel)
    }
}

pub fn cwd_of(dir: &str) -> String {
    if dir.is_empty() {
        ".".into()
    } else {
        dir.into()
    }
}

pub fn stem(rel: &str) -> &str {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.rsplit_once('.').map_or(name, |(s, _)| s)
}

impl Scan {
    pub fn new(project: &Path) -> Scan {
        Scan::scoped(project, None)
    }

    /// Whether `rel` is inside the scope (everything when there is none).
    pub fn includes(&self, rel: &str) -> bool {
        self.scope.as_ref().is_none_or(|s| s.includes(rel))
    }

    /// Scans `project`, parsing only the source files inside `scope`.
    pub fn scoped(project: &Path, scope: Option<Scope>) -> Scan {
        let mut all = BTreeSet::new();
        let mut stack = vec![project.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let Ok(ft) = e.file_type() else { continue };
                let name = e.file_name().to_string_lossy().into_owned();
                if ft.is_dir() {
                    if !SKIP_DIRS.contains(&name.as_str()) {
                        stack.push(e.path());
                    }
                } else if ft.is_file()
                    && all.len() < MAX_FILES
                    && let Some(r) = dla_core::paths::rel_slash(project, &e.path())
                {
                    all.insert(r);
                }
            }
        }
        let mut infos = BTreeMap::new();
        let mut out_of_scope: BTreeMap<Runtime, u64> = BTreeMap::new();
        for rel in &all {
            let Some(lang) = Lang::from_path(Path::new(rel)) else {
                continue;
            };
            if scope.as_ref().is_some_and(|s| !s.includes(rel)) {
                if !dla_census::walk::is_test_path(rel) {
                    *out_of_scope.entry(lang.runtime()).or_default() += 1;
                }
                continue;
            }
            let p = project.join(rel);
            if fs::metadata(&p).map_or(true, |m| m.len() > MAX_FILE_BYTES) {
                continue;
            }
            let Ok(text) = fs::read_to_string(&p) else {
                continue;
            };
            infos.insert(rel.clone(), analyze(rel, lang, text));
        }
        Scan {
            root: project.to_path_buf(),
            all,
            infos,
            scope,
            out_of_scope,
        }
    }

    /// Text of a small project file.
    pub fn read(&self, rel: &str) -> Option<String> {
        let p = self.root.join(rel);
        if fs::metadata(&p).ok()?.len() > MAX_FILE_BYTES {
            return None;
        }
        fs::read_to_string(p).ok()
    }

    pub fn has(&self, rel: &str) -> bool {
        self.all.contains(rel)
    }

    /// Nearest ancestor directory of `rel` (the file's own directory first, up to the
    /// root `""`) containing one of `names`; returns the directory and the name found.
    pub fn nearest(&self, rel: &str, names: &[&str]) -> Option<(String, String)> {
        let mut dir = dir_of(rel).to_owned();
        loop {
            for n in names {
                if self.has(&join(&dir, n)) {
                    return Some((dir, (*n).to_owned()));
                }
            }
            if dir.is_empty() {
                return None;
            }
            dir = dir_of(&dir).to_owned();
        }
    }

    pub fn files_of(&self, rt: Runtime) -> impl Iterator<Item = &FileInfo> {
        self.infos.values().filter(move |f| f.lang.runtime() == rt)
    }
}

fn dirs_of(rel: &str) -> Vec<&str> {
    let mut v: Vec<&str> = rel.split('/').collect();
    v.pop();
    v
}

/// True for files that are tests or test support.
pub fn is_test_path(rel: &str, lang: Lang, text: &str) -> bool {
    let stem = stem(rel);
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let dirs = dirs_of(rel);
    let in_test_dir = dirs.iter().any(|d| {
        matches!(
            *d,
            "test" | "tests" | "__tests__" | "__mocks__" | "testing" | "spec" | "specs"
        )
    });
    match lang {
        Lang::Java | Lang::Kotlin => {
            in_test_dir
                || stem.ends_with("Test")
                || stem.ends_with("Tests")
                || stem.ends_with("IT")
                || stem.starts_with("Test")
                || text.contains("org.junit")
                || text.contains("org.testng")
        }
        Lang::Go => name.ends_with("_test.go"),
        Lang::Rust => in_test_dir || dirs.contains(&"benches"),
        Lang::Python => {
            in_test_dir
                || stem.starts_with("test_")
                || stem.ends_with("_test")
                || name == "conftest.py"
        }
        Lang::JavaScript | Lang::TypeScript | Lang::Tsx => {
            in_test_dir
                || name.contains(".test.")
                || name.contains(".spec.")
                || name.contains(".config.")
        }
        Lang::C | Lang::Cpp => {
            in_test_dir || stem.starts_with("test_") || stem.ends_with("_test") || stem == "test"
        }
    }
}

fn analyze(rel: &str, lang: Lang, text: String) -> FileInfo {
    let is_test = is_test_path(rel, lang, &text)
        || (lang == Lang::Python && matches!(stem(rel), "setup"))
        || (lang == Lang::Python && stem(rel) == "__main__");
    let mut info = FileInfo {
        rel: rel.to_owned(),
        lang,
        text,
        is_test,
        idents: BTreeSet::new(),
        test_idents: BTreeSet::new(),
        entries: Vec::new(),
    };
    let Ok(tree) = parse(lang, &info.text) else {
        return info;
    };
    let src = info.text.as_bytes();
    let root = tree.root_node();
    info.idents = idents(root, src);
    let mut ex = Extract {
        src,
        rel,
        lang,
        entries: Vec::new(),
        test_idents: BTreeSet::new(),
    };
    ex.visit(root, None, false);
    if !is_test {
        info.entries = ex.entries;
    }
    info.test_idents = ex.test_idents;
    info
}

fn text<'a>(n: Node, src: &'a [u8]) -> &'a str {
    n.utf8_text(src).unwrap_or("")
}

/// Identifiers referenced under `root`, leaving out the names of definitions.
pub fn idents(root: Node, src: &[u8]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.child_count() == 0 {
            if n.kind().ends_with("identifier") && !is_definition_name(n) {
                let t = text(n, src);
                if !t.is_empty() {
                    out.insert(t.to_owned());
                }
            }
            continue;
        }
        for i in 0..n.child_count() {
            if let Some(c) = n.child(i as u32) {
                stack.push(c);
            }
        }
    }
    out
}

fn is_definition_name(n: Node) -> bool {
    let Some(p) = n.parent() else { return false };
    let k = p.kind();
    let fnlike = (k.contains("function") || k.contains("method") || k.contains("func"))
        && (k.ends_with("declaration") || k.ends_with("definition") || k.ends_with("item"));
    if fnlike && p.child_by_field_name("name") == Some(n) {
        return true;
    }
    // C and C++: the declarator of a function definition.
    k == "function_declarator"
        && p.child_by_field_name("declarator") == Some(n)
        && p.parent()
            .is_some_and(|g| g.kind() == "function_definition")
}

struct Extract<'a> {
    src: &'a [u8],
    rel: &'a str,
    lang: Lang,
    entries: Vec<Entry>,
    test_idents: BTreeSet<String>,
}

impl Extract<'_> {
    fn push(&mut self, node: Node, owner: Option<&str>, name: &str) {
        if name.is_empty() || name == "main" {
            return;
        }
        let id = match owner {
            Some(o) if !o.is_empty() => format!("{}::{o}.{name}", self.rel),
            _ => format!("{}::{name}", self.rel),
        };
        self.entries.push(Entry {
            id,
            name: name.to_owned(),
            refs: idents(node, self.src),
        });
    }

    fn field_text(&self, n: Node, field: &str) -> String {
        n.child_by_field_name(field)
            .map(|c| text(c, self.src).to_owned())
            .unwrap_or_default()
    }

    fn first_child_text(&self, n: Node, kinds: &[&str]) -> String {
        for i in 0..n.child_count() {
            if let Some(c) = n.child(i as u32)
                && kinds.contains(&c.kind())
            {
                return text(c, self.src).to_owned();
            }
        }
        String::new()
    }

    fn name_of(&self, n: Node) -> String {
        let t = self.field_text(n, "name");
        if t.is_empty() {
            self.first_child_text(n, &["identifier", "type_identifier", "simple_identifier"])
        } else {
            t
        }
    }

    fn has_modifier(&self, n: Node, word: &str) -> bool {
        (0..n.child_count()).any(|i| {
            n.child(i as u32).is_some_and(|c| {
                c.kind() == "modifiers"
                    && text(c, self.src)
                        .split(|ch: char| !ch.is_alphanumeric())
                        .any(|w| w == word)
            })
        })
    }

    fn children<'t>(&self, n: Node<'t>) -> Vec<Node<'t>> {
        (0..n.child_count())
            .filter_map(|i| n.child(i as u32))
            .collect()
    }

    fn visit(&mut self, n: Node, owner: Option<&str>, in_test: bool) {
        match self.lang {
            Lang::Java | Lang::Kotlin => self.jvm(n, owner),
            Lang::Go => self.go(n),
            Lang::Rust => self.rust(n, owner, in_test),
            Lang::Python => self.python(n, owner),
            Lang::JavaScript | Lang::TypeScript | Lang::Tsx => self.js(n, owner),
            Lang::C | Lang::Cpp => self.c(n),
        }
    }

    fn jvm(&mut self, n: Node, owner: Option<&str>) {
        match n.kind() {
            "class_declaration"
            | "interface_declaration"
            | "enum_declaration"
            | "record_declaration"
            | "object_declaration"
            | "annotation_type_declaration" => {
                let name = self.name_of(n);
                for c in self.children(n) {
                    self.jvm(c, Some(&name));
                }
            }
            "method_declaration" | "function_declaration" => {
                if !self.has_modifier(n, "private") {
                    let name = self.name_of(n);
                    self.push(n, owner, &name);
                }
            }
            _ => {
                for c in self.children(n) {
                    self.jvm(c, owner);
                }
            }
        }
    }

    fn go(&mut self, n: Node) {
        match n.kind() {
            "function_declaration" => {
                let name = self.field_text(n, "name");
                if name.chars().next().is_some_and(char::is_uppercase) {
                    self.push(n, None, &name);
                }
            }
            "method_declaration" => {
                let name = self.field_text(n, "name");
                if name.chars().next().is_some_and(char::is_uppercase) {
                    let owner = n
                        .child_by_field_name("receiver")
                        .map(|r| {
                            let t = text(r, self.src);
                            t.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                                .rfind(|w| !w.is_empty())
                                .unwrap_or("")
                                .to_owned()
                        })
                        .unwrap_or_default();
                    self.push(n, Some(&owner), &name);
                }
            }
            "source_file" => {
                for c in self.children(n) {
                    self.go(c);
                }
            }
            _ => {}
        }
    }

    fn preceding_attrs_say_test(&self, n: Node) -> bool {
        let mut p = n.prev_sibling();
        while let Some(s) = p {
            match s.kind() {
                "attribute_item" => {
                    let t = text(s, self.src);
                    if t.contains("cfg(test)") || t.contains("#[test]") || t.contains("::test]") {
                        return true;
                    }
                }
                "line_comment" | "block_comment" => {}
                _ => return false,
            }
            p = s.prev_sibling();
        }
        false
    }

    fn rust(&mut self, n: Node, owner: Option<&str>, in_test: bool) {
        match n.kind() {
            "mod_item" | "function_item" if self.preceding_attrs_say_test(n) => {
                let ids = idents(n, self.src);
                self.test_idents.extend(ids);
            }
            "impl_item" => {
                let ty = self.field_text(n, "type");
                let ty = ty.split('<').next().unwrap_or("").trim().to_owned();
                let is_trait = n.child_by_field_name("trait").is_some();
                for c in self.children(n) {
                    if is_trait && c.kind() == "declaration_list" {
                        for m in self.children(c) {
                            if m.kind() == "function_item" && !in_test {
                                let name = self.field_text(m, "name");
                                self.push(m, Some(&ty), &name);
                            }
                        }
                    } else {
                        self.rust(c, Some(&ty), in_test);
                    }
                }
            }
            "trait_item" => {
                let name = self.field_text(n, "name");
                for c in self.children(n) {
                    self.rust(c, Some(&name), in_test);
                }
            }
            "function_item" => {
                let public = self
                    .children(n)
                    .iter()
                    .any(|c| c.kind() == "visibility_modifier");
                if public && !in_test {
                    let name = self.field_text(n, "name");
                    self.push(n, owner, &name);
                }
            }
            _ => {
                for c in self.children(n) {
                    self.rust(c, owner, in_test);
                }
            }
        }
    }

    fn python(&mut self, n: Node, owner: Option<&str>) {
        match n.kind() {
            "class_definition" => {
                let name = self.field_text(n, "name");
                for c in self.children(n) {
                    self.python(c, Some(&name));
                }
            }
            "function_definition" => {
                let name = self.field_text(n, "name");
                if !name.starts_with('_') {
                    self.push(n, owner, &name);
                }
            }
            _ => {
                for c in self.children(n) {
                    self.python(c, owner);
                }
            }
        }
    }

    fn js(&mut self, n: Node, owner: Option<&str>) {
        match n.kind() {
            "function_declaration" | "generator_function_declaration" => {
                let name = self.field_text(n, "name");
                if !name.starts_with('_') {
                    self.push(n, owner, &name);
                }
            }
            "class_declaration" | "abstract_class_declaration" => {
                let name = self.field_text(n, "name");
                for c in self.children(n) {
                    self.js(c, Some(&name));
                }
            }
            "method_definition" => {
                let name = self.field_text(n, "name");
                if name != "constructor" && !name.starts_with('_') && !name.starts_with('#') {
                    self.push(n, owner, &name);
                }
            }
            "variable_declarator" | "public_field_definition" | "field_definition" => {
                let v = n.child_by_field_name("value");
                if v.is_some_and(|v| {
                    matches!(
                        v.kind(),
                        "arrow_function" | "function_expression" | "function"
                    )
                }) {
                    let name = if n.kind() == "variable_declarator" {
                        self.field_text(n, "name")
                    } else {
                        self.field_text(n, "name")
                            .if_empty(|| self.field_text(n, "property"))
                    };
                    if !name.starts_with('_') && !name.starts_with('#') {
                        self.push(n, owner, &name);
                    }
                }
            }
            "program"
            | "export_statement"
            | "class_body"
            | "lexical_declaration"
            | "variable_declaration"
            | "internal_module"
            | "module"
            | "statement_block"
            | "ambient_declaration" => {
                for c in self.children(n) {
                    self.js(c, owner);
                }
            }
            _ => {}
        }
    }

    fn c(&mut self, n: Node) {
        match n.kind() {
            "function_definition" => {
                let is_static = self.children(n).iter().any(|c| {
                    c.kind() == "storage_class_specifier" && text(*c, self.src) == "static"
                });
                if is_static {
                    return;
                }
                let mut d = n.child_by_field_name("declarator");
                while let Some(x) = d {
                    if x.kind() == "function_declarator" {
                        break;
                    }
                    d = x.child_by_field_name("declarator");
                }
                if let Some(fd) = d {
                    let full = fd
                        .child_by_field_name("declarator")
                        .map(|c| text(c, self.src).to_owned())
                        .unwrap_or_default();
                    let (o, name) = match full.rsplit_once("::") {
                        Some((o, nm)) => (Some(o.to_owned()), nm.to_owned()),
                        None => (None, full),
                    };
                    self.push(n, o.as_deref(), &name);
                }
            }
            "compound_statement" => {}
            _ => {
                for c in self.children(n) {
                    self.c(c);
                }
            }
        }
    }
}

trait IfEmpty {
    fn if_empty(self, f: impl FnOnce() -> String) -> String;
}
impl IfEmpty for String {
    fn if_empty(self, f: impl FnOnce() -> String) -> String {
        if self.is_empty() { f() } else { self }
    }
}
