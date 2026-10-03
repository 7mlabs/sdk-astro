// Human-readable catalog metadata. Numeric summaries are derived by the exporter.
export const product = {
  id: 'astrology', name: 'Astrology Engine', status: 'local-alpha',
  description: 'One Rust calculation core, local packages, and neutral JSON for natal charts, relationships, event calendars, focused queries and offline LLM context preparation.',
  publication: 'Local artifacts are available. No public registry release or new public GitHub remote is verified.',
  runtime: 'Native, offline, inside the caller process; no calculation server or credentials.',
  verifiedPlatform: 'macOS ARM64',
};
export const operations = [
  { id: 'natal', title: 'Basic natal chart', category: 'individual', input: 'Gregorian UTC, latitude/longitude, house system', output: 'Sun–Pluto positions, four angles, twelve houses and body aspects', docId: 'natal', schema: 'natal-request', sampleId: 'natal', defaults: ['Tropical, apparent geocentric', 'Placidus; Whole Sign available', 'Major aspect rules'], fields: ['data.placements', 'data.angles', 'data.houses', 'data.houseCusps', 'data.aspects'] },
  { id: 'natalDomains', title: 'Individual reports', category: 'individual', input: 'Natal input, domains, rulership and aspect/profile options', output: 'Shared natal/context, individual domains, report sections and selection evidence', docId: 'domains', schema: 'natal-domains-request', responseSchema: 'natal-domains-response', sampleId: 'natal-domains', defaults: ['All individual domains', 'Extended aspects', 'Traditional rulership'], fields: ['data.natal', 'data.context', 'data.domains', 'data.customDomains', 'data.profileCatalog'] },
  { id: 'couple', title: 'Two-person synastry', category: 'relationship', input: 'Independent birth inputs for A and B; optional composite', output: 'Two natal charts, cross-relations, house overlays, pair reports and optional chart C', docId: 'couple', schema: 'couple-request', responseSchema: 'couple-response', sampleId: 'couple', defaults: ['All pair domains', 'Extended aspects', 'Traditional rulership'], fields: ['data.subjects', 'data.context', 'data.domains', 'data.composite'] },
  { id: 'composite', title: 'Symbolic chart C', category: 'relationship', input: 'Two birth inputs and explicit midpoint/house policies', output: 'Recomputed midpoint placements, houses, aspects, individual report domains and per-point provenance', docId: 'composite', schema: 'composite-request', responseSchema: 'composite-response', sampleId: 'composite', defaults: ['Shortest-arc midpoint', 'Midpoint house cusps', 'Antipodal ambiguity is an error'], fields: ['data.subjects', 'data.composite.chart', 'data.composite.context', 'data.composite.domains', 'data.composite.provenance'] },
  { id: 'events', title: 'Sky event calendar', category: 'time', input: 'Day/month/year period with a fixed UTC offset, bodies and event families', output: 'Midpoint snapshot, exact events, eclipse maxima, coverage and numerical precision metadata', docId: 'forecast', schema: 'events-request', responseSchema: 'events-response', sampleId: 'events-year', defaults: ['All bodies and global families', 'Major aspects', 'Fixed offset 0'], fields: ['data.period', 'data.snapshot', 'data.events', 'data.coverage', 'calculation.search'] },
  { id: 'forecast', title: 'Personal calendar', category: 'time', input: 'One birth input and a day/month/year period', output: 'Shared natal, transit snapshot, exact natal contacts, personalImpact, domain views and messageContext', docId: 'forecast', schema: 'forecast-request', responseSchema: 'forecast-response', sampleId: 'forecast-day', defaults: ['Include exact natal transits', 'All individual domains', 'Major aspects'], fields: ['data.subject', 'data.snapshot', 'data.events[].personalImpact', 'data.domains', 'data.overview'] },
  { id: 'query', title: 'On-demand queries', category: 'query', input: 'A geometry/aspects/houses/points group and action-specific options', output: 'Small scalar results or a self-contained selection of natal facts', docId: 'query', schema: 'query-request', responseSchema: 'query-response', sampleId: 'query-between', defaults: ['Inspection: all selectors and major aspects', 'Between: motion mode none', 'No implicit cached chart'], fields: ['data.group', 'data.action', 'data.selection', 'data.relations', 'calculation.computationalScope'] },
  { id: 'chart', title: 'Supplied-position chart', category: 'geometry', input: 'Caller-supplied positions, optional ordered house cusps and aspect rules', output: 'Normalized placements, matching aspects and pair midpoints', docId: 'api', schema: 'geometry-request', sampleId: 'chart', defaults: ['Major aspect rules', 'Unknown speed remains null'], fields: ['data.placements', 'data.aspects', 'data.midpoints'] },
  { id: 'harmonic', title: 'Harmonic geometry', category: 'geometry', input: 'Supplied positions and an integer factor', output: 'Transformed longitudes/speeds and recomputed geometry; no physical sky or natal houses', docId: 'api', schema: 'geometry-request', sampleId: 'harmonic', defaults: ['Factor is required', 'Houses and chart B are rejected'], fields: ['data.harmonic', 'data.placements', 'data.aspects', 'data.midpoints', 'warnings'] },
  { id: 'synastry', title: 'Supplied-position synastry', category: 'geometry', input: 'Two caller-supplied position sets and optional cusps for each', output: 'Cross-aspects and A→B/B→A house overlays', docId: 'api', schema: 'geometry-request', sampleId: 'synastry', defaults: ['Cross-chart applying is null', 'No compatibility score'], fields: ['data.personA', 'data.personB', 'data.crossAspects', 'data.overlaysAtoB', 'data.overlaysBtoA'] },
];
export const queryGroups = [
  { id: 'geometry', title: 'Circular geometry', methods: [
    { id: 'normalize', input: ['longitude'], output: ['longitude', 'signIndex', 'sign', 'degreeInSign'], sampleId: 'query-normalize', scope: 'pureGeometry' },
    { id: 'separation', input: ['longitude1', 'longitude2'], output: ['signedDelta', 'separation'], sampleId: 'query-separation', scope: 'pureGeometry' },
    { id: 'midpoint', input: ['longitude1', 'longitude2', 'antipodalPolicy?'], output: ['longitude', 'antipodal', 'resolution'], sampleId: 'query-midpoint', scope: 'pureGeometry' },
  ] },
  { id: 'aspects', title: 'Custom angles and contacts', methods: [
    { id: 'between', input: ['positions[2]', 'rule', 'motionMode?'], output: ['relation', 'aspect.matched', 'aspect.applying', 'distanceRate'], sampleId: 'query-between', scope: 'pureGeometry' },
    { id: 'inspect', input: ['birth', 'pointIds?', 'rulership?', 'aspectPreset or aspectRules'], output: ['points', 'relations', 'aspects', 'coverage'], sampleId: 'query-aspects', scope: 'fullNatalThenSelection', selection: 'bothSelectedEndpoints' },
  ] },
  { id: 'houses', title: 'House placement and facts', methods: [
    { id: 'locate', input: ['longitude', 'houseCusps[12]'], output: ['house', 'boundaryRule'], sampleId: 'query-locate', scope: 'pureGeometry' },
    { id: 'inspect', input: ['birth', 'houseNumbers?', 'rulership?', 'aspectPreset or aspectRules'], output: ['houses', 'primaryPointIds', 'relations', 'aspects'], sampleId: 'query-houses', scope: 'fullNatalThenSelection', selection: 'atLeastOneSelectedEndpoint' },
  ] },
  { id: 'points', title: 'Natal point selection', methods: [
    { id: 'inspect', input: ['birth', 'pointIds?', 'rulership?', 'aspectPreset or aspectRules'], output: ['points', 'primaryPointIds', 'relations', 'aspects'], sampleId: 'query-points', scope: 'fullNatalThenSelection', selection: 'atLeastOneSelectedEndpoint' },
  ] },
];
export const documentDefinitions = [
  ['overview', 'README.md', 'Engine overview', 'Current capabilities, local packages and release status'],
  ['natal', 'docs/natal.md', 'Natal calculation', 'Birth input, provider, time scales and chart output'],
  ['domains', 'docs/domains.md', 'Individual domains', 'Shared context, house rulers and domain selection'],
  ['individual-reports', 'docs/individual-reports.md', 'Report facts', 'Conditions, chains, patterns, sections and evidence'],
  ['couple', 'docs/couple.md', 'Synastry from birth inputs', 'Namespaces, overlays, pair profiles and joins'],
  ['composite', 'docs/composite.md', 'Midpoint composite', 'Symbolic construction, houses, provenance and reports'],
  ['forecast', 'docs/forecast.md', 'Events and personal forecasts', 'Period windows, exact contacts, eclipses and message context'],
  ['query', 'docs/query.md', 'Grouped queries', 'Eight focused actions and compact inspection responses'],
  ['compression', 'docs/payload-compression.md', 'Payload compression', 'Shared context encoding, domain selection, explicit omissions and budgets'],
  ['profiles', 'docs/profiles.md', 'Custom profiles', 'Selectors, sections, limits and versioned definitions'],
  ['api', 'docs/api.md', 'JSON API', 'Operations, custom rules, geometry, envelopes and errors'],
  ['packages', 'docs/packages.md', 'Install packages', 'Real local artifact installation and language examples'],
  ['integrations', 'docs/integrations.md', 'MCP and UI adapters', 'Local stdio tools and a native worker bridge'],
  ['frontend', 'docs/frontend.md', 'Frontend roadmap', 'Rendering scope, input flows and the browser runtime gap'],
  ['testing', 'docs/testing.md', 'Verification', 'Installed-package parity, references and platform limits'],
  ['architecture', 'docs/architecture.md', 'Architecture', 'Shared Rust core, providers, ABI and bindings'],
  ['distribution', 'docs/distribution.md', 'Public release', 'Registry, source, license and target-matrix release gates'],
  ['phases', 'docs/phases.md', 'Migration phases', 'Completed work and future gates'],
  ['implementation', 'docs/implementation.md', 'Implementation plan', 'Source mapping, builds and release work'],
  ['mcp-example', 'examples/mcp/README.md', 'Run the MCP adapter', 'Install and launch the verified local stdio sample'],
  ['frontend-example', 'examples/frontend/README.md', 'Run the worker bridge', 'Native calculation in a queued application worker'],
].map(([id, sourcePath, title, description]) => ({ id, sourcePath, title, description }));
export const savedSamples = [
  ['natal', 'natal', 'Basic natal'], ['natal-domains', 'natal-domains', 'All individual domains'],
  ['custom-profiles', 'custom-profiles', 'Custom individual profile'], ['couple', 'couple', 'All pair domains'],
  ['custom-couple', 'custom-couple', 'Custom pair profile'], ['composite', 'composite', 'Midpoint chart C'],
  ['custom-composite', 'custom-composite', 'Custom composite profile'], ['couple-composite', 'couple-composite', 'Synastry plus chart C'],
  ['events-year', 'events-year', 'Year sky calendar'], ['forecast-day', 'forecast-day', 'Personal day'],
  ['forecast-month', 'forecast-month', 'Personal month'], ['forecast-year', 'forecast-year', 'Personal year with selected slow bodies'],
  ['custom-forecast', 'custom-forecast', 'Custom personal calendar'], ['query-houses', 'query-houses', 'Selected natal houses'],
  ['query-points', 'query-points', 'Selected body, angle and cusp'], ['query-aspects', 'query-aspects', 'A custom natal angle'],
  ['query-between', 'query-between', 'Two supplied positions and a custom angle'],
].map(([id, stem, title]) => ({ id, title, requestPath: `examples/${stem}-request.json`, resultPath: `examples/${stem}-result.json`, origin: 'saved-installed-package-response' }));
const positions = [{ id: 'moon', longitude: 359, speed: 13 }, { id: 'sun', longitude: 1, speed: 1 }];
export const calculatedSamples = [
  { id: 'events-day', title: 'One-day sky calendar', request: { operation: 'events', period: { kind: 'day', year: 2026, month: 3, day: 3, utcOffsetMinutes: 420 } }, sourcePath: 'docs/forecast.md', inputKind: 'period-input' },
  { id: 'natal-domains-lean', title: 'One individual domain', baseRequestPath: 'examples/natal-domains-request.json', overrides: { domains: ['career'], aspectPreset: 'major' }, sourcePath: 'docs/domains.md', inputKind: 'birth-input' },
  { id: 'couple-lean', title: 'One pair domain', baseRequestPath: 'examples/couple-request.json', overrides: { domains: ['communication'], aspectPreset: 'major' }, sourcePath: 'docs/couple.md', inputKind: 'two-birth-inputs' },
  { id: 'composite-lean', title: 'One composite domain', baseRequestPath: 'examples/composite-request.json', overrides: { domains: ['love'], aspectPreset: 'major' }, sourcePath: 'docs/composite.md', inputKind: 'two-birth-inputs' },
  { id: 'forecast-day-lean', title: 'Personal day with one domain', baseRequestPath: 'examples/forecast-day-request.json', overrides: { domains: ['career'] }, sourcePath: 'docs/forecast.md', inputKind: 'birth-and-period-input' },
  { id: 'chart', title: 'Chart from supplied positions', request: { operation: 'chart', positions }, sourcePath: 'docs/api.md' },
  { id: 'harmonic', title: 'Fifth harmonic from supplied positions', request: { operation: 'harmonic', harmonic: 5, positions: [{ id: 'a', longitude: 0 }, { id: 'b', longitude: 72 }] }, sourcePath: 'docs/api.md' },
  { id: 'synastry', title: 'Synastry from supplied positions', request: { operation: 'synastry', positions: [{ id: 'a', longitude: 10 }], otherPositions: [{ id: 'b', longitude: 130 }] }, sourcePath: 'docs/api.md' },
  { id: 'query-normalize', title: 'Normalize a supplied longitude', request: { operation: 'query', group: 'geometry', action: 'normalize', longitude: -10 }, sourcePath: 'docs/query.md' },
  { id: 'query-separation', title: 'Separation across zodiac zero', request: { operation: 'query', group: 'geometry', action: 'separation', longitude1: 359, longitude2: 1 }, sourcePath: 'docs/query.md' },
  { id: 'query-midpoint', title: 'Shortest-arc midpoint across zero', request: { operation: 'query', group: 'geometry', action: 'midpoint', longitude1: 350, longitude2: 10 }, sourcePath: 'schemas/query-request.schema.json' },
  { id: 'query-locate', title: 'Locate a supplied longitude in supplied cusps', request: { operation: 'query', group: 'houses', action: 'locate', longitude: 350, houseCusps: [350, 20, 50, 80, 110, 140, 170, 200, 230, 260, 290, 320] }, sourcePath: 'schemas/query-request.schema.json' },
  { id: 'error-antipodal', title: 'Ambiguous midpoint error', request: { operation: 'query', group: 'geometry', action: 'midpoint', longitude1: 0, longitude2: 180 }, expectedError: 'CALCULATION_FAILED', sourcePath: 'docs/query.md' },
  { id: 'error-calendar', title: 'Invalid Gregorian birth date', request: { operation: 'natal', utc: { year: 2025, month: 2, day: 29, hour: 12, minute: 0 }, location: { latitude: 10.8231, longitude: 106.6297 } }, expectedError: 'INVALID_INPUT', sourcePath: 'docs/natal.md' },
].map(s => ({ ...s, origin: 'native-calculated-example', inputKind: s.inputKind || (s.request?.operation === 'natal' ? 'invalid-birth-input' : 'supplied-geometry') }));
export const languageDefinitions = [
  { id: 'node', name: 'Node.js / TypeScript', packageName: '@7mlabs/astrology', artifactKind: 'tgz', docId: 'packages', api: 'calculate / calculateJson + geometry/aspects/houses/points + compressPayload/expandContext/calculateWithContext', examples: ['examples/node/sample.cjs', 'examples/node/query.cjs', 'examples/node/forecast.cjs', 'examples/node/compression.cjs'], status: 'verified-local-native-package' },
  { id: 'python', name: 'Python', packageName: 'sevenmlabs-astrology', artifactKind: 'whl', docId: 'packages', api: 'calculate / calculate_json + grouped namespace objects + compress_payload/expand_context/calculate_with_context', examples: ['examples/python/sample.py', 'examples/python/query.py', 'examples/python/forecast.py', 'examples/python/compression.py'], status: 'verified-local-native-package' },
  { id: 'dotnet', name: '.NET', packageName: 'SevenMLabs.Astrology', artifactKind: 'nupkg', docId: 'packages', api: 'Engine.Calculate / CalculateJson + Engine.Geometry/Aspects/Houses/Points', examples: ['examples/dotnet/Example.csproj', 'examples/dotnet/Program.cs'], status: 'verified-local-native-package' },
  { id: 'rust', name: 'Rust', api: 'astro_core::calculate_json', examples: ['examples/rust/Cargo.toml', 'examples/rust/src/main.rs'], docId: 'packages', status: 'verified-source-path-dependency', note: 'No crates.io release is verified.' },
  { id: 'c', name: 'C', api: 'astro_calculate_json / astro_free_string', examples: ['examples/c/main.c', 'neutral-engine/include/astro_engine.h'], docId: 'packages', status: 'verified-source-abi-example', note: 'Caller links the native library and owns the input string.' },
];
export const compression = {
  format: 'astro-context/1', docId: 'compression',
  runtime: 'Offline postprocessing in the shared Rust core; it does not calculate a chart or call an LLM.',
  inputLimitBytes: 256 * 1024 * 1024,
  tokenEstimateMethod: 'utf8-bytes-conservative',
  tokenCounting: 'Byte measurements are not exact model tokenizer measurements. Final prompt overhead is not included.',
  modes: [
    { id: 'compact', title: 'Keep all JSON values', description: 'Encode repeated values and homogeneous arrays. Native expansion restores the complete source envelope.' },
    { id: 'focused', title: 'Select report domains or sections', description: 'Keep selected views and complete shared supporting contexts. Publish every semantic omission.' },
    { id: 'budgeted', title: 'Check a delivery budget', description: 'Apply the same scope selection and report exceeded byte/token estimates. Never drop arbitrary facts to force a fit.' },
  ],
  methods: [
    { language: 'node', compress: 'compressPayload', expand: 'expandContext', calculateWithContext: 'calculateWithContext', rawCompress: 'compressPayloadJson', rawExpand: 'expandContextJson' },
    { language: 'python', compress: 'compress_payload', expand: 'expand_context', calculateWithContext: 'calculate_with_context', rawCompress: 'compress_payload_json', rawExpand: 'expand_context_json' },
    { language: 'dotnet', compress: 'Engine.CompressPayload', expand: 'Engine.ExpandContext', calculateWithContext: 'Engine.CalculateWithContext', rawCompress: 'Engine.CompressPayloadJson', rawExpand: 'Engine.ExpandContextJson' },
    { language: 'rust', compress: 'astro_core::compress_json', expand: 'astro_core::expand_context_json', calculateWithContext: null, rawCompress: 'astro_core::compress_json', rawExpand: 'astro_core::expand_context_json' },
    { language: 'c', compress: 'astro_compress_json', expand: 'astro_expand_context_json', calculateWithContext: null, rawCompress: 'astro_compress_json', rawExpand: 'astro_expand_context_json' },
  ],
  sampleDefinitions: [
    { id: 'compression-compact', title: 'All individual domains — compact', sourceSampleId: 'natal-domains', options: { mode: 'compact' } },
    { id: 'compression-focused', title: 'Career domain — focused', sourceSampleId: 'natal-domains', options: { mode: 'focused', domains: ['career'] } },
    { id: 'compression-budgeted', title: 'Career domain — budget exceeded', sourceSampleId: 'natal-domains', options: { mode: 'budgeted', domains: ['career'], maxBytes: 1024 } },
  ],
  evidenceDefinitions: [
    { id: 'compression-parity', sourcePath: 'artifacts/compression-test-results.json' },
    { id: 'compression-sdk-helpers', sourcePath: 'artifacts/compression-binding-tests.json' },
    { id: 'compression-mcp', sourcePath: 'artifacts/compression-mcp-tests.json' },
    { id: 'mcp-calculation-regression', sourcePath: 'artifacts/compression-mcp-regression.json' },
  ],
  sourcePaths: ['bindings/dotnet/Engine.cs', 'bindings/python/sevenmlabs_astrology/__init__.py', 'neutral-engine/include/astro_engine.h', 'neutral-engine/crates/astro-core/src/lib.rs', 'neutral-engine/crates/astro-core/src/compression.rs'],
};
export const integrations = [
  { id: 'mcp', title: 'Local MCP adapter', status: 'verified-native-stdio-example', docId: 'integrations', exampleDocId: 'mcp-example', tools: ['astro_geometry', 'astro_aspects', 'astro_houses', 'astro_points', 'astro_calculate', 'astro_compress_payload', 'astro_calculate_context'], transport: 'stdio', dependency: '@modelcontextprotocol/sdk 1.29.0', sourcePaths: ['examples/mcp/server.cjs', 'examples/mcp/package.json', 'examples/mcp/smoke.cjs'] },
  { id: 'frontend', title: 'Native worker bridge', status: 'verified-node-worker-example', docId: 'integrations', exampleDocId: 'frontend-example', sourcePaths: ['examples/frontend/bridge.cjs', 'examples/frontend/worker.cjs', 'examples/frontend/smoke.cjs'], browser: 'A static browser can inspect stored JSON; native calculation requires a host. Browser WASM is not implemented.' },
];
export const limitations = [
  'No scoring, compatibility rating, generated narrative or guaranteed life-event prediction.',
  'Birth time is Gregorian UTC. The application converts local time; no IANA/DST lookup is provided.',
  'Only tropical apparent geocentric Sun–Pluto, Placidus and Whole Sign are implemented.',
  'Composite C is symbolic: no physical timestamp, latitude, distance or instantaneous motion.',
  'Eclipses are global maxima; local visibility is not calculated.',
  'Browser WASM, Davison, progressions, returns, nodes, asteroids and fixed stars are not implemented.',
  'Native install and adapter verification is limited to the documented macOS ARM64 environment.',
  'Registry names are provisional and public publication is not verified.',
];
