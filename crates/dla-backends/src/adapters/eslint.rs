//! Node: eslint with a generated flat config (typescript-eslint parser with type
//! information, `eslint-plugin-rxjs-x`, `@typescript-eslint` rules).

use std::path::{Path, PathBuf};

use dla_core::static_hit::StaticHit;
use serde_json::{Value, json};

use super::{Job, Outcome, Ran, dedupe, hit, isolate, run_record, version_of};
use crate::{
    exec::{self, Cmd},
    fsx::{self, PathMap},
    sarif,
};

const SOURCE_EXTS: [&str; 8] = ["js", "mjs", "cjs", "jsx", "ts", "tsx", "mts", "cts"];

/// The `file://` URL of `p` (forward slashes, drive letters kept).
pub fn file_url(p: &Path) -> String {
    let s = dla_core::paths::to_slash(p);
    if s.starts_with('/') {
        format!("file://{s}")
    } else {
        format!("file:///{s}")
    }
}

/// Which plugin a rule id belongs to (`rxjs-x/no-x` -> `rxjs-x`).
pub fn plugin_of(rule: &str) -> Option<&str> {
    rule.rsplit_once('/').map(|(p, _)| p)
}

/// The flat config enabling exactly `rules`. `modules` is the `node_modules`
/// directory that holds the plugins; `root` the project (the tsconfig root).
pub fn config(
    modules: &Path,
    root: &Path,
    rules: &[&str],
    project_tsconfig: Option<&Path>,
) -> Result<String, String> {
    let url = file_url(modules);
    let mut imports = format!("import tseslint from '{url}/typescript-eslint/dist/index.js';\n");
    let mut plugins = Vec::new();
    for r in rules {
        match plugin_of(r) {
            Some("rxjs-x") => {
                if !imports.contains("rxjs from") {
                    imports.push_str(&format!(
                        "import rxjs from '{url}/eslint-plugin-rxjs-x/dist/index.js';\n"
                    ));
                    plugins.push("'rxjs-x': rxjs");
                }
            }
            Some("@typescript-eslint") => {
                if !plugins.contains(&"'@typescript-eslint': tseslint.plugin") {
                    plugins.push("'@typescript-eslint': tseslint.plugin");
                }
            }
            _ => return Err(format!("unsupported eslint rule {r:?}")),
        }
    }
    let project_option = if let Some(tsconfig) = project_tsconfig {
        // A scratch program that includes every source file, so files outside the
        // project's own `include` are still typed.
        let p = serde_json::to_string(&dla_core::paths::to_slash(tsconfig))
            .map_err(|e| e.to_string())?;
        return config_with(&imports, &plugins, rules, &format!("project: [{p}]"), root);
    } else {
        let mut globs = Vec::new();
        for ext in ["js", "mjs", "cjs", "jsx", "ts", "tsx"] {
            for depth in 0..6 {
                globs.push(format!("'{}*.{ext}'", "*/".repeat(depth)));
            }
        }
        format!(
            "projectService: {{ allowDefaultProject: [{}], maximumDefaultProjectFileMatchCount_THIS_WILL_SLOW_DOWN_LINTING: 5000 }}",
            globs.join(", ")
        )
    };
    config_with(&imports, &plugins, rules, &project_option, root)
}

/// Writes a scratch tsconfig that extends the project's and includes every source
/// file under `root`.
pub fn scratch_tsconfig(project_tsconfig: &Path, root: &Path) -> Result<String, String> {
    let r = dla_core::paths::to_slash(root);
    let r = r.trim_end_matches('/');
    let mut opts = json!({ "allowJs": true, "noEmit": true, "skipLibCheck": true });
    let types = root.join("node_modules/@types");
    if types.is_dir() {
        opts["typeRoots"] = json!([dla_core::paths::to_slash(&types)]);
    }
    let exclude: Vec<String> = [
        "node_modules",
        ".dla",
        ".git",
        "dist",
        "build",
        "out",
        "coverage",
    ]
    .iter()
    .map(|d| format!("{r}/**/{d}"))
    .collect();
    serde_json::to_string_pretty(&json!({
        "extends": dla_core::paths::to_slash(project_tsconfig),
        "compilerOptions": opts,
        "include": [format!("{r}/**/*")],
        "exclude": exclude,
    }))
    .map_err(|e| e.to_string())
}

fn config_with(
    imports: &str,
    plugins: &[&str],
    rules: &[&str],
    project_option: &str,
    root: &Path,
) -> Result<String, String> {
    let rule_lines: Vec<String> = rules.iter().map(|r| format!("'{r}': 'error'")).collect();
    let root_json =
        serde_json::to_string(&dla_core::paths::to_slash(root)).map_err(|e| e.to_string())?;
    Ok(format!(
        "{imports}export default [\n  {{ ignores: ['**/node_modules/**', '**/.dla/**', '**/.git/**', '**/dist/**', '**/build/**', '**/out/**', '**/coverage/**'] }},\n  {{\n    files: ['**/*.{{{exts}}}'],\n    languageOptions: {{ parser: tseslint.parser, parserOptions: {{ {project_option}, tsconfigRootDir: {root_json} }} }},\n    plugins: {{ {plugins} }},\n    linterOptions: {{ reportUnusedDisableDirectives: 'off' }},\n    rules: {{ {rules} }},\n  }},\n];\n",
        exts = SOURCE_EXTS.join(","),
        plugins = plugins.join(", "),
        rules = rule_lines.join(", "),
    ))
}

/// Hits and fatal-message count parsed from `eslint -f json`.
#[derive(Debug, Default)]
pub struct Parsed {
    /// Hits for wanted rules.
    pub hits: Vec<StaticHit>,
    /// Fatal (parse) errors.
    pub fatal: u64,
    /// The first fatal message.
    pub first_fatal: Option<String>,
}

/// Parses eslint's JSON formatter output.
pub fn parse(log: &Value, wanted: &[&str], paths: &PathMap) -> Parsed {
    let mut p = Parsed::default();
    for f in log.as_array().into_iter().flatten() {
        let file = f.get("filePath").and_then(Value::as_str).unwrap_or("");
        for m in f
            .get("messages")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let msg = m.get("message").and_then(Value::as_str).unwrap_or("");
            match m.get("ruleId").and_then(Value::as_str) {
                Some(rule) if wanted.contains(&rule) => {
                    let line = m
                        .get("line")
                        .and_then(Value::as_u64)
                        .map_or(1, |n| u32::try_from(n).unwrap_or(1));
                    if let Some(rel) = paths.rel(file) {
                        p.hits.push(hit("eslint", rule, &rel, line, msg));
                    }
                }
                Some(_) => {}
                None => {
                    if m.get("fatal").and_then(Value::as_bool) == Some(true) {
                        p.fatal += 1;
                        p.first_fatal
                            .get_or_insert_with(|| msg.chars().take(200).collect::<String>());
                    }
                }
            }
        }
    }
    p
}

/// The `node_modules` directory that holds the eslint at `prog` (global or local
/// install), when it also has `typescript-eslint`.
fn modules_dir(prog: &Path) -> Option<PathBuf> {
    let real = prog.canonicalize().ok()?;
    real.ancestors()
        .filter(|a| a.file_name().is_some_and(|n| n == "node_modules"))
        .find(|d| d.join("typescript-eslint/dist/index.js").is_file())
        .map(Path::to_path_buf)
}

/// Runs eslint with the generated config.
pub fn run(job: &Job) -> Outcome {
    let Some(prog) = job.env.find_program("eslint") else {
        return Outcome::missing("eslint is not on PATH or in the tools directory");
    };
    let version = version_of(Cmd::new(&prog).arg("--version"));
    if fsx::files_with_ext(job.project(), &SOURCE_EXTS).is_empty() {
        return Outcome::Ran(Box::new(Ran {
            version,
            note: Some("not applicable: no JavaScript or TypeScript sources".to_owned()),
            raw: json!({ "runs": [] }),
            ..Ran::default()
        }));
    }
    let Some(modules) = modules_dir(&prog) else {
        return Outcome::blocked(
            "typescript-eslint is not installed next to eslint (run dla install eslint)",
            vec![],
        );
    };
    let ids = job.rule_ids();
    if ids.iter().any(|r| plugin_of(r) == Some("rxjs-x"))
        && !modules.join("eslint-plugin-rxjs-x/dist/index.js").is_file()
    {
        return Outcome::blocked(
            "eslint-plugin-rxjs-x is not installed next to eslint (run dla install eslint)",
            vec![],
        );
    }
    let project_ts = job.project().join("tsconfig.json");
    let scratch_ts = job.scratch.join("tsconfig.dla.json");
    let ts = if project_ts.is_file() {
        match scratch_tsconfig(&project_ts, job.project())
            .map_err(|e| e.to_string())
            .and_then(|t| fsx::write(&scratch_ts, &t).map_err(|e| e.to_string()))
        {
            Ok(()) => Some(scratch_ts.as_path()),
            Err(e) => return Outcome::blocked(format!("cannot write the tsconfig: {e}"), vec![]),
        }
    } else {
        None
    };
    let cfg_text = match config(&modules, job.project(), &ids, ts) {
        Ok(t) => t,
        Err(e) => return Outcome::blocked(e, vec![]),
    };
    let cfg = job.scratch.join("eslint.config.mjs");
    if let Err(e) = fsx::write(&cfg, &cfg_text) {
        return Outcome::blocked(format!("cannot write the config: {e}"), vec![]);
    }
    let cmd = isolate(Cmd::new(&prog), job)
        .cwd(job.project())
        .arg("--no-config-lookup")
        .arg("-c")
        .arg(&cfg)
        .args(["-f", "json", "--no-warn-ignored", "."])
        .timeout(job.timeout);
    let out = exec::run(&cmd);
    let rec = run_record(&cmd, &out);
    if let Some(r) = out.failure_reason() {
        return Outcome::blocked(format!("eslint {r}"), vec![rec]);
    }
    let Some(log) = sarif::first_json(&out.stdout).filter(Value::is_array) else {
        return Outcome::blocked(
            format!(
                "eslint printed no JSON (exit {:?}): {}",
                out.exit,
                out.tail(3)
            ),
            vec![rec],
        );
    };
    let parsed = parse(&log, &ids, &job.paths());
    let mut limitations = vec![
        "rxjs-x rules need rxjs to resolve from the project's node_modules for type information"
            .to_owned(),
    ];
    if !ids.iter().any(|r| plugin_of(r) == Some("rxjs-x")) {
        limitations.clear();
    }
    if parsed.fatal > 0 {
        limitations.push(format!(
            "{} file(s) failed to parse; first: {}",
            parsed.fatal,
            parsed.first_fatal.as_deref().unwrap_or("")
        ));
    }
    Outcome::Ran(Box::new(Ran {
        version,
        command: cmd.display(),
        exit: out.exit,
        hits: dedupe(parsed.hits),
        load_errors: parsed.fatal,
        limitations,
        note: None,
        raw: json!({ "runs": [rec] }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn config_imports_only_what_is_used() {
        let c = config(
            Path::new("/opt/nm"),
            Path::new("/proj"),
            &[
                "rxjs-x/no-ignored-subscription",
                "@typescript-eslint/no-floating-promises",
            ],
            None,
        )
        .unwrap();
        assert!(c.contains("import rxjs from 'file:///opt/nm/eslint-plugin-rxjs-x/dist/index.js'"));
        assert!(c.contains("'rxjs-x/no-ignored-subscription': 'error'"));
        assert!(c.contains("'@typescript-eslint': tseslint.plugin"));
        assert!(c.contains("allowDefaultProject"));
        assert!(c.contains("tsconfigRootDir: \"/proj\""));
        let c = config(
            Path::new("/opt/nm"),
            Path::new("/proj"),
            &["@typescript-eslint/no-floating-promises"],
            Some(Path::new("/scratch/tsconfig.dla.json")),
        )
        .unwrap();
        assert!(!c.contains("rxjs"));
        assert!(c.contains("project: [\"/scratch/tsconfig.dla.json\"]"));
        assert!(!c.contains("projectService"));
        assert!(config(Path::new("/x"), Path::new("/p"), &["unknown/rule"], None).is_err());
    }

    #[test]
    fn scratch_tsconfig_covers_every_source_file() {
        let d = tempfile::tempdir().unwrap();
        let t = scratch_tsconfig(&d.path().join("tsconfig.json"), d.path()).unwrap();
        let v: Value = serde_json::from_str(&t).unwrap();
        let root = dla_core::paths::to_slash(d.path());
        assert_eq!(v["include"][0], format!("{root}/**/*"));
        assert_eq!(v["compilerOptions"]["allowJs"], true);
        assert!(v["extends"].as_str().unwrap().ends_with("tsconfig.json"));
    }

    #[test]
    fn file_urls() {
        assert_eq!(file_url(Path::new("/a/b")), "file:///a/b");
        assert_eq!(file_url(Path::new("C:\\a\\b")), "file:///C:/a/b");
        assert_eq!(
            plugin_of("@typescript-eslint/no-floating-promises"),
            Some("@typescript-eslint")
        );
        assert_eq!(plugin_of("nope"), None);
    }

    fn data(name: &str) -> String {
        fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/data")
                .join(name),
        )
        .unwrap()
    }

    #[test]
    fn eslint_golden() {
        let text = data("eslint.json");
        let log: Value = serde_json::from_str(&text).unwrap();
        let file = log[0]["filePath"].as_str().unwrap();
        let root = file.rsplit_once('/').unwrap().0.to_owned();
        let ids = [
            "rxjs-x/no-ignored-subscription",
            "rxjs-x/no-nested-subscribe",
            "@typescript-eslint/no-floating-promises",
        ];
        let p = parse(&log, &ids, &PathMap::new(Path::new(&root)));
        let got: Vec<_> = p
            .hits
            .iter()
            .map(|h| format!("{}@{}", h.rule, h.site()))
            .collect();
        assert!(
            got.contains(&"rxjs-x/no-nested-subscribe@a.ts:6".to_owned()),
            "{got:?}"
        );
        assert!(
            got.contains(&"@typescript-eslint/no-floating-promises@a.ts:8".to_owned()),
            "{got:?}"
        );
        assert_eq!(p.fatal, 0);
    }

    #[test]
    fn fatal_parse_errors_are_counted() {
        let log = json!([{"filePath":"/proj/a.ts","messages":[
            {"ruleId":null,"fatal":true,"severity":2,"message":"Parsing error: x"},
            {"ruleId":"rxjs-x/no-nested-subscribe","line":3,"message":"m"}]}]);
        let p = parse(
            &log,
            &["rxjs-x/no-nested-subscribe"],
            &PathMap::new(Path::new("/proj")),
        );
        assert_eq!(p.fatal, 1);
        assert_eq!(p.hits.len(), 1);
        assert_eq!(p.hits[0].site(), "a.ts:3");
    }
}
