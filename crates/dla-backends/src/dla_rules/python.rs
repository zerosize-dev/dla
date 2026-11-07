//! Python rules: module and class containers mutated without removal, unbounded
//! `functools.cache` / `lru_cache(maxsize=None)`, handler registration inside functions.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use dla_census::parse::tree_sitter::Node;
use dla_core::static_hit::StaticHit;
use regex::Regex;

use super::ast::{
    CallSite, UseIndex, ancestor, calls, child_of_kind, find_all, last_token, line, named_children,
    removal_is_live, tokens, txt, walk,
};
use super::{RuleInfo, SrcFile, hit};

const CHAIN: &str = "python-chain-head-growth";
const CONTAINER: &str = "python-global-container-growth";
const CACHE_METHOD: &str = "python-cache-method";
const CACHE_FUNCTION: &str = "python-unbounded-cache";
const HANDLER: &str = "python-handler-registration-in-function";

/// Rule table (mirrors the `backend = "dla"` rows of `data/detectors/python.toml`).
pub const RULES: &[RuleInfo] = &[
    RuleInfo {
        rule: CHAIN,
        archetype: "",
        prior: 0.3,
    },
    RuleInfo {
        rule: CONTAINER,
        archetype: "L-COLLECT",
        prior: 0.5,
    },
    RuleInfo {
        rule: CACHE_METHOD,
        archetype: "L-COLLECT",
        prior: 0.6,
    },
    RuleInfo {
        rule: CACHE_FUNCTION,
        archetype: "L-COLLECT",
        prior: 0.4,
    },
    RuleInfo {
        rule: HANDLER,
        archetype: "L-REGISTER",
        prior: 0.45,
    },
];

const ADDS: [&str; 9] = [
    "append",
    "add",
    "update",
    "setdefault",
    "extend",
    "insert",
    "appendleft",
    "extendleft",
    "put",
];
const REMOVES: [&str; 8] = [
    "pop",
    "popitem",
    "remove",
    "clear",
    "discard",
    "popleft",
    "evict",
    "__delitem__",
];

static CONTAINER_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:\w+\.)?(dict|list|set|defaultdict|OrderedDict|deque|Counter|ChainMap|UserDict)$",
    )
    .expect("regex")
});

static PRUNER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)evict|prune|trim|purge|clean|expire|sweep|compact|reset|clear").expect("regex")
});

fn is_unbounded_container(v: Node<'_>, src: &str) -> bool {
    match v.kind() {
        "dictionary" | "list" | "set" => named_children(v).is_empty(),
        "call" => {
            let Some(f) = v.child_by_field_name("function") else {
                return false;
            };
            let name = txt(f, src);
            if !CONTAINER_CALL.is_match(name) {
                return false;
            }
            let args = v
                .child_by_field_name("arguments")
                .map_or("", |a| txt(a, src));
            !(name.ends_with("deque") && args.contains("maxlen"))
        }
        _ => false,
    }
}

struct Decl {
    file: usize,
    name: String,
    line: u32,
}

fn decls(idx: usize, f: &SrcFile, out: &mut Vec<Decl>) {
    let src = f.text.as_str();
    let mut scopes = vec![f.tree.root_node()];
    for c in find_all(f.tree.root_node(), &["class_definition"]) {
        if let Some(b) = c.child_by_field_name("body") {
            scopes.push(b);
        }
    }
    for scope in scopes {
        for st in named_children(scope) {
            if st.kind() != "expression_statement" {
                continue;
            }
            let Some(a) = child_of_kind(st, &["assignment"]) else {
                continue;
            };
            let (Some(left), Some(right)) = (
                a.child_by_field_name("left"),
                a.child_by_field_name("right"),
            ) else {
                continue;
            };
            if left.kind() == "identifier" && is_unbounded_container(right, src) {
                out.push(Decl {
                    file: idx,
                    name: txt(left, src).to_owned(),
                    line: line(a),
                });
            }
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

/// True when `name` is rebound locally in `func` (so a bare use does not touch the global).
fn shadowed(func: Node<'_>, src: &str, name: &str) -> bool {
    let text = txt(func, src);
    if find_all(func, &["global_statement", "nonlocal_statement"])
        .iter()
        .any(|g| tokens(txt(*g, src)).contains(&name))
    {
        return false;
    }
    if let Some(p) = func.child_by_field_name("parameters") {
        if tokens(txt(p, src)).contains(&name) {
            return true;
        }
    }
    find_all(func, &["assignment"]).into_iter().any(|a| {
        a.child_by_field_name("left")
            .is_some_and(|l| l.kind() == "identifier" && txt(l, src) == name)
    }) && text.contains(name)
}

fn imported_names(f: &SrcFile) -> HashSet<String> {
    let mut out = HashSet::new();
    for i in find_all(
        f.tree.root_node(),
        &["import_from_statement", "import_statement"],
    ) {
        for t in tokens(txt(i, &f.text)) {
            out.insert(t.to_owned());
        }
    }
    out
}

fn scan(
    f: &SrcFile,
    cs: &[CallSite<'_>],
    names: &HashSet<&str>,
    decl_file: bool,
    uses: &UseIndex,
    usage: &mut HashMap<String, Usage>,
) {
    let src = f.text.as_str();
    let imported = imported_names(f);
    let counts_adds = |n: &str| decl_file || imported.contains(n);
    for c in cs {
        let toks = super::ast::recv_tokens(&c.recv);
        let Some(last) = toks.last() else {
            if PRUNER.is_match(&c.name) {
                for a in &c.args {
                    let n = last_token(txt(*a, src));
                    if names.contains(n.as_str()) {
                        usage.entry(n).or_default().removed = true;
                    }
                }
            }
            continue;
        };
        if names.contains(last.as_str()) {
            let func = ancestor(c.node, &["function_definition"]);
            if ADDS.contains(&c.name.as_str()) && counts_adds(last) {
                let local = func.is_some_and(|fu| c.recv == *last && shadowed(fu, src, last));
                if func.is_some() && !local {
                    usage.entry(last.clone()).or_default().adds += 1;
                }
            }
        }
        for t in &toks {
            if names.contains(t.as_str()) && REMOVES.contains(&c.name.as_str()) {
                usage
                    .entry(t.clone())
                    .or_default()
                    .remove(removal_is_live(f, c.node, uses));
            }
        }
        if PRUNER.is_match(&c.name) {
            for a in &c.args {
                let n = last_token(txt(*a, src));
                if names.contains(n.as_str()) {
                    usage.entry(n).or_default().removed = true;
                }
            }
        }
    }
    walk(f.tree.root_node(), &mut |n| match n.kind() {
        "assignment" | "augmented_assignment" => {
            let Some(left) = n.child_by_field_name("left") else {
                return;
            };
            let func = ancestor(n, &["function_definition"]);
            match left.kind() {
                "subscript" => {
                    let value = left
                        .child_by_field_name("value")
                        .map_or("", |v| txt(v, src));
                    let name = last_token(value);
                    if names.contains(name.as_str()) {
                        let slice = !find_all(left, &["slice"]).is_empty();
                        let u = usage.entry(name.clone()).or_default();
                        if slice {
                            u.remove(removal_is_live(f, n, uses));
                        } else if func.is_some() && counts_adds(&name) {
                            let local =
                                func.is_some_and(|fu| value == name && shadowed(fu, src, &name));
                            if !local {
                                u.adds += 1;
                            }
                        }
                    }
                }
                "attribute" | "identifier" if n.kind() == "augmented_assignment" => {
                    let name = last_token(txt(left, src));
                    let op = src
                        .get(
                            left.end_byte()
                                ..n.child_by_field_name("right")
                                    .map_or(left.end_byte(), |r| r.start_byte()),
                        )
                        .unwrap_or("")
                        .trim();
                    if names.contains(name.as_str()) && func.is_some() && counts_adds(&name) {
                        let u = usage.entry(name).or_default();
                        if op == "-=" {
                            u.remove(removal_is_live(f, n, uses));
                        } else {
                            u.adds += 1;
                        }
                    }
                }
                "identifier" => {
                    let name = txt(left, src);
                    // Rebinding a global inside a function resets it.
                    if names.contains(name) {
                        if let Some(fu) = func {
                            if !shadowed(fu, src, name) {
                                usage
                                    .entry(name.to_owned())
                                    .or_default()
                                    .remove(removal_is_live(f, n, uses));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        "delete_statement" => {
            for t in tokens(txt(n, src)) {
                if names.contains(t) {
                    usage
                        .entry(t.to_owned())
                        .or_default()
                        .remove(removal_is_live(f, n, uses));
                }
            }
        }
        "comparison_operator" => {
            let t = txt(n, src);
            if t.contains("len(") {
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

fn containers(
    files: &[SrcFile],
    calls_by_file: &[Vec<CallSite<'_>>],
    uses: &UseIndex,
) -> Vec<StaticHit> {
    let mut ds = Vec::new();
    for (i, f) in files.iter().enumerate() {
        decls(i, f, &mut ds);
    }
    let mut out = Vec::new();
    for d in &ds {
        let names: HashSet<&str> = std::iter::once(d.name.as_str()).collect();
        let mut usage: HashMap<String, Usage> = HashMap::new();
        for (i, f) in files.iter().enumerate() {
            scan(f, &calls_by_file[i], &names, i == d.file, uses, &mut usage);
        }
        let u = usage.remove(&d.name).unwrap_or_default();
        if u.adds > 0 && !u.removed {
            let note = if u.dead_removal {
                "; the only removal sits in a function that production code never calls"
            } else {
                ""
            };
            out.push(hit(
                CONTAINER,
                &files[d.file],
                d.line,
                format!(
                    "module or class container `{}` is mutated at {} site(s) inside functions and nothing removes from it, clears it or checks its size{note}",
                    d.name, u.adds
                ),
            ));
        }
    }
    out
}

/// `Some(true)` for an unbounded cache decorator, `Some(false)` for a bounded one.
fn cache_decorator(dec: Node<'_>, src: &str) -> Option<bool> {
    let expr = named_children(dec).into_iter().next()?;
    let (name, args) = if expr.kind() == "call" {
        let f = expr.child_by_field_name("function")?;
        (
            txt(f, src).to_owned(),
            expr.child_by_field_name("arguments"),
        )
    } else {
        (txt(expr, src).to_owned(), None)
    };
    let short = name.rsplit('.').next().unwrap_or("");
    match short {
        "cache" if expr.kind() != "call" => Some(true),
        "lru_cache" => {
            let Some(a) = args else { return Some(false) };
            let kids = named_children(a);
            let first = kids.first();
            let none_kw = kids.iter().any(|k| {
                k.kind() == "keyword_argument"
                    && k.child_by_field_name("name")
                        .is_some_and(|n| txt(n, src) == "maxsize")
                    && k.child_by_field_name("value")
                        .is_some_and(|v| txt(v, src) == "None")
            });
            let none_pos = first.is_some_and(|k| k.kind() == "none");
            Some(none_kw || none_pos)
        }
        _ => None,
    }
}

fn caches(f: &SrcFile) -> Vec<StaticHit> {
    let src = f.text.as_str();
    let mut out = Vec::new();
    for d in find_all(f.tree.root_node(), &["decorated_definition"]) {
        let decorators: Vec<Node<'_>> = named_children(d)
            .into_iter()
            .filter(|c| c.kind() == "decorator")
            .collect();
        let Some(func) = child_of_kind(d, &["function_definition"]) else {
            continue;
        };
        let Some(unbounded_at) = decorators
            .iter()
            .find(|dec| cache_decorator(**dec, src) == Some(true))
        else {
            continue;
        };
        let dec_text: Vec<&str> = decorators.iter().map(|x| txt(*x, src)).collect();
        if dec_text
            .iter()
            .any(|t| t.contains("staticmethod") || t.contains("classmethod"))
        {
            continue;
        }
        let params = func
            .child_by_field_name("parameters")
            .map_or("", |p| txt(p, src));
        let param_names: Vec<&str> = params
            .trim_matches(['(', ')'])
            .split(',')
            .map(|p| p.trim().split([':', '=']).next().unwrap_or("").trim())
            .filter(|p| !p.is_empty())
            .collect();
        let in_class = ancestor(d, &["class_definition", "function_definition"])
            .is_some_and(|a| a.kind() == "class_definition");
        let is_method = in_class && param_names.first().is_some_and(|p| *p == "self");
        let fname = func.child_by_field_name("name").map_or("", |n| txt(n, src));
        if is_method {
            out.push(hit(
                CACHE_METHOD,
                f,
                line(*unbounded_at),
                format!("unbounded cache on method `{fname}`: every instance used as `self` is pinned for the life of the cache"),
            ));
        } else if param_names.iter().any(|p| {
            !matches!(*p, "self" | "cls") && !p.starts_with('*')
                || p.starts_with('*') && p.len() > 1
        }) {
            out.push(hit(
                CACHE_FUNCTION,
                f,
                line(*unbounded_at),
                format!("unbounded cache on `{fname}` keeps one entry per distinct argument tuple"),
            ));
        }
    }
    out
}

static SETUP_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(main|(_?setup|_?configure|_?init|_?install)(_\w*)?|create_app|run)$")
        .expect("regex")
});

fn handlers(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let removers: HashSet<&str> = calls_by_file
        .iter()
        .flatten()
        .filter(|c| {
            c.name == "removeHandler" || (c.name == "unregister" && c.recv_last() == "atexit")
        })
        .map(|c| c.name.as_str())
        .collect();
    let mut out = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        for c in &calls_by_file[i] {
            let kind = match (c.name.as_str(), c.recv_last().as_str()) {
                ("addHandler", _) => "addHandler",
                ("register", "atexit") => "atexit.register",
                ("signal", "signal") => "signal.signal",
                _ => continue,
            };
            let Some(func) = ancestor(c.node, &["function_definition"]) else {
                continue;
            };
            let fname = func.child_by_field_name("name").map_or("", |n| txt(n, src));
            if SETUP_NAME.is_match(fname) {
                continue;
            }
            let ftext = txt(func, src);
            if ftext.contains(".handlers")
                || ftext.contains("hasHandlers")
                || ftext.contains("removeHandler")
            {
                continue;
            }
            let paired = match kind {
                "addHandler" => removers.contains("removeHandler"),
                "atexit.register" => removers.contains("unregister"),
                _ => false,
            };
            if paired {
                continue;
            }
            out.push(hit(
                HANDLER,
                f,
                line(c.node),
                format!(
                    "{kind} runs every time `{fname}` is called, with no guard or matching removal"
                ),
            ));
        }
    }
    out
}

/// Runs the Python rules over the Python files.
pub fn check(files: &[SrcFile]) -> Vec<StaticHit> {
    let calls_by_file: Vec<Vec<CallSite<'_>>> =
        files.iter().map(|f| calls(f, f.tree.root_node())).collect();
    let uses = UseIndex::build(files, &calls_by_file);
    let mut hits = containers(files, &calls_by_file, &uses);
    for f in files {
        hits.extend(caches(f));
    }
    hits.extend(handlers(files, &calls_by_file));
    hits.extend(super::chain::chain_growth(files, &uses, CHAIN));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dla_rules::{parse_sources, sites};

    fn run(src: &str) -> Vec<StaticHit> {
        check(&parse_sources(&[("app/mod.py", src)]))
    }

    #[test]
    fn module_container_growth_is_flagged() {
        let src = "CACHE = {}\nSEEN = []\n\ndef get(k):\n    CACHE[k] = load(k)\n    SEEN.append(k)\n    return CACHE[k]\n";
        assert_eq!(
            sites(&run(src), CONTAINER),
            ["app/mod.py:1", "app/mod.py:2"]
        );
    }

    #[test]
    fn class_container_growth_is_flagged() {
        let src =
            "class Reg:\n    items = []\n\n    def add(self, x):\n        self.items.append(x)\n";
        assert_eq!(sites(&run(src), CONTAINER), ["app/mod.py:2"]);
    }

    #[test]
    fn removal_size_check_or_bounds_make_it_quiet() {
        let popped = "CACHE = {}\n\ndef put(k, v):\n    CACHE[k] = v\n\ndef drop(k):\n    CACHE.pop(k, None)\n\ndrop(1)\n";
        let deleted = "CACHE = {}\n\ndef put(k, v):\n    CACHE[k] = v\n\ndef drop(k):\n    del CACHE[k]\n\ndrop(1)\n";
        let sized = "CACHE = {}\n\ndef put(k, v):\n    if len(CACHE) > 100:\n        return\n    CACHE[k] = v\n";
        let bounded = "from collections import deque\nLOG = deque(maxlen=100)\n\ndef put(x):\n    LOG.append(x)\n";
        let evicted = "CACHE = {}\n\ndef put(k, v):\n    CACHE[k] = v\n    evict_old(CACHE)\n";
        let toplevel = "TABLE = {}\nTABLE['a'] = 1\n";
        let local = "CACHE = {}\n\ndef f():\n    CACHE = {}\n    CACHE['a'] = 1\n";
        let constant = "HANDLERS = {'a': 1}\n\ndef f():\n    return HANDLERS['a']\n";
        for (name, src) in [
            ("popped", popped),
            ("deleted", deleted),
            ("sized", sized),
            ("bounded", bounded),
            ("evicted", evicted),
            ("toplevel", toplevel),
            ("local", local),
            ("constant", constant),
        ] {
            assert!(run(src).is_empty(), "{name}: {:?}", run(src));
        }
    }

    #[test]
    fn removal_in_a_function_nobody_calls_does_not_count() {
        let src = "_LISTENERS = {}\n\ndef subscribe(topic, fn):\n    _LISTENERS.setdefault(topic, []).append(fn)\n\ndef reset():\n    \"\"\"for the tests\"\"\"\n    _LISTENERS.clear()\n";
        let hits = run(src);
        assert_eq!(sites(&hits, CONTAINER), ["app/mod.py:1"]);
        assert!(
            hits[0].message.contains("never calls"),
            "{}",
            hits[0].message
        );
        let hook = format!("{src}\natexit.register(reset)\n");
        assert!(run(&hook).is_empty(), "a callback reference is a use");
        let close = src.replace("def reset", "def close");
        assert!(run(&close).is_empty(), "lifecycle names are live");
    }

    #[test]
    fn global_rebind_counts_as_reset() {
        let src = "CACHE = {}\n\ndef put(k, v):\n    CACHE[k] = v\n\ndef reset():\n    global CACHE\n    CACHE = {}\n\nreset()\n";
        assert!(run(src).is_empty());
    }

    #[test]
    fn cross_module_removal_counts() {
        let a = "CACHE = {}\n\ndef put(k, v):\n    CACHE[k] = v\n";
        let b = "from a import CACHE\n\ndef reset():\n    CACHE.clear()\n\nreset()\n";
        assert!(check(&parse_sources(&[("a.py", a), ("b.py", b)])).is_empty());
        assert_eq!(
            sites(&check(&parse_sources(&[("a.py", a)])), CONTAINER),
            ["a.py:1"]
        );
    }

    #[test]
    fn unbounded_cache_on_method() {
        let src = "import functools\n\nclass A:\n    @functools.lru_cache(maxsize=None)\n    def m(self, x):\n        return x\n\n    @cache\n    def n(self, y):\n        return y\n";
        assert_eq!(
            sites(&run(src), CACHE_METHOD),
            ["app/mod.py:4", "app/mod.py:8"]
        );
    }

    #[test]
    fn bounded_cache_static_and_cached_property_are_quiet() {
        let src = "from functools import lru_cache, cache, cached_property\n\nclass A:\n    @lru_cache(maxsize=128)\n    def m(self, x):\n        return x\n\n    @lru_cache\n    def n(self, x):\n        return x\n\n    @staticmethod\n    @cache\n    def s(x):\n        return x\n\n    @cached_property\n    def p(self):\n        return 1\n\n@cache\ndef constant():\n    return 1\n";
        assert!(run(src).is_empty(), "{:?}", run(src));
    }

    #[test]
    fn unbounded_cache_on_function_with_arguments() {
        let src = "from functools import lru_cache\n\n@lru_cache(maxsize=None)\ndef render(user_id):\n    return user_id\n\n@lru_cache(None)\ndef other(x):\n    return x\n";
        assert_eq!(
            sites(&run(src), CACHE_FUNCTION),
            ["app/mod.py:3", "app/mod.py:7"]
        );
    }

    #[test]
    fn handler_registration_inside_function() {
        let src = "import logging, atexit, signal\n\ndef get_logger(name):\n    log = logging.getLogger(name)\n    log.addHandler(logging.StreamHandler())\n    return log\n\ndef open_conn():\n    atexit.register(close_all)\n    signal.signal(signal.SIGTERM, bye)\n";
        assert_eq!(
            sites(&run(src), HANDLER),
            ["app/mod.py:5", "app/mod.py:9", "app/mod.py:10"]
        );
    }

    #[test]
    fn guarded_setup_and_module_level_registration_are_quiet() {
        let guarded = "import logging\n\ndef get_logger(name):\n    log = logging.getLogger(name)\n    if not log.handlers:\n        log.addHandler(logging.StreamHandler())\n    return log\n";
        assert!(run(guarded).is_empty());
        let setup = "import logging, signal\n\ndef setup_logging():\n    logging.getLogger().addHandler(h)\n\ndef main():\n    signal.signal(signal.SIGINT, stop)\n";
        assert!(run(setup).is_empty());
        let top = "import atexit\natexit.register(cleanup)\nlogging.getLogger().addHandler(h)\n";
        assert!(run(top).is_empty());
        let removed = "def attach(log):\n    log.addHandler(h)\n\ndef detach(log):\n    log.removeHandler(h)\n";
        assert!(run(removed).is_empty());
    }
}
