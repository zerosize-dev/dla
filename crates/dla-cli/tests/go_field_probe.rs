//! Size probes for struct fields in a Go driver. The driver registers the objects it builds
//! with `dlaTrack`; every checkpoint reads the length of the probed fields of the tracked
//! objects by reflection. A tracked field that grows reads `grew` in `dla trend`; a probe that
//! could not be read says why, and `dla alloc-test` does not read a step-growing container as a
//! plateau of the whole process.
//!
//! The project is synthetic: a `health` package with a struct of two maps and a `sync.Map`.

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
	seen      sync.Map
}

func NewServer() *Server {
	return &Server{statusMap: map[string]int32{}, updates: map[string]map[chan int32]struct{}{}}
}

func (s *Server) SetServingStatus(name string, st int32) {
	s.mu.Lock()
	s.statusMap[name] = st
	s.mu.Unlock()
	s.seen.Store(name, st)
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
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "go.mod", "module example.test/svc\n\ngo 1.21\n");
        write(dir.path(), "health/server.go", HEALTH);
        write(
            dir.path(),
            "cmd/app/main.go",
            "package main\n\nimport \"example.test/svc/health\"\n\nfunc main() {\n\ts := health.NewServer()\n\ts.SetServingStatus(\"a\", 1)\n}\n",
        );
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

    fn id_at(&self, site: &str) -> String {
        let o = command(self.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        String::from_utf8(o.stdout)
            .unwrap()
            .lines()
            .skip(1)
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .find(|v| v["site"] == site)
            .unwrap_or_else(|| panic!("no finding at {site}"))["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn driver(&self, h: &str) -> std::path::PathBuf {
        self.run_dir
            .join("harness")
            .join(h)
            .join("driver/zz_dla_driver_test.go")
    }

    /// Fills the hooks: setup builds a server (tracked when `track`), a unit sets a status
    /// and opens one watch for a new name.
    fn fill(&self, h: &str, track: bool, small: bool) {
        let p = self.driver(h);
        let text = fs::read_to_string(&p).unwrap();
        let setup = if track {
            "\tdlaSrv = NewServer()\n\tdlaTrack(dlaSrv)"
        } else {
            "\tdlaSrv = NewServer()"
        };
        let text = text
            .replace("\tpanic(\"DLA-HOOK setup has not been filled in\")", setup)
            .replace(
                "\tpanic(\"DLA-HOOK unit has not been filled in\")",
                if small {
                    "\tdlaSrv.SetServingStatus(\"s\"+dlastrconv.Itoa(key), 1)"
                } else {
                    "\tname := \"svc-\" + dlastrconv.Itoa(key)\n\tdlaSrv.SetServingStatus(name, 1)\n\tdlaSrv.Watch(name, make(chan int32, 1))"
                },
            );
        fs::write(&p, format!("{text}\nvar dlaSrv *Server\n")).unwrap();
    }

    fn verdicts(&self, series: &str) -> Vec<Value> {
        let t = self.ok(&["trend", series]);
        t["targets"]
            .as_array()
            .unwrap_or_else(|| panic!("no targets: {t}"))
            .clone()
    }
}

fn verdict<'a>(v: &'a [Value], finding: &str) -> &'a Value {
    v.iter()
        .find(|t| t["finding"] == finding)
        .unwrap_or_else(|| panic!("no verdict for {finding}: {v:?}"))
}

#[test]
fn tracked_fields_read_grew_an_untracked_one_says_so_and_alloc_test_does_not_read_a_plateau() {
    if !have_go() {
        return;
    }
    let r = Run::new();
    let status = r.id_at("health/server.go:7");
    let updates = r.id_at("health/server.go:8");
    let seen = r.id_at("health/server.go:9");

    // The batch is the package; the probes are fields of Server.
    let gen_out = r.ok(&["harness", "gen", "--lang", "go", "--driver", "--next"]);
    assert_eq!(gen_out["summary"]["probes"], 3, "{gen_out}");
    let probes: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/probes.json")).unwrap(),
    )
    .unwrap();
    let p = &probes["probes"][0]["target"];
    assert_eq!(p["kind"], "go-field", "{probes}");
    assert_eq!(p["type_name"], "Server", "{probes}");
    assert_eq!(p["package_dir"], "health", "{probes}");
    let driver = fs::read_to_string(r.driver("H1")).unwrap();
    assert!(
        driver.contains("probe reads: length of field statusMap of Server (package health)")
            && driver.contains("dlaTrack(<the Server your setup built>)"),
        "{driver}"
    );
    r.fill("H1", true, false);
    let ran = r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let v = r.verdicts(ran["summary"]["series"][0].as_str().unwrap());
    for f in [&status, &updates] {
        let t = verdict(&v, f);
        assert_eq!(t["verdict"], "grew", "{t}");
        assert!(
            t["column"]
                .as_str()
                .unwrap()
                .starts_with("probe:health.Server."),
            "{t}"
        );
    }
    // A sync.Map has no length: its size is unknown, the verdict says so and is no `executed`.
    let t = verdict(&v, &seen);
    assert_eq!(t["verdict"], "not_loaded", "{t}");
    assert!(
        t["detail"].as_str().unwrap().contains("has no length"),
        "{t}"
    );

    // alloc-test on a tracked field: the size probe decides and says so.
    let a = r.ok(&["alloc-test", "H1", "--finding", &status]);
    assert_eq!(a["favors"], "leak", "{a}");
    assert_eq!(a["decided_by"], "probe:health.Server.statusMap", "{a}");

    // A harness whose setup builds the server but never tracks it: no size was read.
    r.ok(&[
        "harness", "gen", "--lang", "go", "--driver", "--probe", &status,
    ]);
    r.fill("H2", false, true);
    let ran = r.ok(&["harness", "run", "H2", "--mode", "both"]);
    let v = r.verdicts(ran["summary"]["series"][0].as_str().unwrap());
    let t = verdict(&v, &status);
    assert_eq!(t["verdict"], "not_loaded", "{t}");
    assert!(
        t["detail"]
            .as_str()
            .unwrap()
            .contains("no tracked object of type Server: call dlaTrack(<your Server>) in setup"),
        "{t}"
    );
    let a = r.ok(&["alloc-test", "H2", "--finding", &status]);
    assert_ne!(a["favors"], "non-leak", "{a}");
    let mut files: Vec<_> = fs::read_dir(r.run_dir.join("alloc"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();
    let full: Value =
        serde_json::from_str(&fs::read_to_string(files.last().unwrap()).unwrap()).unwrap();
    assert!(
        full["limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l.as_str().unwrap().contains("has no readings")),
        "{full}"
    );
}

#[test]
fn an_entry_path_harness_carries_the_same_field_probes() {
    if !have_go() {
        return;
    }
    let r = Run::new();
    let status = r.id_at("health/server.go:7");
    r.ok(&["harness", "gen", "--lang", "go", "--driver", "--next"]);
    r.fill("H1", true, false);
    let ran = r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let _ = r.verdicts(ran["summary"]["series"][0].as_str().unwrap());
    let out = r.ok(&[
        "harness", "gen", "--lang", "go", "--driver", "--entry", &status,
    ]);
    assert_eq!(out["summary"]["purpose"], "entry-path", "{out}");
    assert_eq!(out["summary"]["package"], "health", "{out}");
    let probes: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H2/probes.json")).unwrap(),
    )
    .unwrap();
    assert!(
        probes["probes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["target"]["kind"] == "go-field" && p["target"]["field"] == "statusMap"),
        "{probes}"
    );
}
