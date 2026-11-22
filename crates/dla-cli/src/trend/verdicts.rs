//! The verdict of `dla trend` for every targeted finding of a harness series.
//!
//! The targets are the findings the series' harness declared (`--probe`, `--next`) and every
//! finding an automatic probe of the harness stands for. Per finding, the first that applies:
//! `grew`, `changed`, `constant`, `not_loaded`, then (no probe) `executed` or `not_executed`
//! from the run's coverage, else `unknown`.

use std::{collections::BTreeMap, fs, path::Path};

use dla_census::parse::{Lang, parse};
use dla_core::{
    FindingId, Ledger,
    harness::HarnessManifest,
    series::SeriesFile,
    trend::{ColumnTrend, TargetVerdict, Verdict, probe_column_matches_owner},
};
use dla_stats::GateVerdict;
use serde_json::Value;

use super::candidates::{CLASSES, FUNCTIONS};

/// Everything a verdict reads.
pub struct Inputs<'a> {
    /// The project root.
    pub project: &'a Path,
    /// The run directory.
    pub run_dir: &'a Path,
    /// The ledger.
    pub ledger: &'a Ledger,
    /// The series.
    pub series: &'a SeriesFile,
    /// The analysed columns, with their candidates.
    pub columns: &'a [ColumnTrend],
    /// `(points, needed)` when the series has fewer points after warm-up than the gate needs.
    pub too_short: Option<(usize, usize)>,
}

/// The verdicts of the series, declared targets first. Empty for a series that no harness
/// measured.
pub fn compute(inp: &Inputs<'_>) -> Vec<TargetVerdict> {
    let Some(h) = inp.series.source.harness.as_deref() else {
        return Vec::new();
    };
    let Ok(m) = HarnessManifest::load(inp.run_dir, h) else {
        return Vec::new();
    };
    let mut ids: Vec<(FindingId, bool)> = m.targets.iter().map(|f| (f.clone(), true)).collect();
    for p in &m.probes {
        if let Some(f) = &p.finding
            && !ids.iter().any(|(x, _)| x == f)
        {
            ids.push((f.clone(), false));
        }
    }
    let chosen = chosen_tests(inp.run_dir, &m);
    let dropped = dropped_classes(inp.series);
    ids.into_iter()
        .filter(|(f, _)| inp.ledger.finding(f).is_some())
        .map(|(f, declared)| {
            let mut v = one(inp, &m, &f);
            // A flat reading of a test-loop target whose chosen test classes were all dropped
            // from the loop is no reading: no test chosen for it ran in the measured rounds
            // (its code may have run once, in the run that failed). It is a miss.
            if declared
                && matches!(v.verdict, Verdict::Executed | Verdict::Constant)
                && let Some(classes) = chosen.get(&f.to_string())
                && !classes.is_empty()
                && classes.iter().all(|c| dropped.contains(&c.as_str()))
            {
                v.verdict = Verdict::NotExecuted;
                v.detail = format!(
                    "the test class chosen for it ({}) was dropped from the loop (see the limitations of the series), so no test chosen for it ran in the measured rounds: this is a miss, not a flat reading",
                    classes.join(", ")
                );
            }
            // "The code ran and no column passed" says nothing about a series the gate could
            // not judge: unknown does not settle the target.
            if let Some((n, need)) = inp.too_short
                && v.verdict == Verdict::Executed
            {
                v.verdict = Verdict::Unknown;
                v.detail = format!("series too short: {n} points, {need} needed");
            }
            v.declared = declared;
            v
        })
        .collect()
}

/// The test classes a test-loop harness chose for each of its targets (`tests.json` of the
/// harness, written by `dla harness gen --junit --next`), by finding id. Empty for any other
/// harness.
fn chosen_tests(run_dir: &Path, m: &HarnessManifest) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if !m.is_test_loop() {
        return out;
    }
    let path = run_dir.join("harness").join(&m.id).join("tests.json");
    let Some(doc) = fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    else {
        return out;
    };
    for c in doc["classes"].as_array().into_iter().flatten() {
        let Some(class) = c["class"].as_str() else {
            continue;
        };
        for f in c["for"].as_array().into_iter().flatten() {
            if let Some(id) = f["finding"].as_str() {
                out.entry(id.to_owned()).or_default().push(class.to_owned());
            }
        }
    }
    out
}

/// The test classes the loop dropped (none of its tests passed, or it did not fit the time
/// limit), from the limitations of the series: `dropped test class <name>: <why>`.
fn dropped_classes(series: &SeriesFile) -> Vec<&str> {
    const MARK: &str = "dropped test class ";
    series
        .limitations
        .iter()
        .filter_map(|l| l.find(MARK).map(|i| &l[i + MARK.len()..]))
        .filter_map(|rest| rest.split_once(':').map(|(class, _)| class.trim()))
        .collect()
}

fn tied_to(c: &ColumnTrend, f: &FindingId) -> bool {
    c.finding.as_ref() == Some(f) || c.candidates.contains(f)
}

/// The post-warm-up values of a column in the series.
fn values(series: &SeriesFile, name: &str) -> Vec<f64> {
    let Some(i) = series.columns.iter().position(|c| c.name == name) else {
        return Vec::new();
    };
    series
        .points
        .iter()
        .skip(series.warmup as usize)
        .filter_map(|p| p.values.get(i).copied().flatten())
        .collect()
}

fn num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{v:.0}")
    } else {
        format!("{v:.3}")
    }
}

fn one(inp: &Inputs<'_>, m: &HarnessManifest, f: &FindingId) -> TargetVerdict {
    let make = |verdict, column: Option<&str>, detail: String| TargetVerdict {
        finding: f.clone(),
        verdict,
        column: column.map(str::to_owned),
        detail,
        declared: false,
    };
    // grew: a column tied to the finding passed the gate.
    if let Some(c) = inp
        .columns
        .iter()
        .find(|c| tied_to(c, f) && c.gate == GateVerdict::Pass)
    {
        return make(
            Verdict::Grew,
            Some(&c.name),
            format!(
                "column {} passed the growth gate ({} shape)",
                c.name,
                serde_json::to_value(c.shape)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default()
            ),
        );
    }
    // The probe columns of the finding.
    let owner = inp
        .ledger
        .finding(f)
        .and_then(|x| x.owner.clone())
        .unwrap_or_default();
    let probe_cols: Vec<&ColumnTrend> = inp
        .columns
        .iter()
        .filter(|c| c.name.starts_with("probe:"))
        .filter(|c| {
            c.finding.as_ref() == Some(f)
                || (!owner.is_empty() && probe_column_matches_owner(&c.name, &owner))
                || (c.finding.is_none() && c.candidates.contains(f))
        })
        .collect();
    if !probe_cols.is_empty() {
        let mut stayed: Option<(String, f64, usize)> = None;
        for c in &probe_cols {
            let v = values(inp.series, &c.name);
            if v.len() < 2 {
                continue;
            }
            let (lo, hi) = v
                .iter()
                .fold((f64::MAX, f64::MIN), |(l, h), x| (l.min(*x), h.max(*x)));
            if lo != hi {
                return make(
                    Verdict::Changed,
                    Some(&c.name),
                    format!(
                        "{} went from {} to {} over {} points but did not pass the growth gate",
                        c.name,
                        num(v[0]),
                        num(v[v.len() - 1]),
                        v.len()
                    ),
                );
            }
            stayed.get_or_insert((c.name.clone(), lo, v.len()));
        }
        return match stayed {
            Some((name, value, n)) => make(
                Verdict::Constant,
                Some(&name),
                format!(
                    "{name} stayed {} over {n} points: every unit used the same key or never reached a writer{}",
                    num(value),
                    // The key comes from the caller: one shared object is one key.
                    if inp
                        .ledger
                        .annotation(f)
                        .is_some_and(|a| a.facts.key_from_caller())
                    {
                        "; the key of this holder comes from the caller, so constant can also mean the unit passed the same keys every time (a key compared by identity needs a fresh object per unit: see the key line of the target in the driver file)"
                    } else {
                        ""
                    }
                ),
            ),
            None => make(
                Verdict::Constant,
                probe_cols.first().map(|c| c.name.as_str()),
                "the probe has fewer than two points with a value".to_owned(),
            ),
        };
    }
    // The harness has a probe for the finding and the series has no column for it.
    if let Some(p) = m.probes.iter().find(|p| p.finding.as_ref() == Some(f)) {
        // A reduced record may have carried that column: nothing says the code never loaded.
        if inp
            .series
            .limitations
            .iter()
            .any(|l| l.starts_with("reduced record"))
        {
            return make(
                Verdict::Unknown,
                None,
                format!(
                    "probe {} has no column, and the series has a reduced record (see its limitations): the column may have been lost with it, so whether the code loaded is not known: dla harness run {} again",
                    p.id, m.id
                ),
            );
        }
        let what = match &p.target {
            dla_core::harness::ProbeTarget::PyGlobal { module, name } => format!("{module}.{name}"),
            dla_core::harness::ProbeTarget::PyAttr { module, qualname } => {
                format!("{module}.{qualname}")
            }
            dla_core::harness::ProbeTarget::PyModule { module } => module.clone(),
            dla_core::harness::ProbeTarget::NodeModule { file } => file.clone(),
            dla_core::harness::ProbeTarget::NodeBinding { file, name } => format!("{file} {name}"),
            dla_core::harness::ProbeTarget::JvmStatic { class, field } => {
                // The agent found the class and it has no static field of that name (the
                // holder is an initialiser, `<clinit>`, or an instance field): the class
                // loaded, so the reading is the coverage's, not "never loaded".
                let said = format!("probe {class}.{field}: no such field");
                if inp.series.limitations.iter().any(|l| l == &said) {
                    let mut v = coverage_verdict(inp, f);
                    v.detail = format!(
                        "{class} loaded and has no static field {field}: the holder is not a static field, so its size is not measured; {}",
                        v.detail
                    );
                    return v;
                }
                format!("{class}.{field}")
            }
            dla_core::harness::ProbeTarget::JvmClass { class } => {
                // The agent says `class not loaded: <class>` (the class as the probe spells it)
                // for a class that is not loaded, and nothing for a class that is loaded but has
                // no static field of reference type: an instance-field holder. No such limitation
                // means the class loaded, so the code ran or did not by coverage, not by this probe.
                let said = format!("class not loaded: {class}");
                if !inp.series.limitations.iter().any(|l| l.ends_with(&said)) {
                    let mut v = coverage_verdict(inp, f);
                    v.detail = format!(
                        "the probe read no static field of {class}: the holder is not a static field, so its size is not measured; {}",
                        v.detail
                    );
                    return v;
                }
                class.clone()
            }
            dla_core::harness::ProbeTarget::GoVar { package_dir, name } => {
                format!("{package_dir}.{name}")
            }
            dla_core::harness::ProbeTarget::GoField {
                package_dir,
                type_name,
                field,
            } => {
                // A Go field probe is read from tracked objects: the series says why it
                // could not be (nothing tracked, or a field with no length).
                let path = format!("{package_dir}.{type_name}.{field}");
                let why = inp
                    .series
                    .limitations
                    .iter()
                    .find_map(|l| l.strip_prefix(&format!("probe {path}: ")))
                    .map_or_else(
                        || {
                            format!(
                                "no tracked object of type {type_name}: call dlaTrack(<your {type_name}>) in setup"
                            )
                        },
                        str::to_owned,
                    );
                return make(
                    Verdict::NotLoaded,
                    None,
                    format!(
                        "probe {} ({path}) has no column: its size was never read ({why}), so this target is not measured, only possibly executed",
                        p.id
                    ),
                );
            }
        };
        return make(
            Verdict::NotLoaded,
            None,
            format!(
                "probe {} ({what}) has no column: the workload never loaded that code, and a probe never imports it",
                p.id
            ),
        );
    }
    coverage_verdict(inp, f)
}

/// The function enclosing a site: its name (`Class.method`), start line and end line, and
/// the first line of its decorators.
struct Func {
    name: String,
    lo: u32,
    first: u32,
}

fn enclosing_function(project: &Path, site: &str) -> Option<(String, Func)> {
    let (file, line) = site.rsplit_once(':')?;
    let file = file.replace('\\', "/");
    let file = file.strip_prefix("./").unwrap_or(&file).to_owned();
    let line: usize = line.trim().parse().ok()?;
    let lang = Lang::from_path(Path::new(&file))?;
    let src = fs::read_to_string(project.join(&file)).ok()?;
    let tree = parse(lang, &src).ok()?;
    let row = line.checked_sub(1)?;
    src.lines().nth(row)?;
    let mut node = super::candidates::node_at_line(&tree, &src, row);
    let mut found: Option<Func> = None;
    while let Some(n) = node {
        let name_of = |n: dla_census::parse::tree_sitter::Node<'_>| {
            n.child_by_field_name("name")
                .and_then(|c| c.utf8_text(src.as_bytes()).ok())
                .map(str::to_owned)
        };
        if let Some(func) = found.as_mut() {
            if CLASSES.contains(&n.kind()) {
                if let Some(c) = name_of(n) {
                    func.name = format!("{c}.{}", func.name);
                }
                break;
            }
        } else if FUNCTIONS.contains(&n.kind()) {
            let lo = u32::try_from(n.start_position().row + 1).unwrap_or(u32::MAX);
            // A decorated Python function starts at its first decorator.
            let first = n
                .parent()
                .filter(|p| p.kind() == "decorated_definition")
                .map_or(lo, |p| {
                    u32::try_from(p.start_position().row + 1).unwrap_or(lo)
                });
            found = Some(Func {
                name: name_of(n).unwrap_or_default(),
                lo,
                first,
            });
        }
        node = n.parent();
    }
    found.filter(|f| !f.name.is_empty()).map(|f| (file, f))
}

/// `Unknown` with the reason.
fn unknown(f: &FindingId, why: &str) -> TargetVerdict {
    TargetVerdict {
        finding: f.clone(),
        verdict: Verdict::Unknown,
        column: None,
        detail: why.to_owned(),
        declared: false,
    }
}

fn coverage_verdict(inp: &Inputs<'_>, f: &FindingId) -> TargetVerdict {
    let none = |why: &str| unknown(f, why);
    let Some(h) = inp.series.source.harness.as_deref() else {
        return none("no coverage data for this run");
    };
    let Some(rec) = dla_core::harness::run_record_of(inp.run_dir, h, &inp.series.id) else {
        return none("no coverage data for this run");
    };
    let path = HarnessManifest::dir_in(inp.run_dir, h).join(format!("executed-{}.json", rec.first));
    let Some(doc) = fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    else {
        return none("no coverage data for this run");
    };
    let kind = doc.get("kind").and_then(Value::as_str).unwrap_or("none");
    if kind == "none" {
        let note = doc.get("note").and_then(Value::as_str).unwrap_or("");
        return none(&if note.is_empty() {
            "no coverage data for this run".to_owned()
        } else {
            format!("no coverage data for this run: {note}")
        });
    }
    let Some(finding) = inp.ledger.finding(f) else {
        return none("the finding is not in the ledger");
    };
    let mut sites: Vec<String> = finding.site.iter().cloned().collect();
    if let Some(a) = inp.ledger.annotation(f) {
        sites.extend(a.writers.iter().cloned());
    }
    if sites.is_empty() {
        return none("the finding has no site");
    }
    if kind == "classes" {
        return class_verdict(inp, f, &doc, &sites);
    }
    let covered: Vec<(&str, &str, u64)> = doc
        .get("functions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| {
            Some((
                v.get("file")?.as_str()?,
                v.get("name")?.as_str()?,
                v.get("line").and_then(Value::as_u64).unwrap_or(0),
            ))
        })
        .collect();
    let mut checked: Vec<(String, Func)> = Vec::new();
    for s in &sites {
        if let Some((file, func)) = enclosing_function(inp.project, s)
            && !checked.iter().any(|(f2, g)| *f2 == file && g.lo == func.lo)
        {
            checked.push((file, func));
        }
    }
    let Some((first_file, first_func)) = checked.first() else {
        return none("no function encloses the site or a writer of the finding");
    };
    let ran = |file: &str, func: &Func| {
        covered.iter().any(|(cf, name, line)| {
            (*cf == file || cf.ends_with(&format!("/{file}")))
                && ((*line >= u64::from(func.first) && *line <= u64::from(func.lo))
                    || *name == func.name
                    || name.ends_with(&format!(".{}", func.name)))
        })
    };
    let site_line = finding
        .site
        .as_deref()
        .and_then(|s| s.rsplit_once(':'))
        .map_or(first_func.lo.to_string(), |(_, l)| l.to_owned());
    if let Some((file, func)) = checked.iter().find(|(file, func)| ran(file, func)) {
        return TargetVerdict {
            finding: f.clone(),
            verdict: Verdict::Executed,
            column: None,
            detail: format!("function {} at {file}:{} was executed", func.name, func.lo),
            declared: false,
        };
    }
    let more = if checked.len() > 1 {
        format!(" (and {} writer function(s))", checked.len() - 1)
    } else {
        String::new()
    };
    TargetVerdict {
        finding: f.clone(),
        verdict: Verdict::NotExecuted,
        column: None,
        detail: format!(
            "function {} at {first_file}:{site_line} was not executed{more}",
            first_func.name
        ),
        declared: false,
    }
}

/// JVM: the class of the site or of a writer is among the loaded classes.
fn class_verdict(inp: &Inputs<'_>, f: &FindingId, doc: &Value, sites: &[String]) -> TargetVerdict {
    let loaded: Vec<&str> = doc
        .get("classes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let sources: Vec<&str> = doc
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut names: Vec<String> = Vec::new();
    for s in sites {
        let file = s.rsplit_once(':').map_or(s.as_str(), |(f, _)| f);
        let file = file.replace('\\', "/");
        let Some((stem, _)) = file.rsplit('/').next().and_then(|b| b.rsplit_once('.')) else {
            continue;
        };
        let package = fs::read_to_string(inp.project.join(&file))
            .ok()
            .and_then(|t| {
                t.lines()
                    .find_map(|l| l.trim().strip_prefix("package "))
                    .map(|p| p.trim().trim_end_matches(';').trim().to_owned())
            })
            .unwrap_or_default();
        let fqcn = if package.is_empty() {
            stem.to_owned()
        } else {
            format!("{package}.{stem}")
        };
        let dir = package.replace('.', "/");
        let base = file.rsplit('/').next().unwrap_or(&file);
        let by_source = sources.iter().any(|x| {
            x.rsplit('/').next() == Some(base) && x.rsplit_once('/').map_or("", |(d, _)| d) == dir
        });
        let by_class = loaded
            .iter()
            .map(|c| c.replace('/', "."))
            .any(|c| c == fqcn || c.starts_with(&format!("{fqcn}$")));
        if by_class || by_source {
            return TargetVerdict {
                finding: f.clone(),
                verdict: Verdict::Executed,
                column: None,
                detail: format!("class {fqcn} was loaded"),
                declared: false,
            };
        }
        if !names.contains(&fqcn) {
            names.push(fqcn);
        }
    }
    if names.is_empty() {
        return unknown(f, "no class could be derived from the site or a writer");
    }
    TargetVerdict {
        finding: f.clone(),
        verdict: Verdict::NotExecuted,
        column: None,
        detail: format!("class {} was not loaded", names.join(", ")),
        declared: false,
    }
}

/// Counts of verdicts, for the `dla trend` output beyond the declared targets.
pub fn counts(list: &[TargetVerdict]) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    for v in list {
        let e = m
            .entry(v.verdict.as_str().to_owned())
            .or_insert_with(|| Value::from(0u64));
        *e = Value::from(e.as_u64().unwrap_or(0) + 1);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trend_cmd::analysis::{
        analyze_series,
        testutil::{Col, rising, series},
    };
    use dla_core::{
        EvidenceId, Ledger, NewEvidence, NewFinding, Origin,
        evidence::{Authority, EvidenceCoverage, EvidenceKind, Source, Tool},
    };
    use dla_stats::GateThresholds;
    use serde_json::json;

    const PY: &str = "CACHE = {}\n\n\nclass Cart:\n    def add(self, k):\n        CACHE[k] = 1\n\n\ndef other():\n    return 1\n";

    struct Scene {
        dir: tempfile::TempDir,
        ledger: Ledger,
    }

    fn scene() -> Scene {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.py"), PY).unwrap();
        let mut ledger = Ledger::from_text("").unwrap();
        ledger
            .add_evidence(NewEvidence {
                source: Source::Repo {
                    commit: "c".into(),
                    path: None,
                },
                tool: Tool {
                    name: "t".into(),
                    version: "1".into(),
                },
                authority: Authority::Static,
                kind: EvidenceKind::CensusHolder,
                result: json!({}),
                raw: None,
                coverage: EvidenceCoverage::default(),
                limitations: vec![],
                unknowns: vec![],
            })
            .unwrap();
        // F1: the cache at line 1, written in Cart.add (line 6). F2: a site in `other`.
        for (site, writers) in [("a.py:1", vec!["a.py:6".to_owned()]), ("a.py:10", vec![])] {
            let mut nf =
                NewFinding::new(Origin::parse("census").unwrap(), vec![EvidenceId::new(1)]);
            nf.site = Some(site.into());
            let id = ledger.add_finding(nf).unwrap();
            if !writers.is_empty() {
                let f = ledger.findings().last().unwrap().id.clone();
                let _ = id;
                ledger
                    .annotate(&f, writers, vec![], Some("n".into()))
                    .unwrap();
            }
        }
        Scene { dir, ledger }
    }

    fn manifest(probes: serde_json::Value, targets: &[&str]) -> HarnessManifest {
        serde_json::from_value(json!({
            "id": "H1", "runtime": "python", "mode": "driver", "scratch": "x",
            "run": {"argv": ["x"], "cwd": "."}, "warmup": 1, "iterations": 8, "step": 1,
            "key_modes": ["fresh"], "probes": probes, "targets": targets,
            "agent": "driver-python", "timeout_secs": 60,
        }))
        .unwrap()
    }

    fn verdicts(
        s: &Scene,
        m: &HarnessManifest,
        cols: Vec<Col>,
        matcher_fill: impl FnMut(&mut Vec<ColumnTrend>),
    ) -> Vec<TargetVerdict> {
        verdicts_with(s, m, cols, matcher_fill, None)
    }

    fn verdicts_with(
        s: &Scene,
        m: &HarnessManifest,
        cols: Vec<Col>,
        matcher_fill: impl FnMut(&mut Vec<ColumnTrend>),
        too_short: Option<(usize, usize)>,
    ) -> Vec<TargetVerdict> {
        verdicts_lim(s, m, cols, matcher_fill, too_short, &[])
    }

    fn verdicts_lim(
        s: &Scene,
        m: &HarnessManifest,
        cols: Vec<Col>,
        mut matcher_fill: impl FnMut(&mut Vec<ColumnTrend>),
        too_short: Option<(usize, usize)>,
        limitations: &[&str],
    ) -> Vec<TargetVerdict> {
        m.save(s.dir.path()).unwrap();
        let mut file = series("S1", 1, cols);
        file.limitations = limitations.iter().map(|l| (*l).to_owned()).collect();
        let mut analysed = analyze_series(
            &file,
            &(0..file.columns.len()).collect::<Vec<_>>(),
            &GateThresholds::default(),
        );
        matcher_fill(&mut analysed);
        compute(&Inputs {
            project: s.dir.path(),
            run_dir: s.dir.path(),
            ledger: &s.ledger,
            series: &file,
            columns: &analysed,
            too_short,
        })
    }

    fn py_probe(id: &str, finding: &str, name: &str) -> serde_json::Value {
        json!({"id": id, "finding": finding, "target": {"kind": "py-global", "module": "a", "name": name}, "site": "a.py:1"})
    }

    fn tie(cols: &mut [ColumnTrend], f: u32) {
        for c in cols {
            c.finding = Some(FindingId::new(f));
        }
    }

    #[test]
    fn grew_when_a_tied_column_passed_the_gate() {
        let s = scene();
        let m = manifest(json!([py_probe("P1", "F1", "CACHE")]), &["F1"]);
        let v = verdicts(
            &s,
            &m,
            vec![Col::counter("probe:a.CACHE", &rising(1.0, 9.0))],
            |c| tie(c, 1),
        );
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].verdict, Verdict::Grew);
        assert_eq!(v[0].column.as_deref(), Some("probe:a.CACHE"));
        assert!(v[0].declared);
    }

    #[test]
    fn changed_when_the_probe_moved_but_did_not_pass() {
        let s = scene();
        let m = manifest(json!([py_probe("P1", "F1", "CACHE")]), &["F1"]);
        let wobble = [1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0];
        let v = verdicts(&s, &m, vec![Col::gauge("probe:a.CACHE", &wobble)], |c| {
            tie(c, 1)
        });
        assert_eq!(v[0].verdict, Verdict::Changed, "{v:?}");
        assert!(
            v[0].detail.contains("did not pass the growth gate"),
            "{}",
            v[0].detail
        );
    }

    #[test]
    fn a_flat_reading_of_a_test_loop_target_whose_test_class_was_dropped_is_a_miss() {
        let s = scene();
        let run = |junit: bool, classes: serde_json::Value, limitations: &[&str]| {
            let mut m = manifest(json!([py_probe("P1", "F1", "CACHE")]), &["F1"]);
            if junit {
                m.tools.push(dla_core::harness::JUNIT_TOOL.to_owned());
            }
            m.save(s.dir.path()).unwrap();
            fs::write(
                s.dir.path().join("harness").join("H1").join("tests.json"),
                json!({"classes": classes}).to_string(),
            )
            .unwrap();
            let mut file = series("S1", 1, vec![Col::gauge("probe:a.CACHE", &[1.0; 9])]);
            file.limitations = limitations.iter().map(|l| (*l).to_owned()).collect();
            let mut analysed = analyze_series(&file, &[0], &GateThresholds::default());
            tie(&mut analysed, 1);
            compute(&Inputs {
                project: s.dir.path(),
                run_dir: s.dir.path(),
                ledger: &s.ledger,
                series: &file,
                columns: &analysed,
                too_short: None,
            })
            .remove(0)
        };
        let one = json!([{"class": "a.CartTests", "for": [{"finding": "F1", "by": "name"}]}]);
        let two = json!([
            {"class": "a.CartTests", "for": [{"finding": "F1", "by": "name"}]},
            {"class": "a.CartUnitTests", "for": [{"finding": "F1", "by": "name"}]},
        ]);
        let dropped = "dropped test class a.CartTests: none of its tests passed in the first run (java.lang.IllegalStateException), so it is left out of the loop and the other classes are measured";
        // Its only test class was dropped: the flat reading is a miss, and the detail says why.
        let v = run(true, one.clone(), &[dropped]);
        assert_eq!(v.verdict, Verdict::NotExecuted, "{v:?}");
        assert!(
            v.detail.contains("a.CartTests") && v.detail.contains("dropped from the loop"),
            "{}",
            v.detail
        );
        // Nothing dropped, another of its classes still ran, or another class was dropped: the
        // reading stands.
        assert_eq!(run(true, one.clone(), &[]).verdict, Verdict::Constant);
        assert_eq!(run(true, two, &[dropped]).verdict, Verdict::Constant);
        assert_eq!(
            run(
                true,
                one.clone(),
                &["dropped test class a.OtherTests: it was 3.0 s of the 4.0 s a run of the tests takes"]
            )
            .verdict,
            Verdict::Constant
        );
        // A harness that is not a test loop has no chosen tests.
        assert_eq!(run(false, one, &[dropped]).verdict, Verdict::Constant);
    }

    #[test]
    fn constant_when_the_probe_never_moved() {
        let s = scene();
        let m = manifest(json!([py_probe("P1", "F1", "CACHE")]), &["F1"]);
        let v = verdicts(&s, &m, vec![Col::gauge("probe:a.CACHE", &[1.0; 9])], |c| {
            tie(c, 1)
        });
        assert_eq!(v[0].verdict, Verdict::Constant, "{v:?}");
        assert!(
            v[0].detail.contains(
                "stayed 1 over 8 points: every unit used the same key or never reached a writer"
            ),
            "{}",
            v[0].detail
        );
    }

    #[test]
    fn a_constant_probe_of_a_key_the_caller_passes_says_the_unit_may_have_passed_the_same_objects()
    {
        use dla_core::census::{HolderFacts, SiteFact, SiteKind};
        let mut s = scene();
        let fact = |param: bool| SiteFact {
            site: "a.py:6".into(),
            kind: SiteKind::Insert,
            depth: 0,
            function: Some("add".into()),
            init: false,
            startup: false,
            param,
            constant: false,
            exported: true,
        };
        let m = manifest(json!([py_probe("P1", "F1", "CACHE")]), &["F1"]);
        let flat = [1.0; 9];
        // No fact that the key is a parameter: the sentence of before.
        let v = verdicts(&s, &m, vec![Col::gauge("probe:a.CACHE", &flat)], |c| {
            tie(c, 1)
        });
        assert!(
            !v[0].detail.contains("comes from the caller"),
            "{}",
            v[0].detail
        );
        // The census says the key derives from a parameter: one more sentence.
        s.ledger
            .annotate_record(dla_core::FindingAnnotation {
                writers: vec!["a.py:6".into()],
                facts: HolderFacts {
                    writer_sites: vec![fact(true)],
                    ..HolderFacts::default()
                },
                ..dla_core::FindingAnnotation::new(FindingId::new(1))
            })
            .unwrap();
        let v = verdicts(&s, &m, vec![Col::gauge("probe:a.CACHE", &flat)], |c| {
            tie(c, 1)
        });
        assert_eq!(v[0].verdict, Verdict::Constant);
        assert!(
            v[0].detail.contains("stayed 1 over 8 points: every unit used the same key or never reached a writer; the key of this holder comes from the caller, so constant can also mean the unit passed the same keys every time"),
            "{}",
            v[0].detail
        );
    }

    #[test]
    fn not_loaded_when_the_probe_has_no_column() {
        let s = scene();
        let m = manifest(json!([py_probe("P1", "F1", "CACHE")]), &["F1"]);
        let v = verdicts(&s, &m, vec![Col::gauge("os.rss", &[5.0; 9])], |_| {});
        assert_eq!(v[0].verdict, Verdict::NotLoaded, "{v:?}");
        assert!(v[0].detail.contains("a.CACHE"), "{}", v[0].detail);
    }

    fn write_run(s: &Scene, doc: serde_json::Value) {
        let h = s.dir.path().join("harness/H1");
        fs::create_dir_all(&h).unwrap();
        fs::write(h.join("run-S1.json"), json!({"series": ["S1"]}).to_string()).unwrap();
        fs::write(h.join("executed-S1.json"), doc.to_string()).unwrap();
    }

    #[test]
    fn executed_when_the_function_of_a_writer_ran() {
        let s = scene();
        // The writer at a.py:6 is Cart.add (starts at line 5).
        write_run(
            &s,
            json!({"kind": "functions", "functions": [{"file": "a.py", "name": "Cart.add", "line": 5}]}),
        );
        let m = manifest(json!([]), &["F1"]);
        let v = verdicts(&s, &m, vec![Col::gauge("os.rss", &[5.0; 9])], |_| {});
        assert_eq!(v[0].verdict, Verdict::Executed, "{v:?}");
        assert!(v[0].detail.contains("Cart.add"), "{}", v[0].detail);
    }

    #[test]
    fn a_short_series_never_settles_a_target_as_executed() {
        let s = scene();
        write_run(
            &s,
            json!({"kind": "functions", "functions": [{"file": "a.py", "name": "Cart.add", "line": 5}]}),
        );
        let m = manifest(json!([]), &["F1"]);
        let v = verdicts_with(
            &s,
            &m,
            vec![Col::gauge("os.rss", &[5.0; 4])],
            |_| {},
            Some((4, 8)),
        );
        assert_eq!(v[0].verdict, Verdict::Unknown, "{v:?}");
        assert_eq!(v[0].detail, "series too short: 4 points, 8 needed");
    }

    #[test]
    fn the_function_around_the_last_line_is_found_with_a_trailing_comment_too() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("a.py"),
            "class Cart:\n    def add(self, k, v):\n        self.n = 1\n        self.items[k] = v  # last\n",
        )
        .unwrap();
        for line in [3, 4] {
            let (file, func) = enclosing_function(dir.path(), &format!("a.py:{line}")).unwrap();
            assert_eq!(
                (file.as_str(), func.name.as_str(), func.lo),
                ("a.py", "Cart.add", 2)
            );
        }
    }

    #[test]
    fn not_executed_when_no_enclosing_function_ran() {
        let s = scene();
        write_run(
            &s,
            json!({"kind": "functions", "functions": [{"file": "a.py", "name": "other", "line": 9}]}),
        );
        let m = manifest(json!([]), &["F1"]);
        let v = verdicts(&s, &m, vec![Col::gauge("os.rss", &[5.0; 9])], |_| {});
        assert_eq!(v[0].verdict, Verdict::NotExecuted, "{v:?}");
        assert!(v[0].detail.contains("was not executed"), "{}", v[0].detail);
    }

    #[test]
    fn unknown_without_coverage_data() {
        let s = scene();
        // No coverage record at all (a Go driver run writes no profile).
        let m = manifest(json!([]), &["F1"]);
        let v = verdicts(&s, &m, vec![Col::gauge("os.rss", &[5.0; 9])], |_| {});
        assert_eq!(v[0].verdict, Verdict::Unknown, "{v:?}");
        assert!(
            v[0].detail.contains("no coverage data for this run"),
            "{}",
            v[0].detail
        );
        // A record that says nothing could be measured is unknown too, never not_executed.
        write_run(
            &s,
            json!({"kind": "none", "note": "no Go coverage profile was written to cov/"}),
        );
        let v = verdicts(&s, &m, vec![Col::gauge("os.rss", &[5.0; 9])], |_| {});
        assert_eq!(v[0].verdict, Verdict::Unknown, "{v:?}");
        assert!(
            v[0].detail.contains("no coverage data for this run"),
            "{}",
            v[0].detail
        );
    }

    #[test]
    fn unknown_when_no_function_encloses_the_site() {
        let s = scene();
        // F1's site is the module-level dict but it has a writer; F2's site is in `other`.
        // Drop the writer by asking about a finding whose site is top level and unwritten.
        let mut l = s.ledger.clone();
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![EvidenceId::new(1)]);
        nf.site = Some("a.py:1".into());
        let _ = l.add_finding(nf);
        let s2 = Scene {
            dir: s.dir,
            ledger: l,
        };
        write_run(&s2, json!({"kind": "functions", "functions": []}));
        let m = manifest(json!([]), &["F3"]);
        let v = verdicts(&s2, &m, vec![Col::gauge("os.rss", &[5.0; 9])], |_| {});
        assert_eq!(v[0].verdict, Verdict::Unknown, "{v:?}");
    }

    #[test]
    fn a_class_loaded_is_executed_for_a_jvm_run() {
        let s = scene();
        fs::write(
            s.dir.path().join("Cart.java"),
            "package shop;\nclass Cart {}\n",
        )
        .unwrap();
        let mut l = s.ledger.clone();
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![EvidenceId::new(1)]);
        nf.site = Some("Cart.java:2".into());
        l.add_finding(nf).unwrap();
        let s2 = Scene {
            dir: s.dir,
            ledger: l,
        };
        let m = manifest(json!([]), &["F3"]);
        write_run(
            &s2,
            json!({"kind": "classes", "classes": ["shop.Other"], "sources": []}),
        );
        let v = verdicts(&s2, &m, vec![Col::gauge("os.rss", &[5.0; 9])], |_| {});
        assert_eq!(v[0].verdict, Verdict::NotExecuted, "{v:?}");
        write_run(
            &s2,
            json!({"kind": "classes", "classes": ["shop.Cart$Inner"], "sources": []}),
        );
        let v = verdicts(&s2, &m, vec![Col::gauge("os.rss", &[5.0; 9])], |_| {});
        assert_eq!(v[0].verdict, Verdict::Executed, "{v:?}");
    }

    /// A JVM scene: finding F3 at `Cart.java` (package `shop`), a `jvm-class` probe for
    /// `shop.Cart`, and the coverage record of the run.
    fn jvm_scene(classes: serde_json::Value) -> (Scene, HarnessManifest) {
        let s = scene();
        fs::write(
            s.dir.path().join("Cart.java"),
            "package shop;\nclass Cart {}\n",
        )
        .unwrap();
        let mut l = s.ledger.clone();
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![EvidenceId::new(1)]);
        nf.site = Some("Cart.java:2".into());
        l.add_finding(nf).unwrap();
        let s2 = Scene {
            dir: s.dir,
            ledger: l,
        };
        write_run(
            &s2,
            json!({"kind": "classes", "classes": classes, "sources": []}),
        );
        let m = manifest(
            json!([{"id": "P1", "finding": "F3", "site": "Cart.java:2",
                    "target": {"kind": "jvm-class", "class": "shop.Cart"}}]),
            &["F3"],
        );
        (s2, m)
    }

    #[test]
    fn a_jvm_class_probe_with_no_column_and_no_unloaded_limitation_gives_the_coverage_verdict() {
        let cols = || vec![Col::gauge("os.rss", &[5.0; 9])];
        // The class loaded (the coverage lists it) and the probe had no static field to read.
        let (s, m) = jvm_scene(json!(["shop.Cart"]));
        let v = verdicts_lim(&s, &m, cols(), |_| {}, None, &[]);
        assert_eq!(v[0].verdict, Verdict::Executed, "{v:?}");
        assert!(
            v[0].detail.starts_with(
                "the probe read no static field of shop.Cart: the holder is not a static field, so its size is not measured; "
            ),
            "{}",
            v[0].detail
        );
        assert!(
            v[0].detail.contains("class shop.Cart was loaded"),
            "{}",
            v[0].detail
        );
        // Another limitation, about another class, does not say this one is not loaded.
        let other = "probe shop.Cartel: class not loaded: shop.Cartel";
        let v = verdicts_lim(&s, &m, cols(), |_| {}, None, &[other]);
        assert_eq!(v[0].verdict, Verdict::Executed, "{v:?}");
        // The class did not load by coverage either: not_executed, still not not_loaded.
        let (s, m) = jvm_scene(json!(["shop.Other"]));
        let v = verdicts_lim(&s, &m, cols(), |_| {}, None, &[]);
        assert_eq!(v[0].verdict, Verdict::NotExecuted, "{v:?}");
        assert!(
            v[0].detail
                .starts_with("the probe read no static field of shop.Cart"),
            "{}",
            v[0].detail
        );
    }

    #[test]
    fn a_jvm_class_probe_whose_class_the_agent_says_is_not_loaded_stays_not_loaded() {
        let (s, m) = jvm_scene(json!(["shop.Cart"]));
        let said = "probe shop.Cart: class not loaded: shop.Cart";
        let v = verdicts_lim(
            &s,
            &m,
            vec![Col::gauge("os.rss", &[5.0; 9])],
            |_| {},
            None,
            &[said],
        );
        assert_eq!(v[0].verdict, Verdict::NotLoaded, "{v:?}");
        assert!(v[0].detail.contains("shop.Cart"), "{}", v[0].detail);
    }

    #[test]
    fn a_jvm_static_probe_of_a_field_the_class_does_not_have_gives_the_coverage_verdict() {
        // The holder's owner is the class initialiser: the class loaded, and it has no static
        // field `<clinit>` to read.
        let (s, _) = jvm_scene(json!(["shop.Cart"]));
        let m = manifest(
            json!([{"id": "P1", "finding": "F3", "site": "Cart.java:2",
                    "target": {"kind": "jvm-static", "class": "shop.Cart", "field": "<clinit>"}}]),
            &["F3"],
        );
        let cols = || vec![Col::gauge("os.rss", &[5.0; 9])];
        let v = verdicts_lim(
            &s,
            &m,
            cols(),
            |_| {},
            None,
            &["probe shop.Cart.<clinit>: no such field"],
        );
        assert_eq!(v[0].verdict, Verdict::Executed, "{v:?}");
        assert!(
            v[0].detail.starts_with(
                "shop.Cart loaded and has no static field <clinit>: the holder is not a static field, so its size is not measured; "
            ),
            "{}",
            v[0].detail
        );
        // Without that statement of the agent nothing says the class loaded.
        let v = verdicts_lim(&s, &m, cols(), |_| {}, None, &[]);
        assert_eq!(v[0].verdict, Verdict::NotLoaded, "{v:?}");
    }
}
