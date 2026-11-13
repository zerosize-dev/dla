//! The backend registry: which runtimes a backend serves, the programs that provide it,
//! and the hint shown when it is missing.

use dla_core::runtime::Runtime;

/// What DLA knows about one backend.
#[derive(Debug, Clone, Copy)]
pub struct Backend {
    /// Name used in detector tables, coverage and `dla install`.
    pub name: &'static str,
    /// Runtimes whose detector tables may reference it.
    pub runtimes: &'static [Runtime],
    /// Program names searched for (tools dir first, then `PATH`).
    pub programs: &'static [&'static str],
    /// Seconds a single run may take.
    pub timeout_secs: u64,
    /// One line describing the tool and where it comes from.
    pub summary: &'static str,
}

use Runtime::{Go, Jvm, Native, Node, Python, Rust};

/// Every backend of the detector contract, in `KNOWN_BACKENDS` order.
pub const REGISTRY: [Backend; 13] = [
    Backend {
        name: "dla",
        runtimes: &[Jvm, Go, Rust, Node, Python, Native],
        programs: &[],
        timeout_secs: 120,
        summary: "DLA-owned rules over tree-sitter (built in)",
    },
    Backend {
        name: "staticcheck",
        runtimes: &[Go],
        programs: &["staticcheck"],
        timeout_secs: 600,
        summary: "honnef.co/go/tools staticcheck",
    },
    Backend {
        name: "golangci-lint",
        runtimes: &[Go],
        programs: &["golangci-lint"],
        timeout_secs: 900,
        summary: "golangci-lint v2 with an explicit linter list",
    },
    Backend {
        name: "govet",
        runtimes: &[Go],
        programs: &["go"],
        timeout_secs: 600,
        summary: "go vet (lostcancel), part of the Go toolchain",
    },
    Backend {
        name: "errorprone",
        runtimes: &[Jvm],
        programs: &["javac"],
        timeout_secs: 900,
        summary: "Error Prone javac plugin",
    },
    Backend {
        name: "spotbugs",
        runtimes: &[Jvm],
        programs: &["spotbugs"],
        timeout_secs: 900,
        summary: "SpotBugs on compiled classes",
    },
    Backend {
        name: "detekt",
        runtimes: &[Jvm],
        programs: &["detekt2", "detekt"],
        timeout_secs: 900,
        summary: "detekt 2.x, full analysis mode",
    },
    Backend {
        name: "codeql",
        runtimes: &[Jvm, Python],
        programs: &["codeql"],
        timeout_secs: 3600,
        summary: "CodeQL CLI bundle with the java and python query packs",
    },
    Backend {
        name: "sonar",
        runtimes: &[Jvm],
        programs: &["sonar-scanner"],
        timeout_secs: 600,
        summary: "SonarQube / SonarCloud (server-only: always blocked, DLA queries no server)",
    },
    Backend {
        name: "ruff",
        runtimes: &[Python],
        programs: &["ruff"],
        timeout_secs: 300,
        summary: "ruff linter",
    },
    Backend {
        name: "pylint",
        runtimes: &[Python],
        programs: &["pylint"],
        timeout_secs: 600,
        summary: "pylint with only the listed messages enabled",
    },
    Backend {
        name: "eslint",
        runtimes: &[Node],
        programs: &["eslint"],
        timeout_secs: 900,
        summary: "eslint with typescript-eslint and eslint-plugin-rxjs-x",
    },
    Backend {
        name: "rapx",
        runtimes: &[Rust],
        programs: &["cargo-rapx"],
        timeout_secs: 900,
        summary: "RAPx leak checker (cargo rapx check -m, nightly)",
    },
];

/// The registry entry named `name`.
pub fn backend(name: &str) -> Option<&'static Backend> {
    REGISTRY.iter().find(|b| b.name == name)
}

/// The hint stored in coverage for a missing backend.
pub fn install_hint(name: &str) -> String {
    match name {
        "sonar" => "nothing to install: Sonar rules are evaluated on a SonarQube or SonarCloud server, which DLA does not query".to_owned(),
        "govet" => "install the Go toolchain (https://go.dev/dl/); go vet ships with it".to_owned(),
        other => format!("dla install {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::detectors::KNOWN_BACKENDS;

    #[test]
    fn registry_matches_the_contract_list() {
        let names: Vec<_> = REGISTRY.iter().map(|b| b.name).collect();
        assert_eq!(names, KNOWN_BACKENDS);
    }

    #[test]
    fn hints_name_the_installer() {
        assert_eq!(install_hint("ruff"), "dla install ruff");
        let sonar = install_hint("sonar");
        assert!(sonar.starts_with("nothing to install"), "{sonar}");
        assert!(!sonar.contains("install sonar-scanner"), "{sonar}");
    }
}
