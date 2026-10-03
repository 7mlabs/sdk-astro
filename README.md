![7mlabs Astrology — local computation, structured JSON](assets/readme-banner.svg)

# 7mlabs Astrology

An offline astrology engine with a shared Rust core and native SDKs. Turn birth data into structured chart, relationship and forecast payloads directly inside your application.

[![npm](https://img.shields.io/npm/v/%407mlabs%2Fastrology/latest?label=npm&color=afe8cc)](https://www.npmjs.com/package/@7mlabs/astrology)
[![CI](https://github.com/7mlabs/sdk-astro/actions/workflows/neutral-engine.yml/badge.svg)](https://github.com/7mlabs/sdk-astro/actions/workflows/neutral-engine.yml)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL--3.0-afe8cc)](LICENSE)

**[Documentation](https://sevenmlabs-packages.velikho.chatgpt.site/docs/astrology-node/) · [Explore the engine](https://sevenmlabs-packages.velikho.chatgpt.site/astro/) · [npm](https://www.npmjs.com/package/@7mlabs/astrology) · [Tiếng Việt](README.vi.md)**

- **Local calculation:** native binaries are included in the npm package. No engine server, compiler or runtime data download is required.
- **Structured results:** planets, houses, aspects, derived facts and evidence references in versioned JSON envelopes.
- **One calculation core:** Node.js, Python, .NET, Rust and C share the same Rust/Swiss Ephemeris implementation.
- **Context preparation:** optional payload compression for LLM workflows, with reversible compact encoding and explicit coverage metadata.

## Quick start

Install the stable npm release:

```sh
npm install @7mlabs/astrology
```

Save this as `chart.cjs`, then run `node chart.cjs`:

```js
const { calculate } = require('@7mlabs/astrology');

const birth = {
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 },
  houseSystem: 'placidus'
};

const chart = calculate({ operation: 'natal', ...birth });

console.log({
  sun: chart.data.placements.find(body => body.id === 'sun').sign,
  planets: chart.data.placements.length,
  houses: chart.data.houses.length
});
// { sun: 'Capricorn', planets: 10, houses: 12 }
```

Input time is **UTC**; convert local civil time before calling the SDK. Coordinates use degrees, with north/east positive. The response includes calculation metadata, warnings and errors alongside `data`. [View a complete request and response →](docs/natal.md)

TypeScript declarations are included. `calculate()` is synchronous and throws an error with `.code` and `.result` on calculation failure; `calculateJson()` returns a JSON string containing the envelope. Use a worker for long month/year scans in interactive applications.

## Capabilities

| Area | API | Result |
| --- | --- | --- |
| Natal charts | `natal` | Sun–Pluto, four angles, 12 houses and major aspects |
| Individual reports | `natalDomains` | Ten domain views, advanced facts, custom profiles and supporting evidence |
| Relationship comparison | `couple` | Two natal charts, synastry contacts, house overlays and six domain views |
| Composite charts | `composite` | A symbolic midpoint chart C with ten domain views |
| Astronomical events | `events` | Day/month/year searches for ingress, stations, Moon phases, aspects and global eclipses |
| Personal forecasts | `forecast` | Natal transits, event impacts, calendar overviews and domain-specific context |
| Geometry | `chart`, `harmonic`, `synastry` | Calculations from supplied positions |
| On-demand queries | `geometry`, `aspects`, `houses`, `points` | Eight helpers backed by the `query` operation |
| Payload compression | `compressPayload`, `calculateWithContext` | Compact, focused or budgeted context, with `expandContext` for decoding |

The engine returns calculated data and report context. Narrative interpretation and compatibility scoring belong to the consuming application. Individual domain reports use one person's chart; `couple` compares two people.

### Select individual domains

Using `birth` from the quick start:

```js
const report = calculate({
  operation: 'natalDomains',
  ...birth,
  domains: ['career', 'love']
});

console.log(report.data.domains.career.report.sections);
```

Built-in domains cover career, love, relationships, family, finance, identity, learning, creativity, inner life and daily life. [Domain payloads](docs/domains.md) · [Advanced reports](docs/individual-reports.md) · [Custom profiles](docs/profiles.md)

### Prepare LLM context

```js
const { compressPayload, expandContext } = require('@7mlabs/astrology');

const context = compressPayload(report, { mode: 'compact' });
const restored = expandContext(context);
// restored preserves the JSON values in report.
```

`focused` selects domains/sections; `budgeted` additionally checks delivery limits and reports when they are exceeded. Compression savings depend on the payload. [Compression contract and examples →](docs/payload-compression.md)

## Packages and platforms

| Language | Distribution | Status |
| --- | --- | --- |
| Node.js / TypeScript | [`@7mlabs/astrology`](https://www.npmjs.com/package/@7mlabs/astrology) | Public npm stable: `0.10.0` (`latest`) |
| Python | `sevenmlabs-astrology` | Local `0.10.0` wheels; PyPI release pending |
| .NET | `SevenMLabs.Astrology` | Local `0.10.0` NuGet artifacts; public release pending |
| Rust / C | Core crates and C ABI | Source integration examples |

The npm release includes **macOS ARM64** and **Linux x64** binaries. Linux requires **glibc 2.38+** and `libgcc_s.so.1`. Node.js 18/24 were tested on both targets; Node.js 22 or 24 is recommended for new applications. Windows, macOS Intel, Linux ARM64 and Alpine/musl binaries are not included. [Full platform requirements →](docs/node-release.md)

Update explicitly with `npm install @7mlabs/astrology@latest`. To pin this release:

```sh
npm install --save-exact @7mlabs/astrology@0.10.0
```

Commit your application's lockfile and use `npm ci` for reproducible installs. [Other language installation guides →](docs/packages.md)

## Documentation and examples

Most detailed guides are currently written in Vietnamese; API identifiers and JSON contracts are shared across languages.

| Topic | Guides |
| --- | --- |
| Charts and reports | [Natal](docs/natal.md) · [Domains](docs/domains.md) · [Advanced facts](docs/individual-reports.md) · [Profiles](docs/profiles.md) |
| Relationships | [Synastry](docs/couple.md) · [Composite](docs/composite.md) |
| Time and calculations | [Events / forecasts](docs/forecast.md) · [Queries](docs/query.md) · [API reference](docs/api.md) |
| Integrations | [Compression](docs/payload-compression.md) · [Local MCP](examples/mcp/README.md) · [Frontend host](examples/frontend/README.md) · [UI renderer](examples/frontend/ui/README.md) |
| Installation and development | [Packages](docs/packages.md) · [Architecture](docs/architecture.md) · [Build / release](docs/github-setup.md) · [Roadmap](docs/phases.md) |

[Runnable examples](examples/) include Node.js, Python, .NET, Rust and C. The local MCP adapter and frontend worker bridge are optional integrations. Browser-only calculation requires a separate WASM build, which is not part of this release.

## Calculation scope and validation

Birth calculations currently support Gregorian UTC **1800–2399**, tropical geocentric positions, and **Placidus** or **Whole Sign** houses. Swiss Ephemeris uses the bundled Moshier model. Timezone conversion is caller-managed; IANA timezone conversion, sidereal/topocentric charts, Davison, progressions and return charts are future work. [Calculation scope →](docs/natal.md)

The stable **0.10.0** tarball passed **339 conformance cases** on each of four macOS ARM64/Linux x64 × Node.js 18/24 combinations, along with query, compression and TypeScript checks. All seven build/validation jobs passed, and npm OIDC publication succeeded. The workflow’s post-publish registry check timed out during propagation; subsequent registry integrity checks and a fresh install on macOS ARM64 verified `0.10.0` on `latest`. [Release run](https://github.com/7mlabs/sdk-astro/actions/runs/37098509933) · [Validation details](docs/testing.md)

For source development, start with the [build instructions](docs/github-setup.md). Suggestions and reproducible bug reports are welcome in [GitHub Issues](https://github.com/7mlabs/sdk-astro/issues).

## License

Engine and SDKs are licensed under **[AGPL-3.0-only](LICENSE)**, using the free AGPL option of Swiss Ephemeris. License text and third-party notices are included in the packages. The independent UI renderer has a [separate MIT license](examples/frontend/ui/LICENSE). See [NOTICE](NOTICE) and the [license guide](docs/license.md) for details.
