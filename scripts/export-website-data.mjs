#!/usr/bin/env node
/** Export a reproducible, source-backed static documentation dataset. No website UI files are edited. */
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import fs from 'node:fs';
import assert from 'node:assert/strict';
import { product, operations, queryGroups, documentDefinitions, savedSamples, calculatedSamples,
  languageDefinitions, integrations, limitations, compression } from './website-catalog.mjs';

const engineRoot = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const defaultOutput = path.resolve(engineRoot, '../website/data/astrology');
const args = process.argv.slice(2);
let outputRoot = defaultOutput, sdkPath, checkOnly = false;
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--output' && args[i + 1]) outputRoot = path.resolve(args[++i]);
  else if (args[i] === '--sdk-path' && args[i + 1]) sdkPath = path.resolve(args[++i]);
  else if (args[i] === '--check') checkOnly = true;
  else throw new Error('Usage: node scripts/export-website-data.mjs [--output DIR] [--sdk-path PACKAGE_DIR] [--check]');
}
assert(outputRoot !== engineRoot && outputRoot !== path.parse(outputRoot).root, 'Unsafe dataset output directory');
assert(!engineRoot.startsWith(outputRoot + path.sep), 'Output cannot contain the engine source');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const read = relative => fs.readFileSync(path.join(engineRoot, relative));
const readJson = relative => JSON.parse(read(relative).toString('utf8'));
const sorted = values => [...values].sort((a, b) => a.localeCompare(b, 'en'));
function inventory(directory) {
  if (!fs.existsSync(directory)) return [];
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    if (['target', 'node_modules', 'native', 'bin', 'obj', '__pycache__'].includes(entry.name)) return [];
    const filename = path.join(directory, entry.name);
    return entry.isDirectory() ? inventory(filename) : entry.isFile() ? [filename] : [];
  });
}
const uiManifestPath = 'artifacts/ui/manifest.json';
const uiManifest = fs.existsSync(path.join(engineRoot, uiManifestPath)) ? readJson(uiManifestPath) : null;
const documentEntries = [...documentDefinitions, ...(uiManifest ? [{ id: 'ui-package', sourcePath: 'examples/frontend/ui/README.md', title: 'Install the UI package', description: 'Optional ESM renderer, stylesheet and injected calculation API' }] : [])];
const sourcePaths = new Set([
  'README.md', 'bindings/node/package.json', 'bindings/node/index.d.ts',
  'scripts/website-catalog.mjs', 'scripts/export-website-data.mjs',
  'artifacts/test-results.json', 'artifacts/packages/manifest.json',
  ...Object.keys(readJson('artifacts/packages/manifest.json').files).map(filename => `artifacts/packages/${filename}`),
  ...documentEntries.map(d => d.sourcePath),
  ...(uiManifest ? [uiManifestPath, uiManifest.package.path] : []),
  ...savedSamples.flatMap(s => [s.requestPath, s.resultPath]),
  ...languageDefinitions.flatMap(s => s.examples),
  ...integrations.flatMap(i => i.sourcePaths),
  ...compression.evidenceDefinitions.map(e => e.sourcePath),
  ...compression.sourcePaths,
]);
for (const folder of ['docs', 'schemas', 'neutral-engine', 'bindings', 'examples/frontend', 'examples/mcp']) {
  for (const filename of inventory(path.join(engineRoot, folder))) {
    if (/\.(?:rs|toml|lock|c|h|md|txt|json|cjs|mjs|ts|py|cs|csproj|props|targets|html|css)$/.test(filename)) sourcePaths.add(path.relative(engineRoot, filename).split(path.sep).join('/'));
  }
}
const sourceFiles = sorted(sourcePaths).map(sourcePath => {
  const bytes = read(sourcePath); return { path: sourcePath, bytes: bytes.length, sha256: sha(bytes) };
});
const engineVersion = readJson('bindings/node/package.json').version;
const testResults = readJson('artifacts/test-results.json');
const packageManifest = readJson('artifacts/packages/manifest.json');
assert.equal(testResults.engineVersion, engineVersion, 'Installed-package evidence is stale');
assert.equal(testResults.result, 'passed', 'Installed-package verification did not pass');
assert.equal(packageManifest.engineVersion, engineVersion, 'Artifact manifest version mismatch');
const sourceDigest = sha(Buffer.from(JSON.stringify(sourceFiles)));

if (checkOnly) {
  const manifest = JSON.parse(fs.readFileSync(path.join(outputRoot, 'manifest.json'), 'utf8'));
  assert.equal(manifest.engineVersion, engineVersion);
  assert.equal(manifest.sourceDigest, sourceDigest, 'Website snapshot is stale; rerun the exporter');
  assert.deepEqual(manifest.sourceFiles, sourceFiles);
  for (const file of manifest.files) {
    const bytes = fs.readFileSync(path.join(outputRoot, file.path));
    assert.equal(bytes.length, file.bytes, file.path); assert.equal(sha(bytes), file.sha256, file.path);
  }
  console.log(`Website snapshot verified: ${engineVersion}, ${manifest.files.length} files, ${manifest.sourceFiles.length} source fingerprints`);
  process.exit(0);
}

if (!sdkPath) {
  const installed = path.join(engineRoot, 'artifacts/consumers', testResults.consumerFingerprint, 'node/node_modules/@7mlabs/astrology');
  sdkPath = fs.existsSync(installed) ? installed : path.join(engineRoot, 'bindings/node');
}
const require = createRequire(import.meta.url);
const sdk = require(sdkPath);
const probe = JSON.parse(sdk.calculateJson(JSON.stringify({ operation: 'query', group: 'geometry', action: 'normalize', longitude: 0 })));
assert.equal(probe.engineVersion, engineVersion, 'Native SDK version mismatch');
assert.equal(probe.errors.length, 0);
const stage = outputRoot + `.export-${process.pid}`;
assert(!fs.existsSync(stage), 'Unexpected existing export staging directory');
fs.mkdirSync(stage, { recursive: true });
const exported = [];
function put(relative, bytes, sourcePath) {
  const payload = Buffer.isBuffer(bytes) ? bytes : Buffer.from(bytes);
  const existing = exported.find(entry => entry.path === relative);
  if (existing) {
    assert.equal(existing.sha256, sha(payload), `Conflicting export ${relative}`);
    assert.equal(existing.bytes, payload.length, `Conflicting export size ${relative}`);
    return existing;
  }
  const destination = path.join(stage, relative);
  fs.mkdirSync(path.dirname(destination), { recursive: true }); fs.writeFileSync(destination, payload);
  const entry = { path: relative, url: `/data/astrology/${relative}`, bytes: payload.length, sha256: sha(payload) };
  if (sourcePath) entry.sourcePath = sourcePath;
  exported.push(entry); return entry;
}
const putJson = (relative, value, sourcePath) => put(relative, JSON.stringify(value) + '\n', sourcePath);
const sourceEntry = sourcePath => sourceFiles.find(f => f.path === sourcePath);
const slug = text => text.normalize('NFKD').replace(/[\u0300-\u036f]/g, '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'section';
function parseMarkdown(markdown) {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n'), blocks = [], sections = [], slugCounts = new Map();
  let i = 0;
  const isSpecial = line => /^\s*$|^#{1,6}\s|^```|^~~~|^\s*[-*+]\s|^\s*\d+\.\s|^>/.test(line);
  const cells = line => line.trim().replace(/^\|/, '').replace(/\|$/, '').split(/(?<!\\)\|/).map(s => s.trim().replace(/\\\|/g, '|'));
  while (i < lines.length) {
    const line = lines[i], start = i + 1;
    if (!line.trim()) { i++; continue; }
    const heading = /^(#{1,6})\s+(.+)$/.exec(line);
    if (heading) {
      const base = slug(heading[2]), count = (slugCounts.get(base) || 0) + 1; slugCounts.set(base, count);
      const id = count === 1 ? base : `${base}-${count}`;
      const block = { type: 'heading', level: heading[1].length, text: heading[2], id, line: start };
      blocks.push(block); sections.push({ id, title: heading[2], level: block.level, line: start }); i++; continue;
    }
    const fence = /^(```|~~~)(.*)$/.exec(line);
    if (fence) {
      const content = []; i++; while (i < lines.length && !lines[i].startsWith(fence[1])) content.push(lines[i++]);
      if (i < lines.length) i++;
      blocks.push({ type: 'code', language: fence[2].trim(), text: content.join('\n'), line: start }); continue;
    }
    if (line.includes('|') && i + 1 < lines.length && /^\s*\|?\s*:?-{3,}/.test(lines[i + 1])) {
      const headers = cells(line), alignments = cells(lines[i + 1]).map(s => s.startsWith(':') && s.endsWith(':') ? 'center' : s.endsWith(':') ? 'right' : 'left');
      const rows = []; i += 2; while (i < lines.length && lines[i].includes('|') && lines[i].trim()) rows.push(cells(lines[i++]));
      blocks.push({ type: 'table', headers, rows, alignments, line: start }); continue;
    }
    const list = /^\s*(?:([-*+])|(\d+)\.)\s+(.+)$/.exec(line);
    if (list) {
      const ordered = !!list[2], items = [];
      while (i < lines.length) {
        const item = /^\s*(?:([-*+])|(\d+)\.)\s+(.+)$/.exec(lines[i]);
        if (!item || !!item[2] !== ordered) break;
        items.push({ text: item[3], line: i + 1 }); i++;
      }
      blocks.push({ type: 'list', ordered, items, line: start }); continue;
    }
    if (line.startsWith('>')) {
      const content = []; while (i < lines.length && lines[i].startsWith('>')) content.push(lines[i++].replace(/^>\s?/, ''));
      blocks.push({ type: 'blockquote', text: content.join('\n'), line: start }); continue;
    }
    const content = [line]; i++;
    while (i < lines.length && !isSpecial(lines[i]) && !(lines[i].includes('|') && /^\s*\|?\s*:?-{3,}/.test(lines[i + 1] || ''))) content.push(lines[i++]);
    blocks.push({ type: 'paragraph', text: content.join('\n'), line: start });
  }
  return { blocks, sections };
}
function compactPosition(p) {
  const fields = ['id', 'longitude', 'sign', 'degreeInSign', 'house', 'speed', 'isRetrograde'];
  return Object.fromEntries(fields.filter(k => k in p).map(k => [k, p[k]]));
}
function chartPreview(chart) {
  return { placements: (chart.placements || []).map(compactPosition), angles: (chart.angles || []).map(compactPosition), houses: chart.houses || [], houseCusps: chart.houseCusps || [], aspectCount: (chart.aspects || []).length, aspects: (chart.aspects || []).slice(0, 6) };
}
function contextMetrics(context) {
  return { points: (context.points || []).length, relations: (context.relations || []).length, aspects: (context.aspects || []).length,
    houses: (context.houses || []).length, overlays: Array.isArray(context.overlays) ? context.overlays.length : (context.overlaysAtoB || []).length + (context.overlaysBtoA || []).length, houseRulerRelations: (context.houseRulerRelations || []).length };
}
function viewsPreview(data) {
  return [...Object.entries(data.domains || {}), ...Object.entries(data.customDomains || {})].map(([id, view]) => ({
    id, origin: view.origin || (data.domains?.[id] ? 'builtin' : 'custom'), sections: (view.report?.sections || view.sections || []).map(s => s.id),
    points: (view.pointIds || view.natalPointIds || []).length, aspects: (view.aspects || view.aspectIndexes || view.snapshotAspectIndexes || []).length,
    events: (view.eventReferences || []).length,
  }));
}
function eventPreview(event) {
  return { id: event.id, type: event.type, utc: event.utc, local: event.local, bodyIds: event.bodyIds, details: event.details,
    positions: event.positions.map(compactPosition), precision: event.precision,
    ...(event.personalImpact ? { personalImpact: { primaryNatalPointId: event.personalImpact.primaryNatalPointId, primaryAngle: event.personalImpact.primaryAngle,
      affectedNatalPointIds: event.personalImpact.affectedNatalPointIds, affectedNatalHouseIds: event.personalImpact.affectedNatalHouseIds,
      contactCount: event.personalImpact.contacts.length, houseOverlays: event.personalImpact.houseOverlays, rulerLinks: event.personalImpact.rulerLinks }, highlightReasons: event.highlightReasons } : {}) };
}
function summarize(request, result) {
  const base = { operation: request.operation, engineVersion: result.engineVersion, warnings: result.warnings, errors: result.errors,
    scope: result.calculation?.scope || null, dataIsPreview: true, fullPayloadRequiredForAllReferences: true };
  if (result.errors.length) return { ...base, data: null };
  const d = result.data;
  if (request.operation === 'natal') return { ...base, chart: chartPreview(d) };
  if (request.operation === 'natalDomains') return { ...base, chartKind: d.chartKind, subjectCount: d.subjectCount,
    chart: chartPreview(d.natal), coverage: contextMetrics(d.context), views: viewsPreview(d), profileCatalog: d.profileCatalog };
  if (request.operation === 'couple' || request.operation === 'composite') {
    const subjects = Object.fromEntries(Object.entries(d.subjects).map(([id, subject]) => [id, { id: subject.id, chart: chartPreview(subject.natal) }]));
    const composite = d.composite ? { chartKind: d.composite.chartKind, chart: chartPreview(d.composite.chart), coverage: contextMetrics(d.composite.context), views: viewsPreview(d.composite), provenance: d.composite.provenance } : null;
    return { ...base, chartKind: d.chartKind, subjectCount: d.subjectCount, chartCount: d.chartCount || d.subjectCount, subjects,
      ...(d.context ? { coverage: contextMetrics(d.context), views: viewsPreview(d) } : {}), composite };
  }
  if (request.operation === 'events' || request.operation === 'forecast') return {
    ...base, chartKind: d.chartKind, period: d.period, ...(d.subject ? { chart: chartPreview(d.subject.natal) } : {}),
    snapshot: { utc: d.snapshot.utc, local: d.snapshot.local, positions: d.snapshot.positions.map(compactPosition), relations: d.snapshot.relations ? d.snapshot.relations.length : null,
      aspects: d.snapshot.aspects ? d.snapshot.aspects.length : null, houseOverlays: d.snapshot.houseOverlays || [] },
    coverage: d.coverage || { eventCount: d.overview.eventCount, byType: d.overview.byType }, views: viewsPreview(d),
    eventPreviewLimit: 6, eventPreviewCount: Math.min(d.events.length, 6), events: d.events.slice(0, 6).map(eventPreview),
    highlightCount: d.overview ? d.overview.highlightEventIds.length : null, search: result.calculation.search,
  };
  if (request.operation === 'query') {
    if (request.action !== 'inspect') return { ...base, dataIsPreview: false, fullPayloadRequiredForAllReferences: false, data: d };
    return { ...base, group: d.group, action: d.action, selection: d.selection, coverage: d.coverage, primaryPointIds: d.primaryPointIds, primaryHouseIds: d.primaryHouseIds,
      points: d.points.slice(0, 6), houses: d.houses.map(h => ({ id: h.id, number: h.number, longitude: h.longitude, sign: h.sign, rulerBodyId: h.rulerBodyId, occupantIds: h.occupants.map(p => p.id) })),
      relationPreviewCount: Math.min(d.relations.length, 4), relations: d.relations.slice(0, 4), aspectPreviewCount: Math.min(d.aspects.length, 6), aspects: d.aspects.slice(0, 6) };
  }
  return { ...base, ...(d.placements ? { chart: chartPreview(d), midpointCount: d.midpoints?.length || 0, midpoints: (d.midpoints || []).slice(0, 4), harmonic: d.harmonic || null } : {}),
    ...(d.personA ? { personA: d.personA.map(compactPosition), personB: d.personB.map(compactPosition), crossAspectCount: d.crossAspects.length,
      crossAspects: d.crossAspects.slice(0, 6), overlaysAtoB: d.overlaysAtoB, overlaysBtoA: d.overlaysBtoA } : {}) };
}
try {
  const documents = documentEntries.map(def => {
    const bytes = read(def.sourcePath), markdown = bytes.toString('utf8');
    const raw = put(`docs/${def.id}.md`, bytes, def.sourcePath);
    const parsed = putJson(`docs/${def.id}.json`, { ...def, markdown, ...parseMarkdown(markdown) }, def.sourcePath);
    return { ...def, raw, parsed, source: sourceEntry(def.sourcePath) };
  });
  const schemas = sorted(inventory(path.join(engineRoot, 'schemas'))).filter(p => p.endsWith('.json')).map(filename => {
    const sourcePath = path.relative(engineRoot, filename).split(path.sep).join('/'), bytes = read(sourcePath), schema = JSON.parse(bytes);
    const download = put(`schemas/${path.basename(filename)}`, bytes, sourcePath);
    return { id: path.basename(filename, '.schema.json'), title: schema.title || '', schemaId: schema.$id, download, source: sourceEntry(sourcePath) };
  });
  const samples = [...savedSamples, ...calculatedSamples].map(def => {
    const request = def.request || (def.baseRequestPath ? { ...readJson(def.baseRequestPath), ...def.overrides } : readJson(def.requestPath));
    const result = def.resultPath ? readJson(def.resultPath) : JSON.parse(sdk.calculateJson(JSON.stringify(request)));
    assert.equal(result.engineVersion, engineVersion, `Stale result ${def.id}`);
    assert.equal(result.schemaVersion, '1.0');
    if (def.expectedError) { assert.equal(result.data, null); assert.equal(result.errors[0]?.code, def.expectedError); }
    else assert.equal(result.errors.length, 0, `Unexpected sample error ${def.id}`);
    const summary = summarize(request, result), requestFile = putJson(`samples/${def.id}/request.json`, request, def.requestPath || def.sourcePath);
    const resultFile = putJson(`samples/${def.id}/result.json`, result, def.resultPath);
    const summaryFile = putJson(`samples/${def.id}/summary.json`, summary);
    return { id: def.id, title: def.title, operation: request.operation, group: request.group || null, action: request.action || null,
      origin: def.origin, inputKind: def.inputKind || (['chart', 'harmonic', 'synastry'].includes(request.operation) || (request.operation === 'query' && request.action !== 'inspect') ? 'supplied-geometry' : request.operation === 'events' ? 'period-input' : request.operation === 'forecast' ? 'birth-and-period-input' : ['couple', 'composite'].includes(request.operation) ? 'two-birth-inputs' : 'birth-input'), expectedError: def.expectedError || null,
      sourcePaths: [def.requestPath, def.resultPath, def.baseRequestPath, def.sourcePath].filter(Boolean), request: requestFile, result: resultFile, summary: summaryFile };
  });
  assert.equal(new Set(samples.filter(s => !s.expectedError).map(s => s.operation)).size, operations.length, 'Missing operation sample');
  for (const group of queryGroups) for (const method of group.methods) assert(samples.some(s => s.group === group.id && s.action === method.id), `Missing query ${group.id}.${method.id}`);
  const downloads = Object.entries(packageManifest.files).map(([filename, expectedSha]) => {
    const sourcePath = `artifacts/packages/${filename}`, bytes = read(sourcePath); assert.equal(sha(bytes), expectedSha, filename);
    return { ...put(`packages/${filename}`, bytes, sourcePath), kind: filename.split('.').at(-1), filename, version: engineVersion,
      platform: packageManifest.platform, arch: packageManifest.arch, rid: packageManifest.rid, publication: 'local-alpha-artifact' };
  });
  const sdkEntries = languageDefinitions.map(language => ({ ...language, examples: language.examples.map(sourcePath => ({ sourcePath, ...put(`source/${sourcePath}`, read(sourcePath), sourcePath) })),
    ...(language.artifactKind ? { download: downloads.find(d => d.kind === language.artifactKind) } : {}) }));
  const compressionSamples = compression.sampleDefinitions.map(def => {
    const source = samples.find(sample => sample.id === def.sourceSampleId);
    assert(source, `Unknown compression source sample ${def.sourceSampleId}`);
    const original = JSON.parse(fs.readFileSync(path.join(stage, source.result.path), 'utf8'));
    const request = { payload: original, options: def.options };
    const rawContext = sdk.compressPayloadJson(JSON.stringify(original), JSON.stringify(def.options));
    const context = JSON.parse(rawContext);
    assert.equal(context.engineVersion, engineVersion);
    assert.equal(context.data?.format, compression.format);
    assert.deepEqual(context.errors, []);
    assert.equal(context.data.metrics.outputBytes, Buffer.byteLength(rawContext, 'utf8'), 'Compression metric must count the complete native wire');
    const rawExpanded = sdk.expandContextJson(rawContext), expanded = JSON.parse(rawExpanded);
    assert.equal(expanded.engineVersion, engineVersion);
    if (def.options.mode === 'compact') {
      assert.deepEqual(expanded, original, 'Compact sample must restore the full source envelope');
      assert.equal(context.data.coverage.complete, true); assert.deepEqual(context.data.omitted, []);
    } else {
      assert.deepEqual(Object.keys(expanded.data.domains), ['career']);
      for (const field of ['natal', 'context']) assert.deepEqual(expanded.data[field], original.data[field], `Focused sample changed supporting ${field}`);
      assert.deepEqual(expanded.warnings, original.warnings); assert.deepEqual(expanded.calculation, original.calculation);
      assert.equal(context.data.coverage.complete, false); assert(context.data.omitted.length > 0);
    }
    if (def.options.maxBytes) {
      assert.equal(context.data.budget.exceeded, true);
      assert(context.warnings.some(warning => warning.includes('COMPRESSION_BUDGET_EXCEEDED')));
    }
    const markerCounts = { tables: 0, references: 0, literals: 0 };
    function countMarkers(value) {
      if (!value || typeof value !== 'object') return;
      if (Object.hasOwn(value, '$astroTable')) markerCounts.tables++;
      if (Object.hasOwn(value, '$astroRef')) markerCounts.references++;
      if (Object.hasOwn(value, '$astroLiteral')) markerCounts.literals++;
      for (const entry of Object.values(value)) countMarkers(entry);
    }
    countMarkers(context.data.payload); countMarkers(context.data.dictionary);
    const summary = {
      dataIsPreview: true, fullContextRequiredForAllReferences: true,
      schemaVersion: context.schemaVersion, engineVersion: context.engineVersion,
      format: context.data.format, mode: context.data.mode, options: def.options,
      sourceSampleId: def.sourceSampleId, sourceOperation: 'natalDomains',
      metrics: context.data.metrics, byteReductionPercent: Math.round((1 - context.data.metrics.outputBytes / context.data.metrics.inputBytes) * 10000) / 100,
      coverage: context.data.coverage, omitted: context.data.omitted, budget: context.data.budget,
      encoding: { dictionaryEntryCount: Object.keys(context.data.dictionary).length, markerCounts },
      decodedDomainIds: Object.keys(expanded.data.domains), decodedSourceVersion: expanded.engineVersion,
      warnings: context.warnings, errors: context.errors,
      verification: { nativeCompression: true, nativeExpansion: true, fullJsonRoundtrip: def.options.mode === 'compact', sharedContextUnchanged: true },
    };
    const directory = `compression/${def.id}`;
    return { id: def.id, title: def.title, sourceSampleId: def.sourceSampleId, mode: def.options.mode, options: def.options,
      original: source.result,
      request: putJson(`${directory}/request.json`, request),
      context: put(`${directory}/context.json`, rawContext + '\n'),
      summary: putJson(`${directory}/summary.json`, summary),
      expanded: put(`${directory}/expanded.json`, rawExpanded + '\n'),
      metrics: summary.metrics, byteReductionPercent: summary.byteReductionPercent,
      coverage: summary.coverage, omitted: summary.omitted, budget: summary.budget, warnings: summary.warnings,
      verification: summary.verification };
  });
  const compressionEvidence = compression.evidenceDefinitions.map(def => {
    const result = readJson(def.sourcePath);
    assert.equal(result.engineVersion, engineVersion, `Stale compression evidence ${def.id}`);
    assert.equal(result.result, 'passed', `Failed compression evidence ${def.id}`);
    return { id: def.id, result, download: put(`verification/${path.basename(def.sourcePath)}`, read(def.sourcePath), def.sourcePath) };
  });
  const compressionSources = compression.sourcePaths.map(sourcePath => ({ sourcePath, ...put(`source/${sourcePath}`, read(sourcePath), sourcePath) }));
  const { sampleDefinitions, evidenceDefinitions, sourcePaths: compressionSourcePaths, ...compressionMetadata } = compression;
  const compressionEntry = { ...compressionMetadata, samples: compressionSamples, evidence: compressionEvidence, sources: compressionSources };
  const integrationEntries = integrations.map(integration => {
    const directory = integration.id === 'frontend' ? 'examples/frontend' : 'examples/mcp';
    const paths = sorted(sourcePaths).filter(sourcePath => sourcePath.startsWith(directory + '/'));
    return { ...integration, sources: paths.map(sourcePath => ({ sourcePath, ...put(`source/${sourcePath}`, read(sourcePath), sourcePath) })) };
  });
  let uiPackage = null;
  if (uiManifest) {
    const packageMetadata = readJson('examples/frontend/ui/package.json');
    assert.equal(uiManifest.name, packageMetadata.name); assert.equal(uiManifest.version, packageMetadata.version);
    assert.equal(uiManifest.nativeDependency, false, 'UI package must remain an injected renderer');
    const bytes = read(uiManifest.package.path); assert.equal(bytes.length, uiManifest.package.bytes); assert.equal(sha(bytes), uiManifest.package.sha256);
    const filename = path.basename(uiManifest.package.path);
    const download = { ...put(`packages/${filename}`, bytes, uiManifest.package.path), filename, version: uiManifest.version, kind: 'tgz', publication: 'local-alpha-artifact', native: false };
    const stylesheetSource = `examples/frontend/ui/${uiManifest.stylesheet}`;
    const stylesheet = integrationEntries.find(i => i.id === 'frontend').sources.find(s => s.sourcePath === stylesheetSource);
    assert(stylesheet, 'Optional UI stylesheet was not exported');
    uiPackage = { name: uiManifest.name, version: uiManifest.version, download, api: uiManifest.api, exports: uiManifest.exports,
      runtime: uiManifest.runtime, nativeDependency: false, stylesheetUrl: stylesheet.url, docId: 'ui-package',
      install: `npm install ./${filename}`, stylesheetImport: `${uiManifest.name}/styles.css`,
      compatibility: { engineVersion, schemaVersion: '1.0', model: 'original-engine-envelope',
        rationale: 'The optional renderer has an independent package version and consumes the original calculation envelope. Compressed astro-context/1 must be expanded before rendering.' },
      sourcePath: 'examples/frontend/ui/package.json', artifactManifest: uiManifest };
  }
  const declarations = put('source/index.d.ts', read('bindings/node/index.d.ts'), 'bindings/node/index.d.ts');
  const natalResult = readJson('examples/natal-result.json'), domainResult = readJson('examples/natal-domains-result.json'), coupleResult = readJson('examples/couple-result.json');
  assert.equal(domainResult.calculation.aspectPreset, 'extended', 'Extended preset fixture changed');
  assert(!readJson('examples/natal-request.json').aspectRules, 'Major preset fixture is no longer a default natal');
  const aspectPresets = { major: natalResult.calculation.aspectRules, extended: domainResult.calculation.aspectRules };
  const smokeIds = { natal: 'natal', natalDomains: 'natal-domains-lean', couple: 'couple-lean', composite: 'composite-lean', events: 'events-day', forecast: 'forecast-day-lean', query: 'query-between', chart: 'chart', harmonic: 'harmonic', synastry: 'synastry' };
  const operationEntries = operations.map(operation => ({ ...operation, smokeId: smokeIds[operation.id] }));
  const sampleGroups = operationEntries.map(operation => ({ id: operation.id, title: operation.title, smokeId: operation.smokeId, sampleIds: samples.filter(sample => sample.operation === operation.id).map(sample => sample.id) }));
  const catalog = { formatVersion: 1, version: engineVersion, product, operations: operationEntries, queryGroups, sampleGroups,
    domains: domainResult.data.profileCatalog, coupleDomains: coupleResult.data.profileCatalog, aspectPresets,
    aspectPresetSources: { major: 'examples/natal-result.json#calculation.aspectRules', extended: 'examples/natal-domains-result.json#calculation.aspectRules' },
    sdk: sdkEntries, integrations: integrationEntries, documents, schemas, samples, downloads, uiPackage, declarations, compression: compressionEntry,
    validation: { requestLimitBytes: 4 * 1024 * 1024, unknownFields: 'rejected', nullOptions: 'rejected except documented legacy geometry/speed fields',
      exactIntegerNormalization: 'Raw JSON decimal/exponent integers accepted; tiny fractions rejected before conversion',
      customRules: { angleRange: [0, 180], maxOrbRange: [0, 15], maximumRules: 16, uniqueAngles: true, emptyDisablesMatching: true },
      customProfiles: { maximumProfiles: 8, maximumSections: 8, selectors: ['houses', 'bodies', 'angles'], customOnly: 'domains: [] with nonempty valid customProfiles' },
      errorCodes: ['INVALID_INPUT', 'INPUT_TOO_LARGE', 'CALCULATION_FAILED', 'INTERNAL_ERROR'] },
    limitations, verification: testResults, artifactManifest: packageManifest,
  };
  putJson('catalog.json', catalog);
  const manifest = { formatVersion: 1, engineVersion, schemaVersion: '1.0', sourceDigest, sourceFiles,
    generator: 'astrology/scripts/export-website-data.mjs', nativeEvidence: { consumerFingerprint: testResults.consumerFingerprint, platform: packageManifest.platform, arch: packageManifest.arch, rid: packageManifest.rid },
    sampleCount: samples.length, compressionSampleCount: compressionSamples.length, operationCount: operations.length, queryActionCount: queryGroups.reduce((sum, g) => sum + g.methods.length, 0),
    files: sorted(exported.map(e => e.path)).map(relative => exported.find(e => e.path === relative)) };
  fs.writeFileSync(path.join(stage, 'manifest.json'), JSON.stringify(manifest) + '\n');
  const backup = outputRoot + `.previous-${process.pid}`;
  if (fs.existsSync(outputRoot)) fs.renameSync(outputRoot, backup);
  try { fs.renameSync(stage, outputRoot); } catch (error) { if (fs.existsSync(backup)) fs.renameSync(backup, outputRoot); throw error; }
  if (fs.existsSync(backup)) fs.rmSync(backup, { recursive: true });
  console.log(`Exported ${engineVersion}: ${samples.length} samples, ${operations.length} operations, eight query actions, ${documents.length} documents, ${downloads.length} verified packages → ${outputRoot}`);
} catch (error) {
  fs.rmSync(stage, { recursive: true, force: true }); throw error;
}
