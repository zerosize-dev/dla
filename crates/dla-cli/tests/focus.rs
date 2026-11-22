//! A target that a batch missed gets a harness of its own.
//!
//! A hook-driver run MISSES a target when its verdict is constant, not_loaded, not_executed
//! or unknown. A missed candidate stays pending; when the best-ranked pending candidate is a
//! missed one `dla harness gen --next` gives a FOCUS harness (at most three targets, no
//! siblings, no fresh candidates); a candidate is settled as not reached only when two
//! different drivers missed it and one of the harnesses was focused.

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

    fn id_at(&self, site: &str) -> String {
        self.list()
            .iter()
            .find(|l| l["site"].as_str() == Some(site))
            .unwrap_or_else(|| panic!("no finding at {site}"))["id"]
            .as_str()
            .unwrap()
            .to_owned()
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

/// The verdicts of the stored trend report (the command's output is bounded).
fn verdicts(r: &Run, t: &Value) -> Vec<(String, String)> {
    let stored: Value = serde_json::from_str(
        &fs::read_to_string(
            r.run_dir
                .join(format!("trend/{}.json", t["trend"].as_str().unwrap())),
        )
        .unwrap(),
    )
    .unwrap();
    stored["targets"]
        .as_array()
        .unwrap_or_else(|| panic!("no targets: {t}"))
        .iter()
        .map(|v| {
            (
                v["finding"].as_str().unwrap().to_owned(),
                v["verdict"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

#[test]
fn a_missed_target_gets_a_focus_harness_and_is_given_up_only_by_the_rule() {
    let r = Run::new();
    let ids: Vec<String> = (1..=6)
        .map(|i| r.id_at(&format!("shop/m{i}.py:1")))
        .collect();

    // Batch 1 is broad: six targets (more than three), the unit reaches two of them.
    let gen1 = r.ok(&["harness", "gen", "--lang", "python", "--driver", "--next"]);
    let m1: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/harness.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(m1["targets"].as_array().unwrap().len(), 6, "{m1}");
    assert!(m1.get("purpose").is_none(), "{m1}");
    assert!(gen1["summary"].get("focus").is_none(), "{gen1}");
    let block = fs::read_to_string(r.driver("H1")).unwrap();
    assert!(
        block.contains("# A target whose code does not run in this harness comes back in a small harness of its own"),
        "{block}"
    );
    r.fill("H1", 2, "first");
    let ran = r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let s1 = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = r.ok(&["trend", &s1]);
    let v = verdicts(&r, &trend);
    let grew: Vec<&String> = v
        .iter()
        .filter(|(_, x)| x == "grew")
        .map(|(f, _)| f)
        .collect();
    let missed: Vec<String> = v
        .iter()
        .filter(|(_, x)| x == "constant" || x == "not_loaded")
        .map(|(f, _)| f.clone())
        .collect();
    assert_eq!((grew.len(), missed.len()), (2, 4), "{trend}");
    // `trend` names the missed targets and the batch command first.
    let first = trend["next"][0].as_str().unwrap();
    assert!(
        first.starts_with("H1 missed 4 target(s): ")
            && first.contains("dla harness gen --lang python --driver --next"),
        "{first}"
    );
    for f in &missed {
        assert!(first.contains(f.as_str()), "{first}");
    }

    // The missed ones are pending and missed; the queue's next batch is a focus batch.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["focus"], true, "{st}");
    // (the status output is bounded: the full batch is in the unit tests of the queue)
    assert!(
        missed.contains(
            &st["queue"]["python"]["next"][0]
                .as_str()
                .unwrap()
                .to_owned()
        ),
        "{st}"
    );
    let row = r
        .list()
        .into_iter()
        .find(|l| l["id"] == missed[0].as_str())
        .unwrap();
    assert_eq!(row["queue"], "pending", "{row}");
    assert!(
        row["missed_in"].as_str().unwrap().starts_with("H1 "),
        "{row}"
    );
    assert!(
        row["focus"].as_str().unwrap().starts_with("pending"),
        "{row}"
    );

    // A missed candidate cannot be closed as not exercised: the fix names the focus batch.
    let o = r.run(&[
        "ledger",
        "resolve",
        &missed[0],
        "--as",
        "suspected_static_only",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "target-pending", "{}", o.text);
    let fix = o.json["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains("dla harness gen --lang python --driver --next")
            && fix.contains("focus batch")
            && fix.contains("--runtime python (that closes the queue of python only)"),
        "{fix}"
    );
    // A reason that says what it needs is still accepted (the refusal does not change).
    let o = r.run(&[
        "ledger",
        "resolve",
        &missed[3],
        "--as",
        "unresolved",
        "--reason",
        "env_unavailable",
        "--remedy",
        "needs a container",
    ]);
    assert_eq!(o.code, 0, "{}", o.text);

    // --next gives a focus harness: three targets of the three missed ones left, no fresh.
    let gen2 = r.ok(&[
        "harness", "gen", "--lang", "python", "--driver", "--next", "8",
    ]);
    let s = &gen2["summary"];
    let t2: Vec<String> = s["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["finding"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(t2.len(), 3, "{gen2}");
    assert!(t2.iter().all(|f| missed.contains(f)), "{t2:?} {missed:?}");
    assert!(s["focus"].as_str().unwrap().contains("focus batch"), "{s}");
    assert!(s["focus"].as_str().unwrap().contains("capped at 3"), "{s}");
    let m: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H2/harness.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(m["purpose"], "focus", "{m}");
    assert!(r.run_dir.join("harness/H2/focus.json").is_file());
    let block = fs::read_to_string(r.driver("H2")).unwrap();
    for needle in [
        "# Focus harness: these targets were declared in an earlier harness and their code did not run there. This harness exists for them alone.",
        "Call the writer function of each (named below) with a new key per unit.",
        "dla ledger resolve <F> --as unresolved --reason env_unavailable",
        &format!("#   {} was declared in H1 and read", t2[0]),
    ] {
        assert!(block.contains(needle), "{needle}\n{block}");
    }
    assert!(!block.contains("A target whose code does not run in this harness comes back"));
    let _ = &ids;

    // The focus harness misses again with another driver: one focused harness and two
    // different drivers settle the three as not reached.
    r.fill("H2", 0, "second");
    let ran = r.ok(&["harness", "run", "H2", "--mode", "both"]);
    let s2 = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = r.ok(&["trend", &s2]);
    assert!(
        verdicts(&r, &trend).iter().all(|(_, x)| x != "grew"),
        "{trend}"
    );
    let rows = r.list();
    for f in &t2 {
        let row = rows.iter().find(|l| l["id"] == f.as_str()).unwrap();
        assert_eq!(row["queue"], "settled", "{row}");
        assert_eq!(row["settled_by"], "not_reached", "{row}");
    }
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["python"]["pending"], 0, "{st}");
    let out = r.ok(&["ledger", "close-rest"]);
    assert!(out["closed"].as_u64().unwrap() >= 3, "{out}");
    let md =
        String::from_utf8(command(r.dir.path()).arg("report").output().unwrap().stdout).unwrap();
    assert!(
        md.contains("grew 2") && md.contains("not reached 3"),
        "{md}"
    );
}
