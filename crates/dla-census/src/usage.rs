//! Heuristic writers and removers of a holder across the project.
//!
//! The search is textual and name based: it finds calls such as `name.add(..)`,
//! `name[k] = ..`, `append(name, ..)`, `name <- ..` (writers) and `name.remove(..)`,
//! `delete(name, ..)`, `free(name)`, `name = nil` (removers). It is a lead for the
//! reader, not an analysis; the raw census output marks it as heuristic.

use std::collections::HashMap;

use regex::Regex;

use crate::{
    model::{Raw, Scope, SourceFile},
    parse::Lang,
};

/// Most sites kept per holder and direction.
pub const MAX_SITES: usize = 24;

/// Heuristic writers and removers of one holder, as `file:line`.
#[derive(Debug, Default, Clone)]
pub struct Usage {
    /// Sites that add.
    pub writers: Vec<String>,
    /// Sites that remove.
    pub removers: Vec<String>,
}

const CHAIN: &str = r"(?:\s*\(\s*[^()\n]*\)|\s*\[[^\]\n]*\]|\s*\.\s*(?:borrow_mut|borrow|lock|write|read|unwrap|expect|get_mut|as_mut|as_ref|entry|or_default|or_insert_with|or_insert|get|Load|Store)\s*(?:\([^()\n]*\))?)*";
const W_PREFIX: &str = "add|put|push|offer|append|insert|register|subscribe|emplace|enqueue|attach|compute|schedule|track|enter|intern|remember|memoize|cache|record";
const W_EXACT: &str = "on|once|set|setdefault|update|merge|extend|store|listen|watch|prepend|unshift|write|send|Add|Store|Set|Put|Push";
const R_PREFIX: &str = "remove|delete|clear|pop|poll|drain|evict|unregister|unsubscribe|detach|release|dispose|destroy|shutdown|retain|purge|prune|expire|trim|invalidate|cancel|terminate|abort|dequeue|reap";
const R_EXACT: &str = "take|discard|off|close|free|stop|reset|truncate|shift|splice|swap_remove|unlink|kill|cache_clear|Delete|Close|Stop|Swap|Clear|Pop";
const EMPTY_RHS: &str = r"^(?:nil|None|null|undefined|NULL|nullptr|0|false|\[\s*\]|\{\s*\}|\x22\x22|(?:Vec|HashMap|HashSet|BTreeMap|BTreeSet|VecDeque|dict|list|set|deque|Map|Set|Array)(?:::new)?(?:<[^>]*>)?\(\s*\)|new\s+(?:Map|Set|Array|WeakMap|WeakSet|[A-Z]\w*)(?:<[^>]*>)?\(\s*\)|make\([^)]*\)|None\s*;?)\s*[;,]?\s*(?://.*|#.*)?$";

struct Patterns {
    writer_call: Regex,
    remover_call: Regex,
    assign: Regex,
    remover_fn: Regex,
    writer_fn: Option<Regex>,
    chan_send: Regex,
    chan_recv: Regex,
}

fn build(name: &str, member_access: bool) -> Option<Patterns> {
    let n = regex::escape(name);
    let pre = if member_access {
        format!(r"(?:\.|->|::)\s*{n}\b")
    } else {
        format!(r"\b{n}\b")
    };
    let sep = r"\s*(?:\.|->|::|\?\.)\s*";
    let call = |prefix: &str, exact: &str| {
        Regex::new(&format!(
            r"{pre}{CHAIN}{sep}(?:(?:{prefix})\w*|(?:{exact}))\s*(?:\(|<)"
        ))
        .ok()
    };
    let remover_fn = Regex::new(&format!(
        r"\b(?:delete|free|close|clearInterval|clearTimeout|clearImmediate|cancel|release|drop|unlink|remove|destroy|kill|abort|shutdown|dispose|join|pthread_\w*)\w*\s*\(\s*[^)\n]*{pre}|\bdel\s+[^\n]*{pre}"
    ))
    .ok()?;
    let writer_fn = Regex::new(&format!(r"\bappend\(\s*[\w.*&\[\]]*{pre}")).ok();
    Some(Patterns {
        writer_call: call(W_PREFIX, W_EXACT)?,
        remover_call: call(R_PREFIX, R_EXACT)?,
        assign: Regex::new(&format!(r"{pre}{CHAIN}\s*([-+*|&^]?)=(?:[^=>~]|$)\s*(.*)$")).ok()?,
        remover_fn,
        writer_fn,
        chan_send: Regex::new(&format!(r"{pre}\s*<-")).ok()?,
        chan_recv: Regex::new(&format!(r"<-\s*[\w.*&]*{pre}")).ok()?,
    })
}

fn empty_rhs() -> &'static Regex {
    static R: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| Regex::new(EMPTY_RHS).expect("static pattern"))
}

/// Source lines with comments blanked, kept per file.
struct Code<'a> {
    file: &'a SourceFile,
    lines: Vec<String>,
}

fn strip_comment(lang: Lang, l: &str) -> String {
    let t = l.trim_start();
    if lang == Lang::Python {
        if t.starts_with('#') {
            return String::new();
        }
        return match l.find(" #") {
            Some(i) => l[..i].to_owned(),
            None => l.to_owned(),
        };
    }
    if t.starts_with("//") || t.starts_with('*') || t.starts_with("/*") {
        return String::new();
    }
    let mut from = 0;
    while let Some(i) = l[from..].find("//") {
        let at = from + i;
        if at == 0 || !l[..at].ends_with(':') {
            return l[..at].to_owned();
        }
        from = at + 2;
    }
    l.to_owned()
}

fn dir_of(rel: &str) -> &str {
    rel.rsplit_once('/').map_or("", |(d, _)| d)
}

/// Finds writers and removers for every raw holder, in the same order.
pub fn find(raws: &[Raw], files: &[SourceFile]) -> Vec<Usage> {
    let code: Vec<Code> = files
        .iter()
        .filter(|f| !f.is_test)
        .map(|f| Code {
            file: f,
            lines: f.text.lines().map(|l| strip_comment(f.lang, l)).collect(),
        })
        .collect();
    let mut cache: HashMap<(String, bool), Option<Patterns>> = HashMap::new();
    let mut out = Vec::with_capacity(raws.len());
    for r in raws {
        let Some(name) = r
            .name
            .as_deref()
            .filter(|n| n.len() > 1 || matches!(r.scope, Scope::Lines(..) | Scope::File))
        else {
            out.push(Usage::default());
            continue;
        };
        let pats = cache
            .entry((name.to_owned(), r.member_access))
            .or_insert_with(|| build(name, r.member_access));
        let Some(p) = pats.as_ref() else {
            out.push(Usage::default());
            continue;
        };
        let mut u = Usage::default();
        for c in &code {
            let in_scope = match &r.scope {
                Scope::Project => true,
                Scope::Dir(d) => dir_of(&c.file.rel) == d,
                Scope::File | Scope::Lines(..) => c.file.rel == r.file,
            };
            if !in_scope {
                continue;
            }
            let (lo, hi) = match r.scope {
                Scope::Lines(a, b) => (a, b),
                _ => (1, u32::MAX),
            };
            for (i, l) in c.lines.iter().enumerate() {
                let ln = u32::try_from(i + 1).unwrap_or(u32::MAX);
                if ln < lo || ln > hi || l.trim().is_empty() {
                    continue;
                }
                if c.file.rel == r.file && ln == r.line {
                    continue;
                }
                classify(p, l, &mut u, &format!("{}:{ln}", c.file.rel));
            }
        }
        u.writers.truncate(MAX_SITES);
        u.removers.truncate(MAX_SITES);
        out.push(u);
    }
    out
}

fn classify(p: &Patterns, l: &str, u: &mut Usage, site: &str) {
    let mut writer = p.writer_call.is_match(l)
        || p.chan_send.is_match(l)
        || p.writer_fn.as_ref().is_some_and(|w| w.is_match(l));
    let mut remover =
        p.remover_call.is_match(l) || p.remover_fn.is_match(l) || p.chan_recv.is_match(l);
    if let Some(c) = p.assign.captures(l) {
        let op = c.get(1).map_or("", |m| m.as_str());
        let rhs = c.get(2).map_or("", |m| m.as_str()).trim();
        if op == "-" || empty_rhs().is_match(rhs) {
            remover = true;
        } else {
            writer = true;
        }
    }
    if writer && !u.writers.iter().any(|s| s == site) {
        u.writers.push(site.to_owned());
    }
    if remover && !u.removers.iter().any(|s| s == site) {
        u.removers.push(site.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::finding::{Holder, Region};

    fn file(rel: &str, text: &str) -> SourceFile {
        SourceFile {
            rel: rel.into(),
            text: text.into(),
            lang: Lang::from_path(std::path::Path::new(rel)).unwrap(),
            is_test: false,
        }
    }

    fn raw(name: &str, file: &str, line: u32, scope: Scope, member: bool) -> Raw {
        Raw::new(
            "k",
            Holder::StaticGlobal,
            Region::Heap,
            file,
            line,
            "o".into(),
        )
        .tracking(name, scope, member)
    }

    #[test]
    fn python_module_container() {
        let f = file(
            "a/reg.py",
            "_L = {}\n\ndef sub(t, l):\n    _L.setdefault(t, []).append(l)\n\ndef reset():\n    _L.clear()\n\n# _L.pop(x)\n",
        );
        let u = find(&[raw("_L", "a/reg.py", 1, Scope::Project, false)], &[f]);
        assert_eq!(u[0].writers, ["a/reg.py:4"]);
        assert_eq!(u[0].removers, ["a/reg.py:7"]);
    }

    #[test]
    fn go_slice_and_chan() {
        let f = file(
            "p/pool.go",
            "func (p *P) Put(b []byte) {\n\tp.free = append(p.free, b)\n}\nfunc (p *P) Get() {\n\tp.free = p.free[:n-1]\n\tp.free = nil\n\tdelete(p.m, 1)\n\tch <- 1\n}\n",
        );
        let u = find(
            &[raw("free", "p/pool.go", 99, Scope::Dir("p".into()), true)],
            &[f],
        );
        assert!(u[0].writers.contains(&"p/pool.go:2".to_owned()), "{u:?}");
        assert!(u[0].removers.contains(&"p/pool.go:6".to_owned()), "{u:?}");
    }

    #[test]
    fn rust_borrow_chain_and_member_access_only() {
        let f = file(
            "s.rs",
            "fn a(c: Rc<S>, p: &Rc<S>) {\n    *c.parent.borrow_mut() = Some(Rc::clone(p));\n    p.children.borrow_mut().push(c);\n    let children = 1;\n    children.push(2);\n}\n",
        );
        let u = find(&[raw("children", "s.rs", 99, Scope::File, true)], &[f]);
        assert_eq!(u[0].writers, ["s.rs:3"]);
        assert!(u[0].removers.is_empty());
    }

    #[test]
    fn lines_scope_limits_to_function_and_timers() {
        let f = file(
            "t.js",
            "function a() {\n  const t = setInterval(f, 5);\n  clearInterval(t);\n}\nfunction b() {\n  clearInterval(t);\n}\n",
        );
        let u = find(&[raw("t", "t.js", 2, Scope::Lines(1, 4), false)], &[f]);
        assert_eq!(u[0].removers, ["t.js:3"]);
    }

    #[test]
    fn comments_and_test_files_are_ignored() {
        let mut t = file("x_test.go", "x.items.clear()\n");
        t.is_test = true;
        let f = file("a.js", "// items.clear()\nitems.push(1)\n");
        let u = find(&[raw("items", "a.js", 9, Scope::Project, false)], &[t, f]);
        assert_eq!(u[0].writers, ["a.js:2"]);
        assert!(u[0].removers.is_empty());
    }
}
