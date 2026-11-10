//! Per-language holder enumeration over tree-sitter syntax trees.

use std::{collections::HashSet, sync::OnceLock};

use dla_core::finding::{Holder, Region};
use regex::Regex;

use crate::{
    ast::{self, ancestors, first_word, last_word, text},
    model::{ProjectInfo, Raw, Scope, SourceFile},
    parse::{Lang, tree_sitter::Node},
};

pub mod go;
pub mod java;
pub mod kotlin;
pub mod native;
pub mod node;
pub mod python;
pub mod rust;

/// A call-site rule: a regex over the callee text and what a match is.
pub struct CallRule {
    /// Matches the callee text.
    pub re: Regex,
    /// Declaration kind recorded.
    pub kind: &'static str,
    /// Holder class.
    pub holder: Holder,
    /// Region.
    pub region: Region,
    /// Whether the call is an acquisition or spawn (the result's variable is tracked)
    /// rather than a registration on a receiver.
    pub assigned: bool,
}

/// Builds a rule table from `(pattern, kind, holder, region, assigned)` rows.
pub fn rules(rows: &[(&str, &'static str, Holder, Region, bool)]) -> Vec<CallRule> {
    rows.iter()
        .map(|(p, kind, holder, region, assigned)| CallRule {
            re: Regex::new(p).expect("static call rule"),
            kind,
            holder: *holder,
            region: *region,
            assigned: *assigned,
        })
        .collect()
}

/// A rule table, built once.
pub type Table = OnceLock<Vec<CallRule>>;

/// Per-file working state.
pub struct Cx<'a> {
    /// The file.
    pub file: &'a SourceFile,
    /// Cross-file facts.
    pub info: &'a ProjectInfo,
    /// Found holders.
    pub out: Vec<Raw>,
}

impl<'a> Cx<'a> {
    /// New context.
    pub fn new(file: &'a SourceFile, info: &'a ProjectInfo) -> Self {
        Self {
            file,
            info,
            out: Vec::new(),
        }
    }

    /// Source text.
    pub fn src(&self) -> &'a str {
        &self.file.text
    }

    /// Adds a holder.
    pub fn push(&mut self, r: Raw) {
        self.out.push(r);
    }

    /// Applies a call rule table to a call node with callee text `callee`.
    /// `receiver` is the receiver text for registrations.
    pub fn call(
        &mut self,
        node: Node<'_>,
        callee: &str,
        receiver: Option<&str>,
        table: &[CallRule],
    ) {
        let Some(rule) = table.iter().find(|r| r.re.is_match(callee)) else {
            return;
        };
        let (path, range) = describe(node, self.src(), self.file.lang);
        // A holder created in a static or instance initializer is owned by the field it
        // is stored into, never by the compiler-named `<clinit>` or a constructor.
        let stored = (rule.assigned && self.file.lang == Lang::Java)
            .then(|| initializer_store(node, self.src()))
            .flatten();
        let owner = match &stored {
            Some(f) => join_owner(
                &self.owner_prefix(),
                &join_owner(&class_path(node, self.src(), self.file.lang), f),
            ),
            None => join_owner(&self.owner_prefix(), &path),
        };
        let mut raw = Raw::new(
            rule.kind,
            rule.holder,
            rule.region,
            &self.file.rel,
            ast::line(node),
            owner,
        );
        if let Some(f) = &stored {
            raw = raw.tracking(f, Scope::Project, false);
        } else if rule.assigned {
            if let Some((n, member)) = assigned_name(node, self.src()) {
                let scope = if member {
                    Scope::Project
                } else {
                    range.map_or(Scope::File, |(a, b)| Scope::Lines(a, b))
                };
                raw = raw.tracking(&n, scope, member);
            }
        } else if let Some(n) = receiver.and_then(last_word) {
            let n = n.to_owned();
            raw = raw.tracking(&n, Scope::Project, false);
        }
        self.out.push(raw);
    }

    /// Package or module prefix for owners.
    pub fn owner_prefix(&self) -> String {
        match self.file.lang {
            Lang::Java | Lang::Kotlin => package_of(self.src()).unwrap_or_default(),
            _ => self.file.module(),
        }
    }
}

/// Joins an owner prefix and a path.
pub fn join_owner(prefix: &str, path: &str) -> String {
    match (prefix.is_empty(), path.is_empty()) {
        (true, _) => path.to_owned(),
        (_, true) => prefix.to_owned(),
        _ => format!("{prefix}.{path}"),
    }
}

/// The `package a.b;` declaration of a Java or Kotlin source.
pub fn package_of(src: &str) -> Option<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"(?m)^\s*package\s+([\w.]+)").expect("static"));
    re.captures(src).map(|c| c[1].to_owned())
}

fn class_kinds(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Java => &[
            "class_declaration",
            "interface_declaration",
            "enum_declaration",
            "record_declaration",
        ],
        Lang::Python => &["class_definition"],
        Lang::Rust => &["impl_item", "trait_item"],
        Lang::JavaScript | Lang::TypeScript | Lang::Tsx => &["class_declaration", "class"],
        Lang::Cpp => &["class_specifier", "struct_specifier"],
        _ => &[],
    }
}

fn fn_kinds(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Java => &["method_declaration", "constructor_declaration"],
        Lang::Go => &["function_declaration", "method_declaration"],
        Lang::Rust => &["function_item"],
        Lang::JavaScript | Lang::TypeScript | Lang::Tsx => &[
            "function_declaration",
            "method_definition",
            "generator_function_declaration",
            "arrow_function",
            "function_expression",
            "function",
        ],
        Lang::Python => &["function_definition"],
        Lang::C | Lang::Cpp => &["function_definition"],
        Lang::Kotlin => &[],
    }
}

fn node_name(n: Node<'_>, src: &str, lang: Lang) -> Option<String> {
    match n.kind() {
        "impl_item" => {
            let t = text(n.child_by_field_name("type")?, src);
            first_word(t).map(str::to_owned)
        }
        "method_declaration" if lang == Lang::Go => {
            let recv = n.child_by_field_name("receiver").map(|r| text(r, src))?;
            let ty = last_word(recv).unwrap_or("");
            let name = text(n.child_by_field_name("name")?, src);
            Some(format!("{ty}.{name}"))
        }
        "function_definition" if matches!(lang, Lang::C | Lang::Cpp) => {
            let d = n.child_by_field_name("declarator")?;
            let t = text(d, src);
            first_word(t.split('(').next().unwrap_or(t).rsplit(['*', ' ']).next()?)
                .map(str::to_owned)
        }
        "arrow_function" | "function_expression" | "function" => {
            let p = n.parent()?;
            match p.kind() {
                "variable_declarator" => p
                    .child_by_field_name("name")
                    .map(|c| text(c, src).to_owned()),
                "pair" => p
                    .child_by_field_name("key")
                    .map(|c| text(c, src).to_owned()),
                "assignment_expression" => p
                    .child_by_field_name("left")
                    .and_then(|c| last_word(text(c, src)))
                    .map(str::to_owned),
                _ => None,
            }
        }
        _ => n
            .child_by_field_name("name")
            .map(|c| text(c, src).to_owned()),
    }
}

/// The dotted path of enclosing classes and functions of `n` and the line range of
/// the nearest named enclosing function.
pub fn describe(n: Node<'_>, src: &str, lang: Lang) -> (String, Option<(u32, u32)>) {
    let mut names: Vec<String> = Vec::new();
    let mut range = None;
    let classes = class_kinds(lang);
    let fns = fn_kinds(lang);
    for a in ancestors(n) {
        let k = a.kind();
        if fns.contains(&k) {
            if let Some(name) = node_name(a, src, lang) {
                if range.is_none() {
                    range = Some((ast::line(a), ast::end_line(a)));
                }
                names.push(name);
            }
        } else if classes.contains(&k) {
            if let Some(name) = node_name(a, src, lang) {
                names.push(name);
            }
        } else if k == "static_initializer" {
            names.push("<clinit>".to_owned());
            if range.is_none() {
                range = Some((ast::line(a), ast::end_line(a)));
            }
        }
    }
    names.reverse();
    (names.join("."), range)
}

/// The class path (without functions) enclosing `n`.
pub fn class_path(n: Node<'_>, src: &str, lang: Lang) -> String {
    let classes = class_kinds(lang);
    let mut names: Vec<String> = ancestors(n)
        .filter(|a| classes.contains(&a.kind()))
        .filter_map(|a| node_name(a, src, lang))
        .collect();
    names.reverse();
    names.join(".")
}

/// Java: the field a call's result is assigned to when the call sits in a static
/// initializer block (`SEGMENTS = ..` or `Foo.SEGMENTS = ..`) or a constructor
/// (`this.segments = ..`); `None` anywhere else, for a local variable and for a bare name
/// in a constructor (which may be a parameter or a local).
fn initializer_store(call: Node<'_>, src: &str) -> Option<String> {
    let mut is_static = false;
    let mut block = None;
    for a in ancestors(call) {
        match a.kind() {
            "static_initializer" => {
                is_static = true;
                block = Some(a);
                break;
            }
            "constructor_declaration" => break,
            "method_declaration"
            | "lambda_expression"
            | "class_declaration"
            | "interface_declaration"
            | "enum_declaration"
            | "record_declaration" => {
                return None;
            }
            _ => {}
        }
    }
    if !is_static && !ancestors(call).any(|a| a.kind() == "constructor_declaration") {
        return None;
    }
    let mut cur = call;
    for _ in 0..4 {
        let p = cur.parent()?;
        match p.kind() {
            "assignment_expression" => {
                let left = text(p.child_by_field_name("left")?, src).trim();
                let word = last_word(left)?;
                let qualified = left.contains('.');
                let ok = if is_static {
                    // `NAME = ..` or `Cls.NAME = ..` of an enclosing class; a name the
                    // block declares itself is a local, and `obj.NAME` is another object's.
                    if qualified {
                        let qual = left.rsplit_once('.').map_or("", |(q, _)| q);
                        let cls = class_path(call, src, Lang::Java);
                        let q = qual.rsplit('.').next().unwrap_or("");
                        !q.is_empty() && cls.split('.').any(|c| c == q)
                    } else {
                        !block.is_some_and(|b| declares_local(b, word, src))
                    }
                } else {
                    left.starts_with("this.")
                };
                return ok.then(|| word.to_owned());
            }
            "parenthesized_expression" | "cast_expression" => cur = p,
            _ => return None,
        }
    }
    None
}

/// True when `block` declares a local variable, a catch parameter, a loop variable or a
/// resource called `name`.
fn declares_local(block: Node<'_>, name: &str, src: &str) -> bool {
    let mut stack = vec![block];
    while let Some(n) = stack.pop() {
        let named = match n.kind() {
            "variable_declarator"
            | "formal_parameter"
            | "catch_formal_parameter"
            | "resource"
            | "enhanced_for_statement" => n.child_by_field_name("name"),
            _ => None,
        };
        if named.is_some_and(|c| text(c, src) == name) {
            return true;
        }
        let mut cur = n.walk();
        stack.extend(n.children(&mut cur));
    }
    false
}

/// The variable a call's result is stored in, when the call is the right-hand side of
/// an assignment or declaration.
pub fn assigned_name(call: Node<'_>, src: &str) -> Option<(String, bool)> {
    let mut cur = call;
    for _ in 0..4 {
        let p = cur.parent()?;
        let lhs = match p.kind() {
            "variable_declarator" | "init_declarator" => p
                .child_by_field_name("name")
                .or_else(|| p.child_by_field_name("declarator")),
            "assignment"
            | "assignment_expression"
            | "assignment_statement"
            | "short_var_declaration" => p.child_by_field_name("left"),
            "let_declaration" => p.child_by_field_name("pattern"),
            "var_spec" => p.child_by_field_name("name"),
            "await_expression"
            | "parenthesized_expression"
            | "cast_expression"
            | "unary_expression"
            | "expression_list"
            | "try_expression"
            | "field_expression"
            | "reference_expression"
            | "call_expression"
            | "method_invocation"
            | "argument_list"
            | "arguments" => {
                cur = p;
                continue;
            }
            _ => return None,
        };
        let t = text(lhs?, src);
        let first = t.split(',').next().unwrap_or(t);
        let w = last_word(first)?;
        if w == "_" || w == "err" {
            return None;
        }
        let member = first.contains('.') || first.contains("->");
        return Some((w.to_owned(), member));
    }
    None
}

/// Removes entries that repeat the same line and holder class.
pub fn dedupe(v: Vec<Raw>) -> Vec<Raw> {
    let mut seen: HashSet<(String, u32, &'static str)> = HashSet::new();
    v.into_iter()
        .filter(|r| seen.insert((r.file.clone(), r.line, r.holder.as_str())))
        .collect()
}

/// True when `name` looks like a pointer hint of a linked structure.
pub fn link_name(name: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)^_*(head|tail|first|last|front|back|top|bottom|root|next|prev|previous|parent|children|child|cursor|chain|link|sibling|pending|queue|stack|backlog|buffer|buffers|pool|free|freelist|items|entries|cache|table)\w*$")
            .expect("static")
    })
    .is_match(name)
}

/// True when `name` is assigned (`name = ..`, not `==`) at least `min` times in `text`,
/// optionally through `prefix` (`this.`, `self.`).
pub fn assigned_times(text: &str, prefix: &str, name: &str, min: usize) -> bool {
    let re = Regex::new(&format!(
        r"{}\b{}\s*=[^=>]",
        regex::escape(prefix),
        regex::escape(name)
    ));
    re.is_ok_and(|r| r.find_iter(text).count() >= min)
}

/// Capitalised identifiers of a text (type names, in the languages that capitalise
/// them).
pub fn caps_idents(t: &str) -> Vec<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\b[A-Z][A-Za-z0-9_]*\b").expect("static"))
        .find_iter(t)
        .map(|m| m.as_str().to_owned())
        .collect()
}

/// True for the name of a function that runs once at process start.
pub fn entry_fn_name(name: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)^(?:main|init|initialize|setup|bootstrap|startup|start|serve|run|create_app|make_app|build_app|new_app|app|(?:init|setup|start|boot)_\w+)$")
            .expect("static")
    })
    .is_match(name)
}
