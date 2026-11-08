//! JVM rules (Java and Kotlin): unbounded static/singleton collections, listener
//! registration without removal, Micrometer tags fed by request data.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::LazyLock;

use dla_census::parse::{Lang, tree_sitter::Node};
use dla_core::static_hit::StaticHit;
use regex::Regex;

use super::ast::{
    CallSite, UseIndex, ancestor, calls, child_of_kind, find_all, is_literal, last_token, line,
    named_children, recv_tokens, removal_is_live, tokens, txt, walk,
};
use super::{RuleInfo, SrcFile, hit};

const CHAIN: &str = "jvm-chain-head-growth";
const UNBOUNDED: &str = "jvm-unbounded-collection";
const LISTENER: &str = "jvm-listener-no-removal";
const MICROMETER: &str = "jvm-micrometer-request-tag";

/// Rule table (mirrors the `backend = "dla"` rows of `data/detectors/jvm.toml`).
pub const RULES: &[RuleInfo] = &[
    RuleInfo {
        rule: CHAIN,
        archetype: "",
        prior: 0.3,
    },
    RuleInfo {
        rule: UNBOUNDED,
        archetype: "L-COLLECT",
        prior: 0.5,
    },
    RuleInfo {
        rule: LISTENER,
        archetype: "L-REGISTER",
        prior: 0.4,
    },
    RuleInfo {
        rule: MICROMETER,
        archetype: "L-COLLECT",
        prior: 0.55,
    },
];

static CONTAINER_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(Map|HashMap|ConcurrentHashMap|LinkedHashMap|TreeMap|ConcurrentMap|SortedMap|NavigableMap|Hashtable|List|ArrayList|LinkedList|Set|HashSet|LinkedHashSet|TreeSet|Collection|Queue|Deque|ArrayDeque|ConcurrentLinkedQueue|ConcurrentLinkedDeque|CopyOnWriteArrayList|CopyOnWriteArraySet|Vector|Stack|PriorityQueue|BlockingQueue|BlockingDeque|LinkedBlockingQueue|LinkedBlockingDeque|ConcurrentSkipListMap|ConcurrentSkipListSet|Multimap|ListMultimap|SetMultimap|ArrayListMultimap|HashMultimap|MutableList|MutableMap|MutableSet|MutableCollection|mutableListOf|mutableMapOf|mutableSetOf|arrayListOf|hashMapOf|hashSetOf|linkedMapOf|linkedSetOf|sortedMapOf|mutableListOf)\b",
    )
    .expect("regex")
});

static BOUNDED_HINT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"WeakHashMap|WeakReference|Caffeine|CacheBuilder|\bCache\b|LoadingCache|removeEldestEntry|ArrayBlockingQueue|unmodifiable|\b(List|Set|Map)\.of\b|Arrays\.asList|Immutable|EnumMap|EnumSet|emptyList|emptyMap|emptySet|Collections\.empty|ConcurrentReferenceHashMap|MapMaker|\blistOf\b|\bsetOf\b|\bmapOf\b|\blazy\b|BoundedQueue|EvictingQueue|CircularFifoQueue|maxsize|maxSize|capacity",
    )
    .expect("regex")
});

static CAPPED_QUEUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"LinkedBlocking(Queue|Deque)\s*(<[^>]*>)?\s*\(\s*[^)\s]").expect("regex")
});

const ADDS: [&str; 22] = [
    "add",
    "addAll",
    "put",
    "putAll",
    "putIfAbsent",
    "computeIfAbsent",
    "compute",
    "merge",
    "offer",
    "offerFirst",
    "offerLast",
    "push",
    "addFirst",
    "addLast",
    "getOrPut",
    "append",
    "set",
    "insert",
    "enqueue",
    "plus",
    "addElement",
    "put_all",
];

const REMOVES: [&str; 25] = [
    "remove",
    "removeAll",
    "removeIf",
    "retainAll",
    "clear",
    "poll",
    "pollFirst",
    "pollLast",
    "pop",
    "take",
    "drainTo",
    "removeFirst",
    "removeLast",
    "removeAt",
    "removeElement",
    "evict",
    "evictAll",
    "invalidate",
    "invalidateAll",
    "trimToSize",
    "removeAllElements",
    "removeFirstOrNull",
    "removeLastOrNull",
    "cleanUp",
    "expire",
];

static PRUNER_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)evict|prune|trim|purge|clean|expire|remove|clear|sweep|compact|drain|bound|limit|cap",
    )
    .expect("regex")
});

const SINGLETON_ANN: [&str; 11] = [
    "Component",
    "Service",
    "Repository",
    "Singleton",
    "Configuration",
    "Controller",
    "RestController",
    "ApplicationScoped",
    "ManagedBean",
    "Named",
    "Bean",
];

const FN_KINDS: [&str; 7] = [
    "method_declaration",
    "constructor_declaration",
    "lambda_expression",
    "function_declaration",
    "lambda_literal",
    "secondary_constructor",
    "anonymous_function",
];

const INIT_KINDS: [&str; 3] = [
    "static_initializer",
    "anonymous_initializer",
    "instance_initializer",
];

struct Decl {
    file: usize,
    name: String,
    line: u32,
}

fn has_singleton_annotation(modifiers: &str) -> bool {
    modifiers.split('@').skip(1).any(|a| {
        let name: String = a
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
            .collect();
        SINGLETON_ANN.contains(&name.rsplit('.').next().unwrap_or(""))
    })
}

fn is_unbounded_container(type_and_init: &str) -> bool {
    CONTAINER_TYPE.is_match(type_and_init)
        && !BOUNDED_HINT.is_match(type_and_init)
        && !CAPPED_QUEUE.is_match(type_and_init)
}

/// Method names that register a long-lived subscriber on the owner.
static REGISTRATION_METHOD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(subscribe|register|(add|attach)\w*(Listener|Observer|Subscriber|Handler|Callback|Hook))")
        .expect("regex")
});

fn class_name_of<'a>(class: Node<'_>, src: &'a str) -> &'a str {
    class
        .child_by_field_name("name")
        .map_or("", |n| txt(n, src))
}

/// True when the class itself is annotated as a singleton, exposes `getInstance`, or has a
/// registration method (`subscribe`, `register...`) that makes its instances registries.
fn class_is_long_lived(class: Node<'_>, src: &str) -> bool {
    let cm = child_of_kind(class, &["modifiers"]).map_or("", |m| txt(m, src));
    let body = txt(class, src);
    has_singleton_annotation(cm)
        || body.contains("getInstance(")
        || body.contains("static final ") && body.contains(" INSTANCE")
        || find_all(class, &["method_declaration"])
            .into_iter()
            .any(|m| {
                m.child_by_field_name("name")
                    .is_some_and(|n| REGISTRATION_METHOD.is_match(txt(n, src)))
            })
}

/// Names of Java classes whose instances live as long as a singleton does: the singleton
/// itself and every class on its `extends` chain, since the superclass fields live in it.
fn long_lived_classes(files: &[SrcFile]) -> HashSet<String> {
    let mut parent: HashMap<String, String> = HashMap::new();
    let mut roots: Vec<String> = Vec::new();
    for f in files.iter().filter(|f| f.lang == Lang::Java) {
        let src = f.text.as_str();
        for class in find_all(f.tree.root_node(), &["class_declaration"]) {
            let name = class_name_of(class, src);
            if name.is_empty() {
                continue;
            }
            if let Some(sup) = class.child_by_field_name("superclass")
                && let Some(base) = tokens(txt(sup, src)).into_iter().find(|t| *t != "extends")
            {
                parent.insert(name.to_owned(), base.to_owned());
            }
            if class_is_long_lived(class, src) {
                roots.push(name.to_owned());
            }
        }
    }
    let mut out = HashSet::new();
    for r in roots {
        let mut cur = Some(r);
        while let Some(c) = cur {
            if !out.insert(c.clone()) {
                break;
            }
            cur = parent.get(&c).cloned();
        }
    }
    out
}

fn java_decls(idx: usize, file: &SrcFile, long_lived: &HashSet<String>, out: &mut Vec<Decl>) {
    let src = file.text.as_str();
    for f in find_all(file.tree.root_node(), &["field_declaration"]) {
        let modifiers = child_of_kind(f, &["modifiers"]).map_or("", |m| txt(m, src));
        let class = ancestor(
            f,
            &[
                "class_declaration",
                "enum_declaration",
                "interface_declaration",
            ],
        );
        let Some(class) = class else { continue };
        let is_interface = class.kind() == "interface_declaration";
        let is_static = is_interface || modifiers.split_whitespace().any(|w| w == "static");
        let singleton =
            long_lived.contains(class_name_of(class, src)) || class_is_long_lived(class, src);
        if !(is_static || singleton) {
            continue;
        }
        let ty = f.child_by_field_name("type").map_or("", |t| txt(t, src));
        for d in named_children(f)
            .into_iter()
            .filter(|c| c.kind() == "variable_declarator")
        {
            let Some(name) = d.child_by_field_name("name") else {
                continue;
            };
            let init = d.child_by_field_name("value").map_or("", |v| txt(v, src));
            let both = format!("{ty} {init}");
            if is_unbounded_container(&both) {
                out.push(Decl {
                    file: idx,
                    name: txt(name, src).to_owned(),
                    line: line(d),
                });
            }
        }
    }
}

fn kotlin_decls(idx: usize, file: &SrcFile, out: &mut Vec<Decl>) {
    let src = file.text.as_str();
    for p in find_all(file.tree.root_node(), &["property_declaration"]) {
        let Some(owner) = ancestor(
            p,
            &[
                "function_declaration",
                "anonymous_initializer",
                "lambda_literal",
                "secondary_constructor",
                "class_declaration",
                "object_declaration",
                "companion_object",
                "source_file",
            ],
        ) else {
            continue;
        };
        let keep = match owner.kind() {
            "source_file" | "object_declaration" | "companion_object" => true,
            "class_declaration" => has_singleton_annotation(
                child_of_kind(owner, &["modifiers"]).map_or("", |m| txt(m, src)),
            ),
            _ => false,
        };
        if !keep {
            continue;
        }
        let Some(var) = child_of_kind(p, &["variable_declaration"]) else {
            continue;
        };
        let Some(name) = child_of_kind(var, &["identifier"]) else {
            continue;
        };
        let body = txt(p, src);
        // Drop the modifiers so an annotation never counts as a type.
        let rest = body
            .split_once("val ")
            .or_else(|| body.split_once("var "))
            .map_or(body, |x| x.1);
        if is_unbounded_container(rest) {
            out.push(Decl {
                file: idx,
                name: txt(name, src).to_owned(),
                line: line(p),
            });
        }
    }
}

/// True when the call node sits in code that runs repeatedly: inside a method or lambda,
/// not in a static or instance initializer.
fn in_runtime_code(n: Node<'_>) -> bool {
    let mut cur = n.parent();
    while let Some(p) = cur {
        if INIT_KINDS.contains(&p.kind()) {
            return false;
        }
        if FN_KINDS.contains(&p.kind()) {
            return true;
        }
        cur = p.parent();
    }
    false
}

/// Assignment-shaped mutations in Kotlin: `name[k] = v`, `name += x`, `name -= x`.
/// Returns `(container name, is_removal)`.
fn kotlin_assignment(file: &SrcFile, n: Node<'_>) -> Option<(String, bool)> {
    if file.lang != Lang::Kotlin || n.kind() != "assignment" {
        return None;
    }
    let src = file.text.as_str();
    let kids = named_children(n);
    let (left, right) = (kids.first()?, kids.last()?);
    let op = src.get(left.end_byte()..right.start_byte())?.trim();
    match left.kind() {
        "index_expression" => {
            let recv = named_children(*left).into_iter().next()?;
            Some((last_token(txt(recv, src)), false))
        }
        _ if op == "+=" => Some((last_token(txt(*left, src)), false)),
        _ if op == "-=" => Some((last_token(txt(*left, src)), true)),
        _ => None,
    }
}

fn unbounded_collections(
    files: &[SrcFile],
    calls_by_file: &[Vec<CallSite<'_>>],
    uses: &UseIndex,
) -> Vec<StaticHit> {
    let mut decls = Vec::new();
    let long_lived = long_lived_classes(files);
    for (i, f) in files.iter().enumerate() {
        match f.lang {
            Lang::Java => java_decls(i, f, &long_lived, &mut decls),
            Lang::Kotlin => kotlin_decls(i, f, &mut decls),
            _ => {}
        }
    }
    if decls.is_empty() {
        return Vec::new();
    }
    let names: HashSet<&str> = decls.iter().map(|d| d.name.as_str()).collect();
    let mut adds: BTreeMap<String, usize> = BTreeMap::new();
    let mut removed: HashSet<String> = HashSet::new();
    let mut dead_removal: HashSet<String> = HashSet::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        for c in &calls_by_file[i] {
            let toks = recv_tokens(&c.recv);
            if ADDS.contains(&c.name.as_str())
                && in_runtime_code(c.node)
                && let Some(last) = toks.last().filter(|l| names.contains(l.as_str()))
            {
                *adds.entry(last.clone()).or_default() += 1;
            }
            if REMOVES.contains(&c.name.as_str()) {
                let live = removal_is_live(f, c.node, uses);
                for t in &toks {
                    if names.contains(t.as_str()) {
                        if live {
                            removed.insert(t.clone());
                        } else {
                            dead_removal.insert(t.clone());
                        }
                    }
                }
            }
            if PRUNER_NAME.is_match(&c.name) {
                for a in &c.args {
                    let t = txt(*a, src).trim();
                    if names.contains(t) {
                        removed.insert(t.to_owned());
                    }
                }
            }
        }
        walk(f.tree.root_node(), &mut |n| {
            if let Some((name, removal)) = kotlin_assignment(f, n)
                && names.contains(name.as_str())
            {
                if removal {
                    if removal_is_live(f, n, uses) {
                        removed.insert(name);
                    } else {
                        dead_removal.insert(name);
                    }
                } else if in_runtime_code(n) {
                    *adds.entry(name).or_default() += 1;
                }
            }
            if n.kind().contains("binary") || n.kind().contains("comparison") {
                let t = txt(n, src);
                if (t.contains('<') || t.contains('>'))
                    && (t.contains(".size") || t.contains(".count()") || t.contains(".length"))
                {
                    for tok in tokens(t) {
                        if names.contains(tok) {
                            removed.insert(tok.to_owned());
                        }
                    }
                }
            }
        });
    }
    let mut out = Vec::new();
    for d in decls {
        let n = adds.get(&d.name).copied().unwrap_or(0);
        if n > 0 && !removed.contains(&d.name) {
            let note = if dead_removal.contains(&d.name) {
                "; the only removal sits in a method that production code never calls"
            } else {
                ""
            };
            out.push(hit(
                UNBOUNDED,
                &files[d.file],
                d.line,
                format!(
                    "static or singleton collection `{}` is added to at {n} site(s) and nothing in the project removes from it, clears it or checks its size{note}",
                    d.name
                ),
            ));
        }
    }
    out
}

static LISTENER_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(add|register|attach)([A-Z]\w*)?(Listener|Observer|Callback|Handler|Hook|Subscriber|Watcher|Consumer)s?$",
    )
    .expect("regex")
});

static BUS_RECEIVER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)bus|broker|dispatcher|registry|publisher|subject|topic|emitter|events?$|hub")
        .expect("regex")
});

fn counterparts(name: &str) -> Vec<String> {
    let lower = name.to_string();
    let mut out = Vec::new();
    if let Some(rest) = lower.strip_prefix("add") {
        out.push(format!("remove{rest}"));
        out.push(format!("removeAll{}", rest.trim_start_matches("All")));
    } else if let Some(rest) = lower.strip_prefix("register") {
        out.push(format!("unregister{rest}"));
        out.push(format!("deregister{rest}"));
    } else if let Some(rest) = lower.strip_prefix("attach") {
        out.push(format!("detach{rest}"));
    }
    if lower == "subscribe" {
        for n in ["unsubscribe", "dispose", "cancel", "close"] {
            out.push(n.to_owned());
        }
    }
    out
}

fn enclosing_fn<'t>(n: Node<'t>) -> Option<Node<'t>> {
    ancestor(
        n,
        &[
            "method_declaration",
            "constructor_declaration",
            "function_declaration",
            "secondary_constructor",
            "static_initializer",
            "anonymous_initializer",
        ],
    )
}

fn result_is_kept(n: Node<'_>) -> bool {
    n.parent().is_some_and(|p| {
        matches!(
            p.kind(),
            "variable_declarator"
                | "assignment_expression"
                | "assignment"
                | "return_statement"
                | "property_declaration"
                | "jump_expression"
                | "variable_declaration"
        )
    })
}

fn declared_locally(file: &SrcFile, func: Node<'_>, name: &str) -> bool {
    let kinds: &[&str] = match file.lang {
        Lang::Kotlin => &["variable_declaration"],
        _ => &["variable_declarator"],
    };
    let src = file.text.as_str();
    find_all(func, kinds).into_iter().any(|d| {
        let id = d
            .child_by_field_name("name")
            .or_else(|| child_of_kind(d, &["identifier"]));
        id.is_some_and(|i| txt(i, src) == name)
    })
}

fn listeners(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let all_names: HashSet<&str> = calls_by_file
        .iter()
        .flatten()
        .map(|c| c.name.as_str())
        .collect();
    let mut out = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        for c in &calls_by_file[i] {
            let is_listener = LISTENER_NAME.is_match(&c.name)
                || (c.name == "subscribe" && BUS_RECEIVER.is_match(&c.recv_last()));
            if !is_listener || c.name == "addShutdownHook" {
                continue;
            }
            let Some(func) = enclosing_fn(c.node) else {
                continue;
            };
            if INIT_KINDS.contains(&func.kind()) || result_is_kept(c.node) {
                continue;
            }
            let fn_text = txt(func, src);
            let head = fn_text.split('{').next().unwrap_or("");
            let fname = func
                .child_by_field_name("name")
                .or_else(|| child_of_kind(func, &["identifier"]))
                .map_or("", |n| txt(n, src));
            if fname == "main" || head.contains("@Bean") || head.contains("@PostConstruct") {
                continue;
            }
            let recv = c.recv_last();
            if !recv.is_empty() && declared_locally(f, func, &recv) {
                continue;
            }
            if counterparts(&c.name)
                .iter()
                .any(|n| all_names.contains(n.as_str()))
            {
                continue;
            }
            out.push(hit(
                LISTENER,
                f,
                line(c.node),
                format!(
                    "`{}` registers a callback and the project never calls a matching removal ({})",
                    c.name,
                    counterparts(&c.name)
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "unregister".into())
                ),
            ));
        }
    }
    out
}

static TAINTED_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(request|req|httprequest|servletrequest|exchange|uri|url|path|ip|uuid|email|username|useragent|query|querystring|param|params|header|headers|body|payload|session|sessionid|principal|referer|referrer|host|remoteaddr|clientip|requesturi|requesturl|pathinfo|servletpath|cookie|cookies|tenant|tenantid)$",
    )
    .expect("regex")
});

static TAINTED_GETTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(getRequest|getParameter|getHeader|getRemote|getQuery|getPath|getUri|getUrl|getSession|getCookie|getAttribute|getClientIp)")
        .expect("regex")
});

static REQUEST_PARAM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)@RequestParam|@PathVariable|@RequestHeader|@RequestBody|@QueryParam|@PathParam|@HeaderParam|@CookieValue|HttpServletRequest|ServerHttpRequest|ServerRequest|ServerWebExchange|\bRequest\b|RequestEntity|HttpRequest|\bExchange\b|ApplicationCall",
    )
    .expect("regex")
});

fn request_params(func: Node<'_>, src: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for p in find_all(func, &["formal_parameter", "parameter", "spread_parameter"]) {
        let t = txt(p, src);
        if REQUEST_PARAM.is_match(t) {
            let name = p
                .child_by_field_name("name")
                .or_else(|| child_of_kind(p, &["identifier"]))
                .map(|n| txt(n, src).to_owned());
            if let Some(n) = name {
                out.insert(n);
            }
        }
    }
    out
}

fn is_tainted(expr: Node<'_>, src: &str, params: &HashSet<String>) -> bool {
    if is_literal(expr) {
        return false;
    }
    tokens(txt(expr, src)).into_iter().any(|t| {
        params.contains(t)
            || TAINTED_TOKEN.is_match(t)
            || t == "id"
            || t.ends_with("Id")
            || t.ends_with("ID")
            || t.ends_with("_id")
            || TAINTED_GETTER.is_match(t)
    })
}

fn micrometer(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let mut out = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        if !(src.contains("micrometer")
            || src.contains("MeterRegistry")
            || src.contains("Metrics."))
        {
            continue;
        }
        for c in &calls_by_file[i] {
            let recv_last = c.recv_last();
            let value_args: Vec<Node<'_>> = match c.name.as_str() {
                "of" if recv_last == "Tag" && c.args.len() == 2 => vec![c.args[1]],
                "tag" if c.args.len() == 2 => vec![c.args[1]],
                "tags" => c
                    .args
                    .iter()
                    .enumerate()
                    .filter(|(k, _)| k % 2 == 1)
                    .map(|(_, a)| *a)
                    .collect(),
                "of" if recv_last == "Tags" => c
                    .args
                    .iter()
                    .enumerate()
                    .filter(|(k, _)| k % 2 == 1)
                    .map(|(_, a)| *a)
                    .collect(),
                "counter" | "timer" | "gauge" | "summary" | "longTaskTimer"
                    if (recv_last.to_ascii_lowercase().contains("registry")
                        || recv_last == "Metrics"
                        || recv_last.to_ascii_lowercase().contains("meter"))
                        && c.args.len() >= 3 =>
                {
                    c.args
                        .iter()
                        .enumerate()
                        .skip(1)
                        .filter(|(k, _)| k % 2 == 0)
                        .map(|(_, a)| *a)
                        .collect()
                }
                _ => continue,
            };
            let func = ancestor(c.node, &FN_KINDS);
            let params = func
                .map(|fnode| request_params(fnode, src))
                .unwrap_or_default();
            if let Some(bad) = value_args.iter().find(|a| is_tainted(**a, src, &params)) {
                out.push(hit(
                    MICROMETER,
                    f,
                    line(c.node),
                    format!(
                        "metric tag value `{}` comes from request data, so the number of meters grows with distinct inputs",
                        txt(*bad, src).chars().take(60).collect::<String>()
                    ),
                ));
            }
        }
    }
    out
}

/// Runs the JVM rules over the Java and Kotlin files.
pub fn check(files: &[SrcFile]) -> Vec<StaticHit> {
    let calls_by_file: Vec<Vec<CallSite<'_>>> =
        files.iter().map(|f| calls(f, f.tree.root_node())).collect();
    let uses = UseIndex::build(files, &calls_by_file);
    let mut hits = unbounded_collections(files, &calls_by_file, &uses);
    hits.extend(listeners(files, &calls_by_file));
    hits.extend(micrometer(files, &calls_by_file));
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
    fn collections_in_superclass_of_singleton_and_registries_are_flagged() {
        let inherited = "import java.util.*;\nabstract class Base {\n  protected final Map<String, Object> store = new HashMap<>();\n  protected void remember(String k, Object v) { store.put(k, v); }\n}\n@Service\nclass SessionService extends Base {\n  void open(String id) { remember(id, new Object()); }\n}\n";
        let hits = run(&[("src/S.java", inherited)]);
        assert_eq!(sites(&hits, UNBOUNDED), ["src/S.java:3"]);
        let bus = "import java.util.*;\nclass Bus {\n  private final List<Runnable> subs = new ArrayList<>();\n  void subscribe(Runnable r) { subs.add(r); }\n}\n";
        let hits = run(&[("src/Bus.java", bus)]);
        assert_eq!(sites(&hits, UNBOUNDED), ["src/Bus.java:3"]);
        let with_removal = bus.replace(
            "}\n}\n",
            "}\n  void unsubscribe(Runnable r) { subs.remove(r); }\n}\nclass User { void f(Bus b, Runnable r) { b.unsubscribe(r); } void g(Bus b, Runnable r) { f(b, r); } }\n",
        );
        assert!(sites(&run(&[("src/Bus.java", &with_removal)]), UNBOUNDED).is_empty());
        let transient = "import java.util.*;\nclass Plain {\n  private final List<String> xs = new ArrayList<>();\n  void put(String s) { xs.add(s); }\n}\n";
        assert!(sites(&run(&[("src/P.java", transient)]), UNBOUNDED).is_empty());
    }

    #[test]
    fn java_static_map_without_removal_is_flagged() {
        let hits = run(&[(
            "src/Reg.java",
            "import java.util.*;\nclass Reg {\n  private static final Map<String, Object> CACHE = new HashMap<>();\n  static void put(String k, Object v) { CACHE.put(k, v); }\n}\n",
        )]);
        assert_eq!(sites(&hits, UNBOUNDED), ["src/Reg.java:3"]);
    }

    #[test]
    fn java_removal_size_check_or_eviction_makes_it_quiet() {
        let removed = "class A {\n  static final Map<String,String> M = new HashMap<>();\n  static void a(String k) { M.put(k, k); }\n  static void close() { M.clear(); }\n}\n";
        let sized = "class A {\n  static final List<String> L = new ArrayList<>();\n  static void a(String k) { if (L.size() > 100) { return; } L.add(k); }\n}\n";
        let lru = "class A {\n  static final Map<String,String> M = Collections.synchronizedMap(new LinkedHashMap<>(16, 0.75f, true) {\n    protected boolean removeEldestEntry(Map.Entry<String,String> e) { return size() > 10; }\n  });\n  static void a(String k) { M.put(k, k); }\n}\n";
        let caffeine = "class A {\n  static final Cache<String,String> M = Caffeine.newBuilder().maximumSize(10).build();\n  static void a(String k) { M.put(k, k); }\n}\n";
        let removed_elsewhere = "class B { void f() { A.M.remove(\"x\"); } void g() { f(); } }\n";
        for (name, src) in [
            ("removed", removed),
            ("sized", sized),
            ("lru", lru),
            ("caffeine", caffeine),
        ] {
            let hits = run(&[("src/A.java", src)]);
            assert!(sites(&hits, UNBOUNDED).is_empty(), "{name}: {hits:?}");
        }
        let two = "class A {\n  static final Map<String,String> M = new HashMap<>();\n  static void a(String k) { M.put(k, k); }\n}\n";
        assert!(!sites(&run(&[("A.java", two)]), UNBOUNDED).is_empty());
        assert!(
            sites(
                &run(&[("A.java", two), ("B.java", removed_elsewhere)]),
                UNBOUNDED
            )
            .is_empty()
        );
    }

    #[test]
    fn java_removal_only_in_an_uncalled_method_does_not_count() {
        let src = "class C {\n  private static final Map<String,String> M = new HashMap<>();\n  static void store(String k) { M.put(k, k); }\n  static void invalidateAll() { M.clear(); }\n}\n";
        let hits = run(&[("C.java", src)]);
        assert_eq!(sites(&hits, UNBOUNDED), ["C.java:2"]);
        assert!(hits[0].message.contains("never calls"));
        let used = format!("{src}class T {{ void t() {{ C.invalidateAll(); }} }}\n");
        assert!(run(&[("C.java", &used)]).is_empty());
        let by_ref = format!("{src}class T {{ void t() {{ run(C::invalidateAll); }} }}\n");
        assert!(run(&[("C.java", &by_ref)]).is_empty());
        let annotated = src.replace(
            "static void invalidateAll",
            "@PreDestroy static void invalidateAll",
        );
        assert!(run(&[("C.java", &annotated)]).is_empty());
    }

    #[test]
    fn java_static_init_adds_and_plain_instance_fields_are_ignored() {
        let src = "class A {\n  static final Map<String,String> M = new HashMap<>();\n  static { M.put(\"a\", \"b\"); }\n  private final List<String> items = new ArrayList<>();\n  void f() { items.add(\"x\"); }\n}\n";
        assert!(run(&[("A.java", src)]).is_empty());
    }

    #[test]
    fn java_singleton_service_collection_is_flagged() {
        let src = "@Service\nclass Sessions {\n  private final Map<String, Session> live = new ConcurrentHashMap<>();\n  void open(String id) { live.put(id, new Session()); }\n}\n";
        assert_eq!(
            sites(&run(&[("Sessions.java", src)]), UNBOUNDED),
            ["Sessions.java:3"]
        );
    }

    #[test]
    fn kotlin_object_collection_rules() {
        let leaky = "object Registry {\n    val items = mutableListOf<String>()\n    fun add(x: String) { items.add(x) }\n}\n";
        assert_eq!(sites(&run(&[("R.kt", leaky)]), UNBOUNDED), ["R.kt:2"]);
        let idx = "object Registry {\n    private val map = HashMap<String, Int>()\n    fun add(x: String) { map[x] = 1 }\n}\n";
        assert_eq!(sites(&run(&[("R.kt", idx)]), UNBOUNDED), ["R.kt:2"]);
        let cleared = "object Registry {\n    val items = mutableListOf<String>()\n    fun add(x: String) { items.add(x) }\n    fun reset() { items.clear() }\n}\nfun main() { Registry.reset() }\n";
        assert!(run(&[("R.kt", cleared)]).is_empty());
        let local = "fun f() {\n    val tmp = mutableListOf<String>()\n    tmp.add(\"a\")\n}\n";
        assert!(run(&[("R.kt", local)]).is_empty());
        let readonly = "object Registry {\n    val items = listOf(\"a\")\n    fun add(x: String) { items.add(x) }\n}\n";
        assert!(run(&[("R.kt", readonly)]).is_empty());
    }

    #[test]
    fn kotlin_companion_collection_is_flagged() {
        let src = "class A {\n    companion object {\n        val seen = ArrayList<String>()\n    }\n    fun f(x: String) {\n        seen.add(x)\n    }\n}\n";
        assert_eq!(sites(&run(&[("A.kt", src)]), UNBOUNDED), ["A.kt:3"]);
    }

    #[test]
    fn listener_without_removal_is_flagged_and_paired_removal_is_not() {
        let leak = "class W {\n  void start(Bus bus) { bus.addListener(e -> handle(e)); }\n}\n";
        assert_eq!(sites(&run(&[("W.java", leak)]), LISTENER), ["W.java:2"]);
        let paired = "class W {\n  Listener l = e -> {};\n  void start(Bus bus) { bus.addListener(l); }\n  void stop(Bus bus) { bus.removeListener(l); }\n}\n";
        assert!(run(&[("W.java", paired)]).is_empty());
        let local =
            "class W {\n  void f() { Button b = new Button(); b.addActionListener(e -> {}); }\n}\n";
        assert!(run(&[("W.java", local)]).is_empty());
        let kept = "class W {\n  void f(Bus bus) { Disposable d = bus.registerListener(x -> {}); this.d = d; }\n}\n";
        assert!(run(&[("W.java", kept)]).is_empty());
        let kt = "class W {\n    fun start(bus: Bus) {\n        bus.addListener { println(it) }\n    }\n}\n";
        assert_eq!(sites(&run(&[("W.kt", kt)]), LISTENER), ["W.kt:3"]);
        let kt_ok = "class W {\n    fun start(bus: Bus) {\n        bus.registerHandler(h)\n    }\n    fun stop(bus: Bus) {\n        bus.unregisterHandler(h)\n    }\n}\n";
        assert!(run(&[("W.kt", kt_ok)]).is_empty());
    }

    #[test]
    fn micrometer_tags_from_request_data() {
        let bad = "import io.micrometer.core.instrument.*;\nclass M {\n  void f(HttpServletRequest request, MeterRegistry registry) {\n    registry.counter(\"hits\", \"uri\", request.getRequestURI()).increment();\n    Timer.builder(\"t\").tag(\"user\", userId).register(registry);\n    Tag.of(\"k\", request.getPathInfo());\n  }\n}\n";
        let hits = run(&[("M.java", bad)]);
        assert_eq!(
            sites(&hits, MICROMETER),
            ["M.java:4", "M.java:5", "M.java:6"]
        );
        let good = "import io.micrometer.core.instrument.*;\nclass M {\n  void f(MeterRegistry registry, Status status) {\n    registry.counter(\"hits\", \"method\", \"GET\").increment();\n    Timer.builder(\"t\").tag(\"status\", status.name()).register(registry);\n    Tag.of(\"k\", \"v\");\n  }\n}\n";
        assert!(run(&[("M.java", good)]).is_empty());
        let unrelated =
            "class M { void f(Request request) { builder.tag(\"a\", request.getPath()); } }\n";
        assert!(
            run(&[("M.java", unrelated)]).is_empty(),
            "needs micrometer in the file"
        );
        let kt = "import io.micrometer.core.instrument.Tag\nfun f(request: ServerRequest) {\n    Tag.of(\"path\", request.path())\n}\n";
        assert_eq!(sites(&run(&[("M.kt", kt)]), MICROMETER), ["M.kt:3"]);
    }
}
