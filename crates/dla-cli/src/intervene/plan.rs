//! What an intervention edits: the statements to stub or the patch to apply, checked
//! against the finding's recorded sites before anything is cloned.
//!
//! Nothing here writes. [`stub_plan`] and [`patch_plan`] read the harness's `work/` tree and
//! return the new text of every changed file with the diff; [`apply_plan`] writes it into the
//! clone.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use dla_census::{
    parse::Lang,
    statement::{StatementError, stub_statement},
    walk::is_test_path,
};

use super::diff;
use crate::{
    fail::{Failure, Res},
    intervene_cmd::codes,
};

/// Most changed lines (removed plus added) a patch may carry.
pub const MAX_CHANGED_LINES: usize = 40;
/// How far from a recorded site a hunk may sit.
pub const NEAR_LINES: usize = 10;

/// A `file:line` site, project-relative with `/` separators.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Site {
    /// File.
    pub file: String,
    /// 1-based line.
    pub line: u32,
}

impl Site {
    /// Reads `file:line`; `None` when it has no numeric line.
    pub fn parse(text: &str) -> Option<Site> {
        let (file, line) = text.trim().rsplit_once(':')?;
        let line: u32 = line.parse().ok().filter(|l| *l > 0)?;
        let file = file.trim().replace('\\', "/");
        let file = file.trim_start_matches("./");
        (!file.is_empty()).then(|| Site {
            file: file.to_owned(),
            line,
        })
    }
}

impl std::fmt::Display for Site {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

/// The sites an edit may touch.
#[derive(Debug, Clone, Default)]
pub struct Anchors {
    /// The finding under test, for the fix texts.
    pub finding: String,
    /// Recorded writers of the finding, and its site: `--stub-writer` takes these.
    pub writers: Vec<Site>,
    /// Writers, removers, the site and the owner declaration: `--patch` hunks lie near these.
    pub near: Vec<Site>,
}

/// File names the harness owns (`driver/`, `agent/`): never edited, even when generation
/// copied one into the scratch tree.
pub fn harness_owned_names(harness_dir: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for sub in ["driver", "agent"] {
        let mut stack = vec![harness_dir.join(sub)];
        while let Some(d) = stack.pop() {
            let Ok(rd) = fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if let Some(n) = p.file_name().and_then(|n| n.to_str()) {
                    out.insert(n.to_owned());
                }
            }
        }
    }
    out
}

/// One changed file.
#[derive(Debug, Clone)]
pub struct FileEdit {
    /// Path relative to `work/`.
    pub rel: String,
    /// Text before.
    pub old: String,
    /// Text after.
    pub new: String,
}

/// The checked edit.
#[derive(Debug, Clone)]
pub struct Plan {
    /// `stub-writer` or `patch`.
    pub kind: &'static str,
    /// Changed files.
    pub files: Vec<FileEdit>,
    /// The unified diff of the change.
    pub diff: String,
    /// `file:line` of every change, for the evidence.
    pub sites: Vec<String>,
    /// Removed plus added lines.
    pub changed_lines: usize,
}

fn refuse(code: &str, msg: String, missing: &str, fix: String) -> Failure {
    Failure::refuse(code, msg, missing, fix)
}

/// The path, normalized, when it stays inside `work/`.
fn safe_rel(path: &str) -> Res<String> {
    let norm = path.trim().replace('\\', "/");
    let norm = norm.trim_start_matches("./").to_owned();
    let bad = norm.is_empty()
        || norm.starts_with('/')
        || norm.contains(':')
        || norm.split('/').any(|c| c == ".." || c.is_empty());
    if bad {
        return Err(refuse(
            codes::SCOPE,
            format!("{path:?} is not a path inside the harness's work/ directory"),
            "a project-relative path",
            "name the file relative to the project root, for example src/main/Foo.java".to_owned(),
        ));
    }
    Ok(norm)
}

/// Refuses a file an intervention must not edit.
fn editable(rel: &str, owned: &BTreeSet<String>) -> Res<()> {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let first = rel.split('/').next().unwrap_or("");
    if is_test_path(rel) || matches!(first, "driver" | "agent" | ".dla") {
        return Err(refuse(
            codes::SCOPE,
            format!(
                "{rel} is a test, driver or agent file: an intervention edits project sources only"
            ),
            "a non-test project source file",
            "edit the writer in the source file, not the test or the harness".to_owned(),
        ));
    }
    if owned.contains(name) {
        return Err(refuse(
            codes::SCOPE,
            format!("{rel} has the name of a harness driver or agent file: the harness owns it"),
            "a project source file",
            "edit the project's own writer".to_owned(),
        ));
    }
    Ok(())
}

fn read_work(work: &Path, rel: &str) -> Res<String> {
    let path = work.join(rel);
    fs::read_to_string(&path).map_err(|e| {
        refuse(
            codes::SCOPE,
            format!("{rel} cannot be read in the scratch copy ({e})"),
            "an existing text file under work/",
            "check the path; the scratch copy holds the project's tracked and untracked files"
                .to_owned(),
        )
    })
}

/// Plans `--stub-writer`: every site must be a recorded writer (or the finding's site) and
/// its line a standalone call, expression or assignment statement.
pub fn stub_plan(
    work: &Path,
    owned: &BTreeSet<String>,
    anchors: &Anchors,
    requested: &[String],
) -> Res<Plan> {
    let mut by_file: Vec<(String, Vec<Site>)> = Vec::new();
    for r in requested {
        let Some(site) = Site::parse(r) else {
            return Err(Failure::usage(
                format!("{r:?} is not FILE:LINE"),
                "--stub-writer src/Foo.java:41",
                "dla intervene H1 --finding F1 --stub-writer src/Foo.java:41",
            ));
        };
        if !anchors.writers.contains(&site) {
            let have: Vec<String> = anchors
                .writers
                .iter()
                .take(6)
                .map(Site::to_string)
                .collect();
            return Err(refuse(
                codes::NOT_WRITER,
                format!("{site} is not a recorded writer of the finding or its site"),
                "a recorded writer (an annotation or a census writer) or the finding's site",
                if have.is_empty() {
                    "dla ledger annotate <F> --writer <file:line>   (no writer is recorded yet)"
                        .to_owned()
                } else {
                    format!(
                        "use one of: {}   or record it: dla ledger annotate {} --writer {site}",
                        have.join(", "),
                        anchors.finding
                    )
                },
            ));
        }
        let rel = safe_rel(&site.file)?;
        editable(&rel, owned)?;
        match by_file.iter_mut().find(|(f, _)| *f == rel) {
            Some((_, v)) => v.push(site),
            None => by_file.push((rel, vec![site])),
        }
    }
    let mut files = Vec::new();
    let mut diff_text = String::new();
    let mut sites = Vec::new();
    let mut changed = 0;
    for (rel, reqs) in by_file {
        let old = read_work(work, &rel)?;
        let Some(lang) = Lang::from_path(Path::new(&rel)) else {
            return Err(refuse(
                codes::NOT_STATEMENT,
                format!("{rel}: no parser for this file type, so no statement can be found"),
                "a Java, Kotlin, Go, Python, JavaScript, TypeScript, Rust, C or C++ file",
                "use --patch with a unified diff".to_owned(),
            ));
        };
        let mut stubs = Vec::new();
        for s in &reqs {
            let stub = stub_statement(lang, &old, s.line).map_err(|e| not_statement(s, &e))?;
            if !stubs
                .iter()
                .any(|x: &dla_census::statement::Stub| x.range == stub.range)
            {
                sites.push(format!("{}:{}", rel, stub.first_line));
                stubs.push(stub);
            }
        }
        stubs.sort_by_key(|s| std::cmp::Reverse(s.range.start));
        let mut new = old.clone();
        for s in &stubs {
            new.replace_range(s.range.clone(), &s.replacement);
        }
        changed += stubs
            .iter()
            .map(|s| (s.last_line - s.first_line + 1) as usize)
            .sum::<usize>();
        diff_text.push_str(&diff::same_length(&rel, &old, &new));
        files.push(FileEdit { rel, old, new });
    }
    sites.sort();
    Ok(Plan {
        kind: "stub-writer",
        files,
        diff: diff_text,
        sites,
        changed_lines: changed,
    })
}

fn not_statement(site: &Site, e: &StatementError) -> Failure {
    refuse(
        codes::NOT_STATEMENT,
        format!("{site}: {e}"),
        "a standalone call, expression or assignment statement on that line",
        format!(
            "stub only a statement such as `items.add(x);`; for a declaration, a condition or a return write a unified diff and use --patch (hunks within {NEAR_LINES} lines of a recorded site)"
        ),
    )
}

/// Plans `--patch`: files under `work/` only, hunks near a recorded site, at most
/// [`MAX_CHANGED_LINES`] changed lines, clean apply.
pub fn patch_plan(
    work: &Path,
    owned: &BTreeSet<String>,
    anchors: &Anchors,
    diff_text: &str,
) -> Res<Plan> {
    let patches = diff::parse(diff_text).map_err(|why| {
        refuse(
            codes::PATCH,
            format!("the diff cannot be read: {why}"),
            "a unified diff (`diff -u`, `git diff`) of existing files",
            "write the diff with --- a/<file> and +++ b/<file> headers and @@ hunks".to_owned(),
        )
    })?;
    let mut files = Vec::new();
    let mut sites = Vec::new();
    let mut changed = 0;
    for p in &patches {
        let rel = safe_rel(&p.path)?;
        editable(&rel, owned)?;
        let old = read_work(work, &rel)?;
        let applied = diff::apply(&old, p).map_err(|why| {
            refuse(
                codes::PATCH,
                format!("the diff does not apply: {why}"),
                "hunks that match the scratch copy",
                format!("regenerate the diff against the current {rel}"),
            )
        })?;
        for h in &applied.sites {
            changed += h.removed + h.added;
            let near = anchors.near.iter().any(|a| {
                a.file == rel
                    && (a.line as usize) + NEAR_LINES >= h.first
                    && (a.line as usize) <= h.last + NEAR_LINES
            });
            if !near {
                let have: Vec<String> = anchors.near.iter().take(6).map(Site::to_string).collect();
                return Err(refuse(
                    codes::SCOPE,
                    format!(
                        "the hunk at {rel}:{} is not within {NEAR_LINES} lines of a recorded writer, remover, the site or the owner declaration",
                        h.first
                    ),
                    "hunks near a recorded site of the finding",
                    if have.is_empty() {
                        "no site is recorded: dla ledger annotate <F> --writer <file:line>"
                            .to_owned()
                    } else {
                        format!("recorded sites: {}", have.join(", "))
                    },
                ));
            }
            sites.push(format!("{rel}:{}", h.first));
        }
        files.push(FileEdit {
            rel,
            old,
            new: applied.text,
        });
    }
    if changed > MAX_CHANGED_LINES {
        return Err(refuse(
            codes::TOO_LARGE,
            format!("the diff changes {changed} lines; the limit is {MAX_CHANGED_LINES}"),
            "a smaller edit",
            "reduce the diff to the lines that add to or remove from the holder".to_owned(),
        ));
    }
    if changed == 0 {
        return Err(refuse(
            codes::PATCH,
            "the diff changes nothing".to_owned(),
            "at least one added or removed line",
            "write the edit as a unified diff".to_owned(),
        ));
    }
    Ok(Plan {
        kind: "patch",
        files,
        diff: diff_text.to_owned(),
        sites,
        changed_lines: changed,
    })
}

/// Writes the plan into `work`, a clone of the tree the plan was read from. Refuses when a
/// file is not what the plan read.
pub fn apply_plan(work: &Path, plan: &Plan) -> Res<()> {
    for f in &plan.files {
        let path: PathBuf = work.join(&f.rel);
        let now =
            fs::read_to_string(&path).map_err(|e| Failure::from(dla_core::Error::io(&path, e)))?;
        if now != f.old {
            return Err(refuse(
                codes::SCOPE,
                format!("{} in the clone differs from the harness's copy", f.rel),
                "a faithful clone",
                "run dla intervene again".to_owned(),
            ));
        }
    }
    for f in &plan.files {
        let path = work.join(&f.rel);
        fs::write(&path, &f.new).map_err(|e| Failure::from(dla_core::Error::io(&path, e)))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work(files: &[(&str, &str)]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for (rel, text) in files {
            let p = d.path().join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        d
    }

    fn anchors() -> Anchors {
        let w = vec![Site::parse("a.py:3").unwrap()];
        let mut near = w.clone();
        near.push(Site::parse("a.py:20").unwrap());
        Anchors {
            finding: "F1".into(),
            writers: w,
            near,
        }
    }

    const SRC: &str = "import x\n\ndef f(v):\n    items.append(v)\n    n = 1\n";

    fn code_of(r: Res<Plan>) -> String {
        r.unwrap_err().refusal.code
    }

    #[test]
    fn sites_parse_and_normalize() {
        let s = Site::parse("./src\\A.java:41").unwrap();
        assert_eq!((s.file.as_str(), s.line), ("src/A.java", 41));
        assert!(Site::parse("A.java").is_none());
        assert!(Site::parse("A.java:0").is_none());
        assert!(Site::parse(":4").is_none());
    }

    #[test]
    fn a_stub_needs_a_recorded_writer() {
        let d = work(&[("a.py", SRC)]);
        let none = BTreeSet::new();
        let r = stub_plan(d.path(), &none, &anchors(), &["a.py:4".to_owned()]);
        assert_eq!(code_of(r), codes::NOT_WRITER);
    }

    #[test]
    fn a_stub_replaces_the_statement_and_keeps_the_lines() {
        let d = work(&[(
            "a.py",
            "import x\n\ndef f(v):\n    items.append(\n        v)\n    n = 1\n",
        )]);
        let a = Anchors {
            writers: vec![Site::parse("a.py:4").unwrap()],
            ..anchors()
        };
        let p = stub_plan(d.path(), &BTreeSet::new(), &a, &["a.py:4".to_owned()]).unwrap();
        assert_eq!(p.kind, "stub-writer");
        assert_eq!(
            p.files[0].new,
            "import x\n\ndef f(v):\n    pass\n\n    n = 1\n"
        );
        assert_eq!(p.sites, ["a.py:4"]);
        assert!(p.diff.contains("-    items.append(") && p.diff.contains("+    pass"));
        // The diff the plan carries applies with the applier.
        let patches = diff::parse(&p.diff).unwrap();
        assert_eq!(
            diff::apply(&p.files[0].old, &patches[0]).unwrap().text,
            p.files[0].new
        );
    }

    #[test]
    fn a_line_that_is_not_a_statement_is_refused() {
        let d = work(&[("a.py", SRC)]);
        let a = Anchors {
            writers: vec![Site::parse("a.py:3").unwrap()],
            ..anchors()
        };
        let r = stub_plan(d.path(), &BTreeSet::new(), &a, &["a.py:3".to_owned()]);
        assert_eq!(code_of(r), codes::NOT_STATEMENT);
    }

    #[test]
    fn two_stubs_in_one_file_are_applied_together() {
        let src = "def f(v):\n    a.append(v)\n    b.append(v)\n";
        let d = work(&[("m.py", src)]);
        let a = Anchors {
            writers: vec![
                Site::parse("m.py:2").unwrap(),
                Site::parse("m.py:3").unwrap(),
            ],
            ..anchors()
        };
        let req = [
            "m.py:2".to_owned(),
            "m.py:3".to_owned(),
            "m.py:3".to_owned(),
        ];
        let p = stub_plan(d.path(), &BTreeSet::new(), &a, &req).unwrap();
        assert_eq!(p.files[0].new, "def f(v):\n    pass\n    pass\n");
        assert_eq!(p.sites, ["m.py:2", "m.py:3"]);
    }

    #[test]
    fn tests_driver_files_and_escapes_are_out_of_reach() {
        let d = work(&[("tests/test_a.py", SRC), ("a.py", SRC)]);
        let owned: BTreeSet<String> = ["loop.py".to_owned()].into();
        for (path, why) in [
            ("tests/test_a.py", "test"),
            ("driver/loop.py", "driver"),
            ("loop.py", "name"),
            ("../x.py", "escape"),
            ("/etc/passwd", "absolute"),
        ] {
            let text = format!("--- a/{path}\n+++ b/{path}\n@@ -1 +1 @@\n-a\n+b\n");
            let a = Anchors {
                near: vec![
                    Site::parse(&format!("{path}:1")).unwrap_or(Site::parse("a.py:1").unwrap()),
                ],
                ..anchors()
            };
            let r = patch_plan(d.path(), &owned, &a, &text);
            assert_eq!(code_of(r), codes::SCOPE, "{why}");
        }
    }

    #[test]
    fn a_patch_must_lie_within_ten_lines_of_a_site() {
        let body: String = (1..=60).map(|i| format!("line {i}\n")).collect();
        let d = work(&[("a.py", &body)]);
        let none = BTreeSet::new();
        let diff_at = |n: usize| {
            format!("--- a/a.py\n+++ b/a.py\n@@ -{n},1 +{n},1 @@\n-line {n}\n+LINE {n}\n")
        };
        // Sites at 3 and 20: 13 is ten from 3, 14 is ten from 20 and 30 is ten from 20.
        assert!(patch_plan(d.path(), &none, &anchors(), &diff_at(13)).is_ok());
        assert!(patch_plan(d.path(), &none, &anchors(), &diff_at(30)).is_ok());
        assert_eq!(
            code_of(patch_plan(d.path(), &none, &anchors(), &diff_at(31))),
            codes::SCOPE
        );
        assert_eq!(
            code_of(patch_plan(d.path(), &none, &anchors(), &diff_at(45))),
            codes::SCOPE
        );
    }

    #[test]
    fn a_patch_is_limited_to_forty_changed_lines() {
        let body: String = (1..=60).map(|i| format!("line {i}\n")).collect();
        let d = work(&[("a.py", &body)]);
        let hunk = |lines: usize| {
            let mut s = String::from("--- a/a.py\n+++ b/a.py\n");
            s.push_str(&format!("@@ -2,{lines} +2,{lines} @@\n"));
            for i in 0..lines / 2 {
                s.push_str(&format!("-line {}\n+LINE {}\n", i + 2, i + 2));
            }
            for i in lines / 2..lines {
                s.push_str(&format!(" line {}\n", i + 2));
            }
            s
        };
        let none = BTreeSet::new();
        // 20 removed + 20 added = 40.
        let p = patch_plan(d.path(), &none, &anchors(), &hunk(40)).unwrap();
        assert_eq!(p.changed_lines, 40);
        // One more pair: 21 + 21 = 42.
        let r = patch_plan(d.path(), &none, &anchors(), &hunk(42));
        assert_eq!(code_of(r), codes::TOO_LARGE);
    }

    #[test]
    fn a_diff_that_does_not_apply_or_changes_nothing_is_refused() {
        let d = work(&[("a.py", SRC)]);
        let none = BTreeSet::new();
        let stale = "--- a/a.py\n+++ b/a.py\n@@ -4,1 +4,1 @@\n-    other(v)\n+    pass\n";
        assert_eq!(
            code_of(patch_plan(d.path(), &none, &anchors(), stale)),
            codes::PATCH
        );
        assert_eq!(
            code_of(patch_plan(d.path(), &none, &anchors(), "nonsense")),
            codes::PATCH
        );
        let same = "--- a/a.py\n+++ b/a.py\n@@ -4,1 +4,1 @@\n     items.append(v)\n";
        assert_eq!(
            code_of(patch_plan(d.path(), &none, &anchors(), same)),
            codes::PATCH
        );
    }

    #[test]
    fn apply_plan_writes_only_a_faithful_clone() {
        let d = work(&[("a.py", SRC)]);
        let a = Anchors {
            writers: vec![Site::parse("a.py:4").unwrap()],
            ..anchors()
        };
        let p = stub_plan(d.path(), &BTreeSet::new(), &a, &["a.py:4".to_owned()]).unwrap();
        let clone = work(&[("a.py", "something else\n")]);
        assert!(apply_plan(clone.path(), &p).is_err());
        assert_eq!(
            fs::read_to_string(clone.path().join("a.py")).unwrap(),
            "something else\n"
        );
        let same = work(&[("a.py", SRC)]);
        apply_plan(same.path(), &p).unwrap();
        assert_eq!(
            fs::read_to_string(same.path().join("a.py")).unwrap(),
            p.files[0].new
        );
    }
}
