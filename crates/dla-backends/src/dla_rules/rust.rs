//! Rust rules: unbounded channels, detached tasks in loops or handlers, leaked boxes and
//! forgotten values, `into_raw` without `from_raw`, strong `Rc`/`Arc` cycles.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use dla_census::parse::tree_sitter::Node;
use dla_core::static_hit::StaticHit;
use regex::Regex;

use super::ast::{CallSite, ancestor, calls, find_all, line, tokens, txt};
use super::{RuleInfo, SrcFile, hit};

const CHANNEL: &str = "rust-unbounded-channel";
const DETACHED: &str = "rust-detached-task";
const BOX_LEAK: &str = "rust-box-leak";
const FORGET: &str = "rust-mem-forget";
const INTO_RAW: &str = "rust-into-raw-unpaired";
const RC_CYCLE: &str = "rust-rc-strong-cycle";

/// Rule table (mirrors the `backend = "dla"` rows of `data/detectors/rust.toml`).
pub const RULES: &[RuleInfo] = &[
    RuleInfo {
        rule: CHANNEL,
        archetype: "L-COLLECT",
        prior: 0.35,
    },
    RuleInfo {
        rule: DETACHED,
        archetype: "L-WORKER",
        prior: 0.4,
    },
    RuleInfo {
        rule: BOX_LEAK,
        archetype: "L-OWNERSHIP",
        prior: 0.3,
    },
    RuleInfo {
        rule: FORGET,
        archetype: "L-OWNERSHIP",
        prior: 0.4,
    },
    RuleInfo {
        rule: INTO_RAW,
        archetype: "L-OWNERSHIP",
        prior: 0.45,
    },
    RuleInfo {
        rule: RC_CYCLE,
        archetype: "L-CYCLE",
        prior: 0.5,
    },
];

static HANDLER_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)handle|handler|^on_|serve|accept|process|dispatch|route|connection|conn\b|_conn|request|callback|consume|listen|incoming|message|event",
    )
    .expect("regex")
});

fn enclosing_fn_name<'a>(n: Node<'_>, src: &'a str) -> &'a str {
    ancestor(n, &["function_item"])
        .and_then(|f| f.child_by_field_name("name"))
        .map_or("", |x| txt(x, src))
}

fn channels(f: &SrcFile, cs: &[CallSite<'_>]) -> Vec<StaticHit> {
    let src = f.text.as_str();
    let mentions_flume =
        src.contains("crossbeam") || src.contains("flume") || src.contains("async_channel");
    let std_channel_imported = src.contains("sync::mpsc::channel")
        || src.contains("mpsc::{channel")
        || src.contains("mpsc::channel;");
    let mut out = Vec::new();
    for c in cs {
        let recv_last = c.recv_last();
        let flagged = match c.name.as_str() {
            "unbounded_channel" => Some("unbounded_channel"),
            "unbounded"
                if c.args.is_empty()
                    && (mentions_flume
                        || recv_last.contains("channel")
                        || recv_last == "flume") =>
            {
                Some("unbounded()")
            }
            "channel"
                if c.args.is_empty()
                    && (recv_last == "mpsc" || (c.recv.is_empty() && std_channel_imported)) =>
            {
                Some("std mpsc::channel()")
            }
            _ => None,
        };
        if let Some(kind) = flagged {
            out.push(hit(
                CHANNEL,
                f,
                line(c.node),
                format!("{kind} creates a queue with no capacity limit; it grows without bound when the consumer stalls"),
            ));
        }
    }
    out
}

fn is_spawn(c: &CallSite<'_>) -> bool {
    if !matches!(c.name.as_str(), "spawn" | "spawn_local" | "spawn_blocking") {
        return false;
    }
    // Method-style `cmd.spawn()` / `join_set.spawn()` have a value receiver, not a module path.
    c.node
        .child_by_field_name("function")
        .is_some_and(|f| f.kind() != "field_expression")
        && matches!(
            c.recv.rsplit("::").next().unwrap_or(""),
            "" | "tokio" | "task" | "thread" | "smol" | "async_std" | "rayon"
        )
}

fn result_discarded(call: Node<'_>, src: &str) -> bool {
    let Some(p) = call.parent() else { return false };
    match p.kind() {
        "expression_statement" => true,
        "let_declaration" => p
            .child_by_field_name("pattern")
            .is_some_and(|pat| txt(pat, src) == "_"),
        _ => false,
    }
}

fn detached(f: &SrcFile, cs: &[CallSite<'_>]) -> Vec<StaticHit> {
    let src = f.text.as_str();
    let mut out = Vec::new();
    for c in cs {
        if !is_spawn(c) || !result_discarded(c.node, src) {
            continue;
        }
        let Some(func) = ancestor(c.node, &["function_item"]) else {
            continue;
        };
        let in_loop = {
            let mut cur = c.node.parent();
            let mut found = false;
            while let Some(p) = cur {
                if p.id() == func.id() {
                    break;
                }
                if matches!(
                    p.kind(),
                    "loop_expression" | "for_expression" | "while_expression"
                ) {
                    found = true;
                    break;
                }
                cur = p.parent();
            }
            found
        };
        let fname = enclosing_fn_name(c.node, src);
        let handler = HANDLER_NAME.is_match(fname);
        if !(in_loop || handler) || fname == "main" {
            continue;
        }
        out.push(hit(
            DETACHED,
            f,
            line(c.node),
            format!(
                "{} result is discarded {}, so nothing can cancel or join the task",
                c.name,
                if in_loop {
                    "inside a loop"
                } else {
                    "in a handler-like function"
                }
            ),
        ));
    }
    out
}

fn ownership(f: &SrcFile, cs: &[CallSite<'_>]) -> Vec<StaticHit> {
    let src = f.text.as_str();
    let has_from_raw = cs.iter().any(|c| c.name.starts_with("from_raw"));
    let forget_imported = src.contains("mem::forget") || src.contains("mem::{forget");
    let mut out = Vec::new();
    for c in cs {
        let fname = enclosing_fn_name(c.node, src);
        let once_init = {
            let ctx = ancestor(c.node, &["function_item"]).map_or("", |n| txt(n, src));
            fname == "main"
                || fname.contains("init")
                || fname.contains("setup")
                || ctx.contains("get_or_init")
                || ctx.contains("OnceLock")
                || ctx.contains("OnceCell")
                || ctx.contains("lazy_static")
        };
        let recv_last = c.recv_last();
        match c.name.as_str() {
            "leak" if recv_last == "Box" && !once_init => out.push(hit(
                BOX_LEAK,
                f,
                line(c.node),
                "Box::leak gives up ownership for good; the allocation is never freed",
            )),
            "forget" if (recv_last == "mem" || (c.recv.is_empty() && forget_imported)) && fname != "main" => {
                out.push(hit(
                    FORGET,
                    f,
                    line(c.node),
                    "mem::forget skips the value's Drop, so what it owns is never released",
                ));
            }
            "into_raw" if !has_from_raw => out.push(hit(
                INTO_RAW,
                f,
                line(c.node),
                "into_raw hands out a raw pointer and the file never calls from_raw to take it back",
            )),
            _ => {}
        }
    }
    out
}

static COLLECTION_HEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(Vec|VecDeque|HashMap|BTreeMap|HashSet|BTreeSet|LinkedList|SmallVec|IndexMap)\b")
        .expect("regex")
});

struct StrongField {
    struct_name: String,
    line: u32,
    targets: HashSet<String>,
    single: bool,
}

fn rc_cycles(files: &[SrcFile]) -> Vec<StaticHit> {
    let mut struct_names: HashSet<String> = HashSet::new();
    for f in files {
        for s in find_all(f.tree.root_node(), &["struct_item"]) {
            if let Some(n) = s.child_by_field_name("name") {
                struct_names.insert(txt(n, &f.text).to_owned());
            }
        }
    }
    let mut strong: Vec<(usize, StrongField)> = Vec::new();
    for (fi, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        for s in find_all(f.tree.root_node(), &["struct_item"]) {
            let Some(name) = s
                .child_by_field_name("name")
                .map(|n| txt(n, src).to_owned())
            else {
                continue;
            };
            for fd in find_all(s, &["field_declaration"]) {
                let Some(ty) = fd.child_by_field_name("type") else {
                    continue;
                };
                let t = txt(ty, src);
                if t.contains("Weak") || !(t.contains("Rc<") || t.contains("Arc<")) {
                    continue;
                }
                let targets: HashSet<String> = tokens(t)
                    .into_iter()
                    .filter(|x| struct_names.contains(*x))
                    .map(str::to_owned)
                    .collect();
                if targets.is_empty() {
                    continue;
                }
                strong.push((
                    fi,
                    StrongField {
                        struct_name: name.clone(),
                        line: line(fd),
                        targets,
                        single: !COLLECTION_HEAD.is_match(t.trim()),
                    },
                ));
            }
        }
    }
    let mut edges: HashMap<&str, HashSet<&str>> = HashMap::new();
    for (_, s) in &strong {
        for t in &s.targets {
            edges
                .entry(s.struct_name.as_str())
                .or_default()
                .insert(t.as_str());
        }
    }
    let mut out = Vec::new();
    for (fi, s) in &strong {
        let own: Vec<&StrongField> = strong
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.struct_name == s.struct_name && x.targets.contains(&s.struct_name))
            .collect();
        let self_cycle = s.targets.contains(&s.struct_name)
            && own.len() >= 2
            && own.iter().any(|x| x.single)
            && s.single;
        let mutual = s.targets.iter().any(|t| {
            t != &s.struct_name
                && edges
                    .get(t.as_str())
                    .is_some_and(|e| e.contains(s.struct_name.as_str()))
        });
        if self_cycle || mutual {
            out.push(hit(
                RC_CYCLE,
                &files[*fi],
                s.line,
                format!(
                    "`{}` holds a strong Rc/Arc back to a type that points back at it (parent/child or mutual references) and no Weak breaks the cycle",
                    s.struct_name
                ),
            ));
        }
    }
    out
}

/// Runs the Rust rules over the Rust files.
pub fn check(files: &[SrcFile]) -> Vec<StaticHit> {
    let mut hits = Vec::new();
    for f in files {
        let cs = calls(f, f.tree.root_node());
        hits.extend(channels(f, &cs));
        hits.extend(detached(f, &cs));
        hits.extend(ownership(f, &cs));
    }
    hits.extend(rc_cycles(files));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dla_rules::{parse_sources, sites};

    fn run(src: &str) -> Vec<StaticHit> {
        check(&parse_sources(&[("src/lib.rs", src)]))
    }

    #[test]
    fn unbounded_channels_are_flagged_and_bounded_are_not() {
        let bad = "fn f() {\n    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<u8>();\n    let (a, b) = std::sync::mpsc::channel();\n    let (c, d) = crossbeam_channel::unbounded();\n}\n";
        assert_eq!(
            sites(&run(bad), CHANNEL),
            ["src/lib.rs:2", "src/lib.rs:3", "src/lib.rs:4"]
        );
        let good = "fn f() {\n    let (tx, rx) = tokio::sync::mpsc::channel(100);\n    let (a, b) = std::sync::mpsc::sync_channel(8);\n    let (c, d) = crossbeam_channel::bounded(8);\n}\n";
        assert!(run(good).is_empty());
    }

    #[test]
    fn detached_spawn_in_loop_or_handler() {
        let bad = "async fn serve(l: Listener) {\n    loop {\n        let s = l.accept().await;\n        tokio::spawn(async move { work(s).await; });\n    }\n}\nfn on_request() {\n    let _ = std::thread::spawn(|| {});\n}\n";
        assert_eq!(sites(&run(bad), DETACHED), ["src/lib.rs:4", "src/lib.rs:8"]);
    }

    #[test]
    fn stored_handles_and_one_off_spawns_are_quiet() {
        let good = "async fn serve(l: Listener) {\n    let mut set = JoinSet::new();\n    loop {\n        let s = l.accept().await;\n        set.spawn(work(s));\n        let h = tokio::spawn(work2());\n        handles.push(tokio::spawn(work3()));\n    }\n}\nfn main() {\n    tokio::spawn(background());\n    let mut c = std::process::Command::new(\"x\");\n    c.spawn();\n}\nfn startup() {\n    std::thread::spawn(|| {});\n}\n";
        assert!(run(good).is_empty(), "{:?}", run(good));
    }

    #[test]
    fn leak_forget_and_into_raw() {
        let bad = "fn make(b: Box<T>) -> &'static mut T {\n    Box::leak(b)\n}\nfn drop_it(v: Big) {\n    std::mem::forget(v);\n}\nfn export(v: T) -> *mut T {\n    Box::into_raw(Box::new(v))\n}\n";
        let hits = run(bad);
        assert_eq!(sites(&hits, BOX_LEAK), ["src/lib.rs:2"]);
        assert_eq!(sites(&hits, FORGET), ["src/lib.rs:5"]);
        assert_eq!(sites(&hits, INTO_RAW), ["src/lib.rs:8"]);
    }

    #[test]
    fn paired_or_startup_ownership_transfers_are_quiet() {
        let good = "fn export(v: T) -> *mut T {\n    Box::into_raw(Box::new(v))\n}\nunsafe fn free(p: *mut T) {\n    drop(Box::from_raw(p));\n}\nfn main() {\n    let cfg: &'static Config = Box::leak(Box::new(Config::default()));\n    std::mem::forget(cfg);\n}\nfn init_global() -> &'static Table {\n    Box::leak(Box::new(Table::new()))\n}\n";
        assert!(run(good).is_empty(), "{:?}", run(good));
    }

    #[test]
    fn parent_child_rc_cycle() {
        let bad = "struct Node {\n    parent: Option<Rc<RefCell<Node>>>,\n    children: Vec<Rc<RefCell<Node>>>,\n}\n";
        assert_eq!(sites(&run(bad), RC_CYCLE), ["src/lib.rs:2"]);
        let weak = "struct Node {\n    parent: Option<Weak<RefCell<Node>>>,\n    children: Vec<Rc<RefCell<Node>>>,\n}\n";
        assert!(run(weak).is_empty());
        let list = "struct Node {\n    next: Option<Rc<RefCell<Node>>>,\n}\n";
        assert!(run(list).is_empty());
        let mutual = "struct A {\n    b: Rc<RefCell<B>>,\n}\nstruct B {\n    a: Option<Rc<RefCell<A>>>,\n}\n";
        assert_eq!(
            sites(&run(mutual), RC_CYCLE),
            ["src/lib.rs:2", "src/lib.rs:5"]
        );
    }
}
