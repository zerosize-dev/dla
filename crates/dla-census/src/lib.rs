//! Census of process-lifetime holders, and the tree-sitter parsing layer.
//!
//! # Contract
//!
//! One entry point, [`census`]. It does its own writes through the [`ToolCtx`] (design
//! and shared rules: `dla_core::toolctx`) and returns a bounded [`StageOutcome`]; the
//! CLI (`census_cmd.rs`) only parses arguments and prints.
//!
//! Writes: the full list of `dla_core::census::CensusEntry` as
//! `.dla/<run>/census/<lang>.json`, ONE `census-holder` evidence record
//! (`raw` = that file), one candidate per holder (origin `census`, `holder`, `owner`,
//! `region`, `site`; `CandidateWriter` skips repeated sites), and for each holder with
//! writers or removers a `Ledger::annotate` line so the report lists them. The census
//! never reads archetype data or known-leak tables, and never sets
//! `suggested_archetype`: the candidate set depends on the project's sources (and
//! class files) alone.
//!
//! # How holders are found
//!
//! Per language, tree-sitter syntax trees are walked for declarations that can keep
//! objects, threads or handles alive: statics and module globals, collection fields,
//! registration tables, spawned units, acquired handles, thread-locals, stored
//! closures and (Rust) `Rc` cycles. For the JVM, class files under the project are
//! read first by a small reader ([`classfile`]); the source files they cover are not
//! parsed again. Writers and removers are textual name matches over the project
//! ([`usage`]) and are labelled heuristic.
//!
//! [`parse`] is the shared tree-sitter layer (`Lang`, `parse`, `captures`) also used
//! by the DLA rule backend and workload discovery.

pub mod ast;
pub mod classfile;
pub mod jvm_class;
pub mod lang;
pub mod model;
pub mod parse;
pub mod statement;
pub mod usage;
pub mod vocab;
pub mod walk;

use std::{collections::BTreeMap, fs};

use dla_core::{
    CoverageManifest, EvidenceId, Refusal,
    census::CensusEntry,
    coverage::BackendCoverage,
    error::Error,
    evidence::{CoverageStatus, EvidenceCoverage, EvidenceKind},
    runtime::Runtime,
    toolctx::{Candidate, CandidateWriter, EvidenceSpec, StageOutcome, ToolCtx},
};
use serde_json::{Value, json};

use crate::{
    lang::Cx,
    model::{ProjectInfo, Raw, SourceFile},
    parse::{Lang, parse as parse_source},
};

/// Tool name recorded in evidence.
pub const TOOL: &str = "dla-census";

/// Enumerates every static, global, registry, spawned unit and acquired handle of the
/// `lang` runtime under `ctx.project` and records them as candidates.
///
/// For the JVM, class files under the project are read when present and tree-sitter is
/// the fallback.
pub fn census(ctx: &ToolCtx, lang: Runtime) -> Result<StageOutcome, Refusal> {
    let scope = ctx.scope()?;
    let inside = |rel: &str| scope.as_ref().is_none_or(|s| s.includes(rel));
    let (files, out_of_scope) = walk::source_files_in(&ctx.project, lang, &inside);
    let mut found = enumerate(ctx, lang, &files);
    // Holders read from class files of a source that lies outside the scope.
    found.raws.retain(|r| inside(&r.file));
    let usages = usage::find(&found.raws, &files);
    let entries: Vec<CensusEntry> = found
        .raws
        .iter()
        .zip(&usages)
        .map(|(r, u)| CensusEntry {
            kind: r.kind.clone(),
            holder: r.holder,
            region: r.region,
            site: format!("{}:{}", r.file, r.line),
            owner: r.owner.clone(),
            writers: u.writers.clone(),
            removers: u.removers.clone(),
        })
        .collect();

    let stage = ctx.stage_dir("census")?;
    let raw_name = format!("census/{}.json", lang.as_str());
    let by_holder = count_by(entries.iter().map(|e| e.holder.as_str().to_owned()));
    let by_kind = count_by(entries.iter().map(|e| e.kind.clone()));
    let full = json!({
        "lang": lang.as_str(),
        "source": found.source,
        "files_scanned": files.iter().filter(|f| !f.is_test).count(),
        "class_files": found.classes,
        "class_files_unreadable": found.unreadable,
        "usage": "writers and removers are heuristic name matches, not an analysis",
        "by_holder": by_holder,
        "by_kind": by_kind,
        "entries": entries,
    });
    let path = stage.join(format!("{}.json", lang.as_str()));
    let body = serde_json::to_vec_pretty(&full).unwrap_or_default();
    fs::write(&path, body).map_err(|e| Error::io(&path, e).to_refusal())?;

    let mut ledger = ctx.ledger()?;
    let mut top_kinds: Vec<(&String, &u64)> = by_kind.iter().collect();
    top_kinds.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    top_kinds.truncate(10);
    let mut spec = EvidenceSpec::new(
        EvidenceKind::CensusHolder,
        TOOL,
        env!("CARGO_PKG_VERSION"),
        json!({
            "lang": lang.as_str(),
            "source": found.source,
            "holders": entries.len(),
            "by_holder": by_holder,
            "top_kinds": top_kinds.iter().map(|(k, v)| json!({"kind": k, "n": v})).collect::<Vec<Value>>(),
        }),
    );
    spec.raw = Some(raw_name.clone());
    spec.coverage = EvidenceCoverage {
        status: CoverageStatus::Partial,
        examined: Some(u64::try_from(files.len()).unwrap_or(u64::MAX)),
        total: None,
    };
    spec.limitations = vec![
        "declaration patterns only: holders created through reflection, generated code or dependencies are not listed".into(),
        "writers and removers are textual name matches".into(),
    ];
    let evidence: EvidenceId = ctx.add_evidence(&mut ledger, spec)?;

    let mut writer = CandidateWriter::for_ctx(ctx, &ledger)?;
    let mut annotated = 0u64;
    for e in &entries {
        let id = writer.add(
            &mut ledger,
            Candidate {
                origin: "census".into(),
                evidence: vec![evidence.clone()],
                site: e.site.clone(),
                holder: Some(e.holder),
                owner: Some(e.owner.clone()),
                region: Some(e.region),
                suggested_archetype: None,
            },
        )?;
        if let Some(id) = id
            && (!e.writers.is_empty() || !e.removers.is_empty())
        {
            ledger
                .annotate(
                    &id,
                    e.writers.clone(),
                    e.removers.clone(),
                    Some("heuristic name match over the project's sources".into()),
                )
                .map_err(|er| er.to_refusal())?;
            annotated += 1;
        }
    }

    let enumerated = u64::try_from(entries.len()).unwrap_or(u64::MAX);
    record_out_of_scope(ctx, lang, out_of_scope)?;
    let duplicates = writer.skipped();
    CoverageManifest::update(&ctx.run_dir, |m| {
        let mut cov = BackendCoverage::ran(Some(env!("CARGO_PKG_VERSION").to_owned()), enumerated);
        cov.duplicates = duplicates;
        m.set_backend(lang.as_str(), "census", cov);
    })
    .map_err(|e| e.to_refusal())?;

    Ok(StageOutcome {
        evidence: vec![evidence.to_string()],
        candidates: writer.added(),
        skipped_duplicates: writer.skipped(),
        artifacts: vec![ctx.pointer(&raw_name)],
        summary: with_out_of_scope(
            json!({
                "lang": lang.as_str(),
                "source": found.source,
                "holders": enumerated,
                "by_holder": by_holder,
                "annotated": annotated,
                "files": files.iter().filter(|f| !f.is_test).count(),
                "class_files": found.classes,
            }),
            out_of_scope,
        ),
    })
}

/// Records in `coverage.json` that `n` source files of `runtime` lie outside the scope.
/// Every stage sees the same files, so the larger count wins instead of the counts
/// adding up (a second stage or a re-run never double counts).
pub fn record_out_of_scope(ctx: &ToolCtx, runtime: Runtime, n: u64) -> Result<(), Refusal> {
    if n == 0 {
        return Ok(());
    }
    CoverageManifest::update(&ctx.run_dir, |m| {
        let w = m.worker_mut(runtime.as_str());
        w.out_of_scope = w.out_of_scope.max(n);
    })
    .map(|_| ())
    .map_err(|e| e.to_refusal())
}

/// Adds `out_of_scope` to a summary object when it is non-zero.
pub fn with_out_of_scope(mut v: Value, n: u64) -> Value {
    if n > 0
        && let Value::Object(m) = &mut v
    {
        m.insert("out_of_scope".into(), json!(n));
    }
    v
}

fn count_by(it: impl Iterator<Item = String>) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for k in it {
        *m.entry(k).or_insert(0) += 1;
    }
    m
}

struct Found {
    raws: Vec<Raw>,
    source: &'static str,
    classes: usize,
    unreadable: usize,
}

/// Sorts the holders and drops repeats of one declaration. Coroutine captures share the
/// suspension point line and holder class, so their owner (the spilled field) is part
/// of the identity.
///
/// Of several holders on one line the one owned by a field wins over one owned by a
/// compiler-named initializer (`Foo.<clinit>`), whatever the names sort like.
fn sort_dedup(raws: &mut Vec<Raw>) {
    let synthetic = |r: &Raw| r.owner.ends_with(".<clinit>") || r.owner.ends_with(".<init>");
    raws.sort_by(|a, b| {
        (&a.file, a.line, a.holder.as_str(), synthetic(a), &a.owner).cmp(&(
            &b.file,
            b.line,
            b.holder.as_str(),
            synthetic(b),
            &b.owner,
        ))
    });
    raws.dedup_by(|b, a| {
        a.file == b.file
            && a.line == b.line
            && a.holder == b.holder
            && (a.kind != "coroutine-capture" || a.owner == b.owner)
    });
}

fn enumerate(ctx: &ToolCtx, runtime: Runtime, files: &[SourceFile]) -> Found {
    let mut raws: Vec<Raw> = Vec::new();
    let mut covered = std::collections::HashSet::new();
    let (mut classes, mut unreadable) = (0, 0);
    let mut source = "tree-sitter";
    if runtime == Runtime::Jvm {
        let c = jvm_class::holders(&ctx.project, files);
        if c.classes > 0 {
            source = "class-files+tree-sitter";
            raws.extend(c.raws);
            covered = c.covered;
        }
        classes = c.classes;
        unreadable = c.unreadable;
    }
    let mut info = ProjectInfo::default();
    for f in files {
        match f.lang {
            Lang::Java => info.selfref.extend(lang::java::selfref(&f.text)),
            Lang::Go => {
                info.selfref.extend(lang::go::selfref(&f.text));
                lang::go::collect(&f.text, &mut info);
            }
            Lang::Python => lang::python::collect(&f.text, &mut info),
            Lang::Rust => lang::rust::collect(&f.text, &mut info),
            _ => {}
        }
    }
    info.finish();
    for f in files
        .iter()
        .filter(|f| !f.is_test && !covered.contains(&f.rel))
    {
        let mut cx = Cx::new(f, &info);
        if f.lang == Lang::Kotlin {
            lang::kotlin::holders(&mut cx);
        } else {
            let Ok(tree) = parse_source(f.lang, &f.text) else {
                continue;
            };
            let root = tree.root_node();
            match f.lang {
                Lang::Java => lang::java::holders(&mut cx, root),
                Lang::Go => lang::go::holders(&mut cx, root),
                Lang::Rust => lang::rust::holders(&mut cx, root),
                Lang::JavaScript | Lang::TypeScript | Lang::Tsx => {
                    lang::node::holders(&mut cx, root)
                }
                Lang::Python => lang::python::holders(&mut cx, root),
                Lang::C | Lang::Cpp => lang::native::holders(&mut cx, root),
                Lang::Kotlin => {}
            }
        }
        raws.extend(cx.out);
    }
    sort_dedup(&mut raws);
    Found {
        raws,
        source,
        classes,
        unreadable,
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dedup_tests {
    use super::*;
    use crate::model::SourceFile;
    use crate::parse::Lang;

    #[test]
    fn coroutine_captures_at_one_suspension_point_all_survive() {
        let t = tempfile::tempdir().unwrap();
        let fx = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/kotlin_metadata/LeakyKt$handle$1.class");
        let dir = t.path().join("target/classes/shop");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(fx, dir.join("LeakyKt$handle$1.class")).unwrap();
        let src = SourceFile {
            rel: "src/shop/Leaky.kt".into(),
            text: "package shop\n".into(),
            lang: Lang::Kotlin,
            is_test: false,
        };
        let mut raws = jvm_class::holders(t.path(), std::slice::from_ref(&src)).raws;
        sort_dedup(&mut raws);
        let owners: Vec<&str> = raws
            .iter()
            .filter(|r| r.kind == "coroutine-capture")
            .map(|r| r.owner.as_str())
            .collect();
        assert_eq!(
            owners,
            ["shop.LeakyKt$handle$1.L$0", "shop.LeakyKt$handle$1.L$1"]
        );
    }
}
