export const meta = {
  name: 'fix-pass-u9',
  description: 'Last fix pass: flaky non-leak verdicts in alloc-test (N1) and the 14 open check items (N2), each fixed, checked and refixed; then a snapshot, CI on the three runners with a fix loop, and local acceptance',
  phases: [{ title: 'Fix' }, { title: 'Check' }, { title: 'CI' }, { title: 'Accept' }],
}
const SP = '/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad'
const BR = `${SP}/u9-brief.md`
const NAMES = { N1: 'alloc-test reads a flat column as inconclusive when it happens to vote', N2: 'the items the last checks left open' }
const CTX = `Repository /home/user/dla (Rust workspace and the DLA Claude Code plugin), branch main at 9781542; \`git diff HEAD\` shows only this pass. Do NOT commit or push. Read ${BR} in full first: the rules (disk, -j 2, never weaken a test to get green, tests seen failing, VERIFY rows, shared files, checks before reporting) and your package's section with its decisions. Two packages edit the tree at once: re-read a shared file right before each edit, change only your own lines, never revert what you did not write. If a test binary vanishes or is busy, another agent rebuilt target/: rerun it. Never kill processes by name.`
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
const build = k => agent(`${CTX}\n\nYou fix package ${k} (${NAMES[k]}) of ${BR}. Scratch dir ${SP}/u9-${k}/ (create it; delete only it, by literal path). Confirm the cause of each item first (the evidence files the brief names, the code, a local reproduction where one is possible), then fix it following the decisions. End with the report the brief asks for.`, { label: `fix:${k}`, phase: 'Fix', model: 'sonnet' })
const SCOPE = {
  N1: `Run the unit and command-level tests of the change yourself, and the whole test suites of the touched crates once. Try at least three adversarial cases per decision on prepared series (for example: a byte column whose growth is just above the minimum and retained; a column with no minimum; a real leak in file-backed memory; a Go heap leak of 100 KB with a retained purge; a bound finding tied to os.rss_file), and say for each whether a case that read leak or inconclusive for a good reason before now reads non-leak. Read every harness purge implementation the fixer cites. Run at most four fixtures with \`dla-bench dynamic --parallel 1 --only <fixture>\` (your own copies of the binaries); the full dynamic run happens once at the end, not in your check.`,
  N2: `Go through all 14 items of ${SP}/u9/open-items.json one by one. For the JVM warm-up test read the fixer's measurement logs, then repeat a sample yourself: 6 runs with the warm-up on while three other copies of the measurement run beside it, and 5 runs with the warm-up off, on JDK 21, and 3 and 3 with JDK 25 first on PATH (/usr/lib/jvm/java-25-openjdk-amd64); judge whether the new assertions can fail with the product working and whether they still catch a missing warm-up, and compare them with the macOS figures in the logs the brief names. Run the JNI test on JDK 21 (it must run, not skip) and on JDK 25. Check every changed docs/VERIFY.md row cell by cell (4 cells, every named test exists, counts are what a run shows, no plan names). Run the whole test suites of the touched crates once.`,
}
const check = (k, report, round) => agent(`${CTX}\n\nYou check package ${k} (${NAMES[k]}) of ${BR}. Do not edit repository files; create and delete files only under ${SP}/u9chk-${k}/. For every item and decision of the package: is the cause the real one (read the evidence yourself), is the fix what the decision says and in the right place, was any test skipped, ignored or weakened without the proof the rules ask for, does each new or changed test fail without the change (by reading \`git diff HEAD\`, and by running on a scratch copy where cheap), are the docs and VERIFY rows true? ${SCOPE[k]} Keep the machine time of your own runs to about 75 minutes. Every remaining defect is a problem (behaviour, a weakened or vacuous test, a wrong cause, a false VERIFY row or doc, a clippy break on any target, a worse fixture), minor (wording, a small gap) or a note (no action). Give file:line.\n\nFixer report${round > 1 ? ' (after a refix)' : ''}:\n${report}`, { label: `check${round}:${k}`, phase: 'Check', schema: CHECK })
const refix = (k, report, chk, n) => agent(`${CTX}\n\nYou refix package ${k} (${NAMES[k]}) of ${BR}. Its fixer's report and an independent check follow. Fix every "problem" and "minor" item of the check inside your package's scope (a "note" needs nothing), each with a test you SEE fail without it where it is behaviour; re-run what the brief asks before reporting. Where a check item is wrong, say why with evidence instead of changing code. Keep every case the check confirmed working. Scratch dir ${SP}/u9-${k}/. End with the full report (the state of every item).\n\nFixer report:\n${report}\n\nCheck:\n${JSON.stringify(chk, null, 1)}`, { label: `refix${n}:${k}`, phase: 'Fix', model: 'sonnet' })
const openOf = c => ((c && c.problems) || []).filter(x => x.severity !== 'note')
const hardOf = c => ((c && c.problems) || []).filter(x => x.severity === 'problem')
const runPkg = async k => {
  let report = await build(k)
  await snap(`${k} fixed`)
  let chk = await check(k, report, 1)
  if (openOf(chk).length) {
    log(`${k}: ${openOf(chk).length} open check items, refix`)
    report = await refix(k, report, chk, 1)
    await snap(`${k} refix`)
    chk = await check(k, report, 2)
    if (hardOf(chk).length) {
      log(`${k}: ${hardOf(chk).length} problems after the refix, second refix`)
      report = await refix(k, report, chk, 2)
      await snap(`${k} second refix`)
      chk = await check(k, report, 3)
    }
  }
  const left = openOf(chk).length
  if (left) log(`${k}: ${left} items still open (${hardOf(chk).length} problems)`)
  return { key: k, report, chk }
}
phase('Fix')
const results = (await parallel(['N1', 'N2'].map(k => () => runPkg(k)))).filter(Boolean)
const leftovers = results.map(r => ({ key: r.key, open: openOf(r.chk) })).filter(r => r.open.length)
phase('CI')
const SOLO = CTX.replace('Two packages edit the tree at once: re-read a shared file right before each edit, change only your own lines, never revert what you did not write. ', 'You are the only editor now. ')
if (leftovers.length) {
  log(`Leftovers before CI: ${leftovers.map(l => `${l.key} ${l.open.length}`).join(', ')}`)
  await agent(`${SOLO}\n\nThe package checks of this pass left the items below open. Fix every "problem" and "minor" one under the rules of ${BR} (a test seen failing for behaviour; never weaken a test; keep what the checks confirmed). Where an item is wrong, say why with evidence. Scratch dir ${SP}/u9-left/. Report each item's state, files changed and the checks you ran.\n\n${JSON.stringify(leftovers, null, 1)}`, { label: 'leftovers', phase: 'CI', model: 'sonnet' })
}
const ciRun = n => agent(`You push a snapshot of /home/user/dla to the branch wip/progress and wait for its CI run. Do not edit any repository file and run no git command other than the script below.\n1. Run: bash ${SP}/u7/snapshot.sh "ci attempt ${n}". It prints either "snapshot <sha> (...)" or "no change since <sha>"; take the 40-character <sha>.\n2. Run: bash ${SP}/u8/ci-wait.sh <sha> ${SP}/u9/ci-${n} (use a Bash timeout of 590000 ms). It waits up to 8 minutes and prints "PENDING ..." if the run has not finished: run it again, up to 16 times in all (a run takes 40 to 90 minutes). When it prints "DONE run=<id> conclusion=<c>", it lists every job's conclusion and the log file of each failed job.\n3. For each failed job, read its log file (grep for "FAILED", "FAIL:", "panicked", "error", "is not", "failed" and read around the hits) and write a summary of at most 25 lines: the failing test or fixture names or the step, the assertion or error text, and anything the log shows about the cause.\nReturn: sha, green (true only when the run's conclusion is success), conclusion, failedJobs (job name, log path, summary), notes. If the run never finishes in 16 waits, return green false with conclusion "timeout".`, { label: `ci:attempt${n}`, phase: 'CI', schema: CI, model: 'sonnet' })
let ci = null
const ciHistory = []
for (let n = 1; n <= 4; n++) {
  ci = await ciRun(n)
  ciHistory.push(ci)
  if (!ci || ci.green) break
  log(`CI attempt ${n}: ${ci.conclusion}, failed: ${(ci.failedJobs || []).map(j => j.job).join('; ')}`)
  if (n === 4) break
  const fr = await agent(`${SOLO}\n\nThe CI run of the current tree (commit ${ci.sha} on wip/progress) failed. Its failed jobs, with the path of each job's full log and a summary, follow. For each failure: read the log yourself, find the cause (the runner's platform, toolchain or tools; a defect of this pass; a defect of the product; or a rare failure by chance, in which case find the mechanism that lets chance decide and fix that), reproduce it here if it can be reproduced on Linux (JDK 25 is at /usr/lib/jvm/java-25-openjdk-amd64), and fix it under the rules of ${BR}: never skip, ignore or weaken a test to get green; a product defect is fixed in the product; add the diagnostics a failed assertion needs when the cause cannot be seen from the log, so the next run explains itself. Run the local checks the brief lists. Update the docs/VERIFY.md rows of the items you touch. Scratch dir ${SP}/u9-ci-fix/. Report: each failure, its cause and evidence, the fix, tests and which you saw fail, what only the next CI run can confirm.\n\nEarlier attempts and their failures (do not undo their fixes):\n${JSON.stringify(ciHistory.slice(0, -1).map(c => ({ sha: c.sha, failed: (c.failedJobs || []).map(j => j.job + ': ' + j.summary.slice(0, 400)) })), null, 1)}\n\nThis run:\n${JSON.stringify(ci, null, 1)}`, { label: `ci-fix${n}`, phase: 'CI', model: 'sonnet' })
  const rv = await agent(`${SOLO}\n\nYou review the fix made after a failed CI run (commit ${ci.sha}). Do not edit repository files; scratch only under ${SP}/u9chk-ci/. Check against the rules of ${BR}: was any test skipped, ignored or weakened to get green; is each named cause supported by the CI log (read the log lines: paths below) and the code; is a product defect fixed in the product; does each new or changed test fail without the change; do \`cargo clippy --workspace --all-targets -j 2 -- -D warnings\`, the Windows-target clippy for dla-core, dla-probe and dla-report, \`cargo fmt --all --check\` and the tests of the touched crates pass. Keep the machine time of your own runs to about 60 minutes. Report problems (a weakened test, a wrong cause, a break), minors and notes with file:line.\n\nFailed jobs and logs:\n${JSON.stringify(ci.failedJobs, null, 1)}\n\nFixer report:\n${fr}`, { label: `ci-review${n}`, phase: 'CI', schema: CHECK })
  if (openOf(rv).length) {
    log(`CI fix ${n}: review found ${openOf(rv).length} items, refix`)
    await agent(`${SOLO}\n\nA review of the fix made after a failed CI run found the items below. Fix every "problem" and "minor" one under the rules of ${BR} (never weaken a test; keep what the review confirmed). Scratch dir ${SP}/u9-ci-fix/. Report each item's state and the checks you ran.\n\nReview:\n${JSON.stringify(rv, null, 1)}\n\nFixer report:\n${fr}`, { label: `ci-refix${n}`, phase: 'CI', model: 'sonnet' })
  }
}
phase('Accept')
const acc = await agent(`Repository /home/user/dla, branch main at 9781542 with uncommitted changes. Do NOT commit, push or edit repository files. You run the local acceptance of the working tree. Disk: if \`df --output=avail -B1M /\` is below 3000, run \`bash ${SP}/prune-superseded.sh\`. Cargo with -j 2, shared target/ only, never kill processes by name. Scratch only under ${SP}/u9acc/. Run and record each result: (1) cargo test -j 2 --no-fail-fast --workspace with the default JDK 21 (rerun a failed test alone twice: steady or flaky); (2) the JVM-dependent test binaries again with JDK 25 first on PATH and JAVA_HOME=/usr/lib/jvm/java-25-openjdk-amd64 (dla-sample; dla-cli: sample, attach, alloc_test_jvm, summarize_conformance, harness_capture and the other harness tests; dla-workload: every harness_* test); (3) cargo clippy --workspace --all-targets -j 2 -- -D warnings; cargo clippy --target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report --all-targets -j 2 -- -D warnings; (4) cargo fmt --all --check; (5) target/debug/dla schema --check; (6) python3 bench/runner/test_grade.py; cargo test -j 2 -p dla-cli --test skill_text; (7) ./target/debug/dla-bench static --dla-bin ./target/debug/dla (both with and without --archetypes off) and ./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json ${SP}/u9acc/dynamic.json: all must exit 0 with all 60 fixtures passing (one expected failure, rust-tree-parent-links, is listed as xfail). Compare the dynamic result fixture by fixture with ${SP}/u8acc/dynamic.json (the run before this pass; python-graph-churn failed there by chance): list every fixture whose pass/fail, confirmed, refuted or rule-out figures differ and say why if the logs show it. Rerun a failed fixture alone twice (\`--only\`) and keep the logs of the failing run: say for each failure whether a column that passed the first trend by chance voted in the unbound alloc-test (read its trend and alloc files under the log dir), and which column. (8) \`git status --short\` summary and a scan of \`git diff HEAD\` (added lines) for model names, attribution text, scratch paths and plan names (N1, N2, K1 to K4, U8, U9). Return a plain report: each step pass/fail with counts, differences and failures with evidence, and the disk at the end.`, { label: 'accept', phase: 'Accept' })
await snap('after the acceptance of the last fix pass')
return { packages: results.map(r => ({ key: r.key, open: openOf(r.chk), report: r.report })), ci: ciHistory, acceptance: acc }
