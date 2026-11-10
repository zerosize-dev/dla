//! Kotlin holders, from a line scanner with brace tracking. The Kotlin grammar does
//! not recover from errors reliably enough for a census, and `object` and `companion
//! object` bodies are recognisable from their headers.

use std::sync::OnceLock;

use dla_core::finding::{Holder, Region};
use regex::Regex;

use super::{
    Cx,
    java::{FieldSpec, classify_field},
    join_owner,
};
use crate::model::{Raw, Scope};

#[derive(Clone, Copy, PartialEq)]
enum Frame {
    Object,
    Class,
    Fun,
    /// An `init { .. }` block of a class or object.
    Init,
    Other,
}

struct Pats {
    header: Regex,
    companion: Regex,
    init: Regex,
    store: Regex,
    fun: Regex,
    prop: Regex,
    calls: Vec<(Regex, &'static str, Holder, Region)>,
    assign: Regex,
    factory: Regex,
}

fn pats() -> &'static Pats {
    static P: OnceLock<Pats> = OnceLock::new();
    P.get_or_init(|| {
        use Holder::*;
        use Region::*;
        let c = |p: &str, k, h, r| (Regex::new(p).expect("static"), k, h, r);
        Pats {
            header: Regex::new(r"^\s*(?:(?:private|public|internal|protected|data|sealed|open|abstract|enum|inner|annotation|value)\s+)*(object|class|interface)\s+(\w+)").expect("static"),
            companion: Regex::new(r"^\s*(?:private\s+|public\s+)?companion\s+object\b").expect("static"),
            init: Regex::new(r"^\s*init\s*\{").expect("static"),
            store: Regex::new(r"^\s*(?:init\s*\{\s*)?(?:this\.)?([A-Za-z_]\w*)\s*=[^=]").expect("static"),
            fun: Regex::new(r"^\s*(?:(?:private|public|internal|protected|override|open|suspend|inline|operator|abstract)\s+)*fun\b").expect("static"),
            prop: Regex::new(r"^\s*(?:(?:private|public|internal|protected|override|lateinit|const|open|abstract)\s+)*(val|var)\s+(\w+)\s*(?::\s*([^=]+?))?\s*(?:(?:by\s+\w+|=)\s*(.*))?$").expect("static"),
            calls: vec![
                c(r"\b(?:Thread|thread)\s*(?:\(|\{)|\bGlobalScope\.(?:launch|async)\b|\bCoroutineScope\s*\(|\bExecutors\.\w+\(|\bTimer\s*\(|\bThreadPoolExecutor\s*\(", "spawn", BlockedStack, NonHeap),
                c(r"\.addShutdownHook\b|\.(?:add|register|set|put)\w*(?:Listener|Handler|Callback|Observer|Hook|Subscriber)s?\s*\(|\.subscribe\s*\(", "registration", RegistrationTable, Heap),
                c(r"\b(?:FileInputStream|FileOutputStream|FileReader|FileWriter|RandomAccessFile|Socket|ServerSocket)\s*\(|\.(?:getConnection|openStream|openConnection|allocateDirect)\s*\(", "handle", NativeHandleGlobalRef, OffHeap),
                c(r"\bThreadLocal\s*(?:<[^>]*>)?\s*(?:\(|\.withInitial)", "thread-local", ThreadLocal, Heap),
            ],
            assign: Regex::new(r"(?:val|var)\s+(\w+)\s*(?::[^=]+)?=").expect("static"),
            factory: Regex::new(r"^(?:mutable|array|hashMap|linkedMap|hashSet|linkedSet|sortedMap|sortedSet|concurrent)\w*(?:Of|Map|Set|List)\b|^(?:Array|Hash|Linked|Tree|Concurrent|CopyOnWrite)\w*\(").expect("static"),
        }
    })
}

fn strip(l: &str) -> &str {
    let mut from = 0;
    while let Some(i) = l[from..].find("//") {
        let at = from + i;
        if at == 0 || !l[..at].ends_with(':') {
            return &l[..at];
        }
        from = at + 2;
    }
    l
}

/// Enumerates the holders of one Kotlin file.
pub fn holders(cx: &mut Cx<'_>) {
    let p = pats();
    let prefix = cx.owner_prefix();
    let mut stack: Vec<(Frame, i32, String)> = Vec::new();
    let mut pending: Option<(Frame, String)> = None;
    // Locals declared so far in the current `init` block: an assignment to one of them is
    // not a property store.
    let mut init_locals: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut out: Vec<Raw> = Vec::new();
    for (i, raw_line) in cx.file.text.lines().enumerate() {
        let line = strip(raw_line);
        let ln = u32::try_from(i + 1).unwrap_or(u32::MAX);
        let t = line.trim();
        if t.starts_with('*') || t.starts_with("/*") || t.is_empty() {
            continue;
        }
        let top = stack.last().map_or(Frame::Other, |s| s.0);
        let in_fun = stack
            .iter()
            .any(|s| matches!(s.0, Frame::Fun | Frame::Init))
            || p.init.is_match(line);
        let in_init = top == Frame::Init || p.init.is_match(line);
        if in_init && let Some(c) = p.assign.captures(line) {
            init_locals.push(c[1].to_owned());
        }
        if p.companion.is_match(line) {
            pending = Some((Frame::Object, "Companion".into()));
        } else if let Some(c) = p.header.captures(line) {
            let f = if &c[1] == "object" {
                Frame::Object
            } else {
                Frame::Class
            };
            pending = Some((f, c[2].to_owned()));
        } else if p.init.is_match(line) {
            pending = Some((Frame::Init, String::new()));
            init_locals.clear();
        } else if p.fun.is_match(line) {
            pending = Some((Frame::Fun, String::new()));
        } else if !in_fun
            && let Some(c) = p.prop.captures(line)
            && (top != Frame::Other || stack.is_empty())
        {
            let name = &c[2];
            let ty = c.get(3).map_or("", |m| m.as_str().trim());
            let init = c.get(4).map_or("", |m| m.as_str().trim());
            let is_static = matches!(top, Frame::Object) || stack.is_empty();
            let guess = if !ty.is_empty() {
                ty.to_owned()
            } else if p.factory.is_match(init) {
                "MutableList".to_owned()
            } else if init.chars().next().is_some_and(char::is_uppercase) {
                init.split('(').next().unwrap_or(init).to_owned()
            } else {
                String::new()
            };
            if !guess.is_empty() {
                let class = stack.last().map(|s| s.2.as_str()).unwrap_or("");
                let spec = FieldSpec {
                    ty: &guess,
                    name,
                    is_static,
                    class,
                    init,
                };
                if let Some((kind, holder, region)) = classify_field(&spec, cx.info) {
                    let path: Vec<&str> = stack
                        .iter()
                        .filter(|s| s.0 != Frame::Other)
                        .map(|s| s.2.as_str())
                        .collect();
                    let mut owner_path = path.join(".");
                    if !owner_path.is_empty() {
                        owner_path.push('.');
                    }
                    owner_path.push_str(name);
                    let scope = if is_static {
                        Scope::Project
                    } else {
                        Scope::File
                    };
                    out.push(
                        Raw::new(
                            kind,
                            holder,
                            region,
                            &cx.file.rel,
                            ln,
                            join_owner(&prefix, &owner_path),
                        )
                        .tracking(name, scope, !is_static),
                    );
                }
            }
        }
        for (re, kind, holder, region) in &p.calls {
            if re.is_match(line) {
                let path: Vec<&str> = stack
                    .iter()
                    .filter(|s| !matches!(s.0, Frame::Other | Frame::Init))
                    .map(|s| s.2.as_str())
                    .collect();
                // `held = ThreadLocal<X>()` in an `init` block stores into the property
                // `held` (unless it names a local the block declared).
                let stored = in_init
                    .then(|| p.store.captures(line))
                    .flatten()
                    .map(|c| c[1].to_owned())
                    .filter(|n| !init_locals.contains(n) && !p.assign.is_match(line));
                let class_path = path.join(".");
                let owner_path = match &stored {
                    Some(n) => format!("{class_path}.{n}"),
                    None => class_path,
                };
                let mut r = Raw::new(
                    kind,
                    *holder,
                    *region,
                    &cx.file.rel,
                    ln,
                    join_owner(&prefix, &owner_path),
                );
                if let Some(n) = &stored {
                    r = r.tracking(n, Scope::Project, false);
                } else if let Some(a) = p.assign.captures(line) {
                    r = r.tracking(&a[1], Scope::File, false);
                }
                out.push(r);
                break;
            }
        }
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    let (f, n) = pending.take().unwrap_or((Frame::Other, String::new()));
                    stack.push((f, depth, n));
                }
                '}' => {
                    if stack.last().is_some_and(|s| s.1 == depth) {
                        stack.pop();
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        if !t.ends_with(['(', ',', ':']) && !line.contains('{') {
            pending = None;
        }
    }
    cx.out.extend(super::dedupe(out));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{ProjectInfo, SourceFile},
        parse::Lang,
    };

    #[test]
    fn objects_companions_classes_and_top_level() {
        let src = "package a.b\nobject Registry {\n    val items = mutableListOf<String>()\n    val n = 3\n    fun add(s: String) { val t = Thread { run() }; t.start() }\n}\nclass Svc {\n    companion object {\n        val CACHE = HashMap<String, Int>()\n    }\n    private val listeners = ArrayList<Listener>()\n    val label = \"x\"\n}\nval top = ConcurrentHashMap<String, String>()\n";
        let f = SourceFile {
            rel: "A.kt".into(),
            text: src.into(),
            lang: Lang::Kotlin,
            is_test: false,
        };
        let info = ProjectInfo::default();
        let mut cx = Cx::new(&f, &info);
        holders(&mut cx);
        let got: Vec<(u32, String, &str)> = cx
            .out
            .iter()
            .map(|h| (h.line, h.owner.clone(), h.holder.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (3, "a.b.Registry.items".into(), "static/global"),
                (5, "a.b.Registry".into(), "blocked-stack"),
                (9, "a.b.Svc.Companion.CACHE".into(), "static/global"),
                (11, "a.b.Svc.listeners".into(), "registration-table"),
                (14, "a.b.top".into(), "static/global"),
            ]
        );
    }

    fn run_kt(src: &str) -> Vec<(u32, String, String, Option<String>)> {
        let f = SourceFile {
            rel: "A.kt".into(),
            text: src.into(),
            lang: Lang::Kotlin,
            is_test: false,
        };
        let info = ProjectInfo::default();
        let mut cx = Cx::new(&f, &info);
        holders(&mut cx);
        cx.out
            .iter()
            .map(|h| {
                (
                    h.line,
                    h.owner.clone(),
                    h.holder.as_str().to_owned(),
                    h.name.clone(),
                )
            })
            .collect()
    }

    #[test]
    fn companion_object_and_init_block_stores_are_owned_by_their_property() {
        let got = run_kt(
            "package p\nclass A {\n    companion object {\n        val TL = ThreadLocal.withInitial { ArrayList<String>() }\n        val BUS = Bus()\n    }\n    private val held = ThreadLocal<X>()\n    private var late: ThreadLocal<X>? = null\n    init {\n        late = ThreadLocal<X>()\n        val tmp = ThreadLocal<X>()\n        var scratch: ThreadLocal<X>? = null\n        scratch = ThreadLocal<X>()\n    }\n}\nobject O {\n    val CB = Callbacks()\n    var slot: ThreadLocal<X>? = null\n    init { slot = ThreadLocal<X>() }\n}\n",
        );
        let at = |line: u32| -> Vec<(&str, Option<&str>)> {
            got.iter()
                .filter(|h| h.0 == line)
                .map(|h| (h.1.as_str(), h.3.as_deref()))
                .collect()
        };
        // Property initializers: the property, with the companion or object in the path.
        assert_eq!(at(4), [("p.A.Companion.TL", Some("TL"))]);
        assert_eq!(at(7), [("p.A.held", Some("held"))]);
        assert_eq!(at(17), [("p.O.CB", Some("CB"))]);
        // An init block stores into the property it names.
        assert_eq!(at(10), [("p.A.late", Some("late"))]);
        assert_eq!(at(19), [("p.O.slot", Some("slot"))]);
        // A local of the block is no property: the holder keeps the class path.
        assert_eq!(at(11), [("p.A", Some("tmp"))]);
        assert_eq!(at(13), [("p.A", None)]);
        assert!(got.iter().all(|h| !h.1.contains("<clinit>")), "{got:?}");
    }
}
