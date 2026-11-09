//! Library versions from the project's manifests and lock files.
//!
//! Readers for pom.xml, build.gradle(.kts), go.mod, Cargo.lock, package-lock.json,
//! requirements*.txt, pyproject.toml and the `[[package]]` lock files of uv and
//! poetry. Each yields `(name, version)` pairs; a version that a manifest does not
//! pin (a range, a property that cannot be resolved, a managed version) is `None`.
//! Everything read is bounded; a malformed file yields nothing.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::Path,
};

use dla_core::runtime::Runtime;
use regex::Regex;

use crate::scan::Tree;

/// Largest manifest read. Lock files are bigger than source files.
const MAX_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
/// Most libraries kept.
const MAX_LIBS: usize = 50_000;

/// One library a manifest names.
#[derive(Debug, Clone, PartialEq)]
pub struct Lib {
    /// Lowercase name: `group:artifact` (JVM), module path (Go), crate or package name.
    pub name: String,
    /// The pinned version, when the manifest pins one.
    pub version: Option<String>,
    /// Project-relative manifest.
    pub file: String,
}

/// Everything the manifests say.
#[derive(Debug, Default, Clone)]
pub struct Manifests {
    /// Libraries per runtime.
    pub libs: BTreeMap<Runtime, Vec<Lib>>,
    /// Runtimes with at least one manifest read.
    pub read: BTreeSet<Runtime>,
    /// Runtimes whose manifests list every dependency, transitive ones included
    /// (Cargo.lock, package-lock.json, uv.lock, poetry.lock).
    pub complete: BTreeSet<Runtime>,
}

fn basename(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

/// Reads `rel` under `root` when it is a regular file of at most `cap` bytes.
pub fn read_capped(root: &Path, rel: &str, cap: u64) -> Option<String> {
    let f = fs::File::open(root.join(rel)).ok()?;
    let md = f.metadata().ok()?;
    if !md.is_file() || md.len() > cap {
        return None;
    }
    let mut buf = Vec::new();
    f.take(cap).read_to_end(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).into_owned())
}

fn norm_py(name: &str) -> String {
    name.to_ascii_lowercase().replace(['_', '.'], "-")
}

fn re(p: &str) -> Regex {
    Regex::new(p).expect("static regex")
}

/// Reads every manifest of `runtimes` in `tree`.
pub fn read(tree: &Tree, runtimes: &[Runtime]) -> Manifests {
    let mut m = Manifests::default();
    let mut total = 0usize;
    for rel in &tree.files {
        if total >= MAX_LIBS {
            break;
        }
        let name = basename(rel);
        let lower = name.to_ascii_lowercase();
        let (rt, complete) = match name {
            "pom.xml" | "build.gradle" | "build.gradle.kts" => (Runtime::Jvm, false),
            "go.mod" => (Runtime::Go, false),
            "Cargo.lock" => (Runtime::Rust, true),
            "package-lock.json" => (Runtime::Node, true),
            "uv.lock" | "poetry.lock" => (Runtime::Python, true),
            "pyproject.toml" => (Runtime::Python, false),
            _ if lower.starts_with("requirements") && lower.ends_with(".txt") => {
                (Runtime::Python, false)
            }
            _ => continue,
        };
        if !runtimes.contains(&rt) {
            continue;
        }
        let Some(text) = read_capped(&tree.root, rel, MAX_MANIFEST_BYTES) else {
            continue;
        };
        let libs = match name {
            "pom.xml" => pom(&text),
            "build.gradle" | "build.gradle.kts" => gradle(&text),
            "go.mod" => go_mod(&text),
            "Cargo.lock" => package_blocks(&text, false),
            "uv.lock" | "poetry.lock" => package_blocks(&text, true),
            "package-lock.json" => package_lock(&text),
            "pyproject.toml" => pyproject(&text),
            _ => requirements(&text),
        };
        m.read.insert(rt);
        if complete {
            m.complete.insert(rt);
        }
        let dst = m.libs.entry(rt).or_default();
        for (name, version) in libs {
            total += 1;
            dst.push(Lib {
                name,
                version,
                file: rel.clone(),
            });
        }
    }
    m
}

fn clean_version(v: &str) -> Option<String> {
    let v = v.trim();
    if v.is_empty() || v.contains('$') || v.contains(['[', '(', '{']) {
        None
    } else {
        Some(v.to_owned())
    }
}

/// pom.xml: `<dependency>` blocks and the parent, with `${property}` resolved from
/// `<properties>`.
pub fn pom(text: &str) -> Vec<(String, Option<String>)> {
    let props: BTreeMap<String, String> = re(r"<([\w.\-]+)>\s*([^<>\s][^<>]*?)\s*</[\w.\-]+>")
        .captures_iter(
            re(r"(?s)<properties>(.*?)</properties>")
                .captures(text)
                .and_then(|c| c.get(1))
                .map_or("", |m| m.as_str()),
        )
        .map(|c| (c[1].to_owned(), c[2].to_owned()))
        .collect();
    let tag = |block: &str, t: &str| -> Option<String> {
        let r = Regex::new(&format!(r"<{t}>\s*([^<]+?)\s*</{t}>")).ok()?;
        r.captures(block).map(|c| c[1].to_owned())
    };
    let resolve = |v: String| -> Option<String> {
        if let Some(name) = v.strip_prefix("${").and_then(|r| r.strip_suffix('}')) {
            props.get(name).and_then(|p| clean_version(p))
        } else {
            clean_version(&v)
        }
    };
    let mut out = Vec::new();
    let blocks = re(r"(?s)<(dependency|parent|plugin)>(.*?)</(?:dependency|parent|plugin)>");
    for c in blocks.captures_iter(text) {
        let b = &c[2];
        let (Some(g), Some(a)) = (tag(b, "groupId"), tag(b, "artifactId")) else {
            continue;
        };
        out.push((
            format!("{g}:{a}").to_ascii_lowercase(),
            tag(b, "version").and_then(resolve),
        ));
    }
    out
}

/// Gradle: `"group:artifact:version"` strings and the Spring Boot plugin.
pub fn gradle(text: &str) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let dep = re(r#"['"]([A-Za-z][\w.\-]*\.[\w.\-]*):([\w.\-]+)(?::([^:'"@\s]+))?(?:@\w+)?['"]"#);
    for c in dep.captures_iter(text) {
        out.push((
            format!("{}:{}", &c[1], &c[2]).to_ascii_lowercase(),
            c.get(3).and_then(|v| clean_version(v.as_str())),
        ));
    }
    let plugin = re(
        r#"id\s*\(?\s*['"]org\.springframework\.boot['"]\s*\)?\s*version\s*\(?\s*['"]([^'"]+)['"]"#,
    );
    if let Some(c) = plugin.captures(text) {
        out.push((
            "org.springframework.boot:spring-boot-gradle-plugin".to_owned(),
            clean_version(&c[1]),
        ));
    }
    out
}

/// go.mod: `require` lines and blocks.
pub fn go_mod(text: &str) -> Vec<(String, Option<String>)> {
    let line = re(r"^\s*(?:require\s+)?([\w.\-]+\.[\w.\-/~]*)\s+(v[\w.\-+]+)");
    text.lines()
        .filter(|l| !l.contains("=>"))
        .filter_map(|l| {
            let c = line.captures(l)?;
            Some((c[1].to_ascii_lowercase(), Some(c[2].to_owned())))
        })
        .collect()
}

/// Cargo.lock, uv.lock and poetry.lock: `[[package]]` blocks with `name` and `version`.
pub fn package_blocks(text: &str, python: bool) -> Vec<(String, Option<String>)> {
    let name = re(r#"(?m)^name\s*=\s*"([^"]+)""#);
    let version = re(r#"(?m)^version\s*=\s*"([^"]+)""#);
    text.split("[[package]]")
        .skip(1)
        .filter_map(|blk| {
            let blk = blk.split("\n[").next().unwrap_or(blk);
            let n = name.captures(blk)?[1].to_owned();
            let n = if python { norm_py(&n) } else { n };
            Some((n, version.captures(blk).map(|c| c[1].to_owned())))
        })
        .collect()
}

/// package-lock.json: `packages` (lockfile v2/v3) or `dependencies` (v1).
pub fn package_lock(text: &str) -> Vec<(String, Option<String>)> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(pk) = v.get("packages").and_then(|p| p.as_object()) {
        for (k, e) in pk {
            let Some(idx) = k.rfind("node_modules/") else {
                continue;
            };
            let name = &k[idx + "node_modules/".len()..];
            out.push((
                name.to_ascii_lowercase(),
                e.get("version").and_then(|x| x.as_str()).map(str::to_owned),
            ));
        }
    } else if let Some(d) = v.get("dependencies").and_then(|p| p.as_object()) {
        fn walk(
            d: &serde_json::Map<String, serde_json::Value>,
            out: &mut Vec<(String, Option<String>)>,
            depth: usize,
        ) {
            for (k, e) in d {
                out.push((
                    k.to_ascii_lowercase(),
                    e.get("version").and_then(|x| x.as_str()).map(str::to_owned),
                ));
                if depth < 32
                    && let Some(n) = e.get("dependencies").and_then(|x| x.as_object())
                {
                    walk(n, out, depth + 1);
                }
            }
        }
        walk(d, &mut out, 0);
    }
    out
}

fn pep508(spec: &str) -> Option<(String, Option<String>)> {
    let r = re(r"^\s*([A-Za-z0-9][\w.\-]*)\s*(?:\[[^\]]*\])?\s*(?:(===|==)\s*([\w.\-*+!]+))?");
    let c = r.captures(spec)?;
    let version = c
        .get(3)
        .map(|v| v.as_str().to_owned())
        .filter(|v| !v.contains('*'));
    Some((norm_py(&c[1]), version))
}

/// requirements*.txt: only `==` pins give a version.
pub fn requirements(text: &str) -> Vec<(String, Option<String>)> {
    text.lines()
        .map(|l| l.split('#').next().unwrap_or(""))
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('-'))
        .filter_map(pep508)
        .collect()
}

/// pyproject.toml: the `dependencies` arrays and the poetry tables.
pub fn pyproject(text: &str) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    let arr = re(r"(?s)\bdependencies\s*=\s*\[(.*?)\]");
    let item = re(r#"["']([^"']+)["']"#);
    for c in arr.captures_iter(text) {
        for i in item.captures_iter(&c[1]) {
            out.extend(pep508(&i[1]));
        }
    }
    let mut in_poetry = false;
    let kv = re(r#"^([A-Za-z0-9][\w.\-]*)\s*=\s*(?:"([^"]*)"|\{.*)"#);
    for l in text.lines() {
        let t = l.trim();
        if t.starts_with('[') {
            in_poetry = t.starts_with("[tool.poetry.") && t.contains("dependencies");
            continue;
        }
        if in_poetry && let Some(c) = kv.captures(t) {
            let n = norm_py(&c[1]);
            if n != "python" {
                let v = c
                    .get(2)
                    .map(|m| m.as_str())
                    .filter(|v| v.chars().all(|ch| ch.is_ascii_digit() || ch == '.'))
                    .and_then(clean_version);
                out.push((n, v));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find<'a>(v: &'a [(String, Option<String>)], n: &str) -> Option<&'a Option<String>> {
        v.iter().find(|(x, _)| x == n).map(|(_, v)| v)
    }

    #[test]
    fn pom_resolves_properties() {
        let p = pom(
            r#"<project><parent><groupId>org.springframework.boot</groupId>
<artifactId>spring-boot-starter-parent</artifactId><version>3.2.1</version></parent>
<properties><netty.version>4.1.100.Final</netty.version></properties>
<dependencies><dependency><groupId>io.netty</groupId><artifactId>netty-all</artifactId>
<version>${netty.version}</version></dependency>
<dependency><groupId>a</groupId><artifactId>b</artifactId></dependency>
<dependency><groupId>c</groupId><artifactId>d</artifactId><version>${missing}</version></dependency>
</dependencies></project>"#,
        );
        assert_eq!(
            find(&p, "org.springframework.boot:spring-boot-starter-parent"),
            Some(&Some("3.2.1".to_owned()))
        );
        assert_eq!(
            find(&p, "io.netty:netty-all"),
            Some(&Some("4.1.100.Final".to_owned()))
        );
        assert_eq!(find(&p, "a:b"), Some(&None));
        assert_eq!(find(&p, "c:d"), Some(&None));
    }

    #[test]
    fn gradle_go_cargo_node_python() {
        let g = gradle(
            "implementation(\"io.netty:netty-handler:4.1.50.Final\")\nid 'org.springframework.boot' version '3.1.0'\nruntimeOnly 'x.y:z'",
        );
        assert_eq!(
            find(&g, "io.netty:netty-handler"),
            Some(&Some("4.1.50.Final".to_owned()))
        );
        assert_eq!(
            find(&g, "org.springframework.boot:spring-boot-gradle-plugin"),
            Some(&Some("3.1.0".to_owned()))
        );
        assert_eq!(find(&g, "x.y:z"), Some(&None));

        let m = go_mod(
            "module example.com/m\n\ngo 1.21\n\nrequire (\n\tgithub.com/a/b v1.2.3 // indirect\n)\nrequire golang.org/x/sync v0.5.0\nreplace x.y/z v1 => ./z\n",
        );
        assert_eq!(find(&m, "github.com/a/b"), Some(&Some("v1.2.3".to_owned())));
        assert_eq!(
            find(&m, "golang.org/x/sync"),
            Some(&Some("v0.5.0".to_owned()))
        );
        assert_eq!(m.len(), 2);

        let c = package_blocks(
            "[[package]]\nname = \"serde\"\nversion = \"1.0.1\"\n\n[[package]]\nname = \"x\"\n",
            false,
        );
        assert_eq!(find(&c, "serde"), Some(&Some("1.0.1".to_owned())));
        assert_eq!(find(&c, "x"), Some(&None));

        let n = package_lock(
            r#"{"packages":{"":{"name":"p"},"node_modules/a/node_modules/@s/b":{"version":"2.0.0"}}}"#,
        );
        assert_eq!(find(&n, "@s/b"), Some(&Some("2.0.0".to_owned())));
        let n1 = package_lock(
            r#"{"dependencies":{"a":{"version":"1.0.0","dependencies":{"b":{"version":"2.0.0"}}}}}"#,
        );
        assert_eq!(n1.len(), 2);
        assert!(package_lock("not json").is_empty());

        let r = requirements(
            "Django==4.2.1  # c\nfoo_bar>=1.0\n-r other.txt\n\nrequests[security]==2.0\n",
        );
        assert_eq!(find(&r, "django"), Some(&Some("4.2.1".to_owned())));
        assert_eq!(find(&r, "foo-bar"), Some(&None));
        assert_eq!(find(&r, "requests"), Some(&Some("2.0".to_owned())));
        assert_eq!(r.len(), 3);

        let p = pyproject(
            "[project]\ndependencies = [\n \"httpx==0.27.0\",\n \"rich>=13\",\n]\n[tool.poetry.dependencies]\npython = \"^3.11\"\nflask = \"2.0.1\"\nattrs = \"^23\"\n",
        );
        assert_eq!(find(&p, "httpx"), Some(&Some("0.27.0".to_owned())));
        assert_eq!(find(&p, "rich"), Some(&None));
        assert_eq!(find(&p, "flask"), Some(&Some("2.0.1".to_owned())));
        assert_eq!(find(&p, "attrs"), Some(&None));
        assert_eq!(find(&p, "python"), None);
    }
}
