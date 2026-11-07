//! `dla-bench static`: the M2 acceptance harness.
//!
//! For every fixture the project is copied to a scratch directory, a run is
//! created with the runner's consent and the static stages of `dla` are run
//! (`probe --build`, `sweep` and `census` per detected runtime, `workload
//! discover`). The candidates in the ledger are then graded with the site
//! matcher of the grader ([`crate::grade::site_matches`]).

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Args;
use serde::Serialize;
use serde_json::Value;

use crate::{
    Res,
    fixture::{fixture_dirs, load_expected},
    grade::{Finding, State, leak_outcomes, site_matches},
    run::{Archetypes, apply_archetypes_off, copy_tree},
};

/// Arguments of `dla-bench static`.
#[derive(Args, Debug, Clone)]
pub struct StaticArgs {
    /// Directory with the fixtures.
    #[arg(long, default_value = "bench/fixtures")]
    pub fixtures: PathBuf,
    /// Whether archetype data (archetype tables, known leaks, suggested archetypes) is present.
    #[arg(long, value_enum, default_value = "on")]
    pub archetypes: Archetypes,
    /// The dla binary (default: next to dla-bench, else target/debug/dla, else dla on PATH).
    #[arg(long)]
    pub dla_bin: Option<PathBuf>,
    /// Write the full result as JSON to this file.
    #[arg(long)]
    pub json: Option<PathBuf>,
    /// Repository root holding data/ (the plugin data files).
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Only fixtures whose id contains this text.
    #[arg(long)]
    pub only: Option<String>,
}

/// One expected leak, graded statically.
#[derive(Serialize, Debug, Clone)]
pub struct LeakRow {
    /// Leak id.
    pub id: String,
    /// Expected site.
    pub site: String,
    /// Whether dynamic confirmation is possible.
    pub confirmable: bool,
    /// Some candidate names the site.
    pub flagged: bool,
    /// Origins of the candidates that name it.
    pub origins: Vec<String>,
}

/// Discovered workloads of one fixture.
#[derive(Serialize, Debug, Clone, Default)]
pub struct WorkloadRow {
    /// Test workloads.
    pub test: usize,
    /// Main workloads.
    pub main: usize,
    /// Script workloads.
    pub script: usize,
    /// Dev-server workloads.
    pub dev_server: usize,
}

/// The result for one fixture.
#[derive(Serialize, Debug, Clone, Default)]
pub struct FixtureRow {
    /// Fixture id.
    pub id: String,
    /// Runtimes the probe detected.
    pub runtimes: Vec<String>,
    /// Graded leaks.
    pub leaks: Vec<LeakRow>,
    /// Candidates in the ledger.
    pub candidates: usize,
    /// Candidates per origin.
    pub by_origin: BTreeMap<String, usize>,
    /// Sum of the `enumerated` counts in coverage.json.
    pub enumerated: u64,
    /// `enumerated` per `worker/backend`.
    pub enumerated_by_backend: BTreeMap<String, u64>,
    /// Workloads.
    pub workloads: WorkloadRow,
    /// Backends marked missing, as `worker/backend`.
    pub backends_missing: Vec<String>,
    /// Backends marked blocked, as `worker/backend`.
    pub backends_blocked: Vec<String>,
    /// Stage failures.
    pub errors: Vec<String>,
    /// Reasons this fixture fails the acceptance.
    pub failures: Vec<String>,
}

/// The whole result.
#[derive(Serialize, Debug, Clone)]
pub struct StaticResult {
    /// `on` or `off`.
    pub archetypes: String,
    /// Per fixture.
    pub fixtures: Vec<FixtureRow>,
    /// Failures over all fixtures.
    pub failures: Vec<String>,
}

fn skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".dla"
            | "target"
            | "build"
            | "node_modules"
            | "venv"
            | ".venv"
            | "out"
            | "dist"
            | "__pycache__"
    )
}

fn walk_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Res<()> {
    let rd = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if !skip_dir(&name) {
                walk_files(root, &p, out)?;
            }
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_path_buf());
        }
    }
    Ok(())
}

/// The tracked files of a fixture project (git index when the fixtures are in
/// a checkout, else a walk that skips build output).
fn project_files(project: &Path) -> Res<Vec<PathBuf>> {
    let tracked = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["ls-files", "-z", "--", "."])
        .output();
    if let Ok(o) = tracked
        && o.status.success()
        && !o.stdout.is_empty()
    {
        return Ok(o
            .stdout
            .split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| PathBuf::from(String::from_utf8_lossy(s).into_owned()))
            .collect());
    }
    let mut v = Vec::new();
    walk_files(project, project, &mut v)?;
    v.sort();
    Ok(v)
}

fn copy_project(project: &Path, dest: &Path) -> Res<()> {
    fs::create_dir_all(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    for rel in project_files(project)? {
        let from = project.join(&rel);
        let to = dest.join(&rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::copy(&from, &to).map_err(|e| format!("{}: {e}", from.display()))?;
    }
    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> Res<()> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=bench@example.invalid",
            "-c",
            "user.name=bench",
        ])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn find_dla(arg: Option<&Path>) -> PathBuf {
    if let Some(p) = arg {
        return p.to_path_buf();
    }
    let exe = if cfg!(windows) { "dla.exe" } else { "dla" };
    if let Ok(me) = env::current_exe()
        && let Some(dir) = me.parent()
        && dir.join(exe).is_file()
    {
        return dir.join(exe);
    }
    let dev = Path::new("target/debug").join(exe);
    if dev.is_file() {
        return dev;
    }
    PathBuf::from("dla")
}

struct Dla<'a> {
    bin: &'a Path,
    project: &'a Path,
    plugin_root: &'a Path,
    data_dir: &'a Path,
}

impl Dla<'_> {
    /// Runs one dla command; returns its stdout JSON or an error line.
    fn call(&self, args: &[&str]) -> Result<Value, String> {
        let out = Command::new(self.bin)
            .arg("--project")
            .arg(self.project)
            .args(args)
            .env("DLA_PLUGIN_ROOT", self.plugin_root)
            .env("DLA_DATA_DIR", self.data_dir)
            .env("DLA_CONSENT_TIERS", "1-4")
            .env_remove("DLA_RUN")
            .current_dir(self.project)
            .output()
            .map_err(|e| format!("dla {}: {e}", args.join(" ")))?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        let json: Value = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
        if out.status.success() && json.get("ok") == Some(&Value::Bool(true)) {
            Ok(json)
        } else {
            let msg = json
                .get("message")
                .and_then(Value::as_str)
                .map_or_else(|| stdout.trim().chars().take(300).collect(), str::to_owned);
            Err(format!(
                "dla {} exited {}: {msg} {}",
                args.join(" "),
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr)
                    .trim()
                    .chars()
                    .take(200)
                    .collect::<String>()
            ))
        }
    }
}

struct Candidate {
    origin: String,
    site: String,
}

fn read_candidates(ledger: &Path) -> Res<Vec<Candidate>> {
    let text = fs::read_to_string(ledger).map_err(|e| format!("{}: {e}", ledger.display()))?;
    let mut v = Vec::new();
    for line in text.lines() {
        let Ok(rec) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if rec.get("type").and_then(Value::as_str) != Some("finding_added") {
            continue;
        }
        let Some(f) = rec.get("finding") else {
            continue;
        };
        v.push(Candidate {
            origin: f
                .get("origin")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            site: f
                .get("site")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
        });
    }
    Ok(v)
}

fn current_run(project: &Path) -> Res<PathBuf> {
    let id = fs::read_to_string(project.join(".dla/current"))
        .map_err(|e| format!(".dla/current: {e}"))?;
    Ok(project.join(".dla").join(id.trim()))
}

fn run_fixture(
    fixtures: &Path,
    fixture_dir: &Path,
    bin: &Path,
    plugin_root: &Path,
    work: &Path,
) -> Res<FixtureRow> {
    let id = fixture_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let expected = load_expected(fixtures, &id)?;
    let project = work.join(&id);
    copy_project(&fixture_dir.join("project"), &project)?;
    git(&project, &["init", "-q"])?;
    git(&project, &["add", "-A"])?;
    git(&project, &["commit", "-q", "-m", "fixture"])?;
    let data_dir = work.join("data").join(&id);
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let dla = Dla {
        bin,
        project: &project,
        plugin_root,
        data_dir: &data_dir,
    };
    let mut row = FixtureRow {
        id: id.clone(),
        ..FixtureRow::default()
    };
    let stage = |row: &mut FixtureRow, args: &[&str]| -> Option<Value> {
        match dla.call(args) {
            Ok(v) => Some(v),
            Err(e) => {
                row.errors.push(e);
                None
            }
        }
    };

    if stage(&mut row, &["run", "init"]).is_none() {
        row.failures.push("dla run init failed".to_owned());
        return Ok(row);
    }
    if let Some(p) = stage(&mut row, &["probe", "--build"]) {
        row.runtimes = p["summary"]["runtimes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|r| r.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
    }
    for lang in row.runtimes.clone() {
        stage(&mut row, &["sweep", "--lang", &lang]);
        stage(&mut row, &["census", "--lang", &lang]);
    }
    stage(&mut row, &["workload", "discover"]);

    let run_dir = current_run(&project)?;

    // Candidates and the leaks they name.
    let cands = read_candidates(&run_dir.join("ledger.jsonl"))?;
    row.candidates = cands.len();
    for c in &cands {
        *row.by_origin.entry(c.origin.clone()).or_default() += 1;
    }
    let findings: Vec<Finding> = cands
        .iter()
        .map(|c| Finding {
            state: State::Open,
            site: Some(c.site.clone()),
            holder: None,
            label: None,
            hypothesis: false,
            discriminator: false,
        })
        .collect();
    let outcomes = leak_outcomes(&expected, &findings);
    for (leak, outcome) in expected.leak.iter().zip(outcomes) {
        let origins: BTreeSet<String> = cands
            .iter()
            .zip(&findings)
            .filter(|(_, f)| site_matches(f, leak))
            .map(|(c, _)| c.origin.clone())
            .collect();
        row.leaks.push(LeakRow {
            id: leak.id.clone(),
            site: leak.site.clone(),
            confirmable: leak.confirmable,
            flagged: outcome.flagged,
            origins: origins.into_iter().collect(),
        });
    }

    // Coverage: enumerated counts, missing and blocked backends.
    if let Ok(text) = fs::read_to_string(run_dir.join("coverage.json"))
        && let Ok(cov) = serde_json::from_str::<Value>(&text)
        && let Some(workers) = cov["workers"].as_object()
    {
        for (worker, w) in workers {
            let Some(backends) = w["backends"].as_object() else {
                continue;
            };
            for (name, b) in backends {
                let key = format!("{worker}/{name}");
                let n = b["enumerated"].as_u64().unwrap_or(0);
                row.enumerated += n;
                row.enumerated_by_backend.insert(key.clone(), n);
                match b["status"].as_str() {
                    Some("missing") => row.backends_missing.push(key),
                    Some("blocked") => row.backends_blocked.push(key),
                    _ => {}
                }
            }
        }
    }

    // Workloads.
    if let Ok(text) = fs::read_to_string(run_dir.join("workload/workloads.json"))
        && let Ok(w) = serde_json::from_str::<Value>(&text)
        && let Some(list) = w["workloads"].as_array()
    {
        for item in list {
            match item["kind"].as_str() {
                Some("test") => row.workloads.test += 1,
                Some("main") => row.workloads.main += 1,
                Some("script") => row.workloads.script += 1,
                Some("dev-server") => row.workloads.dev_server += 1,
                _ => {}
            }
        }
    }

    // Acceptance.
    for e in &row.errors {
        row.failures.push(format!("stage failed: {e}"));
    }
    for l in row.leaks.iter().filter(|l| l.confirmable && !l.flagged) {
        row.failures.push(format!(
            "leak {} ({}) is not flagged by any candidate",
            l.id, l.site
        ));
    }
    if row.workloads.test == 0 {
        row.failures.push("no test workload discovered".to_owned());
    }
    if row.workloads.main + row.workloads.script == 0 {
        row.failures
            .push("no main or script workload discovered".to_owned());
    }
    Ok(row)
}

/// The text table.
pub fn render_table(res: &StaticResult) -> String {
    let mut s = format!("archetypes: {}\n\n", res.archetypes);
    let w = res
        .fixtures
        .iter()
        .map(|f| f.id.len())
        .max()
        .unwrap_or(7)
        .max(7);
    s.push_str(&format!(
        "{:<w$}  {:<7}  {:>6}  {:>4}  {:>4}  {:>6}  {:>5}  {:>5}  {:>9}  {:>7}  {:>7}\n",
        "fixture",
        "runtime",
        "leaks",
        "test",
        "main",
        "script",
        "cands",
        "enum",
        "confirmed",
        "missing",
        "blocked",
    ));
    for f in &res.fixtures {
        let conf: Vec<&LeakRow> = f.leaks.iter().filter(|l| l.confirmable).collect();
        let hit = conf.iter().filter(|l| l.flagged).count();
        let all_hit = f.leaks.iter().filter(|l| l.flagged).count();
        s.push_str(&format!(
            "{:<w$}  {:<7}  {:>6}  {:>4}  {:>4}  {:>6}  {:>5}  {:>5}  {:>9}  {:>7}  {:>7}\n",
            f.id,
            f.runtimes.join(","),
            format!("{all_hit}/{}", f.leaks.len()),
            f.workloads.test,
            f.workloads.main,
            f.workloads.script,
            f.candidates,
            f.enumerated,
            format!("{hit}/{}", conf.len()),
            f.backends_missing.len(),
            f.backends_blocked.len(),
        ));
    }
    s.push_str("\nleaks (flagged by origin)\n");
    for f in &res.fixtures {
        for l in &f.leaks {
            s.push_str(&format!(
                "  {}/{} {} {} {}\n",
                f.id,
                l.id,
                l.site,
                if l.flagged { "FLAGGED" } else { "MISSED" },
                if l.confirmable { "" } else { "(static only) " },
            ));
            if !l.origins.is_empty() {
                s.push_str(&format!("      by {}\n", l.origins.join(", ")));
            }
        }
    }
    s.push_str("\nbackends missing or blocked\n");
    for f in &res.fixtures {
        if f.backends_missing.is_empty() && f.backends_blocked.is_empty() {
            continue;
        }
        s.push_str(&format!(
            "  {}: missing [{}] blocked [{}]\n",
            f.id,
            f.backends_missing.join(", "),
            f.backends_blocked.join(", ")
        ));
    }
    s
}

/// Runs the harness.
pub fn run(args: &StaticArgs) -> Res<String> {
    let bin = find_dla(args.dla_bin.as_deref());
    let data_src = args.repo_root.join("data");
    if !data_src.is_dir() {
        return Err(format!("{}: no data directory", data_src.display()));
    }
    let work = env::temp_dir().join(format!("dla-bench-static-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).map_err(|e| format!("{}: {e}", work.display()))?;
    let result = run_in(args, &bin, &data_src, &work);
    let _ = fs::remove_dir_all(&work);
    let res = result?;
    print!("{}", render_table(&res));
    if let Some(out) = &args.json {
        let text = serde_json::to_string_pretty(&res).map_err(|e| e.to_string())?;
        fs::write(out, text + "\n").map_err(|e| format!("{}: {e}", out.display()))?;
    }
    if res.failures.is_empty() {
        Ok(format!("ok: {} fixture(s) pass", res.fixtures.len()))
    } else {
        Err(format!(
            "{} failure(s):\n{}",
            res.failures.len(),
            res.failures.join("\n")
        ))
    }
}

fn run_in(args: &StaticArgs, bin: &Path, data_src: &Path, work: &Path) -> Res<StaticResult> {
    let plugin_root = work.join("plugin");
    copy_tree(data_src, &plugin_root.join("data"))?;
    let off = args.archetypes == Archetypes::Off;
    if off {
        apply_archetypes_off(&plugin_root)?;
    }
    let mut fixtures = Vec::new();
    for dir in fixture_dirs(&args.fixtures)? {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if args
            .only
            .as_ref()
            .is_some_and(|o| !name.contains(o.as_str()))
        {
            continue;
        }
        if !dir.join("expected.toml").is_file() {
            continue;
        }
        fixtures.push(run_fixture(&args.fixtures, &dir, bin, &plugin_root, work)?);
    }
    if fixtures.is_empty() {
        return Err(format!("no fixtures under {}", args.fixtures.display()));
    }
    let failures = fixtures
        .iter()
        .flat_map(|f| f.failures.iter().map(|m| format!("{}: {m}", f.id)))
        .collect();
    Ok(StaticResult {
        archetypes: if off { "off" } else { "on" }.to_owned(),
        fixtures,
        failures,
    })
}
