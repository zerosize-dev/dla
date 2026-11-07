//! Golden and behavioural tests for the report renderer.
//!
//! Goldens live in `tests/golden/`. Set `UPDATE_GOLDEN=1` to rewrite them, then
//! review the diff by hand.

use dla_core::{
    CoverageManifest, Ledger, RunView,
    state::{RunState, RunStatus, now_utc},
};
use dla_report::render_markdown;
use serde_json::json;

fn state(run_id: &str) -> RunState {
    let mut s = RunState::init("/work/project", Some("s1"), 3600, None, now_utc()).unwrap();
    s.run_id = run_id.to_owned();
    s
}

fn view(fixture: &str, coverage: Option<CoverageManifest>) -> RunView {
    let text = std::fs::read_to_string(format!(
        "{}/tests/fixtures/{fixture}.jsonl",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    RunView::new(
        state("20260101-000000-abcd"),
        Ledger::from_text(&text).unwrap(),
        coverage,
    )
}

fn coverage() -> CoverageManifest {
    serde_json::from_value(json!({
        "schema_version": 1,
        "workers": {
            "jvm": {
                "runtime": "jvm",
                "backends": {
                    "mat": {"status": "missing", "reason": "no JDK on PATH",
                            "install_hint": "install a JDK 17 or newer",
                            "enumerated": 0, "dispositioned": 0},
                    "jcmd": {"status": "ran", "version": "17", "enumerated": 5, "dispositioned": 5},
                    "jfr": {"status": "blocked", "reason": "privilege declined",
                            "enumerated": 0, "dispositioned": 0}
                },
                "entry_points": {
                    "discovered": ["Main", "Worker"],
                    "exercised": ["Main"],
                    "not_exercisable": [{"entry": "Worker|batch", "reason": "needs a queue"}]
                }
            }
        },
        "residual": [
            {"region": "heap", "total_slope": 10.0, "attributed_slope": 9.5,
             "residual": 0.5, "unit": "MB/h", "threshold": 1.0},
            {"region": "off-heap", "total_slope": 8.0, "attributed_slope": 2.0,
             "residual": 6.0, "unit": "MB/h", "threshold": 1.0, "open_item": "O1"}
        ],
        "blind_spots": [{"runtime": "jvm", "text": "native memory outside NMT is not visible"}]
    }))
    .unwrap()
}

fn check_golden(name: &str, actual: &str) {
    let path = format!("{}/tests/golden/{name}.md", env!("CARGO_MANIFEST_DIR"));
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(actual, expected, "golden {name} differs");
}

#[test]
fn golden_complete_with_refuted_and_unresolved() {
    let md = render_markdown(&view("complete", None));
    check_golden("complete", &md);
}

#[test]
fn golden_incomplete_with_open_items_and_coverage() {
    let md = render_markdown(&view("incomplete", Some(coverage())));
    check_golden("incomplete", &md);
}

#[test]
fn complete_header_and_sections_in_order() {
    let md = render_markdown(&view("complete", None));
    assert!(md.starts_with("# DLA report\n\nRun: `20260101-000000-abcd`\n\nStatus: COMPLETE\n"));
    assert!(!md.contains("## Open items"));
    assert!(md.contains("Residual: not measured"));
    let order = [
        "## Findings",
        "## Ruled out",
        "## Blind spots and unknowns",
        "## Checklist",
        "## Evidence index",
    ];
    let pos: Vec<usize> = order.iter().map(|h| md.find(h).unwrap()).collect();
    assert!(pos.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn novel_mechanism_label_is_only_for_confirmed_and_strong() {
    let md = render_markdown(&view("complete", None));
    // F1 is confirmed; F3 is unresolved; F2 is refuted.
    assert!(md.contains("### F1: **Novel mechanism (UNCLASSIFIED)**\n"));
    assert!(md.contains("### F3: **UNCLASSIFIED**\n"));
    assert!(md.contains("- **F2** (UNCLASSIFIED):"));
    assert_eq!(md.matches("Novel mechanism").count(), 1);
    let md = render_markdown(&view("incomplete", None));
    assert!(!md.contains("Novel mechanism"));
    assert!(!md.contains("?."), "no doubled punctuation");
}

#[test]
fn findings_are_grouped_confirmed_then_suspected_unresolved_undispositioned() {
    let md = render_markdown(&view("incomplete", None));
    // F1 is a candidate, F2 under_test; both undispositioned, in id order.
    let f1 = md.find("### F1:").unwrap();
    let f2 = md.find("### F2:").unwrap();
    assert!(f1 < f2);
    let md = render_markdown(&view("complete", None));
    let f1 = md.find("### F1:").unwrap();
    let f3 = md.find("### F3:").unwrap();
    assert!(f1 < f3, "confirmed before unresolved");
    assert!(
        !md.contains("### F2:"),
        "refuted findings are not listed as findings"
    );
}

#[test]
fn refuted_lists_discriminator_evidence() {
    let md = render_markdown(&view("complete", None));
    let ruled = md.split("## Ruled out").nth(1).unwrap();
    let ruled = ruled.split("## Blind spots").next().unwrap();
    assert!(ruled.contains("**F2**"));
    assert!(ruled.contains("discriminator E3"));
    assert!(ruled.contains("growth is flat after warmup"));
}

#[test]
fn table_cells_escape_pipes_and_newlines() {
    let md = render_markdown(&view("complete", None));
    assert!(md.contains("artifact src/a\\|b.py (0123456789ab)"));
    let md = render_markdown(&view("incomplete", None));
    // Free text in lists stays on one line.
    assert!(md.contains("Residual heap growth unexplained | pipe"));
}

#[test]
fn incomplete_lists_open_items_with_fix_commands() {
    let md = render_markdown(&view("incomplete", None));
    assert!(md.contains("Status: INCOMPLETE ("));
    let open = md.split("## Open items").nth(1).unwrap();
    let open = open.split("\n## ").next().unwrap();
    assert!(open.contains("**F1**"));
    assert!(open.contains("**F2**"));
    assert!(open.contains("**O1**"));
    assert!(
        !open.contains("**O2**"),
        "resolved open item does not block"
    );
    assert!(open.contains("**intake.budget**"));
    assert!(open.contains("`dla ledger resolve F1 --as unresolved"));
}

#[test]
fn checklist_marks_missing_items() {
    let md = render_markdown(&view("incomplete", None));
    assert!(md.contains("| `intake.budget` |"));
    let row = md.lines().find(|l| l.contains("`intake.budget`")).unwrap();
    assert!(row.contains("**MISSING**"));
    assert!(row.contains("\u{2014}"));
    let row = md
        .lines()
        .find(|l| l.contains("`intake.workload`"))
        .unwrap();
    assert!(row.contains("unresolved"));
    assert!(row.contains("no_harness: ask the owner for a load script"));
}

#[test]
fn coverage_gaps_blind_spots_and_residual() {
    let md = render_markdown(&view("incomplete", Some(coverage())));
    assert!(
        md.contains("- jvm/mat: missing (no JDK on PATH). Install hint: install a JDK 17 or newer")
    );
    assert!(md.contains("- jvm/jfr: blocked (privilege declined)"));
    assert!(!md.contains("jvm/jcmd"), "backends that ran are not gaps");
    assert!(md.contains("- jvm: native memory outside NMT is not visible"));
    assert!(md.contains("needs a queue"));
    assert!(
        md.contains(
            "off-heap 6 MB/h unattributed of 8 total (threshold 1) ABOVE THRESHOLD, see O1"
        )
    );
    assert!(!md.contains("Residual: not measured"));
}

#[test]
fn evidence_limitations_and_unknowns_carry_ids() {
    let md = render_markdown(&view("complete", None));
    assert!(md.contains("- E2: partial coverage (40 of 100 examined)"));
    assert!(md.contains("- E2: limitation: sampled every 5s | misses short bursts"));
    assert!(md.contains("- E2: unknown: Is the cache bounded? Resolve by: read CacheConfig"));
}

#[test]
fn evidence_index_has_raw_pointer_and_source() {
    let md = render_markdown(&view("complete", None));
    assert!(md.contains(
        "| E2 | growth-series | live | pyprobe 2.1 | process 4242 @ 2026-01-01T00:00:00Z | raw/growth.json |"
    ));
}

#[test]
fn empty_ledger_renders_every_section() {
    let v = RunView::new(state("r1"), Ledger::from_text("").unwrap(), None);
    let md = render_markdown(&v);
    assert!(md.contains("Status: INCOMPLETE (8 open)"));
    assert!(md.contains("No findings to report."));
    assert!(md.contains("No evidence recorded."));
    assert!(md.contains("No coverage manifest recorded."));
    assert!(md.ends_with('\n'));
}

#[test]
fn aborted_run_is_incomplete() {
    let mut v = view("complete", None);
    v.state.status = RunStatus::Aborted;
    v.state.incomplete_reason = Some("user stopped".into());
    let md = render_markdown(&v);
    assert!(md.contains("Status: INCOMPLETE (1 open)"));
    assert!(md.contains("run is aborted: user stopped"));
}

#[test]
fn output_is_deterministic() {
    let a = render_markdown(&view("incomplete", Some(coverage())));
    let b = render_markdown(&view("incomplete", Some(coverage())));
    assert_eq!(a, b);
}

#[test]
fn annotations_list_writers_and_removers_under_the_finding() {
    let md = render_markdown(&view("complete", None));
    let f1 = md.split("### F1:").nth(1).unwrap();
    let f1 = f1.split("\n### ").next().unwrap();
    assert!(f1.contains("- Writers: `src/cache.py:42`, `src/loader.py:17`\n"));
    assert!(f1.contains("- Removers: `src/cache.py:90`\n"));
    assert!(f1.contains("- Annotation: nothing evicts on the request path\n"));
    let f3 = md.split("### F3:").nth(1).unwrap();
    let f3 = f3.split("\n## ").next().unwrap();
    assert!(!f3.contains("Writers:") && !f3.contains("Removers:"));
}
