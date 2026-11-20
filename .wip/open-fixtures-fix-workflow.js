export const meta = {
  name: 'open-fixtures-fix-1',
  description: 'First fix pass for the parked fixtures: five product packages (label evidence, Python census and agent, Node probes and registration holders, Go static rules, Go in-run measurement), each fixed, independently checked and refixed; two chains in one tree',
  phases: [{ title: 'Design' }, { title: 'Fix' }, { title: 'Check' }],
}
const SP = '/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad'
const BR = `${SP}/u14-brief.md`
const PRIOR = (args && args.results) || {}
const NAMES = {
  'F-a': 'a registration leak can be labelled on its allocation stack',
  'F-b': 'Python census and agent',
  'F-c': 'Node probes and registration holders',
  'F-d': 'Go static rules',
  'F-e': 'Go command mode measures while the workload runs',
}
const FIX = {
  'F-a': ['go-ticker-legacy'], 'F-b': ['python-frozen-gc', 'python-zstd-options-3144'], 'F-c': ['node-signal-compose-24'],
  'F-d': ['go-feed-merge', 'go-header-index', 'go-trace-context'], 'F-e': ['go-header-index', 'go-trace-context', 'go-feed-merge'],
}
const CTX = `Repository /home/user/dla (Rust workspace and the DLA Claude Code plugin), branch main at 4aae86f; \`git diff HEAD\` shows only this pass. Do NOT commit or push. Read ${BR} in full first: the background, the rules (never edit fixtures, tests seen failing, two packages edit the tree at once and each has its own files, keep the tree compiling, cargo -j 2 and the shared target/ only, VERIFY rows, the checks and bench runs before reporting, the report file) and the section of your package.`
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
const run = (key, prompt, opts) => (key in PRIOR) ? Promise.resolve(PRIOR[key]) : agent(prompt, { ...opts, label: key })
const openOf = c => ((c && c.problems) || []).filter(x => x.severity !== 'note')
const hardOf = c => ((c && c.problems) || []).filter(x => x.severity === 'problem')
const fx = k => FIX[k].map(f => `${SP}/u13/results/${f}.json`).join(', ')
const snap = (k, what) => run(`${k}:snapshot-${what}`, `Run exactly this one command and return its output verbatim, nothing else. Do not run any other git command and do not edit any file:\n\nbash ${SP}/u7/snapshot.sh "open fixtures, ${k} ${what} [skip ci]"`, { phase: 'Fix', model: 'sonnet', effort: 'low' })
const fix = (k, extra) => run(`${k}:fix`, `${CTX}\n\nYou implement package ${k} (${NAMES[k]}). The verified diagnoses it rests on: ${fx(k)} (read the \`verdict\` of each in full, then the \`diagnosis\`). Scratch dir ${SP}/u14/${k}/ (create it). Confirm each cause on the code and the logs before you change anything; where the brief's decision and a verdict differ, the brief decides, and where the code makes a decision impossible or wrong, stop on that item and say why with evidence instead of doing something else. Then implement the items in order. Write the report to ${SP}/u14/${k}/report-fix.md and return it.${extra || ''}`, { phase: 'Fix', model: 'sonnet' })
const check = (k, n, prev) => run(`${k}:check${n}`, `${CTX}\n\nYou check package ${k} (${NAMES[k]}) independently. Do not edit repository files; create and delete files only under ${SP}/u14chk/${k}/ (never a second binary named dla in target/, never a second target dir). The other package's files are not your concern. The fixer's report is ${SP}/u14/${k}/report-${n === 1 ? 'fix' : `refix${n - 1}`}.md${n > 1 ? ` (the earlier report and check are beside it: report-fix.md${n > 2 ? ', report-refix1.md' : ''}; the previous check follows below)` : ''}; the diagnoses are ${fx(k)}. For every item of the package's section: is it done as the brief decides, in the right place; is it right beyond the fixture (try at least three adversarial inputs per new rule, table row or reader: shapes that must fire and shapes that must stay quiet, taken from how real projects are written, and say for each what the code does; replay them as unit-level cases in your scratch dir or by running the built dla on small scratch projects); does each new test fail without the change (by reading \`git diff HEAD -- <the package's files>\`, and by replay where cheap); was any test skipped, ignored or weakened; does anything depend on a name only a fixture has; are the docs and VERIFY rows true and in form? Run the unit tests of the changed modules and the test binaries the package touches, and the bench runs its section names for the parked fixtures (on scratch copies) plus the two fixtures of bench/fixtures most at risk; compare with ${SP}/u11acc/dynamic.json. Keep the machine time of your own runs to about 60 minutes: another agent works on the same 4 CPUs. For each claim of the report: confirmed, false or unverified. Every remaining defect is a problem (behaviour, a rule that fires wrongly or misses its own case, a weakened or vacuous test, a false VERIFY row or doc, a clippy break, a worse fixture), minor (wording, a small gap) or a note (no action). Give file:line.${prev ? `\n\nPrevious check:\n${JSON.stringify(prev, null, 1)}` : ''}`, { phase: 'Check', schema: CHECK })
const refix = (k, n, chk) => run(`${k}:refix${n}`, `${CTX}\n\nYou refix package ${k} (${NAMES[k]}). The reports so far are in ${SP}/u14/${k}/ (report-fix.md${n > 1 ? ', report-refix1.md' : ''}); an independent check follows. Fix every "problem" and "minor" item of it that lies in your package's files (a "note" needs nothing), each with a test you SEE fail without it where it is behaviour; keep every case the check confirmed working; re-run what the brief asks before reporting. Where a check item is wrong, say why with evidence instead of changing code. Scratch dir ${SP}/u14/${k}/. Write the full report (the state of every item of the package, not only the refixes) to ${SP}/u14/${k}/report-refix${n}.md and return it.\n\nCheck:\n${JSON.stringify(chk, null, 1)}`, { phase: 'Fix', model: 'sonnet' })
const pkg = async (k, extra) => {
  await fix(k, extra)
  await snap(k, 'fixed')
  let chk = await check(k, 1, null)
  if (openOf(chk).length) {
    log(`${k}: ${openOf(chk).length} open check items (${hardOf(chk).length} problems), refix`)
    await refix(k, 1, chk)
    await snap(k, 'refix')
    chk = await check(k, 2, chk)
    if (hardOf(chk).length) {
      log(`${k}: ${hardOf(chk).length} problems after the refix, second refix`)
      await refix(k, 2, chk)
      await snap(k, 'second-refix')
      chk = await check(k, 3, chk)
    }
  }
  if (openOf(chk).length) log(`${k}: ${openOf(chk).length} items open at the end (${hardOf(chk).length} problems)`)
  return { key: k, open: openOf(chk), confirmed: chk && chk.confirmed, unverified: chk && chk.unverified }
}
const designE = async () => {
  await run('F-e:design', `${CTX}\n\nYou write the design of package F-e (${NAMES['F-e']}); you do not implement it and you do not edit repository files. Scratch dir ${SP}/u14/F-e/. Read the three verdicts (${fx('F-e')}): each proposes a different shape (the last in-run sample as extra columns; a collection per tick behind a variable with \`.run\` columns; the peak sample replacing the base columns with \`.at_exit\` twins). Read the Go agent templates under harness/go/, how its records become series columns (crates/dla-workload/src/harness/dumps.rs and build.rs, crates/dla-core/src/series.rs), how the trend, the choice of the primary heap column and the alloc-test use them (crates/dla-cli/src/alloc/pick.rs and its callers), and how the heap profile is captured and read (crates/dla-workload/src/harness/capture.rs, crates/dla-cli/src/summarize/go_pprof_heap.rs). Decide by measurement, not by taste: prototype the sampler in a scratch copy of the generated wrapper (take the generated files of a kept harness run, for example by running \`dla harness gen\` and \`dla harness run\` by hand on scratch copies of go-header-index, go-trace-context, go-buffer-pool and go-gc-tuned-batch, and edit the copy in the run directory, never the repository) and measure for each candidate shape: does the in-run reading grow with the iteration count on the two leaking projects and stay flat on the two that do not leak; its scatter between repeated runs; its bias with the number of samples; the run-time cost of the collections; whether go-gc-tuned-batch's collector figures change. Write ${SP}/u14/F-e/design.md: the chosen shape and why the others lose (with the figures), the record fields, the column names and their catalog types, the sampling schedule, the capture, the limitation lines, what the trend, the primary-column choice and the alloc-test do with the new columns, the exact files and functions to change, the tests (each with what fails today), and the risks. Return a summary of the design and the measured figures.`, { phase: 'Design' })
  const rv = await run('F-e:design-review', `${CTX}\n\nYou review the design of package F-e (${NAMES['F-e']}) before it is built: ${SP}/u14/F-e/design.md (its measurements are in the same directory). Do not edit repository files; scratch only under ${SP}/u14chk/F-e-design/. Try to break it: does it meet every constraint of the brief's F-e section; are the measurements enough to carry its choices (repeat the two you trust least); can the reading mislead (a healthy program whose live heap depends on where in a cycle the sample falls; a program that exits in a few milliseconds; a program whose \`main\` never returns but calls os.Exit; goroutines still running at the sample; a very large heap where a collection per sample is slow); does anything existing change meaning (the six go-* fixtures of bench/fixtures, driver mode, the purge logic, the gc-twin test); is each listed test able to fail. Write your review to ${SP}/u14/F-e/design-review.md: what must change in the design (numbered, each with the reason and the replacement), what is fine, what stays unverified. Return it.`, { phase: 'Design' })
  return rv
}
phase('Fix')
const chains = await parallel([
  async () => { const out = []; for (const k of ['F-a', 'F-b', 'F-c']) out.push(await pkg(k)); return out },
  async () => {
    const out = [await pkg('F-d')]
    await designE()
    out.push(await pkg('F-e', ` The design is ${SP}/u14/F-e/design.md and its review is ${SP}/u14/F-e/design-review.md: build the design with every change the review demands (where you do not follow a review point, say why with evidence). Package F-d (Go static rules) is finished and in the tree: keep consistent with it and do not undo it.`))
    return out
  },
])
return chains.filter(Boolean).flat().filter(Boolean)
