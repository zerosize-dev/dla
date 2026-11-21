//! Static callers of a function, matched by name, read from syntax trees.
//!
//! `dla harness gen --entry` shows, for each writer function of a leak finding, who calls it
//! (up to three hops, public callers first) and which expression a first-hop caller passes
//! for the parameter the stored key is built from. A call is attributed to a function by the
//! name of its callee only: no type is resolved, so two functions with one name are not told
//! apart (the block says "matched by name"). Python, Go, Java and JavaScript or TypeScript
//! are read; other runtimes have no chain.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    model::SourceFile,
    parse::{
        Lang, parse,
        tree_sitter::{Node, Point},
    },
};

/// The most hops of a chain.
pub const MAX_HOPS: usize = 3;
/// The most callers one hop lists.
pub const MAX_PER_HOP: usize = 5;
/// The most call sites a key provenance lists.
pub const MAX_SITES: usize = 5;
/// The most mentions of a function read when nothing calls it by name.
pub const MAX_REFERENCES: usize = 200;
/// The most assignments listed for a local name a call site passes.
pub const MAX_ORIGINS: usize = 3;
/// The most `if` conditions listed for a call site.
pub const MAX_GUARDS: usize = 3;
/// Deepest syntax tree level read; deeper subtrees are skipped.
const MAX_DEPTH: usize = 300;
/// Longest argument or callee text kept.
const MAX_TEXT: usize = 80;

/// True when callers are computed for `lang`.
pub fn supported(lang: Lang) -> bool {
    matches!(
        lang,
        Lang::Python | Lang::Go | Lang::Java | Lang::JavaScript | Lang::TypeScript | Lang::Tsx
    )
}

/// A function or method definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Func {
    /// Project-relative file.
    pub file: String,
    /// 1-based line of the definition.
    pub line: usize,
    /// `Class.method` or `function`; `<module>` for code outside any function.
    pub name: String,
    /// A function a user of the project can call: no leading underscore (Python, JavaScript),
    /// an upper-case first letter (Go), a `public` modifier (Java), an `export` (ES modules).
    pub public: bool,
    /// The parameter names, without the receiver (`self`, `cls`, a Go receiver).
    pub params: Vec<String>,
}

impl Func {
    /// The name a call to this function carries: the last segment of the name, or the class
    /// name for a Python `__init__` (a constructor call).
    pub fn call_name(&self) -> String {
        let mut parts = self.name.rsplit('.');
        let last = parts.next().unwrap_or(&self.name);
        if last == "__init__"
            && let Some(class) = parts.next()
        {
            return class.to_owned();
        }
        last.to_owned()
    }

    /// `file:line (name)`.
    pub fn label(&self) -> String {
        format!("{}:{} ({})", self.file, self.line, self.name)
    }
}

/// One argument of a call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    /// The keyword, for a keyword argument.
    pub keyword: Option<String>,
    /// The expression, on one line.
    pub text: String,
}

/// One call whose callee is named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    /// The function around the call.
    pub caller: Func,
    /// 1-based line of the call.
    pub line: usize,
    /// The callee as written.
    pub callee: String,
    /// The name matched against function names: the last segment of the callee.
    pub name: String,
    /// The arguments in order.
    pub args: Vec<Arg>,
    /// The conditions of the `if` statements around the call (innermost first, at most
    /// [`MAX_GUARDS`]) that the call sits in the branch of: the call happens only when they hold.
    pub guards: Vec<String>,
}

/// A function definition with its extent, for finding the function around a line.
#[derive(Debug, Clone)]
struct Def {
    func: Func,
    /// First line, decorators included.
    start: usize,
    end: usize,
}

#[derive(Debug, Default)]
struct FileFacts {
    rel: String,
    defs: Vec<Def>,
    calls: Vec<Call>,
}

/// The calls of a project, by callee name.
#[derive(Debug, Default)]
pub struct Index {
    files: Vec<FileFacts>,
    by_name: BTreeMap<String, Vec<(usize, usize)>>,
}

/// One caller in a hop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller {
    /// The calling function.
    pub func: Func,
    /// The first line in it that calls the function (or mentions it, see `by_reference`).
    pub line: usize,
    /// The function is passed, stored or named there, not called: found only for a function
    /// nothing calls by name (a callback, a class a framework instantiates).
    pub by_reference: bool,
}

/// The callers of the functions of the hop before it (the first hop: of the function itself).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hop {
    /// At most [`MAX_PER_HOP`] callers, public ones first.
    pub callers: Vec<Caller>,
    /// How many more there are.
    pub more: usize,
}

/// What a first-hop call site passes for the parameters a key is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SitePass {
    /// `file:line`.
    pub at: String,
    /// The calling function.
    pub caller: String,
    /// `(parameter, expression)` for each key parameter the call passes.
    pub passed: Vec<(String, String)>,
    /// The conditions the call happens under (see [`Call::guards`]).
    pub guards: Vec<String>,
    /// For each passed expression that is a plain local name: the assignments to it in the
    /// calling function before the call (at most [`MAX_ORIGINS`], the right-hand sides).
    pub origins: Vec<(String, Vec<String>)>,
}

/// Where the key or stored value of a writer comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// The parameters of the writer function the writing statement uses, in order.
    pub params: Vec<String>,
    /// One entry per first-hop call site (at most [`MAX_SITES`]).
    pub sites: Vec<SitePass>,
    /// Further call sites not listed.
    pub more_sites: usize,
}

// ------------------------------------------------------------------ reading a file

fn text<'a>(n: Node<'_>, src: &'a str) -> &'a str {
    src.get(n.start_byte()..n.end_byte()).unwrap_or_default()
}

fn one_line(s: &str) -> String {
    let t = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() > MAX_TEXT {
        let mut c: String = t.chars().take(MAX_TEXT - 3).collect();
        c.push_str("...");
        c
    } else {
        t
    }
}

fn field_text(n: Node<'_>, field: &str, src: &str) -> Option<String> {
    n.child_by_field_name(field).map(|c| text(c, src).to_owned())
}

fn line_of(n: Node<'_>) -> usize {
    n.start_position().row + 1
}

fn is_function_kind(lang: Lang, kind: &str) -> bool {
    match lang {
        Lang::Python => kind == "function_definition",
        Lang::Go => matches!(kind, "function_declaration" | "method_declaration"),
        Lang::Java => matches!(kind, "method_declaration" | "constructor_declaration"),
        _ => matches!(
            kind,
            "function_declaration"
                | "generator_function_declaration"
                | "method_definition"
                | "arrow_function"
                | "function_expression"
                | "function"
        ),
    }
}

fn is_class_kind(lang: Lang, kind: &str) -> bool {
    match lang {
        Lang::Python => kind == "class_definition",
        Lang::Java => matches!(
            kind,
            "class_declaration" | "interface_declaration" | "enum_declaration" | "record_declaration"
        ),
        Lang::Go => false,
        _ => matches!(kind, "class_declaration" | "class"),
    }
}

fn is_call_kind(lang: Lang, kind: &str) -> bool {
    match lang {
        Lang::Python => kind == "call",
        Lang::Go => kind == "call_expression",
        Lang::Java => matches!(kind, "method_invocation" | "object_creation_expression"),
        _ => matches!(kind, "call_expression" | "new_expression"),
    }
}

enum Scope {
    Class { name: String, public: bool },
    Func(Func),
    Anonymous,
}

struct Walker<'a> {
    lang: Lang,
    src: &'a str,
    rel: &'a str,
    /// An ES module (it has `export` statements): only exported functions are public.
    esm: bool,
    facts: FileFacts,
    scope: Vec<Scope>,
}

impl<'a> Walker<'a> {
    fn module_func(&self) -> Func {
        Func {
            file: self.rel.to_owned(),
            line: 1,
            name: "<module>".to_owned(),
            public: false,
            params: Vec::new(),
        }
    }

    fn walk(&mut self, n: Node<'_>, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        let kind = n.kind();
        if is_call_kind(self.lang, kind) {
            self.record_call(n);
        }
        let entered = if is_class_kind(self.lang, kind) {
            let name = field_text(n, "name", self.src).unwrap_or_default();
            let public = self.class_public(n, &name);
            self.scope.push(Scope::Class { name, public });
            true
        } else if is_function_kind(self.lang, kind) {
            match self.function(n) {
                Some(f) => {
                    let start = self.def_start(n);
                    self.facts.defs.push(Def {
                        func: f.clone(),
                        start,
                        end: n.end_position().row + 1,
                    });
                    self.scope.push(Scope::Func(f));
                }
                None => self.scope.push(Scope::Anonymous),
            }
            true
        } else {
            false
        };
        let mut cur = n.walk();
        for c in n.children(&mut cur) {
            self.walk(c, depth + 1);
        }
        if entered {
            self.scope.pop();
        }
    }

    /// The first line of a definition: its decorators' too (Python).
    fn def_start(&self, n: Node<'_>) -> usize {
        match n.parent() {
            Some(p) if p.kind() == "decorated_definition" => line_of(p),
            _ => line_of(n),
        }
    }

    fn class_public(&self, n: Node<'_>, name: &str) -> bool {
        match self.lang {
            Lang::Python => !name.starts_with('_'),
            Lang::Java => true,
            _ => {
                !name.starts_with('_')
                    && (!self.esm || n.parent().is_some_and(|p| p.kind() == "export_statement"))
            }
        }
    }

    fn enclosing_class(&self) -> Option<(&str, bool)> {
        match self.scope.last() {
            Some(Scope::Class { name, public }) => Some((name.as_str(), *public)),
            _ => None,
        }
    }

    fn in_function(&self) -> Option<&Func> {
        self.scope.iter().rev().find_map(|s| match s {
            Scope::Func(f) => Some(f),
            _ => None,
        })
    }

    /// The definition at `n`, or `None` for an anonymous function.
    fn function(&self, n: Node<'_>) -> Option<Func> {
        let src = self.src;
        let kind = n.kind();
        let mut name = field_text(n, "name", src);
        let mut receiver: Option<String> = None;
        if self.lang == Lang::Go && kind == "method_declaration" {
            receiver = n
                .child_by_field_name("receiver")
                .map(|r| {
                    text(r, src)
                        .trim_matches(|c| c == '(' || c == ')')
                        .rsplit(char::is_whitespace)
                        .next()
                        .unwrap_or_default()
                        .trim_start_matches('*')
                        .split('[')
                        .next()
                        .unwrap_or_default()
                        .to_owned()
                })
                .filter(|r| !r.is_empty());
        }
        if name.is_none() && matches!(self.lang, Lang::JavaScript | Lang::TypeScript | Lang::Tsx) {
            name = self.js_assigned_name(n);
        }
        let name = name.filter(|s| !s.is_empty())?;
        let outer_class = self.enclosing_class();
        let nested_in_fn = self.in_function().is_some();
        let class_name = receiver.clone().or_else(|| outer_class.map(|(c, _)| c.to_owned()));
        let full = match (&class_name, nested_in_fn && outer_class.is_none()) {
            (Some(c), false) => format!("{c}.{name}"),
            (_, true) => format!("{}.{name}", self.in_function().map_or("", |f| &f.name)),
            _ => name.clone(),
        };
        let public = !nested_in_fn
            && match self.lang {
                Lang::Python => {
                    (!name.starts_with('_') || (name.starts_with("__") && name.ends_with("__")))
                        && outer_class.is_none_or(|(_, p)| p)
                }
                Lang::Go => {
                    name.chars().next().is_some_and(char::is_uppercase)
                        && receiver
                            .as_deref()
                            .is_none_or(|r| r.chars().next().is_some_and(char::is_uppercase))
                }
                Lang::Java => self.java_public(n),
                _ => self.js_public(n, &name, outer_class),
            };
        Some(Func {
            file: self.rel.to_owned(),
            line: line_of(n),
            name: full,
            public,
            params: self.params(n),
        })
    }

    fn java_public(&self, n: Node<'_>) -> bool {
        let mut cur = n.walk();
        let has_public = n
            .children(&mut cur)
            .any(|c| c.kind() == "modifiers" && text(c, self.src).split_whitespace().any(|w| w == "public"));
        has_public
            || n.parent()
                .and_then(|p| p.parent())
                .is_some_and(|g| g.kind() == "interface_declaration")
    }

    fn js_assigned_name(&self, n: Node<'_>) -> Option<String> {
        let p = n.parent()?;
        match p.kind() {
            "variable_declarator" => field_text(p, "name", self.src),
            "pair" => field_text(p, "key", self.src),
            "assignment_expression" => {
                let left = p.child_by_field_name("left")?;
                if left.kind() == "member_expression" {
                    field_text(left, "property", self.src)
                } else {
                    Some(text(left, self.src).to_owned())
                }
            }
            "public_field_definition" | "field_definition" => {
                field_text(p, "name", self.src).or_else(|| field_text(p, "property", self.src))
            }
            _ => None,
        }
    }

    fn js_public(&self, n: Node<'_>, name: &str, class: Option<(&str, bool)>) -> bool {
        if name.starts_with('_') || name.starts_with('#') {
            return false;
        }
        if n.kind() == "method_definition" {
            let mut cur = n.walk();
            let hidden = n.children(&mut cur).any(|c| {
                c.kind() == "accessibility_modifier"
                    && matches!(text(c, self.src), "private" | "protected")
            });
            return !hidden && class.is_none_or(|(_, p)| p);
        }
        if !self.esm {
            return true;
        }
        // The function, or the declaration it is assigned in, is exported.
        let mut up = n.parent();
        for _ in 0..4 {
            let Some(p) = up else { return false };
            if p.kind() == "export_statement" {
                return true;
            }
            up = p.parent();
        }
        false
    }

    fn params(&self, n: Node<'_>) -> Vec<String> {
        let src = self.src;
        let mut out: Vec<String> = Vec::new();
        if let Some(p) = n.child_by_field_name("parameter") {
            // A single-parameter arrow function.
            out.push(text(p, src).to_owned());
        }
        let Some(list) = n.child_by_field_name("parameters") else {
            return out;
        };
        let mut cur = list.walk();
        for c in list.named_children(&mut cur) {
            match (self.lang, c.kind()) {
                (_, "identifier") => out.push(text(c, src).to_owned()),
                (Lang::Python, "typed_parameter") => {
                    let mut k = c.walk();
                    if let Some(id) = c.named_children(&mut k).find(|x| x.kind() == "identifier") {
                        out.push(text(id, src).to_owned());
                    }
                }
                (Lang::Python, "default_parameter" | "typed_default_parameter") => {
                    out.extend(field_text(c, "name", src));
                }
                (Lang::Go, "parameter_declaration" | "variadic_parameter_declaration") => {
                    let mut k = c.walk();
                    for nm in c.children_by_field_name("name", &mut k) {
                        out.push(text(nm, src).to_owned());
                    }
                }
                (Lang::Java, "formal_parameter") => out.extend(field_text(c, "name", src)),
                (Lang::Java, "spread_parameter") => {
                    let mut k = c.walk();
                    if let Some(d) = c
                        .named_children(&mut k)
                        .find(|x| x.kind() == "variable_declarator")
                    {
                        out.extend(field_text(d, "name", src));
                    }
                }
                (_, "required_parameter" | "optional_parameter") => {
                    out.extend(field_text(c, "pattern", src).filter(|p| !p.starts_with(['{', '['])));
                }
                (_, "assignment_pattern") => {
                    out.extend(field_text(c, "left", src).filter(|p| !p.starts_with(['{', '['])));
                }
                _ => {}
            }
        }
        out.retain(|p| !(self.lang == Lang::Python && (p == "self" || p == "cls")));
        out
    }

    fn record_call(&mut self, n: Node<'_>) {
        let src = self.src;
        let (callee_node, args_node) = match (self.lang, n.kind()) {
            (Lang::Java, "method_invocation") => (n.child_by_field_name("name"), n.child_by_field_name("arguments")),
            (Lang::Java, _) => (n.child_by_field_name("type"), n.child_by_field_name("arguments")),
            (_, "new_expression") => (n.child_by_field_name("constructor"), n.child_by_field_name("arguments")),
            _ => (n.child_by_field_name("function"), n.child_by_field_name("arguments")),
        };
        let (Some(callee_node), Some(args_node)) = (callee_node, args_node) else {
            return;
        };
        let Some(name) = self.callee_name(callee_node) else {
            return;
        };
        let mut args = Vec::new();
        let mut cur = args_node.walk();
        for a in args_node.named_children(&mut cur) {
            if a.kind() == "comment" {
                continue;
            }
            if self.lang == Lang::Python && a.kind() == "keyword_argument" {
                args.push(Arg {
                    keyword: field_text(a, "name", src),
                    text: one_line(&field_text(a, "value", src).unwrap_or_default()),
                });
            } else {
                args.push(Arg {
                    keyword: None,
                    text: one_line(text(a, src)),
                });
            }
        }
        let caller = self
            .in_function()
            .cloned()
            .unwrap_or_else(|| self.module_func());
        let callee = if self.lang == Lang::Java && n.kind() == "method_invocation" {
            match n.child_by_field_name("object") {
                Some(o) => format!("{}.{name}", one_line(text(o, src))),
                None => name.clone(),
            }
        } else {
            one_line(text(callee_node, src))
        };
        self.facts.calls.push(Call {
            caller,
            line: line_of(n),
            callee,
            name,
            args,
            guards: guards_of(n, src),
        });
    }

    fn callee_name(&self, n: Node<'_>) -> Option<String> {
        let src = self.src;
        let last = |s: &str| {
            let s = s.split('<').next().unwrap_or(s).trim();
            s.rsplit(['.', ':']).next().unwrap_or(s).to_owned()
        };
        let name = match n.kind() {
            "identifier" | "type_identifier" | "property_identifier" | "field_identifier" => {
                text(n, src).to_owned()
            }
            "attribute" => field_text(n, "attribute", src)?,
            "selector_expression" => field_text(n, "field", src)?,
            "member_expression" => field_text(n, "property", src)?,
            "scoped_type_identifier" | "generic_type" => last(text(n, src)),
            "index_expression" => return self.callee_name(n.child_by_field_name("operand")?),
            _ => return None,
        };
        (!name.is_empty()).then_some(name)
    }
}

/// The conditions of the `if` statements that `n` sits in the branch of, innermost first, up to
/// the function around it: `if self._run_middlewares:` around a call says the call happens
/// only when the application has middlewares.
fn guards_of(n: Node<'_>, src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut child = n;
    let mut up = n.parent();
    while let Some(p) = up {
        if p.kind().contains("function") || p.kind().contains("method") || p.kind() == "lambda" {
            break;
        }
        if matches!(p.kind(), "if_statement" | "elif_clause" | "if_expression")
            && let (Some(cond), Some(body)) =
                (p.child_by_field_name("condition"), p.child_by_field_name("consequence"))
            && body.id() == child.id()
        {
            let text = one_line(unparen(text(cond, src)));
            if !text.is_empty() && out.len() < MAX_GUARDS {
                out.push(text);
            }
        }
        child = p;
        up = p.parent();
    }
    out
}

/// `(a and b)` without its outer parentheses (JavaScript, Java and Go wrap an `if` condition).
fn unparen(t: &str) -> &str {
    let t = t.trim();
    if t.starts_with('(') && t.ends_with(')') {
        let mut depth = 0usize;
        for (i, c) in t.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && i + 1 < t.len() {
                        return t;
                    }
                }
                _ => {}
            }
        }
        return &t[1..t.len() - 1];
    }
    t
}

fn read_file(f: &SourceFile) -> FileFacts {
    let mut w = Walker {
        lang: f.lang,
        src: &f.text,
        rel: &f.rel,
        esm: matches!(f.lang, Lang::JavaScript | Lang::TypeScript | Lang::Tsx)
            && f.text.lines().any(|l| l.starts_with("export ")),
        facts: FileFacts {
            rel: f.rel.clone(),
            ..FileFacts::default()
        },
        scope: Vec::new(),
    };
    if let Ok(tree) = parse(f.lang, &f.text) {
        w.walk(tree.root_node(), 0);
    }
    w.facts
}

// ------------------------------------------------------------------ the index

impl Index {
    /// The functions that mention `f` by name without calling it, nor defining it: a callback
    /// handed to a framework, a class stored as a default. Lines that import, comment or define
    /// are left out; at most [`MAX_REFERENCES`] are read.
    fn references(&self, files: &[SourceFile], f: &Func) -> Vec<(Func, usize)> {
        let name = f.call_name();
        if name.chars().count() < 3 {
            return Vec::new();
        }
        let Ok(re) = regex::Regex::new(&format!(r"\b{}\b", regex::escape(&name))) else {
            return Vec::new();
        };
        let mut out: Vec<(Func, usize)> = Vec::new();
        for file in files.iter().filter(|x| !x.is_test && supported(x.lang)) {
            for (n, line) in file.text.lines().enumerate() {
                let n = n + 1;
                let t = line.trim_start();
                if t.starts_with(['#', '*'])
                    || t.starts_with("//")
                    || t.starts_with("import ")
                    || t.starts_with("from ")
                    || t.starts_with("package ")
                    || t.starts_with("use ")
                {
                    continue;
                }
                for m in re.find_iter(line) {
                    let before = line[..m.start()].trim_end();
                    let after = line[m.end()..].trim_start();
                    let defines = ["def", "func", "function", "class", "fn", "async", "void"]
                        .iter()
                        .any(|k| before.ends_with(k))
                        || before.ends_with(')') && after.starts_with('{');
                    if after.starts_with('(') || defines {
                        continue;
                    }
                    let func = self
                        .function_at(&file.rel, n)
                        .unwrap_or_else(|| Func {
                            file: file.rel.clone(),
                            line: 1,
                            name: "<module>".to_owned(),
                            public: false,
                            params: Vec::new(),
                        });
                    // The function itself, and a function of the same name.
                    if func.file == f.file && func.name == f.name {
                        continue;
                    }
                    out.push((func, n));
                    break;
                }
                if out.len() >= MAX_REFERENCES {
                    return out;
                }
            }
        }
        out
    }

    /// The assignments to each plain local name `passed` holds, inside the caller of `call` and
    /// before the call.
    fn origins(
        &self,
        files: &[SourceFile],
        call: &Call,
        passed: &[(String, String)],
    ) -> Vec<(String, Vec<String>)> {
        let Some(file) = files.iter().find(|f| f.rel == call.caller.file) else {
            return Vec::new();
        };
        let Some(def) = self
            .files
            .iter()
            .find(|f| f.rel == call.caller.file)
            .and_then(|f| {
                f.defs
                    .iter()
                    .find(|d| d.func.name == call.caller.name && d.func.line == call.caller.line)
            })
        else {
            return Vec::new();
        };
        let mut out: Vec<(String, Vec<String>)> = Vec::new();
        for (_, expr) in passed {
            let is_name = !expr.is_empty()
                && expr.chars().all(|c| c.is_alphanumeric() || c == '_')
                && !expr.chars().next().is_some_and(|c| c.is_ascii_digit())
                && !matches!(expr.as_str(), "None" | "True" | "False" | "nil" | "null" | "self" | "this");
            if !is_name || out.iter().any(|(n, _)| n == expr) {
                continue;
            }
            let Ok(re) = regex::Regex::new(&format!(
                r"(?:^|[^.\w]){}\s*(?::=|=)([^=].*)$",
                regex::escape(expr)
            )) else {
                continue;
            };
            let mut rhs: Vec<String> = Vec::new();
            for (n, line) in file.text.lines().enumerate() {
                let n = n + 1;
                if n < def.start || n > def.end.min(call.line) {
                    continue;
                }
                if let Some(m) = re.captures(line) {
                    let t = one_line(m[1].trim().trim_end_matches(';'));
                    if !t.is_empty() && !rhs.contains(&t) {
                        rhs.push(t);
                    }
                }
            }
            let from = rhs.len().saturating_sub(MAX_ORIGINS);
            if !rhs.is_empty() {
                out.push((expr.clone(), rhs.split_off(from)));
            }
        }
        out
    }

    /// Reads the calls of every non-test file of a supported language.
    pub fn build(files: &[SourceFile]) -> Self {
        let mut ix = Self::default();
        for f in files.iter().filter(|f| !f.is_test && supported(f.lang)) {
            let facts = read_file(f);
            let fi = ix.files.len();
            for (ci, c) in facts.calls.iter().enumerate() {
                let name = c.name.clone();
                ix.by_name.entry(name).or_default().push((fi, ci));
            }
            ix.files.push(facts);
        }
        ix
    }

    /// Number of files read.
    pub fn files(&self) -> usize {
        self.files.len()
    }

    /// The innermost function whose extent holds `line` of `file`. A Python decorator line
    /// belongs to the function it decorates.
    pub fn function_at(&self, file: &str, line: usize) -> Option<Func> {
        let facts = self.files.iter().find(|f| f.rel == file)?;
        facts
            .defs
            .iter()
            .filter(|d| d.start <= line && line <= d.end)
            .max_by_key(|d| (d.start, std::cmp::Reverse(d.end)))
            .map(|d| d.func.clone())
    }

    /// Every call whose callee is named `name`, public callers first, then by file and line.
    pub fn calls_of(&self, name: &str) -> Vec<&Call> {
        let mut v: Vec<&Call> = self
            .by_name
            .get(name)
            .into_iter()
            .flatten()
            .map(|(f, c)| &self.files[*f].calls[*c])
            .collect();
        v.sort_by(|a, b| {
            (!a.caller.public, &a.caller.file, a.line).cmp(&(!b.caller.public, &b.caller.file, b.line))
        });
        v
    }

    fn hop(
        &self,
        files: &[SourceFile],
        of: &[Func],
        seen: &mut BTreeSet<(String, String)>,
    ) -> Hop {
        let mut by_caller: Vec<Caller> = Vec::new();
        for f in of {
            let before = by_caller.len();
            for c in self.calls_of(&f.call_name()) {
                let key = (c.caller.file.clone(), c.caller.name.clone());
                if seen.contains(&key) || by_caller.iter().any(|x| x.func == c.caller) {
                    continue;
                }
                by_caller.push(Caller {
                    func: c.caller.clone(),
                    line: c.line,
                    by_reference: false,
                });
            }
            // Nothing calls it by name: where is it passed or stored?
            if by_caller.len() == before {
                for (func, line) in self.references(files, f) {
                    let key = (func.file.clone(), func.name.clone());
                    if seen.contains(&key) || by_caller.iter().any(|x| x.func == func) {
                        continue;
                    }
                    by_caller.push(Caller {
                        func,
                        line,
                        by_reference: true,
                    });
                }
            }
        }
        by_caller.sort_by(|a, b| {
            (!a.func.public, a.by_reference, &a.func.file, a.line)
                .cmp(&(!b.func.public, b.by_reference, &b.func.file, b.line))
        });
        let more = by_caller.len().saturating_sub(MAX_PER_HOP);
        by_caller.truncate(MAX_PER_HOP);
        for c in &by_caller {
            seen.insert((c.func.file.clone(), c.func.name.clone()));
        }
        Hop {
            callers: by_caller,
            more,
        }
    }

    /// The callers of `func`, then of those, up to [`MAX_HOPS`] hops; it ends at the first hop
    /// with no caller. A function is listed once in all. A function nothing calls by name is
    /// followed by the functions that mention it without calling it.
    ///
    pub fn chain(&self, files: &[SourceFile], func: &Func) -> Vec<Hop> {
        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        seen.insert((func.file.clone(), func.name.clone()));
        let mut hops = Vec::new();
        let mut of = vec![func.clone()];
        for _ in 0..MAX_HOPS {
            let hop = self.hop(files, &of, &mut seen);
            if hop.callers.is_empty() {
                break;
            }
            of = hop.callers.iter().map(|c| c.func.clone()).collect();
            hops.push(hop);
        }
        hops
    }

    /// The parameters of `func` that the statement at `line` of its body uses, and what each
    /// first-hop call site passes for them. With `all_params`, every parameter counts (a
    /// cached function is keyed by all of its arguments).
    pub fn provenance(
        &self,
        files: &[SourceFile],
        func: &Func,
        line: Option<usize>,
        all_params: bool,
    ) -> Provenance {
        let used: Vec<String> = if all_params {
            func.params.clone()
        } else {
            line.and_then(|l| {
                let f = files.iter().find(|f| f.rel == func.file)?;
                Some(statement_identifiers(f, l))
            })
            .map(|ids| func.params.iter().filter(|p| ids.contains(*p)).cloned().collect())
            .unwrap_or_default()
        };
        let mut sites = Vec::new();
        let calls: Vec<&Call> = self
            .calls_of(&func.call_name())
            .into_iter()
            .filter(|c| !(c.caller.file == func.file && c.caller.name == func.name))
            .collect();
        for c in calls.iter().take(MAX_SITES) {
            let passed: Vec<(String, String)> = used
                .iter()
                .filter_map(|p| {
                    let idx = func.params.iter().position(|x| x == p)?;
                    let by_kw = c.args.iter().find(|a| a.keyword.as_deref() == Some(p));
                    let positional = c.args.iter().filter(|a| a.keyword.is_none()).nth(idx);
                    let arg = by_kw.or(positional)?;
                    Some((p.clone(), arg.text.clone()))
                })
                .collect();
            let origins = self.origins(files, c, &passed);
            sites.push(SitePass {
                at: format!("{}:{}", c.caller.file, c.line),
                caller: c.caller.name.clone(),
                passed,
                guards: c.guards.clone(),
                origins,
            });
        }
        Provenance {
            params: used,
            sites,
            more_sites: calls.len().saturating_sub(MAX_SITES),
        }
    }
}

/// The identifiers a statement uses as values, not as the name of an attribute or a method:
/// the statement that holds `line` (a statement, an assignment or a declaration).
pub fn statement_identifiers(f: &SourceFile, line: usize) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let Ok(tree) = parse(f.lang, &f.text) else {
        return out;
    };
    let Some(row) = line.checked_sub(1) else {
        return out;
    };
    let Some(text_line) = f.text.lines().nth(row) else {
        return out;
    };
    let indent = text_line.len() - text_line.trim_start().len();
    let mut node = tree.root_node().descendant_for_point_range(
        Point { row, column: indent },
        Point {
            row,
            column: text_line.trim_end().len().max(indent),
        },
    );
    let mut stmt = None;
    while let Some(n) = node {
        let k = n.kind();
        if is_function_kind(f.lang, k) || is_class_kind(f.lang, k) || k == "module" || k == "program" {
            break;
        }
        if k.ends_with("_statement")
            || k.ends_with("_declaration")
            || k == "assignment"
            || k == "augmented_assignment"
        {
            stmt = Some(n);
            break;
        }
        node = n.parent();
    }
    if let Some(s) = stmt {
        collect_ids(s, &f.text, &mut out, 0);
    }
    out
}

fn collect_ids(n: Node<'_>, src: &str, out: &mut BTreeSet<String>, depth: usize) {
    if depth > MAX_DEPTH || out.len() > 400 {
        return;
    }
    match n.kind() {
        "identifier" => {
            out.insert(text(n, src).to_owned());
            return;
        }
        // The attribute, field or method name is not a value; the object is.
        "attribute" | "field_access" | "member_expression" => {
            if let Some(o) = n.child_by_field_name("object") {
                collect_ids(o, src, out, depth + 1);
            }
            return;
        }
        "selector_expression" => {
            if let Some(o) = n.child_by_field_name("operand") {
                collect_ids(o, src, out, depth + 1);
            }
            return;
        }
        _ => {}
    }
    let mut cur = n.walk();
    for c in n.children(&mut cur) {
        if n.kind() == "method_invocation" && n.child_by_field_name("name") == Some(c) {
            continue;
        }
        collect_ids(c, src, out, depth + 1);
    }
}

#[cfg(test)]
#[path = "callers_tests.rs"]
mod tests;
