//! What a measured Python holder implies, and a line that did not run proves nothing, through the
//! binary with real `python3` runs.
//!
//! The project has a module-level provider object that keeps a list. Its method makes an object and
//! keeps it on one branch (no real provider installed) and returns early on the other:
//!
//! * the line of the object (`shop/providers.py:8`) allocates and keeps nothing on the early-return
//!   branch, so a driver whose unit takes that branch says nothing against a leak at the line;
//! * `dla harness run --capture alloc,heap` and `dla summarize heap-graph` name the holder
//!   (`shop.providers.PROVIDER`, of the project): no finding has it, and that is the first item of
//!   the gate, of `dla run status`, and what refuses `dla harness gen --next`.
//!
//! Each test skips, printing the reason, when `python3` is missing.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn have_python() -> bool {
    let ok = Std::new("python3")
        .args([
            "-c",
            "import sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipping: python3 is not on PATH");
    }
    ok
}

struct Out {
    code: i32,
    text: String,
    json: Value,
}

fn dla(project: &Path, args: &[&str]) -> Out {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
        "DLA_CAPTURE",
        "DLA_CAPTURE_DIR",
        "DLA_TRACEMALLOC_FRAMES",
        "DLA_LINES",
        "DLA_LINES_MODE",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1-4")
        .env("DLA_CONSENT_RAW_VALUES", "1");
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    o.json
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// The provider: line 8 makes the object, line 9 keeps it; `real=True` returns before both.
const PROVIDERS: &str = "class _Provider:\n    def __init__(self):\n        self._items = []\n\n    def make(self, i, real):\n        if real:\n            return None\n        obj = [i]\n        self._items.append(obj)\n        return obj\n\n\nPROVIDER = _Provider()\n\n\ndef make(i, real=False):\n    return PROVIDER.make(i, real)\n";
/// A module-level table of its own: the census lists it, so the first harness targets it.
const CACHE: &str = "REG = {}\n\n\ndef put(k):\n    REG[k] = k\n";
/// A cache that fills to 2200 entries and then stops: a warm-up that lasts four rounds of the
/// driver's 440 units.
const WARM: &str = "_C = {}\n\n\ndef fill(i):\n    if len(_C) < 2200:\n        _C[i] = [i]\n";
/// A function that makes an object and gives it to the caller.
const MAKER: &str = "def build(i):\n    return [i]\n";

const SITE: &str = "shop/providers.py:8";
const COLUMN: &str = "py.blocks@shop/providers.py:8";
const OWNER: &str = "shop.providers.PROVIDER";

struct Run {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
}

impl Run {
    fn ok(&self, args: &[&str]) -> Value {
        ok(&self.project, args)
    }

    /// The `next` entry of `v` that starts with `prefix`.
    fn next_of(v: &Value, prefix: &str) -> Option<String> {
        v["next"]
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .find(|n| n.starts_with(prefix))
            .map(str::to_owned)
    }

    /// The full gate list of the stage (`classify`) or `final`.
    fn gate(&self, stage: &str) -> Vec<Value> {
        let args: Vec<&str> = if stage == "final" {
            vec!["gate", "check", "--final"]
        } else {
            vec!["gate", "check", "--stage", stage]
        };
        let _ = dla(&self.project, &args);
        let name = if stage == "final" { "final" } else { stage };
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join(format!("gate/{name}.json"))).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }

    /// Generates a driver (`--next`, or for `--probe F`), fills its hooks and returns its id.
    fn driver(&self, probe: Option<&str>, setup: &str, unit: &str) -> String {
        let mut args = vec!["harness", "gen", "--lang", "python", "--driver"];
        match probe {
            Some(f) => args.extend(["--probe", f]),
            None => args.push("--next"),
        }
        let g = self.ok(&args);
        let edit = self.project.join(g["summary"]["edit"].as_str().unwrap());
        let text = fs::read_to_string(&edit)
            .unwrap()
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
                setup,
            )
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
                unit,
            );
        assert!(!text.contains("has not been filled in"));
        fs::write(&edit, text).unwrap();
        g["summary"]["harness"].as_str().unwrap().to_owned()
    }
}

const SETUP: &str = "    from shop import cache, providers, maker\n    _state[\"cache\"] = cache\n    _state[\"providers\"] = providers\n    _state[\"maker\"] = maker";
/// The setup of the warm-up driver: only the cache that fills and stops.
const SETUP_WARM: &str = "    from shop import warm\n    _state[\"warm\"] = warm";
/// The unit of the first driver: it takes the branch that keeps the object.
const KEEPS: &str = "    _state[\"cache\"].put(i)\n    _state[\"providers\"].make(i)";

/// Run init, census, discover, the first driver (its unit calls `unit`), one run of it (series S1)
/// and its trend (T1). The census lists `shop/cache.py`'s table (F1) and nothing at the provider.
fn prepared(unit: &str) -> Run {
    prepared_with(
        unit,
        SETUP,
        &[
            ("shop/cache.py", CACHE),
            ("shop/providers.py", PROVIDERS),
            ("shop/maker.py", MAKER),
        ],
    )
}

/// [`prepared`] with the project files `files` and the driver's `setup`.
fn prepared_with(unit: &str, setup: &str, files: &[(&str, &str)]) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write(&project, "shop/__init__.py", "");
    for (rel, text) in files {
        write(&project, rel, text);
    }
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["census", "--lang", "python"]);
    ok(&project, &["workload", "discover", "--lang", "python"]);
    let r = Run {
        _dir: dir,
        project,
        run_dir,
    };
    let h = r.driver(None, setup, unit);
    assert_eq!(h, "H1");
    r.ok(&["harness", "run", "H1", "--mode", "fresh"]);
    r.ok(&["trend", "S1"]);
    r
}

/// Adds the finding at the growing line the way a run that reads only the trend does: one finding
/// at the line, holder `container-field`, no owner. Returns its id.
fn finding_at_the_line(r: &Run) -> String {
    finding_at(r, SITE)
}

/// [`finding_at_the_line`] for the growing line `site`.
fn finding_at(r: &Run, site: &str) -> String {
    let t = r.ok(&["trend", "S1"]);
    // The trend prints the add command of a line that is still growing; a column that grew and
    // levelled off has none, and the trend's own evidence serves.
    let e = match Run::next_of(&t, "dla ledger add --origin growth-series --evidence ") {
        Some(add) => add.split_whitespace().nth(6).unwrap().to_owned(),
        None => t["evidence"][0].as_str().unwrap().to_owned(),
    };
    assert!(e.starts_with('E'), "{}", t["next"]);
    let added = r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &e,
        "--site",
        site,
        "--region",
        "heap",
        "--holder",
        "container-field",
    ]);
    added["id"].as_str().unwrap().to_owned()
}

/// The state of finding `f` as `dla ledger list` prints it.
fn state_of(r: &Run, f: &str) -> String {
    let o = dla(&r.project, &["ledger", "list"]);
    o.text
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|v| v["id"] == f)
        .unwrap_or_else(|| panic!("{}", o.text))["state"]
        .as_str()
        .unwrap()
        .to_owned()
}

// ------------------------------------------------------------------------------------ the line

#[test]
fn a_unit_that_takes_the_other_branch_is_inconclusive_and_cannot_refute_the_leak() {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    let t = r.ok(&["trend", "S1"]);
    assert!(
        t["passed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["column"] == COLUMN),
        "the line did not grow in the first harness: {t}"
    );
    let f = finding_at_the_line(&r);
    // A second driver for the finding whose unit calls the function on the early-return branch.
    let h2 = r.driver(
        Some(&f),
        "    from shop import providers\n    _state[\"providers\"] = providers",
        "    _state[\"providers\"].make(i, True)",
    );
    r.ok(&["harness", "run", &h2, "--mode", "fresh"]);
    let a = r.ok(&["alloc-test", &h2, "--finding", &f]);
    assert_eq!(a["favors"], "inconclusive", "{a}");
    assert_eq!(
        a["basis"],
        format!(
            "the function of {SITE} ran but the line itself did not (a branch the unit does not take)"
        ),
        "{a}"
    );
    // The next step names the harness of the series that showed the growth.
    let next = a["next"].as_array().unwrap();
    assert!(
        next[0].as_str().unwrap().starts_with(&format!(
            "dla alloc-test H1 --finding {f}   (the growth was measured there"
        )),
        "{next:?}"
    );
    // The discriminator cannot refute the leak.
    let o = dla(
        &r.project,
        &[
            "ledger",
            "refute",
            &f,
            "--evidence",
            a["evidence"].as_str().unwrap(),
        ],
    );
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(state_of(&r, &f), "candidate");
}

#[test]
fn a_unit_that_runs_the_line_and_releases_the_object_is_non_leak_with_the_count() {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    let f = finding_at_the_line(&r);
    let h3 = r.driver(
        Some(&f),
        "    from shop import providers\n    _state[\"providers\"] = providers",
        "    _state[\"providers\"].make(i, False)\n    _state[\"providers\"].PROVIDER._items.clear()",
    );
    let run = r.ok(&["harness", "run", &h3, "--mode", "fresh"]);
    // The harness targets the finding: its line is counted in a plain run, with no alloc-test.
    let first = run["summary"]["series"][0].as_str().unwrap();
    let plain: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join(format!("harness/{h3}/lines-{first}.json"))).unwrap(),
    )
    .unwrap();
    assert!(
        plain["sites"][SITE]
            .as_array()
            .is_some_and(|c| c.len() >= 8),
        "{plain}"
    );
    let a = r.ok(&["alloc-test", &h3, "--finding", &f]);
    assert_eq!(a["favors"], "non-leak", "{a}");
    let basis = a["basis"].as_str().unwrap();
    let ran: u64 = basis
        .strip_prefix("the line ran ")
        .and_then(|b| b.split(' ').next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("{basis}"));
    assert!(ran >= 8, "{basis}");
    assert!(
        basis.contains(" times in this series and none of its blocks stayed"),
        "{basis}"
    );
    // The same series says how often the line ran, per point.
    let series = a["series"].as_str().unwrap();
    let runs: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join(format!("harness/{h3}/lines-{series}.json"))).unwrap(),
    )
    .unwrap();
    let counts = runs["sites"][SITE].as_array().unwrap();
    assert!(
        counts.iter().all(|c| c.as_f64().is_some())
            && counts.last().unwrap().as_f64().unwrap() >= 8.0,
        "{runs}"
    );
}

#[test]
fn an_alloc_test_counts_the_line_of_a_finding_on_a_harness_that_does_not_target_it() {
    if !have_python() {
        return;
    }
    // H1 targets the table (F1) only; it measured the line of F2 (its column is tied to F2). Its
    // unit is then changed to take the early-return branch, and the alloc-test on H1 is about F2:
    // the harness's manifest names no line of F2, so the alloc-test itself asks for its count.
    let r = prepared(KEEPS);
    let f = finding_at_the_line(&r);
    let driver = r.run_dir.join("harness/H1/driver/dla_harness.py");
    let text = fs::read_to_string(&driver).unwrap();
    let old = "_state[\"providers\"].make(i)";
    assert_eq!(text.matches(old).count(), 1);
    fs::write(
        &driver,
        text.replace(old, "_state[\"providers\"].make(i, True)"),
    )
    .unwrap();
    let a = r.ok(&["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "inconclusive", "{a}");
    assert_eq!(
        a["basis"],
        format!(
            "the function of {SITE} ran but the line itself did not (a branch the unit does not take)"
        ),
        "{a}"
    );
}

#[test]
fn a_series_with_no_count_of_the_line_is_inconclusive_never_non_leak() {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    let f = finding_at_the_line(&r);
    // The unit runs the line and releases the object: with a counting agent this is non-leak
    // (the test above). The harness's copy of the agent is made one that does not count, as an
    // older agent does not: the series has no count of the line.
    let h = r.driver(
        Some(&f),
        "    from shop import providers\n    _state[\"providers\"] = providers",
        "    _state[\"providers\"].make(i, False)\n    _state[\"providers\"].PROVIDER._items.clear()",
    );
    let agent = r
        .run_dir
        .join(format!("harness/{h}/agent/sitecustomize.py"));
    let text = fs::read_to_string(&agent).unwrap();
    let old = "rec[\"line_counts\"] = dict(_lines[\"counts\"])";
    assert_eq!(text.matches(old).count(), 1);
    fs::write(&agent, text.replace(old, "pass")).unwrap();
    r.ok(&["harness", "run", &h, "--mode", "fresh"]);
    let a = r.ok(&["alloc-test", &h, "--finding", &f]);
    assert_eq!(a["favors"], "inconclusive", "{a}");
    assert_eq!(
        a["basis"],
        format!(
            "this series has no count of how often the line {SITE} ran, so the absence of {COLUMN} proves nothing about it"
        ),
        "{a}"
    );
    let o = dla(
        &r.project,
        &[
            "ledger",
            "refute",
            &f,
            "--evidence",
            a["evidence"].as_str().unwrap(),
        ],
    );
    assert_ne!(o.code, 0, "{}", o.text);
}

// ------------------------------------------------------------------------------------ the holder

/// The capture run of H1 and its heap-graph summary with `--finding` `about`: returns the output.
fn capture_and_summarize(r: &Run, about: Option<&str>) -> Value {
    let run = r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc,heap",
    ]);
    let cmd = run["summary"]["summarize"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|c| c.starts_with("dla summarize heap-graph "))
        .unwrap_or_else(|| panic!("{run}"))
        .to_owned();
    let file = cmd.strip_prefix("dla summarize heap-graph ").unwrap();
    match about {
        Some(f) => r.ok(&["summarize", "heap-graph", file, "--finding", f]),
        None => r.ok(&["summarize", "heap-graph", file]),
    }
}

#[test]
fn a_holder_of_the_project_that_no_finding_covers_leads_the_gate_the_status_and_gen_next() {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    // Before the capture the line has no retention record: one capture item for the series, no
    // add command per line, and `harness gen --next` is not refused for it.
    let items = r.gate("classify");
    assert_eq!(items[0]["kind"], "measured_capture", "{items:?}");
    assert!(
        items[0]["fix"]
            .as_str()
            .unwrap()
            .starts_with("dla harness run H1 --mode fresh --capture alloc,heap"),
        "{}",
        items[0]
    );
    assert!(
        items
            .iter()
            .all(|m| !m["id"].as_str().unwrap_or("").ends_with(COLUMN)),
        "{items:?}"
    );
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["blocking"][0]["kind"], "measured_capture", "{st}");
    assert!(st["_attention"][0].as_str().is_some(), "{st}");

    // The capture, summarized for another finding (the table, F1): the holder of the provider is
    // in the output first, whatever the output is cut to.
    let s = capture_and_summarize(&r, Some("F1"));
    let attention = s["_attention"][0].as_str().unwrap_or_else(|| panic!("{s}"));
    assert!(
        attention.contains("static:shop.providers.PROVIDER")
            && attention.contains(&format!("the owner {OWNER}"))
            && attention.contains("dla ledger add --origin growth-series --evidence "),
        "{attention}"
    );
    assert!(s.to_string().starts_with("{\"_attention\":["), "{s}");
    let evidence = s["evidence"][0].as_str().unwrap();
    let add = format!(
        "dla ledger add --origin growth-series --evidence {evidence} --owner {OWNER} --site {SITE} --holder static/global --region heap"
    );
    assert!(attention.ends_with(&add), "{attention}");

    // The gate: the holder is the first item with the literal command, the line's own item is
    // not listed beside it.
    for stage in ["classify", "final"] {
        let items = r.gate(stage);
        assert_eq!(items[0]["kind"], "measured_holder", "{stage}: {items:?}");
        assert_eq!(items[0]["fix"], add.as_str(), "{stage}");
        assert!(
            items
                .iter()
                .all(|m| !m["id"].as_str().unwrap_or("").ends_with(COLUMN)),
            "{stage}: {items:?}"
        );
    }
    // `dla run status` leads with it, before the queue.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["blocking"][0]["kind"], "measured_holder", "{st}");
    assert_eq!(st["next_command"], add.as_str(), "{st}");
    assert!(
        st["_attention"][0]
            .as_str()
            .unwrap()
            .starts_with("holder:shop.providers.PROVIDER"),
        "{st}"
    );
    // `dla harness gen --next` is refused, whatever the language and mode; the fix is the command.
    let o = dla(
        &r.project,
        &["harness", "gen", "--lang", "python", "--driver", "--next"],
    );
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(
        o.json["error"]["code"], "measured-holder-first",
        "{}",
        o.text
    );
    assert_eq!(o.json["error"]["fix"], add.as_str(), "{}", o.text);
    let o = dla(&r.project, &["harness", "gen", "--lang", "jvm", "--next"]);
    assert_eq!(
        o.json["error"]["code"], "measured-holder-first",
        "{}",
        o.text
    );
    // Without `--next` a harness is still made.
    let o = dla(
        &r.project,
        &["harness", "gen", "--lang", "python", "--driver"],
    );
    assert_ne!(
        o.json["error"]["code"], "measured-holder-first",
        "{}",
        o.text
    );

    // The add command, copied from the item: the item and the refusal are gone, and the line's
    // column is tied to the new finding.
    let args: Vec<&str> = add.split_whitespace().skip(1).collect();
    let added = r.ok(&args);
    let f = added["id"].as_str().unwrap().to_owned();
    for stage in ["classify", "final"] {
        let items = r.gate(stage);
        assert!(
            items.iter().all(|m| m["kind"] != "measured_holder"),
            "{stage}: {items:?}"
        );
    }
    let st = r.ok(&["run", "status"]);
    assert!(st.get("_attention").is_none(), "{st}");
    let o = dla(
        &r.project,
        &["harness", "gen", "--lang", "python", "--driver", "--next"],
    );
    assert_ne!(
        o.json["error"]["code"], "measured-holder-first",
        "{}",
        o.text
    );
    let t = r.ok(&["trend", "S1"]);
    let col = t["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == COLUMN)
        .unwrap_or_else(|| panic!("{t}"));
    assert_eq!(col["candidates"], serde_json::json!([f]), "{col}");
    // The summary of the same capture, now that a finding has the owner, has no `_attention`.
    let again = capture_and_summarize(&r, Some(&f));
    assert!(again.get("_attention").is_none(), "{again}");
}

#[test]
fn a_summary_for_another_finding_still_leads_with_the_holder_when_the_output_is_cut() {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    let s = capture_and_summarize(&r, Some("F1"));
    // The first key is the one the output bound never cuts: a reader that pipes the output through
    // `head` sees it before anything about the finding it asked for.
    let text = s.to_string();
    let head: String = text.chars().take(200).collect();
    assert!(
        head.starts_with("{\"_attention\":[\"A HOLDER OF THE PROJECT HAS NO FINDING"),
        "{head}"
    );
}

#[test]
fn a_driver_that_keeps_the_objects_in_its_own_module_global_holds_them_for_the_tests() {
    if !have_python() {
        return;
    }
    // The unit makes the object in the project (`shop/maker.py:2`) and the driver keeps it in a
    // module-level list of its own: the holder is `__main__._KEPT`, of the tests.
    let r = prepared_with_global(
        "_KEPT.append(_state[\"maker\"].build(i))\n    _state[\"cache\"].put(i)",
    );
    let t = r.ok(&["trend", "S1"]);
    let column = "py.blocks@shop/maker.py:2";
    assert!(
        t["passed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["column"] == column),
        "{t}"
    );
    let s = capture_and_summarize(&r, Some("F1"));
    assert!(s.get("_attention").is_none(), "{s}");
    let h = &s["holders"][0];
    assert_eq!(h["key"], "static:__main__._KEPT", "{s}");
    let raw: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join(s["raw"]["path"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    let held = raw["holders"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["key"] == "static:__main__._KEPT")
        .unwrap();
    assert_eq!(held["of"], "tests", "{raw}");
    // The line is no gate item, and no add command is printed for it; the gate says in one line
    // how many lines the test code holds.
    for stage in ["classify", "final"] {
        let items = r.gate(stage);
        assert!(
            items.iter().all(|m| m["kind"] != "measured_holder"
                && !m["id"].as_str().unwrap_or("").ends_with(column)),
            "{stage}: {items:?}"
        );
    }
    let g = dla(&r.project, &["gate", "check", "--final"]);
    let note = g.json["test_code"]
        .as_str()
        .unwrap_or_else(|| panic!("{}", g.text));
    assert!(note.contains("test code holds 1 growing line"), "{note}");
    // `dla harness gen --next` is not refused for a holder of the tests.
    let o = dla(
        &r.project,
        &["harness", "gen", "--lang", "python", "--driver", "--next"],
    );
    assert_ne!(
        o.json["error"]["code"], "measured-holder-first",
        "{}",
        o.text
    );
}

/// [`prepared`] with a module-level list `_KEPT` in the driver, written before the first run.
fn prepared_with_global(unit_body: &str) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write(&project, "shop/__init__.py", "");
    write(&project, "shop/cache.py", CACHE);
    write(&project, "shop/providers.py", PROVIDERS);
    write(&project, "shop/maker.py", MAKER);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["census", "--lang", "python"]);
    ok(&project, &["workload", "discover", "--lang", "python"]);
    let r = Run {
        _dir: dir,
        project,
        run_dir,
    };
    let g = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    let edit = r.project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            SETUP,
        )
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            &format!("    {unit_body}"),
        );
    fs::write(&edit, format!("_KEPT = []\n{text}")).unwrap();
    r.ok(&["harness", "run", "H1", "--mode", "fresh"]);
    r.ok(&["trend", "S1"]);
    r
}

#[test]
fn lines_no_retention_record_roots_get_their_add_commands_after_the_capture() {
    if !have_python() {
        return;
    }
    // Objects nothing in the program holds (a reference taken through the C API, standing in for
    // an extension module): the capture roots none of them in Python.
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write(&project, "shop/__init__.py", "");
    write(
        &project,
        "shop/store.py",
        "import ctypes\n\n\ndef pin(i):\n    obj = [i]\n    ctypes.pythonapi.Py_IncRef(ctypes.py_object(obj))\n",
    );
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["census", "--lang", "python"]);
    ok(&project, &["workload", "discover", "--lang", "python"]);
    let r = Run {
        _dir: dir,
        project,
        run_dir,
    };
    let h = r.driver(
        None,
        "    from shop import store\n    _state[\"store\"] = store",
        "    _state[\"store\"].pin(i)",
    );
    assert_eq!(h, "H1");
    r.ok(&["harness", "run", "H1", "--mode", "fresh"]);
    r.ok(&["trend", "S1"]);
    let column = "py.blocks@shop/store.py:5";
    // One capture item for the series, and no per-line item.
    let items = r.gate("final");
    let captures: Vec<&Value> = items
        .iter()
        .filter(|m| m["kind"] == "measured_capture")
        .collect();
    assert_eq!(captures.len(), 1, "{items:?}");
    assert_eq!(captures[0]["id"], "S1:capture");
    assert!(
        captures[0]["fix"]
            .as_str()
            .unwrap()
            .starts_with("dla harness run H1 --mode fresh --capture alloc,heap, then the two dla summarize commands"),
        "{}",
        captures[0]
    );
    assert!(
        items
            .iter()
            .all(|m| !m["id"].as_str().unwrap_or("").ends_with(column)),
        "{items:?}"
    );
    // After the capture and its summary the harness has a retention record: the line, which no
    // holder of the project keeps, has its own item and the add command again.
    capture_and_summarize(&r, None);
    let items = r.gate("final");
    assert!(
        items.iter().all(|m| m["kind"] != "measured_capture"),
        "{items:?}"
    );
    let line: Vec<&Value> = items
        .iter()
        .filter(|m| m["id"].as_str().unwrap_or("").ends_with(column))
        .collect();
    assert_eq!(line.len(), 1, "{items:?}");
    assert!(
        line[0]["fix"]
            .as_str()
            .unwrap()
            .starts_with("dla ledger add --origin growth-series --evidence "),
        "{}",
        line[0]
    );
}

// ------------------------------------------------------------ a series that never showed the growth

/// The command `_attention` of a summary prints, as arguments (without the leading `dla`).
fn add_args(summary: &Value) -> Vec<String> {
    let text = summary["_attention"][0].as_str().unwrap();
    let at = text.find("dla ledger add ").unwrap();
    text[at..]
        .split_whitespace()
        .skip(1)
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_probe_of_a_holder_the_unit_never_feeds_is_inconclusive_and_cannot_refute_the_leak() {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    // The capture names the holder; the finding at it has its owner, so a driver made for it
    // gets a probe of the holder.
    let s = capture_and_summarize(&r, Some("F1"));
    let args = add_args(&s);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let f = r.ok(&args)["id"].as_str().unwrap().to_owned();
    let h2 = r.driver(
        Some(&f),
        "    from shop import providers\n    _state[\"providers\"] = providers",
        "    _state[\"providers\"].make(i, True)",
    );
    r.ok(&["harness", "run", &h2, "--mode", "fresh"]);
    let a = r.ok(&["alloc-test", &h2, "--finding", &f]);
    assert_eq!(a["favors"], "inconclusive", "{a}");
    let basis = a["basis"].as_str().unwrap();
    assert!(
        basis.starts_with(&format!("this series does not show the growth of {f}: "))
            && basis.ends_with(&format!(
                " is flat from its first measured point, so the unit of {h2} does not feed it"
            )),
        "{basis}"
    );
    assert!(
        a["columns"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["plateau"]["verdict"] == "never-grew" || c["plateau"] == "never-grew"),
        "{a}"
    );
    let next = a["next"].as_array().unwrap();
    assert!(
        next[0].as_str().unwrap().starts_with(&format!(
            "dla alloc-test H1 --finding {f}   (the growth was measured there"
        )),
        "{next:?}"
    );
    let o = dla(
        &r.project,
        &[
            "ledger",
            "refute",
            &f,
            "--evidence",
            a["evidence"].as_str().unwrap(),
        ],
    );
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(state_of(&r, &f), "candidate");
}

#[test]
fn the_same_finding_on_the_harness_that_grew_is_a_leak_and_reaches_confirmed() {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    let f = finding_at_the_line(&r);
    let t = r.ok(&["trend", "S1"]);
    let e = t["evidence"][0].as_str().unwrap().to_owned();
    let sib = r.ok(&[
        "ledger",
        "add",
        "--sibling-of",
        &f,
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM",
        "--refute-if",
        "growth continues over the extra cycles",
        "--evidence",
        &e,
    ]);
    let sib = sib["id"].as_str().unwrap().to_owned();
    let workload = fs::read_to_string(r.run_dir.join("ledger.jsonl"))
        .unwrap()
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["kind"] == "workload")
        .filter_map(|v| v["evidence_id"].as_str().map(str::to_owned))
        .next_back()
        .unwrap();
    r.ok(&["ledger", "test", &f, "--evidence", &workload]);
    let a = r.ok(&["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "leak", "{a}");
    r.ok(&[
        "ledger",
        "refute",
        &sib,
        "--evidence",
        a["evidence"].as_str().unwrap(),
    ]);
    let c = r.ok(&["ledger", "confirm", &f, "--evidence", &e]);
    assert_eq!(c["to"], "confirmed", "{c}");
}

#[test]
fn a_real_warm_up_on_its_own_harness_is_still_non_leak_with_n_warm() {
    if !have_python() {
        return;
    }
    // The unit fills a cache to 2200 entries (the first four rounds after the warm-up round) and
    // then stops: the column grew in the main window and went flat.
    let r = prepared_with(
        "    _state[\"warm\"].fill(i)",
        SETUP_WARM,
        &[("shop/warm.py", WARM)],
    );
    let f = finding_at(&r, "shop/warm.py:6");
    let a = r.ok(&["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "non-leak", "{a}");
    let supports: Vec<&str> = a["supports"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(supports.contains(&"N-WARM"), "{a}");
}

#[test]
fn a_line_that_ran_and_kept_nothing_is_inconclusive_when_a_record_shows_a_project_holder_keeping_it()
 {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    let f = finding_at_the_line(&r);
    // The capture of H1 names the holder of the project that keeps the line.
    let s = capture_and_summarize(&r, Some("F1"));
    let record = s["evidence"][0].as_str().unwrap().to_owned();
    let h = r.driver(
        Some(&f),
        "    from shop import providers\n    _state[\"providers\"] = providers",
        "    _state[\"providers\"].make(i, False)\n    _state[\"providers\"].PROVIDER._items.clear()",
    );
    r.ok(&["harness", "run", &h, "--mode", "fresh"]);
    let a = r.ok(&["alloc-test", &h, "--finding", &f]);
    assert_eq!(a["favors"], "inconclusive", "{a}");
    let basis = a["basis"].as_str().unwrap();
    assert!(
        basis.starts_with("the line ran ")
            && basis.contains(&format!(
                "static:shop.providers.PROVIDER keeps {SITE} according to the retention record {record}"
            )),
        "{basis}"
    );
    assert!(
        a["next"][0].as_str().unwrap().starts_with(&format!(
            "dla alloc-test H1 --finding {f}   (the capture that shows the holder was made there)"
        )),
        "{}",
        a["next"]
    );
    let o = dla(
        &r.project,
        &[
            "ledger",
            "refute",
            &f,
            "--evidence",
            a["evidence"].as_str().unwrap(),
        ],
    );
    assert_ne!(o.code, 0, "{}", o.text);
}

#[test]
fn a_flat_series_on_the_harness_that_measured_the_growth_is_non_leak_and_says_it_did_not_come_back()
{
    if !have_python() {
        return;
    }
    // H1 grew the table of F1 (its probe). Its unit is changed so that it no longer feeds the
    // table, and the alloc-test on H1 for F1 reads the table flat: the growth was a growth of
    // every unit of H1, and it did not come back there.
    let r = prepared(KEEPS);
    let driver = r.run_dir.join("harness/H1/driver/dla_harness.py");
    let text = fs::read_to_string(&driver).unwrap();
    let old = "_state[\"cache\"].put(i)";
    assert_eq!(text.matches(old).count(), 1);
    fs::write(&driver, text.replace(old, "pass")).unwrap();
    let a = r.ok(&["alloc-test", "H1", "--finding", "F1"]);
    assert_eq!(a["favors"], "non-leak", "{a}");
    let basis = a["basis"].as_str().unwrap();
    assert!(
        basis.starts_with("the growth of F1 did not come back on the harness that measured it: ")
            && basis.ends_with("so it was not a growth of every unit"),
        "{basis}"
    );
    assert!(
        a["supports"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "N-WARM"),
        "{a}"
    );
}

#[test]
fn a_flat_series_on_an_entry_path_harness_of_the_finding_is_non_leak_as_before() {
    if !have_python() {
        return;
    }
    let r = prepared(KEEPS);
    let g = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--entry", "F1",
    ]);
    let h = g["summary"]["harness"].as_str().unwrap().to_owned();
    let edit = r.project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit)
        .unwrap()
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            "    from shop import providers\n    _state[\"providers\"] = providers",
        )
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            "    _state[\"providers\"].make(i, True)",
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    fs::write(
        r.run_dir.join(format!("harness/{h}/driver/VARIES.txt")),
        "the id of the object a caller asks the provider for\n",
    )
    .unwrap();
    r.ok(&["harness", "run", &h, "--mode", "fresh"]);
    // The entry-path harness asks whether a caller can feed the holder: a flat reading there keeps
    // its old meaning.
    let a = r.ok(&["alloc-test", &h, "--finding", "F1"]);
    assert_eq!(a["favors"], "non-leak", "{a}");
}
