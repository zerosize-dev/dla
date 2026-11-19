'use strict';

// Stand-in for the ledger service the exports read from: pages of statement
// lines, generated deterministically from the account and the page number.

const PAGE_SIZE = 40;
const PAGES = 3;

function lineFor(account, page, index) {
  const seed = (account * 7919 + page * 104729 + index * 15485863) >>> 0;
  return {
    account,
    booked: 20240101 + (seed % 365),
    amountCents: (seed % 250000) - 100000,
    memo: `ref-${(seed % 99991).toString(36)}`,
  };
}

async function readPage(account, page, signal) {
  signal.throwIfAborted();
  await new Promise((resolve) => setImmediate(resolve));
  signal.throwIfAborted();
  const lines = new Array(PAGE_SIZE);
  for (let i = 0; i < PAGE_SIZE; i += 1) lines[i] = lineFor(account, page, i);
  return lines;
}

async function readStatement(account, signal) {
  const lines = [];
  for (let page = 0; page < PAGES; page += 1) {
    lines.push(...(await readPage(account, page, signal)));
  }
  return lines;
}

module.exports = { readStatement, PAGE_SIZE, PAGES };
