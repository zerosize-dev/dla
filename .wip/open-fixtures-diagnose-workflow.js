export const meta = {
  name: 'open-fixtures-diagnose',
  description: 'Diagnose the eight parked fixtures: per fixture one diagnosis (why static does not flag it, where the dynamic chain breaks, a product change that is right beyond the fixture) and one independent verification of it',
  phases: [{ title: 'Diagnose' }, { title: 'Verify' }],
}
const SP = '/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad'
const BR = `${SP}/u13-brief.md`
const FIXTURES = ['go-ticker-legacy', 'python-frozen-gc', 'go-feed-merge', 'node-signal-compose-24', 'rust-slot-arena', 'go-header-index', 'python-zstd-options-3144', 'go-trace-context']
const DIAG = {
  type: 'object',
  properties: {
    fixture: { type: 'string' },
    leak: { type: 'string' },
    static: { type: 'object', properties: { passes: { type: 'boolean' }, candidates: { type: 'string' }, cause: { type: 'string' }, proposal: { type: 'string' }, where: { type: 'string' } }, required: ['passes', 'cause', 'proposal', 'where'] },
    dynamic: { type: 'object', properties: { passes: { type: 'boolean' }, broken_step: { type: 'string' }, cause: { type: 'string' }, proposal: { type: 'string' }, where: { type: 'string' } }, required: ['passes', 'broken_step', 'cause', 'proposal', 'where'] },
    generality: { type: 'string' }, risk: { type: 'string' }, tests: { type: 'string' },
    effort_static: { type: 'string', enum: ['none', 'small', 'medium', 'large'] },
    effort_dynamic: { type: 'string', enum: ['none', 'small', 'medium', 'large'] },
    key_doubts: { type: 'string' }, unverified: { type: 'string' },
  },
  required: ['fixture', 'leak', 'static', 'dynamic', 'generality', 'risk', 'tests', 'effort_static', 'effort_dynamic', 'key_doubts', 'unverified'],
}
const VERDICT = {
  type: 'object',
  properties: {
    fixture: { type: 'string' },
    static_cause_right: { type: 'string', enum: ['yes', 'partly', 'no', 'not-applicable'] },
    dynamic_cause_right: { type: 'string', enum: ['yes', 'partly', 'no', 'not-applicable'] },
    corrections: { type: 'string' },
    proposal_generic: { type: 'string', enum: ['yes', 'partly', 'no'] },
    better_proposal: { type: 'string' },
    fixtures_at_risk: { type: 'string' },
    key_doubts: { type: 'string' },
    recommended_change: { type: 'string' },
    unverified: { type: 'string' },
  },
  required: ['fixture', 'static_cause_right', 'dynamic_cause_right', 'corrections', 'proposal_generic', 'better_proposal', 'fixtures_at_risk', 'key_doubts', 'recommended_change', 'unverified'],
}
const results = await pipeline(
  FIXTURES,
  f => agent(`Read ${BR} in full and follow it. Your fixture is \`${f}\` (bench/fixtures-open/${f}). Do every numbered step, then return the fields it asks for. Write a copy of your full findings (with the quoted log lines and code references) to ${SP}/u13/${f}/diagnosis.md as well.`, { label: `diagnose:${f}`, phase: 'Diagnose', schema: DIAG, model: 'sonnet' }),
  (d, f) => d && agent(`Read ${BR} in full for the background and the rules (no repository edits; your scratch dir is ${SP}/u13v/${f}/). Another engineer diagnosed why DLA does not pass the fixture \`${f}\` (bench/fixtures-open/${f}); the diagnosis follows, its long form is ${SP}/u13/${f}/diagnosis.md, and its logs are under ${SP}/u13/${f}/. Verify it independently and try to refute it: (1) read the fixture and say whether the leak and its sites in the answer key are real and observable; (2) for the static cause and the dynamic cause each, check the quoted log lines and code yourself: is that really the first place the chain breaks, or is there an earlier or a different cause (rerun a single dla command on the kept work directory or the scratch fixtures directory where that settles it; do not rerun the whole fixture unless needed); (3) the proposals: would each be right on a real project of that kind, is it keyed to something that only this fixture has, is there a simpler or more general change in the same place, and does existing code already do something similar for another runtime that should be reused; (4) which of the 60 fixtures in bench/fixtures are at risk from the change (name them: the non-leak, decoy-control and near-miss fixtures of that language first) and what should be run to know. End with the change you would recommend, as concretely as the code allows (files, the rule or evidence, the tests), and what is still unverified. Keep machine time to about 20 minutes.\n\nDiagnosis:\n${JSON.stringify(d, null, 1)}`, { label: `verify:${f}`, phase: 'Verify', schema: VERDICT }).then(v => ({ fixture: f, diagnosis: d, verdict: v })),
)
return results.filter(Boolean)
