'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { Deadline } = require('../src/deadline');
const { money, summarize } = require('../src/format');

test('a deadline trips its signal after the budget', async () => {
  const deadline = new Deadline(5);
  await new Promise((resolve) => setTimeout(resolve, 40));
  assert.strictEqual(deadline.signal.aborted, true);
  deadline.release();
});

test('a released deadline never trips', async () => {
  const deadline = new Deadline(5);
  deadline.release();
  await new Promise((resolve) => setTimeout(resolve, 40));
  assert.strictEqual(deadline.signal.aborted, false);
});

test('money formats cents with a sign', () => {
  assert.strictEqual(money(-1205), '-12.05');
  assert.strictEqual(money(7), '0.07');
});

test('summarize splits credits and debits', () => {
  const totals = summarize([{ amountCents: 500 }, { amountCents: -200 }]);
  assert.deepStrictEqual(totals, { count: 2, credit: 500, debit: -200, net: 300 });
});
