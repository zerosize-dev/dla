//! `type-histogram-diff`: type or class histograms across two or more captures.
//!
//! An adapter turns its inputs into [`Capture`]s (`dla_parsers::model::Histogram` plus
//! where it came from); [`run_diff`] does the rest: orders them, diffs the first against
//! the last, finds the types that grow at every step, writes the `histogram-diff`
//! evidence and, with three or more captures, a series and a `growth-series` evidence.
//! A type's growth column is tied (see [`tie`]) to the one finding whose holder's
//! accumulation class is that type, per the heap-graph summaries of the run, or whose
//! owner declares it as its element type. Generic types tie to nothing, and a type that
//! matches several findings is reported as ambiguous. `--finding` narrows; it forces
//! nothing.

use std::collections::BTreeMap;

use dla_core::{
    FindingId,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal},
    runtime::Runtime,
    series::{SeriesRegion, SeriesSourceKind, Unit},
    toolctx::EvidenceSpec,
};
use dla_parsers::model::Histogram;
use serde_json::{Map, Value, json};

use super::{
    AdapterOut, CandidateRow, Ctx, Opts, Recorded, emit, growth,
    rule::{self, Observed},
    tie,
};
use crate::fail::{Failure, Res};

/// Types followed in the series (two columns each).
const TRACKED: usize = 32;
/// Types kept in the raw file's table.
const RAW_TYPES: usize = 300;
/// Longest type name shown.
const NAME_CLIP: usize = 80;

/// One histogram and where it came from.
#[derive(Debug, Clone)]
pub struct Capture {
    /// The input file, or `file#n` for the nth capture of one file.
    pub label: String,
    /// The histogram.
    pub hist: Histogram,
    /// SHA-256 of the input file.
    pub sha256: String,
}

/// What kind of tool made the captures.
pub struct Origin {
    /// Tool name for the evidence and the signal.
    pub tool: &'static str,
    /// Tool class for the signal.
    pub class: &'static str,
    /// Authority of a file from a live process or harness; artifact runs override it.
    pub authority: Authority,
    /// Runtime of the measured process.
    pub runtime: Option<Runtime>,
    /// True when each histogram lists only the largest types.
    pub partial: bool,
    /// How captures are ordered: by their own timestamps when all have one.
    pub order_by_time: bool,
    /// Extra limitation lines.
    pub limitations: Vec<String>,
}

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push_str("...");
        t
    }
}

/// A readable type name: `[B` becomes `byte[]`, `[Ljava.lang.Object;` becomes
/// `java.lang.Object[]`, the `(module@version)` suffix of a class histogram goes, and
/// generated proxy classes (`Svc$$SpringCGLIB$$0`, `$HibernateProxy$..`, `$ByteBuddy$..`)
/// are named after the class they wrap. Nested classes (`Outer$Inner`) are kept.
pub fn normalize_type(raw: &str) -> String {
    let t = raw.split(" (").next().unwrap_or(raw).trim();
    let mut dims = 0;
    let mut base = t;
    while let Some(rest) = base.strip_prefix('[') {
        dims += 1;
        base = rest;
    }
    let element = if dims == 0 {
        base.to_owned()
    } else {
        match base {
            "B" => "byte".into(),
            "C" => "char".into(),
            "I" => "int".into(),
            "J" => "long".into(),
            "S" => "short".into(),
            "Z" => "boolean".into(),
            "F" => "float".into(),
            "D" => "double".into(),
            b => b
                .strip_prefix('L')
                .map_or_else(|| b.to_owned(), |x| x.trim_end_matches(';').to_owned()),
        }
    };
    let mut name = element;
    for marker in ["$$", "$HibernateProxy$", "$ByteBuddy$"] {
        if let Some(i) = name.find(marker) {
            name.truncate(i);
        }
    }
    name.push_str(&"[]".repeat(dims));
    name
}

/// Summed figures per normalized type name for one capture.
fn by_type(c: &Capture) -> BTreeMap<String, (u64, u64)> {
    let mut m: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    for r in &c.hist.rows {
        let e = m.entry(normalize_type(&r.name)).or_default();
        e.0 += r.count;
        e.1 += r.bytes;
    }
    m
}

fn not_enough(n: usize) -> Failure {
    Failure::refuse(
        "not-enough-captures",
        format!("a histogram diff needs 2 or more captures, and the input holds {n}"),
        "2 or more histograms of one process",
        "dla summarize type-histogram-diff <first capture> <second capture> <third capture>",
    )
}

/// Runs the diff over `captures` (in the order given unless `origin.order_by_time` and
/// all of them carry a timestamp).
pub fn run_diff(
    ctx: &Ctx<'_>,
    opts: &Opts,
    mut captures: Vec<Capture>,
    origin: &Origin,
) -> Res<AdapterOut> {
    let tc = ctx.tc;
    let n = captures.len();
    if n < 2 {
        return Err(not_enough(n));
    }
    let timed = captures.iter().all(|c| c.hist.timestamp_ms.is_some());
    let ordered_by = if origin.order_by_time && timed {
        captures.sort_by_key(|c| c.hist.timestamp_ms);
        "timestamp"
    } else {
        "argument"
    };
    let per: Vec<BTreeMap<String, (u64, u64)>> = captures.iter().map(by_type).collect();
    let mut names: Vec<&String> = per.iter().flat_map(BTreeMap::keys).collect();
    names.sort();
    names.dedup();

    // Per type: counts and bytes in every capture (None where absent).
    struct Row<'a> {
        name: &'a String,
        counts: Vec<Option<u64>>,
        bytes: Vec<Option<u64>>,
    }
    let mut rows: Vec<Row<'_>> = names
        .into_iter()
        .map(|name| Row {
            name,
            counts: per.iter().map(|m| m.get(name).map(|x| x.0)).collect(),
            bytes: per.iter().map(|m| m.get(name).map(|x| x.1)).collect(),
        })
        .collect();
    let first_last = |v: &[Option<u64>]| -> (u64, u64) {
        (
            v.first().copied().flatten().unwrap_or(0),
            v.last().copied().flatten().unwrap_or(0),
        )
    };
    let delta = |v: &[Option<u64>]| -> i128 {
        let (a, b) = first_last(v);
        i128::from(b) - i128::from(a)
    };
    rows.sort_by(|a, b| {
        delta(&b.bytes)
            .cmp(&delta(&a.bytes))
            .then_with(|| a.name.cmp(b.name))
    });
    let monotone = |r: &Row<'_>| -> bool {
        n >= rule::MIN_DUMPS
            && rule::gate(&Observed {
                bytes: r.bytes.clone(),
                counts: r.counts.clone(),
            })
            .is_ok()
    };
    let mono: Vec<&Row<'_>> = rows.iter().filter(|r| monotone(r)).collect();

    // ---- raw file
    let (hid, hrel) = emit::next_histogram(tc)?;
    let red = &ctx.redactor;
    let table: Vec<Value> = {
        let mut by_peak: Vec<&Row<'_>> = rows.iter().collect();
        by_peak.sort_by_key(|r| {
            (
                std::cmp::Reverse(r.bytes.iter().flatten().max().copied().unwrap_or(0)),
                r.name.as_str(),
            )
        });
        by_peak
            .into_iter()
            .take(RAW_TYPES)
            .map(|r| json!({"name": red.name(r.name), "counts": r.counts, "bytes": r.bytes}))
            .collect()
    };
    let diff: Vec<Value> = rows
        .iter()
        .take(100)
        .map(|r| {
            json!({
                "name": red.name(r.name),
                "d_count": emit::jn(delta(&r.counts)),
                "d_bytes": emit::jn(delta(&r.bytes)),
            })
        })
        .collect();
    let raw_value = json!({
        "id": hid,
        "tool": origin.tool,
        "ordered_by": ordered_by,
        "partial": origin.partial,
        "captures": captures.iter().map(|c| json!({
            "label": c.label,
            "timestamp_ms": c.hist.timestamp_ms,
            "source": c.hist.source,
            "types": c.hist.rows.len(),
            "sha256": c.sha256,
        })).collect::<Vec<_>>(),
        "types": table,
        "diff": diff,
        "monotone": mono.iter().map(|r| red.name(r.name)).collect::<Vec<_>>(),
        "redaction": red.mode(),
    });
    let raw = emit::write_raw(tc, &hrel, &raw_value)?;

    // ---- the histogram-diff evidence
    let shas: Vec<&str> = captures.iter().map(|c| c.sha256.as_str()).collect();
    let signal = Signal::new(origin.tool, origin.class, &emit::combined_sha(&shas));
    let authority = ctx.authority(origin.authority);
    let mut base = Map::new();
    base.insert("captures".into(), json!(n));
    base.insert("ordered_by".into(), json!(ordered_by));
    base.insert("types".into(), json!(rows.len()));
    base.insert(
        "d_bytes".into(),
        json!(
            rows.iter()
                .map(|r| delta(&r.bytes).max(0))
                .sum::<i128>()
                .min(i128::from(u64::MAX))
        ),
    );
    let mut list = emit::Bounded::new(base, "top", emit::RESULT_BUDGET - 260);
    for r in rows.iter().take(12) {
        let d = delta(&r.bytes);
        if d <= 0 && delta(&r.counts) <= 0 {
            break;
        }
        list.push(json!({
            "type": clip(&red.name(r.name), NAME_CLIP),
            "d_count": emit::jn(delta(&r.counts)),
            "d_bytes": emit::jn(d),
        }));
    }
    let mut result = list.finish();
    result["monotone"] = json!(
        mono.iter()
            .take(5)
            .map(|r| clip(&red.name(r.name), NAME_CLIP))
            .collect::<Vec<_>>()
    );
    if mono.len() > 5 {
        result["monotone_more"] = json!(mono.len() - 5);
    }
    let mut spec = EvidenceSpec::new(
        EvidenceKind::HistogramDiff,
        origin.tool,
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = authority;
    spec.source = Some(tc.artifact_source(&hrel)?);
    spec.coverage = EvidenceCoverage {
        status: if origin.partial {
            CoverageStatus::Partial
        } else {
            CoverageStatus::Complete
        },
        examined: Some(rows.len() as u64),
        total: None,
    };
    spec.limitations = origin.limitations.clone();
    if origin.partial {
        spec.limitations.push(
            "each histogram lists only the largest types; a type outside the list has no figure"
                .into(),
        );
    }
    if n < rule::MIN_DUMPS {
        spec.limitations.push(
            "two captures cannot tell a warm-up fill from a leak: no monotone type is reported and no growth column is written".into(),
        );
    }
    let mut ledger_ref = ctx.ledger()?;
    let ledger = &mut *ledger_ref;
    let diff_ev: Recorded = ctx.record(ledger, spec, &signal, &raw)?;

    let mut out = AdapterOut {
        raw: Some(raw.clone()),
        enumerated: rows.len() as u64,
        ..AdapterOut::default()
    };
    let mut body = Map::new();
    body.insert("captures".into(), json!(n));
    body.insert("ordered_by".into(), json!(ordered_by));
    body.insert(
        "top".into(),
        json!(
            rows.iter()
                .take(opts.top.max(1))
                .filter(|r| delta(&r.bytes) > 0 || delta(&r.counts) > 0)
                .map(|r| json!({
                    "type": clip(&red.name(r.name), 48),
                    "d_count": emit::jn(delta(&r.counts)),
                    "d_bytes": emit::jn(delta(&r.bytes)),
                }))
                .collect::<Vec<_>>()
        ),
    );
    body.insert("monotone".into(), json!(mono.len()));
    body.insert("redaction".into(), json!(red.mode()));
    out.evidence.push(diff_ev);

    if n < rule::MIN_DUMPS {
        body.insert("series".into(), Value::Null);
        body.insert("gate".into(), json!("fail"));
        body.insert("reasons".into(), json!(["captures < 3"]));
        body.insert(
            "needed".into(),
            json!(
                "1 more capture of the same process after more traffic (3 confirm growth; two cannot tell a warm-up fill from a leak); rerun with all captures"
            ),
        );
        out.body = body;
        return Ok(out);
    }

    // ---- series and growth: the monotone types first, then the largest
    let types = tie::TypeIndex::build(
        ledger,
        &tie::Who::new(tc.worker.as_deref(), Some(Runtime::Jvm)),
        &tc.run_dir,
        &tc.project,
        ctx.artifact_run,
    );
    let mut picked: Vec<&Row<'_>> = mono.clone();
    for r in &rows {
        if picked.len() >= TRACKED {
            break;
        }
        if !picked.iter().any(|p| std::ptr::eq(*p, r)) {
            picked.push(r);
        }
    }
    picked.truncate(TRACKED);
    let tracks: Vec<growth::Track> = picked
        .iter()
        .map(|r| {
            let r_tie = tie::resolve(
                types.findings_for_type(ledger, r.name),
                opts.finding.as_ref(),
            );
            let mut t = growth::Track {
                name: format!("type:{}", clip(&red.name(r.name), 120)),
                unit: Unit::Bytes,
                values: r.bytes.clone(),
                counts: Some(r.counts.clone()),
                site: None,
                finding: None,
                candidates: Vec::new(),
                region: SeriesRegion::Heap,
                tie: None,
                ambiguous: r_tie.ambiguous,
            };
            if let Some((f, why)) = r_tie.tie {
                t.tie_to(f, why, None);
            }
            t
        })
        .collect();
    let ambiguous: Vec<(String, Vec<FindingId>)> = tracks
        .iter()
        .zip(&picked)
        .filter(|(t, r)| !t.ambiguous.is_empty() && monotone(r))
        .map(|(t, _)| (t.name.clone(), t.ambiguous.clone()))
        .collect();
    let t0 = captures[0].hist.timestamp_ms.unwrap_or(0);
    let t_ms: Vec<u64> = captures
        .iter()
        .map(|c| c.hist.timestamp_ms.unwrap_or(t0).saturating_sub(t0))
        .collect();
    let g = growth::write(
        ctx,
        ledger,
        growth::Plan {
            t_ms,
            tracks,
            signal,
            tool: origin.tool,
            authority,
            command: "dla summarize type-histogram-diff".into(),
            source_kind: SeriesSourceKind::Dump,
            runtime: origin.runtime,
            limitations: origin.limitations.clone(),
            what: "capture",
            focus: opts.finding.clone(),
        },
    )?;
    let tied_ids = growth::tied_ids(&g, opts.finding.as_ref());
    for f in &tied_ids {
        let fnd = ledger.finding(f);
        out.candidates.push(CandidateRow {
            finding: f.clone(),
            owner: fnd.and_then(|x| x.owner.clone()),
            holder: fnd.and_then(|x| x.holder),
            site: fnd.and_then(|x| x.site.clone()),
            region: fnd.and_then(|x| x.region),
            origin: "growth-series".into(),
            added: false,
            via: None,
        });
    }
    if let Value::Object(m) = g.json.clone() {
        body.extend(m);
    }
    body.insert("tied".into(), json!(tied_ids));
    if let Some(a) = growth::ambiguous_json(&ambiguous) {
        body.insert("ambiguous".into(), a);
    }
    let needed = if !g.pass {
        "no type grows at every step: take another capture after more traffic and rerun with all captures".to_owned()
    } else if let Some(f) = tied_ids.first() {
        format!(
            "a refuted non-leak sibling and a holder, then dla ledger confirm {f} --evidence {}",
            g.evidence.id
        )
    } else if let Some((ty, ids)) = ambiguous.first() {
        let names: Vec<String> = ids.iter().map(ToString::to_string).collect();
        format!(
            "the growing type {ty} matches several findings ({}) and ties none, even with --finding: a heap-graph holder with that accumulation class, or distinct element types, tells them apart",
            names.join(", ")
        )
    } else if let Some(f) = &opts.finding {
        format!(
            "no passing column ties to {f} by holder, site or type: dla summarize heap-graph on 3 or more dumps of the same process records the holder's accumulation class, or annotate {f}'s owner so its declared element type is read"
        )
    } else {
        "no passing type ties to a finding by holder, site or type: dla summarize heap-graph on 3 or more dumps of the same process records each holder's accumulation class; generic types (String, byte[], collections) never tie".to_owned()
    };
    body.insert("needed".into(), json!(needed));
    out.evidence.push(g.evidence);
    out.body = body;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_names_are_normalized() {
        assert_eq!(normalize_type("[B"), "byte[]");
        assert_eq!(normalize_type("[[I"), "int[][]");
        assert_eq!(
            normalize_type("[Ljava.lang.Object; (java.base@21)"),
            "java.lang.Object[]"
        );
        assert_eq!(
            normalize_type("java.lang.String (java.base@21)"),
            "java.lang.String"
        );
        assert_eq!(normalize_type("shop.Svc$$SpringCGLIB$$0"), "shop.Svc");
        assert_eq!(
            normalize_type("shop.Svc$$EnhancerBySpringCGLIB$$ab12cd"),
            "shop.Svc"
        );
        assert_eq!(normalize_type("shop.User$HibernateProxy$x1"), "shop.User");
        assert_eq!(normalize_type("shop.Outer$Inner"), "shop.Outer$Inner");
        assert_eq!(normalize_type("shop.Svc$$Lambda/0x1234"), "shop.Svc");
    }
}
