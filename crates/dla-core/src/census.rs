//! One holder enumerated by `dla census`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::finding::{Holder, Region};

/// A process-lifetime holder: something that can keep objects, threads or handles
/// alive for as long as the process runs.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct CensusEntry {
    /// What kind of declaration it is (for example `static-field`, `module-global`,
    /// `singleton-field`, `registry`, `spawn`, `handle`).
    pub kind: String,
    /// Holder class in the ledger vocabulary.
    pub holder: Holder,
    /// Memory region.
    pub region: Region,
    /// Declaration, spawn or acquire site, `file:line` (project-relative).
    pub site: String,
    /// Qualified name of the owning field or container.
    pub owner: String,
    /// Heuristic `file:line` sites that add to it.
    #[serde(default)]
    pub writers: Vec<String>,
    /// Heuristic `file:line` sites that remove from it.
    #[serde(default)]
    pub removers: Vec<String>,
    /// What the syntax tree says of the holder and of each writer and remover site (see
    /// [`HolderFacts`]); empty for Rust and C holders and in census files of older versions.
    #[serde(flatten)]
    pub facts: HolderFacts,
}

/// What kind of thing a holder is.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerClass {
    /// A real container: map, set, list, slice, queue, deque, cache type, `sync.Map`, a
    /// channel used as a queue.
    Container,
    /// Registry-like: named or typed as a registry, listener table or handler table.
    Registry,
    /// Not a container: a scalar, a pointer or struct reassigned as a whole, a logger, a
    /// function value, a type variable, a pool type, a literal or immutable table.
    NotContainer,
}

impl ContainerClass {
    /// The name used in the ledger and in `dla ledger list`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Container => "container",
            Self::Registry => "registry",
            Self::NotContainer => "not_container",
        }
    }
}

/// How a holder's size is limited.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bound {
    /// No limit declared.
    None,
    /// Weak keys: an entry goes when its key is collected.
    WeakKeys,
    /// Weak values: an entry goes when its value is collected.
    WeakValues,
    /// A size limit: LRU with a size, ring, bounded channel, `maxsize=` given.
    Bounded,
    /// A cache decorator without a size (`lru_cache(None)`, `functools.cache`).
    UnboundedCache,
}

/// How long a holder lives.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Lifetime {
    /// Static, package or module variable, class variable.
    Process,
    /// Field of an instance.
    Object,
    /// Created per call: local, closure, timer or task or handle creation site.
    Unit,
}

/// How instances of a holder's owner type come to exist.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OwnerCreation {
    /// Created inside functions that can run many times.
    PerCall,
    /// Created only at start-up (initialisers and what only they call).
    StartUp,
}

/// Where a registration's receiver comes from.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Receiver {
    /// A local of the enclosing function: it dies with the call.
    Local,
    /// A parameter or a field: it outlives the call.
    Outer,
}

/// What a writer or remover site does to its holder.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SiteKind {
    /// Keyed write, put, computeIfAbsent, setdefault, Store, add, append, push, offer,
    /// register.
    Insert,
    /// The variable itself is assigned.
    Reassign,
    /// Assigned a fresh empty container, or cleared.
    Reset,
    /// Remove, delete, pop, evict.
    Delete,
}

impl SiteKind {
    /// True for a site that adds entries (`insert`).
    pub fn adds(self) -> bool {
        self == Self::Insert
    }
}

/// One writer or remover site with what the syntax tree says of it.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct SiteFact {
    /// `file:line` (project-relative); the same string as in `writers` or `removers`.
    pub site: String,
    /// What the site does.
    pub kind: SiteKind,
    /// 0 when the operation is on the holder itself, 1 or more when it is on an element of
    /// it (`delete(s.updates[k], x)` and `s.updates[k][x] = v` are depth 1).
    #[serde(default)]
    pub depth: u8,
    /// The enclosing function or method.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    /// The enclosing function is an initialiser: Go `init` or a package-level `var =`, a
    /// constructor or factory of the owner type, a static initialiser, Python module top
    /// level or `__init__`, JS module top level or constructor.
    #[serde(default, skip_serializing_if = "is_false")]
    pub init: bool,
    /// The site runs only at start-up: it is in an initialiser, or every caller of its
    /// function (up to three calls up) is one. Read where the callers can be found.
    #[serde(default, skip_serializing_if = "is_false")]
    pub startup: bool,
    /// When `startup` came through callers that are not initialisers themselves, the functions
    /// on the way (the wrappers between the writer's function and the start-up statements that
    /// call them), nearest first. Empty for a direct start-up call. A driver that calls one of
    /// them calls the registration function through it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub startup_via: Vec<String>,
    /// The inserted key or value derives from a parameter of the enclosing function.
    #[serde(default, skip_serializing_if = "is_false")]
    pub param: bool,
    /// The key is a literal or a named constant.
    #[serde(default, rename = "const", skip_serializing_if = "is_false")]
    pub constant: bool,
    /// The enclosing function is part of the public surface.
    #[serde(default, skip_serializing_if = "is_false")]
    pub exported: bool,
    /// The site lies in a thread-local initialiser (a function a thread-local runs for each
    /// thread that reads it: a `withInitial` lambda or method reference, an `initialValue`
    /// override, the `__init__` of a `threading.local` subclass), so the holder gets an entry
    /// for every thread that reads the thread-local, and again whenever the thread-local was
    /// cleared.
    #[serde(default, skip_serializing_if = "is_false")]
    pub thread_init: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// [`HolderFacts::thread_keyed`] for a finding that may have no facts: a thread-local holder
/// is thread-keyed whatever the facts say.
pub fn thread_keyed(holder: Option<Holder>, facts: Option<&HolderFacts>) -> bool {
    facts.map_or(holder == Some(Holder::ThreadLocal), |f| {
        f.thread_keyed(holder)
    })
}

/// What the census knows of a holder beyond its name and its sites. Every field is optional:
/// a census file or ledger of an older version, a Rust or C holder, or a hand-written
/// annotation carries none of it, which the ranking reads as neutral.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq, Default)]
pub struct HolderFacts {
    /// Container, registry-like, or not a container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<ContainerClass>,
    /// How its size is limited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bound: Option<Bound>,
    /// Process, object or unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<Lifetime>,
    /// The holder cannot be named outside its file, class or package.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    /// The holder is a literal or immutable table that is only read.
    #[serde(default, skip_serializing_if = "is_false")]
    pub literal: bool,
    /// The holder is a pool type (`sync.Pool`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub pool: bool,
    /// Its key type is bounded by the program (a class, an enum, a type descriptor).
    #[serde(default, skip_serializing_if = "is_false")]
    pub key_bounded: bool,
    /// The entry is the creation site of a unit (timer, task, goroutine, thread, handle):
    /// the creation is the growth.
    #[serde(default, skip_serializing_if = "is_false")]
    pub unit: bool,
    /// Registrations: where the receiver comes from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Receiver>,
    /// How instances of the owner type are created, where the census can tell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_creation: Option<OwnerCreation>,
    /// The owner class (the owner up to its last segment) also owns a holder of kind
    /// `thread-local` in the census: state of a class that keeps things per thread.
    #[serde(default, skip_serializing_if = "is_false")]
    pub thread_class: bool,
    /// The writer sites, one per entry of `writers`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub writer_sites: Vec<SiteFact>,
    /// The remover sites, one per entry of `removers`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remover_sites: Vec<SiteFact>,
}

impl HolderFacts {
    /// True when an insert at the holder itself takes its key from a parameter of the
    /// enclosing function (not a constant, not at start-up only): the object a caller passes
    /// is the key, so the holder grows only when each unit passes one it has not seen.
    pub fn key_from_caller(&self) -> bool {
        self.writer_sites
            .iter()
            .any(|s| s.kind.adds() && s.depth == 0 && s.param && !s.constant && !s.startup)
    }

    /// True when the holder is kept per thread, so that it grows with the number of distinct
    /// threads or thread lifetimes the code sees: it is a thread-local, or its owner class also
    /// owns one, or an adding writer site lies in a thread-local initialiser. The census does
    /// not record key types, so a map keyed by a thread or a thread id is not recognised.
    pub fn thread_keyed(&self, holder: Option<Holder>) -> bool {
        holder == Some(Holder::ThreadLocal)
            || self.thread_class
            || self
                .writer_sites
                .iter()
                .any(|s| s.kind.adds() && s.thread_init)
    }

    /// True when nothing is recorded.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Adds what `other` records: scalars of `other` win when set, site lists gain the sites
    /// not yet listed.
    pub fn merge(&mut self, other: &Self) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if other.$f.is_some() { self.$f = other.$f; }
            )*};
        }
        take!(
            container,
            bound,
            lifetime,
            private,
            receiver,
            owner_creation
        );
        self.literal |= other.literal;
        self.pool |= other.pool;
        self.key_bounded |= other.key_bounded;
        self.unit |= other.unit;
        self.thread_class |= other.thread_class;
        for (mine, theirs) in [
            (&mut self.writer_sites, &other.writer_sites),
            (&mut self.remover_sites, &other.remover_sites),
        ] {
            for s in theirs {
                if !mine.iter().any(|m| m.site == s.site && m.kind == s.kind) {
                    mine.push(s.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(kind: SiteKind, thread_init: bool) -> SiteFact {
        SiteFact {
            site: "a/B.java:3".into(),
            kind,
            depth: 0,
            function: Some("init".into()),
            init: false,
            startup: false,
            startup_via: Vec::new(),
            param: false,
            constant: false,
            exported: false,
            thread_init,
        }
    }

    #[test]
    fn a_thread_local_holder_is_thread_keyed_alone() {
        let facts = HolderFacts::default();
        assert!(facts.thread_keyed(Some(Holder::ThreadLocal)));
        assert!(!facts.thread_keyed(Some(Holder::ContainerField)));
        assert!(!facts.thread_keyed(None));
    }

    #[test]
    fn an_owner_class_with_a_thread_local_makes_its_containers_thread_keyed_alone() {
        let facts = HolderFacts {
            thread_class: true,
            ..HolderFacts::default()
        };
        assert!(facts.thread_keyed(Some(Holder::ContainerField)));
    }

    #[test]
    fn an_adding_writer_in_a_thread_local_initialiser_is_thread_keyed_alone() {
        let mut facts = HolderFacts {
            writer_sites: vec![site(SiteKind::Insert, true)],
            ..HolderFacts::default()
        };
        assert!(facts.thread_keyed(Some(Holder::ContainerField)));
        // A remover in an initialiser does not add, and an adder elsewhere is not one.
        facts.writer_sites = vec![site(SiteKind::Delete, true), site(SiteKind::Insert, false)];
        assert!(!facts.thread_keyed(Some(Holder::ContainerField)));
    }

    #[test]
    fn a_container_of_a_class_without_a_thread_local_is_not_thread_keyed() {
        let facts = HolderFacts {
            container: Some(ContainerClass::Container),
            writer_sites: vec![site(SiteKind::Insert, false)],
            ..HolderFacts::default()
        };
        assert!(!facts.thread_keyed(Some(Holder::ContainerField)));
    }

    #[test]
    fn merging_keeps_the_thread_class_fact() {
        let mut a = HolderFacts::default();
        a.merge(&HolderFacts {
            thread_class: true,
            ..HolderFacts::default()
        });
        assert!(a.thread_class);
    }
}
