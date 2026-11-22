use super::*;
use crate::{
    Ledger, NewEvidence, NewFinding, Origin,
    evidence::{Authority, EvidenceCoverage, EvidenceKind, Source, Tool},
    state::{RunState, now_utc},
    trend::{TargetVerdict, TrendReport},
};
use serde_json::json;

/// A run directory with `n` census findings and the harnesses, series, trend reports and run
/// records the test writes.
struct Run {
    dir: tempfile::TempDir,
}

/// What a harness is.
#[derive(Clone, Copy)]
enum Kind {
    /// A hook driver that is not entry-path.
    Hooks,
    /// A hook driver with `purpose: entry-path`.
    Entry,
    /// The project's own workload: a fixed command.
    Command,
    /// The project's own JUnit tests in a loop (no hooks).
    TestLoop,
    /// The project's own tests with pytest in a loop (no hooks, no target).
    PytestLoop,
}

fn evidence() -> NewEvidence {
    NewEvidence {
        source: Source::Repo {
            commit: "c".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "1".into(),
        },
        authority: Authority::Static,
        kind: EvidenceKind::CensusHolder,
        result: json!({"lang": "python"}),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    }
}

impl Run {
    fn new(n: u32) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let st = RunState::init("/p", None, 3600, None, now_utc()).unwrap();
        st.save(dir.path()).unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        l.add_evidence(evidence()).unwrap();
        for i in 1..=n {
            let mut nf = NewFinding::new(
                Origin::parse("census").unwrap(),
                vec![crate::EvidenceId::new(1)],
            );
            nf.site = Some(format!("pkg/m{i}.py:3"));
            l.add_finding(nf).unwrap();
        }
        Self { dir }
    }

    fn harness(&self, id: &str, kind: Kind, targets: &[u32]) {
        let mode = if matches!(kind, Kind::Command) {
            "command"
        } else {
            "driver"
        };
        let mut text = json!({
            "id": id, "runtime": "python", "mode": mode,
            "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
            "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
            "targets": targets.iter().map(|n| format!("F{n}")).collect::<Vec<_>>(),
            "agent": "driver-python", "timeout_secs": 60,
        });
        if matches!(kind, Kind::Entry) {
            text["purpose"] = json!("entry-path");
        }
        if matches!(kind, Kind::TestLoop) {
            text["tools"] = json!(["junit-platform-launcher"]);
            text["workload"] = json!("W2");
        }
        if matches!(kind, Kind::PytestLoop) {
            text["tools"] = json!(["pytest-loop"]);
            text["workload"] = json!("W2");
        }
        if matches!(kind, Kind::Command) {
            text["workload"] = json!("W2");
        }
        let m: HarnessManifest = serde_json::from_value(text).unwrap();
        m.save(self.dir.path()).unwrap();
    }

    /// A series of harness `h`, its run record and its trend report.
    fn run(
        &self,
        series: &str,
        h: &str,
        digest: &str,
        varies: Option<&str>,
        read: Option<&[(u32, Verdict)]>,
    ) {
        let text = json!({
            "id": series, "source": {"kind": "harness", "harness": h},
            "axis": "n", "columns": [], "points": []
        });
        let p = self.dir.path().join("series");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join(format!("{series}.json")), text.to_string()).unwrap();
        let mut rec = json!({"series": [series], "driver_digest": digest});
        if let Some(v) = varies {
            rec["varies"] = json!(v);
        }
        std::fs::write(
            self.dir
                .path()
                .join("harness")
                .join(h)
                .join(format!("run-{series}.json")),
            rec.to_string(),
        )
        .unwrap();
        if let Some(verdicts) = read {
            let r = TrendReport {
                schema_version: 1,
                id: format!("T{}", &series[1..]),
                series: series.into(),
                thresholds: dla_stats::GateThresholds::default(),
                columns: vec![],
                targets: verdicts
                    .iter()
                    .map(|(n, v)| TargetVerdict {
                        finding: FindingId::new(*n),
                        verdict: *v,
                        column: None,
                        detail: String::new(),
                        declared: true,
                    })
                    .collect(),
            };
            r.save(self.dir.path()).unwrap();
        }
    }

    fn triggers(&self) -> Triggers {
        Triggers::of(&RunView::load(self.dir.path()).unwrap())
    }
}

fn f(n: u32) -> FindingId {
    FindingId::new(n)
}

#[test]
fn an_entry_path_harness_that_grew_it_reproduces_the_trigger() {
    let run = Run::new(2);
    run.harness("H1", Kind::Entry, &[1, 2]);
    run.run(
        "S1",
        "H1",
        "aa",
        Some("the request path, new per unit"),
        Some(&[(1, Verdict::Grew), (2, Verdict::Constant)]),
    );
    let t = run.triggers();
    assert_eq!(
        t.trigger(&f(1)),
        Trigger::Reproduced {
            harness: "H1".into(),
            varies: "the request path, new per unit".into()
        }
    );
    // F2 ran in it and did not grow: one run is not enough for not_found.
    assert_eq!(t.trigger(&f(2)), Trigger::Unchecked);
}

#[test]
fn a_harness_that_runs_the_projects_own_workload_reproduces_it() {
    let run = Run::new(1);
    run.harness("H1", Kind::Command, &[1]);
    run.run("S1", "H1", "aa", None, Some(&[(1, Verdict::Grew)]));
    assert_eq!(
        run.triggers().trigger(&f(1)),
        Trigger::Reproduced {
            harness: "H1".into(),
            varies: "the project's workload W2".into()
        }
    );
}

/// Writes `harness/<h>/attribution-F<n>.json`: what the loop measured per class, as
/// `(class, first run, later runs)`, over `later` runs after the first.
fn attribution_json(run: &Run, h: &str, n: u32, later: u64, classes: &[(&str, i64, i64)]) {
    let doc = json!({
        "probe": "org.acme.Cache#resolved",
        "later_runs": later,
        "classes": classes.iter().map(|(c, first, rest)| json!({
            "class": c, "first_run": first, "later_runs": rest,
        })).collect::<Vec<_>>(),
        "dropped": [],
    });
    std::fs::write(
        run.dir
            .path()
            .join("harness")
            .join(h)
            .join(format!("attribution-F{n}.json")),
        doc.to_string(),
    )
    .unwrap();
}

fn varies_of(t: &Triggers, n: u32) -> String {
    match t.trigger(&f(n)) {
        Trigger::Reproduced { varies, .. } => varies,
        other => panic!("{other:?}"),
    }
}

fn grew_in_a_test_loop(n: u32) -> Run {
    let run = Run::new(n);
    let ids: Vec<u32> = (1..=n).collect();
    run.harness("H1", Kind::TestLoop, &ids);
    let verdicts: Vec<(u32, Verdict)> = ids.iter().map(|i| (*i, Verdict::Grew)).collect();
    run.run("S1", "H1", "aa", None, Some(&verdicts));
    run
}

#[test]
fn a_test_loop_trigger_names_the_classes_that_were_measured_feeding_the_holder() {
    let run = grew_in_a_test_loop(2);
    attribution_json(
        &run,
        "H1",
        1,
        20,
        &[
            ("org.acme.a.FlatUnitTests", 267, 0),
            ("org.acme.b.SmallUnitTests", 0, 20),
            ("org.acme.c.BigUnitTests", 3, 266),
        ],
    );
    let t = run.triggers();
    // Largest per run first; the figure is the later-runs sum over the later runs.
    assert_eq!(
        varies_of(&t, 1),
        "the project's own tests BigUnitTests, SmallUnitTests (13.3, 1 entries per run)"
    );
    assert_eq!(t.test_loop_harnesses(&f(1)), ["H1"]);
    // F2 has no attribution: not measured, with the command that measures it.
    assert_eq!(
        varies_of(&t, 2),
        "the project's own tests in H1; which test class feeds it is not measured (dla alloc-test H1 --finding F2)"
    );
}

#[test]
fn a_test_loop_trigger_names_at_most_three_feeders_and_counts_the_rest() {
    let run = grew_in_a_test_loop(1);
    attribution_json(
        &run,
        "H1",
        1,
        10,
        &[
            ("a.One", 0, 50),
            ("a.Two", 0, 40),
            ("a.Three", 0, 30),
            ("a.Four", 0, 20),
            ("a.Five", 0, 10),
        ],
    );
    assert_eq!(
        varies_of(&run.triggers(), 1),
        "the project's own tests One, Two, Three and 2 more (5, 4, 3 entries per run)"
    );
}

#[test]
fn a_test_loop_trigger_with_an_attribution_and_no_feeder_says_so() {
    let run = grew_in_a_test_loop(1);
    attribution_json(
        &run,
        "H1",
        1,
        20,
        &[
            ("org.acme.a.FooUnitTests", 5, 0),
            ("org.acme.b.BarUnitTests", 0, 0),
        ],
    );
    assert_eq!(
        varies_of(&run.triggers(), 1),
        "the project's own tests; no single test class added entries per run in H1"
    );
}

#[test]
fn a_class_chosen_for_a_finding_is_not_named_as_what_feeds_it() {
    let run = grew_in_a_test_loop(1);
    // The queue chose a class for F1 by name; nothing measured that it feeds anything.
    let doc = json!({"classes": [{
        "class": "org.acme.a.FooUnitTests", "file": "src/test/X.java", "tests": 3,
        "for": [{"by": "name", "finding": "F1"}],
    }]});
    std::fs::write(
        run.dir.path().join("harness/H1/tests.json"),
        doc.to_string(),
    )
    .unwrap();
    let v = varies_of(&run.triggers(), 1);
    assert!(!v.contains("FooUnitTests"), "{v}");
    assert!(v.contains("not measured"), "{v}");
}

#[test]
fn a_finding_that_did_not_grow_in_a_test_loop_or_grew_in_another_harness_is_not_listed() {
    let run = grew_in_a_test_loop(1);
    run.harness("H2", Kind::TestLoop, &[1]);
    run.run("S2", "H2", "bb", None, Some(&[(1, Verdict::Constant)]));
    run.harness("H3", Kind::Command, &[1]);
    run.run("S3", "H3", "cc", None, Some(&[(1, Verdict::Grew)]));
    assert_eq!(run.triggers().test_loop_harnesses(&f(1)), ["H1"]);
}

#[test]
fn growth_in_a_hook_driver_alone_is_unchecked() {
    let run = Run::new(1);
    run.harness("H1", Kind::Hooks, &[1]);
    run.run("S1", "H1", "aa", None, Some(&[(1, Verdict::Grew)]));
    let t = run.triggers();
    assert_eq!(t.trigger(&f(1)), Trigger::Unchecked);
    assert!(t.grew_in_a_harness(&f(1)));
}

#[test]
fn two_entry_path_harnesses_with_different_drivers_and_no_growth_find_no_trigger() {
    let run = Run::new(1);
    run.harness("H1", Kind::Hooks, &[1]);
    run.run("S1", "H1", "aa", None, Some(&[(1, Verdict::Grew)]));
    run.harness("H2", Kind::Entry, &[1]);
    run.run(
        "S2",
        "H2",
        "bb",
        Some("paths"),
        Some(&[(1, Verdict::Constant)]),
    );
    assert_eq!(run.triggers().trigger(&f(1)), Trigger::Unchecked);
    run.harness("H3", Kind::Entry, &[1]);
    run.run(
        "S3",
        "H3",
        "cc",
        Some("methods and bodies"),
        Some(&[(1, Verdict::Executed)]),
    );
    // H3 ran but nothing measured the holder there: no evidence either way.
    assert_eq!(
        run.triggers().trigger(&f(1)),
        Trigger::Unmeasured {
            runs: vec![
                ("H2".into(), "constant".into()),
                ("H3".into(), "executed".into())
            ]
        }
    );
    // A third harness that measured it flat gives "no trigger found" with H2.
    run.harness("H4", Kind::Entry, &[1]);
    run.run(
        "S4",
        "H4",
        "dd",
        Some("methods and bodies"),
        Some(&[(1, Verdict::Constant)]),
    );
    assert_eq!(
        run.triggers().trigger(&f(1)),
        Trigger::NotFound {
            runs: vec![
                ("H2".into(), "paths".into()),
                ("H4".into(), "methods and bodies".into())
            ]
        }
    );
}

#[test]
fn runs_that_did_not_measure_the_holder_are_not_checked_whatever_the_verdict() {
    for other in [Verdict::Executed, Verdict::Unknown, Verdict::NotExecuted] {
        let run = Run::new(1);
        run.harness("H1", Kind::Entry, &[1]);
        run.run("S1", "H1", "aa", Some("a"), Some(&[(1, other)]));
        run.harness("H2", Kind::Entry, &[1]);
        run.run("S2", "H2", "bb", Some("b"), Some(&[(1, other)]));
        let name = other.as_str();
        assert_eq!(
            run.triggers().trigger(&f(1)),
            Trigger::Unmeasured {
                runs: vec![("H1".into(), name.into()), ("H2".into(), name.into())]
            },
            "{name}"
        );
    }
    // A later run of H1 with a size reading is the one that counts for H1.
    let run = Run::new(1);
    run.harness("H1", Kind::Entry, &[1]);
    run.run("S1", "H1", "aa", Some("a"), Some(&[(1, Verdict::Executed)]));
    run.harness("H2", Kind::Entry, &[1]);
    run.run("S2", "H2", "bb", Some("b"), Some(&[(1, Verdict::Constant)]));
    assert!(matches!(
        run.triggers().trigger(&f(1)),
        Trigger::Unmeasured { .. }
    ));
    run.run("S3", "H1", "aa", Some("a"), Some(&[(1, Verdict::Constant)]));
    assert!(matches!(
        run.triggers().trigger(&f(1)),
        Trigger::NotFound { .. }
    ));
}

#[test]
fn the_same_driver_twice_does_not_count() {
    let run = Run::new(1);
    run.harness("H1", Kind::Hooks, &[1]);
    run.run("S1", "H1", "aa", None, Some(&[(1, Verdict::Grew)]));
    // Two harnesses whose drivers have one digest.
    run.harness("H2", Kind::Entry, &[1]);
    run.run(
        "S2",
        "H2",
        "bb",
        Some("paths"),
        Some(&[(1, Verdict::Constant)]),
    );
    run.harness("H3", Kind::Entry, &[1]);
    run.run(
        "S3",
        "H3",
        "bb",
        Some("paths"),
        Some(&[(1, Verdict::Constant)]),
    );
    assert_eq!(run.triggers().trigger(&f(1)), Trigger::Unchecked);
    // One harness run twice with an edited driver is one harness.
    let run = Run::new(1);
    run.harness("H1", Kind::Entry, &[1]);
    run.run("S1", "H1", "aa", Some("a"), Some(&[(1, Verdict::Constant)]));
    run.run("S2", "H1", "bb", Some("b"), Some(&[(1, Verdict::Constant)]));
    assert_eq!(run.triggers().trigger(&f(1)), Trigger::Unchecked);
}

#[test]
fn a_run_whose_trend_was_not_read_does_not_count() {
    let run = Run::new(1);
    run.harness("H1", Kind::Entry, &[1]);
    run.run("S1", "H1", "aa", Some("a"), Some(&[(1, Verdict::Constant)]));
    run.harness("H2", Kind::Entry, &[1]);
    run.run("S2", "H2", "bb", Some("b"), None);
    assert_eq!(run.triggers().trigger(&f(1)), Trigger::Unchecked);
}

#[test]
fn growth_in_a_later_entry_path_harness_reproduces_after_a_miss() {
    let run = Run::new(1);
    run.harness("H1", Kind::Entry, &[1]);
    run.run("S1", "H1", "aa", Some("a"), Some(&[(1, Verdict::Constant)]));
    run.harness("H2", Kind::Entry, &[1]);
    run.run("S2", "H2", "bb", Some("b"), Some(&[(1, Verdict::Grew)]));
    assert_eq!(
        run.triggers().trigger(&f(1)),
        Trigger::Reproduced {
            harness: "H2".into(),
            varies: "b".into()
        }
    );
}

#[test]
fn an_entry_harness_not_run_yet_is_in_flight() {
    let run = Run::new(1);
    run.harness("H1", Kind::Entry, &[1]);
    let t = run.triggers();
    assert_eq!(t.entry_in_flight(&f(1)), Some("H1"));
    run.run("S1", "H1", "aa", Some("a"), Some(&[(1, Verdict::Constant)]));
    assert_eq!(run.triggers().entry_in_flight(&f(1)), None);
}

#[test]
fn a_repeated_series_says_nothing_about_new_keys() {
    let run = Run::new(1);
    run.harness("H1", Kind::Entry, &[1]);
    run.run("S1", "H1", "aa", Some("a"), Some(&[(1, Verdict::Constant)]));
    run.harness("H2", Kind::Entry, &[1]);
    run.run("S2", "H2", "bb", Some("b"), Some(&[(1, Verdict::Constant)]));
    assert!(matches!(
        run.triggers().trigger(&f(1)),
        Trigger::NotFound { .. }
    ));
    // H2's only read series reused its keys: H2 does not count.
    let p = run.dir.path().join("series/S2.json");
    let mut doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    doc["source"]["mode"] = json!("repeated");
    std::fs::write(&p, doc.to_string()).unwrap();
    assert_eq!(run.triggers().trigger(&f(1)), Trigger::Unchecked);
}

/// Writes `driver/<file>` and `targets.json` (writer functions of F1) of entry-path harness `h`.
fn driver_calling(run: &Run, h: &str, text: &str, writers: &[&str]) {
    let dir = run.dir.path().join("harness").join(h);
    std::fs::create_dir_all(dir.join("driver")).unwrap();
    std::fs::write(dir.join("driver/zz_driver_test.go"), text).unwrap();
    std::fs::write(
        dir.join("targets.json"),
        json!({"targets": [{"finding": "F1", "writer_functions": writers}]}).to_string(),
    )
    .unwrap();
}

/// Records the census facts of F1's one writer site.
fn writer_facts(run: &Run, init: bool, startup: bool, param: bool, constant: bool) {
    use crate::{
        FindingAnnotation,
        census::{HolderFacts, SiteFact, SiteKind},
    };
    let mut l = Ledger::load(run.dir.path()).unwrap();
    l.annotate_record(FindingAnnotation {
        writers: vec!["pkg/m1.go:9".into()],
        facts: HolderFacts {
            writer_sites: vec![SiteFact {
                site: "pkg/m1.go:9".into(),
                kind: SiteKind::Insert,
                depth: 0,
                function: Some("Register".into()),
                init,
                startup,
                param,
                constant,
                exported: true,
            }],
            ..Default::default()
        },
        ..FindingAnnotation::new(f(1))
    })
    .unwrap();
}

/// A run whose unit calls `Register` itself and made F1 grow.
fn a_unit_that_calls_the_writer() -> Run {
    let run = Run::new(1);
    run.harness("H1", Kind::Entry, &[1]);
    run.run(
        "S1",
        "H1",
        "aa",
        Some("the name"),
        Some(&[(1, Verdict::Grew)]),
    );
    // The unit calls Register itself (a comment that names it does not count).
    driver_calling(
        &run,
        "H1",
        "// Register is the writer\nfunc unit(i int) {\n\tbalancer.Register(newBuilder(i))\n}\n",
        &["Register"],
    );
    run
}

#[test]
fn a_start_up_registry_the_unit_feeds_directly_is_the_direct_reading() {
    // Every writer site runs at start-up with constant keys: the census says registration.
    let run = a_unit_that_calls_the_writer();
    writer_facts(&run, false, true, false, true);
    match run.triggers().trigger(&f(1)) {
        Trigger::Direct {
            harness, writer, ..
        } => {
            assert_eq!((harness.as_str(), writer.as_str()), ("H1", "Register"));
        }
        other => panic!("{other:?}"),
    }
    // An initialiser counts the same way.
    let run = a_unit_that_calls_the_writer();
    writer_facts(&run, true, false, false, true);
    assert!(matches!(
        run.triggers().trigger(&f(1)),
        Trigger::Direct { .. }
    ));
}

#[test]
fn a_handler_no_project_code_calls_is_a_public_entry_and_is_reproduced() {
    // `Server.Watch`: exported, not at start-up, no key from a parameter in a project caller.
    let run = a_unit_that_calls_the_writer();
    writer_facts(&run, false, false, false, false);
    assert!(matches!(
        run.triggers().trigger(&f(1)),
        Trigger::Reproduced { .. }
    ));
}

#[test]
fn a_run_time_caller_with_a_key_from_a_parameter_is_reproduced() {
    let run = a_unit_that_calls_the_writer();
    writer_facts(&run, false, false, true, false);
    assert!(matches!(
        run.triggers().trigger(&f(1)),
        Trigger::Reproduced { .. }
    ));
}

#[test]
fn without_census_facts_on_the_writer_neither_reading_is_given() {
    let run = a_unit_that_calls_the_writer();
    let t = run.triggers();
    assert_eq!(t.trigger(&f(1)), Trigger::Unchecked);
    assert_eq!(
        t.direct_without_facts(&f(1)),
        Some(("H1".to_owned(), "Register".to_owned()))
    );
    // A unit that only names the writer in a comment is not a direct call at all.
    let run = Run::new(1);
    run.harness("H1", Kind::Entry, &[1]);
    run.run(
        "S1",
        "H1",
        "aa",
        Some("the name"),
        Some(&[(1, Verdict::Grew)]),
    );
    driver_calling(
        &run,
        "H1",
        "// Register is the writer\nfunc unit(i int) {\n\tserver.Handle(req(i))\n}\n",
        &["Register"],
    );
    assert!(matches!(
        run.triggers().trigger(&f(1)),
        Trigger::Reproduced { .. }
    ));
}

/// A trend report of series `series` with one allocation-site column `py.blocks@<site>` that
/// passed (`pass`) or not, and no target verdict.
fn site_column_trend(run: &Run, series: &str, site: &str, pass: bool) {
    let r: TrendReport = serde_json::from_value(json!({
        "id": format!("T{}", &series[1..]), "series": series,
        "thresholds": serde_json::to_value(dla_stats::GateThresholds::default()).unwrap(),
        "columns": [{
            "name": format!("py.blocks@{site}"), "unit": "count", "kind": "gauge", "region": "heap",
            "site": site, "candidates": [], "n": 8, "noise": 0.0, "rise": 16.0,
            "shape": "steady", "gate": if pass { "pass" } else { "fail" },
        }],
    }))
    .unwrap();
    r.save(run.dir.path()).unwrap();
}

#[test]
fn a_finding_at_a_line_that_grew_in_the_pytest_loop_has_the_trigger_of_the_project_tests() {
    let run = Run::new(2);
    run.harness("H1", Kind::PytestLoop, &[]);
    run.run("S1", "H1", "aa", None, None);
    site_column_trend(&run, "S1", "pkg/m1.py:3", true);
    let t = run.triggers();
    // F1 is at the line: the loop never declared it, and the column ties it. The text does not
    // offer the alloc-test, which measures test classes of a JVM loop.
    assert_eq!(
        varies_of(&t, 1),
        "the project's own tests in H1; which test file feeds it is not measured"
    );
    assert_eq!(t.test_loop_harnesses(&f(1)), ["H1"]);
    // F2 is at another line: nothing grew for it in the loop.
    assert!(t.test_loop_harnesses(&f(2)).is_empty());
    assert!(!matches!(t.trigger(&f(2)), Trigger::Reproduced { .. }));
}

#[test]
fn a_site_column_that_did_not_pass_gives_no_trigger() {
    let run = Run::new(1);
    run.harness("H1", Kind::PytestLoop, &[]);
    run.run("S1", "H1", "aa", None, None);
    site_column_trend(&run, "S1", "pkg/m1.py:3", false);
    let t = run.triggers();
    assert!(t.test_loop_harnesses(&f(1)).is_empty());
    assert!(!matches!(t.trigger(&f(1)), Trigger::Reproduced { .. }));
}
