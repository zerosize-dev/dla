//! A second look: one flat run of one driver does not end a run that has budget left. When
//! no first-pass candidate is pending and budget is left, the candidates a flat reading (verdict
//! `executed`) of ONE driver settled get a focus harness with a different driver; they are settled
//! for good by a second, different driver.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// Six modules, each a module-level dict with a writer function.
fn project(dir: &Path) {
    write(dir, "shop/__init__.py", "");
    for i in 1..=6 {
        write(
            dir,
            &format!("shop/m{i}.py"),
            &format!("_d{i} = {{}}\n\n\ndef put_{i}(key):\n    _d{i}[key] = 1\n"),
        );
    }
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
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path());
        let o = command(dir.path())
            .env("DLA_CONSENT_TIERS", "1,2,3")
            .args(["run", "init", "--session-id", "t", "--budget", "3h"])
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

    fn list(&self) -> Vec<Value> {
        let o = command(self.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        String::from_utf8(o.stdout)
            .unwrap()
            .lines()
            .skip(1)
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn driver(&self, h: &str) -> std::path::PathBuf {
        self.run_dir
            .join("harness")
            .join(h)
            .join("driver/dla_harness.py")
    }

    /// Fills the hooks: setup imports every module; the unit calls the writers of the first
    /// `reach` modules with a new key (none for 0); `tag` makes the driver text differ.
    fn fill(&self, h: &str, reach: u32, tag: &str) {
        let p = self.driver(h);
        let text = fs::read_to_string(&p).unwrap();
        let imports: String = (1..=6)
            .map(|i| format!("    from shop import m{i}\n    _state['m{i}'] = m{i}\n"))
            .collect();
        let calls: String = (1..=reach)
            .map(|i| format!("    _state['m{i}'].put_{i}(key)\n"))
            .collect();
        let unit = format!("    # {tag}\n{calls}    pass");
        let text = text
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
                imports.trim_end_matches('\n'),
            )
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
                &unit,
            );
        fs::write(p, text).unwrap();
    }
}

/// Sets every target verdict of the stored trend report `trend` to `verdict` (the Python
/// probes read `constant` for a unit that touches nothing; a flat reading that ran is
/// `executed`).
fn set_verdicts(r: &Run, trend: &str, verdict: &str) {
    let path = r.run_dir.join(format!("trend/{trend}.json"));
    let mut stored: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    for t in stored["targets"].as_array_mut().unwrap() {
        t["verdict"] = Value::from(verdict);
    }
    fs::write(&path, stored.to_string()).unwrap();
}

fn varies(r: &Run, h: &str, text: &str) {
    fs::write(r.driver(h).with_file_name("VARIES.txt"), text).unwrap();
}

#[test]
fn one_flat_run_gets_a_second_look_with_a_different_driver_while_budget_lasts() {
    let r = Run::new();
    // First pass: one broad harness, one arbitrary unit, every target read flat.
    r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    r.fill("H1", 0, "first");
    let ran = r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let s1 = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = r.ok(&["trend", &s1]);
    set_verdicts(&r, trend["trend"].as_str().unwrap(), "executed");

    // Status, list, close-rest and the final gate ask for the second look.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["second_look"], true, "{st}");
    assert_eq!(st["queue"]["python"]["focus"], true, "{st}");
    assert_eq!(st["queue"]["python"]["pending"], 6, "{st}");
    let row = r
        .list()
        .into_iter()
        .find(|l| l["site"] == "shop/m1.py:1")
        .unwrap();
    assert_eq!(row["queue"], "pending", "{row}");
    assert!(
        row["second_look"]
            .as_str()
            .unwrap()
            .starts_with("pending: one flat run of one driver (H1)"),
        "{row}"
    );
    let out = r.ok(&["ledger", "close-rest"]);
    assert_eq!(out["closed"], 0, "{out}");
    assert_eq!(
        out["batch_command"], "dla harness gen --lang python --driver --next",
        "{out}"
    );
    let _ = r.run(&["gate", "check", "--final"]);
    let full: Value =
        serde_json::from_str(&fs::read_to_string(r.run_dir.join("gate/final.json")).unwrap())
            .unwrap();
    assert!(
        full["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["kind"] == "queue"
                && m["description"].as_str().unwrap().contains("Second look")),
        "{full}"
    );

    // --next writes a second-look harness: 3 targets, the to-dos, the earlier harness named.
    let gen2 = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    assert!(
        gen2["summary"]["focus"]
            .as_str()
            .unwrap()
            .starts_with("second look:"),
        "{gen2}"
    );
    let m: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H2/harness.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(m["purpose"], "second-look", "{m}");
    assert_eq!(m["targets"].as_array().unwrap().len(), 3, "{m}");
    let block = fs::read_to_string(r.driver("H2")).unwrap();
    for needle in [
        "# Second look: one flat run of one driver settled these targets",
        "other call orders than the first driver's",
        "a child of a child",
        "cancel or close the parent before the child",
        "overlapping lifetimes",
        "Write the unit as CHOICES, not as one fixed sequence: a fixed sequence runs one case for every unit",
        "Every slot variable must be used by the unit",
        "was read flat in H1: read the unit of that harness",
    ] {
        assert!(block.contains(needle), "{needle}\n{block}");
    }
    // `harness run` refuses a missing VARIES.txt, then hooks equal to the first driver's.
    r.fill("H2", 0, "first");
    let o = r.run(&["harness", "run", "H2", "--mode", "both"]);
    assert_eq!(o.json["error"]["code"], "varies-missing", "{}", o.text);
    varies(&r, "H2", "a: cancels first\nb: derives deep\nc: overlaps\n");
    let o = r.run(&["harness", "run", "H2", "--mode", "both"]);
    assert_eq!(o.json["error"]["code"], "invalid-value", "{}", o.text);
    assert!(
        o.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("same code as those of H1"),
        "{}",
        o.text
    );
    // A different unit that uses no slot variable of the skeleton is refused, naming the slots.
    let p = r.driver("H2");
    let skeleton = fs::read_to_string(&p).unwrap();
    assert!(
        skeleton.contains("    depth = dla_choice(key, 0, 3) # slot depth: how deep to derive"),
        "{skeleton}"
    );
    let unit = skeleton.replace(
        "    # first\n    pass",
        // One unit in sixteen reaches the writer: the heap it leaves stays under the floor below
        // which a heap residual demands a capture (64 KiB over the series).
        "    # second look\n    if i % 16 == 0:\n        _state['m1'].put_1(key)\n    pass",
    );
    fs::write(&p, &unit).unwrap();
    let o = r.run(&["harness", "run", "H2", "--mode", "both"]);
    assert_eq!(o.json["error"]["code"], "invalid-value", "{}", o.text);
    let msg = o.json["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("does not use slot variable(s): depth (how deep to derive")
            && msg.contains("outcome (the success or the failure path"),
        "{msg}"
    );
    assert!(
        o.json["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("# slot <slot>: not applicable: <reason>"),
        "{}",
        o.text
    );
    // Without the skeleton lines the unit takes no choice from the key at all: refused first.
    let bare: String = unit
        .lines()
        .filter(|l| !l.contains("(every slot variable must be used by the unit)"))
        .map(|l| format!("{l}\n"))
        .collect();
    fs::write(&p, &bare).unwrap();
    let o = r.run(&["harness", "run", "H2", "--mode", "both"]);
    assert!(
        o.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("calls dla_choice(key, slot, n) 0 time(s)"),
        "{}",
        o.text
    );
    // Giving up three slots is refused; two are accepted and listed.
    let give_up = |text: &str, name: &str, why: &str| -> String {
        text.lines()
            .map(|l| {
                if l.contains(&format!("# slot {name}: ")) && l.contains("(every slot variable") {
                    format!("    # slot {name}: not applicable: {why}\n")
                } else {
                    format!("{l}\n")
                }
            })
            .collect()
    };
    let uses = |text: &str, vars: &str| -> String {
        text.replace(
            "    # second look\n",
            &format!("    # second look\n    _ = ({vars})\n"),
        )
    };
    let three = uses(
        &give_up(
            &give_up(
                &give_up(&unit, "order", "no release order in this API"),
                "outcome",
                "no failure path",
            ),
            "when",
            "nothing is released early",
        ),
        "depth, ending",
    );
    fs::write(&p, &three).unwrap();
    let o = r.run(&["harness", "run", "H2", "--mode", "both"]);
    assert!(
        o.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("gives up 3 slots"),
        "{}",
        o.text
    );
    let two = uses(
        &give_up(
            &give_up(&unit, "order", "no release order in this API"),
            "outcome",
            "no failure path",
        ),
        "depth, ending, when",
    );
    fs::write(&p, &two).unwrap();
    let ran = r.ok(&["harness", "run", "H2", "--mode", "both"]);
    assert_eq!(
        ran["summary"]["choices"], "the units took 36 combinations of 3 choices",
        "{ran}"
    );
    assert_eq!(
        ran["summary"]["slots_given_up"],
        serde_json::json!([
            "slot order: not applicable: no release order in this API",
            "slot outcome: not applicable: no failure path"
        ]),
        "{ran}"
    );
    let s2 = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = r.ok(&["trend", &s2]);
    assert_eq!(
        trend["choices"], "the units took 36 combinations of 3 choices",
        "{trend}"
    );
    assert_eq!(
        trend["slots_given_up"].as_array().unwrap().len(),
        2,
        "{trend}"
    );
    set_verdicts(&r, trend["trend"].as_str().unwrap(), "executed");
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["pending"], 3, "{st}");
    // At the floor nothing is asked: the deadline moves to a minute from now.
    let path = r.run_dir.join("state.json");
    let original = fs::read_to_string(&path).unwrap();
    let mut state: Value = serde_json::from_str(&original).unwrap();
    state["budget"]["deadline"] = Value::from(
        (chrono::Utc::now() + chrono::Duration::seconds(60))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string(),
    );
    fs::write(&path, state.to_string()).unwrap();
    let st = r.ok(&["run", "status"]);
    assert!(st["queue"]["python"].get("second_look").is_none(), "{st}");
    assert_eq!(st["queue"]["python"]["closed"], "floor", "{st}");
    fs::write(&path, original).unwrap();
    // The other three get their own look; then nothing is left and the gate item is gone.
    r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    r.fill("H3", 2, "third");
    let p3 = r.driver("H3");
    let text = fs::read_to_string(&p3).unwrap().replace(
        "    # third\n",
        "    # third\n    _ = (depth, ending, when, order, outcome)\n",
    );
    fs::write(&p3, text).unwrap();
    varies(&r, "H3", "a: x\nb: y\nc: z\n");
    let ran = r.ok(&["harness", "run", "H3", "--mode", "both"]);
    let s3 = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = r.ok(&["trend", &s3]);
    set_verdicts(&r, trend["trend"].as_str().unwrap(), "executed");
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["pending"], 0, "{st}");
    assert_eq!(st["queue"]["python"]["closed"], "empty", "{st}");
    let row = r
        .list()
        .into_iter()
        .find(|l| l["site"] == "shop/m1.py:1")
        .unwrap();
    assert_eq!(row["settled_by"], "executed", "{row}");
    assert_eq!(row["second_look"], "done in H2", "{row}");
    let md =
        String::from_utf8(command(r.dir.path()).arg("report").output().unwrap().stdout).unwrap();
    assert!(md.contains("second look 6 (grew 0)"), "{md}");
}

/// Sets the verdict of one finding in the stored trend report `trend`, and of every other target
/// to `rest`.
fn set_verdict_of(r: &Run, trend: &str, id: &str, verdict: &str, rest: &str) {
    let path = r.run_dir.join(format!("trend/{trend}.json"));
    let mut stored: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    for t in stored["targets"].as_array_mut().unwrap() {
        t["verdict"] = Value::from(if t["finding"] == id { verdict } else { rest });
    }
    fs::write(&path, stored.to_string()).unwrap();
}

/// One harness read five targets flat (`executed`) and missed one. `close-rest` while the missed
/// one is pending must not close the five: the second look comes only when the first pass is
/// over, and by then they would be no candidates any more.
#[test]
fn close_rest_leaves_the_flat_candidates_for_their_second_look() {
    let r = Run::new();
    r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    r.fill("H1", 0, "first");
    let ran = r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let s1 = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = r.ok(&["trend", &s1]);
    let missed = r
        .list()
        .into_iter()
        .find(|l| l["site"] == "shop/m6.py:1")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    set_verdict_of(
        &r,
        trend["trend"].as_str().unwrap(),
        &missed,
        "not_executed",
        "executed",
    );
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["pending"], 1, "{st}");

    let out = r.ok(&["ledger", "close-rest"]);
    assert_eq!(out["closed"], 0, "{out}");
    let left = out["left"].as_array().unwrap();
    let flat: Vec<&Value> = left.iter().filter(|l| l["id"] != missed.as_str()).collect();
    assert_eq!(flat.len(), 5, "{out}");
    assert!(
        flat.iter().all(|l| l["why"].as_str().unwrap().contains(
            "waits for its second look with a different driver once the first pass is over"
        )),
        "{out}"
    );
    for l in r.list() {
        assert_eq!(l["state"], "candidate", "{l}");
    }

    // The missed one gets its focus harness and reads flat too: the first pass is over and the
    // queue offers the second look for all six.
    let made = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    let h = made["summary"]["harness"].as_str().unwrap().to_owned();
    r.fill(&h, 0, "focus");
    let ran = r.ok(&["harness", "run", &h, "--mode", "both"]);
    let s2 = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = r.ok(&["trend", &s2]);
    set_verdicts(&r, trend["trend"].as_str().unwrap(), "executed");
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["second_look"], true, "{st}");
    assert_eq!(st["queue"]["python"]["pending"], 5, "{st}");
    // Only the one the focus harness settled is closed; the five now wait in the queue.
    let out = r.ok(&["ledger", "close-rest"]);
    assert_eq!(out["closed"], 1, "{out}");
    assert_eq!(out["left"].as_array().unwrap().len(), 5, "{out}");
}
