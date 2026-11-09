//! Ties: which finding a growth column belongs to.
//!
//! The ledger refuses to confirm a leak finding unless a passing growth column it cites
//! is tied to it (by `finding`, by equal `site`, by `candidates`, or by a column named
//! after the owner). Adapters that produce growth therefore look for the finding first:
//! the candidate they add, or the existing finding with the same owner or site. A tie
//! is made only for a unique match, and never by nearness of strings.

use std::{collections::BTreeSet, fs, path::Path};

use dla_core::{
    FindingId, Ledger,
    attribution::{normalize, owners_match},
};
use serde_json::Value;

/// The finding whose owner is `owner`: an equal owner first, then the only one that
/// matches by trailing dotted components (`Audit.trail` and `shop.codec.Audit.trail`).
/// Several matches give none.
pub fn finding_by_owner(ledger: &Ledger, owner: &str) -> Option<FindingId> {
    let exact: Vec<&FindingId> = ledger
        .findings()
        .filter(|f| f.owner.as_deref() == Some(owner))
        .map(|f| &f.id)
        .collect();
    if let [one] = exact.as_slice() {
        return Some((*one).clone());
    }
    if !exact.is_empty() {
        return exact.first().map(|f| (*f).clone());
    }
    let want = normalize(owner)?;
    let loose: Vec<&FindingId> = ledger
        .findings()
        .filter(|f| {
            f.owner
                .as_deref()
                .is_some_and(|o| normalize(o).as_deref() == Some(want.as_str()))
        })
        .map(|f| &f.id)
        .collect();
    if let [one] = loose.as_slice() {
        return Some((*one).clone());
    }
    let suffix: Vec<&FindingId> = ledger
        .findings()
        .filter(|f| f.owner.as_deref().is_some_and(|o| owners_match(o, &want)))
        .map(|f| &f.id)
        .collect();
    match suffix.as_slice() {
        [one] => Some((*one).clone()),
        _ => None,
    }
}

/// The findings at `site` (`file:line`): the finding's own site, or one of its recorded
/// writers or removers.
pub fn findings_at_site(ledger: &Ledger, site: &str) -> Vec<FindingId> {
    ledger
        .findings()
        .filter(|f| {
            f.site.as_deref() == Some(site)
                || ledger
                    .annotation(&f.id)
                    .is_some_and(|a| a.writers.iter().chain(&a.removers).any(|w| w == site))
        })
        .map(|f| f.id.clone())
        .collect()
}

/// The finding for an owner and/or a site: by owner, else the only finding at the site.
pub fn finding_for(ledger: &Ledger, owner: Option<&str>, site: Option<&str>) -> Option<FindingId> {
    if let Some(f) = owner.and_then(|o| finding_by_owner(ledger, o)) {
        return Some(f);
    }
    let at = findings_at_site(ledger, site?);
    match at.as_slice() {
        [one] => Some(one.clone()),
        _ => None,
    }
}

/// What the heap-graph summaries of a run say about where each holder's memory piles up:
/// per owner, the accumulation class and the classes on the heavy retention path.
#[derive(Debug, Default)]
pub struct AccIndex {
    rows: Vec<(String, BTreeSet<String>)>,
}

fn path_class(entry: &str) -> Option<&str> {
    if entry.starts_with("root ") {
        return None;
    }
    let c = entry.rsplit(" -> ").next().unwrap_or(entry).trim();
    (!c.is_empty() && !c.contains(' ')).then_some(c)
}

impl AccIndex {
    /// Reads `summaries/*.json` of the run directory. Files that are not heap summaries
    /// are skipped.
    pub fn load(run_dir: &Path) -> Self {
        let mut idx = Self::default();
        let Ok(rd) = fs::read_dir(run_dir.join("summaries")) else {
            return idx;
        };
        let mut files: Vec<_> = rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        files.sort();
        for f in files {
            let Ok(text) = fs::read_to_string(&f) else {
                continue;
            };
            let Ok(doc) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            let Some(holders) = doc.get("holders").and_then(Value::as_array) else {
                continue;
            };
            for h in holders {
                let kind = h.get("kind").and_then(Value::as_str).unwrap_or_default();
                let key = h.get("key").and_then(Value::as_str).unwrap_or_default();
                let owner = match (kind, h.get("field").and_then(Value::as_str)) {
                    ("static", Some(field)) => {
                        let class = h
                            .get("owner_class")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        format!("{class}.{field}")
                    }
                    _ => key.to_owned(),
                };
                let mut classes = BTreeSet::new();
                if let Some(c) = h
                    .get("accumulation")
                    .and_then(|a| a.get("class"))
                    .and_then(Value::as_str)
                {
                    classes.insert(c.to_owned());
                }
                for e in h
                    .get("path")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if let Some(c) = e.as_str().and_then(path_class) {
                        classes.insert(c.to_owned());
                    }
                }
                if !classes.is_empty() {
                    idx.rows.push((owner, classes));
                }
            }
        }
        idx
    }

    /// Owners whose accumulation class, or a class on whose retention path, is `ty`.
    pub fn owners_with_class(&self, ty: &str) -> BTreeSet<&str> {
        self.rows
            .iter()
            .filter(|(_, classes)| classes.contains(ty))
            .map(|(o, _)| o.as_str())
            .collect()
    }
}

/// Words in a field declaration that are not type names.
const NOT_TYPES: [&str; 1] = ["Object"];

/// The type names a field declaration mentions: the capitalized identifiers of the
/// declaration (the element type of `List<Entry>`, the type of `Entry[] slots`,
/// `mutableListOf<Session>()`), and `name[]` forms, up to the end of the statement.
/// The field's own name and `Object` are left out.
pub fn declared_types(text: &str, line: u32, field: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut decl = String::new();
    for l in text.lines().skip((line as usize).saturating_sub(1)).take(4) {
        decl.push_str(l);
        decl.push(' ');
        if l.contains(';') || l.contains('=') || l.trim_end().ends_with('{') {
            break;
        }
    }
    let mut word = String::new();
    let mut flush = |word: &mut String, next_is_array: bool| {
        if !word.is_empty() {
            let starts_upper = word.chars().next().is_some_and(char::is_uppercase);
            if word != field && starts_upper && !NOT_TYPES.contains(&word.as_str()) {
                out.insert(word.clone());
            }
            if next_is_array && word != field {
                out.insert(format!("{word}[]"));
            }
            word.clear();
        }
    };
    let chars: Vec<char> = decl.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if c.is_alphanumeric() || *c == '_' || *c == '$' {
            word.push(*c);
        } else {
            let arr = *c == '[' && chars.get(i + 1) == Some(&']');
            flush(&mut word, arr);
        }
    }
    flush(&mut word, false);
    out
}

/// The simple names a histogram type goes by: after the last `.`, and after the last
/// `$` of that (`shop.Leaker$Entry` is `Leaker$Entry` and `Entry`), plus the type itself.
fn type_names(ty: &str) -> Vec<String> {
    let tail = ty.rsplit('.').next().unwrap_or(ty).to_owned();
    let inner = tail.rsplit('$').next().unwrap_or(&tail).to_owned();
    vec![ty.to_owned(), tail, inner]
}

/// What links a growing type to findings: the accumulation classes of the run's heap
/// summaries, and the types each finding's owner declares (read from the source at the
/// finding's site).
#[derive(Debug, Default)]
pub struct TypeIndex {
    acc: AccIndex,
    declared: Vec<(FindingId, BTreeSet<String>)>,
}

impl TypeIndex {
    /// Reads the run's summaries, and the declaration line of every finding that has a
    /// site and an owner.
    pub fn build(ledger: &Ledger, run_dir: &Path, project: &Path) -> Self {
        let mut declared = Vec::new();
        for f in ledger.findings() {
            let (Some(site), Some(owner)) = (f.site.as_deref(), f.owner.as_deref()) else {
                continue;
            };
            let Some((file, line)) = site.rsplit_once(':') else {
                continue;
            };
            let Ok(line) = line.parse::<u32>() else {
                continue;
            };
            let field = owner.rsplit('.').next().unwrap_or(owner);
            let Ok(text) = fs::read_to_string(project.join(file)) else {
                continue;
            };
            let types = declared_types(&text, line, field);
            if !types.is_empty() {
                declared.push((f.id.clone(), types));
            }
        }
        Self {
            acc: AccIndex::load(run_dir),
            declared,
        }
    }

    /// The findings a growing type belongs to: those whose owner's accumulation class (or
    /// retention path) is the type, per the heap-graph summaries, and those whose owner
    /// declares the type.
    pub fn findings_for_type(&self, ledger: &Ledger, ty: &str) -> Vec<FindingId> {
        let mut out: Vec<FindingId> = Vec::new();
        let owners = self.acc.owners_with_class(ty);
        if !owners.is_empty() {
            out.extend(
                ledger
                    .findings()
                    .filter(|f| {
                        f.owner
                            .as_deref()
                            .is_some_and(|fo| owners.iter().any(|o| owners_match(fo, o)))
                    })
                    .map(|f| f.id.clone()),
            );
        }
        let names = type_names(ty);
        for (id, types) in &self.declared {
            if !out.contains(id) && names.iter().any(|n| types.contains(n)) {
                out.push(id.clone());
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_classes_skip_roots_and_labels() {
        assert_eq!(path_class("root class app.Holder"), None);
        assert_eq!(
            path_class("java.util.ArrayList"),
            Some("java.util.ArrayList")
        );
        assert_eq!(
            path_class("elementData -> java.lang.Object[]"),
            Some("java.lang.Object[]")
        );
        assert_eq!(path_class("[3] -> app.Item"), Some("app.Item"));
    }

    #[test]
    fn declarations_name_their_types() {
        let java = "class C {\n    static final List<Entry> TRAIL = new ArrayList<>();\n    byte[] buf;\n    static Map<String, List<Session>> sessions =\n        new HashMap<>();\n}\n";
        let t = declared_types(java, 2, "TRAIL");
        assert!(t.contains("Entry") && t.contains("List") && t.contains("ArrayList"));
        assert!(!t.contains("TRAIL"));
        assert!(declared_types(java, 3, "buf").contains("byte[]"));
        let t = declared_types(java, 4, "sessions");
        assert!(
            t.contains("Session") && t.contains("Map") && t.contains("String"),
            "{t:?}"
        );
        let kt = "object Audit {\n    val trail = mutableListOf<Record>()\n}\n";
        assert!(declared_types(kt, 2, "trail").contains("Record"));
        assert_eq!(declared_types(java, 99, "x").len(), 0);
    }

    #[test]
    fn type_names_cover_nested_forms() {
        assert_eq!(
            type_names("shop.Leaker$Entry"),
            ["shop.Leaker$Entry", "Leaker$Entry", "Entry"]
        );
        assert_eq!(type_names("byte[]")[2], "byte[]");
    }

    #[test]
    fn the_index_maps_classes_to_owners() {
        let t = tempfile::tempdir().unwrap();
        fs::create_dir_all(t.path().join("summaries")).unwrap();
        fs::write(
            t.path().join("summaries/d.json"),
            serde_json::json!({"holders": [{
                "key": "static:app.Holder.ITEMS", "kind": "static",
                "owner_class": "app.Holder", "field": "ITEMS",
                "path": ["root class app.Holder", "ITEMS -> java.util.ArrayList",
                         "elementData -> java.lang.Object[]", "[0] -> app.Item"],
                "accumulation": {"class": "java.lang.Object[]", "children": 5}
            }]})
            .to_string(),
        )
        .unwrap();
        let idx = AccIndex::load(t.path());
        assert_eq!(
            idx.owners_with_class("app.Item")
                .into_iter()
                .collect::<Vec<_>>(),
            ["app.Holder.ITEMS"]
        );
        assert_eq!(idx.owners_with_class("app.Other").len(), 0);
        // A run without summaries has an empty index.
        assert_eq!(AccIndex::load(&t.path().join("none")).rows.len(), 0);
    }
}
