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
                "`budget_left_unrevisited`",
                "`next_command`",
                "reads as non-leak:",
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
            ],
        ),
        (
            "jvm.md",
            &jvm,
            vec![
                "never run `gradle`, `mvn` or",
                "`java -cp` to print or assemble one",
                "`build_fails`",
            ],
        ),
        (
            "agents/worker.md",
            &worker,
            vec![
                "Measure before you close",
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
