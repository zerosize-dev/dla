//! The ranking of candidates: one pure function of a finding and its census annotation,
//! shared by the measurement queue and the report.
//!
//! Tier of a candidate (lower is better):
//!
//! 1. the origin or `also_origins` holds `known-leak:*` or `detector:dla/*` (DLA's own rules);
//! 2. a census entry (origin or `also_origins`) with writers, no removers and a holder in
//!    {registration table, container field, static/global};
//! 3. any other `detector:*` origin;
//! 4. a census entry with writers (removers present, or another holder kind);
//! 5. everything else.
//!
//! A finding whose site is test, example, benchmark or documentation code is tier 6
//! whatever else holds. Tiers 1 to 4 are eligible for the queue; 5 and 6 never are.
//!
//! Inside the eligible set the order is the [`score`]: a sum of named weights over signals
//! of the finding and of the census facts recorded on its annotation (see [`WEIGHTS`]).
//! Descending score first; ties by tier, then findings with two origins first, then ledger
//! order. The score is a pure function of the finding and its annotation.

use crate::{
    census::{Bound, ContainerClass, Lifetime, OwnerCreation, Receiver, SiteFact, SiteKind},
    finding::{Finding, Holder},
    ledger::Annotation,
};

/// The one table of weights. `weights! { name = points }` defines the constant `w::name` and
/// a row of [`WEIGHTS`]; a signal is named by its row.
macro_rules! weights {
    ($($name:ident = $val:expr),* $(,)?) => {
        /// The weight of each signal, by name.
        #[allow(non_upper_case_globals)]
        pub mod w {
            $(
                #[doc = concat!("Weight of `", stringify!($name), "`.")]
                pub const $name: i32 = $val;
            )*
        }
        /// Every signal with its weight, in the order the score adds them.
        pub const WEIGHTS: &[(&str, i32)] = &[$((stringify!($name), $val)),*];
    };
}

weights! {
    // Origin.
    rule_dla = 15,
    rule_strong = 45,
    known_leak = 60,
    multi_origin = 2,
    tier5 = -20,
    // The holder.
    life_process = 20,
    life_object = 3,
    container = 20,
    registry = 10,
    noncontainer = -80,
    weak = -6,
    bounded = -14,
    cache_unbounded = 10,
    literal_table = -25,
    key_type_bounded = -10,
    pool_type = -15,
    // Writers.
    w_grow = 25,
    w_none = -40,
    w_reassign_only = -35,
    w_runtime = 20,
    w_startup_all = -30,
    w_param_flow = 15,
    w_const_only = -10,
    w_exported = 6,
    // Removers.
    growth_unmatched = 25,
    r_none = 8,
    r_holder_level = -12,
    // Registrations and owners.
    recv_local = -20,
    recv_outer = 8,
    owner_percall = -10,
    owner_startup = 5,
}

/// The worst tier the measurement queue takes.
pub const MAX_ELIGIBLE_TIER: u8 = 4;
/// The tier of a finding in test, example, benchmark or documentation code.
pub const TIER_DEMOTED: u8 = 6;

/// Directory names that mark test, example, benchmark or documentation code.
const NON_PRODUCT_DIRS: &[&str] = &[
    "test",
    "tests",
    "testing",
    "testdata",
    "__tests__",
    "spec",
    "example",
    "examples",
    "benchmark",
    "benchmarks",
    "bench",
    "docs",
    "doc",
];

/// Words that mark a path component (a directory, or a file name without its extension, split
/// at separators and capital letters) as test support, example, sample, demo or benchmark
/// code: `testutils`, `mock_store`, `FakeClock`, `e2e`, `interop`. A word inside another word
/// (`contest`, `latest`, `sampler`) does not count.
const NON_PRODUCT_WORDS: &[&str] = &[
    "test",
    "mock",
    "fake",
    "stub",
    "e2e",
    "fixture",
    "interop",
    "example",
    "sample",
    "demo",
    "bench",
    "benchmark",
];

/// Compound words of the same kind that start with `test`.
const NON_PRODUCT_COMPOUNDS: &[&str] = &[
    "testing",
    "testutil",
    "testutils",
    "testdata",
    "testhelper",
    "testhelpers",
    "testlib",
    "testkit",
    "testsupport",
    "mockito",
    "mocking",
];

/// The lower-case words of a path component: split at anything that is not a letter or digit
/// and before a capital that follows a lower-case letter.
fn component_words(c: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for ch in c.chars() {
        if !ch.is_alphanumeric() {
            if !cur.is_empty() {
                words.push(std::mem::take(&mut cur));
            }
            prev_lower = false;
            continue;
        }
        if ch.is_uppercase() && prev_lower && !cur.is_empty() {
            words.push(std::mem::take(&mut cur));
        }
        prev_lower = ch.is_lowercase() || ch.is_ascii_digit();
        cur.extend(ch.to_lowercase());
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

/// True when a path component names test support, an example, a sample, a demo or a benchmark.
fn is_non_product_component(c: &str) -> bool {
    component_words(c).iter().any(|w| {
        NON_PRODUCT_COMPOUNDS.contains(&w.as_str())
            || NON_PRODUCT_WORDS
                .iter()
                .any(|n| w == n || w.strip_suffix('s') == Some(n))
    })
}

/// True when the file name is a test file of some runtime.
fn is_test_file(name: &str) -> bool {
    name == "conftest.py"
        || name.ends_with("_test.go")
        || (name.starts_with("test_") && name.ends_with(".py"))
        || name.ends_with("_test.py")
        || [".test.js", ".test.ts", ".spec.js", ".spec.ts"]
            .iter()
            .any(|s| name.ends_with(s))
        || name.ends_with("Test.java")
        || name.ends_with("Tests.java")
        || name.ends_with("Test.kt")
}

/// True when `site` (`file:line`, or a bare path) names test, example, benchmark or
/// documentation code: a path component from [`NON_PRODUCT_DIRS`] or a test file name.
pub fn is_non_product_site(site: &str) -> bool {
    let path = match site.rsplit_once(':') {
        Some((p, line)) if line.chars().all(|c| c.is_ascii_digit()) && !line.is_empty() => p,
        _ => site,
    };
    let path = path.replace('\\', "/");
    let mut parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let Some(file) = parts.pop() else {
        return false;
    };
    let stem = file.split_once('.').map_or(file, |(s, _)| s);
    parts
        .iter()
        .any(|d| NON_PRODUCT_DIRS.contains(&d.to_ascii_lowercase().as_str()))
        || parts.iter().any(|d| is_non_product_component(d))
        || is_non_product_component(stem)
        || is_test_file(file)
}

/// True for a census registration (`holder` registration-table, a registry) with no writer
/// other than its own site: the call that registers is itself the insert, as when a listener is
/// added to an object handed in from outside (no other statement "writes" that object).
fn registration_is_own_writer(f: &Finding, a: &Annotation) -> bool {
    f.site.is_some()
        && f.holder == Some(Holder::RegistrationTable)
        && a.facts.container == Some(ContainerClass::Registry)
        && a.writers.is_empty()
        && a.facts.writer_sites.is_empty()
        && std::iter::once(&f.origin)
            .chain(&f.also_origins)
            .any(|o| o == "census")
}

/// The tier of a finding, 1 (strongest) to 6 (demoted), from the finding and its
/// annotation only.
pub fn tier(f: &Finding, annotation: Option<&Annotation>) -> u8 {
    if f.site.as_deref().is_some_and(is_non_product_site) {
        return TIER_DEMOTED;
    }
    let origins = || {
        std::iter::once(&f.origin)
            .chain(&f.also_origins)
            .map(|o| o.as_str())
    };
    if origins().any(|o| o.starts_with("known-leak:") || o.starts_with("detector:dla/")) {
        return 1;
    }
    let census = origins().any(|o| o == "census");
    let (writers, removers) = annotation.map_or((0, 0), |a| {
        (
            a.writers.len() + usize::from(registration_is_own_writer(f, a)),
            a.removers.len(),
        )
    });
    if census
        && writers > 0
        && removers == 0
        && matches!(
            f.holder,
            Some(Holder::RegistrationTable | Holder::ContainerField | Holder::StaticGlobal)
        )
    {
        return 2;
    }
    if origins().any(|o| o.starts_with("detector:")) {
        return 3;
    }
    if census && writers > 0 {
        return 4;
    }
    5
}

/// True when a tier is eligible for the measurement queue.
pub fn eligible_tier(tier: u8) -> bool {
    tier <= MAX_ELIGIBLE_TIER
}

/// The sort key inside a tier: findings that carry more than one origin first. Ledger order
/// is the caller's stable tiebreak.
pub fn multi_origin_key(f: &Finding) -> u8 {
    u8::from(f.also_origins.is_empty())
}

/// The text the report prints above the ranked findings.
pub const TIER_NOTE: &str = "Suspected, static-only findings are ordered by a score: a sum of named weights for a DLA rule hit (stronger when the holder lives for the whole process or the rule is about a timer, task, listener or cache), a known leak pattern, a real container that is written by code that can run many times with a key from its caller and has no remover at the depth of the writer, minus weights for holders that are not containers, are bounded or weak, or are only filled at start-up; `dla ledger list` prints each row's score and its three largest contributions. Ties keep the tier (1: a DLA rule or known leak pattern, 2: a census holder with writers and no removers, 3: another detector hit, 4: a census holder with writers that also has removers), then a finding raised by more than one origin first, then the ledger's order; a finding in test, example, mock, sample, demo or benchmark code is demoted below all of them.";

/// Rule names (after `detector:dla/`) that are about a unit created per call or per object:
/// a timer, task, goroutine, thread, handle, listener registration, cache decorator or the
/// growth of a chain head. A hit of such a rule is strong evidence whatever the holder's
/// lifetime.
const UNIT_RULE_STEMS: &[&str] = &[
    "timer",
    "tick",
    "goroutine",
    "task",
    "thread",
    "listener",
    "registration",
    "cache-method",
    "unbounded-cache",
    "chain-head",
    "fopen",
    "malloc",
    "into-raw",
    "box-leak",
    "mem-forget",
    "rc-strong",
    "channel",
    "slice-view",
    "micrometer",
];

/// True when `rule` (the name after `detector:dla/`) is about a unit.
pub fn is_unit_rule(rule: &str) -> bool {
    UNIT_RULE_STEMS.iter().any(|s| rule.contains(s))
}

/// The lifetime of a holder: what the census recorded, else what the holder class says.
pub fn lifetime_of(f: &Finding, facts: Option<Lifetime>) -> Lifetime {
    facts.unwrap_or(match f.holder {
        Some(Holder::StaticGlobal) => Lifetime::Process,
        Some(Holder::ContainerField | Holder::RegistrationTable | Holder::ClosureContinuation) => {
            Lifetime::Object
        }
        _ => Lifetime::Unit,
    })
}

/// The sites the score reads: the structured ones the census recorded, and one unclassified
/// site for each plain `file:line` string that none of them covers (a site added by hand with
/// `dla ledger annotate`, or every site of a language the census does not read).
fn sites_of(
    structured: &[SiteFact],
    plain: &[String],
    kind: SiteKind,
    unit: bool,
) -> Vec<SiteFact> {
    let mut out = structured.to_vec();
    // What the census lists for a unit (a timer, a task) is what its function does to the
    // variable; the creation site is its one writer.
    let plain = if unit && !structured.is_empty() {
        &[]
    } else {
        plain
    };
    out.extend(
        plain
            .iter()
            .filter(|s| !structured.iter().any(|x| &x.site == *s))
            .map(|s| SiteFact {
                site: s.clone(),
                kind,
                depth: 0,
                function: None,
                init: false,
                startup: false,
                param: false,
                constant: false,
                exported: false,
            }),
    );
    out
}

/// Every signal that applies to a finding, with its weight, in the order of [`WEIGHTS`].
/// The score is their sum.
pub fn contributions(f: &Finding, annotation: Option<&Annotation>) -> Vec<(&'static str, i32)> {
    let mut out: Vec<(&'static str, i32)> = Vec::new();
    macro_rules! add {
        ($name:ident) => {
            out.push((stringify!($name), w::$name))
        };
    }
    let default = Annotation::default();
    let a = annotation.unwrap_or(&default);
    let facts = &a.facts;
    let origins = || {
        std::iter::once(&f.origin)
            .chain(&f.also_origins)
            .map(|o| o.as_str())
    };
    let rules: Vec<&str> = origins()
        .filter_map(|o| o.strip_prefix("detector:dla/"))
        .collect();
    let life = lifetime_of(f, facts.lifetime);
    let unit_entry = facts.unit;
    let unit_rule = rules.iter().any(|r| is_unit_rule(r));

    // Origin.
    if !rules.is_empty() {
        add!(rule_dla);
        if life == Lifetime::Process || unit_rule || unit_entry {
            add!(rule_strong);
        }
    }
    if origins().any(|o| o.starts_with("known-leak:")) {
        add!(known_leak);
    }
    if !f.also_origins.is_empty() {
        add!(multi_origin);
    }
    if tier(f, annotation) >= 5 {
        add!(tier5);
    }

    // The holder.
    match life {
        Lifetime::Process => add!(life_process),
        Lifetime::Object => add!(life_object),
        Lifetime::Unit => {}
    }
    let unbounded_cache = facts.bound == Some(Bound::UnboundedCache);
    match facts.container {
        _ if unit_entry => {}
        Some(ContainerClass::Container) => add!(container),
        _ if unbounded_cache => add!(container),
        Some(ContainerClass::Registry) => add!(registry),
        Some(ContainerClass::NotContainer) => add!(noncontainer),
        None => {}
    }
    match facts.bound {
        Some(Bound::WeakKeys | Bound::WeakValues) => add!(weak),
        Some(Bound::Bounded) => add!(bounded),
        Some(Bound::UnboundedCache) => add!(cache_unbounded),
        Some(Bound::None) | None => {}
    }
    if facts.literal {
        add!(literal_table);
    }
    if facts.key_bounded {
        add!(key_type_bounded);
    }
    if facts.pool {
        add!(pool_type);
    }

    // Writers.
    let writers = sites_of(
        &facts.writer_sites,
        &a.writers,
        SiteKind::Insert,
        unit_entry,
    );
    let mut writers = writers;
    if registration_is_own_writer(f, a)
        && let Some(site) = &f.site
    {
        writers.push(SiteFact {
            site: site.clone(),
            kind: SiteKind::Insert,
            depth: 0,
            function: None,
            init: false,
            startup: false,
            param: false,
            constant: false,
            exported: false,
        });
    }
    let removers = sites_of(&facts.remover_sites, &a.removers, SiteKind::Delete, false);
    let inserts: Vec<&SiteFact> = writers
        .iter()
        .filter(|s| s.kind.adds() && s.depth <= 1)
        .collect();
    // A rule hit is itself a claim that the holder grows with no removal.
    let rule_claims_growth = !rules.is_empty() && !unit_entry;
    // A cache decorator without a size inserts for every new argument tuple; a unit entry's
    // growth is its creation site, which the census lists as its one writer.
    let grows = !inserts.is_empty() || rule_claims_growth || unbounded_cache;
    if grows {
        add!(w_grow);
    }
    if writers.is_empty() && !unbounded_cache && !unit_entry {
        add!(w_none);
    }
    let reassign_only = !writers.is_empty()
        && writers
            .iter()
            .all(|s| matches!(s.kind, SiteKind::Reassign | SiteKind::Reset));
    if reassign_only {
        add!(w_reassign_only);
    }
    let runtime = unbounded_cache || inserts.iter().any(|s| !s.init && !s.startup);
    if runtime {
        add!(w_runtime);
    }
    if !inserts.is_empty() && !unbounded_cache && inserts.iter().all(|s| s.init || s.startup) {
        add!(w_startup_all);
    }
    let param = unbounded_cache || inserts.iter().any(|s| s.param);
    if param {
        add!(w_param_flow);
    } else if inserts.iter().any(|s| s.constant) {
        add!(w_const_only);
    }
    if inserts.iter().any(|s| !s.init && !s.startup && s.exported) {
        add!(w_exported);
    }

    // Removers: an initialiser's reset is the initial value, not a removal.
    let removing: Vec<&SiteFact> = removers
        .iter()
        .filter(|s| !(s.kind == SiteKind::Reset && s.init))
        .collect();
    let unmatched = unbounded_cache
        || inserts
            .iter()
            .any(|i| !removing.iter().any(|r| r.depth <= i.depth))
        || (rule_claims_growth && removing.is_empty());
    if unmatched {
        add!(growth_unmatched);
    }
    if removing.is_empty() {
        add!(r_none);
    }
    if removing.iter().any(|r| r.depth == 0) {
        add!(r_holder_level);
    }

    // Registrations and owners.
    match facts.receiver {
        Some(Receiver::Local) => add!(recv_local),
        Some(Receiver::Outer) => add!(recv_outer),
        None => {}
    }
    match facts.owner_creation {
        Some(OwnerCreation::PerCall) => add!(owner_percall),
        Some(OwnerCreation::StartUp) => add!(owner_startup),
        None => {}
    }
    out
}

/// The score of a finding: higher is measured first. A pure function of the finding and its
/// annotation.
pub fn score(f: &Finding, annotation: Option<&Annotation>) -> i32 {
    contributions(f, annotation).iter().map(|(_, p)| p).sum()
}

/// The `n` largest contributions to the score (by size, the earlier signal first on a tie),
/// as `(name, points)`.
pub fn why(f: &Finding, annotation: Option<&Annotation>, n: usize) -> Vec<(&'static str, i32)> {
    let mut c = contributions(f, annotation);
    // Stable: equal sizes keep the order of the table.
    c.sort_by_key(|(_, p)| std::cmp::Reverse(p.abs()));
    c.truncate(n);
    c
}

/// The owner path of a holder without its last segment (`a.b.Server` of `a.b.Server.updates`),
/// and the file of its site.
fn owner_scope(f: &Finding) -> Option<(String, String)> {
    let file = f.site.as_deref()?.rsplit_once(':')?.0.to_owned();
    let (path, _) = f.owner.as_deref()?.rsplit_once('.')?;
    (!path.is_empty()).then(|| (file, path.to_owned()))
}

/// True when two findings are state of one owner, so that a driver that exercises the owner
/// reaches both: the same file and the same owner path up to the last segment, and either
/// fields of an instance (the same struct or class), or module-level and static holders that
/// share a function that writes to them.
pub fn same_owner(a: (&Finding, Option<&Annotation>), b: (&Finding, Option<&Annotation>)) -> bool {
    let (Some(sa), Some(sb)) = (owner_scope(a.0), owner_scope(b.0)) else {
        return false;
    };
    if sa != sb || a.0.id == b.0.id {
        return false;
    }
    let life =
        |x: (&Finding, Option<&Annotation>)| lifetime_of(x.0, x.1.and_then(|a| a.facts.lifetime));
    match (life(a), life(b)) {
        (Lifetime::Object, Lifetime::Object) => true,
        (Lifetime::Process, Lifetime::Process) => {
            let fns = |x: (&Finding, Option<&Annotation>)| -> Vec<(String, String)> {
                x.1.map(|a| {
                    a.facts
                        .writer_sites
                        .iter()
                        .filter_map(|s| {
                            Some((s.site.rsplit_once(':')?.0.to_owned(), s.function.clone()?))
                        })
                        .collect()
                })
                .unwrap_or_default()
            };
            let fb = fns(b);
            fns(a).iter().any(|w| fb.contains(w))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        finding::{Finding, Origin},
        ids::FindingId,
    };

    fn finding(origin: &str, also: &[&str], holder: Option<Holder>, site: &str) -> Finding {
        let mut f = Finding::candidate(FindingId::new(1), Origin::parse(origin).unwrap(), vec![]);
        f.also_origins = also.iter().map(|o| Origin::parse(o).unwrap()).collect();
        f.holder = holder;
        f.site = Some(site.to_owned());
        f
    }

    fn ann(writers: usize, removers: usize) -> Annotation {
        Annotation {
            writers: (0..writers).map(|i| format!("a.py:{}", i + 1)).collect(),
            removers: (0..removers).map(|i| format!("a.py:{}", i + 50)).collect(),
            ..Annotation::default()
        }
    }

    #[test]
    fn tier_one_is_a_known_leak_or_a_dla_rule() {
        let f = finding("known-leak:abc", &[], None, "a.py:1");
        assert_eq!(tier(&f, None), 1);
        let f = finding("detector:dla/python-unbounded-cache", &[], None, "a.py:1");
        assert_eq!(tier(&f, None), 1);
        // A DLA rule joined to a census entry is tier 1, not 2.
        let f = finding(
            "census",
            &["detector:dla/python-cache-method"],
            Some(Holder::StaticGlobal),
            "a.py:1",
        );
        assert_eq!(tier(&f, Some(&ann(1, 0))), 1);
    }

    #[test]
    fn tier_two_is_a_census_holder_with_writers_and_no_removers() {
        for h in [
            Holder::RegistrationTable,
            Holder::ContainerField,
            Holder::StaticGlobal,
        ] {
            let f = finding("census", &[], Some(h), "a.py:1");
            assert_eq!(tier(&f, Some(&ann(2, 0))), 2);
        }
        // No writers, or a removers, or another holder kind: not tier 2.
        let f = finding("census", &[], Some(Holder::StaticGlobal), "a.py:1");
        assert_eq!(tier(&f, Some(&ann(0, 0))), 5);
        assert_eq!(tier(&f, None), 5);
        assert_eq!(tier(&f, Some(&ann(1, 1))), 4);
        let f = finding("census", &[], Some(Holder::ThreadLocal), "a.py:1");
        assert_eq!(tier(&f, Some(&ann(1, 0))), 4);
        // The census may be the second origin.
        let f = finding(
            "detector:semgrep/x",
            &["census"],
            Some(Holder::ContainerField),
            "a.py:1",
        );
        assert_eq!(tier(&f, Some(&ann(1, 0))), 2);
    }

    #[test]
    fn tier_three_is_any_other_detector_and_beats_four() {
        let f = finding("detector:semgrep/leaky", &[], None, "a.py:1");
        assert_eq!(tier(&f, None), 3);
        let f = finding(
            "detector:semgrep/leaky",
            &["census"],
            Some(Holder::ThreadLocal),
            "a.py:1",
        );
        assert_eq!(tier(&f, Some(&ann(2, 1))), 3);
    }

    #[test]
    fn tier_four_and_five() {
        let f = finding("census", &[], Some(Holder::ClosureContinuation), "a.py:1");
        assert_eq!(tier(&f, Some(&ann(3, 0))), 4);
        let f = finding("growth-series", &[], None, "a.py:1");
        assert_eq!(tier(&f, None), 5);
        let f = finding("hypothesis", &[], None, "a.py:1");
        assert_eq!(tier(&f, Some(&ann(3, 0))), 5);
        assert!(eligible_tier(4) && !eligible_tier(5) && !eligible_tier(6));
    }

    #[test]
    fn test_example_benchmark_and_doc_sites_are_demoted_whatever_else_holds() {
        for site in [
            "tests/conftest.py:3",
            "pkg/tests/util.py:3",
            "src/test/java/a/B.java:9",
            "docs/conf.py:1",
            "examples/app.py:1",
            "bench/run.go:4",
            "benchmarks/x.py:2",
            "pkg/testdata/x.go:2",
            "web/__tests__/a.js:2",
            "pkg/spec/a.rb:2",
            "pkg/cache_test.go:5",
            "pkg/test_cache.py:5",
            "pkg/cache_test.py:5",
            "web/a.test.js:1",
            "web/a.spec.ts:1",
            "src/FooTest.java:1",
            "src/FooTests.java:1",
            "src/FooTest.kt:1",
            "conftest.py:1",
            "Tests\\a.py:1",
        ] {
            let f = finding("known-leak:x", &[], Some(Holder::StaticGlobal), site);
            assert_eq!(tier(&f, Some(&ann(2, 0))), 6, "{site}");
        }
        for site in [
            "pkg/cache.py:5",
            "src/contest.py:1",
            "pkg/latest/a.py:1",
            "a.py",
        ] {
            let f = finding("census", &[], Some(Holder::StaticGlobal), site);
            assert_eq!(tier(&f, Some(&ann(2, 0))), 2, "{site}");
        }
    }

    #[test]
    fn multi_origin_sorts_first() {
        let one = finding("census", &[], None, "a.py:1");
        let two = finding("census", &["detector:x/y"], None, "a.py:1");
        assert!(multi_origin_key(&two) < multi_origin_key(&one));
    }

    // ----- the score -----

    use crate::census::{
        Bound, ContainerClass, HolderFacts, Lifetime, OwnerCreation, Receiver, SiteFact, SiteKind,
    };

    fn site(file_line: &str, kind: SiteKind, depth: u8, function: &str) -> SiteFact {
        SiteFact {
            site: file_line.to_owned(),
            kind,
            depth,
            function: Some(function.to_owned()),
            init: false,
            startup: false,
            param: false,
            constant: false,
            exported: false,
        }
    }

    /// A listener added to an object handed in from outside: nothing else "writes" it, the
    /// registration call is the insert; the receiver says whether the object can outlive the call.
    fn own_registration(receiver: Receiver) -> (Finding, Annotation) {
        let f = finding(
            "census",
            &[],
            Some(Holder::RegistrationTable),
            "lib/adapters/http.js:924",
        );
        let a = facts_ann(|h| {
            h.container = Some(ContainerClass::Registry);
            h.bound = Some(Bound::None);
            h.lifetime = Some(Lifetime::Object);
            h.receiver = Some(receiver);
            h.remover_sites = vec![site("lib/adapters/http.js:913", SiteKind::Delete, 0, "httpAdapter")];
        });
        (f, a)
    }

    #[test]
    fn a_registration_on_an_outer_object_is_its_own_writer_and_ranks_above_a_local_one() {
        let (f, a) = own_registration(Receiver::Outer);
        // Eligible (tier 4: it has a remover), not tier 5, and no w_none.
        assert_eq!(tier(&f, Some(&a)), 4);
        let names: Vec<&str> = contributions(&f, Some(&a)).iter().map(|c| c.0).collect();
        assert!(!names.contains(&"w_none") && !names.contains(&"tier5"), "{names:?}");
        assert!(names.contains(&"w_grow"), "{names:?}");
        let (fl, al) = own_registration(Receiver::Local);
        assert!(score(&f, Some(&a)) > score(&fl, Some(&al)));
        assert!(score(&f, Some(&a)) > 0, "{}", score(&f, Some(&a)));
        // Without a remover it is tier 2, as any registration with a writer.
        let mut free = a.clone();
        free.facts.remover_sites.clear();
        free.removers.clear();
        assert_eq!(tier(&f, Some(&free)), 2);
        // A registration of another holder kind, or one that has a writer of its own elsewhere,
        // is unchanged.
        let g = finding("census", &[], Some(Holder::StaticGlobal), "a.js:1");
        assert_eq!(tier(&g, Some(&a)), 5);
    }

    fn facts_ann(f: impl FnOnce(&mut HolderFacts)) -> Annotation {
        let mut a = Annotation::default();
        f(&mut a.facts);
        a.writers = a
            .facts
            .writer_sites
            .iter()
            .map(|s| s.site.clone())
            .collect();
        a.removers = a
            .facts
            .remover_sites
            .iter()
            .map(|s| s.site.clone())
            .collect();
        a
    }

    fn census_holder(holder: Holder) -> Finding {
        finding("census", &[], Some(holder), "pkg/a.py:1")
    }

    fn names(f: &Finding, a: &Annotation) -> Vec<&'static str> {
        contributions(f, Some(a))
            .into_iter()
            .map(|(n, _)| n)
            .collect()
    }

    fn weight(name: &str) -> i32 {
        WEIGHTS.iter().find(|(n, _)| *n == name).unwrap().1
    }

    #[test]
    fn every_signal_has_one_named_weight_and_the_score_is_their_sum() {
        let mut seen = std::collections::BTreeSet::new();
        for (n, _) in WEIGHTS {
            assert!(seen.insert(*n), "{n} twice");
        }
        // The three changes from the research table, and a few that must keep their size.
        assert_eq!(
            [
                weight("rule_dla"),
                weight("rule_strong"),
                weight("life_process")
            ],
            [15, 45, 20]
        );
        assert_eq!(weight("noncontainer"), -80);
        let f = finding(
            "detector:dla/jvm-unbounded-collection",
            &[],
            Some(Holder::StaticGlobal),
            "pkg/a.py:1",
        );
        let a = facts_ann(|h| {
            h.container = Some(ContainerClass::Container);
            let mut w = site("pkg/a.py:5", SiteKind::Insert, 0, "put");
            w.param = true;
            h.writer_sites = vec![w];
        });
        let c = contributions(&f, Some(&a));
        for (n, p) in &c {
            assert_eq!(*p, weight(n), "{n}");
        }
        assert_eq!(score(&f, Some(&a)), c.iter().map(|(_, p)| p).sum::<i32>());
        // The three largest contributions, by size.
        let why = why(&f, Some(&a), 3);
        assert_eq!(why.len(), 3);
        assert!(
            why.windows(2).all(|w| w[0].1.abs() >= w[1].1.abs()),
            "{why:?}"
        );
    }

    #[test]
    fn each_signal_moves_the_score_the_way_its_sign_says() {
        let base = || census_holder(Holder::ContainerField);
        let ins = |init: bool| {
            let mut w = site("pkg/a.py:5", SiteKind::Insert, 0, "put");
            w.init = init;
            w
        };
        let cases: Vec<(&str, Finding, Annotation)> = vec![
            (
                "rule_dla",
                finding(
                    "detector:dla/jvm-unbounded-collection",
                    &[],
                    Some(Holder::ContainerField),
                    "pkg/a.py:1",
                ),
                Annotation::default(),
            ),
            (
                "rule_strong",
                finding(
                    "detector:dla/go-ticker-no-stop",
                    &[],
                    Some(Holder::ContainerField),
                    "pkg/a.py:1",
                ),
                Annotation::default(),
            ),
            (
                "known_leak",
                finding("known-leak:x", &[], None, "pkg/a.py:1"),
                Annotation::default(),
            ),
            (
                "multi_origin",
                finding("census", &["detector:x/y"], None, "pkg/a.py:1"),
                Annotation::default(),
            ),
            (
                "tier5",
                finding("growth-series", &[], None, "pkg/a.py:1"),
                Annotation::default(),
            ),
            (
                "life_process",
                census_holder(Holder::StaticGlobal),
                Annotation::default(),
            ),
            ("life_object", base(), Annotation::default()),
            (
                "container",
                base(),
                facts_ann(|h| h.container = Some(ContainerClass::Container)),
            ),
            (
                "registry",
                base(),
                facts_ann(|h| h.container = Some(ContainerClass::Registry)),
            ),
            (
                "noncontainer",
                base(),
                facts_ann(|h| h.container = Some(ContainerClass::NotContainer)),
            ),
            (
                "weak",
                base(),
                facts_ann(|h| h.bound = Some(Bound::WeakKeys)),
            ),
            (
                "bounded",
                base(),
                facts_ann(|h| h.bound = Some(Bound::Bounded)),
            ),
            (
                "cache_unbounded",
                base(),
                facts_ann(|h| h.bound = Some(Bound::UnboundedCache)),
            ),
            ("literal_table", base(), facts_ann(|h| h.literal = true)),
            (
                "key_type_bounded",
                base(),
                facts_ann(|h| h.key_bounded = true),
            ),
            ("pool_type", base(), facts_ann(|h| h.pool = true)),
            (
                "w_grow",
                base(),
                facts_ann(|h| h.writer_sites = vec![ins(false)]),
            ),
            ("w_none", base(), Annotation::default()),
            (
                "w_reassign_only",
                base(),
                facts_ann(|h| {
                    h.writer_sites = vec![site("pkg/a.py:5", SiteKind::Reassign, 0, "set")]
                }),
            ),
            (
                "w_runtime",
                base(),
                facts_ann(|h| h.writer_sites = vec![ins(false)]),
            ),
            (
                "w_startup_all",
                base(),
                facts_ann(|h| h.writer_sites = vec![ins(true)]),
            ),
            (
                "w_param_flow",
                base(),
                facts_ann(|h| {
                    let mut w = ins(false);
                    w.param = true;
                    h.writer_sites = vec![w];
                }),
            ),
            (
                "w_const_only",
                base(),
                facts_ann(|h| {
                    let mut w = ins(false);
                    w.constant = true;
                    h.writer_sites = vec![w];
                }),
            ),
            (
                "w_exported",
                base(),
                facts_ann(|h| {
                    let mut w = ins(false);
                    w.exported = true;
                    h.writer_sites = vec![w];
                }),
            ),
            (
                "growth_unmatched",
                base(),
                facts_ann(|h| h.writer_sites = vec![ins(false)]),
            ),
            ("r_none", base(), Annotation::default()),
            (
                "r_holder_level",
                base(),
                facts_ann(|h| {
                    h.remover_sites = vec![site("pkg/a.py:9", SiteKind::Delete, 0, "drop")]
                }),
            ),
            (
                "recv_local",
                base(),
                facts_ann(|h| h.receiver = Some(Receiver::Local)),
            ),
            (
                "recv_outer",
                base(),
                facts_ann(|h| h.receiver = Some(Receiver::Outer)),
            ),
            (
                "owner_percall",
                base(),
                facts_ann(|h| h.owner_creation = Some(OwnerCreation::PerCall)),
            ),
            (
                "owner_startup",
                base(),
                facts_ann(|h| h.owner_creation = Some(OwnerCreation::StartUp)),
            ),
        ];
        for (name, f, a) in cases {
            let c = contributions(&f, Some(&a));
            let got = c
                .iter()
                .find(|(n, _)| *n == name)
                .unwrap_or_else(|| panic!("{name} missing in {c:?}"));
            assert_eq!(got.1, weight(name), "{name}");
            assert!(WEIGHTS.iter().any(|(n, _)| *n == name));
        }
        // A holder with no census facts and no writers is not penalised for facts it lacks.
        assert!(!names(&base(), &Annotation::default()).contains(&"noncontainer"));
    }

    /// A static keyed cache whose key comes from a parameter of a public function, with no
    /// remover; a rule hit on an instance field written only by `register*` methods; an outer
    /// map whose entries are only deleted one level down; a map with a real remover; a package
    /// variable filled once in `init`.
    fn shapes() -> [(Finding, Annotation); 5] {
        let mut keyed = site("pkg/c.py:10", SiteKind::Insert, 0, "remember");
        keyed.param = true;
        keyed.exported = true;
        let cache = (
            census_holder(Holder::StaticGlobal),
            facts_ann(|h| {
                h.container = Some(ContainerClass::Container);
                h.lifetime = Some(Lifetime::Process);
                h.writer_sites = vec![keyed.clone()];
            }),
        );
        let mut reg = site("pkg/r.py:20", SiteKind::Insert, 0, "register_listener");
        reg.param = true;
        let rule_field = (
            finding(
                "detector:dla/jvm-unbounded-collection",
                &[],
                Some(Holder::ContainerField),
                "pkg/r.py:3",
            ),
            facts_ann(|h| {
                h.container = Some(ContainerClass::Container);
                h.lifetime = Some(Lifetime::Object);
                h.writer_sites = vec![reg.clone()];
            }),
        );
        let mut outer_ins = site("pkg/u.py:10", SiteKind::Insert, 0, "watch");
        outer_ins.param = true;
        let outer_only = (
            census_holder(Holder::ContainerField),
            facts_ann(|h| {
                h.container = Some(ContainerClass::Container);
                h.lifetime = Some(Lifetime::Object);
                h.writer_sites = vec![outer_ins.clone()];
                h.remover_sites = vec![site("pkg/u.py:30", SiteKind::Delete, 1, "done")];
            }),
        );
        let with_remover = (
            census_holder(Holder::ContainerField),
            facts_ann(|h| {
                h.container = Some(ContainerClass::Container);
                h.lifetime = Some(Lifetime::Object);
                h.writer_sites = vec![outer_ins.clone()];
                h.remover_sites = vec![site("pkg/u.py:30", SiteKind::Delete, 0, "done")];
            }),
        );
        let mut once = site("pkg/i.py:4", SiteKind::Insert, 0, "init");
        once.init = true;
        once.startup = true;
        once.constant = true;
        let in_init = (
            census_holder(Holder::StaticGlobal),
            facts_ann(|h| {
                h.container = Some(ContainerClass::Container);
                h.lifetime = Some(Lifetime::Process);
                h.writer_sites = vec![once.clone()];
            }),
        );
        [cache, rule_field, outer_only, with_remover, in_init]
    }

    #[test]
    fn a_static_keyed_cache_with_parameter_keys_outranks_a_rule_hit_on_a_register_only_field() {
        let [cache, rule_field, ..] = shapes();
        assert!(score(&cache.0, Some(&cache.1)) > score(&rule_field.0, Some(&rule_field.1)));
        // The rule hit alone, with the same holder, would have been ahead by tier.
        assert_eq!(tier(&rule_field.0, Some(&rule_field.1)), 1);
        assert_eq!(tier(&cache.0, Some(&cache.1)), 2);
    }

    #[test]
    fn an_outer_map_with_element_level_deletes_only_outranks_a_map_with_a_real_remover() {
        let [_, _, outer_only, with_remover, _] = shapes();
        let a = score(&outer_only.0, Some(&outer_only.1));
        let b = score(&with_remover.0, Some(&with_remover.1));
        assert!(a > b, "{a} {b}");
        assert!(names(&outer_only.0, &outer_only.1).contains(&"growth_unmatched"));
        assert!(!names(&with_remover.0, &with_remover.1).contains(&"growth_unmatched"));
    }

    #[test]
    fn a_package_variable_assigned_once_in_init_ranks_below_both() {
        let [cache, rule_field, outer_only, with_remover, in_init] = shapes();
        let init = score(&in_init.0, Some(&in_init.1));
        for (f, a) in [&cache, &rule_field, &outer_only, &with_remover] {
            assert!(init < score(f, Some(a)), "{init}");
        }
        let n = names(&in_init.0, &in_init.1);
        assert!(
            n.contains(&"w_startup_all")
                && n.contains(&"w_const_only")
                && !n.contains(&"w_runtime"),
            "{n:?}"
        );
    }

    #[test]
    fn a_rule_hit_without_census_facts_keeps_a_neutral_score_and_old_annotations_still_count() {
        let f = finding(
            "detector:dla/jvm-unbounded-collection",
            &[],
            None,
            "pkg/a.py:1",
        );
        assert!(score(&f, None) > 0);
        // An annotation made by hand: writers and removers as plain `file:line` strings.
        let by_hand = ann(1, 0);
        let c = census_holder(Holder::ContainerField);
        let n = names(&c, &by_hand);
        assert!(
            n.contains(&"w_grow") && n.contains(&"r_none") && !n.contains(&"w_none"),
            "{n:?}"
        );
    }

    #[test]
    fn test_support_example_and_demo_paths_are_demoted_and_look_alikes_are_not() {
        for site in [
            "pkg/testutils/clock.go:3",
            "pkg/internal/testutil/x.go:3",
            "src/main/java/a/MockStore.java:3",
            "pkg/fake_server.py:3",
            "pkg/stub/a.py:3",
            "pkg/e2e/a.py:3",
            "pkg/fixtures/a.py:3",
            "interop/a.go:3",
            "pkg/sample/a.py:3",
            "pkg/demo_app.py:3",
            "pkg/benchmark/a.py:3",
        ] {
            let f = finding("known-leak:x", &[], Some(Holder::StaticGlobal), site);
            assert_eq!(tier(&f, Some(&ann(2, 0))), 6, "{site}");
        }
        for site in [
            "pkg/sampler.go:3",
            "pkg/contest/a.py:3",
            "pkg/attestation/a.py:3",
            "pkg/stubborn.py:3",
            "pkg/demonstrate.py:3",
        ] {
            let f = finding("census", &[], Some(Holder::StaticGlobal), site);
            assert_eq!(tier(&f, Some(&ann(2, 0))), 2, "{site}");
        }
    }

    #[test]
    fn a_writer_added_by_hand_next_to_census_sites_counts_as_an_unclassified_insert() {
        let mut once = site("pkg/a.py:4", SiteKind::Insert, 0, "init");
        once.init = true;
        once.startup = true;
        let mut a = facts_ann(|h| h.writer_sites = vec![once.clone()]);
        let f = census_holder(Holder::ContainerField);
        assert!(!names(&f, &a).contains(&"w_runtime"));
        // `dla ledger annotate F1 --writer pkg/a.py:90`: no census fact says where it runs.
        a.writers.push("pkg/a.py:90".to_owned());
        let n = names(&f, &a);
        assert!(
            n.contains(&"w_runtime") && !n.contains(&"w_startup_all"),
            "{n:?}"
        );
    }
}
