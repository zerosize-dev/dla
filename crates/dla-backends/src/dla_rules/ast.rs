//! Small helpers over tree-sitter nodes shared by the rule modules.

use std::collections::HashSet;

use dla_census::parse::{Lang, tree_sitter::Node};

use super::SrcFile;

/// Source text of a node.
pub fn txt<'a>(n: Node<'_>, src: &'a str) -> &'a str {
    src.get(n.start_byte()..n.end_byte()).unwrap_or("")
}

/// 1-based line where a node starts.
pub fn line(n: Node<'_>) -> u32 {
    u32::try_from(n.start_position().row.saturating_add(1)).unwrap_or(u32::MAX)
}

/// Pre-order walk over a node and all its descendants (named and anonymous).
pub fn walk<'t>(n: Node<'t>, f: &mut impl FnMut(Node<'t>)) {
    f(n);
    let mut c = n.walk();
    let children: Vec<Node<'t>> = n.children(&mut c).collect();
    for ch in children {
        walk(ch, f);
    }
}

/// All descendants (self included) whose kind is in `kinds`.
pub fn find_all<'t>(n: Node<'t>, kinds: &[&str]) -> Vec<Node<'t>> {
    let mut out = Vec::new();
    walk(n, &mut |x| {
        if kinds.contains(&x.kind()) {
            out.push(x);
        }
    });
    out
}

/// Nearest ancestor (excluding `n`) whose kind is in `kinds`.
pub fn ancestor<'t>(n: Node<'t>, kinds: &[&str]) -> Option<Node<'t>> {
    let mut cur = n.parent();
    while let Some(p) = cur {
        if kinds.contains(&p.kind()) {
            return Some(p);
        }
        cur = p.parent();
    }
    None
}

/// True when `outer` contains `inner` (or they are the same node).
pub fn contains(outer: Node<'_>, inner: Node<'_>) -> bool {
    outer.start_byte() <= inner.start_byte() && inner.end_byte() <= outer.end_byte()
}

/// Named children of a node.
pub fn named_children<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c).collect()
}

/// First named child of one of the kinds.
pub fn child_of_kind<'t>(n: Node<'t>, kinds: &[&str]) -> Option<Node<'t>> {
    named_children(n)
        .into_iter()
        .find(|c| kinds.contains(&c.kind()))
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Identifier-like tokens of a piece of text, in order.
pub fn tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| !is_ident_char(c))
        .filter(|t| !t.is_empty())
        .collect()
}

/// Last identifier token of an expression text such as `this.items`, `a.b()` (gives `b`),
/// `Self::CACHE` or `self.cache.lock().unwrap()` (gives `unwrap`).
pub fn last_token(text: &str) -> String {
    tokens(text).last().copied().unwrap_or("").to_owned()
}

/// Strips balanced `(...)`, `<...>` and `[...]` groups from a text, keeping the rest.
pub fn strip_groups(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '(' | '<' | '[' => depth += 1,
            ')' | '>' | ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// Name of the receiver "root" after dropping call groups: `cache.keySet().iterator()`
/// gives tokens `cache`, `keySet`, `iterator`.
pub fn recv_tokens(recv: &str) -> Vec<String> {
    tokens(&strip_groups(recv))
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// A call expression, normalised across grammars.
#[derive(Debug, Clone)]
pub struct CallSite<'t> {
    /// The call node.
    pub node: Node<'t>,
    /// Receiver text (empty for a plain function call).
    pub recv: String,
    /// Called function or method name (last path segment).
    pub name: String,
    /// Argument expression nodes.
    pub args: Vec<Node<'t>>,
}

impl CallSite<'_> {
    /// Last identifier of the receiver, ignoring call groups.
    pub fn recv_last(&self) -> String {
        last_token(&strip_groups(&self.recv))
    }

    /// Every identifier token of the receiver, ignoring call groups.
    pub fn recv_all(&self) -> Vec<String> {
        recv_tokens(&self.recv)
    }
}

fn split_callee(f: Node<'_>, src: &str) -> (String, String) {
    match f.kind() {
        "member_expression" => (
            f.child_by_field_name("object")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default(),
            f.child_by_field_name("property")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default(),
        ),
        "selector_expression" => (
            f.child_by_field_name("operand")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default(),
            f.child_by_field_name("field")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default(),
        ),
        "field_expression" => {
            let recv = f
                .child_by_field_name("value")
                .or_else(|| f.child_by_field_name("argument"))
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default();
            let name = f
                .child_by_field_name("field")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default();
            (recv, name)
        }
        "attribute" => (
            f.child_by_field_name("object")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default(),
            f.child_by_field_name("attribute")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default(),
        ),
        "navigation_expression" => {
            let kids = named_children(f);
            match kids.as_slice() {
                [recv, .., name] => (txt(*recv, src).to_owned(), txt(*name, src).to_owned()),
                _ => (String::new(), txt(f, src).to_owned()),
            }
        }
        "scoped_identifier" | "qualified_identifier" | "scoped_type_identifier" => {
            let full = txt(f, src);
            let sep = if full.contains("::") { "::" } else { "." };
            match full.rsplit_once(sep) {
                Some((a, b)) => (a.to_owned(), b.to_owned()),
                None => (String::new(), full.to_owned()),
            }
        }
        "generic_function" => match f.child_by_field_name("function") {
            Some(inner) => split_callee(inner, src),
            None => (String::new(), txt(f, src).to_owned()),
        },
        _ => (String::new(), txt(f, src).to_owned()),
    }
}

fn kotlin_args<'t>(call: Node<'t>) -> Vec<Node<'t>> {
    let mut out = Vec::new();
    for c in named_children(call) {
        match c.kind() {
            "value_arguments" => {
                for a in named_children(c) {
                    if a.kind() == "value_argument" {
                        if let Some(last) = named_children(a).last() {
                            out.push(*last);
                        }
                    } else {
                        out.push(a);
                    }
                }
            }
            "annotated_lambda" => out.push(c),
            _ => {}
        }
    }
    out
}

/// Every call expression of a file, normalised.
pub fn calls<'t>(file: &'t SrcFile, root: Node<'t>) -> Vec<CallSite<'t>> {
    let src = file.text.as_str();
    let mut out = Vec::new();
    walk(root, &mut |n| match (file.lang, n.kind()) {
        (Lang::Java, "method_invocation") => {
            let recv = n
                .child_by_field_name("object")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default();
            let name = n
                .child_by_field_name("name")
                .map(|o| txt(o, src).to_owned())
                .unwrap_or_default();
            let args = n
                .child_by_field_name("arguments")
                .map(named_children)
                .unwrap_or_default();
            out.push(CallSite {
                node: n,
                recv,
                name,
                args,
            });
        }
        (Lang::Kotlin, "call_expression") => {
            let kids = named_children(n);
            let Some(head) = kids.first() else { return };
            let (recv, name) = if head.kind() == "navigation_expression" {
                split_callee(*head, src)
            } else {
                (String::new(), txt(*head, src).to_owned())
            };
            out.push(CallSite {
                node: n,
                recv,
                name,
                args: kotlin_args(n),
            });
        }
        (Lang::Python, "call") => {
            let Some(f) = n.child_by_field_name("function") else {
                return;
            };
            let (recv, name) = split_callee(f, src);
            let args = n
                .child_by_field_name("arguments")
                .map(named_children)
                .unwrap_or_default();
            out.push(CallSite {
                node: n,
                recv,
                name,
                args,
            });
        }
        (_, "call_expression") if file.lang != Lang::Kotlin => {
            let Some(f) = n.child_by_field_name("function") else {
                return;
            };
            let (recv, name) = split_callee(f, src);
            let args = n
                .child_by_field_name("arguments")
                .map(named_children)
                .unwrap_or_default();
            out.push(CallSite {
                node: n,
                recv,
                name,
                args,
            });
        }
        _ => {}
    });
    out
}

/// True for a string literal node of any grammar.
pub fn is_string_literal(n: Node<'_>) -> bool {
    matches!(
        n.kind(),
        "string_literal"
            | "string"
            | "template_string"
            | "interpreted_string_literal"
            | "raw_string_literal"
            | "concatenated_string"
            | "character_literal"
    )
}

/// True for any literal that cannot carry request data.
pub fn is_literal(n: Node<'_>) -> bool {
    is_string_literal(n)
        || n.kind().ends_with("_literal")
        || matches!(
            n.kind(),
            "number" | "true" | "false" | "null" | "integer" | "float"
        )
}

/// Set of identifiers declared as locals (variables) inside `scope`.
pub fn local_names(scope: Node<'_>, src: &str, lang: Lang) -> HashSet<String> {
    let mut out = HashSet::new();
    let kinds: &[&str] = match lang {
        Lang::Java => &["variable_declarator"],
        Lang::Kotlin => &["variable_declaration"],
        Lang::JavaScript | Lang::TypeScript | Lang::Tsx => &["variable_declarator"],
        Lang::Go => &["short_var_declaration", "var_spec"],
        Lang::Rust => &["let_declaration"],
        Lang::Python => &["assignment"],
        Lang::C | Lang::Cpp => &["init_declarator"],
    };
    walk(scope, &mut |n| {
        if kinds.contains(&n.kind()) {
            let name_node = n
                .child_by_field_name("name")
                .or_else(|| n.child_by_field_name("left"))
                .or_else(|| n.child_by_field_name("pattern"))
                .or_else(|| n.child_by_field_name("declarator"))
                .or_else(|| named_children(n).into_iter().next());
            if let Some(nn) = name_node {
                out.insert(last_token(txt(nn, src)));
                for t in tokens(txt(nn, src)) {
                    out.insert(t.to_owned());
                }
            }
        }
    });
    out
}

/// Names of functions and methods that production code refers to: every call name and
/// every call argument that is a bare name, member access or method reference.
#[derive(Debug, Default)]
pub struct UseIndex {
    counts: std::collections::HashMap<String, usize>,
}

impl UseIndex {
    /// Builds the index over already normalised calls (one list per file).
    pub fn build(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Self {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (i, f) in files.iter().enumerate() {
            for c in &calls_by_file[i] {
                *counts.entry(c.name.clone()).or_default() += 1;
                for a in &c.args {
                    if matches!(
                        a.kind(),
                        "identifier"
                            | "member_expression"
                            | "attribute"
                            | "method_reference"
                            | "navigation_expression"
                            | "callable_reference"
                            | "selector_expression"
                            | "field_expression"
                    ) {
                        *counts.entry(last_token(txt(*a, &f.text))).or_default() += 1;
                    }
                }
            }
        }
        Self { counts }
    }

    /// How many call sites or callback references name `name`.
    pub fn uses(&self, name: &str) -> usize {
        self.counts.get(name).copied().unwrap_or(0)
    }
}

const LIFECYCLE_NAMES: [&str; 14] = [
    "close",
    "destroy",
    "shutdown",
    "dispose",
    "stop",
    "finalize",
    "cleanup",
    "terminate",
    "release",
    "__exit__",
    "__aexit__",
    "__del__",
    "onDestroy",
    "Dispose",
];

const NAMED_FN_KINDS: [&str; 6] = [
    "method_declaration",
    "function_declaration",
    "function_definition",
    "method_definition",
    "generator_function_declaration",
    "secondary_constructor",
];

const ANON_FN_KINDS: [&str; 5] = [
    "arrow_function",
    "function_expression",
    "function",
    "lambda_expression",
    "lambda_literal",
];

/// Whether a removal at `n` can run in production. It cannot when the only function around
/// it is an ordinary named function or method that nothing in the production sources calls
/// or refers to (typical for a `reset()` kept for the tests). Removals outside functions,
/// in constructors and initializers, in lifecycle methods (`close`, `destroy`, ...) and in
/// annotated or decorated functions are assumed to run.
pub fn removal_is_live(file: &SrcFile, n: Node<'_>, uses: &UseIndex) -> bool {
    let src = file.text.as_str();
    let mut cur = n.parent();
    while let Some(p) = cur {
        let kind = p.kind();
        let named = NAMED_FN_KINDS.contains(&kind);
        let anon_named = ANON_FN_KINDS.contains(&kind)
            && p.parent().is_some_and(|pp| {
                matches!(
                    pp.kind(),
                    "variable_declarator"
                        | "field_definition"
                        | "public_field_definition"
                        | "pair"
                        | "assignment_expression"
                )
            });
        if kind == "constructor_declaration"
            || kind == "secondary_constructor"
            || INIT_LIKE.contains(&kind)
        {
            return true;
        }
        if named || anon_named {
            let name = if named {
                p.child_by_field_name("name")
                    .or_else(|| child_of_kind(p, &["identifier"]))
                    .map(|x| txt(x, src).to_owned())
            } else {
                p.parent().and_then(|pp| {
                    pp.child_by_field_name("name")
                        .or_else(|| pp.child_by_field_name("property"))
                        .or_else(|| pp.child_by_field_name("key"))
                        .or_else(|| pp.child_by_field_name("left"))
                        .map(|x| last_token(txt(x, src)))
                })
            };
            let Some(name) = name else { return true };
            if LIFECYCLE_NAMES.contains(&name.as_str()) || name == "main" {
                return true;
            }
            // Annotated (Java, Kotlin) or decorated (Python) functions are driven by a framework.
            let head = txt(p, src).split(['{', '\n']).next().unwrap_or("");
            let modifiers = child_of_kind(p, &["modifiers"]).map_or("", |m| txt(m, src));
            let annotated = modifiers.contains('@')
                && !modifiers.contains("VisibleForTesting")
                && !modifiers.contains("TestOnly");
            let decorated = p
                .parent()
                .is_some_and(|pp| pp.kind() == "decorated_definition");
            let _ = head;
            if annotated || decorated {
                return true;
            }
            return uses.uses(&name) > 0;
        }
        cur = p.parent();
    }
    true
}

const INIT_LIKE: [&str; 3] = [
    "static_initializer",
    "anonymous_initializer",
    "instance_initializer",
];
