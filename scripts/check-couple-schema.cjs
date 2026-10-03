#!/usr/bin/env node
// Development-only strict schema and semantic join validation. No SDK runtime dependency.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Ajv2020 = require(process.env.AJV_2020_MODULE || 'ajv/dist/2020');
const files = process.argv.slice(2);
assert(files.length > 0, 'Usage: node scripts/check-couple-schema.cjs <response.json> [more responses.json]');
const readSchema = name => JSON.parse(fs.readFileSync(path.join(__dirname, `../schemas/${name}.schema.json`), 'utf8'));
const ajv = new Ajv2020({ allErrors: true, strict: true });
for (const name of ['natal-domains-request', 'natal-domains-response', 'composite-response']) ajv.addSchema(readSchema(name));
const requestSchema = readSchema('couple-request');
const responseSchema = readSchema('couple-response');
assert(ajv.validateSchema(requestSchema), JSON.stringify(ajv.errors));
assert(ajv.validateSchema(responseSchema), JSON.stringify(ajv.errors));
const validateRequest = ajv.compile(requestSchema);
const validate = ajv.compile(responseSchema);
const builtinIds = ['attraction', 'communication', 'emotionalConnection', 'longTerm', 'sharedResources', 'homeFamily'];
const birth = { utc: { year: 2000.0, month: 1, day: 1, hour: 12, minute: 0 }, location: { latitude: 10.8231, longitude: 106.6297 } };
const baseRequest = { operation: 'couple', personA: birth, personB: { ...birth, houseSystem: 'wholeSign' } };
const custom = { id: 'myPair', houses: [7], bodies: ['venus'], sections: [{ id: 'study', houses: [3, 9], bodies: ['mercury'] }] };
const requestCases = [
  ['default pair profiles', true, {}], ['all pair profiles', true, { domains: builtinIds }],
  ['one builtin', true, { domains: ['communication'] }],
  ['custom only', true, { domains: [], customProfiles: [custom] }],
  ['custom section extends parent', true, { customProfiles: [custom] }],
  ['empty aspect rules', true, { aspectRules: [] }], ['major preset', true, { aspectPreset: 'major' }],
  ['modern rulership', true, { rulership: 'modern' }],
  ['angle only custom', true, { customProfiles: [{ id: 'pairAngles', angles: ['ascendant'] }] }],
  ['fractional second', true, { personA: { ...birth, utc: { ...birth.utc, second: 0.5 } } }],
  ['missing person A', false, { personA: undefined }], ['missing person B', false, { personB: undefined }],
  ['null person', false, { personA: null }], ['unknown person field', false, { personA: { ...birth, name: 'Person' } }],
  ['nested person operation', false, { personA: { ...birth, operation: 'natal' } }],
  ['unknown top level field', false, { scoring: true }], ['third subject', false, { personC: birth }],
  ['top level UTC', false, { utc: birth.utc }], ['unknown UTC component', false, { personB: { ...birth, utc: { ...birth.utc, zone: 'UTC' } } }],
  ['string calendar number', false, { personB: { ...birth, utc: { ...birth.utc, year: '2000' } } }],
  ['fractional calendar number', false, { personB: { ...birth, utc: { ...birth.utc, day: 1.5 } } }],
  ['invalid year range', false, { personB: { ...birth, utc: { ...birth.utc, year: 2400 } } }],
  ['pole latitude', false, { personA: { ...birth, location: { latitude: 90, longitude: 0 } } }],
  ['invalid house system', false, { personA: { ...birth, houseSystem: 'equal' } }],
  ['null house system', false, { personA: { ...birth, houseSystem: null } }],
  ['empty domains without custom', false, { domains: [] }], ['null domains', false, { domains: null }],
  ['duplicate domain', false, { domains: ['attraction', 'attraction'] }],
  ['individual domain', false, { domains: ['love'] }], ['unknown domain', false, { domains: ['health'] }],
  ['conflicting aspects', false, { aspectRules: [], aspectPreset: 'extended' }],
  ['null aspects', false, { aspectRules: null }], ['too many rules', false, { aspectRules: Array.from({ length: 17 }, (_, angle) => ({ angle, maxOrb: 1 })) }],
  ['invalid orb', false, { aspectRules: [{ angle: 60, maxOrb: 16 }] }], ['unknown rule field', false, { aspectRules: [{ angle: 60, maxOrb: 1, name: 'sextile' }] }],
  ['empty custom profiles', false, { customProfiles: [] }], ['null custom profiles', false, { customProfiles: null }],
  ['empty selectors', false, { customProfiles: [{ id: 'empty' }] }],
  ['individual reserved ID', false, { customProfiles: [{ id: 'career', houses: [1] }] }],
  ['pair reserved ID', false, { customProfiles: [{ id: 'attraction', houses: [1] }] }],
  ['prototype ID', false, { customProfiles: [{ id: 'constructor', houses: [1] }] }],
  ['non ASCII ID', false, { customProfiles: [{ id: 'cặpĐôi', houses: [1] }] }],
  ['trailing newline ID', false, { customProfiles: [{ id: 'myPair\n', houses: [1] }] }],
  ['invalid version', false, { customProfiles: [{ id: 'myPair', version: '../1', houses: [1] }] }],
  ['newline version', false, { customProfiles: [{ id: 'myPair', version: '1.0\n', houses: [1] }] }],
  ['fractional house', false, { customProfiles: [{ id: 'myPair', houses: [1.5] }] }],
  ['house out of range', false, { customProfiles: [{ id: 'myPair', houses: [13] }] }],
  ['duplicate body', false, { customProfiles: [{ id: 'myPair', bodies: ['sun', 'sun'] }] }],
  ['unknown body', false, { customProfiles: [{ id: 'myPair', bodies: ['chiron'] }] }],
  ['unknown angle', false, { customProfiles: [{ id: 'myPair', angles: ['asc'] }] }],
  ['section cannot substitute parent selection', false, { customProfiles: [{ id: 'myPair', sections: [{ id: 'study', bodies: ['sun'] }] }] }],
  ['empty section', false, { customProfiles: [{ id: 'myPair', bodies: ['sun'], sections: [{ id: 'study' }] }] }],
  ['unknown section field', false, { customProfiles: [{ id: 'myPair', bodies: ['sun'], sections: [{ id: 'study', bodies: ['moon'], version: '1' }] }] }],
  ['nine sections', false, { customProfiles: [{ id: 'myPair', bodies: ['sun'], sections: Array.from({ length: 9 }, (_, i) => ({ id: `detail${i}`, bodies: ['moon'] })) }] }],
  ['nine profiles', false, { customProfiles: Array.from({ length: 9 }, (_, i) => ({ id: `pair${i}`, bodies: ['sun'] })) }],
];
for (const [name, expected, options] of requestCases) {
  // Remove undefined to represent omitted JSON properties.
  const request = JSON.parse(JSON.stringify({ ...baseRequest, ...options }));
  assert.equal(validateRequest(request), expected, `${name}: ${JSON.stringify(validateRequest.errors)}`);
}
const errorEnvelope = { schemaVersion: '1.0', engineVersion: '0.6.0-alpha.1', data: null, warnings: [], errors: [{ code: 'INVALID_INPUT', message: 'Invalid person A input' }] };
assert(validate(errorEnvelope), JSON.stringify(validate.errors));
const mapOf = (items, key, name) => {
  const result = new Map(items.map(item => [item[key], item]));
  assert.equal(result.size, items.length, `Duplicate ${name || key}`);
  return result;
};
const unique = values => assert.equal(new Set(values).size, values.length, 'Duplicate references');
const contains = (target, ids, name) => { unique(ids); for (const id of ids) assert(target.has(id), `Missing ${name}: ${id}`); };
const sameSet = (left, right) => assert.deepEqual([...left].sort(), [...right].sort());
const close = (left, right) => assert(Math.abs(left - right) <= 1e-9, `Numeric mismatch ${left} != ${right}`);
const normalize = value => ((value % 360) + 360) % 360;
const houseFor = (longitude, cusps) => cusps.findIndex((cusp, index) => normalize(longitude - cusp) < normalize(cusps[(index + 1) % 12] - cusp)) + 1;
const canonicalRelation = (left, right) => left.startsWith('A:') ? `${left}:${right}` : `${right}:${left}`;
const aspectFields = ['index', 'relationId', 'point1', 'point2', 'angle', 'separation', 'orb', 'maxOrb', 'applying'];
const subsetObject = (item, keys) => Object.fromEntries(keys.map(key => [key, item[key]]));
function checkLocal(context, natal) {
  const points = mapOf(context.points, 'id', 'local points');
  const houses = mapOf(context.houses, 'id', 'local houses');
  const aspects = mapOf(context.aspects, 'index', 'local aspects');
  const relations = mapOf(context.relations, 'id', 'local relations');
  const bodies = mapOf(natal.placements, 'id', 'natal bodies');
  assert.equal(points.size, 26); assert.equal(houses.size, 12); assert.equal(relations.size, 325);
  for (const [index, aspect] of aspects) {
    assert.equal(index, context.aspects.indexOf(aspect));
    contains(points, [aspect.point1, aspect.point2], 'local aspect endpoint');
  }
  for (const relation of relations.values()) {
    contains(points, [relation.point1, relation.point2], 'local relation endpoint');
    contains(aspects, relation.aspectIndexes, 'local relation aspect');
    for (const index of relation.aspectIndexes) {
      const aspect = aspects.get(index);
      sameSet([aspect.point1, aspect.point2], [relation.point1, relation.point2]);
    }
  }
  for (const house of houses.values()) {
    assert.equal(house.id, `H${house.number}`); assert.equal(house.rulerBodyId, house.rulerPlacement.id);
    assert.deepEqual(house.rulerPlacement, bodies.get(house.rulerBodyId));
    for (const body of house.occupants) { assert.equal(body.house, house.number); assert.deepEqual(body, bodies.get(body.id)); }
  }
  const advanced = context.advanced;
  const states = mapOf(advanced.bodyStates, 'bodyId', 'local body states');
  const chains = mapOf(advanced.dispositorChains, 'id', 'local chains');
  const receptions = mapOf(advanced.receptions, 'id', 'local receptions');
  const patterns = mapOf(advanced.aspectPatterns, 'id', 'local patterns');
  sameSet(states.keys(), bodies.keys());
  for (const chain of chains.values()) {
    contains(bodies, [chain.bodyId], 'chain body'); contains(bodies, chain.path, 'chain path');
    contains(bodies, chain.cycleBodyIds, 'chain cycle');
    if (chain.terminalBodyId !== null) contains(bodies, [chain.terminalBodyId], 'chain terminal body');
    assert.equal(chain.path[0], chain.bodyId);
  }
  for (const reception of receptions.values()) contains(bodies, reception.bodyIds, 'reception body');
  for (const pattern of patterns.values()) {
    contains(bodies, pattern.bodyIds, 'pattern body'); contains(aspects, pattern.aspectIndexes, 'pattern aspect');
    if (pattern.apexBodyId !== null) assert(pattern.bodyIds.includes(pattern.apexBodyId));
    for (const index of pattern.aspectIndexes) {
      const aspect = aspects.get(index); assert(pattern.bodyIds.includes(aspect.point1) && pattern.bodyIds.includes(aspect.point2));
    }
  }
  for (const groups of Object.values(advanced.distributions)) {
    for (const group of groups) { contains(bodies, group.bodyIds, 'distribution body'); assert.equal(group.count, group.bodyIds.length); }
  }
  return { points, houses, aspects, relations, bodies, states, chains, receptions, patterns };
}
function checkReferences(response) {
  const { data, calculation } = response;
  const locals = {};
  for (const chartId of ['A', 'B']) {
    const subject = data.subjects[chartId]; assert.equal(subject.id, chartId);
    locals[chartId] = checkLocal(subject.context, subject.natal);
    assert.deepEqual(calculation.subjectCalculations[chartId], subsetObject(subject.calculation, ['julianDayTt', 'julianDayUt1', 'houseSystem']));
    assert.deepEqual(subject.calculation.aspectRules, calculation.aspectRules);
    assert.equal(subject.context.rulership, calculation.rulership);
  }
  const { context } = data;
  const points = mapOf(context.points, 'id', 'pair points');
  const relations = mapOf(context.relations, 'id', 'pair relations');
  const aspects = mapOf(context.aspects, 'index', 'cross aspects');
  const overlays = mapOf([...context.overlaysAtoB, ...context.overlaysBtoA], 'id', 'overlays');
  const rulerRelations = mapOf(context.houseRulerRelations, 'id', 'cross house ruler relations');
  assert.equal(points.size, 52); assert.equal(relations.size, 676); assert.equal(rulerRelations.size, 144);
  for (const point of points.values()) {
    assert.equal(point.id, `${point.chartId}:${point.localId}`);
    const expected = { ...locals[point.chartId].points.get(point.localId), id: point.id, chartId: point.chartId, localId: point.localId };
    assert.deepEqual(point, expected);
  }
  for (const relation of relations.values()) {
    assert(relation.point1.startsWith('A:') && relation.point2.startsWith('B:'));
    assert.equal(relation.id, `${relation.point1}:${relation.point2}`);
    contains(points, [relation.point1, relation.point2], 'cross relation endpoint');
    const delta = ((points.get(relation.point2).longitude - points.get(relation.point1).longitude + 540) % 360) - 180;
    close(relation.signedDelta, delta); close(relation.separation, Math.abs(delta));
    contains(aspects, relation.aspectIndexes, 'cross relation aspect');
    for (const index of relation.aspectIndexes) assert.equal(aspects.get(index).relationId, relation.id);
  }
  for (const [index, aspect] of aspects) {
    assert.equal(index, context.aspects.indexOf(aspect)); assert.equal(aspect.applying, null);
    const relation = relations.get(aspect.relationId); assert(relation);
    assert.equal(aspect.point1, relation.point1); assert.equal(aspect.point2, relation.point2);
    close(aspect.separation, relation.separation); close(aspect.orb, Math.abs(aspect.separation - aspect.angle));
    assert(aspect.orb <= aspect.maxOrb + 1e-9); assert(relation.aspectIndexes.includes(index));
    assert(calculation.aspectRules.some(rule => rule.angle === aspect.angle && rule.maxOrb === aspect.maxOrb));
  }
  for (const [source, target, items] of [['A', 'B', context.overlaysAtoB], ['B', 'A', context.overlaysBtoA]]) {
    sameSet(items.map(item => item.localBodyId), locals[source].bodies.keys());
    for (const overlay of items) {
      assert.equal(overlay.sourceChartId, source); assert.equal(overlay.targetChartId, target);
      assert.equal(overlay.pointId, `${source}:${overlay.localBodyId}`);
      const body = locals[source].bodies.get(overlay.localBodyId);
      assert.equal(overlay.sourceHouseId, `${source}:H${body.house}`);
      assert.equal(overlay.targetHouseId, `${target}:H${overlay.targetHouseNumber}`);
      assert.equal(overlay.targetHouseNumber, houseFor(body.longitude, data.subjects[target].natal.houseCusps));
      const house = locals[target].houses.get(`H${overlay.targetHouseNumber}`); assert(house);
      assert.equal(overlay.targetRulerPointId, `${target}:${house.rulerBodyId}`);
      assert.equal(overlay.id, `${overlay.pointId}->${overlay.targetHouseId}`);
      assert.equal(overlay.targetCuspRelationId, canonicalRelation(overlay.pointId, overlay.targetHouseId));
      assert.equal(overlay.targetRulerRelationId, canonicalRelation(overlay.pointId, overlay.targetRulerPointId));
      const cuspRelation = relations.get(overlay.targetCuspRelationId); const rulerRelation = relations.get(overlay.targetRulerRelationId);
      assert(cuspRelation && rulerRelation); contains(aspects, overlay.aspectIndexes, 'overlay aspects');
      sameSet(overlay.aspectIndexes, new Set([...cuspRelation.aspectIndexes, ...rulerRelation.aspectIndexes]));
    }
  }
  for (const relation of rulerRelations.values()) {
    assert(relation.house1.startsWith('A:H') && relation.house2.startsWith('B:H'));
    assert.equal(relation.id, `${relation.house1}:${relation.house2}`);
    const houseA = locals.A.houses.get(relation.house1.slice(2)); const houseB = locals.B.houses.get(relation.house2.slice(2));
    assert(houseA && houseB); assert.equal(relation.ruler1, `A:${houseA.rulerBodyId}`); assert.equal(relation.ruler2, `B:${houseB.rulerBodyId}`);
    assert.equal(relation.sameBodyId, houseA.rulerBodyId === houseB.rulerBodyId);
    assert.equal(relation.relationId, `${relation.ruler1}:${relation.ruler2}`);
    const cross = relations.get(relation.relationId); assert(cross); assert.deepEqual(relation.aspectIndexes, cross.aspectIndexes);
  }
  const catalog = mapOf(data.profileCatalog, 'id', 'catalog profiles'); sameSet(catalog.keys(), builtinIds);
  assert.equal(calculation.customProfileCount, Object.keys(data.customDomains).length);
  const selectors = definition => ({ houses: definition.houses, bodies: definition.bodies, angles: definition.angles,
    aspectSelection: 'atLeastOneSelectedEndpoint', overlaySelection: 'selectedSourceBodyOrTargetFocusHouse', houseRulerSelection: 'selectedHouseOrRulerEndpoint' });
  const factRefs = (refs, type) => {
    unique(refs.map(ref => `${ref.chartId}:${ref.id}`));
    for (const ref of refs) assert(locals[ref.chartId][type].has(ref.id), `Missing ${type} ${ref.chartId}:${ref.id}`);
  };
  for (const [origin, views] of [['builtin', data.domains], ['custom', data.customDomains]]) for (const [id, view] of Object.entries(views)) {
    assert.equal(view.origin, origin); assert.equal(view.definition.id, id); assert.equal(view.profileVersion, view.definition.version);
    if (origin === 'builtin') assert.deepEqual(view.definition, catalog.get(id)); else assert.equal(view.profileId, id);
    assert.deepEqual(view.selectionRules, selectors(view.definition));
    const primary = new Set(view.pointIds);
    const focus = new Set(['A', 'B'].flatMap(chartId => view.definition.houses.map(number => `${chartId}:H${number}`)));
    sameSet(view.relationIds, [...relations.values()].filter(item => primary.has(item.point1) || primary.has(item.point2)).map(item => item.id));
    sameSet(view.aspects.map(item => item.index), [...aspects.values()].filter(item => primary.has(item.point1) || primary.has(item.point2)).map(item => item.index));
    sameSet(view.overlayIds, [...overlays.values()].filter(item => primary.has(item.pointId) || focus.has(item.targetHouseId)).map(item => item.id));
    sameSet(view.houseRulerRelationIds, [...rulerRelations.values()].filter(item => focus.has(item.house1) || focus.has(item.house2) || primary.has(item.ruler1) || primary.has(item.ruler2)).map(item => item.id));
    contains(points, view.pointIds, 'view point'); contains(relations, view.relationIds, 'view relation');
    contains(overlays, view.overlayIds, 'view overlay'); contains(rulerRelations, view.houseRulerRelationIds, 'view ruler relation');
    const selected = [];
    for (const chartId of ['A', 'B']) {
      const selection = view.selections[chartId]; assert.equal(selection.chartId, chartId);
      assert.deepEqual(selection.selectionRules, subsetObject(view.selectionRules, ['houses', 'bodies', 'angles', 'aspectSelection']));
      contains(locals[chartId].points, selection.pointIds, 'local selected point'); contains(locals[chartId].relations, selection.relationIds, 'local selected relation');
      selected.push(...selection.pointIds.map(pointId => `${chartId}:${pointId}`));
      for (const aspect of selection.aspects) {
        assert.deepEqual(subsetObject(aspect, ['index', 'point1', 'point2', 'angle', 'separation', 'orb', 'maxOrb', 'applying']), locals[chartId].aspects.get(aspect.index));
        contains(locals[chartId].points, aspect.selectedEndpoints, 'local selected aspect endpoint');
      }
    }
    sameSet(primary, selected);
    for (const aspect of view.aspects) {
      assert.deepEqual(subsetObject(aspect, aspectFields), aspects.get(aspect.index));
      sameSet(aspect.selectedEndpoints, [aspect.point1, aspect.point2].filter(pointId => primary.has(pointId)));
      assert(aspect.selectedEndpoints.length > 0);
    }
    const report = view.report; sameSet(report.primaryPointIds, primary);
    const reportPoints = mapOf(report.points, 'id', 'report points'); const reportHouses = mapOf(report.houses, 'id', 'report houses');
    const reportBodies = mapOf(report.bodyFacts, 'id', 'report body facts'); const reportAspects = mapOf(report.aspectFacts, 'index', 'report aspect facts');
    const sections = mapOf(report.sections, 'id', 'report sections'); sameSet(sections.keys(), view.definition.sections.map(section => section.id));
    assert.equal(report.coverage.pointCount, reportPoints.size); assert.equal(report.coverage.houseCount, reportHouses.size);
    assert.equal(report.coverage.bodyFactCount, reportBodies.size); assert.equal(report.coverage.aspectCount, reportAspects.size);
    assert.equal(report.coverage.primaryAspectCount, view.aspects.length); assert.equal(report.coverage.sectionCount, sections.size);
    contains(points, [...reportPoints.keys()], 'report point'); contains(overlays, report.overlayIds, 'report overlay'); contains(rulerRelations, report.houseRulerRelationIds, 'report ruler relation');
    for (const point of reportPoints.values()) {
      assert.deepEqual(subsetObject(point, Object.keys(points.get(point.id))), points.get(point.id));
      assert.equal(point.primary, primary.has(point.id)); contains(sections, point.sectionIds, 'point section');
    }
    for (const house of reportHouses.values()) {
      assert.equal(house.id, `${house.chartId}:${house.localId}`); assert.deepEqual(house.house, locals[house.chartId].houses.get(house.localId));
      assert.equal(house.primary, focus.has(house.id)); contains(sections, house.sectionIds, 'house section');
    }
    for (const body of reportBodies.values()) {
      assert.equal(body.id, `${body.chartId}:${body.localId}`); assert.deepEqual(body.facts, locals[body.chartId].states.get(body.localId));
      assert.equal(body.primary, primary.has(body.id)); contains(sections, body.sectionIds, 'body section');
    }
    for (const aspect of reportAspects.values()) {
      assert.deepEqual(subsetObject(aspect, aspectFields), aspects.get(aspect.index));
      assert.equal(aspect.primaryContact, primary.has(aspect.point1) || primary.has(aspect.point2));
      contains(reportPoints, [aspect.point1, aspect.point2], 'report aspect endpoint'); contains(sections, aspect.sectionIds, 'aspect section');
    }
    factRefs(report.dispositorChainRefs, 'chains'); factRefs(report.natalReceptionRefs, 'receptions'); factRefs(report.natalAspectPatternRefs, 'patterns');
    for (const definition of view.definition.sections) {
      const section = sections.get(definition.id); assert.deepEqual(section.selectionRules, selectors(definition));
      contains(reportPoints, section.primaryPointIds, 'section primary point'); contains(reportPoints, section.pointIds, 'section point');
      sameSet(section.focusHouseIds, ['A', 'B'].flatMap(chartId => definition.houses.map(number => `${chartId}:H${number}`)));
      contains(reportHouses, section.focusHouseIds, 'section focus house'); contains(reportHouses, section.houseIds, 'section house');
      contains(reportBodies, section.bodyFactIds, 'section body fact'); contains(reportAspects, section.aspectIndexes, 'section aspect');
      contains(overlays, section.overlayIds, 'section overlay'); contains(rulerRelations, section.houseRulerRelationIds, 'section ruler relation');
      for (const index of section.aspectIndexes) assert(reportAspects.get(index).sectionIds.includes(section.id));
      for (const pointId of section.pointIds) assert(reportPoints.get(pointId).sectionIds.includes(section.id));
      for (const houseId of section.houseIds) assert(reportHouses.get(houseId).sectionIds.includes(section.id));
      for (const bodyId of section.bodyFactIds) assert(reportBodies.get(bodyId).sectionIds.includes(section.id));
      for (const [refs, parentRefs, type] of [[section.dispositorChainRefs, report.dispositorChainRefs, 'chains'], [section.natalReceptionRefs, report.natalReceptionRefs, 'receptions'], [section.natalAspectPatternRefs, report.natalAspectPatternRefs, 'patterns']]) {
        factRefs(refs, type); const parentSet = new Set(parentRefs.map(ref => `${ref.chartId}:${ref.id}`));
        for (const ref of refs) assert(parentSet.has(`${ref.chartId}:${ref.id}`));
      }
    }
  }
}
let successfulResponses = 0; let rejectedMutations = 0; let rejectedJoins = 0;
for (const file of files) {
  const response = JSON.parse(fs.readFileSync(file, 'utf8'));
  assert(validate(response), `${file}: ${JSON.stringify(validate.errors, null, 2)}`);
  if (response.errors.length > 0) continue;
  checkReferences(response); successfulResponses++;
  const group = Object.keys(response.data.domains).length ? 'domains' : 'customDomains'; const id = Object.keys(response.data[group])[0];
  const view = value => value.data[group][id];
  const mutations = [
    ['missing B', value => { delete value.data.subjects.B; }], ['wrong chart kind', value => { value.data.chartKind = 'individualNatal'; }],
    ['wrong subject count', value => { value.data.subjectCount = 1; }], ['unknown pair context field', value => { value.data.context.score = 100; }],
    ['wrong point count', value => { value.data.context.points.pop(); }], ['missing namespace', value => { value.data.context.points[0].id = 'sun'; }],
    ['wrong relation count', value => { value.data.context.relations.pop(); }], ['wrong overlay count', value => { value.data.context.overlaysAtoB.pop(); }],
    ['wrong ruler relation count', value => { value.data.context.houseRulerRelations.pop(); }], ['missing local advanced facts', value => { delete value.data.subjects.A.context.advanced; }],
    ['missing local calculation', value => { delete value.data.subjects.B.calculation; }], ['single top level JD', value => { value.calculation.julianDayTt = 0; }],
    ['missing report', value => { delete view(value).report; }], ['missing normalized profile', value => { delete view(value).definition.version; }],
    ['wrong selection mode', value => { view(value).selectionRules.overlaySelection = 'any'; }], ['wrong local reference scope', value => { view(value).selections.A.referenceScope = 'crossContext'; }],
    ['missing catalog profile', value => { value.data.profileCatalog.pop(); }], ['missing custom map', value => { delete value.data.customDomains; }],
    ['wrong coverage', value => { value.data.context.coverage.crossApplying = 'supported'; }], ['malformed report primary flag', value => { view(value).report.points[0].primary = 'yes'; }],
  ];
  if (response.data.context.aspects.length) mutations.push(['invalid cross applying', value => { value.data.context.aspects[0].applying = true; }]);
  if (view(response).report.sections.length) mutations.push(['nested section report', value => { view(value).report.sections[0].report = {}; }]);
  for (const [name, mutate] of mutations) {
    const changed = structuredClone(response); mutate(changed); assert(!validate(changed), `${file}: accepted malformed ${name}`); rejectedMutations++;
  }
  const joins = [
    ['point local ID mismatch', value => { value.data.context.points[0].localId = 'moon'; }],
    ['wrong overlay direction', value => { value.data.context.overlaysAtoB[0].sourceChartId = 'B'; }],
    ['unknown relation ID', value => { view(value).relationIds = ['missing']; }],
    ['wrong ruler relation target', value => { const relation = value.data.context.houseRulerRelations[0]; relation.ruler2 = relation.ruler2 === 'B:pluto' ? 'B:sun' : 'B:pluto'; }],
    ['unknown local chain ref', value => { view(value).report.dispositorChainRefs = [{ chartId: 'A', id: 'missing' }]; }],
    ['wrong report body fact', value => { const fact = view(value).report.bodyFacts[0].facts; fact.bodyId = fact.bodyId === 'sun' ? 'moon' : 'sun'; }],
  ];
  if (response.data.context.aspects.length) joins.push(['wrong cross aspect relation', value => { value.data.context.aspects[0].relationId = 'missing'; }]);
  if (view(response).report.sections.length) joins.push(['wrong section overlay ID', value => { view(value).report.sections[0].overlayIds = ['missing']; }]);
  for (const [name, mutate] of joins) {
    const changed = structuredClone(response); mutate(changed); assert(validate(changed), `${file}: ${name} should remain structurally valid`);
    assert.throws(() => checkReferences(changed), undefined, `${file}: semantic validator accepted ${name}`); rejectedJoins++;
  }
}
console.log(`Couple contracts: ${requestCases.length} request cases; ${files.length} responses, ${successfulResponses} success payloads; ${rejectedMutations} malformed mutations and ${rejectedJoins} broken joins rejected; error envelope validated`);
