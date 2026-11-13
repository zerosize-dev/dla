'use strict';

// Template sources. Placeholders are written {{name}}; a section can be
// repeated by listing it once and letting the compiler keep it as one part.

const KINDS = [
  'statement', 'reminder', 'receipt', 'welcome', 'password-reset', 'limit-change',
  'dispute-opened', 'dispute-closed', 'card-replaced', 'transfer-scheduled',
  'transfer-failed', 'tax-summary', 'fee-notice', 'closure',
];

function paragraph(kind, n) {
  return `Section ${n} of the ${kind} notice for {{customer}}: your account {{account}} ` +
    `shows a balance of {{amount}} as of {{date}}. Terms for ${kind} notices are described ` +
    `in schedule ${n} of your agreement, which applies from {{effective}}. `;
}

function sourceFor(kind) {
  const parts = ['{{greeting}} {{customer}},\n\n'];
  for (let n = 1; n <= 90; n += 1) parts.push(paragraph(kind, n));
  parts.push('\n\n{{closing}},\nCustomer Care');
  return parts.join('');
}

function names() {
  return KINDS.slice();
}

module.exports = { sourceFor, names };
