#!/usr/bin/env node
'use strict';

// Optional protocol adapter. The native engine owns every calculation and error.
const fs = require('node:fs');
const path = require('node:path');
const { Server } = require('@modelcontextprotocol/sdk/server/index.js');
const { StdioServerTransport } = require('@modelcontextprotocol/sdk/server/stdio.js');
const { CallToolRequestSchema, ListToolsRequestSchema, McpError, ErrorCode } = require('@modelcontextprotocol/sdk/types.js');
const Ajv2020 = require('ajv/dist/2020.js');
const { calculateJson, compressPayloadJson } = require(process.env.ASTRO_ENGINE_MODULE || '@7mlabs/astrology');

const schemaDirectory = process.env.ASTRO_SCHEMA_DIR || path.resolve(__dirname, '../../schemas');
const querySchema = JSON.parse(fs.readFileSync(path.join(schemaDirectory, 'query-request.schema.json'), 'utf8'));
const queryResponseSchema = JSON.parse(fs.readFileSync(path.join(schemaDirectory, 'query-response.schema.json'), 'utf8'));
const compressionRequestSchema = JSON.parse(fs.readFileSync(path.join(schemaDirectory, 'compression-request.schema.json'), 'utf8'));
const compressionResponseSchema = JSON.parse(fs.readFileSync(path.join(schemaDirectory, 'compression-response.schema.json'), 'utf8'));
const ajv = new Ajv2020({ strict: true, strictRequired: false, allErrors: true, allowUnionTypes: true });
const validateQueryResponse = ajv.compile(queryResponseSchema);
const validateCompressionResponse = ajv.compile(compressionResponseSchema);
const annotations = { readOnlyHint: true, destructiveHint: false, idempotentHint: true, openWorldHint: false };
// This advertises the common envelope; the canonical query response schema is
// additionally enforced by the adapter for the four grouped query tools.
const outputSchema = {
  type: 'object', additionalProperties: false,
  required: ['schemaVersion', 'engineVersion', 'data', 'warnings', 'errors'],
  properties: {
    schemaVersion: { const: '1.0' }, engineVersion: { type: 'string' }, calculation: {}, data: {},
    warnings: { type: 'array', items: { type: 'string' } },
    errors: { type: 'array', items: {
      type: 'object', additionalProperties: false, required: ['code', 'message'],
      properties: { code: { type: 'string' }, message: { type: 'string' } }
    } }
  }
};
const validateEnvelope = ajv.compile(outputSchema);

function groupSchema(group) {
  const branches = querySchema.oneOf.filter(branch => branch.properties.group.const === group)
    .map(branch => {
      const derived = JSON.parse(JSON.stringify(branch));
      delete derived.properties.operation;
      delete derived.properties.group;
      derived.required = derived.required.filter(key => !['operation', 'group'].includes(key));
      return derived;
    });
  if (branches.length === 0) throw new Error(`Query schema has no group ${group}`);
  return { type: 'object', oneOf: branches, $defs: querySchema.$defs };
}

const catalog = [
  ['geometry', 'Longitude geometry', 'Normalize a longitude, compute angular separation, or find a midpoint from supplied numbers.'],
  ['aspects', 'Aspect queries', 'Inspect one supplied pair against a rule, or inspect natal contacts between selected points. Motion inference is explicit for supplied pairs.'],
  ['houses', 'House queries', 'Locate a supplied longitude in twelve cusps, or inspect natal houses and their supporting facts.'],
  ['points', 'Point queries', 'Inspect selected natal bodies, angles or house cusps with supporting houses and relations.']
];
const tools = catalog.map(([group, title, description]) => {
  const inputSchema = groupSchema(group);
  return { name: `astro_${group}`, title, description, inputSchema, outputSchema, annotations };
});
tools.push({
  name: 'astro_calculate', title: 'Native engine calculation',
  description: 'Forward an exact raw JSON request to the offline native engine. Supports chart, harmonic, synastry, natal, natalDomains, couple, composite, events, forecast and query. Returns facts and the complete engine envelope.',
  inputSchema: { type: 'object', additionalProperties: false, required: ['requestJson'],
    properties: { requestJson: { type: 'string', minLength: 2, maxLength: 4 * 1024 * 1024,
      description: 'Engine request serialized as JSON. A string preserves exact numeric literals for integer validation.' } } },
  outputSchema, annotations
});
const contextInput = (field, description) => ({
  type: 'object', additionalProperties: false, required: [field],
  properties: {
    [field]: { type: 'string', minLength: 2, description },
    options: { $ref: '#/$defs/options' }
  },
  $defs: compressionRequestSchema.$defs
});
tools.push({
  name: 'astro_compress_payload', title: 'Prepare existing engine payload for an LLM',
  description: 'Prepare a complete engine envelope already calculated locally. Compact mode preserves JSON values; focused and budgeted modes disclose omissions and retain supporting facts. Read coverage, warnings and budget.exceeded before interpreting. maxTokens uses a conservative UTF-8 byte estimate, not a model tokenizer.',
  inputSchema: contextInput('payloadJson', 'Complete engine response serialized as JSON, including data, warnings, errors and source versions.'),
  outputSchema: compressionResponseSchema, annotations
}, {
  name: 'astro_calculate_context', title: 'Calculate and prepare context locally',
  description: 'Run any native engine request and return only its prepared astro-context/1 envelope. Resolving $astroRef uses dictionary; $astroTable rows use its columns. Preserve source warnings, read coverage and budget.exceeded. No interpretation or LLM request is performed.',
  inputSchema: contextInput('requestJson', 'Exact engine request serialized as JSON. Native calculation validates its 4 MiB request limit and numeric literals.'),
  outputSchema: compressionResponseSchema, annotations
});
const validators = new Map(tools.map(tool => [tool.name, ajv.compile(tool.inputSchema)]));
const probe = JSON.parse(calculateJson('{"operation":"query","group":"geometry","action":"normalize","longitude":0}'));
if (probe.errors.length || typeof compressPayloadJson !== 'function') throw new Error('This adapter requires an engine supporting query and compression (0.10.0-alpha.1 or later).');

const server = new Server({ name: 'sevenmlabs-astrology', version: probe.engineVersion }, {
  capabilities: { tools: { listChanged: false } },
  instructions: 'All calculations and context preparation run locally. Group tools accept action and its arguments; operation and group are fixed by the tool. Interpretations and scores are outside the engine. Context payloads use $astroTable columns/rows, $astroRef dictionary lookups, and $astroLiteral escaped original objects. Inspect omissions and budget.exceeded; maxTokens is a conservative UTF-8 byte estimate. Preserve warnings and subject-local reference scopes.'
});
server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools }));
server.setRequestHandler(CallToolRequestSchema, async request => {
  const { name } = request.params;
  const validate = validators.get(name);
  if (!validate) throw new McpError(ErrorCode.InvalidParams, `Unknown tool: ${name}`);
  const args = request.params.arguments || {};
  if (!validate(args)) throw new McpError(ErrorCode.InvalidParams, `Invalid arguments for ${name}: ${ajv.errorsText(validate.errors)}`);
  const isContext = name === 'astro_compress_payload' || name === 'astro_calculate_context';
  let rawResult;
  if (name === 'astro_compress_payload') {
    rawResult = compressPayloadJson(args.payloadJson, JSON.stringify(args.options || {}));
  } else {
    const rawRequest = name === 'astro_calculate' || name === 'astro_calculate_context' ? args.requestJson
      : JSON.stringify({ ...args, operation: 'query', group: name.slice('astro_'.length) });
    rawResult = calculateJson(rawRequest);
    // A failed calculation is a tool error. Do not wrap it in a successful
    // context envelope and hide the original error from the MCP host.
    if (name === 'astro_calculate_context' && JSON.parse(rawResult).errors.length === 0) {
      rawResult = compressPayloadJson(rawResult, JSON.stringify(args.options || {}));
    }
  }
  const result = JSON.parse(rawResult);
  if (!validateEnvelope(result)) throw new McpError(ErrorCode.InternalError, 'Native engine returned an invalid envelope.');
  if (isContext && !validateCompressionResponse(result)) {
    throw new McpError(ErrorCode.InternalError, `Native context response violates its schema: ${ajv.errorsText(validateCompressionResponse.errors)}`);
  }
  if (!isContext && name !== 'astro_calculate' && !validateQueryResponse(result)) {
    throw new McpError(ErrorCode.InternalError, `Native query response violates its schema: ${ajv.errorsText(validateQueryResponse.errors)}`);
  }
  return { content: [{ type: 'text', text: rawResult }], structuredContent: result, isError: result.errors.length > 0 };
});

server.connect(new StdioServerTransport()).catch(error => {
  console.error(error.message); // stdout belongs exclusively to the MCP transport.
  process.exitCode = 1;
});
