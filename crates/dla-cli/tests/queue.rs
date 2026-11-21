//! The measurement queue, through the binary, on a small Python project: a cached function
//! keyed by its argument, a class-level dict with a writer and no remover, a dict with a
//! writer and a remover, and a candidate in a test file.
//!
//! A candidate ranked for measurement cannot be closed as not exercised until a workload
//! has reached it; `dla harness gen --driver --next` targets the next batch, `dla trend`
//! gives each target a verdict, and `dla ledger close-rest` closes what is left.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn project(dir: &Path) {
    write(dir, "shop/__init__.py", "");
    write(
        dir,
        "shop/cache.py",
        "import functools\n\n\n@functools.lru_cache(None)\ndef price_for(handler, key):\n    return (handler, key)\n",
    );
    write(
        dir,
        "shop/reg.py",
        "class Registry:\n    _by_key = {}\n\n    def put(self, key, value):\n        Registry._by_key[key] = value\n\n\nclass Bounded:\n    _recent = {}\n\n    def put(self, key, value):\n        Bounded._recent[key] = value\n        if len(Bounded._recent) > 10:\n            Bounded._recent.pop(next(iter(Bounded._recent)))\n",
    );
    write(
        dir,
        "tests/test_cache.py",
        "import functools\n\n\n@functools.lru_cache(None)\ndef helper(x):\n    return x\n",
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
    /// Project, `run init` with `init_args`, consent, sweep, census and workload discovery.
    fn new(init_args: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path());
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

    /// `ledger resolve F --as unresolved --reason r --remedy x`.
    fn close(&self, id: &str, reason: &str) -> Out {
        self.run(&[
            "ledger",
            "resolve",
            id,
            "--as",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "x",
        ])
    }

    /// The lines of `dla ledger list --all`: the totals, then one object per finding.
    fn list(&self) -> (Value, Vec<Value>) {
        let o = command(self.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        assert!(o.status.success());
        let text = String::from_utf8(o.stdout).unwrap();
        let mut lines: Vec<Value> = text
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        let totals = lines.remove(0);
        (totals, lines)
    }

    /// The markdown of `dla report`.
    fn report(&self) -> String {
        let o = command(self.dir.path()).arg("report").output().unwrap();
        String::from_utf8(o.stdout).unwrap()
    }

    fn id_at(&self, site: &str) -> String {
        let (_, lines) = self.list();
        lines
            .iter()
            .find(|l| l["site"].as_str() == Some(site))
            .unwrap_or_else(|| panic!("no finding at {site}: {lines:?}"))["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    /// The items of `gate check --final`, from the full file.
    fn missing(&self) -> Vec<Value> {
        let _ = self.run(&["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }

    fn driver(&self, h: &str) -> std::path::PathBuf {
        self.run_dir
            .join("harness")
            .join(h)
            .join("driver/dla_harness.py")
    }

    /// Fills the hooks: `setup` imports both modules, `unit` calls the cached function with
    /// a new key per unit and `extra` more.
    fn fill(&self, h: &str, extra: &str) {
        let p = self.driver(h);
        let text = fs::read_to_string(&p).unwrap();
        let text = text
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
                "    from shop import cache, reg\n    _state['cache'] = cache\n    _state['reg'] = reg",
            )
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
                &format!("    _state['cache'].price_for('h', key)\n{extra}"),
            );
        fs::write(p, text).unwrap();
    }
}

impl Run {
    /// Changes the driver without changing what it does.
    fn fill_more(&self, h: &str) {
        let p = self.driver(h);
        let text = fs::read_to_string(&p).unwrap();
        fs::write(
            &p,
            text.replace(
                "def unit(i, key):",
                "def unit(i, key):\n    # a second attempt",
            ),
        )
        .unwrap();
    }
}

const BATCH: &str = "dla harness gen --lang python --driver --next";

#[test]
fn a_candidate_cannot_be_closed_unmeasured_and_the_gate_and_status_lead_with_the_queue() {
    let r = Run::new(&["--budget", "3h"]);
    let (totals, lines) = r.list();
    let order: Vec<&str> = lines.iter().map(|l| l["site"].as_str().unwrap()).collect();
    // Tier 1 (a DLA rule joined to the census entry) first, in ledger order; the dict with
    // a remover (tier 4) after them.
    assert_eq!(
        order,
        ["shop/cache.py:4", "shop/reg.py:2", "shop/reg.py:9"],
        "{lines:?}"
    );
    assert_eq!(lines[0]["tier"], 1);
    assert_eq!(lines[2]["tier"], 4);
    assert_eq!(lines[0]["rank"], 1);
    assert_eq!(lines[2]["rank"], 3);
    assert_eq!(lines[0]["queue"], "pending");
    assert_eq!(totals["totals"]["queue"]["python"]["pending"], 3);

    let bounded = r.id_at("shop/reg.py:9");
    for reason in ["not_exercised", "no_harness", "time_budget"] {
        let o = r.close(&bounded, reason);
        assert_ne!(o.code, 0, "{reason}: {}", o.text);
        assert_eq!(o.json["error"]["code"], "target-pending", "{}", o.text);
        let fix = o.json["error"]["fix"].as_str().unwrap();
        assert!(fix.starts_with(BATCH), "{fix}");
        assert!(fix.contains("dla trend S<n>"), "{fix}");
        assert!(fix.contains("dla ledger resolve intake.workload"), "{fix}");
        let msg = o.json["error"]["message"].as_str().unwrap();
        assert!(msg.contains("ranked 3 (tier 4)"), "{msg}");
        assert!(msg.contains("of 180 budget minutes remain"), "{msg}");
    }
    // A blocker reason is not refused.
    let ok = r.close(&bounded, "build_fails");
    assert_eq!(ok.code, 0, "{}", ok.text);

    // The status and the final gate lead with one queue item and say the batch command.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["blocking"][0]["kind"], "queue", "{st}");
    assert_eq!(st["next_command"], BATCH, "{st}");
    let q = &st["queue"]["python"];
    assert_eq!(q["open"], true, "{st}");
    assert_eq!(q["pending"], 2, "{st}");
    assert_eq!(q["next"].as_array().unwrap().len(), 2);
    let missing = r.missing();
    assert_eq!(missing[0]["kind"], "queue", "{missing:?}");
    assert!(
        missing[0]["description"]
            .as_str()
            .unwrap()
            .contains("candidate(s) of python are ranked for measurement and not targeted yet"),
        "{missing:?}"
    );
    assert!(missing[0]["fix"].as_str().unwrap().starts_with(BATCH));
    // The item of a pending candidate carries the same batch command, never the close.
    let item = missing.iter().find(|m| m["kind"] == "finding").unwrap();
    assert!(item["fix"].as_str().unwrap().starts_with(BATCH), "{item}");
    assert!(!item["fix"].as_str().unwrap().contains("ledger resolve"));
}

#[test]
fn a_workload_answer_that_is_not_a_blocker_leaves_the_queue_open() {
    let r = Run::new(&["--budget", "3h"]);
    let pending = r.id_at("shop/reg.py:9");
    // The exit the final gate used to print: not_exercised on intake.workload.
    r.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    let o = r.close(&pending, "not_exercised");
    assert_eq!(o.json["error"]["code"], "target-pending", "{}", o.text);
    assert_eq!(r.missing()[0]["kind"], "queue");
    // A blocker reason on intake.workload closes the queue of the runtime.
    r.ok(&[
        "ledger",
        "resolve",
        "intake.workload",
        "--status",
        "unresolved",
        "--reason",
        "build_fails",
        "--remedy",
        "the project does not build here",
    ]);
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["open"], false, "{st}");
    assert_eq!(st["queue"]["python"]["closed"], "blocked", "{st}");
    assert_ne!(r.missing()[0]["kind"], "queue");
    let o = r.close(&pending, "not_exercised");
    assert_eq!(o.code, 0, "{}", o.text);
}

#[test]
fn with_the_queue_off_everything_is_as_before() {
    let r = Run::new(&["--budget", "3h", "--queue", "off"]);
    let state = fs::read_to_string(r.run_dir.join("state.json")).unwrap();
    assert!(state.contains("\"queue\": false"), "{state}");
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["open"], false, "{st}");
    assert_eq!(st["queue"]["python"]["closed"], "off", "{st}");
    assert_ne!(st["blocking"][0]["kind"], "queue", "{st}");
    assert_eq!(st["next_command"], "dla gate check --final", "{st}");
    let missing = r.missing();
    assert!(missing.iter().all(|m| m["kind"] != "queue"), "{missing:?}");
    let item = missing.iter().find(|m| m["kind"] == "finding").unwrap();
    // The fix is the one this item always had: the harness step, then the blocker.
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .starts_with("dla harness gen --lang python --driver, fill the DLA-HOOK bodies"),
        "{item}"
    );
    assert!(!item["fix"].as_str().unwrap().contains("--next"), "{item}");
    let bounded = r.id_at("shop/reg.py:9");
    // The refusals that were there before are the only ones: no harness was generated for
    // the entry points the discovery found.
    let o = r.close(&bounded, "not_exercised");
    assert_eq!(
        o.json["error"]["code"], "harness-not-generated",
        "{}",
        o.text
    );
    assert_eq!(r.close(&bounded, "build_fails").code, 0);
    assert!(
        r.report().contains(
            "\nQueue: off: candidates the workloads did not reach were closed without a targeted workload\n"
        ),
        "{}",
        r.report()
    );
    assert!(!r.report().contains("Targeted:") && !r.report().contains("Not targeted"));
    let lines: Vec<String> = fs::read_to_string(r.run_dir.join("ledger.jsonl"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert!(lines[0].contains("\"queue\":\"off\""), "{}", lines[0]);
}

#[test]
fn under_the_budget_floor_the_close_is_accepted() {
    let r = Run::new(&["--budget", "90s"]);
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["open"], false, "{st}");
    assert_eq!(st["queue"]["python"]["closed"], "floor", "{st}");
    assert_ne!(st["blocking"][0]["kind"], "queue");
    let bounded = r.id_at("shop/reg.py:9");
    for reason in ["not_exercised", "no_harness"] {
        let id = if reason == "not_exercised" {
            bounded.clone()
        } else {
            r.id_at("shop/reg.py:2")
        };
        let o = r.close(&id, reason);
        assert_eq!(o.code, 0, "{reason}: {}", o.text);
    }
    // close-rest says time_budget for what the floor left.
    let out = r.ok(&["ledger", "close-rest"]);
    assert_eq!(out["closed"], 1, "{out}");
    assert_eq!(out["reason"]["time_budget"], 1, "{out}");
}

#[test]
fn next_declares_the_targets_prints_them_and_writes_the_block_in_the_driver() {
    let r = Run::new(&["--budget", "3h"]);
    let gen_out = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    let s = &gen_out["summary"];
    let targets = s["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 3, "{gen_out}");
    assert_eq!(targets[0]["finding"], "F1");
    assert_eq!(targets[0]["probe"], true);
    assert!(
        s["next"]
            .as_str()
            .unwrap()
            .starts_with("fill the hooks so that each unit reaches the targets listed in the driver, then dla harness run H1 --mode both"),
        "{s}"
    );
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/harness.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["targets"], serde_json::json!(["F1", "F2", "F3"]));
    let driver = fs::read_to_string(r.driver("H1")).unwrap();
    let block = driver.find("# Targets of harness H1").unwrap();
    let hook = driver.find("# DLA-HOOK setup:").unwrap();
    assert!(block < hook, "{driver}");
    for needle in [
        "# F1 at shop/cache.py:4: holder static/global, owner shop.cache.price_for",
        "call shop.cache.price_for with arguments that differ for every unit (derive them from key)",
        "writers: shop/reg.py:5 (Registry.put)",
        "removers: shop/reg.py:14 (Bounded.put)",
        "probe reads: size of shop.reg.Registry._by_key",
        "make every unit reach a writer so that it adds an entry whose key derives from the unit's key",
        "never reuse an earlier unit's key",
    ] {
        assert!(driver.contains(needle), "{needle}\n{driver}");
    }
    // The candidates are targeted but not measured: still pending, and the next batch is
    // empty, so the status names the harness to fill and run.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["pending"], 3, "{st}");
    assert!(
        st["next_command"]
            .as_str()
            .unwrap()
            .contains("dla harness run H1 --mode both"),
        "{st}"
    );
    // --next again has nothing to declare and says so.
    let again = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    assert!(
        again["summary"]["next_note"]
            .as_str()
            .unwrap()
            .contains("none was declared"),
        "{again}"
    );
    assert!(again["summary"].get("targets").is_none());
}

#[test]
fn more_than_eight_targets_are_refused() {
    let r = Run::new(&["--budget", "3h"]);
    let o = r.run(&[
        "harness", "gen", "--lang", "python", "--driver", "--next", "9",
    ]);
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "too-many-targets", "{}", o.text);
    assert!(
        o.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("at most 8"),
        "{}",
        o.text
    );
    // Nothing was created.
    assert!(!r.run_dir.join("harness/H1").exists());
    // --probe and --next together over the limit.
    let o = r.run(&[
        "harness", "gen", "--lang", "python", "--driver", "--probe", "F1", "--next", "8",
    ]);
    assert_eq!(o.json["error"]["code"], "too-many-targets", "{}", o.text);
}

#[test]
fn verdicts_settle_candidates_and_close_rest_closes_the_rest() {
    let r = Run::new(&["--budget", "3h"]);
    let cached = r.id_at("shop/cache.py:4");
    let registry = r.id_at("shop/reg.py:2");
    let bounded = r.id_at("shop/reg.py:9");
    // A candidate in test code: not ranked, never held back.
    let tests_f = {
        let o = r.ok(&[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "tests/test_cache.py:4",
        ]);
        o["id"].as_str().unwrap().to_owned()
    };
    let (_, lines) = r.list();
    let t = lines.iter().find(|l| l["id"] == tests_f.as_str()).unwrap();
    assert_eq!(t["tier"], 6, "{t}");
    assert_eq!(t["queue"], "not-eligible");
    assert_eq!(
        lines.last().unwrap()["id"],
        tests_f.as_str(),
        "the demoted one is last"
    );

    r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    // The driver was not filled: the harness cannot run.
    r.fill("H1", "");
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let trend = r.ok(&["trend", "S1"]);
    let verdict = |f: &str| -> Value {
        trend["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["finding"] == f)
            .unwrap_or_else(|| panic!("no verdict for {f}: {trend}"))
            .clone()
    };
    // The unit calls the cached function with a new key: it grew. It touches neither dict:
    // their probes stayed at one value.
    assert_eq!(verdict(&cached)["verdict"], "grew", "{trend}");
    assert_eq!(verdict(&registry)["verdict"], "constant", "{trend}");
    assert_eq!(verdict(&bounded)["verdict"], "constant", "{trend}");
    assert!(
        verdict(&registry)["detail"]
            .as_str()
            .unwrap()
            .contains("stayed 1 over 8 points"),
        "{trend}"
    );
    // The verdicts are in the stored trend report.
    let stored: Value =
        serde_json::from_str(&fs::read_to_string(r.run_dir.join("trend/T1.json")).unwrap())
            .unwrap();
    assert_eq!(stored["targets"].as_array().unwrap().len(), 3);

    // The cached one is settled; the constant ones are pending and still cannot be closed.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["settled"], 1, "{st}");
    assert_eq!(st["queue"]["python"]["pending"], 2, "{st}");
    assert_eq!(
        r.close(&registry, "not_exercised").json["error"]["code"],
        "target-pending"
    );
    // The same driver run again settles nothing.
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    r.ok(&["trend", "S3"]);
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["pending"], 2, "{st}");
    // A changed driver, run again: two drivers targeted them, whatever the verdicts.
    r.fill_more("H1");
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let trend = r.ok(&["trend", "S5"]);
    assert!(
        trend["targets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["verdict"] == "constant"),
        "{trend}"
    );
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["pending"], 0, "{st}");
    assert_eq!(st["queue"]["python"]["open"], false, "{st}");
    assert_eq!(st["queue"]["python"]["closed"], "empty", "{st}");

    // close-rest closes what is left: static evidence gives suspected_static_only, the test
    // file's finding (a census hit with no census holder evidence it cites?) the other.
    let out = r.ok(&["ledger", "close-rest"]);
    assert!(out["closed"].as_u64().unwrap() >= 3, "{out}");
    // The cached one grew: it is left open to test, not closed.
    let left: Vec<&str> = out["left"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["id"].as_str().unwrap())
        .collect();
    assert_eq!(left, [cached.as_str()], "{out}");
    let (_, lines) = r.list();
    let state_of = |id: &str| {
        lines.iter().find(|l| l["id"] == id).unwrap()["state"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    assert_eq!(state_of(&registry), "suspected_static_only(not_exercised)");
    assert_eq!(state_of(&bounded), "suspected_static_only(not_exercised)");
    // The remedy names what was tried and what to do.
    let ledger = fs::read_to_string(r.run_dir.join("ledger.jsonl")).unwrap();
    assert!(
        ledger.contains("targeted in H1, its holder did not change: write a unit that adds a new key to shop.reg.Bounded._recent"),
        "{ledger}"
    );
    assert!(
        ledger.contains("tier 6 (not ranked for measurement), not targeted: dla harness gen --lang python --driver --probe"),
        "{ledger}"
    );
    assert!(
        ledger.contains("\"close_rest\""),
        "one run-config record lists what was closed"
    );

    // The one that grew is tested or closed by the caller; then nothing of the queue is left
    // and the final gate asks for no finding.
    let o = r.run(&[
        "ledger",
        "resolve",
        &cached,
        "--as",
        "suspected_static_only",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    assert_eq!(o.code, 0, "{}", o.text);
    let left = r.missing();
    assert!(
        left.iter()
            .all(|m| m["kind"] != "queue" && m["kind"] != "finding"),
        "{left:?}"
    );

    // The report says how the candidates were targeted, in the header and per finding.
    let md = r.report();
    assert!(
        md.contains(
            "\nQueue: 3 of 3 ranked candidates targeted in 1 harness: grew 1, not reached 2; 0 not targeted\n"
        ),
        "{md}"
    );
    assert!(md.contains("- Targeted: H1 constant\n"), "{md}");
    assert!(
        md.contains("- Not targeted (tier 6, not ranked for measurement)\n"),
        "{md}"
    );
}

#[test]
fn the_stop_hook_leads_with_the_queue_item() {
    let r = Run::new(&["--budget", "3h"]);
    let away = tempfile::tempdir().unwrap();
    let stdin = serde_json::json!({
        "session_id": "0f7c2c1e-0000-4000-8000-000000000001",
        "transcript_path": "/tmp/t.jsonl",
        "cwd": r.dir.path(),
        "permission_mode": "default",
        "hook_event_name": "Stop",
        "stop_hook_active": false,
        "last_assistant_message": "done",
        "background_tasks": [],
        "session_crons": [],
    })
    .to_string();
    let mut c = command(r.dir.path());
    let o = c
        .current_dir(away.path())
        .args(["gate", "stop-hook"])
        .write_stdin(stdin)
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["decision"], "block", "{v}");
    let reason = v["reason"].as_str().unwrap();
    assert!(reason.len() <= 1000, "{} bytes", reason.len());
    let queue = reason
        .find("candidate(s) of python are ranked for measurement")
        .unwrap();
    let finding = reason.find("finding F1").unwrap_or(usize::MAX);
    assert!(queue < finding, "{reason}");
    assert!(reason.contains(BATCH), "{reason}");
}
