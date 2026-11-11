//! `dla-bench calib-extract`: reads the log directory of a `dla-bench dynamic` run and writes
//! the per-fixture, per-column statistics the calibration of the gate thresholds needs.
//!
//! The log directory holds one directory per fixture with the files the run collected:
//! `trend/T*.json`, `alloc/A*.json`, `series/S*.json`, `harness/H*/harness.json` and
//! `ledger.jsonl`. Nothing is recomputed from raw points except what the files do not
//! carry: the span of a series and, for a residual row, the noise of its primary column.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use serde_json::{Map, Value, json};

use crate::{
    Res,
    fixture::load_expected,
    model::{Category, Expected},
};

/// Flags of `dla-bench calib-extract`.
#[derive(Args, Debug)]
pub struct CalibArgs {
    /// The `--log-dir` of a `dla-bench dynamic` run.
    pub log_dir: PathBuf,
    /// The JSON file to write.
    #[arg(long)]
    pub out: PathBuf,
    /// Directory with the fixtures (for the expected outcome of each column).
    #[arg(long, default_value = "bench/fixtures")]
    pub fixtures: PathBuf,
    /// A `key=value` pair copied into the file's `meta` (the trial number, the `--iterations`
    /// value of the run, ...). Repeatable.
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    pub meta: Vec<String>,
}

fn read_json(p: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(p).ok()?).ok()
}

/// `T1.json`, `T2.json`, ... in numeric order.
fn numbered_files(dir: &Path, prefix: char) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut v: Vec<(u64, PathBuf)> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let n = name
                .strip_prefix(prefix)?
                .strip_suffix(".json")?
                .parse::<u64>()
                .ok()?;
            Some((n, e.path()))
        })
        .collect();
    v.sort();
    v.into_iter().map(|(_, p)| p).collect()
}

fn num(v: &Value, key: &str) -> Value {
    v.get(key)
        .filter(|x| x.is_number())
        .cloned()
        .unwrap_or(Value::Null)
}

/// What the fixture's `expected.toml` says about the gate of a column: `"fail"` for every
/// column of a fixture without a leak to confirm, `"pass"` for a column tied to a finding
/// at the site of an expected leak, `null` when it says nothing.
fn expected_gate(exp: Option<&Expected>, tied: bool) -> Value {
    match exp.map(|e| e.category) {
        Some(Category::DecoyControl | Category::NonLeak | Category::Impossible) => json!("fail"),
        Some(Category::ArchetypeLeak | Category::ArchetypeFree | Category::MultiLeak) if tied => {
            json!("pass")
        }
        _ => Value::Null,
    }
}

/// The finding ids of the ledger, worker prefix included, whose site matches an expected leak.
fn leak_findings(ledger: &[Value], exp: Option<&Expected>) -> Vec<String> {
    let Some(exp) = exp else {
        return Vec::new();
    };
    ledger
        .iter()
        .filter(|r| r["type"] == "finding_added")
        .filter_map(|r| {
            let f = &r["finding"];
            let id = f["id"].as_str()?;
            let finding = crate::grade::Finding {
                state: crate::grade::State::Open,
                site: f["site"].as_str().map(str::to_owned),
                owner: f["owner"].as_str().map(str::to_owned),
                worker: None,
                holder: None,
                label: None,
                hypothesis: f["parent"].as_str().is_some_and(|p| !p.is_empty()),
                discriminator: false,
            };
            let hit = exp
                .leak
                .iter()
                .any(|l| l.confirmable && crate::grade::site_matches(&finding, l));
            hit.then(|| id.to_owned())
        })
        .collect()
}

fn harness_info(fx: &Path, id: &str) -> Value {
    let Some(h) = read_json(&fx.join("harness").join(id).join("harness.json")) else {
        return Value::Null;
    };
    json!({
        "id": id,
        "workload": h.get("workload").cloned().unwrap_or(Value::Null),
        "mode": h.get("mode").cloned().unwrap_or(Value::Null),
        "iterations": num(&h, "iterations"),
        "warmup": num(&h, "warmup"),
        "step": num(&h, "step"),
    })
}

fn trend_rows(
    fx_id: &str,
    fx: &Path,
    exp: Option<&Expected>,
    leak_ids: &[String],
) -> (Vec<Value>, BTreeMap<String, Value>) {
    let mut rows = Vec::new();
    let mut trends = BTreeMap::new();
    for p in numbered_files(&fx.join("trend"), 'T') {
        let Some(t) = read_json(&p) else { continue };
        let series = t["series"].as_str().unwrap_or("").to_owned();
        let sfile = read_json(&fx.join("series").join(format!("{series}.json")));
        let harness = sfile
            .as_ref()
            .and_then(|s| s["source"]["harness"].as_str())
            .map(|h| harness_info(fx, h))
            .unwrap_or(Value::Null);
        let th = &t["thresholds"];
        let tid = t["id"].as_str().unwrap_or("").to_owned();
        let mut cols = Vec::new();
        for c in t["columns"].as_array().into_iter().flatten() {
            let cands: Vec<&str> = c["candidates"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            // Exact identities: `jvm/F1` is not `go/F1`. The column's own `finding` counts too.
            let tied = cands
                .iter()
                .copied()
                .chain(c["finding"].as_str())
                .any(|id| leak_ids.iter().any(|l| l == id));
            let noise = c["noise"].as_f64();
            let rise = c["rise"].as_f64();
            let rise_over_noise = match (rise, noise) {
                (Some(r), Some(n)) if n > 0.0 => json!(r / n),
                _ => Value::Null,
            };
            let col = json!({
                "fixture": fx_id,
                "trend": tid,
                "series": series,
                "name": c["name"],
                "unit": c["unit"],
                "kind": c["kind"],
                "region": c["region"],
                "candidates": cands,
                "tied": tied,
                "n": num(c, "n"),
                "p_increasing": num(&c["mk"], "p_increasing"),
                "tau": num(&c["mk"], "tau"),
                "slope": num(&c["sen"], "slope"),
                "sen_lo": num(&c["sen"], "lo"),
                "sen_hi": num(&c["sen"], "hi"),
                "half_first": num(&c["halves"], "first"),
                "half_second": num(&c["halves"], "second"),
                "half_ratio": num(&c["halves"], "ratio"),
                "noise": num(c, "noise"),
                "rise": num(c, "rise"),
                "rise_over_noise": rise_over_noise,
                "shape": c["shape"],
                "gate": c["gate"],
                "gate_failed": c["gate_failed"],
                "expected_gate": expected_gate(exp, tied),
            });
            cols.push(col);
        }
        trends.insert(
            tid.clone(),
            json!({"series": series, "thresholds": th, "harness": harness}),
        );
        rows.extend(cols);
    }
    (rows, trends)
}

fn alloc_rows(fx_id: &str, fx: &Path, exp: Option<&Expected>) -> Vec<Value> {
    let mut out = Vec::new();
    for p in numbered_files(&fx.join("alloc"), 'A') {
        let Some(a) = read_json(&p) else { continue };
        let cols: Vec<Value> = a["columns"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|c| {
                json!({
                    "name": c["name"],
                    "favors": c["favors"],
                    "supports": c["supports"],
                    "voting": c["voting"],
                    "main_slope": num(&c["plateau"], "main_slope"),
                    "extra_slope": num(&c["plateau"], "extra_slope"),
                    "extra_rise": num(&c["plateau"], "extra_rise"),
                    "noise": num(&c["plateau"], "noise"),
                    "plateau_verdict": c["plateau"]["verdict"],
                    "purge_kind": c["purge"]["kind"],
                    "purge_before": num(&c["purge"], "before"),
                    "purge_after": num(&c["purge"], "after"),
                    "purge_drop": num(&c["purge"], "drop"),
                    "purge_growth": num(&c["purge"], "growth"),
                    "purge_verdict": c["purge"]["verdict"],
                    "expected_favors": match exp.map(|e| e.category) {
                        Some(Category::DecoyControl | Category::NonLeak) => json!("non-leak"),
                        _ => Value::Null,
                    },
                })
            })
            .collect();
        out.push(json!({
            "fixture": fx_id,
            "id": a["id"],
            "finding": a["finding"],
            "harness": a["harness"],
            "iterations": num(&a, "iterations"),
            "extra": num(&a, "extra"),
            "favors": a["favors"],
            "columns": cols,
        }));
    }
    out
}

/// Columns the residual totals come from, in the order of preference, per region.
const PRIMARY: [(&str, &[&str]); 3] = [
    (
        "heap",
        &[
            "jvm.heap_used",
            "go.heap_live",
            "node.heap_used",
            "py.traced",
            "native.live_bytes",
        ],
    ),
    ("non-heap", &["jvm.nonheap_used", "go.stacks"]),
    ("off-heap", &["gap.unexplained", "node.external"]),
];

/// The post-warm-up `(x, value)` points of column `name` of a series file; empty when the
/// series lacks the column.
fn points_of(series: &Value, name: &str) -> Vec<(f64, f64)> {
    let Some(idx) = series["columns"]
        .as_array()
        .and_then(|c| c.iter().position(|c| c["name"] == name))
    else {
        return Vec::new();
    };
    let warm = series["warmup"].as_u64().unwrap_or(0) as usize;
    series["points"]
        .as_array()
        .into_iter()
        .flatten()
        .skip(warm)
        .filter_map(|p| Some((p["x"].as_f64()?, p["values"].get(idx)?.as_f64()?)))
        .collect()
}

/// Distance between the first and the last post-warm-up x of column `name` of a series.
fn span_of(series: &Value, name: &str) -> Option<f64> {
    let pts = points_of(series, name);
    (pts.len() >= 3).then(|| pts[pts.len() - 1].0 - pts[0].0)
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_unstable_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// The robust residual scale `dla gap --residual` uses as the noise of a column: 1.4826 x the
/// median absolute deviation of the residuals around the Theil-Sen line (median of the pairwise
/// slopes; intercept the median of `y - slope x`). `None` with fewer than three points or when
/// every x is equal.
pub fn fit_noise(pts: &[(f64, f64)]) -> Option<f64> {
    if pts.len() < 3 || pts.iter().any(|&(x, y)| !x.is_finite() || !y.is_finite()) {
        return None;
    }
    let mut slopes = Vec::new();
    for (i, a) in pts.iter().enumerate() {
        for b in &pts[i + 1..] {
            let dx = b.0 - a.0;
            if dx != 0.0 {
                slopes.push((b.1 - a.1) / dx);
            }
        }
    }
    if slopes.is_empty() {
        return None;
    }
    let slope = median(&mut slopes);
    let mut r: Vec<f64> = pts.iter().map(|&(x, y)| y - slope * x).collect();
    let intercept = median(&mut r);
    let mut res: Vec<f64> = pts
        .iter()
        .map(|&(x, y)| y - (intercept + slope * x))
        .collect();
    let med = median(&mut res);
    let mut dev: Vec<f64> = res.iter().map(|r| (r - med).abs()).collect();
    Some(1.4826 * median(&mut dev))
}

/// The newest gap series of the fixture derived from `series` (the one `dla gap --residual`
/// fits for the off-heap row): the evidence's own `derived` id when it names one, else the
/// highest-numbered series with `source.derived_from == series`.
fn derived_gap_series(fx: &Path, series: &str, evidence: &Value) -> Option<Value> {
    if let Some(d) = evidence["result"]["derived"].as_str()
        && let Some(v) = read_json(&fx.join("series").join(format!("{d}.json")))
    {
        return Some(v);
    }
    numbered_files(&fx.join("series"), 'S')
        .iter()
        .rev()
        .filter_map(|p| read_json(p))
        .find(|v| v["source"]["kind"] == "gap" && v["source"]["derived_from"] == series)
}

/// `series -> column -> noise` from the first trend of each series.
type NoiseBySeries = BTreeMap<String, Map<String, Value>>;

fn residual_rows(
    fx_id: &str,
    fx: &Path,
    ledger: &[Value],
    noise_by_series: &NoiseBySeries,
) -> Vec<Value> {
    let mut out = Vec::new();
    for r in ledger {
        if r["type"] != "evidence" || r["kind"] != "residual" {
            continue;
        }
        let series = r["result"]["series"].as_str().unwrap_or("");
        let sfile = read_json(&fx.join("series").join(format!("{series}.json")));
        for row in r["result"]["rows"].as_array().into_iter().flatten() {
            let region = row["region"].as_str().unwrap_or("");
            let names = PRIMARY
                .iter()
                .find(|(r, _)| *r == region)
                .map_or(&[][..], |(_, n)| *n);
            // The off-heap total is fitted on the derived gap series when there is one; its
            // noise and span come from that series. Otherwise, and for the other regions, the
            // primary column is the first preferred one the series has with a span, and its
            // noise is the trend's.
            let gap = (region == "off-heap")
                .then(|| derived_gap_series(fx, series, r))
                .flatten()
                .and_then(|d| {
                    let pts = points_of(&d, "gap.unexplained");
                    Some((span_of(&d, "gap.unexplained")?, fit_noise(&pts)?))
                });
            let (column, span, noise) = if let Some((span, noise)) = gap {
                (json!("gap.unexplained"), json!(span), json!(noise))
            } else {
                let found = sfile.as_ref().and_then(|s| {
                    names
                        .iter()
                        .find_map(|n| span_of(s, n).map(|sp| ((*n).to_owned(), sp)))
                });
                match found {
                    Some((c, sp)) => {
                        let noise = noise_by_series
                            .get(series)
                            .and_then(|m| m.get(&c).cloned())
                            .unwrap_or(Value::Null);
                        (json!(c), json!(sp), noise)
                    }
                    None => (Value::Null, Value::Null, Value::Null),
                }
            };
            let total = row["total"].as_f64();
            let residual = row["residual"].as_f64();
            let threshold = row["threshold"].as_f64();
            let share = match (residual, total) {
                (Some(r), Some(t)) if t > 0.0 => json!(r / t),
                _ => Value::Null,
            };
            let over = match (residual, threshold) {
                (Some(r), Some(t)) => json!(r > 0.0 && r > t),
                _ => Value::Null,
            };
            // The noise term of the threshold, 3 x noise / span, when both are known.
            let noise_floor = match (noise.as_f64(), span.as_f64()) {
                (Some(n), Some(s)) if s > 0.0 => json!(3.0 * n / s),
                _ => Value::Null,
            };
            out.push(json!({
                "fixture": fx_id,
                "evidence": r["evidence_id"],
                "series": series,
                "region": region,
                "column": column,
                "unit": row["unit"],
                "total": num(row, "total"),
                "attributed": num(row, "attributed"),
                "residual": num(row, "residual"),
                "threshold": num(row, "threshold"),
                "share": share,
                "over": over,
                "open_item": row["open_item"],
                "noise": noise,
                "span": span,
                "noise_floor": noise_floor,
            }));
        }
    }
    out
}

fn read_ledger(fx: &Path) -> Vec<Value> {
    fs::read(fx.join("ledger.jsonl"))
        .map(|b| {
            b.split(|c| *c == b'\n')
                .filter_map(|l| serde_json::from_slice::<Value>(l).ok())
                .filter(Value::is_object)
                .collect()
        })
        .unwrap_or_default()
}

/// Builds the extraction of a log directory.
pub fn extract(log_dir: &Path, fixtures: &Path, meta: &[(String, String)]) -> Res<Value> {
    let rd = fs::read_dir(log_dir).map_err(|e| format!("{}: {e}", log_dir.display()))?;
    let mut ids: Vec<(String, PathBuf)> = rd
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    ids.sort();
    let mut out_fx = Map::new();
    for (id, dir) in ids {
        let has_data = ["trend", "alloc", "ledger.jsonl"]
            .iter()
            .any(|n| dir.join(n).exists());
        if !has_data {
            continue;
        }
        let exp = load_expected(fixtures, &id).ok();
        let ledger = read_ledger(&dir);
        let leak_ids = leak_findings(&ledger, exp.as_ref());
        let (columns, trends) = trend_rows(&id, &dir, exp.as_ref(), &leak_ids);
        // Noise by column of each series, for the residual rows.
        let mut by_series = NoiseBySeries::new();
        for p in numbered_files(&dir.join("trend"), 'T') {
            let Some(t) = read_json(&p) else { continue };
            let series = t["series"].as_str().unwrap_or("").to_owned();
            let by_col: Map<String, Value> = t["columns"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|c| Some((c["name"].as_str()?.to_owned(), c["noise"].clone())))
                .collect();
            by_series.entry(series).or_insert(by_col);
        }
        let expected = exp.as_ref().map(|e| {
            json!({
                "category": e.category.as_str(),
                "pair": e.pair,
                "dynamic_confirmation": e.dynamic_confirmation,
                "leaks": e.leak.iter().map(|l| json!({
                    "id": l.id, "site": l.site, "owner": l.owner, "cause": l.cause,
                    "holder": l.holder.as_str(), "confirmable": l.confirmable,
                })).collect::<Vec<_>>(),
                "non_leaks": e.non_leak.iter().map(|n| json!({
                    "id": n.id, "cause": n.cause,
                })).collect::<Vec<_>>(),
            })
        });
        let residual = residual_rows(&id, &dir, &ledger, &by_series);
        let alloc = alloc_rows(&id, &dir, exp.as_ref());
        out_fx.insert(
            id,
            json!({
                "expected": expected,
                "trends": trends,
                "columns": columns,
                "alloc": alloc,
                "residual": residual,
            }),
        );
    }
    let mut m = Map::new();
    for (k, v) in meta {
        m.insert(k.clone(), json!(v));
    }
    Ok(json!({
        "schema_version": 1,
        "log_dir": log_dir.display().to_string(),
        "meta": m,
        "fixtures": out_fx,
    }))
}

/// Runs the subcommand; the message to print.
pub fn run(a: &CalibArgs) -> Res<String> {
    let meta: Vec<(String, String)> = a
        .meta
        .iter()
        .map(|kv| {
            kv.split_once('=')
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .ok_or_else(|| format!("--meta {kv:?} is not KEY=VALUE"))
        })
        .collect::<Res<_>>()?;
    let doc = extract(&a.log_dir, &a.fixtures, &meta)?;
    let n = doc["fixtures"].as_object().map_or(0, Map::len);
    if n == 0 {
        return Err(format!(
            "{}: no fixture directory with trend, alloc or ledger files",
            a.log_dir.display()
        ));
    }
    let cols: usize = doc["fixtures"]
        .as_object()
        .into_iter()
        .flat_map(Map::values)
        .map(|f| f["columns"].as_array().map_or(0, Vec::len))
        .sum();
    let mut text = serde_json::to_string(&doc).map_err(|e| e.to_string())?;
    text.push('\n');
    if let Some(parent) = a.out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(&a.out, text).map_err(|e| format!("{}: {e}", a.out.display()))?;
    Ok(format!(
        "wrote {} fixture(s), {cols} trend column row(s) to {}",
        n,
        a.out.display()
    ))
}
