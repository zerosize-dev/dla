'use strict';

const test = require('node:test');
const assert = require('node:assert');
const lifecycle = require('../src/lifecycle');
const { handle } = require('../src/handler');
const { PAGE_SIZE, PAGES } = require('../src/upstream');

test('an export returns every line of the statement', async () => {
  const result = await handle({ account: 1007, budgetMs: 5000 });
  assert.strictEqual(result.status, 200);
  assert.strictEqual(result.totals.count, PAGE_SIZE * PAGES);
  assert.ok(result.bytes > 0);
});

test('the same account gives the same totals', async () => {
  const a = await handle({ account: 1042, budgetMs: 5000 });
  const b = await handle({ account: 1042, budgetMs: 5000 });
  assert.deepStrictEqual(a.totals, b.totals);
});

test('termination winds in-flight exports down', async () => {
  lifecycle.onTerminate('test');
  const result = await handle({ account: 1007, budgetMs: 5000 });
  assert.strictEqual(result.status, 503);
  assert.match(result.reason, /terminating: test/);
});
