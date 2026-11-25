//! A Go driver for a package of a nested module. The repository has a module at its root and
//! another in `svc/`; the default workload is in the root module, the holder is in the package
//! `svc/store`. `dla harness gen --lang go --driver --package svc/store` builds and runs the
//! driver from the module that holds the package, the work copy gets a `go.work` that lists
//! both modules, the run measures the holder growing and the coverage point reads the target's
//! function as executed. The same repository without the `replace` line builds through the
//! `go.work`. A package whose only non-test Go files are generated is refused with that reason
//! and `--next` goes on to the next package.
//!
//! The tests build with the real `go` and skip, printing the reason, when it is not on PATH.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

const STORE: &str = r#"package store

import "example.test/root/util"

type Cache struct {
	byID map[int]string
}

func NewCache() *Cache {
	return &Cache{byID: map[int]string{}}
}

func (i *Cache) Add(id int, name string) {
	i.byID[id] = util.Tag(name)
}
"#;

fn project(dir: &Path, replace: bool) {
    write(dir, "go.mod", "module example.test/root\n\ngo 1.21\n");
    write(
        dir,
        "util/util.go",
        "package util\n\nfunc Tag(s string) string { return \"t-\" + s }\n",
    );
    write(
        dir,
        "cmd/app/main.go",
        "package main\n\nimport \"example.test/root/util\"\n\nfunc main() {\n\t_ = util.Tag(\"a\")\n}\n",
    );
    let replace = if replace {
        "\nreplace example.test/root => ../\n"
    } else {
        ""
    };
    write(
        dir,
        "svc/go.mod",
        &format!(
            "module example.test/svc\n\ngo 1.21\n\nrequire example.test/root v0.0.0\n{replace}"
        ),
    );
    write(dir, "svc/store/store.go", STORE);
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
        "GOWORK",
        "GOFLAGS",
    ] {
        c.env_remove(var);
    }
    // No module is fetched: the modules of the repository are all the project needs.
    c.env("GOPROXY", "off");
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
    fn new(setup: impl FnOnce(&Path), init: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        setup(dir.path());
        let o = command(dir.path())
            .env("DLA_CONSENT_TIERS", "1,2,3")
            .args(["run", "init", "--session-id", "t", "--budget", "3h"])
            .args(init)
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

    fn harness(&self, h: &str) -> std::path::PathBuf {
        self.run_dir.join("harness").join(h)
    }

    /// Fills the hooks: setup builds the cache and tracks it, a unit adds one entry.
    fn fill(&self, h: &str) {
        let p = self.harness(h).join("driver/zz_dla_driver_test.go");
        let text = fs::read_to_string(&p).unwrap();
        let text = text
            .replace(
                "\tpanic(\"DLA-HOOK setup has not been filled in\")",
                "\tdlaCache = NewCache()\n\tdlaTrack(dlaCache)",
            )
            .replace(
                "\tpanic(\"DLA-HOOK unit has not been filled in\")",
                "\tdlaCache.Add(key, \"n\")",
            );
        fs::write(&p, format!("{text}\nvar dlaCache *Cache\n")).unwrap();
    }
}

/// The driver of `svc/store` is built and run in `svc`, grows the holder and covers its writer.
fn check_nested_driver(r: &Run) {
    let holder = r.id_at("svc/store/store.go:6");
    let out = r.ok(&[
        "harness",
        "gen",
        "--lang",
        "go",
        "--driver",
        "--package",
        "svc/store",
        "--probe",
        &holder,
    ]);
    assert_eq!(out["summary"]["package"], "svc/store", "{out}");
    let m: Value =
        serde_json::from_str(&fs::read_to_string(r.harness("H1").join("harness.json")).unwrap())
            .unwrap();
    // Built and run from the module of the package, which is not the workload's module.
    assert_eq!(m["build"]["cwd"], "svc", "{m}");
    assert_eq!(m["run"]["cwd"], "svc", "{m}");
    assert_eq!(
        m["run"]["argv"].as_array().unwrap().last().unwrap(),
        "./store"
    );
    assert!(
        r.harness("H1")
            .join("work/svc/store/zz_dla_driver_test.go")
            .is_file()
    );
    // The copy lists both modules in a go.work.
    let work = fs::read_to_string(r.harness("H1").join("work/go.work")).unwrap();
    assert!(work.contains("use (\n\t.\n\t./svc\n)"), "{work}");
    assert!(
        m["gaps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g.as_str().unwrap().contains("has 2 Go modules")),
        "{m}"
    );

    r.fill("H1");
    let ran = r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let series = ran["summary"]["series"][0].as_str().unwrap().to_owned();
    let t = r.ok(&["trend", &series]);
    let v = t["targets"]
        .as_array()
        .unwrap_or_else(|| panic!("no targets: {t}"))
        .iter()
        .find(|x| x["finding"] == holder.as_str())
        .unwrap_or_else(|| panic!("no verdict for {holder}: {t}"));
    assert_eq!(v["verdict"], "grew", "{v}");
    // The coverage point read the writer in `svc/store/store.go` as executed.
    let executed: Value = serde_json::from_str(
        &fs::read_to_string(r.harness("H1").join(format!("executed-{series}.json"))).unwrap(),
    )
    .unwrap();
    assert_eq!(executed["kind"], "functions", "{executed}");
    assert!(
        executed["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |f| f["file"].as_str().unwrap().ends_with("svc/store/store.go")
                    && f["name"] == "Add"
            ),
        "{executed}"
    );
}

#[test]
fn a_driver_for_a_package_of_a_nested_module_builds_runs_and_reads_coverage() {
    if !have_go() {
        return;
    }
    let r = Run::new(|d| project(d, true), &[]);
    check_nested_driver(&r);
}

#[test]
fn without_a_replace_line_the_modules_of_the_repository_resolve_through_the_go_work() {
    if !have_go() {
        return;
    }
    let r = Run::new(|d| project(d, false), &[]);
    // The project alone does not build: the root module is not a released version.
    let alone = std::process::Command::new("go")
        .args(["build", "./..."])
        .current_dir(r.dir.path().join("svc"))
        .env("GOPROXY", "off")
        .env_remove("GOWORK")
        .output()
        .unwrap();
    assert!(!alone.status.success());
    check_nested_driver(&r);
}

#[test]
fn a_package_of_generated_files_is_refused_with_that_reason_and_next_goes_on() {
    if !have_go() {
        return;
    }
    let r = Run::new(
        |d| {
            write(d, "go.mod", "module example.test/g\n\ngo 1.21\n");
            write(
                d,
                "pb/pb.go",
                "// Code generated by a tool. DO NOT EDIT.\n\npackage pb\n\nvar Reg = map[string]int{}\n\nfunc Add(k string) { Reg[k] = 1 }\n",
            );
            write(
                d,
                "store/store.go",
                "package store\n\nvar sessions = map[string][]byte{}\n\nfunc Put(k string, v []byte) { sessions[k] = v }\n",
            );
        },
        &["--include", "pb"],
    );
    let o = r.run(&[
        "harness",
        "gen",
        "--lang",
        "go",
        "--driver",
        "--package",
        "pb",
    ]);
    assert_ne!(o.code, 0, "{}", o.text);
    let msg = o.json["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("holds no Go package to drive") && msg.contains("are generated"),
        "{msg}"
    );
    assert!(!msg.contains("holds no Go package to drive\""), "{msg}");
    assert!(!r.harness("H1").exists());
    // `--next`: pb ranks first (or ties), is skipped, and the line leads the output.
    let out = r.ok(&["harness", "gen", "--lang", "go", "--driver", "--next"]);
    assert_eq!(out["summary"]["package"], "store", "{out}");
    let attention = out["_attention"].as_array().unwrap();
    assert!(
        attention.iter().any(|a| {
            let a = a.as_str().unwrap();
            a.contains("skipped package pb (") && a.contains("generated")
        }),
        "{out}"
    );
}
