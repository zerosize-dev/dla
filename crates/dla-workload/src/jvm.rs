//! JVM workloads: JUnit/TestNG classes, `*Test` mains, `main` classes, Spring Boot apps.

use std::{collections::BTreeMap, sync::LazyLock};

use dla_core::{runtime::Runtime, workload::WorkloadKind};
use regex::Regex;

use crate::model::{FileInfo, Raw, Scan, cwd_of, join, stem};

static PACKAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*package\s+([\w.]+)").expect("static regex"));
static JAVA_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"static\s+void\s+main\s*\(").expect("static regex"));
static KT_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(?:suspend\s+)?fun\s+main\s*\(").expect("static regex"));
/// `fun main(` at the left margin: a top-level function, compiled into `<Stem>Kt`.
static TOP_LEVEL_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:suspend\s+)?fun\s+main\s*\(").expect("static regex"));
/// A class or object declared at the left margin: annotations (`@Suppress("x")`) and modifiers
/// may precede the keyword, on the same line. Group 1 holds the modifiers, group 2 the name.
static TOP_LEVEL_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^((?:(?:@[\w.]+(?:\([^)\n]*\))?|public|internal|private|open|abstract|final|data|sealed|inner|enum|annotation|value)\s+)*)(?:class|object)\s+(\w+)"#)
        .expect("static regex")
});
/// A JUnit 5 test method annotation.
static TEST_ANNOTATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"@(?:Test|ParameterizedTest|RepeatedTest|TestFactory|TestTemplate)\b")
        .expect("static regex")
});
static RUN_ARG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"["']run["']|\brun\)"#).expect("static regex"));

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Tool {
    Maven,
    Gradle,
    Script,
    Bare,
}

fn has_main(f: &FileInfo) -> bool {
    JAVA_MAIN.is_match(&f.text) || KT_MAIN.is_match(&f.text)
}

fn is_kotlin(f: &FileInfo) -> bool {
    f.rel.ends_with(".kt") || f.rel.ends_with(".kts")
}

/// Fully qualified name of the class a file defines, for a test selector (`--tests <fqcn>`).
///
/// Java: the file's stem. Kotlin: the stem when the file declares `class <stem>` or
/// `object <stem>`, else the first top-level class or object the file declares (a test class
/// need not be named like its file), else `<Stem>Kt` (a file of top-level functions).
pub fn fqcn(f: &FileInfo) -> String {
    fqcn_of(&f.rel, &f.text)
}

/// [`fqcn`] from a path and the file's text.
pub fn fqcn_of(rel: &str, text: &str) -> String {
    let stem = stem(rel);
    let name = if is_kotlin_path(rel) && !declares_type(text, stem) {
        first_type(text).unwrap_or_else(|| format!("{stem}Kt"))
    } else {
        stem.to_owned()
    };
    qualified(text, name)
}

/// Fully qualified name of the class that holds a Kotlin or Java file's `main`, for
/// `java <fqcn>`. A Kotlin file with a top-level `fun main` compiles it into `<Stem>Kt`, even
/// when the file also declares `class <Stem>` (a command class plus a top-level main).
pub fn main_fqcn(f: &FileInfo) -> String {
    main_fqcn_of(&f.rel, &f.text)
}

/// [`main_fqcn`] from a path and the file's text.
pub fn main_fqcn_of(rel: &str, text: &str) -> String {
    if is_kotlin_path(rel) && TOP_LEVEL_MAIN.is_match(text) {
        return qualified(text, format!("{}Kt", stem(rel)));
    }
    fqcn_of(rel, text)
}

fn is_kotlin_path(rel: &str) -> bool {
    rel.ends_with(".kt") || rel.ends_with(".kts")
}

fn qualified(text: &str, name: String) -> String {
    match PACKAGE.captures(text) {
        Some(c) => format!("{}.{name}", &c[1]),
        None => name,
    }
}

/// The text without `//` and `/* */` comments (a commented-out class or annotation is not one).
/// String literals (`"..."` with escapes, `"""..."""`) and character literals are copied as they
/// are: a `"*/*"` or `"/api/**"` in a string is not a comment.
fn strip_comments(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let (mut i, mut from) = (0, 0);
    while i < b.len() {
        let rest = &text[i..];
        if let Some(raw) = rest.strip_prefix("\"\"\"") {
            i += 3 + raw.find("\"\"\"").map_or(raw.len(), |e| e + 3);
        } else if b[i] == b'"' || (b[i] == b'\'' && b.get(i + 2) == Some(&b'\'')) {
            let q = b[i];
            i += 1;
            while i < b.len() && b[i] != q && b[i] != b'\n' {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i += 1;
        } else if b[i] == b'\'' && b.get(i + 1) == Some(&b'\\') {
            i += 2;
            while i < b.len() && b[i] != b'\'' && b[i] != b'\n' {
                i += 1;
            }
            i += 1;
        } else if rest.starts_with("//") && !(i > 0 && b[i - 1] == b':') {
            out.push_str(&text[from..i]);
            i += rest.find('\n').unwrap_or(rest.len());
            from = i;
        } else if let Some(block) = rest.strip_prefix("/*") {
            out.push_str(&text[from..i]);
            i += block.find("*/").map_or(rest.len(), |e| e + 4);
            from = i;
        } else {
            i += 1;
        }
        i = i.min(b.len());
        while i < b.len() && !text.is_char_boundary(i) {
            i += 1;
        }
    }
    out.push_str(&text[from.min(text.len())..]);
    out
}

/// The class or object declared at the left margin of a Kotlin file that holds the tests: the
/// first non-abstract one whose body has a JUnit test annotation, else the first non-abstract
/// one (an abstract base class that holds the `@Test` methods is not the class to run; its
/// concrete subclass is), else the first one with tests, else the first (a helper `data class` above the test class is not
/// the test class).
fn first_type(text: &str) -> Option<String> {
    let text = strip_comments(text);
    let found: Vec<(usize, String, bool)> = TOP_LEVEL_TYPE
        .captures_iter(&text)
        .filter_map(|c| {
            let modifiers = c.get(1)?.as_str();
            let r#abstract = modifiers.split_whitespace().any(|m| m == "abstract");
            Some((c.get(0)?.start(), c[2].to_owned(), r#abstract))
        })
        .collect();
    let has_tests = |i: usize| {
        let end = found.get(i + 1).map_or(text.len(), |n| n.0);
        TEST_ANNOTATION.is_match(&text[found[i].0..end])
    };
    let concrete = |i: &usize| !found[*i].2;
    let all = 0..found.len();
    let at = all
        .clone()
        .find(|i| concrete(i) && has_tests(*i))
        .or_else(|| all.clone().find(concrete))
        .or_else(|| all.clone().find(|i| has_tests(*i)))
        .unwrap_or(0);
    found.into_iter().nth(at).map(|f| f.1)
}

/// True when Kotlin or Java text declares `class <name>` or `object <name>` (a whole word).
fn declares_type(text: &str, name: &str) -> bool {
    let text = strip_comments(text);
    let words: Vec<&str> = text
        .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))
        .filter(|w| !w.is_empty())
        .collect();
    words
        .windows(2)
        .any(|w| matches!(w[0], "class" | "object" | "interface") && w[1] == name)
}

fn shell_for(script: &str) -> &'static str {
    let first = script.lines().next().unwrap_or("");
    if first.contains("bash") { "bash" } else { "sh" }
}

fn compile_all(scan: &Scan, root: &str) -> String {
    let kotlin = scan
        .files_of(Runtime::Jvm)
        .any(|f| is_kotlin(f) && (root.is_empty() || f.rel.starts_with(root)));
    if kotlin {
        "mkdir -p out && kotlinc $(find . -name '*.kt' -o -name '*.java' | grep -v '^./out/') -d out"
            .to_owned()
    } else {
        "mkdir -p out && javac -d out $(find . -name '*.java' -not -path './out/*')".to_owned()
    }
}

pub fn discover(scan: &Scan) -> Vec<Raw> {
    let mut groups: BTreeMap<(String, Tool), Vec<&FileInfo>> = BTreeMap::new();
    for f in scan.files_of(Runtime::Jvm) {
        let (root, tool) = match scan.nearest(
            &f.rel,
            &[
                "pom.xml",
                "build.gradle",
                "build.gradle.kts",
                "settings.gradle",
                "settings.gradle.kts",
                "build.sh",
            ],
        ) {
            Some((d, n)) if n == "pom.xml" => (d, Tool::Maven),
            Some((d, n)) if n == "build.sh" => (d, Tool::Script),
            Some((d, _)) => (d, Tool::Gradle),
            None => (String::new(), Tool::Bare),
        };
        groups.entry((root, tool)).or_default().push(f);
    }
    let mut out = Vec::new();
    for ((root, tool), files) in groups {
        let cwd = cwd_of(&root);
        let rt = Runtime::Jvm;
        let java_run = |f: &FileInfo, cp: &str, ea: bool| {
            format!(
                "java{} -cp {cp} {}",
                if ea { " -ea" } else { "" },
                main_fqcn(f)
            )
        };
        let tests: Vec<&FileInfo> = files.iter().copied().filter(|f| f.is_test).collect();
        let mains: Vec<&FileInfo> = files
            .iter()
            .copied()
            .filter(|f| !f.is_test && has_main(f))
            .collect();
        let spring: Vec<&FileInfo> = files
            .iter()
            .copied()
            .filter(|f| !f.is_test && f.text.contains("@SpringBootApplication"))
            .collect();
        let build_file = |name: &str| join(&root, name);
        match tool {
            Tool::Script => {
                let script = build_file("build.sh");
                let body = scan.read(&script).unwrap_or_default();
                let sh = shell_for(&body);
                let main_tests: Vec<&FileInfo> =
                    tests.iter().copied().filter(|f| has_main(f)).collect();
                if !main_tests.is_empty() {
                    out.push(Raw {
                        id: format!("jvm:test:{script}"),
                        kind: WorkloadKind::Test,
                        runtime: rt,
                        command: format!("{sh} ./build.sh"),
                        cwd: cwd.clone(),
                        file: script.clone(),
                        srcs: main_tests.iter().map(|f| f.rel.clone()).collect(),
                        extra: Default::default(),
                    });
                }
                if !mains.is_empty() {
                    let (id, command) = if RUN_ARG.is_match(&body) {
                        (
                            format!("jvm:main:{script}:run"),
                            format!("{sh} ./build.sh run"),
                        )
                    } else {
                        (
                            format!("jvm:main:{}", mains[0].rel),
                            format!(
                                "{} && {}",
                                compile_all(scan, &root),
                                java_run(mains[0], "out", false)
                            ),
                        )
                    };
                    out.push(Raw {
                        id,
                        kind: WorkloadKind::Main,
                        runtime: rt,
                        command,
                        cwd: cwd.clone(),
                        file: mains[0].rel.clone(),
                        srcs: mains.iter().map(|f| f.rel.clone()).collect(),
                        extra: Default::default(),
                    });
                }
            }
            Tool::Maven | Tool::Gradle | Tool::Bare => {
                for f in &tests {
                    let junit = f.text.contains("@Test") || f.text.contains("@ParameterizedTest");
                    let class = fqcn(f);
                    let simple = stem(&f.rel);
                    let command = match (tool, junit) {
                        (Tool::Maven, true) => format!("mvn -q -o test -Dtest={simple}"),
                        (Tool::Gradle, true) => format!("gradle -q --offline test --tests {class}"),
                        (Tool::Maven, false) if has_main(f) => format!(
                            "mvn -q -o test-compile && {}",
                            java_run(f, "target/classes:target/test-classes", true)
                        ),
                        (Tool::Gradle, false) if has_main(f) => format!(
                            "gradle -q --offline testClasses && {}",
                            java_run(
                                f,
                                "build/classes/java/main:build/classes/java/test:build/classes/kotlin/main:build/classes/kotlin/test",
                                true
                            )
                        ),
                        (Tool::Bare, false) if has_main(f) => {
                            format!(
                                "{} && {}",
                                compile_all(scan, &root),
                                java_run(f, "out", true)
                            )
                        }
                        _ => continue,
                    };
                    out.push(Raw {
                        id: format!("jvm:test:{}", f.rel),
                        kind: WorkloadKind::Test,
                        runtime: rt,
                        command,
                        cwd: cwd.clone(),
                        file: f.rel.clone(),
                        srcs: vec![f.rel.clone()],
                        extra: Default::default(),
                    });
                }
                for f in &spring {
                    let command = match tool {
                        Tool::Maven => "mvn -q -o spring-boot:run".to_owned(),
                        Tool::Gradle => "gradle -q --offline bootRun".to_owned(),
                        _ => continue,
                    };
                    out.push(Raw {
                        id: format!("jvm:dev-server:{}", f.rel),
                        kind: WorkloadKind::DevServer,
                        runtime: rt,
                        command,
                        cwd: cwd.clone(),
                        file: f.rel.clone(),
                        srcs: vec![f.rel.clone()],
                        extra: Default::default(),
                    });
                }
                for f in mains
                    .iter()
                    .filter(|m| !spring.iter().any(|s| s.rel == m.rel))
                {
                    let command = match tool {
                        Tool::Maven => format!(
                            "mvn -q -o compile && {}",
                            java_run(f, "target/classes", false)
                        ),
                        Tool::Gradle => format!(
                            "gradle -q --offline classes && {}",
                            java_run(
                                f,
                                "build/classes/java/main:build/classes/kotlin/main",
                                false
                            )
                        ),
                        _ => format!(
                            "{} && {}",
                            compile_all(scan, &root),
                            java_run(f, "out", false)
                        ),
                    };
                    out.push(Raw {
                        id: format!("jvm:main:{}", f.rel),
                        kind: WorkloadKind::Main,
                        runtime: rt,
                        command,
                        cwd: cwd.clone(),
                        file: f.rel.clone(),
                        srcs: vec![f.rel.clone()],
                        extra: Default::default(),
                    });
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{fqcn_of, main_fqcn_of};

    #[test]
    fn kotlin_classes_keep_their_name_and_only_function_files_get_kt() {
        let class = "package shop\n\nclass FooTest {\n  @Test fun a() {}\n}\n";
        assert_eq!(
            fqcn_of("src/test/kotlin/shop/FooTest.kt", class),
            "shop.FooTest"
        );
        let object =
            "package shop\n\nobject Main {\n  @JvmStatic fun main(a: Array<String>) {}\n}\n";
        assert_eq!(fqcn_of("Main.kt", object), "shop.Main");
        let functions = "package shop\n\nfun main() {}\n";
        assert_eq!(fqcn_of("App.kt", functions), "shop.AppKt");
        // A longer name is not the class: `FooTestHelper` does not declare `FooTest`, but it is the
        // first class the file declares.
        let other = "package shop\n\nclass FooTestHelper\nfun main() {}\n";
        assert_eq!(fqcn_of("FooTest.kt", other), "shop.FooTestHelper");
        // A test class named differently from its file.
        let renamed = "package shop\n\ninternal class GetTest {\n  @Test fun a() {}\n}\n";
        assert_eq!(fqcn_of("ApiTests.kt", renamed), "shop.GetTest");
        // A helper class above the test class is not the test class.
        let helper = "package shop\n\nprivate data class Payload(val id: Int)\n\nclass GetTest {\n  @Test fun a() {}\n}\n";
        assert_eq!(fqcn_of("ApiTests.kt", helper), "shop.GetTest");
        // A command class plus a top-level main: the main lives in the `Kt` facade.
        let both = "package app\n\nclass Server {\n  fun run() {}\n}\n\nfun main(args: Array<String>) {\n  Server().run()\n}\n";
        assert_eq!(main_fqcn_of("Server.kt", both), "app.ServerKt");
        assert_eq!(fqcn_of("Server.kt", both), "app.Server");
        // An object's @JvmStatic main is indented: the object is the main class.
        assert_eq!(main_fqcn_of("Main.kt", object), "shop.Main");
        assert_eq!(
            main_fqcn_of("shop/Cart.java", "package shop;\npublic class Cart {}\n"),
            "shop.Cart"
        );
        assert_eq!(
            fqcn_of("shop/Cart.java", "package shop;\npublic class Cart {}\n"),
            "shop.Cart"
        );
        assert_eq!(fqcn_of("Cart.java", "public class Cart {}\n"), "Cart");
    }

    #[test]
    fn the_test_class_is_found_in_less_regular_kotlin_layouts() {
        // An annotation on the same line as the class.
        let same_line = "package demo

@Suppress(\"unused\") class GetTest {
  @Test fun a() {}
}
";
        assert_eq!(fqcn_of("ApiTests.kt", same_line), "demo.GetTest");
        let annotated = "package demo

@SpringBootTest(classes = [App::class]) class GetTest {
  @Test fun a() {}
}
";
        assert_eq!(fqcn_of("ApiTests.kt", annotated), "demo.GetTest");
        // A comment that mentions @Test or a class does not count.
        let comment = "package demo

class Helper {
  // call it from a @Test
  fun x() {}
}

// class ApiTests was renamed
class GetTest {
  @Test fun a() {}
}
";
        assert_eq!(fqcn_of("ApiTests.kt", comment), "demo.GetTest");
        let renamed_comment = "package demo

// class ApiTests was renamed
class GetTest {
  @Test fun a() {}
}
";
        assert_eq!(fqcn_of("ApiTests.kt", renamed_comment), "demo.GetTest");
        // The concrete subclass is the class to run, not the abstract base with the tests.
        let base = "package demo

abstract class BaseTest {
  @Test fun a() {}
}

class GetTest : BaseTest()
";
        assert_eq!(fqcn_of("ApiTests.kt", base), "demo.GetTest");
        // A string that looks like a comment start does not cut the file off.
        for lit in [
            "private const val ACCEPT_ALL = \"*/*\"",
            "private const val API = \"/api/**\"",
            "private const val RAW = \"\"\"/* not a comment\"\"\"",
            "private const val Q = '\"'",
            "private const val E = \"a\\\"/*\"",
        ] {
            let text =
                format!("package demo\n\n{lit}\n\nclass GetTest {{\n  @Test fun a() {{}}\n}}\n");
            assert_eq!(fqcn_of("ApiTests.kt", &text), "demo.GetTest", "{lit}");
        }
        let annotation =
            "package demo\n\n@Sql(\"/db/*.sql\")\nclass GetTest {\n  @Test fun a() {}\n}\n";
        assert_eq!(fqcn_of("FooTest.kt", annotation), "demo.GetTest");
    }
}
