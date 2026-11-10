//! Java holders. Shared field classification is also used by the class-file reader.

use std::collections::HashMap;

use dla_core::finding::{Holder, Region};

use super::{CallRule, Cx, Table, class_path, dedupe, describe, join_owner, link_name, rules};
use crate::{
    ast::{self, ancestors, child_of, children_of, first_word, text, visit},
    model::{ProjectInfo, Raw, Scope},
    parse::{Lang, parse, tree_sitter::Node},
    vocab,
};

/// Type names declared with a field of their own type.
pub fn selfref(src: &str) -> Vec<String> {
    let Ok(tree) = parse(Lang::Java, src) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    visit(tree.root_node(), &mut |n| {
        if !matches!(n.kind(), "class_declaration" | "record_declaration") {
            return;
        }
        let (Some(name), Some(body)) =
            (n.child_by_field_name("name"), n.child_by_field_name("body"))
        else {
            return;
        };
        let name = text(name, src);
        for f in children_of(body, "field_declaration") {
            if let Some(t) = f.child_by_field_name("type")
                && first_word(text(t, src)) == Some(name)
            {
                out.push(name.to_owned());
            }
        }
    });
    out
}

/// What a JVM field is, from its declared type and name. Shared by the source and
/// class-file paths.
pub struct FieldSpec<'a> {
    /// Declared type, as source text or a decoded descriptor.
    pub ty: &'a str,
    /// Field name.
    pub name: &'a str,
    /// Static field.
    pub is_static: bool,
    /// The enclosing class.
    pub class: &'a str,
    /// Initializer text when known.
    pub init: &'a str,
}

/// Classifies a field into a kind, holder and region, or `None` when it cannot hold
/// a growing structure.
pub fn classify_field(
    f: &FieldSpec<'_>,
    info: &ProjectInfo,
) -> Option<(&'static str, Holder, Region)> {
    let ty = f.ty.trim();
    if vocab::jvm_thread_local().is_match(ty) {
        return Some(("thread-local", Holder::ThreadLocal, Region::Heap));
    }
    if vocab::jvm_scalar().is_match(ty) || is_primitive(ty) {
        return None;
    }
    if vocab::jvm_lockish_name().is_match(f.name) && !vocab::jvm_container().is_match(ty) {
        return None;
    }
    let container = vocab::jvm_container().is_match(ty);
    let registration =
        vocab::registration_name().is_match(f.name) || vocab::registration_name().is_match(ty);
    let base = first_word(ty).unwrap_or("");
    let selfref = info.selfref.contains(base);
    if vocab::jvm_threadish().is_match(ty) && !container {
        return Some(("spawned-unit-field", Holder::BlockedStack, Region::NonHeap));
    }
    if vocab::jvm_functional().is_match(ty) && !container {
        return Some((
            if f.is_static {
                "static-closure"
            } else {
                "closure-field"
            },
            Holder::ClosureContinuation,
            Region::Heap,
        ));
    }
    if registration && (container || f.is_static || vocab::jvm_functional().is_match(ty)) {
        return Some((
            if f.is_static {
                "static-registry"
            } else {
                "registry-field"
            },
            Holder::RegistrationTable,
            Region::Heap,
        ));
    }
    if f.is_static {
        return Some((
            if container {
                "static-container"
            } else {
                "static-field"
            },
            Holder::StaticGlobal,
            Region::Heap,
        ));
    }
    if container || (selfref && link_name(f.name)) || (selfref && base == f.class) {
        return Some(("container-field", Holder::ContainerField, Region::Heap));
    }
    None
}

fn is_primitive(t: &str) -> bool {
    matches!(
        t,
        "byte" | "short" | "int" | "long" | "float" | "double" | "boolean" | "char" | "void"
    )
}

/// The call-site rules (shared with the class-file path).
pub fn call_table() -> &'static [CallRule] {
    static T: Table = Table::new();
    T.get_or_init(|| {
        use Holder::*;
        use Region::*;
        rules(&[
            (r"^new\s+(?:java\.lang\.)?Thread\b", "thread", BlockedStack, NonHeap, true),
            (r"^new\s+(?:java\.util\.)?Timer\b", "timer", BlockedStack, NonHeap, true),
            (r"^new\s+(?:\w+\.)*(?:ThreadPoolExecutor|ScheduledThreadPoolExecutor|ForkJoinPool)\b", "executor", BlockedStack, NonHeap, true),
            (r"(?:^|\.)Executors\.\w+$", "executor", BlockedStack, NonHeap, true),
            (r"(?:^|\.)(?:schedule|scheduleAtFixedRate|scheduleWithFixedDelay)$", "scheduled-task", BlockedStack, NonHeap, true),
            (r"(?:^|\.)CompletableFuture\.(?:supplyAsync|runAsync)$", "async-task", BlockedStack, NonHeap, true),
            (r"(?:^|\.)(?:startVirtualThread|ofVirtual|ofPlatform)\b", "thread", BlockedStack, NonHeap, true),
            (r"\.addShutdownHook$", "shutdown-hook", RegistrationTable, Heap, false),
            (r"\.(?:add|register|set|put|attach|install)\w*(?:Listener|Handler|Callback|Observer|Hook|Subscriber|Watcher|Interceptor)s?$", "registration", RegistrationTable, Heap, false),
            (r"\.(?:subscribe|addObserver|addPropertyChangeListener)$", "registration", RegistrationTable, Heap, false),
            (r"^new\s+(?:\w+\.)*(?:FileInputStream|FileOutputStream|FileReader|FileWriter|RandomAccessFile|Socket|ServerSocket|DatagramSocket|ZipFile|JarFile|ProcessBuilder)\b", "handle", NativeHandleGlobalRef, OffHeap, true),
            (r"(?:^|\.)Files\.(?:newInputStream|newOutputStream|newBufferedReader|newBufferedWriter|newByteChannel|walk|list|lines|newDirectoryStream)$", "handle", NativeHandleGlobalRef, OffHeap, true),
            (r"\.(?:getConnection|createStatement|prepareStatement|openStream|openConnection|openChannel)$", "handle", NativeHandleGlobalRef, OffHeap, true),
            (r"\.(?:allocateDirect|allocateMemory|newGlobalRef)$", "native-memory", NativeHandleGlobalRef, OffHeap, true),
            (r"(?:^|\.)System\.(?:loadLibrary|load)$", "native-library", NativeHandleGlobalRef, OffHeap, false),
            (r"\.exec$", "process", BlockedStack, NonHeap, true),
            (r"^new\s+(?:\w+\.)*(?:InheritableThreadLocal|ThreadLocal)\b|(?:^|\.)ThreadLocal\.withInitial$", "thread-local", ThreadLocal, Heap, true),
        ])
    })
}

fn is_singleton(class: Node<'_>, src: &str) -> bool {
    if class.kind() == "enum_declaration" {
        return true;
    }
    let name = class
        .child_by_field_name("name")
        .map(|n| text(n, src))
        .unwrap_or("");
    if let Some(m) = child_of(class, "modifiers")
        && [
            "Singleton",
            "Component",
            "Service",
            "Repository",
            "Bean",
            "Controller",
            "ApplicationScoped",
            "Configuration",
        ]
        .iter()
        .any(|a| text(m, src).contains(&format!("@{a}")))
    {
        return true;
    }
    let Some(body) = class.child_by_field_name("body") else {
        return false;
    };
    let mut c = body.walk();
    body.named_children(&mut c).any(|m| match m.kind() {
        "field_declaration" => {
            text(m, src).contains("static")
                && m.child_by_field_name("type")
                    .is_some_and(|t| first_word(text(t, src)) == Some(name))
        }
        "method_declaration" => {
            m.child_by_field_name("name").is_some_and(|n| {
                matches!(
                    text(n, src),
                    "getInstance" | "instance" | "getDefault" | "shared"
                )
            }) && text(m, src).contains("static")
        }
        _ => false,
    })
}

/// Enumerates the holders of one Java file.
pub fn holders(cx: &mut Cx<'_>, root: Node<'_>) {
    let src = cx.src();
    let mut singleton: HashMap<usize, bool> = HashMap::new();
    let mut found: Vec<Raw> = Vec::new();
    let prefix = cx.owner_prefix();
    visit(root, &mut |n| match n.kind() {
        "field_declaration" => {
            let Some(class) = ancestors(n).find(|a| {
                matches!(
                    a.kind(),
                    "class_declaration"
                        | "enum_declaration"
                        | "record_declaration"
                        | "interface_declaration"
                )
            }) else {
                return;
            };
            let interface = class.kind() == "interface_declaration";
            let mods = child_of(n, "modifiers").map(|m| text(m, src)).unwrap_or("");
            let is_static = mods.split_whitespace().any(|w| w == "static") || interface;
            let ty = n
                .child_by_field_name("type")
                .map(|t| text(t, src))
                .unwrap_or("");
            let class_name = class
                .child_by_field_name("name")
                .map(|c| text(c, src))
                .unwrap_or("");
            let private = mods.split_whitespace().any(|w| w == "private");
            let single = *singleton
                .entry(class.id())
                .or_insert_with(|| is_singleton(class, src));
            for d in children_of(n, "variable_declarator") {
                let name = d
                    .child_by_field_name("name")
                    .map(|c| text(c, src))
                    .unwrap_or("");
                let init = d
                    .child_by_field_name("value")
                    .map(|c| text(c, src))
                    .unwrap_or("");
                let spec = FieldSpec {
                    ty,
                    name,
                    is_static,
                    class: class_name,
                    init,
                };
                let mut classified = classify_field(&spec, cx.info);
                if classified.is_none()
                    && !is_static
                    && init.starts_with("new ")
                    && vocab::jvm_container().is_match(init)
                {
                    classified = classify_field(
                        &FieldSpec {
                            ty: init.trim_start_matches("new "),
                            ..spec
                        },
                        cx.info,
                    );
                }
                let Some((kind, holder, region)) = classified else {
                    continue;
                };
                let kind = if single && !is_static && holder == Holder::ContainerField {
                    "singleton-field"
                } else {
                    kind
                };
                let owner = join_owner(
                    &prefix,
                    &format!("{}.{name}", class_path(n, src, Lang::Java)),
                );
                let scope = if private && !is_static {
                    Scope::File
                } else {
                    Scope::Project
                };
                found.push(
                    Raw::new(kind, holder, region, &cx.file.rel, ast::line(d), owner).tracking(
                        name,
                        scope,
                        !is_static && !private,
                    ),
                );
            }
        }
        "record_declaration" => {
            let Some(params) = n.child_by_field_name("parameters") else {
                return;
            };
            let class_name = n
                .child_by_field_name("name")
                .map(|c| text(c, src))
                .unwrap_or("");
            for p in children_of(params, "formal_parameter") {
                let ty = p
                    .child_by_field_name("type")
                    .map(|t| text(t, src))
                    .unwrap_or("");
                let name = p
                    .child_by_field_name("name")
                    .map(|t| text(t, src))
                    .unwrap_or("");
                let spec = FieldSpec {
                    ty,
                    name,
                    is_static: false,
                    class: class_name,
                    init: "",
                };
                let Some((kind, holder, region)) = classify_field(&spec, cx.info) else {
                    continue;
                };
                let outer = class_path(n, src, Lang::Java);
                let owner = join_owner(
                    &prefix,
                    &join_owner(&outer, &format!("{class_name}.{name}")),
                );
                found.push(
                    Raw::new(kind, holder, region, &cx.file.rel, ast::line(p), owner).tracking(
                        name,
                        Scope::File,
                        true,
                    ),
                );
            }
        }
        "method_invocation" => {
            let name = n
                .child_by_field_name("name")
                .map(|c| text(c, src))
                .unwrap_or("");
            let obj = n.child_by_field_name("object").map(|c| text(c, src));
            let callee = match obj {
                Some(o) => format!("{o}.{name}"),
                None => name.to_owned(),
            };
            if in_resource_spec(n) {
                return;
            }
            let in_clinit = ancestors(n).any(|a| a.kind() == "static_initializer");
            let before = found.len();
            let mut tmp = Cx::new(cx.file, cx.info);
            tmp.call(n, &callee, obj, call_table());
            found.extend(tmp.out);
            if found.len() == before
                && in_clinit
                && let Some(o) = obj
                && is_register_verb(name)
            {
                let (path, _) = describe(n, src, Lang::Java);
                found.push(
                    Raw::new(
                        "static-init-registration",
                        Holder::RegistrationTable,
                        Region::Heap,
                        &cx.file.rel,
                        ast::line(n),
                        join_owner(&prefix, &path),
                    )
                    .tracking(
                        ast::last_word(o).unwrap_or(o),
                        Scope::Project,
                        false,
                    ),
                );
            }
        }
        "object_creation_expression" => {
            if in_resource_spec(n) {
                return;
            }
            let ty = n
                .child_by_field_name("type")
                .map(|c| text(c, src))
                .unwrap_or("");
            let callee = format!("new {ty}");
            let mut tmp = Cx::new(cx.file, cx.info);
            tmp.call(n, &callee, None, call_table());
            found.extend(tmp.out);
        }
        _ => {}
    });
    cx.out.extend(dedupe(found));
}

fn is_register_verb(name: &str) -> bool {
    name.starts_with("add")
        || name.starts_with("put")
        || name.starts_with("register")
        || name == "set"
}

fn in_resource_spec(n: Node<'_>) -> bool {
    ancestors(n).any(|a| a.kind() == "resource_specification")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::SourceFile, parse::parse};

    fn run(src: &str) -> Vec<Raw> {
        let f = SourceFile {
            rel: "p/A.java".into(),
            text: src.into(),
            lang: Lang::Java,
            is_test: false,
        };
        let mut info = ProjectInfo::default();
        info.selfref.extend(selfref(src));
        let tree = parse(Lang::Java, src).unwrap();
        let mut cx = Cx::new(&f, &info);
        holders(&mut cx, tree.root_node());
        cx.out
    }

    #[test]
    fn statics_fields_and_registrations() {
        let rec = run("record R(String id, java.util.List<Line> lines) {}\n");
        assert_eq!(rec.len(), 1);
        assert_eq!((rec[0].line, rec[0].owner.as_str()), (1, "R.lines"));
        let r = run(
            "package p;\nclass A {\n  static final Map<String,Object> CACHE = new HashMap<>();\n  static int n;\n  static final String S = \"x\";\n  private final List<Foo> items = new ArrayList<>();\n  static final List<Listener> LISTENERS = new ArrayList<>();\n  private final Node head;\n  static final ThreadLocal<X> TL = new ThreadLocal<>();\n}\nclass Node { Node next; }\n",
        );
        let got: Vec<(u32, &str)> = r.iter().map(|h| (h.line, h.holder.as_str())).collect();
        assert_eq!(
            got,
            [
                (3, "static/global"),
                (6, "container-field"),
                (7, "registration-table"),
                (8, "container-field"),
                (9, "thread-local"),
                (11, "container-field"),
            ]
        );
        assert_eq!(r[0].owner, "p.A.CACHE");
    }

    #[test]
    fn spawns_handles_and_clinit() {
        let r = run(
            "class A {\n  static { REG.put(1, 2); }\n  void f() {\n    Thread t = new Thread(r);\n    ExecutorService e = Executors.newFixedThreadPool(2);\n    bus.addListener(l);\n    try (var in = new FileInputStream(p)) { }\n    var s = new Socket(h, 1);\n  }\n}\n",
        );
        let kinds: Vec<(&str, u32)> = r.iter().map(|h| (h.kind.as_str(), h.line)).collect();
        assert_eq!(
            kinds,
            [
                ("static-init-registration", 2),
                ("thread", 4),
                ("executor", 5),
                ("registration", 6),
                ("handle", 8)
            ]
        );
        assert_eq!(r[1].name.as_deref(), Some("t"));
        assert_eq!(r[1].owner, "A.f");
    }

    #[test]
    fn a_holder_created_in_an_initializer_is_owned_by_its_field() {
        let r = run(
            "package p;\nclass A {\n  static final ThreadLocal<List<String>> DECL = ThreadLocal.withInitial(ArrayList::new);\n  static final ThreadLocal<List<String>> BLOCK;\n  static {\n    BLOCK = ThreadLocal.withInitial(ArrayList::new);\n  }\n  private final ThreadLocal<X> held;\n  A() {\n    this.held = new ThreadLocal<>();\n  }\n  void f() {\n    ThreadLocal<X> local = new ThreadLocal<>();\n  }\n}\n",
        );
        let at = |line: u32| -> Vec<&str> {
            r.iter()
                .filter(|h| h.line == line)
                .map(|h| h.owner.as_str())
                .collect()
        };
        assert_eq!(at(3), ["p.A.DECL"]);
        assert_eq!(
            at(6),
            ["p.A.BLOCK"],
            "static block: the field, not <clinit>"
        );
        assert_eq!(
            at(10),
            ["p.A.held"],
            "constructor: the field, not the constructor"
        );
        assert_eq!(at(13), ["p.A.f"], "a local variable keeps its method");
        assert!(r.iter().all(|h| !h.owner.contains("<clinit>")), "{r:?}");
        let block = r.iter().find(|h| h.line == 6).unwrap();
        assert_eq!(block.name.as_deref(), Some("BLOCK"));
    }
}
