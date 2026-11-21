//! Which lines of a source file are one site.
//!
//! A DLA rule reports a cache on the decorator line (`@lru_cache(None)`), the census on the
//! `def` line, or the other way round, and a stack of decorators spans several lines. Both
//! name one declaration, so the candidate writer treats the lines of a decorated function or
//! class header, from the first decorator to the `def` or `class` line, as one site anchored at
//! that last line. Every other line is its own site.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use dla_core::toolctx::SiteAnchor;

use crate::{
    ast::{line as node_line, visit},
    parse::{Lang, parse},
};

/// The header spans `(first line, definition line)` of the decorated definitions of a Python
/// source, outermost first.
pub fn python_header_spans(src: &str) -> Vec<(u32, u32)> {
    let Ok(tree) = parse(Lang::Python, src) else {
        return Vec::new();
    };
    let mut spans = Vec::new();
    visit(tree.root_node(), &mut |n| {
        if n.kind() == "decorated_definition"
            && let Some(def) = n.child_by_field_name("definition")
        {
            spans.push((node_line(n), node_line(def)));
        }
    });
    spans
}

/// The anchor line of `line` among `spans`: the definition line of the span holding it.
pub fn anchor_in(spans: &[(u32, u32)], line: u32) -> u32 {
    spans
        .iter()
        .find(|(first, def)| (*first..=*def).contains(&line))
        .map_or(line, |(_, def)| *def)
}

/// An anchor reading the sources under `project` (parsed once per file, only the Python ones).
pub fn site_anchor(project: &Path) -> SiteAnchor {
    let root: PathBuf = project.to_path_buf();
    let cache: Mutex<HashMap<String, Vec<(u32, u32)>>> = Mutex::new(HashMap::new());
    SiteAnchor::new(move |file, line| {
        if !Path::new(file)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("py"))
        {
            return line;
        }
        let Ok(mut cache) = cache.lock() else {
            return line;
        };
        let spans = cache.entry(file.to_owned()).or_insert_with(|| {
            fs::read_to_string(root.join(file))
                .map(|src| python_header_spans(&src))
                .unwrap_or_default()
        });
        anchor_in(spans, line)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_decorator_stack_and_its_def_are_one_anchor() {
        let src = "import functools\n\n@functools.lru_cache(None)\n@other\ndef f(\n    a,\n):\n    return a\n\nX = {}\n";
        let spans = python_header_spans(src);
        assert_eq!(spans, vec![(3, 5)]);
        for l in [3, 4, 5] {
            assert_eq!(anchor_in(&spans, l), 5, "line {l}");
        }
        assert_eq!(
            anchor_in(&spans, 6),
            6,
            "an argument line is not the header"
        );
        assert_eq!(anchor_in(&spans, 8), 8);
        assert_eq!(anchor_in(&spans, 10), 10);
    }

    #[test]
    fn a_decorated_class_and_a_method_inside_it_are_separate_sites() {
        let src = "@dataclass\nclass A:\n    @cache\n    def m(self):\n        pass\n";
        let spans = python_header_spans(src);
        assert_eq!(anchor_in(&spans, 1), 2);
        assert_eq!(anchor_in(&spans, 3), 4);
        assert_ne!(anchor_in(&spans, 1), anchor_in(&spans, 3));
    }

    #[test]
    fn other_languages_and_unreadable_files_keep_their_lines() {
        let a = site_anchor(Path::new("/nonexistent"));
        assert_eq!(a.of("a.go", 3), 3);
        assert_eq!(a.of("missing.py", 3), 3);
        let spans = python_header_spans("def broken(:\n");
        assert_eq!(anchor_in(&spans, 1), 1);
    }
}
