'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { Client } = require('@modelcontextprotocol/sdk/client/index.js');
const { StdioClientTransport, getDefaultEnvironment } = require('@modelcontextprotocol/sdk/client/stdio.js');
const { calculateJson, compressPayloadJson, expandContext } = require(process.env.ASTRO_ENGINE_MODULE || '@7mlabs/astrology');

const environment = getDefaultEnvironment();
for (const key of ['NODE_PATH', 'ASTRO_ENGINE_MODULE', 'ASTRO_SCHEMA_DIR']) {
  if (process.env[key]) environment[key] = process.env[key];
}
const transport = new StdioClientTransport({
  command: process.execPath, args: [path.join(__dirname, 'server.cjs')],
  env: environment, stderr: 'pipe'
});
let stderr = '';
transport.stderr.on('data', chunk => { stderr += chunk.toString(); });
const client = new Client({ name: 'astrology-compression-smoke', version: '1.0.0' });
let calls = 0;
async function call(name, args, expected) {
  const response = await client.callTool({ name, arguments: args }, undefined, { timeout: 60000 });
  assert.deepEqual(response.structuredContent, expected);
  assert.deepEqual(JSON.parse(response.content[0].text), expected);
  assert.equal(response.isError, expected.errors.length > 0);
  calls++;
  return response.structuredContent;
}

(async () => {
  try {
    await client.connect(transport);
    const listed = await client.listTools();
    assert.equal(listed.tools.length, 7);
    assert.ok(listed.tools.some(tool => tool.name === 'astro_compress_payload'));
    assert.ok(listed.tools.some(tool => tool.name === 'astro_calculate_context'));
    for (const file of ['natal-request.json', 'custom-couple-request.json', 'composite-request.json', 'forecast-day-request.json']) {
      const requestJson = fs.readFileSync(path.resolve(__dirname, '..', file), 'utf8');
      const payloadJson = calculateJson(requestJson);
      const source = JSON.parse(payloadJson);
      assert.equal(source.errors.length, 0, file);
      const expected = JSON.parse(compressPayloadJson(payloadJson));
      const existing = await call('astro_compress_payload', { payloadJson }, expected);
      assert.deepEqual(expandContext(existing), source, file);
      const combined = await call('astro_calculate_context', { requestJson }, expected);
      assert.equal(combined.data.format, 'astro-context/1');
      assert.equal(Object.hasOwn(combined, 'result'), false);
      assert.equal(Object.hasOwn(combined, 'context'), false);
    }
    const requestJson = fs.readFileSync(path.resolve(__dirname, '..', 'natal-domains-request.json'), 'utf8');
    const payloadJson = calculateJson(requestJson);
    const focusedOptions = { mode: 'focused', domains: ['career'] };
    const focused = await call('astro_compress_payload', { payloadJson, options: focusedOptions }, JSON.parse(compressPayloadJson(payloadJson, JSON.stringify(focusedOptions))));
    const restored = expandContext(focused);
    assert.deepEqual(Object.keys(restored.data.domains), ['career']);
    assert.equal(focused.data.coverage.complete, false);
    assert.ok(focused.data.omitted.length > 0);

    const budgetOptions = { mode: 'budgeted', domains: ['career'], maxBytes: 128, maxTokens: 128 };
    const budget = await call('astro_calculate_context', { requestJson, options: budgetOptions }, JSON.parse(compressPayloadJson(payloadJson, JSON.stringify(budgetOptions))));
    assert.equal(budget.data.budget.exceeded, true);
    assert.ok(budget.warnings.some(warning => warning.includes('COMPRESSION_BUDGET_EXCEEDED')));
    assert.deepEqual(expandContext(budget).data.domains.career, restored.data.domains.career);

    const unknown = { mode: 'focused', domains: ['missing-profile'] };
    const invalid = await call('astro_compress_payload', { payloadJson, options: unknown }, JSON.parse(compressPayloadJson(payloadJson, JSON.stringify(unknown))));
    assert.equal(invalid.errors[0].code, 'COMPRESSION_SELECTOR_INVALID');
    const rawInvalid = 'no json';
    const malformed = await call('astro_compress_payload', { payloadJson: rawInvalid }, JSON.parse(compressPayloadJson(rawInvalid)));
    assert.ok(malformed.errors.length > 0);
    const badCalendar = '{"operation":"natal","utc":{"year":2000,"month":2,"day":31,"hour":12,"minute":0},"location":{"latitude":0,"longitude":0}}';
    const badCalendarResult = await call('astro_calculate_context', { requestJson: badCalendar }, JSON.parse(calculateJson(badCalendar)));
    assert.equal(badCalendarResult.errors[0].code, 'INVALID_INPUT');
    const fraction = '{"operation":"natal","utc":{"year":2000.0000000000000001,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":0,"longitude":0}}';
    const fractionResult = await call('astro_calculate_context', { requestJson: fraction }, JSON.parse(calculateJson(fraction)));
    assert.equal(fractionResult.errors[0].code, 'INVALID_INPUT');
    await assert.rejects(client.callTool({ name: 'astro_compress_payload', arguments: { payloadJson, options: { mode: 'focused', arbitrary: true } } }), /Invalid arguments/);
    calls++;
    assert.equal(stderr, '');
    console.log(JSON.stringify({ result: 'passed', transport: 'stdio', tools: listed.tools.length, calls, format: focused.data.format, engineVersion: focused.engineVersion }));
  } finally { await client.close(); }
})().catch(error => { console.error(stderr || error); process.exitCode = 1; });
