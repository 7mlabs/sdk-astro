const assert = require('node:assert/strict');
const { calculate, calculateJson, geometry, aspects, houses, points } = require('@7mlabs/astrology');
const request = { operation: 'chart', positions: [{ id: 'moon', longitude: 359, speed: 13 }, { id: 'sun', longitude: 1, speed: 1 }] };
const result = calculate(request);
assert.equal(result.data.aspects[0].orb, 2);
assert.equal(result.data.aspects[0].applying, true);
assert.equal(result.data.midpoints[0].longitude, 0);
assert.throws(() => calculate({ ...request, operation: 'natal' }), { code: 'INVALID_INPUT' });
assert.equal(JSON.parse(calculateJson('{')).errors[0].code, 'INVALID_INPUT');
assert.throws(() => calculateJson(42), TypeError);
for (let i = 0; i < 1000; i++) calculate(request);
console.log('Node installed package: geometry, errors, 1000 repeated calls OK');

const natalRequest = { operation: 'natal', utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 }, location: { latitude: 10.8231, longitude: 106.6297 } };
const natal = calculate(natalRequest);
assert.equal(natal.data.placements.length, 10);
assert.equal(natal.data.houses.length, 12);
assert.equal(natal.data.placements[0].sign, 'Capricorn');
for (let i = 0; i < 1000; i++) assert.deepEqual(calculate(natalRequest), natal);
console.log('Node installed package: natal, 1000 repeated calls OK');

const domainRequest = { ...natalRequest, operation: 'natalDomains' };
const domains = calculate(domainRequest);
assert.equal(Object.keys(domains.data.domains).length, 10);
assert.equal(domains.data.context.points.length, 26);
assert.equal(domains.data.context.relations.length, 325);
assert.equal(domains.data.chartKind, 'individualNatal');
assert.equal(domains.data.subjectCount, 1);
assert.equal(domains.data.context.houseRulerRelations.length, 66);
assert.equal(domains.data.context.advanced.bodyStates.length, 10);
assert.equal(domains.data.context.advanced.dispositorChains.length, 10);
assert.equal(domains.data.profileCatalog.length, 10);
assert.equal(Object.keys(domains.data.customDomains).length, 0);
assert.equal(Object.values(domains.data.domains).reduce((sum, v) => sum + v.report.sections.length, 0), 30);
for (const domain of Object.values(domains.data.domains)) {
  const report = domain.report;
  assert.equal(report.chartKind, 'individualNatal');
  assert.ok(domain.houses.every(house => report.focusHouseIds.includes(house.id)));
  assert.ok(report.houses.every(house => house.roles.length > 0 && house.evidence.length > 0));
  assert.ok(report.aspects.every(aspect => report.pointIds.includes(aspect.point1) && report.pointIds.includes(aspect.point2)));
}
for (let i = 0; i < 20; i++) assert.deepEqual(calculate(domainRequest), domains);
console.log('Node installed package: 10 individual reports, advanced facts, 325 relations, 20 repeated calls OK');

const customRequest = { ...natalRequest, operation: 'natalDomains', domains: [],
  customProfiles: [{ id: 'personalGrowth', houses: [1, 9], bodies: ['sun', 'moon', 'mercury'],
    angles: ['ascendant'], sections: [{ id: 'study', houses: [3, 9], bodies: ['mercury'] }] }] };
const custom = calculate(customRequest);
assert.equal(Object.keys(custom.data.domains).length, 0);
assert.equal(custom.data.customDomains.personalGrowth.origin, 'custom');
assert.equal(custom.data.customDomains.personalGrowth.definition.version, '1.0');
assert.equal(custom.data.customDomains.personalGrowth.report.sections[0].id, 'study');
assert.throws(() => calculate({ ...customRequest, customProfiles: [{ id: 'identity', houses: [1] }] }), { code: 'INVALID_INPUT' });
for (let i = 0; i < 20; i++) assert.deepEqual(calculate(customRequest), custom);
console.log('Node installed package: custom-only profile, section, validation, 20 repeated calls OK');

const coupleRequest = { operation: 'couple', personA: {
  utc: natalRequest.utc, location: natalRequest.location, houseSystem: 'placidus'
}, personB: {
  utc: { year: 1998, month: 6, day: 15, hour: 6, minute: 30 },
  location: { latitude: 21.0278, longitude: 105.8342 }, houseSystem: 'wholeSign'
} };
const couple = calculate(coupleRequest);
assert.equal(couple.data.chartKind, 'coupleSynastry');
assert.equal(couple.data.subjectCount, 2);
assert.equal(Object.keys(couple.data.domains).length, 6);
assert.equal(couple.data.profileCatalog.length, 6);
assert.equal(Object.values(couple.data.domains).reduce((sum, view) => sum + view.report.sections.length, 0), 18);
assert.equal(couple.data.subjects.A.calculation.houseSystem, 'placidus');
assert.equal(couple.data.subjects.B.calculation.houseSystem, 'wholeSign');
assert.equal(couple.data.subjects.A.context.relations.length, 325);
assert.equal(couple.data.subjects.B.context.relations.length, 325);
const cross = couple.data.context;
assert.equal(cross.points.length, 52);
const pointIds = new Set(cross.points.map(point => point.id));
assert.equal(pointIds.size, 52);
assert.ok(cross.points.every(point => point.id === `${point.chartId}:${point.localId}`));
assert.equal(cross.relations.length, 676);
const relationIds = new Set(cross.relations.map(relation => relation.id));
assert.equal(relationIds.size, 676);
assert.ok(cross.relations.every(relation => relation.point1.startsWith('A:') && relation.point2.startsWith('B:')
  && pointIds.has(relation.point1) && pointIds.has(relation.point2)));
assert.ok(cross.aspects.length > 0);
assert.ok(cross.aspects.every(aspect => aspect.applying === null && relationIds.has(aspect.relationId)));
assert.equal(cross.overlaysAtoB.length, 10);
assert.equal(cross.overlaysBtoA.length, 10);
for (const [source, target, overlays] of [['A', 'B', cross.overlaysAtoB], ['B', 'A', cross.overlaysBtoA]]) {
  assert.ok(overlays.every(overlay => overlay.sourceChartId === source && overlay.targetChartId === target
    && overlay.pointId.startsWith(`${source}:`) && overlay.targetHouseId === `${target}:H${overlay.targetHouseNumber}`
    && relationIds.has(overlay.targetCuspRelationId) && relationIds.has(overlay.targetRulerRelationId)));
}
assert.equal(cross.houseRulerRelations.length, 144);
assert.ok(cross.houseRulerRelations.every(relation => relation.house1.startsWith('A:') && relation.house2.startsWith('B:')
  && relationIds.has(relation.relationId)));
const disabledCouple = calculate({ ...coupleRequest, domains: ['communication'], aspectRules: [] });
assert.equal(disabledCouple.data.context.aspects.length, 0);
assert.equal(disabledCouple.data.context.relations.length, 676);
assert.equal(disabledCouple.data.context.overlaysAtoB.length, 10);
assert.throws(() => calculate({ ...coupleRequest, personB: {
  ...coupleRequest.personB, utc: { year: 1998, month: 2, day: 30, hour: 6, minute: 30 }
} }), { code: 'INVALID_INPUT' });
const coupleRequests = [{ ...coupleRequest, domains: ['communication'] }, {
  ...coupleRequest, personA: { ...coupleRequest.personA, houseSystem: 'wholeSign' },
  personB: { ...coupleRequest.personB, houseSystem: 'placidus' },
  domains: [], rulership: 'modern', aspectRules: [{ angle: 0, maxOrb: 3 }, { angle: 90, maxOrb: 3 }],
  customProfiles: [{ id: 'sharedLearning', houses: [3, 9], bodies: ['mercury', 'jupiter'],
    sections: [{ id: 'ideas', houses: [3], bodies: ['mercury'] }] }]
}];
const coupleResults = coupleRequests.map(calculate);
assert.equal(coupleResults[1].data.customDomains.sharedLearning.report.sections[0].id, 'ideas');
for (let i = 0; i < 20; i++) assert.deepEqual(calculate(coupleRequests[i % 2]), coupleResults[i % 2]);
console.log('Node installed package: couple, qualified references, 676 relations, 20 overlays, 144 house-ruler pairs, validation, 20 repeated calls OK');

const compositeRequest = {
  "operation": "composite",
  "personA": {
    "utc": {
      "year": 2000,
      "month": 1,
      "day": 1,
      "hour": 12,
      "minute": 0
    },
    "location": {
      "latitude": 10.8231,
      "longitude": 106.6297
    },
    "houseSystem": "placidus"
  },
  "personB": {
    "utc": {
      "year": 1998,
      "month": 6,
      "day": 15,
      "hour": 6,
      "minute": 30
    },
    "location": {
      "latitude": 21.0278,
      "longitude": 105.8342
    },
    "houseSystem": "placidus"
  },
  "rulership": "traditional",
  "aspectPreset": "extended",
  "houseMethod": "midpoint",
  "antipodalPolicy": "error"
};
const compositeResult = calculate(compositeRequest);
const composite = compositeResult.data.composite;
assert.equal(compositeResult.data.chartCount, 3);
assert.equal(composite.id, 'C');
assert.equal(composite.chartKind, 'midpointComposite');
assert.equal(Object.keys(composite.domains).length, 10);
assert.equal(Object.values(composite.domains).reduce((n, v) => n + v.report.sections.length, 0), 30);
assert.equal(composite.context.points.length, 26);
assert.equal(composite.context.relations.length, 325);
assert.equal(composite.context.houseRulerRelations.length, 66);
assert.equal(composite.provenance.points.length, 26);
assert.ok(!('utc' in composite.chart) && !('location' in composite.chart));
assert.ok(!('julianDayTt' in compositeResult.calculation));
assert.ok(composite.chart.placements.every(p => p.speed === null && p.isRetrograde === null && !('latitude' in p) && !('distanceAu' in p)));
assert.ok(composite.context.aspects.every(a => a.applying === null));
assert.ok(composite.context.advanced.bodyStates.every(b => b.motion === 'notApplicable'));
assert.ok(Object.values(composite.domains).every(v => v.report.chartKind === 'midpointComposite'));
const compositeCoupleRequest = { operation: 'couple', personA: compositeRequest.personA, personB: compositeRequest.personB,
  domains: ['communication'], aspectPreset: compositeRequest.aspectPreset, rulership: compositeRequest.rulership,
  composite: { houseMethod: compositeRequest.houseMethod, antipodalPolicy: compositeRequest.antipodalPolicy } };
const combinedComposite = calculate(compositeCoupleRequest);
assert.equal(combinedComposite.data.chartCount, 3);
assert.deepEqual(combinedComposite.data.composite, composite);
assert.deepEqual(combinedComposite.calculation.composite, compositeResult.calculation);
assert.deepEqual(combinedComposite.data.subjects.A.natal, compositeResult.data.subjects.A.natal);
assert.deepEqual(combinedComposite.data.subjects.B.natal, compositeResult.data.subjects.B.natal);
assert.throws(() => calculate({ ...compositeCoupleRequest, composite: { domains: ['attraction'] } }), { code: 'INVALID_INPUT' });
const disabledCompositeRequest = { ...compositeRequest, houseMethod: 'wholeSignFromMidpointAscendant',
  domains: [], rulership: 'modern', customProfiles: [{ id: 'sharedLearning', houses: [3, 9], bodies: ['mercury', 'jupiter'] }], aspectRules: [] };
delete disabledCompositeRequest.aspectPreset;
const disabledCompositeResult = calculate(disabledCompositeRequest);
assert.equal(disabledCompositeResult.data.composite.context.aspects.length, 0);
assert.equal(disabledCompositeResult.data.composite.context.relations.length, 325);
assert.equal(disabledCompositeResult.data.composite.customDomains.sharedLearning.origin, 'custom');
for (let i = 0; i < 20; i++) {
  const input = i % 2 ? disabledCompositeRequest : compositeCoupleRequest;
  const expected = i % 2 ? disabledCompositeResult : combinedComposite;
  assert.deepEqual(calculate(input), expected);
}
console.log('Node installed package: composite 10/30 reports, symbolic facts, combined reuse, 20 repeated calls OK');

const forecastRequest = {
  "operation": "forecast",
  "birth": {
    "utc": {
      "year": 2000,
      "month": 1,
      "day": 1,
      "hour": 12,
      "minute": 0
    },
    "location": {
      "latitude": 10.8231,
      "longitude": 106.6297
    },
    "houseSystem": "placidus"
  },
  "period": {
    "kind": "day",
    "year": 2026,
    "month": 3,
    "day": 3,
    "utcOffsetMinutes": 420
  }
};
const forecastResult = calculate(forecastRequest);
const forecast = forecastResult.data;
assert.equal(forecast.chartKind, 'individualForecast');
assert.equal(forecast.subject.id, 'N');
assert.equal(forecast.snapshot.relations.length, 260);
assert.equal(forecast.snapshot.houseOverlays.length, 10);
assert.equal(Object.keys(forecast.domains).length, 10);
assert.equal(Object.values(forecast.domains).reduce((n,v) => n + v.sections.length, 0), 30);
assert.ok(forecast.events.some(e => e.type === 'lunarEclipse'));
assert.ok(forecast.events.some(e => e.type === 'natalTransit'));
assert.equal(forecast.overview.eventCount, forecast.events.length);
assert.ok(Object.values(forecast.domains).every(v => v.messageContext.kind === 'dailyMessage' && v.messageContext.narrative === null));
const forecastPointIds = new Set(forecast.subject.context.points.map(p => `N:${p.id}`));
for (const event of forecast.events) {
  assert.ok(event.personalImpact.affectedNatalPointIds.every(id => forecastPointIds.has(id)));
  assert.ok(event.personalImpact.houseOverlays.every(o => forecastPointIds.has(o.natalHouseId) && forecastPointIds.has(o.natalRulerPointId)));
  if (event.type === 'natalTransit') assert.equal(event.personalImpact.primaryNatalPointId, event.details.targetPointId);
}
const quietForecastRequest = { ...forecastRequest, eventTypes: [], includeNatalTransits: false, aspectRules: [], domains: [],
  bodies: ['mercury'], rulership: 'modern', customProfiles: [{ id: 'quietFocus', houses: [3,9], bodies: ['mercury','jupiter'] }] };
const quietForecastResult = calculate(quietForecastRequest);
assert.equal(quietForecastResult.data.events.length, 0);
assert.equal(quietForecastResult.data.snapshot.aspects.length, 0);
assert.equal(quietForecastResult.data.snapshot.relations.length, 26);
assert.throws(() => calculate({ ...forecastRequest, period: { kind: 'day', year: 2026, month: 2, day: 30 } }), { code: 'INVALID_INPUT' });
for (let i=0;i<20;i++) assert.deepEqual(calculate(i%2 ? quietForecastRequest : forecastRequest), i%2 ? quietForecastResult : forecastResult);
console.log('Node installed package: daily forecast, 10/30 views, exact natal events, eclipse impact, 20 repeated calls OK');

const queryCalls = [
  ['geometry', 'normalize', { longitude: -10 }],
  ['geometry', 'separation', { longitude1: 359, longitude2: 1 }],
  ['geometry', 'midpoint', { longitude1: 359, longitude2: 1 }],
  ['aspects', 'between', { positions: [{ id: 'moving', longitude: 350, speed: 1 }, { id: 'fixed', longitude: 28, speed: 4 }], rule: { angle: 37.5, maxOrb: 1 }, motionMode: 'fixedSecond' }],
  ['houses', 'locate', { longitude: 359, houseCusps: Array.from({length:12}, (_,i) => i*30) }],
  ['houses', 'inspect', { birth: forecastRequest.birth, houseNumbers: [2,8], rulership: 'modern' }],
  ['points', 'inspect', { birth: forecastRequest.birth, pointIds: ['sun','midheaven','H7'], aspectRules: [] }],
  ['aspects', 'inspect', { birth: forecastRequest.birth, pointIds: ['sun','moon'], aspectRules: [{angle:37.5,maxOrb:15}] }],
];
const queryGroups = { geometry, aspects, houses, points };
const queryExpected = queryCalls.map(([group, action, options]) => calculate({ operation: 'query', group, action, ...options }));
for (const [i, [group, action, options]] of queryCalls.entries()) assert.deepEqual(queryGroups[group][action](options), queryExpected[i]);
for (const options of [null, [], 1, 'bad', {operation:'natal'}, {group:'houses'}, {action:'inspect'}])
  assert.throws(() => geometry.normalize(options), TypeError);
assert.throws(() => houses.inspect({birth: forecastRequest.birth, houseNumbers:[13]}), { code:'INVALID_INPUT' });
for (let i=0; i<20; i++) { const [group, action, options] = queryCalls[i%queryCalls.length]; assert.deepEqual(queryGroups[group][action](options), queryExpected[i%queryCalls.length]); }
console.log('Node installed package: all eight grouped query methods, invalid routing/options and 20 repeated calls OK');
