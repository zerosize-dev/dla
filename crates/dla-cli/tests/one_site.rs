//! One site is one finding: a rule hit and a census entry at the same site are one ledger
//! finding, and `dla harness gen --probe F<n>` says what it did with the finding it was asked
//! for. Through the binary, on a small Python project with an `lru_cache(None)` function.

use std::{fs, path::Path};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// `shop/cache.py:3` is a decorator line (`@functools.lru_cache(None)`), line 4 its `def`.
fn project(dir: &Path) {
    write(dir, "shop/__init__.py", "");
    write(
        dir,
        "shop/cache.py",
        "import functools\n\n@functools.lru_cache(None)\ndef price_for(handler, key):\n    return (handler, key)\n\n\nREGISTRY = {}\n\n\ndef register(key, value):\n    REGISTRY[key] = value\n",
    );
    write(
        dir,
        "shop/load.py",
        "import argparse\n\nfrom .cache import price_for, register\n\n\ndef main():\n    parser = argparse.ArgumentParser()\n    parser.add_argument(\"--iterations\", type=int, default=100)\n    args = parser.parse_args()\n    for i in range(args.iterations):\n        register(i, price_for(i, i))\n\n\nif __name__ == \"__main__\":\n    main()\n",
    );
}

struct Out {
    code: i32,
    json: Value,
    text: String,
}

fn dla(project: &Path, args: &[&str]) -> Out {
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
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    o.json
}

fn ledger(run_dir: &Path) -> Vec<Value> {
    fs::read_to_string(run_dir.join("ledger.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn findings_at(lines: &[Value], site: &str) -> Vec<Value> {
    lines
        .iter()
        .filter(|l| l["type"] == "finding_added" && l["finding"]["site"] == site)
        .map(|l| l["finding"].clone())
        .collect()
}

/// The project with a static run (sweep, census, discovery) done; returns the run directory.
fn static_run(dir: &Path) -> std::path::PathBuf {
    project(dir);
    let init = ok(dir, &["run", "init", "--session-id", "t"]);
    let run_dir = dir.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(dir, &["sweep", "--lang", "python"]);
    ok(dir, &["census", "--lang", "python"]);
    ok(dir, &["workload", "discover", "--lang", "python"]);
    run_dir
}

#[test]
fn a_rule_hit_and_the_census_entry_of_one_site_are_one_finding() {
    let d = tempfile::tempdir().unwrap();
    let run_dir = static_run(d.path());
    let lines = ledger(&run_dir);
    let at = findings_at(&lines, "shop/cache.py:3");
    assert_eq!(at.len(), 1, "one finding at the decorator line: {at:?}");
    let f = &at[0];
    assert_eq!(f["origin"], "detector:dla/python-unbounded-cache", "{f}");
    // The census entry is recorded on that finding, with its evidence, not as a second finding.
    let joined: Vec<&Value> = lines
        .iter()
        .filter(|l| {
            l["type"] == "finding_annotated"
                && l["also_origin"] == "census"
                && l["finding"] == f["id"]
        })
        .collect();
    assert_eq!(joined.len(), 1, "{joined:?}");
    // The census entry's holder and owner go to the finding (a rule hit names neither).
    assert_eq!(
        joined[0]["owner"], "shop.cache.price_for",
        "{:?}",
        joined[0]
    );
    assert_eq!(joined[0]["holder"], "static/global");
    assert_eq!(joined[0]["also_supports"].as_array().unwrap().len(), 1);
    let note = joined[0]["note"].as_str().unwrap();
    assert!(note.contains("also raised by census"), "{note}");
    // The report lists it once, with both origins.
    let report = fs::read_to_string(run_dir.join("report.md")).unwrap_or_default();
    if !report.is_empty() {
        assert!(
            !report.contains("census` | static/global | `shop.cache.price_for"),
            "{report}"
        );
    }
    // The registry is another site, and one finding as well.
    assert_eq!(findings_at(&lines, "shop/cache.py:8").len(), 1);
    // No census finding was added where a rule had raised the site.
    assert!(
        lines
            .iter()
            .all(|l| l["type"] != "finding_added" || l["finding"]["origin"] != "census"),
        "every census entry of this project joined a rule finding"
    );
}

#[test]
fn probe_of_the_merged_finding_reads_the_cached_function_as_a_global() {
    let d = tempfile::tempdir().unwrap();
    let run_dir = static_run(d.path());
    let id = findings_at(&ledger(&run_dir), "shop/cache.py:3")[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let out = ok(
        d.path(),
        &[
            "harness", "gen", "--lang", "python", "--probes", "none", "--probe", &id,
        ],
    );
    assert_eq!(out["command"], "harness gen");
    let probes: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("harness/H1/probes.json")).unwrap())
            .unwrap();
    let list = probes["probes"].as_array().unwrap();
    let own = list
        .iter()
        .find(|p| p["target"]["kind"] == "py-global" && p["target"]["name"] == "price_for")
        .unwrap_or_else(|| panic!("a probe of the function itself: {list:?}"));
    assert_eq!(own["finding"], id.as_str());
    assert_eq!(own["target"]["module"], "shop.cache");
}

#[test]
fn probe_of_a_finding_merged_into_another_names_the_one_to_use() {
    let d = tempfile::tempdir().unwrap();
    let run_dir = static_run(d.path());
    let lines = ledger(&run_dir);
    let id = findings_at(&lines, "shop/cache.py:3")[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let ev = findings_at(&lines, "shop/cache.py:3")[0]["supports"][0]
        .as_str()
        .unwrap()
        .to_owned();
    // A second finding of the same owner, as an older ledger holds two for one site.
    let added = ok(
        d.path(),
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            &ev,
            "--site",
            "shop/cache.py:4",
            "--owner",
            "shop.cache.price_for",
        ],
    );
    let twin = added["finding"]
        .as_str()
        .or(added["id"].as_str())
        .unwrap_or("")
        .to_owned();
    assert!(twin.starts_with('F'), "{added}");
    ok(d.path(), &["ledger", "duplicate", &twin, "--of", &id]);
    let o = dla(
        d.path(),
        &["harness", "gen", "--lang", "python", "--probe", &twin],
    );
    assert_ne!(o.code, 0, "{}", o.text);
    assert!(o.text.contains(&format!("--probe {twin}")), "{}", o.text);
    assert!(o.text.contains(&format!("duplicate of {id}")), "{}", o.text);
    assert!(
        o.text
            .contains(&format!("dla harness gen --lang python --probe {id}")),
        "the fix names the finding to use: {}",
        o.text
    );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

#[test]
fn probe_of_a_finding_no_probe_can_read_is_refused_with_the_reason() {
    let d = tempfile::tempdir().unwrap();
    let project = d.path().join("project");
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/fixtures/go-buffer-pool/project"),
        &project,
    );
    ok(&project, &["run", "init", "--session-id", "t"]);
    ok(&project, &["census", "--lang", "go"]);
    ok(&project, &["workload", "discover", "--lang", "go"]);
    let o = dla(
        &project,
        &["harness", "gen", "--lang", "go", "--probe", "F1"],
    );
    assert_ne!(o.code, 0, "the request is not dropped: {}", o.text);
    assert!(o.text.contains("--probe F1"), "{}", o.text);
    assert!(
        o.text
            .contains("go command mode has no package-variable probes"),
        "{}",
        o.text
    );
    assert!(
        o.text
            .contains("dla harness gen --lang go --driver --probe F1"),
        "{}",
        o.text
    );
}
