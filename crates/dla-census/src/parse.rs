//! Tree-sitter parsing shared by the census, the DLA rule backend and workload discovery.
//!
//! [`Lang`] maps a file extension to a grammar, [`parse`] builds a syntax tree and
//! [`captures`] runs a tree-sitter query and returns its captures with 1-based lines.
//! The grammars are compiled into the binary (they contain C code), so nothing is
//! loaded at run time.

use std::path::Path;

use dla_core::runtime::Runtime;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator, Tree};

pub use tree_sitter;

/// A source language with a bundled grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lang {
    /// Java (`.java`).
    Java,
    /// Kotlin (`.kt`, `.kts`).
    Kotlin,
    /// Go (`.go`).
    Go,
    /// Rust (`.rs`).
    Rust,
    /// JavaScript (`.js`, `.mjs`, `.cjs`, `.jsx`).
    JavaScript,
    /// TypeScript (`.ts`, `.mts`, `.cts`).
    TypeScript,
    /// TypeScript with JSX (`.tsx`).
    Tsx,
    /// Python (`.py`).
    Python,
    /// C (`.c`, `.h`).
    C,
    /// C++ (`.cc`, `.cpp`, `.cxx`, `.hpp`, `.hh`, `.hxx`).
    Cpp,
}

impl Lang {
    /// Every language, in a fixed order.
    pub const ALL: [Lang; 10] = [
        Lang::Java,
        Lang::Kotlin,
        Lang::Go,
        Lang::Rust,
        Lang::JavaScript,
        Lang::TypeScript,
        Lang::Tsx,
        Lang::Python,
        Lang::C,
        Lang::Cpp,
    ];

    /// The language of a file extension (without the dot, case-insensitive).
    pub fn from_extension(ext: &str) -> Option<Lang> {
        let ext = ext.to_ascii_lowercase();
        Some(match ext.as_str() {
            "java" => Lang::Java,
            "kt" | "kts" => Lang::Kotlin,
            "go" => Lang::Go,
            "rs" => Lang::Rust,
            "js" | "mjs" | "cjs" | "jsx" => Lang::JavaScript,
            "ts" | "mts" | "cts" => Lang::TypeScript,
            "tsx" => Lang::Tsx,
            "py" => Lang::Python,
            "c" | "h" => Lang::C,
            "cc" | "cpp" | "cxx" | "hpp" | "hh" | "hxx" => Lang::Cpp,
            _ => return None,
        })
    }

    /// The language of a path, by its extension.
    pub fn from_path(path: &Path) -> Option<Lang> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Lang::from_extension)
    }

    /// Short lowercase name.
    pub fn name(self) -> &'static str {
        match self {
            Lang::Java => "java",
            Lang::Kotlin => "kotlin",
            Lang::Go => "go",
            Lang::Rust => "rust",
            Lang::JavaScript => "javascript",
            Lang::TypeScript => "typescript",
            Lang::Tsx => "tsx",
            Lang::Python => "python",
            Lang::C => "c",
            Lang::Cpp => "cpp",
        }
    }

    /// The runtime (worker) this language belongs to.
    pub fn runtime(self) -> Runtime {
        match self {
            Lang::Java | Lang::Kotlin => Runtime::Jvm,
            Lang::Go => Runtime::Go,
            Lang::Rust => Runtime::Rust,
            Lang::JavaScript | Lang::TypeScript | Lang::Tsx => Runtime::Node,
            Lang::Python => Runtime::Python,
            Lang::C | Lang::Cpp => Runtime::Native,
        }
    }

    /// The languages whose sources belong to a runtime.
    pub fn of_runtime(runtime: Runtime) -> Vec<Lang> {
        Lang::ALL
            .into_iter()
            .filter(|l| l.runtime() == runtime)
            .collect()
    }

    /// The tree-sitter grammar.
    pub fn language(self) -> Language {
        match self {
            Lang::Java => tree_sitter_java::LANGUAGE.into(),
            Lang::Kotlin => tree_sitter_kotlin_ng::LANGUAGE.into(),
            Lang::Go => tree_sitter_go::LANGUAGE.into(),
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
            Lang::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Lang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Lang::Python => tree_sitter_python::LANGUAGE.into(),
            Lang::C => tree_sitter_c::LANGUAGE.into(),
            Lang::Cpp => tree_sitter_cpp::LANGUAGE.into(),
        }
    }
}

/// Why parsing or querying failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// The grammar does not match the tree-sitter library version.
    #[error("grammar for {0} cannot be loaded: {1}")]
    Grammar(&'static str, String),
    /// The parser gave up (cancelled or timed out).
    #[error("tree-sitter could not parse the {0} source")]
    Failed(&'static str),
    /// The query text is not valid for the grammar.
    #[error("invalid {lang} query: {message}")]
    Query {
        /// Language of the query.
        lang: &'static str,
        /// The tree-sitter message, with the offset.
        message: String,
    },
}

/// Parses `source`. Syntax errors do not fail: the tree holds error nodes.
pub fn parse(lang: Lang, source: &str) -> Result<Tree, ParseError> {
    let mut parser = Parser::new();
    parser
        .set_language(&lang.language())
        .map_err(|e| ParseError::Grammar(lang.name(), e.to_string()))?;
    parser
        .parse(source, None)
        .ok_or(ParseError::Failed(lang.name()))
}

/// One query capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// Capture name without the `@`.
    pub name: String,
    /// Source text of the node.
    pub text: String,
    /// Node kind (grammar symbol name).
    pub kind: String,
    /// 1-based line where the node starts.
    pub line: u32,
    /// 1-based column (byte offset in the line plus one) where the node starts.
    pub column: u32,
    /// 1-based line where the node ends.
    pub end_line: u32,
    /// Index of the match this capture belongs to (captures of one match share it).
    pub match_index: usize,
}

fn line_of(row: usize) -> u32 {
    u32::try_from(row.saturating_add(1)).unwrap_or(u32::MAX)
}

/// Runs a tree-sitter `query` over `tree` and returns every capture in document order
/// of the matches. `source` must be the text the tree was parsed from.
pub fn captures(
    lang: Lang,
    tree: &Tree,
    source: &str,
    query: &str,
) -> Result<Vec<Capture>, ParseError> {
    let q = Query::new(&lang.language(), query).map_err(|e| ParseError::Query {
        lang: lang.name(),
        message: e.to_string(),
    })?;
    let names = q.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&q, tree.root_node(), source.as_bytes());
    let mut out = Vec::new();
    let mut index = 0usize;
    while let Some(m) = matches.next() {
        for c in m.captures {
            let node = c.node;
            let start = node.start_position();
            out.push(Capture {
                name: names
                    .get(c.index as usize)
                    .map(|s| (*s).to_owned())
                    .unwrap_or_default(),
                text: source
                    .get(node.start_byte()..node.end_byte())
                    .unwrap_or_default()
                    .to_owned(),
                kind: node.kind().to_owned(),
                line: line_of(start.row),
                column: u32::try_from(start.column.saturating_add(1)).unwrap_or(u32::MAX),
                end_line: line_of(node.end_position().row),
                match_index: index,
            });
        }
        index += 1;
    }
    Ok(out)
}

/// Parses `source` and runs `query` in one call.
pub fn parse_and_capture(
    lang: Lang,
    source: &str,
    query: &str,
) -> Result<Vec<Capture>, ParseError> {
    let tree = parse(lang, source)?;
    captures(lang, &tree, source, query)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_capture(lang: Lang, src: &str, query: &str) -> Capture {
        let caps = parse_and_capture(lang, src, query).unwrap();
        assert!(!caps.is_empty(), "no capture for {}", lang.name());
        caps[0].clone()
    }

    #[test]
    fn extensions_map_to_languages() {
        for (ext, lang) in [
            ("java", Lang::Java),
            ("kt", Lang::Kotlin),
            ("kts", Lang::Kotlin),
            ("go", Lang::Go),
            ("rs", Lang::Rust),
            ("js", Lang::JavaScript),
            ("mjs", Lang::JavaScript),
            ("ts", Lang::TypeScript),
            ("tsx", Lang::Tsx),
            ("PY", Lang::Python),
            ("c", Lang::C),
            ("h", Lang::C),
            ("cpp", Lang::Cpp),
            ("hpp", Lang::Cpp),
        ] {
            assert_eq!(Lang::from_extension(ext), Some(lang), "{ext}");
        }
        assert_eq!(Lang::from_extension("txt"), None);
        assert_eq!(
            Lang::from_path(Path::new("a/b/Main.java")),
            Some(Lang::Java)
        );
        assert_eq!(Lang::from_path(Path::new("Makefile")), None);
    }

    #[test]
    fn runtimes_group_languages() {
        assert_eq!(Lang::of_runtime(Runtime::Jvm), [Lang::Java, Lang::Kotlin]);
        assert_eq!(Lang::of_runtime(Runtime::Native), [Lang::C, Lang::Cpp]);
        assert_eq!(Lang::Tsx.runtime(), Runtime::Node);
    }

    #[test]
    fn java() {
        let c = first_capture(
            Lang::Java,
            "class A {\n  static int x;\n  void f() {}\n}\n",
            "(method_declaration name: (identifier) @m)",
        );
        assert_eq!((c.name.as_str(), c.text.as_str(), c.line), ("m", "f", 3));
    }

    #[test]
    fn kotlin() {
        let c = first_capture(
            Lang::Kotlin,
            "object Registry {\n    val items = mutableListOf<String>()\n}\n",
            "(object_declaration (identifier) @o)",
        );
        assert_eq!((c.text.as_str(), c.line), ("Registry", 1));
    }

    #[test]
    fn go() {
        let c = first_capture(
            Lang::Go,
            "package main\n\nfunc main() {\n\tgo run()\n}\n",
            "(go_statement) @g",
        );
        assert_eq!((c.text.as_str(), c.line), ("go run()", 4));
    }

    #[test]
    fn rust() {
        let c = first_capture(
            Lang::Rust,
            "fn a() {}\n\nstatic CACHE: u8 = 0;\n",
            "(static_item name: (identifier) @s)",
        );
        assert_eq!((c.text.as_str(), c.line), ("CACHE", 3));
    }

    #[test]
    fn javascript() {
        let c = first_capture(
            Lang::JavaScript,
            "const a = 1;\nsetInterval(() => {}, 10);\n",
            "(call_expression function: (identifier) @f)",
        );
        assert_eq!((c.text.as_str(), c.line), ("setInterval", 2));
    }

    #[test]
    fn typescript_and_tsx() {
        let c = first_capture(
            Lang::TypeScript,
            "interface I { a: number }\nconst x: I = { a: 1 };\n",
            "(interface_declaration name: (type_identifier) @i)",
        );
        assert_eq!((c.text.as_str(), c.line), ("I", 1));
        let c = first_capture(
            Lang::Tsx,
            "const el = <div id=\"a\">hi</div>;\n",
            "(jsx_element) @e",
        );
        assert_eq!(c.line, 1);
    }

    #[test]
    fn python() {
        let c = first_capture(
            Lang::Python,
            "import os\n\n\ndef handler():\n    pass\n",
            "(function_definition name: (identifier) @f)",
        );
        assert_eq!((c.text.as_str(), c.line), ("handler", 4));
    }

    #[test]
    fn c_and_cpp() {
        let c = first_capture(
            Lang::C,
            "#include <stdlib.h>\nint main(void) {\n  char *p = malloc(4);\n  return 0;\n}\n",
            "(call_expression function: (identifier) @f)",
        );
        assert_eq!((c.text.as_str(), c.line), ("malloc", 3));
        let c = first_capture(
            Lang::Cpp,
            "class A {\npublic:\n  int* p = new int(3);\n};\n",
            "(new_expression) @n",
        );
        assert_eq!(c.line, 3);
    }

    #[test]
    fn captures_of_one_match_share_an_index() {
        let caps = parse_and_capture(
            Lang::Python,
            "def a():\n    pass\n\ndef b():\n    pass\n",
            "(function_definition name: (identifier) @name body: (block) @body)",
        )
        .unwrap();
        assert_eq!(caps.len(), 4);
        assert_eq!(caps[0].match_index, caps[1].match_index);
        assert_ne!(caps[1].match_index, caps[2].match_index);
        assert_eq!(caps[3].end_line, 5);
    }

    #[test]
    fn bad_query_is_an_error_and_syntax_errors_still_parse() {
        let tree = parse(Lang::Go, "package main\nfunc (").unwrap();
        assert!(tree.root_node().has_error());
        let err = captures(Lang::Go, &tree, "package main\nfunc (", "(no_such_node) @x");
        assert!(matches!(err, Err(ParseError::Query { .. })));
    }
}
