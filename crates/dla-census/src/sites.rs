//! What the syntax tree says of each holder and of each writer and remover site.
//!
//! [`crate::usage`] finds, by name, the lines that may write to or remove from a holder. This
//! pass reads each of those lines in its syntax tree (Go, Java, Python, JavaScript and
//! TypeScript; other languages keep the name matches and get no facts):
//!
//! - the enclosing function, its parameters, whether it is an initialiser, whether callers
//!   up to three calls up are initialisers only, and whether it is part of the public surface;
//! - what the statement does to the holder (see [`crate::events`]): an insert, a reassignment,
//!   a reset or a delete, on the holder or on an element of it, and whether the inserted key
//!   or value derives from a parameter of the function or is a constant;
//! - whether the site can name the holder at all: a site in another file that only matches
//!   the name of a holder that is private to its file, class or package is dropped, and so is
//!   a site in a method of another type that names a same-named field;
//! - the holder: a real container, registry-like or not a container; bounded, weak or an
//!   unbounded cache decorator; process, object or unit lifetime; private; a literal table;
//!   a pool; and how the instances of its owner type are created.
//!
//! The text of a statement is read with comments blanked and string literals reduced to
//! `"sss"` (the tree finds them), so a name in a string or a comment is never a site.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use dla_core::census::{
    Bound, ContainerClass, HolderFacts, Lifetime, OwnerCreation, Receiver, SiteFact, SiteKind,
};
use regex::Regex;

use crate::{
    ast::{self, ancestors, text, visit},
    events::{Ev, events, idents},
    model::{Raw, SourceFile},
    parse::{Lang, parse, tree_sitter::Node},
    pyimports::{Bind, Imports},
    usage::Usage,
};

/// Calls up the call graph a start-up-only function is looked for.
const STARTUP_HOPS: u8 = 3;
/// A function with more callers than this is not chased.
const MAX_CALLERS: usize = 150;
/// Construction sites of an owner type that are read.
const MAX_CREATIONS: usize = 200;

/// The language families the analysis reads.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Fam {
    Go,
    Java,
    Python,
    Js,
}

fn fam_of(l: Lang) -> Option<Fam> {
    match l {
        Lang::Go => Some(Fam::Go),
        Lang::Java => Some(Fam::Java),
        Lang::Python => Some(Fam::Python),
        Lang::JavaScript | Lang::TypeScript | Lang::Tsx => Some(Fam::Js),
        _ => None,
    }
}

/// One function, method, constructor, initialiser block or function literal.
#[derive(Debug, Clone, Default)]
struct Frame {
    name: Option<String>,
    start: u32,
    end: u32,
    params: Vec<String>,
    /// Go: parameter names with the base name of their types.
    typed: Vec<(String, String)>,
    /// Go: the receiver variable and the base name of its type.
    recv: Option<String>,
    recv_type: Option<String>,
    class: Option<String>,
    /// Java: `public` or `protected` (or a member of an interface), and every class around
    /// it is as well.
    public: bool,
    ctor: bool,
    clinit: bool,
    lit: bool,
    /// A literal handed to an eager iteration: it runs where its caller runs.
    sync_cb: bool,
    private_class: bool,
}

#[derive(Debug, Clone)]
struct Call {
    name: String,
    qual: Option<String>,
    line: u32,
}

/// One parsed file: blanked code lines, frames, calls and constructions.
struct FileCtx {
    rel: String,
    fam: Fam,
    code: Vec<String>,
    frames: Vec<Frame>,
    calls: Vec<Call>,
    /// `(type name, line)` of `T{..}`, `new T(..)`, `T(..)`.
    creations: Vec<(String, u32)>,
    /// Python: the names the file's import statements bind.
    imports: Imports,
}

impl FileCtx {
    /// Frames that contain `line`, innermost first.
    fn chain(&self, line: u32) -> Vec<&Frame> {
        let mut v: Vec<&Frame> = self
            .frames
            .iter()
            .filter(|f| f.start <= line && line <= f.end)
            .collect();
        v.sort_by_key(|f| f.end - f.start);
        v
    }

    fn dir(&self) -> &str {
        self.rel.rsplit_once('/').map_or("", |(d, _)| d)
    }

    fn line(&self, n: u32) -> &str {
        n.checked_sub(1)
            .and_then(|i| self.code.get(i as usize))
            .map_or("", String::as_str)
    }

    /// The statement that starts on `n`: the line, and the lines after it while brackets stay
    /// open.
    fn statement(&self, n: u32) -> String {
        let mut out = self.line(n).to_owned();
        let open = |s: &str| {
            s.bytes()
                .map(|b| match b {
                    b'(' | b'[' | b'{' => 1i32,
                    b')' | b']' | b'}' => -1,
                    _ => 0,
                })
                .sum::<i32>()
        };
        let mut depth = open(&out);
        let mut k = n;
        while depth > 0 && k < n + 8 {
            k += 1;
            let l = self.line(k);
            out.push(' ');
            out.push_str(l.trim());
            depth += open(l);
        }
        out
    }
}

fn blank(src: &str, root: Node<'_>, lang: Lang) -> Vec<String> {
    let mut bytes = src.as_bytes().to_vec();
    let mut skip_to = 0usize;
    visit(root, &mut |n| {
        if n.start_byte() < skip_to {
            return;
        }
        let k = n.kind();
        let comment = k.contains("comment");
        let string = matches!(
            k,
            "interpreted_string_literal"
                | "raw_string_literal"
                | "string_literal"
                | "string"
                | "template_string"
                | "character_literal"
                | "rune_literal"
                | "text_block"
        ) || (lang == Lang::Python && k == "concatenated_string");
        if !comment && !string {
            return;
        }
        let (a, b) = (n.start_byte(), n.end_byte().min(bytes.len()));
        for (i, x) in bytes.iter_mut().enumerate().take(b).skip(a) {
            if *x == b'\n' {
                continue;
            }
            let edge = !comment && (i == a || i + 1 == b);
            *x = if comment {
                b' '
            } else if edge {
                if *x == b'`' || *x == b'\'' || *x == b'"' {
                    *x
                } else {
                    b'"'
                }
            } else {
                b's'
            };
        }
        skip_to = b;
    });
    String::from_utf8_lossy(&bytes)
        .lines()
        .map(str::to_owned)
        .collect()
}

fn line_of(n: Node<'_>) -> u32 {
    ast::line(n)
}

fn base_type(t: &str) -> String {
    let t = t
        .trim()
        .trim_start_matches(['*', '&'])
        .trim_start_matches("[]");
    let t = t.trim_start_matches('*').trim_start_matches("...");
    let t = t.rsplit('.').next().unwrap_or(t);
    t.split('[').next().unwrap_or(t).trim().to_owned()
}

fn ident_names(n: Node<'_>, src: &str, kinds: &[&str], out: &mut Vec<String>) {
    visit(n, &mut |c| {
        if kinds.contains(&c.kind()) {
            out.push(text(c, src).to_owned());
        }
    });
}

fn go_frame(n: Node<'_>, src: &str) -> Option<Frame> {
    let lit = n.kind() == "func_literal";
    let mut f = Frame {
        name: n
            .child_by_field_name("name")
            .map(|c| text(c, src).to_owned()),
        start: line_of(n),
        end: ast::end_line(n),
        lit,
        ..Frame::default()
    };
    if let Some(params) = n.child_by_field_name("parameters") {
        let mut c = params.walk();
        for p in params.named_children(&mut c) {
            let ty = p
                .child_by_field_name("type")
                .map(|t| base_type(text(t, src)))
                .unwrap_or_default();
            let mut c2 = p.walk();
            for nm in p.children_by_field_name("name", &mut c2) {
                let nm = text(nm, src).to_owned();
                f.typed.push((nm.clone(), ty.clone()));
                f.params.push(nm);
            }
        }
    }
    if let Some(r) = n.child_by_field_name("receiver") {
        let mut c = r.walk();
        if let Some(p) = r.named_children(&mut c).next() {
            f.recv = p
                .child_by_field_name("name")
                .map(|t| text(t, src).to_owned());
            f.recv_type = p
                .child_by_field_name("type")
                .map(|t| base_type(text(t, src)));
        }
    }
    Some(f)
}

fn java_modifiers<'a>(n: Node<'_>, src: &'a str) -> &'a str {
    let mut c = n.walk();
    n.named_children(&mut c)
        .find(|m| m.kind() == "modifiers")
        .map_or("", |m| text(m, src))
}

fn java_frame(n: Node<'_>, src: &str) -> Option<Frame> {
    let mut f = Frame {
        start: line_of(n),
        end: ast::end_line(n),
        ..Frame::default()
    };
    match n.kind() {
        "static_initializer" => {
            f.clinit = true;
            f.ctor = true;
            f.name = Some("<clinit>".into());
        }
        "lambda_expression" => {
            f.lit = true;
            if let Some(p) = n.child_by_field_name("parameters") {
                if p.kind() == "identifier" {
                    f.params.push(text(p, src).to_owned());
                } else {
                    ident_names(p, src, &["identifier"], &mut f.params);
                }
            }
        }
        _ => {
            f.name = n
                .child_by_field_name("name")
                .map(|c| text(c, src).to_owned());
            f.ctor = n.kind() == "constructor_declaration";
            if let Some(p) = n.child_by_field_name("parameters") {
                let mut c = p.walk();
                for q in p.named_children(&mut c) {
                    if let Some(nm) = q.child_by_field_name("name") {
                        f.params.push(text(nm, src).to_owned());
                    } else {
                        let mut v = Vec::new();
                        ident_names(q, src, &["identifier"], &mut v);
                        if let Some(last) = v.pop() {
                            f.params.push(last);
                        }
                    }
                }
            }
        }
    }
    let mods = java_modifiers(n, src);
    let in_interface = n
        .parent()
        .and_then(|p| p.parent())
        .is_some_and(|c| c.kind() == "interface_declaration");
    f.public = mods.contains("public")
        || mods.contains("protected")
        || (in_interface && !mods.contains("private"));
    // The classes around it.
    let mut all_public = true;
    for a in ancestors(n).filter(|a| {
        matches!(
            a.kind(),
            "class_declaration"
                | "interface_declaration"
                | "enum_declaration"
                | "record_declaration"
        )
    }) {
        if f.class.is_none() {
            f.class = a
                .child_by_field_name("name")
                .map(|c| text(c, src).to_owned());
        }
        let m = java_modifiers(a, src);
        let nested_in_interface = a
            .parent()
            .and_then(|p| p.parent())
            .is_some_and(|c| c.kind() == "interface_declaration");
        if !(m.contains("public") || m.contains("protected") || nested_in_interface) {
            all_public = false;
        }
    }
    f.private_class = !all_public;
    Some(f)
}

fn py_params(p: Node<'_>, src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut c = p.walk();
    for q in p.named_children(&mut c) {
        match q.kind() {
            "identifier" => out.push(text(q, src).to_owned()),
            "default_parameter" | "typed_default_parameter" => {
                if let Some(n) = q.child_by_field_name("name") {
                    out.push(text(n, src).to_owned());
                }
            }
            _ => {
                let mut v = Vec::new();
                ident_names(q, src, &["identifier"], &mut v);
                if let Some(first) = v.into_iter().next() {
                    out.push(first);
                }
            }
        }
    }
    out
}

fn py_frame(n: Node<'_>, src: &str) -> Option<Frame> {
    let lit = n.kind() == "lambda";
    let name = n
        .child_by_field_name("name")
        .map(|c| text(c, src).to_owned());
    let mut f = Frame {
        start: line_of(n),
        end: ast::end_line(n),
        lit,
        ctor: name.as_deref().is_some_and(|x| {
            matches!(
                x,
                "__init__" | "__new__" | "__post_init__" | "__init_subclass__"
            )
        }),
        name,
        ..Frame::default()
    };
    if let Some(p) = n.child_by_field_name("parameters") {
        f.params = py_params(p, src);
    }
    if let Some(c) = ancestors(n).find(|a| a.kind() == "class_definition") {
        f.class = c
            .child_by_field_name("name")
            .map(|x| text(x, src).to_owned());
        f.private_class = f.class.as_deref().is_some_and(|c| c.starts_with('_'));
    }
    Some(f)
}

fn js_params(p: Node<'_>, src: &str) -> Vec<String> {
    let mut out = Vec::new();
    if p.kind() == "identifier" {
        out.push(text(p, src).to_owned());
        return out;
    }
    let mut c = p.walk();
    for q in p.named_children(&mut c) {
        match q.kind() {
            "assignment_pattern" => {
                if let Some(l) = q.child_by_field_name("left") {
                    ident_names(
                        l,
                        src,
                        &["identifier", "shorthand_property_identifier_pattern"],
                        &mut out,
                    );
                }
            }
            "required_parameter" | "optional_parameter" => {
                if let Some(l) = q.child_by_field_name("pattern") {
                    ident_names(
                        l,
                        src,
                        &["identifier", "shorthand_property_identifier_pattern"],
                        &mut out,
                    );
                }
            }
            _ => ident_names(
                q,
                src,
                &["identifier", "shorthand_property_identifier_pattern"],
                &mut out,
            ),
        }
    }
    out
}

fn js_frame(n: Node<'_>, src: &str) -> Option<Frame> {
    let k = n.kind();
    let mut name = n
        .child_by_field_name("name")
        .map(|c| text(c, src).to_owned());
    let anonymous = matches!(k, "arrow_function" | "function_expression" | "function");
    if anonymous && name.is_none() {
        name = n.parent().and_then(|p| match p.kind() {
            "variable_declarator" => p
                .child_by_field_name("name")
                .map(|c| text(c, src).to_owned()),
            "pair" => p
                .child_by_field_name("key")
                .map(|c| text(c, src).to_owned()),
            "assignment_expression" => p
                .child_by_field_name("left")
                .and_then(|c| ast::last_word(text(c, src)))
                .map(str::to_owned),
            "public_field_definition" | "field_definition" => p
                .child_by_field_name("name")
                .or_else(|| p.child_by_field_name("property"))
                .map(|c| text(c, src).to_owned()),
            _ => None,
        });
    }
    let mut f = Frame {
        start: line_of(n),
        end: ast::end_line(n),
        lit: anonymous && name.is_none(),
        ctor: k == "method_definition" && name.as_deref() == Some("constructor"),
        name,
        ..Frame::default()
    };
    if let Some(p) = n
        .child_by_field_name("parameters")
        .or_else(|| n.child_by_field_name("parameter"))
    {
        f.params = js_params(p, src);
    }
    if let Some(c) = ancestors(n).find(|a| matches!(a.kind(), "class_declaration" | "class")) {
        f.class = c
            .child_by_field_name("name")
            .map(|x| text(x, src).to_owned());
    }
    Some(f)
}

fn callee_parts(f: Node<'_>, src: &str, fam: Fam) -> Option<(String, Option<String>)> {
    match (fam, f.kind()) {
        (_, "identifier") => Some((text(f, src).to_owned(), None)),
        (Fam::Go, "selector_expression") => Some((
            text(f.child_by_field_name("field")?, src).to_owned(),
            f.child_by_field_name("operand")
                .map(|o| text(o, src).to_owned()),
        )),
        (Fam::Python, "attribute") => Some((
            text(f.child_by_field_name("attribute")?, src).to_owned(),
            f.child_by_field_name("object")
                .map(|o| text(o, src).to_owned()),
        )),
        (Fam::Js, "member_expression") => Some((
            text(f.child_by_field_name("property")?, src).to_owned(),
            f.child_by_field_name("object")
                .map(|o| text(o, src).to_owned()),
        )),
        _ => None,
    }
}

fn build_file(sf: &SourceFile) -> Option<FileCtx> {
    let fam = fam_of(sf.lang)?;
    let tree = parse(sf.lang, &sf.text).ok()?;
    let root = tree.root_node();
    let src = sf.text.as_str();
    let code = blank(src, root, sf.lang);
    let mut frames = Vec::new();
    let mut calls = Vec::new();
    let mut creations = Vec::new();
    let mut imports = Imports::default();
    visit(root, &mut |n| {
        let k = n.kind();
        if fam == Fam::Python && matches!(k, "import_statement" | "import_from_statement") {
            imports.add_statement(text(n, src));
        }
        let frame = match fam {
            Fam::Go
                if matches!(
                    k,
                    "function_declaration" | "method_declaration" | "func_literal"
                ) =>
            {
                go_frame(n, src)
            }
            Fam::Java
                if matches!(
                    k,
                    "method_declaration"
                        | "constructor_declaration"
                        | "static_initializer"
                        | "lambda_expression"
                ) =>
            {
                java_frame(n, src)
            }
            Fam::Python if matches!(k, "function_definition" | "lambda") => py_frame(n, src),
            Fam::Js
                if matches!(
                    k,
                    "function_declaration"
                        | "generator_function_declaration"
                        | "method_definition"
                        | "arrow_function"
                        | "function_expression"
                        | "function"
                ) =>
            {
                js_frame(n, src)
            }
            _ => None,
        };
        if let Some(mut f) = frame {
            if f.lit {
                let l = code.get(f.start as usize - 1).map_or("", String::as_str);
                static CB: OnceLock<Regex> = OnceLock::new();
                let cb = CB.get_or_init(|| {
                    Regex::new(r"\.(?:forEach|map|filter|reduce|some|every|stream)\s*\(")
                        .expect("static")
                });
                f.sync_cb = cb.is_match(l);
            }
            frames.push(f);
        }
        match (fam, k) {
            (Fam::Go, "call_expression") | (Fam::Python, "call") | (Fam::Js, "call_expression") => {
                if let Some((name, qual)) = n
                    .child_by_field_name("function")
                    .and_then(|f| callee_parts(f, src, fam))
                {
                    if fam == Fam::Python && name.chars().next().is_some_and(char::is_uppercase) {
                        creations.push((name.clone(), line_of(n)));
                    }
                    if fam == Fam::Go
                        && name == "new"
                        && let Some(a) = n.child_by_field_name("arguments")
                    {
                        creations
                            .push((base_type(text(a, src).trim_matches(['(', ')'])), line_of(n)));
                    }
                    calls.push(Call {
                        name,
                        qual,
                        line: line_of(n),
                    });
                }
            }
            (Fam::Java, "method_invocation") => {
                if let Some(name) = n.child_by_field_name("name") {
                    calls.push(Call {
                        name: text(name, src).to_owned(),
                        qual: n
                            .child_by_field_name("object")
                            .map(|o| text(o, src).to_owned()),
                        line: line_of(n),
                    });
                }
            }
            (Fam::Java, "object_creation_expression") | (Fam::Js, "new_expression") => {
                let ty = n
                    .child_by_field_name("type")
                    .or_else(|| n.child_by_field_name("constructor"));
                if let Some(t) = ty {
                    let name = base_type(text(t, src));
                    creations.push((name.clone(), line_of(n)));
                    calls.push(Call {
                        name,
                        qual: None,
                        line: line_of(n),
                    });
                }
            }
            (Fam::Go, "composite_literal") => {
                if let Some(t) = n.child_by_field_name("type") {
                    creations.push((base_type(text(t, src)), line_of(n)));
                }
            }
            _ => {}
        }
    });
    Some(FileCtx {
        rel: sf.rel.clone(),
        fam,
        code,
        frames,
        calls,
        creations,
        imports,
    })
}

type CallRow = (usize, u32, Option<String>);

/// The parsed project: files by path, call and construction indexes.
struct Project<'a> {
    files: Vec<FileCtx>,
    by_rel: HashMap<&'a str, usize>,
    calls: HashMap<(Fam, String), Vec<CallRow>>,
    creations: HashMap<(Fam, String), Vec<(usize, u32)>>,
    memo: RefCell<HashMap<(usize, u32, u8), bool>>,
    /// Python: the project files a dotted module name reaches from a file, by file and name.
    modules: RefCell<HashMap<(usize, String), Vec<usize>>>,
}

fn new_fn() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(?:New|new|Default|default|Must)[A-Z_]?\w*$").expect("static"))
}

impl<'a> Project<'a> {
    fn build(sources: &'a [SourceFile]) -> Self {
        let mut files = Vec::new();
        let mut by_rel = HashMap::new();
        for s in sources.iter().filter(|s| !s.is_test) {
            if let Some(f) = build_file(s) {
                by_rel.insert(s.rel.as_str(), files.len());
                files.push(f);
            }
        }
        let mut calls: HashMap<(Fam, String), Vec<CallRow>> = HashMap::new();
        let mut creations: HashMap<(Fam, String), Vec<(usize, u32)>> = HashMap::new();
        for (i, f) in files.iter().enumerate() {
            for c in &f.calls {
                calls
                    .entry((f.fam, c.name.clone()))
                    .or_default()
                    .push((i, c.line, c.qual.clone()));
            }
            for (t, l) in &f.creations {
                creations
                    .entry((f.fam, t.clone()))
                    .or_default()
                    .push((i, *l));
            }
        }
        Self {
            files,
            by_rel,
            calls,
            creations,
            memo: RefCell::new(HashMap::new()),
            modules: RefCell::new(HashMap::new()),
        }
    }

    fn file(&self, rel: &str) -> Option<&FileCtx> {
        self.by_rel.get(rel).map(|&i| &self.files[i])
    }

    /// True when `f` runs only once, at start-up.
    fn is_init(&self, fam: Fam, f: Option<&Frame>) -> bool {
        let Some(f) = f else { return true };
        if f.ctor || f.clinit || (f.lit && f.sync_cb) {
            return true;
        }
        let n = f.name.as_deref().unwrap_or("");
        match fam {
            Fam::Go => {
                n == "init"
                    || n == "main"
                    || (f.recv_type.is_none() && !f.lit && new_fn().is_match(n))
            }
            Fam::Python => matches!(n, "main" | "setup" | "setUp"),
            Fam::Java => n == "main",
            Fam::Js => false,
        }
    }

    fn exported(&self, fc: &FileCtx, f: Option<&Frame>) -> bool {
        let Some(f) = f else { return false };
        let n = f.name.as_deref().unwrap_or("");
        match fc.fam {
            Fam::Go => {
                if format!("/{}", fc.rel).contains("/internal/") {
                    return false;
                }
                // An exported method of an unexported type is reachable through the
                // interface the type implements, so the type's own name does not matter.
                !f.lit && n.starts_with(char::is_uppercase)
            }
            Fam::Python => {
                let stem = fc.rel.rsplit_once('.').map_or(fc.rel.as_str(), |(s, _)| s);
                if stem
                    .split('/')
                    .any(|p| p.starts_with('_') && p != "__init__")
                {
                    return false;
                }
                if f.private_class {
                    return false;
                }
                !n.is_empty() && (!n.starts_with('_') || (n.starts_with("__") && n.ends_with("__")))
            }
            Fam::Java => f.public && !f.private_class && !f.lit,
            Fam::Js => !n.is_empty() && !n.starts_with(['_', '#']),
        }
    }

    /// The outermost function around a line: the one whose kind decides initialiser.
    fn outer<'f>(&self, chain: &[&'f Frame]) -> Option<&'f Frame> {
        chain.last().copied()
    }

    fn callers(&self, fc: &FileCtx, f: &Frame) -> Vec<(usize, u32)> {
        let Some(name) = f.name.as_deref() else {
            return Vec::new();
        };
        let godir = (fc.fam == Fam::Go && f.recv_type.is_none()).then(|| fc.dir().to_owned());
        let private_here = match fc.fam {
            Fam::Java => false,
            Fam::Python | Fam::Js => name.starts_with('_') && !name.starts_with("__"),
            Fam::Go => false,
        };
        self.calls
            .get(&(fc.fam, name.to_owned()))
            .into_iter()
            .flatten()
            .filter(|(fi, ln, qual)| {
                let other = &self.files[*fi];
                if let Some(d) = &godir {
                    let base = d.rsplit('/').next().unwrap_or("");
                    match qual {
                        None if other.dir() != d => return false,
                        Some(q) if q != base => return false,
                        _ => {}
                    }
                }
                if private_here && other.rel != fc.rel {
                    return false;
                }
                if fc.fam == Fam::Python
                    && f.class.is_none()
                    && let Some(&target) = self.by_rel.get(fc.rel.as_str())
                    && !self.py_call_may_reach(target, name, *fi, qual.as_deref())
                {
                    return false;
                }
                // The function's own body (recursion) is not a caller.
                !(other.rel == fc.rel && f.start <= *ln && *ln <= f.end)
            })
            .map(|(fi, ln, _)| (*fi, *ln))
            .collect()
    }

    /// Python: the project files the module `dotted` (leading dots for a relative name) names
    /// in the file `from`: `m.py` and `m/__init__.py`, an absolute name matched at the end of
    /// the path because a project often lives under `src/`.
    fn module_files(&self, from: usize, dotted: &str) -> Vec<usize> {
        let key = (from, dotted.to_owned());
        if let Some(v) = self.modules.borrow().get(&key) {
            return v.clone();
        }
        let dots = dotted.chars().take_while(|c| *c == '.').count();
        let tail = dotted[dots..].replace('.', "/");
        let mut cands: Vec<String> = Vec::new();
        let rel = dots > 0;
        let prefix = if rel {
            let mut dir: Vec<&str> = self.files[from].dir().split('/').collect();
            dir.retain(|d| !d.is_empty());
            for _ in 1..dots {
                dir.pop();
            }
            dir.join("/")
        } else {
            String::new()
        };
        let base = match (prefix.is_empty(), tail.is_empty()) {
            (true, _) => tail,
            (false, true) => prefix,
            (false, false) => format!("{prefix}/{tail}"),
        };
        if base.is_empty() {
            cands.push("__init__.py".to_owned());
        } else {
            cands.push(format!("{base}.py"));
            cands.push(format!("{base}/__init__.py"));
        }
        let out: Vec<usize> = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, f)| f.fam == Fam::Python)
            .filter(|(_, f)| {
                cands
                    .iter()
                    .any(|c| f.rel == *c || (!rel && f.rel.ends_with(&format!("/{c}"))))
            })
            .map(|(i, _)| i)
            .collect();
        self.modules.borrow_mut().insert(key, out.clone());
        out
    }

    /// Python: whether the module named `dotted` in file `from` is the file `target`. A
    /// package `__init__` may re-export the function, and a name that no project file has but
    /// whose parent module is in the project may be an attribute of that module: both count.
    /// A name that reaches nothing in the project, or another module, does not.
    fn py_module_is(&self, from: usize, dotted: &str, target: usize) -> bool {
        let files = self.module_files(from, dotted);
        if !files.is_empty() {
            return files.contains(&target)
                || files
                    .iter()
                    .any(|&i| self.files[i].rel.ends_with("__init__.py"));
        }
        // `a.b.attr`: the longest prefix that is a project module.
        let mut parts: Vec<&str> = dotted.split('.').collect();
        while parts.len() > 1 {
            parts.pop();
            let p = parts.join(".");
            if p.chars().all(|c| c == '.') {
                break;
            }
            if !self.module_files(from, &p).is_empty() {
                return true;
            }
        }
        // A relative name that does not resolve: nothing is known.
        dotted.starts_with('.')
    }

    /// Python: whether the call `qual.name(..)` (or `name(..)` without `qual`) in file `fi`
    /// may be a call of the function `name` defined in file `target`. A call that goes to a
    /// function of the same name in another module, one the calling file defines itself or
    /// imports from elsewhere, is not a caller of it. A call on an object (`self.table.f(..)`)
    /// is read as before: it may be.
    fn py_call_may_reach(&self, target: usize, name: &str, fi: usize, qual: Option<&str>) -> bool {
        if fi == target {
            return true;
        }
        let o = &self.files[fi];
        let Some(q) = qual else {
            if o.frames
                .iter()
                .any(|f| !f.lit && f.class.is_none() && f.name.as_deref() == Some(name))
            {
                return false;
            }
            return match o.imports.binds.get(name) {
                Some(Bind::From(m, n)) => n == name && self.py_module_is(fi, m, target),
                Some(Bind::Module(_)) => true,
                None => o.imports.star,
            };
        };
        if !q
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            return true;
        }
        let (root, rest) = q.split_once('.').map_or((q, ""), |(r, t)| (r, t));
        let with_rest = |m: String| {
            if rest.is_empty() {
                m
            } else {
                format!("{m}.{rest}")
            }
        };
        match o.imports.binds.get(root) {
            None => true,
            Some(Bind::Module(d)) => self.py_module_is(fi, &with_rest(d.clone()), target),
            Some(Bind::From(m, n)) => {
                let sep = if m.ends_with('.') { "" } else { "." };
                let dotted = with_rest(format!("{m}{sep}{n}"));
                if !self.module_files(fi, &dotted).is_empty() {
                    self.py_module_is(fi, &dotted, target)
                } else {
                    // `n` is not a module of the project: an object of module `m`, or of
                    // something outside the project.
                    !self.module_files(fi, m).is_empty() || m.starts_with('.')
                }
            }
        }
    }

    /// The functions between `f` and the start-up statements that run it: the named function
    /// around each call of `f` that is not itself an initialiser, then that function's own
    /// callers, up to `hops` calls up. Nearest first, each once. Meaningful when
    /// `startup(fc, f, hops)` holds.
    fn startup_via(&self, fc: &FileCtx, f: &Frame, hops: u8) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if hops == 0 || self.is_init(fc.fam, Some(f)) {
            return out;
        }
        for (fi, ln) in self.callers(fc, f) {
            let o = &self.files[fi];
            let chain = o.chain(ln);
            if self
                .outer(&chain)
                .is_none_or(|x| self.is_init(o.fam, Some(x)))
            {
                continue;
            }
            let Some(g) = chain.iter().rev().find(|g| !g.lit && g.name.is_some()) else {
                continue;
            };
            let name = g.name.clone().unwrap_or_default();
            if !out.contains(&name) {
                out.push(name);
            }
            for n in self.startup_via(o, g, hops - 1) {
                if !out.contains(&n) {
                    out.push(n);
                }
            }
        }
        out
    }

    /// True when every call of `f` (up to `hops` calls up) is an initialiser, or `f` is one.
    fn startup(&self, fc: &FileCtx, f: &Frame, hops: u8) -> bool {
        if self.is_init(fc.fam, Some(f)) {
            return true;
        }
        if hops == 0 || f.lit || f.name.as_deref().is_none_or(|n| n.len() < 3) {
            return false;
        }
        let key = (
            self.by_rel
                .get(fc.rel.as_str())
                .copied()
                .unwrap_or(usize::MAX),
            f.start,
            hops,
        );
        if let Some(&v) = self.memo.borrow().get(&key) {
            return v;
        }
        // Recursion guard.
        self.memo.borrow_mut().insert(key, false);
        let callers = self.callers(fc, f);
        let ok = (1..=MAX_CALLERS).contains(&callers.len())
            && callers
                .iter()
                .all(|&(fi, ln)| self.site_startup(&self.files[fi], ln, hops - 1));
        self.memo.borrow_mut().insert(key, ok);
        ok
    }

    /// True when the code at `line` of `fc` runs at start-up only.
    fn site_startup(&self, fc: &FileCtx, line: u32, hops: u8) -> bool {
        let chain = fc.chain(line);
        let Some(outer) = self.outer(&chain) else {
            return true;
        };
        if self.is_init(fc.fam, Some(outer)) {
            return true;
        }
        hops > 0
            && chain
                .iter()
                .rev()
                .find(|f| !f.lit && f.name.is_some())
                .is_some_and(|f| self.startup(fc, f, hops))
    }
}

// --------------------------------------------------------------------------------------
// Declarations.

fn rx(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).expect("static pattern"))
}

/// The declaration of a holder: its line, and the lines after it while brackets stay open;
/// Python decorators above it, and below it down to the `def`.
fn decl_text(fc: &FileCtx, line: u32) -> String {
    let mut out = fc.statement(line);
    if fc.fam == Fam::Python {
        let mut k = line;
        let mut pre = Vec::new();
        while k > 1 && fc.line(k - 1).trim_start().starts_with('@') {
            k -= 1;
            pre.push(fc.line(k).trim().to_owned());
        }
        pre.reverse();
        let mut k = line;
        let mut post = Vec::new();
        while fc.line(k).trim_start().starts_with('@') && k < line + 8 {
            k += 1;
            post.push(fc.line(k).trim().to_owned());
            if fc.line(k).trim_start().starts_with("def ")
                || fc.line(k).trim_start().starts_with("async def ")
            {
                break;
            }
        }
        out = format!("{} {} {}", pre.join(" "), out.trim(), post.join(" "));
    }
    out
}

/// Facts read from the declaration text.
struct Decl {
    container: Option<ContainerClass>,
    bound: Bound,
    key_bounded: bool,
    pool: bool,
    literal: bool,
    private: bool,
    unbounded_cache: bool,
}

fn classify_decl(fc: &FileCtx, raw: &Raw, name: &str, dtext: &str, longtext: &str) -> Decl {
    static GO_NONCONT: OnceLock<Regex> = OnceLock::new();
    static GO_CONTAINER: OnceLock<Regex> = OnceLock::new();
    static GO_KEEP: OnceLock<Regex> = OnceLock::new();
    static PY_CONT: OnceLock<Regex> = OnceLock::new();
    static PY_NONCONT: OnceLock<Regex> = OnceLock::new();
    static JVM_CONT: OnceLock<Regex> = OnceLock::new();
    static JVM_SCALAR: OnceLock<Regex> = OnceLock::new();
    static JS_CONT: OnceLock<Regex> = OnceLock::new();
    static BOUND: OnceLock<Regex> = OnceLock::new();
    static WEAK_KEYS: OnceLock<Regex> = OnceLock::new();
    static WEAK_VALUES: OnceLock<Regex> = OnceLock::new();
    static KEY_BOUNDED: OnceLock<Regex> = OnceLock::new();
    static POOL: OnceLock<Regex> = OnceLock::new();
    static CHAN: OnceLock<Regex> = OnceLock::new();
    static UNB: OnceLock<Regex> = OnceLock::new();
    static LRU_SIZED: OnceLock<Regex> = OnceLock::new();
    let t = dtext;
    let mut cont: i8 = 0;
    let kind = raw.kind.as_str();
    if matches!(kind, "registration" | "static-init-registration") {
        cont = 1;
    }
    if matches!(
        kind,
        "registry-field" | "static-registry" | "package-registry"
    ) {
        cont = 2;
    }
    let cache_dec = kind == "cache-decorator"
        || rx(&UNB, r"@\s*(?:functools\.)?(?:lru_cache|cache)\b").is_match(t);
    if cache_dec {
        cont = 2;
    }
    match fc.fam {
        _ if cache_dec => {}
        Fam::Go => {
            let keep = rx(
                &GO_KEEP,
                r"\bmap\[|\[\]|\bchan\b|sync\.Map|Unbounded|Buffer|Queue|List|\*?[A-Z]\w*\[",
            );
            if rx(
                &GO_NONCONT,
                r"sync\.(?:Pool|Once|Mutex|RWMutex|WaitGroup)|\bregexp\.|\berror\b|\bfunc\s*\(|atomic\.|\btime\.|Logger|\*?log\.|\bcontext\.|errors\.New|\bbool\b|\bint\d*\b|\buint\d*\b|\bfloat\d*\b",
            )
            .is_match(t)
                && !keep.is_match(t)
            {
                cont = -1;
            } else if rx(
                &GO_CONTAINER,
                r"\bmap\[|\[\]|\bchan\b|sync\.Map|list\.List|\*?list\.|heap\.|container/|\bring\.Ring|Registry|Cache|(?:Set|Queue|Table|Store|Index)\b",
            )
            .is_match(t)
            {
                cont = 2;
            }
        }
        Fam::Python => {
            let c = rx(
                &PY_CONT,
                r"=\s*(?:\{\s*\}|\[\s*\]|dict\(|list\(|set\(|\{[^}]*\}|\[[^\]]*\]|(?:collections\.)?(?:defaultdict|deque|OrderedDict|Counter|ChainMap)\(|(?:weakref\.)?Weak(?:Value|Key)?(?:Dictionary|Set)\(|\w*Cache\w*\()|:\s*(?:Dict|List|Set|MutableMapping|Mapping|MutableSequence|Deque|DefaultDict|OrderedDict|dict|list|set|deque)\b",
            );
            let n = rx(
                &PY_NONCONT,
                r#"TypeVar\(|getLogger|re\.compile|\bLock\(|\bEvent\(|\bobject\(\)|frozenset\(|\bNewType\(|ParamSpec\(|namedtuple\(|NamedTuple|\bEnum\b|Final\[|\bLiteral\[|\bTuple\[|=\s*(?:"|'|\d|True|False)|\bproperty\(|\blambda\b"#,
            );
            if n.is_match(t) && !c.is_match(t) {
                cont = -1;
            } else if c.is_match(t) {
                cont = 2;
            }
        }
        Fam::Java => {
            let ty = t.split(name).next().unwrap_or(t);
            let init = t.split_once('=').map_or("", |(_, r)| r);
            let jc = rx(
                &JVM_CONT,
                r"\b(?:List|ArrayList|LinkedList|Map|HashMap|LinkedHashMap|TreeMap|SortedMap|NavigableMap|ConcurrentHashMap|ConcurrentMap|ConcurrentSkipListMap|WeakHashMap|IdentityHashMap|EnumMap|Set|HashSet|LinkedHashSet|TreeSet|SortedSet|ConcurrentSkipListSet|Queue|Deque|ArrayDeque|PriorityQueue|BlockingQueue|BlockingDeque|LinkedBlockingQueue|LinkedBlockingDeque|ArrayBlockingQueue|ConcurrentLinkedQueue|ConcurrentLinkedDeque|CopyOnWriteArrayList|CopyOnWriteArraySet|Collection|Vector|Stack|Hashtable|Cache|LoadingCache|AsyncCache|Multimap|ListMultimap|SetMultimap|Table|BitSet|MutableList|MutableMap|MutableSet|MutableCollection|Iterable|Properties|StringBuffer|StringBuilder)\b|\[\]|\w*(?:Cache|Registry|Pool|Queue|Store|Chain|Buffer|Holder|Table|Index|Memo|Bus)\b",
            );
            if jc.is_match(ty) || jc.is_match(init) {
                cont = 2;
            } else if cont < 1
                && rx(
                    &JVM_SCALAR,
                    r"\b(?:String|int|long|boolean|Logger|Pattern|Class|Object|Lock|AtomicInteger|AtomicLong|Random|[A-Z]\w*Logger)\b",
                )
                .is_match(ty)
            {
                cont = -1;
            }
        }
        Fam::Js => {
            if rx(
                &JS_CONT,
                r"new\s+(?:Map|Set|WeakMap|WeakSet|Array)\b|=\s*\[|=\s*\{\s*\}|Object\.create",
            )
            .is_match(t)
            {
                cont = 2;
            }
        }
    }
    // Bounds.
    let both = format!("{t} {longtext}");
    let mut bound = Bound::None;
    let sized_lru = rx(
        &LRU_SIZED,
        r"lru_cache\s*\(\s*(?:maxsize\s*=\s*)?\d+|lru_cache\s*(?:\(\s*\))?\s*(?:$|def\b|@)",
    )
    .is_match(t);
    let unbounded_cache = cache_dec
        && !sized_lru
        && Regex::new(
            r"lru_cache\s*\(\s*(?:maxsize\s*=\s*)?None\s*\)|@\s*(?:functools\.)?cache\b(?:\s*\()?",
        )
        .is_ok_and(|r| r.is_match(t));
    if unbounded_cache {
        bound = Bound::UnboundedCache;
    } else if rx(
        &BOUND,
        r"WeakHashMap|WeakMap|WeakSet|WeakValueDictionary|WeakKeyDictionary|WeakReference|ConcurrentReferenceHashMap|SoftReference|maxlen|maximumSize|expireAfter|removeEldestEntry|LRU|lru|ring\.Ring|BoundedQueue|ArrayBlockingQueue|maxsize\s*=\s*\d|WeakRef|\.weakKeys|\.weakValues|\.softValues",
    )
    .is_match(&both)
    {
        bound = if rx(
            &WEAK_KEYS,
            r"WeakHashMap|WeakMap|WeakSet|WeakKeyDictionary|ConcurrentReferenceHashMap|weakKeys",
        )
        .is_match(&both)
        {
            Bound::WeakKeys
        } else if rx(
            &WEAK_VALUES,
            r"WeakValueDictionary|WeakReference|SoftReference|WeakRef|weakValues|softValues",
        )
        .is_match(&both)
        {
            Bound::WeakValues
        } else {
            Bound::Bounded
        };
    }
    if fc.fam == Fam::Go && rx(&CHAN, r"make\(\s*chan\b[^,)]*,\s*[\w.]+").is_match(t) {
        bound = Bound::Bounded;
    }
    let key_bounded = rx(
        &KEY_BOUNDED,
        r"(?:Map|Cache|Table)\s*<\s*(?:Class|Method|Constructor|Field|Type|Package|Enum\w*)\b|map\[reflect\.(?:Type|Kind)\]|map\[\s*type\s*\]|Dict\[\s*(?:type|Type)\b",
    )
    .is_match(t);
    // A pool is a type, not a name: look at the declaration without the name and the value.
    let no_value = t.split('=').next().unwrap_or(t).replacen(name, " ", 1);
    let pool = rx(&POOL, r"(?i)\b[\w.]*pool\b").is_match(&no_value)
        && !Regex::new(r"(?i)threadpool|executor").is_ok_and(|r| r.is_match(&no_value))
        || (fc.fam == Fam::Go && t.contains("sync.Pool"));
    let literal = literal_table(fc.fam, t);
    let private = match fc.fam {
        Fam::Go => !name.starts_with(char::is_uppercase),
        Fam::Python => name.starts_with('_') && !(name.starts_with("__") && name.ends_with("__")),
        Fam::Java => {
            Regex::new(r"\bprivate\b").is_ok_and(|r| r.is_match(t.split(name).next().unwrap_or(t)))
        }
        Fam::Js => {
            if raw.member_access {
                name.starts_with(['_', '#']) || t.contains("private ")
            } else {
                !Regex::new(r"\bexport\b|module\.exports|exports\.").is_ok_and(|r| r.is_match(t))
            }
        }
    };
    Decl {
        container: match cont {
            2 => Some(ContainerClass::Container),
            1 => Some(ContainerClass::Registry),
            -1 => Some(ContainerClass::NotContainer),
            _ => None,
        },
        bound,
        key_bounded,
        pool,
        literal,
        private,
        unbounded_cache,
    }
}

fn literal_table(fam: Fam, t: &str) -> bool {
    static GO: OnceLock<Regex> = OnceLock::new();
    static PY: OnceLock<Regex> = OnceLock::new();
    static JAVA: OnceLock<Regex> = OnceLock::new();
    match fam {
        Fam::Go => {
            rx(
                &GO,
                r"(?:\bmap\[[^\]]*\][\w.*\[\]]*|\[\][\w.*]+)\s*\{\s*[^}\s]",
            )
            .is_match(t)
                && !t.contains("make(")
        }
        Fam::Python => rx(
            &PY,
            r"=\s*(?:\{\s*[^}\s]|\[\s*[^\]\s]|\(\s*[^)\s]|frozenset|MappingProxyType)",
        )
        .is_match(t),
        Fam::Java => rx(
            &JAVA,
            r"\b(?:Map|Set|List)\.(?:of|copyOf)\(|Arrays\.asList|Immutable(?:Map|Set|List)|Collections\.(?:unmodifiable|empty|singleton)",
        )
        .is_match(t),
        Fam::Js => false,
    }
}

// --------------------------------------------------------------------------------------
// The analysis.

/// The writers and removers that survive, and the facts of one holder.
pub struct Analysis {
    /// `file:line` of the sites that add or reassign.
    pub writers: Vec<String>,
    /// `file:line` of the sites that delete or reset.
    pub removers: Vec<String>,
    /// What the tree says.
    pub facts: HolderFacts,
}

fn site_parts(s: &str) -> Option<(&str, u32)> {
    let (f, l) = s.rsplit_once(':')?;
    Some((f, l.parse().ok()?))
}

/// Units: census kinds whose entry is the creation of something that must be released.
fn is_unit_kind(kind: &str) -> bool {
    matches!(
        kind,
        "async-task"
            | "spawn"
            | "goroutine"
            | "timer"
            | "timer-field"
            | "handle"
            | "handle-field"
            | "thread"
            | "spawned-unit-field"
            | "allocation"
            | "acquire"
            | "executor"
            | "scheduled-task"
            | "shutdown-hook"
    )
}

fn lifetime_of(raw: &Raw) -> Lifetime {
    use dla_core::finding::Holder as H;
    let k = raw.kind.as_str();
    if is_unit_kind(k) {
        return Lifetime::Unit;
    }
    if raw.holder == H::StaticGlobal
        || matches!(
            k,
            "package-registry"
                | "static-registry"
                | "static-init-registration"
                | "module-global"
                | "static-container"
                | "static-field"
                | "package-var"
                | "class-attr"
                | "cache-decorator"
                | "module-pointer"
                | "static-closure"
        )
    {
        return Lifetime::Process;
    }
    if matches!(
        raw.holder,
        H::ContainerField | H::RegistrationTable | H::ClosureContinuation
    ) || matches!(
        k,
        "instance-field"
            | "singleton-field"
            | "registry-field"
            | "closure-field"
            | "container-field"
    ) {
        return Lifetime::Object;
    }
    Lifetime::Unit
}

fn owner_type(raw: &Raw, life: Lifetime) -> Option<String> {
    if !raw.member_access && life != Lifetime::Object {
        return None;
    }
    let parts: Vec<&str> = raw.owner.split('.').collect();
    (parts.len() >= 2).then(|| parts[parts.len() - 2].to_owned())
}

/// Reads every holder's declaration and sites. `None` for a holder of a language this pass
/// does not read: its name matches stand as they are.
pub fn analyse(raws: &[Raw], usages: &[Usage], files: &[SourceFile]) -> Vec<Option<Analysis>> {
    let proj = Project::build(files);
    raws.iter()
        .zip(usages)
        .map(|(r, u)| analyse_one(&proj, r, u))
        .collect()
}

fn analyse_one(proj: &Project<'_>, raw: &Raw, usage: &Usage) -> Option<Analysis> {
    let fc = proj.file(&raw.file)?;
    let fam = fc.fam;
    let name = raw.name.clone().unwrap_or_default();
    let mut facts = HolderFacts::default();
    let unit = is_unit_kind(&raw.kind);
    // Declaration.
    let dtext = decl_text(fc, raw.line);
    let evicts = dtext.contains("LinkedHashMap")
        && (raw.line..raw.line + 30)
            .map(|l| fc.line(l))
            .collect::<Vec<_>>()
            .join(" ")
            .contains("removeEldestEntry");
    let longtext = if evicts { " removeEldestEntry" } else { "" }.to_owned();
    let decl = classify_decl(fc, raw, &name, &dtext, &longtext);
    let life = lifetime_of(raw);
    facts.lifetime = Some(life);
    facts.private = Some(decl.private);
    facts.unit = unit;
    facts.bound = Some(decl.bound);
    facts.key_bounded = decl.key_bounded;
    facts.pool = decl.pool && !unit;
    if !unit {
        facts.container = decl.container;
    }
    let owner_ty = owner_type(raw, life);
    let ofile_stem = raw
        .file
        .rsplit('/')
        .next()
        .and_then(|f| f.split('.').next())
        .unwrap_or("");

    // Which sites can name the holder at all.
    let credible = |chain: &[&Frame], text: &str, other: &FileCtx| -> bool {
        let same = other.rel == raw.file;
        match fam {
            Fam::Go => {
                if raw.member_access {
                    // Only a method of the owner type (or code holding a parameter of it)
                    // can name its field; another struct's same-named field is not it.
                    let Some(ty) = &owner_ty else { return same };
                    let typed = chain.iter().any(|f| {
                        f.recv_type.as_deref() == Some(ty.as_str())
                            || f.typed.iter().any(|(_, t)| t == ty)
                    });
                    let other_method = chain.iter().any(|f| f.recv_type.is_some());
                    typed || (same && !other_method)
                } else if same || other.dir() == fc.dir() {
                    true
                } else if decl.private {
                    false
                } else {
                    let pkg = fc.dir().rsplit('/').next().unwrap_or("");
                    Regex::new(&format!(
                        r"\b{}\s*\.\s*{}\b",
                        regex::escape(pkg),
                        regex::escape(&name)
                    ))
                    .is_ok_and(|r| r.is_match(text))
                }
            }
            Fam::Java => {
                same || {
                    let cls = owner_ty.clone().or_else(|| {
                        let p: Vec<&str> = raw.owner.split('.').collect();
                        (p.len() >= 2).then(|| p[p.len() - 2].to_owned())
                    });
                    cls.is_some_and(|c| {
                        Regex::new(&format!(
                            r"\b{}\s*\.\s*{}\b",
                            regex::escape(&c),
                            regex::escape(&name)
                        ))
                        .is_ok_and(|r| r.is_match(text))
                    }) && !decl.private
                }
            }
            Fam::Python => {
                same || (!decl.private
                    && Regex::new(&format!(
                        r"\b{}\s*\.\s*{}\b",
                        regex::escape(ofile_stem),
                        regex::escape(&name)
                    ))
                    .is_ok_and(|r| r.is_match(text)))
            }
            Fam::Js => same || !decl.private,
        }
    };

    let mut census_sites: Vec<&String> = Vec::new();
    for s in usage.writers.iter().chain(&usage.removers) {
        if !census_sites.contains(&s) {
            census_sites.push(s);
        }
    }
    let mut writers: Vec<SiteFact> = Vec::new();
    let mut removers: Vec<SiteFact> = Vec::new();
    for site in census_sites {
        let Some((rel, line)) = site_parts(site) else {
            continue;
        };
        let Some(sfc) = proj.file(rel) else { continue };
        let stmt = sfc.statement(line);
        let chain = sfc.chain(line);
        if !credible(&chain, &stmt, sfc) {
            continue;
        }
        let evs = events(&stmt, &name);
        let in_writers = usage.writers.contains(site);
        let outer = proj.outer(&chain);
        let init = proj.is_init(fam, outer);
        let startup = init || {
            chain
                .iter()
                .rev()
                .find(|f| !f.lit && f.name.is_some())
                .is_some_and(|f| proj.startup(sfc, f, STARTUP_HOPS))
        };
        let function = chain
            .iter()
            .rev()
            .find(|f| !f.lit)
            .and_then(|f| f.name.clone());
        // Start-up reached through wrappers: the functions on the way (Python only).
        let startup_via = if startup && !init && fam == Fam::Python {
            chain
                .iter()
                .rev()
                .find(|f| !f.lit && f.name.is_some())
                .map(|f| proj.startup_via(sfc, f, STARTUP_HOPS))
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let exported = !init && proj.exported(sfc, outer);
        let mk = |kind: SiteKind, depth: u8, ev: Option<&crate::events::Event>| {
            let (param, constant) = match ev {
                Some(e) if kind == SiteKind::Insert => param_flow(sfc, &chain, line, e, &stmt),
                _ => (false, false),
            };
            SiteFact {
                site: site.clone(),
                kind,
                depth,
                function: function.clone(),
                init,
                startup,
                startup_via: startup_via.clone(),
                param,
                constant,
                exported,
            }
        };
        let grow: Vec<_> = evs.iter().filter(|e| e.kind == Ev::Grow).collect();
        let rem: Vec<_> = evs.iter().filter(|e| e.kind == Ev::Remove).collect();
        let rst = evs.iter().any(|e| e.kind == Ev::Reset);
        let reass = evs.iter().any(|e| e.kind == Ev::Reassign);
        if let Some(g) = grow.iter().min_by_key(|e| e.depth) {
            writers.push(mk(SiteKind::Insert, g.depth, Some(g)));
        } else if reass && wraps_itself_through_a_local(sfc, &chain, line, &evs, &name) {
            // `context = new Context(.., head); head = context;`: the head holds the old head.
            writers.push(mk(SiteKind::Insert, 0, None));
        } else if reass || (rst && in_writers) {
            writers.push(mk(SiteKind::Reassign, 0, None));
        }
        // An empty value assigned where the owner is built is its initial value.
        let rst = rst && (!rem.is_empty() || !init);
        if !rem.is_empty() || rst {
            let d = rem.iter().map(|e| e.depth).min().unwrap_or(0);
            removers.push(mk(
                if rem.is_empty() {
                    SiteKind::Reset
                } else {
                    SiteKind::Delete
                },
                d,
                None,
            ));
        }
    }

    // A pointer, struct or scalar that is only ever assigned as a whole is not a container,
    // and its assignments are not writers: the holder is listed only when a rule names it.
    let inserts = writers.iter().any(|w| w.kind == SiteKind::Insert);
    if !unit
        && facts.container.is_none()
        && !writers.is_empty()
        && !inserts
        && removers.is_empty()
        && writers.iter().all(|w| w.kind == SiteKind::Reassign)
    {
        facts.container = Some(ContainerClass::NotContainer);
    }
    if !unit && facts.container == Some(ContainerClass::NotContainer) && !inserts {
        writers.clear();
    }
    // A literal or immutable table that nothing inserts into is only read.
    facts.literal = decl.literal && !writers.iter().any(|w| w.kind == SiteKind::Insert);
    if decl.unbounded_cache {
        facts.container = Some(ContainerClass::Container);
    }

    // A unit entry: the creation is the growth.
    let out_writers: Vec<String> = writers.iter().map(|w| w.site.clone()).collect();
    if unit {
        let chain = fc.chain(raw.line);
        let outer = proj.outer(&chain);
        let init = !chain.is_empty() && proj.is_init(fam, outer) || chain.is_empty();
        let startup = init
            || chain
                .iter()
                .rev()
                .find(|f| !f.lit && f.name.is_some())
                .is_some_and(|f| proj.startup(fc, f, STARTUP_HOPS));
        let run = !startup;
        writers.insert(
            0,
            SiteFact {
                site: format!("{}:{}", raw.file, raw.line),
                kind: SiteKind::Insert,
                depth: 0,
                function: chain
                    .iter()
                    .rev()
                    .find(|f| !f.lit)
                    .and_then(|f| f.name.clone()),
                init,
                startup,
                startup_via: Vec::new(),
                param: false,
                constant: false,
                exported: run && proj.exported(fc, outer),
            },
        );
    }
    // Registrations: where the receiver comes from.
    if matches!(
        raw.kind.as_str(),
        "registration" | "static-init-registration"
    ) && !name.is_empty()
    {
        facts.receiver = receiver_of(fc, raw, &name);
    }
    // How instances of the owner type are made.
    if life == Lifetime::Object
        && let Some(ty) = owner_ty.as_deref().filter(|t| t.len() >= 3)
    {
        facts.owner_creation = owner_creation(proj, fc, ty);
    }
    let removers_out: Vec<String> = removers.iter().map(|r| r.site.clone()).collect();
    facts.writer_sites = writers;
    facts.remover_sites = removers;
    Some(Analysis {
        writers: out_writers,
        removers: removers_out,
        facts,
    })
}

/// True when the holder is assigned a local variable that the function made hold the holder's
/// own value earlier: built from it (`ctx = new Ctx(.., latest); latest = ctx;`) or linked to
/// it (`node.previous = latest; latest = node;`). Either is the chain of
/// `latest = new Ctx(.., latest)`, one hop away.
fn wraps_itself_through_a_local(
    fc: &FileCtx,
    chain: &[&Frame],
    line: u32,
    evs: &[crate::events::Event],
    name: &str,
) -> bool {
    let Some(outer) = chain.last() else {
        return false;
    };
    let Some(rhs) = evs
        .iter()
        .find(|e| e.kind == Ev::Reassign)
        .map(|e| e.rhs.trim())
    else {
        return false;
    };
    let var = rhs.trim_end_matches(';').trim();
    if var.is_empty()
        || !var
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
    {
        return false;
    }
    let v = regex::escape(var);
    let (Ok(built), Ok(linked)) = (
        Regex::new(&format!(r"\b{v}\s*:?=([^=].*)$")),
        Regex::new(&format!(
            r"\b{v}\s*(?:\.|->)\s*[A-Za-z_$][\w$]*\s*=([^=].*)$"
        )),
    ) else {
        return false;
    };
    (outer.start..line).any(|l| {
        let st = fc.statement(l);
        built
            .captures(&st)
            .is_some_and(|c| crate::events::holds_itself(&c[1], name))
            || linked
                .captures(&st)
                .is_some_and(|c| crate::events::links_to(&c[1], name))
    })
}

/// Does the inserted key or value derive from a parameter of the enclosing function (one hop
/// through local assignments), and is it a constant?
fn param_flow(
    fc: &FileCtx,
    chain: &[&Frame],
    line: u32,
    ev: &crate::events::Event,
    _stmt: &str,
) -> (bool, bool) {
    let mut ps: HashSet<String> = HashSet::new();
    let mut recvs: HashSet<String> = ["self", "cls", "this"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    for f in chain {
        ps.extend(f.params.iter().cloned());
        recvs.extend(f.recv.iter().cloned());
    }
    for r in &recvs {
        ps.remove(r);
    }
    let mut taint = ps.clone();
    if let Some(outer) = chain.last() {
        static ASSIGN: OnceLock<Regex> = OnceLock::new();
        let a = rx(
            &ASSIGN,
            r"^\s*(?:for\s+|(?:var|let|const|final)\s+|(?:[\w<>\[\],.?]+\s+)?)([\w$,\s]+?)\s*(?::=|=|\bin\b)\s*(.*)$",
        );
        for _ in 0..3 {
            for l in outer.start..line {
                let t = fc.line(l);
                if let Some(c) = a.captures(t) {
                    let rhs: HashSet<String> = idents(&c[2]).into_iter().collect();
                    if rhs.iter().any(|i| taint.contains(i)) {
                        for v in idents(&c[1]) {
                            taint.insert(v);
                        }
                    }
                }
            }
        }
        for r in &recvs {
            taint.remove(r);
        }
    }
    let ids = idents(&format!("{} {}", ev.args, ev.rhs));
    let param = ids.iter().any(|i| taint.contains(i));
    let constant = !param
        && (ids.is_empty()
            || ids.iter().all(|i| {
                i.chars().next().is_some_and(char::is_uppercase)
                    && i.chars()
                        .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
            }));
    (param, constant)
}

fn receiver_of(fc: &FileCtx, raw: &Raw, name: &str) -> Option<Receiver> {
    if matches!(name, "self" | "this" | "cls") {
        return None;
    }
    let chain = fc.chain(raw.line);
    let Some(f0) = chain.last() else {
        return Some(Receiver::Outer);
    };
    if chain.iter().any(|f| f.params.iter().any(|p| p == name)) {
        return Some(Receiver::Outer);
    }
    let n = regex::escape(name);
    let pat = match fc.fam {
        Fam::Go => format!(r"\b{n}\b[^=;]*:=|\bvar\s+{n}\b"),
        Fam::Java => format!(r"\b[A-Za-z_][\w<>\[\],.? ]*\s+{n}\s*=(?:[^=]|$)"),
        Fam::Python => format!(r"^\s*{n}\s*=(?:[^=]|$)|\bas\s+{n}\b|\bfor\s+{n}\b"),
        Fam::Js => format!(r"\b(?:const|let|var)\s+{n}\b"),
    };
    let re = Regex::new(&pat).ok()?;
    let local = (f0.start..=raw.line).any(|l| re.is_match(fc.line(l)));
    Some(if local {
        Receiver::Local
    } else {
        Receiver::Outer
    })
}

fn owner_creation(proj: &Project<'_>, fc: &FileCtx, ty: &str) -> Option<OwnerCreation> {
    let mut sites: Vec<(usize, u32)> = Vec::new();
    let rows = proj.creations.get(&(fc.fam, ty.to_owned()))?;
    for &(fi, ln) in rows.iter().take(MAX_CREATIONS) {
        let other = &proj.files[fi];
        if fc.fam == Fam::Go && other.dir() != fc.dir() {
            let pkg = fc.dir().rsplit('/').next().unwrap_or("");
            let l = other.line(ln);
            if !Regex::new(&format!(
                r"\b{}\s*\.\s*{}\b",
                regex::escape(pkg),
                regex::escape(ty)
            ))
            .is_ok_and(|r| r.is_match(l))
            {
                continue;
            }
        }
        // `NewT()`: the creation is where the factory is called.
        let chain = other.chain(ln);
        let factory = chain
            .iter()
            .rev()
            .find(|f| !f.lit && f.name.is_some() && f.recv_type.is_none())
            .filter(|f| {
                fc.fam == Fam::Go
                    && f.name
                        .as_deref()
                        .is_some_and(|n| n.starts_with("New") || n.starts_with("new"))
            });
        if let Some(f) = factory {
            let callers = proj.callers(other, f);
            if callers.is_empty() {
                sites.push((fi, ln));
            } else {
                sites.extend(callers.into_iter().take(MAX_CREATIONS));
            }
        } else {
            sites.push((fi, ln));
        }
    }
    if sites.is_empty() {
        return None;
    }
    let all_startup = sites
        .iter()
        .all(|&(fi, ln)| proj.site_startup(&proj.files[fi], ln, STARTUP_HOPS));
    Some(if all_startup {
        OwnerCreation::StartUp
    } else {
        OwnerCreation::PerCall
    })
}
