//! Which evidence records show a holder, and which interventions removed growth.
//!
//! [`names_holder`] decides whether a retention-class record names the holder of a
//! finding. It is stricter than the evidence class: `histogram-diff` is retention class
//! (it lists types that grow) and never names a holder. `dla ledger strong` and
//! `dla ledger label` use it as the tie of a retention record to a finding.
//!
//! Layouts read (the `result` of each producer):
//!
//! * `dla-hprof` `retention-path`: `holders[].key` and `via[].key`
//!   (`static:<Class>.<field>`, `field:<Decl>.<field>`, `thread-local:<Class>.<field>`).
//! * `dla-v8-snapshot` `retention-path`: `holders[].key` (`static:<module>.<name>`).
//! * `valgrind-vgdb` `retention-path`: `alloc` (`function (file:line)`), `root` and `path[]`.
//! * `alloc-stack`: `top[].root` (the root holder of a sampled path, JFR OldObjectSample).
//!
//! The layout decides, never the tool name, so a record of a new tool with one of these
//! layouts is read the same way.

use serde_json::Value;

use crate::{
    attribution::{canonical, owners_match},
    evidence::{Authority, EvidenceKind, EvidenceRecord},
    finding::{Finding, Holder},
    ids::FindingId,
    trend::GrowthSummary,
    verify::SITE_TOLERANCE,
};

/// The canonical owner (`pkg.Class.field`) a holder key names, or `None` for a key that
/// names no field (`thread:main`, `jni-global:...`).
fn key_owner(key: &str) -> Option<String> {
    let k = key.trim();
    // `field:<Decl>.<f>` is an instance field; `thread-local:<Class>.<f>` is the static
    // field whose `ThreadLocal` the values hang from (dla-hprof writes it for the values
    // held by live threads). Both name the field, as `static:` does.
    let k = k
        .strip_prefix("field:")
        .or_else(|| k.strip_prefix("thread-local:"))
        .unwrap_or(k);
    canonical(k)
}

fn strings<'a>(v: &'a Value, key: &str) -> impl Iterator<Item = &'a str> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

/// The `key` of each object in `result[list]`.
fn list_keys<'a>(result: &'a Value, list: &str) -> impl Iterator<Item = &'a Value> {
    result
        .get(list)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn entry_names_owner(entry: &Value, owner: &str) -> bool {
    entry
        .get("key")
        .and_then(Value::as_str)
        .and_then(key_owner)
        .is_some_and(|k| owners_match(&k, owner))
}

/// True when `needle` occurs in `hay` between non-identifier characters.
fn contains_word(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut from = 0;
    while let Some(i) = hay[from..].find(needle) {
        let start = from + i;
        let end = start + needle.len();
        let before = hay[..start].chars().next_back();
        let after = hay[end..].chars().next();
        if !before.is_some_and(ident) && !after.is_some_and(ident) {
            return true;
        }
        from = end;
    }
    false
}

/// The last dotted component of an owner (`frame_table` of `decoder.frame_table`).
fn owner_tail(owner: &str) -> &str {
    owner.rsplit(['.', ':', '/']).next().unwrap_or(owner)
}

fn norm_path(p: &str) -> String {
    p.replace('\\', "/")
}

/// True when two source paths name one file: equal, or one is a trailing run of path
/// components of the other (valgrind reports a basename, the finding a project path).
fn same_file(a: &str, b: &str) -> bool {
    let (a, b) = (norm_path(a), norm_path(b));
    a == b
        || a.strip_suffix(&b).is_some_and(|r| r.ends_with('/'))
        || b.strip_suffix(&a).is_some_and(|r| r.ends_with('/'))
}

fn split_site(site: &str) -> Option<(&str, u32)> {
    let (file, line) = site.rsplit_once(':')?;
    Some((file, line.parse().ok()?))
}

/// The `file:line` in `function (file:line)`.
fn alloc_site(alloc: &str) -> Option<(String, u32)> {
    let inner = alloc.rsplit_once('(')?.1.strip_suffix(')')?;
    let (file, line) = split_site(inner)?;
    Some((file.to_owned(), line))
}

fn vgdb_names(result: &Value, finding: &Finding) -> bool {
    if let (Some(alloc), Some(site)) = (
        result.get("alloc").and_then(Value::as_str),
        finding.site.as_deref(),
    ) && let (Some((af, al)), Some((sf, sl))) = (alloc_site(alloc), split_site(site))
        && same_file(&af, sf)
        && al.abs_diff(sl) <= SITE_TOLERANCE
    {
        return true;
    }
    let Some(owner) = finding.owner.as_deref() else {
        return false;
    };
    let tail = owner_tail(owner);
    if tail.len() < 3 {
        return false;
    }
    result
        .get("root")
        .and_then(Value::as_str)
        .into_iter()
        .chain(strings(result, "path"))
        .any(|s| contains_word(s, tail))
}

/// True when `record` shows a holder of `finding`: a retention record whose holder key
/// (or via key, or sampled root) is the finding's owner, or whose allocation site is the
/// finding's site. False for every other kind, for a record read from source text
/// (authority `static`) and for `histogram-diff`.
pub fn names_holder(record: &EvidenceRecord, finding: &Finding) -> bool {
    if record.authority == Authority::Static {
        return false;
    }
    let r = &record.result;
    match record.kind {
        EvidenceKind::RetentionPath => {
            if let Some(owner) = finding.owner.as_deref()
                && ["holders", "via"]
                    .iter()
                    .any(|l| list_keys(r, l).any(|e| entry_names_owner(e, owner)))
            {
                return true;
            }
            vgdb_names(r, finding)
        }
        EvidenceKind::AllocStack => finding.owner.as_deref().is_some_and(|owner| {
            list_keys(r, "top").any(|row| {
                row.get("root")
                    .and_then(Value::as_str)
                    .and_then(key_owner)
                    .is_some_and(|k| owners_match(&k, owner))
            })
        }),
        _ => false,
    }
}

/// The holder kind a retention record states for the holder it names, when it states one
/// (a `holder_kind` member of the matching `holders` or `via` entry). `dla-hprof` and
/// `dla-v8-snapshot` write it on every holder entry, the same value their growth columns
/// carry; a record of another tool states none and gives `None`. A record that states
/// one must agree with the finding's holder before the finding becomes strong.
pub fn named_holder_kind(record: &EvidenceRecord, finding: &Finding) -> Option<Holder> {
    let owner = finding.owner.as_deref()?;
    ["holders", "via"].iter().find_map(|l| {
        list_keys(&record.result, l)
            .filter(|e| entry_names_owner(e, owner))
            .find_map(|e| e.get("holder_kind")?.as_str()?.parse().ok())
    })
}

/// What an `intervention` record says, read leniently.
#[derive(Debug, Clone, PartialEq)]
pub struct InterventionResult {
    /// The finding the edit tested.
    pub finding: Option<String>,
    /// `growth-removed`, `growth-remains`, `inconclusive` or `invalid`.
    pub effect: Option<String>,
    /// Names of guards that are false.
    pub failed_guards: Vec<String>,
    /// Number of guards present.
    pub guards: usize,
    /// The control trend evidence was written by (`T3`).
    pub control_trend: Option<String>,
    /// The control column.
    pub control_column: Option<String>,
}

impl InterventionResult {
    /// Reads an `intervention` record's `result`; `None` for another kind.
    pub fn of(record: &EvidenceRecord) -> Option<Self> {
        if record.kind != EvidenceKind::Intervention {
            return None;
        }
        let r = &record.result;
        let text = |v: Option<&Value>| v.and_then(Value::as_str).map(str::to_owned);
        let guards = r.get("guards").and_then(Value::as_object);
        Some(Self {
            finding: text(r.get("finding")),
            effect: text(r.get("effect")),
            failed_guards: guards
                .into_iter()
                .flatten()
                .filter(|(_, v)| v.as_bool() == Some(false))
                .map(|(k, _)| k.clone())
                .collect(),
            guards: guards.map_or(0, |g| g.len()),
            control_trend: text(r.pointer("/control/trend")),
            control_column: text(r.pointer("/control/column")),
        })
    }

    /// Why this record cannot stand as evidence that the edit removed the growth of
    /// `finding`; `None` when it can. A guard that is `null` is ignored.
    pub fn why_not_removal_of(&self, finding: &FindingId) -> Option<String> {
        if self.finding.as_deref() != Some(finding.to_string().as_str()) {
            return Some(format!(
                "it tested {}, not {finding}",
                self.finding.as_deref().unwrap_or("no finding")
            ));
        }
        match self.effect.as_deref() {
            Some("growth-removed") => {}
            Some("growth-remains") => {
                return Some(
                    "the stub did not remove the growth: another writer, or this site is not the cause (annotate the missing writer and intervene again)"
                        .to_owned(),
                );
            }
            Some(other) => return Some(format!("its effect is {other}, not growth-removed")),
            None => return Some("it has no effect".to_owned()),
        }
        if self.guards == 0 {
            return Some("it records no guards".to_owned());
        }
        if !self.failed_guards.is_empty() {
            return Some(format!("guard(s) false: {}", self.failed_guards.join(", ")));
        }
        None
    }
}

/// Among `cited` records (the evidence a finding cites), the passing growth record whose
/// trend is `trend` and which has a passing column named `column` tied to `finding`.
pub fn control_record<'a>(
    cited: impl IntoIterator<Item = &'a EvidenceRecord>,
    finding: &Finding,
    trend: &str,
    column: &str,
) -> Option<&'a EvidenceRecord> {
    cited.into_iter().find(|e| {
        crate::machine::is_passing_growth(e)
            && GrowthSummary::from_result(&e.result).is_some_and(|s| {
                s.trend.as_deref() == Some(trend)
                    && s.passed.iter().any(|c| {
                        c.column == column
                            && c.tied_to_owner(
                                &finding.id,
                                finding.site.as_deref(),
                                finding.owner.as_deref(),
                            )
                    })
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        evidence::{EvidenceCoverage, Source, Tool},
        finding::Origin,
        ids::EvidenceId,
    };
    use serde_json::json;

    fn rec(kind: EvidenceKind, authority: Authority, tool: &str, result: Value) -> EvidenceRecord {
        EvidenceRecord {
            evidence_id: EvidenceId::new(1),
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: tool.into(),
                version: "1".into(),
            },
            authority,
            kind,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        }
    }

    fn finding(owner: Option<&str>, site: Option<&str>) -> Finding {
        let mut f = Finding::candidate(
            FindingId::new(1),
            Origin::parse("growth-series").unwrap(),
            vec![EvidenceId::new(1)],
        );
        f.owner = owner.map(str::to_owned);
        f.site = site.map(str::to_owned);
        f
    }

    #[test]
    fn hprof_holders_and_via_name_the_owner() {
        let r = rec(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-hprof",
            json!({"file": "a.hprof", "holders": [
                {"key": "thread:main", "retained": 9, "dominated": 1},
                {"key": "static:shop.codec.Audit.trail", "retained": 5, "dominated": 2}],
                "via": [{"key": "field:shop.cache.Registry.entries", "retained": 3}]}),
        );
        assert!(names_holder(&r, &finding(Some("Audit.trail"), None)));
        assert!(names_holder(
            &r,
            &finding(Some("shop.codec.Audit.trail"), None)
        ));
        assert!(names_holder(&r, &finding(Some("Registry.entries"), None)));
        assert!(!names_holder(&r, &finding(Some("Other.trail"), None)));
        assert!(!names_holder(&r, &finding(None, Some("src/A.java:4"))));
    }

    #[test]
    fn a_thread_local_holder_key_names_its_static_field() {
        let r = rec(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-hprof",
            json!({"holders": [
                {"key": "thread-local:billing.RenderContext.SEGMENTS", "holder_kind": "thread-local"},
                {"key": "thread:pool-1-thread-3"}]}),
        );
        assert!(names_holder(
            &r,
            &finding(Some("billing.RenderContext.SEGMENTS"), None)
        ));
        assert!(names_holder(
            &r,
            &finding(Some("RenderContext.SEGMENTS"), None)
        ));
        assert!(!names_holder(
            &r,
            &finding(Some("billing.RenderContext.OTHER"), None)
        ));
        // A thread holder names no field.
        let t = rec(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-hprof",
            json!({"holders": [{"key": "thread:RenderContext.SEGMENTS"}]}),
        );
        assert!(!names_holder(
            &t,
            &finding(Some("billing.RenderContext.SEGMENTS"), None)
        ));
        assert_eq!(
            named_holder_kind(&r, &finding(Some("RenderContext.SEGMENTS"), None)),
            Some(Holder::ThreadLocal)
        );
    }

    #[test]
    fn a_same_named_holder_of_another_package_is_not_the_owner() {
        let r = rec(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-hprof",
            json!({"holders": [{"key": "static:shop.codec.Audit.trail"}]}),
        );
        // Both are canonical dotted names; a bare field name names nothing.
        assert!(!names_holder(&r, &finding(Some("trail"), None)));
    }

    #[test]
    fn v8_snapshot_holders_name_the_owner() {
        let r = rec(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-v8-snapshot",
            json!({"file": "a.heapsnapshot", "holders": [
                {"key": "static:src.scheduler.shared", "retained": 7, "dominated": 4}]}),
        );
        assert!(names_holder(&r, &finding(Some("scheduler.shared"), None)));
        assert!(!names_holder(&r, &finding(Some("scheduler.other"), None)));
    }

    #[test]
    fn vgdb_names_the_holder_by_allocation_site_or_root() {
        let r = rec(
            EvidenceKind::RetentionPath,
            Authority::Live,
            "valgrind-vgdb",
            json!({"point": "p3", "root": "static frame_table", "complete": true, "walked": 3,
                "path": ["static frame_table", "block 0x1"],
                "alloc": "decode_frame (decoder.c:41)"}),
        );
        assert!(names_holder(&r, &finding(None, Some("src/decoder.c:42"))));
        assert!(names_holder(&r, &finding(None, Some("decoder.c:44"))));
        assert!(!names_holder(&r, &finding(None, Some("src/decoder.c:50"))));
        assert!(!names_holder(&r, &finding(None, Some("src/other.c:41"))));
        assert!(names_holder(
            &r,
            &finding(Some("decoder.frame_table"), Some("src/x.c:1"))
        ));
        assert!(!names_holder(&r, &finding(Some("decoder.frames"), None)));
    }

    #[test]
    fn alloc_stack_names_a_holder_by_its_root_only() {
        let with_root = rec(
            EvidenceKind::AllocStack,
            Authority::Live,
            "jfr",
            json!({"mode": "diff", "top": [{"at": "A.put", "site": "src/A.java:9", "root": "static:shop.Cache.entries"}]}),
        );
        assert!(names_holder(
            &with_root,
            &finding(Some("shop.Cache.entries"), None)
        ));
        assert!(!names_holder(
            &with_root,
            &finding(Some("shop.Other.entries"), None)
        ));
        let bare = rec(
            EvidenceKind::AllocStack,
            Authority::Live,
            "jfr",
            json!({"mode": "diff", "top": [{"at": "A.put", "site": "src/A.java:9"}]}),
        );
        assert!(!names_holder(
            &bare,
            &finding(Some("shop.Cache.entries"), Some("src/A.java:9"))
        ));
    }

    #[test]
    fn a_histogram_never_names_a_holder_and_nor_does_static_text() {
        let h = rec(
            EvidenceKind::HistogramDiff,
            Authority::Dump,
            "dla-hprof-histogram",
            json!({"holders": [{"key": "static:shop.codec.Audit.trail"}]}),
        );
        assert!(!names_holder(&h, &finding(Some("Audit.trail"), None)));
        let s = rec(
            EvidenceKind::RetentionPath,
            Authority::Static,
            "x",
            json!({"holders": [{"key": "static:shop.codec.Audit.trail"}]}),
        );
        assert!(!names_holder(&s, &finding(Some("Audit.trail"), None)));
    }

    #[test]
    fn a_stated_holder_kind_is_read_from_the_matching_entry() {
        let r = rec(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-hprof",
            json!({"holders": [
                {"key": "static:a.B.c", "holder_kind": "registration-table"},
                {"key": "static:shop.codec.Audit.trail", "holder_kind": "static/global"}]}),
        );
        assert_eq!(
            named_holder_kind(&r, &finding(Some("Audit.trail"), None)),
            Some(Holder::StaticGlobal)
        );
        let none = rec(
            EvidenceKind::RetentionPath,
            Authority::Dump,
            "dla-hprof",
            json!({"holders": [{"key": "static:shop.codec.Audit.trail"}]}),
        );
        assert_eq!(
            named_holder_kind(&none, &finding(Some("Audit.trail"), None)),
            None
        );
    }

    #[test]
    fn intervention_results_are_read_with_their_guards() {
        let ok = rec(
            EvidenceKind::Intervention,
            Authority::Test,
            "dla-intervene",
            json!({"finding": "F1", "effect": "growth-removed",
                "guards": {"build_ok": true, "run_ok": true, "exercised": null},
                "control": {"trend": "T3", "column": "probe:A.b"}}),
        );
        let r = InterventionResult::of(&ok).unwrap();
        assert_eq!(r.why_not_removal_of(&FindingId::new(1)), None);
        assert!(r.why_not_removal_of(&FindingId::new(2)).is_some());
        let bad = rec(
            EvidenceKind::Intervention,
            Authority::Test,
            "dla-intervene",
            json!({"finding": "F1", "effect": "growth-removed", "guards": {"build_ok": false}}),
        );
        let why = InterventionResult::of(&bad)
            .unwrap()
            .why_not_removal_of(&FindingId::new(1))
            .unwrap();
        assert!(why.contains("build_ok"));
        let remains = rec(
            EvidenceKind::Intervention,
            Authority::Test,
            "dla-intervene",
            json!({"finding": "F1", "effect": "growth-remains", "guards": {"build_ok": true}}),
        );
        assert!(
            InterventionResult::of(&remains)
                .unwrap()
                .why_not_removal_of(&FindingId::new(1))
                .unwrap()
                .contains("did not remove")
        );
    }
}
