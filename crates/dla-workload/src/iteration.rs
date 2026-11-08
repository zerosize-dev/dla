//! Iteration-argument detection for main and script workloads (spec 4.5).
//!
//! A harness runs a workload once per point with a different amount of work. That needs a
//! way to tell the workload how much work to do. [`detect`] reads the workload's entry file
//! (and the `package.json` script text or the build script that starts it) for the usual
//! ways a program takes an iteration count:
//!
//! | Runtime | Recognised |
//! |---|---|
//! | Go | `flag.Int("n", 2000, ...)`, `flag.Uint64`, `flag.IntVar(&x, "n", 2000, ...)` |
//! | Python | `add_argument("--iterations", ..., default=2000)`, `int(sys.argv[1])` |
//! | Node | the script text (`node load.js --iterations 2000`), quoted `"--iterations"`, an `iterations:` key of the argument defaults object, `.option('--iterations <n>')` |
//! | JVM | `Integer.parseInt(args[0])` (positional), `"--iterations".equals(args[i])`; a `build.sh run` that forwards `$2` |
//! | Rust | `a == "--iterations"` while walking `env::args()`, clap `#[arg(long)] iterations` |
//! | C and C++ | `strcmp(argv[i], "--iterations")`, `getopt` with `case 'n'` |
//!
//! The result is a template with `{n}` (`--iterations {n}`, `-n {n}`, `{n}`) and, when the
//! source shows it, the default count the workload runs without the argument.

use std::sync::LazyLock;

use dla_core::runtime::Runtime;
use regex::Regex;
use serde_json::Value;

use crate::model::{Scan, join};

/// A detected iteration argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IterationArg {
    /// Command-line template with `{n}`: `--iterations {n}`, `-n {n}` or `{n}`.
    pub template: String,
    /// The count the workload runs when it is not given, when the source shows it.
    pub default: Option<u64>,
}

/// Option and variable names that mean "how much work to do", most telling first.
const NAMES: &[&str] = &[
    "iterations",
    "iteration",
    "iters",
    "iter",
    "n",
    "count",
    "requests",
    "batches",
    "runs",
    "rounds",
    "loops",
    "cycles",
    "docs",
    "documents",
    "records",
    "messages",
    "frames",
    "sessions",
    "jobs",
    "reps",
    "repeat",
    "repeats",
    "total",
    "size",
];

/// Position of `name` in the vocabulary (lower is better), or `None`.
fn name_rank(name: &str) -> Option<usize> {
    let norm = name
        .trim_start_matches('-')
        .to_ascii_lowercase()
        .replace('-', "_");
    if let Some(i) = NAMES.iter().position(|n| *n == norm) {
        return Some(i);
    }
    let decorated = ["num_", "n_", "max_"].iter().any(|p| {
        norm.strip_prefix(p)
            .is_some_and(|rest| NAMES.contains(&rest))
    }) || ["_count", "_iterations", "_total"].iter().any(|s| {
        norm.strip_suffix(s)
            .is_some_and(|rest| !rest.is_empty() && rest.len() > 1)
    });
    decorated.then_some(NAMES.len())
}

fn number(s: &str) -> Option<u64> {
    s.replace('_', "").parse().ok()
}

/// A candidate option: its spelling on the command line and its default.
#[derive(Debug, Clone)]
struct Opt {
    /// Spelling as typed (`--iterations`, `-n`).
    flag: String,
    default: Option<u64>,
}

/// Picks the best-named option.
fn best(opts: Vec<Opt>) -> Option<Opt> {
    opts.into_iter()
        .filter_map(|o| name_rank(&o.flag).map(|r| (r, o)))
        .min_by_key(|(r, _)| *r)
        .map(|(_, o)| o)
}

fn arg(o: Opt) -> IterationArg {
    IterationArg {
        template: format!("{} {{n}}", o.flag),
        default: o.default,
    }
}

static GO_FLAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"flag\.(?:Int|Int64|Uint|Uint64)\(\s*"(\w[\w-]*)"\s*,\s*([\d_]+)"#)
        .expect("static regex")
});
static GO_FLAG_VAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"flag\.(?:IntVar|Int64Var|UintVar|Uint64Var)\(\s*[^,]+,\s*"(\w[\w-]*)"\s*,\s*([\d_]+)"#,
    )
    .expect("static regex")
});

fn go(text: &str) -> Option<IterationArg> {
    let mut opts = Vec::new();
    for re in [&*GO_FLAG, &*GO_FLAG_VAR] {
        for c in re.captures_iter(text) {
            opts.push(Opt {
                flag: format!("-{}", &c[1]),
                default: number(&c[2]),
            });
        }
    }
    best(opts).map(arg)
}

static PY_ADD_ARG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"add_argument\(").expect("static regex"));
static PY_OPT_STR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"["'](-{1,2}[A-Za-z][\w-]*)["']"#).expect("static regex"));
static PY_DEFAULT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"default\s*=\s*([\d_]+)").expect("static regex"));
static PY_ARGV: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"int\(\s*sys\.argv\[1\]\s*\)(?:\s*if[^\n]*else\s*([\d_]+))?").expect("static regex")
});
static PY_ARGV_DEFAULT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"sys\.argv\[1\][^\n]*if\s+len\(sys\.argv\)[^\n]*else\s*([\d_]+)")
        .expect("static regex")
});

/// The text between the parentheses opened right before `start` (balanced, string-unaware
/// beyond quotes).
fn call_body(text: &str, start: usize) -> &str {
    let bytes = text.as_bytes();
    let mut depth = 1usize;
    let mut quote: Option<u8> = None;
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' {
                    i += 1;
                } else if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return &text[start..i];
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
    &text[start..]
}

fn python(text: &str) -> Option<IterationArg> {
    let mut opts = Vec::new();
    for m in PY_ADD_ARG.find_iter(text) {
        let body = call_body(text, m.end());
        let default = PY_DEFAULT.captures(body).and_then(|c| number(&c[1]));
        for c in PY_OPT_STR.captures_iter(body) {
            opts.push(Opt {
                flag: c[1].to_owned(),
                default,
            });
        }
    }
    // Prefer the long spelling of the best-named argument.
    if let Some(o) = best(opts.clone()) {
        let long = opts
            .iter()
            .filter(|p| p.flag.starts_with("--") && p.default == o.default)
            .filter(|p| name_rank(&p.flag).is_some())
            .min_by_key(|p| name_rank(&p.flag));
        return Some(arg(long.cloned().unwrap_or(o)));
    }
    if PY_ARGV.is_match(text) {
        let default = PY_ARGV_DEFAULT
            .captures(text)
            .and_then(|c| number(&c[1]))
            .or_else(|| {
                PY_ARGV
                    .captures(text)
                    .and_then(|c| c.get(1))
                    .and_then(|m| number(m.as_str()))
            });
        return Some(IterationArg {
            template: "{n}".into(),
            default,
        });
    }
    None
}

static JS_QUOTED_OPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"["'](-{1,2}[A-Za-z][\w-]*)["']"#).expect("static regex"));
static JS_OPTION_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\.(?:option|requiredOption)\(\s*["']([^"']*)["'](?:[^)]*?,\s*([\d_]+)\s*\))?"#)
        .expect("static regex")
});
static JS_DEFAULTS_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b([A-Za-z_]\w*)\s*:\s*([\d_]+)\b").expect("static regex"));
static JS_PREFIXED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"replace\(\s*/\^--/|slice\(\s*2\s*\)|'--'\s*\+|`--\$\{"#).expect("static regex")
});
static SCRIPT_OPT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|\s)(-{1,2}[A-Za-z][\w-]*)(?:\s+|=)([\d_]+)(?:\s|$)").expect("static regex")
});

/// Options with a numeric value in a shell command line.
fn script_options(cmd: &str) -> Vec<Opt> {
    SCRIPT_OPT
        .captures_iter(cmd)
        .map(|c| Opt {
            flag: c[1].to_owned(),
            default: number(&c[2]),
        })
        .collect()
}

fn node(text: &str, script: Option<&str>) -> Option<IterationArg> {
    if let Some(o) = script.and_then(|s| best(script_options(s))) {
        return Some(arg(o));
    }
    let argv =
        text.contains("process.argv") || text.contains("parseArgs") || text.contains("yargs");
    if !argv {
        return None;
    }
    let mut opts: Vec<Opt> = JS_QUOTED_OPT
        .captures_iter(text)
        .map(|c| Opt {
            flag: c[1].to_owned(),
            default: None,
        })
        .collect();
    for c in JS_OPTION_CALL.captures_iter(text) {
        let spec = &c[1];
        let default = c.get(2).and_then(|m| number(m.as_str()));
        for tok in spec.split([',', ' ', '<', '[', '=']) {
            if tok.starts_with('-') && tok.len() > 1 {
                opts.push(Opt {
                    flag: tok.to_owned(),
                    default,
                });
            }
        }
    }
    // `const args = { iterations: 2000, ... }` with `--${key}` style parsing.
    if JS_PREFIXED.is_match(text) {
        for c in JS_DEFAULTS_KEY.captures_iter(text) {
            if name_rank(&c[1]).is_some() {
                opts.push(Opt {
                    flag: format!("--{}", &c[1]),
                    default: number(&c[2]),
                });
            }
        }
    }
    // A quoted option takes the default of the same-named key when there is one.
    let keys: Vec<(String, u64)> = JS_DEFAULTS_KEY
        .captures_iter(text)
        .filter_map(|c| number(&c[2]).map(|v| (c[1].to_ascii_lowercase(), v)))
        .collect();
    for o in &mut opts {
        if o.default.is_none() {
            let bare = o.flag.trim_start_matches('-').to_ascii_lowercase();
            o.default = keys.iter().find(|(k, _)| *k == bare).map(|(_, v)| *v);
        }
    }
    best(opts).map(arg)
}

static JAVA_POSITIONAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?:Integer\.parseInt|Long\.parseLong|Integer\.valueOf|Long\.valueOf|Integer\.decode)\(\s*args\[(\d)\]\s*\)|args\[(\d)\]\s*\.to(?:Int|Long)\(\)|args\.get(?:OrNull)?\((\d)\)(?:\?)?\.to(?:Int|Long)(?:OrNull)?\(\)",
    )
    .expect("static regex")
});
static JAVA_DEFAULT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:args\.length\s*>\s*0|args\.isNotEmpty\(\))[^:\n]*:\s*([\d_]+)|\?:\s*([\d_]+)")
        .expect("static regex")
});
static JAVA_FLAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""(-{1,2}[A-Za-z][\w-]*)"\s*\.equals\(|\.equals\(\s*"(-{1,2}[A-Za-z][\w-]*)""#)
        .expect("static regex")
});
static SH_SECOND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\{?2(?::-([\d_]+))?\}?").expect("static regex"));

fn jvm(text: &str, script: Option<&str>) -> Option<IterationArg> {
    if let Some(body) = script {
        // `sh ./build.sh run` must pass its second argument on to the program.
        let c = SH_SECOND.captures(body)?;
        let default = c.get(1).and_then(|m| number(m.as_str()));
        let from_source = JAVA_DEFAULT
            .captures(text)
            .and_then(|c| c.get(1).or_else(|| c.get(2)))
            .and_then(|m| number(m.as_str()));
        return Some(IterationArg {
            template: "{n}".into(),
            default: default.or(from_source),
        });
    }
    let flags: Vec<Opt> = JAVA_FLAG
        .captures_iter(text)
        .filter_map(|c| c.get(1).or_else(|| c.get(2)))
        .map(|m| Opt {
            flag: m.as_str().to_owned(),
            default: None,
        })
        .collect();
    if let Some(o) = best(flags) {
        return Some(arg(o));
    }
    let pos = JAVA_POSITIONAL
        .captures(text)
        .and_then(|c| c.get(1).or_else(|| c.get(2)).or_else(|| c.get(3)))?;
    if pos.as_str() != "0" {
        return None;
    }
    let default = JAVA_DEFAULT
        .captures(text)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .and_then(|m| number(m.as_str()));
    Some(IterationArg {
        template: "{n}".into(),
        default,
    })
}

static RUST_OPT_CMP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"==\s*"(-{1,2}[A-Za-z][\w-]*)"|"(-{1,2}[A-Za-z][\w-]*)"\s*=="#)
        .expect("static regex")
});
static RUST_MATCH_ARM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\s*"(-{1,2}[A-Za-z][\w-]*)"(?:\s*\|\s*"(-{1,2}[A-Za-z][\w-]*)")*\s*=>"#)
        .expect("static regex")
});
static RUST_NONE_DEFAULT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"None\s*=>\s*([\d_]+)").expect("static regex"));
static RUST_UNWRAP_DEFAULT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"unwrap_or\(\s*([\d_]+)\s*\)").expect("static regex"));
static RUST_CLAP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)#\[(?:arg|clap)\(([^\]]*)\)\]\s*(?:pub\s+)?(\w+)\s*:\s*(?:Option<)?(?:u|i)(?:8|16|32|64|128|size)",
    )
    .expect("static regex")
});
static RUST_CLAP_DEFAULT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"default_value(?:_t)?\s*=\s*"?([\d_]+)"?"#).expect("static regex")
});

fn rust(text: &str) -> Option<IterationArg> {
    let mut opts: Vec<Opt> = Vec::new();
    let default = RUST_NONE_DEFAULT
        .captures(text)
        .or_else(|| RUST_UNWRAP_DEFAULT.captures(text))
        .and_then(|c| number(&c[1]));
    for c in RUST_OPT_CMP.captures_iter(text) {
        if let Some(m) = c.get(1).or_else(|| c.get(2)) {
            opts.push(Opt {
                flag: m.as_str().to_owned(),
                default,
            });
        }
    }
    for line in text.lines() {
        if let Some(c) = RUST_MATCH_ARM.captures(line) {
            for m in c.iter().skip(1).flatten() {
                opts.push(Opt {
                    flag: m.as_str().to_owned(),
                    default,
                });
            }
        }
    }
    for c in RUST_CLAP.captures_iter(text) {
        let attrs = &c[1];
        let field = c[2].replace('_', "-");
        let flag = if attrs.contains("long") {
            format!("--{field}")
        } else if attrs.contains("short") {
            format!("-{}", field.chars().next().unwrap_or('n'))
        } else {
            continue;
        };
        opts.push(Opt {
            flag,
            default: RUST_CLAP_DEFAULT
                .captures(attrs)
                .and_then(|d| number(&d[1])),
        });
    }
    best(opts).map(arg)
}

static C_STRCMP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"strcmp\(\s*argv\[\s*\w+\s*\]\s*,\s*"(-{1,2}[A-Za-z][\w-]*)"\s*\)\s*==\s*0([^\n]*\n[^\n]*)"#)
        .expect("static regex")
});
static C_GETOPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"getopt(?:_long)?\([^"]*"([^"]*)""#).expect("static regex"));
static C_CASE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"case\s+'(\w)'\s*:[^\n]*\n?[^\n]*\n?[^\n]*(?:atoi|strtoul|strtol|strtoull|atol)\([^)]*optarg")
        .expect("static regex")
});
static C_ASSIGN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\w+)\s*=\s*(?:\(\w+\)\s*)?(?:strtoul|strtol|strtoull|atoi|atol)\(")
        .expect("static regex")
});

fn c_default(text: &str, var: &str) -> Option<u64> {
    let re = Regex::new(&format!(
        r"\b{}\s*=\s*([\d_]+)[uUlL]*\s*[;,]",
        regex::escape(var)
    ))
    .ok()?;
    re.captures(text).and_then(|c| number(&c[1]))
}

fn native(text: &str) -> Option<IterationArg> {
    let mut opts = Vec::new();
    for c in C_STRCMP.captures_iter(text) {
        let var = C_ASSIGN.captures(&c[2]).map(|a| a[1].to_owned());
        opts.push(Opt {
            flag: c[1].to_owned(),
            default: var.and_then(|v| c_default(text, &v)),
        });
    }
    if let Some(spec) = C_GETOPT.captures(text) {
        let letters = spec[1].to_owned();
        for c in C_CASE.captures_iter(text) {
            let ch = &c[1];
            if letters.contains(ch) {
                let var = C_ASSIGN.captures(&c[0]).map(|a| a[1].to_owned());
                opts.push(Opt {
                    flag: format!("-{ch}"),
                    default: var.and_then(|v| c_default(text, &v)),
                });
            }
        }
    }
    best(opts).map(arg)
}

/// The command line of the `package.json` script that `npm run <name>` (or `npm start`,
/// `npm test`) runs, from the manifest text.
pub fn npm_script_in(manifest_text: &str, command: &str) -> Option<String> {
    let mut words = command.split_whitespace();
    if words.next()? != "npm" {
        return None;
    }
    let name = match words.next()? {
        "run" | "run-script" => words.next()?,
        "start" => "start",
        "test" => "test",
        _ => return None,
    };
    let manifest: Value = serde_json::from_str(manifest_text).ok()?;
    manifest
        .get("scripts")?
        .get(name)?
        .as_str()
        .map(str::to_owned)
}

/// [`npm_script_in`] for the `package.json` in the workload's `cwd`.
pub fn npm_script(scan: &Scan, cwd: &str, command: &str) -> Option<String> {
    let dir = if cwd == "." { "" } else { cwd };
    npm_script_in(&scan.read(&join(dir, "package.json"))?, command)
}

/// Detects the iteration argument of a main or script workload.
///
/// `command`, `cwd` and `file` are the workload's fields; `srcs` its source files. Returns
/// `None` when the workload shows no iteration count (the caller then needs `--iter-arg`).
pub fn detect(
    scan: &Scan,
    runtime: Runtime,
    command: &str,
    cwd: &str,
    file: &str,
    srcs: &[String],
) -> Option<IterationArg> {
    let mut texts: Vec<String> = Vec::new();
    for rel in std::iter::once(file).chain(srcs.iter().map(String::as_str)) {
        if let Some(t) = scan
            .infos
            .get(rel)
            .map(|f| f.text.clone())
            .or_else(|| scan.read(rel))
            && !texts.contains(&t)
        {
            texts.push(t);
        }
    }
    let text = texts.join("\n");
    match runtime {
        Runtime::Go => go(&text),
        Runtime::Python => python(&text),
        Runtime::Node => {
            let script = npm_script(scan, cwd, command);
            node(&text, script.as_deref())
        }
        Runtime::Jvm => {
            let dir = cwd_dir(cwd);
            let script = command
                .split_whitespace()
                .find(|w| w.ends_with("build.sh") || w.ends_with("run.sh"))
                .and_then(|w| scan.read(&join(dir, w.trim_start_matches("./"))));
            if command.contains(".sh") && script.is_none() {
                return None;
            }
            jvm(&text, script.as_deref())
        }
        Runtime::Rust => rust(&text),
        Runtime::Native => native(&text),
    }
}

fn cwd_dir(cwd: &str) -> &str {
    if cwd == "." { "" } else { cwd }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(arg: Option<IterationArg>) -> (Option<String>, Option<u64>) {
        arg.map_or((None, None), |a| (Some(a.template), a.default))
    }

    #[test]
    fn go_flags() {
        let src = r#"n := flag.Int("n", 2000, "number of batches to replay")
workers := flag.Int("workers", 4, "x")"#;
        assert_eq!(t(go(src)), (Some("-n {n}".into()), Some(2000)));
        let var = r#"flag.IntVar(&count, "requests", 1_000, "x")"#;
        assert_eq!(t(go(var)), (Some("-requests {n}".into()), Some(1000)));
        assert_eq!(go(r#"flag.Int("workers", 4, "x")"#), None);
    }

    #[test]
    fn python_argparse_and_argv() {
        let src = "parser.add_argument(\"--iterations\", type=int, default=2000)\nparser.add_argument('--seed', default=7)";
        assert_eq!(
            t(python(src)),
            (Some("--iterations {n}".into()), Some(2000))
        );
        let short = "p.add_argument('-n', '--count', type=int, default=50)";
        assert_eq!(t(python(short)), (Some("--count {n}".into()), Some(50)));
        let argv = "n = int(sys.argv[1]) if len(sys.argv) > 1 else 300";
        assert_eq!(t(python(argv)), (Some("{n}".into()), Some(300)));
        assert_eq!(python("print('hi')"), None);
    }

    #[test]
    fn node_script_text_and_source() {
        let from_script = node("", Some("node load.js --iterations 2000"));
        assert_eq!(
            t(from_script),
            (Some("--iterations {n}".into()), Some(2000))
        );
        let src = "function parseArgs(argv) {\n const args = { iterations: 2000, wave: 50 };\n args[argv[i].replace(/^--/, '')] = 1; }\nparseArgs(process.argv.slice(2))";
        assert_eq!(
            t(node(src, None)),
            (Some("--iterations {n}".into()), Some(2000))
        );
        let quoted = "if (process.argv[2] === '-n') count = 3";
        assert_eq!(t(node(quoted, None)), (Some("-n {n}".into()), None));
        assert_eq!(node("console.log(1)", None), None);
    }

    #[test]
    fn jvm_positional_flag_and_script() {
        let src = "int total = args.length > 0 ? Integer.parseInt(args[0]) : 5000;";
        assert_eq!(t(jvm(src, None)), (Some("{n}".into()), Some(5000)));
        let flag = r#"if ("--iterations".equals(args[i])) {"#;
        assert_eq!(t(jvm(flag, None)), (Some("--iterations {n}".into()), None));
        let sh = "if [ \"$1\" = \"run\" ]; then java -cp out shop.Main \"${2:-5000}\"; fi";
        assert_eq!(t(jvm(src, Some(sh))), (Some("{n}".into()), Some(5000)));
        assert_eq!(jvm(src, Some("java -cp out shop.Main")), None);
        assert_eq!(jvm("int a = Integer.parseInt(args[1]);", None), None);
        let kt = "val n = args.getOrNull(0)?.toInt() ?: 300";
        assert_eq!(t(jvm(kt, None)), (Some("{n}".into()), Some(300)));
    }

    #[test]
    fn rust_manual_and_clap() {
        let src =
            "match args.iter().position(|a| a == \"--iterations\") { Some(i) => 1, None => 500 }";
        assert_eq!(t(rust(src)), (Some("--iterations {n}".into()), Some(500)));
        let clap = "#[arg(long, default_value_t = 1000)]\n    iterations: u64,";
        assert_eq!(t(rust(clap)), (Some("--iterations {n}".into()), Some(1000)));
        assert_eq!(rust("fn main() {}"), None);
    }

    #[test]
    fn c_strcmp_and_getopt() {
        let src = "unsigned long iterations = 20000;\nif (strcmp(argv[i], \"--iterations\") == 0 && i + 1 < argc)\n    iterations = strtoul(argv[++i], NULL, 10);\nelse if (strcmp(argv[i], \"--seed\") == 0)\n    seed = strtoul(argv[++i], NULL, 10);";
        assert_eq!(
            t(native(src)),
            (Some("--iterations {n}".into()), Some(20000))
        );
        let opt = "long n = 100;\nwhile ((c = getopt(argc, argv, \"n:\")) != -1) {\n switch (c) {\n case 'n':\n  n = atol(optarg);\n  break;\n }\n}";
        assert_eq!(t(native(opt)), (Some("-n {n}".into()), Some(100)));
    }
}
