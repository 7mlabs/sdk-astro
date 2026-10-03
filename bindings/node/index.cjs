const path = require('node:path');
const filename = path.join(__dirname, 'native', `${process.platform}-${process.arch}`, 'astro.node');
let addon;
try { addon = require(filename); }
catch (cause) { throw new Error(`No usable astrology binary for ${process.platform}-${process.arch}. Build the local package or use a supported release.`, { cause }); }

function calculateJson(input) {
  if (typeof input !== 'string') throw new TypeError('calculateJson expects a JSON string');
  return addon.calculateJson(input);
}
function calculate(input) {
  return checkedResult(calculateJson(JSON.stringify(input)));
}
function checkedResult(output) {
  const result = JSON.parse(output);
  if (result.errors.length) {
    const error = new Error(result.errors[0].message);
    error.code = result.errors[0].code;
    error.result = result;
    throw error;
  }
  return result;
}
function compressPayloadJson(payloadJson, optionsJson = '{}') {
  if (typeof payloadJson !== 'string' || typeof optionsJson !== 'string')
    throw new TypeError('compressPayloadJson expects JSON strings for payload and options');
  // Preserve raw number literals. Rust validates each nested JSON value.
  return addon.compressJson(`{"payload":${payloadJson},"options":${optionsJson}}`);
}
function stringifyContextInput(value) {
  return JSON.stringify(value, (key, item) => {
    const type = typeof item;
    if ((type === 'number' && !Number.isFinite(item))
        || type === 'undefined' || type === 'function' || type === 'symbol' || type === 'bigint')
      throw new TypeError('Context inputs require JSON values; non-finite numbers and unsupported values cannot be preserved');
    if (item !== null && type === 'object') {
      if (Object.getOwnPropertySymbols(item).some(symbol => Object.prototype.propertyIsEnumerable.call(item, symbol)))
        throw new TypeError('Context inputs cannot contain enumerable symbol keys');
      if (item instanceof Number || item instanceof String || item instanceof Boolean)
        throw new TypeError('Context inputs require primitive JSON values, not boxed primitives');
    }
    return item;
  });
}
function compressPayload(payload, options = {}) {
  return checkedResult(compressPayloadJson(stringifyContextInput(payload), stringifyContextInput(options)));
}
function expandContextJson(contextJson) {
  if (typeof contextJson !== 'string') throw new TypeError('expandContextJson expects a JSON string');
  return addon.expandContextJson(contextJson);
}
function expandContext(context) {
  return checkedResult(expandContextJson(stringifyContextInput(context)));
}
function calculateWithContext(input, options = {}) {
  const result = calculate(input);
  return { result, context: compressPayload(result, options) };
}
function queryMethod(group, action) {
  return (options = {}) => {
    if (options === null || typeof options !== 'object' || Array.isArray(options))
      throw new TypeError(`${group}.${action} expects an options object`);
    for (const key of ['operation', 'group', 'action']) {
      if (Object.hasOwn(options, key)) throw new TypeError(`${key} is set by ${group}.${action}`);
    }
    return calculate({ ...options, operation: 'query', group, action });
  };
}

const geometry = Object.freeze({ normalize: queryMethod('geometry', 'normalize'),
  separation: queryMethod('geometry', 'separation'), midpoint: queryMethod('geometry', 'midpoint') });
const aspects = Object.freeze({ between: queryMethod('aspects', 'between'), inspect: queryMethod('aspects', 'inspect') });
const houses = Object.freeze({ locate: queryMethod('houses', 'locate'), inspect: queryMethod('houses', 'inspect') });
const points = Object.freeze({ inspect: queryMethod('points', 'inspect') });
module.exports = { calculate, calculateJson, compressPayload, compressPayloadJson,
  expandContext, expandContextJson, calculateWithContext, geometry, aspects, houses, points };
