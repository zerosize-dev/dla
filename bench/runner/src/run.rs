//! `dla-bench run`: headless Claude Code runs of the fixtures.

use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use clap::{Args, ValueEnum};
use regex::Regex;
use serde_json::Value;

use crate::{
    Res,
    fixture::{FINDINGS_INSTRUCTIONS, UNATTENDED_NOTE, fixture_dirs, load_expected},
    fixture_toolchain::{Host, Provided, Toolchain, observe, provide_all},
    guard,
    model::{Expected, RunJson, Usage},
};

/// The four ablations.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ablation {
    /// No plugin.
    #[value(name = "no-skill")]
    NoSkill,
    /// Skill text and references, no binary, hooks or agents.
    Prose,
    /// Skill, binary and gates; no hooks and no verifier.
    Gates,
    /// The plugin as shipped.
    Full,
}

impl Ablation {
    /// Directory and file spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Ablation::NoSkill => "no-skill",
            Ablation::Prose => "prose",
            Ablation::Gates => "gates",
            Ablation::Full => "full",
        }
    }

    fn uses_plugin(self) -> bool {
        self != Ablation::NoSkill
    }

    fn uses_binary(self) -> bool {
        matches!(self, Ablation::Gates | Ablation::Full)
    }
}

/// The archetypes switch.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Archetypes {
    /// Archetype files present.
    On,
    /// Archetype files removed.
    Off,
}

impl Archetypes {
    /// `on` or `off`.
    pub fn as_str(self) -> &'static str {
        match self {
            Archetypes::On => "on",
            Archetypes::Off => "off",
        }
    }
}

/// Flags of `dla-bench run`.
#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    /// Ablation to run.
    #[arg(long, value_enum)]
    pub ablation: Ablation,
    /// Whether archetype files are present.
    #[arg(long, value_enum, default_value = "on")]
    pub archetypes: Archetypes,
    /// Runs per fixture.
    #[arg(long, default_value_t = 3)]
    pub runs: u32,
    /// Comma-separated fixture IDs, or `all`.
    #[arg(long, default_value = "all")]
    pub fixtures: String,
    /// Directory with the fixtures.
    #[arg(long, default_value = "bench/fixtures")]
    pub fixtures_dir: PathBuf,
    /// Concurrent runs.
    #[arg(long, default_value_t = 2)]
    pub parallel: usize,
    /// Results directory (default: bench/results/<ablation>).
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Where working copies live; must be outside the repository (default:
    /// ${XDG_CACHE_HOME:-$HOME/.cache}/dla-bench).
    #[arg(long)]
    pub work_root: Option<PathBuf>,
    /// Extra path the child may not read (repeatable). The repository, the
    /// fixtures, the results directory and the user's Claude Code directory
    /// are always denied.
    #[arg(long = "deny-read")]
    pub deny_read: Vec<PathBuf>,
    /// Text file used verbatim as the prompt instead of the fixture's task,
    /// for checking the sandbox.
    #[arg(long)]
    pub probe_prompt: Option<PathBuf>,
    /// Model passed to claude.
    #[arg(long)]
    pub model: Option<String>,
    /// Maximum agent turns per run.
    #[arg(long, default_value_t = 80)]
    pub max_turns: u32,
    /// Time budget handed to the child's `dla` runs (DLA_DEFAULT_BUDGET):
    /// seconds, minutes or hours, such as 90s, 20m or 2h.
    #[arg(long, default_value = "20m")]
    pub child_budget: String,
    /// Seconds before a run's process group is killed.
    #[arg(long, default_value_t = 1500)]
    pub timeout: u64,
    /// Stop starting runs once the runs finished in the results directory
    /// (earlier invocations included) cost this much.
    #[arg(long)]
    pub budget_usd: Option<f64>,
    /// Treat a run.json with a nonzero exit_code, timed_out or an error as
    /// unfinished and run it again.
    #[arg(long)]
    pub retry_errored: bool,
    /// Repository root holding the plugin sources.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Directory with the ablation files (default: <repo-root>/bench/ablations).
    #[arg(long)]
    pub ablations_dir: Option<PathBuf>,
    /// Path of the dla binary for gates and full (default: build target/release/dla).
    #[arg(long)]
    pub dla_bin: Option<PathBuf>,
    /// The claude executable.
    #[arg(long, default_value = "claude")]
    pub claude_bin: String,
    /// Keep the working copy of runs that finished cleanly.
    #[arg(long)]
    pub keep_work: bool,
}

// ---------------------------------------------------------------- variants

/// Everything needed to assemble a plugin variant.
#[derive(Debug, Clone)]
pub struct VariantSpec {
    /// Repository root with `.claude-plugin/`, `skills/`, ...
    pub repo_root: PathBuf,
    /// Directory with `prose/SKILL.md`.
    pub ablations_dir: PathBuf,
    /// Ablation (not `no-skill`).
    pub ablation: Ablation,
    /// Whether archetype material is removed.
    pub archetypes_off: bool,
}

/// Copies a directory tree of trusted files. Symbolic links are never followed: they are
/// skipped, so a link cannot pull files from outside the tree.
pub fn copy_tree(src: &Path, dst: &Path) -> Res<()> {
    fs::create_dir_all(dst).map_err(|e| format!("{}: {e}", dst.display()))?;
    let rd = fs::read_dir(src).map_err(|e| format!("{}: {e}", src.display()))?;
    for entry in rd {
        let entry = entry.map_err(|e| format!("{}: {e}", src.display()))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let ft = entry
            .file_type()
            .map_err(|e| format!("{}: {e}", from.display()))?;
        if ft.is_dir() {
            copy_tree(&from, &to)?;
        } else if ft.is_file() {
            fs::copy(&from, &to).map_err(|e| format!("{}: {e}", from.display()))?;
        }
    }
    Ok(())
}

/// Deepest directory nesting copied out of a run directory.
const COLLECT_MAX_DEPTH: usize = 6;
/// Most entries (files and directories) looked at while collecting one run.
const COLLECT_MAX_ENTRIES: usize = 2_000;
/// Most bytes copied out of one run directory.
const COLLECT_MAX_BYTES: u64 = 64 * 1024 * 1024;

/// Copies files out of a run directory that the sandboxed model could write to. Never follows
/// a symbolic link, copies only regular files with an expected name, and stops at fixed depth,
/// entry and byte limits. Whatever is left out is recorded in `skipped`.
#[derive(Default)]
struct Collector {
    entries: usize,
    bytes: u64,
    skipped: Vec<String>,
}

impl Collector {
    fn skip(&mut self, path: &Path, why: &str) {
        if self.skipped.len() < 50 {
            self.skipped.push(format!("{}: {why}", path.display()));
        } else if self.skipped.len() == 50 {
            self.skipped.push("further skips not listed".to_owned());
        }
    }

    /// Whether `path` is a real directory (not a link to one).
    fn is_real_dir(path: &Path) -> bool {
        fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_dir())
    }

    fn copy_file(&mut self, from: &Path, to: &Path) {
        let Ok(meta) = fs::symlink_metadata(from) else {
            return;
        };
        if !meta.file_type().is_file() {
            if meta.file_type().is_symlink() {
                self.skip(from, "symbolic link");
            }
            return;
        }
        if self.bytes.saturating_add(meta.len()) > COLLECT_MAX_BYTES {
            self.skip(from, "byte limit reached");
            return;
        }
        if fs::copy(from, to).is_ok() {
            self.bytes += meta.len();
        }
    }

    /// Copies `ledger.<worker>.jsonl` (regular files only) from `run` to `out_dir`;
    /// worker ledgers of an earlier attempt are removed first. The merged
    /// `ledger.jsonl` is copied separately.
    fn collect_worker_ledgers(&mut self, run: &Path, out_dir: &Path) {
        remove_worker_ledgers(out_dir);
        let Ok(rd) = fs::read_dir(run) else { return };
        for entry in rd.flatten() {
            self.entries += 1;
            if self.entries > COLLECT_MAX_ENTRIES {
                self.skip(run, "entry limit reached");
                return;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_worker_ledger_name(&name) {
                self.copy_file(&entry.path(), &out_dir.join(&name));
            }
        }
    }

    /// Copies `bundle/<id>/index.json`, `manifest.json` and the `excerpt*.txt` files (never
    /// anything else of a bundle, and never through a link) from `run` to `out_dir`, so a
    /// reviewer can see what the verifier was shown; an earlier attempt's bundles are removed
    /// first.
    fn collect_bundle_indexes(&mut self, run: &Path, out_dir: &Path) {
        let _ = fs::remove_dir_all(out_dir.join("bundle"));
        let from = run.join("bundle");
        if !Self::is_real_dir(&from) {
            if fs::symlink_metadata(&from).is_ok() {
                self.skip(&from, "not a directory");
            }
            return;
        }
        let Ok(rd) = fs::read_dir(&from) else { return };
        for entry in rd.flatten() {
            self.entries += 1;
            if self.entries > COLLECT_MAX_ENTRIES {
                self.skip(&from, "entry limit reached");
                return;
            }
            let dir = entry.path();
            if !Self::is_real_dir(&dir) {
                continue;
            }
            let dest = out_dir.join("bundle").join(entry.file_name());
            let _ = fs::create_dir_all(&dest);
            self.copy_file(&dir.join("index.json"), &dest.join("index.json"));
            self.copy_file(&dir.join("manifest.json"), &dest.join("manifest.json"));
            if let Ok(files) = fs::read_dir(&dir) {
                for f in files.flatten() {
                    let name = f.file_name().to_string_lossy().into_owned();
                    if is_excerpt_name(&name) {
                        self.entries += 1;
                        self.copy_file(&f.path(), &dest.join(&name));
                    }
                }
            }
            if fs::read_dir(&dest).is_ok_and(|mut d| d.next().is_none()) {
                let _ = fs::remove_dir(&dest);
            }
        }
    }

    /// Copies `integrity/*.seal` (the seals `dla` wrote for the protected files) from `run`
    /// to `out_dir`; an earlier attempt's seals are removed first.
    fn collect_seals(&mut self, run: &Path, out_dir: &Path) {
        let _ = fs::remove_dir_all(out_dir.join("integrity"));
        let from = run.join("integrity");
        if !Self::is_real_dir(&from) {
            if fs::symlink_metadata(&from).is_ok() {
                self.skip(&from, "not a directory");
            }
            return;
        }
        let Ok(rd) = fs::read_dir(&from) else { return };
        let dest = out_dir.join("integrity");
        let _ = fs::create_dir_all(&dest);
        for entry in rd.flatten() {
            self.entries += 1;
            if self.entries > COLLECT_MAX_ENTRIES {
                self.skip(&from, "entry limit reached");
                return;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_seal_name(&name) {
                self.copy_file(&entry.path(), &dest.join(&name));
            }
        }
        if fs::read_dir(&dest).is_ok_and(|mut d| d.next().is_none()) {
            let _ = fs::remove_dir(&dest);
        }
    }

    /// Copies the files of `src` named `<prefix>*.json`, recursing into real directories.
    fn copy_dir(&mut self, src: &Path, dst: &Path, prefix: char, depth: usize) {
        if depth > COLLECT_MAX_DEPTH {
            self.skip(src, "depth limit reached");
            return;
        }
        let Ok(rd) = fs::read_dir(src) else { return };
        let _ = fs::create_dir_all(dst);
        for entry in rd.flatten() {
            self.entries += 1;
            if self.entries > COLLECT_MAX_ENTRIES {
                self.skip(src, "entry limit reached");
                return;
            }
            let Ok(ft) = entry.file_type() else { continue };
            let name = entry.file_name().to_string_lossy().into_owned();
            let from = entry.path();
            if ft.is_symlink() {
                self.skip(&from, "symbolic link");
            } else if ft.is_dir() {
                self.copy_dir(&from, &dst.join(&name), prefix, depth + 1);
            } else if ft.is_file() {
                if name.starts_with(prefix) && name.ends_with(".json") {
                    self.copy_file(&from, &dst.join(&name));
                } else {
                    self.skip(&from, "unexpected file name");
                }
            }
        }
    }
}

/// Removes a top-level `hooks:` key (and its indented body) from the YAML
/// frontmatter of a SKILL.md; the rest of the text is untouched.
pub fn strip_frontmatter_hooks(text: &str) -> String {
    let mut lines = text.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return String::new();
    };
    if first.trim_end() != "---" {
        return text.to_owned();
    }
    let mut out = String::from(first);
    let mut in_front = true;
    let mut skipping = false;
    for line in lines {
        if in_front {
            if line.trim_end() == "---" {
                in_front = false;
                skipping = false;
                out.push_str(line);
                continue;
            }
            let indented = line.starts_with(' ') || line.starts_with('\t');
            let blank_or_comment = line.trim().is_empty() || line.trim_start().starts_with('#');
            if skipping && (indented || blank_or_comment) {
                continue;
            }
            skipping = false;
            if line.starts_with("hooks:") {
                skipping = true;
                continue;
            }
        }
        out.push_str(line);
    }
    out
}

/// A worker ledger file name: `ledger.<worker>.jsonl` with a non-empty worker
/// (the merged `ledger.jsonl` is not one).
fn is_worker_ledger_name(name: &str) -> bool {
    name.strip_prefix("ledger.")
        .and_then(|rest| rest.strip_suffix(".jsonl"))
        .is_some_and(|w| !w.is_empty())
}

/// Removes collected worker ledgers and bundle indexes of an earlier attempt.
fn remove_worker_ledgers(out_dir: &Path) {
    if let Ok(rd) = fs::read_dir(out_dir) {
        for e in rd.flatten() {
            if is_worker_ledger_name(&e.file_name().to_string_lossy()) {
                let _ = fs::remove_file(e.path());
            }
        }
    }
}

/// An excerpt file of a bundle: `excerpt.txt`, `excerpt-2.txt`, ...
fn is_excerpt_name(name: &str) -> bool {
    name.strip_prefix("excerpt")
        .and_then(|rest| rest.strip_suffix(".txt"))
        .is_some_and(|mid| {
            mid.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
}

/// The integrity seal files of a run (`integrity/<file>.seal`).
fn is_seal_name(name: &str) -> bool {
    name.strip_suffix(".seal")
        .is_some_and(|stem| !stem.is_empty() && !stem.contains(['/', '\\']))
}

fn remove_file_if_exists(p: &Path) {
    let _ = fs::remove_file(p);
}

/// Begin and end markers of the text the archetypes-off ablation removes.
pub const ARCHETYPES_BEGIN: &str = "<!-- archetypes:begin -->";
/// See [`ARCHETYPES_BEGIN`].
pub const ARCHETYPES_END: &str = "<!-- archetypes:end -->";

/// Removes every `<!-- archetypes:begin -->` ... `<!-- archetypes:end -->` block (the markers
/// included, and the line break after the end marker). An unbalanced marker is an error: a
/// variant must not silently keep, or lose, more text than the markers say.
pub fn strip_archetype_blocks(text: &str) -> Res<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(b) = rest.find(ARCHETYPES_BEGIN) {
        let (before, after) = rest.split_at(b);
        if before.contains(ARCHETYPES_END) {
            return Err(format!("`{ARCHETYPES_END}` without a begin marker"));
        }
        out.push_str(before);
        let after = &after[ARCHETYPES_BEGIN.len()..];
        let Some(e) = after.find(ARCHETYPES_END) else {
            return Err(format!("`{ARCHETYPES_BEGIN}` without an end marker"));
        };
        if after[..e].contains(ARCHETYPES_BEGIN) {
            return Err(format!("`{ARCHETYPES_BEGIN}` inside another block"));
        }
        let tail = &after[e + ARCHETYPES_END.len()..];
        rest = tail
            .strip_prefix("\r\n")
            .or_else(|| tail.strip_prefix('\n'))
            .unwrap_or(tail);
    }
    if rest.contains(ARCHETYPES_END) {
        return Err(format!("`{ARCHETYPES_END}` without a begin marker"));
    }
    out.push_str(rest);
    Ok(out)
}

/// Strips the marked blocks from every `.md` file under `dir` (links are not followed).
fn strip_archetype_blocks_in(dir: &Path) -> Res<()> {
    let rd = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in rd.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        let path = entry.path();
        if ft.is_dir() {
            strip_archetype_blocks_in(&path)?;
        } else if ft.is_file() && path.extension().is_some_and(|e| e == "md") {
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            if text.contains(ARCHETYPES_BEGIN) || text.contains(ARCHETYPES_END) {
                let new = strip_archetype_blocks(&text)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                fs::write(&path, new).map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
    }
    Ok(())
}

/// Turns a plugin directory into its archetypes-off form: the archetype rows, the known-leaks
/// reference and the data files are deleted, the `suggested_archetype` of every detector is
/// blanked, and every marked archetype block of every `.md` file is removed.
pub fn apply_archetypes_off(dest: &Path) -> Res<()> {
    let skills = dest.join("skills");
    if skills.is_dir() {
        for skill in fs::read_dir(&skills).map_err(|e| e.to_string())?.flatten() {
            let refs = skill.path().join("references");
            if !refs.is_dir() {
                continue;
            }
            for f in fs::read_dir(&refs).map_err(|e| e.to_string())?.flatten() {
                let name = f.file_name().to_string_lossy().into_owned();
                if name.ends_with("-archetypes.md") || name == "known-leaks.md" {
                    remove_file_if_exists(&f.path());
                }
            }
        }
    }
    remove_file_if_exists(&dest.join("data/archetypes.toml"));
    remove_file_if_exists(&dest.join("data/known-leaks.toml"));
    let detectors = dest.join("data/detectors");
    if detectors.is_dir() {
        let re = Regex::new(r#"(?m)^(\s*suggested_archetype\s*=\s*)"[^"]*""#).expect("valid regex");
        for f in fs::read_dir(&detectors)
            .map_err(|e| e.to_string())?
            .flatten()
        {
            let p = f.path();
            if p.extension().is_some_and(|e| e == "toml") {
                let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
                let new = re.replace_all(&text, r#"$1"""#);
                if new != text {
                    fs::write(&p, new.as_ref()).map_err(|e| format!("{}: {e}", p.display()))?;
                }
            }
        }
    }
    strip_archetype_blocks_in(dest)
}

/// Assembles the plugin variant for an ablation into `dest` (which must not exist).
pub fn build_variant(spec: &VariantSpec, dest: &Path) -> Res<()> {
    let root = &spec.repo_root;
    if spec.ablation == Ablation::NoSkill {
        return Err("the no-skill ablation has no plugin".to_owned());
    }
    fs::create_dir_all(dest.join(".claude-plugin")).map_err(|e| e.to_string())?;
    let manifest = root.join(".claude-plugin/plugin.json");
    fs::copy(&manifest, dest.join(".claude-plugin/plugin.json"))
        .map_err(|e| format!("{}: {e}", manifest.display()))?;
    copy_tree(&root.join("skills"), &dest.join("skills"))?;
    if root.join("data").is_dir() && spec.ablation != Ablation::Prose {
        copy_tree(&root.join("data"), &dest.join("data"))?;
    }
    match spec.ablation {
        Ablation::Prose => {
            let prose = spec.ablations_dir.join("prose/SKILL.md");
            let skill_dir = dest.join("skills/detect-leak-anywhere");
            fs::create_dir_all(&skill_dir).map_err(|e| e.to_string())?;
            fs::copy(&prose, skill_dir.join("SKILL.md"))
                .map_err(|e| format!("{}: {e}", prose.display()))?;
        }
        Ablation::Gates | Ablation::Full => {
            copy_tree(&root.join("bin"), &dest.join("bin"))?;
            if root.join("agents").is_dir() {
                copy_tree(&root.join("agents"), &dest.join("agents"))?;
            }
            if spec.ablation == Ablation::Full {
                copy_tree(&root.join("hooks"), &dest.join("hooks"))?;
            } else {
                remove_file_if_exists(&dest.join("agents/verifier.md"));
                for skill in fs::read_dir(dest.join("skills"))
                    .map_err(|e| e.to_string())?
                    .flatten()
                {
                    let p = skill.path().join("SKILL.md");
                    if let Ok(text) = fs::read_to_string(&p) {
                        fs::write(&p, strip_frontmatter_hooks(&text)).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        Ablation::NoSkill => {}
    }
    if spec.archetypes_off {
        apply_archetypes_off(dest)?;
    }
    Ok(())
}

// ------------------------------------------------------------- environment

const KEEP_CLAUDE_VARS: [&str; 5] = [
    "CLAUDE_CODE_OAUTH_TOKEN",
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "CLAUDE_CODE_USE_FOUNDRY",
    "CLAUDE_CODE_USE_ANTHROPIC_AWS",
];

/// Whether an inherited environment variable may reach the child. Everything
/// that would make the child act as a part of the parent session, and every
/// credential not needed to reach the model, is dropped.
pub fn keep_env_var(name: &str) -> bool {
    if KEEP_CLAUDE_VARS.contains(&name) {
        return true;
    }
    let denied_prefix = [
        "CLAUDE",
        "CCR_",
        "DOCUMENTS_MCP",
        "SBX_",
        "DLA_",
        "MODAL_",
        "FSSPEC_",
        "CLOUDSDK_",
    ];
    let denied_name = [
        "AI_AGENT",
        "SESSION_INGRESS_URL",
        "MAX_THINKING_TOKENS",
        "GH_TOKEN",
        "GITHUB_TOKEN",
        "IS_SANDBOX",
    ];
    !denied_prefix.iter().any(|p| name.starts_with(p)) && !denied_name.contains(&name)
}

/// The default parent of the working copies, resolved from the given
/// environment values: `$XDG_CACHE_HOME/dla-bench`, else `$HOME/.cache/dla-bench`.
/// Empty values count as unset. Kept out of the system temporary directory, which
/// the children may write to.
pub fn default_work_root_from(xdg_cache_home: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    let set = |v: Option<&str>| v.filter(|x| !x.is_empty()).map(PathBuf::from);
    set(xdg_cache_home)
        .or_else(|| set(home).map(|h| h.join(".cache")))
        .map(|c| c.join("dla-bench"))
}

/// The default parent of the working copies, resolved from the environment now.
pub fn default_work_root() -> Res<PathBuf> {
    default_work_root_from(
        std::env::var("XDG_CACHE_HOME").ok().as_deref(),
        std::env::var("HOME").ok().as_deref(),
    )
    .ok_or_else(|| "neither XDG_CACHE_HOME nor HOME is set; pass --work-root".to_owned())
}

/// True when denying `p` would also deny Claude Code's shared per-user
/// temporary directory (`claude-<uid>` under the system temp directory):
/// `p` is that directory, the temp directory itself, or an ancestor of
/// either. Child sessions write their shell state and `TMPDIR` files there,
/// so denying it breaks every Bash call while leaving the run apparently
/// healthy.
pub fn covers_shared_tmp(p: &Path) -> bool {
    let bases = [std::env::temp_dir(), PathBuf::from("/tmp")];
    bases.iter().any(|base| {
        base.starts_with(p)
            || (p.parent() == Some(base.as_path())
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("claude-")))
    })
}

/// A path in the permission-rule syntax: an absolute path starts with `//`.
fn rule_path(p: &Path) -> String {
    let text = p.display().to_string();
    format!("/{}", text.replace('\\', "/"))
}

/// The Claude Code settings that confine a child session: Bash runs in the
/// operating system sandbox with no network, writes only in the working copy
/// and in `allow_write` (the same places the Edit and Write tools may touch)
/// plus `sandbox_extra_write` (places only Bash may write to, such as the
/// system temporary directory that JVM attach needs), and neither the sandbox
/// nor the file tools can read `deny_read`. `deny_write` lists places the
/// sandbox may not write to even when a parent directory is writable.
/// The default permission mode applies; a call that would need a prompt is
/// refused in headless mode.
pub fn sandbox_settings(
    deny_read: &[PathBuf],
    allow_write: &[PathBuf],
    sandbox_extra_write: &[PathBuf],
    deny_write: &[PathBuf],
) -> Value {
    let tools = ["Read", "Edit", "Write", "Glob", "Grep"];
    // Bash is allowed outright so that commands using shell variables are not
    // refused for being uncheckable; every Bash command still runs inside the
    // sandbox, because unsandboxed commands are disallowed and the session
    // refuses to start without the sandbox. Edit and Write are not covered by
    // the sandbox: they may only touch the places Bash may write to.
    let mut allow: Vec<String> = ["Bash", "Read", "Glob", "Grep", "Agent", "Task"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    for tool in ["Edit", "Write"] {
        for p in allow_write {
            allow.push(format!("{tool}({}/**)", rule_path(p)));
        }
    }
    let mut deny: Vec<String> = Vec::new();
    for tool in tools {
        for p in deny_read {
            deny.push(format!("{tool}({}/**)", rule_path(p)));
        }
    }
    deny.push("WebFetch".to_owned());
    deny.push("WebSearch".to_owned());
    let paths =
        |v: &[PathBuf]| -> Vec<String> { v.iter().map(|p| p.display().to_string()).collect() };
    let sandbox_write: Vec<PathBuf> = allow_write
        .iter()
        .chain(sandbox_extra_write)
        .cloned()
        .collect();
    let mut filesystem = serde_json::json!({
        "denyRead": paths(deny_read),
        "allowWrite": paths(&sandbox_write),
    });
    if !deny_write.is_empty() {
        filesystem["denyWrite"] = serde_json::json!(paths(deny_write));
    }
    serde_json::json!({
        "sandbox": {
            "enabled": true,
            "autoAllowBashIfSandboxed": true,
            "allowUnsandboxedCommands": false,
            "failIfUnavailable": true,
            "filesystem": filesystem,
        },
        "permissions": {
            "allow": allow,
            "deny": deny,
        },
    })
}

/// Directories the toolchains write to, moved under `cache` so the sandbox
/// needs one write entry for all of them; package registries are switched off.
pub fn cache_env(cache: &Path, rustup_home: &Path) -> Vec<(String, String)> {
    let at = |sub: &str| cache.join(sub).display().to_string();
    vec![
        ("GOCACHE".to_owned(), at("go-build")),
        ("GOPATH".to_owned(), at("gopath")),
        ("GOMODCACHE".to_owned(), at("gopath/pkg/mod")),
        ("GOTMPDIR".to_owned(), at("go-tmp")),
        ("GOFLAGS".to_owned(), "-modcacherw".to_owned()),
        ("GOPROXY".to_owned(), "off".to_owned()),
        // `auto`, not `local`: with the proxy off nothing is downloaded and a project that
        // asks for no newer Go builds with the local one, while `dla`'s goroutine leak pass may
        // switch to the go1.27 toolchain linked into the module cache (`local` forbids that
        // switch, and `dla` respects it: the M5 gates run had the leak profile blocked).
        ("GOTOOLCHAIN".to_owned(), "auto".to_owned()),
        (
            "MAVEN_OPTS".to_owned(),
            format!("-Dmaven.repo.local={}", at("m2")),
        ),
        ("npm_config_cache".to_owned(), at("npm")),
        ("npm_config_update_notifier".to_owned(), "false".to_owned()),
        ("PIP_CACHE_DIR".to_owned(), at("pip")),
        ("PIP_NO_INDEX".to_owned(), "1".to_owned()),
        ("CARGO_HOME".to_owned(), at("cargo")),
        ("CARGO_NET_OFFLINE".to_owned(), "true".to_owned()),
        ("RUSTUP_HOME".to_owned(), rustup_home.display().to_string()),
        ("XDG_CACHE_HOME".to_owned(), at("xdg")),
    ]
}

/// The sub-directories of the cache that must exist before a run starts.
const CACHE_DIRS: [&str; 8] = [
    "go-build",
    "gopath/pkg/mod",
    "go-tmp",
    "m2",
    "npm",
    "pip",
    "cargo",
    "xdg",
];

// ------------------------------------------------------------ go toolchain

/// The Go release the goroutine leak profile needs (`dla install go-toolchain-1.27`).
const GO_LEAK_TOOLCHAIN: &str = "go1.27.0";

/// The unpacked toolchain directory name inside a Go module cache, for this platform.
pub fn go_toolchain_dir_name(version: &str, os: &str, arch: &str) -> String {
    let arch = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        a => a,
    };
    let os = if os == "macos" { "darwin" } else { os };
    format!("toolchain@v0.0.1-{version}.{os}-{arch}")
}

/// The host module cache: `go env GOMODCACHE`, else `$HOME/go/pkg/mod`.
pub fn host_go_modcache() -> Option<PathBuf> {
    let out = Command::new("go")
        .args(["env", "GOMODCACHE"])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if out.status.success() && !s.is_empty() {
        return Some(PathBuf::from(s));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join("go/pkg/mod"))
}

/// Makes sure the Go leak toolchain is in the shared (host) module cache, once per run of
/// the benchmark and before any child starts. A cached toolchain costs nothing; otherwise
/// `GOTOOLCHAIN=go1.27.0 go version` fetches it (the benchmark process has the network, the
/// children do not). Returns the unpacked directory, or `None` with a warning when it cannot
/// be had; the children then see the `go/goroutineleak` coverage row as missing.
pub fn prefetch_go_toolchain() -> Option<PathBuf> {
    static DIR: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let cache = host_go_modcache()?;
        let dir = prefetch_go_toolchain_with(Path::new("go"), &cache, GO_LEAK_TOOLCHAIN);
        if dir.is_none() {
            eprintln!("warning: {GO_LEAK_TOOLCHAIN} could not be fetched; go/goroutineleak stays missing in the children");
        }
        dir
    })
    .clone()
}

/// [`prefetch_go_toolchain`] for any release (`go1.22.12`) and any module cache: runs
/// `GOTOOLCHAIN=<version> <go> version` in a directory without a `go.mod`, which downloads
/// and unpacks the toolchain into `cache` when it is missing. Returns the unpacked directory.
pub fn prefetch_go_toolchain_with(go: &Path, cache: &Path, version: &str) -> Option<PathBuf> {
    let name = go_toolchain_dir_name(version, std::env::consts::OS, std::env::consts::ARCH);
    let dir = cache.join("golang.org").join(&name);
    if dir.join("bin").is_dir() {
        return Some(dir);
    }
    let fetched = Command::new(go)
        .arg("version")
        .current_dir(std::env::temp_dir())
        .env("GOTOOLCHAIN", version)
        .env("GOMODCACHE", cache)
        .env_remove("GOFLAGS")
        .stdin(Stdio::null())
        .output()
        .is_ok_and(|o| o.status.success());
    (fetched && dir.join("bin").is_dir()).then_some(dir)
}

/// Links the prefetched toolchain into a child's module cache, so `GOTOOLCHAIN=go1.27.0`
/// switches offline: the unpacked directory, and the download-cache entry that Go checks
/// before it unpacks (the small files are copied so a lock can be written, the archive is
/// linked). A failure is not an error: the coverage row says it.
fn link_go_toolchain(cache: &Path) {
    if let Some(dir) = prefetch_go_toolchain() {
        link_toolchain(&dir, cache);
    }
}

#[cfg(not(unix))]
pub fn link_toolchain(_dir: &Path, _cache: &Path) {}

/// [`link_go_toolchain`] for a known unpacked toolchain directory.
#[cfg(unix)]
pub fn link_toolchain(dir: &Path, cache: &Path) {
    let (Some(name), Some(host)) = (
        dir.file_name().and_then(|n| n.to_str()),
        dir.parent().and_then(Path::parent),
    ) else {
        return;
    };
    let modcache = cache.join("gopath/pkg/mod");
    let unpacked = modcache.join("golang.org");
    if fs::create_dir_all(&unpacked).is_err() {
        return;
    }
    {
        let link = unpacked.join(name);
        if link.symlink_metadata().is_err() {
            let _ = std::os::unix::fs::symlink(dir, &link);
        }
        // `toolchain@v0.0.1-go1.27.0.linux-amd64` is cached as `v0.0.1-go1.27.0.linux-amd64.*`.
        let Some(version) = name.strip_prefix("toolchain@") else {
            return;
        };
        let src = host.join("cache/download/golang.org/toolchain/@v");
        let dst = modcache.join("cache/download/golang.org/toolchain/@v");
        if fs::create_dir_all(&dst).is_err() {
            return;
        }
        for e in fs::read_dir(&src).into_iter().flatten().flatten() {
            let file = e.file_name();
            let Some(f) = file.to_str().filter(|f| f.starts_with(version)) else {
                continue;
            };
            let to = dst.join(f);
            if to.symlink_metadata().is_ok() {
                continue;
            }
            if f.ends_with(".zip") {
                let _ = std::os::unix::fs::symlink(e.path(), &to);
            } else {
                let _ = fs::copy(e.path(), &to);
            }
        }
    }
    // Before it switches, Go verifies the toolchain module against the checksum database,
    // and a child's sandbox has no network for sum.golang.org. The host verified it when
    // it fetched the toolchain, so the checksum database cache (lookups and tiles, a few
    // MB) is copied: verification stays on and works offline. A copy, not a link: Go
    // writes to this cache.
    let sumdb = host.join("cache/download/sumdb");
    let to = modcache.join("cache/download/sumdb");
    if sumdb.is_dir() && to.symlink_metadata().is_err() {
        let _ = copy_dir(&sumdb, &to);
    }
}

/// Copies a directory tree (regular files and directories; links are skipped).
#[cfg(unix)]
fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for e in fs::read_dir(from)?.flatten() {
        let ty = e.file_type()?;
        let dst = to.join(e.file_name());
        if ty.is_dir() {
            copy_dir(&e.path(), &dst)?;
        } else if ty.is_file() {
            fs::copy(e.path(), &dst)?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------- tasks

#[derive(Debug, Clone)]
struct Task {
    fixture: String,
    n: u32,
    expected: Expected,
}

/// Orders (fixture, runtime) pairs round-robin across runtimes.
pub fn interleave(mut items: Vec<(String, String)>) -> Vec<String> {
    items.sort();
    let mut buckets: Vec<(String, VecDeque<String>)> = Vec::new();
    for (id, rt) in items {
        match buckets.iter_mut().find(|(r, _)| *r == rt) {
            Some((_, q)) => q.push_back(id),
            None => buckets.push((rt, VecDeque::from([id]))),
        }
    }
    let mut out = Vec::new();
    loop {
        let mut any = false;
        for (_, q) in &mut buckets {
            if let Some(id) = q.pop_front() {
                out.push(id);
                any = true;
            }
        }
        if !any {
            return out;
        }
    }
}

struct Shared {
    queue: Mutex<VecDeque<Task>>,
    /// Cost of the finished runs in the results directory, earlier
    /// invocations included.
    spent: Mutex<f64>,
    /// Consecutive finished runs with zero turns and zero tokens.
    empty_streak: Mutex<u32>,
    /// Set when the queue was abandoned because of a systematic failure.
    aborted: AtomicBool,
}

/// Consecutive runs without a single turn or token after which the queue is
/// abandoned: the cause (authentication, quota, network) is systematic.
pub const MAX_EMPTY_STREAK: u32 = 3;

/// The files a run collects; removed before an attempt so that nothing of an
/// earlier attempt is graded as the answer of the new one.
pub const RUN_ARTIFACTS: [&str; 8] = [
    "findings.json",
    "ledger.jsonl",
    "state.json",
    "report.md",
    "coverage.json",
    "transcript.jsonl",
    "stderr.log",
    "run.json",
];

/// Whether a parsed run.json describes a failed run: a missing or nonzero
/// `exit_code`, `timed_out` true, or a non-null `error`.
pub fn run_json_errored(v: &Value) -> bool {
    v.get("exit_code").and_then(Value::as_i64) != Some(0)
        || v.get("timed_out").and_then(Value::as_bool) == Some(true)
        || v.get("error").is_some_and(|e| !e.is_null())
}

/// Whether the run directory holds a finished run. With `retry_errored` a
/// run.json that is unreadable or describes a failed run is not finished.
pub fn run_finished(run_dir: &Path, retry_errored: bool) -> bool {
    let p = run_dir.join("run.json");
    if !retry_errored {
        return p.is_file();
    }
    fs::read(&p)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .is_some_and(|v| v.is_object() && !run_json_errored(&v))
}

fn subdirs(p: &Path) -> Vec<PathBuf> {
    fs::read_dir(p)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default()
}

/// The summed `cost_usd` of the finished runs under `<out_root>/runs`.
pub fn finished_cost(out_root: &Path, retry_errored: bool) -> f64 {
    let mut total = 0.0;
    let mut level = vec![out_root.join("runs")];
    for _ in 0..4 {
        level = level.iter().flat_map(|d| subdirs(d)).collect();
    }
    for rd in level {
        if !run_finished(&rd, retry_errored) {
            continue;
        }
        if let Some(c) = fs::read(rd.join("run.json"))
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .and_then(|v| v.get("cost_usd").and_then(Value::as_f64))
        {
            total += c;
        }
    }
    total
}

fn git(dir: &Path, args: &[&str]) -> Res<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "bench")
        .env("GIT_AUTHOR_EMAIL", "bench@example.invalid")
        .env("GIT_COMMITTER_NAME", "bench")
        .env("GIT_COMMITTER_EMAIL", "bench@example.invalid")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn list_all_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Res<()> {
    for e in fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
    {
        let p = e.path();
        if e.file_name() == ".git" {
            continue;
        }
        if p.is_dir() {
            list_all_files(root, &p, out)?;
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_path_buf());
        }
    }
    Ok(())
}

/// The files of a fixture's `project/` that belong in the working copy: the
/// tracked files plus untracked files that git does not ignore; every file when
/// the directory is not in a git repository.
pub fn project_files(project: &Path) -> Res<Vec<PathBuf>> {
    let listing = git(
        project,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    );
    let mut files: Vec<PathBuf> = match listing {
        Ok(text) => text
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect(),
        Err(_) => {
            let mut v = Vec::new();
            list_all_files(project, project, &mut v)?;
            v
        }
    };
    files.retain(|f| project.join(f).is_file());
    files.sort();
    files.dedup();
    Ok(files)
}

fn prepare_workdir(src_project: &Path, work: &Path) -> Res<PathBuf> {
    if work.exists() {
        fs::remove_dir_all(work).map_err(|e| format!("{}: {e}", work.display()))?;
    }
    let project = work.join("project");
    fs::create_dir_all(&project).map_err(|e| format!("{}: {e}", project.display()))?;
    let files = project_files(src_project)?;
    if files.is_empty() {
        return Err(format!("{}: no files to copy", src_project.display()));
    }
    for f in files {
        let to = project.join(&f);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(src_project.join(&f), &to).map_err(|e| format!("{}: {e}", f.display()))?;
    }
    git(&project, &["init", "-q"])?;
    git(&project, &["add", "-A"])?;
    git(
        &project,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-m",
            "initial",
        ],
    )?;
    Ok(project)
}

/// The prompt for an ablation.
pub fn build_prompt(ablation: Ablation, task: &str) -> String {
    let task = task.trim();
    match ablation {
        Ablation::NoSkill => format!("{task}\n\n{FINDINGS_INSTRUCTIONS}"),
        Ablation::Prose => format!("/dla:detect-leak-anywhere {task}\n\n{FINDINGS_INSTRUCTIONS}"),
        Ablation::Gates | Ablation::Full => {
            format!("/dla:detect-leak-anywhere {task}\n\n{UNATTENDED_NOTE}")
        }
    }
}

/// The environment of the child: the sanitised parent environment plus the
/// per-run settings. `cache` holds the toolchain caches of the run and `tmp`
/// the temporary files of the child, so runs do not see each other's.
pub fn child_env(
    parent: impl IntoIterator<Item = (String, String)>,
    config_dir: &Path,
    cache: &Path,
    tmp: &Path,
    dla_bin: Option<&Path>,
    args: &RunArgs,
) -> Vec<(String, String)> {
    let parent: Vec<(String, String)> = parent.into_iter().collect();
    let lookup = |k: &str| parent.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    let rustup_home = lookup("RUSTUP_HOME")
        .map(PathBuf::from)
        .or_else(|| lookup("HOME").map(|h| PathBuf::from(h).join(".rustup")))
        .unwrap_or_else(|| PathBuf::from("/root/.rustup"));
    let mut env: Vec<(String, String)> = parent
        .into_iter()
        .filter(|(k, _)| keep_env_var(k))
        .collect();
    let mut set = |k: &str, v: String| {
        env.retain(|(name, _)| name != k);
        env.push((k.to_owned(), v));
    };
    set("CLAUDE_CONFIG_DIR", config_dir.display().to_string());
    set("DISABLE_AUTOUPDATER", "1".to_owned());
    set("CLAUDE_CODE_TMPDIR", tmp.display().to_string());
    for (k, v) in cache_env(cache, &rustup_home) {
        set(&k, v);
    }
    // The parent's DLA_* variables never reach the child; the budget is set explicitly.
    set("DLA_DEFAULT_BUDGET", args.child_budget.clone());
    if args.ablation.uses_plugin() {
        set("DLA_CONSENT_TIERS", "1-4".to_owned());
        set("DLA_CONSENT_RAW_VALUES", "1".to_owned());
        if args.archetypes == Archetypes::Off {
            set("DLA_ARCHETYPES", "off".to_owned());
        }
    }
    if let Some(bin) = dla_bin {
        set("DLA_BIN", bin.display().to_string());
    }
    env
}

struct Finished {
    exit_code: i32,
    timed_out: bool,
    wall: Duration,
}

#[cfg(unix)]
fn kill_group(child: &mut std::process::Child) {
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{}", child.id())])
        .status();
    let _ = child.kill();
}

#[cfg(not(unix))]
fn kill_group(child: &mut std::process::Child) {
    let _ = child.kill();
}

fn spawn_and_wait(
    #[cfg_attr(not(unix), allow(unused_mut))] mut cmd: Command,
    timeout: Duration,
) -> Res<Finished> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let start = Instant::now();
    if guard::stop_signal().is_some() {
        return Err("interrupted".to_owned());
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("cannot start claude: {e}"))?;
    let _group = guard::register_group(child.id());
    let mut timed_out = false;
    let status = loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(s) => break s,
            None => {
                if start.elapsed() >= timeout {
                    timed_out = true;
                    kill_group(&mut child);
                    break child.wait().map_err(|e| e.to_string())?;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    };
    // Sweep stragglers of the group that outlived the leader (an interrupt
    // does it itself, after its grace period).
    #[cfg(unix)]
    if guard::stop_signal().is_none() {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stderr(Stdio::null())
            .status();
    }
    Ok(Finished {
        exit_code: status.code().unwrap_or(-1),
        timed_out,
        wall: start.elapsed(),
    })
}

/// What the transcript says about the run.
#[derive(Debug, Default, PartialEq)]
pub struct TranscriptInfo {
    /// Model from the init event.
    pub model: Option<String>,
    /// Token usage from the final result event.
    pub usage: Usage,
    /// Cost from the final result event.
    pub cost_usd: Option<f64>,
    /// Turns from the final result event.
    pub num_turns: Option<u64>,
    /// Whether a result event was seen.
    pub has_result: bool,
    /// Why the final result event reports a failure (`is_error` true or a
    /// `subtype` other than `success`), such as `result: error_max_turns`.
    pub result_error: Option<String>,
}

/// Reads the init and result events of a stream-json transcript.
pub fn parse_transcript(text: &str) -> TranscriptInfo {
    let mut info = TranscriptInfo::default();
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match v.get("type").and_then(Value::as_str) {
            Some("system") if v.get("subtype").and_then(Value::as_str) == Some("init") => {
                if info.model.is_none() {
                    info.model = v.get("model").and_then(Value::as_str).map(str::to_owned);
                }
            }
            Some("result") => {
                info.has_result = true;
                let subtype = v.get("subtype").and_then(Value::as_str);
                let is_error = v.get("is_error").and_then(Value::as_bool) == Some(true);
                info.result_error = match subtype {
                    Some(st) if st != "success" => Some(format!("result: {st}")),
                    _ if is_error => Some("result: is_error".to_owned()),
                    _ => None,
                };
                let u = v.get("usage");
                let get = |k: &str| {
                    u.and_then(|u| u.get(k))
                        .and_then(Value::as_u64)
                        .unwrap_or(0)
                };
                info.usage = Usage {
                    input_tokens: get("input_tokens"),
                    output_tokens: get("output_tokens"),
                    cache_read_input_tokens: get("cache_read_input_tokens"),
                    cache_creation_input_tokens: get("cache_creation_input_tokens"),
                };
                info.cost_usd = v.get("total_cost_usd").and_then(Value::as_f64);
                info.num_turns = v.get("num_turns").and_then(Value::as_u64);
            }
            _ => {}
        }
    }
    info
}

struct Ctx<'a> {
    args: &'a RunArgs,
    out_root: PathBuf,
    work_root: PathBuf,
    variant_spec: Option<VariantSpec>,
    dla_bin: Option<PathBuf>,
    deny_read: Vec<PathBuf>,
    deny_write: Vec<PathBuf>,
    probe_prompt: Option<String>,
    /// What the fixtures' `[toolchain]` tables need, by fixture.
    toolchains: BTreeMap<String, Provided>,
}

fn run_dir_of(ctx: &Ctx, t: &Task) -> PathBuf {
    ctx.out_root
        .join("runs")
        .join(ctx.args.ablation.as_str())
        .join(format!("archetypes-{}", ctx.args.archetypes.as_str()))
        .join(&t.fixture)
        .join(t.n.to_string())
}

/// An opaque, stable name for one run's working directory: the first 12 hex digits of a
/// SHA-256 of the arm, the fixture and the run number. The child's paths carry neither the
/// fixture id nor the arm; `run.json` records the mapping.
pub fn work_key(scope: &str, ablation: &str, archetypes: &str, fixture: &str, n: u32) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for part in [scope, ablation, archetypes, fixture, &n.to_string()] {
        h.update(part.as_bytes());
        h.update([0]);
    }
    h.finalize()
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn work_key_of(ctx: &Ctx, t: &Task) -> String {
    let scope = ctx
        .out_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    work_key(
        &scope,
        ctx.args.ablation.as_str(),
        ctx.args.archetypes.as_str(),
        &t.fixture,
        t.n,
    )
}

fn copy_if_exists(from: &Path, to: &Path) {
    // `symlink_metadata`: a link in an agent-writable directory is not followed.
    if fs::symlink_metadata(from).is_ok_and(|m| m.file_type().is_file()) {
        let _ = fs::copy(from, to);
    }
}

fn collect_dla_files(project: &Path, out_dir: &Path) {
    copy_if_exists(
        &project.join("findings.json"),
        &out_dir.join("findings.json"),
    );
    let Ok(current) = fs::read_to_string(project.join(".dla/current")) else {
        return;
    };
    let id = current.trim();
    if id.is_empty() || id.contains(['/', '\\']) {
        return;
    }
    collect_run_files(&project.join(".dla").join(id), out_dir);
}

/// Lists the results directory among the denied paths unless a denied tree already covers
/// it. Returns true when it was added: a results directory outside the repository and the
/// fixtures tree shows its path (which can carry the arm name) in the child's settings.
///
/// The paths are compared in canonical form (a `..` or a link in `--out` must not make an
/// outside directory look inside); a path that does not exist is compared as given.
fn add_results_dir(denied: &mut Vec<PathBuf>, out_root: &Path) -> bool {
    let canon = |p: &Path| fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let real = canon(out_root);
    if denied.iter().any(|d| real.starts_with(canon(d))) {
        return false;
    }
    denied.push(out_root.to_path_buf());
    true
}

/// The denied-read paths that lie inside the writable system temporary directory:
/// the sandbox must not write there either, or a child could overwrite the parent
/// session's files (the scratch directory of the orchestrating session, for one).
pub fn deny_write_under_shared_tmp(deny_read: &[PathBuf]) -> Vec<PathBuf> {
    let bases = [PathBuf::from(SANDBOX_SHARED_WRITE), std::env::temp_dir()];
    let mut out: Vec<PathBuf> = deny_read
        .iter()
        .filter(|p| bases.iter().any(|b| p.starts_with(b)))
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Whether `text` is a budget `dla` accepts: a positive integer and `s`, `m` or `h`,
/// whose value in seconds fits in a `u64` (as `dla` checks).
pub fn valid_budget(text: &str) -> bool {
    let Some(unit) = text.chars().last() else {
        return false;
    };
    let mult: u64 = match unit {
        's' => 1,
        'm' => 60,
        'h' => 3600,
        _ => return false,
    };
    let digits = &text[..text.len() - unit.len_utf8()];
    !digits.is_empty()
        && digits.chars().all(|c| c.is_ascii_digit())
        && digits
            .parse::<u64>()
            .is_ok_and(|n| n > 0 && n.checked_mul(mult).is_some())
}

/// Whether `p` lies in the shared temporary directory that the sandbox lets every child
/// write (`/tmp`, the system temp directory, `/private/tmp`, and their resolved forms).
/// `p` need not exist: its deepest existing ancestor is resolved.
pub fn in_shared_tmp(p: &Path) -> bool {
    let mut bases = vec![
        PathBuf::from(SANDBOX_SHARED_WRITE),
        PathBuf::from("/private/tmp"),
        std::env::temp_dir(),
    ];
    let resolved: Vec<PathBuf> = bases
        .iter()
        .filter_map(|b| fs::canonicalize(b).ok())
        .collect();
    bases.extend(resolved);
    let mut rest = PathBuf::new();
    let mut anchor = p;
    let resolved_p = loop {
        if let Ok(c) = fs::canonicalize(anchor) {
            break c.join(&rest);
        }
        match (anchor.parent(), anchor.file_name()) {
            (Some(parent), Some(name)) => {
                rest = Path::new(name).join(&rest);
                anchor = parent;
            }
            _ => break p.to_path_buf(),
        }
    };
    bases
        .iter()
        .any(|b| p.starts_with(b) || resolved_p.starts_with(b))
}

/// The system temporary directory, writable for Bash only: JVM attach sockets and
/// `hsperfdata` live there, so `jcmd` cannot reach a JVM otherwise. The Edit and
/// Write tools do not get it.
pub const SANDBOX_SHARED_WRITE: &str = "/tmp";

/// Name of the file in the output directory that lists what collection left out.
const COLLECT_LOG: &str = "collect-skipped.txt";

/// Copies the files of a dla run directory that grading and inspection read:
/// the ledger, state, report and coverage, the M3 series, trends and alloc-test
/// results (`S*.json`, `T*.json`, `A*.json`), and the `harness.json` and `run-*.json`
/// files of `harness/H*` (never `work/`, `agent/`, `out/` or `logs/`). Directories
/// left by an earlier attempt are replaced.
///
/// The run directory is writable by the sandboxed model, so nothing is followed through a
/// symbolic link, only regular files are copied, and depth, entry count and bytes are
/// capped. What was skipped is logged to stderr and written to `collect-skipped.txt`.
pub fn collect_run_files(run: &Path, out_dir: &Path) {
    for f in ["ledger.jsonl", "state.json", "report.md", "coverage.json"] {
        copy_if_exists(&run.join(f), &out_dir.join(f));
    }
    let mut c = Collector::default();
    c.collect_worker_ledgers(run, out_dir);
    c.collect_bundle_indexes(run, out_dir);
    c.collect_seals(run, out_dir);
    for (d, prefix) in [("series", 'S'), ("trend", 'T'), ("alloc", 'A')] {
        let _ = fs::remove_dir_all(out_dir.join(d));
        let from = run.join(d);
        if Collector::is_real_dir(&from) {
            c.copy_dir(&from, &out_dir.join(d), prefix, 0);
        } else if fs::symlink_metadata(&from).is_ok() {
            c.skip(&from, "not a directory");
        }
    }
    let _ = fs::remove_dir_all(out_dir.join("harness"));
    let harness = run.join("harness");
    if Collector::is_real_dir(&harness)
        && let Ok(rd) = fs::read_dir(&harness)
    {
        for h in rd.flatten() {
            let name = h.file_name().to_string_lossy().into_owned();
            if !name.starts_with('H') || !Collector::is_real_dir(&h.path()) {
                continue;
            }
            c.entries += 1;
            if c.entries > COLLECT_MAX_ENTRIES {
                c.skip(&harness, "entry limit reached");
                break;
            }
            let dest = out_dir.join("harness").join(&name);
            let _ = fs::create_dir_all(&dest);
            c.copy_file(&h.path().join("harness.json"), &dest.join("harness.json"));
            if let Ok(files) = fs::read_dir(h.path()) {
                for f in files.flatten() {
                    c.entries += 1;
                    if c.entries > COLLECT_MAX_ENTRIES {
                        c.skip(&h.path(), "entry limit reached");
                        break;
                    }
                    let fname = f.file_name().to_string_lossy().into_owned();
                    if fname.starts_with("run-") && fname.ends_with(".json") {
                        c.copy_file(&f.path(), &dest.join(&fname));
                    }
                }
            }
        }
    }
    let _ = fs::remove_file(out_dir.join(COLLECT_LOG));
    if !c.skipped.is_empty() {
        eprintln!(
            "collect: left out {} item(s) of {} (see {COLLECT_LOG})",
            c.skipped.len(),
            run.display()
        );
        let _ = fs::write(out_dir.join(COLLECT_LOG), c.skipped.join("\n") + "\n");
    }
}

/// What the run's `state.json` says about how it ended: `incomplete_reason` and the number
/// of times the stop hook blocked the stop (`stop_blocks`). `(None, None)` when the file is
/// missing or unreadable (the run never started, or `dla` was not used).
pub fn read_state_facts(state: &Path) -> (Option<String>, Option<u32>) {
    let Some(v) = fs::read_to_string(state)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    else {
        return (None, None);
    };
    let reason = v
        .get("incomplete_reason")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let blocks = Some(
        v.get("stop_blocks")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
    );
    (reason, blocks)
}

fn execute(ctx: &Ctx, t: &Task) -> Res<RunJson> {
    let args = ctx.args;
    let out_dir = run_dir_of(ctx, t);
    fs::create_dir_all(&out_dir).map_err(|e| format!("{}: {e}", out_dir.display()))?;
    for f in RUN_ARTIFACTS {
        remove_file_if_exists(&out_dir.join(f));
    }
    remove_worker_ledgers(&out_dir);
    let _ = fs::remove_dir_all(out_dir.join("bundle"));
    let work_key = work_key_of(ctx, t);
    let work = ctx.work_root.join(&work_key);
    let src = args.fixtures_dir.join(&t.fixture).join("project");
    let project = prepare_workdir(&src, &work)?;
    let config = work.join("config");
    fs::create_dir_all(&config).map_err(|e| e.to_string())?;
    let cache = work.join("cache");
    for sub in CACHE_DIRS {
        fs::create_dir_all(cache.join(sub)).map_err(|e| format!("{}: {e}", cache.display()))?;
    }
    link_go_toolchain(&cache);
    let provided = ctx.toolchains.get(&t.fixture);
    if let Some(p) = provided {
        p.link_go(&cache);
    }
    let tmp = work.join("tmp");
    fs::create_dir_all(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
    let settings_path = work.join("settings.json");
    let writable = [project.clone(), cache.clone(), tmp.clone()];
    let mut settings = serde_json::to_string_pretty(&sandbox_settings(
        &ctx.deny_read,
        &writable,
        &[PathBuf::from(SANDBOX_SHARED_WRITE)],
        &ctx.deny_write,
    ))
    .map_err(|e| e.to_string())?;
    settings.push('\n');
    fs::write(&settings_path, settings).map_err(|e| format!("{}: {e}", settings_path.display()))?;
    let mut plugin_dir = None;
    if let Some(spec) = &ctx.variant_spec {
        let dest = work.join("plugin");
        build_variant(spec, &dest)?;
        plugin_dir = Some(dest);
    }
    let prompt = match &ctx.probe_prompt {
        Some(p) => p.clone(),
        None => build_prompt(args.ablation, &t.expected.task.prompt),
    };
    let mut cmd = Command::new(&args.claude_bin);
    cmd.arg("-p")
        .arg(&prompt)
        .args(["--output-format", "stream-json", "--verbose"])
        .args(["--max-turns", &args.max_turns.to_string()])
        .arg("--settings")
        .arg(&settings_path)
        .arg("--strict-mcp-config")
        .arg("--no-session-persistence");
    if let Some(m) = &args.model {
        cmd.args(["--model", m]);
    }
    if let Some(p) = &plugin_dir {
        cmd.arg("--plugin-dir").arg(p);
    }
    let mut env = child_env(
        std::env::vars(),
        &config,
        &cache,
        &tmp,
        ctx.dla_bin
            .as_deref()
            .filter(|_| args.ablation.uses_binary()),
        args,
    );
    if let Some(p) = provided {
        p.apply(&mut env);
    }
    let toolchain = Some(observe(&env, &project, &t.expected)).filter(|v| !v.is_empty());
    cmd.current_dir(&project)
        .env_clear()
        .envs(env)
        .stdin(Stdio::null());
    let transcript_path = out_dir.join("transcript.jsonl");
    let stdout = fs::File::create(&transcript_path).map_err(|e| e.to_string())?;
    let stderr = fs::File::create(out_dir.join("stderr.log")).map_err(|e| e.to_string())?;
    cmd.stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr));
    let started_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let (finished, spawn_error) = match spawn_and_wait(cmd, Duration::from_secs(args.timeout)) {
        Ok(f) => (f, None),
        Err(e) => (
            Finished {
                exit_code: -1,
                timed_out: false,
                wall: Duration::ZERO,
            },
            Some(e),
        ),
    };
    if guard::stop_signal().is_some() {
        // An interrupted attempt leaves no run.json: it is not a result.
        return Err("interrupted".to_owned());
    }
    collect_dla_files(&project, &out_dir);
    let mut text = String::new();
    if let Ok(mut f) = fs::File::open(&transcript_path) {
        let mut bytes = Vec::new();
        let _ = f.read_to_end(&mut bytes);
        text = String::from_utf8_lossy(&bytes).into_owned();
    }
    let info = parse_transcript(&text);
    let error = if let Some(e) = spawn_error {
        Some(e)
    } else if finished.timed_out {
        Some(format!("timed out after {} s", args.timeout))
    } else if !info.has_result {
        Some("no result event in the transcript".to_owned())
    } else {
        info.result_error.clone()
    };
    let (incomplete_reason, stop_blocks) = if args.ablation.uses_binary() {
        read_state_facts(&out_dir.join("state.json"))
    } else {
        (None, None)
    };
    let rj = RunJson {
        schema_version: 1,
        fixture: t.fixture.clone(),
        ablation: args.ablation.as_str().to_owned(),
        archetypes: args.archetypes.as_str().to_owned(),
        run: t.n,
        exit_code: finished.exit_code,
        timed_out: finished.timed_out,
        wall_seconds: (finished.wall.as_secs_f64() * 1000.0).round() / 1000.0,
        usage: info.usage,
        cost_usd: info.cost_usd,
        num_turns: info.num_turns,
        model: info.model,
        started_at,
        error: error.clone(),
        incomplete_reason,
        stop_blocks,
        toolchain,
        work_key: Some(work_key),
    };
    let mut body = serde_json::to_string_pretty(&rj).map_err(|e| e.to_string())?;
    body.push('\n');
    fs::write(out_dir.join("run.json"), body).map_err(|e| e.to_string())?;
    if finished.exit_code == 0 && error.is_none() && !args.keep_work {
        let _ = fs::remove_dir_all(&work);
    }
    Ok(rj)
}

fn worker(ctx: &Ctx, shared: &Shared, log: &Mutex<()>) {
    loop {
        if shared.aborted.load(Ordering::SeqCst) || guard::stop_signal().is_some() {
            return;
        }
        if let Some(limit) = ctx.args.budget_usd
            && *shared.spent.lock().expect("lock") >= limit
        {
            return;
        }
        let Some(task) = shared.queue.lock().expect("lock").pop_front() else {
            return;
        };
        let rd = run_dir_of(ctx, &task);
        if run_finished(&rd, ctx.args.retry_errored) {
            let _g = log.lock().expect("lock");
            println!("{} #{}: already done, skipped", task.fixture, task.n);
            continue;
        }
        let res = execute(ctx, &task);
        let _g = log.lock().expect("lock");
        match res {
            Ok(rj) => {
                let u = &rj.usage;
                let empty = rj.num_turns.unwrap_or(0) == 0
                    && u.input_tokens
                        + u.output_tokens
                        + u.cache_read_input_tokens
                        + u.cache_creation_input_tokens
                        == 0;
                {
                    let mut streak = shared.empty_streak.lock().expect("lock");
                    *streak = if empty { *streak + 1 } else { 0 };
                    if *streak >= MAX_EMPTY_STREAK {
                        shared.aborted.store(true, Ordering::SeqCst);
                    }
                }
                let mut spent = shared.spent.lock().expect("lock");
                *spent += rj.cost_usd.unwrap_or(0.0);
                println!(
                    "{} #{}: exit {} in {:.1} min, cost ${:.4}, total ${:.4}{}",
                    task.fixture,
                    task.n,
                    rj.exit_code,
                    rj.wall_seconds / 60.0,
                    rj.cost_usd.unwrap_or(0.0),
                    *spent,
                    rj.error.map(|e| format!(" ({e})")).unwrap_or_default()
                );
            }
            Err(e) => eprintln!("{} #{}: runner error: {e}", task.fixture, task.n),
        }
    }
}

fn ensure_dla_bin(args: &RunArgs) -> Res<PathBuf> {
    if let Some(p) = &args.dla_bin {
        let p = fs::canonicalize(p).map_err(|e| format!("{}: {e}", p.display()))?;
        return Ok(p);
    }
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "dla-cli"])
        .current_dir(&args.repo_root)
        .status()
        .map_err(|e| format!("cargo build: {e}"))?;
    if !status.success() {
        return Err("cargo build --release -p dla-cli failed".to_owned());
    }
    let exe = if cfg!(windows) { "dla.exe" } else { "dla" };
    let p = args.repo_root.join("target/release").join(exe);
    fs::canonicalize(&p).map_err(|e| format!("{}: {e}", p.display()))
}

/// Runs the benchmark; the summary line to print at the end.
pub fn run(args: &RunArgs) -> Res<String> {
    if args.runs == 0 {
        return Err("--runs must be at least 1".to_owned());
    }
    let out_root = args
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from("bench/results").join(args.ablation.as_str()));
    let out_root = absolute(&out_root)?;
    let results_name = out_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "results".to_owned());
    let work_root = match &args.work_root {
        Some(p) => absolute(p)?,
        None => default_work_root()?,
    }
    .join(work_key(&results_name, "", "", "", 0));
    let repo_abs = absolute(&args.repo_root)?;
    if work_root.starts_with(&repo_abs) {
        return Err(format!(
            "--work-root {} is inside the repository {}; working copies must live outside it",
            work_root.display(),
            repo_abs.display()
        ));
    }
    if in_shared_tmp(&work_root) {
        return Err(format!(
            "--work-root {} is inside the system temporary directory, which every child can write; \
             pass a --work-root outside it",
            work_root.display()
        ));
    }
    if !valid_budget(&args.child_budget) {
        return Err(format!(
            "--child-budget {:?} is not a duration; use seconds, minutes or hours such as 90s, 20m or 2h",
            args.child_budget
        ));
    }
    let mut args = args.clone();
    args.fixtures_dir = absolute(&args.fixtures_dir)?;
    args.repo_root = repo_abs.clone();

    // Fixtures.
    let ids: Vec<String> = if args.fixtures == "all" {
        fixture_dirs(&args.fixtures_dir)?
            .iter()
            .filter(|d| d.join("project").is_dir() && d.join("expected.toml").is_file())
            .filter_map(|d| d.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect()
    } else {
        args.fixtures
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect()
    };
    if ids.is_empty() {
        return Err(format!(
            "no fixtures found in {}",
            args.fixtures_dir.display()
        ));
    }
    let mut expected: Vec<(String, Expected)> = Vec::new();
    for id in &ids {
        expected.push((id.clone(), load_expected(&args.fixtures_dir, id)?));
    }
    let order = interleave(
        expected
            .iter()
            .map(|(id, e)| {
                (
                    id.clone(),
                    serde_json::to_value(e.runtime)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_owned))
                        .unwrap_or_default(),
                )
            })
            .collect(),
    );
    let mut queue = VecDeque::new();
    for n in 1..=args.runs {
        for id in &order {
            let exp = expected
                .iter()
                .find(|(i, _)| i == id)
                .map(|(_, e)| e.clone())
                .expect("fixture is in the list");
            queue.push_back(Task {
                fixture: id.clone(),
                n,
                expected: exp,
            });
        }
    }
    let total = queue.len();
    fs::create_dir_all(&out_root).map_err(|e| format!("{}: {e}", out_root.display()))?;
    let _lock = guard::Lock::acquire(&out_root)?;
    guard::install()?;
    let dla_bin = if args.ablation.uses_binary() {
        let built = ensure_dla_bin(&args)?;
        // The repository is unreadable to the child: it gets its own copy.
        let dest_dir = work_root.join("bin");
        fs::create_dir_all(&dest_dir).map_err(|e| format!("{}: {e}", dest_dir.display()))?;
        let dest = dest_dir.join(if cfg!(windows) { "dla.exe" } else { "dla" });
        fs::copy(&built, &dest).map_err(|e| format!("{}: {e}", dest.display()))?;
        Some(dest)
    } else {
        None
    };
    let variant_spec = args.ablation.uses_plugin().then(|| VariantSpec {
        repo_root: repo_abs.clone(),
        ablations_dir: args
            .ablations_dir
            .clone()
            .unwrap_or_else(|| repo_abs.join("bench/ablations")),
        ablation: args.ablation,
        archetypes_off: args.archetypes == Archetypes::Off,
    });
    // The results directory is named after the arm (`bench/results/full`); the child's
    // settings list the denied paths, so a results directory inside a denied tree is not listed.
    let mut denied: Vec<PathBuf> = vec![repo_abs.clone(), args.fixtures_dir.clone()];
    if add_results_dir(&mut denied, &out_root) {
        eprintln!(
            "warning: --out {} lies outside the repository, so its path is listed in the child's \
             denyRead setting (the child cannot read the directory, but it can see the path, \
             which may carry the arm name); give --out a directory inside the repository (the \
             default bench/results) to keep the path out of its settings",
            out_root.display()
        );
    }
    if let Some(a) = &args.ablations_dir {
        denied.push(absolute(a)?);
    }
    if let Ok(home) = std::env::var("HOME") {
        denied.push(PathBuf::from(home).join(".claude"));
    }
    if let Ok(c) = std::env::var("CLAUDE_CONFIG_DIR") {
        denied.push(PathBuf::from(c));
    }
    for p in &args.deny_read {
        denied.push(absolute(p)?);
    }
    let deny_read = normalise_paths(denied);
    if let Some(bad) = deny_read.iter().find(|d| work_root.starts_with(d)) {
        return Err(format!(
            "--work-root {} is inside the denied path {}",
            work_root.display(),
            bad.display()
        ));
    }
    if let Some(bad) = deny_read.iter().find(|d| covers_shared_tmp(d)) {
        return Err(format!(
            "--deny-read {} covers Claude Code's shared temporary directory, which every child \
             session writes to (its Bash tool and TMPDIR live there); deny a narrower path such as \
             the parent session's own subdirectory",
            bad.display()
        ));
    }
    let deny_write = deny_write_under_shared_tmp(&deny_read);
    let probe_prompt = match &args.probe_prompt {
        Some(p) => Some(fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?),
        None => None,
    };
    let toolchain_requests: Vec<(String, Toolchain)> = expected
        .iter()
        .filter_map(|(id, e)| e.toolchain.clone().map(|t| (id.clone(), t)))
        .collect();
    let toolchains = provide_all(
        &Host::system(),
        &work_root.join("toolchains"),
        &toolchain_requests,
    )?;
    let ctx = Ctx {
        args: &args,
        out_root: out_root.clone(),
        work_root,
        variant_spec,
        dla_bin,
        deny_read,
        deny_write,
        probe_prompt,
        toolchains,
    };
    let earlier = finished_cost(&out_root, args.retry_errored);
    let shared = Shared {
        queue: Mutex::new(queue),
        spent: Mutex::new(earlier),
        empty_streak: Mutex::new(0),
        aborted: AtomicBool::new(false),
    };
    let log = Mutex::new(());
    let workers = args.parallel.max(1);
    thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| worker(&ctx, &shared, &log));
        }
    });
    guard::wait_for_exit();
    if let Some(sig) = guard::stop_signal() {
        return Err(format!("interrupted by signal {sig}"));
    }
    let left = shared.queue.lock().expect("lock").len();
    let spent = *shared.spent.lock().expect("lock");
    if shared.aborted.load(Ordering::SeqCst) {
        return Err(format!(
            "aborted: {MAX_EMPTY_STREAK} consecutive runs ended with zero turns and zero tokens, \
             which points to a systematic failure (authentication, quota or network); \
             {left} of {total} run(s) were not started. Check stderr.log of the last runs, fix \
             the cause and start the same command again with --retry-errored. Results in {}",
            out_root.display()
        ));
    }
    Ok(format!(
        "done: {} of {total} run(s) attempted, {left} not started, cost ${:.4} (${spent:.4} in the results directory), results in {}",
        total - left,
        spent - earlier,
        out_root.display()
    ))
}

/// Each path in absolute form and, when it exists, in canonical form too;
/// sorted, without duplicates.
fn normalise_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for p in paths {
        if let Ok(c) = fs::canonicalize(&p) {
            out.push(c);
        }
        out.push(p);
    }
    out.sort();
    out.dedup();
    out
}

fn absolute(p: &Path) -> Res<PathBuf> {
    if p.is_absolute() {
        Ok(p.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|c| c.join(p))
            .map_err(|e: io::Error| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_results_dir_inside_a_denied_tree_is_not_listed_and_one_outside_is() {
        let mut denied = vec![
            PathBuf::from("/repo"),
            PathBuf::from("/repo/bench/fixtures"),
        ];
        // Inside the repository: the arm name stays out of the settings.
        assert!(!add_results_dir(
            &mut denied,
            Path::new("/repo/bench/results/full")
        ));
        assert_eq!(denied.len(), 2);
        // Outside: listed, and the caller warns.
        assert!(add_results_dir(&mut denied, Path::new("/data/m7-full-on")));
        assert_eq!(denied.last().unwrap(), Path::new("/data/m7-full-on"));
    }

    #[cfg(unix)]
    #[test]
    fn a_results_dir_that_leaves_the_repository_by_dotdot_or_a_link_is_listed() {
        let t = tempfile::tempdir().unwrap();
        let repo = t.path().join("repo");
        fs::create_dir_all(repo.join("bench/results")).unwrap();
        fs::create_dir_all(t.path().join("outside")).unwrap();
        let repo = fs::canonicalize(&repo).unwrap();
        // `..` out of the repository: the text starts with the repository path.
        let dotdot = repo.join("bench/results/../../../outside");
        assert!(dotdot.starts_with(&repo));
        let mut denied = vec![repo.clone()];
        assert!(add_results_dir(&mut denied, &dotdot));
        // A link under the results directory that points outside.
        std::os::unix::fs::symlink(t.path().join("outside"), repo.join("bench/results/link"))
            .unwrap();
        let mut denied = vec![repo.clone()];
        assert!(add_results_dir(
            &mut denied,
            &repo.join("bench/results/link")
        ));
        // A `..` that stays inside is still covered.
        let mut denied = vec![repo.clone()];
        assert!(!add_results_dir(
            &mut denied,
            &repo.join("bench/results/../results")
        ));
        assert_eq!(denied.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn collecting_a_run_never_follows_links_and_copies_only_expected_files() {
        use std::os::unix::fs::symlink;
        let t = tempfile::tempdir().unwrap();
        let (run, out, secret) = (
            t.path().join("run"),
            t.path().join("out"),
            t.path().join("host"),
        );
        fs::create_dir_all(&secret).unwrap();
        fs::write(secret.join("S9.json"), "host secret").unwrap();
        fs::create_dir_all(run.join("series")).unwrap();
        fs::create_dir_all(run.join("harness/H1")).unwrap();
        fs::create_dir_all(&out).unwrap();
        fs::write(run.join("series/S1.json"), "{}").unwrap();
        fs::write(run.join("series/notes.txt"), "x").unwrap();
        fs::write(run.join("harness/H1/harness.json"), "{}").unwrap();
        fs::write(run.join("harness/H1/run-1.json"), "{}").unwrap();
        // A link to a host directory, a link to a host file, and a loop.
        symlink(&secret, run.join("series/dir")).unwrap();
        symlink(secret.join("S9.json"), run.join("series/S2.json")).unwrap();
        symlink(&run, run.join("series/loop")).unwrap();
        symlink(&secret, run.join("alloc")).unwrap();
        symlink(secret.join("S9.json"), run.join("harness/H1/run-2.json")).unwrap();
        symlink(secret.join("S9.json"), run.join("ledger.jsonl")).unwrap();
        collect_run_files(&run, &out);
        assert!(out.join("series/S1.json").is_file());
        assert!(!out.join("series/notes.txt").exists());
        assert!(!out.join("series/dir").exists() && !out.join("series/S2.json").exists());
        assert!(!out.join("series/loop").exists());
        assert!(!out.join("alloc").exists());
        assert!(!out.join("ledger.jsonl").exists());
        assert!(out.join("harness/H1/harness.json").is_file());
        assert!(out.join("harness/H1/run-1.json").is_file());
        assert!(!out.join("harness/H1/run-2.json").exists());
        let log = fs::read_to_string(out.join(COLLECT_LOG)).unwrap();
        assert!(
            log.contains("symbolic link") && log.contains("unexpected file name"),
            "{log}"
        );
        // Nothing from the host directory was copied anywhere.
        let mut stack = vec![out.clone()];
        while let Some(d) = stack.pop() {
            for e in fs::read_dir(&d).unwrap().flatten() {
                if e.file_type().unwrap().is_dir() {
                    stack.push(e.path());
                } else {
                    assert_ne!(
                        fs::read_to_string(e.path()).unwrap_or_default(),
                        "host secret"
                    );
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn collecting_worker_ledgers_and_bundle_indexes() {
        use std::os::unix::fs::symlink;
        let t = tempfile::tempdir().unwrap();
        let (run, out, secret) = (
            t.path().join("run"),
            t.path().join("out"),
            t.path().join("host"),
        );
        fs::create_dir_all(&secret).unwrap();
        fs::write(secret.join("x"), "host secret").unwrap();
        fs::create_dir_all(run.join("bundle/B1")).unwrap();
        fs::create_dir_all(run.join("bundle/B2")).unwrap();
        fs::create_dir_all(run.join("bundle/B3")).unwrap();
        fs::create_dir_all(&out).unwrap();
        fs::write(run.join("ledger.jsonl"), "merged\n").unwrap();
        fs::write(run.join("ledger.jvm.jsonl"), "jvm\n").unwrap();
        fs::write(run.join("ledger.rust.jsonl"), "rust\n").unwrap();
        fs::write(run.join("ledger..jsonl"), "no worker").unwrap();
        fs::write(run.join("ledger.jvm.json"), "wrong suffix").unwrap();
        fs::write(run.join("ledgerx.jsonl"), "wrong prefix").unwrap();
        symlink(secret.join("x"), run.join("ledger.evil.jsonl")).unwrap();
        fs::create_dir_all(run.join("ledger.dir.jsonl")).unwrap();
        fs::write(run.join("bundle/B1/index.json"), "{}").unwrap();
        fs::write(run.join("bundle/B1/raw.bin"), "not collected").unwrap();
        symlink(secret.join("x"), run.join("bundle/B2/index.json")).unwrap();
        symlink(&secret, run.join("bundle/B4")).unwrap();
        // Stale files of an earlier attempt.
        fs::write(out.join("ledger.old.jsonl"), "stale").unwrap();
        fs::create_dir_all(out.join("bundle/OLD")).unwrap();
        fs::write(out.join("bundle/OLD/index.json"), "stale").unwrap();
        collect_run_files(&run, &out);
        assert_eq!(
            fs::read_to_string(out.join("ledger.jsonl")).unwrap(),
            "merged\n"
        );
        assert_eq!(
            fs::read_to_string(out.join("ledger.jvm.jsonl")).unwrap(),
            "jvm\n"
        );
        assert!(out.join("ledger.rust.jsonl").is_file());
        for n in [
            "ledger..jsonl",
            "ledger.jvm.json",
            "ledgerx.jsonl",
            "ledger.evil.jsonl",
            "ledger.dir.jsonl",
            "ledger.old.jsonl",
        ] {
            assert!(!out.join(n).exists(), "{n}");
        }
        assert!(out.join("bundle/B1/index.json").is_file());
        assert!(!out.join("bundle/B1/raw.bin").exists());
        assert!(
            !out.join("bundle/B2").exists(),
            "a linked index is not copied"
        );
        assert!(!out.join("bundle/B3").exists(), "no index, no directory");
        assert!(!out.join("bundle/B4").exists());
        assert!(!out.join("bundle/OLD").exists());
        let log = fs::read_to_string(out.join(COLLECT_LOG)).unwrap();
        assert!(
            log.contains("ledger.evil.jsonl") && log.contains("symbolic link"),
            "{log}"
        );
    }

    #[test]
    fn worker_ledgers_respect_the_byte_cap() {
        let t = tempfile::tempdir().unwrap();
        let (run, out) = (t.path().join("run"), t.path().join("out"));
        fs::create_dir_all(&run).unwrap();
        fs::create_dir_all(&out).unwrap();
        let mut c = Collector {
            bytes: COLLECT_MAX_BYTES - 1,
            ..Collector::default()
        };
        fs::write(run.join("ledger.a.jsonl"), "too big for what is left").unwrap();
        c.collect_worker_ledgers(&run, &out);
        assert!(!out.join("ledger.a.jsonl").exists());
        assert!(c.skipped.iter().any(|s| s.contains("byte limit reached")));
    }

    #[test]
    fn default_work_root_is_under_the_cache_dir_not_tmp() {
        assert_eq!(
            default_work_root_from(Some("/x/cache"), Some("/home/u")),
            Some(PathBuf::from("/x/cache/dla-bench"))
        );
        assert_eq!(
            default_work_root_from(Some(""), Some("/home/u")),
            Some(PathBuf::from("/home/u/.cache/dla-bench"))
        );
        assert_eq!(
            default_work_root_from(None, Some("/home/u")),
            Some(PathBuf::from("/home/u/.cache/dla-bench"))
        );
        assert_eq!(default_work_root_from(None, None), None);
        assert_eq!(default_work_root_from(Some(""), Some("")), None);
        let d = default_work_root_from(None, Some("/home/u")).unwrap();
        assert!(!d.starts_with("/tmp"));
        assert!(!covers_shared_tmp(&d));
    }

    #[test]
    fn sandbox_tmp_is_writable_for_bash_only() {
        let deny = [PathBuf::from("/repo")];
        let allow = [PathBuf::from("/w/cache")];
        let v = sandbox_settings(&deny, &allow, &[PathBuf::from("/tmp")], &[]);
        assert_eq!(
            v["sandbox"]["filesystem"]["allowWrite"],
            serde_json::json!(["/w/cache", "/tmp"])
        );
        assert!(v["sandbox"]["filesystem"].get("denyWrite").is_none());
        let rules = serde_json::to_string(&v["permissions"]).unwrap();
        assert!(
            !rules.contains("//tmp"),
            "no Edit or Write rule for /tmp: {rules}"
        );
        let allowed: Vec<&str> = v["permissions"]["allow"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap())
            .collect();
        assert!(allowed.contains(&"Agent") && allowed.contains(&"Task"));
        let v = sandbox_settings(
            &deny,
            &allow,
            &[PathBuf::from("/tmp")],
            &[PathBuf::from("/tmp/claude-0/s")],
        );
        assert_eq!(
            v["sandbox"]["filesystem"]["denyWrite"],
            serde_json::json!(["/tmp/claude-0/s"])
        );
    }

    #[test]
    fn denied_paths_inside_tmp_are_write_denied_too() {
        let deny = [
            PathBuf::from("/repo"),
            PathBuf::from("/tmp/claude-0/-home-u/sess/scratchpad"),
            PathBuf::from("/tmp/claude-0/-home-u/sess/scratchpad"),
            PathBuf::from("/home/u/.claude"),
        ];
        assert_eq!(
            deny_write_under_shared_tmp(&deny),
            vec![PathBuf::from("/tmp/claude-0/-home-u/sess/scratchpad")]
        );
    }

    #[test]
    fn child_budget_reaches_the_child_and_parent_dla_vars_do_not() {
        let parent = vec![
            ("HOME".to_owned(), "/home/u".to_owned()),
            ("DLA_DEFAULT_BUDGET".to_owned(), "3h".to_owned()),
            ("DLA_OTHER".to_owned(), "x".to_owned()),
        ];
        let get = |env: &[(String, String)], k: &str| {
            env.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone())
        };
        for abl in [Ablation::NoSkill, Ablation::Full] {
            let env = child_env(
                parent.clone(),
                Path::new("/cfg"),
                Path::new("/w/cache"),
                Path::new("/w/tmp"),
                None,
                &args(abl, Archetypes::On),
            );
            assert_eq!(get(&env, "DLA_DEFAULT_BUDGET").as_deref(), Some("20m"));
            assert_eq!(get(&env, "DLA_OTHER"), None);
            assert_eq!(
                env.iter()
                    .filter(|(n, _)| n == "DLA_DEFAULT_BUDGET")
                    .count(),
                1
            );
        }
        let mut a = args(Ablation::Gates, Archetypes::On);
        a.child_budget = "90s".into();
        let env = child_env(
            parent,
            Path::new("/cfg"),
            Path::new("/w/cache"),
            Path::new("/w/tmp"),
            None,
            &a,
        );
        assert_eq!(get(&env, "DLA_DEFAULT_BUDGET").as_deref(), Some("90s"));
        assert!(!keep_env_var("DLA_DEFAULT_BUDGET") && !keep_env_var("DLA_BIN"));
        assert!(valid_budget("20m") && valid_budget("90s") && valid_budget("2h"));
        for bad in [
            "",
            "m",
            "20",
            "20x",
            "-5m",
            "1.5h",
            "0s",
            "18446744073709551615h",
            "18446744073709551615m",
            "5124095576030432h",
        ] {
            assert!(!valid_budget(bad), "{bad}");
        }
        assert!(valid_budget("18446744073709551615s") && valid_budget("5124095576030431h"));
    }

    #[test]
    fn shared_tmp_is_recognised_for_missing_paths_too() {
        assert!(in_shared_tmp(Path::new("/tmp/w/res")));
        assert!(in_shared_tmp(Path::new("/tmp")));
        assert!(in_shared_tmp(&std::env::temp_dir().join("nope/deeper")));
        assert!(!in_shared_tmp(Path::new("/home/u/.cache/dla-bench")));
        assert!(!in_shared_tmp(Path::new("/tmpfoo/w")));
    }

    #[test]
    fn collecting_a_run_caps_entries() {
        let t = tempfile::tempdir().unwrap();
        let (run, out) = (t.path().join("run"), t.path().join("out"));
        fs::create_dir_all(run.join("series")).unwrap();
        fs::create_dir_all(&out).unwrap();
        for i in 0..(COLLECT_MAX_ENTRIES + 20) {
            fs::write(run.join(format!("series/S{i}.json")), "{}").unwrap();
        }
        collect_run_files(&run, &out);
        let n = fs::read_dir(out.join("series")).unwrap().count();
        assert!(n <= COLLECT_MAX_ENTRIES, "{n}");
        let log = fs::read_to_string(out.join(COLLECT_LOG)).unwrap();
        assert!(log.contains("entry limit reached"));
    }

    #[test]
    fn denying_the_shared_claude_temp_dir_is_refused() {
        assert!(covers_shared_tmp(Path::new("/tmp")));
        assert!(covers_shared_tmp(Path::new("/tmp/claude-0")));
        assert!(covers_shared_tmp(Path::new("/tmp/claude-1000")));
        assert!(covers_shared_tmp(Path::new("/")));
        assert!(!covers_shared_tmp(Path::new(
            "/tmp/claude-0/-home-user-dla"
        )));
        assert!(!covers_shared_tmp(Path::new("/home/u/.cache/dla-bench")));
        assert!(!covers_shared_tmp(Path::new("/home/user/dla")));
    }

    #[test]
    fn strips_only_the_hooks_block_of_the_frontmatter() {
        let text = "---\nname: x\nhooks:\n  PreToolUse:\n    - matcher: Bash\n\n  # c\ndescription: d\n---\nhooks:\n  body stays\n";
        assert_eq!(
            strip_frontmatter_hooks(text),
            "---\nname: x\ndescription: d\n---\nhooks:\n  body stays\n"
        );
        let plain = "---\nname: x\n---\nbody\n";
        assert_eq!(strip_frontmatter_hooks(plain), plain);
        let no_front = "hooks:\n  a: b\n";
        assert_eq!(strip_frontmatter_hooks(no_front), no_front);
        // hooks as the last key, closing marker right after
        assert_eq!(
            strip_frontmatter_hooks("---\nname: x\nhooks:\n  A: b\n---\nz\n"),
            "---\nname: x\n---\nz\n"
        );
    }

    #[test]
    fn interleaves_fixtures_across_runtimes() {
        let items = [
            ("go-a", "go"),
            ("go-b", "go"),
            ("jvm-a", "jvm"),
            ("jvm-b", "jvm"),
            ("jvm-c", "jvm"),
            ("py-a", "python"),
        ]
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .to_vec();
        assert_eq!(
            interleave(items),
            ["go-a", "jvm-a", "py-a", "go-b", "jvm-b", "jvm-c"]
        );
    }

    #[test]
    fn reads_model_usage_cost_and_turns_from_the_stream() {
        let t = concat!(
            "{\"type\":\"system\",\"subtype\":\"init\",\"model\":\"m-1\"}\n",
            "garbage\n",
            "{\"type\":\"assistant\"}\n",
            "{\"type\":\"result\",\"total_cost_usd\":0.5,\"num_turns\":4,\"usage\":{\"input_tokens\":1,\"output_tokens\":2,\"cache_read_input_tokens\":3,\"cache_creation_input_tokens\":4}}\n"
        );
        let i = parse_transcript(t);
        assert_eq!(i.model.as_deref(), Some("m-1"));
        assert_eq!(i.cost_usd, Some(0.5));
        assert_eq!(i.num_turns, Some(4));
        assert_eq!(
            i.usage,
            Usage {
                input_tokens: 1,
                output_tokens: 2,
                cache_read_input_tokens: 3,
                cache_creation_input_tokens: 4
            }
        );
        assert!(i.has_result);
        assert!(!parse_transcript("").has_result);
    }

    #[test]
    fn result_event_failures_become_the_error_text() {
        let err = |line: &str| parse_transcript(line).result_error;
        assert_eq!(
            err(r#"{"type":"result","subtype":"error_max_turns","is_error":true}"#).as_deref(),
            Some("result: error_max_turns")
        );
        assert_eq!(
            err(r#"{"type":"result","subtype":"error_during_execution"}"#).as_deref(),
            Some("result: error_during_execution")
        );
        assert_eq!(
            err(r#"{"type":"result","subtype":"success","is_error":true}"#).as_deref(),
            Some("result: is_error")
        );
        assert_eq!(
            err(r#"{"type":"result","is_error":true}"#).as_deref(),
            Some("result: is_error")
        );
        assert_eq!(
            err(r#"{"type":"result","subtype":"success","is_error":false}"#),
            None
        );
        assert_eq!(err(r#"{"type":"result"}"#), None);
        // the last result event decides
        let two = "{\"type\":\"result\",\"subtype\":\"error_max_turns\"}\n{\"type\":\"result\",\"subtype\":\"success\"}\n";
        assert_eq!(parse_transcript(two).result_error, None);
    }

    #[test]
    fn errored_run_json_definition() {
        let j = |s: &str| serde_json::from_str::<Value>(s).unwrap();
        assert!(!run_json_errored(&j(
            r#"{"exit_code":0,"timed_out":false,"error":null}"#
        )));
        assert!(!run_json_errored(&j(r#"{"exit_code":0}"#)));
        assert!(run_json_errored(&j(r#"{"exit_code":2}"#)));
        assert!(run_json_errored(&j(r#"{"exit_code":0.0}"#)));
        assert!(run_json_errored(&j(r#"{}"#)));
        assert!(run_json_errored(&j(r#"{"exit_code":0,"timed_out":true}"#)));
        assert!(run_json_errored(&j(r#"{"exit_code":0,"error":"x"}"#)));
    }

    #[test]
    fn child_environment_drops_the_parent_session() {
        for k in [
            "CLAUDE_CODE_SESSION_ID",
            "CLAUDE_SESSION_ID",
            "CLAUDECODE",
            "CLAUDE_CODE_ENTRYPOINT",
            "CCR_SESSION_PROFILE",
            "GH_TOKEN",
            "AI_AGENT",
            "DLA_BIN",
        ] {
            assert!(!keep_env_var(k), "{k} must be dropped");
        }
        for k in [
            "PATH",
            "HOME",
            "HTTPS_PROXY",
            "SSL_CERT_FILE",
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_API_KEY",
            "CLAUDE_CODE_OAUTH_TOKEN",
        ] {
            assert!(keep_env_var(k), "{k} must be kept");
        }
    }

    fn args(abl: Ablation, arch: Archetypes) -> RunArgs {
        RunArgs {
            ablation: abl,
            archetypes: arch,
            runs: 1,
            fixtures: "all".into(),
            fixtures_dir: PathBuf::from("f"),
            parallel: 1,
            out: None,
            work_root: None,
            deny_read: Vec::new(),
            probe_prompt: None,
            model: None,
            max_turns: 5,
            child_budget: "20m".into(),
            timeout: 5,
            budget_usd: None,
            retry_errored: false,
            repo_root: PathBuf::from("."),
            ablations_dir: None,
            dla_bin: None,
            claude_bin: "claude".into(),
            keep_work: false,
        }
    }

    #[test]
    fn child_environment_settings_per_ablation() {
        let parent = vec![
            ("PATH".to_owned(), "/bin".to_owned()),
            ("HOME".to_owned(), "/home/u".to_owned()),
            ("CLAUDE_CODE_SESSION_ID".to_owned(), "parent".to_owned()),
            ("IS_SANDBOX".to_owned(), "0".to_owned()),
        ];
        let get = |env: &[(String, String)], k: &str| {
            env.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone())
        };
        let cache = Path::new("/w/cache");
        let plain = child_env(
            parent.clone(),
            Path::new("/cfg"),
            cache,
            Path::new("/w/tmp"),
            None,
            &args(Ablation::NoSkill, Archetypes::Off),
        );
        assert_eq!(get(&plain, "CLAUDE_CONFIG_DIR").as_deref(), Some("/cfg"));
        assert_eq!(get(&plain, "IS_SANDBOX"), None);
        assert_eq!(get(&plain, "CLAUDE_CODE_SESSION_ID"), None);
        assert_eq!(get(&plain, "DLA_CONSENT_TIERS"), None);
        assert_eq!(get(&plain, "DLA_ARCHETYPES"), None);
        let full = child_env(
            parent,
            Path::new("/cfg"),
            cache,
            Path::new("/w/tmp"),
            Some(Path::new("/x/dla")),
            &args(Ablation::Full, Archetypes::Off),
        );
        assert_eq!(get(&full, "DLA_CONSENT_TIERS").as_deref(), Some("1-4"));
        assert_eq!(get(&full, "DLA_CONSENT_RAW_VALUES").as_deref(), Some("1"));
        assert_eq!(get(&full, "DLA_ARCHETYPES").as_deref(), Some("off"));
        assert_eq!(get(&full, "DLA_BIN").as_deref(), Some("/x/dla"));
    }

    // `run` is unix-only (the sandbox exists only on Linux and macOS), and the
    // expected values use unix path separators.
    #[cfg(unix)]
    #[test]
    fn a_go_toolchain_is_linked_into_the_child_cache() {
        let t = std::env::temp_dir().join(format!("dla-gotc-{}", std::process::id()));
        let _ = fs::remove_dir_all(&t);
        let host = t.join("host");
        let name = "toolchain@v0.0.1-go1.27.0.linux-amd64";
        let dir = host.join("golang.org").join(name);
        fs::create_dir_all(dir.join("bin")).unwrap();
        let dl = host.join("cache/download/golang.org/toolchain/@v");
        fs::create_dir_all(&dl).unwrap();
        for f in ["zip", "ziphash", "lock"] {
            fs::write(dl.join(format!("v0.0.1-go1.27.0.linux-amd64.{f}")), f).unwrap();
        }
        fs::write(dl.join("v0.0.1-go1.26.0.linux-amd64.zip"), "other").unwrap();
        let lookup = host.join("cache/download/sumdb/sum.golang.org/lookup/golang.org");
        fs::create_dir_all(&lookup).unwrap();
        fs::write(lookup.join(name), "sum").unwrap();
        fs::write(
            host.join("cache/download/sumdb/sum.golang.org/latest"),
            "tree",
        )
        .unwrap();
        let cache = t.join("child");
        link_toolchain(&dir, &cache);
        let m = cache.join("gopath/pkg/mod");
        // The checksum database cache is copied, so the switch verifies offline.
        let sum = m.join("cache/download/sumdb/sum.golang.org");
        assert_eq!(fs::read_to_string(sum.join("latest")).unwrap(), "tree");
        let looked = sum.join("lookup/golang.org").join(name);
        assert!(!looked.symlink_metadata().unwrap().file_type().is_symlink());
        assert!(m.join("golang.org").join(name).join("bin").is_dir());
        let c = m.join("cache/download/golang.org/toolchain/@v");
        let zip = c.join("v0.0.1-go1.27.0.linux-amd64.zip");
        assert!(zip.symlink_metadata().unwrap().file_type().is_symlink());
        let hash = c.join("v0.0.1-go1.27.0.linux-amd64.ziphash");
        assert!(!hash.symlink_metadata().unwrap().file_type().is_symlink());
        assert!(!c.join("v0.0.1-go1.26.0.linux-amd64.zip").exists());
        link_toolchain(&dir, &cache);
        let _ = fs::remove_dir_all(&t);
    }

    #[test]
    fn go_toolchain_directory_names_follow_the_go_module_cache() {
        assert_eq!(
            go_toolchain_dir_name("go1.27.0", "linux", "x86_64"),
            "toolchain@v0.0.1-go1.27.0.linux-amd64"
        );
        assert_eq!(
            go_toolchain_dir_name("go1.27.0", "macos", "aarch64"),
            "toolchain@v0.0.1-go1.27.0.darwin-arm64"
        );
    }

    #[cfg(unix)]
    #[test]
    fn toolchain_caches_live_under_the_run_cache() {
        let parent = vec![("HOME".to_owned(), "/home/u".to_owned())];
        let env = child_env(
            parent,
            Path::new("/cfg"),
            Path::new("/w/cache"),
            Path::new("/w/tmp"),
            None,
            &args(Ablation::NoSkill, Archetypes::On),
        );
        let get = |k: &str| env.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
        assert_eq!(get("GOCACHE"), Some("/w/cache/go-build"));
        assert_eq!(get("GOMODCACHE"), Some("/w/cache/gopath/pkg/mod"));
        assert_eq!(get("GOTMPDIR"), Some("/w/cache/go-tmp"));
        assert_eq!(get("GOPROXY"), Some("off"));
        assert_eq!(get("MAVEN_OPTS"), Some("-Dmaven.repo.local=/w/cache/m2"));
        assert_eq!(get("npm_config_cache"), Some("/w/cache/npm"));
        assert_eq!(get("PIP_CACHE_DIR"), Some("/w/cache/pip"));
        assert_eq!(get("CARGO_HOME"), Some("/w/cache/cargo"));
        assert_eq!(get("RUSTUP_HOME"), Some("/home/u/.rustup"));
        assert_eq!(get("CARGO_NET_OFFLINE"), Some("true"));
    }

    #[test]
    fn settings_confine_bash_and_the_file_tools() {
        let deny = [PathBuf::from("/repo"), PathBuf::from("/root/.claude")];
        let allow = [PathBuf::from("/w/cache")];
        let v = sandbox_settings(&deny, &allow, &[], &[]);
        let sb = &v["sandbox"];
        assert_eq!(sb["enabled"], true);
        assert_eq!(sb["autoAllowBashIfSandboxed"], true);
        assert_eq!(sb["allowUnsandboxedCommands"], false);
        assert_eq!(sb["failIfUnavailable"], true);
        assert_eq!(
            sb["filesystem"]["denyRead"],
            serde_json::json!(["/repo", "/root/.claude"])
        );
        assert_eq!(
            sb["filesystem"]["allowWrite"],
            serde_json::json!(["/w/cache"])
        );
        assert!(sb.get("network").is_none(), "no network entries at all");
        let perms = &v["permissions"];
        assert_eq!(
            perms["allow"],
            serde_json::json!([
                "Bash",
                "Read",
                "Glob",
                "Grep",
                "Agent",
                "Task",
                "Edit(//w/cache/**)",
                "Write(//w/cache/**)"
            ]),
            "Edit and Write are scoped to the writable places"
        );
        let denied: Vec<&str> = perms["deny"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap())
            .collect();
        for tool in ["Read", "Edit", "Write", "Glob", "Grep"] {
            assert!(
                denied.contains(&format!("{tool}(//repo/**)").as_str()),
                "{tool}"
            );
            assert!(denied.contains(&format!("{tool}(//root/.claude/**)").as_str()));
        }
        assert!(denied.contains(&"WebFetch") && denied.contains(&"WebSearch"));
        assert!(v.get("permissionMode").is_none());
        assert!(v["permissions"].get("defaultMode").is_none());
    }

    #[test]
    fn prompts_per_ablation() {
        let t = "Find it.\n";
        let ns = build_prompt(Ablation::NoSkill, t);
        assert!(ns.starts_with("Find it.\n\nYou are in a local checkout"));
        assert!(ns.contains("./findings.json"));
        let prose = build_prompt(Ablation::Prose, t);
        assert!(prose.starts_with("/dla:detect-leak-anywhere Find it."));
        assert!(prose.contains("./findings.json"));
        for a in [Ablation::Gates, Ablation::Full] {
            let p = build_prompt(a, t);
            assert!(p.starts_with("/dla:detect-leak-anywhere Find it."));
            assert!(p.ends_with("for raw values."));
            assert!(!p.contains("findings.json"));
        }
    }
}
