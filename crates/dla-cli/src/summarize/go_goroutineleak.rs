//! `non-heap` over the Go `goroutineleak` profile (`pprof.Lookup("goroutineleak")` at
//! `debug=2`), a `should-be-dead` observation.
//!
//! The runtime's own collector proves that a reported goroutine is blocked on a
//! synchronization object nothing reachable can wake, so it can never run again. That is a
//! second signal next to the goroutine growth column (class `gc-reachability`), not a
//! replacement for it:
//!
//! * It never confirms a finding alone: a confirmation still needs a passing growth column
//!   tied to the finding. A group whose creation site is a finding's site or recorded writer
//!   is listed under that finding, to be cited next to the growth evidence.
//! * It never refutes. A goroutine held by a global, or one that is runnable, is not
//!   reported, so an empty profile or a missing group proves nothing.
//!
//! One evidence per run: the total and up to five groups by creation site of the last file
//! given (earlier files give their totals in the raw file).

use dla_core::{
    FindingId,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, tool_class},
    runtime::Runtime,
    toolctx::EvidenceSpec,
};
use dla_parsers::go::dump::{self, CreationGroup};
use serde_json::{Map, json};

use super::{
    Adapter, AdapterOut, CandidateRow, Ctx, Op, Opts, emit, go_goroutine::is_leak_profile,
    go_pprof_heap::root_file, input::Input, tie,
};
use crate::fail::{Failure, Res};

/// Largest profile text read.
const MAX_BYTES: u64 = 128 << 20;
/// Groups shown in the evidence result.
const SHOWN: usize = 5;
/// Groups kept in the raw file.
const RAW_GROUPS: usize = 200;
/// Longest function name shown.
const NAME_CLIP: usize = 56;

const NOTE: &str = "the goroutineleak profile only lists goroutines blocked on objects nothing reachable can wake; a goroutine held by a global or one that is runnable is not reported, so an empty profile or a missing group never refutes a finding";

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push_str("...");
        t
    }
}

fn frame_at(f: &Option<String>, l: Option<u32>) -> Option<String> {
    f.as_ref().map(|f| match l {
        Some(l) => format!("{f}:{l}"),
        None => f.clone(),
    })
}

/// The goroutineleak profile reader.
pub struct GoGoroutineLeak;

impl Adapter for GoGoroutineLeak {
    fn name(&self) -> &'static str {
        "go-goroutineleak"
    }
    fn op(&self) -> Op {
        Op::NonHeap
    }
    fn runtime(&self) -> &'static str {
        "go"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::GC_REACHABILITY
    }
    fn formats(&self) -> &'static str {
        "Go goroutineleak profile text (pprof.Lookup(\"goroutineleak\").WriteTo(f, 2); Go 1.27, or 1.26 with GOEXPERIMENT=goroutineleakprofile)"
    }
    fn sniff(&self, input: &Input) -> bool {
        is_leak_profile(input) && !input.head.starts_with(&[0x1f, 0x8b])
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let tc = ctx.tc;
        let red = &ctx.redactor;
        // (label, sha, total, groups of the leaked goroutines)
        let mut files: Vec<(String, String, u64, Vec<CreationGroup>)> = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let d = dump::parse(&text).map_err(|e| {
                Failure::refuse(
                    "malformed-artifact",
                    format!("{}: {}", i.shown(), e.message()),
                    "the text pprof.Lookup(\"goroutineleak\").WriteTo(f, 2) writes",
                    "dla harness run --capture nonheap   (Go 1.27, or `dla install go-toolchain-1.27`), and pass the goroutineleak-<n> files it writes",
                )
            })?;
            let total = d.goroutines.iter().filter(|g| g.leaked).count() as u64;
            let mut groups = dump::group_by_creator(&d.goroutines, true);
            // A leaked goroutine without a creator (rare: main) is counted, not grouped.
            for g in &mut groups {
                if let Some(f) = &g.file {
                    g.file = Some(root_file(&tc.project, f));
                }
                if let Some(b) = g.blocked.as_mut()
                    && let Some(f) = &b.file
                {
                    b.file = Some(root_file(&tc.project, f));
                }
            }
            files.push((i.shown().to_owned(), i.sha256.clone(), total, groups));
        }
        let Some((_, _, total, groups)) = files.last() else {
            return Err(Failure::refuse(
                "not-enough-captures",
                "no goroutineleak profile in the input".to_owned(),
                "1 or more goroutineleak profiles",
                "dla summarize non-heap <goroutineleak file>",
            ));
        };
        let total = *total;

        let mut ledger_ref = ctx.ledger()?;
        let ledger = &mut *ledger_ref;
        let who = tie::Who::new(tc.worker.as_deref(), Some(Runtime::Go));
        // The creation site of each group, and the findings that own that site.
        let rows: Vec<(&CreationGroup, Option<String>, Vec<FindingId>)> = groups
            .iter()
            .map(|g| {
                let site = if ctx.artifact_run {
                    None
                } else {
                    ctx.resolver()
                        .frame_site(&g.function, g.file.as_deref(), g.line)
                };
                let mut found = site
                    .as_deref()
                    .map(|s| tie::findings_at_site(ledger, &who, s))
                    .unwrap_or_default();
                if let Some(only) = &opts.finding {
                    found.retain(|f| f == only);
                }
                (g, site, found)
            })
            .collect();
        let tied: Vec<FindingId> = {
            let mut v: Vec<FindingId> = rows.iter().flat_map(|r| r.2.clone()).collect();
            v.sort();
            v.dedup();
            v
        };

        // ---- raw
        let (nid, nrel) = emit::next_nonheap(tc)?;
        let group_json =
            |g: &CreationGroup, site: &Option<String>, f: &[FindingId], short: bool| {
                let mut v = json!({
                    "created_by": clip(&red.name(&g.function), if short { 40 } else { NAME_CLIP }),
                    "count": g.count,
                });
                if let Some(s) = site {
                    v["site"] = json!(s);
                } else if let Some(at) = frame_at(&g.file, g.line)
                    && !at.starts_with('/')
                    && !at.starts_with('$')
                {
                    v["at"] = json!(red.name(&at));
                }
                if let Some(b) = &g.blocked
                    && let Some(at) = frame_at(&b.file, b.line)
                    && !at.starts_with('/')
                {
                    v["blocked_at"] = json!(red.name(&at));
                }
                if !g.wait.is_empty() {
                    v["wait_reason"] = json!(red.name(&g.wait));
                }
                if !f.is_empty() {
                    v["findings"] = json!(f);
                }
                v
            };
        let raw_value = json!({
            "id": nid,
            "tool": self.name(),
            "kind": "should-be-dead",
            "files": files.iter().map(|(l, s, t, g)| json!({
                "label": l, "sha256": s, "total": t, "groups": g.len(),
            })).collect::<Vec<_>>(),
            "total": total,
            "groups": rows.iter().take(RAW_GROUPS)
                .map(|(g, s, f)| group_json(g, s, f, false)).collect::<Vec<_>>(),
            "tied": tied,
            "note": NOTE,
            "redaction": red.mode(),
        });
        let raw = emit::write_raw(tc, &nrel, &raw_value)?;

        // ---- the should-be-dead evidence
        let shas: Vec<&str> = files.iter().map(|f| f.1.as_str()).collect();
        let signal = Signal::new(self.name(), self.tool_class(), &emit::combined_sha(&shas));
        let mut base = Map::new();
        base.insert("total".into(), json!(total));
        base.insert("profiles".into(), json!(files.len()));
        base.insert("sites".into(), json!(groups.len()));
        if !tied.is_empty() {
            base.insert("tied".into(), json!(tied));
        }
        let mut list = emit::Bounded::new(base, "groups", emit::RESULT_BUDGET - 120);
        for (g, s, f) in rows.iter().take(SHOWN) {
            list.push(group_json(g, s, f, false));
        }
        let mut spec = EvidenceSpec::new(
            EvidenceKind::ShouldBeDead,
            self.name(),
            env!("CARGO_PKG_VERSION"),
            list.finish(),
        );
        spec.authority = ctx.authority(Authority::Live);
        spec.source = Some(tc.artifact_source(&nrel)?);
        spec.coverage = EvidenceCoverage {
            status: CoverageStatus::Complete,
            examined: Some(total),
            total: None,
        };
        spec.limitations = vec![
            NOTE.into(),
            "a second signal (gc-reachability): it never confirms a finding by itself, the tied goroutine growth column does".into(),
        ];
        let ev = ctx.record(ledger, spec, &signal, &raw)?;

        let mut out = AdapterOut {
            raw: Some(raw.clone()),
            enumerated: total,
            ..AdapterOut::default()
        };
        for f in &tied {
            let fnd = ledger.finding(f);
            out.candidates.push(CandidateRow {
                finding: f.clone(),
                owner: fnd.and_then(|x| x.owner.clone()),
                holder: fnd.and_then(|x| x.holder),
                site: fnd.and_then(|x| x.site.clone()),
                region: fnd.and_then(|x| x.region),
                origin: "should-be-dead".into(),
                added: false,
                via: None,
            });
        }
        let mut body = Map::new();
        body.insert("total".into(), json!(total));
        body.insert("profiles".into(), json!(files.len()));
        body.insert(
            "groups".into(),
            json!(
                rows.iter()
                    .take(opts.top.clamp(1, 3))
                    .map(|(g, s, f)| group_json(g, s, f, true))
                    .collect::<Vec<_>>()
            ),
        );
        body.insert("tied".into(), json!(tied));
        body.insert("redaction".into(), json!(red.mode()));
        let needed = if total == 0 {
            "an empty goroutineleak profile proves nothing (globals and runnable goroutines are not reported): rely on the goroutine growth column"
                .to_owned()
        } else if let Some(f) = tied.first() {
            format!(
                "cite {} next to the tied goroutine growth evidence in dla ledger confirm {f}; it never confirms alone",
                ev.id
            )
        } else {
            "no finding has a creation site equal to these groups: run dla census, or dla summarize non-heap on 3 or more goroutine dumps, so a finding exists at the go statement"
                .to_owned()
        };
        body.insert("needed".into(), json!(needed));
        out.evidence.push(ev);
        out.body = body;
        Ok(out)
    }
}
