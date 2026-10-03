#!/usr/bin/env node
'use strict';
// Development-only schema/native agreement checks using an installed SDK.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Ajv2020 = require(process.env.AJV_2020_MODULE || 'ajv/dist/2020.js');
const { compressPayloadJson, expandContextJson } = require(process.env.ASTRO_ENGINE_MODULE || '@7mlabs/astrology');
const root = path.resolve(__dirname, '..');
const ajv = new Ajv2020({ strict: true, strictRequired: false, allErrors: true, allowUnionTypes: true });
const requestSchema = JSON.parse(fs.readFileSync(path.join(root, 'schemas/compression-request.schema.json'), 'utf8'));
const responseSchema = JSON.parse(fs.readFileSync(path.join(root, 'schemas/compression-response.schema.json'), 'utf8'));
const validateRequest = ajv.compile(requestSchema);
const validateResponse = ajv.compile(responseSchema);
const source = {
  schemaVersion: '1.0', engineVersion: 'test-source-version',
  data: { nullable: null, applying: null, precise: 280.36891967534336 },
  warnings: ['Retain this warning'], errors: []
};
const cases = [
  ['default options', '{}', true],
  ['compact', '{"mode":"compact"}', true],
  ['empty selectors', '{"domains":[],"sections":[]}', true],
  ['focused without selectors', '{"mode":"focused"}', true],
  ['budgeted without selectors', '{"mode":"budgeted"}', true],
  ['integer decimal notation', '{"mode":"budgeted","maxBytes":1.0}', true],
  ['integer exponent notation', '{"mode":"budgeted","maxTokens":2e3}', true],
  ['upper budget cap', '{"maxBytes":268435456,"maxTokens":268435456}', true],
  ['noninteger budget', '{"maxBytes":1.5}', false],
  ['zero budget', '{"maxTokens":0}', false],
  ['above budget cap', '{"maxBytes":268435457}', false],
  ['null budget', '{"maxBytes":null}', false],
  ['unknown mode', '{"mode":"lossy"}', false],
  ['null mode', '{"mode":null}', false],
  ['unknown option', '{"extra":true}', false],
  ['nonobject options', 'null', false],
  ['selector requires focused mode', '{"domains":["career"]}', false],
  ['compact cannot select', '{"mode":"compact","sections":["profession"]}', false],
  ['duplicate selector', '{"mode":"focused","domains":["career","career"]}', false],
  ['empty selector ID', '{"mode":"focused","domains":[""]}', false],
  ['nonstring selector ID', '{"mode":"focused","domains":[7]}', false]
];
let contractCases = 0;
for (const [label, rawOptions, valid] of cases) {
  const options = JSON.parse(rawOptions);
  assert.equal(validateRequest({ payload: source, options }), valid, `${label}: ${ajv.errorsText(validateRequest.errors)}`);
  const result = JSON.parse(compressPayloadJson(JSON.stringify(source), rawOptions));
  assert.ok(validateResponse(result), `${label}: ${ajv.errorsText(validateResponse.errors)}`);
  assert.equal(result.errors.length === 0, valid, `${label}: ${JSON.stringify(result.errors)}`);
  if (valid) assert.deepEqual(JSON.parse(expandContextJson(JSON.stringify(result))), source);
  contractCases++;
}

for (const [label, payload] of [
  ['extra envelope field', { ...source, unexpected: true }],
  ['wrong schema version', { ...source, schemaVersion: '2.0' }],
  ['nonstring warning', { ...source, warnings: [null] }],
  ['extra source error field', { ...source, errors: [{ code: 'INVALID_INPUT', message: 'bad', extra: 1 }] }],
  ['missing source data', { schemaVersion: '1.0', engineVersion: 'v', warnings: [], errors: [] }]
]) {
  assert.equal(validateRequest({ payload }), false, label);
  const result = JSON.parse(compressPayloadJson(JSON.stringify(payload)));
  assert.ok(validateResponse(result), `${label}: ${ajv.errorsText(validateResponse.errors)}`);
  assert.ok(result.errors.length > 0, label);
  contractCases++;
}
// A known-shape selector may be absent semantically. Schema does not claim to
// resolve IDs; the native core returns the explicit selector error.
const absentOptions = { mode: 'focused', domains: ['absent'] };
assert.ok(validateRequest({ payload: source, options: absentOptions }));
const absent = JSON.parse(compressPayloadJson(JSON.stringify(source), JSON.stringify(absentOptions)));
assert.ok(validateResponse(absent));
assert.equal(absent.errors[0].code, 'COMPRESSION_SELECTOR_INVALID');
contractCases++;

const defaultFiles = [
  'natal', 'natal-domains', 'couple', 'composite', 'couple-composite',
  'forecast-day', 'forecast-month', 'forecast-year', 'events-year',
  'custom-profiles', 'custom-couple', 'custom-composite', 'custom-forecast'
].map(stem => path.join(root, 'examples', `${stem}-result.json`));
const files = process.argv.length > 2 ? process.argv.slice(2) : defaultFiles;
const measurements = [];
for (const filename of files) {
  const payloadJson = fs.readFileSync(filename, 'utf8');
  const payload = JSON.parse(payloadJson);
  assert.ok(validateRequest({ payload }), `${filename}: ${ajv.errorsText(validateRequest.errors)}`);
  const rawContext = compressPayloadJson(payloadJson);
  const context = JSON.parse(rawContext);
  assert.ok(validateResponse(context), `${filename}: ${ajv.errorsText(validateResponse.errors)}`);
  assert.deepEqual(context.errors, [], filename);
  assert.equal(context.data.coverage.complete, true, filename);
  assert.deepEqual(context.data.omitted, [], filename);
  assert.equal(context.data.metrics.outputBytes, Buffer.byteLength(rawContext, 'utf8'), filename);
  assert.deepEqual(JSON.parse(expandContextJson(rawContext)), payload, filename);
  measurements.push({ fixture: path.basename(filename), inputBytes: context.data.metrics.inputBytes, outputBytes: context.data.metrics.outputBytes });
}

const collision = { ...source, data: {
  reserved: { $astroRef: 'source-owned-key', $astroTable: { columns: ['source'], rows: [] }, $astroLiteral: 'source-owned-key' },
  rows: Array.from({ length: 12 }, (_, index) => ({ pointId: `point-${index}`, longitude: 12.34567890123456 + index, applying: null })),
  unicode: 'Công việc — 情感'
} };
const collisionContext = JSON.parse(compressPayloadJson(JSON.stringify(collision)));
assert.ok(validateResponse(collisionContext), ajv.errorsText(validateResponse.errors));
assert.deepEqual(JSON.parse(expandContextJson(JSON.stringify(collisionContext))), collision);
const sourceError = { ...source, data: null, errors: [{ code: 'INVALID_INPUT', message: 'source error retained' }] };
const errorContext = JSON.parse(compressPayloadJson(JSON.stringify(sourceError)));
assert.ok(validateResponse(errorContext), ajv.errorsText(validateResponse.errors));
assert.deepEqual(errorContext.errors, []);
assert.deepEqual(JSON.parse(expandContextJson(JSON.stringify(errorContext))), sourceError);

const metadataSource = JSON.parse(compressPayloadJson(JSON.stringify(source)));
for (const field of ['mode', 'coverage', 'omitted', 'metrics', 'budget']) {
  const incomplete = structuredClone(metadataSource);
  delete incomplete.data[field];
  assert.equal(validateResponse(incomplete), false, `Missing ${field} must violate schema`);
  const result = JSON.parse(expandContextJson(JSON.stringify(incomplete)));
  assert.ok(validateResponse(result), ajv.errorsText(validateResponse.errors));
  assert.equal(result.errors[0].code, 'COMPRESSION_CONTEXT_INVALID', `Missing ${field} must violate native decoder contract`);
}
let nested = { precise: 12.34567890123456, unknown: null };
for (let depth = 0; depth < 60; depth++) nested = { $astroRef: nested };
const deepPayload = { ...source, data: nested };
assert.ok(validateRequest({ payload: deepPayload }));
const tooDeep = JSON.parse(compressPayloadJson(JSON.stringify(deepPayload)));
assert.ok(validateResponse(tooDeep), ajv.errorsText(validateResponse.errors));
assert.equal(tooDeep.errors[0].code, 'COMPRESSION_OUTPUT_TOO_DEEP');

console.log(JSON.stringify({ result: 'passed', contractCases, realFixtureRoundtrips: files.length, encodedCollisionRoundtrip: true, sourceErrorPreserved: true, incompleteMetadataRejected: true, deepEncodingRejected: true, measurements }, null, 2));
