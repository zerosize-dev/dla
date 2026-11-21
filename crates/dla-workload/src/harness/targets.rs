//! The targets of a harness: what `dla harness gen` prints for each finding a harness was
//! generated to reach, and the comment block it writes at the top of the driver.

use std::{fs, path::Path};

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

/// The general line printed once after the targets.
pub const GENERAL: &str = "use the project's public API the way a caller would; in fresh mode a unit must never reuse an earlier unit's key";

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
                instruction: instruction(&a.kind, &holder, &owner, &a.site),
                sibling_of: siblings.get(&a.id).cloned(),
                entry: None,
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
                && !fns.iter().any(|(g, _, _)| g.file == f.file && g.name == f.name)
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
                "    callers (matched by name; [public] marks a function a user of the project can call):"
                    .to_owned(),
            );
            let chain = ix.chain(f);
            if chain.is_empty() {
                lines.push("      none found".to_owned());
            }
            for (i, hop) in chain.iter().enumerate() {
                let list: Vec<String> = hop
                    .callers
                    .iter()
                    .map(|c| {
                        format!(
                            "{}{} ({}:{})",
                            c.func.name,
                            if c.func.public { " [public]" } else { "" },
                            c.func.file,
                            c.line
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
                    lines.push("      no call site found".to_owned());
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
        })
    }
}

/// What each target of an entry-path harness is told to do.
const ENTRY_INSTRUCTION: &str =
    "reach it through the project's public entry points, with data a caller controls";

/// The rules of an entry-path unit, as printed in the block.
pub const ENTRY_RULES: [&str; 5] = [
    "Rules of an entry-path unit:",
    "- call only the project's public entry points (the functions, endpoints, commands or messages a user of the project calls), with the objects under test built once in setup",
    "- per unit vary only data that a caller or a remote peer controls (ids, names, paths, methods, headers, bodies, file names, message contents), including inputs the project rejects (unknown paths, unsupported methods, malformed input): error paths often build objects per call",
    "- never create new functions, classes, handlers, loggers or configuration per unit unless a real program does so per request, and if you do, say so in the varies line",
    "- write one line in VARIES.txt next to the driver: what a caller varies per unit; dla harness run refuses the harness until it is written",
];

/// The comment block for a driver, above the first hook. `prefix` is the line comment
/// marker (`#` or `//`); `None` wraps each line in `/* */`.
pub fn block(harness: &str, targets: &[Target], prefix: Option<&str>) -> String {
    let entry = targets.iter().any(|t| t.entry.is_some());
    let mut lines = vec![if entry {
        format!(
            "Entry-path harness {harness}: show whether a caller of the project's public API can make new keys reach these holders (they grew when a driver fed them keys directly)"
        )
    } else {
        format!("Targets of harness {harness}: make every unit reach these, with a new key per unit")
    }];
    for t in targets {
        lines.extend(t.lines());
    }
    if entry {
        lines.extend(ENTRY_RULES.iter().map(|l| (*l).to_owned()));
    } else {
        lines.push(format!("In general: {GENERAL}"));
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
    let body: String = block(harness, targets, prefix)
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
}
