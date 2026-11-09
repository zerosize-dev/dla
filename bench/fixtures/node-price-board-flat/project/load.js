'use strict';

// Feeds market updates through the board.
// Usage: node load.js --iterations 2000 [--ticks 120]

const { Feed } = require('./src/feed');
const board = require('./src/board');
const { row } = require('./src/format');

function parseArgs(argv) {
  const args = { iterations: 2000, ticks: 120 };
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, '');
    if (!(key in args)) throw new Error(`unknown option ${argv[i]}`);
    args[key] = Number.parseInt(argv[i + 1], 10);
  }
  return args;
}

function runWorkload(iterations, ticks = 120) {
  const feed = new Feed(5, ticks);
  let bytes = 0;
  for (let i = 0; i < iterations; i += 1) {
    const update = feed.next();
    const snapshot = board.publish(update.symbol, update.ticks);
    bytes += row(snapshot).length;
  }
  return { updates: iterations, bytes, last: board.current().describe() };
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const started = Date.now();
  const result = runWorkload(args.iterations, args.ticks);
  const rss = Math.round(process.memoryUsage().rss / 1024);
  console.log(`updates=${result.updates} bytes=${result.bytes} last=${result.last} rssKB=${rss} ms=${Date.now() - started}`);
}

module.exports = { runWorkload };
