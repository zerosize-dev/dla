//! Node rules (JavaScript and TypeScript): timers without a stored handle or cleanup,
//! listeners without removal, module-level caches mutated without delete or size check.

use std::collections::{HashMap, HashSet};

use dla_census::parse::tree_sitter::Node;
use dla_core::static_hit::StaticHit;

use super::ast::{
    CallSite, UseIndex, ancestor, calls, child_of_kind, find_all, last_token, line, named_children,
    removal_is_live, tokens, txt, walk,
};
use super::{RuleInfo, SrcFile, hit};

const CHAIN: &str = "node-chain-head-growth";
const TIMER: &str = "node-timer-no-cleanup";
const LISTENER: &str = "node-listener-no-removal";
const CACHE: &str = "node-module-cache-unbounded";

/// Rule table (mirrors `data/detectors/node.dla-fragment.toml`).
pub const RULES: &[RuleInfo] = &[
    RuleInfo {
        rule: CHAIN,
        archetype: "L-CAPTURE",
        prior: 0.3,
    },
    RuleInfo {
        rule: TIMER,
        archetype: "L-WORKER",
        prior: 0.5,
    },
    RuleInfo {
        rule: LISTENER,
        archetype: "L-REGISTER",
        prior: 0.4,
    },
    RuleInfo {
        rule: CACHE,
        archetype: "L-COLLECT",
        prior: 0.5,
    },
];

const FN_KINDS: [&str; 6] = [
    "function_declaration",
    "function_expression",
    "function",
    "arrow_function",
    "method_definition",
    "generator_function_declaration",
];

fn is_global_call(c: &CallSite<'_>, name: &str) -> bool {
    c.name == name
        && matches!(
            c.recv.as_str(),
            "" | "global" | "globalThis" | "window" | "self"
        )
}

enum Handle {
    Discarded,
    /// Stored in a member, property or assigned variable.
    Named(String),
    /// Stored in a local variable declared at the call.
    Local(String),
    Escapes,
}

fn handle_of(call: Node<'_>, src: &str) -> Handle {
    let mut top = call;
    loop {
        let Some(p) = top.parent() else {
            return Handle::Escapes;
        };
        match p.kind() {
            // `setInterval(..).unref()` keeps the chain going.
            "member_expression" => {
                let Some(pp) = p.parent().filter(|x| x.kind() == "call_expression") else {
                    return Handle::Escapes;
                };
                top = pp;
            }
            "parenthesized_expression"
            | "await_expression"
            | "non_null_expression"
            | "as_expression" => top = p,
            "expression_statement" => return Handle::Discarded,
            "variable_declarator" => {
                return p
                    .child_by_field_name("name")
                    .map_or(Handle::Escapes, |n| Handle::Local(last_token(txt(n, src))));
            }
            "assignment_expression" => {
                return p
                    .child_by_field_name("left")
                    .map_or(Handle::Escapes, |n| Handle::Named(last_token(txt(n, src))));
            }
            "field_definition" | "public_field_definition" => {
                return p
                    .child_by_field_name("property")
                    .or_else(|| p.child_by_field_name("name"))
                    .map_or(Handle::Escapes, |n| Handle::Named(last_token(txt(n, src))));
            }
            _ => return Handle::Escapes,
        }
    }
}

/// Name of the function (or method, or variable holding an arrow function) around `n`.
fn enclosing_fn_name<'t>(n: Node<'t>, src: &str) -> Option<(Node<'t>, String)> {
    let f = ancestor(n, &FN_KINDS)?;
    let name = match f.kind() {
        "function_declaration" | "generator_function_declaration" | "method_definition" => f
            .child_by_field_name("name")
            .map(|x| txt(x, src).to_owned()),
        _ => {
            let p = f.parent()?;
            match p.kind() {
                "variable_declarator" => p
                    .child_by_field_name("name")
                    .map(|x| txt(x, src).to_owned()),
                "field_definition" | "public_field_definition" => p
                    .child_by_field_name("property")
                    .or_else(|| p.child_by_field_name("name"))
                    .map(|x| txt(x, src).to_owned()),
                "assignment_expression" => p
                    .child_by_field_name("left")
                    .map(|x| last_token(txt(x, src))),
                "pair" => p.child_by_field_name("key").map(|x| txt(x, src).to_owned()),
                _ => None,
            }
        }
    };
    Some((f, name.unwrap_or_default()))
}

fn rearms(c: &CallSite<'_>, src: &str, fname: &str) -> bool {
    if fname.is_empty() {
        return false;
    }
    let Some(cb) = c.args.first() else {
        return false;
    };
    match cb.kind() {
        "identifier" | "member_expression" => last_token(txt(*cb, src)) == fname,
        "arrow_function" | "function_expression" | "function" => {
            find_all(*cb, &["call_expression"])
                .into_iter()
                .any(|inner| {
                    inner
                        .child_by_field_name("function")
                        .is_some_and(|f| last_token(txt(f, src)) == fname)
                })
        }
        // `this.tick.bind(this)`
        "call_expression" => txt(*cb, src).contains(fname),
        _ => false,
    }
}

/// What the rest of a function does with a local variable that holds a timer handle.
#[derive(Default)]
struct LocalUse {
    returned: bool,
    passed: bool,
    aliases: Vec<String>,
}

fn local_use(func: Node<'_>, src: &str, name: &str) -> LocalUse {
    let mut u = LocalUse::default();
    for r in find_all(func, &["return_statement"]) {
        if tokens(txt(r, src)).contains(&name) {
            u.returned = true;
        }
    }
    for a in find_all(func, &["assignment_expression", "variable_declarator"]) {
        let right = a
            .child_by_field_name("right")
            .or_else(|| a.child_by_field_name("value"));
        let left = a
            .child_by_field_name("left")
            .or_else(|| a.child_by_field_name("name"));
        if let (Some(l), Some(r)) = (left, right) {
            if txt(r, src) == name && txt(l, src) != name {
                u.aliases.push(last_token(txt(l, src)));
            }
        }
    }
    for c in find_all(func, &["call_expression"]) {
        let Some(args) = c.child_by_field_name("arguments") else {
            continue;
        };
        let callee = c
            .child_by_field_name("function")
            .map(|f| last_token(txt(f, src)))
            .unwrap_or_default();
        if matches!(
            callee.as_str(),
            "setInterval" | "setTimeout" | "clearInterval" | "clearTimeout"
        ) {
            continue;
        }
        if named_children(args)
            .iter()
            .any(|a| a.kind() == "identifier" && txt(*a, src) == name)
        {
            u.passed = true;
        }
    }
    u
}

/// Why the timer created at `node` can never be cleared, or `None` when something clears it
/// or the handle is handed to code the rule cannot follow.
fn leak_reason(
    files: &[SrcFile],
    calls_by_file: &[Vec<CallSite<'_>>],
    cleared: &HashSet<String>,
    fi: usize,
    node: Node<'_>,
    depth: usize,
) -> Option<String> {
    let src = files[fi].text.as_str();
    match handle_of(node, src) {
        Handle::Discarded => Some("its handle is discarded, so nothing can clear it".to_owned()),
        Handle::Escapes => None,
        Handle::Named(n) => (!cleared.contains(&n))
            .then(|| format!("its handle `{n}` is never passed to clearInterval or clearTimeout")),
        Handle::Local(n) => {
            if cleared.contains(&n) {
                return None;
            }
            let (func, fname) = enclosing_fn_name(node, src)?;
            let u = local_use(func, src, &n);
            if u.passed || u.aliases.iter().any(|a| cleared.contains(a)) {
                return None;
            }
            if u.returned {
                if depth >= 2 || fname.is_empty() {
                    return None;
                }
                let mut reason = None;
                for (j, cs) in calls_by_file.iter().enumerate() {
                    for c in cs {
                        if c.name == fname && c.node.id() != node.id() {
                            if let Some(r) =
                                leak_reason(files, calls_by_file, cleared, j, c.node, depth + 1)
                            {
                                reason = Some(format!(
                                    "it is returned from `{fname}`, whose caller {r}"
                                ));
                            }
                        }
                    }
                }
                return reason;
            }
            Some(format!(
                "its handle `{n}` is kept in a local variable and never passed to clearInterval or clearTimeout"
            ))
        }
    }
}

fn timers(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let mut cleared: HashSet<String> = HashSet::new();
    for (i, f) in files.iter().enumerate() {
        for c in &calls_by_file[i] {
            if matches!(
                c.name.as_str(),
                "clearInterval" | "clearTimeout" | "clearImmediate"
            ) {
                if let Some(a) = c.args.first() {
                    for t in tokens(txt(*a, &f.text)) {
                        cleared.insert(t.to_owned());
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        for c in &calls_by_file[i] {
            let interval = is_global_call(c, "setInterval");
            let timeout = is_global_call(c, "setTimeout");
            if !(interval || timeout) {
                continue;
            }
            let Some((_, fname)) = enclosing_fn_name(c.node, src) else {
                continue;
            };
            if timeout && !rearms(c, src, &fname) {
                continue;
            }
            let what = if interval {
                "setInterval"
            } else {
                "re-arming setTimeout"
            };
            if let Some(why) = leak_reason(files, calls_by_file, &cleared, i, c.node, 0) {
                out.push(hit(
                    TIMER,
                    f,
                    line(c.node),
                    format!("{what} inside a function: {why}"),
                ));
            }
        }
    }
    out
}

const GLOBAL_EMITTERS: [&str; 8] = [
    "process",
    "window",
    "document",
    "globalThis",
    "global",
    "self",
    "navigator",
    "screen",
];

const PROCESS_EVENTS: [&str; 9] = [
    "SIGINT",
    "SIGTERM",
    "SIGHUP",
    "SIGUSR1",
    "SIGUSR2",
    "exit",
    "beforeExit",
    "uncaughtException",
    "unhandledRejection",
];

fn top_level_names(file: &SrcFile) -> HashSet<String> {
    let src = file.text.as_str();
    let mut out = HashSet::new();
    for n in named_children(file.tree.root_node()) {
        let decl = if n.kind() == "export_statement" {
            child_of_kind(n, &["lexical_declaration", "variable_declaration"])
        } else if matches!(n.kind(), "lexical_declaration" | "variable_declaration") {
            Some(n)
        } else {
            None
        };
        if let Some(d) = decl {
            for v in named_children(d) {
                if let Some(name) = v.child_by_field_name("name") {
                    out.insert(txt(name, src).to_owned());
                }
            }
        }
    }
    out
}

fn listeners(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    // Removals: (receiver last, event text) pairs seen anywhere.
    let mut removal_recv: HashSet<String> = HashSet::new();
    let mut removal_event: HashSet<String> = HashSet::new();
    for (i, f) in files.iter().enumerate() {
        for c in &calls_by_file[i] {
            if matches!(
                c.name.as_str(),
                "off"
                    | "removeListener"
                    | "removeEventListener"
                    | "removeAllListeners"
                    | "unsubscribe"
            ) {
                removal_recv.insert(c.recv_last());
                if let Some(a) = c.args.first() {
                    removal_event.insert(txt(*a, &f.text).trim().to_owned());
                }
            }
        }
    }
    let mut out = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        let globals = top_level_names(f);
        for c in &calls_by_file[i] {
            if !matches!(
                c.name.as_str(),
                "on" | "addListener" | "addEventListener" | "prependListener"
            ) {
                continue;
            }
            if c.args.len() < 2 || c.recv.is_empty() {
                continue;
            }
            if ancestor(c.node, &FN_KINDS).is_none() {
                continue;
            }
            let recv_last = c.recv_last();
            let long_lived = GLOBAL_EMITTERS.contains(&recv_last.as_str())
                || c.recv.starts_with("this.")
                || (globals.contains(&recv_last) && !c.recv.contains('.'));
            if !long_lived {
                continue;
            }
            let event = txt(c.args[0], src).trim().to_owned();
            let event_name = event.trim_matches(['\'', '"', '`']);
            if GLOBAL_EMITTERS.contains(&recv_last.as_str()) && PROCESS_EVENTS.contains(&event_name)
            {
                continue;
            }
            let options = c.args.get(2).map_or("", |o| txt(*o, src));
            if options.contains("signal") || options.contains("once") {
                continue;
            }
            if removal_recv.contains(&recv_last) || removal_event.contains(&event) {
                continue;
            }
            out.push(hit(
                LISTENER,
                f,
                line(c.node),
                format!("`{}({event}, ...)` on long-lived `{recv_last}` inside a function, and no off/removeListener exists for it", c.name),
            ));
        }
    }
    out
}

struct CacheDecl {
    file: usize,
    name: String,
    line: u32,
    exported: bool,
}

fn is_empty_container(v: Node<'_>, src: &str) -> bool {
    match v.kind() {
        "object" | "array" => named_children(v).is_empty(),
        "new_expression" => {
            let ctor = v
                .child_by_field_name("constructor")
                .map_or("", |c| txt(c, src));
            matches!(ctor, "Map" | "Set" | "Array" | "Object")
        }
        "call_expression" => txt(v, src).starts_with("Object.create(null"),
        "as_expression"
        | "satisfies_expression"
        | "non_null_expression"
        | "parenthesized_expression" => named_children(v)
            .first()
            .is_some_and(|inner| is_empty_container(*inner, src)),
        _ => false,
    }
}

fn cache_decls(idx: usize, f: &SrcFile, out: &mut Vec<CacheDecl>) {
    let src = f.text.as_str();
    let exports_text = src.contains("module.exports") || src.contains("exports.");
    for n in named_children(f.tree.root_node()) {
        let exported_kw = n.kind() == "export_statement";
        let decl = if exported_kw {
            child_of_kind(n, &["lexical_declaration", "variable_declaration"])
        } else if matches!(n.kind(), "lexical_declaration" | "variable_declaration") {
            Some(n)
        } else {
            None
        };
        if let Some(d) = decl {
            for v in named_children(d) {
                let (Some(name), Some(value)) = (
                    v.child_by_field_name("name"),
                    v.child_by_field_name("value"),
                ) else {
                    continue;
                };
                if is_empty_container(value, src) {
                    out.push(CacheDecl {
                        file: idx,
                        name: txt(name, src).to_owned(),
                        line: line(v),
                        exported: exported_kw || exports_text,
                    });
                }
            }
        }
    }
    for fd in find_all(
        f.tree.root_node(),
        &["field_definition", "public_field_definition"],
    ) {
        let is_static = txt(fd, src).trim_start().starts_with("static ");
        let (Some(name), Some(value)) = (
            fd.child_by_field_name("property")
                .or_else(|| fd.child_by_field_name("name")),
            fd.child_by_field_name("value"),
        ) else {
            continue;
        };
        if is_static && is_empty_container(value, src) {
            out.push(CacheDecl {
                file: idx,
                name: txt(name, src).to_owned(),
                line: line(fd),
                exported: true,
            });
        }
    }
}

#[derive(Default)]
struct Usage {
    adds: usize,
    removed: bool,
    dead_removal: bool,
}

impl Usage {
    fn remove(&mut self, live: bool) {
        if live {
            self.removed = true;
        } else {
            self.dead_removal = true;
        }
    }
}

fn scan_usage(
    f: &SrcFile,
    file_calls: &[CallSite<'_>],
    names: &HashSet<&str>,
    uses: &UseIndex,
    usage: &mut HashMap<String, Usage>,
) {
    let src = f.text.as_str();
    for c in file_calls {
        let last = c.recv_last();
        if names.contains(last.as_str()) {
            let u = usage.entry(last).or_default();
            match c.name.as_str() {
                "set" | "add" | "push" | "unshift" | "append" | "concat"
                    if ancestor(c.node, &FN_KINDS).is_some() =>
                {
                    u.adds += 1;
                }
                "delete" | "clear" | "shift" | "pop" | "splice" | "reset" | "evict" => {
                    u.remove(removal_is_live(f, c.node, uses));
                }
                _ => {}
            }
        }
        if c.name == "assign" && c.recv == "Object" {
            if let Some(a) = c.args.first() {
                let n = last_token(txt(*a, src));
                if names.contains(n.as_str()) && ancestor(c.node, &FN_KINDS).is_some() {
                    usage.entry(n).or_default().adds += 1;
                }
            }
        }
        // Passing the container to a pruning helper.
        if matches!(c.name.to_ascii_lowercase().as_str(), s if s.contains("evict") || s.contains("prune") || s.contains("trim") || s.contains("purge") || s.contains("cleanup") || s.contains("expire") || s.contains("sweep"))
        {
            for a in &c.args {
                let n = last_token(txt(*a, src));
                if names.contains(n.as_str()) {
                    usage.entry(n).or_default().removed = true;
                }
            }
        }
    }
    walk(f.tree.root_node(), &mut |n| match n.kind() {
        "assignment_expression" | "augmented_assignment_expression" => {
            let Some(left) = n.child_by_field_name("left") else {
                return;
            };
            let lt = txt(left, src);
            match left.kind() {
                "subscript_expression" | "member_expression" => {
                    let obj = left
                        .child_by_field_name("object")
                        .map_or("", |o| txt(o, src));
                    let name = last_token(obj);
                    if names.contains(name.as_str()) {
                        let u = usage.entry(name).or_default();
                        if lt.ends_with(".length") {
                            u.remove(removal_is_live(f, n, uses));
                        } else if ancestor(n, &FN_KINDS).is_some() {
                            u.adds += 1;
                        }
                    }
                }
                // Rebinding inside a function empties or replaces the container.
                "identifier" if names.contains(lt) && ancestor(n, &FN_KINDS).is_some() => {
                    usage
                        .entry(lt.to_owned())
                        .or_default()
                        .remove(removal_is_live(f, n, uses));
                }
                _ => {}
            }
        }
        "unary_expression" => {
            let t = txt(n, src);
            if t.starts_with("delete ") {
                for tok in tokens(t).into_iter().skip(1) {
                    if names.contains(tok) {
                        usage
                            .entry(tok.to_owned())
                            .or_default()
                            .remove(removal_is_live(f, n, uses));
                    }
                }
            }
        }
        "binary_expression" => {
            let t = txt(n, src);
            if (t.contains('<') || t.contains('>'))
                && (t.contains(".size") || t.contains(".length"))
            {
                for tok in tokens(t) {
                    if names.contains(tok) {
                        usage.entry(tok.to_owned()).or_default().removed = true;
                    }
                }
            }
        }
        _ => {}
    });
}

fn caches(
    files: &[SrcFile],
    calls_by_file: &[Vec<CallSite<'_>>],
    uses: &UseIndex,
) -> Vec<StaticHit> {
    let mut decls = Vec::new();
    for (i, f) in files.iter().enumerate() {
        cache_decls(i, f, &mut decls);
    }
    let mut out = Vec::new();
    for d in &decls {
        let names: HashSet<&str> = std::iter::once(d.name.as_str()).collect();
        let mut usage: HashMap<String, Usage> = HashMap::new();
        for (i, f) in files.iter().enumerate() {
            if i == d.file || d.exported {
                scan_usage(f, &calls_by_file[i], &names, uses, &mut usage);
            }
        }
        let u = usage.remove(&d.name).unwrap_or_default();
        if u.adds > 0 && !u.removed {
            let note = if u.dead_removal {
                "; the only removal sits in a function that production code never calls"
            } else {
                ""
            };
            out.push(hit(
                CACHE,
                &files[d.file],
                d.line,
                format!(
                    "module-level container `{}` is mutated at {} site(s) inside functions and is never deleted from, cleared or size-checked{note}",
                    d.name, u.adds
                ),
            ));
        }
    }
    out
}

/// Runs the Node rules over the JavaScript and TypeScript files.
pub fn check(files: &[SrcFile]) -> Vec<StaticHit> {
    let calls_by_file: Vec<Vec<CallSite<'_>>> =
        files.iter().map(|f| calls(f, f.tree.root_node())).collect();
    let mut hits = timers(files, &calls_by_file);
    hits.extend(listeners(files, &calls_by_file));
    let uses = UseIndex::build(files, &calls_by_file);
    hits.extend(caches(files, &calls_by_file, &uses));
    hits.extend(super::chain::chain_growth(files, &uses, CHAIN));
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
    fn interval_without_handle_or_clear_is_flagged() {
        let src = "function onConnect(sock) {\n  setInterval(() => sock.ping(), 1000);\n}\nclass B {\n  start() {\n    this.t = setInterval(() => {}, 10);\n  }\n}\n";
        assert_eq!(sites(&run(&[("a.js", src)]), TIMER), ["a.js:2", "a.js:6"]);
    }

    #[test]
    fn cleared_or_escaping_timers_are_quiet() {
        let cleared = "class B {\n  start() { this.t = setInterval(() => {}, 10); }\n  stop() { clearInterval(this.t); }\n}\n";
        assert!(run(&[("a.js", cleared)]).is_empty());
        let local = "function f() {\n  const t = setInterval(() => {}, 10);\n  setTimeout(() => clearInterval(t), 100);\n}\n";
        assert!(run(&[("a.js", local)]).is_empty());
        let stored = "function f(timers) {\n  timers.push(setInterval(() => {}, 10));\n  return setInterval(() => {}, 5);\n}\n";
        assert!(run(&[("a.js", stored)]).is_empty());
        let top = "setInterval(() => {}, 1000);\n";
        assert!(run(&[("a.js", top)]).is_empty(), "process-lifetime timer");
        let plain_timeout = "function f() { setTimeout(() => done(), 10); }\n";
        assert!(run(&[("a.js", plain_timeout)]).is_empty());
    }

    #[test]
    fn returned_handle_is_followed_to_its_callers() {
        let hb = "function startBeat(s, ms) {\n  const timer = setInterval(() => s.ping(), ms);\n  timer.unref();\n  return timer;\n}\nmodule.exports = { startBeat };\n";
        let leaky = "const { startBeat } = require('./hb');\nclass Session {\n  open() { this.beat = startBeat(this, 5); }\n  close() { this.closed = true; }\n}\n";
        let fixed = "const { startBeat } = require('./hb');\nclass Session {\n  open() { this.beat = startBeat(this, 5); }\n  close() { clearInterval(this.beat); }\n}\n";
        let discarded =
            "const { startBeat } = require('./hb');\nfunction open(s) { startBeat(s, 5); }\n";
        let hits = run(&[("hb.js", hb), ("session.js", leaky)]);
        assert_eq!(sites(&hits, TIMER), ["hb.js:2"]);
        assert!(run(&[("hb.js", hb), ("session.js", fixed)]).is_empty());
        assert_eq!(
            sites(&run(&[("hb.js", hb), ("s.js", discarded)]), TIMER),
            ["hb.js:2"]
        );
        assert!(
            run(&[("hb.js", hb)]).is_empty(),
            "no callers: nothing to say"
        );
    }

    #[test]
    fn rearming_timeout_is_flagged_unless_cleared() {
        let leak = "function poll() {\n  fetchIt();\n  setTimeout(poll, 1000);\n}\n";
        assert_eq!(sites(&run(&[("a.js", leak)]), TIMER), ["a.js:3"]);
        let leak2 = "const tick = () => {\n  work();\n  setTimeout(() => tick(), 5);\n};\n";
        assert_eq!(sites(&run(&[("a.js", leak2)]), TIMER), ["a.js:3"]);
        let ok = "let t;\nfunction poll() {\n  t = setTimeout(poll, 1000);\n}\nfunction stop() { clearTimeout(t); }\n";
        assert!(run(&[("a.js", ok)]).is_empty());
    }

    #[test]
    fn unref_does_not_count_as_cleanup() {
        let src = "function f() {\n  setInterval(() => {}, 5).unref();\n}\n";
        assert_eq!(sites(&run(&[("a.js", src)]), TIMER), ["a.js:2"]);
    }

    #[test]
    fn listener_on_long_lived_emitter_without_off() {
        let src = "const bus = new EventEmitter();\nfunction watch(id) {\n  bus.on('tick', () => id);\n}\n";
        assert_eq!(sites(&run(&[("a.js", src)]), LISTENER), ["a.js:3"]);
        let this_member = "class A {\n  attach() { this.bus.addListener('x', this.h); }\n}\n";
        assert_eq!(sites(&run(&[("a.js", this_member)]), LISTENER), ["a.js:2"]);
        let ts = "function f(): void {\n  window.addEventListener('resize', onResize);\n}\n";
        assert_eq!(sites(&run(&[("a.ts", ts)]), LISTENER), ["a.ts:2"]);
    }

    #[test]
    fn removed_once_local_and_toplevel_listeners_are_quiet() {
        let removed = "const bus = new EventEmitter();\nfunction watch() { bus.on('tick', h); }\nfunction unwatch() { bus.off('tick', h); }\n";
        assert!(run(&[("a.js", removed)]).is_empty());
        let local = "function f(sock) {\n  sock.on('data', d => d);\n  const s = net.connect();\n  s.on('x', y);\n}\n";
        assert!(run(&[("a.js", local)]).is_empty());
        let top = "const bus = new EventEmitter();\nbus.on('x', h);\n";
        assert!(run(&[("a.js", top)]).is_empty());
        let sig = "function main() { process.on('SIGINT', stop); }\n";
        assert!(run(&[("a.js", sig)]).is_empty());
        let abort = "function f(ac) { window.addEventListener('x', h, { signal: ac.signal }); }\n";
        assert!(run(&[("a.js", abort)]).is_empty());
    }

    #[test]
    fn module_cache_growth_is_flagged() {
        let src = "const cache = new Map();\nfunction get(k) {\n  if (!cache.has(k)) cache.set(k, load(k));\n  return cache.get(k);\n}\n";
        assert_eq!(sites(&run(&[("a.js", src)]), CACHE), ["a.js:1"]);
        let obj = "let memo = {};\nexport function f(k, v) { memo[k] = v; }\n";
        assert_eq!(sites(&run(&[("a.js", obj)]), CACHE), ["a.js:1"]);
        let arr = "const seen: string[] = [];\nfunction f(x: string) { seen.push(x); }\n";
        assert_eq!(sites(&run(&[("a.ts", arr)]), CACHE), ["a.ts:1"]);
    }

    #[test]
    fn bounded_module_caches_are_quiet() {
        let deleted = "const cache = new Map();\nfunction put(k, v) { cache.set(k, v); }\nfunction drop(k) { cache.delete(k); }\ndrop('a');\n";
        assert!(run(&[("a.js", deleted)]).is_empty());
        let sized = "const cache = new Map();\nfunction put(k, v) {\n  if (cache.size >= 100) { const first = cache.keys().next().value; cache.delete(first); }\n  cache.set(k, v);\n}\n";
        assert!(run(&[("a.js", sized)]).is_empty());
        let capped = "const log = [];\nfunction add(x) { log.push(x); if (log.length > 50) log.length = 0; }\n";
        assert!(run(&[("a.js", capped)]).is_empty());
        let weak = "const cache = new WeakMap();\nfunction put(k, v) { cache.set(k, v); }\n";
        assert!(run(&[("a.js", weak)]).is_empty());
        let top_only = "const cache = new Map();\ncache.set('a', 1);\n";
        assert!(run(&[("a.js", top_only)]).is_empty());
        let local = "function f() { const m = new Map(); m.set(1, 2); }\n";
        assert!(run(&[("a.js", local)]).is_empty());
    }

    #[test]
    fn removal_in_an_uncalled_function_does_not_count() {
        let src = "const cache = new Map();\nfunction put(k, v) { cache.set(k, v); }\nfunction reset() { cache.clear(); }\nmodule.exports = { put, reset };\n";
        let hits = run(&[("a.js", src)]);
        assert_eq!(sites(&hits, CACHE), ["a.js:1"]);
        let used = format!("{src}reset();\n");
        assert!(run(&[("a.js", &used)]).is_empty());
    }

    #[test]
    fn exported_cache_removal_in_another_file_counts() {
        let a = "export const cache = new Map();\nexport function put(k, v) { cache.set(k, v); }\n";
        let b =
            "import { cache } from './a';\nexport function reset() { cache.clear(); }\nreset();\n";
        assert!(run(&[("a.js", a), ("b.js", b)]).is_empty());
        assert_eq!(sites(&run(&[("a.js", a)]), CACHE), ["a.js:1"]);
    }
}
