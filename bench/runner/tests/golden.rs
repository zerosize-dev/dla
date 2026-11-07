//! The synthetic sample under testdata/rust-sample, graded and compared to the
//! golden metrics that were checked by hand.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dla_bench::{compare, grade};

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/rust-sample")
}

#[test]
fn sample_matches_the_golden_file() {
    let s = sample();
    let m = grade::grade_dir(&s.join("results"), &s.join("fixtures")).expect("grades");
    let got = serde_json::from_str(&grade::render(&m)).unwrap();
    let want: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(s.join("expected-metrics.json")).unwrap())
            .unwrap();
    let diff = compare::diff(&got, &want);
    assert!(diff.is_empty(), "differences: {diff:#?}");
}

#[test]
fn hand_checked_numbers() {
    let s = sample();
    let m = grade::grade_dir(&s.join("results"), &s.join("fixtures")).unwrap();
    let names: Vec<(&str, &str)> = m
        .groups
        .iter()
        .map(|g| (g.ablation.as_str(), g.archetypes.as_str()))
        .collect();
    assert_eq!(
        names,
        [("full", "on"), ("gates", "off"), ("no-skill", "on")]
    );
    let ns = &m.groups[2];
    let r = &ns.metrics.leak_recall;
    assert_eq!((r.num, r.den, r.value), (7, 12, Some(0.583333)));
    let per: Vec<_> = r.per_run.iter().map(|p| (p.num, p.den)).collect();
    assert_eq!(per, [(3, 4), (2, 4), (2, 4)]);
    assert_eq!((r.mean, r.stdev), (Some(0.583333), Some(0.117851)));
    // 1/128 tie in the usage block of the "full" group
    let full = &m.groups[0];
    assert_eq!(full.usage.wall_seconds.mean, Some(0.007812));
    assert_eq!(full.usage.wall_seconds.sum, 0.078125);
    assert_eq!(full.unexplained_growth_share.runs, 2);
    assert_eq!(full.unexplained_growth_share.value, Some(0.278689));
    // sparse run index: entries for runs without data are present and null
    let g = &m.groups[1].metrics.leak_recall;
    assert_eq!(g.per_run.len(), 3);
    assert_eq!(g.per_run[1].value, None);
}

fn write(p: &Path, text: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

const RUN_JSON: &str =
    r#"{"exit_code":0,"timed_out":false,"wall_seconds":1.0,"cost_usd":null,"usage":{}}"#;

#[test]
fn run_without_a_fixture_is_an_error() {
    let t = tempfile::tempdir().unwrap();
    let fx = t.path().join("fx");
    fs::create_dir_all(&fx).unwrap();
    write(
        &t.path()
            .join("res/runs/no-skill/archetypes-on/ghost/1/run.json"),
        RUN_JSON,
    );
    let e = grade::grade_dir(&t.path().join("res"), &fx).unwrap_err();
    assert!(e.contains("ghost"), "{e}");
}

#[test]
fn run_without_run_json_is_incomplete_not_an_error() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let base = t.path().join("res/runs/no-skill/archetypes-on/fx-trap");
    write(&base.join("1/findings.json"), "{}");
    write(&base.join("2/run.json"), RUN_JSON);
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    assert_eq!(m.groups.len(), 1);
    assert_eq!(m.groups[0].usage.runs, 1);
    assert_eq!(m.groups[0].usage.incomplete, 1);
    // the incomplete run index does not extend per_run
    assert_eq!(m.groups[0].metrics.leak_recall.per_run.len(), 2);
}

#[test]
fn run_json_that_is_not_an_object_is_an_error() {
    let s = sample();
    for bad in ["[1]", "3", "null", "not json", ""] {
        let t = tempfile::tempdir().unwrap();
        write(
            &t.path()
                .join("res/runs/no-skill/archetypes-on/fx-trap/1/run.json"),
            bad,
        );
        let e = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap_err();
        assert!(e.contains("run.json"), "{bad}: {e}");
    }
}

#[test]
fn unrelated_directories_are_ignored() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let r = t.path().join("res/runs/no-skill");
    write(&r.join("archetypes-on/fx-trap/1/run.json"), RUN_JSON);
    let mut names = vec!["0", "01", "+1", "1.0", "-1", "a"];
    // Windows refuses directory names that end in a space.
    if !cfg!(windows) {
        names.extend([" 1", "1 "]);
    }
    for name in names {
        write(
            &r.join(format!("archetypes-on/fx-trap/{name}/run.json")),
            RUN_JSON,
        );
        // would be an unknown fixture if looked at
        write(&r.join("archetypes-other/ghost/1/run.json"), RUN_JSON);
        write(&r.join("archetypes-on-2/ghost/1/run.json"), RUN_JSON);
        // Names must differ from real ones by more than letter case: macOS
        // filesystems are case-insensitive, so `Archetypes-On` would be the
        // real `archetypes-on` directory there.
        write(&r.join("archetypes-on-x/ghost/1/run.json"), RUN_JSON);
        write(&r.join("xarchetypes-on/ghost/1/run.json"), RUN_JSON);
    }
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    assert_eq!(m.groups.len(), 1);
    assert_eq!(m.groups[0].usage.runs, 1);
    assert_eq!(m.groups[0].usage.incomplete, 0);
}

#[test]
fn run_json_field_leniency_and_errored() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let base = t.path().join("res/runs/no-skill/archetypes-on/fx-trap");
    let runs = [
        // errored: exit_code missing
        r#"{"timed_out":false,"wall_seconds":"x","cost_usd":"1.5","usage":{"input_tokens":1.5,"output_tokens":"7"}}"#,
        // errored: exit_code not an integer
        r#"{"exit_code":0.0,"timed_out":false,"wall_seconds":2.5,"cost_usd":0.5,"usage":{"input_tokens":10,"output_tokens":3}}"#,
        // not errored
        r#"{"exit_code":0,"timed_out":false,"wall_seconds":1,"cost_usd":null,"usage":{"input_tokens":5}}"#,
        // errored: timed out
        r#"{"exit_code":0,"timed_out":true,"wall_seconds":4,"usage":{"cache_read_input_tokens":2}}"#,
    ];
    for (i, r) in runs.iter().enumerate() {
        write(&base.join(format!("{}/run.json", i + 1)), r);
    }
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    let u = &m.groups[0].usage;
    assert_eq!((u.runs, u.errored), (4, 3));
    assert_eq!((u.input_tokens.sum, u.output_tokens.sum), (15, 3));
    assert_eq!(u.cache_read_input_tokens.sum, 2);
    assert_eq!(u.wall_seconds.sum, 7.5);
    assert_eq!(u.wall_seconds.max, Some(4.0));
    assert_eq!((u.cost_usd.sum, u.cost_usd.mean), (0.5, Some(0.5)));
}

#[test]
fn mean_and_stdev_use_unrounded_ratios() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let base = t.path().join("res/runs/no-skill/archetypes-on");
    let fj = |sites: &[&str]| {
        let items: Vec<String> = sites
            .iter()
            .map(|x| format!(r#"{{"site":"{x}","verdict":"leak-confirmed"}}"#))
            .collect();
        format!(r#"{{"findings":[{}]}}"#, items.join(","))
    };
    // fx-multi has 2 leaks and fx-free 1: 3 leaks per run.
    // Run 1 finds one of them (1/3), run 2 finds all (3/3), run 3 none (0/3).
    let runs: [&[&str]; 3] = [
        &["a/reg.js:5"],
        &["a/reg.js:5", "b/parse.js:40", "lib/chain.go:30"],
        &[],
    ];
    for (i, sites) in runs.iter().enumerate() {
        let n = i + 1;
        for fx in ["fx-multi", "fx-free"] {
            write(&base.join(format!("{fx}/{n}/run.json")), RUN_JSON);
            let own: Vec<&str> = sites
                .iter()
                .copied()
                .filter(|x| (fx == "fx-free") == x.starts_with("lib/"))
                .collect();
            write(&base.join(format!("{fx}/{n}/findings.json")), &fj(&own));
        }
    }
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    let r = &m.groups[0].metrics.leak_recall;
    let per: Vec<_> = r.per_run.iter().map(|p| (p.num, p.den)).collect();
    assert_eq!(per, [(1, 3), (3, 3), (0, 3)]);
    // unrounded ratios 1/3, 1, 0: mean 4/9 = 0.444444; the squared deviations
    // from 4/9 are 1/81, 25/81, 16/81, so stdev = sqrt(42/243) = 0.415740
    assert_eq!((r.mean, r.stdev), (Some(0.444444), Some(0.415740)));
}

#[test]
fn runs_with_den_zero_are_left_out_of_mean_and_stdev() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let base = t.path().join("res/runs/no-skill/archetypes-on/fx-multi");
    write(&base.join("1/run.json"), RUN_JSON);
    write(&base.join("3/run.json"), RUN_JSON);
    write(
        &base.join("3/findings.json"),
        r#"{"findings":[{"site":"a/reg.js:5","verdict":"leak-confirmed"}]}"#,
    );
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    // overclaim_rate has no runs of its category: every den is 0
    let o = &m.groups[0].metrics.overclaim_rate;
    assert_eq!((o.mean, o.stdev, o.value), (None, None, None));
    let r = &m.groups[0].metrics.leak_recall;
    assert_eq!(r.per_run.len(), 3);
    assert_eq!(r.per_run[1].den, 0);
    // ratios 0/2 and 1/2: mean 0.25, stdev 0.25
    assert_eq!((r.mean, r.stdev), (Some(0.25), Some(0.25)));
}

#[test]
fn unexplained_growth_rules() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let base = t.path().join("res/runs/no-skill/archetypes-on/fx-multi");
    let cov = [
        // counts: residual 1+0 (missing counts 0), slope 2+4; the non-object row is skipped
        r#"{"residual":[{"residual":1,"total_slope":2},{"total_slope":4},7,"x"]}"#,
        // counts: non-numeric values count as 0, run counted
        r#"{"residual":[{"residual":"a","total_slope":null}]}"#,
        // absent: empty residual, not an object, invalid JSON, residual not an array
        r#"{"residual":[]}"#,
        r#"[{"residual":[{"residual":9,"total_slope":9}]}]"#,
        "not json",
        r#"{"residual":{"residual":1}}"#,
    ];
    for (i, c) in cov.iter().enumerate() {
        write(&base.join(format!("{}/run.json", i + 1)), RUN_JSON);
        write(&base.join(format!("{}/coverage.json", i + 1)), c);
    }
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    let u = &m.groups[0].unexplained_growth_share;
    // R = 1, T = 6 over k = 2 runs
    assert_eq!((u.runs, u.value), (2, Some(0.166667)));
    // only the all-zero run: T = 0 gives a null value
    let t2 = tempfile::tempdir().unwrap();
    let b2 = t2.path().join("res/runs/no-skill/archetypes-on/fx-multi/1");
    write(&b2.join("run.json"), RUN_JSON);
    write(&b2.join("coverage.json"), cov[1]);
    let m = grade::grade_dir(&t2.path().join("res"), &s.join("fixtures")).unwrap();
    let u = &m.groups[0].unexplained_growth_share;
    assert_eq!((u.runs, u.value), (1, None));
}

#[test]
fn per_fixture_leaks_include_unconfirmable_leaks() {
    let t = tempfile::tempdir().unwrap();
    let fx = t.path().join("fx/u");
    write(
        &fx.join("expected.toml"),
        r#"
schema_version = 1
id = "u"
runtime = "node"
category = "archetype-leak"
dynamic_confirmation = false
[task]
prompt = "p"
[[leak]]
id = "L1"
site = "a.js:1"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
confirmable = false
"#,
    );
    let run = t.path().join("res/runs/no-skill/archetypes-on/u/1");
    write(&run.join("run.json"), RUN_JSON);
    write(
        &run.join("findings.json"),
        r#"{"findings":[{"site":"a.js:1","holder":"static/global","verdict":"leak-confirmed"}]}"#,
    );
    let m = grade::grade_dir(&t.path().join("res"), &t.path().join("fx")).unwrap();
    let g = &m.groups[0];
    let c = &g.fixtures["u"].leaks["L1"];
    assert_eq!((c.found, c.flagged, c.grounded), (1, 1, 1));
    assert_eq!(g.metrics.leak_recall.den, 0, "not in the leak set");
}

#[test]
fn results_without_runs_directory_is_an_error() {
    let t = tempfile::tempdir().unwrap();
    assert!(grade::grade_dir(t.path(), t.path()).is_err());
}

#[test]
fn empty_runs_directory_gives_no_groups() {
    let t = tempfile::tempdir().unwrap();
    fs::create_dir_all(t.path().join("runs")).unwrap();
    let m = grade::grade_dir(t.path(), t.path()).unwrap();
    assert!(m.groups.is_empty());
}

#[test]
fn metrics_file_ends_with_a_newline_and_round_trips() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let out = t.path().join("metrics.json");
    grade::grade_to_file(&s.join("results"), &s.join("fixtures"), &out).unwrap();
    let text = fs::read_to_string(&out).unwrap();
    assert!(text.ends_with("}\n"));
    let back: dla_bench::model::MetricsFile = serde_json::from_str(&text).unwrap();
    assert_eq!(back.groups.len(), 3);
}

// ---- the clarified rules of GRADING.md: encoding, directories, empty groups

#[test]
fn run_json_that_is_not_utf8_is_an_error() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let p = t
        .path()
        .join("res/runs/no-skill/archetypes-on/fx-trap/1/run.json");
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(&p, b"{\"exit_code\":0,\"note\":\"caf\xe9\"}").unwrap();
    let e = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap_err();
    assert!(e.contains("UTF-8") && e.contains("run.json"), "{e}");
}

#[test]
fn expected_toml_that_is_not_utf8_is_an_error() {
    let t = tempfile::tempdir().unwrap();
    write(
        &t.path()
            .join("res/runs/no-skill/archetypes-on/bad/1/run.json"),
        RUN_JSON,
    );
    let fx = t.path().join("fx/bad");
    fs::create_dir_all(&fx).unwrap();
    fs::write(fx.join("expected.toml"), b"id = \"caf\xe9\"\n").unwrap();
    let e = grade::grade_dir(&t.path().join("res"), &t.path().join("fx")).unwrap_err();
    assert!(e.contains("UTF-8") && e.contains("expected.toml"), "{e}");
}

#[test]
fn findings_and_coverage_that_are_not_utf8_count_as_absent() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let base = t.path().join("res/runs/no-skill/archetypes-on/fx-multi");
    write(&base.join("1/run.json"), RUN_JSON);
    fs::write(
        base.join("1/findings.json"),
        b"{\"findings\":[{\"site\":\"a/reg.js:5\",\"verdict\":\"leak-confirmed\",\"evidence\":\"caf\xe9\"}]}",
    )
    .unwrap();
    fs::write(
        base.join("1/coverage.json"),
        b"{\"residual\":[{\"residual\":1,\"total_slope\":2,\"note\":\"\xe9\"}]}",
    )
    .unwrap();
    // the same files in valid UTF-8 are used
    write(&base.join("2/run.json"), RUN_JSON);
    write(
        &base.join("2/findings.json"),
        r#"{"findings":[{"site":"a/reg.js:5","verdict":"leak-confirmed"}]}"#,
    );
    write(
        &base.join("2/coverage.json"),
        r#"{"residual":[{"residual":1,"total_slope":2}]}"#,
    );
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    let g = &m.groups[0];
    assert_eq!(
        (g.usage.sources.none, g.usage.sources.findings),
        (1, 1),
        "run 1 has no source, run 2 has findings.json"
    );
    assert_eq!(g.unexplained_growth_share.runs, 1);
    assert_eq!(g.metrics.leak_recall.per_run[0].num, 0);
    assert_eq!(g.metrics.leak_recall.per_run[1].num, 1);
}

#[test]
fn ledger_lines_are_decoded_one_by_one() {
    let line = |id: &str, site: &str| {
        format!(
            "{{\"type\":\"finding_added\",\"finding\":{{\"id\":\"{id}\",\"site\":\"{site}\"}}}}\n"
        )
    };
    let mut bytes = Vec::new();
    bytes.extend_from_slice(line("F1", "a.js:1").as_bytes());
    // one bad byte inside a string: only this line is lost
    bytes.extend_from_slice(
        b"{\"type\":\"finding_added\",\"finding\":{\"id\":\"F2\",\"site\":\"b.js:\xe9\"}}\n",
    );
    bytes.extend_from_slice(line("F3", "c.js:3").as_bytes());
    let f = grade::findings_from_ledger_bytes(&bytes).expect("has finding_added");
    let sites: Vec<_> = f.iter().map(|f| f.site.as_deref().unwrap()).collect();
    assert_eq!(sites, ["a.js:1", "c.js:3"]);
    // a ledger whose only record is bad has no finding_added: findings.json is used
    let only_bad = b"{\"type\":\"finding_added\",\"finding\":{\"id\":\"F\",\"site\":\"\xff\"}}\n";
    let fj = br#"{"findings":[{"site":"x.js:1","verdict":"leak-confirmed"}]}"#;
    let (src, f) = grade::select_source_bytes(Some(only_bad), Some(fj));
    assert_eq!(src, grade::Source::Findings);
    assert_eq!(f.len(), 1);
}

#[test]
fn trimming_the_discriminator_removes_ascii_whitespace_only() {
    let one = |d: &str| {
        let v = serde_json::json!({"findings":[{"site":"a.js:1","verdict":"not-a-leak","discriminator":d}]});
        grade::findings_from_json(&v).unwrap()[0].is_rule_out()
    };
    assert!(!one(""));
    assert!(!one(" \t\r\n "));
    // U+001C is whitespace for Python's strip() and Rust's trim() but not here
    assert!(one("\u{1c}"));
    assert!(one("\u{a0}"), "no-break space is not trimmed");
    assert!(one("\u{2003}x"));
    assert!(one(" flat "));
}

#[test]
fn unknown_ablation_directories_are_ignored() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    // would be an unknown fixture and a bad run.json if looked at
    write(
        &t.path().join("res/runs/foo/archetypes-on/ghost/1/run.json"),
        "not json",
    );
    write(
        &t.path()
            .join("res/runs/no-skill/archetypes-on/fx-trap/1/run.json"),
        RUN_JSON,
    );
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    assert_eq!(m.groups.len(), 1);
    assert_eq!(m.groups[0].ablation, "no-skill");
}

#[test]
fn a_group_with_only_unfinished_runs_is_written_with_zero_runs() {
    // no fixtures directory at all: expected.toml is not needed
    let t = tempfile::tempdir().unwrap();
    let base = t.path().join("res/runs/prose/archetypes-off/ghost");
    write(&base.join("1/findings.json"), "{}");
    fs::create_dir_all(base.join("2")).unwrap();
    let m = grade::grade_dir(&t.path().join("res"), &t.path().join("none")).unwrap();
    assert_eq!(m.groups.len(), 1);
    let g = &m.groups[0];
    assert_eq!(
        (g.ablation.as_str(), g.archetypes.as_str()),
        ("prose", "off")
    );
    let u = &g.usage;
    assert_eq!((u.runs, u.incomplete, u.errored), (0, 2, 0));
    assert_eq!(u.wall_seconds.sum, 0.0);
    assert_eq!((u.wall_seconds.mean, u.wall_seconds.max), (None, None));
    assert_eq!(u.input_tokens.mean, None);
    assert_eq!((u.cost_usd.sum, u.cost_usd.mean), (0.0, None));
    assert!(g.fixtures.is_empty());
    assert_eq!(g.metrics.leak_recall.den, 0);
    assert!(g.metrics.leak_recall.per_run.is_empty());
    let text = grade::render(&m);
    assert!(text.contains("\"max\": null"), "{text}");
}

#[test]
fn only_fixtures_with_a_finished_run_need_an_expected_toml() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let base = t.path().join("res/runs/no-skill/archetypes-on");
    write(&base.join("fx-trap/1/run.json"), RUN_JSON);
    // unfinished only: no expected.toml needed, not listed under fixtures
    write(&base.join("ghost/1/findings.json"), "{}");
    let m = grade::grade_dir(&t.path().join("res"), &s.join("fixtures")).unwrap();
    let g = &m.groups[0];
    assert_eq!((g.usage.runs, g.usage.incomplete), (1, 1));
    assert_eq!(g.fixtures.keys().collect::<Vec<_>>(), ["fx-trap"]);
}
