//! How often a line ran in a coverage run, read from the data the harness already collects.
//!
//! `dla trend` asks, for a target whose function ran, whether the lines that add to its holder
//! (the writer lines of the census) ran as well. A function that ran says nothing about a line
//! behind a state check. The readers here give, per wanted line, a count or nothing:
//!
//! | Runtime | Source | Count of a line |
//! |---|---|---|
//! | Rust | `llvm-cov export` regions, read when the run is made | the innermost code region (kind 0) that covers it, summed over the instantiations of the same span |
//! | Go | `-coverprofile` blocks in `cov/` | the block that covers it and starts last |
//! | Node | `NODE_V8_COVERAGE` block ranges in `cov/`, at source lines through the source maps | the innermost range of a function that has block coverage |
//!
//! A line that no region, block or range covers, a function V8 did not give block coverage for
//! and a run that wrote no data give `null`: unknown, never zero.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde_json::{Value, json};

use super::cover::{file_url_path, original_in_work, relative_to};

/// A project-relative file and a 1-based line.
pub type Wanted = (String, u32);

/// The most lines a record carries.
pub const MAX_LINES: usize = 400;

/// One entry of the `lines` list of a coverage record: `count` is `null` when the line could not
/// be counted, and `why` then says so.
fn entry(file: &str, line: u32, count: Option<u64>, why: &str) -> Value {
    match count {
        Some(n) => json!({"file": file, "line": line, "count": n}),
        None => json!({"file": file, "line": line, "count": null, "why": why}),
    }
}

/// The lines asked for, without a repeat, at most [`MAX_LINES`].
pub fn dedupe(wanted: &[Wanted]) -> Vec<Wanted> {
    let mut out: Vec<Wanted> = Vec::new();
    for w in wanted {
        if !out.contains(w) {
            out.push(w.clone());
        }
    }
    out.truncate(MAX_LINES);
    out
}

/// True when the path a coverage tool printed names the project-relative `file`.
fn same_file(printed: &str, file: &str) -> bool {
    let printed = printed.trim_start_matches("./");
    printed == file || printed.ends_with(&format!("/{file}"))
}

// ---------------------------------------------------------------------------------------
// Rust

/// How wide a region is: lines, then columns on a one-line region. Smaller is more inner.
type Span = (u32, u32);

/// The innermost code region of one function that covers `line` of the files `ids` of the
/// function: its span and count.
fn rust_function_line(regions: &[Value], ids: &BTreeSet<u64>, line: u32) -> Option<(Span, u64)> {
    let mut best: Option<(Span, u64)> = None;
    for r in regions {
        let Some(r) = r.as_array().filter(|r| r.len() >= 8) else {
            continue;
        };
        let num = |i: usize| r[i].as_u64().unwrap_or(u64::MAX);
        // Kind 0 is a code region; expansions, skipped and gap regions are not counts of a line.
        if num(7) != 0 || !ids.contains(&num(5)) {
            continue;
        }
        let (ls, cs, le, ce) = (num(0), num(1), num(2), num(3));
        if ls == u64::MAX || le == u64::MAX {
            continue;
        }
        // The end column is exclusive: a region that ends before column 1 stops on the line above.
        let end = if ce <= 1 && le > ls { le - 1 } else { le };
        let line = u64::from(line);
        if line < ls || line > end {
            continue;
        }
        let span = (
            u32::try_from(end - ls).unwrap_or(u32::MAX),
            if le == ls {
                u32::try_from(ce.saturating_sub(cs)).unwrap_or(u32::MAX)
            } else {
                0
            },
        );
        let count = num(4);
        if count == u64::MAX {
            continue;
        }
        match best {
            Some((s, _)) if s <= span => {}
            _ => best = Some((span, count)),
        }
    }
    best
}

/// The counts of `wanted` in a `llvm-cov export` document: per line the count of the innermost
/// code region that covers it (the regions of every function of the file are searched, and the
/// instantiations that have the same innermost span add up). A line outside every region is
/// unknown. `work` is the scratch project the paths are made relative to.
pub fn rust_line_counts(export: &Value, work: &Path, wanted: &[Wanted]) -> Vec<Value> {
    let work = work.canonicalize().unwrap_or_else(|_| work.to_path_buf());
    let wanted = dedupe(wanted);
    let mut rel_of: BTreeMap<String, Option<String>> = BTreeMap::new();
    // Per wanted line: the best span found and the counts that have it.
    let mut best: Vec<Option<(Span, u64)>> = vec![None; wanted.len()];
    let funcs = export
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|d| d.get("functions").and_then(Value::as_array))
        .flatten();
    for f in funcs {
        let Some(names) = f.get("filenames").and_then(Value::as_array) else {
            continue;
        };
        let Some(regions) = f.get("regions").and_then(Value::as_array) else {
            continue;
        };
        let rels: Vec<Option<String>> = names
            .iter()
            .map(|n| {
                let n = n.as_str()?;
                rel_of
                    .entry(n.to_owned())
                    .or_insert_with(|| relative_to(&work, n))
                    .clone()
            })
            .collect();
        for (i, (file, line)) in wanted.iter().enumerate() {
            let ids: BTreeSet<u64> = rels
                .iter()
                .enumerate()
                .filter(|(_, r)| r.as_deref().is_some_and(|r| same_file(r, file)))
                .map(|(k, _)| k as u64)
                .collect();
            if ids.is_empty() {
                continue;
            }
            if let Some((span, count)) = rust_function_line(regions, &ids, *line) {
                best[i] = match best[i] {
                    Some((s, c)) if s < span => Some((s, c)),
                    Some((s, c)) if s == span => Some((s, c.saturating_add(count))),
                    _ => Some((span, count)),
                };
            }
        }
    }
    wanted
        .iter()
        .zip(best)
        .map(|((file, line), b)| {
            entry(
                file,
                *line,
                b.map(|(_, c)| c),
                "no code region of the coverage export covers the line",
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Go

/// One block of a Go coverage profile.
struct GoBlock {
    path: String,
    start: (u32, u32),
    end: (u32, u32),
    count: u64,
}

/// The blocks of one profile text (`mode: set` first, then `file:l.c,l.c stmts count`).
fn go_blocks(text: &str) -> Vec<GoBlock> {
    let mut out = Vec::new();
    for l in text.lines() {
        if l.starts_with("mode:") {
            continue;
        }
        let mut parts = l.split_whitespace();
        let (Some(loc), Some(_stmts), Some(count)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let Some((path, range)) = loc.rsplit_once(':') else {
            continue;
        };
        let Some((from, to)) = range.split_once(',') else {
            continue;
        };
        let pos = |s: &str| -> Option<(u32, u32)> {
            let (l, c) = s.split_once('.')?;
            Some((l.parse().ok()?, c.parse().ok()?))
        };
        let (Some(start), Some(end), Ok(count)) = (pos(from), pos(to), count.parse::<u64>()) else {
            continue;
        };
        out.push(GoBlock {
            path: path.to_owned(),
            start,
            end,
            count,
        });
    }
    out
}

/// The count of `line` in the blocks of one profile: the block of `file` that covers the line
/// and starts last (blocks do not nest; two that meet on a line, a condition and the body that
/// follows it, give the body to a line that is inside it).
fn go_line(blocks: &[GoBlock], file: &str, line: u32) -> Option<u64> {
    blocks
        .iter()
        .filter(|b| same_file(&b.path, file) && b.start.0 <= line && line <= b.end.0)
        .max_by_key(|b| b.start)
        .map(|b| b.count)
}

/// The counts of `wanted` in the `.out` profiles of `cov_dir`, summed over the profiles that
/// cover the line, and the note to give when no profile exists.
pub fn go_line_counts(cov_dir: &Path, wanted: &[Wanted]) -> Vec<Value> {
    let wanted = dedupe(wanted);
    let mut files: Vec<_> = fs::read_dir(cov_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("out"))
        .collect();
    files.sort();
    let profiles: Vec<Vec<GoBlock>> = files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .map(|t| go_blocks(&t))
        .collect();
    wanted
        .iter()
        .map(|(file, line)| {
            if profiles.is_empty() {
                return entry(file, *line, None, "no Go coverage profile was written to cov/");
            }
            let counts: Vec<u64> = profiles
                .iter()
                .filter_map(|b| go_line(b, file, *line))
                .collect();
            let total = (!counts.is_empty()).then(|| counts.iter().sum::<u64>());
            entry(
                file,
                *line,
                total,
                "no block of the coverage profile covers the line (outside the covered packages, or not a statement)",
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Node

/// Start offsets (in UTF-16 units, as V8 counts) of the lines of `text`.
fn line_starts(text: &str) -> Vec<usize> {
    let mut out = vec![0usize];
    let mut at = 0usize;
    for c in text.chars() {
        at += c.len_utf16();
        if c == '\n' {
            out.push(at);
        }
    }
    out
}

/// Zero-based line and column of a UTF-16 offset.
fn position(starts: &[usize], offset: usize) -> (u32, u32) {
    let line = starts.partition_point(|s| *s <= offset).saturating_sub(1);
    (
        u32::try_from(line).unwrap_or(u32::MAX),
        u32::try_from(offset - starts[line]).unwrap_or(u32::MAX),
    )
}

/// One range of a function with block coverage, at the lines of the file it is in.
struct NodeRange {
    file: String,
    first: u32,
    last: u32,
    /// Width of the range in the built script, to tell the innermost of two that nest.
    width: usize,
    count: u64,
}

/// The ranges of every function with block coverage of one coverage file, at source lines.
fn node_ranges(
    doc: &Value,
    work: &Path,
    maps: &mut dla_parsers::node::sourcemap::ScriptMaps,
) -> Vec<NodeRange> {
    let mut out = Vec::new();
    for script in doc
        .get("result")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(rel) = script
            .get("url")
            .and_then(Value::as_str)
            .and_then(file_url_path)
            .and_then(|p| relative_to(work, &p))
        else {
            continue;
        };
        if rel.starts_with("node_modules/") || rel.contains("/node_modules/") {
            continue;
        }
        let Ok(source) = fs::read_to_string(work.join(&rel)) else {
            continue;
        };
        let starts = line_starts(&source);
        for f in script
            .get("functions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            // Without block coverage only the function's own count is known, and a line of the
            // function is not that count.
            if f.get("isBlockCoverage").and_then(Value::as_bool) != Some(true) {
                continue;
            }
            for r in f
                .get("ranges")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let (Some(from), Some(to), Some(count)) = (
                    r.get("startOffset").and_then(Value::as_u64),
                    r.get("endOffset").and_then(Value::as_u64),
                    r.get("count").and_then(Value::as_u64),
                ) else {
                    continue;
                };
                let (from, to) = (from as usize, (to as usize).max(from as usize + 1));
                let (l0, c0) = position(&starts, from);
                let (l1, c1) = position(&starts, to - 1);
                let width = to - from;
                if let (Some((fa, la)), Some((fb, lb))) = (
                    original_in_work(maps, work, &rel, l0, c0),
                    original_in_work(maps, work, &rel, l1, c1),
                ) {
                    if fa == fb {
                        out.push(NodeRange {
                            file: fa,
                            first: la,
                            last: lb.max(la),
                            width,
                            count,
                        });
                    }
                    continue;
                }
                // A script with no usable map is at its own lines.
                if maps.lookup(&work.join(&rel), l0, c0).is_none() {
                    out.push(NodeRange {
                        file: rel.clone(),
                        first: l0 + 1,
                        last: l1 + 1,
                        width,
                        count,
                    });
                }
            }
        }
    }
    out
}

/// The counts of `wanted` in the `NODE_V8_COVERAGE` files of `cov_dir`: per coverage file (one
/// process) the innermost range of a function with block coverage that covers the line, summed
/// over the files. A line no such range covers is unknown.
pub fn node_line_counts(cov_dir: &Path, work: &Path, wanted: &[Wanted]) -> Vec<Value> {
    let work = work.canonicalize().unwrap_or_else(|_| work.to_path_buf());
    let wanted = dedupe(wanted);
    let mut maps = dla_parsers::node::sourcemap::ScriptMaps::default();
    let mut total: Vec<Option<u64>> = vec![None; wanted.len()];
    let mut any = false;
    for e in fs::read_dir(cov_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
    {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Some(doc) = fs::read_to_string(&p)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        else {
            continue;
        };
        any = true;
        let ranges = node_ranges(&doc, &work, &mut maps);
        for (i, (file, line)) in wanted.iter().enumerate() {
            let inner = ranges
                .iter()
                .filter(|r| &r.file == file && r.first <= *line && *line <= r.last)
                .min_by_key(|r| r.width);
            if let Some(r) = inner {
                total[i] = Some(total[i].unwrap_or(0).saturating_add(r.count));
            }
        }
    }
    wanted
        .iter()
        .zip(total)
        .map(|((file, line), c)| {
            entry(
                file,
                *line,
                c,
                if any {
                    "no block range of a function with block coverage covers the line"
                } else {
                    "no NODE_V8_COVERAGE data was written to cov/"
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regions(list: &[[u64; 8]]) -> Value {
        json!(list)
    }

    fn export(work: &Path, functions: Vec<Value>) -> Value {
        let _ = work;
        json!({"data": [{"functions": functions}]})
    }

    fn func(file: &Path, count: u64, list: &[[u64; 8]]) -> Value {
        json!({"name": "f", "count": count, "filenames": [file.to_string_lossy()],
            "regions": regions(list)})
    }

    fn count_of(v: &[Value], i: usize) -> Option<u64> {
        v[i]["count"].as_u64()
    }

    #[test]
    fn a_rust_region_that_covers_the_line_gives_its_count_and_the_innermost_wins() {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().canonicalize().unwrap();
        let file = work.join("src/lib.rs");
        // The function body (lines 5 to 12, ran 7 times) has an inner block at lines 8 to 10
        // that never ran, and a one-line region at line 11 that ran 2 times.
        let doc = export(
            &work,
            vec![func(
                &file,
                7,
                &[
                    [5, 1, 12, 2, 7, 0, 0, 0],
                    [8, 20, 10, 6, 0, 0, 0, 0],
                    [11, 5, 11, 20, 2, 0, 0, 0],
                    // An expansion region and a region of another file never count.
                    [9, 1, 9, 30, 99, 0, 0, 1],
                    [9, 1, 9, 30, 98, 1, 1, 0],
                ],
            )],
        );
        let wanted = vec![
            ("src/lib.rs".to_owned(), 6),
            ("src/lib.rs".to_owned(), 9),
            ("src/lib.rs".to_owned(), 11),
            ("src/lib.rs".to_owned(), 40),
            ("src/other.rs".to_owned(), 6),
        ];
        let got = rust_line_counts(&doc, &work, &wanted);
        assert_eq!(count_of(&got, 0), Some(7), "{got:?}");
        assert_eq!(count_of(&got, 1), Some(0), "{got:?}");
        assert_eq!(count_of(&got, 2), Some(2), "{got:?}");
        // Outside every region, and in a file the export does not have: unknown, not zero.
        assert_eq!(got[3]["count"], Value::Null, "{got:?}");
        assert!(got[3]["why"].as_str().unwrap().contains("no code region"));
        assert_eq!(got[4]["count"], Value::Null, "{got:?}");
    }

    #[test]
    fn rust_instantiations_of_one_span_add_up_and_a_region_ending_at_column_one_stops_above() {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().canonicalize().unwrap();
        let file = work.join("src/g.rs");
        let doc = export(
            &work,
            vec![
                func(
                    &file,
                    2,
                    &[[3, 1, 6, 2, 2, 0, 0, 0], [4, 5, 5, 1, 0, 0, 0, 0]],
                ),
                func(
                    &file,
                    3,
                    &[[3, 1, 6, 2, 3, 0, 0, 0], [4, 5, 5, 1, 1, 0, 0, 0]],
                ),
            ],
        );
        let wanted = vec![("src/g.rs".to_owned(), 5), ("src/g.rs".to_owned(), 6)];
        let got = rust_line_counts(&doc, &work, &wanted);
        // Line 4 is inside the inner region (0 + 1 over the two instantiations); line 5 is not
        // (it ends before column 1 of line 5, so it stops at line 4) and takes the outer 2 + 3.
        assert_eq!(count_of(&got, 0), Some(5), "{got:?}");
        assert_eq!(count_of(&got, 1), Some(5), "{got:?}");
        let on_four = rust_line_counts(&doc, &work, &[("src/g.rs".to_owned(), 4)]);
        assert_eq!(count_of(&on_four, 0), Some(1), "{on_four:?}");
    }

    const GO_PROFILE: &str = "mode: set\n\
mod/p/m.go:31.80,32.60 2 1\n\
mod/p/m.go:32.60,34.3 1 0\n\
mod/p/m.go:35.2,40.3 4 1\n\
mod/p/n.go:3.1,4.2 1 1\n";

    #[test]
    fn a_go_block_that_covers_the_line_gives_its_count() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("p1-fresh-cover.out"), GO_PROFILE).unwrap();
        let wanted = vec![
            ("p/m.go".to_owned(), 33),
            ("p/m.go".to_owned(), 38),
            // The line where a block ends and the next begins belongs to the block that starts
            // last: the body.
            ("p/m.go".to_owned(), 32),
            ("p/m.go".to_owned(), 50),
            ("q/m.go".to_owned(), 33),
        ];
        let got = go_line_counts(dir.path(), &wanted);
        assert_eq!(count_of(&got, 0), Some(0), "{got:?}");
        assert_eq!(count_of(&got, 1), Some(1), "{got:?}");
        assert_eq!(count_of(&got, 2), Some(0), "{got:?}");
        assert_eq!(got[3]["count"], Value::Null, "{got:?}");
        assert_eq!(got[4]["count"], Value::Null, "{got:?}");
        // No profile at all: unknown, with the reason.
        let none = tempfile::tempdir().unwrap();
        let got = go_line_counts(none.path(), &wanted[..1]);
        assert!(
            got[0]["why"]
                .as_str()
                .unwrap()
                .contains("no Go coverage profile")
        );
    }

    #[test]
    fn two_go_profiles_add_up() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("a.out"),
            "mode: set\nmod/p/m.go:3.1,4.2 1 0\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("b.out"),
            "mode: set\nmod/p/m.go:3.1,4.2 1 1\n",
        )
        .unwrap();
        let got = go_line_counts(dir.path(), &[("p/m.go".to_owned(), 3)]);
        assert_eq!(count_of(&got, 0), Some(1), "{got:?}");
    }

    fn url_of(work: &Path, f: &str) -> String {
        let shown = work.to_string_lossy().replace('\\', "/");
        let shown = shown.strip_prefix("//?/").unwrap_or(&shown).to_owned();
        if shown.starts_with('/') {
            format!("file://{shown}/{f}")
        } else {
            format!("file:///{shown}/{f}")
        }
    }

    #[test]
    fn a_node_block_range_gives_the_count_at_a_source_line_and_a_function_without_blocks_is_unknown()
     {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().canonicalize().unwrap();
        let cov = work.join("cov");
        fs::create_dir_all(&cov).unwrap();
        // Built script, three lines: the function, a branch block, the statement in it.
        //   0: function put(x) {\n            (offsets 0..20)
        //   1:   if (full) { list.push(x); }\n (offsets 21..51)
        //   2: }\n
        let js = "function put(x) {\n  if (full) { list.push(x); }\n}\nfunction plain() {\n  return 1;\n}\n";
        fs::write(work.join("lib.js"), js).unwrap();
        let branch_from = js.find("{ list").unwrap();
        let branch_to = js.find("}\n}").unwrap() + 1;
        let doc = json!({"result": [{"url": url_of(&work, "lib.js"), "functions": [
            {"functionName": "put", "isBlockCoverage": true, "ranges": [
                {"startOffset": 0, "endOffset": js.find("function plain").unwrap() - 1, "count": 4},
                {"startOffset": branch_from, "endOffset": branch_to, "count": 0}]},
            {"functionName": "plain", "isBlockCoverage": false, "ranges": [
                {"startOffset": js.find("function plain").unwrap(), "endOffset": js.len(), "count": 4}]}
        ]}]});
        fs::write(cov.join("coverage-1.json"), doc.to_string()).unwrap();
        let wanted = vec![
            ("lib.js".to_owned(), 2),
            ("lib.js".to_owned(), 1),
            ("lib.js".to_owned(), 5),
        ];
        let got = node_line_counts(&cov, &work, &wanted);
        // Line 2 is in the branch block that never ran; line 1 is the function's own count; the
        // function without block coverage says nothing about its line.
        assert_eq!(count_of(&got, 0), Some(0), "{got:?}");
        assert_eq!(count_of(&got, 1), Some(4), "{got:?}");
        assert_eq!(got[2]["count"], Value::Null, "{got:?}");
    }

    #[test]
    fn a_node_range_of_a_built_script_is_counted_at_the_line_of_its_source() {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().canonicalize().unwrap();
        let cov = work.join("cov");
        fs::create_dir_all(&cov).unwrap();
        fs::write(
            work.join("src.ts"),
            "// source\nfunction put(x) {\n  if (full) { return; }\n  list.push(x);\n}\n",
        )
        .unwrap();
        // The built script: line 0 the function head, line 1 the branch, line 2 the push, line
        // 3 the end; one segment at column 0 of each line maps to source lines 1 to 4.
        let js = "function put(x) {\n  if (full) { return; }\n  list.push(x);\n}\n//# sourceMappingURL=lib.js.map\n";
        fs::write(work.join("lib.js"), js).unwrap();
        fs::write(
            work.join("lib.js.map"),
            r#"{"version":3,"sources":["src.ts"],"mappings":"AACA;AACA;AACA;AACA"}"#,
        )
        .unwrap();
        let (b0, b1) = (js.find("{ return").unwrap(), js.find("; }").unwrap() + 3);
        let doc = json!({"result": [{"url": url_of(&work, "lib.js"), "functions": [
            {"functionName": "put", "isBlockCoverage": true, "ranges": [
                {"startOffset": 0, "endOffset": js.find("}\n//#").unwrap() + 1, "count": 5},
                {"startOffset": b0, "endOffset": b1, "count": 0}]}]}]});
        fs::write(cov.join("coverage-1.json"), doc.to_string()).unwrap();
        let got = node_line_counts(
            &cov,
            &work,
            &[("src.ts".to_owned(), 3), ("src.ts".to_owned(), 4)],
        );
        assert_eq!(count_of(&got, 0), Some(0), "{got:?}");
        assert_eq!(count_of(&got, 1), Some(5), "{got:?}");
    }
    #[test]
    fn a_node_range_that_starts_and_ends_in_two_sources_counts_for_neither() {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().canonicalize().unwrap();
        let cov = work.join("cov");
        fs::create_dir_all(&cov).unwrap();
        fs::write(work.join("a.ts"), "one\ntwo\n").unwrap();
        fs::write(work.join("b.ts"), "one\ntwo\n").unwrap();
        // Generated line 0 comes from a.ts and line 1 from b.ts (a bundle of two modules).
        let js = "function f() {\nreturn 1; }\n//# sourceMappingURL=lib.js.map\n";
        fs::write(work.join("lib.js"), js).unwrap();
        fs::write(
            work.join("lib.js.map"),
            r#"{"version":3,"sources":["a.ts","b.ts"],"mappings":"AAAA;ACAA"}"#,
        )
        .unwrap();
        let doc = json!({"result": [{"url": url_of(&work, "lib.js"), "functions": [
            {"functionName": "f", "isBlockCoverage": true, "ranges": [
                {"startOffset": 0, "endOffset": js.find("; }").unwrap() + 3, "count": 9}]}]}]});
        fs::write(cov.join("coverage-1.json"), doc.to_string()).unwrap();
        let got = node_line_counts(
            &cov,
            &work,
            &[("a.ts".to_owned(), 1), ("b.ts".to_owned(), 1)],
        );
        assert_eq!(got[0]["count"], Value::Null, "{got:?}");
        assert_eq!(got[1]["count"], Value::Null, "{got:?}");
    }
}
