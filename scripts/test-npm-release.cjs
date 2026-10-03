#!/usr/bin/env node
'use strict';
// Install and verify the final merged tarball, without network or runtime dependencies.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const crypto = require('node:crypto');
const zlib = require('node:zlib');
const vm = require('node:vm');
const { createRequire } = require('node:module');
const { spawnSync } = require('node:child_process');
const root = path.resolve(__dirname, '..');
const sha = value => crypto.createHash('sha256').update(value).digest('hex');
const json = filename => JSON.parse(fs.readFileSync(filename, 'utf8'));
const MAX_BYTES = 256 * 1024 * 1024;
let failureReportPath;
let failureContext;
let conformanceCasesPassed = 0;

function releaseTag(version) {
  const number = '(?:0|[1-9][0-9]*)';
  const match = typeof version === 'string' && new RegExp(`^${number}\\.${number}\\.${number}(-alpha\\.${number})?$`).exec(version);
  assert(match && match[0] === version, 'Only stable x.y.z or alpha x.y.z-alpha.N versions may publish');
  return match[1] ? 'alpha' : 'latest';
}

function options(argv) {
  const allowed = new Set(['manifest', 'reference-cli', 'typescript', 'ajv-module', 'npm-cli', 'report']);
  const result = {};
  for (let index = 0; index < argv.length; index += 2) {
    assert(argv[index].startsWith('--') && allowed.has(argv[index].slice(2)) && argv[index + 1], 'Unknown/missing CLI option');
    const key = argv[index].slice(2);
    assert(!Object.hasOwn(result, key), `Duplicate --${key}`);
    result[key] = path.resolve(argv[index + 1]);
  }
  for (const key of ['manifest', 'reference-cli', 'typescript', 'ajv-module']) {
    assert(result[key] && fs.statSync(result[key]).isFile(), `Pass --${key} /absolute/path`);
  }
  if (!result['npm-cli'] && process.env.NPM_CLI) result['npm-cli'] = path.resolve(process.env.NPM_CLI);
  if (result['npm-cli']) assert(fs.statSync(result['npm-cli']).isFile(), 'npm CLI is missing');
  return result;
}

function run(command, args, cwd, env = {}, timeout = 180000) {
  const cleanEnv = { ...process.env, ...env };
  delete cleanEnv.NODE_PATH;
  delete cleanEnv.NODE_OPTIONS;
  const result = spawnSync(command, args, { cwd, env: cleanEnv, encoding: 'utf8', maxBuffer: MAX_BYTES, timeout });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${path.basename(command)} failed (${result.status}):\n${result.stderr}\n${result.stdout}`);
  return result.stdout;
}

function archiveEntries(bytes) {
  const tar = zlib.gunzipSync(bytes, { maxOutputLength: MAX_BYTES });
  const entries = new Map();
  const string = bytes => bytes.toString('utf8').replace(/\0.*$/s, '');
  for (let offset = 0; offset < tar.length;) {
    assert(offset + 512 <= tar.length, 'Truncated tar header');
    const header = tar.subarray(offset, offset + 512);
    if (header.every(byte => byte === 0)) {
      assert(tar.subarray(offset).every(byte => byte === 0), 'Nonzero data after tar terminator');
      return entries;
    }
    const checksum = parseInt(string(header.subarray(148, 156)).trim(), 8);
    let total = 0;
    for (let index = 0; index < 512; index++) total += index >= 148 && index < 156 ? 32 : header[index];
    assert.equal(total, checksum, 'Invalid tar header checksum');
    assert.equal(string(header.subarray(257, 263)), 'ustar', 'Expected deterministic USTAR archive');
    const prefix = string(header.subarray(345, 500));
    const name = (prefix ? `${prefix}/` : '') + string(header.subarray(0, 100));
    assert(name.startsWith('package/') && !name.includes('\\') && !name.split('/').some(part => ['..', '.'].includes(part)), 'Unsafe archive path');
    assert.equal(header[156], 48, 'Archive must only contain regular files');
    assert(!entries.has(name), `Duplicate tar member: ${name}`);
    const size = parseInt(string(header.subarray(124, 136)).trim(), 8);
    assert(Number.isSafeInteger(size) && size >= 0 && offset + 512 + size <= tar.length, 'Invalid archive size');
    entries.set(name, tar.subarray(offset + 512, offset + 512 + size));
    offset += 512 + Math.ceil(size / 512) * 512;
  }
  throw new Error('Missing tar terminator');
}

function checkNative(bytes, target) {
  assert(bytes.length >= 64, 'Native binary truncated');
  assert.equal(bytes.length, target.bytes);
  assert.equal(sha(bytes), target.sha256);
  if (target.platform === 'darwin') {
    assert.equal(target.arch, 'arm64');
    assert.deepEqual([...bytes.subarray(0, 4)], [207, 250, 237, 254]);
    assert.equal(bytes.readUInt32LE(4), 0x0100000c);
    assert.equal(bytes.readUInt32LE(12), 6);
  } else {
    assert.equal(target.platform, 'linux');
    assert.equal(target.arch, 'x64');
    assert.deepEqual([...bytes.subarray(0, 6)], [127, 69, 76, 70, 2, 1]);
    assert.equal(bytes.readUInt16LE(16), 3);
    assert.equal(bytes.readUInt16LE(18), 62);
  }
}

const typeChecks = `import { calculate, calculateJson, calculateWithContext, compressPayload,
  compressPayloadJson, expandContext, expandContextJson, geometry, aspects, houses, points,
  GeometryNormalizeResult, Result, ContextResult, CompressionOptions } from '@7mlabs/astrology';
const normalized: GeometryNormalizeResult = geometry.normalize({longitude: -10});
const number: number = normalized.data.longitude;
const birth = {utc: {year: 2000, month: 1, day: 1, hour: 12, minute: 0}, location: {latitude: 10, longitude: 106}};
const natal: Result = calculate({...birth, operation: 'natal'});
const pair = calculateWithContext({...birth, operation: 'natalDomains'}, {mode: 'focused', domains: ['career']});
const options: CompressionOptions = {mode: 'budgeted', maxBytes: 1000};
const context: ContextResult = compressPayload(pair.result, options);
const format: 'astro-context/1' = context.data.format;
const restored = expandContext(context);
const raw: string = expandContextJson(compressPayloadJson(calculateJson(JSON.stringify(birth))));
houses.inspect({birth, houseNumbers: [1, 7]});
points.inspect({birth, pointIds: ['sun', 'H7']});
aspects.inspect({birth, pointIds: ['sun', 'moon']});
// @ts-expect-error Unsupported compressor modes must not type-check.
compressPayload(pair.result, {mode: 'lossy'});
// @ts-expect-error Raw APIs require JSON strings.
compressPayloadJson(pair.result);
// @ts-expect-error Unknown query points must not type-check.
points.inspect({birth, pointIds: ['unknown']});
void [number, natal, format, restored, raw];
`;

function main() {
  const args = options(process.argv.slice(2));
  const manifest = json(args.manifest);
  failureReportPath = args.report ?? path.join(path.dirname(args.manifest), `test-${process.platform}-${process.arch}.json`);
  failureContext = { schemaVersion: '1.0', name: manifest.name, engineVersion: manifest.engineVersion,
    packageSha256: manifest.sha256, releaseManifestSha256: sha(fs.readFileSync(args.manifest)),
    sourceCommit: manifest.source?.commitSha, platform: process.platform, arch: process.arch, nodeVersion: process.version };
  assert.equal(manifest.schemaVersion, '1.0');
  assert.equal(manifest.name, '@7mlabs/astrology');
  assert.equal(manifest.version, manifest.engineVersion);
  assert.equal(manifest.abiVersion, 1);
  const tag = releaseTag(manifest.version);
  assert.equal(manifest.tag, tag);
  assert.equal(manifest.access, 'public');
  assert.match(manifest.source.commitSha, /^[0-9a-f]{40}$/);
  assert.equal(manifest.source.repository, '7mlabs/sdk-astro');
  assert.equal(path.basename(manifest.filename), manifest.filename);
  const artifact = path.join(path.dirname(args.manifest), manifest.filename);
  const bytes = fs.readFileSync(artifact);
  assert.equal(bytes.length, manifest.bytes);
  assert.equal(sha(bytes), manifest.sha256);
  const entries = archiveEntries(bytes);
  const metadata = JSON.parse(entries.get('package/package.json'));
  assert.equal(metadata.name, manifest.name);
  assert.equal(metadata.version, manifest.version);
  assert(!Object.hasOwn(metadata, 'private'), 'Release must remove private guard');
  assert.deepEqual(metadata.publishConfig, { access: 'public', tag });
  assert.equal(metadata.license, 'AGPL-3.0-only');
  assert.deepEqual(metadata.os, ['darwin', 'linux']);
  assert.deepEqual(metadata.cpu, ['arm64', 'x64']);
  for (const key of ['scripts', 'dependencies', 'optionalDependencies', 'peerDependencies', 'bundledDependencies', 'bundleDependencies']) {
    assert(!metadata[key] || Object.keys(metadata[key]).length === 0, `Unexpected runtime install mechanism: ${key}`);
  }
  const targetNames = manifest.targets.map(target => `${target.platform}-${target.arch}`).sort();
  assert.deepEqual(targetNames, ['darwin-arm64', 'linux-x64']);
  const nativePaths = manifest.targets.map(target => `package/${target.path}`).sort();
  assert.deepEqual([...entries.keys()].filter(name => name.startsWith('package/native/')).sort(), nativePaths);
  for (const target of manifest.targets) checkNative(entries.get(`package/${target.path}`), target);
  const portablePaths = [...entries.keys()].filter(name => !name.startsWith('package/native/')).map(name => name.slice(8)).sort();
  assert.deepEqual(portablePaths, Object.keys(manifest.portableFiles).sort());
  for (const name of portablePaths) assert.equal(sha(entries.get(`package/${name}`)), manifest.portableFiles[name], name);
  assert.deepEqual(entries.get('package/LICENSE'), fs.readFileSync(path.join(root, 'LICENSE')));
  for (const name of ['index.cjs', 'index.d.ts', 'LICENSE', 'NOTICE']) assert(entries.has(`package/${name}`), name);
  const target = manifest.targets.find(target => target.platform === process.platform && target.arch === process.arch);
  assert(target, `Release verification requires a supported host; got ${process.platform}-${process.arch}`);
  const consumerRoot = path.join(path.dirname(args.manifest), 'consumers');
  fs.mkdirSync(consumerRoot, { recursive: true });
  const consumer = fs.mkdtempSync(path.join(consumerRoot, `${process.platform}-${process.arch}-`));
  fs.writeFileSync(path.join(consumer, 'package.json'), JSON.stringify({ private: true, name: 'npm-release-consumer', version: '1.0.0' }));
  const npmArgs = ['install', '--offline', '--ignore-scripts', '--no-audit', '--no-fund', '--cache', path.join(consumer, 'npm-cache'), artifact];
  run(args['npm-cli'] ? process.execPath : 'npm', args['npm-cli'] ? [args['npm-cli'], ...npmArgs] : npmArgs, consumer);
  const installedRequire = createRequire(path.join(consumer, 'package.json'));
  const installed = path.join(consumer, 'node_modules/@7mlabs/astrology');
  const sdk = installedRequire('@7mlabs/astrology');
  const installedNative = path.join(installed, target.path);
  assert(require.cache[installedNative], 'Addon must load from freshly installed package');
  assert.equal(sha(fs.readFileSync(installedNative)), target.sha256);
  for (const [name, value] of entries) {
    assert.deepEqual(fs.readFileSync(path.join(installed, name.slice(8))), value, `Installed file differs: ${name}`);
  }
  // Simulate a genuinely unsupported target while retaining the installed loader.
  for (const [platform, arch] of [['win32', 'x64'], ['darwin', 'x64'], ['linux', 'arm64']]) {
    assert.throws(() => vm.runInNewContext(fs.readFileSync(path.join(installed, 'index.cjs'), 'utf8'), {
      require: createRequire(path.join(installed, 'index.cjs')), module: { exports: {} },
      __dirname: installed, process: { platform, arch }
    }), new RegExp(`No usable astrology binary for ${platform}-${arch}`));
  }
  console.log(`PASS installed ${manifest.name}@${manifest.version} (${process.platform}-${process.arch})`);
  const fixtureNames = ['natal', 'natal-domains', 'couple', 'composite', 'couple-composite',
    'forecast-day', 'forecast-month', 'forecast-year', 'events-year', 'custom-profiles',
    'custom-couple', 'custom-composite', 'custom-forecast'];
  const requests = fixtureNames.map(name => json(path.join(root, 'examples', `${name}-request.json`)));
  const wanted = new Set(requests.map(request => JSON.stringify(request)));
  const retained = new Map();
  const casesPath = path.join(root, 'tests/conformance/cases.json');
  const cases = json(casesPath);
  assert(cases.length >= 339, 'Full conformance set is required');
  for (const [index, fixture] of cases.entries()) {
    const raw = fixture.rawRequest ?? JSON.stringify(fixture.request);
    const actual = JSON.parse(sdk.calculateJson(raw));
    const reference = JSON.parse(run(args['reference-cli'], [raw], root));
    assert.deepEqual(actual, reference, `${fixture.name}: installed tarball differs from Rust reference`);
    assert.equal(actual.engineVersion, manifest.engineVersion, fixture.name);
    assert.equal(actual.schemaVersion, '1.0', fixture.name);
    assert.equal(actual.errors.length > 0, fixture.error ?? false, fixture.name);
    conformanceCasesPassed++;
    if (!actual.errors.length && fixture.request) {
      const key = JSON.stringify(fixture.request);
      if (wanted.has(key)) retained.set(key, actual);
    }
    if ((index + 1) % 25 === 0 || index + 1 === cases.length) console.log(`PASS parity ${index + 1}/${cases.length}`);
  }
  const birth = { utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 }, location: { latitude: 10.8231, longitude: 106.6297 } };
  const queryCalls = [
    ['geometry', 'normalize', { longitude: -10 }],
    ['geometry', 'separation', { longitude1: 350, longitude2: 10 }],
    ['geometry', 'midpoint', { longitude1: 350, longitude2: 10 }],
    ['aspects', 'between', { positions: [{ id: 'a', longitude: 10 }, { id: 'b', longitude: 87 }], rule: { angle: 77, maxOrb: 0 } }],
    ['houses', 'locate', { longitude: 350, houseCusps: Array.from({ length: 12 }, (_, index) => index * 30) }],
    ['houses', 'inspect', { birth, houseNumbers: [1, 7] }],
    ['points', 'inspect', { birth, pointIds: ['sun', 'H7'] }],
    ['aspects', 'inspect', { birth, pointIds: ['sun', 'moon'] }]
  ];
  const queryFiles = [];
  for (const [index, [group, action, input]] of queryCalls.entries()) {
    const snapshot = JSON.stringify(input);
    const result = sdk[group][action](input);
    assert.deepEqual(result, sdk.calculate({ ...input, operation: 'query', group, action }));
    assert.equal(JSON.stringify(input), snapshot, 'Query helper mutated input');
    assert.throws(() => sdk[group][action]({ ...input, operation: 'query' }), TypeError);
    const filename = path.join(consumer, `query-${index}.json`);
    fs.writeFileSync(filename, JSON.stringify(result));
    queryFiles.push(filename);
  }
  assert.equal(sdk.geometry.normalize({ longitude: -10 }).data.longitude, 350);
  assert.equal(sdk.geometry.separation({ longitude1: 350, longitude2: 10 }).data.separation, 20);
  assert.equal(sdk.houses.locate({ longitude: 350, houseCusps: Array.from({ length: 12 }, (_, index) => index * 30) }).data.house.number, 12);
  const schemaEnv = { AJV_2020_MODULE: args['ajv-module'], ASTRO_ENGINE_MODULE: installed };
  const querySchema = run(process.execPath, [path.join(root, 'scripts/check-query-schema.cjs'), ...queryFiles], consumer, schemaEnv);
  console.log(querySchema.trim());
  const payloadFiles = requests.map((request, index) => {
    const result = retained.get(JSON.stringify(request)) ?? sdk.calculate(request);
    assert.deepEqual(result.errors, []);
    assert.equal(result.engineVersion, manifest.engineVersion);
    const filename = path.join(consumer, `${fixtureNames[index]}-result.json`);
    fs.writeFileSync(filename, JSON.stringify(result));
    return filename;
  });
  const compressionSchema = JSON.parse(run(process.execPath, [path.join(root, 'scripts/check-compression-schema.cjs'), ...payloadFiles], consumer, schemaEnv));
  assert.equal(compressionSchema.result, 'passed');
  assert.equal(compressionSchema.realFixtureRoundtrips, fixtureNames.length);
  const original = sdk.calculate({ ...birth, operation: 'natal' });
  const snapshot = JSON.stringify(original);
  const pair = sdk.calculateWithContext({ ...birth, operation: 'natal' });
  assert.deepEqual(pair.result, original);
  assert.deepEqual(sdk.expandContext(pair.context), original);
  assert.equal(JSON.stringify(original), snapshot);
  assert.equal(sdk.compressPayload(original, { mode: 'budgeted', maxTokens: 1 }).data.budget.exceeded, true);
  for (const invalid of [NaN, Infinity, undefined, () => 1, Symbol('fact'), 1n]) {
    assert.throws(() => sdk.compressPayload({ ...original, data: { invalid } }), TypeError);
  }
  assert.throws(() => sdk.compressPayload(original, { mode: 'invalid' }), error => !!error.code && !!error.result);
  assert.throws(() => sdk.compressPayloadJson(snapshot + '\0'), TypeError);
  fs.writeFileSync(path.join(consumer, 'release-types.ts'), typeChecks);
  run(process.execPath, [args.typescript, '--strict', '--noEmit', '--target', 'ES2022', '--module', 'node16',
    '--moduleResolution', 'node16', 'release-types.ts'], consumer);
  const report = {
    schemaVersion: '1.0', result: 'passed', name: manifest.name, engineVersion: manifest.engineVersion,
    packageSha256: manifest.sha256, releaseManifestSha256: sha(fs.readFileSync(args.manifest)),
    sourceCommit: manifest.source.commitSha, platform: process.platform, arch: process.arch,
    nodeVersion: process.version, hostRelease: os.release(), consumerDirectory: consumer,
    loadedNativePath: installedNative, nativeSha256: target.sha256,
    archivedNativeTargets: targetNames, conformanceCases: cases.length, conformanceSha256: sha(fs.readFileSync(casesPath)),
    referenceCliSha256: sha(fs.readFileSync(args['reference-cli'])), queryHelpers: queryCalls.length,
    querySchemaAndSemanticChecks: querySchema.trim(), compressionSchemaAndRoundtripChecks: compressionSchema,
    typescript: 'passed', unsupportedTargetsRejected: ['win32-x64', 'darwin-x64', 'linux-arm64'],
    installation: 'fresh-consumer-offline-ignore-scripts', runtimeDownloads: false
  };
  fs.writeFileSync(failureReportPath, JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report, null, 2));
}

try { main(); } catch (error) {
  if (failureReportPath && failureContext) {
    fs.writeFileSync(failureReportPath, JSON.stringify({ ...failureContext, result: 'failed',
      conformanceCasesPassed, error: error.message.slice(0, 4000) }, null, 2) + '\n');
  }
  console.error(`npm release test failed: ${error.stack}`);
  process.exitCode = 1;
}
