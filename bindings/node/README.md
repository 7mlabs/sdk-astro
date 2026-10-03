# Node.js astrology package

Offline natal, synastry, composite, forecasts and astronomical event data using the shared Rust/Swiss native core. Uses Node-API directly inside the Node process.

## Install and update

```sh
npm install @7mlabs/astrology@alpha
```

The alpha channel is updated explicitly with the same command. Pin `@7mlabs/astrology@0.10.0-alpha.1` and commit your application's lockfile to reproduce a specific release; `npm ci` restores that lockfile. New applications should use Node.js 22 or 24. Minimum supported Node.js is 18.

The public tarball includes both `darwin-arm64` (Apple Silicon) and `linux-x64` (glibc **2.38+**, with `libgcc_s.so.1`) addons. macOS 15 and Ubuntu 24.04 are the verified build/test environments. The macOS addon targets macOS 11+, but your Node.js version can require a newer OS. Windows, macOS Intel, Linux ARM64 and Alpine/musl are not included. See [platform and release details](https://github.com/7mlabs/sdk-astro/blob/main/docs/node-release.md).

No compiler, Rust installation, engine server or runtime binary download is needed. The installed addon calculates locally; it requires a supported native Node.js host.

```js
const { calculate } = require('@7mlabs/astrology');
const result = calculate({ operation: 'natal',
  utc: { year: 2000, month: 1, day: 1, hour: 12, minute: 0 },
  location: { latitude: 10.8231, longitude: 106.6297 }
});
console.log(result.data.placements, result.data.houses);
```

Package includes TypeScript declarations. `calculate` is synchronous and throws errors with `.code`/`.result`; `calculateJson` returns the envelope including errors. Native Node package does not run in a browser. Maximum input: 4 MiB; geometry permits 1–64 bodies. Source developers can build guarded local candidates with `scripts/build-packages.py`; the public release assembler combines two validated candidates from the same commit.

Operations: natal, natalDomains, couple, composite, events, forecast, chart, harmonic, synastry. `natalDomains` uses the same UTC/location input and returns `data.natal`, `data.context`, `data.domains` for career/love/relationships/family/finance/identity/learning/creativity/innerLife/dailyLife. It defaults to all ten domains, traditional rulers and extended aspect rules. Optional `domains` selects unique IDs; `rulership` selects traditional/modern; `aspectPreset` selects major/extended or supply `aspectRules` (mutually exclusive). Context includes all 325 angle relations among 10 bodies, 4 angles and 12 house cusps, house rulers/occupants and dispositors. Domain selection reasons and references are explicit; no scores or narrative. Full domain contract: repository `docs/domains.md`.

Basic natal supports Gregorian UTC 1800–2399, tropical geocentric apparent Sun–Pluto, Placidus/Whole Sign houses, ASC/MC/DSC/IC and major aspects. Use UTC, not local civil time. Coordinates use degrees, north/east positive. Longitude is normalized and speed is degrees/day. Placidus polar failure returns an error without silently changing house systems. The compiled Moshier and time models require no external data files or downloads.

Source: https://github.com/7mlabs/sdk-astro. The engine/SDK use AGPL-3.0-only, choosing the free AGPL option of Swiss Ephemeris; complete LICENSE, NOTICE and third-party dependency notices are bundled. Distribution and integration must follow AGPL requirements; see repository docs/license.md. Platform evidence and installation instructions are in docs/testing.md, docs/packages.md and docs/node-release.md.

`natalDomains` trả một natal cá nhân (`chartKind: individualNatal`, `subjectCount: 1`). Đọc `data.context.advanced` cho trạng thái/chuỗi chủ tinh/configurations và `data.domains.<id>.report` cho các nhà liên quan, góc chiếu, facts và evidence phục vụ báo cáo. `love`/`relationships` đều dùng một người; operation này không nhận người thứ hai.

`customProfiles` accepts up to eight developer-defined selection profiles, each with up to eight report sections. Use `domains: []` for custom-only output in `data.customDomains`; builtins remain in `data.domains`. `data.profileCatalog` declares the ten builtin definitions and their thirty sections. Each `report.sections` item contains references to the merged report facts; see repository `docs/profiles.md`. Profile selectors share the same local engine, provider, house expansion and aspect rules.

`couple` takes two birth inputs in `personA`/`personB`, each with UTC/location and an optional independent house system. It calculates both natal charts locally and returns qualified points, all 676 cross relations, matched contacts, 10 body overlays in each direction, and 144 cross house-ruler relations. Six builtin domains (18 sections) and symmetric custom profiles organize the comparison. Cross contacts always return `applying: null`; two natal epochs do not describe a shared time evolution. Use repository `docs/couple.md` and the installed Node/Python/.NET couple examples. Individual `love`/`relationships` keep their one-person contract.

`composite` takes two birth inputs and builds a third, symbolic midpoint chart C. It returns source natals in `data.subjects` and C in `data.composite`, including all 10 individual-profile domains/30 sections, 26 points/325 angular relations/66 house-ruler relations, custom profiles and construction provenance. Geometry, rulers, report facts and patterns are recomputed for C. No fictional birth epoch, geographic location or planet motion is assigned: C speed/retrograde/applying are null. Default `houseMethod: "midpoint"` validates the 12 midpoint cusps; invalid ordering fails. An explicit `"wholeSignFromMidpointAscendant"` option builds Whole Sign houses from C ASC. Default antipodal policy fails on ambiguous midpoints; explicit `"lowerLongitude"` resolves them deterministically.

Alternatively add `composite: {}` to a `couple` request to reuse its two computed natals and receive C beside synastry. Child options select C house method/antipodal policy, domains and custom profiles; root rulership/aspect rules are inherited. Root couple domain IDs and child C domain IDs are different catalogs. Omitting `composite` preserves the previous couple payload shape. Use the same calculate API; repository `docs/composite.md` and installed `composite`/`couple-composite` examples show complete calls. Version 0.10.0-alpha.1 uses the experimental alpha channel.

`events` searches a civil day/month/year for ingress, longitude stations, primary Moon phases, exact planetary aspects and real global solar/lunar eclipses. `forecast` adds one `birth` natal and all exact transit contacts to its 26 fixed points. Period examples: `{ kind: 'day', year: 2026, month: 3, day: 3, utcOffsetMinutes: 420 }`, `{ kind: 'month', year: 2026, month: 3 }`, `{ kind: 'year', year: 2026 }`. The offset is fixed and caller-supplied; no IANA/DST conversion. Default aspect preset is major; preset and custom rules are mutually exclusive. Optional `bodies`, `eventTypes` and `includeNatalTransits` select the supported search. Phases/eclipses always use Sun/Moon when enabled, independently of the bodies filter.

Forecast returns one shared natal/context, a midpoint snapshot (default 260 transit-versus-fixed-natal relations), root-solved events with per-event natal house/ruler/contact evidence, 10 domain views/30 sections, custom profiles and calendar overview. `messageContext` contains references for daily/monthly/yearly reports; narrative is null. Snapshot applying uses transit speed against fixed natal coordinates. Global eclipse maxima do not establish local visibility. Highlight selection is a versioned policy, not a score. Each index resolves in its declared snapshot/event/natal context.

Search refines numeric roots, retains repeated retrograde hits and records precision; eclipse maxima use the Swiss eclipse API with no invented bracket/residual. Requests exceeding the explicit event/evaluation budget fail without truncation. Month/year scans can take seconds and create large payloads; native calculate is synchronous, so use application workers for interactive workloads. Repository `docs/forecast.md` defines schemas, limits, units, precision, selectors and joins. Dedicated Node/Python/.NET forecast examples use the installed package; Rust/C accept the same raw JSON. Native target requirements are listed above.

## Grouped on-demand queries

Version 0.9 adds `geometry`, `aspects`, `houses`, and `points` exports. Each method accepts one options object and returns the same full JSON envelope as `calculate`; calculation stays in Rust.

```js
const { aspects, houses } = require('@7mlabs/astrology');
const relation = aspects.between({ positions: [{ id:'a', longitude:350, speed:1 }, { id:'b', longitude:28 }],
  rule:{ angle:37.5, maxOrb:1 }, motionMode:'fixedSecond' });
const result = houses.inspect({ birth, houseNumbers:[7] });
```

Methods: geometry.normalize/separation/midpoint, aspects.between/inspect, houses.locate/inspect, points.inspect. Inspection uses a validated birth, computes the full natal/context internally, and returns the requested selection with self-contained support references. Routing keys operation/group/action are set by the helper. See repository docs/query.md and docs/integrations.md for exact request fields, local MCP and frontend-host integration. Native Node-API requires Node; browser-only React/Vue needs a separate WASM build.

## Payload context preparation

Version 0.10 adds a separate shared-core compressor with compact/focused/budgeted modes, explicit omission/coverage metadata, and a reversible compact representation. Calculation APIs remain synchronous and unchanged. See `docs/payload-compression.md` in the repository for the cross-language API, budget semantics, and complete examples. Native binaries from earlier versions do not export this feature.
