'use strict';

const { EventEmitter } = require('node:events');

// Process-wide event bus shared by the gateway modules.
const bus = new EventEmitter();
bus.setMaxListeners(0);

module.exports = { bus };
