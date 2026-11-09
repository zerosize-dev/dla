'use strict';

// Deterministic tick generator standing in for the upstream market feed.

const SYMBOLS = ['ALB', 'BRN', 'CTX', 'DLM', 'EVR', 'FJD', 'GLN', 'HRT'];

function makeRandom(seed) {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    return state / 4294967296;
  };
}

class Feed {
  constructor(seed, ticksPerUpdate) {
    this.random = makeRandom(seed);
    this.ticksPerUpdate = ticksPerUpdate;
    this.prices = new Map(SYMBOLS.map((s) => [s, 100 + Math.round(this.random() * 400)]));
  }

  next() {
    const symbol = SYMBOLS[Math.floor(this.random() * SYMBOLS.length)];
    let price = this.prices.get(symbol);
    const ticks = new Array(this.ticksPerUpdate);
    for (let i = 0; i < ticks.length; i += 1) {
      price = Math.max(1, price + (this.random() - 0.5) * 2);
      ticks[i] = { t: i, price: Math.round(price * 100) / 100, size: 1 + Math.floor(this.random() * 500) };
    }
    this.prices.set(symbol, price);
    return { symbol, ticks };
  }
}

module.exports = { Feed, SYMBOLS };
