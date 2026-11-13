'use strict';

// Lets support staff ask how one response relates to the one served before it
// (same parcel again, a retry after an error, a different client).
class Trail {
  constructor() {
    this.lastResponse = null;
  }

  track(response) {
    const earlier = this.lastResponse;
    response.explain = () => {
      if (!earlier) return `${response.path} #${response.id}: first response`;
      const same = earlier.parcel === response.parcel ? 'same parcel' : 'other parcel';
      return `${response.path} #${response.id}: ${same} as #${earlier.id} (${earlier.status})`;
    };
    this.lastResponse = response;
  }

  latest() {
    return this.lastResponse;
  }
}

module.exports = (() => {
  const trail = new Trail();
  return {
    track: (response) => trail.track(response),
    latest: () => trail.latest(),
  };
})();
