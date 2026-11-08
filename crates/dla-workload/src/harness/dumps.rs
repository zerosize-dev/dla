//! Reading agent dump records and choosing the measured process.
//!
//! A harness run writes one dump file per measured process (`out/<label>/dump.jsonl`).
//! Several processes append to it when the workload spawns children (a wrapper script,
//! `npm`, `go test`, a tool JVM), so [`select_measured`] picks the record of the process
//! that was asked to run `n` iterations.
//!
//! Records are read leniently. The typed [`HarnessDump`] cannot hold the allocator shim's
//! `raw_sites` (a map of stack objects) nor its `modules`, so [`parse_record`] lifts those
//! two fields out of the JSON first and keeps them next to the typed record.

use std::{collections::BTreeMap, fs, path::Path};

use dla_core::{
    harness::{DumpPhase, HarnessDump},
    runtime::Runtime,
};
use serde_json::Value;

/// Executables that are build or package tools, never the measured program.
const TOOL_PROCESSES: &[&str] = &[
    "javac", "jar", "jlink", "jshell", "mvn", "gradle", "gradlew", "npm", "npx", "yarn", "pnpm",
    "corepack", "pip", "pip3", "cargo", "rustc", "make", "cc", "gcc", "clang", "ld", "sh", "bash",
    "dash",
];

/// Values of `values` that identify the "primary heap" of a record, in preference order.
const PRIMARY_HEAP: &[&str] = &[
    "jvm.heap_used",
    "go.heap_live",
    "node.heap_used",
    "py.traced",
    "native.live_bytes",
    "os.rss",
];

/// One dump record plus the two fields the typed record cannot hold.
#[derive(Debug, Clone)]
pub struct Rec {
    /// The typed record.
    pub dump: HarnessDump,
    /// The allocator shim's `raw_sites` object, or the spec's `{metric: {addr: bytes}}` map.
    pub raw_sites: Option<Value>,
    /// The shim's `modules` list (`[{path, bias}]`).
    pub modules: Option<Value>,
}

impl Rec {
    /// The value of `name` in `values`, if present and not null.
    pub fn value(&self, name: &str) -> Option<f64> {
        self.dump
            .values
            .as_ref()
            .and_then(|v| v.get(name).copied().flatten())
    }

    /// The figure that ranks processes: the first present of the runtime heap figures.
    pub fn primary_heap(&self) -> Option<f64> {
        PRIMARY_HEAP.iter().find_map(|n| self.value(n))
    }

    /// First element of `argv`, or the executable path.
    fn program(&self) -> Option<&str> {
        self.dump
            .argv
            .as_ref()
            .and_then(|a| a.first())
            .map(String::as_str)
            .or(self.dump.exe.as_deref())
    }
}

/// Everything read from one dump file.
#[derive(Debug, Clone, Default)]
pub struct DumpFile {
    /// Records in file order.
    pub records: Vec<Rec>,
    /// Lines that were not usable records.
    pub bad_lines: usize,
}

/// Keeps only numeric and null entries of an object (agents occasionally emit a string).
fn numeric_only(v: &mut Value) {
    if let Value::Object(m) = v {
        m.retain(|_, x| x.is_number() || x.is_null());
    }
}

/// Parses one dump line. `None` for anything that is not a JSON object with the
/// `dla_harness` marker and a known phase.
pub fn parse_record(line: &str) -> Option<Rec> {
    let mut doc: Value = serde_json::from_str(line).ok()?;
    let obj = doc.as_object_mut()?;
    let raw_sites = obj.remove("raw_sites").filter(|v| !v.is_null());
    let modules = obj.remove("modules").filter(|v| !v.is_null());
    for key in ["values", "values_pre_gc"] {
        if let Some(v) = obj.get_mut(key) {
            numeric_only(v);
        }
    }
    if let Some(Value::Object(p)) = obj.get_mut("purge") {
        for key in ["before", "after"] {
            if let Some(v) = p.get_mut(key) {
                numeric_only(v);
            }
        }
    }
    if let Some(Value::Object(g)) = obj.get_mut("groups") {
        g.retain(|_, v| v.is_object());
        for v in g.values_mut() {
            if let Value::Object(m) = v {
                m.retain(|_, x| x.is_number());
            }
        }
    }
    // A malformed optional section must not lose the measurement.
    for key in [
        "probes",
        "types",
        "coverage",
        "detectors",
        "limitations",
        "argv",
    ] {
        let bad = match (obj.get(key), key) {
            (Some(v), "coverage") => !v.is_object(),
            (Some(v), _) => !v.is_array(),
            (None, _) => false,
        };
        if bad {
            obj.remove(key);
        }
    }
    let dump = match serde_json::from_value::<HarnessDump>(doc.clone()) {
        Ok(d) => d,
        Err(_) => {
            // Drop the optional sections one by one until the record parses.
            for key in [
                "probes",
                "types",
                "coverage",
                "detectors",
                "limitations",
                "purge",
                "groups",
                "values_pre_gc",
                "values",
            ] {
                doc.as_object_mut()?.remove(key);
                if let Ok(d) = serde_json::from_value::<HarnessDump>(doc.clone()) {
                    return Some(Rec {
                        dump: d,
                        raw_sites,
                        modules,
                    });
                }
            }
            return None;
        }
    };
    Some(Rec {
        dump,
        raw_sites,
        modules,
    })
}

/// Reads `path`; a missing or unreadable file is an empty [`DumpFile`].
pub fn read_dumps(path: &Path) -> DumpFile {
    let Ok(text) = fs::read_to_string(path) else {
        return DumpFile::default();
    };
    let mut file = DumpFile::default();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        match parse_record(line) {
            Some(r) => file.records.push(r),
            None => file.bad_lines += 1,
        }
    }
    file
}

fn basename(program: &str) -> String {
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    name.strip_suffix(".exe").unwrap_or(name).to_lowercase()
}

/// Whether the record comes from a build or package tool process.
fn is_tool(rec: &Rec) -> bool {
    let Some(p) = rec.program() else {
        return false;
    };
    TOOL_PROCESSES.contains(&basename(p).as_str())
}

/// Whether the record's `runtime` field fits the manifest's runtime (an absent field fits).
fn runtime_matches(rec: &Rec, runtime: Runtime) -> bool {
    let Some(r) = rec.dump.runtime.as_deref() else {
        return true;
    };
    let r = r.to_lowercase();
    match runtime {
        Runtime::Rust | Runtime::Native => r == "native" || r == "rust",
        other => r == other.as_str(),
    }
}

/// Whether `argv` asks for `n` iterations: `n` as a token, or `=n` as a suffix.
pub fn argv_has_n(argv: &[String], n: u64) -> bool {
    let s = n.to_string();
    let suffix = format!("={s}");
    argv.iter()
        .any(|a| *a == s || a.ends_with(&suffix) || a.split_whitespace().any(|t| t == s))
}

/// The measured process's record and what to say about the choice.
#[derive(Debug, Clone)]
pub struct Selection<'a> {
    /// The chosen record.
    pub rec: &'a Rec,
    /// Its process ID, when the record has one.
    pub pid: Option<u32>,
    /// Notes for the series: fallbacks taken.
    pub limitations: Vec<String>,
}

/// Groups records by process, keeping file order within a process.
fn by_pid(records: &[Rec]) -> Vec<(Option<u32>, Vec<&Rec>)> {
    let mut groups: Vec<(Option<u32>, Vec<&Rec>)> = Vec::new();
    for r in records {
        match groups.iter_mut().find(|(p, _)| *p == r.dump.pid) {
            Some((_, v)) => v.push(r),
            None => groups.push((r.dump.pid, vec![r])),
        }
    }
    groups
}

/// Chooses the record of the measured process of a command-mode point (spec 4.3):
/// the `exit` record (else the last checkpoint, else the last tick, with a limitation) of
/// the process whose argv has `n` as a token or `=n` suffix and whose runtime matches,
/// excluding tool processes; ties go to the largest primary heap value.
pub fn select_measured(file: &DumpFile, runtime: Runtime, n: Option<u64>) -> Option<Selection<'_>> {
    let mut limitations = Vec::new();
    let mut groups = by_pid(&file.records);
    groups.retain(|(_, v)| !is_tool(v[0]));
    if groups.is_empty() {
        return None;
    }
    let runtime_ok: Vec<_> = groups
        .iter()
        .filter(|(_, v)| v.iter().any(|r| runtime_matches(r, runtime)))
        .cloned()
        .collect();
    if runtime_ok.is_empty() {
        limitations.push(format!(
            "no dump record names runtime {}; used the records that exist",
            runtime.as_str()
        ));
    } else {
        groups = runtime_ok;
    }
    // Per process: the record that represents it.
    let mut cands: Vec<(Option<u32>, &Rec, bool, bool)> = groups
        .iter()
        .map(|(pid, recs)| {
            let exit = recs.iter().rev().find(|r| r.dump.phase == DumpPhase::Exit);
            let pick = exit
                .or_else(|| {
                    recs.iter()
                        .rev()
                        .find(|r| r.dump.phase == DumpPhase::Checkpoint)
                })
                .or_else(|| recs.last())
                .copied()
                .unwrap_or(recs[0]);
            let has_n = n.is_some_and(|n| {
                recs.iter()
                    .filter_map(|r| r.dump.argv.as_deref())
                    .any(|a| argv_has_n(a, n))
            });
            (*pid, pick, exit.is_some(), has_n)
        })
        .collect();
    let multiple = cands.len() > 1;
    if cands.iter().any(|c| c.3) {
        cands.retain(|c| c.3);
    }
    if cands.iter().any(|c| c.2) {
        cands.retain(|c| c.2);
    }
    let best = cands.iter().max_by(|a, b| {
        let ka = a.1.primary_heap().unwrap_or(-1.0);
        let kb = b.1.primary_heap().unwrap_or(-1.0);
        ka.total_cmp(&kb)
    })?;
    let (pid, rec, has_exit, has_n) = (best.0, best.1, best.2, best.3);
    if multiple && !has_n && n.is_some() {
        limitations.push(format!(
            "no process had {} in its argv; chose pid {} by the largest heap value",
            n.unwrap_or(0),
            pid.map_or_else(|| "?".to_owned(), |p| p.to_string())
        ));
    }
    if !has_exit {
        limitations.push(format!(
            "no exit record from pid {} (killed, or exited without a dump); used its last {} record",
            pid.map_or_else(|| "?".to_owned(), |p| p.to_string()),
            rec.dump.phase.as_str()
        ));
    }
    Some(Selection {
        rec,
        pid,
        limitations,
    })
}

/// The checkpoint records of a driver-mode run, in order: those of the process with the
/// most checkpoints (ties: the later first record). Notes on what was skipped come back
/// with them.
pub fn select_checkpoints(
    file: &DumpFile,
    runtime: Runtime,
) -> (Vec<&Rec>, Option<u32>, Vec<String>) {
    let mut notes = Vec::new();
    let groups: Vec<_> = by_pid(&file.records)
        .into_iter()
        .filter(|(_, v)| !is_tool(v[0]) && v.iter().any(|r| runtime_matches(r, runtime)))
        .map(|(p, v)| {
            let cps: Vec<&Rec> = v
                .into_iter()
                .filter(|r| r.dump.phase == DumpPhase::Checkpoint)
                .collect();
            (p, cps)
        })
        .filter(|(_, v)| !v.is_empty())
        .collect();
    let Some((pid, cps)) = groups
        .iter()
        .max_by_key(|(_, v)| v.len())
        .map(|(p, v)| (*p, v.clone()))
    else {
        return (Vec::new(), None, notes);
    };
    if groups.len() > 1 {
        notes.push(format!(
            "{} processes wrote checkpoints; used pid {} ({} records)",
            groups.len(),
            pid.map_or_else(|| "?".to_owned(), |p| p.to_string()),
            cps.len()
        ));
    }
    (cps, pid, notes)
}

/// Collects the `limitations` strings of a record.
pub fn record_limitations(rec: &Rec) -> &[String] {
    rec.dump.limitations.as_deref().unwrap_or(&[])
}

/// Counts records per phase, for diagnostics.
pub fn phase_counts(file: &DumpFile) -> BTreeMap<&'static str, usize> {
    let mut m = BTreeMap::new();
    for r in &file.records {
        *m.entry(r.dump.phase.as_str()).or_insert(0) += 1;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn line(v: Value) -> String {
        v.to_string()
    }

    fn file(lines: &[Value]) -> DumpFile {
        let text = lines.iter().map(|v| line(v.clone())).collect::<Vec<_>>();
        let mut f = DumpFile::default();
        for l in text {
            f.records.push(parse_record(&l).unwrap());
        }
        f
    }

    #[test]
    fn shim_raw_sites_do_not_lose_the_record() {
        let l = line(json!({
            "dla_harness":1,"phase":"exit","pid":5,"runtime":"native","x":1000,
            "values":{"native.live_bytes":10,"bad":"str"},
            "raw_sites":{"k1":{"live_bytes":5,"live_blocks":1,"frames":[{"a":1,"m":0,"o":2}]}},
            "modules":[{"path":"/x","bias":0}]
        }));
        let r = parse_record(&l).unwrap();
        assert_eq!(r.value("native.live_bytes"), Some(10.0));
        assert!(r.dump.values.as_ref().unwrap().get("bad").is_none());
        assert!(r.raw_sites.is_some() && r.modules.is_some());
        assert!(r.dump.raw_sites.is_none());
    }

    #[test]
    fn garbage_and_unmarked_lines_are_skipped() {
        assert!(parse_record("not json").is_none());
        assert!(parse_record(r#"{"phase":"exit"}"#).is_none());
        assert!(parse_record(r#"{"dla_harness":1,"phase":"weird"}"#).is_none());
        let r = parse_record(r#"{"dla_harness":1,"phase":"exit","probes":"oops"}"#).unwrap();
        assert!(r.dump.probes.is_none());
    }

    #[test]
    fn n_matches_token_and_equals_suffix_but_not_substrings() {
        let a = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        assert!(argv_has_n(&a(&["x", "--iterations", "2000"]), 2000));
        assert!(argv_has_n(&a(&["x", "--iterations=2000"]), 2000));
        assert!(argv_has_n(&a(&["x", "-n 2000"]), 2000));
        assert!(!argv_has_n(&a(&["x", "12000"]), 2000));
    }

    #[test]
    fn the_exit_record_of_the_process_with_n_wins_over_tools_and_larger_heaps() {
        let f = file(&[
            json!({"dla_harness":1,"phase":"exit","pid":1,"runtime":"jvm","argv":["javac","A.java"],"values":{"jvm.heap_used":99999999}}),
            json!({"dla_harness":1,"phase":"tick","pid":2,"runtime":"jvm","argv":["java","Main","2000"],"values":{"jvm.heap_used":1}}),
            json!({"dla_harness":1,"phase":"exit","pid":2,"runtime":"jvm","argv":["java","Main","2000"],"values":{"jvm.heap_used":10}}),
            json!({"dla_harness":1,"phase":"exit","pid":3,"runtime":"jvm","argv":["java","Other","5"],"values":{"jvm.heap_used":500}}),
        ]);
        let s = select_measured(&f, Runtime::Jvm, Some(2000)).unwrap();
        assert_eq!(s.pid, Some(2));
        assert_eq!(s.rec.value("jvm.heap_used"), Some(10.0));
        assert!(s.limitations.is_empty());
    }

    #[test]
    fn ties_go_to_the_largest_heap_and_a_missing_exit_is_a_limitation() {
        let f = file(&[
            json!({"dla_harness":1,"phase":"exit","pid":2,"runtime":"node","argv":["node","a.js"],"values":{"node.heap_used":10}}),
            json!({"dla_harness":1,"phase":"exit","pid":3,"runtime":"node","argv":["node","b.js"],"values":{"node.heap_used":30}}),
        ]);
        let s = select_measured(&f, Runtime::Node, Some(7)).unwrap();
        assert_eq!(s.pid, Some(3));
        assert_eq!(s.limitations.len(), 1);

        let f = file(&[
            json!({"dla_harness":1,"phase":"tick","pid":9,"runtime":"go","values":{"go.heap_live":1}}),
            json!({"dla_harness":1,"phase":"tick","pid":9,"runtime":"go","values":{"go.heap_live":2}}),
        ]);
        let s = select_measured(&f, Runtime::Go, Some(5)).unwrap();
        assert_eq!(s.rec.value("go.heap_live"), Some(2.0));
        assert!(s.limitations[0].contains("no exit record"));
    }

    #[test]
    fn rust_matches_the_native_shim() {
        let f = file(&[
            json!({"dla_harness":1,"phase":"exit","pid":4,"runtime":"native","argv":["./app","--n","10"],"values":{"native.live_bytes":1}}),
        ]);
        let s = select_measured(&f, Runtime::Rust, Some(10)).unwrap();
        assert!(s.limitations.is_empty());
    }

    #[test]
    fn checkpoints_come_from_the_busiest_process_in_order() {
        let f = file(&[
            json!({"dla_harness":1,"phase":"checkpoint","pid":2,"runtime":"python","x":10}),
            json!({"dla_harness":1,"phase":"checkpoint","pid":2,"runtime":"python","x":20}),
            json!({"dla_harness":1,"phase":"tick","pid":2,"runtime":"python"}),
            json!({"dla_harness":1,"phase":"checkpoint","pid":3,"runtime":"python","x":10}),
        ]);
        let (cps, pid, notes) = select_checkpoints(&f, Runtime::Python);
        assert_eq!(pid, Some(2));
        assert_eq!(
            cps.iter().map(|r| r.dump.x).collect::<Vec<_>>(),
            [Some(10.0), Some(20.0)]
        );
        assert_eq!(notes.len(), 1);
    }
}
