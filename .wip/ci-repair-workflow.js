export const meta = {
  name: 'ci-repair-u8',
  description: 'Make CI green on Ubuntu 26.04, macOS and Windows: four fix packages (lints and Windows, Ubuntu runner and JDK 25, macOS, benchmark jobs), each checked and refixed once, then a loop that pushes a snapshot, waits for CI, and fixes what the runners report',
  phases: [{ title: 'Fix' }, { title: 'Check' }, { title: 'CI' }, { title: 'Accept' }],
}
const SP = '/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad'
const BR = `${SP}/u8-brief.md`
const NAMES = { K1: 'Rust 1.100 lints, Windows, doc leftovers', K2: 'the Ubuntu 26.04 runner and JDK 25', K3: 'the macOS runner', K4: 'the benchmark jobs' }
const CTX = `Repository /home/user/dla (Rust workspace and the DLA Claude Code plugin), branch main at 63a30f2; \`git diff HEAD\` shows only this repair. Do NOT commit or push. Read ${BR} in full first: what changed on the CI runners, the rules (disk, -j 2, never weaken a test to get green, tests seen failing, VERIFY rows, shared files, checks before reporting) and your package's section. Up to two packages edit the tree at once: re-read a shared file right before each edit, change only your own lines, never revert what you did not write. If a test binary vanishes or is busy, another agent rebuilt target/: rerun it. Never kill processes by name.`
const CHECK = {
  type: 'object',
  properties: {
    problems: { type: 'array', items: { type: 'object', properties: {
      severity: { type: 'string', enum: ['problem', 'minor', 'note'] }, where: { type: 'string' }, text: { type: 'string' } },
      required: ['severity', 'where', 'text'] } },
    confirmed: { type: 'string' }, unverified: { type: 'string' },
  },
  required: ['problems', 'confirmed', 'unverified'],
}
const CI = {
  type: 'object',
  properties: {
    sha: { type: 'string' }, green: { type: 'boolean' }, conclusion: { type: 'string' },
    failedJobs: { type: 'array', items: { type: 'object', properties: {
      job: { type: 'string' }, log: { type: 'string' }, summary: { type: 'string' } }, required: ['job', 'log', 'summary'] } },
    notes: { type: 'string' },
  },
  required: ['sha', 'green', 'conclusion', 'failedJobs'],
}
const snap = label => agent(`Run exactly this one command and return its output verbatim, nothing else. Do not run any other git command and do not edit any file:\n\nbash ${SP}/u7/snapshot.sh "${label}"`, { label: `snapshot:${label}`, phase: 'Fix', model: 'sonnet', effort: 'low' })
const build = k => agent(`${CTX}\n\nYou fix package ${k} (${NAMES[k]}) of ${BR}. Scratch dir ${SP}/u8-${k}/ (create it; delete only it, by literal path). Find the cause of each item first (the CI log, the code, a local reproduction where one is possible), then fix it. End with the report the brief asks for.`, { label: `fix:${k}`, phase: 'Fix', model: 'sonnet' })
const check = (k, report, round) => agent(`${CTX}\n\nYou check package ${k} (${NAMES[k]}) of ${BR}. Do not edit repository files; create and delete files only under ${SP}/u8chk-${k}/. For every item of the package: is the cause the fixer names the real one (read the CI log lines and the code yourself), is the fix in the right place (a product defect fixed in the product, a test changed only where it asserted something untrue on a supported platform), was any test skipped, ignored or weakened to get green, does each new or changed test fail without the change (by reading \`git diff HEAD\`, and by running on a scratch copy where cheap)? Run the package's tests yourself, and for the Ubuntu and JDK items run them with JDK 25 first on PATH as well as with the default JDK 21. Try at least two adversarial cases per behaviour change. Every remaining defect is a problem (behaviour, a weakened or vacuous test, a wrong cause, a false VERIFY row, a clippy break on any target), minor (wording, a small gap) or a note. Give file:line. Say which items only the macOS or Windows runner can confirm.\n\nFixer report${round > 1 ? ' (after a refix)' : ''}:\n${report}`, { label: `check${round}:${k}`, phase: 'Check', schema: CHECK })
const refix = (k, report, chk) => agent(`${CTX}\n\nYou refix package ${k} (${NAMES[k]}) of ${BR}. Its fixer's report and an independent check follow. Fix every "problem" and "minor" item of the check inside your package's scope (a "note" needs nothing), each with a test you SEE fail without it where it is behaviour; re-run what the brief asks before reporting. Where a check item is wrong, say why with evidence instead of changing code. Keep every case the check confirmed working. Scratch dir ${SP}/u8-${k}/. End with the full report (the state of every item).\n\nFixer report:\n${report}\n\nCheck:\n${JSON.stringify(chk, null, 1)}`, { label: `refix:${k}`, phase: 'Fix', model: 'sonnet' })
const openOf = c => ((c && c.problems) || []).filter(x => x.severity !== 'note')
const runPkg = async k => {
  let report = await build(k)
  await snap(`${k} fixed`)
  let chk = await check(k, report, 1)
  if (openOf(chk).length) {
    log(`${k}: ${openOf(chk).length} open check items, refix`)
    report = await refix(k, report, chk)
    await snap(`${k} refix`)
    chk = await check(k, report, 2)
  }
  const left = openOf(chk).length
  if (left) log(`${k}: ${left} items still open after the refix`)
  return { key: k, report, chk }
}
phase('Fix')
const results = (await parallel(['K2', 'K3', 'K1', 'K4'].map(k => () => runPkg(k)))).filter(Boolean)
const leftovers = results.map(r => ({ key: r.key, open: openOf(r.chk) })).filter(r => r.open.length)
phase('CI')
const SOLO = CTX.replace('Up to two packages edit the tree at once: re-read a shared file right before each edit, change only your own lines, never revert what you did not write. ', 'You are the only editor now. ')
const ciRun = n => agent(`You push a snapshot of /home/user/dla to the branch wip/progress and wait for its CI run. Do not edit any repository file and run no git command other than the script below.\n1. Run: bash ${SP}/u7/snapshot.sh "ci attempt ${n}". It prints either "snapshot <sha> (...)" or "no change since <sha>"; take the 40-character <sha>.\n2. Run: bash ${SP}/u8/ci-wait.sh <sha> ${SP}/u8/ci-${n} (use a Bash timeout of 590000 ms). It waits up to 8 minutes and prints "PENDING ..." if the run has not finished: run it again, up to 14 times in all (a run takes 30 to 70 minutes). When it prints "DONE run=<id> conclusion=<c>", it lists every job's conclusion and the log file of each failed job.\n3. For each failed job, read its log file (grep for "FAILED", "panicked", "error", "is not", "failed" and read around the hits) and write a summary of at most 25 lines: the failing test names or step, the assertion or error text, and anything the log shows about the cause. A job named "integration suite (windows-2025, WHP)" is allowed to fail and does not count.\nReturn: sha, green (true only when the run's conclusion is success), conclusion, failedJobs (job name, log path, summary), notes. If the run never finishes in 14 waits, return green false with conclusion "timeout".`, { label: `ci:attempt${n}`, phase: 'CI', schema: CI, model: 'sonnet' })
let ci = null
const ciHistory = []
for (let n = 1; n <= 5; n++) {
  ci = await ciRun(n)
  ciHistory.push(ci)
  if (!ci || ci.green) break
  log(`CI attempt ${n}: ${ci.conclusion}, failed: ${(ci.failedJobs || []).map(j => j.job).join('; ')}`)
  if (n === 5) break
  const fr = await agent(`${SOLO}\n\nThe CI run of the current tree (commit ${ci.sha} on wip/progress) failed. Its failed jobs, with the path of each job's full log and a summary, follow. For each failure: read the log yourself, find the cause (the runner's platform, toolchain or tools; or a defect of this repair or of the product), reproduce it here if it can be reproduced on Linux (JDK 25 is at /usr/lib/jvm/java-25-openjdk-amd64, Rust is 1.100), and fix it under the rules of ${BR}: never skip, ignore or weaken a test to get green; a product defect is fixed in the product; add the diagnostics a failed assertion needs when the cause cannot be seen from the log, so the next run explains itself. Run the local checks the brief lists. Update the docs/VERIFY.md rows of the items you touch. Scratch dir ${SP}/u8-ci-fix/. Report: each failure, its cause and evidence, the fix, tests and which you saw fail, what only the next CI run can confirm.${leftovers.length && n === 1 ? `\n\nAlso fix these items the package checks left open:\n${JSON.stringify(leftovers, null, 1)}` : ''}\n\nEarlier attempts and their failures (do not undo their fixes):\n${JSON.stringify(ciHistory.slice(0, -1).map(c => ({ sha: c.sha, failed: (c.failedJobs || []).map(j => j.job + ': ' + j.summary.slice(0, 400)) })), null, 1)}\n\nThis run:\n${JSON.stringify(ci, null, 1)}`, { label: `ci-fix${n}`, phase: 'CI', model: 'sonnet' })
  const rv = await agent(`${SOLO}\n\nYou review the fix made after a failed CI run (commit ${ci.sha}). Do not edit repository files; scratch only under ${SP}/u8chk-ci/. Check against the rules of ${BR}: was any test skipped, ignored or weakened to get green; is each named cause supported by the CI log (read the log lines: paths below) and the code; is a product defect fixed in the product; does each new or changed test fail without the change; do \`cargo clippy --workspace --all-targets -j 2 -- -D warnings\`, the Windows-target clippy for dla-core, dla-probe and dla-report, \`cargo fmt --all --check\` and the tests of the touched crates pass. Report problems (a weakened test, a wrong cause, a break), minors and notes with file:line.\n\nFailed jobs and logs:\n${JSON.stringify(ci.failedJobs, null, 1)}\n\nFixer report:\n${fr}`, { label: `ci-review${n}`, phase: 'CI', schema: CHECK })
  if (openOf(rv).length) {
    log(`CI fix ${n}: review found ${openOf(rv).length} items, refix`)
    await agent(`${SOLO}\n\nA review of the fix made after a failed CI run found the items below. Fix every "problem" and "minor" one under the rules of ${BR} (never weaken a test; keep what the review confirmed). Scratch dir ${SP}/u8-ci-fix/. Report each item's state and the checks you ran.\n\nReview:\n${JSON.stringify(rv, null, 1)}\n\nFixer report:\n${fr}`, { label: `ci-refix${n}`, phase: 'CI', model: 'sonnet' })
  }
}
phase('Accept')
const acc = await agent(`Repository /home/user/dla, branch main. Do NOT commit, push or edit repository files. You run the local acceptance of the working tree after the CI repair. Disk: if \`df --output=avail -B1M /\` is below 3000, run \`bash ${SP}/prune-superseded.sh\`. Cargo with -j 2, shared target/ only, never kill processes by name. Scratch only under ${SP}/u8acc/. Run and record each result: (1) cargo test -j 2 --no-fail-fast --workspace with the default JDK 21 (rerun a failed test alone twice: steady or flaky); (2) the JVM-dependent test binaries again with JDK 25 first on PATH and JAVA_HOME=/usr/lib/jvm/java-25-openjdk-amd64 (dla-sample; dla-cli: sample, attach, alloc_test_jvm, summarize_conformance and the harness tests; dla-workload: every harness_* test); (3) cargo clippy --workspace --all-targets -j 2 -- -D warnings; cargo clippy --target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report --all-targets -j 2 -- -D warnings; (4) cargo fmt --all --check; (5) target/debug/dla schema --check; (6) python3 bench/runner/test_grade.py; (7) ./target/debug/dla-bench static --dla-bin ./target/debug/dla (both with and without --archetypes off) and ./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json ${SP}/u8acc/dynamic.json: both must exit 0; compare the dynamic result with ${SP}/p2r3-acc/on/result.json for the fixtures still under bench/fixtures (a fixture that passed there and fails now is a regression; rerun it once alone); (8) \`git status --short\` summary and a scan of \`git diff HEAD\` for model names, attribution text or scratch paths. Return a plain report: each step pass/fail with counts, regressions with evidence, and the disk at the end.`, { label: 'accept', phase: 'Accept' })
await snap('after the CI repair acceptance')
return { packages: results.map(r => ({ key: r.key, open: openOf(r.chk), report: r.report })), ci: ciHistory, acceptance: acc }
