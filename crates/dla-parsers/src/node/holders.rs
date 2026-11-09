//! Holders: the named references that keep a growing structure alive, found from the
//! dominator tree of one snapshot.
//!
//! A holder is an edge `u --name--> v` where `u` alone keeps `v` alive (`u` is the
//! immediate dominator of `v`) and `v` retains a sizeable part of the heap. The edge is
//! classified from what `u` is:
//!
//! * a module-scope context variable, or a property of the global object: `Static`
//!   (static/global);
//! * a variable of a function context: `Closure` (closure/continuation);
//! * the callback of a timer, or a listener in an emitter's `_events`: `Registration`
//!   (registration-table);
//! * any other property of an object: `Instance` (container-field).
//!
//! Holders of the same kind, script, owner and name are summed across their instances
//! (every session's closure variable is one holder). The script of a context or a class
//! comes from the closures that use the context and from the class constructor's
//! `SharedFunctionInfo`, whose `script` edge names the file.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::{graph::Bfs, graph::Dominators, snapshot::Snapshot};

const NONE: u32 = u32::MAX;
/// Hops of a retention path kept.
const PATH_HOPS: usize = 10;
/// Longest name kept.
const NAME_MAX: usize = 120;

/// What kind of reference a holder is.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum HolderKind {
    /// A module-scope variable or a global.
    Static,
    /// A variable of a function context.
    Closure,
    /// A timer callback or an event listener.
    Registration,
    /// A property of an object.
    Instance,
}

/// What piles up under a holder.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Accumulation {
    /// The most common class below the holder (`(closure) beat`, `Session`).
    pub class: String,
    pub count: u64,
    pub bytes: u64,
}

/// One step of a retention path: a node and the edge leaving it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Hop {
    /// The node's name: a constructor, a function, a kind of root.
    pub node: String,
    /// `.name` for a named edge, `[n]` for an element or hidden edge, `...` for a gap.
    pub edge: String,
}

/// One holder in one snapshot.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Holder {
    pub kind: HolderKind,
    /// `timers` or `events` for a registration.
    pub registry: Option<String>,
    /// The variable, property, callback or event name.
    pub name: String,
    /// The class (instance, events) or enclosing function (closure, timers); empty when
    /// unknown.
    pub owner: String,
    /// The script that declares it, as the snapshot names it; `None` when unknown.
    pub script: Option<String>,
    /// Bytes retained, summed over instances.
    pub retained: u64,
    /// Objects retained, summed over instances.
    pub count: u64,
    /// How many references were summed.
    pub instances: u64,
    /// The retention path of the largest instance, root first.
    pub path: Vec<Hop>,
    pub accumulation: Option<Accumulation>,
}

/// Limits of the holder search.
#[derive(Debug, Clone)]
pub struct HolderOptions {
    /// A reference must retain at least this many bytes to be a candidate.
    pub min_retained: u64,
    /// Holders kept, largest first.
    pub max_holders: usize,
    /// Holders whose accumulation class is computed.
    pub accumulate: usize,
}

impl Default for HolderOptions {
    fn default() -> Self {
        Self {
            min_retained: 1024,
            max_holders: 200,
            accumulate: 16,
        }
    }
}

/// The holders of one snapshot with its totals.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HolderSummary {
    pub nodes: u64,
    pub edges: u64,
    pub reachable: u64,
    /// Bytes of the reachable heap.
    pub total_bytes: u64,
    /// Largest object id: ids only grow within one process.
    pub max_id: u32,
    pub holders: Vec<Holder>,
    pub notes: Vec<String>,
}

fn clip(s: &str) -> String {
    if s.chars().count() <= NAME_MAX {
        s.to_owned()
    } else {
        s.chars().take(NAME_MAX).collect()
    }
}

/// Lookups the classification shares.
struct Index {
    /// A closure that uses each context (`u32::MAX` for none).
    ctx_closure: Vec<u32>,
    /// Script nodes of the `SharedFunctionInfo`s of each function name.
    sfi_scripts: HashMap<String, Vec<u32>>,
}

impl Snapshot {
    fn edge_to(&self, n: usize, kind: Option<u32>, name: &str) -> Option<usize> {
        let kind = kind?;
        self.edges_of(n)
            .find(|&e| self.edge_type(e) == kind && self.edge_name(e) == Some(name))
            .map(|e| self.edge_target(e))
    }

    fn internal(&self, n: usize, name: &str) -> Option<usize> {
        self.edge_to(n, self.kinds.e_internal, name)
    }

    fn property(&self, n: usize, name: &str) -> Option<usize> {
        self.edge_to(n, self.kinds.e_property, name)
    }

    fn is_closure(&self, n: usize) -> bool {
        Some(self.node_type(n)) == self.kinds.n_closure
    }

    fn is_context(&self, n: usize) -> bool {
        let name = self.node_name(n);
        name.starts_with("system / ")
            && name.ends_with("Context")
            && !name.ends_with("NativeContext")
    }

    /// The script node of a function: closure, `shared`, `script`.
    fn script_node_of_closure(&self, c: usize) -> Option<usize> {
        let sfi = self.internal(c, "shared")?;
        self.internal(sfi, "script")
    }

    fn script_name(&self, script: usize) -> Option<String> {
        let n = self.node_name(script);
        (!n.is_empty()).then(|| super::url_to_path(n))
    }

    fn build_index(&self) -> Index {
        let n = self.node_count();
        let mut ctx_closure = vec![NONE; n];
        let mut sfi_scripts: HashMap<String, Vec<u32>> = HashMap::new();
        for c in 0..n {
            let t = Some(self.node_type(c));
            if t == self.kinds.n_closure {
                if let Some(ctx) = self.internal(c, "context")
                    && ctx_closure[ctx] == NONE
                {
                    ctx_closure[ctx] = c as u32;
                }
            } else if t == self.kinds.n_code
                && let Some(s) = self.internal(c, "script")
            {
                let v = sfi_scripts.entry(self.node_name(c).to_owned()).or_default();
                if v.len() < 8 && !v.contains(&(s as u32)) {
                    v.push(s as u32);
                }
            }
        }
        Index {
            ctx_closure,
            sfi_scripts,
        }
    }

    fn script_of_context(&self, ix: &Index, ctx: usize) -> Option<usize> {
        let c = ix.ctx_closure[ctx];
        (c != NONE).then(|| self.script_node_of_closure(c as usize))?
    }

    /// True for the context of a module or script: its parent is the native context.
    fn is_top_context(&self, ctx: usize) -> bool {
        match self.internal(ctx, "previous") {
            None => false,
            Some(p) => {
                let n = self.node_name(p);
                n.ends_with("NativeContext") || n.ends_with("ScriptContext")
            }
        }
    }

    /// The name of the function a context belongs to: the last name of its scope info
    /// when a function of that name is declared in the same script.
    fn function_of_context(&self, ix: &Index, ctx: usize, script: Option<usize>) -> Option<String> {
        let scope = self.internal(ctx, "scope_info")?;
        let mut last: Option<usize> = None;
        for e in self.edges_of(scope) {
            if self.edge_type(e) != self.kinds.e_hidden.unwrap_or(NONE) {
                continue;
            }
            let t = self.edge_target(e);
            if Some(self.node_type(t)) == self.kinds.n_string {
                last = Some(t);
            } else {
                break;
            }
        }
        let name = self.node_name(last?);
        let scripts = ix.sfi_scripts.get(name)?;
        let ok = match script {
            Some(s) => scripts.contains(&(s as u32)),
            None => false,
        };
        ok.then(|| name.to_owned())
    }

    /// The script of the class an object was made by: `__proto__.constructor`.
    fn class_script(&self, obj: usize) -> Option<usize> {
        let proto = self.property(obj, "__proto__")?;
        let ctor = self.property(proto, "constructor")?;
        if !self.is_closure(ctor) {
            return None;
        }
        self.script_node_of_closure(ctor)
    }

    fn path_hops(&self, bfs: &Bfs, u: usize, e: usize) -> Vec<Hop> {
        let hop = |from: usize, edge: usize| Hop {
            node: clip(self.node_name(from)),
            edge: self.edge_name(edge).map_or_else(
                || format!("[{}]", self.edge_index(edge)),
                |s| format!(".{}", clip(s)),
            ),
        };
        let mut hops: Vec<Hop> = bfs
            .path_to(u)
            .into_iter()
            .map(|(from, edge)| hop(from, edge))
            .collect();
        hops.push(hop(u, e));
        if hops.len() > PATH_HOPS {
            let tail = hops.split_off(hops.len() - (PATH_HOPS - 2));
            hops.truncate(2);
            hops.push(Hop {
                node: String::new(),
                edge: "...".to_owned(),
            });
            hops.extend(tail);
        }
        hops
    }

    /// Finds the holders of this snapshot. `dom` and `bfs` come from the same snapshot.
    pub fn holders(&self, dom: &Dominators, bfs: &Bfs, opts: &HolderOptions) -> HolderSummary {
        let ix = self.build_index();
        #[derive(Default)]
        struct Acc {
            retained: u64,
            count: u64,
            instances: u64,
            best: Option<(u64, usize, usize, usize)>,
        }
        type Key = (
            HolderKind,
            Option<&'static str>,
            Option<u32>,
            String,
            String,
        );
        let mut agg: BTreeMap<Key, Acc> = BTreeMap::new();
        let near_root = |u: usize| bfs.path_to(u).len() <= 3;
        let mut class_cache: Option<(usize, Option<usize>)> = None;
        for &ui in dom.order() {
            let u = ui as usize;
            let uname = self.node_name(u);
            let is_ctx = self.is_context(u);
            if uname.starts_with("system /") && !is_ctx {
                continue;
            }
            for e in self.edges_of(u) {
                let t = self.edge_type(e);
                let is_context_edge = Some(t) == self.kinds.e_context;
                if !is_context_edge && Some(t) != self.kinds.e_property {
                    continue;
                }
                let v = self.edge_target(e);
                if v == u || dom.idom(v) != Some(u) || dom.retained(v) < opts.min_retained {
                    continue;
                }
                let ename = self.edge_name(e).unwrap_or_default();
                if matches!(ename, "__proto__" | "constructor" | "prototype") {
                    continue;
                }
                // A link of its own kind is part of the structure, not what holds it.
                if !is_ctx && uname == self.node_name(v) {
                    continue;
                }
                let (kind, registry, script, owner, name): (
                    HolderKind,
                    Option<&'static str>,
                    Option<usize>,
                    String,
                    String,
                ) = if is_ctx {
                    let script = self.script_of_context(&ix, u);
                    if self.is_top_context(u) {
                        (
                            HolderKind::Static,
                            None,
                            script,
                            String::new(),
                            ename.to_owned(),
                        )
                    } else {
                        let f = self.function_of_context(&ix, u, script).unwrap_or_default();
                        (HolderKind::Closure, None, script, f, ename.to_owned())
                    }
                } else if uname == "global" && near_root(u) {
                    (
                        HolderKind::Static,
                        None,
                        // A global function names the script that declares it.
                        self.is_closure(v)
                            .then(|| self.script_node_of_closure(v))
                            .flatten(),
                        "global".to_owned(),
                        ename.to_owned(),
                    )
                } else if matches!(uname, "Timeout" | "Immediate")
                    && matches!(ename, "_onTimeout" | "_onImmediate")
                    && self.is_closure(v)
                {
                    let script = self.script_node_of_closure(v);
                    let f = self
                        .internal(v, "context")
                        .and_then(|c| self.function_of_context(&ix, c, script))
                        .unwrap_or_default();
                    (
                        HolderKind::Registration,
                        Some("timers"),
                        script,
                        f,
                        self.node_name(v).to_owned(),
                    )
                } else if ename == "_events" {
                    // The table itself is not named: its entries are.
                    continue;
                } else if bfs
                    .retainer(u)
                    .is_some_and(|(_, re)| self.edge_name(re) == Some("_events"))
                {
                    let emitter = bfs
                        .retainer(u)
                        .map(|(from, _)| self.node_name(from).to_owned())
                        .unwrap_or_default();
                    let listener = if self.is_closure(v) {
                        Some(v)
                    } else {
                        self.edges_of(v)
                            .map(|x| self.edge_target(x))
                            .find(|&x| self.is_closure(x))
                    };
                    let script = listener.and_then(|c| self.script_node_of_closure(c));
                    (
                        HolderKind::Registration,
                        Some("events"),
                        script,
                        emitter,
                        ename.to_owned(),
                    )
                } else {
                    // The class of `u` is looked up once, however many of its edges are holders.
                    let script = match uname {
                        "Object" | "Array" | "Map" | "Set" | "WeakMap" | "WeakSet" => None,
                        _ => match class_cache {
                            Some((who, s)) if who == u => s,
                            _ => {
                                let s = self.class_script(u);
                                class_cache = Some((u, s));
                                s
                            }
                        },
                    };
                    (
                        HolderKind::Instance,
                        None,
                        script,
                        uname.to_owned(),
                        ename.to_owned(),
                    )
                };
                let key: Key = (kind, registry, script.map(|s| s as u32), owner, name);
                let a = agg.entry(key).or_default();
                a.retained = a.retained.saturating_add(dom.retained(v));
                a.count = a.count.saturating_add(dom.dominated(v));
                a.instances += 1;
                if a.best.is_none_or(|b| dom.retained(v) > b.0) {
                    a.best = Some((dom.retained(v), u, e, v));
                }
            }
        }
        let mut ranked: Vec<(Key, Acc)> = agg.into_iter().collect();
        ranked.sort_by(|a, b| b.1.retained.cmp(&a.1.retained).then_with(|| a.0.cmp(&b.0)));
        ranked.truncate(opts.max_holders);
        let (first, kids) = dom.children();
        let mut holders = Vec::with_capacity(ranked.len());
        for (i, ((kind, registry, script, owner, name), a)) in ranked.into_iter().enumerate() {
            let (path, accumulation) = match a.best {
                Some((_, u, e, v)) => (
                    self.path_hops(bfs, u, e),
                    (i < opts.accumulate)
                        .then(|| self.accumulation(v, &first, &kids))
                        .flatten(),
                ),
                None => (Vec::new(), None),
            };
            holders.push(Holder {
                kind,
                registry: registry.map(str::to_owned),
                name: clip(&name),
                owner: clip(&owner),
                script: script.and_then(|s| self.script_name(s as usize)),
                retained: a.retained,
                count: a.count,
                instances: a.instances,
                path,
                accumulation,
            });
        }
        HolderSummary {
            nodes: self.node_count() as u64,
            edges: self.edge_count() as u64,
            reachable: dom.reachable_count() as u64,
            total_bytes: dom.total(),
            max_id: self.max_id(),
            holders,
            notes: vec![
                "retained bytes follow the dominator tree: a holder counts what only it keeps alive".to_owned(),
                "weak edges are skipped; string contents are never read into the summary".to_owned(),
            ],
        }
    }

    /// The most common class among the objects `v` dominates.
    fn accumulation(&self, v: usize, first: &[u32], kids: &[u32]) -> Option<Accumulation> {
        let mut by: BTreeMap<String, (u64, u64)> = BTreeMap::new();
        let mut stack = vec![v as u32];
        let mut seen = 0usize;
        while let Some(x) = stack.pop() {
            let x = x as usize;
            seen += 1;
            if seen > 2_000_000 {
                break;
            }
            let e = by.entry(self.class_name(x)).or_default();
            e.0 += 1;
            e.1 = e.1.saturating_add(self.self_size(x));
            stack.extend_from_slice(&kids[first[x] as usize..first[x + 1] as usize]);
        }
        // Named classes first; the generic rows only when nothing else is there.
        let pick = |named: bool| {
            by.iter()
                .filter(|(c, _)| !named || !c.starts_with('('))
                .max_by_key(|(c, (n, b))| (*b, *n, std::cmp::Reverse((*c).clone())))
                .map(|(c, (n, b))| Accumulation {
                    class: c.clone(),
                    count: *n,
                    bytes: *b,
                })
        };
        pick(true).or_else(|| pick(false))
    }
}

#[cfg(test)]
mod tests {
    use super::super::snapshot::testing::*;
    use super::*;
    use crate::node::Snapshot;

    fn load(nodes: &[N<'_>]) -> Snapshot {
        let t = build(nodes);
        Snapshot::read(t.as_bytes(), t.len() as u64).unwrap()
    }

    fn holders_of(s: &Snapshot) -> HolderSummary {
        let dom = s.dominators(None).unwrap();
        let bfs = s.bfs();
        s.holders(
            &dom,
            &bfs,
            &HolderOptions {
                min_retained: 10,
                ..HolderOptions::default()
            },
        )
    }

    /// A module context holding `shared`, as the closures of a CommonJS module see it.
    fn module(shared_chain: usize) -> Vec<N<'static>> {
        let mut v: Vec<N<'static>> = vec![
            // 0 root
            ("synthetic", "", 0, vec![("element", "0", 1)]),
            // 1 (GC roots) -> context
            ("synthetic", "(GC roots)", 0, vec![("element", "0", 2)]),
            // 2 module context with `shared`, previous native context
            (
                "object",
                "system / Context",
                32,
                vec![("context", "shared", 6), ("internal", "previous", 3)],
            ),
            ("hidden", "system / NativeContext", 100, vec![]),
            // 4 closure using the context, 5 its shared function info and script
            (
                "closure",
                "sharedScheduler",
                24,
                vec![("internal", "context", 2), ("internal", "shared", 4 + 1)],
            ),
            (
                "code",
                "sharedScheduler",
                16,
                vec![("internal", "script", 5 + 1)],
            ),
            // 6 the scheduler instance holds a chain of records
            (
                "object",
                "RetryScheduler",
                48,
                vec![("property", "latest", 8)],
            ),
            ("code", "(unused)", 0, vec![]),
        ];
        // 7 is a script node, referenced by the sfi at index 5 -> 6? keep indexes simple:
        v[5] = (
            "code",
            "sharedScheduler",
            16,
            vec![("internal", "script", 7)],
        );
        v[7] = ("code", "/work/src/scheduler.js", 8, vec![]);
        // The root also reaches the closure so it is retained.
        v[1] = (
            "synthetic",
            "(GC roots)",
            0,
            vec![("element", "0", 2), ("element", "1", 4)],
        );
        // 8.. records chained by `previous`
        for i in 0..shared_chain {
            let at = 8 + i;
            let edges = if i + 1 < shared_chain {
                vec![("property", "previous", at + 1)]
            } else {
                vec![]
            };
            v.push(("object", "Object", 40, edges));
        }
        v
    }

    #[test]
    fn a_module_variable_is_a_static_holder_named_by_its_script() {
        let s = load(&module(6));
        let h = holders_of(&s);
        let top = &h.holders[0];
        assert_eq!(top.kind, HolderKind::Static, "{:?}", h.holders);
        assert_eq!(top.name, "shared");
        assert_eq!(top.script.as_deref(), Some("/work/src/scheduler.js"));
        assert_eq!(top.retained, 48 + 6 * 40);
        // The chain links (Object -> Object) are not holders; `latest` is the field.
        assert!(
            h.holders
                .iter()
                .any(|x| x.kind == HolderKind::Instance && x.name == "latest")
        );
        assert!(h.holders.iter().all(|x| x.name != "previous"));
        assert!(
            top.path.iter().any(|p| p.edge == ".shared"),
            "{:?}",
            top.path
        );
        assert_eq!(
            top.accumulation.as_ref().map(|a| a.class.as_str()),
            Some("Object")
        );
    }

    #[test]
    fn a_timer_callback_is_a_registration_and_a_function_context_is_a_closure() {
        let s = load(&[
            ("synthetic", "", 0, vec![("element", "0", 1)]),
            ("synthetic", "(GC roots)", 0, vec![("element", "0", 2)]),
            (
                "object",
                "Timeout",
                100,
                vec![("property", "_onTimeout", 3)],
            ),
            ("closure", "beat", 56, vec![("internal", "context", 4)]),
            (
                "object",
                "system / Context",
                48,
                vec![("context", "session", 5)],
            ),
            ("object", "Session", 120, vec![("property", "recent", 6)]),
            ("object", "Array", 64, vec![]),
        ]);
        let h = holders_of(&s);
        let reg = h
            .holders
            .iter()
            .find(|x| x.kind == HolderKind::Registration)
            .unwrap();
        assert_eq!(reg.registry.as_deref(), Some("timers"));
        assert_eq!(reg.name, "beat");
        assert_eq!(reg.retained, 56 + 48 + 120 + 64);
        let clo = h
            .holders
            .iter()
            .find(|x| x.kind == HolderKind::Closure)
            .unwrap();
        assert_eq!(clo.name, "session");
    }
}
