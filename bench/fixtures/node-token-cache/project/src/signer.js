'use strict';

const crypto = require('node:crypto');
const { encode, signingInput } = require('./token');

class Signer {
  constructor(secret) {
    this.secret = secret;
  }

  sign(claims) {
    const mac = crypto.createHmac('sha256', this.secret).update(signingInput(claims)).digest('hex');
    return encode(claims, mac.slice(0, 32));
  }

  verify(claims, signature) {
    const mac = crypto.createHmac('sha256', this.secret).update(signingInput(claims)).digest('hex');
    return mac.slice(0, 32) === signature;
  }
}

module.exports = { Signer };
