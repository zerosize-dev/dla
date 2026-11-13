'use strict';

// Replays a queue of month-end report requests.
// Usage: node load.js --iterations 60 [--rows 40000]

const reports = require('./src/reports');

function parseArgs(argv) {
  const args = { iterations: 60, rows: 40000 };
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, '');
    if (!(key in args)) throw new Error(`unknown option ${argv[i]}`);
    args[key] = Number.parseInt(argv[i + 1], 10);
  }
  return args;
}

let counter = 0;

function runWorkload(iterations, rows = 40000) {
  let lines = 0;
  for (let i = 0; i < iterations; i += 1) {
    counter += 1;
    const text = reports.build(1000 + counter, 1 + (counter % 12), rows);
    lines += text.split('\n').length;
  }
  return { reports: iterations, lines, cached: reports.cachedCount() };
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const started = Date.now();
  const result = runWorkload(args.iterations, args.rows);
  const rss = Math.round(process.memoryUsage().rss / 1024);
  console.log(`reports=${result.reports} lines=${result.lines} cached=${result.cached} rssKB=${rss} ms=${Date.now() - started}`);
}

module.exports = { runWorkload };
