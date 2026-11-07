//! The enumeration never depends on archetype data: a small multi-runtime
//! project is swept and censused with the shipped data files and with every
//! archetype table removed and every `suggested_archetype` blanked. The
//! enumerated counts and the (origin, site) sets must be identical.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command as Proc,
};

use assert_cmd::Command;
use serde_json::Value;

const LANGS: [&str; 6] = ["jvm", "go", "rust", "node", "python", "native"];

fn repo_data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// A project with a holder or a leaky pattern in each runtime.
fn synthetic_project(root: &Path) {
    write(
        root,
        "jvm/src/app/Registry.java",
        "package app;\nimport java.util.*;\npublic class Registry {\n    private static final Map<String, Object> CACHE = new HashMap<>();\n    private static final List<Runnable> LISTENERS = new ArrayList<>();\n    public static void put(String k, Object v) { CACHE.put(k, v); }\n    public static void listen(Runnable r) { LISTENERS.add(r); }\n    public static void main(String[] a) { put(\"a\", 1); }\n}\n",
    );
    write(
        root,
        "jvm/pom.xml",
        "<project><modelVersion>4.0.0</modelVersion><groupId>x</groupId><artifactId>x</artifactId><version>1</version></project>\n",
    );
    write(root, "go/go.mod", "module example.test/app\n\ngo 1.22\n");
    write(
        root,
        "go/main.go",
        "package main\n\nimport \"time\"\n\nvar seen = map[string][]byte{}\n\nfunc remember(k string, v []byte) { seen[k] = v }\n\nfunc spin() {\n\tgo func() {\n\t\tfor {\n\t\t\ttime.Sleep(time.Second)\n\t\t}\n\t}()\n\t_ = time.Tick(time.Second)\n}\n\nfunc main() { remember(\"a\", nil); spin() }\n",
    );
    write(
        root,
        "rust/Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    );
    write(
        root,
        "rust/src/main.rs",
        "use std::sync::Mutex;\nstatic LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());\nfn note(s: &str) { LOG.lock().unwrap().push(s.to_owned()); }\nfn pin(v: Vec<u8>) -> &'static mut [u8] { Box::leak(v.into_boxed_slice()) }\nfn main() { note(\"a\"); let _ = pin(vec![1]); }\n",
    );
    write(
        root,
        "node/package.json",
        "{\"name\":\"app\",\"version\":\"1.0.0\",\"main\":\"index.js\",\"scripts\":{\"start\":\"node index.js\"}}\n",
    );
    write(
        root,
        "node/index.js",
        "const cache = new Map();\nconst EventEmitter = require('events');\nconst bus = new EventEmitter();\nfunction remember(k, v) { cache.set(k, v); }\nfunction start() {\n  setInterval(() => remember(Date.now(), {}), 1000);\n  bus.on('tick', () => {});\n}\nstart();\n",
    );
    write(
        root,
        "python/pyproject.toml",
        "[project]\nname = \"app\"\nversion = \"0.1.0\"\n",
    );
    write(
        root,
        "python/app/cache.py",
        "import functools\nimport logging\n\n_MEMO = {}\n\n\ndef remember(key, value):\n    _MEMO[key] = value\n\n\nclass Builder:\n    @functools.lru_cache(maxsize=None)\n    def render(self, key):\n        return key\n\n\ndef attach():\n    logging.getLogger('x').addHandler(logging.NullHandler())\n\n\nif __name__ == '__main__':\n    remember('a', 1)\n",
    );
    write(root, "native/Makefile", "all:\n\tcc -o app main.c\n");
    write(
        root,
        "native/main.c",
        "#include <stdlib.h>\nstatic void *keep[64];\nstatic int n;\nvoid grab(size_t k) { keep[n++ % 64] = malloc(k); }\nint main(void) { grab(8); return 0; }\n",
    );
}

/// Copies the shipped data; with `strip`, removes archetype tables and
/// blanks every `suggested_archetype`.
fn plugin_root(dest: &Path, strip: bool) {
    let data = dest.join("data");
    fs::create_dir_all(data.join("detectors")).unwrap();
    for entry in fs::read_dir(repo_data()).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        if path.is_dir() {
            for f in fs::read_dir(&path).unwrap().flatten() {
                let text = fs::read_to_string(f.path()).unwrap();
                let text = if strip { blank_archetypes(&text) } else { text };
                fs::create_dir_all(data.join(&name)).unwrap();
                fs::write(data.join(&name).join(f.file_name()), text).unwrap();
            }
        } else if !(strip && (name.starts_with("archetypes") || name.starts_with("known-leaks"))) {
            fs::copy(&path, data.join(&name)).unwrap();
        }
    }
}

fn blank_archetypes(text: &str) -> String {
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("suggested_archetype") && t.contains('=') {
            let indent = &line[..line.len() - t.len()];
            let nl = if line.ends_with('\n') { "\n" } else { "" };
            out.push_str(&format!("{indent}suggested_archetype = \"\"{nl}"));
        } else {
            out.push_str(line);
        }
    }
    out
}

struct Outcome {
    /// Sum of the `enumerated` counts of the sweep and census summaries.
    enumerated: u64,
    /// (origin, site) of every candidate.
    sites: BTreeSet<(String, String)>,
    /// Candidates that carry a suggested archetype: (origin, tag).
    tagged: Vec<(String, String)>,
}

fn dla(project: &Path, root: &Path, data_dir: &Path, args: &[&str]) -> Value {
    let out = Command::cargo_bin("dla")
        .unwrap()
        .arg("--project")
        .arg(project)
        .args(args)
        .env("DLA_PLUGIN_ROOT", root)
        .env("DLA_DATA_DIR", data_dir)
        .env("DLA_CONSENT_TIERS", "1-4")
        .env_remove("DLA_RUN")
        .env_remove("DLA_DATA")
        .current_dir(project)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let json: Value = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
    assert_eq!(out.status.code(), Some(0), "dla {args:?}: {stdout}");
    assert_eq!(json["ok"], true, "dla {args:?}: {stdout}");
    json
}

fn enumerate(root: &Path, strip: bool) -> Outcome {
    let work = tempfile::tempdir().unwrap();
    let project = work.path().join("project");
    synthetic_project(&project);
    for args in [
        vec!["init", "-q"],
        vec!["add", "-A"],
        vec![
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "-m",
            "p",
        ],
    ] {
        let st = Proc::new("git")
            .arg("-C")
            .arg(&project)
            .args(&args)
            .status()
            .unwrap();
        assert!(st.success(), "git {args:?}");
    }
    let plugin = root.join(if strip { "stripped" } else { "shipped" });
    plugin_root(&plugin, strip);
    let data_dir = work.path().join("data-dir");
    fs::create_dir_all(&data_dir).unwrap();

    dla(&project, &plugin, &data_dir, &["run", "init"]);
    let mut enumerated = 0;
    for lang in LANGS {
        for cmd in ["sweep", "census"] {
            let v = dla(&project, &plugin, &data_dir, &[cmd, "--lang", lang]);
            if cmd == "census" {
                enumerated += v["candidates"].as_u64().unwrap();
            } else {
                for (_, b) in v["summary"].as_object().unwrap() {
                    enumerated += b["enumerated"].as_u64().unwrap_or(0);
                }
            }
        }
    }
    let run = fs::read_to_string(project.join(".dla/current")).unwrap();
    let ledger =
        fs::read_to_string(project.join(".dla").join(run.trim()).join("ledger.jsonl")).unwrap();
    let mut sites = BTreeSet::new();
    let mut tagged = Vec::new();
    for line in ledger.lines() {
        let rec: Value = serde_json::from_str(line).unwrap();
        if rec["type"] != "finding_added" {
            continue;
        }
        let f = &rec["finding"];
        let origin = f["origin"].as_str().unwrap().to_owned();
        sites.insert((origin.clone(), f["site"].as_str().unwrap().to_owned()));
        if let Some(tag) = f["suggested_archetype"].as_str().filter(|t| !t.is_empty()) {
            tagged.push((origin, tag.to_owned()));
        }
    }
    Outcome {
        enumerated,
        sites,
        tagged,
    }
}

#[test]
fn enumeration_does_not_depend_on_archetype_data() {
    let roots = tempfile::tempdir().unwrap();
    let shipped = enumerate(roots.path(), false);
    let stripped = enumerate(roots.path(), true);

    assert!(
        shipped.enumerated > 0 && !shipped.sites.is_empty(),
        "the synthetic project produced no candidates"
    );
    assert!(
        shipped.sites.iter().any(|(o, _)| o == "census")
            && shipped
                .sites
                .iter()
                .any(|(o, _)| o.starts_with("detector:")),
        "expected census and detector candidates: {:?}",
        shipped.sites
    );
    assert!(
        !shipped.tagged.is_empty(),
        "the shipped data should tag at least one detector candidate (otherwise the test proves nothing)"
    );
    assert_eq!(
        shipped.enumerated, stripped.enumerated,
        "enumerated counts differ"
    );
    assert_eq!(shipped.sites, stripped.sites, "(origin, site) sets differ");
    assert!(
        stripped.tagged.is_empty(),
        "tags without archetype data: {:?}",
        stripped.tagged
    );
    for (origin, tag) in shipped.tagged.iter().chain(&stripped.tagged) {
        assert_ne!(origin, "census", "a census candidate carries {tag}");
    }
}
