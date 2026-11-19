export const meta = {
  name: 'fix-pass-u10',
  description: 'Fix the native summary reading the wrong process record (jni-order-codec chance failure): fix, independent check, refix, second check; one agent at a time beside the other pass',
  phases: [{ title: 'Fix' }, { title: 'Check' }],
}
const SP = '/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad'
const BR = `${SP}/u10-brief.md`
const CTX = `Repository /home/user/dla (Rust workspace and the DLA Claude Code plugin), branch main, HEAD 9781542. Do NOT commit or push. Read ${BR} in full first and follow it: the cause, the decision, the rules (another pass edits the same tree now and its files are not yours; keep the tree compiling; cargo -j 2 and the shared target/ only; tests seen failing; the VERIFY row; the checks before reporting). Never kill processes by name.`
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
const open = c => ((c && c.problems) || []).filter(x => x.severity !== 'note')
phase('Fix')
let report = await agent(`${CTX}\n\nYou make the fix. Scratch dir ${SP}/u10/ (create it). First verify the cause on the real files the brief names (print the per-record totals yourself; if those files are gone, use another /tmp/dla-bench-dynamic-*/jni-order-codec run directory or say so). Then carry out decisions 1 to 4 in that order, each with the tests the brief asks for, seen failing without the change. End with the report the brief asks for.`, { label: 'fix', phase: 'Fix', model: 'sonnet' })
phase('Check')
const check = (round, rep) => agent(`${CTX}\n\nYou check the fix independently. Do not edit repository files; create and delete files only under ${SP}/u10chk/ (never build a second binary named dla; never a second target dir). Keep machine time to about 60 minutes: another pass is working on the same machine. Judge only the files of this pass (the brief lists them; \`git diff HEAD -- <those files>\`). For each decision of the brief: is it done, does it do what the decision says, does its test fail without the change (by reading against HEAD, and where cheap by reverting the hunk in the working tree is NOT allowed: use reading, or a unit-level replay in your scratch dir)? Try adversarial cases on the selection rule: a file with one process only; two processes where neither argv carries the count; the measured process without a shim record; a native harness whose dump holds a build tool and the program; a driver-mode run; an older run directory whose shim-dump.jsonl holds several processes; records with no pid or no argv. Does any case that worked before now read the wrong record or lose its evidence? Run \`cargo test -j 2 --no-fail-fast -p dla-cli --test summarize_native --test summarize_robust\`, the unit tests of the changed modules, \`target/debug/dla schema --check\`, and \`./target/debug/dla-bench dynamic --dla-bin ./target/debug/dla --parallel 1 --only jni-order-codec\` once (build with \`cargo build -j 2 -p dla-cli -p dla-bench\` first; compare with ${SP}/u8acc/dynamic.json). Replay the real case from the brief (the doctored copy of the 17 files) with the new binary. Check the new failure text of dla-bench dynamic by reading and, if cheap, by a unit test run. Check the VERIFY row against the brief's rules and against what was really run. For each claim of the report: confirmed, false or unverified. Every remaining defect is a problem (behaviour, a test that does not test, a false VERIFY row or doc, a worse fixture), minor (wording, a small gap) or a note (no action). Give file:line.\n\nFixer report${round > 1 ? ' (after a refix)' : ''}:\n${rep}`, { label: `check${round}`, phase: 'Check', schema: CHECK })
let chk = await check(1, report)
for (let r = 2; r <= 3; r++) {
  const o = open(chk)
  if (!o.length) break
  if (r === 3 && !o.some(x => x.severity === 'problem')) break
  log(`${o.length} open check items, refix ${r - 1}`)
  report = await agent(`${CTX}\n\nYou refix. The fixer's report and an independent check follow. Fix every "problem" and "minor" item of the check that lies in this pass's files (a "note" needs nothing), each with a test you SEE fail without it where it is behaviour; re-run what the brief asks before reporting. Where a check item is wrong, say why with evidence instead of changing code. Scratch dir ${SP}/u10/. End with the full report (the state of every decision, not only the refixes).\n\nFixer report:\n${report}\n\nCheck:\n${JSON.stringify(chk, null, 1)}`, { label: `refix${r - 1}`, phase: 'Fix', model: 'sonnet' })
  chk = await check(r, report)
}
return { report, lastCheck: chk, open: open(chk) }
