'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { Client } = require('@modelcontextprotocol/sdk/client/index.js');
const { StdioClientTransport, getDefaultEnvironment } = require('@modelcontextprotocol/sdk/client/stdio.js');
const { calculateJson } = require(process.env.ASTRO_ENGINE_MODULE || '@7mlabs/astrology');

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
const client = new Client({ name: 'astrology-mcp-smoke', version: '1.0.0' });
let callCount = 0;
async function call(name, args, nativeRequest) {
  const result = await client.callTool({ name, arguments: args }, undefined, { timeout: 60000 });
  const expected = JSON.parse(calculateJson(typeof nativeRequest === 'string' ? nativeRequest : JSON.stringify(nativeRequest)));
  assert.deepEqual(result.structuredContent, expected);
  assert.deepEqual(JSON.parse(result.content[0].text), expected);
  assert.equal(result.isError, expected.errors.length > 0);
  callCount++;
  return expected;
}
const birth = {
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 }, houseSystem: 'wholeSign'
};
async function grouped(group, args) {
  return call(`astro_${group}`, args, { ...args, operation: 'query', group });
}
(async () => {
  try {
    await client.connect(transport);
    const listed = await client.listTools();
    assert.deepEqual(listed.tools.map(tool => tool.name), ['astro_geometry', 'astro_aspects', 'astro_houses', 'astro_points', 'astro_calculate', 'astro_compress_payload', 'astro_calculate_context']);
    for (const tool of listed.tools) {
      assert.equal(tool.inputSchema.type, 'object');
      assert.equal(tool.annotations.readOnlyHint, true);
      assert.equal(tool.annotations.openWorldHint, false);
    }
    const normalized = await grouped('geometry', { action: 'normalize', longitude: -10 });
    assert.equal(normalized.data.longitude, 350);
    const separated = await grouped('geometry', { action: 'separation', longitude1: 350, longitude2: 10 });
    assert.equal(separated.data.separation, 20);
    const middle = await grouped('geometry', { action: 'midpoint', longitude1: 350, longitude2: 10 });
    assert.equal(middle.data.longitude, 0);
    await grouped('aspects', { action: 'between', positions: [{ id: 'sun', longitude: 0, speed: 1 }, { id: 'moon', longitude: 91, speed: 0.5 }], rule: { angle: 90, maxOrb: 8 }, motionMode: 'relative' });
    await grouped('aspects', { action: 'inspect', birth, pointIds: ['sun', 'moon'], aspectPreset: 'major' });
    const located = await grouped('houses', { action: 'locate', longitude: 0, houseCusps: Array.from({ length: 12 }, (_, i) => i * 30) });
    assert.equal(located.data.house.number, 1);
    await grouped('houses', { action: 'inspect', birth, houseNumbers: [1, 7], rulership: 'modern', aspectRules: [] });
    await grouped('points', { action: 'inspect', birth, pointIds: ['sun', 'ascendant', 'H1'] });
    const badBirth = { ...birth, utc: { ...birth.utc, month: 2, day: 31 } };
    const calendar = await grouped('points', { action: 'inspect', birth: badBirth });
    assert.equal(calendar.errors[0].code, 'INVALID_INPUT');
    assert.equal(calendar.data, null);
    const antipodal = await grouped('geometry', { action: 'midpoint', longitude1: 0, longitude2: 180 });
    assert.equal(antipodal.errors[0].code, 'CALCULATION_FAILED');
    const rawFraction = '{"operation":"query","group":"points","action":"inspect","birth":{"utc":{"year":2000.0000000000000001,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":0,"longitude":0}}}';
    const fraction = await call('astro_calculate', { requestJson: rawFraction }, rawFraction);
    assert.equal(fraction.errors[0].code, 'INVALID_INPUT');
    for (const file of ['natal-request.json', 'custom-couple-request.json', 'forecast-day-request.json']) {
      const rawRequest = fs.readFileSync(path.resolve(__dirname, '..', file), 'utf8');
      const result = await call('astro_calculate', { requestJson: rawRequest }, rawRequest);
      assert.equal(result.errors.length, 0, file);
    }
    await assert.rejects(client.callTool({ name: 'astro_geometry', arguments: { action: 'normalize', longitude: 0, group: 'points' } }), /Invalid arguments/);
    await assert.rejects(client.callTool({ name: 'unknown_tool', arguments: {} }), /Unknown tool/);
    callCount += 2;
    assert.equal(stderr, '');
    console.log(JSON.stringify({ result: 'passed', transport: 'stdio', sdk: '1.29.0', tools: listed.tools.length, calls: callCount, engineVersion: normalized.engineVersion }));
  } finally { await client.close(); }
})().catch(error => { console.error(stderr || error); process.exitCode = 1; });
