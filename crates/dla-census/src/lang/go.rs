//! Go holders.

use dla_core::finding::{Holder, Region};
use regex::Regex;
use std::sync::OnceLock;

use super::{CallRule, Cx, Table, dedupe, describe, join_owner, rules};
use crate::{
    ast::{self, children_of, first_word, last_word, text, visit},
    model::{Raw, Scope},
    parse::{Lang, parse, tree_sitter::Node},
    vocab,
};

/// Type names declared with a field of their own type (through a pointer or slice).
pub fn selfref(src: &str) -> Vec<String> {
    let Ok(tree) = parse(Lang::Go, src) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    visit(tree.root_node(), &mut |n| {
        if n.kind() != "type_spec" {
            return;
        }
        let (Some(name), Some(ty)) = (n.child_by_field_name("name"), n.child_by_field_name("type"))
        else {
            return;
        };
        let name = text(name, src);
        if ty.kind() != "struct_type" {
            return;
        }
        let mut found = false;
        visit(ty, &mut |f| {
            if f.kind() == "field_declaration"
                && let Some(t) = f.child_by_field_name("type")
            {
                let tt = text(t, src);
                found |= tt.trim_start_matches(['*', '[', ']']) == name;
            }
        });
        if found {
            out.push(name.to_owned());
        }
    });
    out
}

fn container_type() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^\*?(?:map\[|\[\]|chan\b|<-chan\b|chan<-|sync\.Map\b|sync\.Pool\b|list\.List\b|ring\.Ring\b|container/|lru\.|\*?\w*(?:Cache|Registry|Pool|Queue|Store|Table|Index|Buffer)$)").expect("static")
    })
}

fn handle_type() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"^\*?(?:os\.File|net\.Conn|net\.Listener|net\.UDPConn|net\.TCPConn|sql\.DB|sql\.Rows|sql\.Stmt|http\.Response|http\.Server|grpc\.ClientConn|exec\.Cmd|zip\.Reader|gzip\.Reader|bufio\.Scanner)$").expect("static")
    })
}

fn call_table() -> &'static [CallRule] {
    static T: Table = Table::new();
    T.get_or_init(|| {
        use Holder::*;
        use Region::*;
        rules(&[
            (r"^time\.(?:NewTicker|Tick|AfterFunc|NewTimer)$", "timer", BlockedStack, NonHeap, true),
            (r"^(?:os\.(?:Open|Create|OpenFile)|net\.(?:Dial|DialTimeout|Listen|ListenPacket)|sql\.Open|http\.(?:Get|Post|Head|PostForm)|grpc\.(?:Dial|NewClient)|exec\.Command|os\.StartProcess)$", "handle", NativeHandleGlobalRef, OffHeap, true),
            (r"^signal\.Notify$|(?:^|\.)(?:HandleFunc|Handle)$", "registration", RegistrationTable, Heap, false),
        ])
    })
}

/// Enumerates the holders of one Go file.
pub fn holders(cx: &mut Cx<'_>, root: Node<'_>) {
    let src = cx.src();
    let prefix = cx.owner_prefix();
    let dir = cx
        .file
        .rel
        .rsplit_once('/')
        .map_or("", |(d, _)| d)
        .to_owned();
    let mut found: Vec<Raw> = Vec::new();
    visit(root, &mut |n| match n.kind() {
        "var_spec"
            if !ast::ancestors(n).any(|a| {
                matches!(
                    a.kind(),
                    "function_declaration" | "method_declaration" | "func_literal"
                )
            }) =>
        {
            let ty = n
                .child_by_field_name("type")
                .map(|t| text(t, src))
                .unwrap_or("");
            let val = n
                .child_by_field_name("value")
                .map(|t| text(t, src))
                .unwrap_or("");
            let mut c = n.walk();
            for id in n.children_by_field_name("name", &mut c) {
                let name = text(id, src);
                if name == "_" {
                    continue;
                }
                let Some((kind, holder, region)) = classify_var(name, ty, val) else {
                    continue;
                };
                found.push(
                    Raw::new(
                        kind,
                        holder,
                        region,
                        &cx.file.rel,
                        ast::line(id),
                        join_owner(&prefix, name),
                    )
                    .tracking(name, Scope::Project, false),
                );
            }
        }
        "field_declaration"
            if n.parent()
                .is_some_and(|p| p.kind() == "field_declaration_list") =>
        {
            let Some(ts) = ast::ancestors(n).find(|a| a.kind() == "type_spec") else {
                return;
            };
            let tname = ts
                .child_by_field_name("name")
                .map(|t| text(t, src))
                .unwrap_or("");
            let ty = n
                .child_by_field_name("type")
                .map(|t| text(t, src))
                .unwrap_or("")
                .trim();
            let names: Vec<&str> = children_of(n, "field_identifier")
                .into_iter()
                .map(|i| text(i, src))
                .collect();
            for name in names {
                let exported = name.chars().next().is_some_and(char::is_uppercase);
                let base = ty.trim_start_matches(['*', '[', ']']);
                let (kind, holder, region) =
                    if ty.starts_with("func(") || ty.contains("[]func(") || ty.contains("]func(") {
                        ("closure-field", Holder::ClosureContinuation, Region::Heap)
                    } else if vocab::registration_name().is_match(name)
                        && (container_type().is_match(ty) || ty.contains("func"))
                    {
                        ("registry-field", Holder::RegistrationTable, Region::Heap)
                    } else if container_type().is_match(ty)
                        || (cx.info.selfref.contains(base)
                            && (base == tname || ty.starts_with('*') || ty.starts_with("[]")))
                    {
                        ("container-field", Holder::ContainerField, Region::Heap)
                    } else if handle_type().is_match(ty) {
                        (
                            "handle-field",
                            Holder::NativeHandleGlobalRef,
                            Region::OffHeap,
                        )
                    } else if ty.contains("time.Ticker") || ty.contains("time.Timer") {
                        ("timer-field", Holder::BlockedStack, Region::NonHeap)
                    } else {
                        continue;
                    };
                let scope = if exported {
                    Scope::Project
                } else {
                    Scope::Dir(dir.clone())
                };
                found.push(
                    Raw::new(
                        kind,
                        holder,
                        region,
                        &cx.file.rel,
                        ast::line(n),
                        join_owner(&prefix, &format!("{tname}.{name}")),
                    )
                    .tracking(name, scope, true),
                );
            }
        }
        "go_statement" => {
            let (path, range) = describe(n, src, Lang::Go);
            let target = n.named_child(0).map(|c| text(c, src)).unwrap_or("");
            let label = if target.starts_with("func") {
                "func literal".to_owned()
            } else {
                first_line(target)
            };
            let owner = join_owner(&prefix, &path);
            let mut r = Raw::new(
                "goroutine",
                Holder::BlockedStack,
                Region::NonHeap,
                &cx.file.rel,
                ast::line(n),
                format!("{owner} -> {label}"),
            );
            if let Some((a, b)) = range {
                r = r.tracking("", Scope::Lines(a, b), false);
                r.name = None;
            }
            found.push(r);
        }
        "call_expression" => {
            let Some(f) = n.child_by_field_name("function") else {
                return;
            };
            let callee = text(f, src);
            let recv = f.child_by_field_name("operand").map(|o| text(o, src));
            let mut tmp = Cx::new(cx.file, cx.info);
            tmp.call(n, callee, recv, call_table());
            found.extend(tmp.out);
        }
        _ => {}
    });
    cx.out.extend(dedupe(found));
}

fn first_line(s: &str) -> String {
    let l = s.lines().next().unwrap_or("");
    l.chars().take(60).collect()
}

fn classify_var(name: &str, ty: &str, val: &str) -> Option<(&'static str, Holder, Region)> {
    let ty = ty.trim();
    let val = val.trim();
    if val.starts_with("errors.New")
        || val.starts_with("fmt.Errorf")
        || val.starts_with("regexp.")
        || val.starts_with("template.Must")
    {
        return None;
    }
    if ty == "error"
        || matches!(
            ty,
            "int"
                | "int32"
                | "int64"
                | "uint"
                | "uint32"
                | "uint64"
                | "string"
                | "bool"
                | "float64"
                | "float32"
                | "byte"
        )
    {
        return None;
    }
    let registration = vocab::registration_name().is_match(name);
    let container = container_type().is_match(ty)
        || val.starts_with("make(")
        || val.starts_with("map[")
        || val.starts_with("[]")
        || val.starts_with("&") && val.contains('{')
        || val.starts_with("sync.");
    if ty.starts_with("func(") || val.starts_with("func(") {
        return Some(("static-closure", Holder::ClosureContinuation, Region::Heap));
    }
    if registration && (container || val.is_empty()) {
        return Some(("package-registry", Holder::RegistrationTable, Region::Heap));
    }
    if container {
        return Some(("package-var", Holder::StaticGlobal, Region::Heap));
    }
    if handle_type().is_match(ty) {
        return Some((
            "package-handle",
            Holder::NativeHandleGlobalRef,
            Region::OffHeap,
        ));
    }
    if (val.starts_with("New") || val.contains(".New") || val.starts_with("new("))
        && first_word(val).is_some()
    {
        return Some(("package-var", Holder::StaticGlobal, Region::Heap));
    }
    if ty.starts_with('*')
        || ty.starts_with("sync.")
        || (ty.chars().next().is_some_and(char::is_uppercase) && last_word(ty).is_some())
    {
        return Some(("package-var", Holder::StaticGlobal, Region::Heap));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ProjectInfo, SourceFile};

    fn run(src: &str) -> Vec<Raw> {
        let f = SourceFile {
            rel: "p/a.go".into(),
            text: src.into(),
            lang: Lang::Go,
            is_test: false,
        };
        let mut info = ProjectInfo::default();
        info.selfref.extend(selfref(src));
        let tree = parse(Lang::Go, src).unwrap();
        let mut cx = Cx::new(&f, &info);
        holders(&mut cx, tree.root_node());
        cx.out
    }

    #[test]
    fn package_vars_fields_goroutines_timers() {
        let r = run(
            "package p\nvar registry = map[string]int{}\nvar ErrX = errors.New(\"x\")\nvar n int\ntype Pool struct {\n\tmu sync.Mutex\n\tfree [][]byte\n\tnext *Pool\n\tdone chan struct{}\n}\nfunc (p *Pool) Run() {\n\tgo func() { <-p.done }()\n\tt := time.NewTicker(time.Second)\n\tdefer t.Stop()\n}\n",
        );
        let got: Vec<(u32, &str)> = r.iter().map(|h| (h.line, h.holder.as_str())).collect();
        assert_eq!(
            got,
            [
                (2, "registration-table"),
                (7, "container-field"),
                (8, "container-field"),
                (9, "container-field"),
                (12, "blocked-stack"),
                (13, "blocked-stack"),
            ]
        );
        assert_eq!(r[4].owner, "p.a.Pool.Run -> func literal");
        assert_eq!(r[5].name.as_deref(), Some("t"));
    }
}
