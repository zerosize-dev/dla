'use strict';

const CARRIERS = ['northline', 'harbor-post', 'kestrel', 'meridian-air'];

function checkpoints(parcel, count) {
  const list = [];
  for (let i = 0; i < count; i += 1) {
    list.push(`${CARRIERS[(parcel + i) % CARRIERS.length]}:hub-${(parcel * 31 + i * 7) % 90}`);
  }
  return list;
}

const ROUTES = {
  '/parcels/track': {
    render(request) {
      return JSON.stringify({ parcel: request.parcel, checkpoints: checkpoints(request.parcel, 9) });
    },
  },
  '/parcels/eta': {
    render(request) {
      return JSON.stringify({ parcel: request.parcel, days: 1 + (request.parcel % 6) });
    },
  },
  '/parcels/label': {
    render(request) {
      return JSON.stringify({ parcel: request.parcel, label: `LBL-${request.parcel.toString(36).toUpperCase()}` });
    },
  },
};

function lookup(path) {
  return ROUTES[path] || null;
}

module.exports = { lookup, ROUTES };
