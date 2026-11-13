'use strict';

// Connects field gateways, delivers a few frames each and disconnects them.
// Usage: node load.js --iterations 20000

const { Relay } = require('./src/relay');
const { KIND } = require('./src/frames');

function parseArgs(argv) {
  const args = { iterations: 20000 };
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, '');
    if (!(key in args)) throw new Error(`unknown option ${argv[i]}`);
    args[key] = Number.parseInt(argv[i + 1], 10);
  }
  return args;
}

function runWorkload(iterations) {
  const written = [];
  const relay = new Relay({ write: (device, line) => written.push(line.length) });
  for (let i = 0; i < iterations; i += 1) {
    const device = 1000 + i;
    relay.connect(device);
    for (let k = 0; k < 3; k += 1) {
      relay.deliver(device, { kind: 1 + (k % 3), device, value: (i * 13 + k) % 4000 });
    }
    relay.disconnect(device);
  }
  return { sessions: iterations, frames: written.length, kinds: Object.keys(KIND).length };
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const result = runWorkload(args.iterations);
  const rss = Math.round(process.memoryUsage().rss / 1024);
  console.log(`sessions=${result.sessions} frames=${result.frames} rssKB=${rss}`);
}

module.exports = { runWorkload };
