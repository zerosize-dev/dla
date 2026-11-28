//! What the report says about the trigger of each confirmed or strong leak: the `Trigger:`
//! line, the counts at the end of the `Findings:` line and the order of the findings. The run
//! is a directory: the harnesses, series, run records and trend reports are read from it.

use std::path::Path;

use dla_core::{
    FindingId, Ledger, RunView,
    state::{RunState, now_utc},
    trend::{TargetVerdict, TrendReport, Verdict},
};
use dla_report::render_markdown;
use serde_json::json;

/// The `dynamic` fixture (F1 confirmed, `src/bus.go:12`) and a second confirmed leak F3
/// (`src/q.go:5`).
fn ledger_text() -> String {
    let base = std::fs::read_to_string(format!(
        "{}/tests/fixtures/dynamic.jsonl",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let ts = r#""schema_version":2,"ts":"2026-01-01T00:00:00Z""#;
    let extra = [
        r#""type":"evidence","evidence_id":"E7","source":{"repo":{"commit":"abc123"}},"tool":{"name":"dla","version":"0.1.0"},"authority":"test","kind":"growth-series","result":{"trend":"T1","series":"S1","axis":"n","n":8,"gate":"pass","failed":0,"passed":[{"column":"probe:Q.items","region":"heap","site":"src/q.go:5","finding":"F3","slope":300,"lo":280.0,"p":0.0001,"ratio":1.05,"shape":"steady","unit":"bytes"}]},"raw":"trend/T1.json""#,
        r#""type":"evidence","evidence_id":"E8","source":{"repo":{"commit":"abc123"}},"tool":{"name":"dla","version":"0.1.0"},"authority":"test","kind":"discriminator","result":{"test":"plateau+purge","favors":"leak","supports":["leak"],"finding":"F3"},"raw":"alloc-test/A2.json""#,
        r#""type":"finding_added","finding":{"id":"F3","state":"candidate","origin":"census","supports":["E6"],"site":"src/q.go:5","holder":"static/global","owner":"Q.items"}"#,
        r#""type":"finding_added","finding":{"id":"F3.n1","state":"candidate","origin":"hypothesis","supports":["E6"],"parent":"F3","hypothesis":"N-WARM: the queue fills during warm-up","refute_if":"growth continues"}"#,
        r#""type":"transition","finding":"F3.n1","from":"candidate","to":"refuted","evidence":["E8"],"by":"dla ledger refute""#,
        r#""type":"transition","finding":"F3","from":"candidate","to":"under_test","evidence":["E2"],"by":"dla ledger test""#,
        r#""type":"transition","finding":"F3","from":"under_test","to":"confirmed","evidence":["E7"],"by":"dla ledger confirm""#,
    ];
    let mut text = base;
    for (i, body) in extra.iter().enumerate() {
        text.push_str(&format!("{{{ts},\"seq\":{},{body}}}\n", 22 + i));
    }
    // The census facts on F1's writer (`Bus.add`): a start-up registration.
    text.push_str(&format!(
        "{{{ts},\"seq\":{},\"type\":\"finding_annotated\",\"finding\":\"F1\",\"writers\":[\"src/bus.go:12\"],\"removers\":[],\"writer_sites\":[{{\"site\":\"src/bus.go:12\",\"kind\":\"insert\",\"depth\":0,\"function\":\"add\",\"startup\":true,\"const\":true}}]}}\n",
        22 + extra.len()
    ));
    text
}

struct Run {
    dir: tempfile::TempDir,
}

impl Run {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        RunState::init("/p", None, 3600, None, now_utc())
            .unwrap()
            .save(dir.path())
            .unwrap();
        Self { dir }
    }

    fn harness(&self, id: &str, entry: bool, targets: &[&str], writer: Option<&str>) {
        let mut m = json!({
            "id": id, "runtime": "go", "mode": "driver",
            "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
            "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
            "targets": targets, "agent": "driver-go", "timeout_secs": 60,
        });
        if entry {
            m["purpose"] = json!("entry-path");
        }
        let m: dla_core::HarnessManifest = serde_json::from_value(m).unwrap();
        m.save(self.dir.path()).unwrap();
        if let Some(w) = writer {
            let t: Vec<_> = targets
                .iter()
                .map(|f| json!({"finding": f, "writer_functions": [w]}))
                .collect();
            std::fs::write(
                self.dir
                    .path()
                    .join("harness")
                    .join(id)
                    .join("targets.json"),
                json!({"harness": id, "targets": t}).to_string(),
            )
            .unwrap();
        }
    }

    fn ran(&self, series: &str, h: &str, digest: &str, verdicts: &[(u32, Verdict)]) {
        let d = self.dir.path();
        std::fs::create_dir_all(d.join("series")).unwrap();
        std::fs::write(
            d.join(format!("series/{series}.json")),
            json!({"id": series, "source": {"kind": "harness", "harness": h},
                "axis": "n", "columns": [], "points": []})
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            d.join("harness").join(h).join(format!("run-{series}.json")),
            json!({"series": [series], "driver_digest": digest, "varies": format!("{h} varies")})
                .to_string(),
        )
        .unwrap();
        TrendReport {
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
                    writers: Vec::new(),
                })
                .collect(),
        }
        .save(d)
        .unwrap();
    }

    /// A series `dla summarize` wrote from the captures of `h` (source kind `dump`), and its
    /// trend report.
    fn captured(&self, series: &str, h: &str, verdicts: &[(u32, Verdict)]) {
        self.ran(series, h, "cc", verdicts);
        let d = self.dir.path();
        std::fs::write(
            d.join(format!("series/{series}.json")),
            json!({"id": series, "source": {"kind": "dump", "harness": h},
                "axis": "n", "columns": [], "points": []})
            .to_string(),
        )
        .unwrap();
        // No run of the harness wrote it.
        std::fs::remove_file(d.join("harness").join(h).join(format!("run-{series}.json"))).unwrap();
    }

    fn report(&self) -> String {
        self.report_with(|_| {})
    }

    fn report_with(&self, f: impl FnOnce(&mut RunState)) -> String {
        let mut st = RunState::load(self.dir.path()).unwrap();
        f(&mut st);
        render_markdown(&view(self.dir.path(), st))
    }
}

fn view(dir: &Path, st: RunState) -> RunView {
    RunView::new(st, Ledger::from_text(&ledger_text()).unwrap(), None).with_disk_facts(dir)
}

fn findings_line(md: &str) -> &str {
    md.lines().find(|l| l.starts_with("Findings:")).unwrap()
}

/// The order of the `### F<n>` headings.
fn order(md: &str) -> Vec<&str> {
    md.lines()
        .filter_map(|l| l.strip_prefix("### "))
        .filter_map(|l| l.split(':').next())
        .filter(|id| *id == "F1" || *id == "F3")
        .collect()
}

#[test]
fn a_reproduced_trigger_names_the_harness_and_what_a_caller_varies() {
    let run = Run::new();
    run.harness("H1", false, &["F1", "F3"], None);
    run.ran("S1", "H1", "aa", &[(1, Verdict::Grew), (3, Verdict::Grew)]);
    run.harness("H2", true, &["F1"], Some("Bus.add"));
    run.ran("S2", "H2", "bb", &[(1, Verdict::Grew)]);
    let md = run.report();
    assert!(
        md.contains("- Trigger: reproduced through the public API in H2 (varies: H2 varies)"),
        "{md}"
    );
    // F3 grew only under a hook driver.
    assert!(
        md.contains("- Trigger: not checked (no entry-path harness was run for it)"),
        "{md}"
    );
    assert!(
        findings_line(&md).ends_with(
            "confirmed and strong by trigger: reproduced through the public API 1, no trigger found 0, trigger not checked 1"
        ),
        "{}",
        findings_line(&md)
    );
}

#[test]
fn no_trigger_found_after_two_different_entry_path_harnesses_says_what_was_tried() {
    let run = Run::new();
    run.harness("H1", false, &["F1", "F3"], None);
    run.ran("S1", "H1", "aa", &[(1, Verdict::Grew), (3, Verdict::Grew)]);
    run.harness("H2", true, &["F3"], Some("Q.push"));
    run.ran("S2", "H2", "bb", &[(3, Verdict::Constant)]);
    run.harness("H3", true, &["F3"], Some("Q.push"));
    run.ran("S3", "H3", "cc", &[(3, Verdict::Constant)]);
    let md = run.report();
    assert!(
        md.contains("- Trigger: not found: it grows when the harness passes new keys to Q.push; two workloads through the public API did not grow it (H2: H2 varies; H3: H3 varies)"),
        "{md}"
    );
    assert!(
        findings_line(&md).ends_with(
            "reproduced through the public API 0, no trigger found 1, trigger not checked 1"
        ),
        "{}",
        findings_line(&md)
    );
}

#[test]
fn two_entry_path_runs_that_did_not_measure_the_holder_are_not_checked_not_not_found() {
    let run = Run::new();
    run.harness("H1", false, &["F1", "F3"], None);
    run.ran("S1", "H1", "aa", &[(1, Verdict::Grew), (3, Verdict::Grew)]);
    run.harness("H7", true, &["F3"], Some("Q.push"));
    run.ran("S2", "H7", "bb", &[(3, Verdict::Executed)]);
    run.harness("H9", true, &["F3"], Some("Q.push"));
    run.ran("S3", "H9", "cc", &[(3, Verdict::Unknown)]);
    let md = run.report();
    assert!(
        md.contains("- Trigger: not checked: the entry-path harnesses H7 and H9 ran, but nothing measured the holder there (verdicts: executed, unknown)"),
        "{md}"
    );
    assert!(!md.contains("did not grow it"), "{md}");
    assert!(
        findings_line(&md).ends_with(
            "reproduced through the public API 0, no trigger found 0, trigger not checked 2"
        ),
        "{}",
        findings_line(&md)
    );
}

#[test]
fn reproduced_leaks_are_listed_first() {
    let run = Run::new();
    // F1 is first in the ledger; F3 is the reproduced one.
    run.harness("H1", false, &["F1", "F3"], None);
    run.ran("S1", "H1", "aa", &[(1, Verdict::Grew), (3, Verdict::Grew)]);
    run.harness("H2", true, &["F3"], Some("Q.push"));
    run.ran("S2", "H2", "bb", &[(3, Verdict::Grew)]);
    assert_eq!(order(&run.report()), ["F3", "F1"]);
    // With nothing reproduced the ledger order stands.
    let plain = Run::new();
    plain.harness("H1", false, &["F1", "F3"], None);
    plain.ran("S1", "H1", "aa", &[(1, Verdict::Grew), (3, Verdict::Grew)]);
    assert_eq!(order(&plain.report()), ["F1", "F3"]);
}

#[test]
fn a_trigger_that_was_not_checked_says_why() {
    let run = Run::new();
    run.harness("H1", false, &["F1", "F3"], None);
    run.ran("S1", "H1", "aa", &[(1, Verdict::Grew), (3, Verdict::Grew)]);
    let off = run.report_with(|st| st.queue = false);
    assert!(
        off.contains("- Trigger: not checked (the run was started with --queue off)"),
        "{off}"
    );
    let floor = run.report_with(|st| {
        st.budget.deadline = now_utc() + chrono::TimeDelta::seconds(100);
    });
    assert!(
        floor.contains(
            "- Trigger: not checked (not enough time was left: the budget was at the floor)"
        ),
        "{floor}"
    );
    // Nothing grew in a harness for F3.
    let none = Run::new();
    none.harness("H1", false, &["F1"], None);
    none.ran("S1", "H1", "aa", &[(1, Verdict::Grew)]);
    assert!(
        none.report()
            .contains("- Trigger: not checked (no harness series that grew is tied to it)")
    );
}

#[test]
fn a_trigger_with_one_entry_path_harness_without_growth_says_what_is_missing() {
    let run = Run::new();
    run.harness("H1", false, &["F1", "F3"], None);
    run.ran("S1", "H1", "aa", &[(1, Verdict::Grew), (3, Verdict::Grew)]);
    run.harness("H2", true, &["F3"], Some("Q.push"));
    run.ran("S2", "H2", "bb", &[(3, Verdict::Constant)]);
    let md = run.report();
    assert!(
        md.contains("- Trigger: not checked (the entry-path harness H2 did not grow it (varies: H2 varies) and a second, different one was not run)"),
        "{md}"
    );
}

#[test]
fn a_view_without_a_run_directory_says_nothing_about_triggers() {
    let st = RunState::init("/p", None, 3600, None, now_utc()).unwrap();
    let md = render_markdown(&RunView::new(
        st,
        Ledger::from_text(&ledger_text()).unwrap(),
        None,
    ));
    assert!(!md.contains("Trigger:"), "{md}");
    assert!(!findings_line(&md).contains("by trigger"));
}

#[test]
fn growth_by_calling_the_registration_function_itself_has_its_own_reading_and_count() {
    let run = Run::new();
    run.harness("H2", true, &["F1", "F3"], Some("Bus.add"));
    run.ran("S2", "H2", "bb", &[(1, Verdict::Grew), (3, Verdict::Grew)]);
    // F1's unit calls the writer itself; F3's does not name any writer.
    let dir = run.dir.path().join("harness/H2");
    std::fs::create_dir_all(dir.join("driver")).unwrap();
    std::fs::write(
        dir.join("driver/zz_driver_test.go"),
        "func unit(i int) {\n\tbus.add(name(i))\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("targets.json"),
        json!({"harness": "H2", "targets": [
            {"finding": "F1", "writer_functions": ["Bus.add"]},
            {"finding": "F3", "writer_functions": ["Q.push"]},
        ]})
        .to_string(),
    )
    .unwrap();
    let md = run.report();
    assert!(
        md.contains(
            "- Trigger: grown only by calling the registration function itself (Bus.add) in H2"
        ),
        "{md}"
    );
    assert!(
        findings_line(&md).contains("grown only by calling the registration function itself 1"),
        "{}",
        findings_line(&md)
    );
    assert!(
        findings_line(&md).contains("reproduced through the public API 1"),
        "{}",
        findings_line(&md)
    );
}

#[test]
fn a_confirmed_finding_the_verifier_reads_as_not_a_leak_is_counted_apart_and_listed_last() {
    let run = Run::new();
    let ts = r#""schema_version":2,"ts":"2026-01-01T00:00:00Z""#;
    let mut text = ledger_text();
    let n = text.lines().count();
    let ver = |seq: usize, f: &str, site: &str, label: &str| {
        format!(
            "{{{ts},\"seq\":{seq},\"type\":\"verification\",\"finding\":\"{f}\",\"site\":\"{site}\",\"holder\":\"static/global\",\"mechanism\":\"m\",\"label\":\"{label}\",\"bundle\":\"{}\",\"agrees_site\":true,\"agrees_holder\":true,\"agrees_label\":false}}\n",
            "a".repeat(64)
        )
    };
    // F1 (listed first by default) is disputed: the verifier reads N-INTENT.
    text.push_str(&ver(n + 1, "F1", "src/bus.go:12", "N-INTENT"));
    let st = RunState::load(run.dir.path()).unwrap();
    let md = render_markdown(
        &RunView::new(st, Ledger::from_text(&text).unwrap(), None).with_disk_facts(run.dir.path()),
    );
    let line = findings_line(&md);
    assert!(
        line.contains("confirmed but disputed by the verifier (reads N-INTENT) 1"),
        "{line}"
    );
    assert!(md.contains("DISAGREES"), "{md}");
    let o = order(&md);
    assert!(
        o.iter().position(|h| h.contains("F3")) < o.iter().position(|h| h.contains("F1")),
        "{o:?}"
    );
}

/// Reads the number after `label` in the `Findings:` line.
fn counted(line: &str, label: &str) -> usize {
    let at = line
        .find(label)
        .unwrap_or_else(|| panic!("{label}: {line}"));
    line[at + label.len()..]
        .trim_start()
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("{label}: {line}"))
}

#[test]
fn a_finding_disputed_by_the_verifier_and_released_is_counted_once_and_the_plain_one_is_not_hidden()
{
    let run = Run::new();
    let ts = r#""schema_version":2,"ts":"2026-01-01T00:00:00Z""#;
    let mut text = ledger_text();
    let mut seq = text.lines().count();
    let mut push = |body: String| {
        seq += 1;
        text.push_str(&format!("{{{ts},\"seq\":{seq},{body}}}\n"));
    };
    // F1 is released: its family holds a confirmed N-RELEASE sibling (the flat driver with the
    // release called) ...
    push(r#""type":"evidence","evidence_id":"E9","source":{"repo":{"commit":"abc123"}},"tool":{"name":"dla","version":"0.1.0"},"authority":"test","kind":"discriminator","result":{"test":"plateau+purge","favors":"non-leak","supports":["non-leak"],"finding":"F1","harness":"H1","series":"S1"}"#.to_owned());
    push(r#""type":"finding_added","finding":{"id":"F1.n2","state":"candidate","origin":"hypothesis","supports":["E6"],"parent":"F1","hypothesis":"N-RELEASE","refute_if":"the holder still grows with the release called","remover":"src/bus.go:30"}"#.to_owned());
    push(r#""type":"transition","finding":"F1.n2","from":"candidate","to":"under_test","evidence":["E2"],"by":"dla ledger test""#.to_owned());
    push(r#""type":"transition","finding":"F1.n2","from":"under_test","to":"confirmed","evidence":["E9"],"by":"dla ledger confirm""#.to_owned());
    // ... and the blind verifier reads it as N-INTENT.
    push(format!(
        "\"type\":\"verification\",\"finding\":\"F1\",\"site\":\"src/bus.go:12\",\"holder\":\"registration-table\",\"mechanism\":\"m\",\"label\":\"N-INTENT\",\"bundle\":\"{}\",\"agrees_site\":true,\"agrees_holder\":true,\"agrees_label\":false",
        "a".repeat(64)
    ));
    // F3 is a plain confirmed leak.
    let st = RunState::load(run.dir.path()).unwrap();
    let md = render_markdown(
        &RunView::new(st, Ledger::from_text(&text).unwrap(), None).with_disk_facts(run.dir.path()),
    );
    let line = findings_line(&md);
    // Two confirmed leaks, each in one bucket: F3 plain, F1 disputed (the verifier's word comes
    // first); nothing is counted twice and nothing is hidden.
    assert!(
        line.contains("leak hypotheses: confirmed 1, confirmed but disputed by the verifier (reads N-INTENT) 1, strong 0"),
        "{line}"
    );
    assert!(
        !line.contains("grows only while callers skip the release"),
        "{line}"
    );
    assert!(!line.contains("no leak confirmed"), "{line}");
    assert_eq!(
        counted(line, "leak hypotheses: confirmed")
            + counted(
                line,
                "confirmed but disputed by the verifier (reads N-INTENT)"
            ),
        2,
        "{line}"
    );
    assert!(
        line.contains("non-leak explanations: confirmed 1"),
        "{line}"
    );
    // F1's own section still says it grows only while the release is skipped.
    assert!(
        md.contains("Grows only while callers skip the release"),
        "{md}"
    );
}

#[test]
fn a_target_that_grew_in_a_capture_series_has_a_trigger_that_is_not_checked_for_want_of_an_entry_harness()
 {
    let run = Run::new();
    run.harness("H1", false, &["F1", "F3"], None);
    run.captured("S1", "H1", &[(1, Verdict::Grew)]);
    let md = run.report();
    assert!(
        md.contains("- Trigger: not checked (no entry-path harness was run for it)"),
        "{md}"
    );
    // The capture series is no run of the harness: F3 has no verdict in it and still reads as
    // having no grown series.
    let f3 = md.split("### F3").nth(1).unwrap();
    assert!(
        f3.contains("no harness series that grew is tied to it"),
        "{f3}"
    );
    let f1 = md
        .split("### F1")
        .nth(1)
        .unwrap()
        .split("### F3")
        .next()
        .unwrap();
    assert!(
        !f1.contains("no harness series that grew is tied to it"),
        "{f1}"
    );
}
