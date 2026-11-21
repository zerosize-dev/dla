//! What one statement does to a named holder: the reader of writer and remover sites.
//!
//! The text it reads has its comments blanked and its string literals reduced to `"sss"` by
//! the syntax tree (see [`crate::sites`]), so a name inside a string or a comment never
//! counts. From there it follows the postfix steps after the holder (`[k]`, `.get(k)`) to
//! tell an operation on the holder itself (depth 0) from one on an element of it (depth 1
//! and up), and reads the verb or the assignment that follows.

use std::sync::OnceLock;

use regex::Regex;

use crate::usage::{R_EXACT, R_PREFIX, W_EXACT, W_PREFIX, empty_rhs};

/// What a statement does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ev {
    /// Adds an entry or element.
    Grow,
    /// Removes an entry or element.
    Remove,
    /// Assigns a fresh empty value to the holder.
    Reset,
    /// Assigns another value to the holder itself.
    Reassign,
}

/// One event on the holder in one statement.
#[derive(Debug, Clone)]
pub struct Event {
    /// What happens.
    pub kind: Ev,
    /// 0: on the holder, 1 and more: on an element of it.
    pub depth: u8,
    /// The text of the call arguments or of the index.
    pub args: String,
    /// The right-hand side of an assignment.
    pub rhs: String,
}

/// Verbs that read an element out of the holder and so deepen what follows.
const GET_VERBS: &[&str] = &[
    "get",
    "at",
    "getOrDefault",
    "Load",
    "get_mut",
    "entry",
    "or_default",
    "or_insert_with",
    "or_insert",
    "borrow_mut",
    "borrow",
    "lock",
    "unwrap",
    "expect",
];

fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).expect("static pattern"))
}

fn w_verb(v: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, &format!(r"^(?:(?:{W_PREFIX})\w*|(?:{W_EXACT}))$")).is_match(v)
}

fn r_verb(v: &str) -> bool {
    static R: OnceLock<Regex> = OnceLock::new();
    re(&R, &format!(r"^(?:(?:{R_PREFIX})\w*|(?:{R_EXACT}))$")).is_match(v)
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// The text between the bracket at `start` and its match, and the index after the match.
fn balanced(text: &str, start: usize, open: u8, close: u8) -> (&str, usize) {
    let b = text.as_bytes();
    let mut d = 0usize;
    for (i, &c) in b.iter().enumerate().skip(start) {
        if c == open {
            d += 1;
        } else if c == close {
            d = d.saturating_sub(1);
            if d == 0 {
                return (&text[start + 1..i], i + 1);
            }
        }
    }
    (text.get(start + 1..).unwrap_or(""), text.len())
}

/// The `[..]` and `.get(..)` steps after a holder name: how many, and where they end.
fn postfix(text: &str, pos: usize) -> (Vec<(bool, String)>, usize) {
    static CALL: OnceLock<Regex> = OnceLock::new();
    let call = re(&CALL, r"^\s*(?:\?\.|\.)\s*([A-Za-z_$][\w$]*)\s*\(");
    let mut steps = Vec::new();
    let mut i = pos;
    loop {
        let rest = &text[i..];
        let j = i + (rest.len() - rest.trim_start_matches(' ').len());
        if text.as_bytes().get(j) == Some(&b'[') {
            let (inner, e) = balanced(text, j, b'[', b']');
            steps.push((true, inner.to_owned()));
            i = e;
            continue;
        }
        if let Some(m) = call.captures(&text[j..])
            && GET_VERBS.contains(&&m[1])
        {
            let open = j + m.get(0).map_or(0, |g| g.end()) - 1;
            let (inner, e) = balanced(text, open, b'(', b')');
            steps.push((false, inner.to_owned()));
            i = e;
            continue;
        }
        return (steps, i);
    }
}

fn depth_of(n: usize) -> u8 {
    u8::try_from(n).unwrap_or(u8::MAX)
}

/// Every event on `name` in `line` (one logical statement, comments and strings blanked).
pub fn events(line: &str, name: &str) -> Vec<Event> {
    static METHOD: OnceLock<Regex> = OnceLock::new();
    static FNSTYLE: OnceLock<Regex> = OnceLock::new();
    static ASSIGN: OnceLock<Regex> = OnceLock::new();
    static SEND: OnceLock<Regex> = OnceLock::new();
    let method = re(
        &METHOD,
        r"^\s*(?:\?\.|\.|->|::)\s*([A-Za-z_$][\w$]*)\s*(?:\(|<)",
    );
    let fnstyle = re(
        &FNSTYLE,
        r"\b(append|delete|close|free|clearTimeout|clearInterval|clearImmediate|del)\b\s*\(?\s*[\w.*&\[\]]*$",
    );
    let assign = re(&ASSIGN, r"^\s*(?:\?\.|\.)?\s*([-+*|&^]?)=([^=>~].*|)$");
    let send = re(&SEND, r"^\s*<-");
    static RECV: OnceLock<Regex> = OnceLock::new();
    let recv = re(&RECV, r"<-\s*[\w.*&]*$");
    let own_append = Regex::new(&format!(
        r"^(?:append\(|.*\bappend\(\s*{n}\b|{n}\s*[+|]|.*\+\s*{n}\b)",
        n = regex::escape(name)
    ))
    .ok();
    let mut out = Vec::new();
    for (at, _) in line.match_indices(name) {
        let b = line.as_bytes();
        if at > 0 && is_ident(b[at - 1]) {
            continue;
        }
        let end = at + name.len();
        if end < b.len() && is_ident(b[end]) {
            continue;
        }
        let (steps, p) = postfix(line, end);
        let depth = steps.len();
        let rest = &line[p..];
        let pre = &line[..at];
        if let Some(m) = method.captures(rest) {
            let verb = &m[1];
            let open = m.get(0).map_or(0, |g| g.end()) - 1;
            let args = if rest.as_bytes().get(open) == Some(&b'(') {
                balanced(rest, open, b'(', b')').0.to_owned()
            } else {
                String::new()
            };
            let kind = if r_verb(verb) {
                Some(Ev::Remove)
            } else if w_verb(verb) {
                Some(Ev::Grow)
            } else {
                None
            };
            if let Some(kind) = kind {
                out.push(Event {
                    kind,
                    depth: depth_of(depth),
                    args,
                    rhs: String::new(),
                });
            }
            continue;
        }
        // `v := <-s.jobs`: a receive takes an element out of a channel holder. The channel of a
        // timer (`<-tick.C`) is read, not emptied: the name is followed by a field.
        if recv.is_match(pre) && !rest.trim_start().starts_with(['.', '[']) {
            out.push(Event {
                kind: Ev::Remove,
                depth: 0,
                args: String::new(),
                rhs: String::new(),
            });
            continue;
        }
        if send.is_match(rest) {
            out.push(Event {
                kind: Ev::Grow,
                depth: 0,
                args: rest.get(2..).unwrap_or("").to_owned(),
                rhs: String::new(),
            });
            continue;
        }
        let asg = assign.captures(rest);
        if let Some(fm) = fnstyle.captures(pre)
            && asg.is_none()
        {
            let verb = &fm[1];
            if verb == "append" {
                out.push(Event {
                    kind: Ev::Grow,
                    depth: depth_of(depth),
                    args: rest.to_owned(),
                    rhs: String::new(),
                });
            } else {
                let d = if verb == "del" {
                    depth.saturating_sub(1)
                } else {
                    depth
                };
                out.push(Event {
                    kind: Ev::Remove,
                    depth: depth_of(d),
                    args: rest.to_owned(),
                    rhs: String::new(),
                });
            }
            continue;
        }
        if let Some(c) = asg {
            let op = c.get(1).map_or("", |m| m.as_str());
            let rhs = c.get(2).map_or("", |m| m.as_str()).trim().to_owned();
            if let Some((true, idx)) = steps.last() {
                if op == "-" {
                    out.push(Event {
                        kind: Ev::Remove,
                        depth: depth_of(depth - 1),
                        args: idx.clone(),
                        rhs,
                    });
                } else {
                    out.push(Event {
                        kind: Ev::Grow,
                        depth: depth_of(depth - 1),
                        args: idx.clone(),
                        rhs,
                    });
                }
            } else if depth == 0 {
                // `x = wrap(x)`, `x = {next: x}`: the new value holds the old one, a chain
                // that can grow.
                let own = holds_itself(&rhs, name)
                    || own_append.as_ref().is_some_and(|r| r.is_match(&rhs));
                let kind = if op == "-" || empty_rhs().is_match(&rhs) {
                    Ev::Reset
                } else if own || op == "+" || op == "|" {
                    Ev::Grow
                } else {
                    Ev::Reassign
                };
                out.push(Event {
                    kind,
                    depth: 0,
                    args: if kind == Ev::Grow {
                        rhs.clone()
                    } else {
                        String::new()
                    },
                    rhs,
                });
            }
        }
    }
    out
}

/// True when `value` holds the holder's own value as one of the parts it is built from: the
/// name stands as an argument, a list element or a field value (`wrap(x)`, `Node(v, x)`,
/// `{next: x}`), not as the thing read from (`x.next`, `x[1:]`) or computed on (`x + 1`).
pub fn holds_itself(value: &str, name: &str) -> bool {
    let value = value.trim().trim_end_matches(';').trim();
    if value == name {
        return false;
    }
    let b = value.as_bytes();
    value.match_indices(name).any(|(at, _)| {
        let end = at + name.len();
        if (at > 0 && is_ident(b[at - 1])) || (end < b.len() && is_ident(b[end])) {
            return false;
        }
        matches!(
            value[end..].trim_start().bytes().next(),
            Some(b')' | b',' | b'}' | b']') | None
        )
    })
}

/// True when `value` is the holder's own value or holds it (see [`holds_itself`]): what a
/// node's link field is set to when the node is about to become the new head.
pub fn links_to(value: &str, name: &str) -> bool {
    let v = value.trim().trim_end_matches(';').trim();
    let tail = v.rsplit(['.', '>']).next().unwrap_or(v);
    tail == name || holds_itself(value, name)
}

/// The identifiers of a text, without keywords and string literals.
pub fn idents(text: &str) -> Vec<String> {
    static STR: OnceLock<Regex> = OnceLock::new();
    static ID: OnceLock<Regex> = OnceLock::new();
    let strs = re(&STR, r#""[^"]*"|'[^']*'|`[^`]*`"#);
    let id = re(&ID, r"[A-Za-z_$][\w$]*");
    let t = strs.replace_all(text, "\"\"");
    id.find_iter(&t)
        .map(|m| m.as_str())
        .filter(|w| !KEYWORDS.contains(w))
        .map(str::to_owned)
        .collect()
}

const KEYWORDS: &[&str] = &[
    "if",
    "else",
    "for",
    "while",
    "switch",
    "case",
    "return",
    "new",
    "var",
    "let",
    "const",
    "final",
    "static",
    "public",
    "private",
    "protected",
    "class",
    "interface",
    "func",
    "def",
    "self",
    "cls",
    "this",
    "super",
    "null",
    "nil",
    "None",
    "True",
    "False",
    "true",
    "false",
    "int",
    "long",
    "string",
    "String",
    "bool",
    "boolean",
    "byte",
    "char",
    "float",
    "double",
    "void",
    "in",
    "is",
    "not",
    "and",
    "or",
    "await",
    "async",
    "try",
    "catch",
    "finally",
    "throw",
    "throws",
    "import",
    "from",
    "as",
    "with",
    "yield",
    "go",
    "defer",
    "chan",
    "map",
    "range",
    "type",
    "struct",
    "package",
    "fmt",
    "make",
    "len",
    "cap",
    "append",
    "delete",
    "del",
    "typeof",
    "instanceof",
    "function",
    "export",
    "extends",
    "implements",
    "break",
    "continue",
    "do",
    "goto",
    "select",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn one(line: &str, name: &str) -> Vec<(Ev, u8)> {
        events(line, name)
            .iter()
            .map(|e| (e.kind, e.depth))
            .collect()
    }

    #[test]
    fn element_writes_and_deletes_have_a_depth() {
        assert_eq!(one("s.updates[k][x] = v", "updates"), [(Ev::Grow, 1)]);
        assert_eq!(one("s.updates[k] = v", "updates"), [(Ev::Grow, 0)]);
        assert_eq!(one("delete(s.updates[k], x)", "updates"), [(Ev::Remove, 1)]);
        assert_eq!(one("delete(s.updates, k)", "updates"), [(Ev::Remove, 0)]);
        assert_eq!(one("del self.m[k]", "m"), [(Ev::Remove, 0)]);
        assert_eq!(one("d[k].append(x)", "d"), [(Ev::Grow, 1)]);
        assert_eq!(one("d.pop(k)", "d"), [(Ev::Remove, 0)]);
    }

    #[test]
    fn a_made_container_stored_under_a_key_is_an_insert() {
        assert_eq!(one("m[k] = make(chan int)", "m"), [(Ev::Grow, 0)]);
        assert_eq!(one("m = make(map[string]int)", "m"), [(Ev::Reset, 0)]);
        assert_eq!(one("m = other", "m"), [(Ev::Reassign, 0)]);
        assert_eq!(one("m = append(m, x)", "m").first(), Some(&(Ev::Grow, 0)));
        assert_eq!(
            one("this.items = new ArrayList<>()", "items"),
            [(Ev::Reset, 0)]
        );
    }

    #[test]
    fn a_name_inside_another_word_is_not_the_holder() {
        assert!(one("cache2.put(k, v)", "cache").is_empty());
        assert!(one("mycache.put(k, v)", "cache").is_empty());
        assert_eq!(one("self.cache.put(k, v)", "cache"), [(Ev::Grow, 0)]);
    }

    #[test]
    fn a_value_built_from_the_holder_is_a_chain_and_a_value_read_from_it_is_not() {
        assert_eq!(one("head = Node(v, head)", "head"), [(Ev::Grow, 0)]);
        assert_eq!(one("p.ctx = trace.Start(p.ctx, s)", "ctx"), [(Ev::Grow, 0)]);
        assert_eq!(one("h = {next: h}", "h"), [(Ev::Grow, 0)]);
        assert_eq!(one("head = head.next", "head"), [(Ev::Reassign, 0)]);
        assert_eq!(one("x = x[1:]", "x"), [(Ev::Reassign, 0)]);
        assert_eq!(one("n = n * 2", "n"), [(Ev::Reassign, 0)]);
        assert!(holds_itself("wrap(a, a)", "a"));
        assert!(!holds_itself("a", "a"));
    }

    #[test]
    fn a_receive_empties_a_channel_holder_and_reading_a_timer_channel_does_not() {
        assert_eq!(one("v := <-s.jobs", "jobs"), [(Ev::Remove, 0)]);
        assert_eq!(one("s.jobs <- v", "jobs"), [(Ev::Grow, 0)]);
        assert!(one("case <-tick.C:", "tick").is_empty());
    }
}
