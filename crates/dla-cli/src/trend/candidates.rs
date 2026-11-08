//! Candidate findings of a column: the findings whose site is the column's site, or
//! lies in the same file inside the same enclosing function, method or class body.

use std::{collections::HashMap, fs, path::Path};

use dla_census::parse::{
    Lang, parse,
    tree_sitter::{Node, Point, Tree},
};
use dla_core::{FindingId, Ledger, finding::Polarity, trend::ColumnTrend};

/// Node kinds that are a function or method body in some supported grammar.
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
    "struct_item",
    "impl_item",
    "trait_item",
    "interface_declaration",
    "enum_declaration",
    "object_declaration",
    "enum_item",
    "record_declaration",
];

/// Splits `file:line`, dropping a leading `./`.
fn split(site: &str) -> Option<(String, u32)> {
    let (file, line) = site.rsplit_once(':')?;
    let line = line.trim().parse().ok()?;
    let file = file.replace('\\', "/");
    Some((file.strip_prefix("./").unwrap_or(&file).to_owned(), line))
}

/// The 1-based line range of the innermost function around `line`, else of the
/// innermost class.
fn enclosing(tree: &Tree, line: u32) -> Option<(u32, u32)> {
    let row = (line as usize).checked_sub(1)?;
    let root = tree.root_node();
    let start = Point { row, column: 0 };
    let end = Point {
        row,
        column: usize::MAX / 2,
    };
    let mut node: Option<Node<'_>> = root.descendant_for_point_range(start, end);
    let mut class = None;
    while let Some(n) = node {
        if FUNCTIONS.contains(&n.kind()) {
            return Some(range(n));
        }
        if class.is_none() && CLASSES.contains(&n.kind()) {
            class = Some(range(n));
        }
        node = n.parent();
    }
    class
}

fn range(n: Node<'_>) -> (u32, u32) {
    (
        u32::try_from(n.start_position().row + 1).unwrap_or(u32::MAX),
        u32::try_from(n.end_position().row + 1).unwrap_or(u32::MAX),
    )
}

/// Finds the candidates of columns against a ledger, parsing each source file once.
pub struct Matcher<'a> {
    project: &'a Path,
    ledger: &'a Ledger,
    trees: HashMap<String, Option<Tree>>,
}

impl<'a> Matcher<'a> {
    /// A matcher reading sources under `project` and findings from `ledger`.
    pub fn new(project: &'a Path, ledger: &'a Ledger) -> Self {
        Self {
            project,
            ledger,
            trees: HashMap::new(),
        }
    }

    fn tree(&mut self, file: &str) -> Option<&Tree> {
        if !self.trees.contains_key(file) {
            let parsed = Lang::from_path(Path::new(file)).and_then(|lang| {
                let src = fs::read_to_string(self.project.join(file)).ok()?;
                parse(lang, &src).ok()
            });
            self.trees.insert(file.to_owned(), parsed);
        }
        self.trees.get(file)?.as_ref()
    }

    /// The candidates of `c`: the finding it names (when the ledger has it), then the
    /// findings at exactly its site, then those in the same enclosing body, and when
    /// none of these exists, the findings of the same file nearest first. Leak-side
    /// findings only (an `N-*` hypothesis is not a candidate). Empty for a column with
    /// neither a site nor a finding.
    pub fn candidates(&mut self, c: &ColumnTrend) -> Vec<FindingId> {
        let mut out: Vec<FindingId> = Vec::new();
        if let Some(f) = &c.finding
            && self.ledger.finding(f).is_some()
        {
            out.push(f.clone());
        }
        // A probe column named after a holder (`probe:Audit.trail`) is tied to the
        // findings whose owner is that holder, whether or not it has a site.
        for f in self.ledger.findings() {
            if f.polarity() == Polarity::Leak
                && f.owner
                    .as_deref()
                    .is_some_and(|o| dla_core::trend::probe_column_matches_owner(&c.name, o))
                && !out.contains(&f.id)
            {
                out.push(f.id.clone());
            }
        }
        let Some((file, line)) = c.site.as_deref().and_then(split) else {
            return out;
        };
        let sited: Vec<(FindingId, String, u32)> = self
            .ledger
            .findings()
            .filter(|f| f.polarity() == Polarity::Leak)
            .filter_map(|f| {
                let (ff, fl) = f.site.as_deref().and_then(split)?;
                Some((f.id.clone(), ff, fl))
            })
            .collect();
        for (id, ff, fl) in &sited {
            if *ff == file && *fl == line && !out.contains(id) {
                out.push(id.clone());
            }
        }
        let same_file: Vec<&(FindingId, String, u32)> =
            sited.iter().filter(|(_, ff, _)| *ff == file).collect();
        if !same_file.is_empty()
            && let Some((lo, hi)) = self.tree(&file).and_then(|t| enclosing(t, line))
        {
            for (id, _, fl) in &same_file {
                if (lo..=hi).contains(fl) && !out.contains(id) {
                    out.push(id.clone());
                }
            }
        }
        if out.is_empty() {
            // Nothing at the site or in its body: fall back to the findings of the same
            // file, nearest line first. An allocation site (`native.live_bytes@f.rs:16`)
            // often sits in a constructor while the holder candidate is the field it fills
            // (`f.rs:10`), in a different body of the same file.
            let mut near = same_file;
            near.sort_by_key(|(id, _, fl)| (fl.abs_diff(line), id.to_string()));
            out.extend(near.into_iter().map(|(id, _, _)| id.clone()));
        }
        out
    }

    /// Fills `candidates` of every column.
    pub fn fill(&mut self, columns: &mut [ColumnTrend]) {
        for c in columns {
            c.candidates = self.candidates(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trend_cmd::analysis::{
        analyze_series,
        testutil::{Col, rising, series},
    };
    use dla_core::{
        NewEvidence, NewFinding, Origin,
        evidence::{Authority, EvidenceCoverage, EvidenceKind, Source, Tool},
    };
    use dla_stats::GateThresholds;
    use serde_json::json;

    const GO: &str = "package cache\n\nvar entries = map[string]int{}\n\nfunc Put(k string) {\n\tentries[k] = 1\n}\n\nfunc Other() {\n\tprintln(1)\n}\n";

    fn ledger_with(sites: &[(&str, Option<&str>)]) -> Ledger {
        let mut l = Ledger::from_text("").unwrap();
        l.add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: Authority::Static,
            kind: EvidenceKind::StaticHit,
            result: json!({}),
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        for (origin, site) in sites {
            let mut nf = NewFinding::new(
                Origin::parse(origin).unwrap(),
                vec![dla_core::EvidenceId::new(1)],
            );
            nf.site = site.map(str::to_owned);
            l.add_finding(nf).unwrap();
        }
        l
    }

    fn trend_of(site: &str, finding: Option<FindingId>) -> ColumnTrend {
        let mut c = Col::counter("probe:Cache.entries", &rising(1.0, 5.0)).site(site);
        c.finding = finding;
        let s = series("S1", 1, vec![c]);
        analyze_series(&s, &[0], &GateThresholds::default()).remove(0)
    }

    #[test]
    fn exact_site_then_same_function_or_class_then_the_named_finding() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("cache.go"), GO).unwrap();
        let ledger = ledger_with(&[
            ("census", Some("cache.go:6")),  // F1: inside Put
            ("census", Some("cache.go:10")), // F2: inside Other
            ("census", Some("cache.go:5")),  // F3: the func line of Put
            ("census", None),                // F4: no site
            ("census", Some("other.go:6")),  // F5: other file
        ]);
        let mut m = Matcher::new(dir.path(), &ledger);
        // Column site is inside Put: F1 exact, F3 same function; F2 is another function.
        let t = trend_of("cache.go:6", None);
        assert_eq!(m.candidates(&t), [FindingId::new(1), FindingId::new(3)]);
        // A column at the top-level var has no enclosing body and no exact match: the
        // findings of the same file, nearest first (F3 at :5, F1 at :6, F2 at :10).
        assert_eq!(
            m.candidates(&trend_of("cache.go:3", None)),
            [FindingId::new(3), FindingId::new(1), FindingId::new(2)]
        );
        // The named finding comes first, even without a site match.
        let t = trend_of("cache.go:10", Some(FindingId::new(4)));
        assert_eq!(m.candidates(&t), [FindingId::new(4), FindingId::new(2)]);
        // A named finding the ledger does not have is not a candidate.
        assert!(
            m.candidates(&trend_of("nowhere.go:1", Some(FindingId::new(9))))
                .is_empty()
        );
    }

    #[test]
    fn a_probe_column_is_tied_to_the_finding_with_that_owner() {
        let dir = tempfile::tempdir().unwrap();
        let mut ledger = ledger_with(&[("census", Some("Audit.kt:3"))]);
        let mut nf = NewFinding::new(
            Origin::parse("census").unwrap(),
            vec![dla_core::EvidenceId::new(1)],
        );
        nf.owner = Some("shop.codec.Audit.trail".into());
        ledger.add_finding(nf).unwrap();
        let mut m = Matcher::new(dir.path(), &ledger);
        let mut c = trend_of("nowhere.kt:1", None);
        c.name = "probe:Audit.trail.size".into();
        c.site = None;
        assert_eq!(m.candidates(&c), [FindingId::new(2)]);
    }

    #[test]
    fn a_class_body_matches_findings_in_its_methods() {
        let dir = tempfile::tempdir().unwrap();
        let java = "class Cache {\n  static Map<String,String> M = new HashMap<>();\n  void put(String k) {\n    M.put(k, k);\n  }\n}\nclass Other {\n  void f() {}\n}\n";
        fs::write(dir.path().join("Cache.java"), java).unwrap();
        let ledger = ledger_with(&[
            ("census", Some("Cache.java:4")),
            ("census", Some("Cache.java:8")),
        ]);
        let mut m = Matcher::new(dir.path(), &ledger);
        // The probe is defined on the field line (class level): F1 in a method of the class.
        let t = trend_of("Cache.java:2", None);
        assert_eq!(m.candidates(&t), [FindingId::new(1)]);
    }

    #[test]
    fn non_leak_siblings_and_unreadable_files_are_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let mut ledger = ledger_with(&[("census", Some("cache.go:6"))]);
        let mut nf = NewFinding::new(
            Origin::parse("hypothesis").unwrap(),
            vec![dla_core::EvidenceId::new(1)],
        );
        nf.sibling_of = Some(FindingId::new(1));
        nf.hypothesis = Some("N-WARM".into());
        nf.refute_if = Some("keeps growing".into());
        nf.site = Some("cache.go:6".into());
        ledger.add_finding(nf).unwrap();
        let mut m = Matcher::new(dir.path(), &ledger);
        // No source file on disk: only the exact site matches.
        assert_eq!(
            m.candidates(&trend_of("cache.go:6", None)),
            [FindingId::new(1)]
        );
    }
}
