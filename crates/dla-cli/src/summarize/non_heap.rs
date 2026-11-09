//! `non-heap`: threads, descriptors, class loaders and native memory across captures.
//!
//! An adapter turns its inputs into [`SnapCapture`]s (`dla_parsers::model::NonHeapSnapshot`);
//! [`run_diff`] diffs metrics and resource groups, writes the `non-heap` evidence and,
//! with three or more captures, a series and a `growth-series` evidence. A group's column
//! (threads created at one site, descriptors of one kind) is tied to the finding whose
//! site or writers include the group's creation site.

use std::collections::BTreeMap;

use dla_core::{
    FindingId,
    error::codes as core_codes,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal},
    runtime::Runtime,
    series::{SeriesRegion, SeriesSourceKind, Unit},
    toolctx::EvidenceSpec,
};
use dla_parsers::model::NonHeapSnapshot;
use serde_json::{Map, Value, json};

use super::{
    AdapterOut, CandidateRow, Ctx, Opts, Recorded, emit, growth,
    rule::{self, Observed},
    tie,
};
use crate::fail::{Failure, Res};

/// Items followed in the series.
const TRACKED: usize = 48;
/// Longest name shown.
const NAME_CLIP: usize = 72;

/// One snapshot and where it came from.
#[derive(Debug, Clone)]
pub struct SnapCapture {
    /// The input file.
    pub label: String,
    /// The parsed snapshot.
    pub snap: NonHeapSnapshot,
    /// SHA-256 of the input file.
    pub sha256: String,
}

/// What made the snapshots.
pub struct SnapOrigin {
    /// Tool name for the evidence and the signal.
    pub tool: &'static str,
    /// Tool class for the signal.
    pub class: &'static str,
    /// Authority of a file from a live process or a test run; artifact runs override it.
    pub authority: Authority,
    /// Runtime of the measured process.
    pub runtime: Option<Runtime>,
    /// Order captures by their own timestamps when all have one.
    pub order_by_time: bool,
    /// Extra limitation lines.
    pub limitations: Vec<String>,
}

struct Item {
    /// `metric:<category>/<name>` or `group:<kind>/<key>`.
    name: String,
    unit: Unit,
    values: Vec<Option<u64>>,
    site: Option<String>,
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

/// Runs the diff over `captures`.
pub fn run_diff(
    ctx: &Ctx<'_>,
    opts: &Opts,
    mut captures: Vec<SnapCapture>,
    origin: &SnapOrigin,
) -> Res<AdapterOut> {
    let tc = ctx.tc;
    let red = &ctx.redactor;
    let n = captures.len();
    if n == 0 {
        return Err(Failure::refuse(
            "not-enough-captures",
            "no non-heap snapshot in the input".to_owned(),
            "1 or more snapshots (2 or more for a diff)",
            "dla summarize non-heap <snapshot> [<later snapshot> ...]",
        ));
    }
    let timed = captures.iter().all(|c| c.snap.timestamp_ms.is_some());
    let ordered_by = if n == 1 {
        "single"
    } else if origin.order_by_time && timed {
        captures.sort_by_key(|c| c.snap.timestamp_ms);
        "timestamp"
    } else {
        "argument"
    };

    // ---- items across captures
    let mut items: BTreeMap<String, Item> = BTreeMap::new();
    for (ci, c) in captures.iter().enumerate() {
        for m in &c.snap.metrics {
            let (unit, mult) = match m.unit.as_str() {
                "kb" => (Unit::Bytes, 1024),
                "bytes" => (Unit::Bytes, 1),
                _ => (Unit::Count, 1),
            };
            let name = format!("metric:{}/{}", red.name(&m.category), red.name(&m.name));
            let it = items.entry(name.clone()).or_insert_with(|| Item {
                name,
                unit,
                values: vec![None; n],
                site: None,
            });
            it.values[ci] = Some(u64::try_from(m.value.max(0)).unwrap_or(0) * mult);
        }
        for g in &c.snap.groups {
            let name = format!("group:{}/{}", red.name(&g.kind), red.name(&g.key));
            let site = if ctx.artifact_run {
                None
            } else {
                g.site.as_ref().and_then(|f| {
                    ctx.resolver()
                        .frame_site(&f.function, f.file.as_deref(), f.line)
                })
            };
            let it = items.entry(name.clone()).or_insert_with(|| Item {
                name,
                unit: Unit::Count,
                values: vec![None; n],
                site,
            });
            it.values[ci] = Some(it.values[ci].unwrap_or(0) + g.count);
        }
    }
    let delta = |v: &[Option<u64>]| -> i128 {
        let first = v.first().copied().flatten().unwrap_or(0);
        let last = v.last().copied().flatten().unwrap_or(0);
        if n == 1 {
            i128::from(last)
        } else {
            i128::from(last) - i128::from(first)
        }
    };
    let mut ranked: Vec<&Item> = items.values().collect();
    ranked.sort_by(|a, b| {
        delta(&b.values)
            .cmp(&delta(&a.values))
            .then_with(|| a.name.cmp(&b.name))
    });
    let monotone = |it: &Item| {
        n >= rule::MIN_DUMPS
            && rule::gate(&Observed {
                bytes: it.values.clone(),
                counts: it.values.clone(),
            })
            .is_ok()
    };
    let mono: Vec<&Item> = ranked.iter().copied().filter(|i| monotone(i)).collect();

    // ---- raw file
    let (nid, nrel) = emit::next_nonheap(tc)?;
    let raw_value = json!({
        "id": nid,
        "tool": origin.tool,
        "mode": if n == 1 { "single" } else { "diff" },
        "ordered_by": ordered_by,
        "captures": captures.iter().map(|c| json!({
            "label": c.label,
            "timestamp_ms": c.snap.timestamp_ms,
            "source": c.snap.source,
            "metrics": c.snap.metrics.len(),
            "groups": c.snap.groups.len(),
            "sha256": c.sha256,
        })).collect::<Vec<_>>(),
        "items": ranked.iter().take(400).map(|i| json!({
            "name": i.name, "unit": i.unit, "values": i.values, "site": i.site,
        })).collect::<Vec<_>>(),
        "monotone": mono.iter().map(|i| i.name.clone()).collect::<Vec<_>>(),
        "redaction": red.mode(),
    });
    let raw = emit::write_raw(tc, &nrel, &raw_value)?;

    // ---- the non-heap evidence
    let shas: Vec<&str> = captures.iter().map(|c| c.sha256.as_str()).collect();
    let signal = Signal::new(origin.tool, origin.class, &emit::combined_sha(&shas));
    let authority = ctx.authority(origin.authority);
    let row = |i: &Item, short: bool| {
        let mut v = json!({
            "item": clip(&i.name, if short { 44 } else { NAME_CLIP }),
            "d": delta(&i.values),
            "last": i.values.last().copied().flatten().unwrap_or(0),
        });
        if let Some(s) = &i.site {
            v["site"] = json!(s);
        }
        v
    };
    let mut base = Map::new();
    base.insert("mode".into(), json!(if n == 1 { "single" } else { "diff" }));
    base.insert("captures".into(), json!(n));
    base.insert("items".into(), json!(items.len()));
    base.insert("monotone".into(), json!(mono.len()));
    let mut list = emit::Bounded::new(base, "top", emit::RESULT_BUDGET - 100);
    for i in ranked.iter().take(8) {
        if delta(&i.values) <= 0 {
            break;
        }
        list.push(row(i, false));
    }
    let mut spec = EvidenceSpec::new(
        EvidenceKind::NonHeap,
        origin.tool,
        env!("CARGO_PKG_VERSION"),
        list.finish(),
    );
    spec.authority = authority;
    spec.source = Some(tc.artifact_source(&nrel)?);
    spec.coverage = EvidenceCoverage {
        status: CoverageStatus::Complete,
        examined: Some(items.len() as u64),
        total: None,
    };
    spec.limitations = origin.limitations.clone();
    let mut ledger_ref = ctx.ledger()?;
    let ledger = &mut *ledger_ref;
    let ev: Recorded = ctx.record(ledger, spec, &signal, &raw)?;
    let mut out = AdapterOut {
        raw: Some(raw.clone()),
        enumerated: items.len() as u64,
        ..AdapterOut::default()
    };
    let mut body = Map::new();
    body.insert("mode".into(), json!(if n == 1 { "single" } else { "diff" }));
    body.insert("captures".into(), json!(n));
    body.insert("ordered_by".into(), json!(ordered_by));
    body.insert(
        "top".into(),
        json!(
            ranked
                .iter()
                .take(opts.top.clamp(1, 5))
                .filter(|i| delta(&i.values) > 0)
                .map(|i| row(i, true))
                .collect::<Vec<_>>()
        ),
    );
    body.insert("monotone".into(), json!(mono.len()));
    body.insert("redaction".into(), json!(red.mode()));
    out.evidence.push(ev);

    if let Some(f) = &opts.finding
        && ledger.finding(f).is_none()
    {
        return Err(Failure::from(dla_core::Refusal::new(
            core_codes::UNKNOWN_FINDING,
            format!("no finding {f} in the ledger"),
            "an existing finding",
            "dla ledger show",
        )));
    }
    if n < rule::MIN_DUMPS {
        body.insert("series".into(), Value::Null);
        body.insert("gate".into(), json!("fail"));
        body.insert("reasons".into(), json!(["captures < 3"]));
        body.insert(
            "needed".into(),
            json!("2 more snapshots of the same process after more traffic (3 confirm growth); rerun with all snapshots"),
        );
        out.body = body;
        return Ok(out);
    }

    // ---- series and growth
    let mut picked: Vec<&Item> = mono.clone();
    for i in &ranked {
        if picked.len() >= TRACKED {
            break;
        }
        if !picked.iter().any(|p| std::ptr::eq(*p, *i)) {
            picked.push(i);
        }
    }
    picked.truncate(TRACKED);
    let mut tracks: Vec<growth::Track> = picked
        .iter()
        .map(|i| {
            let matches: Vec<FindingId> = i
                .site
                .as_deref()
                .map(|s| tie::findings_at_site(ledger, s))
                .unwrap_or_default();
            let (finding, candidates) = match matches.as_slice() {
                [one] => (Some(one.clone()), vec![one.clone()]),
                _ => (None, matches),
            };
            growth::Track {
                name: i.name.clone(),
                unit: i.unit,
                values: i.values.clone(),
                counts: None,
                site: i.site.clone().filter(|_| finding.is_some()),
                finding,
                candidates,
                region: SeriesRegion::NonHeap,
            }
        })
        .collect();
    let mut tie_note: Option<&str> = None;
    if let Some(f) = &opts.finding {
        let passing: Vec<usize> = picked
            .iter()
            .enumerate()
            .filter(|(_, i)| monotone(i))
            .map(|(k, _)| k)
            .collect();
        let matched: Vec<usize> = passing
            .iter()
            .copied()
            .filter(|&k| tracks[k].candidates.contains(f))
            .collect();
        let to_tie: Vec<usize> = if matched.is_empty() {
            passing.iter().copied().take(1).collect()
        } else {
            matched
        };
        if let Some(&first) = to_tie.first() {
            if !tracks[first].candidates.contains(f) {
                tie_note = Some("tied by --finding");
            }
            for k in to_tie {
                tracks[k].finding = Some(f.clone());
                if !tracks[k].candidates.contains(f) {
                    tracks[k].candidates.push(f.clone());
                }
            }
        }
    }
    let t0 = captures[0].snap.timestamp_ms.unwrap_or(0);
    let t_ms: Vec<u64> = captures
        .iter()
        .map(|c| c.snap.timestamp_ms.unwrap_or(t0).saturating_sub(t0))
        .collect();
    let cands: Vec<(String, Vec<FindingId>)> = tracks
        .iter()
        .map(|t| (t.name.clone(), t.candidates.clone()))
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
            command: "dla summarize non-heap".into(),
            source_kind: SeriesSourceKind::Dump,
            runtime: origin.runtime,
            limitations: origin.limitations.clone(),
            what: "snapshot",
        },
    )?;
    let mut tied_ids: Vec<FindingId> = Vec::new();
    for (name, c) in &cands {
        if g.passed.contains(name) {
            for f in c {
                if !tied_ids.contains(f) {
                    tied_ids.push(f.clone());
                }
            }
        }
    }
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
    if let Some(note) = tie_note {
        body.insert("tie".into(), json!(note));
    }
    let needed = if !g.pass {
        "nothing grows at every step: take another snapshot after more traffic and rerun with all snapshots".to_owned()
    } else if tied_ids.is_empty() {
        "the growing items name no finding: give --finding <F>, or run dla census so a spawn or handle holder exists at the creation site".to_owned()
    } else {
        format!(
            "a refuted non-leak sibling and a holder, then dla ledger confirm {} --evidence {}",
            tied_ids[0], g.evidence.id
        )
    };
    body.insert("needed".into(), json!(needed));
    out.evidence.push(g.evidence);
    out.body = body;
    Ok(out)
}
