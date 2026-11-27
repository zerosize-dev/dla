//! Growth measured over dumps is a lead to decide, like a passed trend column. A summarizer's
//! dump series (`dla summarize heap-graph`) has no trend and no harness in its file, so the
//! growth it measured reaches the run only through its evidence: a growth record whose passed
//! column names a finding, and a `dump` discriminator that favors leak. `close-rest` leaves such
//! a finding open, `ledger resolve` refuses to close it, the final gate lists it (open, or
//! closed `unresolved` by an older close), and the commands it prints are accepted as printed.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, HarnessManifest, Ledger, NewEvidence, Source, Tool,
    evidence::EvidenceCoverage,
};
use serde_json::{Value, json};

fn command(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1,2,3");
    c
}

struct R {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

impl R {
    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        (
            o.status.code().unwrap(),
            serde_json::from_str(&text).unwrap_or(Value::Null),
        )
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        v
    }
    fn final_gate(&self) -> Vec<Value> {
        let _ = self.run(&["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }
    /// The state the ledger holds for `id`.
    fn state(&self, id: &str) -> String {
        let l = Ledger::load(&self.run_dir).unwrap();
        l.finding(&id.parse().unwrap()).unwrap().state.to_string()
    }
}

fn evidence(
    authority: Authority,
    kind: EvidenceKind,
    path: Option<&str>,
    result: Value,
) -> NewEvidence {
    NewEvidence {
        source: match path {
            Some(p) => Source::Artifact {
                path: p.into(),
                sha256: "0".repeat(64),
            },
            None => Source::Repo {
                commit: "c".into(),
                path: None,
            },
        },
        tool: Tool {
            name: "t".into(),
            version: "1".into(),
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

fn add(r: &R, e: NewEvidence) -> String {
    let mut l = Ledger::load(&r.run_dir).unwrap();
    l.add_evidence(e).unwrap();
    l.evidence().last().unwrap().evidence_id.to_string()
}

/// A Node run with one candidate F1 (`app.Conn.request`, at `src/conn.js:3`), harness H1 that
/// ran (workload evidence) and nothing else: no trend, and no series of harness H1.
fn scene() -> (R, String) {
    scene_with(Some("src/conn.js:3"))
}

/// [`scene`] whose F1 has the given site, or none.
fn scene_with(site: Option<&str>) -> (R, String) {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("src/conn.js"),
        "const requests = [];\nclass Conn {\n  request(x) { requests.push(x); }\n}\nmodule.exports = { Conn };\n",
    )
    .unwrap();
    let o = command(dir.path())
        .args(["run", "init", "--session-id", "t", "--budget", "3h"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
    let r = R { dir, run_dir };
    let m: HarnessManifest = serde_json::from_value(json!({
        "id": "H1", "runtime": "node", "mode": "driver",
        "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": [], "agent": "driver-node", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&r.run_dir).unwrap();
    let workload = add(
        &r,
        evidence(
            Authority::Test,
            EvidenceKind::Workload,
            None,
            json!({"harness": "H1", "mode": "driver", "runtime": "node", "series": []}),
        ),
    );
    // The finding, raised from a growth record of the run.
    let seed = add(
        &r,
        evidence(
            Authority::Dump,
            EvidenceKind::GrowthSeries,
            None,
            json!({"series": "S1", "axis": "dump", "n": 3, "gate": "pass", "passed": []}),
        ),
    );
    let mut args = vec![
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &seed,
        "--holder",
        "static/global",
        "--owner",
        "app.Conn.request",
    ];
    if let Some(site) = &site {
        args.extend(["--site", site]);
    }
    let added = r.ok(&args);
    assert_eq!(added["id"], "F1", "{added}");
    (r, workload)
}

/// What a `dla summarize heap-graph` over three snapshots of harness H1 wrote for series S4:
/// the series file, a retention record per snapshot (read from the capture files of H1), the
/// growth record whose passed column names F1 and, when `favors` is given, the discriminator.
/// Returns (growth evidence, discriminator evidence).
fn dump_series(r: &R, favors: Option<&str>) -> (String, Option<String>) {
    fs::create_dir_all(r.run_dir.join("series")).unwrap();
    fs::write(
        r.run_dir.join("series/S4.json"),
        json!({"schema_version": 1, "id": "S4",
            "source": {"kind": "dump", "command": "dla summarize heap-graph"},
            "axis": "dump", "warmup": 0, "columns": [], "points": [], "events": [],
            "limitations": []})
        .to_string(),
    )
    .unwrap();
    for i in 1..=3 {
        add(
            r,
            evidence(
                Authority::Dump,
                EvidenceKind::RetentionPath,
                Some(&format!(
                    "harness/H1/out/p{i}/capture/7/heap-{i}.heapsnapshot"
                )),
                json!({"holders": []}),
            ),
        );
    }
    let growth = add(
        r,
        evidence(
            Authority::Dump,
            EvidenceKind::GrowthSeries,
            None,
            json!({"series": "S4", "axis": "dump", "n": 3, "gate": "pass",
                "passed": [{"column": "holder:static:app.Conn.request", "finding": "F1",
                    "candidates": ["F1"]}]}),
        ),
    );
    let disc = favors.map(|f| discriminator(r, f));
    (growth, disc)
}

fn discriminator(r: &R, favors: &str) -> String {
    add(
        r,
        evidence(
            Authority::Dump,
            EvidenceKind::Discriminator,
            None,
            json!({"test": "dump-plateau", "favors": favors, "supports": ["leak"],
                "finding": "F1", "column": "holder:static:app.Conn.request", "series": "S4",
                "reason": "retained bytes and count grew over every interval"}),
        ),
    )
}

/// The words of a printed command, quotes honored, its comment (`   (..)`, `   #..`) dropped
/// and its placeholders filled.
fn words_of(printed: &str) -> Vec<String> {
    let cmd = printed
        .split("   #")
        .next()
        .unwrap()
        .split("   (")
        .next()
        .unwrap()
        .trim();
    let mut out: Vec<String> = Vec::new();
    let (mut cur, mut quoted, mut any) = (String::new(), false, false);
    for ch in cmd.chars() {
        match ch {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            c if c.is_whitespace() && !quoted => {
                if any {
                    out.push(std::mem::take(&mut cur));
                    any = false;
                }
            }
            c => {
                cur.push(c);
                any = true;
            }
        }
    }
    if any {
        out.push(cur);
    }
    out.into_iter()
        .map(|w| match w.as_str() {
            "<holder>" => "static/global".to_owned(),
            "<file:line>" => "src/conn.js:3".to_owned(),
            _ => w,
        })
        .skip(1)
        .collect()
}

/// Runs the commands of `fix` in order, each as printed. The sibling the add creates is `F1.n1`.
fn run_fix(r: &R, fix: &str) -> usize {
    let steps: Vec<&str> = fix.split("   then   ").collect();
    for s in &steps {
        let w = words_of(s);
        let args: Vec<&str> = w.iter().map(String::as_str).collect();
        r.ok(&args);
    }
    steps.len()
}

fn close(r: &R, id: &str, as_: &str) -> (i32, Value) {
    r.run(&[
        "ledger",
        "resolve",
        id,
        "--as",
        as_,
        "--reason",
        "inconclusive",
        "--remedy",
        "x",
    ])
}

fn left_of<'a>(out: &'a Value, id: &str) -> Option<&'a str> {
    out["left"]
        .as_array()?
        .iter()
        .find(|l| l["id"] == id)
        .and_then(|l| l["why"].as_str())
}

#[test]
fn close_rest_leaves_a_finding_whose_dump_growth_nothing_decided() {
    let (r, _w) = scene();
    let (growth, d) = dump_series(&r, Some("leak"));
    let d = d.unwrap();
    let out = r.ok(&["ledger", "close-rest"]);
    assert_eq!(out["closed"], 0, "{out}");
    let why = left_of(&out, "F1").unwrap_or_else(|| panic!("F1 not left open: {out}"));
    assert!(why.contains("S4"), "{why}");
    assert!(why.contains("grew over every dump"), "{why}");
    assert!(why.contains("dla summarize heap-graph"), "{why}");
    assert!(
        why.contains(&format!("dla ledger confirm F1 --evidence {growth}")),
        "{why}"
    );
    assert!(why.contains(&d), "{why}");
    assert_eq!(r.state("F1"), "candidate");
    // Closing it by hand is refused with the same commands, whatever the way it is closed.
    for as_ in ["unresolved", "suspected_static_only"] {
        let (code, v) = close(&r, "F1", as_);
        assert_ne!(code, 0, "{as_}: {v}");
        assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
        let msg = v["error"]["message"].as_str().unwrap();
        assert!(
            msg.contains("S4") && msg.contains("grew over every dump"),
            "{msg}"
        );
        assert!(
            v["error"]["fix"]
                .as_str()
                .unwrap()
                .contains(&format!("dla ledger confirm F1 --evidence {growth}")),
            "{v}"
        );
    }
    // The open finding is a blocking item of the final gate with the same route.
    let missing = r.final_gate();
    assert!(
        missing.iter().any(
            |m| m["id"] == "F1" && m["fix"].as_str().unwrap().contains("dla ledger confirm F1")
        ),
        "{missing:?}"
    );
}

#[test]
fn a_finding_closed_before_its_dump_growth_is_listed_and_the_printed_commands_confirm_it() {
    let (r, _w) = scene();
    // Closed by an older close-rest or resolve, before the series was summarized.
    r.ok(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "not ranked",
    ]);
    assert_eq!(r.state("F1"), "unresolved");
    assert!(
        !r.final_gate().iter().any(|m| m["kind"] == "growth"),
        "nothing grew yet"
    );
    let (growth, d) = dump_series(&r, Some("leak"));
    let d = d.unwrap();
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| m["kind"] == "growth" && m["id"] == "F1")
        .unwrap_or_else(|| panic!("{missing:?}"));
    let text = item["description"].as_str().unwrap();
    assert!(text.contains("closed unresolved"), "{text}");
    assert!(
        text.contains("holder:static:app.Conn.request of S4"),
        "{text}"
    );
    assert!(text.contains("grew over every dump"), "{text}");
    assert!(text.contains("nothing decided it since"), "{text}");
    assert!(!text.contains("no alloc-test decided it"), "{text}");
    let fix = item["fix"].as_str().unwrap().to_owned();
    assert!(fix.contains(&format!("dla ledger refute F1.n1 --evidence {d}")) || fix.contains(&format!("dla ledger add --sibling-of F1 --origin hypothesis --hypothesis N-WARM --refute-if \"growth continues over the extra cycles\" --evidence {d}")), "{fix}");
    assert!(fix.contains("dla ledger test F1 --evidence E"), "{fix}");
    // The confirm route ends the commands; the way out of a growth that is no leak of the project
    // follows it as a parenthesis.
    assert!(
        fix.contains(&format!(
            "dla ledger confirm F1 --evidence {growth}   (if the growth is not a leak of the project: "
        )),
        "{fix}"
    );
    // close-rest does not touch it either (it closes candidates only), and a re-close is refused.
    let (code, v) = close(&r, "F1", "unresolved");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
    // The commands, in order, as printed.
    let n = run_fix(&r, &fix);
    assert!(n >= 4, "{fix}");
    assert_eq!(r.state("F1"), "confirmed");
    assert!(
        !r.final_gate()
            .iter()
            .any(|m| m["id"] == "F1" && m["kind"] == "growth"),
        "{:?}",
        r.final_gate()
    );
}

#[test]
fn a_newer_non_leak_dump_discriminator_or_alloc_test_lifts_the_lead_and_close_rest_closes() {
    for lift in ["dump", "alloc-test"] {
        let (r, _w) = scene();
        dump_series(&r, Some("leak"));
        let still = r.ok(&["ledger", "close-rest"]);
        assert!(left_of(&still, "F1").is_some(), "{still}");
        if lift == "dump" {
            discriminator(&r, "non-leak");
        } else {
            add(
                &r,
                evidence(
                    Authority::Test,
                    EvidenceKind::Discriminator,
                    None,
                    json!({"test": "plateau+purge", "favors": "non-leak", "supports": ["N-WARM"],
                        "finding": "F1", "harness": "H1"}),
                ),
            );
        }
        assert!(
            !r.final_gate().iter().any(|m| m["kind"] == "growth"),
            "{lift}: {:?}",
            r.final_gate()
        );
        let out = r.ok(&["ledger", "close-rest"]);
        assert!(left_of(&out, "F1").is_none(), "{lift}: {out}");
        assert_eq!(r.state("F1"), "unresolved", "{lift}: {out}");
    }
}

#[test]
fn the_dump_growth_of_a_sibling_or_of_a_decided_root_is_no_lead() {
    // The growth names the non-leak sibling.
    let (r, _w) = scene();
    r.ok(&[
        "ledger",
        "add",
        "--sibling-of",
        "F1",
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM",
        "--refute-if",
        "growth continues over the extra cycles",
        "--evidence",
        "E3",
    ]);
    add(
        &r,
        evidence(
            Authority::Dump,
            EvidenceKind::GrowthSeries,
            None,
            json!({"series": "S4", "axis": "dump", "n": 3, "gate": "pass",
                "passed": [{"column": "holder:static:x", "finding": "F1.n1"}]}),
        ),
    );
    add(
        &r,
        evidence(
            Authority::Dump,
            EvidenceKind::Discriminator,
            None,
            json!({"test": "dump-plateau", "favors": "leak", "supports": ["leak"],
                "finding": "F1.n1", "series": "S4"}),
        ),
    );
    assert!(
        !r.final_gate().iter().any(|m| m["kind"] == "growth"),
        "{:?}",
        r.final_gate()
    );
    let out = r.ok(&["ledger", "close-rest"]);
    assert!(
        left_of(&out, "F1").is_none_or(|w| !w.contains("S4")),
        "{out}"
    );
    // A confirmed root: its growth is decided, the walk lists nothing.
    let (r, _w) = scene();
    let (growth, d) = dump_series(&r, Some("leak"));
    let d = d.unwrap();
    let fix = r.final_gate();
    let fix = fix
        .iter()
        .find(|m| m["id"] == "F1")
        .map(|m| m["fix"].as_str().unwrap().to_owned())
        .unwrap();
    run_fix(&r, &fix);
    assert_eq!(r.state("F1"), "confirmed", "{growth} {d}");
    assert!(
        !r.final_gate().iter().any(|m| m["kind"] == "growth"),
        "{:?}",
        r.final_gate()
    );
}

#[test]
fn a_finding_with_a_trend_column_and_a_dump_growth_is_one_item() {
    let (r, _w) = scene();
    // Close F1 first, so the growth items (closed findings) are the ones listed.
    r.ok(&[
        "ledger",
        "resolve",
        "F1",
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    dump_series(&r, Some("leak"));
    // A harness series of H1 with a passed column at the same finding.
    fs::create_dir_all(r.run_dir.join("harness/H1")).unwrap();
    fs::write(
        r.run_dir.join("series/S5.json"),
        json!({"id": "S5", "source": {"kind": "harness", "harness": "H1"},
            "axis": "n", "columns": [], "points": []})
        .to_string(),
    )
    .unwrap();
    fs::write(
        r.run_dir.join("harness/H1/run-S5.json"),
        json!({"series": ["S5"]}).to_string(),
    )
    .unwrap();
    let col = dla_core::trend::ColumnTrend {
        name: "probe:Conn.request".into(),
        unit: dla_core::series::Unit::Count,
        kind: dla_core::series::ColumnKind::Gauge,
        region: dla_core::series::SeriesRegion::Heap,
        site: Some("src/conn.js:3".into()),
        finding: None,
        candidates: vec![dla_core::FindingId::new(1)],
        n: 8,
        mk: None,
        sen: None,
        halves: None,
        every_delta: None,
        noise: 0.0,
        rise: 8.0,
        shape: dla_stats::TrendShape::Steady,
        gate: dla_stats::GateVerdict::Pass,
        gate_failed: vec![],
        gate_note: None,
        resized: false,
    };
    dla_core::trend::TrendReport {
        schema_version: 1,
        id: "T1".into(),
        series: "S5".into(),
        thresholds: dla_stats::GateThresholds::default(),
        columns: vec![col],
        targets: vec![],
    }
    .save(&r.run_dir)
    .unwrap();
    let items: Vec<Value> = r
        .final_gate()
        .into_iter()
        .filter(|m| m["id"] == "F1")
        .collect();
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0]["kind"], "growth");
}

#[test]
fn a_leak_confirmed_on_the_dumps_of_the_same_capture_accounts_for_the_other_holders_that_grew() {
    let (r, _w) = scene();
    // A second holder of the same capture: a field of the object that leaks.
    let seed = add(
        &r,
        evidence(
            Authority::Dump,
            EvidenceKind::GrowthSeries,
            None,
            json!({"series": "S2", "axis": "dump", "n": 3, "gate": "pass", "passed": []}),
        ),
    );
    let added = r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &seed,
        "--site",
        "src/conn.js:1",
        "--holder",
        "container-field",
        "--owner",
        "app.Conn.socket",
    ]);
    assert_eq!(added["id"], "F2", "{added}");
    fs::create_dir_all(r.run_dir.join("series")).unwrap();
    fs::write(
        r.run_dir.join("series/S4.json"),
        json!({"schema_version": 1, "id": "S4",
            "source": {"kind": "dump", "command": "dla summarize heap-graph"},
            "axis": "dump", "warmup": 0, "columns": [], "points": [], "events": [],
            "limitations": []})
        .to_string(),
    )
    .unwrap();
    for i in 1..=3 {
        add(
            &r,
            evidence(
                Authority::Dump,
                EvidenceKind::RetentionPath,
                Some(&format!(
                    "harness/H1/out/p{i}/capture/7/heap-{i}.heapsnapshot"
                )),
                json!({"holders": []}),
            ),
        );
    }
    add(
        &r,
        evidence(
            Authority::Dump,
            EvidenceKind::GrowthSeries,
            None,
            json!({"series": "S4", "axis": "dump", "n": 3, "gate": "pass",
                "passed": [
                    {"column": "holder:static:app.Conn.request", "finding": "F1", "candidates": ["F1"]},
                    {"column": "holder:field:app.Conn.socket", "finding": "F2", "candidates": ["F2"]}]}),
        ),
    );
    for (f, column) in [
        ("F1", "holder:static:app.Conn.request"),
        ("F2", "holder:field:app.Conn.socket"),
    ] {
        add(
            &r,
            evidence(
                Authority::Dump,
                EvidenceKind::Discriminator,
                None,
                json!({"test": "dump-plateau", "favors": "leak", "supports": ["leak"],
                    "finding": f, "column": column, "series": "S4",
                    "reason": "retained bytes and count grew over every interval"}),
            ),
        );
    }
    // Both are leads while nothing is decided: neither closes.
    let (code, v) = close(&r, "F2", "unresolved");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
    // F1 is confirmed through its printed route.
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| {
            m["id"] == "F1"
                && m["fix"]
                    .as_str()
                    .is_some_and(|f| f.contains("dla ledger confirm F1"))
        })
        .unwrap_or_else(|| panic!("{missing:?}"));
    let fix = item["fix"].as_str().unwrap().to_owned();
    run_fix(&r, &fix);
    assert_eq!(r.state("F1"), "confirmed");
    // F2 grew in the same capture's dumps: the confirmed leak accounts for it, so it is no
    // longer a lead and closes.
    assert!(
        !r.final_gate().iter().any(|m| m["id"] == "F2"
            && m["fix"]
                .as_str()
                .is_some_and(|f| f.contains("dla ledger confirm F2"))),
        "{:?}",
        r.final_gate()
    );
    let (code, v) = close(&r, "F2", "unresolved");
    assert_eq!(code, 0, "{v}");
    assert_eq!(r.state("F2"), "unresolved");
}

/// A heap-graph series `series` of harness `harness`: a retention record per snapshot, the
/// growth record whose passed columns name `findings`, a leak discriminator for each.
fn holder_series(r: &R, series: &str, harness: &str, findings: &[(&str, &str)]) -> String {
    for i in 1..=3 {
        add(
            r,
            evidence(
                Authority::Dump,
                EvidenceKind::RetentionPath,
                Some(&format!(
                    "harness/{harness}/out/p{i}/capture/7/heap-{i}.heapsnapshot"
                )),
                json!({"holders": []}),
            ),
        );
    }
    let passed: Vec<Value> = findings
        .iter()
        .map(|(f, c)| json!({"column": c, "finding": f, "candidates": [f]}))
        .collect();
    let growth = add(
        r,
        evidence(
            Authority::Dump,
            EvidenceKind::GrowthSeries,
            None,
            json!({"series": series, "axis": "dump", "n": 3, "gate": "pass", "passed": passed}),
        ),
    );
    for (f, c) in findings {
        add(
            r,
            evidence(
                Authority::Dump,
                EvidenceKind::Discriminator,
                None,
                json!({"test": "dump-plateau", "favors": "leak", "supports": ["leak"],
                    "finding": f, "column": c, "series": series,
                    "reason": "retained bytes and count grew over every interval"}),
            ),
        );
    }
    growth
}

/// F2, a second holder, raised the way the census raises one.
fn second_holder(r: &R) {
    let seed = add(
        r,
        evidence(
            Authority::Dump,
            EvidenceKind::GrowthSeries,
            None,
            json!({"series": "S2", "axis": "dump", "n": 3, "gate": "pass", "passed": []}),
        ),
    );
    let added = r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &seed,
        "--site",
        "src/conn.js:1",
        "--holder",
        "container-field",
        "--owner",
        "app.Conn.socket",
    ]);
    assert_eq!(added["id"], "F2", "{added}");
}

fn confirm_f1(r: &R) {
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| {
            m["id"] == "F1"
                && m["fix"]
                    .as_str()
                    .is_some_and(|f| f.contains("dla ledger confirm F1"))
        })
        .unwrap_or_else(|| panic!("{missing:?}"));
    let fix = item["fix"].as_str().unwrap().to_owned();
    run_fix(r, &fix);
    assert_eq!(r.state("F1"), "confirmed");
}

#[test]
fn a_part_that_grew_in_a_later_capture_too_is_explained_by_the_leak_of_the_first() {
    let (r, _w) = scene();
    second_holder(&r);
    holder_series(
        &r,
        "S4",
        "H1",
        &[
            ("F1", "holder:static:app.Conn.request"),
            ("F2", "holder:field:app.Conn.socket"),
        ],
    );
    confirm_f1(&r);
    // A later capture of another harness shows F2 grow again; nothing confirmed there.
    holder_series(&r, "S5", "H2", &[("F2", "holder:field:app.Conn.socket")]);
    let (code, v) = close(&r, "F2", "unresolved");
    assert_eq!(code, 0, "{v}");
}

#[test]
fn a_holder_that_grew_only_in_another_capture_stays_a_lead() {
    let (r, _w) = scene();
    second_holder(&r);
    holder_series(&r, "S4", "H1", &[("F1", "holder:static:app.Conn.request")]);
    holder_series(&r, "S5", "H2", &[("F2", "holder:field:app.Conn.socket")]);
    confirm_f1(&r);
    let (code, v) = close(&r, "F2", "unresolved");
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "growth-not-measured", "{v}");
}

/// The alternative a dump lead's route prints after its last command, as commands: the text in
/// the parenthesis that starts `if the growth is not a leak of the project: `, cut at `; `.
fn alternative_of(why: &str) -> Vec<String> {
    let start = why
        .find("(if the growth is not a leak of the project: ")
        .unwrap_or_else(|| panic!("no alternative in {why}"))
        + "(if the growth is not a leak of the project: ".len();
    let text = &why[start..];
    let text = &text[..text.rfind(')').unwrap()];
    text.split("; ")
        .map(|c| c.replace("N-OBSERVER|N-INTENT", "N-OBSERVER"))
        .collect()
}

#[test]
fn a_dump_lead_with_no_site_offers_the_alloc_test_first_and_one_favoring_non_leak_lifts_it() {
    let (r, _w) = scene_with(None);
    dump_series(&r, Some("leak"));
    let out = r.ok(&["ledger", "close-rest"]);
    let why = left_of(&out, "F1").unwrap_or_else(|| panic!("F1 not left open: {out}"));
    let route = why.split("nothing decided it: ").nth(1).unwrap();
    // No placeholder blocks the route: the first command measures, on the harness of the dumps.
    assert!(
        route.starts_with("dla alloc-test H1 --finding F1"),
        "{route}"
    );
    // The annotation stays behind it, for whoever can fill it in.
    assert!(
        route.contains("dla ledger annotate F1 --site <file:line>"),
        "{route}"
    );
    // The refusal and the final gate print the same first command.
    let (code, v) = close(&r, "F1", "unresolved");
    assert_ne!(code, 0, "{v}");
    assert!(
        v["error"]["fix"]
            .as_str()
            .unwrap()
            .starts_with("dla alloc-test H1 --finding F1"),
        "{v}"
    );
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| m["kind"] == "finding" || m["kind"] == "growth")
        .unwrap_or_else(|| panic!("{missing:?}"));
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .starts_with("dla alloc-test H1 --finding F1"),
        "{item}"
    );
    // An alloc-test of the harness that favors non-leak lifts the lead.
    add(
        &r,
        evidence(
            Authority::Test,
            EvidenceKind::Discriminator,
            None,
            json!({"test": "plateau+purge", "favors": "non-leak", "supports": ["N-WARM"],
                "finding": "F1", "harness": "H1"}),
        ),
    );
    assert!(
        !r.final_gate().iter().any(|m| m["kind"] == "growth"),
        "{:?}",
        r.final_gate()
    );
}

#[test]
fn a_non_leak_explanation_confirmed_on_the_retention_record_of_the_holder_lifts_the_dump_lead() {
    let (r, workload) = scene();
    let (growth, d) = dump_series(&r, Some("leak"));
    let d = d.unwrap();
    // A retention-path record of the holder, read from a dump of the harness.
    let record = add(
        &r,
        evidence(
            Authority::Dump,
            EvidenceKind::RetentionPath,
            Some("harness/H1/out/p3/capture/7/heap-3.heapsnapshot"),
            json!({"holders": [{"key": "static:app.Conn.request", "retained": 900,
                "holder_kind": "static/global"}]}),
        ),
    );
    let out = r.ok(&["ledger", "close-rest"]);
    let why = left_of(&out, "F1").unwrap_or_else(|| panic!("F1 not left open: {out}"));
    // The confirm route as before, and the way out of a growth that is no leak of the project.
    assert!(
        why.contains(&format!("dla ledger confirm F1 --evidence {growth}")),
        "{why}"
    );
    let alt = alternative_of(why);
    assert_eq!(alt.len(), 4, "{alt:?}");
    assert!(
        alt[0].starts_with(
            "dla ledger add --sibling-of F1 --origin hypothesis --hypothesis N-OBSERVER"
        ),
        "{alt:?}"
    );
    assert_eq!(
        alt[1],
        format!("dla ledger test F1.n1 --evidence {workload}")
    );
    assert_eq!(
        alt[2],
        format!("dla ledger confirm F1.n1 --evidence {record}")
    );
    assert_eq!(
        alt[3],
        format!(
            "dla ledger resolve F1 --as unresolved --reason inconclusive --remedy \"explained by F1.n1: <what keeps the holder, from {record}>\""
        )
    );
    // The first two steps as printed.
    for step in &alt[..2] {
        let w = words_of(step);
        let args: Vec<&str> = w.iter().map(String::as_str).collect();
        r.ok(&args);
    }
    assert_eq!(r.state("F1.n1"), "under_test");
    // Anything but the record of the holder is refused, with the record named in the fix: the
    // growth record, the discriminator, and the workload.
    for other in [&growth, &d, &workload] {
        let (code, v) = r.run(&["ledger", "confirm", "F1.n1", "--evidence", other]);
        assert_ne!(code, 0, "{other}: {v}");
        assert_eq!(v["error"]["code"], "leak-refutation-required", "{v}");
        assert!(
            v["error"]["fix"]
                .as_str()
                .unwrap()
                .contains(&format!("dla ledger confirm F1.n1 --evidence {record}")),
            "{v}"
        );
    }
    assert_eq!(r.state("F1.n1"), "under_test");
    // On the record of the holder it is confirmed, and the lead is gone.
    let w = words_of(&alt[2]);
    r.ok(&w.iter().map(String::as_str).collect::<Vec<_>>());
    assert_eq!(r.state("F1.n1"), "confirmed");
    assert!(
        !r.final_gate().iter().any(|m| m["id"] == "F1"
            && m["fix"]
                .as_str()
                .is_some_and(|f| f.contains("dla ledger confirm F1 --evidence"))),
        "{:?}",
        r.final_gate()
    );
    // The newest record decides: a growth record of the holder written after brings it back.
    let (growth2, _) = dump_series(&r, Some("leak"));
    let missing = r.final_gate();
    let item = missing
        .iter()
        .find(|m| {
            m["id"] == "F1"
                && m["fix"]
                    .as_str()
                    .is_some_and(|f| f.contains("dla ledger confirm F1 --evidence"))
        })
        .unwrap_or_else(|| panic!("{missing:?}"));
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .contains(&format!("dla ledger confirm F1 --evidence {growth2}")),
        "{item}"
    );
}

#[test]
fn a_confirmed_explanation_lifts_the_lead_and_the_root_is_closed_not_refuted() {
    let (r, _workload) = scene();
    dump_series(&r, Some("leak"));
    let record = add(
        &r,
        evidence(
            Authority::Dump,
            EvidenceKind::RetentionPath,
            Some("harness/H1/out/p3/capture/7/heap-3.heapsnapshot"),
            json!({"holders": [{"key": "static:app.Conn.request", "retained": 900,
                "holder_kind": "static/global"}]}),
        ),
    );
    let out = r.ok(&["ledger", "close-rest"]);
    let alt = alternative_of(left_of(&out, "F1").unwrap());
    // No explanation yet: the root is not refuted by the record alone.
    let (code, v) = r.run(&["ledger", "refute", "F1", "--evidence", &record]);
    assert_ne!(code, 0, "{v}");
    for step in &alt[..3] {
        let w = words_of(step);
        r.ok(&w.iter().map(String::as_str).collect::<Vec<_>>());
    }
    assert_eq!(r.state("F1.n1"), "confirmed");
    // Confirmed: the explanation does not refute the root (a retention record shows what keeps
    // the holder, not that the growth stops: a refute still needs a discriminator)...
    let (code, v) = r.run(&["ledger", "refute", "F1", "--evidence", &record]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "discriminator-required", "{v}");
    // ...but the lead is lifted, and the printed last step closes the root unresolved.
    assert!(
        alt[3].starts_with("dla ledger resolve F1 --as unresolved --reason inconclusive"),
        "{}",
        alt[3]
    );
    let w = words_of(&alt[3]);
    r.ok(&w.iter().map(String::as_str).collect::<Vec<_>>());
    assert!(r.state("F1").starts_with("unresolved"), "{}", r.state("F1"));
}
