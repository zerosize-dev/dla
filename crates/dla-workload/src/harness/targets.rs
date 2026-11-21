//! The targets of a harness: what `dla harness gen` prints for each finding a harness was
//! generated to reach, and the comment block it writes at the top of the driver.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dla_census::parse::{
    Lang, parse,
    tree_sitter::{Node, Point},
};
use dla_core::{
    FindingId, Ledger,
    evidence::EvidenceKind,
    harness::{ProbeSpec, ProbeTarget},
    toolctx::ToolCtx,
};
use serde_json::{Value, json};

use super::probes::{Asked, python_module};

/// How many writers and removers a target lists.
const SHOWN_SITES: usize = 3;

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

/// Node kinds that are a class-like body.
const CLASSES: &[&str] = &[
    "class_declaration",
    "class_definition",
    "class_specifier",
    "struct_specifier",
    "impl_item",
    "interface_declaration",
    "object_declaration",
];

/// What the generator prints about one target.
#[derive(Debug, Clone)]
pub struct Target {
    /// The finding.
    pub id: FindingId,
    /// `file:line`.
    pub site: String,
    /// Holder kind (`static/global`, `container-field`, ...) or the census kind.
    pub holder: String,
    /// The census kind (`cache-decorator`, ...), which picks the instruction.
    pub kind: String,
    /// The owner of the holder.
    pub owner: String,
    /// The rule's message, when a detector raised the finding.
    pub message: Option<String>,
    /// Up to three writers as `file:line (function)`, and how many there are.
    pub writers: (Vec<String>, usize),
    /// Up to three removers.
    pub removers: (Vec<String>, usize),
    /// What the probe reads; `None`: no probe.
    pub probe: Option<String>,
    /// One line of instruction by holder kind.
    pub instruction: String,
    /// `--next` added this target because it holds state of the same owner as this earlier
    /// target.
    pub sibling_of: Option<FindingId>,
    /// What an entry-path harness adds: the callers chain and where the key comes from.
    pub entry: Option<Entry>,
    /// Rust: the crate features that gate the module of the site (`#[cfg(feature = "x")]` on a
    /// `mod` declaration), read by [`feature_gate`]; `None` when no declaration gates it.
    pub feature_gate: Option<FeatureGate>,
}

/// The crate features a Rust site lies behind, and whether the driver build enables them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureGate {
    /// The features named by the `#[cfg(feature = "..")]` attributes of the `mod` declarations
    /// from the crate root down to the site's module.
    pub features: Vec<String>,
    /// Every one of them is among the features the harness is built with.
    pub enabled: bool,
}

impl FeatureGate {
    /// `behind feature x: generate with --features x`, or what the build already does.
    pub fn text(&self) -> String {
        let list = self.features.join(",");
        let plural = if self.features.len() > 1 {
            "features"
        } else {
            "feature"
        };
        if self.enabled {
            format!(
                "behind {plural} {}: the driver is built with --features {list}",
                self.features.join(", ")
            )
        } else {
            format!(
                "behind {plural} {}: generate with --features {list} (this harness is built without {})",
                self.features.join(", "),
                if self.features.len() > 1 {
                    "them"
                } else {
                    "it"
                }
            )
        }
    }
}

/// The features that gate the module a Rust file belongs to, read with a plain line scan of
/// the `mod` declarations from the file up to the crate root (`lib.rs`, `main.rs`): a
/// `#[cfg(feature = "x")]` attribute on the line or lines right above `mod name;` names `x`,
/// and so does a declaration inside `cfg_x! { .. }` when the crate defines that macro with
/// `macro_rules!` and a `feature = "x"` attribute in its body (the habit of crates that wrap
/// their gated items). `file` is project-relative.
///
/// It does not see: a module declared with `#[path = ".."]`, in another macro or inline
/// (`mod name { .. }`), a gate on the item itself instead of its module, `cfg_attr`, a `not(..)`
/// gate (skipped, not read as a requirement), a target-specific or platform gate, and a feature
/// another feature turns on. For `any(a, b)` it names the first, which is enough to enable.
pub fn feature_gate(project: &Path, file: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut current = Path::new(file).to_path_buf();
    for _ in 0..12 {
        let stem = current
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        if stem == "lib" || stem == "main" {
            break;
        }
        let dir = current.parent().map(Path::to_path_buf).unwrap_or_default();
        let (name, decl_dir) = if stem == "mod" {
            (
                dir.file_name().and_then(|s| s.to_str()).map(str::to_owned),
                dir.parent().map(Path::to_path_buf).unwrap_or_default(),
            )
        } else {
            (Some(stem), dir.clone())
        };
        let Some(name) = name else { break };
        // Where `mod name;` can be written: the file of the directory that holds the module.
        let mut parents: Vec<PathBuf> = ["lib.rs", "main.rs", "mod.rs"]
            .iter()
            .map(|f| decl_dir.join(f))
            .collect();
        if let (Some(dn), Some(up)) = (
            decl_dir.file_name().and_then(|s| s.to_str()),
            decl_dir.parent(),
        ) {
            parents.push(up.join(format!("{dn}.rs")));
        }
        let root = crate_root(project, &current);
        let macro_features = |m: &str| -> Vec<String> {
            root.as_deref()
                .map(|r| macro_features(project, r, m))
                .unwrap_or_default()
        };
        let mut next = None;
        for p in parents {
            let Ok(text) = fs::read_to_string(project.join(&p)) else {
                continue;
            };
            if let Some(features) = mod_gate(&text, &name, &macro_features) {
                for f in features {
                    if !found.contains(&f) {
                        found.push(f);
                    }
                }
                next = Some(p);
                break;
            }
        }
        let Some(p) = next else { break };
        current = p;
    }
    found.sort();
    found
}

/// The directory of the nearest `lib.rs` or `main.rs` at or above `file`'s directory.
fn crate_root(project: &Path, file: &Path) -> Option<PathBuf> {
    let mut dir = file.parent()?.to_path_buf();
    loop {
        if project.join(&dir).join("lib.rs").is_file()
            || project.join(&dir).join("main.rs").is_file()
        {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// The features a `cfg_x!` macro gates its items with: the first feature of each `#[cfg(..)]`
/// attribute in the body of `macro_rules! cfg_x`, found in the `.rs` files under `root`.
fn macro_features(project: &Path, root: &Path, name: &str) -> Vec<String> {
    let def = regex::Regex::new(&format!(r"macro_rules!\s+{}\b", regex::escape(name)));
    let (Ok(def), Ok(feature)) = (def, regex::Regex::new(r#"feature\s*=\s*"([^"]+)""#)) else {
        return Vec::new();
    };
    let mut stack = vec![project.join(root)];
    let mut seen = 0;
    while let Some(d) = stack.pop() {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for e in rd.filter_map(Result::ok) {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            seen += 1;
            if seen > 500 || p.extension().and_then(|x| x.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&p) else {
                continue;
            };
            let Some(at) = def.find(&text) else { continue };
            let mut out: Vec<String> = Vec::new();
            // The body of the macro: from its first `{` to the brace that closes it.
            let (mut depth, mut started) = (0i32, false);
            for line in text[at.end()..].lines().take(40) {
                let t = line.trim();
                if t.starts_with("#[cfg(") && !t.contains("not(") {
                    out.extend(feature.captures_iter(t).take(1).map(|c| c[1].to_owned()));
                }
                depth += i32::try_from(t.matches('{').count()).unwrap_or(0)
                    - i32::try_from(t.matches('}').count()).unwrap_or(0);
                started |= t.contains('{');
                if started && depth <= 0 {
                    break;
                }
            }
            out.dedup();
            return out;
        }
    }
    Vec::new()
}

/// The features of the `cfg` attributes right above `mod <name>;` in `text`, and of the
/// `cfg_x! { .. }` blocks around it (`macro_features` gives the features of a macro); `None`
/// when no such declaration is in it.
fn mod_gate(
    text: &str,
    name: &str,
    macro_features: &dyn Fn(&str) -> Vec<String>,
) -> Option<Vec<String>> {
    let decl = regex::Regex::new(&format!(
        r"^\s*(pub(\([^)]*\))?\s+)?mod\s+{}\s*;",
        regex::escape(name)
    ))
    .ok()?;
    let feature = regex::Regex::new(r#"feature\s*=\s*"([^"]+)""#).ok()?;
    let open = regex::Regex::new(r"^\s*(cfg_[a-z0-9_]+)!\s*\{").ok()?;
    let mut attrs: Vec<&str> = Vec::new();
    // `(brace depth after the opening line, macro name)` of the `cfg_x! {` blocks around.
    let mut blocks: Vec<(i32, String)> = Vec::new();
    let mut depth = 0i32;
    for line in text.lines() {
        let t = line.trim();
        let delta = i32::try_from(t.matches('{').count()).unwrap_or(0)
            - i32::try_from(t.matches('}').count()).unwrap_or(0);
        if let Some(c) = open.captures(line) {
            depth += delta;
            blocks.push((depth, c[1].to_owned()));
            attrs.clear();
            continue;
        }
        if t.starts_with("#[") {
            attrs.push(t);
            continue;
        }
        if t.starts_with("//") || t.is_empty() {
            continue;
        }
        if decl.is_match(line) {
            let mut out = Vec::new();
            for a in &attrs {
                if a.starts_with("#[cfg(") && !a.contains("not(") {
                    // `any(a, b)`: one is enough, the first is named; `all(..)`: every one.
                    let take = if a.contains("any(") { 1 } else { usize::MAX };
                    out.extend(feature.captures_iter(a).take(take).map(|c| c[1].to_owned()));
                }
            }
            for (_, m) in &blocks {
                out.extend(macro_features(m));
            }
            return Some(out);
        }
        attrs.clear();
        depth += delta;
        while blocks.last().is_some_and(|(d, _)| depth < *d) {
            blocks.pop();
        }
    }
    None
}

/// Sets the feature gate of each Rust target from the module of its site; `enabled` are the
/// features the harness build has.
pub fn add_feature_gates(project: &Path, targets: &mut [Target], enabled: &[String]) {
    for t in targets {
        let file = t.site.rsplit_once(':').map_or(t.site.as_str(), |(f, _)| f);
        if !file.ends_with(".rs") {
            continue;
        }
        let features = feature_gate(project, file);
        if !features.is_empty() {
            let on = features.iter().all(|f| enabled.contains(f));
            t.feature_gate = Some(FeatureGate {
                features,
                enabled: on,
            });
        }
    }
}

/// What `dla harness gen --entry` says about a target besides what every harness says.
#[derive(Debug, Clone, Default)]
pub struct Entry {
    /// The writer functions the callers chains start from (`Class.method`).
    pub writer_functions: Vec<String>,
    /// The lines of the callers chains and the key provenance, in the block.
    pub lines: Vec<String>,
}

/// The name of the function (`Class.method`) that encloses `file:line`, when the file parses.
fn enclosing_name(project: &Path, site: &str) -> Option<String> {
    let (file, line) = site.rsplit_once(':')?;
    let line: usize = line.trim().parse().ok()?;
    let lang = Lang::from_path(Path::new(file))?;
    let src = fs::read_to_string(project.join(file)).ok()?;
    let tree = parse(lang, &src).ok()?;
    let row = line.checked_sub(1)?;
    let text = src.lines().nth(row)?;
    let indent = text.len() - text.trim_start().len();
    // The range ends where the line does: a range that reaches past the end of the last
    // line of a function is not inside it.
    let mut node: Option<Node<'_>> = tree.root_node().descendant_for_point_range(
        Point {
            row,
            column: indent,
        },
        Point {
            row,
            column: text.trim_end().len().max(indent),
        },
    );
    let name_of = |n: Node<'_>| {
        n.child_by_field_name("name")
            .and_then(|c| c.utf8_text(src.as_bytes()).ok())
            .map(str::to_owned)
    };
    let mut func = None;
    while let Some(n) = node {
        if func.is_none() && FUNCTIONS.contains(&n.kind()) {
            func = name_of(n);
        } else if func.is_some() && CLASSES.contains(&n.kind()) {
            return name_of(n).map(|c| format!("{c}.{}", func.unwrap_or_default()));
        }
        node = n.parent();
    }
    func
}

fn sites(project: &Path, list: &[String]) -> (Vec<String>, usize) {
    let shown = list
        .iter()
        .take(SHOWN_SITES)
        .map(|s| match enclosing_name(project, s) {
            Some(f) => format!("{s} ({f})"),
            None => s.clone(),
        })
        .collect();
    (shown, list.len())
}

/// The message of the rule that raised the finding, from the static-hit evidence it cites
/// (the raw sweep file holds every hit).
fn rule_message(ctx: &ToolCtx, ledger: &Ledger, id: &FindingId) -> Option<String> {
    let f = ledger.finding(id)?;
    let site = f.site.as_deref()?;
    let (file, line) = site.rsplit_once(':')?;
    let line: u64 = line.parse().ok()?;
    for e in f.supports.iter().filter_map(|e| ledger.evidence_by_id(e)) {
        if e.kind != EvidenceKind::StaticHit {
            continue;
        }
        let Some(raw) = &e.raw else { continue };
        let Ok(text) = fs::read_to_string(ctx.run_dir.join(raw)) else {
            continue;
        };
        let Ok(doc) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let hit = doc.get("hits").and_then(Value::as_array).and_then(|hits| {
            hits.iter().find(|h| {
                h.get("file").and_then(Value::as_str) == Some(file)
                    && h.get("line").and_then(Value::as_u64) == Some(line)
            })
        });
        if let Some(m) = hit.and_then(|h| h.get("message")).and_then(Value::as_str) {
            return Some(m.trim().to_owned());
        }
    }
    None
}

/// What the probe of a target reads, from the harness's probes.
fn probe_text(project: &Path, a: &Asked, probes: &[ProbeSpec]) -> Option<String> {
    let file = a.site.rsplit_once(':').map_or(a.site.as_str(), |(f, _)| f);
    let module = python_module(project, file);
    let own = probes
        .iter()
        .find(|p| p.finding.as_ref() == Some(&a.id))
        .or_else(|| {
            // A module probe that stands for several candidates carries no finding.
            probes.iter().find(|p| match &p.target {
                ProbeTarget::PyModule { module: m } => *m == module,
                ProbeTarget::NodeModule { file: f } => f == file,
                ProbeTarget::JvmClass { class } => a.owner.starts_with(&format!("{class}.")),
                _ => false,
            })
        })?;
    Some(match &own.target {
        ProbeTarget::PyGlobal { module, name } => {
            format!("size of {module}.{name} (cache_info().currsize or len)")
        }
        ProbeTarget::PyAttr { module, qualname } => {
            format!("size of {module}.{qualname} (cache_info().currsize or len)")
        }
        ProbeTarget::PyModule { module } => {
            format!("size and reachable memory of the containers in module {module}")
        }
        ProbeTarget::NodeModule { file } => format!("the top-level bindings of {file}"),
        ProbeTarget::NodeBinding { file, name } => format!("{name} in {file}"),
        ProbeTarget::JvmStatic { class, field } => format!("static field {class}.{field}"),
        ProbeTarget::JvmClass { class } => format!("the static fields of {class}"),
        ProbeTarget::GoVar { package_dir, name } => {
            format!("package variable {package_dir}.{name}")
        }
        ProbeTarget::GoField {
            package_dir,
            type_name,
            field,
        } => format!(
            "length of field {field} of {type_name} (package {package_dir}), read by reflection from the objects you pass to dlaTrack: in dlaSetup write dlaTrack(<the {type_name} your setup built>)"
        ),
    })
}

/// The instruction line for a holder kind.
fn instruction(kind: &str, holder: &str, owner: &str, site: &str) -> String {
    if kind == "cache-decorator" {
        return format!(
            "call {} with arguments that differ for every unit (derive them from key)",
            if owner.is_empty() { site } else { owner }
        );
    }
    match holder {
        "static/global" | "container-field" => {
            "make every unit reach a writer so that it adds an entry whose key derives from the unit's key".to_owned()
        }
        "registration-table" => {
            "register through the public API once per unit, and release only what a normal caller of that API releases".to_owned()
        }
        _ => format!("make every unit run the code at {site}"),
    }
}

/// Added to the instruction of a registration on an object handed in from outside (a parameter,
/// a callback argument): it grows only when the object outlives the call.
pub const OUTER_RECEIVER: &str = "; the object registered on comes from outside the function, so the count grows only when the same object is used again: make the units share it the way a long-lived caller does (a reused connection, a pooled or shared object) and do not shorten its life (no per-unit close of it, no timeouts that defeat the reuse)";

/// What a target says when the census reads every writer of the holder as a start-up
/// registration (an initialiser, or called from one): calling the writer with new names grows
/// it and shows nothing about a caller.
const STARTUP_WRITERS: &str = " The census reads every writer of this holder as a start-up registration: reach the holder through a run-time entry (a request, a message, a connection), not by calling the registration function with new names.";

/// The general line printed once after the targets.
pub const GENERAL: &str = "use the project's public API the way a caller would; in fresh mode a unit must never reuse an earlier unit's key";

/// What a unit covers of the objects it touches, in every driver block: a leak that needs a
/// derived object, a cancel or a drop in another order is not reached by a unit that only
/// creates and uses.
pub const LIFE: &str = "a unit covers the life of the objects it touches: create, derive (also from derived objects), use, cancel or close, drop or release, and not always in the same order (vary it by key)";

/// Describes every declared target of a harness.
pub fn describe(
    ctx: &ToolCtx,
    ledger: &Ledger,
    asked: &[Asked],
    probes: &[ProbeSpec],
    siblings: &std::collections::BTreeMap<FindingId, FindingId>,
) -> Vec<Target> {
    asked
        .iter()
        .map(|a| {
            let f = ledger.finding(&a.id);
            let ann = ledger.annotation(&a.id);
            let holder = f
                .and_then(|f| f.holder)
                .map_or_else(|| a.kind.clone(), |h| h.to_string());
            let writers = ann.map_or_else(Vec::new, |a| a.writers.clone());
            let removers = ann.map_or_else(Vec::new, |a| a.removers.clone());
            let owner = f
                .and_then(|f| f.owner.clone())
                .filter(|o| !o.is_empty())
                .unwrap_or_else(|| a.owner.clone());
            Target {
                id: a.id.clone(),
                site: a.site.clone(),
                holder: holder.clone(),
                kind: a.kind.clone(),
                owner: owner.clone(),
                message: rule_message(ctx, ledger, &a.id),
                writers: sites(&ctx.project, &writers),
                removers: sites(&ctx.project, &removers),
                probe: probe_text(&ctx.project, a, probes),
                instruction: {
                    let mut text = instruction(&a.kind, &holder, &owner, &a.site);
                    if holder == "registration-table"
                        && ann.is_some_and(|a| {
                            a.facts.receiver == Some(dla_core::census::Receiver::Outer)
                        })
                    {
                        text.push_str(OUTER_RECEIVER);
                    }
                    if ann.is_some_and(|a| {
                        !a.facts.writer_sites.is_empty()
                            && a.facts.writer_sites.iter().all(|s| s.startup || s.init)
                    }) {
                        text.push_str(STARTUP_WRITERS);
                    }
                    text
                },
                sibling_of: siblings.get(&a.id).cloned(),
                entry: None,
                feature_gate: None,
            }
        })
        .collect()
}

/// Adds what an entry-path harness says about each target: the callers chain of each writer
/// function (up to three hops, matched by name) and where the key comes from. Runtimes with
/// no syntax-tree reader of calls get a line that says "not computed".
pub fn add_entry_facts(project: &Path, rt: dla_core::runtime::Runtime, targets: &mut [Target]) {
    use dla_census::callers::{self, Func, Index};
    let files = dla_census::walk::source_files(project, rt);
    let readable = files.iter().any(|f| callers::supported(f.lang));
    let ix = if readable {
        Index::build(&files)
    } else {
        Index::default()
    };
    for t in targets {
        if !readable {
            t.entry = Some(Entry {
                writer_functions: Vec::new(),
                lines: vec![
                    format!(
                        "  callers: not computed (no reader of calls for {} sources)",
                        rt.as_str()
                    ),
                    "  key from: not computed".to_owned(),
                ],
            });
            continue;
        }
        // The writer functions: the decorated function of a cache decorator, then the
        // function around each listed writer line.
        let mut fns: Vec<(Func, Option<usize>, bool)> = Vec::new();
        let mut add = |f: Option<Func>, line: Option<usize>, all: bool| {
            if let Some(f) = f.filter(|f| f.name != "<module>")
                && !fns
                    .iter()
                    .any(|(g, _, _)| g.file == f.file && g.name == f.name)
            {
                fns.push((f, line, all));
            }
        };
        if t.kind == "cache-decorator"
            && let Some((file, line)) = t.site.rsplit_once(':')
            && let Ok(line) = line.parse::<usize>()
        {
            add(ix.function_at(file, line), None, true);
        }
        for w in &t.writers.0 {
            let at = w.split(' ').next().unwrap_or_default();
            if let Some((file, line)) = at.rsplit_once(':')
                && let Ok(line) = line.parse::<usize>()
            {
                add(ix.function_at(file, line), Some(line), false);
            }
        }
        let mut lines = Vec::new();
        let mut names = Vec::new();
        if fns.is_empty() {
            lines.push("  callers: no writer function was found to start from".to_owned());
            lines.push("  key from: not found".to_owned());
        }
        for (f, line, all) in &fns {
            names.push(f.name.clone());
            lines.push(format!("  writer function: {}", f.label()));
            lines.push(
                "    callers (matched by name; [public] marks a function a user of the project can call; \"referenced\" is a place that passes or stores it without calling it, shown when nothing calls it):"
                    .to_owned(),
            );
            let chain = ix.chain(&files, f);
            if chain.is_empty() {
                lines.push("      none found".to_owned());
            }
            for (i, hop) in chain.iter().enumerate() {
                let list: Vec<String> = hop
                    .callers
                    .iter()
                    .map(|c| {
                        format!(
                            "{}{} ({}:{}{})",
                            c.func.name,
                            if c.func.public { " [public]" } else { "" },
                            c.func.file,
                            c.line,
                            if c.by_reference { ", referenced" } else { "" }
                        )
                    })
                    .collect();
                let more = if hop.more > 0 {
                    format!(" and {} more", hop.more)
                } else {
                    String::new()
                };
                lines.push(format!("      hop {}: {}{more}", i + 1, list.join(", ")));
            }
            let p = ix.provenance(&files, f, *line, *all);
            if p.params.is_empty() {
                lines.push(format!(
                    "    key from: no parameter of {} (it is built from state the holder's owner keeps, a constant or a counter)",
                    f.name
                ));
            } else {
                let ps: Vec<String> = p.params.iter().map(|x| format!("`{x}`")).collect();
                lines.push(format!(
                    "    key from: parameter {} of {}",
                    ps.join(", "),
                    f.name
                ));
                if p.sites.is_empty() {
                    lines.push(
                        "      no call site found (it may be called through a variable or a framework: read the places above that mention it)"
                            .to_owned(),
                    );
                }
                for site in &p.sites {
                    let what = if site.passed.is_empty() {
                        "passes none of them by name or position".to_owned()
                    } else {
                        site.passed
                            .iter()
                            .map(|(k, v)| format!("{k} = {v}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    lines.push(format!("      {} ({}): {what}", site.at, site.caller));
                    if !site.guards.is_empty() {
                        lines.push(format!(
                            "        the call happens only when: {}",
                            site.guards.join("; ")
                        ));
                    }
                    for (name, rhs) in &site.origins {
                        lines.push(format!("        {name} is set by: {}", rhs.join("; ")));
                    }
                }
                if p.more_sites > 0 {
                    lines.push(format!("      and {} more call sites", p.more_sites));
                }
            }
        }
        t.entry = Some(Entry {
            writer_functions: names,
            lines,
        });
    }
}

fn site_list(label: &str, s: &(Vec<String>, usize)) -> String {
    if s.0.is_empty() {
        return format!("{label}: none found");
    }
    let more = s.1.saturating_sub(s.0.len());
    let tail = if more > 0 {
        format!(" and {more} more")
    } else {
        String::new()
    };
    format!("{label}: {}{tail}", s.0.join(", "))
}

impl Target {
    /// The lines that describe the target, in the block and the text output.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!(
            "{} at {}: holder {}{}",
            self.id,
            self.site,
            if self.holder.is_empty() {
                "unknown"
            } else {
                &self.holder
            },
            if self.owner.is_empty() {
                String::new()
            } else {
                format!(", owner {}", self.owner)
            }
        )];
        if let Some(s) = &self.sibling_of {
            out[0].push_str(&format!(" (sibling_of {s}: state of the same owner)"));
        }
        if let Some(m) = &self.message {
            out.push(format!("  rule: {m}"));
        }
        out.push(format!("  {}", site_list("writers", &self.writers)));
        out.push(format!("  {}", site_list("removers", &self.removers)));
        if let Some(e) = &self.entry {
            out.extend(e.lines.iter().cloned());
        }
        if let Some(g) = &self.feature_gate {
            out.push(format!("  {}", g.text()));
        }
        out.push(match &self.probe {
            Some(p) => format!("  probe reads: {p}"),
            None => "  no probe: tied by allocation stacks or type counts".to_owned(),
        });
        out.push(format!(
            "  to do: {}",
            if self.entry.is_some() {
                ENTRY_INSTRUCTION
            } else {
                self.instruction.as_str()
            }
        ));
        out
    }

    /// The target as the `summary.targets` JSON.
    pub fn json(&self) -> Value {
        json!({
            "finding": self.id,
            "site": self.site,
            "holder": self.holder,
            "owner": self.owner,
            "rule": self.message,
            "writers": self.writers.0,
            "removers": self.removers.0,
            "probe": self.probe,
            "instruction": self.instruction,
            "sibling_of": self.sibling_of,
            "writer_functions": self.entry.as_ref().map(|e| &e.writer_functions),
            "feature_gate": self.feature_gate.as_ref().map(FeatureGate::text),
        })
    }
}

/// What each target of an entry-path harness is told to do.
const ENTRY_INSTRUCTION: &str =
    "reach it through the project's public entry points, with data a caller controls";

/// The rules of an entry-path unit, as printed in the block.
pub const ENTRY_RULES: [&str; 6] = [
    "Rules of an entry-path unit:",
    "- call only the project's public entry points (the functions, endpoints, commands or messages a user of the project calls), with the objects under test built once in setup",
    "- per unit vary only data that a caller or a remote peer controls (ids, names, paths, methods, headers, bodies, file names, message contents), including inputs the project rejects (unknown paths, unsupported methods, malformed input): error paths often build objects per call",
    "- never create new functions, classes, handlers, loggers or configuration per unit unless a real program does so per request, and if you do, say so in the varies line",
    "- a unit covers the life of the objects it touches: create, derive (also from derived objects), use, cancel or close, drop or release, and not always in the same order (vary it by key)",
    "- write one line in VARIES.txt next to the driver: what a caller varies per unit; dla harness run refuses the harness until it is written",
];

/// The line of a broad batch's block about targets whose code does not run, and the helper
/// that takes order and depth from the key.
pub fn broad_note(helper: &str) -> String {
    format!(
        "A target whose code does not run in this harness comes back in a small harness of its own; reach as many as one unit naturally can. The driver has {helper}(key, slot, n), choice number slot of n options taken from the key: let order and depth come from the key rather than from one fixed sequence."
    )
}

/// The lines of a focus harness's block, above its targets: what the harness is, what to do,
/// the way out for a target no driver can reach, and per target where it was missed and why.
pub fn focus_notes(
    missed: &std::collections::BTreeMap<FindingId, dla_core::queue::Miss>,
    asked: &[Asked],
) -> Vec<String> {
    let mut out = vec![
        "Focus harness: these targets were declared in an earlier harness and their code did not run there. This harness exists for them alone. Call the writer function of each (named below) with a new key per unit. A target that no driver can reach here (it needs a container, a peer, another platform) is not left silently: after the run, dla ledger resolve <F> --as unresolved --reason env_unavailable --remedy \"<what it needs>\".".to_owned(),
    ];
    for a in asked {
        if let Some(m) = missed.get(&a.id) {
            out.push(format!(
                "  {} was declared in {} and read {}: {}",
                a.id,
                m.harness,
                m.verdict.as_str().replace('_', " "),
                m.detail
            ));
        }
    }
    out
}

/// What a second look is told to vary, general to any project.
const SECOND_LOOK_TODOS: [&str; 5] = [
    "  - other call orders than the first driver's (do the steps in a different order, or in a different order per key)",
    "  - objects derived from derived objects, three deep where the API allows it (a child of a child, a clone of a clone)",
    "  - the failing and cancelling paths: cancel or close the parent before the child, drop an object before its work completes, fail half-way through",
    "  - overlapping lifetimes: start the next before the last one ended",
    "  - one long-lived object reused across units next to a fresh one per unit",
];

/// The recipe of the choices of a second look, general to any project.
const SECOND_LOOK_RECIPE: [&str; 5] = [
    "  - slot 0, depth: how deep to derive when objects of the type are made from objects of the same type (1 to 3; a child of a child needs 3)",
    "  - slot 1, ending: which object of the chain gets the ending call (cancel, close, abort, shutdown, remove, unsubscribe): none, the first, a middle one, the last",
    "  - slot 2, release of the intermediates: at once (a temporary), before the ending call, after it",
    "  - slot 3, the order of the remaining releases (forward or reversed)",
    "  - slot 4, success or failure where the API has one (complete or fail half-way)",
];

/// The lines of a second-look harness's block, above its targets.
pub fn second_look_notes(
    missed: &std::collections::BTreeMap<FindingId, dla_core::queue::Miss>,
    asked: &[Asked],
    helper: &str,
) -> Vec<String> {
    let mut out = vec![
        "Second look: one flat run of one driver settled these targets (their code ran and their holder stayed flat), and budget is left. A flat reading of one arbitrary unit says that unit does not leak; a leak that needs a sequence stays hidden. This harness needs a DIFFERENT driver: dla harness run refuses hooks equal to an earlier harness's, and a VARIES.txt line per target (what this driver does that the first did not).".to_owned(),
        "To vary, at least some of:".to_owned(),
    ];
    out.extend(SECOND_LOOK_TODOS.iter().map(|l| (*l).to_owned()));
    out.push(format!(
        "Write the unit as CHOICES, not as one fixed sequence: a fixed sequence runs one case for every unit, and a leak that needs another order never shows. The driver has {helper}(key, slot, n): choice number slot of n options, from the digits of the key in mixed radix (every combination of the slots occurs within n0*n1*... units). Take at least two choices with it ({helper} is checked by dla harness run), for the owner type of each target:"
    ));
    out.extend(SECOND_LOOK_RECIPE.iter().map(|l| (*l).to_owned()));
    for a in asked {
        if let Some(m) = missed.get(&a.id) {
            out.push(format!(
                "  {} was read flat in {}: read the unit of that harness (harness/{}/driver) and do something else",
                a.id, m.harness, m.harness
            ));
        }
    }
    out
}

/// The comment block for a driver, above the first hook. `prefix` is the line comment
/// marker (`#` or `//`); `None` wraps each line in `/* */`.
pub fn block(harness: &str, targets: &[Target], prefix: Option<&str>) -> String {
    block_with(harness, targets, prefix, &[])
}

/// [`block`] with `notes`: lines about the driver itself (the Go package it is in), placed
/// right under the first line.
pub fn block_with(
    harness: &str,
    targets: &[Target],
    prefix: Option<&str>,
    notes: &[String],
) -> String {
    let entry = targets.iter().any(|t| t.entry.is_some());
    let mut lines = vec![if entry {
        format!(
            "Entry-path harness {harness}: show whether a caller of the project's public API can make new keys reach these holders (they grew when a driver fed them keys directly)"
        )
    } else {
        format!(
            "Targets of harness {harness}: make every unit reach these, with a new key per unit"
        )
    }];
    lines.extend(notes.iter().cloned());
    for t in targets {
        lines.extend(t.lines());
    }
    if entry {
        lines.extend(ENTRY_RULES.iter().map(|l| (*l).to_owned()));
    } else {
        lines.push(format!("In general: {GENERAL}"));
        lines.push(format!("Also: {LIFE}"));
    }
    lines
        .iter()
        .map(|l| match prefix {
            Some(p) => format!("{p} {l}").trim_end().to_owned(),
            None => format!("/* {} */", l.replace("*/", "* /")),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Writes the block above the first hook of the driver file `path` (the first line that
/// reads `DLA-HOOK <name>:` in a comment), with that line's indentation. Returns false when
/// the file has no such line.
pub fn write_block(path: &Path, harness: &str, targets: &[Target]) -> std::io::Result<bool> {
    write_block_with(path, harness, targets, &[])
}

/// [`write_block`] with the driver notes of [`block_with`].
pub fn write_block_with(
    path: &Path,
    harness: &str,
    targets: &[Target],
    notes: &[String],
) -> std::io::Result<bool> {
    let text = fs::read_to_string(path)?;
    let prefix = match path.extension().and_then(|e| e.to_str()) {
        Some("py") => Some("#"),
        Some("c") => None,
        _ => Some("//"),
    };
    let re = regex::Regex::new(r"DLA-HOOK [a-z]+:").expect("static regex");
    let mut at = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let t = line.trim_start();
        if re.is_match(line) && (t.starts_with('#') || t.starts_with("//") || t.starts_with("/*")) {
            at = Some((offset, line.len() - t.len()));
            break;
        }
        offset += line.len();
    }
    let Some((offset, indent)) = at else {
        return Ok(false);
    };
    let pad = &text[offset..offset + indent];
    let body: String = block_with(harness, targets, prefix, notes)
        .lines()
        .map(|l| format!("{pad}{l}\n"))
        .collect();
    let mut out = String::with_capacity(text.len() + body.len());
    out.push_str(&text[..offset]);
    out.push_str(&body);
    out.push_str(&text[offset..]);
    fs::write(path, out)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> Target {
        Target {
            id: "F8".parse().unwrap(),
            site: "pkg/web_app.py:82".into(),
            holder: "static/global".into(),
            kind: "cache-decorator".into(),
            owner: "pkg.web_app._build".into(),
            message: Some("unbounded cache".into()),
            writers: (vec!["pkg/web_app.py:82 (_build)".into()], 1),
            removers: (vec![], 0),
            probe: Some("size of pkg.web_app._build (cache_info().currsize or len)".into()),
            instruction: instruction(
                "cache-decorator",
                "static/global",
                "pkg.web_app._build",
                "pkg/web_app.py:82",
            ),
            sibling_of: None,
            entry: None,
            feature_gate: None,
        }
    }

    #[test]
    fn the_block_goes_above_the_first_hook_with_its_indentation() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("dla_harness.py");
        fs::write(
            &p,
            "\"\"\"Fill the two DLA-HOOK bodies below.\"\"\"\n\ndef setup():\n    # DLA-HOOK setup: do it\n    raise X\n",
        )
        .unwrap();
        assert!(write_block(&p, "H2", &[target()]).unwrap());
        let text = fs::read_to_string(&p).unwrap();
        let hook = text.find("    # DLA-HOOK setup:").unwrap();
        let block = text.find("    # Targets of harness H2").unwrap();
        assert!(block < hook, "{text}");
        assert!(text.contains(
            "    # F8 at pkg/web_app.py:82: holder static/global, owner pkg.web_app._build\n"
        ));
        assert!(text.contains("call pkg.web_app._build with arguments that differ for every unit (derive them from key)"));
        assert!(text.contains("never reuse an earlier unit's key"));
        assert!(text.contains(
            "    # Also: a unit covers the life of the objects it touches: create, derive (also from derived objects), use, cancel or close, drop or release, and not always in the same order (vary it by key)\n"
        ));
        assert!(text.starts_with("\"\"\"Fill"), "{text}");
    }

    #[test]
    fn a_c_driver_gets_comment_pairs() {
        let b = block("H1", &[target()], None);
        assert!(
            b.lines()
                .all(|l| l.starts_with("/* ") && l.ends_with(" */")),
            "{b}"
        );
    }

    #[test]
    fn instructions_by_holder_kind() {
        assert!(
            instruction("cache-decorator", "static/global", "m.f", "a.py:1")
                .starts_with("call m.f with arguments")
        );
        assert!(
            instruction("", "container-field", "", "a.py:1")
                .starts_with("make every unit reach a writer")
        );
        assert!(
            instruction("", "static/global", "", "a.py:1")
                .starts_with("make every unit reach a writer")
        );
        assert!(
            instruction("", "registration-table", "", "a.py:1")
                .starts_with("register through the public API once per unit")
        );
        assert_eq!(
            instruction("", "thread-local", "", "a.py:9"),
            "make every unit run the code at a.py:9"
        );
    }

    #[test]
    fn the_enclosing_function_is_named_with_its_class() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("a.py"),
            "CACHE = {}\n\nclass C:\n    def put(self, k):\n        CACHE[k] = 1\n\ndef free(k):\n    CACHE[k] = 2\n",
        )
        .unwrap();
        assert_eq!(
            enclosing_name(dir.path(), "a.py:5").as_deref(),
            Some("C.put")
        );
        assert_eq!(
            enclosing_name(dir.path(), "a.py:8").as_deref(),
            Some("free")
        );
        assert_eq!(enclosing_name(dir.path(), "a.py:1"), None);
    }
    fn writer_target() -> Target {
        Target {
            id: "F3".parse().unwrap(),
            site: "a.py:1".into(),
            holder: "static/global".into(),
            kind: "container".into(),
            owner: "a.CACHE".into(),
            message: None,
            writers: (vec!["a.py:5 (C.put)".into()], 1),
            removers: (Vec::new(), 0),
            probe: None,
            instruction: "make every unit reach a writer".into(),
            sibling_of: None,
            entry: None,
            feature_gate: None,
        }
    }

    #[test]
    fn an_entry_path_block_has_the_callers_the_key_and_the_rules_and_no_general_line() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("a.py"),
            "CACHE = {}\n\nclass C:\n    def put(self, k, v):\n        CACHE[k] = v\n\ndef serve(req):\n    return C().put(req.path, 1)\n",
        )
        .unwrap();
        let mut t = vec![writer_target()];
        add_entry_facts(dir.path(), dla_core::runtime::Runtime::Python, &mut t);
        assert_eq!(t[0].entry.as_ref().unwrap().writer_functions, ["C.put"]);
        let b = block("H5", &t, Some("#"));
        for want in [
            "# Entry-path harness H5:",
            "#   writer function: a.py:4 (C.put)",
            "#       hop 1: serve [public] (a.py:8)",
            "(matched by name",
            "#     key from: parameter `k`, `v` of C.put",
            "#       a.py:8 (serve): k = req.path, v = 1",
            "# Rules of an entry-path unit:",
            "#   to do: reach it through the project's public entry points, with data a caller controls",
        ] {
            assert!(b.contains(want), "{want}\n{b}");
        }
        assert!(!b.contains("In general:"), "{b}");
        assert!(
            b.contains("# - a unit covers the life of the objects it touches"),
            "{b}"
        );
        // A harness that is not entry-path keeps its block.
        let plain = block("H5", &[writer_target()], Some("#"));
        assert!(plain.contains("In general:") && !plain.contains("Rules of an entry-path unit"));
    }

    #[test]
    fn a_runtime_without_a_reader_of_calls_says_not_computed() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("lib.rs"), "fn put() {}\n").unwrap();
        let mut t = vec![writer_target()];
        add_entry_facts(dir.path(), dla_core::runtime::Runtime::Rust, &mut t);
        let lines = t[0].entry.as_ref().unwrap().lines.join("\n");
        assert!(lines.contains("callers: not computed"), "{lines}");
        assert!(lines.contains("key from: not computed"), "{lines}");
    }

    fn crate_tree(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (rel, text) in files {
            let p = dir.path().join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        dir
    }

    #[test]
    fn a_module_declared_behind_a_feature_names_it_down_the_tree() {
        let dir = crate_tree(&[
            (
                "src/lib.rs",
                "pub mod plain;

#[cfg(feature = \"rt\")]\n#[doc(hidden)]
pub mod task;

#[cfg(not(feature = \"rt\"))]
mod fallback;
#[cfg(all(feature = \"io\", unix))]
mod sys;
",
            ),
            (
                "src/plain.rs",
                "pub static A: u8 = 0;
",
            ),
            (
                "src/task/mod.rs",
                "#[cfg(feature = \"time\")]
mod pinned;
mod open;
",
            ),
            (
                "src/task/pinned.rs",
                "pub static B: u8 = 0;
",
            ),
            (
                "src/task/open.rs",
                "pub static C: u8 = 0;
",
            ),
            (
                "src/fallback.rs",
                "pub static D: u8 = 0;
",
            ),
            (
                "src/sys.rs",
                "pub static E: u8 = 0;
",
            ),
            (
                "src/bin/tool.rs",
                "fn main() {}
",
            ),
        ]);
        let gate = |f: &str| feature_gate(dir.path(), f);
        assert!(gate("src/plain.rs").is_empty());
        assert_eq!(gate("src/task/mod.rs"), ["rt"]);
        // Two declarations down: the features of both, sorted.
        assert_eq!(gate("src/task/pinned.rs"), ["rt", "time"]);
        assert_eq!(gate("src/task/open.rs"), ["rt"]);
        // A `not(..)` gate is not read as a requirement; `all(..)` names its feature.
        assert!(gate("src/fallback.rs").is_empty());
        assert_eq!(gate("src/sys.rs"), ["io"]);
        // A file nothing declares (a binary) has no gate.
        assert!(gate("src/bin/tool.rs").is_empty());
        assert!(gate("src/lib.rs").is_empty());
    }

    #[test]
    fn a_module_inside_a_cfg_macro_block_is_gated_by_the_macro_features() {
        let dir = crate_tree(&[
            (
                "src/lib.rs",
                "mod cfg;\n\ncfg_rt! {\n    pub mod context;\n    pub mod task;\n}\n\ncfg_time! {\n    pub mod time;\n}\n\npub mod sync;\n",
            ),
            (
                "src/cfg.rs",
                "macro_rules! cfg_rt {\n    ($($item:item)*) => {\n        $(\n            #[cfg(feature = \"rt\")]\n            #[cfg_attr(docsrs, doc(cfg(feature = \"rt\")))]\n            $item\n        )*\n    }\n}\n\nmacro_rules! cfg_time {\n    ($($item:item)*) => {\n        $(\n            #[cfg(feature = \"time\")]\n            $item\n        )*\n    }\n}\n",
            ),
            ("src/task/spawn_pinned.rs", "pub static A: u8 = 0;\n"),
            ("src/task/mod.rs", "mod spawn_pinned;\n"),
            ("src/sync.rs", "pub static B: u8 = 0;\n"),
            ("src/time.rs", "pub static C: u8 = 0;\n"),
        ]);
        assert_eq!(feature_gate(dir.path(), "src/task/spawn_pinned.rs"), ["rt"]);
        assert_eq!(feature_gate(dir.path(), "src/time.rs"), ["time"]);
        assert!(feature_gate(dir.path(), "src/sync.rs").is_empty());
    }

    #[test]
    fn a_module_file_beside_its_parent_is_found_by_the_parent_name() {
        let dir = crate_tree(&[
            (
                "src/lib.rs",
                "#[cfg(feature = \"net\")]\nmod net;
",
            ),
            (
                "src/net.rs",
                "mod tcp;
",
            ),
            (
                "src/net/tcp.rs",
                "pub static T: u8 = 0;
",
            ),
        ]);
        assert_eq!(feature_gate(dir.path(), "src/net/tcp.rs"), ["net"]);
    }

    #[test]
    fn the_gate_line_says_what_to_generate_with_or_what_the_build_has() {
        let dir = crate_tree(&[
            (
                "src/lib.rs",
                "#[cfg(feature = \"rt\")]\nmod task;
",
            ),
            (
                "src/task.rs",
                "pub static A: u8 = 0;
",
            ),
        ]);
        let mut t = target();
        t.site = "src/task.rs:1".into();
        let mut list = [t];
        add_feature_gates(dir.path(), &mut list, &[]);
        let b = block("H1", &list, Some("//"));
        assert!(
            b.contains("//   behind feature rt: generate with --features rt (this harness is built without it)"),
            "{b}"
        );
        add_feature_gates(dir.path(), &mut list, &["rt".to_owned()]);
        let b = block("H1", &list, Some("//"));
        assert!(
            b.contains("//   behind feature rt: the driver is built with --features rt"),
            "{b}"
        );
        // A site that is not Rust is left alone.
        let mut py = target();
        py.site = "pkg/a.py:3".into();
        let mut list = [py];
        add_feature_gates(dir.path(), &mut list, &[]);
        assert!(list[0].feature_gate.is_none());
    }
}
