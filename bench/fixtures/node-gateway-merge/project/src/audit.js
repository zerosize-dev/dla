'use strict';

const { bus } = require('./bus');

// Keeps a short account of how each request settled, for the operations dashboard.
const settled = [];
const MAX_NOTES = 200;

function note(text) {
  settled.push(text);
  if (settled.length > MAX_NOTES) settled.shift();
}

// Called once per request: waits for the request's own settle event and writes
// a note with the timings that were collected on the way.
function watch(request, id, stages) {
  const context = { path: request.path, parcel: request.parcel, client: request.client, stages };
  bus.on(`settled:${id}`, (event) => {
    const total = context.stages.reduce((sum, ms) => sum + ms, 0);
    note(`${context.path} #${id} ${event.status} ${event.bytes}B ${total.toFixed(1)}ms`);
  });
}

function recent() {
  return settled.slice();
}

module.exports = { watch, recent };
