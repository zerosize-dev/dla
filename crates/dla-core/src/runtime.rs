//! Runtime (worker) names.

string_enum! {
    /// A language runtime DLA investigates. The wire spelling is also the default
    /// worker key in `coverage.json` and the value of `--lang`.
    pub enum Runtime {
        /// Java, Kotlin and other JVM languages.
        Jvm = "jvm",
        /// Go.
        Go = "go",
        /// Rust.
        Rust = "rust",
        /// Node.js (JavaScript and TypeScript).
        Node = "node",
        /// Python.
        Python = "python",
        /// C and C++.
        Native = "native",
    }
}

impl Runtime {
    /// The accepted `--lang` spellings, joined for help and error text.
    pub fn choices() -> String {
        Self::ALL
            .iter()
            .map(|r| r.as_str())
            .collect::<Vec<_>>()
            .join("|")
    }
}
