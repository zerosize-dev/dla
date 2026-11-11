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
        "from `grep -n` or `cat -n` output, never from counting an unnumbered listing",
    ] {
        assert!(nothing.contains(want), "When nothing fits lacks {want:?}");
    }
    let worker = fs::read_to_string(root().join("agents/worker.md")).unwrap();
    for want in [
        "UNCLASSIFIED only when no archetype fits the mechanism",
        "run the command the refusal names once, then label again",
        "label decision",
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
