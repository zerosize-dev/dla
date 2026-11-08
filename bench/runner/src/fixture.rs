//! Reading fixtures.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{Res, model::Expected};

/// Loads and parses `<dir>/<id>/expected.toml`.
pub fn load_expected(fixtures: &Path, id: &str) -> Res<Expected> {
    let p = fixtures.join(id).join("expected.toml");
    let bytes = fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let text = String::from_utf8(bytes).map_err(|_| format!("{}: not valid UTF-8", p.display()))?;
    toml::from_str::<Expected>(&text).map_err(|e| format!("{}: {e}", p.display()))
}

/// Fixture directories (sorted by name); a missing directory has none.
pub fn fixture_dirs(fixtures: &Path) -> Res<Vec<PathBuf>> {
    if !fixtures.exists() {
        return Ok(Vec::new());
    }
    let rd = fs::read_dir(fixtures).map_err(|e| format!("{}: {e}", fixtures.display()))?;
    let mut dirs: Vec<PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    Ok(dirs)
}

/// The prompt template appended to every task for runs without a ledger.
pub const FINDINGS_INSTRUCTIONS: &str = r#"You are in a local checkout of the project. You may read, build, run and test it on this machine.
This machine has no network access. Do not change the project's source files;
scratch files outside the source tree are fine. There is nobody to answer questions: decide for yourself.

When you are done, write ./findings.json with your conclusions:
{"findings": [{"site": "path/to/file:line", "holder": "...", "region": "...", "verdict": "...",
  "mechanism": "...", "cause": "...", "evidence": "...", "discriminator": "..."}]}
- site: project-relative file:line where the memory is retained, or allocated and never released.
  Leave it "" when there is no source line (for example when only heap dumps are given).
- owner: optional. The field or container that holds the memory, as package.Class.field. Give it for every
  finding without a source line; it names the holder in the dump and is never a source location.
- holder: static/global (a static field, singleton or module global), registration-table (a listener,
  timer, callback or subscription list), container-field (a collection field of a long-lived object),
  thread-local (thread-local or async-context storage), blocked-stack (a thread, goroutine, task or
  coroutine that never finishes), closure/continuation (a captured scope), native-handle/global-ref
  (native memory, a handle or a global reference outside the managed heap), cycle-only (objects reachable
  only from each other).
- region: heap, off-heap or non-heap.
- verdict: leak-confirmed (you demonstrated the growth by running or measuring), leak-suspected (you saw it
  in the code but did not demonstrate it), or not-a-leak (growth you investigated and ruled out).
- discriminator: for not-a-leak, the observation that rules the leak out.
- cause is optional free text or a label; evidence says what you ran or measured.
List every leak you suspect and every growth you ruled out."#;

/// The note appended to the task for runs that use the dla gates.
pub const UNATTENDED_NOTE: &str = "This is an unattended run: nobody can answer questions. The benchmark runner has already given consent for privilege tiers 1-4 and for raw values.";
