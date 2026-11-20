//! Retention chains: a long-lived head reference that is advanced to a new object which
//! keeps the previous head (`rec.previous = latest; latest = rec`, or
//! `latest = new Context(..., latest)`), with nothing in the project that unlinks old
//! entries. The head can be a module global, a static field or a member of a long-lived
//! object. Shared by the JVM, Node and Python rule sets; Go has its own entry point
//! ([`go_chain_growth`]) because its heads are fields of a receiver or package variables
//! and a wrapping object is built by an ordinary call, not a constructor.

use std::collections::{HashMap, HashSet};

use dla_census::parse::{Lang, tree_sitter::Node};
use dla_core::{
    finding::{Holder, Region},
    static_hit::StaticHit,
};

use super::ast::{
    UseIndex, ancestor, find_all, last_token, line, local_names, named_children, removal_is_live,
    tokens, txt, walk,
};
use super::{SrcFile, hit};

const ASSIGN_KINDS: [&str; 2] = ["assignment_expression", "assignment"];
const FN_KINDS: [&str; 7] = [
    "method_declaration",
    "function_declaration",
    "function_definition",
    "method_definition",
    "arrow_function",
    "function_expression",
    "lambda_expression",
];
const CREATION_KINDS: [&str; 6] = [
    "object_creation_expression",
    "new_expression",
    "call",
    "call_expression",
    "object",
    "dictionary",
];

fn sides<'t>(a: Node<'t>) -> Option<(Node<'t>, Node<'t>)> {
    if let (Some(l), Some(r)) = (
        a.child_by_field_name("left"),
        a.child_by_field_name("right"),
    ) {
        return Some((l, r));
    }
    let kids = named_children(a);
    match kids.as_slice() {
        [l, .., r] => Some((*l, *r)),
        _ => None,
    }
}

/// The head a left-hand side names: a bare identifier, or `this.x` / `self.x` / `Type.x`.
fn head_of(left: Node<'_>, src: &str) -> Option<(String, bool)> {
    let t = txt(left, src).trim();
    match left.kind() {
        "identifier" => Some((t.to_owned(), false)),
        "field_access" | "member_expression" | "attribute" | "navigation_expression" => {
            let (recv, name) = t.rsplit_once('.')?;
            let recv = recv.trim();
            let ok = matches!(recv, "this" | "self" | "cls")
                || recv.chars().next().is_some_and(char::is_uppercase);
            ok.then(|| (name.trim().to_owned(), true))
        }
        _ => None,
    }
}

fn is_head_expr(text: &str, name: &str) -> bool {
    let t = text.trim();
    t == name
        || t.strip_suffix(name).is_some_and(|p| {
            p.ends_with('.') && {
                let recv = p.trim_end_matches('.');
                matches!(recv, "this" | "self" | "cls")
                    || recv.chars().next().is_some_and(char::is_uppercase)
            }
        })
}

fn params_of(func: Node<'_>, src: &str) -> HashSet<String> {
    func.child_by_field_name("parameters")
        .or_else(|| func.child_by_field_name("parameter"))
        .or_else(|| {
            named_children(func)
                .into_iter()
                .find(|c| c.kind().contains("parameters"))
        })
        .map(|p| tokens(txt(p, src)).into_iter().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// Bare arguments (or object fields) of a creation expression that equal the head.
fn creation_holds_head(right: Node<'_>, src: &str, name: &str) -> bool {
    if !CREATION_KINDS.contains(&right.kind()) {
        return false;
    }
    if matches!(right.kind(), "call" | "call_expression") {
        let first = right
            .child_by_field_name("function")
            .or_else(|| named_children(right).into_iter().next())
            .map(|c| last_token(txt(c, src)))
            .unwrap_or_default();
        if !first.chars().next().is_some_and(char::is_uppercase) {
            return false;
        }
    }
    let mut found = false;
    walk(right, &mut |n| {
        if found || n.id() == right.id() {
            return;
        }
        let Some(p) = n.parent() else { return };
        let holder = matches!(
            p.kind(),
            "argument_list" | "arguments" | "value_argument" | "pair" | "value_arguments"
        );
        let direct = ancestor(n, &CREATION_KINDS).is_some_and(|a| a.id() == right.id());
        if holder && direct && is_head_expr(txt(n, src), name) {
            if p.kind() == "pair"
                && p.child_by_field_name("value")
                    .is_none_or(|v| v.id() != n.id())
            {
                return;
            }
            found = true;
        }
    });
    found
}

struct Advance {
    file: usize,
    line: u32,
    name: String,
}

/// Runs the retention-chain check over `files` (all of one runtime) and reports `rule`.
pub fn chain_growth(files: &[SrcFile], uses: &UseIndex, rule: &str) -> Vec<StaticHit> {
    let mut advances: Vec<Advance> = Vec::new();
    // `H = <derived from H>` anywhere: unlinks or pops the chain.
    let mut unlinked: HashSet<String> = HashSet::new();
    for (fi, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        let globals = if f.lang == Lang::Python {
            HashSet::new()
        } else {
            top_names(f)
        };
        for a in find_all(f.tree.root_node(), &ASSIGN_KINDS) {
            let Some((left, right)) = sides(a) else {
                continue;
            };
            let rt = txt(right, src).trim();
            let func = ancestor(a, &FN_KINDS);
            // Pop, unlink or reset of a head with this name, wherever it is written.
            let loose = last_token(txt(left, src));
            let derived = !loose.is_empty()
                && rt != loose
                && !matches!(right.kind(), "identifier")
                && tokens(rt).contains(&loose.as_str())
                && !creation_holds_head(right, src, &loose);
            let resets = func.is_some()
                && matches!(rt, "null" | "None" | "undefined" | "nil")
                && removal_is_live(f, a, uses);
            if derived || resets {
                unlinked.insert(loose);
                continue;
            }
            let Some(func) = func else { continue };
            let Some((name, member)) = head_of(left, src) else {
                continue;
            };
            if !member {
                let in_global = find_all(func, &["global_statement"])
                    .iter()
                    .any(|g| tokens(txt(*g, src)).contains(&name.as_str()));
                let local = params_of(func, src).contains(&name)
                    || (f.lang != Lang::Python && local_names(func, src, f.lang).contains(&name));
                let ok = match f.lang {
                    Lang::Python => in_global,
                    Lang::JavaScript | Lang::TypeScript | Lang::Tsx => {
                        !local && globals.contains(&name)
                    }
                    _ => !local,
                };
                if !ok {
                    continue;
                }
            }
            let mut chained = creation_holds_head(right, src, &name);
            if !chained && right.kind() == "identifier" {
                let x = rt;
                chained = find_all(func, &ASSIGN_KINDS).into_iter().any(|b| {
                    let Some((bl, br)) = sides(b) else {
                        return false;
                    };
                    is_head_expr(txt(br, src), &name)
                        && matches!(
                            bl.kind(),
                            "field_access"
                                | "member_expression"
                                | "attribute"
                                | "navigation_expression"
                        )
                        && txt(bl, src).trim().split('.').next() == Some(x)
                });
            }
            if !chained && right.kind() == "identifier" {
                // `X = new T(.., head); head = X;`
                chained = find_all(
                    func,
                    &["variable_declarator", "assignment_expression", "assignment"],
                )
                .into_iter()
                .any(|d| {
                    let (l, v) = if d.kind() == "variable_declarator" {
                        (
                            d.child_by_field_name("name"),
                            d.child_by_field_name("value"),
                        )
                    } else {
                        sides(d).map_or((None, None), |(l, r)| (Some(l), Some(r)))
                    };
                    l.is_some_and(|l| txt(l, src).trim() == rt)
                        && v.is_some_and(|v| creation_holds_head(v, src, &name))
                });
            }
            if chained {
                advances.push(Advance {
                    file: fi,
                    line: line(a),
                    name,
                });
            }
        }
    }
    advances
        .into_iter()
        .filter(|a| !unlinked.contains(&a.name))
        .map(|a| {
            hit(
                rule,
                &files[a.file],
                a.line,
                format!(
                    "`{}` is moved to a new object that keeps the previous value, and nothing in the project unlinks older entries, so the chain only grows",
                    a.name
                ),
            )
        })
        .collect()
}

/// Calls that never wrap their argument in a new object that keeps it.
const GO_PLAIN_CALLS: [&str; 14] = [
    "append", "len", "cap", "make", "new", "copy", "min", "max", "string", "int", "int64",
    "float64", "uint64", "bool",
];

/// True for a type text that can hold the value it replaces: an interface (`context.Context`,
/// `any`, one declared in the project), a func, or a pointer to a type with a field of its
/// own type. Scalars, strings, slices, maps and time types cannot. `error` is left out: an
/// error wrapped around itself (`err = fmt.Errorf("%w", err)`) is nearly always a one-off on
/// a failure path, not a chain that grows with the work.
fn go_can_hold_predecessor(
    ty: &str,
    interfaces: &HashSet<String>,
    selfref: &HashSet<String>,
) -> bool {
    let t = ty.trim();
    if t.starts_with("func(") || t.starts_with("interface") {
        return true;
    }
    if let Some(base) = t.strip_prefix('*') {
        return selfref.contains(base.trim());
    }
    matches!(t, "context.Context" | "any") || interfaces.contains(t)
}

/// The Go retention-chain check: a field of a receiver (or a package variable) is
/// reassigned from a call that takes the field's own value as an argument
/// (`p.ctx = start(p.ctx, ...)`), the field's type can hold what it replaces, and nothing in
/// the project resets or unwraps the field. Reported at the assignment.
pub fn go_chain_growth(files: &[SrcFile], rule: &str) -> Vec<StaticHit> {
    let mut field_types: HashMap<(String, String), String> = HashMap::new();
    let mut var_types: HashMap<String, String> = HashMap::new();
    let mut interfaces: HashSet<String> = HashSet::new();
    let mut selfref: HashSet<String> = HashSet::new();
    let mut type_file: HashMap<String, usize> = HashMap::new();
    for (fi, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        walk(f.tree.root_node(), &mut |n| match n.kind() {
            "type_spec" => {
                let (Some(name), Some(ty)) =
                    (n.child_by_field_name("name"), n.child_by_field_name("type"))
                else {
                    return;
                };
                let name = txt(name, src).to_owned();
                match ty.kind() {
                    "interface_type" => {
                        interfaces.insert(name);
                    }
                    "struct_type" => {
                        type_file.entry(name.clone()).or_insert(fi);
                        for fd in find_all(ty, &["field_declaration"]) {
                            let Some(t) = fd.child_by_field_name("type") else {
                                continue;
                            };
                            let tt = txt(t, src).trim().to_owned();
                            if tt.trim_start_matches(['*', '[', ']']) == name {
                                selfref.insert(name.clone());
                            }
                            let mut c = fd.walk();
                            for id in fd.children_by_field_name("name", &mut c) {
                                field_types
                                    .insert((name.clone(), txt(id, src).to_owned()), tt.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            "var_spec"
                if ancestor(
                    n,
                    &["function_declaration", "method_declaration", "func_literal"],
                )
                .is_none() =>
            {
                if let Some(t) = n.child_by_field_name("type") {
                    let mut c = n.walk();
                    for id in n.children_by_field_name("name", &mut c) {
                        var_types.insert(txt(id, src).to_owned(), txt(t, src).trim().to_owned());
                    }
                }
            }
            _ => {}
        });
    }

    struct GoAdvance {
        file: usize,
        line: u32,
        key: String,
        field: String,
        owner: String,
        holder: Holder,
    }
    let mut advances: Vec<GoAdvance> = Vec::new();
    let mut unlinked: HashSet<String> = HashSet::new();
    for (fi, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        for func in find_all(
            f.tree.root_node(),
            &["function_declaration", "method_declaration"],
        ) {
            let Some(body) = func.child_by_field_name("body") else {
                continue;
            };
            let recv = func.child_by_field_name("receiver").and_then(|r| {
                let p = named_children(r).into_iter().next()?;
                let name = p
                    .child_by_field_name("name")
                    .map(|n| txt(n, src).to_owned())?;
                let ty = txt(p.child_by_field_name("type")?, src)
                    .trim_start_matches('*')
                    .split('[')
                    .next()?
                    .to_owned();
                Some((name, ty))
            });
            let params = params_of(func, src);
            let locals = local_names(body, src, f.lang);
            for a in find_all(body, &["assignment_statement"]) {
                let (Some(l), Some(r)) = (
                    a.child_by_field_name("left"),
                    a.child_by_field_name("right"),
                ) else {
                    continue;
                };
                let (ls, rs) = (named_children(l), named_children(r));
                let (&[l], &[r]) = (ls.as_slice(), rs.as_slice()) else {
                    continue;
                };
                let lt = txt(l, src).trim();
                // The head: a field of the receiver, or a package variable.
                let (key, field, ty, owner, holder) = match l.kind() {
                    "selector_expression" => {
                        let (Some((rn, rt)), Some(op), Some(fld)) = (
                            recv.as_ref(),
                            l.child_by_field_name("operand"),
                            l.child_by_field_name("field"),
                        ) else {
                            continue;
                        };
                        if txt(op, src) != rn {
                            continue;
                        }
                        let fld = txt(fld, src).to_owned();
                        let at = type_file.get(rt).copied().unwrap_or(fi);
                        (
                            format!("{rt}.{fld}"),
                            fld.clone(),
                            field_types.get(&(rt.clone(), fld.clone())).cloned(),
                            format!(
                                "{}.{rt}.{fld}",
                                dla_census::model::module_path(&files[at].rel)
                            ),
                            Holder::ContainerField,
                        )
                    }
                    "identifier" => {
                        let name = lt.to_owned();
                        if params.contains(&name) || locals.contains(&name) {
                            continue;
                        }
                        let Some(t) = var_types.get(&name) else {
                            continue;
                        };
                        (
                            format!("var {name}"),
                            name.clone(),
                            Some(t.clone()),
                            format!("{}.{name}", dla_census::model::module_path(&f.rel)),
                            Holder::StaticGlobal,
                        )
                    }
                    _ => continue,
                };
                let rt_text = txt(r, src).trim();
                let advance = r.kind() == "call_expression" && {
                    let callee = r.child_by_field_name("function");
                    let callee_text = callee.map_or("", |c| txt(c, src));
                    let plain = GO_PLAIN_CALLS.contains(&callee_text);
                    let args = r
                        .child_by_field_name("arguments")
                        .map(named_children)
                        .unwrap_or_default();
                    // `p.ctx.Parent()` is a step back, not a wrap: the head is the receiver.
                    let on_head = callee_text.starts_with(&format!("{lt}."));
                    !plain && !on_head && args.iter().any(|x| txt(*x, src).trim() == lt)
                };
                if advance {
                    let holds = ty
                        .as_deref()
                        .is_some_and(|t| go_can_hold_predecessor(t, &interfaces, &selfref));
                    if holds {
                        advances.push(GoAdvance {
                            file: fi,
                            line: line(a),
                            key,
                            field,
                            owner,
                            holder,
                        });
                    }
                } else if rt_text == "nil"
                    || rt_text.starts_with("context.Background(")
                    || rt_text.starts_with("context.TODO(")
                    || tokens(rt_text).contains(&lt.rsplit('.').next().unwrap_or(lt))
                {
                    // A reset, or a step back (`p.ctx = p.ctx.Parent()`): the chain is bounded.
                    unlinked.insert(key);
                }
            }
        }
    }
    advances
        .into_iter()
        .filter(|a| !unlinked.contains(&a.key))
        .map(|a| {
            let mut h = hit(
                rule,
                &files[a.file],
                a.line,
                format!(
                    "`{}` is reassigned from a call that wraps its own previous value, and nothing in the project resets or unwraps it, so the chain behind it only grows",
                    a.field
                ),
            );
            h.holder = Some(a.holder);
            h.owner = Some(a.owner);
            h.region = Some(Region::Heap);
            h
        })
        .collect()
}

fn top_names(f: &SrcFile) -> HashSet<String> {
    let src = f.text.as_str();
    let mut out = HashSet::new();
    for n in named_children(f.tree.root_node()) {
        let decl = if n.kind() == "export_statement" {
            named_children(n).into_iter().next()
        } else {
            Some(n)
        };
        if let Some(d) = decl
            && matches!(d.kind(), "lexical_declaration" | "variable_declaration")
        {
            for v in named_children(d) {
                if let Some(name) = v.child_by_field_name("name") {
                    out.insert(txt(name, src).to_owned());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::dla_rules::{Runtime, analyze, parse_sources, sites};

    fn hits(rt: Runtime, sources: &[(&str, &str)]) -> Vec<dla_core::static_hit::StaticHit> {
        analyze(&parse_sources(sources), rt)
    }

    #[test]
    fn python_global_backlink_chain() {
        let leak = "_newest = None\n\ndef parse(x):\n    global _newest\n    rec = Record(x)\n    if _newest is not None:\n        rec.previous = _newest\n    rec.previous = _newest\n    _newest = rec\n    return rec\n";
        let h = hits(Runtime::Python, &[("p.py", leak)]);
        assert_eq!(sites(&h, "python-chain-head-growth"), ["p.py:9"]);
        let local = "def parse(x):\n    newest = None\n    rec = Record(x)\n    rec.previous = newest\n    newest = rec\n    return rec\n";
        assert!(hits(Runtime::Python, &[("p.py", local)]).is_empty());
        let popped =
            format!("{leak}\ndef pop():\n    global _newest\n    _newest = _newest.previous\n");
        assert!(hits(Runtime::Python, &[("p.py", &popped)]).is_empty());
    }

    #[test]
    fn python_constructor_chain_on_self() {
        let leak =
            "class Log:\n    def add(self, x):\n        self.latest = Entry(x, self.latest)\n";
        let h = hits(Runtime::Python, &[("l.py", leak)]);
        assert_eq!(sites(&h, "python-chain-head-growth"), ["l.py:3"]);
        let derived = "class Acc:\n    def add(self, x):\n        self.total = Money(self.total.amount + x)\n";
        assert!(hits(Runtime::Python, &[("l.py", derived)]).is_empty());
    }

    #[test]
    fn java_static_head_chain() {
        let leak = "class Ctx {\n  private static Ctx latest;\n  private final Ctx previous;\n  private Ctx(int n, Ctx previous) { this.previous = previous; }\n  static synchronized Ctx open(int n) {\n    Ctx c = new Ctx(n, latest);\n    latest = c;\n    return c;\n  }\n}\n";
        let h = hits(Runtime::Jvm, &[("Ctx.java", leak)]);
        assert_eq!(sites(&h, "jvm-chain-head-growth"), ["Ctx.java:7"]);
        // The constructor argument form without a local.
        let direct = "class Ctx {\n  private static Ctx latest;\n  static Ctx open(int n) {\n    latest = new Ctx(n, latest);\n    return latest;\n  }\n}\n";
        let h = hits(Runtime::Jvm, &[("Ctx.java", direct)]);
        assert_eq!(sites(&h, "jvm-chain-head-growth"), ["Ctx.java:4"]);
        let back = "class Node2 {\n  static Node2 head;\n  Node2 next;\n  static void push(Node2 n) {\n    n.next = head;\n    head = n;\n  }\n}\n";
        let h = hits(Runtime::Jvm, &[("N.java", back)]);
        assert_eq!(sites(&h, "jvm-chain-head-growth"), ["N.java:6"]);
        let popped = "class Node2 {\n  static Node2 head;\n  Node2 next;\n  static void push(Node2 n) {\n    n.next = head;\n    head = n;\n  }\n  static void pop() {\n    head = head.next;\n  }\n}\n";
        assert!(hits(Runtime::Jvm, &[("N.java", popped)]).is_empty());
    }

    #[test]
    fn node_member_and_module_chains() {
        let leak = "class Sched {\n  enqueue(job) {\n    const record = { job, previous: null };\n    record.previous = this.latest;\n    this.latest = record;\n  }\n}\n";
        let h = hits(Runtime::Node, &[("s.js", leak)]);
        assert_eq!(sites(&h, "node-chain-head-growth"), ["s.js:5"]);
        let literal = "let head = null;\nfunction push(x) {\n  head = { x, prev: head };\n}\n";
        let h = hits(Runtime::Node, &[("h.js", literal)]);
        assert_eq!(sites(&h, "node-chain-head-growth"), ["h.js:3"]);
        let local = "function f(xs) {\n  let head = null;\n  for (const x of xs) head = { x, prev: head };\n  return head;\n}\n";
        assert!(hits(Runtime::Node, &[("h.js", local)]).is_empty());
        let unlinked = format!("{leak}\nfunction trim(s) {{ s.latest = s.latest.previous; }}\n");
        assert!(hits(Runtime::Node, &[("s.js", &unlinked)]).is_empty());
    }

    #[test]
    fn go_field_wrapped_around_its_own_value() {
        let leak = "package rates\n\ntype P struct {\n\tctx context.Context\n}\n\nfunc (p *P) Lookup(pair string) {\n\tp.ctx = trace.Start(p.ctx, \"lookup\")\n}\n";
        let h = hits(Runtime::Go, &[("rates/p.go", leak)]);
        assert_eq!(sites(&h, "go-chain-head-growth"), ["rates/p.go:8"]);
        assert_eq!(h[0].holder, Some(dla_core::finding::Holder::ContainerField));
        assert_eq!(h[0].region, Some(dla_core::finding::Region::Heap));
        assert_eq!(h[0].owner.as_deref(), Some("rates.p.P.ctx"));
        // A package variable is a head as well.
        let global = "package p\n\nvar current context.Context\n\nfunc step() {\n\tcurrent = trace.Start(current, \"x\")\n}\n";
        assert_eq!(
            sites(
                &hits(Runtime::Go, &[("p/g.go", global)]),
                "go-chain-head-growth"
            ),
            ["p/g.go:6"]
        );
    }

    #[test]
    fn go_chain_is_quiet_for_locals_resets_and_other_types() {
        let local = "package p\n\nfunc f(ctx context.Context) context.Context {\n\tctx = trace.Start(ctx, \"x\")\n\treturn ctx\n}\n";
        assert!(hits(Runtime::Go, &[("p/a.go", local)]).is_empty());
        let fresh = "package p\n\ntype P struct{ ctx context.Context }\n\nfunc (p *P) Reset() {\n\tp.ctx = context.Background()\n}\n";
        assert!(hits(Runtime::Go, &[("p/a.go", fresh)]).is_empty());
        let scalars = "package p\n\ntype P struct {\n\tn     int\n\titems []int\n\tpath  string\n}\n\nfunc (p *P) f(x int, s string) {\n\tp.n = max(p.n, x)\n\tp.items = append(p.items, x)\n\tp.path = filepath.Join(p.path, s)\n}\n";
        assert!(hits(Runtime::Go, &[("p/a.go", scalars)]).is_empty());
        // An error wrapped around itself is a one-off on a failure path, not a chain.
        let err = "package p\n\ntype R struct{ err error }\n\nfunc (r *R) fail() {\n\tr.err = fmt.Errorf(\"retry: %w\", r.err)\n}\n";
        assert!(hits(Runtime::Go, &[("p/r.go", err)]).is_empty());
        // The same wrap, undone somewhere in the project, is bounded.
        let wrapped = "package p\n\ntype P struct{ ctx context.Context }\n\nfunc (p *P) Lookup() {\n\tp.ctx = trace.Start(p.ctx, \"x\")\n}\n";
        let undone =
            format!("{wrapped}\nfunc (p *P) Done() {{\n\tp.ctx = context.Background()\n}}\n");
        assert!(hits(Runtime::Go, &[("p/a.go", &undone)]).is_empty());
        let popped = format!("{wrapped}\nfunc (p *P) Back() {{\n\tp.ctx = p.ctx.Parent()\n}}\n");
        assert!(hits(Runtime::Go, &[("p/a.go", &popped)]).is_empty());
        // A context and its cancel function assigned together is a one-time derivation.
        let pair = "package p\n\ntype P struct {\n\tctx    context.Context\n\tcancel context.CancelFunc\n}\n\nfunc (p *P) Start() {\n\tp.ctx, p.cancel = context.WithCancel(p.ctx)\n}\n";
        assert!(hits(Runtime::Go, &[("p/a.go", pair)]).is_empty());
        // A struct value or a field of another receiver is not a head.
        let other = "package p\n\ntype Cfg struct{ a int }\ntype P struct{ cfg Cfg }\n\nfunc (p *P) f(q *P) {\n\tp.cfg = merge(p.cfg, 1)\n\tq.cfg = merge(q.cfg, 2)\n}\n";
        assert!(hits(Runtime::Go, &[("p/a.go", other)]).is_empty());
    }
}
