//! The hook entry points of `dla gate`: `stop-hook`, `read-hook`, `write-hook` and
//! `bash-hook`.
//!
//! Claude Code runs these as command hooks and pipes the event JSON on stdin. They never
//! take `--run`: the project comes from the event's `cwd` (the git toplevel, else the
//! directory itself) and the run from `<project>/.dla/current`. With no active run they
//! allow silently.
//!
//! The PreToolUse gates fail closed: input that cannot be read, or a panic, ends in exit
//! 2 with a message. The stop hooks fail open: a failure prints a note on stderr and exits
//! 0, so a broken hook never traps a session. Hooks are a guard rail. `Bash`, `Grep` and
//! `@file` mentions reach the same files by other routes, and the guarantee is the
//! integrity seals inside `dla` (see `dla_core::integrity`).

use std::{
    io::Read,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
};

use dla_core::{
    RunState, RunView,
    gate::{self, GateTarget, Missing},
    ids::valid_worker_name,
    state::{self, DLA_DIR, RunStatus, Stage},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::git;

/// Largest hook input read from stdin. A `Write` of a very large file is the only thing
/// that gets near it; anything bigger is refused as malformed.
const MAX_INPUT: u64 = 32 * 1024 * 1024;

/// Consecutive failing stop attempts with an unchanged digest before the hook lets the
/// session stop and stamps the run (or the worker) instead.
pub const STOP_LIMIT: u32 = 3;

/// The longest block reason handed back to the model, in bytes.
const MAX_REASON: usize = 1000;

/// File names the read gate protects until the classify gate has passed.
const LOCKED_SUFFIXES: [&str; 4] = [
    "-archetypes.md",
    "known-leaks.md",
    "archetypes.toml",
    "known-leaks.toml",
];

/// Which hook entry point to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hook {
    /// Stop (`subagent` false) or SubagentStop (`subagent` true).
    Stop {
        /// SubagentStop: gate the worker named in the subagent transcript.
        subagent: bool,
    },
    /// PreToolUse for Read, Grep, Glob (and Bash text).
    Read,
    /// PreToolUse for Write, Edit, NotebookEdit (and Bash text).
    Write,
    /// PreToolUse for Bash: both checks.
    Bash,
}

impl Hook {
    fn pre_tool_use(self) -> bool {
        !matches!(self, Hook::Stop { .. })
    }
}

/// What a hook prints and how it exits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookOut {
    /// Printed on stdout (empty for an allow).
    pub stdout: String,
    /// Printed on stderr.
    pub stderr: String,
    /// Process exit code.
    pub code: u8,
}

impl HookOut {
    fn allow() -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            code: 0,
        }
    }

    fn deny(reason: &str) -> Self {
        Self {
            stdout: json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason,
                }
            })
            .to_string(),
            stderr: String::new(),
            code: 0,
        }
    }

    fn block(reason: &str) -> Self {
        Self {
            stdout: json!({"decision": "block", "reason": reason}).to_string(),
            stderr: String::new(),
            code: 0,
        }
    }

    /// PreToolUse failure: exit 2, the message on stderr is shown to the model.
    fn fail_closed(why: &str) -> Self {
        Self {
            stdout: String::new(),
            stderr: format!(
                "dla gate: {why}. The call is blocked because the gate cannot tell whether it is allowed."
            ),
            code: 2,
        }
    }

    /// Stop failure: a note, and the session may stop.
    fn fail_open(why: &str) -> Self {
        Self {
            stdout: String::new(),
            stderr: format!("dla gate: {why}; the stop is allowed"),
            code: 0,
        }
    }
}

/// The fields of the hook event that the gates read. Unknown fields are ignored.
#[derive(Debug, Default, Deserialize)]
struct Input {
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    tool_input: Option<Value>,
    #[serde(default)]
    agent_transcript_path: Option<String>,
    /// Set on every hook call made inside a subagent (absent in the main session).
    #[serde(default)]
    agent_id: Option<String>,
    /// The main session's transcript. A subagent's own transcript sits next to it, see
    /// [`subagent_transcript`].
    #[serde(default)]
    transcript_path: Option<String>,
}

impl Input {
    fn field(&self, name: &str) -> Option<&str> {
        self.tool_input.as_ref()?.get(name)?.as_str()
    }
}

/// Runs a hook: reads stdin, never panics out. `project_arg` is the global `--project`.
pub fn run(project_arg: Option<&Path>, hook: Hook) -> HookOut {
    let mut text = String::new();
    let read = std::io::stdin()
        .lock()
        .take(MAX_INPUT + 1)
        .read_to_string(&mut text);
    run_with(project_arg, hook, read.map(|_| text))
}

/// [`run`] with the stdin text (or the read error) given, for tests.
pub fn run_with(project_arg: Option<&Path>, hook: Hook, stdin: std::io::Result<String>) -> HookOut {
    let result = catch_unwind(AssertUnwindSafe(|| inner(project_arg, hook, stdin)));
    match result {
        Ok(Ok(out)) => out,
        Ok(Err(why)) => fail(hook, &why),
        Err(p) => {
            let msg = p
                .downcast_ref::<&str>()
                .map(|s| (*s).to_owned())
                .or_else(|| p.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_owned());
            fail(hook, &format!("internal error ({msg})"))
        }
    }
}

fn fail(hook: Hook, why: &str) -> HookOut {
    if hook.pre_tool_use() {
        HookOut::fail_closed(why)
    } else {
        HookOut::fail_open(why)
    }
}

fn inner(
    project_arg: Option<&Path>,
    hook: Hook,
    stdin: std::io::Result<String>,
) -> Result<HookOut, String> {
    if std::env::var_os("DLA_HOOK_TEST_PANIC").is_some() {
        // Lets the tests prove the panic path of the gates (never set outside tests).
        panic!("forced by DLA_HOOK_TEST_PANIC");
    }
    let text = stdin.map_err(|e| format!("cannot read the hook input: {e}"))?;
    if text.len() as u64 > MAX_INPUT {
        return Err("the hook input is larger than 32 MB".to_owned());
    }
    let input: Input =
        serde_json::from_str(&text).map_err(|e| format!("malformed hook input: {e}"))?;
    match hook {
        Hook::Stop { subagent } => stop(project_arg, &input, subagent),
        Hook::Read => pre_tool(project_arg, &input, false, true),
        Hook::Write => pre_tool(project_arg, &input, true, false),
        Hook::Bash => pre_tool(project_arg, &input, true, true),
    }
}

// ----------------------------------------------------------------- locating

/// A run found from the hook's `cwd`.
struct Found {
    project: PathBuf,
    run_dir: PathBuf,
    /// `None` when `state.json` cannot be read.
    state: Option<RunState>,
}

impl Found {
    /// A run in progress. An unreadable state counts as active for the write gate (fail
    /// closed) and is handled by the callers that need the state.
    fn active(&self) -> bool {
        self.state
            .as_ref()
            .is_none_or(|s| s.status == RunStatus::Active)
    }
}

fn project_candidates(project_arg: Option<&Path>, input: &Input) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(p) = project_arg {
        out.push(std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf()));
        return out;
    }
    let cwd = input
        .cwd
        .as_deref()
        .filter(|c| !c.trim().is_empty())
        .map(|c| PathBuf::from(c.replace('\\', "/")))
        .or_else(|| std::env::current_dir().ok());
    if let Some(cwd) = cwd {
        out.push(git::find_root(&cwd).unwrap_or(cwd));
    }
    if let Some(env) = std::env::var_os("CLAUDE_PROJECT_DIR").filter(|v| !v.is_empty()) {
        let p = PathBuf::from(env);
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// The project and current run for the event, or `None` when there is no run.
fn find_run(project_arg: Option<&Path>, input: &Input) -> Result<Option<Found>, String> {
    for project in project_candidates(project_arg, input) {
        let run_id = match state::read_current(&project) {
            Ok(Some(id)) => id,
            Ok(None) => continue,
            Err(e) => return Err(format!("cannot read .dla/current: {e}")),
        };
        if run_id.contains(['/', '\\']) || run_id.starts_with('.') {
            continue;
        }
        let run_dir = state::run_dir(&project, &run_id);
        if !run_dir.is_dir() {
            continue;
        }
        let state = RunState::load(&run_dir).ok();
        return Ok(Some(Found {
            project,
            run_dir,
            state,
        }));
    }
    Ok(None)
}

// ------------------------------------------------------------ path handling

fn slash(s: &str) -> String {
    s.replace('\\', "/")
}

fn has_drive(p: &str) -> bool {
    let b = p.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

fn is_absolute(p: &str) -> bool {
    p.starts_with('/') || has_drive(p)
}

/// Git Bash and WSL spell `C:/x` as `/c/x` or `/mnt/c/x`; both become `c:/x`.
fn unmsys(p: &str) -> String {
    for prefix in ["/mnt/", "/"] {
        if let Some(rest) = p.strip_prefix(prefix) {
            let b = rest.as_bytes();
            if b.len() >= 2
                && b[0].is_ascii_alphabetic()
                && b[1] == b'/'
                && (prefix == "/mnt/" || cfg!(windows))
            {
                return format!("{}:{}", &rest[..1], &rest[1..]);
            }
        }
    }
    p.to_owned()
}

/// Resolves `.` and `..` lexically in a `/`-separated absolute path.
fn lexical(p: &str) -> String {
    let (root, rest) = if has_drive(p) {
        (&p[..2], &p[2..])
    } else {
        ("", p)
    };
    let mut parts: Vec<&str> = Vec::new();
    for c in rest.split('/') {
        match c {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            c => parts.push(c),
        }
    }
    format!("{root}/{}", parts.join("/"))
}

/// Resolves symlinks in the longest existing prefix of `p` and appends the rest.
fn real(p: &str) -> String {
    let mut rest: Vec<String> = Vec::new();
    let mut cur = PathBuf::from(p);
    loop {
        if let Ok(c) = std::fs::canonicalize(&cur) {
            let mut out = slash(&c.to_string_lossy());
            if let Some(stripped) = out.strip_prefix("//?/") {
                out = stripped.to_owned();
            }
            out.truncate(out.trim_end_matches('/').len());
            for r in rest.iter().rev() {
                out.push('/');
                out.push_str(r);
            }
            if out.is_empty() {
                out.push('/');
            }
            return out;
        }
        let Some(name) = cur.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            return p.to_owned();
        };
        rest.push(name);
        if !cur.pop() {
            return p.to_owned();
        }
    }
}

/// A path as the gates compare it: `/` separators, absolute against `cwd`, `..` and
/// symlinks resolved. On Windows (and for drive paths) the comparison ignores case.
fn normalize(path: &str, cwd: &str) -> String {
    let mut p = slash(path.trim());
    if let Some(s) = p.strip_prefix("//?/") {
        p = s.to_owned();
    }
    let p = unmsys(&p);
    let abs = if is_absolute(&p) {
        p
    } else {
        format!("{}/{p}", slash(cwd).trim_end_matches('/'))
    };
    let abs = lexical(&unmsys(&abs));
    fold(&real(&abs))
}

fn fold(p: &str) -> String {
    if cfg!(windows) || has_drive(p) {
        p.to_lowercase()
    } else {
        p.to_owned()
    }
}

fn base_name(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

// ------------------------------------------------------------- PreToolUse

fn cwd_of(input: &Input, project: &Path) -> String {
    input
        .cwd
        .clone()
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| slash(&project.to_string_lossy()))
}

fn pre_tool(
    project_arg: Option<&Path>,
    input: &Input,
    write: bool,
    read: bool,
) -> Result<HookOut, String> {
    let Some(found) = find_run(project_arg, input)? else {
        return Ok(HookOut::allow());
    };
    let tool = input.tool_name.as_deref().unwrap_or("");
    if write
        && found.active()
        && let Some(why) = check_write(input, tool, &found)
    {
        return Ok(HookOut::deny(&why));
    }
    if read
        && let Some(state) = found
            .state
            .as_ref()
            .filter(|s| s.status == RunStatus::Active)
        && !state.stage_passed(Stage::Classify)
        && !worker_passed_classify(input, state)
        && let Some(why) = check_read(input, tool, &found)
    {
        return Ok(HookOut::deny(&why));
    }
    Ok(HookOut::allow())
}

/// The reason to deny a write, or `None`.
fn check_write(input: &Input, tool: &str, found: &Found) -> Option<String> {
    match tool {
        "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => {
            let path = input
                .field("file_path")
                .or_else(|| input.field("notebook_path"))
                .or_else(|| input.field("path"))?;
            protected_path(path, input, found)
        }
        "Bash" => bash_write(input.field("command")?, found),
        _ => None,
    }
}

fn dla_dir_of(found: &Found) -> String {
    let project = slash(&found.project.to_string_lossy());
    fold(&real(&lexical(&unmsys(&format!(
        "{}/{DLA_DIR}",
        project.trim_end_matches('/')
    )))))
}

/// Denies a path under `<project>/.dla/` that names a protected run file.
fn protected_path(path: &str, input: &Input, found: &Found) -> Option<String> {
    let norm = normalize(path, &cwd_of(input, &found.project));
    let dla_dir = dla_dir_of(found);
    let rest = norm.strip_prefix(&format!("{dla_dir}/"))?;
    let name = base_name(rest);
    let comps: Vec<&str> = rest.split('/').collect();
    let in_integrity = comps.len() >= 2 && comps[1] == "integrity";
    let protected = matches!(
        name,
        "report.md" | "state.json" | "coverage.json" | "current"
    ) || name.starts_with("ledger")
        || in_integrity;
    protected.then(|| {
        format!(
            "{name} under .dla/ is written by dla only. Use the dla commands (dla ledger ..., dla report, dla run ...); a file edited by hand fails the integrity check and the report says INCOMPLETE."
        )
    })
}

// ----------------------------------------------------------- Bash, writing

fn word_basename(w: &str) -> String {
    let w = w.trim_matches(['"', '\'']);
    let b = w.rsplit(['/', '\\']).next().unwrap_or(w);
    b.to_ascii_lowercase()
}

fn is_dla_word(w: &str) -> bool {
    matches!(word_basename(w).as_str(), "dla" | "dla.exe")
}

/// Splits a command line at `&&`, `||`, `;`, `|` and newlines (quotes are not tracked:
/// this is a best-effort check).
fn segments(cmd: &str) -> Vec<&str> {
    cmd.split(['\n', ';', '|', '&'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

/// The first word of a segment, past `VAR=value` assignments.
fn first_word(seg: &str) -> &str {
    seg.split_whitespace()
        .find(|w| !(w.contains('=') && !w.starts_with(['/', '.', '"', '\''])))
        .unwrap_or("")
}

/// True when the text names the `.dla` directory: `.dla/`, `.dla\`, or `.dla` as a whole word.
fn mentions_dla(s: &str) -> bool {
    let mut from = 0;
    while let Some(i) = s[from..].find(".dla") {
        let at = from + i;
        let before_ok = s[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '-'));
        let after = s[at + 4..].chars().next();
        let after_ok = after.is_none_or(|c| matches!(c, '/' | '\\' | '\'' | '"' | ' ' | '\t'));
        if before_ok && after_ok {
            return true;
        }
        from = at + 4;
    }
    false
}

/// Targets of `>` and `>>` redirections in a segment.
fn redirect_targets(seg: &str) -> Vec<String> {
    let bytes = seg.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'>' {
            let mut j = i + 1;
            if j < bytes.len() && bytes[j] == b'>' {
                j += 1;
            }
            // `2>&1` and `>&2` duplicate a descriptor, they write no file.
            if j < bytes.len() && bytes[j] == b'&' {
                i = j + 1;
                continue;
            }
            let rest = seg[j..].trim_start();
            let target: String = rest
                .trim_start_matches(['"', '\''])
                .chars()
                .take_while(|c| !c.is_whitespace() && !matches!(c, ';' | '|' | '&' | '"' | '\''))
                .collect();
            if !target.is_empty() {
                out.push(target);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

const WRITE_WORDS: [&str; 9] = [
    "tee", "cp", "mv", "rm", "truncate", "dd", "install", "ln", "rsync",
];
const INTERPRETERS: [&str; 8] = [
    "python", "python3", "python2", "node", "perl", "ruby", "php", "pwsh",
];
const SCRIPT_WRITES: [&str; 12] = [
    "write", "unlink", "remove", "rename", "truncate", "shutil", "system(", "append", "'w'",
    "\"w\"", "'a'", "\"a\"",
];

/// Best-effort check of a Bash command that names `.dla/` together with a write form.
/// Commands whose first word is `dla` (or the shim) are exempt, except that a redirection
/// out of them into `.dla/` is not.
fn bash_write(cmd: &str, found: &Found) -> Option<String> {
    let _ = found;
    for seg in segments(cmd) {
        let from_dla = is_dla_word(first_word(seg));
        let bad_redirect = redirect_targets(seg).iter().any(|t| mentions_dla(t));
        let mut bad = bad_redirect;
        if !from_dla && !bad && mentions_dla(seg) {
            let lower = seg.to_ascii_lowercase();
            let words: Vec<String> = lower.split_whitespace().map(word_basename).collect();
            bad = words.iter().any(|w| WRITE_WORDS.contains(&w.as_str()))
                || (words.iter().any(|w| w == "sed")
                    && lower
                        .split_whitespace()
                        .any(|w| w.starts_with("-i") || w == "--in-place"))
                || (words.iter().any(|w| INTERPRETERS.contains(&w.as_str()))
                    && SCRIPT_WRITES.iter().any(|k| lower.contains(k)));
        }
        if bad {
            return Some(
                "This command writes under .dla/, which dla writes by itself. Use the dla commands (dla ledger ..., dla report, dla run ...). This check is best effort; the seals dla keeps on the run files make an edit visible and the report says INCOMPLETE."
                    .to_owned(),
            );
        }
    }
    None
}

// ------------------------------------------------------------ read gate

fn locked_name(s: &str) -> bool {
    let lower = s.to_ascii_lowercase().replace('\\', "/");
    LOCKED_SUFFIXES.iter().any(|n| lower.contains(n))
}

/// The transcript of the subagent a PreToolUse call runs in. SubagentStop names it
/// (`agent_transcript_path`); a PreToolUse call carries only `agent_id` and the main
/// session's `transcript_path`, and the subagent's file is
/// `<dir of the main transcript>/<its stem>/subagents/agent-<agent_id>.jsonl` (the layout
/// the Claude Code reference shows for SubagentStop; see docs/VERIFY.md).
fn subagent_transcript(input: &Input) -> Option<PathBuf> {
    if let Some(p) = input.agent_transcript_path.as_deref() {
        return Some(expand_home(p));
    }
    let id = input.agent_id.as_deref().filter(|i| !i.is_empty())?;
    let main = expand_home(input.transcript_path.as_deref()?);
    let stem = main.file_stem()?;
    let name = if id.starts_with("agent-") {
        format!("{id}.jsonl")
    } else {
        format!("agent-{id}.jsonl")
    };
    Some(main.parent()?.join(stem).join("subagents").join(name))
}

/// True when the call runs in a subagent whose worker passed the classify gate itself
/// (`dla --worker <name> gate check --stage classify`). The worker is found as the
/// SubagentStop gate finds it: the last `--worker <name>` on a `dla` command of the
/// subagent's transcript. A call that is not in a subagent, or whose worker cannot be
/// identified, gets the core's rule.
fn worker_passed_classify(input: &Input, state: &RunState) -> bool {
    input.agent_id.is_some()
        && subagent_transcript(input)
            .and_then(|p| worker_from_transcript(&p.to_string_lossy()))
            .is_some_and(|w| state.worker_stage_passed(&w, Stage::Classify))
}

const READ_REASON: &str = "The archetype and known-leak tables unlock after the classify gate, so that the first hypotheses come from the evidence and not from a list. Run `dla gate check --stage classify`; it lists what is still missing.";

/// The reason to deny a read of the archetype files before the classify gate, or `None`.
fn check_read(input: &Input, tool: &str, found: &Found) -> Option<String> {
    let _ = found;
    let hit = match tool {
        "Read" => input
            .field("file_path")
            .or_else(|| input.field("path"))
            .is_some_and(|p| {
                let b = base_name(&slash(p)).to_ascii_lowercase();
                LOCKED_SUFFIXES
                    .iter()
                    .any(|n| b.ends_with(n) || b == n.trim_start_matches('-'))
            }),
        "Grep" => ["path", "glob"]
            .iter()
            .any(|k| input.field(k).is_some_and(locked_name)),
        "Glob" => ["pattern", "path"]
            .iter()
            .any(|k| input.field(k).is_some_and(locked_name)),
        "Bash" => input.field("command").is_some_and(locked_name),
        _ => false,
    };
    hit.then(|| READ_REASON.to_owned())
}

// ------------------------------------------------------------------ stop

fn digest_of(view: &RunView, missing: &[Missing]) -> String {
    let mut ids: Vec<String> = missing
        .iter()
        .map(|m| format!("{}:{}", m.kind, m.id))
        .collect();
    ids.sort();
    let slice_lines: u64 = view.slices.iter().map(|s| s.lines).sum();
    let mut h = Sha256::new();
    h.update(format!("{}:{slice_lines}:", view.ledger.lines().len()));
    h.update(ids.join(","));
    hex::encode(h.finalize())
}

fn clip(mut s: String, max: usize) -> String {
    if s.len() > max {
        let mut cut = max;
        while !s.is_char_boundary(cut) {
            cut -= 1;
        }
        s.truncate(cut);
    }
    s
}

fn block_reason(missing: &[Missing], worker: Option<&str>) -> String {
    let full = match worker {
        Some(w) => format!("dla --worker {w} gate check --final"),
        None => "dla gate check --final".to_owned(),
    };
    let mut s = format!(
        "dla: {} item(s) are still missing before this {} can stop.",
        missing.len(),
        if worker.is_some() { "worker" } else { "run" }
    );
    for m in missing.iter().take(3) {
        s.push_str(&format!(
            "\n- {}: {} Fix: {}",
            m.id,
            m.description.trim_end(),
            m.fix
        ));
    }
    s.push_str(&format!("\nThe full list: {full}"));
    clip(s, MAX_REASON)
}

fn stop(project_arg: Option<&Path>, input: &Input, subagent: bool) -> Result<HookOut, String> {
    let Some(found) = find_run(project_arg, input)? else {
        return Ok(HookOut::allow());
    };
    let Some(state) = found
        .state
        .as_ref()
        .filter(|s| s.status == RunStatus::Active)
    else {
        return Ok(HookOut::allow());
    };
    if subagent {
        let Some(path) = input.agent_transcript_path.as_deref() else {
            return Ok(HookOut::allow());
        };
        let Some(worker) = worker_from_transcript(path) else {
            return Ok(HookOut::allow());
        };
        if state.worker(&worker).is_none() {
            return Ok(HookOut::allow());
        }
        return stop_worker(&found, &worker);
    }
    stop_core(&found)
}

fn stop_core(found: &Found) -> Result<HookOut, String> {
    let view = RunView::load(&found.run_dir).map_err(|e| format!("cannot load the run: {e}"))?;
    let missing = gate::missing(&view, GateTarget::Final, None);
    if missing.is_empty() {
        return Ok(HookOut::allow());
    }
    let digest = digest_of(&view, &missing);
    let ids: Vec<String> = missing.iter().take(5).map(|m| m.id.clone()).collect();
    let stamped = RunState::update(&found.run_dir, |s| {
        let n = if s.stop_block_digest.as_deref() == Some(digest.as_str()) {
            s.stop_blocks.saturating_add(1)
        } else {
            1
        };
        s.stop_blocks = n;
        s.stop_block_digest = Some(digest.clone());
        if n >= STOP_LIMIT {
            s.status = RunStatus::Incomplete;
            s.incomplete_reason = Some(format!(
                "{} {STOP_LIMIT} times with no ledger change: {}",
                gate::STOP_BLOCKED_REASON,
                ids.join(", ")
            ));
        }
        Ok(n >= STOP_LIMIT)
    })
    .map_err(|e| format!("cannot update the run state: {e}"))?;
    if stamped {
        return Ok(HookOut {
            stdout: String::new(),
            stderr: format!(
                "dla gate: the stop was blocked {STOP_LIMIT} times with no change to the ledger; the run is stamped INCOMPLETE and the stop is allowed"
            ),
            code: 0,
        });
    }
    Ok(HookOut::block(&block_reason(&missing, None)))
}

fn stop_worker(found: &Found, worker: &str) -> Result<HookOut, String> {
    let view = RunView::load_for(&found.run_dir, Some(worker))
        .map_err(|e| format!("cannot load the worker's slice: {e}"))?;
    let missing = gate::missing(&view, GateTarget::Final, Some(worker));
    if missing.is_empty() {
        return Ok(HookOut::allow());
    }
    let digest = digest_of(&view, &missing);
    // Release the slice lock before the state update.
    drop(view);
    let stalled = RunState::update(&found.run_dir, |s| {
        let n = s
            .worker_stop
            .entry(worker.to_owned())
            .or_default()
            .bump(&digest);
        if n >= STOP_LIMIT {
            if let Some(w) = s.workers.iter_mut().find(|w| w.name == worker) {
                w.stalled = true;
            }
            return Ok(true);
        }
        Ok(false)
    })
    .map_err(|e| format!("cannot update the run state: {e}"))?;
    if stalled {
        return Ok(HookOut {
            stdout: String::new(),
            stderr: format!(
                "dla gate: worker {worker} was blocked {STOP_LIMIT} times with no change; it is marked stalled and may stop"
            ),
            code: 0,
        });
    }
    Ok(HookOut::block(&block_reason(&missing, Some(worker))))
}

// ------------------------------------------------------------ transcripts

fn expand_home(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(p)
}

/// Every string under a `command` key, anywhere in a JSON value.
fn commands(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                if k == "command"
                    && let Some(s) = x.as_str()
                {
                    out.push(s.to_owned());
                } else {
                    commands(x, out);
                }
            }
        }
        Value::Array(a) => a.iter().for_each(|x| commands(x, out)),
        _ => {}
    }
}

/// The worker named by `--worker <name>` (or `--worker=<name>`) in one command that runs `dla`.
fn worker_in_command(cmd: &str) -> Option<String> {
    let words: Vec<&str> = cmd.split_whitespace().collect();
    if !words.iter().any(|w| is_dla_word(w)) {
        return None;
    }
    let mut found = None;
    for (i, w) in words.iter().enumerate() {
        let name = if let Some(v) = w.strip_prefix("--worker=") {
            Some(v)
        } else if *w == "--worker" {
            words.get(i + 1).copied()
        } else {
            None
        };
        if let Some(n) = name {
            let n = n.trim_matches(['"', '\'']);
            if valid_worker_name(n) {
                found = Some(n.to_owned());
            }
        }
    }
    found
}

/// The worker a subagent transcript belongs to: the last `--worker <name>` on a Bash
/// command that runs `dla`.
fn worker_from_transcript(path: &str) -> Option<String> {
    let text = std::fs::read_to_string(expand_home(path)).ok()?;
    let mut last = None;
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let mut cmds = Vec::new();
        commands(&v, &mut cmds);
        for c in cmds {
            if let Some(w) = worker_in_command(&c) {
                last = Some(w);
            }
        }
    }
    last
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_handles_windows_separators_and_dots() {
        assert_eq!(
            normalize("C:\\proj\\.dla\\run\\..\\run\\state.json", "C:\\proj"),
            "c:/proj/.dla/run/state.json"
        );
        assert_eq!(
            normalize("a/./b/../c", "/nonexistent-root"),
            "/nonexistent-root/a/c"
        );
    }

    #[test]
    fn redirect_targets_skip_descriptor_duplication() {
        assert!(redirect_targets("dla run status 2>&1").is_empty());
        assert_eq!(
            redirect_targets("echo x >> .dla/r/ledger.jsonl"),
            [".dla/r/ledger.jsonl"]
        );
        assert_eq!(redirect_targets("echo x > \"a b\""), ["a"]);
    }

    #[test]
    fn mentions_dla_is_a_whole_directory_name() {
        assert!(mentions_dla("cat .dla/x"));
        assert!(mentions_dla("rm -rf .dla"));
        assert!(!mentions_dla("cat foo.dla.txt"));
        assert!(!mentions_dla("cat my.dla/x"));
    }

    #[test]
    fn worker_is_taken_from_a_dla_command_only() {
        assert_eq!(
            worker_in_command("dla --worker jvm ledger add --origin census"),
            Some("jvm".into())
        );
        assert_eq!(
            worker_in_command("\"/p/bin/dla\" --worker=go run status"),
            Some("go".into())
        );
        assert_eq!(worker_in_command("echo --worker jvm"), None);
    }

    #[test]
    fn locked_names() {
        assert!(locked_name("cat skills/x/references/go-archetypes.md"));
        assert!(locked_name("cat data\\Known-Leaks.toml"));
        assert!(!locked_name("cat README.md"));
    }
}
