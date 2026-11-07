//! Renders a numbered table of contents from a section tree.

use crate::section::Section;
use std::rc::Rc;

/// One line per section: `1.2 Title (line 14)`; deep sections also name where they sit.
pub fn render(root: &Rc<Section>) -> String {
    let mut out = String::new();
    walk(root, &mut Vec::new(), &mut out);
    out
}

fn walk(section: &Rc<Section>, numbers: &mut Vec<usize>, out: &mut String) {
    for (i, child) in section.children().iter().enumerate() {
        numbers.push(i + 1);
        let label: Vec<String> = numbers.iter().map(|n| n.to_string()).collect();
        out.push_str(&label.join("."));
        out.push(' ');
        out.push_str(&child.title);
        out.push_str(&format!(" (line {})", child.line));
        if child.level >= 3 {
            let trail = child.breadcrumb();
            out.push_str(&format!(" in {}", trail[..trail.len() - 1].join(" > ")));
        }
        out.push('\n');
        walk(child, numbers, out);
        numbers.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn numbers_nested_sections() {
        let parsed = parse("# A\n## B\n## C\n# D\n").unwrap();
        let toc = render(&parsed.root);
        let lines: Vec<&str> = toc.lines().collect();
        assert_eq!(lines[0], "1 A (line 1)");
        assert_eq!(lines[2], "1.2 C (line 3)");
        assert_eq!(lines[3], "2 D (line 4)");
    }

    #[test]
    fn deep_sections_name_their_ancestors() {
        let parsed = parse("# A\n## B\n### C\n").unwrap();
        assert!(render(&parsed.root).contains("1.1.1 C (line 3) in A > B"));
    }
}
