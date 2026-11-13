'use strict';

const { ledgerFor } = require('./ledger');
const { byCategory, runningBalance } = require('./aggregate');
const { renderReport } = require('./render');

const KEEP = 24;
const recent = new Map();

// Builds the month-end report for one account. The finance team re-opens the
// last few reports often, so the most recent ones are kept as rendered text.
function build(account, month, rows = 40000) {
  const key = `${account}/${month}`;
  if (recent.has(key)) {
    const text = recent.get(key);
    recent.delete(key);
    recent.set(key, text);
    return text;
  }
  const lines = ledgerFor(account, month, rows);
  const groups = byCategory(lines);
  const balances = runningBalance(lines);
  const text = renderReport(account, month, groups, balances);
  recent.set(key, text);
  if (recent.size > KEEP) recent.delete(recent.keys().next().value);
  return text;
}

function cachedCount() {
  return recent.size;
}

module.exports = { build, cachedCount };
