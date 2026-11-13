'use strict';

const { Session } = require('./session');
const { Counters } = require('./stats');
const { pack } = require('./frames');

class Relay {
  constructor(store) {
    this.store = store;
    this.counters = new Counters();
    this.sessions = new Map();
  }

  connect(device) {
    const session = new Session(device, this.store, this.counters);
    this.sessions.set(device, session);
    return session;
  }

  disconnect(device) {
    const session = this.sessions.get(device);
    if (!session) return false;
    session.close();
    this.sessions.delete(device);
    return true;
  }

  deliver(device, frame) {
    const session = this.sessions.get(device);
    if (!session) return 0;
    return session.feed(pack(frame));
  }
}

module.exports = { Relay };
