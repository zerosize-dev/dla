'use strict';

const { movingAverage, volumeWeighted } = require('./stats');

class Board {
  constructor() {
    this.latest = null;
    this.sequence = 0;
  }

  // Builds the snapshot for one instrument from its latest ticks and makes it the
  // board's current snapshot. `options.log` receives a note about the change when
  // `options.verbose` is set.
  publish(symbol, ticks, options = {}) {
    const previousCount = this.latest ? this.latest.count : null;
    const changeNote = () =>
      previousCount !== null ? `${symbol}: ${previousCount} -> ${ticks.length} ticks` : `${symbol}: first snapshot`;

    const snapshot = {
      id: ++this.sequence,
      symbol,
      count: ticks.length,
      vwap: volumeWeighted(ticks),
      averages: movingAverage(ticks, 8),
      ticks,
      describe() {
        return `${symbol}#${snapshot.id} (${ticks.length} ticks)`;
      },
    };
    if (options.verbose && options.log) options.log(changeNote());
    this.latest = snapshot;
    return snapshot;
  }

  current() {
    return this.latest;
  }

  reset() {
    this.latest = null;
    this.sequence = 0;
  }
}

module.exports = (() => {
  const board = new Board();
  return {
    publish: (...args) => board.publish(...args),
    current: () => board.current(),
    reset: () => board.reset(),
  };
})();
