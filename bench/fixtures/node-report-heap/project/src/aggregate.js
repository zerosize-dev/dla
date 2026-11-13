'use strict';

function byCategory(lines) {
  const groups = new Map();
  for (const line of lines) {
    let group = groups.get(line.category);
    if (!group) {
      group = { category: line.category, lines: [], cents: 0 };
      groups.set(line.category, group);
    }
    group.lines.push({ ...line });
    group.cents += line.cents;
  }
  return groups;
}

function runningBalance(lines) {
  const sorted = lines.slice().sort((a, b) => a.day - b.day || a.cents - b.cents);
  let balance = 0;
  return sorted.map((line) => {
    balance += line.cents;
    return { day: line.day, balance };
  });
}

module.exports = { byCategory, runningBalance };
