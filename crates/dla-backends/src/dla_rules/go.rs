//! Go rules: goroutines without a stop path, `time.Tick`, tickers never stopped, a slice
//! parameter kept un-copied in accumulating state while callers pass views of a larger
//! buffer, and a field wrapped around its own previous value.

use std::collections::{HashMap, HashSet};

use dla_census::{model::module_path, parse::tree_sitter::Node};
use dla_core::{
    finding::{Holder, Region},
    static_hit::StaticHit,
};

use super::ast::{
    CallSite, ancestor, calls, child_of_kind, find_all, last_token, line, named_children, tokens,
    txt, walk,
};
use super::{RuleInfo, SrcFile, hit};

const GOROUTINE: &str = "go-goroutine-no-stop-path";
const TICK: &str = "go-time-tick";
const TICKER: &str = "go-ticker-no-stop";
const VIEW: &str = "go-slice-view-retained";
const CHAIN: &str = "go-chain-head-growth";

/// Rule table (mirrors the `backend = "dla"` rows of `data/detectors/go.toml`).
pub const RULES: &[RuleInfo] = &[
    RuleInfo {
        rule: GOROUTINE,
        archetype: "L-WORKER",
        prior: 0.45,
    },
    RuleInfo {
        rule: TICK,
        archetype: "L-WORKER",
        prior: 0.6,
    },
    RuleInfo {
        rule: TICKER,
        archetype: "L-WORKER",
        prior: 0.5,
    },
    RuleInfo {
        rule: VIEW,
        archetype: "",
        prior: 0.3,
    },
    RuleInfo {
        rule: CHAIN,
        archetype: "",
        prior: 0.3,
    },
];

fn is_stop_token(t: &str) -> bool {
    let l = t.to_ascii_lowercase();
    const EXACT: [&str; 12] = [
        "done", "stop", "quit", "cancel", "shutdown", "closing", "closed", "exit", "kill", "term",
        "abort", "finish",
    ];
    EXACT.contains(&l.as_str())
        || [
            "done", "stop", "quit", "cancel", "shutdown", "closing", "closed", "abort",
        ]
        .iter()
        .any(|s| l.ends_with(s) || l.starts_with(s))
}

fn has_stop_signal(n: Node<'_>, src: &str) -> bool {
    let t = txt(n, src);
    t.contains(".Err()") || tokens(t).into_iter().any(is_stop_token)
}

/// True when the subtree can leave the enclosing loop (return, labelled or direct break,
/// goto, panic or a process exit). `nested` is true inside a select, switch or inner loop,
/// where a bare `break` only leaves that construct.
fn can_exit(n: Node<'_>, src: &str, nested: bool) -> bool {
    let mut c = n.walk();
    let kids: Vec<Node<'_>> = n.children(&mut c).collect();
    for k in kids {
        match k.kind() {
            "return_statement" | "goto_statement" => return true,
            "break_statement" => {
                if !nested || child_of_kind(k, &["label_name"]).is_some() {
                    return true;
                }
            }
            "call_expression" => {
                let f = k
                    .child_by_field_name("function")
                    .map_or("", |f| txt(f, src));
                if matches!(
                    f,
                    "panic"
                        | "os.Exit"
                        | "log.Fatal"
                        | "log.Fatalf"
                        | "log.Fatalln"
                        | "runtime.Goexit"
                ) {
                    return true;
                }
                if can_exit(k, src, nested) {
                    return true;
                }
            }
            "func_literal" => {}
            "select_statement"
            | "expression_switch_statement"
            | "type_switch_statement"
            | "for_statement" => {
                if can_exit(k, src, true) {
                    return true;
                }
            }
            _ => {
                if can_exit(k, src, nested) {
                    return true;
                }
            }
        }
    }
    false
}

/// For loops of a body, not descending into nested function literals.
fn loops_of<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    let mut c = n.walk();
    let kids: Vec<Node<'t>> = n.children(&mut c).collect();
    for k in kids {
        match k.kind() {
            "func_literal" => {}
            "for_statement" => {
                out.push(k);
                loops_of(k, out);
            }
            _ => loops_of(k, out),
        }
    }
}

struct Index<'t> {
    funcs: HashMap<String, Vec<(usize, Node<'t>)>>,
    chans: HashSet<String>,
    closed: HashSet<String>,
    /// Function name to its parameter names, in order (variadics and unnamed ones included as "").
    params: HashMap<String, Vec<String>>,
    /// Every call: callee name, last token of each argument, and the enclosing function name.
    call_args: Vec<(String, Vec<String>, String)>,
}

fn param_names(func: Node<'_>, src: &str) -> Vec<String> {
    let Some(list) = func.child_by_field_name("parameters") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in named_children(list) {
        let names: Vec<String> = named_children(p)
            .into_iter()
            .filter(|c| c.kind() == "identifier")
            .map(|c| txt(c, src).to_owned())
            .collect();
        if names.is_empty() {
            out.push(String::new());
        } else {
            out.extend(names);
        }
    }
    out
}

fn enclosing_func_name(n: Node<'_>, src: &str) -> String {
    ancestor(n, &["function_declaration", "method_declaration"])
        .and_then(|f| f.child_by_field_name("name"))
        .map(|nm| txt(nm, src).to_owned())
        .unwrap_or_default()
}

impl Index<'_> {
    /// True when `name`, used inside function `func`, is closed somewhere in the project,
    /// directly or under another name at a call site where it is passed as an argument.
    fn is_closed(&self, name: &str, func: &str, depth: u8) -> bool {
        if self.closed.contains(name) {
            return true;
        }
        if depth == 0 || func.is_empty() {
            return false;
        }
        let Some(pos) = self
            .params
            .get(func)
            .and_then(|ps| ps.iter().position(|p| p == name))
        else {
            return false;
        };
        self.call_args.iter().any(|(callee, args, caller)| {
            callee == func
                && args
                    .get(pos)
                    .is_some_and(|a| self.is_closed(a, caller, depth - 1))
        })
    }
}

fn build_index<'t>(files: &'t [SrcFile], calls_by_file: &[Vec<CallSite<'t>>]) -> Index<'t> {
    let mut funcs: HashMap<String, Vec<(usize, Node<'t>)>> = HashMap::new();
    let mut chans = HashSet::new();
    let mut closed = HashSet::new();
    let mut params: HashMap<String, Vec<String>> = HashMap::new();
    let mut call_args = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        walk(f.tree.root_node(), &mut |n| match n.kind() {
            "function_declaration" | "method_declaration" => {
                if let (Some(name), Some(body)) =
                    (n.child_by_field_name("name"), n.child_by_field_name("body"))
                {
                    params
                        .entry(txt(name, src).to_owned())
                        .or_insert_with(|| param_names(n, src));
                    funcs
                        .entry(txt(name, src).to_owned())
                        .or_default()
                        .push((i, body));
                }
            }
            "short_var_declaration" | "assignment_statement" | "var_spec" => {
                let t = txt(n, src);
                if t.contains("make(chan") || t.contains("make(<-chan") {
                    let left = n
                        .child_by_field_name("left")
                        .or_else(|| n.child_by_field_name("name"))
                        .or_else(|| named_children(n).into_iter().next());
                    if let Some(l) = left {
                        for tok in tokens(txt(l, src)) {
                            chans.insert(tok.to_owned());
                        }
                    }
                }
                if n.kind() == "var_spec"
                    && !find_all(n, &["channel_type"]).is_empty()
                    && let Some(l) = named_children(n).first()
                {
                    for tok in tokens(txt(*l, src)) {
                        chans.insert(tok.to_owned());
                    }
                }
            }
            "parameter_declaration" | "field_declaration"
                if child_of_kind(n, &["channel_type"]).is_some() =>
            {
                for ch in named_children(n) {
                    if matches!(ch.kind(), "identifier" | "field_identifier") {
                        chans.insert(txt(ch, src).to_owned());
                    }
                }
            }
            _ => {}
        });
        for c in &calls_by_file[i] {
            call_args.push((
                c.name.clone(),
                c.args.iter().map(|a| last_token(txt(*a, src))).collect(),
                enclosing_func_name(c.node, src),
            ));
            if c.name == "close"
                && c.recv.is_empty()
                && let Some(a) = c.args.first()
            {
                closed.insert(last_token(txt(*a, src)));
            }
        }
    }
    Index {
        funcs,
        chans,
        closed,
        params,
        call_args,
    }
}

/// Describes why a loop never ends, or `None` when it has a stop path.
fn leaky_loop(l: Node<'_>, src: &str, idx: &Index<'_>) -> Option<String> {
    let kids = named_children(l);
    let header = kids.iter().find(|k| k.kind() != "block").copied();
    let body = l.child_by_field_name("body");
    match header {
        None => {
            let body = body?;
            let has_select = !find_all(body, &["select_statement"]).is_empty();
            if has_stop_signal(body, src) {
                return None;
            }
            if !has_select && can_exit(body, src, false) {
                return None;
            }
            Some(if has_select {
                "`for { select { ... } }` with no done, context or stop case".to_owned()
            } else {
                "infinite `for` loop with no return, break, done or context check".to_owned()
            })
        }
        Some(h) if h.kind() == "range_clause" => {
            let range_expr = h
                .child_by_field_name("right")
                .map_or_else(|| txt(h, src).to_owned(), |r| txt(r, src).to_owned());
            let name = last_token(&range_expr);
            let ticking = range_expr.contains("time.Tick(") || range_expr.ends_with(".C");
            if ticking {
                let body = body?;
                if has_stop_signal(body, src) || can_exit(body, src, false) {
                    return None;
                }
                return Some("ranges over a ticker with no stop path".to_owned());
            }
            let func = enclosing_func_name(l, src);
            if idx.chans.contains(&name) && !idx.is_closed(&name, &func, 4) {
                if body.is_some_and(|b| has_stop_signal(b, src) && can_exit(b, src, false)) {
                    return None;
                }
                return Some(format!(
                    "ranges over channel `{name}` that is never closed in the project"
                ));
            }
            None
        }
        _ => None,
    }
}

fn goroutines(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let idx = build_index(files, calls_by_file);
    let mut out = Vec::new();
    for (fi, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        for g in find_all(f.tree.root_node(), &["go_statement"]) {
            let Some(call) = child_of_kind(g, &["call_expression"]) else {
                continue;
            };
            let Some(callee) = call.child_by_field_name("function") else {
                continue;
            };
            let bodies: Vec<(usize, Node<'_>)> = if callee.kind() == "func_literal" {
                callee
                    .child_by_field_name("body")
                    .map(|b| (fi, b))
                    .into_iter()
                    .collect()
            } else {
                let name = last_token(txt(callee, src));
                idx.funcs.get(&name).cloned().unwrap_or_default()
            };
            if bodies.is_empty() {
                continue;
            }
            let mut reason = None;
            let mut all_leaky = true;
            for (bi, b) in &bodies {
                let mut loops = Vec::new();
                loops_of(*b, &mut loops);
                let bsrc = files[*bi].text.as_str();
                let found = loops
                    .iter()
                    .find_map(|l| leaky_loop(*l, bsrc, &idx).map(|r| (line(*l), r)));
                match found {
                    Some((ln, why)) => reason = Some((ln, why)),
                    None => all_leaky = false,
                }
            }
            if let (true, Some((ln, why))) = (all_leaky, reason) {
                out.push(hit(
                    GOROUTINE,
                    f,
                    line(g),
                    format!("goroutine has no stop path: loop at line {ln} {why}"),
                ));
            }
        }
    }
    out
}

fn tickers(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let mut out = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        let stopped: HashSet<String> = calls_by_file[i]
            .iter()
            .filter(|c| c.name == "Stop")
            .map(CallSite::recv_last)
            .collect();
        for c in &calls_by_file[i] {
            if c.recv == "time" && c.name == "Tick" {
                out.push(hit(
                    TICK,
                    f,
                    line(c.node),
                    "time.Tick cannot be stopped, so its ticker is never released; use time.NewTicker and Stop",
                ));
            }
            if c.recv == "time" && c.name == "NewTicker" {
                let Some(p) = c.node.parent() else { continue };
                let holder = match p.kind() {
                    "expression_list" => p.parent(),
                    _ => Some(p),
                };
                let Some(h) = holder else { continue };
                if !matches!(
                    h.kind(),
                    "short_var_declaration" | "assignment_statement" | "var_spec"
                ) {
                    continue;
                }
                let left = h
                    .child_by_field_name("left")
                    .or_else(|| named_children(h).into_iter().next());
                let Some(l) = left else { continue };
                let name = last_token(txt(l, src));
                if name.is_empty() || name == "_" || stopped.contains(&name) {
                    continue;
                }
                out.push(hit(
                    TICKER,
                    f,
                    line(c.node),
                    format!("ticker `{name}` is created here and `{name}.Stop()` is never called in this file"),
                ));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------
// go-slice-view-retained
// ---------------------------------------------------------------------------------------

/// Lower-case prefixes of method names that insert their argument into a container.
/// `Write*` is not here: the writers of the standard library copy.
const INSERT_PREFIX: [&str; 16] = [
    "push",
    "add",
    "put",
    "store",
    "insert",
    "append",
    "enqueue",
    "offer",
    "register",
    "remember",
    "record",
    "track",
    "intern",
    "memoize",
    "attach",
    "subscribe",
];

/// Method names that insert only when they are exactly the word: `Set(k, p)` is a map-like
/// setter, `SetBytes(p)` or `updateFrom(p)` are conversions of the receiver's own type.
const INSERT_EXACT: [&str; 6] = ["set", "update", "save", "keep", "retain", "cache"];

/// `bytes` functions whose result points into their argument.
const VIEW_FUNCS: [&str; 18] = [
    "Fields",
    "FieldsFunc",
    "Split",
    "SplitN",
    "SplitAfter",
    "SplitAfterN",
    "Trim",
    "TrimSpace",
    "TrimLeft",
    "TrimRight",
    "TrimPrefix",
    "TrimSuffix",
    "TrimFunc",
    "TrimLeftFunc",
    "TrimRightFunc",
    "Cut",
    "CutPrefix",
    "CutSuffix",
];

/// Methods whose result is a window into the receiver's own buffer.
const VIEW_METHODS: [&str; 4] = ["Bytes", "ReadSlice", "Peek", "Next"];

/// One function or method of the project.
struct Func<'t> {
    file: usize,
    name: String,
    body: Node<'t>,
    /// Receiver name and type of a method.
    recv: Option<(String, String)>,
    /// Parameter names (empty for an unnamed one) and type texts, in order.
    params: Vec<(String, String)>,
}

fn typed_params(func: Node<'_>, src: &str) -> Vec<(String, String)> {
    let Some(list) = func.child_by_field_name("parameters") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in named_children(list) {
        let ty = p
            .child_by_field_name("type")
            .map(|t| txt(t, src).to_owned())
            .unwrap_or_default();
        let ty = if p.kind() == "variadic_parameter_declaration" {
            format!("...{ty}")
        } else {
            ty
        };
        let mut c = p.walk();
        let names: Vec<String> = p
            .children_by_field_name("name", &mut c)
            .map(|n| txt(n, src).to_owned())
            .collect();
        if names.is_empty() {
            out.push((String::new(), ty));
        } else {
            out.extend(names.into_iter().map(|n| (n, ty.clone())));
        }
    }
    out
}

fn receiver_of(func: Node<'_>, src: &str) -> Option<(String, String)> {
    let list = func.child_by_field_name("receiver")?;
    let p = named_children(list).into_iter().next()?;
    let name = p
        .child_by_field_name("name")
        .map(|n| txt(n, src).to_owned())
        .unwrap_or_default();
    let ty = p.child_by_field_name("type").map_or("", |t| txt(t, src));
    let ty = ty.trim_start_matches('*');
    Some((name, ty.split('[').next().unwrap_or(ty).to_owned()))
}

/// The single expression of an `expression_list`, or the node itself.
fn exprs<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    if n.kind() == "expression_list" {
        named_children(n)
    } else {
        vec![n]
    }
}

fn short_text(t: &str) -> String {
    let one: String = t.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > 48 {
        let cut: String = one.chars().take(45).collect();
        format!("{cut}...")
    } else {
        one
    }
}

/// Where in the project a view of a larger buffer is made: `bytes.Cut` at `a.go:7`.
fn view_origin(
    e: Node<'_>,
    src: &str,
    rel: &str,
    locals: &HashMap<String, String>,
    fields: &HashMap<String, String>,
    statics: &HashSet<String>,
) -> Option<String> {
    let at = || format!("{rel}:{}", line(e));
    match e.kind() {
        "parenthesized_expression" => {
            let inner = named_children(e).into_iter().next()?;
            view_origin(inner, src, rel, locals, fields, statics)
        }
        "slice_expression" => {
            // A window of a package-level table pins nothing that is not already there for
            // the whole life of the process.
            let operand = e.child_by_field_name("operand")?;
            if operand.kind() == "identifier"
                && statics.contains(txt(operand, src))
                && !locals.contains_key(txt(operand, src))
            {
                return None;
            }
            Some(format!(
                "sub-slice `{}` at {}",
                short_text(txt(e, src)),
                at()
            ))
        }
        "call_expression" => {
            let f = e.child_by_field_name("function")?;
            if f.kind() != "selector_expression" {
                return None;
            }
            let operand = txt(f.child_by_field_name("operand")?, src);
            let name = txt(f.child_by_field_name("field")?, src);
            if operand == "bytes" && VIEW_FUNCS.contains(&name) {
                Some(format!("`bytes.{name}` at {}", at()))
            } else if operand != "bytes" && VIEW_METHODS.contains(&name) {
                Some(format!("`{operand}.{name}()` at {}", at()))
            } else {
                None
            }
        }
        "index_expression" => {
            let o = e.child_by_field_name("operand")?;
            view_origin(o, src, rel, locals, fields, statics)
        }
        "identifier" => locals.get(txt(e, src)).cloned(),
        "selector_expression" => {
            let f = txt(e.child_by_field_name("field")?, src);
            fields.get(f).map(|o| format!("field `{f}`, set from {o}"))
        }
        _ => None,
    }
}

/// Locals of a function body assigned from a view (`a, b, ok := bytes.Cut(..)`,
/// `parts := bytes.Fields(..)`, `for _, f := range parts`), in source order.
fn local_views(
    body: Node<'_>,
    src: &str,
    rel: &str,
    fields: &HashMap<String, String>,
    statics: &HashSet<String>,
) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    for st in find_all(
        body,
        &[
            "short_var_declaration",
            "assignment_statement",
            "range_clause",
        ],
    ) {
        let (Some(l), Some(r)) = (
            st.child_by_field_name("left"),
            st.child_by_field_name("right"),
        ) else {
            continue;
        };
        let (ls, rs) = (exprs(l), exprs(r));
        if st.kind() == "range_clause" {
            if let (Some(rx), Some(v)) = (rs.first(), ls.get(1))
                && v.kind() == "identifier"
            {
                let name = txt(*v, src).to_owned();
                match view_origin(*rx, src, rel, &out, fields, statics) {
                    Some(o) => out.insert(name, o),
                    None => out.remove(&name),
                };
            }
            continue;
        }
        let pairs: Vec<(Node<'_>, Node<'_>)> = if rs.len() == 1 {
            ls.iter().map(|x| (*x, rs[0])).collect()
        } else if ls.len() == rs.len() {
            ls.iter().copied().zip(rs.iter().copied()).collect()
        } else {
            continue;
        };
        for (a, b) in pairs {
            if a.kind() != "identifier" {
                continue;
            }
            let name = txt(a, src).to_owned();
            match view_origin(b, src, rel, &out, fields, statics) {
                Some(o) => out.insert(name, o),
                None => out.remove(&name),
            };
        }
    }
    out
}

/// The expression a `literal_element` wraps (or the node itself).
fn element_value(n: Node<'_>) -> Node<'_> {
    if n.kind() == "literal_element" {
        named_children(n).into_iter().next().unwrap_or(n)
    } else {
        n
    }
}

/// True when `e` stores the slice `p` itself: the bare name, a sub-slice of it, or a
/// composite literal (optionally behind `&`) with it as an element. A call (`bytes.Clone`,
/// `string`, `append([]byte(nil), p...)`) copies, so it never holds.
fn holds(e: Node<'_>, p: &str, src: &str) -> bool {
    match e.kind() {
        "identifier" => txt(e, src) == p,
        "parenthesized_expression" | "literal_element" => named_children(e)
            .into_iter()
            .next()
            .is_some_and(|c| holds(c, p, src)),
        "slice_expression" => e
            .child_by_field_name("operand")
            .is_some_and(|o| holds(o, p, src)),
        "unary_expression" => e
            .child_by_field_name("operand")
            .is_some_and(|o| holds(o, p, src)),
        "composite_literal" => e
            .child_by_field_name("body")
            .is_some_and(|b| holds(b, p, src)),
        "literal_value" => named_children(e).into_iter().any(|k| holds(k, p, src)),
        "keyed_element" => e
            .child_by_field_name("value")
            .is_some_and(|v| holds(element_value(v), p, src)),
        _ => false,
    }
}

/// Root identifier of a selector or index chain, and the field right after it.
fn chain_root(n: Node<'_>, src: &str) -> Option<(String, Option<String>)> {
    let mut cur = n;
    let mut first = None;
    loop {
        match cur.kind() {
            "identifier" => return Some((txt(cur, src).to_owned(), first)),
            "selector_expression" => {
                first = Some(txt(cur.child_by_field_name("field")?, src).to_owned());
                cur = cur.child_by_field_name("operand")?;
            }
            "index_expression" | "unary_expression" => {
                cur = cur.child_by_field_name("operand")?;
            }
            "parenthesized_expression" => cur = named_children(cur).into_iter().next()?,
            _ => return None,
        }
    }
}

/// One un-copied store of a parameter into receiver or package state.
struct Store {
    line: u32,
    param: usize,
    /// Container text (`x.order`).
    container: String,
    holder: Holder,
    /// Name of the receiver field or package variable, and the receiver type.
    field: String,
    recv_type: Option<String>,
}

struct ViewIndex<'t> {
    files: &'t [SrcFile],
    funcs: Vec<Func<'t>>,
    /// Struct type name to the file that declares it.
    type_file: HashMap<String, usize>,
    pkg_vars: HashSet<String>,
    /// Field names assigned from a view, with the origin.
    fields: HashMap<String, String>,
    /// View locals of each function, aligned with `funcs`.
    locals: Vec<HashMap<String, String>>,
    /// Callee name to `(file, index into that file's calls)`.
    callers: HashMap<String, Vec<(usize, usize)>>,
}

fn is_top_level(n: Node<'_>) -> bool {
    !crate::dla_rules::ast::ancestor(
        n,
        &["function_declaration", "method_declaration", "func_literal"],
    )
    .is_some()
}

impl<'t> ViewIndex<'t> {
    fn build(files: &'t [SrcFile], calls_by_file: &[Vec<CallSite<'t>>]) -> Self {
        let mut funcs = Vec::new();
        let mut type_file = HashMap::new();
        let mut pkg_vars = HashSet::new();
        for (i, f) in files.iter().enumerate() {
            let src = f.text.as_str();
            walk(f.tree.root_node(), &mut |n| match n.kind() {
                "function_declaration" | "method_declaration" => {
                    if let (Some(name), Some(body)) =
                        (n.child_by_field_name("name"), n.child_by_field_name("body"))
                    {
                        funcs.push(Func {
                            file: i,
                            name: txt(name, src).to_owned(),
                            body,
                            recv: receiver_of(n, src),
                            params: typed_params(n, src),
                        });
                    }
                }
                "type_spec" => {
                    if let (Some(name), Some(ty)) =
                        (n.child_by_field_name("name"), n.child_by_field_name("type"))
                        && ty.kind() == "struct_type"
                    {
                        type_file.entry(txt(name, src).to_owned()).or_insert(i);
                    }
                }
                "var_spec" if is_top_level(n) => {
                    let mut c = n.walk();
                    for id in n.children_by_field_name("name", &mut c) {
                        pkg_vars.insert(txt(id, src).to_owned());
                    }
                }
                _ => {}
            });
        }
        let mut callers: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
        for (fi, cs) in calls_by_file.iter().enumerate() {
            for (ci, c) in cs.iter().enumerate() {
                callers.entry(c.name.clone()).or_default().push((fi, ci));
            }
        }
        let mut ix = Self {
            files,
            funcs,
            type_file,
            pkg_vars,
            fields: HashMap::new(),
            locals: Vec::new(),
            callers,
        };
        ix.settle_views();
        ix
    }

    /// Finds the struct fields that are assigned from a view, with the view locals of every
    /// function. A field can be set from a local that is itself set from a field, so the
    /// two are iterated until nothing new appears (at most a few rounds).
    fn settle_views(&mut self) {
        for _ in 0..4 {
            let mut locals = Vec::with_capacity(self.funcs.len());
            let mut fields = self.fields.clone();
            for f in &self.funcs {
                let file = &self.files[f.file];
                let src = file.text.as_str();
                let lv = local_views(f.body, src, &file.rel, &self.fields, &self.pkg_vars);
                for a in find_all(f.body, &["assignment_statement"]) {
                    let (Some(l), Some(r)) = (
                        a.child_by_field_name("left"),
                        a.child_by_field_name("right"),
                    ) else {
                        continue;
                    };
                    let (ls, rs) = (exprs(l), exprs(r));
                    let (Some(l), Some(r)) = (ls.first(), rs.first()) else {
                        continue;
                    };
                    if ls.len() != 1 || rs.len() != 1 || l.kind() != "selector_expression" {
                        continue;
                    }
                    // `x.buf = x.buf[:0]` keeps the same buffer: not a view of another one.
                    if r.kind() == "slice_expression"
                        && r.child_by_field_name("operand")
                            .is_some_and(|o| txt(o, src) == txt(*l, src))
                    {
                        continue;
                    }
                    if let (Some(o), Some(name)) = (
                        view_origin(*r, src, &file.rel, &lv, &self.fields, &self.pkg_vars),
                        l.child_by_field_name("field"),
                    ) {
                        fields.entry(txt(name, src).to_owned()).or_insert(o);
                    }
                }
                for k in find_all(f.body, &["keyed_element"]) {
                    let (Some(key), Some(val)) =
                        (k.child_by_field_name("key"), k.child_by_field_name("value"))
                    else {
                        continue;
                    };
                    let key = element_value(key);
                    if key.kind() != "identifier" {
                        continue;
                    }
                    if let Some(o) = view_origin(
                        element_value(val),
                        src,
                        &file.rel,
                        &lv,
                        &self.fields,
                        &self.pkg_vars,
                    ) {
                        fields.entry(txt(key, src).to_owned()).or_insert(o);
                    }
                }
                locals.push(lv);
            }
            let grew = fields.len() != self.fields.len();
            self.fields = fields;
            self.locals = locals;
            if !grew {
                break;
            }
        }
    }

    /// The function whose body holds `n`.
    fn enclosing(&self, file: usize, n: Node<'_>) -> Option<usize> {
        let f = ancestor(n, &["function_declaration", "method_declaration"])?;
        let body = f.child_by_field_name("body")?;
        self.funcs
            .iter()
            .position(|g| g.file == file && g.body.id() == body.id())
    }

    /// A call site that passes a view of a larger buffer as argument `pos` of the function
    /// `fi`, directly or through a parameter of its own function (a couple of hops). Calls
    /// are matched by name, so a call must also agree with the function in the number of
    /// arguments, and a method is only reached by a call with a receiver (`x.Note(..)`).
    fn feed(
        &self,
        calls_by_file: &[Vec<CallSite<'_>>],
        fi: usize,
        pos: usize,
        depth: u8,
        seen: &mut HashSet<(usize, usize)>,
    ) -> Option<(String, String, String)> {
        if !seen.insert((fi, pos)) {
            return None;
        }
        let target = &self.funcs[fi];
        for (cfile, ci) in self.callers.get(&target.name)? {
            let c = &calls_by_file[*cfile][*ci];
            if c.args.len() != target.params.len() || (target.recv.is_some() && c.recv.is_empty()) {
                continue;
            }
            let Some(arg) = c.args.get(pos) else { continue };
            let file = &self.files[*cfile];
            let src = file.text.as_str();
            let enc = self.enclosing(*cfile, c.node);
            let none = HashMap::new();
            let locals = enc.map_or(&none, |g| &self.locals[g]);
            if let Some(o) = view_origin(*arg, src, &file.rel, locals, &self.fields, &self.pkg_vars)
            {
                let site = format!("{}:{}", file.rel, line(c.node));
                let call = format!("{}({})", target.name, short_text(txt(*arg, src)));
                return Some((site, call, o));
            }
            if depth > 0
                && arg.kind() == "identifier"
                && let Some(g) = enc
                && let Some(j) = self.funcs[g]
                    .params
                    .iter()
                    .position(|(n, _)| n == txt(*arg, src))
                && let Some(found) = self.feed(calls_by_file, g, j, depth - 1, seen)
            {
                return Some(found);
            }
        }
        None
    }

    /// Stores of slice parameters of `f`, un-copied, into receiver or package state.
    fn stores(&self, fi: usize) -> Vec<Store> {
        let f = &self.funcs[fi];
        let file = &self.files[f.file];
        let src = file.text.as_str();
        let params: HashSet<&str> = f.params.iter().map(|(n, _)| n.as_str()).collect();
        let locals = crate::dla_rules::ast::local_names(f.body, src, file.lang);
        let state = |n: Node<'_>| -> Option<(Holder, String, Option<String>)> {
            let (root, first) = chain_root(n, src)?;
            if f.recv
                .as_ref()
                .is_some_and(|(r, _)| !r.is_empty() && *r == root)
            {
                let field = first?;
                return Some((
                    Holder::ContainerField,
                    field,
                    f.recv.as_ref().map(|(_, t)| t.clone()),
                ));
            }
            (self.pkg_vars.contains(&root)
                && !params.contains(root.as_str())
                && !locals.contains(&root))
            .then_some((Holder::StaticGlobal, root, None))
        };
        let mut out = Vec::new();
        for (pi, (p, ty)) in f.params.iter().enumerate() {
            if p.is_empty() || p == "_" || !ty.starts_with("[]") {
                continue;
            }
            // The parameter is replaced by a copy (or anything else) before it is stored.
            let reassigned = find_all(f.body, &["assignment_statement", "short_var_declaration"])
                .into_iter()
                .filter_map(|a| a.child_by_field_name("left"))
                .any(|l| {
                    exprs(l)
                        .into_iter()
                        .any(|x| x.kind() == "identifier" && txt(x, src) == p)
                });
            if reassigned {
                continue;
            }
            for a in find_all(f.body, &["assignment_statement"]) {
                let (Some(l), Some(r)) = (
                    a.child_by_field_name("left"),
                    a.child_by_field_name("right"),
                ) else {
                    continue;
                };
                let (ls, rs) = (exprs(l), exprs(r));
                if ls.len() != 1 || rs.len() != 1 {
                    continue;
                }
                let (l, r) = (ls[0], rs[0]);
                let append_arg = (r.kind() == "call_expression")
                    .then(|| {
                        let f = r.child_by_field_name("function")?;
                        let args = named_children(r.child_by_field_name("arguments")?);
                        (txt(f, src) == "append"
                            && args.len() >= 2
                            && txt(args[0], src) == txt(l, src))
                        .then_some(args)
                    })
                    .flatten();
                let stored = if let Some(args) = &append_arg {
                    args[1..]
                        .iter()
                        .any(|x| x.kind() != "variadic_argument" && holds(*x, p, src))
                } else {
                    l.kind() == "index_expression" && holds(r, p, src)
                };
                if !stored {
                    continue;
                }
                let target = if l.kind() == "index_expression" {
                    l.child_by_field_name("operand").unwrap_or(l)
                } else {
                    l
                };
                if let Some((holder, field, recv_type)) = state(l) {
                    out.push(Store {
                        line: line(a),
                        param: pi,
                        container: short_text(txt(target, src)),
                        holder,
                        field,
                        recv_type,
                    });
                }
            }
            for c in find_all(f.body, &["call_expression"]) {
                let Some(callee) = c.child_by_field_name("function") else {
                    continue;
                };
                if callee.kind() != "selector_expression" {
                    continue;
                }
                let (Some(operand), Some(name)) = (
                    callee.child_by_field_name("operand"),
                    callee.child_by_field_name("field"),
                ) else {
                    continue;
                };
                let lname = txt(name, src).to_ascii_lowercase();
                if !INSERT_PREFIX.iter().any(|pre| lname.starts_with(pre))
                    && !INSERT_EXACT.contains(&lname.as_str())
                {
                    continue;
                }
                let Some(args) = c.child_by_field_name("arguments").map(named_children) else {
                    continue;
                };
                if !args
                    .iter()
                    .any(|x| x.kind() != "variadic_argument" && holds(*x, p, src))
                {
                    continue;
                }
                // The receiver itself (`x.Add(p)`) is a call into the type's own method.
                if operand.kind() == "identifier"
                    && f.recv.as_ref().is_some_and(|(r, _)| r == txt(operand, src))
                {
                    continue;
                }
                if let Some((holder, field, recv_type)) = state(operand) {
                    out.push(Store {
                        line: line(c),
                        param: pi,
                        container: short_text(txt(operand, src)),
                        holder,
                        field,
                        recv_type,
                    });
                }
            }
        }
        out
    }

    fn owner(&self, f: &Func<'_>, s: &Store) -> String {
        match &s.recv_type {
            Some(t) => {
                let at = self.type_file.get(t).copied().unwrap_or(f.file);
                format!("{}.{t}.{}", module_path(&self.files[at].rel), s.field)
            }
            None => format!("{}.{}", module_path(&self.files[f.file].rel), s.field),
        }
    }
}

fn slice_views(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let ix = ViewIndex::build(files, calls_by_file);
    let mut out = Vec::new();
    for (fi, f) in ix.funcs.iter().enumerate() {
        for s in ix.stores(fi) {
            let mut seen = HashSet::new();
            let Some((site, call, origin)) = ix.feed(calls_by_file, fi, s.param, 2, &mut seen)
            else {
                continue;
            };
            let param = &f.params[s.param].0;
            let mut h = hit(
                VIEW,
                &files[f.file],
                s.line,
                format!(
                    "slice parameter `{param}` of `{}` is stored without a copy in `{}`, and a caller passes a view of a larger buffer ({call} at {site}, from {origin}), so each stored entry keeps its whole source buffer alive",
                    f.name, s.container
                ),
            );
            h.holder = Some(s.holder);
            h.owner = Some(ix.owner(f, &s));
            h.region = Some(Region::Heap);
            out.push(h);
        }
    }
    out
}

/// Runs the Go rules over the Go files.
pub fn check(files: &[SrcFile]) -> Vec<StaticHit> {
    let calls_by_file: Vec<Vec<CallSite<'_>>> =
        files.iter().map(|f| calls(f, f.tree.root_node())).collect();
    let mut hits = goroutines(files, &calls_by_file);
    hits.extend(tickers(files, &calls_by_file));
    hits.extend(slice_views(files, &calls_by_file));
    hits.extend(super::chain::go_chain_growth(files, CHAIN));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dla_rules::{parse_sources, sites};

    fn run(sources: &[(&str, &str)]) -> Vec<StaticHit> {
        check(&parse_sources(sources))
    }

    #[test]
    fn select_loop_without_stop_case_is_flagged() {
        let src = "package p\n\nfunc start(in chan int) {\n\tgo func() {\n\t\tfor {\n\t\t\tselect {\n\t\t\tcase v := <-in:\n\t\t\t\tuse(v)\n\t\t\t}\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", src)]), GOROUTINE), ["a.go:4"]);
    }

    #[test]
    fn select_loop_with_done_or_ctx_is_quiet() {
        let done = "package p\n\nfunc start(in chan int, done chan struct{}) {\n\tgo func() {\n\t\tfor {\n\t\t\tselect {\n\t\t\tcase v := <-in:\n\t\t\t\tuse(v)\n\t\t\tcase <-done:\n\t\t\t\treturn\n\t\t\t}\n\t\t}\n\t}()\n}\n";
        assert!(run(&[("a.go", done)]).is_empty());
        let ctx = "package p\n\nfunc start(ctx context.Context, in chan int) {\n\tgo func() {\n\t\tfor {\n\t\t\tselect {\n\t\t\tcase <-ctx.Done():\n\t\t\t\treturn\n\t\t\tcase v := <-in:\n\t\t\t\tuse(v)\n\t\t\t}\n\t\t}\n\t}()\n}\n";
        assert!(run(&[("a.go", ctx)]).is_empty());
    }

    #[test]
    fn named_method_goroutine_is_resolved() {
        let leak = "package p\n\ntype S struct{ in chan int }\n\nfunc (s *S) run() {\n\tfor {\n\t\tselect {\n\t\tcase v := <-s.in:\n\t\t\tuse(v)\n\t\t}\n\t}\n}\n\nfunc (s *S) Start() {\n\tgo s.run()\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), GOROUTINE), ["a.go:15"]);
        let ok = leak.replace(
            "\t\t\tuse(v)\n",
            "\t\t\tuse(v)\n\t\tcase <-s.quit:\n\t\t\treturn\n",
        );
        assert!(run(&[("a.go", &ok)]).is_empty());
    }

    #[test]
    fn range_over_never_closed_channel_is_flagged() {
        let leak = "package p\n\nfunc start() {\n\tjobs := make(chan int)\n\tgo func() {\n\t\tfor j := range jobs {\n\t\t\tuse(j)\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), GOROUTINE), ["a.go:5"]);
        let closed = "package p\n\nfunc start() {\n\tjobs := make(chan int)\n\tgo func() {\n\t\tfor j := range jobs {\n\t\t\tuse(j)\n\t\t}\n\t}()\n\tclose(jobs)\n}\n";
        assert!(run(&[("a.go", closed)]).is_empty());
        let slice = "package p\n\nfunc start(items []int) {\n\tgo func() {\n\t\tfor _, i := range items {\n\t\t\tuse(i)\n\t\t}\n\t}()\n}\n";
        assert!(run(&[("a.go", slice)]).is_empty());
    }

    #[test]
    fn channel_closed_under_another_name_is_quiet() {
        let renamed = "package p\n\nfunc consume(jobs <-chan int) {\n\tfor j := range jobs {\n\t\tuse(j)\n\t}\n}\n\nfunc main() {\n\tqueue := make(chan int)\n\tgo consume(queue)\n\tqueue <- 1\n\tclose(queue)\n}\n";
        assert!(run(&[("a.go", renamed)]).is_empty());
        let deferred = "package p\n\nfunc drain(in <-chan int) {\n\tgo func() {\n\t\tfor v := range in {\n\t\t\tuse(v)\n\t\t}\n\t}()\n}\n\nfunc main() {\n\tch := make(chan int)\n\tdefer close(ch)\n\tdrain(ch)\n}\n";
        assert!(run(&[("a.go", deferred)]).is_empty());
        let relayed = "package p\n\nfunc consume(jobs <-chan int) {\n\tgo func() {\n\t\tfor j := range jobs {\n\t\t\tuse(j)\n\t\t}\n\t}()\n}\n\nfunc mid(q <-chan int) { consume(q) }\n\nfunc main() {\n\tc := make(chan int)\n\tmid(c)\n\tclose(c)\n}\n";
        assert!(run(&[("a.go", relayed)]).is_empty());
        let never = renamed.replace("\tclose(queue)\n", "");
        assert_eq!(sites(&run(&[("a.go", &never)]), GOROUTINE).len(), 1);
    }

    #[test]
    fn polling_loops_need_an_exit() {
        let leak = "package p\n\nfunc start() {\n\tgo func() {\n\t\tfor {\n\t\t\ttime.Sleep(time.Second)\n\t\t\tpoll()\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), GOROUTINE), ["a.go:4"]);
        let exits = "package p\n\nfunc start(ln net.Listener) {\n\tgo func() {\n\t\tfor {\n\t\t\tc, err := ln.Accept()\n\t\t\tif err != nil {\n\t\t\t\treturn\n\t\t\t}\n\t\t\tserve(c)\n\t\t}\n\t}()\n}\n";
        assert!(run(&[("a.go", exits)]).is_empty());
        let finite = "package p\n\nfunc start() {\n\tgo func() {\n\t\tfor i := 0; i < 3; i++ {\n\t\t\twork()\n\t\t}\n\t}()\n\tgo work()\n}\n";
        assert!(run(&[("a.go", finite)]).is_empty());
    }

    #[test]
    fn select_break_does_not_end_the_outer_loop() {
        let src = "package p\n\nfunc start(in chan int) {\n\tgo func() {\n\t\tfor {\n\t\t\tselect {\n\t\t\tcase v := <-in:\n\t\t\t\tif v == 0 {\n\t\t\t\t\tbreak\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", src)]), GOROUTINE), ["a.go:4"]);
    }

    #[test]
    fn time_tick_and_unstopped_tickers() {
        let tick =
            "package p\n\nfunc f() {\n\tfor range time.Tick(time.Second) {\n\t\twork()\n\t}\n}\n";
        assert_eq!(sites(&run(&[("a.go", tick)]), TICK), ["a.go:4"]);
        let leak = "package p\n\nfunc f() {\n\tt := time.NewTicker(time.Second)\n\t_ = t\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), TICKER), ["a.go:4"]);
        let ok =
            "package p\n\nfunc f() {\n\tt := time.NewTicker(time.Second)\n\tdefer t.Stop()\n}\n";
        assert!(sites(&run(&[("a.go", ok)]), TICKER).is_empty());
    }

    #[test]
    fn ticker_range_loop_needs_stop() {
        let leak = "package p\n\nfunc f() {\n\tt := time.NewTicker(time.Second)\n\tdefer t.Stop()\n\tgo func() {\n\t\tfor range t.C {\n\t\t\twork()\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), GOROUTINE), ["a.go:6"]);
    }

    /// 1-based line of the first line of `src` holding `needle`.
    fn at(src: &str, needle: &str) -> usize {
        src.lines()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("no line with {needle}"))
            + 1
    }

    const INDEX_GO: &str = "package edge\n\nimport \"container/list\"\n\ntype Index struct {\n\tlimit   int\n\torder   *list.List\n\tbyToken map[string]*list.Element\n}\n\ntype seen struct {\n\ttoken []byte\n\tcount int\n}\n\nfunc (x *Index) Note(token []byte) {\n\tx.byToken[string(token)] = x.order.PushFront(&seen{token: token, count: 1})\n}\n";
    const REQUEST_GO: &str = "package edge\n\nimport \"bytes\"\n\ntype Request struct {\n\tTenant  []byte\n\tRequest []byte\n}\n\nfunc ParseRequest(data []byte) Request {\n\tvar req Request\n\tname, value, ok := bytes.Cut(data, []byte(\": \"))\n\tif ok && string(name) == \"X-Request-Id\" {\n\t\treq.Request = value\n\t}\n\treturn req\n}\n";
    const ROUTER_GO: &str = "package edge\n\ntype Router struct{ seen *Index }\n\nfunc (r *Router) Handle(data []byte) {\n\treq := ParseRequest(data)\n\tr.seen.Note(req.Request)\n}\n";

    fn view_hits(sources: &[(&str, &str)]) -> Vec<StaticHit> {
        run(sources)
            .into_iter()
            .filter(|h| h.rule == VIEW)
            .collect()
    }

    #[test]
    fn view_stored_in_a_container_call_is_flagged_at_the_store() {
        let hits = view_hits(&[
            ("edge/index.go", INDEX_GO),
            ("edge/request.go", REQUEST_GO),
            ("edge/router.go", ROUTER_GO),
        ]);
        assert_eq!(hits.len(), 1, "{hits:?}");
        let h = &hits[0];
        assert_eq!(
            h.site(),
            format!("edge/index.go:{}", at(INDEX_GO, "PushFront"))
        );
        assert_eq!(h.holder, Some(Holder::ContainerField));
        assert_eq!(h.region, Some(Region::Heap));
        assert_eq!(h.owner.as_deref(), Some("edge.index.Index.order"));
        for part in [
            "`token`",
            "`Note`",
            "`x.order`",
            "Note(req.Request)",
            "edge/router.go:7",
            "`bytes.Cut` at edge/request.go:12",
        ] {
            assert!(h.message.contains(part), "{part}: {}", h.message);
        }
    }

    #[test]
    fn copies_and_other_stores_are_quiet() {
        for (from, to) in [
            ("token: token,", "token: bytes.Clone(token),"),
            ("token: token,", "token: slices.Clone(token),"),
            ("token: token,", "token: append([]byte(nil), token...),"),
            ("token: token,", "token: []byte(string(token)),"),
        ] {
            let copied = INDEX_GO.replace(from, to);
            assert_ne!(copied, INDEX_GO);
            let hits = view_hits(&[
                ("edge/index.go", &copied),
                ("edge/request.go", REQUEST_GO),
                ("edge/router.go", ROUTER_GO),
            ]);
            assert!(hits.is_empty(), "{to}: {hits:?}");
        }
        // The parameter is copied into a fresh slice before it is stored.
        let fresh = INDEX_GO.replace(
            "\tx.byToken",
            "\ttoken = append([]byte(nil), token...)\n\tx.byToken",
        );
        assert!(
            view_hits(&[
                ("edge/index.go", &fresh),
                ("edge/request.go", REQUEST_GO),
                ("edge/router.go", ROUTER_GO),
            ])
            .is_empty()
        );
        // A `string` parameter is not a view of a buffer.
        let text = "package p\n\ntype S struct{ m map[string]string }\n\nfunc (s *S) Set(id, sku string) {\n\ts.m[id] = sku\n}\n\nfunc feed(s *S, b []byte) {\n\ts.Set(\"a\", string(b[1:]))\n}\n";
        assert!(view_hits(&[("p/s.go", text)]).is_empty());
    }

    #[test]
    fn a_store_needs_a_caller_that_passes_a_view() {
        let whole = ROUTER_GO.replace("req.Request", "data");
        assert!(
            view_hits(&[
                ("edge/index.go", INDEX_GO),
                ("edge/request.go", REQUEST_GO),
                ("edge/router.go", &whole),
            ])
            .is_empty(),
            "a caller that passes a whole buffer is no evidence"
        );
        // A view made two calls away is still found through the parameter.
        let relay = "package edge\n\ntype Router struct{ seen *Index }\n\nfunc (r *Router) Handle(data []byte) {\n\tr.note(data[4:])\n}\n\nfunc (r *Router) note(t []byte) {\n\tr.seen.Note(t)\n}\n";
        let hits = view_hits(&[("edge/index.go", INDEX_GO), ("edge/router.go", relay)]);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert!(
            hits[0].message.contains("sub-slice `data[4:]`"),
            "{}",
            hits[0].message
        );
    }

    #[test]
    fn ring_slot_assignment_with_a_struct_field_view() {
        let ring = "package indexer\n\ntype Indexer struct {\n\tring [][]byte\n\tnext int\n}\n\nfunc (x *Indexer) Remember(headline []byte) {\n\tx.ring[x.next] = headline\n\tx.next = (x.next + 1) % len(x.ring)\n}\n";
        let parse = "package merge\n\nimport \"bytes\"\n\ntype Item struct {\n\tID       string\n\tHeadline []byte\n}\n\nfunc Parse(doc []byte) []Item {\n\tfields := bytes.SplitN(doc, []byte(\"|\"), 3)\n\treturn []Item{{ID: string(fields[0]), Headline: fields[2]}}\n}\n";
        let round = "package merge\n\nfunc (m *Merger) Round(doc []byte) {\n\tfor _, it := range Parse(doc) {\n\t\tm.index.Remember(it.Headline)\n\t}\n}\n";
        let hits = view_hits(&[
            ("indexer/indexer.go", ring),
            ("merge/item.go", parse),
            ("merge/merge.go", round),
        ]);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].site(), "indexer/indexer.go:9");
        assert!(
            hits[0].message.contains("field `Headline`")
                && hits[0]
                    .message
                    .contains("`bytes.SplitN` at merge/item.go:11"),
            "{}",
            hits[0].message
        );
        // `x.ring = append(x.ring, headline)` and a package-level registry are stores too.
        let appended = ring.replace(
            "x.ring[x.next] = headline",
            "x.ring = append(x.ring, headline)",
        );
        assert_eq!(
            view_hits(&[
                ("indexer/indexer.go", &appended),
                ("merge/item.go", parse),
                ("merge/merge.go", round),
            ])
            .len(),
            1
        );
        let global = "package indexer\n\nvar recent [][]byte\n\nfunc Remember(headline []byte) {\n\trecent = append(recent, headline)\n}\n";
        let hits = view_hits(&[
            ("indexer/g.go", global),
            ("merge/item.go", parse),
            (
                "merge/merge.go",
                &round.replace("m.index.Remember", "Remember"),
            ),
        ]);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].holder, Some(Holder::StaticGlobal));
        assert_eq!(hits[0].owner.as_deref(), Some("indexer.g.recent"));
    }

    #[test]
    fn a_pool_put_fed_by_its_own_buffers_is_quiet() {
        let pool = "package render\n\ntype BufferPool struct{ free [][]byte }\n\nfunc (p *BufferPool) Get() []byte {\n\tn := len(p.free)\n\tbuf := p.free[n-1]\n\tp.free = p.free[:n-1]\n\treturn buf[:0]\n}\n\nfunc (p *BufferPool) Put(buf []byte) { p.free = append(p.free, buf) }\n";
        let user = "package render\n\ntype rendered struct{ buf []byte }\n\ntype Renderer struct {\n\tpool  *BufferPool\n\tindex map[string]*rendered\n}\n\nfunc (r *Renderer) Render(id string) {\n\tif old, ok := r.index[id]; ok {\n\t\tr.pool.Put(old.buf)\n\t}\n\tbuf := append(r.pool.Get(), 'x')\n\tr.index[id] = &rendered{buf: buf}\n\tfresh := make([]byte, 0, 64)\n\tr.pool.Put(fresh)\n}\n";
        assert!(view_hits(&[("render/pool.go", pool), ("render/batch.go", user)]).is_empty());
    }

    #[test]
    fn conversions_and_static_tables_are_not_stores_of_a_view() {
        let setter = "package p\n\ntype S struct{ m Cache; z nat }\n\nfunc (s *S) SetBytes(buf []byte) {\n\ts.z.SetBytes(buf)\n}\n\nfunc (s *S) Keep(k string, buf []byte) {\n\ts.m.Set(k, buf)\n}\n\nfunc feed(s *S, b []byte) {\n\ts.SetBytes(b[2:])\n\ts.Keep(\"a\", b[2:])\n}\n";
        let hits = view_hits(&[("p/s.go", setter)]);
        let sites: Vec<_> = hits.iter().map(StaticHit::site).collect();
        // `Set(k, buf)` stores its argument; `SetBytes(buf)` converts into the receiver's own type.
        assert_eq!(
            sites,
            [format!("p/s.go:{}", at(setter, "s.m.Set"))],
            "{hits:?}"
        );
        // A window of a package-level table pins nothing that is not there for the whole process.
        let table = "package p\n\nvar optab = []int{1, 2, 3, 4}\n\nvar ranges [][]int\n\nfunc put(t []int) {\n\tranges = append(ranges, t)\n}\n\nfunc init() {\n\tput(optab[1:3])\n}\n";
        assert!(view_hits(&[("p/t.go", table)]).is_empty());
        let local = table.replace("put(optab[1:3])", "buf := make([]int, 8)\n\tput(buf[1:3])");
        assert_eq!(view_hits(&[("p/t.go", &local)]).len(), 1);
    }

    #[test]
    fn scanner_windows_fire_and_other_shapes_a_real_project_writes_stay_quiet() {
        let log = "package a\n\ntype Log struct{ lines [][]byte }\n\nfunc (l *Log) Record(line []byte) { l.lines = append(l.lines, line) }\n\nfunc pump(sc *bufio.Scanner, l *Log) {\n\tfor sc.Scan() {\n\t\tl.Record(sc.Bytes())\n\t}\n}\n";
        let hits = view_hits(&[("a/log.go", log)]);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert!(
            hits[0].message.contains("`sc.Bytes()`"),
            "{}",
            hits[0].message
        );
        // Each of these is fed a view and none retains it in accumulating state.
        let quiet = "package a\n\ntype S struct {\n\tlast []byte\n\tout  [][]byte\n}\n\n// An overwrite keeps one buffer, not one per call.\nfunc (s *S) SetLast(p []byte) { s.last = p }\n\n// The map belongs to the caller.\nfunc fill(m map[string][]byte, p []byte) { m[\"a\"] = p }\n\n// The slice is local to the call.\nfunc local(p []byte) [][]byte {\n\tvar acc [][]byte\n\tacc = append(acc, p)\n\treturn acc\n}\n\nfunc (s *S) Keep(p []byte) {\n\tdst := make([]byte, len(p))\n\tcopy(dst, p)\n\ts.out = append(s.out, dst)\n}\n\nfunc use(s *S, data []byte, m map[string][]byte) {\n\ts.SetLast(data[2:])\n\tfill(m, data[3:])\n\t_ = local(data[1:])\n\ts.Keep(data[2:])\n}\n";
        assert!(view_hits(&[("a/s.go", quiet)]).is_empty());
    }

    #[test]
    fn calls_of_another_shape_do_not_count_as_the_feeding_call() {
        // Calls are matched by name: a call with another argument count, or a bare call of a
        // method, is some other function of that name.
        let other = "package edge\n\nfunc Handle(data []byte) {\n\tseen.Note(\"x\", data[2:])\n\tNote(data[2:])\n}\n";
        assert!(view_hits(&[("edge/index.go", INDEX_GO), ("edge/other.go", other)]).is_empty());
    }
}
