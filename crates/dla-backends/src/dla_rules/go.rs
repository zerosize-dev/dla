//! Go rules: goroutines without a stop path, `time.Tick`, tickers never stopped.

use std::collections::{HashMap, HashSet};

use dla_census::parse::tree_sitter::Node;
use dla_core::static_hit::StaticHit;

use super::ast::{
    CallSite, calls, child_of_kind, find_all, last_token, line, named_children, tokens, txt, walk,
};
use super::{RuleInfo, SrcFile, hit};

const GOROUTINE: &str = "go-goroutine-no-stop-path";
const TICK: &str = "go-time-tick";
const TICKER: &str = "go-ticker-no-stop";

/// Rule table (mirrors `data/detectors/go.dla-fragment.toml`).
pub const RULES: &[RuleInfo] = &[
    RuleInfo {
        rule: GOROUTINE,
        archetype: "L-WORKER",
        prior: 0.45,
    },
    RuleInfo {
        rule: TICK,
        archetype: "L-WORKER",
        prior: 0.6,
    },
    RuleInfo {
        rule: TICKER,
        archetype: "L-WORKER",
        prior: 0.5,
    },
];

fn is_stop_token(t: &str) -> bool {
    let l = t.to_ascii_lowercase();
    const EXACT: [&str; 12] = [
        "done", "stop", "quit", "cancel", "shutdown", "closing", "closed", "exit", "kill", "term",
        "abort", "finish",
    ];
    EXACT.contains(&l.as_str())
        || [
            "done", "stop", "quit", "cancel", "shutdown", "closing", "closed", "abort",
        ]
        .iter()
        .any(|s| l.ends_with(s) || l.starts_with(s))
}

fn has_stop_signal(n: Node<'_>, src: &str) -> bool {
    let t = txt(n, src);
    t.contains(".Err()") || tokens(t).into_iter().any(is_stop_token)
}

/// True when the subtree can leave the enclosing loop (return, labelled or direct break,
/// goto, panic or a process exit). `nested` is true inside a select, switch or inner loop,
/// where a bare `break` only leaves that construct.
fn can_exit(n: Node<'_>, src: &str, nested: bool) -> bool {
    let mut c = n.walk();
    let kids: Vec<Node<'_>> = n.children(&mut c).collect();
    for k in kids {
        match k.kind() {
            "return_statement" | "goto_statement" => return true,
            "break_statement" => {
                if !nested || child_of_kind(k, &["label_name"]).is_some() {
                    return true;
                }
            }
            "call_expression" => {
                let f = k
                    .child_by_field_name("function")
                    .map_or("", |f| txt(f, src));
                if matches!(
                    f,
                    "panic"
                        | "os.Exit"
                        | "log.Fatal"
                        | "log.Fatalf"
                        | "log.Fatalln"
                        | "runtime.Goexit"
                ) {
                    return true;
                }
                if can_exit(k, src, nested) {
                    return true;
                }
            }
            "func_literal" => {}
            "select_statement"
            | "expression_switch_statement"
            | "type_switch_statement"
            | "for_statement" => {
                if can_exit(k, src, true) {
                    return true;
                }
            }
            _ => {
                if can_exit(k, src, nested) {
                    return true;
                }
            }
        }
    }
    false
}

/// For loops of a body, not descending into nested function literals.
fn loops_of<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    let mut c = n.walk();
    let kids: Vec<Node<'t>> = n.children(&mut c).collect();
    for k in kids {
        match k.kind() {
            "func_literal" => {}
            "for_statement" => {
                out.push(k);
                loops_of(k, out);
            }
            _ => loops_of(k, out),
        }
    }
}

struct Index<'t> {
    funcs: HashMap<String, Vec<(usize, Node<'t>)>>,
    chans: HashSet<String>,
    closed: HashSet<String>,
}

fn build_index<'t>(files: &'t [SrcFile], calls_by_file: &[Vec<CallSite<'t>>]) -> Index<'t> {
    let mut funcs: HashMap<String, Vec<(usize, Node<'t>)>> = HashMap::new();
    let mut chans = HashSet::new();
    let mut closed = HashSet::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        walk(f.tree.root_node(), &mut |n| match n.kind() {
            "function_declaration" | "method_declaration" => {
                if let (Some(name), Some(body)) =
                    (n.child_by_field_name("name"), n.child_by_field_name("body"))
                {
                    funcs
                        .entry(txt(name, src).to_owned())
                        .or_default()
                        .push((i, body));
                }
            }
            "short_var_declaration" | "assignment_statement" | "var_spec" => {
                let t = txt(n, src);
                if t.contains("make(chan") || t.contains("make(<-chan") {
                    let left = n
                        .child_by_field_name("left")
                        .or_else(|| n.child_by_field_name("name"))
                        .or_else(|| named_children(n).into_iter().next());
                    if let Some(l) = left {
                        for tok in tokens(txt(l, src)) {
                            chans.insert(tok.to_owned());
                        }
                    }
                }
                if n.kind() == "var_spec" && !find_all(n, &["channel_type"]).is_empty() {
                    if let Some(l) = named_children(n).first() {
                        for tok in tokens(txt(*l, src)) {
                            chans.insert(tok.to_owned());
                        }
                    }
                }
            }
            "parameter_declaration" | "field_declaration"
                if child_of_kind(n, &["channel_type"]).is_some() =>
            {
                for ch in named_children(n) {
                    if matches!(ch.kind(), "identifier" | "field_identifier") {
                        chans.insert(txt(ch, src).to_owned());
                    }
                }
            }
            _ => {}
        });
        for c in &calls_by_file[i] {
            if c.name == "close" && c.recv.is_empty() {
                if let Some(a) = c.args.first() {
                    closed.insert(last_token(txt(*a, src)));
                }
            }
        }
    }
    Index {
        funcs,
        chans,
        closed,
    }
}

/// Describes why a loop never ends, or `None` when it has a stop path.
fn leaky_loop(l: Node<'_>, src: &str, idx: &Index<'_>) -> Option<String> {
    let kids = named_children(l);
    let header = kids.iter().find(|k| k.kind() != "block").copied();
    let body = l.child_by_field_name("body");
    match header {
        None => {
            let body = body?;
            let has_select = !find_all(body, &["select_statement"]).is_empty();
            if has_stop_signal(body, src) {
                return None;
            }
            if !has_select && can_exit(body, src, false) {
                return None;
            }
            Some(if has_select {
                "`for { select { ... } }` with no done, context or stop case".to_owned()
            } else {
                "infinite `for` loop with no return, break, done or context check".to_owned()
            })
        }
        Some(h) if h.kind() == "range_clause" => {
            let range_expr = h
                .child_by_field_name("right")
                .map_or_else(|| txt(h, src).to_owned(), |r| txt(r, src).to_owned());
            let name = last_token(&range_expr);
            let ticking = range_expr.contains("time.Tick(") || range_expr.ends_with(".C");
            if ticking {
                let body = body?;
                if has_stop_signal(body, src) || can_exit(body, src, false) {
                    return None;
                }
                return Some("ranges over a ticker with no stop path".to_owned());
            }
            if idx.chans.contains(&name) && !idx.closed.contains(&name) {
                if body.is_some_and(|b| has_stop_signal(b, src) && can_exit(b, src, false)) {
                    return None;
                }
                return Some(format!(
                    "ranges over channel `{name}` that is never closed in the project"
                ));
            }
            None
        }
        _ => None,
    }
}

fn goroutines(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let idx = build_index(files, calls_by_file);
    let mut out = Vec::new();
    for (fi, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        for g in find_all(f.tree.root_node(), &["go_statement"]) {
            let Some(call) = child_of_kind(g, &["call_expression"]) else {
                continue;
            };
            let Some(callee) = call.child_by_field_name("function") else {
                continue;
            };
            let bodies: Vec<(usize, Node<'_>)> = if callee.kind() == "func_literal" {
                callee
                    .child_by_field_name("body")
                    .map(|b| (fi, b))
                    .into_iter()
                    .collect()
            } else {
                let name = last_token(txt(callee, src));
                idx.funcs.get(&name).cloned().unwrap_or_default()
            };
            if bodies.is_empty() {
                continue;
            }
            let mut reason = None;
            let mut all_leaky = true;
            for (bi, b) in &bodies {
                let mut loops = Vec::new();
                loops_of(*b, &mut loops);
                let bsrc = files[*bi].text.as_str();
                let found = loops
                    .iter()
                    .find_map(|l| leaky_loop(*l, bsrc, &idx).map(|r| (line(*l), r)));
                match found {
                    Some((ln, why)) => reason = Some((ln, why)),
                    None => all_leaky = false,
                }
            }
            if let (true, Some((ln, why))) = (all_leaky, reason) {
                out.push(hit(
                    GOROUTINE,
                    f,
                    line(g),
                    format!("goroutine has no stop path: loop at line {ln} {why}"),
                ));
            }
        }
    }
    out
}

fn tickers(files: &[SrcFile], calls_by_file: &[Vec<CallSite<'_>>]) -> Vec<StaticHit> {
    let mut out = Vec::new();
    for (i, f) in files.iter().enumerate() {
        let src = f.text.as_str();
        let stopped: HashSet<String> = calls_by_file[i]
            .iter()
            .filter(|c| c.name == "Stop")
            .map(CallSite::recv_last)
            .collect();
        for c in &calls_by_file[i] {
            if c.recv == "time" && c.name == "Tick" {
                out.push(hit(
                    TICK,
                    f,
                    line(c.node),
                    "time.Tick cannot be stopped, so its ticker is never released; use time.NewTicker and Stop",
                ));
            }
            if c.recv == "time" && c.name == "NewTicker" {
                let Some(p) = c.node.parent() else { continue };
                let holder = match p.kind() {
                    "expression_list" => p.parent(),
                    _ => Some(p),
                };
                let Some(h) = holder else { continue };
                if !matches!(
                    h.kind(),
                    "short_var_declaration" | "assignment_statement" | "var_spec"
                ) {
                    continue;
                }
                let left = h
                    .child_by_field_name("left")
                    .or_else(|| named_children(h).into_iter().next());
                let Some(l) = left else { continue };
                let name = last_token(txt(l, src));
                if name.is_empty() || name == "_" || stopped.contains(&name) {
                    continue;
                }
                out.push(hit(
                    TICKER,
                    f,
                    line(c.node),
                    format!("ticker `{name}` is created here and `{name}.Stop()` is never called in this file"),
                ));
            }
        }
    }
    out
}

/// Runs the Go rules over the Go files.
pub fn check(files: &[SrcFile]) -> Vec<StaticHit> {
    let calls_by_file: Vec<Vec<CallSite<'_>>> =
        files.iter().map(|f| calls(f, f.tree.root_node())).collect();
    let mut hits = goroutines(files, &calls_by_file);
    hits.extend(tickers(files, &calls_by_file));
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
    fn select_loop_without_stop_case_is_flagged() {
        let src = "package p\n\nfunc start(in chan int) {\n\tgo func() {\n\t\tfor {\n\t\t\tselect {\n\t\t\tcase v := <-in:\n\t\t\t\tuse(v)\n\t\t\t}\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", src)]), GOROUTINE), ["a.go:4"]);
    }

    #[test]
    fn select_loop_with_done_or_ctx_is_quiet() {
        let done = "package p\n\nfunc start(in chan int, done chan struct{}) {\n\tgo func() {\n\t\tfor {\n\t\t\tselect {\n\t\t\tcase v := <-in:\n\t\t\t\tuse(v)\n\t\t\tcase <-done:\n\t\t\t\treturn\n\t\t\t}\n\t\t}\n\t}()\n}\n";
        assert!(run(&[("a.go", done)]).is_empty());
        let ctx = "package p\n\nfunc start(ctx context.Context, in chan int) {\n\tgo func() {\n\t\tfor {\n\t\t\tselect {\n\t\t\tcase <-ctx.Done():\n\t\t\t\treturn\n\t\t\tcase v := <-in:\n\t\t\t\tuse(v)\n\t\t\t}\n\t\t}\n\t}()\n}\n";
        assert!(run(&[("a.go", ctx)]).is_empty());
    }

    #[test]
    fn named_method_goroutine_is_resolved() {
        let leak = "package p\n\ntype S struct{ in chan int }\n\nfunc (s *S) run() {\n\tfor {\n\t\tselect {\n\t\tcase v := <-s.in:\n\t\t\tuse(v)\n\t\t}\n\t}\n}\n\nfunc (s *S) Start() {\n\tgo s.run()\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), GOROUTINE), ["a.go:15"]);
        let ok = leak.replace(
            "\t\t\tuse(v)\n",
            "\t\t\tuse(v)\n\t\tcase <-s.quit:\n\t\t\treturn\n",
        );
        assert!(run(&[("a.go", &ok)]).is_empty());
    }

    #[test]
    fn range_over_never_closed_channel_is_flagged() {
        let leak = "package p\n\nfunc start() {\n\tjobs := make(chan int)\n\tgo func() {\n\t\tfor j := range jobs {\n\t\t\tuse(j)\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), GOROUTINE), ["a.go:5"]);
        let closed = "package p\n\nfunc start() {\n\tjobs := make(chan int)\n\tgo func() {\n\t\tfor j := range jobs {\n\t\t\tuse(j)\n\t\t}\n\t}()\n\tclose(jobs)\n}\n";
        assert!(run(&[("a.go", closed)]).is_empty());
        let slice = "package p\n\nfunc start(items []int) {\n\tgo func() {\n\t\tfor _, i := range items {\n\t\t\tuse(i)\n\t\t}\n\t}()\n}\n";
        assert!(run(&[("a.go", slice)]).is_empty());
    }

    #[test]
    fn polling_loops_need_an_exit() {
        let leak = "package p\n\nfunc start() {\n\tgo func() {\n\t\tfor {\n\t\t\ttime.Sleep(time.Second)\n\t\t\tpoll()\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), GOROUTINE), ["a.go:4"]);
        let exits = "package p\n\nfunc start(ln net.Listener) {\n\tgo func() {\n\t\tfor {\n\t\t\tc, err := ln.Accept()\n\t\t\tif err != nil {\n\t\t\t\treturn\n\t\t\t}\n\t\t\tserve(c)\n\t\t}\n\t}()\n}\n";
        assert!(run(&[("a.go", exits)]).is_empty());
        let finite = "package p\n\nfunc start() {\n\tgo func() {\n\t\tfor i := 0; i < 3; i++ {\n\t\t\twork()\n\t\t}\n\t}()\n\tgo work()\n}\n";
        assert!(run(&[("a.go", finite)]).is_empty());
    }

    #[test]
    fn select_break_does_not_end_the_outer_loop() {
        let src = "package p\n\nfunc start(in chan int) {\n\tgo func() {\n\t\tfor {\n\t\t\tselect {\n\t\t\tcase v := <-in:\n\t\t\t\tif v == 0 {\n\t\t\t\t\tbreak\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", src)]), GOROUTINE), ["a.go:4"]);
    }

    #[test]
    fn time_tick_and_unstopped_tickers() {
        let tick =
            "package p\n\nfunc f() {\n\tfor range time.Tick(time.Second) {\n\t\twork()\n\t}\n}\n";
        assert_eq!(sites(&run(&[("a.go", tick)]), TICK), ["a.go:4"]);
        let leak = "package p\n\nfunc f() {\n\tt := time.NewTicker(time.Second)\n\t_ = t\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), TICKER), ["a.go:4"]);
        let ok =
            "package p\n\nfunc f() {\n\tt := time.NewTicker(time.Second)\n\tdefer t.Stop()\n}\n";
        assert!(sites(&run(&[("a.go", ok)]), TICKER).is_empty());
    }

    #[test]
    fn ticker_range_loop_needs_stop() {
        let leak = "package p\n\nfunc f() {\n\tt := time.NewTicker(time.Second)\n\tdefer t.Stop()\n\tgo func() {\n\t\tfor range t.C {\n\t\t\twork()\n\t\t}\n\t}()\n}\n";
        assert_eq!(sites(&run(&[("a.go", leak)]), GOROUTINE), ["a.go:6"]);
    }
}
