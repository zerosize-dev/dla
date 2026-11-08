//! The grader: `bench/GRADING.md` implemented literally.

use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use regex::Regex;
use serde_json::Value;

use crate::{
    Res,
    fixture::load_expected,
    model::{
        Category, Cost, Expected, FixtureStats, Group, Leak, LeakCounts, Metrics, MetricsFile,
        PerRun, Ratio, Sources, SumMean, Unexplained, UsageStats, WallSeconds,
    },
};

/// Rounds to six decimals, half to even, on the exact binary value.
pub fn round6(x: f64) -> f64 {
    format!("{x:.6}").parse().unwrap_or(x)
}

/// State of a normalised finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Confirmed.
    Confirmed,
    /// Strong.
    Strong,
    /// Refuted.
    Refuted,
    /// Suspected (static only).
    Suspected,
    /// Unresolved.
    Unresolved,
    /// Anything else.
    Open,
}

/// A finding reduced to what grading needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// State.
    pub state: State,
    /// `path:line`.
    pub site: Option<String>,
    /// Holder.
    pub holder: Option<String>,
    /// Archetype label.
    pub label: Option<String>,
    /// Opened as a hypothesis under another finding.
    pub hypothesis: bool,
    /// Carries a ruling-out discriminator.
    pub discriminator: bool,
}

impl Finding {
    /// A non-hypothesis finding in state confirmed or strong.
    pub fn is_claim(&self) -> bool {
        !self.hypothesis && matches!(self.state, State::Confirmed | State::Strong)
    }

    /// A refuted finding with a discriminator.
    pub fn is_rule_out(&self) -> bool {
        self.state == State::Refuted && self.discriminator
    }
}

/// Where a run's findings came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// `ledger.jsonl`.
    Ledger,
    /// `findings.json`.
    Findings,
    /// Neither.
    None,
}

fn str_of(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn state_of_to(to: Option<&str>) -> State {
    match to {
        Some("confirmed") => State::Confirmed,
        Some("strong") => State::Strong,
        Some("refuted") => State::Refuted,
        Some("suspected_static_only") => State::Suspected,
        Some("unresolved") => State::Unresolved,
        _ => State::Open,
    }
}

/// Replays a ledger leniently. `None` when it holds no `finding_added` record.
pub fn findings_from_ledger(text: &str) -> Option<Vec<Finding>> {
    findings_from_ledger_bytes(text.as_bytes())
}

/// Like [`findings_from_ledger`] on raw bytes: lines are split on the byte
/// `\n` and decoded one by one; a line that is not valid UTF-8 is skipped
/// like any other line that is not a JSON object.
pub fn findings_from_ledger_bytes(bytes: &[u8]) -> Option<Vec<Finding>> {
    // Findings in the order of their (latest) `finding_added` lines.
    let mut list: Vec<(String, Finding)> = Vec::new();
    let mut saw_added = false;
    for line in bytes.split(|b| *b == b'\n') {
        let Ok(line) = std::str::from_utf8(line) else {
            continue;
        };
        let Ok(rec) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if !rec.is_object() {
            continue;
        }
        let Some(kind) = rec.get("type").and_then(Value::as_str) else {
            continue;
        };
        match kind {
            "finding_added" => {
                saw_added = true;
                let Some(f) = rec.get("finding").filter(|f| f.is_object()) else {
                    continue;
                };
                let Some(id) = f.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let finding = Finding {
                    state: State::Open,
                    site: str_of(f, "site"),
                    holder: str_of(f, "holder"),
                    label: str_of(f, "archetype"),
                    hypothesis: f
                        .get("parent")
                        .and_then(Value::as_str)
                        .is_some_and(|p| !p.is_empty()),
                    discriminator: false,
                };
                list.retain(|(i, _)| i != id);
                list.push((id.to_owned(), finding));
            }
            "transition" => {
                let Some(id) = rec.get("finding").and_then(Value::as_str) else {
                    continue;
                };
                if let Some((_, f)) = list.iter_mut().find(|(i, _)| i == id) {
                    f.state = state_of_to(rec.get("to").and_then(Value::as_str));
                }
            }
            // `dla ledger annotate --holder/--site` fills a holder or site the finding
            // was added without; it never changes one that is set.
            "finding_annotated" => {
                let Some(id) = rec.get("finding").and_then(Value::as_str) else {
                    continue;
                };
                if let Some((_, f)) = list.iter_mut().find(|(i, _)| i == id) {
                    if f.holder.is_none() {
                        f.holder = str_of(&rec, "holder");
                    }
                    if f.site.is_none() {
                        f.site = str_of(&rec, "site");
                    }
                }
            }
            _ => {}
        }
    }
    if !saw_added {
        return None;
    }
    Some(
        list.into_iter()
            .map(|(_, mut f)| {
                f.discriminator = f.state == State::Refuted;
                f
            })
            .collect(),
    )
}

fn label_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(L-[A-Z]+|N-[A-Z]+|UNCLASSIFIED)$").expect("valid regex"))
}

/// The whitespace of the grading contract: ASCII space, tab, CR and LF only.
fn is_ascii_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

/// Maps a parsed `findings.json`. `None` unless it is an object whose
/// `findings` member is an array.
pub fn findings_from_json(v: &Value) -> Option<Vec<Finding>> {
    let arr = v.as_object()?.get("findings")?.as_array()?;
    let mut out = Vec::new();
    for e in arr {
        if !e.is_object() {
            continue;
        }
        let verdict = e.get("verdict").and_then(Value::as_str);
        let state = match verdict {
            Some("leak-confirmed") => State::Confirmed,
            Some("leak-suspected") => State::Suspected,
            Some("not-a-leak") => State::Refuted,
            _ => State::Open,
        };
        let label = e
            .get("cause")
            .and_then(Value::as_str)
            .filter(|c| label_re().is_match(c))
            .map(str::to_owned);
        let discriminator = verdict == Some("not-a-leak")
            && e.get("discriminator")
                .and_then(Value::as_str)
                .is_some_and(|d| !d.trim_matches(is_ascii_ws).is_empty());
        out.push(Finding {
            state,
            site: str_of(e, "site"),
            holder: str_of(e, "holder"),
            label,
            hypothesis: false,
            discriminator,
        });
    }
    Some(out)
}

/// Applies the source selection rule to the optional file contents.
pub fn select_source(ledger: Option<&str>, findings_json: Option<&str>) -> (Source, Vec<Finding>) {
    select_source_bytes(ledger.map(str::as_bytes), findings_json.map(str::as_bytes))
}

/// [`select_source`] on raw file contents: a `findings.json` that is not
/// valid UTF-8 is treated as absent, and the ledger is read line by line.
pub fn select_source_bytes(
    ledger: Option<&[u8]>,
    findings_json: Option<&[u8]>,
) -> (Source, Vec<Finding>) {
    if let Some(f) = ledger.and_then(findings_from_ledger_bytes) {
        return (Source::Ledger, f);
    }
    if let Some(v) = findings_json
        .and_then(|b| std::str::from_utf8(b).ok())
        .and_then(|t| serde_json::from_str::<Value>(t).ok())
        && let Some(f) = findings_from_json(&v)
    {
        return (Source::Findings, f);
    }
    (Source::None, Vec::new())
}

fn site_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(.*?):([0-9]+)(?::[0-9]+)?$").expect("valid regex"))
}

fn alt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(.*?):([0-9]+)(?:-([0-9]+))?$").expect("valid regex"))
}

/// Replaces `\` with `/` and removes every leading `./`.
pub fn normalise_path(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    while let Some(rest) = s.strip_prefix("./") {
        s = rest.to_owned();
    }
    s
}

/// Parses a finding's site into a normalised path and a line.
pub fn parse_site(site: &str) -> Option<(String, u64)> {
    let c = site_re().captures(site)?;
    let line = c[2].parse().ok()?;
    Some((normalise_path(&c[1]), line))
}

fn path_matches(found: &str, wanted: &str) -> bool {
    found == wanted || found.ends_with(&format!("/{wanted}"))
}

fn leak_ranges(leak: &Leak) -> Vec<(String, u64, u64)> {
    let mut v = Vec::new();
    if let Some(c) = site_re().captures(&leak.site)
        && let Ok(line) = c[2].parse::<u64>()
    {
        let (lo, hi) = leak
            .lines
            .map_or((line, line), |[a, b]| (u64::from(a), u64::from(b)));
        v.push((c[1].to_owned(), lo, hi));
    }
    for alt in &leak.alt_sites {
        if let Some(c) = alt_re().captures(alt)
            && let Ok(lo) = c[2].parse::<u64>()
        {
            let hi = match c.get(3) {
                Some(m) => match m.as_str().parse::<u64>() {
                    Ok(h) => h,
                    Err(_) => continue,
                },
                None => lo,
            };
            v.push((c[1].to_owned(), lo, hi));
        }
    }
    v
}

/// Whether the finding's site names the leak (site range or any alt site).
pub fn site_matches(finding: &Finding, leak: &Leak) -> bool {
    let Some((path, line)) = finding.site.as_deref().and_then(parse_site) else {
        return false;
    };
    leak_ranges(leak)
        .iter()
        .any(|(p, lo, hi)| path_matches(&path, p) && *lo <= line && line <= *hi)
}

/// The outcome of one expected leak in one run.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LeakOutcome {
    /// Some non-hypothesis confirmed or strong finding matches.
    pub found: bool,
    /// Some non-hypothesis live finding matches.
    pub flagged: bool,
    /// Holder of the chosen finding, when found.
    pub chosen_holder: Option<String>,
    /// Label of the chosen finding, when found.
    pub chosen_label: Option<String>,
}

/// Evaluates every expected leak against the findings of one run.
pub fn leak_outcomes(exp: &Expected, findings: &[Finding]) -> Vec<LeakOutcome> {
    exp.leak
        .iter()
        .map(|leak| {
            let matching: Vec<&Finding> =
                findings.iter().filter(|f| site_matches(f, leak)).collect();
            let found = matching.iter().any(|f| f.is_claim());
            let flagged = matching.iter().any(|f| {
                !f.hypothesis
                    && matches!(
                        f.state,
                        State::Confirmed
                            | State::Strong
                            | State::Suspected
                            | State::Unresolved
                            | State::Open
                    )
            });
            let chosen = if found {
                matching
                    .iter()
                    .find(|f| !f.hypothesis && f.state == State::Strong)
                    .or_else(|| {
                        matching
                            .iter()
                            .find(|f| !f.hypothesis && f.state == State::Confirmed)
                    })
            } else {
                None
            };
            LeakOutcome {
                found,
                flagged,
                chosen_holder: chosen.and_then(|f| f.holder.clone()),
                chosen_label: chosen.and_then(|f| f.label.clone()),
            }
        })
        .collect()
}

/// Metric names in output order.
pub const METRIC_NAMES: [&str; 12] = [
    "leak_recall",
    "archetype_free_recall",
    "flagged_recall",
    "second_leak_recall",
    "grounded_attribution",
    "label_accuracy",
    "forced_fit_rate",
    "overclaim_rate",
    "false_confirmation_rate",
    "non_leak_precision",
    "trap_miss_rate",
    "trap_false_alarm_rate",
];

/// One run, graded.
#[derive(Debug, Clone)]
pub struct RunGrade {
    /// Fixture category.
    pub category: Category,
    /// Source of the findings.
    pub source: Source,
    /// At least one claim.
    pub claim: bool,
    /// At least one rule-out.
    pub rule_out: bool,
    /// Per expected leak.
    pub leaks: Vec<LeakOutcome>,
    /// `(num, den)` of the twelve metrics, in [`METRIC_NAMES`] order.
    pub parts: [(u64, u64); 12],
}

/// Grades one run's normalised findings against its fixture.
pub fn grade_findings(exp: &Expected, source: Source, findings: &[Finding]) -> RunGrade {
    let cat = exp.category;
    let outcomes = leak_outcomes(exp, findings);
    let claim = findings.iter().any(Finding::is_claim);
    let rule_out = findings.iter().any(Finding::is_rule_out);
    let mut p = [(0u64, 0u64); 12];
    let in_set = matches!(
        cat,
        Category::ArchetypeLeak | Category::ArchetypeFree | Category::MultiLeak
    );
    if in_set {
        let mut found_n = 0u64;
        let mut set_n = 0u64;
        for (leak, o) in exp.leak.iter().zip(&outcomes) {
            if !leak.confirmable {
                continue;
            }
            set_n += 1;
            p[0].1 += 1;
            p[2].1 += 1;
            let unclassified = leak.cause == "UNCLASSIFIED";
            if unclassified {
                p[1].1 += 1;
            }
            if o.flagged {
                p[2].0 += 1;
            }
            if o.found {
                found_n += 1;
                p[0].0 += 1;
                if unclassified {
                    p[1].0 += 1;
                }
                p[4].1 += 1;
                if o.chosen_holder.as_deref() == Some(leak.holder.as_str()) {
                    p[4].0 += 1;
                }
                if let Some(label) = &o.chosen_label {
                    if unclassified {
                        p[6].1 += 1;
                        if label != "UNCLASSIFIED" {
                            p[6].0 += 1;
                        }
                    } else {
                        p[5].1 += 1;
                        if *label == leak.cause {
                            p[5].0 += 1;
                        }
                    }
                }
            }
        }
        if cat == Category::MultiLeak && found_n >= 1 {
            p[3] = (found_n - 1, set_n - 1);
        }
    }
    if cat == Category::Impossible {
        p[7] = (u64::from(claim), 1);
    }
    if matches!(cat, Category::DecoyControl | Category::NonLeak) {
        p[8] = (u64::from(claim), 1);
    }
    if cat == Category::NonLeak {
        p[9] = (u64::from(!claim && rule_out), 1);
    }
    if cat == Category::ArchetypeLeak && exp.pair.is_some() {
        let missed = exp
            .leak
            .iter()
            .zip(&outcomes)
            .any(|(l, o)| l.confirmable && !o.found);
        p[10] = (u64::from(missed), 1);
    }
    if cat == Category::DecoyControl {
        p[11] = (u64::from(claim), 1);
    }
    RunGrade {
        category: cat,
        source,
        claim,
        rule_out,
        leaks: outcomes,
        parts: p,
    }
}

/// A run directory as read from disk.
#[derive(Debug)]
pub struct RunData {
    /// Fixture ID.
    pub fixture: String,
    /// Run index.
    pub n: u32,
    /// Parsed `run.json`.
    pub run_json: Value,
    /// Graded findings.
    pub grade: RunGrade,
    /// `(residual, total_slope)` sums when coverage has a non-empty residual.
    pub residual: Option<(f64, f64)>,
}

/// A run index: a positive decimal integer without leading zeros.
fn run_index(name: &str) -> Option<u32> {
    if name.is_empty() || name.starts_with('0') || !name.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    name.parse().ok()
}

fn read_dirs(p: &Path) -> Res<Vec<(String, PathBuf)>> {
    let rd = fs::read_dir(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut v: Vec<(String, PathBuf)> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    v.sort();
    Ok(v)
}

/// A `coverage.json` text; absent when unreadable or not valid UTF-8.
fn read_utf8_opt(p: &Path) -> Option<String> {
    fs::read(p).ok().and_then(|b| String::from_utf8(b).ok())
}

fn residual_of(text: &str) -> Option<(f64, f64)> {
    let v: Value = serde_json::from_str(text).ok()?;
    let rows = v.get("residual")?.as_array()?;
    if rows.is_empty() {
        return None;
    }
    let (mut r, mut t) = (0.0, 0.0);
    for row in rows {
        r += row.get("residual").and_then(Value::as_f64).unwrap_or(0.0);
        t += row
            .get("total_slope")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
    }
    Some((r, t))
}

/// Grades the runs under one results directory into `metrics.json` content.
pub fn grade_dir(results: &Path, fixtures: &Path) -> Res<MetricsFile> {
    let runs_root = results.join("runs");
    if !runs_root.is_dir() {
        return Err(format!("{}: no runs/ directory", results.display()));
    }
    let mut expected: HashMap<String, Expected> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    // (ablation, archetypes) -> runs
    let mut groups: BTreeMap<(String, String), (Vec<RunData>, u64)> = BTreeMap::new();
    for (abl, abl_dir) in read_dirs(&runs_root)? {
        if !["no-skill", "prose", "gates", "full"].contains(&abl.as_str()) {
            continue;
        }
        for (arch_dir_name, arch_dir) in read_dirs(&abl_dir)? {
            let arch = match arch_dir_name.as_str() {
                "archetypes-on" => "on",
                "archetypes-off" => "off",
                _ => continue,
            };
            for (fx, fx_dir) in read_dirs(&arch_dir)? {
                for (idx, run_dir) in read_dirs(&fx_dir)? {
                    let Some(n) = run_index(&idx) else {
                        continue;
                    };
                    let key = (abl.clone(), arch.to_owned());
                    let Ok(bytes) = fs::read(run_dir.join("run.json")) else {
                        groups.entry(key).or_default().1 += 1;
                        continue;
                    };
                    let rj = match std::str::from_utf8(&bytes) {
                        Err(_) => {
                            errors.push(format!(
                                "{}: run.json is not valid UTF-8",
                                run_dir.display()
                            ));
                            continue;
                        }
                        Ok(text) => match serde_json::from_str::<Value>(text) {
                            Ok(v) if v.is_object() => v,
                            _ => {
                                errors.push(format!(
                                    "{}: run.json is not a JSON object",
                                    run_dir.display()
                                ));
                                continue;
                            }
                        },
                    };
                    // Only a fixture with a finished run needs its expected.toml.
                    if !expected.contains_key(&fx) {
                        match load_expected(fixtures, &fx) {
                            Ok(e) => {
                                expected.insert(fx.clone(), e);
                            }
                            Err(e) => {
                                errors.push(format!("{}: fixture {fx}: {e}", fx_dir.display()));
                                break;
                            }
                        }
                    }
                    let exp = &expected[&fx];
                    let ledger = fs::read(run_dir.join("ledger.jsonl")).ok();
                    let fj = fs::read(run_dir.join("findings.json")).ok();
                    let (source, findings) = select_source_bytes(ledger.as_deref(), fj.as_deref());
                    let grade = grade_findings(exp, source, &findings);
                    let residual = read_utf8_opt(&run_dir.join("coverage.json"))
                        .as_deref()
                        .and_then(residual_of);
                    groups.entry(key).or_default().0.push(RunData {
                        fixture: fx.clone(),
                        n,
                        run_json: rj,
                        grade,
                        residual,
                    });
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let mut out = Vec::new();
    for ((abl, arch), (mut runs, incomplete)) in groups {
        runs.sort_by(|a, b| (a.fixture.as_str(), a.n).cmp(&(b.fixture.as_str(), b.n)));
        out.push(aggregate(&abl, &arch, &runs, incomplete, &expected));
    }
    Ok(MetricsFile {
        schema_version: 1,
        groups: out,
    })
}

fn div(num: u64, den: u64) -> Option<f64> {
    (den > 0).then(|| round6(num as f64 / den as f64))
}

fn build_ratio(idx: usize, runs: &[RunData], max_run: u32) -> Ratio {
    let mut num = 0;
    let mut den = 0;
    let mut per_run = Vec::new();
    for i in 1..=max_run {
        let (mut n, mut d) = (0, 0);
        for r in runs.iter().filter(|r| r.n == i) {
            n += r.grade.parts[idx].0;
            d += r.grade.parts[idx].1;
        }
        num += n;
        den += d;
        per_run.push(PerRun {
            run: i,
            num: n,
            den: d,
            value: div(n, d),
        });
    }
    let vals: Vec<f64> = per_run
        .iter()
        .filter(|p| p.den > 0)
        .map(|p| p.num as f64 / p.den as f64)
        .collect();
    let (mean, stdev) = if vals.is_empty() {
        (None, None)
    } else {
        let k = vals.len() as f64;
        let s: f64 = vals.iter().sum();
        let m = s / k;
        let q: f64 = vals.iter().map(|v| (v - m) * (v - m)).sum();
        (Some(round6(m)), Some(round6((q / k).sqrt())))
    };
    Ratio {
        num,
        den,
        value: div(num, den),
        per_run,
        mean,
        stdev,
    }
}

fn token(rj: &Value, key: &str) -> u64 {
    rj.get("usage")
        .and_then(|u| u.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn aggregate(
    abl: &str,
    arch: &str,
    runs: &[RunData],
    incomplete: u64,
    expected: &HashMap<String, Expected>,
) -> Group {
    let max_run = runs.iter().map(|r| r.n).max().unwrap_or(0);
    let m = |i| build_ratio(i, runs, max_run);
    let metrics = Metrics {
        leak_recall: m(0),
        archetype_free_recall: m(1),
        flagged_recall: m(2),
        second_leak_recall: m(3),
        grounded_attribution: m(4),
        label_accuracy: m(5),
        forced_fit_rate: m(6),
        overclaim_rate: m(7),
        false_confirmation_rate: m(8),
        non_leak_precision: m(9),
        trap_miss_rate: m(10),
        trap_false_alarm_rate: m(11),
    };
    // Unexplained growth.
    let (mut r_sum, mut t_sum, mut k) = (0.0, 0.0, 0u64);
    for run in runs {
        if let Some((r, t)) = run.residual {
            r_sum += r;
            t_sum += t;
            k += 1;
        }
    }
    let unexplained = Unexplained {
        value: if k > 0 && t_sum != 0.0 {
            Some(round6(r_sum / t_sum))
        } else {
            None
        },
        runs: k,
    };
    // Usage.
    let count = runs.len() as u64;
    let mut sources = Sources::default();
    let mut errored = 0;
    let mut wall_sum = 0.0;
    let mut wall_max = 0.0f64;
    let (mut cost_sum, mut cost_n) = (0.0, 0u64);
    for r in runs {
        match r.grade.source {
            Source::Ledger => sources.ledger += 1,
            Source::Findings => sources.findings += 1,
            Source::None => sources.none += 1,
        }
        let exit_ok = r.run_json.get("exit_code").and_then(Value::as_i64) == Some(0);
        let timed_out = r.run_json.get("timed_out").and_then(Value::as_bool) == Some(true);
        if !exit_ok || timed_out {
            errored += 1;
        }
        let w = r
            .run_json
            .get("wall_seconds")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        wall_sum += w;
        wall_max = wall_max.max(w);
        if let Some(c) = r.run_json.get("cost_usd").and_then(Value::as_f64) {
            cost_sum += c;
            cost_n += 1;
        }
    }
    let tok = |key: &str| {
        let sum: u64 = runs.iter().map(|r| token(&r.run_json, key)).sum();
        SumMean {
            sum,
            mean: div(sum, count),
        }
    };
    let usage = UsageStats {
        runs: count,
        incomplete,
        errored,
        sources,
        input_tokens: tok("input_tokens"),
        output_tokens: tok("output_tokens"),
        cache_read_input_tokens: tok("cache_read_input_tokens"),
        cache_creation_input_tokens: tok("cache_creation_input_tokens"),
        wall_seconds: WallSeconds {
            sum: round6(wall_sum),
            mean: (count > 0).then(|| round6(wall_sum / count as f64)),
            max: (count > 0).then(|| round6(wall_max)),
        },
        cost_usd: Cost {
            sum: round6(cost_sum),
            mean: (cost_n > 0).then(|| round6(cost_sum / cost_n as f64)),
        },
    };
    // Per fixture.
    let mut fixtures: BTreeMap<String, FixtureStats> = BTreeMap::new();
    for r in runs {
        let exp = &expected[&r.fixture];
        let st = fixtures
            .entry(r.fixture.clone())
            .or_insert_with(|| FixtureStats {
                category: exp.category.as_str().to_owned(),
                runs: 0,
                claim_runs: 0,
                rule_out_runs: 0,
                leaks: exp
                    .leak
                    .iter()
                    .map(|l| (l.id.clone(), LeakCounts::default()))
                    .collect(),
            });
        st.runs += 1;
        st.claim_runs += u64::from(r.grade.claim);
        st.rule_out_runs += u64::from(r.grade.rule_out);
        for (leak, o) in exp.leak.iter().zip(&r.grade.leaks) {
            let c = st.leaks.entry(leak.id.clone()).or_default();
            c.found += u64::from(o.found);
            c.flagged += u64::from(o.flagged);
            c.grounded +=
                u64::from(o.found && o.chosen_holder.as_deref() == Some(leak.holder.as_str()));
        }
    }
    Group {
        ablation: abl.to_owned(),
        archetypes: arch.to_owned(),
        metrics,
        unexplained_growth_share: unexplained,
        usage,
        fixtures,
    }
}

/// Serialises metrics as pretty JSON with a trailing newline.
pub fn render(m: &MetricsFile) -> String {
    let mut s = serde_json::to_string_pretty(m).expect("metrics serialise");
    s.push('\n');
    s
}

/// Grades `results` and writes `out`.
pub fn grade_to_file(results: &Path, fixtures: &Path, out: &Path) -> Res<MetricsFile> {
    let m = grade_dir(results, fixtures)?;
    fs::write(out, render(&m)).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(m)
}
