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

async function main() {
  await setup();
  const total = WARMUP + ROUNDS;
  let done = 0;
  for (let round = 1; round <= total; round += 1) {
    for (let n = 0; n < STEP; n += 1) {
      done += 1;
      const result = unit(done, REPEATED ? done % REPEAT : done);
      if (result && typeof result.then === 'function') await result;
    }
    // let timers and I/O callbacks run between rounds, as they would in a long-lived process
    await new Promise((resolve) => setImmediate(resolve));
    dla.checkpoint(done, round === total);
  }
}

main().catch((err) => {
  console.error(err && err.stack ? err.stack : err);
  process.exitCode = 1;
});
