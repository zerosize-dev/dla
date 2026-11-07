//! Turns heading-marked text into a tree of sections.

use crate::section::Section;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    EmptyTitle { line: usize },
    Empty,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyTitle { line } => write!(f, "heading on line {line} has no title"),
            ParseError::Empty => write!(f, "document has no headings"),
        }
    }
}

pub struct Parsed {
    pub root: Rc<Section>,
    pub headings: usize,
    pub body_lines: usize,
}

/// Lines starting with one or more `#` are headings; everything else is body text.
pub fn parse(text: &str) -> Result<Parsed, ParseError> {
    let root = Section::new("document", 0, 0);
    let mut open: Vec<Rc<Section>> = vec![Rc::clone(&root)];
    let mut headings = 0;
    let mut body_lines = 0;

    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let level = raw.chars().take_while(|c| *c == '#').count();
        if level == 0 {
            if !raw.trim().is_empty() {
                body_lines += 1;
            }
            continue;
        }
        let title = raw[level..].trim();
        if title.is_empty() {
            return Err(ParseError::EmptyTitle { line: line_no });
        }
        while open.last().map_or(false, |top| top.level >= level) {
            open.pop();
        }
        let section = Section::new(title, level, line_no);
        let parent = open.last().expect("root stays open");
        Section::append(parent, Rc::clone(&section));
        open.push(section);
        headings += 1;
    }

    if headings == 0 {
        return Err(ParseError::Empty);
    }
    Ok(Parsed { root, headings, body_lines })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nests_by_heading_level() {
        let parsed = parse("# One\ntext\n## Two\n## Three\n# Four\n").unwrap();
        assert_eq!(parsed.headings, 4);
        assert_eq!(parsed.body_lines, 1);
        let top = parsed.root.children();
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].children().len(), 2);
    }

    #[test]
    fn rejects_blank_titles_and_empty_documents() {
        assert_eq!(parse("# \n").err(), Some(ParseError::EmptyTitle { line: 1 }));
        assert_eq!(parse("just words\n").err(), Some(ParseError::Empty));
    }
}
