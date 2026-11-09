//! A small tolerant XML reader for tool output (Valgrind). It builds a tree with an explicit
//! stack (no recursion), ignores attributes, decodes the standard entities and accepts a
//! document that stops early (a killed process), which it reports.

use super::ParseError;

const MAX_DEPTH: usize = 64;

#[derive(Debug, Default, Clone)]
pub(crate) struct Node {
    pub name: String,
    /// The character data directly inside the element, trimmed.
    pub text: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// Text of the first child named `name`.
    pub fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).map(|c| c.text.as_str())
    }
}

pub(crate) struct Document {
    /// A virtual node holding the top-level elements.
    pub root: Node,
    /// The input ended while elements were still open.
    pub truncated: bool,
}

pub(crate) fn parse(input: &str) -> Result<Document, ParseError> {
    let b = input.as_bytes();
    let mut stack: Vec<Node> = vec![Node::default()];
    let mut text = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'<' {
            let end = input[i..].find('<').map_or(b.len(), |p| i + p);
            text.push_str(&input[i..end]);
            i = end;
            continue;
        }
        let rest = &input[i..];
        if rest.starts_with("<!--") {
            i += rest.find("-->").map_or(rest.len(), |p| p + 3);
        } else if let Some(body) = rest.strip_prefix("<![CDATA[") {
            let end = body.find("]]>");
            text.push_str(&body[..end.unwrap_or(body.len())]);
            i += 9 + end.map_or(body.len(), |p| p + 3);
        } else if rest.starts_with("<?") || rest.starts_with("<!") {
            i += rest.find('>').map_or(rest.len(), |p| p + 1);
        } else if let Some(close) = rest.strip_prefix("</") {
            let Some(end) = close.find('>') else {
                i = b.len();
                continue;
            };
            let name = close[..end].trim();
            i += 2 + end + 1;
            flush_text(&mut stack, &mut text);
            // Pop to the matching open element; an unmatched close tag is ignored.
            if let Some(depth) = stack
                .iter()
                .rposition(|n| n.name == name)
                .filter(|&d| d > 0)
            {
                while stack.len() > depth {
                    close_top(&mut stack);
                }
            }
        } else {
            // A start tag. Quotes in attribute values may contain '>'.
            let mut j = 1;
            let mut quote = 0u8;
            let mut found = None;
            let tag = rest.as_bytes();
            while j < tag.len() {
                let c = tag[j];
                if quote != 0 {
                    if c == quote {
                        quote = 0;
                    }
                } else if c == b'"' || c == b'\'' {
                    quote = c;
                } else if c == b'>' {
                    found = Some(j);
                    break;
                }
                j += 1;
            }
            let Some(end) = found else {
                // A tag cut off by the end of input.
                i = b.len();
                continue;
            };
            let inner = &rest[1..end];
            let self_closing = inner.ends_with('/');
            let inner = inner.strip_suffix('/').unwrap_or(inner);
            let name = inner
                .split(|c: char| c.is_whitespace())
                .next()
                .unwrap_or("")
                .to_owned();
            i += end + 1;
            if name.is_empty() {
                return Err(ParseError::new(format!(
                    "malformed XML: empty element name near byte {i}"
                )));
            }
            flush_text(&mut stack, &mut text);
            if stack.len() > MAX_DEPTH {
                return Err(ParseError::new(format!(
                    "malformed XML: elements nested deeper than {MAX_DEPTH}"
                )));
            }
            stack.push(Node {
                name,
                ..Node::default()
            });
            if self_closing {
                close_top(&mut stack);
            }
        }
    }
    flush_text(&mut stack, &mut text);
    let truncated = stack.len() > 1;
    while stack.len() > 1 {
        close_top(&mut stack);
    }
    let root = stack.pop().unwrap_or_default();
    Ok(Document { root, truncated })
}

fn flush_text(stack: &mut [Node], text: &mut String) {
    if text.is_empty() {
        return;
    }
    if let Some(top) = stack.last_mut() {
        let t = unescape(text);
        let t = t.trim();
        if !t.is_empty() {
            if !top.text.is_empty() {
                top.text.push(' ');
            }
            top.text.push_str(t);
        }
    }
    text.clear();
}

fn close_top(stack: &mut Vec<Node>) {
    if stack.len() > 1
        && let Some(done) = stack.pop()
        && let Some(parent) = stack.last_mut()
    {
        parent.children.push(done);
    }
}

/// Decodes `&lt; &gt; &amp; &quot; &apos;` and numeric references. Unknown entities stay.
pub(crate) fn unescape(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains('&') {
        return s.into();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(p) = rest.find('&') {
        out.push_str(&rest[..p]);
        rest = &rest[p..];
        // Look only at the first 11 bytes so a run of bare `&` stays linear.
        let Some(end) = rest.as_bytes().iter().take(11).position(|&b| b == b';') else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let name = &rest[1..end];
        let decoded = match name {
            "lt" => Some('<'),
            "gt" => Some('>'),
            "amp" => Some('&'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => name
                .strip_prefix("#x")
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| name.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_tree_and_decodes_entities() {
        let d =
            parse("<?xml version=\"1.0\"?><a><b>x &lt; y</b><c k=\"1>2\"/><b>2</b></a>").unwrap();
        let a = d.root.child("a").unwrap();
        assert_eq!(a.children_named("b").count(), 2);
        assert_eq!(a.child_text("b"), Some("x < y"));
        assert!(a.child("c").is_some());
        assert!(!d.truncated);
    }

    #[test]
    fn a_cut_document_is_reported() {
        let d = parse("<a><b>1</b><c><d>par").unwrap();
        assert!(d.truncated);
        assert_eq!(d.root.child("a").unwrap().child_text("b"), Some("1"));
        let d = parse("<a><b").unwrap();
        assert!(d.truncated);
    }

    #[test]
    fn nesting_is_bounded() {
        let deep = "<a>".repeat(1000);
        assert!(parse(&deep).is_err());
    }

    #[test]
    fn stray_close_tags_and_cdata() {
        let d = parse("</x><a><![CDATA[<raw>]]></a></zz>").unwrap();
        assert_eq!(d.root.child("a").unwrap().text, "<raw>");
        assert_eq!(unescape("a&#65;&#x42;&bogus;&"), "aAB&bogus;&");
    }

    #[test]
    fn many_bare_ampersands_stay_linear() {
        let big = "&".repeat(2_000_000);
        let t = std::time::Instant::now();
        assert_eq!(unescape(&big), big.as_str());
        let mixed = format!("{big};x");
        assert_eq!(unescape(&mixed), mixed.as_str());
        assert!(
            t.elapsed() < std::time::Duration::from_secs(2),
            "{:?}",
            t.elapsed()
        );
    }
}
