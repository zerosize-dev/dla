'use strict';

// DLA loop driver. One process, one checkpoint record per round (x = units completed).
//
// Fill the two DLA-HOOK bodies below and nothing else; keep the loop as it is.
//
// Keys: DLA_KEY_MODE=fresh uses key = i (every unit sees a new key); DLA_KEY_MODE=repeated uses
// key = i % repeat (a bounded key space). Rounds: DLA_WARMUP warm-up rounds, then DLA_ROUNDS
// measured rounds of DLA_STEP units each (defaults are baked in at generation time).
//
// Run from the scratch project root with the agent preloaded:
//
//     NODE_OPTIONS="--require <harness dir>/dla-preload.cjs" node <harness dir>/dla-harness.cjs
//
// This file is CommonJS whatever the project's package type is (the .cjs extension). Hooks run
// inside async functions: load a CommonJS file with projectRequire(file), and an ES module (a .mjs
// file, or any file in a package whose package.json says "type": "module") with
// `await projectImport(file)`.

const path = require('path');

const dla = require(process.env.DLA_AGENT || path.join(__dirname, 'dla-preload.cjs'));

const STEP = Number.parseInt(process.env.DLA_STEP || '@@DLA_STEP@@', 10);
const WARMUP = Number.parseInt(process.env.DLA_WARMUP || '@@DLA_WARMUP@@', 10);
const ROUNDS = Number.parseInt(process.env.DLA_ROUNDS || '@@DLA_ROUNDS@@', 10);
const REPEAT = Number.parseInt(process.env.DLA_REPEAT || '@@DLA_REPEAT@@', 10);
const REPEATED = process.env.DLA_KEY_MODE === 'repeated';

// The scratch project root; load project files with projectRequire('src/service.js') so that the
// modules named in probes.json are loaded through the agent.
const ROOT = path.resolve(process.env.DLA_PROJECT || process.cwd());
const projectRequire = (file) => require(path.resolve(ROOT, file));
// The same for ES modules: dynamic import() of the file URL (returns a promise of the namespace).
const projectImport = (file) => import(require('url').pathToFileURL(path.resolve(ROOT, file)).href);

const state = {};

async function setup() {
  // DLA-HOOK setup: load the project modules (projectRequire for CommonJS, await projectImport for ES modules) and construct the services/caches/pools the workload needs; keep them in `state` (or module variables) for unit()
  throw new Error('DLA-HOOK setup has not been filled in');
}

async function unit(i, key) {
  // DLA-HOOK unit: perform exactly one unit of the workload for this key (derive request ids/names/payloads from key so a repeated key reproduces an earlier request)
  throw new Error('DLA-HOOK unit has not been filled in');
}

// Time limit of this process (seconds; `dla harness run` sets DLA_TIME_LIMIT_SECS from the
// harness timeout, 0 or unset = no fitting). The loop fits its rounds into it: once MIN_STEP units
// have run (inside the first round, so one slow round cannot outlast the limit before the first
// look) and after every round it knows the time per unit, and when the units left would not fit
// into the limit (with a margin of 15 percent and 5 seconds for the final checkpoint and the
// exit), it lowers the step of the round in progress and of the rounds left, never below
// MIN_STEP. 10 units: with fewer, the per-round costs (the forced collection and the checkpoint)
// and the heap noise swamp a leak of one entry per unit. The x axis stays the cumulative unit
// count.
const LIMIT_SECS = Number.parseFloat(process.env.DLA_TIME_LIMIT_SECS || '0');
const MIN_STEP = 10;

async function main() {
  await setup();
  const total = WARMUP + ROUNDS;
  const loopStart = process.hrtime.bigint();
  let step = STEP;
  let done = 0;
  let warned = false;
  // `inRound` units of round `round` have run; `left` rounds follow it.
  const fit = (round, inRound) => {
    if (!(LIMIT_SECS > 0) || done === 0) return;
    const left = total - round;
    const spent = Number(process.hrtime.bigint() - loopStart) / 1e9;
    const perUnit = spent / done;
    const budget = LIMIT_SECS * 0.85 - 5 - process.uptime();
    const need = perUnit * (step - inRound + step * left);
    if (need <= budget) return;
    const units = Math.floor(budget / perUnit); // units that still fit, the round in progress included
    const lowered = Math.max(Math.min(MIN_STEP, step), Math.min(step, Math.floor((units + inRound) / (left + 1))));
    if (lowered < step) {
      dla.note(`the driver lowered its step from ${step} to ${lowered} units per round in round ${round} (after ${inRound} units of it): one unit takes ${(perUnit * 1000).toFixed(1)} ms and the ${LIMIT_SECS} s time limit would cut the process (the x axis is still the cumulative unit count)`);
      step = lowered;
    }
    if (!warned && perUnit * (step - inRound + step * left) > budget) {
      warned = true;
      dla.note(`the unit is too slow for a series: even ${step} units per round (the floor) needs ${(perUnit * (step - inRound + step * left)).toFixed(0)} s for the units left and ${Math.max(0, budget).toFixed(0)} s remain; make the unit cheaper (one request to a local server, no sleeps, smaller payloads) or raise timeout_secs in harness.json`);
    }
  };
  for (let round = 1; round <= total; round += 1) {
    for (let n = 0; n < step; n += 1) {
      done += 1;
      const result = unit(done, REPEATED ? done % REPEAT : done);
      if (result && typeof result.then === 'function') await result;
      if (n + 1 === Math.min(MIN_STEP, STEP)) fit(round, n + 1);
    }
    // let timers and I/O callbacks run between rounds, as they would in a long-lived process
    await new Promise((resolve) => setImmediate(resolve));
    dla.checkpoint(done, round === total);
    if (round < total) fit(round + 1, 0);
  }
}

// The process ends when the loop is done and the last checkpoint is written: open servers,
// keep-alive sockets and timers of the units must not hold it until the kill. process.exit()
// still writes the coverage data (NODE_V8_COVERAGE) and runs the agent's exit handler.
function finish(code) {
  const exit = () => process.exit(code);
  try {
    process.stdout.write('', () => process.stderr.write('', exit));
    setTimeout(exit, 2000).unref();
  } catch (_) {
    exit();
  }
}

main().then(
  () => finish(0),
  (err) => {
    console.error(err && err.stack ? err.stack : err);
    finish(1);
  },
);
