//! A Go measurement batch is one package: `dla harness gen --lang go --driver --next` takes
//! the best-ranked pending candidate and the pending candidates of its package directory, and
//! generates the driver inside that package, because a Go test file reads the unexported
//! names of its own package only.
//!
//! The project is synthetic: `health` (two fields of one struct), `store` (a package
//! variable and a field) and `ignored` (a file no build includes).

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

const HEALTH: &str = r#"package health

import "sync"

type Server struct {
	mu        sync.Mutex
	statusMap map[string]int32
	updates   map[string]map[chan int32]struct{}
}

func NewServer() *Server {
	return &Server{statusMap: map[string]int32{}, updates: map[string]map[chan int32]struct{}{}}
}

func (s *Server) SetServingStatus(name string, st int32) {
	s.mu.Lock()
	s.statusMap[name] = st
	s.mu.Unlock()
}

func (s *Server) Watch(name string, ch chan int32) {
	s.mu.Lock()
	if s.updates[name] == nil {
		s.updates[name] = map[chan int32]struct{}{}
	}
	s.updates[name][ch] = struct{}{}
	s.mu.Unlock()
}
"#;

fn project(dir: &Path, health_builds: bool) {
    write(dir, "go.mod", "module example.test/svc\n\ngo 1.21\n");
    let health = if health_builds {
        HEALTH.to_owned()
    } else {
        format!("//go:build ignore\n\n{HEALTH}")
    };
    write(dir, "health/server.go", &health);
    write(
        dir,
        "store/store.go",
        "package store\n\nvar sessions = map[string][]byte{}\n\nfunc Put(k string, v []byte) { sessions[k] = v }\n\ntype Index struct {\n\tbyID map[int]string\n}\n\nfunc NewIndex() *Index { return &Index{byID: map[int]string{}} }\n\nfunc (i *Index) Add(id int, name string) { i.byID[id] = name }\n",
    );
    write(
        dir,
        "ignored/only.go",
        "//go:build ignore\n\npackage ignored\n\nvar seen = map[string]int{}\n\nfunc Note(k string) { seen[k]++ }\n",
    );
    write(
        dir,
        "cmd/app/main.go",
        "package main\n\nimport (\n\t\"example.test/svc/health\"\n\t\"example.test/svc/store\"\n)\n\nfunc main() {\n\ts := health.NewServer()\n\ts.SetServingStatus(\"a\", 1)\n\tstore.Put(\"k\", nil)\n\tstore.NewIndex().Add(1, \"x\")\n}\n",
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

fn have_go() -> bool {
    let ok = std::process::Command::new("go")
        .arg("version")
        .stdin(std::process::Stdio::null())
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("skipping: go is not on PATH");
    }
    ok
}

impl Run {
    fn new(health_builds: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path(), health_builds);
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
            &["sweep", "--lang", "go"][..],
            &["census", "--lang", "go"][..],
            &["workload", "discover", "--lang", "go"][..],
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

    /// `(id, site)` of every finding.
    fn sites(&self) -> Vec<(String, String)> {
        let o = command(self.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        String::from_utf8(o.stdout)
            .unwrap()
            .lines()
            .skip(1)
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .map(|v| {
                (
                    v["id"].as_str().unwrap().to_owned(),
                    v["site"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    }

    fn id_at(&self, site: &str) -> String {
        self.sites()
            .into_iter()
            .find(|(_, s)| s == site)
            .unwrap_or_else(|| panic!("no finding at {site}"))
            .0
    }
}

fn targets(out: &Value) -> Vec<String> {
    out["summary"]["targets"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|t| t["finding"].as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_next_batch_is_the_candidates_of_one_package_and_the_driver_is_in_it() {
    if !have_go() {
        return;
    }
    let r = Run::new(true);
    // Rank order: health Server.updates, store sessions, health Server.statusMap, store
    // Index.byID. The batch is the top candidate's package, not the first four of the queue.
    let (updates, status) = (r.id_at("health/server.go:8"), r.id_at("health/server.go:7"));
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["go"]["package"], "health", "{st}");
    assert!(
        st["blocking"][0]["description"]
            .as_str()
            .unwrap()
            .contains("Next batch (all in package health): "),
        "{st}"
    );
    let out = r.ok(&["harness", "gen", "--lang", "go", "--driver", "--next"]);
    assert_eq!(targets(&out), [updates.clone(), status.clone()], "{out}");
    assert_eq!(out["summary"]["package"], "health", "{out}");
    let m: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/harness.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(m["targets"], serde_json::json!([updates, status]));
    let argv = m["run"]["argv"].as_array().unwrap();
    assert_eq!(argv.last().unwrap(), "./health", "{m}");
    let placed = r
        .run_dir
        .join("harness/H1/work/health/zz_dla_driver_test.go");
    let text = fs::read_to_string(&placed).unwrap();
    assert!(text.starts_with("// Code generated"), "{text}");
    assert!(text.contains("package health\n"), "{text}");
    // Another package's directory got no driver.
    assert!(
        !r.run_dir
            .join("harness/H1/work/store/zz_dla_driver_test.go")
            .exists()
    );
    // The block says which package the driver is in and what it can reach.
    assert!(
        text.contains("// The driver is in package health (directory health): the hooks can read its unexported names"),
        "{text}"
    );
}

#[test]
fn next_with_package_takes_the_candidates_of_that_package() {
    if !have_go() {
        return;
    }
    let r = Run::new(true);
    let (sessions, by_id) = (r.id_at("store/store.go:3"), r.id_at("store/store.go:8"));
    let out = r.ok(&[
        "harness",
        "gen",
        "--lang",
        "go",
        "--driver",
        "--next",
        "--package",
        "store",
    ]);
    assert_eq!(targets(&out), [sessions, by_id], "{out}");
    assert_eq!(out["summary"]["package"], "store", "{out}");
    assert!(
        r.run_dir
            .join("harness/H1/work/store/zz_dla_driver_test.go")
            .is_file()
    );
    // A package with no pending candidate is refused with the packages that have some.
    let o = r.run(&[
        "harness",
        "gen",
        "--lang",
        "go",
        "--driver",
        "--next",
        "--package",
        "cmd/app",
    ]);
    assert_ne!(o.code, 0, "{}", o.text);
    let fix = o.json["error"]["fix"].as_str().unwrap();
    assert!(fix.contains("health"), "{fix}");
    assert!(
        o.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("no pending candidate of go has its site in that package"),
        "{}",
        o.text
    );
    assert!(!r.run_dir.join("harness/H2").exists());
}

#[test]
fn a_package_that_cannot_hold_the_driver_is_skipped_and_its_candidates_stay_pending() {
    if !have_go() {
        return;
    }
    // `health` ranks first and no build includes its only file.
    let r = Run::new(false);
    let (updates, sessions, by_id) = (
        r.id_at("health/server.go:10"),
        r.id_at("store/store.go:3"),
        r.id_at("store/store.go:8"),
    );
    let out = r.ok(&["harness", "gen", "--lang", "go", "--driver", "--next"]);
    assert_eq!(targets(&out), [sessions, by_id], "{out}");
    assert_eq!(out["summary"]["package"], "store", "{out}");
    let note = out["summary"]["next_note"].as_str().unwrap();
    assert!(
        note.contains("package health cannot hold the driver: its 2 best-ranked candidate(s)")
            && note.contains("(go list fails there"),
        "{note}"
    );
    assert!(note.contains("stay pending and are not targeted"), "{note}");
    // The line that leads the output names the skipped package and why.
    assert!(
        out["_attention"][0]
            .as_str()
            .unwrap()
            .starts_with("--next skipped package health (go list fails there"),
        "{out}"
    );
    // The skipped candidates are still pending: nothing targeted them.
    let list = command(r.dir.path())
        .args(["ledger", "list", "--all"])
        .output()
        .unwrap();
    let text = String::from_utf8(list.stdout).unwrap();
    let line = text
        .lines()
        .skip(1)
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|v| v["id"] == updates.as_str())
        .unwrap();
    assert_eq!(line["queue"], "pending", "{line}");
}

#[test]
fn a_focus_batch_stays_inside_one_package() {
    if !have_go() {
        return;
    }
    let r = Run::new(true);
    let driver = |h: &str| {
        r.run_dir
            .join("harness")
            .join(h)
            .join("driver/zz_dla_driver_test.go")
    };
    // Two harnesses whose units do nothing: every target of both is missed (health first,
    // then store).
    let mut missed = Vec::new();
    for pkg in ["health", "store"] {
        let args = [
            "harness",
            "gen",
            "--lang",
            "go",
            "--driver",
            "--next",
            "--package",
            pkg,
        ];
        let out = r.ok(&args);
        let h = out["summary"]["harness"].as_str().unwrap().to_owned();
        let text = fs::read_to_string(driver(&h)).unwrap();
        let text = text
            .replace("\tpanic(\"DLA-HOOK setup has not been filled in\")", "")
            .replace("\tpanic(\"DLA-HOOK unit has not been filled in\")", "");
        fs::write(driver(&h), text).unwrap();
        let ran = r.ok(&["harness", "run", &h, "--mode", "both"]);
        let trend = r.ok(&["trend", ran["summary"]["series"][0].as_str().unwrap()]);
        assert_eq!(trend["targets"].as_array().unwrap().len(), 2, "{trend}");
        assert!(
            trend["targets"]
                .as_array()
                .unwrap()
                .iter()
                .all(|t| t["verdict"] == "not_loaded"),
            "{trend}"
        );
        missed.push(h);
    }
    // The best-ranked pending candidate (a health field) was missed: the batch is a focus
    // batch of the missed health candidates, though two more missed ones wait in `store`.
    let out = r.ok(&["harness", "gen", "--lang", "go", "--driver", "--next"]);
    let s = &out["summary"];
    assert_eq!(s["package"], "health", "{out}");
    assert!(
        s["focus"].as_str().unwrap().contains("focus batch"),
        "{out}"
    );
    let m: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H3/harness.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(m["purpose"], "focus", "{m}");
    assert_eq!(m["targets"].as_array().unwrap().len(), 2, "{m}");
    assert_eq!(
        m["run"]["argv"].as_array().unwrap().last().unwrap(),
        "./health"
    );
    assert!(
        r.run_dir
            .join("harness/H3/work/health/zz_dla_driver_test.go")
            .is_file()
    );
}
