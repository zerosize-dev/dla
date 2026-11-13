'use strict';

const { Signer } = require('./signer');
const { Gateway } = require('./gateway');

// The process serves every client through one gateway. The logical clock is
// advanced by whoever drives it (the replay in load.js, or a test).
const clock = { now: 0 };
const signer = new Signer('edge-secret-2024');
const gateway = new Gateway(signer, () => clock.now);

module.exports = { gateway, signer, clock };
