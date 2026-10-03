'use strict';
const assert = require('node:assert/strict');
const { resolveEngineModule } = require('./local-http.cjs');
const engineModule = resolveEngineModule();
const { calculateJson } = require(engineModule);
const { createEngineBridge } = require('./bridge.cjs');
const bridge = createEngineBridge({ engineModule });
const birth = {
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 }
};
const expected = input => JSON.parse(calculateJson(typeof input === 'string' ? input : JSON.stringify(input)));
(async () => {
  let checks = 0;
  let version;
  try {
    const first = { operation: 'query', group: 'geometry', action: 'normalize', longitude: -10 };
    const result = await bridge.asyncCalculate(first);
    assert.deepEqual(result, expected(first));
    assert.equal(result.data.longitude, 350);
    version = result.engineVersion;
    checks++;
    const concurrent = [
      { operation: 'query', group: 'geometry', action: 'normalize', longitude: 725 },
      { operation: 'query', group: 'geometry', action: 'separation', longitude1: 350, longitude2: 10 },
      { operation: 'query', group: 'geometry', action: 'midpoint', longitude1: 350, longitude2: 10 },
      { operation: 'query', group: 'points', action: 'inspect', birth, pointIds: ['sun', 'ascendant', 'H1'] }
    ];
    const results = await Promise.all(concurrent.map(request => bridge.asyncCalculate(request)));
    results.forEach((result, index) => assert.deepEqual(result, expected(concurrent[index])));
    checks += concurrent.length;
    for (const invalid of [
      { operation: 'query', group: 'points', action: 'inspect', birth: { ...birth, utc: { ...birth.utc, month: 2, day: 31 } } },
      '{"operation":"query","group":"houses","action":"inspect","birth":{"utc":{"year":2000.0000000000000001,"month":1,"day":1,"hour":12,"minute":0},"location":{"latitude":0,"longitude":0}}}',
      '{invalid JSON'
    ]) {
      const errorResult = await bridge.asyncCalculate(invalid);
      assert.deepEqual(errorResult, expected(invalid));
      assert.equal(errorResult.data, null);
      assert.equal(errorResult.errors[0].code, 'INVALID_INPUT');
      checks++;
    }
    const circular = {}; circular.self = circular;
    await assert.rejects(bridge.asyncCalculate(circular), /circular/i);
    checks++;
  } finally { await bridge.close(); }
  await assert.rejects(bridge.asyncCalculate({ operation: 'query' }), /closed/);
  checks++;
  const closingBridge = createEngineBridge({ engineModule });
  const pending = closingBridge.asyncCalculate({ operation: 'query', group: 'geometry', action: 'normalize', longitude: 0 });
  const rejected = assert.rejects(pending, /closed/);
  await closingBridge.close();
  await rejected;
  checks++;
  console.log(JSON.stringify({ result: 'passed', runtime: 'Node worker_threads native host', checks, concurrentCalls: 4, engineVersion: version }));
})().catch(error => { console.error(error); process.exitCode = 1; });
