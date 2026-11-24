//! A run does not keep one full copy of the project per harness: `dla harness gen` thins the
//! copies of the older harnesses (`work/` keeps the files that differ from the project and
//! `work-thinned.json` lists the files taken out), `dla report` thins all of them, and using a
//! harness again brings its files back first.
//!
//! The project is a small Python package; each test skips, printing the reason, when `python3`
//! is missing.
#![cfg(unix)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn have_python() -> bool {
    let ok = Std::new("python3")
        .args([
            "-c",
            "import sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipping: python3 is not on PATH");
    }
    ok
}

struct Out {
    code: i32,
    text: String,
    err: String,
    json: Value,
}

fn dla_env(project: &Path, args: &[&str], env: &[(&str, &str)]) -> Out {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
        "DLA_KEEP_WORK",
    ] {
        c.env_remove(var);
    }
    c.env("DLA_CONSENT_TIERS", "1-4");
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        err: String::from_utf8_lossy(&o.stderr).into_owned(),
        text,
    }
}

fn dla(project: &Path, args: &[&str]) -> Out {
    dla_env(project, args, &[])
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    o.json
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

struct Run {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
}

impl Run {
    /// A project with a store module and a table module, a run with census and discovery.
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("project");
        write(&project, "shop/__init__.py", "");
        write(&project, "shop/store.py", "def pin(i):\n    return [i]\n");
        write(
            &project,
            "shop/cache.py",
            "REG = {}\n\n\ndef put(k):\n    REG[k] = k\n",
        );
        let init = ok(&project, &["run", "init", "--session-id", "t"]);
        let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
        ok(&project, &["census", "--lang", "python"]);
        ok(&project, &["workload", "discover", "--lang", "python"]);
        Self {
            _dir: dir,
            project,
            run_dir,
        }
    }

    /// `dla harness gen --lang python --driver --next`, with extra environment.
    fn gen_env(&self, env: &[(&str, &str)]) -> Value {
        let o = dla_env(
            &self.project,
            &["harness", "gen", "--lang", "python", "--driver", "--next"],
            env,
        );
        assert_eq!(o.code, 0, "{}", o.text);
        o.json
    }

    fn gen_one(&self) -> Value {
        self.gen_env(&[])
    }

    fn gens(&self, n: usize) -> Vec<Value> {
        (0..n).map(|_| self.gen_one()).collect()
    }

    fn hdir(&self, id: &str) -> PathBuf {
        self.run_dir.join("harness").join(id)
    }

    fn work(&self, id: &str) -> PathBuf {
        self.hdir(id).join("work")
    }

    fn list(&self, id: &str) -> PathBuf {
        self.hdir(id).join("work-thinned.json")
    }

    fn listed(&self, id: &str) -> Value {
        serde_json::from_str(&fs::read_to_string(self.list(id)).unwrap()).unwrap()
    }

    fn thinned(&self, id: &str) -> bool {
        self.list(id).is_file()
    }

    /// Fills the two hooks of the driver of `id` so that it runs.
    fn fill_hooks(&self, id: &str) {
        let edit = self.hdir(id).join("driver/dla_harness.py");
        let text = fs::read_to_string(&edit)
            .unwrap()
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK setup has not been filled in\")",
                "    from shop import store\n    _state[\"store\"] = store",
            )
            .replace(
                "    raise NotImplementedError(\"DLA-HOOK unit has not been filled in\")",
                "    _state[\"store\"].pin(i)",
            );
        assert!(!text.contains("has not been filled in"));
        fs::write(&edit, text).unwrap();
    }

    /// Every file of the harness copy, by path.
    fn snapshot(&self, id: &str) -> BTreeMap<String, Vec<u8>> {
        fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
            for e in fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(root, &p, out);
                } else {
                    let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
                    out.insert(rel, fs::read(&p).unwrap_or_default());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&self.work(id), &self.work(id), &mut out);
        out
    }
}

fn ids(v: &Value) -> Vec<String> {
    v["thinned"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|i| i.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn six_gens_thin_the_first_one_or_two_and_the_output_names_them() {
    if !have_python() {
        return;
    }
    let r = Run::new();
    let outs = r.gens(6);
    // Nothing is thinned while four harnesses besides the new one are not yet there.
    for (i, o) in outs.iter().take(4).enumerate() {
        assert!(
            o.get("thinned").is_none() && o.get("thinned_bytes").is_none(),
            "gen {}: {o}",
            i + 1
        );
    }
    assert_eq!(ids(&outs[4]), ["H1"], "{}", outs[4]);
    assert_eq!(ids(&outs[5]), ["H2"], "{}", outs[5]);
    for o in &outs[4..] {
        assert!(o["thinned_bytes"].as_u64().unwrap() > 0, "{o}");
    }
    for id in ["H1", "H2"] {
        assert!(r.thinned(id), "{id}");
        let list = r.listed(id);
        assert_eq!(list["dla_work_thin"], 1);
        let files = list["files"].as_array().unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
        assert!(paths.contains(&"shop/store.py"), "{paths:?}");
        let mut total = 0;
        for f in files {
            let p = f["path"].as_str().unwrap();
            // Exactly the removed files: gone from the copy, equal to the project's file.
            assert!(!r.work(id).join(p).exists(), "{p} is still in {id}");
            let theirs = fs::read(r.project.join(p)).unwrap();
            assert_eq!(f["len"].as_u64().unwrap(), theirs.len() as u64, "{p}");
            assert_eq!(f["digest"].as_str().unwrap().len(), 64, "{p}");
            total += theirs.len() as u64;
        }
        assert_eq!(list["bytes"].as_u64().unwrap(), total);
        // The directories and the harness's own files stay.
        assert!(r.work(id).join("shop").is_dir());
        assert!(r.hdir(id).join("driver/dla_harness.py").is_file());
        assert!(r.hdir(id).join("harness.json").is_file());
    }
    assert_eq!(
        outs[4]["thinned_bytes"].as_u64().unwrap(),
        r.listed("H1")["bytes"].as_u64().unwrap()
    );
    // The four used last and the new one keep whole copies.
    for id in ["H3", "H4", "H5", "H6"] {
        assert!(!r.thinned(id), "{id}");
        assert!(r.work(id).join("shop/store.py").is_file(), "{id}");
    }
}

#[test]
fn a_run_of_a_thinned_harness_fills_it_and_gives_the_result_of_one_never_thinned() {
    if !have_python() {
        return;
    }
    let r = Run::new();
    r.gen_one();
    r.fill_hooks("H1");
    r.gens(5);
    r.fill_hooks("H6");
    assert!(r.thinned("H1") && !r.thinned("H6"));
    let (p, w) = (
        r.project.join("shop/store.py"),
        r.work("H1").join("shop/store.py"),
    );
    assert!(!w.exists());

    let thin = ok(&r.project, &["harness", "run", "H1", "--mode", "fresh"]);
    assert_eq!(thin["ok"], true, "{thin}");
    // The file is back with the project's bytes and time, and the list is gone.
    assert_eq!(fs::read(&w).unwrap(), fs::read(&p).unwrap());
    assert_eq!(
        fs::metadata(&w).unwrap().modified().unwrap(),
        fs::metadata(&p).unwrap().modified().unwrap()
    );
    assert!(!r.thinned("H1"));
    assert!(r.hdir("H1").join("work.lock").is_file());

    let whole = ok(&r.project, &["harness", "run", "H6", "--mode", "fresh"]);
    assert_eq!(whole["ok"], true, "{whole}");
    let (a, b) = (&thin["summary"], &whole["summary"]);
    assert_eq!(a["points"], b["points"], "{a} {b}");
    assert_eq!(
        a["series"].as_array().map(Vec::len),
        b["series"].as_array().map(Vec::len)
    );
    let load = |run: &Value| -> Value {
        let id = run["summary"]["series"][0].as_str().unwrap();
        serde_json::from_str(
            &fs::read_to_string(r.run_dir.join(format!("series/{id}.json"))).unwrap(),
        )
        .unwrap()
    };
    let (sa, sb) = (load(&thin), load(&whole));
    let names = |s: &Value| -> Vec<String> {
        s["columns"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_owned())
            .collect()
    };
    assert_eq!(names(&sa), names(&sb));
    assert_eq!(
        sa["points"].as_array().unwrap().len(),
        sb["points"].as_array().unwrap().len()
    );
    assert_eq!(sa["warmup"], sb["warmup"]);
    let run_json = |h: &str, s: &str| -> Value {
        serde_json::from_str(&fs::read_to_string(r.hdir(h).join(format!("run-{s}.json"))).unwrap())
            .unwrap()
    };
    let (ra, rb) = (
        run_json("H1", thin["summary"]["series"][0].as_str().unwrap()),
        run_json("H6", whole["summary"]["series"][0].as_str().unwrap()),
    );
    for k in [
        "exits",
        "partial",
        "points",
        "units",
        "x_range",
        "detectors",
    ] {
        assert_eq!(ra[k], rb[k], "{k}");
    }
}

#[test]
fn a_harness_whose_lock_is_held_is_not_thinned() {
    if !have_python() {
        return;
    }
    let r = Run::new();
    r.gens(4);
    // A process that reads `work/` holds the shared lock.
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(r.hdir("H1").join("work.lock"))
        .unwrap();
    lock.lock_shared().unwrap();
    // `work.lock` of H1 is now the newest time of the run; make H1 the oldest again.
    lock.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(86_400))
        .unwrap();
    let fifth = r.gen_one();
    assert!(!r.thinned("H1"), "{fifth}");
    assert!(r.work("H1").join("shop/store.py").is_file());
    assert!(ids(&fifth).is_empty(), "{fifth}");
    let skipped = fifth["thin_skipped"].as_array().unwrap();
    assert_eq!(skipped[0]["harness"], "H1", "{fifth}");
    assert!(
        skipped[0]["reason"].as_str().unwrap().contains("in use"),
        "{fifth}"
    );
    drop(lock);
    let sixth = r.gen_one();
    assert!(ids(&sixth).contains(&"H1".to_owned()), "{sixth}");
    assert!(r.thinned("H1"));
}

#[test]
fn a_project_file_changed_after_the_thin_makes_the_run_refuse_and_nothing_changes() {
    if !have_python() {
        return;
    }
    let r = Run::new();
    r.gen_one();
    r.fill_hooks("H1");
    r.gens(4);
    assert!(r.thinned("H1"));
    write(
        &r.project,
        "shop/store.py",
        "def pin(i):\n    return [i, i]\n",
    );
    let before = r.snapshot("H1");
    let list = fs::read_to_string(r.list("H1")).unwrap();

    let o = dla(&r.project, &["harness", "run", "H1", "--mode", "fresh"]);
    assert_eq!(o.code, 2, "{}", o.text);
    let e = &o.json["error"];
    assert_eq!(e["code"], "work-copy-stale", "{}", o.text);
    assert!(
        e["message"].as_str().unwrap().contains("shop/store.py"),
        "{e}"
    );
    assert!(e["message"].as_str().unwrap().contains("1 file(s)"), "{e}");
    let fix = e["fix"].as_str().unwrap();
    assert!(fix.contains("dla harness gen --lang python"), "{fix}");
    assert!(fix.contains("reseal"), "{fix}");
    // Nothing was brought back, the list is as it was.
    assert_eq!(before, r.snapshot("H1"));
    assert_eq!(list, fs::read_to_string(r.list("H1")).unwrap());

    // The way out the fix names: a harness on the project as it is now.
    let g = r.gen_one();
    assert_eq!(g["ok"], true, "{g}");
    assert!(
        fs::read_to_string(r.work("H6").join("shop/store.py"))
            .unwrap()
            .contains("[i, i]")
    );
}

#[test]
fn with_keep_work_set_nothing_is_thinned() {
    if !have_python() {
        return;
    }
    let r = Run::new();
    for _ in 0..6 {
        let g = r.gen_env(&[("DLA_KEEP_WORK", "1")]);
        assert!(
            g.get("thinned").is_none() && g.get("thin_skipped").is_none(),
            "{g}"
        );
    }
    for n in 1..=6 {
        let id = format!("H{n}");
        assert!(!r.thinned(&id), "{id}");
        assert!(r.work(&id).join("shop/store.py").is_file(), "{id}");
    }
    // Without it the next gen thins as usual (the fill is not tied to the variable).
    let g = r.gen_one();
    assert_eq!(ids(&g), ["H1", "H2", "H3"], "{g}");
}

#[test]
fn report_thins_every_harness_and_a_run_brings_one_back() {
    if !have_python() {
        return;
    }
    let r = Run::new();
    r.gen_one();
    r.fill_hooks("H1");
    r.gens(2);
    for id in ["H1", "H2", "H3"] {
        assert!(!r.thinned(id), "{id}");
    }
    let o = dla(&r.project, &["report"]);
    assert_eq!(o.code, 0, "{}", o.err);
    for id in ["H1", "H2", "H3"] {
        assert!(r.thinned(id), "{id}");
        assert!(!r.work(id).join("shop/store.py").exists(), "{id}");
    }
    assert!(
        o.err.contains("thinned: [\"H1\",\"H2\",\"H3\"]"),
        "{}",
        o.err
    );
    // Reporting again has nothing left to thin.
    let again = dla(&r.project, &["report"]);
    assert!(!again.err.contains("thinned:"), "{}", again.err);
    let run = ok(&r.project, &["harness", "run", "H1", "--mode", "fresh"]);
    assert_eq!(run["ok"], true, "{run}");
    assert!(!r.thinned("H1") && r.work("H1").join("shop/store.py").is_file());
    assert!(r.thinned("H2"));
}
