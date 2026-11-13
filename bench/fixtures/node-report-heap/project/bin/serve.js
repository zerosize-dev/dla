'use strict';

// Production entry point: replays the month-end queue and prints a summary.
// Started through `npm start` so that the old-space limit in package.json applies.

const { runWorkload } = require('../load');

const result = runWorkload(Number.parseInt(process.env.REPORTS || '60', 10));
console.log(`reports=${result.reports} lines=${result.lines} cached=${result.cached}`);
