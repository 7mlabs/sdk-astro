'use strict';
const assert = require('node:assert/strict');
const { calculate, compressPayload, expandContext, calculateWithContext } = require(process.env.ASTRO_ENGINE_MODULE || '@7mlabs/astrology');

const request = {
  operation: 'natalDomains',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  domains: ['career', 'love'], aspectPreset: 'extended'
};
const result = calculate(request);
const compact = compressPayload(result);
assert.deepEqual(expandContext(compact), result);

const { result: original, context } = calculateWithContext(request, {
  mode: 'budgeted', domains: ['career'], maxBytes: 64 * 1024
});
const retained = expandContext(context);
assert.equal(original.data.domains.love !== undefined, true);
assert.deepEqual(Object.keys(retained.data.domains), ['career']);
console.log(JSON.stringify({
  compact: compact.data.metrics,
  focused: context.data.metrics,
  coverage: context.data.coverage,
  omitted: context.data.omitted,
  budget: context.data.budget,
  warnings: context.warnings
}, null, 2));
// Send the complete context envelope, including dictionary and coverage, to the
// LLM. Keep original for UI/audit; maxBytes counts the native minified envelope.
if (process.argv.includes('--payload')) console.log(JSON.stringify(context));
