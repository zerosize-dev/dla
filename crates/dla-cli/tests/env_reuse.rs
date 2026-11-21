//! A harness rerun gets the dependency paths its last run had: `dla harness run` records
//! the values of a fixed list of variables (`PYTHONPATH`...), and every command that runs the
//! harness's processes again sets the ones its own environment lacks and says so
//! (`env_reused`, and a line in the harness log). Python driver harness whose setup imports
//! a module that exists only in a directory named by `PYTHONPATH`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

struct Out {
    code: i32,
    json: Value,
    text: String,
}

struct Run {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl Run {
    fn dla(&self, env: &[(&str, &Path)], args: &[&str]) -> Out {
        let mut c = Command::cargo_bin("dla").unwrap();
        c.arg("--project").arg(self.dir.path().join("project"));
        for var in [
            "DLA_RUN",
            "DLA_WORKER",
            "DLA_DEFAULT_BUDGET",
            "DLA_CONSENT_TIERS",
            "DLA_CONSENT_RAW_VALUES",
            "CLAUDE_SESSION_ID",
            "CLAUDE_CODE_SESSION_ID",
            "DLA_DATA_DIR",
            "PYTHONPATH",
            "NODE_PATH",
            "CLASSPATH",
            "GOFLAGS",
            "GOWORK",
            "LD_LIBRARY_PATH",
            "DYLD_LIBRARY_PATH",
            "VIRTUAL_ENV",
        ] {
            c.env_remove(var);
        }
        c.env("DLA_CONSENT_TIERS", "1,2,3");
        for (k, v) in env {
            c.env(k, v);
        }
        let o = c.args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        Out {
            code: o.status.code().unwrap(),
            json: serde_json::from_str(&text).unwrap_or(Value::Null),
            text,
        }
    }

    fn ok(&self, env: &[(&str, &Path)], args: &[&str]) -> Value {
        let o = self.dla(env, args);
        assert_eq!(o.code, 0, "{args:?}: {}", o.text);
        o.json
    }

    fn mark(&self) -> String {
        fs::read_to_string(self.dir.path().join("mark")).unwrap_or_default()
    }

    fn record_of(&self, series: &str) -> Value {
        let text = fs::read_to_string(
            self.run_dir
                .join("harness/H1")
                .join(format!("run-{series}.json")),
        )
        .unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn logs(&self) -> String {
        let dir = self.run_dir.join("harness/H1/logs");
        let mut all = String::new();
        for e in fs::read_dir(dir).unwrap().flatten() {
            all.push_str(&fs::read_to_string(e.path()).unwrap_or_default());
        }
        all
    }
}

/// A project whose cache module imports `extlib`, which only the `deps` and `other` directories
/// hold; a driver whose setup writes where `extlib` was found into the file `mark`.
fn prepared() -> (Run, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let project = root.join("project");
    write(&project, "shop/__init__.py", "");
    write(
        &project,
        "shop/cache.py",
        "import functools\n\nimport extlib\n\n\n@functools.lru_cache(None)\ndef price_for(handler, key):\n    return (handler, key, extlib.NAME)\n",
    );
    write(root, "deps/extlib.py", "NAME = 'deps'\n");
    write(root, "other/extlib.py", "NAME = 'other'\n");
    let (deps, other) = (root.join("deps"), root.join("other"));
    let r = Run {
        run_dir: PathBuf::new(),
        dir,
    };
    let init = r.ok(&[], &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    let r = Run { run_dir, ..r };
    r.ok(&[], &["sweep", "--lang", "python"]);
    r.ok(&[], &["census", "--lang", "python"]);
    r.ok(&[], &["workload", "discover", "--lang", "python"]);
    r.ok(
        &[],
        &["harness", "gen", "--lang", "python", "--driver", "--next"],
    );
    let driver = r.run_dir.join("harness/H1/driver/dla_harness.py");
    let text = fs::read_to_string(&driver).unwrap();
    let text = text
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
            "    import os\n    from shop import cache\n    import extlib\n    open(os.environ['DLA_TEST_MARK'], 'w').write(extlib.__file__)\n    _state['cache'] = cache",
        )
        .replace(
            "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
            "    _state['cache'].price_for('h', key)",
        );
    fs::write(&driver, text).unwrap();
    (r, deps, other)
}

fn marked<'a>(r: &Run, extra: &'a [(&'a str, &'a Path)]) -> Vec<(&'a str, PathBuf)> {
    let mut v: Vec<(&str, PathBuf)> = extra.iter().map(|(k, p)| (*k, p.to_path_buf())).collect();
    v.push(("DLA_TEST_MARK", r.dir.path().join("mark")));
    v
}

fn dla_with(r: &Run, extra: &[(&str, &Path)], args: &[&str]) -> Out {
    let env = marked(r, extra);
    let env: Vec<(&str, &Path)> = env.iter().map(|(k, p)| (*k, p.as_path())).collect();
    r.dla(&env, args)
}

fn ok_with(r: &Run, extra: &[(&str, &Path)], args: &[&str]) -> Value {
    let o = dla_with(r, extra, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    o.json
}

#[test]
fn a_rerun_without_the_variable_gets_the_value_of_the_last_run_and_says_so() {
    let (r, deps, other) = prepared();
    let secret = Path::new("hunter2");
    // Without the variable the first run cannot import the module.
    let o = dla_with(&r, &[], &["harness", "run", "H1", "--mode", "both"]);
    assert_ne!(o.code, 0, "{}", o.text);

    // With it, the run records it, and nothing else of the environment.
    let first = ok_with(
        &r,
        &[("PYTHONPATH", &deps), ("DLA_TEST_TOKEN", secret)],
        &["harness", "run", "H1", "--mode", "both"],
    );
    assert!(first.get("env_reused").is_none(), "{first}");
    assert!(r.mark().contains("deps"), "{}", r.mark());
    let series = first["summary"]["series"][0].as_str().unwrap().to_owned();
    let rec = r.record_of(&series);
    assert_eq!(
        rec["dependency_env"],
        serde_json::json!({"PYTHONPATH": deps.to_string_lossy()})
    );
    assert!(!rec.to_string().contains("hunter2"), "{rec}");

    // The same command typed without it works and names what it took.
    fs::remove_file(r.dir.path().join("mark")).unwrap();
    let again = ok_with(&r, &[], &["harness", "run", "H1", "--mode", "both"]);
    assert_eq!(again["summary"]["env_reused"], serde_json::json!(["PYTHONPATH"]), "{again}");
    assert!(r.mark().contains("deps"), "{}", r.mark());
    assert!(
        r.logs().contains("dla: reused from the last run of H1: PYTHONPATH"),
        "{}",
        r.logs()
    );
    // The rerun passes it on: its own record has the value it ran with.
    let series = again["summary"]["series"][0].as_str().unwrap();
    assert_eq!(
        r.record_of(series)["dependency_env"]["PYTHONPATH"],
        deps.to_string_lossy().as_ref()
    );
    assert_eq!(r.record_of(series)["env_reused"], serde_json::json!(["PYTHONPATH"]));

    // A value set in the environment wins, and is the one recorded next.
    fs::remove_file(r.dir.path().join("mark")).unwrap();
    let set = ok_with(
        &r,
        &[("PYTHONPATH", &other)],
        &["harness", "run", "H1", "--mode", "both"],
    );
    assert!(set["summary"].get("env_reused").is_none(), "{set}");
    assert!(r.mark().contains("other"), "{}", r.mark());
    let series = set["summary"]["series"][0].as_str().unwrap();
    assert_eq!(
        r.record_of(series)["dependency_env"]["PYTHONPATH"],
        other.to_string_lossy().as_ref()
    );
    // The latest run is the one that counts.
    fs::remove_file(r.dir.path().join("mark")).unwrap();
    ok_with(&r, &[], &["harness", "run", "H1", "--mode", "both"]);
    assert!(r.mark().contains("other"), "{}", r.mark());
}

#[test]
fn alloc_test_reuses_the_environment_of_the_last_run_too() {
    let (r, deps, _) = prepared();
    let first = ok_with(
        &r,
        &[("PYTHONPATH", &deps)],
        &["harness", "run", "H1", "--mode", "both"],
    );
    let series = first["summary"]["series"][0].as_str().unwrap().to_owned();
    ok_with(&r, &[("PYTHONPATH", &deps)], &["trend", &series]);
    let f = {
        let (_, lines) = ledger_list(&r);
        lines
            .iter()
            .find(|l| l["site"].as_str() == Some("shop/cache.py:6"))
            .unwrap_or_else(|| panic!("{lines:?}"))["id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    fs::remove_file(r.dir.path().join("mark")).unwrap();
    let a = dla_with(&r, &[], &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a.json["env_reused"], serde_json::json!(["PYTHONPATH"]), "{}", a.text);
    assert!(r.mark().contains("deps"), "{}", r.mark());
}

fn ledger_list(r: &Run) -> (Value, Vec<Value>) {
    let o = r.ok_text(&["ledger", "list", "--all"]);
    let mut lines: Vec<Value> = o.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    let totals = lines.remove(0);
    (totals, lines)
}

impl Run {
    fn ok_text(&self, args: &[&str]) -> String {
        self.dla(&[], args).text
    }
}
