//! `dla ledger list` and `dla ledger close-rest`: the ranked candidates, and the close of
//! what the measurement queue does not hold back.

use clap::Args;
use dla_core::{
    FindingId, FindingState, RunView, UnresolvedReason,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool,
    },
    finding::Finding,
    queue::{self, Closed, Index, Standing, Why},
    rank,
    trend::Verdict,
};
use serde_json::{Value, json};

use super::{ResolveArgs, ResolveAs, open, resolve_finding};
use crate::{
    ctx::Ctx,
    fail::Res,
    parse::parse_enum,
};

/// Findings `dla ledger list` prints by default.
const DEFAULT_LIMIT: usize = 40;

/// Flags of `dla ledger list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Only findings in this state (candidate, under_test, confirmed, strong, refuted,
    /// suspected_static_only, unresolved).
    #[arg(long, value_parser = parse_enum::<FindingState>)]
    state: Option<FindingState>,
    /// At most this many findings (default 40).
    #[arg(long, value_name = "N")]
    limit: Option<usize>,
    /// Every finding, no limit.
    #[arg(long, conflicts_with = "limit")]
    all: bool,
}

/// Flags of `dla ledger close-rest`.
#[derive(Args, Debug)]
pub struct CloseRestArgs {
    /// The remedy every closed finding gets (default: one built from the finding's history).
    #[arg(long)]
    remedy: Option<String>,
}

fn verdict_text(v: Option<Verdict>) -> String {
    v.map_or_else(|| "not run".to_owned(), |v| v.as_str().replace('_', " "))
}

/// The queue status of a finding as `ledger list` prints it.
fn status_of(
    r: &queue::Ranked,
    ix: &Index,
    queues: &[queue::RuntimeQueue],
) -> (&'static str, Option<Standing>) {
    if !r.eligible() {
        return ("not-eligible", None);
    }
    if r.state != FindingState::Candidate {
        return ("closed", None);
    }
    let _ = queues;
    match ix.standing(&r.id) {
        Standing::Pending => ("pending", Some(Standing::Pending)),
        s => ("settled", Some(s)),
    }
}

/// `dla ledger list`: one line of JSON per finding in rank order, the ranked candidates
/// first, then the other findings in ledger order; a first line with the totals.
pub fn list(ctx: &Ctx, a: ListArgs) -> Res<String> {
    let view = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
    let ranked = queue::ranked(&view);
    let ix = Index::of(&view);
    let queues = queue::queues(&view, ctx.worker.as_deref());
    let mut order: Vec<&FindingId> = ranked.iter().map(|r| &r.id).collect();
    let in_rank: std::collections::BTreeSet<&FindingId> = order.iter().copied().collect();
    let others: Vec<FindingId> = view
        .ledger
        .findings()
        .filter(|f| !in_rank.contains(&f.id))
        .map(|f| f.id.clone())
        .collect();
    order.extend(others.iter());
    let by_id: std::collections::BTreeMap<&FindingId, &queue::Ranked> =
        ranked.iter().map(|r| (&r.id, r)).collect();

    let mut lines: Vec<(String, Value)> = Vec::new();
    for id in order {
        let Some(f) = view.ledger.finding(id) else {
            continue;
        };
        if a.state.is_some_and(|s| s != f.state) {
            continue;
        }
        let ann = view.ledger.annotation(id);
        let r = by_id.get(id);
        let (qs, standing) = r.map_or(("not-eligible", None), |r| status_of(r, &ix, &queues));
        let tier = r.map_or_else(|| rank::tier(f, ann), |r| r.tier);
        let mut e = json!({
            "id": id.to_string(),
            "state": f.state_label(),
            "tier": tier,
            "rank": r.and_then(|r| r.rank),
            "origin": f.origin.as_str(),
            "site": f.site,
            "holder": f.holder.map(|h| h.to_string()),
            "owner": f.owner,
            "writers": ann.map_or(0, |a| a.writers.len()),
            "removers": ann.map_or(0, |a| a.removers.len()),
            "queue": qs,
        });
        if !f.also_origins.is_empty() {
            e["also_origins"] = json!(f.also_origins.iter().map(|o| o.as_str()).collect::<Vec<_>>());
        }
        let targeted = ix.targeted_in(id);
        if !targeted.is_empty() {
            let mut m = serde_json::Map::new();
            for (h, v) in targeted {
                m.insert(h, json!(v.map(Verdict::as_str)));
            }
            e["harnesses"] = Value::Object(m);
        }
        if let Some(Standing::Settled(why)) = standing {
            e["settled_by"] = json!(match why {
                Why::Moved => "growth",
                Why::Executed => "executed",
                Why::TwoDrivers => "two_drivers",
            });
        }
        lines.push((f.state.as_str().to_owned(), e));
    }
    let total = lines.len();
    let limit = if a.all {
        usize::MAX
    } else {
        a.limit.unwrap_or(DEFAULT_LIMIT)
    };
    let eligible = ranked.iter().filter(|r| r.eligible()).count();
    let mut totals = json!({
        "totals": {
            "findings": view.ledger.findings().count(),
            "listed": total.min(limit),
            "matching": total,
            "ranked": ranked.len(),
            "eligible": eligible,
            "queue": queues.iter().map(|q| (q.runtime.clone(), json!({
                "eligible": q.eligible, "settled": q.settled, "pending": q.pending.len(),
                "open": q.open(), "closed": q.closed,
            }))).collect::<serde_json::Map<_, _>>(),
        }
    });
    if total > limit {
        totals["totals"]["more"] = json!(format!("{} more: dla ledger list --all", total - limit));
    }
    let mut out = totals.to_string();
    for (_, e) in lines.into_iter().take(limit) {
        out.push('\n');
        out.push_str(&e.to_string());
    }
    Ok(out)
}

/// The remedy `close-rest` builds from a finding's history.
fn default_remedy(f: &Finding, r: Option<&queue::Ranked>, ix: &Index, runtime: Option<&str>) -> String {
    let place = f
        .owner
        .as_deref()
        .filter(|o| !o.is_empty())
        .or(f.site.as_deref())
        .unwrap_or("its holder");
    let targeted = ix.targeted_in(&f.id);
    if targeted.is_empty() {
        let lang = runtime.map_or_else(String::new, |rt| format!(" --lang {rt}"));
        return match r {
            Some(r) if r.eligible() => format!(
                "ranked {} (tier {}), not targeted: dla harness gen{lang} --driver --probe {}",
                r.rank.unwrap_or(0),
                r.tier,
                f.id
            ),
            Some(r) => format!(
                "tier {} (not ranked for measurement), not targeted: dla harness gen{lang} --driver --probe {} if it matters",
                r.tier, f.id
            ),
            None => format!(
                "not targeted: dla harness gen{lang} --driver --probe {}",
                f.id
            ),
        };
    }
    let names: Vec<String> = targeted.iter().map(|(h, _)| h.clone()).collect();
    let list = match names.as_slice() {
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        more => format!("{} and {}", more[..more.len() - 1].join(", "), more[more.len() - 1]),
    };
    let best = ix.best_verdict(&f.id);
    let outcome = match best {
        Some(Verdict::Constant) => "its holder did not change".to_owned(),
        Some(Verdict::NotLoaded) => "its probe never loaded: the workload did not import its module".to_owned(),
        Some(Verdict::NotExecuted) => "its code did not run".to_owned(),
        Some(Verdict::Executed | Verdict::Unknown) => "its code ran and showed no growth".to_owned(),
        Some(Verdict::Changed) => "its holder changed but did not pass the growth gate".to_owned(),
        Some(Verdict::Grew) => "its holder grew".to_owned(),
        None => "no run of it was read with dla trend".to_owned(),
    };
    format!("targeted in {list}, {outcome}: write a unit that adds a new key to {place}")
}

/// What `close-rest` did with one finding.
struct Closed1 {
    id: String,
    to: &'static str,
    reason: UnresolvedReason,
}

/// `dla ledger close-rest`: dispositions every open candidate that is not pending in an open
/// queue, through the rules of `dla ledger resolve`.
pub fn close_rest(ctx: &Ctx, a: CloseRestArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let view = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
    let ranked = queue::ranked(&view);
    let by_id: std::collections::BTreeMap<&FindingId, &queue::Ranked> =
        ranked.iter().map(|r| (&r.id, r)).collect();
    let ix = Index::of(&view);
    let queues = queue::queues(&view, ctx.worker.as_deref());
    let mut only: Option<Option<String>> = None;
    let mine = |f: &Finding| {
        ctx.worker
            .as_deref()
            .is_none_or(|w| f.id.prefix() == Some(w))
    };
    let candidates: Vec<Finding> = view
        .ledger
        .findings()
        .filter(|f| f.state == FindingState::Candidate)
        .filter(|f| view.ledger.merged_into(&f.id).is_none())
        .filter(|f| mine(f))
        .cloned()
        .collect();
    let mut closed: Vec<Closed1> = Vec::new();
    let mut left: Vec<(String, String)> = Vec::new();
    for f in &candidates {
        let runtime = dla_core::gate::finding_runtime_with(&view, f, &mut only);
        let q = runtime
            .as_deref()
            .and_then(|rt| queues.iter().find(|q| q.runtime == rt));
        // Pending in an open queue: it waits for a measurement.
        if q.is_some_and(|q| q.open() && q.pending.contains(&f.id)) {
            let cmd = q.map(|q| q.command.clone()).unwrap_or_default();
            left.push((f.id.to_string(), format!("pending in the {} queue: {cmd}", runtime.unwrap_or_default())));
            continue;
        }
        // A candidate whose column grew is a lead to test, not one to close.
        if ix.best_verdict(&f.id) == Some(Verdict::Grew) {
            left.push((
                f.id.to_string(),
                format!("its column grew: dla ledger test {}", f.id),
            ));
            continue;
        }
        let static_evidence = f.supports.iter().filter_map(|e| view.ledger.evidence_by_id(e)).any(
            |e| matches!(e.kind, EvidenceKind::StaticHit | EvidenceKind::CensusHolder),
        );
        let (to, disposition) = if static_evidence {
            ("suspected_static_only", ResolveAs::SuspectedStaticOnly)
        } else {
            ("unresolved", ResolveAs::Unresolved)
        };
        // The queue closed because the budget reached the floor: that is what time_budget says.
        let reason = if q.is_some_and(|q| q.closed == Some(Closed::Floor)) {
            UnresolvedReason::TimeBudget
        } else {
            UnresolvedReason::NotExercised
        };
        let remedy = a.remedy.clone().unwrap_or_else(|| {
            default_remedy(f, by_id.get(&f.id).copied(), &ix, runtime.as_deref())
        });
        let args = ResolveArgs {
            target: f.id.to_string(),
            disposition: Some(disposition),
            reason: Some(reason),
            remedy: Some(remedy),
            by: None,
            status: None,
            evidence: Vec::new(),
            note: None,
        };
        match resolve_finding(ctx, &mut ledger, f.id.clone(), args) {
            Ok(_) => closed.push(Closed1 {
                id: f.id.to_string(),
                to,
                reason,
            }),
            Err(e) => left.push((f.id.to_string(), e.refusal.message.clone())),
        }
    }
    let count = |to: &str| closed.iter().filter(|c| c.to == to).count();
    let by_reason = |r: UnresolvedReason| closed.iter().filter(|c| c.reason == r).count();
    // One run-config record lists what was closed; the full list is a file.
    let mut record = Value::Null;
    if !closed.is_empty() {
        let n = ledger.lines().len() + 1;
        let rel = format!("queue/close-rest-{n}.json");
        let path = ctx.run_dir.join(&rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| dla_core::Error::io(dir, e))?;
        }
        let full = json!({
            "closed": closed.iter().map(|c| json!({"id": c.id, "as": c.to, "reason": c.reason.as_str()})).collect::<Vec<_>>(),
            "left_open": left.iter().map(|(i, w)| json!({"id": i, "why": w})).collect::<Vec<_>>(),
        });
        std::fs::write(&path, serde_json::to_string_pretty(&full).unwrap_or_default() + "\n")
            .map_err(|e| dla_core::Error::io(&path, e))?;
        let project = ctx
            .run_dir
            .parent()
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf);
        let commit = project
            .as_deref()
            .and_then(crate::git::head_commit)
            .unwrap_or_else(|| "unknown".to_owned());
        let ids: Vec<&str> = closed.iter().take(40).map(|c| c.id.as_str()).collect();
        let line = ledger.add_evidence(NewEvidence {
            source: Source::Repo { commit, path: None },
            tool: Tool {
                name: "dla".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            authority: Authority::Static,
            kind: EvidenceKind::RunConfig,
            result: json!({
                "close_rest": {
                    "closed": closed.len(),
                    "suspected_static_only": count("suspected_static_only"),
                    "unresolved": count("unresolved"),
                    "not_exercised": by_reason(UnresolvedReason::NotExercised),
                    "time_budget": by_reason(UnresolvedReason::TimeBudget),
                    "left_open": left.len(),
                    "ids": ids.join(","),
                },
            }),
            raw: Some(rel),
            coverage: EvidenceCoverage {
                status: CoverageStatus::Complete,
                examined: None,
                total: None,
            },
            limitations: Vec::new(),
            unknowns: Vec::new(),
        })?;
        record = json!(super::pointer(ctx, line.seq));
    }
    let mut out = json!({
        "ok": true,
        "closed": closed.len(),
        "as": {
            "suspected_static_only": count("suspected_static_only"),
            "unresolved": count("unresolved"),
        },
        "reason": {
            "not_exercised": by_reason(UnresolvedReason::NotExercised),
            "time_budget": by_reason(UnresolvedReason::TimeBudget),
        },
        "left_open": left.len(),
        "left": left.iter().take(8).map(|(i, w)| json!({"id": i, "why": w})).collect::<Vec<_>>(),
        "record": record,
    });
    if let Some(q) = queues.iter().find(|q| q.open()) {
        out["batch_command"] = json!(q.command);
    }
    if left.len() > 8 {
        out["left_more"] = json!(left.len() - 8);
    }
    Ok(out)
}
