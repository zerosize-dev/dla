'use strict';

const test = require('node:test');
const assert = require('node:assert');
const reports = require('../src/reports');
const { ledgerFor } = require('../src/ledger');
const { byCategory, runningBalance } = require('../src/aggregate');
const { cents } = require('../src/render');

test('the ledger stand-in is deterministic', () => {
  assert.deepStrictEqual(ledgerFor(5, 3, 10), ledgerFor(5, 3, 10));
});

test('categories add up to the ledger total', () => {
  const lines = ledgerFor(7, 2, 500);
  const total = lines.reduce((sum, l) => sum + l.cents, 0);
  const grouped = [...byCategory(lines).values()].reduce((sum, g) => sum + g.cents, 0);
  assert.strictEqual(grouped, total);
});

test('the running balance ends at the ledger total', () => {
  const lines = ledgerFor(9, 4, 300);
  const balances = runningBalance(lines);
  assert.strictEqual(balances[balances.length - 1].balance, lines.reduce((s, l) => s + l.cents, 0));
});

test('a report names the account and closes with a balance', () => {
  const text = reports.build(1234, 6, 200);
  assert.match(text, /^account 1234 month 6/);
  assert.match(text, /closing balance -?\d+\.\d\d$/);
});

test('re-opening a report returns the same text', () => {
  const first = reports.build(1300, 1, 200);
  assert.strictEqual(reports.build(1300, 1, 200), first);
  assert.strictEqual(cents(-1205), '-12.05');
});
