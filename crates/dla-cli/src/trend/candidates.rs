//! Candidate findings of a column: the findings whose site is the column's site, or
//! lies in the same file inside the same enclosing function, method or class body, or
//! whose census writers include the column's exact site (file and line). A finding that
//! only shares the file is never a candidate.

use std::{collections::HashMap, fs, path::Path};

use dla_census::parse::{
    Lang, parse,
    tree_sitter::{Node, Point, Tree},
};
use dla_core::{FindingId, Ledger, attribution::Resolver, finding::Polarity, trend::ColumnTrend};

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
fn enclosing(tree: &Tree, src: &str, line: u32) -> Option<(u32, u32)> {
    let row = (line as usize).checked_sub(1)?;
    let root = tree.root_node();
    // Start at the first non-blank column: a `fn` line begins at its indentation, so a
    // range from column 0 would not lie inside the function and would name its parent.
    let indent = src
        .lines()
        .nth(row)
        .map_or(0, |l| l.len() - l.trim_start().len());
    let start = Point {
        row,
        column: indent,
    };
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
    resolver: Option<Resolver>,
    trees: HashMap<String, Option<(Tree, String)>>,
}

impl<'a> Matcher<'a> {
    /// A matcher reading sources under `project` and findings from `ledger`.
    pub fn new(project: &'a Path, ledger: &'a Ledger) -> Self {
        Self {
            project,
            ledger,
            resolver: None,
            trees: HashMap::new(),
        }
    }

    /// Reads the census writers of the run at `run_dir`, so a column at exactly a writer
    /// line of a finding's holder is a candidate for that finding.
    #[must_use]
    pub fn with_census(mut self, run_dir: &Path) -> Self {
        self.resolver = Some(Resolver::new(run_dir, self.project));
        self
    }

    /// The census writers of a finding's owner as `(file, line)`, empty without a census.
    fn writers_of(&self, owner: Option<&str>) -> Vec<(String, u32)> {
        let (Some(resolver), Some(owner)) = (&self.resolver, owner) else {
            return Vec::new();
        };
        resolver
            .resolve(owner)
            .map(|r| r.writers.iter().filter_map(|w| split(w)).collect())
            .unwrap_or_default()
    }

    fn tree(&mut self, file: &str) -> Option<&(Tree, String)> {
        if !self.trees.contains_key(file) {
            let parsed = Lang::from_path(Path::new(file)).and_then(|lang| {
                let src = fs::read_to_string(self.project.join(file)).ok()?;
                let tree = parse(lang, &src).ok()?;
                Some((tree, src))
            });
            self.trees.insert(file.to_owned(), parsed);
        }
        self.trees.get(file)?.as_ref()
    }

    /// The candidates of `c`: the finding it names (when the ledger has it), then the
    /// findings at exactly its site, then those in the same enclosing body, then those
    /// with a census writer at exactly the site. A writer elsewhere in the site's body
    /// ties nothing. Findings that only share the file are never candidates.
    /// Leak-side findings only (an `N-*` hypothesis is not a candidate). Empty for a
    /// column with neither a site nor a finding.
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
        let body = self
            .tree(&file)
            .and_then(|(t, src)| enclosing(t, src, line));
        for (id, ff, fl) in &sited {
            if *ff == file
                && body.is_some_and(|(lo, hi)| (lo..=hi).contains(fl))
                && !out.contains(id)
            {
                out.push(id.clone());
            }
        }
        // A census writer of the finding's holder AT the column's site (equal file and
        // line). A writer somewhere in the site's body is not enough: it may belong to
        // another holder that the body also touches.
        let leaks: Vec<(FindingId, Option<String>)> = self
            .ledger
            .findings()
            .filter(|f| f.polarity() == Polarity::Leak)
            .map(|f| (f.id.clone(), f.owner.clone()))
            .collect();
        for (id, owner) in leaks {
            if out.contains(&id) {
                continue;
            }
            let hit = self
                .writers_of(owner.as_deref())
                .iter()
                .any(|(wf, wl)| *wf == file && *wl == line);
            if hit {
                out.push(id);
            }
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
        // findings that only share the file are not candidates.
        assert!(m.candidates(&trend_of("cache.go:3", None)).is_empty());
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

    const ARENA: &str = "use std::collections::HashMap;\n\npub struct Arena {\n    pub pages: Vec<Page>,\n    pub free: Vec<usize>,\n}\n\nimpl Arena {\n    pub fn new() -> Self {\n        Arena { pages: Vec::new(), free: Vec::new() }\n    }\n\n    pub fn hold(&mut self, p: Page) {\n        self.pages.push(p);\n    }\n\n    pub fn release(&mut self, i: usize) {\n        self.free.push(i);\n    }\n}\n";

    fn write_census(run_dir: &Path, entries: &[(&str, &str, &[&str])]) {
        let dir = run_dir.join("census");
        fs::create_dir_all(&dir).unwrap();
        let entries: Vec<serde_json::Value> = entries
            .iter()
            .map(|(owner, site, writers)| {
                json!({"kind":"struct-field","holder":"container-field","region":"heap",
                    "site":site,"owner":owner,"writers":writers,"removers":[]})
            })
            .collect();
        fs::write(
            dir.join("census.json"),
            json!({"entries": entries}).to_string(),
        )
        .unwrap();
    }

    fn finding_of(l: &mut Ledger, site: &str, owner: &str) {
        let mut nf = NewFinding::new(
            Origin::parse("census").unwrap(),
            vec![dla_core::EvidenceId::new(1)],
        );
        nf.site = Some(site.into());
        nf.owner = Some(owner.into());
        l.add_finding(nf).unwrap();
    }

    #[test]
    fn a_column_ties_to_the_census_writer_and_not_to_a_sibling_field_of_the_file() {
        // The rust-slot-arena shape: the column sits at the writer of `Arena.pages`, and
        // `Arena.free`, declared in the same file at another line, is not a candidate.
        let project = tempfile::tempdir().unwrap();
        let run = tempfile::tempdir().unwrap();
        fs::write(project.path().join("arena.rs"), ARENA).unwrap();
        write_census(
            run.path(),
            &[
                ("Arena.pages", "arena.rs:4", &["arena.rs:14"]),
                ("Arena.free", "arena.rs:5", &["arena.rs:18"]),
            ],
        );
        let mut ledger = ledger_with(&[]);
        finding_of(&mut ledger, "arena.rs:5", "Arena.free");
        finding_of(&mut ledger, "arena.rs:4", "Arena.pages");
        let mut m = Matcher::new(project.path(), &ledger).with_census(run.path());
        // F2 is `pages`: the column is at its writer line.
        assert_eq!(
            m.candidates(&trend_of("arena.rs:14", None)),
            [FindingId::new(2)]
        );
        // A column on the `fn hold` line is not at the writer (:14): no tie.
        assert!(m.candidates(&trend_of("arena.rs:13", None)).is_empty());
        // Without the census, nothing but the file is shared: no candidate.
        let mut bare = Matcher::new(project.path(), &ledger);
        assert!(bare.candidates(&trend_of("arena.rs:14", None)).is_empty());
    }

    /// A source of `lines` where each `(line, text)` replaces its line (1-based).
    fn source(lines: usize, put: &[(usize, &str)]) -> String {
        let mut v: Vec<String> = vec![String::new(); lines];
        for (n, t) in put {
            v[n - 1] = (*t).to_owned();
        }
        v.join("\n") + "\n"
    }

    #[test]
    fn a_writer_in_the_body_of_a_column_ties_nothing_but_a_writer_at_a_column_site_does() {
        // The rust-delayed-queue shape. `place` (orders.rs:27-31) allocates at :28 and
        // holds the writer (:30) of the wrong holder `Orders.open`, whose declaration is
        // elsewhere. The right holder `Scheduler.queue` is written at scheduler.rs:53.
        let project = tempfile::tempdir().unwrap();
        let run = tempfile::tempdir().unwrap();
        fs::write(
            project.path().join("orders.rs"),
            source(
                40,
                &[
                    (10, "    open: Vec<Order>,"),
                    (27, "    pub fn place(&mut self, o: Order) {"),
                    (28, "        let boxed = Box::new(o);"),
                    (30, "        self.open.push(*boxed);"),
                    (31, "    }"),
                ],
            ),
        )
        .unwrap();
        fs::write(
            project.path().join("scheduler.rs"),
            source(
                60,
                &[
                    (20, "    queue: Vec<Slot>,"),
                    (50, "    pub fn defer(&mut self, s: Slot) {"),
                    (53, "        self.queue.push(s);"),
                    (54, "    }"),
                ],
            ),
        )
        .unwrap();
        write_census(
            run.path(),
            &[
                ("Orders.open", "orders.rs:10", &["orders.rs:30"]),
                ("Scheduler.queue", "scheduler.rs:20", &["scheduler.rs:53"]),
            ],
        );
        let mut ledger = ledger_with(&[]);
        finding_of(&mut ledger, "orders.rs:10", "Orders.open");
        finding_of(&mut ledger, "scheduler.rs:20", "Scheduler.queue");
        let mut m = Matcher::new(project.path(), &ledger).with_census(run.path());
        // The column at the allocation line: the wrong holder's writer lies in its body
        // (:30), which is not the column's site. It ties nothing.
        assert!(m.candidates(&trend_of("orders.rs:28", None)).is_empty());
        // The column at the writer of the right holder, in another file, ties it.
        assert_eq!(
            m.candidates(&trend_of("scheduler.rs:53", None)),
            [FindingId::new(2)]
        );
        // The wrong holder's own writer line ties the wrong holder, as before.
        assert_eq!(
            m.candidates(&trend_of("orders.rs:30", None)),
            [FindingId::new(1)]
        );
    }

    #[test]
    fn a_pool_column_ties_the_finding_at_its_site_and_no_field_written_in_the_same_body() {
        // The native-split-freelist shape. `pool_grow` (pool.c:60-75) allocates at :67,
        // where F1 has its site; `block.next` and `bump` are written
        // in the same body (:70, :72) and are not candidates.
        let project = tempfile::tempdir().unwrap();
        let run = tempfile::tempdir().unwrap();
        fs::write(
            project.path().join("pool.c"),
            source(
                80,
                &[
                    (60, "static void pool_grow(struct pool *p) {"),
                    (67, "    struct chunk *c = malloc(sizeof *c);"),
                    (70, "    c->next = p->chunks;"),
                    (72, "    p->bump = c->data;"),
                    (75, "}"),
                ],
            ),
        )
        .unwrap();
        write_census(
            run.path(),
            &[
                ("Block.next", "types.h:11", &["pool.c:70"]),
                ("Pool.bump", "types.h:20", &["pool.c:72"]),
                ("Pool.chunks", "types.h:21", &["pool.c:69"]),
            ],
        );
        let mut ledger = ledger_with(&[("census", Some("pool.c:67"))]);
        finding_of(&mut ledger, "types.h:11", "Block.next");
        finding_of(&mut ledger, "types.h:20", "Pool.bump");
        finding_of(&mut ledger, "types.h:21", "Pool.chunks");
        let mut m = Matcher::new(project.path(), &ledger).with_census(run.path());
        // F1 is the finding at the column's exact site; the other three have writers
        // in the body of `pool_grow` but none at :67.
        assert_eq!(
            m.candidates(&trend_of("pool.c:67", None)),
            [FindingId::new(1)]
        );
    }

    #[test]
    fn an_interning_column_ties_the_finding_whose_writer_is_at_its_site_and_not_a_list_link_written_nearby()
     {
        // The native-parcel-router shape: the column at intern.c:52 ties F1 (here
        // `Interner.table`, declared at :5, tied through its census writer at exactly :52;
        // in the real fixture the finding at that site is F3, `label_intern`) and not
        // `Label.next`, written at :41 and :55 in the same function.
        let project = tempfile::tempdir().unwrap();
        let run = tempfile::tempdir().unwrap();
        fs::write(
            project.path().join("intern.c"),
            source(
                70,
                &[
                    (38, "static const char *intern(const char *s) {"),
                    (41, "    l->next = table;"),
                    (52, "    table = l;"),
                    (55, "    l->next = NULL;"),
                    (58, "}"),
                ],
            ),
        )
        .unwrap();
        write_census(
            run.path(),
            &[
                ("Interner.table", "intern.c:5", &["intern.c:52"]),
                ("Label.next", "types.h:9", &["intern.c:41", "intern.c:55"]),
            ],
        );
        let mut ledger = ledger_with(&[]);
        finding_of(&mut ledger, "intern.c:5", "Interner.table");
        finding_of(&mut ledger, "types.h:9", "Label.next");
        let mut m = Matcher::new(project.path(), &ledger).with_census(run.path());
        assert_eq!(
            m.candidates(&trend_of("intern.c:52", None)),
            [FindingId::new(1)]
        );
    }
}
