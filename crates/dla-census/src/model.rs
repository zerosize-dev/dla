//! The census data model: raw holders found in source, before they become ledger
//! candidates.

use dla_core::finding::{Holder, Region};

use crate::parse::Lang;

/// Where heuristic writers and removers of a holder are looked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// Every source file of the runtime.
    Project,
    /// Files in one directory (Go package scope).
    Dir(String),
    /// The declaring file.
    File,
    /// A line range of the declaring file (a function body).
    Lines(u32, u32),
}

/// One holder as found in source.
#[derive(Debug, Clone)]
pub struct Raw {
    /// Declaration kind (`static-field`, `module-global`, `spawn`, ...).
    pub kind: String,
    /// Holder class.
    pub holder: Holder,
    /// Region.
    pub region: Region,
    /// Project-relative file.
    pub file: String,
    /// Declaration line.
    pub line: u32,
    /// Qualified owner.
    pub owner: String,
    /// The identifier to track for writers and removers.
    pub name: Option<String>,
    /// Where to look.
    pub scope: Scope,
    /// Only `x.name` / `x->name` accesses count (a field, not a variable).
    pub member_access: bool,
}

/// One parsed-or-readable source file.
pub struct SourceFile {
    /// Project-relative path with `/` separators.
    pub rel: String,
    /// Contents.
    pub text: String,
    /// Language.
    pub lang: Lang,
    /// A test file (holders in it are not enumerated).
    pub is_test: bool,
}

impl SourceFile {
    /// Dotted module path of the file (`a/b/c.py` gives `a.b.c`).
    pub fn module(&self) -> String {
        module_path(&self.rel)
    }
}

/// Dotted module path of a relative file path, without extension.
pub fn module_path(rel: &str) -> String {
    let stem = rel.rsplit_once('.').map_or(rel, |(s, _)| s);
    stem.trim_start_matches("./")
        .trim_end_matches("/__init__")
        .replace('/', ".")
}

impl Raw {
    /// A holder with no tracked name, whole-project scope and no member-access rule.
    pub fn new(
        kind: &str,
        holder: Holder,
        region: Region,
        file: &str,
        line: u32,
        owner: String,
    ) -> Self {
        Self {
            kind: kind.to_owned(),
            holder,
            region,
            file: file.to_owned(),
            line,
            owner,
            name: None,
            scope: Scope::File,
            member_access: false,
        }
    }

    /// Sets the tracked name and scope.
    pub fn tracking(mut self, name: &str, scope: Scope, member_access: bool) -> Self {
        self.name = Some(name.to_owned());
        self.scope = scope;
        self.member_access = member_access;
        self
    }
}

/// Cross-file facts gathered before the per-file pass.
#[derive(Debug, Default, Clone)]
pub struct ProjectInfo {
    /// Type names whose declaration holds a field of its own type (nodes of linked
    /// structures).
    pub selfref: std::collections::HashSet<String>,
    /// Rust: for each struct, the types it holds through `Rc`/`Arc` (without `Weak`),
    /// with whether the field is a collection of them.
    pub rc_edges: std::collections::HashMap<String, Vec<(String, bool)>>,
    /// Rust: structs that hold a `Weak` pointer to the named type.
    pub weak: std::collections::HashSet<(String, String)>,
    /// Owner types that are plausibly process-lifetime: named by a static or module
    /// global, constructed in `main`/`init`/startup code, self-recursive structures,
    /// or reached from another long-lived owner (see [`ProjectInfo::finish`]).
    pub longlived: std::collections::HashSet<String>,
    /// For each owner type, the type names mentioned by its fields.
    pub holds: std::collections::HashMap<String, Vec<String>>,
}

impl ProjectInfo {
    /// Closes `longlived` over `holds`: whatever a long-lived owner holds is reachable
    /// for as long as the owner is.
    pub fn finish(&mut self) {
        let named = self
            .holds
            .keys()
            .filter(|t| crate::vocab::longlived_type_name().is_match(t))
            .cloned()
            .collect::<Vec<_>>();
        self.longlived.extend(named);
        let mut work: Vec<String> = self.longlived.iter().cloned().collect();
        while let Some(t) = work.pop() {
            if let Some(next) = self.holds.get(&t) {
                for n in next {
                    if self.longlived.insert(n.clone()) {
                        work.push(n.clone());
                    }
                }
            }
        }
    }

    /// True when instances of `ty` are plausibly alive for the whole process: a
    /// project-derived fact or a role name such as `Registry`, `Cache` or `Server`.
    pub fn is_long_lived(&self, ty: &str) -> bool {
        self.longlived.contains(ty) || crate::vocab::longlived_type_name().is_match(ty)
    }

    /// Records that `owner` mentions the types named in `ty_text` and, when
    /// `owner` itself appears in it, that it is a recursive structure.
    pub fn note_fields(&mut self, owner: &str, ty_text: &str) {
        for t in crate::lang::caps_idents(ty_text) {
            if t == owner {
                self.longlived.insert(owner.to_owned());
            } else {
                self.holds.entry(owner.to_owned()).or_default().push(t);
            }
        }
    }
}
