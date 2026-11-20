'use strict';

function money(cents) {
  const sign = cents < 0 ? '-' : '';
  const abs = Math.abs(cents);
  return `${sign}${Math.floor(abs / 100)}.${String(abs % 100).padStart(2, '0')}`;
}

function csvLine(line) {
  return [line.booked, money(line.amountCents), line.memo].join(',');
}

function summarize(lines) {
  let credit = 0;
  let debit = 0;
  for (const line of lines) {
    if (line.amountCents >= 0) credit += line.amountCents;
    else debit += line.amountCents;
  }
  return { count: lines.length, credit, debit, net: credit + debit };
}

module.exports = { money, csvLine, summarize };
