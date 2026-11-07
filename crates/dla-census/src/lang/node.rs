//! JavaScript and TypeScript holders.

use std::sync::OnceLock;

use dla_core::finding::{Holder, Region};
use regex::Regex;

use super::{CallRule, Cx, Table, class_path, dedupe, join_owner, link_name, rules};
use crate::{
    ast::{ancestors, line, text, visit},
    model::{Raw, Scope},
    parse::tree_sitter::Node,
    vocab,
};

fn call_table() -> &'static [CallRule] {
    static T: Table = Table::new();
    T.get_or_init(|| {
        use Holder::*;
        use Region::*;
        rules(&[
            (r"^(?:(?:global|globalThis|timers|window)\.)?(?:setInterval|setTimeout|setImmediate)$", "timer", BlockedStack, NonHeap, true),
            (r"^new\s+(?:\w+\.)*(?:Worker|SharedWorker)$|(?:^|\.)(?:fork|spawn|exec|execFile|spawnSync)$", "spawn", BlockedStack, NonHeap, true),
            (r"\.(?:on|once|addListener|prependListener|addEventListener|subscribe|addEventHandler)$", "registration", RegistrationTable, Heap, false),
            (r"^new\s+(?:\w+\.)*(?:WebSocket|WebSocketServer|EventSource|Socket|Server)$|^(?:net|http|https|http2|tls|dgram|fs|fsPromises|fs\.promises)\.(?:createServer|createConnection|connect|createSocket|createReadStream|createWriteStream|open|openSync|request)$", "handle", NativeHandleGlobalRef, OffHeap, true),
            (r"^new\s+(?:\w+\.)*AsyncLocalStorage$", "thread-local", ThreadLocal, Heap, true),
        ])
    })
}

fn handleish_name() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)(timer|interval|handle|conn|socket|server|instance|current|singleton|worker|client|pool)").expect("static")
    })
}

fn is_func(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration"
            | "function_expression"
            | "function"
            | "arrow_function"
            | "method_definition"
            | "generator_function_declaration"
            | "generator_function"
    )
}

fn value_class(
    val: Option<Node<'_>>,
    src: &str,
    name: &str,
    is_let: bool,
    rebound: bool,
) -> Option<Holder> {
    let registration = vocab::registration_name().is_match(name);
    let container = |h: Holder| {
        Some(if registration {
            Holder::RegistrationTable
        } else {
            h
        })
    };
    let Some(v) = val else {
        return (is_let && (rebound || link_name(name) || handleish_name().is_match(name)))
            .then_some(Holder::StaticGlobal);
    };
    match v.kind() {
        "array" => {
            let empty = v.named_child_count() == 0;
            (empty
                || !name
                    .chars()
                    .all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit()))
            .then(|| container(Holder::StaticGlobal))
            .flatten()
        }
        "object" => (v.named_child_count() == 0)
            .then(|| container(Holder::StaticGlobal))
            .flatten(),
        "new_expression" => {
            let ctor = v
                .child_by_field_name("constructor")
                .map(|c| text(c, src))
                .unwrap_or("");
            let base = ctor.rsplit('.').next().unwrap_or(ctor);
            if matches!(
                base,
                "Date"
                    | "RegExp"
                    | "Error"
                    | "URL"
                    | "Promise"
                    | "TextEncoder"
                    | "TextDecoder"
                    | "URLSearchParams"
                    | "Intl"
            ) {
                return None;
            }
            if base == "AsyncLocalStorage" {
                return Some(Holder::ThreadLocal);
            }
            if base.chars().next().is_some_and(char::is_uppercase) {
                return container(Holder::StaticGlobal);
            }
            None
        }
        "call_expression" => {
            let f = v
                .child_by_field_name("function")
                .map(|c| text(c, src))
                .unwrap_or("");
            let base = f.rsplit('.').next().unwrap_or(f);
            (f.starts_with("Object.create")
                || f == "Array.from"
                || base.starts_with("create")
                || base.starts_with("make")
                || base.starts_with("build"))
            .then(|| container(Holder::StaticGlobal))
            .flatten()
        }
        "null" | "undefined" => (is_let
            && (rebound || link_name(name) || handleish_name().is_match(name)))
        .then_some(Holder::StaticGlobal),
        _ => None,
    }
}

/// Enumerates the holders of one JavaScript or TypeScript file.
pub fn holders(cx: &mut Cx<'_>, root: Node<'_>) {
    let src = cx.src();
    let prefix = cx.owner_prefix();
    let lang = cx.file.lang;
    let mut found: Vec<Raw> = Vec::new();
    visit(root, &mut |n| match n.kind() {
        "variable_declarator" => {
            if ancestors(n).any(|a| is_func(a.kind()) || a.kind() == "class_body") {
                return;
            }
            let Some(name_node) = n.child_by_field_name("name") else {
                return;
            };
            if name_node.kind() != "identifier" {
                return;
            }
            let name = text(name_node, src);
            let is_let = n.parent().is_some_and(|p| {
                p.kind() == "variable_declaration" || text(p, src).starts_with("let")
            });
            let rebound = super::assigned_times(src, "", name, 2);
            let Some(holder) =
                value_class(n.child_by_field_name("value"), src, name, is_let, rebound)
            else {
                return;
            };
            let kind = match holder {
                Holder::RegistrationTable => "module-registry",
                Holder::ThreadLocal => "thread-local",
                _ => "module-global",
            };
            found.push(
                Raw::new(
                    kind,
                    holder,
                    Region::Heap,
                    &cx.file.rel,
                    line(n),
                    join_owner(&prefix, name),
                )
                .tracking(name, Scope::Project, false),
            );
        }
        "assignment_expression" => {
            let (Some(l), Some(r)) = (
                n.child_by_field_name("left"),
                n.child_by_field_name("right"),
            ) else {
                return;
            };
            if l.kind() != "member_expression" {
                return;
            }
            let obj = l
                .child_by_field_name("object")
                .map(|c| text(c, src))
                .unwrap_or("");
            let prop = l
                .child_by_field_name("property")
                .map(|c| text(c, src))
                .unwrap_or("");
            if obj != "this" || !in_constructor(n, src) {
                return;
            }
            let rebound = enclosing_class_text(n, src)
                .is_some_and(|t| super::assigned_times(t, "this.", prop, 2));
            push_field(
                &mut found,
                cx,
                n,
                prop,
                value_class(Some(r), src, prop, true, rebound),
                lang,
                src,
                &prefix,
            );
        }
        "field_definition" | "public_field_definition" => {
            let name = n
                .child_by_field_name("property")
                .or_else(|| n.child_by_field_name("name"))
                .map(|c| text(c, src))
                .unwrap_or("");
            let val = n.child_by_field_name("value");
            let is_static = text(n, src).trim_start().starts_with("static");
            let rebound = enclosing_class_text(n, src)
                .is_some_and(|t| super::assigned_times(t, "this.", name, 2));
            let holder = value_class(val, src, name, true, rebound);
            let holder = if is_static && holder.is_some() {
                Some(Holder::StaticGlobal)
            } else {
                holder
            };
            push_field(&mut found, cx, n, name, holder, lang, src, &prefix);
        }
        "call_expression" => {
            let Some(f) = n.child_by_field_name("function") else {
                return;
            };
            let callee = text(f, src);
            let recv = (f.kind() == "member_expression")
                .then(|| f.child_by_field_name("object").map(|o| text(o, src)))
                .flatten();
            let mut tmp = Cx::new(cx.file, cx.info);
            tmp.call(n, callee, recv, call_table());
            found.extend(tmp.out);
        }
        "new_expression" => {
            let ctor = n
                .child_by_field_name("constructor")
                .map(|c| text(c, src))
                .unwrap_or("");
            let callee = format!("new {ctor}");
            let mut tmp = Cx::new(cx.file, cx.info);
            tmp.call(n, &callee, None, call_table());
            found.extend(tmp.out);
        }
        _ => {}
    });
    cx.out.extend(dedupe(found));
}

fn enclosing_class_text<'a>(n: Node<'_>, src: &'a str) -> Option<&'a str> {
    ancestors(n)
        .find(|a| matches!(a.kind(), "class_declaration" | "class"))
        .map(|c| text(c, src))
}

fn in_constructor(n: Node<'_>, src: &str) -> bool {
    ancestors(n)
        .find(|a| a.kind() == "method_definition")
        .and_then(|m| m.child_by_field_name("name"))
        .is_some_and(|nm| text(nm, src) == "constructor")
}

#[allow(clippy::too_many_arguments)]
fn push_field(
    found: &mut Vec<Raw>,
    cx: &Cx<'_>,
    n: Node<'_>,
    name: &str,
    holder: Option<Holder>,
    lang: crate::parse::Lang,
    src: &str,
    prefix: &str,
) {
    let Some(holder) = holder else {
        return;
    };
    let class = class_path(n, src, lang);
    if class.is_empty() {
        return;
    }
    let (kind, holder) = match holder {
        Holder::StaticGlobal => ("instance-field", Holder::ContainerField),
        Holder::RegistrationTable => ("registry-field", Holder::RegistrationTable),
        other => ("instance-field", other),
    };
    found.push(
        Raw::new(
            kind,
            holder,
            Region::Heap,
            &cx.file.rel,
            line(n),
            join_owner(prefix, &format!("{class}.{name}")),
        )
        .tracking(name, Scope::File, true),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{ProjectInfo, SourceFile},
        parse::{Lang, parse},
    };

    fn run(src: &str) -> Vec<Raw> {
        let f = SourceFile {
            rel: "src/a.js".into(),
            text: src.into(),
            lang: Lang::JavaScript,
            is_test: false,
        };
        let info = ProjectInfo::default();
        let tree = parse(Lang::JavaScript, src).unwrap();
        let mut cx = Cx::new(&f, &info);
        holders(&mut cx, tree.root_node());
        cx.out
    }

    #[test]
    fn module_values_fields_and_calls() {
        let r = run(
            "const A = 1;\nconst seen = new Map();\nconst LIST = [1,2];\nlet timer = null;\nclass L {\n  constructor() {\n    this.head = null;\n    this.items = [];\n    this.n = 0;\n  }\n}\nfunction go(s) {\n  const t = setInterval(() => {}, 5);\n  s.on('x', f);\n  const w = new Worker('a.js');\n}\n",
        );
        let got: Vec<(u32, &str, &str)> = r
            .iter()
            .map(|h| (h.line, h.kind.as_str(), h.holder.as_str()))
            .collect();
        assert_eq!(
            got,
            [
                (2, "module-global", "static/global"),
                (4, "module-global", "static/global"),
                (7, "instance-field", "container-field"),
                (8, "instance-field", "container-field"),
                (13, "timer", "blocked-stack"),
                (14, "registration", "registration-table"),
                (15, "spawn", "blocked-stack"),
            ]
        );
        assert_eq!(r[4].name.as_deref(), Some("t"));
        assert_eq!(r[5].name.as_deref(), Some("s"));
    }
}
