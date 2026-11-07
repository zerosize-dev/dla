//! C and C++ holders.

use std::sync::OnceLock;

use dla_core::finding::{Holder, Region};
use regex::Regex;

use super::{CallRule, Cx, Table, dedupe, describe, join_owner, rules};
use crate::{
    ast::{ancestors, line, text, visit},
    model::{Raw, Scope},
    parse::{Lang, tree_sitter::Node},
    vocab,
};

fn call_table() -> &'static [CallRule] {
    static T: Table = Table::new();
    T.get_or_init(|| {
        use Holder::*;
        use Region::*;
        rules(&[
            (r"^(?:malloc|calloc|realloc|reallocarray|strdup|strndup|aligned_alloc|posix_memalign|mmap|fopen|fdopen|popen|opendir|open|socket|accept|dlopen|CreateFile\w*|sqlite3_open\w*|curl_easy_init|g_malloc\w*|av_malloc\w*|OPENSSL_malloc)$", "allocation", NativeHandleGlobalRef, OffHeap, true),
            (r"^pthread_create$|^thrd_create$|^CreateThread$|^fork$|^std::(?:thread|jthread|async)$", "thread", BlockedStack, NonHeap, true),
            (r"^(?:signal|atexit|on_exit|pthread_atfork|sigaction)$|^register_\w+$|^\w+_register\w*$|^\w+_set_(?:callback|handler|hook)\w*$|^\w+_(?:subscribe|add_listener|add_handler)\w*$", "registration", RegistrationTable, Heap, false),
        ])
    })
}

fn acquire_suffix() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^\w+_(?:open|create|new|alloc|acquire|connect|init_new|spawn|start)$")
            .expect("static")
    })
}

fn std_container() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"std::(?:vector|map|unordered_\w+|list|deque|set|queue|stack|multimap|multiset)\b",
        )
        .expect("static")
    })
}

fn callback_field() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)^(?:on_\w+|\w*_cb|cb_?\w*|\w*callback\w*|\w*handler\w*|\w*hook\w*)$")
            .expect("static")
    })
}

const PRIMITIVES: &[&str] = &[
    "int",
    "unsigned",
    "long",
    "short",
    "char",
    "float",
    "double",
    "size_t",
    "ssize_t",
    "bool",
    "_Bool",
    "uint8_t",
    "uint16_t",
    "uint32_t",
    "uint64_t",
    "int8_t",
    "int16_t",
    "int32_t",
    "int64_t",
    "ptrdiff_t",
    "time_t",
    "off_t",
    "uintptr_t",
    "intptr_t",
    "sig_atomic_t",
    "signed",
    "wchar_t",
    "atomic_int",
];

fn declarator_name<'t>(d: Node<'t>, src: &'t str) -> Option<&'t str> {
    let mut cur = d;
    loop {
        match cur.kind() {
            "identifier" | "field_identifier" | "type_identifier" => return Some(text(cur, src)),
            _ => {
                cur = cur
                    .child_by_field_name("declarator")
                    .or_else(|| cur.named_child(0))?
            }
        }
    }
}

fn pointer_depth(d: Node<'_>) -> usize {
    let mut depth = 0;
    let mut cur = Some(d);
    while let Some(c) = cur {
        if c.kind() == "pointer_declarator" {
            depth += 1;
        }
        cur = c.child_by_field_name("declarator");
    }
    depth
}

/// A function prototype or definition declarator (`T *f(args)`), as opposed to a
/// function-pointer variable (`T (*f)(args)`).
fn is_prototype(d: Node<'_>) -> bool {
    let mut cur = Some(d);
    while let Some(c) = cur {
        if c.kind() == "function_declarator" {
            return c.child_by_field_name("declarator").is_some_and(|i| {
                matches!(
                    i.kind(),
                    "identifier" | "field_identifier" | "qualified_identifier"
                )
            });
        }
        cur = c.child_by_field_name("declarator");
    }
    false
}

fn is_function_pointer(d: Node<'_>) -> bool {
    let mut found = false;
    visit(d, &mut |n| {
        found |= n.kind() == "function_declarator";
    });
    found
}

fn type_base<'a>(n: Node<'_>, src: &'a str) -> &'a str {
    n.child_by_field_name("type")
        .map(|t| text(t, src))
        .unwrap_or("")
}

fn is_primitive(ty: &str) -> bool {
    ty.split_whitespace()
        .all(|w| PRIMITIVES.contains(&w) || w == "const" || w == "volatile" || w == "static")
}

/// Enumerates the holders of one C or C++ file.
pub fn holders(cx: &mut Cx<'_>, root: Node<'_>) {
    let src = cx.src();
    let prefix = cx.owner_prefix();
    let mut found: Vec<Raw> = Vec::new();
    visit(root, &mut |n| match n.kind() {
        "declaration" => {
            let top = n.parent().is_some_and(|p| {
                matches!(
                    p.kind(),
                    "translation_unit"
                        | "preproc_ifdef"
                        | "preproc_if"
                        | "linkage_specification"
                        | "declaration_list"
                        | "namespace_definition"
                )
            }) && !ancestors(n).any(|a| a.kind() == "function_definition");
            if !top {
                return;
            }
            let full = text(n, src);
            let ty = type_base(n, src);
            if full.trim_start().starts_with("extern") && !full.contains('=') {
                return;
            }
            let is_const =
                full.split_whitespace().take(3).any(|w| w == "const") && !full.contains('*');
            let mut c = n.walk();
            for d in n.children_by_field_name("declarator", &mut c) {
                if is_prototype(d) {
                    continue;
                }
                let Some(name) = declarator_name(d, src) else {
                    continue;
                };
                let ptr = pointer_depth(d) > 0 || d.kind() == "array_declarator" || is_array(d);
                if is_const
                    || (!ptr && is_primitive(ty))
                    || ty.starts_with("char") && ptr && full.contains('"')
                {
                    continue;
                }
                let holder = if vocab::registration_name().is_match(name) {
                    Holder::RegistrationTable
                } else if is_function_pointer(d) {
                    Holder::ClosureContinuation
                } else {
                    Holder::StaticGlobal
                };
                found.push(
                    Raw::new(
                        "file-scope",
                        holder,
                        Region::OffHeap,
                        &cx.file.rel,
                        line(d),
                        join_owner(&prefix, name),
                    )
                    .tracking(name, Scope::Project, false),
                );
            }
        }
        "field_declaration" => {
            let Some(st) = ancestors(n).find(|a| {
                matches!(
                    a.kind(),
                    "struct_specifier" | "union_specifier" | "class_specifier"
                )
            }) else {
                return;
            };
            let sname = st
                .child_by_field_name("name")
                .map(|c| text(c, src).to_owned())
                .or_else(|| {
                    st.parent()
                        .filter(|p| p.kind() == "type_definition")
                        .and_then(|p| p.child_by_field_name("declarator"))
                        .and_then(|d| declarator_name(d, src).map(str::to_owned))
                })
                .unwrap_or_default();
            let ty = type_base(n, src);
            let mut c = n.walk();
            for d in n.children_by_field_name("declarator", &mut c) {
                let Some(name) = declarator_name(d, src) else {
                    continue;
                };
                let depth = pointer_depth(d);
                let is_std = text(n, src).contains("std::");
                let (kind, holder) = if is_function_pointer(d) {
                    if !callback_field().is_match(name)
                        && !name.contains("fn")
                        && !name.contains("func")
                    {
                        continue;
                    }
                    ("callback-field", Holder::ClosureContinuation)
                } else if is_std && std_container().is_match(text(n, src)) {
                    ("container-field", Holder::ContainerField)
                } else if depth == 0
                    || (depth == 1
                        && matches!(
                            ty.trim_start_matches("const ").trim(),
                            "void" | "char" | "wchar_t"
                        ))
                {
                    continue;
                } else {
                    ("pointer-field", Holder::ContainerField)
                };
                found.push(
                    Raw::new(
                        kind,
                        holder,
                        Region::OffHeap,
                        &cx.file.rel,
                        line(n),
                        join_owner(&prefix, &format!("{sname}.{name}")),
                    )
                    .tracking(name, Scope::Project, true),
                );
            }
        }
        "assignment_expression" => {
            let (Some(l), Some(r)) = (
                n.child_by_field_name("left"),
                n.child_by_field_name("right"),
            ) else {
                return;
            };
            if l.kind() == "subscript_expression" {
                // A caller's pointer stored into an element of a container field.
                let arg = l.child_by_field_name("argument");
                if r.kind() == "identifier"
                    && arg.is_some_and(|a| a.kind() == "field_expression")
                    && is_parameter(n, text(r, src), src)
                {
                    let (path, _) = describe(n, src, cx.file.lang);
                    let field = arg
                        .and_then(|a| a.child_by_field_name("field"))
                        .map(|f| text(f, src))
                        .unwrap_or("");
                    found.push(
                        Raw::new(
                            "retained-pointer-store",
                            Holder::ContainerField,
                            Region::OffHeap,
                            &cx.file.rel,
                            line(n),
                            join_owner(&prefix, &format!("{path}.{field}")),
                        )
                        .tracking(field, Scope::Project, true),
                    );
                }
                return;
            }
            if l.kind() != "field_expression" {
                return;
            }
            let field = l
                .child_by_field_name("field")
                .map(|c| text(c, src))
                .unwrap_or("");
            if callback_field().is_match(field)
                && matches!(
                    r.kind(),
                    "identifier"
                        | "pointer_expression"
                        | "qualified_identifier"
                        | "lambda_expression"
                )
            {
                let (path, _) = describe(n, src, cx.file.lang);
                found.push(
                    Raw::new(
                        "callback-assignment",
                        Holder::ClosureContinuation,
                        Region::Heap,
                        &cx.file.rel,
                        line(n),
                        join_owner(&prefix, &format!("{path}.{field}")),
                    )
                    .tracking(field, Scope::Project, true),
                );
            }
        }
        "call_expression" => {
            let Some(f) = n.child_by_field_name("function") else {
                return;
            };
            let callee = text(f, src);
            let mut tmp = Cx::new(cx.file, cx.info);
            tmp.call(n, callee, None, call_table());
            if tmp.out.is_empty()
                && acquire_suffix().is_match(callee)
                && super::assigned_name(n, src).is_some()
            {
                let rule = &super::CallRule {
                    re: Regex::new("^$").expect("static"),
                    kind: "acquire",
                    holder: Holder::NativeHandleGlobalRef,
                    region: Region::OffHeap,
                    assigned: true,
                };
                let mut t2 = Cx::new(cx.file, cx.info);
                t2.call(n, "", None, std::slice::from_ref(rule));
                tmp.out = t2.out;
            }
            found.extend(tmp.out);
        }
        "new_expression" if cx.file.lang == Lang::Cpp => {
            let mut tmp = Cx::new(cx.file, cx.info);
            let rule = super::CallRule {
                re: Regex::new("^$").expect("static"),
                kind: "new",
                holder: Holder::NativeHandleGlobalRef,
                region: Region::OffHeap,
                assigned: true,
            };
            tmp.call(n, "", None, std::slice::from_ref(&rule));
            found.extend(tmp.out);
        }
        _ => {}
    });
    cx.out.extend(dedupe(found));
}

/// True when `name` is a parameter of the function enclosing `n`.
fn is_parameter(n: Node<'_>, name: &str, src: &str) -> bool {
    let Some(f) = ancestors(n).find(|a| a.kind() == "function_definition") else {
        return false;
    };
    let Some(d) = f.child_by_field_name("declarator") else {
        return false;
    };
    let t = text(d, src);
    let params = t.split_once('(').map_or("", |(_, p)| p);
    Regex::new(&format!(r"\b{}\b", regex::escape(name))).is_ok_and(|r| r.is_match(params))
}

fn is_array(d: Node<'_>) -> bool {
    let mut cur = Some(d);
    while let Some(c) = cur {
        if c.kind() == "array_declarator" {
            return true;
        }
        cur = c.child_by_field_name("declarator");
    }
    false
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
            rel: "src/a.c".into(),
            text: src.into(),
            lang: Lang::C,
            is_test: false,
        };
        let info = ProjectInfo::default();
        let tree = parse(Lang::C, src).unwrap();
        let mut cx = Cx::new(&f, &info);
        holders(&mut cx, tree.root_node());
        cx.out
    }

    #[test]
    fn prototypes_are_not_variables_and_stored_parameters_are_holders() {
        let r = run(
            "ring *ring_create(size_t n);\nvoid (*hook)(int);\nvoid put(ring *r, const void *item)\n{\n    r->items[r->next] = item;\n    int local = 0;\n    r->slots[0] = local;\n}\n",
        );
        let got: Vec<(u32, &str)> = r.iter().map(|h| (h.line, h.kind.as_str())).collect();
        assert_eq!(got, [(2, "file-scope"), (5, "retained-pointer-store")]);
    }

    #[test]
    fn globals_fields_allocations_callbacks() {
        let src = "static int counter;\nstatic entry *table[64];\nstatic const char *NAME = \"x\";\nstruct cache {\n    entry **buckets;\n    entry *head;\n    char *label;\n    void *user;\n    size_t count;\n    void (*on_evict)(int);\n};\nvoid f(struct cache *c) {\n    entry *e = calloc(1, sizeof *e);\n    c->buckets = calloc(4, 8);\n    opts.on_frame = handler;\n    FILE *fp = fopen(\"a\", \"r\");\n    pthread_t t;\n    pthread_create(&t, 0, run, 0);\n}\n";
        let r = run(src);
        let got: Vec<(u32, &str)> = r.iter().map(|h| (h.line, h.kind.as_str())).collect();
        assert_eq!(
            got,
            [
                (2, "file-scope"),
                (5, "pointer-field"),
                (6, "pointer-field"),
                (10, "callback-field"),
                (13, "allocation"),
                (14, "allocation"),
                (15, "callback-assignment"),
                (16, "allocation"),
                (18, "thread"),
            ]
        );
        assert_eq!(r[4].name.as_deref(), Some("e"));
        assert!(r[5].member_access);
    }
}
