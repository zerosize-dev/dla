//! Errors, cleanup strings, the target's identity and small file helpers shared by the
//! attach backends.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;

/// Why an attach step did not happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// A tool the backend needs is not installed.
    ToolMissing,
    /// The target (or the box) does not meet a precondition.
    NotReady,
    /// The tool ran and failed.
    Failed,
}

/// A failed attach step, with the item that is missing and the fixing command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachError {
    /// Category.
    pub kind: ErrorKind,
    /// What happened.
    pub message: String,
    /// The missing item.
    pub missing: String,
    /// The command that fixes it.
    pub fix: String,
}

impl AttachError {
    /// A missing tool.
    pub fn tool_missing(message: impl Into<String>, missing: &str, fix: &str) -> Self {
        Self {
            kind: ErrorKind::ToolMissing,
            message: message.into(),
            missing: missing.to_owned(),
            fix: fix.to_owned(),
        }
    }

    /// A precondition that does not hold.
    pub fn not_ready(message: impl Into<String>, missing: &str, fix: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::NotReady,
            message: message.into(),
            missing: missing.to_owned(),
            fix: fix.into(),
        }
    }

    /// A tool that ran and failed.
    pub fn failed(message: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Failed,
            message: message.into(),
            missing: "a successful run of the tool".to_owned(),
            fix: fix.into(),
        }
    }
}

impl std::fmt::Display for AttachError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AttachError {}

/// A cleanup action in the grammar `dla run abort` executes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cleanup {
    /// `rm <absolute path>`: delete a file DLA created.
    Rm(PathBuf),
    /// `run <JSON array argv>`: run a command (10 s timeout).
    Run(Vec<String>),
    /// `note <text>`: nothing to execute.
    Note(String),
}

impl Cleanup {
    /// The action as the string stored in the process registry.
    pub fn render(&self) -> String {
        match self {
            Self::Rm(p) => format!("rm {}", p.display()),
            Self::Run(argv) => format!(
                "run {}",
                serde_json::to_string(argv).unwrap_or_else(|_| "[]".to_owned())
            ),
            Self::Note(t) => format!("note {t}"),
        }
    }

    /// Parses a stored action; `None` when it is not one of the three forms.
    pub fn parse(text: &str) -> Option<Self> {
        if let Some(p) = text.strip_prefix("rm ") {
            return Path::new(p)
                .is_absolute()
                .then(|| Self::Rm(PathBuf::from(p)));
        }
        if let Some(j) = text.strip_prefix("run ") {
            let argv: Vec<String> = serde_json::from_str(j).ok()?;
            return (!argv.is_empty()).then_some(Self::Run(argv));
        }
        text.strip_prefix("note ").map(|t| Self::Note(t.to_owned()))
    }

    /// Whether this action has anything to execute.
    pub fn executable(&self) -> bool {
        !matches!(self, Self::Note(_))
    }
}

/// What DLA knows about the target before it touches it.
#[derive(Debug, Clone, Serialize)]
pub struct TargetInfo {
    /// Process ID.
    pub pid: u32,
    /// Resolved executable path, or the first argument when the link is unreadable.
    pub exe: Option<String>,
    /// Real user ID.
    pub uid: Option<u32>,
    /// Real group ID followed by the supplementary groups.
    pub gids: Vec<u32>,
    /// Start time, UTC `YYYY-MM-DDTHH:MM:SSZ`.
    pub start_time: Option<String>,
    /// The argument vector.
    pub args: Vec<String>,
    /// Resident memory in bytes.
    pub rss: Option<u64>,
}

impl TargetInfo {
    /// The base name of the executable (`java`, `node`, `python3`).
    pub fn exe_name(&self) -> String {
        let from = self
            .exe
            .as_deref()
            .or_else(|| self.args.first().map(String::as_str))
            .unwrap_or("");
        crate::proc::exe_name(from).to_owned()
    }
}

/// Whether the process `target.pid` is still the process that was inspected: alive, and (when
/// the start time was readable) with the same start time. A reused pid reads as gone.
pub fn same_process(target: &TargetInfo) -> bool {
    if !crate::os::is_alive(target.pid) {
        return false;
    }
    match (
        target.start_time.as_deref(),
        crate::os::process_start_time(target.pid),
    ) {
        (Some(was), Some(now)) => crate::os::same_start_time(was, &now),
        (Some(_), None) => false,
        (None, _) => true,
    }
}

/// Reads the identity of `pid`; `None` when it is not running.
pub fn inspect_target(pid: u32) -> Option<TargetInfo> {
    if !crate::os::is_alive(pid) {
        return None;
    }
    let args = crate::proc::command_line(pid).unwrap_or_default();
    let exe = std::fs::read_link(format!("/proc/{pid}/exe"))
        .ok()
        .map(|p| p.display().to_string())
        .map(|s| s.trim_end_matches(" (deleted)").to_owned())
        .or_else(|| args.first().cloned());
    let (uid, gids) = std::fs::read_to_string(format!("/proc/{pid}/status"))
        .map(|t| parse_status_ids(&t))
        .unwrap_or_default();
    let rss = crate::os::sample(pid)
        .ok()
        .and_then(|s| s.values.get(crate::os::RSS_COLUMN).copied())
        .map(|v| v as u64);
    Some(TargetInfo {
        pid,
        exe,
        uid,
        gids,
        start_time: crate::os::process_start_time(pid),
        args,
        rss,
    })
}

/// The real uid and the group list (real gid first) of a `/proc/<pid>/status` text.
pub fn parse_status_ids(status: &str) -> (Option<u32>, Vec<u32>) {
    let field = |name: &str| {
        status
            .lines()
            .find_map(|l| l.strip_prefix(name))
            .map(|r| r.split_whitespace().map(str::to_owned).collect::<Vec<_>>())
    };
    let uid = field("Uid:").and_then(|f| f.first()?.parse().ok());
    let mut gids = Vec::new();
    if let Some(g) = field("Gid:").and_then(|f| f.first()?.parse().ok()) {
        gids.push(g);
    }
    if let Some(groups) = field("Groups:") {
        gids.extend(groups.iter().filter_map(|g| g.parse::<u32>().ok()));
    }
    (uid, gids)
}

/// The environment of `pid` (readable for the same user or root); empty when it is not.
pub fn proc_environ(pid: u32) -> BTreeMap<String, String> {
    let Ok(raw) = std::fs::read(format!("/proc/{pid}/environ")) else {
        return BTreeMap::new();
    };
    raw.split(|&b| b == 0)
        .filter_map(|kv| {
            let s = String::from_utf8_lossy(kv);
            let (k, v) = s.split_once('=')?;
            Some((k.to_owned(), v.to_owned()))
        })
        .collect()
}

/// Whether a file with this owner and mode grants the `want` bits (`0o2` write, `0o1` search)
/// to `uid` with groups `gids`. Root passes.
pub fn mode_grants(mode: u32, owner: u32, group: u32, uid: u32, gids: &[u32], want: u32) -> bool {
    if uid == 0 {
        return true;
    }
    let bits = if uid == owner {
        (mode >> 6) & 7
    } else if gids.contains(&group) {
        (mode >> 3) & 7
    } else {
        mode & 7
    };
    bits & want == want
}

/// Checks that a process with `uid`/`gids` can create files in `dir`: write and search
/// permission on the directory and search permission on every ancestor. Returns the first
/// problem as text.
pub fn dir_writable_by(dir: &Path, uid: u32, gids: &[u32]) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let mut cur = Some(dir);
        let mut first = true;
        while let Some(p) = cur {
            let m = std::fs::metadata(p).map_err(|e| format!("{}: {e}", p.display()))?;
            let want = if first { 0o3 } else { 0o1 };
            if !mode_grants(m.mode(), m.uid(), m.gid(), uid, gids, want) {
                return Err(format!(
                    "{} (mode {:o}, owner {}) does not grant {} to user {uid}",
                    p.display(),
                    m.mode() & 0o7777,
                    m.uid(),
                    if first { "write" } else { "search" }
                ));
            }
            first = false;
            cur = p.parent().filter(|q| !q.as_os_str().is_empty());
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (dir, uid, gids);
        Ok(())
    }
}

/// The current user's ID (`None` off Linux).
pub fn own_uid() -> Option<u32> {
    if let Ok(t) = std::fs::read_to_string("/proc/self/status") {
        return parse_status_ids(&t).0;
    }
    // No /proc (macOS): ask `id -u`.
    let out = std::process::Command::new("id")
        .arg("-u")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// Waits until `path` exists and its size has not changed for `quiet`, at most `max`.
/// Returns the final size, or `None` when the file never appeared.
pub fn wait_stable(path: &Path, max: Duration, quiet: Duration) -> Option<u64> {
    let deadline = Instant::now() + max;
    let mut last: Option<(u64, Instant)> = None;
    loop {
        let size = std::fs::metadata(path).ok().map(|m| m.len());
        match (size, last) {
            (Some(s), Some((prev, since))) if s == prev => {
                if since.elapsed() >= quiet && s > 0 {
                    return Some(s);
                }
            }
            (Some(s), _) => last = Some((s, Instant::now())),
            (None, _) => {}
        }
        if Instant::now() >= deadline {
            return last.map(|(s, _)| s);
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// `text` cut to `max` characters with a trailing `...`.
pub fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut t: String = text.chars().take(max).collect();
    t.push_str("...");
    t
}

/// The first line of `stderr` or `stdout` that says what went wrong.
pub fn first_error_line(stderr: &str, stdout: &str) -> String {
    [stderr, stdout]
        .iter()
        .flat_map(|s| s.lines())
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("Picked up "))
        .unwrap_or("it exited with an error")
        .to_owned()
}

/// Files DLA writes into the attach directory besides the main artifact.
const RESERVED_NAMES: [&str; 2] = ["preflight.json", "heaptrack.log"];
/// Name endings of derived artifacts (`memray stats`, `heaptrack_print`).
const RESERVED_SUFFIXES: [&str; 2] = [".stats.json", "-print.txt"];

/// Rejects names that are not a plain file name; returns it with `default_ext` appended when
/// it has no extension.
pub fn output_name(name: &str, default_ext: &str) -> Result<String, String> {
    let ok = !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        && !name.contains("..");
    if !ok {
        return Err(format!(
            "{name:?} is not a plain file name (letters, digits, '.', '_' and '-', at most 64 characters)"
        ));
    }
    let full = if name.contains('.') || default_ext.is_empty() {
        name.to_owned()
    } else {
        format!("{name}.{default_ext}")
    };
    let lower = full.to_ascii_lowercase();
    if RESERVED_NAMES.contains(&lower.as_str())
        || RESERVED_SUFFIXES.iter().any(|s| lower.ends_with(s))
    {
        return Err(format!(
            "{name:?} is a file DLA writes itself into the attach directory; pick another name"
        ));
    }
    Ok(full)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_strings_round_trip_through_the_grammar() {
        let cases = [
            Cleanup::Rm(PathBuf::from("/tmp/heaptrack_fifo42")),
            Cleanup::Run(vec!["memray".into(), "detach".into(), "123".into()]),
            Cleanup::Note("inspector stays open until the process exits".into()),
        ];
        for c in cases {
            assert_eq!(Cleanup::parse(&c.render()).as_ref(), Some(&c));
        }
        assert_eq!(
            Cleanup::Run(vec!["memray".into(), "detach".into(), "9".into()]).render(),
            r#"run ["memray","detach","9"]"#
        );
        assert_eq!(Cleanup::parse("rm relative/path"), None);
        assert_eq!(Cleanup::parse("run []"), None);
        assert_eq!(Cleanup::parse("run not json"), None);
        assert_eq!(Cleanup::parse("remove /x"), None);
        assert!(!Cleanup::Note("x".into()).executable());
    }

    #[test]
    fn status_ids_take_real_uid_and_all_groups() {
        let s = "Name:\tjava\nUid:\t1000\t1000\t1000\t1000\nGid:\t100\t100\t100\t100\nGroups:\t100 27 44\n";
        assert_eq!(parse_status_ids(s), (Some(1000), vec![100, 100, 27, 44]));
        assert_eq!(parse_status_ids("Name:\tx\n"), (None, vec![]));
    }

    #[test]
    fn mode_bits_follow_owner_group_other() {
        // rwx------ owned by 5
        assert!(mode_grants(0o700, 5, 9, 5, &[], 0o3));
        assert!(!mode_grants(0o700, 5, 9, 6, &[9], 0o3));
        // rwxr-x--- group 9 may search but not write
        assert!(mode_grants(0o750, 5, 9, 6, &[9], 0o1));
        assert!(!mode_grants(0o750, 5, 9, 6, &[9], 0o2));
        // other rwx
        assert!(mode_grants(0o707, 5, 9, 7, &[], 0o3));
        assert!(mode_grants(0o000, 5, 9, 0, &[], 0o3), "root passes");
    }

    #[cfg(unix)]
    #[test]
    fn a_world_unwritable_dir_is_reported_for_another_user() {
        use std::os::unix::fs::PermissionsExt;
        let d = std::env::temp_dir().join(format!("dla-writable-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o700)).unwrap();
        let me = own_uid().unwrap();
        assert!(dir_writable_by(&d, me, &[]).is_ok());
        if me != 0 {
            assert!(dir_writable_by(&d, me + 1, &[]).is_err());
        }
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn output_names_are_plain_file_names() {
        assert_eq!(output_name("before", "hprof").unwrap(), "before.hprof");
        assert_eq!(output_name("a.b.hprof", "hprof").unwrap(), "a.b.hprof");
        for bad in ["", "../x", "a/b", ".hidden", "a b", "x..y"] {
            assert!(output_name(bad, "bin").is_err(), "{bad}");
        }
    }

    #[test]
    fn output_names_cannot_take_the_files_dla_writes() {
        for bad in [
            "preflight.json",
            "Preflight.JSON",
            "heaptrack.log",
            "a.stats.json",
            "x-print.txt",
        ] {
            let e = output_name(bad, "bin").unwrap_err();
            assert!(e.contains("writes itself"), "{bad}: {e}");
        }
        assert!(output_name("preflight", "json").is_err());
        assert!(output_name("before.json", "json").is_ok());
    }

    #[test]
    fn clip_keeps_short_text() {
        assert_eq!(clip("abc", 5), "abc");
        assert_eq!(clip("abcdef", 3), "abc...");
    }
}
