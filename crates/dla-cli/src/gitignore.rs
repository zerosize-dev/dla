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

/// True if the line already ignores the `.dla` directory.
pub fn ignores_dla(line: &str) -> bool {
    matches!(line.trim(), ".dla" | ".dla/" | "/.dla" | "/.dla/")
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
    if text.lines().any(ignores_dla) {
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
        for v in [".dla", ".dla/", "/.dla", "/.dla/", "  .dla/  "] {
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
}
