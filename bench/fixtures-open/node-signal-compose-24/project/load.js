'use strict';

// Replays export requests one after another.
// Usage: node load.js --iterations 20000

const { handle } = require('./src/handler');

function parseArgs(argv) {
  const args = { iterations: 20000 };
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, '');
    if (!(key in args)) throw new Error(`unknown option ${argv[i]}`);
    args[key] = Number.parseInt(argv[i + 1], 10);
  }
  return args;
}

async function runWorkload(iterations) {
  let served = 0;
  let rows = 0;
  for (let i = 0; i < iterations; i += 1) {
    const result = await handle({ account: 1000 + (i % 250), budgetMs: 30000 });
    if (result.status === 200) {
      served += 1;
      rows += result.totals.count;
    }
  }
  return { requests: iterations, served, rows };
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const started = Date.now();
  runWorkload(args.iterations).then((result) => {
    const rss = Math.round(process.memoryUsage().rss / 1024);
    console.log(`requests=${result.requests} served=${result.served} rows=${result.rows} rssKB=${rss} ms=${Date.now() - started}`);
  });
}

module.exports = { runWorkload };
