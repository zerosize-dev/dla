'use strict';

function movingAverage(ticks, window) {
  const out = [];
  let sum = 0;
  for (let i = 0; i < ticks.length; i += 1) {
    sum += ticks[i].price;
    if (i >= window) sum -= ticks[i - window].price;
    out.push(sum / Math.min(i + 1, window));
  }
  return out;
}

function volumeWeighted(ticks) {
  let value = 0;
  let volume = 0;
  for (const tick of ticks) {
    value += tick.price * tick.size;
    volume += tick.size;
  }
  return volume === 0 ? 0 : value / volume;
}

module.exports = { movingAverage, volumeWeighted };
