'use strict';

// Deterministic stand-in for the general ledger export.

const CATEGORIES = ['rent', 'payroll', 'travel', 'software', 'tax', 'supplies', 'utilities', 'fees'];

function ledgerFor(account, month, rows) {
  const lines = new Array(rows);
  let seed = (account * 2654435761 + month * 40503) >>> 0;
  for (let i = 0; i < rows; i += 1) {
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    lines[i] = {
      account,
      month,
      day: 1 + (seed % 28),
      category: CATEGORIES[(seed >>> 8) % CATEGORIES.length],
      cents: ((seed >>> 4) % 900000) - 300000,
      memo: `entry ${i} of ${account}/${month} ref ${(seed % 65521).toString(16)}`,
    };
  }
  return lines;
}

module.exports = { ledgerFor, CATEGORIES };
