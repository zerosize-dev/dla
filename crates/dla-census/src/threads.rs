//! Holders kept per thread: what the census can tell of them cheaply.
//!
//! A holder that is kept per thread grows with the number of distinct threads or thread
//! lifetimes the code sees. Two facts say so beside the holder kind `thread-local`
//! (`dla_core::census::HolderFacts::thread_keyed`):
//!
//! - `thread_class`: the owner class (the owner up to its last segment) also owns a holder of
//!   kind `thread-local`;
//! - `thread_init` on a writer site: the site lies in a thread-local initialiser. Read from the
//!   text: for Java and Kotlin the lines inside a `withInitial(...)` call, the method that a
//!   `withInitial(X::name)` reference names, and a method called `initialValue`; for Python the
//!   `__init__` of a class derived from a `local` class. Rust holders carry no site facts, and
//!   a map keyed by a thread or a thread id is not recognised (the census records no key types).

use std::collections::{BTreeMap, BTreeSet};

use dla_core::{census::CensusEntry, finding::Holder};

use crate::{model::SourceFile, parse::Lang};

/// Sets `thread_class` and the `thread_init` of writer sites on `entries`.
pub fn mark(entries: &mut [CensusEntry], files: &[SourceFile]) {
    let classes: BTreeSet<String> = entries
        .iter()
        .filter(|e| e.holder == Holder::ThreadLocal)
        .filter_map(|e| class_of(&e.owner).map(str::to_owned))
        .collect();
    let inits: BTreeMap<&str, Initialisers> = files
        .iter()
        .filter_map(|f| Initialisers::of(f).map(|i| (f.rel.as_str(), i)))
        .collect();
    for e in entries {
        if class_of(&e.owner).is_some_and(|c| classes.contains(c)) {
            e.facts.thread_class = true;
        }
        for s in &mut e.facts.writer_sites {
            let Some((file, line)) = s.site.rsplit_once(':') else {
                continue;
            };
            let Ok(line) = line.parse::<u32>() else {
                continue;
            };
            if let Some(i) = inits.get(file) {
                s.thread_init = i.covers(line, s.function.as_deref());
            }
        }
    }
}

/// The owner up to its last segment; `None` when the owner has one segment.
fn class_of(owner: &str) -> Option<&str> {
    owner
        .rsplit_once('.')
        .map(|(c, _)| c)
        .filter(|c| !c.is_empty())
}

/// The thread-local initialisers of one file.
#[derive(Debug, Default)]
struct Initialisers {
    /// Line ranges (first, last) of initialiser bodies.
    ranges: Vec<(u32, u32)>,
    /// Names of functions that are initialisers.
    names: BTreeSet<String>,
}

impl Initialisers {
    fn covers(&self, line: u32, function: Option<&str>) -> bool {
        self.ranges.iter().any(|&(a, b)| (a..=b).contains(&line))
            || function.is_some_and(|f| self.names.contains(f))
    }

    fn of(f: &SourceFile) -> Option<Self> {
        let out = match f.lang {
            Lang::Java | Lang::Kotlin => jvm(&f.text),
            Lang::Python => python(&f.text),
            _ => return None,
        };
        (!out.ranges.is_empty() || !out.names.is_empty()).then_some(out)
    }
}

fn line_of(text: &str, at: usize) -> u32 {
    u32::try_from(text[..at].bytes().filter(|b| *b == b'\n').count() + 1).unwrap_or(u32::MAX)
}

fn jvm(text: &str) -> Initialisers {
    let mut out = Initialisers::default();
    let needle = "withInitial";
    let mut from = 0;
    while let Some(at) = text[from..].find(needle) {
        let start = from + at;
        from = start + needle.len();
        let rest = &text[from..];
        let Some(open) = rest.find('(').filter(|o| rest[..*o].trim().is_empty()) else {
            continue;
        };
        let body_start = from + open + 1;
        let mut depth = 1usize;
        let mut end = text.len();
        for (i, c) in text[body_start..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = body_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        let body = &text[body_start..end];
        // `Type::name` or `this::name`: the named method is the initialiser.
        if let Some((_, name)) = body.trim().rsplit_once("::") {
            let name = name.trim();
            if !name.is_empty()
                && name != "new"
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            {
                out.names.insert(name.to_owned());
            }
        }
        out.ranges.push((line_of(text, start), line_of(text, end)));
    }
    // An `initialValue` override: any method of that name.
    if text.contains("initialValue") {
        out.names.insert("initialValue".to_owned());
    }
    out
}

fn python(text: &str) -> Initialisers {
    let mut out = Initialisers::default();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        let t = l.trim_start();
        let indent = l.len() - t.len();
        let derived = t.strip_prefix("class ").is_some_and(|rest| {
            rest.split_once('(')
                .and_then(|(_, b)| b.split_once(')'))
                .is_some_and(|(bases, _)| {
                    bases
                        .split(',')
                        .any(|b| b.trim().rsplit('.').next() == Some("local"))
                })
        });
        if !derived {
            i += 1;
            continue;
        }
        // The class body: the lines after the header that are blank or indented deeper.
        let mut j = i + 1;
        while j < lines.len() {
            let n = lines[j];
            if !n.trim().is_empty() && n.len() - n.trim_start().len() <= indent {
                break;
            }
            j += 1;
        }
        // The `__init__` inside it, down to the next line at its own indentation or less.
        let mut k = i + 1;
        while k < j {
            let m = lines[k];
            let mt = m.trim_start();
            if mt.starts_with("def __init__") {
                let mi = m.len() - mt.len();
                let mut e = k + 1;
                while e < j {
                    let n = lines[e];
                    if !n.trim().is_empty() && n.len() - n.trim_start().len() <= mi {
                        break;
                    }
                    e += 1;
                }
                out.ranges.push((
                    u32::try_from(k + 1).unwrap_or(u32::MAX),
                    u32::try_from(e).unwrap_or(u32::MAX),
                ));
                k = e;
            } else {
                k += 1;
            }
        }
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::{
        census::{HolderFacts, SiteFact, SiteKind},
        finding::Region,
    };

    fn file(rel: &str, lang: Lang, text: &str) -> SourceFile {
        SourceFile {
            rel: rel.into(),
            text: text.into(),
            lang,
            is_test: false,
        }
    }

    fn entry(holder: Holder, owner: &str, site: &str, writers: &[(&str, &str)]) -> CensusEntry {
        CensusEntry {
            kind: "x".into(),
            holder,
            region: Region::Heap,
            site: site.into(),
            owner: owner.into(),
            writers: writers.iter().map(|(s, _)| (*s).to_owned()).collect(),
            removers: Vec::new(),
            facts: HolderFacts {
                writer_sites: writers
                    .iter()
                    .map(|(s, f)| SiteFact {
                        site: (*s).to_owned(),
                        kind: SiteKind::Insert,
                        depth: 0,
                        function: Some((*f).to_owned()),
                        init: false,
                        startup: false,
                        startup_via: Vec::new(),
                        param: false,
                        constant: false,
                        exported: false,
                        thread_init: false,
                    })
                    .collect(),
                ..HolderFacts::default()
            },
        }
    }

    const JAVA: &str = "package p;\nclass Pool {\n  final java.util.List<X> live = new java.util.ArrayList<>();\n  final ThreadLocal<X> own = ThreadLocal.withInitial(this::init);\n  X init() {\n    X x = new X();\n    live.add(x);\n    return x;\n  }\n}\nclass Lambda {\n  final java.util.List<X> live = new java.util.ArrayList<>();\n  final ThreadLocal<X> own = ThreadLocal.withInitial(() -> {\n    X x = new X();\n    live.add(x);\n    return x;\n  });\n}\nclass Plain {\n  final java.util.List<X> live = new java.util.ArrayList<>();\n  void add(X x) {\n    live.add(x);\n  }\n}\n";

    #[test]
    fn an_owner_class_with_a_thread_local_marks_its_other_holders() {
        let mut es = vec![
            entry(Holder::ThreadLocal, "p.Pool.own", "A.java:4", &[]),
            entry(Holder::ContainerField, "p.Pool.live", "A.java:3", &[]),
            entry(Holder::ContainerField, "p.Plain.live", "A.java:19", &[]),
        ];
        mark(&mut es, &[file("A.java", Lang::Java, JAVA)]);
        assert!(es[1].facts.thread_class);
        assert!(!es[2].facts.thread_class);
        assert!(es[1].facts.thread_keyed(Some(Holder::ContainerField)));
        assert!(!es[2].facts.thread_keyed(Some(Holder::ContainerField)));
    }

    #[test]
    fn a_writer_in_a_method_reference_target_is_a_thread_init_writer_alone() {
        // No thread-local holder entry: only the writer fact can make it thread-keyed.
        let mut es = vec![entry(
            Holder::ContainerField,
            "p.Pool.live",
            "A.java:3",
            &[("A.java:7", "init")],
        )];
        mark(&mut es, &[file("A.java", Lang::Java, JAVA)]);
        assert!(!es[0].facts.thread_class);
        assert!(es[0].facts.writer_sites[0].thread_init);
        assert!(es[0].facts.thread_keyed(Some(Holder::ContainerField)));
    }

    #[test]
    fn a_writer_inside_a_with_initial_lambda_is_a_thread_init_writer() {
        let mut es = vec![
            entry(
                Holder::ContainerField,
                "p.Lambda.live",
                "A.java:12",
                &[("A.java:15", "<init>")],
            ),
            entry(
                Holder::ContainerField,
                "p.Plain.live",
                "A.java:19",
                &[("A.java:21", "add")],
            ),
        ];
        mark(&mut es, &[file("A.java", Lang::Java, JAVA)]);
        assert!(es[0].facts.writer_sites[0].thread_init);
        assert!(!es[1].facts.writer_sites[0].thread_init);
    }

    #[test]
    fn an_initial_value_override_is_an_initialiser() {
        let src = "class T {\n  java.util.List<X> live = new java.util.ArrayList<>();\n  ThreadLocal<X> own = new ThreadLocal<X>() {\n    protected X initialValue() {\n      live.add(null);\n      return null;\n    }\n  };\n}\n";
        let mut es = vec![entry(
            Holder::ContainerField,
            "T.live",
            "T.java:2",
            &[("T.java:5", "initialValue")],
        )];
        mark(&mut es, &[file("T.java", Lang::Java, src)]);
        assert!(es[0].facts.writer_sites[0].thread_init);
    }

    #[test]
    fn a_python_local_subclass_init_is_an_initialiser() {
        let src = "import threading\n\nITEMS = []\n\n\nclass Own(threading.local):\n    def __init__(self):\n        ITEMS.append(self)\n\n\ndef other():\n    ITEMS.append(1)\n";
        let mut es = vec![entry(
            Holder::StaticGlobal,
            "m.ITEMS",
            "m.py:3",
            &[("m.py:8", "__init__"), ("m.py:12", "other")],
        )];
        mark(&mut es, &[file("m.py", Lang::Python, src)]);
        assert!(es[0].facts.writer_sites[0].thread_init);
        assert!(!es[0].facts.writer_sites[1].thread_init);
    }
}
