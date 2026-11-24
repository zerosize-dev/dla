//! The trigger of a confirmed leak, through the binary, on a small Python project: a cache
//! keyed by an object the router creates for every request to an unknown path (a caller of the
//! public API makes it grow), and a table keyed by a constructor argument that only set-up code
//! passes (no caller does).
//!
//! A direct driver grows both. The entry-path driver the test writes grows the first only:
//! its trigger reads `reproduced`; the second reads `unchecked` after one entry-path harness
//! and `not_found` after a second, different one.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::{Value, json};

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn project(dir: &Path) {
    write(dir, "app/__init__.py", "");
    write(
        dir,
        "app/router.py",
        "import functools\n\n\nclass Handler:\n    \"\"\"The router makes a new one for every request to an unknown path.\"\"\"\n\n    def __init__(self, status):\n        self.status = status\n\n\n@functools.lru_cache(None)\ndef wrap(handler):\n    return (\"wrapped\", handler.status)\n\n\nclass Router:\n    def __init__(self, routes):\n        self.routes = routes\n\n    def dispatch(self, path):\n        h = self.routes.get(path)\n        if h is None:\n            h = Handler(404)\n        return wrap(h)\n",
    );
    write(
        dir,
        "app/sessions.py",
        "_by_name = {}\n\n\nclass Sessions:\n    def __init__(self, name):\n        _by_name[name] = self\n\n    def get(self, key):\n        return key\n",
    );
    write(
        dir,
        "app/main.py",
        "from app.router import Handler, Router\nfrom app.sessions import Sessions\n\n\ndef build():\n    router = Router({\"/a\": Handler(200)})\n    sessions = Sessions(\"main\")\n    return router, sessions\n\n\ndef serve(path):\n    router, sessions = build()\n    return router.dispatch(path), sessions.get(path)\n",
    );
}

struct Out {
    code: i32,
    json: Value,
    text: String,
}

struct Run {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

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
    c
}

impl Run {
    fn new(init_args: &[&str]) -> Self {
        Self::with_project(init_args, project)
    }

    fn with_project(init_args: &[&str], make: fn(&Path)) -> Self {
        let dir = tempfile::tempdir().unwrap();
        make(dir.path());
        let mut args = vec!["run", "init", "--session-id", "t"];
        args.extend_from_slice(init_args);
        let o = command(dir.path())
            .env("DLA_CONSENT_TIERS", "1,2,3")
            .args(&args)
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(v["ok"], true, "{v}");
        let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
        let r = Self { dir, run_dir };
        for a in [
            &["sweep", "--lang", "python"][..],
            &["census", "--lang", "python"][..],
            &["workload", "discover", "--lang", "python"][..],
        ] {
            r.ok(a);
        }
        r
    }

    fn run(&self, args: &[&str]) -> Out {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        Out {
            code: o.status.code().unwrap(),
            json: serde_json::from_str(&text).unwrap_or(Value::Null),
            text,
        }
    }

    fn ok(&self, args: &[&str]) -> Value {
        let o = self.run(args);
        assert_eq!(o.code, 0, "{args:?}: {}", o.text);
        o.json
    }

    /// The refusal of a command: its `error` object.
    fn refused(&self, args: &[&str]) -> Value {
        let o = self.run(args);
        assert_ne!(o.code, 0, "{args:?} was not refused: {}", o.text);
        o.json["error"].clone()
    }

    fn id_at(&self, site: &str) -> String {
        let o = command(self.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        text.lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .find(|l| l["site"].as_str() == Some(site))
            .unwrap_or_else(|| panic!("no finding at {site}: {text}"))["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn row(&self, id: &str) -> Value {
        let o = command(self.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        String::from_utf8(o.stdout)
            .unwrap()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .find(|l| l["id"].as_str() == Some(id))
            .unwrap()
    }

    fn driver(&self, h: &str) -> std::path::PathBuf {
        self.run_dir
            .join("harness")
            .join(h)
            .join("driver/dla_harness.py")
    }

    /// Fills the hooks of the driver of `h`.
    fn fill(&self, h: &str, setup: &str, unit: &str) {
        let p = self.driver(h);
        let text = fs::read_to_string(&p).unwrap();
        let text = text
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
                setup,
            )
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
                unit,
            );
        assert_ne!(text, fs::read_to_string(&p).unwrap());
        fs::write(p, text).unwrap();
    }

    fn varies(&self, h: &str, line: &str) {
        fs::write(
            self.run_dir
                .join("harness")
                .join(h)
                .join("driver/VARIES.txt"),
            format!("{line}\n"),
        )
        .unwrap();
    }

    /// The whole list of the final gate (`gate/final.json`, `missing`, in its own order), which
    /// `run status` cuts to what fits.
    fn gate_items(&self) -> Vec<Value> {
        let _ = self.run(&["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }

    fn status(&self) -> Value {
        self.ok(&["run", "status"])
    }

    fn report(&self) -> String {
        let o = command(self.dir.path()).arg("report").output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        // The command prints the report.
        text
    }

    /// The stop hook's reason, called as a hook process from another directory.
    fn stop_reason(&self) -> String {
        let away = tempfile::tempdir().unwrap();
        let mut c = Command::cargo_bin("dla").unwrap();
        for var in [
            "DLA_RUN",
            "DLA_WORKER",
            "CLAUDE_SESSION_ID",
            "CLAUDE_CODE_SESSION_ID",
        ] {
            c.env_remove(var);
        }
        let input = json!({
            "session_id": "0f7c2c1e-0000-4000-8000-000000000001",
            "transcript_path": "/tmp/t.jsonl",
            "cwd": self.dir.path(),
            "permission_mode": "default",
            "hook_event_name": "Stop",
            "stop_hook_active": false,
            "last_assistant_message": "done",
            "background_tasks": [],
            "session_crons": [],
        })
        .to_string();
        let o = c
            .current_dir(away.path())
            .args(["gate", "stop-hook"])
            .write_stdin(input)
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
        v["reason"].as_str().unwrap_or_default().to_owned()
    }

    /// Confirms `f`: the sibling, `alloc-test` for the discriminator, the refute and test
    /// commands it prints, and `confirm` citing the first trend evidence that passes.
    fn confirm(&self, f: &str, workload: &str, growth: &[Value]) {
        self.ok(&[
            "ledger",
            "add",
            "--sibling-of",
            f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            workload,
        ]);
        let a = self.ok(&["alloc-test", "H1", "--finding", f]);
        assert_eq!(a["favors"], "leak", "{a}");
        for cmd in a["next"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
        {
            if cmd.starts_with("dla ledger refute") || cmd.starts_with("dla ledger test") {
                let words: Vec<&str> = cmd.split_whitespace().skip(1).collect();
                self.ok(&words);
            }
        }
        let confirmed = growth
            .iter()
            .filter_map(Value::as_str)
            .any(|ev| self.run(&["ledger", "confirm", f, "--evidence", ev]).code == 0);
        assert!(confirmed, "{f} was not confirmed by {growth:?}");
    }
}

const CACHE_SETUP: &str = "    from app import router, sessions\n    _state['router'] = router\n    _state['sessions'] = sessions";
/// The direct driver: a new handler and a new constructor argument per unit.
const DIRECT_UNIT: &str = "    r, s = _state['router'], _state['sessions']\n    r.wrap(r.Handler(i))\n    s.Sessions('n%d' % key)";

const ENTRY_SETUP: &str = "    from app import router, sessions\n    _state['router'] = router.Router({'/a': router.Handler(200)})\n    _state['sessions'] = sessions.Sessions('main')";
/// Entry-path unit: an unknown path per unit (the router makes a handler for it).
const ENTRY_UNIT: &str =
    "    _state['router'].dispatch('/missing/' + str(key))\n    _state['sessions'].get(key)";
/// A second, different entry-path unit: known and unknown paths and lookups.
const ENTRY_UNIT_2: &str = "    _state['router'].dispatch('/a')\n    _state['sessions'].get('k%d' % key)\n    _state['sessions'].get(None)";

#[test]
fn the_trigger_is_reproduced_for_one_holder_and_not_found_for_the_other() {
    let r = Run::new(&["--budget", "3h"]);
    let (cache, table) = (r.id_at("app/router.py:11"), r.id_at("app/sessions.py:1"));

    // `--entry` asks for a finding that grew in some harness, a hook driver, at most 8.
    let e = r.refused(&[
        "harness", "gen", "--lang", "python", "--driver", "--entry", &cache,
    ]);
    assert_eq!(e["code"], "entry-not-grown", "{e}");
    assert!(
        e["fix"]
            .as_str()
            .unwrap()
            .contains(&format!("--probe {cache}")),
        "{e}"
    );
    let ids: Vec<String> = (1..=9).map(|n| format!("F{n}")).collect();
    let e = r.refused(&[
        "harness",
        "gen",
        "--lang",
        "python",
        "--driver",
        "--entry",
        &ids.join(","),
    ]);
    assert_eq!(e["code"], "too-many-targets", "{e}");
    let e = r.refused(&["harness", "gen", "--lang", "python", "--entry", &cache]);
    assert_eq!(e["code"], "invalid-value", "{e}");
    let e = r.refused(&[
        "harness", "gen", "--lang", "python", "--driver", "--entry", &cache, "--probe", &table,
    ]);
    assert_eq!(e["code"], "invalid-value", "{e}");
    assert!(
        !r.run_dir.join("harness").exists()
            || fs::read_dir(r.run_dir.join("harness")).unwrap().count() == 0
    );

    // The direct driver grows both holders.
    let g = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    let workload = g["evidence"][0].as_str().unwrap().to_owned();
    assert!(g["summary"].get("purpose").is_none(), "{g}");
    r.fill("H1", CACHE_SETUP, DIRECT_UNIT);
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let t = r.ok(&["trend", "S1"]);
    let verdicts: Vec<(&str, &str)> = t["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            (
                v["finding"].as_str().unwrap(),
                v["verdict"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        verdicts,
        [(cache.as_str(), "grew"), (table.as_str(), "grew")],
        "{t}"
    );
    assert!(t["targets"][0].get("trigger").is_none(), "{t}");
    let growth = t["evidence"].as_array().unwrap().clone();
    r.confirm(&cache, &workload, &growth);
    r.confirm(&table, &workload, &growth);

    // Confirmed on direct injection alone: the trigger is open, in every place that lists items.
    let st = r.status();
    assert_eq!(st["triggers"][&cache]["trigger"], "unchecked", "{st}");
    assert_eq!(st["triggers"][&table]["trigger"], "unchecked", "{st}");
    // The untied growing line of the harness has no retention record: its capture item leads, and
    // the trigger items follow it in the whole list (`run status` is cut to what fits).
    assert_eq!(st["blocking"][0]["kind"], "measured_capture", "{st}");
    let items = r.gate_items();
    let trigger = items
        .iter()
        .find(|m| m["kind"] == "trigger")
        .unwrap_or_else(|| panic!("{items:?}"));
    let fix = trigger["fix"].as_str().unwrap();
    assert!(
        fix.starts_with(&format!(
            "dla harness gen --lang python --driver --entry {cache},{table}, fill"
        )),
        "{fix}"
    );
    let _ = r.run(&["gate", "check", "--final"]);
    let full: Value =
        serde_json::from_str(&fs::read_to_string(r.run_dir.join("gate/final.json")).unwrap())
            .unwrap();
    let missing = full["missing"].as_array().unwrap();
    // The capture item leads; the two trigger items (one per confirmed finding) follow it.
    assert_eq!(missing[0]["kind"], "measured_capture", "{missing:?}");
    assert_eq!(missing[1]["kind"], "trigger", "{missing:?}");
    assert_eq!(missing[2]["kind"], "trigger", "{missing:?}");
    // `run status` clips long fixes; the gate file has them whole.
    assert!(
        missing[1]["fix"]
            .as_str()
            .unwrap()
            .starts_with(fix.trim_end_matches("...")),
        "{fix}"
    );
    let stop = r.stop_reason();
    assert!(
        stop.contains(&format!("{cache} grew when the harness fed it new keys; whether a caller of the public API can do that is not shown")),
        "{stop}"
    );
    assert_eq!(r.row(&cache)["trigger"], "unchecked");

    // The entry-path harness: its block, its refusal without a varies line, its run.
    let g = r.ok(&[
        "harness",
        "gen",
        "--lang",
        "python",
        "--driver",
        "--entry",
        &format!("{cache},{table}"),
    ]);
    assert_eq!(g["summary"]["purpose"], "entry-path", "{g}");
    let m: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H2/harness.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(m["purpose"], "entry-path");
    let block = fs::read_to_string(r.driver("H2")).unwrap();
    for want in [
        "Entry-path harness H2:",
        "writer function: app/router.py:12 (wrap)",
        "callers (matched by name",
        "hop 1: Router.dispatch [public] (app/router.py:24)",
        "hop 2: serve [public] (app/main.py:13)",
        "key from: parameter `handler` of wrap",
        "app/router.py:24 (Router.dispatch): handler = h",
        "h is set by: self.routes.get(path); Handler(404)",
        "writer function: app/sessions.py:5 (Sessions.__init__)",
        "hop 1: build [public] (app/main.py:7)",
        "key from: parameter `name` of Sessions.__init__",
        "app/main.py:7 (build): name = \"main\"",
        "Rules of an entry-path unit:",
        "call only the project's public entry points",
        "per unit vary only data that a caller or a remote peer controls",
        "never create new functions, classes, handlers, loggers or configuration per unit",
    ] {
        assert!(block.contains(want), "{want}\n{block}");
    }
    let varies = fs::read_to_string(r.run_dir.join("harness/H2/driver/VARIES.txt")).unwrap();
    assert!(varies.starts_with("DLA-VARIES-PLACEHOLDER"), "{varies}");
    r.fill("H2", ENTRY_SETUP, ENTRY_UNIT);
    let e = r.refused(&["harness", "run", "H2", "--mode", "both"]);
    assert_eq!(e["code"], "varies-missing", "{e}");
    assert!(e["fix"].as_str().unwrap().contains("VARIES.txt"), "{e}");
    // Item: the harness exists and has not run.
    let st = r.status();
    assert_eq!(st["blocking"][0]["kind"], "measured_capture", "{st}");
    let items = r.gate_items();
    assert!(
        items.iter().any(|m| m["fix"]
            .as_str()
            .unwrap()
            .starts_with("fill the hooks and VARIES.txt of H2")),
        "{items:?}"
    );
    let line = "the request path: a different unknown path per unit (the router rejects it)";
    r.varies("H2", line);
    let ran = r.ok(&["harness", "run", "H2", "--mode", "both"]);
    let first = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let record: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join(format!("harness/H2/run-{first}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(record["varies"], line);
    assert_eq!(record["purpose"], "entry-path");
    let t = r.ok(&["trend", &first]);
    let by: std::collections::BTreeMap<&str, (&str, &str)> = t["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            (
                v["finding"].as_str().unwrap(),
                (
                    v["verdict"].as_str().unwrap(),
                    v["trigger"].as_str().unwrap(),
                ),
            )
        })
        .collect();
    assert_eq!(by[cache.as_str()], ("grew", "reproduced"), "{t}");
    assert_eq!(by[table.as_str()], ("constant", "unchecked"), "{t}");
    assert!(
        t["trigger_next"]
            .as_str()
            .unwrap()
            .contains(&format!("--entry {table}")),
        "{t}"
    );

    // One entry-path harness without growth: the item asks for a second, different one.
    let st = r.status();
    assert_eq!(st["triggers"][&cache]["trigger"], "reproduced", "{st}");
    assert_eq!(st["triggers"][&cache]["harness"], "H2");
    assert_eq!(st["triggers"][&cache]["varies"], line);
    assert_eq!(st["triggers"][&table]["trigger"], "unchecked");
    assert_eq!(st["blocking"][0]["kind"], "measured_capture", "{st}");
    let items = r.gate_items();
    let trigger = items
        .iter()
        .find(|m| m["kind"] == "trigger")
        .unwrap_or_else(|| panic!("{items:?}"));
    assert_eq!(trigger["id"], table);
    assert!(
        trigger["fix"]
            .as_str()
            .unwrap()
            .starts_with("write a second, different entry-path harness"),
        "{trigger}"
    );
    assert!(
        r.stop_reason()
            .contains("did not grow in the entry-path harness H2")
    );
    assert_eq!(r.row(&cache)["trigger"], "reproduced");
    let md = r.report();
    assert!(
        md.contains(&format!(
            "- Trigger: reproduced through the public API in H2 (varies: {line})"
        )),
        "{md}"
    );
    assert!(
        md.contains("- Trigger: not checked (the entry-path harness H2 did not grow it"),
        "{md}"
    );

    // A second entry-path harness with another driver: no trigger found, the item is gone.
    let g = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--entry", &table,
    ]);
    assert_eq!(g["summary"]["harness"], "H3", "{g}");
    r.fill("H3", ENTRY_SETUP, ENTRY_UNIT_2);
    r.varies(
        "H3",
        "known and unknown session keys, and a missing key (None)",
    );
    let ran = r.ok(&["harness", "run", "H3", "--mode", "both"]);
    let t = r.ok(&["trend", ran["summary"]["series"][0].as_str().unwrap()]);
    assert_eq!(t["targets"][0]["trigger"], "not_found", "{t}");
    let st = r.status();
    assert_eq!(st["triggers"][&table]["trigger"], "not_found", "{st}");
    assert!(
        st["blocking"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["kind"] != "trigger"),
        "{st}"
    );
    assert_eq!(r.row(&table)["trigger"], "not_found");
    let md = r.report();
    assert!(
        md.contains("- Trigger: not found: it grows when the harness passes new keys to Sessions.__init__; two workloads through the public API did not grow it (H2: the request path: a different unknown path per unit (the router rejects it); H3: known and unknown session keys, and a missing key (None))"),
        "{md}"
    );
    let findings = md.lines().find(|l| l.starts_with("Findings:")).unwrap();
    assert!(
        findings.ends_with(
            "reproduced through the public API 1, no trigger found 1, trigger not checked 0"
        ),
        "{findings}"
    );
    // The reproduced one is listed first.
    let order: Vec<&str> = md.lines().filter(|l| l.starts_with("### F")).collect();
    assert!(order[0].starts_with(&format!("### {cache}:")), "{order:?}");

    // The bundle of the reproduced leak holds the driver without the generated target block.
    let b = r.ok(&["bundle", &cache]);
    let dir = r.dir.path().join(b["dir"].as_str().unwrap());
    let driver = fs::read_to_string(dir.join("entry-path/H2/dla_harness.py")).unwrap();
    assert!(driver.contains("dispatch('/missing/'"), "{driver}");
    assert!(
        driver.contains("the generated target block is not part of the bundle"),
        "{driver}"
    );
    for hidden in [
        "Entry-path harness",
        "app/router.py:11",
        "writer function",
        "Targets of harness",
    ] {
        assert!(!driver.contains(hidden), "{hidden} in the bundled driver");
    }
    assert_eq!(
        fs::read_to_string(dir.join("entry-path/H2/VARIES.txt"))
            .unwrap()
            .trim(),
        line
    );
    let verdict: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("entry-path/H2/verdict.json")).unwrap())
            .unwrap();
    assert_eq!(verdict["verdict"], "grew");
    assert_eq!(verdict["varies"], line);
    let index: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("index.json")).unwrap()).unwrap();
    assert_eq!(index["entry_path"][0]["harness"], "H2");
    // The bundle of the other one holds both entry-path harnesses, each without growth.
    let b = r.ok(&["bundle", &table]);
    let dir = r.dir.path().join(b["dir"].as_str().unwrap());
    for h in ["H2", "H3"] {
        let v: Value = serde_json::from_str(
            &fs::read_to_string(dir.join(format!("entry-path/{h}/verdict.json"))).unwrap(),
        )
        .unwrap();
        assert_ne!(v["verdict"], "grew", "{h}");
    }
}

#[test]
fn with_the_queue_off_the_trigger_is_not_asked_and_the_report_says_so() {
    let r = Run::new(&["--budget", "3h", "--queue", "off"]);
    let (cache, table) = (r.id_at("app/router.py:11"), r.id_at("app/sessions.py:1"));
    let g = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--probe", &cache, "--probe", &table,
    ]);
    let workload = g["evidence"][0].as_str().unwrap().to_owned();
    r.fill("H1", CACHE_SETUP, DIRECT_UNIT);
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let t = r.ok(&["trend", "S1"]);
    let growth = t["evidence"].as_array().unwrap().clone();
    r.confirm(&cache, &workload, &growth);
    let st = r.status();
    assert!(
        st["blocking"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["kind"] != "trigger"),
        "{st}"
    );
    assert_eq!(st["triggers"][&cache]["trigger"], "unchecked");
    assert_eq!(
        st["triggers"][&cache]["not_checked"],
        "the run was started with --queue off"
    );
    let md = r.report();
    assert!(
        md.contains("- Trigger: not checked (the run was started with --queue off)"),
        "{md}"
    );
    // `--entry` still works when asked for.
    r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--entry", &cache,
    ]);
}

/// The manifest of `h` turned into a test loop (the tool a JUnit repeat lists), with the
/// `tests.json` that names a class for `cache` and another for `table`, and one for both.
fn make_test_loop(r: &Run, h: &str, cache: &str, table: &str) {
    let dir = r.run_dir.join("harness").join(h);
    let p = dir.join("harness.json");
    let mut m: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    m["tools"] = json!(["junit-platform-launcher"]);
    fs::write(&p, m.to_string()).unwrap();
    let doc = json!({"classes": [
        {"class": "org.acme.other.OtherUnitTests", "file": "src/test/java/org/acme/other/OtherUnitTests.java",
         "tests": 3, "for": [{"by": "name", "finding": table}]},
        {"class": "org.acme.router.RouterUnitTests", "file": "src/test/java/org/acme/router/RouterUnitTests.java",
         "tests": 7, "for": [{"by": "name", "finding": cache}]},
        {"class": "org.acme.router.KRouterUnitTests", "file": "src/test/kotlin/org/acme/router/KRouterUnitTests.kt",
         "tests": 4, "for": [{"by": "mention", "finding": cache}, {"by": "name", "finding": table}]},
    ]});
    fs::write(dir.join("tests.json"), doc.to_string()).unwrap();
}

/// Writes what the loop of `h` measured per class for `finding`.
fn write_attribution(r: &Run, h: &str, finding: &str, classes: &[(&str, i64, i64)]) {
    let doc = json!({
        "probe": "org.acme.router.Router#cache",
        "later_runs": 20,
        "classes": classes.iter().map(|(c, first, later)| json!({
            "class": c, "first_run": first, "later_runs": later,
        })).collect::<Vec<_>>(),
        "dropped": [],
    });
    fs::write(
        r.run_dir
            .join("harness")
            .join(h)
            .join(format!("attribution-{finding}.json")),
        doc.to_string(),
    )
    .unwrap();
}

/// The manifest of `h` turned into a pytest loop (its tool), with the file the loop records of the
/// test files it ran and dropped.
fn make_pytest_loop(r: &Run, h: &str, ran: &[&str], dropped: &[&str]) {
    let dir = r.run_dir.join("harness").join(h);
    let p = dir.join("harness.json");
    let mut m: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    m["tools"] = json!(["pytest-loop"]);
    fs::write(&p, m.to_string()).unwrap();
    let out = dir.join("out/p1-fresh");
    fs::create_dir_all(&out).unwrap();
    fs::write(
        out.join("pytest-files.json"),
        json!({"files": ran, "dropped": dropped}).to_string(),
    )
    .unwrap();
}

#[test]
fn the_bundle_and_the_trigger_of_a_pytest_loop_finding_list_the_test_files_and_measure_none() {
    let r = Run::new(&["--budget", "3h", "--queue", "off"]);
    let (cache, table) = (r.id_at("app/router.py:11"), r.id_at("app/sessions.py:1"));
    let g = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--probe", &cache, "--probe", &table,
    ]);
    let workload = g["evidence"][0].as_str().unwrap().to_owned();
    r.fill("H1", CACHE_SETUP, DIRECT_UNIT);
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let t = r.ok(&["trend", "S1"]);
    let growth = t["evidence"].as_array().unwrap().clone();
    r.confirm(&cache, &workload, &growth);
    make_pytest_loop(
        &r,
        "H1",
        &["tests/test_router.py", "tests/test_sessions.py"],
        &["tests/test_broken.py"],
    );
    let b = r.ok(&["bundle", &cache]);
    let dir = r.dir.path().join(b["dir"].as_str().unwrap());
    let doc: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("test-loop/H1/tests.json")).unwrap())
            .unwrap();
    assert_eq!(doc["harness"], "H1", "{doc}");
    assert_eq!(
        doc["test_files_run"],
        json!(["tests/test_router.py", "tests/test_sessions.py"]),
        "{doc}"
    );
    assert_eq!(
        doc["dropped_files"],
        json!(["tests/test_broken.py"]),
        "{doc}"
    );
    // No per-file figure and no class list: nothing measures which file feeds the holder.
    for absent in ["fed_by", "classes_run", "ran_and_added_nothing", "per_run"] {
        assert!(doc.get(absent).is_none(), "{absent} in {doc}");
    }
    let note = doc["note"].as_str().unwrap();
    assert!(
        note.contains("pytest") && note.contains("not measured"),
        "{note}"
    );
    let index: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("index.json")).unwrap()).unwrap();
    assert_eq!(
        index["test_loop"][0]["files"][0], "test-loop/H1/tests.json",
        "{index}"
    );
    // The trigger line: the project's own tests, and no alloc-test hint (it measures JVM classes).
    let md = r.report();
    assert!(
        md.contains(
            "reproduced through the public API in H1 (varies: the project's own tests in H1; which test file feeds it is not measured)"
        ),
        "{md}"
    );
    assert!(
        !md.contains(&format!("dla alloc-test H1 --finding {cache}")),
        "{md}"
    );
    // The loop recorded no files: the bundle says so and still writes the file.
    fs::remove_file(r.run_dir.join("harness/H1/out/p1-fresh/pytest-files.json")).unwrap();
    let b = r.ok(&["bundle", &cache]);
    let dir = r.dir.path().join(b["dir"].as_str().unwrap());
    let doc: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("test-loop/H1/tests.json")).unwrap())
            .unwrap();
    assert_eq!(doc["test_files_run"], json!([]), "{doc}");
    assert!(
        doc["note"].as_str().unwrap().contains("were not recorded"),
        "{doc}"
    );
}

#[test]
fn the_bundle_and_the_trigger_of_a_test_loop_finding_name_what_was_measured_to_feed_it() {
    let r = Run::new(&["--budget", "3h", "--queue", "off"]);
    let (cache, table) = (r.id_at("app/router.py:11"), r.id_at("app/sessions.py:1"));
    let g = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--probe", &cache, "--probe", &table,
    ]);
    let workload = g["evidence"][0].as_str().unwrap().to_owned();
    r.fill("H1", CACHE_SETUP, DIRECT_UNIT);
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let t = r.ok(&["trend", "S1"]);
    let growth = t["evidence"].as_array().unwrap().clone();
    r.confirm(&cache, &workload, &growth);
    // Before the harness is a test loop, the bundle has no `test-loop/`.
    let b = r.ok(&["bundle", &cache]);
    let dir = r.dir.path().join(b["dir"].as_str().unwrap());
    assert!(!dir.join("test-loop").exists());
    let index: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("index.json")).unwrap()).unwrap();
    assert!(index.get("test_loop").is_none(), "{index}");
    let tests_of = |b: &Value| -> (Value, String) {
        let dir = r.dir.path().join(b["dir"].as_str().unwrap());
        let text = fs::read_to_string(dir.join("test-loop/H1/tests.json")).unwrap();
        (serde_json::from_str(&text).unwrap(), text)
    };

    // No attribution: every class that ran is listed, and the bundle says which of them feeds the
    // holder was not measured. A class chosen for the finding by name is not named as its feeder.
    make_test_loop(&r, "H1", &cache, &table);
    let b = r.ok(&["bundle", &cache]);
    let (doc, text) = tests_of(&b);
    assert_eq!(doc["harness"], "H1", "{doc}");
    assert!(doc.get("fed_by").is_none(), "{doc}");
    let run: Vec<(&str, Option<&str>)> = doc["classes_run"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["class"].as_str().unwrap(), c["file"].as_str()))
        .collect();
    assert_eq!(
        run,
        [
            (
                "org.acme.other.OtherUnitTests",
                Some("src/test/java/org/acme/other/OtherUnitTests.java")
            ),
            (
                "org.acme.router.RouterUnitTests",
                Some("src/test/java/org/acme/router/RouterUnitTests.java")
            ),
            (
                "org.acme.router.KRouterUnitTests",
                Some("src/test/kotlin/org/acme/router/KRouterUnitTests.kt")
            ),
        ],
        "{doc}"
    );
    let note = doc["note"].as_str().unwrap();
    assert!(note.contains("not measured"), "{note}");
    assert!(!text.contains("\"by\""), "{text}");
    let dir = r.dir.path().join(b["dir"].as_str().unwrap());
    let index: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("index.json")).unwrap()).unwrap();
    assert_eq!(index["test_loop"][0]["harness"], "H1", "{index}");
    assert_eq!(
        index["test_loop"][0]["files"][0], "test-loop/H1/tests.json",
        "{index}"
    );
    // The file is part of the manifest.
    let manifest = fs::read_to_string(dir.join("manifest.json")).unwrap();
    assert!(manifest.contains("test-loop/H1/tests.json"), "{manifest}");
    let md = r.report();
    assert!(
        md.contains(&format!(
            "reproduced through the public API in H1 (varies: the project's own tests in H1; which test class feeds it is not measured (dla alloc-test H1 --finding {cache}))"
        )),
        "{md}"
    );
    assert!(!md.contains("RouterUnitTests"), "{md}");

    // Measured: the class that fed it, with its figure, and the ones that ran and added nothing.
    write_attribution(
        &r,
        "H1",
        &cache,
        &[
            ("org.acme.other.OtherUnitTests", 267, 0),
            ("org.acme.router.RouterUnitTests", 0, 0),
            ("org.acme.router.KRouterUnitTests", 2, 266),
        ],
    );
    let b = r.ok(&["bundle", &cache]);
    let (doc, text) = tests_of(&b);
    assert!(doc.get("classes_run").is_none(), "{doc}");
    assert_eq!(doc["fed_by"].as_array().unwrap().len(), 1, "{doc}");
    assert_eq!(
        doc["fed_by"][0]["class"],
        "org.acme.router.KRouterUnitTests"
    );
    assert_eq!(
        doc["fed_by"][0]["file"],
        "src/test/kotlin/org/acme/router/KRouterUnitTests.kt"
    );
    assert_eq!(doc["fed_by"][0]["per_run"], 13.3, "{doc}");
    let flat: Vec<&str> = doc["ran_and_added_nothing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["class"].as_str().unwrap())
        .collect();
    assert_eq!(
        flat,
        [
            "org.acme.other.OtherUnitTests",
            "org.acme.router.RouterUnitTests"
        ],
        "{doc}"
    );
    assert_eq!(
        doc["ran_and_added_nothing"][1]["file"],
        "src/test/java/org/acme/router/RouterUnitTests.java"
    );
    assert!(
        doc["note"]
            .as_str()
            .unwrap()
            .contains("measured per test class"),
        "{doc}"
    );
    assert!(!text.contains("\"by\""), "{text}");
    let md = r.report();
    assert!(
        md.contains(
            "reproduced through the public API in H1 (varies: the project's own tests KRouterUnitTests (13.3 entries per run))"
        ),
        "{md}"
    );

    // Measured and nothing grew per class.
    write_attribution(
        &r,
        "H1",
        &cache,
        &[
            ("org.acme.other.OtherUnitTests", 267, 0),
            ("org.acme.router.RouterUnitTests", 0, 0),
        ],
    );
    let b = r.ok(&["bundle", &cache]);
    let (doc, _) = tests_of(&b);
    assert_eq!(doc["fed_by"].as_array().unwrap().len(), 0, "{doc}");
    assert_eq!(
        doc["ran_and_added_nothing"].as_array().unwrap().len(),
        2,
        "{doc}"
    );
    let md = r.report();
    assert!(
        md.contains(
            "(varies: the project's own tests; no single test class added entries per run in H1)"
        ),
        "{md}"
    );
}

/// A package whose plugin modules register codecs at their top level, through a wrapper.
fn registry_project(dir: &Path) {
    write(dir, "pkg/__init__.py", "");
    write(
        dir,
        "pkg/core.py",
        "CODECS = {}\n\n\ndef register_codec(id, ext):\n    CODECS[ext.lower()] = id\n\n\ndef register_codecs(id, exts):\n    for ext in exts:\n        register_codec(id, ext)\n\n\ndef lookup(ext):\n    return CODECS.get(ext.lower())\n",
    );
    write(
        dir,
        "pkg/plugin_a.py",
        "from . import core\n\ncore.register_codecs(\"A\", [\".a\", \".aa\"])\n",
    );
}

/// The same package plus a function a caller uses to add a plugin at run time.
fn registry_project_with_a_run_time_entry(dir: &Path) {
    registry_project(dir);
    write(
        dir,
        "pkg/api.py",
        "from . import core\n\n\ndef add_plugin(name, exts):\n    core.register_codecs(name, exts)\n",
    );
}

const REGISTRY_CORE_SETUP: &str = "    from pkg import core\n    _state['core'] = core";
/// An entry-path unit that only reads the table: it never registers.
const REGISTRY_LOOKUP_UNIT: &str = "    _state['core'].lookup('.u%d' % key)";
const REGISTRY_SETUP: &str =
    "    from pkg import api, core\n    _state['core'] = core\n    _state['api'] = api";
/// A hook driver that registers a new extension through the wrapper, per unit.
const REGISTRY_UNIT: &str = "    _state['core'].register_codecs('x', ['.u%d' % key])";
/// An entry-path unit: a caller adds a plugin through the public function.
const REGISTRY_ENTRY_UNIT: &str = "    _state['api'].add_plugin('p%d' % key, ['.p%d' % key])";

#[test]
fn a_registry_grown_only_by_its_own_writer_is_not_confirmed_and_the_non_leak_way_closes_it() {
    let r = Run::with_project(&["--budget", "3h"], registry_project);
    let f = r.id_at("pkg/core.py:1");
    let g = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    let workload = g["evidence"][0].as_str().unwrap().to_owned();
    r.fill("H1", REGISTRY_CORE_SETUP, REGISTRY_UNIT);
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let t = r.ok(&["trend", "S1"]);
    let growth = t["evidence"].as_array().unwrap().clone();
    let ev = growth[0].as_str().unwrap().to_owned();
    assert_eq!(t["targets"][0]["verdict"], "grew", "{t}");

    // The alloc-test of the direct driver does not offer the confirm that would be refused.
    let a = r.ok(&["alloc-test", "H1", "--finding", &f]);
    let next: Vec<&str> = a["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        !next.iter().any(|c| c.starts_with("dla ledger confirm")),
        "{next:?}"
    );
    assert!(
        next.iter().any(|c| c.starts_with(&format!(
            "dla harness gen --lang python --driver --entry {f}"
        ))) && next
            .iter()
            .any(|c| c.starts_with(&format!("dla alloc-test <H> --finding {f}")))
            && !next.iter().any(|c| c.contains("--hypothesis N-INTENT")),
        "{next:?}"
    );

    // The census reads the writer as a start-up registration through the wrapper, and the only
    // growth measured is the unit's own call of that wrapper.
    let e = r.refused(&["ledger", "confirm", &f, "--evidence", &ev]);
    assert_eq!(e["code"], "registration-only", "{e}");
    let message = e["message"].as_str().unwrap();
    for want in [
        "start-up registration (register_codec)",
        "(H1)",
        "register_codecs itself with new keys",
    ] {
        assert!(message.contains(want), "{want}: {message}");
    }
    let fix = e["fix"].as_str().unwrap().to_owned();
    // One route: an entry-path harness whose unit does not call the registration function.
    for want in [
        &format!("dla harness gen --lang python --driver --entry {f}"),
        &format!("dla alloc-test <H> --finding {f}"),
        "dla ledger confirm F",
        "--hypothesis N-INTENT",
        "<the sibling id the add prints>",
    ] {
        let want = if want == "dla ledger confirm F" {
            format!("dla ledger confirm {f} again")
        } else {
            want.to_owned()
        };
        assert!(fix.contains(&want), "{want}: {fix}");
    }
    assert!(!fix.contains(&format!("{f}.n")), "{fix}");

    // An earlier sibling (the warm-up one) exists: the N-INTENT sibling gets the next id.
    r.ok(&[
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
        &workload,
    ]);

    // The entry-path harness: a unit that uses the public API without registering anything.
    let g = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--entry", &f,
    ]);
    assert_eq!(g["summary"]["purpose"], "entry-path", "{g}");
    let h = g["summary"]["harness"].as_str().unwrap().to_owned();
    r.fill(&h, REGISTRY_CORE_SETUP, REGISTRY_LOOKUP_UNIT);
    r.varies(&h, "the extension a caller looks up");
    let ran = r.ok(&["harness", "run", &h, "--mode", "both"]);
    let series = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let t2 = r.ok(&["trend", &series]);
    assert_eq!(t2["targets"][0]["verdict"], "constant", "{t2}");
    assert_eq!(t2["targets"][0]["trigger"], "direct", "{t2}");
    let trend_ev = t2["evidence"][0].as_str().unwrap().to_owned();
    let a = r.ok(&["alloc-test", &h, "--finding", &f]);
    assert_eq!(a["favors"], "non-leak", "{a}");
    let alloc_ev = a["evidence"].as_str().unwrap().to_owned();
    let refute = a["next"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|c| c.starts_with(&format!("dla ledger refute {f} ")))
        .unwrap_or_else(|| panic!("no refute of {f}: {a}"));
    let words: Vec<&str> = refute.split_whitespace().skip(1).collect();
    r.ok(&words);
    assert_eq!(r.row(&f)["state"], "refuted");

    // Record the reading: the sibling the add prints, tested and confirmed with the same evidence.
    let add = r.ok(&[
        "ledger",
        "add",
        "--sibling-of",
        &f,
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-INTENT",
        "--refute-if",
        "run-time input reaches the registration call",
        "--evidence",
        &trend_ev,
    ]);
    let sibling = add["id"].as_str().unwrap().to_owned();
    assert_eq!(sibling, format!("{f}.n2"), "{add}");
    r.ok(&["ledger", "test", &sibling, "--evidence", &workload]);
    r.ok(&["ledger", "confirm", &sibling, "--evidence", &alloc_ev]);
    assert_eq!(r.row(&sibling)["state"], "confirmed");
}

#[test]
fn a_registry_an_entry_path_harness_grew_without_the_direct_call_is_confirmed_as_before() {
    let r = Run::with_project(&["--budget", "3h"], registry_project_with_a_run_time_entry);
    let f = r.id_at("pkg/core.py:1");
    let g = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    let workload = g["evidence"][0].as_str().unwrap().to_owned();
    r.fill("H1", REGISTRY_SETUP, REGISTRY_UNIT);
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let t = r.ok(&["trend", "S1"]);
    let growth = t["evidence"].as_array().unwrap().clone();
    // A run-time function of the project (`add_plugin`) calls the wrapper: the census does not
    // read the writer as a start-up registration, so the direct driver is not refused on that.
    let g = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--entry", &f,
    ]);
    assert_eq!(g["summary"]["purpose"], "entry-path", "{g}");
    r.fill("H2", REGISTRY_SETUP, REGISTRY_ENTRY_UNIT);
    r.varies(
        "H2",
        "the plugin name and extension a caller of add_plugin passes",
    );
    let ran = r.ok(&["harness", "run", "H2", "--mode", "both"]);
    let t2 = r.ok(&["trend", ran["summary"]["series"][0].as_str().unwrap()]);
    assert_eq!(t2["targets"][0]["verdict"], "grew", "{t2}");
    assert_eq!(t2["targets"][0]["trigger"], "reproduced", "{t2}");
    r.confirm(&f, &workload, &growth);
    assert_eq!(r.row(&f)["state"], "confirmed");
}

/// One registration function that fills two module-level tables.
fn two_table_project(dir: &Path) {
    write(dir, "pkg/__init__.py", "");
    write(
        dir,
        "pkg/core.py",
        "CODECS = {}\nMIMES = {}\n\n\ndef register_codec(id, ext):\n    CODECS[ext.lower()] = id\n    MIMES[ext.lower()] = id\n",
    );
    write(
        dir,
        "pkg/plugin_a.py",
        "from . import core\n\ncore.register_codec(\"A\", \".a\")\n",
    );
}

#[test]
fn every_table_a_hook_drivers_registration_call_grows_is_direct_not_only_its_target() {
    let r = Run::with_project(&["--budget", "3h"], two_table_project);
    let (codecs, mimes) = (r.id_at("pkg/core.py:1"), r.id_at("pkg/core.py:2"));
    // The driver is generated for one table only; nothing is edited by hand.
    r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--probe", &codecs,
    ]);
    r.fill(
        "H1",
        REGISTRY_CORE_SETUP,
        "    _state['core'].register_codec('x', '.u%d' % key)",
    );
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let t = r.ok(&["trend", "S1"]);
    let ev = t["evidence"][0].as_str().unwrap().to_owned();
    let targets: Vec<&str> = t["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["finding"].as_str().unwrap())
        .collect();
    assert_eq!(targets, [codecs.as_str()], "{t}");
    // The other table is no target, but its probe column passes and is tied to it.
    assert!(
        t["passed"].as_array().unwrap().iter().any(|c| {
            c["candidates"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == mimes.as_str()))
        }),
        "{t}"
    );
    r.ok(&[
        "ledger",
        "add",
        "--sibling-of",
        &mimes,
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM",
        "--refute-if",
        "growth continues over the extra cycles",
        "--evidence",
        &ev,
    ]);
    let a = r.ok(&["alloc-test", "H1", "--finding", &mimes]);
    assert!(
        !a["next"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .any(|c| c.starts_with("dla ledger confirm")),
        "{a}"
    );
    for id in [&codecs, &mimes] {
        let e = r.refused(&["ledger", "confirm", id, "--evidence", &ev]);
        assert_eq!(e["code"], "registration-only", "{id}: {e}");
    }
}
