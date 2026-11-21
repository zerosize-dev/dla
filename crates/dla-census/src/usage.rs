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
pub(crate) const W_PREFIX: &str = "add|put|push|offer|append|insert|register|subscribe|emplace|enqueue|attach|compute|schedule|track|enter|intern|remember|memoize|cache|record";
pub(crate) const W_EXACT: &str = "on|once|set|setdefault|update|merge|extend|store|listen|watch|prepend|unshift|write|send|Add|Store|Set|Put|Push";
pub(crate) const R_PREFIX: &str = "remove|delete|clear|pop|poll|drain|evict|unregister|unsubscribe|detach|release|dispose|destroy|shutdown|retain|purge|prune|expire|trim|invalidate|cancel|terminate|abort|dequeue|reap";
pub(crate) const R_EXACT: &str = "take|discard|off|close|free|stop|reset|truncate|shift|splice|swap_remove|unlink|kill|cache_clear|Delete|Close|Stop|Swap|Clear|Pop";
const EMPTY_RHS: &str = r"^(?:nil|None|null|undefined|NULL|nullptr|0|false|\[\s*\]|\{\s*\}|\x22\x22|(?:Vec|HashMap|HashSet|BTreeMap|BTreeSet|VecDeque|dict|list|set|deque|Map|Set|Array)(?:::new)?(?:<[^>]*>)?\(\s*\)|new\s+(?:Map|Set|Array|WeakMap|WeakSet|[A-Z]\w*)(?:<[^>]*>)?\(\s*\)|make\([^)]*\)|None\s*;?)\s*[;,]?\s*(?://.*|#.*)?$";

/// A regex compiled when a line first needs it: compiling is the costly part of a census, and
/// most of the seven patterns of a holder are never asked about a line (a pattern that needs
/// `<-` or `=` is not tried on a line without one).
#[derive(Clone)]
struct Lazy {
    src: std::sync::Arc<str>,
    re: std::sync::Arc<std::sync::OnceLock<Option<Regex>>>,
}

impl Lazy {
    fn new(src: String) -> Self {
        Self {
            src: src.into(),
            re: Default::default(),
        }
    }

    /// The regex, or `None` when the pattern does not compile.
    fn get(&self) -> Option<&Regex> {
        self.re.get_or_init(|| Regex::new(&self.src).ok()).as_ref()
    }

    fn is_match(&self, l: &str) -> bool {
        self.get().is_some_and(|r| r.is_match(l))
    }
}

#[derive(Clone)]
struct Patterns {
    writer_call: Lazy,
    remover_call: Lazy,
    assign: Lazy,
    remover_fn: Lazy,
    writer_fn: Lazy,
    chan_send: Lazy,
    chan_recv: Lazy,
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
        Lazy::new(format!(
            r"{pre}{CHAIN}{sep}(?:(?:{prefix})\w*|(?:{exact}))\s*(?:\(|<)"
        ))
    };
    let remover_fn = Lazy::new(format!(
        r"\b(?:delete|free|close|clearInterval|clearTimeout|clearImmediate|cancel|release|drop|unlink|remove|destroy|kill|abort|shutdown|dispose|join|pthread_\w*)\w*\s*\(\s*[^)\n]*{pre}|\bdel\s+[^\n]*{pre}"
    ));
    Some(Patterns {
        writer_call: call(W_PREFIX, W_EXACT),
        remover_call: call(R_PREFIX, R_EXACT),
        assign: Lazy::new(format!(
            r"{pre}({CHAIN})\s*([-+*|&^]?)=(?:[^=>~]|$)\s*(.*)$"
        )),
        remover_fn,
        writer_fn: Lazy::new(format!(r"\bappend\(\s*[\w.*&\[\]]*{pre}")),
        chan_send: Lazy::new(format!(r"{pre}\s*<-")),
        chan_recv: Lazy::new(format!(r"<-\s*[\w.*&]*{pre}")),
    })
}

pub(crate) fn empty_rhs() -> &'static Regex {
    static R: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| Regex::new(EMPTY_RHS).expect("static pattern"))
}

/// Source lines with comments blanked, kept per file, with an index of the lines each word
/// occurs on.
struct Code<'a> {
    file: &'a SourceFile,
    lines: Vec<String>,
    /// Hash of an ASCII word (a run of letters, digits and `_`) to the 1-based lines of this
    /// file that hold a word of that hash, ascending. A hash collision only adds a line the
    /// per-line test then rejects.
    words: HashMap<u64, Vec<u32>>,
    /// Lines with a non-ASCII byte, ascending: the regexes' `\b` is Unicode-aware, so such a
    /// line is always looked at.
    wild: Vec<u32>,
}

fn word_hash(w: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    w.hash(&mut h);
    h.finish()
}

impl<'a> Code<'a> {
    fn new(file: &'a SourceFile) -> Self {
        let lines: Vec<String> = file
            .text
            .lines()
            .map(|l| strip_comment(file.lang, l))
            .collect();
        let mut words: HashMap<u64, Vec<u32>> = HashMap::new();
        let mut wild = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            let ln = u32::try_from(i + 1).unwrap_or(u32::MAX);
            if !l.is_ascii() {
                wild.push(ln);
                continue;
            }
            for w in l
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .filter(|w| !w.is_empty())
            {
                let v = words.entry(word_hash(w)).or_default();
                if v.last() != Some(&ln) {
                    v.push(ln);
                }
            }
        }
        Self {
            file,
            lines,
            words,
            wild,
        }
    }

    /// The lines (1-based, ascending) a pattern built around `name` can match: every line
    /// when `name` is not a plain word, else the lines holding that word and the non-ASCII
    /// lines. Every pattern contains the name as a whole word, so no other line can match.
    fn candidates(&self, name: &str) -> Vec<u32> {
        let plain =
            !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
        if !plain {
            return (1..=u32::try_from(self.lines.len()).unwrap_or(u32::MAX)).collect();
        }
        let mut v: Vec<u32> = self
            .words
            .get(&word_hash(name))
            .cloned()
            .unwrap_or_default();
        v.extend_from_slice(&self.wild);
        v.sort_unstable();
        v.dedup();
        v
    }
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
    let code: Vec<Code> = files.iter().filter(|f| !f.is_test).map(Code::new).collect();
    let mut cache: HashMap<(String, bool), Option<Patterns>> = HashMap::new();
    let mut out = Vec::with_capacity(raws.len());
    for r in raws {
        let Some(name) = r.name.as_deref().filter(|n| {
            n.len() > 1 || matches!(r.scope, Scope::Lines(..) | Scope::File | Scope::Project)
        }) else {
            out.push(Usage::default());
            continue;
        };
        // A one-character name is too common to chase across the project: look in the
        // declaring file only.
        let scope = match &r.scope {
            Scope::Project if name.len() == 1 => &Scope::File,
            other => other,
        };
        // The lines of the scope that hold the name (no pattern can match another line), in
        // file order. A holder with none needs no patterns: compiling them is the costly
        // part, so it waits for the first line that needs them.
        let mut hits: Vec<(&Code, u32)> = Vec::new();
        for c in &code {
            let in_scope = match scope {
                Scope::Project => true,
                Scope::Dir(d) => dir_of(&c.file.rel) == d,
                Scope::File | Scope::Lines(..) => c.file.rel == r.file,
            };
            if !in_scope {
                continue;
            }
            let (lo, hi) = match scope {
                Scope::Lines(a, b) => (*a, *b),
                _ => (1, u32::MAX),
            };
            for ln in c.candidates(name) {
                let l = &c.lines[(ln - 1) as usize];
                if ln < lo || ln > hi || l.trim().is_empty() {
                    continue;
                }
                // Every pattern of `Patterns` contains the holder's name literally, so a line
                // without it matches none: the substring test replaces seven regex searches.
                if !l.contains(name) {
                    continue;
                }
                if c.file.rel == r.file && ln == r.line {
                    continue;
                }
                hits.push((c, ln));
            }
        }
        if hits.is_empty() {
            out.push(Usage::default());
            continue;
        }
        let pats = cache
            .entry((name.to_owned(), r.member_access))
            .or_insert_with(|| build(name, r.member_access))
            .clone();
        let Some(p) = pats.as_ref() else {
            out.push(Usage::default());
            continue;
        };
        // Inside the declaring file of a JVM class, a field is also used unqualified
        // (`store.put(..)` with no `this.`); elsewhere only `x.name` counts.
        let bare = if r.member_access {
            cache
                .entry((name.to_owned(), false))
                .or_insert_with(|| build(name, false))
                .clone()
        } else {
            None
        };
        let mut u = Usage::default();
        for (c, ln) in hits {
            let l = &c.lines[(ln - 1) as usize];
            let jvm_same_file =
                c.file.rel == r.file && matches!(c.file.lang, Lang::Java | Lang::Kotlin);
            let p = match (&bare, jvm_same_file) {
                (Some(b), true) => b,
                _ => p,
            };
            classify(p, l, &mut u, &format!("{}:{ln}", c.file.rel));
        }
        u.writers.truncate(MAX_SITES);
        u.removers.truncate(MAX_SITES);
        out.push(u);
    }
    out
}

fn classify(p: &Patterns, l: &str, u: &mut Usage, site: &str) {
    // Each pattern needs a literal the line must hold (`<-` for a channel, `=` for an
    // assignment, `append(`, a `(` for a remover function or `del`); a line without it
    // cannot match, so the pattern is neither compiled nor run.
    let arrow = l.contains("<-");
    let mut writer = p.writer_call.is_match(l)
        || (arrow && p.chan_send.is_match(l))
        || (l.contains("append(") && p.writer_fn.is_match(l));
    let mut remover = p.remover_call.is_match(l)
        || ((l.contains('(') || l.contains("del")) && p.remover_fn.is_match(l))
        || (arrow && p.chan_recv.is_match(l));
    if l.contains('=')
        && let Some(c) = p.assign.get().and_then(|r| r.captures(l))
    {
        let chain = c.get(1).map_or("", |m| m.as_str());
        let op = c.get(2).map_or("", |m| m.as_str());
        let rhs = c.get(3).map_or("", |m| m.as_str()).trim();
        // `x[k] = make(..)` stores a value under a key: a write, not a reset of `x`.
        let element = chain.trim_end().ends_with(']');
        if op == "-" || (!element && empty_rhs().is_match(rhs)) {
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
    fn the_word_index_finds_what_a_scan_of_every_line_finds() {
        // Lines with non-ASCII text are always looked at; a name that is not a plain word
        // (`$cache`) is looked for on every line; a longer word holding the name is not a use.
        let f = file(
            "a/reg.py",
            "_L = {}\n\ndef sub(t, l):\n    s = \"caf\u{e9}\"; _L.setdefault(t, []).append(l)\n\ndef other():\n    _LX.put(1)\n    x_L.put(2)\n\ndef reset():\n    _L.clear()\n",
        );
        let u = find(&[raw("_L", "a/reg.py", 1, Scope::Project, false)], &[f]);
        assert_eq!(u[0].writers, ["a/reg.py:4"], "{u:?}");
        assert_eq!(u[0].removers, ["a/reg.py:11"], "{u:?}");
        let decl = file(
            "a/Svc.java",
            "class Svc {\n    static Map $cache = new HashMap();\n}\n",
        );
        let user = file(
            "b/Use.java",
            "class Use {\n    void f(Object k) {\n        Svc.$cache.put(k, k);\n        Svc.cache.put(k, k);\n    }\n}\n",
        );
        let u = find(
            &[raw("$cache", "a/Svc.java", 2, Scope::Project, true)],
            &[decl, user],
        );
        assert_eq!(u[0].writers, ["b/Use.java:3"], "{u:?}");
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

    #[test]
    fn java_unqualified_field_use_and_short_static_names() {
        let f = file(
            "p/B.java",
            "class B {\n  protected final Map store = new HashMap();\n  void a(String k) { store.put(k, k); }\n  void b(String k) { store.remove(k); }\n}\nclass Z {\n  static final Deque D = new ArrayDeque();\n  static void p(String s) { D.addLast(s); }\n  static String q() { return D.pollFirst(); }\n}\n",
        );
        let other = file(
            "p/O.java",
            "class O { void f() { store.put(1, 2); D.addLast(1); } }\n",
        );
        let u = find(
            &[
                raw("store", "p/B.java", 2, Scope::Project, true),
                raw("D", "p/B.java", 7, Scope::Project, false),
            ],
            &[f, other],
        );
        assert_eq!(u[0].writers, ["p/B.java:3"]);
        assert_eq!(u[0].removers, ["p/B.java:4"]);
        assert_eq!(u[1].writers, ["p/B.java:8"]);
        assert_eq!(u[1].removers, ["p/B.java:9"]);
    }

    #[test]
    fn go_field_wrapped_from_a_call_is_a_writer() {
        let f = file(
            "p/poller.go",
            "func (p *P) Lookup() {\n\tp.ctx = trace.Start(p.ctx, \"x\")\n\tq, err := p.src.Fetch(p.ctx, pair)\n}\nfunc (p *P) Where() string {\n\treturn trace.Path(p.ctx, 3)\n}\n",
        );
        let u = find(
            &[raw("ctx", "p/poller.go", 99, Scope::Dir("p".into()), true)],
            &[f],
        );
        assert_eq!(u[0].writers, ["p/poller.go:2"], "{u:?}");
        assert!(u[0].removers.is_empty(), "{u:?}");
    }

    #[test]
    fn an_element_assigned_a_made_container_is_a_writer_and_the_holder_reset_is_a_remover() {
        let f = file(
            "p/s.go",
            "func (s *S) Watch(k string) {\n\ts.updates[k] = make(map[int]int)\n}\nfunc (s *S) Drop() {\n\ts.updates = make(map[string]map[int]int)\n}\n",
        );
        let u = find(
            &[raw("updates", "p/s.go", 99, Scope::Dir("p".into()), true)],
            &[f],
        );
        assert_eq!(u[0].writers, ["p/s.go:2"], "{u:?}");
        assert_eq!(u[0].removers, ["p/s.go:5"], "{u:?}");
    }
}
