# Brief HO: pick projects for a blind test of a leak-finding tool

You do research only. You change nothing in /home/user/dla and you run no tool from it.
Your scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u106
(create it; write only there).

## What is needed

A tool that looks for memory leaks in a project's source tree is to be tested on open-source
projects it has never been run on. For each project there must be one documented memory leak at
a known version, so that the result of a run can be scored afterwards.

Find TWO projects for each of five runtimes: Python, Go, Node.js (JavaScript or TypeScript),
Rust, JVM (Java or Kotlin). Ten in all.

NOT allowed (already used): aiohttp, grpc-go, axios, tokio (any crate of it), tokio-util,
spring-data-commons (and any other Spring project), Jetty, Pillow.

## Criteria (all must hold; check each one, do not assume)

1. A public issue or advisory that reports a real memory leak in the project's own code (growth
   without bound under a use that the public API allows), and a fix commit or pull request that
   a later release contains. Not a leak in a dependency. Not a leak that exists only in the
   project's tests, examples or benchmarks. Not a file-descriptor-only or goroutine-only leak
   unless memory grows with it.
2. A release tag (or a commit) that still has the leak: the last release before the fix. Check
   that the tag exists (`git ls-remote --tags <url> | grep <tag>`).
3. The project is a library or framework with a public API and a test suite, between about
   5 000 and 150 000 lines of source in the main language. Well known (a few thousand stars or
   widely depended on).
4. It can be built with package registries only (PyPI, npm, crates.io, the Go proxy, Maven
   Central), on Linux x86_64, without a database, a cloud service, a GPU or Docker. Toolchains
   on the machine: Python 3.13 (3.12 available), Go, Node, Rust stable, JDK. A pure-Python
   project must work on Python 3.12 or 3.13 at the leaky version.
5. The leak can be reached in one process through the public API in a loop (no need for hours
   of traffic or a second machine).

## How to choose (this matters)

Do NOT consider how easy or hard the leak would be for a tool to find, and do not prefer one
kind of leak. Take the first projects that meet the criteria in the order your search finds
them, and write down the searches you ran. If you looked at a project and dropped it, list it
with the criterion that failed. Different kinds of leak across the ten are welcome but not a
goal.

Do not clone whole repositories unless you must check a criterion; a shallow clone of one tag
into your scratch directory is fine (`git clone --depth 1 --branch <tag> <url>`), delete nothing
outside your scratch directory.

## Output: TWO files in your scratch directory

`candidates.md` (the test runner reads this BEFORE the runs; it must not give the answer away):
for each project: runtime; name; repository URL; the leaky tag or commit; lines of source
(rough); the exact commands to fetch that version and install its dependencies so that its
package imports or builds (try them if cheap, and say whether you tried); the test command. NO
mention of the issue, the leaking file, function, class or mechanism. No link to the issue or
the fix.

`answer-key.md` (read only AFTER the runs): for each project: links to the issue and the fix;
the fixed version; file and line (at the leaky tag) of the place that keeps the memory, and of
the code that adds to it; the mechanism in two sentences; the public API calls that trigger it;
how sure you are that the leak is present at the tag (say what you checked: the diff of the fix
applies to the tag, or you read the code at the tag).

## Report (your final message)

One line per project: runtime, name, tag, tried the install (yes/no). Nothing about the leaks.
Then the dropped projects with one reason each, and what you could not check.

Rules: plain English, full sentences. No `rm -rf` on variable paths. Do not kill processes by
name. If an action is denied, stop and report it; do not ask anyone to do it for you.
