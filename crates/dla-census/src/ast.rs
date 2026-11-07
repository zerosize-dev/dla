//! Small helpers over tree-sitter nodes shared by the language modules.

use crate::parse::tree_sitter::Node;

/// Source text of a node.
pub fn text<'a>(n: Node<'_>, src: &'a str) -> &'a str {
    src.get(n.start_byte()..n.end_byte()).unwrap_or("")
}

/// 1-based start line of a node.
pub fn line(n: Node<'_>) -> u32 {
    u32::try_from(n.start_position().row.saturating_add(1)).unwrap_or(u32::MAX)
}

/// 1-based end line of a node.
pub fn end_line(n: Node<'_>) -> u32 {
    u32::try_from(n.end_position().row.saturating_add(1)).unwrap_or(u32::MAX)
}

/// Visits every node under `root` (inclusive) in document order, without recursion.
pub fn visit<'t>(root: Node<'t>, f: &mut dyn FnMut(Node<'t>)) {
    let mut cursor = root.walk();
    loop {
        f(cursor.node());
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.node() == root {
                return;
            }
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return;
            }
        }
    }
}

/// Named children of `n` with the given kind.
pub fn children_of<'t>(n: Node<'t>, kind: &str) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c)
        .filter(|ch| ch.kind() == kind)
        .collect()
}

/// First named child with the given kind.
pub fn child_of<'t>(n: Node<'t>, kind: &str) -> Option<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c).find(|ch| ch.kind() == kind)
}

/// Ancestors of `n`, nearest first.
pub fn ancestors<'t>(n: Node<'t>) -> impl Iterator<Item = Node<'t>> {
    std::iter::successors(n.parent(), |p| p.parent())
}

/// Text of a node's named field.
pub fn field_text<'a>(n: Node<'_>, field: &str, src: &'a str) -> Option<&'a str> {
    n.child_by_field_name(field).map(|c| text(c, src))
}

/// The last identifier-like word of `s` (`a.b.c` gives `c`).
pub fn last_word(s: &str) -> Option<&str> {
    let t = s.trim_end_matches(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'));
    let start = t
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
        .map_or(0, |i| i + 1);
    let w = &t[start..];
    (!w.is_empty()).then_some(w)
}

/// The first identifier-like word of `s`.
pub fn first_word(s: &str) -> Option<&str> {
    let t = s.trim_start_matches(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'));
    let end = t
        .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
        .unwrap_or(t.len());
    let w = &t[..end];
    (!w.is_empty()).then_some(w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        assert_eq!(last_word("a.b.cache"), Some("cache"));
        assert_eq!(last_word("self.items[0]"), Some("0"));
        assert_eq!(last_word("x, err"), Some("err"));
        assert_eq!(first_word("  *p, q"), Some("p"));
        assert_eq!(last_word("()"), None);
    }
}
