//! Probe derivation from census candidates and top-level name scans (spec 4.6).
//!
//! A probe is a holder the agent reads at every point: a JVM static field, a Python module
//! global, a Node module binding, a Go package variable. [`derive`] builds the list for a
//! harness from the census candidates of the runtime (and `--probe F` findings);
//! [`resolve_site`] maps a probe result path an agent reported back to a `file:line` and the
//! finding it belongs to.
//!
//! # Derivation
//!
//! | Runtime | Census candidate | Probe |
//! |---|---|---|
//! | JVM | `static*` owner `pkg.Class.FIELD` | `JvmStatic`; plus `JvmClass` for every class holding a candidate |
//! | Python | `module-global`, `module-pointer` owner `a.b.NAME` | `PyGlobal`; `class-attr`, `cache-decorator` -> `PyAttr`; `PyModule` for every module with a candidate |
//! | Node | any holder in a file | `NodeModule` for the file, with the file's top-level names |
//! | Go | package variable | `GoVar`, driver mode only |
//! | Rust, native | | none (the allocator shim groups by call site) |
//!
//! # Site resolution
//!
//! Exact census owner first, then an owner that ends with the path, then (Node) the census
//! entry of the file whose last segment is the property, then the top-level declaration
//! line of the name found with tree-sitter.

use std::{fs, path::Path};

use dla_census::{
    ast::{child_of, children_of, line as node_line, text, visit},
    parse::{Lang, parse, tree_sitter::Node},
};
use dla_core::{
    FindingId, Ledger, Refusal,
    census::CensusEntry,
    error::codes,
    harness::{HarnessManifest, ProbeSpec, ProbeTarget},
    runtime::Runtime,
    toolctx::ToolCtx,
};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------------------
// Declarations (tree-sitter)
// ---------------------------------------------------------------------------------------

/// A declaration found by [`declarations`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    /// Declared name.
    pub name: String,
    /// Enclosing class, for members.
    pub scope: Option<String>,
    /// 1-based line of the name.
    pub line: u32,
    /// `binding`, `class`, `function` or `field`.
    pub kind: &'static str,
}

fn identifiers_in<'t>(n: Node<'t>, src: &str, out: &mut Vec<(String, u32)>) {
    visit(n, &mut |c| {
        if matches!(
            c.kind(),
            "identifier" | "shorthand_property_identifier_pattern"
        ) {
            out.push((text(c, src).to_owned(), node_line(c)));
        }
    });
}

fn js_declarations(root: Node<'_>, src: &str, out: &mut Vec<Decl>) {
    let mut c = root.walk();
    for top in root.named_children(&mut c) {
        let node = if top.kind() == "export_statement" {
            top.child_by_field_name("declaration").unwrap_or(top)
        } else {
            top
        };
        match node.kind() {
            "lexical_declaration" | "variable_declaration" => {
                for d in children_of(node, "variable_declarator") {
                    let Some(name) = d.child_by_field_name("name") else {
                        continue;
                    };
                    let mut names = Vec::new();
                    if name.kind() == "identifier" {
                        names.push((text(name, src).to_owned(), node_line(name)));
                    } else {
                        identifiers_in(name, src, &mut names);
                    }
                    for (n, line) in names {
                        out.push(Decl {
                            name: n,
                            scope: None,
                            line,
                            kind: "binding",
                        });
                    }
                }
            }
            "class_declaration" | "function_declaration" | "generator_function_declaration" => {
                if let Some(n) = node.child_by_field_name("name") {
                    out.push(Decl {
                        name: text(n, src).to_owned(),
                        scope: None,
                        line: node_line(n),
                        kind: if node.kind() == "class_declaration" {
                            "class"
                        } else {
                            "function"
                        },
                    });
                }
            }
            _ => {}
        }
    }
}

fn py_body_decls(body: Node<'_>, src: &str, scope: Option<&str>, out: &mut Vec<Decl>) {
    let mut c = body.walk();
    for top in body.named_children(&mut c) {
        let node = if top.kind() == "decorated_definition" {
            top.child_by_field_name("definition").unwrap_or(top)
        } else {
            top
        };
        match node.kind() {
            "expression_statement" => {
                if let Some(a) = child_of(node, "assignment")
                    && let Some(l) = a.child_by_field_name("left")
                    && l.kind() == "identifier"
                {
                    out.push(Decl {
                        name: text(l, src).to_owned(),
                        scope: scope.map(str::to_owned),
                        line: node_line(l),
                        kind: if scope.is_some() { "field" } else { "binding" },
                    });
                }
            }
            "class_definition" | "function_definition" => {
                let Some(n) = node.child_by_field_name("name") else {
                    continue;
                };
                let name = text(n, src).to_owned();
                out.push(Decl {
                    name: name.clone(),
                    scope: scope.map(str::to_owned),
                    line: node_line(n),
                    kind: if node.kind() == "class_definition" {
                        "class"
                    } else {
                        "function"
                    },
                });
                if node.kind() == "class_definition"
                    && scope.is_none()
                    && let Some(b) = node.child_by_field_name("body")
                {
                    py_body_decls(b, src, Some(&name), out);
                }
            }
            _ => {}
        }
    }
}

fn java_declarations(root: Node<'_>, src: &str, out: &mut Vec<Decl>) {
    visit(root, &mut |n| {
        if !matches!(
            n.kind(),
            "class_declaration"
                | "interface_declaration"
                | "enum_declaration"
                | "record_declaration"
        ) {
            return;
        }
        let Some(name) = n.child_by_field_name("name") else {
            return;
        };
        let class = text(name, src).to_owned();
        out.push(Decl {
            name: class.clone(),
            scope: None,
            line: node_line(name),
            kind: "class",
        });
        let Some(body) = n.child_by_field_name("body") else {
            return;
        };
        for f in children_of(body, "field_declaration") {
            let mut c = f.walk();
            for d in f
                .named_children(&mut c)
                .filter(|d| d.kind() == "variable_declarator")
            {
                if let Some(nm) = d.child_by_field_name("name") {
                    out.push(Decl {
                        name: text(nm, src).to_owned(),
                        scope: Some(class.clone()),
                        line: node_line(nm),
                        kind: "field",
                    });
                }
            }
        }
    });
}

fn go_declarations(root: Node<'_>, src: &str, out: &mut Vec<Decl>) {
    let mut c = root.walk();
    for top in root.named_children(&mut c) {
        if top.kind() != "var_declaration" {
            continue;
        }
        visit(top, &mut |n| {
            if n.kind() != "var_spec" {
                return;
            }
            let mut cc = n.walk();
            for id in n.named_children(&mut cc) {
                if id.kind() != "identifier" {
                    break;
                }
                out.push(Decl {
                    name: text(id, src).to_owned(),
                    scope: None,
                    line: node_line(id),
                    kind: "binding",
                });
            }
        });
    }
}

/// Declarations of `src`: module-level names (JavaScript, TypeScript, Python, Go), classes
/// with their members (Python class bodies, Java fields).
pub fn declarations(lang: Lang, src: &str) -> Vec<Decl> {
    let Ok(tree) = parse(lang, src) else {
        return Vec::new();
    };
    let root = tree.root_node();
    let mut out = Vec::new();
    match lang {
        Lang::JavaScript | Lang::TypeScript | Lang::Tsx => js_declarations(root, src, &mut out),
        Lang::Python => py_body_decls(root, src, None, &mut out),
        Lang::Java => java_declarations(root, src, &mut out),
        Lang::Go => go_declarations(root, src, &mut out),
        _ => {}
    }
    out
}

/// Line of the declaration of `name` (a member of `scope` when given, else top-level).
pub fn declaration_line(lang: Lang, src: &str, name: &str, scope: Option<&str>) -> Option<u32> {
    declarations(lang, src)
        .into_iter()
        .find(|d| d.name == name && d.scope.as_deref() == scope)
        .map(|d| d.line)
}

/// Names a Node agent reads from a module: top-level bindings and classes (functions are
/// skipped), in source order, without duplicates.
pub fn node_top_level_names(lang: Lang, src: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for d in declarations(lang, src) {
        if d.kind != "function" && !names.contains(&d.name) {
            names.push(d.name);
        }
    }
    names
}

// ---------------------------------------------------------------------------------------
// Candidates
// ---------------------------------------------------------------------------------------

/// A census candidate as the probe code sees it.
#[derive(Debug, Clone)]
pub struct Cand {
    /// The finding the census raised for it.
    pub finding: Option<FindingId>,
    /// Qualified owner (`shop.OrderCache.ORDERS_BY_ID`).
    pub owner: String,
    /// Census kind (`static-container`, `module-global`, ...).
    pub kind: String,
    /// `file:line`.
    pub site: String,
}

/// The runtime a source file belongs to.
pub fn file_runtime(rel: &str) -> Option<Runtime> {
    Lang::from_path(Path::new(rel)).map(Lang::runtime)
}

fn site_file(site: &str) -> &str {
    site.rsplit_once(':').map_or(site, |(f, _)| f)
}

/// The census entries of runtime `rt` (`census/<rt>.json`), empty when the census did not run.
pub fn census_entries(ctx: &ToolCtx, rt: Runtime) -> Vec<CensusEntry> {
    let path = ctx
        .run_dir
        .join("census")
        .join(format!("{}.json", rt.as_str()));
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(doc) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    doc.get("entries")
        .and_then(|e| serde_json::from_value(e.clone()).ok())
        .unwrap_or_default()
}

fn finding_for_site(ledger: &Ledger, site: &str) -> Option<FindingId> {
    ledger
        .findings()
        .find(|f| f.origin.as_str() == "census" && f.site.as_deref() == Some(site))
        .map(|f| f.id.clone())
}

/// Census candidates of `rt` joined with their findings.
fn candidates(ctx: &ToolCtx, ledger: &Ledger, rt: Runtime) -> Vec<Cand> {
    census_entries(ctx, rt)
        .into_iter()
        .map(|e| Cand {
            finding: finding_for_site(ledger, &e.site),
            owner: e.owner,
            kind: e.kind,
            site: e.site,
        })
        .collect()
}

/// Dotted module name of a Python source file (`ingest/registry.py` -> `ingest.registry`;
/// a `src/` layout without `src/__init__.py` drops `src`).
pub fn python_module(project: &Path, rel: &str) -> String {
    let mut parts: Vec<&str> = rel.trim_end_matches(".py").split('/').collect();
    if parts.last() == Some(&"__init__") {
        parts.pop();
    }
    if parts.first() == Some(&"src")
        && parts.len() > 1
        && !project.join("src/__init__.py").is_file()
    {
        parts.remove(0);
    }
    parts.join(".")
}

fn jvm_class_of(owner: &str) -> Option<(&str, &str)> {
    owner.rsplit_once('.')
}

/// Targets for one candidate, specific ones first.
fn targets_for(
    project: &Path,
    rt: Runtime,
    c: &Cand,
    driver: bool,
    go_dir: Option<&str>,
) -> (Vec<ProbeTarget>, Vec<ProbeTarget>) {
    let file = site_file(&c.site);
    let mut specific = Vec::new();
    let mut module = Vec::new();
    match rt {
        Runtime::Jvm => {
            if let Some((class, field)) = jvm_class_of(&c.owner) {
                if c.kind.contains("static") {
                    specific.push(ProbeTarget::JvmStatic {
                        class: class.to_owned(),
                        field: field.to_owned(),
                    });
                }
                module.push(ProbeTarget::JvmClass {
                    class: class.to_owned(),
                });
            }
        }
        Runtime::Python => {
            let m = python_module(project, file);
            let rest = c
                .owner
                .strip_prefix(&format!("{m}."))
                .unwrap_or(&c.owner)
                .to_owned();
            match c.kind.as_str() {
                "module-global" | "module-pointer" if !rest.contains('.') => {
                    specific.push(ProbeTarget::PyGlobal {
                        module: m.clone(),
                        name: rest,
                    });
                }
                "class-attr" | "cache-decorator" | "class-closure" if rest.contains('.') => {
                    specific.push(ProbeTarget::PyAttr {
                        module: m.clone(),
                        qualname: rest,
                    });
                }
                _ => {}
            }
            module.push(ProbeTarget::PyModule { module: m });
        }
        Runtime::Node => module.push(ProbeTarget::NodeModule {
            file: file.to_owned(),
        }),
        Runtime::Go => {
            if driver && c.kind == "package-var" {
                let dir = file.rsplit_once('/').map_or("", |(d, _)| d);
                if go_dir.is_none_or(|g| g == dir)
                    && let Some((_, name)) = c.owner.rsplit_once('.')
                {
                    specific.push(ProbeTarget::GoVar {
                        package_dir: dir.to_owned(),
                        name: name.to_owned(),
                    });
                }
            }
        }
        Runtime::Rust | Runtime::Native => {}
    }
    (specific, module)
}

/// The probes of a harness.
#[derive(Debug, Clone, Default)]
pub struct Derived {
    /// The probes, `P1`.. in order.
    pub probes: Vec<ProbeSpec>,
    /// Notes for the manifest (gaps).
    pub notes: Vec<String>,
}

/// Derives the probes of a harness for `rt`: census candidates when `auto`, plus one probe
/// set per `extra` finding. `go_dir` limits Go probes to one package directory.
///
/// Refuses `unknown-finding` for an `extra` finding the ledger does not have and
/// `invalid-value` for one whose site belongs to another runtime.
pub fn derive(
    ctx: &ToolCtx,
    rt: Runtime,
    driver: bool,
    auto: bool,
    extra: &[FindingId],
    go_dir: Option<&str>,
) -> Result<Derived, Refusal> {
    let ledger = ctx.ledger()?;
    let mut cands: Vec<Cand> = if auto {
        candidates(ctx, &ledger, rt)
    } else {
        Vec::new()
    };
    let known = census_entries(ctx, rt);
    for id in extra {
        let f = ledger.finding(id).ok_or_else(|| {
            Refusal::new(
                codes::UNKNOWN_FINDING,
                format!("--probe {id}: the ledger has no such finding"),
                format!("finding {id}"),
                "dla report   (lists the findings), or drop --probe",
            )
        })?;
        let Some(site) = f.site.clone() else {
            return Err(Refusal::new(
                codes::SITE_REQUIRED,
                format!("--probe {id}: the finding has no site to derive a probe from"),
                format!("site on finding {id}"),
                "dla ledger annotate ... or probe another finding",
            ));
        };
        if file_runtime(site_file(&site)) != Some(rt) {
            return Err(Refusal::new(
                codes::INVALID_VALUE,
                format!(
                    "--probe {id}: its site {site} is not {} code, so this harness cannot probe it",
                    rt.as_str()
                ),
                format!("a {} finding", rt.as_str()),
                format!("dla harness gen --lang {} --probe {id}", rt.as_str()),
            ));
        }
        let owner = f
            .owner
            .clone()
            .or_else(|| {
                known
                    .iter()
                    .find(|e| e.site == site)
                    .map(|e| e.owner.clone())
            })
            .unwrap_or_default();
        let kind = known
            .iter()
            .find(|e| e.site == site)
            .map_or_else(String::new, |e| e.kind.clone());
        if let Some(c) = cands.iter_mut().find(|c| c.site == site) {
            c.finding = Some(id.clone());
        } else {
            cands.push(Cand {
                finding: Some(id.clone()),
                owner,
                kind,
                site,
            });
        }
    }

    let mut out = Derived::default();
    let mut specific: Vec<(ProbeTarget, Option<FindingId>, Option<String>)> = Vec::new();
    // Module-level targets with the candidates that need them.
    let mut modules: Vec<(ProbeTarget, Vec<&Cand>)> = Vec::new();
    for c in &cands {
        let (sp, md) = targets_for(&ctx.project, rt, c, driver, go_dir);
        for t in sp {
            if !specific.iter().any(|(x, _, _)| *x == t) {
                specific.push((t, c.finding.clone(), Some(c.site.clone())));
            }
        }
        for t in md {
            match modules.iter_mut().find(|(x, _)| *x == t) {
                Some((_, v)) => v.push(c),
                None => modules.push((t, vec![c])),
            }
        }
    }
    if rt == Runtime::Go && !driver && !cands.is_empty() {
        out.notes.push(
            "go command mode has no package-variable probes: growth shows in the goroutine groups and runtime totals (use --driver for GoVar probes)".to_owned(),
        );
    }
    let mut n = 0u32;
    let mut next_id = || {
        n += 1;
        format!("P{n}")
    };
    for (t, finding, site) in specific {
        out.probes.push(ProbeSpec {
            id: next_id(),
            finding,
            target: t,
            site,
        });
    }
    for (t, owners) in modules {
        // A module probe stands for a single candidate only when it has exactly one.
        let (finding, site) = match owners.as_slice() {
            [one] if !matches!(t, ProbeTarget::NodeModule { .. }) => {
                (one.finding.clone(), Some(one.site.clone()))
            }
            _ => (None, None),
        };
        out.probes.push(ProbeSpec {
            id: next_id(),
            finding,
            target: t,
            site,
        });
    }
    Ok(out)
}

/// The text of `probes.json`: `{"probes": [...]}` with the scanned top-level `names` added to
/// every `node-module` target (the contract type has no such field).
pub fn probes_json(project: &Path, probes: &[ProbeSpec]) -> String {
    let mut list: Vec<Value> = Vec::new();
    for p in probes {
        let mut v = serde_json::to_value(p).unwrap_or(Value::Null);
        if let ProbeTarget::NodeModule { file } = &p.target
            && let Some(lang) = Lang::from_path(Path::new(file))
            && let Ok(src) = fs::read_to_string(project.join(file))
            && let Some(t) = v.get_mut("target").and_then(Value::as_object_mut)
        {
            t.insert("names".into(), json!(node_top_level_names(lang, &src)));
        }
        list.push(v);
    }
    let mut s = serde_json::to_string_pretty(&json!({ "probes": list })).unwrap_or_default();
    s.push('\n');
    s
}

// ---------------------------------------------------------------------------------------
// Site resolution
// ---------------------------------------------------------------------------------------

/// Last dot-separated segment.
fn last_segment(s: &str) -> &str {
    s.rsplit('.').next().unwrap_or(s)
}

/// File of a dotted JVM class (`shop.OrderCache`) under the project: `**/shop/OrderCache.java`.
fn jvm_file(project: &Path, class: &str) -> Option<String> {
    let rel_tail = format!("{}.java", class.replace('.', "/"));
    let mut stack = vec![project.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            let name = e.file_name().to_string_lossy().into_owned();
            let path = e.path();
            if ft.is_dir() {
                if !crate::model::SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(path);
                }
            } else if let Some(rel) = dla_core::paths::rel_slash(project, &path)
                && (rel == rel_tail || rel.ends_with(&format!("/{rel_tail}")))
            {
                return Some(rel);
            }
        }
    }
    None
}

/// Declaration line of a dotted path in a Python (`mod.name`, `mod.Class.attr`) or Java
/// (`pkg.Class.FIELD`) project; `(file, line)`.
fn declared_dotted(project: &Path, rt: Runtime, path: &str) -> Option<String> {
    let segs: Vec<&str> = path.split('.').collect();
    match rt {
        Runtime::Python => {
            for cut in (1..segs.len()).rev() {
                let module = segs[..cut].join("/");
                for cand in [format!("{module}.py"), format!("{module}/__init__.py")] {
                    let Ok(src) = fs::read_to_string(project.join(&cand)) else {
                        continue;
                    };
                    let rest = &segs[cut..];
                    let (scope, name) = match rest {
                        [name] => (None, *name),
                        [scope, name, ..] => (Some(*scope), *name),
                        [] => continue,
                    };
                    if let Some(line) = declaration_line(Lang::Python, &src, name, scope) {
                        return Some(format!("{cand}:{line}"));
                    }
                }
            }
            None
        }
        Runtime::Jvm => {
            if segs.len() < 2 {
                return None;
            }
            let (class, field) = (segs[..segs.len() - 1].join("."), segs[segs.len() - 1]);
            let file = jvm_file(project, &class)?;
            let src = fs::read_to_string(project.join(&file)).ok()?;
            let simple = last_segment(&class);
            let line = declaration_line(Lang::Java, &src, field, Some(simple))?;
            Some(format!("{file}:{line}"))
        }
        Runtime::Go => {
            let (name, dir) = (segs.last()?, segs[..segs.len() - 1].join("/"));
            let pkg_dir = dir.rsplit_once('/').map_or("", |(d, _)| d);
            for e in fs::read_dir(project.join(pkg_dir)).ok()?.flatten() {
                let n = e.file_name().to_string_lossy().into_owned();
                if n.ends_with(".go")
                    && !n.ends_with("_test.go")
                    && let Ok(src) = fs::read_to_string(e.path())
                    && let Some(line) = declaration_line(Lang::Go, &src, name, None)
                {
                    return Some(format!("{}:{line}", join_rel(pkg_dir, &n)));
                }
            }
            None
        }
        _ => None,
    }
}

fn join_rel(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_owned()
    } else {
        format!("{dir}/{name}")
    }
}

/// Core of [`resolve_site`] over already loaded census entries.
pub fn resolve_in(
    project: &Path,
    rt: Runtime,
    entries: &[CensusEntry],
    probe_path: &str,
) -> Option<String> {
    // Node: `src/scheduler.js:shared.latest`.
    if let Some((file, rest)) = probe_path.split_once(':')
        && !file.is_empty()
        && !rest.is_empty()
        && !file.contains('\\')
    {
        let module = file
            .rsplit_once('.')
            .map_or(file, |(m, _)| m)
            .replace('/', ".");
        let (name, prop) = rest
            .split_once('.')
            .map_or((rest, None), |(n, p)| (n, Some(p)));
        let exact = format!("{module}.{rest}");
        let binding = format!("{module}.{name}");
        let in_file = |e: &&CensusEntry| site_file(&e.site) == file;
        if let Some(e) = entries
            .iter()
            .find(|e| e.owner == exact)
            .or_else(|| entries.iter().find(|e| e.owner == binding))
            .or_else(|| {
                let prop = prop?;
                let last = last_segment(prop);
                let mut hits: Vec<&CensusEntry> = entries
                    .iter()
                    .filter(in_file)
                    .filter(|e| last_segment(&e.owner) == last)
                    .collect();
                hits.sort_by_key(|e| e.site.clone());
                hits.first().copied()
            })
        {
            return Some(e.site.clone());
        }
        let lang = Lang::from_path(Path::new(file))?;
        let src = fs::read_to_string(project.join(file)).ok()?;
        return declaration_line(lang, &src, name, None).map(|l| format!("{file}:{l}"));
    }
    // Dotted: exact owner, then progressively shorter prefixes of the path, then suffixes.
    if let Some(e) = entries.iter().find(|e| e.owner == probe_path) {
        return Some(e.site.clone());
    }
    let segs: Vec<&str> = probe_path.split('.').collect();
    for cut in (2..segs.len()).rev() {
        let prefix = segs[..cut].join(".");
        if let Some(e) = entries.iter().find(|e| e.owner == prefix) {
            return Some(e.site.clone());
        }
    }
    let suffix = format!(".{probe_path}");
    let mut by_suffix = entries
        .iter()
        .filter(|e| e.owner.ends_with(&suffix) || probe_path.ends_with(&format!(".{}", e.owner)));
    if let Some(e) = by_suffix.next() {
        return Some(e.site.clone());
    }
    declared_dotted(project, rt, probe_path)
}

/// The site and finding of a probe result path reported by an agent (for example
/// `shop.OrderCache.ORDERS_BY_ID`, `src/scheduler.js:shared.latest`,
/// `ingest.registry._LISTENERS`): the census entry whose owner matches, else the
/// top-level declaration of that name, else nothing.
pub fn resolve_site(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    probe_path: &str,
) -> (Option<String>, Option<FindingId>) {
    let entries = census_entries(ctx, manifest.runtime);
    let Some(site) = resolve_in(&ctx.project, manifest.runtime, &entries, probe_path) else {
        return (None, None);
    };
    let finding = ctx.ledger().ok().and_then(|l| finding_for_site(&l, &site));
    (Some(site), finding)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: &str, owner: &str, site: &str) -> CensusEntry {
        CensusEntry {
            kind: kind.into(),
            holder: dla_core::finding::Holder::StaticGlobal,
            region: dla_core::finding::Region::Heap,
            site: site.into(),
            owner: owner.into(),
            writers: Vec::new(),
            removers: Vec::new(),
        }
    }

    #[test]
    fn js_top_level_names() {
        let src = "const a = 1, { b, c: d } = x;\nlet shared = null;\nclass Sched { static n = 0; }\nfunction f() {}\nexport const e = new Map();\nfunction g() { const inner = 1; }\n";
        assert_eq!(
            node_top_level_names(Lang::JavaScript, src),
            ["a", "b", "d", "shared", "Sched", "e"]
        );
        assert_eq!(
            declaration_line(Lang::JavaScript, src, "shared", None),
            Some(2)
        );
    }

    #[test]
    fn python_declarations_and_members() {
        let src = "import functools\n_LISTENERS = {}\n\nclass B:\n    cache = {}\n\n    @functools.lru_cache\n    def memo(self):\n        pass\n";
        assert_eq!(
            declaration_line(Lang::Python, src, "_LISTENERS", None),
            Some(2)
        );
        assert_eq!(
            declaration_line(Lang::Python, src, "cache", Some("B")),
            Some(5)
        );
        assert_eq!(
            declaration_line(Lang::Python, src, "memo", Some("B")),
            Some(8)
        );
        assert_eq!(declaration_line(Lang::Python, src, "nothing", None), None);
    }

    #[test]
    fn java_fields_and_go_vars() {
        let java = "package shop;\npublic class OrderCache {\n    private static final int MAX = 3;\n    static final java.util.Map<String, String> ORDERS = new java.util.HashMap<>();\n}\n";
        assert_eq!(
            declaration_line(Lang::Java, java, "ORDERS", Some("OrderCache")),
            Some(4)
        );
        let go = "package w\n\nvar (\n\ttopics = []string{}\n\tother  int\n)\n";
        assert_eq!(declaration_line(Lang::Go, go, "other", None), Some(5));
    }

    #[test]
    fn resolve_prefers_exact_owner_then_suffix() {
        let entries = vec![
            entry(
                "static-container",
                "shop.OrderCache.ORDERS_BY_ID",
                "src/shop/OrderCache.java:8",
            ),
            entry(
                "module-global",
                "src.scheduler.shared",
                "src/scheduler.js:90",
            ),
            entry(
                "instance-field",
                "src.scheduler.RetryScheduler.latest",
                "src/scheduler.js:15",
            ),
        ];
        let p = Path::new("/nonexistent");
        assert_eq!(
            resolve_in(p, Runtime::Jvm, &entries, "shop.OrderCache.ORDERS_BY_ID").as_deref(),
            Some("src/shop/OrderCache.java:8")
        );
        assert_eq!(
            resolve_in(p, Runtime::Jvm, &entries, "OrderCache.ORDERS_BY_ID").as_deref(),
            Some("src/shop/OrderCache.java:8")
        );
        assert_eq!(
            resolve_in(p, Runtime::Node, &entries, "src/scheduler.js:shared").as_deref(),
            Some("src/scheduler.js:90")
        );
        // The binding's line (spec 4.6), not the property's.
        assert_eq!(
            resolve_in(p, Runtime::Node, &entries, "src/scheduler.js:shared.latest").as_deref(),
            Some("src/scheduler.js:90")
        );
        // Without a census entry for the binding, the file's entry named like the property.
        assert_eq!(
            resolve_in(
                p,
                Runtime::Node,
                &entries[2..],
                "src/scheduler.js:shared.latest"
            )
            .as_deref(),
            Some("src/scheduler.js:15")
        );
        assert_eq!(
            resolve_in(p, Runtime::Node, &entries, "src/nowhere.js:x"),
            None
        );
    }

    #[test]
    fn python_module_names() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            python_module(d.path(), "ingest/registry.py"),
            "ingest.registry"
        );
        assert_eq!(python_module(d.path(), "ingest/__init__.py"), "ingest");
        assert_eq!(python_module(d.path(), "src/pkg/m.py"), "pkg.m");
    }

    #[test]
    fn candidates_map_to_targets() {
        let d = tempfile::tempdir().unwrap();
        let c = |kind: &str, owner: &str, site: &str| Cand {
            finding: None,
            owner: owner.into(),
            kind: kind.into(),
            site: site.into(),
        };
        let (sp, md) = targets_for(
            d.path(),
            Runtime::Jvm,
            &c(
                "static-container",
                "shop.OrderCache.ORDERS_BY_ID",
                "src/shop/OrderCache.java:8",
            ),
            false,
            None,
        );
        assert_eq!(
            sp,
            [ProbeTarget::JvmStatic {
                class: "shop.OrderCache".into(),
                field: "ORDERS_BY_ID".into()
            }]
        );
        assert_eq!(
            md,
            [ProbeTarget::JvmClass {
                class: "shop.OrderCache".into()
            }]
        );
        let (sp, md) = targets_for(
            d.path(),
            Runtime::Python,
            &c(
                "cache-decorator",
                "reports.builder.ReportBuilder.category_totals",
                "reports/builder.py:14",
            ),
            false,
            None,
        );
        assert_eq!(
            sp,
            [ProbeTarget::PyAttr {
                module: "reports.builder".into(),
                qualname: "ReportBuilder.category_totals".into()
            }]
        );
        assert_eq!(
            md,
            [ProbeTarget::PyModule {
                module: "reports.builder".into()
            }]
        );
        let (sp, _) = targets_for(
            d.path(),
            Runtime::Python,
            &c(
                "module-global",
                "ingest.registry._LISTENERS",
                "ingest/registry.py:3",
            ),
            false,
            None,
        );
        assert_eq!(
            sp,
            [ProbeTarget::PyGlobal {
                module: "ingest.registry".into(),
                name: "_LISTENERS".into()
            }]
        );
        let (sp, _) = targets_for(
            d.path(),
            Runtime::Go,
            &c(
                "package-var",
                "internal.workload.workload.topics",
                "internal/workload/workload.go:19",
            ),
            false,
            None,
        );
        assert!(sp.is_empty(), "go command mode has no GoVar");
        let (sp, _) = targets_for(
            d.path(),
            Runtime::Go,
            &c(
                "package-var",
                "internal.workload.workload.topics",
                "internal/workload/workload.go:19",
            ),
            true,
            None,
        );
        assert_eq!(
            sp,
            [ProbeTarget::GoVar {
                package_dir: "internal/workload".into(),
                name: "topics".into()
            }]
        );
    }

    #[test]
    fn probes_json_adds_node_names() {
        let d = tempfile::tempdir().unwrap();
        fs::write(
            d.path().join("m.js"),
            "const a = 1;\nlet b;\nfunction f(){}\n",
        )
        .unwrap();
        let probes = vec![ProbeSpec {
            id: "P1".into(),
            finding: None,
            target: ProbeTarget::NodeModule {
                file: "m.js".into(),
            },
            site: None,
        }];
        let v: Value = serde_json::from_str(&probes_json(d.path(), &probes)).unwrap();
        assert_eq!(v["probes"][0]["target"]["kind"], "node-module");
        assert_eq!(v["probes"][0]["target"]["names"], json!(["a", "b"]));
    }
}
