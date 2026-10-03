#!/usr/bin/env node
// Development validation only; Ajv is not a runtime dependency of any consumer package.
// AJV_2020_MODULE can identify an already installed Ajv 2020 module by absolute path.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const Ajv2020 = require(process.env.AJV_2020_MODULE || 'ajv/dist/2020');

const files = process.argv.slice(2);
assert(files.length > 0, 'Usage: node scripts/check-domain-schema.cjs <response.json> [more responses.json]');
const schema = JSON.parse(fs.readFileSync(
  path.join(__dirname, '../schemas/natal-domains-response.schema.json'), 'utf8'));
const ajv = new Ajv2020({ allErrors: true, strict: true });
assert(ajv.validateSchema(schema), JSON.stringify(ajv.errors, null, 2));
const validate = ajv.compile(schema);
const requestSchema = JSON.parse(fs.readFileSync(
  path.join(__dirname, '../schemas/natal-domains-request.schema.json'), 'utf8'));
assert(ajv.validateSchema(requestSchema), JSON.stringify(ajv.errors, null, 2));
const validateRequest = ajv.compile(requestSchema);
const baseRequest = {
  operation: 'natalDomains',
  utc: { year: 2000.0, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
};
const requestCases = [
  ['default builtin selection', true, {}],
  ['all ten builtins', true, { domains: ['career', 'love', 'relationships', 'family', 'finance',
    'identity', 'learning', 'creativity', 'innerLife', 'dailyLife'] }],
  ['custom only', true, { domains: [], customProfiles: [{ id: 'myReport', bodies: ['sun'] }] }],
  ['section extends parent', true, { customProfiles: [{ id: 'myReport', version: 'v1.2-beta',
    houses: [1.0], sections: [{ id: 'career', bodies: ['saturn'], angles: ['midheaven'] }] }] }],
  ['angle only custom', true, { customProfiles: [{ id: 'angleView', angles: ['ascendant'] }] }],
  ['empty domains without profiles', false, { domains: [] }],
  ['empty custom array', false, { customProfiles: [] }],
  ['null custom array', false, { customProfiles: null }],
  ['null domains', false, { domains: null }],
  ['unknown builtin', false, { domains: ['health'] }],
  ['duplicate builtin', false, { domains: ['career', 'career'] }],
  ['empty profile selectors', false, { customProfiles: [{ id: 'empty' }] }],
  ['sections cannot replace parent selectors', false, { customProfiles: [{ id: 'empty',
    sections: [{ id: 'detail', houses: [1] }] }] }],
  ['builtin profile ID', false, { customProfiles: [{ id: 'identity', houses: [1] }] }],
  ['prototype profile ID', false, { customProfiles: [{ id: 'constructor', houses: [1] }] }],
  ['prototype section ID', false, { customProfiles: [{ id: 'myReport', houses: [1],
    sections: [{ id: 'prototype', bodies: ['sun'] }] }] }],
  ['non ASCII profile ID', false, { customProfiles: [{ id: 'côngViệc', houses: [1] }] }],
  ['trailing newline profile ID', false, { customProfiles: [{ id: 'myReport\n', houses: [1] }] }],
  ['profile ID starts uppercase', false, { customProfiles: [{ id: 'Report', houses: [1] }] }],
  ['too long profile ID', false, { customProfiles: [{ id: 'a'.repeat(65), houses: [1] }] }],
  ['invalid version', false, { customProfiles: [{ id: 'myReport', version: '../1', houses: [1] }] }],
  ['trailing newline version', false, { customProfiles: [{ id: 'myReport', version: '1.0\n', houses: [1] }] }],
  ['null version', false, { customProfiles: [{ id: 'myReport', version: null, houses: [1] }] }],
  ['too long version', false, { customProfiles: [{ id: 'myReport', version: '1'.repeat(33), houses: [1] }] }],
  ['fractional house selector', false, { customProfiles: [{ id: 'myReport', houses: [1.5] }] }],
  ['out of range house', false, { customProfiles: [{ id: 'myReport', houses: [13] }] }],
  ['string house selector', false, { customProfiles: [{ id: 'myReport', houses: ['1'] }] }],
  ['duplicate house selector', false, { customProfiles: [{ id: 'myReport', houses: [1, 1] }] }],
  ['duplicate body selector', false, { customProfiles: [{ id: 'myReport', bodies: ['sun', 'sun'] }] }],
  ['unknown body selector', false, { customProfiles: [{ id: 'myReport', bodies: ['chiron'] }] }],
  ['unknown angle selector', false, { customProfiles: [{ id: 'myReport', angles: ['asc'] }] }],
  ['null selectors', false, { customProfiles: [{ id: 'myReport', houses: null, bodies: ['sun'] }] }],
  ['null sections', false, { customProfiles: [{ id: 'myReport', houses: [1], sections: null }] }],
  ['empty section selectors', false, { customProfiles: [{ id: 'myReport', houses: [1], sections: [{ id: 'empty' }] }] }],
  ['nine profiles', false, { customProfiles: Array.from({ length: 9 }, (_, i) => ({ id: `report${i}`, houses: [1] })) }],
  ['nine sections', false, { customProfiles: [{ id: 'myReport', houses: [1],
    sections: Array.from({ length: 9 }, (_, i) => ({ id: `detail${i}`, houses: [1] })) }] }],
  ['unknown profile field', false, { customProfiles: [{ id: 'myReport', houses: [1], unknown: true }] }],
  ['conflicting aspect settings', false, { aspectPreset: 'major', aspectRules: [] }],
];
for (const [name, expected, options] of requestCases) {
  assert.equal(validateRequest({ ...baseRequest, ...options }), expected,
    `${name}: ${JSON.stringify(validateRequest.errors, null, 2)}`);
}

const errorEnvelope = {
  schemaVersion: '1.0', engineVersion: '0.5.0-alpha.1', data: null, warnings: [],
  errors: [{ code: 'INVALID_INPUT', message: 'Invalid birth input' }],
};
assert(validate(errorEnvelope), JSON.stringify(validate.errors, null, 2));

const builtinIds = ['career', 'love', 'relationships', 'family', 'finance',
  'identity', 'learning', 'creativity', 'innerLife', 'dailyLife'];
const setOf = (items, key) => new Set(items.map(item => item[key]));
function checkReferences(response) {
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

let successfulResponses = 0;
let rejectedMutations = 0;
let rejectedReferenceMutations = 0;
for (const file of files) {
  const response = JSON.parse(fs.readFileSync(file, 'utf8'));
  assert(validate(response), `${file}: ${JSON.stringify(validate.errors, null, 2)}`);
  if (response.errors.length > 0) continue;
  checkReferences(response);
  successfulResponses++;
  const group = Object.keys(response.data.domains).length > 0 ? 'domains' : 'customDomains';
  const domainId = Object.keys(response.data[group])[0];
  const firstView = value => value.data[group][domainId];
  const mutations = [
    ['missing individual chart kind', value => { delete value.data.chartKind; }],
    ['wrong subject count', value => { value.data.subjectCount = 2; }],
    ['missing advanced facts', value => { delete value.data.context.advanced; }],
    ['missing domain report', value => { delete firstView(value).report; }],
    ['missing normalized definition', value => { delete firstView(value).definition.version; }],
    ['wrong origin', value => { firstView(value).origin = 'pair'; }],
    ['missing catalog profile', value => { value.data.profileCatalog.pop(); }],
    ['missing custom domains', value => { delete value.data.customDomains; }],
    ['malformed dignity', value => { value.data.context.advanced.bodyStates[0].dignity.domicile = 'yes'; }],
    ['malformed dispositor path', value => { value.data.context.advanced.dispositorChains[0].path = 'sun'; }],
    ['wrong ruler relation count', value => { value.data.context.houseRulerRelations.pop(); }],
    ['invalid custom domain key', value => { value.data.customDomains.constructor = structuredClone(firstView(value)); }],
  ];
  const report = firstView(response).report;
  if (report.houses.length > 0) mutations.push(['missing report house evidence', value => {
    delete firstView(value).report.houses[0].evidence;
  }]);
  if (report.aspects.length > 0) {
    mutations.push(['malformed report aspect', value => {
      firstView(value).report.aspects[0].primaryContact = 'yes';
    }]);
    mutations.push(['missing aspect section references', value => {
      delete firstView(value).report.aspects[0].sectionIds;
    }]);
  }
  if (report.sections.length > 0) {
    mutations.push(['malformed section aspect reference', value => {
      firstView(value).report.sections[0].aspectIndexes = ['0'];
    }]);
    mutations.push(['missing section references', value => {
      delete firstView(value).report.sections[0].bodyFactIds;
    }]);
    const changed = structuredClone(response);
    firstView(changed).report.sections[0].primaryPointIds = ['missingPoint'];
    assert(validate(changed), `${file}: reference mutation should remain structurally valid`);
    assert.throws(() => checkReferences(changed));
    rejectedReferenceMutations++;
  }
  for (const [name, mutate] of mutations) {
    const changed = structuredClone(response);
    mutate(changed);
    assert(!validate(changed), `${file}: schema incorrectly accepted ${name}`);
    rejectedMutations++;
  }
}
console.log(`Domain contracts: ${requestCases.length} request cases; ${files.length} responses validated, ${successfulResponses} success payloads, ${rejectedMutations} malformed mutations and ${rejectedReferenceMutations} broken joins rejected; error envelope validated`);
