'use strict';

// Renders a stream of notices.
// Usage: node load.js --iterations 20000

const { render } = require('./src/renderer');
const { LOCALES } = require('./src/locales');
const { names } = require('./src/templates');
const cache = require('./src/cache');

function parseArgs(argv) {
  const args = { iterations: 20000 };
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, '');
    if (!(key in args)) throw new Error(`unknown option ${argv[i]}`);
    args[key] = Number.parseInt(argv[i + 1], 10);
  }
  return args;
}

const KINDS = names();
let counter = 0;

function runWorkload(iterations) {
  let chars = 0;
  for (let i = 0; i < iterations; i += 1) {
    counter += 1;
    const mix = (counter * 2654435761) >>> 0;
    const text = render({
      kind: KINDS[mix % KINDS.length],
      locale: LOCALES[(mix >>> 8) % LOCALES.length],
      customer: `Customer ${counter % 5000}`,
      account: `AC-${100000 + (counter % 9000)}`,
      cents: (counter * 37) % 900000,
      date: `2025-03-${String(1 + (counter % 28)).padStart(2, '0')}`,
      effective: `2025-0${1 + (counter % 9)}-01`,
    });
    chars += text.length;
  }
  return { rendered: iterations, chars, templates: cache.size() };
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const started = Date.now();
  const result = runWorkload(args.iterations);
  const rss = Math.round(process.memoryUsage().rss / 1024);
  console.log(`rendered=${result.rendered} chars=${result.chars} templates=${result.templates} rssKB=${rss} ms=${Date.now() - started}`);
}

module.exports = { runWorkload };
