//! End-to-end tests of `dla correlate`: a JVM harness whose project calls a native library
//! through JNI. The series and the run record are written by hand, the way `dla harness run`
//! writes them; everything after that is the real command line.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::{Value, json};

const SITE: &str = "native/src/lib.rs:40";
const COLUMN: &str = "native.live_bytes@native/src/lib.rs:40";

fn dla(project: &Path, worker: Option<&str>) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    if let Some(w) = worker {
        c.arg("--worker").arg(w);
    }
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn call(project: &Path, worker: Option<&str>, args: &[&str]) -> (i32, Value) {
    let out = dla(project, worker).args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), v)
}

fn ok(project: &Path, worker: Option<&str>, args: &[&str]) -> Value {
    let (code, v) = call(project, worker, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

struct Fx {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl Fx {
    fn project(&self) -> &Path {
        self.dir.path()
    }
}

/// A run with the worker `jvm` registered.
fn run_with_worker() -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    let init = ok(p, None, &["run", "init", "--session-id", "t"]);
    let run_id = init["run_id"].as_str().unwrap().to_owned();
    ok(p, None, &["run", "dispatch", "--workers", "jvm,rust"]);
    // The first worker command opens the slice.
    ok(p, Some("jvm"), &["probe"]);
    Fx {
        run_dir: p.join(".dla").join(run_id),
        dir,
    }
}

fn manifest(with_ffi: bool) -> Value {
    let mut m = json!({
        "schema_version": 1, "id": "H1", "runtime": "jvm", "mode": "command",
        "workload": "jvm:main:build.sh:run", "scratch": "harness/H1/work",
        "run": {"argv": ["sh", "./build.sh", "run", "{n}"], "cwd": "."},
        "iter_arg": "{n}", "warmup": 1, "iterations": 10, "step": 100,
        "key_modes": ["native"], "agent": "jvm-agent", "timeout_secs": 60,
    });
    if with_ffi {
        m["ffi"] = json!({
            "kind": "jni", "library": "codec", "native_dir": "native",
            "methods": [
                {"export": "Java_shop_codec_Native_open", "managed": "shop.codec.Native.open",
                 "declared_at": "kotlin/shop/Native.kt:9"},
                {"export": "Java_shop_codec_Native_close", "managed": "shop.codec.Native.close",
                 "declared_at": "kotlin/shop/Native.kt:13"},
            ],
        });
    }
    m
}

/// Writes the harness manifest, its series and the run record. `native_step` is the growth
/// per x of the site column, `rss_step` that of `os.rss_anon`.
fn write_harness(fx: &Fx, with_ffi: bool, native_step: f64, rss_step: f64) {
    let hdir = fx.run_dir.join("harness/H1");
    fs::create_dir_all(&hdir).unwrap();
    fs::write(hdir.join("harness.json"), manifest(with_ffi).to_string()).unwrap();
    let cols = [
        ("os.rss_anon", "process", None, 5.0e7, rss_step),
        ("jvm.heap_used", "heap", None, 2.5e6, 1450.0),
        ("native.live_bytes", "heap", None, 2.0e7, native_step),
        (COLUMN, "heap", Some(SITE), 6000.0, native_step),
    ];
    let columns: Vec<Value> = cols
        .iter()
        .map(|(n, r, s, _, _)| {
            json!({"name": n, "unit": "bytes", "kind": "gauge", "region": r, "baseline": false,
                "after_gc": false, "scope": "process", "site": s, "source": "test"})
        })
        .collect();
    let points: Vec<Value> = (0..11)
        .map(|k| {
            let k = f64::from(k);
            json!({"x": k + 1.0, "t_ms": 50 * (k as u64),
                "values": cols.iter().map(|c| c.3 + c.4 * k + (k % 3.0) * c.4 * 0.01)
                    .collect::<Vec<_>>()})
        })
        .collect();
    let series = json!({"schema_version": 1, "id": "S1",
        "source": {"kind": "harness", "harness": "H1", "workload": "jvm:main:build.sh:run",
            "mode": "native", "runtime": "jvm"},
        "axis": "n", "warmup": 1, "columns": columns, "points": points,
        "events": [], "limitations": []});
    fs::create_dir_all(fx.run_dir.join("series")).unwrap();
    fs::write(fx.run_dir.join("series/S1.json"), series.to_string()).unwrap();
    let record = json!({"harness": "H1", "series": ["S1"],
        "ffi_sites": [{"site": SITE, "method": "shop.codec.Native.open", "live_bytes": 61430816}]});
    fs::write(hdir.join("run-S1.json"), record.to_string()).unwrap();
}

/// Appends a workload evidence to the worker slice, as `dla harness run` does.
fn workload_evidence(fx: &Fx) -> String {
    let ledger = fx.run_dir.join("ledger.jvm.jsonl");
    let text = fs::read_to_string(&ledger).unwrap();
    let seq = text.lines().count() + 1;
    let n = text.matches("\"type\":\"evidence\"").count() + 1;
    let line = json!({"schema_version": 4, "seq": seq, "ts": "2026-01-02T03:05:00Z",
        "type": "evidence", "evidence_id": format!("jvm/E{n}"),
        "source": {"repo": {"commit": "abc"}}, "tool": {"name": "dla", "version": "1"},
        "authority": "test", "kind": "workload",
        "result": {"harness": "H1", "mode": "command"},
        "coverage": {"status": "complete"}});
    let mut f = fs::OpenOptions::new().append(true).open(&ledger).unwrap();
    writeln!(f, "{line}").unwrap();
    format!("jvm/E{n}")
}

fn ledger_lines(fx: &Fx) -> Vec<Value> {
    fs::read_to_string(fx.run_dir.join("ledger.jvm.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn a_matching_native_site_becomes_a_correlation_candidate_in_the_worker() {
    let fx = run_with_worker();
    write_harness(&fx, true, 3074.0, 7200.0);
    let p = fx.project();
    let v = ok(p, Some("jvm"), &["correlate", "H1"]);
    assert_eq!(v["candidates"], 1, "{v}");
    let found = &v["correlated"][0];
    assert_eq!(found["site"], SITE);
    assert_eq!(found["method"], "shop.codec.Native.open");
    let fid = found["finding"].as_str().unwrap().to_owned();
    assert!(fid.starts_with("jvm/F"), "{fid}");
    assert_eq!(v["raw"]["sha256"].as_str().unwrap().len(), 64);
    assert!(
        fx.run_dir
            .join(v["raw"]["path"].as_str().unwrap())
            .is_file()
    );

    let lines = ledger_lines(&fx);
    let added = lines
        .iter()
        .find(|l| l["type"] == "finding_added")
        .expect("finding added");
    let f = &added["finding"];
    assert_eq!(f["origin"], "correlation");
    assert_eq!(f["site"], SITE);
    assert_eq!(f["holder"], "native-handle/global-ref");
    assert_eq!(f["region"], "off-heap");
    assert_eq!(f["owner"], "shop.codec.Native.open");
    let ann = lines
        .iter()
        .find(|l| l["type"] == "finding_annotated")
        .expect("writer annotation");
    assert_eq!(ann["writers"], json!(["kotlin/shop/Native.kt:9"]));

    let ev = lines
        .iter()
        .find(|l| l["type"] == "evidence" && l["kind"] == "alloc-stack")
        .expect("correlation evidence");
    let r = &ev["result"];
    assert!(r.to_string().len() <= 2048);
    assert_eq!(r["boundary"]["kind"], "jni");
    assert_eq!(r["declared_at"], "kotlin/shop/Native.kt:9");
    assert!(r["native"]["slope"].as_f64().unwrap() > 3000.0);
    assert!(r["managed"]["slope"].as_f64().unwrap() > 7000.0);
    assert_eq!(r["signal"]["class"], "alloc-shim");
    assert!(ev["raw"].as_str().unwrap().starts_with("correlate/C"));

    // The core ledger did not get it: the candidate is in the worker's slice.
    let core = fs::read_to_string(fx.run_dir.join("ledger.jsonl")).unwrap();
    assert!(!core.contains("\"correlation\""));
}

#[test]
fn a_finding_of_another_runtime_at_the_site_does_not_suppress_the_candidate() {
    let fx = run_with_worker();
    write_harness(&fx, true, 3074.0, 7200.0);
    let p = fx.project();
    // A core finding (no worker prefix) at the same native site, e.g. from valgrind.
    let core = fx.run_dir.join("ledger.jsonl");
    let text = fs::read_to_string(&core).unwrap();
    let seq = text.lines().count() + 1;
    let eid = format!("E{}", text.matches("\"type\":\"evidence\"").count() + 1);
    let line = json!({"schema_version": 4, "seq": seq, "ts": "2026-01-02T03:04:00Z",
        "type": "evidence", "evidence_id": eid,
        "source": {"repo": {"commit": "abc"}}, "tool": {"name": "valgrind", "version": "1"},
        "authority": "test", "kind": "alloc-stack",
        "result": {"site": SITE, "bytes": 1000},
        "coverage": {"status": "complete"}});
    let mut f = fs::OpenOptions::new().append(true).open(&core).unwrap();
    writeln!(f, "{line}").unwrap();
    drop(f);
    ok(
        p,
        None,
        &[
            "ledger",
            "add",
            "--origin",
            "residual",
            "--evidence",
            "E1",
            "--site",
            SITE,
        ],
    );
    let v = ok(p, Some("jvm"), &["correlate", "H1"]);
    assert_eq!(v["candidates"], 1, "{v}");
    assert!(v.get("existing").is_none(), "{v}");
    assert!(
        v["correlated"][0]["finding"]
            .as_str()
            .unwrap()
            .starts_with("jvm/F")
    );
    // A second run adds neither a finding nor an evidence record.
    let before = ledger_lines(&fx).len();
    let again = ok(p, Some("jvm"), &["correlate", "H1"]);
    assert_eq!(again["candidates"], 0);
    assert_eq!(ledger_lines(&fx).len(), before);
}

#[test]
fn a_second_run_adds_nothing() {
    let fx = run_with_worker();
    write_harness(&fx, true, 3074.0, 7200.0);
    let p = fx.project();
    ok(p, Some("jvm"), &["correlate", "H1"]);
    let before = ledger_lines(&fx).len();
    let v = ok(p, Some("jvm"), &["correlate", "H1"]);
    assert_eq!(v["candidates"], 0);
    assert_eq!(v["skipped_duplicates"], 1);
    assert_eq!(ledger_lines(&fx).len(), before);
}

#[test]
fn the_shim_column_ties_to_the_finding_and_a_normal_confirmation_succeeds() {
    let fx = run_with_worker();
    write_harness(&fx, true, 3074.0, 7200.0);
    let p = fx.project();
    let w = Some("jvm");
    let found = ok(p, w, &["correlate", "H1"]);
    let f = found["correlated"][0]["finding"]
        .as_str()
        .unwrap()
        .to_owned();
    let trend = ok(p, w, &["trend", "S1"]);
    let col = trend["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == COLUMN)
        .unwrap_or_else(|| panic!("{trend}"));
    assert_eq!(col["candidates"], json!([f.clone()]), "{col}");
    let growth = trend["evidence"][0].as_str().unwrap().to_owned();

    let harness_ev = workload_evidence(&fx);
    ok(
        p,
        w,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth stops",
            "--evidence",
            &growth,
        ],
    );
    ok(p, w, &["ledger", "test", &f, "--evidence", &harness_ev]);
    let disc = {
        let ledger = fx.run_dir.join("ledger.jvm.jsonl");
        let text = fs::read_to_string(&ledger).unwrap();
        let seq = text.lines().count() + 1;
        let n = text.matches("\"type\":\"evidence\"").count() + 1;
        let line = json!({"schema_version": 4, "seq": seq, "ts": "2026-01-02T03:06:00Z",
            "type": "evidence", "evidence_id": format!("jvm/E{n}"),
            "source": {"repo": {"commit": "abc"}}, "tool": {"name": "dla", "version": "1"},
            "authority": "test", "kind": "discriminator",
            "result": {"test": "plateau", "favors": "leak", "supports": ["leak"], "finding": f},
            "coverage": {"status": "complete"}});
        let mut file = fs::OpenOptions::new().append(true).open(&ledger).unwrap();
        writeln!(file, "{line}").unwrap();
        format!("jvm/E{n}")
    };
    ok(
        p,
        w,
        &["ledger", "refute", &format!("{f}.n1"), "--evidence", &disc],
    );
    let done = ok(p, w, &["ledger", "confirm", &f, "--evidence", &growth]);
    assert_eq!(done["to"], "confirmed", "{done}");
    let state = ledger_lines(&fx)
        .into_iter()
        .find(|l| l["type"] == "finding_added" && l["finding"]["origin"] == "correlation")
        .unwrap();
    assert_eq!(state["finding"]["holder"], "native-handle/global-ref");
}

#[test]
fn without_the_correlation_the_native_column_confirms_nothing() {
    let fx = run_with_worker();
    write_harness(&fx, true, 3074.0, 7200.0);
    let p = fx.project();
    let w = Some("jvm");
    // A finding at another site in the same worker cannot take the native column.
    let ev = workload_evidence(&fx);
    let added = ok(
        p,
        w,
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            &ev,
            "--site",
            "kotlin/shop/Audit.kt:9",
            "--holder",
            "static/global",
        ],
    );
    let f = added["id"].as_str().unwrap().to_owned();
    let trend = ok(p, w, &["trend", "S1"]);
    let col = trend["passed"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["column"] == COLUMN)
        .unwrap();
    assert!(!col["candidates"].to_string().contains(&f), "{col}");
}

#[test]
fn a_site_that_does_not_match_the_footprint_is_reported_not_added() {
    let fx = run_with_worker();
    // The footprint grows 70 times faster than the site.
    write_harness(&fx, true, 100.0, 7200.0);
    let p = fx.project();
    let v = ok(p, Some("jvm"), &["correlate", "H1"]);
    assert_eq!(v["candidates"], 0, "{v}");
    assert_eq!(v["evidence"], json!([]));
    assert!(
        v["rejected"][0]["why"].as_str().unwrap().contains("ratio"),
        "{v}"
    );
    assert!(
        !ledger_lines(&fx)
            .iter()
            .any(|l| l["type"] == "finding_added")
    );
}

#[test]
fn a_harness_without_a_boundary_is_refused_with_the_fix() {
    let fx = run_with_worker();
    write_harness(&fx, false, 3074.0, 7200.0);
    let (code, v) = call(fx.project(), Some("jvm"), &["correlate", "H1"]);
    assert_eq!(code, 2, "{v}");
    let e = &v["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "no {k}: {v}");
    }
    assert!(e["fix"].as_str().unwrap().contains("dla probe"));
}

#[test]
fn an_unknown_harness_or_series_is_refused() {
    let fx = run_with_worker();
    write_harness(&fx, true, 3074.0, 7200.0);
    let (code, v) = call(fx.project(), Some("jvm"), &["correlate", "H9"]);
    assert_ne!(code, 0, "{v}");
    assert_eq!(v["ok"], false);
    let (code, v) = call(
        fx.project(),
        Some("jvm"),
        &["correlate", "H1", "--series", "S9"],
    );
    assert_eq!(code, 2, "{v}");
    assert!(v["error"]["fix"].as_str().unwrap().contains("dla"));
}

#[test]
fn a_run_record_without_boundary_sites_is_an_empty_result() {
    let fx = run_with_worker();
    write_harness(&fx, true, 3074.0, 7200.0);
    let rec = fx.run_dir.join("harness/H1/run-S1.json");
    fs::write(
        rec,
        json!({"harness": "H1", "series": ["S1"], "ffi_sites": []}).to_string(),
    )
    .unwrap();
    let v = ok(fx.project(), Some("jvm"), &["correlate", "H1"]);
    assert_eq!(v["candidates"], 0);
    assert_eq!(v["sites"], 0);
    assert!(
        v["next"][0]
            .as_str()
            .unwrap()
            .contains("no native allocation site")
    );
}
