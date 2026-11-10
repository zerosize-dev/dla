//! Unified diffs: parsing, a pure-Rust applier, and a diff writer for same-length edits.
//!
//! `dla intervene --patch` takes a unified diff from the model. No external `patch` is
//! run: [`parse`] reads the diff, [`apply`] checks every context and removed line against
//! the file (allowing a hunk to sit a few lines from where its header says) and builds the
//! new text, and refuses when a hunk does not fit. [`same_length`] writes the diff of a
//! stub edit, which never moves a line.

/// Most lines a hunk may sit from its stated position.
const MAX_SHIFT: usize = 100;

/// One line of a hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// An unchanged line.
    Context(String),
    /// A line removed from the file.
    Remove(String),
    /// A line added; the flag is true for `\ No newline at end of file`.
    Add(String, bool),
}

/// A hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    /// 1-based first line in the old file (the line before an insertion when `old_len` is 0).
    pub old_start: usize,
    /// Lines of the old file the hunk covers.
    pub old_len: usize,
    /// The lines.
    pub lines: Vec<Line>,
}

/// The hunks for one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePatch {
    /// Path with any `a/` or `b/` prefix removed.
    pub path: String,
    /// The hunks, in file order.
    pub hunks: Vec<Hunk>,
}

/// Where a hunk landed and what it changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HunkSite {
    /// First old line the hunk changed (1-based).
    pub first: usize,
    /// Last old line the hunk changed (an insertion counts the line it follows).
    pub last: usize,
    /// Lines removed.
    pub removed: usize,
    /// Lines added.
    pub added: usize,
    /// How far from the stated position the hunk applied (0 when exact).
    pub shift: i64,
}

/// A diff applied to one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    /// The new text.
    pub text: String,
    /// One entry per hunk.
    pub sites: Vec<HunkSite>,
}

/// The path without `a/` or `b/` and without any leading `./`, so the same file written two
/// ways is one key.
fn strip_prefix(path: &str) -> &str {
    let mut p = path
        .strip_prefix("a/")
        .or_else(|| path.strip_prefix("b/"))
        .unwrap_or(path);
    while let Some(rest) = p.strip_prefix("./") {
        p = rest;
    }
    p
}

/// The path on a `---` or `+++` line: up to the first tab, quotes removed.
fn header_path(rest: &str) -> String {
    let p = rest.split('\t').next().unwrap_or(rest).trim();
    p.trim_matches('"').to_owned()
}

fn parse_range(s: &str) -> Option<(usize, usize)> {
    let s = s.trim_start_matches(['-', '+']);
    match s.split_once(',') {
        Some((a, b)) => Some((a.parse().ok()?, b.parse().ok()?)),
        None => Some((s.parse().ok()?, 1)),
    }
}

/// `@@ -a,b +c,d @@` -> `(a, b, c, d)`.
fn parse_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
    let body = line.strip_prefix("@@ ")?;
    let (ranges, _) = body.split_once(" @@")?;
    let (old, new) = ranges.split_once(' ')?;
    let (a, b) = parse_range(old)?;
    let (c, d) = parse_range(new)?;
    Some((a, b, c, d))
}

/// Parses a unified diff. Refuses a diff that creates or deletes a file, one with no
/// hunks, and a hunk whose line counts do not match its header.
pub fn parse(text: &str) -> Result<Vec<FilePatch>, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<FilePatch> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let Some(old) = lines[i].strip_prefix("--- ") else {
            i += 1;
            continue;
        };
        let Some(new) = lines.get(i + 1).and_then(|l| l.strip_prefix("+++ ")) else {
            return Err(format!("line {}: `---` is not followed by `+++`", i + 1));
        };
        let (old, new) = (header_path(old), header_path(new));
        if old == "/dev/null" || new == "/dev/null" {
            return Err(
                "the diff creates or deletes a file; only edits to existing files are allowed"
                    .to_owned(),
            );
        }
        let path = strip_prefix(&new).to_owned();
        i += 2;
        let mut hunks: Vec<Hunk> = Vec::new();
        while i < lines.len() && lines[i].starts_with("@@") {
            let Some((old_start, old_len, _new_start, new_len)) = parse_hunk_header(lines[i])
            else {
                return Err(format!("line {}: malformed hunk header", i + 1));
            };
            i += 1;
            let (mut have_old, mut have_new) = (0, 0);
            let mut body: Vec<Line> = Vec::new();
            while (have_old < old_len || have_new < new_len) && i < lines.len() {
                let l = lines[i];
                if let Some(rest) = l.strip_prefix('\\') {
                    let _ = rest;
                    if let Some(Line::Add(_, flag)) = body.last_mut() {
                        *flag = true;
                    }
                    i += 1;
                    continue;
                }
                match l.chars().next() {
                    Some('+') => {
                        body.push(Line::Add(l[1..].to_owned(), false));
                        have_new += 1;
                    }
                    Some('-') => {
                        body.push(Line::Remove(l[1..].to_owned()));
                        have_old += 1;
                    }
                    Some(' ') => {
                        body.push(Line::Context(l[1..].to_owned()));
                        have_old += 1;
                        have_new += 1;
                    }
                    // Editors strip the space of an empty context line.
                    None => {
                        body.push(Line::Context(String::new()));
                        have_old += 1;
                        have_new += 1;
                    }
                    Some(_) => break,
                }
                i += 1;
            }
            // A trailing "no newline" marker belongs to the hunk just read.
            while i < lines.len() && lines[i].starts_with('\\') {
                if let Some(Line::Add(_, flag)) = body.last_mut() {
                    *flag = true;
                }
                i += 1;
            }
            if have_old != old_len || have_new != new_len {
                return Err(format!(
                    "hunk at old line {old_start} of {path} has {have_old} old and {have_new} new lines, its header says {old_len} and {new_len}"
                ));
            }
            hunks.push(Hunk {
                old_start,
                old_len,
                lines: body,
            });
        }
        if hunks.is_empty() {
            return Err(format!("no hunks for {path}"));
        }
        match out.iter_mut().find(|f| f.path == path) {
            Some(f) => {
                f.hunks.extend(hunks);
                f.hunks.sort_by_key(|h| h.old_start);
            }
            None => out.push(FilePatch { path, hunks }),
        }
    }
    if out.is_empty() {
        return Err("the diff has no `---`/`+++` file header".to_owned());
    }
    Ok(out)
}

fn same(a: &str, b: &str) -> bool {
    a.trim_end_matches(['\r', '\n']) == b.trim_end_matches(['\r', '\n'])
}

/// Applies `patch` to `old`. Every context and removed line must match; a hunk may sit up
/// to [`MAX_SHIFT`] lines from its stated position (the nearest fit wins, after the
/// previous hunk).
pub fn apply(old: &str, patch: &FilePatch) -> Result<Applied, String> {
    let src: Vec<&str> = old.split_inclusive('\n').collect();
    let eol = if old.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = String::with_capacity(old.len());
    let mut sites = Vec::new();
    let mut cursor = 0usize; // next old line (0-based) not yet copied
    for (n, h) in patch.hunks.iter().enumerate() {
        let want: Vec<&str> = h
            .lines
            .iter()
            .filter_map(|l| match l {
                Line::Context(t) | Line::Remove(t) => Some(t.as_str()),
                Line::Add(..) => None,
            })
            .collect();
        let nominal = if h.old_len == 0 {
            h.old_start
        } else {
            h.old_start.saturating_sub(1)
        };
        let fits = |p: usize| {
            p >= cursor
                && p + want.len() <= src.len()
                && want.iter().zip(&src[p..]).all(|(w, s)| same(w, s))
        };
        let found = (0..=MAX_SHIFT).find_map(|d| {
            [nominal.checked_add(d), nominal.checked_sub(d)]
                .into_iter()
                .flatten()
                .find(|&p| fits(p))
        });
        let Some(start) = found else {
            let first = want.first().copied().unwrap_or("");
            return Err(format!(
                "hunk {} of {} does not apply near line {} (first line expected: {:?})",
                n + 1,
                patch.path,
                h.old_start,
                first.trim()
            ));
        };
        for s in &src[cursor..start] {
            out.push_str(s);
        }
        let mut at = start; // old line index
        let (mut first, mut last) = (usize::MAX, 0usize);
        let (mut removed, mut added) = (0, 0);
        for l in &h.lines {
            match l {
                Line::Context(_) => {
                    out.push_str(src[at]);
                    at += 1;
                }
                Line::Remove(_) => {
                    first = first.min(at + 1);
                    last = last.max(at + 1);
                    removed += 1;
                    at += 1;
                }
                Line::Add(t, no_eol) => {
                    // An insertion touches the old line it follows.
                    first = first.min(at.max(1));
                    last = last.max(at.max(1));
                    added += 1;
                    out.push_str(t);
                    if !no_eol {
                        out.push_str(eol);
                    }
                }
            }
        }
        cursor = at;
        let shift = start as i64 - nominal as i64;
        if first == usize::MAX {
            (first, last) = (at + 1, at + 1);
        }
        sites.push(HunkSite {
            first,
            last,
            removed,
            added,
            shift,
        });
    }
    for s in &src[cursor..] {
        out.push_str(s);
    }
    Ok(Applied { text: out, sites })
}

/// The unified diff between `old` and `new` when both have the same number of lines (a
/// stub never moves a line), three lines of context.
pub fn same_length(path: &str, old: &str, new: &str) -> String {
    let (a, b): (Vec<&str>, Vec<&str>) = (
        old.split_inclusive('\n').collect(),
        new.split_inclusive('\n').collect(),
    );
    debug_assert_eq!(a.len(), b.len());
    let changed: Vec<usize> = (0..a.len().min(b.len()))
        .filter(|&i| a[i] != b[i])
        .collect();
    let mut out = format!("--- a/{path}\n+++ b/{path}\n");
    let mut i = 0;
    while i < changed.len() {
        let lo = changed[i].saturating_sub(3);
        let mut hi = changed[i];
        let mut j = i;
        while j + 1 < changed.len() && changed[j + 1] <= hi + 7 {
            j += 1;
            hi = changed[j];
        }
        let hi = (hi + 3).min(a.len() - 1);
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            lo + 1,
            hi - lo + 1,
            lo + 1,
            hi - lo + 1
        ));
        for k in lo..=hi {
            let text = |s: &str| {
                let t = s.trim_end_matches(['\r', '\n']);
                format!("{t}\n")
            };
            if a[k] == b[k] {
                out.push_str(&format!(" {}", text(a[k])));
            } else {
                out.push_str(&format!("-{}", text(a[k])));
                out.push_str(&format!("+{}", text(b[k])));
            }
        }
        i = j + 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD: &str = "a\nb\nc\nd\ne\nf\ng\nh\n";

    fn patch(body: &str) -> Vec<FilePatch> {
        parse(&format!("--- a/x.txt\n+++ b/x.txt\n{body}")).unwrap()
    }

    #[test]
    fn a_hunk_replaces_a_line() {
        let p = patch("@@ -2,3 +2,3 @@\n b\n-c\n+C\n d\n");
        let r = apply(OLD, &p[0]).unwrap();
        assert_eq!(r.text, "a\nb\nC\nd\ne\nf\ng\nh\n");
        assert_eq!(
            r.sites,
            [HunkSite {
                first: 3,
                last: 3,
                removed: 1,
                added: 1,
                shift: 0
            }]
        );
    }

    #[test]
    fn a_hunk_may_sit_a_few_lines_from_its_header() {
        let p = patch("@@ -5,3 +5,3 @@\n b\n-c\n+C\n d\n");
        let r = apply(OLD, &p[0]).unwrap();
        assert_eq!(r.text, "a\nb\nC\nd\ne\nf\ng\nh\n");
        assert_eq!(r.sites[0].shift, -3);
        assert_eq!(r.sites[0].first, 3);
    }

    #[test]
    fn an_insertion_and_a_deletion() {
        let p = patch("@@ -3,2 +3,3 @@\n c\n+c2\n d\n@@ -7,2 +8,1 @@\n g\n-h\n");
        let r = apply(OLD, &p[0]).unwrap();
        assert_eq!(r.text, "a\nb\nc\nc2\nd\ne\nf\ng\n");
        assert_eq!((r.sites[0].added, r.sites[0].removed), (1, 0));
        assert_eq!((r.sites[1].added, r.sites[1].removed), (0, 1));
    }

    #[test]
    fn a_mismatch_is_refused_with_the_line() {
        let p = patch("@@ -2,2 +2,2 @@\n b\n-X\n+Y\n");
        let e = apply(OLD, &p[0]).unwrap_err();
        assert!(e.contains("does not apply near line 2"), "{e}");
    }

    #[test]
    fn crlf_files_keep_their_endings() {
        let old = "a\r\nb\r\nc\r\n";
        let p = patch("@@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n");
        assert_eq!(apply(old, &p[0]).unwrap().text, "a\r\nB\r\nc\r\n");
    }

    #[test]
    fn creating_or_deleting_a_file_is_refused() {
        let e = parse("--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1 @@\n+x\n").unwrap_err();
        assert!(e.contains("creates or deletes"), "{e}");
        assert!(parse("just text\n").is_err());
    }

    #[test]
    fn a_short_hunk_is_refused() {
        let e = parse("--- a/x\n+++ b/x\n@@ -1,3 +1,3 @@\n a\n-b\n").unwrap_err();
        assert!(e.contains("has"), "{e}");
    }

    #[test]
    fn git_headers_and_tabs_are_tolerated() {
        let d = "diff --git a/x.txt b/x.txt\nindex 1..2 100644\n--- a/x.txt\t2024-01-01\n+++ b/x.txt\t2024-01-02\n@@ -1 +1 @@\n-a\n+A\n";
        let p = parse(d).unwrap();
        assert_eq!(p[0].path, "x.txt");
        assert!(apply(OLD, &p[0]).unwrap().text.starts_with("A\nb\n"));
    }

    #[test]
    fn same_length_diff_round_trips_through_the_applier() {
        let new = OLD.replace("e\n", "E\n").replace("a\n", "A\n");
        let d = same_length("x.txt", OLD, &new);
        let p = parse(&d).unwrap();
        assert_eq!(apply(OLD, &p[0]).unwrap().text, new);
    }
}
