export const meta = {
  name: 'fix-pass-u7',
  description: 'Fix the 14 open items in three packages (budget/time, coverage steps, JVM non-heap), each checked and refixed up to twice, then an integration check over the whole round and the full acceptance run; snapshots to wip/progress after every fix',
  phases: [{ title: 'Fix' }, { title: 'Check' }, { title: 'Integrate' }, { title: 'Accept' }],
}
const SP = '/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad'
const BR = `${SP}/u7-brief.md`
const ITEMS = `${SP}/u6/open-items-final.md`
const NAMES = { M1: 'budget and time', M2: 'coverage warnings and harness steps', M3: 'JVM non-heap verdicts' }
const CTX = `Repository /home/user/dla (Rust workspace and the DLA Claude Code plugin), branch main. HEAD (d1fca56) holds the round as work in progress; its base is c65e617, so \`git diff c65e617\` with the working tree is the whole round and \`git diff HEAD\` is only this pass. Do NOT commit or push. Read ${BR} in full first: its rules (disk, -j 2, tests seen failing, VERIFY rows, shared files, checks before reporting) and your package's section with its decisions. The open items are in ${ITEMS}. Up to two packages edit the tree at once: re-read a shared file right before each edit, change only your own lines, never revert what you did not write. If a test binary vanishes or is busy, another agent rebuilt target/: rerun it. Never kill processes by name.`
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
const snap = label => agent(`Run exactly this one command and return its output verbatim, nothing else. Do not run any other git command and do not edit any file:\n\nbash ${SP}/u7/snapshot.sh "${label}"`, { label: `snapshot:${label}`, phase: 'Fix', model: 'sonnet', effort: 'low' })
const build = k => agent(`${CTX}\n\nYou fix package ${k} (${NAMES[k]}) of ${BR}. Scratch dir ${SP}/u7-${k}/ (create it; delete only it, by literal path). For each of its items: check it against the current tree, then fix it following the decisions, with a test you SEE fail without the change where it is behaviour. End with the report the brief asks for.`, { label: `fix:${k}`, phase: 'Fix', model: 'sonnet' })
const check = (k, report, round) => agent(`${CTX}\n\nYou check package ${k} (${NAMES[k]}) of ${BR}. Do not edit repository files; create and delete files only under ${SP}/u7chk-${k}/. For every item of the package's section in ${ITEMS} and every decision in the brief: is it done, does it do what the decision says, does its test fail without the change (judge by reading \`git diff HEAD\` and, where cheap, by running on a scratch copy)? Try at least two adversarial cases per behaviour change, including whether a case that worked before now breaks, with the real binary where you can (target/debug/dla after building). Run the tests and bench fixtures the brief names for the package; run timing- or JVM-dependent tests several times, some under load. For each claim of the report: confirmed, false or unverified. Every remaining defect is a problem (behaviour, a test that does not test, a false VERIFY row or doc, a Windows clippy break, a worse bench fixture), minor (wording, a small gap) or a note (no action). Give file:line.\n\nFixer report${round > 1 ? ' (after a refix)' : ''}:\n${report}`, { label: `check${round}:${k}`, phase: 'Check', schema: CHECK })
const refix = (k, report, chk) => agent(`${CTX}\n\nYou refix package ${k} (${NAMES[k]}) of ${BR}. Its fixer's report and an independent check follow. Fix every "problem" and "minor" item of the check inside your package's scope (a "note" needs nothing), each with a test you SEE fail without it where it is behaviour; re-run what the brief asks before reporting. Where a check item is wrong, say why with evidence instead of changing code. Do not trade one defect for another: before changing a rule, re-read the brief's decisions and keep every case the check confirmed working. Scratch dir ${SP}/u7-${k}/. End with the full report (the state of every item, not only the refixes).\n\nFixer report:\n${report}\n\nCheck:\n${JSON.stringify(chk, null, 1)}`, { label: `refix:${k}`, phase: 'Fix', model: 'sonnet' })
const openOf = c => ((c && c.problems) || []).filter(x => x.severity !== 'note')
const runPkg = async k => {
  let report = await build(k)
  await snap(`${k} fixed`)
  let chk = await check(k, report, 1)
  for (let r = 2; r <= 3; r++) {
    const open = openOf(chk)
    if (!open.length) break
    log(`${k}: ${open.length} open check items, refix ${r - 1}`)
    report = await refix(k, report, chk)
    await snap(`${k} refix ${r - 1}`)
    chk = await check(k, report, r)
  }
  const left = openOf(chk).length
  if (left) log(`${k}: ${left} items still open after two refixes`)
  return { key: k, report, chk }
}
phase('Fix')
const results = (await parallel(['M1', 'M3', 'M2'].map(k => () => runPkg(k)))).filter(Boolean)
const reports = results.map(r => `## ${r.key} (${NAMES[r.key]})\n${r.report}\n\nLast check of ${r.key}:\n${JSON.stringify(r.chk, null, 1)}`).join('\n\n')
phase('Integrate')
const ICTX = CTX.replace('Up to two packages edit the tree at once: re-read a shared file right before each edit, change only your own lines, never revert what you did not write. ', 'All packages have finished. ')
const integ = (round, extra) => agent(`${ICTX}\n\nYou are the integration check of the whole round (\`git diff c65e617\` with the working tree). Do not edit repository files; scratch only under ${SP}/u7integ/. Check: (1) open items left by the three packages' last checks (below) and whether this pass undid anything of the round in shared files (crates/dla-core/src/gate.rs, toolctx.rs, state.rs, crates/dla-cli/src/gate_hooks.rs, run.rs, alloc_cmd.rs, crates/dla-report/src/lib.rs, the goldens, SKILL.md, references/*.md, harness/jvm/README.md, docs/VERIFY.md, schemas/); (2) the texts agree: one instruction between the floor (max(10 min, 20%)) and the deadline across SKILL.md, stages.md, recovery.md, the Stop hook, the gate and tool fixes; the harness step (JUnit, command, driver, no harness, blocker, build failure) is the same in the gate fix, the Stop hook, the report's Coverage warning and the docs; the non-heap route text with the alloc-test verdicts and jvm.md; (3) run cargo test -j 2 --no-fail-fast for the whole workspace (all test binaries; rerun a vanished binary), cargo clippy --workspace --all-targets -j 2 -- -D warnings, cargo clippy --target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report --all-targets -j 2 -- -D warnings, cargo fmt --all --check, target/debug/dla schema --check, python3 bench/runner/test_grade.py, and target/debug/dla-bench dynamic --parallel 1 --only <f> for jvm-order-cache, python-report-memo, jni-order-codec, go-subscription-workers and node-token-cache (build dla and dla-bench first; compare with ${SP}/p2r3-acc/on/result.json); (4) docs/VERIFY.md rows of the round (the \`## M7\` table from the row "Measured line" region on, roughly rows 470 to the end): 4 cells each, no nested backticks, every named test exists, "corrected" only where tests ran, no plan or package names (P4..P9, L1..L3, M1..M3, C1b, A5, round, refix, integration check, scratch names), no stale byte counts; no model names or attribution text anywhere in \`git diff c65e617\`; SKILL.md within the skill_text limit; (5) two end-to-end scenarios with the real binary that the round exists for: an interactive run with 55 of 60 minutes left and open findings (the Stop hook must say to keep measuring, and \`dla ledger resolve --reason time_budget\` must be refused), and a JVM run whose JUnit harness exercised 0 entry points (the gate fix, Stop hook and report must give the same possible next step). Give file:line for every problem.${extra || ''}\n\nPackage reports and their last checks:\n${reports}`, { label: `integrate${round}`, phase: 'Integrate', schema: CHECK })
let ic = await integ(1)
for (let r = 2; r <= 3; r++) {
  const open = openOf(ic)
  if (!open.length) break
  log(`integration: ${open.length} open items, fix ${r - 1}`)
  const fr = await agent(`${ICTX.replace('All packages have finished. ', 'You are the only editor now. ')}\n\nFix every "problem" and "minor" item of the integration check below, following the rules and decisions of ${BR} (tests seen failing, VERIFY rows, disk). Where an item is wrong, say why with evidence. Do not trade one defect for another: keep every case the check confirmed working. Scratch dir ${SP}/u7integ-fix/. End with a report: each item's state, files changed, tests and which you saw fail, checks run.\n\nIntegration check:\n${JSON.stringify(ic, null, 1)}`, { label: `integrate-fix${r - 1}`, phase: 'Integrate', model: 'sonnet' })
  await snap(`integration fix ${r - 1}`)
  ic = await integ(r, `\n\nA fixer addressed the previous integration check; its report:\n${fr}`)
}
phase('Accept')
const acc = await agent(`Repository /home/user/dla, branch main. Do NOT commit, push or edit repository files. You run the full acceptance of the working tree. Disk: if \`df --output=avail -B1M /\` is below 3000, run \`bash ${SP}/prune-superseded.sh\`; if still below 1500, wait up to 30 minutes, then stop and report. Cargo with -j 2, shared target/ only, never kill processes by name. Scratch only under ${SP}/u7acc/ (output files and logs there). Run, in order, and record each command's result: (1) cargo build -j 2 -p dla-cli -p dla-bench; (2) cargo test -j 2 --no-fail-fast --workspace, every test binary (prune with the script between crates if disk is low; if a binary vanishes, rerun it; rerun a failed test alone twice and report whether it is steady or flaky); (3) cargo clippy --workspace --all-targets -j 2 -- -D warnings and cargo clippy --target x86_64-pc-windows-gnu -p dla-core -p dla-probe -p dla-report --all-targets -j 2 -- -D warnings; (4) cargo fmt --all --check; (5) target/debug/dla schema --check; (6) python3 bench/runner/test_grade.py; (7) cargo test -j 2 -p dla-cli --test skill_text; (8) ./target/debug/dla-bench static --dla-bin ./target/debug/dla --json ${SP}/u7acc/static-on.json and the same with --archetypes off to static-off.json; (9) ./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 2 --json ${SP}/u7acc/dynamic.json (all fixtures in bench/fixtures). Compare the dynamic result with ${SP}/p2r3-acc/on/result.json (its \`fixtures\` list and \`failures\` list, one "<fixture>: <text>" per failure): for every fixture give pass/fail now and in the baseline; any fixture that passed in the baseline and fails now is a regression, with its failure text and log dir; rerun each regression once alone (--only <fixture> --parallel 1) and say whether it repeats. Return a plain report: each step pass/fail with counts, regressions with evidence, fixtures that fail in both, fixtures that now pass, and the disk at the end.`, { label: 'accept', phase: 'Accept' })
await snap('after acceptance')
return { packages: results.map(r => ({ key: r.key, open: openOf(r.chk), report: r.report })), integration: ic, acceptance: acc }
