//! One test per rule of bench/GRADING.md.

use dla_bench::{
    grade::{
        Finding, Source, State, VerifCounts, findings_from_json, findings_from_ledger,
        grade_findings, leak_outcomes, normalise_path, parse_site, read_ledger_bytes, round6,
        select_source, site_matches,
    },
    model::Expected,
};
use serde_json::json;

fn exp(toml_text: &str) -> Expected {
    toml::from_str(toml_text).expect("expected.toml parses")
}

const TRAP: &str = r#"
schema_version = 1
id = "t"
runtime = "python"
category = "archetype-leak"
pair = "t-ctl"
dynamic_confirmation = true
[task]
prompt = "p"
[[leak]]
id = "L1"
site = "src/cache.py:10"
lines = [8, 12]
alt_sites = ["src/hub.py:20-22", "src/hub.py:40"]
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
"#;

fn finding(state: State, site: Option<&str>) -> Finding {
    Finding {
        state,
        site: site.map(str::to_owned),
        owner: None,
        worker: None,
        holder: Some("static/global".into()),
        label: Some("L-COLLECT".into()),
        hypothesis: false,
        discriminator: false,
    }
}

// ---- source selection

#[test]
fn ledger_wins_when_it_has_a_finding_added() {
    let ledger = "{\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"site\":\"a:1\"}}\n";
    let fj = "{\"findings\":[{\"site\":\"b:2\",\"verdict\":\"leak-confirmed\"}]}";
    let (src, f) = select_source(Some(ledger), Some(fj));
    assert_eq!(src, Source::Ledger);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].site.as_deref(), Some("a:1"));
}

#[test]
fn ledger_without_finding_added_falls_back_to_findings_json() {
    let ledger = "{\"type\":\"evidence\"}\nnot json\n";
    let fj = "{\"findings\":[{\"site\":\"b:2\",\"verdict\":\"leak-confirmed\"}]}";
    let (src, f) = select_source(Some(ledger), Some(fj));
    assert_eq!(src, Source::Findings);
    assert_eq!(f[0].state, State::Confirmed);
}

#[test]
fn source_none_when_nothing_usable() {
    assert_eq!(select_source(None, None).0, Source::None);
    assert_eq!(select_source(Some(""), Some("[]")).0, Source::None);
    assert_eq!(
        select_source(None, Some("{\"findings\": {}}")).0,
        Source::None
    );
    assert_eq!(select_source(None, Some("not json")).0, Source::None);
    let (src, f) = select_source(None, Some("{\"findings\": []}"));
    assert_eq!(src, Source::Findings);
    assert!(f.is_empty());
}

// ---- lenient ledger replay

#[test]
fn ledger_replay_skips_bad_lines_and_ignores_other_records() {
    let text = [
        "",
        "garbage",
        "[1]",
        "{\"a\":1}",
        "{\"type\":3}",
        "{\"type\":\"evidence\",\"id\":\"E1\"}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"site\":\"a.py:3\",\"holder\":\"static/global\",\"archetype\":\"L-WORKER\"}}",
        "{\"type\":\"transition\",\"finding\":\"F1\",\"to\":\"confirmed\"}",
        "{\"type\":\"transition\",\"finding\":\"nope\",\"to\":\"strong\"}",
    ]
    .join("\n");
    let f = findings_from_ledger(&text).expect("has finding_added");
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].state, State::Confirmed);
    assert_eq!(f[0].site.as_deref(), Some("a.py:3"));
    assert_eq!(f[0].holder.as_deref(), Some("static/global"));
    assert_eq!(f[0].label.as_deref(), Some("L-WORKER"));
    assert!(!f[0].hypothesis);
    assert!(!f[0].discriminator);
}

#[test]
fn an_annotation_fills_a_missing_holder_or_site_and_never_overwrites() {
    let text = [
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"site\":\"a.c:29\"}}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"F2\",\"holder\":\"static/global\"}}",
        "{\"type\":\"finding_annotated\",\"finding\":\"F1\",\"holder\":\"native-handle/global-ref\",\"site\":\"b.c:1\"}",
        "{\"type\":\"finding_annotated\",\"finding\":\"F2\",\"holder\":\"cycle-only\",\"site\":\"c.c:2\"}",
        "{\"type\":\"finding_annotated\",\"finding\":\"F9\",\"holder\":\"cycle-only\"}",
        "{\"type\":\"finding_annotated\",\"finding\":\"F1\",\"holder\":5}",
    ]
    .join("\n");
    let f = findings_from_ledger(&text).expect("has finding_added");
    assert_eq!(f.len(), 2);
    assert_eq!(f[0].holder.as_deref(), Some("native-handle/global-ref"));
    assert_eq!(f[0].site.as_deref(), Some("a.c:29"));
    assert_eq!(f[1].holder.as_deref(), Some("static/global"));
    assert_eq!(f[1].site.as_deref(), Some("c.c:2"));
}

#[test]
fn an_annotation_that_replaced_a_holder_changes_it() {
    let text = [
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"holder\":\"container-field\"}}",
        "{\"type\":\"finding_annotated\",\"finding\":\"F1\",\"holder\":\"static/global\",\"evidence\":[\"E2\"],\"replaced_holder\":\"container-field\"}",
    ]
    .join("\n");
    let f = findings_from_ledger(&text).expect("has finding_added");
    assert_eq!(f[0].holder.as_deref(), Some("static/global"));
}

#[test]
fn a_label_record_sets_the_label_and_the_last_one_wins() {
    let text = [
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"F1\",\"archetype\":\"UNCLASSIFIED\"}}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"F2\",\"archetype\":\"UNCLASSIFIED\"}}",
        "{\"type\":\"finding_added\",\"worker\":\"jvm\",\"finding\":{\"id\":\"F1\"}}",
        "{\"type\":\"finding_labeled\",\"finding\":\"F1\",\"label\":\"L-COLLECT\",\"previous\":\"UNCLASSIFIED\"}",
        "{\"type\":\"finding_labeled\",\"finding\":\"F9\",\"label\":\"L-WORKER\"}",
        "{\"type\":\"finding_labeled\",\"finding\":\"F2\",\"label\":7}",
        "{\"type\":\"finding_labeled\",\"finding\":\"F2\",\"label\":\"L-HANDLE\"}",
        "{\"type\":\"finding_labeled\",\"finding\":\"F2\",\"label\":\"UNCLASSIFIED\",\"previous\":\"L-HANDLE\"}",
        "{\"type\":\"finding_labeled\",\"worker\":\"jvm\",\"finding\":\"F1\",\"label\":\"L-REGISTER\"}",
        "{\"type\":\"finding_labeled\",\"finding\":\"jvm/F1\",\"label\":\"L-TLS\"}",
        "{\"type\":\"finding_labeled\",\"finding\":\"F1\",\"label\":\"L-CYCLE\"}",
    ]
    .join("\n");
    let f = findings_from_ledger(&text).expect("has finding_added");
    assert_eq!(f.len(), 3);
    // F1 (no worker): the last record keyed to it, L-CYCLE.
    assert_eq!(f[0].label.as_deref(), Some("L-CYCLE"));
    // F2: a non-string label changes nothing, the later records win.
    assert_eq!(f[1].label.as_deref(), Some("UNCLASSIFIED"));
    // jvm/F1: the worker member and the prefixed id name one key; the later one wins.
    assert_eq!(f[2].label.as_deref(), Some("L-TLS"));
}

#[test]
fn ledger_state_mapping() {
    let mut lines = String::new();
    for (i, to) in [
        "confirmed",
        "strong",
        "refuted",
        "suspected_static_only",
        "unresolved",
        "under_test",
        "whatever",
    ]
    .iter()
    .enumerate()
    {
        lines.push_str(&format!(
            "{{\"type\":\"finding_added\",\"finding\":{{\"id\":\"F{i}\"}}}}\n{{\"type\":\"transition\",\"finding\":\"F{i}\",\"to\":\"{to}\"}}\n"
        ));
    }
    lines.push_str("{\"type\":\"finding_added\",\"finding\":{\"id\":\"G\"}}\n");
    lines.push_str("{\"type\":\"transition\",\"finding\":\"G\"}\n");
    let f = findings_from_ledger(&lines).unwrap();
    let states: Vec<State> = f.iter().map(|x| x.state).collect();
    assert_eq!(
        states,
        [
            State::Confirmed,
            State::Strong,
            State::Refuted,
            State::Suspected,
            State::Unresolved,
            State::Open,
            State::Open,
            State::Open
        ]
    );
    // discriminator is true exactly for refuted
    let disc: Vec<bool> = f.iter().map(|x| x.discriminator).collect();
    assert_eq!(
        disc,
        [false, false, true, false, false, false, false, false]
    );
}

#[test]
fn ledger_hypothesis_needs_a_non_empty_string_parent() {
    let text = [
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"A\",\"parent\":\"F1\"}}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"B\",\"parent\":\"\"}}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"C\",\"parent\":7}}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"D\"}}",
    ]
    .join("\n");
    let h: Vec<bool> = findings_from_ledger(&text)
        .unwrap()
        .iter()
        .map(|f| f.hypothesis)
        .collect();
    assert_eq!(h, [true, false, false, false]);
}

#[test]
fn ledger_fields_must_be_strings_else_null() {
    let text = "{\"type\":\"finding_added\",\"finding\":{\"id\":\"A\",\"site\":5,\"holder\":null,\"archetype\":[1]}}";
    let f = findings_from_ledger(text).unwrap();
    assert_eq!(
        (f[0].site.clone(), f[0].holder.clone(), f[0].label.clone()),
        (None, None, None)
    );
}

#[test]
fn repeated_id_moves_the_finding_to_the_later_line() {
    let text = [
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"A\",\"site\":\"one:1\"}}",
        "{\"type\":\"transition\",\"finding\":\"A\",\"to\":\"confirmed\"}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"B\",\"site\":\"two:2\"}}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":\"A\",\"site\":\"three:3\"}}",
    ]
    .join("\n");
    let f = findings_from_ledger(&text).unwrap();
    assert_eq!(f.len(), 2);
    assert_eq!(f[0].site.as_deref(), Some("two:2"));
    assert_eq!(f[1].site.as_deref(), Some("three:3"));
    assert_eq!(f[1].state, State::Open, "the replacement starts open");
}

#[test]
fn malformed_finding_added_creates_nothing_but_selects_the_ledger() {
    for line in [
        "{\"type\":\"finding_added\",\"finding\":5}",
        "{\"type\":\"finding_added\",\"finding\":{\"id\":7,\"site\":\"a:1\"}}",
        "{\"type\":\"finding_added\",\"finding\":{\"site\":\"a:1\"}}",
    ] {
        let (src, f) = select_source(
            Some(line),
            Some("{\"findings\":[{\"site\":\"b:2\",\"verdict\":\"leak-confirmed\"}]}"),
        );
        assert_eq!(src, Source::Ledger, "{line}");
        assert!(f.is_empty(), "{line}");
    }
}

#[test]
fn ledger_with_a_finding_added_but_no_usable_finding_is_still_the_source() {
    let (src, f) = select_source(
        Some("{\"type\":\"finding_added\"}\n"),
        Some("{\"findings\":[{\"site\":\"b:2\",\"verdict\":\"leak-confirmed\"}]}"),
    );
    assert_eq!(src, Source::Ledger);
    assert!(f.is_empty());
}

// ---- findings.json mapping

#[test]
fn findings_json_mapping() {
    let v = json!({"findings": [
        {"site": "a:1", "holder": "h", "verdict": "leak-confirmed", "cause": "L-COLLECT"},
        {"site": "a:2", "verdict": "leak-suspected", "cause": "l-collect"},
        {"site": "a:3", "verdict": "not-a-leak", "discriminator": "  flat "},
        {"site": "a:4", "verdict": "not-a-leak", "discriminator": " \t"},
        {"site": "a:5", "verdict": "leak-confirmed", "discriminator": "ignored"},
        {"site": 5, "holder": 6, "verdict": "other", "cause": "UNCLASSIFIED"},
        "skipped", 3, null,
        {"verdict": "not-a-leak", "discriminator": 4}
    ]});
    let f = findings_from_json(&v).unwrap();
    assert_eq!(f.len(), 7);
    assert_eq!(f[0].state, State::Confirmed);
    assert_eq!(f[0].label.as_deref(), Some("L-COLLECT"));
    assert_eq!(f[0].holder.as_deref(), Some("h"));
    assert_eq!(f[1].state, State::Suspected);
    assert_eq!(
        f[1].label, None,
        "lower case does not match the label pattern"
    );
    assert_eq!(f[2].state, State::Refuted);
    assert!(f[2].discriminator);
    assert!(!f[3].discriminator, "blank after trimming");
    assert!(!f[4].discriminator, "only not-a-leak counts");
    assert_eq!(
        (f[5].state, f[5].site.clone(), f[5].holder.clone()),
        (State::Open, None, None)
    );
    assert_eq!(f[5].label.as_deref(), Some("UNCLASSIFIED"));
    assert!(!f[6].discriminator, "non-string discriminator");
    assert!(f.iter().all(|x| !x.hypothesis));
    for label in ["N-WARM", "L-X", "UNCLASSIFIED"] {
        let v = json!({"findings": [{"cause": label}]});
        assert_eq!(
            findings_from_json(&v).unwrap()[0].label.as_deref(),
            Some(label)
        );
    }
    for bad in ["L-", "L-A1", "X-ABC", "L-ABC ", "free text"] {
        let v = json!({"findings": [{"cause": bad}]});
        assert_eq!(findings_from_json(&v).unwrap()[0].label, None, "{bad}");
    }
    assert!(findings_from_json(&json!([])).is_none());
    assert!(findings_from_json(&json!({"findings": null})).is_none());
}

// ---- site parsing and matching

#[test]
fn site_parsing_and_path_normalisation() {
    assert_eq!(parse_site("a/b.py:12"), Some(("a/b.py".into(), 12)));
    assert_eq!(parse_site("a/b.py:12:7"), Some(("a/b.py".into(), 12)));
    assert_eq!(parse_site(".\\.\\a\\b.py:3"), Some(("a/b.py".into(), 3)));
    assert_eq!(parse_site("./././a.py:3"), Some(("a.py".into(), 3)));
    assert_eq!(parse_site("C:\\w\\a.py:3"), Some(("C:/w/a.py".into(), 3)));
    assert_eq!(parse_site("a.py"), None);
    assert_eq!(parse_site("a.py:x"), None);
    assert_eq!(parse_site("a.py:3-5"), None);
    assert_eq!(normalise_path("x/./y"), "x/./y", "only leading ./ goes");
}

#[test]
fn site_matching_rules() {
    let e = exp(TRAP);
    let leak = &e.leak[0];
    let m = |s: &str| site_matches(&finding(State::Confirmed, Some(s)), leak);
    assert!(m("src/cache.py:10"));
    assert!(m("src/cache.py:8"), "range start is inside");
    assert!(m("src/cache.py:12"), "range end is inside");
    assert!(!m("src/cache.py:7"));
    assert!(!m("src/cache.py:13"));
    assert!(m("./src/cache.py:9"));
    assert!(m("src\\cache.py:9"));
    assert!(m("/work/proj/src/cache.py:9:4"), "suffix path with column");
    assert!(m("proj\\src\\cache.py:11"));
    assert!(!m("xsrc/cache.py:10"), "suffix must start at a slash");
    assert!(!m("cache.py:10"), "shorter than the leak path");
    assert!(!m("src/cache.py"));
    assert!(m("src/hub.py:20"), "alt range start");
    assert!(m("src/hub.py:22"), "alt range end");
    assert!(m("lib/src/hub.py:21"));
    assert!(!m("src/hub.py:23"));
    assert!(m("src/hub.py:40"), "alt single line");
    assert!(!m("src/hub.py:41"));
    assert!(!site_matches(&finding(State::Confirmed, None), leak));
}

#[test]
fn lines_default_to_the_site_line() {
    let e = exp(&TRAP.replace("lines = [8, 12]\n", ""));
    let leak = &e.leak[0];
    assert!(site_matches(
        &finding(State::Open, Some("src/cache.py:10")),
        leak
    ));
    assert!(!site_matches(
        &finding(State::Open, Some("src/cache.py:11")),
        leak
    ));
}

// ---- found, flagged, chosen

#[test]
fn found_flagged_and_chosen_finding() {
    let e = exp(TRAP);
    let mut hyp = finding(State::Confirmed, Some("src/cache.py:10"));
    hyp.hypothesis = true;
    let mut confirmed = finding(State::Confirmed, Some("src/cache.py:10"));
    confirmed.holder = Some("container-field".into());
    let mut strong = finding(State::Strong, Some("src/hub.py:21"));
    strong.label = Some("L-REGISTER".into());
    let o = &leak_outcomes(&e, &[hyp.clone(), confirmed.clone(), strong])[0];
    assert!(o.found && o.flagged);
    assert_eq!(
        o.chosen_holder.as_deref(),
        Some("static/global"),
        "strong beats an earlier confirmed"
    );
    assert_eq!(o.chosen_label.as_deref(), Some("L-REGISTER"));
    let o = &leak_outcomes(&e, &[confirmed.clone()])[0];
    assert_eq!(o.chosen_holder.as_deref(), Some("container-field"));
    // hypotheses never find or flag
    let o = &leak_outcomes(&e, &[hyp])[0];
    assert!(!o.found && !o.flagged);
    // flagged states
    for (state, flagged) in [
        (State::Suspected, true),
        (State::Unresolved, true),
        (State::Open, true),
        (State::Refuted, false),
    ] {
        let o = &leak_outcomes(&e, &[finding(state, Some("src/cache.py:10"))])[0];
        assert!(!o.found);
        assert_eq!(o.flagged, flagged, "{state:?}");
    }
}

// ---- metrics

const MULTI: &str = r#"
schema_version = 1
id = "m"
runtime = "node"
category = "multi-leak"
dynamic_confirmation = true
[task]
prompt = "p"
[[leak]]
id = "L1"
site = "a.js:5"
holder = "registration-table"
region = "heap"
cause = "L-REGISTER"
share = 0.7
[[leak]]
id = "L2"
site = "b.js:9"
holder = "static/global"
region = "heap"
cause = "UNCLASSIFIED"
share = 0.3
[[leak]]
id = "L3"
site = "c.js:1"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
confirmable = false
"#;

fn at(state: State, site: &str, holder: &str, label: Option<&str>) -> Finding {
    Finding {
        state,
        site: Some(site.into()),
        owner: None,
        worker: None,
        holder: Some(holder.into()),
        label: label.map(str::to_owned),
        hypothesis: false,
        discriminator: state == State::Refuted,
    }
}

#[test]
fn leak_set_excludes_unconfirmable_leaks() {
    let e = exp(MULTI);
    let g = grade_findings(
        &e,
        Source::Findings,
        &[at(State::Confirmed, "c.js:1", "static/global", None)],
    );
    assert_eq!(g.parts[0], (0, 2), "L3 is not in the leak set");
    assert_eq!(g.parts[2], (0, 2));
}

#[test]
fn recall_attribution_label_and_forced_fit_metrics() {
    let e = exp(MULTI);
    let g = grade_findings(
        &e,
        Source::Findings,
        &[
            at(State::Strong, "a.js:5", "static/global", Some("L-REGISTER")),
            at(
                State::Confirmed,
                "b.js:9",
                "static/global",
                Some("L-WORKER"),
            ),
        ],
    );
    assert_eq!(g.parts[0], (2, 2), "leak_recall");
    assert_eq!(g.parts[1], (1, 1), "archetype_free_recall");
    assert_eq!(g.parts[2], (2, 2), "flagged_recall");
    assert_eq!(g.parts[3], (1, 1), "second_leak_recall");
    assert_eq!(
        g.parts[4],
        (1, 2),
        "grounded_attribution: L1 holder is wrong"
    );
    assert_eq!(g.parts[5], (1, 1), "label_accuracy: L1 label is right");
    assert_eq!(g.parts[6], (1, 1), "forced_fit_rate: L2 got a label");
    // no label at all: excluded from both label metrics
    let g = grade_findings(
        &e,
        Source::Findings,
        &[
            at(State::Confirmed, "a.js:5", "registration-table", None),
            at(State::Confirmed, "b.js:9", "static/global", None),
        ],
    );
    assert_eq!(g.parts[5], (0, 0));
    assert_eq!(g.parts[6], (0, 0));
    // UNCLASSIFIED label on an UNCLASSIFIED leak is not forced
    let g = grade_findings(
        &e,
        Source::Findings,
        &[at(
            State::Confirmed,
            "b.js:9",
            "static/global",
            Some("UNCLASSIFIED"),
        )],
    );
    assert_eq!(g.parts[6], (0, 1));
}

#[test]
fn second_leak_recall_needs_a_found_leak() {
    let e = exp(MULTI);
    let none = grade_findings(&e, Source::None, &[]);
    assert_eq!(none.parts[3], (0, 0));
    let one = grade_findings(
        &e,
        Source::Findings,
        &[at(State::Confirmed, "b.js:9", "static/global", None)],
    );
    assert_eq!(one.parts[3], (0, 1));
    let both = grade_findings(
        &e,
        Source::Findings,
        &[
            at(State::Confirmed, "a.js:5", "registration-table", None),
            at(State::Confirmed, "b.js:9", "static/global", None),
        ],
    );
    assert_eq!(both.parts[3], (1, 1));
}

#[test]
fn claim_and_rule_out_definitions() {
    let c = finding(State::Confirmed, None);
    let s = finding(State::Strong, None);
    let mut h = finding(State::Confirmed, None);
    h.hypothesis = true;
    let susp = finding(State::Suspected, None);
    assert!(c.is_claim() && s.is_claim());
    assert!(!h.is_claim() && !susp.is_claim());
    let mut r = finding(State::Refuted, None);
    assert!(!r.is_rule_out());
    r.discriminator = true;
    assert!(r.is_rule_out());
    r.hypothesis = true;
    assert!(r.is_rule_out(), "hypotheses count as rule-outs");
    let mut not_refuted = finding(State::Confirmed, None);
    not_refuted.discriminator = true;
    assert!(!not_refuted.is_rule_out());
}

fn plain(cat: &str, extra: &str) -> Expected {
    exp(&format!(
        "schema_version = 1\nid = \"x\"\nruntime = \"go\"\ncategory = \"{cat}\"\ndynamic_confirmation = {}\n[task]\nprompt = \"p\"\n{extra}",
        cat != "impossible"
    ))
}

const NONLEAK: &str = "[[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n";

#[test]
fn run_level_rates() {
    let claim = [finding(State::Confirmed, Some("z:1"))];
    let ruled = [at(State::Refuted, "z:1", "x", None)];
    let mut blank = at(State::Refuted, "z:1", "x", None);
    blank.discriminator = false;
    let imp = plain(
        "impossible",
        "[[leak]]\nid=\"L1\"\nsite=\"a:1\"\nholder=\"static/global\"\nregion=\"heap\"\ncause=\"L-COLLECT\"\nconfirmable=false\n",
    );
    assert_eq!(
        grade_findings(&imp, Source::Findings, &claim).parts[7],
        (1, 1)
    );
    assert_eq!(
        grade_findings(&imp, Source::Findings, &ruled).parts[7],
        (0, 1)
    );
    let decoy = plain("decoy-control", NONLEAK);
    let g = grade_findings(&decoy, Source::Findings, &claim);
    assert_eq!(
        (g.parts[8], g.parts[11], g.parts[9]),
        ((1, 1), (1, 1), (0, 0))
    );
    let g = grade_findings(&decoy, Source::Findings, &ruled);
    assert_eq!((g.parts[8], g.parts[11]), ((0, 1), (0, 1)));
    let nl = plain("non-leak", NONLEAK);
    let g = grade_findings(&nl, Source::Findings, &claim);
    assert_eq!(
        (g.parts[8], g.parts[9], g.parts[11]),
        ((1, 1), (0, 1), (0, 0))
    );
    assert_eq!(
        grade_findings(&nl, Source::Findings, &ruled).parts[9],
        (1, 1)
    );
    assert_eq!(
        grade_findings(&nl, Source::Findings, &[blank]).parts[9],
        (0, 1)
    );
    assert_eq!(grade_findings(&nl, Source::None, &[]).parts[9], (0, 1));
}

#[test]
fn trap_miss_requires_a_pair_and_counts_missed_confirmable_leaks() {
    let with_pair = exp(TRAP);
    let hit = [at(
        State::Confirmed,
        "src/cache.py:10",
        "static/global",
        None,
    )];
    assert_eq!(
        grade_findings(&with_pair, Source::Findings, &hit).parts[10],
        (0, 1)
    );
    assert_eq!(
        grade_findings(&with_pair, Source::None, &[]).parts[10],
        (1, 1)
    );
    let sus = [at(
        State::Suspected,
        "src/cache.py:10",
        "static/global",
        None,
    )];
    assert_eq!(
        grade_findings(&with_pair, Source::Findings, &sus).parts[10],
        (1, 1)
    );
    let no_pair = exp(&TRAP.replace("pair = \"t-ctl\"\n", ""));
    assert_eq!(
        grade_findings(&no_pair, Source::None, &[]).parts[10],
        (0, 0)
    );
}

#[test]
fn metrics_of_other_categories_stay_zero() {
    let nl = plain("non-leak", NONLEAK);
    let g = grade_findings(&nl, Source::None, &[]);
    for i in [0, 1, 2, 3, 4, 5, 6, 7, 10] {
        assert_eq!(g.parts[i], (0, 0), "metric {i}");
    }
}

#[test]
fn round6_is_half_to_even_on_the_binary_value() {
    assert_eq!(
        round6(1.0 / 128.0),
        0.007812,
        "the 1/128 tie goes to the even digit"
    );
    assert_eq!(
        round6(3.0 / 128.0),
        0.023438,
        "0.0234375 ties up to the even digit"
    );
    assert_eq!(round6(7.0 / 3.0), 2.333333);
    assert_eq!(round6(2.0 / 3.0), 0.666667);
    assert_eq!(
        round6(0.0000005),
        0.0,
        "the binary value of 5e-7 lies just below the tie"
    );
    assert_eq!(
        round6(0.0000015),
        0.000002,
        "the binary value of 1.5e-6 lies just above the tie"
    );
    assert_eq!(round6(0.5), 0.5);
    assert_eq!(round6(1.0), 1.0);
}

// ---- clarified rules

#[test]
fn site_is_matched_against_the_whole_string() {
    assert_eq!(
        parse_site("a.py:3\n"),
        None,
        "a trailing newline does not match"
    );
    assert_eq!(parse_site("a.py:3:4\n"), None);
    assert_eq!(
        parse_site("a\nb.py:3"),
        None,
        "dot does not cross a newline"
    );
    assert_eq!(parse_site("a.py:3"), Some(("a.py".into(), 3)));
}

#[test]
fn expected_paths_are_used_exactly_as_written() {
    let e = exp(&TRAP
        .replace("src/cache.py:10", "./src/cache.py:10")
        .replace("src/hub.py:20-22", ".\\\\src/hub.py:20-22"));
    let leak = &e.leak[0];
    let m = |s: &str| site_matches(&finding(State::Confirmed, Some(s)), leak);
    assert!(
        !m("src/cache.py:10"),
        "the leading ./ of the expected path stays"
    );
    assert!(m("x/./src/cache.py:10"), "suffix rule on the literal path");
    assert!(!m("src/hub.py:21"), "backslash of the alt path stays");
    assert!(m("src/hub.py:40"), "other alt entries still work");
}

#[test]
fn malformed_alt_sites_are_ignored() {
    let e = exp(&TRAP.replace(
        r#"["src/hub.py:20-22", "src/hub.py:40"]"#,
        r#"["src/hub.py", "src/hub.py:x", "src/hub.py:5-", "src/hub.py:1-2-3", "src/hub.py:5:6", "src/hub.py:40"]"#,
    ));
    let leak = &e.leak[0];
    let m = |s: &str| site_matches(&finding(State::Confirmed, Some(s)), leak);
    assert!(m("src/hub.py:40"));
    assert!(!m("src/hub.py:5"));
    assert!(!m("src/hub.py:1"));
    assert!(!m("src/hub.py:2"));
    assert!(!m("src/hub.py:6"));
}

#[test]
fn chosen_finding_is_never_a_hypothesis() {
    let e = exp(TRAP);
    let mut hyp = finding(State::Strong, Some("src/cache.py:10"));
    hyp.hypothesis = true;
    hyp.holder = Some("thread-local".into());
    let mut plain = finding(State::Confirmed, Some("src/cache.py:10"));
    plain.holder = Some("container-field".into());
    let o = &leak_outcomes(&e, &[hyp.clone(), plain.clone()])[0];
    assert!(o.found);
    assert_eq!(o.chosen_holder.as_deref(), Some("container-field"));
    let mut plain_strong = plain.clone();
    plain_strong.state = State::Strong;
    plain_strong.holder = Some("static/global".into());
    let o = &leak_outcomes(&e, &[plain, hyp, plain_strong])[0];
    assert_eq!(o.chosen_holder.as_deref(), Some("static/global"));
}

#[test]
fn confirmable_defaults_to_true() {
    let e = exp(TRAP);
    assert!(e.leak[0].confirmable);
    let e = exp(&TRAP.replace(
        "cause = \"L-COLLECT\"",
        "cause = \"L-COLLECT\"\nconfirmable = false",
    ));
    assert!(!e.leak[0].confirmable);
}

#[test]
fn dump_backed_defaults_to_false_and_does_not_change_the_grade() {
    let e = exp(TRAP);
    assert!(!e.leak[0].dump_backed);
    let marked = exp(&TRAP.replace(
        "cause = \"L-COLLECT\"",
        "cause = \"L-COLLECT\"\ndump_backed = true",
    ));
    assert!(marked.leak[0].dump_backed);
    let findings = [at(
        State::Confirmed,
        "src/cache.py:10",
        "static/global",
        None,
    )];
    let a = grade_findings(&e, Source::Ledger, &findings);
    let b = grade_findings(&marked, Source::Ledger, &findings);
    assert_eq!(a.leaks, b.leaks);
    assert_eq!(a.claim, b.claim);
}

#[test]
fn second_leak_recall_counts_only_the_leak_set() {
    let e = exp(r#"
schema_version = 1
id = "m"
runtime = "node"
category = "multi-leak"
dynamic_confirmation = true
[task]
prompt = "p"
[[leak]]
id = "L1"
site = "a.js:1"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
[[leak]]
id = "L2"
site = "b.js:1"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
[[leak]]
id = "L3"
site = "c.js:1"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
confirmable = false
"#);
    // leak set = L1, L2; finding L1 and the unconfirmable L3: found in set = 1
    let fs = [
        at(State::Confirmed, "a.js:1", "static/global", None),
        at(State::Confirmed, "c.js:1", "static/global", None),
    ];
    let g = grade_findings(&e, Source::Findings, &fs);
    assert_eq!(g.parts[3], (0, 1));
    assert_eq!(g.parts[0], (1, 2));
    // only the unconfirmable leak found: no leak of the set found, metric stays empty
    let fs = [at(State::Confirmed, "c.js:1", "static/global", None)];
    let g = grade_findings(&e, Source::Findings, &fs);
    assert_eq!(g.parts[3], (0, 0));
}

// ---- M4 rules: owner matching, workers, merged findings, artifact entry

const ARTIFACT: &str = r#"
schema_version = 1
id = "a"
runtime = "jvm"
entry = "artifact"
category = "archetype-leak"
dynamic_confirmation = true
[task]
prompt = "p"
[[leak]]
id = "L1"
owner = "quotes.RfqDesk.OPEN"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
"#;

fn owned(state: State, owner: Option<&str>, site: Option<&str>) -> Finding {
    Finding {
        owner: owner.map(str::to_owned),
        ..finding(state, site)
    }
}

#[test]
fn a_leak_without_site_matches_by_owner() {
    let e = exp(ARTIFACT);
    let leak = &e.leak[0];
    let m =
        |o: Option<&str>, site: Option<&str>| site_matches(&owned(State::Confirmed, o, site), leak);
    assert!(m(Some("quotes.RfqDesk.OPEN"), None));
    assert!(m(Some("app.quotes.RfqDesk.OPEN"), None));
    assert!(m(Some("  quotes.RfqDesk.OPEN\n"), None), "trimmed");
    assert!(
        m(Some("quotes.RfqDesk.OPEN"), Some("x.java:1")),
        "the site plays no part"
    );
    assert!(
        !m(Some("appquotes.RfqDesk.OPEN"), None),
        "needs a dot boundary"
    );
    assert!(!m(Some("RfqDesk.OPEN"), None), "shorter is not a match");
    assert!(!m(Some("quotes.RfqDesk.OPEN.next"), None));
    assert!(!m(Some("quotes.rfqdesk.open"), None), "case-sensitive");
    assert!(
        !m(None, Some("quotes/RfqDesk.java:12")),
        "no owner never matches"
    );
}

#[test]
fn an_owner_on_a_leak_with_a_site_is_not_used() {
    let e = exp(&TRAP.replace(
        "cause = \"L-COLLECT\"",
        "cause = \"L-COLLECT\"\nowner = \"a.B.c\"",
    ));
    let f = owned(State::Confirmed, Some("a.B.c"), None);
    assert!(!site_matches(&f, &e.leak[0]));
    assert!(site_matches(
        &owned(State::Confirmed, None, Some("src/cache.py:9")),
        &e.leak[0]
    ));
}

#[test]
fn owner_flows_from_added_annotated_and_findings_json() {
    let text = [
        r#"{"type":"finding_added","finding":{"id":"F1","owner":"a.B.c"}}"#,
        r#"{"type":"finding_added","finding":{"id":"F2"}}"#,
        r#"{"type":"finding_added","finding":{"id":"F3","owner":7}}"#,
        r#"{"type":"finding_annotated","finding":"F1","owner":"x.Y.z"}"#,
        r#"{"type":"finding_annotated","finding":"F2","owner":"x.Y.z"}"#,
    ]
    .join("\n");
    let f = findings_from_ledger(&text).unwrap();
    assert_eq!(f[0].owner.as_deref(), Some("a.B.c"));
    assert_eq!(f[1].owner.as_deref(), Some("x.Y.z"));
    assert_eq!(f[2].owner, None);
    let j = findings_from_json(&json!({"findings": [{"owner": "a.B.c"}, {"owner": 3}]})).unwrap();
    assert_eq!(j[0].owner.as_deref(), Some("a.B.c"));
    assert_eq!(j[1].owner, None);
    assert!(j.iter().all(|f| f.worker.is_none()));
}

#[test]
fn worker_prefixes_and_worker_fields_key_the_findings() {
    let text = [
        // The same bare id from two workers is two findings.
        r#"{"type":"finding_added","worker":"jvm","finding":{"id":"F1","site":"a.kt:1"}}"#,
        r#"{"type":"finding_added","worker":"rust","finding":{"id":"F1","site":"b.rs:2"}}"#,
        // A prefixed id keeps its own prefix whatever the line says.
        r#"{"type":"finding_added","worker":"rust","finding":{"id":"jvm/F2","site":"c.kt:3"}}"#,
        r#"{"type":"finding_added","finding":{"id":"F1","site":"core.txt:4"}}"#,
        r#"{"type":"transition","worker":"rust","finding":"F1","to":"confirmed"}"#,
        r#"{"type":"transition","finding":"jvm/F2","to":"strong"}"#,
        r#"{"type":"transition","finding":"F1","to":"refuted"}"#,
        r#"{"type":"finding_added","finding":{"id":"/F9"}}"#,
    ]
    .join("\n");
    let f = findings_from_ledger(&text).unwrap();
    let view: Vec<(Option<&str>, &str, State)> = f
        .iter()
        .map(|x| {
            (
                x.worker.as_deref(),
                x.site.as_deref().unwrap_or("-"),
                x.state,
            )
        })
        .collect();
    assert_eq!(
        view,
        [
            (Some("jvm"), "a.kt:1", State::Open),
            (Some("rust"), "b.rs:2", State::Confirmed),
            (Some("jvm"), "c.kt:3", State::Strong),
            (None, "core.txt:4", State::Refuted),
            (None, "-", State::Open),
        ]
    );
}

#[test]
fn a_merged_duplicate_is_removed_and_the_canonical_keeps_its_state() {
    let base = [
        r#"{"type":"finding_added","finding":{"id":"jvm/F1","site":"a.kt:1"}}"#,
        r#"{"type":"finding_added","finding":{"id":"rust/F1","site":"a.kt:1"}}"#,
        r#"{"type":"transition","finding":"jvm/F1","to":"suspected_static_only"}"#,
        r#"{"type":"transition","finding":"rust/F1","to":"confirmed"}"#,
    ];
    let with = |extra: &str| {
        let mut v = base.to_vec();
        v.push(extra);
        findings_from_ledger(&v.join("\n")).unwrap()
    };
    let merged = with(r#"{"type":"finding_merged","canonical":"jvm/F1","duplicate":"rust/F1"}"#);
    assert_eq!(merged.len(), 1);
    assert_eq!(
        merged[0].state,
        State::Suspected,
        "nothing is copied to the canonical"
    );
    assert_eq!(merged[0].worker.as_deref(), Some("jvm"));
    // A worker field qualifies bare ids in the record.
    let bare =
        with(r#"{"type":"finding_merged","worker":"rust","canonical":"jvm/F1","duplicate":"F1"}"#);
    assert_eq!(bare.len(), 1);
    for ignored in [
        r#"{"type":"finding_merged","canonical":"jvm/F1","duplicate":"jvm/F1"}"#,
        r#"{"type":"finding_merged","canonical":"jvm/F1","duplicate":"nope/F1"}"#,
        r#"{"type":"finding_merged","canonical":"nope/F1","duplicate":"jvm/F1"}"#,
        r#"{"type":"finding_merged","canonical":"jvm/F1"}"#,
        r#"{"type":"finding_merged","canonical":1,"duplicate":2}"#,
    ] {
        assert_eq!(with(ignored).len(), 2, "{ignored}");
    }
    // The duplicate can be added again later.
    let mut v = base.to_vec();
    v.push(r#"{"type":"finding_merged","canonical":"jvm/F1","duplicate":"rust/F1"}"#);
    v.push(r#"{"type":"finding_added","finding":{"id":"rust/F1","site":"b.kt:2"}}"#);
    assert_eq!(findings_from_ledger(&v.join("\n")).unwrap().len(), 2);
}

#[test]
fn a_merge_removes_the_claim_that_only_the_duplicate_made() {
    let e = exp(TRAP);
    let lines = [
        r#"{"type":"finding_added","finding":{"id":"jvm/F1","site":"x.py:1"}}"#,
        r#"{"type":"finding_added","finding":{"id":"rust/F1","site":"src/cache.py:10"}}"#,
        r#"{"type":"transition","finding":"rust/F1","to":"confirmed"}"#,
    ];
    let plain = findings_from_ledger(&lines.join("\n")).unwrap();
    assert!(grade_findings(&e, Source::Ledger, &plain).leaks[0].found);
    let mut v = lines.to_vec();
    v.push(r#"{"type":"finding_merged","canonical":"jvm/F1","duplicate":"rust/F1"}"#);
    let merged = findings_from_ledger(&v.join("\n")).unwrap();
    let g = grade_findings(&e, Source::Ledger, &merged);
    assert!(!g.leaks[0].found && !g.claim);
}

#[test]
fn verification_and_slice_records_change_nothing() {
    let text = [
        r#"{"type":"slice_started","worker":"jvm","runtime":"jvm","base_seq":3}"#,
        r#"{"type":"finding_added","finding":{"id":"F1","site":"a.py:1"}}"#,
        r#"{"type":"verification","finding":"F1","to":"confirmed","result":"agrees"}"#,
        r#"{"type":"slice_done","worker":"jvm","open":0}"#,
        r#"{"type":"slice_merged","worker":"jvm","lines":4}"#,
    ]
    .join("\n");
    let f = findings_from_ledger(&text).unwrap();
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].state, State::Open);
}

#[test]
fn the_chosen_finding_carries_the_worker() {
    let e = exp(TRAP);
    let text = [
        r#"{"type":"finding_added","finding":{"id":"jvm/F1","site":"src/cache.py:10"}}"#,
        r#"{"type":"finding_added","finding":{"id":"rust/F1","site":"src/cache.py:10"}}"#,
        r#"{"type":"transition","finding":"jvm/F1","to":"confirmed"}"#,
        r#"{"type":"transition","finding":"rust/F1","to":"strong"}"#,
    ]
    .join("\n");
    let f = findings_from_ledger(&text).unwrap();
    let o = &leak_outcomes(&e, &f)[0];
    assert_eq!(
        o.chosen_worker.as_deref(),
        Some("rust"),
        "strong beats confirmed"
    );
}

#[test]
fn source_overclaim_counts_artifact_runs_with_a_claim_that_names_a_site() {
    let art = exp(ARTIFACT);
    let part = |f: Vec<Finding>| grade_findings(&art, Source::Ledger, &f).parts[12];
    let claim = |site: Option<&str>| owned(State::Confirmed, Some("quotes.RfqDesk.OPEN"), site);
    assert_eq!(part(vec![]), (0, 1), "every artifact run counts");
    assert_eq!(part(vec![claim(None)]), (0, 1));
    assert_eq!(
        part(vec![claim(Some("   "))]),
        (0, 1),
        "a blank site is no site"
    );
    assert_eq!(part(vec![claim(Some("a/B.java:3"))]), (1, 1));
    let mut strong = claim(Some("a/B.java:3"));
    strong.state = State::Strong;
    assert_eq!(part(vec![strong]), (1, 1));
    let mut suspected = claim(Some("a/B.java:3"));
    suspected.state = State::Suspected;
    assert_eq!(part(vec![suspected]), (0, 1), "only claims count");
    let mut hypothesis = claim(Some("a/B.java:3"));
    hypothesis.hypothesis = true;
    assert_eq!(part(vec![hypothesis]), (0, 1));
    // A repository run never counts, whatever it claims.
    let repo = exp(TRAP);
    let p = grade_findings(
        &repo,
        Source::Ledger,
        &[finding(State::Confirmed, Some("src/cache.py:10"))],
    );
    assert_eq!(p.parts[12], (0, 0));
}

// ---- verification metrics

fn counts(lines: &[&str]) -> VerifCounts {
    VerifCounts::of(&read_ledger_bytes(lines.join("\n").as_bytes()).unwrap())
}

const ADDED: &str = r#"{"type":"finding_added","finding":{"id":"F1","site":"a.py:1"}}"#;
const CONFIRMED: &str = r#"{"type":"transition","finding":"F1","to":"confirmed"}"#;

fn ver(finding: &str, site: bool, holder: bool, label: &str) -> String {
    format!(
        r#"{{"type":"verification","finding":"{finding}","holder":"static/global","mechanism":"m","label":"UNCLASSIFIED","bundle":"aa","agrees_site":{site},"agrees_holder":{holder}{label}}}"#
    )
}

#[test]
fn a_claim_with_a_verification_counts_its_agreements() {
    let v = ver("F1", true, false, r#","agrees_label":true"#);
    let c = counts(&[ADDED, CONFIRMED, &v]);
    assert_eq!(
        c,
        VerifCounts {
            records: 1,
            claims: 1,
            verified: 1,
            site: 1,
            holder: 0,
            label: 1,
            label_den: 1
        }
    );
}

#[test]
fn a_verification_without_a_label_verdict_leaves_the_label_denominator() {
    for label in ["", r#","agrees_label":null"#] {
        let v = ver("F1", true, true, label);
        let c = counts(&[ADDED, CONFIRMED, &v]);
        assert_eq!((c.verified, c.label, c.label_den), (1, 0, 0), "{label}");
    }
    // A label verdict of another type is no verification at all.
    let v = ver("F1", true, true, r#","agrees_label":"yes""#);
    let c = counts(&[ADDED, CONFIRMED, &v]);
    assert_eq!((c.records, c.verified), (0, 0));
}

#[test]
fn the_last_verification_of_a_finding_wins() {
    let first = ver("F1", false, false, r#","agrees_label":false"#);
    let second = ver("F1", true, true, "");
    let c = counts(&[ADDED, CONFIRMED, &first, &second]);
    assert_eq!((c.records, c.verified, c.site, c.holder), (1, 1, 1, 1));
    assert_eq!(c.label_den, 0, "the later record has no label verdict");
}

#[test]
fn only_confirmed_or_strong_findings_without_a_parent_are_claims() {
    let refuted = r#"{"type":"finding_added","finding":{"id":"F2","site":"b.py:1"}}"#;
    let refuted_to = r#"{"type":"transition","finding":"F2","to":"refuted"}"#;
    let hyp = r#"{"type":"finding_added","finding":{"id":"F1.n1","parent":"F1"}}"#;
    let hyp_to = r#"{"type":"transition","finding":"F1.n1","to":"confirmed"}"#;
    let strong = r#"{"type":"finding_added","finding":{"id":"F3","site":"c.py:1"}}"#;
    let strong_to = r#"{"type":"transition","finding":"F3","to":"strong"}"#;
    let v: Vec<String> = ["F1", "F2", "F1.n1", "F3"]
        .iter()
        .map(|f| ver(f, true, true, ""))
        .collect();
    let mut lines = vec![
        ADDED, CONFIRMED, refuted, refuted_to, hyp, hyp_to, strong, strong_to,
    ];
    lines.extend(v.iter().map(String::as_str));
    let c = counts(&lines);
    assert_eq!((c.records, c.claims, c.verified), (4, 2, 2));
}

#[test]
fn old_shapes_unknown_findings_and_workers_are_handled() {
    // The pre-schema-3 form has no agreement flags: not a verification.
    let old = r#"{"type":"verification","finding":"F1","to":"confirmed","result":"agrees"}"#;
    let c = counts(&[ADDED, CONFIRMED, old]);
    assert_eq!((c.records, c.claims, c.verified), (0, 1, 0));
    // A finding the ledger does not have.
    let other = ver("F9", true, true, "");
    assert_eq!(counts(&[ADDED, CONFIRMED, &other]).records, 0);
    // A worker line keys its finding as <worker>/<id>.
    let wadd = r#"{"type":"finding_added","worker":"jvm","finding":{"id":"F1","site":"a.py:1"}}"#;
    let wtr = r#"{"type":"transition","worker":"jvm","finding":"F1","to":"confirmed"}"#;
    let wver = r#"{"type":"verification","worker":"jvm","finding":"F1","holder":"static/global","mechanism":"m","label":"UNCLASSIFIED","bundle":"aa","agrees_site":true,"agrees_holder":true}"#;
    let c = counts(&[wadd, wtr, wver]);
    assert_eq!((c.records, c.claims, c.verified), (1, 1, 1));
    // A duplicate that was merged away takes its verification with it.
    let wadd2 = r#"{"type":"finding_added","worker":"rust","finding":{"id":"F1","site":"a.py:1"}}"#;
    let merged = r#"{"type":"finding_merged","canonical":"jvm/F1","duplicate":"rust/F1"}"#;
    let wver2 = wver.replace("jvm", "rust");
    let c = counts(&[wadd, wtr, wadd2, &wver2, merged]);
    assert_eq!((c.records, c.verified), (0, 0));
}
