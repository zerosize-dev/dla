'use strict';

const { bus } = require('./bus');
const routes = require('./routes');
const audit = require('./audit');
const trail = require('./trail');

let sequence = 0;

function timings(request) {
  const stages = new Array(32);
  for (let i = 0; i < stages.length; i += 1) {
    stages[i] = ((request.parcel * (i + 3)) % 17) / 4;
  }
  return stages;
}

function handle(request) {
  const id = ++sequence;
  const route = routes.lookup(request.path);
  if (!route) {
    return { id, path: request.path, parcel: request.parcel, status: 404, body: '' };
  }
  const stages = timings(request);
  const body = route.render(request);
  const response = { id, path: request.path, parcel: request.parcel, status: 200, body };

  trail.track(response);
  audit.watch(request, id, stages);
  bus.emit(`settled:${id}`, { id, status: response.status, bytes: body.length });
  return response;
}

module.exports = { handle };
