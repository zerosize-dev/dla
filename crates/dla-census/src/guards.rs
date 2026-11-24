//! The lines above a line of a function that decide whether the line runs.
//!
//! A plain rule over the text of formatted code, the same in every language: walking up from the
//! line to the start of its function, keep (a) each line with smaller indentation than what has
//! been kept so far that opens a condition or a loop, and (b) each `if` at the same or smaller
//! indentation whose block leaves early before the line. Nearest first, at most [`MAX_GUARDS`]
//! lines.

use std::{fs, path::Path};

use crate::parse::{Lang, parse, tree_sitter::Point};

/// The most guard lines given.
pub const MAX_GUARDS: usize = 6;
/// Characters kept of a line of source.
pub const CUT: usize = 140;
/// Lines walked up at most when the start of the function is not known.
const WALK: usize = 400;

/// Words that open a condition or a loop.
const OPENERS: &[&str] = &[
    "if", "else", "match", "switch", "case", "while", "for", "loop", "when", "try", "catch",
    "except", "select",
];
/// Words that leave a block early.
const LEAVES: &[&str] = &[
    "return",
    "break",
    "continue",
    "throw",
    "raise",
    "panic",
    "panic!",
    "unreachable!",
];
/// Words that start a function.
const FUNCTION_WORDS: &[&str] = &["fn", "func", "function", "def", "fun"];

/// Node kinds that are a function or method.
const FUNCTIONS: &[&str] = &[
    "function_declaration",
    "method_declaration",
    "function_definition",
    "function_item",
    "method_definition",
    "constructor_declaration",
    "function_expression",
    "generator_function_declaration",
];

/// One line of source: its 1-based number and its text, trimmed and cut.
pub type Shown = (u32, String);

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn is_blank_or_comment(t: &str) -> bool {
    t.is_empty()
        || t.starts_with("//")
        || t.starts_with('#')
        || t.starts_with("/*")
        || t.starts_with('*')
}

/// The words of a line: runs of letters, digits, `_` and `!`.
fn words(t: &str) -> impl Iterator<Item = &str> {
    t.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '!'))
        .filter(|w| !w.is_empty())
}

/// The first word of a line once leading closing braces and spaces are dropped.
fn first_word(t: &str) -> &str {
    let t = t.trim_start_matches(|c: char| c == '}' || c == ')' || c.is_whitespace());
    words(t).next().unwrap_or("")
}

fn opens_condition(t: &str) -> bool {
    OPENERS.contains(&first_word(t))
}

fn is_if(t: &str) -> bool {
    let t = t.trim_start_matches(|c: char| c == '}' || c.is_whitespace());
    let t = t.strip_prefix("else").map_or(t, str::trim_start);
    t.strip_prefix("if")
        .is_some_and(|rest| rest.starts_with([' ', '(', '\t']))
}

fn starts_function(t: &str) -> bool {
    words(t)
        .take(6)
        .any(|w| FUNCTION_WORDS.contains(&w) || w == "constructor")
        && !opens_condition(t)
}

/// True when the block that starts at `lines[at]` (an `if`) leaves early: its own line after the
/// condition, or a deeper line before the next line at its own indentation, holds a leaving
/// word. `before` is the index the writer is at; a block that reaches it does not count.
fn leaves_early(lines: &[&str], at: usize, before: usize) -> bool {
    let base = indent(lines[at]);
    let mut i = at;
    let leaves = |t: &str| words(t).any(|w| LEAVES.contains(&w));
    // The rest of the `if` line: `if x { return; }`.
    if let Some((_, tail)) = lines[at]
        .split_once('{')
        .or_else(|| lines[at].split_once(':'))
        && leaves(tail)
    {
        return true;
    }
    i += 1;
    while i < before {
        let t = lines[i].trim();
        if t.is_empty() {
            i += 1;
            continue;
        }
        if indent(lines[i]) <= base {
            return false;
        }
        if leaves(t) {
            return true;
        }
        i += 1;
    }
    false
}

fn cut(text: &str) -> String {
    let t = text.trim();
    if t.chars().count() > CUT {
        let mut s: String = t.chars().take(CUT - 3).collect();
        s.push_str("...");
        s
    } else {
        t.to_owned()
    }
}

/// The guard lines of `line` (1-based) in `src`, nearest first. `start` is the 1-based line of
/// the start of the function around it when known; the walk stops there.
pub fn guard_lines(src: &str, line: u32, start: Option<u32>) -> Vec<Shown> {
    let lines: Vec<&str> = src.lines().collect();
    let Some(row) = (line as usize).checked_sub(1).filter(|r| *r < lines.len()) else {
        return Vec::new();
    };
    let floor = match start {
        // The function's own line is not a guard.
        Some(s) => (s as usize).min(row + 1),
        None => row.saturating_sub(WALK),
    };
    let mut out: Vec<Shown> = Vec::new();
    let mut cur = indent(lines[row]);
    let mut i = row;
    while i > floor && out.len() < MAX_GUARDS {
        i -= 1;
        let l = lines[i];
        let t = l.trim();
        if is_blank_or_comment(t) {
            continue;
        }
        let ind = indent(l);
        if ind < cur {
            if start.is_none() && starts_function(t) {
                break;
            }
            if opens_condition(t) {
                out.push((u32::try_from(i + 1).unwrap_or(u32::MAX), cut(t)));
            }
            cur = ind;
        } else if ind == cur && is_if(t) && leaves_early(&lines, i, row) {
            out.push((u32::try_from(i + 1).unwrap_or(u32::MAX), cut(t)));
        }
    }
    out
}

/// The 1-based first line of the innermost function that holds `line` of the file, read with
/// the syntax tree; `None` for a file of no known language or that does not parse.
pub fn function_start(src: &str, path: &Path, line: u32) -> Option<u32> {
    let lang = Lang::from_path(path)?;
    let tree = parse(lang, src).ok()?;
    let row = (line as usize).checked_sub(1)?;
    let column = src.lines().nth(row).map_or(0, indent);
    let at = Point { row, column };
    let mut node = tree.root_node().descendant_for_point_range(at, at);
    while let Some(n) = node {
        if FUNCTIONS.contains(&n.kind()) {
            return u32::try_from(n.start_position().row + 1).ok();
        }
        node = n.parent();
    }
    None
}

/// The text of a line of a project file: the line trimmed and cut, `None` when it cannot be read.
pub fn line_text(project: &Path, file: &str, line: u32) -> Option<String> {
    let src = fs::read_to_string(project.join(file)).ok()?;
    let text = src.lines().nth((line as usize).checked_sub(1)?)?;
    Some(cut(text))
}

/// The guard lines of `file:line` of the project, `[(line, text)]` nearest first, read with the
/// syntax tree for the start of the function. Empty when the file cannot be read.
pub fn guards_of(project: &Path, file: &str, line: u32) -> Vec<Shown> {
    let Ok(src) = fs::read_to_string(project.join(file)) else {
        return Vec::new();
    };
    let start = function_start(&src, Path::new(file), line);
    guard_lines(&src, line, start)
}

/// The sentence that says what a unit must do for a writer line that did not run.
pub const INTRO: &str = "the unit must put the object in the state in which these lines let the writer run, and a state that normal set-up removes (a full buffer, a missing configuration, an error path, a second thread) needs a unit or a set-up that does not remove it";

/// The lines that describe one writer line that did not run, for a driver block or a remedy: the
/// line with its source text and its count in the last run, then the lines above it that decide
/// whether it runs, nearest first, each with its line number. `count` is how often the line ran
/// in the last run (`None`: not counted).
pub fn writer_lines(project: &Path, file: &str, line: u32, count: Option<u64>) -> Vec<String> {
    let text = line_text(project, file, line).unwrap_or_default();
    let ran = match count {
        Some(1) => "ran 1 time".to_owned(),
        Some(n) => format!("ran {n} times"),
        None => "was not counted".to_owned(),
    };
    let mut out = vec![if text.is_empty() {
        format!("{file}:{line} {ran} in the last run")
    } else {
        format!("{file}:{line} {ran} in the last run: {text}")
    }];
    let guards = guards_of(project, file, line);
    if guards.is_empty() {
        out.push("  no condition above it was found in its function".to_owned());
    }
    for (n, t) in guards {
        out.push(format!("  above it, line {n}: {t}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUST: &str = "impl Q {\n    pub fn offer(&mut self, value: u32) {\n        if self.draining {\n            return;\n        }\n        if let Err(Refused::Full(value)) = self.ring.put(value) {\n            log();\n            self.pending.push(value);\n        }\n    }\n}\n";

    #[test]
    fn a_condition_that_opens_the_block_of_the_line_is_a_guard() {
        let g = guard_lines(RUST, 8, Some(2));
        assert_eq!(g.len(), 2, "{g:?}");
        assert_eq!(g[0].0, 6);
        assert!(g[0].1.starts_with("if let Err(Refused::Full"));
        // The early return before the block is the second.
        assert_eq!(g[1], (3, "if self.draining {".to_owned()));
    }

    const GO: &str = "func (m *meter) Counter(name string) (int, error) {\n\tif del, ok := m.delegate.Load().(Meter); ok {\n\t\treturn del.Counter(name)\n\t}\n\tm.mtx.Lock()\n\tdefer m.mtx.Unlock()\n\tm.instruments = append(m.instruments, name)\n\treturn 1, nil\n}\n";

    #[test]
    fn an_if_that_returns_early_above_the_line_is_a_guard_and_a_closed_block_below_is_not() {
        let g = guard_lines(GO, 7, Some(1));
        assert_eq!(g.len(), 1, "{g:?}");
        assert_eq!(g[0].0, 2);
        assert!(g[0].1.contains("m.delegate.Load()"));
    }

    #[test]
    fn a_walk_with_no_known_function_start_stops_at_a_function_header() {
        let src = "def before():\n    if x:\n        return 1\n\ndef put(self, v):\n    if self.ready:\n        return\n    for a in b:\n        self.items.append(v)\n";
        let g = guard_lines(src, 9, None);
        let lines: Vec<u32> = g.iter().map(|(l, _)| *l).collect();
        // The `for` above the line, the early return before it; nothing from `before`.
        assert_eq!(lines, [8, 6], "{g:?}");
    }

    #[test]
    fn a_block_that_is_not_a_condition_and_a_closed_loop_beside_the_line_are_not_guards() {
        // `with` encloses the line but decides nothing.
        let py = "def put(self, v):\n    with self.lock:\n        self.items.append(v)\n";
        assert!(guard_lines(py, 3, Some(1)).is_empty());
        // The loop above is closed before the `if` that encloses the line.
        let rs = "fn f() {\n    for x in y {\n        g(x);\n    }\n    if a {\n        w.push(1);\n    }\n}\n";
        let g = guard_lines(rs, 6, Some(1));
        assert_eq!(g, [(5, "if a {".to_owned())], "{g:?}");
    }

    #[test]
    fn at_most_six_lines_and_each_is_cut() {
        let mut s = String::from("fn f() {\n");
        for d in 0..8 {
            s.push_str(&format!("{}if c{d} {{\n", "    ".repeat(d + 1)));
        }
        s.push_str(&format!("{}x.push(1);\n", "    ".repeat(9)));
        let g = guard_lines(&s, 10, Some(1));
        assert_eq!(g.len(), MAX_GUARDS, "{g:?}");
        assert_eq!(g[0].0, 9);
        let long = format!(
            "fn f() {{\n    if {} {{\n        x.push(1);\n    }}\n}}\n",
            "a".repeat(300)
        );
        let g = guard_lines(&long, 3, Some(1));
        assert!(g[0].1.chars().count() <= CUT, "{g:?}");
        assert!(g[0].1.ends_with("..."));
    }

    #[test]
    fn a_writer_line_is_described_with_its_text_count_and_guards() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/q.rs"), RUST).unwrap();
        let out = writer_lines(dir.path(), "src/q.rs", 8, Some(0));
        assert_eq!(
            out[0],
            "src/q.rs:8 ran 0 times in the last run: self.pending.push(value);"
        );
        assert_eq!(out.len(), 3, "{out:?}");
        assert!(
            out[1].starts_with("  above it, line 6: if let Err("),
            "{out:?}"
        );
        assert_eq!(out[2], "  above it, line 3: if self.draining {");
        let none = writer_lines(dir.path(), "src/q.rs", 40, None);
        assert!(none[0].contains("was not counted"), "{none:?}");
    }

    #[test]
    fn the_function_start_comes_from_the_syntax_tree() {
        assert_eq!(function_start(RUST, Path::new("src/q.rs"), 8), Some(2));
        assert_eq!(function_start(GO, Path::new("m.go"), 7), Some(1));
        assert_eq!(function_start(RUST, Path::new("README"), 8), None);
    }
}
