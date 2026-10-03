'use strict';
const assert = require('node:assert/strict');
const http = require('node:http');
const { createStandaloneServer } = require('./host.cjs');
const { OPERATIONS, resolveEngineModule } = require('./local-http.cjs');
const { calculateJson } = require(resolveEngineModule());
const birth = { utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 }, houseSystem: 'placidus' };
const personB = { utc: { year: 1998, month: 6, day: 15, hour: 6, minute: 30 },
  location: { latitude: 21.0278, longitude: 105.8342 }, houseSystem: 'wholeSign' };
const positions = [{ id: 'a', longitude: 350, speed: 1 }, { id: 'b', longitude: 10, speed: 13 }];
const requests = [
  { operation: 'chart', positions },
  { operation: 'harmonic', positions, harmonic: 3 },
  { operation: 'synastry', positions, otherPositions: [{ id: 'c', longitude: 130 }] },
  { operation: 'natal', ...birth },
  { operation: 'natalDomains', ...birth, domains: ['career'] },
  { operation: 'couple', personA: birth, personB, domains: ['communication'] },
  { operation: 'composite', personA: birth, personB, houseMethod: 'wholeSignFromMidpointAscendant', domains: ['career'] },
  { operation: 'events', period: { kind: 'day', year: 2026, month: 3, day: 3 } },
  { operation: 'forecast', birth, period: { kind: 'day', year: 2026, month: 3, day: 3 }, domains: ['career'] },
  { operation: 'query', group: 'geometry', action: 'normalize', longitude: -10 }
];
function call(url, route = '/api/astro/status', options = {}) {
  const endpoint = new URL(route, url);
  const body = options.body === undefined ? null : Buffer.isBuffer(options.body) ? options.body : Buffer.from(options.body);
  const headers = { Connection: 'close', ...options.headers };
  if (body !== null && !headers['Transfer-Encoding'] && !headers['Content-Length']) headers['Content-Length'] = String(body.length);
  return new Promise((resolve, reject) => {
    const request = http.request(endpoint, { method: options.method || 'GET', headers }, response => {
      const chunks = [];
      response.on('data', chunk => chunks.push(chunk));
      response.on('end', () => {
        const raw = Buffer.concat(chunks).toString('utf8');
        let json; try { json = JSON.parse(raw); } catch { json = null; }
        resolve({ status: response.statusCode, headers: response.headers, raw, json });
      });
    });
    request.on('error', reject); request.end(body);
  });
}
const post = (url, body, headers = {}) => call(url, '/api/astro/calculate', {
  method: 'POST', body: typeof body === 'string' || Buffer.isBuffer(body) ? body : JSON.stringify(body),
  headers: { 'Content-Type': 'application/json', Origin: url, ...headers }
});

(async () => {
  let checks = 0;
  const host = createStandaloneServer();
  const url = await host.start(0);
  try {
    const status = await call(url);
    assert.equal(status.status, 200); assert.equal(status.json.available, true);
    assert.deepEqual(status.json.operations, OPERATIONS);
    assert.equal(status.json.runtime, 'localNodeWorker'); checks++;
    const page = await call(url, '/');
    assert.equal(page.status, 200);
    assert.match(page.raw, /\/ui\/example\.mjs/);
    assert.match(page.headers['content-security-policy'], /script-src 'self'/); checks++;
    for (const filename of ['index.mjs', 'styles.css', 'example.mjs', 'example.css', 'data.mjs', 'dom.mjs', 'wheel.mjs']) {
      const asset = await call(url, `/ui/${filename}`);
      assert.equal(asset.status, 200, filename);
      assert.match(asset.headers['content-type'], filename.endsWith('.css') ? /^text\/css/ : /^text\/javascript/); checks++;
    }
    for (const request of requests) {
      const result = await post(url, request);
      assert.equal(result.status, 200);
      assert.deepEqual(result.json, JSON.parse(calculateJson(JSON.stringify(request))), request.operation);
      assert.deepEqual(result.json.errors, [], request.operation); checks++;
    }
    for (const raw of [
      '{"operation":"query","group":"houses","action":"inspect","birth":{"utc":{"year":2000.0000000000000001,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":0,"longitude":0}}}',
      '{invalid JSON',
      JSON.stringify({ operation: 'query', group: 'geometry', action: 'midpoint', longitude1: 0, longitude2: 180 })
    ]) {
      const result = await post(url, raw);
      assert.equal(result.status, 200);
      assert.deepEqual(result.json, JSON.parse(calculateJson(raw)));
      assert.equal(result.json.data, null); checks++;
    }
    const queued = requests.slice(0, 3).concat(requests[9]);
    const results = await Promise.all(queued.map(request => post(url, request)));
    results.forEach((result, i) => {
      assert.deepEqual(result.json, JSON.parse(calculateJson(JSON.stringify(queued[i])))); checks++;
    });
    for (const [route, options, expectedStatus, code] of [
      ['/api/astro/status', { headers: { Host: 'attacker.example' } }, 403, 'FORBIDDEN_HOST'],
      ['/api/astro/status', { headers: { Origin: 'https://attacker.example' } }, 403, 'FORBIDDEN_ORIGIN'],
      ['/api/astro/status', { headers: { Origin: 'null' } }, 403, 'FORBIDDEN_ORIGIN'],
      ['/api/astro/status', { headers: { 'Sec-Fetch-Site': 'cross-site' } }, 403, 'FORBIDDEN_ORIGIN'],
      ['/api/astro/status', { headers: { 'Sec-Fetch-Site': 'same-site' } }, 403, 'FORBIDDEN_ORIGIN'],
      ['/api/astro/status', { method: 'POST' }, 405, 'METHOD_NOT_ALLOWED'],
      ['/api/astro/calculate', { method: 'GET' }, 405, 'METHOD_NOT_ALLOWED'],
      ['/api/astro/calculate', { method: 'OPTIONS', headers: { Origin: 'https://attacker.example' } }, 403, 'FORBIDDEN_ORIGIN'],
      ['/api/astro/calculate', { method: 'POST', body: '{}', headers: { 'Content-Type': 'text/plain' } }, 415, 'JSON_REQUIRED'],
      ['/api/astro/missing', {}, 404, 'UNKNOWN_ROUTE'],
      ['/api/astro/calculate', { method: 'POST', body: Buffer.from([0xc3, 0x28]), headers: { 'Content-Type': 'application/json' } }, 400, 'INVALID_ENCODING']
    ]) {
      const result = await call(url, route, options);
      assert.equal(result.status, expectedStatus); assert.equal(result.json.error.code, code);
      assert.equal(result.headers['access-control-allow-origin'], undefined); checks++;
    }
    const allowed = await post(url, requests[9], { 'Sec-Fetch-Site': 'same-origin' });
    assert.equal(allowed.status, 200); checks++;
    const traversal = await call(url, '/ui/..%2fbridge.cjs');
    assert.equal(traversal.status, 404); checks++;
  } finally { await host.close(); }

  const bounded = createStandaloneServer({ maxPendingRequests: 1, maxInputBytes: 128 });
  const boundedUrl = await bounded.start(0);
  try {
    await bounded.engine.ready;
    for (const headers of [{}, { 'Transfer-Encoding': 'chunked' }]) {
      const result = await post(boundedUrl, ' '.repeat(129), headers);
      assert.equal(result.status, 413); assert.equal(result.json.error.code, 'INPUT_TOO_LARGE'); checks++;
    }
    const raw = JSON.stringify(requests[9]);
    let finishSlow;
    const slow = new Promise((resolve, reject) => {
      const request = http.request(new URL('/api/astro/calculate', boundedUrl), { method: 'POST',
        headers: { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(raw), Connection: 'close' } }, response => {
        let result = ''; response.on('data', data => { result += data; });
        response.on('end', () => resolve({ status: response.statusCode, json: JSON.parse(result) }));
      });
      request.on('error', reject); request.write(raw.slice(0, 12));
      finishSlow = () => request.end(raw.slice(12));
    });
    let occupied = false;
    for (let i = 0; i < 20; i++) {
      if ((await call(boundedUrl)).json.inFlightRequests === 1) { occupied = true; break; }
      await new Promise(resolve => setTimeout(resolve, 10));
    }
    assert.equal(occupied, true);
    const rejected = await post(boundedUrl, requests[9]);
    assert.equal(rejected.status, 429); assert.equal(rejected.json.error.code, 'RUNTIME_BUSY'); checks++;
    finishSlow();
    assert.equal((await slow).status, 200);
    assert.equal((await call(boundedUrl)).json.inFlightRequests, 0); checks++;
  } finally { await bounded.close(); }
  const unavailable = createStandaloneServer({ engineModule: '/does-not-exist/astrology-native-sdk' });
  const unavailableUrl = await unavailable.start(0);
  try {
    const status = await call(unavailableUrl);
    assert.equal(status.status, 503); assert.equal(status.json.available, false);
    const result = await post(unavailableUrl, requests[9]);
    assert.equal(result.status, 503); assert.equal(result.json.error.code, 'ENGINE_UNAVAILABLE'); checks++;
  } finally { await unavailable.close(); }
  console.log(JSON.stringify({ result: 'passed', runtime: 'loopback HTTP to native worker',
    checks, operations: OPERATIONS.length, engineVersion: JSON.parse(calculateJson(JSON.stringify(requests[9]))).engineVersion }));
})().catch(error => { console.error(error); process.exitCode = 1; });
