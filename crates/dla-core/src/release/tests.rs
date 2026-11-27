use serde_json::{Value, json};

use super::*;
use crate::{
    Ledger, NewEvidence, NewFinding, Origin,
    evidence::{Authority, EvidenceCoverage, Source, Tool},
    state::{RunState, now_utc},
};

const FLAT: &str = "non-leak";

fn evidence(l: &mut Ledger, kind: EvidenceKind, result: Value) -> EvidenceId {
    l.add_evidence(NewEvidence {
        source: Source::Repo {
            commit: "c".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "1".into(),
        },
        authority: Authority::Test,
        kind,
        result,
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
    l.evidence_in_order().last().unwrap().evidence_id.clone()
}

fn alloc_test(l: &mut Ledger, favors: &str, harness: &str) -> EvidenceId {
    evidence(
        l,
        EvidenceKind::Discriminator,
        json!({"test": "plateau+purge", "favors": favors, "finding": "F1",
            "harness": harness, "series": "S1"}),
    )
}

fn manifest(id: &str, runtime: &str, extra: Value) -> HarnessManifest {
    let mut v = json!({
        "id": id, "runtime": runtime, "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 10, "key_modes": ["fresh"],
        "agent": "driver-python", "timeout_secs": 60,
    });
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    serde_json::from_value(v).unwrap()
}

/// A run with F1 (leak-side), its sibling F1.n1 (`N-WARM`) and F1.n2 (`N-RELEASE`, remover
/// `a.py:30`), the harnesses H1 and H2 (drivers that target F1) and H3 (the project's test loop).
struct Scene {
    dir: tempfile::TempDir,
    ledger: Ledger,
}

fn scene(runtime: &str) -> Scene {
    let dir = tempfile::tempdir().unwrap();
    RunState::init("/p", None, 3600, None, now_utc())
        .unwrap()
        .save(dir.path())
        .unwrap();
    let mut l = Ledger::create(dir.path()).unwrap();
    let first = evidence(&mut l, EvidenceKind::StaticHit, json!({}));
    let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![first.clone()]);
    nf.site = Some("a.py:1".into());
    l.add_finding(nf).unwrap();
    let sibling = |l: &mut Ledger, hypothesis: &str, remover: Option<&str>| {
        let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![first.clone()]);
        nf.sibling_of = Some(FindingId::new(1));
        nf.hypothesis = Some(hypothesis.into());
        nf.refute_if = Some("growth continues".into());
        nf.remover = remover.map(str::to_owned);
        l.add_finding(nf)
    };
    sibling(&mut l, "N-WARM: warm-up", None).unwrap();
    sibling(&mut l, "N-RELEASE", Some("a.py:30")).unwrap();
    for (h, extra) in [
        ("H1", json!({"targets": ["F1"]})),
        ("H2", json!({"targets": ["F1"]})),
        ("H3", json!({"mode": "command", "workload": "w"})),
    ] {
        manifest(h, runtime, extra).save(dir.path()).unwrap();
    }
    Scene { dir, ledger: l }
}

impl Scene {
    fn view(self) -> (tempfile::TempDir, RunView) {
        let state = RunState::init("/p", None, 3600, None, now_utc()).unwrap();
        let mut view = RunView::new(state, self.ledger, None);
        view.run_dir = Some(self.dir.path().to_path_buf());
        (self.dir, view)
    }
}

fn f(n: &str) -> FindingId {
    n.parse().unwrap()
}

/// Writes the coverage record of `harness` with one counted line.
fn coverage(dir: &Path, harness: &str, lines: Value) {
    let hdir = HarnessManifest::dir_in(dir, harness);
    std::fs::create_dir_all(&hdir).unwrap();
    std::fs::write(
        hdir.join("executed.json"),
        json!({"runtime": "rust", "kind": "functions", "lines": lines}).to_string(),
    )
    .unwrap();
}

fn decide(view: &RunView, ev: &EvidenceId) -> Result<Decision, Refusal> {
    let s = view.ledger.finding(&f("F1.n2")).unwrap();
    let e = view.ledger.evidence_by_id(ev).unwrap();
    check_release_decision(view, s, &[e], Favors::NonLeak)
}

#[test]
fn n_release_is_listed_and_a_sibling_names_its_release() {
    assert!(crate::finding::NON_LEAK_IDS.contains(&ID));
    assert_eq!(nonleak_id("N-RELEASE: stop() is skipped"), Some(ID));
    let table = crate::archetypes::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data"),
    )
    .unwrap()
    .unwrap();
    let row = table.get(ID).expect("the archetype row");
    assert!(row.summary.contains("skips a release the API requires"));
    assert!(row.disconfirm.is_empty() && row.holders.is_empty());
    let mut s = scene("python");
    let first = EvidenceId::new(1);
    let add = |l: &mut Ledger, hypothesis: &str, remover: Option<&str>| {
        let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![first.clone()]);
        nf.sibling_of = Some(FindingId::new(1));
        nf.hypothesis = Some(hypothesis.into());
        nf.refute_if = Some("r".into());
        nf.remover = remover.map(str::to_owned);
        l.add_finding(nf)
    };
    // The sibling of the scene named its release; a sibling without one, one with a malformed
    // line and a remover on another hypothesis are refused.
    assert!(is_release(s.ledger.finding(&f("F1.n2")).unwrap()));
    for (h, r) in [
        ("N-RELEASE", None),
        ("N-RELEASE", Some("not a site")),
        ("N-WARM", Some("a.py:30")),
        ("a leak of its own", Some("a.py:30")),
    ] {
        let err = add(&mut s.ledger, h, r).unwrap_err().to_string();
        assert!(
            err.contains("remover") || err.contains("N-RELEASE"),
            "{h}: {err}"
        );
    }
    assert_eq!(
        s.ledger.findings().count(),
        3,
        "nothing refused was written"
    );
}

#[test]
fn the_remover_line_is_counted_where_the_runtime_counts_lines() {
    let s = scene("rust");
    let dir = s.dir.path().to_path_buf();
    let m = manifest("H1", "rust", json!({"targets": ["F1"]}));
    let run = |lines: Value| {
        coverage(&dir, "H1", lines);
        remover_run(&dir, &m, None, "src/lib.rs:30")
    };
    assert_eq!(
        run(json!([{"file": "src/lib.rs", "line": 30, "count": 12}])),
        RemoverRun::Ran(12)
    );
    assert_eq!(
        run(json!([{"file": "src/lib.rs", "line": 30, "count": 0}])),
        RemoverRun::NotRun
    );
    assert!(matches!(
        run(json!([{"file": "src/lib.rs", "line": 30, "count": null, "why": "no region"}])),
        RemoverRun::NotShown(w) if w == "no region"
    ));
    // The run did not ask about the line.
    assert_eq!(
        run(json!([{"file": "src/lib.rs", "line": 31, "count": 4}])),
        RemoverRun::NotAsked
    );
    // A runtime that cannot count lines.
    for rt in ["jvm", "native"] {
        let m = manifest("H1", rt, json!({"targets": ["F1"]}));
        assert!(matches!(
            remover_run(&dir, &m, None, "src/A.java:30"),
            RemoverRun::NotShown(_)
        ));
    }
    // Python: the counts of the agent, per point.
    let py = manifest("H2", "python", json!({"targets": ["F1"]}));
    let write = |counts: Value| {
        LineRuns {
            dla_line_runs: 1,
            series: "S1".into(),
            sites: serde_json::from_value(json!({ "pkg/a.py:30": counts })).unwrap(),
            unavailable: None,
        }
        .save(&dir, "H2")
        .unwrap();
        remover_run(&dir, &py, Some("S1"), "pkg/a.py:30")
    };
    assert_eq!(write(json!([0, 3, 9])), RemoverRun::Ran(9));
    assert_eq!(write(json!([0, 0, 0])), RemoverRun::NotRun);
    assert_eq!(
        remover_run(&dir, &py, Some("S1"), "pkg/a.py:99"),
        RemoverRun::NotAsked
    );
}

#[test]
fn an_alloc_test_decides_n_release_only_when_the_remover_line_ran() {
    let mut s = scene("rust");
    let flat = alloc_test(&mut s.ledger, FLAT, "H1");
    let (dir, view) = s.view();
    let line = |count: Value| {
        coverage(
            dir.path(),
            "H1",
            json!([{"file": "a.py", "line": 30, "count": count}]),
        )
    };
    // The line ran: decided, no note.
    line(json!(7));
    assert_eq!(decide(&view, &flat).unwrap(), Decision { note: None });
    // The line never ran: refused, and the refusal says the release was not reached.
    line(json!(0));
    let e = decide(&view, &flat).unwrap_err();
    assert_eq!(e.code, codes::RELEASE_NOT_RUN);
    assert!(e.message.contains("ran 0 times"), "{}", e.message);
    // A runtime that cannot count the line: accepted with the note.
    let (dir2, view2) = {
        let mut s = scene("jvm");
        let flat = alloc_test(&mut s.ledger, FLAT, "H1");
        let (d, v) = s.view();
        assert_eq!(flat, EvidenceId::new(2));
        (d, v)
    };
    let d = decide(&view2, &flat).unwrap();
    assert_eq!(d.note.as_deref(), Some(NOT_SHOWN));
    drop(dir2);
}

#[test]
fn any_other_record_or_harness_does_not_decide_n_release() {
    let mut s = scene("rust");
    let growth = evidence(
        &mut s.ledger,
        EvidenceKind::GrowthSeries,
        json!({"gate": "pass", "series": "S1"}),
    );
    let on_loop = alloc_test(&mut s.ledger, FLAT, "H3");
    let leak_side = alloc_test(&mut s.ledger, "leak", "H1");
    let flat = alloc_test(&mut s.ledger, FLAT, "H1");
    let (dir, view) = s.view();
    coverage(
        dir.path(),
        "H1",
        json!([{"file": "a.py", "line": 30, "count": 5}]),
    );
    coverage(
        dir.path(),
        "H3",
        json!([{"file": "a.py", "line": 30, "count": 5}]),
    );
    for (ev, why) in [
        (&growth, "not an alloc-test"),
        (&on_loop, "not a driver made for"),
        (&leak_side, "favors leak, not non-leak"),
    ] {
        let e = decide(&view, ev).unwrap_err();
        assert_eq!(e.code, codes::RELEASE_NOT_RUN, "{ev}");
        assert!(e.message.contains(why), "{ev}: {}", e.message);
        // The fix names the alloc-test that would do.
        assert_eq!(
            e.fix,
            format!("dla ledger confirm F1.n2 --evidence {flat}"),
            "{ev}"
        );
    }
    assert!(decide(&view, &flat).is_ok());
}

#[test]
fn drivers_that_disagree_hold_the_confirm_until_a_sibling_that_names_the_difference_is_refuted() {
    // H1 flat, then H2 leak: the newest alloc-test of each harness made for F1 disagree.
    let mut s = scene("python");
    let flat = alloc_test(&mut s.ledger, FLAT, "H1");
    let leak = alloc_test(&mut s.ledger, "leak", "H2");
    let (_d, view) = s.view();
    let d = drivers_disagree(&view, &f("F1")).expect("a disagreement");
    assert_eq!(
        (d.flat.evidence.clone(), d.flat.harness.as_str()),
        (flat, "H1")
    );
    assert_eq!(
        (d.leak.evidence.clone(), d.leak.harness.as_str()),
        (leak, "H2")
    );
    let r = drivers_disagree_refusal(&view, &f("F1"), &d);
    assert_eq!(r.code, codes::DRIVERS_DISAGREE);
    assert!(
        r.message.contains("What does H1 do that H2 does not?"),
        "{}",
        r.message
    );
    assert!(
        r.fix
            .contains("--hypothesis N-RELEASE --remover <file:line>"),
        "{}",
        r.fix
    );
    assert!(
        r.fix.contains("dla alloc-test H1 --finding F1"),
        "{}",
        r.fix
    );
    assert!(r.fix.contains("pooled or worker threads"), "{}", r.fix);
}

#[test]
fn one_driver_or_agreeing_drivers_hold_nothing() {
    for favors in [["leak", "leak"], [FLAT, FLAT]] {
        let mut s = scene("python");
        alloc_test(&mut s.ledger, favors[0], "H1");
        alloc_test(&mut s.ledger, favors[1], "H2");
        let (_d, view) = s.view();
        assert!(drivers_disagree(&view, &f("F1")).is_none(), "{favors:?}");
    }
    // One harness, leak then flat or flat then leak: only its newest counts.
    for order in [["leak", FLAT], [FLAT, "leak"]] {
        let mut s = scene("python");
        alloc_test(&mut s.ledger, order[0], "H1");
        alloc_test(&mut s.ledger, order[1], "H1");
        let (_d, view) = s.view();
        assert!(drivers_disagree(&view, &f("F1")).is_none(), "{order:?}");
    }
    // The flat alloc-test ran on the project's test loop, which is no driver made for F1.
    let mut s = scene("python");
    alloc_test(&mut s.ledger, FLAT, "H3");
    alloc_test(&mut s.ledger, "leak", "H1");
    let (_d, view) = s.view();
    assert!(drivers_disagree(&view, &f("F1")).is_none());
}

#[test]
fn a_rerun_of_the_flat_driver_that_now_favors_leak_settles_it() {
    let mut s = scene("python");
    alloc_test(&mut s.ledger, FLAT, "H1");
    alloc_test(&mut s.ledger, "leak", "H2");
    alloc_test(&mut s.ledger, "leak", "H1");
    let (_d, view) = s.view();
    assert!(drivers_disagree(&view, &f("F1")).is_none());
}

#[test]
fn a_third_driver_settles_it_only_with_a_sibling_added_after_and_refuted_by_a_newer_alloc_test() {
    let build = |refute: bool, third_on: &str| {
        let mut s = scene("python");
        manifest("H4", "python", json!({"targets": ["F1"]}))
            .save(s.dir.path())
            .unwrap();
        alloc_test(&mut s.ledger, FLAT, "H1");
        alloc_test(&mut s.ledger, "leak", "H2");
        // The sibling that names the difference, added after both.
        let mut nf = NewFinding::new(
            Origin::parse("hypothesis").unwrap(),
            vec![EvidenceId::new(1)],
        );
        nf.sibling_of = Some(FindingId::new(1));
        nf.hypothesis = Some("N-INTENT".into());
        nf.refute_if = Some("r".into());
        s.ledger.add_finding(nf).unwrap();
        let sib = f("F1.n3");
        let third = alloc_test(&mut s.ledger, "leak", third_on);
        if refute {
            let ws = evidence(
                &mut s.ledger,
                EvidenceKind::Workload,
                json!({"harness": "H1"}),
            );
            s.ledger
                .transition(&sib, FindingState::UnderTest, None, None, vec![ws], "t")
                .unwrap();
            s.ledger
                .transition(&sib, FindingState::Refuted, None, None, vec![third], "t")
                .unwrap();
        }
        s.view()
    };
    // Not refuted: still held.
    let (_d, v) = build(false, "H4");
    assert!(drivers_disagree(&v, &f("F1")).is_some());
    // Refuted by an alloc-test on a third driver: settled.
    let (_d, v) = build(true, "H4");
    assert!(drivers_disagree(&v, &f("F1")).is_none());
    // Refuted by an alloc-test on the leak-favoring driver itself: it names no difference.
    let (_d, v) = build(true, "H2");
    assert!(drivers_disagree(&v, &f("F1")).is_some());
}

/// Records that F1 is explained as `removal: on-release`, by the line `a.py:30`.
fn explain_on_release(l: &mut Ledger) {
    let ws = evidence(l, EvidenceKind::Workload, json!({"harness": "H1"}));
    l.transition(&f("F1"), FindingState::UnderTest, None, None, vec![ws], "t")
        .unwrap();
    l.annotate_record(crate::record::FindingAnnotation {
        writers: vec!["a.py:9".to_owned()],
        removers: vec!["a.py:30".to_owned()],
        holder: Some(crate::finding::Holder::StaticGlobal),
        ..crate::record::FindingAnnotation::new(f("F1"))
    })
    .unwrap();
    l.explain(
        &f("F1"),
        crate::finding::ExplanationTag::Inferred,
        &["a.py:9".to_owned()],
        RemovalClaim::OnRelease,
        Some("a.py:30"),
        None,
        "The static/global holder at a.py:1 is added at a.py:9 and a.py:30 does the release of it only when the caller calls stop()",
        &["a.py:9".to_owned()],
        &["a.py:30".to_owned()],
    )
    .unwrap();
}

/// Adds the `N-RELEASE` sibling of F1 naming `remover` and decides it (`to`) on `by`.
fn decided_release(l: &mut Ledger, remover: &str, to: FindingState, by: EvidenceId) -> FindingId {
    let id = l.next_sibling_id(&FindingId::new(1));
    let mut nf = NewFinding::new(
        Origin::parse("hypothesis").unwrap(),
        vec![EvidenceId::new(1)],
    );
    nf.sibling_of = Some(FindingId::new(1));
    nf.hypothesis = Some("N-RELEASE".into());
    nf.refute_if = Some("the holder still grows with the release called".into());
    nf.remover = Some(remover.to_owned());
    l.add_finding(nf).unwrap();
    let ws = evidence(l, EvidenceKind::Workload, json!({"harness": "H1"}));
    l.transition(&id, FindingState::UnderTest, None, None, vec![ws], "t")
        .unwrap();
    l.transition(&id, to, None, None, vec![by], "t").unwrap();
    id
}

#[test]
fn the_same_remover_is_the_same_line_of_the_same_file() {
    assert!(same_remover("a.py:30", "a.py:30"));
    assert!(same_remover("./a.py:30", "a.py:30"));
    assert!(same_remover("src/a.py:30", "a.py:30"));
    assert!(same_remover(" a.py:30 ", "a.py:30"));
    assert!(!same_remover("a.py:31", "a.py:30"));
    assert!(!same_remover("b.py:30", "a.py:30"));
    assert!(!same_remover("xa.py:30", "a.py:30"));
    assert!(same_remover("stop", "stop"));
    assert!(!same_remover("stop", "a.py:30"));
}

#[test]
fn a_refuted_sibling_clears_an_untested_release_only_when_it_names_the_same_remover() {
    let mut s = scene("python");
    explain_on_release(&mut s.ledger);
    let leak = alloc_test(&mut s.ledger, "leak", "H1");
    let root = s.ledger.finding(&f("F1")).unwrap().clone();
    // F1.n2 names a.py:30 and is not decided: the release was not tested.
    assert!(release_untested(&s.ledger, &root).is_some());
    // A refuted sibling that names another release: still untested.
    decided_release(
        &mut s.ledger,
        "a.py:99",
        FindingState::Refuted,
        leak.clone(),
    );
    let u = release_untested(&s.ledger, &root).expect("another release decides nothing");
    assert_eq!(u.remover, "a.py:30");
    // A sibling of another file at the same line: still untested.
    decided_release(
        &mut s.ledger,
        "b.py:30",
        FindingState::Refuted,
        leak.clone(),
    );
    assert!(release_untested(&s.ledger, &root).is_some());
    // The one that names the same line of the same file (spelled with a directory) clears it.
    decided_release(&mut s.ledger, "src/a.py:30", FindingState::Refuted, leak);
    assert!(release_untested(&s.ledger, &root).is_none());
}

#[test]
fn a_confirmed_sibling_clears_an_untested_release_whatever_remover_it_names() {
    let mut s = scene("python");
    explain_on_release(&mut s.ledger);
    let _leak = alloc_test(&mut s.ledger, "leak", "H1");
    let flat = alloc_test(&mut s.ledger, FLAT, "H1");
    let root = s.ledger.finding(&f("F1")).unwrap().clone();
    assert!(release_untested(&s.ledger, &root).is_some());
    // The ledger closes the root first when the sibling is confirmed.
    s.ledger
        .transition(
            &f("F1"),
            FindingState::Refuted,
            None,
            None,
            vec![flat.clone()],
            "t",
        )
        .unwrap();
    // The explanation relies on a.py:30; the confirmed sibling names b.py:20. The family's
    // growth was measured flat with a release called: the finding grows only while callers skip
    // it, and the report counts it apart.
    decided_release(&mut s.ledger, "b.py:20", FindingState::Confirmed, flat);
    assert!(release_untested(&s.ledger, &root).is_none());
    assert!(release_confirmed(&s.ledger, &f("F1")).is_some());
}

#[test]
fn a_runtime_that_counts_lines_but_left_no_record_is_not_shown_to_have_run_the_release() {
    for rt in ["rust", "go", "node"] {
        let mut s = scene(rt);
        let flat = alloc_test(&mut s.ledger, FLAT, "H1");
        let (dir, view) = s.view();
        // No `executed.json`: no sign that the release ran.
        let e = decide(&view, &flat).unwrap_err();
        assert_eq!(e.code, codes::RELEASE_NOT_RUN, "{rt}");
        assert!(
            e.message.contains("no coverage record"),
            "{rt}: {}",
            e.message
        );
        assert!(
            e.fix
                .starts_with("dla alloc-test H1 --finding F1   (it counts a.py:30 and writes the coverage record)"),
            "{rt}: {}",
            e.fix
        );
        // A record that does not ask about the line (a run before the sibling named it).
        coverage(
            dir.path(),
            "H1",
            json!([{"file": "a.py", "line": 5, "count": 9}]),
        );
        let e = decide(&view, &flat).unwrap_err();
        assert_eq!(e.code, codes::RELEASE_NOT_RUN, "{rt}");
        assert!(e.message.contains("did not count"), "{rt}: {}", e.message);
        // A record that counted the line above 0: decided, no note.
        coverage(
            dir.path(),
            "H1",
            json!([{"file": "a.py", "line": 30, "count": 3}]),
        );
        assert_eq!(
            decide(&view, &flat).unwrap(),
            Decision { note: None },
            "{rt}"
        );
        // A record that says the tool could not count the line: the note.
        std::fs::write(
            HarnessManifest::dir_in(dir.path(), "H1").join("executed.json"),
            json!({"runtime": rt, "kind": "none", "functions": [], "note": "no coverage was written"})
                .to_string(),
        )
        .unwrap();
        let d = decide(&view, &flat).unwrap();
        assert_eq!(d.note.as_deref(), Some(NOT_SHOWN), "{rt}");
        std::fs::write(
            HarnessManifest::dir_in(dir.path(), "H1").join("executed.json"),
            json!({"runtime": rt, "kind": "functions",
                "lines": [{"file": "a.py", "line": 30, "count": null, "why": "inlined"}]})
            .to_string(),
        )
        .unwrap();
        let d = decide(&view, &flat).unwrap();
        assert_eq!(d.note.as_deref(), Some(NOT_SHOWN), "{rt}");
    }
}

#[test]
fn a_python_run_without_line_counts_of_its_series_is_refused_and_a_jvm_or_native_one_is_noted() {
    let mut s = scene("python");
    let flat = alloc_test(&mut s.ledger, FLAT, "H1");
    let (dir, view) = s.view();
    let e = decide(&view, &flat).unwrap_err();
    assert_eq!(e.code, codes::RELEASE_NOT_RUN);
    assert!(e.message.contains("no line counts"), "{}", e.message);
    assert!(
        e.fix.starts_with("dla alloc-test H1 --finding F1"),
        "{}",
        e.fix
    );
    // The counts of another series of the harness are not the counts of this run.
    LineRuns {
        dla_line_runs: 1,
        series: "S2".into(),
        sites: serde_json::from_value(json!({"a.py:30": [0, 4, 8]})).unwrap(),
        unavailable: None,
    }
    .save(dir.path(), "H1")
    .unwrap();
    assert!(decide(&view, &flat).is_err());
    LineRuns {
        dla_line_runs: 1,
        series: "S1".into(),
        sites: serde_json::from_value(json!({"a.py:30": [0, 4, 8]})).unwrap(),
        unavailable: None,
    }
    .save(dir.path(), "H1")
    .unwrap();
    assert_eq!(decide(&view, &flat).unwrap(), Decision { note: None });
    // Counts that are all unknown: the agent could not count the line.
    LineRuns {
        dla_line_runs: 1,
        series: "S1".into(),
        sites: serde_json::from_value(json!({"a.py:30": [null, null]})).unwrap(),
        unavailable: None,
    }
    .save(dir.path(), "H1")
    .unwrap();
    assert_eq!(
        decide(&view, &flat).unwrap().note.as_deref(),
        Some(NOT_SHOWN)
    );
    // The agent said it could not count lines in this run: the record carries the note and no
    // sites, which is not a count of 0 and not a missing record.
    LineRuns {
        dla_line_runs: 1,
        series: "S1".into(),
        sites: Default::default(),
        unavailable: Some(format!(
            "{}: another tool already traces this process (sys.settrace)",
            crate::harness::LINE_COUNTS_UNAVAILABLE
        )),
    }
    .save(dir.path(), "H1")
    .unwrap();
    assert_eq!(
        decide(&view, &flat).unwrap().note.as_deref(),
        Some(NOT_SHOWN)
    );
    assert!(matches!(
        remover_run(
            dir.path(),
            &HarnessManifest::load(dir.path(), "H1").unwrap(),
            Some("S1"),
            "a.py:30"
        ),
        RemoverRun::NotShown(why) if why.contains("another tool")
    ));
    // The JVM and native code cannot count a line at all.
    for rt in ["jvm", "native"] {
        let mut s = scene(rt);
        let flat = alloc_test(&mut s.ledger, FLAT, "H1");
        let (_d, view) = s.view();
        assert_eq!(
            decide(&view, &flat).unwrap().note.as_deref(),
            Some(NOT_SHOWN),
            "{rt}"
        );
    }
}

#[test]
fn the_newest_alloc_test_of_a_harness_is_the_newest_by_time_after_a_merge() {
    // H2 favors leak. A worker's alloc-test of H1 that favors leak is merged; then the core
    // records an alloc-test of H1 that favors non-leak. By evidence ID the worker's `w/E1`
    // sorts after the core's `E<n>`, so reading the ID order took the older, leak-favoring
    // one for the newest and the two drivers looked as if they agreed.
    let mut s = scene("python");
    let leak_h2 = alloc_test(&mut s.ledger, "leak", "H2");
    let dir = s.dir.path().to_path_buf();
    let mut w = Ledger::load_for(&dir, Some("w")).unwrap();
    let old = alloc_test(&mut w, "leak", "H1");
    assert_eq!(old.to_string(), "w/E1");
    w.finish_slice().unwrap();
    crate::merge::merge(&dir).unwrap();
    s.ledger = Ledger::load(&dir).unwrap();
    let newest = alloc_test(&mut s.ledger, FLAT, "H1");
    assert!(newest.to_string().starts_with('E'));
    let (_d, view) = s.view();
    let drivers = drivers_of(&view, &f("F1"));
    let h1 = drivers.iter().find(|d| d.harness == "H1").unwrap();
    assert_eq!(
        (h1.evidence.clone(), h1.favors),
        (newest.clone(), Favors::NonLeak)
    );
    // drivers_of is oldest first: H2's, then the worker's alloc-test is gone (H1's newest moved).
    assert_eq!(drivers.last().unwrap().evidence, newest);
    let d = drivers_disagree(&view, &f("F1")).expect("H1 now favors non-leak, H2 leak");
    assert_eq!((d.flat.evidence, d.leak.evidence), (newest, leak_h2));
}

/// Time passes between two writes of a test, so the order of their `ts` is the order of the writes.
fn tick() {
    std::thread::sleep(std::time::Duration::from_millis(5));
}

#[test]
fn a_main_alloc_test_written_while_a_slice_is_open_is_the_newest_by_time_after_the_merge() {
    // The worker's alloc-test of H1 (leak) is recorded first, the core's of H1 (non-leak) while
    // the slice is still open, so it is newer; the merge appends the worker's line after it. By
    // the position of the line the worker's would be the newest and the two drivers (H1 flat,
    // H2 leak) would look as if they agreed.
    let mut s = scene("python");
    let leak_h2 = alloc_test(&mut s.ledger, "leak", "H2");
    let dir = s.dir.path().to_path_buf();
    let mut w = Ledger::load_for(&dir, Some("w")).unwrap();
    let old = alloc_test(&mut w, "leak", "H1");
    assert_eq!(old.to_string(), "w/E1");
    tick();
    let newest = alloc_test(&mut s.ledger, FLAT, "H1");
    w.finish_slice().unwrap();
    crate::merge::merge(&dir).unwrap();
    s.ledger = Ledger::load(&dir).unwrap();
    // The merge put the worker's line after the core's, and the key still says it is older.
    assert!(s.ledger.evidence_order(&old) < s.ledger.evidence_order(&newest));
    assert_eq!(
        s.ledger.evidence_in_order().last().unwrap().evidence_id,
        newest
    );
    let (_d, view) = s.view();
    let drivers = drivers_of(&view, &f("F1"));
    let h1 = drivers.iter().find(|d| d.harness == "H1").unwrap();
    assert_eq!(
        (h1.evidence.clone(), h1.favors),
        (newest.clone(), Favors::NonLeak)
    );
    let d = drivers_disagree(&view, &f("F1")).expect("H1 is flat now, H2 grows");
    assert_eq!((d.flat.evidence, d.leak.evidence), (newest, leak_h2));
}

#[test]
fn the_alloc_tests_of_two_workers_merged_in_the_opposite_order_of_their_times_stay_in_time_order() {
    // Slices merge by worker name: `a` before `b`. `b` recorded its alloc-test of H1 (leak) first
    // and `a` recorded its (non-leak) after it, so `a`'s is the newest although `b`'s line comes
    // later in the merged ledger.
    let mut s = scene("python");
    let leak_h2 = alloc_test(&mut s.ledger, "leak", "H2");
    let dir = s.dir.path().to_path_buf();
    let mut a = Ledger::load_for(&dir, Some("a")).unwrap();
    let mut b = Ledger::load_for(&dir, Some("b")).unwrap();
    let by_b = alloc_test(&mut b, "leak", "H1");
    tick();
    let by_a = alloc_test(&mut a, FLAT, "H1");
    a.finish_slice().unwrap();
    b.finish_slice().unwrap();
    crate::merge::merge(&dir).unwrap();
    s.ledger = Ledger::load(&dir).unwrap();
    assert!(s.ledger.evidence_order(&by_b) < s.ledger.evidence_order(&by_a));
    assert!(
        s.ledger.evidence_order(&by_b).unwrap().at > s.ledger.evidence_order(&by_a).unwrap().at,
        "b's line is the later one in the file"
    );
    let (_d, view) = s.view();
    let d = drivers_disagree(&view, &f("F1")).expect("H1 is flat now, H2 grows");
    assert_eq!((d.flat.evidence, d.leak.evidence), (by_a, leak_h2));
}
