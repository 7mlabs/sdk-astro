#!/usr/bin/env node
// Development-only strict JSON Schema and semantic validation; no SDK runtime dependency.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Ajv2020 = require(process.env.AJV_2020_MODULE || 'ajv/dist/2020');
const files = process.argv.slice(2);
assert(files.length, 'Usage: node scripts/check-composite-schema.cjs <response.json> [more responses.json]');
const readSchema = name => JSON.parse(fs.readFileSync(path.join(__dirname, `../schemas/${name}.schema.json`), 'utf8'));
const ajv = new Ajv2020({ allErrors: true, strict: true });
for (const name of ['natal-domains-request', 'natal-domains-response', 'composite-request', 'composite-response', 'couple-request', 'couple-response']) ajv.addSchema(readSchema(name));
const validateRequest = ajv.getSchema('urn:7mlabs:astrology:composite-request:1.0');
const validateCoupleRequest = ajv.getSchema('urn:7mlabs:astrology:couple-request:1.0');
const validate = ajv.getSchema('urn:7mlabs:astrology:composite-response:1.0');
const validateCouple = ajv.getSchema('urn:7mlabs:astrology:couple-response:1.0');
const builtinIds = ['career', 'love', 'relationships', 'family', 'finance', 'identity', 'learning', 'creativity', 'innerLife', 'dailyLife'];
const bodyIds = ['sun', 'moon', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'uranus', 'neptune', 'pluto'];
const angleIds = ['ascendant', 'midheaven', 'descendant', 'imumCoeli'];
const birth = { utc: { year: 2000.0, month: 1, day: 1, hour: 12, minute: 0 }, location: { latitude: 10.8231, longitude: 106.6297 } };
const baseRequest = { operation: 'composite', personA: birth, personB: { ...birth, houseSystem: 'wholeSign' } };
const custom = { id: 'ourGrowth', houses: [1], bodies: ['sun'], sections: [{ id: 'learning', houses: [3, 9], bodies: ['mercury'] }] };
const optionCases = [
  ['defaults', true, {}], ['all ten profiles', true, { domains: builtinIds }],
  ['one profile', true, { domains: ['career'] }], ['custom only', true, { domains: [], customProfiles: [custom] }],
  ['midpoint houses', true, { houseMethod: 'midpoint' }], ['whole sign houses', true, { houseMethod: 'wholeSignFromMidpointAscendant' }],
  ['explicit antipodal error', true, { antipodalPolicy: 'error' }], ['lower antipodal branch', true, { antipodalPolicy: 'lowerLongitude' }],
  ['angle only custom', true, { customProfiles: [{ id: 'ourAngle', angles: ['ascendant'] }] }],
  ['integer-valued decimal house', true, { customProfiles: [{ id: 'ourHouse', houses: [1.0] }] }],
  ['null options house', false, { houseMethod: null }], ['unknown house method', false, { houseMethod: 'placidus' }],
  ['null antipodal policy', false, { antipodalPolicy: null }], ['unknown antipodal policy', false, { antipodalPolicy: 'average' }],
  ['unknown field', false, { prediction: true }], ['empty domains', false, { domains: [] }], ['null domains', false, { domains: null }],
  ['pair domain ID', false, { domains: ['attraction'] }], ['unknown domain', false, { domains: ['health'] }],
  ['duplicate domain', false, { domains: ['career', 'career'] }], ['empty custom profiles', false, { customProfiles: [] }],
  ['null custom profiles', false, { customProfiles: null }], ['empty selector', false, { customProfiles: [{ id: 'empty' }] }],
  ['reserved builtin ID', false, { customProfiles: [{ id: 'identity', houses: [1] }] }],
  ['prototype ID', false, { customProfiles: [{ id: 'constructor', houses: [1] }] }],
  ['invalid version', false, { customProfiles: [{ ...custom, version: '../1' }] }],
  ['non ASCII ID', false, { customProfiles: [{ ...custom, id: 'cặpĐôi' }] }],
  ['newline ID', false, { customProfiles: [{ ...custom, id: 'ourGrowth\n' }] }],
  ['fractional house', false, { customProfiles: [{ id: 'ourHouse', houses: [1.5] }] }],
  ['house out of range', false, { customProfiles: [{ id: 'ourHouse', houses: [13] }] }],
  ['duplicate body', false, { customProfiles: [{ id: 'ourBody', bodies: ['sun', 'sun'] }] }],
  ['unsupported body', false, { customProfiles: [{ id: 'ourBody', bodies: ['chiron'] }] }],
  ['invalid angle', false, { customProfiles: [{ id: 'ourAngle', angles: ['asc'] }] }],
  ['null sections', false, { customProfiles: [{ ...custom, sections: null }] }],
  ['section cannot replace parent', false, { customProfiles: [{ id: 'empty', sections: custom.sections }] }],
  ['empty section', false, { customProfiles: [{ ...custom, sections: [{ id: 'empty' }] }] }],
  ['unknown section field', false, { customProfiles: [{ ...custom, sections: [{ id: 'study', bodies: ['sun'], version: '1' }] }] }],
  ['nine sections', false, { customProfiles: [{ ...custom, sections: Array.from({ length: 9 }, (_, i) => ({ id: `section${i}`, houses: [1] })) }] }],
  ['nine profiles', false, { customProfiles: Array.from({ length: 9 }, (_, i) => ({ id: `profile${i}`, houses: [1] })) }],
];
let requestCaseCount = 0;
for (const [name, expected, options] of optionCases) {
  assert.equal(validateRequest({ ...baseRequest, ...options }), expected, `${name}: ${JSON.stringify(validateRequest.errors)}`);
  assert.equal(validateCoupleRequest({ operation: 'couple', personA: birth, personB: birth, composite: options }), expected,
    `couple ${name}: ${JSON.stringify(validateCoupleRequest.errors)}`);
  requestCaseCount += 2;
}
const standaloneCases = [
  ['missing A', false, { personA: undefined }], ['missing B', false, { personB: undefined }],
  ['null person', false, { personA: null }], ['nested operation', false, { personA: { ...birth, operation: 'natal' } }],
  ['unknown person field', false, { personA: { ...birth, name: 'A' } }],
  ['fractional year', false, { personA: { ...birth, utc: { ...birth.utc, year: 2000.5 } } }],
  ['string year', false, { personA: { ...birth, utc: { ...birth.utc, year: '2000' } } }],
  ['fractional second', true, { personA: { ...birth, utc: { ...birth.utc, second: 0.5 } } }],
  ['invalid year', false, { personA: { ...birth, utc: { ...birth.utc, year: 2400 } } }],
  ['latitude at pole', false, { personA: { ...birth, location: { latitude: 90, longitude: 0 } } }],
  ['unknown birth house system', false, { personA: { ...birth, houseSystem: 'equal' } }],
  ['empty rules', true, { aspectRules: [] }], ['major preset', true, { aspectPreset: 'major' }],
  ['conflicting rules', false, { aspectPreset: 'major', aspectRules: [] }], ['modern rulers', true, { rulership: 'modern' }],
  ['null rules', false, { aspectRules: null }], ['null rulers', false, { rulership: null }],
  ['invalid orb', false, { aspectRules: [{ angle: 60, maxOrb: 16 }] }],
  ['too many rules', false, { aspectRules: Array.from({ length: 17 }, (_, angle) => ({ angle, maxOrb: 1 })) }],
  ['third physical input', false, { personC: birth }], ['nested composite setting', false, { composite: {} }],
];
for (const [name, expected, options] of standaloneCases) {
  assert.equal(validateRequest({ ...baseRequest, ...options }), expected, `${name}: ${JSON.stringify(validateRequest.errors)}`);requestCaseCount++;
}
for (const options of [null, { aspectRules: [] }, { rulership: 'modern' }, { operation: 'composite' }, { personA: birth }, { composite: {} }]) {
  assert(!validateCoupleRequest({ operation: 'couple', personA: birth, personB: birth, composite: options })); requestCaseCount++;
}
const errorEnvelope = { schemaVersion: '1.0', engineVersion: '0.7.0-alpha.1', data: null, warnings: [],
  errors: [{ code: 'CALCULATION_FAILED', message: 'Ambiguous antipodal midpoint' }] };
assert(validate(errorEnvelope), JSON.stringify(validate.errors));assert(validateCouple(errorEnvelope), JSON.stringify(validateCouple.errors));
const norm = value => ((value % 360) + 360) % 360;
const delta = (b, a) => norm(b - a + 180) - 180;
const near = (a, b) => assert(Math.abs(a - b) <= 1e-8, `${a} != ${b}`);
const nearLongitude = (a, b) => near(delta(a, b), 0);
function midpoint(a, b, policy, tolerance) {
  const d = delta(b, a); const antipodal = Math.abs(Math.abs(d) - 180) <= tolerance;
  if (antipodal) { assert.equal(policy, 'lowerLongitude');return Math.min(norm(a + d / 2), norm(a + d / 2 + 180)); }
  return norm(a + d / 2);
}
nearLongitude(midpoint(350, 10, 'error', 1e-10), 0);
nearLongitude(midpoint(10, 350, 'error', 1e-10), 0);
assert.throws(() => midpoint(0, 180, 'error', 1e-10));
nearLongitude(midpoint(350, 170, 'lowerLongitude', 1e-10), 80);
function mapOf(items, key, label) { const result = new Map(items.map(item => [item[key], item])); assert.equal(result.size, items.length, `Duplicate ${label}`);return result; }
const sameSet = (a, b) => assert.deepEqual([...a].sort(), [...b].sort());
const contains = (map, ids, label) => { sameSet(ids, new Set(ids));for (const id of ids) assert(map.has(id), `Missing ${label}: ${id}`); };
const subset = (value, fields) => Object.fromEntries(fields.map(field => [field, value[field]]));
function houseFor(longitude, cusps) {
  for (let i = 0; i < 12; i++) if (norm(longitude - cusps[i]) < norm(cusps[(i + 1) % 12] - cusps[i])) return i + 1;
  throw new Error('No house');
}
const signs = ['Aries', 'Taurus', 'Gemini', 'Cancer', 'Leo', 'Virgo', 'Libra', 'Scorpio', 'Sagittarius', 'Capricorn', 'Aquarius', 'Pisces'];
const traditional = ['mars', 'venus', 'mercury', 'moon', 'sun', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'saturn', 'jupiter'];
const modern = ['mars', 'venus', 'mercury', 'moon', 'sun', 'mercury', 'venus', 'pluto', 'jupiter', 'saturn', 'uranus', 'neptune'];
function checkComposite(response) {
  const { data } = response; const c = data.composite;
  const calculation = data.chartKind === 'coupleSynastry' ? response.calculation.composite : response.calculation;
  assert(c && calculation);assert.equal(data.chartCount, 3);
  assert.equal(calculation.customProfileCount, Object.keys(c.customDomains).length);
  const sourcePoints = {};
  for (const chartId of ['A', 'B']) {
    const source = data.subjects[chartId];assert.equal(source.id, chartId);
    const meta = source.calculation;const sourceMetadata = calculation.sourceCalculations[chartId];
    assert.deepEqual(sourceMetadata, subset(meta, ['julianDayTt', 'julianDayUt1', 'houseSystem']));
    assert.equal(c.provenance.sourceHouseSystems[chartId], meta.houseSystem);
    sourcePoints[chartId] = new Map([...source.natal.placements, ...source.natal.angles,
      ...source.natal.houses.map(house => ({ ...house, id: `H${house.number}` }))].map(point => [point.id, point]));
  }
  for (const field of ['method', 'houseMethod', 'antipodalPolicy', 'toleranceDegrees']) assert.equal(calculation[field], c.provenance[field]);
  const points = mapOf(c.context.points, 'id', 'C point');const bodies = mapOf(c.chart.placements, 'id', 'C body');const angles = mapOf(c.chart.angles, 'id', 'C angle');
  sameSet(bodies.keys(), bodyIds);sameSet(angles.keys(), angleIds);
  const houses = mapOf(c.context.houses, 'id', 'C house');const provenance = mapOf(c.provenance.points, 'id', 'C provenance');sameSet(provenance.keys(), points.keys());
  for (const point of points.values()) {
    const p = provenance.get(point.id);assert.equal(p.kind, point.kind);
    const a = sourcePoints.A.get(point.id);const b = sourcePoints.B.get(point.id);assert(a && b);
    assert.deepEqual(p.sourcePointIds, [`A:${point.id}`, `B:${point.id}`]);
    assert.deepEqual(p.sourceLongitudes, [a.longitude, b.longitude]);nearLongitude(p.longitude, point.longitude);
    const antipodal = Math.abs(Math.abs(delta(b.longitude, a.longitude)) - 180) <= c.provenance.toleranceDegrees;
    assert.equal(p.antipodal, antipodal);
    if (point.id === 'descendant' || point.id === 'imumCoeli') {
      const parent = point.id === 'descendant' ? 'ascendant' : 'midheaven';
      assert.equal(p.construction, point.id === 'descendant' ? 'oppositeCompositeAscendant' : 'oppositeCompositeMidheaven');
      assert.equal(p.resolution, 'notRequiredForDerivedPoint');nearLongitude(point.longitude, angles.get(parent).longitude + 180);
    } else if (point.kind === 'houseCusp' && c.provenance.houseMethod === 'wholeSignFromMidpointAscendant') {
      assert.equal(p.construction, 'wholeSignFromMidpointAscendant');assert.equal(p.resolution, 'notRequiredForDerivedPoint');
      nearLongitude(point.longitude, Math.floor(angles.get('ascendant').longitude / 30) * 30 + (point.houseNumber - 1) * 30);
    } else {
      assert.equal(p.construction, 'shortestArcMidpoint');assert.equal(p.resolution, antipodal ? 'lowerLongitude' : 'unambiguous');
      nearLongitude(point.longitude, midpoint(a.longitude, b.longitude, c.provenance.antipodalPolicy, c.provenance.toleranceDegrees));
    }
    const signIndex = Math.floor(point.longitude / 30);assert.equal(point.signIndex, signIndex);assert.equal(point.sign, signs[signIndex]);near(point.degreeInSign, point.longitude - 30 * signIndex);
    assert.equal(point.speed, null);
    if (point.kind === 'body' || point.kind === 'angle') assert.deepEqual(subset(point, Object.keys(point.kind === 'body' ? bodies.get(point.id) : angles.get(point.id))), point.kind === 'body' ? bodies.get(point.id) : angles.get(point.id));
    if (point.kind === 'houseCusp') {
      assert.equal(point.id, `H${point.houseNumber}`);nearLongitude(c.chart.houseCusps[point.houseNumber - 1], point.longitude);
      const house = c.chart.houses[point.houseNumber - 1];assert.equal(house.number, point.houseNumber);nearLongitude(house.longitude, point.longitude);
    }
  }
  let zodiacSpan = 0;
  for (let i = 0; i < 12; i++) { const span = norm(c.chart.houseCusps[(i + 1) % 12] - c.chart.houseCusps[i]);assert(span > 0 && span < 180);zodiacSpan += span; }
  near(zodiacSpan, 360);
  const rulers = c.context.rulership === 'traditional' ? traditional : modern;
  assert.equal(c.context.rulership, calculation.rulership);
  for (const body of bodies.values()) assert.equal(body.house, houseFor(body.longitude, c.chart.houseCusps));
  for (const house of houses.values()) {
    assert.equal(house.id, `H${house.number}`);assert.equal(house.rulerBodyId, rulers[house.signIndex]);
    assert.deepEqual(house.rulerPlacement, bodies.get(house.rulerBodyId));
    assert.deepEqual(subset(house, ['number', 'longitude', 'sign', 'degreeInSign']), c.chart.houses[house.number - 1]);
    assert.deepEqual(house.occupants, [...bodies.values()].filter(body => body.house === house.number));
  }
  const aspects = mapOf(c.context.aspects, 'index', 'C aspect');const relations = mapOf(c.context.relations, 'id', 'C relation');
  const expectedAspects = [];
  const pointList = [...points.values()];
  for (let i = 0; i < pointList.length; i++) for (let j = i + 1; j < pointList.length; j++) {
    const first = pointList[i];const second = pointList[j];const id = `${first.id}:${second.id}`;const relation = relations.get(id);assert(relation);
    assert.equal(relation.point1, first.id);assert.equal(relation.point2, second.id);const signedDelta = delta(second.longitude, first.longitude);
    // At the antipodal floating-point boundary either signed orientation describes the same arc.
    if (Math.abs(Math.abs(signedDelta) - 180) < 1e-8) near(Math.abs(relation.signedDelta), Math.abs(signedDelta));
    else near(relation.signedDelta, signedDelta);
    near(relation.separation, Math.abs(signedDelta));
    const indices = [];
    for (const rule of calculation.aspectRules) {
      const orb = Math.abs(Math.abs(signedDelta) - rule.angle);
      if (orb <= rule.maxOrb) { const index = expectedAspects.length;indices.push(index);expectedAspects.push({ index, point1: first.id, point2: second.id, angle: rule.angle, separation: Math.abs(signedDelta), orb, maxOrb: rule.maxOrb, applying: null }); }
    }
    assert.deepEqual(relation.aspectIndexes, indices);
  }
  assert.equal(aspects.size, expectedAspects.length);
  for (const expected of expectedAspects) { const actual = aspects.get(expected.index);for (const field of Object.keys(expected)) typeof expected[field] === 'number' ? near(actual[field], expected[field]) : assert.equal(actual[field], expected[field]); }
  const bodyAspects = [...aspects.values()].filter(a => bodies.has(a.point1) && bodies.has(a.point2)).map(a => ({ body1: a.point1, body2: a.point2, ...subset(a, ['angle', 'separation', 'orb', 'maxOrb', 'applying']) }));
  assert.deepEqual(c.chart.aspects, bodyAspects);
  for (const relation of c.context.houseRulerRelations) {
    const h1 = houses.get(relation.house1);const h2 = houses.get(relation.house2);assert(h1 && h2);
    assert.equal(relation.ruler1, h1.rulerBodyId);assert.equal(relation.ruler2, h2.rulerBodyId);assert.equal(relation.sameRuler, h1.rulerBodyId === h2.rulerBodyId);
    if (relation.sameRuler) { assert.equal(relation.relationId, null);assert.deepEqual(relation.aspectIndexes, []); }
    else { const r = [...relations.values()].find(item => (item.point1 === h1.rulerBodyId && item.point2 === h2.rulerBodyId) || (item.point1 === h2.rulerBodyId && item.point2 === h1.rulerBodyId));assert(r);assert.equal(relation.relationId, r.id);assert.deepEqual(relation.aspectIndexes, r.aspectIndexes); }
  }
  const states = mapOf(c.context.advanced.bodyStates, 'bodyId', 'C body state');sameSet(states.keys(), bodies.keys());
  for (const state of states.values()) { assert.equal(state.motion, 'notApplicable');const index = bodies.get(state.bodyId).signIndex;
    assert.equal(state.element, ['fire', 'earth', 'air', 'water'][index % 4]);assert.equal(state.modality, ['cardinal', 'fixed', 'mutable'][index % 3]);assert.equal(state.polarity, index % 2 ? 'negative' : 'positive');
    assert.equal(state.houseType, ['angular', 'succedent', 'cadent'][(bodies.get(state.bodyId).house - 1) % 3]); }
  for (const [group, stateField] of [['elements', 'element'], ['modalities', 'modality'], ['polarities', 'polarity'], ['houseTypes', 'houseType']]) for (const distribution of c.context.advanced.distributions[group]) {
    const ids = [...states.values()].filter(state => state[stateField] === distribution.id).map(state => state.bodyId);assert.equal(distribution.count, ids.length);assert.deepEqual(distribution.bodyIds, ids);
  }
  for (const dispositor of c.context.dispositors) { const body = bodies.get(dispositor.bodyId);assert(body);assert.equal(dispositor.sign, body.sign);assert.equal(dispositor.dispositorBodyId, rulers[body.signIndex]);assert.deepEqual(dispositor.dispositorPlacement, bodies.get(dispositor.dispositorBodyId)); }
  const chains = mapOf(c.context.advanced.dispositorChains, 'id', 'C chain');const receptions = mapOf(c.context.advanced.receptions, 'id', 'C reception');const patterns = mapOf(c.context.advanced.aspectPatterns, 'id', 'C pattern');
  for (const chain of chains.values()) { contains(bodies, chain.path, 'chain body');assert.equal(chain.path[0], chain.bodyId);for (let i = 1; i < chain.path.length; i++) assert.equal(chain.path[i], rulers[bodies.get(chain.path[i - 1]).signIndex]); }
  for (const reception of receptions.values()) { contains(bodies, reception.bodyIds, 'reception body');assert.equal(reception.bodyIds.length, 2);const [a,b] = reception.bodyIds;assert.equal(rulers[bodies.get(a).signIndex], b);assert.equal(rulers[bodies.get(b).signIndex], a); }
  for (const pattern of patterns.values()) { contains(bodies, pattern.bodyIds, 'pattern body');contains(aspects, pattern.aspectIndexes, 'pattern aspect');if (pattern.apexBodyId !== null) assert(pattern.bodyIds.includes(pattern.apexBodyId)); }
  for (const view of [...Object.values(c.domains), ...Object.values(c.customDomains)]) {
    const selectedBodies = [...bodies.values()].filter(body => view.definition.bodies.includes(body.id)
      || view.definition.houses.includes(body.house)
      || view.definition.houses.some(number => houses.get(`H${number}`).rulerBodyId === body.id));
    assert.deepEqual(view.bodies.map(item => item.placement), selectedBodies);
    assert.deepEqual(view.angles, [...angles.values()].filter(angle => view.definition.angles.includes(angle.id)));
    assert.deepEqual(view.houses, [...houses.values()].filter(house => view.definition.houses.includes(house.number)));
    const primary = new Set([...selectedBodies.map(body => body.id), ...view.definition.angles,
      ...view.definition.houses.map(number => `H${number}`)]);
    sameSet(view.pointIds, primary);
    sameSet(view.relationIds, [...relations.values()].filter(r => primary.has(r.point1) || primary.has(r.point2)).map(r => r.id));
    const matchingAspects = [...aspects.values()].filter(a => primary.has(a.point1) || primary.has(a.point2));
    sameSet(view.aspects.map(a => a.index), matchingAspects.map(a => a.index));
    for (const a of view.aspects) {
      assert.deepEqual(subset(a, Object.keys(aspects.get(a.index))), aspects.get(a.index));
      sameSet(a.selectedEndpoints, [a.point1, a.point2].filter(id => primary.has(id)));
    }
    const report = view.report;
    for (const point of report.points) assert.deepEqual(point, points.get(point.id));
    for (const fact of report.bodyFacts) assert.deepEqual(fact, states.get(fact.bodyId));
    for (const house of report.houses) {
      assert.deepEqual(subset(house, Object.keys(houses.get(house.id))), houses.get(house.id));
      contains(chains, [house.rulerDispositorChainId], 'report house ruler chain');
      assert.equal(house.rulerPlacementHouseId, `H${bodies.get(house.rulerBodyId).house}`);
      contains(aspects, house.aspectIndexes, 'report house aspect');
      for (const evidence of house.evidence) {
        if (evidence.rule === 'dispositorChain') contains(chains, [evidence.chainId], 'house evidence chain');
        if (evidence.rule === 'aspectEndpoint') contains(aspects, [evidence.aspectIndex], 'house evidence aspect');
      }
    }
    for (const a of report.aspects) {
      assert.deepEqual(subset(a, Object.keys(aspects.get(a.index))), aspects.get(a.index));
      assert.equal(a.primaryContact, primary.has(a.point1) || primary.has(a.point2));
      contains(houses, a.relatedHouseIds, 'report aspect house');
    }
    for (const [items, map] of [[report.dispositorChains, chains], [report.receptions, receptions], [report.aspectPatterns, patterns]])
      for (const item of items) assert.deepEqual(item, map.get(item.id));
  }
  checkReports({ data: c, calculation });
}

const setOf = (items, key) => new Set(items.map(item => item[key]));
function checkReports(response) {
  const { data, calculation } = response;
  const { context } = data;
  const contextPoints = setOf(context.points, 'id');
  const contextHouses = setOf(context.houses, 'id');
  const contextAspects = setOf(context.aspects, 'index');
  const catalog = new Map(data.profileCatalog.map(profile => [profile.id, profile]));
  assert.deepEqual([...catalog.keys()].sort(), [...builtinIds].sort());
  assert.equal(calculation.customProfileCount, Object.keys(data.customDomains).length);
  for (const [origin, views] of [['builtin', data.domains], ['custom', data.customDomains]]) {
    for (const [id, view] of Object.entries(views)) {
      const { definition, report } = view;
      assert.equal(view.origin, origin);
      assert.equal(definition.id, id);
      if (origin === 'builtin') assert.deepEqual(definition, catalog.get(id));
      else {
        assert.equal(view.profileId, id);
        assert.equal(view.profileVersion, definition.version);
      }
      const selectors = profile => ({ houses: profile.houses, bodies: profile.bodies,
        angles: profile.angles, aspectSelection: 'atLeastOneSelectedEndpoint' });
      assert.deepEqual(view.selectionRules, selectors(definition));
      const sections = new Map(report.sections.map(section => [section.id, section]));
      assert.equal(sections.size, report.sections.length);
      assert.deepEqual([...sections.keys()].sort(), definition.sections.map(section => section.id).sort());
      const reportPoints = setOf(report.points, 'id');
      const reportHouses = setOf(report.houses, 'id');
      const reportAspects = setOf(report.aspects, 'index');
      const bodyFacts = setOf(report.bodyFacts, 'bodyId');
      const chains = setOf(report.dispositorChains, 'id');
      const receptions = setOf(report.receptions, 'id');
      const patterns = setOf(report.aspectPatterns, 'id');
      assert.equal(report.coverage.sectionCount, sections.size);
      assert.equal(report.coverage.primaryAspectCount, view.aspects.length);
      assert.equal(report.coverage.reportAspectCount, report.aspects.length);
      assert.equal(report.coverage.relatedHouseCount, report.houses.length);
      for (const sectionDefinition of definition.sections) {
        const section = sections.get(sectionDefinition.id);
        assert.deepEqual(section.selectionRules, selectors(sectionDefinition));
        for (const pointId of section.primaryPointIds) {
          assert(contextPoints.has(pointId), `Missing global section point ${pointId}`);
          assert(reportPoints.has(pointId), `Missing report section point ${pointId}`);
        }
        for (const houseId of [...section.focusHouseIds, ...section.relatedHouseIds]) {
          assert(contextHouses.has(houseId), `Missing global section house ${houseId}`);
          assert(reportHouses.has(houseId), `Missing report section house ${houseId}`);
        }
        for (const index of section.aspectIndexes) {
          assert(contextAspects.has(index), `Missing global section aspect ${index}`);
          assert(reportAspects.has(index), `Missing report section aspect ${index}`);
        }
        for (const [refs, targets] of [[section.bodyFactIds, bodyFacts],
          [section.dispositorChainIds, chains], [section.receptionIds, receptions],
          [section.aspectPatternIds, patterns]]) {
          for (const ref of refs) assert(targets.has(ref), `Missing section fact ${ref}`);
        }
      }
      for (const house of report.houses) {
        for (const evidence of house.evidence) {
          if (evidence.sectionId !== undefined) assert(sections.has(evidence.sectionId));
        }
      }
      for (const aspect of report.aspects) {
        assert(reportPoints.has(aspect.point1) && reportPoints.has(aspect.point2));
        for (const sectionId of aspect.sectionIds) {
          assert(sections.has(sectionId), `Missing aspect section ${sectionId}`);
          assert(sections.get(sectionId).aspectIndexes.includes(aspect.index));
        }
      }
    }
  }
}
let successes = 0;let rejectedMutations = 0;let rejectedJoins = 0;let errors = 0;
for (const file of files) {
  const response = JSON.parse(fs.readFileSync(file, 'utf8'));
  const structural = response.data?.chartKind === 'coupleSynastry' ? validateCouple : validate;
  assert(structural(response), `${file}: ${JSON.stringify(structural.errors, null, 2)}`);
  if (response.errors.length) { errors++;continue; }
  checkComposite(response);successes++;
  const c = value => value.data.composite;
  const metadata = value => value.data.chartKind === 'coupleSynastry' ? value.calculation.composite : value.calculation;
  const group = Object.keys(c(response).domains).length ? 'domains' : 'customDomains';const id = Object.keys(c(response)[group])[0];
  const view = value => c(value)[group][id];
  const mutations = [
    ['physical C UTC', value => { c(value).chart.utc = birth.utc; }],
    ['physical C location', value => { c(value).chart.location = birth.location; }],
    ['physical C latitude', value => { c(value).chart.placements[0].latitude = 0; }],
    ['physical C distance', value => { c(value).chart.placements[0].distanceAu = 1; }],
    ['physical C speed', value => { c(value).chart.placements[0].speed = 1; }],
    ['retrograde C', value => { c(value).chart.placements[0].isRetrograde = true; }],
    ['physical context latitude', value => { c(value).context.points[0].latitude = 0; }],
    ['physical context motion', value => { c(value).context.advanced.bodyStates[0].motion = 'direct'; }],
    ['wrong motion source', value => { c(value).context.advanced.rules.motion.source = 'instantaneousLongitudeSpeed'; }],
    ['wrong body population', value => { c(value).context.advanced.rules.distributions.population = 'allNatalBodies'; }],
    ['physical C JD', value => { metadata(value).julianDayTt = 1; }],
    ['physical C ephemeris', value => { metadata(value).ephemeris = 'moshier'; }],
    ['wrong chart count', value => { value.data.chartCount = 2; }],
    ['missing source B', value => { delete value.data.subjects.B; }],
    ['missing C provenance', value => { delete c(value).provenance; }],
    ['wrong provenance count', value => { c(value).provenance.points.pop(); }],
    ['wrong point count', value => { c(value).context.points.pop(); }],
    ['wrong relation count', value => { c(value).context.relations.pop(); }],
    ['missing advanced', value => { delete c(value).context.advanced; }],
    ['wrong report chart kind', value => { view(value).report.chartKind = 'individualNatal'; }],
    ['wrong report body motion', value => { view(value).report.bodyFacts[0].motion = 'direct'; }],
    ['missing report', value => { delete view(value).report; }],
    ['missing catalog', value => { c(value).profileCatalog.pop(); }],
    ['unknown C field', value => { c(value).compatibilityScore = 80; }],
    ['unknown provenance field', value => { c(value).provenance.synthesizedBirthTime = 0; }],
  ];
  if (c(response).context.aspects.length) mutations.push(['applying C aspect', value => { c(value).context.aspects[0].applying = true; }]);
  if (view(response).report.sections.length) mutations.push(['nested section report', value => { view(value).report.sections[0].report = {}; }]);
  if (response.data.chartKind === 'coupleSynastry') {
    mutations.push(['missing combined metadata', value => { delete value.calculation.composite; }]);
    mutations.push(['missing combined chart', value => { delete value.data.composite; }]);
    mutations.push(['missing combined count', value => { delete value.data.chartCount; }]);
  }
  for (const [name, mutate] of mutations) {
    const changed = structuredClone(response);mutate(changed);
    assert(!structural(changed), `${file}: schema accepted malformed ${name}`);rejectedMutations++;
  }
  const joins = [
    ['wrong corresponding source ID', value => { c(value).provenance.points[0].sourcePointIds[0] = 'A:moon'; }],
    ['wrong source longitude', value => { c(value).provenance.points[0].sourceLongitudes[0] = norm(c(value).provenance.points[0].sourceLongitudes[0] + 1); }],
    ['wrong constructed longitude', value => { c(value).provenance.points[0].longitude = norm(c(value).provenance.points[0].longitude + 1); }],
    ['wrong midpoint resolution', value => { c(value).provenance.points[0].resolution = 'notRequiredForDerivedPoint'; }],
    ['wrong antipodal flag', value => { c(value).provenance.points[0].antipodal = !c(value).provenance.points[0].antipodal; }],
    ['wrong source metadata', value => { metadata(value).sourceCalculations.A.julianDayTt += 1; }],
    ['missing relation join', value => { view(value).relationIds = ['missing']; }],
    ['wrong C distribution', value => { c(value).context.advanced.distributions.elements[0].count += 1; }],
    ['wrong report body source', value => { const fact = view(value).report.bodyFacts[0];fact.bodyId = fact.bodyId === 'sun' ? 'moon' : 'sun'; }],
    ['wrong house ruler join', value => { c(value).context.houseRulerRelations[0].ruler1 = 'missing'; }],
  ];
  if (view(response).report.sections.length) joins.push(['missing section point', value => { view(value).report.sections[0].primaryPointIds = ['missing']; }]);
  for (const [name, mutate] of joins) {
    const changed = structuredClone(response);mutate(changed);
    assert(structural(changed), `${file}: ${name} should be structurally valid: ${JSON.stringify(structural.errors)}`);
    assert.throws(() => checkComposite(changed), undefined, `${file}: semantic validator accepted ${name}`);rejectedJoins++;
  }
}
console.log(`Composite contracts: ${requestCaseCount} request cases; ${files.length} responses (${successes} success, ${errors} error); ${rejectedMutations} malformed structures and ${rejectedJoins} broken semantic joins rejected; wraparound and antipodal policy checks passed`);
