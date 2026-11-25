//! The retention path of a heap-graph holder, in words: `kept_by` (the path from the root to
//! the holder, short) and `root` (what the root is, and for a native handle the project line
//! of the callback it calls).
//!
//! The path is the one the parser already computed for the summary file
//! (`dla_parsers::node::Holder::path`, root first). Here it is made readable: the synthetic
//! `""` and `(GC roots)` steps go, runs of contexts and arrays become `...`, the edges of a
//! bound function read `bound function (this)`, and the whole is cut to [`PATH_CAP`]
//! characters.
//!
//! A line is given only when it is known: the function's row in the snapshot's `locations`
//! table gives a script id and a position; the `scripts.json` sidecar the agent writes next to
//! the snapshots maps the id to the script; a source map of a bundle maps the position to the
//! source file; and the file must be a project file. Anything else gives no line, and the text
//! says so. A line is never guessed.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use dla_parsers::node::{Holder, HolderKind, Hop, Site, sourcemap::ScriptMaps, url_to_path};
use serde_json::Value;

/// Longest a `kept_by` path is.
const PATH_CAP: usize = 160;
/// Largest `scripts.json` read.
const SIDECAR_BYTES: u64 = 16 << 20;
/// The edge of the objects a `WeakRef` was read from in the current job (never a holder).
const JOB_KEEP: &str = "weak_refs_keep_during_job";

/// What the output shows of one holder's retention.
#[derive(Debug, Clone, Default)]
pub struct Kept {
    /// `(Global handles) > HTTPParser[1] > bound function (this) > Conn.request`.
    pub kept_by: String,
    /// What the root is, in a sentence.
    pub root: String,
    /// True when a native handle keeps the objects.
    pub native: bool,
    /// `src/conn.ts:119`, the project line of the callback the native handle calls.
    pub line: Option<String>,
}

/// The script ids of one process: `scripts.json` next to the snapshots, and the source maps of
/// the scripts it names.
#[derive(Default)]
pub struct Sidecar {
    scripts: HashMap<u32, String>,
    /// Scripts the agent saw with a source map (a URL or inline).
    mapped: HashSet<u32>,
    maps: ScriptMaps,
}

impl Sidecar {
    /// The sidecar in the directory of `snapshot`; empty when there is none or it is damaged.
    pub fn beside(snapshot: &Path) -> Self {
        let mut me = Self::default();
        let Some(file) = snapshot.parent().map(|d| d.join("scripts.json")) else {
            return me;
        };
        if fs::metadata(&file).is_ok_and(|m| m.len() > SIDECAR_BYTES) {
            return me;
        }
        let Some(doc) = fs::read_to_string(&file)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        else {
            return me;
        };
        let ids = |key: &str| -> Vec<(u32, String)> {
            doc.get(key)
                .and_then(Value::as_object)
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| Some((k.parse().ok()?, v.as_str()?.to_owned())))
                        .collect()
                })
                .unwrap_or_default()
        };
        me.scripts = ids("scripts").into_iter().collect();
        me.mapped = ids("maps")
            .into_iter()
            .filter(|(_, m)| !m.is_empty())
            .map(|(id, _)| id)
            .collect();
        me
    }

    /// `src/conn.ts:119` for a creation site, when it maps to a project file.
    pub fn line(&mut self, site: Site, project: &Path) -> Option<String> {
        let url = self.scripts.get(&site.script_id)?.clone();
        let script = url_to_path(&url);
        let (file, line) = match self.maps.lookup(Path::new(&script), site.line, site.column) {
            Some((f, l)) => (f.to_string_lossy().into_owned(), l),
            // A script with a map the reader cannot follow (inline, remote, missing) has a
            // position in the generated file, not in the source: no line.
            None if self.mapped.contains(&site.script_id) => return None,
            None => (script, site.line),
        };
        project_file(&file, project).map(|rel| format!("{rel}:{}", line + 1))
    }
}

/// The path of a project file relative to the project, `None` for a package, a Node module
/// or a file that is not the project's.
fn project_file(path: &str, project: &Path) -> Option<String> {
    let s = path.replace('\\', "/");
    if s.starts_with("node:") {
        return None;
    }
    let in_modules = |r: &str| r.starts_with("node_modules/") || r.contains("/node_modules/");
    if let Some(rel) = super::node_heapsnapshot::harness_work_relative(&s) {
        return (!in_modules(rel)).then(|| rel.to_owned());
    }
    if let Ok(rel) = Path::new(&s).strip_prefix(project) {
        let rel = rel.to_string_lossy().replace('\\', "/");
        return (!in_modules(&rel)).then_some(rel);
    }
    if s.contains("node_modules/") {
        return None;
    }
    let parts: Vec<&str> = s
        .split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect();
    // The longest tail that exists in the project; a one-component tail matches only a path
    // that is one component.
    for i in 0..parts.len() {
        if parts.len() - i < 2 && parts.len() > 1 {
            break;
        }
        let tail = parts[i..].join("/");
        if project.join(&tail).is_file() {
            return Some(tail);
        }
    }
    None
}

fn a_an(word: &str) -> &'static str {
    let mut c = word.chars();
    let (first, second) = (c.next(), c.next());
    match first {
        // An acronym is read letter by letter: an HTTPParser, a URLTable.
        Some(f) if f.is_ascii_uppercase() && second.is_some_and(|s| s.is_ascii_uppercase()) => {
            if "AEFHILMNORSX".contains(f) {
                "an"
            } else {
                "a"
            }
        }
        Some('a' | 'e' | 'i' | 'o' | 'A' | 'E' | 'I' | 'O') => "an",
        _ => "a",
    }
}

/// A root that is native code's: a handle the runtime keeps for something outside the heap
/// (V8's handle roots, and the C++ objects of Node that the snapshot names `Node / <Class>`).
fn is_handle_root(name: &str) -> bool {
    name.starts_with("Node / ")
        || matches!(
            name,
            "(Global handles)"
                | "(Strong roots)"
                | "(Handle scope)"
                | "(Stack roots)"
                | "(Eternal handles)"
        )
}

fn is_synthetic(name: &str) -> bool {
    name.is_empty() || name == "(GC roots)"
}

/// A step that says nothing a reader needs: a context or an array.
fn is_filler(name: &str) -> bool {
    name.starts_with("system / ")
        || matches!(
            name,
            "Array" | "(array)" | "(object elements)" | "(object properties)" | "(internal array)"
        )
}

fn is_bound(edge: &str) -> bool {
    edge == ".bound_this" || edge == ".bound_function" || edge.starts_with(".bound_argument")
}

fn bound_words(edge: &str) -> String {
    match edge {
        ".bound_this" => "bound function (this)".to_owned(),
        ".bound_function" => "bound function (target)".to_owned(),
        e => format!(
            "bound function (argument {})",
            e.trim_start_matches(".bound_argument")
                .trim_start_matches('_')
        ),
    }
}

/// The steps of the shown path: `None` is a `...`.
fn steps(path: &[Hop]) -> Vec<Option<String>> {
    let mut out: Vec<Option<String>> = Vec::new();
    let last = path.len().saturating_sub(1);
    for (i, h) in path.iter().enumerate() {
        if is_synthetic(&h.node) && h.edge != "..." {
            continue;
        }
        let step = if h.edge == "..." {
            None
        } else if i == last {
            Some(format!("{}{}", h.node, h.edge))
        } else if is_bound(&h.edge) {
            Some(bound_words(&h.edge))
        } else if is_filler(&h.node) {
            None
        } else if is_handle_root(&h.node) {
            Some(h.node.clone())
        } else {
            Some(format!("{}{}", h.node, h.edge))
        };
        if step.is_none() && out.last().is_some_and(Option::is_none) {
            continue;
        }
        out.push(step);
    }
    out
}

fn shown(parts: &[Option<String>]) -> String {
    parts
        .iter()
        .map(|p| p.as_deref().unwrap_or("..."))
        .collect::<Vec<_>>()
        .join(" > ")
}

/// The path text, cut to [`PATH_CAP`]: the first step, `...`, and as many of the last steps
/// as fit.
fn kept_by(path: &[Hop]) -> String {
    let parts = steps(path);
    let all = shown(&parts);
    if all.chars().count() <= PATH_CAP {
        return all;
    }
    for keep in (1..parts.len()).rev() {
        let mut cut = vec![parts[0].clone(), None];
        cut.extend(parts[parts.len() - keep..].iter().cloned());
        let t = shown(&cut);
        if t.chars().count() <= PATH_CAP {
            return t;
        }
    }
    let last = parts.last().and_then(Clone::clone).unwrap_or_default();
    let t: String = last.chars().take(PATH_CAP - 4).collect();
    format!("... > {t}")
}

/// What the root of a holder is, and the line of the callback a native handle calls.
fn root_of(
    h: &Holder,
    in_project: bool,
    side: &mut Sidecar,
    project: &Path,
) -> (String, bool, Option<String>) {
    let path = &h.path;
    let Some(at) = path.iter().position(|p| !is_synthetic(&p.node)) else {
        return (
            "the path of this holder is not recorded".to_owned(),
            false,
            None,
        );
    };
    let first = &path[at];
    if first.edge.trim_start_matches('.') == JOB_KEEP {
        return (
            "the WeakRef list of the current job, which is never a holder".to_owned(),
            false,
            None,
        );
    }
    let owner = if in_project {
        "of the project"
    } else {
        "of a library"
    };
    // A strong root or a stack root above a variable of a function says nothing about native
    // code: only a property chain from a handle, or a global handle, does.
    let kind = h.root_kind.unwrap_or(h.kind);
    let native = is_handle_root(&first.node)
        && match kind {
            HolderKind::Static => false,
            // A listener or timer below a handle: the handle is the root.
            HolderKind::Registration => h.registry.as_deref() != Some("finalization"),
            HolderKind::Closure => {
                first.node == "(Global handles)" || first.node.starts_with("Node / ")
            }
            HolderKind::Instance => true,
        };
    if !native {
        let name = |edge: &str| format!(" ({})", edge.trim_start_matches('.'));
        let text = match kind {
            HolderKind::Static => match path.iter().find(|p| p.node == "global") {
                Some(g) => format!("a global {owner}{}", name(&g.edge)),
                None => {
                    let var = path
                        .iter()
                        .find(|p| p.node.starts_with("system / ") && p.edge.starts_with('.'));
                    format!(
                        "a module variable {owner}{}",
                        var.map_or(String::new(), |v| name(&v.edge))
                    )
                }
            },
            HolderKind::Registration if h.registry.as_deref() == Some("finalization") => {
                format!("a finalization registry {owner}")
            }
            HolderKind::Registration => {
                format!("a timer or event-listener registration {owner}")
            }
            HolderKind::Closure => "a variable of a function (closure context)".to_owned(),
            HolderKind::Instance => "an object property reachable from the roots".to_owned(),
        };
        return (text, false, None);
    }
    // A native handle: the callback of the project on the path, nearest to the holder first.
    let last = path.len() - 1;
    let funcs: Vec<usize> = (at + 1..last)
        .filter(|&i| is_bound(&path[i].edge) || path[i].site.is_some())
        .collect();
    let mut chosen: Option<(usize, Option<String>)> = None;
    for &i in funcs.iter().rev() {
        if let Some(l) = path[i].site.and_then(|s| side.line(s, project)) {
            chosen = Some((i, Some(l)));
            break;
        }
    }
    let chosen = chosen.or_else(|| funcs.last().map(|&i| (i, None)));
    let first_named = path[at + 1..]
        .iter()
        .find(|p| !is_filler(&p.node) && !is_bound(&p.edge))
        .map_or("an object", |p| p.node.as_str());
    let Some((i, line)) = chosen else {
        let t = format!(
            "a native handle keeps {} {first_named}; no function of a script is on the path, so no line is given",
            a_an(first_named)
        );
        return (t, true, None);
    };
    let what = if is_bound(&path[i].edge) {
        "bound function"
    } else {
        "function"
    };
    let created = match &line {
        Some(l) => format!("created at {l}"),
        None => {
            "(its creation line is not known: no location of it maps to a project file)".to_owned()
        }
    };
    let this = (path[i].edge == ".bound_this")
        .then(|| path.get(i + 1))
        .flatten()
        .map(|n| format!(" whose this is the {}", n.node))
        .unwrap_or_default();
    // The step right after the root is the function itself, or an object that keeps it.
    let direct = path[at + 1..i].iter().all(|p| is_filler(&p.node));
    let head = if direct {
        format!("a native handle keeps a {what} {created}{this}")
    } else {
        format!(
            "a native handle keeps {} {first_named}, which keeps a {what} {created}{this}",
            a_an(first_named)
        )
    };
    (head, true, line)
}

/// The shown retention of a holder.
pub fn describe(h: &Holder, in_project: bool, side: &mut Sidecar, project: &Path) -> Kept {
    let (root, native, line) = root_of(h, in_project, side, project);
    Kept {
        kept_by: kept_by(&h.path),
        root,
        native,
        line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hop(node: &str, edge: &str) -> Hop {
        Hop {
            node: node.into(),
            edge: edge.into(),
            site: None,
        }
    }

    fn holder(kind: HolderKind, root_kind: HolderKind, path: Vec<Hop>) -> Holder {
        Holder {
            kind,
            registry: None,
            name: "request".into(),
            owner: "Conn".into(),
            script: None,
            retained: 1,
            count: 1,
            instances: 1,
            path,
            accumulation: None,
            root_kind: Some(root_kind),
        }
    }

    fn bound_path(site: Option<Site>) -> Vec<Hop> {
        let mut bind = hop("native_bind", ".bound_this");
        bind.site = site;
        vec![
            hop("", "[0]"),
            hop("(GC roots)", "[10]"),
            hop("(Global handles)", ".2"),
            hop("HTTPParser", "[1]"),
            bind,
            hop("Conn", ".request"),
        ]
    }

    /// A project with a `src/conn.js` and a sidecar that names script 12 by a path of another
    /// machine.
    fn project(sidecar: &str) -> (tempfile::TempDir, Sidecar) {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("src")).unwrap();
        fs::write(d.path().join("src/conn.js"), "x\n").unwrap();
        fs::create_dir_all(d.path().join("in")).unwrap();
        fs::write(d.path().join("in/scripts.json"), sidecar).unwrap();
        let side = Sidecar::beside(&d.path().join("in/heap-1.heapsnapshot"));
        (d, side)
    }

    const SIDECAR: &str = r#"{"dla_scripts":1,"scripts":{"12":"file:///ci/work/src/conn.js","13":"node:net"},"maps":{}}"#;

    fn site(id: u32, line: u32) -> Site {
        Site {
            script_id: id,
            line,
            column: 2,
        }
    }

    #[test]
    fn the_path_drops_the_synthetic_roots_and_reads_the_bound_edges() {
        assert_eq!(
            kept_by(&bound_path(None)),
            "(Global handles) > HTTPParser[1] > bound function (this) > Conn.request"
        );
        let other = [
            (".bound_function", "bound function (target)"),
            (".bound_argument_1", "bound function (argument 1)"),
        ];
        for (edge, words) in other {
            let mut p = bound_path(None);
            p[4].edge = edge.into();
            assert!(kept_by(&p).contains(words), "{edge}");
        }
    }

    #[test]
    fn runs_of_contexts_and_arrays_collapse_to_one_gap_and_the_holder_stays() {
        let p = vec![
            hop("", "[0]"),
            hop("(GC roots)", "[1]"),
            hop("(Strong roots)", "[3]"),
            hop("system / Context", ".previous"),
            hop("system / Context", ".previous"),
            hop("Array", "[4]"),
            hop("Cache", ".items"),
            hop("system / Context", ".latest"),
        ];
        assert_eq!(
            kept_by(&p),
            "(Strong roots) > ... > Cache.items > system / Context.latest"
        );
        // The gap the parser itself leaves for a long path is the same gap.
        let g = vec![
            hop("Server", ".conns"),
            hop("", "..."),
            hop("Array", "[2]"),
            hop("Conn", ".request"),
        ];
        assert_eq!(kept_by(&g), "Server.conns > ... > Conn.request");
    }

    #[test]
    fn a_long_path_is_cut_to_the_cap_between_the_first_step_and_the_last_ones() {
        let mut p = vec![hop("(Global handles)", "[1]")];
        for i in 0..30 {
            p.push(hop(&format!("StepNumber{i}"), ".next"));
        }
        p.push(hop("Conn", ".request"));
        let t = kept_by(&p);
        assert!(t.chars().count() <= PATH_CAP, "{t}");
        assert!(t.starts_with("(Global handles) > ... > "), "{t}");
        assert!(t.ends_with("Conn.request"), "{t}");
    }

    #[test]
    fn a_native_root_names_the_line_the_function_maps_to() {
        let (d, mut side) = project(SIDECAR);
        let h = holder(
            HolderKind::Instance,
            HolderKind::Instance,
            bound_path(Some(site(12, 118))),
        );
        let k = describe(&h, true, &mut side, d.path());
        assert!(k.native);
        assert_eq!(k.line.as_deref(), Some("src/conn.js:119"));
        assert_eq!(
            k.root,
            "a native handle keeps an HTTPParser, which keeps a bound function created at src/conn.js:119 whose this is the Conn"
        );
        // Node's own script, a script the sidecar does not know and a function with no row
        // map to no line.
        for s in [Some(site(13, 3)), Some(site(99, 3)), None] {
            let h = holder(HolderKind::Instance, HolderKind::Instance, bound_path(s));
            let k = describe(&h, true, &mut side, d.path());
            assert!(k.native && k.line.is_none(), "{s:?}");
            assert!(
                k.root.contains("(its creation line is not known"),
                "{}",
                k.root
            );
            assert!(!k.root.contains(".js:"), "{}", k.root);
        }
    }

    #[test]
    fn the_line_comes_from_the_source_map_of_a_bundle_and_not_from_the_bundle() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("dist")).unwrap();
        fs::create_dir_all(d.path().join("in")).unwrap();
        fs::write(
            d.path().join("dist/b.js"),
            "a\n\n\nfn()\n//# sourceMappingURL=b.js.map\n",
        )
        .unwrap();
        fs::write(
            d.path().join("dist/b.js.map"),
            r#"{"version":3,"sources":["../src/conn.ts"],"names":[],"mappings":";;;AAyBA"}"#,
        )
        .unwrap();
        fs::write(d.path().join("dist/c.js"), "a\n\n\nfn()\n").unwrap();
        let sidecar = format!(
            r#"{{"scripts":{{"7":"file://{}","8":"file://{}"}},"maps":{{"7":"b.js.map","8":"inline"}}}}"#,
            d.path().join("dist/b.js").display(),
            d.path().join("dist/c.js").display()
        );
        fs::write(d.path().join("in/scripts.json"), sidecar).unwrap();
        let mut side = Sidecar::beside(&d.path().join("in/heap-1.heapsnapshot"));
        assert_eq!(
            side.line(site(7, 3), d.path()).as_deref(),
            Some("src/conn.ts:26")
        );
        // Script 8 says it has a map the reader cannot follow: the bundle's own line is no
        // line of the source.
        assert_eq!(side.line(site(8, 3), d.path()), None);
    }

    #[test]
    fn a_package_or_a_foreign_file_is_no_project_line() {
        let (d, _) = project(SIDECAR);
        assert_eq!(
            project_file(
                &format!("{}/node_modules/x/a.js", d.path().display()),
                d.path()
            ),
            None
        );
        assert_eq!(project_file("/elsewhere/lib/other.js", d.path()), None);
        assert_eq!(project_file("node:net", d.path()).as_deref(), None);
        assert_eq!(
            project_file("/ci/work/src/conn.js", d.path()).as_deref(),
            Some("src/conn.js")
        );
    }

    #[test]
    fn roots_that_are_not_native_say_what_they_are_and_give_no_line() {
        let (d, mut side) = project(SIDECAR);
        let ctx = vec![
            hop("", "[0]"),
            hop("(GC roots)", "[1]"),
            hop("system / Context", ".shared"),
            hop("Cache", ".items"),
        ];
        let words = |h: &Holder, project: bool, side: &mut Sidecar| {
            describe(h, project, side, d.path()).root
        };
        let h = holder(HolderKind::Instance, HolderKind::Static, ctx);
        assert_eq!(
            words(&h, true, &mut side),
            "a module variable of the project (shared)"
        );
        assert_eq!(
            words(&h, false, &mut side),
            "a module variable of a library (shared)"
        );
        let g = vec![
            hop("(GC roots)", "[1]"),
            hop("global", ".cache"),
            hop("Map", ".items"),
        ];
        let h = holder(HolderKind::Instance, HolderKind::Static, g);
        assert_eq!(words(&h, false, &mut side), "a global of a library (cache)");
        // A handle above a module variable is still a module variable: the static decides.
        let both = vec![
            hop("(Strong roots)", "[1]"),
            hop("system / Context", ".shared"),
            hop("Cache", ".items"),
        ];
        let h = holder(HolderKind::Instance, HolderKind::Static, both);
        let k = describe(&h, true, &mut side, d.path());
        assert!(
            !k.native && k.line.is_none() && !k.root.contains("native"),
            "{}",
            k.root
        );
        let mut h = holder(
            HolderKind::Registration,
            HolderKind::Registration,
            vec![hop("(GC roots)", "[1]"), hop("Map", ".cells")],
        );
        h.registry = Some("finalization".into());
        assert_eq!(
            words(&h, true, &mut side),
            "a finalization registry of the project"
        );
        // A variable of a function reached through a strong root: a closure, not native.
        let h = holder(
            HolderKind::Closure,
            HolderKind::Closure,
            vec![
                hop("(GC roots)", "[1]"),
                hop("(Strong roots)", "[2]"),
                hop("system / Context", ".board"),
            ],
        );
        let k = describe(&h, true, &mut side, d.path());
        assert!(
            !k.native && k.root.contains("closure context"),
            "{}",
            k.root
        );
    }

    #[test]
    fn the_weak_ref_job_list_is_never_a_holder_and_is_said_so() {
        let (d, mut side) = project(SIDECAR);
        let h = holder(
            HolderKind::Instance,
            HolderKind::Instance,
            vec![
                hop("(Strong roots)", ".weak_refs_keep_during_job"),
                hop("Conn", ".request"),
            ],
        );
        let k = describe(&h, true, &mut side, d.path());
        assert!(!k.native && k.root.contains("never a holder"), "{}", k.root);
    }

    #[test]
    fn a_handle_that_keeps_the_object_with_no_function_between_says_so() {
        let (d, mut side) = project(SIDECAR);
        let h = holder(
            HolderKind::Instance,
            HolderKind::Instance,
            vec![
                hop("(GC roots)", "[1]"),
                hop("Node / TCPWrap", ".native_to_javascript"),
                hop("Socket", "._handle"),
                hop("Conn", ".request"),
            ],
        );
        let k = describe(&h, true, &mut side, d.path());
        assert!(k.native && k.line.is_none());
        assert!(
            k.root.starts_with("a native handle keeps a Socket"),
            "{}",
            k.root
        );
        assert!(k.root.contains("no line is given"), "{}", k.root);
    }
}
