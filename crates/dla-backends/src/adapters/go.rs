//! Go: staticcheck (SARIF), golangci-lint (SARIF file) and `go vet -json`.

use std::fs;

use dla_core::static_hit::StaticHit;
use serde_json::Value;

use super::{Job, Outcome, Ran, attach_file, csv, dedupe, hit, isolate, run_record, version_of};
use crate::{
    exec::{self, Cmd},
    fsx::{self, PathMap},
    sarif,
};

/// Rule ids tools use for "the package did not load".
const LOAD_ERROR_RULES: [&str; 4] = ["compile", "typecheck", "parse", "load"];

/// Hits and errors parsed from one output.
#[derive(Debug, Default, PartialEq)]
pub struct Parsed {
    /// Hits for wanted rules.
    pub hits: Vec<StaticHit>,
    /// Load or compile errors.
    pub load_errors: u64,
    /// The first such error message.
    pub first_error: Option<String>,
}

/// Directories (absolute) of every Go module under the project: each `go.mod` that
/// is not under a skipped directory (or `vendor`/`testdata`). `./...` stops at nested
/// modules, so each one is run from its own directory.
pub fn module_dirs(project: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mods = fsx::walk_with(
        project,
        10_000,
        &|n| fsx::skipped(n) || n == "vendor" || n == "testdata",
        &|r| r == "go.mod" || r.ends_with("/go.mod"),
    );
    let mut dirs: Vec<_> = mods
        .iter()
        .map(|m| match m.rsplit_once('/') {
            Some((d, _)) => project.join(d),
            None => project.to_path_buf(),
        })
        .collect();
    dirs.truncate(MAX_MODULES);
    dirs
}

/// The most modules a run covers.
const MAX_MODULES: usize = 16;

fn go_cmd(program: &std::path::Path, job: &Job, dir: &std::path::Path) -> Result<Cmd, String> {
    let tmp = job
        .ctx
        .tmp_dir()
        .map_err(|e| format!("cannot create the scratch directory: {}", e.message))?;
    let cmd = Cmd::new(program)
        .cwd(dir)
        .env("GOCACHE", tmp.join("gocache").into_os_string())
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local")
        .env("GOFLAGS", "-buildvcs=false -modcacherw")
        .timeout(job.timeout);
    Ok(isolate(cmd, job))
}

/// The result of one tool run in one module.
struct Single {
    parsed: Parsed,
    cmd: Cmd,
    exit: Option<i32>,
    rec: Value,
}

/// A tool run in one module: the result, or the failure reason and run records.
type ModuleRun = Result<Single, (String, Vec<Value>)>;

/// Runs `one` in every module and merges the results: blocked only when it fails in
/// every module, otherwise the failures become limitations.
fn per_module(
    backend: &str,
    version: String,
    job: &Job,
    one: &dyn Fn(&std::path::Path) -> ModuleRun,
) -> Outcome {
    let dirs: Vec<_> = module_dirs(job.project())
        .into_iter()
        .filter(|d| {
            let rel = d
                .strip_prefix(job.project())
                .ok()
                .map(dla_core::paths::to_slash)
                .unwrap_or_default();
            job.may_contain(&rel)
        })
        .collect();
    if dirs.is_empty() {
        return Outcome::blocked(
            "no go.mod under the project: go tools cannot load packages",
            vec![],
        );
    }
    let mut parsed = Parsed::default();
    let mut recs = Vec::new();
    let mut failures = Vec::new();
    let mut first: Option<(Cmd, Option<i32>)> = None;
    for d in &dirs {
        let shown = d
            .strip_prefix(job.project())
            .ok()
            .map(dla_core::paths::to_slash)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| ".".to_owned());
        match one(d) {
            Ok(s) => {
                recs.push(s.rec);
                parsed.hits.extend(s.parsed.hits);
                parsed.load_errors += s.parsed.load_errors;
                if parsed.first_error.is_none() {
                    parsed.first_error = s.parsed.first_error;
                }
                first.get_or_insert((s.cmd, s.exit));
            }
            Err((why, r)) => {
                recs.extend(r);
                failures.push(format!("{shown}: {why}"));
            }
        }
    }
    let Some((cmd, exit)) = first else {
        return Outcome::blocked(failures.join("; "), recs);
    };
    let mut out = finish(backend, version, &cmd, exit, parsed, recs);
    if let Outcome::Ran(r) = &mut out {
        r.limitations.extend(
            failures
                .into_iter()
                .map(|f| format!("{backend} failed in module {f}")),
        );
    }
    out
}

/// Parses staticcheck's SARIF.
pub fn parse_staticcheck(log: &Value, wanted: &[&str], paths: &PathMap) -> Parsed {
    let mut p = Parsed::default();
    for r in sarif::results(log) {
        if LOAD_ERROR_RULES.contains(&r.rule.as_str()) {
            p.load_errors += 1;
            p.first_error.get_or_insert(r.message.clone());
            continue;
        }
        if !wanted.contains(&r.rule.as_str()) {
            continue;
        }
        if let Some(f) = paths.rel(&r.uri) {
            p.hits
                .push(hit("staticcheck", &r.rule, &f, r.line, &r.message));
        }
    }
    p
}

/// Runs staticcheck in every module.
pub fn staticcheck(job: &Job) -> Outcome {
    let Some(prog) = job.env.find_program("staticcheck") else {
        return Outcome::missing("staticcheck is not on PATH or in the tools directory");
    };
    let version = version_of(Cmd::new(&prog).arg("-version"));
    let ids = job.rule_ids();
    per_module("staticcheck", version, job, &|dir| {
        let cmd = go_cmd(&prog, job, dir).map_err(|e| (e, vec![]))?;
        let cmd = cmd.args(["-checks", &csv(&ids), "-f", "sarif", "./..."]);
        let out = exec::run(&cmd);
        let rec = run_record(&cmd, &out);
        if let Some(r) = out.failure_reason() {
            return Err((format!("staticcheck {r}"), vec![rec]));
        }
        let Some(log) = sarif::first_json(&out.stdout) else {
            return Err((
                format!(
                    "staticcheck printed no SARIF (exit {:?}): {}",
                    out.exit,
                    out.tail(3)
                ),
                vec![rec],
            ));
        };
        let parsed = parse_staticcheck(&log, &ids, &job.paths().with_base(dir));
        Ok(Single {
            parsed,
            cmd,
            exit: out.exit,
            rec,
        })
    })
}

fn finish(
    backend: &str,
    version: String,
    cmd: &Cmd,
    exit: Option<i32>,
    parsed: Parsed,
    recs: Vec<Value>,
) -> Outcome {
    if parsed.hits.is_empty()
        && parsed.load_errors > 0
        && let Some(e) = &parsed.first_error
    {
        return Outcome::blocked(format!("{backend} could not load the packages: {e}"), recs);
    }
    let mut limitations = Vec::new();
    if parsed.load_errors > 0 {
        limitations.push(format!(
            "{} package load error(s); first: {}",
            parsed.load_errors,
            parsed.first_error.as_deref().unwrap_or("")
        ));
    }
    Outcome::Ran(Box::new(Ran {
        version,
        command: cmd.display(),
        exit,
        hits: dedupe(parsed.hits),
        load_errors: parsed.load_errors,
        limitations,
        note: None,
        raw: serde_json::json!({ "runs": recs }),
    }))
}

/// The golangci-lint v2 configuration enabling exactly the listed rules.
pub fn golangci_config(rules: &[&str]) -> String {
    let mut linters: Vec<&str> = Vec::new();
    let mut gocritic: Vec<&str> = Vec::new();
    for r in rules {
        if let Some(check) = r.strip_prefix("gocritic/") {
            gocritic.push(check);
            if !linters.contains(&"gocritic") {
                linters.push("gocritic");
            }
        } else if !linters.contains(r) {
            linters.push(r);
        }
    }
    // golangci-lint defaults to one issue per line, 50 per linter and 3 identical
    // messages; every finding of the enabled linters must be reported.
    let mut y = format!(
        "version: \"2\"\nlinters:\n  default: none\n  enable: [{}]\n",
        linters.join(", ")
    );
    if !gocritic.is_empty() {
        y.push_str(&format!(
            "  settings:\n    gocritic:\n      disable-all: true\n      enabled-checks: [{}]\n",
            gocritic.join(", ")
        ));
    }
    y.push_str(
        "issues:\n  uniq-by-line: false\n  max-issues-per-linter: 0\n  max-same-issues: 0\n",
    );
    y
}

/// Parses golangci-lint's SARIF. The linter name is the rule, except for gocritic
/// where the checker name leads the message (`deferInLoop: ...`).
pub fn parse_golangci(log: &Value, wanted: &[&str], paths: &PathMap) -> Parsed {
    let mut p = Parsed::default();
    for r in sarif::results(log) {
        if LOAD_ERROR_RULES.contains(&r.rule.as_str()) {
            p.load_errors += 1;
            p.first_error.get_or_insert(r.message.clone());
            continue;
        }
        let (rule, message) = if r.rule == "gocritic" {
            match r.message.split_once(':') {
                Some((check, rest)) if !check.contains(' ') => {
                    (format!("gocritic/{check}"), rest.trim().to_owned())
                }
                _ => (r.rule.clone(), r.message.clone()),
            }
        } else {
            (r.rule.clone(), r.message.clone())
        };
        if !wanted.contains(&rule.as_str()) {
            continue;
        }
        if let Some(f) = paths.rel(&r.uri) {
            p.hits
                .push(hit("golangci-lint", &rule, &f, r.line, &message));
        }
    }
    p
}

/// Runs golangci-lint with an explicit linter list in every module.
pub fn golangci(job: &Job) -> Outcome {
    let Some(prog) = job.env.find_program("golangci-lint") else {
        return Outcome::missing("golangci-lint is not on PATH or in the tools directory");
    };
    let version = version_of(Cmd::new(&prog).arg("--version"));
    let ids = job.rule_ids();
    let cfg = job.scratch.join("golangci.yml");
    if let Err(e) = crate::fsx::write(&cfg, &golangci_config(&ids)) {
        return Outcome::blocked(format!("cannot write the config: {e}"), vec![]);
    }
    let n = std::cell::Cell::new(0_usize);
    per_module("golangci-lint", version, job, &|dir| {
        let i = n.get();
        n.set(i + 1);
        let sarif_path = job.scratch.join(format!("golangci-{i}.sarif"));
        let text_path = job.scratch.join(format!("golangci-{i}.txt"));
        let _ = fs::remove_file(&sarif_path);
        let cmd = go_cmd(&prog, job, dir).map_err(|e| (e, vec![]))?;
        let cmd = cmd
            .env(
                "GOLANGCI_LINT_CACHE",
                job.scratch.join("cache/golangci").into_os_string(),
            )
            .arg("run")
            .arg("--config")
            .arg(&cfg)
            .arg("--path-mode=abs")
            .arg("--show-stats=false")
            .arg(format!("--output.sarif.path={}", sarif_path.display()))
            .arg(format!("--output.text.path={}", text_path.display()))
            .arg("./...");
        let out = exec::run(&cmd);
        let mut rec = run_record(&cmd, &out);
        if let Some(r) = out.failure_reason() {
            return Err((format!("golangci-lint {r}"), vec![rec]));
        }
        let Ok(text) = fs::read_to_string(&sarif_path) else {
            return Err((
                format!(
                    "golangci-lint wrote no SARIF (exit {:?}): {}",
                    out.exit,
                    out.tail(3)
                ),
                vec![rec],
            ));
        };
        attach_file(&mut rec, "output_file_json", &text);
        let Ok(log) = serde_json::from_str::<Value>(&text) else {
            return Err(("golangci-lint wrote malformed SARIF".to_owned(), vec![rec]));
        };
        let parsed = parse_golangci(&log, &ids, &job.paths().with_base(dir));
        Ok(Single {
            parsed,
            cmd,
            exit: out.exit,
            rec,
        })
    })
}

/// Parses `go vet -json` output (a `# package` header, then one indented JSON object
/// per package, possibly on stderr).
pub fn parse_govet(text: &str, wanted: &[&str], paths: &PathMap) -> Parsed {
    let mut p = Parsed::default();
    let mut buf = String::new();
    let mut in_obj = false;
    for line in text.lines() {
        if !in_obj {
            if line == "{" {
                in_obj = true;
                buf.clear();
                buf.push_str(line);
                buf.push('\n');
            } else if !line.trim().is_empty() && !line.starts_with('#') && line.trim() != "{}" {
                p.load_errors += 1;
                p.first_error.get_or_insert(line.trim().to_owned());
            }
            continue;
        }
        buf.push_str(line);
        buf.push('\n');
        if line == "}" {
            in_obj = false;
            let Ok(v) = serde_json::from_str::<Value>(&buf) else {
                p.load_errors += 1;
                continue;
            };
            collect_vet(&v, wanted, paths, &mut p);
        }
    }
    p
}

fn collect_vet(v: &Value, wanted: &[&str], paths: &PathMap, p: &mut Parsed) {
    let Some(pkgs) = v.as_object() else { return };
    for analyzers in pkgs.values() {
        let Some(an) = analyzers.as_object() else {
            continue;
        };
        for (name, diags) in an {
            if !wanted.contains(&name.as_str()) {
                continue;
            }
            for d in diags.as_array().into_iter().flatten() {
                let posn = d.get("posn").and_then(Value::as_str).unwrap_or("");
                let msg = d.get("message").and_then(Value::as_str).unwrap_or("");
                if let Some((file, line)) = split_posn(posn)
                    && let Some(f) = paths.rel(file)
                {
                    p.hits.push(hit("govet", name, &f, line, msg));
                }
            }
        }
    }
}

/// Splits `file:line:col` (the file may contain a drive colon).
fn split_posn(posn: &str) -> Option<(&str, u32)> {
    let (rest, _col) = posn.rsplit_once(':')?;
    let (file, line) = rest.rsplit_once(':')?;
    Some((file, line.parse().ok()?))
}

/// Runs `go vet -json` with only the listed analyzers in every module.
pub fn govet(job: &Job) -> Outcome {
    let Some(prog) = job.env.find_program("go") else {
        return Outcome::missing("the go toolchain is not on PATH");
    };
    let version = version_of(Cmd::new(&prog).arg("version"));
    let ids = job.rule_ids();
    per_module("govet", version, job, &|dir| {
        let cmd = go_cmd(&prog, job, dir).map_err(|e| (e, vec![]))?;
        let mut cmd = cmd.args(["vet", "-json"]);
        for id in &ids {
            cmd = cmd.arg(format!("-{id}"));
        }
        let cmd = cmd.arg("./...");
        let out = exec::run(&cmd);
        let rec = run_record(&cmd, &out);
        if let Some(r) = out.failure_reason() {
            return Err((format!("go vet {r}"), vec![rec]));
        }
        let combined = format!("{}\n{}", out.stdout, out.stderr);
        let parsed = parse_govet(&combined, &ids, &job.paths().with_base(dir));
        Ok(Single {
            parsed,
            cmd,
            exit: out.exit,
            rec,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn data(name: &str) -> String {
        fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/data")
                .join(name),
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"))
    }

    fn pm() -> PathMap {
        PathMap::new(Path::new("/proj"))
    }

    #[test]
    fn staticcheck_golden() {
        let log: Value = serde_json::from_str(&data("staticcheck.sarif")).unwrap();
        let ids = [
            "SA1015", "SA2001", "SA2003", "SA5003", "SA5005", "SA6002", "SA9001",
        ];
        let p = parse_staticcheck(&log, &ids, &pm());
        let got: Vec<_> = p
            .hits
            .iter()
            .map(|h| format!("{}@{}", h.rule, h.site()))
            .collect();
        assert_eq!(
            got,
            [
                "SA5003@p.go:51",
                "SA2001@p.go:62",
                "SA2003@p.go:64",
                "SA6002@q.go:14",
                "SA9001@q.go:25"
            ]
        );
        assert_eq!(p.load_errors, 0);
        // A rule that is not listed is dropped.
        let p = parse_staticcheck(&log, &["SA9001"], &pm());
        assert_eq!(p.hits.len(), 1);
    }

    #[test]
    fn staticcheck_load_errors_are_not_hits() {
        let log: Value = serde_json::from_str(&data("staticcheck-compile-error.sarif")).unwrap();
        let p = parse_staticcheck(&log, &["SA5003"], &pm());
        assert!(p.hits.is_empty());
        assert_eq!(p.load_errors, 1);
        assert!(p.first_error.unwrap().contains("no required module"));
    }

    #[test]
    fn golangci_golden_maps_gocritic_checks() {
        let log: Value = serde_json::from_str(&data("golangci.sarif")).unwrap();
        let ids = [
            "bodyclose",
            "noctx",
            "fatcontext",
            "containedctx",
            "gocritic/deferInLoop",
        ];
        let p = parse_golangci(&log, &ids, &PathMap::new(Path::new("/p")));
        assert!(p.hits.is_empty(), "paths outside the project are dropped");
        let root = log["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
            ["artifactLocation"]["uri"]
            .as_str()
            .unwrap()
            .rsplit_once('/')
            .unwrap()
            .0
            .to_owned();
        let p = parse_golangci(&log, &ids, &PathMap::new(Path::new(&root)));
        let got: Vec<_> = p
            .hits
            .iter()
            .map(|h| format!("{}@{}", h.rule, h.site()))
            .collect();
        assert_eq!(
            got,
            [
                "bodyclose@p.go:18",
                "containedctx@p.go:13",
                "fatcontext@p.go:41",
                "gocritic/deferInLoop@p.go:48",
                "gocritic/deferInLoop@p.go:51",
                "gocritic/deferInLoop@q.go:25",
                "noctx@p.go:23",
                "noctx@q.go:30"
            ]
        );
        assert_eq!(
            p.hits[3].message,
            "Possible resource leak, 'defer' is called in the 'for' loop"
        );
    }

    #[test]
    fn golangci_config_lists_exactly_the_rules() {
        let y = golangci_config(&["bodyclose", "gocritic/deferInLoop", "noctx"]);
        assert!(y.contains("enable: [bodyclose, gocritic, noctx]"));
        assert!(y.contains("enabled-checks: [deferInLoop]"));
        assert!(y.contains("default: none"));
        assert!(!golangci_config(&["noctx"]).contains("gocritic"));
    }

    #[test]
    fn golangci_config_disables_dedupe_and_caps() {
        let y = golangci_config(&["noctx", "sqlclosecheck", "rowserrcheck"]);
        assert!(y.contains("\nissues:\n  uniq-by-line: false\n"));
        let g = golangci_config(&["gocritic/deferInLoop"]);
        let (lint, issues) = g.split_once("\nissues:").unwrap();
        assert!(lint.contains("settings:"), "settings stay under linters");
        assert!(!issues.contains("settings:"));
        assert!(y.contains("max-issues-per-linter: 0"));
        assert!(y.contains("max-same-issues: 0"));
    }

    #[test]
    fn module_dirs_finds_nested_modules_and_skips_vendor() {
        let d = tempfile::tempdir().unwrap();
        for p in [
            "svc/go.mod",
            "tools/x/go.mod",
            "svc/vendor/dep/go.mod",
            "node_modules/n/go.mod",
        ] {
            let f = d.path().join(p);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, "module m\n").unwrap();
        }
        let got: Vec<_> = module_dirs(d.path())
            .iter()
            .map(|p| p.strip_prefix(d.path()).unwrap().to_owned())
            .collect();
        assert_eq!(got, [Path::new("svc"), Path::new("tools/x")]);
        fs::write(d.path().join("go.mod"), "module r\n").unwrap();
        assert_eq!(module_dirs(d.path()).len(), 3);
        let empty = tempfile::tempdir().unwrap();
        assert!(module_dirs(empty.path()).is_empty());
    }

    #[test]
    fn nested_module_paths_are_project_relative() {
        let pm = PathMap::new(Path::new("/proj")).with_base(Path::new("/proj/svc"));
        assert_eq!(pm.rel("a.go").as_deref(), Some("svc/a.go"));
        assert_eq!(pm.rel("./pkg/b.go").as_deref(), Some("svc/pkg/b.go"));
        assert_eq!(pm.rel("/proj/svc/c.go").as_deref(), Some("svc/c.go"));
    }

    #[test]
    fn govet_golden() {
        let text = data("govet.txt");
        let root = text
            .lines()
            .find_map(|l| l.split_once("\"posn\": \"").map(|(_, r)| r.to_owned()))
            .unwrap();
        let root = root.rsplit_once('/').unwrap().0.to_owned();
        let p = parse_govet(&text, &["lostcancel"], &PathMap::new(Path::new(&root)));
        assert_eq!(p.hits.len(), 1, "{p:?}");
        assert_eq!(p.hits[0].site(), "p.go:41");
        assert_eq!(p.hits[0].rule, "lostcancel");
        assert!(p.hits[0].message.contains("cancel function"));
        assert_eq!(p.load_errors, 0);
    }

    #[test]
    fn govet_empty_packages_are_not_errors() {
        let p = parse_govet("# a\n{}\n# [a]\n{}\n", &["lostcancel"], &pm());
        assert_eq!(p, Parsed::default());
    }

    #[test]
    fn govet_text_lines_count_as_load_errors() {
        let p = parse_govet(
            "# example.com/p\nvet: p.go:3:1: undefined: x\n",
            &["lostcancel"],
            &pm(),
        );
        assert_eq!(p.load_errors, 1);
        assert!(p.first_error.unwrap().contains("undefined"));
    }

    #[test]
    fn posn_split_handles_drive_letters() {
        assert_eq!(split_posn("C:/a/b.go:12:5"), Some(("C:/a/b.go", 12)));
        assert_eq!(split_posn("nonsense"), None);
    }
}
