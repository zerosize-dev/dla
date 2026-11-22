//! The test loop of a JVM project: the project's own JUnit 5 tests run in a loop by a harness
//! with no hooks (`dla harness gen --lang jvm --junit`), as the first pass of the queue.
//!
//! A project's tests are units somebody already wrote. This module decides, from the workload
//! discovery result and the test sources, which ranked candidates have a test class that
//! exercises their owner class, and how those candidates are cut into batches that one run
//! measures (see the constants for what bounds a batch). It is a pure function of the run
//! directory and the project's files; `dla harness gen --junit --next` and every text that names
//! the next step read the same plan.
//!
//! A candidate's owner class is the class of the file of its site. A test class is chosen for
//! it by name first (the test class's name holds the owner class's name, or its singular, at a
//! camel-case boundary: `LazyUnitTests` for `Lazy`), else by mention (the test's source uses the
//! owner class's name as an identifier).

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use crate::{
    ids::FindingId,
    runtime::Runtime,
    view::RunView,
    workload::{Workload, WorkloadKind, WorkloadReport},
};

/// The most findings one test-loop harness declares as targets. A target keeps its probe
/// columns (the reachable objects, `.bytes` and `.size`: three) whatever the 64-column cap of a
/// series drops, and a series' other columns (the baseline of the process and the JVM, about
/// 20) are not dropped for them: (64 - 20) / 3 leaves 14, and 16 is the limit the siblings of
/// a hook driver have. The probe walk does not grow with it: every census candidate is probed
/// at every point, declared or not, so a declared target costs a column, not time. The manifest
/// holds the list and nothing else per target.
pub const MAX_TARGETS: usize = 16;

/// The most test classes one test-loop harness runs.
pub const MAX_CLASSES: usize = 10;

/// The most tests (methods annotated `@Test`, `@ParameterizedTest`, `@RepeatedTest`,
/// `@TestFactory` or `@TestTemplate`) the classes of one harness hold in all. A series is 18
/// runs of the classes (a warm-up round and 8 measured rounds of 2): about 6 ms a test, so
/// 1000 tests are a run of about two minutes; a harness that would hold more is cut.
pub const MAX_TESTS: usize = 1000;

/// The test classes taken for one candidate by name.
pub const PER_NAME: usize = 2;
/// The test classes taken for one candidate by mention, when none matches by name.
pub const PER_MENTION: usize = 1;

/// One test class that a test-loop harness chose for a finding, from `harness/<H>/tests.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// The fully qualified class name.
    pub class: String,
    /// Project-relative source file.
    pub file: String,
    /// The tests it holds.
    pub tests: u64,
    /// How it was chosen for the finding: `name` or `mention`.
    pub by: String,
}

impl Chosen {
    /// The class name without its package.
    pub fn simple_name(&self) -> &str {
        self.class.rsplit('.').next().unwrap_or(&self.class)
    }
}

/// The test classes the harness `harness` chose for `id`, in the order of `tests.json`. Empty
/// when the file is missing, unreadable, or names no class for the finding.
pub fn chosen_for(run_dir: &Path, harness: &str, id: &FindingId) -> Vec<Chosen> {
    let path = crate::HarnessManifest::dir_in(run_dir, harness).join("tests.json");
    let Some(doc) = fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
    else {
        return Vec::new();
    };
    let want = id.to_string();
    let mut out = Vec::new();
    for c in doc["classes"].as_array().into_iter().flatten() {
        let Some(class) = c["class"].as_str() else {
            continue;
        };
        let Some(by) = c["for"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|f| f["finding"].as_str() == Some(want.as_str()))
        else {
            continue;
        };
        out.push(Chosen {
            class: class.to_owned(),
            file: c["file"].as_str().unwrap_or_default().to_owned(),
            tests: c["tests"].as_u64().unwrap_or(0),
            by: by["by"].as_str().unwrap_or_default().to_owned(),
        });
    }
    out
}

/// The rule, in one sentence, as `dla harness gen --junit --next` prints it.
pub fn rule() -> String {
    format!(
        "for each candidate in rank order the test classes whose name holds its owner class's name (at most {PER_NAME}), else the one that uses the owner class's name (the package's own first, then the one with the fewest tests); a harness holds at most {MAX_TARGETS} targets, {MAX_CLASSES} test classes, {MAX_TESTS} tests and the classes of one module, so a run stays about two minutes; a candidate whose classes are all in the batch rides along; the rest wait for the next batch"
    )
}

/// How a test class was chosen for an owner class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Via {
    /// The test class's name holds the owner class's name.
    Name,
    /// The test class's source uses the owner class's name.
    Mention,
}

impl Via {
    /// `name` or `mention`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Mention => "mention",
        }
    }
}

/// One test class of the project.
#[derive(Debug, Clone)]
pub struct TestClass {
    /// Project-relative source file.
    pub file: String,
    /// The file's name without its extension.
    pub stem: String,
    /// The working directory of its workload (a module): one harness runs the classes of one.
    pub cwd: String,
    /// The tests it holds (see [`MAX_TESTS`]).
    pub tests: usize,
    /// The directory of the file under `src/test/<language>`, for the package match.
    package_dir: String,
    /// The capitalized identifiers of the source.
    words: BTreeSet<String>,
}

/// The test classes of a project, when the test loop can run them.
#[derive(Debug, Clone, Default)]
pub struct TestIndex {
    /// The classes a JUnit repeat can run, in file order.
    pub classes: Vec<TestClass>,
}

/// The test classes chosen for one owner class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    /// The owner class (the file name of the candidate's site, without the extension).
    pub owner: String,
    /// Indices into [`TestIndex::classes`], with how each was chosen.
    pub classes: Vec<(usize, Via)>,
}

/// One class of a batch and why it is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassChoice {
    /// Project-relative source file of the test class.
    pub file: String,
    /// The tests it holds.
    pub tests: usize,
    /// Its module (working directory of its workload).
    pub cwd: String,
    /// The candidates it was taken for, with how.
    pub for_ids: Vec<(FindingId, Via)>,
}

/// One harness of the test loop.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Batch {
    /// The candidates, in rank order (the ones a class of the batch also serves follow).
    pub ids: Vec<FindingId>,
    /// The test classes and why.
    pub classes: Vec<ClassChoice>,
}

impl Batch {
    /// The tests of all classes.
    pub fn tests(&self) -> usize {
        self.classes.iter().map(|c| c.tests).sum()
    }
}

/// What the test loop makes of the pending candidates: batches, and the rest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Plan {
    /// The batches, the next one first.
    pub batches: Vec<Batch>,
    /// The candidates that have no test class, or whose test class is too big for one harness:
    /// they wait for a driver.
    pub driver: Vec<FindingId>,
}

impl Plan {
    /// The candidates the batches hold.
    pub fn candidates(&self) -> usize {
        self.batches.iter().map(|b| b.ids.len()).sum()
    }

    /// The sentence that names the test loop first: "N candidates in M test-loop batches,
    /// nothing to write; then K candidates need a driver". Empty when no batch is left.
    pub fn sentence(&self) -> String {
        if self.batches.is_empty() {
            return String::new();
        }
        let (n, m) = (self.candidates(), self.batches.len());
        let then = if self.driver.is_empty() {
            String::new()
        } else {
            format!(
                "; then {} candidate{} need{} a driver",
                self.driver.len(),
                if self.driver.len() == 1 { "" } else { "s" },
                if self.driver.len() == 1 { "s" } else { "" }
            )
        };
        format!(
            "{n} candidate{} in {m} test-loop batch{}, nothing to write{then}",
            if n == 1 { "" } else { "s" },
            if m == 1 { "" } else { "es" }
        )
    }
}

/// True when `name` is an executable on `PATH`.
pub fn on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let p = dir.join(name);
        p.is_file() || (cfg!(windows) && dir.join(format!("{name}.exe")).is_file())
    })
}

/// The wrapper script `name` in the workload's directory or the nearest parent, as a path
/// relative to the project root.
pub fn find_wrapper(project: &Path, cwd: &str, name: &str) -> Option<String> {
    let mut dir: Vec<&str> = cwd
        .split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect();
    loop {
        let rel = if dir.is_empty() {
            name.to_owned()
        } else {
            format!("{}/{name}", dir.join("/"))
        };
        if project.join(&rel).is_file() {
            return Some(rel);
        }
        dir.pop()?;
    }
}

/// `maven` or `gradle`: the build tool a test workload's command runs through.
pub fn build_tool(w: &Workload) -> Option<&'static str> {
    let first = w.command.split_whitespace().next()?;
    let name = first.rsplit('/').next().unwrap_or(first);
    if name.starts_with("mvn") {
        Some("maven")
    } else if name.starts_with("gradle") {
        Some("gradle")
    } else {
        None
    }
}

/// True when the tool that resolves the test class path is there: the project's wrapper, or
/// the tool on `PATH`.
fn tool_ready(project: &Path, w: &Workload) -> bool {
    match build_tool(w) {
        Some("maven") => find_wrapper(project, &w.cwd, "mvnw").is_some() || on_path("mvn"),
        Some("gradle") => find_wrapper(project, &w.cwd, "gradlew").is_some() || on_path("gradle"),
        _ => false,
    }
}

/// Why workload `w` cannot be run by the JUnit repeat driver; `None` when it can: a JVM test
/// of a Gradle or Maven project whose file uses the JUnit 5 (`org.junit.jupiter`) API.
pub fn junit_problem(project: &Path, w: &Workload) -> Option<String> {
    let text = fs::read_to_string(project.join(&w.file)).ok();
    junit_problem_in(w, text.as_deref())
}

/// [`junit_problem`] with the text of the workload's file already read (`None`: unreadable).
pub fn junit_problem_in(w: &Workload, text: Option<&str>) -> Option<String> {
    if w.runtime != Runtime::Jvm || w.kind != WorkloadKind::Test {
        return Some(format!(
            "workload {} is a {} {} workload, not a JVM test",
            w.id,
            w.runtime.as_str(),
            w.kind.as_str()
        ));
    }
    if build_tool(w).is_none() {
        return Some(format!(
            "workload {} does not run through Maven or Gradle (command: {}), so there is no build tool to resolve the test class path",
            w.id, w.command
        ));
    }
    let Some(text) = text else {
        return Some(format!("{} cannot be read", w.file));
    };
    // `kotlin.test` runs on JUnit 5 when the project uses the kotlin-test-junit5 backend; the
    // build step checks that a junit-platform-engine is on the class path.
    if text.contains("org.junit.jupiter")
        || (text.contains("kotlin.test") && !text.contains("org.junit.Test"))
    {
        return None;
    }
    if text.contains("org.junit.Test") || text.contains("org.testng") {
        Some(format!(
            "{} uses JUnit 4 or TestNG, which the Jupiter launcher runs only with the vintage engine (junit-vintage-engine) or not at all",
            w.file
        ))
    } else {
        Some(format!(
            "{} imports neither org.junit.jupiter nor kotlin.test, so it is not a JUnit 5 test",
            w.file
        ))
    }
}

/// The annotations that make a method a test.
const TEST_ANNOTATIONS: [&str; 5] = [
    "Test",
    "ParameterizedTest",
    "RepeatedTest",
    "TestFactory",
    "TestTemplate",
];

/// The tests of a source: the annotations above, by whole name (`@TestFactory` is one,
/// `@TestInstance` none).
pub fn count_tests(text: &str) -> usize {
    let b = text.as_bytes();
    let mut n = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'@' {
            let start = i + 1;
            let mut end = start;
            while end < b.len() && (b[end].is_ascii_alphanumeric() || b[end] == b'_') {
                end += 1;
            }
            if TEST_ANNOTATIONS.contains(&&text[start..end]) {
                n += 1;
            }
            i = end.max(i + 1);
        } else {
            i += 1;
        }
    }
    n
}

/// The capitalized identifiers of a source.
fn words_of(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut cur = String::new();
    for c in text.chars().chain(std::iter::once(' ')) {
        if c.is_alphanumeric() || c == '_' || c == '$' {
            cur.push(c);
        } else {
            if cur.chars().next().is_some_and(char::is_uppercase) {
                out.insert(std::mem::take(&mut cur));
            }
            cur.clear();
        }
    }
    out
}

fn stem_of(file: &str) -> String {
    let name = file.rsplit('/').next().unwrap_or(file);
    name.rsplit_once('.').map_or(name, |(s, _)| s).to_owned()
}

/// The directory of `file` below `src/<main|test>/<language>/`, `""` when it is not laid out so.
fn package_dir(file: &str) -> String {
    let dir = file.rsplit_once('/').map_or("", |(d, _)| d);
    let padded = format!("/{dir}/");
    for marker in ["/src/main/", "/src/test/"] {
        if let Some(i) = padded.find(marker) {
            // Drop the language directory (`java`, `kotlin`).
            return padded[i + marker.len()..]
                .split_once('/')
                .map_or("", |(_, p)| p.trim_end_matches('/'))
                .to_owned();
        }
    }
    String::new()
}

/// True when `owner` (or its singular, `Lazy` for `Lazys`; `CustomCollection` for
/// `CustomCollections`) occurs in `test` and is followed by something that is not a lowercase
/// letter or a digit: the end of the name or the next capitalized word.
fn name_matches(test: &str, owner: &str) -> bool {
    let singular = owner
        .strip_suffix('s')
        .filter(|s| s.chars().count() >= 4 && !s.ends_with('s'));
    [Some(owner), singular].into_iter().flatten().any(|o| {
        test.match_indices(o).any(|(i, m)| {
            test[i + m.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_lowercase() || c.is_ascii_digit()))
        })
    })
}

impl TestIndex {
    /// The test classes of the run's project the JUnit repeat can run, from the workload
    /// discovery result; `None` when it cannot run any (no JVM test workload that passes
    /// [`junit_problem`], or no build tool to resolve the class path).
    pub fn of(view: &RunView, worker: Option<&str>) -> Option<Self> {
        let run_dir = view.run_dir.as_deref()?;
        let project = Path::new(&view.state.project_root);
        let file = worker.map_or_else(|| "workloads.json".to_owned(), |w| format!("{w}.json"));
        let text = fs::read_to_string(run_dir.join("workload").join(file)).ok()?;
        let report: WorkloadReport = serde_json::from_str(&text).ok()?;
        let mut classes = Vec::new();
        let mut ready: BTreeMap<String, bool> = BTreeMap::new();
        for w in &report.workloads {
            if w.runtime != Runtime::Jvm || w.kind != WorkloadKind::Test {
                continue;
            }
            let Ok(src) = fs::read_to_string(project.join(&w.file)) else {
                continue;
            };
            if junit_problem_in(w, Some(&src)).is_some() {
                continue;
            }
            let ok = *ready
                .entry(format!(
                    "{}\u{0}{}",
                    w.cwd,
                    w.command.split_whitespace().next().unwrap_or("")
                ))
                .or_insert_with(|| tool_ready(project, w));
            if !ok {
                continue;
            }
            classes.push(TestClass {
                file: w.file.clone(),
                stem: stem_of(&w.file),
                cwd: w.cwd.clone(),
                tests: count_tests(&src),
                package_dir: package_dir(&w.file),
                words: words_of(&src),
            });
        }
        if classes.is_empty() {
            return None;
        }
        classes.sort_by(|a, b| a.file.cmp(&b.file));
        Some(Self { classes })
    }

    /// The test classes for the owner class of the file `site_file`: by name (at most
    /// [`PER_NAME`], the closest name first), else by mention (at most [`PER_MENTION`], the
    /// one of the owner's package first, then the one with the fewest tests). `None` when no
    /// class of at most [`MAX_TESTS`] tests uses the owner class.
    pub fn pick(&self, site_file: &str) -> Option<Pick> {
        if site_file.contains("/src/test/") || site_file.starts_with("src/test/") {
            return None;
        }
        let owner = stem_of(site_file);
        if owner.is_empty() {
            return None;
        }
        let pkg = package_dir(site_file);
        let small = |c: &&(usize, &TestClass)| c.1.tests <= MAX_TESTS && c.1.tests > 0;
        let all: Vec<(usize, &TestClass)> = self.classes.iter().enumerate().collect();
        let mut by_name: Vec<&(usize, &TestClass)> = all
            .iter()
            .filter(|(_, c)| name_matches(&c.stem, &owner))
            .filter(small)
            .collect();
        by_name.sort_by_key(|(_, c)| (c.stem.len(), c.tests, c.file.clone()));
        if let Some((_, first)) = by_name.first() {
            // The classes of one pick are of one module and fit one harness together: a harness
            // runs the classes of one module, and a pick that does not fit alone would never
            // run. The closest name decides the module.
            let cwd = first.cwd.clone();
            let mut tests = 0;
            let classes: Vec<(usize, Via)> = by_name
                .into_iter()
                .filter(|(_, c)| c.cwd == cwd)
                .filter(|(_, c)| {
                    let fits = tests + c.tests <= MAX_TESTS;
                    if fits {
                        tests += c.tests;
                    }
                    fits
                })
                .take(PER_NAME)
                .map(|(i, _)| (*i, Via::Name))
                .collect();
            return Some(Pick { owner, classes });
        }
        let mut by_mention: Vec<&(usize, &TestClass)> = all
            .iter()
            .filter(|(_, c)| c.words.contains(&owner))
            .filter(small)
            .collect();
        by_mention.sort_by_key(|(_, c)| (c.package_dir != pkg, c.tests, c.file.clone()));
        let classes: Vec<(usize, Via)> = by_mention
            .into_iter()
            .take(PER_MENTION)
            .map(|(i, _)| (*i, Via::Mention))
            .collect();
        (!classes.is_empty()).then_some(Pick { owner, classes })
    }
}

/// The batches of the candidates `order` (pending, in rank order), each with its pick; a
/// candidate without a pick goes to [`Plan::driver`].
///
/// A batch takes candidates by rank while the classes of all of them stay inside
/// [`MAX_CLASSES`], [`MAX_TESTS`] and one module, and the candidates inside [`MAX_TARGETS`];
/// the first that does not fit starts the next batch. Candidates further down whose classes
/// are all in the batch already (the other holders of the same owner, other candidates of the
/// same file) are added after: they cost nothing.
pub fn plan(index: &TestIndex, order: &[(FindingId, Option<Pick>)]) -> Plan {
    let mut out = Plan::default();
    let mut rest: Vec<(&FindingId, &Pick)> = Vec::new();
    for (id, pick) in order {
        match pick {
            Some(p) => rest.push((id, p)),
            None => out.driver.push(id.clone()),
        }
    }
    while !rest.is_empty() {
        let mut batch = Batch::default();
        let mut taken: Vec<usize> = Vec::new();
        for (k, (id, pick)) in rest.iter().enumerate() {
            let fits = batch.ids.len() < MAX_TARGETS && adds(index, &batch, pick).is_some();
            if !fits {
                if batch.ids.is_empty() {
                    // Nothing fits even alone: one class is more than a harness holds.
                    taken.push(k);
                    out.driver.push((*id).clone());
                    continue;
                }
                break;
            }
            add(index, &mut batch, id, pick);
            taken.push(k);
        }
        // Free riders.
        for (k, (id, pick)) in rest.iter().enumerate() {
            if batch.ids.len() >= MAX_TARGETS {
                break;
            }
            if taken.contains(&k) || batch.ids.is_empty() {
                continue;
            }
            if adds(index, &batch, pick) == Some(0) {
                add(index, &mut batch, id, pick);
                taken.push(k);
            }
        }
        let mut k = 0;
        rest.retain(|_| {
            let keep = !taken.contains(&k);
            k += 1;
            keep
        });
        if !batch.ids.is_empty() {
            out.batches.push(batch);
        }
    }
    out.driver
        .sort_by_key(|f| order.iter().position(|(o, _)| o == f).unwrap_or(usize::MAX));
    out
}

/// How many classes `pick` adds to `batch`, or `None` when the batch would pass a bound.
fn adds(index: &TestIndex, batch: &Batch, pick: &Pick) -> Option<usize> {
    let mut classes = batch.classes.len();
    let mut tests = batch.tests();
    let mut added = 0;
    let cwd = batch.classes.first().map(|c| c.cwd.as_str());
    for (i, _) in &pick.classes {
        let c = &index.classes[*i];
        if cwd.is_some_and(|d| d != c.cwd) {
            return None;
        }
        if !batch.classes.iter().any(|x| x.file == c.file) {
            classes += 1;
            tests += c.tests;
            added += 1;
        }
    }
    (classes <= MAX_CLASSES && tests <= MAX_TESTS).then_some(added)
}

fn add(index: &TestIndex, batch: &mut Batch, id: &FindingId, pick: &Pick) {
    batch.ids.push(id.clone());
    for (i, via) in &pick.classes {
        let c = &index.classes[*i];
        match batch.classes.iter_mut().find(|x| x.file == c.file) {
            Some(x) => x.for_ids.push((id.clone(), *via)),
            None => batch.classes.push(ClassChoice {
                file: c.file.clone(),
                tests: c.tests,
                cwd: c.cwd.clone(),
                for_ids: vec![(id.clone(), *via)],
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(file: &str, tests: usize, words: &[&str]) -> TestClass {
        TestClass {
            file: file.to_owned(),
            stem: stem_of(file),
            cwd: ".".to_owned(),
            tests,
            package_dir: package_dir(file),
            words: words.iter().map(|w| (*w).to_owned()).collect(),
        }
    }

    fn index(classes: Vec<TestClass>) -> TestIndex {
        TestIndex { classes }
    }

    #[test]
    fn names_match_at_a_camel_case_boundary_and_in_the_singular() {
        assert!(name_matches("LazyUnitTests", "Lazy"));
        assert!(name_matches("TestLazy", "Lazy"));
        assert!(name_matches("Lazy", "Lazy"));
        assert!(!name_matches("LazynessTests", "Lazy"));
        assert!(!name_matches("Lazy2Tests", "Lazy"));
        assert!(name_matches(
            "TypedPropertyPathUnitTests",
            "TypedPropertyPaths"
        ));
        assert!(name_matches(
            "KTypedPropertyPathUnitTests",
            "TypedPropertyPaths"
        ));
        // A short name is not stripped of its `s`.
        assert!(!name_matches("BusTests", "Bs"));
        assert!(!name_matches("ClassTests", "Clas"));
    }

    #[test]
    fn tests_are_counted_by_annotation_name() {
        let t = "@Test void a() {}\n@ParameterizedTest @ValueSource void b() {}\n@TestInstance(PER_CLASS)\n@TestMethodOrder(X)\n@TestFactory void c() {}\n// @Test in a comment counts: the count is a size, not a verdict\n";
        assert_eq!(count_tests(t), 4);
        assert_eq!(count_tests("@Tests @Testing"), 0);
    }

    #[test]
    fn the_package_of_a_file_is_its_directory_under_the_source_root() {
        assert_eq!(
            package_dir("src/main/java/org/acme/util/Lazy.java"),
            "org/acme/util"
        );
        assert_eq!(
            package_dir("mod/src/test/kotlin/org/acme/util/LazyTests.kt"),
            "org/acme/util"
        );
        assert_eq!(package_dir("Lazy.java"), "");
    }

    #[test]
    fn a_name_beats_a_mention_and_the_closest_name_comes_first() {
        let ix = index(vec![
            class("src/test/java/a/LazyIntegrationTests.java", 3, &["Lazy"]),
            class("src/test/java/a/LazyTests.java", 5, &[]),
            class("src/test/java/a/OtherTests.java", 2, &["Lazy"]),
        ]);
        let p = ix.pick("src/main/java/a/Lazy.java").unwrap();
        assert_eq!(p.owner, "Lazy");
        assert_eq!(p.classes, vec![(1, Via::Name), (0, Via::Name)]);
    }

    #[test]
    fn a_mention_is_taken_when_no_name_matches_the_one_of_the_package_first() {
        let ix = index(vec![
            class("src/test/java/b/UsesTests.java", 1, &["Lazy"]),
            class("src/test/java/a/AlsoTests.java", 9, &["Lazy"]),
            class("src/test/java/a/NoneTests.java", 1, &["Other"]),
        ]);
        let p = ix.pick("src/main/java/a/Lazy.java").unwrap();
        assert_eq!(p.classes, vec![(1, Via::Mention)]);
        assert!(ix.pick("src/main/java/a/Nothing.java").is_none());
    }

    #[test]
    fn a_class_with_no_tests_or_too_many_is_not_a_unit() {
        let ix = index(vec![
            class("src/test/java/a/LazyTests.java", 0, &[]),
            class("src/test/java/a/LazyBigTests.java", MAX_TESTS + 1, &[]),
        ]);
        assert!(ix.pick("src/main/java/a/Lazy.java").is_none());
    }

    #[test]
    fn the_classes_of_one_pick_are_of_one_module() {
        // The closest name is in module m1; the other name match is in m2 and is left out.
        let mut a = class("m1/src/test/java/a/LazyTests.java", 5, &[]);
        a.cwd = "m1".into();
        let mut b = class("m2/src/test/java/a/LazyUnitTests.java", 3, &[]);
        b.cwd = "m2".into();
        let ix = index(vec![a, b]);
        let p = ix.pick("m1/src/main/java/a/Lazy.java").unwrap();
        assert_eq!(p.classes, vec![(0, Via::Name)]);
        let order = vec![(FindingId::new(1), Some(p))];
        let plan = plan(&ix, &order);
        assert_eq!(plan.batches.len(), 1);
        assert!(plan.batches[0].classes.iter().all(|c| c.cwd == "m1"));
    }

    #[test]
    fn the_classes_of_one_pick_fit_one_harness_together() {
        // Two name matches that hold more tests together than one harness runs: the closest
        // alone, and the candidate is in a batch, not sent to a driver.
        let ix = index(vec![
            class("src/test/java/a/LazyTests.java", 600, &[]),
            class("src/test/java/a/LazyUnitTests.java", 600, &[]),
        ]);
        let p = ix.pick("src/main/java/a/Lazy.java").unwrap();
        assert_eq!(p.classes, vec![(0, Via::Name)]);
        let plan = plan(&ix, &[(FindingId::new(1), Some(p))]);
        assert_eq!(plan.batches.len(), 1);
        assert!(plan.driver.is_empty());
        assert_eq!(plan.batches[0].tests(), 600);
    }

    #[test]
    fn a_file_under_the_test_tree_has_no_owner_class() {
        let ix = index(vec![class("src/test/java/a/LazyTests.java", 3, &["Lazy"])]);
        assert!(ix.pick("src/test/java/a/Lazy.java").is_none());
    }

    fn ids(n: u32) -> Vec<FindingId> {
        (1..=n).map(FindingId::new).collect()
    }

    #[test]
    fn candidates_are_batched_by_rank_and_the_batch_is_closed_by_a_bound() {
        // Twelve owners with one class of 150 tests each: 6 classes fit 1000 tests, so 2 batches.
        let ix = index(
            (0..12)
                .map(|i| class(&format!("src/test/java/a/O{i:02}Tests.java"), 150, &[]))
                .collect(),
        );
        let order: Vec<(FindingId, Option<Pick>)> = ids(12)
            .into_iter()
            .enumerate()
            .map(|(i, f)| (f, ix.pick(&format!("src/main/java/a/O{i:02}.java"))))
            .collect();
        let p = plan(&ix, &order);
        assert_eq!(p.batches.len(), 2, "{p:?}");
        assert_eq!(p.batches[0].ids.len(), 6);
        assert_eq!(p.batches[0].tests(), 900);
        assert_eq!(p.batches[0].ids[0], FindingId::new(1));
        assert_eq!(p.batches[1].ids[0], FindingId::new(7));
        assert!(p.driver.is_empty());
        assert_eq!(
            p.sentence(),
            "12 candidates in 2 test-loop batches, nothing to write"
        );
    }

    #[test]
    fn the_holders_of_one_class_ride_along_and_cost_nothing() {
        let ix = index(vec![
            class("src/test/java/a/AaaTests.java", 10, &[]),
            class("src/test/java/a/BbbTests.java", 10, &[]),
        ]);
        // F1 and F3 are in Aaa, F2 in Bbb: with a 1-class bound F3 would wait for the next
        // batch by rank, but it needs no class that is not there.
        let pa = ix.pick("src/main/java/a/Aaa.java");
        let pb = ix.pick("src/main/java/a/Bbb.java");
        let order = vec![
            (FindingId::new(1), pa.clone()),
            (FindingId::new(2), pb),
            (FindingId::new(3), pa),
        ];
        let p = plan(&ix, &order);
        assert_eq!(p.batches.len(), 1);
        assert_eq!(p.batches[0].ids, ids(3));
        assert_eq!(p.batches[0].classes.len(), 2);
        assert_eq!(p.batches[0].classes[0].for_ids.len(), 2);
    }

    #[test]
    fn a_candidate_without_a_test_waits_for_a_driver_and_is_counted() {
        let ix = index(vec![class("src/test/java/a/AaaTests.java", 10, &[])]);
        let order = vec![
            (FindingId::new(1), None),
            (FindingId::new(2), ix.pick("src/main/java/a/Aaa.java")),
            (FindingId::new(3), None),
        ];
        let p = plan(&ix, &order);
        assert_eq!(p.driver, vec![FindingId::new(1), FindingId::new(3)]);
        assert_eq!(p.candidates(), 1);
        assert_eq!(
            p.sentence(),
            "1 candidate in 1 test-loop batch, nothing to write; then 2 candidates need a driver"
        );
    }

    #[test]
    fn a_batch_never_mixes_two_modules() {
        let mut a = class("m1/src/test/java/a/AaaTests.java", 10, &[]);
        a.cwd = "m1".into();
        let mut b = class("m2/src/test/java/a/BbbTests.java", 10, &[]);
        b.cwd = "m2".into();
        let ix = index(vec![a, b]);
        let order = vec![
            (FindingId::new(1), ix.pick("m1/src/main/java/a/Aaa.java")),
            (FindingId::new(2), ix.pick("m2/src/main/java/a/Bbb.java")),
        ];
        let p = plan(&ix, &order);
        assert_eq!(p.batches.len(), 2);
    }

    #[test]
    fn the_target_bound_closes_a_batch_of_cheap_candidates() {
        let ix = index(vec![class("src/test/java/a/AaaTests.java", 10, &[])]);
        let pick = ix.pick("src/main/java/a/Aaa.java");
        let order: Vec<_> = ids(MAX_TARGETS as u32 + 3)
            .into_iter()
            .map(|f| (f, pick.clone()))
            .collect();
        let p = plan(&ix, &order);
        assert_eq!(p.batches.len(), 2);
        assert_eq!(p.batches[0].ids.len(), MAX_TARGETS);
        assert_eq!(p.batches[1].ids.len(), 3);
    }
}
