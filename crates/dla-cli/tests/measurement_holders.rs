//! A holder that belongs to the measurement and not to the project makes no finding, no dump lead
//! and no residual-capture item: the driver's own statics, the DLA agent and the flight recorder
//! the capture starts. A holder of a JDK cache or of the project in the same dumps still does,
//! and `dla summarize heap-graph` names what it left out on one line.

#[path = "../../dla-parsers/tests/common/hprof_builder.rs"]
mod hprof_builder;

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use hprof_builder::{B, F, T_INT, T_OBJ, align8};
use serde_json::Value;

const T0: u64 = 1_800_000_000_000;

/// The statics the dumps grow: (class, static field, items at each of the three dumps).
type Holders<'a> = &'a [(&'a str, &'a str, [usize; 3])];

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1,2,3");
    c
}

struct P {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl P {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let o = dla(dir.path())
            .args(["run", "init", "--session-id", "t", "--budget", "3h"])
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
        Self { dir, run_dir }
    }

    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = dla(self.dir.path()).args(args).output().unwrap();
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

    /// The driver source of harness H1, as `dla harness gen` writes it: class `app.DlaLoop`.
    fn driver(&self) {
        let d = self.run_dir.join("harness/H1/driver");
        fs::create_dir_all(&d).unwrap();
        fs::write(
            d.join("DlaLoop.java"),
            "package app;\n\npublic class DlaLoop {\n  static Object TCP;\n}\n",
        )
        .unwrap();
    }

    /// Three dumps of one process, each under `harness/H1/out/`, where each of `holders` keeps a
    /// list of the given number of items in a static field of its class.
    fn dumps(&self, holders: Holders<'_>) -> Vec<String> {
        (0..3)
            .map(|i| {
                let rel = format!("harness/H1/out/p{}/capture/7/heap-{}.hprof", i + 1, i + 1);
                let abs = self.dir.path().join(&rel);
                fs::create_dir_all(abs.parent().unwrap()).unwrap();
                let sizes: Vec<(&str, &str, usize)> =
                    holders.iter().map(|(c, f, n)| (*c, *f, n[i])).collect();
                fs::write(&abs, dump(T0 + 30_000 * i as u64, &sizes)).unwrap();
                rel
            })
            .collect()
    }

    fn summarize(&self, files: &[String]) -> Value {
        let mut args = vec!["summarize", "heap-graph"];
        args.extend(files.iter().map(String::as_str));
        self.ok(&args)
    }

    fn findings(&self) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .filter(|v| v["type"] == "finding_added")
            .map(|v| v["finding"].clone())
            .collect()
    }

    fn final_gate(&self) -> Vec<Value> {
        let _ = self.run(&["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }
}

/// One dump: for each `(class, static field, n)` the class has a static list of `n` items.
fn dump(ts: u64, holders: &[(&str, &str, usize)]) -> Vec<u8> {
    let mut b = B::new(8, ts);
    let (hdr, arr_hdr, refsz) = (12u64, 16u64, 4u64);
    let _ = (hdr, arr_hdr, refsz, align8(1), T_INT);
    let obj = b.declare("java.lang.Object");
    let list = b.declare("java.util.ArrayList");
    let obj_arr = b.declare("[Ljava/lang/Object;");
    let byte_arr = b.declare("[B");
    let item = b.declare("app.Item");
    let classes: Vec<u64> = holders.iter().map(|(c, ..)| b.declare(c)).collect();
    let mut statics: Vec<u64> = Vec::new();
    for (k, (_, _, n)) in holders.iter().enumerate() {
        let mut ids = Vec::new();
        for i in 0..*n {
            let payload = b.byte_array(&vec![k as u8; 40 + (i % 3)]);
            ids.push(b.instance(item, &[F::Id(payload), F::I4(i as i32)]));
        }
        let arr = b.obj_array(obj_arr, &ids);
        statics.push(b.instance(list, &[F::Id(arr), F::I4(*n as i32)]));
    }
    b.class_dump(obj, 0, 0, 999, &[], &[]);
    b.class_dump(
        list,
        obj,
        0,
        999,
        &[("elementData", T_OBJ), ("size", T_INT)],
        &[],
    );
    b.class_dump(obj_arr, obj, 0, 999, &[], &[]);
    b.class_dump(byte_arr, obj, 0, 999, &[], &[]);
    b.class_dump(item, obj, 0, 999, &[("payload", T_OBJ), ("n", T_INT)], &[]);
    for (k, (_, field, _)) in holders.iter().enumerate() {
        b.class_dump(classes[k], obj, 0, 999, &[], &[(field, statics[k])]);
        // A class a loader keeps: its statics are roots.
        b.root(0x05, classes[k], 0, 0);
    }
    b.finish(3)
}

const OWN: Holders<'static> = &[
    ("app.Holder", "ITEMS", [200, 400, 600]),
    ("app.DlaLoop", "TCP", [200, 400, 600]),
    ("dla.harness.DlaDump", "ROOT_CLASS", [200, 400, 600]),
    ("java.net.InetAddress", "cache", [200, 400, 600]),
];

#[test]
fn the_driver_and_the_agent_make_no_lead_and_a_project_holder_and_a_jdk_cache_still_do() {
    let p = P::new();
    p.driver();
    let files = p.dumps(OWN);
    let v = p.summarize(&files);
    assert_eq!(v["gate"], "pass", "{v}");
    // The measurement's own holders are named on one line, and nowhere as a passed column.
    let line = v["left_out"].as_str().unwrap_or_else(|| panic!("{v}"));
    assert!(
        line.starts_with("the measurement's own holders, left out: "),
        "{line}"
    );
    assert!(line.contains("app.DlaLoop.TCP"), "{line}");
    assert!(line.contains("dla.harness.DlaDump.ROOT_CLASS"), "{line}");
    assert!(
        !line.contains("InetAddress") && !line.contains("app.Holder"),
        "{line}"
    );
    let passed: Vec<&str> = v["passed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(passed.contains(&"static:app.Holder.ITEMS"), "{passed:?}");
    assert!(
        passed.contains(&"static:java.net.InetAddress.cache"),
        "{passed:?}"
    );
    assert!(
        passed
            .iter()
            .all(|k| !k.contains("DlaLoop") && !k.contains("dla.harness")),
        "{passed:?}"
    );
    // Findings: the project's holder and the JDK cache only.
    let owners: Vec<String> = p
        .findings()
        .iter()
        .filter_map(|f| f["owner"].as_str().map(str::to_owned))
        .collect();
    assert_eq!(owners.len(), 2, "{owners:?}");
    assert!(
        owners.contains(&"app.Holder.ITEMS".to_owned()),
        "{owners:?}"
    );
    assert!(
        owners.contains(&"java.net.InetAddress.cache".to_owned()),
        "{owners:?}"
    );
    // The series has no column for them, so `dla trend` and the gate have nothing to turn into an
    // item: no gate item names them, and the leads are those two findings.
    let series: Value =
        serde_json::from_str(&fs::read_to_string(p.run_dir.join("series/S1.json")).unwrap())
            .unwrap();
    let columns: Vec<&str> = series["columns"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();
    assert!(
        columns
            .iter()
            .all(|c| !c.contains("DlaLoop") && !c.contains("dla.harness"))
    );
    p.ok(&["trend", "S1"]);
    for m in p.final_gate() {
        let text = m.to_string();
        assert!(
            !text.contains("DlaLoop") && !text.contains("dla.harness"),
            "{text}"
        );
    }
    let out = p.ok(&["ledger", "close-rest"]);
    let left: Vec<&str> = out["left"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| l["id"].as_str())
        .collect();
    assert_eq!(left.len(), 2, "both leads are left to decide: {out}");
}

#[test]
fn a_holder_that_is_only_the_measurements_makes_no_candidate_at_all() {
    let p = P::new();
    p.driver();
    let files = p.dumps(&[
        ("app.DlaLoop", "TCP", [200, 400, 600]),
        ("dla.harness.DlaDump", "ROOT_CLASS", [100, 300, 500]),
        ("jdk.jfr.internal.StringPool", "cache", [150, 350, 550]),
    ]);
    let v = p.summarize(&files);
    assert_eq!(v["gate"], "fail", "{v}");
    assert!(
        v["left_out"].as_str().unwrap().contains("StringPool.cache"),
        "{v}"
    );
    assert!(p.findings().is_empty(), "{:?}", p.findings());
    assert!(
        p.final_gate()
            .iter()
            .all(|m| m["kind"] != "finding" && m["kind"] != "growth"),
        "{:?}",
        p.final_gate()
    );
}

/// Ids of the findings the summarizer added, by owner.
fn finding_of(p: &P, owner: &str) -> String {
    p.findings()
        .iter()
        .find(|f| f["owner"] == owner)
        .unwrap_or_else(|| panic!("no finding for {owner}"))["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// The words of a printed command without its program, quotes honored.
fn words(cmd: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let (mut cur, mut quoted, mut any) = (String::new(), false, false);
    for ch in cmd.trim().chars() {
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
    out.into_iter().skip(1).collect()
}

fn run_printed(p: &P, cmd: &str) -> Value {
    let w = words(cmd);
    p.ok(&w.iter().map(String::as_str).collect::<Vec<_>>())
}

#[test]
fn a_growing_jdk_cache_is_decided_by_a_confirmed_explanation_and_the_newest_record_decides() {
    let p = P::new();
    p.driver();
    // The harness the dumps came from ran: its workload evidence puts a sibling under test.
    let m: dla_core::HarnessManifest = serde_json::from_value(serde_json::json!({
        "id": "H1", "runtime": "jvm", "mode": "driver", "scratch": "x",
        "run": {"argv": ["java", "app.DlaLoop"], "cwd": "."},
        "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
        "targets": [], "agent": "driver-jvm", "timeout_secs": 60,
    }))
    .unwrap();
    m.save(&p.run_dir).unwrap();
    let mut ledger = dla_core::Ledger::load(&p.run_dir).unwrap();
    ledger
        .add_evidence(dla_core::NewEvidence {
            source: dla_core::Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: dla_core::Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: dla_core::Authority::Test,
            kind: dla_core::EvidenceKind::Workload,
            result: serde_json::json!({"harness": "H1", "mode": "driver", "runtime": "jvm", "series": []}),
            raw: None,
            coverage: dla_core::evidence::EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
    drop(ledger);
    let files = p.dumps(OWN);
    let v = p.summarize(&files);
    let f = finding_of(&p, "java.net.InetAddress.cache");
    let growth = v["growth_evidence"].as_str().unwrap().to_owned();
    let left_for = |p: &P, f: &str| -> Option<String> {
        let out = p.ok(&["ledger", "close-rest"]);
        out["left"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| l["id"] == f)
            .and_then(|l| l["why"].as_str().map(str::to_owned))
    };
    assert!(left_for(&p, &f).is_some(), "{f} is not a lead");
    // The full route is in the final gate (the output of close-rest is cut at its bound).
    let missing = p.final_gate();
    let why = missing
        .iter()
        .find(|m| {
            m["id"] == f.as_str()
                && m["fix"]
                    .as_str()
                    .is_some_and(|x| x.contains("dla ledger confirm"))
        })
        .unwrap_or_else(|| panic!("{missing:?}"))["fix"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        why.contains("(if the growth is not a leak of the project: dla ledger add --sibling-of"),
        "{why}"
    );
    assert!(why.starts_with("dla alloc-test H1 --finding"), "{why}");
    // The alternative, as printed (N-OBSERVER chosen).
    let start = why
        .find("(if the growth is not a leak of the project: ")
        .unwrap()
        + "(if the growth is not a leak of the project: ".len();
    let alt: Vec<String> = why[start..why.rfind(')').unwrap()]
        .split("; ")
        .map(|c| c.replace("N-OBSERVER|N-INTENT", "N-OBSERVER"))
        .collect();
    let sibling = format!("{f}.n1");
    assert!(
        alt[0].starts_with(&format!("dla ledger add --sibling-of {f} ")),
        "{alt:?}"
    );
    run_printed(&p, &alt[0]);
    run_printed(&p, &alt[1]);
    // The record that names the holder: the one the route cites, or the summary it asks for.
    let record = if alt[2].starts_with("dla summarize heap-graph") {
        let v = run_printed(&p, &alt[2]);
        v["dumps"][0]["evidence"].as_str().unwrap().to_owned()
    } else {
        alt[2].rsplit("--evidence ").next().unwrap().to_owned()
    };
    // Any other record is refused, and the fix names the record of the holder.
    let (code, v) = p.run(&["ledger", "confirm", &sibling, "--evidence", &growth]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["error"]["code"], "leak-refutation-required", "{v}");
    assert!(
        v["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("retention-path record")
            || v["error"]["fix"]
                .as_str()
                .unwrap()
                .contains("dla ledger confirm"),
        "{v}"
    );
    let confirm = format!("dla ledger confirm {sibling} --evidence {record}");
    run_printed(&p, &confirm);
    let leads = |p: &P| -> bool {
        p.final_gate()
            .iter()
            .any(|m| m["id"] == f.as_str() && m["kind"] == "growth")
    };
    // close-rest closes the finding the explanation decided: no lead is left to it.
    assert!(
        left_for(&p, &f).is_none(),
        "the confirmed explanation lifts the lead"
    );
    assert!(!leads(&p), "{:?}", p.final_gate());
    // A newer growth record of the same dumps brings it back (the finding is closed unresolved,
    // so the final gate lists it as growth nothing decided).
    p.summarize(&files);
    assert!(leads(&p), "the newest record decides: {:?}", p.final_gate());
}

/// A `py-1.retain.json` with a module global of the project and one of the driver module.
const PY_RETAIN: &str = r#"{"dla_pyretain":1,"frames":1,"sites":[
 {"site":"pkg/store.py:9","blocks_first":20,"blocks_last":220,"found_tracked":200,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"module-global","module":"pkg.store","key":"ITEMS","of":"project"},
             "steps":[{"type":"pkg.store.Item","via":"[7]"},{"type":"list","via":".ITEMS"}]}]},
 {"site":"pkg/store.py:30","blocks_first":20,"blocks_last":220,"found_tracked":200,"found_untracked":0,"kept":40,
  "chains":[{"count":40,"root":{"kind":"module-global","module":"dla_harness","key":"_seen","of":"project"},
             "steps":[{"type":"pkg.store.Item","via":"[7]"},{"type":"list","via":"._seen"}]}]}
],"limitations":[]}"#;

#[test]
fn a_python_holder_in_the_drivers_module_is_left_out_and_the_projects_is_not() {
    let p = P::new();
    let d = p.run_dir.join("harness/H1/driver");
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join("dla_harness.py"), "_seen = []\n").unwrap();
    let f = "harness/H1/out/p1/capture/7/py-1.retain.json";
    let abs = p.dir.path().join(f);
    fs::create_dir_all(abs.parent().unwrap()).unwrap();
    fs::write(&abs, PY_RETAIN).unwrap();
    let v = p.ok(&["summarize", "heap-graph", "--backend", "py-retain", f]);
    assert_eq!(
        v["left_out"], "the measurement's own holders, left out: dla_harness._seen",
        "{v}"
    );
    let keys: Vec<&str> = v["holders"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|h| h["key"].as_str())
        .collect();
    assert_eq!(keys, ["static:pkg.store.ITEMS"], "{v}");
    let text = v["next"].to_string();
    assert!(text.contains("pkg.store.ITEMS"), "{text}");
    assert!(!text.contains("dla_harness"), "{text}");
    // The gate reads the driver's holder as the tests': no item asks for a finding at it.
    for m in p.final_gate() {
        assert!(!m.to_string().contains("dla_harness._seen"), "{m}");
    }
}
