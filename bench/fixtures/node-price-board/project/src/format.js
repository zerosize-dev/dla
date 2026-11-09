'use strict';

function price(value) {
  return value.toFixed(2);
}

function row(snapshot) {
  return `${snapshot.symbol.padEnd(5)} ${price(snapshot.vwap).padStart(8)} ${String(snapshot.count).padStart(5)}`;
}

module.exports = { price, row };
