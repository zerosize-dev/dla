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

use crate::{
    finding::{Finding, Holder},
    ledger::Annotation,
};

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
    parts
        .iter()
        .any(|d| NON_PRODUCT_DIRS.contains(&d.to_ascii_lowercase().as_str()))
        || is_test_file(file)
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
    let (writers, removers) = annotation.map_or((0, 0), |a| (a.writers.len(), a.removers.len()));
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
pub const TIER_NOTE: &str = "Suspected, static-only findings are ordered by how likely a read-only signal makes a leak, strongest first, and keep the ledger's order within a tier (a finding raised by more than one origin first): (1) a DLA rule or a known leak pattern, (2) a census holder with writers and no removers in a registration table, container field or static/global, (3) another detector hit, (4) a census holder with writers that also has removers or another holder kind, (5) everything else, such as a holder with no writers. A finding in test, example, benchmark or documentation code is demoted below all five.";

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
}
