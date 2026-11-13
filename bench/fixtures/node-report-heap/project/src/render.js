'use strict';

function cents(value) {
  const sign = value < 0 ? '-' : '';
  const abs = Math.abs(value);
  return `${sign}${Math.floor(abs / 100)}.${String(abs % 100).padStart(2, '0')}`;
}

function renderReport(account, month, groups, balances) {
  const out = [`account ${account} month ${month}`];
  for (const group of [...groups.values()].sort((a, b) => a.category.localeCompare(b.category))) {
    out.push(`${group.category.padEnd(10)} ${String(group.lines.length).padStart(6)} ${cents(group.cents).padStart(12)}`);
  }
  const closing = balances.length ? balances[balances.length - 1].balance : 0;
  out.push(`closing balance ${cents(closing)}`);
  return out.join('\n');
}

module.exports = { renderReport, cents };
