# .NET astrology package

Offline natal, synastry, composite, forecasts and astronomical event data backed by the same Rust/Swiss native core used by Node.js and Python.

```csharp
using SevenMLabs.Astrology;
var result = Engine.Calculate(new {
    operation = "natal",
    utc = new { year = 2000, month = 1, day = 1, hour = 12, minute = 0 },
    location = new { latitude = 10.8231, longitude = 106.6297 }
});
Console.WriteLine(result);
```

Restore the generated nupkg from the local artifacts feed. Managed package targets net10.0; native assets use `runtimes/<rid>/native`. `Engine.Calculate` returns an owned JsonElement and throws EngineException with Code/ResultJson. `Engine.CalculateJson` returns the raw envelope; native output is released in finally.

Operations: natal, natalDomains, couple, composite, events, forecast, chart, harmonic, synastry. `natalDomains` uses the same UTC/location input and returns `data.natal`, `data.context`, `data.domains` for career/love/relationships/family/finance/identity/learning/creativity/innerLife/dailyLife. It defaults to all ten domains, traditional rulers and extended aspect rules. Optional `domains` selects unique IDs; `rulership` selects traditional/modern; `aspectPreset` selects major/extended or supply `aspectRules` (mutually exclusive). Context includes all 325 angle relations among 10 bodies, 4 angles and 12 house cusps, house rulers/occupants and dispositors. Domain selection reasons and references are explicit; no scores or narrative. Full domain contract: repository `docs/domains.md`.

Basic natal supports Gregorian UTC 1800–2399, tropical geocentric apparent Sun–Pluto, Placidus/Whole Sign houses, ASC/MC/DSC/IC and major aspects. Use UTC, not local civil time. Coordinates use degrees, north/east positive. Longitude is normalized and speed is degrees/day. Placidus polar failure returns an error without silently changing house systems. The compiled Moshier and time models require no external data files or downloads.

Version 0.10.0 is a local package and has not been published to a public registry. Source: https://github.com/7mlabs/sdk-astro. The engine/SDK use AGPL-3.0-only, choosing the free AGPL option of Swiss Ephemeris; complete LICENSE, NOTICE and third-party dependency notices are bundled. Distribution and integration must follow AGPL requirements; see repository docs/license.md. Platform evidence and installation instructions are in docs/testing.md and docs/packages.md.

`natalDomains` trả một natal cá nhân (`chartKind: individualNatal`, `subjectCount: 1`). Đọc `data.context.advanced` cho trạng thái/chuỗi chủ tinh/configurations và `data.domains.<id>.report` cho các nhà liên quan, góc chiếu, facts và evidence phục vụ báo cáo. `love`/`relationships` đều dùng một người; operation này không nhận người thứ hai.

`customProfiles` accepts up to eight developer-defined selection profiles, each with up to eight report sections. Use `domains: []` for custom-only output in `data.customDomains`; builtins remain in `data.domains`. `data.profileCatalog` declares the ten builtin definitions and their thirty sections. Each `report.sections` item contains references to the merged report facts; see repository `docs/profiles.md`. Profile selectors share the same local engine, provider, house expansion and aspect rules.

`couple` takes two birth inputs in `personA`/`personB`, each with UTC/location and an optional independent house system. It calculates both natal charts locally and returns qualified points, all 676 cross relations, matched contacts, 10 body overlays in each direction, and 144 cross house-ruler relations. Six builtin domains (18 sections) and symmetric custom profiles organize the comparison. Cross contacts always return `applying: null`; two natal epochs do not describe a shared time evolution. Use repository `docs/couple.md` and the installed Node/Python/.NET couple examples. Individual `love`/`relationships` keep their one-person contract.

`composite` takes two birth inputs and builds a third, symbolic midpoint chart C. It returns source natals in `data.subjects` and C in `data.composite`, including all 10 individual-profile domains/30 sections, 26 points/325 angular relations/66 house-ruler relations, custom profiles and construction provenance. Geometry, rulers, report facts and patterns are recomputed for C. No fictional birth epoch, geographic location or planet motion is assigned: C speed/retrograde/applying are null. Default `houseMethod: "midpoint"` validates the 12 midpoint cusps; invalid ordering fails. An explicit `"wholeSignFromMidpointAscendant"` option builds Whole Sign houses from C ASC. Default antipodal policy fails on ambiguous midpoints; explicit `"lowerLongitude"` resolves them deterministically.

Alternatively add `composite: {}` to a `couple` request to reuse its two computed natals and receive C beside synastry. Child options select C house method/antipodal policy, domains and custom profiles; root rulership/aspect rules are inherited. Root couple domain IDs and child C domain IDs are different catalogs. Omitting `composite` preserves the previous couple payload shape. Use the same calculate API; repository `docs/composite.md` and installed `composite`/`couple-composite` examples show complete calls. Version 0.10.0 remains locally built; the stable npm release does not publish this binding to its registry.

`events` searches a civil day/month/year for ingress, longitude stations, primary Moon phases, exact planetary aspects and real global solar/lunar eclipses. `forecast` adds one `birth` natal and all exact transit contacts to its 26 fixed points. Period examples: `{ kind: 'day', year: 2026, month: 3, day: 3, utcOffsetMinutes: 420 }`, `{ kind: 'month', year: 2026, month: 3 }`, `{ kind: 'year', year: 2026 }`. The offset is fixed and caller-supplied; no IANA/DST conversion. Default aspect preset is major; preset and custom rules are mutually exclusive. Optional `bodies`, `eventTypes` and `includeNatalTransits` select the supported search. Phases/eclipses always use Sun/Moon when enabled, independently of the bodies filter.

Forecast returns one shared natal/context, a midpoint snapshot (default 260 transit-versus-fixed-natal relations), root-solved events with per-event natal house/ruler/contact evidence, 10 domain views/30 sections, custom profiles and calendar overview. `messageContext` contains references for daily/monthly/yearly reports; narrative is null. Snapshot applying uses transit speed against fixed natal coordinates. Global eclipse maxima do not establish local visibility. Highlight selection is a versioned policy, not a score. Each index resolves in its declared snapshot/event/natal context.

Search refines numeric roots, retains repeated retrograde hits and records precision; eclipse maxima use the Swiss eclipse API with no invented bracket/residual. Requests exceeding the explicit event/evaluation budget fail without truncation. Month/year scans can take seconds and create large payloads; native calculate is synchronous, so use application workers for interactive workloads. Repository `docs/forecast.md` defines schemas, limits, units, precision, selectors and joins. Dedicated Node/Python/.NET forecast examples use the installed package; Rust/C accept the same raw JSON. Local package candidates have been verified on macOS ARM64 and Linux x64 in CI. This binding remains unpublished; Python wheels and NuGet runtime assembly require their own public release validation.

## Grouped on-demand queries

Version 0.9 adds nested static groups under Engine: Geometry.Normalize/Separation/Midpoint, Aspects.Between/Inspect, Houses.Locate/Inspect and Points.Inspect. Each accepts one serializable options object and returns the full JSON envelope.

```csharp
var house = Engine.Houses.Inspect(new { birth, houseNumbers = new[] { 7 } });
var angle = Engine.Geometry.Separation(new { longitude1 = 359, longitude2 = 1 });
```

All calculation remains in Rust; birth inspection computes natal/context then selects self-contained requested facts. Routing keys operation/group/action are set by the helper. See repository docs/query.md and docs/integrations.md. Desktop/native .NET can host frontend integrations; browser-only Blazor cannot load this native P/Invoke binary without a separate browser runtime.

## Payload context preparation

Version 0.10 adds a separate shared-core compressor with compact/focused/budgeted modes, explicit omission/coverage metadata, and a reversible compact representation. Calculation APIs remain synchronous and unchanged. See `docs/payload-compression.md` in the repository for the cross-language API, budget semantics, and complete examples. Native binaries from earlier versions do not export this feature.
