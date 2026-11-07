'use strict';

const { encode } = require('./frames');

// Topic fan-out and message dispatch for sessions of one gateway.
class Router {
  constructor(metrics) {
    this.metrics = metrics;
    this.topics = new Map();
  }

  subscribe(session, topic) {
    let members = this.topics.get(topic);
    if (!members) {
      members = new Set();
      this.topics.set(topic, members);
    }
    members.add(session);
    session.topics.add(topic);
  }

  unsubscribe(session, topic) {
    const members = this.topics.get(topic);
    if (!members) return;
    members.delete(session);
    if (members.size === 0) this.topics.delete(topic);
    session.topics.delete(topic);
  }

  publish(from, topic, body) {
    const members = this.topics.get(topic);
    if (!members) return 0;
    const frame = encode({ type: 'event', topic, from: from.user, body });
    let delivered = 0;
    for (const member of members) {
      if (member === from) continue;
      member.sendRaw(frame);
      delivered += 1;
    }
    this.metrics.inc('events_out', delivered);
    return delivered;
  }

  dispatch(session, message) {
    switch (message.type) {
      case 'hello':
        session.user = String(message.user || 'anonymous');
        session.send({ type: 'welcome', id: session.id });
        break;
      case 'subscribe':
        this.subscribe(session, String(message.topic));
        break;
      case 'unsubscribe':
        this.unsubscribe(session, String(message.topic));
        break;
      case 'publish':
        this.publish(session, String(message.topic), message.body);
        break;
      case 'pong':
        break;
      case 'bye':
        session.close('bye');
        break;
      default:
        this.metrics.inc('unknown_type');
    }
  }
}

module.exports = { Router };
