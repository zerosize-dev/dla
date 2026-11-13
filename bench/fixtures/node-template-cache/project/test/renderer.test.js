'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { render, money } = require('../src/renderer');
const { compile } = require('../src/compile');
const { sourceFor, names } = require('../src/templates');
const cache = require('../src/cache');
const { runWorkload } = require('../load');

const NOTICE = {
  kind: 'receipt',
  locale: 'fr',
  customer: 'Mme Laurent',
  account: 'AC-100200',
  cents: 123456,
  date: '2025-03-04',
  effective: '2025-01-01',
};

test('compile keeps literals and fields apart', () => {
  const t = compile('Hello {{name}}, you owe {{amount}}.');
  assert.strictEqual(t.render({ name: 'Ada', amount: '3.50' }), 'Hello Ada, you owe 3.50.');
});

test('a missing field renders as nothing', () => {
  assert.strictEqual(compile('a{{x}}b').render({}), 'ab');
});

test('a notice is addressed in the customer language', () => {
  const text = render(NOTICE);
  assert.ok(text.startsWith('Cher Mme Laurent,'));
  assert.ok(text.includes('1234.56'));
  assert.ok(text.endsWith('Customer Care'));
});

test('the same notice kind and language share one compiled template', () => {
  render(NOTICE);
  const before = cache.size();
  render({ ...NOTICE, customer: 'Someone else' });
  assert.strictEqual(cache.size(), before);
});

test('every notice kind has a source', () => {
  for (const kind of names()) assert.ok(sourceFor(kind).includes('{{customer}}'));
  assert.strictEqual(money(5), '0.05');
});

test('the replay renders every notice it is given', () => {
  const result = runWorkload(300);
  assert.strictEqual(result.rendered, 300);
  assert.ok(result.templates > 0);
});
