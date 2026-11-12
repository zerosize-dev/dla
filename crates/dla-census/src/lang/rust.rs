//! Rust holders.

use std::sync::OnceLock;

use dla_core::finding::{Holder, Region};
use regex::Regex;

use super::{CallRule, Cx, Table, dedupe, join_owner, rules};
use crate::{
    ast::{ancestors, children_of, line, text, visit},
    model::{ProjectInfo, Raw, Scope},
    parse::{Lang, parse, tree_sitter::Node},
    vocab,
};

fn container() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"\b(?:Vec|VecDeque|HashMap|HashSet|BTreeMap|BTreeSet|BinaryHeap|LinkedList|IndexMap|IndexSet|SlotMap|Slab|Arena|LruCache|Cache|Sender|SyncSender|UnboundedSender|Receiver|UnboundedReceiver|Pool|JoinHandle|Bytes|BytesMut)\b|\[[^\]]*;\s*\w+\]|\[\w+\]").expect("static")
    })
}

fn scalar_static() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^\s*(?:&'static\s+)?(?:&?str|u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize|f32|f64|bool|char|Atomic\w+|\[\s*&?(?:'static\s+)?str\s*;\s*\w+\]|\[\s*\w+\s*;\s*\w+\])\s*$").expect("static")
    })
}

fn rc_target(ty: &str) -> Option<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"\b(?:Rc|Arc)\s*<\s*(?:(?:RefCell|Mutex|RwLock|Cell)\s*<\s*)?([A-Z]\w*)")
            .expect("static")
    });
    if ty.contains("Weak<") {
        return None;
    }
    re.captures(ty).map(|c| c[1].to_owned())
}

fn weak_target(ty: &str) -> Option<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"\bWeak\s*<\s*(?:(?:RefCell|Mutex|RwLock|Cell)\s*<\s*)?([A-Z]\w*)")
            .expect("static")
    });
    re.captures(ty).map(|c| c[1].to_owned())
}

/// Records the Rc/Arc edges and Weak fields of every struct in a source.
pub fn collect(src: &str, info: &mut ProjectInfo) {
    let Ok(tree) = parse(Lang::Rust, src) else {
        return;
    };
    visit(tree.root_node(), &mut |n| {
        match n.kind() {
            "static_item" | "macro_invocation" | "const_item" => {
                if n.kind() != "macro_invocation"
                    || text(n, src).starts_with("lazy_static")
                    || text(n, src).starts_with("thread_local")
                {
                    info.longlived.extend(super::caps_idents(text(n, src)));
                }
                return;
            }
            "function_item" => {
                let name = n.child_by_field_name("name").map_or("", |c| text(c, src));
                if super::entry_fn_name(name) {
                    info.longlived.extend(super::caps_idents(text(n, src)));
                }
                return;
            }
            _ => {}
        }
        if n.kind() != "struct_item" {
            return;
        }
        let (Some(name), Some(body)) =
            (n.child_by_field_name("name"), n.child_by_field_name("body"))
        else {
            return;
        };
        let name = text(name, src).to_owned();
        for f in children_of(body, "field_declaration") {
            let Some(t) = f.child_by_field_name("type") else {
                continue;
            };
            let ty = text(t, src);
            info.note_fields(&name, ty);
            if let Some(w) = weak_target(ty) {
                info.weak.insert((name.clone(), w));
            }
            if let Some(target) = rc_target(ty) {
                let is_container = container().is_match(ty);
                info.rc_edges
                    .entry(name.clone())
                    .or_default()
                    .push((target, is_container));
            }
        }
    });
}

fn in_cycle(s: &str, target: &str, info: &ProjectInfo) -> bool {
    if info.weak.contains(&(s.to_owned(), target.to_owned())) {
        return false;
    }
    let edges = info.rc_edges.get(s).map(Vec::as_slice).unwrap_or(&[]);
    if target == s {
        edges.iter().filter(|(t, _)| t == s).count() >= 2
    } else {
        info.rc_edges
            .get(target)
            .is_some_and(|back| back.iter().any(|(t, _)| t == s))
            && !info.weak.contains(&(target.to_owned(), s.to_owned()))
    }
}

/// Callee text without turbofish generics: `Box::<T>::into_raw` is `Box::into_raw`.
fn strip_turbofish(callee: &str) -> String {
    let mut out = String::with_capacity(callee.len());
    let mut rest = callee;
    while let Some(i) = rest.find("::<") {
        out.push_str(&rest[..i]);
        let mut depth = 0usize;
        let mut end = None;
        let mut prev = ' ';
        for (j, ch) in rest[i + 2..].char_indices() {
            let arrow = prev == '-';
            prev = ch;
            match ch {
                '<' => depth += 1,
                // The `>` of `->` (a `fn() -> u8` type) closes nothing.
                '>' if arrow => {}
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i + 2 + j + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        match end {
            Some(e) => rest = &rest[e..],
            None => return callee.to_owned(),
        }
    }
    out.push_str(rest);
    out
}

/// A function path that, passed as a value, leaks or hands out a raw pointer.
fn leaking_path() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?:^|::)Box::(?:into_raw|into_non_null|leak)$").expect("leaking path")
    })
}

fn call_table() -> &'static [CallRule] {
    static T: Table = Table::new();
    T.get_or_init(|| {
        use Holder::*;
        use Region::*;
        rules(&[
            (r"(?:^|::|\.)(?:spawn|spawn_blocking|spawn_local|spawn_scoped)$", "spawn", BlockedStack, NonHeap, true),
            (r"(?:^|::)(?:Box::leak|mem::forget|ManuallyDrop::new)$|::leak$|\.leak$", "leak", StaticGlobal, Heap, true),
            (r"(?:^|::)Box::(?:into_raw|into_non_null)$", "handle", NativeHandleGlobalRef, OffHeap, true),
            (r"(?:^|::)(?:channel|unbounded_channel|unbounded|sync_channel|bounded)$", "channel", ContainerField, Heap, true),
            (r"(?:^|::)(?:File::(?:open|create|create_new)|OpenOptions::.*open|TcpStream::connect|TcpListener::bind|UdpSocket::bind|UnixStream::connect|UnixListener::bind)$|\.open$", "handle", NativeHandleGlobalRef, OffHeap, true),
            (r"\.(?:subscribe|add_listener|add_observer|set_hook|set_handler|on_\w+|connect_\w+|register\w*)$", "registration", RegistrationTable, Heap, false),
        ])
    })
}

fn in_test_code(n: Node<'_>, src: &str) -> bool {
    ancestors(n).any(|a| {
        if a.kind() != "mod_item" && a.kind() != "function_item" {
            return false;
        }
        let mut prev = a.prev_sibling();
        while let Some(p) = prev {
            if p.kind() != "attribute_item" {
                break;
            }
            let t = text(p, src);
            if t.contains("cfg(test)") || t == "#[test]" {
                return true;
            }
            prev = p.prev_sibling();
        }
        a.kind() == "mod_item"
            && a.child_by_field_name("name")
                .is_some_and(|nm| text(nm, src) == "tests")
    })
}

/// Enumerates the holders of one Rust file.
pub fn holders(cx: &mut Cx<'_>, root: Node<'_>) {
    let src = cx.src();
    let prefix = cx.owner_prefix();
    let mut found: Vec<Raw> = Vec::new();
    visit(root, &mut |n| {
        if in_test_code(n, src) {
            return;
        }
        match n.kind() {
            "static_item" => {
                let name = n
                    .child_by_field_name("name")
                    .map(|c| text(c, src))
                    .unwrap_or("");
                let ty = n
                    .child_by_field_name("type")
                    .map(|c| text(c, src))
                    .unwrap_or("");
                if scalar_static().is_match(ty) || name == "_" {
                    return;
                }
                let holder = if vocab::registration_name().is_match(name)
                    || vocab::registration_name().is_match(ty)
                {
                    Holder::RegistrationTable
                } else {
                    Holder::StaticGlobal
                };
                found.push(
                    Raw::new(
                        "static",
                        holder,
                        Region::Heap,
                        &cx.file.rel,
                        line(n),
                        join_owner(&prefix, name),
                    )
                    .tracking(name, Scope::Project, false),
                );
            }
            "macro_invocation" => {
                let mac = n
                    .child_by_field_name("macro")
                    .map(|c| text(c, src))
                    .unwrap_or("");
                let mac = mac.rsplit("::").next().unwrap_or(mac);
                if !matches!(mac, "lazy_static" | "thread_local") {
                    return;
                }
                static R: OnceLock<Regex> = OnceLock::new();
                let re = R.get_or_init(|| {
                    Regex::new(r"static\s+(?:ref\s+)?(\w+)\s*:\s*([^=]+?)\s*=").expect("static")
                });
                let body = text(n, src);
                for c in re.captures_iter(body) {
                    let m = c.get(0).map_or(0, |m| m.start());
                    let ln = line(n) + u32::try_from(body[..m].matches('\n').count()).unwrap_or(0);
                    let name = &c[1];
                    let (kind, holder) = if mac == "thread_local" {
                        ("thread-local", Holder::ThreadLocal)
                    } else {
                        ("lazy-static", Holder::StaticGlobal)
                    };
                    found.push(
                        Raw::new(
                            kind,
                            holder,
                            Region::Heap,
                            &cx.file.rel,
                            ln,
                            join_owner(&prefix, name),
                        )
                        .tracking(name, Scope::Project, false),
                    );
                }
            }
            "field_declaration" => {
                let Some(st) = n
                    .parent()
                    .and_then(|p| p.parent())
                    .filter(|s| s.kind() == "struct_item")
                else {
                    return;
                };
                let sname = st
                    .child_by_field_name("name")
                    .map(|c| text(c, src))
                    .unwrap_or("");
                let name = n
                    .child_by_field_name("name")
                    .map(|c| text(c, src))
                    .unwrap_or("");
                let ty = n
                    .child_by_field_name("type")
                    .map(|c| text(c, src))
                    .unwrap_or("");
                let public = text(n, src).trim_start().starts_with("pub");
                let (kind, holder) =
                    if let Some(t) = rc_target(ty).filter(|t| in_cycle(sname, t, cx.info)) {
                        let _ = t;
                        ("rc-cycle", Holder::CycleOnly)
                    } else if ty.contains("Box<dyn Fn")
                        || ty.contains("Box<dyn FnMut")
                        || ty.contains("Arc<dyn Fn")
                        || ty.contains("Rc<dyn Fn")
                    {
                        ("closure-field", Holder::ClosureContinuation)
                    } else if container().is_match(ty) {
                        if vocab::registration_name().is_match(name) {
                            ("registry-field", Holder::RegistrationTable)
                        } else if cx.info.is_long_lived(sname) {
                            ("container-field", Holder::ContainerField)
                        } else {
                            return;
                        }
                    } else {
                        return;
                    };
                let scope = if public { Scope::Project } else { Scope::File };
                found.push(
                    Raw::new(
                        kind,
                        holder,
                        Region::Heap,
                        &cx.file.rel,
                        line(n),
                        join_owner(&prefix, &format!("{sname}.{name}")),
                    )
                    .tracking(name, scope, true),
                );
            }
            "call_expression" => {
                let Some(f) = n.child_by_field_name("function") else {
                    return;
                };
                let callee = strip_turbofish(text(f, src));
                let recv = f.child_by_field_name("value").map(|o| text(o, src));
                let mut tmp = Cx::new(cx.file, cx.info);
                tmp.call(n, &callee, recv, call_table());
                // A function path passed as a value (`.map(Box::into_raw)`) leaks or
                // hands out a raw pointer for every element, as the call would.
                if let Some(args) = n.child_by_field_name("arguments") {
                    let mut cur = args.walk();
                    for a in args.named_children(&mut cur) {
                        // `Box::into_raw as fn(Box<u8>) -> *mut u8`: the cast's value.
                        let a = match a.kind() {
                            "type_cast_expression" => a.child_by_field_name("value").unwrap_or(a),
                            _ => a,
                        };
                        if !matches!(a.kind(), "scoped_identifier" | "generic_function") {
                            continue;
                        }
                        let path = strip_turbofish(text(a, src));
                        if leaking_path().is_match(&path) {
                            tmp.call(a, &path, None, call_table());
                        }
                    }
                }
                found.extend(tmp.out);
            }
            _ => {}
        }
    });
    cx.out.extend(dedupe(found));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceFile;

    fn run(src: &str) -> Vec<Raw> {
        let f = SourceFile {
            rel: "src/a.rs".into(),
            text: src.into(),
            lang: Lang::Rust,
            is_test: false,
        };
        let mut info = ProjectInfo::default();
        collect(src, &mut info);
        info.finish();
        let tree = parse(Lang::Rust, src).unwrap();
        let mut cx = Cx::new(&f, &info);
        holders(&mut cx, tree.root_node());
        cx.out
    }

    #[test]
    fn cycle_pair_versus_weak() {
        let strong = "struct S {\n    parent: RefCell<Option<Rc<S>>>,\n    children: RefCell<Vec<Rc<S>>>,\n    title: String,\n}\n";
        let r = run(strong);
        assert_eq!(
            r.iter().map(|h| h.holder.as_str()).collect::<Vec<_>>(),
            ["cycle-only", "cycle-only"]
        );
        let weak =
            "struct S {\n    parent: RefCell<Weak<S>>,\n    children: RefCell<Vec<Rc<S>>>,\n}\n";
        let r = run(weak);
        assert_eq!(
            r.iter().map(|h| h.holder.as_str()).collect::<Vec<_>>(),
            ["container-field"]
        );
    }

    #[test]
    fn statics_spawns_leaks_and_test_modules_skipped() {
        let src = "static CACHE: Mutex<Vec<u8>> = Mutex::new(Vec::new());\nstatic N: AtomicUsize = AtomicUsize::new(0);\nthread_local! {\n    static T: RefCell<u8> = RefCell::new(0);\n}\nfn f() {\n    let h = thread::spawn(|| {});\n    let b = Box::leak(Box::new(1));\n    let (tx, rx) = mpsc::channel();\n}\n#[cfg(test)]\nmod tests {\n    static X: Vec<u8> = Vec::new();\n}\n";
        let r = run(src);
        let got: Vec<(u32, &str)> = r.iter().map(|h| (h.line, h.holder.as_str())).collect();
        assert_eq!(
            got,
            [
                (1, "static/global"),
                (4, "thread-local"),
                (7, "blocked-stack"),
                (8, "static/global"),
                (9, "container-field")
            ]
        );
        assert_eq!(r[2].name.as_deref(), Some("h"));
    }

    #[test]
    fn box_into_raw_is_a_native_handle_and_leak_forget_manually_drop_stay_static() {
        let src = "fn f() {\n    let a = Box::into_raw(Box::new(1));\n    let b = Box::leak(Box::new(2));\n    let c = std::mem::forget(3);\n    let d = ManuallyDrop::new(4);\n    let e = alloc::boxed::Box::into_raw(Box::new(5));\n}\n";
        let got: Vec<(u32, &str, &str)> = run(src)
            .iter()
            .map(|h| (h.line, h.holder.as_str(), h.region.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (2, "native-handle/global-ref", "off-heap"),
                (3, "static/global", "heap"),
                (4, "static/global", "heap"),
                (5, "static/global", "heap"),
                (6, "native-handle/global-ref", "off-heap"),
            ]
        );
    }

    #[test]
    fn turbofish_calls_and_function_paths_are_the_same_holders() {
        let src = "fn f(v: Vec<Box<u8>>) {\n    let a = Box::<u32>::into_raw(Box::new(1));\n    let b = Box::<Vec<u8>>::into_non_null(Box::new(vec![]));\n    let c = Box::<[u8; 4]>::leak(Box::new([0; 4]));\n    let d: Vec<*mut u8> = v.into_iter().map(Box::into_raw).collect();\n    let e: Vec<&mut u8> = Vec::<Box<u8>>::new().into_iter().map(Box::leak).collect();\n    let g = v.iter().map(Box::<u8>::into_raw).count();\n    let h = Vec::<u8>::new();\n}\n";
        let got: Vec<(u32, &str)> = run(src)
            .iter()
            .map(|h| (h.line, h.holder.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (2, "native-handle/global-ref"),
                (3, "native-handle/global-ref"),
                (4, "static/global"),
                (5, "native-handle/global-ref"),
                (6, "static/global"),
                (7, "native-handle/global-ref"),
            ]
        );
    }

    #[test]
    fn a_fn_type_arrow_and_a_cast_do_not_hide_the_holder() {
        assert_eq!(
            strip_turbofish("Box::<fn() -> u8>::into_raw"),
            "Box::into_raw"
        );
        let src = "fn f(v: Vec<Box<u8>>) {\n    let a = Box::<fn() -> u8>::into_raw(Box::new(|| 1));\n    let b: Vec<*mut u8> = v.into_iter().map(Box::into_raw as fn(Box<u8>) -> *mut u8).collect();\n}\n";
        let got: Vec<(u32, &str)> = run(src)
            .iter()
            .map(|h| (h.line, h.holder.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (2, "native-handle/global-ref"),
                (3, "native-handle/global-ref")
            ]
        );
    }

    #[test]
    fn strip_turbofish_removes_nested_generics_only() {
        assert_eq!(strip_turbofish("Box::<Vec<u8>>::into_raw"), "Box::into_raw");
        assert_eq!(strip_turbofish("a::b::<T>::c::<U>"), "a::b::c");
        assert_eq!(strip_turbofish("Box::into_raw"), "Box::into_raw");
        assert_eq!(strip_turbofish("x::<T"), "x::<T");
    }

    #[test]
    fn container_fields_need_a_long_lived_owner() {
        let transient = "struct Parsed {\n    hits: Vec<u8>,\n}\nfn parse() -> Parsed {\n    Parsed { hits: Vec::new() }\n}\n";
        assert!(run(transient).is_empty());
        let role = "struct SessionCache {\n    rows: Vec<u8>,\n}\n";
        assert_eq!(run(role).len(), 1);
        let built = "struct Collector {\n    rows: Vec<u8>,\n}\nfn main() {\n    let c = Collector { rows: Vec::new() };\n}\n";
        assert_eq!(run(built).len(), 1);
        let held = "static APP: Mutex<App> = Mutex::new(App::new());\nstruct App {\n    jobs: Vec<Job>,\n}\nstruct Job {\n    rules: Vec<u8>,\n}\n";
        let owners: Vec<String> = run(held).iter().map(|h| h.owner.clone()).collect();
        assert_eq!(owners, ["src.a.APP", "src.a.App.jobs", "src.a.Job.rules"]);
    }
}
