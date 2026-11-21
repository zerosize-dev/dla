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
    assert_eq!(
        run.triggers().trigger(&f(1)),
        Trigger::NotFound {
            runs: vec![
                ("H2".into(), "paths".into()),
                ("H3".into(), "methods and bodies".into())
            ]
        }
    );
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
