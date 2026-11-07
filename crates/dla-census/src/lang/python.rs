//! Python holders.

use std::sync::OnceLock;

use dla_core::finding::{Holder, Region};
use regex::Regex;

use super::{CallRule, Cx, Table, class_path, dedupe, describe, join_owner, link_name, rules};
use crate::{
    ast::{ancestors, field_text, line, text, visit},
    model::{ProjectInfo, Raw, Scope},
    parse::{Lang, parse, tree_sitter::Node},
    vocab,
};

fn call_table() -> &'static [CallRule] {
    static T: Table = Table::new();
    T.get_or_init(|| {
        use Holder::*;
        use Region::*;
        rules(&[
            (r"(?:^|\.)(?:Thread|Timer|Process|ThreadPoolExecutor|ProcessPoolExecutor|Pool|Popen)$", "spawn", BlockedStack, NonHeap, true),
            (r"(?:^|\.)(?:create_task|ensure_future|run_in_executor|start_new_thread|call_later|call_at|call_soon|call_soon_threadsafe|run_coroutine_threadsafe)$", "async-task", BlockedStack, NonHeap, true),
            (r"^(?:signal\.signal|atexit\.register|weakref\.finalize)$|\.(?:addHandler|add_done_callback|add_signal_handler|add_listener|add_handler|add_observer|subscribe|register|register_\w+|on)$", "registration", RegistrationTable, Heap, false),
            (r"^(?:socket\.socket|sqlite3\.connect|urllib\.request\.urlopen|urlopen|mmap\.mmap|tempfile\.\w+|ctypes\.\w*CDLL|ctypes\.cdll\.LoadLibrary|psycopg2\.connect|zipfile\.ZipFile|tarfile\.open|asyncio\.open_connection|aiohttp\.ClientSession|requests\.Session|open|io\.open|os\.open|os\.popen)$", "handle", NativeHandleGlobalRef, OffHeap, true),
            (r"(?:^|\.)(?:local|ContextVar)$", "thread-local", ThreadLocal, Heap, true),
        ])
    })
}

fn container_call() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?:^|\.)(?:dict|list|set|deque|defaultdict|OrderedDict|Counter|ChainMap|WeakValueDictionary|WeakKeyDictionary|WeakSet|bytearray|array|Queue|SimpleQueue|LifoQueue|PriorityQueue|ExitStack|WeakMethod)(?:\.\w+)?$").expect("static")
    })
}

fn singleton_marker() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)singleton|\b(?:cls|type\(self\))\._instance\b|get_instance|\b_instance\s*=\s*None")
            .expect("static")
    })
}

/// Records which classes are plausibly process-lifetime (instantiated at module
/// level or from startup code, singleton-shaped) and which types each class holds.
pub fn collect(src: &str, info: &mut ProjectInfo) {
    let Ok(tree) = parse(Lang::Python, src) else {
        return;
    };
    visit(tree.root_node(), &mut |n| match n.kind() {
        "class_definition" => {
            let name = field_text(n, "name", src).unwrap_or("");
            if singleton_marker().is_match(text(n, src)) {
                info.longlived.insert(name.to_owned());
            }
        }
        "assignment" => {
            let (Some(l), Some(r)) = (
                n.child_by_field_name("left"),
                n.child_by_field_name("right"),
            ) else {
                return;
            };
            if !text(l, src).starts_with("self.") {
                return;
            }
            if let Some(c) = ancestors(n).find(|a| a.kind() == "class_definition") {
                let cname = field_text(c, "name", src).unwrap_or("");
                info.note_fields(cname, text(r, src));
            }
        }
        "function_definition" => {
            if super::entry_fn_name(field_text(n, "name", src).unwrap_or("")) {
                info.longlived.extend(super::caps_idents(text(n, src)));
            }
        }
        "call" if !ancestors(n).any(|a| a.kind() == "function_definition") => {
            if let Some(f) = n.child_by_field_name("function") {
                info.longlived.extend(super::caps_idents(text(f, src)));
            }
        }
        _ => {}
    });
}

fn cache_decorator() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^@\s*(?:functools\.)?(?:lru_cache|cache|memoize|cached|ttl_cache|memoized)\b")
            .expect("static")
    })
}

fn handleish() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)(instance|singleton|current|conn|socket|server|client|pool|registry|cache)",
        )
        .expect("static")
    })
}

fn value_class(
    v: Node<'_>,
    src: &str,
    name: &str,
    module_level: bool,
    rebound: bool,
) -> Option<(&'static str, Holder)> {
    let registration = vocab::registration_name().is_match(name);
    let pick = |kind: &'static str, h: Holder| {
        if registration && h == Holder::StaticGlobal {
            (kind, Holder::RegistrationTable)
        } else {
            (kind, h)
        }
    };
    let upper = name
        .chars()
        .all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit());
    match v.kind() {
        "list" | "dictionary" | "set" => {
            (v.named_child_count() == 0 || !upper).then(|| pick("container", Holder::StaticGlobal))
        }
        "list_comprehension" | "dictionary_comprehension" | "set_comprehension" => {
            Some(pick("container", Holder::StaticGlobal))
        }
        "call" => {
            let f = field_text(v, "function", src).unwrap_or("");
            let base = f.rsplit('.').next().unwrap_or(f);
            if matches!(base, "local" | "ContextVar") {
                return Some(("thread-local", Holder::ThreadLocal));
            }
            if container_call().is_match(f) {
                return Some(pick("container", Holder::StaticGlobal));
            }
            let capital = base.chars().next().is_some_and(char::is_uppercase);
            let factory = base.starts_with("create_")
                || base.starts_with("make_")
                || base.starts_with("build_")
                || base.starts_with("get_or_create");
            if (capital || factory) && (module_level || container_name(base)) {
                return Some(pick("instance", Holder::StaticGlobal));
            }
            None
        }
        "lambda" => module_level.then_some(("closure", Holder::ClosureContinuation)),
        "none" => (link_name(name) || handleish().is_match(name) || rebound)
            .then_some(("pointer", Holder::StaticGlobal)),
        _ => None,
    }
}

/// A module-level name that some function rebinds through a `global` statement.
fn global_rebound(src: &str, name: &str) -> bool {
    Regex::new(&format!(
        r"(?m)^\s*global\s+(?:[\w, ]*\b){}\b",
        regex::escape(name)
    ))
    .is_ok_and(|r| r.is_match(src))
}

fn container_name(base: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?:Cache|Registry|Pool|Queue|List|Map|Set|Stack|Buffer|Store|Table|Index|Bus|Hub|Tracker|Memo|Log|History|Backlog)$").expect("static")
    })
    .is_match(base)
}

/// Enumerates the holders of one Python file.
pub fn holders(cx: &mut Cx<'_>, root: Node<'_>) {
    let src = cx.src();
    let prefix = cx.owner_prefix();
    let mut found: Vec<Raw> = Vec::new();
    let mut seen_attr: std::collections::HashSet<String> = std::collections::HashSet::new();
    visit(root, &mut |n| match n.kind() {
        "assignment" => {
            let (Some(l), Some(r)) = (
                n.child_by_field_name("left"),
                n.child_by_field_name("right"),
            ) else {
                return;
            };
            let func = ancestors(n).find(|a| a.kind() == "function_definition");
            let class = ancestors(n).find(|a| a.kind() == "class_definition");
            match (func, l.kind()) {
                (None, "identifier") => {
                    let name = text(l, src);
                    if name.starts_with("__") && name.ends_with("__") {
                        return;
                    }
                    let rebound = global_rebound(src, name);
                    let Some((kind, holder)) = value_class(r, src, name, true, rebound) else {
                        return;
                    };
                    let cp = class_path(n, src, Lang::Python);
                    let (owner, kind, member) = if class.is_some() {
                        (
                            format!("{cp}.{name}"),
                            if holder == Holder::ClosureContinuation {
                                "class-closure"
                            } else {
                                "class-attr"
                            },
                            true,
                        )
                    } else {
                        (
                            name.to_owned(),
                            if holder == Holder::ThreadLocal {
                                "thread-local"
                            } else if kind == "pointer" {
                                "module-pointer"
                            } else {
                                "module-global"
                            },
                            false,
                        )
                    };
                    found.push(
                        Raw::new(
                            kind,
                            holder,
                            Region::Heap,
                            &cx.file.rel,
                            line(n),
                            join_owner(&prefix, &owner),
                        )
                        .tracking(name, Scope::Project, member),
                    );
                }
                (Some(_), "attribute") => {
                    let obj = field_text(l, "object", src).unwrap_or("");
                    let attr = field_text(l, "attribute", src).unwrap_or("");
                    if !matches!(obj, "self" | "cls") {
                        return;
                    }
                    let rebound = class
                        .is_some_and(|c| super::assigned_times(text(c, src), "self.", attr, 2));
                    let Some((kind, holder)) = value_class(r, src, attr, false, rebound) else {
                        return;
                    };
                    let cp = class_path(n, src, Lang::Python);
                    let (hk, h) = match holder {
                        Holder::StaticGlobal if obj == "cls" => {
                            ("class-attr", Holder::StaticGlobal)
                        }
                        Holder::StaticGlobal => ("instance-field", Holder::ContainerField),
                        Holder::RegistrationTable => ("registry-field", Holder::RegistrationTable),
                        other => ("instance-field", other),
                    };
                    if h == Holder::ContainerField {
                        // Only collections (or the link of a linked structure) held by
                        // a plausibly process-lifetime class are container fields.
                        let owner_class = cp.rsplit('.').next().unwrap_or("");
                        if !(kind == "container" || kind == "instance" || link_name(attr))
                            || !(cx.info.is_long_lived(owner_class)
                                || cx.info.selfref.contains(owner_class))
                        {
                            return;
                        }
                    }
                    // Repeated assignments to one attribute are one holder.
                    if !seen_attr.insert(format!("{cp}.{attr}")) {
                        return;
                    }
                    let scope = if attr.starts_with('_') {
                        Scope::File
                    } else {
                        Scope::Project
                    };
                    found.push(
                        Raw::new(
                            hk,
                            h,
                            Region::Heap,
                            &cx.file.rel,
                            line(n),
                            join_owner(&prefix, &format!("{cp}.{attr}")),
                        )
                        .tracking(attr, scope, true),
                    );
                }
                _ => {}
            }
        }
        "decorated_definition" => {
            let Some(def) = n.child_by_field_name("definition") else {
                return;
            };
            let name = def
                .child_by_field_name("name")
                .map(|c| text(c, src))
                .unwrap_or("");
            let mut c = n.walk();
            for d in n.named_children(&mut c).filter(|d| d.kind() == "decorator") {
                if cache_decorator().is_match(text(d, src).trim()) {
                    let cp = class_path(n, src, Lang::Python);
                    let owner = if cp.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{cp}.{name}")
                    };
                    found.push(
                        Raw::new(
                            "cache-decorator",
                            Holder::StaticGlobal,
                            Region::Heap,
                            &cx.file.rel,
                            line(d),
                            join_owner(&prefix, &owner),
                        )
                        .tracking(name, Scope::Project, false),
                    );
                }
            }
        }
        "call" => {
            let Some(f) = n.child_by_field_name("function") else {
                return;
            };
            if ancestors(n).any(|a| a.kind() == "with_item") {
                return;
            }
            let callee = text(f, src);
            let recv = (f.kind() == "attribute")
                .then(|| field_text(f, "object", src))
                .flatten();
            let mut tmp = Cx::new(cx.file, cx.info);
            tmp.call(n, callee, recv, call_table());
            found.extend(tmp.out);
            let _ = describe;
        }
        _ => {}
    });
    cx.out.extend(dedupe(found));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{ProjectInfo, SourceFile},
        parse::parse,
    };

    fn run(src: &str) -> Vec<Raw> {
        let f = SourceFile {
            rel: "pkg/reg.py".into(),
            text: src.into(),
            lang: Lang::Python,
            is_test: false,
        };
        let mut info = ProjectInfo::default();
        collect(src, &mut info);
        info.finish();
        let tree = parse(Lang::Python, src).unwrap();
        let mut cx = Cx::new(&f, &info);
        holders(&mut cx, tree.root_node());
        cx.out
    }

    #[test]
    fn module_class_instance_and_calls() {
        let src = "import threading\n_LISTENERS = {}\nLIMITS = (1, 2)\nNAMES = ['a']\nstore = Store()\nclass C:\n    seen = set()\n    def __init__(self):\n        self.rows = []\n        self.n = 0\n        self.lock = threading.local()\n@lru_cache(maxsize=None)\ndef f(x):\n    t = threading.Thread(target=g)\n    with open('p') as fh:\n        pass\n    signal.signal(2, h)\napp = C()\n";
        let r = run(src);
        let got: Vec<(u32, &str, &str)> = r
            .iter()
            .map(|h| (h.line, h.kind.as_str(), h.holder.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (2, "module-global", "registration-table"),
                (5, "module-global", "static/global"),
                (7, "class-attr", "static/global"),
                (9, "instance-field", "container-field"),
                (11, "instance-field", "thread-local"),
                (12, "cache-decorator", "static/global"),
                (14, "spawn", "blocked-stack"),
                (17, "registration", "registration-table"),
                (18, "module-global", "static/global"),
            ]
        );
        assert_eq!(r[0].owner, "pkg.reg._LISTENERS");
        assert_eq!(r[3].owner, "pkg.reg.C.rows");
    }

    #[test]
    fn transient_owners_and_non_collections_are_not_container_fields() {
        // `Parsed` is a result object; `Server` is a role name; `Worker` is built at
        // module level; `_fut` is rebound but is not a collection or a link.
        let src = "class Parsed:\n    def __init__(self):\n        self.hits = []\n\nclass Server:\n    def __init__(self):\n        self._waiters = []\n        self._fut = None\n    def start(self):\n        self._waiters = []\n        self._fut = None\n\nclass Worker:\n    def __init__(self):\n        self.jobs = {}\n\nw = Worker()\n";
        let r = run(src);
        let fields: Vec<&str> = r
            .iter()
            .filter(|h| h.kind == "instance-field")
            .map(|h| h.owner.as_str())
            .collect();
        assert_eq!(fields, ["pkg.reg.Server._waiters", "pkg.reg.Worker.jobs"]);
    }

    #[test]
    fn class_held_by_a_long_lived_class_is_reachable() {
        let src = "class Entry:\n    def __init__(self):\n        self.tags = []\n\nclass Registry:\n    def __init__(self):\n        self.e = Entry()\n";
        let r = run(src);
        assert!(r.iter().any(|h| h.owner == "pkg.reg.Entry.tags"), "{r:?}");
    }
}
