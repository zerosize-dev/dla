//! A JVM probe measures an instance field of the objects that are alive. Three classes keep a
//! list in an instance field: `Registry` is filled by every unit and never emptied, `Drainer`
//! is filled and emptied by every unit, and `Parked` is held only through a thread-local of the
//! main thread (the driver keeps it in a `ThreadLocal`, which the walk does not descend into).
//! The probe of each is a `jvm-instance` probe: its column grows for `Registry` and `Parked`,
//! stays flat for `Drainer`, and the record counts the instances it found.
//!
//! Skips, printing the reason, when `java`, `javac` or `sh` is missing.

#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn has(tool: &str) -> bool {
    Std::new(tool)
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn dla(project: &Path, args: &[&str]) -> (i32, Value, String) {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    (
        o.status.code().unwrap(),
        serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    )
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, json, text) = dla(project, args);
    assert_eq!(code, 0, "{args:?}: {text}");
    assert_eq!(json["ok"], true, "{text}");
    json
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

const BUILD: &str = "#!/bin/sh\nset -e\ncd \"$(dirname \"$0\")\"\nrm -rf out\nmkdir -p out\njavac -d out src/inst/*.java\nif [ \"$1\" = \"run\" ]; then\n    java -cp out inst.Main \"${2:-100}\"\nfi\n";

/// A class with one list in an instance field and a method that adds to it.
fn class(name: &str) -> String {
    format!(
        r#"package inst;

import java.util.ArrayList;
import java.util.List;

public class {name} {{
    final List<String> items = new ArrayList<>();

    public void add(String s) {{
        items.add(s);
    }}

    public void clear() {{
        items.clear();
    }}

    public int size() {{
        return items.size();
    }}
}}
"#
    )
}

const MAIN: &str = "package inst;\n\npublic final class Main {\n    public static void main(String[] args) {\n        Registry r = new Registry();\n        Drainer d = new Drainer();\n        Parked p = new Parked();\n        for (int i = 0; i < 100; i++) {\n            r.add(\"a\" + i);\n            d.add(\"a\" + i);\n            d.clear();\n            p.add(\"a\" + i);\n        }\n        System.out.println(r.size() + \" \" + d.size() + \" \" + p.size());\n    }\n}\n";

const STATICS: &str = "    static Registry registry;\n    static Drainer drainer;\n    static final ThreadLocal<Parked> parked = new ThreadLocal<>();";
const SETUP: &str = "        registry = new Registry();\n        drainer = new Drainer();\n        parked.set(new Parked());";
const UNIT: &str = "        registry.add(\"u\" + i);\n        drainer.add(\"u\" + i);\n        drainer.clear();\n        parked.get().add(\"u\" + i);";

struct Scene {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
}

fn scene() -> Scene {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write(&project, "build.sh", BUILD);
    for name in ["Registry", "Drainer", "Parked"] {
        write(&project, &format!("src/inst/{name}.java"), &class(name));
    }
    write(&project, "src/inst/Main.java", MAIN);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "jvm"]);
    ok(&project, &["workload", "discover", "--lang", "jvm"]);
    Scene {
        _dir: dir,
        project,
        run_dir,
    }
}

impl Scene {
    fn driver(&self, h: &str) -> PathBuf {
        self.run_dir
            .join("harness")
            .join(h)
            .join("driver/DlaLoop.java")
    }

    fn fill(&self, h: &str, statics: &str, setup: &str, unit: &str) {
        let p = self.driver(h);
        let text = fs::read_to_string(&p)
            .unwrap()
            .replace(
                "        throw new UnsupportedOperationException(\"DLA-HOOK setup has not been filled in\");",
                setup,
            )
            .replace(
                "        throw new UnsupportedOperationException(\"DLA-HOOK unit has not been filled in\");",
                unit,
            )
            .replace(
                "    private DlaLoop() {}",
                &format!("    private DlaLoop() {{}}\n\n{statics}"),
            );
        assert!(!text.contains("has not been filled in"));
        fs::write(p, text).unwrap();
    }

    fn finding(&self, owner: &str) -> String {
        let (code, _, text) = dla(&self.project, &["ledger", "list", "--all"]);
        assert_eq!(code, 0, "{text}");
        text.lines()
            .skip(1)
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .find(|r| r["owner"] == owner)
            .unwrap_or_else(|| panic!("no finding of {owner}"))["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    /// The last checkpoint record of the run's dump file, and its probe of `path`.
    fn probe(&self, path: &str) -> Value {
        let dump =
            fs::read_to_string(self.run_dir.join("harness/H1/out/p1-fresh/dump.jsonl")).unwrap();
        let last: Value = dump
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .rfind(|r| r["phase"] == "checkpoint")
            .expect("a checkpoint record");
        last["probes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"] == path)
            .unwrap_or_else(|| panic!("no probe of {path}: {last}"))
            .clone()
    }
}

#[test]
fn an_instance_field_is_measured_on_the_objects_that_are_alive() {
    if !(has("java") && has("javac")) {
        eprintln!("skipping: java or javac is not available");
        return;
    }
    let s = scene();
    let ids: Vec<(&str, String)> = ["Registry", "Drainer", "Parked"]
        .iter()
        .map(|c| (*c, s.finding(&format!("inst.{c}.items"))))
        .collect();
    ok(
        &s.project,
        &[
            "harness",
            "gen",
            "--lang",
            "jvm",
            "--driver",
            "--package",
            "inst",
            "--next",
            "3",
        ],
    );
    // The probe of an instance field is a `jvm-instance` probe, ahead of the class probe.
    let probes: Value = serde_json::from_str(
        &fs::read_to_string(s.run_dir.join("harness/H1/probes.json")).unwrap(),
    )
    .unwrap();
    let probes = probes["probes"].as_array().unwrap();
    for (class, _) in &ids {
        let at = |kind: &str| {
            probes
                .iter()
                .position(|p| {
                    p["target"]["kind"] == kind && p["target"]["class"] == format!("inst.{class}")
                })
                .unwrap_or_else(|| panic!("no {kind} probe of {class}: {probes:?}"))
        };
        assert!(at("jvm-instance") < at("jvm-class"), "{class}");
    }
    let block = fs::read_to_string(s.driver("H1")).unwrap();
    assert!(
        block.contains("probe reads: instance field inst.Registry.items"),
        "{block}"
    );
    s.fill("H1", STATICS, SETUP, UNIT);
    let run = ok(&s.project, &["harness", "run", "H1", "--mode", "fresh"]);
    let series = run["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = ok(&s.project, &["trend", &series]);

    // The holders that every unit fills grow: not `executed` with a flat column.
    let verdict = |finding: &str| -> String {
        trend["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["finding"] == finding)
            .unwrap_or_else(|| panic!("no target of {finding}: {trend}"))["verdict"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let (registry, drainer, parked) = (&ids[0].1, &ids[1].1, &ids[2].1);
    assert_eq!(verdict(registry), "grew", "{trend}");
    assert_eq!(verdict(parked), "grew", "{trend}");
    // The one that the unit empties stays flat, and says so by its column.
    assert_eq!(verdict(drainer), "constant", "{trend}");
    for t in trend["targets"].as_array().unwrap() {
        let column = t["column"].as_str().unwrap_or_default();
        assert!(
            column.starts_with("probe:inst."),
            "the target reads its own column: {t}"
        );
    }

    // The record keeps the shape of a static probe's and counts the instances it found.
    let units = {
        let dump =
            fs::read_to_string(s.run_dir.join("harness/H1/out/p1-fresh/dump.jsonl")).unwrap();
        dump.lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .rfind(|r| r["phase"] == "checkpoint")
            .unwrap()["x"]
            .as_u64()
            .unwrap()
    };
    for (class, expect_size) in [("Registry", units), ("Parked", units), ("Drainer", 0)] {
        let p = s.probe(&format!("inst.{class}.items"));
        assert_eq!(p["instances"], 1, "{class}: {p}");
        assert_eq!(p["size"].as_u64().unwrap(), expect_size, "{class}: {p}");
        assert!(p["reachable"].as_u64().unwrap() >= 1, "{class}: {p}");
        assert!(p["error"].is_null(), "{class}: {p}");
    }
}
