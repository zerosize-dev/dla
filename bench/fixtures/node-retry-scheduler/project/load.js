'use strict';

// Submits webhook jobs in waves and advances the manual clock until the
// schedule is empty.
// Usage: node load.js --iterations 2000 [--wave 50] [--step 100]

const { ManualClock } = require('./src/clock');
const { Endpoint } = require('./src/endpoint');
const { sharedScheduler } = require('./src/scheduler');
const { makeJob } = require('./src/jobs');

function parseArgs(argv) {
  const args = { iterations: 2000, wave: 50, step: 100 };
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, '');
    if (!(key in args)) throw new Error(`unknown option ${argv[i]}`);
    args[key] = Number.parseInt(argv[i + 1], 10);
  }
  return args;
}

function makeRandom(seed) {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    return state / 4294967296;
  };
}

function runWorkload(iterations, options = {}) {
  const wave = options.wave || 50;
  const step = options.step || 100;
  const scheduler = sharedScheduler({ clock: new ManualClock(), endpoint: new Endpoint() });
  const clock = scheduler.clock;
  const random = makeRandom(11);
  let submitted = 0;
  while (submitted < iterations || scheduler.pending.length > 0) {
    for (let i = 0; i < wave && submitted < iterations; i += 1) {
      scheduler.submit(makeJob(submitted, random));
      submitted += 1;
    }
    scheduler.tick();
    clock.advance(step);
  }
  return { scheduler, clock };
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const started = process.hrtime.bigint();
  const { scheduler, clock } = runWorkload(args.iterations, args);
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6;
  const stats = scheduler.stats();
  console.log(`jobs=${args.iterations} delivered=${stats.delivered} dead=${stats.dead} retried=${stats.retried}`);
  console.log(`pending=${stats.pending} virtual_ms=${clock.now()} elapsed_ms=${elapsedMs.toFixed(0)} rss_mb=${(process.memoryUsage().rss / 1048576).toFixed(1)}`);
}

module.exports = { runWorkload, parseArgs };
