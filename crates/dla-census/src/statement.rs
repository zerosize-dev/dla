//! Statement spans for `dla intervene --stub-writer`.
//!
//! [`statement_at`] answers one question: which byte range of `src` is the standalone
//! expression, call or assignment statement at a line? "Standalone" means the statement
//! sits directly in a block (or file, or case body) and its value is not used: a call
//! `items.add(x);`, an assignment `count += 1`, `m[k] = v`. A declaration with an
//! initializer, a `return`, a condition, a loop, a lambda body that is an expression and a
//! tail expression are not, and the answer is `None` for them.
//!
//! [`stub_statement`] gives the same answer with the reason it refuses and the text that
//! replaces the statement: the language's no-op, followed by as many newlines as the
//! statement had, so no other line moves.

use std::ops::Range;

use tree_sitter::Node;

use crate::parse::{Lang, parse};

/// Node kinds whose children are statements, in any of the supported grammars.
const CONTAINERS: [&str; 17] = [
    "block",
    "statement_block",
    "compound_statement",
    "statement_list",
    "statements",
    "source_file",
    "program",
    "module",
    "translation_unit",
    "switch_block_statement_group",
    "switch_rule",
    "constructor_body",
    "case_statement",
    "switch_case",
    "switch_default",
    "expression_case",
    "default_case",
];

/// Comment node kinds, never statements.
fn is_comment(kind: &str) -> bool {
    kind.contains("comment")
}

/// Why a line has no stub-able statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementError {
    /// The source could not be parsed.
    Unparsed,
    /// No statement covers the line (blank line, a declaration outside a block, a comment).
    NoStatement,
    /// Two statements share the line (`a(); b();`).
    Ambiguous,
    /// A statement covers the line but is not a standalone expression, call or assignment.
    NotStandalone {
        /// The grammar kind of the statement (`if_statement`, `return_expression`).
        kind: String,
    },
}

impl std::fmt::Display for StatementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unparsed => f.write_str("the file could not be parsed"),
            Self::NoStatement => f.write_str("no statement covers the line"),
            Self::Ambiguous => f.write_str("two statements share the line"),
            Self::NotStandalone { kind } => write!(
                f,
                "the statement ({kind}) is not a standalone call, expression or assignment"
            ),
        }
    }
}

/// A statement found and the text that replaces it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stub {
    /// Byte range of the statement in the source.
    pub range: Range<usize>,
    /// 1-based first line of the statement.
    pub first_line: u32,
    /// 1-based last line of the statement.
    pub last_line: u32,
    /// The no-op plus the statement's newlines.
    pub replacement: String,
}

/// The byte range of the standalone statement at 1-based `line`, or `None` when the line
/// has no such statement. A statement spanning several lines answers for each of them.
pub fn statement_at(lang: Lang, src: &str, line: u32) -> Option<Range<usize>> {
    stub_statement(lang, src, line).ok().map(|s| s.range)
}

/// The language's no-op statement.
pub fn noop(lang: Lang) -> &'static str {
    match lang {
        Lang::Python => "pass",
        Lang::Kotlin => "Unit",
        Lang::Rust => "();",
        Lang::Java
        | Lang::Go
        | Lang::JavaScript
        | Lang::TypeScript
        | Lang::Tsx
        | Lang::C
        | Lang::Cpp => ";",
    }
}

/// [`statement_at`] with the reason for a refusal, and the replacement text.
pub fn stub_statement(lang: Lang, src: &str, line: u32) -> Result<Stub, StatementError> {
    if line == 0 {
        return Err(StatementError::NoStatement);
    }
    let tree = parse(lang, src).map_err(|_| StatementError::Unparsed)?;
    let row = (line - 1) as usize;
    let mut found: Vec<Node<'_>> = Vec::new();
    collect(tree.root_node(), row, &mut found);
    // The innermost statement covering the line; two disjoint ones are ambiguous.
    let Some(inner) = found
        .iter()
        .copied()
        .max_by_key(|n| (n.start_byte(), std::cmp::Reverse(n.end_byte())))
    else {
        return Err(StatementError::NoStatement);
    };
    if found
        .iter()
        .any(|n| n.start_byte() > inner.start_byte() || n.end_byte() < inner.end_byte())
    {
        return Err(StatementError::Ambiguous);
    }
    if !accepted(lang, inner, src) {
        return Err(StatementError::NotStandalone {
            kind: inner.kind().to_owned(),
        });
    }
    let range = inner.start_byte()..inner.end_byte();
    let text = src.get(range.clone()).unwrap_or_default();
    let newlines = text.matches('\n').count();
    let mut replacement = noop(lang).to_owned();
    replacement.push_str(&"\n".repeat(newlines));
    Ok(Stub {
        range,
        first_line: u32::try_from(inner.start_position().row + 1).unwrap_or(u32::MAX),
        last_line: u32::try_from(inner.end_position().row + 1).unwrap_or(u32::MAX),
        replacement,
    })
}

/// Statements (children of a container) whose lines include `row`.
fn collect<'t>(node: Node<'t>, row: usize, out: &mut Vec<Node<'t>>) {
    if node.start_position().row > row || node.end_position().row < row {
        return;
    }
    let in_container = CONTAINERS.contains(&node.kind());
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if in_container && child.is_named() && !is_comment(child.kind()) && covers(child, row) {
            out.push(child);
        }
        collect(child, row, out);
    }
}

fn covers(n: Node<'_>, row: usize) -> bool {
    n.start_position().row <= row && n.end_position().row >= row
}

/// True for a statement whose value is unused and which does only a call, an expression
/// or an assignment.
fn accepted(lang: Lang, stmt: Node<'_>, src: &str) -> bool {
    let kind = stmt.kind();
    let inner_ok = |allowed: &[&str]| {
        let mut c = stmt.walk();
        let mut named = stmt
            .named_children(&mut c)
            .filter(|n| !is_comment(n.kind()));
        match (named.next(), named.next()) {
            (Some(only), None) => allowed.contains(&only.kind()),
            _ => false,
        }
    };
    match lang {
        Lang::Java => {
            kind == "expression_statement"
                && inner_ok(&[
                    "method_invocation",
                    "assignment_expression",
                    "update_expression",
                    "object_creation_expression",
                ])
        }
        Lang::JavaScript | Lang::TypeScript | Lang::Tsx => {
            kind == "expression_statement"
                && inner_ok(&[
                    "call_expression",
                    "assignment_expression",
                    "augmented_assignment_expression",
                    "update_expression",
                    "new_expression",
                    "await_expression",
                ])
        }
        Lang::Go => {
            matches!(
                kind,
                "assignment_statement" | "inc_statement" | "dec_statement"
            ) || (kind == "expression_statement" && inner_ok(&["call_expression"]))
        }
        Lang::Rust => {
            kind == "expression_statement"
                && inner_ok(&[
                    "call_expression",
                    "assignment_expression",
                    "compound_assignment_expr",
                ])
                && src
                    .get(stmt.start_byte()..stmt.end_byte())
                    .is_some_and(|t| t.trim_end().ends_with(';'))
        }
        Lang::Python => {
            kind == "expression_statement"
                && inner_ok(&["call", "assignment", "augmented_assignment", "await"])
        }
        Lang::C | Lang::Cpp => {
            kind == "expression_statement"
                && inner_ok(&[
                    "call_expression",
                    "assignment_expression",
                    "update_expression",
                ])
        }
        Lang::Kotlin => matches!(kind, "call_expression" | "assignment"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lang: Lang, src: &str, line: u32) -> Option<String> {
        statement_at(lang, src, line).map(|r| src[r].to_owned())
    }

    fn refusal(lang: Lang, src: &str, line: u32) -> StatementError {
        stub_statement(lang, src, line).unwrap_err()
    }

    /// Applies the stub and checks the line count is unchanged.
    fn applied(lang: Lang, src: &str, line: u32) -> String {
        let s = stub_statement(lang, src, line).unwrap();
        let mut out = src.to_owned();
        out.replace_range(s.range, &s.replacement);
        assert_eq!(out.matches('\n').count(), src.matches('\n').count());
        out
    }

    const JAVA: &str = "class A {\n  void f() {\n    items.add(x);\n    count += 1;\n    int y = 2;\n    if (c) {\n      log.add(1);\n    }\n    return;\n  }\n}\n";

    #[test]
    fn java() {
        assert_eq!(text(Lang::Java, JAVA, 3).as_deref(), Some("items.add(x);"));
        assert_eq!(text(Lang::Java, JAVA, 4).as_deref(), Some("count += 1;"));
        assert_eq!(text(Lang::Java, JAVA, 7).as_deref(), Some("log.add(1);"));
        // A declaration with an initializer, a condition, a return, a brace line.
        assert!(matches!(
            refusal(Lang::Java, JAVA, 5),
            StatementError::NotStandalone { .. }
        ));
        assert!(matches!(
            refusal(Lang::Java, JAVA, 6),
            StatementError::NotStandalone { .. }
        ));
        assert!(matches!(
            refusal(Lang::Java, JAVA, 9),
            StatementError::NotStandalone { .. }
        ));
        assert!(statement_at(Lang::Java, JAVA, 99).is_none());
        let out = applied(Lang::Java, JAVA, 3);
        assert!(out.contains("    ;\n    count"));
    }

    #[test]
    fn java_multi_line_call_answers_for_each_line() {
        let src = "class A { void f() {\n  items\n    .add(x);\n  g();\n} }\n";
        let want = Some("items\n    .add(x);".to_owned());
        assert_eq!(text(Lang::Java, src, 2), want);
        assert_eq!(text(Lang::Java, src, 3), want);
        let out = applied(Lang::Java, src, 2);
        assert_eq!(out, "class A { void f() {\n  ;\n\n  g();\n} }\n");
    }

    #[test]
    fn two_statements_on_one_line_are_ambiguous() {
        let src = "class A { void f() {\n  a.add(1); b.add(2);\n} }\n";
        assert_eq!(refusal(Lang::Java, src, 2), StatementError::Ambiguous);
    }

    #[test]
    fn a_lambda_expression_body_is_not_a_statement() {
        let src = "class A { void f() {\n  xs.forEach(x -> items.add(x));\n} }\n";
        // The outer call statement is the innermost statement: stubbing it is allowed,
        // but the lambda's own expression is never singled out.
        let s = stub_statement(Lang::Java, src, 2).unwrap();
        assert!(src[s.range].starts_with("xs.forEach"));
    }

    #[test]
    fn kotlin() {
        let src = "class A {\n  fun f() {\n    items.add(x)\n    count += 1\n    val y = 1\n    return\n  }\n}\n";
        assert_eq!(text(Lang::Kotlin, src, 3).as_deref(), Some("items.add(x)"));
        assert_eq!(text(Lang::Kotlin, src, 4).as_deref(), Some("count += 1"));
        assert!(matches!(
            refusal(Lang::Kotlin, src, 5),
            StatementError::NotStandalone { .. }
        ));
        assert!(matches!(
            refusal(Lang::Kotlin, src, 6),
            StatementError::NotStandalone { .. }
        ));
        let out = applied(Lang::Kotlin, src, 3);
        assert!(out.contains("    Unit\n    count"));
    }

    #[test]
    fn go() {
        let src = "package p\n\nfunc f() {\n\ta.x = append(a.x, v)\n\tb.Add(1)\n\tc := 1\n\ti++\n\tch <- 1\n\tgo run()\n}\n";
        assert_eq!(
            text(Lang::Go, src, 4).as_deref(),
            Some("a.x = append(a.x, v)")
        );
        assert_eq!(text(Lang::Go, src, 5).as_deref(), Some("b.Add(1)"));
        assert_eq!(text(Lang::Go, src, 7).as_deref(), Some("i++"));
        for line in [6, 8, 9] {
            assert!(
                matches!(
                    refusal(Lang::Go, src, line),
                    StatementError::NotStandalone { .. }
                ),
                "line {line}"
            );
        }
        let out = applied(Lang::Go, src, 5);
        assert!(out.contains("\t;\n\tc := 1"));
    }

    #[test]
    fn go_case_bodies_are_statements() {
        let src =
            "package p\n\nfunc f(k int) {\n\tswitch k {\n\tcase 1:\n\t\tseen.Add(k)\n\t}\n}\n";
        assert_eq!(text(Lang::Go, src, 6).as_deref(), Some("seen.Add(k)"));
    }

    #[test]
    fn python() {
        let src = "def f():\n    items.append(x)\n    n = 2\n    n += 1\n    \"doc\"\n    for i in y:\n        z.add(i)\n    return n\n";
        assert_eq!(
            text(Lang::Python, src, 2).as_deref(),
            Some("items.append(x)")
        );
        assert_eq!(text(Lang::Python, src, 3).as_deref(), Some("n = 2"));
        assert_eq!(text(Lang::Python, src, 4).as_deref(), Some("n += 1"));
        assert_eq!(text(Lang::Python, src, 7).as_deref(), Some("z.add(i)"));
        for line in [5, 6, 8] {
            assert!(
                matches!(
                    refusal(Lang::Python, src, line),
                    StatementError::NotStandalone { .. }
                ),
                "line {line}"
            );
        }
        let out = applied(Lang::Python, src, 2);
        assert!(out.starts_with("def f():\n    pass\n    n = 2"));
    }

    #[test]
    fn python_multi_line_call_keeps_its_lines() {
        let src = "def f():\n    items.append(\n        x,\n    )\n    g()\n";
        let out = applied(Lang::Python, src, 3);
        assert_eq!(out, "def f():\n    pass\n\n\n    g()\n");
    }

    #[test]
    fn javascript_and_typescript() {
        let js = "function f() {\n  items.push(x);\n  count++;\n  const y = 1;\n  return y;\n}\n";
        assert_eq!(
            text(Lang::JavaScript, js, 2).as_deref(),
            Some("items.push(x);")
        );
        assert_eq!(text(Lang::JavaScript, js, 3).as_deref(), Some("count++;"));
        assert!(matches!(
            refusal(Lang::JavaScript, js, 4),
            StatementError::NotStandalone { .. }
        ));
        assert!(matches!(
            refusal(Lang::JavaScript, js, 5),
            StatementError::NotStandalone { .. }
        ));
        let ts =
            "function f(): void {\n  items.push(x);\n  m.set(k, v);\n  const y: number = 1;\n}\n";
        assert_eq!(
            text(Lang::TypeScript, ts, 3).as_deref(),
            Some("m.set(k, v);")
        );
        assert!(matches!(
            refusal(Lang::TypeScript, ts, 4),
            StatementError::NotStandalone { .. }
        ));
        let tsx = "function F() {\n  items.push(1);\n  return null;\n}\n";
        assert_eq!(text(Lang::Tsx, tsx, 2).as_deref(), Some("items.push(1);"));
        let out = applied(Lang::JavaScript, js, 2);
        assert!(out.contains("  ;\n  count++;"));
    }

    #[test]
    fn javascript_await_call_is_a_statement() {
        let src = "async function f() {\n  await queue.add(1);\n}\n";
        assert_eq!(
            text(Lang::JavaScript, src, 2).as_deref(),
            Some("await queue.add(1);")
        );
    }

    #[test]
    fn rust() {
        let src = "fn f() {\n    v.push(1);\n    x = 2;\n    let y = 3;\n    println!(\"a\");\n    v.len()\n}\n";
        assert_eq!(text(Lang::Rust, src, 2).as_deref(), Some("v.push(1);"));
        assert_eq!(text(Lang::Rust, src, 3).as_deref(), Some("x = 2;"));
        for line in [4, 5, 6] {
            assert!(
                matches!(
                    refusal(Lang::Rust, src, line),
                    StatementError::NotStandalone { .. }
                ),
                "line {line}"
            );
        }
        let out = applied(Lang::Rust, src, 2);
        assert!(out.contains("    ();\n    x = 2;"));
    }

    #[test]
    fn c_and_cpp() {
        let c = "void f(void) {\n  g(1);\n  x = 2;\n  y++;\n  int z = 3;\n}\n";
        assert_eq!(text(Lang::C, c, 2).as_deref(), Some("g(1);"));
        assert_eq!(text(Lang::C, c, 3).as_deref(), Some("x = 2;"));
        assert_eq!(text(Lang::C, c, 4).as_deref(), Some("y++;"));
        assert!(matches!(
            refusal(Lang::C, c, 5),
            StatementError::NotStandalone { .. }
        ));
        let cpp = "void f() {\n  v.push_back(1);\n  m[k] = 2;\n  auto z = 3;\n}\n";
        assert_eq!(text(Lang::Cpp, cpp, 2).as_deref(), Some("v.push_back(1);"));
        assert_eq!(text(Lang::Cpp, cpp, 3).as_deref(), Some("m[k] = 2;"));
        assert!(matches!(
            refusal(Lang::Cpp, cpp, 4),
            StatementError::NotStandalone { .. }
        ));
        let out = applied(Lang::Cpp, cpp, 2);
        assert!(out.contains("  ;\n  m[k]"));
    }

    #[test]
    fn blank_and_comment_lines_have_no_statement() {
        let src = "def f():\n    # note\n\n    g()\n";
        // The function around them is a statement, but not a stub-able one.
        assert!(matches!(
            refusal(Lang::Python, src, 2),
            StatementError::NotStandalone { .. }
        ));
        assert!(statement_at(Lang::Python, src, 3).is_none());
        assert_eq!(
            refusal(Lang::Python, "x = 1\n", 5),
            StatementError::NoStatement
        );
        assert!(statement_at(Lang::Python, src, 0).is_none());
    }
}
