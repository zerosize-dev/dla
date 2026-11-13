'use strict';

// Replays tracking lookups.
// Usage: node load.js --iterations 20000

const gateway = require('./src/gateway');
const audit = require('./src/audit');
const trail = require('./src/trail');

const PATHS = ['/parcels/track', '/parcels/eta', '/parcels/label', '/parcels/track', '/parcels/missing'];

function parseArgs(argv) {
  const args = { iterations: 20000 };
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, '');
    if (!(key in args)) throw new Error(`unknown option ${argv[i]}`);
    args[key] = Number.parseInt(argv[i + 1], 10);
  }
  return args;
}

let counter = 0;

function runWorkload(iterations) {
  let ok = 0;
  let bytes = 0;
  for (let i = 0; i < iterations; i += 1) {
    counter += 1;
    const request = {
      path: PATHS[counter % PATHS.length],
      parcel: 100000 + ((counter * 7919) % 50000),
      client: `shop-${counter % 40}`,
    };
    const response = gateway.handle(request);
    if (response.status === 200) {
      ok += 1;
      bytes += response.body.length;
    }
  }
  return { requests: iterations, ok, bytes, last: trail.latest().explain(), notes: audit.recent().length };
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const started = Date.now();
  const result = runWorkload(args.iterations);
  const rss = Math.round(process.memoryUsage().rss / 1024);
  console.log(`requests=${result.requests} ok=${result.ok} bytes=${result.bytes} last="${result.last}" rssKB=${rss} ms=${Date.now() - started}`);
}

module.exports = { runWorkload };
