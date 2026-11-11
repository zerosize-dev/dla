//! `dla-bench gates` and `dla-bench close-calls`.

use std::{collections::BTreeMap, fs, path::Path};

use assert_cmd::Command;
use dla_bench::{
    closecalls,
    gates::{self, BaselineFile, Baselines, Inputs, ReleaseBlock, Stat, Status, make_baseline},
    grade,
    model::MetricsFile,
};
use predicates::prelude::*;
use serde_json::{Value, json};

fn ratio(runs: &[(u64, u64)]) -> Value {
    let per_run: Vec<Value> = runs
        .iter()
        .enumerate()
        .map(|(i, (n, d))| {
            json!({"run": i + 1, "num": n, "den": d,
            "value": if *d > 0 { json!(*n as f64 / *d as f64) } else { Value::Null }})
        })
        .collect();
    let (n, d): (u64, u64) = runs.iter().fold((0, 0), |a, r| (a.0 + r.0, a.1 + r.1));
    json!({"num": n, "den": d, "value": null, "per_run": per_run, "mean": null, "stdev": null})
}

/// Three runs; each pair is (num, den) of one run.
fn flat(num: u64, den: u64) -> Value {
    ratio(&[(num, den), (num, den), (num, den)])
}

fn decoy_row(pass: bool) -> Value {
    json!({"pairs": ["a/b"],
        "leak": {"found_runs": if pass { 3 } else { 2 }, "runs": 3},
        "decoy": {"claim_runs": 0, "runs": 3}, "pass": pass})
}

fn fixture(runs: u64, tokens: u64) -> Value {
    json!({"category": "x", "runs": runs, "claim_runs": 0, "rule_out_runs": 0, "leaks": {},
        "tokens_total": tokens, "agreement": {"runs": runs, "outcomes": [], "unanimous": true}})
}

/// `tokens` is the tokens per run of each smoke fixture.
fn group(abl: &str, arch: &str, m: &Value, tokens: u64) -> Value {
    json!({"ablation": abl, "archetypes": arch, "metrics": m,
        "unexplained_growth_share": {"value": null, "runs": 0},
        "usage": {"runs": 3, "incomplete": 0, "errored": 0,
            "sources": {"ledger": 3, "findings": 0, "none": 0},
            "input_tokens": {"sum": 0, "mean": 0.0}, "output_tokens": {"sum": 0, "mean": 0.0},
            "cache_read_input_tokens": {"sum": 0, "mean": 0.0},
            "cache_creation_input_tokens": {"sum": 0, "mean": 0.0},
            "wall_seconds": {"sum": 0.0, "mean": 0.0, "max": 0.0},
            "cost_usd": {"sum": 0.0, "mean": null}},
        "decoys": {"L-COLLECT": decoy_row(true)},
        "fixtures": {
            "native-vendor-codec": fixture(3, tokens * 3),
            "python-report-memo": fixture(3, tokens * 3),
            "python-report-memo-bounded": fixture(3, tokens * 3)}})
}

/// A metrics object that passes every gate against `baseline()`.
fn good_metrics() -> Value {
    json!({
        "leak_recall": flat(4, 4),
        "archetype_free_recall": flat(2, 2),
        "flagged_recall": flat(4, 4),
        "second_leak_recall": flat(2, 2),
        "grounded_attribution": flat(4, 4),
        "label_accuracy": flat(3, 4),
        "forced_fit_rate": flat(0, 2),
        "overclaim_rate": flat(0, 2),
        "false_confirmation_rate": flat(0, 4),
        "non_leak_precision": flat(2, 2),
        "trap_miss_rate": flat(0, 4),
        "trap_false_alarm_rate": flat(0, 4),
        "source_overclaim_rate": flat(0, 1),
        "undiscriminated_refutations": flat(0, 3),
        "uncited_findings": flat(0, 10),
        "uncited_checklist_answers": flat(0, 8),
        "dropped_rate": flat(0, 10),
    })
}

fn file(groups: Vec<Value>) -> MetricsFile {
    serde_json::from_value(json!({"schema_version": 1, "groups": groups})).expect("metrics parse")
}

fn good() -> MetricsFile {
    file(vec![
        group("full", "on", &good_metrics(), 1000),
        group("gates", "on", &good_metrics(), 1000),
        group("gates", "off", &good_metrics(), 1000),
    ])
}

fn baseline() -> BaselineFile {
    BaselineFile {
        schema_version: 1,
        baselines: Baselines {
            second_leak_recall: Some(1.0),
            forced_fit_rate: Some(0.0),
            non_leak_precision: Some(1.0),
        },
        release: None,
    }
}

fn smoke() -> Vec<String> {
    gates::SMOKE_FIXTURES.map(String::from).to_vec()
}

fn run(
    m: &MetricsFile,
    b: Option<&BaselineFile>,
    skill_lines: Option<usize>,
) -> Vec<gates::GateResult> {
    gates::evaluate(&Inputs {
        metrics: m,
        baseline: b,
        ablation: "gates",
        recall_ablation: "gates",
        writing: false,
        smoke: &smoke(),
        skill_lines,
        shipped_rows: None,
    })
}

fn failed(r: &[gates::GateResult]) -> Vec<u8> {
    r.iter()
        .filter(|g| g.status == Status::Fail)
        .map(|g| g.n)
        .collect()
}

fn with_metric(name: &str, v: Value, on_only: bool) -> MetricsFile {
    let mut on = good_metrics();
    on[name] = v.clone();
    let mut off = good_metrics();
    if !on_only {
        off[name] = v;
    }
    file(vec![
        group("gates", "on", &on, 1000),
        group("gates", "off", &off, 1000),
    ])
}

#[test]
fn every_gate_passes_on_good_metrics() {
    let r = run(&good(), Some(&baseline()), Some(123));
    assert_eq!(r.len(), 10);
    assert!(failed(&r).is_empty(), "{r:#?}");
}

#[test]
fn gate_1_one_claim_on_an_impossible_fixture_fails() {
    let m = with_metric("overclaim_rate", ratio(&[(0, 2), (1, 2), (0, 2)]), true);
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![1]);
}

#[test]
fn gate_2_recall_with_archetypes_on_must_not_be_below_off() {
    let mut on = good_metrics();
    on["leak_recall"] = ratio(&[(3, 4), (4, 4), (4, 4)]);
    let m = file(vec![
        group("gates", "on", &on, 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    let r = run(&m, Some(&baseline()), Some(10));
    assert_eq!(failed(&r), vec![2]);
    // Equal is enough.
    let m = file(vec![
        group("gates", "on", &good_metrics(), 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    assert!(failed(&run(&m, Some(&baseline()), Some(10))).is_empty());
}

#[test]
fn gate_2_needs_three_runs_and_the_off_arm() {
    let mut on = good_metrics();
    on["leak_recall"] = ratio(&[(4, 4), (4, 4)]);
    let m = file(vec![
        group("gates", "on", &on, 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![2]);
    let m = file(vec![group("gates", "on", &good_metrics(), 1000)]);
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![2]);
}

#[test]
fn gate_2_extra_close_call_runs_do_not_enter_the_mean() {
    // Runs 4 and 5 hold only the close-call fixtures; they must not move the base mean.
    let mut on = good_metrics();
    on["leak_recall"] = ratio(&[(4, 4), (4, 4), (4, 4), (0, 1), (0, 1)]);
    let m = file(vec![
        group("gates", "on", &on, 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    assert!(failed(&run(&m, Some(&baseline()), Some(10))).is_empty());
}

#[test]
fn gates_3_4_and_6_compare_with_the_baseline() {
    let m = with_metric(
        "second_leak_recall",
        ratio(&[(1, 2), (2, 2), (2, 2)]),
        false,
    );
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![3]);
    let m = with_metric("forced_fit_rate", ratio(&[(0, 2), (1, 2), (0, 2)]), false);
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![4]);
    let m = with_metric(
        "non_leak_precision",
        ratio(&[(2, 2), (1, 2), (2, 2)]),
        false,
    );
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![6]);
    // Without a baseline file the three gates cannot pass.
    assert_eq!(failed(&run(&good(), None, Some(10))), vec![3, 4, 6]);
}

#[test]
fn gate_5_a_failing_row_or_a_missing_shipped_row_fails() {
    let mut g = group("gates", "on", &good_metrics(), 1000);
    g["decoys"]["L-REGISTER"] = decoy_row(false);
    let m = file(vec![g, group("gates", "off", &good_metrics(), 1000)]);
    let r = run(&m, Some(&baseline()), Some(10));
    assert_eq!(failed(&r), vec![5]);
    assert!(r[4].detail.iter().any(|d| d.contains("L-REGISTER")));
    let rows = vec!["L-COLLECT".to_owned(), "L-TLS".to_owned()];
    let r = gates::evaluate(&Inputs {
        metrics: &good(),
        baseline: Some(&baseline()),
        ablation: "gates",
        recall_ablation: "gates",
        writing: false,
        smoke: &smoke(),
        skill_lines: Some(10),
        shipped_rows: Some(&rows),
    });
    assert_eq!(failed(&r), vec![5]);
    assert!(
        r[4].detail
            .iter()
            .any(|d| d.contains("L-TLS") && d.contains("no pair"))
    );
}

#[test]
fn gate_6_a_refutation_without_a_discriminator_fails() {
    let m = with_metric(
        "undiscriminated_refutations",
        ratio(&[(0, 3), (1, 3), (0, 3)]),
        true,
    );
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![6]);
}

#[test]
fn gate_7_any_uncited_finding_or_answer_fails() {
    let m = with_metric(
        "uncited_findings",
        ratio(&[(0, 10), (0, 10), (1, 10)]),
        true,
    );
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![7]);
    let m = with_metric(
        "uncited_checklist_answers",
        ratio(&[(1, 8), (0, 8), (0, 8)]),
        true,
    );
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![7]);
}

#[test]
fn gate_8_a_dropped_run_fails_and_no_complete_report_is_no_data() {
    let m = with_metric("dropped_rate", ratio(&[(0, 10), (1, 10), (0, 10)]), true);
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![8]);
    let m = with_metric("dropped_rate", ratio(&[(0, 0), (0, 0), (0, 0)]), true);
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![8]);
}

#[test]
fn gate_9_skill_length_and_tokens_against_the_previous_release() {
    assert_eq!(failed(&run(&good(), Some(&baseline()), Some(200))), vec![9]);
    assert!(failed(&run(&good(), Some(&baseline()), Some(199))).is_empty());
    assert_eq!(failed(&run(&good(), Some(&baseline()), None)), vec![9]);
    let mut b = baseline();
    b.release = Some(ReleaseBlock {
        label: None,
        ablation: "gates".into(),
        arms: BTreeMap::from([("gates/on".to_owned(), BTreeMap::new())]),
        tokens_per_run: [("gates/on".to_owned(), 1000.0 / 1.10 - 1.0)].into(),
    });
    // 1000 tokens per run against a previous 908.09 (+10% = 998.9): over.
    assert_eq!(failed(&run(&good(), Some(&b), Some(10))), vec![9]);
    b.release.as_mut().unwrap().tokens_per_run = [("gates/on".to_owned(), 1000.0 / 1.10)].into();
    assert!(
        failed(&run(&good(), Some(&b), Some(10))).is_empty(),
        "exactly +10% is allowed"
    );
}

#[test]
fn gate_9_needs_smoke_runs_in_the_judged_arm() {
    let mut g = group("gates", "on", &good_metrics(), 1000);
    g["fixtures"] = json!({});
    let m = file(vec![g, group("gates", "off", &good_metrics(), 1000)]);
    assert_eq!(failed(&run(&m, Some(&baseline()), Some(10))), vec![9]);
}

fn arm_stat(
    name: &str,
    mean: f64,
    stdev: f64,
    den_run: f64,
) -> BTreeMap<String, BTreeMap<String, Stat>> {
    BTreeMap::from([(
        "gates/on".to_owned(),
        BTreeMap::from([(
            name.to_owned(),
            Stat {
                mean,
                stdev,
                den_run,
            },
        )]),
    )])
}

fn release(mean: f64, stdev: f64, den_run: f64) -> BaselineFile {
    let mut b = baseline();
    b.release = Some(ReleaseBlock {
        label: Some("prev".into()),
        ablation: "gates".into(),
        arms: [(
            "gates/on".to_owned(),
            BTreeMap::from([(
                "leak_recall".to_owned(),
                Stat {
                    mean,
                    stdev,
                    den_run,
                },
            )]),
        )]
        .into(),
        tokens_per_run: Default::default(),
    });
    b
}

#[test]
fn gate_10_tolerance_is_the_larger_of_two_stdevs_and_one_runs_share() {
    // Now: leak_recall mean 0.75 (3/4 each run), den 4 per run, stdev 0.
    let now = with_metric("leak_recall", flat(3, 4), true);
    // Previous 0.95, stdev 0: tolerance 1/4 = 0.25; worse by 0.20: allowed.
    assert!(
        failed(&run(&now, Some(&release(0.95, 0.0, 4.0)), Some(10)))
            .iter()
            .all(|n| *n != 10)
    );
    // Previous 1.0: worse by 0.25 is still allowed (not more than).
    assert!(
        failed(&run(&now, Some(&release(1.0, 0.0, 4.0)), Some(10)))
            .iter()
            .all(|n| *n != 10)
    );
    // Previous 1.01: worse by 0.26: regression.
    assert!(failed(&run(&now, Some(&release(1.01, 0.0, 4.0)), Some(10))).contains(&10));
    // A wide previous stdev widens the tolerance to 2 x 0.2 = 0.4.
    assert!(
        failed(&run(&now, Some(&release(1.1, 0.2, 4.0)), Some(10)))
            .iter()
            .all(|n| *n != 10)
    );
    // A lower rate is better: forced_fit_rate going up is the regression.
    let up = with_metric("forced_fit_rate", flat(1, 2), true);
    let mut b = release(0.0, 0.0, 2.0);
    b.release.as_mut().unwrap().arms = arm_stat("forced_fit_rate", 0.0, 0.0, 2.0);
    // 0.5 worse than 0 with tolerance 1/2: allowed; with den 3 per run it is not.
    assert!(
        failed(&run(&up, Some(&b), Some(10)))
            .iter()
            .all(|n| *n != 10)
    );
    let up3 = with_metric("forced_fit_rate", flat(2, 3), true);
    b.release.as_mut().unwrap().arms = arm_stat("forced_fit_rate", 0.0, 0.0, 3.0);
    assert!(failed(&run(&up3, Some(&b), Some(10))).contains(&10));
}

#[test]
fn gate_10_without_a_previous_release_passes_with_a_note() {
    let r = run(&good(), Some(&baseline()), Some(10));
    assert_eq!(r[9].status, Status::Pass);
    assert!(
        r[9].detail
            .iter()
            .any(|d| d.contains("no previous release"))
    );
}

#[test]
fn a_baseline_never_gets_looser() {
    let mut lower = good_metrics();
    lower["second_leak_recall"] = flat(1, 2);
    lower["forced_fit_rate"] = flat(1, 2);
    lower["non_leak_precision"] = flat(1, 2);
    let m = file(vec![group("gates", "on", &lower, 1000)]);
    let b = make_baseline(&m, "gates", &smoke(), Some(&baseline()), true, None).unwrap();
    assert_eq!(
        b.baselines,
        baseline().baselines,
        "the old, stricter baselines stay"
    );
    // From scratch they are the 3-run means.
    let b = make_baseline(&m, "gates", &smoke(), None, true, None).unwrap();
    assert_eq!(b.baselines.second_leak_recall, Some(0.5));
    assert_eq!(b.baselines.forced_fit_rate, Some(0.5));
    // A stricter measurement tightens them.
    let b2 = make_baseline(&good(), "gates", &smoke(), Some(&b), true, None).unwrap();
    assert_eq!(b2.baselines.second_leak_recall, Some(1.0));
    assert_eq!(b2.baselines.forced_fit_rate, Some(0.0));
    assert_eq!(b2.baselines.non_leak_precision, Some(1.0));
}

#[test]
fn the_release_block_holds_stats_and_smoke_tokens() {
    let b = make_baseline(&good(), "gates", &smoke(), None, false, Some("1.0.0-rc.1")).unwrap();
    let rel = b.release.expect("release block");
    assert_eq!(rel.label.as_deref(), Some("1.0.0-rc.1"));
    assert_eq!(
        rel.arms["gates/on"]["leak_recall"],
        Stat {
            mean: 1.0,
            stdev: 0.0,
            den_run: 4.0
        }
    );
    assert_eq!(rel.tokens_per_run["gates/on"], 1000.0);
    assert_eq!(rel.tokens_per_run["gates/off"], 1000.0);
    // --baselines-only keeps whatever block was there.
    let kept = make_baseline(
        &good(),
        "gates",
        &smoke(),
        Some(&release(0.5, 0.0, 4.0)),
        true,
        None,
    )
    .unwrap();
    assert_eq!(kept.release.unwrap().label.as_deref(), Some("prev"));
}

#[test]
fn smoke_tokens_are_per_run_over_the_smoke_fixtures_only() {
    let mut g = group("gates", "on", &good_metrics(), 600);
    g["fixtures"]["other-fixture"] = fixture(3, 999_999);
    let m = file(vec![g]);
    // Each smoke fixture has 3 runs of 600 tokens.
    assert_eq!(
        gates::smoke_tokens_per_run(&m.groups[0], &smoke()),
        Some(600.0)
    );
}

#[test]
fn stat_uses_only_the_base_runs() {
    let m = file(vec![group("gates", "on", &good_metrics(), 1)]);
    let r = ratio(&[(1, 2), (2, 2), (1, 2), (0, 1), (0, 1)]);
    let r: dla_bench::model::Ratio = serde_json::from_value(r).unwrap();
    let s = gates::stat(&r, 3).unwrap();
    assert!((s.mean - (0.5 + 1.0 + 0.5) / 3.0).abs() < 1e-12);
    assert!((s.den_run - 2.0).abs() < 1e-12);
    drop(m);
}

// ---------------------------------------------------------------- the commands

fn bin() -> Command {
    Command::cargo_bin("dla-bench").unwrap()
}

fn write_json(p: &Path, v: &Value) {
    fs::write(p, serde_json::to_string_pretty(v).unwrap()).unwrap();
}

#[test]
fn the_gates_command_prints_the_table_and_exits_one_when_a_gate_fails() {
    let t = tempfile::tempdir().unwrap();
    let (mf, bf, skill) = (
        t.path().join("m.json"),
        t.path().join("b.json"),
        t.path().join("SKILL.md"),
    );
    write_json(&mf, &serde_json::to_value(good()).unwrap());
    write_json(&bf, &serde_json::to_value(baseline()).unwrap());
    fs::write(&skill, "line\n".repeat(120)).unwrap();
    let ok = bin()
        .args(["gates"])
        .arg(&mf)
        .arg("--baseline")
        .arg(&bf)
        .arg("--skill")
        .arg(&skill)
        .arg("--rows")
        .arg(t.path().join("none.toml"))
        .assert()
        .success()
        .stdout(predicate::str::contains("10 of 10 gates pass"));
    drop(ok);
    fs::write(&skill, "line\n".repeat(250)).unwrap();
    bin()
        .args(["gates"])
        .arg(&mf)
        .arg("--baseline")
        .arg(&bf)
        .arg("--skill")
        .arg(&skill)
        .arg("--rows")
        .arg(t.path().join("none.toml"))
        .assert()
        .code(1)
        .stdout(
            predicate::str::contains("FAIL  context cost")
                .and(predicate::str::contains("9 of 10 gates pass")),
        );
}

#[test]
fn write_baseline_writes_the_file_and_judges_against_it() {
    let t = tempfile::tempdir().unwrap();
    let (mf, bf, skill) = (
        t.path().join("m.json"),
        t.path().join("b.json"),
        t.path().join("SKILL.md"),
    );
    write_json(&mf, &serde_json::to_value(good()).unwrap());
    fs::write(&skill, "x\n").unwrap();
    bin()
        .args(["gates"])
        .arg(&mf)
        .arg("--write-baseline")
        .arg(&bf)
        .arg("--label")
        .arg("m7-baseline")
        .arg("--skill")
        .arg(&skill)
        .arg("--rows")
        .arg(t.path().join("none.toml"))
        .assert()
        .success()
        .stdout(predicate::str::contains("baseline written"));
    let written: BaselineFile = serde_json::from_str(&fs::read_to_string(&bf).unwrap()).unwrap();
    assert_eq!(written.baselines, baseline().baselines);
    assert_eq!(
        written.release.unwrap().label.as_deref(),
        Some("m7-baseline")
    );
}

#[test]
fn a_missing_metrics_file_is_an_error_not_a_failed_gate() {
    bin()
        .args(["gates", "/nonexistent/metrics.json"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("metrics.json"));
}

fn m7_sample() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/m7-sample")
}

#[test]
fn close_calls_lists_the_disagreeing_fixtures_and_the_command_for_runs_4_and_5() {
    let s = m7_sample();
    let m = grade::grade_dir(&s.join("results"), &s.join("fixtures")).unwrap();
    let calls = closecalls::find(&m, &["gates/on".to_owned()]).unwrap();
    let ids: Vec<&str> = calls.iter().map(|c| c.fixture.as_str()).collect();
    assert_eq!(ids, ["m7-decoy2", "m7-multi", "m7-pair2"]);
    let multi = calls.iter().find(|c| c.fixture == "m7-multi").unwrap();
    assert_eq!(multi.runs, 3);
    assert_eq!(multi.outcomes[1], (2, "found:L1".to_owned()));
    let refs: Vec<&closecalls::CloseCall> = calls.iter().collect();
    let cmd = closecalls::command("gates", "on", &refs, Path::new("bench/results/m7")).unwrap();
    assert_eq!(
        cmd,
        "dla-bench run --ablation gates --archetypes on --fixtures m7-decoy2,m7-multi,m7-pair2 --runs 5 --out bench/results/m7"
    );
    // The other arm has no close call (one run per fixture).
    assert!(
        closecalls::find(&m, &["no-skill".to_owned()])
            .unwrap()
            .is_empty()
    );
    assert!(
        closecalls::find(&m, &["gates/off".to_owned()])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_fixture_with_five_runs_is_listed_but_not_run_again() {
    let call = closecalls::CloseCall {
        ablation: "gates".into(),
        archetypes: "on".into(),
        fixture: "f".into(),
        runs: 5,
        outcomes: vec![],
    };
    assert!(closecalls::command("gates", "on", &[&call], Path::new("o")).is_none());
}

#[test]
fn the_close_calls_command_grades_a_results_directory_and_prints_the_command() {
    let s = m7_sample();
    bin()
        .arg("close-calls")
        .arg(s.join("results"))
        .arg("--fixtures")
        .arg(s.join("fixtures"))
        .args(["--arm", "gates"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("gates archetypes-on: 3 close call(s)")
                .and(predicate::str::contains(
                    "m7-multi: #1 found:L1,L2 | #2 found:L1 | #3 found:L1,L2",
                ))
                .and(predicate::str::contains(
                    "--fixtures m7-decoy2,m7-multi,m7-pair2 --fixtures-dir",
                ))
                .and(predicate::str::contains("--model sample-model --runs 5")),
        );
    // A directory where every fixture agrees.
    bin()
        .arg("close-calls")
        .arg(s.join("results"))
        .arg("--fixtures")
        .arg(s.join("fixtures"))
        .args(["--arm", "no-skill"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no close calls"));
}

// ------------------------------------------------------- repairs: arms, baselines, commands

fn arms_good() -> MetricsFile {
    file(vec![
        group("full", "on", &good_metrics(), 1000),
        group("gates", "on", &good_metrics(), 1000),
        group("gates", "off", &good_metrics(), 1000),
    ])
}

fn run_full(m: &MetricsFile, b: Option<&BaselineFile>) -> Vec<gates::GateResult> {
    gates::evaluate(&Inputs {
        metrics: m,
        baseline: b,
        ablation: "full",
        recall_ablation: "gates",
        writing: false,
        smoke: &smoke(),
        skill_lines: Some(10),
        shipped_rows: None,
    })
}

#[test]
fn the_shipped_arm_is_judged_and_the_recall_gate_compares_gates_on_with_gates_off() {
    assert!(failed(&run_full(&arms_good(), Some(&baseline()))).is_empty());
    // A flaw in the shipped arm fails its gates, whatever the gates arm does.
    let mut bad = good_metrics();
    bad["overclaim_rate"] = flat(1, 2);
    bad["dropped_rate"] = flat(1, 10);
    let m = file(vec![
        group("full", "on", &bad, 1000),
        group("gates", "on", &good_metrics(), 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    assert_eq!(failed(&run_full(&m, Some(&baseline()))), vec![1, 8]);
    // Gate 2 reads gates/on against gates/off, not the shipped arm.
    let mut low = good_metrics();
    low["leak_recall"] = ratio(&[(3, 4), (3, 4), (3, 4)]);
    let m = file(vec![
        group("full", "on", &good_metrics(), 1000),
        group("gates", "on", &low, 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    assert_eq!(failed(&run_full(&m, Some(&baseline()))), vec![2]);
    // Without the gates arms, gate 2 cannot pass and says which group is missing.
    let m = file(vec![group("full", "on", &good_metrics(), 1000)]);
    let r = run_full(&m, Some(&baseline()));
    assert_eq!(failed(&r), vec![2]);
    assert!(r[1].detail[0].contains("gates/on"), "{:?}", r[1].detail);
}

#[test]
fn gate_9_without_token_counts_says_the_metrics_do_not_have_them() {
    let m = file(vec![
        group("full", "on", &good_metrics(), 0),
        group("gates", "on", &good_metrics(), 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    let r = run_full(&m, Some(&baseline()));
    assert_eq!(failed(&r), vec![9]);
    assert!(
        r[8].detail
            .iter()
            .any(|d| d.contains("tokens_total") && d.contains("do not have it")),
        "{:?}",
        r[8].detail
    );
    assert!(
        r[8].detail.iter().all(|d| !d.contains("full/on: 0 tokens")),
        "{:?}",
        r[8].detail
    );
    // A baseline cannot be written from such metrics: it would record 0.
    let e = make_baseline(&m, "full", &smoke(), None, false, None).unwrap_err();
    assert!(e.contains("no token counts"), "{e}");
}

#[test]
fn gate_9_reports_every_arm_and_judges_the_shipped_one() {
    let mut b = baseline();
    b.release = Some(ReleaseBlock {
        label: None,
        ablation: "full".into(),
        arms: BTreeMap::from([("full/on".to_owned(), BTreeMap::new())]),
        tokens_per_run: [
            ("full/on".to_owned(), 1000.0),
            ("gates/off".to_owned(), 500.0),
        ]
        .into(),
    });
    // gates/off doubled its tokens; the shipped arm did not: the gate passes and says so.
    let r = run_full(&arms_good(), Some(&b));
    assert!(failed(&r).is_empty(), "{r:#?}");
    assert!(
        r[8].detail
            .iter()
            .any(|d| d.starts_with("gates/off:") && d.contains("not judged")),
        "{:?}",
        r[8].detail
    );
    // The shipped arm over its limit fails.
    b.release
        .as_mut()
        .unwrap()
        .tokens_per_run
        .insert("full/on".into(), 800.0);
    assert_eq!(failed(&run_full(&arms_good(), Some(&b))), vec![9]);
}

fn release_arms(arms: &[(&str, f64, f64)]) -> BaselineFile {
    let mut b = baseline();
    b.release = Some(ReleaseBlock {
        label: Some("prev".into()),
        ablation: "full".into(),
        arms: arms
            .iter()
            .map(|(arm, mean, den)| {
                (
                    (*arm).to_owned(),
                    BTreeMap::from([(
                        "leak_recall".to_owned(),
                        Stat {
                            mean: *mean,
                            stdev: 0.0,
                            den_run: *den,
                        },
                    )]),
                )
            })
            .collect(),
        tokens_per_run: Default::default(),
    });
    b
}

#[test]
fn gate_10_compares_every_arm_present_in_both_files() {
    // The shipped arm is flat and gates/off regressed (now 0.5 against 1.0, share 1/4).
    let mut worse = good_metrics();
    worse["leak_recall"] = flat(2, 4);
    let m = file(vec![
        group("full", "on", &good_metrics(), 1000),
        group("gates", "on", &good_metrics(), 1000),
        group("gates", "off", &worse, 1000),
    ]);
    let b = release_arms(&[("full/on", 1.0, 4.0), ("gates/off", 1.0, 4.0)]);
    let r = run_full(&m, Some(&b));
    assert_eq!(failed(&r), vec![10]);
    assert!(
        r[9].detail[0].contains("gates/off leak_recall"),
        "{:?}",
        r[9].detail
    );
    // An arm that is only in one of the files is not compared; the others still are.
    let b = release_arms(&[("full/on", 1.0, 4.0), ("no-skill/on", 1.0, 4.0)]);
    assert!(failed(&run_full(&m, Some(&b))).is_empty());
    // No arm in common: nothing was compared, which is not a pass.
    let b = release_arms(&[("prose/on", 1.0, 4.0)]);
    assert_eq!(failed(&run_full(&m, Some(&b))), vec![10]);
}

#[test]
fn gate_10_allows_one_run_share_of_the_previous_release_not_the_smaller_denominator() {
    // Now 0.75 on 4 per run; previous 1.0 on 1000 per run: worse by 0.25 where one run's share
    // of the previous release is 0.001. The share of the smaller denominator (1/4) would let
    // it through.
    let now = with_metric("leak_recall", flat(3, 4), true);
    assert!(
        failed(&run(&now, Some(&release(1.0, 0.0, 4.0)), Some(10)))
            .iter()
            .all(|n| *n != 10)
    );
    assert!(failed(&run(&now, Some(&release(1.0, 0.0, 1000.0)), Some(10))).contains(&10));
}

#[test]
fn gate_5_on_metrics_without_a_decoys_block_says_to_grade_again() {
    let mut g = group("gates", "on", &good_metrics(), 1000);
    g["decoys"] = json!({});
    g["metrics"]
        .as_object_mut()
        .unwrap()
        .remove("uncited_findings");
    let m = file(vec![g, group("gates", "off", &good_metrics(), 1000)]);
    let rows = vec!["L-TLS".to_owned()];
    let r = gates::evaluate(&Inputs {
        metrics: &m,
        baseline: Some(&baseline()),
        ablation: "gates",
        recall_ablation: "gates",
        writing: false,
        smoke: &smoke(),
        skill_lines: Some(10),
        shipped_rows: Some(&rows),
    });
    assert!(failed(&r).contains(&5));
    assert_eq!(r[4].detail.len(), 1, "{:?}", r[4].detail);
    assert!(r[4].detail[0].contains("do not have the decoys block"));
    assert!(
        r[4].detail
            .iter()
            .all(|d| !d.contains("no pair was graded"))
    );
    // Metrics that do have the fields, but no graded pair, keep the plain message.
    let mut g = group("gates", "on", &good_metrics(), 1000);
    g["decoys"] = json!({});
    let m = file(vec![g, group("gates", "off", &good_metrics(), 1000)]);
    let r = run(&m, Some(&baseline()), Some(10));
    assert!(r[4].detail[0].contains("no archetype row has a decoy"));
}

fn gate_args(t: &Path, metrics: &Value, extra: &[&str]) -> Command {
    let (mf, skill) = (t.join("m.json"), t.join("SKILL.md"));
    write_json(&mf, metrics);
    fs::write(&skill, "x\n").unwrap();
    let mut c = bin();
    c.arg("gates")
        .arg(&mf)
        .arg("--skill")
        .arg(&skill)
        .arg("--rows")
        .arg(t.join("none.toml"))
        .args(extra);
    c
}

#[test]
fn write_baseline_judges_against_the_stored_file_before_writing() {
    let t = tempfile::tempdir().unwrap();
    let bf = t.path().join("b.json");
    // The stored file: strict baselines and a previous release with 1 token per run.
    let mut stored = baseline();
    stored.release = Some(ReleaseBlock {
        label: Some("prev".into()),
        ablation: "full".into(),
        arms: BTreeMap::from([("full/on".to_owned(), BTreeMap::new())]),
        tokens_per_run: [("full/on".to_owned(), 1.0)].into(),
    });
    write_json(&bf, &serde_json::to_value(&stored).unwrap());
    // Weaker metrics: second-leak recall 0.5 against the stored 1.0.
    let mut weak = good_metrics();
    weak["second_leak_recall"] = flat(1, 2);
    let m = file(vec![
        group("full", "on", &weak, 1000),
        group("gates", "on", &good_metrics(), 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    gate_args(
        t.path(),
        &serde_json::to_value(&m).unwrap(),
        &["--write-baseline", bf.to_str().unwrap()],
    )
    .assert()
    .code(1)
    .stdout(
        predicate::str::contains("second_leak_recall 0.5000 < baseline 1.0000")
            .and(predicate::str::contains("tokens per run >")),
    );
    // The stricter stored baselines survive the write.
    let after: BaselineFile = serde_json::from_str(&fs::read_to_string(&bf).unwrap()).unwrap();
    assert_eq!(after.baselines, stored.baselines);
}

#[test]
fn write_baseline_with_two_baselines_keeps_the_stricter_value_of_each() {
    let t = tempfile::tempdir().unwrap();
    let (bf, gf) = (t.path().join("b.json"), t.path().join("g.json"));
    // F is strict on second-leak recall, G on forced fits.
    let strict_f = BaselineFile {
        schema_version: 1,
        baselines: Baselines {
            second_leak_recall: Some(1.0),
            forced_fit_rate: Some(0.9),
            non_leak_precision: Some(0.2),
        },
        release: None,
    };
    let strict_g = BaselineFile {
        schema_version: 1,
        baselines: Baselines {
            second_leak_recall: Some(0.1),
            forced_fit_rate: Some(0.0),
            non_leak_precision: Some(0.2),
        },
        release: None,
    };
    write_json(&bf, &serde_json::to_value(&strict_f).unwrap());
    write_json(&gf, &serde_json::to_value(&strict_g).unwrap());
    let mut weak = good_metrics();
    weak["second_leak_recall"] = flat(1, 2);
    weak["forced_fit_rate"] = flat(1, 2);
    let m = file(vec![
        group("full", "on", &weak, 1000),
        group("gates", "on", &good_metrics(), 1000),
        group("gates", "off", &good_metrics(), 1000),
    ]);
    gate_args(
        t.path(),
        &serde_json::to_value(&m).unwrap(),
        &[
            "--write-baseline",
            bf.to_str().unwrap(),
            "--baseline",
            gf.to_str().unwrap(),
            "--baselines-only",
        ],
    )
    .assert()
    .code(1)
    .stdout(
        predicate::str::contains("second_leak_recall 0.5000 < baseline 1.0000").and(
            predicate::str::contains("forced_fit_rate 0.5000 > baseline 0.0000"),
        ),
    );
    let after: BaselineFile = serde_json::from_str(&fs::read_to_string(&bf).unwrap()).unwrap();
    assert_eq!(after.baselines.second_leak_recall, Some(1.0));
    assert_eq!(after.baselines.forced_fit_rate, Some(0.0));
    // A stricter measurement still tightens the value.
    assert_eq!(after.baselines.non_leak_precision, Some(1.0));
}

#[test]
fn the_first_baseline_run_passes_and_sets_the_values() {
    let t = tempfile::tempdir().unwrap();
    let bf = t.path().join("new.json");
    gate_args(
        t.path(),
        &serde_json::to_value(arms_good()).unwrap(),
        &["--write-baseline", bf.to_str().unwrap()],
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("this run sets it"));
    let written: BaselineFile = serde_json::from_str(&fs::read_to_string(&bf).unwrap()).unwrap();
    assert_eq!(written.baselines, baseline().baselines);
    // Without --write-baseline the missing baselines fail.
    gate_args(t.path(), &serde_json::to_value(arms_good()).unwrap(), &[])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("no second_leak_recall baseline"));
}

#[test]
fn close_calls_repeats_the_model_of_the_runs_and_a_non_default_fixtures_dir() {
    let s = m7_sample();
    let m = grade::grade_dir(&s.join("results"), &s.join("fixtures")).unwrap();
    let calls = closecalls::find(&m, &["gates/on".to_owned()]).unwrap();
    let refs: Vec<&closecalls::CloseCall> = calls.iter().collect();
    let models = closecalls::models_of(
        &s.join("results"),
        "gates",
        "on",
        &["m7-multi", "m7-decoy2"],
    );
    assert_eq!(models, ["sample-model"]);
    let cmd = closecalls::command_with(
        "gates",
        "on",
        &refs,
        Path::new("out"),
        closecalls::CommandOpts {
            model: Some("sample-model"),
            fixtures_dir: Some(Path::new("my/fixtures")),
        },
    )
    .unwrap();
    assert!(
        cmd.ends_with("--fixtures-dir my/fixtures --model sample-model --runs 5 --out out"),
        "{cmd}"
    );
    // The default directory is not repeated: no flag without a non-default value.
    let out = bin()
        .arg("close-calls")
        .arg(s.join("results"))
        .args(["--arm", "gates", "--fixtures"])
        .arg(s.join("fixtures"))
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("--model sample-model"));
}

#[test]
fn close_calls_warns_when_the_earlier_runs_used_several_models() {
    let t = tempfile::tempdir().unwrap();
    let s = m7_sample();
    let dest = t.path().join("results");
    copy_dir(&s.join("results"), &dest);
    let rj = dest.join("runs/gates/archetypes-on/m7-multi/2/run.json");
    let text = fs::read_to_string(&rj)
        .unwrap()
        .replace("sample-model", "other-model");
    fs::write(&rj, text).unwrap();
    bin()
        .arg("close-calls")
        .arg(&dest)
        .args(["--arm", "gates", "--fixtures"])
        .arg(s.join("fixtures"))
        .assert()
        .success()
        .stdout(
            predicate::str::contains("used several models (other-model, sample-model)")
                .and(predicate::str::contains("--model sample-model").not()),
        );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}
