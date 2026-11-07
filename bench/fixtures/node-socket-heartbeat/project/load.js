'use strict';

// Replays client sessions against an in-process gateway.
// Usage: node load.js --iterations 2000 [--frames 12] [--window 32]

const { Gateway } = require('./src/gateway');
const { LoopbackSocket } = require('./src/loopback');

function parseArgs(argv) {
  const args = { iterations: 2000, frames: 12, window: 32 };
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

function script(random, index, frames) {
  const topic = `room-${Math.floor(random() * 40)}`;
  const steps = [
    { type: 'hello', user: `device-${index}` },
    { type: 'subscribe', topic },
  ];
  for (let n = 0; n < frames; n += 1) {
    steps.push({
      type: 'publish',
      topic,
      body: { seq: n, reading: Math.round(random() * 10000) / 100, unit: 'kPa', tag: `probe-${index}-${n}` },
    });
  }
  return steps;
}

function runWorkload(iterations, options = {}) {
  const frames = options.frames || 12;
  const window = options.window || 32;
  const gateway = new Gateway({ heartbeatMs: options.heartbeatMs });
  const random = makeRandom(7);
  const open = [];
  for (let i = 0; i < iterations; i += 1) {
    const socket = new LoopbackSocket();
    gateway.accept(socket);
    for (const step of script(random, i, frames)) socket.clientSend(step);
    open.push(socket);
    if (open.length > window) open.shift().clientHangUp();
  }
  for (const socket of open) socket.clientHangUp();
  return gateway;
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const started = process.hrtime.bigint();
  const gateway = runWorkload(args.iterations, args);
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6;
  const snap = gateway.snapshot();
  console.log(`connections=${args.iterations} frames_in=${snap.counters.frames_in} frames_out=${snap.counters.frames_out}`);
  console.log(`open_sessions=${snap.sessions} elapsed_ms=${elapsedMs.toFixed(0)} rss_mb=${(process.memoryUsage().rss / 1048576).toFixed(1)}`);
}

module.exports = { runWorkload, parseArgs };
