'use strict';

const { sourceFor } = require('./templates');
const { compile } = require('./compile');

// Compiled templates, one per notice kind and language.
const compiled = new Map();

function templateFor(kind, locale) {
  const key = `${kind}@${locale}`;
  let template = compiled.get(key);
  if (!template) {
    template = compile(sourceFor(kind));
    compiled.set(key, template);
  }
  return template;
}

function size() {
  return compiled.size;
}

module.exports = { templateFor, size };
