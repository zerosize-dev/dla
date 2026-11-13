'use strict';

// Splits a template into literal runs and placeholders once, so that rendering
// is a single pass over the parts.
function compile(source) {
  const parts = [];
  const pattern = /\{\{(\w+)\}\}/g;
  let last = 0;
  let match;
  while ((match = pattern.exec(source)) !== null) {
    if (match.index > last) parts.push({ literal: source.slice(last, match.index) });
    parts.push({ field: match[1] });
    last = pattern.lastIndex;
  }
  if (last < source.length) parts.push({ literal: source.slice(last) });
  return {
    parts,
    render(values) {
      let out = '';
      for (const part of parts) out += part.literal !== undefined ? part.literal : String(values[part.field] ?? '');
      return out;
    },
  };
}

module.exports = { compile };
