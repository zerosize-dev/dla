//! The names a Python file binds with `import` statements, so that a call `core.f(..)` or
//! `f(..)` can be tied to the module it reaches or shown to reach another one.

use std::collections::HashMap;

/// What a name bound by an import statement stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Bind {
    /// `import a.b as c` binds `c` to `a.b`; `import a.b` binds `a` to `a`.
    Module(String),
    /// `from m import n [as k]` binds `k` to `n` of the module `m` (leading dots for a
    /// relative import): a submodule, or a name the module defines.
    From(String, String),
}

/// The bindings of one file.
#[derive(Debug, Clone, Default)]
pub(crate) struct Imports {
    pub binds: HashMap<String, Bind>,
    /// A `from m import *` is present: any unbound name may come from it.
    pub star: bool,
}

impl Imports {
    /// Reads one `import` or `from .. import ..` statement, as written in the source.
    pub fn add_statement(&mut self, stmt: &str) {
        let flat: String = stmt
            .lines()
            .map(|l| l.split('#').next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join(" ")
            .replace(['(', ')', '\\'], " ");
        let flat = flat.trim();
        if let Some(rest) = flat.strip_prefix("import ") {
            for part in rest.split(',') {
                let mut w = part.split_whitespace();
                let Some(module) = w.next() else { continue };
                match (w.next(), w.next()) {
                    (Some("as"), Some(alias)) => {
                        self.binds
                            .insert(alias.to_owned(), Bind::Module(module.to_owned()));
                    }
                    _ => {
                        let root = module.split('.').next().unwrap_or(module);
                        self.binds
                            .insert(root.to_owned(), Bind::Module(root.to_owned()));
                    }
                }
            }
        } else if let Some(rest) = flat.strip_prefix("from ")
            && let Some((module, names)) = rest.split_once(" import ")
        {
            let module = module.trim();
            if names.trim() == "*" {
                self.star = true;
                return;
            }
            for part in names.split(',') {
                let mut w = part.split_whitespace();
                let Some(name) = w.next() else { continue };
                let bound = match (w.next(), w.next()) {
                    (Some("as"), Some(alias)) => alias,
                    _ => name,
                };
                self.binds.insert(
                    bound.to_owned(),
                    Bind::From(module.to_owned(), name.to_owned()),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_forms_of_an_import_statement_are_read() {
        let mut i = Imports::default();
        i.add_statement("import a.b.c, d as e");
        i.add_statement("from . import core");
        i.add_statement("from pkg import (core as c,\n    other)");
        i.add_statement("from .util import *");
        assert_eq!(i.binds["a"], Bind::Module("a".into()));
        assert_eq!(i.binds["e"], Bind::Module("d".into()));
        assert_eq!(i.binds["core"], Bind::From(".".into(), "core".into()));
        assert_eq!(i.binds["c"], Bind::From("pkg".into(), "core".into()));
        assert_eq!(i.binds["other"], Bind::From("pkg".into(), "other".into()));
        assert!(i.star);
    }
}
