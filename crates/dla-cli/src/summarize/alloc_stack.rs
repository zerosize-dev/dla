//! `alloc-stack-diff`: allocation stacks across one or more captures.
//!
//! An adapter turns its inputs into [`ProfileCapture`]s (`dla_parsers::model::AllocProfile`);
//! [`run_diff`] groups the stacks by their frames, diffs the first capture against the
//! last, writes the `alloc-stack` evidence and, when the profiles carry live bytes and
//! there are three or more of them, a series and a `growth-series` evidence.
//!
//! Allocation churn is not retention: a profile that only counts bytes allocated gives
//! the evidence (`live: false`) and never a passing growth column. A stack's live bytes
//! growing at every step is a column, tied to the finding that lists the stack's project
//! frame among its writers or sites, or whose owner is the stack's root path.

use std::collections::BTreeMap;

use dla_core::{
    FindingId, NewFinding, Origin,
    attribution::normalize,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal},
    finding::{Holder, Region},
    record::FindingAnnotation,
    runtime::Runtime,
    series::{SeriesRegion, SeriesSourceKind, Unit},
    toolctx::EvidenceSpec,
};
use dla_parsers::model::{AllocProfile, Frame};
use serde_json::{Map, Value, json};

use super::{
    AdapterOut, CandidateRow, Ctx, Opts, Recorded, emit, growth,
    rule::{self, Observed},
    tie,
};
use crate::fail::{Failure, Res};

/// Frames that make a stack's key.
const KEY_FRAMES: usize = 8;
/// Stacks followed in the series.
const TRACKED: usize = 32;
/// Stacks kept in the raw file.
const RAW_GROUPS: usize = 200;
/// Longest function name shown.
const NAME_CLIP: usize = 72;

/// One profile and where it came from.
#[derive(Debug, Clone)]
pub struct ProfileCapture {
    /// The input file.
    pub label: String,
    /// The parsed profile.
    pub profile: AllocProfile,
    /// SHA-256 of the input file.
    pub sha256: String,
}

/// What made the profiles.
pub struct ProfileOrigin {
    /// Tool name for the evidence and the signal.
    pub tool: &'static str,
    /// Tool class for the signal.
    pub class: &'static str,
    /// Authority of a file from a live process or a test run; artifact runs override it.
    pub authority: Authority,
    /// Runtime of the measured process.
    pub runtime: Option<Runtime>,
    /// Region the stacks' memory is in.
    pub region: SeriesRegion,
    /// Order captures by their own timestamps when all have one.
    pub order_by_time: bool,
    /// Extra limitation lines.
    pub limitations: Vec<String>,
}

struct Stack {
    key: String,
    frames: Vec<Frame>,
    root_path: Vec<String>,
    kind: Option<String>,
    live_bytes: Vec<Option<u64>>,
    live_blocks: Vec<Option<u64>>,
    total_bytes: Vec<Option<u64>>,
    /// `file:line` of the first frame in the project.
    site: Option<String>,
    /// `Class.method` of the first frame in the project, else of the first frame.
    label: String,
    /// The function of the first frame in the project.
    site_fn: Option<String>,
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

fn frame_key(f: &Frame) -> String {
    match (&f.file, f.line) {
        (Some(file), Some(l)) => format!("{}@{file}:{l}", f.function),
        _ => f.function.clone(),
    }
}

/// The holder a root path starts with: its first entry, when that is a holder key
/// (`static:<Class>.<field>` or `static <Class>.<field>`). The entries after it are the
/// chain down to the sampled object and never name a holder.
fn root_owner(path: &[String]) -> Option<String> {
    let e = path.first()?.trim();
    let rest = e
        .strip_prefix("static:")
        .or_else(|| e.strip_prefix("static "))?;
    normalize(&format!("static:{}", rest.trim()))
}

fn not_enough() -> Failure {
    Failure::refuse(
        "not-enough-captures",
        "no allocation profile in the input".to_owned(),
        "1 or more allocation profiles (2 or more for a diff)",
        "dla summarize alloc-stack-diff <profile> [<later profile> ...]",
    )
}

/// Runs the diff (or, for one profile, the leak-at-exit view).
pub fn run_diff(
    ctx: &Ctx<'_>,
    opts: &Opts,
    captures: Vec<ProfileCapture>,
    origin: &ProfileOrigin,
) -> Res<AdapterOut> {
    run_diff_with(ctx, opts, captures, origin, false)
}

/// Kind of the per-holder total rows (see [`run_diff_with`]).
const HOLDER_TOTAL: &str = "holder-total";
/// Per-holder totals followed in the series.
const TRACKED_HOLDERS: usize = 8;

/// [`run_diff`]; with `holder_totals` it also follows, for every root holder
/// (`static:<Class>.<field>`) the profiles name, the sum of that holder's live bytes and
/// blocks over all its stacks, as a column `holder:<key>`. A profile that was taken
/// counts a holder it does not mention as 0. For sampling profilers (JFR) the total is
/// the stable quantity: single stacks come and go between samples. The column ties by
/// its holder only and learns no writer.
pub fn run_diff_with(
    ctx: &Ctx<'_>,
    opts: &Opts,
    mut captures: Vec<ProfileCapture>,
    origin: &ProfileOrigin,
    holder_totals: bool,
) -> Res<AdapterOut> {
    let tc = ctx.tc;
    let n = captures.len();
    if n == 0 {
        return Err(not_enough());
    }
    let timed = captures.iter().all(|c| c.profile.timestamp_ms.is_some());
    let ordered_by = if n == 1 {
        "single"
    } else if origin.order_by_time && timed {
        captures.sort_by_key(|c| c.profile.timestamp_ms);
        "timestamp"
    } else {
        "argument"
    };
    let sampled = captures.iter().any(|c| c.profile.sampled);
    let has_live = captures
        .iter()
        .any(|c| c.profile.groups.iter().any(|g| g.live_bytes > 0));

    // ---- group by stack
    let red = &ctx.redactor;
    let mut stacks: BTreeMap<String, Stack> = BTreeMap::new();
    for (ci, c) in captures.iter().enumerate() {
        for g in &c.profile.groups {
            // Stacks of different holders stay apart: the holder is part of the key.
            let key = format!(
                "{} || {}",
                g.frames
                    .iter()
                    .take(KEY_FRAMES)
                    .map(frame_key)
                    .collect::<Vec<_>>()
                    .join(" < "),
                g.root_path.first().map_or("", |r| r.trim())
            );
            let s = stacks.entry(key.clone()).or_insert_with(|| {
                let project_frame = if ctx.artifact_run {
                    None
                } else {
                    g.frames.iter().find_map(|f| {
                        // A frame with a file but no line (folded stacks) has no site: a
                        // line is not invented.
                        if f.file.is_some() && f.line.is_none_or(|l| l == 0) {
                            return None;
                        }
                        ctx.resolver()
                            .frame_site(&f.function, f.file.as_deref(), f.line)
                            .map(|site| (f, site))
                    })
                };
                let label_frame = project_frame.as_ref().map(|(f, _)| *f).or(g.frames.first());
                Stack {
                    key,
                    frames: g.frames.iter().take(KEY_FRAMES).cloned().collect(),
                    root_path: g.root_path.clone(),
                    kind: g.kind.clone(),
                    live_bytes: vec![None; n],
                    live_blocks: vec![None; n],
                    total_bytes: vec![None; n],
                    site_fn: project_frame.as_ref().map(|(f, _)| f.function.clone()),
                    site: project_frame.map(|(_, s)| s),
                    label: label_frame.map_or_else(
                        || "(no frames)".to_owned(),
                        |f| clip(&red.name(&f.function), NAME_CLIP),
                    ),
                }
            });
            let add =
                |slot: &mut Option<u64>, v: u64| *slot = Some(slot.unwrap_or(0).saturating_add(v));
            add(&mut s.live_bytes[ci], g.live_bytes);
            add(&mut s.live_blocks[ci], g.live_blocks);
            if let Some(t) = g.total_bytes {
                add(&mut s.total_bytes[ci], t);
            }
        }
    }
    // Per-holder totals: kept apart from the stacks, so they are not ranked or listed as
    // stacks, and followed in the series first.
    let mut totals: BTreeMap<String, Stack> = BTreeMap::new();
    if holder_totals && has_live {
        for s in stacks.values() {
            let (Some(owner), Some(first)) = (root_owner(&s.root_path), s.root_path.first()) else {
                continue;
            };
            let key = format!("holder:{}", first.trim().replace(' ', ":"));
            let t = totals.entry(owner).or_insert_with(|| Stack {
                key: key.clone(),
                frames: Vec::new(),
                root_path: vec![first.clone()],
                kind: Some(HOLDER_TOTAL.to_owned()),
                live_bytes: vec![None; n],
                live_blocks: vec![None; n],
                total_bytes: vec![None; n],
                site: None,
                label: clip(&red.name(&key), NAME_CLIP),
                site_fn: None,
            });
            for i in 0..n {
                for (to, from) in [
                    (&mut t.live_bytes, &s.live_bytes),
                    (&mut t.live_blocks, &s.live_blocks),
                ] {
                    // A recording that never mentions the holder stays a gap (None): a
                    // sample that was not taken is not a total of zero.
                    if let Some(v) = from[i] {
                        to[i] = Some(to[i].unwrap_or(0).saturating_add(v));
                    }
                }
            }
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
    let mut ranked: Vec<&Stack> = stacks.values().collect();
    ranked.sort_by(|a, b| {
        let by = |s: &Stack| {
            if has_live {
                delta(&s.live_bytes)
            } else {
                delta(&s.total_bytes)
            }
        };
        by(b).cmp(&by(a)).then_with(|| a.key.cmp(&b.key))
    });

    // ---- raw file
    let (aid, arel) = emit::next_alloc(tc)?;
    let groups_raw: Vec<Value> = {
        let mut by_peak: Vec<&Stack> = stacks.values().collect();
        by_peak.sort_by_key(|s| {
            (
                std::cmp::Reverse(
                    s.live_bytes
                        .iter()
                        .chain(&s.total_bytes)
                        .flatten()
                        .max()
                        .copied()
                        .unwrap_or(0),
                ),
                s.key.clone(),
            )
        });
        by_peak
            .into_iter()
            .take(RAW_GROUPS)
            .map(|s| {
                json!({
                    "label": s.label,
                    "site": s.site,
                    "frames": s.frames.iter().map(|f| json!({
                        "function": red.name(&f.function),
                        "file": f.file.as_deref().map(|x| red.name(x)),
                        "line": f.line,
                    })).collect::<Vec<_>>(),
                    "root_path": s.root_path.iter().map(|r| red.name(r)).collect::<Vec<_>>(),
                    "kind": s.kind.as_deref().map(|k| red.name(k)),
                    "live_bytes": s.live_bytes,
                    "live_blocks": s.live_blocks,
                    "total_bytes": s.total_bytes,
                })
            })
            .collect()
    };
    let raw_value = json!({
        "id": aid,
        "tool": origin.tool,
        "mode": if n == 1 { "single" } else { "diff" },
        "ordered_by": ordered_by,
        "live": has_live,
        "sampled": sampled,
        "captures": captures.iter().map(|c| json!({
            "label": c.label,
            "timestamp_ms": c.profile.timestamp_ms,
            "source": c.profile.source,
            "groups": c.profile.groups.len(),
            "sha256": c.sha256,
        })).collect::<Vec<_>>(),
        "stacks": groups_raw,
        "redaction": red.mode(),
    });
    let raw = emit::write_raw(tc, &arel, &raw_value)?;

    // ---- the alloc-stack evidence
    let shas: Vec<&str> = captures.iter().map(|c| c.sha256.as_str()).collect();
    let signal = Signal::new(origin.tool, origin.class, &emit::combined_sha(&shas));
    let authority = ctx.authority(origin.authority);
    let top_row = |s: &Stack, short: bool| -> Value {
        let mut v = json!({"at": clip(&s.label, if short { 40 } else { NAME_CLIP })});
        if let Some(site) = &s.site {
            v["site"] = json!(site);
        }
        if has_live {
            v["d_bytes"] = json!(emit::jn(delta(&s.live_bytes)));
            v["d_blocks"] = json!(emit::jn(delta(&s.live_blocks)));
        } else {
            v["alloc_bytes"] = json!(emit::jn(delta(&s.total_bytes)));
        }
        if let Some(r) = s.root_path.first() {
            v["root"] = json!(clip(&red.name(r), 48));
        }
        v
    };
    let mut base = Map::new();
    base.insert("mode".into(), json!(if n == 1 { "single" } else { "diff" }));
    base.insert("captures".into(), json!(n));
    base.insert("live".into(), json!(has_live));
    base.insert("sampled".into(), json!(sampled));
    base.insert("stacks".into(), json!(stacks.len()));
    let mut list = emit::Bounded::new(base, "top", emit::RESULT_BUDGET - 120);
    for s in ranked.iter().take(8) {
        let d = if has_live {
            delta(&s.live_bytes)
        } else {
            delta(&s.total_bytes)
        };
        if d <= 0 {
            break;
        }
        list.push(top_row(s, false));
    }
    let mut spec = EvidenceSpec::new(
        EvidenceKind::AllocStack,
        origin.tool,
        env!("CARGO_PKG_VERSION"),
        list.finish(),
    );
    spec.authority = authority;
    spec.source = Some(tc.artifact_source(&arel)?);
    spec.coverage = EvidenceCoverage {
        status: CoverageStatus::Complete,
        examined: Some(stacks.len() as u64),
        total: None,
    };
    spec.limitations = origin.limitations.clone();
    if !has_live {
        spec.limitations.push(
            "the profile records bytes allocated, not bytes still live: allocation churn is not retention, so it never yields a growth column".into(),
        );
    }
    if sampled {
        spec.limitations
            .push("the profile is sampled: counts and bytes are estimates".into());
    }
    let mut ledger_ref = ctx.ledger()?;
    let ledger = &mut *ledger_ref;
    let stack_ev: Recorded = ctx.record(ledger, spec, &signal, &raw)?;

    let mut out = AdapterOut {
        raw: Some(raw.clone()),
        enumerated: stacks.len() as u64,
        ..AdapterOut::default()
    };
    let mut body = Map::new();
    body.insert("mode".into(), json!(if n == 1 { "single" } else { "diff" }));
    body.insert("captures".into(), json!(n));
    body.insert("ordered_by".into(), json!(ordered_by));
    body.insert("live".into(), json!(has_live));
    body.insert("sampled".into(), json!(sampled));
    body.insert(
        "top".into(),
        json!(
            ranked
                .iter()
                .take(opts.top.clamp(1, 5))
                .filter(|s| {
                    if has_live {
                        delta(&s.live_bytes) > 0
                    } else {
                        delta(&s.total_bytes) > 0
                    }
                })
                .map(|s| top_row(s, true))
                .collect::<Vec<_>>()
        ),
    );
    body.insert("redaction".into(), json!(red.mode()));
    let stack_ev_id = stack_ev.id.clone();
    out.evidence.push(stack_ev);

    if n < rule::MIN_DUMPS || !has_live {
        body.insert("series".into(), Value::Null);
        body.insert("gate".into(), json!("fail"));
        let why = if !has_live {
            "the profile has no live bytes (allocation only)"
        } else {
            "captures < 3"
        };
        body.insert("reasons".into(), json!([why]));
        body.insert(
            "needed".into(),
            json!(if has_live {
                "2 more profiles of the same process after more traffic (3 confirm growth); rerun with all profiles"
            } else {
                "a profile of live (still allocated) bytes: JFR OldObjectSample, heaptrack leaked, valgrind still-reachable or dhat end-of-run"
            }),
        );
        out.body = body;
        return Ok(out);
    }

    // ---- series and growth
    let monotone = |s: &Stack| {
        rule::gate(&Observed {
            bytes: s.live_bytes.clone(),
            counts: if s.live_blocks.iter().flatten().any(|b| *b > 0) {
                s.live_blocks.clone()
            } else {
                s.live_bytes.clone()
            },
        })
        .is_ok()
    };
    let mut held: Vec<&Stack> = totals.values().collect();
    held.sort_by_key(|s| std::cmp::Reverse((monotone(s), delta(&s.live_bytes))));
    held.truncate(TRACKED_HOLDERS);
    let mut picked: Vec<&Stack> = held;
    picked.extend(ranked.iter().copied().filter(|s| monotone(s)));
    for s in &ranked {
        if picked.len() >= TRACKED {
            break;
        }
        if !picked.iter().any(|p| std::ptr::eq(*p, *s)) {
            picked.push(s);
        }
    }
    picked.truncate(TRACKED);
    let mut tracks: Vec<growth::Track> = Vec::new();
    let mut new_candidates: Vec<CandidateRow> = Vec::new();
    let who = tie::Who::new(tc.worker.as_deref(), origin.runtime);
    let holders: Vec<String> = picked
        .iter()
        .filter_map(|s| root_owner(&s.root_path))
        .collect();
    for s in &picked {
        let owner = root_owner(&s.root_path);
        let mut matches: Vec<(FindingId, tie::TieReason)> = Vec::new();
        if let Some(o) = owner.as_deref() {
            // The stack names its holder: only that holder's finding.
            for f in tie::findings_by_holder(ledger, &who, o, &holders, None) {
                matches.push((f, tie::TieReason::holder(o)));
            }
        } else if s.root_path.is_empty()
            && let Some(site) = &s.site
        {
            // No holder named (heaptrack, dhat, valgrind): the first project frame ties
            // by the finding's own site or its recorded writers.
            for f in tie::findings_at_site(ledger, &who, site) {
                matches.push((f, tie::TieReason::site(site)));
            }
        }
        // A passing stack that names an owner no finding has: a candidate, on a run with
        // source and a declaration the resolver finds.
        if matches.is_empty()
            && opts.finding.is_none()
            && monotone(s)
            && let Some(o) = owner.as_deref()
            && !ctx.artifact_run
            && let Some(r) = ctx.resolver().resolve(o)
            && tc.in_scope(r.site.rsplit_once(':').map_or(r.site.as_str(), |(f, _)| f))
            && tie::findings_by_holder(ledger, &who, o, &[], None).is_empty()
        {
            let mut nf = NewFinding::new(
                Origin::parse("growth-series").map_err(dla_core::Refusal::from)?,
                vec![stack_ev_id.clone()],
            );
            nf.holder = Some(r.holder.unwrap_or(Holder::StaticGlobal));
            nf.owner = Some(o.to_owned());
            nf.region = Some(r.region.unwrap_or(match origin.region {
                SeriesRegion::Heap => Region::Heap,
                _ => Region::OffHeap,
            }));
            nf.site = Some(r.site.clone());
            let line = ledger.add_finding(nf).map_err(|e| e.to_refusal())?;
            if let dla_core::Record::FindingAdded { finding } = line.record {
                new_candidates.push(CandidateRow {
                    finding: finding.id.clone(),
                    owner: Some(o.to_owned()),
                    holder: finding.holder,
                    site: Some(r.site.clone()),
                    region: finding.region,
                    origin: "growth-series".into(),
                    added: true,
                    via: Some(r.via.as_str()),
                });
                matches.push((finding.id, tie::TieReason::holder(o)));
            }
        }
        let r_tie = tie::resolve(matches, opts.finding.as_ref());
        let mut t = growth::Track {
            name: if s.kind.as_deref() == Some(HOLDER_TOTAL) {
                s.label.clone()
            } else {
                format!("stack:{}", s.label)
            },
            unit: Unit::Bytes,
            values: s.live_bytes.clone(),
            counts: s
                .live_blocks
                .iter()
                .any(|b| b.is_some_and(|b| b > 0))
                .then(|| s.live_blocks.clone()),
            site: None,
            finding: None,
            candidates: Vec::new(),
            region: origin.region,
            tie: None,
            ambiguous: r_tie.ambiguous,
        };
        if let Some((f, why)) = r_tie.tie {
            let site = tie::site_of(ledger, &f);
            t.tie_to_all(f, &r_tie.also, why, site);
        }
        tracks.push(t);
    }
    // Two stacks can share a label: make the column names unique.
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for t in &mut tracks {
        let k = seen.entry(t.name.clone()).or_insert(0);
        *k += 1;
        if *k > 1 {
            t.name = format!("{}#{k}", t.name);
        }
    }
    let ambiguous: Vec<(String, Vec<FindingId>)> = tracks
        .iter()
        .zip(&picked)
        .filter(|(t, s)| !t.ambiguous.is_empty() && monotone(s))
        .map(|(t, _)| (t.name.clone(), t.ambiguous.clone()))
        .collect();
    let t0 = captures[0].profile.timestamp_ms.unwrap_or(0);
    let t_ms: Vec<u64> = captures
        .iter()
        .map(|c| c.profile.timestamp_ms.unwrap_or(t0).saturating_sub(t0))
        .collect();
    let sites: Vec<(String, Option<(String, String)>)> = tracks
        .iter()
        .zip(&picked)
        .map(|(t, s)| (t.name.clone(), s.site.clone().zip(s.site_fn.clone())))
        .collect();
    let tracks_by_holder: Vec<String> = tracks
        .iter()
        .filter(|t| t.tie.as_ref().is_some_and(|w| w.rule == "holder"))
        .map(|t| t.name.clone())
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
            command: "dla summarize alloc-stack-diff".into(),
            source_kind: SeriesSourceKind::Dump,
            runtime: origin.runtime,
            limitations: origin.limitations.clone(),
            what: "capture",
            focus: opts.finding.clone(),
        },
    )?;
    let tied_ids = growth::tied_ids(&g, opts.finding.as_ref());
    // A stack tied by its holder shows where the holder is written: its first project
    // frame becomes a writer of the finding, and its site when the finding has none. A
    // site tie learns nothing, and neither does a constructor or allocator frame.
    for (name, f) in &g.tied {
        let by_holder = tracks_by_holder.contains(name);
        let Some((_, Some((site, function)))) = sites.iter().find(|(n, _)| n == name) else {
            continue;
        };
        if by_holder && !tie::is_allocator_frame(function) {
            add_writer(ctx, ledger, f, site)?;
        }
    }
    for f in &tied_ids {
        if new_candidates.iter().any(|c| &c.finding == f) {
            continue;
        }
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
    out.candidates.extend(new_candidates);
    if let Value::Object(m) = g.json.clone() {
        body.extend(m);
    }
    body.insert("tied".into(), json!(tied_ids));
    body.insert(
        "candidates".into(),
        json!(
            out.candidates
                .iter()
                .filter(|c| c.added)
                .map(CandidateRow::to_json)
                .collect::<Vec<_>>()
        ),
    );
    if let Some(a) = growth::ambiguous_json(&ambiguous) {
        body.insert("ambiguous".into(), a);
    }
    let needed = if !g.pass {
        "no stack's live bytes grow at every step: take another profile after more traffic and rerun with all profiles".to_owned()
    } else if let Some(f) = tied_ids.first() {
        format!(
            "a refuted non-leak sibling and a holder, then dla ledger confirm {f} --evidence {}",
            g.evidence.id
        )
    } else if let Some(f) = &opts.finding {
        format!(
            "no passing column ties to {f} by holder, site or type: a stack ties to the finding whose owner is its root holder (static:<Class>.<field>), or, with no root path, to the finding at its first project frame or among that finding's writers"
        )
    } else {
        "the growing stacks tie to no finding: a stack ties to the finding whose owner is its root holder, or, with no root path, to the finding at its first project frame (dla census so a holder exists there)".to_owned()
    };
    body.insert("needed".into(), json!(needed));
    out.evidence.push(g.evidence);
    out.body = body;
    Ok(out)
}

/// Records `site` as a writer of `f` (once), and as the finding's site when it has none
/// and the run has source.
fn add_writer(ctx: &Ctx<'_>, ledger: &mut dla_core::Ledger, f: &FindingId, site: &str) -> Res<()> {
    let has_site = ledger.finding(f).is_some_and(|x| x.site.is_some());
    let known = ledger
        .annotation(f)
        .is_some_and(|a| a.writers.iter().any(|w| w == site))
        || ledger
            .finding(f)
            .is_some_and(|x| x.site.as_deref() == Some(site));
    if known && has_site {
        return Ok(());
    }
    let mut a = FindingAnnotation::new(f.clone());
    if !known {
        a.writers.push(site.to_owned());
    }
    if !has_site && !ctx.artifact_run {
        a.site = Some(site.to_owned());
    }
    if a.writers.is_empty() && a.site.is_none() {
        return Ok(());
    }
    ledger
        .annotate_record(a)
        .map_err(|e| Failure::from(e.to_refusal()))?;
    Ok(())
}
