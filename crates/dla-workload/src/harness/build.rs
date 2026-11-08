//! Turning dump records and OS samples into series files.
//!
//! [`build_series`] takes the measured points of one key mode (the chosen dump record, the
//! symbolized allocator sites and the OS figures of each) and produces the
//! [`SeriesFile`]: one column per reported figure, named by the catalog in
//! `dla_core::series::column_meta`, capped at [`MAX_COLUMNS`].

use std::collections::{BTreeMap, BTreeSet, HashMap};

use dla_core::{
    FindingId,
    harness::{DumpPhase, HarnessManifest, ProbeTarget},
    runtime::Runtime,
    series::{
        Axis, Column, KeyMode, MAX_COLUMNS, Point, SERIES_SCHEMA_VERSION, SeriesEvent, SeriesFile,
        SeriesSource, SeriesSourceKind, column_meta,
    },
    toolctx::ToolCtx,
};
use serde_json::json;

use super::{dumps::Rec, probes::resolve_site, symbolize::Sites};

/// OS columns in the order they lead a series.
const OS_ORDER: &[&str] = &[
    "os.rss",
    "os.rss_anon",
    "os.rss_file",
    "os.rss_shmem",
    "os.swap",
    "os.vm_data",
    "os.vm_hwm",
    "os.pss",
    "os.pss_anon",
    "os.private_dirty",
    "os.phys_footprint",
    "os.private",
    "os.pagefile",
    "os.fds",
    "os.handles",
    "os.threads",
    "cgroup.current",
    "cgroup.anon",
    "cgroup.file",
];

/// One measured point of a series.
#[derive(Debug, Clone, Default)]
pub struct PointInput {
    /// Axis value: iteration count `n`, or units completed.
    pub x: f64,
    /// Milliseconds since the measured process started.
    pub t_ms: u64,
    /// The chosen dump record; `None` when the agent wrote nothing for this point.
    pub rec: Option<Rec>,
    /// The record's allocator sites, symbolized.
    pub sites: Option<Sites>,
    /// The process the sampler followed (the root of the measured process tree).
    pub root_pid: Option<u32>,
    /// The OS sampler's last reading before the point ended.
    pub os_last: BTreeMap<String, f64>,
    /// The OS sampler's largest value per column up to the point.
    pub os_peak: BTreeMap<String, f64>,
}

/// What a series is about, apart from its points.
#[derive(Debug, Clone)]
pub struct SeriesParams {
    /// `S<n>`.
    pub id: String,
    /// Harness, alloc-test, ...
    pub kind: SeriesSourceKind,
    /// Key mode of the points.
    pub mode: KeyMode,
    /// What `x` means.
    pub axis: Axis,
    /// Leading warm-up points among the points given.
    pub warmup: u32,
    /// The measured command line.
    pub command: String,
    /// Notes the caller already has (selection fallbacks, missing points).
    pub limitations: Vec<String>,
    /// The purge asked for (`DLA_PURGE`), to attach the purge event of the last point.
    pub purge: Option<String>,
}

#[derive(Default)]
struct Acc {
    site: Option<String>,
    finding: Option<FindingId>,
    source: String,
    vals: Vec<Option<f64>>,
}

struct Table {
    cols: BTreeMap<String, Acc>,
    points: usize,
}

impl Table {
    fn set(&mut self, name: &str, i: usize, v: f64, source: &str) -> &mut Acc {
        let n = self.points;
        let acc = self.cols.entry(name.to_owned()).or_insert_with(|| Acc {
            source: source.to_owned(),
            vals: vec![None; n],
            ..Acc::default()
        });
        if v.is_finite() {
            acc.vals[i] = Some(v);
        }
        acc
    }
}

/// What the figures of a runtime's agent are called in `Column::source`.
fn source_tag(runtime: Runtime, phase: DumpPhase) -> &'static str {
    if phase == DumpPhase::Checkpoint {
        return "driver";
    }
    match runtime {
        Runtime::Jvm => "jvm-agent",
        Runtime::Python => "py-agent",
        Runtime::Node => "node-agent",
        Runtime::Go => "go-harness",
        Runtime::Rust | Runtime::Native => "shim",
    }
}

/// A `file:line` site the catalog may carry: ends in `:<digits>`.
fn looks_like_site(s: &str) -> bool {
    s.rsplit_once(':').is_some_and(|(f, l)| {
        !f.is_empty() && !l.is_empty() && l.bytes().all(|b| b.is_ascii_digit())
    })
}

fn type_prefix(runtime: Runtime) -> Option<&'static str> {
    match runtime {
        Runtime::Jvm => Some("jvm.class:"),
        Runtime::Python => Some("py.type:"),
        _ => None,
    }
}

/// Probe specs that name one value, so that their site and finding describe the result.
fn specific(t: &ProbeTarget) -> bool {
    !matches!(
        t,
        ProbeTarget::JvmClass { .. }
            | ProbeTarget::PyModule { .. }
            | ProbeTarget::NodeModule { .. }
    )
}

/// Class of a column for capping, lowest value dropped first when over the cap.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    Type,
    Group,
    Other,
    Probe,
    Baseline,
}

fn classify(name: &str) -> Class {
    let meta = column_meta(name);
    if name.starts_with("jvm.class:") || name.starts_with("py.type:") {
        Class::Type
    } else if name.starts_with("probe:") {
        Class::Probe
    } else if name.contains('@') {
        Class::Group
    } else if meta.baseline {
        Class::Baseline
    } else {
        Class::Other
    }
}

/// Last minus first post-warm-up value; columns with fewer than two values grow by 0.
fn growth(vals: &[Option<f64>], warmup: usize) -> f64 {
    let v: Vec<f64> = vals.iter().skip(warmup).flatten().copied().collect();
    match (v.first(), v.last()) {
        (Some(a), Some(b)) if v.len() >= 2 => b - a,
        _ => 0.0,
    }
}

fn order_key(name: &str) -> (usize, String) {
    match OS_ORDER.iter().position(|n| *n == name) {
        Some(i) => (i, String::new()),
        None => (OS_ORDER.len(), name.to_owned()),
    }
}

/// Builds the series of one key mode. `points` are in x order; the first `warmup` are
/// warm-up. Never fails: whatever could not be read is a null value or a limitation.
pub fn build_series(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    params: &SeriesParams,
    points: &[PointInput],
) -> SeriesFile {
    let runtime = manifest.runtime;
    let n = points.len();
    let mut t = Table {
        cols: BTreeMap::new(),
        points: n,
    };
    let mut limitations = params.limitations.clone();
    let note = |s: String, l: &mut Vec<String>| {
        if !l.contains(&s) {
            l.push(s);
        }
    };
    // (point, base) pairs for which an absent site means zero.
    let mut present_bases: Vec<BTreeSet<String>> = vec![BTreeSet::new(); n];
    let mut site_cache: HashMap<String, (Option<String>, Option<FindingId>)> = HashMap::new();
    let mut type_counts: Vec<BTreeMap<String, f64>> = vec![BTreeMap::new(); n];
    let with_rec = points.iter().filter(|p| p.rec.is_some()).count();

    for (i, p) in points.iter().enumerate() {
        if let Some(rec) = &p.rec {
            let src = source_tag(runtime, rec.dump.phase);
            let d = &rec.dump;
            for (name, v) in d.values.iter().flatten() {
                if let Some(v) = v {
                    t.set(name, i, *v, src);
                }
            }
            for (name, v) in d.values_pre_gc.iter().flatten() {
                if let (Some(v), false) = (v, name.starts_with("os.")) {
                    t.set(&format!("{name}.pre_gc"), i, *v, src);
                }
            }
            for (base, sites) in d.groups.iter().flatten() {
                present_bases[i].insert(base.clone());
                for (site, v) in sites {
                    let acc = t.set(&format!("{base}@{site}"), i, *v, src);
                    if looks_like_site(site) {
                        acc.site = Some(site.clone());
                    }
                }
            }
            for probe in d.probes.iter().flatten() {
                let Some(path) = probe.path.clone().or_else(|| probe.name.clone()) else {
                    continue;
                };
                let primary = probe.reachable.or(probe.size);
                if primary.is_none() && probe.bytes.is_none() {
                    if let Some(e) = &probe.error {
                        note(format!("probe {path}: {e}"), &mut limitations);
                    }
                    continue;
                }
                let (site, finding) = site_cache
                    .entry(path.clone())
                    .or_insert_with(|| {
                        let spec = probe.id.as_deref().and_then(|id| {
                            manifest
                                .probes
                                .iter()
                                .find(|s| s.id == id && specific(&s.target))
                        });
                        let mut site = spec.and_then(|s| s.site.clone());
                        let mut finding = spec.and_then(|s| s.finding.clone());
                        if site.is_none() || finding.is_none() {
                            let (s, f) = resolve_site(ctx, manifest, &path);
                            site = site.or(s);
                            finding = finding.or(f);
                        }
                        (site, finding)
                    })
                    .clone();
                let base = format!("probe:{path}");
                let put = |t: &mut Table, name: String, v: f64| {
                    let acc = t.set(&name, i, v, src);
                    acc.site.clone_from(&site);
                    acc.finding.clone_from(&finding);
                };
                if let Some(v) = primary {
                    put(&mut t, base.clone(), v);
                }
                if let (Some(_), Some(s)) = (probe.reachable, probe.size) {
                    put(&mut t, format!("{base}.size"), s);
                }
                if let Some(b) = probe.bytes {
                    put(&mut t, format!("{base}.bytes"), b);
                }
            }
            for ty in d.types.iter().flatten() {
                if let (Some(name), Some(c)) = (&ty.name, ty.count) {
                    type_counts[i].insert(name.clone(), c);
                }
            }
            for l in d.limitations.iter().flatten() {
                note(l.clone(), &mut limitations);
            }
        }
        if let Some(sites) = &p.sites {
            present_bases[i].insert("native.live_bytes".into());
            present_bases[i].insert("native.live_blocks".into());
            for (site, tot) in &sites.by_site {
                for (base, v) in [
                    ("native.live_bytes", tot.bytes),
                    ("native.live_blocks", tot.blocks),
                ] {
                    let acc = t.set(&format!("{base}@{site}"), i, v, "shim");
                    acc.site = Some(site.clone());
                }
            }
            for l in &sites.limitations {
                note(l.clone(), &mut limitations);
            }
        }
        // OS figures. The sampler sums the whole process tree, which is right for a bare
        // process and wrong when a wrapper (`go test`, `sh`, `npm`) compiles or spawns
        // other work: then the measured process's own figures are all there is.
        let agent_os = p.rec.as_ref().is_some_and(|r| r.value("os.rss").is_some());
        let same_process = p
            .rec
            .as_ref()
            .is_some_and(|r| r.dump.pid.is_some() && r.dump.pid == p.root_pid);
        let sampler_os = !agent_os || same_process;
        if agent_os && !same_process {
            note(
                "os.* columns are the measured process's own figures: the sampled process tree (wrapper, build and sibling processes) is not mixed in".to_owned(),
                &mut limitations,
            );
        }
        for (name, v) in &p.os_last {
            let wanted = name.starts_with("cgroup.") || (name.starts_with("os.") && sampler_os);
            if wanted && t.cols.get(name).is_none_or(|a| a.vals[i].is_none()) {
                t.set(name, i, *v, "os-sampler");
            }
        }
        let peak = p
            .os_peak
            .get("os.vm_hwm")
            .or_else(|| p.os_peak.get("os.rss"))
            .copied();
        let own = t.cols.get("os.vm_hwm").and_then(|a| a.vals[i]);
        match (own, peak) {
            (Some(o), Some(pk)) if sampler_os => {
                t.set("os.vm_hwm", i, o.max(pk), "os-sampler");
            }
            (None, Some(pk)) => {
                t.set("os.vm_hwm", i, pk, "os-sampler");
            }
            _ => {}
        }
    }

    // A site missing from a record that lists its metric holds nothing at that point.
    for (name, acc) in &mut t.cols {
        let Some((base, _)) = name.split_once('@') else {
            continue;
        };
        for (i, v) in acc.vals.iter_mut().enumerate() {
            if v.is_none() && present_bases[i].contains(base) {
                *v = Some(0.0);
            }
        }
    }

    // Types seen in at least half of the points that have a record.
    if let Some(prefix) = type_prefix(runtime) {
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        for tc in &type_counts {
            for name in tc.keys() {
                *seen.entry(name.as_str()).or_default() += 1;
            }
        }
        let keep: Vec<String> = seen
            .into_iter()
            .filter(|(_, c)| with_rec > 0 && *c * 2 >= with_rec)
            .map(|(n, _)| n.to_owned())
            .collect();
        for name in keep {
            for (i, tc) in type_counts.iter().enumerate() {
                if let Some(c) = tc.get(&name) {
                    let src = source_tag(runtime, DumpPhase::Exit);
                    t.set(&format!("{prefix}{name}.count"), i, *c, src);
                }
            }
        }
    }

    // Columns with no value at all say nothing.
    t.cols.retain(|_, a| a.vals.iter().any(Option::is_some));

    // Cap.
    let warm = params.warmup as usize;
    if t.cols.len() > MAX_COLUMNS {
        let mut excess = t.cols.len() - MAX_COLUMNS;
        let mut ranked: Vec<(Class, f64, String)> = t
            .cols
            .iter()
            .map(|(name, a)| (classify(name), growth(&a.vals, warm), name.clone()))
            .collect();
        ranked.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut dropped: BTreeMap<&'static str, usize> = BTreeMap::new();
        for (class, _, name) in ranked {
            if excess == 0 {
                break;
            }
            t.cols.remove(&name);
            excess -= 1;
            let label = match class {
                Class::Type => "per-type",
                Class::Group => "grouped",
                Class::Other => "other",
                Class::Probe => "probe",
                Class::Baseline => "baseline",
            };
            *dropped.entry(label).or_default() += 1;
        }
        for (label, c) in dropped {
            limitations.push(format!(
                "dropped {c} {label} columns (cap of {MAX_COLUMNS}, lowest growth first)"
            ));
        }
    }

    let mut names: Vec<String> = t.cols.keys().cloned().collect();
    names.sort_by_key(|n| {
        let class = classify(n);
        (std::cmp::Reverse(class), order_key(n))
    });
    let columns: Vec<Column> = names
        .iter()
        .map(|name| {
            let meta = column_meta(name);
            let a = &t.cols[name];
            Column {
                name: name.clone(),
                unit: meta.unit,
                kind: meta.kind,
                region: meta.region,
                baseline: meta.baseline,
                after_gc: meta.after_gc,
                scope: meta.scope,
                site: a.site.clone(),
                finding: a.finding.clone(),
                source: a.source.clone(),
            }
        })
        .collect();
    let mut last_x = f64::NEG_INFINITY;
    let pts: Vec<Point> = points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let x = if p.x.is_finite() && p.x >= last_x {
                p.x
            } else {
                last_x.max(0.0)
            };
            last_x = x;
            Point {
                x,
                t_ms: p.t_ms,
                values: names.iter().map(|n| t.cols[n].vals[i]).collect(),
            }
        })
        .collect();

    let mut events = Vec::new();
    if let (Some(kind), Some(last)) = (&params.purge, points.last())
        && let Some(purge) = last.rec.as_ref().and_then(|r| r.dump.purge.as_ref())
    {
        let kind = purge.kind.clone().unwrap_or_else(|| kind.clone());
        events.push(SeriesEvent {
            after_point: u32::try_from(n - 1).unwrap_or(u32::MAX),
            kind: format!("purge:{kind}"),
            detail: Some(
                json!({"kind": kind, "before": purge.before, "after": purge.after}).to_string(),
            ),
        });
    } else if params.purge.is_some() && n > 0 {
        limitations.push(format!(
            "purge {} was requested but the last point's record carries no purge result",
            params.purge.as_deref().unwrap_or("")
        ));
    }

    let mut source = SeriesSource::new(params.kind);
    source.harness = Some(manifest.id.clone());
    source.workload = manifest.workload.clone();
    source.mode = Some(params.mode);
    source.runtime = Some(runtime);
    source.command = Some(params.command.clone());
    SeriesFile {
        schema_version: SERIES_SCHEMA_VERSION,
        id: params.id.clone(),
        source,
        axis: params.axis,
        warmup: params.warmup.min(u32::try_from(n).unwrap_or(u32::MAX)),
        columns,
        points: pts,
        events,
        limitations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::dumps::parse_record;
    use dla_core::harness::{AgentKind, CommandSpec, HarnessMode};
    use serde_json::{Value, json};

    fn manifest(runtime: Runtime) -> HarnessManifest {
        HarnessManifest {
            schema_version: 1,
            id: "H1".into(),
            runtime,
            mode: HarnessMode::Command,
            workload: Some("w".into()),
            scratch: "harness/H1/work".into(),
            build: None,
            run: CommandSpec {
                argv: vec!["x".into()],
                cwd: ".".into(),
                env: vec![],
            },
            iter_arg: None,
            warmup: 1,
            iterations: 3,
            step: 100,
            key_modes: vec![KeyMode::Native],
            probes: vec![],
            agent: AgentKind::None,
            detectors: vec![],
            tools: vec![],
            gaps: vec![],
            timeout_secs: 60,
        }
    }

    fn ctx() -> ToolCtx {
        let d = std::env::temp_dir();
        ToolCtx {
            project: d.clone(),
            run_dir: d,
            run_id: "r".into(),
            source_commit: "c".into(),
            data_flag: None,
        }
    }

    fn pt(x: f64, rec: Value) -> PointInput {
        PointInput {
            x,
            t_ms: x as u64,
            rec: parse_record(&rec.to_string()),
            ..PointInput::default()
        }
    }

    fn params() -> SeriesParams {
        SeriesParams {
            id: "S1".into(),
            kind: SeriesSourceKind::Harness,
            mode: KeyMode::Native,
            axis: Axis::N,
            warmup: 1,
            command: "x".into(),
            limitations: vec![],
            purge: None,
        }
    }

    #[test]
    fn values_groups_probes_and_pre_gc_become_catalog_columns() {
        let pts: Vec<PointInput> = (1..=4)
            .map(|k| {
                let n = 100.0 * f64::from(k);
                pt(
                    n,
                    json!({"dla_harness":1,"phase":"exit","pid":1,"runtime":"node",
                    "values":{"node.heap_used": 1000.0*n, "os.rss": 5e6},
                    "values_pre_gc":{"node.heap_used": 2000.0*n},
                    "groups":{"node.timers":{"src/heartbeat.js:15": n}},
                    "probes":[{"id":"P1","path":"src/s.js:latest","size":1,"reachable":n*3.0,"bytes":null}]}),
                )
            })
            .collect();
        let s = build_series(&ctx(), &manifest(Runtime::Node), &params(), &pts);
        s.validate().unwrap();
        let names: Vec<_> = s.columns.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"node.heap_used.pre_gc"));
        assert!(names.contains(&"node.timers@src/heartbeat.js:15"));
        assert!(names.contains(&"probe:src/s.js:latest"));
        assert!(names.contains(&"probe:src/s.js:latest.size"));
        assert_eq!(names[0], "os.rss");
        let g = s.column_index("node.timers@src/heartbeat.js:15").unwrap();
        assert_eq!(s.columns[g].site.as_deref(), Some("src/heartbeat.js:15"));
        assert_eq!(
            s.values(g),
            [Some(100.0), Some(200.0), Some(300.0), Some(400.0)]
        );
    }

    #[test]
    fn a_site_missing_from_a_listing_record_is_zero_and_sampler_fills_os() {
        let mut a = pt(
            100.0,
            json!({"dla_harness":1,"phase":"exit","pid":1,"runtime":"go","groups":{"go.goroutines":{"a.go:3":5}},"values":{"go.heap_live":1}}),
        );
        a.os_last.insert("os.rss".into(), 123.0);
        a.os_peak.insert("os.vm_hwm".into(), 200.0);
        let b = pt(
            200.0,
            json!({"dla_harness":1,"phase":"exit","pid":1,"runtime":"go","groups":{"go.goroutines":{}},"values":{"go.heap_live":2}}),
        );
        let s = build_series(&ctx(), &manifest(Runtime::Go), &params(), &[a, b]);
        let g = s.column_index("go.goroutines@a.go:3").unwrap();
        assert_eq!(s.values(g), [Some(5.0), Some(0.0)]);
        let r = s.column_index("os.rss").unwrap();
        assert_eq!(s.values(r), [Some(123.0), None]);
        let h = s.column_index("os.vm_hwm").unwrap();
        assert_eq!(s.values(h), [Some(200.0), None]);
        assert_eq!(s.columns[r].source, "os-sampler");
    }

    #[test]
    fn the_cap_drops_the_slowest_growing_groups_first_and_says_so() {
        let pts: Vec<PointInput> = (1..=4)
            .map(|k| {
                let mut groups = serde_json::Map::new();
                let mut sites = serde_json::Map::new();
                for j in 0..80 {
                    sites.insert(format!("f{j}.go:1"), json!(f64::from(k) * f64::from(j)));
                }
                groups.insert("go.goroutines".into(), Value::Object(sites));
                pt(
                    f64::from(k),
                    json!({"dla_harness":1,"phase":"exit","pid":1,"runtime":"go",
                    "values":{"go.heap_live": 10.0*f64::from(k)}, "groups": groups}),
                )
            })
            .collect();
        let s = build_series(&ctx(), &manifest(Runtime::Go), &params(), &pts);
        assert_eq!(s.columns.len(), MAX_COLUMNS);
        assert!(s.column_index("go.heap_live").is_some());
        assert!(s.column_index("go.goroutines@f79.go:1").is_some());
        assert!(s.column_index("go.goroutines@f1.go:1").is_none());
        assert!(
            s.limitations
                .iter()
                .any(|l| l.starts_with("dropped 17 grouped"))
        );
        s.validate().unwrap();
    }

    #[test]
    fn types_need_to_appear_in_half_of_the_points() {
        let mk = |x: f64, types: Value| {
            pt(
                x,
                json!({"dla_harness":1,"phase":"exit","pid":1,"runtime":"jvm","values":{"jvm.heap_used":x},"types":types}),
            )
        };
        let pts = vec![
            mk(
                1.0,
                json!([{"name":"shop.Order","count":1},{"name":"x.Rare","count":1}]),
            ),
            mk(2.0, json!([{"name":"shop.Order","count":2}])),
            mk(3.0, json!([{"name":"shop.Order","count":3}])),
            mk(4.0, json!([{"name":"shop.Order","count":4}])),
        ];
        let s = build_series(&ctx(), &manifest(Runtime::Jvm), &params(), &pts);
        assert!(s.column_index("jvm.class:shop.Order.count").is_some());
        assert!(s.column_index("jvm.class:x.Rare.count").is_none());
    }

    #[test]
    fn the_purge_event_carries_before_and_after_for_alloc_test() {
        let mut p = params();
        p.purge = Some("trim".into());
        let pts = vec![
            pt(
                1.0,
                json!({"dla_harness":1,"phase":"exit","pid":1,"values":{"native.live_bytes":1}}),
            ),
            pt(
                2.0,
                json!({"dla_harness":1,"phase":"exit","pid":1,"values":{"native.live_bytes":2},
                "purge":{"kind":"trim","before":{"os.rss":10},"after":{"os.rss":4}}}),
            ),
        ];
        let s = build_series(&ctx(), &manifest(Runtime::Native), &p, &pts);
        assert_eq!(s.events.len(), 1);
        assert_eq!(s.events[0].after_point, 1);
        assert_eq!(s.events[0].kind, "purge:trim");
        let d: Value = serde_json::from_str(s.events[0].detail.as_deref().unwrap()).unwrap();
        assert_eq!(d["after"]["os.rss"], 4.0);
    }
}
