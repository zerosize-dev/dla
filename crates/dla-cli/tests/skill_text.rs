//! The shipped skill text: size limits, the rules and the blocks that must come first, and the
//! words the prose ablation must not borrow from the binary.

use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn skill() -> String {
    fs::read_to_string(root().join("skills/detect-leak-anywhere/SKILL.md")).unwrap()
}

/// The frontmatter between the first two `---` lines, and the body after them.
fn split(text: &str) -> (&str, &str) {
    let rest = text.strip_prefix("---\n").expect("frontmatter");
    let end = rest.find("\n---\n").expect("frontmatter end");
    (&rest[..end], &rest[end + 5..])
}

/// The text of the `## <title>` section, up to the next `## ` heading.
fn section<'a>(body: &'a str, title: &str) -> &'a str {
    let head = format!("\n## {title}");
    let start = body
        .find(&head)
        .unwrap_or_else(|| panic!("no section {title}"))
        + 1;
    let rest = &body[start..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |i| i + 3);
    &rest[..end]
}

#[test]
fn the_skill_stays_inside_its_size_budget() {
    let text = skill();
    // Claude Code re-attaches only the first 5,000 tokens of an invoked skill after a
    // compaction (about 18 KB), and recommends under 500 lines.
    assert!(text.len() <= 18 * 1024, "{} bytes", text.len());
    assert!(
        text.lines().count() <= 500,
        "{} lines",
        text.lines().count()
    );
}

#[test]
fn the_frontmatter_keeps_name_description_and_hooks_and_no_effort() {
    let text = skill();
    let (front, _) = split(&text);
    let keys: Vec<&str> = front
        .lines()
        .filter(|l| !l.starts_with(' ') && !l.starts_with('\t'))
        .filter_map(|l| l.split_once(':').map(|(k, _)| k))
        .collect();
    assert_eq!(keys, ["name", "description", "hooks"], "{front}");
    assert!(front.contains("name: detect-leak-anywhere"));
    // The benchmark stays comparable: the high-effort advice is text, not a setting.
    assert!(!front.contains("effort:"));
    let (_, body) = split(&text);
    assert!(body.contains("Work at high effort"));
}

#[test]
fn five_rules_then_the_checklist_and_both_blocks_come_first() {
    let text = skill();
    let (_, body) = split(&text);
    let rules = section(body, "Non-negotiables (MUST)");
    let numbered: Vec<&str> = rules
        .lines()
        .filter(|l| l.starts_with(|c: char| c.is_ascii_digit()))
        .collect();
    assert_eq!(numbered.len(), 5, "{rules}");
    for (i, l) in numbered.iter().enumerate() {
        assert!(l.starts_with(&format!("{}. ", i + 1)), "{l}");
    }
    // Rules come first in the body; the four blocks start in the first half of the budget.
    let first_heading = body.find("\n## ").unwrap();
    assert!(body[first_heading..].starts_with("\n## Non-negotiables (MUST)"));
    // The first half of the 18 KB budget: what survives a compaction with room to spare.
    let half = 9 * 1024;
    for title in [
        "Non-negotiables (MUST)",
        "Stage checklist",
        "When nothing fits",
        "Before you stop",
    ] {
        let at = text
            .find(&format!("\n## {title}"))
            .unwrap_or_else(|| panic!("{title}"));
        assert!(at < half, "{title} starts at byte {at}, past {half}");
    }
}

#[test]
fn the_checklist_carries_the_m6_steps() {
    let text = skill();
    let (_, body) = split(&text);
    let list = section(body, "Stage checklist");
    let code = list.split("```").nth(1).expect("a fenced checklist");
    assert_eq!(code.matches("- [ ] ").count(), 9, "{code}");
    for want in [
        "gate: classify",
        "-archetypes.md",
        "known-leaks.md",
        "dla ledger label",
        "--as UNCLASSIFIED",
        "dla ledger explain",
        "strong",
        "dla intervene",
        "dla bundle",
        "dla:verifier",
        "in every path",
        "dla ledger merge",
        "dla gate check --final",
        "dla report",
    ] {
        assert!(code.contains(want), "checklist lacks {want:?}:\n{code}");
    }
    // Strong comes after explain, and last.
    assert!(code.find("explain").unwrap() < code.find("strong").unwrap());
}

#[test]
fn nothing_fits_and_before_you_stop_say_what_they_must() {
    let text = skill();
    let (_, body) = split(&text);
    let nothing = section(body, "When nothing fits");
    for want in [
        "UNCLASSIFIED",
        "holder and mechanism",
        "success",
        "Never force a label",
        "refuses",
        "only when no archetype fits the mechanism",
        "run the command the refusal names once, then label again",
        "label decision for each confirmed finding",
        "`--note` only to keep the ID, and it must say why none of the runtime's rows fits",
        "what is met and what is missing",
        "`dla ledger duplicate F --of G`",
        "from `grep -n` or `cat -n` output, never from counting an unnumbered listing",
    ] {
        assert!(nothing.contains(want), "When nothing fits lacks {want:?}");
    }
    let worker = fs::read_to_string(root().join("agents/worker.md")).unwrap();
    for want in [
        "UNCLASSIFIED only when no archetype fits the mechanism",
        "run the command the refusal names once, then label again",
        "label decision",
        "label-runtime-row",
        "`dla --run <id> --worker <name> ledger duplicate F --of G`",
        "`grep -n` or `cat -n` output",
    ] {
        assert!(worker.contains(want), "worker.md lacks {want:?}");
    }
    let node =
        fs::read_to_string(root().join("skills/detect-leak-anywhere/references/node.md")).unwrap();
    assert!(node.contains("Timers (`setInterval`, `setTimeout`), event listeners and subscriptions are `registration-table`"));
    let stop = section(body, "Before you stop");
    for want in [
        "dla gate check --final",
        "reason and a next step",
        "INCOMPLETE",
        "Do not rely on it",
        "integrity seals",
    ] {
        assert!(stop.contains(want), "Before you stop lacks {want:?}");
    }
}

#[test]
fn a_time_budget_refusal_that_names_a_smaller_plan_is_followed_not_ended() {
    // `harness run`, `alloc-test` and `intervene` refuse a plan the deadline cannot hold and
    // name a smaller one; the texts must not send the model straight to `time_budget`.
    let recovery =
        fs::read_to_string(root().join("skills/detect-leak-anywhere/references/recovery.md"))
            .unwrap();
    let texts = [
        ("SKILL.md", skill()),
        (
            "agents/worker.md",
            fs::read_to_string(root().join("agents/worker.md")).unwrap(),
        ),
        ("references/recovery.md", recovery),
    ];
    for (name, text) in &texts {
        assert!(
            text.contains("names a smaller"),
            "{name} lacks the smaller-plan rule"
        );
        assert!(
            text.contains("partial run"),
            "{name} lacks the partial-run rule"
        );
    }
}

#[test]
fn the_prose_arm_has_the_unclassified_wording_and_no_binary_commands() {
    let text = fs::read_to_string(root().join("bench/ablations/prose/SKILL.md")).unwrap();
    assert!(text.contains("UNCLASSIFIED is a result, not a failure"));
    assert!(text.contains("Never force a label"));
    for banned in [
        "dla ",
        "`dla`",
        "ledger",
        "gate check",
        "intervene",
        "verifier",
        "archetypes.toml",
        "hooks:",
    ] {
        assert!(!text.contains(banned), "the prose arm mentions {banned:?}");
    }
}

#[test]
fn the_runtime_row_rule_names_the_known_leaks_route() {
    let text = std::fs::read_to_string(
        root().join("skills/detect-leak-anywhere/references/evidence-and-report.md"),
    )
    .unwrap();
    let rule = text
        .lines()
        .find(|l| l.contains("`ledger label F3 --as L-X`"))
        .expect("the label row of the move table");
    assert!(
        rule.contains("a shipped `known-leaks.toml` row names that archetype for the runtime"),
        "{rule}"
    );
}

#[test]
fn the_skill_asks_before_closing_for_time_and_never_extends_an_unattended_run() {
    let skill = skill();
    for want in [
        "extend by <to_finish> minutes to measure the next <n> candidates?",
        "`dla run extend <dur>`",
        "A worker, or a run whose `dla run init` reported a `consent` object, never extends",
        "Answer `intake.budget` with the `dla run init` evidence",
        "`auto`",
    ] {
        assert!(skill.contains(want), "SKILL.md lacks {want:?}");
    }
    let recovery =
        fs::read_to_string(root().join("skills/detect-leak-anywhere/references/recovery.md"))
            .unwrap();
    assert!(recovery.contains("dla run extend <dur>") && recovery.contains("never extends"));
    let stages =
        fs::read_to_string(root().join("skills/detect-leak-anywhere/references/stages.md"))
            .unwrap();
    assert!(
        stages.contains("clamp(20 + 15*R + 3*C, 60, 180)"),
        "stages.md states the formula"
    );
    let report = fs::read_to_string(
        root().join("skills/detect-leak-anywhere/references/evidence-and-report.md"),
    )
    .unwrap();
    assert!(report.contains("`Time:` line"));
}

#[test]
fn the_plugin_time_budget_defaults_to_auto_and_still_takes_a_fixed_value() {
    let text = fs::read_to_string(root().join(".claude-plugin/plugin.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let b = &v["userConfig"]["time_budget"];
    assert_eq!(b["default"], "auto", "{b}");
    let d = b["description"].as_str().unwrap();
    assert!(d.contains("auto") && d.contains("90s, 45m or 2h"), "{d}");
}

#[test]
fn each_closing_reason_is_taught_with_when_it_is_true() {
    let read = |p: &str| fs::read_to_string(root().join(p)).unwrap();
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let recovery = read("skills/detect-leak-anywhere/references/recovery.md");
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    let worker = read("agents/worker.md");
    let skill = skill();
    // Every text that lists the reasons lists `inconclusive`, and says `time_budget` holds
    // only when the deadline is near or a command was refused for time.
    for (name, text) in [("SKILL.md", &skill), ("stages.md", &stages)] {
        assert!(
            text.contains("`inconclusive`") && text.contains("measurement tied to the finding"),
            "{name} lacks the inconclusive rule"
        );
        assert!(
            text.contains("a command was refused for time"),
            "{name} lacks the time_budget condition"
        );
    }
    assert!(worker.contains("`time-left`") && worker.contains("`inconclusive`"));
    assert!(recovery.contains("`inconclusive`"));
    assert!(report.contains("leak hypotheses:") && report.contains("non-leak explanations:"));
    // A stalled or killed worker is no longer closed `time_budget` whatever happened.
    assert!(
        !recovery.contains("--reason time_budget|env_unavailable"),
        "recovery.md still closes a killed worker's items time_budget"
    );
    // Non-heap growth goes through the alloc-test route before any close.
    assert!(skill.contains("`N-WARM` sibling") && skill.contains("dla alloc-test"));
}

#[test]
fn the_keep_going_rules_sit_inside_before_you_stop_with_their_reason() {
    let text = skill();
    let (front, body) = split(&text);
    let stop = section(body, "Before you stop");
    for want in [
        // The reason, then the rule: unspent budget measured nothing.
        "Keep measuring while time is left",
        "measured nothing, so it shows neither a leak nor its absence",
        "never ask whether to continue",
        "An unattended run records the blocker with `--remedy`",
        "what only the user can supply",
        // A harness is run, or generated, before a close that says nothing ran.
        "or generate one when entry points were discovered",
        "the ledger refuses otherwise",
        // The run that measured nothing says so first.
        "say first and plainly that the run measured nothing",
        // The closing message copies the report and never calls an explanation confirmed.
        "Closing message:",
        "never \"confirmed\"",
        "this run did not test the rest",
        "Copy every number from `dla report` or `dla run status`",
        // A passing gate is not a verdict.
        "not that the finding is a leak",
        "`Verified: DISAGREES` finding is told to the user as disputed",
    ] {
        assert!(stop.contains(want), "Before you stop lacks {want:?}");
    }
    // The time_budget condition stays in the reasons bullet, with its refusal.
    assert!(stop.contains(
        "`time_budget` only when the deadline is near or a command was refused for time"
    ));
    // Item 9 is a loop with an exit, and the pointer is keyed by situation.
    let list = section(body, "Stage checklist");
    let item9 = list
        .lines()
        .find(|l| l.contains("- [ ] 9."))
        .expect("item 9");
    assert!(
        item9.contains("loop") && item9.contains("until nothing is runnable or time ends"),
        "{item9}"
    );
    assert!(body.contains("Read by situation (`references/`)"));
    for file in [
        "recovery.md",
        "stages.md",
        "evidence-and-report.md",
        "safety-and-effects.md",
    ] {
        assert!(body.contains(file), "the pointer lacks {file}");
    }
    // The description applies to a growing service whoever says leak, and names what it is not.
    let desc = front
        .lines()
        .find(|l| l.starts_with("description:"))
        .expect("description");
    assert!(desc.contains("leak named or not"), "{desc}");
    assert!(desc.contains("not CPU profiling or GC tuning"), "{desc}");
    assert!(desc.len() < 700, "{} bytes", desc.len());
}

#[test]
fn the_references_state_the_loop_the_final_items_and_the_closing_message() {
    let read = |p: &str| fs::read_to_string(root().join(p)).unwrap();
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    let recovery = read("skills/detect-leak-anywhere/references/recovery.md");
    let jvm = read("skills/detect-leak-anywhere/references/jvm.md");
    let worker = read("agents/worker.md");
    let verifier = read("agents/verifier.md");
    for (name, text, wants) in [
        (
            "stages.md",
            &stages,
            vec![
                "Contents:",
                "Never ask the user whether to continue while time is left",
                "no fixed number of rounds",
                "`harness-not-generated`",
                "`coverage_contradiction`",
                "`test-loop/<H>/tests.json`",
                "`dla census --lang python` writes it too when the run has none and the project has Python test files",
                "the classes that ran and added nothing",
                "`budget_left_unrevisited`",
                "`next_command`",
                "reads as non-leak:",
                "--runtime <rt>`; it closes the queue of that runtime only",
            ],
        ),
        (
            "evidence-and-report.md",
            &report,
            vec![
                "Contents:",
                "`Coverage warning:`",
                "## Explained, not leaks",
                "### The closing message",
                "never \"confirmed\"",
            ],
        ),
        (
            "recovery.md",
            &recovery,
            vec![
                "## Tempting moves",
                "should I continue?",
                "`budget_left_unrevisited`",
                "The blocker names one runtime and closes the queue of that runtime only",
                "`workload-ran`",
            ],
        ),
        (
            "jvm.md",
            &jvm,
            vec![
                "never run `gradle`, `mvn` or",
                "`java -cp` to print or assemble one",
                "`fed_by` and `ran_flat`",
                "`build_fails`",
            ],
        ),
        (
            "agents/worker.md",
            &worker,
            vec![
                "Measure before you close",
                "(with `--runtime <runtime>`, your runtime)",
                "`harness-not-generated`",
                "each stage section as you reach it",
            ],
        ),
        (
            "agents/verifier.md",
            &verifier,
            vec![
                "whether the owner would call this a defect",
                "reads as non-leak:",
                "DISAGREES: reads as non-leak",
                "`test-loop/` directory",
                "`fed_by`",
                "`ran_and_added_nothing`",
                "`classes_run`",
                "compared by identity",
            ],
        ),
    ] {
        for want in wants {
            assert!(text.contains(want), "{name} lacks {want:?}");
        }
    }
    // The tempting-move table holds only moves no gate refuses.
    let table = recovery.split("## Tempting moves").nth(1).unwrap();
    assert!(
        table.lines().filter(|l| l.starts_with("| ")).count() >= 6,
        "{table}"
    );
}

#[test]
fn the_measurement_queue_loop_is_in_the_checklist_and_the_rules_and_nothing_closes_before_it_ends()
{
    let read = |p: &str| fs::read_to_string(root().join(p)).unwrap();
    let text = skill();
    let (_, body) = split(&text);
    // The loop is in the checklist an agent copies, with its exact commands.
    let list = section(body, "Stage checklist");
    let code = list.split("```").nth(1).expect("a fenced checklist");
    for want in [
        "dla ledger list",
        "dla harness gen --lang <rt> --driver --next",
        "dla harness run H<n> --mode both",
        "dla trend S<n>",
        "`targets`",
        "grew -> dla ledger test",
        "constant, not_loaded, not_executed",
        "dla ledger close-rest",
    ] {
        assert!(code.contains(want), "the checklist lacks {want:?}:\n{code}");
    }
    assert_eq!(code.matches("- [ ] ").count(), 9, "{code}");
    // The rules name the refusal and its fix, and the stop section the queue.
    assert!(list.contains("`target-pending`") && list.contains("never by another reason"));
    let stop = section(body, "Before you stop");
    assert!(stop.contains("`dla ledger close-rest`") && stop.contains("No `queue` is open"));
    // The skill never mentions the switch of fixed policies.
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let recovery = read("skills/detect-leak-anywhere/references/recovery.md");
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    let worker = read("agents/worker.md");
    for (name, t) in [
        ("SKILL.md", &text),
        ("stages.md", &stages),
        ("recovery.md", &recovery),
        ("evidence-and-report.md", &report),
        ("worker.md", &worker),
    ] {
        assert!(!t.contains("--queue"), "{name} mentions the queue switch");
        assert!(!t.contains("not_exercised\" in a shell loop"), "{name}");
    }
    for want in [
        "`dla ledger list`",
        "`--next N`",
        "`too-many-targets`",
        "`target-pending`",
        "`dla ledger close-rest",
        "`not_loaded`",
        "`not_executed`",
        "never close it with another reason",
        "`queue` object per runtime",
    ] {
        assert!(stages.contains(want), "stages.md lacks {want:?}");
    }
    // The old advice to close what the harness did not exercise is gone from stage 7.
    assert!(
        !stages.contains(
            "A candidate the harness could not exercise is neither confirmed nor refuted:"
        ),
        "stage 7 still sanctions the close"
    );
    assert!(recovery.contains("## A candidate is pending (`target-pending`)"));
    assert!(recovery.contains("## Too many targets (`too-many-targets`)"));
    assert!(
        report.contains("`Queue:` line")
            && report.contains("`Targeted: H2 constant, H3 not executed`")
    );
    assert!(report.contains("`Not targeted (rank 143)`"));
    for want in [
        "ledger list",
        "--driver --next",
        "`target-pending`",
        "ledger close-rest",
        "`not_loaded`",
    ] {
        assert!(worker.contains(want), "worker.md lacks {want:?}");
    }
}

#[test]
fn every_text_that_offers_the_workload_blocker_names_the_runtime() {
    // The blocker closes the queue of one runtime; the agent must say which.
    let read = |p: &str| fs::read_to_string(root().join(p)).unwrap();
    for (name, text) in [
        ("SKILL.md", skill()),
        (
            "references/stages.md",
            read("skills/detect-leak-anywhere/references/stages.md"),
        ),
        (
            "references/recovery.md",
            read("skills/detect-leak-anywhere/references/recovery.md"),
        ),
        ("agents/worker.md", read("agents/worker.md")),
    ] {
        assert!(
            text.contains("intake.workload") && text.contains("--runtime <"),
            "{name} offers the blocker without --runtime"
        );
        assert!(
            !text.contains("closes the runtime's queue")
                && !text.contains("closes the queue of the runtime"),
            "{name} says the blocker closes every queue of the runtime"
        );
    }
}

#[test]
fn the_references_teach_the_python_allocation_site_columns_and_their_route() {
    // Line breaks do not matter: every text is read with its whitespace collapsed.
    let read = |p: &str| {
        fs::read_to_string(root().join(p))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let python = read("skills/detect-leak-anywhere/references/python.md");
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    // What the columns count, where they exist and how they tie.
    for want in [
        "`py.blocks@<file:line>`",
        "(driver runs only)",
        "ties to the finding whose own site is that line and to no other",
        "dla ledger add --origin growth-series --evidence E --site <file:line> --region heap --holder <kind>",
        "native-handle/global-ref",
        "`dla alloc-test H --finding F`",
        "not by `py.traced`",
    ] {
        assert!(python.contains(want), "python.md lacks: {want}");
    }
    // The blind spot says what shows and what does not.
    assert!(
        !python.contains("Reference-count errors in C extensions show only as native growth"),
        "the old blind spot is still there"
    );
    assert!(
        python.contains(
            "shows as a growing `py.blocks@` site, at the line that calls into the extension"
        ) && python.contains("Memory the extension takes with `malloc` does not"),
        "python.md does not correct the blind spot"
    );
    // The two general references name the Python columns where they describe per-site counts.
    assert!(stages.contains("`py.blocks@<file:line>`"), "stages.md");
    assert!(
        report.contains("`py.blocks@<file:line>`"),
        "evidence-and-report.md"
    );
}

#[test]
fn the_references_teach_the_stack_capture_after_an_alloc_test_at_a_growing_line() {
    // Line breaks do not matter: every text is read with its whitespace collapsed.
    let read = |p: &str| {
        fs::read_to_string(root().join(p))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let python = read("skills/detect-leak-anywhere/references/python.md");
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    let verify = read("docs/VERIFY.md");
    for want in [
        "After `dla alloc-test` favors leak, capture the stacks that lead to the line",
        "run the `dla harness run H --mode <m> --capture alloc,heap` command it prints first, then the two `dla summarize` commands that run prints",
        "then label, `dla bundle` and explain",
        "`stacks-not-captured`",
        "The frame above the growing line shows who receives the object",
        "read that module's source for the reference it keeps",
    ] {
        assert!(python.contains(want), "python.md lacks: {want}");
    }
    assert!(
        report.contains("| `stacks-not-captured` |"),
        "evidence-and-report.md does not list the stacks-not-captured refusal"
    );
    assert!(
        verify.contains(
            "| A finding at a growing line reached the verifier without a stack | corrected |"
        ),
        "VERIFY.md has no row for the finding that reached the verifier without a stack"
    );
}

#[test]
fn the_references_teach_a_measured_holder_first_and_that_a_line_the_unit_does_not_execute_proves_nothing()
 {
    // Line breaks do not matter: every text is read with its whitespace collapsed.
    let read = |p: &str| {
        fs::read_to_string(root().join(p))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let python = read("skills/detect-leak-anywhere/references/python.md");
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    let verifier = read("agents/verifier.md");
    let verify = read("docs/VERIFY.md");
    for want in [
        "A holder \"of the project\" has a module global or a class attribute of a project source file that is not test code as its root",
        "a holder \"of the tests\" is rooted in test code (the driver module `__main__` or the generated harness module, a file under a directory named `test` or `tests`, `test_*.py`, `*_test.py`, `conftest.py`) or in a module of the test runner",
        "A holder of the project that keeps a growing line and that no finding has the owner of is the first thing to act on",
        "`dla harness gen --next` is refused with `measured-holder-first`",
        "A growing line that only a holder of the tests keeps is no finding of the project",
        "A line the unit does not execute proves nothing.",
        "a line on a branch the unit does not take allocates nothing, so nothing stays, and that is no evidence against a leak at the line",
        "When the function ran and the line did not, or the series has no count of the line, it says `inconclusive`",
        "Never refute a leak with such a result.",
    ] {
        assert!(python.contains(want), "python.md lacks: {want}");
    }
    for want in [
        "A holder of the project (a module global or a class attribute of a project source file that is not test code) that keeps a growing line and that no finding has the owner of is the first thing to act on, before the next driver and before any alloc-test",
        "`dla harness gen --next` is refused with `measured-holder-first`",
        "A holder of the tests (the driver, a file of test code, the test runner) is no finding",
        "A line the unit does not execute proves nothing (a branch it does not take allocates nothing): an alloc-test that says `inconclusive` because the line did not run is not evidence against the leak.",
    ] {
        assert!(stages.contains(want), "stages.md lacks: {want}");
    }
    assert!(
        stages.contains("A series of another driver that never showed the growth is not evidence against it: when the deciding column of a finding is flat from its first measured point of the alloc-test's series, the result is `inconclusive`")
            && stages.contains("The harness the finding was raised on and an entry-path harness of the finding are the exceptions: a harness the finding cites evidence from")
            && stages.contains("the run record of a hand-written hook driver does not count")
            && stages.contains("A harness that only declares the finding a target or has a probe for it, and that the finding cites nothing from, is another driver."),
        "stages.md lacks the sentence on a series that never showed the growth"
    );
    assert!(
        stages.contains(
            "Only a column that grew in the main window and then stopped is a plateau (`N-WARM`)."
        ),
        "stages.md lacks the plateau sentence"
    );
    assert!(
        report.contains("| `measured-holder-first` |"),
        "evidence-and-report.md does not list the measured-holder-first refusal"
    );
    assert!(
        verifier.contains(
            "A `favors: inconclusive` alloc-test whose text says the line did not run (\"the function of ... ran but the line itself did not\", or that the series has no count of the line) is not evidence against the leak: the unit never executed the line."
        ),
        "verifier.md lacks the sentence on an alloc-test that says the line did not run"
    );
    for want in [
        "| A holder of the project that a capture named had no consequence | corrected |",
        "| A line the unit did not execute was read as a line that ran and kept nothing | corrected |",
        "| A series that never showed the growth was read as a plateau | corrected |",
    ] {
        assert!(verify.contains(want), "VERIFY.md lacks the row: {want}");
    }
    // The skill itself is not touched by these rules: it stays short.
    assert!(!skill().contains("measured-holder-first"));
}

#[test]
fn the_references_teach_the_pytest_loop_as_the_first_pass_of_a_python_queue() {
    // Line breaks do not matter: every text is read with its whitespace collapsed.
    let read = |p: &str| {
        fs::read_to_string(root().join(p))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let python = read("skills/detect-leak-anywhere/references/python.md");
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let skill = read("skills/detect-leak-anywhere/SKILL.md");
    let readme = read("harness/python/README.md");
    let verifier = read("agents/verifier.md");
    for want in [
        "The project's own tests in a loop (`dla harness gen --lang python --pytest [--tests a,b]`)",
        "again and again",
        "declares no target",
        "pytest in the harness interpreter",
        "python3 -m pip install pytest",
        "`bin` directory of a virtual environment",
        "the scratch copy is made when the harness is generated and keeps the extension modules built in place",
        "`python3 setup.py build_ext --inplace`",
        "`DLA-PYTEST` line",
        "`dropped test file ...`",
        "`dla harness gen --lang python --driver --probe F`",
        "re-runs the whole suite per run",
        "a candidate of the queue still needs its driver",
        "`--tests` narrows the loop: it is for a loop that cannot run in full (it stopped with a `DLA-PYTEST` line) or for a second look at a part. The first step of the queue is the loop over all test files, and a narrowed loop does not replace it",
    ] {
        assert!(python.contains(want), "python.md lacks: {want}");
    }
    for want in [
        "A Python project with test files starts the same way, with one loop over all of them: `dla harness gen --lang python --pytest`",
        "`--tests a,b` for files or directories; no candidate, no target, nothing to fill",
        "goes on with the driver batches once it wrote a series or an attempt failed",
        "a loop narrowed with `--tests` does not count for that, only the loop over all test files does",
    ] {
        assert!(stages.contains(want), "stages.md lacks: {want}");
    }
    assert!(
        stages.contains(
            "a Python test loop lists the test files that ran and measures nothing per file"
        ),
        "stages.md: the bundle of a Python test loop"
    );
    assert!(
        skill.contains("Python tests: first dla harness gen --lang python --pytest"),
        "SKILL.md lacks the pytest loop in checklist item 4"
    );
    for want in [
        "`dla_pytest_loop.py`",
        "`-o addopts=`",
        "`DLA-PYTEST the tests import <name> from <path>, which is outside the project: ...`",
        "`dropped test file <path>: none of its tests passed in the first run (first failure: ...)`",
        "`the tests are too slow for a series`",
        "`DLA-PYTEST summary: units= tests_run= files= dropped= failed_units= first_failure=`",
    ] {
        assert!(
            readme.contains(want),
            "harness/python/README.md lacks: {want}"
        );
    }
    assert!(
        verifier.contains("`test_files_run`") && verifier.contains("`dropped_files`"),
        "agents/verifier.md does not say what a Python bundle lists"
    );
}

#[test]
fn the_worker_is_told_to_run_the_command_run_status_prints_and_the_python_loop_is_named() {
    let worker = fs::read_to_string(root().join("agents/worker.md")).unwrap();
    for want in [
        "run the command it prints for that queue (`next_command`, the same as `queue.<runtime>.command`), exactly as printed",
        "harness gen --lang python --pytest",
        "harness gen --lang python --pytest` without `--tests` (the project's own tests in a loop",
    ] {
        assert!(worker.contains(want), "worker.md lacks {want:?}");
    }
    assert!(
        !worker.contains(
            "run the batch command it prints (`dla --run <id> --worker <name> harness gen --lang <runtime> --driver --next`)"
        ),
        "worker.md still names the driver batch as the queue command"
    );
}

#[test]
fn the_python_test_loop_is_never_put_under_a_shell_timeout_and_a_registration_table_has_its_reading()
 {
    // Line breaks do not matter: every text is read with its whitespace collapsed.
    let read = |p: &str| {
        fs::read_to_string(root().join(p))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let python = read("skills/detect-leak-anywhere/references/python.md");
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    assert!(
        python.contains(
            "The loop runs up to its time limit of 1800 s. Run `dla harness run <H>` for it as a plain foreground command with the tool's longest timeout. When the tool moves the command to the background because it runs long, wait for the notice that it ended and do other read-only work meanwhile. Never start it with the shell's `&` or `nohup` and never put it under `timeout`: a shell background job is killed when the command returns, and a cut run writes no series."
        ),
        "python.md does not keep the loop out of the shell's timeout"
    );
    assert!(
        stages.contains(
            "The Python test loop runs up to its time limit of 1800 s. Run `dla harness run <H>` for it as a plain foreground command with the tool's longest timeout. When the tool moves the command to the background because it runs long, wait for the notice that it ended and do other read-only work meanwhile. Never start it with the shell's `&` or `nohup` and never put it under `timeout`: a shell background job is killed when the command returns, and a cut run writes no series."
        ),
        "stages.md does not keep the loop out of the shell's timeout"
    );
    let worker = read("agents/worker.md");
    for (name, text) in [("python.md", &python), ("stages.md", &stages)] {
        assert!(
            !text.contains("in the background and wait"),
            "{name} tells the agent to start the loop in the shell's background"
        );
    }
    assert!(
        worker.contains("as a plain foreground command with the tool's longest timeout")
            && worker.contains("never with the shell's `&`, `nohup` or `timeout`"),
        "worker.md does not say how the test loop is run"
    );
    assert!(
        python.contains(
            "is reported as grown by the registration function itself, not as reproduced through the public API; it is a leak only if run-time input reaches that call."
        ),
        "python.md does not describe the registration-table reading"
    );
    assert!(
        python.contains(
            "Any hook driver whose unit calls the registration function (or a wrapper the census recorded) gives that reading, and `dla ledger confirm` refuses such a finding with `registration-only`; one entry-path harness whose unit does not call it decides both readings"
        ),
        "python.md does not say that a direct finding cannot be confirmed"
    );
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    assert!(
        report.contains("(trigger `direct`; the fix gives one route, an entry-path harness whose unit does not call it, then `dla alloc-test`) | `registration-only` |"),
        "evidence-and-report.md does not list the registration-only refusal"
    );
}

#[test]
fn the_references_say_the_alloc_test_runs_on_a_harness_about_the_finding() {
    let read = |p: &str| {
        fs::read_to_string(root().join(p))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let python = read("skills/detect-leak-anywhere/references/python.md");
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    let verify = read("docs/VERIFY.md");
    assert!(
        python.contains(
            "The alloc-test runs on a harness that targets the finding or measured it, never on another one (`harness-not-about`)"
        ),
        "python.md lacks the rule"
    );
    assert!(
        stages.contains("The alloc-test runs on a harness that targets the finding, runs the project's own workload, or measured it")
            && stages.contains(
                "never on another one: the other is refused with `harness-not-about` before it runs"
            ),
        "stages.md lacks the rule"
    );
    assert!(
        report.contains("| `harness-not-about` |"),
        "evidence-and-report.md does not list the harness-not-about refusal"
    );
    assert!(
        verify.contains(
            "| An alloc-test on an unrelated harness refuted the finding at the leak line | corrected |"
        ),
        "VERIFY.md has no row for the alloc-test on an unrelated harness"
    );
}

#[test]
fn the_references_say_a_finding_with_a_leak_favoring_alloc_test_is_confirmed_or_refuted() {
    let read = |p: &str| {
        fs::read_to_string(root().join(p))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let report = read("skills/detect-leak-anywhere/references/evidence-and-report.md");
    let verify = read("docs/VERIFY.md");
    let sentence = "After an alloc-test that favors leak on a driver made for the finding (a hook driver that targets it), run the commands of its `next` in order (the refute of each open non-leak sibling, then the confirm with the growth evidence); a finding with such an alloc-test is confirmed or refuted, never closed unresolved (`leak-favored`). An alloc-test of the project's whole workload or of a test loop that favors leak does not isolate the finding: its next step is a narrower driver, and closing it unresolved stays possible.";
    assert!(stages.contains(sentence), "stages.md lacks the sentence");
    assert!(
        report.contains(sentence),
        "evidence-and-report.md lacks the sentence"
    );
    assert!(
        report.contains("| `leak-favored` |"),
        "evidence-and-report.md does not list the leak-favored refusal"
    );
    assert!(
        verify.contains(
            "| A finding whose alloc-test favored leak was closed as inconclusive | corrected |"
        ),
        "VERIFY.md has no row for the finding closed after a leak-favoring alloc-test"
    );
    // The code exists in the binary's refusal list.
    let error = fs::read_to_string(root().join("crates/dla-core/src/error.rs")).unwrap();
    assert!(
        error.contains("\"leak-favored\""),
        "error.rs lacks the code"
    );
}

#[test]
fn the_references_teach_who_holds_the_objects_of_a_growing_line() {
    // Line breaks do not matter: every text is read with its whitespace collapsed.
    let read = |p: &str| {
        fs::read_to_string(root().join(p))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let python = read("skills/detect-leak-anywhere/references/python.md");
    let stages = read("skills/detect-leak-anywhere/references/stages.md");
    let readme = read("harness/python/README.md");
    let verifier = read("agents/verifier.md");
    let verify = read("docs/VERIFY.md");
    for want in [
        "a `py.blocks@<file:line>` column says where the surviving objects were made, not who keeps them",
        "The route is a growing line, then who holds it, then one finding at the holder",
        "`dla harness run H --mode <m> --capture alloc,heap`",
        "(the trend says so in `holders_hint`)",
        "the two `dla summarize` commands that run prints (`alloc-stack-diff` for the stacks, `heap-graph` for the holders): one capture serves both",
        "a stack summary made before the finding was added counts for the finding once it exists in the bundle and for the capture offer",
        "`dla ledger label` counts a stack only by its stated ties",
        "`minor_sites` and tie nothing",
        "`first` is null for a file of one dump point",
        "the `py.type:<T>.count` columns of those types are the holder's too",
        "Every process runs with `PYTHONDONTWRITEBYTECODE=1` (a value the environment has is kept)",
        "`no_chain` (the command output says `no_chain_sites`)",
        "(`other_holders` says how many were left out; an allocation-stack record is cut the same way to the stacks at the finding's line, `other_stacks`)",
        "`ats` the distinct places that run names",
        "is never at a place outside the project",
        "`dla ledger add` warns when its `--owner` is a holder of a record",
        "an `unreferenced` entry does not, so the stack summary is still required for a native line",
        "a file outside the project is `<outside>/<name>`",
        "`static:<module>.<name>` is a module global or a class attribute (static/global)",
        "`stack:<function>@<file:line>` a function frame (blocked-stack)",
        "`path`, the chain with the most objects from a kept object to the holder",
        "it accounts for every line the holder keeps",
        "never add a finding per line",
        "`unreferenced`: no Python container refers to the blocks, so the test runner, pytest or the tracer do not hold them",
        "is explained by the extension function the line calls",
        "`heap` writes `py-1.retain.json` once, at the last dump point",
        "`py-retain` (`heap-graph`) ties a holder to the finding whose owner is the holder",
    ] {
        assert!(python.contains(want), "python.md lacks: {want}");
    }
    for want in [
        "`--capture histogram,alloc,nonheap,heap`",
        "a growing `py.blocks@` line is explained by `--capture alloc,heap`",
        "the one Python holder-chains file (`py-1.retain.json`: who keeps the objects of the growing lines)",
    ] {
        assert!(stages.contains(want), "stages.md lacks: {want}");
    }
    assert!(
        readme.contains("`dla summarize heap-graph` reads the file (`py-retain`)")
            && readme.contains("sets it to `1` when the run's consent allows raw values"),
        "harness/python/README.md does not say what reads the chains file"
    );
    // The verifier is told what the record means and what a block no Python object holds is not.
    for want in [
        "`index.json` lists the harnesses behind the evidence (`harnesses`)",
        "a driver made for this finding",
        "a driver with no target",
        "`minor_sites` (lines where it keeps only a small part, so it is not their holder)",
        "The bundle holds only the holders of this finding's line (`other_holders` counts the others)",
        "does not come from the test runner or from the measuring tools",
        "A `static:` holder is a module global or a class attribute",
        "a chain that ends at a module global or a class attribute names the holder",
        "A holder with `unreferenced` keeps blocks that no Python container refers to",
        "they are held from native code or a reference was lost there, so the answer is in the extension function the line calls",
        "the test runner, pytest or the tracer do not hold them, and N-OBSERVER does not explain them",
    ] {
        assert!(verifier.contains(want), "agents/verifier.md lacks: {want}");
    }
    assert!(
        verify.contains(
            "| A growing line said where objects were made and nothing said who keeps them | corrected |"
        ),
        "VERIFY.md has no row for the holder chains"
    );
}

#[test]
fn the_texts_say_older_harness_copies_are_thinned_and_come_back_on_use() {
    let stages =
        fs::read_to_string(root().join("skills/detect-leak-anywhere/references/stages.md"))
            .unwrap();
    for want in [
        "Older harnesses keep only the files of their copy that differ from the project (`work-thinned.json` lists the others), and a harness that is used again gets its files back first.",
        "`work-copy-stale` means the project changed after a harness was thinned: `dla harness gen` writes a harness on the project as it is now.",
    ] {
        assert!(stages.contains(want), "stages.md lacks: {want}");
    }
    assert!(
        !skill().contains("work-copy-stale"),
        "SKILL.md stays as it is: the stage text carries the rule"
    );
    let verify = fs::read_to_string(root().join("docs/VERIFY.md")).unwrap();
    assert!(
        verify.contains("| A run kept one full copy of the project per harness | corrected |"),
        "VERIFY.md has no row for the thinned harness copies"
    );
}

#[test]
fn the_rust_texts_say_a_feature_gating_code_inside_functions_needs_to_be_on() {
    let rust =
        fs::read_to_string(root().join("skills/detect-leak-anywhere/references/rust.md")).unwrap();
    for want in [
        "a crate whose type is a no-op unless its feature is on gates no `mod`, so a driver built without the feature never runs the code of the target.",
        "which repeats the `--workload`, `--no-default-features` and features of the generating call so that it runs as printed; `dla harness gen` prints it whole under `regenerate`, and its `_attention` line (one per feature) only says that the feature is off.",
        "A target that reads not executed leads with that command, and the queue keeps it open, whatever the number of drivers that missed it, until a harness that targeted it was built with the feature.",
    ] {
        assert!(rust.contains(want), "rust.md lacks: {want}");
    }
    let want = "the driver holds to Rust 1.70 and newer, writing an `unsafe extern` block only for a crate of edition 2024 or later; a failed build names the pinned channel.";
    assert!(rust.contains(want), "rust.md lacks: {want}");
    let readme = fs::read_to_string(root().join("harness/rust/README.md")).unwrap();
    for want in [
        "## Features that gate code inside functions",
        "`harness/H<n>/features.json` keeps the same for `dla trend`, the queue and `dla ledger close-rest`",
    ] {
        assert!(readme.contains(want), "README.md lacks: {want}");
    }
    assert!(
        !skill().contains("features.json"),
        "SKILL.md stays as it is: the Rust reference carries the rule"
    );
    let verify = fs::read_to_string(root().join("docs/VERIFY.md")).unwrap();
    assert!(
        verify.contains("| Code behind a feature the build left off was read as code that did not run | corrected |"),
        "VERIFY.md has no row for the feature-gated code"
    );
}

#[test]
fn the_node_texts_say_how_a_built_typescript_library_is_run_and_measured() {
    let node =
        fs::read_to_string(root().join("skills/detect-leak-anywhere/references/node.md")).unwrap();
    for want in [
        "the work copy of a git project holds the build output that git ignores when a `package.json`",
        "`node_modules` beside such a `package.json` is a symbolic link to the project's own",
        "declaration files (`*.d.ts`, `*.d.mts`, `*.d.cts`)",
        "are at the original position, `src/a/b.ts:139` instead of `lib/node/chunk-AB12.js:1076`",
        "the record keeps the built position in `built_sites`",
        "`dla ledger resolve <O> --as unresolved` is refused (`capture-not-summarized`)",
        "summary computed from capture files of that harness exists",
        "under 64 KiB asks for no capture",
    ] {
        assert!(node.contains(want), "node.md lacks: {want}");
    }
    let readme = fs::read_to_string(root().join("harness/node/README.md")).unwrap();
    for want in [
        "## A project that is built before it runs",
        "`build_output_kept` in the `harness gen` output",
        "node's own `module.SourceMap`",
        "so the records of a plain JavaScript project are the same as before",
    ] {
        assert!(
            readme.contains(want),
            "harness/node/README.md lacks: {want}"
        );
    }
}

#[test]
fn the_texts_say_a_function_that_ran_is_not_a_writer_line_that_ran() {
    let refs = root().join("skills/detect-leak-anywhere/references");
    let stages = fs::read_to_string(refs.join("stages.md")).unwrap();
    for want in [
        "`writer_not_executed` (its function ran and every line that adds to its holder has a count of 0 in the coverage run: Rust, Go and Node count lines) is a miss, not a reading",
        "a flat series says something about a holder only when the lines that add to it ran in that series",
        "a state that normal set-up removes (a full buffer, a missing configuration, an error path, a second thread) needs a unit or a set-up that does not remove it",
        "`remedy` shows each line with the lines above it that decide whether it runs",
        "`not_loaded`, `not_executed`, `writer_not_executed` or `unknown`)",
        "The JVM agent reports classes and methods only",
    ] {
        assert!(stages.contains(want), "stages.md lacks: {want}");
    }
    let rust = fs::read_to_string(refs.join("rust.md")).unwrap();
    for want in [
        "the count of the innermost code region that covers the line",
        "a line outside every region is unknown, never 0",
        "so does one whose writer lines ran fewer than one time per ten units of the coverage round",
    ] {
        assert!(rust.contains(want), "rust.md lacks: {want}");
    }
    let go = fs::read_to_string(refs.join("go.md")).unwrap();
    for want in [
        "the block of `p1-<mode>-cover.out` that covers the line and starts last gives its count",
        "(`-covermode` stays `set`, so a count is 0 or 1 and says whether the line ran at all)",
        "runs its function and never the append: the target reads `writer_not_executed`",
    ] {
        assert!(go.contains(want), "go.md lacks: {want}");
    }
    let node = fs::read_to_string(refs.join("node.md")).unwrap();
    for want in [
        "the innermost range of a function that has block coverage (`isBlockCoverage`) that covers the line, at the source line through the source map",
        "a function V8 gave no block ranges for leaves the line unknown",
    ] {
        assert!(node.contains(want), "node.md lacks: {want}");
    }
    let verify = fs::read_to_string(root().join("docs/VERIFY.md")).unwrap();
    for want in [
        "| A function that ran was read as a writer line that ran (a Rust library the tool had not been run on) |",
        "| A function that ran was read as a writer line that ran (a Go library the tool had not been run on) |",
    ] {
        assert!(verify.contains(want), "VERIFY.md lacks the row: {want}");
    }
}
