//! `.gitignore` handling for the `.dla/` run directory.

use std::{fs, io::Write, path::Path};

use dla_core::Error;

/// What `ensure_dla_ignored` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// There was no `.gitignore`; it was created.
    Created,
    /// The entry was appended to an existing file.
    Appended,
    /// An equivalent entry was already present; nothing changed.
    AlreadyIgnored,
}

impl Outcome {
    /// Stable label for JSON output.
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Created => "created",
            Outcome::Appended => "appended",
            Outcome::AlreadyIgnored => "already-ignored",
        }
    }
}

/// What one `.gitignore` line does to the `.dla` directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Effect {
    Ignores,
    Reincludes,
    None,
}

/// Git strips trailing whitespace (and a CR from CRLF files) but never leading spaces,
/// so `  .dla/` is a different pattern that matches nothing.
fn effect(line: &str) -> Effect {
    let t = line.trim_end_matches([' ', '\r']);
    match t {
        ".dla" | ".dla/" | "/.dla" | "/.dla/" => Effect::Ignores,
        "!.dla" | "!.dla/" | "!/.dla" | "!/.dla/" => Effect::Reincludes,
        _ => Effect::None,
    }
}

/// True if, reading the lines in order, the last entry about `.dla` ignores it. A later
/// negation cancels an earlier ignore, as git does.
pub fn ignores_dla(text: &str) -> bool {
    let mut ignored = false;
    for line in text.lines() {
        match effect(line) {
            Effect::Ignores => ignored = true,
            Effect::Reincludes => ignored = false,
            Effect::None => {}
        }
    }
    ignored
}

/// Makes sure `<project>/.gitignore` ignores `.dla/`. Idempotent.
pub fn ensure_dla_ignored(project: &Path) -> Result<Outcome, Error> {
    let path = project.join(".gitignore");
    let existing = match fs::read(&path) {
        Ok(b) => Some(b),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(Error::io(&path, e)),
    };
    let Some(bytes) = existing else {
        fs::write(&path, b".dla/\n").map_err(|e| Error::io(&path, e))?;
        return Ok(Outcome::Created);
    };
    let text = String::from_utf8_lossy(&bytes);
    if ignores_dla(&text) {
        return Ok(Outcome::AlreadyIgnored);
    }
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut add = Vec::new();
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        add.extend_from_slice(eol.as_bytes());
    }
    add.extend_from_slice(format!(".dla/{eol}").as_bytes());
    let mut f = fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .map_err(|e| Error::io(&path, e))?;
    f.write_all(&add).map_err(|e| Error::io(&path, e))?;
    Ok(Outcome::Appended)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(initial: Option<&str>) -> (Outcome, String) {
        let dir = tempfile::tempdir().unwrap();
        if let Some(t) = initial {
            fs::write(dir.path().join(".gitignore"), t).unwrap();
        }
        let o = ensure_dla_ignored(dir.path()).unwrap();
        (
            o,
            fs::read_to_string(dir.path().join(".gitignore")).unwrap(),
        )
    }

    #[test]
    fn creates_missing_file() {
        assert_eq!(run(None), (Outcome::Created, ".dla/\n".to_owned()));
    }

    #[test]
    fn appends_and_keeps_trailing_newline() {
        assert_eq!(
            run(Some("target/\n")),
            (Outcome::Appended, "target/\n.dla/\n".to_owned())
        );
        assert_eq!(
            run(Some("target/")),
            (Outcome::Appended, "target/\n.dla/\n".to_owned())
        );
        assert_eq!(
            run(Some("a\r\nb")),
            (Outcome::Appended, "a\r\nb\r\n.dla/\r\n".to_owned())
        );
        assert_eq!(run(Some("")), (Outcome::Appended, ".dla/\n".to_owned()));
    }

    #[test]
    fn recognises_existing_variants() {
        for v in [".dla", ".dla/", "/.dla", "/.dla/", ".dla/  "] {
            let text = format!("target/\n{v}\nother\n");
            assert_eq!(
                run(Some(&text)),
                (Outcome::AlreadyIgnored, text.clone()),
                "{v}"
            );
        }
    }

    #[test]
    fn does_not_mistake_similar_entries() {
        let (o, t) = run(Some(".dla-cache/\n# .dla\n"));
        assert_eq!(o, Outcome::Appended);
        assert!(t.ends_with(".dla/\n"));
    }

    #[test]
    fn second_call_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        ensure_dla_ignored(dir.path()).unwrap();
        let before = fs::read(dir.path().join(".gitignore")).unwrap();
        assert_eq!(
            ensure_dla_ignored(dir.path()).unwrap(),
            Outcome::AlreadyIgnored
        );
        assert_eq!(fs::read(dir.path().join(".gitignore")).unwrap(), before);
    }

    #[test]
    fn leading_whitespace_is_not_trimmed() {
        let (o, t) = run(Some("  .dla/\n"));
        assert_eq!(o, Outcome::Appended);
        assert_eq!(t, "  .dla/\n.dla/\n");
    }

    #[test]
    fn later_negation_cancels_an_earlier_ignore() {
        for neg in ["!.dla", "!.dla/", "!/.dla", "!/.dla/"] {
            let (o, t) = run(Some(&format!(".dla/\n{neg}\n")));
            assert_eq!(o, Outcome::Appended, "{neg}");
            assert_eq!(t, format!(".dla/\n{neg}\n.dla/\n"));
        }
        // An ignore after the negation wins again.
        let (o, _) = run(Some(".dla/\n!.dla/\n.dla\n"));
        assert_eq!(o, Outcome::AlreadyIgnored);
    }

    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// The decision matches `git check-ignore` for a file inside a run directory.
    #[test]
    fn agrees_with_git_check_ignore() {
        if !git_available() {
            eprintln!("git not available; skipping");
            return;
        }
        let cases = [
            ".dla/\n!.dla/\n",
            "  .dla/\n",
            ".dla/\n",
            ".dla\r\n",
            "/.dla/\n!/.dla\n",
            ".dla/\n!.dla/\n.dla\n",
            "target/\n",
            "",
            ".dla/  \n",
            ".dla/\t\n",
        ];
        for initial in cases {
            let dir = tempfile::tempdir().unwrap();
            let git = |args: &[&str]| {
                std::process::Command::new("git")
                    .args(args)
                    .current_dir(dir.path())
                    .output()
                    .unwrap()
            };
            assert!(git(&["init", "-q"]).status.success());
            fs::write(dir.path().join(".gitignore"), initial).unwrap();
            ensure_dla_ignored(dir.path()).unwrap();
            fs::create_dir_all(dir.path().join(".dla/zz")).unwrap();
            fs::write(dir.path().join(".dla/zz/f"), "x").unwrap();
            let ignored = git(&["check-ignore", "-q", ".dla/zz/f"]).status.success();
            assert!(ignored, "not ignored after init for {initial:?}");
        }
    }
}
