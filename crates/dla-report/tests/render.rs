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
            md.contains("### F1: **L-COLLECT**\n\n- State: strong\n- Label: L-COLLECT (evidence: E4 growth-series, E5 retention-path)\n"),
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
