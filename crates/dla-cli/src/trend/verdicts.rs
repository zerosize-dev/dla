//! The verdict of `dla trend` for every targeted finding of a harness series.
//!
//! The targets are the findings the series' harness declared (`--probe`, `--next`) and every
//! finding an automatic probe of the harness stands for. Per finding, the first that applies:
//! `grew`, `changed`, `constant`, `not_loaded`, then (no probe) `executed` or `not_executed`
//! from the run's coverage, else `unknown`.

use std::{fs, path::Path};

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
    ids.into_iter()
        .filter(|(f, _)| inp.ledger.finding(f).is_some())
        .map(|(f, declared)| {
            let mut v = one(inp, &m, &f);
            v.declared = declared;
            v
        })
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
            format!("column {} passed the growth gate ({:?} shape)", c.name, c.shape),
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
            let (lo, hi) = v.iter().fold((f64::MAX, f64::MIN), |(l, h), x| (l.min(*x), h.max(*x)));
            if lo != hi {
                return make(
                    Verdict::Changed,
                    Some(&c.name),
                    format!(
                        "probe {} went from {} to {} over {} points but did not pass the growth gate",
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
                    "probe {name} stayed {} over {n} points: every unit used the same key or never reached a writer",
                    num(value)
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
        let what = match &p.target {
            dla_core::harness::ProbeTarget::PyGlobal { module, name } => format!("{module}.{name}"),
            dla_core::harness::ProbeTarget::PyAttr { module, qualname } => {
                format!("{module}.{qualname}")
            }
            dla_core::harness::ProbeTarget::PyModule { module } => module.clone(),
            dla_core::harness::ProbeTarget::NodeModule { file } => file.clone(),
            dla_core::harness::ProbeTarget::NodeBinding { file, name } => format!("{file} {name}"),
            dla_core::harness::ProbeTarget::JvmStatic { class, field } => format!("{class}.{field}"),
            dla_core::harness::ProbeTarget::JvmClass { class } => class.clone(),
            dla_core::harness::ProbeTarget::GoVar { package_dir, name } => {
                format!("{package_dir}.{name}")
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
    let indent = src
        .lines()
        .nth(row)
        .map_or(0, |l| l.len() - l.trim_start().len());
    let mut node = tree.root_node().descendant_for_point_range(
        dla_census::parse::tree_sitter::Point { row, column: indent },
        dla_census::parse::tree_sitter::Point {
            row,
            column: usize::MAX / 2,
        },
    );
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
            detail: format!(
                "function {} at {file}:{} was executed",
                func.name, func.lo
            ),
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
            x.rsplit('/').next() == Some(base)
                && x.rsplit_once('/').map_or("", |(d, _)| d) == dir
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
