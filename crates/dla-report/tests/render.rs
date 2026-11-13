//! Golden and behavioural tests for the report renderer.
//!
//! Goldens live in `tests/golden/`. Set `UPDATE_GOLDEN=1` to rewrite them, then
//! review the diff by hand.

use dla_core::{
    AnswerStatus, ChecklistAnswer, CoverageManifest, Ledger, RunView, UnresolvedReason,
    state::{Consent, ConsentGiver, EntryPath, RunState, RunStatus, now_utc},
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

/// A symptom run with no findings, every checklist item dispositioned and consent recorded:
/// the final gate has nothing to list.
fn gate_complete_view() -> RunView {
    let mut ledger = Ledger::from_text("").unwrap();
    for e in ledger.checklist() {
        ledger
            .answer_checklist(ChecklistAnswer {
                item: e.item.id,
                status: AnswerStatus::Unresolved,
                evidence: vec![],
                note: None,
                reason: Some(UnresolvedReason::ToolMissing),
                remedy: Some("no target".into()),
            })
            .unwrap();
    }
    let mut st = state("20260101-000000-abcd");
    st.entry = EntryPath::Symptom;
    st.consent = Some(Consent {
        tiers: vec![1],
        raw_values: false,
        given_by: ConsentGiver::User,
        recorded_at: now_utc(),
    });
    RunView::new(st, ledger, None)
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

fn dynamic_coverage() -> CoverageManifest {
    serde_json::from_value(json!({
        "schema_version": 1,
        "workers": {
            "go": {
                "runtime": "go",
                "entry_points": {
                    "discovered": ["cmd/bus.Run", "cmd/bus.Serve", "cmd/cache.Warm"],
                    "exercised": ["cmd/bus.Run", "cmd/cache.Warm"]
                },
                "iterations": 8000
            },
            "python": {"runtime": "python"}
        }
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
fn golden_dynamic_confirmed_and_ruled_out() {
    let md = render_markdown(&view("dynamic", Some(dynamic_coverage())));
    check_golden("dynamic", &md);
}

#[test]
fn confirmed_findings_show_their_growth_column() {
    let md = render_markdown(&view("dynamic", None));
    // The tied column (F1) is chosen, not the first one (os.rss).
    assert!(
        md.contains("- Growth: `probe:Registry.listeners` 412 bytes/n (steady, gate pass, E3)\n"),
        "{md}"
    );
    assert_eq!(md.matches("- Growth:").count(), 1);
    // Findings that are not confirmed never get a growth line.
    let md = render_markdown(&view("complete", None));
    assert!(!md.contains("- Growth:"));
}

#[test]
fn refuted_findings_name_the_discriminator_verdict() {
    let md = render_markdown(&view("dynamic", None));
    let ruled = md.split("## Ruled out").nth(1).unwrap();
    let ruled = ruled.split("## Blind spots").next().unwrap();
    assert!(
        ruled.contains("  - Ruled out by: plateau+purge (leak) E4\n"),
        "{ruled}"
    );
    assert!(
        ruled.contains("  - Ruled out by: plateau (non-leak) E5\n"),
        "{ruled}"
    );
    // A discriminator with no test or verdict in its result is still named.
    let md = render_markdown(&view("complete", None));
    assert!(md.contains("  - Ruled out by: discriminator E3\n"), "{md}");
}

#[test]
fn coverage_line_reports_iterations_and_exercised_entry_points() {
    let md = render_markdown(&view("dynamic", Some(dynamic_coverage())));
    assert!(
        md.contains("Coverage: go 8000 iterations, 2 of 3 entry points exercised\n"),
        "{md}"
    );
    let md = render_markdown(&view("dynamic", None));
    assert!(md.contains("Coverage: not measured\n"));
    // The jvm worker of the other fixture has exercised entry points but no iterations.
    let md = render_markdown(&view("incomplete", Some(coverage())));
    assert!(
        md.contains("Coverage: jvm no iterations, 1 of 2 entry points exercised\n"),
        "{md}"
    );
}

#[test]
fn coverage_line_says_how_many_project_classes_loaded_when_no_entry_point_matched() {
    let cov = |exercised: serde_json::Value, project: serde_json::Value| -> CoverageManifest {
        serde_json::from_value(json!({
            "schema_version": 1,
            "workers": {"jvm": {
                "runtime": "jvm",
                "entry_points": {"discovered": ["a/B.kt::B.f", "a/C.kt::C.g"], "exercised": exercised},
                "iterations": 750,
                "project_classes": project
            }}
        }))
        .unwrap()
    };
    let md = render_markdown(&view("dynamic", Some(cov(json!([]), json!(37)))));
    assert!(
        md.contains(
            "Coverage: jvm 750 iterations, 0 of 2 entry points exercised, 37 project classes loaded\n"
        ),
        "{md}"
    );
    // One class reads in the singular.
    let md = render_markdown(&view("dynamic", Some(cov(json!([]), json!(1)))));
    assert!(
        md.contains("0 of 2 entry points exercised, 1 project class loaded\n"),
        "{md}"
    );
    // Nothing loaded, or an entry point matched: the line is the plain one.
    for (exercised, project) in [
        (json!([]), json!(0)),
        (json!([]), json!(null)),
        (json!(["a/B.kt::B.f"]), json!(37)),
    ] {
        let md = render_markdown(&view("dynamic", Some(cov(exercised.clone(), project))));
        assert!(!md.contains("project class"), "{exercised} {md}");
    }
}

#[test]
fn complete_header_and_sections_in_order() {
    let md = render_markdown(&gate_complete_view());
    assert!(md.starts_with("# DLA report\n\nRun: `20260101-000000-abcd`\n\nStatus: COMPLETE\n"));
    assert!(!md.contains("## Open items"));
    let md = render_markdown(&view("complete", None));
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
fn unclassified_is_rendered_plainly_at_the_same_weight_in_every_state() {
    let md = render_markdown(&view("complete", None));
    // F1 is confirmed; F3 is unresolved; F2 is refuted.
    assert!(md.contains("### F1: **UNCLASSIFIED**\n"));
    assert!(md.contains("### F3: **UNCLASSIFIED**\n"));
    assert!(md.contains("- **F2** (UNCLASSIFIED):"));
    assert!(!md.contains("Novel mechanism"));
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
    assert!(md.contains("Status: INCOMPLETE ("));
    assert!(md.contains("no consent is recorded"));
    assert!(md.contains("No findings to report."));
    assert!(md.contains("No evidence recorded."));
    assert!(md.contains("No coverage manifest recorded."));
    assert!(md.ends_with('\n'));
}

#[test]
fn status_follows_the_final_gate() {
    use dla_core::gate::{GateTarget, missing};
    // Everything dispositioned, but a residual over its threshold has no resolved open item:
    // `gate check --final` and `run status` list it, so the report must not say COMPLETE.
    let mut v = gate_complete_view();
    v.coverage = Some(
        serde_json::from_value(json!({
            "schema_version": 1,
            "workers": {},
            "residual": [{"region": "heap", "total_slope": 10.0, "attributed_slope": 2.0,
                "residual": 8.0, "unit": "MB/h", "threshold": 1.0}]
        }))
        .unwrap(),
    );
    let gate = missing(&v, GateTarget::Final, None);
    assert_eq!(gate.len(), 1, "{gate:?}");
    let md = render_markdown(&v);
    assert!(md.contains("Status: INCOMPLETE (1 open)"), "{md}");
    assert!(md.contains("residual growth in heap"), "{md}");
    // The open list is the gate's list.
    for m in &gate {
        let fix = m.fix.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(md.contains(&fix), "{md}");
    }
}

#[test]
fn aborted_run_is_incomplete() {
    let mut v = view("complete", None);
    v.state.status = RunStatus::Aborted;
    v.state.incomplete_reason = Some("user stopped".into());
    let md = render_markdown(&v);
    assert!(md.contains("Status: INCOMPLETE ("));
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

/// Builds a view from fixture text, renumbering `seq` so edited fixtures stay gap-free.
fn view_of_text(text: &str) -> RunView {
    let text: String = text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            let mut v: serde_json::Value = serde_json::from_str(l).unwrap();
            v["seq"] = json!(i + 1);
            format!("{v}\n")
        })
        .collect();
    let text = text.as_str();
    RunView::new(
        state("20260101-000000-abcd"),
        Ledger::from_text(text).unwrap(),
        None,
    )
}

fn dynamic_text() -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/dynamic.jsonl",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn an_untied_passing_column_is_not_shown_as_the_findings_growth() {
    let text = dynamic_text().replace(r#""site":"src/bus.go:12","finding":"F1","#, "");
    let md = render_markdown(&view_of_text(&text));
    assert!(
        md.contains(
            "- Growth: not tied to this finding (passing columns of E3: `os.rss`, `probe:Registry.listeners`)\n"
        ),
        "{md}"
    );
    assert!(!md.contains("bytes/n (steady"), "{md}");
}

#[test]
fn a_confirmed_non_leak_sibling_is_explained_not_novel_and_has_no_growth_line() {
    // The leak root F1 is refuted by E5 (non-leak, bound to F1) and its N-WARM sibling is
    // confirmed by the same discriminator.
    let text = dynamic_text().replace(
        r#""favors":"non-leak","supports":["N-WARM","N-HWM"],"finding":"F2""#,
        r#""favors":"non-leak","supports":["N-WARM","N-HWM"],"finding":"F1""#,
    );
    let mut lines: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let tr = |finding: &str, from: &str, to: &str, ev: &str| {
        json!({"schema_version":2,"seq":0,"ts":"2026-01-01T00:00:00Z","type":"transition",
            "finding":finding,"from":from,"to":to,"evidence":[ev],"by":"test"})
    };
    lines.retain(|v| {
        v["type"] != "transition"
            || !["F1", "F1.n1", "F2"].contains(&v["finding"].as_str().unwrap())
    });
    let at = lines
        .iter()
        .position(|v| v["type"] == "checklist_answer")
        .unwrap();
    let new = [
        tr("F1", "candidate", "under_test", "E2"),
        tr("F1.n1", "candidate", "under_test", "E2"),
        tr("F1", "under_test", "refuted", "E5"),
        tr("F1.n1", "under_test", "confirmed", "E5"),
    ];
    lines.splice(at..at, new);
    let text: String = lines.iter().map(|v| format!("{v}\n")).collect();
    let md = render_markdown(&view_of_text(&text));
    let section = md
        .split("### F1.n1")
        .nth(1)
        .and_then(|s| s.split("\n### ").next())
        .unwrap_or_default();
    assert!(!section.is_empty(), "{md}");
    assert!(
        section.contains("- Explained by N-WARM (discriminator E5)\n"),
        "{section}"
    );
    assert!(!section.contains("Novel mechanism"), "{section}");
    assert!(!section.contains("- Verified:"), "{section}");
    assert!(!section.contains("- Growth:"), "{section}");
}

/// A merged two-worker run: `jvm` finished, `rust` was killed with one unmerged slice
/// line; a duplicate was folded into `jvm/F1`, one finding was verified and agreed, one
/// did not. The state is a module scope with excluded paths and out-of-scope counts.
fn workers_view() -> RunView {
    use dla_core::state::{EntryPath, Scope, ScopeKind, WorkerEntry, WorkerStatus};
    let mut st = state("20260101-000000-abcd");
    st.status = RunStatus::Incomplete;
    st.incomplete_reason = Some("workers did not finish: rust (slice ends at line 3)".to_owned());
    st.scope = Some(Scope {
        kind: ScopeKind::Module,
        path: "services/api".to_owned(),
        excluded: vec!["services/api/generated".to_owned(), "vendor".to_owned()],
    });
    let entry = |name: &str, status| WorkerEntry {
        name: name.to_owned(),
        runtime: name.to_owned(),
        share_secs: 600,
        deadline: now_utc(),
        started_at: None,
        status,
        slice: format!("ledger.{name}.jsonl"),
        stalled: false,
    };
    st.workers = vec![
        entry("jvm", WorkerStatus::Done),
        entry("rust", WorkerStatus::Killed),
    ];
    st.entry = EntryPath::Repository;
    let text = std::fs::read_to_string(format!(
        "{}/tests/fixtures/workers.jsonl",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let cov: CoverageManifest = serde_json::from_value(json!({
        "schema_version": 1,
        "workers": {
            "jvm": {"runtime": "jvm", "out_of_scope": 12},
            "rust": {"runtime": "rust", "out_of_scope": 1}
        },
        "blind_spots": [{"runtime": "go", "text": "no worker covered this runtime"}]
    }))
    .unwrap();
    RunView::new(st, Ledger::from_text(&text).unwrap(), Some(cov)).with_slices(vec![
        dla_core::slice::SliceSummary {
            worker: "jvm".to_owned(),
            lines: 4,
            done: true,
            torn_tail: false,
        },
        dla_core::slice::SliceSummary {
            worker: "rust".to_owned(),
            lines: 4,
            done: false,
            torn_tail: true,
        },
    ])
}

#[test]
fn golden_workers_merge_verification_and_scope() {
    let md = render_markdown(&workers_view());
    check_golden("workers", &md);
}

#[test]
fn workers_section_lists_status_lines_done_and_torn_tail() {
    let md = render_markdown(&workers_view());
    assert!(
        md.contains("- **jvm** (jvm): status done; 4 lines merged; slice done; torn tail: no\n"),
        "{md}"
    );
    assert!(
        md.contains("- **rust** (rust): status killed; 3 of 4 lines merged; slice not done; torn tail: yes, final line dropped\n"),
        "{md}"
    );
    // Only the slice with lines the main ledger lacks is listed as unmerged.
    let unmerged = md.split("### Unmerged slices").nth(1).unwrap();
    let unmerged = unmerged.split("\n## ").next().unwrap();
    assert!(
        unmerged.contains("**rust**: 4 lines in ledger.rust.jsonl, 3 merged"),
        "{unmerged}"
    );
    assert!(!unmerged.contains("jvm"), "{unmerged}");
    assert!(md.contains("Scope: module `services/api`; excluded: `services/api/generated`, `vendor`; out of scope: jvm 12 files, rust 1 file\n"), "{md}");
}

#[test]
fn scope_line_names_the_build_logic_that_was_not_read() {
    let mut st = state("20260101-000000-abcd");
    st.entry = dla_core::state::EntryPath::Repository;
    let cov: CoverageManifest = serde_json::from_value(json!({
        "schema_version": 1,
        "workers": {"jvm": {"runtime": "jvm", "build_logic": 3}}
    }))
    .unwrap();
    let md = render_markdown(&RunView::new(st, Ledger::from_text("").unwrap(), Some(cov)));
    assert!(
        md.contains(
            "Scope: repository; excluded: none; out of scope: none; build logic not read: jvm 3 files\n"
        ),
        "{md}"
    );
}

#[test]
fn a_merged_duplicate_is_shown_under_its_canonical_finding_only() {
    let md = render_markdown(&workers_view());
    assert!(md.contains("- Merged duplicates: rust/F1\n"), "{md}");
    assert!(!md.contains("### rust/F1"), "{md}");
    assert!(
        md.contains("- **rust/F1** merged into **jvm/F1** (a duplicate of the same leak; its evidence E1 is kept on jvm/F1)"),
        "{md}"
    );
    // A `dla ledger duplicate` merge is reported the same way, so the reason is not
    // claimed to be a shared site and holder.
    assert!(!md.contains("same site and holder"), "{md}");
    // 3 findings are enumerated, not 4.
    assert!(
        md.contains("Findings: 3 enumerated, 0 dispositioned"),
        "{md}"
    );
}

#[test]
fn verified_line_agrees_or_names_both_values() {
    let md = render_markdown(&workers_view());
    assert!(
        md.contains("- Verified: agrees (verifier site `src/Registry.java:44`, holder static/global, label UNCLASSIFIED; bundle aaaaaaaaaaaa)\n"),
        "{md}"
    );
    assert!(
        md.contains("- Verified: DISAGREES (site: finding `native/pool.rs:7` vs verifier none; holder: finding native-handle/global-ref vs verifier thread-local; bundle aaaaaaaaaaaa); finding mechanism: not stated; verifier mechanism: a thread local\n"),
        "{md}"
    );
}

#[test]
fn entry_line_names_the_artifacts_of_an_artifact_run() {
    use dla_core::state::EntryPath;
    let mut st = state("20260101-000000-abcd");
    st.entry = EntryPath::Artifact;
    st.artifacts = vec!["dumps/a.hprof.gz".to_owned(), "dumps/b.hprof.gz".to_owned()];
    let md = render_markdown(&RunView::new(st, Ledger::from_text("").unwrap(), None));
    assert!(
        md.contains("Entry: artifact (`dumps/a.hprof.gz`, `dumps/b.hprof.gz`)\n"),
        "{md}"
    );
    assert!(md.contains("Scope: repository; excluded: none; out of scope: none\n"));
    assert!(!md.contains("## Workers"), "no workers, no section");
}

// ------------------------------------------------------------------ schema 6 lines

mod v6 {
    use super::*;
    use dla_core::{
        Authority, EvidenceKind, FindingState, NewEvidence, NewFinding, Origin, Source, Tool,
        archetypes::Table,
        evidence::{EvidenceCoverage, Signal},
        finding::{ExplanationTag, Holder},
        labels::Archetypes,
        record::RemovalClaim,
    };

    const TABLE: &str = r#"
[L-COLLECT]
holders = ["static/global"]
regions = ["heap"]
requires = [["growth-series", "retention-path"]]
disconfirm = ["N-WARM"]
repair = ["bound/evict"]
summary = "A collection gains entries and nothing removes them."
"#;

    fn ev(
        kind: EvidenceKind,
        authority: Authority,
        tool: &str,
        class: &str,
        src: &str,
        mut result: serde_json::Value,
    ) -> NewEvidence {
        result["signal"] = Signal::new(tool, class, src).to_value();
        NewEvidence {
            source: Source::Repo {
                commit: "abc".into(),
                path: None,
            },
            tool: Tool {
                name: tool.into(),
                version: "0".into(),
            },
            authority,
            kind,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        }
    }

    /// F1 confirmed, labelled L-COLLECT, explained, then strong on a vgdb retention record.
    pub fn ledger() -> Ledger {
        let mut l = Ledger::from_text("").unwrap();
        let id = |s: &str| -> dla_core::FindingId { s.parse().unwrap() };
        let e = |s: &str| -> dla_core::EvidenceId { s.parse().unwrap() };
        for n in [
            ev(
                EvidenceKind::StaticHit,
                Authority::Static,
                "t",
                "x",
                "s1",
                json!({}),
            ),
            ev(
                EvidenceKind::Workload,
                Authority::Test,
                "t",
                "x",
                "s2",
                json!({"harness": "H1", "mode": "command", "runtime": "go"}),
            ),
            ev(
                EvidenceKind::Discriminator,
                Authority::Test,
                "dla",
                "x",
                "s3",
                json!({"test": "plateau", "favors": "leak", "supports": ["leak"], "finding": null}),
            ),
            ev(
                EvidenceKind::GrowthSeries,
                Authority::Test,
                "dla-trend",
                "harness-series",
                "aa",
                json!({"trend": "T1", "series": "S1", "n": 8, "gate": "pass", "failed": 0,
                "passed": [{"column": "probe:CACHE", "region": "heap", "site": "a.go:1", "finding": "F1", "shape": "steady"}]}),
            ),
            ev(
                EvidenceKind::RetentionPath,
                Authority::Live,
                "valgrind-vgdb",
                "leak-checker",
                "bb",
                json!({"root": "static table", "path": [], "alloc": "grow (a.go:2)"}),
            ),
        ] {
            l.add_evidence(n).unwrap();
        }
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![e("E1")]);
        nf.site = Some("a.go:1".into());
        nf.holder = Some(Holder::StaticGlobal);
        l.add_finding(nf).unwrap();
        let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![e("E1")]);
        nf.sibling_of = Some(id("F1"));
        nf.hypothesis = Some("N-WARM".into());
        nf.refute_if = Some("growth continues".into());
        l.add_finding(nf).unwrap();
        let mv = |l: &mut Ledger, f: &str, to, ev: &[&str]| {
            l.transition(
                &id(f),
                to,
                None,
                None,
                ev.iter().map(|x| e(x)).collect(),
                "t",
            )
            .unwrap();
        };
        mv(&mut l, "F1.n1", FindingState::Refuted, &["E3"]);
        mv(&mut l, "F1", FindingState::UnderTest, &["E2"]);
        mv(&mut l, "F1", FindingState::Confirmed, &["E4"]);
        let t = Table::parse(TABLE, "t").unwrap();
        l.label(
            &id("F1"),
            "L-COLLECT",
            &[e("E4"), e("E5")],
            None,
            Archetypes::On(&t),
        )
        .unwrap();
        l.annotate(&id("F1"), vec!["a.go:9".into()], vec![], None)
            .unwrap();
        l.explain(
            &id("F1"),
            ExplanationTag::Observed,
            &[],
            RemovalClaim::Never,
            None,
            Some(&e("E5")),
            "Requests add entries to the static/global table at a.go:1 through put at a.go:9 and no code removes them.",
            &[],
            &[],
        )
        .unwrap();
        mv(&mut l, "F1", FindingState::Strong, &["E5"]);
        l
    }

    fn md(l: Ledger) -> String {
        render_markdown(&RunView::new(state("20260101-000000-abcd"), l, None))
    }

    #[test]
    fn a_labelled_explained_strong_finding_shows_its_proof() {
        let md = md(ledger());
        assert!(
            md.contains("### F1: **L-COLLECT**\n\n- State: strong\n- Origin: `census`\n- Label: L-COLLECT (evidence: E4 growth-series, E5 retention-path)\n"),
            "{md}"
        );
        assert!(
            md.contains("- Strong: second class E5 retention-path that names the holder, independent of the growth record E4 (dla-trend)\n"),
            "{md}"
        );
        assert!(
            md.contains("- Explanation (observed): Requests add entries"),
            "{md}"
        );
        assert!(
            md.contains("- Explanation structure: adds: a.go:9; removal: never; evidence: E5\n"),
            "{md}"
        );
        assert!(md.contains("- Verified: no\n"), "{md}");
    }

    #[test]
    fn unclassified_with_a_note_shows_the_note_and_a_verified_label_disagreement_both_mechanisms() {
        let t = Table::parse(TABLE, "t").unwrap();
        let mut l = ledger();
        let id: dla_core::FindingId = "F1".parse().unwrap();
        l.verify(
            &id,
            Some("a.go:1".into()),
            Holder::StaticGlobal,
            "the table keeps every entry",
            "UNCLASSIFIED",
            &"a".repeat(64),
            Archetypes::On(&t),
        )
        .unwrap();
        let md = md(l);
        assert!(
            md.contains("- Verified: DISAGREES (label: finding L-COLLECT vs verifier UNCLASSIFIED; bundle aaaaaaaaaaaa); finding mechanism: Requests add entries"),
            "{md}"
        );
        assert!(
            md.contains("verifier mechanism: the table keeps every entry\n"),
            "{md}"
        );
        // And when everything agrees the label is named.
        let mut l = ledger();
        l.verify(
            &id,
            Some("a.go:1".into()),
            Holder::StaticGlobal,
            "m",
            "L-COLLECT",
            &"a".repeat(64),
            Archetypes::On(&t),
        )
        .unwrap();
        let md = super::render_markdown(&RunView::new(state("20260101-000000-abcd"), l, None));
        assert!(md.contains("- Verified: agrees (verifier site `a.go:1`, holder static/global, label L-COLLECT; bundle aaaaaaaaaaaa)\n"), "{md}");
    }

    #[test]
    fn an_explicit_unclassified_label_shows_its_note() {
        let t = Table::parse(TABLE, "t").unwrap();
        let mut l = Ledger::from_text("").unwrap();
        let src = ledger();
        for line in src
            .lines()
            .iter()
            .take_while(|x| !matches!(x.record, dla_core::Record::FindingLabeled(_)))
        {
            l = Ledger::from_text(
                &(l.lines()
                    .iter()
                    .chain(std::iter::once(line))
                    .map(|x| serde_json::to_string(x).unwrap() + "\n")
                    .collect::<String>()),
            )
            .unwrap();
        }
        let id: dla_core::FindingId = "F1".parse().unwrap();
        l.label(
            &id,
            "UNCLASSIFIED",
            &[],
            Some("a pool keyed by thread; no row fits"),
            Archetypes::On(&t),
        )
        .unwrap();
        let md = md(l);
        assert!(md.contains("### F1: **UNCLASSIFIED**\n"), "{md}");
        assert!(
            md.contains("- Label: UNCLASSIFIED (a pool keyed by thread; no row fits)\n"),
            "{md}"
        );
    }
}

// ------------------------------------------------------------------ Measured line, ranking

mod ranking {
    use super::*;

    struct Spec {
        origin: &'static str,
        holder: Option<&'static str>,
        writers: &'static [&'static str],
        removers: &'static [&'static str],
    }

    const REMEDY: &str = "run a harness over it";

    /// A ledger with one static-hit record and one `suspected_static_only(no_harness)`
    /// finding per spec, in the given order (F1, F2, ...). `remedy` is `Some` for all rows.
    fn static_only_text(specs: &[Spec], remedy: impl Fn(usize) -> String) -> String {
        let mut lines = vec![json!({"type": "evidence", "evidence_id": "E1",
            "source": {"repo": {"commit": "abc123"}}, "tool": {"name": "t", "version": "0"},
            "authority": "static", "kind": "census-holder", "result": {}})];
        for (i, s) in specs.iter().enumerate() {
            let n = i + 1;
            let mut f = json!({"id": format!("F{n}"), "state": "candidate", "origin": s.origin,
                "supports": ["E1"], "owner": format!("pkg.Owner{n}.items"),
                "site": format!("src/o{n}.kt:{n}")});
            if let Some(h) = s.holder {
                f["holder"] = json!(h);
            }
            lines.push(json!({"type": "finding_added", "finding": f}));
            if !s.writers.is_empty() || !s.removers.is_empty() {
                lines.push(
                    json!({"type": "finding_annotated", "finding": format!("F{n}"),
                    "writers": s.writers, "removers": s.removers}),
                );
            }
            lines.push(json!({"type": "transition", "finding": format!("F{n}"),
                "from": "candidate", "to": "suspected_static_only", "evidence": [],
                "by": "dla ledger resolve", "reason": "no_harness", "remedy": remedy(n)}));
        }
        lines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let mut l = l.clone();
                l["schema_version"] = json!(1);
                l["seq"] = json!(i + 1);
                l["ts"] = json!("2026-01-01T00:00:00Z");
                format!("{l}\n")
            })
            .collect()
    }

    fn render(text: &str) -> String {
        render_markdown(&view_of_text(text))
    }

    /// The finding IDs of full sections, in order.
    fn section_ids(md: &str) -> Vec<String> {
        md.lines()
            .filter_map(|l| l.strip_prefix("### F"))
            .map(|l| format!("F{}", l.split(':').next().unwrap()))
            .collect()
    }

    fn tiers() -> Vec<Spec> {
        // Tier 5, 4, 3, 2, 1 in ledger order F1..F5.
        vec![
            Spec {
                origin: "census",
                holder: Some("container-field"),
                writers: &[],
                removers: &[],
            },
            Spec {
                origin: "census",
                holder: Some("static/global"),
                writers: &["a.kt:1"],
                removers: &["a.kt:9"],
            },
            Spec {
                origin: "detector:semgrep/leaky",
                holder: None,
                writers: &[],
                removers: &[],
            },
            Spec {
                origin: "census",
                holder: Some("registration-table"),
                writers: &["a.kt:1"],
                removers: &[],
            },
            Spec {
                origin: "known-leak:okhttp-1",
                holder: None,
                writers: &[],
                removers: &[],
            },
        ]
    }

    #[test]
    fn static_only_findings_are_ordered_by_tier_then_ledger_order() {
        let md = render(&static_only_text(&tiers(), |_| REMEDY.into()));
        assert_eq!(section_ids(&md), ["F5", "F4", "F3", "F2", "F1"], "{md}");
        assert!(
            md.contains("Suspected, static-only findings are ordered"),
            "{md}"
        );
        assert!(md.contains("such as a holder with no writers."), "{md}");
        assert!(
            !md.contains("none of them was exercised") && !md.contains("most were not measured"),
            "{md}"
        );
    }

    #[test]
    fn a_tier_keeps_ledger_order_and_the_other_census_shapes_land_in_the_right_tier() {
        let mut specs = tiers();
        // Another tier-4 (thread-local with writers, no removers) after F2, and a tier-2 before F4.
        specs.push(Spec {
            origin: "census",
            holder: Some("thread-local"),
            writers: &["b.kt:2"],
            removers: &[],
        });
        specs.push(Spec {
            origin: "census",
            holder: Some("container-field"),
            writers: &["b.kt:3"],
            removers: &[],
        });
        let md = render(&static_only_text(&specs, |_| REMEDY.into()));
        assert_eq!(
            section_ids(&md),
            ["F5", "F4", "F7", "F3", "F2", "F6", "F1"],
            "{md}"
        );
    }

    #[test]
    fn every_finding_section_prints_its_origin() {
        let md = render(&static_only_text(&tiers(), |_| REMEDY.into()));
        for o in [
            "`known-leak:okhttp-1`",
            "`census`",
            "`detector:semgrep/leaky`",
        ] {
            assert!(md.contains(&format!("- Origin: {o}\n")), "{o}\n{md}");
        }
        let full = render_markdown(&view("complete", None));
        assert_eq!(
            full.matches("\n### F").count(),
            full.matches("\n- Origin: ").count(),
            "{full}"
        );
    }

    fn leak(list: &[String]) -> &'static [&'static str] {
        let v: Vec<&'static str> = list
            .iter()
            .map(|s| &*Box::leak(s.clone().into_boxed_str()))
            .collect();
        Box::leak(v.into_boxed_slice())
    }

    fn many(n: usize) -> Vec<Spec> {
        (0..n)
            .map(|_| Spec {
                origin: "census",
                holder: Some("container-field"),
                writers: &["a.kt:1"],
                removers: &[],
            })
            .collect()
    }

    #[test]
    fn the_first_fifteen_get_sections_and_the_rest_one_table() {
        let md = render(&static_only_text(&many(17), |_| REMEDY.into()));
        assert_eq!(section_ids(&md).len(), 15, "{md}");
        assert!(md.contains("#### Other static-only findings (2)\n"), "{md}");
        assert!(
            md.contains(&format!("Remedy for all of them: {REMEDY}\n")),
            "{md}"
        );
        let table = md
            .split("#### Other static-only findings (2)")
            .nth(1)
            .unwrap();
        let rows: Vec<&str> = table.lines().filter(|l| l.starts_with("| F")).collect();
        assert_eq!(rows.len(), 2, "{table}");
        assert!(
            rows[0].starts_with(
                "| F16 | `census` | container-field | `pkg.Owner16.items` | no_harness |"
            ),
            "{rows:?}"
        );
        assert!(table.contains("| ID | Origin | Holder | Owner or site | Reason |"));
        // Exactly 15 is all sections, no table.
        let md = render(&static_only_text(&many(15), |_| REMEDY.into()));
        assert!(!md.contains("Other static-only findings"), "{md}");
    }

    #[test]
    fn differing_remedies_are_not_printed_once_nor_per_row() {
        let md = render(&static_only_text(&many(17), |n| format!("remedy {n}")));
        let table = md
            .split("#### Other static-only findings (2)")
            .nth(1)
            .unwrap();
        assert!(!table.contains("Remedy for all of them"), "{table}");
        assert!(!table.contains("remedy 16"), "{table}");
    }

    #[test]
    fn other_states_keep_their_full_sections_after_the_table() {
        let mut text = static_only_text(&many(17), |_| REMEDY.into());
        // The golden-style fixture's unresolved F3 and confirmed F1 as extra findings.
        let n = text.lines().count();
        let extra = [
            json!({"type": "finding_added", "finding": {"id": "F18", "state": "candidate",
                "origin": "growth-series", "supports": ["E1"]}}),
            json!({"type": "transition", "finding": "F18", "from": "candidate",
                "to": "unresolved", "evidence": [], "by": "dla ledger resolve",
                "reason": "no_harness", "remedy": "x"}),
        ];
        for (i, mut l) in extra.into_iter().enumerate() {
            l["schema_version"] = json!(1);
            l["seq"] = json!(n + i + 1);
            l["ts"] = json!("2026-01-01T00:00:00Z");
            text.push_str(&format!("{l}\n"));
        }
        let md = render(&text);
        let table_at = md.find("#### Other static-only findings (2)").unwrap();
        let f18 = md.find("### F18: **UNCLASSIFIED**").unwrap();
        assert!(table_at < f18, "{md}");
        assert!(
            md[f18..].contains("- State: unresolved(no_harness)\n"),
            "{md}"
        );
    }

    #[test]
    fn a_candidate_and_its_hypothesis_keep_full_sections_beside_a_cut_table() {
        let mut text = static_only_text(&many(17), |_| REMEDY.into());
        let n = text.lines().count();
        let extra = [
            json!({"type": "finding_added", "finding": {"id": "F18", "state": "candidate",
                "origin": "growth-series", "supports": ["E1"]}}),
            json!({"type": "finding_added", "finding": {"id": "F18.n1", "state": "candidate",
                "origin": "hypothesis", "supports": ["E1"], "parent": "F18",
                "hypothesis": "N-WARM: warm-up growth", "refute_if": "flat after warm-up"}}),
        ];
        for (i, mut l) in extra.into_iter().enumerate() {
            l["schema_version"] = json!(1);
            l["seq"] = json!(n + i + 1);
            l["ts"] = json!("2026-01-01T00:00:00Z");
            text.push_str(&format!("{l}\n"));
        }
        let md = render(&text);
        assert!(md.contains("#### Other static-only findings (2)\n"), "{md}");
        for (id, origin) in [("F18", "growth-series"), ("F18.n1", "hypothesis")] {
            let at = md
                .find(&format!("### {id}: "))
                .unwrap_or_else(|| panic!("{id}\n{md}"));
            assert!(md[at..].contains("- State: candidate\n"), "{id}\n{md}");
            assert!(
                md[at..].contains(&format!("- Origin: `{origin}`\n")),
                "{id}\n{md}"
            );
        }
        assert_eq!(section_ids(&md).len(), 17, "15 static-only plus 2 others");
    }

    #[test]
    fn a_big_blind_spots_list_is_counted_not_listed() {
        let entries: Vec<_> = (0..608)
            .map(|i| {
                json!({"entry": format!("com.example.service.Handler{i}|handle"),
                            "reason": "no_harness"})
            })
            .collect();
        let cov: CoverageManifest = serde_json::from_value(json!({
            "schema_version": 1,
            "workers": {"jvm": {"runtime": "jvm", "entry_points": {
                "discovered": [], "exercised": [], "not_exercisable": entries}}}
        }))
        .unwrap();
        let mut v = view_of_text(&static_only_text(&many(362), |_| REMEDY.into()));
        v.coverage = Some(cov);
        let md = render_markdown(&v);
        assert!(
            md.contains("- jvm: and 593 more not listed (593 x no_harness)"),
            "{md}"
        );
        assert_eq!(md.matches("Handler").count(), 15, "{md}");
        assert!(md.len() < 80 * 1024, "{} bytes", md.len());
    }

    #[test]
    fn a_static_only_section_lists_five_writers_and_counts_the_rest() {
        let writers: Vec<String> = (0..24)
            .map(|i| format!("src/main/kotlin/com/example/service/Writer{i}.kt:{}", i + 1))
            .collect();
        let w: &'static [&'static str] = leak(&writers);
        let specs: Vec<Spec> = (0..3)
            .map(|_| Spec {
                origin: "census",
                holder: Some("container-field"),
                writers: w,
                removers: &[],
            })
            .collect();
        let md = render(&static_only_text(&specs, |_| REMEDY.into()));
        assert_eq!(md.matches("Writer4.kt:5").count(), 3, "{md}");
        assert!(!md.contains("Writer5.kt:6"), "{md}");
        assert_eq!(
            md.matches("and 19 more (in the ledger annotation)").count(),
            3,
            "{md}"
        );
    }

    #[test]
    fn a_kotlin_shaped_run_with_many_writers_renders_under_64_kb() {
        let writers: Vec<String> = (0..24)
            .map(|i| {
                format!(
                    "src/main/kotlin/com/example/service/persistence/OrderRepository{i}.kt:{}",
                    i + 1
                )
            })
            .collect();
        let w: &'static [&'static str] = leak(&writers);
        let specs: Vec<Spec> = (0..362)
            .map(|_| Spec {
                origin: "census",
                holder: Some("container-field"),
                writers: w,
                removers: &[],
            })
            .collect();
        // Owners and sites the length a Kotlin service has.
        let text = static_only_text(&specs, |_| REMEDY.into())
            .replace(
                "pkg.Owner",
                "com.example.service.persistence.impl.OrderRepositoryOwner",
            )
            .replace(
                "src/o",
                "src/main/kotlin/com/example/service/persistence/impl/o",
            );
        let md = render(&text);
        assert!(md.len() < 64 * 1024, "{} bytes", md.len());
        assert!(md.contains("#### Other static-only findings (347)\n"));
    }

    #[test]
    fn ranked_output_is_deterministic() {
        let t = static_only_text(&many(20), |_| REMEDY.into());
        assert_eq!(render(&t), render(&t));
    }

    #[test]
    fn a_run_with_362_census_findings_renders_under_80_kb() {
        let t = static_only_text(&many(362), |_| REMEDY.into());
        let md = render(&t);
        assert!(md.len() < 80 * 1024, "{} bytes", md.len());
        assert!(md.contains("#### Other static-only findings (347)\n"));
    }
}

mod measured_line {
    use super::*;
    use dla_core::{
        Authority, EvidenceKind, NewEvidence, Source, Tool, evidence::EvidenceCoverage,
    };

    const NOTHING: &str = "Measured: nothing. No harness ran and nothing was sampled or captured, so every finding rests on reading the code: this report neither shows nor rules out a leak.\n";

    fn ev(kind: EvidenceKind, authority: Authority) -> NewEvidence {
        NewEvidence {
            source: Source::Repo {
                commit: "abc".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "0".into(),
            },
            authority,
            kind,
            result: json!({}),
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        }
    }

    #[test]
    fn nothing_measured_follows_the_status_line_for_complete_and_incomplete() {
        let md = render_markdown(&gate_complete_view());
        assert!(
            md.contains(&format!("Status: COMPLETE\n\n{NOTHING}\nTime: budget 1h")),
            "{md}"
        );
        let md = render_markdown(&view("incomplete", None));
        assert!(
            md.contains(&format!(
                "Status: INCOMPLETE ({} open)\n\n{NOTHING}",
                open_count(&md)
            )),
            "{md}"
        );
    }

    fn open_count(md: &str) -> usize {
        md.lines()
            .find_map(|l| l.strip_prefix("Status: INCOMPLETE ("))
            .and_then(|r| r.split(' ').next())
            .and_then(|n| n.parse().ok())
            .unwrap()
    }

    #[test]
    fn a_measured_run_counts_harness_runs_series_and_captures() {
        let mut l = Ledger::from_text("").unwrap();
        for (k, a) in [
            (EvidenceKind::CensusHolder, Authority::Static),
            (EvidenceKind::Workload, Authority::Static),
            (EvidenceKind::Workload, Authority::Test),
            (EvidenceKind::Workload, Authority::Test),
            (EvidenceKind::GrowthSeries, Authority::Live),
            (EvidenceKind::HistogramDiff, Authority::Dump),
        ] {
            l.add_evidence(ev(k, a)).unwrap();
        }
        let md = render_markdown(&RunView::new(state("20260101-000000-abcd"), l, None));
        assert!(
            md.contains("\n\nMeasured: 2 harness runs, 1 series, 1 capture\n\n"),
            "{md}"
        );
        assert!(!md.contains("Measured: nothing"));
    }
}

// ------------------------------------------------------------------ Time line

mod time_line {
    use super::*;
    use dla_core::state::{BudgetEstimate, BudgetExtension};

    fn mins(n: u64) -> std::time::Duration {
        std::time::Duration::from_secs(n * 60)
    }

    /// The report of a state whose budget started `used` minutes before its newest
    /// ledger line (the ledger line is stamped now).
    fn report(mut st: RunState, ledger: Ledger, used: u64) -> String {
        st.budget.started_at = now_utc() - mins(used);
        render_markdown(&RunView::new(st, ledger, None))
    }

    fn time_of(md: &str) -> &str {
        md.lines()
            .find_map(|l| l.strip_prefix("Time: "))
            .unwrap_or_else(|| panic!("no Time line in {md}"))
    }

    fn with_evidence() -> Ledger {
        let mut l = Ledger::from_text("").unwrap();
        l.add_evidence(dla_core::NewEvidence {
            source: dla_core::Source::Repo {
                commit: "abc".into(),
                path: None,
            },
            tool: dla_core::Tool {
                name: "t".into(),
                version: "0".into(),
            },
            authority: dla_core::Authority::Static,
            kind: dla_core::EvidenceKind::StaticHit,
            result: json!({"lang": "jvm"}),
            raw: None,
            coverage: dla_core::evidence::EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        l
    }

    #[test]
    fn it_follows_the_measured_line_and_leaves_the_status_line_alone() {
        let md = report(
            state("20260101-000000-abcd"),
            Ledger::from_text("").unwrap(),
            0,
        );
        let lines: Vec<&str> = md.lines().collect();
        let status = lines
            .iter()
            .position(|l| l.starts_with("Status: "))
            .unwrap();
        let measured = lines
            .iter()
            .position(|l| l.starts_with("Measured: "))
            .unwrap();
        let time = lines.iter().position(|l| l.starts_with("Time: ")).unwrap();
        assert!(status < measured && time > measured, "{md}");
        assert_eq!(time, measured + 2, "right after the Measured line: {md}");
        assert!(
            lines[status] == "Status: COMPLETE"
                || lines[status].starts_with("Status: INCOMPLETE (")
        );
    }

    #[test]
    fn a_fixed_budget_names_its_length_and_the_time_used() {
        let mut st = state("20260101-000000-abcd");
        st.budget.seconds = 2700;
        let md = report(st, with_evidence(), 12);
        assert_eq!(time_of(&md), "budget 45m, 12m used.", "{md}");
    }

    #[test]
    fn a_settled_extended_auto_budget_says_how_it_was_sized_and_extended() {
        let mut st = state("20260101-000000-abcd");
        st.budget.auto = true;
        st.budget.settled = true;
        st.budget.seconds = 3 * 3600;
        st.budget.estimate = Some(BudgetEstimate::new(1, 362));
        st.budget.extensions = vec![BudgetExtension {
            at: now_utc(),
            seconds: 1800,
            lapsed_seconds: 0,
            deadline: now_utc(),
        }];
        let md = report(st, with_evidence(), 178);
        assert_eq!(
            time_of(&md),
            "budget 3h (auto: 1 runtime, 362 candidates; extended by 30m), 2h58m used.",
            "{md}"
        );
    }

    #[test]
    fn an_extension_made_after_the_deadline_says_what_was_spent_past_it() {
        let mut st = state("20260101-000000-abcd");
        // Fixed 60 m, extended by 30 m at minute 70: the budget is 100 m long, and the
        // 10 m past the deadline are named rather than left unexplained.
        st.budget.seconds = 6000;
        st.budget.extensions = vec![BudgetExtension {
            at: now_utc(),
            seconds: 1800,
            lapsed_seconds: 600,
            deadline: now_utc(),
        }];
        let md = report(st, with_evidence(), 75);
        assert_eq!(
            time_of(&md),
            "budget 1h40m (extended by 30m, plus 10m spent past the deadline), 1h15m used.",
            "{md}"
        );
    }

    #[test]
    fn a_budget_in_seconds_is_not_truncated_to_minutes() {
        let mut st = state("20260101-000000-abcd");
        st.budget.seconds = 90;
        let md = report(st, with_evidence(), 0);
        assert!(time_of(&md).starts_with("budget 1m30s, "), "{md}");
    }

    #[test]
    fn an_unsized_auto_budget_says_so() {
        let mut st = state("20260101-000000-abcd");
        st.budget.auto = true;
        let md = report(st, with_evidence(), 5);
        assert_eq!(
            time_of(&md),
            "budget 1h (auto, not sized), 5m used.",
            "{md}"
        );
    }

    /// Three python/jvm candidates, two closed `time_budget`, one open.
    fn closed_for_time() -> Ledger {
        closed_for_time_n(2)
    }

    /// Three candidates, the first `closed` of them closed `time_budget`.
    fn closed_for_time_n(closed: usize) -> Ledger {
        let mut text = String::new();
        text += "{\"schema_version\":1,\"seq\":1,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"evidence\",\"evidence_id\":\"E1\",\"source\":{\"repo\":{\"commit\":\"abc\"}},\"tool\":{\"name\":\"t\",\"version\":\"0\"},\"authority\":\"static\",\"kind\":\"static-hit\",\"result\":{\"lang\":\"jvm\"}}\n";
        let mut seq = 2;
        for n in 1..=3 {
            text += &format!(
                "{{\"schema_version\":1,\"seq\":{seq},\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"finding_added\",\"finding\":{{\"id\":\"F{n}\",\"state\":\"candidate\",\"origin\":\"census\",\"supports\":[\"E1\"],\"site\":\"src/f{n}.kt:{n}\"}}}}\n"
            );
            seq += 1;
        }
        for n in 1..=closed {
            text += &format!(
                "{{\"schema_version\":1,\"seq\":{seq},\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"transition\",\"finding\":\"F{n}\",\"from\":\"candidate\",\"to\":\"unresolved\",\"evidence\":[],\"by\":\"t\",\"reason\":\"time_budget\",\"remedy\":\"x\"}}\n"
            );
            seq += 1;
        }
        Ledger::from_text(&text).unwrap()
    }

    #[test]
    fn findings_closed_for_lack_of_time_say_what_measuring_them_needs() {
        let mut st = state("20260101-000000-abcd");
        st.budget.seconds = 3600;
        st.budget.started_at = now_utc() - mins(60);
        let md = render_markdown(&RunView::new(st, closed_for_time(), None));
        // The two findings closed for time x 3 + their one runtime without a harness run
        // x 15 = 21 more minutes (the open third finding is not in the sentence); 60 + 21.
        assert!(
            time_of(&md).ends_with(
                "2 findings were closed for lack of time; measuring them needs about 21 more minutes (a new run with --budget 81m)."
            ),
            "{md}"
        );
        assert!(time_of(&md).starts_with("budget 1h, "), "{md}");
    }

    #[test]
    fn one_finding_closed_for_lack_of_time_reads_in_the_singular() {
        let mut st = state("20260101-000000-abcd");
        st.budget.seconds = 3600;
        st.budget.started_at = now_utc() - mins(60);
        let l = closed_for_time_n(1);
        let md = render_markdown(&RunView::new(st, l, None));
        assert!(
            time_of(&md).contains(" 1 finding was closed for lack of time; measuring it needs"),
            "{md}"
        );
    }

    #[test]
    fn a_run_with_nothing_closed_for_time_has_no_such_sentence() {
        let md = report(state("20260101-000000-abcd"), with_evidence(), 3);
        assert!(!md.contains("closed for lack of time"), "{md}");
    }
}

mod findings_line {
    use super::*;

    const DISC: &str = "{\"schema_version\":1,\"seq\":2,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"evidence\",\"evidence_id\":\"E2\",\"source\":{\"repo\":{\"commit\":\"abc\"}},\"tool\":{\"name\":\"t\",\"version\":\"0\"},\"authority\":\"test\",\"kind\":\"discriminator\",\"result\":{\"favors\":\"non-leak\"}}\n";

    fn findings_of(md: &str) -> &str {
        md.lines()
            .find(|l| l.starts_with("Findings: "))
            .unwrap_or_else(|| panic!("{md}"))
    }

    /// The run of the user's report: a leak hypothesis refuted by a discriminator, and its
    /// `N-WARM` sibling confirmed. One test, two states.
    fn a_warmup_that_held() -> Ledger {
        let mut t = String::new();
        t += "{\"schema_version\":1,\"seq\":1,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"evidence\",\"evidence_id\":\"E1\",\"source\":{\"repo\":{\"commit\":\"abc\"}},\"tool\":{\"name\":\"t\",\"version\":\"0\"},\"authority\":\"static\",\"kind\":\"static-hit\",\"result\":{}}\n";
        t += DISC;
        t += "{\"schema_version\":1,\"seq\":3,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"state\":\"candidate\",\"origin\":\"census\",\"supports\":[\"E1\"]}}\n";
        t += "{\"schema_version\":1,\"seq\":4,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"finding_added\",\"finding\":{\"id\":\"F1.n1\",\"state\":\"candidate\",\"origin\":\"hypothesis\",\"supports\":[\"E1\"],\"parent\":\"F1\",\"hypothesis\":\"N-WARM: jit\",\"refute_if\":\"x\"}}\n";
        t += "{\"schema_version\":1,\"seq\":5,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"transition\",\"finding\":\"F1\",\"from\":\"candidate\",\"to\":\"refuted\",\"evidence\":[\"E2\"],\"by\":\"t\"}\n";
        t += "{\"schema_version\":1,\"seq\":6,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"transition\",\"finding\":\"F1.n1\",\"from\":\"candidate\",\"to\":\"under_test\",\"evidence\":[],\"by\":\"t\"}\n";
        t += "{\"schema_version\":1,\"seq\":7,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"transition\",\"finding\":\"F1.n1\",\"from\":\"under_test\",\"to\":\"confirmed\",\"evidence\":[\"E2\"],\"by\":\"t\"}\n";
        Ledger::from_text(&t).unwrap()
    }

    #[test]
    fn a_confirmed_explanation_is_not_a_confirmed_leak() {
        let md = render_markdown(&RunView::new(
            state("20260101-000000-abcd"),
            a_warmup_that_held(),
            None,
        ));
        let line = findings_of(&md);
        assert!(
            line.starts_with("Findings: 2 enumerated, 2 dispositioned; leak hypotheses: confirmed 0, strong 0, refuted 1, suspected 0, unresolved 0; non-leak explanations: confirmed 1, refuted 0, suspected 0, unresolved 0"),
            "{line}"
        );
        // A discriminator of authority test is a measurement, and no leak is confirmed.
        assert!(line.ends_with("; no leak confirmed"), "{line}");
    }

    #[test]
    fn a_measured_run_with_no_leak_says_so_in_words() {
        let mut t = String::new();
        t += "{\"schema_version\":1,\"seq\":1,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"evidence\",\"evidence_id\":\"E1\",\"source\":{\"repo\":{\"commit\":\"abc\"}},\"tool\":{\"name\":\"t\",\"version\":\"0\"},\"authority\":\"static\",\"kind\":\"static-hit\",\"result\":{}}\n";
        t += DISC;
        t += "{\"schema_version\":1,\"seq\":3,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"state\":\"candidate\",\"origin\":\"census\",\"supports\":[\"E1\"]}}\n";
        t += "{\"schema_version\":1,\"seq\":4,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"transition\",\"finding\":\"F1\",\"from\":\"candidate\",\"to\":\"refuted\",\"evidence\":[\"E2\"],\"by\":\"t\"}\n";
        let md = render_markdown(&RunView::new(
            state("20260101-000000-abcd"),
            Ledger::from_text(&t).unwrap(),
            None,
        ));
        // E2 is a test-authority discriminator: something was measured.
        let line = findings_of(&md);
        assert!(line.ends_with("; no leak confirmed"), "{line}");
        assert!(
            line.contains("leak hypotheses: confirmed 0, strong 0, refuted 1"),
            "{line}"
        );
    }
}

mod coverage_warning_and_explained {
    use super::*;

    fn cov(
        discovered: usize,
        exercised: usize,
        iterations: Option<u64>,
        project: Option<u64>,
    ) -> CoverageManifest {
        let names = |n: usize| {
            (0..n)
                .map(|i| format!("a/B{i}.kt::B{i}.f"))
                .collect::<Vec<_>>()
        };
        serde_json::from_value(json!({
            "schema_version": 1,
            "workers": {"jvm": {
                "runtime": "jvm",
                "entry_points": {"discovered": names(discovered), "exercised": names(exercised)},
                "iterations": iterations,
                "project_classes": project
            }}
        }))
        .unwrap()
    }

    fn md_of(c: CoverageManifest) -> String {
        render_markdown(&RunView::new(
            state("20260101-000000-abcd"),
            Ledger::from_text("").unwrap(),
            Some(c),
        ))
    }

    fn warning(md: &str) -> Option<&str> {
        md.lines().find(|l| l.starts_with("Coverage warning: "))
    }

    #[test]
    fn zero_of_many_after_iterations_is_flagged_next_to_measured() {
        let md = md_of(cov(225, 0, Some(750), Some(37)));
        let w = warning(&md).unwrap_or_else(|| panic!("{md}"));
        assert!(
            w.contains("this run exercised 0 of 225 entry points (jvm)"),
            "{w}"
        );
        assert!(w.contains("hypotheses, not measured leaks"), "{w}");
        assert!(
            w.contains("750 iterations ran and none called an entry point"),
            "{w}"
        );
        assert!(w.contains("37 project classes loaded"), "{w}");
        assert!(
            w.contains("To measure: dla harness gen --lang jvm --driver, fill the DLA-HOOK bodies"),
            "{w}"
        );
        // Right after the Measured line, before the Time line.
        let lines: Vec<&str> = md.lines().collect();
        let measured = lines
            .iter()
            .position(|l| l.starts_with("Measured: "))
            .unwrap();
        let time = lines.iter().position(|l| l.starts_with("Time: ")).unwrap();
        assert_eq!(lines[measured + 2], w, "{md}");
        assert!(time > measured + 2, "{md}");
        // The Status line and the Measured text are unchanged.
        assert!(md.contains("Status: INCOMPLETE") || md.contains("Status: COMPLETE"));
    }

    #[test]
    fn a_minority_is_flagged_and_half_or_more_is_not() {
        let md = md_of(cov(10, 4, Some(100), None));
        let w = warning(&md).unwrap_or_else(|| panic!("{md}"));
        assert!(w.contains("exercised 4 of 10"), "{w}");
        assert!(w.contains("most entry points were never called"), "{w}");
        assert!(warning(&md_of(cov(10, 5, Some(100), None))).is_none());
        assert!(warning(&md_of(cov(2, 1, None, None))).is_none());
        // No entry points discovered, or no manifest: nothing to warn about.
        assert!(warning(&md_of(cov(0, 0, Some(5), None))).is_none());
        let none = render_markdown(&RunView::new(
            state("20260101-000000-abcd"),
            Ledger::from_text("").unwrap(),
            None,
        ));
        assert!(warning(&none).is_none());
    }

    #[test]
    fn a_runtime_that_cannot_record_exercised_entry_points_gets_no_warning() {
        for rt in ["rust", "native"] {
            let c: CoverageManifest = serde_json::from_value(json!({
                "schema_version": 1,
                "workers": {rt: {
                    "runtime": rt,
                    "entry_points": {"discovered": ["src/lib.rs::f", "src/lib.rs::g"], "exercised": []},
                    "iterations": 880
                }},
                "blind_spots": [{"runtime": rt, "text": "the allocator shim does not record which entry points ran"}]
            }))
            .unwrap();
            assert!(warning(&md_of(c)).is_none(), "{rt}");
        }
        // With no iteration at all, "0 exercised" is no shim limit: the warning stays.
        let c: CoverageManifest = serde_json::from_value(json!({
            "schema_version": 1,
            "workers": {"rust": {
                "runtime": "rust",
                "entry_points": {"discovered": ["src/lib.rs::f", "src/lib.rs::g"], "exercised": []}
            }},
            "blind_spots": [{"runtime": "rust", "text": "the allocator shim does not record which entry points ran"}]
        }))
        .unwrap();
        let md = md_of(c);
        let w = warning(&md).unwrap_or_else(|| panic!("{md}"));
        assert!(w.contains("no workload iteration was recorded"), "{w}");
    }

    #[test]
    fn no_iterations_names_that_cause() {
        let md = md_of(cov(3, 0, None, None));
        let w = warning(&md).unwrap_or_else(|| panic!("{md}"));
        assert!(w.contains("no workload iteration was recorded"), "{w}");
    }

    const E: &str = "{\"schema_version\":1,\"seq\":1,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"evidence\",\"evidence_id\":\"E1\",\"source\":{\"repo\":{\"commit\":\"abc\"}},\"tool\":{\"name\":\"t\",\"version\":\"0\"},\"authority\":\"static\",\"kind\":\"static-hit\",\"result\":{}}\n";

    /// A leak hypothesis F1 still unresolved and its `N-WARM` sibling confirmed.
    fn explained_ledger() -> Ledger {
        let mut t = String::from(E);
        t += "{\"schema_version\":1,\"seq\":2,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"evidence\",\"evidence_id\":\"E2\",\"source\":{\"repo\":{\"commit\":\"abc\"}},\"tool\":{\"name\":\"t\",\"version\":\"0\"},\"authority\":\"test\",\"kind\":\"discriminator\",\"result\":{\"favors\":\"non-leak\"}}\n";
        t += "{\"schema_version\":1,\"seq\":3,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"state\":\"candidate\",\"origin\":\"census\",\"supports\":[\"E1\"]}}\n";
        t += "{\"schema_version\":1,\"seq\":4,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"finding_added\",\"finding\":{\"id\":\"F1.n1\",\"state\":\"candidate\",\"origin\":\"hypothesis\",\"supports\":[\"E1\"],\"parent\":\"F1\",\"hypothesis\":\"N-WARM: jit\",\"refute_if\":\"x\"}}\n";
        t += "{\"schema_version\":1,\"seq\":5,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"transition\",\"finding\":\"F1\",\"from\":\"candidate\",\"to\":\"refuted\",\"evidence\":[\"E2\"],\"by\":\"t\"}\n";
        t += "{\"schema_version\":1,\"seq\":6,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"transition\",\"finding\":\"F1.n1\",\"from\":\"candidate\",\"to\":\"under_test\",\"evidence\":[],\"by\":\"t\"}\n";
        t += "{\"schema_version\":1,\"seq\":7,\"ts\":\"2026-01-01T00:00:00Z\",\"type\":\"transition\",\"finding\":\"F1.n1\",\"from\":\"under_test\",\"to\":\"confirmed\",\"evidence\":[\"E2\"],\"by\":\"t\"}\n";
        Ledger::from_text(&t).unwrap()
    }

    #[test]
    fn a_confirmed_explanation_has_its_own_section_not_the_findings_list() {
        let md = render_markdown(&RunView::new(
            state("20260101-000000-abcd"),
            explained_ledger(),
            None,
        ));
        let findings = md.split("## Findings").nth(1).unwrap();
        let findings = findings.split("## Explained, not leaks").next().unwrap();
        assert!(!findings.contains("### F1.n1"), "{md}");
        assert!(
            findings.contains("No leak hypotheses to report"),
            "{findings}"
        );
        let explained = md.split("## Explained, not leaks").nth(1).unwrap();
        let explained = explained.split("\n## ").next().unwrap();
        assert!(explained.contains("### F1.n1: Hypothesis N-WARM"), "{md}");
        assert!(
            explained.contains("not a leak and not a defect"),
            "{explained}"
        );
        assert!(explained.contains("Supporting evidence"), "{explained}");
    }

    #[test]
    fn golden_zero_coverage_with_an_explanation() {
        let md = render_markdown(&RunView::new(
            state("20260101-000000-abcd"),
            explained_ledger(),
            Some(cov(225, 0, Some(750), Some(37))),
        ));
        check_golden("zero_coverage_explained", &md);
    }

    #[test]
    fn no_explanation_means_no_section() {
        let md = render_markdown(&RunView::new(
            state("20260101-000000-abcd"),
            Ledger::from_text(E).unwrap(),
            None,
        ));
        assert!(!md.contains("## Explained, not leaks"), "{md}");
    }
}
