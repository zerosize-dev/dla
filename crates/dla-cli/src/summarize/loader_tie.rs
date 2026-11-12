//! Ties a class loader group to the finding whose holder is that loader.
//!
//! `VM.classloader_stats` lists the classes each loader defined, by the loader's class
//! (`quotes.RuleLoader`). A loader held by a static keeps every class it defined, so the
//! group grows when the leak is classes that are never unloaded. The group ties to a
//! finding by type identity: the finding's holder is declared in the project's sources
//! with exactly that class as its own type. A prefix, a simple name from another package,
//! a JDK loader and a generic type never match. Several findings with that holder type
//! tie none (see [`super::tie::resolve`]).

use std::{collections::BTreeSet, fs, path::Path};

use dla_core::{FindingId, Ledger};

use super::tie::{ProjectClasses, TieReason, Who, own_type, qualify_type, site_is_jvm};

/// The loader class a `classloader` group key names, or `None` for a key that is not
/// one class (hidden classes of a loader, a loader without a class name).
pub fn loader_class(kind: &str, key: &str) -> Option<String> {
    if kind != "classloader" {
        return None;
    }
    let key = key.trim();
    let plain = !key.is_empty()
        && key.split('.').all(|p| {
            !p.is_empty()
                && p.chars()
                    .all(|c| c.is_alphanumeric() || matches!(c, '_' | '$'))
        })
        && key
            .rsplit('.')
            .next()
            .and_then(|l| l.chars().next())
            .is_some_and(|c| c.is_alphabetic());
    plain.then(|| key.replace('$', "."))
}

/// The findings whose holder is declared with the type `loader` (a dotted qualified
/// class name), each with the type tie that says why. `classes` is the project's declared
/// classes, scanned once by the caller. Empty on an artifact run (there are no sources
/// to read).
pub(super) fn findings_for_loader(
    ledger: &Ledger,
    who: &Who,
    project: &Path,
    classes: &ProjectClasses,
    artifact_run: bool,
    loader: &str,
) -> Vec<(FindingId, TieReason)> {
    if artifact_run {
        return Vec::new();
    }
    if !classes.declares(loader) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for f in ledger.findings().filter(|f| who.sees(f)) {
        let (Some(site), Some(owner)) = (f.site.as_deref(), f.owner.as_deref()) else {
            continue;
        };
        if !site_is_jvm(site) {
            continue;
        }
        let Some((file, line)) = site
            .rsplit_once(':')
            .and_then(|(file, line)| Some((file, line.parse::<u32>().ok()?)))
        else {
            continue;
        };
        let Ok(text) = fs::read_to_string(project.join(file)) else {
            continue;
        };
        let field = owner.rsplit('.').next().unwrap_or(owner);
        let stem = Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let Some(written) = own_type(&text, line, field) else {
            continue;
        };
        let exact = qualify_type(&text, stem, &written)
            .iter()
            .any(|n| n.replace('$', ".") == loader);
        if exact && seen.insert(f.id.clone()) {
            out.push((f.id.clone(), TieReason::ty(loader)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_plain_class_name_is_a_loader_class() {
        assert_eq!(
            loader_class("classloader", "quotes.RuleLoader").as_deref(),
            Some("quotes.RuleLoader")
        );
        assert_eq!(
            loader_class("classloader", "quotes.Outer$Loader").as_deref(),
            Some("quotes.Outer.Loader")
        );
        assert_eq!(
            loader_class("classloader", "quotes.RuleLoader + hidden classes"),
            None
        );
        assert_eq!(loader_class("classloader", "<unknown>"), None);
        assert_eq!(loader_class("thread", "quotes.RuleLoader"), None);
        assert_eq!(loader_class("classloader", ""), None);
    }
}
