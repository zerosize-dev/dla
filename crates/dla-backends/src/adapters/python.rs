//! Python: ruff (SARIF) and pylint (`json2`).

use serde_json::{Value, json};

use super::{Job, Outcome, Ran, csv, dedupe, hit, isolate, run_record, version_of};
use crate::{
    exec::{self, Cmd},
    fsx::{self, PathMap},
    sarif,
};

fn no_sources(version: String) -> Outcome {
    Outcome::Ran(Box::new(Ran {
        version,
        note: Some("not applicable: no Python sources".to_owned()),
        raw: json!({ "runs": [] }),
        ..Ran::default()
    }))
}

fn has_python(job: &Job) -> bool {
    !fsx::walk(job.project(), 1, &|r| {
        std::path::Path::new(r)
            .extension()
            .is_some_and(|e| e == "py" || e == "pyi")
    })
    .is_empty()
}

/// Parses ruff's SARIF (file URIs).
pub fn parse_ruff(
    log: &Value,
    wanted: &[&str],
    paths: &PathMap,
) -> Vec<dla_core::static_hit::StaticHit> {
    sarif::results(log)
        .into_iter()
        .filter(|r| wanted.contains(&r.rule.as_str()))
        .filter_map(|r| {
            let f = paths.rel(&r.uri)?;
            Some(hit("ruff", &r.rule, &f, r.line, &r.message))
        })
        .collect()
}

/// Runs ruff with only the listed rules, ignoring any project configuration.
pub fn ruff(job: &Job) -> Outcome {
    let Some(prog) = job.env.find_program("ruff") else {
        return Outcome::missing("ruff is not on PATH or in the tools directory");
    };
    let version = version_of(Cmd::new(&prog).arg("--version"));
    if !has_python(job) {
        return no_sources(version);
    }
    let ids = job.rule_ids();
    let cmd = isolate(Cmd::new(&prog), job)
        .cwd(job.project())
        .env(
            "RUFF_CACHE_DIR",
            job.scratch.join("cache/ruff").into_os_string(),
        )
        .args([
            "check",
            "--isolated",
            "--no-cache",
            "--select",
            &csv(&ids),
            "--extend-exclude",
            ".dla",
            "--output-format",
            "sarif",
            ".",
        ])
        .timeout(job.timeout);
    let out = exec::run(&cmd);
    let rec = run_record(&cmd, &out);
    if let Some(r) = out.failure_reason() {
        return Outcome::blocked(format!("ruff {r}"), vec![rec]);
    }
    let Some(log) = sarif::first_json(&out.stdout) else {
        return Outcome::blocked(
            format!(
                "ruff printed no SARIF (exit {:?}): {}",
                out.exit,
                out.tail(3)
            ),
            vec![rec],
        );
    };
    let hits = dedupe(parse_ruff(&log, &ids, &job.paths()));
    Outcome::Ran(Box::new(Ran {
        version,
        command: cmd.display(),
        exit: out.exit,
        hits,
        raw: json!({ "runs": [rec] }),
        ..Ran::default()
    }))
}

/// Hits and fatal-message count from pylint `json2`.
#[derive(Debug, Default)]
pub struct PylintParsed {
    /// Hits for wanted message ids.
    pub hits: Vec<dla_core::static_hit::StaticHit>,
    /// Messages of type `fatal` (a module could not be parsed).
    pub fatal: u64,
}

/// Parses pylint `--output-format=json2`.
pub fn parse_pylint(log: &Value, wanted: &[&str], paths: &PathMap) -> PylintParsed {
    let mut p = PylintParsed::default();
    for m in log
        .get("messages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if m.get("type").and_then(Value::as_str) == Some("fatal") {
            p.fatal += 1;
            continue;
        }
        let id = m.get("messageId").and_then(Value::as_str).unwrap_or("");
        if !wanted.contains(&id) {
            continue;
        }
        let raw = m
            .get("absolutePath")
            .and_then(Value::as_str)
            .or_else(|| m.get("path").and_then(Value::as_str))
            .unwrap_or("");
        let Some(file) = paths.rel(raw).or_else(|| {
            m.get("path")
                .and_then(Value::as_str)
                .and_then(|r| paths.rel(r))
        }) else {
            continue;
        };
        let line = m
            .get("line")
            .and_then(Value::as_u64)
            .map_or(1, |n| u32::try_from(n).unwrap_or(1));
        let msg = m.get("message").and_then(Value::as_str).unwrap_or("");
        p.hits.push(hit("pylint", id, &file, line, msg));
    }
    p
}

/// Runs pylint with every message disabled except the listed ones.
pub fn pylint(job: &Job) -> Outcome {
    let Some(prog) = job.env.find_program("pylint") else {
        return Outcome::missing("pylint is not on PATH or in the tools directory");
    };
    let version = version_of(Cmd::new(&prog).arg("--version"));
    if !has_python(job) {
        return no_sources(version);
    }
    let ids = job.rule_ids();
    let rc = job.scratch.join("pylintrc");
    if let Err(e) = fsx::write(&rc, "[MASTER]\n") {
        return Outcome::blocked(format!("cannot write the rcfile: {e}"), vec![]);
    }
    let ignore = crate::fsx::SKIP_DIRS.join(",");
    let cmd = isolate(Cmd::new(&prog), job)
        .cwd(job.project())
        .env(
            "PYLINTHOME",
            job.scratch.join("cache/pylint").into_os_string(),
        )
        .arg("--disable=all")
        .arg(format!("--enable={}", csv(&ids)))
        .arg(format!("--rcfile={}", rc.display()))
        .args([
            "--persistent=n",
            "--output-format=json2",
            "--recursive=y",
            "--jobs=1",
        ])
        .arg(format!("--ignore={ignore}"))
        .arg(".")
        .timeout(job.timeout);
    let out = exec::run(&cmd);
    let rec = run_record(&cmd, &out);
    if let Some(r) = out.failure_reason() {
        return Outcome::blocked(format!("pylint {r}"), vec![rec]);
    }
    let Some(log) = sarif::first_json(&out.stdout) else {
        return Outcome::blocked(
            format!(
                "pylint printed no JSON (exit {:?}): {}",
                out.exit,
                out.tail(3)
            ),
            vec![rec],
        );
    };
    let parsed = parse_pylint(&log, &ids, &job.paths());
    let mut limitations = Vec::new();
    if parsed.fatal > 0 {
        limitations.push(format!("{} module(s) could not be parsed", parsed.fatal));
    }
    Outcome::Ran(Box::new(Ran {
        version,
        command: cmd.display(),
        exit: out.exit,
        hits: dedupe(parsed.hits),
        load_errors: parsed.fatal,
        limitations,
        raw: json!({ "runs": [rec] }),
        ..Ran::default()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::Path};

    fn data(name: &str) -> String {
        fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/data")
                .join(name),
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"))
    }

    fn root_of(log: &str) -> String {
        let i = log.find("file://").unwrap() + "file://".len();
        let rest = &log[i..];
        let end = rest.find('"').unwrap();
        rest[..end].rsplit_once('/').unwrap().0.to_owned()
    }

    #[test]
    fn ruff_golden() {
        let text = data("ruff.sarif");
        let root = root_of(&text);
        let log: Value = serde_json::from_str(&text).unwrap();
        let p = PathMap::new(Path::new(&root));
        let hits = parse_ruff(&log, &["B019", "RUF006", "SIM115"], &p);
        let got: Vec<_> = hits
            .iter()
            .map(|h| format!("{}@{}", h.rule, h.site()))
            .collect();
        assert_eq!(
            got,
            [
                "B019@leaks.py:3",
                "B019@leaks.py:5",
                "RUF006@leaks.py:8",
                "SIM115@leaks.py:10"
            ]
        );
        assert_eq!(parse_ruff(&log, &["RUF006"], &p).len(), 1);
        assert!(hits[0].message.contains("lru_cache"));
    }

    #[test]
    fn ruff_fixture_golden_matches_the_known_site() {
        let text = data("ruff-report-memo.sarif");
        let root = root_of(&text);
        let root = root.strip_suffix("/reports").unwrap().to_owned();
        let log: Value = serde_json::from_str(&text).unwrap();
        let hits = parse_ruff(&log, &["B019"], &PathMap::new(Path::new(&root)));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].site(), "reports/builder.py:14");
    }

    #[test]
    fn pylint_golden() {
        let text = data("pylint.json");
        let log: Value = serde_json::from_str(&text).unwrap();
        let abs = log["messages"][0]["absolutePath"].as_str().unwrap();
        let root = abs.rsplit_once('/').unwrap().0.to_owned();
        let p = parse_pylint(&log, &["W1518"], &PathMap::new(Path::new(&root)));
        assert_eq!(p.hits.len(), 2);
        assert_eq!(p.hits[0].rule, "W1518");
        assert_eq!(p.hits[0].site(), "leaks.py:3");
        assert_eq!(p.hits[1].site(), "leaks.py:5");
        assert_eq!(p.fatal, 0);
        assert!(
            parse_pylint(&log, &["W0000"], &PathMap::new(Path::new(&root)))
                .hits
                .is_empty()
        );
    }

    #[test]
    fn pylint_fatal_messages_are_counted_not_hit() {
        let log = json!({"messages":[
            {"type":"fatal","messageId":"E0001","path":"a.py","line":1,"message":"syntax"},
            {"type":"warning","messageId":"W1518","path":"b.py","line":4,"message":"m"}]});
        let p = parse_pylint(&log, &["W1518"], &PathMap::new(Path::new("/proj")));
        assert_eq!(p.fatal, 1);
        assert_eq!(p.hits.len(), 1);
        assert_eq!(p.hits[0].site(), "b.py:4");
    }
}
